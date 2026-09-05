# Decision: the `me` 0.8.1 release and the flash rule while the operator sleeps

Fable decision agent, standing in for the operator ("Fable may answer for me", continuity
`c655de6`). Brief: `design/agent-briefs/decision-me-0.8.1-release-and-flash-rule-brief.md`.
Read-only; nothing committed, nothing flashed, nothing rebooted, no `.jsonl` read.
Local clock at writing: `2026-09-04T22:09:21-07:00` (= 2026-09-05 05:09 UTC; the repo's
session label is 2026-09-05). Engrave master moved five times while this ran — from the
brief's `308a905` to `0c76c63` at writing — every move a records/briefs commit.

## Decisions

- **Q1 — RELEASE v0.8.1 NOW.** Do not hold for H2 or H3. The tree is already merged
  (`8c83e4e`, `--no-ff`) and GREEN; the release commit is the CHANGELOG header line only;
  tag the release commit's SHA after the staging ritual; a sonnet push agent runs it.
- **Q2 — NO UNATTENDED FLASH. Every flash waits for the operator's explicit word with the
  device placed in BOOTSEL by their hand.** I decline the "unless fable rules otherwise"
  opening. The controller MAY, unattended, build and sign the image (`sh2-flash -b`) so the
  artifact is ready, and MAY flash at the operator's word under the preconditions in §Q2.

---

## Q1 — when `me` 0.8.1 is tagged and released

### Facts verified

**The brief's tree has moved: H1b is already merged to master.** The brief said master
`308a905`, branch `hashlock-h1b` at `278a0e4` under review. Measured:

```
$ git merge-base --is-ancestor 278a0e4 master && echo merged
merged                                   # (was NOT merged at my first check; merged mid-run)
$ git show -s --format='%h parents=%p %s' 8c83e4e
8c83e4e parents=5f4b634 4d5b6b7 merge: hashlock-h1b (H1b -- me bumps to ms-codec 0.8, refuses a
kind-0x03 preimage plate by name on both verbs; me 0.8.1 unreleased) -- post-impl GREEN 0C/0I at
278a0e4, Minors folded at 4d5b6b7
$ git log --oneline 8c83e4e..master
0c76c63 / c655de6 / fede50f / bf80390    # continuity, briefs, the IMPLEMENTATION RECORD
$ git diff --stat 8c83e4e..master -- crates/ Cargo.lock Cargo.toml
(empty)                                  # no code has moved since the merge
$ git rev-parse --short origin/master
d723cac                                  # 20 local commits unpushed
```

**The post-implementation review is GREEN, persisted, and its Minors are folded with a gate.**

```
$ git show --stat --format='%h %s' 5f4b634 | head -3
5f4b634 report: H1b post-impl adversarial review (opus) -- 0C/0I/3M/2N GREEN at 278a0e4; verbatim
 design/agent-reports/hashlock-H1b-post-impl.md | 384 +++
$ sed -n 11p design/agent-reports/hashlock-H1b-post-impl.md
**Verdict: GREEN — 0 Critical / 0 Important.** 3 Minor, 2 Nit.
$ git show -s --format=%b 4d5b6b7 | head -1
Gate at this tree (own target dir, --no-fail-fast): fmt clean; nextest 619 run / 616 passed / 3
failed (the box-local history_purge trio) / 2 skipped; clippy only the pre-existing is_multiple_of
nightly lint (composer_records.rs:114). No behaviour change: the two new assertions pass unmutated.
```

The `history_purge` trio is the known box-local failure (`design/FOLLOWUPS.md:15709` "the 3
box-local `history_purge`"; the H1b plan §Task 5 says "green but for the three box-local
`history_purge` tests"). The `is_multiple_of` lint is nightly-only: CI pins
`RUST_TOOLCHAIN: '1.85.0'` (`release.yml:48`) and runs `cargo clippy --all-targets --locked
-- -D warnings` (`:116`); the same line (`if s.len() % 2 != 0`) was present at `d723cac`,
whose `test (rust + go)` run 33944274589 succeeded (`gh run list`: `d723cac ci/staging push
completed success`). Neither can turn the staging run red.

**M-1 touched the CHANGELOG that will ship, and the fold landed.** The reviewer measured the
H1b bullet's "or with a wrong X length" over-claim (X = 18 is `Bip93OutsideTheProfile(53)`);
the fold narrowed it:

```
$ git show master:crates/me-cli/CHANGELOG.md | sed -n '50,52p'
  id — or with a wrong X length the codec can name (`PreimageLengthMismatch`,
  which it reaches only when the string length sits in the profile's length
  sets, i.e. X ∈ {16, 17, 20, 21, 24, 25, 28, 29, 32, 33}) — is named a preimage
```

**Version and pin are already 0.8.1 / ms-codec 0.8 at master; ms-codec 0.8.0 is on crates.io.**

```
$ git show master:crates/me-cli/Cargo.toml | grep -nE '^version|ms-codec'
3:version = "0.8.1"
53:ms-codec = "0.8"
$ git show master:Cargo.lock | grep -A1 -E 'name = "(mnemonic-engrave|ms-codec)"'
name = "mnemonic-engrave" / version = "0.8.1" ; name = "ms-codec" / version = "0.8.0"
$ curl -s https://crates.io/api/v1/crates/ms-codec | ...max_version
max_version 0.8.0 updated 2026-09-05T03:28:43.964426Z
```

**F-454 names this cut and explicitly hands the tag to "the operator's call, or a fable
decision".** (`design/FOLLOWUPS.md:15419`, the ADVANCED paragraph on master):

> **Still OPEN**: the version bump is not a release. This closes only when the `v0.8.1` tag
> exists and `release.yml`'s assemble + sign has published the binary — the tag is explicitly
> NOT in the H1b plan's scope (the operator's call, or a fable decision, recorded when taken).
> Until then a release-binary operator still admits `+`-signed paths. Owning phase is
> unchanged: before composer S4's journey runs with a release binary.

The H1b IMPLEMENTATION RECORD ends: "Release of me 0.8.1: fable decision
`decision-me-0.8.1-release-and-flash-rule.md`." — this file.

**The CHANGELOG lives at `crates/me-cli/CHANGELOG.md`** (not the root path the brief gave);
`[Unreleased]` carries H0, the seam-corpus rows, H1b, the L24 id/kind mismatch, and the
`+`-sign refusal (`git show master:crates/me-cli/CHANGELOG.md | grep -n '^## \['` →
`8:## [Unreleased]`, `72:## [0.8.0] - 2026-09-02`).

**How v0.8.0 was cut — fully automated after one tag command; no manual step.**

```
$ git show --stat --format='%h %s' db9173c | tail -4
 Cargo.lock                 | 2 +-
 crates/me-cli/CHANGELOG.md | 2 ++
 crates/me-cli/Cargo.toml   | 2 +-
$ git show db9173c -- crates/me-cli/CHANGELOG.md | grep '^+' | grep -v '^+++'
+## [0.8.0] - 2026-09-02
+
$ git for-each-ref refs/tags/v0.8.0 --format='%(taggerdate:iso) %(subject)'
2026-09-02 03:39:21 -0700 me v0.8.0 — descriptor input and the composer's host inputs
$ git tag -v v0.8.0 → error: no signature found     # annotated, unsigned (-a -F msg)
$ grep -n "startsWith(github.ref, 'refs/tags/v')" .github/workflows/release.yml
329:    if: startsWith(github.ref, 'refs/tags/v')
$ grep -n 'secrets\.' .github/workflows/release.yml
477:  MINISIGN_SECRET_KEY: ${{ secrets.MINISIGN_SECRET_KEY }}
487:  MINISIGN_PASSWORD: ${{ secrets.MINISIGN_SECRET_KEY_PASSWORD }}
$ gh release view v0.8.0 --repo bg002h/mnemonic-engrave --json assets -q '.assets[].name'
mnemonic-engrave-v0.8.0-{linux-amd64,linux-arm64,macos-amd64,macos-arm64}.tar.gz
mnemonic-engrave-v0.8.0-windows-amd64.zip  SHA256SUMS  SHA256SUMS.minisig      # 7 assets
```

`design/agent-reports/composer-S1-push-report.md` records the v0.8.0 ritual verbatim:
`scripts/push-via-staging.sh master` → required context success, no bypass line → `git tag -a
v0.8.0 <tip> -F tag-me-v0.8.0.msg` → `git push origin v0.8.0` → release run 33620586356 all
jobs success → 7 assets. "No `cargo publish` performed." `design/RELEASE_PROCESS.md` does not
exist; the push report is the process record. `scripts/push-via-staging.sh` exists (2433 B,
executable), gates on `test (rust + go)` only and aborts if the tip moves. Branch protection
measured: `{"contexts":["test (rust + go)"],"enforce_admins":false,"strict":false}`. No
`ci/staging` ref exists on origin now (`git ls-remote origin refs/heads/ci/staging` → empty),
so nothing is mid-ritual. `gh` is authenticated as `bg002h`.

**The L26 precedent** (`mnemonic-secret/design/SPEC_ms_hashlock.md:719`): "OPERATOR RULING L26
(2026-09-05): … the operator chose **"release regardless of the device"** — 0.18.0 does not
wait for a measured flash or boot; the controller flashed nothing." And the operator's latest
grant (continuity `c655de6`, line 2133): "pushes, publishes, tags and releases proceed without a
confirmation round".

**H2 and H3 do not touch `me`.** `grep -c 'crates/me-cli\|me-cli'
design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` → `0`. Memory `hashlock-phrase-cycle.md`:
"H3 records (… me's classifier learns the kind via me 0.8.1)" — H3 *records* the 0.8.1 cut; it
does not produce it.

**What RELEASED v0.8.0 does with a preimage plate: refuses it (exit 4), misdiagnosed as
"outside the profile"; never placed.** No 0.8.0 binary is on this box (`target/release/me`
and `~/.cargo/bin/me` both report `me 0.7.0`), so this is source at the tag plus the 0.7.0
proxy, which shares the codec pin:

```
$ git show v0.8.0:crates/me-cli/Cargo.toml | grep ms-codec   → 53:ms-codec = "0.7"
$ git show v0.8.0:crates/me-cli/src/seal/record.rs | grep -ci preimage   → 0
$ git show v0.7.0:crates/me-cli/Cargo.toml | grep ms-codec   → 37:ms-codec = "0.7"
$ printf '%s\n' ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c \
    | ./target/release/me sysw pack --out /tmp/decision-pre.bin; echo exit=$?
me: record 0 (records count from 0) is a VALID BIP-93 codex32 string — the checksum is good — but
not a constellation `ms1` record, so this container cannot place it.
      ... the 4-character id must be `entr`. This one is 75 characters.
      Plain BIP-93 secrets are 48 or 74 characters ... re-encode the entropy as `ms1` rather than
      editing the string.
exit=4                                   # no /tmp/decision-pre.bin written
```

(The vector is `preimage-plate-0x03` from `crates/me-cli/testdata/codex32_seam_vectors.json`
at `278a0e4`.) So v0.8.0 is safe on the funds axis and wrong on the *advice* axis — it tells the
operator to "re-encode the entropy as `ms1`", i.e. to cut a spend secret as a seed. That
misdirection, plus F-454's `+`-sign disagreement with the device, is what 0.8.1 fixes — a
reason to release sooner, not later.

### DECISION (Q1)

**Release v0.8.1 immediately — now, on the merged master — without waiting for H2 or H3.**

Rationale in one line each: the GREEN gate has closed and its Minors are folded with a
gate; F-454's owning phase is "before composer S4's journey runs with a release binary" and
its text says the tag is the close; L26 set the precedent that a host release does not wait
for the device; the operator granted tags and releases without a confirmation round; H2/H3
touch no `me` file, so waiting buys nothing and leaves release-binary users on the wrong
advice; and the release is one CHANGELOG header line plus an unsigned annotated tag on a
workflow that has already proven itself end to end.

### Conditions

1. **No code moved since the merge.** `git diff --stat 8c83e4e..HEAD -- crates/ Cargo.lock
   Cargo.toml` must be empty at the moment of the release commit. If it is not, STOP: the
   GREEN does not cover that diff and it needs its own gate before a tag.
2. **The release commit is the header line only** — `## [0.8.1] - 2026-09-05` inserted
   under `## [Unreleased]` (leaving `[Unreleased]` empty), exactly the shape of `db9173c`.
   `Cargo.toml` (0.8.1) and `Cargo.lock` (0.8.1, ms-codec 0.8.0) already carry the release;
   nothing else belongs in that commit. F-454's CLOSED line is written **after** the release
   is verified, in the records commit, because F-454 says it closes when the tag exists and
   the assets are published.
3. **FREEZE master from the staging push until the tag push.** Master has been moving every
   few minutes (briefs, continuity). Hold every other commit for the window; the script
   aborts if the tip moves, which is the safe failure.
4. **Tag the release commit's SHA explicitly**, not "HEAD at tag time".
5. **No re-review is owed before the tag.** The `4d5b6b7` fold is Minors (a narrowed claim,
   two assertions that pass unmutated, a `:136`→`:137`, one clause) — a wording-class fold
   under the proportional rule; the staging run is its machine check. Belt-and-braces sonnet
   fold-verification may run in parallel but does not gate the tag.
6. **A sonnet push agent runs it** (memory `push-via-sonnet-agent-automatically`), brief names
   the report file `design/agent-reports/push-me-v0.8.1-release.md`, agent writes it itself.
7. **No `cargo publish`** — `me` is not on crates.io (v0.8.0: "No `cargo publish` performed").

### The exact sequence

```sh
cd /scratch/code/shibboleth/mnemonic-engrave

# 0. preconditions (all must hold; any miss = stop and report)
git status --short                                             # empty
git diff --stat 8c83e4e..HEAD -- crates/ Cargo.lock Cargo.toml # EMPTY
git show HEAD:crates/me-cli/Cargo.toml | grep '^version'       # version = "0.8.1"
git ls-remote origin refs/heads/ci/staging                     # empty: no ritual in flight
git tag -l v0.8.1                                              # empty

# 1. release commit: the header line, same shape as db9173c
sed -i '0,/^## \[Unreleased\]$/s//## [Unreleased]\n\n## [0.8.1] - 2026-09-05/' crates/me-cli/CHANGELOG.md
git diff --stat                    # exactly: crates/me-cli/CHANGELOG.md | 2 ++
git add crates/me-cli/CHANGELOG.md
git commit -F /path/to/msg         # fish eats backticks: -F, verify with git log -1 --format=%B
#   subject: me 0.8.1 -- ms-codec 0.8; a kind-0x03 preimage plate refused BY NAME on
#            sysw pack and seal; id/kind mismatch named (L24); +-signed path components
#            refused (F-454); carried in [Unreleased] since 0.8.0
#   trailer: Co-Authored-By + Claude-Session lines
REL=$(git rev-parse HEAD)

# 2. FREEZE master. Staging ritual (gates on `test (rust + go)`, aborts if the tip moves)
scripts/push-via-staging.sh master
#   verify the final push output has NO "Bypassed rule violations" line; ci/staging deleted
git fetch origin && [ "$(git rev-parse origin/master)" = "$REL" ]   # or REL is an ancestor

# 3. tag the release commit (annotated, unsigned, message from a file — like v0.8.0)
git tag -a v0.8.1 "$REL" -F /path/to/tag-me-v0.8.1.msg
#   first line: me v0.8.1 — hashlock preimage plates refused by name; ms-codec 0.8
git cat-file -p v0.8.1 | head -2   # object $REL
git push origin v0.8.1             # expect: * [new tag]  v0.8.1 -> v0.8.1

# 4. watch the tag-event run (full SHA, --repo; gh fails silently empty otherwise)
gh run list --repo bg002h/mnemonic-engrave --commit "$REL" --json databaseId,event,status,conclusion
gh run watch <tag-run-id> --repo bg002h/mnemonic-engrave --exit-status
gh run view  <tag-run-id> --repo bg002h/mnemonic-engrave --json jobs \
   -q '.jobs[] | "\(.name): \(.conclusion)"'          # every job success, incl. assemble + sign + release

# 5. verify the assets and the binary
gh release view v0.8.1 --repo bg002h/mnemonic-engrave --json url,assets -q '.url, (.assets[].name)'
#   expect 7: 4 tar.gz + windows zip + SHA256SUMS + SHA256SUMS.minisig
cd "$(mktemp -d)" && gh release download v0.8.1 --repo bg002h/mnemonic-engrave \
   -p 'SHA256SUMS*' -p '*linux-amd64.tar.gz'
minisign -Vm SHA256SUMS -P RWQPmgBXsuw5yi8W0SfDr8KF+IqY/Z5U2p724emSODS1UPfJBP3agbKW
sha256sum -c --ignore-missing SHA256SUMS
tar xzf mnemonic-engrave-v0.8.1-linux-amd64.tar.gz && ./*/me --version      # me 0.8.1
printf '%s\n' ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c \
   | ./*/me sysw pack --out ./p.bin; echo exit=$?
#   expect exit=4 and stderr naming "hashlock PREIMAGE plate (kind 0x03)"; no p.bin

# 6. records (a second, separate commit; pushed via the ritual again or with the next batch)
#   FOLLOWUPS F-454: CLOSED 2026-09-05 — tag v0.8.1 at $REL, run <id>, 7 assets, url
#   continuity; the push agent's own report at design/agent-reports/push-me-v0.8.1-release.md
```

### Abort rules (Q1)

- Required context fails or times out on `ci/staging` → no master push, no tag; report.
- The final push prints "Bypassed rule violations" → the freeze was broken; do not tag;
  re-stage the new tip (CLAUDE.md, 2026-08-16 lesson).
- `assemble + sign + release` fails after the tag is pushed → **leave the tag alone** (a
  pushed tag is a public name; never delete or move it). Report; the fix is a v0.8.2 after
  the cause is found, or the operator's call at breakfast.
- Asset count ≠ 7, minisign verify fails, or the downloaded `me` does not report `0.8.1` /
  does not name the kind → report as NOT released even though the tag exists; do not close
  F-454.

---

## Q2 — the flash rule while the operator sleeps

### Facts verified

**There is no RP2350 on this computer's USB bus at all — not in BOOTSEL, not running.**

```
$ lsusb | awk '{print $6}' | sort -u | tr '\n' ' '
04e8:6300 05ac:0220 05e3:0608 152d:0580 1a40:0101 1d6b:0002 1d6b:0003 1e71:2007 1ea7:0066 26ce:01a2 26ce:0a08 8087:0033
$ (cd /scratch/code/shibboleth/seedhammer && nix develop --command picotool info)
No accessible RP-series devices in BOOTSEL mode were found.
$ ~/bin/sh/sh2-flash -n          # dry run, writes nothing
== Preflight ==  OK signing key present  OK key matches the burned OTP fingerprint (846aa289…)
                 OK signing script present  OK picotool reachable in the devshell
== Build ==      .. c4a64fc Merge hashlock-h0 ...   would run: nix run .#build-firmware
== Device ==     FAIL: no RP2350 in BOOTSEL (2e8a:000f) on the USB bus.
                 Hold the button on the control board while connecting USB, then re-run.
exit=1
```

So "a device detected ready" is not a state the controller can wait for tonight; it can
only come into being when a hand holds the button while plugging in.

**`sh2-flash` cannot put a running device into BOOTSEL, cannot judge the boot, and signs with
the operator's OTP key.** From `~/bin/sh/sh2-flash` (read in full):

- BOOTSEL is by hand: it only checks `lsusb | grep 2e8a:000f` and dies with "Hold the button
  on the control board while connecting USB" — there is no `picotool reboot -f` of a running
  firmware anywhere in it.
- It signs locally with `~/.sh2/sh2-boot-key.pem` (present, mode 0600), after asserting the
  key's fingerprint equals the OTP slot-1 fingerprint `846aa289…` (constant `SH2_BOOTKEY_FP`),
  and it refuses to flash unless `picotool info -a` on the artifact says `signature: verified`.
- After `picotool load --verify` it runs `picotool reboot` and prints "Judge the boot on
  MACHINE power": `Init()` requires a 20–28 V USB-PD contract before it configures the LCD
  and reboots into BOOTSEL when it does not find one, so on a computer port a correctly
  signed, ACCEPTED image "still gives a dark screen and a device that re-enumerates as
  RP2350 Boot — indistinguishable from a signature rejection".

Memory `sh2-flash-script.md`: "**The SH2 has a single USB-C port**, used for both BOOTSEL and
power." Flashing means tethered to the computer; judging the boot means unplugging it and
connecting the high-wattage supply, then looking at the startup screen for `(UNLOCKED)`.
Every one of those is a hand or an eye. `picotool info` reporting a build id would prove only
that flash holds the image — which `--verify` already proved — never that the bootrom booted
it. There is no boot judgement the controller can make.

Memory `sh2-pd-negotiation-is-slow-sometimes.md`: PD negotiation is non-deterministic in
duration; on 2026-08-10 a correct image "appeared not to boot, was reflashed, and then booted
on the same supply". A controller with no screen would misread that every time.

Memory `sh2-boot-key-burned.md` / `HARDWARE_INVENTORY.md`: three boards answer to
`2e8a:000f` (SH2 chipid `0x77c483b745abf55c`; Pico 2 `0x66d3d60ff20abf2f`; Pico 2 W
`0xb3d19289d3ec3f0e`, secure boot 0) and two have been in BOOTSEL at once; slot 0 (SeedHammer
AB's key) is the recovery path and "must never be revoked"; slots 2–3 free; a boot failure is
"retryable at zero OTP cost" — never burn another slot.

**The operator's own words, three times, put the flash on their word — including in the grant
that lifted everything else.** Continuity `c655de6` line 2133: "pushes, publishes, tags and
releases proceed without a confirmation round; … the flash still needs a device state only
the operator can establish unless fable rules otherwise." Memory `agent-concurrency-budget.md`:
"it does not conjure a device into BOOTSEL -- the flash still waits for the operator to say
the SH2 is ready." H2 plan line 58: "**Flash only via `~/bin/sh/sh2-flash -y` at the
operator's word**; never picotool by hand." Continuity line 1929–1930: "`~/bin/sh/sh2-flash -y`
with the SH2 in BOOTSEL -- ONLY at the operator's word; boot judgement is the operator's".
And the last flash (`a739f75`): "FLASHED fork main 839fa5a at the operator's word; boot
judgement pending on machine power" — the pattern in use.

**State of the firmware tips.**

```
$ git -C /scratch/code/shibboleth/seedhammer log --oneline -1 main ; rev-parse origin/main ; status
c4a64fc Merge hashlock-h0: H0 reader guards ...   origin/main: c4a64fc   (clean, HEAD=main)
$ find /scratch/code/shibboleth /tmp -maxdepth 3 -name 'seedhammerii-*.uf2' -newer .../bg839fa5a.uf2
/scratch/code/shibboleth/seedhammer/seedhammerii-v0.0.0-bg839fa5a.signed.uf2    # nothing newer
$ git -C seedhammer branch -a | grep -iE 'h2|hashlock'
  hashlock-h0                              # H2 has no branch yet
```

The brief called `c4a64fc` "built": no `c4a64fc` image exists on disk — the 1,583,132 /
62,800 measurement in continuity was `-o /dev/null`. The device runs `bg839fa5a` (the last
signed image on disk, 2026-09-04 10:58; `a739f75`). The operator's "Let's assume it booted"
(continuity 1936–1941) was recorded with "the controller has NOT flashed anything … if the
device is later found on `bg839fa5a`, flash before any preimage plate exists". Consequence,
verbatim from the spec (L26): "A device still on 839fa5aa cuts a preimage plate as a seed
until it is flashed." That is the only funds-shaped argument for flashing sooner, and it
needs a preimage plate to exist — which only the operator, awake, running `ms hashlock`, can
make.

### DECISION (Q2)

**The controller may NOT flash the SeedHammer II unattended. Every flash — `c4a64fc` now,
H2's merge tip later — waits for the operator's explicit word in their own message with the
device placed in BOOTSEL by their hand.** I do not rule otherwise on the "unless fable rules
otherwise" clause.

Why, in order of weight: (1) nothing can be detected — no RP2350 is on the bus and the
script cannot create BOOTSEL; (2) even with a device in BOOTSEL, the boot judgement is
physically the operator's (one USB-C port, PD contract before LCD, eyes on the startup
screen), so an unattended flash necessarily ends in "boot judgement pending" with nobody to
judge it — exactly the state the brief says is worse than deferral; (3) there is no benefit
overnight: the guarded firmware matters only when a preimage plate is at the machine, and
that needs the operator at the machine; (4) the operator said it three times, most recently
in the very grant that lifted every other confirmation.

### What the controller MAY do unattended

- **Build and sign, stopping before the device:** `~/bin/sh/sh2-flash -b` in the fork
  checkout at `main = c4a64fc` (clean tree; the dry run above confirms the preflight passes).
  Output `seedhammerii-v0.0.0-bgc4a64fc.uf2` + `.signed.uf2` beside the six existing images
  (`*.uf2` is untracked/ignored — the checkout is clean with six present). Record the signed
  image's sha256 in continuity, noting that a signed sha changes on every signing run
  (randomised ECDSA nonce) and is not an identity across flashes. Repeat for H2's merge tip
  after it merges, so whichever tip the operator picks is ready.
- **Stage the command in continuity** for the operator's first message:
  `~/bin/sh/sh2-flash -y` (builds HEAD = fork main) or
  `~/bin/sh/sh2-flash -y /scratch/code/shibboleth/seedhammer/seedhammerii-v0.0.0-bg<tip>.signed.uf2`.
- **Recommend one flash, not two:** if H2 merges before the operator wakes, flash H2's tip
  (it carries H0); `c4a64fc` alone only if H2 is not merged when they say the word.

### Preconditions for a flash at the operator's word (all must hold)

1. The operator's own message says the SH2 is in BOOTSEL and names the tip (or accepts "fork
   main"). A "proceed"/"go ahead" that does not mention the device is not a flash order.
2. `lsusb | grep -c 2e8a:000f` prints exactly `1`, and `picotool info -a` (devshell) reports
   chipid `0x77c483b745abf55c` — the SH2, not the Pico 2 / Pico 2 W.
3. `git -C /scratch/code/shibboleth/seedhammer status --short` is empty and `HEAD` equals
   `origin/main` equals the tip the operator named. A dirty or unpushed tree → abort (the
   script only warns; the rule is stricter).
4. `~/bin/sh/sh2-flash -y` only, from the fork checkout; never `picotool` by hand; never the
   unsigned `.uf2`.
5. The controller's whole judgement is the script's own: key fingerprint matches OTP slot 1,
   `signature: verified` on the artifact, `picotool load --verify` exit 0. Record those plus
   the signed sha256 in continuity, in the `a739f75` form, with **"boot judgement pending on
   machine power"**. The boot itself is judged by the operator — startup screen with
   `(UNLOCKED)` on machine power — and a dark screen is first a PD-timing question: wait.

### Abort rule

- Any preflight `FAIL`, `lsusb` count ≠ 1, chipid mismatch, dirty tree, or tip ≠ named tip →
  stop before `picotool load`; report; do not retry without the operator.
- `picotool load --verify` non-zero → at most ONE retry of `sh2-flash -y <same .signed.uf2>`
  (retryable at zero OTP cost); a second failure → stop, leave the device in BOOTSEL, report.
- A dark screen after the flash is **not** the controller's to act on: no reflash, no
  recovery image, and **never** `picotool otp`, never a second OTP slot, never any write to
  `BOOT_FLAGS1`, never `picotool reboot -f` of a running firmware. Slot 0 stays intact.
- If the device is ever found back on `bg839fa5a` when a preimage plate exists, the standing
  instruction is "flash before any preimage plate exists" — still at the operator's word.
