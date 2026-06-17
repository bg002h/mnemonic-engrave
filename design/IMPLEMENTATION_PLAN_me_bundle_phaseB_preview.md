# `me bundle --preview` Phase B Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`). Implementers have access to the seedhammer submodule source at `third_party/seedhammer` (after Task 1) — when a micro-detail (exact field/method name, iterator re-range semantics) is uncertain, confirm against that source and compile; do not guess.

**Goal:** Add a `me-preview` Go sidecar that renders each public plate to faithful SVG (optional PNG) by reusing SeedHammer's upstream `backup`/`engrave`/`bspline`, plus `me bundle --preview <dir>` integration (version-checked, graceful-degrade), plus a signed cross-platform release-CI. `me` → v0.3.0.

**Architecture:** Two binaries. `me` (Rust) validates + orchestrates + shells out to `me-preview`; `me-preview` (Go) renders only (no secrets, no network). Sidecar pins seedhammer v1.4.2 via a git submodule. Release-CI bundles both into minisign-signed per-platform archives.

**Tech Stack:** Go (sidecar), Rust (`me`), seedhammer v1.4.2 (`backup`/`engrave`/`bspline`/`bezier`/`font/sh`/`font/vector`), `kortschak-qr v0.3.2`, GitHub Actions + minisign.

**Spec:** `design/SPEC_me_bundle_phaseB_preview.md` (GREEN: render/fidelity R0→R1, release-CI R0→R1). **This plan must pass the plan R0 architect gate before any code.**

---

## File Structure
- **Create** `third_party/seedhammer` — git submodule (`github.com/seedhammer/seedhammer` @ v1.4.2 / `713aee2`) + `.gitmodules`.
- **Create** `preview/` Go module: `go.mod`, `main.go` (CLI + `var version string`), `params.go` (replicated SH2 params + plate constants), `layout.go` (mode selection + EngraveText + fit), `render_svg.go`, `render_png.go`, `params_test.go` (geometry-golden), `render_test.go`, `version_test.go`.
- **Modify** `crates/me-cli/src/manifest.rs` — add `PlateEntry.preview: Option<String>`.
- **Create** `crates/me-cli/src/preview.rs` — sidecar discovery + version check + spawn.
- **Modify** `crates/me-cli/src/main.rs` — `--preview <DIR>` / `--png` on `Command::Bundle`; wire preview after manifest; graceful degrade.
- **Modify** `crates/me-cli/src/lib.rs` — `pub mod preview;`.
- **Create** `crates/me-cli/tests/preview_cross_lang.rs` — auto-skips when `go` absent.
- **Create** `.github/workflows/release.yml`; `minisign.pub` (placeholder until keygen) + README verify section.
- **Modify** `Cargo.toml` → `0.3.0`; `CHANGELOG.md`; `design/FOLLOWUPS.md`.

---

### Task 1: Submodule + preview module scaffold + version bump

**Files:** `.gitmodules`, `third_party/seedhammer` (submodule), `preview/go.mod`, `preview/main.go` (stub), `crates/me-cli/Cargo.toml`

- [ ] **Step 1: Add the submodule pinned to v1.4.2**
```bash
cd /<repo>
git submodule add --name seedhammer https://github.com/seedhammer/seedhammer.git third_party/seedhammer
git -C third_party/seedhammer checkout 713aee2   # tag v1.4.2
```
- [ ] **Step 2: Create `preview/go.mod`**
```
module mnemonic-engrave/preview

go 1.25.10

require (
	github.com/seedhammer/kortschak-qr v0.3.2
	seedhammer.com v0.0.0
)

replace seedhammer.com => ../third_party/seedhammer
```
**`require seedhammer.com v0.0.0`** (NOT v1.4.2) — resolves plan-R0 I-1: the Go module proxy lacks v1.4.2, so `v1.4.2` would fail `go mod tidy`; the `v0.0.0` sentinel + local `replace` is the exact `firmware/ndef-roundtrip/go.mod` precedent. `go 1.25.10` matches the submodule's `go.mod` (resolves m-2).
- [ ] **Step 3: Minimal `preview/main.go` (compiles; real CLI in Task 6)**
```go
package main

import "fmt"

// version is set at build time via -ldflags "-X main.version=<semver>".
var version string

func main() { fmt.Println("me-preview", version) }
```
- [ ] **Step 4: `go mod tidy` + build** (in `preview/`): `go mod tidy && go build .`
Expected: the local `replace` + `require seedhammer.com v0.0.0` resolves seedhammer via the submodule WITHOUT a proxy lookup (the proxy lacks v1.4.2); `go mod tidy` adds `gonum` as an indirect require; builds a `preview` binary. The `v0.0.0` sentinel is what prevents the v1.4.2 proxy 404 — do NOT change it back to v1.4.2.
- [ ] **Step 5: Bump crate** — `crates/me-cli/Cargo.toml` `version = "0.3.0"`.
- [ ] **Step 6: Commit**
```bash
git add .gitmodules third_party preview/go.mod preview/go.sum preview/main.go crates/me-cli/Cargo.toml
git commit -m "build(me): add seedhammer v1.4.2 submodule + preview Go module scaffold; bump 0.3.0"
```

---

### Task 2: `params.go` — replicated SH2 params + plate constants + drift-guard

**Files:** `preview/params.go`, `preview/params_test.go`

- [ ] **Step 1: Write the geometry-golden drift-guard test (failing)**
`preview/params_test.go`:
```go
package main

import (
	"testing"

	"seedhammer.com/bspline"
	"seedhammer.com/engrave"
)

// MD1_REF is a fixed reference string; its engraved geometry bbox is a stable
// proxy for "params unchanged". Use the Phase A vector.
const MD1_REF = "md1yqpqqxqq8xtwhw4xwn4qh"

func TestParamsGeometryGolden(t *testing.T) {
	eng, mode, err := engraveBest(MD1_REF) // layout.go (Task 3)
	if err != nil {
		t.Fatalf("engrave: %v", err)
	}
	_ = mode
	b := bspline.Measure(engrave.PlanEngraving(params.StepperConfig, eng)).Bounds
	// Golden bbox (machine units) — fill from the first green run, then PIN.
	// A param change alters T-weighting -> bounds -> this fails.
	if b.Dx() != wantDx || b.Dy() != wantDy {
		t.Fatalf("geometry drift: got %dx%d want %dx%d (replicated params may be stale)", b.Dx(), b.Dy(), wantDx, wantDy)
	}
}
```
(Implementer: `wantDx/wantDy` are consts captured from the first green run and then pinned; document the source ref SHA.)

- [ ] **Step 2: Implement `params.go`**
```go
package main

import "seedhammer.com/engrave"

// Replicated VERBATIM from seedhammer v1.4.2 cmd/controller/platform_sh2.go
// (//go:build tinygo && rp — not host-importable) and cross-checked against
// the host-compilable gui/gui_test.go. Re-verify on any third_party/seedhammer bump.
const mm = 6400 // Millimeter in machine units (200/8 * 256 microsteps)
const strokeWidth = 1920 // 0.3 * mm

// NB: TicksPerSecond == Speed == topSpeed (30*mm) is a real SH2 hardware
// equality on the SH2, not a coincidence.
var params = engrave.Params{
	StrokeWidth: strokeWidth,
	Millimeter:  mm,
	StepperConfig: engrave.StepperConfig{
		Speed:          30 * mm,   // topSpeed = 192000
		EngravingSpeed: 8 * mm,    // 51200
		Acceleration:   250 * mm,  // 1600000
		Jerk:           2600 * mm, // 16640000
		TicksPerSecond: 30 * mm,   // == topSpeed
	},
}

// Plate geometry (replicated from gui toPlate): 85x85 mm, 3 mm safety margin.
const plateSizeMM = 85
const safetyMarginMM = 3
```
- [ ] **Step 3: Run** `go test ./... -run Golden` (fails until layout.go exists; sequence after Task 3 — or stub `engraveBest` to unblock). Pin `wantDx/wantDy` after the first green run.
- [ ] **Step 4: Commit** `feat(me-preview): replicated SH2 engrave.Params + plate constants + geometry-golden`

---

### Task 3: `layout.go` — mode selection + EngraveText + fit check

**Files:** `preview/layout.go`, `preview/layout_test.go`

- [ ] **Step 1: Failing test** — `engraveBest(MD1_REF)` returns a non-empty Engraving + a mode in {"text+qr","text","qr"}; an oversize string returns an error.
- [ ] **Step 2: Implement** (replicates fork `validateMdmk` against upstream libs):
```go
package main

import (
	"fmt"
	"iter"

	qr "github.com/seedhammer/kortschak-qr"
	"seedhammer.com/backup"
	"seedhammer.com/bspline"
	"seedhammer.com/engrave"
	"seedhammer.com/font/sh"
)

const qrScale = 3

func paragraphFor(mode string, s string, qrc *qr.Code) (backup.Paragraph, bool) {
	switch mode {
	case "text+qr":
		return backup.Paragraph{Text: s, QR: qrc, QRScale: qrScale}, true
	case "text":
		return backup.Paragraph{Text: s}, true
	case "qr":
		return backup.Paragraph{QR: qrc, QRScale: qrScale}, true
	}
	return backup.Paragraph{}, false
}

func fits(eng engrave.Engraving) bool {
	b := bspline.Measure(engrave.PlanEngraving(params.StepperConfig, eng)).Bounds
	lo, hi := safetyMarginMM*mm, (plateSizeMM-safetyMarginMM)*mm
	return b.In(bspline.Bounds{Min: bezier.Pt(lo, lo), Max: bezier.Pt(hi, hi)})
}

// engraveBest renders the first fitting mode (text+qr > text > qr), like validateMdmk.
// If `force` is non-empty, only that mode is tried.
func engrave_(mode, s string, qrc *qr.Code) (engrave.Engraving, error) {
	p, ok := paragraphFor(mode, s, qrc)
	if !ok {
		return nil, fmt.Errorf("unknown mode %q", mode)
	}
	return backup.EngraveText(params, backup.Text{Paragraphs: []backup.Paragraph{p}, Font: sh.Font}), nil
}

func engraveBest(s string) (engrave.Engraving, string, error) {
	qrc, err := qr.Encode(s, qr.L)
	if err != nil {
		return nil, "", fmt.Errorf("qr encode: %w", err)
	}
	for _, mode := range []string{"text+qr", "text", "qr"} {
		eng, err := engrave_(mode, s, qrc)
		if err != nil {
			return nil, "", err
		}
		if fits(eng) {
			return eng, mode, nil
		}
	}
	return nil, "", fmt.Errorf("string does not fit any plate mode")
}

func engraveMode(mode, s string) (engrave.Engraving, error) {
	qrc, err := qr.Encode(s, qr.L)
	if err != nil {
		return nil, err
	}
	eng, err := engrave_(mode, s, qrc)
	if err != nil {
		return nil, err
	}
	if !fits(eng) {
		return nil, fmt.Errorf("mode %q does not fit a plate", mode)
	}
	return eng, nil
}
```
(Add `"seedhammer.com/bezier"` import. Implementer: confirm `bspline.Bounds.In`, `bezier.Pt`, `backup.EngraveText`/`Text`/`Paragraph`, `qr.Encode`/`qr.L`, `sh.Font` against the submodule; `engrave.Engraving = iter.Seq[Command]` so `iter` import may be unused — drop if so. **Re-range caution:** `fits` and the caller both range the Engraving via PlanEngraving — if `backup.EngraveText`'s returned Seq is not safely re-rangeable, build a fresh Engraving per use.)
- [ ] **Step 3: Run** `go test ./...` green. Backfill Task 2's golden consts.
- [ ] **Step 4: Commit** `feat(me-preview): plate layout + mode selection (validateMdmk-equivalent) + fit check`

---

### Task 4: `render_svg.go` — exact cubic-Bézier SVG

**Files:** `preview/render_svg.go`, `preview/render_test.go`

- [ ] **Step 1: Failing test** — `renderSVG(eng)` for `engraveBest(MD1_REF)` returns a string containing `<svg`, at least one `<path`, and a `viewBox`.
- [ ] **Step 2: Implement**
```go
package main

import (
	"fmt"
	"strings"

	"seedhammer.com/bspline"
	"seedhammer.com/engrave"
)

// renderSVG mirrors seedhammer's OWN SVG renderer (internal/golden/golden.go:175-194)
// (resolves plan-R0 C-1): a SINGLE <path> accumulating commands —
//   pen-UP segment  (!line) -> "M C3.x C3.y"   (reposition cursor to next run start)
//   pen-DOWN segment (line) -> "C C1 C2 C3"     (C0 is the IMPLICIT cursor = prior C3;
//                                                NO M, NO C0 — preserves B-spline G1 continuity)
//   skip dt==0 (zero-duration) segments (NOT just pen-up).
// Emitting "M C0 C ..." per cubic (as a naive reading would) re-specifies C0 and breaks continuity.
func renderSVG(eng engrave.Engraving) string {
	bounds := bspline.Measure(engrave.PlanEngraving(params.StepperConfig, eng)).Bounds

	var d strings.Builder
	var seg bspline.Segment
	for k := range engrave.PlanEngraving(params.StepperConfig, eng) {
		c, dt, line := seg.Knot(k)
		if dt == 0 {
			continue // zero-duration (incl. window-priming) — skip
		}
		if line {
			fmt.Fprintf(&d, " C %d %d, %d %d, %d %d", c.C1.X, c.C1.Y, c.C2.X, c.C2.Y, c.C3.X, c.C3.Y)
		} else {
			fmt.Fprintf(&d, " M %d %d", c.C3.X, c.C3.Y) // pen-up: cursor jump
		}
	}

	var b strings.Builder
	fmt.Fprintf(&b, `<svg xmlns="http://www.w3.org/2000/svg" viewBox="%d %d %d %d">`+"\n",
		bounds.Min.X, bounds.Min.Y, bounds.Dx(), bounds.Dy())
	fmt.Fprintf(&b, `<path fill="none" stroke="black" stroke-width="%d" stroke-linecap="round" stroke-linejoin="round" d="%s"/>`+"\n",
		strokeWidth, strings.TrimSpace(d.String()))
	b.WriteString("</svg>\n")
	return b.String()
}
```
(Implementer: this matches `internal/golden/golden.go`'s loop verbatim in structure — read it. Two `PlanEngraving` calls (bounds + walk) are safe — CRUX 2 confirmed `EngraveText`/`PlanEngraving` return re-rangeable closures. Confirm `seg.Knot(k) (bezier.Cubic, uint, bool)`: 2nd = `dt` ticks, 3rd = pen-down `line` flag.)
- [ ] **Step 3: Run** `go test ./... -run SVG` green. **Step 4: Commit** `feat(me-preview): exact cubic-Bézier SVG renderer`

---

### Task 5: `render_png.go` — opt-in raster

**Files:** `preview/render_png.go`, test in `render_test.go`

- [ ] **Step 1: Failing test** — `renderPNG(eng)` returns bytes with a valid PNG header (`\x89PNG`).
- [ ] **Step 2: Implement** — same cubics, sampled via `bezier.Sample` (or `bezier.Interpolator`) onto an `image.RGBA` sized from the bounds (scaled down by a factor so the image is reasonable), draw 1px polylines, `png.Encode`. Use only stdlib `image`/`image/png` + `bezier` (no new heavy dep). Pen-down segments only.
- [ ] **Step 3: Run green. Step 4: Commit** `feat(me-preview): optional PNG raster output`

---

### Task 6: `main.go` — CLI (`--version` / `render`)

**Files:** `preview/main.go`, `preview/version_test.go`

- [ ] **Step 1: Failing tests** — `--version` prints `me-preview <version>`; `render --format svg` reading a string on stdin writes an SVG to `--out` and prints `mode <m>` to stdout; `render --mode text` forces the mode; an oversize string exits non-zero.
- [ ] **Step 2: Implement** the CLI (flag/std `os.Args`): subcommands `render` (`--mode` optional, `--format svg|png`, `--out FILE|-`, string from stdin) and the top-level `--version`. Keep `var version string` (Task 1) as the `-X` target. On `render`: read stdin → `engraveBest` (or `engraveMode` if `--mode`) → `renderSVG`/`renderPNG` → write `--out` → print `mode <m>` to stdout; on no-fit, stderr + exit 1.
- [ ] **Step 3: Run** `go test ./...` + `go vet` + `gofmt -l` clean. **Step 4: Commit** `feat(me-preview): render/version CLI`

---

### Task 7: Rust — manifest `preview` field

**Files:** `crates/me-cli/src/manifest.rs`

- [ ] **Step 1: Failing test** — a `PlateEntry` with `preview: Some("out/plate-1.svg")` serializes a `"preview"` key; `None` omits it (existing Phase A golden unaffected).
- [ ] **Step 2: Implement** — add to `PlateEntry`:
```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
```
Then add `preview: None` to **EVERY** `PlateEntry { … }` struct-literal site, or the build won't compile (resolves plan-R0 I-2): the **4 constructors in `bundle.rs`** (the unchunked-md1 loop, the chunked-md1 loop, the mk1 loop, the trailing ms1 reminder) **AND the `PlateEntry` literals in `manifest.rs`'s own `#[cfg(test)]` module** (the checklist/serialization tests construct them directly). Grep `PlateEntry {` across `bundle.rs` + `manifest.rs` and update each.
- [ ] **Step 3: Run** `cargo test -p mnemonic-engrave` green (incl. the Phase A golden — unchanged because `None` omits). **Step 4: Commit** `feat(me): manifest preview field (Phase B)`

---

### Task 8: Rust — `preview.rs` (sidecar discovery + version check + spawn)

**Files:** `crates/me-cli/src/preview.rs`, `crates/me-cli/src/lib.rs`

- [ ] **Step 1: Failing tests** — `locate_sidecar()` returns the path next to `current_exe` or on PATH, else `None`; `check_version(path, expected)` errors on mismatch.
- [ ] **Step 2: Implement** — `pub fn locate_sidecar() -> Option<PathBuf>` (check `current_exe().parent()/me-preview[.exe]` then `$PATH`); `pub fn sidecar_version(path) -> io::Result<String>` (run `--version`, parse `me-preview <ver>`); `pub fn render_plate(path, string, dir, idx, png) -> Result<String, PreviewError>` (spawn `render --format … --out dir/plate-<idx>.<ext>`, pipe `string` to stdin, return the path). `PreviewError` maps to exit codes. Add `pub mod preview;` to lib.rs.
- [ ] **Step 3: Run green. Step 4: Commit** `feat(me): preview sidecar discovery + version check + spawn`

---

### Task 9: Rust — `me bundle --preview` wiring + graceful degrade

**Files:** `crates/me-cli/src/main.rs`, `crates/me-cli/tests/cli.rs`

- [ ] **Step 1: Failing CLI tests** (use a fake `me-preview` on PATH or skip if absent): `--preview <dir>` with no sidecar → exit 0 + stderr "preview skipped" note + manifest still emitted; with a version-mismatched sidecar → exit 2.
- [ ] **Step 2: Implement** — add `--preview <DIR>` + `--png` to `Command::Bundle`. After building the manifest: if `--preview`, `locate_sidecar()`; absent → stderr note, continue (exit 0, degrade); present → `sidecar_version` vs `env!("CARGO_PKG_VERSION")`, mismatch → eprintln + exit 2; else for each non-ms1 plate `render_plate(...)`, set `plate.preview = Some(path)`, note on stderr. ms1 never rendered. (Reuse the manifest mode the sidecar reports.)
- [ ] **Step 3: Run** `cargo test -p mnemonic-engrave` + clippy `-D warnings` + fmt green. **Step 4: Commit** `feat(me): bundle --preview wiring (version-checked, graceful degrade)`

---

### Task 10: Rust↔Go cross-lang preview test

**Files:** `crates/me-cli/tests/preview_cross_lang.rs`

- [ ] **Step 1: Test** (auto-skip when `go` absent, mirroring `cross_lang.rs`): `go build` the sidecar with `-ldflags -X main.version=<crate ver>` into a temp dir alongside a copy/symlink of the `me` test binary path expectation; run `me bundle --preview <tmp>` on `MD1 + MK1_A + MK1_B`; assert one `.svg` per public plate, none for ms1, each non-empty/contains `<svg`; assert the `--version`-match path renders and a mismatched fake errors exit 2.
- [ ] **Step 2: Run** (with `go` on PATH) green; (without `go`) skips. **Step 3: Commit** `test(me): preview cross-lang round-trip (auto-skip without go)`

---

### Task 11: `release.yml` + minisign pubkey + README verify

**Files:** `.github/workflows/release.yml`, `minisign.pub`, `README.md`

- [ ] **Step 1: Write `release.yml`** per spec §10: triggers on `v*` tag (+ a `build-only` job on PRs, no signing). Jobs:
  - `go-build` (ubuntu): `checkout submodules: true`; loop the 6 GOOS/GOARCH (linux/darwin/windows × amd64/arm64; the spec's 5 + linux/arm64) with `CGO_ENABLED=0 go build -trimpath -ldflags="-s -w -X main.version=$VERSION" -o dist/me-preview-<os>-<arch>[.exe] .` in `preview/`; `$VERSION` from `cargo metadata`/`Cargo.toml`; upload artifacts.
  - `rust-build` matrix: ubuntu (x86_64 native + `cross` aarch64-linux), macos-latest (aarch64-apple native + `rustup target add x86_64-apple-darwin`), windows-latest (x86_64-msvc). `RUSTFLAGS="--remap-path-prefix=$(pwd)=."`. Pin concrete toolchains. Upload artifacts. (windows/arm64 omitted.)
  - `assemble` (needs all; tag only): download artifacts; per platform build `mnemonic-engrave-${TAG}-<os>-<arch>.tar.gz`(/`.zip`) with `me`+`me-preview`+`minisign.pub`+verify-note + **`THIRD_PARTY_LICENSES`** (see below); `sha256sum … > SHA256SUMS`; minisign-sign `SHA256SUMS` (key from Secrets to a temp file, `rm` in `if: always()`); `softprops/action-gh-release` uploads archives + `SHA256SUMS` + `.minisig`. Optional `attest-build-provenance`.
  - **`THIRD_PARTY_LICENSES` (license compliance — load-bearing for the sidecar archive):** `me`/`me-preview` are **MIT** (our code), but `me-preview` statically links **kortschak-qr (BSD-3-Clause)** and **gonum (BSD-3-Clause, + its `THIRD_PARTY_LICENSES/`)**, whose BSD-3 terms REQUIRE the copyright notice + license text be retained in binary distributions. seedhammer is **Unlicense / public domain** (no obligation, but note it for courtesy). So any archive containing `me-preview` MUST bundle a `THIRD_PARTY_LICENSES` file concatenating: kortschak-qr's LICENSE, gonum's LICENSE + its `THIRD_PARTY_LICENSES/*`, and a seedhammer Unlicense note. (The Rust-only crates.io publish of `me` is unaffected — it links neither.) No license CONFLICT: MIT ⊕ BSD-3 ⊕ Unlicense are all permissive; the combined work stays MIT with third-party notices.
- [ ] **Step 2:** Commit a placeholder `minisign.pub` with a `TODO: replace with real pubkey from minisign -G` note + a README "Verifying releases" section (the `minisign -Vm SHA256SUMS -P <pubkey>` + `sha256sum -c` commands). **The real keypair is generated by the maintainer (`minisign -G`); the secret key goes to GitHub Secrets `MINISIGN_SECRET_KEY`/`_PASSWORD`, the pubkey replaces the placeholder.** (This is a maintainer action, flagged — not done by the implementer.)
- [ ] **Step 3:** Validate the workflow YAML (`actionlint` if available; else a careful read). Cannot fully run CI locally. **Step 4: Commit** `ci(me): signed cross-platform release workflow (minisign)`

---

### Task 12: CHANGELOG + FOLLOWUPS + final sweep

**Files:** `crates/me-cli/CHANGELOG.md`, `design/FOLLOWUPS.md`

- [ ] **Step 1:** CHANGELOG `0.3.0`: "Added `me bundle --preview <dir>` (+ `--png`): faithful host-side SVG plate previews via the `me-preview` Go sidecar (pins seedhammer v1.4.2); version-checked with graceful degrade. Signed cross-platform release archives (minisign)."
- [ ] **Step 2:** FOLLOWUPS: mark `me-bundle-preview-sidecar` RESOLVED (Phase B shipped, incl. release-CI). Note the maintainer minisign-keygen action as a one-time release prerequisite.
- [ ] **Step 3:** Full sweep: `cargo test -p mnemonic-engrave` + clippy + fmt; `cd preview && go test ./... && go vet ./... && gofmt -l .`. All green. **Step 4: Commit** `docs(me): CHANGELOG 0.3.0 + FOLLOWUPS (Phase B done)`

---

## Self-Review
**1. Spec coverage:** §3 sidecar/trust/submodule → Tasks 1,2,3. §4.1 CLI+`var version` → Task 6. §4.2 layout/fit → Task 3. §4.3 params+drift-guard → Task 2. §4.4 exact cubic SVG + PNG → Tasks 4,5. §5 me integration (discovery/version/degrade) → Tasks 8,9. §6 exit codes → Tasks 8,9. §7 files → all. §8 tests → Tasks 2,4,5,6,10 (+CI dry-run Task 11). §9 lockstep → Tasks 1,6,9,12. §10 release-CI → Task 11. All covered.
**2. Placeholder scan:** the geometry-golden `wantDx/wantDy` (Task 2) and `minisign.pub` (Task 11) are generate-and-pin artifacts (captured from first green run / maintainer keygen), explicitly flagged — not TODO placeholders. The Go micro-syntax "confirm against submodule" notes are real (the implementer has the source + compiler).
**3. Type consistency:** `engraveBest`/`engraveMode`/`renderSVG`/`renderPNG`/`params` used consistently across Tasks 2–6; `locate_sidecar`/`sidecar_version`/`render_plate`/`PreviewError` across Tasks 8–9; `PlateEntry.preview` (Task 7) consumed in Task 9.
**Known for plan-R0:** confirm `seedhammer.com v1.4.2` `go.mod` require interplay with the local `replace` (Task 1 Step 4 note); `bspline.Segment` zero-value + `seg.Knot` semantics + Seq re-range (Tasks 3,4); `bezier.Pt`/`Cubic` fields (verified in source: `Point{X,Y}`, `Pt(x,y)`, `Cubic{C0..C3}`).

## Execution Handoff
Plan saved to `design/IMPLEMENTATION_PLAN_me_bundle_phaseB_preview.md`. **Must pass the plan R0 architect gate (0C/0I) before any code.** Then: **subagent-driven-development** in an isolated worktree; the `release.yml` (Task 11) gets a careful read-review (can't run CI locally) + the maintainer keygen flagged.
