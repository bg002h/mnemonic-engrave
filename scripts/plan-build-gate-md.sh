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
#      Cargo.toml + Cargo.lock + rust-toolchain.toml + .cargo/ + any clippy/
#      rustfmt config, so the copy builds on the repo's PINNED toolchain and
#      not on rustup's default), with CARGO_TARGET_DIR kept OUTSIDE the copy so
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
#      extracted, and the `miniscript = { workspace = true, features =
#      ["compiler"] }` dev-dependency when compose_crosscheck.rs was, and the
#      `CliError::Compose(String)` variant with its Display arm when
#      cmd/compose.rs was (the plan states these one-line edits in prose).
#   4. cargo build -p md-codec --all-targets; cargo nextest run -p md-codec
#      -E 'binary(/^compose_/)' (the compose test BINARIES, not tests whose
#      NAME contains "compose" -- the first draft filtered by name and ran 18
#      of 46), with exactly one PINNED red accepted: the MANIFEST-comparison
#      test failing with "MANIFEST lacks", because test_vectors.rs is a
#      fragment this gate does not assemble; cargo clippy -p md-codec
#      --all-targets -- -D warnings;
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
# The toolchain PIN and cargo/lint config travel with the copy. Without
# rust-toolchain.toml the scratch build ran on whatever rustup's default was
# (a 1.97 nightly here, against the repo's pinned 1.85.0): everything built and
# passed, and then clippy failed on a PRE-EXISTING test with a lint that did not
# exist in 1.85 -- a red the repo itself can never see. A gate must build what
# the repo builds.
for f in rust-toolchain.toml rust-toolchain clippy.toml rustfmt.toml .rustfmt.toml; do [ -f "$SRC/$f" ] && cp "$SRC/$f" "$WORK/"; done
[ -d "$SRC/.cargo" ] && cp -r "$SRC/.cargo" "$WORK/"
# md-codec's display_grouping_conformance test reads ../../design/display-grouping-vectors.tsv
# (a checksum-pinned copy of the toolkit's canonical vectors); without it the whole-crate run in
# the copy is red for a reason that has nothing to do with the plan (measured 2026-09-02, S0b).
mkdir -p "$WORK/design"; cp "$SRC"/design/display-grouping-vectors.tsv* "$WORK/design/" 2>/dev/null || true
echo "   toolchain: $(cd "$WORK" && rustc --version 2>/dev/null)"
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
# synthesise the dev-dependency the plan states in prose for the compiler cross-check
ctoml = os.path.join(work, "crates/md-codec/Cargo.toml")
if "crates/md-codec/tests/compose_crosscheck.rs" in blocks and os.path.exists(ctoml):
    s = open(ctoml).read()
    if "[dev-dependencies]" in s and 'features = ["compiler"]' not in s:
        s = s.replace("[dev-dependencies]", '[dev-dependencies]\nminiscript = { workspace = true, features = ["compiler"] }', 1)
        open(ctoml, "w").write(s); print("   added dev-dependency miniscript/compiler to md-codec Cargo.toml")
# synthesise the CliError::Compose variant the plan states in prose (error.rs is a fragment)
errs = os.path.join(work, "crates/md-cli/src/error.rs")
if "crates/md-cli/src/cmd/compose.rs" in blocks and os.path.exists(errs):
    s = open(errs).read()
    if "Compose(String)" not in s:
        s = s.replace("    BadArg(String),\n", "    BadArg(String),\n    Compose(String),\n", 1)
        s = s.replace('            CliError::BadArg(m) => write!(f, "{m}"),\n', '            CliError::BadArg(m) => write!(f, "{m}"),\n            CliError::Compose(m) => write!(f, "{m}"),\n', 1)
        open(errs, "w").write(s); print("   added CliError::Compose(String) + Display arm to md-cli error.rs")
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
echo "== 4 -- run the compose test BINARIES (compose_*.rs) =="
# ONE red is pinned, not skipped: the MANIFEST-comparison test fails with
# "MANIFEST lacks" until the test_vectors.rs fragment is pasted (a fragment of
# an existing file, which this gate does not assemble). Any other failure, or
# that test failing for any other reason, fails the gate.
TESTLOG="$(mktemp)"; trap 'rm -f "$TESTLOG"' EXIT
set +e
cargo nextest run -p md-codec --locked --no-fail-fast -E 'binary(/^compose_/)' >"$TESTLOG" 2>&1
rc=$?
set -e
tail -8 "$TESTLOG"
if [ "$rc" -ne 0 ]; then
  fails="$(grep -E '^\s+FAIL \[' "$TESTLOG" | sed -E 's/.* md-codec::[a-z_]+ //' | sort -u)"
  if [ "$fails" = "every_compose_vector_in_the_manifest_is_exactly_what_compose_renders" ] && grep -q 'MANIFEST lacks' "$TESTLOG"; then
    echo "   PINNED RED: only the MANIFEST-comparison test failed, with 'MANIFEST lacks' (test_vectors.rs fragment not assembled) -- accepted"
  else
    echo "   compose tests FAILED (not the pinned red):"; echo "$fails" | sed 's/^/     /'; exit 100
  fi
fi
echo "== 5 -- clippy md-codec =="
cargo clippy -p md-codec --all-targets --locked -- -D warnings 2>&1 | tail -3
echo "== 6 -- compile-check md-cli (tests --no-run; the compose subcommand's main.rs arm is a fragment) =="
cargo test -p md-cli --locked --no-run 2>&1 | tail -3
echo "== NOT covered: main.rs/lib.rs/test_vectors.rs fragments; md-cli test ASSERTIONS (need the wired binary); the Go port. =="
