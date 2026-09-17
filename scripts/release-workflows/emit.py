import sys, pathlib
sys.path.insert(0, '.')
from gen import gen

ALL5 = ["linux-x86_64","linux-aarch64","macos-x86_64","macos-aarch64","windows-x86_64"]
PORTABLE3 = ["macos-x86_64","macos-aarch64","windows-x86_64"]
MACOS_ONLY = ["macos-x86_64","macos-aarch64"]

SMOKE = {
 # A REAL round trip, through PRIVATE CHANNELS -- `ms` refuses hex entropy and
 # ms1 strings on argv (its own guard), so the obvious one-liner does not work
 # and was caught locally before it reached CI.
 "ms": '''          printf '00000000000000000000000000000000' > e.hex
          MS1="$("$BIN" encode --hex - --group-size 0 < e.hex | grep '^ms1')"
          printf '%s\\n' "$MS1" > card.ms1
          DEC="$("$BIN" decode --in card.ms1)"
          case "$DEC" in *abandon*) ;; *) echo "::error::round trip failed"; exit 1 ;; esac
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
 "ms": "# THIS REPO ALREADY SHIPS LINUX. The musl workflow publishes static aarch64 +\n"
       "# x86_64 builds, a BETTER Linux artifact than a glibc-linked one, so this\n"
       "# workflow adds macOS only.\n#\n"
       "# NO WINDOWS TARGET, AND THAT IS DELIBERATE. `ms` pins secret-bearing heap\n"
       "# pages with POSIX mlock(2) so they cannot be swapped to disk\n"
       "# (crates/ms-cli/src/mlock.rs, SPEC_secret_memory_hygiene_v0_9_B.md). The\n"
       "# module has no cfg(unix) gating and does not compile for MSVC: `cannot find\n"
       "# function mlock in crate libc`, measured on this matrix.\n#\n"
       "# The fix is NOT to cfg it out. That would ship a Windows binary whose seed\n"
       "# material is swappable while every other platform pins it -- a silent\n"
       "# downgrade in exactly the code where it matters most. Windows has\n"
       "# VirtualLock/VirtualUnlock and a real port is possible, but mlock.rs is an\n"
       "# INLINE COPY of mnemonic-toolkit\'s with a G6 CI invariant asserting the two\n"
       "# are byte-equal, so the change lands in both repos or CI goes red. That is a\n"
       "# spec cycle, not a build flag. Tracked in design/FOLLOWUPS.md.\n",
 "mk": "# THIS REPO ALREADY SHIPS LINUX. `musl-binaries.yml` publishes static aarch64 +\n"
       "# x86_64 musl builds, which are a BETTER Linux artifact than a glibc-linked one.\n"
       "# So this workflow adds only what was missing: macOS and Windows. Every platform\n"
       "# mnemonic-engrave supports is then covered.\n",
 "mt": "# This repo shipped NO binaries at all, so this workflow covers all five targets.\n",
}

MD_SMOKE = '''          # A real round trip, not just --version.
          MD1="$("$BIN" encode 'wpkh(@0/<0;1>/*)' --group-size 0 | grep '^md1')"
          DEC="$("$BIN" decode "$MD1")"
          case "$DEC" in *'wpkh(@0'*) ;; *) echo "::error::round trip failed"; exit 1 ;; esac
          echo "round trip ok: $MD1"
'''

# mt's history-purge tests drive a REAL interactive zsh and fish on a pty, so
# they need those shells present. Its own ci.yml installs them; a release gate
# that did not would fail six tests for a missing dependency and look like a
# broken build.
TEST_SETUP = {
 "mt": """      - name: Install the shells the history-purge tests drive
        run: |
          sudo apt-get update
          sudo apt-get install -y zsh fish
          /usr/bin/zsh --version
          /usr/bin/fish --version
""",
}

REPOS = {
 "descriptor-mnemonic":  dict(bin_="md", pkg="md-cli", branch="main",   names=ALL5),
 "mnemonic-secret":      dict(bin_="ms", pkg="ms-cli", branch="master", names=MACOS_ONLY),
 "mnemonic-key":         dict(bin_="mk", pkg="mk-cli", branch="main",   names=PORTABLE3),
 "mnemonic-transaction": dict(bin_="mt", pkg="mt-cli", branch="main",   names=ALL5),
}

for repo, kw in REPOS.items():
    b = kw["bin_"]
    text = gen(why_subset=WHY[b], smoke=SMOKE.get(b, MD_SMOKE), test_setup=TEST_SETUP.get(b, ''), **kw)
    p = pathlib.Path(f"/scratch/code/shibboleth/{repo}/.github/workflows/release.yml")
    p.write_text(text)
    import yaml
    d = yaml.safe_load(text)
    print(f"{repo:<22} {b}  targets={[m['name'] for m in d['jobs']['build']['strategy']['matrix']['include']]}")
