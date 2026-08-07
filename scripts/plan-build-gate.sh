#!/usr/bin/env bash
#
# plan-build-gate.sh -- compile the Rust that lives inside the implementation
# plan, so a fold that does not build never reaches a reviewer.
#
# WHY THIS EXISTS
#   Plan A carries complete Rust in markdown. Every R0 reviewer extracts it into
#   a scratch crate and builds it; the controller did not, so transcription
#   defects survived from a fold until someone else's execution round hours
#   later -- and by then another fold was usually stacked on top. Round 3 of the
#   plan review found FIVE compile errors introduced by round 2's folds:
#   mismatched Result types in a match, .clone() on a slice, a generic that lost
#   inference. Every one is a `cargo build` away.
#
#   This is the missing feedback loop, not a replacement for review. Review finds
#   design defects; this finds E0308.
#
# WHAT IT DOES
#   Extracts the ```rust blocks for the NEW files (src/seal/*.rs,
#   tests/seal_cli.rs) into a scratch copy of the crate, then:
#     - builds and RUNS the seal lib tests
#     - COMPILE-CHECKS tests/seal_cli.rs (--no-run)
#     - clippy, both as the plan's stated gate and --all-targets (informational)
#
# WHAT IT DOES NOT DO
#   1. src/main.rs and src/validate.rs are MODIFICATIONS to existing files -- the
#      plan gives fragments (a match arm, an enum variant, a function to insert),
#      not whole files, so they cannot be assembled mechanically. Those still need
#      a reviewer's execution pass.
#   2. tests/seal_cli.rs is COMPILED BUT NOT RUN. It drives the `me` binary
#      through assert_cmd, and this scratch crate's binary is built from the
#      UNMODIFIED main.rs -- it has no `seal` subcommand, so every case would
#      fail at runtime for a reason that says nothing about the plan. Type
#      errors in the CLI tests are caught here; their ASSERTIONS are not
#      exercised. That is a reviewer's job.
#      (Until 2026-08-07 this file was extracted and then never compiled at all,
#      because steps 5-7 were all `--lib`. Round 4's fold added a test here.)
#
# Usage:  scripts/plan-build-gate.sh [path/to/plan.md]
set -euo pipefail

PLAN="${1:-design/IMPLEMENTATION_PLAN_encrypted_payload_hostA.md}"
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="${TMPDIR:-/tmp}/plan-build-gate"

[ -f "$REPO/$PLAN" ] || { echo "no plan at $REPO/$PLAN" >&2; exit 2; }

echo "== 1 -- scratch copy of the crate =="
rm -rf "$WORK"
mkdir -p "$WORK"
cp -r "$REPO/crates" "$REPO/Cargo.toml" "$WORK/" 2>/dev/null
[ -f "$REPO/Cargo.lock" ] && cp "$REPO/Cargo.lock" "$WORK/"
echo "   $WORK"

echo "== 2 -- extract the plan's Rust into the seal module =="
python3 - "$REPO/$PLAN" "$WORK" <<'PY'
import re, sys, os, collections
plan, work = sys.argv[1], sys.argv[2]
lines = open(plan).read().split("\n")

# Track the current target file and whether the anchor said "Prepend".
anchor = re.compile(r'(create|prepend to|add to|add above|in)\s+`([^`]*\.rs)`', re.I)
blocks = collections.OrderedDict()   # path -> list[(prepend?, code)]
cur, prepend = None, False
i = 0
while i < len(lines):
    m = anchor.search(lines[i])
    if m:
        verb, path = m.group(1), m.group(2)
        # normalise: the plan writes both `crates/me-cli/src/...` and `src/...`
        p = path.split("crates/me-cli/")[-1]
        # only the NEW files are mechanically assemblable
        if p.startswith("src/seal/") or p == "tests/seal_cli.rs":
            cur, prepend = p, verb.lower().startswith("prepend")
        else:
            cur = None
    if lines[i].startswith("```rust") and cur:
        i += 1
        buf = []
        while i < len(lines) and not lines[i].startswith("```"):
            buf.append(lines[i]); i += 1
        code = "\n".join(buf)
        # A block opening with `//!` is a file header: inner doc comments must
        # lead the file, so it always goes first regardless of the anchor verb.
        blocks.setdefault(cur, []).append((prepend or code.lstrip().startswith("//!"), code))
    i += 1

# mod.rs's `pub mod X;` lines live in prose, not in fences — synthesise them
# from whichever seal files were actually extracted.
mods = sorted({os.path.basename(p)[:-3] for p in blocks
               if p.startswith("src/seal/") and not p.endswith("mod.rs")})
decl = "\n".join(f"pub mod {m};" for m in mods) if mods else None
if decl:
    print(f"   synthesised mod.rs decls: {', '.join(mods)}")

for p, parts in blocks.items():
    dest = os.path.join(work, "crates/me-cli", p)
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    head = [c for pre, c in parts if pre]
    tail = [c for pre, c in parts if not pre]
    body = head + tail
    if p.endswith("seal/mod.rs") and mods:
        already = set(re.findall(r'pub mod (\w+);', "\n".join(body)))
        need = [m for m in mods if m not in already]
        if need:
            at = 1 if body and body[0].lstrip().startswith("//!") else 0
            body.insert(at, "\n".join(f"pub mod {m};" for m in need))
    open(dest, "w").write("\n\n".join(body) + "\n")
    print(f"   {p}: {len(parts)} block(s), {len(head)} prepended")
PY

echo "== 2b -- apply the validate.rs extraction the plan specifies =="
python3 - "$REPO/$PLAN" "$WORK" <<'PY2'
import re, sys
plan, work = sys.argv[1], sys.argv[2]
txt = open(plan).read()
m = re.search(r'(/// First interior separator.*?\n\})', txt, re.S)
if not m:
    print("   WARN: first_noncanonical block not found"); sys.exit(0)
vp = f"{work}/crates/me-cli/src/validate.rs"
v = open(vp).read()
if "fn first_noncanonical" not in v:
    v = v.replace("/// Validate one md1/mk1 string", m.group(1) + "\n\n/// Validate one md1/mk1 string", 1)
    open(vp, "w").write(v)
    print("   first_noncanonical added to validate.rs")
PY2

echo "== 3 -- dependencies from the plan's Cargo stanza =="
python3 - "$REPO/$PLAN" "$WORK" <<'PY'
import re, sys
plan, work = sys.argv[1], sys.argv[2]
txt = open(plan).read()
m = re.search(r'```toml\n(.*?)```', txt, re.S)
deps = m.group(1).strip() if m else ""
ct = f"{work}/crates/me-cli/Cargo.toml"
s = open(ct).read()
if deps and "aes-gcm" not in s:
    s = s.replace("[dependencies]", "[dependencies]\n" + deps, 1)
if "tempfile" not in s:
    s = s.replace("[dev-dependencies]", "[dev-dependencies]\ntempfile = \"3\"", 1)
open(ct, "w").write(s)
print("   deps merged")
PY

echo "== 4 -- wire the module =="
LIB="$WORK/crates/me-cli/src/lib.rs"
grep -q "pub mod seal;" "$LIB" || sed -i 's|^pub mod validate;|pub mod seal;\npub mod validate;|' "$LIB"
grep -q "pub mod seal;" "$LIB" && echo "   lib.rs declares seal"

echo "== 5 -- BUILD =="
cd "$WORK"
if cargo build -p mnemonic-engrave --lib 2>&1 | tee /tmp/plan-gate-build.log | tail -25; then
  echo "   PASS: the plan's seal module compiles"
else
  echo "   FAIL: see /tmp/plan-gate-build.log"; exit 1
fi

echo "== 6 -- TEST (seal lib) =="
cargo test -p mnemonic-engrave --lib seal 2>&1 | tail -15

echo "== 6b -- COMPILE-CHECK tests/seal_cli.rs (not run: see header) =="
if [ -f "$WORK/crates/me-cli/tests/seal_cli.rs" ]; then
  if cargo test -p mnemonic-engrave --test seal_cli --no-run 2>&1 \
       | tee /tmp/plan-gate-cli.log | tail -15; then
    echo "   PASS: the CLI tests compile"
  else
    echo "   FAIL: see /tmp/plan-gate-cli.log"; exit 1
  fi
else
  echo "   SKIP: no tests/seal_cli.rs extracted"
fi

echo "== 7 -- CLIPPY (the plan's stated gate) =="
cargo clippy -p mnemonic-engrave --lib -- -D warnings 2>&1 | tail -10 \
  && echo "   clippy clean"

echo "== 7b -- CLIPPY --all-targets (informational, does not gate) =="
cargo clippy -p mnemonic-engrave --all-targets 2>&1 \
  | grep -E '^(warning|error)' | sort | uniq -c | sort -rn | head -10 \
  || echo "   none"

echo
echo "== RESULT =="
echo "   NOT covered: src/main.rs and src/validate.rs arrive as fragments, so a"
echo "   reviewer's execution pass still owns them; and tests/seal_cli.rs is"
echo "   COMPILED BUT NOT RUN (its binary lacks the seal subcommand here), so its"
echo "   assertions are unexercised. Type errors caught; behaviour not."
