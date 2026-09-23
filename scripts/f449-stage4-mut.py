#!/usr/bin/env python3
# mut.py <file> <old> <new> <pkg> <test-regex>: apply, prove it applied, build, run, report, revert.
import subprocess, sys, os
f, old, new, pkg, rx = sys.argv[1:6]
os.chdir(os.environ.get("MUTDIR", "/scratch/code/shibboleth/.tmp/s4-plan/fork"))
env = dict(os.environ, PATH="/scratch/code/shibboleth/.toolchain/go/bin:" + os.environ["PATH"])
s = open(f).read()
n = s.count(old)
if n != 1:
    print(f"NOT-APPLIED ({n} matches): {old[:60]!r}"); sys.exit(2)
open(f, "w").write(s.replace(old, new))
try:
    b = subprocess.run(["go", "vet", pkg], capture_output=True, text=True, env=env)
    if b.returncode != 0 and "ArtifactDir" not in b.stderr:
        print("DID-NOT-COMPILE:", b.stderr[:300]); sys.exit(3)
    r = subprocess.run(["go", "test", "-count=1", "-run", rx, pkg], capture_output=True, text=True, env=env)
    if "build failed" in r.stdout or "setup failed" in r.stdout:
        print("DID-NOT-COMPILE:", r.stdout[:300]); sys.exit(3)
    fails = sorted({l.strip().split()[2] for l in r.stdout.splitlines() if l.strip().startswith("--- FAIL")})
    print(("CAUGHT " if r.returncode else "SURVIVED ") + " ".join(fails))
finally:
    subprocess.run(["git", "checkout", "--", f])
