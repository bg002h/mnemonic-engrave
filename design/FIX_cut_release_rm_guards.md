# FIX: guard the recursive deletes in cut-release

Status: for one review round, then a build agent applies it. Small-fix track.
Ruling (Brian, 2026-11-01): "rewrite is with guards on $D". Base: engrave
`origin/master` 85b72a97. This document changes nothing else; the script, the
harness and the spec change only when it is applied.

Files it changes: `scripts/cut-release.sh`, `scripts/test/cut-release.test.sh`,
`scripts/test/fixtures/cut-release/fake-gh`, `design/SPEC_cut_release.md`.

## 0. What was found and decided

1. `verify_tag` (script, `rm -rf "$d"`) is the only recursive delete in
   `cut-release.sh`. `d` is `"$CR_CACHE/$tag"`.
2. A tag with `/` cannot reach the delete from the command line.
   `verify_tag` has two callers. `verify_main` calls `parse_tag` and
   `row_by_prefix` first; `push_flow` runs `tags_load` first, which does the
   same (refusal P5, exit 3, covered by existing row 7). `verify_tag` itself
   also calls `row_by_prefix` before the delete, and a tag with no table row
   aborts there under the script's `set -e`. `row_by_prefix` needs the tag's
   prefix to equal a table prefix, and no table prefix has a `/`. A tag also
   ends in `N.N.N`, so it is never `.` or `..`. Observed on the unmodified
   script (section 3, "Command-line probe"): `--verify` with
   `x/ms-cli-v0.20.2`, `../ms-cli-v0.20.2` or `ms-cli-v0.20.2/../../v1.2.3`
   exits 2 with the U8 text. The tag guards are therefore defence in depth,
   and their rows call `verify_tag` and the new `cache_guard` function
   directly with `set +e` (rows 74 and 75). `row_by_prefix` failing is then
   ignored, which is the only way a bad tag gets as far as the delete; the
   pre-fix run of row 74 shows it deleting a directory outside the cache.
3. An empty `CR_CACHE` cannot come from the environment either:
   `${CUT_RELEASE_CACHE:-...}` turns a set-but-empty variable into the
   default. The guard still tests it: it costs nothing, and a later edit of
   that line would make the case reachable. Row 73 is a direct call.
   This document does not change the `:-`. (Open point for the controller:
   if an empty `CUT_RELEASE_CACHE` should be refused rather than mean "use the
   default", change `:-` to `-` on the `CR_CACHE=` line and add a row; that is
   a behaviour change this document does not make.)
4. **Convention used for the refusal.** The spec has one bad-environment
   refusal: `CUT_RELEASE_DATE` (spec section 2, `main`): exit 2,
   `cut-release: <VAR> must be <form>`, checked in `main` right after
   `parse_args`, in every mode, before any change. `CUT_RELEASE_CACHE` gets the
   same: exit 2, checked in `main` straight after the date check. That also
   keeps the refusal before the tag push in `--push` (a check only inside
   `verify_tag` would first fire after the tags were pushed). `verify_tag`
   repeats the check through `cache_guard`; if it ever fired after the tag
   push, the existing EXIT trap (spec section 5.3 step 4a) turns it into exit 7
   with the finishing lines, so no trap logic changes.
5. All three `verify_tag` refusals use exit 2 and the `cut-release: ` prefix
   on stderr (spec section 2, "Streams").
6. Row numbers. The spec keeps the offline rows first (`--offline` runs rows
   1-70 and expects 70), so the seven new offline rows are rows 71-77 and the
   eight network rows move from 71-78 to 78-85. Totals become 85 rows, 77
   offline. The harness names rows by function, not number.
7. Beyond the harness file the brief names, `fixtures/cut-release/fake-gh`
   has one `rm -rf "$scratch"` (the `pr merge` scratch clone). It is part of
   the harness, so it is guarded too (edit F1), and the static row (77)
   scans it.
8. "Strictly below `T`" is lexical and also rejects any `..` component;
   otherwise `"$T/x/../.."` would pass. The `T` pattern is applied to the
   resolved `T` (the harness builds it with `readlink -f`); if
   `~/.cache/cut-release-test` is a symlink to a directory with another name,
   the harness now refuses to start (exit 2, message names `T`).

### Refusal texts and exit codes (all exit 2)

| id | where | stderr |
|---|---|---|
| R1 | `main` after the date check; again in `cache_guard` | `cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '<cache>')` |
| R2 | `cache_guard` | `cut-release: '<tag>' cannot name a cache directory: it has a '/' or is '.' or '..'` |
| R3 | `cache_guard` | `cut-release: refusing to delete '<dir>': it is not exactly '<cache>/<tag>' strictly below the cache` |
| R4 | harness `rm_T`, `rm_below_T` | `cut-release.test: refusing to delete '<T>': T is not */cut-release-test/run-*` |
| R5 | harness `rm_below_T` | `cut-release.test: refusing to delete '<path>': it has a '..' component` |
| R6 | harness `rm_below_T` | `cut-release.test: refusing to delete '<path>': it is not strictly below <T>` |
| R7 | fake-gh (exit 1, as its other failures) | `fake gh: refusing to delete '<dir>': it is not below <T>` |

"Absolute path other than `/`" means: starts with `/` and has at least one
non-`/` character (so `//` is refused as `/`). The tests are R1: `[[ $CR_CACHE
!= /* || $CR_CACHE != *[!/]* ]]`; R2: `case $tag in */*|.|..`; R3: `dir` is
not equal to `"$CR_CACHE/$tag"`, or does not match `"$CR_CACHE"/?*`.

### Spec-sentence to row coverage

| spec sentence (after this fix) | row |
|---|---|
| S1: cache must be absolute, not `/`; exit 2 and text, in every mode, before any work | 71, 72 (72 also with `--push`) |
| S2 check (1) inside `verify_tag` (empty cache) | 73 |
| S2 check (2): tag with `/`, `.`, `..` | 74 |
| S2 check (3): dir exactly `<cache>/<tag>`, strictly below | 75 |
| S3: harness deletes of `T`, of other paths, refusal text and exit | 76 |
| S3: every recursive delete is `rm -rf -- "${VAR:?}"` | 77 |

## 1. Spec delta (`design/SPEC_cut_release.md`)

Each edit is an exact OLD and NEW; each OLD occurs exactly once in its file at
the base. Apply them in file order with the applier in section 5.

### S1 | design/SPEC_cut_release.md | section 2, CUT_RELEASE_CACHE row

````text
| `CUT_RELEASE_CACHE` | `${XDG_CACHE_HOME:-$HOME/.cache}/cut-release` | downloads go to `<cache>/<tag>/`, which is deleted and recreated per verify |
````

````text
| `CUT_RELEASE_CACHE` | `${XDG_CACHE_HOME:-$HOME/.cache}/cut-release` | downloads go to `<cache>/<tag>/`, which is deleted and recreated per verify (the delete is guarded, §5.4). It must be an absolute path other than `/`, else exit 2 `cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '<cache>')`, checked right after `CUT_RELEASE_DATE`, in every mode, before any other work |
````

### S2 | design/SPEC_cut_release.md | section 5.4, the delete guard

````text
### 5.4 Verify (one tag; shared by `--push` and `--verify`)

The section opens with `== verify <tag>`. Every check runs, even after a
````

````text
### 5.4 Verify (one tag; shared by `--push` and `--verify`)

**Cache directory.** Before V1, verify deletes and recreates `<cache>/<tag>`
(`<cache>` is `CUT_RELEASE_CACHE` or its default). It first checks, in this
order, and on the first failure prints one line to stderr, exits 2 and deletes
nothing: (1) `<cache>` is an absolute path other than `/` (a run of slashes
counts as `/`): `cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '<cache>')`;
(2) `<tag>` has no `/` and is not `.` or `..`: `cut-release: '<tag>' cannot name a cache directory: it has a '/' or is '.' or '..'`;
(3) the directory is exactly `<cache>/<tag>` and lies strictly below
`<cache>`: `cut-release: refusing to delete '<dir>': it is not exactly '<cache>/<tag>' strictly below the cache`.
The delete is `rm -rf -- "${d:?}"`. Check (1) has already run in `main`
before any change, so no new exit appears after the tag push. Checks (2) and
(3) cannot fail from the command line (a tag reaches this point only after its
prefix matched a table row, no prefix has a `/`, and a tag ends in
`MAJOR.MINOR.PATCH`); they guard the function itself, and rows 74 and 75 call
it directly.

The section opens with `== verify <tag>`. Every check runs, even after a
````

### S3 | design/SPEC_cut_release.md | section 9, Harness setup

````text
never under /tmp. T is deleted on full success. `PATH=$T/bin:$HOME/.cargo/bin:$PATH`.
````

````text
never under /tmp. T is deleted on full success. Every recursive delete in the
harness and its fixtures is `rm -rf -- "${VAR:?}"`. Deleting `T` itself (at
the start and at the end) is refused unless `T` matches
`*/cut-release-test/run-*`. Deleting any other path (`mk_eng`'s argument, the
paths `fresh` clears, `FXDIR`, `FXBARE`, the assets directory) is refused
unless it lies strictly below `T` and has no `..` component. A refusal deletes
nothing, prints `cut-release.test: refusing to delete '<path>': <why>` to
stderr and exits 2. Fake gh's merge scratch clone is deleted only when it
lies below `T`, else fake gh prints `fake gh: refusing to delete '<dir>': it is not below <T>` to stderr and exits 1.
Rows 76 and 77 test this. `PATH=$T/bin:$HOME/.cargo/bin:$PATH`.
````

### S4 | design/SPEC_cut_release.md | section 9, fake gh (pr merge)

````text
  `git merge --no-ff` of the head in a scratch clone of the bare remote,
  then pushes. `FAKE_GH_KILL=TERM`: on the first `run list` call, runs
````

````text
  `git merge --no-ff` of the head in a scratch clone of the bare remote,
  then pushes and deletes the clone (guarded as in Harness setup).
  `FAKE_GH_KILL=TERM`: on the first `run list` call, runs
````

### S5 | design/SPEC_cut_release.md | section 9, intro counts

````text
number of rows run equals the expected count: 78 by default, 70 with
`--offline` (rows 1-70). A row "shows L" when the named stream has L as a whole line.
````

````text
number of rows run equals the expected count: 85 by default, 77 with
`--offline` (rows 1-77). A row "shows L" when the named stream has L as a whole line.
````

### S6 | design/SPEC_cut_release.md | section 9, new offline rows 71-77

````text
exists; toolkit: local master = bare master = (b) |

Network rows (skipped by `--offline`; need gh auth and rustup's cargo
````

````text
exists; toolkit: local master = bare master = (b) |
| 71 | cr_cache_root_slash | `CUT_RELEASE_CACHE=/`; `--verify ms-cli-v0.20.2`, run from `$T/none` | 2; `cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '/')`; stdout has no line starting `== verify`; `$T/gh.log` absent or empty |
| 72 | cr_cache_root_relative | `CUT_RELEASE_CACHE=cache`; `--verify ms-cli-v0.20.2`, run from `$T/none`; then F-ms ready to push, as row 36 (`cr_push_run_failed`) sets it up, `CUT_RELEASE_CACHE=cache`, `--push ms-cli-v0.20.3` | 2; the same line with `'cache'`; stdout has no line starting `== verify`; `$T/none/cache` does not exist; the push: 2, the same line, nothing changed (tree clean, HEAD and the bare remote unchanged, gh.log and pvs.log empty, no local or remote tag `ms-cli-v0.20.3`) |
| 73 | cr_cache_root_empty | source the script, `set +e`, `CR_CACHE=''`, call `verify_tag ms-cli-v0.20.2` (the command line cannot reach this: `${CUT_RELEASE_CACHE:-...}` turns an empty variable into the default) | 2; the same line with `''`; stdout has no line starting `== verify` |
| 74 | cr_cache_tag_slash | `$T/cache` and `$T/escape/ms-cli-v0.20.2/keep` exist. Source the script, `set +e`, call `verify_tag '../escape/ms-cli-v0.20.2'`; then `cache_guard <tag> "$CR_CACHE/<tag>"` for the tags `.`, `..` and `a/b`; then the command line `--verify 'x/ms-cli-v0.20.2'` | verify_tag: 2; `cut-release: '../escape/ms-cli-v0.20.2' cannot name a cache directory: it has a '/' or is '.' or '..'`; `$T/escape/ms-cli-v0.20.2/keep` still exists. Each cache_guard: 2 and that line for its tag. Command line: 2; `cut-release: 'x/ms-cli-v0.20.2' is not a release tag of any repo in the table` (U8; it never reaches verify_tag) |
| 75 | cr_cache_dir_not_below | source the script, `set +e`; `cache_guard ms-cli-v0.20.2 <dir>` for <dir> = `$T/cache`, `$T/cache/other`, `$T`, `$T/cache/ms-cli-v0.20.2/x`; then `cache_guard '' $T/cache/` (an empty tag, which only check (3) catches); then `cache_guard ms-cli-v0.20.2 $T/cache/ms-cli-v0.20.2` | the first four: 2; `cut-release: refusing to delete '<dir>': it is not exactly '<T>/cache/ms-cli-v0.20.2' strictly below the cache`. The empty tag: 2; `cut-release: refusing to delete '<T>/cache/': it is not exactly '<T>/cache/' strictly below the cache`. The last: 0 and nothing on stderr |
| 76 | cr_harness_rm_guards | `rm_T` and `rm_below_T` called in subshells. A scratch directory `<cache-root>/guard.XXXXXX` (made by `mktemp -d`; it matches no `*/cut-release-test/run-*`) holds a file `keep`; `$T/guard/cut-release-test/run-1` holds `sub`; `$T/guard/outside` exists | with `T` = the scratch directory: `rm_T`, and `rm_below_T <scratch>/x`: 2; `cut-release.test: refusing to delete '<scratch>': T is not */cut-release-test/run-*`; `keep` survives. `rm_T` with `T` empty: 2, the same line with `''`. With `T` = `run-1`: `rm_below_T` of `outside`, of `run-1` itself and of `''`: 2; `cut-release.test: refusing to delete '<path>': it is not strictly below <run-1>`; of `run-1/sub/../../outside`: 2; `cut-release.test: refusing to delete '<path>': it has a '..' component`; `outside` and `sub` survive. `rm_below_T run-1/sub`: 0, `sub` gone. `rm_T`: 0, `run-1` gone. The scratch directory is removed at the end of the row with `rm -f` and `rmdir` |
| 77 | cr_static_rm | `grep -nE '(^|[^[:alnum:]_])rm +(-[A-Za-z]*[rR][A-Za-z]*\|--recursive)( \|$)'` over `scripts/cut-release.sh`, the harness, `fixtures/cut-release/fake-gh` and `fake-pvs`, comment lines dropped | exactly 4 lines (1 in the script, 2 in the harness, 1 in fake-gh), each matching `rm -<flags> -- "${NAME:?}"` |

Network rows (skipped by `--offline`; need gh auth and rustup's cargo
````

### S7 | design/SPEC_cut_release.md | section 9, network rows move to 78-85

````text
| 71 | crn_verify_ms |
````

````text
| 78 | crn_verify_ms |
````

````text
| 72 | crn_verify_mt |
````

````text
| 79 | crn_verify_mt |
````

````text
| 73 | crn_verify_md_unsigned |
````

````text
| 80 | crn_verify_md_unsigned |
````

````text
| 74 | crn_verify_engrave_retired |
````

````text
| 81 | crn_verify_engrave_retired |
````

````text
| 75 | crn_verify_all |
````

````text
| 82 | crn_verify_all |
````

````text
| 76 | crn_real_mt_dry |
````

````text
| 83 | crn_real_mt_dry |
````

````text
| 77 | crn_real_ms_dry |
````

````text
| 84 | crn_real_ms_dry |
````

````text
| 78 | crn_real_tk_dry |
````

````text
| 85 | crn_real_tk_dry |
````

### S8 | design/SPEC_cut_release.md | section 10, gate count

````text
   `cut-release.test: 78 passed, 0 failed` (with gh logged in and
````

````text
   `cut-release.test: 85 passed, 0 failed` (with gh logged in and
````

## 2. Script and harness changes

### C1 | scripts/cut-release.sh | `main`, after the date check

````text
    refuse 2 "CUT_RELEASE_DATE must be YYYY-MM-DD"
  fi
  need_tools
````

````text
    refuse 2 "CUT_RELEASE_DATE must be YYYY-MM-DD"
  fi
  cache_root_check
  need_tools
````

### C2 | scripts/cut-release.sh | two new functions, above `verify_tag`

````text
verify_tag() {
  local tag=$1 slug prefix N bin archive d rel names list prevtag PV pn want s out rc c comment
````

````text
# cache_root_check: CR_CACHE must be an absolute path other than "/" (a run of
# slashes is "/"); refuses with exit 2 otherwise (spec 2, CUT_RELEASE_CACHE)
cache_root_check() {
  if [[ $CR_CACHE != /* || $CR_CACHE != *[!/]* ]]; then
    refuse 2 "CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '$CR_CACHE')"
  fi
}
# cache_guard <tag> <dir>: run before verify deletes <dir>. Refuses with exit 2
# unless the cache root is sound, <tag> is one plain path component and <dir>
# is exactly "$CR_CACHE/<tag>", strictly below the cache (spec 5.4)
cache_guard() {
  local tag=$1 dir=$2
  cache_root_check
  case $tag in
    */*|.|..) refuse 2 "'$tag' cannot name a cache directory: it has a '/' or is '.' or '..'" ;;
  esac
  if [ "$dir" != "$CR_CACHE/$tag" ] || [[ $dir != "$CR_CACHE"/?* ]]; then
    refuse 2 "refusing to delete '$dir': it is not exactly '$CR_CACHE/$tag' strictly below the cache"
  fi
}

verify_tag() {
  local tag=$1 slug prefix N bin archive d rel names list prevtag PV pn want s out rc c comment
````

### C3 | scripts/cut-release.sh | `verify_tag`, the delete

````text
  d="$CR_CACHE/$tag"
  rm -rf "$d"; mkdir -p "$d"
````

````text
  d="$CR_CACHE/$tag"
  cache_guard "$tag" "$d"
  rm -rf -- "${d:?}"; mkdir -p "$d"
````

### H1 | scripts/test/cut-release.test.sh | header comment and EXPECT counts

````text
# cut-release.test.sh -- the 78 rows of design/SPEC_cut_release.md section 9.
````

````text
# cut-release.test.sh -- the 85 rows of design/SPEC_cut_release.md section 9.
````

````text
#   bash scripts/test/cut-release.test.sh            # 78 rows (70 offline cr_, 8 network crn_)
#   bash scripts/test/cut-release.test.sh --offline  # rows 1-70 only
````

````text
#   bash scripts/test/cut-release.test.sh            # 85 rows (77 offline cr_, 8 network crn_)
#   bash scripts/test/cut-release.test.sh --offline  # rows 1-77 only
````

````text
# number of rows run is the expected 78 (70 with --offline).
````

````text
# number of rows run is the expected 85 (77 with --offline).
````

````text
# push-via-staging and a throwaway minisign key. The network rows (71-78) use
````

````text
# push-via-staging and a throwaway minisign key. The network rows (78-85) use
````

````text
EXPECT=78
[ "$OFFLINE" -eq 0 ] || EXPECT=70
````

````text
EXPECT=85
[ "$OFFLINE" -eq 0 ] || EXPECT=77
````

### H2 | scripts/test/cut-release.test.sh | the delete helpers, the start-up delete and `cleanup`

````text
T=$(readlink -f "$CACHEROOT")/run-$$
rm -rf "$T"
mkdir -p "$T/bin" "$T/key" "$T/key2" "$T/none" "$T/remote" "$T/work"
KEEP=0
cleanup() {
    if [ "$KEEP" -eq 0 ]; then rm -rf "$T"; else echo "cut-release.test: kept $T" >&2; fi
}
````

````text
T=$(readlink -f "$CACHEROOT")/run-$$
# Every recursive delete in this file goes through rm_T or rm_below_T (spec
# section 9, Harness setup). A refusal deletes nothing and exits 2.
rm_refuse() { echo "cut-release.test: refusing to delete '$1': $2" >&2; exit 2; }
t_check() {
    case ${T:-} in
        */cut-release-test/run-*) ;;
        *) rm_refuse "${T:-}" "T is not */cut-release-test/run-*" ;;
    esac
}
rm_T() { t_check; rm -rf -- "${T:?}"; }
rm_below_T() { # path...
    local p
    t_check
    for p in "$@"; do
        case $p in
            */..|*/../*) rm_refuse "$p" "it has a '..' component" ;;
        esac
        case $p in
            "${T:?}"/?*) ;;
            *) rm_refuse "$p" "it is not strictly below $T" ;;
        esac
    done
    for p in "$@"; do rm -rf -- "${p:?}"; done
}
rm_T
mkdir -p "$T/bin" "$T/key" "$T/key2" "$T/none" "$T/remote" "$T/work"
KEEP=0
cleanup() {
    if [ "$KEEP" -eq 0 ]; then rm_T; else echo "cut-release.test: kept $T" >&2; fi
}
````

### H3 | scripts/test/cut-release.test.sh | `mk_eng`

````text
mk_eng() { # dir
    rm -rf "$1"
````

````text
mk_eng() { # dir
    rm_below_T "$1"
````

### H4 | scripts/test/cut-release.test.sh | `fresh`

````text
    rm -rf "$T/remote" "$T/work" "$T/assets" "$T/clone2" "$T/sib" "$T/cache" "$T/eng" "$T/fakebin" "$T/pk" "$T"/merge.*
````

````text
    rm_below_T "$T/remote" "$T/work" "$T/assets" "$T/clone2" "$T/sib" "$T/cache" "$T/eng" "$T/fakebin" "$T/pk" "$T"/merge.*
````

### H5 | scripts/test/cut-release.test.sh | `fx_init` and `net_clone`

````text
    FXDIR=$T/work/$1; FXBARE=$T/remote/$1.git
    rm -rf "$FXDIR" "$FXBARE"
````

````text
    FXDIR=$T/work/$1; FXBARE=$T/remote/$1.git
    rm_below_T "$FXDIR" "$FXBARE"
````

````text
    FXDIR=$T/work/$repo; FXBARE=$T/remote/$repo.git
    rm -rf "$FXDIR" "$FXBARE"
````

````text
    FXDIR=$T/work/$repo; FXBARE=$T/remote/$repo.git
    rm_below_T "$FXDIR" "$FXBARE"
````

### H6 | scripts/test/cut-release.test.sh | `mk_assets`

````text
    rm -rf "$d" "$T/pk"
````

````text
    rm_below_T "$d" "$T/pk"
````

### H7 | scripts/test/cut-release.test.sh | the seven new rows, after row 70

````text
    x_eq "$(git --git-dir="$FXBARE" rev-parse "$c^")" "$b" "(b) is master"
}

# --------------------------------------------------------------- network rows
````

````text
    x_eq "$(git --git-dir="$FXBARE" rev-parse "$c^")" "$b" "(b) is master"
}
cr_cache_root_slash() {
    fresh; FXDIR=$T/none
    CUT_RELEASE_CACHE=/ run_cr --verify ms-cli-v0.20.2
    x_rc 2
    x_err "cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '/')"
    if grep -q '^== verify' "$OUT"; then rowfail "verify started"; fi
    [ ! -s "$T/gh.log" ] || rowfail "gh was called"
}
cr_cache_root_relative() {
    fresh; FXDIR=$T/none
    CUT_RELEASE_CACHE=cache run_cr --verify ms-cli-v0.20.2
    x_rc 2
    x_err "cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is 'cache')"
    if grep -q '^== verify' "$OUT"; then rowfail "verify started"; fi
    [ ! -e "$T/none/cache" ] || rowfail "a relative cache directory was created"
    fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3; snap
    CUT_RELEASE_CACHE=cache run_cr --push ms-cli-v0.20.3
    x_rc 2
    x_err "cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is 'cache')"
    x_untouched ms-cli-v0.20.3
    x_no_remote_tag ms-cli-v0.20.3
}
cr_cache_root_empty() {
    fresh
    local rc=0
    (source "$CR_REAL"; set +e; CR_CACHE=''; verify_tag ms-cli-v0.20.2) >"$OUT" 2>"$ERR" || rc=$?
    RC=$rc
    x_rc 2
    x_err "cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '')"
    if grep -q '^== verify' "$OUT"; then rowfail "verify started"; fi
}
cr_cache_tag_slash() {
    fresh
    local rc=0 t
    mkdir -p "$T/cache" "$T/escape/ms-cli-v0.20.2"
    : >"$T/escape/ms-cli-v0.20.2/keep"
    (source "$CR_REAL"; set +e; verify_tag '../escape/ms-cli-v0.20.2') >"$OUT" 2>"$ERR" || rc=$?
    RC=$rc
    x_rc 2
    x_err "cut-release: '../escape/ms-cli-v0.20.2' cannot name a cache directory: it has a '/' or is '.' or '..'"
    [ -f "$T/escape/ms-cli-v0.20.2/keep" ] || rowfail "verify_tag deleted a directory outside the cache"
    for t in . .. a/b; do
        rc=0
        (source "$CR_REAL"; set +e; cache_guard "$t" "$CR_CACHE/$t") >"$OUT" 2>"$ERR" || rc=$?
        RC=$rc
        x_rc 2
        x_err "cut-release: '$t' cannot name a cache directory: it has a '/' or is '.' or '..'"
    done
    FXDIR=$T/none
    run_cr --verify 'x/ms-cli-v0.20.2'
    x_rc 2
    x_err "cut-release: 'x/ms-cli-v0.20.2' is not a release tag of any repo in the table"
}
cr_cache_dir_not_below() {
    fresh
    local rc d
    for d in "$T/cache" "$T/cache/other" "$T" "$T/cache/ms-cli-v0.20.2/x"; do
        rc=0
        (source "$CR_REAL"; set +e; cache_guard ms-cli-v0.20.2 "$d") >"$OUT" 2>"$ERR" || rc=$?
        RC=$rc
        x_rc 2
        x_err "cut-release: refusing to delete '$d': it is not exactly '$T/cache/ms-cli-v0.20.2' strictly below the cache"
    done
    rc=0
    (source "$CR_REAL"; set +e; cache_guard '' "$T/cache/") >"$OUT" 2>"$ERR" || rc=$?
    RC=$rc
    x_rc 2
    x_err "cut-release: refusing to delete '$T/cache/': it is not exactly '$T/cache/' strictly below the cache"
    rc=0
    (source "$CR_REAL"; set +e; cache_guard ms-cli-v0.20.2 "$T/cache/ms-cli-v0.20.2") >"$OUT" 2>"$ERR" || rc=$?
    RC=$rc
    x_rc 0
    [ ! -s "$ERR" ] || rowfail "the good path wrote to stderr: $(head -c 200 "$ERR")"
}
cr_harness_rm_guards() {
    fresh
    local g=$T/guard bad run out p
    run=$g/cut-release-test/run-1; out=$g/outside
    bad=$(mktemp -d "$CACHEROOT/guard.XXXXXX") || { rowfail "mktemp failed"; return; }
    : >"$bad/keep"
    mkdir -p "$run/sub" "$out"
    ( T=$bad; rm_T ) >"$OUT" 2>"$ERR"; RC=$?
    x_rc 2; x_err "cut-release.test: refusing to delete '$bad': T is not */cut-release-test/run-*"
    [ -f "$bad/keep" ] || rowfail "rm_T deleted a T that is not */cut-release-test/run-*"
    ( T=''; rm_T ) >"$OUT" 2>"$ERR"; RC=$?
    x_rc 2; x_err "cut-release.test: refusing to delete '': T is not */cut-release-test/run-*"
    ( T=$bad; rm_below_T "$bad/x" ) >"$OUT" 2>"$ERR"; RC=$?
    x_rc 2; x_err "cut-release.test: refusing to delete '$bad': T is not */cut-release-test/run-*"
    for p in "$out" "$run" ''; do
        ( T=$run; rm_below_T "$p" ) >"$OUT" 2>"$ERR"; RC=$?
        x_rc 2; x_err "cut-release.test: refusing to delete '$p': it is not strictly below $run"
    done
    ( T=$run; rm_below_T "$run/sub/../../outside" ) >"$OUT" 2>"$ERR"; RC=$?
    x_rc 2; x_err "cut-release.test: refusing to delete '$run/sub/../../outside': it has a '..' component"
    [ -d "$out" ] && [ -d "$run/sub" ] || rowfail "a refused rm_below_T deleted something"
    ( T=$run; rm_below_T "$run/sub" ) >"$OUT" 2>"$ERR"; RC=$?
    x_rc 0
    [ ! -e "$run/sub" ] || rowfail "rm_below_T did not delete a path below T"
    ( T=$run; rm_T ) >"$OUT" 2>"$ERR"; RC=$?
    x_rc 0
    [ ! -e "$run" ] || rowfail "rm_T did not delete a T that matches"
    rm -f -- "$bad/keep"; rmdir -- "$bad"
}
cr_static_rm() {
    fresh
    local hits bad
    hits=$(grep -nE '(^|[^[:alnum:]_])rm +(-[A-Za-z]*[rR][A-Za-z]*|--recursive)( |$)' "$CR_REAL" "$SELF" "$FIX/fake-gh" "$FIX/fake-pvs" | grep -vE '^[^:]+:[0-9]+:[[:space:]]*#' || true)
    x_eq "$(printf '%s\n' "$hits" | grep -c . || true)" 4 "recursive rm lines"
    bad=$(printf '%s\n' "$hits" | grep -vE 'rm +-[A-Za-z]+ +-- +"\$\{[A-Za-z_]+:\?\}"' | grep . || true)
    [ -z "$bad" ] || rowfail "recursive rm without -- and \${NAME:?}: $(printf '%s' "$bad" | tr '\n' '|')"
}

# --------------------------------------------------------------- network rows
````

### H8 | scripts/test/cut-release.test.sh | `ROW_LIST`

````text
    cr_land_pr_not_clean cr_land_pr_branch_moved cr_follow_on_engrave_demo cr_stale_staging cr_push_gh_fails_after_tag
)
````

````text
    cr_land_pr_not_clean cr_land_pr_branch_moved cr_follow_on_engrave_demo cr_stale_staging cr_push_gh_fails_after_tag
    cr_cache_root_slash cr_cache_root_relative cr_cache_root_empty cr_cache_tag_slash cr_cache_dir_not_below
    cr_harness_rm_guards cr_static_rm
)
````

### F1 | scripts/test/fixtures/cut-release/fake-gh | `pr merge` scratch clone

````text
        rm -rf "$scratch"
````

````text
        case $scratch in
            "${T:?}"/merge.?*) ;;
            *) echo "fake gh: refusing to delete '$scratch': it is not below $T" >&2; exit 1 ;;
        esac
        rm -rf -- "${scratch:?}"
````

## 3. RED rows

All seven rows are new (sections 1 and 2, edit H7). They were run as printed
below against the unmodified script and harness (edits H1, H7 and H8 applied,
nothing else: the rows are registered and counted, no guard and no helper
exists). Observed on base 85b72a97, in a scratch copy outside `/tmp`.

Run from the repo root, as a non-root user. `/ms-cli-v0.20.2` must not exist:
before the fix, rows 71 and 73 reach `rm -rf` and `mkdir` of
`//ms-cli-v0.20.2` and `/ms-cli-v0.20.2` (observed: `mkdir ... Permission
denied`). Nothing else leaves `$T`, except row 76's scratch directory
`~/.cache/cut-release-test/guard.XXXXXX` (removed by the row).

```
CUT_RELEASE_TEST_ONLY="cr_cache_root_slash cr_cache_root_relative cr_cache_root_empty cr_cache_tag_slash cr_cache_dir_not_below cr_harness_rm_guards cr_static_rm" bash scripts/test/cut-release.test.sh --offline
```

Expected RED: seven `FAIL <row>: ...` lines, then `cut-release.test: 0 passed, 7 failed`,
`cut-release.test: kept <T>` and exit 1. A failing run keeps its `T`; delete it
with `rm -r -- <the path printed>`.

Expected GREEN (sections 1 and 2 all applied): seven `ok` lines, `cut-release.test: 7 passed, 0 failed`, then
`cut-release.test: ran 7 rows, expected 77` and `cut-release.test: kept <T>` and
exit 1 (the `CUT_RELEASE_TEST_ONLY` aid always exits 1, by design; delete the
kept `T` with `rm -r -- <path>`).

### Observed pre-fix, per row

`$T` stands for `~/.cache/cut-release-test/run-<pid>`. Each entry is copied
from the `FAIL` line of the run (cut for length); a row lists every failed
assertion, the first ones are shown.

| row | observed pre-fix | why it fails |
|---|---|---|
| 71 cr_cache_root_slash | `exit 1, wanted 2 (stderr: mkdir: cannot create directory '//ms-cli-v0.20.2': Permission denied)`; `stderr lacks line: cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '/')`; `gh was called` | no refusal; the script goes on to `rm -rf //ms-cli-v0.20.2` and calls gh |
| 72 cr_cache_root_relative | `exit 8, wanted 2 (stderr: )`; `stderr lacks line: ... (the cache directory is 'cache')`; `verify started`; `a relative cache directory was created`; `exit 0, wanted 2 (stderr: )`; `stderr lacks line: ... (the cache directory is 'cache')`; `tree not clean`; `gh.log is not empty`; `pvs.log is not empty`; `the bare remote changed`; `HEAD moved`; `local tag ms-cli-v0.20.3 exists`; `tag ms-cli-v0.20.3 is on the bare remote` | no refusal; the verify ran and made `cache/` in the working directory, and the push ran and pushed the tag |
| 73 cr_cache_root_empty | `exit 0, wanted 2 (stderr: mkdir: cannot create directory '/ms-cli-v0.20.2': Permission denied)`; `stderr lacks line: ... (the cache directory is '')`; `verify started` | an empty cache root is accepted; `d` becomes `/ms-cli-v0.20.2` |
| 74 cr_cache_tag_slash | `exit 0, wanted 2 (stderr: )`; `stderr lacks line: cut-release: '../escape/ms-cli-v0.20.2' cannot name a cache directory: ...`; `verify_tag deleted a directory outside the cache`; then, for each of `.`, `..`, `a/b`, `exit 127, wanted 2 (stderr: scripts/test/cut-release.test.sh: line 1009: cache_guard: command not found)` | the delete leaves the cache: the sentinel `$T/escape/ms-cli-v0.20.2/keep` is gone; `cache_guard` does not exist yet. The final command-line step (U8) passes before and after |
| 75 cr_cache_dir_not_below | for each of the four bad directories, `exit 127, wanted 2 (stderr: scripts/test/cut-release.test.sh: line 1024: cache_guard: command not found)` and `stderr lacks line: cut-release: refusing to delete '$T/cache': it is not exactly '$T/cache/ms-cli-v0.20.2' strictly below the cache`; for the empty tag and for the good path `exit 127` (wanted 2 and 0) | `cache_guard` does not exist yet |
| 76 cr_harness_rm_guards | `exit 127, wanted 2 (stderr: scripts/test/cut-release.test.sh: line 1042: rm_T: command not found)`; `stderr lacks line: cut-release.test: refusing to delete '<scratch>': T is not */cut-release-test/run-*`; the same for `''`; 18 failed assertions in all | `rm_T` and `rm_below_T` do not exist yet |
| 77 cr_static_rm | `recursive rm lines: got '9', wanted '4'`; `recursive rm without -- and ${NAME:?}: scripts/cut-release.sh:1546: rm -rf "$d"; ...` | nine unguarded recursive deletes: `cut-release.sh:1546`, `cut-release.test.sh:36, 40, 83, 117, 150, 275, 1079`, `fake-gh:183` |

Rows 73 to 76 fail on a missing function (exit 127) or on the missing
refusal; the behaviour they assert is the one in section 2. The sentinel in
row 74 is the pre-fix evidence that the unguarded `rm -rf` leaves the cache
(`verify_tag` with the tag `../escape/ms-cli-v0.20.2` deleted
`$T/escape/ms-cli-v0.20.2`). Rows 71 and 72 fail through the real command line
on the unmodified script.

### Command-line probe (observed; base script; scratch working directory that is not a repository; `CUT_RELEASE_CACHE` a scratch directory)

```
--verify x/ms-cli-v0.20.2                  exit 2  cut-release: 'x/ms-cli-v0.20.2' is not a release tag of any repo in the table
--verify ../ms-cli-v0.20.2                 exit 2  cut-release: '../ms-cli-v0.20.2' is not a release tag of any repo in the table
--verify ms-cli-v0.20.2/../../v1.2.3       exit 2  cut-release: 'ms-cli-v0.20.2/../../v1.2.3' is not a release tag of any repo in the table
```

`--dry-run` and `--push` with `x/ms-cli-v0.20.3` stop earlier in that
directory (exit 3, `not inside a git checkout`); their tag refusal is P5 (row
7, `tags_load`), so no `/` tag reaches `verify_tag` from the command line.
Therefore rows 74 and 75 call the functions directly; row 74 also runs the
command-line case so that the U8 refusal stays covered.

## 4. The tests that change

- New rows 71-77 in `scripts/test/cut-release.test.sh` and in spec section 9
  (edits S6, H7, H8).
- Renumbered, no behaviour change: the eight network rows (71-78 become
  78-85) in spec section 9 (edit S7) and in the harness header comments
  (edit H1). The harness names rows by function, so no function name changes.
- Counts: `EXPECT` 78/70 becomes 85/77; spec section 9 intro and section 10
  step 5 say 85 and 77 (edits S5, S8, H1). `design/FOLLOWUPS.md` keeps its old
  row numbers; they are history.
- Existing rows whose setup now goes through the guards, with no assertion or
  expected output changed: every row that calls `fresh`, `mk_eng`, `fx_init`,
  `net_clone` or `mk_assets` (edits H2 to H6), and the rows that use fake gh's
  `pr merge` (rows 46 and 67, edit F1).
- Not touched: `.github/`, `scripts/push-via-staging.sh`, `fake-pvs`, the
  toolkit manual (no `mr`, `md-cli` or toolkit flag or printed text changes;
  `CUT_RELEASE_CACHE` appears only in the spec, the script and the harness).

## 5. Applying and checking

The edits in sections 1 and 2 are the only `### ID | file | title` entries;
each holds an even number of four-backtick `text` blocks, OLD then NEW. This
applier is what the author ran (stdlib only). Run it from a scratch copy of
the base, or in the worktree to apply for real; it stops at the first OLD that
does not occur exactly once.

```python
#!/usr/bin/env python3
# usage: apply.py FIXDOC TARGET_ROOT [ID,ID,...]
import re, sys, pathlib
doc, root = sys.argv[1], pathlib.Path(sys.argv[2])
only = set(sys.argv[3].split(',')) if len(sys.argv) > 3 else None
text = pathlib.Path(doc).read_text()
head = re.compile(r'^### (\w+) \| ([^|]+?) \| (.*)$', re.M)
fence = re.compile(r'^````text\n(.*?)\n````$', re.M | re.S)
marks = [(m.start(), m) for m in head.finditer(text)]
ends = [m.start() for m in re.finditer(r'^## \d+\. ', text, re.M)] + [pos for pos, _ in marks]
n = 0
for pos, m in marks:
    nxt = min([e for e in ends if e > pos] or [len(text)])
    blocks = fence.findall(text[m.end():nxt])
    ident, rel = m.group(1), m.group(2).strip()
    if len(blocks) % 2 or not blocks:
        sys.exit(f"{ident}: {len(blocks)} blocks")
    if only is not None and ident not in only:
        continue
    p = root / rel
    s = p.read_text()
    for j in range(0, len(blocks), 2):
        c = s.count(blocks[j])
        if c != 1:
            sys.exit(f"{ident} pair {j//2 + 1}: OLD occurs {c} times in {rel}")
        s = s.replace(blocks[j], blocks[j + 1]); n += 1
    p.write_text(s)
    print(f"{ident}: {len(blocks)//2} pair(s) applied to {rel}")
print(f"total pairs: {n}")
```

Build order: apply H1, H7 and H8 first and run the printed command for the
RED result; then apply the rest (all of S, C, H2 to H6, F1), re-run the
printed command for the GREEN result, then the whole offline gate:

```
bash scripts/test/cut-release.test.sh --offline
```

Machine-checked before review (author's run, scratch copy of base 85b72a97):

- The applier above applied all 32 OLD/NEW pairs of sections 1 and 2, each OLD
  found exactly once, and `bash -n` passes on the three changed shell files.
- The seven rows, run with the printed command on the applied tree: `7 passed,
  0 failed` (then `ran 7 rows, expected 77`, exit 1, as in section 3).
- The whole offline gate on the applied tree: `cut-release.test: 77 passed,
  0 failed`, exit 0.
- Ten hand-made mutants of the applied tree were each failed by at least one of
  the seven rows: no `cache_guard` call in `verify_tag`; R1 without its
  `*[!/]*` test; R3 without its strictly-below clause; R3 without its equality
  test; no `cache_root_check` call in `main`; no `cache_root_check` inside
  `cache_guard`; R2 without the `.` and `..` cases; `rm_T` without
  `t_check`; `rm_below_T` without its `..` test; the script's delete without
  `--`.

Not machine-checked: the network rows (78-85; they need gh auth and rustup's
cargo, and do not touch the deleted paths) and the wording of the S1 to S4
spec sentences.
