# T5 Implementation Plan — guided bundle sequencing (multi-card md1/mk1 → confirm → engrave)

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development or executing-plans. `- [ ]` checkboxes; strict TDD (fail → run-fail → impl → run-pass → commit per task).

**Goal:** A new `engraveBundle` device program that NFC-gathers a bundle of PUBLIC md1/mk1 strings (multiple distinct cards), auto-verifies each card's integrity, then guides a verbatim multi-plate engrave across all cards — the device analogue of host `me bundle`.

**Architecture:** A csid-keyed `bundleGatherer` wrapping the shipped per-card gatherers (`mk1Gatherer`/`md1Gatherer`) UNCHANGED — but with the inversion that a NEW chunk_set_id starts a NEW card (vs single-card foreign-rejection). Single-string mk1 refused (host parity); single-string md1 a standalone card. Three flows: gather → review/confirm → guided verbatim engrave. Reuses `mk.Decode`/`md.DecodeChunks` (integrity) and the `multiPlateEngrave` mechanism unchanged for `deriveXpubFlow`.

**Tech stack:** Go (host tests via `/home/bcg/.local/go/bin/go`; TinyGo-safe). Module `seedhammer.com`.

**Spec:** `design/SPEC_seedhammer_T5_bundle_sequencing.md` (GREEN @ R1, `1a23f9d`). **Recon:** `design/agent-reports/seedhammer-T5-recon-refresh-postT10.md` + `design/cycle-prep-recon-T5-bundle-sequencing.md`. **Spec R0+R1:** `design/agent-reports/seedhammer-T5-bundle-sequencing-spec-review-R{0,1}.md`.
**Reuse (cite):** `mk1Gatherer`/`mk1GatherFlow` (`gui/mk1_inspect.go:48-83,156`), `md1Gatherer`/`md1GatherFlow` (`gui/md1_gather.go:23-63,72`), `mk.Decode` (`mk/mk.go:148-224`), `md.DecodeChunks` (`md/expand.go:25`), `mk.ParseHeader` (`mk/mk.go`), `md.ParseChunkHeader` (`md/chunk.go:185`), `multiPlateEngrave`/`abortWarning` (`gui/derive_xpub.go:263-302`), `validateMdmk` (`gui/gui.go:1897`), the scanner shell (`gui/mk1_inspect.go:163-208`), `hasMKPrefix`/`hasMDPrefix` (`gui/mk1_inspect.go:19`/`gui/md1_inspect.go:15`). **Program lockstep:** the 8 sites + nav-test per the spec §3.

## Locked decisions (from spec, R0-gated)
- Bundle-completeness = **Option A** (operator-driven; faithful to host `me bundle`). Single mk1 → REFUSE; single md1 → standalone 1-plate card; only chunked cards csid-keyed; ms1 → refuse (explicit `codex32.String` case). `engraveBundle` inserted BEFORE `qaProgram`. md1/mk1 engraved VERBATIM. `deriveXpubFlow` call site byte-unchanged.

---

## Task 0: Worktree + baseline + test fixtures (M-1)
- [ ] **Step 1:** `git worktree add ../seedhammer-wt-t5 -b feat/t5-bundle bb0e506` from the fork (sibling-dir; sandbox-fallback `git checkout -b` in place + say so).
- [ ] **Step 2:** Baseline `/home/bcg/.local/go/bin/go test ./gui/... ./mk/... ./md/... ./codex32/...` → all pass; else BLOCKED.
- [ ] **Step 3 (M-1 — fixtures):** In a test helper (`gui/bundle_testdata_test.go`), GENERATE complete, distinct-csid chunk sets at test time rather than relying on non-existent vendored constants: a chunked **mk1** set via `mk.Encode(card)` for two DIFFERENT cards (distinct xpubs → distinct csids), and a chunked **md1** set via the `md` package's chunked-encode test path (the `#10a` tests build chunked md1 by `split`-ing a descriptor — reuse that helper, or vendor the `wshSortedmultiChunks` constant already embedded in `gui/md1_gather_test.go` plus a second generated md1 set). Confirm each generated set: every chunk `ValidMK`/`ValidMD`, and `mk.Decode`/`md.DecodeChunks` round-trips. (M-2 note: the real `multiPlateEngrave` call site is `gui/derive_xpub.go:162`; `:269/:300-301` are interior copy strings.)
- [ ] **Step 4: Commit** (signed+DCO, author Brian Goss, Co-Authored-By trailer; explicit paths) — `gui: bundle test fixtures (generated distinct-csid chunk sets) (T5)`.

---

## Task 1: `classify` + `bundleGatherer` (the data model)

**Files:** Create `gui/bundle.go`; Test `gui/bundle_test.go`.

Types:
```go
type bundleCardKind int        // cardMK1, cardMD1
type bundleCard struct {
    kind    bundleCardKind
    label   string             // "mk1 key" / "md1 descriptor"
    strings []string           // verbatim chunk strings in index order (or [single])
    summary string             // from mk.Card / md.Template, for review
}
type scanClass int             // clsMs1Refuse, clsSingleMK1Refuse, clsStandaloneMD1, clsChunkedMK1, clsChunkedMD1, clsDrop
```
`classify(obj scan.Object) (scanClass, csid uint32, str string)` (R0-C1/C2):
- `codex32.String` (or any HRP-`ms`) → `clsMs1Refuse` (R0-C2 — ms1 arrives as `codex32.String`, not `mdmkText`).
- `mdmkText` with `hasMKPrefix` → `mk.ParseHeader(str)`: `!Chunked` → `clsSingleMK1Refuse` (R0-C1, host parity); else `clsChunkedMK1` + `ChunkSetID`.
- `mdmkText` with `hasMDPrefix` → `md.ParseChunkHeader(str)`: `!Chunked` → `clsStandaloneMD1`; else `clsChunkedMD1` + `ChunkSetID`.
- else → `clsDrop`.

`bundleGatherer` keyed by csid for CHUNKED cards; standalone md1 cards appended directly:
```go
type bundleGatherer struct {
    mkSets map[uint32]*mk1Gatherer   // reuse UNCHANGED
    mdSets map[uint32]*md1Gatherer   // reuse UNCHANGED
    cards  []bundleCard              // completed + verified
}
```
`offer(obj) bundleOfferStatus` (statuses: `bundleRefusedMs1, bundleRefusedSingleMK1, bundleAddedSingleMD1, bundleChunkProgress, bundleCardComplete, bundleDuplicate, bundleDropped`):
- `clsMs1Refuse`/`clsSingleMK1Refuse`/`clsDrop` → the matching refused/dropped status (no state change).
- `clsStandaloneMD1` → `md.Decode(str)` (BCH+structural = integrity, R0-C1); on success append a `cardMD1{strings:[str]}` (dedup by string), status `bundleAddedSingleMD1`; on error → dropped/error.
- `clsChunkedMK1` → `g.mkSets[csid]` (create if new = NEW CARD, R0-I2); `sub.offer(str)`; on `sub.complete()` → `mk.Decode(sub.collected())` → on success append `cardMK1`, status `bundleCardComplete`; else keep progressing.
- `clsChunkedMD1` → same via `g.mdSets[csid]` + `md.DecodeChunks`.

- [ ] **Step 1: Failing tests** (`gui/bundle_test.go`): classify each input class correctly; `clsMs1Refuse` for a `codex32.String` ms1 (NOT dropped); `clsSingleMK1Refuse` for a single mk1; `clsStandaloneMD1` for a single md1; a chunked mk1 set (Task-0 fixture) offered chunk-by-chunk → `bundleCardComplete` after the last, `cards` has 1 verified `cardMK1`; a SECOND distinct-csid mk1 set → a 2nd card (R0-I2, new-csid=new-card, NOT foreign); a dropped chunk → card never completes (not in `cards`); a tampered chunk → `mk.Decode`/`md.DecodeChunks` fails → not added (I-1); duplicate complete card → `bundleDuplicate`, no double-add; two distinct single md1 → 2 cards (no csid-0 collision, R0-C1).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `gui/bundle.go` per above. Reuse `mk1Gatherer`/`md1Gatherer` UNCHANGED (do NOT touch their single-card foreign semantics — I-7).
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `gui: classify + bundleGatherer (csid-keyed, new-csid=new-card) (T5)`.

---

## Task 2: `engraveBundle` program (8-site lockstep, before `qaProgram`)

**Files:** Modify `gui/gui.go`; Test `gui/gui_test.go` (or `gui/bundle_program_test.go`).

- [ ] **Step 1: Failing test** (mirror `gui/derive_xpub_program_test.go`): a nav test that reaches `engraveBundle`, asserts it is navigable (wrap-correct), and asserts a NON-BLANK title (R0-I-B). Update the navigable-bound the existing nav test hard-codes (`:30-31`).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** — insert `engraveBundle` in the `program` enum **BEFORE `qaProgram`** (R0-I-A), and update ALL 8 sites: enum (`gui/gui.go:147-151`); dispatch switch (`:1489-1502`) → call `bundleFlow`; left-wrap (`:1628-1631`) + right-wrap (`:1636-1639`) bounds off `engraveBundle`; **title switch (`:1655-1660`) — ADD an arm (no blank fail-open, R0-I-B)**; `npage` const (`:1834`) = `int(engraveBundle)+1`; **`layoutMainPlates` switch (`:1842-1850`) — ADD a case (else panic)**; `npages` const (`:1852-1853`). Add the StartScreen plate/label for the new program.
- [ ] **Step 4: Run → PASS** + `TestAllocs` green (re-run after the enum change, R0-I-A).
- [ ] **Step 5: Commit** — `gui: engraveBundle program + 8-site lockstep (before qaProgram) (T5)`.

---

## Task 3: `bundleFlow` Phase 1 (gather) + Phase 2 (review/confirm)

**Files:** Create `gui/bundle_flow.go`; Test `gui/bundle_flow_test.go`.

- [ ] **Step 1: Failing tests** (drive via `runUI`+`click`/`runes`; `NFCReader()==nil` → exercise the gatherer-completion + flow directly): the gather screen shows a running tally; offering the Task-0 fixtures (interleaved) accumulates 2 verified cards; an ms1 `codex32.String` → on-screen refusal message ("Type the ms1 share on-device — never over NFC"), NOT silently dropped (R0-C2); a single mk1 → refusal ("scan all its chunks"); **"Done adding" with 0 complete cards → warn/no-op** (R0 acceptance #8); "Done" while a card is mid-chunk-set → warn that card incomplete + drop it; Phase 2 review lists the cards (type + summary, each verified) and Confirm advances.
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `bundleFlow(ctx, th)`: the scanner-shell loop (clone `mk1GatherFlow`'s goroutine guard `gui/mk1_inspect.go:163-208`) driving `bundleGatherer.offer`; per-status feedback strings; a "Done adding cards" affordance (Button) → if any in-progress incomplete card, warn+drop; if 0 cards, warn no-op; else → `bundleReviewFlow` (list cards, Confirm/Back) → returns the confirmed `[]bundleCard` to Phase 3.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `gui: bundleFlow gather + review/confirm (Phase 1+2) (T5)`.

---

## Task 4: Phase 3 — guided verbatim engrave (`deriveXpubFlow` byte-unchanged, R0-I-C)

**Files:** Modify `gui/bundle_flow.go` + `gui/derive_xpub.go` (parameterize); Test `gui/bundle_flow_test.go`.

- [ ] **Step 1: Failing tests:** confirming a 2-card bundle drives "Card 1 of 2 · Plate P of Q" then "Card 2 of 2 · …" through `validateMdmk`, engraving each card's strings VERBATIM (assert the engraved strings equal the gathered strings, I-4); a single-md1 card → 1 plate; set-level abort mid-bundle → partial-bundle warning, no completed state (I-5); an ms1 reminder prompt appears at the end. **Assert `deriveXpubFlow`'s existing behavior is unchanged** (its test still passes verbatim).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** — generalize `multiPlateEngrave`/`abortWarning` (`gui/derive_xpub.go:263-302`) by ADDING optional params (card label + "Card X of Y" context) with defaults that leave the `deriveXpubFlow` call site (`gui/derive_xpub.go:162`) BYTE-UNCHANGED (R0-I-C, M-2) — or add a thin `bundleEngrave(ctx,th,cards)` sequencer alongside it that reuses the same per-plate `validateMdmk` machinery. Loop cards → per-card plates "Card X of Y · Plate P of Q"; set-level abort (extend `abortWarning`) records no completed state; append the ms1 reminder (mirror host `bundle.rs:296-306`).
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `gui: bundle guided verbatim engrave + set-abort + ms1 reminder (Phase 3) (T5)`.

---

## Task 5: No-regression + fuzz

**Files:** Test only.

- [ ] **Step 1:** `/home/bcg/.local/go/bin/go test -count=1 ./...` + `TestAllocs` green; single-card mk1/md1/ms1 flows + `deriveXpubFlow` + `backupWalletFlow` unchanged (their tests pass verbatim, I-7); `go vet ./gui/...` clean (vs baseline); `gofmt -l` empty.
- [ ] **Step 2: Fuzz** `FuzzBundleGatherer` (arbitrary sequences of {ms1, single-mk1, single-md1, chunked-mk1/md1 chunks, garbage} → no panic; only verified-complete cards ever land in `cards`; ms1 never added). ≥1M execs.
- [ ] **Step 3: Run → 0 panics.**
- [ ] **Step 4: Commit** — `gui: no-regression + fuzz for the bundle gatherer (T5)`.

---

## Acceptance (GREEN bar for the exec review)
The spec §5's 10 items: multi-card gather; new-csid=new-card; per-card integrity (partial/tampered never added); ms1 refused-not-dropped; single-string mk1 refused / md1 standalone (no collision); guided "Card X of Y · Plate P of Q" verbatim engrave (md1+mk1); set-level abort no-state; empty/half-scanned handling; program nav (before qaProgram, non-blank title, no panic, TestAllocs re-run); no-regression + fuzz 0 panics. `deriveXpubFlow` byte-unchanged.

## Self-review (author, pre-R0)
- Spec coverage: Phase 1 gather → T1+T3; Phase 2 → T3; Phase 3 → T4; program → T2; no-regression → T5. Invariants I-1..I-8 + R0 folds (C-1 single-string, C-2 ms1-classify, I-A enum-before-qa, I-B title arm, I-C param) all mapped. ✓
- Minors: M-1 (generate distinct-csid fixtures, no v3c0) → T0.3; M-2 (real call site :162) → T0.3/T4. ✓
- No placeholders; types consistent (`bundleCard`/`bundleGatherer`/`scanClass`); each step cites the reused fn + the spec invariant. ✓

## Gate
This plan MUST pass opus R0 to 0C/0I before code; fold → persist verbatim → re-dispatch after every fold until GREEN. Then single-implementer TDD in the worktree → mandatory whole-diff adversarial exec review → merge no-ff (signed+DCO) → push bg002h.
