#!/usr/bin/env bash
# Assemble the demo bundle into ./dist.
#
# dist/ is what gets served -- locally with `python3 -m http.server`, or from
# nginx at /SH2. It holds MY pages plus the fork's emulator page copied in
# VERBATIM under emu/, so the emulator stays byte-identical to what the fork
# ships and cannot drift into a demo-only fork of itself.
set -euo pipefail
cd "$(dirname "$0")"

FORK="${FORK:-../../../seedhammer}"
EMU="$FORK/cmd/emu"

[ -d "$EMU" ] || { echo "no emulator at $EMU (set FORK=/path/to/seedhammer)" >&2; exit 1; }

# emu.wasm is gitignored in the fork, so it may not exist yet. Build it unless
# it is already newer than EVERY Go source in the fork, plus go.mod/go.sum. The
# emulator links gui/, md/ and the rest; checking only cmd/emu once shipped a
# stale wasm after a gui/ copy change (2026-09-23).
if [ ! -f "$EMU/emu.wasm" ] || [ -n "$(find "$FORK" \( -name '*.go' -o -name go.mod -o -name go.sum \) -newer "$EMU/emu.wasm" -print -quit)" ]; then
  echo "== building emu.wasm"
  command -v go >/dev/null || { echo "go not on PATH; needed to build emu.wasm" >&2; exit 1; }
  (cd "$EMU" && ./build.sh)
else
  echo "== emu.wasm is current ($(wc -c < "$EMU/emu.wasm") bytes)"
fi

rm -rf dist && mkdir -p dist/emu
cp src/index.html src/mission.html src/app.css src/missions.js dist/
cp "$EMU/index.html" "$EMU/emu.wasm" "$EMU/wasm_exec.js" dist/emu/

# ---- cache-bust the assets whose CONTENT changes between deploys -----------
#
# A browser that fetched one of these while it was being served with the wrong
# MIME type keeps the bad copy until its max-age expires, and the page stays
# broken for that visitor no matter what the server now says. Measured live:
# the same URL returned `application/javascript` under `cache: "no-store"` and
# `text/html` from cache, in the same browser, at the same moment.
#
# Waiting out an expiry is not an answer when people are scanning a QR code, so
# the URL changes whenever the bytes do: a content hash makes a stale entry
# unreachable rather than merely stale. The wasm is deliberately NOT stamped --
# it is 11 MB, cached hard on purpose, and its name already changes per release.
STAMP="$( { cat dist/missions.js dist/app.css; } | sha256sum | cut -c1-12 )"
for f in dist/index.html dist/mission.html; do
  sed -i "s|\./missions\.js|./missions.js?v=${STAMP}|g; s|"app\.css"|"app.css?v=${STAMP}"|g" "$f"
done
echo "== asset stamp: ${STAMP}"

# A favicon, only because its absence is the one console error the emulator
# page produces and it reads as a fault when someone opens devtools.
printf '' > dist/favicon.ico

echo "== dist/"
find dist -type f | sort | while read -r f; do printf '   %8s  %s\n' "$(wc -c < "$f")" "${f#dist/}"; done
cat <<'EOF'

Serve it:
  python3 -m http.server 8391 --directory dist
  open http://127.0.0.1:8391/

Deploy (only when you mean to publish):
  rsync -a --delete dist/ <host>:/opt/quantoshi/sh2/
  # plus the location block in nginx-SH2.conf
EOF
