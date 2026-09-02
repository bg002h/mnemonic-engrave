#!/usr/bin/env bash
# plan-build-gate-go.sh -- compile and run the Go that lives inside a seedhammer-fork
# implementation plan (the wallet-policy COMPOSER's Stage 2), so a fold that does
# not build never reaches a reviewer.
#
# WHY THIS EXISTS. The two sibling gates (plan-build-gate-md.sh, plan-build-gate-me.sh)
# cover the Rust repos. Stage 2 adds Go to the fork's `md/`, `mk/` and `sysw/`
# packages, and its byte-parity tests need the Rust corpus vendored beside them.
# Same shape as the siblings, different language and source tree.
#
# WHAT IT DOES
#   1. Scratch copy of the fork WITHOUT .git (199 MB; on /scratch, not the tmpfs):
#      $SCRATCH (default /scratch/code/shibboleth/.plan-build-gate-go/seedhammer).
#      Go is the nix-store toolchain the fork's flake pins (GO env var overrides).
#   2. VENDORS into the copy what the plan's tests read: every
#      `compose_*` / `keyed_compose_*` vector file from descriptor-mnemonic
#      `crates/md-codec/tests/vectors/` (MD_REPO, default the sibling checkout) into
#      `md/testdata/vectors/`, and `crates/me-cli/testdata/record_class_vectors.json`
#      from mnemonic-engrave (ME_REPO) into `sysw/testdata/`. The REAL vendoring is a
#      plan task with provenance files; this is the gate's stand-in so the tests run.
#   3. Extracts every ```go block that follows an anchor naming one of the NEW files
#      a plan may create:
#        md/compose*.go   mk/compose*.go   sysw/composer_*.go   gui/composer_*.go
#      Anchor grammar as the siblings: `Create <path>`, `Prepend to <path>`,
#      `Add to <path>`, `In <path>`, `Replace <path>` with the path in backticks;
#      the NEXT ```go fence is that file's content; blocks for one path are
#      concatenated in plan order (a block opening with `package ` goes first);
#      `Replace` discards earlier blocks. Blocks anchored on EXISTING files
#      (script_emit.go, policy_shape.go, record.go, multisig_build_slots.go,
#      testdata_test.go) are fragments and are NOT assembled -- the controller
#      hand-wires them in the scratch copy before review, as Stage 0 and 1 did.
#   4. `go vet ./md/ ./mk/ ./sysw/` then `go test -count=1 ./md/ ./mk/ ./sysw/`
#      (the packages the plan touches); when the plan created any
#      `gui/composer_*.go`, also `go vet ./gui/` and
#      `go test -run '^TestComposer' ./gui/` (the whole gui package is sharded
#      separately by scripts/gui-shard-test.sh and is a plan task, not this gate).
#
# EXTRACTING NOTHING IS A FAILURE, NOT A PASS (exit 3).
#
# Usage:  scripts/plan-build-gate-go.sh design/IMPLEMENTATION_PLAN_composer_S2_fork_codec.md
set -euo pipefail
PLAN="${1:?plan path required}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FORK="${FORK_REPO:-/scratch/code/shibboleth/seedhammer}"
MD_REPO="${MD_REPO:-/scratch/code/shibboleth/descriptor-mnemonic}"
ME_REPO="${ME_REPO:-$HERE}"
SCRATCH="${SCRATCH:-/scratch/code/shibboleth/.plan-build-gate-go}"
WORK="$SCRATCH/seedhammer"
GO="${GO:-$(ls -d /nix/store/*-go-1.2*/bin/go 2>/dev/null | sort | tail -1)}"
[ -x "$GO" ] || { echo "no go toolchain found (set GO=/path/to/go)" >&2; exit 2; }
[ -f "$HERE/$PLAN" ] || [ -f "$PLAN" ] || { echo "no plan at $PLAN" >&2; exit 2; }
[ -f "$HERE/$PLAN" ] && PLAN="$HERE/$PLAN"
echo "== 1 -- scratch copy of the fork (no .git) =="
rm -rf "$WORK"; mkdir -p "$WORK"
( cd "$FORK" && tar --exclude=.git -cf - . ) | ( cd "$WORK" && tar -xf - )
echo "   $WORK  (go: $("$GO" version))"
echo "== 2 -- vendor the corpus the plan's tests read =="
mkdir -p "$WORK/md/testdata/vectors" "$WORK/sysw/testdata"
n=$(ls "$MD_REPO"/crates/md-codec/tests/vectors/ 2>/dev/null | grep -cE '^(keyed_)?compose_' || true)
[ "$n" -gt 0 ] && cp "$MD_REPO"/crates/md-codec/tests/vectors/*compose_* "$WORK/md/testdata/vectors/"
echo "   compose vector files vendored: $n"
if [ -f "$ME_REPO/crates/me-cli/testdata/record_class_vectors.json" ]; then
  cp "$ME_REPO/crates/me-cli/testdata/record_class_vectors.json" "$WORK/sysw/testdata/"; echo "   record_class_vectors.json vendored"
else
  echo "   record_class_vectors.json NOT present in $ME_REPO (Stage 1 not merged yet); tests that read it will fail here"
fi
echo "== 3 -- extract the plan's Go =="
python3 - "$PLAN" "$WORK" <<'PY'
import re, sys, os, collections
plan, work = sys.argv[1], sys.argv[2]
lines = open(plan).read().split("\n")
anchor = re.compile(r'\b(create|prepend to|add to|in|replace)\s+`([^`]*\.go)`', re.I)
ok = re.compile(r'^md/compose[A-Za-z0-9_]*\.go$|^mk/compose[A-Za-z0-9_]*\.go$|^sysw/composer_[A-Za-z0-9_]+\.go$|^gui/composer_[A-Za-z0-9_]+\.go$')
blocks = collections.OrderedDict(); cur = None; prepend = False; replace = False; fragments = set()
i = 0
while i < len(lines):
    m = anchor.search(lines[i])
    if m:
        verb, path = m.group(1), m.group(2)
        if ok.match(path): cur, prepend, replace = path, verb.lower().startswith("prepend"), verb.lower() == "replace"
        else:
            cur = None
            if path.endswith(".go"): fragments.add(path)
    if lines[i].startswith("```go") and cur:
        i += 1; buf = []
        while i < len(lines) and not lines[i].startswith("```"): buf.append(lines[i]); i += 1
        code = "\n".join(buf)
        if replace:
            dropped = len(blocks.get(cur, []))
            blocks[cur] = []
            if dropped: print("   replaced %s (dropped %d earlier block%s)" % (cur, dropped, "" if dropped==1 else "s"))
            replace = False
        blocks.setdefault(cur, []).append((prepend or code.lstrip().startswith("package "), code))
    i += 1
if not blocks:
    sys.stderr.write("\nplan-build-gate-go: EXTRACTED NOTHING from %s\n  Recognised anchors: md/compose*.go, mk/compose*.go, sysw/composer_*.go, gui/composer_*.go.\n  Refusing rather than reporting a pass on an empty extraction.\n" % plan)
    sys.exit(3)
for path, parts in blocks.items():
    parts.sort(key=lambda t: not t[0])
    full = os.path.join(work, path); os.makedirs(os.path.dirname(full), exist_ok=True)
    open(full, "w").write("\n\n".join(c for _, c in parts) + "\n")
    print("   wrote %s (%d block%s)" % (path, len(parts), "" if len(parts)==1 else "s"))
if fragments:
    print("   NOT assembled (fragments of existing files; controller hand-wires, reviewer's execution pass): " + ", ".join(sorted(fragments)))
PY
cd "$WORK"
export CGO_ENABLED=0 GOFLAGS=-mod=mod GOPROXY=off GOTOOLCHAIN=local
echo "== 4 -- gofmt on the extracted files =="
GOFMT="$(dirname "$GO")/gofmt"
for f in $(cd "$WORK" && find md mk sysw gui -maxdepth 1 \( -name 'compose*.go' -o -name 'composer_*.go' \) 2>/dev/null | sort); do
  d=$("$GOFMT" -l "$WORK/$f" 2>&1 || true); [ -z "$d" ] || echo "   gofmt would change: $d"
done
echo "== 5 -- go vet ./md/ ./mk/ ./sysw/ =="
"$GO" vet ./md/ ./mk/ ./sysw/ 2>&1 | tail -20
echo "== 6 -- go test -count=1 ./md/ ./mk/ ./sysw/ =="
"$GO" test -count=1 ./md/ ./mk/ ./sysw/ 2>&1 | tail -30
if ls "$WORK"/gui/composer_*.go >/dev/null 2>&1; then
  echo "== 7 -- gui: vet + the composer tests only (go test -run '^TestComposer' ./gui/; the whole package is sharded elsewhere) =="
  "$GO" vet ./gui/ 2>&1 | tail -20
  "$GO" test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -30
fi
echo "== 8 -- DEAD-IN-PROD: functions the plan declares in production files that no production file calls =="
# Go does not flag unused package-scope functions, and every plan test can call a
# function directly, so a whole feature can be built, tested and never joined to
# the flow while every gate above prints ok (composer S3 R0 r0, 2026-09-02: 14
# such functions). This counts, per NEW production file the plan created, every
# top-level `func name(` (methods excluded) whose name occurs in no OTHER
# non-test .go file of the same package. A hit is not always a defect (an API for
# a later stage is one), so it prints and does not fail; the reviewer decides.
python3 - "$WORK" <<'PY2'
import os, re, sys
work = sys.argv[1]
def code_only(src):
    # drop // comments and /* */ blocks so a doc comment naming the function does not count as a caller
    src = re.sub(r'/\*.*?\*/', '', src, flags=re.S)
    return re.sub(r'//[^\n]*', '', src)
for pkg in ("md", "mk", "sysw", "gui"):
    d = os.path.join(work, pkg)
    if not os.path.isdir(d): continue
    new_files = [f for f in os.listdir(d) if (f.startswith("compose") or f.startswith("composer_")) and f.endswith(".go") and not f.endswith("_test.go")]
    if not new_files: continue
    prod = {f: code_only(open(os.path.join(d, f)).read()) for f in os.listdir(d) if f.endswith(".go") and not f.endswith("_test.go")}
    dead = []
    for f in new_files:
        for m in re.finditer(r'^func ([A-Za-z_][A-Za-z0-9_]*)\(', prod[f], re.M):
            name = m.group(1)
            if name in ("init", "main"): continue
            refs = sum(len(re.findall(r'\b' + re.escape(name) + r'\b', body)) for body in prod.values())
            if refs <= 1: dead.append("%s/%s: %s" % (pkg, f, name))
    print("   %s: %d new production file(s), %d function(s) with no production caller" % (pkg, len(new_files), len(dead)))
    for x in dead: print("      DEAD-IN-PROD " + x)
PY2
echo "== NOT covered: fragments of existing files (hand-wired by the controller before review); the whole ./gui/ package (sharded separately); the TinyGo firmware build and its size delta (a plan task); the real vendoring with provenance (a plan task); whether a DEAD-IN-PROD function is a defect or a deliberate later-stage API (a reviewer's call). =="
