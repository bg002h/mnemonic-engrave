# R0 GATE REVIEW — SPEC_seedhammer_T5_bundle_sequencing.md

**Reviewer:** opus architect (R0 pre-implementation gate). **Date:** 2026-06-19.
**Spec under review:** `design/SPEC_seedhammer_T5_bundle_sequencing.md`
**Authoritative sources verified:** GO fork `/scratch/code/shibboleth/seedhammer` @ `bb0e506` (confirmed HEAD); HOST `crates/me-cli/src/bundle.rs`. All citations below were read directly, not trusted from the spec prose. Baseline `go test ./gui/ -run 'TestAllocs|TestEngraveXpubProgramNavigable'` = GREEN at `bb0e506`.

---

## VERDICT: NOT GREEN

2 Critical, 3 Important, 5 Minor. The Option-A faithfulness claim is **upheld**; the new-csid-=-new-card *chunked* path is **sound**; but the routing model has an **unhandled single-string / csid=0 collision class** that breaches the spec's own I-1 (no partial/un-integrity-checked card engraved) and diverges from the host it claims to mirror, and the ms1-refusal mechanism is mis-stated against the actual scan classifier. These must be folded before code.

---

## RULINGS ON THE THREE QUESTIONS THE PROMPT REQUIRED

### A. Option-A faithfulness claim (spec §1, §4, I-8) — **UPHELD (correct).**
`bundle.rs` genuinely has NO expected-card-count and NO md1↔mk1 cross-match:
- Grouping is purely by `chunk_set_id` into independent `BTreeMap`s, one per format (`bundle.rs:200-223`); each group is proven complete/consistent/integral on its own (`md_codec::chunk::reassemble` `:246`; `mk_codec::decode` `:273`). No code anywhere counts "cards a wallet needs" or matches an md1 key to an mk1 card.
- ms1 refused up front by a classify-only pre-scan over all lines before any validation (`:188-192`), exit 3.
- ms1 reminder appended as a trailing plate unconditionally (`:296-306`, `ms1_required: true` `:319`).
- md1 carried VERBATIM into each plate (`:255-266`, `string: Some(s.clone())`); chunked md1 reassembled only to PROVE integrity, never re-encoded for output.

The spec's characterization of the host (§1 lines 8-10, §4 lines 35-36, I-8) is accurate. **No finding here.** (One nuance the spec under-states feeds C-1 below: the host additionally REFUSES single-string mk1 — `BundleError::Mk1SingleString`, `bundle.rs:56-58,128` — and treats unchunked md1 as a standalone bch-only plate, `:228-240`. The spec's "faithful mirror" claim is therefore incomplete on the single-string axis.)

### B. New-csid = new-card soundness, incl. the csid-collision edge (spec I-2, risk #1) — **SOUND for chunked cards; UNSOUND for single-string / csid=0 (see C-1).**
For genuinely *chunked* cards the wrap-in-a-csid-keyed-map design is correct and needs no edit to the single-card foreign semantics:
- `mk1Gatherer.offer` / `md1Gatherer.offer` key on the parsed header and self-validate consistency (`mk1_inspect.go:55-73`, `md1_gather.go:30-53`); each sub-gatherer only ever sees its own csid's chunks, so the `gatherForeign` arm (`mk1_inspect.go:65-67`, `md1_gather.go:45-47`) is never hit inside a correctly-routed sub-gatherer. The `offer`/`complete`/`collected` API (`mk1_inspect.go:55-83`, `md1_gather.go:30-63`) fully supports per-csid driving. I-7 (no edit to single-card foreign semantics) is preserved.
- A chunk "valid for no existing card" simply primes a NEW sub-gatherer — correct, this IS the inversion.
- Two distinct *chunked* cards with the SAME 20-bit csid: collide into one sub-gatherer, where the per-card integrity gate catches them — mk1 cross-chunk `SHA-256[0:4]` over the reassembled stream (`mk/mk.go:214-222`) and md1 csid re-derivation (`md/chunk.go:284-291`, `ErrChunkSetIDMismatch`). A foreign chunk re-stamped with a colliding csid produces an index collision (`gatherDup`) or an integrity failure. This matches the host's behavior exactly. **Acceptable.**

The UNSOUND edge is **single-string** mk1/md1, whose header carries NO csid (zero value) → C-1.

### C. Invariants/acceptance complete enough to prevent a partial/tampered card being engraved? — **NO, not yet** (C-1 single-string hole + I-1 wording). For *chunked* cards: YES — `mk.Decode` (`mk/mk.go:148-224`) and `md.DecodeChunks`/`Reassemble` (`md/chunk.go:207-293`) both perform full reassembly + completeness + consistency + the cross-chunk integrity gate, and the gatherer only reports `complete()` at `len(set)==total` (`mk1_inspect.go:75`, `md1_gather.go:55`), so a partial chunked card cannot complete and a tampered one fails Decode. The hole is single-string.

---

## CRITICAL

### C-1. Single-string (non-chunked) mk1/md1 in a bundle context is undefined, and the obvious csid-keyed routing collides them at csid=0 and engraves a single mk1 with NO integrity gate — breaches I-1 and diverges from the host. (spec §2 IN line 16, I-1, I-2, §5 acceptance; vs `bundle.rs:128`, `mk/mk.go:177-181`, `mk1_inspect.go:62-64`, `md1_gather.go:38-40`)

The spec defines the bundle as keyed by `chunk_set_id` and says "a NEW csid starts a NEW card." But the scanner accepts **single (non-chunked)** mk1 and md1 strings as `mdmkText` (`scan.go:72`, gated only by `codex32.ValidMK`/`ValidMD`, which accept single short/long mk1 — `codex32/mdmk.go:124-149`). A single string has NO csid on the wire:
- single mk1: `parseHeaderSyms` returns `Header{Chunked:false, TotalChunks:1, ChunkIndex:0}` with **ChunkSetID = 0** (never set; `mk/mk.go:74-76`).
- single md1: `ParseChunkHeader` returns `ChunkHeader{Chunked:false}` with **ChunkSetID = 0** (the discriminator short-circuits before reading any csid; `md/chunk.go:193-196`).

Consequences the spec does not address:
1. **csid=0 collision** — two distinct single cards (or a single + a chunked card that happens to derive csid 0) both key to bucket 0, silently merging distinct cards.
2. **Integrity bypass for single mk1** — if a fresh `mk1Gatherer` is primed with a single mk1, it sets `total=1`, `complete()` is immediately true (`mk1_inspect.go:75`), and `mk.Decode([single])` reassembles via the `!first.Chunked` single-fragment path **with NO cross-chunk hash** (`mk/mk.go:177-181`, comment "single-string fragment IS the bytecode (no hash)"). So a single mk1 would be "verified" and engraved with only its own BCH as integrity — exactly the case the **host explicitly REFUSES** (`BundleError::Mk1SingleString`, `bundle.rs:56-58,128`). This breaks the "faithful mirror" claim (I-8) and the I-1 spirit (a card only counts when genuinely whole/integral).
3. **single md1 today returns `gatherIgnored` when offered to a fresh `md1Gatherer`** (`md1_gather.go:38-40`), so a single md1 routed into a sub-gatherer would be silently dropped — yet the host engraves it as a standalone bch-only plate (`bundle.rs:228-240`). Either behavior may be defensible, but the spec must CHOOSE and state it.

**Fix (pick one and write it into §2/§4/the invariants + an acceptance test):** Mirror the host. (a) **Refuse single-string mk1 in the bundle** with a clear message (e.g. "synthetic/short key card — re-issue as a chunked card"), matching `bundle.rs:128`; do NOT route it into a csid bucket. (b) Decide single md1: either treat it as its own standalone bch-only plate (host parity, `bundle.rs:228-240`) keyed not by csid but as a distinct singleton, OR refuse it — but state which, and DO NOT bucket it under csid=0. (c) State the routing key precisely: bundle membership keys on `(format, chunk_set_id)` for *chunked* cards only; single-string cards take a separate, non-csid-keyed path. Add an acceptance case for "two single cards do not merge" and "single mk1 is refused / single md1 handled per host." Without this, the implementer's natural `map[uint32]*subGatherer` keyed on header csid is a latent merge-and-bypass bug.

### C-2. ms1 refusal as specified will NOT fire on the actual scan path — an ms1 scan is classified as a `codex32.String`, not `mdmkText`, so it is silently dropped, not refused. (spec §2 IN line 16, §4 line 34, I-3, acceptance #4; vs `scan.go:70-73`, `gui.go:1881-1882`, `validateMStar` `codex32_polish.go:259-266`)

The spec says ms1 (HRP `ms`) is "refused in this channel" with a hand-type message. But on the NFC scan path the classifier tries `codex32.New(buf)` BEFORE the md/mk check (`scan.go:70-73`): a valid ms1 string parses as a `codex32.String` and is returned as such — it is **never** an `mdmkText`. The existing gather loops type-assert `mdmkText` (`mk1_inspect.go:218`, `md1_gather.go:135`); a `codex32.String` simply fails the assertion and is dropped with no message. So a bundle gather loop that copies that pattern would IGNORE ms1, not REFUSE it — failing acceptance #4 ("refused with the 'type on-device' message") and weakening the I-3 secret-spine UX (silent drop vs explicit refusal).

(The hand-entry classifier `validateMStar` `codex32_polish.go:259-266` does route `ms` to `codex32.New` for the keyboard path, and that is the secret entry path — but that's the keyboard, not the bundle NFC channel; it is not a "refusal.")

**Fix:** State in §4/I-3 that the bundle gather loop must add an explicit `case codex32.String:` (and/or detect HRP `ms`/`MS` on the raw scan) and surface the refusal message + continue, rather than relying on the `mdmkText`-only assertion to drop it. Note that the secret is still never *gathered* (it's dropped either way), so this is a UX-correctness + acceptance-test fidelity fix, not a secret-leak fix — but acceptance #4 as written is currently unsatisfiable against the real classifier. Add the explicit-refusal assertion to the acceptance test. (The end-of-bundle ms1 reminder, §2/§4 line 18, is independently fine — it mirrors `bundle.rs:296-306` and needs no scan-path change.)

---

## IMPORTANT

### I-A. Enum insertion position vs `qaProgram` is unspecified and the "8 sites" are not all simple enumerations — two are derived consts (`int(engraveXpub)+1`) that silently miscount if `engraveBundle` is appended after `qaProgram`. (spec §2 IN line 15, §3 line 30, I-6, risk #3; vs `gui.go:147-151,1834,1853`)

The enum is `backupWallet=0, engraveXpub=1, qaProgram=2` (`gui.go:147-151`). `qaProgram` is debug-only, reached solely via the `FOREVERLAURA!` debug command (`gui.go:1598`), and is deliberately EXCLUDED from the navigable range and from the two page-count consts, which are written as `const npage = int(engraveXpub) + 1` (`:1834`) and `const npages = int(engraveXpub) + 1` (`:1853`). The spec lists these as "sites 6 and 8" but does not state that they are VALUE-DERIVED from `engraveXpub`, nor where `engraveBundle` lands relative to `qaProgram`. If the implementer appends `engraveBundle` AFTER `qaProgram` (`...qaProgram=2, engraveBundle=3`), then `int(engraveXpub)+1 == 2` still excludes it and the new program is unreachable + the pager shows the wrong dot count — a silent lockstep miss that the nav-test may or may not catch depending on how it's rewritten.

**Fix:** Specify the insertion: `backupWallet=0, engraveXpub=1, engraveBundle=2, qaProgram=3` (keep `qaProgram` last so it stays out of the navigable range), and rewrite BOTH consts to `int(engraveBundle)+1` (and the two wrap bounds at `:1630,1637-1638` to wrap at `engraveBundle`, not `engraveXpub`). State explicitly that sites 6 and 8 are derived-from-the-upper-navigable-enumerant, not literal `engraveXpub` references. Without this the "8 sites coherent" invariant (I-6) is under-specified at exactly the spot that panics (T4's failure surface).

### I-B. The `layoutMainPlates` panic guard is the ONLY hard panic; the title switch fails OPEN (empty title), so the nav-test as currently written would NOT catch a missing title case. (spec I-6, acceptance #7; vs `gui.go:1655-1660` no-default vs `:1842-1850` panic)

`layoutMainPlates` panics on a missing case (`:1849`). But the title switch (`:1655-1660`) has NO default — a missing `engraveBundle` arm yields `titleTxt = ""` (blank title), not a panic. The acceptance #7 / nav-test asserts the program is "titled"; that's good, but the spec should state that the title case must be added (fails-open silently otherwise) AND that `layoutMainPlates` is the one that panics. The existing nav-test (`derive_xpub_program_test.go:8-40`) asserts a specific title string is present after navigation — so a blank-title regression WOULD be caught IF the rewritten test navigates onto `engraveBundle` and asserts its title. Make that explicit in acceptance #7 ("assert the engraveBundle title string renders, and that navigating onto it does not panic").

**Fix:** In §2/I-6, enumerate that BOTH the title arm (fails open) and the `layoutMainPlates` arm (panics) must gain an `engraveBundle` case, and require the nav-test to navigate ONTO `engraveBundle` and assert its concrete title — not merely that Right wraps.

### I-C. Generalizing `multiPlateEngrave`/`abortWarning` must preserve `deriveXpubFlow`'s EXACT mk1 copy, but the spec doesn't pin the parameterization contract — risk of regressing the live caller. (spec §2 IN line 18, risk #5, I-7; vs `derive_xpub.go:162,263-293,298-302`)

`multiPlateEngrave` (`derive_xpub.go:263-293`) and `abortWarning` (`:298-302`) are wired ONLY into `deriveXpubFlow` (`:162` — grep-confirmed single caller). The mechanism (`validateMdmk(params, s)` per string, `:267`) is genuinely format-agnostic and safe to reuse. BUT the copy is mk1-specific in two load-bearing spots the spec glosses: the error title `"Account Xpub"` + "This key card doesn't fit a plate." (`:269`) and `abortWarning`'s "This key card set can't be restored…" (`:300-301`). If the implementer parameterizes by mutating these strings without preserving the existing `deriveXpubFlow` call's text, that's an I-7 regression (changed UX on a shipped flow).

**Fix:** State the contract: add a label/title parameter (and the abort copy) to the generalized helper, and require `deriveXpubFlow`'s call site to pass the CURRENT mk1 strings verbatim so its behavior is byte-unchanged. Add to acceptance #8 a `deriveXpubFlow` UX-string-unchanged assertion (or keep a thin mk1 wrapper that supplies the old strings). The spec already lists I-7; this just pins the mechanism so the reviewer at exec-review can check it.

---

## MINOR (non-blocking)

- **M-1 (empty bundle / "done" with zero cards).** §7 risk #2 covers the half-scanned card on "done" but not the EMPTY bundle (operator hits done having scanned nothing). Host returns `BundleError::Empty` exit 2 (`bundle.rs:183-184,38`). State that "done with 0 verified cards" is a no-op / warning, not a zero-plate engrave. Add to acceptance.
- **M-2 (duplicate WHOLE card).** Host's BTreeMap-by-csid naturally dedups, and within a sub-gatherer a repeat chunk is `gatherDup`. But scanning the SAME complete card twice across the bundle (re-presenting an already-completed csid) is undefined — define "already added this card" feedback. Likely a one-liner (check the csid is already in the verified set before re-priming).
- **M-3 (acceptance #1 fixtures are real and available — strengthen, don't invent).** Chunked md1 golden = `wshSortedmultiChunks` (6 chunks, real xpubs → expandOK) at `gui/md1_gather_test.go:14-21`; chunked mk1 golden = `v1c0`/`v1c1` (2-chunk) + `v3c1` (distinct csid) at `gui/mk1_inspect_test.go:12-14`. The spec's "embedded golden chunk sets" is achievable verbatim from these; cite them in the plan so the implementer doesn't try to build md1 chunks (no exported `md.split`/`md.Encode`; recon-refresh line 11 confirms).
- **M-4 (addressText / other scan types in the gather loop).** The scanner can also emit `bip39.Mnemonic`, `*bip380.Descriptor`, `slip39words.Share`, `addressText`, `debugCommand` (`scan.go:58-79`). The bundle loop should ignore non-md/mk objects (like the existing gather loops' `mdmkText`-only assertion) but the spec should note debugCommand/Mnemonic appearing mid-bundle is just ignored (no secret concern — a scanned bip39 mnemonic is the only "secret-ish" one, and it is NOT engraved by the bundle path; confirm it's dropped, not routed).
- **M-5 (alloc gate scope is correctly off-path, confirm in plan).** `TestAllocs`/`BenchmarkAllocs` gate ONLY `StartScreen.Flow` and `DescriptorScreen.Confirm` (`gui_test.go:64-71`); the gather/engrave flows are NOT alloc-gated (same as the existing mk1/md1 gather flows). Adding `engraveBundle` to the navigable range adds one pager dot to the alloc-gated `StartScreen` draw — the existing 2-program loop (`layoutMainPager` `:1852-1873`) already exercises this pattern alloc-free; baseline confirmed GREEN. The spec's claim (I-6, "any bundle build happens off the alloc-gated screens") is correct — just have the plan re-run `TestAllocs` after the enum change as a guard, since the StartScreen IS gated.

---

## SCOPE BOUNDARY (prompt Q9) — CLEAN.
Deferring Option B (md1↔mk1 cross-match), on-device production (T6), and verify-bundle/restore-doc (T6) is clean and well-justified: Option B is conditional on md1 carrying sparse/optional xpubs (`md/expand.go:56-64`, `XpubPresent` gate) and needs net-new normalization (recon-refresh Job 2) — correctly out. T5 sized as one cycle is appropriate: the read paths (gather/decode/integrity for both formats) and the engrave mechanism (`multiPlateEngrave`/`validateMdmk`) all exist; T5 is orchestration + one new program + generalization. No new codec, no encoder. Reasonable.

---

## WHAT TO FOLD BEFORE RE-DISPATCH
1. **C-1:** define single-string mk1/md1 handling (refuse single mk1 per host; decide single md1; route singles off the csid map; forbid csid=0 bucketing); add acceptance + invariant. **Blocking.**
2. **C-2:** specify the explicit ms1 refusal in the bundle loop (`codex32.String`/HRP-ms case), since the `mdmkText`-only assertion silently drops it; fix acceptance #4. **Blocking.**
3. **I-A:** pin enum insertion (`engraveBundle` before `qaProgram`) and rewrite the two derived consts + two wrap bounds off `engraveBundle`.
4. **I-B:** require both the title arm (fails open) and `layoutMainPlates` arm (panics) + a nav-test that lands ON `engraveBundle` and asserts its title.
5. **I-C:** pin the `multiPlateEngrave`/`abortWarning` parameterization contract to keep `deriveXpubFlow`'s copy byte-unchanged; add the I-7 assertion.

Re-dispatch after folding (folds can introduce drift). GREEN only at 0C/0I. Minors may be folded opportunistically but do not block.
