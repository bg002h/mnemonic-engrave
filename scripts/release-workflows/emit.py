import sys, pathlib
sys.path.insert(0, '.')
from gen import gen

ALL5 = ["linux-x86_64","linux-aarch64","macos-x86_64","macos-aarch64","windows-x86_64"]
PORTABLE3 = ["macos-x86_64","macos-aarch64","windows-x86_64"]

SMOKE = {
 # A REAL round trip, through PRIVATE CHANNELS -- `ms` refuses hex entropy and
 # ms1 strings on argv (its own guard), so the obvious one-liner does not work
 # and was caught locally before it reached CI.
 "ms": '''          printf '00000000000000000000000000000000' > e.hex
          MS1="$("$BIN" encode --hex - --group-size 0 < e.hex | grep '^ms1')"
          printf '%s\\n' "$MS1" > card.ms1
          "$BIN" decode --in card.ms1 | grep -q abandon || { echo "::error::round trip failed"; exit 1; }
          echo "round trip ok: $MS1"
''',
 # --help only. A real round trip needs a valid xpub/origin-path pair (mk checks
 # the xpub's depth and child against the path) and mk encodes to 2 chunks, so a
 # fixture belongs in the repo's own suite -- which the `test` job runs. This
 # step exists to prove the CROSS-BUILT ARTIFACT EXECUTES on this platform.
 "mk": '''          "$BIN" --help >/dev/null
          echo "binary runs"
''',
 # --help only, same reasoning: a transaction fixture belongs in the test job.
 "mt": '''          "$BIN" --help >/dev/null
          echo "binary runs"
''',
}

WHY = {
 "md": "# This repo shipped NO binaries at all -- only man pages -- so this workflow\n"
       "# covers all five targets.\n#\n"
       "# BINARIES RATHER THAN crates.io, deliberately: this workspace pins\n"
       "# rust-miniscript to a git rev for PR #953 (Display flattens a non-caterpillar\n"
       "# taptree into one Bitcoin Core rejects), a [patch.crates-io] NEVER carries into\n"
       "# a published crate, and a git dependency fails `cargo publish` outright. A\n"
       "# binary built HERE has the patch applied. See design/FOLLOWUPS.md,\n"
       "# `md-codec-unpublishable-while-patched-to-miniscript-master`.\n",
 "ms": "# THIS REPO ALREADY SHIPS LINUX. `man-release.yml`/the musl workflow publish\n"
       "# static aarch64 + x86_64 musl builds, which are a BETTER Linux artifact than a\n"
       "# glibc-linked one. So this workflow adds only what was missing: macOS and\n"
       "# Windows. Every platform mnemonic-engrave supports is then covered.\n",
 "mk": "# THIS REPO ALREADY SHIPS LINUX. `musl-binaries.yml` publishes static aarch64 +\n"
       "# x86_64 musl builds, which are a BETTER Linux artifact than a glibc-linked one.\n"
       "# So this workflow adds only what was missing: macOS and Windows. Every platform\n"
       "# mnemonic-engrave supports is then covered.\n",
 "mt": "# This repo shipped NO binaries at all, so this workflow covers all five targets.\n",
}

MD_SMOKE = '''          # A real round trip, not just --version.
          MD1="$("$BIN" encode 'wpkh(@0/<0;1>/*)' --group-size 0 | grep '^md1')"
          "$BIN" decode "$MD1" | grep -q 'wpkh(@0' || { echo "::error::round trip failed"; exit 1; }
          echo "round trip ok: $MD1"
'''

REPOS = {
 "descriptor-mnemonic":  dict(bin_="md", pkg="md-cli", branch="main",   names=ALL5),
 "mnemonic-secret":      dict(bin_="ms", pkg="ms-cli", branch="master", names=PORTABLE3),
 "mnemonic-key":         dict(bin_="mk", pkg="mk-cli", branch="main",   names=PORTABLE3),
 "mnemonic-transaction": dict(bin_="mt", pkg="mt-cli", branch="main",   names=ALL5),
}

for repo, kw in REPOS.items():
    b = kw["bin_"]
    text = gen(why_subset=WHY[b], smoke=SMOKE.get(b, MD_SMOKE), **kw)
    p = pathlib.Path(f"/scratch/code/shibboleth/{repo}/.github/workflows/release.yml")
    p.write_text(text)
    import yaml
    d = yaml.safe_load(text)
    print(f"{repo:<22} {b}  targets={[m['name'] for m in d['jobs']['build']['strategy']['matrix']['include']]}")
