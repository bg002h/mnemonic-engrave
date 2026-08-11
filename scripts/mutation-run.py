#!/usr/bin/env python3
"""
mutation-run.py -- run every SPEC S11.3 mutation-table row from Plan B Phase
B2b's implementation plan against the seedhammer-b2b fork worktree, and
report KILLED/SURVIVED/NOT APPLIED for each. This is F-96's runner (Task 7).

WHY THIS EXISTS
  B2a-ii ran ~50 mutants by hand for want of exactly this. CLAUDE.md's
  standing rule: "when an artifact will be folded repeatedly, commit the
  extractor as a script so the check is a command, not a thing to remember."

THE CENTRAL DESIGN DECISION: DERIVE THE ROWS, DO NOT TRANSCRIBE THEM
  scripts/plan-mutation-anchors.py ("pma" below) already parses the plan's
  mutation tables -- | `file.go` | `anchor` | replace with | must be killed
  by | -- including the file-anchor convention that binds ```go blocks to
  paths. This script IMPORTS that module and reuses its row scanner
  (pma.rows) and its anchor regex (pma.CODE_SPAN) rather than hand-copying
  the table into a second list. The only cells pma.rows does not return
  (the replacement and "killed by" text) are recovered by re-splitting the
  SAME physical line pma.rows already identified -- so the table is parsed
  exactly once, by one piece of code.
  A hand-copied command has already broken this feature's green criterion
  twice, and a hand-copied row count was wrong once. The plan's tables are
  the single source of truth; this script reads them, it does not restate
  them.

TWO THINGS A MACHINE GENUINELY CANNOT DERIVE FROM THE TABLE, AND HOW THIS
SCRIPT SUPPLIES THEM WITHOUT GUESSING
  1. WHICH GO TEST to run. The "must be killed by" cell is PROSE ("the
     restart test -- `"SESSION 2"` never drawn"), not a -run regex. TESTS_BY_LINE
     below supplies the mapping, and every entry is cited to the actual test
     function's own doc comment in the fork, which independently states
     which row it kills -- so the mapping is CHECKABLE by a reviewer without
     re-deriving it (grep the cited comment).
  2. WHETHER A "replace with" CELL IS A LITERAL, MECHANICAL EDIT. Most cells
     are -- a bare code span that is a complete drop-in for the anchor
     (`classify_replacement`'s "literal" case), or the explicit "*delete the
     line*" marker, or the explicit "`X` in place of `Y`" phrasing (used
     once, for the A-C1 buffer row). Two cells are NOT: "revert the whole
     callback to `...`" names a replacement for MORE than the anchor line
     (the guard above it must also go, and the anchor cell does not name
     that line), and "hoist above `for {`" is a code MOVE to a location
     `for {` cannot even name uniquely (it occurs 3x in run_flow.go). Both
     are classified NOT_MECHANICAL structurally (the cell is neither the
     delete marker, the "in place of" pattern, nor a bare code span) and
     printed as NOT APPLIED with the reason -- never guessed at.

SS11.3's TWO PROCEDURAL RULES, NORMATIVE, ENFORCED HERE (not left as
discipline):
  0. A mutant that fails to BUILD is KILLED, not an error. Go's own
     "declared and not used" catches at least one row here (`if !wiping {`
     -> `if false {`) -- classifying that as inconclusive would show a real
     kill as a coverage gap.
  1. Assert the substitution matched, EXACTLY ONCE, before it is applied. A
     match count != 1 is a hard failure, never a mutant result -- "a
     silently-failing sed reads exactly like a surviving mutation."
  2. Restore from a file COPY, never `git checkout` (which would discard any
     uncommitted work in the worktree that has nothing to do with this
     script).

WHAT THIS SCRIPT DOES NOT COVER -- state this in any review brief
  - The two NOT_MECHANICAL rows above: applying them is a reviewer's
    execution pass. (Row "revert the whole callback" -- the Critical -- was
    verified killable BY HAND during Task 3, per the plan's own text: "before
    the fix, every one of them failed." That verification predates this
    script and is not repeated by it.)
  - Whether a replacement is a real mutation rather than an equivalent
    implementation: a semantically identical substitution would survive
    every run and this script has no way to tell the difference from a real
    coverage gap.
  - Collateral damage. Every per-row run is FILTERED with -run to the named
    test(s); a filtered run proves that test fails, and nothing about any
    other test. Only the final unfiltered suite (step 7.0) can see collateral
    damage, and it is not optional for exactly that reason.
  - Whether the TESTS_BY_LINE mapping itself is correct. It is hand-supplied
    (see point 1 above) and cited to test-file comments, but this script
    cannot verify a cited comment says what this file's own comment claims
    it says -- that is a reviewer's job, and the citations are there to make
    it a five-second grep rather than a re-derivation.

Usage:  scripts/mutation-run.py [plan.md] [worktree]
  plan.md    default: design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md
  worktree   default: /scratch/code/shibboleth/seedhammer-b2b (the fork
             worktree Task 7 mutates and restores; NEVER the plain
             /scratch/code/shibboleth/seedhammer checkout, which other work
             depends on and which this script never touches).
"""
import atexit
import filecmp
import importlib.util
import json
import os
import pathlib
import signal
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
DEFAULT_PLAN = SCRIPT_DIR.parent / "design" / "IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md"
DEFAULT_WORKTREE = pathlib.Path("/scratch/code/shibboleth/seedhammer-b2b")
# The devshell comes from the ORIGINAL seedhammer checkout's flake.nix -- the
# worktree carries its own copy of flake.nix too (git worktrees check out
# every tracked file), but the plan's own "Running things" convention and
# plan-build-gate-go.sh both key off this path, so this script matches it
# rather than introducing a second convention.
FLAKE_DIR = pathlib.Path("/scratch/code/shibboleth/seedhammer")
NIX = "/nix/var/nix/profiles/default/bin/nix"

# Load plan-mutation-anchors.py as a module and reuse its row parser and
# CODE_SPAN regex -- see the module docstring's "central design decision".
_pma_spec = importlib.util.spec_from_file_location("plan_mutation_anchors", SCRIPT_DIR / "plan-mutation-anchors.py")
pma = importlib.util.module_from_spec(_pma_spec)
_pma_spec.loader.exec_module(pma)

# ---------------------------------------------------------------------------
# The one piece of data this script supplies that the table's prose cannot:
# which Go test(s) a row's "must be killed by" cell is actually naming.
#
# Keyed by the PLAN LINE NUMBER pma.rows() reports for that row -- not by
# row content, so a citation here breaks loudly (KeyError) rather than
# silently if the plan is re-edited and the row moves or its anchor changes,
# instead of quietly mis-pairing a stale entry with a new row.
#
# Every list is cited to the fork test file whose OWN doc comment
# independently states it kills that row -- grep the citation to check this
# script's claim without re-deriving it.
# ---------------------------------------------------------------------------
TESTS_BY_LINE = {
    617: ["TestWipeGuardLifecycleAndArmed"],
    # gui/wipe_guard_test.go:38 doc: "the guard is uninstalled... nil again
    # once it returns."
    618: ["TestWipeGuardLifecycleAndArmed", "TestRunPostCutWindowRestartsFromCutEnd"],
    # gui/wipe_guard_test.go:38 doc pins armed() during a running job (this
    # row's own case). The plan's cell also names Task 4.1's post-cut test;
    # included for fidelity to the cell, though wipe_guard_test.go alone
    # already discriminates the mutant (see this script's own verification
    # note in the header of run_row()).
    712: ["TestRunWipeUnwindsAndRestartsTheFlow"],
    # gui/run_flow_test.go:180 doc: "Kills the `break`->`return` mutant (3.3
    # row 1)".
    # 713 (revert the whole callback): NOT_MECHANICAL, see below.
    714: [],
    # "the compiler, not a test" (plan's own words) -- no -run target; a
    # BUILD FAILURE is the expected kill, asserted specially in run_row().
    # 715 (hoist `wiping` above `for {`): NOT_MECHANICAL, see below.
    1141: ["TestRunNotArmedNeverWarns"],
    # gui/run_flow_test.go:440 doc: "Kills 4.3's `armed := ctx.wipe.armed()`
    # -> `armed := true` row."
    1142: ["TestRunWarningThenWipe"],
    # gui/run_flow_test.go:316 doc: "Kills 4.3's `armed := ctx.wipe.armed()`
    # -> `armed := false` row."
    1143: ["TestRunPostCutWindowRestartsFromCutEnd"],
    # gui/run_flow_test.go:506 doc: "Kills 4.3's `a.idle.start = now // row
    # 2...` deletion row."
    1144: ["TestRunWarningThenWipe"],
    # gui/run_flow_test.go:320 doc: "Kills 4.3's `if armed {` -> `if false {`
    # row the same way."
    1145: ["TestRunWarningCountdownIsReal"],
    # gui/run_flow_test.go:346 doc: "Kills 4.3's `const wipeWarningDelay =
    # 30 * time.Second` -> `= 0` row."
    1146: ["TestWipeWarningOpClampsNegativeRemaining"],
    # gui/run_flow_test.go:624 doc: "kills 4.3's `if secs < 0 {` -> `if false
    # {` row" -- a direct unit call, per the plan, since Run itself can never
    # reach a negative `remaining`.
    1147: ["TestRunWarningBufferDoesNotGrow"],
    # gui/run_flow_test.go:589 doc: "Kills 4.3's `a.warnBuf.Reset()` deletion
    # row."
    1148: ["TestRunWarningBufferDoesNotGrow"],
    # gui/run_flow_test.go:593 doc: "Kills 4.3's `&a.warnBuf` -> `&ctx.B` row
    # (A-C1 itself)."
    1149: ["TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver"],
    # gui/run_flow_test.go:662 doc: "the `ctx.keepAwake` -> `false` mutant
    # (4.3's row, attributed to this step)".
    1150: ["TestRunKeepAwakeCannotPostponeAnArmedWipe"],
    # gui/run_flow_test.go:698 doc: "Kills 4.3's `(ctx.keepAwake && !armed)`
    # -> `(ctx.keepAwake)` row."
    1303: ["TestUnlockEngraveMnemonicZeroesMOnConfirmDiscard", "TestUnlockEngraveMnemonicZeroesMOnEngraveSeedError"],
    # The plan's own cell says "all three of 6.1's tests", but step 6.1
    # explicitly says "TWO, not three" -- the third early return
    # (masterFingerprintFor's error arm) is recorded as cryptographically
    # unreachable, not tested. gui/unlock_session_test.go's
    # TestUnlockEngraveMnemonicZeroesMOnEngraveSeedError doc comment calls
    # this out directly: "the plan's table says 'all three of 6.1's tests'
    # must kill it, and this is the engraveSeed-specific instance." Both
    # existing tests are run; there is no third.
    1304: ["TestUnlockEngraveMnemonicZeroesMOnConfirmDiscard", "TestUnlockEngraveMnemonicZeroesMOnEngraveSeedError"],
    # Both tests use watchMnemonicParsed + assertMnemonicZeroed
    # (gui/unlock_session_test.go:744), whose own comment names it "step
    # 6.1's mandatory guard... the hook must have fired" -- the fired-guard
    # this row targets.
}

# Rows the plan itself scopes to a build failure, not a test -- see
# TESTS_BY_LINE[714]'s comment. Any OTHER row whose mechanical application
# also happens to produce a build failure gets flagged as a DIVERGENCE from
# its own "killed by" cell (still classified KILLED per rule 0 -- a build
# failure is always a kill -- but printed loudly rather than silently folded
# in, since it means the mechanical literal edit did not produce the
# scenario the cell's prose describes).
EXPECT_BUILD_FAILURE = {714}

NOT_MECHANICAL = {
    713: (
        "the cell reads \"revert the whole callback to `ctx.Done = ctx.Done || "
        "!yield(o)`\" -- not a drop-in for the anchor line alone. The guard "
        "`if ctx.Done { return }` immediately above the anchor must ALSO be "
        "removed for this to be the described mutant; the anchor cell does not "
        "name that line, and a literal single-line substitution at the anchor "
        "produces different (and broken) code, not the mutant. THIS IS THE "
        "CRITICAL Task 3 found and fixed. It is not unverified: the plan "
        "records it as checked BY HAND while the fix was being written "
        "('Verified killable: before the fix, every one of them failed'), and "
        "gui/run_flow_test.go:184's doc comment states the same for "
        "TestRunWipeUnwindsAndRestartsTheFlow. Not re-applied here -- printed "
        "as NOT APPLIED rather than silently skipped, per this script's own "
        "rule against guessing a rewrite."
    ),
    715: (
        "the cell reads \"hoist above `for {`\" -- a structural MOVE (declare "
        "`wiping` above the loop; delete the inline `:=` in its old spot), not "
        "a substitution at the anchor. `for {` is not even a usable anchor for "
        "the destination: it occurs 3x in gui/run_flow.go (the outer session "
        "loop this row means, plus two inner loops), so the target line cannot "
        "be named uniquely by the cell's own text. Not applied."
    ),
}

# ---------------------------------------------------------------------------
# Table parsing -- reuses pma.rows()/pma.CODE_SPAN; only re-splits the same
# line pma already located, to recover the two cells pma.rows() does not
# return (replacement, killed-by). No row content is retyped.
# ---------------------------------------------------------------------------


def load_rows(plan_text):
    lines = plan_text.split("\n")
    out = []
    for lineno, fname, _anchor_cell in pma.rows(plan_text):
        raw = lines[lineno - 1]
        cells = [c.strip() for c in raw.strip().strip("|").split("|")]
        spans = pma.CODE_SPAN.findall(cells[1])
        if len(spans) != 1:
            sys.exit(f"line {lineno}: anchor cell does not have exactly one code span "
                      f"({len(spans)} found) -- plan-mutation-anchors.py should already "
                      f"reject this plan; run it first")
        out.append({
            "lineno": lineno,
            "file": fname,
            "anchor": spans[0],
            "replace_cell": cells[2],
            "killed_by_cell": cells[3],
        })
    return out


_SUBREPLACE_RE = re.compile(r"^`([^`]*)`\s+in place of\s+`([^`]*)`$")


def classify_replacement(cell):
    """Return (mode, *args) for a "-> replace with" cell.

    mode is one of:
      "delete"        -- remove the anchor's whole physical line
      "sub", new, old  -- replace the substring `old` (found within/near the
                          anchor) with `new`; used only where the cell says
                          so explicitly ("X in place of Y")
      "literal", new   -- replace the WHOLE anchor with `new`; used when the
                          cell is nothing but a single bare code span, i.e.
                          a complete drop-in for the anchor
      "not_mechanical" -- anything else: prose describing an edit broader
                          than, or differently scoped than, the anchor
    """
    s = cell.strip()
    if s == "*delete the line*":
        return ("delete",)
    m = _SUBREPLACE_RE.match(s)
    if m:
        return ("sub", m.group(1), m.group(2))
    spans = pma.CODE_SPAN.findall(s)
    if len(spans) == 1 and s == f"`{spans[0]}`":
        return ("literal", spans[0])
    return ("not_mechanical",)


# ---------------------------------------------------------------------------
# Mutation application -- restore is ALWAYS from a file copy (cp), never
# `git checkout`, per SS11.3 rule 2: git checkout would discard any
# uncommitted work in the worktree unrelated to this script.
# ---------------------------------------------------------------------------


class MatchCountError(Exception):
    """A substitution's target did not occur exactly once. Rule 1: this is a
    hard failure, never a mutant result."""


def apply_mutation(path, mode_tuple, anchor):
    """Mutate `path` in place per mode_tuple. Returns nothing; raises
    MatchCountError on any match count != 1."""
    content = path.read_text()
    if content.count(anchor) != 1:
        raise MatchCountError(f"anchor occurs {content.count(anchor)} times in {path.name}, want 1")

    mode = mode_tuple[0]
    if mode == "delete":
        lines = content.splitlines(keepends=True)
        hit = [i for i, l in enumerate(lines) if anchor in l]
        if len(hit) != 1:
            raise MatchCountError(f"anchor is on {len(hit)} physical lines in {path.name}, want 1")
        del lines[hit[0]]
        path.write_text("".join(lines))
    elif mode == "literal":
        _, new = mode_tuple
        path.write_text(content.replace(anchor, new, 1))
    elif mode == "sub":
        _, new, old = mode_tuple
        if content.count(old) != 1:
            raise MatchCountError(f"substitution target {old!r} occurs {content.count(old)} times in {path.name}, want 1")
        path.write_text(content.replace(old, new, 1))
    else:
        raise ValueError(f"apply_mutation called with non-mechanical mode {mode!r}")


def nix_go_test(worktree, test_names, timeout_s=300):
    """Run `go test -run <regex> -timeout <t>s ./gui/` under the shared
    devshell, cwd = worktree. test_names=[] runs with -run '^$' -- no test
    executes, but the package (and its test binary) still has to BUILD, which
    is exactly what row 714 depends on."""
    run_arg = "^(" + "|".join(re.escape(n) for n in test_names) + ")$" if test_names else "^$"
    inner = f"cd {shlex.quote(str(worktree))} && CGO_ENABLED=0 go test -run {shlex.quote(run_arg)} -timeout {timeout_s}s ./gui/"
    try:
        return subprocess.run(
            [NIX, "develop", str(FLAKE_DIR), "--command", "bash", "-c", inner],
            capture_output=True, text=True, timeout=timeout_s + 60,
        )
    except subprocess.TimeoutExpired as e:
        class _Fake:
            pass
        f = _Fake()
        f.returncode = 124
        f.stdout = (e.stdout or b"").decode() if isinstance(e.stdout, bytes) else (e.stdout or "")
        f.stderr = f"TIMEOUT after {timeout_s + 60}s -- go test's own -timeout should have fired first; " \
                   f"this is the outer safety net, and hitting it is itself worth investigating"
        return f


def classify_go_test_result(proc):
    """-> ("KILLED_BUILD" | "KILLED_TEST" | "SURVIVED" | "INCONCLUSIVE", detail)."""
    out = proc.stdout + proc.stderr
    if "[build failed]" in out or re.search(r"^# \S", out, re.M):
        return ("KILLED_BUILD", out.strip())
    if proc.returncode == 0:
        if "no tests to run" in out:
            return ("INCONCLUSIVE", "go test ran but matched no tests -- TESTS_BY_LINE names a test "
                                     "that does not exist; this is a bug in this script's mapping, not "
                                     "a mutation result:\n" + out.strip())
        return ("SURVIVED", out.strip())
    if "--- FAIL:" in out or proc.returncode != 0:
        return ("KILLED_TEST", out.strip())
    return ("INCONCLUSIVE", out.strip())


def run_row(row, idx, total, worktree):
    lineno = row["lineno"]
    path = worktree / "gui" / row["file"]
    mode_tuple = classify_replacement(row["replace_cell"])
    header = f"-- row {idx}/{total} (plan line {lineno})  {row['file']}: {row['anchor'][:70]!r}"
    print(header)

    if mode_tuple[0] == "not_mechanical":
        reason = NOT_MECHANICAL.get(lineno, row["replace_cell"])
        print(f"   NOT APPLIED -- {reason}")
        return {"lineno": lineno, "status": "NOT_APPLIED", "detail": reason}

    if lineno not in TESTS_BY_LINE:
        return {"lineno": lineno, "status": "ERROR", "detail": "no TESTS_BY_LINE entry -- script bug"}
    test_names = TESTS_BY_LINE[lineno]

    backup_dir = pathlib.Path(tempfile.mkdtemp(prefix="mutation-run-"))
    backup = backup_dir / path.name
    shutil.copy2(path, backup)  # the file-copy restore point; never `git checkout`
    _mark_inflight(path, backup)  # F-101: on disk, so a KILL is recoverable
    try:
        apply_mutation(path, mode_tuple, row["anchor"])
        print(f"   applied ({mode_tuple[0]}); running: "
              f"{'go test -run ' + repr('|'.join(test_names)) if test_names else 'go build via `go test -run ^$` (row names the compiler, not a test)'}")
        proc = nix_go_test(worktree, test_names)
        status, detail = classify_go_test_result(proc)
        if status == "SURVIVED":
            print(f"   *** SURVIVED *** -- BLOCKING. output:\n{_indent(detail)}")
        elif status == "INCONCLUSIVE":
            print(f"   *** INCONCLUSIVE *** -- BLOCKING (this is a runner bug, not a mutation result). output:\n{_indent(detail)}")
        else:
            kind = "build failure" if status == "KILLED_BUILD" else "test failure"
            expected_build = lineno in EXPECT_BUILD_FAILURE
            note = ""
            if status == "KILLED_BUILD" and not expected_build:
                note = ("  <-- DIVERGENCE: this row's own cell names a runtime test "
                         f"({test_names}), not the compiler, but the literal mechanical "
                         "edit produced a build failure instead. Still a KILL per rule 0, "
                         "but flagged: see this script's header note on rows shorter than "
                         "their anchor.")
            elif status == "KILLED_TEST" and expected_build:
                note = "  <-- unexpected: this row is scoped to a compiler kill but a test ran and failed instead"
            print(f"   KILLED ({kind}){note}")
        return {"lineno": lineno, "status": status, "detail": detail}
    except MatchCountError as e:
        print(f"   *** MATCH COUNT FAILURE *** -- BLOCKING (rule 1). {e}")
        return {"lineno": lineno, "status": "MATCH_COUNT_FAILURE", "detail": str(e)}
    finally:
        shutil.copy2(backup, path)  # restore from the copy, never git checkout
        if not filecmp.cmp(path, backup, shallow=False):
            print(f"   !! restore verification FAILED for {path} -- worktree may be left dirty !!")
        _clear_inflight()  # only after the restore has been verified above
        shutil.rmtree(backup_dir, ignore_errors=True)


def _indent(s, prefix="      "):
    return "\n".join(prefix + l for l in s.splitlines())


# F-101: CRASH SAFETY.
#
# run_row's try/finally restores the file, and that is enough for an exception.
# It is NOT enough for a KILL: SIGKILL cannot be caught at all, and SIGTERM's
# default action terminates the process without unwinding, so `finally` never
# runs. Measured 2026-08-09 -- three times in ten minutes a backgrounded run was
# killed mid-row and left `armed := true` applied in gui/run_flow.go, the mutant
# that makes §10.2.4's wipe timer permanently armed.
#
# What makes that worse than an annoyance: the file left behind COMPILES and
# passes most of the suite. A `git add` at the wrong moment commits a
# funds-safety guard that has been disabled, as one plausible line inside a
# function nobody re-reads.
#
# Two mechanisms, because one cannot cover both cases:
#
#   SENTINEL   a JSON file written BEFORE the mutation is applied and removed
#              after the restore verifies. It survives SIGKILL, a power cut and
#              a reboot, because it is on disk rather than in this process. The
#              next run finds it and restores from the backup automatically.
#   HANDLERS   for the signals that CAN be caught (INT, TERM, HUP), restore
#              immediately rather than making the operator run a recovery.
#
# The sentinel is the load-bearing half. Handlers are the courtesy.

SENTINEL = pathlib.Path(tempfile.gettempdir()) / "mutation-run-inflight.json"
_inflight = {"path": None, "backup": None}


def _restore_inflight(reason):
    """Restore the mutated file if one is applied. Safe to call twice."""
    path, backup = _inflight.get("path"), _inflight.get("backup")
    if not path or not backup:
        return False
    p, b = pathlib.Path(path), pathlib.Path(backup)
    if not b.exists():
        print(f"   !! {reason}: backup {b} is GONE -- {p} may still hold a mutant !!")
        return False
    shutil.copy2(b, p)
    ok = filecmp.cmp(p, b, shallow=False)
    print(f"   restored {p.name} after {reason}" if ok
          else f"   !! restore verification FAILED for {p} after {reason} !!")
    _inflight["path"] = _inflight["backup"] = None
    SENTINEL.unlink(missing_ok=True)
    return ok


def _mark_inflight(path, backup):
    _inflight["path"], _inflight["backup"] = str(path), str(backup)
    SENTINEL.write_text(json.dumps(_inflight))


def _clear_inflight():
    _inflight["path"] = _inflight["backup"] = None
    SENTINEL.unlink(missing_ok=True)


def _install_signal_handlers():
    def handler(signum, _frame):
        name = signal.Signals(signum).name
        print(f"\n!! caught {name} mid-row -- restoring before exit")
        _restore_inflight(name)
        # Re-raise with the default action so the exit status is honest.
        signal.signal(signum, signal.SIG_DFL)
        os.kill(os.getpid(), signum)
    for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        try:
            signal.signal(sig, handler)
        except (ValueError, OSError):
            pass  # not on the main thread, or not supported here
    atexit.register(lambda: _restore_inflight("interpreter exit"))


def recover_sentinel():
    """Restore a mutant left by a previous run that was killed. Runs BEFORE
    preflight_clean, because the mutant it recovers is exactly what would make
    that check refuse to start."""
    if not SENTINEL.exists():
        return
    try:
        st = json.loads(SENTINEL.read_text())
    except (OSError, ValueError) as e:
        sys.exit(f"sentinel {SENTINEL} exists but is unreadable ({e}); a previous run may have "
                 f"left a MUTANT applied. Inspect the worktree by hand before continuing.")
    path, backup = st.get("path"), st.get("backup")
    print(f"!! a previous run was killed with a mutation applied to {path}")
    if not backup or not pathlib.Path(backup).exists():
        sys.exit(f"   its backup ({backup}) is gone, so this script cannot restore it.\n"
                 f"   Restore {path} from git by hand, verify the diff is EMPTY, then delete "
                 f"{SENTINEL}.")
    _inflight["path"], _inflight["backup"] = path, backup
    if not _restore_inflight("a previous run being killed"):
        sys.exit("   automatic restore failed; fix by hand before running mutations.")


def preflight_clean(worktree):
    r = subprocess.run(["git", "-C", str(worktree), "status", "--short"], capture_output=True, text=True)
    if r.stdout.strip():
        sys.exit(f"worktree {worktree} is not clean before this script has touched anything; refusing to "
                 f"start (a mutation's restore would be indistinguishable from a pre-existing change):\n{r.stdout}")


def final_unfiltered_suite(worktree):
    # Step 7.0: "one unfiltered go test ./gui/ ./gui/op/ ./seal/ at the end,
    # before the commit" -- this also runs ./bip39/, a harmless superset (it
    # is part of the feature's overall green criterion elsewhere and no row
    # here touches it), so this single command demonstrates both the plan's
    # named packages and the fuller cross-check.
    pkgs = ["./gui/", "./gui/op/", "./seal/", "./bip39/"]
    inner = f"cd {shlex.quote(str(worktree))} && CGO_ENABLED=0 go test {' '.join(pkgs)}"
    proc = subprocess.run([NIX, "develop", str(FLAKE_DIR), "--command", "bash", "-c", inner],
                           capture_output=True, text=True, timeout=600)
    return proc


def main(argv):
    plan_path = pathlib.Path(argv[1]) if len(argv) > 1 else DEFAULT_PLAN
    worktree = pathlib.Path(argv[2]) if len(argv) > 2 else DEFAULT_WORKTREE
    if not plan_path.is_file():
        sys.exit(f"no plan at {plan_path}")
    if not (worktree / ".git").exists():
        sys.exit(f"no worktree at {worktree}")

    # F-101: order matters. recover_sentinel BEFORE preflight_clean, because the
    # mutant a previous kill left behind is exactly what makes preflight refuse
    # to start -- and refusing with "worktree is dirty" tells the operator
    # nothing about which line is a live mutant.
    _install_signal_handlers()
    recover_sentinel()
    preflight_clean(worktree)

    rows = load_rows(plan_path.read_text())
    print(f"== parsed {len(rows)} mutation-table rows from {plan_path} (via plan-mutation-anchors.py's own scanner) ==\n")

    results = []
    for i, row in enumerate(rows, 1):
        results.append(run_row(row, i, len(rows), worktree))
        print()

    print("== per-row summary ==")
    blocking = []
    for r in results:
        print(f"   line {r['lineno']:5d}  {r['status']}")
        if r["status"] in ("SURVIVED", "INCONCLUSIVE", "MATCH_COUNT_FAILURE", "ERROR"):
            blocking.append(r)

    print()
    print("== step 7.0's final unfiltered suite: go test ./gui/ ./gui/op/ ./seal/ ./bip39/ ==")
    suite = final_unfiltered_suite(worktree)
    suite_ok = suite.returncode == 0
    print(_indent((suite.stdout + suite.stderr).strip()))
    print(f"   {'PASS' if suite_ok else 'FAIL'} (exit {suite.returncode})")

    print()
    print("== worktree cleanliness ==")
    status = subprocess.run(["git", "-C", str(worktree), "status", "--short"], capture_output=True, text=True)
    clean = not status.stdout.strip()
    print("   clean" if clean else f"   DIRTY:\n{status.stdout}")

    print()
    print("== NOT APPLIED (not mechanically expressible; a reviewer's execution pass) ==")
    for lineno, reason in NOT_MECHANICAL.items():
        print(f"   line {lineno}: {reason}")
    print()
    print("== step 5.2 -- not a table row at all ==")
    print("   \"swap the read of ctx.keepAwake past ctx.Reset()\" is a statement REORDERING, not a")
    print("   substitution, so no anchor can express it (the plan says so explicitly). It stays a")
    print("   hand-run ordering check and is not attempted here.")

    print()
    print("== this script's blind spots (same shape as plan-build-gate-go.sh / plan-mutation-anchors.py) ==")
    print("   - The two NOT_MECHANICAL rows above were not applied; their mutants are unverified BY")
    print("     THIS SCRIPT (one -- the FrameCallback Critical -- was verified by hand during Task 3;")
    print("     see NOT_MECHANICAL[713]).")
    print("   - Cannot tell a real mutation from an equivalent implementation: a substitution that")
    print("     changes nothing observable would SURVIVE and this script would report it exactly like")
    print("     a genuine coverage gap.")
    print("   - Per-row runs are FILTERED (-run); a filtered run cannot see collateral damage. Only the")
    print("     final unfiltered suite above checks that.")
    print("   - TESTS_BY_LINE is hand-supplied, not derived from the table's prose (which a machine")
    print("     cannot turn into a Go identifier). Each entry cites the fork test file's own doc")
    print("     comment; that citation is not verified by this script and is a reviewer's five-second")
    print("     grep, not a re-derivation.")

    ok = clean and suite_ok and not blocking
    print()
    print("== RESULT ==")
    if ok:
        print("   every mechanically-applicable row KILLED its mutant; final unfiltered suite PASS;")
        print("   worktree clean")
        return 0
    print("   FAIL:")
    if blocking:
        print(f"     {len(blocking)} blocking row(s): {[r['lineno'] for r in blocking]}")
    if not suite_ok:
        print("     final unfiltered suite is RED")
    if not clean:
        print("     worktree is not clean after restore")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
