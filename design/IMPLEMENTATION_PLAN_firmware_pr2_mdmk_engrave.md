# Firmware PR2 — BCH-validated md1/mk1 engraving — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make SeedHammer II recognize an `md1`/`mk1` string over NFC, **BCH-validate it** (reject corruption), and engrave it verbatim as a text/QR plate — the same plate flow descriptors use.

**Architecture:** Add a md/mk BCH verifier *inside* the `codex32` package (`codex32/mdmk.go`) that reuses the package-private `engine`/`fe`/`inputHRP`/`inputData`/`isValid` with our NUMS-derived target residues and our initial residue. Wire it into `gui/scan.go` (a new branch after codex32) returning a small typed wrapper, and add a `gui` engrave flow mirroring `descriptorFlow` (TEXT+QR / TEXT / QR-ONLY → `backup.EngraveText`). No semantic decode — the string is engraved verbatim; the verifier only rejects corruption.

**Tech Stack:** Go 1.25+ (host `go test`; `~/.local/go/bin/go` is go1.26.4). Must be TinyGo-compatible (this runs on-device) — **no `math/big`** in `codex32/mdmk.go`; use `uint64` (regular) and a hi/lo `uint64` pair (long). Work in the fork `/scratch/code/shibboleth/seedhammer` (origin `bg002h/seedhammer`, upstream `seedhammer/seedhammer` `main`).

> **Design source:** `design/SPEC_seedhammer_engrave.md` §7 PR2 (architect R-loop GREEN). **Per the iterative-architect-review standard, this plan-doc must pass its own architect R0 gate (0C/0I) before any code.** Planning artifact lives in `mnemonic-engrave`; code lives in the `seedhammer` fork.
>
> **`go` PATH:** prefix commands with `export PATH="$HOME/.local/go/bin:$PATH"`.
>
> **Plan status:** architect gate **GREEN** (plan-R0 → plan-R1, 0C/0I; reports in `design/agent-reports/firmware-pr2-mdmk-plan-R{0,1}-review.md`). Eligible for execution. plan-R0 caught 3 Criticals (`inputData` type, MK_LONG `uint64` overflow, `NewErrorScreen` misuse) + 2 Importants (consts provenance, mk length-gate stub) — all folded.
>
> **⚠️ THE #1 RISK (read before Task 2):** the md/mk BCH initial residue is `POLYMOD_INIT = 0x23181b3`, **NOT** codex32's `1`. If you copy `newShortChecksum`'s `residue` field and only swap `target`, every check is wrong. The parity test (Task 2) MUST use **Rust-sourced** golden vectors (never Go-self-generated ones), or this bug passes silently.

---

## File Structure

| File | Change | Responsibility |
|---|---|---|
| `codex32/mdmk.go` (new) | Create | `ValidMD(s string) bool`, `ValidMK(s string) bool` — md/mk BCH verify reusing the package-private engine. TinyGo-safe (uint64 only). |
| `codex32/mdmk_test.go` (new) | Create | Parity test vs Rust-sourced golden md1/mk1 vectors (regular + long), positives + negatives (single-char tamper, all-zeros). |
| `gui/scan.go` | Edit | Add the md1/mk1 branch after `codex32.New`; define the scan-result wrapper type. |
| `gui/gui.go` | Edit | `engraveObjectFlow` case + `validateMdmk` + `mdmkFlow` (mirror `validateDescriptor`/`descriptorFlow`). |
| `gui/scan_test.go` | Edit | `TestScan` table entries for a valid md1 and mk1 string. |
| `gui/gui_test.go` | Edit | `TestMdmkEngraveScreen` + an `ErrTooLarge` test. |

No change to `backup/backup.go` — `EngraveText`/`Text`/`Paragraph` are reused verbatim.

---

## Task 1: Branch off upstream main

- [ ] **Step 1:** `cd /scratch/code/shibboleth/seedhammer && git fetch upstream && git checkout -b feat/engrave-mdmk upstream/main`
Expected: branch `feat/engrave-mdmk` at `upstream/main` (independent of PR1's branch; the files/lines don't conflict).

---

## Task 2: md/mk BCH verifier in `codex32/mdmk.go` (the crux)

**Files:** Create `codex32/mdmk.go`, `codex32/mdmk_test.go`.

> **Approach (verified by recon):** reuse the existing engine via `newShortChecksum().generator` (regular) and `newLongChecksum().generator` (long) — the generator polynomial is provably identical to ours. Build an `engine{generator, residue, target}` where `residue` is seeded to `POLYMOD_INIT` (NOT `1`) and `target` is our NUMS const unpacked MSB-first into 5-bit symbols. Then mirror `codex32.New`'s verify path: split HRP, feed `inputHRP` + `inputData`, check `isValid()`. md1 = regular only; mk1 = regular OR long (pick by data length, mirroring `codex32.New`).

- [ ] **Step 0 (SPIKE — do this FIRST, the family "trial-compile" caveat):** Before writing the full verifier, write a throwaway `main`/test that: (a) reads how `codex32.New` splits HRP + converts to 5-bit symbols + drives the engine (`codex32/codex32.go:98-124`, `splitHRP` `:453`, `inputHRP`/`inputData`/`isValid`), and (b) confirms feeding HRP `md` + the symbols of a **known Rust-generated `md1` string** through an engine with `residue=POLYMOD_INIT`, `target=MD_REGULAR_CONST(unpacked)`, `generator=newShortChecksum().generator` yields `isValid()==true`, while codex32's `residue=1` yields false. This nails the init-residue + unpacking before you build the API. Iterate until the known-good md1 validates.

- [ ] **Step 1: Write the parity test (`codex32/mdmk_test.go`) — RED.**

Use **Rust-sourced** golden vectors (do NOT generate them in Go):
- md1 regular (md-codec 0.36): `md1yqpqqxqq8xtwhw4xwn4qh`
- mk1 regular (mk-codec `test_vectors/v0.1.json`): `mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x`
- mk1 long: source one long-code mk1 string from `mnemonic-key/crates/mk-codec/src/test_vectors/v0.1.json` (a card whose chunk uses the 15-symbol long code) and pin it. If none is present, generate one with `mk` and record it in the test comment with provenance.

```go
package codex32

import "testing"

func TestMDMKValid(t *testing.T) {
	for _, tc := range []struct {
		name  string
		s     string
		valid func(string) bool
		want  bool
	}{
		{"md1 regular ok", "md1yqpqqxqq8xtwhw4xwn4qh", ValidMD, true},
		{"mk1 regular ok", "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x", ValidMK, true},
		// {"mk1 long ok", "<pinned long mk1 from v0.1.json>", ValidMK, true},
		{"md1 wrong hrp", "mk1yqpqqxqq8xtwhw4xwn4qh", ValidMD, false},
		{"mk1 wrong hrp", "md1yqpqqxqq8xtwhw4xwn4qh", ValidMK, false},
	} {
		if got := tc.valid(tc.s); got != tc.want {
			t.Errorf("%s: got %v want %v", tc.name, got, tc.want)
		}
	}
}

func TestMDMKRejectsTamper(t *testing.T) {
	// Flip the last data char of a valid md1; BCH must reject (md1 = pure verify,
	// no correction — unlike mk-codec's decode_string, codex32-style verify does
	// not auto-correct).
	const good = "md1yqpqqxqq8xtwhw4xwn4qh"
	bad := []byte(good)
	last := bad[len(bad)-1]
	if last == 'q' {
		bad[len(bad)-1] = 'p'
	} else {
		bad[len(bad)-1] = 'q'
	}
	if ValidMD(string(bad)) {
		t.Error("tampered md1 accepted")
	}
}

func TestMDMKRejectsAllZeros(t *testing.T) {
	// NUMS anti-trivial property: an all-"q" (zero) data+checksum must not self-validate.
	if ValidMD("md1qqqqqqqqqqqqqqqqqqqqqq") {
		t.Error("all-zero md1 self-validated")
	}
}
```

- [ ] **Step 2:** `export PATH="$HOME/.local/go/bin:$PATH" && cd /scratch/code/shibboleth/seedhammer && go test ./codex32/ -run TestMDMK` → FAIL (undefined `ValidMD`/`ValidMK`).

- [ ] **Step 3: Implement `codex32/mdmk.go`.**

```go
package codex32

// md1 (HRP "md") and mk1 (HRP "mk") reuse codex32's BCH machinery with
// NUMS-derived target residues and a NON-codex32 initial residue.
//
// CRITICAL: the initial residue is POLYMOD_INIT = 0x23181b3, NOT codex32's 1.
// Copying newShortChecksum's residue field and only swapping target is WRONG.
//
// Constants (sources of truth, verified against the Rust codecs):
//   MD regular target = 0x0815c07747a3392e7   (descriptor-mnemonic md-codec bch.rs)
//   MK regular target = 0x1062435f91072fa5c   (mnemonic-key consts.rs:18)
//   MK long target    = 0x41890d7e441cbe97273 (mnemonic-key consts.rs:21; 75 bits)
//   POLYMOD_INIT      = 0x23181b3             (both, md-codec bch.rs)
// md1 is regular-only (md-codec dropped the long code).

// unpackSyms returns n 5-bit GF(32) symbols of (hi<<64 | lo), MSB-first.
// hi is 0 for any value < 2^64 (all regular consts + POLYMOD_INIT).
func unpackSyms(hi, lo uint64, n int) []fe {
	out := make([]fe, n)
	for i := 0; i < n; i++ {
		shift := uint(5 * (n - 1 - i))
		var v uint64
		if shift >= 64 {
			v = hi >> (shift - 64)
		} else {
			v = (lo >> shift) | (hi << (64 - shift))
		}
		out[i] = fe(v & 0x1f)
	}
	return out
}

// verifyMDMK validates s against the given HRP, generator, target, and the
// md/mk initial residue (POLYMOD_INIT), for an n-symbol checksum (13 regular /
// 15 long). Mirrors codex32.New's verify path: split HRP, feed the data STRING
// to inputData (the engine decodes runes internally — do NOT pre-decode to
// []fe), then isValid. Pure verify, no error correction.
func verifyMDMK(s, hrp string, generator, target []fe, n int) bool {
	gotHRP, data := splitHRP(s) // codex32.go:453 -> (hrp string, data string)
	if gotHRP != hrp {
		return false
	}
	e := &engine{
		generator: generator,
		residue:   unpackSyms(0, 0x23181b3, n), // POLYMOD_INIT — NOT codex32's 1
		target:    target,
	}
	if err := e.inputHRP(hrp); err != nil { // confirm inputHRP's return against codex32.New
		return false
	}
	if err := e.inputData(data); err != nil { // C1: data is a STRING, not []fe
		return false
	}
	return e.isValid()
}

// ValidMD reports whether s is a structurally valid, BCH-correct md1 string
// (regular code only — md-codec dropped the long code).
func ValidMD(s string) bool {
	return verifyMDMK(s, "md", newShortChecksum().generator,
		unpackSyms(0, 0x0815c07747a3392e7, 13), 13)
}

// ValidMK reports whether s is a structurally valid, BCH-correct mk1 string,
// regular (13-symbol) or long (15-symbol). The regular/long selection is by
// data-part length, per mk-codec's bch_code_for_length (string_layer/bch.rs:117)
// — derive mk's own thresholds from that function; do NOT copy codex32's
// 48-93/125-127 bounds (mk's total lengths differ).
func ValidMK(s string) bool {
	if mkIsLong(s) { // fill from mk-codec bch_code_for_length; gate by data-part length
		return verifyMDMK(s, "mk", newLongChecksum().generator,
			unpackSyms(0x418, 0x90d7e441cbe97273, 15), 15) // C2: correct hi/lo split of MK_LONG_CONST
	}
	return verifyMDMK(s, "mk", newShortChecksum().generator,
		unpackSyms(0, 0x1062435f91072fa5c, 13), 13)
}
```

> **Iteration points (gated by the Rust-sourced parity test):**
> (1) **`splitHRP`/`inputHRP`/`inputData` exact use** — confirm `splitHRP(s) (string,string)` and `inputHRP`'s return against `codex32.New` (`codex32/codex32.go:98-124`); pass the data **string** to `inputData` (the engine decodes runes internally — no `[]fe` decode). Also add codex32.New-style length-bracket rejection.
> (2) **`mkIsLong(s)`** — derive mk's regular/long selection from mk-codec's `bch_code_for_length` (`mnemonic-key/crates/mk-codec/src/string_layer/bch.rs:117`), by **data-part length**; do NOT copy codex32's 48-93/125-127 bounds (mk's total lengths differ).
> (3) **Constants** — `POLYMOD_INIT=0x23181b3`, md target `0x0815c07747a3392e7`, mk targets `0x1062435f91072fa5c`/`0x41890d7e441cbe97273`, and `GEN_REGULAR == newShortChecksum().generator` were verified against the Rust source in recon (md-codec `src/bch.rs:17`, mk-codec `src/consts.rs:18,21`). **Do not trust hand-transcribed values — the parity test against Rust-GENERATED golden vectors (never Go-self-generated) is the hard gate, and it is the only guard against the init-residue bug.**

- [ ] **Step 4:** `go test ./codex32/ -run TestMDMK -v` → PASS (all positives valid, wrong-HRP/tamper/all-zeros rejected). If md1 fails, re-check the init residue (Step 0 spike).

- [ ] **Step 5: Commit** (`git commit -s` — DCO; no Co-Authored-By on upstream commits): `codex32: add md1/mk1 BCH verification (ValidMD/ValidMK)`.

---

## Task 3: Scanner recognition (`gui/scan.go`)

**Files:** Edit `gui/scan.go`, `gui/scan_test.go`.

- [ ] **Step 1: Add a `TestScan` entry (RED).** In `gui/scan_test.go`, add table entries for a valid md1 and mk1 string with expected `Content` = the new wrapper type (defined next), then run `go test ./gui/ -run TestScan` → FAIL (type undefined / unrecognized format).

- [ ] **Step 2: Define the wrapper type + scan branch.** In `gui/scan.go`, add (gui-package-local):
```go
// mdmkText is a BCH-validated md1/mk1 string to be engraved verbatim.
type mdmkText string
```
and insert the branch after the `codex32.New` arm (`gui/scan.go:68`), before the final `else`:
```go
	} else if codex32.ValidMD(string(buf)) || codex32.ValidMK(string(buf)) {
		return mdmkText(buf), nil
	} else {
		return nil, errScanUnknownFormat
	}
```

- [ ] **Step 3:** `go test ./gui/ -run TestScan` → PASS (md1/mk1 strings scan to `mdmkText`; existing cases unaffected).

- [ ] **Step 4: Commit** (`-s`): `gui: recognize md1/mk1 strings in the scanner`.

---

## Task 4: Engrave flow (`gui/gui.go`)

**Files:** Edit `gui/gui.go`, `gui/gui_test.go`.

- [ ] **Step 1: Add `validateMdmk` + `mdmkFlow` + the `engraveObjectFlow` case.** Mirror `validateDescriptor` (`gui/gui.go:399`) and `descriptorFlow` (`:1785`):
```go
func validateMdmk(params engrave.Params, s string) ([]string, []Plate, error) {
	qrc, err := qr.Encode(s, qr.L)
	if err != nil {
		return nil, nil, err
	}
	const qrScale = 3
	type te struct {
		label string
		par   backup.Paragraph
	}
	opts := []te{
		{"TEXT + QR", backup.Paragraph{Text: s, QR: qrc, QRScale: qrScale}},
		{"TEXT ONLY", backup.Paragraph{Text: s}},
		{"QR ONLY", backup.Paragraph{QR: qrc, QRScale: qrScale}},
	}
	var labels []string
	var plates []Plate
	var lastErr error
	for _, o := range opts {
		plan := backup.EngraveText(params, backup.Text{
			Paragraphs: []backup.Paragraph{o.par},
			Font:       sh.Font,
		})
		p, err := toPlate(plan, params)
		if err != nil {
			lastErr = err
			continue
		}
		labels = append(labels, o.label)
		plates = append(plates, p)
	}
	if len(plates) == 0 {
		return nil, nil, lastErr
	}
	return labels, plates, nil
}

func mdmkFlow(ctx *Context, th *Colors, s mdmkText) {
	labels, plates, err := validateMdmk(ctx.Platform.EngraverParams(), string(s))
	if err != nil {
		// validateMdmk only errors when NO mode (TEXT+QR/TEXT/QR-ONLY) fits a
		// plate — rare for an md1/mk1 string. Surface it like the descriptor
		// ErrTooLarge path: render NewErrorScreen(err) (gui.go:384 — takes ONE
		// error arg, no .Flow) via the standard `for !ctx.Done { ...Layout...;
		// ctx.Frame(...) }` loop used by backupWalletFlow (gui.go:~1748). A bare
		// `return` is acceptable for v1 (matches descriptorFlow siblings).
		return
	}
	cs := &ChoiceScreen{Title: "Engrave", Lead: "Choose engraving", Choices: labels}
	for {
		choice, ok := cs.Choose(ctx, th)
		if !ok {
			return
		}
		if NewEngraveScreen(ctx, plates[choice]).Engrave(ctx, &engraveTheme) {
			return
		}
	}
}
```
And in `engraveObjectFlow` (`gui/gui.go:1689`), add before `default:`:
```go
	case mdmkText:
		mdmkFlow(ctx, th, scan)
```
> **Iteration point:** the exact `ErrorScreen` constructor/flow — copy the descriptor `ErrTooLarge` handling (`gui/gui.go:386`); imports (`qr`, `backup`, `sh`, `engrave`) already exist in gui.go (used by `validateDescriptor`).

- [ ] **Step 2: Add tests (`gui/gui_test.go`).** Mirror `TestEngraveScreen` (`:240`) + `newTestEngraveScreen`:
```go
func TestMdmkEngraveScreen(t *testing.T) {
	ctx := NewContext(newPlatform())
	_, plates, err := validateMdmk(ctx.Platform.EngraverParams(), "md1yqpqqxqq8xtwhw4xwn4qh")
	if err != nil {
		t.Fatal(err)
	}
	if len(plates) == 0 {
		t.Fatal("no engravings produced")
	}
	// drive NewEngraveScreen(ctx, plates[0]) like TestEngraveScreen
}

func TestMdmkErrTooLarge(t *testing.T) {
	ctx := NewContext(newPlatform())
	huge := "md1" + strings_Repeat_q(5000) // an over-long string
	if _, _, err := validateMdmk(ctx.Platform.EngraverParams(), huge); err == nil {
		t.Error("expected ErrTooLarge for oversize input")
	}
}
```
> **Iteration point:** match the exact `TestEngraveScreen` driving (button presses) and helper names; `strings.Repeat` for the oversize string.

- [ ] **Step 3:** `go test ./gui/ -run 'TestMdmk|TestScan|TestEngrave'` → PASS.

- [ ] **Step 4: Commit** (`-s`): `gui: engrave md1/mk1 strings as text/QR plates`.

---

## Task 5: Full gate + PR

- [ ] **Step 1:** `export PATH="$HOME/.local/go/bin:$PATH" && cd /scratch/code/shibboleth/seedhammer && go test ./codex32/ ./gui/ -timeout 120s` → all green (incl. existing tests). Then `go vet ./codex32/ ./gui/` and `gofmt -l codex32/mdmk.go codex32/mdmk_test.go gui/scan.go gui/gui.go gui/scan_test.go gui/gui_test.go` (clean).
- [ ] **Step 2:** Push to fork: `git push -u origin feat/engrave-mdmk`.
- [ ] **Step 3 (CONFIRM WITH USER FIRST — outward-facing):** open the PR to upstream:
```bash
gh pr create --repo seedhammer/seedhammer --base main --head bg002h:feat/engrave-mdmk \
  --title "gui: BCH-validated md1/mk1 engraving" \
  --body "...summary, why, test plan, Signal contact bg002h.66..."
```
(DCO sign-off via `-s`; amend author to "Brian Goss" as in PR1; the controller confirms before opening.)

---

## Self-Review

- **Design coverage (§7 PR2):** BCH verifier reusing the generic engine → Task 2 ✓; scanner recognition (case-insensitive HRP + BCH) → Task 3 ✓ (note: `codex32.New`/our verify lowercases via `feFromRune`, so case-insensitivity comes for free — confirm in Step 0); text/QR plate with TEXT+QR/TEXT/QR-ONLY choice → Task 4 ✓; per-string, no reassembly → verifier operates on one string ✓; BCH parity test → Task 2 ✓.
- **Placeholder scan:** the BCH verifier body has explicit iteration points (exact `codex32.New` helper calls, mk regular/long selection, MK_LONG hi/lo) — these are flagged compile/parity-iteration items (family "trial-compile" pattern), gated by the Rust-sourced parity test, NOT behavioral TBDs. The init-residue value, targets, and approach are fully specified.
- **Type consistency:** `mdmkText` defined in Task 3, used in Task 4's `engraveObjectFlow` case + `mdmkFlow`; `ValidMD`/`ValidMK` defined in Task 2, used in Task 3.

## Open items to confirm during execution
- **#1: the init residue is `POLYMOD_INIT = 0x23181b3`, not `1`** — verified by the Step-0 spike + parity test.
- Source a real **long** mk1 vector (15-symbol code) for the parity test; pin with provenance.
- The MK_LONG hi/lo split of `0x41890d7e441cbe97273` (75 bits) — confirm `unpackSyms` reproduces the Rust symbols.
- Exact `ErrorScreen` constructor + `TestEngraveScreen` driving helpers (copy from existing code).
- Plate fit for the longest realistic mk1 chunk + QR — `toPlate`/`ErrTooLarge` is the backstop; `validateMdmk` already drops modes that don't fit and only errors if none fit.
