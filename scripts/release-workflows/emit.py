import sys, pathlib
sys.path.insert(0, '.')
from gen import gen

ALL5 = ["linux-x86_64","linux-aarch64","macos-x86_64","macos-aarch64","windows-x86_64"]
PORTABLE3 = ["macos-x86_64","macos-aarch64","windows-x86_64"]
MACOS_ONLY = ["macos-x86_64","macos-aarch64"]

SMOKE = {
 "ms": '''          # A REAL acceptance pass, through PRIVATE CHANNELS -- ms refuses hex
          # entropy and ms1 strings on argv, so the obvious one-liners do not work.
          printf '00000000000000000000000000000000' > e.hex
          MS1="$("$BIN" encode --hex - --group-size 0 < e.hex | grep '^ms1')"
          printf '%s\\n' "$MS1" > card.ms1
          DEC="$("$BIN" decode --in card.ms1)"
          case "$DEC" in *abandon*) ;; *) echo "::error::round trip failed"; exit 1 ;; esac

          # Shamir: 3-of-5 rebuilds, 2 does not. The threshold, on this platform.
          printf 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about\\n' > seed.txt
          "$BIN" split --in seed.txt -k 3 -n 5 --group-size 0 > shares.txt
          n=$(grep -c '^ms1' shares.txt); [ "$n" = "5" ] || { echo "::error::split gave $n shares"; exit 1; }
          grep -o '^ms1[a-z0-9]*' shares.txt | sed -n '1p;3p;5p' > three.txt
          OUT="$("$BIN" combine --in three.txt)"
          case "$OUT" in *abandon*) ;; *) echo "::error::3 shares did not rebuild"; exit 1 ;; esac
          grep -o '^ms1[a-z0-9]*' shares.txt | sed -n '1p;3p' > two.txt
          if "$BIN" combine --in two.txt >/dev/null 2>&1; then
            echo "::error::2 shares rebuilt the secret -- the threshold does not hold here"; exit 1
          fi
          echo "acceptance ok on this platform: round trip + 3-of-5 + 2 refused"
''',
 "mk": '''          "$BIN" --help >/dev/null
          "$BIN" encode --help | grep -q -- '--policy-id-stub' || {
            echo "::error::encode's flag surface is not what this platform built"; exit 1; }
          echo "binary runs; flag surface intact"
''',
 "mt": '''          "$BIN" --help >/dev/null
          "$BIN" encode --help >/dev/null
          echo "binary runs; subcommands resolve"
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
       "# workflow adds macOS and Windows.\n#\n"
       "# THE WINDOWS BINARY DOES NOT LOCK SECRET MEMORY, AND SAYS SO AT RUNTIME.\n"
       "# `ms` pins secret-bearing heap pages with POSIX mlock(2) so they cannot be\n"
       "# swapped to disk. Windows has no mlock; the non-POSIX arm reports a distinct\n"
       "# ERRNO_UNSUPPORTED, which rides the SAME failure counting and report_at_exit\n"
       "# a real EPERM rides, so every run prints that the regions were left unpinned\n"
       "# and that the OS may write them to the page file. Operator decision, taken\n"
       "# knowingly: a warned build beats no build, and it is never silent.\n"
       "# A real VirtualLock port is tracked in design/FOLLOWUPS.md.\n",
 "mk": "# THIS REPO ALREADY SHIPS LINUX. `musl-binaries.yml` publishes static aarch64 +\n"
       "# x86_64 musl builds, which are a BETTER Linux artifact than a glibc-linked one.\n"
       "# So this workflow adds only what was missing: macOS and Windows. Every platform\n"
       "# mnemonic-engrave supports is then covered.\n",
 "mt": "# This repo shipped NO binaries at all, so this workflow covers all five targets.\n",
}

MD_SMOKE = '''          # A REAL acceptance pass, on this platform, using the commands the
          # demo hands to people. Deterministic on every target: same policy,
          # same md1, same Template-ID, same corrections.
          MD1="$("$BIN" encode 'wpkh(@0/<0;1>/*)' --group-size 0 | grep '^md1')"
          DEC="$("$BIN" decode "$MD1")"
          case "$DEC" in *'wpkh(@0'*) ;; *) echo "::error::round trip failed: $DEC"; exit 1 ;; esac

          # compose -> the zen-hodl policy the demo builds on the device
          ZEN="$("$BIN" compose --wrapper tr --path '1of1,older=32768' 2>/dev/null)"
          case "$ZEN" in *'older(32768)'*) ;; *) echo "::error::compose failed: $ZEN"; exit 1 ;; esac

          # BCH repair: four damaged characters, named and corrected
          BADMD1=md1yqfdsqsjuqqpr5e55uzqqgqqqrqqvf4d7h59r2
          GOODMD1=md1yqfdsssjuqqcr5e55uqqqgqqq6qqvf4d7h59r2
          if "$BIN" decode "$BADMD1" >/dev/null 2>&1; then
            echo "::error::the damaged string decoded; BCH detection is broken here"; exit 1
          fi
          REP="$("$BIN" repair "$BADMD1" || true)"
          case "$REP" in *"$GOODMD1"*) ;; *) echo "::error::repair did not restore: $REP"; exit 1 ;; esac
          case "$REP" in *'4 corrections'*) ;; *) echo "::error::correction count changed: $REP"; exit 1 ;; esac

          # cross-implementation identity: must match what the DEVICE shows
          ID="$("$BIN" inspect "$GOODMD1" | grep 'wallet-descriptor-template-id:' | awk '{print $2}')"
          [ "$ID" = "73c33a5dea17b45376a6246995df0cbb" ] || {
            echo "::error::Template-ID is $ID on this platform, not 73c33a5d... -- the"
            echo "::error::identity hash is not platform-independent, which breaks the"
            echo "::error::whole point of a template id"; exit 1; }
          echo "acceptance ok on this platform: $MD1 / $ID"
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

# Which OSes can run each suite. md and mk use only PermissionsExt/mode(0o..),
# which macOS has. ms hardcodes /usr/bin/script and mt hardcodes
# /usr/bin/{zsh,fish,script,timeout}; those paths differ on macOS, so their
# suites stay on Linux rather than being loosened to travel.
TEST_OS = {"md": "[ubuntu-latest, macos-latest]", "mk": "[ubuntu-latest, macos-latest]"}

REPOS = {
 "descriptor-mnemonic":  dict(bin_="md", pkg="md-cli", branch="main",   names=ALL5),
 "mnemonic-secret":      dict(bin_="ms", pkg="ms-cli", branch="master", names=PORTABLE3),
 "mnemonic-key":         dict(bin_="mk", pkg="mk-cli", branch="main",   names=PORTABLE3),
 "mnemonic-transaction": dict(bin_="mt", pkg="mt-cli", branch="main",   names=ALL5),
}

for repo, kw in REPOS.items():
    b = kw["bin_"]
    text = gen(why_subset=WHY[b], smoke=SMOKE.get(b, MD_SMOKE), test_setup=TEST_SETUP.get(b, ''), test_os=TEST_OS.get(b, '[ubuntu-latest]'), **kw)
    p = pathlib.Path(f"/scratch/code/shibboleth/{repo}/.github/workflows/release.yml")
    p.write_text(text)
    import yaml
    d = yaml.safe_load(text)
    print(f"{repo:<22} {b}  targets={[m['name'] for m in d['jobs']['build']['strategy']['matrix']['include']]}")
