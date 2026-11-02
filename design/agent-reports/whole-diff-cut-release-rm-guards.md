# Whole-diff review: cut-release rm guards (008048be vs f5d31da2)

Reviewer: Sonnet 5.5, medium. Lens: does the build match FIX_cut_release_rm_guards.md,
and can each new row fail when its guard is removed.

## Build vs FIX
Script (C1-C3), harness (H1, H2, H7, H8 and the H3-H6 call sites) and fake-gh (F1)
match the FIX text; spec edits S1-S8 are present once each (key phrases grepped:
5.4 "Cache directory", section 9 delete rule, 85/77 counts, rows 71-77, network rows
78-85, gate count 85). No stale 78/70 counts remain in the harness; the spec's only
78/70 hits are row numbers of existing rows.

## Plant results (each as a scratch change to the worktree, reverted after;
## rows run: the 7 new rows, `--offline`, via CUT_RELEASE_TEST_ONLY)
| planted removal | rows that failed |
|---|---|
| `cache_root_check` call in `main` | cr_cache_root_slash, cr_cache_root_relative |
| `cache_root_check` inside `cache_guard` | cr_cache_root_empty |
| tag `case` (`*/*`, `.`, `..`) | cr_cache_tag_slash |
| "dir equals cache/tag" test | cr_cache_dir_not_below |
| "strictly below cache" test | cr_cache_dir_not_below |
| `cache_guard` call in `verify_tag` | cr_cache_root_empty, cr_cache_tag_slash |
| `--` on the script delete / `${d:?}` dropped | cr_static_rm |
| `t_check` in `rm_T` | cr_harness_rm_guards |
| `t_check` in `rm_below_T` | cr_harness_rm_guards |
| `..` component test | cr_harness_rm_guards |
| strictly-below test (removed; and loosened to `"$T"*`) | cr_harness_rm_guards (both) |
| `--` dropped in fake-gh delete; raw delete in `mk_eng`; raw delete in `fresh` | cr_static_rm |
| **fake-gh `case $scratch` refusal removed** | **none (7 passed, 0 failed)** |

All other guards are caught. Worktree clean after the plants (`git status` empty).

## Findings

**M1 | design/SPEC_cut_release.md (section 9, Harness setup, last sentences) and
fake-gh:183-186 | the fake-gh refusal (R7) has no row that can fail.**
Removing the `case $scratch in ... esac` block leaves all 7 new rows green; row 77
checks only the shape `rm -rf -- "${NAME:?}"`, not the `case`. The spec says
"Rows 76 and 77 test this", which overclaims for the fake-gh half. Likelihood:
Minor, fails closed otherwise: `scratch` is always `$T/merge.XXXXXX` from `mktemp -d`
in the same script, so reaching the refusal needs an edit of that line plus a `T`
mismatch (two unusual conditions). Fix text (spec, replace "Rows 76 and 77 test
this."): "Row 76 tests the harness deletes; row 77 checks that every recursive delete
has the guarded shape (fake gh's refusal itself is not exercised)."

N1 | each failed or `CUT_RELEASE_TEST_ONLY` run leaves
`~/.cache/cut-release-test/run-<pid>` (by design, "kept"). My plant runs added about
20; the controller may remove them with `rm -r -- <path>`.

Nothing re-reported from F-713.

C / I / M / N: 0 / 0 / 1 / 1
