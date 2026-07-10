# Impl log — me sidecar discovery integrity (Cycle D / F11)

Single-implementer TDD execution of `design/SPEC_me_sidecar_discovery.md`
(GREEN at R0 round 1). Branch `me-sidecar-discovery`, base merged master
`4908dbb` (Cycles A/B/C landed). One section per step, committed each step.

Design summary (folds R0 L1/R1-N5): `locate_in(exe_dir, explicit) -> Option<PathBuf>`
is PURE (no env, no `$PATH`); the `ME_PREVIEW_BIN` env read + the set-but-missing
fail-loud (`EXIT_USAGE` 2) live in the WRAPPER (`wire_previews`), before the version
gate; `locate_sidecar(explicit)` is a thin wrapper that supplies `current_exe().parent()`
and delegates to `locate_in`. Because "migrate the existing tests to `ME_PREVIEW_BIN`"
is a D1 task and those tests can only be green once the env *success* path is read, the
env read's success/None path is introduced in D1 (the migration's prerequisite); D2 then
upgrades the set-but-missing case from graceful-`None` to fail-loud `EXIT_USAGE(2)`.

## Baseline

`ME_REQUIRE_GO=1 cargo test --locked` (submodule `third_party/seedhammer` initialized):
**96 passed, 0 failed, 0 ignored** across 8 result groups (lib 56, doctests 1,
cli.rs 28, cross_lang 1, golden 3, preview_cross_lang 1, prop 6, plus one 0-test
group). Matches the spec's stated baseline.

## Step D1 — drop the `$PATH` fallback; pure `locate_in`; migrate the tests

**TDD reds first.**
1. `planted_path_sidecar_ignored` (cli.rs `mod preview`) — behavioral lock at the
   real entry point: a version-matched fake `me-preview` planted ONLY on `$PATH`
   (no co-located, `ME_PREVIEW_BIN` removed) → expect exit 0 + "preview skipped" +
   no preview keys + nothing written. Confirmed genuinely RED against current code
   (old `$PATH` arm found + ran the fake → "rendered plate 1/2/3", previews present).
2. `locate_in` unit matrix (preview.rs `mod tests`, 5 tests) — compile-red (function
   did not exist): (b) exe-adjacent hit; (c) exe-adjacent miss → None; (c) no exe_dir
   → None; (a) explicit wins over a present exe-adjacent; (d) explicit returned
   verbatim without a re-check.

**Implementation.**
- preview.rs: added pure `fn locate_in(exe_dir: Option<&Path>, explicit: Option<&Path>)
  -> Option<PathBuf>` (explicit precedence, then exe-adjacent `is_file()`; no env, no
  `$PATH`). Deleted the old `$PATH` scan arm. `locate_sidecar` is now
  `pub fn locate_sidecar(explicit: Option<&Path>) -> Option<PathBuf>`, a thin wrapper
  supplying `current_exe().parent()` and delegating to `locate_in`. Doc comment
  rewritten (security rationale enriched in D3).
- main.rs `wire_previews`: reads `ME_PREVIEW_BIN` at the top (before the version gate);
  `.filter(|v| !v.is_empty())` treats empty as unset; `.filter(|p| p.is_file())` passes
  it as `explicit` only when it exists. **D1 leaves a set-but-missing path falling
  through to graceful degrade (None); D2 upgrades that to fail-loud `EXIT_USAGE`.**

**MANDATORY test migration (spec I1).** Migrated EXACTLY the 7 `mod preview` tests that
inject a sidecar and expect discovery from `.env("PATH", &bindir)` to
`.env("ME_PREVIEW_BIN", bindir.join("me-preview"))`: `empty_sidecar_output_exit_4`,
`render_failure_exit_4`, `matched_version_renders_and_sets_preview_exit_0`,
`png_flag_renders_png`, `dirty_preview_dir_refused_exit_2`, `mismatched_version_exit_2`,
`unwritable_preview_dir_exit_2`; plus the +1 `preview_cross_lang.rs`
(`real_sidecar_renders_public_plates_only`, replacing the `$PATH`-prepend with
`ME_PREVIEW_BIN=bindir/me-preview`). **The 2 survivors were left un-migrated**:
`absent_sidecar_degrades…` (:~614, absence path — added `.env_remove("ME_PREVIEW_BIN")`
for hermeticity + refreshed its stale comment, per R1-N4; `$PATH` injection untouched)
and `no_preview_flag_is_byte_for_byte_phase_a` (:~660, never calls locate — fully
untouched). Refreshed the `mod preview` module doc (was "PATH-only discovery is
deterministic").

**Green.** `ME_REQUIRE_GO=1 cargo test --locked` → **102 passed, 0 failed, 0 ignored**
(96 baseline + 5 `locate_in` unit + 1 `planted_path` integration; migration is net-zero
on count). `cargo clippy --all-targets -- -D warnings` → clean.
