# SeedHammer input Slice 1 (BIP-39 word-entry polish) — SPEC review — R0

- **Stage:** spec R0 gate (mandatory; must reach 0C/0I before writing-plans).
- **Date:** 2026-06-17
- **Reviewer:** opus `feature-dev:code-architect` (read-only; verified against `gui/gui.go`, `gui/widget.go`, `gui/event.go`, `bip39/bip39.go`, and the three `*_test.go` harness files).
- **Spec:** `design/SPEC_seedhammer_input_slice1.md` (committed `05c2c82`).
- **Verdict:** **NOT-GREEN — 1 Critical / 4 Important.** Four changes well-targeted; prior design-review Slice-1 findings mostly folded correctly; gaps are concrete under-specifications a fresh engineer would hit.

---

## VERBATIM REVIEW OUTPUT

**VERDICT: NOT-GREEN — 1 Critical / 4 Important.**

### CRITICAL

**C1. Change 3 breaks `TestInputSeedCodex32`, which the spec does not list for update.** §4.3 moves `inputCodex32Flow`'s OK to Button3; §6 lists only `TestWordKeyboardScreen` (`gui_test.go:281`). But `TestInputSeedCodex32` confirms with `click(&ctx.Router, Button2)` (`gui/codex32_input_test.go:31`, comment :29). After change 3, codex32 accepts only on Button3 → this test hangs to timeout (exactly the wrong-button case its header comment describes). There are exactly two Button2-accept sites in the test tree: `gui_test.go:281` and `codex32_input_test.go:31`; the spec names only the first. **Fix:** add `TestInputSeedCodex32` (`codex32_input_test.go:31` + comments :29) to §6's Button2→Button3 update list.

### IMPORTANT

**I1. `LastWordCandidates` must guard unset (`-1`) words BEFORE calling `m.Valid()`.** `Valid()`→`splitMnemonic` (`bip39.go:149-154`) does `ent.Or(ent, big.NewInt(int64(w)))` per word; a `-1` word yields negative/garbage entropy, not a clean rejection — so brute-forcing on `Valid()` alone is nondeterministic on a `-1`-bearing prefix. **Fix:** first action — reject `len(prefix)%3 != 0`; scan `prefix[:len-1]`, return nil if any word `< 0` or `>= NumWords`; THEN brute force. Operate on a clone (`slices.Clone`) so the caller's live `mnemonic` (last slot `-1`) is not mutated.

**I2. The 12-word mask-restriction does not compose with `completeBIP39Word`/`nvalid` as specified.** The mask mechanism (`updateValidKeys`, `gui.go:921-930`) is candidate-agnostic and works, BUT: in `inputWordsFlow` the accept gate + label come from `completeBIP39Word(frag, nvalid)` (`gui.go:554,562,605`), which completes when `nvalid==1` OR `frag == LabelFor(w)` for a full-wordlist word (`gui.go:866`). For the 12-word last word: (a) `nvalid` must be the candidate-scoped count (else Change 2's count is wrong and a fragment unique among the 128 but not in the full 2048 never lights accept); (b) the exact-full-label clause would accept a real BIP-39 word that is NOT one of the 128 candidates → checksum-invalid, defeating Change 4. The spec's "any word the user completes is checksum-valid" is NOT guaranteed by the mask alone. **Fix:** define a candidate-scoped helper `(completedWord, nvalid, complete)` over the candidate `[]Word` (via `LabelFor(w)` prefixes); gate complete on `nvalid==1` OR fragment == a *candidate* label. Do not reuse unmodified `completeBIP39Word`/`updateValidBIP39Keys` for this slot.

**I3. "Compute `cands` once on entering the last word" is unimplementable against `inputWordsFlow`'s loop.** There is no "entered a word" hook: `selected` is mutated inside the accept loop (`gui.go:570-578`); a single invocation from `newInputFlow` (`gui.go:1882`, selected=0) walks 0→last in one call; Edit re-enters via `inputWordsFlow(...,s.selected)` (`gui.go:1955`). **Fix:** key the last-word logic off `selected == len(mnemonic)-1` per frame; memoize `cands` (compute on first observing that state / when `selected` or an earlier word changes), NOT every frame (avoid an O(2048-SHA256) loop in the render path). `nil` cands → normal path.

**I4. The 24-word "no typing, just accept" path conflicts with the accept gate + backspace requirement.** The OK button only shows when `completeBIP39Word(frag,nvalid)` completes (`gui.go:605-606`); empty fragment → no OK button. And "backspaceable" needs something in `kbd.Fragment`. **Fix (option a):** on entering the 24-word last-word state, pre-seed `kbd.Fragment = LabelFor(cands[0])` and mark complete so OK lights; existing `'⌫'` handling (`gui.go:1032-1034`) makes it deletable; once the user diverges, revert that slot to the normal full-wordlist path (wrong final word caught by the Confirm backstop). State the mechanism explicitly.

### MINOR

**M1.** Use `layoutTitlef(ctx, dims.X, th.Text, "Word %d of %d", selected+1, len(mnemonic))` (`gui.go:1520`), not `fmt.Sprintf` into `layoutTitle` (`gui.go:1516`).
**M2.** The title test passes robustly because `uiContains` (`gui_test.go:478-482`) strips spaces from the needle and `ExtractText` concatenates glyphs without spaces → both become `"word7of24"`. No change; the space-stripping (not just case-insensitivity) is what makes it robust.
**M3.** `inputSLIP39Flow` (`gui.go:684`) is dead code (called only from commented-out `gui.go:1894`); its Button3 change is correct-by-inspection — no test exercises it. Note in the spec.
**M4.** Ensure the new match-count label does not disturb the `longest`/`widestWord`-based word-box centering (`gui.go:548,589-602`).

### Folding of the prior design review
- **C1 (Button3 collision):** correctly folded; event-model verified (`EventRouter.Next` matches/dequeues `events[0]`, `event.go:266-279`; with Button3 off the keyboard filter, the press stays queued for `okBtn.Clicked`). No other Button3 consumers in the keyboard path (only `gui.go:952/:1009`). Gap = the missed test (C1 above).
- **I3 prior (12-word=128, Valid() backstop, narrowed over-claim):** math correct; backstop retained (`gui.go:1973`); caveat correctly narrowed to "last-word-typo class only." But the 128-case *integration* is under-specified (I2 above) — exactly the prior "integration needs care" warning.
- **I4 prior (empty-fragment guard):** correctly folded (§4.2).
- **Tap-to-accept deferral (C2 prior):** correctly folded + justified (nav button already touch-tappable via `layoutNavigation` `op.Input(...).Clip(...)`; harness has no coordinate-tap injection — confirmed only `click`/`press`/`runes` in `event_test.go:42-76`). Consequence: the 24-word "no typing" path must light the OK *nav button* via the button path → I4 mechanism must be pinned.

### Testability
Listed tests drivable by the existing harness (`runes`/`click`/`press`, `ExtractText`+`uiContains`, `synctest`) with no coordinate-tap helper; `LastWordCandidates` is a pure unit test. Miss = `TestInputSeedCodex32` (C1). The 12-word last-word test needs a concrete named fixture (a known 11-word prefix + a specific letter absent from all 128 candidates) or it's hand-wavy.

### To reach GREEN
Fold C1 + resolve I1–I4 (prefix-guard ordering + non-mutation; candidate-scoped completion/count helper closing the exact-label hole; per-frame-with-memoization recompute keyed on `selected==len-1`; explicit 24-word fragment-preseed). The four user-approved decisions (3-slice split, both-lengths last word, tap deferral, fingerprint→Slice 3) are correctly reflected and need no rework.

---

## Loop summary (added by main session)
| Round | Verdict | Findings |
|---|---|---|
| R0 | NOT-GREEN (1C/4I) | C1 missed `TestInputSeedCodex32` Button2→Button3; I1 `LastWordCandidates` guard-ordering + non-mutation; I2 candidate-scoped completion/count (mask alone doesn't guarantee checksum-valid completion); I3 recompute keyed on `selected==len-1` + memoize; I4 24-word fragment-preseed to light OK. Minors M1–M4. |
