#!/usr/bin/env bash
# plan-build-gate-me.sh -- compile and run the Rust that lives inside a
# mnemonic-engrave (me-cli) implementation plan, so a fold that does not build
# never reaches a reviewer.
#
# WHY THIS EXISTS. plan-build-gate.sh is hard-wired to me-cli's `src/seal/*`
# files and plan-build-gate-md.sh to descriptor-mnemonic's `compose`. The
# composer's Stage 1 adds NEW files under `crates/me-cli/src/sysw/` and tests
# under `crates/me-cli/tests/`, in THIS repo. Same shape as the md gate,
# different source tree and anchors.
#
# WHAT IT DOES
#   1. Scratch copy of THIS repo's `crates/`, `Cargo.toml`, `Cargo.lock` and any
#      toolchain/cargo/lint config, with CARGO_TARGET_DIR kept OUTSIDE the copy.
#      This repo has no rust-toolchain.toml; CI pins RUST_TOOLCHAIN in
#      .github/workflows/release.yml (1.85.0 today), so the gate reads that
#      value and builds under `RUSTUP_TOOLCHAIN=<it>` when rustup has it,
#      printing `rustc --version` from inside the copy either way. (Lesson from
#      the md gate: a copy on rustup's default nightly reported lints the repo's
#      pinned compiler does not have.)
#   2. Extracts every ```rust block that follows an anchor naming one of the
#      NEW files a plan may create:
#        crates/me-cli/src/sysw/composer_*.rs
#        crates/me-cli/tests/sysw_composer*.rs
#      Anchor grammar (same as the sibling gates): a line containing
#      `Create <path>`, `Prepend to <path>`, `Add to <path>`, `In <path>` or
#      `Replace <path>` with the path in backticks; the NEXT ```rust fence is
#      that file's content. Several blocks for one path are concatenated in plan
#      order; a block opening with `//!` goes first; `Replace` discards every
#      earlier block for that path. Blocks anchored on EXISTING files (sysw/mod.rs,
#      sysw/record.rs, main.rs, coverage.rs) are fragments and are NOT assembled.
#   3. Synthesises `pub mod composer_records;` (and any other extracted
#      `sysw/composer_<x>.rs` as `pub mod composer_<x>;`) into
#      `crates/me-cli/src/sysw/mod.rs` after `pub mod coverage;`.
#   4. cargo build -p mnemonic-engrave --all-targets --locked; cargo nextest run
#      -p mnemonic-engrave --locked --no-fail-fast -E 'binary(/^sysw_composer/)
#      | test(/composer_records/)'; cargo clippy -p mnemonic-engrave
#      --all-targets --locked -- -D warnings. No pinned red: a plan that needs
#      one states it and this script is amended, not argued with.
#
# EXTRACTING NOTHING IS A FAILURE, NOT A PASS (exit 3).
#
# Usage:  scripts/plan-build-gate-me.sh design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md
set -euo pipefail
PLAN="${1:?plan path required}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="${ME_REPO:-$HERE}"
WORK="${TMPDIR:-/tmp}/plan-build-gate-me"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${TMPDIR:-/tmp}/plan-build-gate-me-target}"
[ -f "$HERE/$PLAN" ] || [ -f "$PLAN" ] || { echo "no plan at $PLAN" >&2; exit 2; }
[ -f "$HERE/$PLAN" ] && PLAN="$HERE/$PLAN"
echo "== 1 -- scratch copy of mnemonic-engrave =="
rm -rf "$WORK"; mkdir -p "$WORK"
cp -r "$SRC/crates" "$SRC/Cargo.toml" "$WORK/"; [ -f "$SRC/Cargo.lock" ] && cp "$SRC/Cargo.lock" "$WORK/"
for f in rust-toolchain.toml rust-toolchain clippy.toml rustfmt.toml .rustfmt.toml; do [ -f "$SRC/$f" ] && cp "$SRC/$f" "$WORK/"; done
[ -d "$SRC/.cargo" ] && cp -r "$SRC/.cargo" "$WORK/"
# The workspace may reference path deps or submodule crates the copy does not
# carry; the sysw plan touches me-cli only, and me-cli's deps are crates.io +
# the in-tree mnemonic-io-lib, which `crates/` contains.
PIN="$(grep -E '^\s*RUST_TOOLCHAIN:' "$SRC/.github/workflows/release.yml" 2>/dev/null | head -1 | sed -E "s/.*RUST_TOOLCHAIN:\s*'?([0-9.]+)'?.*/\1/")"
if [ -n "$PIN" ] && rustup toolchain list 2>/dev/null | grep -q "^$PIN-"; then export RUSTUP_TOOLCHAIN="$PIN"; fi
echo "   $WORK  (target: $CARGO_TARGET_DIR)"
echo "   toolchain: $(cd "$WORK" && rustc --version 2>/dev/null)  (CI pin: ${PIN:-none found})"
echo "== 2 -- extract the plan's Rust =="
python3 - "$PLAN" "$WORK" <<'PY'
import re, sys, os, collections
plan, work = sys.argv[1], sys.argv[2]
lines = open(plan).read().split("\n")
anchor = re.compile(r'\b(create|prepend to|add to|in|replace)\s+`([^`]*\.rs)`', re.I)
ok = re.compile(r'^crates/me-cli/src/sysw/composer_[A-Za-z0-9_]+\.rs$|^crates/me-cli/tests/sysw_composer[A-Za-z0-9_]*\.rs$')
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
    sys.stderr.write("\nplan-build-gate-me: EXTRACTED NOTHING from %s\n  Recognised anchors: crates/me-cli/src/sysw/composer_*.rs, crates/me-cli/tests/sysw_composer*.rs.\n  Refusing rather than reporting a pass on an empty extraction.\n" % plan)
    sys.exit(3)
for path, parts in blocks.items():
    parts.sort(key=lambda t: not t[0])
    full = os.path.join(work, path); os.makedirs(os.path.dirname(full), exist_ok=True)
    open(full, "w").write("\n\n".join(c for _, c in parts) + "\n")
    print("   wrote %s (%d block%s)" % (path, len(parts), "" if len(parts)==1 else "s"))
modrs = os.path.join(work, "crates/me-cli/src/sysw/mod.rs")
mods = sorted({os.path.basename(p)[:-3] for p in blocks if p.startswith("crates/me-cli/src/sysw/composer_")})
if mods and os.path.exists(modrs):
    s = open(modrs).read()
    add = "".join("pub mod %s;\n" % m for m in mods if ("pub mod %s;" % m) not in s)
    if add:
        s = s.replace("pub mod coverage;\n", "pub mod coverage;\n" + add, 1); open(modrs, "w").write(s)
        print("   registered in sysw/mod.rs: " + ", ".join(mods))
if fragments:
    print("   NOT assembled (fragments of existing files; reviewer's execution pass): " + ", ".join(sorted(fragments)))
PY
cd "$WORK"
echo "== 3 -- build mnemonic-engrave (all targets) =="
cargo build -p mnemonic-engrave --all-targets --locked 2>&1 | tail -3
echo "== 4 -- run the composer test BINARIES and the composer_records unit tests =="
cargo nextest run -p mnemonic-engrave --locked --no-fail-fast -E 'binary(/^sysw_composer/) | test(/composer_records/)' 2>&1 | tail -8
echo "== 5 -- clippy mnemonic-engrave =="
cargo clippy -p mnemonic-engrave --all-targets --locked -- -D warnings 2>&1 | tail -3
echo "== NOT covered: sysw/mod.rs, sysw/record.rs, main.rs, coverage.rs fragments; the CLI tests' assertions against a wired binary; the payload spec fold; mnemonic-secret; the Go port. =="
