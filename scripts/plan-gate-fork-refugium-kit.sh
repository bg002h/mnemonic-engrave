#!/usr/bin/env bash
# plan-gate-fork-refugium-kit.sh -- machine gate for design/IMPLEMENTATION_PLAN_fork_refugium_kit.md (goal 2, plan S2).
#
# Modes:
#   static             (default) plan and coverage-map checks; needs ME_REPO (a clone holding the S1/S1b spec commits)
#   counts [--built [PHASES]]  each ROW / CHANGE line's count: `now=` at the baselines, or `built=` (and CHANGE present) with
#                      --built; PHASES (comma-separated, e.g. ME-1,SH-1) limits the check to those phases' lines;
#                      needs FORK (a seedhammer checkout) and ME_WT (a mnemonic-engrave checkout); Go from GO_BIN
#   probes             the plan's RED probes against FORK (must be a clean checkout at the fork baseline)
#   selftest           shows this gate going RED: runs `static` on broken copies of the plan and the map
#   release            the G-REL gate (plan §9): fail-closed: exit 1 unless F-124 and F-123 are both CLOSED in ME_WT's design/FOLLOWUPS.md (exit 2 if the file is missing)
#
# Env: PLAN, COV (default: this repo's plan and map), ME_REPO (default /scratch/code/shibboleth/mnemonic-engrave),
#      FORK, ME_WT, GO_BIN (default /home/bcg/.local/go/bin), CARGO_TARGET_DIR (counts: set your own).
#
# NOT COVERED: whether a Rule cell is the right behaviour; the rows' RED/GREEN runs after the baseline (each phase's
# build runs them); the emulator walks (WALK lines) and hardware rows (HW lines), which are run by hand; any spec
# sentence outside S1 §14b-§14g and S1b §14.2-§14.8 (plus the one §14.7 map line; the §14.10 row names are not machine-checked); spec sentences the map does not quote.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
PLAN="${PLAN:-$here/design/IMPLEMENTATION_PLAN_fork_refugium_kit.md}"
COV="${COV:-$here/design/IMPLEMENTATION_PLAN_fork_refugium_kit-coverage.tsv}"
ME_REPO="${ME_REPO:-/scratch/code/shibboleth/mnemonic-engrave}"
GO_BIN="${GO_BIN:-/home/bcg/.local/go/bin}"
S1_SHA=c4737e8b
S1B_SHA=8451c344
mode="${1:-static}"

static() {
	local tmp
	tmp="$(mktemp -d)"
	git -C "$ME_REPO" show "$S1_SHA:design/SPEC_systemwide_payloads.md" >"$tmp/S1.md"
	git -C "$ME_REPO" show "$S1B_SHA:design/SPEC_encrypted_payload_delivery.md" >"$tmp/S1b.md"
	git -C "$ME_REPO" show "$S1B_SHA:scripts/gen-mnemblob-v2-vectors.py" >"$tmp/gen.py"
	local rc=0
	PLAN="$PLAN" COV="$COV" SPECDIR="$tmp" python3 -I - <<'PY' || rc=$?
import os, re, sys, importlib.util
plan = open(os.environ["PLAN"], encoding="utf-8").read()
cov = open(os.environ["COV"], encoding="utf-8").read().splitlines()
specdir = os.environ["SPECDIR"]
specs = {k: " ".join(open(f"{specdir}/{k}.md", encoding="utf-8").read().split()) for k in ("S1", "S1b")}
errs = []
if "```" in plan:
    errs.append("plan holds a code fence")
phases = set(re.findall(r"^PHASE (\S+) ", plan, re.M))
names = {}
for kind, name, phase in re.findall(r"^(ROW|CHANGE|WALK|HW) (\S+(?: \S+)?) \| (\S+) \|", plan, re.M):
    key = name if kind in ("ROW", "CHANGE") else f"{kind} {name}"
    if key in names:
        errs.append(f"duplicate {kind} {name}")
    names[key] = phase
    if phase not in phases:
        errs.append(f"{kind} {name}: phase {phase} is not a PHASE line")
rows = re.findall(r"^ROW (\S+) \| (\S+) \| (me|sh) \| (\S+) \| (\S+) \| now=(\d+) \| built=(\d+)$", plan, re.M)
if len(rows) != len(re.findall(r"^ROW ", plan, re.M)):
    errs.append("a ROW line does not parse")
steps = set()
for ph, body in re.findall(r"^### (\S+) steps\n(.*?)(?=^### |^## )", plan, re.M | re.S):
    for n in re.findall(r"^(\d+)\. ", body, re.M):
        steps.add(f"TASK {ph} step {n}")
hdr = cov[0].split("\t")
if hdr != ["spec", "section", "quote", "owner"]:
    errs.append("coverage map header")
used = 0
for i, line in enumerate(cov[1:], 2):
    f = line.split("\t")
    if len(f) != 4:
        errs.append(f"map line {i}: {len(f)} fields"); continue
    spec, sec, quote, owner = f
    if spec not in specs:
        errs.append(f"map line {i}: spec {spec}"); continue
    if " ".join(quote.split()) not in specs[spec]:
        errs.append(f"map line {i}: quote not in {spec}@pin: {quote[:70]}")
    if owner.startswith("not this plan's: "):
        continue
    if owner in names or owner in steps:
        used += 1; continue
    errs.append(f"map line {i}: owner {owner!r} is no ROW/CHANGE/WALK/HW/TASK of the plan")
# The owed vectors (row v2_titles_refusals' plan leg): recompute with the S1b generator.
spec_ = importlib.util.spec_from_file_location("gen", f"{specdir}/gen.py")
g = importlib.util.module_from_spec(spec_); spec_.loader.exec_module(g)
salt = g.rep("0bad", 8)
owed = [
    g.build("N-not-hex", 6, g.BEEF6, 600_000, salt, g.rep("0c0e", 6), [], [g.MS1["A16"], "text:zz"]),
    g.build("N-mnemonic", 6, g.BEEF6, 600_000, salt, g.rep("0c0f", 6), [],
            [" ".join(["abandon"] * 11 + ["about"]), g.titles_record(["title SEED A"])]),
]
flat = plan.replace("\n", "").replace(" ", "")
for v in owed:
    shape = "|{}|{}|{}|{}|{}|{}|{}|".format(v["name"], v["pw_form"], v["iterations"], v["pub_len"],
                                           v["ct_len"], v["record_count"], v["blob_len"])
    for want in (shape, v["derived_key_hex"], v["tag_hex"], v["blob_sha256"], v["header_hex"]):
        if want not in flat:
            errs.append(f"owed vector {v['name']}: plan lacks {want[:40]}")
if "s1br3-nothex-nonms1-vectors" not in plan:
    errs.append("owed line id s1br3-nothex-nonms1-vectors not in plan")
for e in errs:
    print("FAIL", e)
print(f"static: {len(rows)} ROW lines, {len(names)} named lines, {len(cov)-1} map lines ({used} to rows/tasks), "
      f"{len(phases)} phases, 2 owed vectors recomputed, {len(errs)} failures")
sys.exit(1 if errs else 0)
PY
	rm -r -- "$tmp"
	return $rc
}

counts() {
	local built="${1:-}" field=now phases="${2:-}"
	[ "$built" = "--built" ] && field=built
	: "${FORK:?set FORK to a seedhammer checkout}" "${ME_WT:?set ME_WT to a mnemonic-engrave checkout}"
	: "${CARGO_TARGET_DIR:?set CARGO_TARGET_DIR (your own)}"
	local tmp
	tmp="$(mktemp -d)"
	(cd "$ME_WT" && PATH="$HOME/.cargo/bin:$PATH" cargo nextest list --locked --workspace) >"$tmp/me.txt" 2>"$tmp/me.err" ||
		{ tail -5 "$tmp/me.err"; exit 1; }
	for pkg in ./codex32/ ./sysw/ ./seal/ ./gui/ ./cmd/emu/; do
		for tags in - refugium; do
			local out tagflag=()
			out="$tmp/go$(printf '%s' "$pkg$tags" | tr -c 'a-z0-9' _).txt"
			[ "$tags" != - ] && tagflag=(-tags "$tags")
			(cd "$FORK" && PATH="$GO_BIN:$PATH" go test "${tagflag[@]}" -list '.*' "$pkg") >"$out" 2>&1 || true
		done
	done
	local rc=0
	PLAN="$PLAN" FIELD="$field" TMP="$tmp" PHASES="$phases" python3 -I - <<'PY' || rc=$?
import os, re, sys
plan = open(os.environ["PLAN"], encoding="utf-8").read()
tmp, field = os.environ["TMP"], os.environ["FIELD"]
phases = set(filter(None, os.environ["PHASES"].split(",")))
me = [l.split()[-1].split("::")[-1] for l in open(f"{tmp}/me.txt") if l.strip()]
def golist(pkg, tags):
    p = f"{tmp}/go" + re.sub(r"[^a-z0-9]", "_", pkg + tags) + ".txt"
    got = [l.strip() for l in open(p) if re.match(r"^Test\w+$", l.strip())]
    if not got:
        print(f"FAIL go test -list {pkg} {tags} listed nothing (build error?)"); sys.exit(1)
    return got
errs = 0; n = 0
for name, phase, repo, target, tags, now, built in re.findall(
        r"^ROW (\S+) \| (\S+) \| (me|sh) \| (\S+) \| (\S+) \| now=(\d+) \| built=(\d+)$", plan, re.M):
    if phases and phase not in phases:
        continue
    got = me.count(name) if repo == "me" else golist(target, tags).count(name)
    want = int(now if field == "now" else built)
    n += 1
    if got != want:
        errs += 1; print(f"FAIL ROW {name} ({repo} {target} {tags}): {got}, want {field}={want}")
for name, phase, repo, target, tags in re.findall(
        r"^CHANGE (\S+) \| (\S+) \| (me|sh) \| (\S+) \| (\S+) \|", plan, re.M):
    if phases and phase not in phases:
        continue
    got = me.count(name) if repo == "me" else golist(target, tags).count(name)
    deleted = "deleted;" in re.search(rf"^CHANGE {name} \|.*$", plan, re.M).group(0)
    want = 1 if field == "now" else (0 if deleted else 1)
    n += 1
    if got != want:
        errs += 1; print(f"FAIL CHANGE {name}: {got}, want {want}")
print(f"counts ({field}): {n} lines, {errs} failures")
sys.exit(1 if errs else 0)
PY
	rm -r -- "$tmp"
	return $rc
}

probes() {
	: "${FORK:?set FORK to a clean seedhammer checkout at the fork baseline}"
	if [ -n "$(git -C "$FORK" status --porcelain)" ]; then
		echo "probes: FORK is not clean" >&2; exit 2
	fi
	local p1="$FORK/seal/zz_s2probe_test.go" p2="$FORK/sysw/zz_s2probe_test.go" log
	log="$(mktemp)"
	trap 'rm -f -- "$p1" "$p2" "$log"' RETURN
	printf '%s\n' 'package seal' 'import ("encoding/hex"; "testing")' \
		'func TestS2Probe(t *testing.T) {' \
		'	h, _ := hex.DecodeString("4d4e454d424c4f4202010106000927c0feedfeedfeedfeedfeedfeedfeedfeedfacefacefacefacefaceface00000000000000f2")' \
		'	_, err := ParseHeader(h); t.Logf("PROBE seal-v2-header: %v", err) }' >"$p1"
	printf '%s\n' 'package sysw' 'import "testing"' \
		'func TestS2Probe(t *testing.T) {' \
		'	c := Classify("mr1q2qyyter2scsqzpl40veqyppqqxpxpgnxjakqqn88sqhs32rd4eysmyjsmg0770kkufgjegg")' \
		'	t.Logf("PROBE sysw-mr1-unknown: %v", c == ClassUnknown) }' >"$p2"
	(cd "$FORK" && PATH="$GO_BIN:$PATH" go test -count=1 -v -run '^TestS2Probe$' ./seal/ ./sysw/ &&
		PATH="$GO_BIN:$PATH" go test -count=1 -v -tags refugium -run '^TestRefugiumUnlockCodex32PlateIsTextOnly$' ./gui/) >"$log" 2>&1 || true
	local fail=0
	grep -q 'PROBE seal-v2-header: seal: unsupported format version 2' "$log" || { echo "FAIL probe seal-v2-header"; fail=1; }
	grep -q 'PROBE sysw-mr1-unknown: true' "$log" || { echo "FAIL probe sysw-mr1-unknown"; fail=1; }
	grep -q -- '--- PASS: TestRefugiumUnlockCodex32PlateIsTextOnly' "$log" || { echo "FAIL probe refugium-text-only"; fail=1; }
	grep -E 'PROBE|--- (PASS|FAIL)' "$log"
	echo "probes: $((3 - fail)) of 3 RED states seen"
	return $fail
}

selftest() {
	local tmp rc fails=0
	tmp="$(mktemp -d)"
	# 1. a quote that is not in the spec
	cp "$PLAN" "$tmp/p.md"; sed '2s/the record holds any ASCII capital letter/the record holds any capital/' "$COV" >"$tmp/c.tsv"
	PLAN="$tmp/p.md" COV="$tmp/c.tsv" "$0" static >"$tmp/1.log" 2>&1 && rc=0 || rc=$?
	[ $rc -ne 0 ] && grep -q 'quote not in S1' "$tmp/1.log" && echo "selftest 1 (bad quote): RED as required, rc=$rc" || { echo "selftest 1: gate did not fail"; fails=1; }
	# 2. a code fence in the plan
	{ cat "$PLAN"; printf '\n```\nx\n```\n'; } >"$tmp/p.md"
	PLAN="$tmp/p.md" COV="$COV" "$0" static >"$tmp/2.log" 2>&1 && rc=0 || rc=$?
	[ $rc -ne 0 ] && grep -q 'code fence' "$tmp/2.log" && echo "selftest 2 (code fence): RED as required, rc=$rc" || { echo "selftest 2: gate did not fail"; fails=1; }
	# 3. a wrong owed-vector tag
	sed 's/f00d80f68d3a83fbdef6c4f22c0e08b0/f00d80f68d3a83fbdef6c4f22c0e08b1/' "$PLAN" >"$tmp/p.md"
	PLAN="$tmp/p.md" COV="$COV" "$0" static >"$tmp/3.log" 2>&1 && rc=0 || rc=$?
	[ $rc -ne 0 ] && grep -q 'owed vector N-not-hex' "$tmp/3.log" && echo "selftest 3 (owed vector): RED as required, rc=$rc" || { echo "selftest 3: gate did not fail"; fails=1; }
	# 4. an owner naming no row
	sed '2s/TestSyswMrStringVectors$/TestNoSuchRow/' "$COV" >"$tmp/c.tsv"
	PLAN="$PLAN" COV="$tmp/c.tsv" "$0" static >"$tmp/4.log" 2>&1 && rc=0 || rc=$?
	[ $rc -ne 0 ] && grep -q "owner 'TestNoSuchRow'" "$tmp/4.log" && echo "selftest 4 (bad owner): RED as required, rc=$rc" || { echo "selftest 4: gate did not fail"; fails=1; }
	# 5. a missing FOLLOWUPS file: release must fail closed with rc 2
	mkdir -p "$tmp/empty/design"
	ME_WT="$tmp/empty" "$0" release >"$tmp/5.log" 2>&1 && rc=0 || rc=$?
	[ $rc -eq 2 ] && grep -q 'G-REL: no ' "$tmp/5.log" && echo "selftest 5 (missing FOLLOWUPS): RED as required, rc=$rc" || { echo "selftest 5: gate did not fail"; fails=1; }
	# 6. F-124's heading renamed: release must say not found, rc 1
	mkdir -p "$tmp/ren/design"
	printf '### F-124: x\n**Status:** CLOSED\n### F-123 x\n**Status:** CLOSED\n' >"$tmp/ren/design/FOLLOWUPS.md"
	ME_WT="$tmp/ren" "$0" release >"$tmp/6.log" 2>&1 && rc=0 || rc=$?
	[ $rc -eq 1 ] && grep -q 'G-REL: F-124 not found' "$tmp/6.log" && echo "selftest 6 (renamed heading): RED as required, rc=$rc" || { echo "selftest 6: gate did not fail"; fails=1; }
	# 7. counts --built ME-1 at the baselines reports exactly the 7 ME-1 ROW failures (needs FORK, ME_WT, CARGO_TARGET_DIR)
	if [ -n "${FORK:-}" ] && [ -n "${ME_WT:-}" ] && [ -n "${CARGO_TARGET_DIR:-}" ]; then
		"$0" counts --built ME-1 >"$tmp/7.log" 2>&1 && rc=0 || rc=$?
		[ $rc -ne 0 ] && grep -q 'counts (built): 10 lines, 7 failures' "$tmp/7.log" && echo "selftest 7 (counts --built ME-1): RED as required, rc=$rc" || { echo "selftest 7: unexpected counts"; fails=1; }
	else
		echo "selftest 7 skipped: set FORK, ME_WT, CARGO_TARGET_DIR"
	fi
	rm -r -- "$tmp"
	return $fails
}

release() {
	: "${ME_WT:?set ME_WT to a mnemonic-engrave checkout at the release head}"
	local f="$ME_WT/design/FOLLOWUPS.md" blocked=0 id st
	[ -f "$f" ] || { echo "G-REL: no $f"; return 2; }
	for id in F-124 F-123; do
		st="$(awk -v h="### $id " 'index($0,h)==1{f=1;next} f&&/^### /{exit} f&&/^\*\*Status:\*\*/{print; exit}' "$f")"
		case "$st" in
		'**Status:** CLOSED'*) ;;
		'') echo "G-REL: $id not found"; blocked=1 ;;
		*) echo "G-REL: $id OPEN"; blocked=1 ;;
		esac
	done
	[ $blocked -eq 0 ] && echo "G-REL: clear"
	return $blocked
}

case "$mode" in
static) static ;;
counts) counts "${2:-}" "${3:-}" ;;
probes) probes ;;
selftest) selftest ;;
release) release ;;
*) echo "usage: $0 static|counts [--built [PHASES]]|probes|selftest|release" >&2; exit 2 ;;
esac
