# SeedHammer input Slice 1 (BIP-39 word-entry polish) — SPEC review — R2 (convergence)

- **Stage:** spec R2 convergence gate (verify R1's 1C folded; confirm internal consistency; fresh-eyes re-scan).
- **Date:** 2026-06-17
- **Reviewer:** opus `feature-dev:code-architect` (read-only; verified against `bip39/bip39.go`, `bip39/wordlist.go`, `gui/gui.go`, `gui/gui_test.go`, `gui/codex32_input_test.go`).
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Cleared to proceed to writing-plans.

---

## VERBATIM REVIEW OUTPUT

**VERDICT: GREEN — 0 Critical / 0 Important.** Cleared to proceed to writing-plans.

### 1. C1 (R1) — fully folded and consistent
The 24-word count is now **8** in every normative location; independently re-derived: `splitMnemonic` sets `checkBits = len(m)/3` (`bip39/bip39.go:159`) → 8 (24-word) / 4 (12-word). The last 11-bit group's checksum occupies its low `checkBits` bits; free entropy occupies the high `11 - checkBits`. 24-word: `11-8=3` free bits → 2³ = **8** valid last words (each entropy choice fixes the checksum to exactly one word; `wordlist.go` has 2048 distinct labels, no collapse). 12-word: `11-4=7` → 2⁷ = **128**. Four required sites all say 8: §4.4 math (line 77), `LastWordCandidates` doc-comment (line 86), §6 24-word integration test (line 119), §6 unit-test bullet (line 115); plus §9 (line 138). No lingering contradictory wording — surviving `nvalid==1`/`1 match` (lines 36,52) describe generic narrow-to-one / existing `completeBIP39Word` (`gui.go:866`); lines 101/106 "no pre-seed" are explicit negations; line 101 "no `len(cands)==1` special case — neither supported length yields a single candidate." ✓

### 2. Unification — no regression; composes correctly
Both lengths route through one candidate-scoped path over an 8- or 128-element `[]Word` set:
- Helper (line 101 a/b/c): mask via `LabelFor(w)[len(frag)]` OR-clear (= `updateValidKeys` mechanism, `gui.go:921-930`); `nvalid` = matching-candidate count; complete on `nvalid==1` OR `frag ==` a *candidate* label. Length-agnostic; closes the I2 hole (does not reuse `completeBIP39Word`'s exact-full-label clause, `gui.go:866`). ✓
- OK nav button lights via `complete`, no pre-seed, no coordinate-tap helper (I4): consistent with the real lighting path (`okBtn` shown only when complete, `gui.go:605-606`); no injected fragment → nothing for the next frame to fight (line 106). ✓
- I1 guard/clone (lines 84-92): `len%3` + per-word `<0`/`>=NumWords` guard before any `Valid()`, then `slices.Clone`; correct vs `ent.Or(ent, big.NewInt(int64(w)))` (`bip39.go:152-154`) and the live `mnemonic[selected]==-1`. ✓
- I3 memoized recompute (lines 99-100,106): keyed on `selected==len-1`, computed on first observation, recomputed on `selected`/earlier-word change, never per-frame; correct vs hook-less loop (`gui.go:570-578`, Edit re-entry `gui.go:1955`). 8-vs-128 doesn't change this. ✓
- §9 resolved-decisions line consistent with "both lengths, 8 and 128." ✓

### 3. Fresh-eyes whole-document re-scan — no new Critical/Important
- Change 3 (Button3) anchors exact: keyboard filter `ButtonFilter(Button3)` (`gui.go:952`) + commit `case Center, Button3:` (`gui.go:1009-1011`); `okBtn` Button2 (`gui.go:543`); both Button2-accept test sites named (`gui_test.go:281`, `codex32_input_test.go:31`/:29); no other keyboard Button3 consumer. ✓
- `TestWordKeyboardScreen` edge: `make(Mnemonic,1)`, `selected==0==len-1` → per-frame rule fires but `LastWordCandidates` returns nil on `len%3==1` → normal keyboard; passes once Button2→Button3 lands (line 118). ✓
- Change 1/2 anchors: `layoutTitlef` (`gui.go:1520`); match-count guarded `len(frag)>0` + centering caveat (`gui.go:548,589-602`). ✓
- Testability: all listed tests use `runes`/`click`/`press` + `ExtractText`/`uiContains` + `synctest`; 12-word fixture derived from `LastWordCandidates`, asserts a non-candidate full word does NOT light accept (I2 regression test); 24-word asserts `8 matches`→`1 match`. No coordinate-tap helper needed. ✓
- Backstop preserved: `SeedScreen.Confirm` `mnemonic.Valid()` (`gui.go:1973`) retained (lines 104,110); scope narrowed to "last-word-typo class only." ✓
- No internal contradictions remaining.

**Minor (non-blocking, FYI — do NOT loop):** line 77 "first N−1 words" reads as first 23/11; doc-comment (line 87 "final slot ignored") already consistent. Helper loop `m[last]=w` over `0..NumWords-1`: `NumWords` typed `Word` (`bip39.go:28`) so `w` is `Word` — trivially correct. Informational only; below the Important bar.

The spec is internally consistent, implementable against the verified code anchors, and testable with the existing host harness. **GREEN — proceed to writing-plans.**

---

## Loop summary
| Round | Verdict | Findings |
|---|---|---|
| R0 | NOT-GREEN (1C/4I) | C1 missed codex32 test; I1 guard/clone; I2 candidate-scoped completion; I3 memoized recompute; I4 24-word OK-lighting; minors |
| R1 | NOT-GREEN (1C/0I) | NEW C1: 24-word last word = 8 candidates, not 1 (latent R0 error). All R0 findings verified folded. |
| **R2** | **GREEN (0C/0I)** | C1 folded in all 5 sites (8/128), independently re-derived; unification composes (I1/I2/I3/I4 coherent); §9 consistent; fresh re-scan clean. **Cleared to writing-plans.** |
