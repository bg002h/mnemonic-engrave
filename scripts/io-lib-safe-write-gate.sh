#!/usr/bin/env bash
# io-lib-safe-write-gate.sh -- the committed gate for design/PLAN_io_lib_safe_write.md.
# Pattern: scripts/plan-build-gate.sh. Run before every commit of the plan and of each build phase.
#
#   scripts/io-lib-safe-write-gate.sh plan    spec/plan/coverage-map consistency (no build)
#   scripts/io-lib-safe-write-gate.sh red1    after RED-1: 15 tests, exactly W1-W4 FAIL, 11 pass
#   scripts/io-lib-safe-write-gate.sh red2    after RED-2: unit-test build fails with E0425 on same_file
#   scripts/io-lib-safe-write-gate.sh green   after GREEN / version commit: 38 rows pass, workspace
#                                              nextest, doctests, lint-gate, Windows+macOS compile
#                                              checks, version 0.1.3, empty [dependencies]
#
# Each step is captured ONCE to $OUT/<step>.txt (OUT defaults to a fresh dir under $TMPDIR-free
# /scratch/cache) and the verdict is read from that file. Exit 0 = every step passed.
#
# NOT covered: (1) whether a mutant of the plan's mutant table fails its row (run by hand, pasted
# in the build report); (2) any row RUNNING on Windows or macOS (only `cargo check --tests` there);
# (6) WHY W1-W4 fail at red1 (the gate reads names and counts, not the failure reason: read
# the failure lines in $OUT/red1.txt against the spec's "measured today" cells); (7) the Go
# oracles that nextest skips without ME_REQUIRE_GO, and ci/code-docs-check.sh (run by hand);
# (3) the nightly clippy step of lint-gate.sh (WARN only, red on the base tree); the release steps R1-R4 and consumer steps C1-C5; (4) the BEHAVIOUR of the code against the
# spec beyond what the spec's own rows assert; (5) quote checks use single-line fragments, so a
# spec sentence that wraps is matched by a fragment within one line.
# (8) red1 is #41's history; 0.1.3's RED is W20-W24 on e09d538d, by hand.
set -uo pipefail
HERE=$(cd "$(dirname "$0")/.." && pwd)
cd "$HERE" || exit 2
MODE=${1:-plan}
SPEC=design/SPEC_io_lib_safe_write.md
PLAN=design/PLAN_io_lib_safe_write.md
MAP=design/io-lib-safe-write-coverage.tsv
OUT=${OUT:-/scratch/cache/io-lib-gate-out-$$}
mkdir -p "$OUT"
export PATH="$HOME/.cargo/bin:$PATH"
# the repo has no rust-toolchain.toml; CI pins RUST_TOOLCHAIN 1.85.0 (release.yml:56) and the rustup default here is nightly
export RUSTUP_TOOLCHAIN=1.85.0
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/scratch/cache/target-io-lib-safe-write}
FAILS=0
ok() { echo "PASS $*"; }
bad() { echo "FAIL $*"; FAILS=$((FAILS + 1)); }

# ---- row ids -> names, extracted from the spec's section-8 table (the only source of names)
rows_from_spec() {
  grep -E '^\| (W|S)[0-9]+ `[a-z0-9_]+`' "$SPEC" | sed -E 's/^\| ((W|S)[0-9]+) `([a-z0-9_]+)`.*/\1 \3/'
  grep -E '^\| `(an_existing_world_readable_target_is_tightened_not_inherited|a_shrinking_overwrite_leaves_no_stale_tail)`' "$SPEC" \
    | sed -E 's/^\| `([a-z0-9_]+)`.*/EX \1/'
}

check_plan() {
  local n
  # spec texts of the `spec: r3` commit
  grep -qF 'x86_64 `docker run` step' "$SPEC" && ok "spec r3 M1 text" || bad "spec r3 M1 text missing"
  grep -qF 'W5-W11 and W17 pass.' "$SPEC" && grep -qF 'W12-W16 and W18 land in the GREEN commit' "$SPEC" \
    && ! grep -qF 'lands in the GREEN commit)' "$SPEC" && ok "spec r2 M2 texts" || bad "spec r2 M2 texts"
  # no code fences in the plan
  n=$(grep -c '^```' "$PLAN"); [ "$n" -eq 0 ] && ok "plan has no fenced code block" || bad "plan has $n fence lines"
  # every spec row name is in the plan (the two existing rows by name, the W/S rows by id)
  local id name total=0
  while read -r id name; do
    total=$((total + 1))
    if [ "$id" = EX ]; then
      grep -qF "\`$name\`" "$PLAN" || bad "plan lacks existing row $name"
    else
      grep -qF "[$id]" "$PLAN" || bad "plan lacks row id $id"
    fi
  done < <(rows_from_spec)
  [ "$total" -eq 37 ] && ok "37 spec rows (24 W + 11 S + 2 existing)" || bad "spec rows found: $total (expected 37)"
  # coverage map
  local lines=0 missing=0 noowner=0 line owner quote
  while IFS=$'\t' read -r _ _ quote owner; do
    lines=$((lines + 1))
    grep -qF -- "$quote" "$SPEC" || { echo "  quote not in spec: $quote"; missing=$((missing + 1)); }
    case "$owner" in
      CODE | OUT:*) ;;
      *) grep -qF "[$owner]" "$PLAN" || { echo "  owner not in plan: $owner"; noowner=$((noowner + 1)); } ;;
    esac
  done < "$MAP"
  [ "$missing" -eq 0 ] && ok "all $lines coverage quotes occur in the spec" || bad "$missing quotes not in spec"
  [ "$noowner" -eq 0 ] && ok "all owner ids occur in the plan" || bad "$noowner owners not in plan"
  echo "COUNT map rows=$lines ROW/STEP=$(awk -F'\t' '$4!="CODE" && $4 !~ /^OUT:/' "$MAP" | wc -l) CODE=$(awk -F'\t' '$4=="CODE"' "$MAP" | wc -l) OUT=$(awk -F'\t' '$4 ~ /^OUT:/' "$MAP" | wc -l)"
  # every W/S/EX row owns at least one map row
  while read -r id name; do
    [ "$id" = EX ] && continue
    awk -F'\t' -v i="$id" '$4==i{f=1} END{exit !f}' "$MAP" || bad "row $id has no coverage-map line"
  done < <(rows_from_spec)
}

nextest() { # nextest <step> <args...>   captured once
  local step=$1; shift
  timeout 1800 cargo nextest run --locked --build-jobs 6 -j 6 --no-fail-fast "$@" > "$OUT/$step.txt" 2>&1
  return $?
}

need_not_root() { [ "$(id -u)" -ne 0 ] && ok "not root" || bad "running as root: W4/S10 preconditions panic"; }
need_toolchain() {
  cargo --version | grep -q '^cargo 1\.85\.0' && ok "cargo 1.85.0" || bad "cargo --version is not 1.85.0 ($(cargo --version))"
}

IOFILTER='package(mnemonic-io-lib) and test(/write::tests::/)'

mode_red1() {
  need_toolchain; need_not_root
  nextest red1 -p mnemonic-io-lib -E "$IOFILTER"
  local sum; sum=$(grep -E 'Summary' "$OUT/red1.txt" | tail -1)
  echo "$sum"
  echo "$sum" | grep -qE ' 15 tests run: 11 passed, 4 failed' && ok "15 run, 11 passed, 4 failed" || bad "summary is not 15 run / 11 passed / 4 failed"
  local want got
  want=$(rows_from_spec | awk '$1=="W1"||$1=="W2"||$1=="W3"||$1=="W4"{print $2}' | sort)
  got=$(grep -E '^ +FAIL ' "$OUT/red1.txt" | sed -E 's/.*write::tests::([a-z0-9_]+).*/\1/' | sort -u)
  [ "$want" = "$got" ] && ok "failing set is exactly W1-W4" || { bad "failing set differs"; echo "want: $want"; echo "got: $got"; }
}

mode_red2() {
  need_toolchain
  cargo nextest run --locked --build-jobs 6 -p mnemonic-io-lib --no-run > "$OUT/red2.txt" 2>&1
  local rc=$?
  [ "$rc" -ne 0 ] && ok "unit-test build fails" || bad "unit-test build succeeded (rc 0)"
  grep -q 'E0425' "$OUT/red2.txt" && grep -q 'same_file' "$OUT/red2.txt" && ok "E0425 names same_file" || bad "no E0425 on same_file"
  grep -oE 'error\[E[0-9]+\]' "$OUT/red2.txt" | sort | uniq -c
}

mode_green() {
  need_toolchain; need_not_root
  nextest green-io -p mnemonic-io-lib -E "$IOFILTER"
  local sum; sum=$(grep -E 'Summary' "$OUT/green-io.txt" | tail -1); echo "$sum"
  echo "$sum" | grep -qE ' 38 tests run: 38 passed' && ok "38 rows passed" || bad "not 38 run / 38 passed"
  local id name
  while read -r id name; do
    grep -qE "PASS .*write::tests::$name\$" "$OUT/green-io.txt" || bad "row $id $name did not PASS"
  done < <(rows_from_spec)
  nextest green-ws --workspace --all-targets
  grep -E 'Summary' "$OUT/green-ws.txt" | tail -1
  grep -qE 'Summary .* 0 failed|Summary .* [0-9]+ passed(, [0-9]+ skipped)?$' "$OUT/green-ws.txt" && ! grep -qE '^ +(FAIL|TIMEOUT|SIGABRT) ' "$OUT/green-ws.txt" \
    && ok "workspace nextest" || bad "workspace nextest has failures"
  cargo test --locked --doc > "$OUT/green-doc.txt" 2>&1 && ok "doctests" || bad "doctests"
  # lint-gate.sh = fmt + clippy 1.85.0 (CI's) + clippy nightly (not a CI job). The nightly step is red on the
  # base tree (cloned_ref_to_slice_refs in me-cli tests, measured 2026-11-22), so it is a WARN unless the
  # nightly section carries a mnemonic-io-lib diagnostic (`--> crates/mnemonic-io-lib/`); fmt and clippy 1.85.0 are hard.
  if bash scripts/lint-gate.sh > "$OUT/green-lint.txt" 2>&1; then ok "lint-gate"
  elif grep -q '^== clippy (nightly)' "$OUT/green-lint.txt" && ! sed -n '/^== clippy (nightly)/,$p' "$OUT/green-lint.txt" | grep -qE -- '--> crates/mnemonic-io-lib/|could not compile .mnemonic-io-lib.' \
    && ! sed -n '1,/^== clippy (nightly)/p' "$OUT/green-lint.txt" | grep -qE '^(error|Diff in)'; then
    echo "WARN lint-gate: only the nightly step is red, and not in mnemonic-io-lib (base-tree baseline red)"
  else bad "lint-gate (see $OUT/green-lint.txt)"; fi
  local t
  for t in x86_64-pc-windows-gnu x86_64-apple-darwin; do
    cargo check --locked -p mnemonic-io-lib --tests --target "$t" > "$OUT/green-check-$t.txt" 2>&1 \
      && ok "cargo check --tests $t" || bad "cargo check --tests $t"
  done
  if [ "${IOLIB_PRE_BUMP:-0}" != 1 ]; then
    grep -qE '^version = "0\.1\.3"' crates/mnemonic-io-lib/Cargo.toml && ok "crate version 0.1.3" || bad "crate version is not 0.1.3"
    awk '/name = "mnemonic-io-lib"/{getline; print}' Cargo.lock | grep -q '"0.1.3"' && ok "Cargo.lock 0.1.3" || bad "Cargo.lock not 0.1.3"
  fi
  cargo metadata --locked --format-version 1 --no-deps 2> /dev/null \
    | python3 -c 'import json,sys; m=json.load(sys.stdin); p=[x for x in m["packages"] if x["name"]=="mnemonic-io-lib"][0]; sys.exit(1 if [d for d in p["dependencies"] if d["kind"] is None] else 0)' \
    && ok "[dependencies] empty" || bad "[dependencies] not empty"
  grep -q 'inputs' crates/mnemonic-io-lib/src/lib.rs && ok "lib.rs doc mentions inputs" || bad "lib.rs doc lacks the same_file sentence"
  grep -q 'one of my inputs' crates/mnemonic-io-lib/src/write.rs && ok "write.rs module doc has the same_file sentence" || bad "write.rs module doc lacks the same_file sentence"
  ! grep -q 'tightening the OPEN file' crates/mnemonic-io-lib/src/channel.rs && ok "channel.rs old wording gone" || bad "channel.rs still says tightening the OPEN file"
}

case "$MODE" in
  plan) check_plan ;;
  red1) mode_red1 ;;
  red2) mode_red2 ;;
  green) mode_green ;;
  *) echo "usage: $0 plan|red1|red2|green"; exit 2 ;;
esac
echo "out dir: $OUT"
if [ "$FAILS" -ne 0 ]; then echo "GATE FAIL ($MODE): $FAILS"; exit 1; fi
echo "GATE PASS ($MODE)"
