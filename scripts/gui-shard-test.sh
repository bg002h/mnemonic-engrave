#!/usr/bin/env bash
# gui-shard-test.sh -- run a Go package's tests as PARALLEL SHARDS, provably.
#
# WHY THIS EXISTS. seedhammer's `gui` package is ~93% of the whole suite and runs
# on ONE core (zero t.Parallel(), zero testing.Short()). It grew 440.6s -> 496.8s
# in a single S6b phase and was projected to pass Go's 600s per-package default
# by P6. A package that dies at 600s reports EVERY ASSERTION PASSING, so the
# ceiling does not fail loudly -- it fails silently and green.
#
# Sharding fixes both problems at once: each shard is its own process with its
# own timeout budget, and they run concurrently on the 24 cores available.
#
# THE DANGER THIS SCRIPT EXISTS TO PREVENT. The obvious way to shard is to write
# an "everything else" regex by hand. That SILENTLY DROPS every test added
# afterwards, and a shard set that skips tests still reports ok. This project's
# standing rule is that a skipped gate is the default failure mode, so:
#
#   THE PARTITION IS ENUMERATED FROM `go test -list`, AND THE UNION IS ASSERTED
#   AGAINST THE FULL LIST BEFORE ANYTHING RUNS. Exhaustiveness is CHECKED, never
#   assumed. If one test would be missed, this script refuses to run at all.
#
# It also prints what it does NOT cover, per house practice:
#   - subtests are not partitioned; a shard runs a top-level test whole;
#   - it shards ONE package, so cross-package ordering effects are unchanged;
#   - it does not detect tests that depend on each other's side effects. Sharding
#     would EXPOSE such coupling as a failure, not hide it, but the failure would
#     look like a flake. If a shard fails and a full run passes, suspect coupling
#     before suspecting the shard.

set -euo pipefail

PKG="${1:-./gui/}"
SHARDS="${2:-6}"
TIMEOUT="${3:-20m}"
OUT="$(mktemp -d)"

command -v go >/dev/null || { echo "go not on PATH" >&2; exit 2; }

echo "=== enumerating tests in $PKG ==="
go test "$PKG" -list '.*' 2>/dev/null | grep -E '^(Test|Example|Fuzz)' | sort -u > "$OUT/all.txt"
TOTAL=$(wc -l < "$OUT/all.txt")
[ "$TOTAL" -gt 0 ] || { echo "no tests enumerated -- refusing to report success" >&2; exit 2; }
echo "    $TOTAL top-level tests"

# Partition. Heavy tests first, one per shard, so the longest shard is the
# longest single test rather than a pile of them. Everything else round-robins.
python3 - "$OUT" "$SHARDS" <<'PY'
import sys, os
out, k = sys.argv[1], int(sys.argv[2])
names = [l.strip() for l in open(os.path.join(out, "all.txt")) if l.strip()]
# measured hot spots (S6b baseline): 86% of wall in four tests
heavy = ["TestBothEngraveFlowsDriveTheRetryLoop", "TestBuildWalkTypedSeed",
         "TestSupplyPassphraseRunTellsTheOperatorWhatIsMissing",
         "TestSingleSigPassphraseRunTellsTheOperatorWhatIsMissing"]
buckets = [[] for _ in range(k)]
for i, h in enumerate(x for x in heavy if x in names):
    buckets[i % k].append(h)
rest = [n for n in names if n not in heavy]
for i, n in enumerate(rest):
    buckets[(i + len(heavy)) % k].append(n)
for i, b in enumerate(buckets):
    with open(os.path.join(out, f"shard{i}.txt"), "w") as f:
        f.write("\n".join(b))
# EXHAUSTIVENESS: the union must equal the input, exactly.
union = sorted(x for b in buckets for x in b)
if union != sorted(names):
    missing = set(names) - set(union); extra = set(union) - set(names)
    print(f"PARTITION IS NOT EXHAUSTIVE -- missing {sorted(missing)}, extra {sorted(extra)}")
    sys.exit(2)
print(f"    partition verified exhaustive: {len(union)} == {len(names)}")
PY

echo "=== running $SHARDS shards in parallel (timeout $TIMEOUT each) ==="
pids=(); start=$(date +%s)
for i in $(seq 0 $((SHARDS-1))); do
    [ -s "$OUT/shard$i.txt" ] || continue
    RE="^($(paste -sd'|' "$OUT/shard$i.txt"))\$"
    go test "$PKG" -run "$RE" -count=1 -timeout "$TIMEOUT" \
        > "$OUT/out$i.txt" 2> "$OUT/err$i.txt" &
    pids+=("$!:$i")
done

fail=0
for p in "${pids[@]}"; do
    pid="${p%%:*}"; i="${p##*:}"
    if wait "$pid"; then st=ok; else st=FAIL; fail=1; fi
    n=$(wc -l < "$OUT/shard$i.txt")
    printf '  shard %d: %-4s %3d tests  %s\n' "$i" "$st" "$n" "$(tail -1 "$OUT/out$i.txt")"
    [ -s "$OUT/err$i.txt" ] && { echo "    stderr NOT empty:"; head -3 "$OUT/err$i.txt"; fail=1; }
done
echo "=== wall: $(( $(date +%s) - start ))s ==="

if [ "$fail" -ne 0 ]; then
    echo "RESULT: FAIL -- artifacts in $OUT" >&2
    exit 1
fi
echo "RESULT: ok -- all $TOTAL tests ran across $SHARDS shards"
rm -rf "$OUT"
