# SPEC — me preview hardening (Cycle A: F8 + F9 + F10)

Status: **GREEN — R0 passed at round 1 (0C/0I, 4 non-blocking nits)** (reviews:
`me-preview-hardening-spec-R0-round0.md` = 0C/2I/4L/1N all folded; `…-round1.md` = GREEN,
all 7 folds verified closed, no fold drift). Cleared for single-implementer TDD.
Closes three confirmed-low funds-audit
follow-ups recorded in `design/FOLLOWUPS.md`:
`me-preview-stale-plates-and-sidecar-output-validation` (F8+F9) and
`me-output-file-permissions` (F10). Evidence: `design/agent-reports/funds-audit-D3-bundle-round0.md`
(D3-1, D3-2), `funds-audit-D5-hygiene-round0.md` (D5-2, D5-3), and the verify verdicts.
Process: R0 architect gate to 0C/0I → single implementer, TDD, worktree → post-impl
adversarial review. Executed locally (cloud routine env failed to start).

Recon (verified against current master `9fafb6b`, 2026-07-09):
- F8: `wire_previews` (crates/me-cli/src/main.rs:262) checks only `!dir.is_dir()` before
  rendering; `render_plate` (preview.rs:128) writes `dir/plate-{idx}.{ext}` with no
  cleanup of pre-existing `plate-*` files. Stale higher-index plates from a prior run
  persist. CONFIRMED.
- F9: `render_plate` (preview.rs:163-171) treats `out.status.success()` (exit 0) as the
  sole success criterion — never stats/re-reads the written `--out` file. A sidecar that
  exits 0 while writing nothing/garbage yields a recorded-valid preview. CONFIRMED.
- F10: NDEF `--out` (main.rs:140) and manifest (main.rs:205) use bare `std::fs::write`
  (mode 0o666 & ~umask → 0o644); Go sidecar `writeOut` (preview/main.go:128) uses
  `os.WriteFile(path, payload, 0o644)`. All world-readable. CONFIRMED.

## Non-goals

- Sidecar discovery integrity (F11) and PNG stroke width / render goldens (F13/F15) are
  separate later cycles — untouched here.
- No change to NDEF bytes, manifest schema, exit-code semantics for existing success
  paths, or the sidecar stdin/stdout contract.

## A1 (F8) — refuse a preview directory containing foreign `plate-*` artifacts

Fail-closed (matches the project's fail-closed posture): before rendering, scan `dir`
for existing files matching the glob `plate-*.svg` / `plate-*.png`. If any exist, refuse:
`EXIT_USAGE` (2) with a message naming the directory and instructing the user to use an
empty/clean directory. This prevents cross-run plate mixing without silently deleting
user files (deletion is riskier than refusal and could clobber an unrelated file that
happens to match).

Placement: in `wire_previews` (main.rs), immediately after the `dir.is_dir()` check and
BEFORE the render loop. Scan with `std::fs::read_dir`; match file names against a small
helper `is_plate_artifact(name) -> bool` (`plate-` prefix AND `.svg`/`.png` suffix).

Acceptance (R0 L4 — test placement):
- Integration test in `cli.rs` (with a version-matched fake so the scan is reached):
  rendering into a dir already containing `plate-2.svg` → exit 2, no render, message
  names the dir. (Fails today: renders and overwrites plate-1 while leaving plate-2.)
- Rendering into an empty dir → unchanged success.
- **Unit test** on `is_plate_artifact` for the near-miss set: `plate-2.svg`/`plate-1.png`
  → true; `notes.txt`, `plate.txt` (wrong ext), `plateau.svg` (no `-` after `plate`),
  `plate-.svg` (edge: accept — it IS a plate artifact form) → classify each explicitly.
  Helper must not over-match.

## A2 (F9) — validate sidecar output before recording a preview

After `render_plate` returns Ok(path), the caller must confirm the file is a plausible
render before setting `plate.preview`. Add validation in `render_plate` itself (so the
guarantee is colocated with the write contract): after `wait_with_output` success,
`std::fs::metadata(&out_path)` must show a regular file with len > 0; then read the first
bytes and check a format signature:
- SVG: the file, trimmed of leading ASCII whitespace, begins with `<svg` or `<?xml`.
- PNG: the first 8 bytes equal the PNG magic `89 50 4E 47 0D 0A 1A 0A`.
On failure, return a new `PreviewError::EmptyOutput { path, reason }` → caller maps to
`EXIT_INVALID` (4) (a render that produced no usable artifact is an invalid outcome,
same class as a render failure), prints the error, and does NOT set `plate.preview`.
The signature check reads only a **bounded prefix** (first 512 bytes) via
`File::read` into a fixed buffer — never the whole file (R0 Nit).

**Match-arm edit (R0 L1):** `wire_previews`'s error mapping currently is
`match e { PreviewError::Render { .. } => EXIT_INVALID, _ => EXIT_USAGE }`. Add an arm
`PreviewError::EmptyOutput { .. } => EXIT_INVALID` — the default `_` would otherwise map
it to EXIT_USAGE (2), not the intended 4.

**Existing-test migration (R0 I1 — MANDATORY, these 3 tests break otherwise):**
- `preview.rs::write_fake_sidecar` currently `cat > "$out"` (echoes the raw stdin). Change
  it to write a **format-appropriate signature stub**: parse `--format` in the fake shell
  script; for `svg` write `printf '<svg/>'`, for `png` write the 8 PNG-magic bytes
  (`printf '\211PNG\r\n\032\n'`). Keep `cat > /dev/null` to still drain stdin.
- `preview.rs::render_plate_writes_file_and_returns_path` asserts `body == raw_string` —
  REWRITE to assert the file exists, is non-empty, and (svg) starts with `<svg`. The raw
  string is no longer the file body.
- `preview.rs::render_plate_png_uses_png_extension` uses `md1x` raw → now writes PNG
  magic via the migrated fake; assertion (extension) unchanged, but it now also passes
  validation.
- `cli.rs::png_flag_renders_png` reuses `cli.rs::write_fake` which writes `<svg/>` even
  under `--png` → under PNG validation the file fails. Migrate `cli.rs::write_fake` the
  same way (branch on `--format`), so `--png` runs write PNG magic. (The SVG e2e
  `bundle_preview_*` tests already write `<svg/>` and stay green.)

Acceptance (hermetic fake sidecar, extends existing scaffolding in preview.rs):
- Fake exits 0 but writes a 0-byte `--out` file → `render_plate` errs (EmptyOutput);
  `wire_previews` returns Some(4); no preview path recorded. (Fails today: Ok + recorded.)
- Fake exits 0 writing `garbage` (no SVG/PNG signature) → EmptyOutput.
- Fake writing a real `<svg …>` → Ok (regression guard for the happy path).
- PNG happy path: 8-byte PNG magic + IHDR stub → Ok.
- All 3 migrated tests above stay green.

## A3 (F10) — restrictive permissions on all written artifacts

Every file `me`/`me-preview` writes containing or depicting md1/mk1 material → owner-only
(0o600 on Unix; no-op semantics on Windows where mode bits differ, guarded by cfg).
Sites:
- main.rs:140 NDEF `--out`: replace `std::fs::write` with a helper `write_private(path,
  bytes)` using `OpenOptions` with `.write(true).create(true).truncate(true)` and, under
  `#[cfg(unix)]`, `.mode(0o600)`; on non-unix the same options without `.mode`.
  **`truncate(true)` is load-bearing (R0 I2):** `fs::write` truncates; without it a
  shrinking overwrite (a smaller manifest over a larger one) would leave trailing stale
  bytes → invalid JSON.
- main.rs:205 manifest: same helper.
- preview/main.go:128 `writeOut`: `os.WriteFile(path, payload, 0o600)` (os.WriteFile
  already truncates).

Note zeroize/leak scope honestly: 0o600 protects at-rest on multi-user hosts; it does
NOT change that the manifest embeds raw strings and previews depict QR — that visibility
is intrinsic to the artifacts' purpose. Defense-in-depth, not a secrecy guarantee. Also
**0o600 binds on CREATE; an overwrite of a pre-existing file keeps its old mode** (R0 L3)
— acceptable residual: the NDEF/manifest targets are user-named and the preview targets
are forced-fresh by A1.

Acceptance:
- After `me --out f.ndef` (Unix): `metadata(f).permissions().mode() & 0o077 == 0`.
- After `me bundle --manifest m.json`: same.
- **Overwrite-shrink regression (R0 I2):** write a large manifest, then a smaller one to
  the same path; assert the file parses as valid JSON (no trailing bytes) and length
  equals the smaller payload.
- Go `writeOut` perm (R0 L2): a **Go unit test** on `writeOut` (or the go-gated real
  cross-lang) asserts the written file's mode & 0o077 == 0. NOT the shell-fake Rust e2e
  (the shell fake is not Go `writeOut`, so it can't exercise this).
- Windows: create/write still succeeds (cfg-guarded path compiles and runs).

## Ordering & verification

A3 first (mechanical, isolated), then A1, then A2 (A1+A2 both touch the preview flow;
A2's fake-sidecar tests are the larger surface). TDD per finding: failing test first;
A3's perm assertions are genuine reds today. Full verification before PR:
`ME_REQUIRE_GO=1 cargo test --locked` (root) + `go test ./...` in preview/, both green,
plus a manual `me bundle --preview` e2e into a fresh tmp dir with the real sidecar.

## Open questions — adjudicated at R0 round 0 (all 4 defaults confirmed sound)

1. A1 refuse-on-dirty vs clean-namespace → **refuse** (fail-closed, never deletes).
2. A2 validate in `render_plate` vs `wire_previews` → **render_plate** (colocated with
   the write contract; the match-arm edit at the caller handles exit mapping — L1).
3. A2 signature strictness → **signature-only** (bounded 512-byte prefix; full XML parse
   is scope creep and the finding is about empty/garbage output).
4. A3 Windows → **cfg-guarded no-op** (D5-2's multi-user-host threat model is Unix).
