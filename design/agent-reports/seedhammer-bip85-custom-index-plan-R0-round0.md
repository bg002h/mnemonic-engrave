# BIP-85 custom-index plan — R0 review (round 0) — VERBATIM agent report

**Agent:** `a0a1bdf842eaa9809` (adversarial opus architect; RAN golden + overflow-guard + validator probes on a 64-bit host in a throwaway worktree). **Fork HEAD:** `8459654`. **Plan commit:** `1af1481`. **Date:** 2026-06-20.
**Verdict:** GREEN (0C/0I). 2 non-blocking Minors. Cleared for single-implementer TDD.

---

# BIP-85 custom-index plan — R0 review (round 0)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** 8459654  **Plan commit:** 1af1481  **Verdict:** GREEN (0C/0I)

## Golden + overflow-guard re-verification (MANDATE #1) — RAN
Isolated throwaway worktree off `8459654` (`strconv.IntSize=64`, go1.26.4 amd64); removed after (fork clean on `main @ 8459654`).
(a) **Golden byte-identical, two paths.** In-tree `deriveBip85Child(abandon,"",12,2147483647)` → `jewel solution patient quarter elite grace quarter dinosaur taste parent dial clump` (valid, 12w). index 0 → `prosper short ramp…fold`; index 1 → `sing slogan bar…desert` — match shipped goldens (`bip85_test.go:31`/`:79`). biptool (SEPARATE `bip32.ParsePath` path), master XPRV `xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu`: `…/2147483647h` → byte-identical; `…/2147483648h` → `bip32: path element out of range`; `…/0h`/`…/1h` match. Plan goldens CORRECT.
(b) **Silent truncation REPRODUCED + guard catches it.** UNGUARDED `deriveBip85Child(…,12,1<<31)` → `err=nil`, `"success fuel awesome…"`; `1<<31+1` → `err=nil`, `"blossom december breeze…"`. `uint32(1<<31)+h=0` (UNHARDENED element 0), `uint32(1<<31+1)+h=1` (UNHARDENED element 1) — off-spec, no error. Ran the plan's guard (`index<0` then `index > bip85MaxIndex`, `bip85MaxIndex = hdkeychain.HardenedKeyStart-1 = 2147483647`, BEFORE the `:54` cast): ERRORS on `1<<31`,`1<<31+1`,`1<<40`; ACCEPTS `0`/`1`/`2147483647`; distinct messages (`"bip85: invalid index: -1"` vs `"bip85: index 2147483648 exceeds the maximum 2147483647"`, R0-M3). Guard placement CORRECT.

## Two-layer validator/guard (MANDATE #2)
- `parseBip85Index` uses `strconv.ParseUint(s,10,64)` (never bare `int`). Ran the matrix: ACCEPT `"0"/"7"/"007"/"1000000"/"2147483647"`; REJECT `""/"12a"/"a12"/"-1"/"+1"/" 1"/"1 "/"0x10"/"1.0"/"2147483648"/"9999999999"/"9223372036854775808"/"١٢"`. `"9999999999"` (10 digits) is **RANGE-rejected** (`>2147483647`), not length — R0-M2 satisfied, validator is the range authority; no early length cap. `int(v)` return safe (`v≤2^31-1`).
- `deriveBip85Child` guard independent of the picker (only production caller `bip85DeriveFlow` `:220`); closes direct/test/future callers; no residual un-rejected `uint32()` site; no bare-`int` parse anywhere.

## Keyboard reuse + re-prompt + scope (MANDATE #3)
- `bip85IndexEntryFlow` byte-faithful clone of `passphraseFlow` (`gui.go:509-536`)/`typeAddressFlow` (`verify_address.go:44-71`) with `NewAddressKeyboard` (cleartext), Back(Button1)→`(0,false)`, OK(Button3)→`parseBip85Index(kbd.Fragment)`, parse error → `showError`+`kbd.Clear()`+`continue` (re-prompt, no silent 0/abort). All symbols exist: `Fragment`(`passphrase_keyboard.go:48`), `Clear()`(`:171`), `NewAddressKeyboard`(`:133`), `showError`/`ErrorScreen.Layout` Button3-dismiss (`gui.go:223-225`). `bip85ParamPickFlow` swaps the `0..9` ChoiceScreen (`:128-137`); `bip85IndexChoices`(`:112`) removed (grep: only `:106/112/137` + the re-pinned test).
- **Clear-on-error adjudication ENDORSED** (less-confusing common choice; `kbd.Clear()` exists `:171`).
- **m\*-free + firmware-only CONFIRMED:** changes only `gui/bip85.go`+`gui/bip85_test.go`; +`strconv`, +`gui/layout` (already used by the clone source); no `md`/`mk`/`codex32`; no new program/enum/t5-M1-guard/lockstep/CLI/schema/docs-mirror/SemVer. Security spine unchanged (typed-only master, scrub, child steel-only never NFC, mainnet-only; index public). Output unchanged. Baseline tests + `go vet ./gui/` green @HEAD.

## Coverage + Minors + quality (MANDATE #4)
- Invariant map complete: I-1 (Task 2 golden + index-0; Task 4 Step 0 re-probe), I-2 (Task 1 validator + Task 2 guard + Task 5 fuzz `1<<31`/`1<<31+1`; silent-truncation is the explicit Task-2 failing test), I-3 (scope), I-4 (security spine; two-secret scrub kept).
- 3 spec-Minors folded: M-1 (fuzz success-assert `index>bip85MaxIndex` + corpus `1<<31` AND `1<<31+1`), M-2 (validator range authority; `"9999999999"` range-rejected), M-3 (distinct upper message).
- `TestBip85ParamBounds` re-pin (Task 4 Step 1) drops the enumerated `bip85IndexChoices` check (live `:138-163`) for the validator contract; "Old" snippets match source byte-for-byte (picker `:128-137`, fuzz, doc-comment `:106-112`).
- Quality: no placeholders; failing-test-first w/ concrete FAIL→PASS; per-task import accretion compiles (`strconv` Task 1, `gui/layout` Task 3); signatures consistent (`parseBip85Index(string)(int,error)`, `bip85IndexEntryFlow(*Context,*Colors)(int,bool)` — `*Colors` matches `&descriptorTheme`; `bip85MaxIndex` untyped const); idioms match shipped helpers; `bip85_test.go` already imports `testing/synctest`.

## Critical / Important
None / None.
## Minor
- **m-1 (UX nit, non-blocking).** `kbd.Clear()` (`passphrase_keyboard.go:171-178`) resets `Fragment=""` AND `revealed=false`. Since the index is public (`NewAddressKeyboard` sets `revealed=true`), Clear() re-prompts MASKED (`***`). Not a correctness bug (tests don't assert reveal). Optional: re-set `kbd = NewAddressKeyboard(ctx)` (or `kbd.revealed=true`) after Clear() to keep cleartext. Implementer's discretion.
- **m-2 (polish).** Task 3's `"5"` re-entry note is verbose but the impl/test is correct. No action.

## Verified-correct
High-index golden 2 independent paths; index 0/1 shipped-unchanged; master XPRV; `2^31` rejection (biptool + reproduced unhardened-wrap); silent-truncation bug reproduced (UNHARDENED elements 0/1, err=nil); guard placement + distinct messages; validator matrix incl. range-not-length `"9999999999"`; all cited file:lines @8459654; only-production-caller fact; BIP-85 index max `2^31-1` vs `HardenedKeyStart`; baseline tests+vet green. Worktree cleaned.

## Bottom line
**GREEN (0C/0I).** Ran the verification on a 64-bit host: the golden is byte-identical across two code paths; the silent `uint32`-truncation bug is real and reproduced (UNHARDENED elements 0/1, `err=nil`); the plan's two-layer defense (`parseBip85Index` ParseUint(…,64) rejecting `>2^31-1` + the independent `index > bip85MaxIndex` guard before the cast) closes every path incl. direct callers, with distinct messages. All three spec-Minors folded; both ambiguities honored (leading-zero ACCEPT, empty→re-prompt); keyboard clone byte-faithful; scope m*-free/firmware-only/no-lockstep; every "Old" snippet matches live source; each task failing-test-first. Two non-blocking Minors (Clear() re-masks the public index; a verbose note). Proceed to single-implementer TDD in `feat/bip85-custom-index`, then the mandatory whole-diff exec review. Re-run the Task 4 Step 0 staleness probe at impl time.
