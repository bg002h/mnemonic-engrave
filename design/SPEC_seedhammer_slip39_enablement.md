# SPEC — SeedHammer SLIP-39 share entry + verbatim engrave (Cycle C, Tier 1)

**Date:** 2026-06-18
**Target repo:** the SeedHammer II firmware fork `bg002h/seedhammer` (Go/TinyGo). Fork-side only — **no upstream PR** (post-#36 directive).
**Base:** fork `main` `9b0a02c` (post-Cycle-A1/B). Branch `feat/slip39-entry-engrave` off `9b0a02c`.
**Predecessors:** `design/cycle-prep-recon-slip39-enablement.md` (verified vs `9b0a02c` + SLIP-0039), `design/RECON_seedhammer_slip39_codex32_input.md`.
**Scope decision (user-chosen):** **Tier 1 = single-share entry + verbatim engrave, NO secret recovery.** This mirrors the codex32 A1 progression (entry+engrave) and re-enables the firmware's original dormant SLIP-39 design.
**Out of scope (future Cycle D):** secret *recovery* (combining k shares — GF(256) Shamir + 4-round Feistel + PBKDF2 + two-level group/member combine); passphrase entry; 256-bit/33-word shares (plate-fit unvalidatable without hardware); multi-group collection; NFC-scanned shares; any `go-slip39` dependency.

---

## 1. Goal

Re-enable on-device SLIP-39 share **entry and verbatim engraving** (durable metal backup of a SLIP-39 share — the same use case codex32 single-share serves), gated by a correct in-tree **RS1024 checksum validation** + share-metadata parse. The device does NOT reconstruct the master secret (that is Tier-2/Cycle-D). 128-bit (20-word) shares only.

The firmware already contains the dormant entry UX (`inputSLIP39Flow` + helpers, active source but unreachable) and a wordlist-only `slip39` package; this cycle adds the missing `ParseShare`/`Share` API (RS1024 + bit-field decode — **error-detection crypto only, no secret handling**) and re-wires the menu/entry/engrave call sites.

## 2. Scope

**In:**
- **C1 — `slip39` package API:** an in-tree **RS1024 checksum over GF(1024)** + `ParseShare(mnemonic string) (Share, error)` that validates a 20-word share (word count, RS1024 checksum with the `ext`-selected customization string, field ranges) and returns decoded metadata. Gated against the **official SLIP-0039 test vectors**.
- **C2 — gui re-enablement:** re-enable the `"SLIP-39"` menu choice, the `case 3:` keypad-entry path (`inputSLIP39Flow` → `slip39.ParseShare`), and the `case slip39.Share:` verbatim engrave branch; fix the dormant code's references to use the real (aliased) package.
- **C3 — pre-engrave confirm:** a light confirm screen (mirror codex32's `confirmCodex32Flow`) showing the parsed share metadata (id / member index / member threshold / word count) with Back/Engrave.

**Out:** everything in the "Out of scope" line above. `codex32`/`mdmk.go` untouched. The wordlist data (`slip39/wordlist.*`) is unchanged.

**Files:** `slip39/share.go` (+ `slip39/share_test.go`) — new RS1024 + `ParseShare`/`Share`; `slip39/slip39.go` (only if a tiny helper is needed); `gui/gui.go` (re-enable menu + `case 3:` + `case slip39.Share:`; the dormant `inputSLIP39Flow` is already present); `gui/slip39_confirm.go` (new, the confirm flow — or fold into an existing gui file); `gui/*_test.go`. `gui/scan.go` stays as-is (NFC SLIP-39 remains disabled). `slip39/wordlist.*`, `codex32/*`, `mdmk.go` unchanged.

## 3. Background — the dormant state on `9b0a02c`

- `slip39` package is **wordlist-ONLY**: exports `Word`, `Mnemonic`, `NumWords`, `LabelFor`, `ClosestWord`, `ShortestWord=4`, `LongestWord=8`, the 1024-word list. **No** `ParseShare`/`Share`/RS1024/Shamir. The package is imported in `gui/gui.go` aliased as `slip39words "seedhammer.com/slip39"` (`gui.go:40`).
- Dormant (active source, unreachable) entry UX: `inputSLIP39Flow` (`gui.go:755`), `emptySLIP39Mnemonic` (`gui.go:503`), `completeSLIP39Word` (`gui.go:922`), `updateValidSLIP39Keys` (`gui.go:966`).
- Commented call sites: menu choice `"12 WORDS", "24 WORDS", "CODEX32" /* , "SLIP-39" */` (`gui.go:1983`); `case 3:` entry block (`gui.go:2002-2019`, calls `emptySLIP39Mnemonic(20)`, `inputSLIP39Flow`, then `slip39.ParseShare(...)`); `case slip39.Share:` engrave branch (`gui.go:1810-1840`, `const maximumLength = 20` at `:1814`, engraves via `backup.EngraveSeed`). **These reference a bare `slip39.ParseShare`/`slip39.Share`/`scan.Identifier/.MemberIndex/.MemberThreshold` that does not exist** — the gui's import alias is `slip39words`, so the re-enabled code must call `slip39words.ParseShare` etc. (and the new API must export those symbols + fields).
- `gui/scan.go:61-65` SLIP-39 NFC block — structurally stale (`res.Content` no longer exists); **stays disabled** (sensitive material is hand-typed, not over RF — consistent with the codex32/ms1 posture).
- Engrave layout: `backup.EngraveSeed(params, backup.Seed{Mnemonic []string, ShortestWord, LongestWord, ...})` lays words in up to 3 columns; the BIP-39 path already engraves 24 words; a 20-word SLIP-39 share fits the same envelope.

## 4. Design

### 4.1 C1 — `slip39` RS1024 + `ParseShare` (the only crypto; land + test first)

**RS1024 checksum (GF(1024), error-detection only — NOT secret handling).** Per SLIP-0039: a 3-word/30-bit Reed-Solomon checksum. `rs1024_polymod(values)` with generator `GEN = [0xe0e040,0x1c1c080,0x3838100,0x7070200,0xe0e0009,0x1c0c2412,0x38086c24,0x3090fc48,0x21b1f890,0x3f3f120]`; verify = `polymod([cs bytes...] + dataWordIndices) == 1`, where the customization string `cs = "shamir"` if `ext==0` else `"shamir_extendable"` (the 2024 extendable-backup amendment; modern shares use `ext==1`). The exact reference algorithm + the constant are pinned in the implementation plan.

**`Share` struct + `ParseShare`:**
```go
// Share is a parsed SLIP-39 share's header metadata (Tier 1: no secret value
// reconstruction). All fields are decoded from the share's bit layout; the
// RS1024 checksum has been verified.
type Share struct {
	Mnemonic        []string // the share words (length 20 for 128-bit)
	Identifier      int      // 15-bit random identifier (shared across a set)
	Extendable      bool     // ext flag (selects the RS1024 customization string)
	IterationExp    int      // iteration exponent (4 bits)
	GroupIndex      int      // 4 bits
	GroupThreshold  int      // decoded (stored value + 1)
	GroupCount      int      // decoded (stored value + 1)
	MemberIndex     int      // 4 bits
	MemberThreshold int      // decoded (stored value + 1)
}

// ParseShare validates a SLIP-39 share mnemonic (Tier 1, 128-bit/20-word only)
// and returns its decoded header. It checks: exactly 20 words, all in the SLIP-39
// wordlist, a valid RS1024 checksum (customization string per the ext bit), and
// field-range sanity. It does NOT reconstruct or decrypt any secret. Returns a
// non-nil error (a classifiable sentinel) on any failure.
func ParseShare(mnemonic string) (Share, error)
```
Decode (order matters — R0 M4): (1) split the mnemonic; reject ≠20 words (256-bit/33-word → a clear "256-bit not supported" sentinel; plate-fit unvalidated); (2) map each word → its 10-bit index via an **exact** wordlist lookup (the in-package `index` map — NOT `ClosestWord`'s fuzzy match), **normalizing each word to uppercase first (`strings.ToUpper`)** — the in-tree wordlist is UPPERCASE (`wordlist.go`'s `words` = `"ACADEMICACID…"`, so `LabelFor` returns uppercase) while the official SLIP-0039 test vectors are lowercase; without this, the §6 vector tests would fail every word as not-in-wordlist (R3 fix). An unknown word → not-in-wordlist sentinel; (3) pack indices → bitstream; (4) **extract the `ext` bit (bit 15) FIRST**, then verify RS1024 with `cs = "shamir"`(ext=0)/`"shamir_extendable"`(ext=1) → bad-checksum sentinel on mismatch; (5) decode the header fields at the SLIP-0039 offsets (id 15b, ext 1b, iterationExp 4b, groupIndex 4b, groupThreshold 4b, groupCount 4b, memberIndex 4b, memberThreshold 4b; then the 130-bit padded share value = 2 zero pad-bits + 128-bit value; then the 30-bit/3-word checksum — totals 200 bits = 20×10). Thresholds/counts are stored as value−1 (decode `+1`). Error sentinels (private) + a `Describe`-style classifier for the GUI: bad checksum, not in wordlist, wrong length, unsupported size. **Pure, host-testable against the official vectors.** `Share.Mnemonic` = the 20 input words (for verbatim engrave).

### 4.2 C2 — gui re-enablement (the dormant code is STALE — do NOT uncomment verbatim; use the concrete substitutions below)

The dormant blocks were written against an old API and will NOT compile as-is (R0 C1/C2/I1/I2/I3): they call `scan.Words() (w, err)` (the new `Share` has `Mnemonic []string`, no `Words()`), a stale 3-arg `.Engrave(ctx, ops, &engraveTheme)` (real signature is 2-arg `Engrave(ctx, th)`), a 20-char title that overflows `backup.MaxTitleLen=18`, and `fmt.Sprintf` while `gui.go` does not import `"fmt"`. Apply these exact substitutions:

- **Import:** add `"fmt"` to `gui/gui.go`'s import block (used by the title; alternatively build the title with `strconv` — the plan picks one).
- **Menu (`gui.go:1983`):** `Choices: []string{"12 WORDS", "24 WORDS", "CODEX32", "SLIP-39"}`. (The `ChoiceScreen` lead text "Choose number of words" becomes slightly loose with a 4th non-word-count option — acceptable; M3.)
- **Entry (`case 3:`, `gui.go:2002-2019`):** re-enable; the dormant block already builds the space-joined word string and calls `ParseShare` — the only fixes are the alias (`slip39.ParseShare` → `slip39words.ParseShare`) and surfacing errors:
  ```go
  case 3:
      m := emptySLIP39Mnemonic(20)
      if !inputSLIP39Flow(ctx, th, m, 0) {
          continue
      }
      words := make([]string, len(m))
      for i, w := range m {
          words[i] = slip39words.LabelFor(w)
      }
      share, err := slip39words.ParseShare(strings.Join(words, " "))
      if err != nil {
          showError(ctx, th, "Invalid SLIP-39 share", slip39words.Describe(err)) // dismissible, then re-loop
          continue
      }
      return share, true
  ```
  (`inputSLIP39Flow(ctx, th, m, 0) bool` fills `m slip39words.Mnemonic`; the words are joined into the space-separated string `ParseShare` consumes — this is exactly the dormant conversion, so **R0 I3 is resolved by keeping `ParseShare(string)` per §4.1** (string is also what the official vectors are, so it test-gates trivially). `gui.go` **already imports `"strings"`** (`gui.go:14`); only `"fmt"` needs adding — R1 m1.)
- **Error modal helper (R1 m2):** `showError(...)` above is a NEW small helper to add (Cycle-B's `showCodex32Error` hard-codes the title `"Invalid share"`, so don't reuse it directly): `func showError(ctx *Context, th *Colors, title, msg string)` running the standard dismissible `ErrorScreen` loop (`&ErrorScreen{Title: title, Body: msg}`; `for !ctx.Done { d, ok := errScr.Layout(ctx, th, dims); if ok { return }; ctx.Frame(op.Layer(d, <background>)) }`). Call it `showError(ctx, th, "Invalid SLIP-39 share", slip39words.Describe(err))` at entry and `showError(ctx, th, "Too large", "Share doesn't fit a plate.")` in the engrave helper.
- **Engrave:** route through a helper that ALWAYS returns `true` (recognized), mirroring Cycle-A1's `engraveCodex32` so a cancel/fit-failure never falls to the caller's `scanUnknownFormat` ("Unknown format") path:
  ```go
  case slip39words.Share:
      return engraveSLIP39(ctx, th, scan)
  ```
  ```go
  func engraveSLIP39(ctx *Context, th *Colors, scan slip39words.Share) bool {
      if !confirmSLIP39Flow(ctx, th, scan) { // C3; Back → recognized, declined
          return true
      }
      seedDesc := backup.Seed{
          Mnemonic:     scan.Mnemonic, // verbatim; ParseShare guaranteed 20 words
          ShortestWord: slip39words.ShortestWord,
          LongestWord:  slip39words.LongestWord,
          Title:        fmt.Sprintf("%d #%d/%d", scan.Identifier, scan.MemberIndex+1, scan.MemberThreshold), // <=18 chars (max "32767 #16/16" = 12); R0 I1
          Font:         constant.Font,
      }
      params := ctx.Platform.EngraverParams()
      seedSide, err := backup.EngraveSeed(params, seedDesc)
      if err != nil {
          showError(ctx, th, "Too large", "Share doesn't fit a plate.") // recognized but unfittable
          return true
      }
      plate, err := toPlate(seedSide, params)
      if err != nil {
          showError(ctx, th, "Too large", "Share doesn't fit a plate.")
          return true
      }
      for {
          if NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme) { // 2-arg; R0 C2
              return true
          }
      }
  }
  ```
  The dormant `const maximumLength = 20` + `len(w) > maximumLength` guard is DROPPED — `ParseShare` already restricts to 20-word/128-bit (a 33-word share is rejected at entry with the unsupported-size sentinel). `showError` is the new 4-arg `showError(ctx, th, title, msg)` helper defined in the "Error modal helper" bullet above (R2 m1 — not a 3-arg form).

### 4.3 C3 — pre-engrave confirm

A light confirm screen modeled on `confirmCodex32Flow` (Back=Button1 / Engrave=Button3): title "Confirm SLIP-39 Share", lines for `id`, `member <i> of <memberThreshold>`, `group <gi>` (if groupCount>1), word count. Returns engrave/back. (No Recover action — recovery is Tier-2/Cycle-D.)

## 5. Error handling / backstops

`ParseShare` is the validity authority for entry (the OK path requires `ParseShare == nil`). RS1024 is error-detection only — a bug rejects/accepts a share (UX/correctness), it never handles or leaks a secret. The share is engraved **verbatim** (no decryption, no master-secret derivation). Hand-typed on the air-gapped touchscreen (no RF). Passphrase never entered (recovery-only); the master secret is never reconstructed, so the silent-wrong-passphrase footgun and the "raw BIP-32 seed vs BIP-39" ambiguity do not arise in Tier 1.

## 6. Testing (host: `go test ./slip39/... ./gui/...`)

- **C1 (pure, highest value):** `slip39` table tests against the **official SLIP-0039 vectors** (`trezor/python-shamir-mnemonic/vectors.json`): every 20-word mnemonic in a *valid* vector set `ParseShare`-OK with the expected decoded metadata; a checksum-corrupted mnemonic (flip one word) → bad-checksum sentinel; a 33-word share → unsupported-size sentinel; a non-wordlist word → not-in-wordlist sentinel; RS1024 verified for both `ext==0`/`ext==1` customization strings.
- **Concrete anchor vector (R1 C1 — corrected):** write a NEW test in `slip39/share_test.go` (note: `backup_test.go`'s `TestSLIP39` is an engrave golden-image test — it never calls `ParseShare`, and its title literal `"7945 #1 1/1"` is a hand-written display string, **NOT** a decoded field; do not assert against it). Use the same `vectors.json` "valid mnemonic without sharing (128 bits)" share (`"duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard"`). Robust assertions for this 1-of-1 single-group share: `GroupThreshold==1, GroupCount==1, MemberIndex==0, MemberThreshold==1`. Its **decoded `Identifier` is 7945** — verified against the fork's `slip39/wordlist.txt`: "duckling" is 0-based index **248** (line 249), "enlarge" is **288** (line 289), so `Identifier = (idx0<<10 | idx1) >> 5 = (248<<10 | 288) >> 5 = 254240 >> 5 = 7945` (the top 15 bits, above the 1-bit ext + 4-bit iteration exponent). (History, R1→R2: an R1 round mis-derived this as 10027 using indices 313/360 from a *different* wordlist; R2 corrected it against the fork's actual list. The `backup_test.go` title "7945 #1 1/1" happens to be correct here — but **the plan MUST still precompute each embedded vector's expected header fields against an independent reference (the `trezor/python-shamir-mnemonic` decoder) using the FORK's wordlist indices, and hard-code the verified values** — `vectors.json` exposes only the master secret, and the 10027 misstep is exactly why: never trust a number without re-deriving it against the real wordlist.)
- **C2/C3 (gui, `runUI`+`ExtractText`+`uiContains`):** drive the menu to SLIP-39 (index 3), enter a valid 20-word share, assert the confirm shows the id/member info and the share engraves; assert an invalid share (bad checksum) surfaces the error label. Keep all codex32 (A1/B) + BIP-39 guard tests green.

## 7. Versioning / commits

Firmware version `-ldflags`-injected (no source bump; next tag would be a MINOR — new input capability). Commits on `feat/slip39-entry-engrave` (off `9b0a02c`), signed (SSH) + DCO, author Brian Goss. Fork-side; no upstream PR. Stage explicit paths. README's "SLIP-39 disabled" framing updated when this ships.

## 8. Resolved decisions

- **Tier 1 (entry + verbatim engrave), NOT recovery** — user-chosen ("Proceed" on the recommended tier). Recovery = future Cycle D (the XL, security-critical Shamir/Feistel part + the crypto-source A/B/C decision).
- **In-tree RS1024**, no `go-slip39` (it self-describes as unaudited/not-hardened, and its repo is 404 → proxy-vendor-only; an in-tree checksum needs none of that and addresses the maintainer's footprint objection). RS1024 is error-detection, not secret-handling — low audit stakes, gated by official vectors.
- **128-bit / 20-word only** (matches the dormant `emptySLIP39Mnemonic(20)`; 256-bit/33-word plate-fit is unvalidatable without hardware → deferred).
- **Keypad entry only**; NFC SLIP-39 stays disabled (sensitive material hand-typed, consistent with codex32/ms1 posture).
- **Confirm screen added** (C3) — addresses the maintainer's "not polished" objection, mirrors codex32 A1.
- **`inputSLIP39Flow`'s static "Input Words" title is kept** as-is for Tier 1 (R0 M1) — functional; a dynamic "Word N of 20" progress title (à la BIP-39's `inputWordsFlow`) is a deferred nice-to-have, not in this cycle.

**SPEC R0 GATE: PASSED (GREEN — 0C/0I at R4).** Loop R0→R4 (reviews persisted to design/agent-reports/seedhammer-slip39-spec-review-R{0..4}.md): caught stale dormant code (R0), the anchor-vector identifier saga settled at 7945 verified vs the fork wordlist (R1/R2), and the uppercase-normalization sentence (R3). Cleared for implementation.

**R0 OUTCOME (2026-06-18):** R0 = RED (2 Critical / 3 Important) — the dormant code was stale (`scan.Words()`, 3-arg `.Engrave`, 20-char title > `MaxTitleLen=18`, missing `fmt` import, §4.1/§4.2 `ParseShare`-input contradiction). All folded above with concrete substitutions (§4.2) + the engrave routed through an always-`true` `engraveSLIP39` helper (avoids the A1 "Unknown format"-on-cancel pitfall) + RS1024 decode-ordering pinned (§4.1) + a concrete anchor vector (§6). Re-dispatched R1.

## 9. Process note

Per project standard: this spec MUST pass the opus-architect **R0 gate to 0C/0I before any code** (fold → persist verbatim to `design/agent-reports/` → re-dispatch until GREEN). Then plan → plan R0 → single-implementer subagent TDD in a worktree → mandatory whole-diff adversarial execution review. Proceeding autonomously (user directive); the architect gates + execution review are the quality controls.
