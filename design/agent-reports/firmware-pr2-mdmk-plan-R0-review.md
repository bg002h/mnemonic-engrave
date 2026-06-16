# Firmware PR2 plan-R0 review — BCH-validated md1/mk1 engraving

- **Stage:** implementation-plan R0 gate (pre-code)
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (verified Go claims against the fork; Rust consts were outside its checkout scope — see I1)
- **Plan:** `design/IMPLEMENTATION_PLAN_firmware_pr2_mdmk_engrave.md`
- **Verdict:** **NOT-GREEN** — 3 Critical, 2 Important, 2 Minor. Folded below; plan-R1 required.

---

## VERBATIM REVIEW OUTPUT

Scope note: the plan's cited Rust "source of truth" files do not exist in the seedhammer checkout — only `crates/me-cli/**` adjacent. All Go-side claims verified against `/scratch/code/shibboleth/seedhammer`; Rust consts could not be verified from the fork and remain unaudited there.

## CRITICAL

- **C1 — `inputData` takes `string`, not `[]fe`. Plan does not compile.** `checksum.go:114` `func (e *engine) inputData(s string) error`. The plan calls `e.inputData(dataSyms)` with `dataSyms []fe`. Type error. The verify path must pass the raw data substring (the part after `1`), exactly as `codex32.New` does (`codex32.go:113` `check.inputData(res)` where `res` is a `string`). There is no `[]fe` decode step to mirror — `inputData`→`inputChar`→`feFromRune` decodes internally.

- **C2 — MK_LONG hi/lo split is wrong and overflows uint64.** Plan: `unpackSyms(0x4, 0x1890d7e441cbe97273, 15)`. `0x1890d7e441cbe97273` is 17 hex digits = 68 bits, exceeding uint64 max → Go constant-overflow compile error. Correct split of `0x41890d7e441cbe97273` (75 bits): **lo=`0x90d7e441cbe97273`**, **hi=`0x418`**. Reconstruction `0x418<<64 | 0x90d7e441cbe97273 = 0x41890d7e441cbe97273` ✓.

- **C3 — `NewErrorScreen` signature mismatch; `mdmkFlow` does not compile.** Plan: `NewErrorScreen("Too Large", "...").Flow(ctx, th)`. Actual: `gui.go:384` `func NewErrorScreen(err error) *ErrorScreen` (one `error` arg), and `ErrorScreen` has **no `Flow`** — only `Layout(ctx, th, dims)`. The render pattern is the `for !ctx.Done { ... Layout ... ctx.Frame }` loop in `backupWalletFlow` (`gui.go:1748-1757`), or just `return` on error like `descriptorFlow`'s siblings.

## IMPORTANT

- **I1 — Rust consts and golden vectors unverifiable from the fork.** The init-residue (`POLYMOD_INIT=0x23181b3`), targets, and `GEN_REGULAR==newShortChecksum().generator` could not be checked from within the fork (Rust crates absent there). The Step-0 spike + parity gate is the only guard and must run against vectors produced by the actual Rust tooling, not transcribed values. Re-confirm `GEN_REGULAR` equals the codex32 short generator before reusing it — if the polynomials differ, the reuse strategy is invalid.

- **I2 — mk regular/long selection is an unfilled stub controlling correctness.** `if /* long-length */ false` always takes regular, so any long mk1 is mis-verified. The real gate (`codex32.go:99-107`) selects by total string length (codex32: short 48–93, long 125–127); md/mk have different lengths, so those bounds cannot be copied — derive mk's own length gate (and a long mk1 vector is required to test it).

## MINOR

- **M1 — `engine` has four fields** (`_case charCase, generator, residue, target []fe`, `checksum.go:11-18`), not three; the composite literal omitting `_case` defaults fine, and a fresh engine per call is required (plan does this ✓).
- **M2 — prose wiring:** `validateMdmk` should mirror `validateDescriptor` (`gui.go:399-447`), NOT `descriptorFlow` (which uses `DescriptorScreen.Confirm`). The plan's `validateMdmk` body is correct; fix the prose.

## Confirmed correct
- Init-residue fix is real/necessary: `inputFe` reads `e.residue[0]` and never resets — seeding `POLYMOD_INIT` (not codex32's 1) is the right mechanism; `isValid` compares `residue==target` ✓.
- `unpackSyms` MSB-first (incl. n=15 cross-boundary) correct once C2's hi/lo is fixed.
- Case-insensitivity free (HRP `unicode.ToLower`; data `feFromRune`→`invCharsTbl` maps both cases).
- Pure verify, no correction → single-char tamper rejected for both md1/mk1; tamper/all-zeros tests sound.
- `Plate`, `toPlate`/`ErrTooLarge`, `EngraveText`, `Text`/`Paragraph`/`QRScale`, `ChoiceScreen.Choose`, `NewEngraveScreen`, `qr`/`sh` imports exist and match.
- TinyGo-safe: no math/big, uint64-only in mdmk.go.
- `scan.go` branch placement after `codex32.New` before final `else` valid; `mdmkText` local type compiles.

## Verdict: NOT-GREEN
1. C1 — `inputData(string)`: pass the data substring, drop the `[]fe` arg.
2. C2 — fix MK_LONG split to `hi=0x418, lo=0x90d7e441cbe97273`.
3. C3 — `NewErrorScreen(err error)` + Layout-loop (no `.Flow`).
4. I1 — pin Rust-generated golden vectors; verify `GEN_REGULAR`==short generator + all consts against the Rust source.
5. I2 — implement the real mk short/long length gate (derive mk bounds; not codex32's) + supply a long mk1 vector.

---

## FOLD DISPOSITION (added by main session)

| Finding | Sev | Disposition |
|---|---|---|
| C1 `inputData(string)` | **Critical** | FOLDED — `validate` now splits HRP, passes the data **string** to `inputData`, handles its `error`; `unpackSyms` retained only for target + init residue (those are `[]fe`). |
| C2 MK_LONG hi/lo | **Critical** | FOLDED — `unpackSyms(0x418, 0x90d7e441cbe97273, 15)`. |
| C3 NewErrorScreen | **Critical** | FOLDED — `mdmkFlow` returns on error (like descriptorFlow siblings); optional ErrorScreen via the `backupWalletFlow:1748` Layout loop noted. |
| I1 consts provenance | **Important** | FOLDED — plan now cites the verified Rust sources (md-codec `src/bch.rs`, mk-codec `src/consts.rs`; GEN_REGULAR==codex32 short generator proven in earlier recon) and mandates Rust-generated parity vectors. |
| I2 mk length gate | **Important** | FOLDED — derive mk's regular/long selection from mk-codec `string_layer/bch.rs` `bch_code_for_length` (by data-part length); long mk1 vector required for Task 2. |
| M1/M2 | Minor | FOLDED — `_case` field noted; prose fixed to mirror `validateDescriptor`. |

plan-R1 re-dispatch follows.
