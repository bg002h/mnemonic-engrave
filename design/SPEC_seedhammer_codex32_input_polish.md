# SPEC — SeedHammer CODEX32 on-device input polish (Cycle A1)

**Date:** 2026-06-17
**Target repo:** the SeedHammer II firmware fork `bg002h/seedhammer` (Go/TinyGo). Fork-side only — **no upstream PR** (per the user's post-#36 directive; the fork is the maintained line).
**Base:** fork `main` `3c4d3d3` — i.e. **A0 is already done** (Slice 1's BIP-39 polish merged into fork main: progress title, match count, Button3 primary-accept with the keyboard committing on Center only, last-word checksum-candidate keyboard, plus the `runUI`/`ExtractText`/`uiContains` test pattern). Branch this cycle as `feat/codex32-input-polish` off `3c4d3d3`.
**Predecessors:** `design/RECON_seedhammer_slip39_codex32_input.md`, `design/cycle-prep-recon-codex32-slip39.md`, `design/agent-reports/seedhammer-codex32-polish-design-review-R0.md` (this spec folds that review's findings).
**Out of scope (Cycle B):** multi-share k-of-n `Interpolate` recovery. **Also out:** widening the firmware's long-code length gate to BIP-93; changing `Split()`'s threshold remap; anything touching `mdmk.go`.

---

## 1. Goal

Polish the on-device CODEX32 single-share entry UX (already fork-enabled via #34) so it's legible and error-tolerant, in four parts:
1. **Error-class feedback** — show *why* a string is invalid instead of silently toggling the OK button.
2. **Char counter + live field parse** — a length readout (window-aware) + parsed id / threshold / share-index as the user types.
3. **Pre-engrave confirmation** — review the parsed fields before engraving.
4. **Keyboard tidy** — the BIP-39 full-QWERTY layout with the never-valid `b/i/o` statically dimmed.

Button3-accept already comes from A0 (the merge). Fork-side; the spec goes through the opus R0 gate before any code.

## 2. Scope

**In:** C1 (codex32 package API additions) + C2–C5 (gui). **Out:** multi-share recovery (Cycle B); long-code gate widening; `Split()` changes; `mdmk.go`. **Files:** `codex32/codex32.go` (+ `codex32/*_test.go`), `gui/gui.go` (`inputCodex32Flow` only), `gui/codex32_input_test.go`. The shared `Keyboard` widget is touched only via the codex32 keyboard instance (C5) — **`TestWordKeyboardScreen` (BIP-39) must stay green** as the no-cross-contamination guard.

## 3. Background — the flow (post-A0 anchors on `3c4d3d3`)

- `inputCodex32Flow` (`gui/gui.go:672`); keypad alphabet `"1234567890\nqwertyup\nasdfghjk\nlzxcvnm"` (`:673`); per-keystroke `codex32.New(kbd.Fragment)` (`:682`); `okBtn := &Clickable{Button: Button3}` (`:677`, Button3 via A0); static title `"Input Codex32 Share"` (`:721`).
- Accepted share → `case codex32.String:` (`gui/gui.go:1819`) → `backupSeedStringFlow` (`:1928`) engraves the string **verbatim** (TEXT+QR), using only `id` from `Split()`. **Unchanged by this cycle** (single-share verbatim engrave stays; recovery is Cycle B).
- `codex32` package (untouched by Slice 1): `New` (`codex32.go:98`); private sentinels `errInvalidChecksum`/`errInvalidLength`/… (`:24-37`); length consts `shortCodeMinLength=48`/`shortCodeMaxLength=93`/`longCodeMinLength=125`/`longCodeMaxLength=127` (`:41-44`); `partsInner` (the panicking parser — do NOT reuse, `:127`); `Split` (the `0→1` threshold remap — do NOT route through, `:394`); `splitHRP` (returns `("",p1)` if no `1`, `:453`); `feFromRune` (non-panicking, `gf32.go:126`); `setCase`/mixed-case (`checksum.go:132`).

## 4. Design

### 4.1 C1 — codex32 package API additions (the linchpin; land + test first)

**(a) Exported length constants.** Export the four gate constants so the GUI reads them instead of hard-coding 48/93/125/127:
```go
const (
	ShortCodeMinLength = shortCodeMinLength // 48
	ShortCodeMaxLength = shortCodeMaxLength // 93
	LongCodeMinLength  = longCodeMinLength  // 125
	LongCodeMaxLength  = longCodeMaxLength  // 127
)
```

**(b) `Describe` — error classifier.** Keep the sentinels private; add one exported function mapping a `New` error to a short UI label:
```go
// Describe returns a short human label for an error returned by New
// (e.g. "bad checksum", "invalid character"), or "" for nil.
func Describe(err error) string
```
Implemented with `errors.Is` against the private sentinels (`New` wraps with `%w`). Mapping: `errInvalidChecksum`→"bad checksum", `errInvalidLength`→"wrong length", `errInvalidCharacter`→"invalid character", `errInvalidCase`→"mixed case", `errInvalidThreshold`→"bad threshold", `errInvalidShareIndex`→"bad share index", `errIncompleteGroup`→"incomplete group"; unknown non-nil → "invalid".

**(c) `ParsePrefix` — fail-soft partial parser** (fresh code; do NOT reuse `partsInner`, which panics on short/malformed input):
```go
type Fields struct {
	HRP            string // "" until the '1' separator is seen
	Threshold      int    // 0..9; valid only if ThresholdKnown
	ThresholdKnown bool
	Identifier     string // up to 4 chars; valid only if IdentifierKnown
	IdentifierKnown bool
	ShareIndex     rune   // valid only if ShareIndexKnown
	ShareIndexKnown bool
	Unshared       bool   // true iff ShareIndexKnown && ShareIndex is 's'/'S'
}

// ParsePrefix parses whatever header fields are determinable from an
// in-progress codex32 fragment, without panicking. The returned error is
// non-nil ONLY for a determinable violation (bad threshold digit, non-bech32
// char, mixed case, threshold-0-without-S); a merely-too-short fragment
// returns (partialFields, nil). It never attempts to split payload/checksum
// (their boundary depends on the final total length).
func ParsePrefix(frag string) (Fields, error)
```
Determinability (data part = everything after the first `1`, via `splitHRP`): HRP on `1` (validate `ms`/`MS` case-folded); `Threshold = data[0]` at len≥1 (valid ∈ {0,2–9}; `1` → "bad threshold"); `Identifier = data[1:5]` at len≥5 (bech32 chars via `feFromRune`); `ShareIndex = data[5]` at len≥6 (enforce threshold-0 ⇒ index `s`/`S`); payload/checksum never split. Detect mixed case (moot via the force-uppercasing keypad but honest for a package API). Pure, host-testable.

### 4.2 C2 — error-class feedback (`inputCodex32Flow`)

Per `kbd.Update` iteration, call `codex32.New` and `codex32.ParsePrefix` **once each** and thread the results (no redundant calls in the layout block). Show feedback under the entry, timed to determinability:
- **Field errors eagerly:** if `ParsePrefix` returns an error, show `Describe`-style text (e.g. "bad threshold", "invalid character") — these are determinable mid-entry.
- **Checksum verdict only inside an accept window:** `New` returns `errInvalidLength` for any length in 1–47 or 94–124, so suppress that as an *error* (it's "keep typing", shown by C3); only `errInvalidChecksum` on a full valid-length string (48–93 or 125–127) is a true "bad checksum".
- **Valid → no error message** (the OK button on Button3 shows, as today).

### 4.3 C3 — char counter + live field parse (replaces the static title + blob)

Use a **window model** driven by the exported length consts — there is NO single target length (BIP-93: short total 48–93, firmware-long 125–127, dead-zone 94–124 rejected). Readout per current `len(kbd.Fragment)`:
- `< ShortCodeMinLength (48)` → `"N chars"` (no denominator — target undeterminable).
- `48..93` → `"short · N chars"` (acceptable-length window).
- `94..124` → `"N chars — keep typing"` (the firmware's dead zone; NOT shown as an error).
- `125..127` → `"long · N chars"`.
- `> 127` → `"too long"`.
Plus a live field line from `ParsePrefix.Fields`: `"id ABCD · thr 2 · share C"`, each segment appearing once its field is `…Known`. ASCII:
```
+--------------------------------------+
|        short · 41 chars              |
|   id ABCD · thr 2 · share C          |
|   MS12ABCDC W5N4...                  |
+--------------------------------------+
```

### 4.4 C4 — pre-engrave confirmation screen

After accept, before engraving, show a confirm screen (mirrors `SeedScreen.Confirm`). **Branch on the RAW share index from `ParsePrefix`, NOT `Split()`** (`Split()` remaps threshold 0→1 and would mislabel an unshared secret as "1-of-1"):
- index `s`/`S` (Unshared) → `"Unshared secret (S) · id ABCD"` (no threshold number).
- else → `"Share <index> · id ABCD · part of a k-of-n set"` + a clear note: **"engraves THIS share, not a recovered seed"** (recovery is Cycle B).
Buttons: Back (Button1) / Engrave (Button3). ASCII:
```
+--------------------------------------+
|         Confirm Codex32 Share        |
|   id:        ABCD                    |
|   share:     C   (of a k-of-n set)   |   or: "Unshared secret (S)"
|   length:    48 chars                |
|   engraves THIS share, not a seed    |
|   [Back]                  [Engrave]  |
+--------------------------------------+
```

### 4.5 C5 — keyboard tidy (static dimming)

Switch the codex32 keypad to the BIP-39 full-QWERTY+digits layout and **statically dim `b/i/o`** (never valid in codex32). Implementation: build the keyboard with the full alphabet, then set `disabled = true` once on the `b`/`i`/`o` keys after `NewKeyboard` populates `allKeys` — do NOT use `updateValidKeys` (it's BIP-39-wordlist-driven and assumes lowercase `a..z`; codex32 keys are uppercased + include digits). Per-instance, so the BIP-39 keyboard is unaffected. Keep every `codex32.Alphabet` char and the `1` separator enabled. `Valid()`/`adjust()` already skip disabled keys for D-pad nav.

## 5. Error handling / backstops

`codex32.New` remains the sole validity authority (the OK button still gates on `New(...) == nil`). `Split()`'s remap is untouched (the engrave-title path uses only `id`). The verbatim single-share engrave is unchanged. `ParsePrefix`/`Describe` are advisory (display only) — they never gate acceptance.

## 6. Testing (host: `go test ./gui/... ./codex32/...`)

- **C1 (pure, highest value):** `codex32` table tests over BIP-93 vector prefixes — `ParsePrefix` returns the right `Fields`/`…Known`/error at each fragment length (HRP, threshold incl. the `1`-is-bad and threshold-0⇒S rules, id, share index, mixed-case detection, too-short→nil-error); `Describe` returns the right label per sentinel; exported length consts equal the private ones.
- **C2/C3 (GUI):** use the **`runUI` + `ExtractText` + `uiContains` pattern Slice 1 established** (now on the base via A0) — drive `runes(...)` and assert the rendered status text: e.g. a too-short fragment shows `"chars"` and no checksum error; a bad-threshold prefix shows `"bad threshold"`; the field line shows `"id "`/`"thr "`/`"share "` once parsed; the 94–124 dead zone shows `"keep typing"` not an error. **No special test seam needed** (the status is rendered text).
- **C4:** drive to the confirm screen; BIP-93 vector 1 (`ms10test…`, threshold 0, index S) → asserts `"Unshared secret"`; a `ms12…` share → asserts `"Share"` + the "not a recovered seed" note.
- **C5:** assert the codex32 keyboard's `b`/`i`/`o` keys have `disabled==true` and every `codex32.Alphabet` char (+`1`) is enabled; regression-assert a BIP-39 `NewKeyboard(wordKeys)` has `b/i/o` ENABLED (no cross-contamination). Keep `TestWordKeyboardScreen` + `codex32_input_test` green.

## 7. Versioning / commits

Firmware version is `-ldflags`-injected (no source bump). Commits on `feat/codex32-input-polish` (off `3c4d3d3`), signed + DCO, author Brian Goss. Fork-side; no upstream PR.

## 8. Resolved decisions

- Scope: all four polish items (user-approved). Multi-share recovery → Cycle B.
- A0 (Slice-1 merge) **done** (`3c4d3d3`); this cycle branches off it.
- C1 API: **`Describe` + exported length consts**, sentinels stay private (architect rec).
- C3: **window model**, no single denominator (architect IMP-1, BIP-93-verified).
- C4: branch on **raw index via `ParsePrefix`**, not `Split()` (architect IMP-3).
- C5: **static dimming** at construction, not `updateValidKeys` (architect MIN-1).
- C2/C3 testability: **`runUI`+`ExtractText`** (no new seam) — supersedes the design-review's "add a seam" note (that pattern is now on the base).

## 9. Process note

Per project standard: this spec MUST pass the opus-architect **R0 gate to 0C/0I before any code** (fold → persist verbatim to `design/agent-reports/` → re-dispatch until GREEN). Implementation = single subagent per task (C1 first) + two-stage review, then a mandatory whole-diff adversarial execution review.
