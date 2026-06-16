# Firmware PR2 plan-R1 review — BCH-validated md1/mk1 engraving

- **Stage:** implementation-plan R1 (re-dispatch after plan-R0 fold) — convergence check
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect`
- **Plan:** `design/IMPLEMENTATION_PLAN_firmware_pr2_mdmk_engrave.md` (post plan-R0 fold)
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Plan cleared its pre-implementation R0 gate.

---

## VERBATIM REVIEW OUTPUT

## Per-finding verification

**[C1] `inputData(string)` + `splitHRP`/`inputHRP` signatures — RESOLVED.**
- `inputData(s string) error` — checksum.go:114. Plan passes `data` (a string from `splitHRP`). ✓
- `splitHRP(s string) (string, string)` — codex32.go:453. Plan's `gotHRP, data := splitHRP(s)` matches. ✓
- `inputHRP(hrp string) error` — checksum.go:77; `codex32.New` uses `if err := check.inputHRP(hrp); err != nil` (codex32.go:110). Plan matches. ✓
- `unpackSyms` used only for `target`/`residue` (`[]fe`), not data. ✓

**[C2] MK_LONG hi/lo — RESOLVED.** `0x418<<64 | 0x90d7e441cbe97273 == 0x41890d7e441cbe97273`. ✓ Both halves fit uint64. ✓

**[C3] NewErrorScreen — RESOLVED.** Signature `NewErrorScreen(err error)` (gui.go:384). `mdmkFlow` now bare-`return`s on error; no call to a non-existent signature, so it compiles. ✓

**[I1] consts provenance — RESOLVED.** Iteration-points note + #1-risk callout mandate Rust-GENERATED golden vectors (never Go-self-generated) as the hard gate for POLYMOD_INIT/targets/generator. ✓

**[I2] mk length gate — RESOLVED.** `ValidMK` calls `mkIsLong(s)`, flagged as to-be-derived from mk-codec `bch_code_for_length` by data-part length (NOT codex32's bounds); long-mk1 vector required. Correctly scoped and gated. ✓

## New-issue sweep
- **`verifyMDMK` compiles** against real signatures: `&engine{generator, residue, target}` — all three `[]fe` fields (checksum.go:15-17); unset `_case` defaults `noCase` (matches a fresh engine). ✓
- **`mkIsLong` is a named, flagged undefined helper** — would not compile until implemented (desired loud failure), not a compile-but-misbehave stub. ✓
- **`unpackSyms` shift-safe:** `shift==0` → else branch, `hi<<64==0` in Go (defined), `v=lo`. ✓ `shift>=64` (n=15, max shift 70) → `hi>>(shift-64)`; for hi=0x418, `0x418>>6=0x10`. ✓ For n=13 max shift 60<64, hi=0 throughout. ✓ `shift==64` never occurs (shifts are multiples of 5). ✓

No new Critical or Important. One non-blocking Minor (by design): `mkIsLong` undefined until implemented — covered by the SPIKE/iteration-point framing.

## Verdict: GREEN (0C/0I)
All 3 Critical + 2 Important folds verified RESOLVED against real signatures (`codex32/checksum.go:11-17,77,114`; `codex32/codex32.go:98-124,453`; `gui/gui.go:384`). No new C/I; `unpackSyms` shift-safe for n=13 and n=15.

---

## Loop summary (added by main session)

| Round | Verdict | Folded |
|---|---|---|
| plan-R0 | NOT-GREEN | 3C (`inputData([]fe)`; MK_LONG hi/lo overflow; `NewErrorScreen` misuse) + 2I (consts provenance; mk length-gate stub) |
| **plan-R1** | **GREEN** | none — converged |

Plan cleared the mandatory pre-implementation R0 gate. Eligible for execution; the BCH port's #1 risk (init residue = POLYMOD_INIT, not 1) is gated by the Rust-sourced parity test.
