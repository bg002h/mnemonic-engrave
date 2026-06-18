# Cycle C: slip39-enablement (Tier 1) — IMPLEMENTATION PLAN R0 GATE REVIEW — R0

- **Stage:** mandatory implementation-plan R0 gate (0C/0I before any code).
- **Plan reviewed:** `design/IMPLEMENTATION_PLAN_seedhammer_slip39_enablement.md` (committed `6854452`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `af5e27bc07e328e64`), read-only vs fork `9b0a02c` + SLIP-0039; independently re-derived id 7945 & 29019.
- **Outcome:** **GREEN — 0 Critical / 0 Important** (first round). 5 Minors, all confirmed non-issues.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

## R0 Implementation-Plan Gate — Cycle C, Tier 1 (SLIP-39 Share Entry + Verbatim Engrave)

**Reviewer:** opus architect (adversarial, read-only)
**Plan commit:** `6854452` — `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_seedhammer_slip39_enablement.md`
**Base fork tree:** `9b0a02c` at `/scratch/code/shibboleth/seedhammer`
**Spec:** R0 GREEN at R4

---

### Verification Area 1 — `ParseShare` correctness and compile accuracy

**Header assembly.** Words: DUCKLING = index 248 (wordlist.txt line 249, 0-based), ENLARGE = index 288 (line 289), ACADEMIC = 0, ACADEMIC = 0. Re-derived independently:

```
hdr = uint64(248)<<30 | uint64(288)<<20 | 0 | 0
    = 0x3E00000000 | 0x12000000
    = 0x3E12000000
```

Bit extractions:
- `id = hdr >> 25 = 0x3E12000000 / 2^25 = 29019 * 2^5 + 11... `

Let me redo: `0x3E12000000 = 266,291,609,600`. `266,291,609,600 >> 25 = 266,291,609,600 / 33,554,432 = 7,936 + ...` — let me compute directly:

`(248 * 2^30 + 288 * 2^20) >> 25`:
= `248 * 2^5 + 288 / 2^5` (integer division: `288 >> 5 = 9`)
= `248 * 32 + 9 = 7936 + 9 = 7945`. **Identifier = 7945.** ✓

- `ext = (hdr >> 24) & 1`: `(248*2^6 + 288/2^4) & 1 = (15872 + 18) & 1 = 15890 & 1 = 0`. **Extendable = false.** ✓
- `iterExp = (hdr>>20)&0xf`: `(0x3E12000000 >> 20) & 0xf = 0x3E120 & 0xf = 0`. ✓
- groupThreshold, groupCount stored at bits 15..12 and 11..8 respectively = 0 each → decoded +1 = 1. ✓
- MemberIndex bits 7..4 = 0. MemberThreshold bits 3..0 = 0 → +1 = 1. ✓

Testify re-derived: idx0=906 (line 907), idx1=883 (line 884).
`id = (906*32 + 883/32) = 28992 + 27 = 29019`. ✓
`ext = (906*64 + 883/16) & 1 = (57984 + 55) & 1 = 58039 & 1 = 1`. **Extendable = true.** ✓

**RS1024 GEN constant.** Plan: `[0xe0e040, 0x1c1c080, 0x3838100, 0x7070200, 0xe0e0009, 0x1c0c2412, 0x38086c24, 0x3090fc48, 0x21b1f890, 0x3f3f120]`. Python reference (trezor/python-shamir-mnemonic) confirmed identical. ✓

**Operator precedence.** `chk = (chk&0xfffff)<<10 ^ uint32(v)`. In Go, `<<` has higher precedence than `^`, so this parses as `((chk&0xfffff)<<10) ^ uint32(v)`. Python reference: `chk = (chk & 0xFFFFF) << 10 ^ v` — same semantics (Python `<<` > `^`). ✓

**`rs1024Verify` customization string.** `rs1024Verify` prepends `[]byte(cs)` as `int` values (ASCII byte values), then appends all 20 word indices, and verifies polymod == 1. This matches the SLIP-0039 spec: `polymod([ord(c) for c in cs] + indices) == 1`. ✓

**ext-first → cs-selection → verify order.** The plan extracts `ext` from the header (bits 39..25 through 3..0 packed in the first 4 words), selects the customization string, then calls `rs1024Verify`. This is correct — the ext bit is encoded in the very header being validated; it must be extracted before verification to choose the right `cs`. The SLIP-0039 spec mandates exactly this ordering. ✓

**`uint32` sufficiency for `rs1024Polymod`.** Max intermediate: `(0xfffff << 10) = 0x3FFFFC00` (30 bits) XOR `uint32(v)` (max 10 bits) = at most 30 bits. XOR with GEN (max `0x3090fc48` = 30 bits). The polymod always stays within 30 bits, well within `uint32`. ✓

### Verification Area 2 — `exactWord` correctness

`ClosestWord` is a prefix binary search. When `word` sorts past the last entry, returns `(-1, false)`. `exactWord` guards with `w < 0 || LabelFor(w) != u`: the `w < 0` guard covers the "past end" case; the `LabelFor(w) != u` guard rejects prefix matches (e.g., "ACADEMI" → `ClosestWord` finds "ACADEMIC" and returns `(0, true)` with prefix match, but `LabelFor(0) = "ACADEMIC" != "ACADEMI"` → rejected). ✓

**Panic safety.** `ClosestWord(-1)` is never called — the guard fires before any second `LabelFor` call on an invalid index. `LabelFor` calls `w.valid()` first and returns `""` for invalid `w`, but the `w < 0` guard prevents reaching `LabelFor` in that case. ✓

**Case normalization.** `strings.ToUpper` applied before `ClosestWord`. The `words` string in `wordlist.go` is uppercase ("ACADEMICACID..."), `LabelFor` returns uppercase. Lowercase input (test vectors) and uppercase input (GUI) both match. ✓

**`ParseShare` panic proof.** Empty: `errWrongLength`. 33 words: `errUnsupportedSize`. Non-ASCII: `strings.ToUpper` safe; `ClosestWord` returns `(-1, false)` for out-of-range → `errNotInWordlist`. All 20 valid words: header assembly in `uint64` (no overflow). `LabelFor` is guarded. No panic path exists. ✓

### Verification Area 3 — `uint64` necessity

`uint(248) << 30` on a 32-bit RP2350/TinyGo platform: `248 * 2^30 = 266,287,972,352` which exceeds `uint32` max (`4,294,967,295`). `uint` is 32 bits on RP2350. The `uint64` assembly is not just good practice; it is a correctness requirement. ✓

### Verification Area 4 — gui compile accuracy

**`backup.Seed` fields.** Confirmed at `backup/backup.go:15-23`: `Title string`, `Mnemonic []string`, `ShortestWord int`, `LongestWord int`, `Font *vector.Face`. Plan uses all correctly. ✓

**`backup.EngraveSeed(params, seedDesc)`** signature: `func EngraveSeed(params engrave.Params, plate Seed) (engrave.Engraving, error)`. Plan calls `backup.EngraveSeed(params, seedDesc)`. ✓

**`toPlate(seedSide, params)`** — in-package helper at `gui.go:2648`, confirmed signature `func toPlate(plan engrave.Engraving, params engrave.Params) (Plate, error)`. ✓

**`NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)`** — `NewEngraveScreen` at `gui.go:2338`, `Engrave` at `gui.go:2350`: `func (s *EngraveScreen) Engrave(ctx *Context, th *Colors) bool`. The plan uses 2-arg form correctly. ✓

**`engraveTheme`** — package-level `var engraveTheme Colors` at `theme.go:39`. ✓

**`layoutNavigation` / `NavButton` / `assets.IconBack` / `assets.IconHammer`** — all confirmed in real source. ✓

**`layoutTitle(ctx, dims.X, th.Text, ...)`** — confirmed at `gui.go:1633`: `func layoutTitle(ctx *Context, width int, col color.RGBA, title string) (op.Op, image.Rectangle)`. ✓

**`widget.Labelw(&ctx.B, ctx.Styles.body, dims.X-2*8, th.Text, ln)`** — confirmed at `widget/label.go:16`: `func Labelw(buf *op.Buffer, st text.Style, width int, col color.RGBA, txt string) (op.Op, image.Point)`. ✓

**`leadingSize`** — `const leadingSize = 44` at `theme.go:43`. ✓

**`ErrorScreen.Layout(ctx, th, dims) (op.Op, bool)`** — confirmed at `gui.go:205`. ✓

**`Clickable{Button, AltButton}`** — confirmed at `widget.go:7-16`. ✓

**`Center` button constant** — confirmed at `event.go:27`. ✓

**Imports in `slip39_polish.go`:** `fmt` (used in Sprintf), `image` (used in `image.Pt`), `backup` (EngraveSeed, Seed), `constant` (Font), `assets` (IconBack, IconHammer), `layout` (Rectangle), `op` (Op, Layer, Color), `widget` (Labelw), `slip39words` (Share, ShortestWord, LongestWord). All used, none missing or extra. ✓

### Verification Area 5 — `gui.go` edits / no new imports

**`case 3:` analysis.** Uses: `emptySLIP39Mnemonic` (in-package, `gui.go:503`), `inputSLIP39Flow` (in-package, `gui.go:755`), `strings.Builder` (`strings` already imported at `gui.go:14`), `slip39words.LabelFor`, `slip39words.ParseShare`, `slip39words.Describe` (alias already imported at `gui.go:40`), `showError` (new in-package function in `slip39_polish.go`). No `fmt` needed. ✓

**`break` semantics.** In Go, `break` inside a `switch` exits the switch, falls to the inner `for` loop, which then calls `cs.Choose` again — re-shows the menu. This matches `case 2:`'s natural fall-off behavior. ✓

**Menu with 4 choices.** Plan: `Choices: []string{"12 WORDS", "24 WORDS", "CODEX32", "SLIP-39"}`. Index 3 → `case 3:` in the switch. ✓

**`case slip39words.Share:` type.** `ParseShare` returns `Share` (value, not pointer). `case slip39words.Share:` matches the value type. ✓

**Dormant block replacement.** The dormant block at `gui.go:1810-1840` is correctly identified: the `// TODO: re-enable SLIP39` comment through the closing `// }`. Replacing this with `case slip39words.Share: return engraveSLIP39(ctx, th, scan)` is the correct atomic substitution. ✓

### Verification Area 6 — always-true `engraveSLIP39`

All terminal paths in `engraveSLIP39` return `true`:
- `!confirmSLIP39Flow(...)` (Back) → `return true`. ✓
- `backup.EngraveSeed` error → `showError(...); return true`. ✓
- `toPlate` error → `showError(...); return true`. ✓
- Engrave complete (`Engrave` returns `true`) → `return true`. ✓
- Engrave cancelled (`Engrave` returns `false`) → loop continues, no `false` return. ✓ (The `for` loop only exits when `Engrave` returns `true`.)

This mirrors `engraveCodex32`'s always-`true` contract and prevents the "Unknown format" pitfall on cancel. ✓

### Verification Area 7 — test validity

**`TestParseShare`:** Duckling vector exercises ext=0 (`shamir` cs), checks Identifier=7945 (independently re-derived ✓), all 1-of-1 fields. Testify vector exercises ext=1 (`shamir_extendable` cs), Identifier=29019 (independently re-derived ✓). Uppercase input test is non-vacuous (proves case-insensitivity works from the GUI path). Bad checksum and unknown word tests use real sentinel errors. ✓

**`TestDescribe`:** All 5 sentinels plus nil and unknown. Messages match `Describe` implementation. ✓

**`TestConfirmSLIP39Render`:** `runUI` + first frame. `uiContains` strips spaces from the search string → searches for "id7945" and "member1of1" in lowercase extracted text. Since vector fonts typically have no space glyph, ExtractText omits spaces — existing tests like `TestConfirmCodex32Unshared` use the same `uiContains(c, "id TEST")` pattern which searches for "idtest". Pattern is established and valid. ✓

**`TestEngraveSLIP39BackoutRecognized`:** `Button1` pre-queued → `confirmSLIP39Flow` fires `backBtn.Clicked` on first iteration → returns `false` → `engraveSLIP39` returns `true` → `engraveObjectFlow` returns `true` → test passes. Non-vacuous; directly tests the "always true on cancel" contract. ✓

**Identifier 7945.** Hard-coded from independent re-derivation against the fork's own wordlist. NOT 10027 (the prior saga's misstep). ✓

### Verification Area 8 — scope and atomicity

**Unchanged files.** `codex32/*`, `mdmk.go`, `slip39/wordlist.*`, `gui/scan.go` — the plan makes no changes to these. ✓

**No Shamir/Feistel/PBKDF2/passphrase.** RS1024 is error-detection only. No secret reconstruction. ✓

**Task 1 independence.** `slip39/share.go` + `slip39/share_test.go` compile independently (`go test ./slip39/...`). No gui dependency. ✓

**Task 2 atomicity.** `gui/slip39_polish.go` + `gui/slip39_polish_test.go` + `gui/gui.go` edits must land together: `slip39_polish.go` defines `engraveSLIP39` which is referenced by the `case slip39words.Share:` in `gui.go`; `slip39_polish_test.go` references `confirmSLIP39Flow` and `engraveSLIP39`. All three can compile together as one commit. ✓

**No forward references.** Task 2 depends on Task 1 (uses `slip39words.ParseShare`, `slip39words.Share`, `slip39words.Describe`). Task 1 is committed first. The build sequence respects this. ✓

---

### Findings

**CRITICAL:** None.

**IMPORTANT:** None.

**MINOR:**

**M1 — `engraveTheme` passed by address.** `engraveSLIP39` ends with `NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)`. `engraveTheme` is a `var Colors` (not a pointer), so `&engraveTheme` is correct — `Engrave` takes `*Colors`. This is consistent with the rest of the codebase (e.g., `gui.go:1971` uses `Engrave(ctx, &engraveTheme)`). No issue. ✓

**M2 — `ctx.Styles.body` field access.** `ctx.Styles.body` is an unexported field of `Styles`. `slip39_polish.go` is `package gui` so it has access. ✓ No issue.

**M3 — `op.Layer(frameOps...)` with `append`.** The `append([]op.Op{nav, titleOp}, body...)` call may re-use the underlying slice if capacity allows; the second `append(frameOps, ...)` mutates the same slice. This is standard Go slice usage and safe here — the result is only passed once to `op.Layer`. ✓ No issue.

**M4 — `wordsLong = 33` and other unusual lengths.** The plan correctly gates only `wordsLong = 33` as `errUnsupportedSize` and everything else as `errWrongLength`. A 33-word share will never be accepted. If someone passes 20 words of which some have valid checksums for the 33-word layout, the `wordsShort` check is first and will process them correctly. ✓

**M5 — `slip39_polish.go` filename vs spec's `slip39_confirm.go`.** The spec §4.3 mentions `gui/slip39_confirm.go` as a possible name; the plan uses `gui/slip39_polish.go`. This is a plan-level naming decision not a defect — the spec says "or fold into an existing gui file". The name mirrors `codex32_polish.go`. ✓

---

### Verdict

**GREEN — 0 Critical / 0 Important.**

All eight verification areas pass. The RS1024 implementation is algorithmically faithful to the SLIP-0039 Python reference. Identifier=7945 and Identifier=29019 re-derived independently against the fork's actual wordlist and confirmed. The bit extraction layout is correct for the 40-bit header. Operator precedence in `rs1024Polymod` is correct. `exactWord` correctly enforces exact (not prefix) matching. `uint64` necessity for the 40-bit header on a 32-bit RP2350 target is confirmed. All GUI API calls (`Engrave`, `ErrorScreen.Layout`, `layoutNavigation`, `layoutTitle`, `widget.Labelw`, `backup.Seed`, `backup.EngraveSeed`) match the real signatures in the fork source. `gui.go` requires no new imports. `engraveSLIP39` always returns `true`. Both tests are non-vacuous and will pass. Task atomicity is sound. The plan is cleared for implementation.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | **GREEN — 0C/0I** | Clean first-round pass. Architect re-derived id=7945 (duckling) + id=29019 (testify, ext=1) independently; verified RS1024 GEN + Go `<<`>`^` precedence + `uint32` polymod sufficiency + `uint64` 40-bit-header necessity on 32-bit RP2350; `exactWord` exact-not-prefix + panic-safe; all gui API sigs (EngraveSeed/toPlate/Engrave-2-arg/ErrorScreen.Layout/layoutNavigation/Labelw/backup.Seed); NO new gui.go imports (fmt confined to slip39_polish.go); always-true engraveSLIP39; both tests non-vacuous. 5 Minors all explicitly non-issues (engraveTheme by-addr, ctx.Styles.body in-package, append reuse, 33-word gate, filename). |

GATE PASSED at R0. The recon's pre-verification (scratch decode + RS1024 self-check) + uint64/fmt-in-new-file refinements avoided a code-time repeat of the spec's identifier saga. Proceed to single-implementer TDD.
