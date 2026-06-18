# Cycle A1: codex32-input-polish — IMPLEMENTATION PLAN R0 GATE REVIEW — R0

- **Stage:** mandatory implementation-plan R0 gate (project standard — must reach 0 Critical / 0 Important before any code). Read-only, adversarial.
- **Plan reviewed:** `design/IMPLEMENTATION_PLAN_seedhammer_codex32_input_polish.md` (committed `f0f4734`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `a20daaedf02423d96`). Verified against fork `main` `3c4d3d3` (the build base) with file:line evidence; BIP-93 facts via the spec/code.
- **Outcome:** **RED — 1 Important (IMP-2).** Folded; re-dispatched for R1.

> NOTE: the verbatim architect output is reproduced below, recovered from the agent transcript (`tasks/a20daaedf02423d96.output`). A working-notes preamble that preceded the formal report has been omitted; the report proper (from `## REVIEW REPORT`) is verbatim.

---

## VERBATIM REVIEW OUTPUT

## REVIEW REPORT

**Target:** `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_seedhammer_codex32_input_polish.md`
**Base:** fork `main` `3c4d3d3`
**Reviewer:** opus architect
**Date:** 2026-06-17

---

### CRITICAL

None found.

---

### IMPORTANT

**IMP-1: `splitHRP` no-separator return semantics — plan description is imprecise, but code is correct**

`splitHRP` when no '1' is found returns `("", s)` where the second value is the ENTIRE input string, not an empty data part. The plan's prose says "returns `("", whole)` until a 1 separator" — this is technically correct, but the significant implication is: when `hrp == ""`, the `data` variable inside `ParsePrefix` equals `frag` (the whole input). The code correctly handles this by early-returning `(Fields{}, nil)` when `hrp == ""`. However, if a future implementer moves the `checkCase(frag)` call AFTER the `hrp==""` guard (for performance), they'd need `checkCase(data)` not `checkCase(hrp)`. As written, the structure is safe. No code change needed, but this warrants a code comment inside `ParsePrefix` noting that `data` contains the entire string when `hrp==""` to prevent maintenance confusion.

**Severity: IMPORTANT** — no bug exists in the plan's code as written, but the implicit aliasing is a maintenance trap. The implementer should add the comment.

**Concrete fix:** Inside `ParsePrefix`, after `hrp, data := splitHRP(frag)`, add:
```go
// When splitHRP finds no '1', data == frag (entire input). We return
// early below, so no field from data is accessed in that case.
```

---

**IMP-2: `ParsePrefix` identifier field stored as raw bytes, not runes — potential mismatch with uppercase vector test**

The plan stores `f.Identifier = data[1:5]` (a string slice of raw bytes from the input). For the uppercase test vector `"MS12NAMEA..."`, `data = "2NAMEA..."`, `data[1:5] = "NAME"`. The test expects `f.Identifier != "NAME"` (checking for exact equality). This works because all bech32 chars are ASCII single-byte. But the `codex32FieldLine` function does `strings.ToUpper(f.Identifier)` — for `"NAME"` (already uppercase), `strings.ToUpper("NAME") = "NAME"`. For `"name"` (lowercase), `strings.ToUpper("name") = "NAME"`. Both produce `"id NAME"` in the output. This is correct. ✓

However: the test for `"ms12name"` checks `f.Identifier != "name"` and `codex32FieldLine(f)` produces `"id NAME · thr 2"`. The `f.Identifier = "name"` (lowercase) and `strings.ToUpper("name") = "NAME"`. ✓. No defect here — downgrading to a note.

This is NOT an IMPORTANT — downgraded to MINOR.

---

**IMP-2 (real): `confirmCodex32Flow` uses `widget.Labelw` but `codex32_polish.go` header imports don't include `widget` until Task 4 ships the helpers**

The plan notes at Task 4 Step 3:
> "Note: this file's imports (assets, layout, op, widget) are used by Task 5/6 additions to the same file; if a task is implemented in isolation, drop the unused imports and re-add them when the later task lands (Go rejects unused imports)."

But the plan's Task 4 Step 3 writes ALL six functions (`codex32StatusLine`, `codex32FieldLine`, `codex32Feedback`) into `codex32_polish.go` WITH the full import block `("fmt", "image", "strings", "codex32", "assets", "layout", "op", "widget")`. The helpers only use `fmt`, `strings`, and `codex32` — NOT `image`, `assets`, `layout`, `op`, `widget`. Go will fail to compile with unused imports.

The note says to "temporarily comment them" — but this means the Task 4 Step 3 code block as written CANNOT compile as provided. The implementer must diverge from the written code to get a green test at Step 4. The plan's "if a task is implemented in isolation" caveat partially covers this, but only in the note, not in the actual code block.

**file:line evidence:** Task 4, Step 3 — `gui/codex32_polish.go` import block lists `"seedhammer.com/gui/assets"`, `"seedhammer.com/gui/layout"`, `"seedhammer.com/gui/op"`, `"seedhammer.com/gui/widget"`, `"image"`. None of these are used by `codex32StatusLine`, `codex32FieldLine`, or `codex32Feedback`.

**Concrete fix:** The import block in Task 4 Step 3 should contain ONLY `"fmt"`, `"strings"`, and `"seedhammer.com/codex32"`. The remaining imports should be added in Task 5 when `confirmCodex32Flow` is appended. The plan's instruction to "temporarily comment" is operationally correct but conflicts with the "run to verify it passes" step at Task 4 Step 4 — which WILL fail if the full import block is present. The implementer MUST use the restricted import set in Task 4.

---

### MINOR

**MIN-1: `splitHRP` returns `("", p1)` not `("", whole)` — plan's wording is slightly misleading**

`strings.Cut(s, "1")` when no `"1"` found: `p1=s`, `p2=""`, `ok=false`. `splitHRP` returns `("", p1)` = `("", s)`. The FIRST return value (HRP) is `""`, the second (data) is `s`. The plan in multiple places says "splitHRP returns `("", whole)` if no 1" — this is correct, but the variable name in `splitHRP` is `p1`, which becomes the data. Just a naming confusion in the prose; the code path is correct.

**MIN-2: `feFromRune` called with `rune(data[5])` — byte-to-rune cast is correct but implicit**

`data` is a string; `data[5]` is a `byte`. `rune(data[5])` casts byte (0..255) to rune. For ASCII bech32 chars this is always correct. For bytes 128..255 (non-ASCII), `feFromRune` returns `(0, false)` since `invCharsTbl` only covers 0..127. This is the correct behavior (non-ASCII = invalid char). No bug, but a comment would help clarity.

**MIN-3: The `codex32Keys` row layout comment in C5 calls the digit row "full QWERTY + digits" but the original `alph` was already digits + partial QWERTY**

The plan says the keyboard is "switched to the BIP-39 full-QWERTY+digits layout." The BIP-39 `wordKeys = "qwertyuiop\nasdfghjkl\nzxcvbnm"` has NO digit row. The codex32 original `alph = "1234567890\nqwertyup\nasdfghjk\nlzxcvnm"` has digits but a PARTIAL letter layout. The new `codex32Keys = "1234567890\nqwertyuiop\nasdfghjkl\nzxcvbnm"` is digits + FULL QWERTY — not the same as BIP-39's layout (which has no digits). The comment in the code correctly says "digit row + the BIP-39 full-QWERTY letter rows" which is accurate. The spec description "switch to BIP-39 full-QWERTY layout" is slightly misleading (BIP-39 has no digits), but the implementation is correct.

**MIN-4: `ParsePrefix` does not validate HRP value ("ms" or "MS")**

The spec §4.1c says: "HRP on `1` (validate ms/MS case-folded)". The plan's `ParsePrefix` sets `f.HRP = hrp` but does NOT validate that `hrp` is actually `"ms"` or `"MS"`. An input like `"xx1rest..."` would set `f.HRP = "xx"` without error. This is advisory-only (New is the authority), and the plan's spec says `ParsePrefix` is "fail-soft," so this omission is acceptable. However, the spec notes HRP validation as a determinability step. If the HRP is wrong, the display could show misleading field info. Low severity since it only affects display.

**MIN-5: Task 4's `codex32Frame` test helper calls `inputCodex32Flow` which in Task 4 still uses the OLD keyboard (no `newCodex32Keyboard` yet)**

The `codex32Frame` helper calls `inputCodex32Flow`, which at Task 4 time still initializes with `const alph = "1234567890\nqwertyup\nasdfghjk\nlzxcvnm"` and `kbd := NewKeyboard(ctx, alph)`. Task 6 swaps this to `newCodex32Keyboard`. The test `TestCodex32FlowReadout` types strings like `"ms10tests"...` — checking whether any character in the test vectors is absent from the Task-4 keyboard (old `alph`):
- `alph = "1234567890\nqwertyup\nasdfghjk\nlzxcvnm"` — contains: digits, q,w,e,r,t,y,u,p,a,s,d,f,g,h,j,k,l,z,x,c,v,n,m (but NOT 'o','i','b')
- Test vector `"ms10tests" + repeat("x", 91)`: m✓, s✓, 1✓, 0✓, t✓, e✓, s✓, t✓, s✓, x✓ — all present in old keyboard ✓
- Test vector `"ms10fauxsxxxxxxxxxx...ve740yyge2ghp"`: m✓, s✓, 1✓, 0✓, f✓, a✓, u✓, x✓, s✓... y✓, g✓, h✓, p✓ — all present ✓
- Test vector `"ms12name"`: m✓, s✓, 1✓, 2✓, n✓, a✓, m✓, e✓ ✓
- Test vector `"ms11"`: m✓, s✓, 1✓, 1✓ ✓ (note: '1' is in the digit row and is how the separator gets typed)

All vectors work with old keyboard. ✓ No issue.

**MIN-6: `word` variable shadowing in the plan's rewrite**

In the plan's `inputCodex32Flow`, the local `word` variable is of type `op.Op`. First assigned by `widget.Labelw(...)`, then reassigned by `op.Layer(word, ...).Offset(wordOff)`. This pattern (reassigning `word` twice) is present in the original code too. No shadowing issue. ✓ (This is a non-issue, noting for completeness.)

**MIN-7: `confirmCodex32Flow` string construction — "engraves THIS share, not a recovered seed" vs spec's "engraves THIS share, not a seed"**

The spec at §4.4 says the note is: `"engraves THIS share, not a recovered seed"`. The plan's code produces: `"engraves THIS share, not a recovered seed"`. The test `uiContains(c, "not a recovered seed")` → cleaned to `"notarecoveredseed"`. This matches the string. ✓ No defect.

---

### SCOPE DISCIPLINE CHECK

- `Split()` — not modified anywhere. ✓
- `mdmk.go` — not mentioned anywhere in the plan. ✓
- Long-code gate — not widened. ✓
- `TestWordKeyboardScreen` / `TestInputSeedCodex32` — guarded by existing guard test steps. ✓
- Fork-side only, no upstream PR — explicitly confirmed. ✓
- Commits signed + DCO, author Brian Goss — specified in all commit steps. ✓
- No `git add -A` — all commits use explicit paths. ✓

---

### KEY VERIFIED FACTS (with file:line evidence)

1. `splitHRP` signature: `func splitHRP(s string) (string, string)` — `codex32/codex32.go:453`. Plan's usage is correct.
2. `feFromRune` signature: `func feFromRune(c rune) (fe, bool)` — `codex32/gf32.go:126`. Plan's usage `if _, ok := feFromRune(c); !ok` is correct.
3. `New` wraps all errors via `fmt.Errorf("codex32: %w", ...)` — `codex32/codex32.go:106,112,117,122`. `errors.Is` matching works. ✓
4. All seven sentinels that `Describe` maps exist at `codex32/codex32.go:23-37`. ✓
5. `Alphabet = "QPZRY9X8GF2TVDW0S3JN54KHCE6MUA7L"` — `codex32/codex32.go:21`. ✓
6. `inputCodex32Flow` at `gui/gui.go:672-731` — body matches plan's description of what it replaces. ✓
7. `engraveObjectFlow` `case codex32.String:` at `gui/gui.go:1819-1826` — falls through to `return true` at line 1834; back-out `return false` is correct. ✓
8. `NewKeyboard` builds `allKeys` then reassigns `k.keys[i] = allKeys[:len(row)]` at `gui/gui.go:877` — shared backing array confirmed. ✓
9. `Keyboard.Clear()` at `gui/gui.go:894-898` — does NOT reset `disabled`. ✓
10. `layoutNavigation` signature: `func layoutNavigation(buf *op.Buffer, th *Colors, dims image.Point, btns ...NavButton)` at `gui/gui.go:1640`. Plan's calls match. ✓
11. `layoutTitle` signature: `func layoutTitle(ctx *Context, width int, col color.RGBA, title string)` at `gui/gui.go:1611`. Plan's calls match. ✓
12. `widget.Label` signature: `func Label(buf *op.Buffer, st text.Style, col color.RGBA, txt string) (op.Op, image.Point)` — `gui/widget/label.go:12`. Plan's calls match. ✓
13. `widget.Labelw` signature: `func Labelw(buf *op.Buffer, st text.Style, width int, col color.RGBA, txt string) (op.Op, image.Point)` — `gui/widget/label.go:16`. Plan's calls match. ✓
14. `uiContains` strips spaces from needle only, lowercases both — `gui/gui_test.go:479-484`. Plan's test needles are compatible. ✓
15. `ExtractText` collects runes only from `*glyph` elements within non-empty clip bounds — `gui/op/op.go:327-331`. Spaces likely absent from output (consistent with `TestWordFlowMatchCount` pattern). Plan's space-insensitive assertions are correct. ✓
16. `Clickable{Button: Button3, AltButton: Center}` is the established pattern — `gui/gui.go:1277,1488`. Plan's `engraveBtn` declaration matches. ✓
17. `assets.IconHammer` exists — `gui/assets/embed.go:91`. ✓
18. `leadingSize = 44` — `gui/theme.go:43`. Accessible in `gui/codex32_polish.go`. ✓
19. `codex32Keys` `b/i/o` absent from bech32 confirmed via `invCharsTbl` — `codex32/gf32.go:38-47`. ✓
20. `module path` is `seedhammer.com/...` — `gui/gui.go:1-41` imports confirm. ✓

---

### VERDICT

**One IMPORTANT finding (IMP-2): the import block in Task 4 Step 3 as written will cause a compile failure** because `assets`, `layout`, `op`, `widget`, and `image` are imported but not used by the three helper functions added in that step. The plan's prose notes this risk and says "temporarily comment them," but the code block itself includes the full import set, and the Step 4 `go test` instruction expects PASS. The implementer must use a restricted import block at Task 4 Step 3 (adding the GUI imports only when Task 5 lands).

**All other plan elements are compile-accurate and behaviorally correct against the 3c4d3d3 codebase.** No code blocks have incorrect API signatures, struct field names, or import paths. The C4 back-out `return false` is correct. `ParsePrefix` cannot panic on any input. The shared-backing-array mechanism for static dimming is verified. The `uiContains`/`ExtractText` test assertions are sound. `TestInputSeedCodex32` and `TestWordKeyboardScreen` remain unaffected.

**VERDICT: RED — 1 Important (IMP-2 unused-import compile failure in Task 4 Step 3)**

Fold required: restrict the import block in Task 4 Step 3's `gui/codex32_polish.go` to only `"fmt"`, `"strings"`, and `"seedhammer.com/codex32"`. Expand the import block to add `"image"`, `"seedhammer.com/gui/assets"`, `"seedhammer.com/gui/layout"`, `"seedhammer.com/gui/op"`, `"seedhammer.com/gui/widget"` at the start of Task 5 Step 3 (when `confirmCodex32Flow` is appended). After this fold, re-dispatch for R1 verification.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | **RED — 0C / 1I** (IMP-2) | IMP-2: Task 4 Step 3 wrote the full import block into `gui/codex32_polish.go`, but its three helpers use only `fmt`/`strings`/`codex32` → unused-import compile failure; Step 4 `go test` would not build. **Folded:** Task 4 Step 3 import block restricted to `fmt`/`strings`/`codex32`; Task 5 Step 3 now expands the block (adds `image`/`assets`/`layout`/`op`/`widget`) when `confirmCodex32Flow` lands. Also folded the cheap clarity Minors as `ParsePrefix` code comments: IMP-1 (`data` aliases whole input when no `1`), MIN-2 (byte→rune is panic-safe for ASCII), MIN-4 (HRP recorded-not-rejected; New folds HRP into the checksum). The architect verified 20 load-bearing facts compile-accurate (signatures, import paths, the C4 `return false` back-out matches `engraveObjectFlow`'s convention, `ParsePrefix` cannot panic, the shared `allKeys`/`keys` backing-array dimming, `uiContains`/`ExtractText` space-insensitive assertions, `TestInputSeedCodex32`/`TestWordKeyboardScreen` unaffected, scope discipline: no `Split()`/`mdmk.go`/gate-widening). |

Re-dispatched for R1 verification after the fold.
