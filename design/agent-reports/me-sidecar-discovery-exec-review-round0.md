# Post-implementation adversarial execution review — me sidecar discovery integrity (Cycle D / F11), round 0

Reviewer: opus post-impl execution reviewer (independent, mandatory non-deferrable gate)
Date: 2026-07-09
Diff under review: `origin/master..me-sidecar-discovery` == `4908dbb..HEAD` (`3aba5dd`)
Base: merged master `4908dbb` (Cycles A/B/C landed). `merge-base HEAD origin/master` = `4908dbb`.
Worktree: `/scratch/code/shibboleth/me-cycleD`
Spec: `design/SPEC_me_sidecar_discovery.md` (GREEN at R0 round 1: 0C/0I/0L/6 advisory nits)
Standard: GREEN = 0 Critical / 0 Important, verified by re-performed probes (not by assertion).

Verdict: **GREEN — 0 Critical, 0 Important, 0 Low, 2 Nits (advisory).**

The diff genuinely and completely closes F11. I re-performed the teeth check, all four
fail-loud/precedence probes, and the full suites myself; every acceptance criterion holds.
The two nits are cosmetic / marginal-edge notes that do not gate.

---

## 1. Does it actually close F11? — YES, no residual auto-reach of a $PATH binary

**Grep for residual `$PATH` in the locate path — clean.**
`grep -n 'split_paths\|var_os("PATH")\|var("PATH")\|env("PATH"' crates/me-cli/src/preview.rs
crates/me-cli/src/main.rs` → **exit 1 (no matches)**. The only `std::env` refs in `preview.rs`
are `current_exe()` (line 97, the `locate_sidecar` wrapper) and `temp_dir()` (line 379, a test
helper). No `$PATH` anywhere in production discovery.

**`locate_in` (preview.rs:67–80) is pure and has NO `$PATH` arm.** Signature
`fn locate_in(exe_dir: Option<&Path>, explicit: Option<&Path>) -> Option<PathBuf>`; body is
exactly two arms — (1) `explicit` returned verbatim, (2) `exe_dir.join(sidecar_filename())` if
`is_file()` — then `None`. It reads no env and consults no `$PATH` (property true *by
construction*: the code path is deleted, not merely bypassed).

**`locate_sidecar` (preview.rs:96–99) has no `$PATH` arm.** It supplies
`current_exe().ok().parent()` as `exe_dir` and delegates to `locate_in`. The old `// 2) On $PATH`
block (the entire F11 exposure) is deleted.

**Version gate still applies to an explicitly-chosen binary — re-verified by probe.** The
`ME_PREVIEW_BIN` path flows `wire_previews` → `locate_sidecar(explicit)` → returned → the
UNCHANGED version gate (`main.rs:268–283`). Probe with an explicit binary printing `me-preview
9.9.9`:
```
$ ME_PREVIEW_BIN=<fake_wrong> me bundle --preview out4   # fake prints "me-preview 9.9.9"
exit=2
me: me-preview version mismatch: sidecar is "9.9.9", expected "0.3.0"; refusing to render ...
SVGs written: 0
```
and with a matched-version explicit binary:
```
$ ME_PREVIEW_BIN=<fake_match> me bundle --preview out5   # fake prints "me-preview 0.3.0"
exit=0 ; rendered plate 1/2/3 ; plate-1.svg plate-2.svg plate-3.svg written
```
So an explicit opt-in raises precedence for *discovery* but does NOT bypass `--version` match.
Confirmed.

## 2. Fail-loud correctness (L1/D2) — re-performed all three probes with the real `me`

Control flow traced in `wire_previews` (`main.rs:238–266`): the `ME_PREVIEW_BIN` read
(`var_os` → `.filter(|v| !v.is_empty())` → `.map(PathBuf::from)`) and the `!p.is_file()` →
`eprintln!` + `return Some(EXIT_USAGE)` sit at the **very top**, strictly BEFORE
`locate_sidecar`, the version gate (268–283), the `dir.is_dir()` check (286–292), the Cycle-A
dirty-dir scan (299–321), and the render loop (324–). It does NOT collide with merged Cycle A:
the `None` graceful-degrade, `EmptyOutput`/F9, and dirty-dir refusal all run strictly after this
new step. Re-performed against `target/debug/me`:
```
PROBE 1  ME_PREVIEW_BIN=/nonexistent/me-preview  -> exit 2
  me: ME_PREVIEW_BIN=/nonexistent/me-preview does not point to an existing file
      (set it to the me-preview binary, or unset it for co-located discovery)
PROBE 2  ME_PREVIEW_BIN=<a directory>            -> exit 2   (same distinct message; is_file() false for a dir)
PROBE 3  ME_PREVIEW_BIN=""  (empty)              -> exit 0   "me: preview skipped (install me-preview)"  (treated as unset -> co-located-only)
```
All three exactly match the spec's D2 contract and the impl-log's recorded e2e. The set-but-missing
branch fails loud with a distinct message naming both the env var and the path; empty is treated
as unset (POSIX-conventional, least-surprising); a directory also fails loud (wording accurate).

## 3. Test migration integrity (I1) — exactly 7 + 1, survivors correct, teeth re-verified

`git diff` migration set — the `.env("PATH", &bindir)` → `.env("ME_PREVIEW_BIN",
bindir.join("me-preview"))` change lands in EXACTLY the 7 inject-and-expect-discovery `mod
preview` tests (cli.rs:446/466/483/529/560/594/762 = `empty_sidecar_output_exit_4`,
`render_failure_exit_4`, `matched_version_renders_and_sets_preview_exit_0`, `png_flag_renders_png`,
`dirty_preview_dir_refused_exit_2`, `mismatched_version_exit_2`, `unwritable_preview_dir_exit_2`)
plus the +1 cross-lang test (`preview_cross_lang.rs`, real sidecar via `ME_PREVIEW_BIN`). Count
confirmed: `grep '\.env("ME_PREVIEW_BIN"'` in cli.rs = 7 migrated + 1 (`&missing`, the D2 test);
`grep '\.env("PATH"'` in cli.rs now = only 3 (the two survivors + the new `planted_path`).

**Survivors left correct:**
- `absent_sidecar_degrades_exit_0_with_note_and_manifest` (cli.rs:610–...): kept `.env("PATH",
  &bindir)` (empty bindir, now moot for discovery) and **gained `.env_remove("ME_PREVIEW_BIN")`
  (cli.rs:618)** so a stray ambient env var cannot make it falsely pass — asserts graceful degrade
  (exit 0, "preview skipped", manifest emitted) with NO sidecar reachable. Comment refreshed. ✔
- `no_preview_flag_is_byte_for_byte_phase_a` (cli.rs:726–748): fully untouched — a fake is present
  but `--preview` is absent, so `locate` never runs; byte-for-byte golden compare. ✔
- `unwritable_preview_dir_exit_2` (cli.rs:750–771): adversarial check — `ME_PREVIEW_BIN` points at
  the **existing** fake, and the *missing* path is the separate `--preview` target dir, so
  set-but-missing does NOT misfire; still exit 2. ✔

**Teeth check re-performed (non-vacuous).** In a scratch git worktree at
`/scratch/code/shibboleth/me-review-scratch-d` (detached at `3aba5dd`, submodule inited, separate
`CARGO_TARGET_DIR`), I re-introduced a `$PATH` fallback into `locate_in` and ran the one test:
```
test preview::planted_path_sidecar_ignored ... FAILED
  a $PATH-only sidecar must be ignored -> skip note: me: rendered plate 1 -> .../plate-1.svg
  me: rendered plate 2 -> ...  me: rendered plate 3 -> ...
test result: FAILED. 0 passed; 1 failed
```
So the regression lock is genuinely red when the fallback returns — it truly guards the closure at
the real entry point. Scratch worktree + target removed and `git worktree prune`d afterward.

## 4. `locate_in` unit matrix — pure, covers explicit-precedence, exe-adjacent-hit, none

Five unit tests in `preview.rs mod tests` (514–586): `locate_in_exe_adjacent_hit` (co-located
happy path, otherwise uncovered by integration), `locate_in_exe_adjacent_miss_is_none`,
`locate_in_no_exe_dir_is_none`, `locate_in_explicit_wins_over_exe_adjacent` (precedence with BOTH
present), `locate_in_explicit_returned_verbatim` (returned without re-check). Inputs are only the
two `Option<&Path>` args — pure, no `current_exe`/env/`$PATH`. Matrix matches spec R0-N3 (a/b/c/d).

## 5. Scope — confined to the 5 expected files + design docs; no bleed into A/B/C

`git diff --name-only 4908dbb HEAD`: `README.md`, `crates/me-cli/src/main.rs`,
`crates/me-cli/src/preview.rs`, `crates/me-cli/tests/cli.rs`,
`crates/me-cli/tests/preview_cross_lang.rs`, and 4 `design/` docs (SPEC + impl-log + 2 R0 reviews).
**No changes under `preview/` (the Go sidecar), no `manifest`/`ndef`/`lib` behavioral change.**
`main.rs` has exactly 2 hunks (the `--preview` clap help doc; the `wire_previews` env-read +
`locate_sidecar` signature). All Cycle-A/B/C tests (perms, goldens, dirty-dir, EmptyOutput,
cross-lang, prop) remain green in the full run.

## 6. Full suites — all green, counts as expected

```
env PATH="/home/bcg/.local/go/bin:$PATH" ME_REQUIRE_GO=1 cargo test --locked
  lib 61 | bin(main.rs) 1 | cli 30 | cross_lang 1 | golden 3 | preview_cross_lang 1 | prop 6 | doctests 0
  = 103 passed, 0 failed, 0 ignored, 0 skipped   ✔ (matches expected 103)
cargo clippy --all-targets --locked -- -D warnings   -> clean ✔
cd preview && go test -count=1 ./...                 -> ok  mnemonic-engrave/preview  (fresh, not cached) ✔
```
(61 lib = 56 baseline + 5 `locate_in` unit; cli 30 = 28 baseline + `planted_path_sidecar_ignored`
+ `set_but_missing_me_preview_bin_exit_2`; migration is net-zero on count.)

---

## FINDINGS (all Nits — advisory, none gates GREEN)

**N1 (Nit) — bare-name `ME_PREVIEW_BIN` has an `is_file()`(CWD-relative) vs
`Command::new`(PATH-search) resolution mismatch.** `main.rs:248` checks `p.is_file()` (resolved
relative to CWD for a bare name), but the sidecar is later exec'd via `Command::new(path)`
(`preview.rs:109,144`), which — for a program token containing no path separator — searches
`$PATH`. So `ME_PREVIEW_BIN=me-preview` (a bare relative name, no slash) could version-check a
CWD file yet exec a `$PATH` one. This is NOT the F11 exposure: it requires the user to *explicitly*
set a bare relative `ME_PREVIEW_BIN` (a deliberate opt-in), and it is squarely inside the spec's
accepted vouching threat model ("an attacker who can set the victim's `ME_PREVIEW_BIN` already
controls the process environment and can do strictly worse"). F11 (ambient auto-discovery with no
user action) is fully closed. Optional hardening if ever desired: reject a separator-less
`ME_PREVIEW_BIN` or canonicalize it before exec. Marginal; no action required this cycle.

**N2 (Nit, pre-existing) — stale comment in `unwritable_preview_dir_exit_2` (cli.rs:752–753).**
The comment says "the matched sidecar's render fails -> exit 2," but with a non-existent `--preview`
dir the exit-2 now comes from the Cycle-A `!dir.is_dir()` gate (`main.rs:286–292`) *before* render
runs. The test's `.code(2)` assertion is still correct; only the rationale comment is inaccurate.
Pre-existing (Cycle-A-era), not introduced by this diff — noting for a future sweep.

---

## Verdict

**GREEN — 0 Critical, 0 Important, 0 Low, 2 Nits.** F11 is completely closed: no production path
consults `$PATH`; the pure `locate_in` has no `$PATH` arm; `locate_sidecar` searches only the
exe-adjacent dir; the `ME_PREVIEW_BIN` opt-in fails loud on set-but-missing (exit 2, distinct
message) BEFORE the version gate and does not disturb merged Cycle A; the version gate still applies
to an explicitly-chosen binary; the test migration is exactly 7+1 with the two survivors left
correct (the absence test hardened with `.env_remove`); and the `planted_path_sidecar_ignored` lock
is re-verified non-vacuous. 103 tests pass, clippy clean, go green. Cleared for the controller to
open the PR. The 2 nits are advisory.
