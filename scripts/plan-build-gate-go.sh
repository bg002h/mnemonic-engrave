#!/usr/bin/env bash
#
# plan-build-gate-go.sh -- compile the Go that lives inside an implementation
# plan, so a fold that does not build never reaches a reviewer.
#
# WHY THIS EXISTS (F-74)
#   plan-build-gate.sh extracts ```rust and builds it. Plan B's documents carry
#   GO, so every Go fragment in the B1 plan reached three R0 reviewers and one
#   whole-diff reviewer UNCOMPILED. The B1 plan said so in its own gate-coverage
#   section rather than letting the brief imply coverage it did not have -- but
#   saying it is not checking it.
#
#   The exposure is measured, not theoretical. Continuity §4 recorded folds as
#   the dominant defect source across this feature, and compile errors as a
#   recurring class among them: round 3 of the Plan A review burned an entire
#   opus round on FIVE of them, each one a `cargo build` away. Plan B had the
#   same exposure with none of the coverage.
#
# THE TWO TIERS, AND WHY THERE ARE TWO
#   Go plans are mostly MODIFICATIONS -- a new `case` arm, a struct field, a
#   function to insert into an existing file. Those cannot be assembled
#   mechanically, because the surrounding file is not in the plan. So:
#
#   TIER 1 -- FULL TYPE CHECK, and it GATES.
#     ```go blocks anchored to a NEW file ("create `gui/unlock_flow.go`") are
#     assembled into a scratch copy of the fork and run through `go build` and
#     `go vet`. This is a real compile against the real packages: undefined
#     identifiers, wrong types, bad signatures and unused imports all fail here.
#
#   TIER 2 -- PARSE ONLY, and it is INFORMATIONAL.
#     Every other ```go block is offered to `gofmt -e` under three wrappers
#     (file scope, function body, struct body). If one parses, the block is at
#     least syntactically Go. If none do, that is USUALLY FINE -- a bare `case`
#     arm legitimately parses under none of them -- so it is reported, never
#     failed on.
#
# WHAT IT DOES NOT DO -- state this in the review brief
#   Tier 2 proves SYNTAX, never SEMANTICS. A fragment that names a function that
#   does not exist, passes the wrong type, or returns the wrong arity will parse
#   happily and is still a reviewer's execution pass. The script prints every
#   such block with its line number so the brief can name them instead of
#   implying they were checked.
#
#   A gate that hides its own blind spot is worse than no gate.
#
# Usage: scripts/plan-build-gate-go.sh <plan.md> [fork-root]
set -uo pipefail

PLAN="${1:?usage: plan-build-gate-go.sh <plan.md> [fork-root]}"
FORK="${2:-/scratch/code/shibboleth/seedhammer}"
WORK="${TMPDIR:-/tmp}/plan-build-gate-go"

[ -f "$PLAN" ] || { echo "no plan at $PLAN" >&2; exit 2; }
[ -d "$FORK" ] || { echo "no fork at $FORK" >&2; exit 2; }

# `nix` is not reliably on PATH -- it resolves in an interactive shell and not
# in the non-interactive one every agent and CI invocation runs. Same fix as
# sh2-flash: prefer PATH, fall back to the profile.
NIX="$(command -v nix 2>/dev/null || true)"
[ -n "$NIX" ] || NIX=/nix/var/nix/profiles/default/bin/nix
[ -x "$NIX" ] || { echo "no usable nix: not on PATH and $NIX is not executable" >&2; exit 2; }

fail=0

echo "== 1 -- scratch copy of the fork =="
rm -rf "$WORK"
mkdir -p "$WORK"
# Copy tracked files only: a worktree carries build artefacts and .git noise.
( cd "$FORK" && git ls-files -z | xargs -0 tar cf - ) | ( cd "$WORK" && tar xf - )
echo "   $WORK  ($(find "$WORK" -name '*.go' | wc -l | tr -d ' ') .go files)"

echo
echo "== 2 -- extract the plan's Go =="
python3 - "$PLAN" "$WORK" <<'PY'
import re, sys, os, json, collections

plan, work = sys.argv[1], sys.argv[2]
lines = open(plan).read().split("\n")

# An anchor names the file a following block belongs to. "create"/"new file"
# mean the block is a WHOLE FILE and can be assembled; anything else means the
# block is a fragment of a file that already exists.
# Two anchor shapes, because plans write both:
#   verb-first   "create `gui/unlock_flow.go`"      / "in `gui/gui.go`"
#   path-first   "`gui/unlock_flow.go`, new file."  / "`gui/gui.go` — add a case"
# Missing the path-first form is not hypothetical: it is how the B1 plan names
# EVERY new file, so the first version of this gate type-checked nothing at all
# while reporting success.
anchor_vf = re.compile(r'(create|new file|add to|insert into|in|modify)\s+`([^`]*\.go)`', re.I)
anchor_pf = re.compile(r'`([^`]*\.go)`[\s,—–-]*\(?(new file|new)\b', re.I)

whole = collections.OrderedDict()   # path -> [code, ...]
frags = []                          # (lineno, path-or-None, code)
cur, is_new = None, False
i = 0
while i < len(lines):
    # An anchor binds only until the next heading. Without this, `cur` leaks
    # across tasks and a later, unrelated block is assembled into the previous
    # task's file -- which surfaced as "non-declaration statement outside
    # function body" in a file the plan never claimed to be writing.
    if lines[i].startswith("#"):
        cur, is_new = None, False
    m = anchor_vf.search(lines[i])
    if m:
        verb, path = m.group(1).lower(), m.group(2)
        cur, is_new = path, verb in ("create", "new file")
    elif (m := anchor_pf.search(lines[i])):
        cur, is_new = m.group(1), True
    if lines[i].startswith("```go"):
        start = i + 1
        i += 1
        buf = []
        while i < len(lines) and not lines[i].startswith("```"):
            buf.append(lines[i]); i += 1
        code = "\n".join(buf)
        # A block anchored as a NEW FILE but carrying an elision marker is a
        # fragment wearing a whole-file label -- plans legitimately write `...`
        # for "and the rest". Type-checking it fails on the elision, which says
        # nothing about the plan. Demote it to TIER 2 and SAY SO, rather than
        # failing for a reason that is not a defect.
        elided = any(l.strip() in ("...", "…") for l in code.split("\n"))
        if cur and is_new and not elided:
            whole.setdefault(cur, []).append(code)
        else:
            if cur and is_new and elided:
                print(f"   DEMOTED {cur} block at line {start+1}: contains `...`, "
                      f"so it is NOT a whole file and is NOT type-checked")
            frags.append((start + 1, cur, code))
    i += 1

for p, parts in whole.items():
    dest = os.path.join(work, p)
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    body = "\n\n".join(parts) + "\n"
    # Plans omit the package clause -- they show the interesting code, not a
    # compilable file. Synthesise it from the path, or the file is "expected
    # 'package', found 'func'" and TIER 1 fails for a reason that says nothing
    # about the plan.
    if not re.match(r'\s*(//[^\n]*\n|/\*.*?\*/\s*)*\s*package\s', body, re.S):
        d = os.path.dirname(p)
        pkg = "main" if d.startswith("cmd/") else (os.path.basename(d) or "main")
        body = f"package {pkg}\n\n" + body
        print(f"           synthesised `package {pkg}`")
    with open(dest, "w") as f:
        f.write(body)
    print(f"   TIER 1  {p}  ({len(parts)} block(s), {sum(c.count(chr(10))+1 for c in parts)} lines)")

json.dump(frags, open(os.path.join(work, ".frags.json"), "w"))
# Record what TIER 1 actually wrote, so step 3 tests THAT rather than guessing
# from file mtimes. An earlier version guessed with `find -newer go.mod`, got it
# wrong, and printed "SKIP: no whole-file blocks were extracted" one line under
# a TIER 1 line saying it had extracted 34 -- extract-then-never-compile, the
# exact bug plan-build-gate.sh had with tests/seal_cli.rs.
with open(os.path.join(work, ".whole.txt"), "w") as f:
    f.write("\n".join(whole) + ("\n" if whole else ""))
print(f"   TIER 2  {len(frags)} fragment block(s) -- parse-only")
if not whole:
    print("   TIER 1  no whole-file blocks found; nothing to type-check")
PY

echo
echo "== 3 -- TIER 1: go build + go vet (GATES) =="
if [ -s "$WORK/.whole.txt" ]; then
    echo "   type-checking: $(tr '\n' ' ' < "$WORK/.whole.txt")"
    # Build ONLY the packages the plan touched. `./...` drags in cmd/kdfbench
    # and cmd/sealread, which are TinyGo-only (`machine` is not in host std) and
    # fail by design -- sanctioned noise that would mask a real failure.
    PKGS=$( sed 's|/[^/]*$||' "$WORK/.whole.txt" | sort -u | sed 's|^|./|' | tr '\n' ' ' )
    echo "   packages: $PKGS"
    out=$( cd "$WORK" && "$NIX" develop "$FORK" --command go build $PKGS 2>&1 )
    if [ -n "$out" ]; then
        echo "$out" | sed 's/^/   /'
        echo "   FAIL: the plan's whole-file Go does not build"
        fail=1
    else
        echo "   PASS: go build $PKGS clean"
        # BASELINE the vet run against the UNMODIFIED fork and report only what
        # the plan introduced. The repo carries pre-existing findings that have
        # nothing to do with any plan -- e.g. "testing.ArtifactDir requires
        # go1.26 or later (file is go1.25)" in gui/op/draw_test.go. A denylist of
        # those would rot the first time one is fixed or another appears; a
        # baseline diff cannot. Without this the gate FAILED ON CORRECT INPUT,
        # which gets a gate ignored just as fast as crying wolf does.
        base=$( cd "$FORK" && "$NIX" develop "$FORK" --command go vet $PKGS 2>&1 | sort )
        now=$( cd "$WORK" && "$NIX" develop "$FORK" --command go vet $PKGS 2>&1 | sort )
        vout=$( comm -13 <(printf '%s\n' "$base") <(printf '%s\n' "$now") )
        if [ -n "$vout" ]; then
            echo "$vout" | sed 's/^/   /'
            echo "   FAIL: go vet reports findings the unmodified fork does not"
            fail=1
        else
            echo "   PASS: go vet introduces nothing new (baselined against $FORK)"
        fi
    fi
else
    echo "   SKIP: no whole-file blocks were extracted"
fi

echo
echo "== 4 -- TIER 2: gofmt -e parse check (INFORMATIONAL, never gates) =="
python3 - "$WORK" "$NIX" "$FORK" <<'PY'
import json, os, subprocess, sys
work, nix, fork = sys.argv[1], sys.argv[2], sys.argv[3]
frags = json.load(open(os.path.join(work, ".frags.json")))
if not frags:
    print("   no fragment blocks")
    raise SystemExit(0)

def parses(src):
    p = subprocess.run([nix, "develop", fork, "--command", "gofmt", "-e"],
                       input=src, capture_output=True, text=True)
    return p.returncode == 0

unparsed = []
for lineno, path, code in frags:
    wrappers = [
        "package p\n" + code,                              # decls, whole funcs
        "package p\nfunc _() {\n" + code + "\n}",          # statements
        "package p\ntype _ struct {\n" + code + "\n}",     # struct fields
        "package p\ntype _ interface {\n" + code + "\n}",  # interface methods
    ]
    if not any(parses(w) for w in wrappers):
        unparsed.append((lineno, path))

ok = len(frags) - len(unparsed)
print(f"   {ok}/{len(frags)} fragment block(s) parse as Go under some wrapper")
if unparsed:
    print("   NOT mechanically parseable -- these need a reviewer's execution pass:")
    for lineno, path in unparsed:
        print(f"     plan line {lineno}  ({path or 'no file anchor'})")
PY

echo
echo "== RESULT =="
if [ "$fail" -eq 0 ]; then
    echo "   TIER 1 clean (or nothing to check)"
else
    echo "   TIER 1 FAILED -- fix before review"
fi
cat <<'EOT'
   NOT covered: TIER 2 proves SYNTAX only. A fragment naming a function that
   does not exist, passing a wrong type, or returning the wrong arity parses
   happily. Fragments are MODIFICATIONS to existing files and cannot be
   assembled mechanically -- they remain a reviewer's execution pass. Say so in
   the review brief; naming what is already machine-verified is what moves
   reviewer budget onto what tools cannot reach.
EOT
exit "$fail"
