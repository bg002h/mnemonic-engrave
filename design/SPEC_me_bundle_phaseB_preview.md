# Spec — `me bundle --preview` Phase B (faithful plate-preview sidecar)

- **Status:** Draft for review (promoted from `DESIGN_me_bundle_preview.md` §B + the GREEN design R0/R1). Pending spec self-review → user review → spec R0 architect gate.
- **Date:** 2026-06-16
- **Provenance:** FOLLOWUP `me-bundle-preview-sidecar`; recon `cycle-prep-recon-me-bundle-preview-sidecar.md` (me-repo `8993579`, seedhammer v1.4.2 ref, fork `6ab12c0`). Brainstorm decisions (Phase A Q&A + this cycle): full scope, **prebuilt sidecar**, **bundled signed archive**, **no runtime network**, **exact B-spline SVG**, **SVG + optional `--png`**.
- **SemVer:** new `me-preview` binary + `me bundle --preview` flag ⇒ **MINOR**, `me` v0.2.x → **v0.3.0**. The Go sidecar is versioned in lockstep with `me`.

## 1. Goal
Render a **faithful** host-side image of each public plate a wallet backup will engrave — pixel-faithful to what the SeedHammer II device cuts — so the user can eyeball a plate before engraving. A Go sidecar `me-preview` reuses SeedHammer's upstream `engrave`/`backup` rendering; `me` (Rust) does all validation and orchestration. No secret ever reaches the sidecar; no network.

## 2. Scope & non-goals
- **In scope (v0.3.0):** the `me-preview` Go sidecar (render one validated public string → SVG, optional PNG); `me bundle --preview <dir>` integration (drive the sidecar per public plate; `--version` staleness check; graceful degrade); the fidelity contract (replicated SH2 params + exact B-spline strokes); **AND the signed cross-platform release-CI** (§10) that bundles `me` + `me-preview` into per-platform archives with `SHA256SUMS` + a minisign signature.
- **Non-goals:** no `ms1` rendering (the tool never has it); no on-device interaction; no SeedHammer source change (sidecar only READS upstream libs); no preview of the *engraving motion/timing* (geometry only).

## 3. Architecture — trust split & dependency pin
- **`me` (Rust):** validates (Phase A), then for each PUBLIC plate shells out to `me-preview`. Never sends `ms1`. Pure Rust + the sidecar binary.
- **`me-preview` (Go):** input = one validated public md1/mk1 string + mode; output = an SVG (optional PNG) of that plate. No validation, no secrets, no network, no stdin secrets beyond the public string `me` passes.
- **Dependency pin:** the sidecar's `go.mod` pins **UPSTREAM seedhammer v1.4.2** (`module seedhammer.com`) via a **git submodule** at `third_party/seedhammer` (pinned to tag v1.4.2 = commit `713aee2`) + `replace seedhammer.com => ../third_party/seedhammer` (relative to `preview/go.mod`, which sits one level under the repo root, as does `third_party/`), plus `github.com/seedhammer/kortschak-qr v0.3.2`. **Why a submodule, not the Go proxy:** the module proxy tops out at `seedhammer.com v1.4.1` — v1.4.2 is NOT fetchable via `require seedhammer.com v1.4.2`, so a submodule (immutable, auditable SHA, recorded in git history) is the only reproducible option; it also makes the `replace` path identical for dev and CI (both `checkout --recurse-submodules`), retiring the old local `../../../seedhammer-ref-v1.4.2` dev path. It imports `backup`, `engrave`, `bspline`, `bezier`, `font/sh`, `font/vector` — **NOT `gui` and NOT `cmd/controller`** (host-portable, decoupled from PR #35).

## 4. The sidecar `me-preview` (Go)

### 4.1 CLI contract (how `me` invokes it; stable, machine-oriented)
- `me-preview --version` → prints a single line `me-preview <SEMVER>` to stdout, exit 0. `<SEMVER>` MUST equal the `me` crate version it ships with (the lockstep pin). **The `main` package MUST declare `var version string`** — it is the link-time target the build sets via `-X main.version=$VERSION` (§10.2); without the declaration the Go linker silently ignores `-X` and `--version` would print empty, breaking the lockstep (resolves m-2-a).
- `me-preview render [--mode <text+qr|text|qr>] --format <svg|png> --out <file>` reading the **string from stdin** (public md1/mk1; never argv). Writes the image to `--out` (or stdout if `-`). Exit 0 on success; non-zero with a stderr message on failure (e.g. string too large for ANY mode → the no-fit condition `validateMdmk` hits).
- **The sidecar owns mode selection** (so `me` needs no plate-fit logic): with no `--mode`, it replicates `validateMdmk`'s loop — try TEXT+QR, then TEXT, then QR-only — and renders the **first mode that fits a plate** (the device's preferred variant). `--mode` forces a specific one (errors if it doesn't fit). On success it prints the chosen mode to stdout (`mode <text+qr|text|qr>`) so `me` can record it.

### 4.2 Plate layout — replicate `validateMdmk` against upstream libs
The sidecar reproduces the fork's `gui/gui.go:validateMdmk` (`6ab12c0`) layout decisions using UPSTREAM `backup`/`engrave`:
- QR: `qr.Encode(s, qr.L)` (error-correction level **L**), `qrScale = 3`.
- Build `backup.Text{ Paragraphs: []backup.Paragraph{ <mode> }, Font: sh.Font }` where `<mode>` ∈ { `{Text:s, QR:qrc, QRScale:3}` (text+qr), `{Text:s}` (text), `{QR:qrc, QRScale:3}` (qr) }.
- `eng := backup.EngraveText(params, plate)` → `engrave.Engraving`. **Fit check (replicates `toPlate`, identical in fork & upstream `gui/gui.go`):** plate is 85×85 mm with a 3 mm safety margin; a mode fits iff `bspline.Measure(engrave.PlanEngraving(params.StepperConfig, eng)).Bounds.In(bspline.Bounds{Min: bezier.Pt(3*mm, 3*mm), Max: bezier.Pt(82*mm, 82*mm)})` (where `mm = params.Millimeter = 6400`). The sidecar replicates the constants `plateSizeMM=85`, `safetyMarginMM=3` in `params.go` — it does NOT import `gui`. If no mode fits → exit non-zero so `me` can report it (mirrors Phase A oversize handling).

### 4.3 `engrave.Params` — replicate the SH2 device values (THE fidelity crux)
The canonical params live in `cmd/controller/platform_sh2.go` (`//go:build tinygo && rp`, **not host-importable**): `engrave.Params{ StrokeWidth: strokeWidth, Millimeter: mm, StepperConfig: engraverConf }`. **A host-compilable copy with the IDENTICAL values/formulas exists in the v1.4.2 ref at `gui/gui_test.go:336-359`** — the sidecar cites BOTH (`platform_sh2.go` canonical + `gui_test.go` as the non-TinyGo verification anchor). The exact replicated values:
- `mm = Millimeter = 6400` (= 200 fullSteps/rev ÷ 8 mm/rev × 256 microsteps).
- `StrokeWidth = 0.3·mm = 1920`.
- `StepperConfig{ TicksPerSecond: 30·mm (=192000), Speed: 30·mm (=192000), EngravingSpeed: 8·mm (=51200), Acceleration: 250·mm (=1600000), Jerk: 2600·mm (=16640000) }`. **NB: `TicksPerSecond == Speed == topSpeed = 30·mm` is a real SH2 hardware equality, not a coincidence — comment it in `params.go` to prevent drift** (resolves m-4).

The sidecar **replicates these exact constants** in a host-side `params.go`, with:
- a doc comment citing both source files + the ref SHA;
- a **drift guard**: a Go test asserting `bspline.Measure(PlanEngraving(StepperConfig, EngraveText(params, <fixed reference plate>)))` `.Bounds` equals a committed golden bbox — any param change alters the `T` weighting → the Bézier control points → the measured bounds, so drift is caught (we can't import the gated original to diff directly; the geometry-golden is the proxy).
- fidelity contract: "faithful to the SH2 device **given these replicated params at the pinned v1.4.2 ref**; re-verify on any seedhammer-ref bump."

### 4.4 Rendering — exact B-spline strokes → SVG (+ optional PNG)
- Convert the engraving to the device's planned curve: `curve := engrave.PlanEngraving(params.StepperConfig, eng)` → `bspline.Curve` (= `iter.Seq[bspline.Knot]`; `Knot{ Ctrl bezier.Point, T uint, Engrave bool }`). With the correct replicated params this is the SAME time-parameterized curve the device traces.
- Walk `curve` through a `bspline.Segment`: each `seg.Knot(k)` returns `(bezier.Cubic, ticks uint, engrave bool)` — **the third value is the pen-down flag, not validity** (resolves m-1). **Skip segments where `engrave == false`** (pen-up travel; also the first few window-priming segments have `C0 == {0,0}` — naturally pen-up, so skipped) (resolves m-3).
- **Emit each pen-down `bezier.Cubic` as an SVG cubic-Bézier path directly** from its exported control points `C0,C1,C2,C3` (`bezier/bezier.go`): `M C0.x C0.y C C1.x C1.y C2.x C2.y C3.x C3.y`. This is **exact** (no sampling/faceting) and reuses the device's own curve math ⇒ glyph B-splines render faithfully (resolves design m-5 AND spec-R0 I-2 — direct cubic emission, not polyline sampling).
- Canvas/viewBox from `bspline.Measure(curve).Bounds` (`Min`/`Max`/`Dx()`/`Dy()`; machine units, optionally scaled to mm via `params.Millimeter`). **Note: `Measure` returns a conservative convex-hull bbox, so the viewBox has a small margin beyond the actual strokes** (resolves m-2). Stroke width from `params.StrokeWidth`. Background = the 85×85 mm plate rectangle.
- `--format png` (opt-in): rasterize the SAME cubics by sampling them (`bezier.Sample`/`Interpolator`, or `golang.org/x/image` — already transitively available) onto an `image.RGBA` and PNG-encode; no SVG→PNG shell-out. (Sampling is acceptable for the raster path; the SVG remains the exact/authoritative artifact.)

## 5. `me` integration (Rust) — `me bundle --preview`
- Add `--preview <DIR>` (and `--png`) to the `bundle` subcommand. When given: after building the manifest (Phase A), for each plate with `kind != ms1`, invoke `me-preview render --format <svg|png> --out <DIR>/plate-<N>.<ext>` piping the plate `string` to its stdin (NO `--mode` — the sidecar picks the preferred fitting mode per §4.1, so `me` needs no plate-fit logic). Record the written path in the manifest (`plates[].preview`), and note rendered files on stderr. `ms1` plate is never rendered.
- **Sidecar discovery + version check (resolves design I-4):** locate `me-preview` next to the running `me` executable (via `std::env::current_exe`) else on `$PATH`. Before the first render, run `me-preview --version` and compare to `me`'s own version (`env!("CARGO_PKG_VERSION")`): **mismatch → refuse `--preview` with a clear error** (exit 2) naming both versions (never a silent stale-layout render). **Absent `me-preview` + `--preview` → graceful degrade**: emit the manifest + checklist as normal, print a stderr note that preview was skipped (how to install the sidecar), exit 0. No runtime network, ever.
- Without `--preview`, `me bundle` is byte-for-byte Phase A behavior.

## 6. Error handling / exit codes (consistent with Phase A: 0/2/3/4)
- `--preview` + version mismatch, or `--preview <DIR>` unwritable/not-a-dir → exit 2. Sidecar render failure (e.g. string too large for any plate) → surface the sidecar's stderr, exit 4. `me-preview` absent + `--preview` → exit 0 (degrade, with note). `ms1`/validation/integrity failures are unchanged from Phase A (3/4) and happen BEFORE any render.

## 7. File structure
- **New** `third_party/seedhammer` — git submodule (`https://github.com/seedhammer/seedhammer.git` @ v1.4.2 / `713aee2`), `.gitmodules` committed.
- **New** `preview/` Go module in me-repo: `go.mod` (`replace seedhammer.com => ../third_party/seedhammer` + `require kortschak-qr v0.3.2`), `main.go` (CLI: `--version` / `render`), `params.go` (replicated SH2 `engrave.Params` + plate-fit constants + source citation), `layout.go` (validateMdmk-equivalent: qr.L/scale3/modes/EngraveText/fit-check), `render_svg.go` (PlanEngraving → bspline.Segment → direct cubic-Bézier SVG), `render_png.go` (raster), `*_test.go` (geometry-golden drift guard + render smoke).
- **New** `.github/workflows/release.yml` (the §10 release-CI), `minisign.pub` at repo root (public key, also in README).
  - **`go.mod` note (resolves spec-R0 I-1):** the upstream `bspline/optimize.go` imports `gonum.org/v1/gonum` unconditionally (no build tag), so `go mod tidy` will add `gonum v0.17.0` as an **indirect** require and the `me-preview` binary will be substantially larger (~tens of MB) than its own LOC suggests. This is an accepted trade-off (excluding it would require an upstream build-tag change, which is out of scope — the sidecar only uses `bspline.{Segment,Knot,Curve,Measure,Bounds}`, never `InterpolatePoints`). Note it so the dependency set isn't surprising.
- **Modify** `crates/me-cli/src/main.rs` — `--preview <DIR>`/`--png` on `Command::Bundle`; sidecar discovery + `--version` check + degrade; per-plate invocation.
- **Modify** `crates/me-cli/src/bundle.rs` or a new `preview.rs` — the Rust side: sidecar locate/spawn helper, mode selection, manifest `preview` field. Add `preview: Option<String>` to `PlateEntry` (`manifest.rs`, `skip_serializing_if = "Option::is_none"`).
- **New** `crates/me-cli/tests/` cross-lang preview test (auto-skips when `go` absent, like `cross_lang.rs`).
- **Modify** `Cargo.toml` → `0.3.0`; `CHANGELOG.md`.

## 8. Testing
- **Sidecar (Go):** `--version` prints the pinned semver; `render` on a known md1 + mk1 (Phase A's vectors) produces non-empty SVG containing `<path`/`<svg` with a plausible stroke count; the **params-drift geometry-golden** (a committed `bspline.Measure` bounding-box for a fixed reference string — fails if replicated params change geometry); PNG smoke (valid PNG header, non-trivial size); oversize string → non-zero exit.
- **Rust↔Go cross-lang (auto-skip when `go` absent):** build `me-preview`, run `me bundle --preview <tmp>` on md1+mk1, assert one SVG per public plate (none for ms1), each non-empty; the `--version`-mismatch path errors (exit 2); a missing-sidecar path degrades (exit 0, manifest still emitted).
- **Rust unit:** sidecar discovery (current_exe then PATH); mode selection; manifest `preview` field serialization.
- All existing Phase A + converter tests stay green; clippy `-D warnings` + fmt + `go vet`/`gofmt` clean.
- **Release-CI (§10):** a non-tag PR job does a **build-only dry-run** of the full target matrix (every Go target cross-compiles; the Rust matrix builds) so breakage is caught off-tag; the tag job additionally assembles + `sha256sum` + minisign-signs (signing only on tags, key from Secrets). A unit assertion that `me-preview --version` == `me`'s `CARGO_PKG_VERSION` guards the lockstep pin.

## 9. Lockstep / release
- `me-preview` version == `me` version (the `--version` pin); bump together.
- Seedhammer-ref pin = v1.4.2; on any bump, **re-verify §4.3 replicated params + the geometry-golden** (the cross-pin to record).
- No GUI `schema_mirror` (no SeedHammer change). If `me` gains a toolkit manual, mirror `me bundle --preview`.
- Bump `me` → 0.3.0 + CHANGELOG. The **signed cross-platform release archive** (`me` + `me-preview` + `SHA256SUMS` + `SHA256SUMS.minisig`) is produced by the §10 release-CI; the runtime contract it satisfies is "binaries co-located, version-matched" (§5).
- **Submodule bump procedure:** bumping `third_party/seedhammer` past v1.4.2 = update the submodule SHA + re-run the §4.3 geometry-golden (catches param drift) + re-confirm the replicated params/fit constants against the new ref. Record in CHANGELOG.

## 10. Release CI (`.github/workflows/release.yml`) — tag-triggered
Triggered on a `v*` tag. Produces per-platform archives bundling `me` + `me-preview`, a `SHA256SUMS`, and a minisign signature; attaches them to the GitHub release.

### 10.1 Targets
linux/{amd64,arm64}, macos/{amd64,arm64}, windows/amd64. **windows/arm64 is omitted** (no GitHub-hosted runner; cross-MSVC is impractical) — documented as unsupported in v0.3.0.

### 10.2 Build jobs (parallel, upload artifacts)
- **`me-preview` (Go): ALL targets from ONE `ubuntu-latest` runner.** `CGO_ENABLED=0 GOOS=… GOARCH=… go build -trimpath -ldflags="-s -w -X main.version=$VERSION" -o me-preview-<os>-<arch>[.exe] .` (run in `preview/`; `main.go` is the module-root main per §7). **`$VERSION` is extracted from the crate (`cargo metadata`/`Cargo.toml`)** and injected via `-X main.version=` so `me-preview --version` prints the exact `me` semver (the §4.1/§5 lockstep) — resolves m-2. Confirmed safe: the only CGO in the seedhammer tree is `driver/otp/otp_rp2350.go` (`//go:build tinygo && rp2350`, unreachable); `gonum`/`kortschak-qr`/`font/sh` (`//go:embed sh.bin`) are pure Go. Checkout uses `submodules: true`.
- **`me` (Rust): a 3-runner matrix** — `ubuntu-latest` (native `x86_64-unknown-linux-gnu`; `cross` for `aarch64-unknown-linux-gnu`), `macos-latest` (arm64; builds BOTH `aarch64-apple-darwin` native + `x86_64-apple-darwin` via `rustup target add` — Apple SDK cross, no `cross`), `windows-latest` (`x86_64-pc-windows-msvc`). `cargo build --release --target <triple>` (for reproducibility set the FULL mapping, e.g. `RUSTFLAGS="--remap-path-prefix=$(pwd)=."` — not the bare flag — resolves m-4).

### 10.3 Assemble + sign job (needs: all build jobs)
- `download-artifact` all binaries; for each platform build `mnemonic-engrave-<version>-<os>-<arch>.tar.gz` (`.zip` for windows) containing `me`[.exe] + `me-preview`[.exe] + `minisign.pub` + a README/verify note.
- `sha256sum <archives> > SHA256SUMS`.
- **minisign sign** `SHA256SUMS` → `SHA256SUMS.minisig`: secret key + password from GitHub Secrets `MINISIGN_SECRET_KEY` / `MINISIGN_SECRET_KEY_PASSWORD` (key generated locally via `minisign -G`; secret key NEVER committed; `minisign.pub` committed + in README). Trusted comment includes the tag. **The secret key is written to a temp file only for the signing step and `rm`'d immediately after in an `if: always()` post-step (or use a minisign action that never persists it to disk)** — resolves m-1.
- Upload archives + `SHA256SUMS` + `SHA256SUMS.minisig` to the release (`softprops/action-gh-release`).
- **Optional belt-and-suspenders:** `actions/attest-build-provenance` per archive (verify via `gh attestation verify … --repo bg002h/mnemonic-engrave`) — layered on top of minisign, not a replacement.

### 10.4 Verification (documented in README)
```
minisign -Vm SHA256SUMS -P <pubkey-from-README>   # or -p minisign.pub
sha256sum -c SHA256SUMS --ignore-missing
```
The pubkey is pinned in the README (rotation = an explicit, auditable README change noting the final version of the retired key).

### 10.5 Reproducibility (seedhammer ethos)
`-trimpath` (Go) + `--remap-path-prefix` (Rust) + pinned Go/Rust toolchain versions + the submodule-locked seedhammer SHA make `me-preview` (and best-effort `me`) bit-reproducible, so a sophisticated user can rebuild and compare. **`release.yml` MUST pin concrete toolchains (e.g. `go-version: '1.25.x'`, Rust `toolchain: <dated stable>`), never `latest`/`stable`** (resolves m-3). Document the exact toolchain versions + rebuild steps in the repo.

### 10.6 Trust-anchor rationale (why minisign, not cosign/attestations-only)
A wallet-adjacent tool must be **offline-verifiable** against a **persistent, bookmarkable** trust anchor. minisign's README-pinned public key satisfies both; cosign-keyless/Rekor needs the transparency log reachable at verify time and anchors trust in transient OIDC identity; GH attestations need `gh` + GitHub's Sigstore instance. minisign is the primary; attestations may be added as a secondary audit layer.
