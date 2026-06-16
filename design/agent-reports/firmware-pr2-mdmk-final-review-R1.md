# Firmware PR2 (#35) — final architect review — R1 (convergence)

- **Stage:** final whole-diff review, R1 (re-dispatch after R0 fold). Agent-API dispatch had recovered.
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-reviewer`
- **Scope:** fold commit `6ab12c0` on `feat/engrave-mdmk` (atop `801cf1c`+`5e2336e`).
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Final-review loop converged.

---

## VERBATIM REVIEW OUTPUT

## R1 Final Review — PR #35 fold commit (`6ab12c0`)

Reviewed: `codex32/mdmk.go`, `codex32/checksum.go`, `codex32/codex32.go`, `codex32/mdmk_test.go`, `gui/gui.go`, `gui/mdmk_gui_test.go`, `gui/scan.go`.

**[I1] md1/mk1 case-tolerance — RESOLVED**
- `codex32/mdmk.go:3` imports `strings`; `:96` uses `strings.EqualFold(gotHRP, hrp)`; `:111` feeds original-case `e.inputHRP(gotHRP)`.
- (a) Uppercase validates: `feFromRune`/`invCharsTbl` (`gf32.go:38-47`) and `inputHRP`'s `unicode.ToLower` (`checksum.go:82,90`) make residue case-independent; `setCase` accepts a consistent uppercase run → `isValid()` true. Confirmed.
- (b) Mixed case rejected: lowercase-HRP+uppercase-data and mixed HRP ("Md") both hit `errInvalidCase` in `setCase` (`checksum.go:139-152`) → false. Confirmed.
- (c) Matches `codex32.New` (`codex32.go:109-118`): identical split→inputHRP(original)→inputData→isValid; `EqualFold` only gates the fixed md/mk HRP name. Confirmed.
- (d) No over-admission: `EqualFold` gates HRP name only; case-consistency + BCH `isValid()` still gate. Confirmed.
- `TestMDMKCaseHandling` (`mdmk_test.go:87-100`) asserts both directions; mixed-case also in `TestMDMKNoPanicOnMalformed`.

**[M1] mdmkFlow comment — RESOLVED** (`gui/gui.go:1789-1792`): now says it returns silently "like backupSeedStringFlow, NOT like descriptorFlow." Accurate.

**[M2] TestMdmkNoModeFitsRejected — RESOLVED** (`gui/mdmk_gui_test.go:35-41`): 1200 chars passes `qr.Encode` then fails all three `toPlate` variants → returns `lastErr`; distinct from the 5000-char QR-overflow test.

**[M3] TestMDMKLengthBracket — RESOLVED** (`mdmk_test.go:80`): now `{5, 94, 95, 109}` — adds reserved 94/95 + over-max 109.

**New Critical/Important:** None. `EqualFold`+`inputHRP(gotHRP)` admits nothing extra (mixed-case rejected, BCH gates). No unused imports; logic traces clean. (A noted non-issue: `ValidMK` calls `splitHRP` for the length bracket before the HRP-name check, but `verifyMDMK` still rejects via `EqualFold`/checksum — no leak.)

**Verdict: GREEN (0C/0I)**

---

## Loop summary (added by main session)
| Round | Verdict | Folded |
|---|---|---|
| R0 | NOT-GREEN | 1 Important (md1/mk1 lowercase-only) + 3 Minor (mdmkFlow comment, oversize-test branch, length/case cases) → all folded in `6ab12c0` |
| **R1** | **GREEN** | none — converged |

PR #35 updated with `6ab12c0`; formal final review complete (GREEN). This was the deferred `firmware-deferred-formal-reviews` item (a).
