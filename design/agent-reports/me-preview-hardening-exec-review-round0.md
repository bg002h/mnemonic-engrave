# Post-implementation adversarial execution review — me preview hardening (Cycle A: F8+F9+F10), round 0

Reviewer: opus post-implementation adversarial execution reviewer (MANDATORY gate, non-deferrable).
Worktree `/scratch/code/shibboleth/me-cycleA`, branch `me-preview-hardening`, base master `9fafb6b`
(spec commit `5d68002`). Diff under review: `master..me-preview-hardening` (5 commits: spec + A3 + A1 +
A2 + impl-log). Standard: GREEN = 0 Critical / 0 Important before the controller opens the PR.
Scope of mandate: catch implementation-introduced regressions TDD misses (the R0 gate already cleared
plan correctness). Environment: Go 1.26.4 (`/home/bcg/.local/go/bin`), cargo nightly 1.97 on PATH.

Read in full first: `design/SPEC_me_preview_hardening.md` (GREEN R0 round 1), both R0 review rounds,
the impl log, and the whole source diff. Probes were run in an isolated **detached probe worktree**
(`/scratch/code/shibboleth/me-review-scratch/probe`, since removed) and a scratch target/e2e area
(all cleaned up); the review worktree was never modified except this report file.

---

## Verdict up front

**GREEN (0 Critical / 0 Important).** 0C / 0I / 1 Low / 2 Nit. All Lows/Nits are pre-adjudicated
residuals or cosmetic; none block the PR. Full suite reproduced at the claimed 90/0/0 + Go ok +
clippy clean. Every safety-critical guard was independently proven to have teeth by perturbation.

---

## 1. Funds-safety / behavior regressions from the diff

**A1 false-refuse on a legitimate empty/first-run dir? NO.** `wire_previews` scans via
`std::fs::read_dir(dir)` (main.rs:275) only AFTER the `dir.is_dir()` gate (262). Rust `read_dir`
never yields `.`/`..`, so an empty dir yields zero entries → no `is_plate_artifact` hit → no refusal.
Independently confirmed by e2e: a fresh dir rendered all 3 plates + manifest at **exit 0**. The
refusal only fires when a name actually matches `plate-` prefix AND `.svg`/`.png` suffix. A `read_dir`
*error* also refuses (main.rs:290) — but that only reaches when the dir is a non-readable dir (e.g.
mode 0o300); an ordinary empty readable dir reads cleanly. Fail-closed, no false-refuse of the
first-run case.

**Can a preview be recorded for a plate whose file failed validation? NO — ordering is correct.**
`render_plate` calls `validate_render_output(&out_path, png)?` (preview.rs:181) BEFORE its final
`Ok(out_path…)` (183). In `wire_previews`, `plate.preview = Some(path)` is set only on the `Ok` arm
(main.rs:310); the `Err` arm (312) returns an exit code without touching `plate.preview`. Verified the
caller too: `run_bundle_cli` does `if let Some(code) = wire_previews(…) { return code; }` (main.rs:192-194)
**before** `serde_json::to_string_pretty` (197) and the manifest write (204) — so an EmptyOutput on any
plate aborts the whole bundle emit: no manifest written, no partial/orphan preview path recorded. (A
plate-1.svg already on disk when plate-2 fails is left as an owner-only orphan; the next run is refused
by A1, and no manifest ever references it. Fail-closed, no funds gap.)

**Does `write_private` truncate match `fs::write` at BOTH sites? YES — verified with teeth.**
`write_private` (main.rs:346) = `OpenOptions::new().write(true).create(true).truncate(true)` +
`#[cfg(unix)] .mode(0o600)`, then `write_all`. It replaces `std::fs::write` at the NDEF `--out` site
(main.rs:140) and the manifest site (main.rs:205) — both confirmed by the diff. **Overwrite-shrink
re-performed manually on the real binary:** a 1031-byte manifest (md1+2×mk1) overwritten by a 351-byte
manifest (md1+ms1 reminder) → file ends `…5d 0a 7d` (`]\n}`), parses as valid JSON (`wallet_plates=2`),
no trailing bytes. **Teeth proven independently:** in the probe worktree I removed `.truncate(true)`;
`perms::manifest_overwrite_shrink_no_trailing_bytes` then FAILED at the byte-identity assertion
(cli.rs:787). Restored → green. The I2 guard is genuine, not vacuous.

**Does 0o600 break any consumer? NO.** The NDEF/manifest/SVG artifacts are consumed by the same user
that created them (the file owner reads them; NFC push to the SeedHammer device is a content transfer,
unaffected by mode bits). Confirmed owner can re-read a 0o600 plate (`open().read(5)` → `<svg`). On a
multi-user host, other users lose read access — which is exactly the F10 (D5-2) intent. No test or CI
consumer reads these across users. No breakage.

## 2. A2 validation correctness

**Empty-file and garbage-file probes re-performed → EmptyOutput → exit 4, no preview recorded.** The
suite's `render_plate_rejects_empty_output` / `render_plate_rejects_garbage_output` (preview.rs) and
`empty_sidecar_output_exit_4` (cli.rs) pass. I proved they have teeth: in the probe worktree I neutered
`validate_render_output` to `if out_path.exists() { return Ok(()); }`; all three then FAILED
(the two lib tests panicked expecting EmptyOutput; the CLI test got `code=0` — the exact F9 bug — instead
of 4). So the exit-4 / no-preview outcome is genuinely enforced by the A2 gate, not incidental. Bounded
prefix read confirmed: `validate_render_output` reads at most a fixed `[u8; 512]` via `File::read`
(preview.rs), never the whole file — real plates are 150-488 KB and their `<svg`/PNG signature sits at
byte 0, well inside 512 B.

**Real sidecar output still passes.** Built the real `me-preview 0.3.0` (`-ldflags -X main.version=0.3.0`)
and ran `me bundle --preview` end-to-end: exit 0, all 3 plates rendered, `plate-1.svg` begins
`3c 73 76 67 20 78` = `<svg x` at byte 0 → clears the SVG gate. The `ME_REQUIRE_GO=1` suite's
`preview_cross_lang::real_sidecar_renders_public_plates_only` also passes.

**BOM false-reject flag (implementer-raised): truly impossible with the pinned sidecar.** `render_svg.go`
writes `<svg xmlns=…` as the very first bytes of its output builder (`render_svg.go:40`); nothing is
written to `b` before that line (the path-data builder `d` is separate and only interpolated into the
`<path …>` element). No `xml.Header`, no BOM, no leading whitespace anywhere in the Go render/writeout
path (grep confirms). PNG goes through `png.Encode` → 8-byte magic. So the only theoretical false-reject
(a leading UTF-8 BOM pushing `<?xml`/`<svg` off offset 0) cannot arise here. It is a documented
forward-looking comment in `validate_render_output`, not a live defect. Not a finding.

## 3. Test integrity (I1 migration)

The three R0-named tests were migrated; I re-verified two can still go red (teeth), plus a third guard:

- **`png_flag_renders_png` (I1 #3):** perturbed `cli.rs::write_fake` in the probe to write `<svg/>`
  regardless of `--format`. The test then FAILED with `code=4` at cli.rs:530 — i.e. the PNG signature
  gate correctly rejects an SVG body written to a `.png` `--out`, and the test detects it. This proves
  the `--png` path is **not** vacuously passing: validation is genuinely exercised for PNG.
- **`render_plate_rejects_empty_output` / `_garbage_output` (A2 negatives):** both went red under the
  neutered gate (above). Genuine reds-first.
- No 4th test breaks. Full suite is 90/0/0 with the migrated fakes; the real cross-lang and the
  svg-only `matched_version_…` tests stay green because the migrated fake still writes `<svg/>` for svg.

No test passes vacuously: the `--png` test writes real PNG magic through the migrated fake and its body
must clear the PNG-magic check (proven by the red above).

## 4. Scope

`git diff --name-only master..` = exactly `crates/me-cli/src/main.rs`, `…/src/preview.rs`,
`…/tests/cli.rs`, `preview/main.go`, `preview/writeout_test.go` + the 4 design docs (spec, impl-log,
2 R0 reviews). **`git diff master.. -- crates/me-cli/tests/golden.rs` is EMPTY.** No NDEF-byte change
(A3 changes file MODE only; `&bytes` is written unchanged — cross_lang + golden tests pass, proving
NDEF/golden bytes unchanged). No manifest-schema change. No success-path exit-code change (A1/A2 only
convert previously-wrong outcomes — dirty-dir render, 0-byte/garbage "success" — into refusals). No
bleed into F11 (`locate_sidecar`/version gate untouched), F13 (stroke width), or F15 (render goldens).

## 5. Full suite reproduction (actual numbers)

- `env PATH=… ME_REQUIRE_GO=1 cargo test --locked` (scratch CARGO_TARGET_DIR): **90 passed / 0 failed /
  0 ignored** — lib 56, main 1, cli 28, cross_lang 1, golden 3, preview_cross_lang 1. Matches the impl
  log's claim exactly.
- `go test -count=1 ./...` in `preview/`: **ok** (incl. `TestWriteOutPermIsOwnerOnly` — verified PASS via
  `-run`). `go vet ./...`: clean.
- `cargo clippy --all-targets -- -D warnings`: clean.

---

## Findings

### Low

**L1 (informational, pre-adjudicated) — 0o600 binds on CREATE; overwriting a pre-existing
world-readable NDEF/manifest keeps its old mode.** `main.rs:346 write_private`. If `wallet.ndef` /
`m.json` already exists at 0o644 (from an older tool or `touch`), `me --out`/`--manifest` overwrites the
content but not the mode → the new secret-bearing content is world-readable. This is exactly R0 L3,
adjudicated GREEN as an accepted residual and documented on `write_private` ("0o600 binds on CREATE …
NDEF/manifest targets are user-named; preview targets are forced-fresh by the dirty-dir refusal"). Not
an implementation-introduced regression — flagged only for completeness. No change required. (If ever
full coverage is wanted, a post-open `set_permissions(0o600)` closes it; out of scope for this cycle.)

### Nit

**N1 — A1 dir scan silently skips non-UTF-8 filenames.** `main.rs:278` uses
`entry.file_name().to_str()`; a non-UTF-8 name yields `None` and is skipped (not refused). This cannot
miss a real collision: the tool only ever writes pure-ASCII `plate-{idx}.{svg,png}` names (valid UTF-8),
so any non-UTF-8 file is by definition not a tool-written plate and cannot cause manifest plate-mixing.
Theoretical only; no funds impact. Left as-is is fine.

**N2 — `empty_sidecar_output_exit_4` (cli.rs) is not `#[cfg(unix)]`-gated** while it relies on a
`#!/bin/sh` fake. This matches the pre-existing convention of the entire `mod preview` block (its
siblings `png_flag_renders_png`, `render_failure_exit_4`, etc. are likewise ungated shell fakes), so it
introduces no new portability regression relative to master. Consistent with the established pattern;
no action.

---

## Adversarial edge cases checked and cleared

- Empty/first-run dir → no A1 refuse (e2e exit 0). ✓
- Validation strictly before `plate.preview` set, and before manifest serialization/write. ✓
- Truncate present at both write sites; teeth proven by removal (shrink test red). ✓
- Overwrite-shrink on the real binary → valid JSON, clean tail, no trailing bytes. ✓
- A2 empty/garbage → exit 4 + no preview; teeth proven by neutering the gate (all 3 red). ✓
- `--png` not vacuous: svg-under-png → exit 4 (gate is format-specific and exercised). ✓
- Real sidecar SVG begins `<svg` at byte 0; no BOM/prolog anywhere in the Go path → no false-reject. ✓
- 0o600 on all real artifacts (plate-1/2/3.svg + manifest.json = `-rw-------`); owner re-reads fine. ✓
- Dirty-dir refusal on real binary: exit 2, stale file not clobbered (md5 identical), never deleted. ✓
- Scope: golden.rs empty diff; NDEF bytes / manifest schema / success exit codes unchanged; no
  F11/F13/F15 bleed. ✓
- Suite 90/0/0 + Go ok + go vet clean + clippy -D warnings clean. ✓

---

## Verdict

**GREEN (0 Critical / 0 Important).** 1 Low (L1: pre-adjudicated create-only-mode residual, documented),
2 Nit (N1 non-UTF-8 scan skip — no funds impact; N2 cfg-gating parity — matches existing convention).
The implementation faithfully realizes the GREEN R0 spec, every safety guard (A1 refuse, A2 gate, A3
truncate+0o600) was independently proven to have teeth, and no implementation-introduced regression was
found. Cleared for the controller to open the PR.
