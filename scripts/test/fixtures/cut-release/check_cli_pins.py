#!/usr/bin/env python3
"""Fixture stub for docs/manual-gui/tests/check_cli_pins.py.

OK when [manual-gui] toolkit-tag-implied equals install.sh's first
mnemonic-toolkit-v pin, or when [installer-ahead] equals it and some file under
<manual-dir>/src holds {#installer-ahead}.
"""
import os
import re
import sys

args = sys.argv[1:]
manual = args[args.index("--manual-dir") + 1]
install = args[args.index("--install-sh") + 1]
pin = re.search(r"mnemonic-toolkit-v[0-9.]+", open(install).read()).group(0)
sections = {}
cur = None
for line in open(os.path.join(manual, "pinned-upstream.toml")):
    line = line.rstrip("\n")
    m = re.match(r"^\[(.+)\]\s*$", line)
    if m:
        cur = m.group(1)
        sections[cur] = {}
        continue
    m = re.match(r'^([A-Za-z0-9_-]+)\s*=\s*"(.*)"\s*$', line)
    if m and cur:
        sections[cur][m.group(1)] = m.group(2)
gui = sections.get("manual-gui", {}).get("toolkit-tag-implied", "")
ahead = sections.get("installer-ahead", {}).get("toolkit-tag-implied", "")
prose = False
for root, _dirs, files in os.walk(os.path.join(manual, "src")):
    for f in files:
        if "{#installer-ahead}" in open(os.path.join(root, f)).read():
            prose = True
if gui == pin or (ahead == pin and prose):
    print("OK: stub")
    sys.exit(0)
print("  pinned-upstream.toml [manual-gui] toolkit-tag-implied = %s, but install.sh pins %s" % (gui, pin))
sys.exit(1)
