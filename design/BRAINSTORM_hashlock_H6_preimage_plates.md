# BRAINSTORM — Hashlock H6: engraving preimage plates on the SeedHammer II

**STATUS: DRAFT.** Written to the operator's nine fixed decisions of 2026-09-05
(the brief, `design/agent-briefs/hashlock-H6-brainstorm-draft-brief.md`). Those
nine are FIXED and are not reopened here; every section states what they settle,
the mechanism they land on, and the questions they leave open. The questions are
consolidated for the operator in
`design/agent-reports/hashlock-H6-brainstorm-draft-report.md`.

**Citations measured at:** engrave `07a92e9b`, seedhammer fork main `fb0dd04`,
mnemonic-secret `504ff46`. Every `file:line` below was grepped at those
revisions. Numbers labelled MEASURED were produced by running the fork's own
code against a scratch module (`backup.CharsPerLine`, `backup.LinesPerPlate`,
`qr.Encode`, `engrave.ConstantQR`) at fork `fb0dd04`; re-measure at spec time.

**What H6 reverses.** Brainstorm ruling L7 (`design/BRAINSTORM_hashlock_phrase.md`
§2) scoped the device to the digest alone: *"It never stores, shows, engraves or
sources a preimage."* H2 implemented that literally — `hashlockPhraseRoute`
derives X on the stack and drops it when the function returns
(`gui/composer_hashlock.go:43-86`), and the fork's own record says so at
`gui/composer_hash.go:27-28`. H6 is the stage that lifts three of those four
verbs: store (for the composition's lifetime, decision 9), show (on a review
screen, decision 6) and engrave (decision 5). *Source* — reading a preimage plate
back into a flow — stays out; H0's inertness elsewhere is untouched (decision 3).
Three shipped records will become false and must be rewritten by H6, not left:
`gui/composer_hash.go:27-28`, `SPEC_wallet_policy_composer.md` §6c and §14, and
`codex32/mspayload.go:63-93`'s `IsPreimage` header (which says the device "learns
to USE a preimage in stage H2, not here" and that a preimage engraved as a seed
"exposes a spend secret as a backup" — the second half stays true and the plate
this stage cuts is precisely *not* a seed plate).

---

## §1. The two paths and the material each holds

### Decisions applied

Decision 3 fixes two entry points and no others: **(a)** the wallet-policy
composer at HOLD, where the device itself has the phrase, the method and the
preimage in hand; **(b)** the **unsealed** wallet-policy payload built by
`me sysw pack`, carrying either an ms1 preimage string or a new phrase-and-method
record. The sealed payload and the typed `M*1 STRING` door are out. Decision 4
fixes the architecture as hybrid C: composer-native for (a), a separate
"Hashlock plates" flow under the Wallet Policy door for (b), and payload records
also appear as pickable digests on `Which hash?`. Decision 9 accepts RAM
retention for the composition's lifetime.

### Mechanism

**Path (a) — device-derived.** `hashlockPhraseRoute(ctx, th, st, idx, payload)`
(`gui/composer_hashlock.go:43`) holds `phrase []byte` in the outer loop and
`x [32]byte` from `hashlockDeriveFlow` (`:59`); on HOLD it writes only the digest
(`gui/composer_hashlock.go:69` `st.list.Paths[idx].Hash = &d`) and notes the
digest's provenance (`:70`, `composerNotePhraseDigest`). Both `phrase` and `x`
die with the function. H6's change is that the triple `(phrase, method, x)` must
survive into `composerState`.

`composerState` is a struct literal built at exactly one production site,
`gui/composer_flow.go:48` (`st := &composerState{reg: &seedRegistry{}, bound:
composerBoundFrom(ctx.sysw)}`), and torn down through one deferred
`composerFlowExit(st)` (`gui/composer_flow.go:20-23, :59`) that already scrubs
the seed registry and clears the H5 state hook. That defer is the natural home
for whatever H6 chooses to do at exit — and `composerFlowExit`'s doc comment
records that a *second* defer costs 96 B of flash under TinyGo, so H6 adds to
this one rather than beside it.

The existing per-digest provenance set is the model for the new field:
`phraseDigests map[[32]byte]struct{}` (`gui/composer_state.go:53`), keyed by
digest value, never by path index, because "Remove path" splices the slice
(`gui/composer_state.go:34-53`). A preimage store keyed by digest inherits that
reasoning exactly: `map[[32]byte]hashlockMaterial` where the key is H and the
value carries X, the phrase bytes and the method. `composerNotePhraseDigest`
(`gui/composer_state.go:280`) is the one insertion point and it allocates the
nil map; H6's store needs the same nil-guard, for the same reason (a nil-map
assignment panics in the GUI goroutine at the moment the operator holds to
confirm a hash that gates funds).

`composerAnyPathByPhrase(st)` (`gui/composer_state.go:299`, walking
`st.list.Paths` and testing `*p.Hash` against the set) is the predicate shape H6
reuses for "does any CURRENT path carry a digest whose preimage we hold" — which
is what decision 6's "not on any path, will not be cut" line inverts.

**Path (b) — payload-delivered.** The unsealed container is `sysw`, not `seal`.
`sysw.Classify` (`sysw/record.go:111`) dispatches: reserved prefixes first
(`sysw/record.go:14-21`, `text:`/`pass:`/`tx:`), then `IsComposerRecord`
(`sysw/composer_records.go:58`) for `key:`/`hash:`/`now:`, then
`classifyConstellation`. An ms1 preimage string reaching that last arm is refused
today by `isStrictMs1` (`sysw/classify.go:116-127`), whose final line is
`return err == nil && !codex32.IsPreimage(c)` — H0's inertness. Admission is a
separate table, `admitted` (`gui/sysw_admit.go:32`), whose `progWalletPolicy` row
(`gui/sysw_admit.go:64-73`) already carries `ClassKey`, `ClassHash` and
`ClassNow` and is the only program that does.

The decoder for path (b)'s ms1 form already exists and has no caller:
`codex32.DecodeMS1Preimage` (`codex32/mspayload.go:113`) returns the 32 bytes of
an unshared 33-byte `0x03` single and refuses everything else with
`errMSBadPrefix`/`errMSBadLength`. `DecodeMS1` stays unchanged and keeps refusing
`0x03` at all five of its callers — H2 §6 is explicit that its callers all treat
the result as a seed. H6 is the first consumer `DecodeMS1Preimage` was written
for.

**Provenance and lifetime, side by side.**

| | (a) composer-native | (b) payload-delivered |
| --- | --- | --- |
| holds | phrase bytes, method, X, H | X and H (ms1 form); phrase, method and X (phrase-record form) |
| origin | typed on this device, this composition | `me sysw pack`, unsealed flash |
| lifetime | one `composerFlow` call, dropped by `composerFlowExit` | the session's `syswSession.records`, which outlives any one flow |
| trust | the device derived it; `chars: <n>` and the digest were shown | declared; the digest is re-derivable from X and checkable against `hash:` records |
| in flash? | never | **yes, in cleartext** — the container is unsealed |

That last row is the load-bearing asymmetry: (a)'s secret never touches flash and
(b)'s always does. It is what decision 7's transit warning exists for, and it is
why the two paths cannot share one warning.

### Open questions

- **Q1.1** Does the composer store the phrase BYTES, or only X and the method?
  A plate in the phrase form needs the phrase; a plate in the string form needs
  only X. Decision 1 says the phrase form is offered "only when the device knows
  the phrase", which implies the bytes are kept — but not that they must be.
- **Q1.2** Is the store keyed by digest (mirroring `phraseDigests`) or carried on
  the path? A digest key means two paths sharing one phrase share one entry, and
  a phrase re-typed as 64 hex still matches — both of which
  `gui/composer_state.go:34-53` argues are the right answers for provenance. It
  is not stated that they are the right answers for *plates*.
- **Q1.3** What happens to the store when the operator changes a path's hash away
  from a phrase digest, or removes the path? Nothing deletes from `phraseDigests`
  by design ("a digest no path carries is simply never matched"); a preimage the
  composition no longer uses is a different object — decision 6's review already
  has copy for it ("not on any path, will not be cut"), but only for the
  payload-delivered case.
- **Q1.4** Does path (b)'s material live in `composerState` at all, or is the
  "Hashlock plates" flow stateless over `ctx.sysw`? Decision 4 says that flow cuts
  "without a composition", which suggests the latter.
- **Q1.5** May a composition hold more than one phrase? `hashlockOtherPathLine`
  (`gui/composer_hashlock.go:114-130`) already warns that another path carries a
  different hash, calling two phrases "a legal composition ... a backup burden
  the operator must choose knowingly". H6 turns that burden into a plate count.

---

## §2. Host records and `me sysw pack`

### Decisions applied

Decision 3: the payload may carry EITHER the ms1 preimage string OR a **new**
phrase-and-method record, Rust first with vectors, then Go. Decision 7:
`me sysw pack --pack-preimage`, one flag for both record kinds, in the pattern of
`--seal-secret`; without it the H1b refusal stands, its text gaining *"re-run with
--pack-preimage if that is what you intend"*; with it, a transit warning and a
warning when the flag is passed with no preimage/phrase record, or when a
preimage's digest matches none of the payload's `hash:` records.

### Mechanism

**The siblings, looked at first.** The three shipped composer record classes are
defined once in Rust (`crates/me-cli/src/sysw/composer_records.rs:28-32`:
`KEY_PREFIX = "key:"`, `HASH_PREFIX = "hash:"`, `NOW_PREFIX = "now:"`) and ported
predicate by predicate to Go (`sysw/composer_records.go:24-31`, whose header says
"ported ... from the host's `crates/me-cli/src/sysw/composer_records.rs`"). Their
body conventions are NOT uniform, and that matters for the new kind:

| record | body encoding | parser |
| --- | --- | --- |
| `key:` | hex of the UTF-8 text `[fingerprint/path]xpub` | `sysw.ParseKeyRecord` |
| `hash:` | the 32-byte digest as **64 raw lowercase hex characters** — no second layer | `sysw/composer_records.go:100` |
| `now:` | hex of the UTF-8 text `<seconds>[,<height>]` | `sysw/composer_records.go:136` |

So `hash:` is *already* the "raw hex of a fixed-width binary value" idiom, and
`key:`/`now:` are the "hex of a UTF-8 text" idiom. `unhexLower`
(`sysw/composer_records.go:82`) is the shared hex rule: even length, `[0-9a-f]`
only, uppercase refused rather than folded, because SPEC_systemwide_payloads §6.6
hashes the wire spelling.

A preimage record fits the `hash:` idiom exactly (`preimage:` + 64 raw lowercase
hex). A phrase-and-method record is text and fits the `key:`/`now:` idiom (hex of
a UTF-8 body), and its body needs at least two fields — the method and the phrase
— which means an internal separator, and `now:` has already chosen `,` for that
job.

**The ms1 form needs no new record at all.** An ms1 preimage single is a bare
constellation string; what refuses it is `isStrictMs1`'s H0 clause
(`sysw/classify.go:126`) on the device and `unknown_reason`'s preimage arm on the
host (`crates/me-cli/src/sysw/mod.rs:209-211`, reached through `admit_check` at
`:463` from `pack_with` at `:335`). Admitting it means giving it a class — a
fourth composer class — not a prefix.

**The refusal that is being gated.** Today `me sysw pack` refuses a preimage
plate by index with the H1b text at `crates/me-cli/src/main.rs:2805-2810`:

> record {i} (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), not
> a seed record; this container cannot place one yet. A preimage backs a hashlock
> spend path, not a wallet — keep it with the policy it unlocks, and do not
> re-encode it as entropy.

Decision 7's addendum lands on that string. The `--seal-secret` pattern it copies
is at `crates/me-cli/src/main.rs:932-937`: a boolean flag, checked against a
content predicate, whose refusal ends *"Re-run with --seal-secret if that is what
you intend."*

**The sealing decision is content-derived, and this is where it bites.**
`decide_sealing` (`crates/me-cli/src/main.rs:2353-2410`) seals iff some record's
class `is_secret()`; on the device the mirror is `Class.IsSecret()`
(`sysw/record.go:60-66`: Mnemonic, Codex32Secret, Passphrase). If a preimage
class is marked secret, `me sysw pack` will SEAL the payload by default — which
decision 3 puts out of scope. If it is not marked secret, the payload stays
unsealed and a spend secret sits in flash in cleartext, which is exactly what
decision 7's transit warning names. There is no third option in the shipped code;
one of the two must be chosen deliberately. (Related: `decide_sealing` prints the
CLASS names of secret records, never the records — a new class's name would
appear on stderr, which is fine, but a class named for what it is would be
telling.)

**Where the `hash:`-match warning gets its answer.** The digest of a candidate
preimage is `sha256(X)`; `hash:` records decode with
`sysw::composer_records`'s parser on the host and `sysw.ParseHashRecord`
(`sysw/composer_records.go:100`) on the device. `me` already walks the whole
record vector by index in `admit_check`
(`crates/me-cli/src/sysw/mod.rs:463-470`), so the cross-record check decision 7
asks for has a place to stand — but note that `admit_check` today is a
per-record predicate with no payload-wide state, while the "at most one `now:`"
rule lives in `pack_with` (`SPEC_wallet_policy_composer.md` §6a names both sites).
A digest-vs-`hash:` check is payload-wide and belongs with the latter.

**Lockstep.** The vendored corpus convention is
`sysw/testdata/record_class_vectors.json` plus
`record_class_vectors.provenance.json` (repo, remote, path, commit, sha256, row
count `47`, `recorded_at`), regenerated by the Rust primary's own test
(`crates/me-cli/tests/sysw_composer_records.rs`) and never edited by hand. A new
record kind adds rows there and re-pins the sha. The ms hashlock corpus is
pinned the same way in the fork at `hashlock/testdata/hashlock-v0.8.json`
(H2 §7.1, sha `a46c197a…1d30`).

### Open questions

- **Q2.1** The phrase record's exact wire shape. Candidates:
  `preimage-phrase:<hex of "hardened,<phrase>">` (the `now:` idiom, one separator),
  `preimage-phrase:<hex of the labelled QR text of decision 2>` (one text, two
  consumers), or two records (`preimage-method:` + `preimage-phrase:`). Decision 2
  pins the *plate/QR* spelling and explicitly leaves the record spelling open.
- **Q2.2** The ms1 preimage record's spelling: a new class over the bare ms1
  string (relaxing `isStrictMs1`'s H0 clause under a program-scoped rule), or a
  prefixed `preimage:<64 hex>` record that never carries an ms1 string at all.
  The second leaves H0 untouched everywhere; the first is what an operator holding
  `ms hashlock --out` output actually has in hand.
- **Q2.3** Is the new class **secret** for `Class.IsSecret()` /
  `decide_sealing`? Both answers have a cost (see the mechanism above); neither
  is settled by decisions 3, 7 or 9.
- **Q2.4** How does a payload phrase record spell its method? `ms hashlock`'s card
  prints `preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = "ms-hashlock-v1",
  iterations = 100000, dkLen = 32)` (`crates/ms-cli/src/cmd/hashlock.rs:281-288`);
  `--json` emits `{kdf, hash, salt, iterations, dklen}` (`:322`); the device's
  `hashlockMethod.String()` returns `sha256`/`hardened`
  (`gui/composer_hashlock.go:35-40`); decision 2's QR line wants
  `pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32`. Four
  spellings of one fact. Which is normative on the wire?
- **Q2.5** Does the device VERIFY a payload phrase record by re-deriving X
  (~10 s hardened) before offering it, or take the record's word? Verification is
  the only thing that makes the "digest matches no `hash:` record" warning
  meaningful on the device rather than only on the host.
- **Q2.6** Does `--pack-preimage` also relax `me seal`'s `RecordError::PreimagePlate`
  (`crates/me-cli/src/seal/record.rs:130-136`)? Decision 3 says the sealed payload
  is out of scope, which argues no — but the two refusals share their text today
  and would then diverge.
- **Q2.7** Is the auto-appended `now:` rule extended? `me sysw pack` appends
  `now:` when the records include a `key:` or `hash:` and no `now:` of their own
  (`crates/me-cli/src/main.rs:1730-1760`). A payload of preimage records alone is a
  composer payload by any reading and would today get no bound.

---

## §3. Device admission and the composer screens

### Decisions applied

Decision 3: only the unsealed payload, only the Wallet Policy program; H0's
inertness elsewhere untouched. Decision 4: a separate "Hashlock plates" flow
under the Wallet Policy door for payload-delivered material, and `Which hash?`
also lists payload preimage/phrase records as pickable digests.

### Mechanism

**The classifier and the admission table are separate rules, and the split is the
design.** `gui/sysw_admit.go:1-14` states it: *"admission is (class -> program).
The container selects flags, never admission."* So a new preimage class can be
admitted at `progWalletPolicy` alone (`gui/sysw_admit.go:64-73`) and refused
everywhere else without touching any other program's row — the same move C12 made
for the seed classes. What that row does NOT control is `isStrictMs1`
(`sysw/classify.go:116`), which is the *classifier*: today it answers `false` for
a preimage single, so the record never reaches any class and goes inert
(`sysw/descriptor.go:46-48`, quoted in `gui/composer_hash.go:43-46`: "stays in the
session, is offered to nobody, and reaches no screen"). Admitting the ms1 form at
one program still means the classifier stops calling it Unknown *globally*, and
the admission table is then the only thing keeping it out of Backup Wallet.

**The door counts what it can see.** `composerDoorCounts`
(`gui/composer_door.go:41-56`) walks `s.records` and counts `ClassKey` → keys,
`ClassMnemonic`/`ClassCodex32Secret` → seeds, `ClassUnknown` → inert;
`composerDoorLines` (`:63-90`) turns those into §8r's lead, and
`composerDoorFlow` (`:98`) builds the rows — "Scan cards", "From payload" (gated
by `composerDoorHasConsumablePolicy`, `:93-97`), "Build a new policy". A fourth
row for "Hashlock plates" is a two-line change *there*; what it needs is a
predicate of the same shape as `composerDoorHasConsumablePolicy` and a key-state
line of the same shape as §8r's, because a door row whose route dead-ends is the
F-437 defect that the door was built to remove.

**`Which hash?` is already label-keyed and has room.** `composerHashRows`
(`gui/composer_hash.go:157-175`) builds `composerHashRowSet{labels, lead, digests,
phraseRow, hexRow, noneRow}`, and `composerHashEdit` (`:184-224`) dispatches on
those names with a `default:` that PANICS rather than assigns — H2 §5's fix for
the r2 C-4 defect, where the shipped index-keyed switch cleared the lock when a
row moved. Adding payload preimage digests means either extending `digests`
(which is `composerPayloadDigests`, `gui/composer_hash.go:47-65`, reading
`ClassHash` records only) or adding a second named row band. The `taking`
predicate at `:194` (`sel < len(rows.digests) || sel == rows.phraseRow ||
sel == rows.hexRow`) fires §8i and must be extended in lockstep, and
`composerPickScreenMaxRows` is checked against the longest row set (H2 §5).

Note the row form: `composerHashRow` (`gui/composer_hash.go:38-41`) prints
`hash %d  %s..%s` — first 8 and last 8 — because *"a full 64-hex row would be CUT
rather than wrapped at the 436 px label budget, and a cut digest is worse than an
elided one"*. A preimage row inherits that constraint.

**What stays inert.** H0's five `DecodeMS1` callers (`gui/ms1_decode.go:22`,
`gui/codex32_polish.go:106`, `gui/singlesig_verify.go:185`,
`gui/multisig_verify.go:1237`, `bundle/verify.go:138`) keep refusing `0x03`. The
scan door refuses upstream (`gui/scan.go:89`,
`codex32.New(...) == nil && !codex32.IsPreimage(s)`), and `engraveCodex32`
(`gui/codex32_polish.go:232-235`) refuses at the choke point every route to
`backup.EngraveSeedString` passes through, with the shipped body *"This record is
a hashlock preimage, not a seed. It is not engraved as one."* The sealed payload's
own refusal (`gui/unlock_kdf.go:415-420`, with its noun at `:433`; H5 §5) is untouched: `seal.AdmitSection`
belongs to the frozen Sealed Payload (`seal/record.go`), a different container
from `sysw`.

**A collision H6 inherits, stated because it now has consequences.**
`codex32.IsPreimage` (`codex32/mspayload.go:94-101`) does not consult the id: a
plain BIP-93 33-byte seed beginning `0x03` is indistinguishable from a preimage
plate, roughly 1 in 256 of 33-byte seeds, and is refused as a preimage
(`codex32/mspayload.go:78-92`). H0 accepted that in the safe direction — a
refusal costs a re-encode, a wrong cut exposes a spend secret. H6 makes the
direction *unsafe*: the same misclassification now routes such a string TO a
plate flow that engraves it as a preimage. The polarity of the trade reverses
with this stage.

### Open questions

- **Q3.1** Does the composer's `Which hash?` show a payload PHRASE record before
  deriving it? Hardened derivation is ~10 s (`gui/composer_hashlock.go` countdown;
  9,715 it/s measured, brainstorm §3.4), so either the row shows no digest until
  the operator picks it, or the screen pays 10 s per phrase record to build itself.
- **Q3.2** Does a payload preimage record's digest join `phraseDigests` (making
  §8h's phrase form fire for it), or does §8h need a third form? H5 §2.5 already
  records that the phrase form OVERCOUNTS on some wallets and that counting
  exactly "would need three variants of this body".
- **Q3.3** Is the "Hashlock plates" door row shown always, or only when the
  payload holds preimage/phrase records? `composerDoorHasConsumablePolicy` gates
  "From payload"; "Scan cards" and "Build" are unconditional.
- **Q3.4** Is the ms1 preimage form admitted through the classifier (a real
  class) or read straight out of a prefixed record? See Q2.2 — the device
  consequence is whether `isStrictMs1`'s H0 line changes.
- **Q3.5** Does the "Hashlock plates" flow re-use the payload's `hash:` records to
  say which policy a preimage belongs to, and what does it show when no `hash:`
  record matches?
- **Q3.6** Is anything done about the 1-in-256 33-byte-seed collision now that
  its consequence has flipped (see above), or is it accepted a second time with
  the reversal recorded?

---

## §4. Retention and the Done review

### Decisions applied

Decision 9: RAM retention of the preimage, phrase and method for the
composition's lifetime is accepted — secret-handling, non-gating under the
2026-08-27 ruling, F-483 territory. Decision 6: cut at Done, after a review step
offering per-plate form and QR, and the review lists a payload-delivered preimage
that no path uses as *"not on any path, will not be cut"*.

### Mechanism

**Retention has one owner and one exit.** `composerFlow`
(`gui/composer_flow.go:47-131`) builds the state at `:48`, installs the H5 hook
at `:58`, and defers `composerFlowExit(st)` at `:59` — installed at the top,
before any secret can exist, *"so every exit below (a Back, a refusal, a ctx.Done
unwind, a panic) is covered without an implementer remembering to add one to a new
return"*. `composerFlowExit` (`:20-23`) calls `st.reg.scrub()` and
`clearComposerStateHook()`. H6's material goes into that same function; a second
defer measured +96 B of flash and the comment at `:53-57` says why.

F-483 (`design/FOLLOWUPS.md:16003`) already records that the typed phrase lives in
`kbd.Fragment`, an immutable Go string, for the life of the phrase screen and
across every Back via `initial` — so the phrase is *already* unwipeable today,
before H6 stores it deliberately. Decision 9 accepts the class; the follow-up is
where the measurement goes.

**Where "Done" is, exactly — and there are two of them.** The shape's Done is
`composerShapeFlow`'s `default:` arm (`gui/composer_shape.go:434-445`), which
validates the path list, runs the key-order step, fires §8h when
`composerEveryPathHashed(st.list)` and returns. The engrave Done is
`composerEngraveStep` (`gui/composer_flow.go:335-394`): form pick → cards →
Full/Watch-only → census (`confirmReviewScreen(ctx, th, "Plates To Cut",
composerCensusLines(...))`, `:389-390`) → `bundleEngrave(ctx, th, "Wallet Policy",
cards, "", "")` (`:393`). Decision 4's "cut at Done inside the composer's bundle"
and decision 6's "review step offering per-plate form and QR" both point at the
second — the census screen is the review surface that already exists, and
`composerCensusLines` derives its counts *through* `bundlePlatePlan`, the same
function `bundleEngrave` loops (`gui/composer_census.go:9-18`), so a plate the
review names is a plate that gets cut.

**Per-plate form and QR already has a precedent, and it is the wrong one.**
`bundleEngrave` (`gui/bundle_flow.go:616-657`) shows, per plate, a `ChoiceScreen`
titled `Card %d of %d | Plate %d of %d` with labels from `validateMdmkStrings`
(`gui/gui.go:2626-2648`), which offers `TEXT + QR`, `TEXT ONLY`, `QR ONLY` for a
single string. That is a per-plate style picker of exactly the shape decision 6
describes — but it builds its QR with `engrave.QR`
(`backup/backup.go:462`, via `backup.Paragraph.QR`), the content-dependent
engraving that `backup/passphrase.go:112-114` forbids for a secret. See §5.

**The Back contract this review inherits.** H2 §4.6 is normative about the phrase
route (`composerHashEdit` returns `false` ONLY for Back at `Which hash?`, because
`false` at path creation REMOVES the path, `gui/composer_shape.go:269`). A review
screen that can decline a plate needs its own statement of what Back means — the
composer's own rule is "going back should lose nothing"
(`SPEC_wallet_policy_composer.md` §7b) and `confirmReviewScreen` returning false
today sends `composerEngraveStep` back to the form pick.

### Open questions

- **Q4.1** Which "Done"? The shape's Done (`gui/composer_shape.go:434`) or the
  engrave Done (`gui/composer_flow.go:335`)? Decision 4's "cut at Done inside the
  composer's bundle" reads as the second; decision 6's "cut at Done (composer)"
  is not explicit.
- **Q4.2** What happens to a plate the operator DECLINES at the review — is it
  dropped from the bundle (and the census recount shown), is the whole cut
  aborted (`bundleAbortWarning`'s existing set-level rule,
  `gui/bundle_flow.go:625-631`), or is the decline remembered so Back and forward
  reproduce it?
- **Q4.3** How many hashlock plates per bundle? One per distinct preimage, one per
  path, or one per phrase? §1's Q1.5 and `hashlockOtherPathLine` make more than
  one legal.
- **Q4.4** Does the review offer form and QR PER PLATE (as `bundleEngrave` does,
  one screen per plate) or ONCE for all hashlock plates? The former is the
  shipped shape; the latter is one screen instead of N.
- **Q4.5** Is the "not on any path, will not be cut" line a row on the review, a
  separate warning screen, or a refusal to enter the flow when NOTHING will be
  cut?
- **Q4.6** Is the preimage retained across a Back out of `composerEngraveStep`
  and into the shape again (so the operator can add a path and come back), or
  dropped at the first exit from the engrave step?
- **Q4.7** Does the census (`composerCensusLines`) count hashlock plates, and does
  it distinguish them from the public plates in the same set the way
  `bundlePlateMark` refuses to mark a `cardMS1` (`gui/bundle_flow.go:566-575`)?

---

## §5. The plate layout and QR text

### Decisions applied

Decision 5: a DEDICATED layout for both forms, never the seed layouts; title band
`HASHLOCK PREIMAGE` / `NOT A SEED`; then the locator header; then the body (the
ms1 string, or phrase + method line); QR on the right when chosen; sized so a
100-character phrase fits at the smallest rung. Decision 1: the QR, when chosen,
ALWAYS encodes the phrase and method as plain text, never the ms1 string, and is
offered only with the phrase form. Decision 2 fixes the QR text as labelled lines
with a version tag, e.g. `hashlock v1` / `method: pbkdf2-hmac-sha256
iterations=100000 salt=ms-hashlock-v1 dklen=32` (or `method: sha256`) /
`phrase: <phrase>` last, no trailing newline.

### Mechanism

**The right model is `backup.Passphrase`, not `backup.Text`.** The fork has four
plate layouts: `Seed` (words + SeedQR, `backup/backup.go:16-24`), `SeedString`
(string only, `:26-31`), `Text` (paragraphs, optional per-paragraph QR,
`:33-53`) and `Passphrase` (`backup/passphrase.go:23-49`). `Passphrase` is the
one built for a secret the operator must transcribe: it engraves verbatim, never
uppercased; it renders every space as a visible mark `SpaceMark = '\x1f'`
(`backup/passphrase.go:21`) with a legend `"\x1f = SPACE"` (`:168`) because *"one
space and two look identical ... while 'hunter2 ' is a different wallet from
'hunter2'"*; it puts fingerprints in the 10 mm screw-hole bands (`:239-266`); and
its QR is opt-in (`QR bool`, `:47`) and built with `engrave.ConstantQR`
(`:112-121`), never `engrave.QR`, *"the latter engraves in a content-dependent
pattern and would leak the secret through timing"*. Its QR also encodes the
passphrase EXACTLY as entered — real `0x20` spaces, never `SpaceMark`
(`:93-100`) — because *"a scanner that saw it would hand a wallet different
bytes, silently opening a different wallet"*. Every one of those five decisions
transfers to a hashlock plate unchanged.

**Geometry, MEASURED at fork `fb0dd04`** (`backup.CharsPerLine` /
`backup.LinesPerPlate`, `backup/backup.go:88-96`, at `sh2.Params()`:
`Millimeter = 6400`, `StrokeWidth = 1920`, `plateSize = 85`, `outerMargin = 3`):

| rung | chars/line (full width) | lines/plate |
| --- | --- | --- |
| 6.0 mm | 19 | 13 |
| 5.0 mm | 23 | 15 |
| 4.4 mm | 26 | 17 |
| 3.8 mm | 31 | 20 |
| 3.4 mm | 34 | 23 |
| 3.0 mm | 39 | 26 |

The ms1 preimage string is **75 characters** (SPEC_ms_hashlock §1; measured on the
corpus row `ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c`),
so the string form is 4 lines at 6.0 mm and 2 at 3.0 mm — it fits with room to
spare at every rung. A 100-character phrase is 6 lines at 6.0 mm and 3 at 3.0 mm
at full width. Neither is the problem.

**The QR is the problem, and it is measured, not argued.** `engrave.ConstantQR`
REFUSES any code larger than 37 modules (`engrave/engrave.go:418-426`:
`if dim > 37 { return nil, fmt.Errorf("engrave: constant QR size too large: %d",
dim) }`), because `bitmapForQRStatic` (`:394-414`) tabulates 21/25/29/33/37 only
and larger versions would hit its `panic("unsupported qr code version")`. The
comment states the bound's origin: *"ECC-L caps at 106 bytes and the passphrase
caps at 100 (spec O6) ... Raise both together or not at all."*

MEASURED with the fork's own `qr.Encode(s, qr.L)`, byte mode, ECC-L:

| payload | bytes | modules | ConstantQR |
| --- | --- | --- | --- |
| bare 100-char phrase (today's passphrase plate) | 100 | 37 | accepted |
| `phrase: ` + 100 chars | 108 | 41 | **REFUSED** |
| `hashlock v1` + `method: hardened` + `phrase: ` + 100 | 137 | 45 | **REFUSED** |
| `hashlock v1` + `method: sha256` + `phrase: ` + 100 (decision 2's short form) | 135 | 45 | **REFUSED** |
| `hashlock v1` + the full pbkdf2 method line + `phrase: ` + 100 (decision 2's long form) | 194 | 53 | **REFUSED** |

The module thresholds at ECC-L, measured: ≥1 → 21, ≥18 → 25, ≥33 → 29, ≥54 → 33,
≥79 → 37, ≥107 → 41, ≥135 → 45, ≥155 → 49, ≥193 → 53. **106 bytes is the ceiling
for a constant-time QR on this device.** Decision 2's shortest legal payload with
a 100-character phrase is 135 bytes.

Physical sizes at `passphraseQRScale = 3` (0.9 mm modules), MEASURED: 37 modules
= 33.30 mm, 41 = 36.90, 45 = 40.50, 49 = 44.10, 53 = 47.70. At the free-text
plate's `freeTextQRScale = 2` (`backup/fit.go:19`, 0.6 mm modules): 53 modules =
31.80 mm. So the *space* exists on an 85 mm plate — 53 modules at scale 2 is
smaller than today's passphrase QR at scale 3. What does not exist is a
constant-time engraving for it.

Three exits, none free:

1. **Raise `bitmapForQRStatic` and `ConstantQR` together** to v6–v9. Its own
   comment invites this ("Raise both together or not at all"), but v7 and up need
   six alignment patterns rather than one (`engrave/engrave.go:401-412` tabulates a
   single marker at `(dim-9, dim-9)` for 25–37), so it is a real change in
   `engrave` with its own goldens and its own constant-time argument
   (`constantTimeQRModules`, `constantTimeStartEnd`, `findPath`).
2. **Use `engrave.QR`** (`engrave/engrave.go:277`), which has no size bound and
   scans row by row — and leaks the phrase through the toolpath, which is the one
   thing `backup/passphrase.go:112-114` says must not happen for a secret.
3. **Shrink the payload** below 107 bytes: a shorter method token, a shorter
   label set, or a lower phrase cap for the QR form specifically. `method:
   hardened` + a 100-char phrase is still 137; only a bare or near-bare phrase
   fits.

**The QR-on-the-right column budget, if the QR stays at 53 modules.** Usable
width is 85 − 2×3 = 79 mm. A 47.70 mm QR (scale 3) plus a 2 mm gap
(`passphraseQRGap`, `backup/passphrase.go:74`) leaves 29.3 mm of text column ≈ 14
characters per line at 3.0 mm (derived from the measured 39 chars at 79 mm). A
100-character phrase is then 8 lines, and the 73-character method line another 6
— before the title band and the header. At scale 2 (31.80 mm) the column grows to
45.2 mm ≈ 22 characters, and the same content is 5 + 4 lines. Note also that the
passphrase plate stacks its QR BELOW the text (`passphraseLayoutFor`,
`backup/passphrase.go:283-292`: `l.envY = l.textY + l.blockH + gap` at `:289`), so "on the right"
is new geometry, not a parameter change.

**Band budget for the title and header.** `MaxTitleLen = 18`
(`backup/backup.go:71`), enforced at every rung by the free-text plate's
`TestTitleCapFitsAtEveryRung` (`backup/freetext.go:11-28`: at 6.0 mm an
18-character title clears the screw holes by 0.620 mm and 20 characters do not
clear at all). `HASHLOCK PREIMAGE` is 17 characters and `NOT A SEED` is 10, so
both fit a title row — but only one row is a title. The passphrase plate's own
band rule is *at most two lines per band*, because a band offers
`innerMargin 10 − outerMargin 3 = 7 mm` and three 3 mm lines need 9 mm
(`backup/passphrase.go:250-254`), and its own `TestPassphraseBandBudget` caps a
metadata line at 64 mm — the shipped 32-character `FINGERPRINTS TYPED, NOT
VERIFIED` is already AT that ceiling with zero spare
(`backup/passphrase.go:187-196`, which records a spec string that measured 8 mm
over and went RED).

### Open questions

- **Q5.1** Which exit does the QR take? (Raise `ConstantQR`; use the leaky
  `engrave.QR`; or shrink decision 2's text.) This is the most consequential
  unsettled question in the stage — see the report.
- **Q5.2** What exactly is in the "locator header"? Decision 4 names *path number,
  digest, policy csid* for the composer form and *digest, the payload's `hash:`
  position when matched, template id when the payload carries one* for the flow
  form. See Q5.3 for `csid`.
- **Q5.3** What is "policy csid"? The composer's identity surface is
  Template-ID / Policy-ID (32 hex) and `mk1 stub (template)` / `mk1 stub (policy)`
  (8 hex) — `composerStubLines`, `gui/composer_stub.go:30-72`, via
  `md.FormAwareIdChunks` / `md.FormAwareStubChunks`. `md`'s literal *chunk set id*
  is 20 bits, unexported (`deriveChunkSetID`, `md/identity.go:31-33`), and exists
  only in a CHUNKED md1 header (`md/chunk.go:45-52`); the only exported `csid` in
  the tree is `mk`'s per-card `chunk_set_id` (`gui/bundle.go:43-51`). Whichever is
  meant, it must be spelled with the label the stub screen uses, or the operator
  cannot match the plate to the card.
- **Q5.4** Is the policy id available at Done, and where from? A KEY-LESS or
  PARTIALLY seated composition has no Policy-ID — `composerStubLines` adds the
  keyed pair only when `len(keyedChunks) > 0` (`gui/composer_stub.go:62-72`), and
  `composerFormsFor` (`gui/composer_engrave.go:40-61`) says a partially seated
  policy *"has no id yet"*. So the header field can be empty at the moment the
  plate is cut.
- **Q5.5** For the flow form, where does "template id when the payload carries
  one" come from? The payload's md1/mk1 records reach the composer through
  `composerCardSources`, not through the "Hashlock plates" flow, which decision 4
  says runs "without a composition".
- **Q5.6** Does the phrase form get `SpaceMark` and its legend? A hashlock phrase
  is printable ASCII with real spaces (`hashlock.ValidatePhrase`,
  `hashlock/hashlock.go:92-111`) and the verbatim rule is as strict as the
  passphrase's — H2 §2 forbids every normaliser by name. Reusing `SpaceMark`
  means reusing its glyph and its legend row.
- **Q5.7** Is there a title band AND a `NOT A SEED` row, or one row of two
  phrases? `backup.Text` has one `Title` and one `Footer`
  (`backup/backup.go:44-53`), both capped at 18 by convention; `HASHLOCK PREIMAGE`
  / `NOT A SEED` is two rows.
- **Q5.8** Does the string form get a QR at all? Decision 1 says the QR toggle is
  offered only with the phrase form — so a plate carrying the ms1 string has NO
  machine-readable copy, while `me`'s own preview sidecar and every md1/mk1 plate
  do. Deliberate, or an omission worth naming in the spec?
- **Q5.9** Which font/face? The passphrase plate uses `plate.Font` at a fixed
  6.0 mm em that does not scale with length (`backup/passphrase.go:54-61`), the
  free-text plate walks `FontSizes` (`backup/backup.go:83-87`). Decision 5's
  "smallest rung" implies the ladder.

---

## §6. The free-text and passphrase warning

### Decisions applied

Decision 8: text stays free — WARN (a confirm naming what the text looks like: an
ms1 string) then cut as typed. Never refuse.

### Mechanism

**There is no guard today, in either program.** A grep of
`gui/freetext_flow.go` and `gui/passphrase_flow.go` at `fb0dd04` for
`IsMS1Shaped`, `codex32.New`, `ms1` and `Preimage` returns nothing: an operator
who types an ms1 preimage string into Engrave Text gets it cut verbatim, and the
same in Engrave Password. That is what decision 8 preserves — it adds a
confirm, not a refusal.

**The predicate already exists, in the right shape.** `hashlock.IsMS1Shaped`
(`hashlock/hashlock.go:122-148`) is the HOST's `looks_like_ms1` ported byte for
byte: trim, ASCII-lowercase, strip the display separators (space, tab, CR, LF,
`-`, `,`), then require ≥ 48 characters, an `ms1` prefix and only bech32
characters — **no checksum**. H2 §2 rule 3 explains why the shape test and not a
parse: *"a grouped or mistyped plate the host refuses would be derived from on the
device, and the two would disagree on what a phrase is."* A grouped plate — which
is what `ms hashlock`'s card prints (`crates/ms-cli/src/cmd/hashlock.rs:340,348`,
`render_grouped`) — is exactly what an operator retypes, so a checksum test would
miss the case the warning exists for. `codex32.IsPreimage`
(`codex32/mspayload.go:94`) is the narrower, checksum-bearing test and would
answer `false` for a plate typed with its display separators intact.

**Where the confirm goes.** Both programs have a confirm surface before the cut:
`ftConfirmFlow` (`gui/freetext_flow.go:1362-1402`, title `Confirm`, paged) and
`ppConfirmFlow` (`gui/passphrase_flow.go:497`). The composer's own confirm-to-
proceed idiom is `composerConfirmScreen` + `composerConfirmBody`
(`gui/composer_shape.go:77`, `gui/composer_copy.go:36-38`, whose body ends
`"\n\nHold button to confirm."` at `:37`). The free-text program's OK is a tap, not a HOLD.

**The fit gate.** Every new modal body is measured, not budgeted:
`assertModalBodyFits` (`gui/modal_fits_test.go:202`) renders the specific body
and measures its headroom with `modalHeadroom` (`:183`), requiring ≥
`modalBodyMargin = 80` normalised characters (`:52`); the file's own comment says
there is no capacity constant because capacity depends on how the words wrap,
"which is why this measures each body instead of budgeting all of them"
(`:33-35`). A new body goes in that table.

**Two asymmetries worth naming.** First, the warning fires on shape, and the
shape test is deliberately wider than a parse — so a free-text plate that
legitimately begins `ms1` and is 48+ bech32 characters raises it. Second, an
`ms1` preimage typed into Engrave PASSWORD becomes a BIP-39 passphrase, which is
a different failure from engraving it as text: `passphraseEntryFlow`
(`gui/passphrase_flow.go:74`) feeds a wallet, not a plate.

### Open questions

- **Q6.1** Which predicate: `hashlock.IsMS1Shaped` (shape, no checksum, catches
  the grouped plate) or `codex32.IsPreimage` (checksum, catches only a clean
  string)? Decision 8 says "an ms1 string", which the first tests and the second
  does not.
- **Q6.2** Does the warning distinguish a *preimage* plate from any ms1 string?
  `IsMS1Shaped` cannot — it never parses. Saying "this looks like an ms1 string"
  is true and vague; saying "this looks like a hashlock preimage plate" needs the
  checksum and misses the grouped case.
- **Q6.3** Does the same warning fire in Engrave Password, and does it say
  something different there (the string becomes a wallet's passphrase, not a
  plate)?
- **Q6.4** Where in the flow — at OK on the text entry screen, or at the confirm
  screen that already exists? And is it once per composition or once per edit?
- **Q6.5** Is it a tap-to-continue (the free-text program's idiom) or a HOLD (the
  composer's)? Decision 8 says "a confirm", not which.
- **Q6.6** Does the warning also fire for a payload-sourced free text
  (`engraveTextFlowFrom`, `gui/freetext_flow.go:1485`), where the operator never
  typed the string and the host already had a chance to refuse it?

---

## §7. Tests, gates and acceptance

### Decisions applied

Decisions 2 (exact QR spelling pinned in spec and corpus), 3 (Rust first with
vectors, then Go), 5 (a fit gate for the 100-character phrase at the smallest
rung), 7 (host warnings).

### Mechanism

**Rust-primary is not optional here.** The new record kind is normative payload
behaviour, so it lands in `crates/me-cli/src/sysw/composer_records.rs` with
`CASES` rows (`:306`) first, and the Go port
(`sysw/composer_records.go`) follows. The corpus is regenerated by the Rust test
(`crates/me-cli/tests/sysw_composer_records.rs`) into
`crates/me-cli/testdata/record_class_vectors.json`, copied to
`sysw/testdata/record_class_vectors.json`, and pinned by
`record_class_vectors.provenance.json` (repo, remote, path, commit
`c05074f1d45970ca416785dfa9d9a812aaa21dbd`, sha256
`5b3960cad7f924f6f1e7f19ef49599814733cee4874d0f5eb48c28af4cd8b312`, `vectors: 47`,
`recorded_at: 2026-09-02`) — all of which H6 re-pins.

**The QR text is a cross-repo string and needs a corpus row of its own.** It is
produced by the device and (per decision 2) will one day be parsed by
`ms hashlock`; nothing else in the tree pins a device-authored text that a host
tool must read. The `hashlock-v0.8.json` corpus is the natural home (its
`lockstep` array already names what the fork must drive in both directions), and
adding to it re-pins its sha `a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30`
and its provenance file, which H2 §7.1 requires the fork to assert as a literal.

**Plate gates that already exist and must be extended.** `backup` pins plate
output as golden bytes (`backup/text-{0..5}-shards-1.bin`,
`backup/testdata/`); the free-text plate has `TestTitleCapFitsAtEveryRung`
(`backup/freetext_test.go:75`, cited at `backup/freetext.go:17-19`) and the passphrase plate has
`TestPassphraseBandBudget` (`backup/passphrase_test.go:521`, whose 64 mm cap the
comment at `backup/passphrase.go:187-196` records a spec string failing), both of which are
GEOMETRIC assertions rather than text ones — and
`design/FOLLOWUPS.md`'s recorded lesson is that text extraction cannot see
clipping. A new layout needs both a golden and a band-budget test, and decision
5's "100-character phrase at the smallest rung" is a fit gate whose failure mode
must be a refusal (`EngraveText` refuses rather than draws what it cannot lay
out, `backup/backup.go:388-400`; `toPlate` rejects overflow).

**Device gates.** `assertModalBodyFits` (`gui/modal_fits_test.go:202`) for every
new modal body; `TestComposerCopyTableCoversEveryBody` and
`TestComposerCopyIsVerbatimFromTheSpec` (H5 §1.3-1.4) diff the shipped copy
against the spec text, so every blockquote H6 writes becomes a test; the
`composerHashRowSet` label-keyed switch needs a row-by-label test with 0, 1 and 2
payload digests (H2 §5, §7.3); `composerPickScreenMaxRows` against the longest
row set.

**Walk.** `cmd/emu/walk_hashlock_phrase.js` is the existing arm; H5 §4 rebuilt it
to read the composition state through the `!tinygo` seam
(`gui/composer_state_hook.go` / `gui/composer_state_hook_tinygo.go`, on the
`gui/frame_hook.go` model) and publish it as `window.shComposerPathHashes()`.
H5's doctrine sentence binds H6: *"a walk may READ state only to assert that what
the screen shows equals what is stored; it never drives through a hook."* H5 §4.5
also fixes the three-run pattern — unmutated PASS, plus two mutations each
recorded WITH the assertion that failed, with the tree restored and the
restoration CHECKED between runs.

**Parallel gates.** `scripts/gui-shard-test.sh <pkg> 24` (the fork's `gui`
package has zero `t.Parallel()` calls; sharding took it 493 s → 112 s) and
`cargo nextest run --locked` on the Rust side.

**Firmware size.** H5's plan-round fold recorded, at its own gated tree: shipped
1,599,208 B; baseline fork `b9a9a30` 1,597,404 B. Its lesson binds H6: *"a '0'
that a whole-image build produces is not a structural zero until something has
moved around it"* — a 0 B claim survived a spec round, two fold verifications and
a plan build gate and fell to four string literals. H6 adds a plate layout, a QR
path and a record class, so the delta is not expected to be small; it is stated
for the STAGE against a named baseline, re-measured at `fb0dd04`.

**Acceptance.** H2 §8's shape: the emulator arm is the acceptance until the
operator walks it on the flashed device. For H6 the irreducible on-device step is
that a cut plate, read back by a person and re-entered on the host through
`ms hashlock <ms1>` or `--hashlock-phrase`, reproduces the digest the composer
showed. **The SH2 has no camera** (memory: `sh2-has-no-camera.md`), so a QR the
device engraves cannot be read back by the device — only by a phone or the
operator's eyes. That is a hard limit on what any acceptance can prove
on-machine.

### Open questions

- **Q7.1** Which repo owns the QR-text corpus rows — the ms hashlock corpus
  (re-pinning its sha in the fork) or the record-class corpus? Decision 2 says
  `ms hashlock` learning to parse the text is a follow-on, which argues for
  pinning the text now and the parser later.
- **Q7.2** What is the acceptance for a plate nobody can read back on-device? A
  phone scan of the QR, a host re-entry of the typed phrase, or a
  golden-bytes-only gate with the live read-back deferred to the operator's walk?
- **Q7.3** Does the walk exercise the new flow at all, given that a walk may only
  READ state (H5 §4.1) and the plate is engraving output rather than state? The
  `freetextPlateHook` / `passphrasePlateHook` pattern
  (`gui/freetext_flow.go:1416`, `gui/passphrase_flow.go:549`) exists precisely
  because *"the confirm screen is inspectable via `op.Drawer.ExtractText`, a
  `bspline.Curve` is not"* — a third such hook is the shipped answer, and it is
  a hook that carries a SECRET.
- **Q7.4** Is a mutation gate defined for the "digest matches no `hash:` record"
  warning on both sides, since it is the only cross-record check in the stage?
- **Q7.5** Does `me bundle --preview` gain a hashlock plate? The sidecar renders
  public plates only and `me` *"never passes secret material"* to it
  (`crates/me-cli/src/preview.rs:2-5`), so the answer is probably no — but the
  spec should say so rather than leave it to be discovered.
