# SPEC — SeedHammer on-device input UX, Slice 1: BIP-39 seed-word entry polish

**Date:** 2026-06-17
**Target repo:** the SeedHammer II firmware fork `bg002h/seedhammer` (Go/TinyGo, RP2350, 480×320 touchscreen + 3 side buttons + D-pad). Public domain (Unlicense) — contributions stay public domain.
**Motivation:** upstream PR #34 was closed for "input UI not polished enough." This is the first of three sequenced slices that raise on-device secret-input polish. Slice 1 is the low-risk, no-security-surface, fully host-testable, *generic* one — it improves BIP-39 seed-word entry for every SeedHammer user.
**Feeds:** the writing-plans implementation plan, after passing the opus-architect R0 gate (0C/0I).
**Predecessors:** `design/RECON_seedhammer_input_ux.md` (4-agent investigation) and `design/agent-reports/seedhammer-input-ux-design-review-R0.md` (pre-spec architect review; this spec incorporates its Slice-1 findings).

---

## 1. Goal

Make the existing on-device BIP-39 word-entry flow feel polished and self-explanatory, via four additive changes:
1. a per-word progress indicator,
2. a remaining-match count,
3. consistent primary-button behavior (accept on Button3, matching every other screen),
4. last-word checksum assistance for both 12- and 24-word seeds.

Ships as **one focused, signed + DCO commit/PR, rebased on `upstream/main`**.

## 2. Scope

**In scope (Slice 1):** the four changes above, in `gui/gui.go` + one new pure-Go helper in `bip39/bip39.go`. The Button3 change (change 3) touches all three input flows (`inputWordsFlow`, `inputCodex32Flow`, `inputSLIP39Flow`) because the keyboard widget is shared.

**Out of scope (explicitly deferred):**
- **Tap-the-predicted-word-box to accept** — deferred. The accept *nav button* is already touch-tappable (`layoutNavigation` wires every nav button via `op.Input(...).Clip(...)`, `gui/gui.go:1551`), so there is no touch UX gap. Adding tap-the-box would require a new touch target *and* a new pointer-coordinate test helper (the host harness has no coordinate-tap injection). When pursued (a later slice), lead with a small standalone PR that adds the coordinate-tap test helper (it also backfills coverage for the keyboard keys, word-review boxes, and nav buttons).
- **In-flow "back one word" editing** — the review screen's Edit button (`SeedScreen.Confirm`) already lets the user fix any word.
- **Passphrase entry** — Slices 2 (keyboard widget) and 3 (flow + verification).
- **CODEX32 / SLIP-39 input polish** beyond the shared Button3 change.

## 3. Background — the existing flow (anchors)

- `inputWordsFlow` (`gui/gui.go:539`) — the per-word keyboard screen. `selected` tracks the current word index; `len(mnemonic)` is 12 or 24. The current word is rendered as `"%2d: <fragment-or-word>"` in a box (`selected+1`, ~`gui.go:595`), and the screen title is set via `layoutTitle` (~`gui.go:612`).
- `NewKeyboard` (`gui/gui.go:790`) builds the keyboard from `wordKeys = "qwertyuiop\nasdfghjkl\nzxcvbnm"` (`gui.go:537`). `Keyboard.Update` (`gui.go:939`) handles touch + D-pad; it currently binds **Button3** as a synonym for Center to commit the focused key: filter `ButtonFilter(Button3)` (`gui.go:952`) and handler `case Center, Button3: k.rune(); return true` (`gui.go:1009-1011`).
- `updateValidBIP39Keys(frag, keys)` (`gui.go:869-893`) binary-searches the wordlist, returns `nvalid` (count of words matching the current prefix), and disables impossible next-letters by setting a 32-bit mask (`key.r - 'a'`, `gui.go:884,921-930`); `Keyboard.Valid()` (`gui.go:932`) reports whether a key is enabled.
- `completeBIP39Word(frag, nvalid)` (`gui.go:860`) returns the completed word + `true` when `nvalid==1` or the fragment is already an exact word label.
- Accept is `okBtn := &Clickable{Button: Button2}` (`gui.go:543`), rendered `StylePrimary` (`gui.go:606`) — the **only** screen whose primary action is Button2.
- `SeedScreen.Confirm` (`gui.go:1919`) reviews the words and validates the checksum via `mnemonic.Valid()` (`gui.go:1973`); invalid → error screen.
- `bip39` primitives: `Mnemonic.Valid()` (`bip39/bip39.go:107`), `ChecksumWord` (`:182-186`), `checksum()` (`:175`), `ClosestWord` (`:95`), `LabelFor` (`:79`), `splitMnemonic`.

## 4. Design

### 4.1 Change 1 — Per-word progress title

**Behavior:** the title shows `Word N of 24` (e.g. `Word 7 of 24`), replacing the static "Input Words".
**Where:** `inputWordsFlow` title render (~`gui.go:612`). `selected` and `len(mnemonic)` are already in scope.
**Implementation:** use the existing `layoutTitlef(ctx, dims.X, th.Text, "Word %d of %d", selected+1, len(mnemonic))` (`gui.go:1520`) — do not `fmt.Sprintf` into `layoutTitle`. Pure render change; no logic/state change.
**Edge cases:** none — `selected` is always a valid `0..len-1` index inside the flow.

### 4.2 Change 2 — Remaining-match count

**Behavior:** while typing a word, show the count of still-matching BIP-39 words (e.g. `12 matches`) near the fragment box. Shown only once ≥1 character is typed (avoid a meaningless "2048 matches" on an empty fragment). When the word is complete (`nvalid==1`), the count reads `1 match`.
**Where:** `inputWordsFlow` render path; data source is `nvalid` from `updateValidBIP39Keys` (`gui.go:869`), already computed each keystroke and in scope (`gui.go:552`).
**Implementation:** render a small label (reuse an existing text style) adjacent to the predicted-word box; ensure it does not disturb the `longest`/`widestWord`-based centering of the word box (`gui.go:548,589-602`). Guard: `if len(kbd.Fragment) > 0 { show "<nvalid> match[es]" }`. Pluralize ("1 match" vs "N matches"). In the 12-word last-word state the count reflects the **candidate-scoped** `nvalid` (§4.4), not the full-wordlist count.
**Edge cases:** empty fragment → no count. This is presentation only; it does not alter the existing key-dimming.

### 4.3 Change 3 — Button3 primary-action consistency

**Problem (architect C1):** the keyboard widget itself consumes Button3 (`case Center, Button3` at `gui.go:1009`, filter `:952`), and `kbd.Update` runs every frame, so a nav button bound to Button3 would never see the event. Moving accept to Button3 is therefore *not* a render-only tweak — it requires freeing Button3 from the keyboard.

**Behavior:**
- The keyboard commits the focused key on **Center only** (the D-pad center). Button3 no longer types a key.
- The word-accept nav button moves from **Button2 → Button3** (`StylePrimary`), matching `ChoiceScreen`, `ErrorScreen`, `SeedScreen.Confirm`, etc.
- Back stays on **Button1**.

**Where (all three flows, since the keyboard is shared):**
- Keyboard: remove `Button3` from the filter (`gui.go:952`) and from the commit `case` (`gui.go:1009-1011`) so it reads `case Center:`.
- `inputWordsFlow`: change `okBtn := &Clickable{Button: Button2}` (`gui.go:543`) to `Button3`.
- `inputCodex32Flow` (`gui.go:623`) and `inputSLIP39Flow` (`gui.go:684`): move their OK button to Button3 the same way, so all three are consistent.

**Implementation notes:** verify no other code path relies on Button3-types-key. The change is uniform across the three flows. The PR description must explicitly call out that Button3 no longer commits a key on the keyboard (Center still does) — it removes a convenience binding.

**Edge cases:** the D-pad Center commit path is unchanged and remains the keyboard's commit affordance for button-only navigation. Note `inputSLIP39Flow` is currently dead code (called only from commented-out `gui.go:1894`), so its Button3 change is correct-by-inspection — no test exercises it.

### 4.4 Change 4 — Last-word checksum assistance (both seed lengths)

**Math (verified, architect I3):** for the final word, given the first N−1 words: a **24-word** seed has exactly **1** valid last word (3 entropy bits fixed by prior words + 8 checksum bits fixed by the entropy); a **12-word** seed has **128** valid last words (7 free entropy bits + 4 checksum bits).

**New helper — `bip39.LastWordCandidates`:**
```go
// LastWordCandidates returns every word that, placed in the final slot,
// yields a checksum-valid mnemonic given the already-filled earlier words.
// It operates on a copy and does NOT mutate prefix. It returns nil if
// len(prefix)%3 != 0 (matching Valid's own guard) or if ANY of the first
// len(prefix)-1 words is unset (< 0) or out of range (>= NumWords).
// Otherwise it returns 1 candidate for a 24-word mnemonic and 128 for a
// 12-word one. The final slot of prefix is ignored.
func LastWordCandidates(prefix Mnemonic) []Word
```
**Implementation (order is load-bearing):**
1. **Guard first, before any `Valid()` call.** Return nil if `len(prefix)%3 != 0`; scan `prefix[:len(prefix)-1]` and return nil if any word is `< 0` or `>= NumWords`. This ordering is mandatory: `Valid()`→`splitMnemonic` (`bip39/bip39.go:149-154`) does `ent.Or(ent, big.NewInt(int64(w)))` per word, so a `-1` word produces garbage/negative entropy rather than a clean rejection — brute-forcing on `Valid()` alone would be nondeterministic on a `-1`-bearing prefix.
2. **Clone:** `m := slices.Clone(prefix)` — never mutate the caller's live `mnemonic` (whose last slot is `-1` and must stay `-1` until accept).
3. For `w` in `0..NumWords-1`, set `m[last]=w` and collect `w` where `m.Valid()`. O(NumWords) SHA-256 — trivially fast.

Place next to `ChecksumWord`/`checksum` (`bip39/bip39.go:120-186`). Pure Go; unit-tested independently (no GUI). (An optimized bit-twiddling version is possible but unnecessary.)

**UI integration in `inputWordsFlow`** (`gui.go:539`). The flow has **no "entered a word" hook** — `selected` advances inside the accept loop (`gui.go:570-578`), a single invocation from `newInputFlow` (`gui.go:1882`, `selected=0`) walks 0→last in one call, and the review-screen Edit re-enters via `inputWordsFlow(..., s.selected)` (`gui.go:1955`). So drive the last-word logic off the **current frame state**, not an entry event:

- **Per-frame rule:** if `selected == len(mnemonic)-1`, use the candidate path; else the normal full-wordlist path.
- **Memoize `cands`:** compute `cands := bip39.LastWordCandidates(mnemonic)` when the flow first observes `selected == len(mnemonic)-1`, and recompute only when `selected` changes or an earlier word was edited — cache it; do **not** run the `NumWords`-`Valid()` loop every frame. If `cands == nil` (an earlier slot is unset), fall back to the normal full keyboard.
- **24-word (`len(cands)==1`):** on entering the last-word state, **pre-seed** `kbd.Fragment = bip39.LabelFor(cands[0])` and mark it complete, so the OK nav button (Button3) lights immediately with no typing — the user sees the word and presses accept. Backspace works via the existing `'⌫'` handling (`gui.go:1032-1034`); if the user backspaces/diverges from the candidate, **revert that slot to the normal full-wordlist** `updateValidBIP39Keys`/`completeBIP39Word` path (a wrong final word is then caught by the Confirm backstop). Do **not** silently auto-commit; require the accept press.
- **12-word (`len(cands)==128`):** use a **candidate-scoped** completion+mask path — do **not** reuse the unmodified `completeBIP39Word`/`updateValidBIP39Keys`, because `completeBIP39Word`'s exact-full-label clause (`gui.go:866`) would otherwise accept a real BIP-39 word that is **not** one of the 128 candidates → checksum-invalid, defeating Change 4. Define a small helper that, given the candidate set (`[]Word`) and the current fragment: (a) builds the valid-key 32-bit mask by OR-clearing, for each candidate `w` whose `bip39.LabelFor(w)` has the fragment as a prefix, the bit for the next letter `LabelFor(w)[len(frag)]` (same mask mechanism as `updateValidKeys`, `gui.go:921-930`); (b) computes `nvalid` as the count of candidates still matching the fragment; (c) reports complete when `nvalid==1` **or** the fragment exactly equals a *candidate's* label (a candidate, not any full-wordlist label). Match-count (Change 2) shows this candidate-scoped `nvalid`.
- **Words 1..N−1:** unchanged — normal full-wordlist `updateValidBIP39Keys` + `completeBIP39Word`.

**Backstop (architect I3):** this does **not** remove the `SeedScreen.Confirm` `mnemonic.Valid()` check (`gui.go:1973`). A transcription error in words 1..N−1 still produces an invalid checksum caught there. Change 4 eliminates the last-word-typo error class only.

**Edge cases:** reaching the last word with an earlier slot unset (e.g. Edit on a partial mnemonic) → `LastWordCandidates` returns nil → normal full keyboard. **Invalidate the `cands` cache** whenever `selected` changes or an earlier word is edited (the memoization rule above). The 24-word pre-seeded word is backspaceable via the existing `'⌫'` handling, and diverging from it reverts the slot to the normal path (per the 24-word bullet).

## 5. Error handling

No new error surfaces. The single backstop is the existing `mnemonic.Valid()` gate in `SeedScreen.Confirm` (`gui.go:1973`), which stays. `LastWordCandidates` returns nil (not an error) on an incomplete/unsupported prefix; callers fall back to normal behavior.

## 6. Testing strategy

All Slice-1 behavior is exercisable by the existing host harness (`go test ./gui/... ./bip39/...`): `runes`/`click`/`press` drive entry (`gui/event_test.go`), `ExtractText` (`gui_test.go:470`) asserts on-screen text, `synctest` (`gui_test.go:158`) gives deterministic time. Required tests:
- **`bip39.LastWordCandidates`** (pure unit): a known 23-word prefix → `len==1` and the result completes a `Valid()` 24-word mnemonic; a known 11-word prefix → `len==128`, every candidate completes a `Valid()` 12-word mnemonic; an incomplete prefix → `nil`.
- **Progress title:** drive partial entry, assert `ExtractText` contains `Word 7 of 24` (case-insensitive `uiContains` is fine).
- **Match count:** type a prefix, assert the expected `N matches` text; assert nothing shown on empty fragment.
- **Button3 accept:** drive accept via `click(Button3)`; assert the word is committed and `selected` advances. Assert the keyboard still commits the focused key via Center. **Update both Button2-accept test sites** (the only two in the tree): `TestWordKeyboardScreen` (`gui_test.go:281`) and `TestInputSeedCodex32` (`gui/codex32_input_test.go:31`, plus its comments at :29) — both must switch `click(Button2)` → `click(Button3)`, or they hang to timeout.
- **24-word last word:** enter 23 valid words, reach the last word, assert the single candidate is shown and accept finishes a valid seed.
- **12-word last word:** derive the fixture from `LastWordCandidates` — take a known 11-word valid prefix, compute its 128 candidates, then assert a specific letter that begins **none** of the candidates (at the empty fragment) is disabled while a candidate-prefix letter is enabled, the candidate-scoped count is shown, and a completed last word yields `Valid()`. Also assert a full-wordlist word that is **not** a candidate does not light accept (closes the I2 hole).

No coordinate-tap helper is needed in Slice 1 (tap-the-box deferred). Manual on-device QA is a nice-to-have but not required for merge.

## 7. Versioning

The firmware version is injected at build time via `-ldflags '-X main.Version=...'` (`cmd/controller/main.go:14`) — there is no committed version constant to bump in this slice.

## 8. Upstream-PR plan

One focused PR against `seedhammer/seedhammer`, branched off current `upstream/main`, commits signed + DCO (`Signed-off-by`), author Brian Goss. The PR description:
- frames it as generic BIP-39 entry polish (progress, match count, primary-button consistency, last-word checksum help),
- explicitly notes the Button3 binding change (Center still commits the focused key),
- references that it directly addresses the input-polish concern raised on PR #34.

## 9. Resolved decisions / no open questions

- Scope decomposed into 3 slices; this is Slice 1 (user-approved).
- 12-word last word: **filter the keyboard mask to the 128 valid words** (user-approved "both lengths").
- Tap-to-accept: **deferred** (architect-recommended; no touch UX gap).
- Fingerprint-engraving choice: **Slice 3** concern, not here.
