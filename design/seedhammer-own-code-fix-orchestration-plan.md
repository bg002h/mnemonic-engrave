# Orchestration plan — SeedHammer own-code bug-hunt remediation (8 findings)

**Author:** opus architect `a7cec57f5067164e7` (read-only design deliverable). **Base:** fork `main` @ `3a23dbb`. **Findings source:** `design/agent-reports/seedhammer-fork-own-code-bughunt.md` (8 confirmed). **Date:** 2026-06-20. **Directive:** "fix all 8 — multi-agent parallel, disciplined, concurrent where DEFINITELY safe." User approved; executing autonomously.

## Tracks (dedup applied)

| Track | Findings | Files written | Branch (off `3a23dbb`) | Weight |
|---|---|---|---|---|
| **A — verify-correctness** | H1, H2, M1, L2, **L1 (2 of 3 sites)** | `bundle/verify.go`, `gui/multisig_verify.go`, `gui/multisig_supply.go`, `gui/singlesig_verify.go`, `gui/md1_gather.go` (+ their `_test.go`) | `fix/verify-cluster` | heavyweight (single coherent cycle) |
| **B — secret-scrub batch** | M2, M3, M4, **L1 (codex32_polish site)** | `slip39/combine.go`, `seedxor/seedxor.go`, `gui/bip85.go`, `gui/codex32_polish.go` (+ their `_test.go`) | `fix/scrub-batch` | light (one batched spec/plan/R0) |

Finding key: **H1** `gui/multisig_verify.go:100` mk1 self-comparison false-PASS · **H2** `bundle/verify.go:64` multi-chunk md1 order-sensitive false-FAIL · **M1** `bundle/verify.go:83-97` ms1 entropy-only false-PASS · **M2** `slip39/combine.go:101-117` group-share scrub gap · **M3** `seedxor/seedxor.go:38,44` per-part entropy un-wiped · **M4** `gui/bip85.go:101-108` leaf EC privkey not `Zero()`'d · **L1** `DecodeMS1` probe scrub ×3 sites · **L2** `gui/multisig_verify.go` md1/mk1 tautology + over-claiming success copy.

## Two map corrections the architect found (load-bearing for safe concurrency)
1. **H1's root is in `gui/multisig_supply.go`** — `extractSuppliedMd1` (`:26`) structurally refuses `cardMK1` (`case cardMK1, cardMS1: return nil, false`), so the operator mk1 plate is *never read back*. Fixing only `multisig_verify.go` misses the root. Track A owns `multisig_supply.go`.
2. **L1's `gui/singlesig_verify.go:116` site is *inside* Track A's files** — so Track A takes BOTH verify-flow L1 sites (`singlesig_verify.go:116`, `multisig_verify.go:93`); only the disjoint `gui/codex32_polish.go:103` site goes to Track B. **This split is what keeps the two tracks file-disjoint** — the basis for A∥B concurrency. Do NOT move any Track-A-file L1 site into Track B.

## Dedup decisions (assessed at source, not assumed)
- **H1 ≠ L2 (both kept, Track A):** H1 = wire the *read-back* operator mk1 into `verifyMultisig` (mirror `singleSigReadbackCards`); L2 = the *separate* md1-leg tautology (`reDerived.MD1 = clone(suppliedMd1)` compared to `suppliedMd1`) + the over-claiming success copy at `:104`. Two distinct fix items in one spec.
- **H2 ≠ M1 (both kept, Track A):** different legs of `bundle.Verify` (chunk-order vs entropy-only). Co-located → must be designed together so one implementer edits `bundle/verify.go`.

## Concurrency model — "definitely safe"
- **A ∥ B concurrent through EVERY phase** — spec, spec-R0, plan, plan-R0, implement, exec-review — because: **(a) disjoint file sets** (verified; shared helper `wipeBytes` in `gui/slip39_polish.go` is *called* by both, *edited* by neither; no shared test file) AND **(b) no semantic interaction** (B only adds zeroing of already-discarded buffers — no signature/return/control-flow change any Track-A test observes; the `codex32.DecodeMS1` call sites are independent and neither track edits `codex32`).
- **Implement: A and B in TWO separate worktrees** (`fix/verify-cluster`, `fix/scrub-batch`). **Within each track: strictly serial, single implementer, TDD** (the "no parallel re-implementations" rule). Track B's 4 tiny fixes run sequentially in its one worktree.
- **Merge: SERIAL — B first, then A** (lower-variance; disjoint files make either order conflict-free). Re-test after each merge.

## Gate composition (max safe parallelism per phase)
| Phase | Parallelism | Why |
|---|---|---|
| Spec authoring | A ∥ B | read/design only |
| Spec R0 (opus, →0C/0I) | A ∥ B (distinct reviewers/reports) | read-only; loop each independently |
| Plan authoring | A ∥ B | read/design only |
| Plan R0 (→0C/0I) | A ∥ B | read-only |
| Implement | A ∥ B (disjoint worktrees); serial within a track | disjoint files + no interaction |
| Exec review (whole-diff, mandatory) | A ∥ B | read-only over committed diffs |
| Merge to main | **SERIAL** (B → A) | avoid integration races; re-test each |

Every track runs the full mandatory pipeline; no gate traded for speed. R0/exec reviews persisted verbatim to `design/agent-reports/`; folds re-dispatch until GREEN. Merges `--no-ff -S -s`, author "Brian Goss <goss.brian@gmail.com>", Co-Authored-By trailer.

## Final integration (after both tracks merge)
1. Merge B → main; `go build/test/vet` on main.
2. Rebase A onto updated main (no-op diff expected — disjoint), re-test, merge A → main; re-test.
3. Final integration pass on main: `go build ./...`, `go test ./...` (full), `go vet ./...`, **the TinyGo device-build CI gate** (the real gate — not just host build).
4. **Final whole-repo adversarial exec review over the COMBINED diff** (`3a23dbb`..HEAD) — warranted (Track A rewrites the verification safety net); in addition to each track's per-track exec review. Persist verbatim.
5. Push bg002h only after GREEN.

## Risks the plan hard-codes against
1. **Test-masking (the hunt's headline lesson):** existing unit suites pass synthetic/derived data as "readback" → blind to the real gather→Verify wiring → they keep passing even if H1/H2 are mis-fixed. **Track A's GREEN bar is NEW flow-level tests** (shuffled-chunk md1 → PASS [H2]; mutated engraved mk1 bytes → FAIL [H1]; non-English `mnem` ms1 with matching entropy → FAIL [M1]), each proven to FAIL on `3a23dbb` first. The exec reviewer must confirm the new tests exercise `collected()`/`extractSuppliedMd1`/`ms1Entropy`.
2. **H2 fix-location is load-bearing:** fix in `gui/md1_gather.go:collected()` (return chunks in `ChunkIndex` order, iterate `0..total-1`) — NOT in `bundle/verify.go` (where `[]string` carries no index → forces a signature change rippling into consumers). Spec must LOCK this before plan-R0.
3. **Shared `wipeBytes` (`gui/slip39_polish.go:344`):** Track B fixes only *call* it; spec forbids editing it. A "needs a new helper" urge is a design smell → flag at R0.
4. **TinyGo device build is the real gate** — `defer pkey.Zero()` / `defer`-wrapped scrubs must compile on the device target; final pass runs the TinyGo build, not just host `go build`.
5. **Agent-API dispatch failure** (the 529/rate-limit class that twice aborted the hunt): flag explicitly and defer the formal gate to API recovery — never silently substitute inline self-review for a mandatory R0/exec gate.
