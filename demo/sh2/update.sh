#!/usr/bin/env bash
# Update the already-installed SH2 demo on Quantoshi.
#
# deploy.sh INSTALLS (creates the directory, backs up nginx, prints a block for
# a human to paste). This script UPDATES an install that already exists and is
# already serving. The two jobs have different risks, so they are different
# scripts:
#
#   - It NEVER touches nginx. The block is already in place, the config is
#     shared with 19 other directives for other sites, and editing a live web
#     server's config to ship a static asset is how an unrelated site goes down.
#     (deploy.sh backs nginx up on every run; 13 such backups have accumulated.)
#
#   - It NEVER runs `rsync --delete` against the live directory. build.sh
#     content-hashes asset filenames, so names change between releases and a
#     plain rsync would accumulate dead files forever -- but --delete pointed at
#     the wrong path is the one mistake that could hurt Quantoshi.
#
#     Instead: rsync into a STAGING directory, then swap it in by rename. Two
#     renames, sub-millisecond, and nginx resolves `alias /opt/quantoshi/sh2/`
#     per request so no reload is needed. The previous release is kept, so
#     rollback is a rename rather than a rebuild.
#
#   - It VERIFIES over HTTPS after the swap and ROLLS BACK BY ITSELF if the
#     verify fails. The MIME type on emu.wasm is the failure that silently
#     breaks the page while still answering 200, so it is checked explicitly.
#
# Usage:
#   ./update.sh                 build, deploy, verify (rollback on failure)
#   ./update.sh --dry-run       print every remote command, change nothing
#   ./update.sh --no-build      deploy the existing dist/ as-is
#   ./update.sh --rollback      restore the previous release
#   ./update.sh --list          show the releases currently on the host
set -euo pipefail
cd "$(dirname "$0")"

HOST="${SH2_HOST:-root@quantoshi.xyz}"
KEY="${SH2_KEY:-$HOME/.ssh/id_ed25519}"
REMOTE_DIR=/opt/quantoshi/sh2
BASE_URL=https://quantoshi.xyz/SH2/
KEEP=3                       # previous releases to retain
DRY=0; BUILD=1; ROLLBACK=0; LIST=0

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run|--dry) DRY=1 ;;
    --no-build)      BUILD=0 ;;
    --rollback)      ROLLBACK=1 ;;
    --list)          LIST=1 ;;
    --host)          HOST="$2"; shift ;;
    -h|--help)       sed -n '2,32p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

SSH=(ssh -o BatchMode=yes -o ConnectTimeout=10 -i "$KEY" "$HOST")
rrun() {
  if [ "$DRY" = 1 ]; then printf '   would run (remote): %s\n' "$*"; return 0; fi
  "${SSH[@]}" "$*"
}
say() { printf '== %s\n' "$*"; }

# ---------------------------------------------------------------- list ------
if [ "$LIST" = 1 ]; then
  say "releases on $HOST"
  "${SSH[@]}" "ls -1dt ${REMOTE_DIR} ${REMOTE_DIR}.prev-* 2>/dev/null | \
     while read -r d; do printf '%-46s %s  %s files\n' \"\$d\" \
       \"\$(date -r \"\$d\" -u +%Y-%m-%dT%H:%M:%SZ)\" \
       \"\$(find \"\$d\" -type f | wc -l)\"; done"
  exit 0
fi

# ------------------------------------------------------------ rollback ------
if [ "$ROLLBACK" = 1 ]; then
  say "ROLLBACK to the most recent previous release"
  prev=$("${SSH[@]}" "ls -1dt ${REMOTE_DIR}.prev-* 2>/dev/null | head -1" || true)
  [ -n "$prev" ] || { echo "no ${REMOTE_DIR}.prev-* on the host; nothing to roll back to" >&2; exit 1; }
  echo "-- restoring $prev"
  stamp=$(date -u +%Y%m%dT%H%M%SZ)
  rrun "set -e; mv '$REMOTE_DIR' '${REMOTE_DIR}.failed-${stamp}'; mv '$prev' '$REMOTE_DIR'"
  code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 20 "$BASE_URL" || echo 000)
  echo "-- $BASE_URL -> $code"
  [ "$code" = 200 ] && say "rolled back. nginx was never touched." || \
    { echo "rollback did not restore a 200; investigate on the host" >&2; exit 1; }
  exit 0
fi

# --------------------------------------------------------------- build ------
if [ "$BUILD" = 1 ]; then
  say "building"
  ./build.sh
fi

# ------------------------------------------------------- local preflight ----
say "local preflight"
[ -d dist ] || { echo "no dist/ -- run ./build.sh (or drop --no-build)" >&2; exit 1; }
for f in index.html emu/emu.wasm emu/wasm_exec.js; do
  [ -s "dist/$f" ] || { echo "dist/$f missing or empty -- refusing to deploy" >&2; exit 1; }
done
LOCAL_WASM=$(wc -c < dist/emu/emu.wasm)
LOCAL_FILES=$(find dist -type f | wc -l)
LOCAL_SUM=$(find dist -type f -exec sha256sum {} + | sort -k2 | sha256sum | cut -c1-16)
echo "   $LOCAL_FILES files, emu.wasm $LOCAL_WASM bytes, tree $LOCAL_SUM"

# ------------------------------------------------------ remote preflight ----
say "remote preflight"
if [ "$DRY" = 0 ]; then
  "${SSH[@]}" "test -f '$REMOTE_DIR/emu/emu.wasm'" \
    || { echo "REFUSING: $REMOTE_DIR is not an existing SH2 deploy. Use deploy.sh to install." >&2; exit 1; }
  avail=$("${SSH[@]}" "df -Pk /opt | awk 'NR==2{print \$4}'")
  need=$(( (LOCAL_WASM / 1024) * 3 ))
  [ "$avail" -gt "$need" ] || { echo "REFUSING: only ${avail}K free on /opt, need ~${need}K for staging" >&2; exit 1; }
  echo "   target is an SH2 deploy; ${avail}K free on /opt"
fi

# --------------------------------------------------------------- stage ------
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
STAGING="${REMOTE_DIR}.staging-${STAMP}"
say "staging to $STAGING"
rrun "rm -rf '$STAGING' && mkdir -p '$STAGING'"
if [ "$DRY" = 1 ]; then
  printf '   would run: rsync -a -e "ssh -i %s" dist/ %s:%s/\n' "$KEY" "$HOST" "$STAGING"
else
  rsync -a -e "ssh -o BatchMode=yes -i $KEY" dist/ "$HOST:$STAGING/"
fi

say "verifying the staged copy before it goes live"
if [ "$DRY" = 0 ]; then
  rw=$("${SSH[@]}" "wc -c < '$STAGING/emu/emu.wasm'")
  rf=$("${SSH[@]}" "find '$STAGING' -type f | wc -l")
  [ "$rw" = "$LOCAL_WASM" ] || { echo "staged emu.wasm is $rw bytes, local is $LOCAL_WASM -- aborting, live site untouched" >&2; "${SSH[@]}" "rm -rf '$STAGING'"; exit 1; }
  [ "$rf" = "$LOCAL_FILES" ] || { echo "staged file count $rf != local $LOCAL_FILES -- aborting, live site untouched" >&2; "${SSH[@]}" "rm -rf '$STAGING'"; exit 1; }
  echo "   staged copy matches: $rf files, emu.wasm $rw bytes"
fi

# ---------------------------------------------------------------- swap ------
PREV="${REMOTE_DIR}.prev-${STAMP}"
say "swapping in (two renames; nginx is NOT touched and needs no reload)"
rrun "set -e; mv '$REMOTE_DIR' '$PREV'; mv '$STAGING' '$REMOTE_DIR'"

# --------------------------------------------------------------- verify -----
if [ "$DRY" = 1 ]; then say "dry run complete -- nothing was changed"; exit 0; fi

say "verifying the live site"
fail=""
code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 20 "$BASE_URL" || echo 000)
echo "   $BASE_URL -> $code"
[ "$code" = 200 ] || fail="page returned $code"

ctype=$(curl -sI --max-time 30 "${BASE_URL}emu/emu.wasm" | tr -d '\r' | awk -F': ' 'tolower($1)=="content-type"{print $2}')
echo "   emu.wasm content-type -> ${ctype:-<none>}"
# The MIME type is the failure that silently breaks the page while still 200.
[ "$ctype" = "application/wasm" ] || fail="${fail:+$fail; }emu.wasm content-type is '${ctype:-<none>}', expected application/wasm"

wlen=$(curl -sI --max-time 30 "${BASE_URL}emu/emu.wasm" | tr -d '\r' | awk -F': ' 'tolower($1)=="content-length"{print $2}')
echo "   emu.wasm content-length -> ${wlen:-<none>} (local $LOCAL_WASM)"
[ "$wlen" = "$LOCAL_WASM" ] || fail="${fail:+$fail; }served emu.wasm is ${wlen:-<none>} bytes, deployed $LOCAL_WASM"

if [ -n "$fail" ]; then
  echo "!! VERIFY FAILED: $fail" >&2
  echo "!! rolling back automatically" >&2
  "${SSH[@]}" "set -e; mv '$REMOTE_DIR' '${REMOTE_DIR}.failed-${STAMP}'; mv '$PREV' '$REMOTE_DIR'"
  code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 20 "$BASE_URL" || echo 000)
  echo "!! rolled back; $BASE_URL -> $code. The bad release is at ${REMOTE_DIR}.failed-${STAMP}" >&2
  exit 1
fi

# ---------------------------------------------------------------- prune -----
say "pruning old releases (keeping $KEEP)"
"${SSH[@]}" "ls -1dt ${REMOTE_DIR}.prev-* 2>/dev/null | tail -n +$((KEEP+1)) | xargs -r rm -rf; \
             ls -1d ${REMOTE_DIR}.prev-* 2>/dev/null | wc -l | xargs printf '   %s previous release(s) retained\n'"

say "UPDATED. $BASE_URL is serving $LOCAL_FILES files, tree $LOCAL_SUM"
echo "   roll back with: $0 --rollback"
