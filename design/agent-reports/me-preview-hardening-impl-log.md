# Implementation log — me preview hardening (Cycle A: F8 + F9 + F10)

Single-implementer TDD execution of `design/SPEC_me_preview_hardening.md` (GREEN at R0
round 1, 0C/0I). Worktree `/scratch/code/shibboleth/me-cycleA`, branch
`me-preview-hardening`, base master `5d68002` (spec commit) / `9fafb6b` (code base).

Order per spec §"Ordering & verification": **A3 first** (mechanical perms/truncate),
then **A1** (dirty-dir refuse), then **A2** (sidecar-output validation, incl. the
mandatory I1 fake/test migration).

Baseline (before any change): Rust `ME_REQUIRE_GO=1 cargo test --locked` = 82 passed
(lib 54 + main 0 + cli 23 + cross_lang 1 + golden 3 + preview_cross_lang 1), 0 skips;
Go `go test ./...` in `preview/` = ok.

Scratch target dir (outside the worktree): `/scratch/code/shibboleth/me-cycleA-scratch/target`.

---

## Step A3 (F10) — owner-only permissions on written artifacts + truncate

**Tests written first (RED):**
- Go `preview/writeout_test.go::TestWriteOutPermIsOwnerOnly` — asserts `writeOut` on a
  real path yields `mode & 0o077 == 0`. Failure line:
  `writeout_test.go:24: writeOut created a group/other-accessible file: mode 0644, want no bits in 0o077`.
- Rust `cli.rs::perms::ndef_out_file_is_owner_only` — failure line:
  `assertion left == right failed: NDEF --out must be owner-only, got 100644`.
- Rust `cli.rs::perms::manifest_file_is_owner_only` — failure line:
  `assertion left == right failed: manifest must be owner-only, got 100644`.
- Rust `cli.rs::perms::manifest_overwrite_shrink_no_trailing_bytes` (I2 regression guard)
  — GREEN today (`fs::write` truncates); it is a guard for the new `OpenOptions` path.
  Teeth proven: with `.truncate(true)` removed from `write_private`, it FAILS at the
  byte-identity assertion (`cli.rs:689`, trailing stale bytes); restored → PASS.

**Change:**
- `preview/main.go` `writeOut`: `os.WriteFile(path, payload, 0o644)` → `0o600`.
- `crates/me-cli/src/main.rs`: new `write_private(path, bytes)` using
  `OpenOptions::new().write(true).create(true).truncate(true)` + `#[cfg(unix)] .mode(0o600)`;
  replaces `std::fs::write` at the NDEF `--out` site and the manifest site. Doc comment
  records the create-only mode residual (R0 L3) and the load-bearing truncate (R0 I2).

**Final counts after A3:** Go `go test ./...` ok. Rust `ME_REQUIRE_GO=1 cargo test --locked`
= 85 passed (lib 54 + main 0 + cli 26 + cross_lang 1 + golden 3 + preview_cross_lang 1),
0 skips. `cargo clippy --all-targets -- -D warnings` clean.

---

## Step A1 (F8) — refuse a preview dir containing foreign `plate-*` artifacts

**Tests written first (RED):**
- Rust integration `cli.rs::preview::dirty_preview_dir_refused_exit_2` — pre-seeds
  `plate-9.svg` in the outdir, version-matched fake, expects exit 2 + message names the
  dir + no `plate-1.svg` rendered + stale file survives. Failure line (today):
  `Unexpected return code, failed var == 2 … code=0` (renders into the dirty dir).
- Rust unit `main.rs::tests::is_plate_artifact_classifies_near_miss_set` — red by
  non-existence (`is_plate_artifact` undefined).

**Change (`crates/me-cli/src/main.rs`):**
- New pure helper `is_plate_artifact(name) -> bool` = `plate-` prefix AND `.svg`/`.png`
  suffix. Classifies the R0 near-miss set: `plate-2.svg`/`plate-1.png`/`plate-.svg` → true;
  `notes.txt`/`plate.txt`/`plateau.svg` → false.
- `wire_previews`: after the `dir.is_dir()` gate and BEFORE the render loop, `read_dir`
  the target; if any entry name `is_plate_artifact`, refuse with `EXIT_USAGE` (2), naming
  the dir — never deletes. A `read_dir` error also refuses (EXIT_USAGE). Scanned once;
  the loop's own writes are not re-scanned (no self-collision).

**Final counts after A1:** Go ok. Rust = 87 passed (lib 54 + main 1 + cli 27 + cross_lang 1
+ golden 3 + preview_cross_lang 1), 0 skips. Clippy clean.

---

## Step A2 (F9) — validate sidecar output before recording a preview

**Tests written first (RED — validation not yet implemented, render_plate still Ok):**
- `preview.rs::tests::render_plate_rejects_empty_output` — fake exits 0 writing a 0-byte
  `--out`; expects `EmptyOutput`. Failure: `.unwrap_err()` panic at `preview.rs:393`
  (got Ok).
- `preview.rs::tests::render_plate_rejects_garbage_output` — fake writes `garbage`;
  expects `EmptyOutput`. Failure: `.unwrap_err()` panic at `preview.rs:409` (got Ok).
- `cli.rs::preview::empty_sidecar_output_exit_4` — real CLI, fake writes 0-byte out;
  expects exit 4. Failure: `Unexpected return code, failed var == 4 … code=0`.

**Change:**
- `preview.rs`: new `PreviewError::EmptyOutput { path, reason }` variant + its exhaustive
  `Display` arm. New `validate_render_output(out_path, png)` called after the
  `status.success()` check in `render_plate`: `metadata` must show a non-empty regular
  file; then a **bounded 512-byte prefix** read (`File::read` into a fixed `[u8; 512]`)
  must carry the format signature — PNG magic `89 50 4E 47 0D 0A 1A 0A` for png, or (svg)
  `<svg`/`<?xml` after trimming leading ASCII whitespace (`trim_leading_ascii_ws`). Added
  `Read` to the `std::io` import. The check is **format-specific** (keyed on `render_plate`'s
  `png` bool — R0 N2's intended reading).
- `main.rs` `wire_previews` (R0 L1): added `PreviewError::EmptyOutput { .. } => EXIT_INVALID`
  arm so the new error maps to exit 4, not the default `_ => EXIT_USAGE` (2).

**Mandatory I1 existing-test migration (part of A2, keeps the suite green):**
- `preview.rs::write_fake_sidecar`: `cat > "$out"` (raw echo) → parse `--format`, drain
  stdin to `/dev/null`, write `<svg/>` (svg) or `printf '\211PNG\r\n\032\n'` (png = the
  exact 8-byte PNG magic).
- `preview.rs::render_plate_writes_file_and_returns_path`: dropped the
  `body == raw_string` assertion (inherently incompatible with signature validation);
  now asserts path returned + file written + non-empty + `starts_with("<svg")`.
- `preview.rs::render_plate_png_uses_png_extension`: unchanged body (extension-only);
  now also clears validation because the migrated fake writes PNG magic.
- `cli.rs::write_fake`: same format-aware migration so the `--png` run
  (`png_flag_renders_png`) writes PNG magic instead of `<svg/>` → clears validation.

**Independent-sweep confirmation:** the real cross-lang test
`preview_cross_lang.rs::real_sidecar_renders_public_plates_only` stays green (real SVG
begins `<svg` at byte 0); `matched_version_renders_and_sets_preview_exit_0` /
`no_preview_flag_*` stay green (migrated fake still writes `<svg/>` for svg). Exactly the
3 R0-named tests were migrated; no 4th broke.

**Final counts after A2:** Go `go test ./...` ok. Rust `ME_REQUIRE_GO=1 cargo test --locked`
= 90 passed (lib 56 + main 1 + cli 28 + cross_lang 1 + golden 3 + preview_cross_lang 1),
0 skips. `cargo clippy --all-targets -- -D warnings` clean.

---

## Final verification (whole diff)

- `ME_REQUIRE_GO=1 cargo test --locked` (worktree root) → **90 passed, 0 failed, 0 skips**
  (lib 56, main 1, cli 28, cross_lang 1, golden 3, preview_cross_lang 1). Baseline was 82;
  +8 net new (A3 +3 Rust, A1 +2 Rust, A2 +3 Rust; 3 existing tests migrated in place).
- `go test -count=1 ./...` in `preview/` → ok (includes the new `TestWriteOutPermIsOwnerOnly`).
- `cargo clippy --all-targets -- -D warnings` → clean.

**Manual e2e** (real `me` + real version-matched `me-preview 0.3.0` built with
`-ldflags -X main.version=0.3.0`, on `md1yqpqqxqq8xtwhw4xwn4qh` + MK1_A + MK1_B):
1. Fresh-dir `me bundle --preview <dir> --manifest <dir>/manifest.json` → **exit 0**;
   `plate-1/2/3.svg` + `manifest.json` all written at **mode 600** (`-rw-------`).
2. Re-run into the now-dirty dir → **exit 2**, refusal:
   `me: preview directory <dir> already contains plate artifacts (e.g. plate-1.svg);
   use an empty/clean directory`. No plate overwritten.
3. `me --out wallet.ndef` (convert path) → exit 0, NDEF file at **mode 600**.

## Notes for the post-impl reviewer

- **A2 signature gate is format-specific** (keyed on `render_plate`'s `png` bool), per R0 N2.
  A leading UTF-8 BOM before `<?xml`/`<svg` would false-reject, but the pinned sidecar emits
  none (`render_svg.go` writes `<svg` at byte 0); documented in a code comment. Not a defect
  for the pinned submodule.
- **A3 mode binds on CREATE only** (R0 L3): overwriting a pre-existing world-readable
  NDEF/manifest keeps its old mode. Accepted residual, documented on `write_private`.
- `write_private` uses `OpenOptions .write.create.truncate` + `#[cfg(unix)] .mode(0o600)`;
  non-unix compiles the same options without `.mode` (cfg-guarded no-op — R0 OQ4). Only
  Unix was exercised here.
- The I2 overwrite-shrink guard passes today with `fs::write` too (that already truncates);
  its teeth were verified out-of-band by removing `.truncate(true)` (test then fails at the
  byte-identity assertion) and restoring.
- Go `writeOut` perm test asserts on `writeOut` directly (not the shell-fake Rust e2e),
  per R0 L2.
