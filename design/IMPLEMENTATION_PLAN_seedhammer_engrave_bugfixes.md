# IMPLEMENTATION PLAN — SeedHammer II engraving-subsystem bug fixes (3 confirmed)

> **Status:** R0 GREEN (0C/0I) — plan R0 `a2e8d8ddfe491c06b` (round 0), persisted at `design/agent-reports/seedhammer-engrave-bugfixes-plan-R0-round0.md`; reviewer applied every diff and reproduced every FAIL→PASS in a throwaway worktree (BUG-1 wrap 1.84e19, BUG-2 dim37/41 panics, BUG-3 golden 16276/16277 mismatches, N=23 golden byte-identical f907eea↔fixed). 2 non-blocking Minors folded as implementer notes below. Cleared for single-implementer TDD. Single-author per project policy. Derived from the **GREEN** spec `design/SPEC_seedhammer_engrave_bugfixes.md` (R0 `a07f08aed3b2a9615`, 0C/0I) and bug-hunt `design/agent-reports/seedhammer-engrave-bughunt.md`.
> **Every code block in this plan was compiled and run** in a throwaway worktree off `f907eea` (now removed); each test was verified to FAIL on `f907eea` and PASS after its fix, and the legacy goldens were verified byte-identical.

## Goal

Fix three confirmed, hardware-reachable defects in the inherited SeedHammer engraving subsystem, faithfully and minimally, with a fail-on-buggy TDD test for each, leaving every ≤24-word and QR-bearing layout coordinate-identical:

1. **BUG-1 [CRITICAL]** — `SafePointer.Progress` unsigned underflow on move knots (`engrave/engrave.go:1467`): drop the `k.Engrave &&` so the early-`break` guard is symmetric.
2. **BUG-2 [HIGH]** — uncaught QR-version `panic` reachable from `EngraveSeedString` (`backup/backup.go`) for codex32 strings whose QR exceeds dim 33: add a primary size guard in `EngraveSeedString` plus a defense-in-depth early size-check in `engrave.ConstantQR`.
3. **BUG-3 [HIGH]** — `frontSideSeed` 33-word SLIP-39 plate overlap (`backup/backup.go`): rework the layout for `N > 24` (rebalanced `ceil/floor` two-column split + adaptive font + single contiguous column-2 block); the `N ≤ 24` path runs unchanged.

**Firmware-only cycle.** No `me`/CLI/schema/docs surface. No m-format codec edits. Merges back to fork `main` (`bg002h/seedhammer`); upstream offering is out of scope.

## Architecture

- **Fork / source of truth:** `/scratch/code/shibboleth/seedhammer`, branch `main`, HEAD `f907eea`. Do NOT modify the fork main working tree; all work happens in a dedicated worktree on branch `feat/engrave-bugfixes` branched off `f907eea`.
- **Go toolchain:** `export PATH=$PATH:/home/bcg/.local/go/bin` (go1.26.4). `go.mod` declares `go 1.25.10`.
- **Test command:** `go test ./engrave/... ./backup/...`. Goldens are gzip-compressed encoded B-splines under `<pkg>/testdata/*.bin`, compared knot-by-knot with ±1 unit slack by `seedhammer.com/internal/golden.CompareBSpline`; regenerate with `-update`.
- **Files touched (impl):** `engrave/engrave.go` (BUG-1 guard + BUG-2 `ConstantQR` early check), `backup/backup.go` (BUG-2 `EngraveSeedString` guard + `errors` import; BUG-3 `frontSideSeed` rework).
- **Files touched (tests):** `engrave/engrave_test.go` (BUG-1: new `TestSafePointerNoUnderflow`), `backup/backup_test.go` (BUG-2: `TestEngraveSeedStringTooLong` + `TestEngraveSeedStringHappy`; BUG-3: `TestSLIP39Large` golden + `TestSLIP39LargeGeometry` + helpers), plus a new committed golden `backup/testdata/slip39-33-words.bin`.
- **No signature changes.** `ConstantQR` keeps its `(*ConstantQRCmd, error)` signature (the early check returns the existing `error`). `bitmapForQRStatic` is **not** converted to error-return; its `panic` becomes an unreachable assertion because no caller can reach it with `dim > 33` (R0-ratified: `ConstantQRCmd` is built only inside `ConstantQR`, which now rejects `dim > 33` before constructing the cmd, so the `:616` `ConstantQRCmd.Engrave` site is unreachable with an unvalidated size). `SafePointer.Progress` change is internal.

## Tech Stack

Go 1.26 (module declares 1.25). Packages: `seedhammer.com/engrave`, `seedhammer.com/backup`, `seedhammer.com/bspline`, `seedhammer.com/bezier`, `seedhammer.com/codex32`, `seedhammer.com/slip39`, `seedhammer.com/font/constant`, `github.com/seedhammer/kortschak-qr`, `seedhammer.com/internal/golden`. Production scale (from the test harness, `mm = 6400`): `F(v) = round(v*Millimeter)`, `I(v) = Millimeter*v`; `F(4.1) = 26240` units (4.1mm), `F(85) = 544000` units (85mm).

### Source facts confirmed against `f907eea` (used by every step below)

- `engrave/engrave.go:1448` `progress uint`; `:1452-1458` `Resume` prepends a leading non-engrave move; `:1460-1487` `Progress`; the asymmetric guard is at line **1467** (`if k.Engrave && s.progress < k.T {`), the unconditional subtraction at **1470** (`s.progress -= k.T`).
- `engrave/engrave.go:384-402` `bitmapForQRStatic` switches on `{21,25,29,33}`, `default: panic("unsupported qr code version")` at **399**. `ConstantQR` at **406-410** (eager `bitmapForQRStatic(dim)` at 410); the cmd is constructed at **477-480**. `engrave.go` already imports `"errors"` (line 6).
- `bspline/bspline.go:24-28` `type Knot struct { Ctrl bezier.Point; T uint; Engrave bool }`. `bezier.Pt(x, y)` constructs a `bezier.Point{X, Y int}`; `Point` is comparable (`==` works).
- `backup/backup.go:75-87` `EngraveSeedString` (`qr.Encode` at 77 → `ConstantQR` at 81, no guard between). `backup.go` does **not** import `"errors"` — the BUG-2 guard adds it. `plateFontSize = 4.1` at **89**. `frontSideSeed` at **161-225**; legacy `maxCol1=16, maxCol2=4` at 172-173; `pfs := params.F(plateFontSize)` at 168; `col1Height := pfs*endCol1` at 176; emission order col1 → col2-top → QR → col2-bottom → title.
- `backup/backup_test.go`: `compareGolden(t, name, plan)` at **377-391** (signature `(t testing.TB, name string, plan engrave.Engraving)`); `params`/`conf` globals at 39-52; `genSeed` at 341-375 (builds a BIP-39 `Seed` with QR + mfp); `TestSLIP39` at 202-225 (splits a word string, uses `slip39words.ShortestWord/LongestWord` and `constant.Font`). Committed goldens: `seed-0-words-24.bin`, `seed-1-words-12.bin`, `slip39-0.bin`, `codex32-{0,1}.bin`, `text-{0,1,2}-shards-1.bin`. There is **no** committed 23/24-word SLIP-39 golden.
- `slip39/wordlist.go:7-9` `ShortestWord = 4`, `LongestWord = 8`. SLIP-39 layout does **not** validate checksums — `wordColumn` renders whatever strings are in `plate.Mnemonic` — so the BUG-3 test may build a 33-element `[]string` from real SLIP-39 words without a valid share.
- `engrave/engrave_test.go`: `package engrave` (same-package; `sp.progress`, `sp.safePoint` are accessible). Existing `TestSafePointer` at **321-371**. `bezier`/`bspline` already imported.

### Verified geometry oracle (the BUG-3 exact-numbers table, recomputed in Go — matches the spec table exactly)

| N  | col1Rows=ceil(N/2) / col2Rows=floor(N/2) | pfsN = min(F(4.1), 16·F(4.1)/col1Rows) | col1 bottom anchor | col2 bottom anchor | no-overlap |
|----|---|---|---|---|---|
| 27 | 14 / 13 | 26240 (4.100mm) | 71.20mm | 67.10mm | ✓ (4.10mm gap) |
| 30 | 15 / 15 | 26240 (4.100mm) | 73.25mm | 73.25mm | ✓ (0.00mm gap) |
| **33** | **17 / 16** | **24696 (3.859mm)** | **75.30mm** | **71.44mm** | **✓ (3.86mm gap)** |

`pfsN` for N=33 = `16*26240/17 = 419840/17 = 24696` (integer division). `col1Height = 24696*17 = 419832`; top anchor `(544000-419832)/2 = 62084`; col1 bottom `62084 + 24696*17 = 481916` units = 75.30mm; col2 bottom `62084 + 24696*16 = 457220` units = 71.44mm. (N=25 from the spec table is illustrative/unreachable — no GUI path emits 25 words; reachable `N>24` = {27,30,33}.)

---

## Task 0 — Worktree + clean baseline

**Files:** none (setup only).

- [ ] Create the isolated worktree on a new branch off fork main `f907eea`:
  ```bash
  export PATH=$PATH:/home/bcg/.local/go/bin
  git -C /scratch/code/shibboleth/seedhammer worktree add -b feat/engrave-bugfixes /tmp/engrave-bugfixes f907eea
  ```
  Expect: `Preparing worktree ... HEAD is now at f907eea Merge BIP-85 custom hardened index (feat/bip85-custom-index)`.
- [ ] Confirm toolchain + base commit:
  ```bash
  cd /tmp/engrave-bugfixes && go version && git log --oneline -1
  ```
  Expect: `go version go1.26.4 linux/amd64` and `f907eea Merge BIP-85 custom hardened index (feat/bip85-custom-index)`.
- [ ] Record the green baseline:
  ```bash
  cd /tmp/engrave-bugfixes && go test ./engrave/... ./backup/... 2>&1 | tail -5
  ```
  Expect:
  ```
  ok  	seedhammer.com/engrave	<time>
  ok  	seedhammer.com/backup	<time>
  ```

> **Commit discipline for every commit in this cycle** (fork standard): SSH-signed + DCO, author **Brian Goss <goss.brian@gmail.com>**, with the `Co-Authored-By` trailer. Stage **explicit paths only** — never `git add -A`. Template:
> ```bash
> cd /tmp/engrave-bugfixes
> git add <explicit paths>
> git -c user.name="Brian Goss" -c user.email="goss.brian@gmail.com" \
>     commit -S -s -m "<subject>" -m "<body>" \
>     -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
> ```

> **⚠ Plan-R0 Minor-2 — execute the bugs in STRICT task order, one commit each before starting the next bug's tests.** `backup/backup_test.go` is staged in BOTH the BUG-2 and BUG-3 commits (different hunks). If you write all tests up front, the BUG-2 commit would capture BUG-3 test code too. So: finish + COMMIT BUG-1, then BUG-2 (write its tests → fix → commit), then BUG-3 (write its tests → fix → commit). Do NOT pre-author later bugs' tests.

---

## BUG-1 — symmetric break guard in `SafePointer.Progress`

**Root cause:** at `engrave/engrave.go:1467` the early `break` is guarded by `k.Engrave && s.progress < k.T`, so a **move** knot (`Engrave == false`) falls through to the unconditional `s.progress -= k.T` at 1470. The driver-reported progress lags the leading move knot's `T` by up to a partial flushed word, so `s.progress < k.T` on the leading move → the uint subtraction wraps to ~1.84e19, the loop retires all history and selects a garbage safe point.

**Fix:** make the guard symmetric — break whenever `s.progress < k.T`, independent of `k.Engrave`. This is **break-based on true elapse** (the R0-ratified form); a saturating-subtract is explicitly rejected.

### Task 1.1 — Files

- **Modify** `engrave/engrave_test.go` — add `TestSafePointerNoUnderflow` (`package engrave`, same-package field access).
- **Modify** `engrave/engrave.go` — line 1467 guard.

### Task 1.2 — Write the failing test FIRST

- [ ] Append to `engrave/engrave_test.go` (after `TestSafePointer`, before `FuzzConstantQR`). The required imports (`testing`, `seedhammer.com/bezier`, `seedhammer.com/bspline`) are already present in the file.

  ```go
  func TestSafePointerNoUnderflow(t *testing.T) {
  	// A leading non-engrave MOVE knot (the universal start of every engraving),
  	// followed by a clamped triple at P and a trailing engrave knot. The move's
  	// T is larger than the first reported progress, which on f907eea wraps
  	// s.progress via the asymmetric guard at engrave.go:1467.
  	P := bezier.Pt(10, 10)
  	hist := []bspline.Knot{
  		{Ctrl: bezier.Pt(5, 5), T: 100, Engrave: false}, // leading move
  		{Ctrl: P, T: 50, Engrave: true},                 // k0 of clamped triple
  		{Ctrl: P, T: 50, Engrave: true},                 // k1
  		{Ctrl: P, T: 50, Engrave: true},                 // k2 (triple ends at cum T=250)
  		{Ctrl: bezier.Pt(20, 20), T: 50, Engrave: true},
  	}
  	sp := new(SafePointer)
  	for _, k := range hist {
  		sp.Knot(k)
  	}

  	// Cumulative T through the END of the clamped triple (k2): 100+50+50+50.
  	const tripleEnd = 250

  	// Independent sum of all T fed, for the wrap-catcher counter-invariant.
  	var totalTicks uint
  	for _, k := range hist {
  		totalTicks += k.T
  	}

  	// Feed progress in increments that keep completed < tripleEnd until the
  	// final step, so the safe point must NOT advance to P early.
  	steps := []uint{0, 50, 50, 49, 1, 100} // cumulative: 0,50,100,149,150,250
  	completed := uint(0)
  	for i, d := range steps {
  		sp.Progress(d)
  		completed += d

  		// (a) Wrap-catcher counter-invariant: progress never exceeds total ticks.
  		if sp.progress > totalTicks {
  			t.Fatalf("step %d: sp.progress=%d exceeds totalTicks=%d (underflow wrap)", i, sp.progress, totalTicks)
  		}

  		// (b) Safe-point reference: never select the triple's control point P
  		// before its cumulative T has fully elapsed (completed >= tripleEnd).
  		if completed < tripleEnd && sp.safePoint == P {
  			t.Fatalf("step %d: safePoint advanced to %v at completed=%d < tripleEnd=%d (not-yet-reached safe point)",
  				i, P, completed, tripleEnd)
  		}
  	}

  	// After the full triple has elapsed, the safe point MUST be P.
  	if sp.safePoint != P {
  		t.Fatalf("after completed=%d safePoint=%v, want %v", completed, sp.safePoint, P)
  	}
  }
  ```

- [ ] Run it against unmodified `f907eea` and confirm it FAILS:
  ```bash
  cd /tmp/engrave-bugfixes && go test ./engrave/ -run TestSafePointerNoUnderflow 2>&1 | tail -6
  ```
  Expect (verified):
  ```
  --- FAIL: TestSafePointerNoUnderflow (0.00s)
      engrave_test.go:NN: step 0: sp.progress=18446744073709551316 exceeds totalTicks=300 (underflow wrap)
  FAIL
  FAIL	seedhammer.com/engrave	0.00Xs
  ```

### Task 1.3 — Minimal implementation

- [ ] Apply the one-line guard change in `engrave/engrave.go`. Old (lines 1463-1471):
  ```go
  	for len(s.history) > s.completed {
  		k := s.history[s.completed]
  		// Stop when an engraving knot later than progress
  		// is reached.
  		if k.Engrave && s.progress < k.T {
  			break
  		}
  		s.progress -= k.T
  ```
  New:
  ```go
  	for len(s.history) > s.completed {
  		k := s.history[s.completed]
  		// Stop when a knot later than progress is reached. The guard
  		// is symmetric across move and engrave knots: a leading move
  		// knot whose duration k.T exceeds the driver-reported progress
  		// must not retire early, or the unsigned s.progress -= k.T
  		// below would underflow (wrap) and corrupt the safe point.
  		if s.progress < k.T {
  			break
  		}
  		s.progress -= k.T
  ```

### Task 1.4 — Run, expect PASS; confirm no regression

- [ ] Run the new test and the existing `TestSafePointer` together:
  ```bash
  cd /tmp/engrave-bugfixes && go test ./engrave/ -run 'TestSafePointerNoUnderflow|TestSafePointer$' -v 2>&1 | grep -E 'RUN|PASS|FAIL'
  ```
  Expect (verified): both `--- PASS`.
- [ ] Run the full engrave package (no regression elsewhere):
  ```bash
  cd /tmp/engrave-bugfixes && go test ./engrave/... 2>&1 | tail -3
  ```
  Expect: `ok  	seedhammer.com/engrave  <time>`.

### Task 1.5 — Commit

- [ ] Stage and commit:
  ```bash
  cd /tmp/engrave-bugfixes
  git add engrave/engrave.go engrave/engrave_test.go
  git -c user.name="Brian Goss" -c user.email="goss.brian@gmail.com" \
      commit -S -s \
      -m "engrave: fix SafePointer.Progress uint underflow on move knots" \
      -m "The early-break guard in SafePointer.Progress only protected engrave
  knots (k.Engrave && s.progress < k.T), so a leading non-engrave move
  knot whose duration exceeds the driver-reported progress fell through to
  the unconditional s.progress -= k.T and wrapped the unsigned counter to
  ~1.84e19, corrupting the selected safe point on resume. Make the guard
  symmetric (break whenever s.progress < k.T). Add TestSafePointerNoUnderflow
  asserting the counter invariant and the safe-point reference." \
      -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
  ```

---

## BUG-2 — guard the QR-version panic reachable from `EngraveSeedString`

**Root cause:** `EngraveSeedString` does `qr.Encode(seed, qr.M)` then `engrave.ConstantQR(qrc)` with no size guard. `ConstantQR` eagerly calls `bitmapForQRStatic(dim)`, which `panic`s for any `dim` not in `{21,25,29,33}`. Codex32 short codes up to 93 chars → dim 37, and long codes 125-127 chars → dim 41, both reachable from the GUI; the panic is uncaught (no `recover` anywhere) → controller crash on the watchdog-less RP2350.

**Fix:** (primary) reject `qrc.Size > 33` in `EngraveSeedString` before `ConstantQR`, returning a clean error onto the existing `err != nil` path. (Defense-in-depth) add the same early size-check inside `ConstantQR` before `bitmapForQRStatic`, so no `ConstantQR` caller can panic; the `panic` in `bitmapForQRStatic` becomes an unreachable assertion. `bitmapForQRStatic` is **not** converted to error-return (R0-ratified). `33` is the exact correct cutoff (dim 33 = V4, the last supported; dim 37 = V5).

### Task 2.1 — Files

- **Modify** `backup/backup_test.go` — add `TestEngraveSeedStringTooLong` + `TestEngraveSeedStringHappy`.
- **Modify** `backup/backup.go` — add `"errors"` import + guard in `EngraveSeedString`.
- **Modify** `engrave/engrave.go` — early size-check in `ConstantQR`.

### Task 2.2 — Write the failing test FIRST

- [ ] Append to `backup/backup_test.go`. Imports needed: `"seedhammer.com/codex32"` and `"seedhammer.com/font/constant"` are already imported by the file (used by `TestCodex32`/`TestSLIP39`); `"testing"` too. The two codex32 strings below were **verified**: under `qr.M` after `strings.ToUpper` they encode to dim 37 and dim 41 respectively, and both panic through `EngraveSeedString` on `f907eea`.

  ```go
  // engraveStringRecovered runs EngraveSeedString under a recover so a panic on
  // current code is reported as a test failure instead of crashing the run.
  func engraveStringRecovered(t *testing.T, raw string) (err error, panicked bool) {
  	t.Helper()
  	defer func() {
  		if r := recover(); r != nil {
  			panicked = true
  		}
  	}()
  	cx, e := codex32.New(raw)
  	if e != nil {
  		t.Fatalf("codex32.New(%q): %v", raw, e)
  	}
  	id, _, _ := cx.Split()
  	s := SeedString{Title: id, Seed: cx.String(), Font: constant.Font}
  	_, err = EngraveSeedString(params, s)
  	return err, false
  }

  func TestEngraveSeedStringTooLong(t *testing.T) {
  	cases := []struct{ name, s string }{
  		// 93-char short code -> QR dim 37 (V5, unsupported).
  		{"dim37", "ms10testsqqrsu9guyv4rzwplgex4gkmzd9c8wl593jfe4gdg47mtm3xt6tv7pelw7h7qxzs3rq0jvtg3ye6xggmhcl92"},
  		// 127-char BIP-93 long-code reference -> QR dim 41 (V6, unsupported).
  		{"dim41", "ms100c8vsm32zxfguhpchtlupzry9x8gf2tvdw0s3jn54khce6mua7lqpzygsfjd6an074rxvcemlh8wu3tk925acdefghjklmnpqrstuvwxy06fhpv80undvarhrak"},
  	}
  	for _, tc := range cases {
  		t.Run(tc.name, func(t *testing.T) {
  			err, panicked := engraveStringRecovered(t, tc.s)
  			if panicked {
  				t.Fatalf("%s: EngraveSeedString panicked; want a returned error", tc.name)
  			}
  			if err == nil {
  				t.Fatalf("%s: want non-nil error, got nil", tc.name)
  			}
  		})
  	}
  }

  func TestEngraveSeedStringHappy(t *testing.T) {
  	// 74-char codex32 short code -> QR dim 33 (V4, supported).
  	const happy = "ms10leetsllhdmn9m42vcsamx24zrxgs3qrl7ahwvhw4fnzrhve25gvezzyq0pgjxpzx0ysaam"
  	cx, err := codex32.New(happy)
  	if err != nil {
  		t.Fatal(err)
  	}
  	id, _, _ := cx.Split()
  	s := SeedString{Title: id, Seed: cx.String(), Font: constant.Font}
  	e, err := EngraveSeedString(params, s)
  	if err != nil {
  		t.Fatalf("happy path returned error: %v", err)
  	}
  	if e == nil {
  		t.Fatal("happy path returned nil Engraving")
  	}
  }
  ```

- [ ] Run against unmodified `f907eea` and confirm the too-long test FAILS (the recover converts the panic to a failure):
  ```bash
  cd /tmp/engrave-bugfixes && go test ./backup/ -run TestEngraveSeedStringTooLong 2>&1 | tail -8
  ```
  Expect (verified):
  ```
  --- FAIL: TestEngraveSeedStringTooLong (0.00s)
      --- FAIL: TestEngraveSeedStringTooLong/dim37 (0.00s)
          backup_test.go:NN: dim37: EngraveSeedString panicked; want a returned error
      --- FAIL: TestEngraveSeedStringTooLong/dim41 (0.00s)
          backup_test.go:NN: dim41: EngraveSeedString panicked; want a returned error
  FAIL
  FAIL	seedhammer.com/backup	0.00Xs
  ```
  (`TestEngraveSeedStringHappy` already passes on `f907eea` — dim 33 is supported.)

### Task 2.3 — Minimal implementation

- [ ] **Primary guard** — add `"errors"` to `backup/backup.go`'s import block. Old (lines 4-13):
  ```go
  import (
  	"fmt"
  	"image"
  	"math"
  	"strings"

  	qr "github.com/seedhammer/kortschak-qr"
  	"seedhammer.com/engrave"
  	"seedhammer.com/font/vector"
  )
  ```
  New:
  ```go
  import (
  	"errors"
  	"fmt"
  	"image"
  	"math"
  	"strings"

  	qr "github.com/seedhammer/kortschak-qr"
  	"seedhammer.com/engrave"
  	"seedhammer.com/font/vector"
  )
  ```

- [ ] Add the guard in `EngraveSeedString` after `qr.Encode`, before `ConstantQR`. Old (lines 75-84):
  ```go
  func EngraveSeedString(params engrave.Params, plate SeedString) (engrave.Engraving, error) {
  	seed := strings.ToUpper(plate.Seed)
  	qrc, err := qr.Encode(seed, qr.M)
  	if err != nil {
  		return nil, err
  	}
  	qrCmd, err := engrave.ConstantQR(qrc)
  	if err != nil {
  		return nil, err
  	}
  ```
  New:
  ```go
  func EngraveSeedString(params engrave.Params, plate SeedString) (engrave.Engraving, error) {
  	seed := strings.ToUpper(plate.Seed)
  	qrc, err := qr.Encode(seed, qr.M)
  	if err != nil {
  		return nil, err
  	}
  	if qrc.Size > 33 {
  		return nil, errors.New("seed too long to engrave QR")
  	}
  	qrCmd, err := engrave.ConstantQR(qrc)
  	if err != nil {
  		return nil, err
  	}
  ```

- [ ] **Defense-in-depth** — early size-check in `ConstantQR` (`engrave.go` already imports `"errors"`). Old (lines 406-410):
  ```go
  func ConstantQR(qrc *qr.Code) (*ConstantQRCmd, error) {
  	dim := qrc.Size
  	qr := bitmapForQR(qrc)
  	engraved := newBitmap(dim, dim)
  	posMarkers, alignMarkers := bitmapForQRStatic(dim)
  ```
  New:
  ```go
  func ConstantQR(qrc *qr.Code) (*ConstantQRCmd, error) {
  	dim := qrc.Size
  	if dim > 33 {
  		// bitmapForQRStatic only supports versions 1-4 (dims 21/25/29/33).
  		// Reject larger versions here so no caller can trigger the panic
  		// at bitmapForQRStatic's default case.
  		return nil, errors.New("seed too long to engrave QR")
  	}
  	qr := bitmapForQR(qrc)
  	engraved := newBitmap(dim, dim)
  	posMarkers, alignMarkers := bitmapForQRStatic(dim)
  ```

### Task 2.4 — Run, expect PASS; confirm no regression

- [ ] New BUG-2 tests:
  ```bash
  cd /tmp/engrave-bugfixes && go test ./backup/ -run 'TestEngraveSeedString' -v 2>&1 | grep -E 'RUN|PASS|FAIL'
  ```
  Expect (verified): all `--- PASS` (`dim37`, `dim41`, `TooLong`, `Happy`).
- [ ] Confirm `ConstantQR`'s early check does **not** regress `TestConstantQR` (its `n=16..40` loop tops out at dim 33 under `qr.Q`, verified):
  ```bash
  cd /tmp/engrave-bugfixes && go test ./engrave/ -run 'TestConstantQR$|TestCSQR' 2>&1 | tail -3
  ```
  Expect: `ok  	seedhammer.com/engrave  <time>`.
- [ ] Full both packages:
  ```bash
  cd /tmp/engrave-bugfixes && go test ./engrave/... ./backup/... 2>&1 | tail -3
  ```
  Expect: both `ok`.

### Task 2.5 — Commit

- [ ] Stage and commit:
  ```bash
  cd /tmp/engrave-bugfixes
  git add backup/backup.go engrave/engrave.go backup/backup_test.go
  git -c user.name="Brian Goss" -c user.email="goss.brian@gmail.com" \
      commit -S -s \
      -m "backup,engrave: reject over-large QR instead of panicking" \
      -m "EngraveSeedString encoded any codex32 string to a QR and called
  engrave.ConstantQR with no size guard; ConstantQR eagerly calls
  bitmapForQRStatic which panics for QR dims outside {21,25,29,33}.
  Codex32 short codes up to 93 chars (dim 37) and long codes 125-127
  (dim 41) are GUI-reachable, so the panic crashed the watchdog-less
  RP2350 controller. Guard qrc.Size > 33 in EngraveSeedString (primary)
  and add a defense-in-depth early check in ConstantQR so no caller can
  reach the panic. Add TestEngraveSeedStringTooLong/Happy." \
      -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
  ```

---

## BUG-3 — large-N layout rework in `frontSideSeed`

**Root cause:** the legacy column layout is hardcoded for ≤24-word BIP-39 (`maxCol1=16, maxCol2=4`): column 2 is split into a TOP block (rows `endCol1..endCol2`, anchored at the top of the col-1 band) and a BOTTOM block (rows `endCol2..N`, anchored at the bottom of the col-1 band). The two blocks grow toward each other; for N=33 col-2 must hold 17 rows in a 16-row band, so the bottom block's top rises above the top block's bottom → +4.10mm overlap → garbled, unreadable plate. `gui.toPlate` only rejects off-plate bounds, not on-plate overlap, so it engraves.

**Fix (A1, R0-ratified):** for `N > 24`, rebalance into two columns of `col1Rows = ceil(N/2)` and `col2Rows = floor(N/2)` (so `col2Rows ≤ col1Rows`), shrink the font to `pfsN = min(F(4.1), 16*F(4.1)/col1Rows)` (only shrinks at N=33), and emit column 2 as **one contiguous block** from the shared top anchor — eliminating the two-block collision. For `N ≤ 24` the existing path runs unchanged (same constants, same two-block col-2, same `pfs = F(4.1)`), so BIP-39 {12,18,20,23,24} and every QR-bearing layout are coordinate-identical. The single predicate `N > 24` is the sole behavioral change. **Deliberate (NOT a regression):** N=27 and N=30 move from the legacy two-block split to the rebalanced single-block split (both stay at full 4.1mm font) — this is by design (one uniform large-N rule) and is pinned by the N=30 test; the post-impl review must treat the N=27/30 layout change as expected.

### Task 3.1 — Files

- **Modify** `backup/backup_test.go` — add `slip39Words` helper, `seedLayout` helper, `TestSLIP39Large` (golden), `TestSLIP39LargeGeometry`, `TestSLIP39_23WordPin`, and `TestSLIP39_30WordInBounds`. (Imports: `strings`, `slip39words "seedhammer.com/slip39"`, `constant "seedhammer.com/font/constant"`, and `engrave` are already in the file; add `"slices"` in Task 3.5 for the within-bounds pin — see imports note.)
- **Create** `backup/testdata/slip39-33-words.bin` — the committed N=33 golden (generated under the fixed code with `-update`).
- **Modify** `backup/backup.go` — rework `frontSideSeed`.

> **Imports note:** the within-bounds pin walks the plan via `engrave.PlanEngraving` + `slices.Collect`. `backup_test.go` already imports `engrave` and `bspline` and `slices` is **not** yet imported — add `"slices"` to the test import block when adding the within-bounds pin (Task 3.5). `strings`, `constant`, `slip39words`, `engrave` are already imported.

### Task 3.2 — Write the failing tests FIRST (golden + geometry)

- [ ] Append helpers + the golden test + the geometry test to `backup/backup_test.go`. The 33 words are real SLIP-39 words (layout does not validate checksums):

  ```go
  // slip39Words builds an N-word slice of real SLIP-39 words for layout tests.
  // frontSideSeed does not validate SLIP-39 checksums, so any valid-wordlist
  // strings exercise the geometry.
  func slip39Words(n int) []string {
  	base := strings.Fields("duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard shadow pistol always adequate wildlife fancy gross oasis cylinder mustang wrist rescue view")
  	out := make([]string, n)
  	for i := range out {
  		out[i] = base[i%len(base)]
  	}
  	return out
  }

  // seedLayout mirrors frontSideSeed's large-N (N>24) anchor computation so the
  // test can assert the chosen geometry explicitly (pfsN and the no-overlap
  // relation), independent of the byte-exact golden.
  func seedLayout(n int) (pfs, col1Rows, col2Rows, col1Bot, col2Bot int) {
  	full := params.F(4.1)
  	col1Rows = (n + 1) / 2 // ceil(N/2)
  	col2Rows = n / 2       // floor(N/2)
  	pfs = full
  	if alt := 16 * full / col1Rows; alt < pfs {
  		pfs = alt
  	}
  	col1H := pfs * col1Rows
  	plateY := params.F(85)
  	top := (plateY - col1H) / 2
  	col1Bot = top + pfs*col1Rows
  	col2Bot = top + pfs*col2Rows
  	return
  }

  func TestSLIP39Large(t *testing.T) {
  	seedDesc := Seed{
  		Mnemonic:     slip39Words(33),
  		ShortestWord: slip39words.ShortestWord,
  		LongestWord:  slip39words.LongestWord,
  		Title:        "7945 #1 1/1",
  		Font:         constant.Font,
  	}
  	side, err := EngraveSeed(params, seedDesc)
  	if err != nil {
  		t.Fatal(err)
  	}
  	compareGolden(t, "slip39-33-words", side)
  }

  func TestSLIP39LargeGeometry(t *testing.T) {
  	pfs, col1Rows, col2Rows, col1Bot, col2Bot := seedLayout(33)
  	// Pin the chosen adaptive scale (3.859mm), not just "some" non-overlap layout.
  	if pfs != 24696 {
  		t.Errorf("pfsN = %d, want 24696", pfs)
  	}
  	if col1Rows != 17 || col2Rows != 16 {
  		t.Errorf("rows = %d/%d, want 17/16", col1Rows, col2Rows)
  	}
  	// No overlap: column 2's bottom edge must not fall below column 1's bottom.
  	if col2Bot > col1Bot {
  		t.Errorf("col2 bottom %d > col1 bottom %d (overlap)", col2Bot, col1Bot)
  	}
  	// Within plate bounds on the Y axis.
  	plateY := params.F(85)
  	if col1Bot > plateY || col2Bot > plateY {
  		t.Errorf("off-plate: col1Bot=%d col2Bot=%d plateY=%d", col1Bot, col2Bot, plateY)
  	}
  }
  ```

- [ ] Run the geometry test on `f907eea` — it FAILS (the legacy code never computes `pfsN`, but `seedLayout` is a test-local oracle, so this test PASSES even on `f907eea`; it is the **pin**, not the fail-on-buggy gate). The fail-on-buggy gate is the golden test in the next step. Run the geometry test now only to confirm it compiles and the oracle is correct:
  ```bash
  cd /tmp/engrave-bugfixes && go test ./backup/ -run TestSLIP39LargeGeometry -v 2>&1 | grep -E 'PASS|FAIL'
  ```
  Expect (verified): `--- PASS` (the formula oracle is correct: pfsN=24696, 17/16, no overlap, in bounds).

> **Why two tests:** `TestSLIP39LargeGeometry` is a human-readable pin of the chosen geometry (`pfsN==24696` + no-overlap + in-bounds), but it asserts the test's own formula so it cannot fail on buggy production code. `TestSLIP39Large` is the **byte-exact fail-on-buggy gate**: its committed golden is generated under the fixed code, and the buggy `f907eea` plan differs in thousands of knots. Both are required by the spec (acceptance #1). The byte-exact golden + the N=23 byte-identical cross-check (Task 3.5) already prove production correctness; the geometry oracle is a readable secondary pin.
>
> **Plan-R0 Minor-1 (OPTIONAL belt-and-suspenders, implementer's discretion):** to make the no-overlap check independent of the test-local formula too, `TestSLIP39LargeGeometry` MAY additionally measure col-1/col-2 y-ranges from the *emitted* plan — `slices.Collect(engrave.PlanEngraving(conf, side))`, band knots by `Ctrl.X` (col-1 at x≈10mm, col-2 at x≈44mm), and assert max(col-2 y) ≤ max(col-1 y). Not required (the golden is the gate); skip if it adds noise.

### Task 3.3 — Implement the `frontSideSeed` rework

- [ ] Replace the body of `frontSideSeed` (lines 161-225). Old:
  ```go
  func frontSideSeed(params engrave.Params, plate Seed, qrc *engrave.ConstantQRCmd) engrave.Engraving {
  	return func(yield func(engrave.Command) bool) {
  		plateDims := image.Point{
  			X: params.F(85),
  			Y: params.F(85),
  		}
  		t := engrave.NewTransform(yield)
  		pfs := params.F(plateFontSize)
  		constant := engrave.NewConstantStringer(plate.Font, params, pfs)

  		const (
  			maxCol1 = 16
  			maxCol2 = 4
  		)
  		endCol1 := min(maxCol1, len(plate.Mnemonic))
  		col1Height := pfs * endCol1

  		// Engrave master fingerprint.
  		innerMargin := params.I(innerMargin)
  		metaMargin := params.I(4)
  		if plate.MasterFingerprint != 0 {
  			mfp := fmt.Sprintf("%.8X", plate.MasterFingerprint)
  			offy := (plateDims.Y-col1Height)/2 - metaMargin
  			mfpStr := engrave.String(plate.Font, params.F(plateSmallFontSize), mfp)
  			mfpszX, mfpszY := mfpStr.Measure()
  			t.Offset((plateDims.X-mfpszX)/2, offy-mfpszY)
  			mfpStr.Engrave(t.Yield)
  		}

  		// Engrave column 1.
  		off := t.Offset(innerMargin, (plateDims.Y-col1Height)/2)
  		wordColumn(off, constant, plate.Font, pfs, plate.Mnemonic, plate.ShortestWord, plate.LongestWord, 0, endCol1)

  		// Engrave (top of) column 2.
  		endCol2 := min(endCol1+maxCol2, len(plate.Mnemonic))
  		off = t.Offset(params.I(44), (plateDims.Y-col1Height)/2)
  		wordColumn(off, constant, plate.Font, pfs, plate.Mnemonic, plate.ShortestWord, plate.LongestWord, endCol1, endCol2)

  		// Engrave seed QR.
  		if qrc != nil {
  			const qrScale = 3
  			qrCmd := qrc.Engrave(params.StepperConfig, params.StrokeWidth, qrScale)
  			qrsz := qrc.Size * params.StrokeWidth * qrScale
  			t.Offset(params.I(60)-qrsz/2, (plateDims.Y-qrsz)/2)
  			qrCmd(t.Yield)
  		}

  		{
  			// Engrave bottom of column 2.
  			height := (len(plate.Mnemonic) - endCol2) * pfs
  			off := t.Offset(params.I(44), (plateDims.Y+col1Height)/2-height)
  			wordColumn(off, constant, plate.Font, pfs, plate.Mnemonic, plate.ShortestWord, plate.LongestWord, endCol2, len(plate.Mnemonic))
  		}

  		// Engrave title.
  		title := strings.ToUpper(plate.Title)
  		{
  			offy := (plateDims.Y+col1Height)/2 + metaMargin
  			title := engrave.String(plate.Font, params.F(plateSmallFontSize), title)
  			titleWidth, _ := title.Measure()
  			t.Offset((plateDims.X-titleWidth)/2, offy)
  			title.Engrave(t.Yield)
  		}
  	}
  }
  ```
  New:
  ```go
  func frontSideSeed(params engrave.Params, plate Seed, qrc *engrave.ConstantQRCmd) engrave.Engraving {
  	return func(yield func(engrave.Command) bool) {
  		plateDims := image.Point{
  			X: params.F(85),
  			Y: params.F(85),
  		}
  		t := engrave.NewTransform(yield)

  		const (
  			maxCol1 = 16
  			maxCol2 = 4
  			// largeN is the inclusive upper bound of the legacy
  			// 16+4+4 two-block column-2 layout. Word counts above
  			// largeN use the rebalanced single-block layout.
  			largeN = 24
  		)
  		n := len(plate.Mnemonic)
  		// pfs is the plate font size, endCol1 the number of rows in
  		// column 1, and col1Height the height of column 1. For N<=24
  		// these are the legacy values; for N>24 column 1 is rebalanced
  		// to ceil(N/2) rows and the font is shrunk just enough to keep
  		// those rows within the legacy column envelope (16 rows at the
  		// full font).
  		pfs := params.F(plateFontSize)
  		endCol1 := min(maxCol1, n)
  		if n > largeN {
  			endCol1 = (n + 1) / 2 // ceil(N/2)
  			pfs = min(pfs, maxCol1*params.F(plateFontSize)/endCol1)
  		}
  		col1Height := pfs * endCol1
  		constant := engrave.NewConstantStringer(plate.Font, params, pfs)

  		// Engrave master fingerprint.
  		innerMargin := params.I(innerMargin)
  		metaMargin := params.I(4)
  		if plate.MasterFingerprint != 0 {
  			mfp := fmt.Sprintf("%.8X", plate.MasterFingerprint)
  			offy := (plateDims.Y-col1Height)/2 - metaMargin
  			mfpStr := engrave.String(plate.Font, params.F(plateSmallFontSize), mfp)
  			mfpszX, mfpszY := mfpStr.Measure()
  			t.Offset((plateDims.X-mfpszX)/2, offy-mfpszY)
  			mfpStr.Engrave(t.Yield)
  		}

  		// Engrave column 1.
  		off := t.Offset(innerMargin, (plateDims.Y-col1Height)/2)
  		wordColumn(off, constant, plate.Font, pfs, plate.Mnemonic, plate.ShortestWord, plate.LongestWord, 0, endCol1)

  		if n > largeN {
  			// Column 2 is a single contiguous block (rows endCol1..N)
  			// anchored at the shared top, eliminating the legacy
  			// two-block collision. The large-N path is SLIP-39 only
  			// (qrc==nil), so no QR is engraved here.
  			off := t.Offset(params.I(44), (plateDims.Y-col1Height)/2)
  			wordColumn(off, constant, plate.Font, pfs, plate.Mnemonic, plate.ShortestWord, plate.LongestWord, endCol1, n)
  		} else {
  			// Engrave (top of) column 2.
  			endCol2 := min(endCol1+maxCol2, n)
  			off := t.Offset(params.I(44), (plateDims.Y-col1Height)/2)
  			wordColumn(off, constant, plate.Font, pfs, plate.Mnemonic, plate.ShortestWord, plate.LongestWord, endCol1, endCol2)

  			// Engrave seed QR.
  			if qrc != nil {
  				const qrScale = 3
  				qrCmd := qrc.Engrave(params.StepperConfig, params.StrokeWidth, qrScale)
  				qrsz := qrc.Size * params.StrokeWidth * qrScale
  				t.Offset(params.I(60)-qrsz/2, (plateDims.Y-qrsz)/2)
  				qrCmd(t.Yield)
  			}

  			// Engrave bottom of column 2.
  			height := (n - endCol2) * pfs
  			off = t.Offset(params.I(44), (plateDims.Y+col1Height)/2-height)
  			wordColumn(off, constant, plate.Font, pfs, plate.Mnemonic, plate.ShortestWord, plate.LongestWord, endCol2, n)
  		}

  		// Engrave title.
  		title := strings.ToUpper(plate.Title)
  		{
  			offy := (plateDims.Y+col1Height)/2 + metaMargin
  			title := engrave.String(plate.Font, params.F(plateSmallFontSize), title)
  			titleWidth, _ := title.Measure()
  			t.Offset((plateDims.X-titleWidth)/2, offy)
  			title.Engrave(t.Yield)
  		}
  	}
  }
  ```

  > **Why this preserves the ≤24 path byte-identically (verified):** the `else` branch emits col1 → col2-top → QR → col2-bottom → title in the **exact original order** and with the original anchors (`pfs = F(4.1)`, `endCol1 = min(16,n)`, `col1Height = pfs*endCol1`, `endCol2 = min(endCol1+4, n)`, the same `(plateY±col1Height)/2` math). The only edits in the ≤24 path are pure renames (`len(plate.Mnemonic)` → `n`) and a block-scope change that does not alter emitted commands. This was confirmed by the existing goldens passing unchanged.

### Task 3.4 — Generate the committed golden, run, expect PASS; confirm fail-on-buggy

- [ ] Generate the N=33 golden under the fixed code:
  ```bash
  cd /tmp/engrave-bugfixes && go test ./backup/ -run TestSLIP39Large -update 2>&1 | tail -2
  ls -l backup/testdata/slip39-33-words.bin
  ```
  Expect: `ok  	seedhammer.com/backup ...` and the `.bin` file present (~5KB).
- [ ] Run the golden + geometry tests against the fixed code:
  ```bash
  cd /tmp/engrave-bugfixes && go test ./backup/ -run 'TestSLIP39Large' -v 2>&1 | grep -E 'RUN|PASS|FAIL'
  ```
  Expect (verified): `TestSLIP39Large` and `TestSLIP39LargeGeometry` both `--- PASS`.
- [ ] **Confirm fail-on-buggy** (the load-bearing gate): temporarily restore the original `frontSideSeed` and verify the golden test FAILS, then restore the fix:
  ```bash
  cd /tmp/engrave-bugfixes
  cp backup/backup.go /tmp/backup_fixed.go
  git show f907eea:backup/backup.go > backup/backup.go   # restore buggy frontSideSeed (also drops BUG-2 guard)
  go test ./backup/ -run TestSLIP39Large 2>&1 | tail -4   # EXPECT FAIL (~16276 knot mismatches)
  cp /tmp/backup_fixed.go backup/backup.go && rm /tmp/backup_fixed.go
  go test ./backup/ -run TestSLIP39Large 2>&1 | tail -2   # EXPECT ok again
  ```
  Expect (verified) on buggy code:
  ```
  --- FAIL: TestSLIP39Large (0.0Xs)
      backup_test.go:NN: spline lengths 16753, 16277, with 16276/16277 knot mismatches
  FAIL
  ```
  > Note: `git show f907eea:backup/backup.go` also reverts the BUG-2 guard; that is fine for this isolated fail-on-buggy check (it only runs `TestSLIP39Large`). Be sure to restore `backup/backup.go` from `/tmp/backup_fixed.go` afterward.

### Task 3.5 — Regression pins (≤24 byte-identical, N=23 fresh pin, N=30 new-path within-bounds)

- [ ] **Zero-churn check** for all committed ≤24 + QR goldens (the #1 risk gate):
  ```bash
  cd /tmp/engrave-bugfixes && go test ./backup/ -run 'TestSeed|TestSLIP39$|TestCodex32|TestText|TestConstantSeedTiming|TestConstantStringTiming' 2>&1 | tail -3
  ```
  Expect (verified): `ok  	seedhammer.com/backup  <time>` — **no golden churn** (`seed-0-words-24`, `seed-1-words-12`, `slip39-0`, `codex32-{0,1}`, `text-*` all byte-identical; these prove the `N>24` predicate is the sole behavioral change and that the shared ≤24 + QR path is untouched).
- [ ] **N=23 freshly-generated pin** (untouched ≤24 path; there is no pre-existing 23-word golden, so this asserts the new code leaves the legacy formula producing legacy output for N=23 ≤ 24). Add to `backup/backup_test.go`:
  ```go
  func TestSLIP39_23WordPin(t *testing.T) {
  	seedDesc := Seed{
  		Mnemonic:     slip39Words(23),
  		ShortestWord: slip39words.ShortestWord,
  		LongestWord:  slip39words.LongestWord,
  		Title:        "7945 #1 1/1",
  		Font:         constant.Font,
  	}
  	side, err := EngraveSeed(params, seedDesc)
  	if err != nil {
  		t.Fatal(err)
  	}
  	compareGolden(t, "slip39-23-words", side)
  }
  ```
  Generate and verify:
  ```bash
  cd /tmp/engrave-bugfixes && go test ./backup/ -run TestSLIP39_23WordPin -update 2>&1 | tail -1
  go test ./backup/ -run TestSLIP39_23WordPin 2>&1 | tail -1
  ```
  Expect: `.bin` created, then `ok`. (This commits `backup/testdata/slip39-23-words.bin`.)
- [ ] **N=30 new-path within-bounds pin** (N=30 newly takes the `N>24` path at the full 4.1mm font — pin its y-range and confirm no font change). Add `"slices"` to the test import block, then add:
  ```go
  func TestSLIP39_30WordInBounds(t *testing.T) {
  	// N=30 takes the rebalanced N>24 path but stays at the full 4.1mm font.
  	pfs, col1Rows, col2Rows, col1Bot, col2Bot := seedLayout(30)
  	if pfs != params.F(4.1) {
  		t.Errorf("N=30 pfsN = %d, want full font %d (4.1mm, no shrink)", pfs, params.F(4.1))
  	}
  	if col1Rows != 15 || col2Rows != 15 {
  		t.Errorf("N=30 rows = %d/%d, want 15/15", col1Rows, col2Rows)
  	}
  	if col2Bot > col1Bot {
  		t.Errorf("N=30 col2 bottom %d > col1 bottom %d (overlap)", col2Bot, col1Bot)
  	}

  	// And the actual engraved plan stays within [0, F(85)] on both axes.
  	seedDesc := Seed{
  		Mnemonic:     slip39Words(30),
  		ShortestWord: slip39words.ShortestWord,
  		LongestWord:  slip39words.LongestWord,
  		Title:        "7945 #1 1/1",
  		Font:         constant.Font,
  	}
  	side, err := EngraveSeed(params, seedDesc)
  	if err != nil {
  		t.Fatal(err)
  	}
  	plateMax := params.F(85)
  	for _, k := range slices.Collect(engrave.PlanEngraving(conf, side)) {
  		if k.Ctrl.X < 0 || k.Ctrl.Y < 0 || k.Ctrl.X > plateMax || k.Ctrl.Y > plateMax {
  			t.Fatalf("N=30 knot %v out of plate bounds [0,%d]", k.Ctrl, plateMax)
  		}
  	}
  }
  ```
  Run:
  ```bash
  cd /tmp/engrave-bugfixes && go test ./backup/ -run 'TestSLIP39_30WordInBounds' -v 2>&1 | grep -E 'PASS|FAIL'
  ```
  Expect (verified): `--- PASS` (pfsN stays at 26240, 15/15 rows, no overlap, plan in bounds — N=30's bbox is y=[0,81.97]mm ⊂ [0,85]mm).

### Task 3.6 — Full package run + commit

- [ ] Full backup + engrave run:
  ```bash
  cd /tmp/engrave-bugfixes && go test ./engrave/... ./backup/... 2>&1 | tail -3
  ```
  Expect: both `ok`.
- [ ] Stage and commit (explicit paths, including both new golden files):
  ```bash
  cd /tmp/engrave-bugfixes
  git add backup/backup.go backup/backup_test.go \
      backup/testdata/slip39-33-words.bin backup/testdata/slip39-23-words.bin
  git -c user.name="Brian Goss" -c user.email="goss.brian@gmail.com" \
      commit -S -s \
      -m "backup: rework frontSideSeed layout for >24-word SLIP-39 plates" \
      -m "The legacy 16+4+4 two-block column-2 layout overlapped for 33-word
  SLIP-39 shares: column 2 needs 17 rows in a 16-row band, so the bottom
  block rose 4.10mm above the top block, garbling the plate (gui.toPlate
  only rejects off-plate bounds, not on-plate overlap). For N>24 rebalance
  into ceil(N/2)/floor(N/2) columns, shrink the font to
  min(F(4.1), 16*F(4.1)/col1Rows) (only at N=33 -> 3.859mm), and emit
  column 2 as one contiguous block. N<=24 and every QR-bearing plate run
  the unchanged path (byte-identical goldens). N=27/30 intentionally move
  to the rebalanced split at the full font. Add 33-word no-overlap golden
  + geometry pins and 23/30-word regression pins." \
      -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
  ```

---

## Final task — full verification

**Files:** none (verification only).

- [ ] Full test of both packages (all three fixes + all new tests):
  ```bash
  cd /tmp/engrave-bugfixes && go test ./engrave/... ./backup/... 2>&1 | tail -5
  ```
  Expect (verified):
  ```
  ok  	seedhammer.com/engrave	<time>
  ok  	seedhammer.com/backup	<time>
  ```
- [ ] `go build` the whole module:
  ```bash
  cd /tmp/engrave-bugfixes && go build ./... 2>&1 | tail -3
  ```
  Expect: no output (clean build).
- [ ] `go vet` the touched packages:
  ```bash
  cd /tmp/engrave-bugfixes && go vet ./engrave/ ./backup/ 2>&1 | tail -5
  ```
  Expect: only the **pre-existing** `testing.ArtifactDir requires go1.26 or later (file is go1.25)` notices for `engrave/engrave_test.go` and `backup/backup_test.go` — these come from the existing golden helpers' `t.ArtifactDir()` calls and the `go.mod` declaring `go 1.25.10` while the toolchain is go1.26.4. They predate this cycle (the implementing diffs do not add `t.ArtifactDir()` calls) and are **out of scope**. There must be **no new** vet findings attributable to this cycle's files. (If desired, confirm a non-touched package is clean: `go vet ./slip39/` → exit 0.)
- [ ] Final fork-state sanity (the worktree is on the feature branch; the fork main is untouched):
  ```bash
  cd /tmp/engrave-bugfixes && git log --oneline -3 && git status --porcelain
  git -C /scratch/code/shibboleth/seedhammer status --porcelain  # must be empty
  ```
  Expect: three new signed commits on `feat/engrave-bugfixes`, empty worktree status, and empty fork-main status.

> **TinyGo device-build gate** (`tinygo build` for the RP2350 controller) is the controller's call **post-merge** — note it but do NOT run it in-plan.

---

## Done-when (acceptance, all verified in the throwaway worktree)

- BUG-1: `TestSafePointerNoUnderflow` FAILS on `f907eea` (`sp.progress=1.84e19 > totalTicks=300` at step 0) and PASSES after the 1-line guard change; existing `TestSafePointer` stays green.
- BUG-2: `TestEngraveSeedStringTooLong` FAILS (panics, caught by recover) on `f907eea` and PASSES (returns `"seed too long to engrave QR"`) after the fix for both dim-37 (93-char) and dim-41 (127-char) codex32 strings; `TestEngraveSeedStringHappy` (dim-33) stays green; `TestConstantQR` not regressed (max dim 33).
- BUG-3: `TestSLIP39Large` golden FAILS on `f907eea` (~16276 knot mismatches) and PASSES after the rework; `TestSLIP39LargeGeometry` pins `pfsN==24696`, 17/16 rows, no-overlap, in-bounds; `TestSLIP39_23WordPin` and `TestSLIP39_30WordInBounds` pass; all committed ≤24 + QR goldens (`TestSeed`/`TestSLIP39`/`TestCodex32`/`TestText`/`TestConstant*Timing`) show zero churn.
- `go test ./engrave/... ./backup/...` all green; `go build ./...` clean; `go vet ./engrave/ ./backup/` shows only the pre-existing `ArtifactDir`/go1.25 notices.

## Risks (carried from the spec, with mitigations executed in this plan)

1. **BUG-1 over-correction → wrong safe point.** Mitigated by the break-based fix and the safe-point **reference** assertion (assertion (b)) plus the unchanged `TestSafePointer` (catches over-correction). Both verified.
2. **BUG-2 fan-out.** Avoided: `bitmapForQRStatic` is NOT converted to error-return; the primary guard + `ConstantQR` early check fully fix the bug without touching `ConstantQRCmd.Engrave`'s signature (the `:616` site is unreachable with an unvalidated size). Verified `TestConstantQR` not regressed.
3. **BUG-3 regressing the shared ≤24 + QR path (#1 risk).** Mitigated by the single `N>24` predicate, the verbatim-order `else` branch, and the byte-identical golden gates — all confirmed green. N=27/30 layout change is deliberate and pinned (N=30 test). Residual legibility of the 3.859mm N=33 font is a subjective hardware judgement flagged in the spec (geometry + engraveability proven; stroke fixed at 0.3mm → glyph ≥ ~12.9× stroke); not a code blocker.

## Post-implementation (mandatory)

After all three commits land on `feat/engrave-bugfixes`, dispatch the **mandatory, non-deferrable independent adversarial execution review** over the whole diff (R0 = plan correctness; this catches implementation-introduced regressions TDD misses). Persist the review verbatim to `design/agent-reports/`. The reviewer MUST treat the N=27/30 golden/layout change as **expected** (only N≤24 and QR plates are required byte-identical). Do not merge with any open Critical/Important.
