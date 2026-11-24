#!/usr/bin/env bash
# Regression test for bh-me-I14: demo/sh2/build.sh must republish the
# emulator when only a //go:embed asset of the fork changed.
#
# A fixture fork (go.mod, cmd/emu with a main that embeds
# sysw_test_payload.bin, the emulator's build.sh, index.html, wasm_exec.js)
# is assembled twice: first with the payload "fixture-alpha", then with only
# the payload changed to "fixture-bravo". The second dist/emu/emu.wasm must
# carry fixture-bravo. Needs go on PATH; builds under a fresh mktemp -d.
# Exit 0 = PASS, 1 = the defect, 2 = the fixture did not build.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
T="$(mktemp -d)"
trap 'chmod -R u+w "$T" 2>/dev/null; rm -r "$T"' EXIT
export GOCACHE="$T/gocache" GOPATH="$T/gopath" GOFLAGS=-mod=mod GOTOOLCHAIN=local
mkdir -p "$T/demo" "$T/fork/cmd/emu"
cp -r "$ROOT/demo/sh2" "$T/demo/sh2"
rm -rf "$T/demo/sh2/dist"
F="$T/fork"
printf 'module fixture\n\ngo 1.21\n' > "$F/go.mod"
cat > "$F/cmd/emu/main.go" <<'EOF'
package main

import _ "embed"

//go:embed sysw_test_payload.bin
var payload string

func main() { println(payload) }
EOF
cat > "$F/cmd/emu/build.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
GOOS=js GOARCH=wasm go build -trimpath -o emu.wasm .
EOF
chmod +x "$F/cmd/emu/build.sh"
echo '<html></html>' > "$F/cmd/emu/index.html"
echo '// stub' > "$F/cmd/emu/wasm_exec.js"
printf 'fixture-alpha' > "$F/cmd/emu/sysw_test_payload.bin"
(cd "$T/demo/sh2" && FORK="$F" bash build.sh > "$T/b1.log" 2>&1) || { cat "$T/b1.log"; echo "FIXTURE: first assembly failed"; exit 2; }
grep -q fixture-alpha "$T/demo/sh2/dist/emu/emu.wasm" || { echo "FIXTURE: first wasm lacks fixture-alpha"; exit 2; }
# The payload must be strictly newer than emu.wasm for any mtime test.
sleep 1.1
printf 'fixture-bravo' > "$F/cmd/emu/sysw_test_payload.bin"
(cd "$T/demo/sh2" && FORK="$F" bash build.sh > "$T/b2.log" 2>&1) || { cat "$T/b2.log"; echo "FIXTURE: second assembly failed"; exit 2; }
grep -h '^== ' "$T/b2.log" | head -1
if grep -q fixture-bravo "$T/demo/sh2/dist/emu/emu.wasm"; then
  echo "PASS: second dist wasm contains fixture-bravo"
  exit 0
fi
echo "FAIL: second dist wasm lacks fixture-bravo (fixture-alpha lines: $(grep -c fixture-alpha "$T/demo/sh2/dist/emu/emu.wasm"))"
exit 1
