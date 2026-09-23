#!/usr/bin/env bash
# plan-apply-gate.sh <fork-base> [plan] -- the build gate for the F-449 stage-4 plan.
#
# Extracts every `Create `path`:` block (whole file) and `Apply to `path`:` block
# (a unified diff) from the plan, per `## Task N`, applies them IN ORDER to a
# fresh clone of <fork-base> (and the design/journeys ones to a scratch copy of
# mnemonic-engrave's journeys), and gates EACH task boundary: go build ./...,
# go vet (md clean; gui only the two baseline ArtifactDir lines), go test ./md/,
# and the task's own gui tests; Task 7 adds ./cmd/emu/ and node --check. Ends
# with the whole gui package, sharded, and gofmt against the five-file baseline.
#
# WHAT IT DOES NOT COVER: the emulator walk (a browser; Task 7 Step 4), the
# TinyGo size (Task 8 Step 2), the mutation pass (Task 8 Step 3), the Liana and
# Core evidence (Task 9 Step 1), and anything in prose.
set -euo pipefail
BASE="${1:?fork base dir (a clean commit: the stage-3 merge, or the r0-s3 scratch)}"
PLAN="${2:-/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_f449_stage4_device.md}"
ENG="${ENG:-/scratch/code/shibboleth/mnemonic-engrave}"
W=/scratch/code/shibboleth/.tmp/f449s4-apply-gate
export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH TMPDIR=$W/tmp
rm -rf "$W"; mkdir -p "$W/tmp" "$W/blocks" "$W/eng/design"
git clone -q "$BASE" "$W/fork"
cp -r "$ENG/design/journeys" "$W/eng/design/journeys"
python3 - "$PLAN" "$W/blocks" <<'PY'
import re, sys, os
plan, out = sys.argv[1], sys.argv[2]
lines = open(plan).read().split("\n")
task, n, pending, i = None, 0, None, 0
man = open(os.path.join(out, "manifest.tsv"), "w")
while i < len(lines):
    l = lines[i]
    m = re.match(r"^## Task (\d+) ", l)
    if m:
        task = int(m.group(1))
    m = re.match(r"^(Create|Apply to) `([^`]+)`", l)
    if m and task:
        pending = (m.group(1), m.group(2))
    if pending and l.startswith("```") and len(l) > 3:
        j = i + 1
        body = []
        while not (lines[j] == "```"):
            body.append(lines[j]); j += 1
        n += 1
        f = os.path.join(out, "%03d" % n)
        open(f, "w").write("\n".join(body) + "\n")
        man.write("%d\t%s\t%s\t%s\n" % (task, pending[0], pending[1], f))
        pending, i = None, j
    i += 1
man.close()
print("extracted", n, "blocks")
PY
gate() {
  local t="$1" rx="$2"
  cd "$W/fork"
  go build ./... || { echo "BOUNDARY Task $t: BUILD FAILED"; exit 1; }
  go vet ./md/ || { echo "BOUNDARY Task $t: md VET FAILED"; exit 1; }
  if go vet ./gui/ 2>&1 | grep -v ArtifactDir | grep -v '^#' | grep .; then echo "BOUNDARY Task $t: gui VET"; exit 1; fi
  go test -count=1 ./md/ >/dev/null || { echo "BOUNDARY Task $t: md TESTS FAILED"; go test -count=1 ./md/ | tail -20; exit 1; }
  if [ -n "$rx" ]; then go test -count=1 -run "$rx" ./gui/ >"$W/tmp/t$t.txt" 2>&1 || { tail -30 "$W/tmp/t$t.txt"; echo "BOUNDARY Task $t: gui TESTS FAILED"; exit 1; }; fi
  echo "BOUNDARY Task $t: green (md $(go test ./md/ -list '.*' | grep -cE '^(Test|Fuzz|Example)') tests)"
}
declare -A RX=(
  [2]='^TestCompose' [3]='TestPolicy|TestMd1|TestTemplate' [4]='Address|Taproot|Underivable|DeviceDerives'
  [5]='TestComposerCopy|KeyPath|Liana|Inspect|Unnamed|Fable|TestEmulatorWalks|Modal'
  [6]='TestComposer|Liana|Unspendable|TestEmulatorWalks|Modal' [7]='TestEmulatorWalks'
)
last=0
while IFS=$'\t' read -r t kind path file; do
  if [ "$t" != "$last" ] && [ "$last" != 0 ]; then gate "$last" "${RX[$last]:-}"; fi
  last=$t
  case "$path" in
    design/*) root="$W/eng" ;;
    *) root="$W/fork" ;;
  esac
  if [ "$kind" = "Create" ]; then
    mkdir -p "$(dirname "$root/$path")"; cp "$file" "$root/$path"
  else
    (cd "$root" && git apply --unidiff-zero -p1 "$file" 2>/dev/null) || (cd "$root" && patch -s -p1 < "$file") \
      || { echo "Task $t: $path DID NOT APPLY"; exit 1; }
  fi
done < "$W/blocks/manifest.tsv"
gate "$last" "${RX[$last]:-}"
cd "$W/fork"
go test -count=1 ./cmd/emu/ >/dev/null && node --check cmd/emu/shots_composer.js || { echo "Task 7: cmd/emu FAILED"; exit 1; }
bash -n "$W/eng/design/journeys/transcript_composer.sh" && python3 -m py_compile "$W/eng/design/journeys/capture_composer.py"
diff <(printf '%s\n' gui/transaction.go gui/transaction_golden_test.go gui/transaction_txrecord_test.go mt/mt.go mt/mt_test.go) <(gofmt -l . | sort) || { echo "gofmt drifted"; exit 1; }
/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24 > "$W/tmp/gui.txt" 2>&1; tail -1 "$W/tmp/gui.txt"
grep -q "RESULT: ok" "$W/tmp/gui.txt" || { echo "gui shards FAILED"; exit 1; }
echo "ALL BOUNDARIES GREEN"
