# Cycle B: codex32-multishare-recovery — WHOLE-DIFF ADVERSARIAL EXECUTION REVIEW

- **Stage:** mandatory, non-deferrable post-implementation execution review over the whole diff (ultracode phase 4). Read-only; ran the toolchain + scratch edge-case probes.
- **Diff reviewed:** branch `feat/codex32-multishare-recovery`, base `bf7f811` … head `11bd74f` (3 implementation commits). 5 files.
- **Reviewer:** `feature-dev:code-reviewer` (agentId `a02551a5b86fcae1a`).
- **Outcome:** **SHIP-READY — 0 Critical / 0 Important.** 1 Minor (M1, coverage) — folded.

> NOTE: verbatim reviewer output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

## INDEPENDENT ADVERSARIAL EXECUTION REVIEW — Cycle B (codex32 multi-share recovery)
**Branch:** `feat/codex32-multishare-recovery` | **Base:** `bf7f811` | **Head:** `11bd74f` | **3 commits**
**Reviewed files:** `codex32/polish.go`, `codex32/polish_test.go`, `gui/codex32_polish.go`, `gui/codex32_polish_test.go`, `gui/gui.go`
**Reviewer:** Independent adversarial execution review (non-author)

---

### CRITICAL

None.

---

### IMPORTANT

None.

---

### MINOR

**M1 — `TestConsistentShares` coverage gap: `errMismatchedLength`, `errMismatchedHRP`, `errMismatchedID` never exercised**
File: `codex32/polish_test.go`, `TestConsistentShares`

The test exercises only `errRepeatedIndex` (`a,a`) and `errMismatchedThreshold` (`a,cash`). The `a,cash` pair differs in threshold (2 vs 3) AND id (NAME vs CASH), but since the switch in `ConsistentShares` checks threshold before id, the id-mismatch branch is never reached. The length and HRP branches are similarly untested. The implementation is three sequential field comparisons following the same pattern as the two tested cases, so silent regression risk is low — but a future refactor (e.g., reordering the switch cases, or changing a field name) could silently break one of those branches without a test catching it.

Fix (optional, non-blocking): add two rows to `TestConsistentShares`:
- A share with different id but same threshold (requires constructing or finding a test vector with threshold=2 and a different 4-char id and valid checksum).
- A share with different length (a long-code share vs a short-code share).

These are meaningful for a security-critical codex32 package but are not a bug today.

---

### Detailed trace notes (evidence for SHIP-READY verdict)

**`ConsistentShares` panic safety:** `ConsistentShares` calls `share.parts()` for each share. `parts()` panics only if `partsInner` returns an error, which it does only for malformed strings (invalid threshold digit, invalid share index). All callers pass `New`-valid strings (the keypad gates the OK button on `New==nil`). The `ConsistentShares` loop iterates the full slice including `shares[0]`, but self-comparison is harmless — all fields match, dedup map is initially empty for `shares[0]`. No panic path for `New`-valid inputs.

**Sentinel order matches `Interpolate`:** `Interpolate` pass-1 checks length → HRP → threshold → ID (lines 201–211 of `codex32.go`). `ConsistentShares` follows the identical order. `errRepeatedIndex` is checked in a separate dedup pass, same as `Interpolate`'s second phase. The `Describe` mapping covers all six cross-share sentinels plus the existing per-share sentinels; no double-mapping or shadowing (each is a distinct `errors.New` value checked with `errors.Is` exact equality).

**`TestDescribe` R1-1 correctly addressed:** The `{errInsufficientShares, "invalid"}` row was replaced with `{errInsufficientShares, "need more shares"}` and the six cross-share sentinel rows were inserted. No stale "invalid" row remains for any now-mapped sentinel.

**Button2 drain (R0 C1):** `recoverClicked := recoverBtn.Clicked(ctx)` is called unconditionally on every frame in `confirmCodex32Flow`, before `engraveBtn.Clicked(ctx)`. For an unshared secret, Button2 is consumed but the `!f.Unshared && recoverClicked` guard prevents the Recover action. `TestConfirmCodex32UnsharedNoRecover` confirms this with `click(Button2, Button3)`: Button2 is drained in frame 1, Button3 is consumed in frame 1 (same iteration, after Button2), returning `codex32Engrave`. Traced through `EventRouter.Next`'s head-matching semantics — no hang possible in either direct-call or `runUI` mode.

**Recover NavButton render:** `if !f.Unshared { navBtns = append(...recoverBtn...) }` — Recover button is appended only for shares. The NavButton conditional is inside the frame-rendering path (after all event-draining), so it doesn't affect event handling for the already-consumed `recoverClicked` value. Correct.

**`recoverCodex32Flow` loop termination:** Loop exits when `len(shares) == k` (exact). `k = f.Threshold` from the first share's header; `New`-valid guarantees `ThresholdKnown && Threshold ∈ {2..9}`. The `!f.ThresholdKnown || f.Threshold < 2` guard is unreachable for `New`-valid shares but correctly documented. Back (`!ok`) returns `(_, false)`. The `append(shares, cand)` for the `ConsistentShares` check is a temporary slice; `shares` is not mutated on rejection. Go's slice growth (cap doubles) means the backing array for the temp slice may diverge from `shares`' backing array, but `shares` is never read beyond its length before the next `append(shares, cand)` on success — confirmed safe.

**Unshared-secret rejection wired correctly (M4):** `pf.Unshared` is checked BEFORE `ConsistentShares` is called, with the explicit message "enter a share, not the secret" (not the generic "bad share index"). This matches the spec's M4 requirement.

**`Interpolate` defense-in-depth:** After `ConsistentShares` passes and exactly k shares are in `shares`, `Interpolate(shares, 'S')` is called. `ConsistentShares` plus exactly-k shares guarantees `Interpolate`'s pass-1 fields are consistent and `s0Parts.threshold == k == len(shares)`. The only remaining way `Interpolate` could fail is `errRepeatedIndex` (caught by `ConsistentShares`' dedup pass) or `errInvalidShareIndex` for 'S' (impossible — 'S' is in the bech32 alphabet). The `if err != nil` handler is genuine defense-in-depth with a correct error modal, not dead code.

**`engraveCodex32` return contract:** Returns `true` for `codex32Back`, `true` after `backupSeedStringFlow` for `codex32Engrave`, and loops (never returns false) for `codex32Recover`. `engraveObjectFlow`'s `case codex32.String: return engraveCodex32(...)` short-circuits the function's `return true` at line 1850, but since `engraveCodex32` always returns `true`, the net effect is identical. The other cases (`bip39.Mnemonic`, `*bip380.Descriptor`, `mdmkText`) fall through to `return true` unaffected.

**`TestEngraveCodex32BackoutNotUnknown` (A1 regression):** Button1 → `confirmCodex32Flow` returns `codex32Back` → `engraveCodex32` returns `true` → `engraveObjectFlow` returns `true`. Test still passes.

**`TestRecoverCodex32` BIP-93 vector-2:** Shares A (`MS12NAMEA...`) and C (`MS12NAMEC...`) produce secret S (`MS12NAMES6XQ...`) confirmed against `codex32_test.go`'s own use of the same vector. `Interpolate`'s Lagrange math is existing, tested, BIP-93-conformant code. The direct-call test pre-queues 48 rune events + Button3; `for kbd.Update(ctx) {}` drains all 48 runes in one frame, Button3 confirms in the same frame iteration. No spin-wait.

**`TestRecoverCodex32Mismatch` reliability:** The CASH share (`MS13CASHA...`) differs from the NAME set in threshold (3 vs 2) — `ConsistentShares` returns `errMismatchedThreshold` ("mismatched threshold") immediately. All 48 rune events are drained in one `frame()` call; Button3 confirm and `showCodex32Error` invocation happen in at most the next frame. `uiContains(content, "mismatched")` fires within 3 frames. The 8-frame poll is not flaky.

**`inputCodex32Flow` title param:** Signature changed to `(ctx, th, title string)`. Local `titleOp, _ := layoutTitle(ctx, dims.X, th.Text, title)` uses `titleOp` — no shadowing. Both callers updated: `newInputFlow` passes `"Input Codex32 Share"`, `recoverCodex32Flow` passes dynamic "Share N of K · id ID". `TestInputSeedCodex32` calls `newInputFlow` (not `inputCodex32Flow` directly) — unaffected. `codex32Frame` helper updated to pass `"Input Codex32 Share"`.

**Scope verification:** `codex32.go` and `mdmk.go` were read and contain no new code from this cycle (error vars, `Interpolate`, `Split`, `parts`, `New` all unchanged). Imports in `codex32_polish.go` are all used (`backup` and `constant` for `engraveCodex32`, `image` for `image.Pt` in `confirmCodex32Flow`). Commit message carries `Signed-off-by: Brian Goss <goss.brian@gmail.com>` (DCO). SSH signing outcome not directly verifiable from worktree reads, but the implementer noted the `-s` flag was used.

---

### Verdict

**SHIP-READY — 0 Critical / 0 Important**

The implementation correctly delivers `ConsistentShares`, the `Describe` extensions, `confirmCodex32Flow` → action enum with Button2 unconditional drain (R0 C1 fix), `recoverCodex32Flow`, and `engraveCodex32`, all matching the GREEN plan and spec. The test suite is non-vacuous and covers the critical paths. The one minor gap (three untested branches in `ConsistentShares`) is a coverage observation, not a defect, and does not block merge. The BIP-93 vector-2 recovery path is end-to-end verified by `TestRecoverCodex32`. The A1 regressions are green. No behavioral drift from the plan was introduced.


---

## Fold note (main session)
Verdict **SHIP-READY (0C/0I)** — gate passed. **M1 (FOLDED, partial):** `TestConsistentShares` didn't exercise the length/HRP/ID-mismatch branches in isolation (the `a,cash` pair hits threshold-mismatch first). Added a **length-mismatch** case — short NAME share (48) vs the BIP-93 vector-5 long secret (127) → `errMismatchedLength` — which is cleanly testable with the existing corpus (commit `a473db1`, test-only, full suite re-run ALL PASS). `errMismatchedID`/`errMismatchedHRP` are **not exercisable** without fabricated vectors (the corpus has no same-threshold/different-id share, and every valid string uses HRP "ms"); the reviewer agreed these branches are trivially-correct sequential field comparisons, so this residual gap is documented, not a blocker. Production code unchanged by the fold → the SHIP-READY verdict stands; no re-review needed.

Post-fold head: `a473db1` (4 commits).
