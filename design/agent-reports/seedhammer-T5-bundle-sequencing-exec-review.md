# T5 — guided bundle sequencing: whole-diff adversarial EXECUTION REVIEW

**Reviewer:** opus architect (mandatory non-deferrable post-implementation exec review)
**Date:** 2026-06-19
**Branch under review:** `feat/t5-bundle` @ worktree `/scratch/code/shibboleth/seedhammer-wt-t5`, cumulative diff `bb0e506..HEAD` (6 commits).
**Production code:** `gui/gui.go` (+16/-7), new `gui/bundle.go` (312 LoC), new `gui/bundle_flow.go` (376 LoC). `gui/derive_xpub.go` byte-unchanged.
**Authoritative sources cross-checked:** Go fork @ `bb0e506` + diff; host `me bundle` (`crates/me-cli/src/bundle.rs`); the `mk`/`md`/`codex32` codecs; `gui/scan.go`, `gui/mk1_inspect.go`, `gui/md1_gather.go`.

---

## VERDICT: GREEN (0 Critical / 0 Important)

The feature is faithful to host `me bundle` (Option A, operator-driven, per-chunk-set integrity), and the high-stakes safety properties hold under adversarial probing. No partial, tampered, wrong, or collided card can reach the engrave plan; no secret is ever gathered or engraved; the program lockstep has no reachable panic or blank title; `deriveXpubFlow` is byte-unchanged. Findings below are 2 Minor (non-blocking).

---

## Observed test / vet / fuzz output

**Full suite** — `/home/bcg/.local/go/bin/go test -count=1 ./...`: ALL PASS (gui 6.005s, md, mk, codex32, engrave, all packages `ok`; no `FAIL`).

**TestAllocs** — `go test -count=1 -run TestAllocs ./gui/...`: `ok seedhammer.com/gui 1.102s` (+ op/saver/text/widget all ok). GREEN after the enum change.

**go vet ./gui/...**: single line —
`gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later (file is go1.25)`.
Confirmed **byte-identical at baseline `bb0e506`** (ran `go vet ./gui/...` in a clean `git worktree add /tmp/sh-baseline bb0e506`: same file:line, same message). **Pre-existing, NOT a regression.**

**gofmt -l .** (whole tree): empty (no formatting drift).

**FuzzBundleGatherer** — `go test -run '^$' -fuzz '^FuzzBundleGatherer$' -fuzztime 1500000x ./gui/`:
`elapsed: 40s, execs: 1500000 ... new interesting: 0 (total: 167) ... PASS`. **1,500,000 execs, 0 panics, 0 crashers.** The committed corpus entry `gui/testdata/fuzz/FuzzBundleGatherer/9c1ffb19ac30159a` = `[]byte(",")` is benign (op byte indexes the pool mod len).

**Bundle/Classify/Program tests** (verbose): 26 tests + fuzz seeds all PASS (TestBundlePlanVerbatim, TestBundleGatherTamperedChunkNeverAdded, TestBundleGatherSecondCardNewCsid, TestBundleGatherMs1Refused, TestBundleGatherSingleMK1Refused, TestEngraveBundleProgramNavigable, TestEngraveXpubProgramNavigable, TestBundleEngraveSetAbort, …).

**Host `me bundle`** — `cargo test bundle` in `crates/me-cli`: `5 passed; 0 failed` (bundle.rs suite), confirming the mirrored model is itself green.

**Independent adversarial probes** (scratch module `/tmp/t5probe` with a `replace seedhammer.com => worktree`; reviewer-only, since removed — feature source untouched):
- `TestCsidCollisionMixedSetRejected`: two genuinely-distinct 60-byte bytecodes stamped with the **same** header csid (`0x12345`), each a valid 2-chunk set; a MIXED reassembly `[cardA[0], cardB[1]]` (and the reverse) → `mk.Decode` returns `mk: cross-chunk integrity hash mismatch`. PASS.
- `TestSameCsidDuplicateIndexRejected`: same-csid duplicate-index set → `mk: duplicate chunk index`. PASS.
- `TestVectorsAreSingleStringMD1`: `wpkh_basic` + `tr_keyonly` are genuine non-chunked single-string md1 (Chunked=false, `md.Decode` OK) → route to `clsStandaloneMD1`. PASS.
- `TestPublicCardsNotCodex32`: real mk1 chunks + a single md1 NEVER satisfy `codex32.New` (so are never misrouted to ms1-refuse) and are `ValidMK`/`ValidMD`. PASS.

---

## Required explicit attestations

### (a) No partial / tampered / wrong card can be engraved. CONFIRMED.
A card lands in `g.cards` ONLY after its integrity gate passes:
- **chunked mk1** — `offerChunkedMK1` adds only on `sub.complete()` THEN `mk.Decode(collected)` success (`gui/bundle.go:188-207`). `mk.Decode` → `reassemble` enforces count, csid-equality, no duplicate index, and the SHA-256[0:4] cross-chunk hash (`mk/mk.go:183-223`). On error it `delete`s the sub and returns `bundleDropped` — never added (`gui/bundle.go:195-199`).
- **chunked md1** — `offerChunkedMD1` adds only on `sub.complete()` THEN `md.DecodeChunks(collected)` success (`gui/bundle.go:228-245`). `DecodeChunks`→`Reassemble` enforces version/csid/count consistency, completeness, contiguous indices, and re-derives the csid from the decoded descriptor comparing to the header csid (`md/chunk.go:244-291`).
- **standalone md1** — `offerStandaloneMD1` adds only on `md.Decode(str)` success = BCH + full structural decode (`gui/bundle.go:151-160`; `md/md.go:1216-1230`).
- **dropped chunk** → never `complete()` → never added (`TestBundleGatherDroppedChunkNeverCompletes`).
- **Phase 3 engraves the gathered strings VERBATIM**: `bundlePlatePlan` flattens `c.strings` directly (`gui/bundle_flow.go:303-318`); `validateMdmk(params, p.str)` QR/text-encodes the exact string `p.str` (`gui/gui.go` validateMdmk) — engraved == gathered, no mutation (I-4; `TestBundlePlanVerbatim`).

### (b) The csid-collision edge cannot silently produce a corrupt complete card. CONFIRMED (empirically).
Two distinct cards sharing a header csid route to one sub-gatherer and could interleave (the `mk1Gatherer`/`md1Gatherer` foreign guard only rejects a DIFFERENT csid/total, `gui/mk1_inspect.go:65`, `gui/md1_gather.go:45`). The per-card integrity gate is the backstop: the reviewer probe forced exactly this collision and `mk.Decode` rejected every mixed reassembly with `cross-chunk integrity hash mismatch` (a SHA-256 preimage would be required to forge a consistent franken-bytecode). Same for md1 via the re-derived-csid gate (`md/chunk.go:289-290`). A same-csid-different-total chunk yields `gatherForeign` → `default: bundleDropped` (`gui/bundle.go:210-211,248-249`) — no state corruption. No corrupt "complete" card is producible.

### (c) No secret is ever gathered / engraved. CONFIRMED.
The scanner yields `codex32.String` ONLY for a valid codex32 (the ms1 secret class; `gui/scan.go:70`); `classify` maps **every** `codex32.String` → `clsMs1Refuse` regardless of HRP (`gui/bundle.go:64-70`), surfaced as "Type the ms1 share on-device — never over NFC" (`gui/bundle_flow.go:55-56`) — explicit, rendered, never silently dropped (`TestBundleGatherMs1Refused`, `TestBundleGatherFeedback`). Every other scan object — a `bip39.Mnemonic` SEED (`gui/scan.go:61`), an output descriptor, an `addressText`, a `debugCommand` — is a distinct type that falls to `classify`'s `default → clsDrop → bundleDropped` (`gui/bundle.go:95-96,139-140`); none is gathered or engraved. A single (non-chunked) mk1 is `clsSingleMK1Refuse` (host parity `bundle.rs:128`). The end-of-bundle ms1 reminder renders (`gui/bundle_flow.go:360,374-375`). No secret-bearing path into `cards` exists.

### (d) `deriveXpubFlow` is byte-unchanged. CONFIRMED.
`git diff bb0e506..HEAD -- gui/derive_xpub.go` is EMPTY. `multiPlateEngrave` / `abortWarning` (in that file) are therefore untouched; `bundleEngrave` is a clean SIBLING reusing the same lower-level `validateMdmk` / `ChoiceScreen.Choose` / `NewEngraveScreen.Engrave` primitives (`gui/bundle_flow.go:327-361`). The only non-test production touch besides the new files is `gui/gui.go` (the 8-site lockstep). The intentional structural duplication is acceptable (R0-M2 rationale: Go has no default params; a sibling avoids mutating the call site `deriveXpubFlow` depends on) — divergence risk is low because the load-bearing per-plate machinery is shared, not copied.

### (e) The program lockstep has no reachable panic / blank title. CONFIRMED.
`engraveBundle` inserted in the enum BEFORE `qaProgram` (`gui/gui.go:147-152`). All 8 sites coherent: dispatch arm → `bundleFlow` (`:1497-1499`); left-wrap `m.prog = engraveBundle` (`:1634`); right-wrap `if m.prog > engraveBundle` (`:1641`); title arm present `"Engrave Bundle"` (`:1664-1665`); `npage = int(engraveBundle)+1` (`:1840`); `layoutMainPlates` case `backupWallet, engraveXpub, engraveBundle` (`:1850`); `npages = int(engraveBundle)+1` (`:1859`); nav-test bound updated + lands on `engraveBundle` asserting non-blank "Bundle" (`gui/bundle_program_test.go`, `gui/derive_xpub_program_test.go` diff). `qaProgram` is reachable ONLY via the `FOREVERLAURA!` debug command which returns `startScreenAction{prog: qaProgram}` straight to the dispatch switch (`:1602`, `:1491`); it is NEVER reached by `m.prog` navigation (which wraps at `engraveBundle`), so `m.prog ∈ {backupWallet, engraveXpub, engraveBundle}` always — every value has a title arm AND a `layoutMainPlates` case. No blank title, no reachable `panic("invalid page")`. `TestAllocs` green after the enum change.

---

## Findings

### Minor

**M-1 (cosmetic / divergence-hardening) — no compile-time guard couples the lockstep to the enum.**
`npage`/`npages`/the wrap bounds/`layoutMainPlates` are all hand-keyed off `engraveBundle`. They are correct today and TDD-covered (`TestEngraveBundleProgramNavigable`), but a future program insertion repeats the exact T4 failure surface the spec warns about (risk #3). This is pre-existing structure, not introduced by T5. Optional hardening: a `const _ = uint(qaProgram - (engraveBundle + 1))` style static assertion (or a `//go:` comment) documenting that `engraveBundle` must remain the last navigable program. No action required for merge.

**M-2 (defense-in-depth note) — `bundleEngrave`'s "verified card won't fit a plate" branch aborts the whole set.**
If a card that already passed its integrity gate produces `len(plates)==0` from `validateMdmk`, `bundleEngrave` aborts the entire bundle via `bundleAbortWarning` (`gui/bundle_flow.go:331-337`). This is the safe direction (never engrave a partial), and `TestBundlePlanValidatesEachPlate` shows the real fixtures always fit, so the branch is unreachable in practice. It is correct as written; noting only that the failure mode is "abort all" rather than "skip one" — which is the right choice for a wallet backup (a bundle missing a card is unusable anyway). No action required.

---

## Faithfulness to host `me bundle` (Option A) — CONFIRMED
- per-chunk-set completeness/consistency/integrity, operator-driven, no invented expected-card-count (`bundle.rs:177-323` ↔ `bundleGatherer` + `bundleDoneDecision`).
- ms1 refused (`bundle.rs:97-99,188-192` ↔ `clsMs1Refuse`) + trailing ms1 reminder (`bundle.rs:296-306` ↔ `bundleMs1ReminderText`).
- single md1 = standalone bch-only plate (`bundle.rs:150,229-240` ↔ `clsStandaloneMD1`/`md.Decode`); single mk1 = unsupported/refused (`bundle.rs:128` ↔ `clsSingleMK1Refuse`).
- chunked md1 set is gated by reassembly (`bundle.rs:243-247` ↔ `md.DecodeChunks`) even when the set is a single chunk (`wsh_multi_chunked`, csid 0x157ae) — routed by the chunked-flag bit, not the string count (`md/chunk.go:193`), and still integrity-checked (confirmed via `md1CardB` fixture + `FuzzBundleGatherer`'s dual md.Decode/md.DecodeChunks assertion).
- md1/mk1 engraved VERBATIM, no re-encode (`bundle.rs:255-292` ↔ `bundlePlatePlan` + `validateMdmk`).

## Set-level abort + done-handling — CONFIRMED
Mid-bundle abort → `bundleAbortWarning`, dismiss-only, records no completed state (`gui/bundle_flow.go:344-350,365-370`; `TestBundleEngraveSetAbort`). "Done" with 0 cards → `bundleDoneEmpty` warn no-op (`:153-154`). "Done" with a half-scanned card → `bundleDonePending` precedes the cards check, warns + `dropPending()` (drops only primed-incomplete subs, completed cards untouched), then proceeds with the complete cards (`:151-166,211-218`, `bundle.go:286-297`; `TestBundleDoneDecision` incl. complete+pending). Phase-2 "Back from review" starts a fresh accumulator on the next `bundleGatherFlow` (`bundle_flow.go:30-35,96`) — no stranding, no double-engrave (engrave only runs after Confirm returns true, then `return`s).

## Fixtures authenticity — CONFIRMED
mk1 fixtures via `mk.Encode` with distinct Fingerprints → distinct deterministic csids, each round-trips `mk.Decode` and is `ValidMK` (`bundle_testdata_test.go:87-105`; `TestBundleFixturesDistinct`). md1 fixtures are the in-tree reachable real vectors `wshSortedmultiChunks` (0x2d950) and `wsh_multi_chunked` (0x157ae), each `md.DecodeChunks`-verified (`:107-129`).

---

**Bottom line:** GREEN. Clear to proceed to merge (no-ff, signed+DCO, author Brian Goss) per the gate. The 2 Minors are optional hardening, not blockers. Reviewer made no changes to the feature branch's tracked source.
