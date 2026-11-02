#!/usr/bin/env bash
# cut-release.sh -- one release script for every constellation repo.
#
# Spec: design/SPEC_cut_release.md (this repo). The spec decides every
# operator-visible text, refusal, exit code and order; this file only says how.
# One copy, here; other repos reach it by absolute path and run it from inside
# a checkout of the repo the tags belong to.
#
#   cut-release.sh --dry-run [--trailer 'Key: value']... <tag>...
#   cut-release.sh --push    [--trailer 'Key: value']... <tag>...
#   cut-release.sh --verify  [<tag>...]
#   cut-release.sh --land-pr <number>
#
# The release table in section 3 of the spec is the only per-repo knowledge
# here. The script never writes release prose, never edits another repo, and
# never names a CI context: the staging helper discovers those itself.
set -euo pipefail

SCRIPT_PATH=$(readlink -f "${BASH_SOURCE[0]}")
SCRIPT_DIR=$(dirname "$SCRIPT_PATH")

CR_POLL=${CUT_RELEASE_POLL_SECONDS:-15}
CR_PVS=${CUT_RELEASE_PUSH_VIA_STAGING:-$SCRIPT_DIR/push-via-staging.sh}
CR_PUBKEY_FILE=${CUT_RELEASE_PUBKEY_FILE:-$SCRIPT_DIR/../minisign.pub}
CR_CACHE=${CUT_RELEASE_CACHE:-${XDG_CACHE_HOME:-$HOME/.cache}/cut-release}

# ---------------------------------------------------------------- the table
# repo|default|crate|manifest|prefix|bin|changelog|U|H template|workflows|archive|sums
# A '-' means none. @N@ is the version, @D@ the date. Tag workflows are the
# names that are waited on; other workflows are only listed afterwards.
CR_ROWS=(
'mnemonic-engrave|master|mnemonic-engrave|crates/me-cli/Cargo.toml|v|me|crates/me-cli/CHANGELOG.md|## [Unreleased]|## [@N@] - @D@|release|mnemonic-engrave-v@N@-linux-amd64.tar.gz|SHA256SUMS'
'mnemonic-engrave|master|mnemonic-io-lib|crates/mnemonic-io-lib/Cargo.toml|mnemonic-io-lib-v|-|-|-|-|-|-|-'
'mnemonic-toolkit|master|mnemonic-toolkit|crates/mnemonic-toolkit/Cargo.toml|mnemonic-toolkit-v|mnemonic|CHANGELOG.md|## [Unreleased]|## mnemonic-toolkit [@N@] — @D@|release man-pages install-pin-check changelog-check|mnemonic-@N@-x86_64-linux-musl.tar.gz|SHA256SUMS.x86_64'
'mnemonic-secret|master|ms-cli|crates/ms-cli/Cargo.toml|ms-cli-v|ms|CHANGELOG.md|## ms-cli [Unreleased]|## ms-cli [@N@] — @D@|release man-release|ms-@N@-x86_64-linux-musl.tar.gz|SHA256SUMS.x86_64'
'mnemonic-secret|master|ms-codec|crates/ms-codec/Cargo.toml|ms-codec-v|-|CHANGELOG.md|## ms-codec [Unreleased]|## ms-codec [@N@] — @D@|-|-|-'
'descriptor-mnemonic|main|md-cli|crates/md-cli/Cargo.toml|descriptor-mnemonic-md-cli-v|md|CHANGELOG.md|## md-cli [Unreleased]|## md-cli [@N@] — @D@|release man-pages|md-@N@-linux-amd64.tar.gz|SHA256SUMS.portable'
'descriptor-mnemonic|main|md-codec|crates/md-codec/Cargo.toml|md-codec-v|-|CHANGELOG.md|## md-codec [Unreleased]|## md-codec [@N@] — @D@|-|-|-'
'mnemonic-key|main|mk-cli|crates/mk-cli/Cargo.toml|mk-cli-v|mk|crates/mk-cli/CHANGELOG.md|## [Unreleased]|## [@N@] — @D@|release musl-binaries CI|mk-@N@-x86_64-linux-musl.tar.gz|SHA256SUMS.x86_64'
'mnemonic-key|main|mk-codec|crates/mk-codec/Cargo.toml|mk-codec-v|-|crates/mk-codec/CHANGELOG.md|## [Unreleased]|## [@N@] — @D@|CI|-|-'
'mnemonic-transaction|main|mt-cli|crates/mt-cli/Cargo.toml|mt-cli-v|mt|-|-|-|release|mt-@N@-linux-amd64.tar.gz|SHA256SUMS.portable'
'mnemonic-gui|master|mnemonic-gui|Cargo.toml|mnemonic-gui-v|mnemonic-gui|CHANGELOG.md|## Unreleased|## mnemonic-gui [@N@] — @D@|build schema-mirror|mnemonic-gui-v@N@-x86_64-linux.tar.gz|SHA256SUMS'
)
# Engrave's own archive is named with the tag (vN), the others with N.
CR_BIN_PREFIXES=(v mnemonic-toolkit-v ms-cli-v descriptor-mnemonic-md-cli-v mk-cli-v mt-cli-v mnemonic-gui-v)
# Sibling clones scanned for the follow-on block: name default
CR_SIBLINGS=(
'mnemonic-engrave master' 'mnemonic-toolkit master' 'mnemonic-secret master'
'descriptor-mnemonic main' 'mnemonic-key main' 'mnemonic-transaction main'
'mnemonic-gui master' 'mnemonic-refugium master' 'seedhammer main'
)
CR_PATHSPECS=(. ':!design' ':!vendor' ':!**/CHANGELOG.md' ':!**/agent-reports/**' ':!**/*release-history*' ':!**/cli-version-history.txt' ':!**/Cargo.lock')

# state shared by the functions below
MODE=""
TAGS=()
TRAILERS=()
PR_NUMBER=""
declare -A NEWVER=()
TOUCHED=()
LOCAL_ACTIVE=0
FREEZE_ACTIVE=0
POST_TAG=0
DONE_FLAG=0
HANDLED=0
PINS_OPENED=0
SKIP_PINS=0
IS_TK=0
B_SHA=""
C_SHA=""
GITDIR=""
TOP=""
REPO=""
DEFAULT=""
SLUG=""
ENGRAVE_TOP=""
ENGRAVE_12=""
BIN_TAGS=""

# ----------------------------------------------------------------- helpers
say() { printf '%s\n' "$*"; }
err() { printf 'cut-release: %s\n' "$*" >&2; }
indent3() { sed 's/^/   /' >&2; }
indent9() { sed 's/^/         /'; }
refuse() { local code=$1; shift; err "$*"; exit "$code"; }
trunc12() { printf '%s' "${1:0:12}"; }

usage() {
  cat <<'CR_USAGE_END'
usage: cut-release.sh --dry-run [--trailer 'Key: value']... <tag>...
       cut-release.sh --push    [--trailer 'Key: value']... <tag>...
       cut-release.sh --verify  [<tag>...]
       cut-release.sh --land-pr <number>

Run --dry-run, --push and --land-pr from inside a checkout of the repo the
tags belong to. The release notes must already be under the crate's
[Unreleased] heading on the default branch; the script never writes prose.
--dry-run changes nothing remote and leaves tracked files as they were.
--push commits, lands the commit through push-via-staging.sh, pushes the
tags by full ref, waits for their release workflows, verifies the published
release and lists the follow-on pin moves.
--verify checks published releases (no tag: the newest release of every CLI).
--land-pr merges a pull request once every check is green.
Exit: 0 ok, 2 usage, 3 refused before any change, 4 a local check failed,
5 staging failed (no tag pushed), 6 tag push failed (no tag pushed),
CR_USAGE_END
}

# row_split <row string>: sets R_* from one table row
row_split() {
  IFS='|' read -r R_REPO R_DEFAULT R_CRATE R_MANIFEST R_PREFIX R_BIN R_CHLOG R_U R_HT R_WF R_ARCH R_SUMS <<<"$1"
}
# row_by_prefix <prefix>: loads the row owning that tag prefix; returns 1 if none
row_by_prefix() {
  local r
  for r in "${CR_ROWS[@]}"; do
    row_split "$r"
    if [ "$R_PREFIX" = "$1" ]; then return 0; fi
  done
  return 1
}
# parse_tag <tag>: PT_PREFIX / PT_N, returns 1 when the tag has no semver tail
parse_tag() {
  if [[ $1 =~ ^([^0-9]*)([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
    PT_PREFIX=${BASH_REMATCH[1]}
    PT_N=${BASH_REMATCH[2]}
    return 0
  fi
  return 1
}
triple() { printf '%s' "${1%%-*}"; }
# tcmp <a> <b>: prints -1, 0 or 1 comparing the numeric triples
tcmp() {
  local IFS=.
  local -a a b
  local i
  a=($(triple "$1"))
  b=($(triple "$2"))
  for i in 0 1 2; do
    if (( 10#${a[i]:-0} > 10#${b[i]:-0} )); then printf '1'; return 0; fi
    if (( 10#${a[i]:-0} < 10#${b[i]:-0} )); then printf -- '-1'; return 0; fi
  done
  printf '0'
}
re_escape() { printf '%s' "$1" | sed 's/[][\.*^$/+?(){}|]/\\&/g'; }
# H prefix / heading for the current row (R_HT) and version $1, date $2
h_prefix() { local t=$R_HT; printf '%s' "${t%%@N@*}$1]"; }
h_own_prefix() { local t=$R_HT; printf '%s' "${t%%@N@*}"; }
h_heading() { local t=$R_HT; t=${t//@N@/$1}; t=${t//@D@/$2}; printf '%s' "$t"; }
archive_name() { local a=$R_ARCH; printf '%s' "${a//@N@/$1}"; }

# --------------------------------------------------------------- arguments
parse_args() {
  local a bad_opt="" bad_trailer="" trailer_bad=0
  local -a pos=()
  local modes=0
  while [ "$#" -gt 0 ]; do
    a=$1; shift
    case "$a" in
      --help) usage; exit 0 ;;
      --dry-run|--push|--verify|--land-pr)
        modes=$((modes + 1)); MODE=${a#--} ;;
      --trailer)
        if [ "$#" -gt 0 ]; then
          if [[ $1 != *": "* ]] && [ "$trailer_bad" -eq 0 ]; then trailer_bad=1; bad_trailer=$1; fi
          TRAILERS+=("$1"); shift
        elif [ "$trailer_bad" -eq 0 ]; then
          trailer_bad=1; bad_trailer=""
        fi ;;
      -*) if [ -z "$bad_opt" ]; then bad_opt=$a; fi ;;
      *) pos+=("$a") ;;
    esac
  done
  if [ "$modes" -ne 1 ]; then
    refuse 2 "give exactly one of --dry-run, --push, --verify, --land-pr (see --help)"
  fi
  if { [ "$MODE" = dry-run ] || [ "$MODE" = push ]; } && [ "${#pos[@]}" -eq 0 ]; then
    refuse 2 "name at least one tag"
  fi
  if [ "$trailer_bad" -eq 1 ]; then
    refuse 2 "--trailer needs 'Key: value', got '$bad_trailer'"
  fi
  if { [ "$MODE" = verify ] || [ "$MODE" = land-pr ]; } && [ "${#TRAILERS[@]}" -gt 0 ]; then
    refuse 2 "--trailer goes only with --dry-run or --push"
  fi
  if [ -n "$bad_opt" ]; then
    refuse 2 "unknown option '$bad_opt'"
  fi
  if [ "$MODE" = land-pr ]; then
    if [ "${#pos[@]}" -ne 1 ] || [[ ! ${pos[0]} =~ ^[0-9]+$ ]]; then
      refuse 2 "--land-pr takes one PR number"
    fi
    PR_NUMBER=${pos[0]}
  fi
  if [ "$MODE" = verify ]; then
    for a in ${pos[@]+"${pos[@]}"}; do
      if parse_tag "$a" && row_by_prefix "$PT_PREFIX"; then
        if [ "$R_BIN" = "-" ]; then
          refuse 2 "$a is a library tag; it has no release assets to verify"
        fi
      else
        refuse 2 "'$a' is not a release tag of any repo in the table"
      fi
    done
  fi
  if [ "$MODE" = dry-run ] || [ "$MODE" = push ] || [ "$MODE" = verify ]; then
    TAGS=(${pos[@]+"${pos[@]}"})
  fi
}

# --------------------------------------------------------------- preflight
need_tools() {
  local t
  local -a list
  case "$MODE" in
    dry-run) list=(git cargo jq sha256sum tar sed awk) ;;
    push) list=(git cargo jq sha256sum tar sed awk gh minisign) ;;
    verify) list=(gh minisign jq sha256sum tar) ;;
    land-pr) list=(git gh jq) ;;
  esac
  for t in "${list[@]}"; do
    command -v "$t" >/dev/null 2>&1 || refuse 3 "'$t' is not on PATH"
  done
}

# tag table for the run, index-parallel arrays
T_TAG=(); T_ROW=(); T_CRATE=(); T_N=(); T_PREFIX=(); T_X=(); T_BIN=(); T_MANIFEST=()
T_PREV=(); T_PREVV=()
tags_load() {
  local i tag
  local -a seen_crate=()
  for i in "${!TAGS[@]}"; do
    tag=${TAGS[$i]}
    parse_tag "$tag" || refuse 3 "'$tag' does not end in a MAJOR.MINOR.PATCH version"
    if ! row_by_prefix "$PT_PREFIX" || [ "$R_REPO" != "$REPO" ]; then
      local pl="" r
      for r in "${CR_ROWS[@]}"; do
        row_split "$r"
        if [ "$R_REPO" = "$REPO" ]; then pl+=" $R_PREFIX"; fi
      done
      refuse 3 "'$tag' is not a release tag of $REPO (its prefixes:$pl)"
    fi
    local j
    for j in "${!T_CRATE[@]}"; do
      if [ "${T_CRATE[$j]}" = "$R_CRATE" ]; then
        refuse 3 "two tags for $R_CRATE: ${T_TAG[$j]} $tag"
      fi
    done
    T_TAG+=("$tag"); T_N+=("$PT_N"); T_PREFIX+=("$PT_PREFIX"); T_CRATE+=("$R_CRATE")
    T_BIN+=("$R_BIN"); T_MANIFEST+=("$R_MANIFEST")
    T_ROW+=("$(printf '%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s' "$R_REPO" "$R_DEFAULT" "$R_CRATE" "$R_MANIFEST" "$R_PREFIX" "$R_BIN" "$R_CHLOG" "$R_U" "$R_HT" "$R_WF" "$R_ARCH" "$R_SUMS")")
  done
}

key_line2() { sed -n '2p' "$CR_PUBKEY_FILE" 2>/dev/null || true; }

preflight_common_repo() {
  local url
  TOP=$(git rev-parse --show-toplevel 2>/dev/null) ||
    refuse 3 "not inside a git checkout; run it from the repo you are releasing"
  cd "$TOP"
  url=$(git remote get-url origin 2>/dev/null || true)
  REPO=${url%/}; REPO=${REPO##*/}; REPO=${REPO%.git}
  local r found=0
  for r in "${CR_ROWS[@]}"; do
    row_split "$r"
    if [ "$R_REPO" = "$REPO" ]; then found=1; break; fi
  done
  if [ "$found" -eq 0 ]; then
    refuse 3 "origin is '$url'; '$REPO' is not in the release table"
  fi
  DEFAULT=$R_DEFAULT
  SLUG="bg002h/$REPO"
  GITDIR=$(git rev-parse --absolute-git-dir)
  if [ "$REPO" = mnemonic-toolkit ]; then IS_TK=1; command -v python3 >/dev/null 2>&1 || refuse 3 "'python3' is not on PATH"; fi
}

check_key() {
  local k
  k=$(key_line2)
  if ! [[ $k =~ ^RW[A-Za-z0-9+/]{54}$ ]]; then
    refuse 3 "no minisign public key at $CR_PUBKEY_FILE line 2"
  fi
  PUBKEY=$k
}
check_gh_auth() {
  gh auth status >/dev/null 2>&1 || refuse 3 "gh is not logged in (gh auth login)"
}

# newest origin tag with this prefix: prints the tag or nothing
newest_origin_tag() {
  local prefix=$1 out
  out=$(git ls-remote --tags origin "refs/tags/${prefix}*" 2>/dev/null || true)
  printf '%s\n' "$out" | awk '{print $2}' | sed 's#^refs/tags/##; s#\^{}$##' |
    grep -E "^$(re_escape "$prefix")[0-9]+\.[0-9]+\.[0-9]+\$" | sort -u |
    sed "s#^$(re_escape "$prefix")##" | sort -t. -k1,1n -k2,2n -k3,3n | tail -n 1 |
    awk -v p="$prefix" 'NF{print p $0}' || true
}

preflight_dry_push() {
  local i err_f
  tags_load
  if [ "$MODE" = push ]; then
    ENGRAVE_TOP=$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null || true)
    local f bad=0
    for f in scripts/cut-release.sh scripts/push-via-staging.sh minisign.pub; do
      if [ -n "$ENGRAVE_TOP" ] && [ -n "$(git -C "$ENGRAVE_TOP" status --porcelain --untracked-files=no -- "$f" 2>/dev/null)" ]; then
        err "$f in $ENGRAVE_TOP has uncommitted changes; a release runs only committed tooling"
        bad=1
      fi
    done
    [ "$bad" -eq 0 ] || exit 3
    check_key
    check_gh_auth
  fi
  # P10
  err_f=$(mktemp)
  if ! git fetch --quiet --tags origin 2>"$err_f"; then
    err "git fetch origin failed:"
    indent3 <"$err_f"
    rm -f "$err_f"
    exit 3
  fi
  rm -f "$err_f"
  # P11
  local branch
  branch=$(git symbolic-ref --short -q HEAD 2>/dev/null || true)
  [ -n "$branch" ] || branch=HEAD
  if [ "$branch" != "$DEFAULT" ]; then
    refuse 3 "on branch '$branch'; release from $DEFAULT (git switch $DEFAULT)"
  fi
  # P12 / P12a
  local head_sha tip_sha
  head_sha=$(git rev-parse HEAD)
  tip_sha=$(git rev-parse "origin/$DEFAULT" 2>/dev/null || true)
  if [ "$head_sha" != "$tip_sha" ]; then
    if [ -n "$tip_sha" ] && git merge-base --is-ancestor "$tip_sha" HEAD; then
      local k
      k=$(git rev-list --count "origin/$DEFAULT..HEAD")
      err "HEAD $(trunc12 "$head_sha") is $k commit(s) ahead of origin/$DEFAULT $(trunc12 "$tip_sha"); the release commit must sit directly on the remote tip. A 'release: ...' commit from an interrupted cut-release.sh run: drop it (git reset --hard origin/$DEFAULT) and run again. A 'pins: installer -> ...' commit: finish as cut-release.sh printed when it stopped, and never land it without its pull request. Any other commit: land it first."
      exit 3
    fi
    refuse 3 "HEAD $(trunc12 "$head_sha") is not origin/$DEFAULT $(trunc12 "$tip_sha"); the release commit must sit directly on the remote tip (git pull --ff-only, or land your commits first)"
  fi
  # P13
  local dirty
  dirty=$(git status --porcelain --untracked-files=no)
  if [ -n "$dirty" ]; then
    err "tracked files are modified; the release commit carries only what cut-release.sh writes. Commit or restore them first:"
    printf '%s\n' "$dirty" | sed 's/^...//; s/^/   /' >&2
    exit 3
  fi
  # P14
  if [ -f rust-toolchain.toml ]; then
    local chan cv
    chan=$(sed -n 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' rust-toolchain.toml | sed -n '1p')
    if [[ $chan =~ ^[0-9][0-9.]*$ ]]; then
      cv=$(cargo --version 2>&1 | sed -n '1p' || true)
      case "$cv" in
        "cargo $chan "*) : ;;
        *) refuse 3 "cargo --version is '$cv' but rust-toolchain.toml pins $chan; put rustup's cargo first on PATH (PATH=\$HOME/.cargo/bin:\$PATH)" ;;
      esac
    fi
  fi
  # P15
  local t
  for t in "${TAGS[@]}"; do
    if [ -n "$(git ls-remote --tags origin "refs/tags/$t" 2>/dev/null || true)" ]; then
      err "tag $t already exists on origin"
      err "if an interrupted cut-release.sh --push pushed it: cut-release.sh --verify $t, then the moves in $GITDIR/cut-release/follow-on.txt"
      exit 3
    fi
    if git rev-parse -q --verify "refs/tags/$t" >/dev/null 2>&1; then
      refuse 3 "tag $t already exists locally but not on origin; delete it if you made it by hand: git tag -d $t"
    fi
  done
  # P16
  mkdir -p "$GITDIR/cut-release"
  META="$GITDIR/cut-release/metadata.json"
  err_f=$(mktemp)
  if ! cargo metadata --no-deps --format-version 1 --offline >"$META" 2>"$err_f"; then
    err "cargo metadata failed:"
    indent3 <"$err_f"
    rm -f "$err_f"
    exit 3
  fi
  rm -f "$err_f"
  WS_ROOT=$(jq -r '.workspace_root' "$META")
  TARGET_DIR=$(jq -r '.target_directory' "$META")
  for i in "${!TAGS[@]}"; do
    local x
    x=$(jq -r --arg n "${T_CRATE[$i]}" '[.packages[]|select(.name==$n)][0].version // empty' "$META")
    [ -n "$x" ] || refuse 3 "cargo metadata has no package '${T_CRATE[$i]}' (${T_MANIFEST[$i]})"
    T_X+=("$x")
    local pl pv
    pl=$(newest_origin_tag "${T_PREFIX[$i]}")
    T_PREV+=("$pl")
    pv=""
    [ -z "$pl" ] || pv=${pl#"${T_PREFIX[$i]}"}
    T_PREVV+=("$pv")
  done
  # P17, P18, P22
  for i in "${!TAGS[@]}"; do
    local X=${T_X[$i]} N=${T_N[$i]} tag=${TAGS[$i]} c
    if [ "$N" != "$X" ]; then
      c=$(tcmp "$N" "$X")
      if [ "$c" -lt 0 ] || { [ "$c" -eq 0 ] && [[ $X != *-* ]]; }; then
        refuse 3 "$tag: ${T_CRATE[$i]} is $X in ${T_MANIFEST[$i]}; $N is not newer"
      fi
    fi
  done
  for i in "${!TAGS[@]}"; do
    local N=${T_N[$i]} tag=${TAGS[$i]}
    if [ -n "${T_PREVV[$i]}" ]; then
      if [ "$(tcmp "${T_PREVV[$i]}" "$N")" -ge 0 ]; then
        refuse 3 "$tag: origin already has ${T_PREV[$i]}, which is not older than $N"
      fi
    fi
  done
  for i in "${!TAGS[@]}"; do
    local X=${T_X[$i]} N=${T_N[$i]} tag=${TAGS[$i]}
    if [ "$N" != "$X" ]; then
      if [[ $X == *-* ]]; then
        if [ "$(tcmp "$N" "$X")" -ne 0 ]; then
          refuse 3 "$tag: ${T_CRATE[$i]} is $X in ${T_MANIFEST[$i]}; the next version is $(triple "$X") (a skip is released by hand)"
        fi
      else
        local ma mi pa
        IFS=. read -r ma mi pa <<<"$X"
        local s1="$ma.$mi.$((pa + 1))" s2="$ma.$((mi + 1)).0" s3="$((ma + 1)).0.0"
        if [ "$N" != "$s1" ] && [ "$N" != "$s2" ] && [ "$N" != "$s3" ]; then
          refuse 3 "$tag: ${T_CRATE[$i]} is $X in ${T_MANIFEST[$i]}; the next version is one of $s1 $s2 $s3 (a skip is released by hand)"
        fi
      fi
    fi
  done
  # P19, P20
  if [ "$IS_TK" -eq 1 ]; then
    local X=${T_X[0]} N=${T_N[0]}
    if [ "$X" = "$N" ]; then
      refuse 3 "mnemonic-toolkit is already $N in crates/mnemonic-toolkit/Cargo.toml; the toolkit release needs its version bump in commit (b), so release it by hand or restore the previous version"
    fi
    local first
    first=$(grep -oE 'mnemonic-toolkit-v[0-9.]+' scripts/install.sh | sed -n '1p' || true)
    if [ "$first" != "mnemonic-toolkit-v$X" ]; then
      refuse 3 "scripts/install.sh self-pins $first but Cargo.toml says $X; land the previous release's pins PR first"
    fi
  fi
  # P21
  for i in "${!TAGS[@]}"; do
    local X=${T_X[$i]} c=${T_CRATE[$i]} mp req
    while IFS=$'\t' read -r mp req; do
      [ -n "$mp" ] || continue
      if [ "$req" != "=$X" ] && [ "$req" != "^$X" ] && [ "$req" != "*" ]; then
        refuse 3 "$mp pins $c at '$req'; expected '=$X', '^$X' or no version (fix the pin by hand first)"
      fi
    done < <(jq -r --arg c "$c" --arg root "$WS_ROOT/" '.packages[] | select(.source==null) | . as $p | .dependencies[] | select(.name==$c and .path!=null) | [($p.manifest_path|ltrimstr($root)), .req] | @tsv' "$META")
  done
  # P23
  local s
  s=$(git ls-remote origin refs/heads/ci/staging 2>/dev/null | awk 'NR==1{print $1}' || true)
  if [ -n "$s" ] && [ "$s" != "$head_sha" ] && ! git merge-base --is-ancestor "$s" HEAD 2>/dev/null; then
    local s12
    s12=$(trunc12 "$s")
    cat >&2 <<EOF
cut-release: origin has ci/staging at $s12; push-via-staging.sh cannot push a new release commit over it (an interrupted or failed staging run left it). If no staging run is in progress, delete it, then run again: git push origin --delete ci/staging
EOF
    exit 3
  fi
  HEAD_SHA=$head_sha
}

# ------------------------------------------------------------- CHANGELOG
# cl_scan <file> <U> <H-prefix> <own-prefix>: sets CL_* for one crate's file
cl_scan() {
  local out
  CL_NU=0; CL_NH=0; CL_ULINE=0; CL_NONEMPTY=0; CL_OWNLINE=0; CL_OWNTEXT=""; CL_FIRSTBODY=""
  if [ ! -f "$1" ]; then return 0; fi
  out=$(awk -v U="$2" -v H="$3" -v OWN="$4" '
    function rtrim(s) { sub(/[ \t\r]+$/, "", s); return s }
    BEGIN { nU = 0; nH = 0; uline = 0; inbody = 0; nonempty = 0; ownline = 0; owntext = ""; firstbody = "" }
    {
      line = $0; t = rtrim(line)
      if (t == U) { nU++; if (nU == 1) { uline = NR; inbody = 1 } else { inbody = 0 }; next }
      if (t == H || index(line, H " ") == 1) nH++
      if (index(line, OWN) == 1 && substr(line, length(OWN) + 1) ~ /^[0-9]+\.[0-9]+\.[0-9]+\]/) {
        if (ownline == 0) { ownline = NR; owntext = line }
      }
      if (index(line, "## ") == 1) { inbody = 0 }
      else if (inbody && t != "") { nonempty = 1; if (firstbody == "") firstbody = line }
    }
    END { print nU; print nH; print uline; print nonempty; print ownline; print owntext; print firstbody }
  ' "$1")
  local -a cl
  mapfile -t cl <<<"$out"
  CL_NU=${cl[0]:-0}; CL_NH=${cl[1]:-0}; CL_ULINE=${cl[2]:-0}; CL_NONEMPTY=${cl[3]:-0}
  CL_OWNLINE=${cl[4]:-0}; CL_OWNTEXT=${cl[5]:-}; CL_FIRSTBODY=${cl[6]:-}
}

# changelog_promote <file> <U-line> <heading-line>
changelog_promote() {
  local f=$1 u=$2 hd=$3
  awk -v U="$u" -v HD="$hd" '
    function rtrim(s) { sub(/[ \t\r]+$/, "", s); return s }
    BEGIN { state = 0; skip = 0; dropped = 0 }
    state == 0 { print; if (rtrim($0) == U) { print ""; print HD; state = 1 } ; next }
    state == 1 {
      if (index($0, "## ") == 1) { state = 2; print; next }
      if (skip) {
        if ($0 ~ /^[ \t\r]*$/) { skip = 0; dropped = 1 }
        next
      }
      if (!dropped && index($0, "**Next release: ") == 1) { skip = 1; next }
      print; next
    }
    { print }
  ' "$f" >"$f.cr-tmp"
  cat "$f.cr-tmp" >"$f"
  rm -f "$f.cr-tmp"
}

# changelog_check <index>: preflight C1-C6 for one tag; sets CL_STATE[i]
CL_STATE=()
changelog_check() {
  local i=$1 N=${T_N[$i]} F U hpre own
  row_split "${T_ROW[$i]}"
  if [ "$R_CHLOG" = "-" ]; then CL_STATE[$i]=none; return 0; fi
  F=$R_CHLOG; U=$R_U
  hpre=$(h_prefix "$N"); own=$(h_own_prefix)
  cl_scan "$F" "$U" "$hpre" "$own"
  if [ "$CL_NU" -gt 1 ]; then refuse 3 "$F has $CL_NU '$U' lines; keep one"; fi
  if [ "$CL_NH" -gt 1 ]; then refuse 3 "$F has $CL_NH '$hpre' headings; keep one"; fi
  if [ "$CL_NU" -eq 1 ] && [ "$CL_NONEMPTY" -eq 1 ] && [ "$CL_NH" -ge 1 ]; then
    refuse 3 "$F has both '$hpre' and entries under '$U'"
  fi
  if [ "$CL_NU" -eq 1 ] && [ "$CL_OWNLINE" -gt 0 ] && [ "$CL_OWNLINE" -lt "$CL_ULINE" ]; then
    refuse 3 "$F:$CL_ULINE: '$U' sits below '$CL_OWNTEXT'; it must be above every release heading"
  fi
  if [ "$CL_NU" -eq 1 ] && [ "$CL_NONEMPTY" -eq 1 ] && [[ $CL_FIRSTBODY == "**Next release: "* ]]; then
    if [[ $CL_FIRSTBODY != "**Next release: $N**"* ]]; then
      local v=${CL_FIRSTBODY#"**Next release: "}
      v=${v%%\*\**}
      refuse 3 "$F:$CL_ULINE: '$U' says 'Next release: $v', not $N"
    fi
  fi
  if [ "$CL_NU" -eq 1 ] && [ "$CL_NONEMPTY" -eq 0 ] && [ "$CL_NH" -eq 0 ]; then
    refuse 3 "$F:$CL_ULINE: the '$U' section is empty; nothing to release for ${T_CRATE[$i]}"
  fi
  if [ "$CL_NU" -eq 0 ] && [ "$CL_NH" -eq 0 ]; then
    refuse 3 "$F has no '$U' section; the fix PR writes the release notes there"
  fi
  if [ "$CL_NU" -eq 1 ] && [ "$CL_NONEMPTY" -eq 1 ]; then CL_STATE[$i]=promote; else CL_STATE[$i]=resume; fi
}

# C7: a crate in this run builds a changelogged workspace crate by path that
# is not in the run, and that crate has entries under its U.
changelog_c7() {
  local i c d dreq r
  for i in "${!TAGS[@]}"; do
    c=${T_CRATE[$i]}
    while IFS= read -r d; do
      [ -n "$d" ] || continue
      local j in_run=0
      for j in "${!TAGS[@]}"; do [ "${T_CRATE[$j]}" = "$d" ] && in_run=1; done
      [ "$in_run" -eq 0 ] || continue
      for r in "${CR_ROWS[@]}"; do
        row_split "$r"
        if [ "$R_REPO" = "$REPO" ] && [ "$R_CRATE" = "$d" ] && [ "$R_CHLOG" != "-" ]; then
          local dpre=$R_PREFIX dU=$R_U dF=$R_CHLOG own
          own=$(h_own_prefix)
          cl_scan "$dF" "$dU" "$(h_prefix 0.0.0)" "$own"
          if [ "$CL_NU" -ge 1 ] && [ "$CL_NONEMPTY" -eq 1 ]; then
            refuse 3 "$d has entries under '$dU' and $c builds it by path; tag $d in this same run (${dpre}MAJOR.MINOR.PATCH) or move those entries"
          fi
        fi
      done
    done < <(jq -r --arg c "$c" '.packages[] | select(.name==$c) | .dependencies[] | select(.path!=null) | .name' "$META" | sort -u)
  done
}

# ---------------------------------------------------------- Cargo.lock check
# lock_diff_check <old-lock> <new-lock> <crate>...
lock_diff_check() {
  local old=$1 new=$2
  shift 2
  local e=$# ns a r k c v
  ns=$(git diff --no-index --numstat -- "$old" "$new" || true)
  a=$(awk 'NR==1{print $1}' <<<"$ns")
  r=$(awk 'NR==1{print $2}' <<<"$ns")
  a=${a:-0}; r=${r:-0}
  if [ "$a" = "-" ]; then a=0; fi
  if [ "$r" = "-" ]; then r=0; fi
  k=$(( a > r ? a : r ))
  if [ "$a" -ne "$e" ] || [ "$r" -ne "$e" ]; then
    err "Cargo.lock changed beyond the bumped crates' version lines ($k line(s) changed, $e expected):"
    { git diff --no-index -U0 -- "$old" "$new" || true; } | grep -E '^[-+]' | grep -vE '^(\+\+\+|---) ' | indent3 || true
    return 1
  fi
  for c in "$@"; do
    v=${NEWVER[$c]:-}
    [ -n "$v" ] || continue
    if [ "$(awk -v n="$c" '/^name = /{x=$3; gsub(/"/,"",x); nm=x} /^version = /{ if (nm==n) { y=$3; gsub(/"/,"",y); print y; exit } }' "$new")" != "$v" ]; then
      err "Cargo.lock changed beyond the bumped crates' version lines ($c is not at $v in the new lock; $e expected):"
      return 1
    fi
  done
  say "== Cargo.lock: $k version line(s) changed ($*)"
}

# ------------------------------------------------------------- toolkit text
# tk_examples_b <file> <X> <N>
tk_examples_b() {
  local f=$1 X=$2 N=$3
  awk -v X="$X" -v N="$N" '
    function rf(s, pre, post,    key, out, p, rest) {
      key = pre X post; out = ""
      while ((p = index(s, key)) > 0) {
        rest = substr(s, p + length(key))
        if (post == "" && substr(rest, 1, 1) ~ /^[0-9]$/) {
          out = out substr(s, 1, p + length(key) - 1); s = rest; continue
        }
        out = out substr(s, 1, p - 1) pre N post; s = rest
      }
      return out s
    }
    {
      line = $0
      if (line == "mnemonic " X) { print "mnemonic " N; next }
      line = rf(line, "`mnemonic` v", "")
      line = rf(line, "`mnemonic` **v", "**")
      line = rf(line, "mnemonic-toolkit v", "")
      line = rf(line, "\"mnemonic ", "\"")
      line = rf(line, "expected mnemonic ", ",")
      print line
    }
  ' "$f" >"$f.cr-tmp"
  cat "$f.cr-tmp" >"$f"
  rm -f "$f.cr-tmp"
}

# tk_examples_c <file> <X> <N>
tk_examples_c() {
  local f=$1 X=$2 N=$3
  awk -v X="$X" -v N="$N" '
    function isdig(c) { return c ~ /^[0-9]$/ }
    # replace X by N where not part of a longer number
    function rep(s,    out, p, rest, prev, nxt) {
      out = ""
      while ((p = index(s, X)) > 0) {
        prev = (p > 1) ? substr(s, p - 1, 1) : ""
        rest = substr(s, p + length(X))
        nxt = substr(rest, 1, 1)
        if ((prev != "" && prev ~ /^[0-9.]$/) || (nxt != "" && isdig(nxt))) {
          out = out substr(s, 1, p + length(X) - 1); s = rest; continue
        }
        out = out substr(s, 1, p - 1) N; s = rest
      }
      return out s
    }
    function has(s, key,    p, rest, nxt) {
      while ((p = index(s, key)) > 0) {
        rest = substr(s, p + length(key)); nxt = substr(rest, 1, 1)
        if (nxt == "" || !isdig(nxt)) return 1
        s = rest
      }
      return 0
    }
    {
      inst = has($0, "mnemonic-toolkit-v" X) || has($0, "\047mnemonic " X "\047")
      if (!inst) { print; next }
      if ($0 ~ /^mnemonic +mnemonic-toolkit +/) {
        told = "mnemonic-toolkit-v" X; tnew = "mnemonic-toolkit-v" N
        p = index($0, told)
        if (p > 0) {
          before = substr($0, 1, p - 1)
          after = substr($0, p + length(told))
          match(after, /^ */)
          sp = RLENGTH
          rest2 = substr(after, sp + 1)
          pad = 37 - length(tnew); if (pad < 1) pad = 1
          if (sp == 0) pad = 0
          printf "%s%s%*s%s\n", rep(before), tnew, pad, "", rep(rest2)
          next
        }
      }
      print rep($0)
    }
  ' "$f" >"$f.cr-tmp"
  cat "$f.cr-tmp" >"$f"
  rm -f "$f.cr-tmp"
}

# lines of a file naming a version with number boundaries: "<n>:<text>"
lines_naming() {
  local f=$1 x=$2
  grep -nE "(^|[^0-9.])$(re_escape "$x")([^0-9]|\$)" "$f" || true
}

# fs_value <file|-> <short>: current value of that entry in first_signed(); rc 1 if no entry
fs_value() {
  awk -v S="$2" '
    BEGIN { inf = 0; found = 0; pat = "^[[:space:]]*" S "\\)[[:space:]]+echo \"" }
    /^first_signed\(\)/ { inf = 1; next }
    inf && /^}/ { inf = 0 }
    inf && $0 ~ pat {
      s = $0; sub(pat, "", s); sub(/".*$/, "", s)
      print s; found = 1; exit
    }
    END { exit found ? 0 : 1 }
  ' "$1"
}
fs_set() {
  local f=$1 n=$2
  awk -v N="$n" '
    BEGIN { inf = 0; done = 0 }
    /^first_signed\(\)/ { inf = 1 }
    inf && /^}/ { inf = 0 }
    inf && !done && /^[[:space:]]*mnemonic\)[[:space:]]+echo "" ;;[[:space:]]*$/ {
      sub(/echo "" ;;/, "echo \"" N "\" ;;"); done = 1
    }
    { print }
  ' "$f" >"$f.cr-tmp"
  cat "$f.cr-tmp" >"$f"
  rm -f "$f.cr-tmp"
}

# [manual-gui] toolkit-tag-implied value of a pinned-upstream.toml (stdin file arg)
toml_manual_gui_tag() {
  awk '
    /^\[/ { sec = $0; sub(/[ \t\r]+$/, "", sec) }
    sec == "[manual-gui]" && /^toolkit-tag-implied[[:space:]]*=/ { s = $0; sub(/^[^=]*=[[:space:]]*"/, "", s); sub(/".*$/, "", s); print s; exit }
  ' "$1"
}
toml_set_installer_ahead() {
  local f=$1 T=$2 has_sec has_key
  has_sec=$(grep -cE '^\[installer-ahead\][[:space:]]*$' "$f" || true)
  has_key=$(awk '/^\[/ { sec = $0; sub(/[ \t\r]+$/, "", sec) } sec == "[installer-ahead]" && /^toolkit-tag-implied[[:space:]]*=/ { c++ } END { print c + 0 }' "$f")
  if [ "$has_key" -gt 0 ]; then
    awk -v T="$T" '
      /^\[/ { sec = $0; sub(/[ \t\r]+$/, "", sec) }
      sec == "[installer-ahead]" && !done && /^toolkit-tag-implied[[:space:]]*=/ { sub(/=.*$/, "= \"" T "\""); done = 1 }
      { print }
    ' "$f" >"$f.cr-tmp"
    cat "$f.cr-tmp" >"$f"; rm -f "$f.cr-tmp"
  elif [ "$has_sec" -gt 0 ]; then
    awk -v T="$T" '
      { print }
      !done && /^\[installer-ahead\][ \t\r]*$/ { print "toolkit-tag-implied = \"" T "\""; done = 1 }
    ' "$f" >"$f.cr-tmp"
    cat "$f.cr-tmp" >"$f"; rm -f "$f.cr-tmp"
  else
    printf '\n[installer-ahead]\ntoolkit-tag-implied = "%s"\n' "$T" >>"$f"
  fi
}

# ------------------------------------------------------------ local phase
touch_file() {
  local f=$1 t
  for t in ${TOUCHED[@]+"${TOUCHED[@]}"}; do [ "$t" = "$f" ] && return 0; done
  TOUCHED+=("$f")
}
restore_touched() {
  if [ "${#TOUCHED[@]}" -gt 0 ]; then
    git restore --source=HEAD --staged --worktree -- "${TOUCHED[@]}" >/dev/null 2>&1 || true
  fi
}
# local_fail: the message lines are already printed; undo and exit 4
local_fail() {
  restore_touched
  LOCAL_ACTIVE=0
  err "tracked files restored; nothing was committed or pushed"
  exit 4
}

edit_manifest_version() {
  local f=$1 X=$2 N=$3
  awk -v X="$X" -v N="$N" '
    /^\[/ { sec = $0; sub(/[ \t\r]+$/, "", sec) }
    !done && sec == "[package]" && index($0, "version = \"" X "\"") == 1 {
      print "version = \"" N "\"" substr($0, length("version = \"" X "\"") + 1); done = 1; next
    }
    { print }
    END { exit done ? 0 : 3 }
  ' "$f" >"$f.cr-tmp" || { rm -f "$f.cr-tmp"; return 1; }
  cat "$f.cr-tmp" >"$f"
  rm -f "$f.cr-tmp"
}

local_phase() {
  local i X N crate manifest bumped=() lockcrates=() F
  LOCAL_ACTIVE=1
  # 1. versions
  for i in "${!TAGS[@]}"; do
    X=${T_X[$i]}; N=${T_N[$i]}; crate=${T_CRATE[$i]}; manifest=${T_MANIFEST[$i]}
    if [ "$X" != "$N" ]; then
      touch_file "$manifest"
      if ! edit_manifest_version "$manifest" "$X" "$N"; then
        err "cannot find version = \"$X\" in the [package] section of $manifest"
        local_fail
      fi
      say "== $crate $X -> $N ($manifest)"
      bumped+=("$i"); lockcrates+=("$crate"); NEWVER[$crate]=$N
    else
      say "== $crate is already $N ($manifest); no version edit"
    fi
  done
  # 2. pins
  local d_manifest d_req old new xe ne
  for i in ${bumped[@]+"${bumped[@]}"}; do
    X=${T_X[$i]}; N=${T_N[$i]}; crate=${T_CRATE[$i]}
    while IFS=$'\t' read -r d_manifest d_req; do
      [ -n "$d_manifest" ] || continue
      [ "$d_req" != "*" ] || continue
      xe=$(re_escape "$X"); ne=$N
      touch_file "$d_manifest"
      sed -i -E "/^$(re_escape "$crate") = \\{/ s/version = \"(=|\\^)?$xe\"/version = \"\\1$ne\"/" "$d_manifest"
      old=${d_req#^}; new=${old/"$X"/"$N"}
      say "== pin: $d_manifest $crate $old -> $new"
    done < <(jq -r --arg c "$crate" --arg root "$WS_ROOT/" '.packages[] | select(.source==null) | . as $p | .dependencies[] | select(.name==$c and .path!=null) | [($p.manifest_path|ltrimstr($root)), .req] | @tsv' "$META")
  done
  # 3. CHANGELOG
  for i in "${!TAGS[@]}"; do
    N=${T_N[$i]}
    row_split "${T_ROW[$i]}"
    if [ "$R_CHLOG" = "-" ]; then say "== CHANGELOG: none in $REPO"; continue; fi
    F=$R_CHLOG
    local hd hpre
    hd=$(h_heading "$N" "$CR_DATE"); hpre=$(h_prefix "$N")
    if [ "${CL_STATE[$i]}" = promote ]; then
      touch_file "$F"
      changelog_promote "$F" "$R_U" "$hd"
      say "== CHANGELOG: $F '$R_U' -> '$hd'"
    else
      say "== CHANGELOG: $F already has '$hpre'"
    fi
  done
  # toolkit (b) text
  if [ "$IS_TK" -eq 1 ]; then tk_local_b; fi
  # 4. cargo update
  if [ "${#bumped[@]}" -gt 0 ]; then
    mkdir -p "$GITDIR/cut-release"
    touch_file Cargo.lock
    git show HEAD:Cargo.lock >"$GITDIR/cut-release/Cargo.lock.old"
    if ! cargo update --workspace --offline >"$GITDIR/cut-release/update.log" 2>&1; then
      err "cargo update --workspace --offline failed (log $GITDIR/cut-release/update.log); run 'cargo fetch' once online, then retry"
      local_fail
    fi
    # 5. lock diff
    if ! lock_diff_check "$GITDIR/cut-release/Cargo.lock.old" Cargo.lock "${lockcrates[@]}"; then
      local_fail
    fi
    LOCK_K=${#lockcrates[@]}
  else
    LOCK_K=0
  fi
  # 6. build
  if ! cargo build --locked --workspace >"$GITDIR/cut-release/build.log" 2>&1; then
    err "cargo build --locked --workspace failed (log $GITDIR/cut-release/build.log); last 20 lines:"
    tail -n 20 "$GITDIR/cut-release/build.log" | indent3
    local_fail
  fi
  # 7. built binaries
  VERSION_CLAUSES=()
  local bin out first
  for i in "${!TAGS[@]}"; do
    bin=${T_BIN[$i]}; N=${T_N[$i]}
    [ "$bin" != "-" ] || continue
    out=$("$TARGET_DIR/debug/$bin" --version 2>&1 || true)
    first=${out%%$'\n'*}
    if [ "$first" != "$bin $N" ]; then
      err "$bin --version printed '$first', not '$bin $N'"
      local_fail
    fi
    say "== built: $bin --version = '$bin $N'"
    VERSION_CLAUSES+=("$bin --version = '$bin $N'")
  done
  # 8. engrave
  if [ "$REPO" = mnemonic-engrave ]; then
    if out=$(demo/sh2/check-install-links.sh 2>&1); then
      say "== engrave: demo/sh2/check-install-links.sh OK"
    else
      err "demo/sh2/check-install-links.sh failed; land the engrave demo follow-on for the sibling release it names first:"
      printf '%s\n' "$out" | tail -n 15 | indent3
      local_fail
    fi
  fi
  # toolkit (c)
  if [ "$IS_TK" -eq 1 ]; then tk_local_c; fi
}

# toolkit commit (b) text edits and checks (spec 6.1)
tk_local_b() {
  local X=${T_X[0]} N=${T_N[0]} f k
  local readmes=(README.md crates/mnemonic-toolkit/README.md)
  for f in "${readmes[@]}"; do
    k=$(grep -cF "<!-- toolkit-version: $X -->" "$f" || true)
    if [ "$k" -ne 1 ]; then
      err "$f must hold '<!-- toolkit-version: $X -->' exactly once, found $k"
      local_fail
    fi
    touch_file "$f"
    sed -i "s/<!-- toolkit-version: $(re_escape "$X") -->/<!-- toolkit-version: $N -->/" "$f"
  done
  touch_file .examples-build/gen.sh
  touch_file .examples-build/Examples.md
  tk_examples_b .examples-build/gen.sh "$X" "$N"
  tk_examples_b .examples-build/Examples.md "$X" "$N"
  local gk ek
  gk=$(git diff --numstat -- .examples-build/gen.sh | awk '{print $1+0}')
  ek=$(git diff --numstat -- .examples-build/Examples.md | awk '{print $1+0}')
  local left_gen left_ex inst_cnt=0 unexp=() line
  left_gen=$(lines_naming .examples-build/gen.sh "$X")
  left_ex=$(lines_naming .examples-build/Examples.md "$X")
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    unexp+=(".examples-build/gen.sh:$line")
  done <<<"$left_gen"
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    if [[ $line == *"mnemonic-toolkit-v$X"* ]] || [[ $line == *"'mnemonic $X'"* ]]; then
      inst_cnt=$((inst_cnt + 1))
    else
      unexp+=(".examples-build/Examples.md:$line")
    fi
  done <<<"$left_ex"
  if [ "${#unexp[@]}" -gt 0 ] || [ "$inst_cnt" -lt 1 ]; then
    err "toolkit (b) left ${#unexp[@]} unexpected line(s) naming $X:"
    for line in ${unexp[@]+"${unexp[@]}"}; do printf '   %s\n' "$line" >&2; done
    local_fail
  fi
  say "== toolkit (b): README markers 2, .examples-build/gen.sh $gk line(s), Examples.md $ek line(s); $inst_cnt installer line(s) left for (c)"
  TK_INST_LEFT=$inst_cnt
}

tk_local_c() {
  local X=${T_X[0]} N=${T_N[0]} P="mnemonic-toolkit-v${T_X[0]}" T="mnemonic-toolkit-v${T_N[0]}"
  # (b) files are staged; (c) edits stay in the work tree
  git add -- $(git diff --name-only)
  TK_B_FILES=$(git diff --cached --name-only | LC_ALL=C sort)
  touch_file scripts/install.sh
  touch_file docs/manual-gui/pinned-upstream.toml
  TK_G=$(toml_manual_gui_tag docs/manual-gui/pinned-upstream.toml)
  sed -i "s/$(re_escape "$P")/$T/g" scripts/install.sh
  local fsv fsnote
  if fsv=$(fs_value scripts/install.sh mnemonic); then
    if [ -z "$fsv" ]; then
      fs_set scripts/install.sh "$N"
      fsnote="\"\" -> \"$N\""
    else
      fsnote="already \"$fsv\""
    fi
  else
    err "cannot find the 'mnemonic)' entry of first_signed() in scripts/install.sh"
    local_fail
  fi
  tk_examples_c .examples-build/Examples.md "$X" "$N"
  local ik
  ik=$(git diff --numstat -- .examples-build/Examples.md | awk '{print $1+0}')
  toml_set_installer_ahead docs/manual-gui/pinned-upstream.toml "$T"
  # checks
  local first bad
  first=$(grep -oE 'mnemonic-toolkit-v[0-9.]+' scripts/install.sh | sed -n '1p' || true)
  if [ "$first" != "$T" ]; then
    err "toolkit (c): install.sh self-pin = tag failed:"
    printf '   first mnemonic-toolkit-v... in scripts/install.sh is %s, not %s\n' "$first" "$T" >&2
    local_fail
  fi
  bad=$(grep -nE "$(re_escape "$P")([^0-9]|\$)" scripts/install.sh || true)
  if [ -n "$bad" ]; then
    err "toolkit (c): no $P left in install.sh failed:"
    printf '%s\n' "$bad" | indent3
    local_fail
  fi
  bad=$(lines_naming .examples-build/Examples.md "$X")
  if [ -n "$bad" ]; then
    err "toolkit (c): Examples golden names no $X failed:"
    printf '%s\n' "$bad" | indent3
    local_fail
  fi
  if python3 docs/manual-gui/tests/check_cli_pins.py --manual-dir docs/manual-gui --install-sh scripts/install.sh >/dev/null 2>&1; then
    TK_PINS_OK=1
    say "== toolkit (c): check_cli_pins.py OK"
  else
    TK_PINS_OK=0
    say "== toolkit (c): check_cli_pins.py fails on (c); the pins PR owes the GUI manual's {#installer-ahead} section (listed below)"
  fi
  say "== toolkit (c): install.sh self-pin $P -> $T; first_signed mnemonic $fsnote; Examples.md $ik installer line(s); [installer-ahead] toolkit-tag-implied = \"$T\""
  TK_C_FILES=$(git diff --name-only | LC_ALL=C sort)
}

# ------------------------------------------------------------ follow-on (8)
SIB_FETCHED=" "
sib_ok() { git -C "$1" rev-parse --git-dir >/dev/null 2>&1; }
sib_fetch() {
  case "$SIB_FETCHED" in *" $1 "*) return 0 ;; esac
  SIB_FETCHED+="$1 "
  git -C "$1" fetch --quiet origin >/dev/null 2>&1 || true
}
# sib_first_signed_empty <short>: 0 when the sibling toolkit's entry is ""
sib_first_signed_empty() {
  local d=$CR_SIB_ROOT/mnemonic-toolkit v
  sib_ok "$d" || return 1
  sib_fetch "$d"
  v=$(git -C "$d" show origin/master:scripts/install.sh 2>/dev/null | fs_value - "$1") || return 1
  [ -z "$v" ]
}

owed_block() {
  local T=$1 G=$2
  cat <<EOF
   toolkit pins PR (owed before it can merge; holder-facing text, so the PR's author writes it):
     docs/manual-gui/src/80-troubleshooting/82-binary-and-launch.md {#installer-ahead}: name $T as installed ahead of the GUI's $G and say what a GUI user sees differently (precedent 1402d547; create the section as in af986ee4 if it is missing)
     docs/manual-gui/CHANGELOG.md: one line for the declaration
     re-check: python3 docs/manual-gui/tests/check_cli_pins.py --manual-dir docs/manual-gui --install-sh scripts/install.sh
     land it: cut-release.sh --land-pr <PR number>   (from a mnemonic-toolkit checkout)
EOF
}

pathspecs_text() {
  local s out=""
  for s in "${CR_PATHSPECS[@]}"; do
    if [ "$s" = . ]; then out+="${out:+ }."; else out+="${out:+ }'$s'"; fi
  done
  printf '%s' "$out"
}

refs_block() {
  local P=$1 prefix=$2 pat flags name def dir hits n total
  say "   references to $P on each sibling's default branch (history paths excluded; some name an older tier on purpose):"
  if [ "$prefix" = v ]; then
    pat="mnemonic-engrave/releases/download/$P/"; flags="-n -F"
  else
    pat=$P; flags="-n -w -F"
  fi
  local entry
  for entry in "${CR_SIBLINGS[@]}"; do
    name=${entry% *}; def=${entry#* }
    dir=$CR_SIB_ROOT/$name
    if ! sib_ok "$dir"; then
      say "     $name: not found at $dir; check it by hand"
      continue
    fi
    sib_fetch "$dir"
    # shellcheck disable=SC2086
    hits=$(git -C "$dir" grep $flags -e "$pat" "origin/$def" -- "${CR_PATHSPECS[@]}" 2>/dev/null || true)
    if [ -z "$hits" ]; then
      say "     (none)"
      continue
    fi
    total=$(printf '%s\n' "$hits" | wc -l)
    printf '%s\n' "$hits" | sed -n '1,40p' | awk -v name="$name" -v pre="origin/$def:" '
      { s = substr($0, length(pre) + 1); i = index(s, ":"); path = substr(s, 1, i - 1); s = substr(s, i + 1)
        j = index(s, ":"); ln = substr(s, 1, j - 1); txt = substr(s, j + 1)
        print "     " name ":" path ":" ln ": " substr(txt, 1, 160) }'
    if [ "$total" -gt 40 ]; then
      n=$((total - 40))
      say "     ... $n more: git -C $dir grep $flags '$pat' origin/$def -- $(pathspecs_text)"
    fi
  done
}

followon_block() {
  local i T P X N prefix bin stem label short key Pe Xe S=$CR_SIB_ROOT
  say "== FOLLOW-ON: not done by this script; each is a commit or PR in the named repo"
  for i in "${!TAGS[@]}"; do
    T=${TAGS[$i]}; P=${T_PREV[$i]}; X=${T_PREVV[$i]}; N=${T_N[$i]}; prefix=${T_PREFIX[$i]}; bin=${T_BIN[$i]}
    say "-- $T (previous ${P:-none})"
    [ -n "$P" ] || continue
    Pe=$(re_escape "$P"); Xe=$(re_escape "$X")
    stem=""; label=""; short=""; key=""
    case "$prefix" in
      ms-cli-v) stem=ms-; label=ms-cli; short=ms; key=ms-cli-tag-implied ;;
      descriptor-mnemonic-md-cli-v) stem=md-; label=md-cli; short=md; key=md-cli-tag-implied ;;
      mnemonic-toolkit-v) stem=mnemonic-; label=mnemonic-toolkit ;;
      mk-cli-v) short=mk; key=mk-cli-tag-implied ;;
      mnemonic-gui-v) short=mnemonic-gui ;;
    esac
    if [ -n "$stem" ]; then
      say "   engrave demo (its CI's required 'test (rust + go)' fails until this lands):"
      say "     cd $S/mnemonic-engrave && git switch master && git pull --ff-only"
      say "     sed -i 's#\\b$Pe\\b#$T#g; s#\\b$stem$Xe-#$stem$N-#g' demo/sh2/src/index.html demo/sh2/CLI_SPINE.md"
      say "     ./demo/sh2/check-install-links.sh"
      say "     git commit -m 'demo/sh2: install block advertises $label v$N' -- demo/sh2/src/index.html demo/sh2/CLI_SPINE.md"
      say "     scripts/push-via-staging.sh master"
    fi
    if [ -n "$short" ]; then
      say "   toolkit installer pin (a PR in $S/mnemonic-toolkit from origin/master):"
      say "     scripts/install.sh component_info: $P -> $T"
      if [ ! -d "$S/mnemonic-toolkit" ] || ! sib_ok "$S/mnemonic-toolkit"; then
        say "     first_signed: $S/mnemonic-toolkit not found; check 'first_signed $short' by hand"
      elif sib_first_signed_empty "$short"; then
        say "     scripts/install.sh first_signed $short: \"\" -> \"$N\" (this release is signed)"
      fi
      say "     mirrors: git grep -n -F -- '--tag $P' .github/workflows docs/manual/src docs/quickstart/src"
      say "     Examples golden: sed -i -E \"/$Pe|'$bin $Xe'/ s#$Xe#$N#g\" .examples-build/Examples.md"
      if [ "${#T}" -ne "${#P}" ]; then
        say "     Examples golden: $T and $P differ in length; keep the --list GIT_TAG column 37 wide"
      fi
      say "     GUI manual: python3 docs/manual-gui/tests/check_cli_pins.py --manual-dir docs/manual-gui --install-sh scripts/install.sh"
      if [ "$short" = mnemonic-gui ]; then
        say "       if it fails: the GUI tag cannot run ahead; the GUI manual re-pins to $T (its [mnemonic-gui] and [manual-gui] tables)"
      else
        say "       if it fails: [installer-ahead] $key = \"$T\" in pinned-upstream.toml, plus {#installer-ahead} text (holder-facing: the PR's author writes it; precedent 99310923)"
      fi
      say "     gates: sh scripts/install-assets.test.sh; then land it: cut-release.sh --land-pr <PR number>"
    fi
    if [ "$prefix" = mnemonic-toolkit-v ]; then
      owed_block "$T" "$TK_G0"
    fi
    refs_block "$P" "$prefix"
  done
}

# ------------------------------------------------------------- dry-run print
files_join() { tr '\n' ' ' <<<"$1" | sed 's/ $//'; }
count_lines() { if [ -z "$1" ]; then printf 0; else printf '%s\n' "$1" | wc -l | tr -d ' '; fi; }

print_dry_run() {
  local i tag w files k
  if [ "$IS_TK" -eq 1 ]; then
    say "DRY RUN: would commit (b) $(count_lines "$TK_B_FILES") file(s): $(files_join "$TK_B_FILES")"
    say "DRY RUN: would commit (c) $(count_lines "$TK_C_FILES") file(s): $(files_join "$TK_C_FILES")"
    say "DRY RUN: would run push-via-staging.sh $DEFAULT for (b)"
    say "DRY RUN: would tag ${TAGS[0]} on commit (c) and push refs/tags/${TAGS[0]}"
    row_split "${T_ROW[0]}"
    say "DRY RUN: would wait for ${TAGS[0]} workflows: $R_WF"
    say "DRY RUN: would verify ${TAGS[0]}: $(archive_name "${T_N[0]}") in $R_SUMS, every SHA256SUMS* signed, '${T_BIN[0]} ${T_N[0]}'"
    say "DRY RUN: would run sh scripts/install-assets.test.sh on (c), push (c) to refs/heads/release-pins-${T_N[0]} and open a draft PR against $DEFAULT"
    return 0
  fi
  files=$(git diff --name-only HEAD | LC_ALL=C sort)
  k=$(count_lines "$files")
  if [ "$k" -eq 0 ]; then
    say "DRY RUN: nothing to commit; would stage HEAD $(trunc12 "$HEAD_SHA") as it is"
  else
    say "DRY RUN: would commit $k file(s): $(files_join "$files")"
  fi
  say "DRY RUN: would run push-via-staging.sh $DEFAULT"
  for i in "${!TAGS[@]}"; do
    say "DRY RUN: would tag ${TAGS[$i]} on the release commit and push refs/tags/${TAGS[$i]}"
  done
  for i in "${!TAGS[@]}"; do
    row_split "${T_ROW[$i]}"
    w=$R_WF; [ "$w" != "-" ] || w=none
    say "DRY RUN: would wait for ${TAGS[$i]} workflows: $w"
  done
  for i in "${!TAGS[@]}"; do
    row_split "${T_ROW[$i]}"
    if [ "$R_BIN" = "-" ]; then
      say "DRY RUN: would check ${TAGS[$i]} points at the release commit"
    else
      say "DRY RUN: would verify ${TAGS[$i]}: $(archive_name "${T_N[$i]}") in $R_SUMS, every SHA256SUMS* signed, '$R_BIN ${T_N[$i]}'"
    fi
  done
}

# ------------------------------------------------------------ remote phase
commit_message() { # subject files local-checks
  cat <<EOF
$1

Cut by cut-release.sh (mnemonic-engrave $ENGRAVE_12). Versions and the CHANGELOG
heading only: the gate is CI on this exact commit (push-via-staging.sh),
not a local suite.

Files: $2
Local checks: $3
EOF
}

local_checks_text() {
  local s="cargo build --locked --workspace OK; Cargo.lock $LOCK_K version line(s)" c
  for c in ${VERSION_CLAUSES[@]+"${VERSION_CLAUSES[@]}"}; do s+="; $c"; done
  printf '%s' "$s"
}

trailer_args() {
  TRAILER_ARGS=()
  local t
  for t in ${TRAILERS[@]+"${TRAILERS[@]}"}; do TRAILER_ARGS+=(--trailer "$t"); done
}

toolkit_lines_by_hand() {
  local N=${T_N[0]} b12 c12
  b12=$(trunc12 "$B_SHA"); c12=$(trunc12 "$C_SHA")
  cat >&2 <<EOF
cut-release: the toolkit release commit (b) $b12 IS on origin/master; its tag goes on (c) $c12, which is only on local master (do not reset it). If the output above shows a bypass, STOP and report it.
cut-release: otherwise finish by hand: git tag -a mnemonic-toolkit-v$N -m 'mnemonic-toolkit $N' $C_SHA && git push origin refs/tags/mnemonic-toolkit-v$N; when its release, man-pages, install-pin-check and changelog-check runs are completed/success (gh run list --repo bg002h/mnemonic-toolkit --branch mnemonic-toolkit-v$N): cut-release.sh --verify mnemonic-toolkit-v$N, then the pins-PR line below.
EOF
}

print_pins_pr_line() {
  local N=${T_N[0]}
  cat >&2 <<EOF
cut-release: pins PR: git checkout --quiet --detach $C_SHA && sh scripts/install-assets.test.sh && git checkout --quiet master && git push origin $C_SHA:refs/heads/release-pins-$N && gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-$N --draft --title 'pins: installer -> mnemonic-toolkit-v$N' --body-file $GITDIR/cut-release/pins-pr-body.md
EOF
}

lift_freeze() {
  if [ "$FREEZE_ACTIVE" -eq 1 ]; then say "== FREEZE LIFTED"; FREEZE_ACTIVE=0; fi
}

run_staging() {
  local out rc landed tip
  out=$GITDIR/cut-release/pvs.out
  landed=$RELEASE_SHA
  [ "$IS_TK" -eq 0 ] || landed=$B_SHA
  say "== FREEZE $SLUG $DEFAULT: no commits or merges until this script prints FREEZE LIFTED"
  FREEZE_ACTIVE=1
  if [ "$IS_TK" -eq 1 ]; then git checkout --quiet --detach "$B_SHA"; fi
  set +e
  "$CR_PVS" "$DEFAULT" 2>&1 | tee "$out"
  rc=${PIPESTATUS[0]}
  set -e
  if [ "$IS_TK" -eq 1 ]; then git checkout --quiet "$DEFAULT"; fi
  if [ "$rc" -ne 0 ]; then
    lift_freeze
    err "push-via-staging.sh exited $rc; no tag was pushed."
    git fetch --quiet origin || true
    tip=$(git rev-parse "origin/$DEFAULT" 2>/dev/null || true)
    if [ "$tip" = "$landed" ]; then
      if [ "$IS_TK" -eq 1 ]; then
        toolkit_lines_by_hand
        print_pins_pr_line
      else
        err "the release commit IS on origin/$DEFAULT. If the output above shows a bypass, STOP and report it; otherwise re-run the same command: it finds the commit on $DEFAULT, re-stages it and tags."
      fi
    else
      err "origin/$DEFAULT is unchanged. To retry, drop the local commit and run again: git reset --hard origin/$DEFAULT"
    fi
    exit 5
  fi
  if grep -qi 'bypassed rule violations' "$out"; then
    lift_freeze
    err "push-via-staging.sh output contains 'Bypassed rule violations'; not tagging. STOP and report it."
    exit 5
  fi
  if ! grep -q '^== OK: ' "$out"; then
    lift_freeze
    err "push-via-staging.sh exited 0 without its '== OK:' line; not tagging"
    exit 5
  fi
  lift_freeze
}

tk_reset() {
  [ "$IS_TK" -eq 1 ] || return 0
  [ -n "$B_SHA" ] || return 0
  { git checkout --quiet "$DEFAULT" && git reset --quiet --keep "$B_SHA"; } || true
}
finish_lines() {
  err "finish: cut-release.sh --verify $BIN_TAGS; once it shows 0 failed, make the follow-on moves in $GITDIR/cut-release/follow-on.txt"
  if [ "$IS_TK" -eq 1 ] && [ "$PINS_OPENED" -eq 0 ] && [ "$SKIP_PINS" -eq 0 ]; then
    print_pins_pr_line
  fi
}
after_tag_exit() {
  HANDLED=1
  tk_reset
  finish_lines
  exit "$1"
}
on_signal() {
  trap - INT TERM HUP
  HANDLED=1
  err "interrupted after the tag push; the tags are pushed and stay pushed: never delete or move them."
  tk_reset
  finish_lines
  exit 7
}
on_exit() {
  local rc=$?
  trap - EXIT
  if [ "$FREEZE_ACTIVE" -eq 1 ]; then say "== FREEZE LIFTED"; FREEZE_ACTIVE=0; fi
  if [ "$LOCAL_ACTIVE" -eq 1 ]; then restore_touched; LOCAL_ACTIVE=0; fi
  if [ "$POST_TAG" -eq 1 ] && [ "$DONE_FLAG" -eq 0 ] && [ "$HANDLED" -eq 0 ]; then
    HANDLED=1
    err "stopped after the tag push by a failing command (above); the tags are pushed and stay pushed: never delete or move them."
    tk_reset
    finish_lines
    exit 7
  fi
  return "$rc"
}

push_tags() {
  local i t refs=() out x all_ok=1 any=0 line
  for i in "${!TAGS[@]}"; do
    git tag -a "${TAGS[$i]}" -m "${T_CRATE[$i]} ${T_N[$i]}" "$TAG_SHA"
    refs+=("refs/tags/${TAGS[$i]}")
  done
  if out=$(git push --atomic origin "${refs[@]}" 2>&1); then
    POST_TAG=1
  else
    for t in "${TAGS[@]}"; do
      x=$(git ls-remote origin "refs/tags/$t^{}" | awk 'NR==1{print $1}' || true)
      if [ "$x" = "$TAG_SHA" ]; then any=1; else all_ok=0; fi
    done
    if [ "$all_ok" -eq 1 ]; then
      say "== the tag push reported an error, but origin has every tag on $(trunc12 "$TAG_SHA"); continuing"
      POST_TAG=1
    else
      local none=1
      for t in "${TAGS[@]}"; do
        x=$(git ls-remote origin "refs/tags/$t" | awk 'NR==1{print $1}' || true)
        [ -z "$x" ] || none=0
      done
      if [ "$none" -eq 1 ]; then
        for t in "${TAGS[@]}"; do git tag -d "$t" >/dev/null 2>&1 || true; done
        err "pushing the tags failed (atomic; none is on origin):"
        printf '%s\n' "$out" | indent3
        if [ "$IS_TK" -eq 1 ]; then
          toolkit_lines_by_hand
        else
          err "re-run the same command; it finds the release commit on $DEFAULT and tags it"
        fi
        exit 6
      fi
      for t in "${TAGS[@]}"; do
        x=$(git ls-remote origin "refs/tags/$t^{}" | awk 'NR==1{print $1}' || true)
        err "after the failed push origin has $t -> ${x:-absent}"
      done
      err "STOP and report it"
      exit 6
    fi
  fi
  trap on_signal INT TERM HUP
  for t in "${TAGS[@]}"; do
    x=$(git ls-remote origin "refs/tags/$t^{}" | awk 'NR==1{print $1}' || true)
    if [ "$x" != "$TAG_SHA" ]; then
      HANDLED=1
      err "origin's $t peels to ${x:-absent}, not $TAG_SHA; STOP and report it"
      exit 6
    fi
    say "== tag $t -> $(trunc12 "$TAG_SHA") pushed"
  done
}

wait_for_runs() {
  local i tag sha w id n runs status conclusion url
  for i in "${!TAGS[@]}"; do
    tag=${TAGS[$i]}; sha=$TAG_SHA
    row_split "${T_ROW[$i]}"
    [ "$R_WF" != "-" ] || continue
    for w in $R_WF; do
      n=0; id=""
      while :; do
        runs=$(gh run list --repo "$SLUG" --branch "$tag" --event push --limit 50 --json databaseId,name,headSha,status,conclusion,url)
        id=$(jq -r --arg w "$w" --arg s "$sha" '[.[] | select(.name==$w and .headSha==$s)][0].databaseId // empty' <<<"$runs")
        [ -z "$id" ] || break
        n=$((n + 1))
        if [ "$n" -ge 40 ]; then
          err "no '$w' run appeared for $tag within 10 minutes"
          err "the tags are pushed and stay pushed: never delete or move them."
          say "== follow-on pins are NOT listed: the release is not verified"
          after_tag_exit 7
        fi
        sleep "$CR_POLL"
      done
      say "== waiting for $tag: $w run $id"
      n=0
      while :; do
        runs=$(gh run view "$id" --repo "$SLUG" --json status,conclusion,url)
        status=$(jq -r '.status' <<<"$runs")
        [ "$status" != completed ] || break
        n=$((n + 1))
        if [ "$n" -ge 360 ]; then
          url=$(jq -r '.url' <<<"$runs")
          err "run $id ($w, $tag) did not finish within 90 minutes: $url"
          err "the tags are pushed and stay pushed: never delete or move them."
          err "a flake: gh run rerun $id --failed --repo $SLUG, then cut-release.sh --verify $tag. A real defect: STOP; the fix is a new patch release."
          say "== follow-on pins are NOT listed: the release is not verified"
          after_tag_exit 7
        fi
        sleep "$CR_POLL"
      done
      conclusion=$(jq -r '.conclusion' <<<"$runs")
      if [ "$conclusion" != success ]; then
        url=$(jq -r '.url' <<<"$runs")
        err "run $id ($w, $tag) concluded '$conclusion': $url"
        err "the tags are pushed and stay pushed: never delete or move them."
        err "a flake: gh run rerun $id --failed --repo $SLUG, then cut-release.sh --verify $tag. A real defect: STOP; the fix is a new patch release."
        say "== follow-on pins are NOT listed: the release is not verified"
        after_tag_exit 7
      fi
      say "== run $id $w ($tag) success"
    done
  done
  # the runs nobody waits for
  for i in "${!TAGS[@]}"; do
    tag=${TAGS[$i]}
    row_split "${T_ROW[$i]}"
    local waited
    if [ "$R_WF" = "-" ]; then waited='[]'; else waited=$(printf '%s\n' $R_WF | jq -R . | jq -s .); fi
    runs=$(gh run list --repo "$SLUG" --branch "$tag" --event push --limit 50 --json databaseId,name,headSha,status,conclusion,url)
    jq -r --arg s "$TAG_SHA" --argjson waited "$waited" '.[] | select(.headSha==$s) | select(.name as $n | ($waited | index($n)) == null) | "   not waited: \(.name) run \(.databaseId) \(.status)/\(if (.conclusion // "") == "" then "-" else .conclusion end)"' <<<"$runs"
  done
}

push_flow() {
  local i files k subj msgf crate_list="" b12 c12 N X
  mkdir -p "$GITDIR/cut-release"
  trailer_args
  for i in "${!TAGS[@]}"; do
    if [ -n "$crate_list" ]; then crate_list+=" + "; fi
    crate_list+="${T_CRATE[$i]} ${T_N[$i]}"
  done
  if [ "$IS_TK" -eq 1 ]; then
    N=${T_N[0]}; X=${T_X[0]}
    msgf=$GITDIR/cut-release/commit-msg.txt
    subj="release: mnemonic-toolkit $N (installer self-pin follows in the pins commit)"
    commit_message "$subj" "$(files_join "$TK_B_FILES")" "$(local_checks_text)" >"$msgf"
    git commit -q -F "$msgf" "${TRAILER_ARGS[@]}"
    B_SHA=$(git rev-parse HEAD)
    say "== commit $(trunc12 "$B_SHA") $subj"
    git add -- $TK_C_FILES
    subj="pins: installer -> mnemonic-toolkit-v$N"
    local pinchk="OK"
    [ "${TK_PINS_OK:-0}" -eq 1 ] || pinchk="fails, prose owed"
    cat >"$msgf" <<EOF
$subj

Cut by cut-release.sh (mnemonic-engrave $ENGRAVE_12). The tag mnemonic-toolkit-v$N
goes on this commit (install-pin-check). It reaches master by pull request,
after the release verifies; that PR owes the GUI manual's {#installer-ahead}
text.

Files: $(files_join "$TK_C_FILES")
Local checks: install.sh self-pin = tag; no mnemonic-toolkit-v$X left in install.sh; Examples golden names no $X; check_cli_pins.py $pinchk
EOF
    git commit -q -F "$msgf" "${TRAILER_ARGS[@]}"
    C_SHA=$(git rev-parse HEAD)
    say "== commit $(trunc12 "$C_SHA") $subj"
    LOCAL_ACTIVE=0
    RELEASE_SHA=$B_SHA
    TAG_SHA=$C_SHA
    { owed_block "${TAGS[0]}" "$TK_G0"; printf '\nTagged commit: %s. Merge with a merge commit only: squash or rebase would leave the tag off master.\n' "$C_SHA"; } >"$GITDIR/cut-release/pins-pr-body.md"
  else
    files=$(git diff --name-only HEAD | LC_ALL=C sort)
    if [ -n "$files" ]; then
      subj="release: $crate_list"
      msgf=$GITDIR/cut-release/commit-msg.txt
      commit_message "$subj" "$(files_join "$files")" "$(local_checks_text)" >"$msgf"
      git add -- $files
      git commit -q -F "$msgf" "${TRAILER_ARGS[@]}"
      LOCAL_ACTIVE=0
      RELEASE_SHA=$(git rev-parse HEAD)
      say "== commit $(trunc12 "$RELEASE_SHA") $subj"
    else
      LOCAL_ACTIVE=0
      RELEASE_SHA=$(git rev-parse HEAD)
    fi
    TAG_SHA=$RELEASE_SHA
  fi
  printf '%s\n' "$FOLLOWON" >"$GITDIR/cut-release/follow-on.txt"
  run_staging
  push_tags
  # 5. wait
  wait_for_runs
  # 6. verify
  local bad=() tag
  for i in "${!TAGS[@]}"; do
    tag=${TAGS[$i]}
    if [ "${T_BIN[$i]}" = "-" ]; then
      say "   ok    $tag peels to $(trunc12 "$TAG_SHA")"
    else
      verify_tag "$tag"
      if [ "$VF_FAIL" -gt 0 ]; then bad+=("$tag"); fi
    fi
  done
  if [ "${#bad[@]}" -gt 0 ]; then
    for tag in "${bad[@]}"; do
      err "$tag did not verify (above); the tags stay pushed. STOP and report it."
    done
    say "== follow-on pins are NOT listed: the release is not verified"
    after_tag_exit 8
  fi
  if [ "$IS_TK" -eq 1 ]; then
    toolkit_remote_tail
  else
    say "$FOLLOWON"
    DONE_FLAG=1
    say "== DONE: ${TAGS[*]} published and verified"
  fi
}

# 6.3 steps 4-6
toolkit_remote_tail() {
  local N=${T_N[0]} T=${TAGS[0]} out line url num c12 body
  c12=$(trunc12 "$C_SHA")
  body=$GITDIR/cut-release/pins-pr-body.md
  git checkout --quiet --detach "$C_SHA"
  out=$(sh scripts/install-assets.test.sh || true)
  git checkout --quiet "$DEFAULT"
  printf '%s\n' "$out"
  line=$(printf '%s\n' "$out" | awk 'NF{l=$0} END{print l}')
  if [ "$line" != "[install-assets.test] OK" ]; then
    SKIP_PINS=1
    cat >&2 <<EOF
cut-release: scripts/install-assets.test.sh failed on (c) $c12; the pins PR was not opened. The tag is published. Fix forward on top of (c): git push origin $C_SHA:refs/heads/release-pins-$N, commit the fix to that branch, then gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-$N --draft --title 'pins: installer -> mnemonic-toolkit-v$N' --body-file $body (a PR without (c) leaves the tag off master).
EOF
    after_tag_exit 9
  fi
  if ! git push origin "$C_SHA:refs/heads/release-pins-$N" || ! out=$(gh pr create --repo bg002h/mnemonic-toolkit --base master --head "release-pins-$N" --draft --title "pins: installer -> mnemonic-toolkit-v$N" --body-file "$body"); then
    err "could not open the pins PR (above); (c) is at refs/heads/release-pins-$N; open it by hand: gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-$N --draft --title 'pins: installer -> mnemonic-toolkit-v$N' --body-file $body"
    after_tag_exit 9
  fi
  PINS_OPENED=1
  url=$(printf '%s\n' "$out" | awk 'NF{l=$0} END{print l}')
  if ! [[ $url =~ ^https://github\.com/bg002h/mnemonic-toolkit/pull/[0-9]+$ ]]; then
    err "gh pr create exited 0 but printed no pull request URL; find the PR: gh pr list --repo bg002h/mnemonic-toolkit --head release-pins-$N"
    after_tag_exit 9
  fi
  num=${url##*/pull/}
  say "$FOLLOWON"
  say "== toolkit: pins PR $url is open as a draft; it owes the text above. Its author marks it ready, then: cut-release.sh --land-pr $num"
  tk_reset
  DONE_FLAG=1
  say "== DONE: $T published and verified"
}

# --------------------------------------------------------------- verify (5.4)
VF_OK=0; VF_FAIL=0
v_ok() { say "   ok    $*"; VF_OK=$((VF_OK + 1)); }
v_fail() { say "   FAIL  $*"; VF_FAIL=$((VF_FAIL + 1)); }
v_none() { say "   --    $*"; VF_OK=$((VF_OK + 1)); }

# newest published release tag with a prefix older than a version (or any)
release_tags_for_prefix() { # list-json prefix
  jq -r '.[] | select(.isDraft==false and .isPrerelease==false) | .tagName' <<<"$1" |
    grep -E "^$(re_escape "$2")[0-9]+\.[0-9]+\.[0-9]+\$" || true
}
newest_of() { # prefix, tags on stdin -> newest tag
  local p=$1 t
  while IFS= read -r t; do [ -n "$t" ] && printf '%s\n' "${t#"$p"}"; done |
    sort -t. -k1,1n -k2,2n -k3,3n | tail -n 1 | awk -v p="$p" 'NF{print p $0}'
}

verify_tag() {
  local tag=$1 slug prefix N bin archive d rel names list prevtag PV pn want s out rc c comment
  VF_OK=0; VF_FAIL=0
  parse_tag "$tag"; prefix=$PT_PREFIX; N=$PT_N
  row_by_prefix "$prefix"
  slug="bg002h/$R_REPO"; bin=$R_BIN; archive=$(archive_name "$N")
  d="$CR_CACHE/$tag"
  rm -rf "$d"; mkdir -p "$d"
  say "== verify $tag"
  # V1
  if ! rel=$(gh release view "$tag" --repo "$slug" --json isDraft,isPrerelease,assets 2>/dev/null); then
    v_fail "release not found"
    say "== verify $tag: $VF_OK ok, $VF_FAIL failed"
    return 0
  fi
  names=$(jq -r '.assets[].name' <<<"$rel")
  local draft pre
  draft=$(jq -r '.isDraft' <<<"$rel"); pre=$(jq -r '.isPrerelease' <<<"$rel")
  if [ "$draft" = true ]; then v_fail "release is a draft"; fi
  if [ "$pre" = true ]; then v_fail "release is a prerelease"; fi
  if [ "$draft" != true ] && [ "$pre" != true ]; then
    v_ok "release is published (not draft, not prerelease), $(jq '.assets|length' <<<"$rel") assets"
  fi
  # V2
  list=$(gh release list --repo "$slug" --limit 100 --json tagName,isDraft,isPrerelease 2>/dev/null || echo '[]')
  prevtag=$(release_tags_for_prefix "$list" "$prefix" |
    while IFS= read -r c; do [ "$(tcmp "${c#"$prefix"}" "$N")" -lt 0 ] && printf '%s\n' "$c"; done |
    newest_of "$prefix" || true)
  if [ -z "$prevtag" ]; then
    v_none "no earlier $prefix release to compare against"
  else
    PV=${prevtag#"$prefix"}
    if pn=$(gh release view "$prevtag" --repo "$slug" --json assets 2>/dev/null | jq -r '.assets[].name'); then
      local missing=0
      while IFS= read -r s; do
        [ -n "$s" ] || continue
        want=${s//"$PV"/"$N"}
        if ! grep -qxF -- "$want" <<<"$names"; then v_fail "missing vs $prevtag: $want"; missing=1; fi
      done <<<"$pn"
      if [ "$missing" -eq 0 ]; then v_ok "every asset of $prevtag is present (names with $PV -> $N)"; fi
    else
      v_fail "missing vs $prevtag: (its assets could not be read)"
    fi
  fi
  # V3
  local sums=()
  while IFS= read -r s; do
    [ -n "$s" ] || continue
    case "$s" in SHA256SUMS*.minisig) continue ;; SHA256SUMS*) sums+=("$s") ;; esac
  done <<<"$names"
  for s in ${sums[@]+"${sums[@]}"}; do
    if ! grep -qxF -- "$s.minisig" <<<"$names"; then v_fail "no $s.minisig asset (unsigned release)"; fi
  done
  # V4
  gh release download "$tag" --repo "$slug" --dir "$d" --pattern 'SHA256SUMS*' --pattern "$archive" >/dev/null 2>&1 || true
  for s in ${sums[@]+"${sums[@]}"}; do
    [ -f "$d/$s.minisig" ] || continue
    rc=0
    out=$(cd "$d" && minisign -Vm "$s" -x "$s.minisig" -P "$PUBKEY" 2>&1) || rc=$?
    if [ "$rc" -ne 0 ]; then
      v_fail "$s.minisig does not verify:"
      printf '%s\n' "$out" | indent9
    else
      c=$(printf '%s\n' "$out" | grep '^Trusted comment: ' | sed -n '1p' || true)
      comment=${c#Trusted comment: }
      if [[ $comment == *" $tag $s" ]]; then
        v_ok "$s signed: $c"
      else
        v_fail "$s trusted comment is '$comment', not '... $tag $s'"
      fi
    fi
  done
  # V5, V6
  if [ ! -f "$d/$archive" ]; then
    v_fail "$archive is not a release asset"
  else
    local found=() f
    for s in ${sums[@]+"${sums[@]}"}; do
      [ -f "$d/$s" ] || continue
      if [ -n "$(awk -v n="$archive" '$2 == n' "$d/$s")" ]; then found+=("$s"); fi
    done
    if [ "${#found[@]}" -eq 0 ]; then
      v_fail "$archive is not listed in any SHA256SUMS* file"
    elif [ "${#found[@]}" -gt 1 ]; then
      v_fail "$archive is listed in ${#found[@]} SHA256SUMS* files"
    else
      if (cd "$d" && awk -v n="$archive" '$2 == n' "${found[0]}" | sha256sum -c --status - 2>/dev/null); then
        v_ok "$archive matches ${found[0]}"
      else
        v_fail "$archive does not match ${found[0]}"
      fi
    fi
    mkdir -p "$d/x"
    tar -xzf "$d/$archive" -C "$d/x" >/dev/null 2>&1 || true
    local cands k first
    cands=$(find "$d/x" -type f -name "$bin" 2>/dev/null || true)
    k=$(count_lines "$cands")
    if [ "$k" -ne 1 ]; then
      v_fail "$archive has $k files named '$bin'"
    else
      out=$(timeout 30 "$cands" --version 2>&1 || true)
      first=${out%%$'\n'*}
      if [ "$first" = "$bin $N" ]; then
        v_ok "RAN $bin --version -> $bin $N"
      else
        v_fail "$bin --version printed '$first', not '$bin $N'"
      fi
    fi
  fi
  say "== verify $tag: $VF_OK ok, $VF_FAIL failed"
}

verify_main() {
  local tag total=0 failed=0 p list tags=()
  if [ "${#TAGS[@]}" -gt 0 ]; then
    tags=("${TAGS[@]}")
  else
    for p in "${CR_BIN_PREFIXES[@]}"; do
      row_by_prefix "$p"
      list=$(gh release list --repo "bg002h/$R_REPO" --limit 100 --json tagName,isDraft,isPrerelease 2>/dev/null || echo '[]')
      tag=$(release_tags_for_prefix "$list" "$p" | newest_of "$p" || true)
      if [ -n "$tag" ]; then tags+=("$tag"); else tags+=("$p"); fi
    done
  fi
  for tag in "${tags[@]}"; do
    if ! parse_tag "$tag" || ! row_by_prefix "$PT_PREFIX"; then
      say "   FAIL  no published release with prefix ${tag} in the release table"
      total=$((total + 1)); failed=$((failed + 1))
      continue
    fi
    verify_tag "$tag"
    total=$((total + 1))
    if [ "$VF_FAIL" -gt 0 ]; then failed=$((failed + 1)); fi
  done
  say "== verify: $total release(s), $failed failed"
  [ "$failed" -eq 0 ] || exit 8
}

# -------------------------------------------------------------- land-pr (7)
land_pr() {
  local n=$PR_NUMBER view state draft base head oid mstate url cross V tagoid polls cnt checks pend badn
  view=$(gh pr view "$n" --repo "$SLUG" --json state,isDraft,baseRefName,headRefName,headRefOid,mergeStateStatus,url,isCrossRepository)
  state=$(jq -r '.state' <<<"$view"); draft=$(jq -r '.isDraft' <<<"$view")
  base=$(jq -r '.baseRefName' <<<"$view"); head=$(jq -r '.headRefName' <<<"$view")
  oid=$(jq -r '.headRefOid' <<<"$view"); url=$(jq -r '.url' <<<"$view")
  cross=$(jq -r '.isCrossRepository' <<<"$view")
  if [ "$state" != OPEN ]; then refuse 3 "PR #$n is $state, not OPEN"; fi
  if [ "$draft" = true ]; then refuse 3 "PR #$n is a draft; its author marks it ready (gh pr ready $n) once the owed edits are in"; fi
  if [ "$base" != "$DEFAULT" ]; then refuse 3 "PR #$n targets '$base', not $DEFAULT"; fi
  if [[ $head =~ ^release-pins-(.+)$ ]]; then
    V=${BASH_REMATCH[1]}
    git fetch --quiet --tags origin "$head" >/dev/null 2>&1 || true
    if ! git merge-base --is-ancestor "mnemonic-toolkit-v$V^{}" "$oid" 2>/dev/null; then
      tagoid=$(git rev-parse -q --verify "mnemonic-toolkit-v$V^{}" 2>/dev/null || true)
      refuse 9 "PR #$n does not contain mnemonic-toolkit-v$V (${tagoid:0:12}); merging it would leave the tag off $DEFAULT"
    fi
  fi
  polls=0
  while :; do
    checks=$(gh pr checks "$n" --repo "$SLUG" --json name,bucket,link 2>/dev/null || true)
    cnt=$(jq 'length' <<<"${checks:-[]}" 2>/dev/null || echo 0)
    [ "$cnt" -gt 0 ] && break
    polls=$((polls + 1))
    if [ "$polls" -ge 40 ]; then refuse 9 "no checks appeared on PR #$n within 10 minutes"; fi
    sleep "$CR_POLL"
  done
  polls=0
  while :; do
    pend=$(jq '[.[] | select(.bucket=="pending")] | length' <<<"$checks")
    [ "$pend" -eq 0 ] && break
    polls=$((polls + 1))
    if [ "$polls" -ge 240 ]; then refuse 9 "PR #$n checks did not finish within 60 minutes"; fi
    sleep "$CR_POLL"
    checks=$(gh pr checks "$n" --repo "$SLUG" --json name,bucket,link 2>/dev/null || true)
  done
  badn=$(jq '[.[] | select(.bucket != "pass" and .bucket != "skipping")] | length' <<<"$checks")
  if [ "$badn" -gt 0 ]; then
    jq -r --arg n "$n" --arg q "'" '.[] | select(.bucket != "pass" and .bucket != "skipping") | "cut-release: PR #\($n) check \($q)\(.name)\($q) is \(.bucket): \(.link)"' <<<"$checks" >&2
    err "fix it on the PR branch, then run cut-release.sh --land-pr $n again"
    exit 9
  fi
  mstate=$(jq -r '.mergeStateStatus' <<<"$view")
  polls=0
  while :; do
    mstate=$(gh pr view "$n" --repo "$SLUG" --json mergeStateStatus | jq -r '.mergeStateStatus')
    [ "$mstate" = UNKNOWN ] || break
    polls=$((polls + 1))
    [ "$polls" -lt 10 ] || break
    sleep "$CR_POLL"
  done
  if [ "$mstate" != CLEAN ]; then refuse 9 "PR #$n merge state is $mstate, not CLEAN"; fi
  local mout
  if ! mout=$(gh pr merge "$n" --repo "$SLUG" --merge --match-head-commit "$oid" 2>&1); then
    err "gh pr merge failed:"
    printf '%s\n' "$mout" | indent3
    exit 9
  fi
  git fetch --quiet origin || true
  if ! git merge-base --is-ancestor "$oid" "origin/$DEFAULT" 2>/dev/null; then
    refuse 9 "$(trunc12 "$oid") is not on origin/$DEFAULT after the merge; STOP and report it"
  fi
  local remote_oid reason
  remote_oid=$(git ls-remote origin "refs/heads/$head" | awk 'NR==1{print $1}' || true)
  if [ "$cross" != true ] && [ "$remote_oid" = "$oid" ]; then
    local dout
    if ! dout=$(git push origin --delete "$head" 2>&1); then
      err "merged, but deleting $head failed:"
      printf '%s\n' "$dout" | indent3
      exit 9
    fi
    say "== merged PR #$n ($url) as a merge commit; $head deleted"
  else
    if [ "$cross" = true ]; then
      reason="it is a fork's branch"
    else
      reason="origin's $head is $(trunc12 "${remote_oid:-absent}"), not the merged $(trunc12 "$oid")"
    fi
    say "== merged PR #$n ($url) as a merge commit; $head left in place ($reason)"
  fi
}

# -------------------------------------------------------------------- main
main() {
  CR_DATE=${CUT_RELEASE_DATE:-$(date +%F)}
  parse_args "$@"
  if ! [[ $CR_DATE =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
    refuse 2 "CUT_RELEASE_DATE must be YYYY-MM-DD"
  fi
  need_tools
  case "$MODE" in
    verify)
      check_key
      check_gh_auth
      verify_main
      ;;
    land-pr)
      preflight_common_repo
      check_gh_auth
      land_pr
      ;;
    dry-run|push)
      preflight_common_repo
      preflight_dry_push
      ENGRAVE_TOP=${ENGRAVE_TOP:-$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null || true)}
      ENGRAVE_12=unknown
      if [ -n "$ENGRAVE_TOP" ]; then ENGRAVE_12=$(git -C "$ENGRAVE_TOP" rev-parse --short=12 HEAD 2>/dev/null || echo unknown); fi
      CR_SIB_ROOT=${CUT_RELEASE_SIBLINGS:-$(dirname "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")")}
      local i
      CL_STATE=()
      for i in "${!TAGS[@]}"; do changelog_check "$i"; done
      changelog_c7
      BIN_TAGS=""
      for i in "${!TAGS[@]}"; do
        if [ "${T_BIN[$i]}" != "-" ]; then BIN_TAGS+="${BIN_TAGS:+ }${TAGS[$i]}"; fi
      done
      TK_G0=""
      if [ "$IS_TK" -eq 1 ] && [ -f docs/manual-gui/pinned-upstream.toml ]; then
        TK_G0=$(toml_manual_gui_tag docs/manual-gui/pinned-upstream.toml)
      fi
      FOLLOWON=$(followon_block)
      trap on_exit EXIT
      local label='DRY RUN'
      if [ "$MODE" = push ]; then label=PUSH; fi
      say "== cut-release $label: $REPO ($DEFAULT) -> ${TAGS[*]}"
      say "== script: $SCRIPT_PATH (mnemonic-engrave $ENGRAVE_12)"
      if [ "$MODE" = push ]; then
        say "== PUSH: these tags cannot be taken back once pushed: ${TAGS[*]}"
      fi
      local_phase
      if [ "$MODE" = dry-run ]; then
        print_dry_run
        restore_touched
        LOCAL_ACTIVE=0
        say "$FOLLOWON"
        say "== DRY RUN OK: nothing was pushed; tracked files are as they were"
      else
        push_flow
      fi
      ;;
  esac
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then main "$@"; fi
