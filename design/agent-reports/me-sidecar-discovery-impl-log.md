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

## Step D2 — `ME_PREVIEW_BIN` set-but-missing = fail-loud `EXIT_USAGE(2)`

**TDD red first.** `set_but_missing_me_preview_bin_exit_2` (cli.rs `mod preview`) — set
`ME_PREVIEW_BIN` to a non-existent path → expect exit 2 + a distinct message naming both
`ME_PREVIEW_BIN` and the path, and NO "preview skipped" note. Confirmed RED against D1
(D1 degraded gracefully: "preview skipped", exit 0).

**Implementation.** In `wire_previews`, upgraded the D1 env-read block: after reading
`ME_PREVIEW_BIN` (empty → unset), if the value is set and `!is_file()`, print
`me: ME_PREVIEW_BIN=<path> does not point to an existing file (…)` and `return
Some(EXIT_USAGE)` — strictly BEFORE the version gate, the `dir.is_dir()` check, the
Cycle-A dirty-dir scan, and the render loop (R1-N5: the fail-loud branch lives in
`wire_previews`, which returns `Option<i32>`, NOT in the `Option`-returning `locate_in`).
When set-and-existing, the path is forwarded to `locate_sidecar` as `explicit` and still
flows through the UNCHANGED version gate. Cycle-A logic (None graceful-degrade,
EmptyOutput, dirty-dir refusal) untouched.

Note: `is_file()` is false for a directory too, so `ME_PREVIEW_BIN` pointing at a dir
also fails loud — the "does not point to an existing file" wording is accurate for both.

**Green.** `ME_REQUIRE_GO=1 cargo test --locked` → **103 passed, 0 failed, 0 ignored**
(D1 102 + 1). `cargo clippy --all-targets -- -D warnings` → clean.

## Step D3 — documentation

- `locate_sidecar` doc comment: already rewritten in D1 (the stale "Each entry on
  `$PATH`" bullet is gone; it now states discovery is co-located-only, spells out the
  F11 security rationale — "$PATH is deliberately NOT searched … so a future editor does
  not restore the fallback" — and points a non-standard install at `ME_PREVIEW_BIN`).
  No further change needed here.
- `--preview` clap help (main.rs): replaced "If the sidecar is missing…" with the
  discovery contract — sidecar found only alongside `me`, `$PATH` not searched, explicit
  `ME_PREVIEW_BIN=/path/to/me-preview` for non-standard installs, still degrades if
  absent.
- README: added a **Plate previews** subsection under Usage — states co-located-only
  discovery, "does not search `$PATH`", the `ME_PREVIEW_BIN` opt-in (with an example),
  graceful skip, and the set-but-missing exit-2 error.

No test asserts on help/README text (verified), so this step is doc-only.

**Green.** `ME_REQUIRE_GO=1 cargo test --locked` → **103 passed, 0 failed, 0 ignored**.
`cargo clippy --all-targets -- -D warnings` → clean.

## Final verification

- `ME_REQUIRE_GO=1 cargo test --locked` (root, submodule initialized) →
  **103 passed, 0 failed, 0 ignored** (baseline 96 + 7 new: 5 `locate_in` unit,
  `planted_path_sidecar_ignored`, `set_but_missing_me_preview_bin_exit_2`). Zero skips.
- `cargo clippy --all-targets -- -D warnings` → clean.
- `go test ./...` in `preview/` → `ok  mnemonic-engrave/preview` (still green).

**Manual e2e** (scratch harness outside the worktree: a copy of `me` at `appdir/me`, a
fake `me-preview` that prints `me-preview 0.3.0` and writes a valid SVG; input =
md1 + 2×mk1). All four recorded:
1. Co-located `appdir/me-preview` present, no `ME_PREVIEW_BIN`, `$PATH` clean →
   **exit 0**, 3 plates rendered ("rendered plate 1/2/3"), 3 preview keys, 3 SVGs on disk.
2. Co-located removed, the fake planted on `$PATH` only → **exit 0**, "me: preview
   skipped (install me-preview)", 0 SVGs, 0 preview keys — the `$PATH` binary is NOT used.
3. `ME_PREVIEW_BIN=<the pathbin fake>` (co-located still absent) → **exit 0**, renders
   3 plates again.
4. `ME_PREVIEW_BIN=/nonexistent/me-preview` → **exit 2**, distinct message:
   `me: ME_PREVIEW_BIN=/nonexistent/me-preview does not point to an existing file (set
   it to the me-preview binary, or unset it for co-located discovery)`.

Scratch harness cleaned up after recording.

## For the post-impl reviewer

- **D1/D2 split of the env read.** The `ME_PREVIEW_BIN` *success/None* read landed in D1
  (it is the prerequisite for the mandated D1 test migration — those tests can only be
  green once an existing `ME_PREVIEW_BIN` is honored); D2 upgraded only the set-but-missing
  case from graceful-`None` to fail-loud `EXIT_USAGE(2)`. Both live in `wire_previews`
  (the wrapper), before the version gate; `locate_in` remains pure. Net behavior at HEAD is
  exactly the spec's D2 contract.
- **Empty `ME_PREVIEW_BIN` is treated as unset** (`.filter(|v| !v.is_empty())`) — a
  spec-silent edge; chosen as least-surprising (conventional POSIX path-var semantics). Not
  a set-but-missing error.
- **`is_file()` also rejects a directory** → a `ME_PREVIEW_BIN` pointing at a dir also
  fails loud; the "does not point to an existing file" wording is accurate for both cases.
- **Survivors left correct:** `absent_sidecar_degrades…` keeps its `$PATH` line (now moot
  for discovery) and gained `.env_remove("ME_PREVIEW_BIN")` + a refreshed comment;
  `no_preview_flag_is_byte_for_byte_phase_a` is fully untouched (locate never runs there).
- **Cycle-A surface undisturbed:** the dirty-dir refusal, `EmptyOutput`/F9 gate, and the
  None graceful-degrade all still run strictly after the new env-read/locate step (verified
  green by `dirty_preview_dir_refused_exit_2`, `empty_sidecar_output_exit_4`).
