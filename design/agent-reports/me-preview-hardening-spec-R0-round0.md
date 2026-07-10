# R0 architect review — SPEC_me_preview_hardening.md (round 0)

Reviewer: opus R0 architect (adversarial). Worktree `/scratch/code/shibboleth/me-cycleA`,
branch `me-preview-hardening`, base master `9fafb6b`.
Target: `design/SPEC_me_preview_hardening.md` (Cycle A: F8 + F9 + F10).
Standard: GREEN = 0 Critical / 0 Important before implementation.

Context read in full: `design/FOLLOWUPS.md` (the two cited slugs),
`design/agent-reports/funds-audit-D3-bundle-round0.md` (D3-1/D3-2),
`funds-audit-D5-hygiene-round0.md` (D5-2/D5-3); and the code the spec changes:
`crates/me-cli/src/main.rs` (`wire_previews` 227-296, `fs::write` 140 & 205),
`crates/me-cli/src/preview.rs` (`render_plate` 119-172 + the `#[cfg(test)]` fake scaffolding),
`preview/main.go` (`writeOut` 123-129), plus the renderers (`render_svg.go`, `render_png.go`)
and every test that exercises `--preview` (`tests/cli.rs`, `tests/preview_cross_lang.rs`).

---

## Recon verification (citations vs current source)

All spec citations are accurate against `9fafb6b` — no citation decay:
- `wire_previews` checks only `!dir.is_dir()` at `main.rs:262`, before the render loop (271-294). ✓
- `render_plate` writes `dir/plate-{idx}.{ext}` at `preview.rs:128`; treats `out.status.success()`
  as the sole success criterion at `preview.rs:163-171` (no stat/re-read of `--out`). ✓
- Bare `std::fs::write` for NDEF `--out` at `main.rs:140` and manifest at `main.rs:205`. ✓
- Go `writeOut` uses `os.WriteFile(path, payload, 0o644)` at `preview/main.go:128`. ✓

Each finding is real against current code; each A-item targets the right site.

## Correctness — does each A-item close its finding?

**A1 (F8) closes the mixing scenario. SOUND.** The scan/refuse is placed after `dir.is_dir()`
(262) and before the render loop (271), i.e. immediately before the only code path that writes
`plate-N` files. Because the refusal returns `Some(EXIT_USAGE)` before `render_plate` is ever
called, no plate is written when a foreign `plate-*` exists → the D3-1/D5-3 trailing-leftover
mixing cannot occur. Placement after the sidecar-locate/version gates is defensible: mixing is
only introduced by an actual render, which only happens once those gates pass, so guarding the
render path is sufficient (an absent/mismatched sidecar writes nothing this run).

**A2 (F9) — validate-in-`render_plate` is sufficient; no path records a bad preview. SOUND, and
verified compatible with the REAL sidecar.** On `EmptyOutput`, `render_plate` returns `Err`;
`wire_previews` takes the `Err(e)` arm, never sets `plate.preview`, and returns from the function
before manifest serialization — so (a) no preview path is recorded for the failed plate and (b)
the whole bundle emit is aborted (no partial manifest), matching the existing `Render`-failure
behavior. I confirmed the signature check does **not** false-reject production output:
`render_svg.go:40` emits `<svg xmlns="http://www.w3.org/2000/svg" viewBox=…` as the first bytes
(begins with `<svg`, no leading whitespace, no comment/prolog), and `render_png.go:86` uses
`png.Encode`, which always writes the 8-byte PNG magic. So the real `me → me-preview` e2e
(`preview_cross_lang.rs`, which renders only SVG) still passes. Good — this de-risks OQ3.

**A3 (F10) — OpenOptions(0o600) works at both `main.rs` sites and the manifest path flows through
it. SOUND with two caveats (see I2, L3).** The manifest is written via `fs::write` at `main.rs:205`
only on the `--manifest <file>` branch (else stdout `println!` at 211 — nothing to protect), so
replacing 205 covers it; NDEF `--out` at 140 likewise. `.mode(0o600)` (from `OpenOptionsExt`,
`#[cfg(unix)]`) applies at file-CREATE time. Because A1 guarantees the preview dir holds no
`plate-*` when rendering starts, every `plate-N.{svg,png}` write is a fresh create → 0o600 always
applies on that side (A1+A3 compose cleanly). Caveats below concern the manifest/NDEF overwrite
case and the truncate semantics.

## Findings

### Important

**I1 — A2's signature validation silently breaks 3 EXISTING passing tests; the spec neither
lists them nor plans their migration, and one of them is fundamentally incompatible with the new
behavior.** Adding the SVG/PNG signature gate to `render_plate` turns these current greens red:

1. `preview.rs::render_plate_writes_file_and_returns_path` — its fake `write_fake_sidecar` does
   `cat > "$out"`, writing the raw plate string `md1yqpqqxqq8xtwhw4xwn4qh`; the test then asserts
   `body == "md1yqpqqxqq8xtwhw4xwn4qh"`. Under A2, that body has no `<svg`/`<?xml`/PNG-magic
   signature → `render_plate` returns `EmptyOutput` → the test's `.unwrap()` panics. The body
   assertion is *inherently* incompatible with signature validation — it must be rewritten, not
   just tweaked.
2. `preview.rs::render_plate_png_uses_png_extension` — same `cat > "$out"` fake, `png=true`, pipes
   `md1x`; `md1x` is not PNG magic → `EmptyOutput` → `.unwrap()` panics.
3. `cli.rs::preview::png_flag_renders_png` — uses `write_fake`, which writes `printf '<svg/>'`
   *regardless of format*, then runs with `--png`. `<svg/>` written to `plate-1.png` fails the PNG
   magic check → exit 4, but the test asserts `.success()`.

(`cli.rs::matched_version_renders_and_sets_preview_exit_0` and `no_preview_flag_is_byte_for_byte_phase_a`
survive because `write_fake` emits `<svg/>` for the SVG path; `render_failure_exit_4`,
`mismatched_version_exit_2`, `absent_sidecar_*`, `unwritable_preview_dir_exit_2` never reach a
successful write. So exactly the 3 above break — no more, no fewer.)

The spec's A2 acceptance ("Fake writing a real `<svg …>` → Ok", "PNG happy path: 8-byte PNG magic
+ IHDR stub → Ok") implies new format-valid fakes, but nowhere states that the shared
`write_fake_sidecar` (preview.rs) and `write_fake` (cli.rs) helpers must be made format-aware, nor
that the three tests above must be migrated. A TDD implementer will hit unexplained reds; the
danger is they "fix" it by weakening the signature check (defeating F9) instead of updating the
fakes. **Fix:** the spec must (a) make both fake helpers emit format-appropriate valid content
(`<svg/>` for svg; PNG magic + IHDR stub for png — branch on `--format`/out extension), (b)
rewrite `render_plate_writes_file_and_returns_path`'s body assertion (drop `body == raw_string`;
assert path returned + file written + starts with `<svg`), and (c) list all 3 as tests-to-update
in the ordering/verification section.

**I2 — `write_private` is under-specified: it must preserve `std::fs::write`'s truncate semantics,
and the acceptance tests (fresh files) will not catch a missing truncate.** The spec says only
"replace `std::fs::write` with a helper `write_private(path, bytes)` using `OpenOptions` with
`.mode(0o600)`". `std::fs::write` is create+**truncate**+write_all. A naive `OpenOptions`
(`.write(true).create(true).mode(0o600)` without `.truncate(true)`) will, on a re-run that
overwrites a longer prior file with shorter content, leave trailing bytes — e.g.
`me bundle --manifest m.json` for a big wallet then a small one yields
`{short-json}{tail-of-old-json}` = invalid JSON. The A3 acceptance criteria all write FRESH files,
so they cannot detect this regression vs current behavior. **Fix:** specify
`OpenOptions::new().write(true).create(true).truncate(true).mode(0o600)` (unix) and add an
overwrite-shrink regression test (write long, then short, assert exact new contents / no trailing
bytes) for at least the manifest path. (Blast radius is limited — a truncated manifest is
detectably-invalid JSON and a device stops NDEF parse at the 0xFE terminator — but this is a real
correctness regression the spec should not leave to chance.)

### Low / Nit

**L1 — `wire_previews`' error→exit-code match arm must gain an `EmptyOutput` branch, or the new
error maps to exit 2, not the spec's 4.** The current arm is `PreviewError::Render { .. } =>
EXIT_INVALID, _ => EXIT_USAGE` (`main.rs:288-291`). A new `EmptyOutput` variant falls into `_` →
EXIT_USAGE (2), contradicting A2's "caller maps to EXIT_INVALID (4)". The acceptance criterion
"`wire_previews` returns Some(4)" will catch this, so it is not a defect per se — but the spec
prose should state the match-arm edit explicitly (`EmptyOutput { .. } => EXIT_INVALID`) so it is
not left to be discovered only via a failing test.

**L2 — A3's Go-perm acceptance offers a test avenue that cannot exercise the Go change.** "assert
… in a Go test on `writeOut`, **or** in the Rust hermetic e2e" — the Rust hermetic path uses a
`/bin/sh` fake sidecar (cli.rs `write_fake`), not the Go `writeOut`; its `> "$out"` redirection
creates a 0o644 file, so a `mode & 0o077 == 0` assertion there would FAIL (and would test the
shell, not Go). Only a Go unit test on `writeOut` or the go-gated real `preview_cross_lang.rs`
round-trip validates the Go 0o600 fix. Drop the "or Rust hermetic e2e" option; require a Go
`writeOut` unit test (or assert perms in the real cross-lang test, which is `ME_REQUIRE_GO`-gated).

**L3 — `.mode(0o600)` applies only on CREATE; overwriting a pre-existing world-readable
manifest/NDEF leaves the old mode.** On the preview side this is moot (A1 forces fresh plate
files), but `me --out existing.ndef` / `me bundle --manifest existing.json` over a 0o644 file keeps
0o644. D5-2 is about the tool *creating* world-readable files, so fresh-create 0o600 addresses the
finding; note this residual honestly (defense-in-depth beyond scope) or `set_permissions(0o600)`
after open if full coverage is wanted. Not a blocker.

**L4 — A1 test placement/scaffolding guidance is thin.** `wire_previews` is private to the `me`
binary, so the refusal-→exit-2 behavior must be an *integration* test (cli.rs `preview` mod) that
installs a version-MATCHED fake (so control reaches the scan past the locate+version gates) and
pre-seeds the dir with `plate-2.svg`. The spec says A2 "extends existing test scaffolding in
preview.rs", but A1's refusal test belongs in cli.rs. The pure `is_plate_artifact` helper can be
unit-tested directly (the `plateau.svg`/`plate.txt` near-miss cases). State both layers and where
each lives.

**Nit — A2 read size.** The spec says "read the first bytes" without a count. Real SVG has no
leading whitespace and begins `<svg`, so reading a small prefix (e.g. 64 B) then trimming leading
ASCII whitespace is robust; specify a bounded read rather than reading the whole (possibly ~150 KB)
file.

## Soundness of the 4 open-question defaults

1. **Refuse-not-delete (A1): SOUND — CONFIRM.** Fail-closed, never deletes a user file that
   happens to match `plate-*` — the safer choice, consistent with project posture. Accepted
   consequence: a legitimate re-render into the same dir now requires manual cleanup (UX friction,
   not a safety issue). Fine.
2. **Validate in `render_plate` (A2): SOUND — CONFIRM.** Colocates the guarantee with the write;
   verified no `wire_previews` path records a preview after `EmptyOutput` (see Correctness).
   Requires the L1 match-arm edit.
3. **Signature-not-parse (A2): SOUND — CONFIRM.** Catches the empty/garbage class the finding
   names, and I verified it does not false-reject real `<svg …>` / PNG output. A full XML parse is
   scope creep and could reject valid-but-unusual SVG. Accepting `<?xml` in addition to `<svg` is
   harmless breadth (real output uses `<svg`).
4. **Windows cfg no-op (A3): SOUND — CONFIRM.** `OpenOptionsExt::mode` is unix-only and the D5-2
   threat model (multi-user POSIX host, mode bits) is Unix-centric; Windows uses ACLs (different
   mechanism, out of scope). Go's `os.WriteFile` mode is already near-inert on Windows, consistent.

## Scope

No bleed. A1/A2/A3 do not touch `locate_sidecar`/version gate (F11), stroke rendering (F13), or any
render golden (F15). NDEF bytes unchanged (A3 changes file MODE only, `&bytes` untouched). Manifest
schema unchanged — the spec chose D3-1 option (a) "refuse", keeping `plate-N.{ext}` names and the
existing `preview` field, rather than option (c) "namespace filenames with chunk_set_id" which
would have altered manifest preview paths. Exit codes for legitimate SUCCESS paths (empty dir +
valid render) stay 0; A1/A2 only convert previously-wrong outcomes (dirty-dir render; 0-byte/garbage
"success") into refusals — the intended F8/F9 fixes, consistent with the non-goal.

## Test integrity (failing-first)

- A1 refusal: genuine red today (renders + exit 0 today; exit 2 after) — needs a version-matched
  fake to reach the scan (L4). ✓
- A1 `is_plate_artifact` near-miss cases: new pure helper → red by non-existence. ✓
- A2 0-byte / garbage: genuine reds today (today: Ok + recorded; after: `EmptyOutput` + Some(4)). ✓
  Happy-path `<svg>`/PNG cases are regression guards (correctly labeled as such). ✓
- A3 perm assertions: genuine reds today (files are 0o644 → `& 0o077 != 0`), EXCEPT the Go-side via
  the hermetic fake, which cannot go red-for-the-right-reason (L2).
- Blocking: the 3 tests in I1 go red as unplanned collateral of A2 and must be migrated.

---

## Verdict

**NOT GREEN (0C / 2I).**

- I1 (Important) — A2 breaks 3 existing tests (`render_plate_writes_file_and_returns_path`,
  `render_plate_png_uses_png_extension`, `png_flag_renders_png`); spec must plan the fake-helper
  and test migration, and rewrite the raw-string body assertion.
- I2 (Important) — `write_private` must specify `truncate(true)` to preserve `fs::write` semantics;
  add an overwrite-shrink regression test (fresh-file acceptance tests miss it).
- L1 — spec must state the `wire_previews` `EmptyOutput => EXIT_INVALID` match-arm edit.
- L2 — drop A3's "or Rust hermetic e2e" Go-perm avenue; require a Go `writeOut` unit test / real
  cross-lang.
- L3 — note 0o600 applies on CREATE only (manifest/NDEF overwrite keeps old mode).
- L4 — clarify A1 test placement (integration in cli.rs, version-matched fake) + helper unit test.
- Nit — specify a bounded prefix read for the A2 signature check.

Fold I1+I2 (and, cheaply, the Lows), persist the fold, and re-dispatch for round 1.
