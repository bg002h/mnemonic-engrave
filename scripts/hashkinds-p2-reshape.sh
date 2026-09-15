#!/usr/bin/env bash
#
# hashkinds-p2-reshape.sh -- replay the finished `hashkinds-p2` tree as the six
# per-task commits IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md
# describes, gating every boundary from a clean checkout.
#
# WHY THIS IS A GATE AND NOT BOOKKEEPING. A task-by-task plan makes a claim at
# every boundary -- "after Task N the workspace resolves, builds, and its tests
# pass" -- and NOTHING that looks only at the final tree can check it. Not the
# block-vs-tree checker, not the test suite, not nine review rounds. Running it
# the first time found two defects all of those had missed:
#
#   * Task 2 bumped `ms-codec` to 0.10.0 while `crates/ms-cli/Cargo.toml` pinned
#     `ms-codec = "=0.9.0"`. `cargo metadata` itself fails -- so NO cargo command
#     runs at all -- for four consecutive boundaries, until Task 5b moves the
#     pin. That is why `cargo metadata` is checked FIRST below: a resolution
#     failure is invisible to a gate that starts at `cargo build`.
#   * Task 1's Files block listed three ms-codec paths, but its rename of
#     `digest` -> `digest_sha256` breaks five files including
#     `crates/ms-cli/src/cmd/hashlock.rs`. Only Step 4b's grep said so.
#
# THE CHECK THAT MAKES IT TRUSTWORTHY is the last one: the shaped branch's tree
# must be BYTE-IDENTICAL to the source branch's. That is what proves this
# re-ordered the work rather than rewrote it.
#
# WHAT IT DOES NOT DO. It does not verify the commit MESSAGES, and it does not
# check that each commit contains only what its task owns -- a file pulled a
# task too early still gates green if nothing depends on the difference (the
# corpus is the live example: pulled whole at Task 2 it reds Task 2, but a
# quieter mis-split might not). It also assumes the plan's task list; if the plan
# gains a task, this script must gain one too.
#
# Usage:  scripts/hashkinds-p2-reshape.sh [source-rev] [base-rev] [worktree]
set -euo pipefail

SRC="${1:-hashkinds-p2}"
BASE="${2:-7a0e96f}"
WT="${3:-/scratch/code/shibboleth/ms-worktrees/p2-shaped}"
MS="/scratch/code/shibboleth/ms-worktrees/hashkinds-p2"
export PATH=/home/bcg/.cargo/bin:$PATH

[ -d "$WT" ] || { echo "no such worktree: $WT" >&2; exit 2; }
cd "$WT"
SRC_SHA="$(git -C "$MS" rev-parse "$SRC")"
echo "reshaping $SRC ($SRC_SHA) from $BASE"

git checkout -q --detach "$BASE"
git branch -f hashkinds-p2-shaped "$BASE" >/dev/null
git checkout -q hashkinds-p2-shaped
git clean -qfd
git reset -q --hard "$BASE"

gate () {                       # $1 = task label
	local label="$1" meta tests clippy fmt
	meta=$(cargo metadata --locked --offline --format-version=1 >/dev/null 2>&1 \
		&& echo resolves || echo UNRESOLVABLE)
	if [ "$meta" = UNRESOLVABLE ]; then
		printf '%-34s %s\n' "$label" "UNRESOLVABLE -- no cargo command can run"
		return 1
	fi
	tests=$(cargo nextest run --locked --all-targets 2>&1 \
		| sed -e 's/\x1b\[[0-9;]*m//g' | grep -oE '[0-9]+ tests run: [0-9]+ passed' || echo "TESTS FAILED")
	clippy=$(cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | grep -c '^error' || true)
	fmt=$(cargo +1.95.0 fmt --all -- --check >/dev/null 2>&1 && echo clean || echo DIRTY)
	printf '%-34s %-12s %-30s clippy=%s fmt=%s\n' "$label" "$meta" "$tests" "$clippy" "$fmt"
}

take () { git -C "$WT" checkout "$SRC_SHA" -- "$@"; }

# ── Task 1 ────────────────────────────────────────────────────────────────
# hashlock.rs minus qr_text (Task 4 owns that); ripemd WITHOUT the version
# bump (Task 5b owns that, with the pin); and the rename collateral.
python3 - "$SRC_SHA" <<'PY'
import subprocess, sys
rev = sys.argv[1]
def show(r, p):
    return subprocess.run(["git", "show", f"{r}:{p}"], capture_output=True, text=True).stdout
c = show(rev, "crates/ms-codec/Cargo.toml").replace('version = "0.10.0"', 'version = "0.9.0"', 1)
open("crates/ms-codec/Cargo.toml", "w").write(c)
base, final = show("7a0e96f", "crates/ms-codec/src/hashlock.rs"), show(rev, "crates/ms-codec/src/hashlock.rs")
qr = lambda t: t[t.index("/// The QR text a hashlock PHRASE plate carries"):]
open("crates/ms-codec/src/hashlock.rs", "w").write(final.replace(qr(final), qr(base)))
PY
take crates/ms-codec/tests/hashlock_derivation.rs crates/ms-codec/tests/hashlock_repro.rs \
     crates/ms-cli/src/cmd/decode.rs crates/ms-cli/tests/hashlock_phrase_rule.rs vendor
git checkout -q "$BASE" -- Cargo.lock
sed -i '/^use ms_codec::hashlock::{$/,/^};$/ s/^    digest, /    digest_sha256, /' crates/ms-cli/src/cmd/hashlock.rs
sed -i 's/\blet h = digest(&d\.x);/let h = digest_sha256(\&d.x);/' crates/ms-cli/src/cmd/hashlock.rs
cargo metadata --offline --format-version=1 >/dev/null 2>&1 || true
git add -A && git commit -q -F - <<'MSG'
hashlock: four digest functions, the named dispatch, and the rename collateral

Plan Task 1. HashKind, DigestBytes, the four digest functions, HashKind::digest
as the one named dispatch, token() and opcode(). Three unit tests asserting
STRUCTURE where the KAT asserts values. No #[deprecated] alias.

The rename of `digest` reaches ms-cli; Task 1's Files block lists only ms-codec
and only Step 4b's grep says otherwise, so the collateral lands here -- a task
that leaves the workspace red is not a task boundary.
MSG
gate "1 digest functions + dispatch"

# ── Task 2 ────────────────────────────────────────────────────────────────
# The corpus splits: DERIVATION columns here, the qr_text array at Task 4.
python3 - "$SRC_SHA" <<'PY'
import json, subprocess, sys
rev = sys.argv[1]
P = "crates/ms-codec/tests/vectors/hashlock-v0.8.json"
show = lambda r: json.loads(subprocess.run(["git","show",f"{r}:{P}"],capture_output=True,text=True).stdout)
base, final = show("7a0e96f"), show(rev)
mixed = dict(final)
mixed["qr_text"] = base["qr_text"]
if "format" in base:
    mixed["format"] = base["format"]
open(P, "w").write(json.dumps(mixed, indent=2) + "\n")
PY
git add -A && git commit -q -F - <<'MSG'
hashlock: KAT rows for the three new kinds, computed in python3

Plan Task 2. Each derivation row gains *_h_hash256, *_h_ripemd160 and
*_h_hash160, computed OUTSIDE this crate -- a row generated by the code it
tests is not a KAT.

NO VERSION BUMP HERE, against an earlier revision of the plan: ms-cli pins
ms-codec exactly, so bumping the codec alone makes the workspace unresolvable
for four consecutive boundaries. The bump belongs with the pin, at Task 5b.
MSG
gate "2 KAT corpus rows"

# ── Task 3 ────────────────────────────────────────────────────────────────
take crates/ms-codec/tests/hashlock_kat.rs
git add -A && git commit -q -F - <<'MSG'
hashlock: the per-kind KAT, functions and dispatch, mutation-verified

Plan Task 3. All four functions AND the dispatch over 22 row/stem pairs.
No hex dev-dependency: the file defines its own hex() as a fold, because
map(format!).collect() trips clippy's format_collect under -D warnings.
MSG
gate "3 the KAT"

# ── Task 4 ────────────────────────────────────────────────────────────────
take crates/ms-codec/tests/vectors/hashlock-v0.8.json crates/ms-codec/tests/hashlock_qr_text.rs
python3 - "$SRC_SHA" <<'PY'
import subprocess, sys
rev = sys.argv[1]
P = "crates/ms-codec/src/hashlock.rs"
final = subprocess.run(["git","show",f"{rev}:{P}"],capture_output=True,text=True).stdout
cur = open(P).read()
qr = lambda t: t[t.index("/// The QR text a hashlock PHRASE plate carries"):]
open(P, "w").write(cur.replace(qr(cur), qr(final)))
PY
git add -A && git commit -q -F - <<'MSG'
hashlock: qr_text names the kind on its own line

Plan Task 4. SPEC §13.1: a plate read years later must name the hash it commits
to rather than leaving it to the md1 card stored SEPARATELY. Its own line, never
appended to method: -- H6 §6.5 pins that line at 73 characters.

Worst case re-keyed on ripemd160, the longest token: 210 hardened / 151 not,
against sha256's 207 / 148.
MSG
gate "4 qr_text names the kind"

# ── Task 5 ────────────────────────────────────────────────────────────────
take crates/ms-cli/src/cmd/hashlock.rs crates/ms-cli/tests/hashlock_kind.rs \
     crates/ms-cli/tests/gui_schema_emits_spec_v7_json.rs crates/ms-cli/tests/hashlock_outputs.rs
git add -A && git commit -q -F - <<'MSG'
ms hashlock: --kind, with a four-digest lookup when it is absent

Plan Task 5. The change is at the SOURCE, not the print sites: one
kind.digest(&d.x) feeds the stdout record, three --json keys and two card lines.

SPEC §13.4 is satisfied on a channel the relevant audience reads in every
kind-omitted cell -- the stderr listing, except under --json AND
--no-engraving-card together, the only pair pinned to exactly the advisory,
where it travels in the object instead.

Carries the card/notice boundary: above it is what --no-engraving-card may
suppress, below it is a hazard notice it may not, and every notice also carries
!(json && no_engraving_card) because that pair is a purity contract.
MSG
gate "5 ms hashlock --kind"

# ── Task 5b ───────────────────────────────────────────────────────────────
take crates/ms-cli/Cargo.toml CHANGELOG.md MIGRATION.md
sed -i 's/^version = "0.9.0"$/version = "0.10.0"/' crates/ms-codec/Cargo.toml
cargo metadata --offline --format-version=1 >/dev/null 2>&1 || true
git add -A && git commit -q -F - <<'MSG'
release: ms-codec 0.10.0 / ms-cli 0.19.0, with the migration record

Plan Task 5b. The version bump lives HERE, with the exact-version pin that
tracks it -- splitting them leaves the workspace unresolvable for four
consecutive task boundaries.

MIGRATION.md gains v0.9 -> v0.10 per RELEASE_PROCESS item 5, including the one
an upgrader most needs: a plate with NO hash: line means the kind is UNKNOWN,
not sha256.
MSG
gate "5b release records"

# ── the check that makes the reshape trustworthy ──────────────────────────
echo
if [ -z "$(git diff --stat "$SRC_SHA" HEAD)" ]; then
	echo "TREE IDENTICAL to $SRC ($SRC_SHA) -- this is a re-ordering, not a rewrite."
else
	echo "TREE DIFFERS from $SRC -- the reshape LOST OR ADDED something:"
	git diff --stat "$SRC_SHA" HEAD
	exit 1
fi
