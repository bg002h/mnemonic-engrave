# Funds-safety audit — D4: Go preview sidecar & Rust↔Go boundary fidelity

Auditor dimension: **D4** — `preview/` Go sidecar (`main.go`, `layout.go`, `params.go`,
`render_png.go`, `render_svg.go`) + the Rust `me` ↔ Go `me-preview` boundary
(`crates/me-cli/src/preview.rs`, `main.rs::wire_previews`).
Repo: `/scratch/code/shibboleth/mnemonic-engrave`. Submodule pin:
`third_party/seedhammer` @ `713aee2` (upstream v1.4.2).
Toolchain used for probes: `/home/bcg/.local/go/bin/go` (go1.26.4), out-of-tree probe module.

TL;DR: **No critical or important funds-safety bug found in the sidecar itself.** The
sidecar renders the exact validated string with correct glyphs, refuses nothing silently
(missing glyph → panic; oversize → error; QR overflow → error), and the params are a
verbatim, numerically-exact replica of the device's SH2 params. The residual findings are
**verification/fidelity gaps** (moderate/low), the most notable being that the preview's
md1/mk1 QR-and-text model **cannot be cross-checked against the fork's actual on-device
md1/mk1 engraving code, which is not present in this repo** (upstream v1.4.2 has no md1/mk1
support).

---

## Findings

### D4-1 (moderate) — Preview↔fork-device md1/mk1 fidelity is unverifiable from this repo; the "render `s` as both text and QR" model may diverge from what the fork actually engraves

**Files:** `preview/layout.go:16-26,47-62` (mode/QR selection), `preview/main.go:66-104`.

The pinned submodule (upstream v1.4.2) has **no md1/mk1 engraving path** — grep across
`third_party/seedhammer/gui|backup|engrave` for `md1|mk1|ValidMD|ValidMK|validateMdmk`
returns nothing; the only `EncodeCompact` use is the generic descriptor path
(`gui/gui.go:401`). The sidecar therefore replicates the upstream **descriptor** layout
(`validateDescriptor`, `gui/gui.go:399-447`): `qr.L`, `qrScale = 3`, mode order
text+qr → text → qr. Those constants match (verified below).

But there is a structural difference the repo cannot resolve:

- In the upstream descriptor path, **TEXT and QR encode different byte-strings**:
  `Text = desc.Encode()` (readable) vs `QR = qr.Encode(desc.EncodeCompact(), qr.L)` (compact).
- The sidecar for md1/mk1 uses the **single string `s` for BOTH** text and QR
  (`layout.go:16-23`, `engraveBest`→`qr.Encode(s, qr.L)`).

For md1/mk1 that is plausible (the constellation string is already the compact form), and it
is internally consistent (preview text == preview QR == `plate.string`, byte-identical to the
user's validated input — see the sound-results section). **The unverifiable risk:** if the
fork's real md1/mk1 engrave path builds the QR from a *different representation* than the raw
lowercase string, the preview QR diverges from the physically-cut QR. Concretely, a probe
(`qr.Encode`) shows an **uppercased** bech32 md1 encodes to QR **version 3 (size 21)** via
alphanumeric mode, whereas the preview's lowercase byte-mode encodes to **version 5 (size 25)**
— a materially different module pattern for the same logical content. If the fork uppercases
for QR efficiency (a common bech32-QR practice) or encodes the NDEF bytes, a user validating
the preview QR would be validating an artifact that differs from the engraved plate
(funds-safety scope (a): preview-vs-device divergence).

Blast radius is bounded: the load-bearing recovery artifact is the human-readable md1/mk1
**text**, which the preview renders correctly, and on recovery the user scans the *actual*
plate, not the preview. So this is a fidelity/verification gap, not a proven funds-loss path
— hence moderate.

**Concrete test that would close it:** a cross-fidelity golden that renders a known md1 through
BOTH the fork's on-device md1/mk1 engrave path AND `me-preview`, asserting (a) byte-identical
`bspline` command streams for the text and (b) identical QR module bitmaps. Writable only once
the fork's md1/mk1 engrave code is vendored/accessible (today it is not in
`third_party/seedhammer`).

---

### D4-2 (low) — Geometry golden does not detect a device-constant drift on a submodule bump

**Files:** `preview/params.go:8-23`, `preview/params_test.go:24-34`.

`params.go` hardcodes `mm = 6400` and `strokeWidth = 1920` with a comment "Re-verify on any
`third_party/seedhammer` bump." `TestParamsGeometryGolden` computes a bbox from the preview's
**own** hardcoded params fed through the submodule's `PlanEngraving`, so it catches a change to
the submodule's B-spline/timing *algorithm* — but **not** a change to a device param *constant*
the preview never reads (e.g. `tmc2209.Microsteps`, `fullStepsPerRevolution`, the `0.3` stroke
ratio). If a future bump changed `Microsteps`, the device's engraved geometry would change while
the preview's hardcoded `6400` (and the golden derived from it) would not — a silent
preview-vs-device divergence. The cross-check to the device's real constants is currently a
manual comment only.

Probe result: `tmc2209.Microsteps` **is host-importable** (= 256), and
`200/8 * Microsteps == 6400`, `6400*3/10 == 1920`. So a machine-checkable guard is cheap.

**Concrete test:** in `params_test.go`, import `seedhammer.com/driver/tmc2209` and assert
`mm == 200/8*tmc2209.Microsteps` and `strokeWidth == mm*3/10`, so a submodule constant bump
fails the build instead of silently drifting the preview.

---

### D4-3 (low) — SVG and PNG previews render the same geometry at different stroke widths

**Files:** `preview/render_svg.go:39-45` vs `preview/render_png.go:60-83`.

Both renderers walk the identical `engrave.PlanEngraving(...)` centerline (same `C`/`M` knots,
same `dt==0` and pen-up skips), so they show the **same strokes at the same positions** — no
"one shows what the other doesn't" for content/geometry. However the SVG draws with
`stroke-width = 1920` (0.3 mm, round caps, i.e. the physical engraved width) while the PNG draws
**1px Bresenham hairlines with no stroke width** (`drawLine`, always 1px). The PNG therefore
under-represents stroke thickness / legibility. Not a funds-loss path (centerlines identical),
but a fidelity inconsistency between the two artifacts a user may choose between.

**Concrete test:** render the same string to SVG and PNG and assert either (a) the PNG scales
its line thickness by `strokeWidth*scale`, or (b) a documented invariant that the PNG is
centerline-only; a regression test pinning the intended behavior.

---

## Checked and found SOUND (negative results)

**1. Params are a verbatim, numerically-exact replica (field-by-field).**
`engrave.Params` = `{StrokeWidth, Millimeter, StepperConfig{Speed, EngravingSpeed,
Acceleration, Jerk, TicksPerSecond}}` (7 fields; struct def `engrave/engrave.go:22-44`).
Every field is set by `preview/params.go` and equals both the tinygo device source
(`cmd/controller/platform_sh2.go:180-407`) and the host-compilable reference
(`gui/gui_test.go:336-360`):

| field | device | preview | value |
|---|---|---|---|
| Millimeter (`mm`) | `200/8*Microsteps` | `6400` | 6400 |
| StrokeWidth | `0.3*mm` | `1920` | 1920 |
| Speed | `30*mm` | `30*mm` | 192000 |
| EngravingSpeed | `8*mm` | `8*mm` | 51200 |
| Acceleration | `250*mm` | `250*mm` | 1600000 |
| Jerk | `2600*mm` | `2600*mm` | 16640000 |
| TicksPerSecond | `topSpeed(=30*mm)` | `30*mm` | 192000 |

No field is omitted or divergent. `Microsteps=256` confirmed via probe (`stepExp=8`,
`driver/tmc2209/tmc2209.go:22-25`).

**2. Glyph coverage of the md1/mk1 alphabet is complete and monospace; a missing glyph is
LOUD, not silently skipped.** Alphabet = bech32 charset `qpzry9x8gf2tvdw0s3jn54khce6mu7l` ∪
HRP `{m,d,k}` ∪ separator/digits `{0-9}` = 32 distinct chars (b/i/o never appear — excluded
from bech32 data and absent from the `md`/`mk` HRPs). Probe over `sh.Font.Decode`:
**all 32 present, every advance = 4000 = the 'W' advance** used by `EngraveText` as the
fixed `charWidth` (`backup/backup.go:259-263`) → 0 missing, 0 width-mismatch, genuinely
monospace. Missing/zero-advance runes (e.g. `_`, or any rune ≥ 0x7F) return `found=false`
(`font/vector/font.go:72-85`), and `engrave.String` then **panics**
(`engrave/engrave.go:1357-1360`) — which crashes the sidecar → non-zero exit → the Rust caller
reports a render failure. No silent character skip is possible.

**3. Fit / clipping bounds exactly match the device; oversize is refused loudly.**
Preview `fits()` (`layout.go:28-32`) checks the measured bbox ⊂ `[3mm, 82mm]²` in machine
units (`safetyMarginMM=3`, `plateSizeMM=85`). Device `toPlate` (`gui/gui.go:2471-2484`) checks
⊂ `[safetyMargin*mm, 85*mm − safetyMargin*mm]²` with `safetyMargin=3` (`gui/gui.go:47`) —
identical bounds `[19200, 524800]²`. A payload that overflows the engravable area yields
`engraveBest` "does not fit any plate mode" → non-zero exit (test
`TestRenderOversizeExitsNonZero`). No silent off-plate clipping.

**4. QR content is byte-faithful and overflow is loud.** `qr.Encode(s, qr.L)` — level **L**
matches the device descriptor path (`gui/gui.go:401`), `qrScale=3` matches (`layout.go:14`,
`gui/gui.go:405`). Probes: content is the exact `s` bytes; a 1-char change produces a different
bitmap (no silent collision); payloads > 2953 bytes (QR-40 byte-mode / level-L capacity) return
`ERROR: text too long to encode as QR` — **not** a silent truncation/wrap. Overflow propagates
out of `engraveBest`/`engraveMode` → non-zero exit.

**5. Rust↔Go boundary is clean.**
- **Payload:** the string handed to the sidecar is `plate.string` = the **verbatim, trimmed,
  pristine-validated** input line (`bundle.rs:180`, `:205/234/260/287` `Some(s.clone())`),
  byte-identical to the string the single-string `me` path converts to NDEF. It is piped on
  **stdin** (`preview.rs:151-159`), never argv/env → no process-table leakage.
- **Version handshake enforced:** `wire_previews` requires `sidecar --version` == the crate
  version **exactly**; any mismatch, an empty version (plain `go build` with no `-ldflags`),
  or an unparseable version → `EXIT_USAGE (2)` with no render (`main.rs:245-259`,
  `preview.rs:94-108`). A stale sidecar cannot silently render.
- **Exit codes:** render failure (bad/oversize input) → `EXIT_INVALID (4)`; spawn/IO/version
  failure → `EXIT_USAGE (2)`; sidecar absent → graceful degrade, note on stderr, `exit 0`
  with the manifest+checklist still emitted (`main.rs:227-296`).
- **BrokenPipe handling:** an EPIPE on stdin write is swallowed so the child's real exit
  status/stderr surfaces (`preview.rs:155-159`) — errors are not masked.

**6. ms1 never reaches the sidecar (secret containment).** `run_bundle` does a classify-only
pre-scan and returns `RefusedSecret` before building any manifest if any `ms` line is present
(`bundle.rs:186-192`). Independently, `wire_previews` skips `PlateKind::Ms1` and any plate with
`string: None` (`main.rs:271-277`); the trailing ms1 reminder plate carries `string: None`
(`bundle.rs:301`). Double-guarded. Preview output files are public md1/mk1 content written
`0o644` — acceptable, as md1 (descriptor) and mk1 (key card) are public constellation artifacts
(only ms1 is secret, and it is excluded).

**7. SVG renderer is a faithful port of seedhammer's own golden renderer.**
`render_svg.go:22-46` matches `internal/golden/golden.go::Vectorize` (`:177-197`) command-for-
command: skip `dt==0`; pen-down → `C c1 c2 c3` (implicit C0 = prior C3, preserving B-spline G1
continuity); pen-up → `M c3`; single accumulated `<path>`; `stroke-width=strokeWidth`,
round cap/join. Only the viewBox differs (golden adds a 20-unit display margin; the preview
crops tight) — cosmetic, the stroke path data is structurally identical.

**8. Re-rangeability and partial-failure are safe.** `backup.EngraveText` returns a plain
re-invocable closure, so `fits()` and the renderers can each range it independently
(confirmed: tests produce non-empty SVG cubics and non-empty PNG). `os.WriteFile` writes the
whole payload or errors; a render failure mid-loop returns a non-zero code from `wire_previews`
before the manifest is serialized, so `me` exits non-zero rather than emitting a
partial-but-complete-looking manifest. No silent partial success.

## FOLLOWUPS cross-check
No open item in `design/FOLLOWUPS.md` concerns the preview sidecar's fidelity/glyph/params
surface; the resolved `me-bundle-preview-sidecar` (Phase B) item documents the v1.4.2 pin and
the "replicated validateMdmk" intent, which D4-1 flags as unverifiable against source in-repo.
None of the D4 findings duplicate a known deferred item.

## Coverage note / limits
The single unavoidable gap (D4-1) is that the fork's on-device md1/mk1 engrave path is not in
this repo, so the "preview == what the device cuts" claim is verified only against the upstream
**descriptor** path and internal consistency, not against the fork's md1/mk1 code. Everything
checkable in-repo (params, glyphs, fit bounds, QR content/overflow, the Rust↔Go boundary,
SVG↔golden equivalence, ms1 containment) was checked and is sound.
