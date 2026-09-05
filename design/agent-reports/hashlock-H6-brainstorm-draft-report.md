# REPORT — Hashlock H6 brainstorm draft: what the operator's nine decisions do NOT settle

Companion to `design/BRAINSTORM_hashlock_H6_preimage_plates.md` (written to the
brief `design/agent-briefs/hashlock-H6-brainstorm-draft-brief.md`). Citations
measured at engrave `07a92e9b`, seedhammer fork main `fb0dd04`, mnemonic-secret
`504ff46`. Numbers labelled MEASURED were produced by running the fork's own
`qr.Encode`, `engrave.ConstantQR`, `backup.CharsPerLine` and
`backup.LinesPerPlate` against a scratch module at `fb0dd04`.

Two parts. **Part A** is the consolidated list of unsettled questions, most
consequential first, each with options and a one-line recommendation. **Part B**
is what I believe is unsound in the fixed nine — recorded because the brief asks
for it, not to reopen anything.

---

# PART A — UNSETTLED QUESTIONS

## A1. The QR (spec §5) — the one that blocks the stage

**Q5.1 — decision 2's QR text does not fit the device's constant-time QR
engraver. Which exit?**
MEASURED at `fb0dd04`: `engrave.ConstantQR` refuses any code over 37 modules
(`engrave/engrave.go:418-426`), which at ECC-L is **106 bytes**. Decision 2's
text with a 100-character phrase measures 194 bytes / 53 modules for the full
pbkdf2 method line and 135 bytes / 45 modules for `method: sha256` — both
refused. Even `phrase: ` + 100 characters is 108 bytes / 41 modules and refused.
Only a bare 100-byte phrase (today's passphrase plate) fits.
- **(a)** Raise `bitmapForQRStatic` + `ConstantQR` to v6–v9 as the code's own
  comment invites ("Raise both together or not at all"); v7+ needs six alignment
  patterns rather than one (`engrave/engrave.go:401-412`), so it is a real change
  in `engrave` with goldens and a fresh constant-time argument.
- **(b)** Use `engrave.QR` (`engrave/engrave.go:277`), which has no size bound —
  and engraves content-dependently, leaking the phrase through the toolpath,
  which `backup/passphrase.go:112-114` forbids for exactly this material.
- **(c)** Shrink the QR payload below 107 bytes: a shorter method token, fewer
  labels, or a lower phrase cap for the QR form only.
- **Recommend (a)**, scoped to v6 and v7 with a measured module ceiling stated in
  the spec; (b) trades a funds secret for convenience and (c) undoes decision 2.

**Q5.6 — does the phrase form use `SpaceMark` and its legend?** A phrase is
printable ASCII with real spaces (`hashlock/hashlock.go:92-111`) and the verbatim
rule is as strict as the passphrase's.
- **(a)** Reuse `backup.SpaceMark = '\x1f'` and `passphraseLegend`
  (`backup/passphrase.go:21,168`) verbatim; **(b)** plain spaces plus a character
  count in the header; **(c)** both.
- **Recommend (a)** — "one space and two look identical" is the passphrase
  plate's own reason and it holds identically here.

**Q5.8 — does the ms1-string plate get a QR at all?** Decision 1 says the QR
toggle is offered only with the phrase form.
- **(a)** No QR, as decided; **(b)** allow a QR of the ms1 string on that form
  too (it is the harder of the two to transcribe: 75 bech32 characters, no word
  boundaries); **(c)** no QR, but say so in the spec with the reason.
- **Recommend (c)** unless Part B item D changes the operator's mind.

**Q5.9 — fixed em or the `FontSizes` ladder?** The passphrase plate pins 6.0 mm
and does not scale with length (`backup/passphrase.go:54-61`); the free-text
plate walks the ladder (`backup/backup.go:83`).
- **(a)** Ladder, auto-fit, as decision 5's "smallest rung" implies; **(b)** two
  fixed rungs (with QR / without), the passphrase plate's shape.
- **Recommend (b)** — two measured rungs are two goldens; a ladder is six.

## A2. The host record and sealing (spec §2)

**Q2.3 — is the new preimage class SECRET?** `decide_sealing`
(`crates/me-cli/src/main.rs:2353-2410`) seals iff some record's class
`is_secret()`; the device mirror is `Class.IsSecret()` (`sysw/record.go:60-66`).
There is no third branch.
- **(a)** Secret → `me sysw pack` SEALS by default, which decision 3 puts out of
  scope; the operator must pass `--no-passphrase` to get the payload H6 consumes.
- **(b)** Not secret → the payload stays unsealed and a spend secret sits in
  flash in cleartext, which is what decision 7's transit warning names.
- **(c)** Secret, with `--pack-preimage` also suppressing the seal for this class.
- **Recommend (b)** as the reading that matches decisions 3 and 7 — and say so in
  the spec as a deliberate choice, with `decide_sealing`'s stderr line naming the
  class so the operator sees why the container is cleartext.

**Q2.1 — the phrase record's wire shape.** The siblings are not uniform:
`hash:` is raw 64 lowercase hex, `key:` and `now:` are hex of a UTF-8 text with
`,` as `now:`'s internal separator (`sysw/composer_records.go:24-31,100,136`).
- **(a)** `preimage-phrase:<hex of "<method>,<phrase>">` — the `now:` idiom, one
  separator, one record.
- **(b)** `preimage-phrase:<hex of decision 2's exact labelled text>` — one string
  with two consumers (wire and QR), so they cannot drift.
- **(c)** Two records, `preimage-method:` and `preimage-phrase:`.
- **Recommend (b)**: it makes decision 2's spelling the single normative form and
  removes Q2.4 entirely; the cost is a longer body and a parser that must accept
  its own output.

**Q2.4 — how is a method spelled on the wire?** Four spellings exist today:
`ms hashlock`'s card line (`crates/ms-cli/src/cmd/hashlock.rs:281-288`), its
`--json` object (`:322`), the device's `hashlockMethod.String()` →
`sha256`/`hardened` (`gui/composer_hashlock.go:35-40`), and decision 2's QR line.
- **(a)** Decision 2's QR spelling is normative everywhere; **(b)** the short
  token (`hardened`/`sha256`) on the wire, the long line on the plate; **(c)**
  the `--json` object.
- **Recommend (a)** — one spelling, and it is the one already fixed.

**Q2.2 — is the ms1 preimage form a class or a prefixed record?**
- **(a)** Give the ms1 single a class, which means `isStrictMs1`'s H0 clause
  (`sysw/classify.go:126`) stops answering `false` globally and only
  `gui/sysw_admit.go:64-73` keeps it out of other programs.
- **(b)** A prefixed `preimage:<64 hex>` record that never carries an ms1 string,
  leaving H0 untouched everywhere.
- **Recommend (a)** — (b) is safer but the operator's hand holds an ms1 string
  from `ms hashlock --out`, and forcing a hex re-encode moves work onto them.

**Q2.5 — does the DEVICE re-derive a payload phrase's X?** ~10 s hardened.
- **(a)** Yes, at pick time, so the "digest matches no `hash:` record" check is
  real on-device; **(b)** no, trust the record and rely on the host warning;
  **(c)** yes, but only when the operator asks to cut the plate.
- **Recommend (c)** — pays the 10 s once, at the moment it decides a plate.

**Q2.6 — does `--pack-preimage` also relax `me seal`?** Decision 3 says the
sealed payload is out of scope. **(a)** No, and the two refusal texts diverge;
**(b)** yes, symmetric. **Recommend (a)**, with the divergence stated in
`crates/me-cli/src/seal/record.rs:130-136`'s comment.

**Q2.7 — does the auto-`now:` rule extend to preimage records?** Today it fires
on `key:`/`hash:` (`crates/me-cli/src/main.rs:1730-1760`). **(a)** Extend;
**(b)** leave. **Recommend (a)** — a preimage payload is a composer payload.

## A3. Retention and the Done review (spec §4)

**Q4.1 — which "Done"?** The shape's Done (`gui/composer_shape.go:434-445`) or
the engrave Done (`gui/composer_flow.go:335-394`)?
- **(a)** The engrave Done, with the review folded into the census screen
  (`confirmReviewScreen(ctx, th, "Plates To Cut", …)`, `:389-390`), whose counts
  already derive through `bundlePlatePlan` — the same function `bundleEngrave`
  loops (`gui/composer_census.go:9-18`).
- **(b)** The shape's Done, beside §8h.
- **Recommend (a)** — a review that names plates belongs where the plates are
  counted, and (b) would name plates before the form pick exists.

**Q4.2 — what happens to a plate the operator declines?**
- **(a)** Dropped from the bundle, census recounted and re-shown; **(b)** the
  whole cut aborted, `bundleEngrave`'s existing set-level rule
  (`gui/bundle_flow.go:625-631`: "a partial bundle can't be used"); **(c)**
  remembered, so Back and forward reproduce the choice.
- **Recommend (a) + (c)**: a hashlock plate is not part of the policy's card set,
  so (b)'s reasoning does not reach it.

**Q4.3 — how many hashlock plates per bundle?**
- **(a)** One per distinct preimage; **(b)** one per path that carries a
  phrase-set hash; **(c)** exactly one, refusing a second phrase.
- **Recommend (a)** — `hashlockOtherPathLine` (`gui/composer_hashlock.go:114-130`)
  already rules a second phrase legal, and (c) would retract that.

**Q4.4 — form and QR per plate, or once for all hashlock plates?**
`bundleEngrave` asks per plate (`gui/bundle_flow.go:616-657`).
- **(a)** Per plate, matching the shipped shape; **(b)** once.
- **Recommend (b)** when there is one plate and (a) when there is more than one,
  stated as that rule rather than as two screens.

**Q4.5 — how is "not on any path, will not be cut" delivered?** **(a)** A row on
the review; **(b)** its own screen; **(c)** a refusal to enter the flow when
NOTHING will be cut. **Recommend (a) + (c)**.

**Q4.6 — is the material retained across a Back out of the engrave step?**
**(a)** Yes, for the composition's lifetime as decision 9 says; **(b)** dropped
at the first exit from the engrave step. **Recommend (a)** — decision 9 settles
the lifetime; this is only its consequence.

**Q4.7 — does the census distinguish a hashlock plate from the public plates?**
`bundlePlateMark` never marks a `cardMS1` (`gui/bundle_flow.go:566-575`).
**(a)** Same rule, never marked; **(b)** marked with the policy stub.
**Recommend (a)**.

**Q1.1 — store the phrase bytes, or only X and the method?** **(a)** Phrase +
method + X (decision 1's "only when the device knows the phrase" implies it);
**(b)** X + method only, and the phrase form is offered only in the same
composition where it was typed, from a screen-local copy. **Recommend (a)**.

**Q1.2 — keyed by digest or carried on the path?** **(a)** By digest, mirroring
`phraseDigests` (`gui/composer_state.go:34-53`); **(b)** on `md.SpendPath`.
**Recommend (a)** — the C16 index lesson applies unchanged.

**Q1.3 — what happens when a path's hash moves off a phrase digest?** **(a)**
Nothing deletes, and the review's "not on any path" line covers it (extending
that line to the composer-native case too); **(b)** delete on edit.
**Recommend (a)**, and extend the line — decision 6 wrote it for the
payload-delivered case only.

**Q1.4 — does path (b) material live in `composerState`?** **(a)** No; the
"Hashlock plates" flow reads `ctx.sysw` and holds nothing, which is what
decision 4's "without a composition" implies; **(b)** yes, a shared store.
**Recommend (a)**.

**Q1.5 — may one composition hold more than one phrase?** **(a)** Yes, as today
(`gui/composer_hashlock.go:114-130` warns and proceeds); **(b)** refuse a second.
**Recommend (a)** — (b) is a new refusal in an area H2 deliberately left as a
warning under ruling L12.

## A4. Device admission and the screens (spec §3)

**Q3.1 — does `Which hash?` show a payload PHRASE record before deriving it?**
- **(a)** Row shows "phrase record N (derive to see the digest)" and derives on
  pick; **(b)** derive every phrase record when the screen builds (10 s each,
  hardened); **(c)** show the digest only when the record also carries one.
- **Recommend (a)** — the screen must not block, and `composerHashRow`'s elided
  `first8..last8` form (`gui/composer_hash.go:38-41`) has no value to print yet.

**Q3.2 — does a payload preimage digest join `phraseDigests`?** **(a)** Yes, so
§8h's phrase form fires; **(b)** no, and §8h gains a third form; **(c)** no, and
§8h's plain form is left as is. **Recommend (c)** — H5 §2.5 already recorded that
the phrase form overcounts in the safe direction and that counting exactly would
need three variants.

**Q3.3 — is the "Hashlock plates" door row conditional?** **(a)** Only when the
payload holds preimage/phrase records, gated like `composerDoorHasConsumablePolicy`
(`gui/composer_door.go:93-97`); **(b)** always, with a lead saying why it is
empty. **Recommend (a)** — F-437's rule is that a door row names the route it
takes, and a row that dead-ends breaks it.

**Q3.5 — what identifies the policy a payload preimage belongs to?** **(a)** The
matching `hash:` record's index; **(b)** nothing, and the header says so; **(c)**
a new field on the record. **Recommend (a) + (b)** — print the match when there
is one and say "no hash record in this payload has this digest" when there is
not, mirroring `hashlockRelationLine` (`gui/composer_hashlock.go:91-108`).

**Q5.3 — what is "policy csid"?** The composer's identity surface is
Template-ID / Policy-ID (32 hex) and `mk1 stub (template)` / `mk1 stub (policy)`
(8 hex) (`gui/composer_stub.go:30-72`). `md`'s literal chunk-set id is 20 bits,
unexported (`md/identity.go:31-33`) and present only in a chunked md1 header
(`md/chunk.go:45-52`); the only exported `csid` is `mk`'s per-card
`chunk_set_id` (`gui/bundle.go:43-51`).
- **(a)** The 8-hex `mk1 stub (policy)`; **(b)** the 32-hex Policy-ID; **(c)** the
  20-bit md1 chunk-set id.
- **Recommend (a)**, spelled with the stub screen's own label, so the operator can
  match plate to card. (c) is not stable for a single-chunk md1 and (b) is 32 hex
  characters on a plate that is already tight.

**Q5.4 — is a policy id available at Done at all?** A key-less or partially
seated composition has none (`gui/composer_engrave.go:40-61`: "no id yet";
`gui/composer_stub.go:62-72` adds the keyed pair only when keyed chunks exist).
- **(a)** Print the TEMPLATE stub when there is no policy stub, labelled as such;
  **(b)** leave the field empty; **(c)** refuse to cut a hashlock plate from a
  composition with no policy id.
- **Recommend (a)** — (c) would refuse the C26 key-less template, which is a
  first-class composition.

**Q5.5 — where does the flow form's "template id" come from?** The
payload's md1/mk1 reach the composer through `composerCardSources`, not through a
flow that runs "without a composition". **(a)** Read the payload's `ClassMDMK`
records directly in the flow; **(b)** omit the field there. **Recommend (a)**.

**Q3.4 / Q3.6** are folded into Q2.2 and Part B item H respectively.

## A5. The free-text and passphrase warning (spec §6)

**Q6.1 — which predicate?** **(a)** `hashlock.IsMS1Shaped`
(`hashlock/hashlock.go:122-148`): shape only, no checksum, catches the GROUPED
plate an operator retypes from `ms hashlock`'s card; **(b)**
`codex32.IsPreimage` (`codex32/mspayload.go:94-101`): checksum-bearing, catches
only a clean string; **(c)** both, with different copy.
- **Recommend (a)** — H2 §2 rule 3 chose the shape test for precisely this
  reason, and reusing it keeps host and device agreeing on what an ms1 string is.

**Q6.2 — what does the warning claim?** **(a)** "This looks like an ms1
string" — true under (a) above, vague; **(b)** "This looks like a hashlock
preimage plate" — needs the checksum and misses the grouped case; **(c)** (a)'s
claim plus a sentence naming the H6 route.
- **Recommend (c)** — H2 §2 rule 3's refusal names the route that exists; a
  warning that names none teaches the operator to tap through.

**Q6.3 — same warning in Engrave Password?** **(a)** Yes, same body; **(b)** yes,
different body (there the string becomes a wallet's passphrase, not a plate);
**(c)** free text only. **Recommend (b)**.

**Q6.4 — where in the flow?** **(a)** At OK on the text entry screen; **(b)** at
the existing confirm screen (`gui/freetext_flow.go:1362-1402`). **Recommend (a)**
— earlier is cheaper, and the confirm screen is already paged.

**Q6.5 — tap or HOLD?** **(a)** Tap, the free-text program's idiom; **(b)** HOLD,
the composer's (`gui/composer_copy.go:36-38`). **Recommend (a)** — decision 8 says
"never refuse", and a HOLD in a program that has none reads as a refusal.

**Q6.6 — does it fire for payload-sourced free text?**
(`gui/freetext_flow.go:1485`.) **(a)** Yes; **(b)** no, the host already had a
chance to refuse. **Recommend (a)** — the host's chance is not a check that ran.

## A6. Tests, gates and acceptance (spec §7)

**Q7.1 — which corpus owns the QR text?** **(a)** The ms hashlock corpus
(`crates/ms-codec/tests/vectors/hashlock-v0.8.json`), re-pinning its sha and the
fork's vendored copy; **(b)** the record-class corpus
(`sysw/testdata/record_class_vectors.json`, 47 rows, sha
`5b3960…b312`). **Recommend (a)** for the QR text (it is `ms`'s format to parse
later) and (b) for the new record class.

**Q7.2 — acceptance for a plate nobody can read back on-device?** The SH2 has no
camera. **(a)** Phone scan of the QR plus a host re-entry of the typed phrase;
**(b)** host re-entry only; **(c)** golden bytes only, live read-back deferred to
the operator's walk. **Recommend (a)**, with (c) as the CI gate.

**Q7.3 — does the walk reach the plate?** H5 §4.1's doctrine is that a walk may
READ state only. `freetextPlateHook` / `passphrasePlateHook`
(`gui/freetext_flow.go:1416`, `gui/passphrase_flow.go:549`) are the shipped
answer for plate output — and a third such hook would carry a SECRET.
- **(a)** A plate hook, `!tinygo`-guarded like the H5 composition-state seam;
  **(b)** assert the confirm/review screen only and leave the plate to goldens.
- **Recommend (b)** — a production-absent hook that carries a preimage is a
  bigger claim than the walk needs.

**Q7.4 — mutation gate for the digest-vs-`hash:` warning?** **(a)** Both sides,
each with a stated mutation; **(b)** host only. **Recommend (a)**.

**Q7.5 — does `me bundle --preview` gain a hashlock plate?** The sidecar renders
public plates and `me` "never passes secret material to it"
(`crates/me-cli/src/preview.rs:1-5`). **(a)** No, stated in the spec; **(b)** yes,
for the string form. **Recommend (a)**.

---

# PART B — WHAT I BELIEVE IS UNSOUND IN THE FIXED DECISIONS

Recorded because the brief asks. Item B1 is a measured contradiction; the rest
are judgement calls.

## B1. Decisions 1, 2 and 5 are jointly unsatisfiable on the shipped device (MEASURED)

Decision 1 puts the QR on the phrase form, so its content is always a SECRET.
Decision 5 fixes the phrase cap at 100 characters. Decision 2 fixes the QR text
as labelled lines with the method named in full.

The only constant-time QR the device has refuses anything over 37 modules —
`engrave/engrave.go:418-426`, whose comment states the bound's origin: *"ECC-L
caps at 106 bytes and the passphrase caps at 100 (spec O6). bitmapForQRStatic
tabulates 21/25/29/33/37 only ... Raise both together or not at all."* MEASURED
with the fork's own `qr.Encode` at ECC-L:

| payload | bytes | modules | `ConstantQR` |
| --- | --- | --- | --- |
| bare 100-char phrase (today's passphrase plate) | 100 | 37 | accepted |
| `phrase: ` + 100 chars | 108 | 41 | **refused** |
| decision 2, `method: sha256`, 100-char phrase | 135 | 45 | **refused** |
| decision 2, full pbkdf2 method line, 100-char phrase | 194 | 53 | **refused** |

The alternative engraver, `engrave.QR` (`engrave/engrave.go:277`), has no bound
and is content-dependent — which `backup/passphrase.go:112-114` forbids for this
exact material: *"ConstantQR, never engrave.QR: the latter engraves in a
content-dependent pattern and would leak the secret through timing."* So the
three decisions cannot all hold without changing `engrave`. Q5.1 lists the exits;
this is not a preference, it is a build failure waiting at implementation time.

## B2. "QR on the right" is new geometry and fights decision 5's own fit target (MEASURED)

The only secret-QR plate the device has stacks its QR BELOW the text
(`backup/passphrase.go:283-292`, `l.envY = l.textY + l.blockH + gap` at `:289`).
Putting it on the right narrows the text column: usable width is 85 − 2×3 = 79 mm,
and a 53-module QR at the passphrase plate's scale 3 is 47.70 mm (MEASURED),
leaving 29.3 mm ≈ **14 characters per line at the 3.0 mm rung** — against 39 at
full width (MEASURED via `backup.CharsPerLine`). A 100-character phrase is then 8
lines and the 73-character method line another 6, before the title band and the
locator header. Below the QR, the same content is 3 and 2 lines. If the QR must
be beside the text, the spec should carry the measured column width and the
resulting line counts, not the phrase cap alone.

## B3. Decision 3's "unsealed payload" and decision 7's transit warning presume a branch that does not exist

`decide_sealing` (`crates/me-cli/src/main.rs:2353-2410`) has exactly two
outcomes and picks by content. A preimage record is either secret — and the
container SEALS, which is the case decision 3 excludes — or not secret, and a
spend secret is written to flash in cleartext with a warning as its only
mitigation. Decision 7 assumes the second without saying so, and the sealing line
`me` prints will then read *"NOT SEALED — no record in this payload is secret
material"*, which is false of the payload the operator just built. Whichever way
Q2.3 goes, that sentence needs an arm.

## B4. Decision 5's title band is false of the phrase-form plate

`HASHLOCK PREIMAGE` / `NOT A SEED` is exactly right for the string form: that
plate carries X. The phrase form carries the phrase and the method, from which X
is *derived* — it does not carry a preimage. This is the same shape of defect H5
§1.2 folded in the confirm modal, where adding "and this digest" to a list of
things "not on your plates" made the claim FALSE of the one item that IS on them.
Suggest the phrase form's band read `HASHLOCK PHRASE` / `NOT A SEED`, with the
method line immediately under it — one word, and it stops the plate asserting
something it is not.

## B5. Decision 1 gives the QR to the form that needs it least

The ms1 string is 75 bech32 characters with no word boundaries and a BCH checksum
the operator cannot verify by eye; a hashlock phrase is printable ASCII a person
chose. Decision 1 gives the machine-readable copy to the second and withholds it
from the first. If B1's exit is (a) — raising `ConstantQR` — the string form's QR would
cost nothing at all: a 75-byte ms1 string measures **33 modules** at ECC-L
(MEASURED; 33 modules is already inside today's `ConstantQR` bound), so it needs
no change to `engrave` whatever B1's exit turns out to be. Worth reconsidering only if B1 goes that way.

## B6. Decision 8 warns and then routes nowhere

Decision 8's confirm names what the text looks like and cuts as typed. With H6
shipping a dedicated preimage plate in the same firmware, an operator who reaches
Engrave Text with an ms1 preimage string is one screen away from the right tool
and is told only that their text looks like an ms1 string. H2 §2 rule 3 set the
precedent for the opposite: its refusal names the route that exists (*"On the
host, run ms hashlock with it and load the hash: record it prints."*). The
warning should name the H6 route. This does not change decision 8 — it is still a
warning, still never a refusal — it only adds a sentence.

## B7. Decision 4 names a field the composer does not have

"Policy csid" has no referent in this tree. The composer prints Template-ID,
Policy-ID and two `mk1 stub` values (`gui/composer_stub.go:30-72`); `md`'s
chunk-set id is 20 bits, unexported and chunked-only (`md/identity.go:31-33`,
`md/chunk.go:45-52`); `csid` as an exported word belongs to `mk` cards
(`gui/bundle.go:43-51`). See Q5.3 — the plate must use whatever label the stub
screen uses, or the operator cannot match one to the other.

## B8. H6 reverses the polarity of a collision H0 accepted (funds-relevant)

`codex32.IsPreimage` does not consult the id: a plain BIP-93 33-byte seed
beginning `0x03` is indistinguishable from a preimage plate — *"roughly 1 in 256
of 33-byte seeds"* — and is refused
(`codex32/mspayload.go:78-101`). H0 accepted that deliberately, on a stated
trade: *"A refusal costs a re-encode; a wrong cut exposes a spend secret."* H6
inverts the consequence. The same misclassification now routes such a string INTO
a plate flow that engraves it under a band reading `HASHLOCK PREIMAGE` /
`NOT A SEED`, with a header digest computed as `sha256` of something that is not
a preimage — a person's seed, labelled as not a seed, on steel. The trade that
justified the collision no longer holds in the direction H6 travels, and the spec
should either re-derive it or add the id check (`hash`) on the H6 admission path
only, leaving H0's kind-byte rule untouched everywhere else.
