# R0 architect review — SPEC_me_preview_hardening.md (round 1)

Reviewer: opus R0 architect (adversarial), ROUND 1. Worktree
`/scratch/code/shibboleth/me-cycleA`, branch `me-preview-hardening`, base master `9fafb6b`.
Target: `design/SPEC_me_preview_hardening.md` (Cycle A: F8 + F9 + F10).
Standard: GREEN = 0 Critical / 0 Important before implementation.

Round 0 verdict: NOT GREEN (0C / 2I / 4L / 1N). All 7 findings were folded into the spec;
this round verifies each fold closes its finding and re-reviews fresh for fold-induced drift.

Context re-read in full this round: `design/agent-reports/me-preview-hardening-spec-R0-round0.md`;
the pre-implementation code the spec changes (`crates/me-cli/src/main.rs` `wire_previews`
227-296 + `fs::write` 140/205; `crates/me-cli/src/preview.rs` `render_plate` 119-172 +
`#[cfg(test)]` fakes; `crates/me-cli/tests/cli.rs` incl. `mod preview` fakes/tests;
`preview/main.go` `writeOut` 123-129); and — for the fresh "other broken test" sweep —
`crates/me-cli/tests/preview_cross_lang.rs`, `preview/version_test.go`, `preview/render_test.go`,
and `crates/me-cli/src/bundle.rs` (admission rules for the overwrite-shrink pair).

---

## Fold verification — do the round-0 findings close?

### I1 (was Important) — A2 breaks 3 existing tests; migration must be listed and correct. **CLOSED.**

The spec now carries an explicit "Existing-test migration (R0 I1 — MANDATORY, these 3 tests
break otherwise)" subsection (§A2, lines 75-89) that:

- Migrates `preview.rs::write_fake_sidecar` from `cat > "$out"` (raw echo) to a format-aware
  signature stub: parse `--format`; svg → `printf '<svg/>'`; png → `printf '\211PNG\r\n\032\n'`;
  keep `cat > /dev/null` to drain stdin. **Byte-checked:** `\211 P N G \r \n \032 \n` =
  `89 50 4E 47 0D 0A 1A 0A`, exactly the PNG magic in §A2 line 63 and exactly what
  `preview/render_test.go:46` asserts the real encoder emits (`\x89PNG\r\n\x1a\n`). Correct.
- Rewrites `render_plate_writes_file_and_returns_path` to drop the `body == raw_string`
  assertion (the one *inherently* incompatible with signature validation) and assert
  path-returned + file-written + starts-with-`<svg`. Correct: the migrated fake writes `<svg/>`,
  which both passes render_plate's new SVG signature gate and satisfies the rewritten assertion.
- Re-characterises `render_plate_png_uses_png_extension`: `md1x` on stdin is now drained; the
  fake writes PNG magic; the extension assertion is unchanged and now also clears validation.
  Correct.
- Migrates `cli.rs::write_fake` the same way (branch on `--format`) so `png_flag_renders_png`'s
  `--png` run writes PNG magic instead of `<svg/>`. Correct.

**Independent "other broken test" sweep (the adversarial half of I1).** I enumerated every test
that reaches a *successful* render via a fake or the real sidecar, and every body/mode assertion:

- `preview.rs` users of `write_fake_sidecar`: only `render_plate_writes_file_and_returns_path`
  and `render_plate_png_uses_png_extension` render; both are listed. `has_sidecar_name_*`,
  `sidecar_version_parses_prefix`, `sidecar_version_empty_when_unset` never render;
  `render_plate_propagates_nonzero_exit` uses its own inline `exit 1` fake (unaffected — it
  fails before any write). Only `render_plate_writes_file_and_returns_path` ever reads a fake's
  output body (`fs::read_to_string`), and it is the one rewritten. No hidden raw-echo dependency.
- `cli.rs::mod preview` users of `write_fake`: `matched_version_renders_and_sets_preview_exit_0`
  (svg → migrated fake still writes `<svg/>` → stays green), `png_flag_renders_png` (listed),
  `mismatched_version_exit_2` / `absent_sidecar_*` / `no_preview_flag_*` / `unwritable_preview_dir_exit_2`
  never reach a successful write (`unwritable_preview_dir_exit_2` in particular now exits at the
  `dir.is_dir()` gate — `missing` is not a dir — *before* the A1 scan or any render, so it is
  independent of the fake's body). No `--png` test reuses an svg-only fake after the fold.
- **Real sidecar — `preview_cross_lang.rs::real_sidecar_renders_public_plates_only`:** reads each
  preview and asserts `<svg`/`<path` present. The real SVG begins `<svg xmlns=…` at offset 0
  (`render_svg.go:40`), so A2's signature gate passes; A3's mode change does not touch content;
  the outdir is a fresh `unique_dir` so A1 finds nothing. **Stays green.** Not omitted by the spec
  because it does not break.

Conclusion: exactly the 3 tests round-0 named break, all 3 are listed with correct migrations,
and no 4th test breaks. I1 fully closed.

### I2 (was Important) — `write_private` must specify truncate; acceptance must catch trailing bytes. **CLOSED.**

§A3 (lines 104-110) now specifies `OpenOptions::new().write(true).create(true).truncate(true)`
and, under `#[cfg(unix)]`, `.mode(0o600)`, **at both sites** — main.rs:140 (NDEF `--out`)
explicitly and main.rs:205 (manifest) via "same helper". `truncate(true)` is flagged load-bearing
with the exact failure mode (shrinking overwrite leaving trailing stale bytes → invalid JSON).
The overwrite-shrink acceptance (lines 124-126) — "write a large manifest, then a smaller one to
the same path; assert the file parses as valid JSON (no trailing bytes) and length equals the
smaller payload" — genuinely catches the bug: without `truncate(true)` the overwritten file is
`{small-json}{tail-of-large-json}`, which fails BOTH the JSON-parse and the exact-length
assertion. The test is feasible on the manifest path: `bundle.rs:390` confirms a lone unchunked
md1 (`MD1_UNCHUNKED == MD1_VALID`) is a valid bundle, so `md1 + mk1_a + mk1_b` (4 plates) as the
large write and `md1` alone (md1 + synthesized ms1 reminder = 2 plates) as the smaller write is a
valid, size-distinct pair. (The NDEF `--out` path — two md1 vectors of different length — is an
even simpler fallback if wanted.) I2 fully closed.

### L1 — `EmptyOutput => EXIT_INVALID` match-arm edit. **CLOSED.**
§A2 lines 70-73 state it verbatim ("Add an arm `PreviewError::EmptyOutput { .. } => EXIT_INVALID`
— the default `_` would otherwise map it to EXIT_USAGE (2)"). Matches the real `_`-defaulting arm
at main.rs:288-291. Genuinely addressed, not paraphrased.

### L2 — Go writeOut perm test must be a Go test, not the shell-fake e2e. **CLOSED.**
§A3 lines 127-129 require "a **Go unit test** on `writeOut` (or the go-gated real cross-lang)"
and explicitly reject the shell-fake Rust e2e with the correct reason (the fake is not Go
`writeOut`). Confirmed there is no existing Go test asserting the old `0o644`, so flipping to
`0o600` breaks nothing and the perm assertion is a clean new addition (candidate homes:
`version_test.go::TestRenderSVGToFile`/`TestRenderPNGToFile`, which already round-trip real files
through `writeOut`). Genuinely addressed.

### L3 — 0o600 binds on CREATE only. **CLOSED.**
§A3 lines 116-119 note it honestly ("0o600 binds on CREATE; an overwrite of a pre-existing file
keeps its old mode … acceptable residual: NDEF/manifest targets are user-named and preview targets
are forced-fresh by A1"). Correct and consistent with A1's fresh-create guarantee. Genuinely
addressed.

### L4 — A1 test placement (integration in cli.rs, version-matched fake) + helper unit test. **CLOSED.**
§A1 "Acceptance (R0 L4 — test placement)" (lines 45-53) puts the refusal test in cli.rs with a
version-matched fake and a pre-seeded `plate-2.svg`, keeps the empty-dir success case, and adds a
`is_plate_artifact` unit test over the near-miss set. Genuinely addressed.

### Nit — bounded prefix read. **CLOSED.**
§A2 lines 67-68 specify "a **bounded prefix** (first 512 bytes) via `File::read` into a fixed
buffer — never the whole file". Genuinely addressed.

All 7 round-0 findings close.

---

## Fresh adversarial pass (fold-induced drift)

1. **Migrated fake breaking a *different* test that relied on raw-echo.** None. The only consumer
   that ever asserted `write_fake_sidecar`'s output *body* is the rewritten
   `render_plate_writes_file_and_returns_path`; `cli.rs::write_fake` already wrote `<svg/>`
   (never raw-echo) pre-fold, so no cli.rs test depended on echoed stdin. Verified above.

2. **`is_plate_artifact` edge `plate-.svg` accepted — contradiction?** None. The real sidecar
   only ever writes `plate-{idx>=1}.{ext}`, never `plate-.svg`; so `plate-.svg` can only be a
   foreign file, and accepting it (→ refuse the dir) is strictly fail-closed. There is no
   within-run self-collision: A1 scans once before the loop, and the loop's own writes are not
   re-scanned. Consistent with A1's whole-dir refusal (which already refuses on the tool's own
   prior `plate-1.svg`). No contradiction.

3. **Bounded 512-byte read vs a valid SVG behind a long XML prolog / BOM.** Non-issue for the
   pinned sidecar. `render_svg.go:40` emits `<svg` at byte 0 (no prolog, no comment, no BOM), well
   inside 512 B; and the gate additionally accepts a leading `<?xml`, so even a future prolog up to
   ~500 B would pass. The only theoretical false-reject is a leading UTF-8 BOM (`EF BB BF` is not
   ASCII whitespace, so it would not be trimmed and would push `<?xml`/`<svg` off offset 0) — but
   this sidecar emits no BOM, so it cannot arise here. Worth a one-line forward-looking comment in
   the impl, not a spec change. No false-reject of real output; OQ3 (signature-only) stays sound.

4. **A1 × A2 × A3 composition on the multi-plate render tests.** `matched_version_…` and the real
   cross-lang test render 3 public plates into one fresh dir: A1 scans the empty dir once (pass),
   the loop writes `plate-1/2/3` with unique numbers (no collision), A2 validates each `<svg/>`
   (pass), A3 creates each at 0o600 fresh (owner reads its own file — the readback assertions
   still pass). Clean.

5. **New `PreviewError::EmptyOutput` variant vs exhaustive matches.** `impl Display for
   PreviewError` (preview.rs:26) is an exhaustive `match self`, so the compiler *forces* a Display
   arm for the new variant — it cannot be silently missed. The `wire_previews` exit-mapping match
   has a `_` (handled by L1). `IsTxtBusy for PreviewError` uses `matches!` (non-exhaustive; new
   variant → `false`, correct). So the only additional edit is compiler-enforced. No blocker.

6. **Ordering (A3 → A1 → A2) vs the collateral migrations.** The 3 I1 tests go red only when A2's
   validation lands, and the spec's migrations are part of A2 — so they are updated in the same
   step, never left red across a commit boundary. New negative tests (0-byte / garbage → EmptyOutput
   → Some(4)) are genuine reds-first. A clean single-implementer TDD sequence exists.

7. **No existing test regresses from A3.** Grep across `crates/me-cli/tests/**` and `preview/**`
   found zero mode/permission assertions on written artifacts (only the fakes' own `set_mode(0o755)`
   on the script, and content-only readbacks). Truncate affects only shrinking overwrites (no
   existing test overwrites), and mode never affects content. A2 is the sole source of existing-test
   breakage, and that is fully migrated (I1).

## Nits (non-blocking; do not gate GREEN)

- **N1 — stale test-name shorthand.** §A2 line 89 refers to "the SVG e2e `bundle_preview_*` tests";
  no test is literally named `bundle_preview_*`. The intended tests are
  `cli.rs::preview::matched_version_renders_and_sets_preview_exit_0` (and the real
  `preview_cross_lang.rs::real_sidecar_renders_public_plates_only`). Cosmetic; the intent is
  unambiguous. Fix at implementation time if convenient.
- **N2 — signature check is format-specific.** §A2 lists the SVG and PNG signatures separately;
  `render_plate` already carries `png: bool`, so the natural (and intended) reading is "check the
  signature for the requested format". Both that reading and a looser "either signature" reading
  pass all acceptance tests, so this is not a defect — worth one clarifying clause if edited.
- **N3 — A2 negative-path test layer.** The acceptance bullet folds `render_plate` errs (a
  preview.rs unit test) and `wire_previews` returns Some(4) (only observable via the CLI exit code
  in cli.rs) into one line. The two-layer split is the same one L4 already established for A1; an
  implementer will write both. Terse, not wrong.
- **N4 — Display/`reason` wording.** The spec need not enumerate `EmptyOutput`'s `reason` strings;
  a reminder that the new variant needs a Display arm (compiler-forced anyway) and that a missing
  `--out` file (metadata `Err`) is also `EmptyOutput` would make §A2 airtight. Optional.

## Soundness of the 4 OQ defaults (re-confirmed)

Unchanged from round 0 and still sound: (1) refuse-not-delete, (2) validate-in-`render_plate`
(+ L1 arm), (3) signature-not-parse (verified no false-reject of real `<svg …>` / PNG), (4)
Windows cfg no-op. No fold disturbed any of these.

---

## Verdict

**GREEN (0C / 0I).**

- I1 — CLOSED: all 3 breaking tests listed with correct migrations; PNG-magic stub byte-verified;
  independent sweep finds no 4th breakage (cross-lang + all Go tests stay green).
- I2 — CLOSED: `truncate(true)` specified at both write sites; overwrite-shrink acceptance
  genuinely catches trailing bytes; valid large/small manifest pair confirmed feasible.
- L1 — CLOSED: explicit `EmptyOutput => EXIT_INVALID` match-arm edit.
- L2 — CLOSED: Go `writeOut` unit test required; shell-fake avenue removed; no existing 0o644
  assertion to migrate.
- L3 — CLOSED: create-only mode residual noted honestly.
- L4 — CLOSED: A1 refusal test in cli.rs (version-matched fake) + `is_plate_artifact` unit test.
- Nit — CLOSED: bounded 512-byte prefix read specified.
- N1–N4 — cosmetic Nits only; none block a clean single-implementer TDD pass.

Cleared for the single-implementer TDD phase.
