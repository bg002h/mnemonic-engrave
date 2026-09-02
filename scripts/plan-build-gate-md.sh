#!/usr/bin/env bash
# plan-build-gate-md.sh -- compile and run the Rust that lives inside a
# descriptor-mnemonic implementation plan (the wallet-policy COMPOSER's Stage 0),
# so a fold that does not build never reaches a reviewer.
#
# WHY THIS EXISTS. plan-build-gate.sh is hard-wired to me-cli's `src/seal/*`
# files and refuses (exit 3) on any other plan; it has no path to
# descriptor-mnemonic at all. The composer's first stage adds a `compose` module
# to md-codec and a `compose` subcommand to md-cli in a DIFFERENT repo, and the
# standing rule is that a plan with executable content is built before it is
# reviewed. Same shape as plan-build-gate.sh, different crate and anchors.
#
# WHAT IT DOES
#   1. Scratch copy of /scratch/code/shibboleth/descriptor-mnemonic (crates +
#      Cargo.toml + Cargo.lock), with CARGO_TARGET_DIR kept OUTSIDE the copy so
#      dependency builds are cached across runs and /tmp (a 32 GB tmpfs) is not
#      filled by a fresh target/ per run.
#   2. Extracts every ```rust block that follows an anchor naming one of the
#      NEW files this plan creates:
#        crates/md-codec/src/compose.rs   crates/md-codec/src/compose/*.rs
#        crates/md-codec/tests/compose_*.rs
#        crates/md-cli/src/cmd/compose.rs crates/md-cli/tests/cli_compose*.rs
#      Anchor grammar (same as plan-build-gate.sh, plus Replace): a line
#      containing `Create <path>`, `Prepend to <path>`, `Add to <path>`,
#      `In <path>` or `Replace <path>` with the path in backticks; the NEXT
#      ```rust fence is that file's content. Several blocks for one path are
#      concatenated in plan order; a block that opens with `//!` goes first;
#      a `Replace` block DISCARDS every earlier block for that path, which is
#      how a TDD plan's stub file (task N) gives way to its full file (task
#      N+1) without the two being concatenated into a redefinition error. Blocks anchored on EXISTING files (lib.rs,
#      main.rs, test_vectors.rs) are fragments and are NOT assembled -- they
#      need a reviewer's execution pass, and this script says so at the end.
#   3. Synthesises `pub mod compose;` into md-codec's lib.rs and
#      `pub mod compose;` into md-cli's cmd/mod.rs when those files were
#      extracted (the plan states the one-line edits in prose).
#   4. cargo build -p md-codec --all-targets; cargo nextest run -p md-codec
#      -E 'test(compose)'; cargo clippy -p md-codec --all-targets -- -D warnings;
#      cargo test -p md-cli --no-run (compile-check the CLI tests; their
#      assertions run only against a real `md` with the subcommand wired, which
#      needs the main.rs fragment a reviewer applies).
#
# EXTRACTING NOTHING IS A FAILURE, NOT A PASS (exit 3), for the reason
# plan-build-gate.sh records: a gate that cannot see its input must say so.
#
# Usage:  scripts/plan-build-gate-md.sh design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md
set -euo pipefail
PLAN="${1:?plan path required}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="${MD_REPO:-/scratch/code/shibboleth/descriptor-mnemonic}"
WORK="${TMPDIR:-/tmp}/plan-build-gate-md"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${TMPDIR:-/tmp}/plan-build-gate-md-target}"
[ -f "$HERE/$PLAN" ] || [ -f "$PLAN" ] || { echo "no plan at $PLAN" >&2; exit 2; }
[ -f "$HERE/$PLAN" ] && PLAN="$HERE/$PLAN"
echo "== 1 -- scratch copy of descriptor-mnemonic =="
rm -rf "$WORK"; mkdir -p "$WORK"
cp -r "$SRC/crates" "$SRC/Cargo.toml" "$WORK/"; [ -f "$SRC/Cargo.lock" ] && cp "$SRC/Cargo.lock" "$WORK/"
echo "   $WORK  (target: $CARGO_TARGET_DIR)"
echo "== 2 -- extract the plan's Rust =="
python3 - "$PLAN" "$WORK" <<'PY'
import re, sys, os, collections
plan, work = sys.argv[1], sys.argv[2]
lines = open(plan).read().split("\n")
anchor = re.compile(r'\b(create|prepend to|add to|in|replace)\s+`([^`]*\.rs)`', re.I)
ok = re.compile(r'^crates/md-codec/src/compose(/[A-Za-z0-9_]+)?\.rs$|^crates/md-codec/tests/compose_[A-Za-z0-9_]+\.rs$|^crates/md-cli/src/cmd/compose\.rs$|^crates/md-cli/tests/cli_compose[A-Za-z0-9_]*\.rs$')
blocks = collections.OrderedDict(); cur = None; prepend = False; replace = False; fragments = set()
i = 0
while i < len(lines):
    m = anchor.search(lines[i])
    if m:
        verb, path = m.group(1), m.group(2)
        if ok.match(path): cur, prepend, replace = path, verb.lower().startswith("prepend"), verb.lower() == "replace"
        else:
            cur = None
            if path.endswith(".rs"): fragments.add(path)
    if lines[i].startswith("```rust") and cur:
        i += 1; buf = []
        while i < len(lines) and not lines[i].startswith("```"): buf.append(lines[i]); i += 1
        code = "\n".join(buf)
        if replace:
            dropped = len(blocks.get(cur, []))
            blocks[cur] = []
            if dropped: print("   replaced %s (dropped %d earlier block%s)" % (cur, dropped, "" if dropped==1 else "s"))
            replace = False
        blocks.setdefault(cur, []).append((prepend or code.lstrip().startswith("//!"), code))
    i += 1
if not blocks:
    sys.stderr.write("\nplan-build-gate-md: EXTRACTED NOTHING from %s\n  Recognised anchors: crates/md-codec/src/compose*.rs, crates/md-codec/tests/compose_*.rs,\n  crates/md-cli/src/cmd/compose.rs, crates/md-cli/tests/cli_compose*.rs.\n  Refusing rather than reporting a pass on an empty extraction.\n" % plan)
    sys.exit(3)
for path, parts in blocks.items():
    parts.sort(key=lambda t: not t[0])
    full = os.path.join(work, path); os.makedirs(os.path.dirname(full), exist_ok=True)
    open(full, "w").write("\n\n".join(c for _, c in parts) + "\n")
    print("   wrote %s (%d block%s)" % (path, len(parts), "" if len(parts)==1 else "s"))
# synthesise the one-line module registrations the plan states in prose
lib = os.path.join(work, "crates/md-codec/src/lib.rs")
if any(p.startswith("crates/md-codec/src/compose") for p in blocks):
    s = open(lib).read()
    if "pub mod compose;" not in s:
        s = s.replace("pub mod codex32;", "pub mod codex32;\npub mod compose;", 1); open(lib, "w").write(s); print("   registered pub mod compose in md-codec lib.rs")
    if os.path.isdir(os.path.join(work, "crates/md-codec/src/compose")) and os.path.exists(os.path.join(work, "crates/md-codec/src/compose.rs")):
        sys.stderr.write("plan-build-gate-md: both compose.rs and compose/ exist; pick one layout\n"); sys.exit(4)
cmdmod = os.path.join(work, "crates/md-cli/src/cmd/mod.rs")
if "crates/md-cli/src/cmd/compose.rs" in blocks and os.path.exists(cmdmod):
    s = open(cmdmod).read()
    if "pub mod compose;" not in s:
        open(cmdmod, "w").write(s.rstrip("\n") + "\npub mod compose;\n"); print("   registered pub mod compose in md-cli cmd/mod.rs")
if fragments:
    print("   NOT assembled (fragments of existing files; reviewer's execution pass): " + ", ".join(sorted(fragments)))
PY
cd "$WORK"
echo "== 3 -- build md-codec (all targets) =="
cargo build -p md-codec --all-targets --locked 2>&1 | tail -3
echo "== 4 -- run the compose tests =="
if command -v cargo-nextest >/dev/null 2>&1; then cargo nextest run -p md-codec --locked -E 'test(compose)' 2>&1 | tail -6; else cargo test -p md-codec --locked compose 2>&1 | tail -6; fi
echo "== 5 -- clippy md-codec =="
cargo clippy -p md-codec --all-targets --locked -- -D warnings 2>&1 | tail -3
echo "== 6 -- compile-check md-cli (tests --no-run; the compose subcommand's main.rs arm is a fragment) =="
cargo test -p md-cli --locked --no-run 2>&1 | tail -3
echo "== NOT covered: main.rs/lib.rs/test_vectors.rs fragments; md-cli test ASSERTIONS (need the wired binary); the Go port. =="
