#!/usr/bin/env python3
"""F-695 reproduction: every fixed `producer | grep -q` site, BEFORE vs AFTER.

For each site the harness first proves it is testing the real text: every line
of the BEFORE fragment must occur in the file at the pinned origin SHA, and
every line of the AFTER fragment in the file at branch `f695-pipefail`
(`git show`, in the main clones under ROOT). It then evaluates the condition
in bash, under the file's own `set` options, with the producer stubbed:

  big      1 MiB of output with the match on LINE 1 (reproduces the SIGPIPE)
  absent   1 MiB of output, no match
  empty    no output
  prodfail (command producers only) prints the match, then exits 1

Each scenario runs REPS times; the cell is how many runs took the TRUE branch.
Pass criteria: AFTER is REPS/REPS on big, identical to BEFORE on the other
scenarios (behaviour preserved), and BEFORE is < REPS on big (the defect
reproduces). No command here talks to a device: picotool, lsusb, ms, md and
the release binary are shell stubs that `cat` a fixture.
"""
import os, subprocess, sys, tempfile, textwrap

ROOT = "/scratch/code/shibboleth"
REPS = 10
BEFORE = {  # origin default-branch SHAs the sweep started from
    "mnemonic-engrave": "e5c0a52f", "descriptor-mnemonic": "1bea51ec",
    "seedhammer": "80b12c5",
}
AFTER = "f695-pipefail"
PVS_REPOS = {"mnemonic-engrave": "e5c0a52f", "descriptor-mnemonic": "1bea51ec",
             "mnemonic-secret": "cfbcfdd", "mnemonic-toolkit": "af5cc1a5",
             "seedhammer": "80b12c5"}

# (id, repo, file, set-options, producer kind, var, before, after, match, filler)
#   kind "var": the fixture is loaded into shell variable `var`
#   kind "cmd": `var` is the stubbed command name (on PATH, or a path var)
CASES = [
 ("engrave release.yml:93 docs-only (-v)", "mnemonic-engrave", ".github/workflows/release.yml",
  "set -eo pipefail  # HYPOTHETICAL: this step runs bash -e (no pipefail) today",
  "var", "CHANGED",
  """echo "$CHANGED" | grep -qvE '^(design/|[^/]+\\.md$)'""",
  """grep -qvE '^(design/|[^/]+\\.md$)' <<<"$CHANGED\"""",
  "src/lib.rs", "design/notes.md"),
 ("engrave demo/sh2/rehearse.sh:29", "mnemonic-engrave", "demo/sh2/rehearse.sh",
  "set -uo pipefail", "cmd", "ms",
  """ms split --help 2>&1 | grep -q -- '--in'""",
  """{ ms_split_help="$(ms split --help 2>&1)" && grep -q -- '--in' <<<"$ms_split_help"; }""",
  "      --in <FILE>", "filler help text line"),
 ("engrave pico2-bootkey-rehearsal.sh:180/208/373 WARNING traps", "mnemonic-engrave",
  "scripts/pico2-bootkey-rehearsal.sh", "set -euo pipefail", "var", "out",
  """printf '%s' "$out" | grep -qi 'WARNING'""",
  """grep -qi 'WARNING' <<<"$out\"""",
  "WARNING: redundant rows disagree", "field BOOT_FLAGS1 = 0x3"),
 ("engrave pico2-bootkey-rehearsal.sh:1105 verified", "mnemonic-engrave",
  "scripts/pico2-bootkey-rehearsal.sh", "set -euo pipefail", "cmd", "picotool",
  """picotool info -a "$WORKDIR/blinky-mykey.signed.uf2" 2>/dev/null | grep -qi 'signature: *verified'""",
  """{ IMGINFO="$(picotool info -a "$WORKDIR/blinky-mykey.signed.uf2" 2>/dev/null)" \\
      && grep -qi 'signature: *verified' <<<"$IMGINFO"; }""",
  " signature:     verified", " metadata block 1"),
 ("engrave push-master.sh:162 bypass", "mnemonic-engrave", "scripts/push-master.sh",
  "set -uo pipefail", "var", "OUT",
  """printf '%s' "$OUT" | grep -qi 'bypass'""",
  """grep -qi 'bypass' <<<"$OUT\"""",
  "remote: Bypassed rule violations for refs/heads/master:", "remote: counting objects"),
 ("push-via-staging.sh:213 bypass (5 copies)", "mnemonic-engrave", "scripts/push-via-staging.sh",
  "set -euo pipefail", "var", "OUT",
  """echo "$OUT" | grep -qi "bypassed rule violations\"""",
  """grep -qi "bypassed rule violations" <<<"$OUT\"""",
  "remote: Bypassed rule violations for refs/heads/master:", "remote: counting objects"),
 ("engrave release-workflows/emit.py:32 (emitted mk smoke)", "mnemonic-engrave",
  "scripts/release-workflows/emit.py", "set -eo pipefail  # the emitted step is shell: bash",
  "cmd", "$BIN",
  """"$BIN" encode --help | grep -q -- '--policy-id-stub'""",
  # The emitted text is two statements; under `bash -e` a failing capture ends
  # the step (a failure, like the BEFORE's `|| { ...; exit 1; }`). Inside an
  # `if` condition -e is suspended, so the harness models that with `&&`.
  """{ ENCODE_HELP="$("$BIN" encode --help)" &&
          grep -q -- '--policy-id-stub' <<<"$ENCODE_HELP"; }""",
  "      --policy-id-stub <HEX>", "filler help text line"),
 ("engrave sh2-flash:288 verified", "mnemonic-engrave", "scripts/sh2-flash",
  "set -euo pipefail", "cmd", "picotool",
  """devshell picotool info -a "$SIGNED" 2>/dev/null | grep -q 'signature:.*verified'""",
  """{ SIGINFO="$(devshell picotool info -a "$SIGNED" 2>/dev/null)" \\
          && grep -q 'signature:.*verified' <<<"$SIGINFO"; }""",
  " signature:     verified", " metadata block 1"),
 ("engrave sh2-flash:305 lsusb", "mnemonic-engrave", "scripts/sh2-flash",
  "set -euo pipefail", "cmd", "lsusb",
  """lsusb 2>/dev/null | grep -qi "$BOOTSEL_ID\"""",
  """{ USB_LIST="$(lsusb 2>/dev/null)" && grep -qi "$BOOTSEL_ID" <<<"$USB_LIST"; }""",
  "Bus 001 Device 042: ID 2e8a:000f Raspberry Pi RP2350 Boot", "Bus 001 Device 001: ID 1d6b:0002 Linux Foundation hub"),
 ("engrave sign-firmware.sh:101 HASH_ERR", "mnemonic-engrave", "scripts/sign-firmware.sh",
  "set -euo pipefail", "var", "HASH_ERR",
  """printf '%s' "$HASH_ERR" | grep -qiE 'missing SIGNATURE section|missing HASH_DEF item'""",
  """grep -qiE 'missing SIGNATURE section|missing HASH_DEF item' <<<"$HASH_ERR\"""",
  "error: missing SIGNATURE section", "note: parsing block"),
 ("engrave sign-firmware.sh:180 verified", "mnemonic-engrave", "scripts/sign-firmware.sh",
  "set -euo pipefail", "var", "INFO",
  """printf '%s' "$INFO" | grep -qi 'signature: *verified'""",
  """grep -qi 'signature: *verified' <<<"$INFO\"""",
  " signature:     verified", " metadata block 1"),
 ("engrave scripts/test/run-e2e.sh:66", "mnemonic-engrave", "scripts/test/run-e2e.sh",
  "set -uo pipefail; want='wrote [0-9]+ bytes'", "var", "out",
  """printf '%s' "$out" | grep -qiE "$want\"""",
  """grep -qiE "$want" <<<"$out\"""",
  "wrote 1234 bytes", "progress line"),
 ("dm scripts/gen-compose-golden.sh:34", "descriptor-mnemonic", "scripts/gen-compose-golden.sh",
  "set -euo pipefail", "cmd", "$MD",
  """"$MD" compose --help 2>&1 | grep -q -- '--unspendable'""",
  """compose_help="$("$MD" compose --help 2>&1)" && grep -q -- '--unspendable' <<<"$compose_help\"""",
  "      --unspendable <KEY>", "filler help text line"),
 ("seedhammer scripts/oracle-live.sh:104", "seedhammer", "scripts/oracle-live.sh",
  "set -uo pipefail", "var", "mint_out",
  """printf '%s\\n' "$mint_out" | grep -q '^=== RUN   TestAssembledMd1MatchesThePrimaryByteForByte$'""",
  """grep -q '^=== RUN   TestAssembledMd1MatchesThePrimaryByteForByte$' <<<"$mint_out\"""",
  "=== RUN   TestAssembledMd1MatchesThePrimaryByteForByte", "    oracle_test.go:12: log line"),
]

def show(repo, rev, path):
    return subprocess.run(["git", "-C", f"{ROOT}/{repo}", "show", f"{rev}:{path}"],
                          check=True, capture_output=True, text=True).stdout

def verbatim(fragment, text):
    """Every line of the fragment (minus a trailing continuation) occurs in text."""
    for ln in fragment.splitlines():
        # a `{ ...; }` wrapper is the harness's grouping, not the file's text
        ln = ln.strip().rstrip("\\").strip().removeprefix("{ ").removesuffix("; }").removesuffix(" &&")
        if ln and ln not in text:
            return ln
    return None

def fixture(d, name, first, filler, size):
    p = os.path.join(d, name)
    with open(p, "w") as f:
        if first is not None:
            f.write(first + "\n")
        n = 0
        while n < size:
            f.write(filler + "\n"); n += len(filler) + 1
    return p

def run(opts, kind, var, cond, fx, d, prodfail=False):
    stubdir = os.path.join(d, "bin"); os.makedirs(stubdir, exist_ok=True)
    stub = os.path.join(stubdir, "producer")
    with open(stub, "w") as f:
        f.write(f"#!/bin/sh\ncat '{fx}'\n" + ("exit 1\n" if prodfail else ""))
    os.chmod(stub, 0o755)
    for name in ("ms", "picotool", "lsusb"):
        dst = os.path.join(stubdir, name)
        if os.path.lexists(dst): os.unlink(dst)
        os.symlink(stub, dst)
    setup = [opts, f'export PATH="{stubdir}:$PATH"',
             'devshell() { "$@"; }',  # sh2-flash's non-DRY devshell, minus nix
             f'BIN="{stub}"; MD="{stub}"; WORKDIR=/nonexistent; SIGNED=/nonexistent',
             'BOOTSEL_ID="2e8a:000f"']
    if kind == "var":
        setup.append(f'{var}="$(cat \'{fx}\')"')
    body = "\n".join(setup) + textwrap.dedent(f"""
        t=0
        for _ in $(seq {REPS}); do
          if {cond}
          then t=$((t+1)); fi
        done
        echo "$t"
        """)
    r = subprocess.run(["bash", "-c", body], capture_output=True, text=True)
    if r.returncode != 0:
        return f"ERR({r.returncode}:{r.stderr.strip()[:60]})"
    return int(r.stdout.strip())

def main():
    ok = True
    # the five push-via-staging copies must stay byte-identical
    blobs = {r: subprocess.run(["git", "-C", f"{ROOT}/{r}", "rev-parse", f"{AFTER}:scripts/push-via-staging.sh"],
                               check=True, capture_output=True, text=True).stdout.strip() for r in PVS_REPOS}
    same = len(set(blobs.values())) == 1
    print(f"push-via-staging.sh blob on {AFTER} in {len(blobs)} repos: "
          f"{'IDENTICAL ' + next(iter(blobs.values()))[:12] if same else blobs}")
    ok &= same
    with tempfile.TemporaryDirectory(dir=os.environ.get("F695_TMP")) as d:
        print(f"\nREPS={REPS}; cells = runs that took the TRUE branch (before -> after)\n")
        print(f"{'site':58} {'big':>9} {'absent':>9} {'empty':>9} {'prodfail':>9}  verdict")
        for (cid, repo, path, opts, kind, var, before, after, match, filler) in CASES:
            miss_b = verbatim(before, show(repo, BEFORE[repo], path))
            miss_a = verbatim(after, show(repo, AFTER, path))
            if miss_b or miss_a:
                print(f"{cid:58} FRAGMENT NOT IN FILE: {miss_b or miss_a!r}"); ok = False; continue
            fx = {"big": fixture(d, "big", match, filler, 1 << 20),
                  "absent": fixture(d, "absent", None, filler, 1 << 20),
                  "empty": fixture(d, "empty", None, "", 0)}
            open(fx["empty"], "w").close()
            cells, res = [], {}
            for sc in ("big", "absent", "empty", "prodfail"):
                if sc == "prodfail" and kind != "cmd":
                    cells.append("-"); continue
                src = fx["big"] if sc == "prodfail" else fx[sc]
                b = run(opts, kind, var, before, src, d, prodfail=(sc == "prodfail"))
                a = run(opts, kind, var, after, src, d, prodfail=(sc == "prodfail"))
                res[sc] = (b, a); cells.append(f"{b}->{a}")
            good = (res["big"][1] == REPS and isinstance(res["big"][0], int) and res["big"][0] < REPS
                    and all(res[s][0] == res[s][1] for s in res if s != "big"))
            ok &= good
            print(f"{cid:58} " + " ".join(f"{c:>9}" for c in cells) + f"  {'PASS' if good else 'FAIL'}")
    print("\nOVERALL:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
