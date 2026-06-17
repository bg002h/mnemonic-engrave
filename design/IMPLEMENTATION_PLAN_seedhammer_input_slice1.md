# SeedHammer input-UX Slice 1 (BIP-39 seed-word entry polish) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Polish the SeedHammer II on-device BIP-39 word-entry flow — per-word progress, a remaining-match count, primary-button (Button3) consistency, and last-word checksum assistance — as one focused, signed+DCO upstream PR.

**Architecture:** Minimal additive changes in `gui/gui.go` plus one new pure-Go helper in `bip39/bip39.go`. No refactor. The last-word assist adds two small `gui`-package helpers (`updateValidCandidateKeys`, `completeCandidateWord`) that mirror the existing `updateValidBIP39Keys`/`completeBIP39Word` but operate over a precomputed candidate set; `inputWordsFlow` routes to them when on the final word. TDD on host via `go test ./gui/... ./bip39/...`.

**Tech Stack:** Go (host build/test; firmware is TinyGo but `gui`/`bip39` are host-testable), the SeedHammer `op`/`widget`/`layout` immediate-mode GUI, the `bip39` package.

**Spec:** `mnemonic-engrave/design/SPEC_seedhammer_input_slice1.md` (GREEN, R0→R2).

**Repo & case conventions (read before starting):**
- Work in the seedhammer fork `/scratch/code/shibboleth/seedhammer`.
- `bip39.LabelFor` returns **UPPERCASE** word labels (e.g. `"ABANDON"`); the keyboard's `keyboardKey.r` is lowercase; `Keyboard.rune()` appends `unicode.ToUpper(r)`, so `kbd.Fragment` is **uppercase**. Mask code bridges case with `unicode.ToLower(rune(label[i])) - 'a'` (see `gui/gui.go:884`). New code must follow this exactly.
- `bip39/bip39.go` does **not** import `slices`; use `make`+`copy` (mirroring `FixChecksum`, `bip39/bip39.go:120-126`). `gui/gui.go` does **not** import `fmt`; use `widget.Labelf`/`layoutTitlef` for formatting (they take a format string + args).
- Commits must be **signed + DCO**, authored **Brian Goss**. The fork has `commit.gpgsign=true`; its `user.name` is currently `bg`, so set it (Task 0) before committing.

---

## File structure

| File | Responsibility | Change |
|------|----------------|--------|
| `bip39/bip39.go` | BIP-39 representation/validation | **Add** `LastWordCandidates` (pure helper) |
| `bip39/bip39_test.go` | bip39 unit tests | **Add** `TestLastWordCandidates` |
| `gui/gui.go` | on-device flows + keyboard widget | **Modify** `Keyboard.Update` (free Button3); `inputWordsFlow`/`inputCodex32Flow`/`inputSLIP39Flow` (OK→Button3); `inputWordsFlow` (progress title, match count, last-word path); **add** `updateValidCandidateKeys`, `completeCandidateWord` |
| `gui/gui_test.go` | gui tests | **Update** `TestWordKeyboardScreen` (Button2→Button3); **add** `TestWordFlowProgressTitle`, `TestWordFlowMatchCount`, `TestUpdateValidCandidateKeys`, `TestCompleteCandidateWord`, `TestWordFlowLastWord24` |
| `gui/codex32_input_test.go` | codex32 entry test | **Update** `TestInputSeedCodex32` (Button2→Button3) |

---

## Task 0: Branch + commit identity

**Files:** none (git setup).

- [ ] **Step 1: Fetch upstream and branch off pristine `upstream/main`**

The PR must be a clean Slice-1-only diff, so branch off `upstream/main` (NOT the fork's `main`, which carries unrelated md1/mk1+codex32 merges).

Run:
```bash
cd /scratch/code/shibboleth/seedhammer
git fetch -q upstream
git checkout -b feat/bip39-entry-polish upstream/main
```
Expected: `Switched to a new branch 'feat/bip39-entry-polish'`.

- [ ] **Step 2: Set commit identity (signed + DCO author)**

Run:
```bash
git config user.name "Brian Goss"
git config user.email "goss.brian@gmail.com"
git config commit.gpgsign   # expect: true
```
Commit with `-s` (DCO sign-off) on every task.

- [ ] **Step 3: Verify a clean baseline**

Run:
```bash
go test ./gui/... ./bip39/...
```
Expected: `ok` for all packages (baseline green before changes).

---

## Task 1: `bip39.LastWordCandidates`

**Files:**
- Modify: `bip39/bip39.go` (add func near `FixChecksum`, ~:120-126)
- Test: `bip39/bip39_test.go`

- [ ] **Step 1: Write the failing test**

Add to `bip39/bip39_test.go`:
```go
func TestLastWordCandidates(t *testing.T) {
	build := func(n int) Mnemonic {
		m := make(Mnemonic, n)
		for i := range m {
			m[i] = Word(i % int(NumWords))
		}
		return m.FixChecksum() // checksum-valid
	}

	// 24-word: exactly 8 candidates, all valid, including the real last word.
	v24 := build(24)
	c24 := LastWordCandidates(v24)
	if len(c24) != 8 {
		t.Fatalf("24-word: got %d candidates, want 8", len(c24))
	}
	foundLast := false
	for _, w := range c24 {
		m := make(Mnemonic, len(v24))
		copy(m, v24)
		m[len(m)-1] = w
		if !m.Valid() {
			t.Errorf("24-word candidate %d is not checksum-valid", w)
		}
		if w == v24[len(v24)-1] {
			foundLast = true
		}
	}
	if !foundLast {
		t.Errorf("24-word candidates %v do not include the real last word %d", c24, v24[len(v24)-1])
	}

	// 12-word: exactly 128 candidates.
	v12 := build(12)
	if c12 := LastWordCandidates(v12); len(c12) != 128 {
		t.Fatalf("12-word: got %d candidates, want 128", len(c12))
	}

	// Incomplete prefix (an earlier word unset) -> nil.
	bad := make(Mnemonic, len(v24))
	copy(bad, v24)
	bad[5] = -1
	if got := LastWordCandidates(bad); got != nil {
		t.Errorf("incomplete prefix: got %v, want nil", got)
	}

	// Unsupported length (len%3 != 0) -> nil.
	if got := LastWordCandidates(make(Mnemonic, 13)); got != nil {
		t.Errorf("len 13: got %v, want nil", got)
	}

	// Must not mutate the input's final slot.
	before := v24[len(v24)-1]
	_ = LastWordCandidates(v24)
	if v24[len(v24)-1] != before {
		t.Errorf("LastWordCandidates mutated input final slot: %d -> %d", before, v24[len(v24)-1])
	}
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `go test ./bip39/ -run TestLastWordCandidates -v`
Expected: FAIL — `undefined: LastWordCandidates`.

- [ ] **Step 3: Write the implementation**

Add to `bip39/bip39.go` immediately after `FixChecksum` (after ~:126):
```go
// LastWordCandidates returns every word that, placed in the final slot,
// yields a checksum-valid mnemonic given the already-filled earlier words.
// It operates on a copy and does not mutate prefix. It returns nil if
// len(prefix)%3 != 0 or if any of the first len(prefix)-1 words is unset
// (< 0) or out of range (>= NumWords). Otherwise it returns 8 candidates
// for a 24-word mnemonic and 128 for a 12-word one. The final slot of
// prefix is ignored.
func LastWordCandidates(prefix Mnemonic) []Word {
	// Guard BEFORE any Valid()/splitMnemonic call: splitMnemonic ORs each
	// word into a big.Int, so a -1 word would corrupt the entropy rather
	// than be cleanly rejected.
	if len(prefix) == 0 || len(prefix)%3 != 0 {
		return nil
	}
	for _, w := range prefix[:len(prefix)-1] {
		if w < 0 || w >= NumWords {
			return nil
		}
	}
	m := make(Mnemonic, len(prefix))
	copy(m, prefix)
	var cands []Word
	for w := Word(0); w < NumWords; w++ {
		m[len(m)-1] = w
		if m.Valid() {
			cands = append(cands, w)
		}
	}
	return cands
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `go test ./bip39/ -run TestLastWordCandidates -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add bip39/bip39.go bip39/bip39_test.go
git commit -s -m "bip39: add LastWordCandidates (checksum-valid final words)"
```

---

## Task 2: Free Button3 from the keyboard; move word-accept to Button3

**Files:**
- Modify: `gui/gui.go` — `Keyboard.Update` (:952 filter, :1009 case), `inputWordsFlow` (:543), `inputCodex32Flow` (:626), `inputSLIP39Flow` (:688)
- Test: `gui/gui_test.go` (:281), `gui/codex32_input_test.go` (:31)

- [ ] **Step 1: Update the two existing tests to expect Button3 (the failing test)**

In `gui/gui_test.go`, `TestWordKeyboardScreen` (~:281), change:
```go
		click(&ctx.Router, Button2)
```
to:
```go
		click(&ctx.Router, Button3)
```

In `gui/codex32_input_test.go`, `TestInputSeedCodex32` (~:29-31), change the comment + button:
```go
	// Keypad: type the share, then confirm with Button3 (OK).
	runes(&ctx.Router, share)
	click(&ctx.Router, Button3)
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `go test ./gui/ -run 'TestWordKeyboardScreen|TestInputSeedCodex32' -v`
Expected: FAIL/hang-then-fail — the keyboard still commits the focused key on Button3 and the OK button is still Button2, so accept never fires on Button3. (If a test hangs, that confirms the wrong-button case; Ctrl-C and proceed — the code change in Step 3 fixes it.)

- [ ] **Step 3: Make the code changes**

In `gui/gui.go`, `Keyboard.Update` — remove `Button3` from the event filter (~:952). Change:
```go
		e, ok := k.inp.Next(ctx, ButtonFilter(Left), ButtonFilter(Right), ButtonFilter(Up), ButtonFilter(Down), ButtonFilter(Center), RuneFilter(), ButtonFilter(Button3))
```
to:
```go
		e, ok := k.inp.Next(ctx, ButtonFilter(Left), ButtonFilter(Right), ButtonFilter(Up), ButtonFilter(Down), ButtonFilter(Center), RuneFilter())
```

And remove `Button3` from the commit case (~:1009). Change:
```go
			case Center, Button3:
				k.rune()
				return true
```
to:
```go
			case Center:
				k.rune()
				return true
```

In `inputWordsFlow` (~:543), `inputCodex32Flow` (~:626), and `inputSLIP39Flow` (~:688), change each OK button from Button2 to Button3:
```go
	okBtn := &Clickable{Button: Button3}
```
(There are exactly three `okBtn := &Clickable{Button: Button2}` sites — one per flow. The back button stays `Button1`; do not touch it.)

- [ ] **Step 4: Run the full gui + bip39 suites**

Run: `go test ./gui/... ./bip39/...`
Expected: `ok` for all. `TestWordKeyboardScreen` and `TestInputSeedCodex32` now pass via Button3; the keyboard commits the focused key on Center only.

- [ ] **Step 5: Verify nothing else relied on keyboard-Button3 and run vet/gofmt**

Run:
```bash
grep -rn "Button3" gui/gui.go | grep -i keyboard   # expect: no keyboard-commit reference remains
go vet ./gui/... ./bip39/...
gofmt -l gui/gui.go gui/gui_test.go gui/codex32_input_test.go bip39/bip39.go bip39/bip39_test.go
```
Expected: vet clean (the pre-existing `gui/op/draw_test.go` go1.25/1.26 note, if shown, is unrelated); `gofmt -l` prints nothing.

- [ ] **Step 6: Commit**

```bash
git add gui/gui.go gui/gui_test.go gui/codex32_input_test.go
git commit -s -m "gui: make Button3 the primary accept across input flows

The keyboard commits the focused key on Center only; Button3 is freed for
the screen-level OK action, matching every other screen."
```

---

## Task 3: Per-word progress title

**Files:**
- Modify: `gui/gui.go` — `inputWordsFlow` title render (~:619, the `layoutTitle(... "Input Words")` call)
- Test: `gui/gui_test.go`

- [ ] **Step 1: Write the failing test**

Add to `gui/gui_test.go`:
```go
func TestWordFlowProgressTitle(t *testing.T) {
	ctx := NewContext(newPlatform())
	m := emptyBIP39Mnemonic(24)
	frame, quit := runUI(ctx, func() {
		inputWordsFlow(ctx, &descriptorTheme, m, 0)
	})
	defer quit()
	content, ok := frame()
	if !ok {
		t.Fatal("inputWordsFlow produced no frame")
	}
	if !uiContains(content, "Word 1 of 24") {
		t.Errorf("title missing %q; got %q", "Word 1 of 24", content)
	}
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `go test ./gui/ -run TestWordFlowProgressTitle -v`
Expected: FAIL — title reads "Input Words", not "Word 1 of 24".

- [ ] **Step 3: Make the code change**

In `gui/gui.go`, `inputWordsFlow`, change the title line (~:619):
```go
		title, _ := layoutTitle(ctx, dims.X, th.Text, "Input Words")
```
to:
```go
		title, _ := layoutTitlef(ctx, dims.X, th.Text, "Word %d of %d", selected+1, len(mnemonic))
```

- [ ] **Step 4: Run test to verify it passes**

Run: `go test ./gui/ -run TestWordFlowProgressTitle -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add gui/gui.go gui/gui_test.go
git commit -s -m "gui: show \"Word N of M\" progress in word-entry title"
```

---

## Task 4: Remaining-match count

**Files:**
- Modify: `gui/gui.go` — `inputWordsFlow` render (add a count label below the word box, ~:600-617)
- Test: `gui/gui_test.go`

- [ ] **Step 1: Write the failing test**

Add to `gui/gui_test.go`:
```go
func TestWordFlowMatchCount(t *testing.T) {
	ctx := NewContext(newPlatform())
	m := emptyBIP39Mnemonic(24)
	frame, quit := runUI(ctx, func() {
		inputWordsFlow(ctx, &descriptorTheme, m, 0)
	})
	defer quit()

	// Empty fragment: no match count shown.
	content, ok := frame()
	if !ok {
		t.Fatal("no frame")
	}
	if uiContains(content, "match") {
		t.Errorf("match count shown on empty fragment; got %q", content)
	}

	// Type a complete word (ABANDON) -> exactly one match.
	runes(&ctx.Router, "abandon")
	content, ok = frame()
	if !ok {
		t.Fatal("no frame after typing")
	}
	if !uiContains(content, "1 match") {
		t.Errorf("expected %q; got %q", "1 match", content)
	}
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `go test ./gui/ -run TestWordFlowMatchCount -v`
Expected: FAIL — no match-count label rendered.

- [ ] **Step 3: Make the code change**

In `gui/gui.go`, `inputWordsFlow`, locate the word-box construction (~:600-610):
```go
		top, _ := content.CutBottom(kbdsz.Y)
		word, _ := layoutWord(&ctx.B, selected+1, wordLabel)
		txtBg := op.Layer(
			word,
			op.Compose(
				op.Color(&ctx.B, th.Text),
				op.RoundedRect2(&ctx.B, r, cornerRadius),
			),
		).Offset(top.Center(longest))
```
Replace it with (capture the word-box offset, then add a count label below it):
```go
		top, _ := content.CutBottom(kbdsz.Y)
		wordOff := top.Center(longest)
		word, _ := layoutWord(&ctx.B, selected+1, wordLabel)
		txtBg := op.Layer(
			word,
			op.Compose(
				op.Color(&ctx.B, th.Text),
				op.RoundedRect2(&ctx.B, r, cornerRadius),
			),
		).Offset(wordOff)

		var countOp op.Op
		if len(kbd.Fragment) > 0 {
			noun := "matches"
			if nvalid == 1 {
				noun = "match"
			}
			cl, csz := widget.Labelf(&ctx.B, ctx.Styles.word, th.Text, "%d %s", nvalid, noun)
			countOp = cl.Offset(image.Pt((dims.X-csz.X)/2, wordOff.Y+longest.Y+8))
		}
```
Then add `countOp` to the frame's layer list. Find the `ctx.Frame(op.Layer(...))` at the end of the loop (~:619-625):
```go
		ctx.Frame(op.Layer(
			kbdOp,
			txtBg,
			nav,
			title,
			op.Color(&ctx.B, th.Background),
		))
```
and insert `countOp` after `txtBg`:
```go
		ctx.Frame(op.Layer(
			kbdOp,
			txtBg,
			countOp,
			nav,
			title,
			op.Color(&ctx.B, th.Background),
		))
```
(`countOp`'s zero value `op.Op{}` layers harmlessly when the fragment is empty — same pattern as `layoutNavigation` returning `op.Op{}` for `StyleNone`.)

- [ ] **Step 4: Run test to verify it passes**

Run: `go test ./gui/ -run TestWordFlowMatchCount -v`
Expected: PASS.

- [ ] **Step 5: Run the full suite + gofmt**

Run:
```bash
go test ./gui/... ./bip39/...
gofmt -l gui/gui.go gui/gui_test.go
```
Expected: `ok`; `gofmt -l` prints nothing.

- [ ] **Step 6: Commit**

```bash
git add gui/gui.go gui/gui_test.go
git commit -s -m "gui: show remaining-match count during word entry"
```

---

## Task 5: Last-word checksum assistance

**Files:**
- Modify: `gui/gui.go` — add `updateValidCandidateKeys` + `completeCandidateWord` (near `completeBIP39Word`/`updateValidBIP39Keys`, ~:860-893); wire the last-word path into `inputWordsFlow`
- Test: `gui/gui_test.go`

### 5a — the two candidate helpers (unit-tested directly)

- [ ] **Step 1: Write the failing helper tests**

Add to `gui/gui_test.go`:
```go
func validMnemonic(n int) bip39.Mnemonic {
	m := make(bip39.Mnemonic, n)
	for i := range m {
		m[i] = bip39.Word(i % int(bip39.NumWords))
	}
	return m.FixChecksum()
}

func TestCompleteCandidateWord(t *testing.T) {
	v := validMnemonic(24)
	cands := bip39.LastWordCandidates(v)
	if len(cands) != 8 {
		t.Fatalf("expected 8 candidates, got %d", len(cands))
	}
	last := v[len(v)-1]

	// Exact candidate label completes to that word.
	if w, ok := completeCandidateWord(cands, bip39.LabelFor(last), 1); !ok || w != last {
		t.Errorf("exact candidate: got (%d,%v), want (%d,true)", w, ok, last)
	}

	// A non-candidate full BIP-39 word must NOT complete (the I2 hole).
	inCands := map[bip39.Word]bool{}
	for _, w := range cands {
		inCands[w] = true
	}
	var nonCand bip39.Word = -1
	for w := bip39.Word(0); w < bip39.NumWords; w++ {
		if !inCands[w] {
			nonCand = w
			break
		}
	}
	if _, ok := completeCandidateWord(cands, bip39.LabelFor(nonCand), 1); ok {
		t.Errorf("non-candidate word %q completed but must not", bip39.LabelFor(nonCand))
	}
}

func TestUpdateValidCandidateKeys(t *testing.T) {
	ctx := NewContext(newPlatform())
	v := validMnemonic(24)
	cands := bip39.LastWordCandidates(v)

	// Expected enabled first-letters = first letter (lowercased) of each candidate label.
	wantEnabled := map[rune]bool{}
	for _, w := range cands {
		label := bip39.LabelFor(w)
		wantEnabled[unicode.ToLower(rune(label[0]))] = true
	}

	kbd := NewKeyboard(ctx, wordKeys)
	updateValidCandidateKeys(cands, "", kbd.allKeys)
	for i := range kbd.allKeys {
		key := &kbd.allKeys[i]
		if key.r == '⌫' {
			continue
		}
		enabled := !key.disabled
		if enabled != wantEnabled[key.r] {
			t.Errorf("key %q: enabled=%v, want %v", key.r, enabled, wantEnabled[key.r])
		}
	}
}
```
(Add `"unicode"` to `gui/gui_test.go`'s imports if not present.)

- [ ] **Step 2: Run to verify failure**

Run: `go test ./gui/ -run 'TestCompleteCandidateWord|TestUpdateValidCandidateKeys' -v`
Expected: FAIL — `undefined: completeCandidateWord` / `updateValidCandidateKeys`.

- [ ] **Step 3: Implement the helpers**

Add to `gui/gui.go` next to `completeBIP39Word`/`updateValidBIP39Keys` (~:867-893):
```go
// completeCandidateWord reports completion against a fixed candidate set
// (the checksum-valid last words). Unlike completeBIP39Word it never
// completes on a non-candidate label, so a checksum-invalid final word can
// never be accepted.
func completeCandidateWord(cands []bip39.Word, frag string, nvalid int) (bip39.Word, bool) {
	for _, w := range cands {
		if frag == bip39.LabelFor(w) {
			return w, true
		}
	}
	if nvalid == 1 {
		for _, w := range cands {
			if strings.HasPrefix(bip39.LabelFor(w), frag) {
				return w, true
			}
		}
	}
	return -1, false
}

// updateValidCandidateKeys restricts the keyboard to letters that extend the
// fragment toward one of the candidate words, mirroring updateValidBIP39Keys
// but over a fixed candidate set. Returns the number of still-matching
// candidates.
func updateValidCandidateKeys(cands []bip39.Word, frag string, keys []keyboardKey) int {
	mask := ^uint32(0)
	nvalid := 0
	for _, w := range cands {
		label := bip39.LabelFor(w)
		if !strings.HasPrefix(label, frag) {
			continue
		}
		nvalid++
		suffix := label[len(frag):]
		if len(suffix) > 0 {
			idx := unicode.ToLower(rune(suffix[0])) - 'a'
			mask &^= 1 << idx
		}
	}
	if nvalid == 1 {
		mask = ^uint32(0)
	}
	updateValidKeys(mask, keys)
	return nvalid
}
```

- [ ] **Step 4: Run to verify pass**

Run: `go test ./gui/ -run 'TestCompleteCandidateWord|TestUpdateValidCandidateKeys' -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add gui/gui.go gui/gui_test.go
git commit -s -m "gui: add candidate-scoped key/completion helpers for last word"
```

### 5b — wire the last-word path into `inputWordsFlow`

- [ ] **Step 6: Write the failing integration test**

Add to `gui/gui_test.go`:
```go
func TestWordFlowLastWord24(t *testing.T) {
	v := validMnemonic(24)

	// Candidate count visible on entering the last word (empty fragment).
	{
		ctx := NewContext(newPlatform())
		m := make(bip39.Mnemonic, 24)
		copy(m, v)
		m[23] = -1 // last slot unset; first 23 are valid
		frame, quit := runUI(ctx, func() {
			inputWordsFlow(ctx, &descriptorTheme, m, 23)
		})
		content, ok := frame()
		quit()
		if !ok {
			t.Fatal("no frame at last word")
		}
		if !uiContains(content, "8 matches") {
			t.Errorf("expected %q at last word; got %q", "8 matches", content)
		}
	}

	// Typing the correct last word commits it.
	{
		ctx := NewContext(newPlatform())
		m := make(bip39.Mnemonic, 24)
		copy(m, v)
		m[23] = -1
		want := v[23]
		runes(&ctx.Router, bip39.LabelFor(want))
		click(&ctx.Router, Button3)
		inputWordsFlow(ctx, &descriptorTheme, m, 23)
		if m[23] != want {
			t.Errorf("last word committed %d (%q), want %d (%q)",
				m[23], bip39.LabelFor(m[23]), want, bip39.LabelFor(want))
		}
	}
}
```

- [ ] **Step 7: Run to verify failure**

Run: `go test ./gui/ -run TestWordFlowLastWord24 -v`
Expected: FAIL — the count shows the full-wordlist match count (not `8 matches`) because the last-word path isn't wired yet. (The commit sub-test may already pass via the normal path; the count sub-test must fail.)

- [ ] **Step 8: Wire the candidate path into `inputWordsFlow`**

In `gui/gui.go`, `inputWordsFlow`, after `var nvalid int` (~:550) add the candidate state + routing closures:
```go
	var nvalid int
	var cands []bip39.Word
	candsFor := -1
	onLastWord := func() bool { return selected == len(mnemonic)-1 && cands != nil }
	updateKeys := func(frag string) int {
		if onLastWord() {
			return updateValidCandidateKeys(cands, frag, kbd.allKeys)
		}
		return updateValidBIP39Keys(frag, kbd.allKeys)
	}
	completeWord := func(frag string, nv int) (bip39.Word, bool) {
		if onLastWord() {
			return completeCandidateWord(cands, frag, nv)
		}
		return completeBIP39Word(frag, nv)
	}
```
At the very top of the `for !ctx.Done {` loop body (before `for kbd.Update(ctx) {`), recompute the candidate set when entering the last-word state, and apply the candidate mask immediately:
```go
	for !ctx.Done {
		if selected == len(mnemonic)-1 && candsFor != selected {
			cands = bip39.LastWordCandidates(mnemonic)
			candsFor = selected
			nvalid = updateKeys(kbd.Fragment)
			wordLabel = kbd.Fragment
			if cw, ok := completeWord(kbd.Fragment, nvalid); ok {
				wordLabel = bip39.LabelFor(cw)
			}
		}
		for kbd.Update(ctx) {
			nvalid = updateKeys(kbd.Fragment)
			wordLabel = kbd.Fragment
			if completedWord, ok := completeWord(wordLabel, nvalid); ok {
				wordLabel = bip39.LabelFor(completedWord)
			}
		}
```
(Replace the existing `for kbd.Update(ctx) {` block's calls to `updateValidBIP39Keys`/`completeBIP39Word` with `updateKeys`/`completeWord` as shown.)

In the `for okBtn.Clicked(ctx) {` block, route the completion check through `completeWord` and leave the post-accept reset on the full keyboard (the next frame's top-of-loop guard re-applies the candidate mask for the new `selected`):
```go
		for okBtn.Clicked(ctx) {
			w, ok := completeWord(kbd.Fragment, nvalid)
			if !ok {
				continue
			}
			kbd.Clear()
			wordLabel = ""
			nvalid = updateValidBIP39Keys("", kbd.allKeys)
			mnemonic[selected] = w
			for {
				selected++
				if selected == len(mnemonic) {
					return
				}
				if mnemonic[selected] == -1 {
					break
				}
			}
		}
```
In the nav-completion check (~:613), route through `completeWord`:
```go
		if _, ok := completeWord(kbd.Fragment, nvalid); ok {
			nav2, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{{Clickable: okBtn, Style: StylePrimary, Icon: assets.IconCheckmark}}...)
			nav = op.Layer(
				nav,
				nav2,
			)
		}
```
Finally, extend the Task 4 match-count guard so the count shows at the last word even on an empty fragment (where "8 matches"/"128 matches" is informative — unlike the "2048 matches" the guard suppresses for ordinary words). Change the count condition added in Task 4:
```go
		if len(kbd.Fragment) > 0 {
```
to:
```go
		if len(kbd.Fragment) > 0 || onLastWord() {
```
(For non-last words `onLastWord()` is false, so the empty-fragment suppression from Task 4 is preserved; `TestWordFlowMatchCount` — which runs at `selected=0` — is unaffected.)

- [ ] **Step 9: Run to verify pass**

Run: `go test ./gui/ -run TestWordFlowLastWord24 -v`
Expected: PASS — `8 matches` shown at the last word; the correct last word commits.

- [ ] **Step 10: Run the full suites + vet + gofmt**

Run:
```bash
go test ./gui/... ./bip39/...
go vet ./gui/... ./bip39/...
gofmt -l gui/gui.go gui/gui_test.go bip39/bip39.go bip39/bip39_test.go
```
Expected: all `ok`; vet clean (ignore the unrelated pre-existing `gui/op/draw_test.go` go1.25/1.26 note if shown); `gofmt -l` prints nothing. In particular `TestWordKeyboardScreen` still passes — with `make(bip39.Mnemonic, 1)` the last-word guard fires (`selected==0==len-1`) but `LastWordCandidates` returns nil on `len%3==1`, so the flow falls back to the normal full keyboard.

- [ ] **Step 11: Commit**

```bash
git add gui/gui.go gui/gui_test.go
git commit -s -m "gui: assist last-word entry with checksum-valid candidates

On the final word, restrict the keyboard to the checksum-valid candidate
words (8 for 24-word, 128 for 12-word) so any completed word is valid; the
SeedScreen.Confirm Valid() check remains the backstop for earlier words."
```

---

## Final verification

- [ ] **Run the complete host suite**

Run: `go test ./gui/... ./bip39/...`
Expected: all `ok`.

- [ ] **Confirm the diff is Slice-1-only and based on upstream/main**

Run:
```bash
git fetch -q upstream
git log --oneline upstream/main..HEAD     # expect: the Task 1-5 commits only
git diff --stat upstream/main...HEAD       # expect: bip39/bip39.go(+_test), gui/gui.go, gui/gui_test.go, gui/codex32_input_test.go
```

---

## Self-review (run against the spec)

**1. Spec coverage:**
- §4.1 progress title → Task 3 ✓
- §4.2 match count (guard `len(frag)>0`, candidate-scoped at last word) → Task 4 (+ Task 5 routes `nvalid`) ✓
- §4.3 Button3 across all three flows + both test sites → Task 2 ✓
- §4.4 `LastWordCandidates` (8/128, guard, clone) → Task 1; candidate-scoped completion/mask + memoized per-frame routing keyed on `selected==len-1` → Task 5 ✓
- §5 backstop `mnemonic.Valid()` retained → unchanged in `SeedScreen.Confirm` (no task modifies it) ✓
- §6 tests: `LastWordCandidates` unit (8/128/nil/non-mutation) → Task 1; both Button2→Button3 updates → Task 2; progress/match-count → Tasks 3/4; candidate completion incl. non-candidate-rejection (I2) + mask → Task 5a; 24-word `8 matches`+commit → Task 5b ✓
- §7 no version bump → none made ✓

**2. Placeholder scan:** none — every step has concrete code/commands.

**3. Type consistency:** `LastWordCandidates(prefix bip39.Mnemonic) []bip39.Word`; `updateValidCandidateKeys(cands []bip39.Word, frag string, keys []keyboardKey) int`; `completeCandidateWord(cands []bip39.Word, frag string, nvalid int) (bip39.Word, bool)`; closures `updateKeys(string) int`, `completeWord(string,int)(bip39.Word,bool)`, `onLastWord() bool` — all consistent across tasks. `widget.Labelf(buf,style,color,format,args...)` matches existing `layoutWord` usage. `emptyBIP39Mnemonic(int)`, `validMnemonic(int)`, `runUI`, `uiContains`, `runes`, `click` all match the verified harness.

---

## Execution handoff

This plan must first pass an **opus-architect plan-R0 gate (0C/0I)** per the project standard, then execute via **superpowers:subagent-driven-development** (fresh subagent per task + two-stage review) in an isolated worktree on branch `feat/bip39-entry-polish`.
