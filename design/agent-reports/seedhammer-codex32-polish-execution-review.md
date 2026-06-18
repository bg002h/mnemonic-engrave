# Cycle A1: codex32-input-polish — WHOLE-DIFF ADVERSARIAL EXECUTION REVIEW

- **Stage:** mandatory, non-deferrable post-implementation execution review over the whole diff (phase 4 of the refined ultracode policy). Catches implementation-introduced regressions TDD/R0 miss. Read-only.
- **Diff reviewed:** branch `feat/codex32-input-polish`, base `3c4d3d3` … head `27e6ad0` (the 6 implementation commits). 5 files: `codex32/polish.go`, `codex32/polish_test.go`, `gui/codex32_polish.go`, `gui/codex32_polish_test.go`, `gui/gui.go`.
- **Reviewer:** `feature-dev:code-reviewer` (agentId `af4212256f72bf28e`; ran the toolchain — full `go test`, `go vet`, adversarial scratch inputs).
- **Outcome:** **SHIP-READY — 0 Critical / 0 Important.** 3 Minors (M1 folded post-review; M2/M3 accepted — see fold note).

> NOTE: verbatim reviewer output, recovered from the agent transcript (`tasks/af4212256f72bf28e.output`); a short working-notes preamble was omitted; the report proper (from `# Adversarial Execution Review`) is verbatim.

---

## VERBATIM REVIEW OUTPUT

# Adversarial Execution Review — SeedHammer CODEX32 Input Polish (Cycle A1)
**Branch:** `feat/codex32-input-polish` | **Base:** `3c4d3d3` | **Head:** `27e6ad0` | **Reviewer model:** claude-sonnet-4-6

---

## What I reviewed

All five changed files: `codex32/polish.go`, `codex32/polish_test.go`, `gui/codex32_polish.go`, `gui/codex32_polish_test.go`, `gui/gui.go`. Verified `codex32/codex32.go` and `codex32/mdmk.go` are untouched (no `ParsePrefix`, `Describe`, or exported constants appear in those files). Read the full `New`/`partsInner`/`splitHRP`/`feFromRune`/`Split` source to ground every adversarial case.

---

## CRITICAL

None found.

---

## IMPORTANT

None found.

---

## MINOR

**M1 — UX oddity: pressing Back in the confirm screen sets "Unknown format" on the start screen**
File/lines: `/scratch/code/shibboleth/seedhammer-wt-codex32/gui/gui.go:1437-1439`

`engraveObjectFlow` returns `false` when `confirmCodex32Flow` returns `false` (user pressed Back). The call site at line 1437 treats `false` as "scan not handled" and sets `s.Status = scanUnknownFormat`, displaying "Unknown format" on the start screen. Before this patch, there was no confirm screen, so a codex32 string entered via `inputCodex32Flow` always proceeded to engrave (no cancellable confirm step). The "Unknown format" label is confusing when the user deliberately pressed Back from a well-formed codex32 string. This is consistent with the plan's explicit instruction ("use `return false` on back-out"), but the `return false` convention in this function conflates "user cancelled" with "unrecognized object type". Not a correctness bug; the engrave path is unaffected. Fix (if desired, out of scope for cycle A1): the caller at line 1437 would need to distinguish "user cancelled" from "format unknown", requiring an architecture change.

**M2 — `ParsePrefix("1")` / `ParsePrefix("1x")`: separator present but treated as absent**
File/lines: `/scratch/code/shibboleth/seedhammer-wt-codex32/codex32/polish.go:80-83`

`splitHRP("1")` returns `("", "")` and `splitHRP("1x")` returns `("", "x")`; in both cases `hrp == ""` triggers the early-return before touching `data`. For `"1x"`, this suppresses the threshold error for `'x'` (not a valid threshold digit), because the code treats an empty HRP as meaning "no separator seen yet." These are purely advisory: `New` would reject both strings with `errInvalidLength` (too short) and then `errInvalidThreshold`. No test covers these inputs, no user can type them on the force-uppercasing keypad (a real `"1"` prefix would go through `splitHRP("1xxxx...")` → `("", "xxxx...")` which then falls into the threshold check normally). Not a panic risk, not a correctness issue for the OK gate.

**M3 — `addLine` clamp: second info line can overlap the first in degenerate layouts**
File/lines: `/scratch/code/shibboleth/seedhammer-wt-codex32/gui/gui.go:720-731`

When the status line is already clamped to `top.Max.Y - sz.Y`, `lineY` advances to `top.Max.Y + 4`, so the second line clamps independently to the same `top.Max.Y - sz.Y` region, producing visual overlap. This matches the `inputWordsFlow` precedent and only affects extreme layout configurations (keyboard taller than content area). Tests assert on text presence, not pixel position, so the suite remains green.

---

## Positive findings (things that could have gone wrong but didn't)

**`ParsePrefix` panic-safety:** Confirmed safe on all adversarial inputs — empty string, `"1"`, `"ms1"`, non-ASCII/multibyte bytes in identifier position, bytes 128–255 in share-index position, 127-char max vectors. `feFromRune` returns `(0, false)` for bytes ≥ 128 (outside the 128-element `invCharsTbl`), so the `errInvalidCharacter` path is taken without any panic or out-of-bounds access.

**Threshold-0 timing (spec §4.1c):** The `if f.ThresholdKnown && f.Threshold == 0 && !f.Unshared` check is correctly gated by `len(data) >= 6`. `"ms10te"` (len < 6) returns nil — verified against the test and traced manually.

**Once-per-frame refactor (C2/C3):** `for kbd.Update(ctx)` drains ALL queued rune events in a single frame iteration (router `Next` returns each queued event immediately; the drain loop runs until no more rune events remain). `New` and `ParsePrefix` are then called once on the stable, fully-typed fragment. Accept and back semantics are preserved: both `okBtn` and `backBtn` use `ButtonFilter` (not pointer), which works before frame rendering.

**Shared backing array (C5):** `NewKeyboard` constructs `keys[i] = allKeys[prevIdx:nextIdx]` slices that share the same backing array as `allKeys`. Mutating `allKeys[i].disabled = true` in `newCodex32Keyboard` is reflected by `keys[row][col].disabled` reads in `Update`/`adjust`/`Valid`. `Clear()` resets only `Fragment`/`row`/`col`, not `disabled` — static dimming survives. BIP-39 keyboard is a separate heap-allocated instance; no cross-contamination.

**C4 confirm gate correctness:** `confirmCodex32Flow` calls `codex32.ParsePrefix(scan.String())` where `scan` is New-valid. For any New-valid string: consistent case (passes `checkCase`), valid bech32 chars (passes `feFromRune`), valid threshold digit (passes switch), valid bech32 identifier chars (passes `feFromRune`), valid bech32 share index (passes `feFromRune`). `ParsePrefix` returns nil error and fully populated `Fields` on a New-valid string — the `"id " + f.Identifier` + Unshared branch is always safe. `Split()` is used only for `id` after the confirm gate and is untouched.

**`Describe` completeness:** All seven sentinels `New` can return (`errInvalidChecksum`, `errInvalidLength`, `errInvalidCharacter`, `errInvalidCase`, `errInvalidThreshold`, `errInvalidShareIndex`, `errIncompleteGroup`) map to human-readable labels. The `fmt.Errorf("invalid character: %c", c)` non-sentinel from `inputHRP`'s second loop (unreachable via `ParsePrefix`) and `Interpolate`-only errors fall through to "invalid" as intended.

**`uiContains` assertions are non-vacuous:** Verified that "0chars", "idname", "thr2", "badthreshold", "keeptyping", "badchecksum" cannot be supplied by other rendered content (title "Input Codex32 Share", navigation icons) and are exclusive to the new status/field/feedback lines.

**`TestInputSeedCodex32` guard:** The test vector `"ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw"` contains no 'b', 'i', or 'o'. All characters are processed by the dimmed-b/i/o keyboard without silent drops. The test still accepts on Button3.

**Scope hygiene:** `codex32/codex32.go` and `codex32/mdmk.go` are untouched (verified by grep). `Split()` is called only for `id` in the existing post-confirm code path. No stray debug code.

---

## Verdict

**SHIP-READY — 0 Critical / 0 Important**

The three Minors are low-risk and consistent with the plan's explicit design choices (M1) or pre-existing framework patterns (M2, M3). The green test suite is meaningful: the tests are non-vacuous, the white-box assertions use correct sentinels, and the integration tests exercise the actual rendering path. No implementation-introduced regressions found.


---

## Fold note (main session)
Verdict was **SHIP-READY (0C/0I)** — gate passed. Disposition of the 3 Minors:

- **M1 (FOLDED):** "Back at the confirm screen → 'Unknown format'." On closer inspection the fix is a one-liner, NOT the architecture change the reviewer assumed: `engraveObjectFlow` returns `true` for every *recognized* object (only `default:`/unrecognized returns `false` → `scanUnknownFormat`, gui.go:1856-1859 + caller gui.go:1437-1438). Backing out of the confirm is a deliberate cancel of a *recognized* codex32 string, so the correct, consistent return is `true` (handled), matching how backing out of every other engrave flow behaves. Changed the codex32 back-out from `return false` to `return true` + added regression test `TestEngraveCodex32BackoutNotUnknown` (drives Button1 at the confirm, asserts `engraveObjectFlow` returns true). Test fails on the old code, passes on the new. Committed `2209c66`. Full `go test ./...` re-run: ALL PASS.
- **M2 (ACCEPTED):** `ParsePrefix("1")`/`("1x")` treat a position-0 separator as "no separator yet" → suppress an advisory threshold error. Reviewer explicitly: "not a defect" — advisory-only, unreachable on the force-uppercasing keypad, `New` is the validity authority. No change.
- **M3 (ACCEPTED):** `addLine` clamp can visually overlap on degenerate (keyboard-taller-than-content) displays. Matches the pre-existing `inputWordsFlow` precedent; visual-only, no correctness impact, tests assert text presence not pixel position. No change.

Post-fold head: `2209c66` (7 commits). M1's one-line code change + dedicated regression test + green full suite is adequately verified without re-dispatching the whole-diff review (the change is self-contained and directly covered).
