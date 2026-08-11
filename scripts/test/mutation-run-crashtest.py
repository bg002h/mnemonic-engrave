#!/usr/bin/env python3
"""F-101 -- prove mutation-run.py's crash safety, under real signals.

The claim being tested is that a KILLED run cannot leave a mutant applied in the
worktree. That claim is worthless asserted: `try/finally` looks like it covers
it and does not, because SIGKILL cannot be caught and SIGTERM's default action
terminates without unwinding.

So this sends the actual signals to an actual child process:

  SIGKILL  nothing can catch it, so the mutant IS left behind -- and the
           on-disk sentinel is what a later run uses to recover it.
  SIGTERM  catchable, so the handler restores before exiting, and the exit
           status stays honest (-15 rather than a masked 0).

Run: python3 scripts/test/mutation-run-crashtest.py
Exits non-zero on failure. No arguments, no fixtures, no network.
"""

import importlib.util, os, pathlib, shutil, signal, subprocess, sys, tempfile

SRC = "/scratch/code/shibboleth/mnemonic-engrave/scripts/mutation-run.py"

def load(sentinel):
    spec = importlib.util.spec_from_file_location("mr", SRC)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    m.SENTINEL = pathlib.Path(sentinel)
    return m

CHILD = """
import sys, os, signal, pathlib
sys.argv=['x']
import importlib.util
spec = importlib.util.spec_from_file_location("mr", %r)
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
m.SENTINEL = pathlib.Path(%r)
src, bak, how = pathlib.Path(%r), pathlib.Path(%r), %r
if how == 'term':
    m._install_signal_handlers()
m._mark_inflight(src, bak)
src.write_text("MUTANT armed := true\\n")          # apply the mutation
os.kill(os.getpid(), signal.SIGKILL if how=='kill' else signal.SIGTERM)
"""

def scenario(how):
    d = pathlib.Path(tempfile.mkdtemp())
    src, bak, sent = d/"run_flow.go", d/"run_flow.go.bak", d/"sentinel.json"
    src.write_text("ORIGINAL armed := ctx.wipe.armed()\n")
    shutil.copy2(src, bak)
    p = subprocess.run([sys.executable, "-c", CHILD % (SRC, str(sent), str(src), str(bak), how)],
                       capture_output=True, text=True)
    killed_clean = src.read_text().startswith("ORIGINAL")
    print(f"[{how}] child rc={p.returncode} sentinel={'yes' if sent.exists() else 'no'} "
          f"file={'ORIGINAL' if killed_clean else 'MUTANT'}")
    if how == 'kill':
        assert not killed_clean, "SIGKILL should have left the mutant (nothing can catch it)"
        assert sent.exists(), "sentinel must survive SIGKILL -- that is its whole job"
        m = load(sent); m.recover_sentinel()
        rec = src.read_text().startswith("ORIGINAL")
        print(f"[{how}] after recover_sentinel(): file={'ORIGINAL' if rec else 'MUTANT'} "
              f"sentinel={'yes' if sent.exists() else 'no'}")
        assert rec, "recovery must restore the original"
        assert not sent.exists(), "sentinel must be cleared after a successful recovery"
    else:
        assert killed_clean, "SIGTERM handler should have restored before exiting"
        assert not sent.exists(), "handler must clear the sentinel after restoring"
    print(f"[{how}] PASS")

scenario('kill')
scenario('term')
