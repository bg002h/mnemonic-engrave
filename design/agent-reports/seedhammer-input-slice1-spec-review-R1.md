# SeedHammer input Slice 1 (BIP-39 word-entry polish) — SPEC review — R1

- **Stage:** spec R1 convergence gate (verify R0's 1C/4I folded; scan for new C/I).
- **Date:** 2026-06-17
- **Reviewer:** opus `feature-dev:code-architect` (read-only; verified against `gui/gui.go`, `bip39/bip39.go`, `bip39/wordlist.go`, `gui/gui_test.go`, `gui/codex32_input_test.go`, `gui/event_test.go`).
- **Verdict:** **NOT-GREEN — 1 Critical / 0 Important.** R0's C1 + I1–I4 + all minors folded correctly, BUT the amendment hard-coded a latent factual error: the 24-word last-word candidate count is **8, not 1**. (Folded in R2; see the R2 review.)

---

## VERBATIM REVIEW OUTPUT

**VERDICT: NOT-GREEN — 1 Critical / 0 Important.**

The four R0 Importants (I1–I4) and Critical C1 are folded correctly, and all minors are addressed. However, the amendment carried forward — and now load-bears on — a factual error latent in R0: the 24-word last-word candidate count. R0 asserted "1"; the amended spec hard-codes that into the `LastWordCandidates` contract, the I4 pre-seed mechanism, and the §6 test. The correct count is **8**. This makes the 24-word path unimplementable as written and its test false.

### CRITICAL

**C1 (NEW). The 24-word last-word count is wrong: there are 8 valid last words, not 1 — and the spec now hard-codes "1" into the helper contract, the pre-seed UX, and the test.** §4.4 states "a 24-word seed has exactly 1 valid last word (3 entropy bits fixed by prior words + 8 checksum bits)." Incorrect. The final word holds 11 bits = 3 entropy + 8 checksum (`checkBits = len(m)/3 = 8` for 24 words, `bip39/bip39.go:159`). Those 3 entropy bits are NOT fixed by the prior 23 words — they live in the last word. As they range over their 8 values, `ent` changes, so the SHA-256 checksum (`ChecksumWord`, `bip39.go:182-186`) changes; for each of the 8 entropy-bit choices there is exactly one matching valid word → **exactly 8 distinct valid last words** for 24-word (the wordlist has no duplicate labels). 12-word (**128**) is correct. Three normative parts depend on `len(cands)==1`: the helper contract ("returns 1 candidate"), the I4 pre-seed (would silently pick one of 8 valid words and hide the other 7 → nudges the user toward a *different valid seed/wallet*, a footgun), and the §6 test ("assert the single candidate is shown" — unwritable). **Fix:** correct to 8 everywhere; redefine the 24-word path to mirror the 12-word candidate-scoped path (drop the single-candidate pre-seed); fix the unit-test bullet `len==1`→`len==8`.

### Verification of R0 findings (all folded correctly)
- **C1 (R0) Button2→Button3 tests:** RESOLVED — §6 lists both `TestWordKeyboardScreen` (`gui_test.go:281`) and `TestInputSeedCodex32` (`codex32_input_test.go:31`+`:29`); verified these are the only two Button2-accept sites (the other Button2 hits are an event-ordering fixture `event_test.go:16` and `SeedScreen.Confirm`'s `editBtn` `gui.go:1922`, both unaffected).
- **I1 (R0) guard + non-mutation:** RESOLVED — §4.4 step 1 mandates the guard before any `Valid()` call and clone via `slices.Clone`; correct against `splitMnemonic`'s `big.Int.Or` (`bip39.go:152-154`) where `-1` pollutes entropy.
- **I2 (R0) candidate-scoped completion/count/mask:** RESOLVED and realizable — mask via `LabelFor(w)[len(frag)]` OR-clear (compatible with `updateValidKeys`, `gui.go:921-930`), `nvalid` over candidates, complete only on a *candidate* label — closes the `completeBIP39Word` exact-full-label hole (`gui.go:866`).
- **I3 (R0) per-frame recompute + memoize:** RESOLVED — keyed on `selected==len-1`, computed on first observation, recomputed on `selected`/earlier-word change, no per-frame 2048-`Valid()` loop; correct against the no-per-word-hook structure (`gui.go:570-578`, Edit re-entry `gui.go:1955`).
- **I4 (R0) light OK without pre-seed mechanism premise:** the mechanism is realizable but its premise (`len(cands)==1`) is false → folds into C1 (drop pre-seed; the shared candidate-scoped path lights OK via `complete`, satisfying the no-coordinate-tap-helper concern).

### Minors — all addressed
M1 `layoutTitlef` (`gui.go:1520`) ✓; M2 title-test robustness via `uiContains` space-strip ✓; M3 `inputSLIP39Flow` dead-code note (`gui.go:1894`) ✓; M4 match-count must not disturb `longest`/`widestWord` centering (`gui.go:548,589-602`) ✓; 12-word test fixture derived from `LastWordCandidates` ✓.

### New-issue scan (beyond C1)
- Per-frame rule vs pre-seed: the only contradiction is the one in C1 (pre-seed presumes 1 candidate); removing the pre-seed clears it.
- Cache invalidation correct (selected mutated only in the accept loop; Edit re-invokes fresh).
- Candidate helper vs `completeBIP39Word` for non-last words: no interaction (gated behind `selected==len-1 && cands!=nil`).
- `TestWordKeyboardScreen` (`make(Mnemonic,1)`, `gui_test.go:282`): with `selected==0,len==1`, the per-frame rule fires but `LastWordCandidates` returns nil (`len%3==1`) → normal path; the I1 guard protects this existing test. (Still needs the Button2→Button3 update.)

### To reach GREEN
Correct the 24-word count 1→8 (§4.4 math, helper contract, §6 24-word test, unit-test bullet) and replace the 24-word single-candidate pre-seed with the shared candidate-scoped path. No other findings block.

Sources: BIP-39 8-valid-last-words result (Blockplate "The Special Last Word of a Seed Phrase"; bip39-checksum-words-finder).

---

## Loop summary (added by main session)
| Round | Verdict | Findings |
|---|---|---|
| R0 | NOT-GREEN (1C/4I) | C1 missed codex32 test; I1 guard/clone; I2 candidate-scoped completion; I3 memoized recompute; I4 24-word OK-lighting; minors |
| R1 | NOT-GREEN (1C/0I) | **NEW C1: 24-word last word = 8 candidates, not 1** (latent error from R0, hard-coded by the R0 fold). All R0 findings verified folded. → corrected: 24-word now uses the shared candidate-scoped path (8 candidates); no pre-seed/special-case. |
