#!/usr/bin/env bash
# liana-live-gate.sh -- SPEC_liana_unspendable_internal_key.md §8.8, as a
# command anyone can re-run: drive Liana's OWN importer
# (`LianaDescriptor::from_str`, the entry point behind the GUI's "Import the
# wallet") over every vendored Liana-unspendable case plus controls, and diff
# the result against the committed expectation.
#
#   scripts/liana-live-gate.sh                  # exit 0 = Liana agrees with the record
#   scripts/liana-live-gate.sh --update         # re-record the expectation (review the diff!)
#
# Environment:
#   LIANA_CHECKOUT     a clone of https://github.com/wizardsardine/liana at the
#                      expectation's tag (default: /scratch/code/shibboleth/.tmp/fable-liana-src-v15)
#   LIANA_GATE_TARGET  cargo target dir for the harness build
#                      (default: harnesses/liana/target/gate-build; NOT /tmp,
#                      which is a small tmpfs on the author's box)
#
# NEVER A SILENT PASS (plan Task 5 Step 4, R0 I-7). §8.8 is REQUIRED: a missing
# checkout, a checkout at another tag, a failed build or a failed control is a
# loud non-zero exit naming the fix. The stage may not close on a skip.
#
# THE LIANA PATH. `harnesses/liana/Cargo.toml` takes Liana as a Cargo PATH
# dependency, so an environment override cannot reach it. This gate therefore
# builds a COPY of the harness whose Cargo.toml it rewrites to point at
# $LIANA_CHECKOUT/liana (the v15.0 workspace-member layout), and asserts the
# rewrite happened. The committed Cargo.toml is left untouched.
#
# THE PROBE RULE (R0 C-3, the whole lesson of that review). Liana emits ONE
# message -- "Descriptor is not compatible with a Liana spending policy." --
# for at least three causes: a genuine policy refusal, an internal key that
# is not the recipe over THESE leaves, and a raw NUMS internal key
# (analysis.rs:596-600). A control cannot separate them: it carries its own
# correct internal key and passes while a probe fails for a key reason that
# reads as a policy reason. So before sending, for EVERY probe this gate
# recomputes the Liana unspendable xpub over that probe's own leaf set (the
# harness's `unspendable` subcommand, Liana's recipe over Liana's own
# miniscript types) and asserts the descriptor carries it. The one record
# deliberately exempt is the stale-key control, whose job is to prove the
# rule can fire: the gate asserts its key does NOT match.
#
# CONTROLS. A run is VOID (exit 3) unless the accept control is accepted AND
# the stale-key control is refused -- a harness that accepts everything, or
# refuses everything, cannot pass.
#
# INPUTS AND EXPECTATION: design/evidence/f449-stage2/
#   liana-live-gate-in.jsonl       {"name","variant","desc","role"} per line;
#                                  role = vendored | control-accept | control-stale-key
#   liana-live-gate-expected.jsonl line 1: {"liana_tag","liana_commit",...};
#                                  then the harness's output, one line per input
# Every `vendored` record must be byte-identical to the
# design/evidence/composer-fable-r0/fable-liana-parse-in.jsonl record of the
# same name with variant liana-unspendable-xpub, and the vendored SET must equal
# that file's liana-unspendable-xpub set -- the same two JSONLs
# descriptor-mnemonic's scripts/vendor-liana-evidence.sh reads, so this gate
# runs exactly the cases md vendors, no more and no fewer.
set -euo pipefail
cd "$(dirname "$0")/.."
repo=$(pwd)

update=0
case "${1:-}" in
  "") ;;
  --update) update=1 ;;
  *) echo "usage: $0 [--update]" >&2; exit 2 ;;
esac

ev=design/evidence/f449-stage2
gate_in=$ev/liana-live-gate-in.jsonl
expected=$ev/liana-live-gate-expected.jsonl
composer_in=design/evidence/composer-fable-r0/fable-liana-parse-in.jsonl
want_tag=v15.0

checkout=${LIANA_CHECKOUT:-/scratch/code/shibboleth/.tmp/fable-liana-src-v15}
if [[ ! -f "$checkout/liana/Cargo.toml" ]]; then
  cat >&2 <<EOF
liana-live-gate: FAIL -- no Liana $want_tag checkout at $checkout (need $checkout/liana/Cargo.toml).
  SPEC §8.8 is REQUIRED; this gate never skips. Fetch it with:
    git clone https://github.com/wizardsardine/liana "$checkout"
    git -C "$checkout" checkout $want_tag
  or point LIANA_CHECKOUT at an existing clone.
EOF
  exit 1
fi
tag=$(git -C "$checkout" describe --tags --exact-match 2>/dev/null || true)
commit=$(git -C "$checkout" rev-parse HEAD)
if [[ "$tag" != "$want_tag" ]]; then
  echo "liana-live-gate: FAIL -- $checkout is at '${tag:-<untagged>}' ($commit), not $want_tag." >&2
  echo "  The expectation was measured at $want_tag; run: git -C \"$checkout\" checkout $want_tag" >&2
  exit 1
fi
if [[ -n "$(git -C "$checkout" status --porcelain --untracked-files=no)" ]]; then
  echo "liana-live-gate: FAIL -- $checkout has local modifications; the verdict would not be $want_tag's." >&2
  exit 1
fi

# Build a copy of the harness pointed at this checkout.
target=${LIANA_GATE_TARGET:-$repo/harnesses/liana/target/gate-build}
src=$repo/harnesses/liana/target/gate-src
rm -rf "$src"; mkdir -p "$src"
cp -r harnesses/liana/src "$src/src"
python3 - harnesses/liana/Cargo.toml "$src/Cargo.toml" "$checkout/liana" <<'PY'
import re, sys
src, dst, path = sys.argv[1:4]
s = open(src).read()
new, n = re.subn(r'(?m)^liana = \{ path = "[^"]*" \}$', f'liana = {{ path = "{path}" }}', s)
if n != 1:
    sys.exit(f"liana-live-gate: FAIL -- expected exactly one `liana = {{ path = ... }}` line in {src}, found {n}")
open(dst, "w").write(new)
PY
grep -qF "liana = { path = \"$checkout/liana\" }" "$src/Cargo.toml" \
  || { echo "liana-live-gate: FAIL -- Cargo.toml rewrite did not take" >&2; exit 1; }
echo "liana-live-gate: building harness against $want_tag ($commit)" >&2
CARGO_TARGET_DIR="$target" cargo build --quiet --manifest-path "$src/Cargo.toml" >&2
harness=$target/debug/liana-harness
[[ -x "$harness" ]] || { echo "liana-live-gate: FAIL -- no harness binary at $harness" >&2; exit 1; }

work=$(mktemp -d "$repo/harnesses/liana/target/gate-run.XXXXXX")
trap 'rm -rf "$work"' EXIT

# Inputs: vendored set check, then the probe rule.
python3 - "$gate_in" "$composer_in" "$work" <<'PY'
import json, sys
gate_in, composer_in, work = sys.argv[1:4]
recs = [json.loads(l) for l in open(gate_in) if l.strip()]
roles = {"vendored", "control-accept", "control-stale-key"}
for r in recs:
    if r.get("role") not in roles:
        sys.exit(f"liana-live-gate: FAIL -- {r.get('name')}: role {r.get('role')!r} not in {sorted(roles)}")
for role in ("control-accept", "control-stale-key"):
    if sum(r["role"] == role for r in recs) != 1:
        sys.exit(f"liana-live-gate: FAIL -- need exactly one {role} record")
comp = {}
for l in open(composer_in):
    if l.strip():
        c = json.loads(l)
        if c.get("variant") == "liana-unspendable-xpub":
            comp[c["name"]] = c
vend = {r["name"]: r for r in recs if r["role"] == "vendored"}
if set(vend) != set(comp):
    sys.exit("liana-live-gate: FAIL -- vendored set drifted from composer-fable-r0's "
             f"liana-unspendable-xpub set: gate-only={sorted(set(vend)-set(comp))} "
             f"composer-only={sorted(set(comp)-set(vend))}")
for n, r in vend.items():
    if r["desc"] != comp[n]["desc"] or r["variant"] != comp[n]["variant"]:
        sys.exit(f"liana-live-gate: FAIL -- {n}: gate input differs from composer-fable-r0's record")
with open(f"{work}/probes.txt", "w") as fh:
    for r in recs:
        fh.write(r["desc"].split("#")[0] + "\n")
with open(f"{work}/parse-in.jsonl", "w") as fh:
    for r in recs:
        fh.write(json.dumps({"name": r["name"], "variant": r["variant"], "desc": r["desc"]}) + "\n")
print(f"liana-live-gate: {len(vend)} vendored cases + 2 controls", file=sys.stderr)
PY

"$harness" unspendable < "$work/probes.txt" > "$work/recomputed.txt"
python3 - "$gate_in" "$work/recomputed.txt" <<'PY'
import json, re, sys
gate_in, recomputed = sys.argv[1:3]
recs = [json.loads(l) for l in open(gate_in) if l.strip()]
rec = [l.strip() for l in open(recomputed) if l.strip()]
if len(rec) != len(recs):
    sys.exit(f"liana-live-gate: FAIL -- recomputed {len(rec)} internal keys for {len(recs)} probes")
first_key = lambda d: re.match(r"tr\(([^/,]+)", d).group(1)
for r, d in zip(recs, rec):
    have, want = first_key(r["desc"]), first_key(d)
    if r["role"] == "control-stale-key":
        if have == want:
            sys.exit(f"liana-live-gate: FAIL -- {r['name']}: the stale-key control's key MATCHES "
                     "the recipe, so it cannot show the probe rule firing")
    elif have != want:
        sys.exit(f"liana-live-gate: FAIL (probe rule, R0 C-3) -- {r['name']}: internal key {have} "
                 f"is not Liana's recipe over this probe's own leaves ({want}). A refusal of this "
                 "probe would be a KEY refusal wearing a policy refusal's words.")
print(f"liana-live-gate: probe rule holds for {len(recs) - 1} probes; stale-key control differs as it must",
      file=sys.stderr)
PY

"$harness" parse < "$work/parse-in.jsonl" > "$work/out.jsonl"

if ! python3 - "$gate_in" "$work/out.jsonl" <<'PY'
import json, sys
gate_in, out = sys.argv[1:3]
roles = {json.loads(l)["name"]: json.loads(l)["role"] for l in open(gate_in) if l.strip()}
res = [json.loads(l) for l in open(out) if l.strip()]
for r in res:
    role = roles[r["name"]]
    if role == "control-accept" and not r["ok"]:
        sys.exit(f"liana-live-gate: VOID -- accept control {r['name']} was refused: {r.get('error')}")
    if role == "control-stale-key" and r["ok"]:
        sys.exit(f"liana-live-gate: VOID -- stale-key control {r['name']} was ACCEPTED")
PY
then
  exit 3
fi

meta=$(python3 -c 'import json,sys; print(json.dumps({"liana_tag":sys.argv[1],"liana_commit":sys.argv[2],"entry_point":"LianaDescriptor::from_str","harness":"harnesses/liana (parse)","inputs":sys.argv[3]}, sort_keys=True))' "$want_tag" "$commit" "$gate_in")

if [[ $update -eq 1 ]]; then
  { echo "$meta"; cat "$work/out.jsonl"; } > "$expected"
  echo "liana-live-gate: wrote $expected -- review and commit the diff" >&2
  exit 0
fi

[[ -f "$expected" ]] || { echo "liana-live-gate: FAIL -- no expectation at $expected (run with --update once, then review)" >&2; exit 1; }
if ! diff -u <(tail -n +2 "$expected") "$work/out.jsonl" >&2; then
  echo "liana-live-gate: FAIL -- Liana $want_tag ($commit) disagrees with the committed expectation" >&2
  exit 1
fi
exp_commit=$(head -1 "$expected" | python3 -c 'import json,sys; print(json.load(sys.stdin)["liana_commit"])')
echo "liana-live-gate: PASS -- $(wc -l < "$work/out.jsonl") verdicts match; Liana $want_tag at $commit (expectation recorded at $exp_commit)"
