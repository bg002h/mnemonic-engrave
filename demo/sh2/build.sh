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
# it is already newer than the Go sources.
if [ ! -f "$EMU/emu.wasm" ] || [ -n "$(find "$EMU" -name '*.go' -newer "$EMU/emu.wasm" -print -quit)" ]; then
  echo "== building emu.wasm"
  command -v go >/dev/null || { echo "go not on PATH; needed to build emu.wasm" >&2; exit 1; }
  (cd "$EMU" && ./build.sh)
else
  echo "== emu.wasm is current ($(wc -c < "$EMU/emu.wasm") bytes)"
fi

rm -rf dist && mkdir -p dist/emu
cp src/index.html src/mission.html src/app.css src/missions.js dist/
cp "$EMU/index.html" "$EMU/emu.wasm" "$EMU/wasm_exec.js" dist/emu/

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
