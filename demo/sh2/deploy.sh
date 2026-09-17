#!/usr/bin/env bash
# Deploy (or roll back) the SH2 demo on the Quantoshi host.
#
# WRITTEN TO BE UNDOABLE, because the honest answer to "can we undo this?" is
# only yes if the deploy was built that way.
#
#   ./deploy.sh --host user@quantoshi         push dist/ and print the rollback
#   ./deploy.sh --host user@quantoshi --dry   show every command, change nothing
#   ./deploy.sh --host user@quantoshi --rollback
#
# WHAT IT TOUCHES, and nothing else:
#   /opt/quantoshi/sh2/                 a NEW directory. Rollback = rm -rf.
#   nginx config                        ONE location block, backed up first.
#
# THE ONE THING THAT COULD HURT QUANTOSHI is `rsync --delete` pointed at the
# wrong directory: it would remove everything there that is not in dist/. So
# this script NEVER passes --delete, and refuses a target that is not either
# empty or already an SH2 deploy. A stale leftover file is a trivial problem; a
# wiped app directory is not.
set -euo pipefail

HOST=""; DRY=0; ROLLBACK=0
REMOTE_DIR=/opt/quantoshi/sh2
NGINX_CONF=/etc/nginx/sites-available/quantoshi.conf
while [ $# -gt 0 ]; do
  case "$1" in
    --host) HOST="$2"; shift 2 ;;
    --dry) DRY=1; shift ;;
    --rollback) ROLLBACK=1; shift ;;
    --dir) REMOTE_DIR="$2"; shift 2 ;;
    --conf) NGINX_CONF="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[ -n "$HOST" ] || { echo "need --host user@host" >&2; exit 2; }

run() {
  if [ "$DRY" = "1" ]; then printf '   would run: %s\n' "$*";
  else "$@"; fi
}
rrun() {  # run on the remote
  if [ "$DRY" = "1" ]; then printf '   would run (remote): %s\n' "$*";
  else ssh "$HOST" "$*"; fi
}

if [ "$ROLLBACK" = "1" ]; then
  echo "== ROLLBACK"
  echo "-- removing the deploy directory (nothing else lives there)"
  rrun "sudo rm -rf '$REMOTE_DIR'"
  echo "-- restoring the most recent nginx config backup"
  rrun "ls -1t ${NGINX_CONF}.bak-* 2>/dev/null | head -1"
  rrun "latest=\$(ls -1t ${NGINX_CONF}.bak-* 2>/dev/null | head -1); \
        [ -n \"\$latest\" ] && sudo cp \"\$latest\" '$NGINX_CONF' && echo restored \"\$latest\" || echo 'no backup found -- remove the /SH2/ block by hand'"
  echo "-- validating and reloading"
  rrun "sudo nginx -t && sudo systemctl reload nginx"
  echo "== rolled back. Quantoshi itself was never modified."
  exit 0
fi

[ -d dist ] || { echo "no dist/ -- run ./build.sh first" >&2; exit 1; }
echo "== deploying $(find dist -type f | wc -l) files ($(du -sh dist | cut -f1)) to $HOST:$REMOTE_DIR"

echo "-- refusing a target that is not empty or not already an SH2 deploy"
rrun "test ! -e '$REMOTE_DIR' || test -f '$REMOTE_DIR/emu/emu.wasm' || \
      { echo 'REFUSING: $REMOTE_DIR exists and is not an SH2 deploy'; exit 1; }"

echo "-- creating the directory"
rrun "sudo mkdir -p '$REMOTE_DIR' && sudo chown \$(id -un):\$(id -gn) '$REMOTE_DIR'"

echo "-- copying (NO --delete, deliberately)"
if [ "$DRY" = "1" ]; then
  printf '   would run: rsync -a dist/ %s:%s/\n' "$HOST" "$REMOTE_DIR"
else
  rsync -a dist/ "$HOST:$REMOTE_DIR/"
fi

echo "-- backing up the nginx config BEFORE touching it"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
rrun "sudo cp '$NGINX_CONF' '${NGINX_CONF}.bak-${STAMP}' && echo 'backup: ${NGINX_CONF}.bak-${STAMP}'"

cat <<EOF

-- THE NGINX BLOCK IS NOT APPLIED AUTOMATICALLY.
   Editing a live web server's config from a script is where an unrelated site
   goes down. Paste the block from nginx-SH2.conf inside the existing
   \`server { server_name quantoshi.xyz ... }\`, then:

       sudo nginx -t && sudo systemctl reload nginx

   \`nginx -t\` validates BEFORE anything takes effect, and \`reload\` keeps the
   old config running if the new one is bad. Never \`restart\` here.

-- VERIFY (the MIME type is the one that silently breaks the page):
       curl -sI https://quantoshi.xyz/SH2/emu/emu.wasm | grep -i content-type
       # expect: content-type: application/wasm
       curl -s -o /dev/null -w '%{http_code}\\n' https://quantoshi.xyz/SH2/

-- UNDO, if anything is wrong:
       $0 --host $HOST --rollback
   or by hand:
       sudo rm -rf $REMOTE_DIR
       sudo cp ${NGINX_CONF}.bak-${STAMP} $NGINX_CONF
       sudo nginx -t && sudo systemctl reload nginx
EOF
