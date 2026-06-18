<!--
Persisted verbatim. opus-architect R1 re-dispatch of the Seed XOR combine plan R0 gate
(@ 225bc1e). Reviewer agentId afc59fae5851e7bf3. Verdict: GREEN — 0C/0I. Both R0 Importants (I-1
title render contract, I-2 all-10-callers) folded correctly + verified vs shipped bc63caa; M-1
offline oracle folded; no drift. One non-blocking MINOR M-3 (table-row '2 callers' phrasing) —
tidied post-review. Cleared for implementation. Do not edit.
-->

# R1 GATE REVIEW — Seed XOR combine plan

**Reviewer:** opus architect (adversarial R1 re-dispatch of the R0 gate, read-only)
**Plan:** `design/IMPLEMENTATION_PLAN_seedhammer_seedxor.md` (commit `225bc1e`)
**R0 review folded:** `design/agent-reports/seedhammer-seedxor-plan-review-R0.md` (0C / 2I + 2m)
**Base verified against:** fork `main` `bc63caa814e20d2de6140e40026015456c35a2b0`
**Port oracle:** `mnemonic_toolkit::seed_xor::seed_xor_combine`
**Date:** 2026-06-18

---

## Verification Results

### I-1 — render contract is now genuinely additive ✔
Task 2 Step 1 (plan lines 164-172) now specifies the exact two-branch contract:
- `title == ""` → "render the EXISTING `layoutTitlef("Word %d of %d", selected+1, len(mnemonic))` line byte-identically" (line 167-169).
- `title != ""` → "render `layoutTitle(ctx, dims.X, th.Text, title)` in place of the word-position line … like `inputSLIP39Flow`" (line 170-172).

Verified against shipped source:
- The dynamic title line exists exactly as the plan quotes it — `gui.go:701`: `title, _ := layoutTitlef(ctx, dims.X, th.Text, "Word %d of %d", selected+1, len(mnemonic))`. The empty-title branch preserves this verbatim.
- `inputSLIP39Flow` does render only a free-form `layoutTitle(..., title)` — `gui.go:868`: `titleOp, _ := layoutTitle(ctx, dims.X, th.Text, title)`, and its signature already carries `title string` (`gui.go:796`). So the non-empty branch is modeled on real, shipped precedent.
- `TestWordFlowProgressTitle` (`gui_test.go:487-500`) calls `inputWordsFlow(ctx, &descriptorTheme, m, 0)` and asserts `uiContains(content, "Word 1 of 24")` (`gui_test.go:498`). With the empty-title caller (`""`), the contract renders the `layoutTitlef` line unchanged, so the assertion stays green.

This is implementable as a simple conditional on the existing `gui.go:701` line (`if title == "" { layoutTitlef(...) } else { layoutTitle(..., title) }`) — additive, byte-identical for the empty-title path. I-1 folded correctly.

### I-2 — all 10 callers enumerated and complete ✔
`grep -n 'inputWordsFlow('` over both files returns exactly:
- `gui.go:580` — the `func` definition (not a call).
- `gui.go:2025`, `gui.go:2102` — the 2 production callers (both pass `selected` as a literal/field; adding `""` preserves behavior).
- `gui_test.go:285, 491, 507, 604, 625, 642, 662, 681` — the 8 test callers (all 4-arg `inputWordsFlow(ctx, &descriptorTheme, m, N)`).

Task 2 Step 2 (plan lines 173-177) lists precisely `gui.go:2025, :2102` and `gui_test.go:285, :491, :507, :604, :625, :642, :662, :681` — a complete, exact match to the grep, 2 + 8 = 10, with the def at `:580` correctly excluded. Passing `""` at each preserves the "Word N of M" behavior, so `./gui/` compiles and the existing tests (including `TestWordFlowProgressTitle`) stay green. I-2 folded correctly.

### M-1 — offline oracle folded ✔
Task 1 Step 1 (plan lines 55-66) now names the toolkit in-repo G1 byte-pin / G2 round-trip (`tests/lib_seed_xor.rs`) as the **primary offline oracle** ("authoritative correctness anchor"), keeps the Coldcard vectors as an **interop cross-check**, and explicitly says "do NOT block on the fetch — note any deferred interop vector." Folded as recommended.

### M-2 — redundant copy ✔
`append([]byte(nil), parts[0].Entropy()...)` retained as defensive hygiene (plan line 114); R0 marked it "no change required." Undisturbed.

### No-drift check — R0-verified elements undisturbed ✔
- `seedxor.Combine` inline Go: pure XOR fold, `interopLen(n) == 16||24||32` load-bearing guard (lines 97-99, 115), `append([]byte(nil), …)` copy (line 114), `bip39.New(out)` safety with the `len(out) ∈ {16,24,32}` justification (line 129) — all unchanged from R0.
- I1 per-part guard `if !isMnemonicComplete(m) || !m.Valid()` before collecting each part (lines 247-249) — unchanged.
- Mandatory Button2-drained Seed-XOR fingerprint gate on the only success path (lines 262-265, 268-273); clone of `confirmSLIP39Fingerprint` with the unconditional `drainBtn.Clicked(ctx)` — matches shipped `slip39_polish.go:439,445`. Unchanged.
- Menu `case 4` + `bip39.Mnemonic` dispatch reuse (lines 284-287): verified the switch is index-based `case 0,1 / 2 / 3` with no existing `case 4` (`gui.go:2022-2056`), and `case bip39.Mnemonic:` already exists at `gui.go:1849`. No new dispatch case. Unchanged.
- Port fidelity section: unchanged.

**Stale-residue grep.** No "Likely \"\"", no "render exactly as inputSLIP39Flow does" without the empty-title contract. One hit for the literal `2 callers` — at **line 36**, the File-structure table `gui/gui.go` row: "update the 2 callers." This is *scoped to the gui.go row*, where there are genuinely exactly 2 call sites (`:2025`, `:2102`); the 8 test callers live in the separate `gui/gui_test.go` row. It is locally accurate, and the authoritative instruction (Task 2 Step 2, lines 173-177) and self-review (lines 320-321) both correctly enumerate all 10. An implementer following Task 2 updates all 10 sites and compiles green. This is a documentation-phrasing nit echoing the old undercount, not a correctness defect — recorded MINOR, non-blocking.

---

## Findings

### CRITICAL
None.

### IMPORTANT
None.

### MINOR
- **M-3 (new, non-blocking) — table-row phrasing echoes the R0 undercount.** Plan line 36 (`gui/gui.go` File-structure row) says "update the 2 callers." It is technically correct scoped to `gui.go` (2 call sites there), and Task 2 Step 2 authoritatively lists all 10, so it does not mislead an implementer. Optional tidy: change to "update the gui.go callers (all 10 sites incl. gui_test.go enumerated in Task 2)" to remove any residual echo of the R0 undercount.
- **M-1 / M-2** — folded / retained as above; no action.

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

Both Important findings are folded correctly and verify against shipped fork source at `bc63caa`: I-1's title contract is genuinely additive (`title==""` preserves `gui.go:701`'s `layoutTitlef` byte-identically, keeping `TestWordFlowProgressTitle`/`gui_test.go:498` green; `title!=""` mirrors `inputSLIP39Flow`/`gui.go:868`), and I-2's 10-caller enumeration exactly matches the grep (2 in gui.go + 8 in gui_test.go, def at `:580` excluded). M-1's offline toolkit G1/G2 oracle is folded. No drift: the `seedxor.Combine` internals, I1 panic guard, mandatory Button2-drained gate, menu `case 4` + `bip39.Mnemonic` dispatch reuse, and port fidelity are all undisturbed. The single `2 callers` residue at line 36 is a scoped, locally-accurate table-row phrasing (MINOR M-3, non-blocking). Plan is cleared for implementation.
