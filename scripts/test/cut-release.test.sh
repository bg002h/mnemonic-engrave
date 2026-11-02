#!/usr/bin/env bash
# cut-release.test.sh -- the 78 rows of design/SPEC_cut_release.md section 9.
#
#   bash scripts/test/cut-release.test.sh            # 78 rows (70 offline cr_, 8 network crn_)
#   bash scripts/test/cut-release.test.sh --offline  # rows 1-70 only
#
# Prints "ok <row>" or "FAIL <row>: <why>" per row and ends with
# "cut-release.test: <p> passed, <f> failed". Exit 0 iff no row failed and the
# number of rows run is the expected 78 (70 with --offline).
#
# Everything runs against local bare repos under $T, a fake gh, a fake
# push-via-staging and a throwaway minisign key. The network rows (71-78) use
# the real key, the real push-via-staging seam and read-only gh calls.
# This test is a local gate (spec section 10 step 4); no workflow runs it.
set -uo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$HERE/../.." && pwd)
FIX=$HERE/fixtures/cut-release
CR_REAL=$ROOT/scripts/cut-release.sh
SELF=$HERE/cut-release.test.sh

OFFLINE=0
for a in "$@"; do
    case "$a" in
        --offline) OFFLINE=1 ;;
        *) echo "usage: $0 [--offline]" >&2; exit 2 ;;
    esac
done
EXPECT=78
[ "$OFFLINE" -eq 0 ] || EXPECT=70

CACHEROOT=${XDG_CACHE_HOME:-$HOME/.cache}/cut-release-test
mkdir -p "$CACHEROOT"
T=$(readlink -f "$CACHEROOT")/run-$$
rm -rf "$T"
mkdir -p "$T/bin" "$T/key" "$T/key2" "$T/none" "$T/remote" "$T/work"
KEEP=0
cleanup() {
    if [ "$KEEP" -eq 0 ]; then rm -rf "$T"; else echo "cut-release.test: kept $T" >&2; fi
}
trap cleanup EXIT

REAL_PATH=$PATH
export PATH="$T/bin:$HOME/.cargo/bin:$PATH"
export CUT_RELEASE_TEST_T=$T
export CUT_RELEASE_SIBLINGS=$T/none
export CUT_RELEASE_DATE=2026-11-01
export CUT_RELEASE_POLL_SECONDS=0
export CUT_RELEASE_CACHE=$T/cache
export CARGO_TARGET_DIR=$T/target
export CUT_RELEASE_PUSH_VIA_STAGING=$T/bin/fake-pvs
export CUT_RELEASE_PUBKEY_FILE=$T/key/pub
export GIT_AUTHOR_NAME=Test GIT_AUTHOR_EMAIL=t@example.invalid
export GIT_COMMITTER_NAME=Test GIT_COMMITTER_EMAIL=t@example.invalid
cat >"$T/gitconfig" <<'EOF'
[init]
	defaultBranch = master
[commit]
	gpgsign = false
[tag]
	gpgsign = false
[protocol "file"]
	allow = always
[advice]
	detachedHead = false
EOF
export GIT_CONFIG_GLOBAL=$T/gitconfig
export GIT_CONFIG_SYSTEM=/dev/null
# the real cargo's lockfile/registry choices must not depend on the user's config
unset RUSTC_WRAPPER

cp "$FIX/fake-gh" "$T/bin/gh"
cp "$FIX/fake-pvs" "$T/bin/fake-pvs"
chmod +x "$T/bin/gh" "$T/bin/fake-pvs"

minisign -G -W -f -p "$T/key/pub" -s "$T/key/sec" >/dev/null 2>&1 || { echo "cut-release.test: cannot make a minisign key" >&2; exit 2; }
minisign -G -W -f -p "$T/key2/pub" -s "$T/key2/sec" >/dev/null 2>&1 || { echo "cut-release.test: cannot make a second minisign key" >&2; exit 2; }

# The script under test runs from a committed scratch copy, so its own
# "committed tooling only" check (P7) does not depend on the state of this tree.
mk_eng() { # dir
    rm -rf "$1"
    mkdir -p "$1/scripts"
    cp "$CR_REAL" "$ROOT/scripts/push-via-staging.sh" "$1/scripts/"
    cp "$ROOT/minisign.pub" "$1/minisign.pub"
    git -C "$1" init -q -b master
    git -C "$1" add -A
    git -C "$1" commit -q -m "tooling copy"
}
mk_eng "$T/engbase"
CR=$T/engbase/scripts/cut-release.sh
OUT=$T/out
ERR=$T/err
RC=0

# ------------------------------------------------------------------ plumbing
PASSED=0
FAILED=0
RUN=0
ROWERR=""
rowfail() { ROWERR+="${ROWERR:+; }$*"; }
s12() { printf '%s' "${1:0:12}"; }

x_rc() { [ "$RC" -eq "$1" ] || rowfail "exit $RC, wanted $1 (stderr: $(head -c 400 "$ERR" 2>/dev/null | tr '\n' '|'))"; }
x_err() { grep -Fxq -- "$1" "$ERR" 2>/dev/null || rowfail "stderr lacks line: $1"; }
x_out() { grep -Fxq -- "$1" "$OUT" 2>/dev/null || rowfail "stdout lacks line: $1"; }
x_any() { grep -Fxq -- "$1" "$OUT" 2>/dev/null || grep -Fxq -- "$1" "$ERR" 2>/dev/null || rowfail "no stream has line: $1"; }
x_err_start() { grep -Fq -- "$1" "$ERR" 2>/dev/null && [ -n "$(grep -F -- "$1" "$ERR" | awk -v p="$1" 'index($0,p)==1')" ] || rowfail "stderr has no line starting: $1"; }
x_out_start() { [ -n "$(grep -F -- "$1" "$OUT" 2>/dev/null | awk -v p="$1" 'index($0,p)==1')" ] || rowfail "stdout has no line starting: $1"; }
x_no_line() { if grep -Fxq -- "$1" "$OUT" 2>/dev/null || grep -Fxq -- "$1" "$ERR" 2>/dev/null; then rowfail "unexpected line: $1"; fi; }
x_last_err() { [ "$(tail -n 1 "$ERR" 2>/dev/null)" = "$1" ] || rowfail "last stderr line is '$(tail -n 1 "$ERR" 2>/dev/null)', wanted '$1'"; }
x_eq() { [ "$1" = "$2" ] || rowfail "$3: got '$1', wanted '$2'"; }

fresh() {
    cd "$T" || return 1
    rm -rf "$T/remote" "$T/work" "$T/assets" "$T/clone2" "$T/sib" "$T/cache" "$T/eng" "$T/fakebin" "$T/pk" "$T"/merge.*
    rm -f "$T/gh.log" "$T/pvs.log" "$T/gh.killed" "$T/out" "$T/err"
    mkdir -p "$T/remote" "$T/work"
    FXDIR=""; FXBARE=""; FXREPO=""; FXDEF=""
}

run_cr() { ( cd "$FXDIR" && "$CR" "$@" ) >"$OUT" 2>"$ERR"; RC=$?; }
run_cr_net() { ( cd "$FXDIR" && env -u CUT_RELEASE_PUSH_VIA_STAGING -u CUT_RELEASE_PUBKEY_FILE PATH="$HOME/.cargo/bin:$REAL_PATH" "$ROOT/scripts/cut-release.sh" "$@" ) >"$OUT" 2>"$ERR"; RC=$?; }
bsha() { git --git-dir="$FXBARE" rev-parse "$1"; }
gitdir() { git -C "$FXDIR" rev-parse --absolute-git-dir; }
snap() { PRE_LS=$(git ls-remote "$FXBARE" 2>/dev/null); PRE_HEAD=$(git -C "$FXDIR" rev-parse HEAD); }
x_untouched() { # tags...
    [ -z "$(git -C "$FXDIR" status --porcelain)" ] || rowfail "tree not clean"
    [ ! -s "$T/gh.log" ] || rowfail "gh.log is not empty"
    [ ! -s "$T/pvs.log" ] || rowfail "pvs.log is not empty"
    [ "$(git ls-remote "$FXBARE" 2>/dev/null)" = "$PRE_LS" ] || rowfail "the bare remote changed"
    [ "$(git -C "$FXDIR" rev-parse HEAD)" = "$PRE_HEAD" ] || rowfail "HEAD moved"
    local t
    for t in "$@"; do
        if git -C "$FXDIR" rev-parse -q --verify "refs/tags/$t" >/dev/null 2>&1; then rowfail "local tag $t exists"; fi
    done
}
x_no_remote_tag() { # tag...
    local t
    for t in "$@"; do
        if [ -n "$(git --git-dir="$FXBARE" for-each-ref "refs/tags/$t")" ]; then rowfail "tag $t is on the bare remote"; fi
    done
}

# ------------------------------------------------------------------ fixtures
fx_init() { # repo default
    FXREPO=$1; FXDEF=$2
    FXDIR=$T/work/$1; FXBARE=$T/remote/$1.git
    rm -rf "$FXDIR" "$FXBARE"
    git init -q --bare -b "$2" "$FXBARE"
    git init -q -b "$2" "$FXDIR"
    git -C "$FXDIR" remote add origin "$FXBARE"
}
fx_commit() {
    git -C "$FXDIR" add -A
    git -C "$FXDIR" commit -q -m "$1"
    git -C "$FXDIR" push -q origin "HEAD:refs/heads/$FXDEF"
}
fx_tag() { # tag
    git -C "$FXDIR" tag -a -m "$1" "$1"
    git -C "$FXDIR" push -q origin "refs/tags/$1"
}
mk_pkg() { # dir name version [bin]
    mkdir -p "$1/src"
    {
        echo "[package]"
        echo "name = \"$2\""
        echo "version = \"$3\""
        echo 'edition = "2021"'
        if [ -n "${4:-}" ]; then
            echo
            echo "[[bin]]"
            echo "name = \"$4\""
            echo 'path = "src/main.rs"'
        fi
    } >"$1/Cargo.toml"
}

fx_ms() {
    fx_init mnemonic-secret master
    cd "$FXDIR" || return 1
    mkdir -p crates
    printf '[workspace]\nmembers = ["crates/ms-codec", "crates/ms-cli"]\nresolver = "2"\n' >Cargo.toml
    mk_pkg crates/ms-codec ms-codec 0.10.1
    : >crates/ms-codec/src/lib.rs
    mk_pkg crates/ms-cli ms-cli 0.20.2 ms
    printf '\n[dependencies]\nms-codec = { path = "../ms-codec", version = "=0.10.1" }\n' >>crates/ms-cli/Cargo.toml
    printf 'fn main() {\n    println!("ms {}", env!("CARGO_PKG_VERSION"));\n}\n' >crates/ms-cli/src/main.rs
    cat >CHANGELOG.md <<'EOF'
# Changelog

## ms-cli [Unreleased]

- a fix.

## ms-cli [0.20.2] — 2026-11-01

- old.

## ms-codec [0.10.1] — 2026-11-01

- old.
EOF
    cargo generate-lockfile --offline >/dev/null 2>&1
    fx_commit "ms fixture"
    fx_tag ms-cli-v0.20.2
    fx_tag ms-codec-v0.10.1
}

fx_mt() {
    fx_init mnemonic-transaction main
    cd "$FXDIR" || return 1
    mkdir -p crates
    printf '[workspace]\nmembers = ["crates/mt-codec", "crates/mt-cli"]\nresolver = "2"\n' >Cargo.toml
    mk_pkg crates/mt-codec mt-codec 0.1.0
    : >crates/mt-codec/src/lib.rs
    mk_pkg crates/mt-cli mt-cli 0.1.1 mt
    printf '\n[dependencies]\nmt-codec = { path = "../mt-codec", version = "0.1.0" }\n' >>crates/mt-cli/Cargo.toml
    printf 'fn main() {\n    println!("mt {}", env!("CARGO_PKG_VERSION"));\n}\n' >crates/mt-cli/src/main.rs
    cargo generate-lockfile --offline >/dev/null 2>&1
    fx_commit "mt fixture"
    fx_tag mt-cli-v0.1.1
}

fx_tk() {
    fx_init mnemonic-toolkit master
    cd "$FXDIR" || return 1
    mkdir -p crates .examples-build scripts docs/manual-gui/tests docs/manual-gui/src
    printf '[workspace]\nmembers = ["crates/mnemonic-toolkit"]\nresolver = "2"\n' >Cargo.toml
    mk_pkg crates/mnemonic-toolkit mnemonic-toolkit 0.105.1 mnemonic
    printf 'fn main() {\n    println!("mnemonic {}", env!("CARGO_PKG_VERSION"));\n}\n' >crates/mnemonic-toolkit/src/main.rs
    printf '# mnemonic-toolkit\n\n<!-- toolkit-version: 0.105.1 -->\n' >README.md
    printf '# crate readme\n\n<!-- toolkit-version: 0.105.1 -->\n' >crates/mnemonic-toolkit/README.md
    cp "$FIX/tk-gen.sh" .examples-build/gen.sh
    cp "$FIX/tk-Examples.md" .examples-build/Examples.md
    cp "$FIX/tk-install.sh" scripts/install.sh
    cp "$FIX/install-assets.test.sh" scripts/install-assets.test.sh
    cp "$FIX/check_cli_pins.py" docs/manual-gui/tests/check_cli_pins.py
    printf '[manual-gui]\ntoolkit-tag-implied = "mnemonic-toolkit-v0.105.1"\n' >docs/manual-gui/pinned-upstream.toml
    cat >CHANGELOG.md <<'EOF'
# Changelog

## [Unreleased]

- a toolkit fix.

## mnemonic-toolkit [0.105.1] — 2026-09-25

- old.
EOF
    cargo generate-lockfile --offline >/dev/null 2>&1
    fx_commit "toolkit fixture"
    fx_tag mnemonic-toolkit-v0.105.1
}

# insert_above <file> <line> <text lines...>
insert_above() {
    local f=$1 n=$2
    shift 2
    { sed -n "1,$((n - 1))p" "$f"; printf '%s\n' "$@"; sed -n "$n,\$p" "$f"; } >"$f.new"
    mv "$f.new" "$f"
}
codec_entries() { insert_above "$FXDIR/CHANGELOG.md" 7 '## ms-codec [Unreleased]' '' '- a codec fix.' ''; fx_commit "codec entries"; }

# mk_assets <tag> : $T/assets/<tag> with the archive, the sums file and its signature
mk_assets() {
    local tag=$1 prefix bin archive sums n d key
    case "$tag" in
        ms-cli-v*) prefix=ms-cli-v; bin=ms; sums=SHA256SUMS.x86_64; n=${tag#"$prefix"}; archive=ms-$n-x86_64-linux-musl.tar.gz ;;
        mnemonic-toolkit-v*) prefix=mnemonic-toolkit-v; bin=mnemonic; sums=SHA256SUMS.x86_64; n=${tag#"$prefix"}; archive=mnemonic-$n-x86_64-linux-musl.tar.gz ;;
        *) echo "mk_assets: no recipe for $tag" >&2; return 1 ;;
    esac
    d=$T/assets/$tag
    rm -rf "$d" "$T/pk"
    mkdir -p "$d" "$T/pk"
    printf '#!/bin/sh\necho "%s %s"\n' "$bin" "${MK_ASSETS_BINVER:-$n}" >"$T/pk/$bin"
    chmod +x "$T/pk/$bin"
    tar -czf "$d/$archive" -C "$T/pk" "$bin"
    (cd "$d" && sha256sum "$archive" >"$sums")
    if [ "${MK_ASSETS_TAMPER:-}" = 1 ]; then printf 'x' >>"$d/$archive"; fi
    key=${MK_ASSETS_KEY:-$T/key/sec}
    if [ "${MK_ASSETS_NOSIG:-}" != 1 ]; then
        minisign -S -s "$key" -m "$d/$sums" -x "$d/$sums.minisig" -t "${MK_ASSETS_COMMENT:-$bin $tag $sums}" >/dev/null 2>&1
    fi
    if [ -n "${MK_ASSETS_EXTRA:-}" ]; then : >"$d/$MK_ASSETS_EXTRA"; fi
}
# the released-before assets that V2 compares against
mk_prev_ms() { mk_assets ms-cli-v0.20.2; }

finish_line() { # bin-tags
    printf 'cut-release: finish: cut-release.sh --verify %s; once it shows 0 failed, make the follow-on moves in %s/cut-release/follow-on.txt' "$1" "$(gitdir)"
}
pins_line() { # csha N
    printf "cut-release: pins PR: git checkout --quiet --detach %s && sh scripts/install-assets.test.sh && git checkout --quiet master && git push origin %s:refs/heads/release-pins-%s && gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-%s --draft --title 'pins: installer -> mnemonic-toolkit-v%s' --body-file %s/cut-release/pins-pr-body.md" "$1" "$1" "$2" "$2" "$2" "$(gitdir)"
}
P23_TEXT() { printf 'cut-release: origin has ci/staging at %s; push-via-staging.sh cannot push a new release commit over it (an interrupted or failed staging run left it). If no staging run is in progress, delete it, then run again: git push origin --delete ci/staging' "$1"; }
U1_TEXT='cut-release: give exactly one of --dry-run, --push, --verify, --land-pr (see --help)'

# F-tk with the push of row 42 already done; leaves the state rows 44-47, 66, 67 start from
tk_pushed() {
    fresh; fx_tk
    mk_assets mnemonic-toolkit-v0.105.1 || return 1
    mk_assets mnemonic-toolkit-v0.105.2 || return 1
    run_cr --push mnemonic-toolkit-v0.105.2
    [ "$RC" -eq 0 ] || rowfail "setup push exited $RC: $(head -c 300 "$ERR" | tr '\n' '|')"
}

# ---------------------------------------------------------------- offline rows
cr_usage_no_mode() {
    fresh; fx_ms
    run_cr ms-cli-v0.20.3
    x_rc 2; x_err "$U1_TEXT"
}
cr_usage_two_modes() {
    fresh; fx_ms
    run_cr --dry-run --push ms-cli-v0.20.3
    x_rc 2; x_err "$U1_TEXT"
}
cr_usage_no_tags() {
    fresh; fx_ms
    run_cr --dry-run
    x_rc 2; x_err "cut-release: name at least one tag"
}
cr_help() {
    fresh; fx_ms
    run_cr --help
    x_rc 0
    x_eq "$(sed -n 1p "$OUT")" "usage: cut-release.sh --dry-run [--trailer 'Key: value']... <tag>..." "line 1"
}
cr_verify_lib_tag() {
    fresh; fx_ms
    run_cr --verify ms-codec-v0.10.1
    x_rc 2; x_err "cut-release: ms-codec-v0.10.1 is a library tag; it has no release assets to verify"
}
cr_not_in_table() {
    fresh; fx_init mnemonic-refugium master
    echo x >"$FXDIR/f"; fx_commit init
    run_cr --dry-run mr-cli-v0.1.0
    x_rc 3; x_err "cut-release: origin is '$T/remote/mnemonic-refugium.git'; 'mnemonic-refugium' is not in the release table"
}
cr_wrong_prefix() {
    fresh; fx_ms
    run_cr --dry-run mk-cli-v0.14.0
    x_rc 3; x_err "cut-release: 'mk-cli-v0.14.0' is not a release tag of mnemonic-secret (its prefixes: ms-cli-v ms-codec-v)"
}
cr_not_semver() {
    fresh; fx_ms
    run_cr --dry-run ms-cli-v0.21
    x_rc 3; x_err "cut-release: 'ms-cli-v0.21' does not end in a MAJOR.MINOR.PATCH version"
}
cr_duplicate_crate() {
    fresh; fx_ms
    run_cr --dry-run ms-cli-v0.20.3 ms-cli-v0.20.4
    x_rc 3; x_err "cut-release: two tags for ms-cli: ms-cli-v0.20.3 ms-cli-v0.20.4"
}
cr_not_default_branch() {
    fresh; fx_ms
    git -C "$FXDIR" switch -q -c topic
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3; x_err "cut-release: on branch 'topic'; release from master (git switch master)"
}
cr_behind_origin() {
    fresh; fx_ms
    git clone -q "$FXBARE" "$T/clone2"
    echo more >"$T/clone2/extra.txt"
    git -C "$T/clone2" add -A; git -C "$T/clone2" commit -q -m more; git -C "$T/clone2" push -q origin HEAD:refs/heads/master
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3
    x_err "cut-release: HEAD $(s12 "$(git -C "$FXDIR" rev-parse HEAD)") is not origin/master $(s12 "$(bsha refs/heads/master)"); the release commit must sit directly on the remote tip (git pull --ff-only, or land your commits first)"
}
cr_dirty_tracked() {
    fresh; fx_ms
    echo '// edit' >>"$FXDIR/crates/ms-cli/src/main.rs"
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3
    x_err "cut-release: tracked files are modified; the release commit carries only what cut-release.sh writes. Commit or restore them first:"
    x_err "   crates/ms-cli/src/main.rs"
}
cr_tag_on_origin() {
    fresh; fx_ms
    git clone -q "$FXBARE" "$T/clone2"
    git -C "$T/clone2" tag -a -m x ms-cli-v0.20.3
    git -C "$T/clone2" push -q origin refs/tags/ms-cli-v0.20.3
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3; x_err "cut-release: tag ms-cli-v0.20.3 already exists on origin"
}
cr_not_newer_than_cargo() {
    fresh; fx_ms
    run_cr --dry-run ms-cli-v0.20.1
    x_rc 3; x_err "cut-release: ms-cli-v0.20.1: ms-cli is 0.20.2 in crates/ms-cli/Cargo.toml; 0.20.1 is not newer"
}
cr_not_newer_than_tag() {
    fresh; fx_ms
    git -C "$FXDIR" tag -a -m x ms-cli-v0.20.5
    git -C "$FXDIR" push -q origin refs/tags/ms-cli-v0.20.5
    git -C "$FXDIR" tag -d ms-cli-v0.20.5 >/dev/null
    run_cr --dry-run ms-cli-v0.20.4
    x_rc 3; x_err "cut-release: ms-cli-v0.20.4: origin already has ms-cli-v0.20.5, which is not older than 0.20.4"
}
cr_no_unreleased() {
    fresh; fx_ms
    sed -i 3,6d "$FXDIR/CHANGELOG.md"; fx_commit "no U"
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3; x_err "cut-release: CHANGELOG.md has no '## ms-cli [Unreleased]' section; the fix PR writes the release notes there"
}
cr_unreleased_empty() {
    fresh; fx_ms
    sed -i 5d "$FXDIR/CHANGELOG.md"; fx_commit "empty U"
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3; x_err "cut-release: CHANGELOG.md:3: the '## ms-cli [Unreleased]' section is empty; nothing to release for ms-cli"
}
cr_unreleased_below_release() {
    fresh; fx_ms
    local f=$FXDIR/CHANGELOG.md
    { sed -n 1,2p "$f"; sed -n 7,10p "$f"; sed -n 3,6p "$f"; sed -n '11,$p' "$f"; } >"$f.new"; mv "$f.new" "$f"; fx_commit "U below"
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3; x_err "cut-release: CHANGELOG.md:7: '## ms-cli [Unreleased]' sits below '## ms-cli [0.20.2] — 2026-11-01'; it must be above every release heading"
}
cr_unreleased_and_heading() {
    fresh; fx_ms
    insert_above "$FXDIR/CHANGELOG.md" 7 '## ms-cli [0.20.3] — 2026-11-01' '' '- x.' ''; fx_commit "both"
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3; x_err "cut-release: CHANGELOG.md has both '## ms-cli [0.20.3]' and entries under '## ms-cli [Unreleased]'"
}
cr_next_release_mismatch() {
    fresh; fx_ms
    sed -i '5s/.*/**Next release: 0.21.0** — x./' "$FXDIR/CHANGELOG.md"; fx_commit "next"
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3; x_err "cut-release: CHANGELOG.md:3: '## ms-cli [Unreleased]' says 'Next release: 0.21.0', not 0.20.3"
}
cr_codec_entries_unreleased() {
    fresh; fx_ms; codec_entries
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3; x_err "cut-release: ms-codec has entries under '## ms-codec [Unreleased]' and ms-cli builds it by path; tag ms-codec in this same run (ms-codec-vMAJOR.MINOR.PATCH) or move those entries"
}
cr_stale_pin() {
    fresh; fx_ms
    sed -i 's/version = "=0.10.1"/version = "=0.10.0"/' "$FXDIR/crates/ms-cli/Cargo.toml"; fx_commit "stale pin"
    codec_entries
    run_cr --dry-run ms-codec-v0.10.2
    x_rc 3; x_err "cut-release: crates/ms-cli/Cargo.toml pins ms-codec at '=0.10.0'; expected '=0.10.1', '^0.10.1' or no version (fix the pin by hand first)"
}
cr_toolchain_mismatch() {
    fresh; fx_ms
    printf '[toolchain]\nchannel = "1.85.0"\n' >"$FXDIR/rust-toolchain.toml"; fx_commit "toolchain"
    mkdir -p "$T/fakebin"
    printf '#!/bin/sh\nif [ "$1" = "--version" ]; then echo "cargo 9.9.9 (fake)"; exit 0; fi\nexit 1\n' >"$T/fakebin/cargo"
    chmod +x "$T/fakebin/cargo"
    PATH="$T/fakebin:$PATH" run_cr --dry-run ms-cli-v0.20.3
    x_rc 3; x_err "cut-release: cargo --version is 'cargo 9.9.9 (fake)' but rust-toolchain.toml pins 1.85.0; put rustup's cargo first on PATH (PATH=\$HOME/.cargo/bin:\$PATH)"
}
cr_push_tooling_dirty() {
    fresh; fx_ms
    mk_eng "$T/eng"
    echo "# modified" >>"$T/eng/minisign.pub"
    ( cd "$FXDIR" && "$T/eng/scripts/cut-release.sh" --push ms-cli-v0.20.3 ) >"$OUT" 2>"$ERR"; RC=$?
    x_rc 3; x_err "cut-release: minisign.pub in $T/eng has uncommitted changes; a release runs only committed tooling"
}
cr_dry_run_ms_two_tags() {
    fresh; fx_ms; codec_entries; snap
    run_cr --dry-run ms-codec-v0.10.2 ms-cli-v0.20.3
    x_rc 0
    x_out "== ms-codec 0.10.1 -> 0.10.2 (crates/ms-codec/Cargo.toml)"
    x_out "== ms-cli 0.20.2 -> 0.20.3 (crates/ms-cli/Cargo.toml)"
    x_out "== pin: crates/ms-cli/Cargo.toml ms-codec =0.10.1 -> =0.10.2"
    x_out "== CHANGELOG: CHANGELOG.md '## ms-codec [Unreleased]' -> '## ms-codec [0.10.2] — 2026-11-01'"
    x_out "== CHANGELOG: CHANGELOG.md '## ms-cli [Unreleased]' -> '## ms-cli [0.20.3] — 2026-11-01'"
    x_out "== Cargo.lock: 2 version line(s) changed (ms-codec ms-cli)"
    x_out "== built: ms --version = 'ms 0.20.3'"
    x_out "DRY RUN: would commit 4 file(s): CHANGELOG.md Cargo.lock crates/ms-cli/Cargo.toml crates/ms-codec/Cargo.toml"
    x_out "DRY RUN: would run push-via-staging.sh master"
    x_out "DRY RUN: would tag ms-codec-v0.10.2 on the release commit and push refs/tags/ms-codec-v0.10.2"
    x_out "DRY RUN: would wait for ms-codec-v0.10.2 workflows: none"
    x_out "DRY RUN: would wait for ms-cli-v0.20.3 workflows: release man-release"
    x_out "DRY RUN: would check ms-codec-v0.10.2 points at the release commit"
    x_out "DRY RUN: would verify ms-cli-v0.20.3: ms-0.20.3-x86_64-linux-musl.tar.gz in SHA256SUMS.x86_64, every SHA256SUMS* signed, 'ms 0.20.3'"
    x_out "     mnemonic-engrave: not found at $T/none/mnemonic-engrave; check it by hand"
    x_out "== DRY RUN OK: nothing was pushed; tracked files are as they were"
    x_untouched ms-codec-v0.10.2 ms-cli-v0.20.3
}
cr_dry_run_resume() {
    fresh; fx_ms
    local f=$FXDIR/CHANGELOG.md
    sed -i 's/^version = "0.20.2"/version = "0.20.3"/' "$FXDIR/crates/ms-cli/Cargo.toml"
    { sed -n 1,3p "$f"; echo; echo '## ms-cli [0.20.3] — 2026-11-01'; sed -n '4,$p' "$f"; } >"$f.new"; mv "$f.new" "$f"
    (cd "$FXDIR" && cargo update --workspace --offline >/dev/null 2>&1)
    fx_commit "hand release"
    snap
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 0
    x_out "== ms-cli is already 0.20.3 (crates/ms-cli/Cargo.toml); no version edit"
    x_out "== CHANGELOG: CHANGELOG.md already has '## ms-cli [0.20.3]'"
    x_out "DRY RUN: nothing to commit; would stage HEAD $(s12 "$PRE_HEAD") as it is"
    x_untouched ms-cli-v0.20.3
}
cr_wrong_version_output() {
    fresh; fx_ms
    sed -i 's/println!.*/println!("ms 9.9.9");/' "$FXDIR/crates/ms-cli/src/main.rs"; fx_commit "wrong output"
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 4
    x_err "cut-release: ms --version printed 'ms 9.9.9', not 'ms 0.20.3'"
    x_last_err "cut-release: tracked files restored; nothing was committed or pushed"
    [ -z "$(git -C "$FXDIR" status --porcelain)" ] || rowfail "tree not clean"
}
cr_fn_lock_diff_check() {
    fresh; fx_ms
    local old=$T/lock.old new=$T/lock.new
    cp "$FXDIR/Cargo.lock" "$old"
    awk '/^name = "ms-cli"/ { c = 1 } c && /^version = / { sub(/0\.20\.2/, "0.20.3"); c = 0 } /^dependencies = \[/ { d = 1 } { print }' "$old" >"$new"
    sed -i 's/ "ms-codec",/ "ms-codec-x",/' "$new"
    local rc=0
    (set +e; source "$CR_REAL"; lock_diff_check "$old" "$new" ms-cli) >"$OUT" 2>"$ERR" || rc=$?
    RC=$rc
    x_rc 1
    x_err "cut-release: Cargo.lock changed beyond the bumped crates' version lines (2 line(s) changed, 1 expected):"
}
promote_check() { # name input-lines expected-lines U heading
    local f=$T/promote.md
    printf '%s\n' "${PROMOTE_IN[@]}" >"$f"
    (set +e; source "$CR_REAL"; changelog_promote "$f" "$1" "$2") >"$OUT" 2>"$ERR"
    local got want
    got=$(cat "$f"); want=$(printf '%s\n' "${PROMOTE_WANT[@]}")
    x_eq "$got" "$want" "promoted file"
}
cr_fn_promote_plain() {
    PROMOTE_IN=('## Unreleased' '' '- x.' '' '## mnemonic-gui [0.63.0] — 2026-09-25')
    PROMOTE_WANT=('## Unreleased' '' '## mnemonic-gui [0.64.0] — 2026-11-01' '' '- x.' '' '## mnemonic-gui [0.63.0] — 2026-09-25')
    promote_check '## Unreleased' '## mnemonic-gui [0.64.0] — 2026-11-01'
}
cr_fn_promote_next_release() {
    PROMOTE_IN=('## [Unreleased]' '' '**Next release: 0.14.0** — a.' 'b.' '' '### Changed' '- y' '' '## [0.13.0] — 2026-08-19')
    PROMOTE_WANT=('## [Unreleased]' '' '## [0.14.0] — 2026-11-01' '' '### Changed' '- y' '' '## [0.13.0] — 2026-08-19')
    promote_check '## [Unreleased]' '## [0.14.0] — 2026-11-01'
}
cr_fn_tk_examples() {
    fresh
    local g=$T/gen.sh e=$T/Examples.md
    cp "$FIX/tk-gen.sh" "$g"; cp "$FIX/tk-Examples.md" "$e"
    local X=0.105.1 N=0.105.2
    (set +e; source "$CR_REAL"; tk_examples_b "$g" "$X" "$N"; tk_examples_b "$e" "$X" "$N") >"$OUT" 2>"$ERR"
    local gc ec left bad=0 l
    gc=$(diff "$FIX/tk-gen.sh" "$g" | grep -c '^>' || true)
    ec=$(diff "$FIX/tk-Examples.md" "$e" | grep -c '^>' || true)
    x_eq "$gc" 5 "gen.sh lines changed after b"
    x_eq "$(grep -cF "$X" "$g" || true)" 0 "X left in gen.sh after b"
    x_eq "$ec" 4 "Examples.md lines changed after b"
    left=$(grep -F "$X" "$e" || true)
    x_eq "$(printf '%s\n' "$left" | grep -c .)" 4 "X lines left in Examples.md after b"
    while IFS= read -r l; do
        case "$l" in *"mnemonic-toolkit-v$X"*|*"'mnemonic $X'"*) ;; *) bad=$((bad + 1)) ;; esac
    done <<<"$left"
    x_eq "$bad" 0 "left lines that are not installer lines"
    (set +e; source "$CR_REAL"; tk_examples_c "$e" "$X" "$N") >>"$OUT" 2>>"$ERR"
    x_eq "$(grep -cF "$X" "$e" || true)" 0 "X left in Examples.md after c"
    x_eq "$(diff "$FIX/tk-Examples.md" "$e" | grep -c '^>' || true)" 8 "Examples.md lines changed in total"
}
cr_fn_tk_repad() {
    fresh
    local e=$T/Examples.md
    cp "$FIX/tk-Examples.md" "$e"
    (set +e; source "$CR_REAL"; tk_examples_c "$e" 0.105.1 0.1000.0) >"$OUT" 2>"$ERR"
    local got want
    got=$(grep -E '^mnemonic +mnemonic-toolkit +' "$e")
    want='mnemonic        mnemonic-toolkit     (none)         mnemonic-toolkit-v0.1000.0           mnemonic-0.1000.0-x86_64-linux-musl.tar.gz'
    x_eq "$got" "$want" "the --list line"
}

# the common assertions of the happy ms push (row 33), reused by nothing else
cr_push_ms_happy() {
    fresh; fx_ms; codec_entries; mk_prev_ms; mk_assets ms-cli-v0.20.3
    local trailer='Co-Authored-By: Test <t@example.invalid>'
    run_cr --push --trailer "$trailer" ms-codec-v0.10.2 ms-cli-v0.20.3
    x_rc 0
    local tip
    tip=$(bsha refs/heads/master)
    x_eq "$(git -C "$FXDIR" rev-parse HEAD)" "$tip" "local HEAD vs bare master"
    x_eq "$(git --git-dir="$FXBARE" log -1 --format=%s refs/heads/master)" "release: ms-codec 0.10.2 + ms-cli 0.20.3" "subject"
    git --git-dir="$FXBARE" log -1 --format=%B refs/heads/master | grep -Fxq -- "$trailer" || rowfail "trailer missing"
    local t
    for t in ms-codec-v0.10.2 ms-cli-v0.20.3; do
        x_eq "$(git --git-dir="$FXBARE" cat-file -t "refs/tags/$t")" tag "$t object type"
        x_eq "$(bsha "refs/tags/$t^{commit}")" "$tip" "$t peels to"
    done
    x_eq "$(git --git-dir="$FXBARE" show refs/heads/master:CHANGELOG.md | sed -n 3,9p)" "$(printf '%s\n' '## ms-cli [Unreleased]' '' '## ms-cli [0.20.3] — 2026-11-01' '' '- a fix.' '' '## ms-codec [Unreleased]')" "CHANGELOG lines 3-9"
    x_out "   ok    RAN ms --version -> ms 0.20.3"
    x_out "   not waited: fuzz-smoke run 1003 completed/success"
    x_out "== FREEZE LIFTED"
    x_out "== DONE: ms-codec-v0.10.2 ms-cli-v0.20.3 published and verified"
    ! grep -q 'fake gh: unexpected' "$T/gh.log" || rowfail "gh.log has an unexpected call"
}
cr_push_staging_red() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    FAKE_PVS=red run_cr --push ms-cli-v0.20.3
    x_rc 5
    x_err "cut-release: push-via-staging.sh exited 1; no tag was pushed."
    x_err "cut-release: origin/master is unchanged. To retry, drop the local commit and run again: git reset --hard origin/master"
    x_no_remote_tag ms-cli-v0.20.3
}
cr_push_bypass_line() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    FAKE_PVS=bypass run_cr --push ms-cli-v0.20.3
    x_rc 5
    x_err "cut-release: push-via-staging.sh output contains 'Bypassed rule violations'; not tagging. STOP and report it."
    x_no_remote_tag ms-cli-v0.20.3
}
cr_push_run_failed() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    FAKE_GH_CONCLUSION=failure run_cr --push ms-cli-v0.20.3
    x_rc 7
    x_err "cut-release: run 1001 (release, ms-cli-v0.20.3) concluded 'failure': https://github.com/bg002h/mnemonic-secret/actions/runs/1001"
    x_err "cut-release: the tags are pushed and stay pushed: never delete or move them."
    x_out "== follow-on pins are NOT listed: the release is not verified"
    x_err "$(finish_line ms-cli-v0.20.3)"
    [ -n "$(git --git-dir="$FXBARE" for-each-ref refs/tags/ms-cli-v0.20.3)" ] || rowfail "tag not on the bare remote"
}
cr_push_bad_signature() {
    fresh; fx_ms; mk_prev_ms
    MK_ASSETS_KEY=$T/key2/sec mk_assets ms-cli-v0.20.3
    run_cr --push ms-cli-v0.20.3
    x_rc 8
    x_out_start "   FAIL  SHA256SUMS.x86_64.minisig does not verify:"
}
cr_push_missing_minisig() {
    fresh; fx_ms; mk_prev_ms
    MK_ASSETS_NOSIG=1 mk_assets ms-cli-v0.20.3
    run_cr --push ms-cli-v0.20.3
    x_rc 8
    x_out "   FAIL  no SHA256SUMS.x86_64.minisig asset (unsigned release)"
}
cr_push_missing_asset() {
    fresh; fx_ms
    MK_ASSETS_EXTRA=ms-0.20.2-macos-arm64.tar.gz mk_assets ms-cli-v0.20.2
    mk_assets ms-cli-v0.20.3
    run_cr --push ms-cli-v0.20.3
    x_rc 8
    x_out "   FAIL  missing vs ms-cli-v0.20.2: ms-0.20.3-macos-arm64.tar.gz"
}

cr_toolkit_dry_run() {
    fresh; fx_tk; snap
    run_cr --dry-run mnemonic-toolkit-v0.105.2
    x_rc 0
    x_out "== toolkit (b): README markers 2, .examples-build/gen.sh 5 line(s), Examples.md 4 line(s); 4 installer line(s) left for (c)"
    x_out '== toolkit (c): install.sh self-pin mnemonic-toolkit-v0.105.1 -> mnemonic-toolkit-v0.105.2; first_signed mnemonic "" -> "0.105.2"; Examples.md 4 installer line(s); [installer-ahead] toolkit-tag-implied = "mnemonic-toolkit-v0.105.2"'
    x_out "== toolkit (c): check_cli_pins.py fails on (c); the pins PR owes the GUI manual's {#installer-ahead} section (listed below)"
    x_out "DRY RUN: would commit (b) 7 file(s): .examples-build/Examples.md .examples-build/gen.sh CHANGELOG.md Cargo.lock README.md crates/mnemonic-toolkit/Cargo.toml crates/mnemonic-toolkit/README.md"
    x_out "DRY RUN: would commit (c) 3 file(s): .examples-build/Examples.md docs/manual-gui/pinned-upstream.toml scripts/install.sh"
    x_untouched mnemonic-toolkit-v0.105.2
    [ -z "$(git --git-dir="$FXBARE" for-each-ref 'refs/heads/release-pins-*')" ] || rowfail "a release-pins branch is on the bare remote"
}
cr_toolkit_pins_ok() {
    fresh; fx_tk
    echo 'The {#installer-ahead} section.' >"$FXDIR/docs/manual-gui/src/82.md"; fx_commit "installer-ahead prose"
    run_cr --dry-run mnemonic-toolkit-v0.105.2
    x_rc 0
    x_out "== toolkit (c): check_cli_pins.py OK"
}
cr_toolkit_push_happy() {
    fresh; fx_tk; mk_assets mnemonic-toolkit-v0.105.1; mk_assets mnemonic-toolkit-v0.105.2
    run_cr --push mnemonic-toolkit-v0.105.2
    x_rc 0
    local b c
    b=$(bsha refs/heads/master)
    c=$(bsha refs/tags/mnemonic-toolkit-v0.105.2^{commit})
    x_eq "$(git --git-dir="$FXBARE" rev-parse "$c^")" "$b" "(c)'s parent"
    x_eq "$(bsha refs/heads/release-pins-0.105.2)" "$c" "the release-pins branch"
    x_eq "$(git -C "$FXDIR" rev-parse HEAD)" "$b" "local master"
    grep -Fq -- "pr create" "$T/gh.log" || rowfail "no pr create in gh.log"
    grep -F -- "pr create" "$T/gh.log" | grep -Fq -- "--draft" || rowfail "pr create without --draft"
    grep -F -- "pr create" "$T/gh.log" | grep -Fq -- "--body-file $(gitdir)/cut-release/pins-pr-body.md" || rowfail "pr create without the body file"
    grep -Fq -- "land it: cut-release.sh --land-pr <PR number>" "$(gitdir)/cut-release/pins-pr-body.md" || rowfail "body lacks the land-it line"
    x_out "== toolkit: pins PR https://github.com/bg002h/mnemonic-toolkit/pull/77 is open as a draft; it owes the text above. Its author marks it ready, then: cut-release.sh --land-pr 77"
}
cr_toolkit_install_assets_fail() {
    fresh; fx_tk; mk_assets mnemonic-toolkit-v0.105.1; mk_assets mnemonic-toolkit-v0.105.2
    FAKE_INSTALL_ASSETS=fail run_cr --push mnemonic-toolkit-v0.105.2
    x_rc 9
    local c
    c=$(bsha refs/tags/mnemonic-toolkit-v0.105.2^{commit})
    x_err "cut-release: scripts/install-assets.test.sh failed on (c) $(s12 "$c"); the pins PR was not opened. The tag is published. Fix forward on top of (c): git push origin $c:refs/heads/release-pins-0.105.2, commit the fix to that branch, then gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-0.105.2 --draft --title 'pins: installer -> mnemonic-toolkit-v0.105.2' --body-file $(gitdir)/cut-release/pins-pr-body.md (a PR without (c) leaves the tag off master)."
    ! grep -Fq -- "pr create" "$T/gh.log" || rowfail "pr create is in gh.log"
}
cr_land_pr_draft() {
    tk_pushed
    FAKE_PR_DRAFT=true run_cr --land-pr 77
    x_rc 3; x_err "cut-release: PR #77 is a draft; its author marks it ready (gh pr ready 77) once the owed edits are in"
}
cr_land_pr_failed_check() {
    tk_pushed
    FAKE_PR_CHECKS='[{"name":"manual-gui","bucket":"fail","link":"https://example.invalid/x"}]' run_cr --land-pr 77
    x_rc 9; x_err "cut-release: PR #77 check 'manual-gui' is fail: https://example.invalid/x"
}
cr_land_pr_happy() {
    tk_pushed
    local c
    c=$(bsha refs/heads/release-pins-0.105.2)
    run_cr --land-pr 77
    x_rc 0
    local tip
    tip=$(bsha refs/heads/master)
    x_eq "$(git --git-dir="$FXBARE" rev-list --parents -n 1 "$tip" | wc -w | tr -d ' ')" 3 "merge commit parents (words)"
    git --git-dir="$FXBARE" rev-list --parents -n 1 "$tip" | grep -Fq -- "$c" || rowfail "(c) is not a parent of master"
    [ -z "$(git --git-dir="$FXBARE" for-each-ref refs/heads/release-pins-0.105.2)" ] || rowfail "the pins branch is still on the bare remote"
    x_out "== merged PR #77 (https://github.com/bg002h/mnemonic-toolkit/pull/77) as a merge commit; release-pins-0.105.2 deleted"
    x_eq "$(grep -c '^pr merge' "$T/gh.log" || true)" 1 "pr merge lines"
    grep -Fxq -- "pr merge 77 --repo bg002h/mnemonic-toolkit --merge --match-head-commit $c" "$T/gh.log" || rowfail "pr merge argv is not the expected shape"
}
cr_land_pr_tag_not_in_pr() {
    tk_pushed
    local b
    b=$(bsha refs/heads/master)
    FAKE_PR_OID=$b run_cr --land-pr 77
    x_rc 9
    x_err "cut-release: PR #77 does not contain mnemonic-toolkit-v0.105.2 ($(s12 "$(bsha refs/tags/mnemonic-toolkit-v0.105.2^{commit})")); merging it would leave the tag off master"
}
cr_dry_run_mt() {
    fresh; fx_mt; snap
    run_cr --dry-run mt-cli-v0.1.2
    x_rc 0
    x_out "== CHANGELOG: none in mnemonic-transaction"
    x_out "== Cargo.lock: 1 version line(s) changed (mt-cli)"
    x_out "DRY RUN: would commit 2 file(s): Cargo.lock crates/mt-cli/Cargo.toml"
    x_out "DRY RUN: would wait for mt-cli-v0.1.2 workflows: release"
    x_untouched mt-cli-v0.1.2
}
cr_static_push() {
    local all l1 l2 line bad=0
    all=$(grep -nE '\bgit push\b' "$CR_REAL" || true)
    l1=$(printf '%s\n' "$all" | grep -E '^[0-9]+:cut-release: ' || true)
    l2=$(printf '%s\n' "$all" | grep -vE '^[0-9]+:cut-release: ' | grep . || true)
    x_eq "$(printf '%s\n' "$l2" | grep -c .)" 3 "L2 line count"
    printf '%s\n' "$l2" | grep -Fq -- '--atomic origin' || rowfail "L2 has no --atomic origin line"
    printf '%s\n' "$l2" | grep -Fq -- 'refs/heads/release-pins-' || rowfail "L2 has no release-pins line"
    printf '%s\n' "$l2" | grep -Fq -- 'origin --delete' || rowfail "L2 has no origin --delete line"
    for pat in '--tags' '--force' ' -f ' 'HEAD:' '--mirror'; do
        if printf '%s\n' "$l2" | grep -Fq -- "$pat"; then rowfail "L2 contains $pat"; fi
    done
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        line=${line#*:}
        case "$line" in
            "cut-release: otherwise finish by hand: "*|"cut-release: pins PR: "*|"cut-release: scripts/install-assets.test.sh failed on (c) "*|"cut-release: origin has ci/staging at "*) ;;
            *) bad=1; rowfail "L1 line with an unexpected start: ${line:0:60}" ;;
        esac
        case "$line" in *'"'*|*'$('*|*'`'*) rowfail "L1 line holds a quote, \$( or backtick" ;; esac
    done <<<"$l1"
    x_eq "$(printf '%s\n' "$l1" | grep -c .)" 4 "L1 line count"
}
cr_bash_n() {
    bash -n "$CR_REAL" || rowfail "bash -n on the script failed"
    bash -n "$SELF" || rowfail "bash -n on the harness failed"
}
cr_ahead_of_origin() {
    fresh; fx_ms
    echo more >"$FXDIR/extra.txt"; git -C "$FXDIR" add -A; git -C "$FXDIR" commit -q -m 'release: ms-cli 0.20.3'
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3
    x_err "cut-release: HEAD $(s12 "$(git -C "$FXDIR" rev-parse HEAD)") is 1 commit(s) ahead of origin/master $(s12 "$(bsha refs/heads/master)"); the release commit must sit directly on the remote tip. A 'release: ...' commit from an interrupted cut-release.sh run: drop it (git reset --hard origin/master) and run again. A 'pins: installer -> ...' commit: finish as cut-release.sh printed when it stopped, and never land it without its pull request. Any other commit: land it first."
}
cr_toolkit_already_bumped() {
    fresh; fx_tk
    sed -i 's/^version = "0.105.1"/version = "0.105.2"/' "$FXDIR/crates/mnemonic-toolkit/Cargo.toml"
    (cd "$FXDIR" && cargo update --workspace --offline >/dev/null 2>&1)
    fx_commit "bumped"
    run_cr --dry-run mnemonic-toolkit-v0.105.2
    x_rc 3
    x_err "cut-release: mnemonic-toolkit is already 0.105.2 in crates/mnemonic-toolkit/Cargo.toml; the toolkit release needs its version bump in commit (b), so release it by hand or restore the previous version"
}
cr_toolkit_selfpin_lags() {
    fresh; fx_tk
    sed -i 's/mnemonic-toolkit-v0.105.1/mnemonic-toolkit-v0.105.0/' "$FXDIR/scripts/install.sh"; fx_commit "lagging pin"
    run_cr --dry-run mnemonic-toolkit-v0.105.2
    x_rc 3
    x_err "cut-release: scripts/install.sh self-pins mnemonic-toolkit-v0.105.0 but Cargo.toml says 0.105.1; land the previous release's pins PR first"
}
cr_push_pvs_no_ok_line() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    FAKE_PVS=silent run_cr --push ms-cli-v0.20.3
    x_rc 5
    x_err "cut-release: push-via-staging.sh exited 0 without its '== OK:' line; not tagging"
    x_no_remote_tag ms-cli-v0.20.3
}
cr_push_pvs_failed_after_landing() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    FAKE_PVS=landed-fail run_cr --push ms-cli-v0.20.3
    x_rc 5
    x_err "cut-release: push-via-staging.sh exited 1; no tag was pushed."
    x_err "cut-release: the release commit IS on origin/master. If the output above shows a bypass, STOP and report it; otherwise re-run the same command: it finds the commit on master, re-stages it and tags."
    x_no_remote_tag ms-cli-v0.20.3
    local first
    first=$(bsha refs/heads/master)
    run_cr --push ms-cli-v0.20.3
    x_rc 0
    x_out "== ms-cli is already 0.20.3 (crates/ms-cli/Cargo.toml); no version edit"
    x_eq "$(bsha refs/tags/ms-cli-v0.20.3^{commit})" "$first" "the tag peels to the first run's commit"
    x_eq "$(bsha refs/heads/master)" "$first" "bare master after the second run"
}
cr_toolkit_pvs_failed_after_landing() {
    fresh; fx_tk; mk_assets mnemonic-toolkit-v0.105.1; mk_assets mnemonic-toolkit-v0.105.2
    FAKE_PVS=landed-fail run_cr --push mnemonic-toolkit-v0.105.2
    x_rc 5
    local c b
    c=$(git -C "$FXDIR" rev-parse HEAD); b=$(git -C "$FXDIR" rev-parse HEAD~1)
    x_err "cut-release: the toolkit release commit (b) $(s12 "$b") IS on origin/master; its tag goes on (c) $(s12 "$c"), which is only on local master (do not reset it). If the output above shows a bypass, STOP and report it."
    x_err "cut-release: otherwise finish by hand: git tag -a mnemonic-toolkit-v0.105.2 -m 'mnemonic-toolkit 0.105.2' $c && git push origin refs/tags/mnemonic-toolkit-v0.105.2; when its release, man-pages, install-pin-check and changelog-check runs are completed/success (gh run list --repo bg002h/mnemonic-toolkit --branch mnemonic-toolkit-v0.105.2): cut-release.sh --verify mnemonic-toolkit-v0.105.2, then the pins-PR line below."
    [ -f "$(gitdir)/cut-release/pins-pr-body.md" ] || rowfail "pins-pr-body.md does not exist"
    x_no_remote_tag mnemonic-toolkit-v0.105.2
    local cmd
    cmd=$(grep -F 'cut-release: otherwise finish by hand: ' "$ERR" | sed 's/^cut-release: otherwise finish by hand: //; s/; when its .*$//')
    (cd "$FXDIR" && bash -c "$cmd") >/dev/null 2>&1 || rowfail "the printed tag commands failed"
    x_eq "$(bsha refs/tags/mnemonic-toolkit-v0.105.2^{commit})" "$c" "the tag peels to (c)"
}
cr_push_tag_push_fails() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    printf '#!/bin/sh\nwhile read old new ref; do\n  case "$ref" in refs/tags/*) echo "tags rejected here" >&2; exit 1 ;; esac\ndone\nexit 0\n' >"$FXBARE/hooks/pre-receive"
    chmod +x "$FXBARE/hooks/pre-receive"
    run_cr --push ms-cli-v0.20.3
    x_rc 6
    x_err "cut-release: pushing the tags failed (atomic; none is on origin):"
    x_err "cut-release: re-run the same command; it finds the release commit on master and tags it"
    git -C "$FXDIR" rev-parse -q --verify refs/tags/ms-cli-v0.20.3 >/dev/null 2>&1 && rowfail "the local tag still exists"
    x_eq "$(git -C "$FXDIR" rev-parse HEAD)" "$(bsha refs/heads/master)" "bare master vs the release commit"
}
cr_push_run_never_appears() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    FAKE_GH_WORKFLOWS=fuzz-smoke run_cr --push ms-cli-v0.20.3
    x_rc 7
    x_err "cut-release: no 'release' run appeared for ms-cli-v0.20.3 within 10 minutes"
    x_err "$(finish_line ms-cli-v0.20.3)"
}
cr_push_interrupted_after_tag() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    ( cd "$FXDIR" && FAKE_GH_KILL=TERM bash -c 'export CUT_RELEASE_TEST_PID=$$; exec "$0" --push ms-cli-v0.20.3' "$CR" ) >"$OUT" 2>"$ERR"; RC=$?
    x_rc 7
    local n
    n=$(wc -l <"$ERR")
    x_eq "$(sed -n "$((n - 1))p" "$ERR")" "cut-release: interrupted after the tag push; the tags are pushed and stay pushed: never delete or move them." "second-to-last stderr line"
    x_last_err "$(finish_line ms-cli-v0.20.3)"
    x_eq "$(sed -n 1p "$(gitdir)/cut-release/follow-on.txt")" "== FOLLOW-ON: not done by this script; each is a commit or PR in the named repo" "follow-on.txt line 1"
    [ -n "$(git --git-dir="$FXBARE" for-each-ref refs/tags/ms-cli-v0.20.3)" ] || rowfail "tag not on the bare remote"
}
cr_toolkit_run_failed() {
    fresh; fx_tk; mk_assets mnemonic-toolkit-v0.105.1; mk_assets mnemonic-toolkit-v0.105.2
    FAKE_GH_CONCLUSION=failure run_cr --push mnemonic-toolkit-v0.105.2
    x_rc 7
    local c b
    c=$(bsha refs/tags/mnemonic-toolkit-v0.105.2^{commit}); b=$(bsha refs/heads/master)
    x_err "$(pins_line "$c" 0.105.2)"
    x_eq "$(git -C "$FXDIR" rev-parse HEAD)" "$b" "local master vs bare master"
    x_eq "$(git -C "$FXDIR" rev-parse HEAD)" "$(git --git-dir="$FXBARE" rev-parse "$c^")" "local master vs (b)"
    [ -z "$(git --git-dir="$FXBARE" for-each-ref refs/heads/release-pins-0.105.2)" ] || rowfail "release-pins-0.105.2 is on the bare remote"
}
cr_push_release_draft() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    FAKE_GH_DRAFT=1 run_cr --push ms-cli-v0.20.3
    x_rc 8; x_out "   FAIL  release is a draft"
}
cr_push_trusted_comment() {
    fresh; fx_ms; mk_prev_ms
    MK_ASSETS_COMMENT='ms ms-cli-v0.20.2 SHA256SUMS.x86_64' mk_assets ms-cli-v0.20.3
    run_cr --push ms-cli-v0.20.3
    x_rc 8
    x_out "   FAIL  SHA256SUMS.x86_64 trusted comment is 'ms ms-cli-v0.20.2 SHA256SUMS.x86_64', not '... ms-cli-v0.20.3 SHA256SUMS.x86_64'"
}
cr_push_checksum_mismatch() {
    fresh; fx_ms; mk_prev_ms
    MK_ASSETS_TAMPER=1 mk_assets ms-cli-v0.20.3
    run_cr --push ms-cli-v0.20.3
    x_rc 8; x_out "   FAIL  ms-0.20.3-x86_64-linux-musl.tar.gz does not match SHA256SUMS.x86_64"
}
cr_push_wrong_binary_version() {
    fresh; fx_ms; mk_prev_ms
    MK_ASSETS_BINVER=0.20.2 mk_assets ms-cli-v0.20.3
    run_cr --push ms-cli-v0.20.3
    x_rc 8; x_out "   FAIL  ms --version printed 'ms 0.20.2', not 'ms 0.20.3'"
}
cr_version_skip() {
    fresh; fx_ms
    run_cr --dry-run ms-cli-v0.20.4
    x_rc 3
    x_err "cut-release: ms-cli-v0.20.4: ms-cli is 0.20.2 in crates/ms-cli/Cargo.toml; the next version is one of 0.20.3 0.21.0 1.0.0 (a skip is released by hand)"
}
cr_land_pr_not_clean() {
    tk_pushed
    FAKE_PR_MERGE_STATE=BLOCKED run_cr --land-pr 77
    x_rc 9; x_err "cut-release: PR #77 merge state is BLOCKED, not CLEAN"
    ! grep -q '^pr merge' "$T/gh.log" || rowfail "pr merge is in gh.log"
}
cr_land_pr_branch_moved() {
    tk_pushed
    local c
    c=$(bsha refs/heads/release-pins-0.105.2)
    FAKE_PR_BRANCH_MOVES=1 run_cr --land-pr 77
    x_rc 0
    x_out "== merged PR #77 (https://github.com/bg002h/mnemonic-toolkit/pull/77) as a merge commit; release-pins-0.105.2 left in place (origin's release-pins-0.105.2 is $(s12 "$(bsha refs/heads/release-pins-0.105.2)"), not the merged $(s12 "$c"))"
    [ -n "$(git --git-dir="$FXBARE" for-each-ref refs/heads/release-pins-0.105.2)" ] || rowfail "the bare branch is gone"
}
cr_follow_on_engrave_demo() {
    fresh; fx_ms
    local sib=$T/sib/mnemonic-engrave bare=$T/sib/mnemonic-engrave.git
    mkdir -p "$T/sib"
    git init -q --bare -b master "$bare"
    git init -q -b master "$sib"
    git -C "$sib" remote add origin "$bare"
    mkdir -p "$sib/demo/sh2/src"
    cp "$FIX/engrave-index.html" "$sib/demo/sh2/src/index.html"
    cp "$FIX/engrave-CLI_SPINE.md" "$sib/demo/sh2/CLI_SPINE.md"
    echo readme >"$sib/README.md"
    git -C "$sib" add -A; git -C "$sib" commit -q -m demo; git -C "$sib" push -q origin HEAD:refs/heads/master
    echo local-edit >>"$sib/README.md"
    CUT_RELEASE_SIBLINGS=$T/sib run_cr --dry-run ms-cli-v0.20.3
    x_rc 0
    local sed_line commit_line
    sed_line=$(grep -F "     sed -i 's#" "$OUT" | sed 's/^ *//' | sed -n 1p)
    commit_line=$(grep -F "     git commit -m 'demo/sh2: " "$OUT" | sed 's/^ *//' | sed -n 1p)
    [ -n "$sed_line" ] && [ -n "$commit_line" ] || { rowfail "no printed (a) sed or commit line"; return; }
    (cd "$sib" && bash -c "$sed_line" && bash -c "$commit_line") >/dev/null 2>&1 || rowfail "the printed (a) lines failed"
    x_eq "$(git -C "$sib" show --name-only --format= HEAD | sort | tr '\n' ' ')" "demo/sh2/CLI_SPINE.md demo/sh2/src/index.html " "files in the commit"
    x_eq "$(git -C "$sib" show --numstat --format= HEAD | awk '{s += $1} END {print s}')" 7 "lines changed"
    x_eq "$(git -C "$sib" status --porcelain)" " M README.md" "status after the commit"
    x_eq "$(grep -c 'ms-cli-v0.20.2' "$sib/demo/sh2/src/index.html" || true)" 0 "old tag left in index.html"
    x_eq "$(grep -c 'ms-cli-v0.20.2' "$sib/demo/sh2/CLI_SPINE.md" || true)" 0 "old tag left in CLI_SPINE.md"
}
cr_stale_staging() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    git clone -q "$FXBARE" "$T/clone2"
    echo stale >"$T/clone2/stale.txt"
    git -C "$T/clone2" add -A; git -C "$T/clone2" commit -q -m stale
    git -C "$T/clone2" push -q origin HEAD:refs/heads/ci/staging
    local s head tip
    s=$(bsha refs/heads/ci/staging); head=$(git -C "$FXDIR" rev-parse HEAD); tip=$(bsha refs/heads/master)
    run_cr --push ms-cli-v0.20.3
    x_rc 3; x_err "$(P23_TEXT "$(s12 "$s")")"
    x_no_remote_tag ms-cli-v0.20.3
    run_cr --dry-run ms-cli-v0.20.3
    x_rc 3; x_err "$(P23_TEXT "$(s12 "$s")")"
    x_eq "$(bsha refs/heads/master)" "$tip" "bare master"
    x_eq "$(git -C "$FXDIR" rev-parse HEAD)" "$head" "HEAD"
    [ -z "$(git -C "$FXDIR" status --porcelain)" ] || rowfail "tree not clean"
    [ ! -s "$T/pvs.log" ] || rowfail "pvs.log is not empty"
    # M2: S = HEAD and S behind HEAD are allowed
    git --git-dir="$FXBARE" update-ref refs/heads/ci/staging "$head"
    run_cr --dry-run ms-cli-v0.20.3; x_rc 0
    git -C "$FXDIR" commit -q --allow-empty -m child
    git -C "$FXDIR" push -q origin "HEAD:refs/heads/master"
    git --git-dir="$FXBARE" update-ref refs/heads/ci/staging "$head"
    run_cr --dry-run ms-cli-v0.20.3; x_rc 0
    [ ! -s "$T/pvs.log" ] || rowfail "pvs.log is not empty after the allowed legs"
}
cr_push_gh_fails_after_tag() {
    fresh; fx_ms; mk_prev_ms; mk_assets ms-cli-v0.20.3
    FAKE_GH_FAIL=1 run_cr --push ms-cli-v0.20.3
    x_rc 7
    x_err "cut-release: stopped after the tag push by a failing command (above); the tags are pushed and stay pushed: never delete or move them."
    x_err "$(finish_line ms-cli-v0.20.3)"
    [ -n "$(git --git-dir="$FXBARE" for-each-ref refs/tags/ms-cli-v0.20.3)" ] || rowfail "tag not on the bare remote"
    [ -f "$(gitdir)/cut-release/follow-on.txt" ] || rowfail "follow-on.txt is missing"
    # the toolkit leg
    fresh; fx_tk; mk_assets mnemonic-toolkit-v0.105.1; mk_assets mnemonic-toolkit-v0.105.2
    FAKE_GH_FAIL=1 run_cr --push mnemonic-toolkit-v0.105.2
    x_rc 7
    local c b
    c=$(bsha refs/tags/mnemonic-toolkit-v0.105.2^{commit}); b=$(bsha refs/heads/master)
    x_err "cut-release: stopped after the tag push by a failing command (above); the tags are pushed and stay pushed: never delete or move them."
    x_err "$(finish_line mnemonic-toolkit-v0.105.2)"
    x_err "$(pins_line "$c" 0.105.2)"
    [ -f "$(gitdir)/cut-release/follow-on.txt" ] || rowfail "follow-on.txt is missing (toolkit)"
    x_eq "$(git -C "$FXDIR" rev-parse HEAD)" "$b" "local master vs bare master"
    x_eq "$(git --git-dir="$FXBARE" rev-parse "$c^")" "$b" "(b) is master"
}

# --------------------------------------------------------------- network rows
CUT_RELEASE_SIBLINGS_REAL=${CUT_RELEASE_SIBLINGS_REAL:-/scratch/code/shibboleth}
net_clone() { # repo default : a clone of the real repo whose origin is a bare copy
    local repo=$1 def=$2
    FXREPO=$repo; FXDEF=$def
    FXDIR=$T/work/$repo; FXBARE=$T/remote/$repo.git
    rm -rf "$FXDIR" "$FXBARE"
    GIT_CONFIG_GLOBAL=/dev/null git clone -q --bare --reference-if-able "$CUT_RELEASE_SIBLINGS_REAL/$repo" "https://github.com/bg002h/$repo" "$FXBARE" || return 1
    git clone -q "$FXBARE" "$FXDIR" || return 1
    (cd "$FXDIR" && cargo fetch --locked >/dev/null 2>&1) || return 1
}
net_version() { # manifest -> X from the [package] section
    awk '/^\[/ { sec = $0 } sec == "[package]" && /^version = / { v = $3; gsub(/"/, "", v); print v; exit }' "$FXDIR/$1"
}
net_next() { # X -> patch+1, or the triple when X has a suffix
    local x=$1 ma mi pa
    if [[ $x == *-* ]]; then printf '%s' "${x%%-*}"; return; fi
    IFS=. read -r ma mi pa <<<"$x"
    printf '%s.%s.%s' "$ma" "$mi" "$((pa + 1))"
}
crn_verify_ms() {
    fresh; FXDIR=$T/none
    run_cr_net --verify ms-cli-v0.20.2
    x_rc 0
    local s
    for s in x86_64 aarch64 portable; do
        x_out "   ok    SHA256SUMS.$s signed: Trusted comment: ms ms-cli-v0.20.2 SHA256SUMS.$s"
    done
    x_out "   ok    every asset of ms-cli-v0.20.1 is present (names with 0.20.1 -> 0.20.2)"
    x_out "   ok    ms-0.20.2-x86_64-linux-musl.tar.gz matches SHA256SUMS.x86_64"
    x_out "   ok    RAN ms --version -> ms 0.20.2"
    x_out "== verify ms-cli-v0.20.2: 7 ok, 0 failed"
}
crn_verify_mt() {
    fresh; FXDIR=$T/none
    run_cr_net --verify mt-cli-v0.1.1
    x_rc 0
    x_out "   ok    SHA256SUMS.portable signed: Trusted comment: mt mt-cli-v0.1.1 SHA256SUMS.portable"
    x_out "== verify mt-cli-v0.1.1: 5 ok, 0 failed"
}
crn_verify_md_unsigned() {
    fresh; FXDIR=$T/none
    run_cr_net --verify descriptor-mnemonic-md-cli-v0.20.3
    x_rc 8
    x_out "   FAIL  no SHA256SUMS.aarch64.minisig asset (unsigned release)"
    x_out "   FAIL  no SHA256SUMS.portable.minisig asset (unsigned release)"
    x_out "   ok    RAN md --version -> md 0.20.3"
    x_out "== verify descriptor-mnemonic-md-cli-v0.20.3: 4 ok, 2 failed"
}
crn_verify_engrave_retired() {
    fresh; FXDIR=$T/none
    run_cr_net --verify v0.12.0
    x_rc 8
    x_out "   FAIL  SHA256SUMS.minisig does not verify:"
    grep -Fq -- "Signature key id in SHA256SUMS.minisig is CA39ECB257009A0F" "$OUT" || rowfail "no minisign key-id line"
    x_out "   ok    RAN me --version -> me 0.12.0"
}
crn_verify_all() {
    fresh; FXDIR=$T/none
    run_cr_net --verify
    local last
    last=$(tail -n 1 "$OUT")
    if [ "$RC" -eq 0 ]; then
        x_eq "$last" "== verify: 7 release(s), 0 failed" "last line"
    else
        x_rc 8
    fi
    x_eq "$(grep -c '^== verify [^:]*:' "$OUT" || true)" 7 "== verify <tag>: lines"
}
crn_real_mt_dry() {
    fresh
    net_clone mnemonic-transaction main || { rowfail "clone or cargo fetch failed"; return; }
    local X N
    X=$(net_version crates/mt-cli/Cargo.toml); N=$(net_next "$X")
    snap
    run_cr_net --dry-run "mt-cli-v$N"
    x_rc 0
    x_out "== mt-cli $X -> $N (crates/mt-cli/Cargo.toml)"
    x_out "== built: mt --version = 'mt $N'"
    x_out "== DRY RUN OK: nothing was pushed; tracked files are as they were"
    [ -z "$(git -C "$FXDIR" status --porcelain)" ] || rowfail "tree not clean"
}
crn_real_ms_dry() {
    fresh
    net_clone mnemonic-secret master || { rowfail "clone or cargo fetch failed"; return; }
    local Xc X Nc N f=$FXDIR/CHANGELOG.md first
    Xc=$(net_version crates/ms-codec/Cargo.toml); Nc=$(net_next "$Xc")
    X=$(net_version crates/ms-cli/Cargo.toml); N=$(net_next "$X")
    first=$(grep -n '^## ' "$f" | sed -n 1p | cut -d: -f1)
    insert_above "$f" "$first" '## ms-codec [Unreleased]' '' '- test.' '' '## ms-cli [Unreleased]' '' '- test.' ''
    fx_commit "test entries"
    run_cr_net --dry-run "ms-codec-v$Nc" "ms-cli-v$N"
    x_rc 0
    x_out "== pin: crates/ms-cli/Cargo.toml ms-codec =$Xc -> =$Nc"
    x_out "== Cargo.lock: 2 version line(s) changed (ms-codec ms-cli)"
    x_out "== built: ms --version = 'ms $N'"
    [ -z "$(git -C "$FXDIR" status --porcelain)" ] || rowfail "tree not clean"
}
crn_real_tk_dry() {
    fresh
    net_clone mnemonic-toolkit master || { rowfail "clone or cargo fetch failed"; return; }
    local X N f=$FXDIR/CHANGELOG.md
    X=$(net_version crates/mnemonic-toolkit/Cargo.toml); N=$(net_next "$X")
    cl_body=$(cd "$FXDIR" && awk '$0 == "## [Unreleased]" { u = 1; next } u && /^## / { exit } u && NF { n++ } END { print n + 0 }' CHANGELOG.md)
    if [ "$cl_body" -eq 0 ]; then
        local ln
        ln=$(grep -nx '## \[Unreleased\]' "$f" | cut -d: -f1 | sed -n 1p)
        insert_above "$f" "$((ln + 1))" '' '- test.'
        fx_commit "test entry"
    fi
    run_cr_net --dry-run "mnemonic-toolkit-v$N"
    x_rc 0
    x_out_start "== toolkit (b): README markers 2,"
    local ok=0
    grep -Fxq -- "== toolkit (c): check_cli_pins.py OK" "$OUT" && ok=$((ok + 1))
    grep -Fxq -- "== toolkit (c): check_cli_pins.py fails on (c); the pins PR owes the GUI manual's {#installer-ahead} section (listed below)" "$OUT" && ok=$((ok + 1))
    x_eq "$ok" 1 "check_cli_pins.py lines shown"
    [ -z "$(git -C "$FXDIR" status --porcelain)" ] || rowfail "tree not clean"
}

# ----------------------------------------------------------------------- run
ROW_LIST=(
    cr_usage_no_mode cr_usage_two_modes cr_usage_no_tags cr_help cr_verify_lib_tag
    cr_not_in_table cr_wrong_prefix cr_not_semver cr_duplicate_crate cr_not_default_branch
    cr_behind_origin cr_dirty_tracked cr_tag_on_origin cr_not_newer_than_cargo cr_not_newer_than_tag
    cr_no_unreleased cr_unreleased_empty cr_unreleased_below_release cr_unreleased_and_heading cr_next_release_mismatch
    cr_codec_entries_unreleased cr_stale_pin cr_toolchain_mismatch cr_push_tooling_dirty cr_dry_run_ms_two_tags
    cr_dry_run_resume cr_wrong_version_output cr_fn_lock_diff_check cr_fn_promote_plain cr_fn_promote_next_release
    cr_fn_tk_examples cr_fn_tk_repad cr_push_ms_happy cr_push_staging_red cr_push_bypass_line
    cr_push_run_failed cr_push_bad_signature cr_push_missing_minisig cr_push_missing_asset cr_toolkit_dry_run
    cr_toolkit_pins_ok cr_toolkit_push_happy cr_toolkit_install_assets_fail cr_land_pr_draft cr_land_pr_failed_check
    cr_land_pr_happy cr_land_pr_tag_not_in_pr cr_dry_run_mt cr_static_push cr_bash_n
    cr_ahead_of_origin cr_toolkit_already_bumped cr_toolkit_selfpin_lags cr_push_pvs_no_ok_line cr_push_pvs_failed_after_landing
    cr_toolkit_pvs_failed_after_landing cr_push_tag_push_fails cr_push_run_never_appears cr_push_interrupted_after_tag cr_toolkit_run_failed
    cr_push_release_draft cr_push_trusted_comment cr_push_checksum_mismatch cr_push_wrong_binary_version cr_version_skip
    cr_land_pr_not_clean cr_land_pr_branch_moved cr_follow_on_engrave_demo cr_stale_staging cr_push_gh_fails_after_tag
)
NET_LIST=(
    crn_verify_ms crn_verify_mt crn_verify_md_unsigned crn_verify_engrave_retired crn_verify_all
    crn_real_mt_dry crn_real_ms_dry crn_real_tk_dry
)

run_row() {
    local row=$1
    ROWERR=""
    : >"$OUT"; : >"$ERR"
    RUN=$((RUN + 1))
    "$row" || true
    if [ -z "$ROWERR" ]; then
        PASSED=$((PASSED + 1)); echo "ok $row"
    else
        FAILED=$((FAILED + 1)); echo "FAIL $row: $ROWERR"
    fi
}

for r in "${ROW_LIST[@]}"; do run_row "$r"; done
if [ "$OFFLINE" -eq 0 ]; then
    for r in "${NET_LIST[@]}"; do run_row "$r"; done
fi

echo "cut-release.test: $PASSED passed, $FAILED failed"
if [ "$FAILED" -ne 0 ]; then KEEP=1; exit 1; fi
if [ "$RUN" -ne "$EXPECT" ]; then
    echo "cut-release.test: ran $RUN rows, expected $EXPECT" >&2
    KEEP=1
    exit 1
fi
exit 0
