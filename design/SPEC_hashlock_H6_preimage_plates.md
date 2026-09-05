# SPEC — Hashlock H6: preimage plates on the SeedHammer II

**STATUS: DRAFT -- R0 pending (0 lenses run).** Written from
`design/BRAINSTORM_hashlock_H6_preimage_plates.md` against the operator's
rulings of 2026-09-05 (decisions 1-9 of
`design/agent-briefs/hashlock-H6-brainstorm-draft-brief.md`, and the Group A/B
rulings that settled `design/agent-reports/hashlock-H6-brainstorm-draft-report.md`
and `design/agent-reports/hashlock-H6-brainstorm-journey-questions.md`). Nothing
here has been reviewed. **Every number in this spec is a measurement of the text
and the geometry as written**, taken on the scratch gate tree
`/scratch/code/shibboleth/.tmp/h6-gate` (a copy of fork main `fb0dd04`); the
capture is `design/agent-reports/hashlock-H6-spec-author-report.md` §1.
Citations measured at engrave `07a92e9b`, fork main `fb0dd04`, mnemonic-secret
`504ff46`; re-grep at plan time.

**What this stage does, in one sentence.** It puts a hashlock preimage on steel:
either as the ms1 kind-`0x03` string `ms hashlock` prints, or as the phrase and
its method in plain text, on a plate layout of its own that says `NOT A SEED`.

**What it reverses, named so nothing is folded silently.** Ruling L7 scoped the
device to the digest alone -- *"It never stores, shows, engraves or sources a
preimage"* -- and H2 implemented that literally: `hashlockPhraseRoute` derives X
on the stack and drops it when the function returns
(`gui/composer_hashlock.go:43-86`). H6 lifts three of those four verbs (store,
show, engrave) and leaves the fourth (source: reading a preimage plate back into
a seed flow) refused. Three shipped records become false and are rewritten by
THIS stage:

1. `gui/composer_hash.go:27-28` -- *"THE COMPOSER DERIVES A PREIMAGE IN RAM FOR
   ONE SCREEN (H2) AND NEVER STORES, SHOWS OR ENGRAVES IT."*
2. `gui/composer_hashlock.go:17-20` -- *"The preimage lives on the stack here and
   is dropped when this function returns (L7, L15)."*
3. `SPEC_wallet_policy_composer.md` §6c and its §14 row -- *"From H2 the composer
   derives a preimage in RAM for the length of one screen and never stores, shows
   or engraves it."*

`codex32/mspayload.go:63-93`'s `IsPreimage` header keeps its second half (*"a
hashlock preimage is not a seed; engraved as one it exposes a spend secret as a
backup"*) -- still true, and still the reason the plate this stage cuts is not a
seed plate -- and loses its *"the device learns to USE a preimage in stage H2,
not here"* sentence, which H6 makes false.

---

## §1. Scope

**In:**

1. Two plate FORMS on one dedicated layout (§6): the ms1 preimage string
   (text only), and the phrase with its method (optional QR).
2. Raising the constant-time QR encoder from v5 to **v9** so the decided QR text
   can be cut without leaking the phrase through the toolpath (§7).
3. A new payload record class for the ms1 preimage string, and a NEW
   `phrase:` record for a phrase and its method -- Rust first, with corpus rows
   (§3), ported to Go (§4).
4. `me sysw pack --pack-preimage`, the sealing consequence, and four host
   warnings (§3).
5. The `hash` id requirement on the H6 admission path, both sides (§4.3).
6. `Which hash?` gains preimage and phrase rows (§5.1); a **Hashlock plates**
   flow under the Wallet Policy door (§5.2); the Done review, folded into the
   `Plates To Cut` census (§5.3).
7. Retention of phrase, method and preimage for the composition's lifetime,
   keyed by digest, scrubbed by the flow-exit defer (§2.2).
8. `PREIMAGE REQUIRED` marking on the composer's md1 plates (§10.3); the abort
   arm (§8.4); provenance-aware §8h and reconcile copy (§10.1, §10.2).
9. The free-text and passphrase ms1 warning (§9).

**Out (§13):** reading a preimage plate back into any seed flow; the sealed
**Sealed Payload** container (`me seal`), whose refusal stands unchanged; the
typed `M*1 STRING` door; `ms hashlock` learning to parse the QR text (§8.6 pins
the text; the parser is a follow-on); `ms split` of a preimage (F-468); the
salt/iteration parameters (F-469); a words-plus-SeedQR secret plate (F-455); a
concrete-descriptor plate (F-457).

---

## §2. The two paths, the state model, and retention

### §2.1 The two paths

| | **(a) composer-native** | **(b) payload-delivered** |
| --- | --- | --- |
| entry | the phrase route at HOLD (`gui/composer_hashlock.go:69-70`) | an unsealed or unlocked `me sysw pack` payload |
| holds | phrase bytes, method, X, H | X and H (ms1 form); phrase, method, X and H (`phrase:` form) |
| lifetime | one `composerFlow` call | the session's records (`gui/sysw_session.go:54`) |
| cut by | the composer's engrave step (§5.3) | the Hashlock plates flow (§5.2) **or** the composer, when a path carries the digest |
| in flash? | never | **yes** -- cleartext when unsealed, ciphertext when sealed (§3.4) |

### §2.2 Retention -- NORMATIVE

`composerState` (`gui/composer_state.go:26-79`) gains ONE field, beside
`phraseDigests` (`:53`) and keyed the same way:

```go
// hashlockHeld is the material THIS COMPOSITION may cut onto a preimage
// plate, keyed by digest exactly as phraseDigests is (H5 §2's C16 reasoning:
// "Remove path" splices the slice, so an index is not an identity).
hashlockHeld map[[32]byte]hashlockMaterial
```

where `hashlockMaterial` carries `phrase []byte`, `method hashlockMethod`,
`preimage [32]byte` and `provenance` (`hashlockFromPhrase` |
`hashlockFromPayload`). Rules:

1. **One insertion site**, `composerHoldHashlockMaterial(st, h, m)`, which
   ALLOCATES when the map is nil. `composerState` is built as a zero-value
   struct literal at its one production site (`gui/composer_flow.go:48`) and in
   every test, so the map arrives nil and an assignment into a nil map panics --
   in the GUI goroutine, at the moment the operator holds to confirm a hash that
   gates funds. This is `composerNotePhraseDigest`'s rule
   (`gui/composer_state.go:280-286`) applied to the second map, and it is
   normative for the same demonstrated reason.
2. **Nothing deletes.** A digest no path carries is not removed; it is REPORTED
   (§5.3 item 4). Deletion is what `phraseDigests`' own comment
   (`gui/composer_state.go:46-50`) refuses, and the same argument holds here.
3. **The scrub is the flow-exit defer.** `composerFlowExit(st)`
   (`gui/composer_flow.go:20-23`, installed at `:59` before any secret can
   exist) gains `composerScrubHashlockHeld(st)`, which `wipeBytes` every
   `phrase` and zeroes every `preimage`. **It goes in the EXISTING defer, not
   beside it:** `composerFlowExit`'s own comment records that a second
   `defer` costs 96 B of firmware flash because TinyGo removes the empty stub's
   CALL and not the defer bookkeeping around it.
4. **Retention survives a Back out of the engrave step.** `composerEngraveStep`
   returning false sends `composerFlow` round its loop
   (`gui/composer_flow.go:47-131`) with the state intact; the material is gone
   only when `composerFlow` returns or power is lost. §8.4's abort copy says
   exactly that and no more.
5. Secret-handling defects in this field are non-gating (operator ruling
   2026-08-27) and are logged as follow-ups. F-483 already records that the
   typed phrase lives in `kbd.Fragment`, an immutable Go string, before H6
   stores anything.

### §2.3 Payload-delivered material is NOT copied into `composerState`

The Hashlock plates flow (§5.2) reads `ctx.sysw` and holds nothing: decision 4
says it cuts "without a composition". When the composer takes a payload preimage
onto a path (§5.1), the material IS entered into `hashlockHeld` with
`provenance = hashlockFromPayload`, because the composer's review and abort copy
must speak about everything it may cut.

---

## §3. Host: the records, `me sysw pack`, sealing -- Rust first

### §3.1 The two carriers -- NORMATIVE

| carrier | wire form | why this shape |
| --- | --- | --- |
| **preimage** | the bare ms1 kind-`0x03` string, id `hash`, 75 characters | it is what `ms hashlock --out` writes (`crates/ms-cli/src/cmd/hashlock.rs:299-306`); a prefixed re-encoding would make the operator transform a file they already hold |
| **phrase** | `phrase:<hex of the UTF-8 text "<method>,<phrase>">` | the `now:` idiom exactly (`sysw/composer_records.go:136-160`): hex of a UTF-8 text, one `,`, cut on the FIRST comma so the phrase may contain commas |

`<method>` is `hardened` or `sha256` -- the spelling
`hashlockMethod.String()` already uses (`gui/composer_hashlock.go:35-40`) and
`ms hashlock --method` already accepts. **The wire carries a method SELECTOR and
the plate carries the method DEFINITION (§8.6), and the difference is
deliberate:** a wire record is read by a tool that already knows the parameter
set, while a plate is read by a person who may have neither the tool nor this
firmware.

Prefix reservation follows `sysw/record.go:14-21` and
`sysw/composer_records.go:24-31`: `phrase:` joins `key:`, `hash:`, `now:` as a
RESERVED prefix, so a `phrase:` record whose body is not valid lowercase hex is
`ClassUnknown` and refused rather than treated as free text.

**Body validation, each failure `ClassUnknown` and refused with its own line:**
the body must be even-length lowercase hex (`unhexLower`,
`sysw/composer_records.go:82-97`); the decoded bytes must be valid UTF-8; the
text must contain at least one `,`; the field before the first `,` must be
exactly `hardened` or `sha256`; everything after it must pass the phrase rule of
`SPEC_ms_hashlock` §4.3 in the host's order -- non-empty, printable ASCII
`0x20..=0x7E`, not ms1-shaped, at most 100 characters, not exactly 64 hex
characters -- which is `ms-cli`'s `validate_phrase`
(`crates/ms-cli/src/hashlock_phrase.rs:118`) and the device's
`hashlock.ValidatePhrase` (`hashlock/hashlock.go:92-111`), byte for byte.

**Rust first.** `crates/me-cli/src/sysw/composer_records.rs` gains
`PHRASE_PREFIX`, `PhraseRecord`, `phrase_record(method, phrase)`, the parse arm
and `CASES` rows (`:306`); the corpus
`crates/me-cli/testdata/record_class_vectors.json` is regenerated by its own
test and re-vendored to `sysw/testdata/record_class_vectors.json` with the
provenance pin re-recorded (today: commit
`c05074f1d45970ca416785dfa9d9a812aaa21dbd`, sha256
`5b3960cad7f924f6f1e7f19ef49599814733cee4874d0f5eb48c28af4cd8b312`, 47 rows).
Nothing is decided in Go.

### §3.2 `--pack-preimage`

One flag, both carriers, in the pattern of `--seal-secret`
(`crates/me-cli/src/main.rs:126`, its refusal at `:932-937`):

```
/// Admit a hashlock PREIMAGE into this payload: an ms1 kind-0x03 plate
/// string, or a `phrase:` record carrying a hashlock phrase and its method.
///
/// Both are BEARER material -- whoever holds the preimage can spend any
/// key-less hashlock path it unlocks -- so admission is explicit, exactly as
/// `--seal-secret` makes encrypting seed material explicit. It is NOT
/// `--seal-secret` and the two do not substitute.
#[arg(long)]
pack_preimage: bool,
```

Without it, `me sysw pack` keeps refusing by index through
`admit_check` (`crates/me-cli/src/sysw/mod.rs:463-470`) and
`unknown_reason`'s preimage arm (`:209-211`), and the shipped text
(`crates/me-cli/src/main.rs:2805-2810`) gains one sentence -- §8.1.

### §3.3 Host warnings -- NORMATIVE, all four (§8.2)

`me sysw pack --pack-preimage` prints, on stderr:

1. **Always, when a preimage or `phrase:` record is admitted:** the transit
   warning (§8.2.1).
2. **When the flag is given and no preimage or `phrase:` record is present:** the
   no-op warning (§8.2.2). A warning, never a refusal -- the flag loosens
   admission and loosening it over nothing costs nothing.
3. **When an admitted preimage's digest matches none of the payload's `hash:`
   records:** the orphan warning (§8.2.3). Payload-wide, so it lives with
   `pack_with` (`crates/me-cli/src/sysw/mod.rs:335`) beside the "at most one
   `now:`" rule, not in the per-record `admit_check`.
4. **When the payload is SEALED and holds a preimage or `phrase:` record:** the
   sealed-transit note (§8.2.4).

### §3.4 Sealing -- NORMATIVE, and one correction to the ruling's premise

`ClassPreimage` and `ClassPhrase` **are secret**: `Class.IsSecret()`
(`sysw/record.go:60-66`) answers true for both, and `is_secret()` on the Rust
side likewise. Three consequences, all automatic:

1. `decide_sealing` (`crates/me-cli/src/main.rs:2353-2410`) SEALS by default,
   because it seals iff some record's class is secret. `--no-passphrase`
   (`crates/me-cli/src/main.rs:244`) produces the cleartext payload; the sealing
   line already says which way it went and why, and now names the class.
2. On the device, `syswFlags` (`gui/sysw_admit.go:143-162`) derives
   `flagSecretInPlaintext` from `c.IsSecret()` alone, so an unsealed payload
   holding a preimage raises F1 -- *"A SECRET is stored unencrypted in flash."*
   (`gui/sysw_load.go:283`) -- at LOAD, with no new wiring. A sealed one with a
   weak passphrase raises F2 the same way.
3. `bundleSetCarriesASecret` and the seed-plate arms are untouched: a preimage
   plate is not a `bundleCard` (§5.3).

**CORRECTION, measured (report §2, deviation D1).** The ruling's premise for
warning 4 -- *"the device cannot unlock it (the F-474 arm refuses a preimage in a
sealed section)"* -- does not hold for this container. `sysw.Open`
(`sysw/open.go:36-73`) runs no admission at all, and `syswSession.load`
(`gui/sysw_session.go:79-110`) appends `p.Public` **and** `p.Secret` into
`s.records`, so a sealed `sysw` payload's preimage IS reachable after unlocking.
The F-474 arm (`gui/unlock_kdf.go:415-420`, its noun at `:433`) belongs to
`unlockSealedFlow` (`gui/unlock_flow.go:98`) over `seal.Payload` -- the frozen
**Sealed Payload** container, a different product, whose `AdmitSection`
(`seal/open.go:149`) does refuse a preimage plate and whose host half refuses it
at pack time already (`crates/me-cli/src/seal/record.rs:130-136`). §8.2.4 states
what is true of each container and claims nothing about the other.

---

## §4. Device admission and classification

### §4.1 Two new classes, admitted at one program

`sysw.Class` gains `ClassPreimage` and `ClassPhrase`
(`sysw/record.go:24-54`). `admitted` (`gui/sysw_admit.go:32`) gains both to
`progWalletPolicy` **and to no other row** (`:64-73`), exactly as `ClassKey`,
`ClassHash` and `ClassNow` are. The row comment states, normatively, that a
hashlock phrase is **not** a BIP-39 passphrase and is never admitted at
`progPassword`: the terminology ruling L2 exists because interchanging them
opens a different wallet, and `progPassword`'s row stays `{ClassPassphrase}`.

### §4.2 The classifier

`Classify` (`sysw/record.go:111-139`) gains a `PhrasePrefix` arm beside the
other reserved prefixes, and `classifyConstellation`'s ms1 arm learns the
preimage class through a NEW predicate. `isStrictMs1`
(`sysw/classify.go:116-127`) is **unchanged**: its final line
(`return err == nil && !codex32.IsPreimage(c)`) is H0's inertness and keeps a
preimage out of every seed class. The preimage class is answered BEFORE it, by
`isPreimagePlateRecord`, so the two rules never overlap.

### §4.3 The `hash` id is required on the H6 path -- NORMATIVE (ruling A7)

`codex32.IsPreimage` (`codex32/mspayload.go:94-101`) does not consult the id, and
the host's `preimage_plate` (`crates/me-cli/src/seal/record.rs:287-320`) does not
either -- it tests `unshared && len(data)==33 && data[0]==0x03`. That was the
safe direction under H0, where the consequence of a false positive was a
REFUSAL: *"A refusal costs a re-encode; a wrong cut exposes a spend secret"*
(`codex32/mspayload.go:78-92`). H6 inverts the consequence -- a false positive
now routes a string INTO a flow that engraves it under a band reading
`NOT A SEED` -- so the H6 admission path adds the id:

```go
// codex32.IsPreimagePlate is IsPreimage PLUS the id `hash` (SPEC_ms_hashlock
// §1 rule 2, ruling L14). H0's kind-byte rule is unchanged and still governs
// inertness everywhere else; this narrower predicate governs ADMISSION to a
// flow that ENGRAVES, where a plain BIP-93 33-byte secret beginning 0x03
// (roughly 1 in 256 of them) must not arrive.
func IsPreimagePlate(s String) bool {
	if !IsPreimage(s) {
		return false
	}
	id, _, _ := s.Split()
	return id == "hash"
}
```

`s.Split()` is `codex32/codex32.go:394-401`. The Rust primary gains the same
narrowing as `preimage_plate_admissible(s)`, leaving `preimage_plate` -- the
DIAGNOSTIC predicate behind the refusal message -- unchanged, so a mistagged
plate is still named a preimage plate when it is refused and still not admitted
when `--pack-preimage` is passed. The refusal copy is §8.1.2.

**H0's inertness elsewhere is untouched:** `DecodeMS1` keeps refusing `0x03` at
all five callers (`gui/ms1_decode.go:22`, `gui/codex32_polish.go:106`,
`gui/singlesig_verify.go:185`, `gui/multisig_verify.go:1237`,
`bundle/verify.go:138`); the scan door refuses upstream (`gui/scan.go:89`);
`engraveCodex32` refuses at the choke point (`gui/codex32_polish.go:232-235`);
`seal.Classify` and `seal.AdmitSection` are not touched.

---

## §5. Screens

### §5.1 `Which hash?` gains two row bands

`composerHashRows` (`gui/composer_hash.go:157-175`) builds the row set once and
records each named band's index; the switch in `composerHashEdit`
(`:184-224`) dispatches on those names and its `default` PANICS rather than
assigns (H2 §5, r2 C-4). Row order:

1. the payload's `hash:` digests -- `hash <i>  <first8>..<last8>`, unchanged;
2. **the payload's preimage records** -- `preimage <i>  <first8>..<last8>`;
3. **the payload's `phrase:` records** -- `phrase record <i> (derive to see the
   digest)` before derivation, `phrase <i>  <first8>..<last8>` after;
4. `Type a hashlock phrase`;
5. `Type 64 hex`;
6. `No hash lock`.

Indices are 1-based positions among records of that class, as
`composerHashRow(i+1, d)` already is (`gui/composer_hash.go:161`). MEASURED at
`sh2DisplaySize`: every row above draws on ONE line in `composerPageLines`' band
(411 px, 23 px per row) -- `preimage 10  b867db87..edbc96cb` and
`phrase record 10 (derive to see the digest)` included.

**Derivation is LAZY (ruling B).** A `phrase:` row derives on PICK, behind the
existing `Deriving` countdown (H2 §4.4), and the result is entered into
`hashlockHeld` so the row shows its digest for the rest of the composition. A
hardened phrase costs about 10 s at the measured 9,715 iterations/s, and three
records would be a 30 s stall before a list could be drawn; the cost belongs on
an action the operator chose.

**The method is the RECORD's, never a pick.** A `phrase:` record names its
method (§3.1); the device does not offer the method screen for it, so the
mistake J4-1 describes -- picking the wrong method for a phrase the operator did
not choose -- cannot occur.

`taking` (`gui/composer_hash.go:194`), which fires the §8i rule modal, extends to
the two new bands. `composerPickScreenMaxRows` (`gui/composer_paged.go:243`) is
checked against the longest row set.

### §5.2 The Hashlock plates flow

A fourth route on the Wallet Policy door, after "Scan cards", "From payload" and
"Build a new policy" (`composerDoorFlow`, `gui/composer_door.go:98-119`). It is
**conditional**, gated by a predicate of the same shape as
`composerDoorHasConsumablePolicy` (`:93-97`): offered only when the loaded
payload holds at least one `ClassPreimage` or `ClassPhrase` record. A door row
that names a route it cannot take is the F-437 defect the door exists to remove.

The flow: list the payload's preimage and phrase records; the operator picks;
per record it offers form and QR (§5.3 item 3); then it cuts. It builds no
composition and reads nothing from `composerState`.

**Its locator header** (§6.3) prints the digest always, the matching `hash:`
record's position when one matches, and the `mk1 stub` **only when it can compute
one from an md1 record in the same payload** -- no payload record carries an id
(`sysw/record.go:14-21`, `sysw/composer_records.go:24-31`), so an md1 in the same
payload is the only source, and the field is omitted otherwise.

### §5.3 The Done review -- folded into `Plates To Cut`

The review is at the ENGRAVE Done, inside `composerEngraveStep`
(`gui/composer_flow.go:335-394`), on the census screen
`confirmReviewScreen(ctx, th, "Plates To Cut", composerCensusLines(...))`
(`:389-390`, `gui/multisig_build.go:1895`). Normative:

1. **The preimage plate is not a `bundleCard` and does not enter `plan`.** It
   follows S6b's passphrase-plate precedent (`gui/multisig_build_census.go:88-93`:
   entering `plan` or the inventory *"would tell a reader it travels WITH the
   set"*), so `buildPlateCensusLines`' count and its *"a set is only a backup
   when all of it exists"* claim (`:66-72`) are about the policy plates alone
   and stay byte-unchanged.
2. **Its own census block**, appended by `composerCensusLines`
   (`gui/composer_census.go:86-94`): a heading, one row per plate, and the
   apart-storage line (§8.3).
3. **Form and QR are chosen PER PLATE**, on the same screen, because two
   preimages in one policy may want different forms. The QR is offered only on
   the phrase form (decision 1) and only when the device holds the phrase.
4. **A retained preimage no current path carries is LISTED, never cut**, for BOTH
   provenances (ruling B; journey Q4): a row
   `preimage <first8>..<last8>: not on any path, will not be cut`. Nothing is
   deleted from `hashlockHeld` to achieve this (§2.2 item 2).
5. **A declined plate is dropped from the plan and NAMED in the census** as
   `declined, will not be cut`. Declining does not abort the run: a preimage
   plate is not part of the policy set, so `bundleEngrave`'s set-level
   "a partial bundle can't be used" reasoning (`gui/bundle_flow.go:625-631`) does
   not reach it.
6. **No cap on the number of preimage plates.** One per path that carries held
   material, one per payload record in the Hashlock plates flow.
7. **A phrase is MASKED on screen, with a `show` toggle** (journey Q13). The
   review and the Hashlock plates flow print `phrase: <n> characters` and the
   method, and reveal the characters only while the toggle is held -- the
   affordance the operator already met on the phrase keyboard, which
   `NewPassphraseKeyboard` carries (`gui/passphrase_keyboard.go:80`) and H2 §4.2
   inherits as-is. It applies to BOTH provenances: a device-typed phrase was
   just typed, but a payload-delivered one is a secret the operator never typed,
   may not own, and did not ask to see, in whatever room the machine lives in.
   Secret-handling and therefore non-gating, so it costs no round; it is
   specified rather than left out because the plate preview is the one screen
   that must show the phrase and the review is not.

### §5.4 Cut order -- preimage plates FIRST

`composerEngraveStep` cuts every accepted preimage plate BEFORE calling
`bundleEngrave`, in its own loop:

```go
for _, pl := range plates {           // §6, one Plate per accepted preimage
	if !NewEngraveScreen(ctx, pl).Engrave(ctx, &engraveTheme) {
		return composerAbortNoPreimage(ctx, th, st)   // §8.4
	}
}
return bundleEngrave(ctx, th, "Wallet Policy", cards, markTitle, "") == bundleEngraveDone
```

`NewEngraveScreen` is `gui/gui.go:3296`; the plate is built by §6's own layout
function and `toPlate` (`gui/gui.go:3620`), on the `ppBuildPlate` pattern
(`gui/passphrase_flow.go:570-588`). Ordering removes the window in which the md1
plates exist and the preimage does not; §8.4's abort arm covers the window
ordering cannot (a blank runs out mid-plate). The ms1 secret cards keep their
place at the head of `cards` (`gui/composer_flow.go:388`), so the whole run is
secrets-then-policy.

---

## §6. The plate -- a dedicated layout

### §6.1 The type

A new `backup.Hashlock`, on the `backup.Passphrase` pattern
(`backup/passphrase.go:23-49`) and **never** through `validateMdmkStrings`
(`gui/gui.go:2626-2648`), whose single-string arm offers `TEXT + QR` / `TEXT
ONLY` / `QR ONLY` and QR-encodes THAT STRING -- which decision 1 forbids.

```go
type Hashlock struct {
	// Form selects the band text and the body (§6.2).
	Form HashlockForm // HashlockString | HashlockPhrase
	// MS1 is the kind-0x03 plate string, for HashlockString. Engraved VERBATIM.
	MS1 string
	// Phrase and Method are for HashlockPhrase. Phrase is engraved VERBATIM,
	// with every space rendered as backup.SpaceMark; Method is §8.6's line.
	Phrase string
	Method string
	// Locator rows, pre-formatted by the caller (§6.3). backup takes no
	// dependency on md or hashlock, exactly as Passphrase takes none.
	Locator []string
	// QR is opt-in and legal only on HashlockPhrase (§6.4).
	QR   bool
	Font *vector.Face
}
```

`SpaceMark` (`backup/passphrase.go:21`) and its legend
(`passphraseLegend`, `:168`) are reused unchanged on the phrase form: *"one space
and two look identical ... while 'hunter2 ' is a different wallet from
'hunter2'"* is true of a hashlock phrase for the same reason -- `Correct Horse`
and `correct horse` derive different preimages, and H2 §2 forbids every
normaliser by name. The legend row is drawn only when the phrase contains a
space.

### §6.2 The bands -- form-specific (ruling A4)

| form | Title (plate row 0) | Footer (last plate row) |
| --- | --- | --- |
| string | `HASHLOCK PREIMAGE` | `NOT A SEED` |
| phrase | `HASHLOCK PHRASE` | `NOT A SEED` |

MEASURED lengths: 17, 15, 10 -- all inside `MaxTitleLen = 18`
(`backup/backup.go:71`), the cap `TestTitleCapFitsAtEveryRung`
(`backup/freetext_test.go:75`) proves clears the screw holes at 6.0 mm by
0.620 mm. Both are engraved VERBATIM, never through `TitleString`, which
upper-cases and truncates (`backup/freetext.go:11-19`).

**The phrase form's method line is the first body row, directly beneath the
title band** (ruling A4). It is a BODY row, not a band row.

### §6.3 The locator header -- body rows, not band rows (ruling A5)

Rows, in order, immediately after the method line (phrase form) or the title
(string form):

```
path <n>                            (composer-native only)
hash  <first8>..<last8>             (always)
mk1 stub (policy): <8 hex>          (when every slot is seated)
mk1 stub (template): <8 hex>        (otherwise, and when no policy id exists)
matches hash <i> in the payload     (Hashlock plates flow, when one matches)
```

The stub labels are `gui/composer_stub.go:53-70`'s literal labels, so the plate
and the screen the operator copied into their notebook use the same words. A
key-less or partially seated composition has no Policy-ID
(`gui/composer_engrave.go:40-61`: *"no id yet"*; `gui/composer_stub.go:56-72`
adds the keyed pair only when keyed chunks exist), which is why the template stub
is the fallback rather than an empty field.

**They are body rows because they do not fit a band.** MEASURED at the smallest
rung, `constant.Font`, W advance 600 against `Metrics{Ascent:800, Height:900}`,
so the advance at 3.0 mm is exactly 12,800 units = 2.0000 mm: the passphrase
plate's own 64 mm band ceiling (`backup/passphrase_test.go:521`, 409,600 units)
holds **32 characters** at 3.0 mm, 25 at 3.8 mm and 16 at 6.0 mm.
`mk1 stub (template): 1a2b3c4d` is 29 characters and
`path 2   hash  b867db87..edbc96cb` is 33 -- the second is over the cap at every
rung. The body wraps at the full 79 mm width instead (39 characters at 3.0 mm).

### §6.4 The QR -- phrase form only, BELOW the text (rulings A2, A3)

- **Only the phrase form carries a QR.** The string form is text-only; the ruling
  declines the brainstorm's B5 (a QR of the ms1 string), so a preimage-string
  plate has no machine-readable copy by design.
- **The QR encodes §8.6's text**, never the ms1 string (decision 1).
- **It is built with `engrave.ConstantQR`** (`engrave/engrave.go:418`), never
  `engrave.QR` (`:277`): the latter engraves in a content-dependent pattern and
  *"would leak the secret through timing"* (`backup/passphrase.go:112-114`). The
  phrase is a spend secret and the rule is the passphrase plate's, unchanged.
- **It is stacked BELOW the text**, as `passphraseLayoutFor` stacks its own
  (`backup/passphrase.go:283-292`, `l.envY = l.textY + l.blockH + gap` at `:289`),
  with `gap = 2 mm` (`passphraseQRGap`, `:74`).
- **Envelope 53 modules, scale 2.** The size is VARIABLE with the phrase length,
  so the layout reserves the worst case and centres the actual code inside it,
  as `passphraseQREnvelope = 37` does (`:70-72`).

### §6.5 Geometry -- MEASURED, and the fit is tight

All at `sh2.Params()` (`internal/sh2/params.go:43-53`: `Millimeter = 6400`,
`StrokeWidth = 1920`), `constant.Font`, plate 85 mm, `outerMargin = 3`,
`innerMargin = 10`.

| rung | chars/line (79 mm) | lines/plate | advance |
| --- | --- | --- | --- |
| 6.0 mm | 19 | 13 | 4.000 mm |
| 5.0 mm | 23 | 15 | 3.435 mm |
| 4.4 mm | 26 | 17 | 2.933 mm |
| 3.8 mm | 31 | 20 | 2.533 mm |
| 3.4 mm | 34 | 23 | 2.267 mm |
| 3.0 mm | 39 | 26 | 2.000 mm |

The vertical budget for the centred group is **65 mm** (85 − 2 × `innerMargin`),
the span `passphraseLayoutFor` centres inside while the bands hold the title and
footer. Worst-case bodies -- header `path 2` + `hash  <first8>..<last8>` +
`mk1 stub (template): <8 hex>`, a blank, the 73-character hardened method line, a
blank, and a 100-character phrase; or the same header, a blank and the
75-character ms1 string:

| form | rung | text | + gap + QR | total | 65 mm |
| --- | --- | --- | --- | --- | --- |
| phrase + QR (scale 2) | 3.0 mm | 30.0 mm | 2 + 31.80 | **63.80 mm** | **FITS, 1.20 mm spare** |
| phrase + QR (scale 2) | 3.4 mm | 37.4 mm | 2 + 31.80 | 71.20 mm | OVER |
| phrase + QR (scale 3) | 3.0 mm | 30.0 mm | 2 + 47.70 | 79.70 mm | OVER |
| phrase, no QR | 4.4 mm | 57.2 mm | — | 57.2 mm | FITS |
| phrase, no QR | 5.0 mm | 80.0 mm | — | 80.0 mm | OVER |
| string, no QR | 6.0 mm | 60.0 mm | — | 60.0 mm | FITS |

**Three normative consequences.**

1. **The QR scale is 2, not the passphrase plate's 3.** At 53 modules, scale 3 is
   47.70 mm and does not fit at any rung; scale 2 is 31.80 mm. This is the free-text
   plate's own scale (`freeTextQRScale = 2`, `backup/fit.go:16-19`: *"0.6mm modules
   against the 0.9mm every other plate uses"*).
2. **The worst-case phrase-plus-QR plate fits at exactly one rung, with
   1.20 mm to spare.** The layout AUTO-FITS down `backup.FontSizes`
   (`backup/backup.go:83`) and REFUSES at the bottom rung rather than drawing
   what it cannot lay out, as `EngraveText` does
   (`backup/backup.go:388-400`) and as `toPlate` enforces. The refusal names the
   measured ceiling.
3. **The method line is the largest single term and must not grow.** At 73
   characters it is 2 lines at 3.0 mm; a 79th character would make it 3 and put
   the worst case 3.0 mm over budget. §11 pins this with a mutation.

---

## §7. Raising the constant-time QR encoder to v9 -- NORMATIVE (ruling A1)

`ConstantQR` refuses anything over 37 modules today
(`engrave/engrave.go:418-426`), because `bitmapForQRStatic` (`:394-414`)
tabulates 21/25/29/33/37 and a larger version would reach its
`panic("unsupported qr code version")`. Its own comment states the bound's origin
and the rule for changing it: *"ECC-L caps at 106 bytes and the passphrase caps
at 100 (spec O6) ... Raise both together or not at all."*

### §7.1 The ceiling is v9 = 53 modules

MEASURED with the fork's own encoder, ECC-L byte mode:

| payload | bytes | modules | version | `ConstantQR` today |
| --- | --- | --- | --- | --- |
| §8.6 text, hardened, 100-char phrase (**worst case**) | 194 | 53 | **9** | refuses |
| §8.6 text, sha256, 100-char phrase | 135 | 45 | 7 | refuses |
| §8.6 text, hardened, 28-char anchor phrase | 122 | 41 | 6 | refuses |
| §8.6 text, sha256, 1-char phrase | 36 | 29 | 3 | accepts |

ECC-L thresholds, measured: ≥1 → 21 (v1), ≥18 → 25, ≥33 → 29, ≥54 → 33,
≥79 → 37 (v5, today's cap, 106 bytes), ≥107 → 41 (v6), ≥135 → 45 (v7),
≥155 → 49 (v8), **≥193 → 53 (v9)**, ≥231 → 57 (v10).

**The ruling names "v7 = 53 modules"; that is v9.** QR side = 4 × version + 17,
so v7 is 45 modules and v9 is 53. The 194-byte worst case needs **v9**, and the
new ceiling is `dim > 53`.

### §7.2 The alignment-pattern change

Derived EMPIRICALLY from the encoder's own bitmap by testing the 5×5 ring shape
at every candidate centre (§11 keeps the derivation as a test, so the table
cannot be transcribed wrong):

| version | dim | rings | centres |
| --- | --- | --- | --- |
| 2 | 25 | 1 | (18,18) |
| 3 | 29 | 1 | (22,22) |
| 4 | 33 | 1 | (26,26) |
| 5 | 37 | 1 | (30,30) |
| **6** | **41** | **1** | **(34,34)** |
| **7** | **45** | **6** | (22,6) (6,22) (22,22) (38,22) (22,38) (38,38) |
| **8** | **49** | **6** | (24,6) (6,24) (24,24) (42,24) (24,42) (42,42) |
| **9** | **53** | **6** | (26,6) (6,26) (26,26) (46,26) (26,46) (46,46) |

`fillMarker` takes a TOP-LEFT, so each entry is `centre − 2`. Normative:

1. **v6 is free.** Its single ring sits at `(dim−9, dim−9)` = (32,32), which is
   exactly the formula `bitmapForQRStatic` already uses for 25/29/33/37; adding
   `41` to that case arm is the whole change.
2. **v7-v9 need the six-ring layout.** `bitmapForQRStatic` gains a per-version
   centre table for 45/49/53 (the three rows above, verbatim), each expanded to
   six top-lefts by dropping the three combinations the position markers occupy.
   The `default: panic` arm STAYS -- it is what keeps an unhandled version from
   being drawn as if it had no alignment patterns at all.
3. **`ConstantQR`'s bound becomes `dim > 53`**, and its comment records the new
   pair (v9, 192 bytes at the last full step below 194 -- see §7.1's table) and
   the same "raise both together" rule.
4. **The constant-time argument is re-earned, not inherited.**
   `constantTimeQRModules`, `constantTimeStartEnd` and `findPath`
   (`engrave/engrave.go:349`, `:377-379`, `:430-470`) all take `dim`; the deliverable is
   a test that, for each of v6..v9, the emitted move count is a function of `dim`
   ALONE -- identical for two different payloads of the same version -- and that
   `findPath` never returns "QR modules spaced too far for constant time
   engraving" on any of them. Without that test the raise is a size change with
   an unproven security property.
5. **Goldens.** One golden per newly-admitted version (v6, v7, v8, v9) on a
   fixed payload, in `backup/testdata/`, alongside the plate goldens of §11.

---

## §8. Copy -- operator-facing strings

ASCII only. Every DEVICE body below was measured on the gate tree with
`assertModalBodyFits` (`gui/modal_fits_test.go:202`), which renders the specific
body and binary-searches its headroom with `modalHeadroom` (`:183`), requiring at
least `modalBodyMargin = 80` normalised characters (`:52`). Host lines are stderr
and carry no panel budget.

### §8.1 Host refusals

**§8.1.1** `crates/me-cli/src/main.rs:2805-2810`'s existing text gains one
sentence (decision 7):

> record {i} (records count from 0) is a hashlock PREIMAGE plate (kind 0x03),
> not a seed record; this container cannot place one yet. A preimage backs a
> hashlock spend path, not a wallet — keep it with the policy it unlocks, and do
> not re-encode it as entropy. Re-run with --pack-preimage if that is what you
> intend.

**§8.1.2** A NEW arm, for a kind-`0x03` single whose id is not `hash` (§4.3):

> record {i} (records count from 0) is a kind-0x03 preimage payload whose
> 4-character id is not `hash`. A preimage plate is kind 0x03 under the id
> `hash` (SPEC_ms_hashlock §1 rule 2), and --pack-preimage admits only that.
> Re-encode it with `ms hashlock` rather than editing the string.

Its device-side twin is not a modal: such a record is `ClassUnknown` and inert,
visible only in the door's not-understood count (`gui/composer_door.go:41-56`).
The wording measured **165 characters drawn, headroom 397** on `errorScreenBody`
so that it can be surfaced later without a re-measure.

### §8.2 Host warnings

**§8.2.1, transit -- always, when a preimage or `phrase:` record is admitted:**

> me: WARNING — this payload carries a hashlock PREIMAGE. Anyone who holds the
> tag can read it, and for a key-less hashlock path the preimage alone spends
> the coins. Treat this payload as bearer material until it is on the machine
> and erased.

**§8.2.2, the flag with nothing to admit:**

> me: --pack-preimage was passed and this payload holds no preimage plate and no
> `phrase:` record. Nothing was admitted that would otherwise have been refused.

**§8.2.3, orphan digest:**

> me: WARNING — record {i} (records count from 0) is a preimage whose digest
> {first8}..{last8} matches no `hash:` record in this payload. Nothing here
> tells the device which policy it unlocks, and the Hashlock plates flow will
> print the digest alone.

**§8.2.4, sealed transit** (see §3.4's correction -- each clause is true of the
container it names):

> me: this payload is SEALED and holds a hashlock preimage, so the device needs
> the passphrase above before it can reach it. `me seal` — the Sealed Payload
> container — refuses a preimage plate outright; this one does not.

### §8.3 The census block (device rows, paged by `confirmReviewScreen`)

Heading, then one row per plate, then the apart-storage line:

> Plus {n} preimage plate(s), cut first and NOT part of this backup:

> path {n}  {first8}..{last8}  {phrase, hardened, QR | phrase, sha256 | preimage string}

> preimage {first8}..{last8}: not on any path, will not be cut

> preimage {first8}..{last8}: declined, will not be cut

> Keep each preimage plate apart from the policy plates and from the others.

The stand-alone notice form of the fourth row, for a review whose only entry is
an unused preimage, measured **107 drawn / headroom 455**:

> One preimage this composition holds is on no path of this policy. It will not
> be cut. Go back and set a path's hash to it, or leave it.

### §8.4 The abort arm

`bundleAbortWarningText` (`gui/bundle_flow.go:780-792`) gains a third clause,
prepended to the seed clause because it is the funds-losing fact:

> NO PREIMAGE PLATE WAS CUT. The phrase dies with this composition. Do not fund
> this wallet.

Fires when the run ends before every accepted preimage plate is cut. MEASURED on
`errorScreenBody`: **288 drawn / headroom 244** alone, and **411 drawn /
headroom 121** in the LONGEST variant, with the shipped seed clause following it.

**Four longer drafts were measured and rejected**, because headroom is a LINE
budget and each cost a line: *"...leaving the composer or losing power destroys
it, and no plate already cut can recover it..."* 488/**39**; *"...leaving the
composer or losing power destroys it..."* 455/**79**; *"...The phrase is held
only by this composition and dies with it..."* 428/**79**; *"...and no plate
already cut carries it..."* 441/**79**. All four are under the 80-character
margin in the combined body and none may be restored without re-measuring.

**It says "dies with this composition", not "is now gone"** (report §2, deviation
D2). An abort inside `bundleEngrave` returns `bundleEngraveAborted`, so
`composerEngraveStep` returns false and `composerFlow` loops back to the shape
with the state intact (`gui/composer_flow.go:47-131`) -- the phrase is still
held. Saying it is gone would be false on the screen whose job is to stop a
funding decision, which is the defect class H5 §1.2 removed from the confirm
modal when "and this digest" made a claim false of the one item that WAS on the
plates.

### §8.5 The QR toggle warning

Confirm-to-proceed, on the model of `ftWarnQR`
(`gui/freetext_flow.go:1214-1216`), which already warns for strictly less
dangerous content:

> The QR makes the phrase readable by any camera. A photograph of the plate is a
> copy of the phrase, and the phrase spends this path.

MEASURED through `confirmWarningBody`, wrapped in `composerConfirmBody`: **126
drawn / headroom 378**.

### §8.6 The QR text -- NORMATIVE, byte for byte (decision 2)

Three labelled lines, LF-separated, **no trailing newline**, the phrase LAST:

```
hashlock v1
method: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32
phrase: <phrase>
```

or, for the sha256 method, the middle line is exactly:

```
method: sha256
```

Rules:

1. `hashlock v1` is the VERSION TAG and is version 1 of this text, not of the
   derivation. A future parameter set gets `hashlock v2`.
2. The method line names the algorithm IN FULL, so a reader with the plate and no
   tool can reproduce the derivation. Its parameters are `HASHLOCK_SALT`,
   `HASHLOCK_ITERATIONS` and `HASHLOCK_DKLEN`
   (`crates/ms-codec/src/hashlock.rs:27,30,32`), rendered lowercase; the device
   reads them from `hashlock.Salt`, `hashlock.Iterations` and
   `hashlock.PreimageLen` (`hashlock/hashlock.go:21,24,27`) and never from a
   literal, so a parameter change cannot leave the plate lying.
3. The phrase is the LAST line so a reader knows where it ends: it may itself
   contain `:` and spaces, and everything after `phrase: ` on the final line is
   the phrase, verbatim, with real `0x20` spaces and never `SpaceMark` -- the
   passphrase plate's rule (`backup/passphrase.go:93-100`: *"a scanner that saw
   it would hand a wallet different bytes"*).
4. The method line is 73 characters and the whole text is **194 bytes** at a
   100-character phrase (§6.5, §7.1). It is pinned by corpus rows in the
   `ms hashlock` corpus (§11.2), because `ms hashlock` learning to parse it is a
   follow-on and the text must be fixed before the parser exists.

### §8.7 The plate's own words

Band texts are §6.2's four literals. The phrase form's space legend is
`passphraseLegend` (`backup/passphrase.go:168`), reused verbatim.

---

## §9. The free-text and passphrase warning (ruling A8)

**Never a refusal.** Both programs cut as typed. `engraveTextFlow` and
`engravePassphraseFlow` gain a confirm-to-proceed at OK on the text-entry screen
-- earlier than the confirm summary, which is already paged
(`ftConfirmFlow`, `gui/freetext_flow.go:1362-1402`; `ppConfirmFlow`,
`gui/passphrase_flow.go:497`). It fires once per composition, re-armed by an
edit.

**The predicate is `hashlock.IsMS1Shaped`** (`hashlock/hashlock.go:122-148`), the
host's `looks_like_ms1` ported byte for byte: trim, ASCII-lowercase, strip the
display separators, then ≥ 48 characters, an `ms1` prefix and only bech32
characters -- **no checksum**. That is deliberate and is H2 §2 rule 3's own
argument: a GROUPED plate is what `ms hashlock`'s card prints
(`crates/ms-cli/src/cmd/hashlock.rs:340,348`, `render_grouped`) and therefore what
an operator retypes, and `codex32.IsPreimage` would answer false for it.

**The body**, as ruled, unchanged by measurement:

> This looks like an ms1 string. A seed plate comes from a payload; a marked
> hashlock plate comes from the Wallet Policy program, from a phrase typed there
> or a preimage packed on the host. Continue here to cut it as plain text.

MEASURED: **204 drawn / headroom 302** through `confirmWarningBody` wrapped in
`composerConfirmBody`, and **184 / 378** on `errorScreenBody`. It fits as ruled;
no wording change is taken.

The passphrase program shows the SAME body: the string is about to become a
BIP-39 passphrase rather than a plate, but the sentence that matters -- what it
looks like, and where the marked plate comes from -- is identical, and two
near-identical bodies is how one of them goes stale.

---

## §10. Provenance-aware copy and marking

### §10.1 §8h gains a cut-in-this-set form

`composerCopyHashEveryPathFor` (`gui/composer_copy.go:527-532`) chooses today
between the plain form (`:182-186`) and the phrase form (`:520-525`). It gains a
third arm, chosen when EVERY hashed path's digest has material in
`hashlockHeld` that this run will cut:

> HASH ON EVERY PATH
> Every way to spend this wallet needs the preimage of a hash. This run cuts a
> preimage plate for each one. Store those plates apart from these, and apart
> from each other.

and, when at least one of those digests came from a phrase typed here:

> HASH ON EVERY PATH
> Every way to spend this wallet needs a hashlock preimage. This run cuts a
> plate for each phrase and its method. Store those plates apart from these, and
> apart from each other.

MEASURED on `errorScreenBody`: **153 drawn / headroom 378** and **159 / 378**.
When some hashed path has no plate in this run, the SHIPPED forms stand
unchanged -- they say the preimage is "not on these plates", which is then true
of at least one path, and H5 §2.5 already recorded that this family overcounts in
the safe direction.

### §10.2 The reconcile screen is suppressed for payload phrases

`composerCopyHashlockReconcile` (`gui/composer_copy.go:484-490`) tells the operator
to *"run ms hashlock with this phrase and method on the host and check the digest
matches"*. For a phrase that CAME from the host in the payload the host already
has it and the instruction is a no-op that reads as a task. The screen is drawn
only when the digest's provenance is `hashlockFromPhrase`. Its TEXT is unchanged
-- H5 §1.1's needle *"run ms hashlock with this phrase"* survives, and
`TestHashlockReconcileScreenIsReachableOnAMixedPolicy`
(`gui/composer_hashlock_test.go:920`) and the walk (`cmd/emu/walk_hashlock_phrase.js:458`)
keep theirs.

### §10.3 `PREIMAGE REQUIRED` on the composer's md1 plates

`composerEngraveStep` passes a marking to `bundleEngrave`
(`gui/composer_flow.go:393`, today `"", ""`): `markTitle = "PREIMAGE REQUIRED"`
when any path of the composition carries a hash, `""` otherwise; the footer stays
empty. This is the single-sig mechanism unchanged
(`singleSigPlateMark`, `gui/singlesig.go:365-374`, whose `"PASSWORD REQUIRED"` is
also 17 characters against `MaxTitleLen = 18`), and `bundlePlateMark`
(`gui/bundle_flow.go:565-579`) already refuses to mark a `cardMS1`, so the seed
plates in the same run stay unmarked with no change.

It closes the plate half of F-132 (`design/FOLLOWUPS.md:4298`): the year-later
reader holding only the descriptor plates is the one person nothing on the device
currently tells that a preimage exists.

---

## §11. Tests, each with the mutation that must fail it

### §11.1 Host, Rust first

- `phrase:` classification, every `CASES` row: method `hardened`/`sha256`, a
  phrase with a comma, an empty phrase, 100 and 101 characters, a 64-hex phrase,
  an ms1-shaped phrase, non-UTF-8 bytes, uppercase hex, an unknown method.
  MUTATION: cut on the LAST comma → the comma-bearing phrase row fails.
  MUTATION: accept an unknown method → the unknown-method row classifies.
- `--pack-preimage` admits a plate and a `phrase:` record; without it both are
  refused by index with §8.1.1's text. MUTATION: admit unconditionally → the
  no-flag rows classify.
- The id rule (§4.3): a kind-`0x03` single under id `hash` is admissible; the
  same payload under any other id is refused with §8.1.2. MUTATION: drop the id
  test from `preimage_plate_admissible` → the mistagged row is admitted.
  **This row is the funds-relevant one**: without it a plain BIP-93 33-byte
  secret beginning `0x03` reaches a plate flow.
- `decide_sealing` seals a payload holding a preimage and says so; with
  `--no-passphrase` it does not and says so. MUTATION: make the class non-secret
  → the seal row reports NOT SEALED.
- All four warnings of §8.2 fire on their condition and not otherwise.
  MUTATION: drop the orphan check → the orphan row prints no warning.
- The corpus is regenerated and re-vendored; the Go side asserts the sha256
  literal. MUTATION: edit one vendored row → the sha test reds.

### §11.2 The QR text corpus (mnemonic-secret)

Rows in `crates/ms-codec/tests/vectors/hashlock-v0.8.json`, whose sha and
provenance are re-pinned on both sides (H2 §7.1 requires the fork to assert the
literal): the exact §8.6 text for the anchor phrase under both methods, for a
100-character phrase, for a phrase containing `:` and for a phrase containing a
trailing space. MUTATION: emit a trailing newline → every row fails.
MUTATION: put `phrase:` before `method:` → every row fails.

### §11.3 The constant-time QR raise

- The alignment table of §7.2 is DERIVED in the test from the encoder's own
  bitmap and compared against the table the code carries, for v2..v9.
  MUTATION: change one centre → the derivation disagrees.
- `ConstantQR` accepts 41/45/49/53 and refuses 57. MUTATION: raise the bound to
  57 without the table → `bitmapForQRStatic` panics, which the test asserts it
  no longer does.
- Constant-time: for each of v6..v9, two different payloads of the same version
  emit the SAME move count, and `findPath` returns no error. MUTATION: make the
  move list depend on a module's colour → the two payloads differ.
- Goldens for v6, v7, v8, v9 on fixed payloads.

### §11.4 The plate

- Golden bytes for both forms at their fitting rungs, with and without the QR.
- The §6.5 fit gate: the worst-case phrase plate (100-character phrase, hardened
  method line, three header rows, QR) lays out at 3.0 mm and REFUSES at any
  larger rung, measured on the real render rather than on arithmetic.
  MUTATION: add one character to the method line → the worst case no longer
  fits and the gate reds. MUTATION: set the QR scale to 3 → it reds.
- Band budget: the four band literals are within `MaxTitleLen`; every locator
  row is a BODY row and no locator ink lands in a screw-hole band.
  MUTATION: move `mk1 stub (template): <8 hex>` into a band → the geometry gate
  fails at 3.0 mm (29 characters against the 32-character cap is inside, but the
  33-character `path` row is not, and the gate measures the longest).
- The QR encodes §8.6's text and NOT the ms1 string. MUTATION: encode
  `plate.MS1` → the decode assertion fails.
- Spaces: a phrase with a leading, trailing or doubled space renders `SpaceMark`
  in the text block and REAL spaces in the QR. MUTATION: mark the QR too → the
  QR decodes to a different phrase.
- The plate is not a `bundleCard`: `bundlePlatePlan` over the run's cards returns
  the same plan with and without preimage plates. MUTATION: add the plate as a
  card → the census count and the "all of it exists" claim change.

### §11.5 The device flows

- `Which hash?` by LABEL with 0, 1 and 2 records of each new class; every row
  does what its label says; `Type 64 hex` never clears the lock (H2's C-4
  regression test). MUTATION: reintroduce an index-keyed `default` that assigns
  → the displaced-row rows fail.
- A `phrase:` row shows the derive marker before it is picked and its digest
  after. MUTATION: derive at row-build time → a timing assertion on the screen's
  first frame fails.
- `hashlockHeld` on a zero-value `composerState` built exactly as `composerFlow`
  builds it: one HOLD, no panic, the material present. MUTATION: assign without
  the nil check → panic.
- `composerFlowExit` scrubs it. MUTATION: remove the scrub call → a test reading
  the map through the `!tinygo` seam after exit finds the phrase.
- The review: a declined plate is dropped and named; an unused preimage is listed
  and not cut; the plan's remaining plates are unchanged. MUTATION: drop the
  declined plate silently → the census row assertion fails.
- Cut order: the preimage plates are engraved before `bundleEngrave` is called.
  MUTATION: move the loop after it → an order assertion on the engrave hook
  fails.
- The abort arm fires when the run ends before a preimage plate is cut, and not
  when every one was. MUTATION: fire unconditionally → the all-cut row fails.
- §10.1's third §8h form fires on its predicate and not otherwise; §10.2's
  reconcile screen is drawn for a device-typed phrase and not for a payload one.
  MUTATION: key §10.2 on `phraseDigests` instead of provenance → the payload row
  draws it.
- `PREIMAGE REQUIRED` marks the md1 plates when any path is hashed and no plate
  when none is, and never marks a `cardMS1`. MUTATION: mark unconditionally →
  the unhashed row fails.
- §9's warning fires for a grouped ms1 plate string, an ungrouped one and an
  UPPERCASE one, and not for ordinary text; the plate is still cut as typed.
  MUTATION: use `codex32.IsPreimage` instead of `IsMS1Shaped` → the grouped row
  fails. MUTATION: refuse instead of warning → the still-cut assertion fails.
- Every body §8 adds is in `modal_fits_test.go`'s table and passes
  `assertModalBodyFits`; §8.4's LONGEST variant (preimage arm + seed arm) is the
  row that matters.

### §11.6 Whole gates

Four packages; the 24 `gui` shards (`scripts/gui-shard-test.sh <pkg> 24`);
`gofmt`; `go vet`; `cargo nextest run --locked` on the Rust side; the firmware
size stated for the STAGE against a named baseline, re-measured at `fb0dd04`
(H5's plan-round fold recorded 1,599,208 B shipped against 1,597,404 B at
`b9a9a30`; H6 adds a plate layout, a QR version table and two record classes, so
the delta is not expected to be small).

### §11.7 The walk

`cmd/emu/walk_hashlock_phrase.js` gains an H6 arm: type the anchor phrase, HOLD,
reach the census, accept a preimage plate, and assert the census row carries the
same `first8..last8` the confirm modal carried. H5 §4.1's doctrine binds -- *"a
walk may READ state only to assert that what the screen shows equals what is
stored; it never drives through a hook"* -- so the walk asserts the SCREEN and
the goldens assert the plate; no third hook carrying a preimage is added.

---

## §12. Acceptance

H6 is done when, on the flashed device:

1. The operator types the anchor phrase, reaches Done, accepts a preimage plate
   in the phrase form with a QR, and the plate is cut; a phone scan of the QR
   returns §8.6's text byte for byte, and `ms hashlock --hashlock-phrase-stdin`
   over the phrase on that plate reproduces the digest the composer showed.
2. The same composition's md1 plates carry `PREIMAGE REQUIRED`.
3. `ms hashlock --out X.txt`, `me sysw pack --pack-preimage --no-passphrase`, a
   tap, the Hashlock plates flow, and a cut plate whose ms1 string round-trips
   through `ms hashlock --in`.
4. A `phrase:` record in the same payload appears on `Which hash?` as
   `phrase record 1 (derive to see the digest)`, derives on pick, and its digest
   matches the host's.
5. A kind-`0x03` string under an id other than `hash` is refused on the host with
   §8.1.2 and is inert on the device.
6. A preimage plate presented to any seed flow is still refused (H0's walk).

Until the operator walks it, §11.7's emulator arm and §11.4's goldens are the
acceptance. **The SH2 has no camera**, so nothing on the device can read a plate
it cut; item 1's read-back is a phone and the host, and no acceptance may be
written that assumes otherwise.

---

## §13. Out of scope (this stage)

Reading a preimage plate back into any seed flow; the Sealed Payload container
(`me seal`), whose `RecordError::PreimagePlate` refusal stands
(`crates/me-cli/src/seal/record.rs:130-136`); the typed `M*1 STRING` door;
`ms hashlock` parsing §8.6's text (the text is pinned here, the parser is a
follow-on); `ms split` of a preimage (F-468); the salt and iteration parameters
(F-469); a words-plus-SeedQR secret plate (F-455); a concrete-descriptor plate
and its census refusal (F-457); a restore document for the composer; secret
handling of the retained phrase (F-483, non-gating by the 2026-08-27 ruling).

---

## §14. Citations -- measured at fork `fb0dd04`, engrave `07a92e9b`, ms `504ff46`

| claim | where |
| --- | --- |
| the phrase route drops X on return; HOLD assigns the digest and notes provenance | `gui/composer_hashlock.go:17-20`, `:43-86`, `:69`, `:70` |
| the composer's own record that H6 makes false | `gui/composer_hash.go:27-28`; `SPEC_wallet_policy_composer.md` §6c, §14 |
| `composerState`, `phraseDigests`, the nil-map rule, the value-set predicate | `gui/composer_state.go:26-79`, `:53`, `:46-50`, `:280-286`, `:299` |
| state construction, the ONE defer, the 96 B second-defer measurement | `gui/composer_flow.go:20-23`, `:48`, `:59`, `:53-57` |
| the engrave step, the census call, `bundleEngrave`'s marking parameters | `gui/composer_flow.go:335-394`, `:388`, `:389-390`, `:393` |
| `Which hash?` rows, the label-keyed switch, the `taking` predicate, the row form | `gui/composer_hash.go:157-175`, `:184-224`, `:194`, `:38-41`, `:161` |
| the door, its counts, its conditional route | `gui/composer_door.go:41-56`, `:93-97`, `:98-119` |
| the census and its completeness claim; the passphrase-plate exclusion precedent | `gui/multisig_build_census.go:63-73`, `:88-93`; `gui/composer_census.go:86-94` |
| `confirmReviewScreen` (paged) | `gui/multisig_build.go:1895` |
| `bundleEngrave`, the plate plan, the set-level abort, the marking rule | `gui/bundle_flow.go:616-657`, `:466`, `:625-631`, `:565-579`, `:780-792` |
| the standalone-plate cut pattern | `gui/passphrase_flow.go:570-588`, `gui/gui.go:3296` (`NewEngraveScreen`), `:3620` (`toPlate`) |
| `validateMdmkStrings` offers TEXT+QR / TEXT ONLY / QR ONLY for one string | `gui/gui.go:2626-2648` |
| the admission table and its one-program rule; the flags derive from `IsSecret` | `gui/sysw_admit.go:1-14`, `:32`, `:64-73`, `:143-162` |
| F1/F2's text at load | `gui/sysw_load.go:261-290`, `:283` |
| the session holds Public AND Secret records | `gui/sysw_session.go:54`, `:79-110` |
| `sysw.Open` runs no admission | `sysw/open.go:36-73` |
| the F-474 arm belongs to the Sealed Payload | `gui/unlock_flow.go:98`, `gui/unlock_kdf.go:415-420`, `:433`; `seal/open.go:149` |
| the classifier, the reserved prefixes, `IsSecret` | `sysw/record.go:14-21`, `:24-54`, `:60-66`, `:111-139` |
| the composer record parsers and the hex rule | `sysw/composer_records.go:24-31`, `:58`, `:82-97`, `:100`, `:136-160` |
| H0's ms1 inertness in the classifier | `sysw/classify.go:116-127` |
| `IsPreimage` (no id), its stated trade, `DecodeMS1Preimage`, `Split` | `codex32/mspayload.go:78-101`, `:113-128`; `codex32/codex32.go:394-401` |
| the five `DecodeMS1` callers and the two door refusals | `gui/ms1_decode.go:22`, `gui/codex32_polish.go:106`, `:232-235`, `gui/singlesig_verify.go:185`, `gui/multisig_verify.go:1237`, `bundle/verify.go:138`, `gui/scan.go:89` |
| the phrase rule, the ms1 shape test, the constants | `hashlock/hashlock.go:21,24,27`, `:92-111`, `:122-148` |
| the passphrase plate: struct, space mark, legend, QR rules, bands, stacking, gap | `backup/passphrase.go:21`, `:23-49`, `:70-74`, `:93-100`, `:112-114`, `:168`, `:250-254`, `:283-292` (`:289`) |
| the 64 mm band ceiling | `backup/passphrase_test.go:521` |
| plate constants, the size ladder, the capacity functions, `EngraveText`'s refusal | `backup/backup.go:71`, `:83`, `:88-96`, `:388-400` |
| the free-text QR scale and the title cap | `backup/fit.go:16-19`; `backup/freetext.go:11-19`, `backup/freetext_test.go:75` |
| `ftWarnQR` and the free-text confirm | `gui/freetext_flow.go:1214-1216`, `:1362-1402`, `:1485` |
| the passphrase confirm | `gui/passphrase_flow.go:497` |
| `ConstantQR`'s bound and its "raise both together" rule; `bitmapForQRStatic`; `engrave.QR` | `engrave/engrave.go:418-426`, `:394-414`, `:277`, `:378-380`, `:430-470` |
| machine parameters | `internal/sh2/params.go:43-53` |
| the fit gate: per-body render, headroom search, margin 80, "no capacity constant" | `gui/modal_fits_test.go:202`, `:183`, `:52`, `:33-35` |
| the composer confirm surface and its HOLD | `gui/composer_shape.go:77`; `gui/composer_copy.go:36-38` |
| the §8h forms and their chooser; the reconcile body | `gui/composer_copy.go:182-186`, `:520-525`, `:527-532`, `:484-490` |
| the single-sig marking this reuses | `gui/singlesig.go:365-374` |
| the stub labels the locator copies | `gui/composer_stub.go:53-70`, `:62-72`; `gui/composer_engrave.go:40-61` |
| `me sysw pack`: the flag pattern, the preimage refusal, admission, sealing, `--no-passphrase` | `crates/me-cli/src/main.rs:126`, `:244`, `:932-937`, `:2353-2410`, `:2805-2810`; `crates/me-cli/src/sysw/mod.rs:209-211`, `:335`, `:463-470` |
| the host preimage predicate that does not consult the id | `crates/me-cli/src/seal/record.rs:287-320`; the Sealed Payload refusal at `:130-136` |
| the host composer records, Rust-primary | `crates/me-cli/src/sysw/composer_records.rs:28-32`, `:306` |
| the vendored corpus and its provenance pin | `sysw/testdata/record_class_vectors.provenance.json` (commit `c05074f1…1dbd`, sha256 `5b3960ca…b312`, 47 rows) |
| the derivation constants and the phrase cap, Rust-primary | `crates/ms-codec/src/hashlock.rs:27,30,32`; `crates/ms-cli/src/hashlock_phrase.rs:24`, `:118` |
| `ms hashlock`'s method line, its grouped plate output, `--out` | `crates/ms-cli/src/cmd/hashlock.rs:281-288`, `:299-306`, `:340,348` |
| the ms1 preimage string is 75 characters, id `hash`, kind `0x03` | `SPEC_ms_hashlock` §1; corpus `kind[0].ms1` |
| F-132 (a preimage required, absent from the backup, unmentioned by it) | `design/FOLLOWUPS.md:4298` |
| F-483 (the phrase in an unwipeable Go string) | `design/FOLLOWUPS.md:16003` |

---

## §15. The rulings this spec applies

| ruling | where it lands |
| --- | --- |
| decisions 1-9 (2026-09-05) | throughout; §1 lists what each fixes |
| A1 constant time kept; raise the encoder | §7 (ceiling **v9 = 53 modules**, correcting "v7") |
| A2 QR only with the phrase form | §6.4; B5 of the draft report is declined |
| A3 QR below the text, passphrase stacking | §6.4, §6.5 (scale **2**, not the passphrase plate's 3 -- measured) |
| A4 form-specific bands | §6.2 |
| A5 the `mk1 stub` locator, policy then template | §6.3 |
| A6 the classes are secret; seal by default | §3.4, with the F-474 premise corrected |
| A7 the `hash` id on the H6 admission path | §4.3, §8.1.2 |
| A8 the free-text warning names the route | §9 (fits as ruled; no wording change) |
| B: preimage plate cut first + abort arm | §5.4, §8.4 |
| B: its own layout, own census line, excluded from the completeness claim | §6.1, §5.3 items 1-2 |
| B: retain phrase + method + preimage keyed by digest, scrubbed at flow exit | §2.2 |
| B: unused retained material listed, both provenances | §5.3 item 4, §8.3 |
| B: lazy derivation of payload phrase records | §5.1 |
| B: review at the engrave Done, per-plate form and QR, declined plates named | §5.3 |
| B: `PREIMAGE REQUIRED` marking | §10.3 |
| B: provenance-aware §8h and reconcile | §10.1, §10.2 |
| B: `phrase:` at `progWalletPolicy` only, never Password | §4.1 |
| B: the QR toggle carries an `ftWarnQR`-shaped warning | §8.5 |
| B: a `SpaceMark` equivalent | §6.1 |
| B: wire shape and method spelling follow the siblings, Rust first | §3.1 |
| B: the ms hashlock corpus owns the QR text rows | §11.2 |
| journey Q1-Q14 | Q1 §5.4+§8.4, Q2 §5.3, Q3 §6.1, Q4 §5.3.4, Q5 §8.5, Q6 §6.3, Q7 §5.1, Q8 §10.3, Q9 §2.2, Q10 §10.1-§10.2, Q11 §4.1, Q12 §5.3.6, Q13 §5.3 (masked with `show`), Q14 §9 |
