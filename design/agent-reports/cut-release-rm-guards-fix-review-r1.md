**⚠ 2 Important: I1 (the cache-root check accepts `/.`, `/..`, `/usr/..` and a symlink to `/`, so verify deletes `/<tag>`); I2 (no row covers the refusal before the `--push` tag push).**

# FIX review r1: cut-release recursive-delete guards

Reviewer: independent adversarial review (Opus, round 1). Document under review:
`design/FIX_cut_release_rm_guards.md` at 42430811 (branch `fix/cut-release-rm-guards`).
Scratch copy: detached worktree of 85b72a97 at `me-worktrees/review-rm-guards-r1`,
section 5 applier run on it. First round, so no finding comes from a previous fix commit.

## Findings (appended as found)

### I1 (Important) The cache-root check accepts paths that resolve to `/`, so verify still deletes `/<tag>`

**Where:** FIX doc C2 (`cache_root_check`, NEW block line 305), section 0 lines 76-79 ("Absolute path
other than `/` means ..."), S2 NEW check (1) (lines 121-122), S6/H7 row 71.

**Failure (executed on the applied scratch copy, `rm`/`mkdir`/`gh` stubbed):** `cache_root_check` is
lexical: `[[ $CR_CACHE != /* || $CR_CACHE != *[!/]* ]]`. It refuses `''`, `/`, `//`, `///`, but accepts
`/.`, `/..`, `/./`, `//.`, `/tmp/..`, `/x//..`, `/proc/self/root` and any symlink to `/`. For each,
`verify_tag ms-cli-v0.20.2` returns 0 and runs `rm -rf -- /./ms-cli-v0.20.2` (and so on), which
`readlink -m` resolves to `/ms-cli-v0.20.2`: the delete the "other than `/`" refusal exists to stop.
Through the real command line (harness layout; row 71 extended with these values; document's guard):

    FAIL cr_cache_root_slash: exit 1, wanted 2 (stderr: mkdir: cannot create directory '/./ms-cli-v0.20.2': Permission denied|)
       stderr lacks line: ... (the cache directory is '/.')
       gh was called (/.)
       exit 1, wanted 2 (stderr: mkdir: cannot create directory '/usr/../ms-cli-v0.20.2': Permission denied|)
       ... the same for '/usr/..' and for '$T/rootlink' (a symlink to /)

So the recursive delete ran on `/<tag>` (harmless here only because the run is non-root and the
directory is absent). It does not fail closed. Graded Important, not Critical: the deleted entry is
always named exactly `<tag>` (a release tag), and S2's wording "(a run of slashes counts as `/`)"
decides a lexical rule, so this is a missing case rather than a broken stated guarantee. The
symlinked-root and `..`-in-root cases are ones the brief names.

**Fix (run as written: RED above with the document's guard; GREEN = the seven rows give
`7 passed, 0 failed`; whole offline gate at the end of this report):**
- C2 NEW: replace the first comment and the `if` line of `cache_root_check` with

      # cache_root_check: CR_CACHE must be an absolute path that does not resolve to
      # "/" (readlink -m: "//", "/.", "/usr/.." and a symlink to "/" all do); refuses
      # with exit 2 otherwise (spec 2, CUT_RELEASE_CACHE)
      cache_root_check() {
        if [[ $CR_CACHE != /* ]] || [ "$(readlink -m -- "$CR_CACHE")" = / ]; then

  (the `refuse` line and its text are unchanged). The script already depends on GNU coreutils
  (`readlink -f` line 19, `sha256sum`, `timeout`). `readlink -m` needs no component to exist and
  returns the path itself, exit 0, for a symlink loop (checked), so the check cannot abort under `set -e`.
  Re-probed: `/.`, `/..`, `/./`, `//.`, `/tmp/..`, `/x//..`, `/proc/self/root`, a symlink to `/` now
  exit 2; `$HOME` and `/x//` still pass.
- Section 0, lines 76-79: replace "means: starts with `/` and has at least one non-`/` character (so
  `//` is refused as `/`). The tests are R1: `[[ $CR_CACHE != /* || $CR_CACHE != *[!/]* ]]`;" with
  "means: starts with `/` and `readlink -m` does not resolve it to `/` (so `//`, `/.`, `/usr/..` and a
  symlink to `/` are refused as `/`). The tests are R1: `[[ $CR_CACHE != /* ]] || [ "$(readlink -m -- "$CR_CACHE")" = / ]`;".
- S2 NEW: replace "(a run of slashes counts as `/`)" with "(a path that `readlink -m` resolves to `/`,
  such as `//`, `/.`, `/usr/..` or a symlink to `/`, counts as `/`)".
- S6 row 71, column 3: "`CUT_RELEASE_CACHE` set in turn to `/`, `/.`, `/usr/..` and `$T/rootlink` (a
  symlink to `/`, removed by the row); `--verify ms-cli-v0.20.2`, run from `$T/none`"; column 4:
  "each: 2; `cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '<value>')`;
  stdout has no line starting `== verify`; `$T/gh.log` absent or empty".
- H7 NEW: replace `cr_cache_root_slash` with

      cr_cache_root_slash() {
          local c
          fresh; FXDIR=$T/none
          ln -s / "$T/rootlink"
          for c in / /. /usr/.. "$T/rootlink"; do
              CUT_RELEASE_CACHE=$c run_cr --verify ms-cli-v0.20.2
              x_rc 2
              x_err "cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '$c')"
              if grep -q '^== verify' "$OUT"; then rowfail "verify started ($c)"; fi
              [ ! -s "$T/gh.log" ] || rowfail "gh was called ($c)"
          done
          rm -f -- "$T/rootlink"
      }

- Section 3: row 71's pre-fix entry gains the three new values (observed text above); section 5's
  mutant "R1 without its `*[!/]*` test" becomes "R1 without its `readlink -m` test".

### I2 (Important) No row tests the one property that makes the `main` check matter: refusal before the `--push` tag push

**Where:** FIX doc section 0 item 4 ("That also keeps the refusal before the tag push in `--push`"),
S1 ("in every mode, before any other work"), the coverage map line "S1: ... in every mode, before any
work | 71, 72", S6/H7 rows 71 and 72 (both run only `--verify`).

**Failure (planted, executed):** plant A moves `cache_root_check` from the top of `main` into the
`verify)` branch only. All seven new rows still pass (`7 passed, 0 failed`; rows 71-72 use `--verify`,
which still refuses; row 75's direct calls do not go through `main`), and no older row sets a bad cache.
Under plant A a `--push` with a bad cache pushes the tag and only then refuses, from `cache_guard`
inside `verify_tag`, which the EXIT trap turns into exit 7 (observed with the fix row below:
`exit 7, wanted 2 ... stopped after the tag push by a failing command ... tag ms-cli-v0.20.3 is on the
bare remote`). Section 0 item 4 names exactly this as the reason for the `main` check, and the coverage
map claims rows 71-72 cover "in every mode", so the map's claim is false and the guarantee can regress
silently. (C1's exact code is right; the gap is the gate.)

**Fix (run as written):** S6 row 72 and H7 `cr_cache_root_relative` gain a `--push` step after the
existing `--verify` assertions.
- H7 NEW, in `cr_cache_root_relative`, replace
  `    [ ! -e "$T/none/cache" ] || rowfail "a relative cache directory was created"` + `}` with

      [ ! -e "$T/none/cache" ] || rowfail "a relative cache directory was created"
      fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3; snap
      CUT_RELEASE_CACHE=cache run_cr --push ms-cli-v0.20.3
      x_rc 2
      x_err "cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is 'cache')"
      x_untouched ms-cli-v0.20.3
      x_no_remote_tag ms-cli-v0.20.3
  }

- S6 row 72, column 3: append "; then F-ms ready to push, as row 36 (`cr_push_run_failed`) sets it up,
  `CUT_RELEASE_CACHE=cache`, `--push ms-cli-v0.20.3`"; column 4: append "; the push: 2, the same line,
  nothing changed (tree clean, HEAD and the bare remote unchanged, gh.log and pvs.log empty, no local or
  remote tag `ms-cli-v0.20.3`)".
- Coverage map: "S1: ... in every mode, before any work | 71, 72 (72 also with `--push`)".
- Section 3, row 72 observed pre-fix (base script, as run): after the existing lines,
  `exit 0, wanted 2 (stderr: )`; `stderr lacks line: ... (the cache directory is 'cache')`;
  `tree not clean`; `gh.log is not empty`; `pvs.log is not empty`; `the bare remote changed`;
  `HEAD moved`; `local tag ms-cli-v0.20.3 exists`; `tag ms-cli-v0.20.3 is on the bare remote`.
  Observed results of this text: RED on the base as quoted; RED against plant A (`exit 7, wanted 2`,
  tag on the bare remote); GREEN (`ok cr_cache_root_relative`) with the document applied.

## Minors (one line each; executed fix texts for M4 and M7 in the appendix)

- **M1** S1/R1 (FIX doc line 104, C2): with `XDG_CACHE_HOME` relative (the XDG Base Directory spec calls it invalid and says to ignore it) and `CUT_RELEASE_CACHE` unset, every `--push`/`--verify` now refuses with "CUT_RELEASE_CACHE must be an absolute path ..." although the operator never set that variable (executed). It fails closed, the parenthetical shows the effective `<x>/cut-release`, and setting `CUT_RELEASE_CACHE` to an absolute path, the remedy the text implies, works. S1 decides a relative default is refused, so the builder has nothing to choose. Optional: S1 adds "(a relative `XDG_CACHE_HOME` gives a relative default and is refused the same way)".
- **M2** H2 `rm_below_T` (FIX doc ~line 410): `rm_below_T "$T//"` passes both checks (`"${T:?}"/?*` matches with `?` = `/`) and `rm -rf -- "$T//"` deletes `T` itself (executed: rc 0, T gone). Contained to `T`; no caller builds such a path (all arguments are `$T/<literal>`). Conditions: a future caller with a doubled trailing slash, and a wish to keep `T`. Optional: add `*//*` to the R5 case with the text "it has a '..' or '//' component".
- **M3** H2 `t_check`: the pattern's `*` crosses `/`, so `T=<x>/cut-release-test/run-1/../../outside` passes and `rm_T` deleted `outside` (executed in the sandbox). Unreachable: `T` is `$(readlink -f "$CACHEROOT")/run-$$`, which has no `..`. Conditions: `T` built some other way, and with `..`. Optional: apply R5's `..` case in `t_check` too (or see the design line).
- **M4** S6/H7 row 76 versus S3: three plants, one per S3 clause and none in the author's ten, survive all seven rows (`7 passed, 0 failed` for each). Plant B: `t_check` pattern `*/run-*` (accepts a `run-*` directory outside `cut-release-test`). Plant C: `"${T:?}"/*` (accepts `$T/`, which deletes `T`). Plant D: the delete moved inside the check loop, so a refusal of a later argument comes after earlier ones were deleted, breaking "A refusal deletes nothing". The guards as written are right and their inputs come only from harness literals, so a regression needs the plant plus a new caller (two conditions). The appendix's row 76 additions catch each plant (`6 passed, 1 failed`, `FAIL cr_harness_rm_guards`) and pass on the document as written.
- **M5** F1 / S3 last sentence / S4: fake-gh's guard is a tautology, because `scratch=$(mktemp -d "$T/merge.XXXXXX")` (fake-gh line 171) always matches `"${T:?}"/merge.?*`. Plant E (guard deleted) survives the seven rows, and the coverage map gives the S3 fake-gh sentence no row. Fails closed and cannot be reached. Optional: the coverage map adds the line "S3 fake-gh refusal (R7) | none: unreachable, `scratch` is `mktemp -d "$T/merge.XXXXXX"`".
- **M6** S6/H7 row 77: the regex (run on 9 spellings) catches `rm -r -f`, `rm -fr --`, `rm -Rf`, `rm --recursive` and `rm  -rf`, but misses `rm -f -r -- "$d"`, `"rm" -rf "$d"`, `find "$d" -delete` and `git clean -fdx`. So S3's "Every recursive delete ... is `rm -rf -- "${VAR:?}"`" is only partly enforced against future edits. Conditions: a future edit uses one of those spellings, and its path is unsafe. Optional: S6 row 77's column 3 names the spellings it does not catch.
- **M7** S3 "T is deleted on full success" / "A refusal deletes nothing" versus H2: a refusal in the main shell (any `rm_below_T` from `fresh`, `mk_eng` or `fx_init`) exits 2, then the EXIT trap runs `cleanup` with `KEEP=0` and deletes `T`, so the run that hit the refusal is not kept (executed with the harness's own lines 33-66: T gone; with `KEEP=1` first in `rm_refuse`: `cut-release.test: kept <T>`, T left). Fails closed (the refused path survives and is named). Optional fix text in the appendix.

## Nits

- **N1** S3 does not say that deleting another path is also refused when `T` fails the `*/cut-release-test/run-*` test (R4 from `rm_below_T`). Row 76 expects it and the R-table lists it; S3 should add "(and, first, `T` matches as above)".
- **N2** The spec's Harness setup formula (`SPEC_cut_release.md` line 673) still reads `T=${XDG_CACHE_HOME:-$HOME/.cache}/cut-release-test/run-$$`. The harness resolves the cache root with `readlink -f`, and the new R4 refusal depends on that resolved form (the author's point 6 is the consequence). S3 should name the resolved `T`.

## Case tables (part d): every cell decided by the document's exact code and spec text unless marked

`cache_root_check` (R1) and the default:

| `CR_CACHE` source/value | document's code | correct? |
|---|---|---|
| unset, XDG unset / absolute / empty | `$HOME/.cache/cut-release` or `$XDG/cut-release`: pass | yes (executed) |
| `CUT_RELEASE_CACHE=''` | `:-` gives the default: pass | yes (author point 3; agree) |
| `HOME=''`, XDG unset | `/.cache/cut-release`: pass | yes |
| `HOME` unset, XDG unset | line 25 `set -u` abort, exit 1 | pre-existing, not this fix |
| XDG relative | R1 refusal | decided by S1 (M1 wording) |
| `cache`, `~` (unexpanded), `-x` | R1 | yes (row 72) |
| `/`, `//`, `///` | R1 | yes (row 71) |
| `/.`, `/..`, `/./`, `//.`, `/usr/..`, `/x//..`, `/proc/self/root`, symlink to `/` | **pass, delete `/<tag>`** | **I1** |
| `/x//`, `$HOME`, a path with a space or newline | pass, quoted | yes (`$HOME/<tag>` is what the operator asked for) |
| symlink to a real directory | followed; only `<tag>` replaced, its sibling kept | yes (executed, probe 6) |

`cache_guard` tag (R2) and dir (R3), executed through the document's functions:

| input | result | correct? |
|---|---|---|
| `ms-cli-v0.20.2` | pass | yes |
| `a/b`, `/x`, `x/`, `./`, `/`, `.`, `..` | R2 | yes (row 74) |
| `''` | passes R2; R3 (`"$CR"/?*` fails) | yes (author point 8 on the text) |
| `*`, `?`, newline, `-rf`, space, `.hidden`, `...` | one component; quoted `rm -rf --` | yes |
| dir ≠ `$CR/$tag` | R3 | yes (row 75) |
| `<cache>/<tag>` a symlink to a directory outside | link removed, target's file survives, a real directory is made | yes (executed, probe 6) |

`t_check` / `rm_T` (R4):

| `T` | result | correct? |
|---|---|---|
| `<root>/cut-release-test/run-<pid>` with spaces in `<root>` | pass | yes (whole gate 77/0 with a spaced `XDG_CACHE_HOME`) |
| `''`, unset | R4 (`${T:-}`) | yes |
| `<root>/guard.X` | R4 | yes |
| `<x>/run-1` outside `cut-release-test` | R4 | yes (plant B shows no row; M4) |
| `.../cut-release-test/run-1/../../x` | pass, deletes `x` | M3 (unreachable) |
| `cut-release-test` a symlink to another name | R4 at start-up | author point 6 (agree: Minor) |

`rm_below_T` (R5/R6):

| argument | result | correct? |
|---|---|---|
| `$T/x` (several args) | all checked, then all deleted | yes (plant D shows no row; M4) |
| `$T`, `''`, outside | R6 | yes |
| `$T/` | R6 | yes (plant C shows no row; M4) |
| `$T//` | **pass, deletes `T`** | M2 |
| `$T/..`, `$T/x/..`, `$T/x/../y` | R5 | yes |
| `$T/.` | passes; GNU `rm` refuses `.`, exit 1, nothing deleted | fails closed |
| symlink below `T` to outside | the link only | yes |

fake-gh (R7): the input is always `mktemp -d "$T/merge.XXXXXX"`, so the refusal is unreachable (M5).

No cell is left for the builder: every one follows from the document's exact code. The two wrong cells (I1, M2) are wrong as decided, not undecided.

## Author's flagged points (judged, not re-reported)

1 (trailer): process only, nothing to judge. 2 (renumbering 71-77 / 78-85, 77/85 totals): agree; spec rows 1-85 run contiguously and no stale 70/78 count remains outside history. 3 (empty cache means the default): agree, it is the pre-fix behaviour and passes R1. 4 (the extra fake-gh delete): agree it is guarded; see M5 on its reachability. 5 (row 76's `guard.XXXXXX` outside `T`): acceptable; the row removes it, and a crashed run leaves an empty directory plus `keep`, never a delete outside. 6 (symlinked `cut-release-test`): agree, Minor (two conditions; fails closed; message names `T`); N2 is the spec side. 7 (pre-fix RED reaches `/ms-cli-v0.20.2`): agree, and I1 shows the same reach survives the fix for `/.` and similar values. 8 (unreachable tag guards, R3 text for an empty tag): agree. 9 (stale run directories): nothing to judge. The check order in `main` (parse_args, date, cache, `need_tools`) is right: both `verify_tag` call sites (`push_flow` after the tag push, `verify_main` after `parse_tag`/`row_by_prefix`) come after it.

## Machine checks run (scratch worktree of 85b72a97 plus the section 5 applier; sandbox `XDG_CACHE_HOME` with a space)

- The whole offline gate on the document as written, with T containing a space: `cut-release.test: 77 passed, 0 failed`, exit 0, T removed.
- The whole offline gate with every fix in this report applied (I1, I2, the M4 additions): `cut-release.test: 77 passed, 0 failed`, exit 0, T removed.
- `ci/code-docs-check.sh`: ok (166 files).
- Plants (seven rows each): A (I2) 7/0 survives; B, C, D 7/0 survive and become 6/1 with the M4 additions; E 7/0 survives (M5).
- Probes: R1 values (I1), tag values, a symlinked tag directory and root (probe 6), `$T//`, `T` with `..`, a main-shell refusal with `KEEP` 0 and 1, the row 77 regex on 9 spellings.
- Not run: the network rows 78-85.

## Appendix: executed fix texts for Minors (optional for the fold)

M4: H7 `cr_harness_rm_guards`. Replace `    for p in "$out" "$run" ''; do` with

    mkdir -p "$bad/run-1"
    ( T=$bad/run-1; rm_T ) >"$OUT" 2>"$ERR"; RC=$?
    x_rc 2; x_err "cut-release.test: refusing to delete '$bad/run-1': T is not */cut-release-test/run-*"
    [ -d "$bad/run-1" ] || rowfail "rm_T deleted a run-* directory that is not below cut-release-test"
    rmdir -- "$bad/run-1"
    for p in "$out" "$run" "$run/" ''; do

and insert before `    [ -d "$out" ] && [ -d "$run/sub" ] || rowfail "a refused rm_below_T deleted something"`:

    ( T=$run; rm_below_T "$run/sub" "$out" ) >"$OUT" 2>"$ERR"; RC=$?
    x_rc 2; x_err "cut-release.test: refusing to delete '$out': it is not strictly below $run"

S6 row 76 gains: "`rm_T` with `T` = `<scratch>/run-1` (made, then removed with `rmdir`): 2, the R4 line, `run-1` survives; `rm_below_T` of `run-1/` too: 2, the R6 line; `rm_below_T run-1/sub outside`: 2, the R6 line for `outside`, and `sub` survives". Run: plants B, C, D each give `6 passed, 1 failed` (`FAIL cr_harness_rm_guards`: B `rm_T deleted a run-* directory that is not below cut-release-test`; C `stderr lacks line ... run-1/': it is not strictly below ...`; D `a refused rm_below_T deleted something`). The document plus these lines passes, and so does the whole gate (77/0).

M7: H2, replace `rm_refuse() { echo "cut-release.test: refusing to delete '$1': $2" >&2; exit 2; }` with `rm_refuse() { KEEP=1; echo "cut-release.test: refusing to delete '$1': $2" >&2; exit 2; }`. S3 then adds after "exits 2": "(the run directory is kept and named)". Run: a main-shell refusal prints `cut-release.test: kept <T>` and T survives. With this line the seven rows still give `7 passed, 0 failed` (row 76's refusals run in subshells, so `KEEP` does not leak out).

## Design (ungraded)

Making `T` end in exactly `/cut-release-test/run-<digits>` (`[[ $T =~ /cut-release-test/run-[0-9]+$ ]]`) closes M3 and the plant-B class, and row 76 then needs no directory outside `T`. It changes the pattern the brief decided, so it is a question for the architect.

r1: 0C/2I/7M/2N
