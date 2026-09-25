# F-695 review: pipefail / `grep -q` sweep

**Question:** is every change behaviour-preserving except that the SIGPIPE
false-failure is gone, and did the sweep miss or misjudge any instance? Above
all, in the device scripts, can any change turn a refusal into a pass?

**Answer:** yes, the changes preserve behaviour. At every one of the 20 fixed
sites the new code reaches the branch the old code would have reached without
SIGPIPE, for every stubbed producer case, including a producer that fails
outright and one that prints the match and then fails. **No device-script
change turns a refusal into a pass at any output size those scripts can
produce.** The one constructible refusal-to-pass path is a property of bash
here-strings (at 64 KiB or more, with every temp directory unwritable or full).
No current site can reach it (Minor 1). The sweep missed one instance of the
measured class, outside its directory scope, and it fails closed (Minor 3).
Several "safe" reasons in the implementer's table are wrong even though the
verdicts are right (Minor 2).

**0 Critical / 0 Important / 4 Minor / 1 Nit. ready to ship: yes**

Nothing that talks to a device was run. `picotool`, `lsusb` and `devshell` were
shell-function or PATH stubs throughout.

## What was verified, with evidence

| # | check | result |
|---|---|---|
| 1 | Committed `search.sh`, re-run at origin and at `F695_AFTER=f695-pipefail` | 133 / 123 hits, **byte-identical** to `hits-before.txt` / `hits-after.txt` |
| 2 | Device sites: an independent harness (not the implementer's) that extracts the real old text from origin and the new text from the branch by line range, and runs each under the file's own `set` options with an external Python producer that writes **one line per flush** (Rust/Go-like). Seven cases × 10 reps: big_first (1 MiB, match on line 1), absent, empty, fail (exit 1, no output), match_fail (prints match, then exits 1), small_first, big_last | see the table below: correct at every cell |
| 3 | Failing-producer semantics | preserved at every command-producer site (details below) |
| 4 | push-via-staging: the 5 copies | branch blob `9bb4c963981a` in all five repos, one hunk each; origin was `81ee83f0` in all five. Check-runs judging (lines 143-208, byte-identical to origin) exercised with a stubbed `gh`: older-fail/newer-pass → green; older-pass/newer-fail → "CI is not green"; incomplete → refuse; missing context → "NEVER RAN"; `test (ubuntu-latest)x` does not match `test (ubuntu-latest)` → missing. All correct. Bypass detector on a 1 MiB `$OUT` with the match on line 1: old 0/10 detected, new 10/10 |
| 5 | emit.py → mk | ran a copy of the branch's emit.py/gen.py with the output path redirected to scratch (the real script **writes into the live repos' release.yml**). Generated mk `release.yml` is **byte-identical to mnemonic-key origin/main** (full-file diff: 0 lines; smoke block 529 = 529 bytes). The smoke step is top-level `shell: bash` + `set -euo pipefail`, so a failing `encode --help` still turns the step red. The ms, mt and toolkit outputs also match their origin files (0 lines); dm differs by 9 lines, the pre-existing drift the implementer noted. No emitted workflow contains an early-exit reader |
| 6 | Implementer's `repro.py` re-run (`F695_TMP` in scratch) | output **identical** to committed `repro.out`, `OVERALL: PASS` |
| 7 | shellcheck 0.11.0, before → after | sh2-flash 0→0, pico2-bootkey-rehearsal 2→2, sign-firmware 0→0, demo rehearse 21→21, push-master 0→0, push-via-staging 0→0, run-e2e 5→5, dm gen-compose-golden 0→0, seedhammer oracle-live 3→3. `bash -n` passes on all nine |

### Device-script harness (runs that reached the PASS path, of 10)

| site | variant | big_first | absent | empty | fail | match_fail | small_first | big_last |
|---|---|---|---|---|---|---|---|---|
| sh2-flash:288 verified (flash proceeds) | old | **0** | 0 | 0 | 0 | 0 | 10 | 10 |
| | new | 10 | 0 | 0 | 0 | 0 | 10 | 10 |
| sh2-flash:305 lsusb BOOTSEL | old | **0** | 0 | 0 | 0 | 0 | 10 | 10 |
| | new | 10 | 0 | 0 | 0 | 0 | 10 | 10 |
| rehearsal:1105 phase-3 verified | old | **0** | 0 | 0 | 0 | 0 | 10 | 10 |
| | new | 10 | 0 | 0 | 0 | 0 | 10 | 10 |
| rehearsal:180/208/373 WARNING trap (PASS = trap *skipped*) | old | **10 (fail OPEN)** | 10 | 10 | 10 | 0 | 0 | 0 |
| | new | 0 | 10 | 10 | 10 | 0 | 0 | 0 |
| sign-firmware:101 HASH_ERR (PASS = seal branch) | old | 0 | 0 | 0 | 0 | 10 | 10 | 10 |
| | new | 10 | 0 | 0 | 0 | 10 | 10 | 10 |
| sign-firmware:180 verified | old | 0 | 0 | 0 | 0 | 10 | 10 | 10 |
| | new | 10 | 0 | 0 | 0 | 10 | 10 | 10 |
| push-master:162 bypass (PASS = bypass detected) | old | 0 | 0 | 0 | 0 | 10 | 10 | 10 |
| | new | 10 | 0 | 0 | 0 | 10 | 10 | 10 |

For the captured-variable sites (WARNING traps, sign-firmware, push-master),
"fail" and "match_fail" describe only the variable's *content*. Producer failure
is handled earlier by the `out="$(…)" || die` / `set -e` capture. The traps'
match_fail column is 0 because the content contains WARNING and the trap fires.

The harness can fail. The old code's big_first column is the real defect,
reproduced, and my first two harness versions reported false results (a zsh
word-splitting slip, then `eval` returning the `&&`-list status under `set -e`).
I diagnosed both and fixed them before these numbers were taken.

### Failing-producer semantics (brief item 2)

Every file with a changed command-producer site runs under `pipefail`: sh2-flash,
rehearsal and sign-firmware use `-euo`; demo rehearse.sh uses `-uo`; dm
gen-compose-golden uses `-euo`. So the old pipelines already failed on a failing
producer. The `{ v="$(cmd)" && grep -q P <<<"$v"; }` form keeps that: cmd's exit
status becomes the assignment's status and `&&` short-circuits. The fail and
match_fail columns above are 0→0 at sh2-flash:288, sh2-flash:305 and
rehearsal:1105. No site loses producer-failure detection. In emit.py the capture
is a separate top-level statement under `set -e`, which aborts the step. None of
the new variable names (`SIGINFO`, `USB_LIST`, `IMGINFO`, `ms_split_help`,
`compose_help`, `ENCODE_HELP`) collides with an existing name in its file.

## Findings

### Minor 1: a here-string of 64 KiB or more needs a temp file; if none can be created, the grep returns 1 and a `grep … && die` / `if grep; then refuse` site fails OPEN

Bash 5.3 (this box) feeds a here-string through a pipe when it is under about
64 KiB. At that size or above it writes a temp file, trying `$TMPDIR`, `/tmp`,
`/var/tmp` and then the cwd. If none is writable, or the one it picks is full,
the redirection fails. bash prints `cannot create temp file for here-document`
to stderr, and the command's status is 1 **without grep running**.

Reproduction, with `/tmp` as an 8 KiB tmpfs and `/var/tmp` read-only in a user
namespace, and a 100 KB captured value whose WARNING is on its last line:

```
new form: grep -qi WARNING <<<"$x" && die   ->  "cannot create temp file for here-document: No space left on device", PASSED (trap skipped), rc=0
old form: printf '%s' "$x" | grep -qi WARNING && die   ->  DIE (refused), rc=3
```

With `/tmp` read-only the threshold is exact: 60000 bytes match, 65535 and above
return NOMATCH rc=1.

This is the only way any change turns a refusal into a pass. **No current site
can reach it.** The OTP `picotool otp get` output is a few hundred bytes (pipe
path), `git push` output is far below 64 KiB, and a GitHub runner's `/tmp` is
writable. It also fails loudly, not silently. At every size the old form was
already worse: above about 16 KiB it fails OPEN whenever the match is early.
Only the fail-OPEN sites are exposed: the three WARNING traps, the push-master
and push-via-staging bypass detectors, gen-compose-golden's refusal, and
release.yml's docs-only detector. The `|| die` sites fail closed.

*Optional hardening*, not required to ship: at the three OTP traps, a pure-bash
test uses no temp file and no second process, e.g.
`[[ ${out,,} == *warning* ]] && die …`.

### Minor 2: seven "safe" reasons are factually wrong. The run block itself sets pipefail

Table rows #42, #48, #49, #50, #51, #59 and #65 give "no `shell:` (no pipefail)".
The step's own `run:` block begins `set -euo pipefail` in each of:
descriptor-mnemonic `man-pages.yml:338`; mnemonic-secret `man-release.yml:79`,
`:267`, `:289`, `:327`; mnemonic-key `musl-binaries.yml:196`; and toolkit
`man-pages.yml:295`. I checked this by parsing each workflow and locating the
step. The **verdicts still stand** for a different reason: each producer (a grep
of `Cross.toml` or `Cargo.lock`) emits exactly **one** matching line. Measured:
1 `image =` line in each Cross.toml, and 1 `mnemonic-engrave?rev=` in ms
Cargo.lock. The producer has nothing left to write when `head` exits, so it
cannot be SIGPIPE'd. #79 is additionally masked by `|| true`. This only corrects
the record. The other workflow rows (engrave release.yml:93, ms rust.yml:301,
tk gui-pin-drift / install-pin-check / manual-gui) have no pipefail in their
blocks, as stated.

### Minor 3: missed instance outside the search scope. dm `crates/md-cli/tests/fixtures/decompose/generate.sh:69-70`

```sh
if ! "$MK" encode --keys /dev/null --help >/dev/null 2>&1 \
   && ! "$MK" encode --help 2>/dev/null | grep -q -- '--keys'; then
  echo "MK=$MK does not support 'mk encode --keys' — it is too old." >&2; exit 1
```

This is under `set -euo pipefail` and is exactly the measured class: a Rust
`mk encode --help` piped into `grep -q`. `search.sh` only selects `scripts/`,
`ci/` and `demo/`, so it never looked at `crates/**/tests/fixtures/`. A SIGPIPE
makes the pipeline false and the `!` makes it true: a **false "too old"
refusal**, which fails closed. It runs only when the first probe fails. With a
stub `MK` that fails the first probe and prints `--keys` on line 1 of a 1 MB
help, the real text refused 20/20. At the real help size the implementer
measured about 1/200.

A broader search found nothing else to fix. It covered every tracked file in all
seven repos plus mnemonic-transaction, with no directory filter. It also looked
for readers the committed regex cannot see: `env X=… grep`, `command grep`,
`egrep`/`rg`, flags after the pattern, `sed '/x/q'`, `|&`, `yes |`, and
multi-line pipelines repo-wide (none). The other 19 new locations were Rust
identifiers, prose, masked (`|| true`) sites, and small builtin writes. Specifically:
tk `docs/manual/tests/lint.sh:179` is masked; md `pathological/extract-keyed-card.sh:57`
writes 22 lines through builtin printf; md `seating/generate.sh:761` uses
`head -n -1`, which reads everything; mt `check-provenance.sh:80` is masked; and
mt `live-smoke-test.sh:76` is an informational `sed file | head -1` of a short
stderr.

**Suggested:** fix generate.sh:69 with the same capture form, or record it as a
known residue. Widen `search.sh` to every tracked file, so the claim "the sweep
is complete" covers the tree and not three directories.

### Minor 4: one "safe" verdict rests on output size alone with an external producer (the brief asks for these to be listed)

- **#19 engrave `scripts/plan-build-gate-me.sh:61`**:
  `rustup toolchain list 2>/dev/null | grep -q "^$PIN-"` under `set -euo pipefail`.
  The stated reason is "a few lines; a false result only skips the toolchain pin".
  rustup is a Rust binary, so size alone is not a sufficient argument (the
  report's own mk measurement says so). Measured here: 315 bytes, **0/300**
  false results with the match on line 1, so it is not observed to flake. The
  consequence is that the build gate runs on the default toolchain instead of
  the CI pin. The toolchain line it prints makes that visible, but the gate does
  not fail. Cheap to convert to the capture form. It does not block.

The other external-producer "safe" verdicts I spot-checked rest on mechanism or
on measurement, not on size:

- **openssl `-text` at rehearsal:267 and sign-firmware:75.** The matched line is
  openssl's last line (383 bytes), and 0/500 runs each failed.
- **bitcoind `--version | head -1` (dm #41, tk #61).** 466 bytes from C++
  stdio, 0/300 against local Core v25. It is the step's last line and fails
  closed.
- **git show (tk #72).** 0/300, with the match on line 36.
- **sh2-flash:255 `find | sort | head -1`.** 0/100 aborts at 10, 60 and 200
  files (up to 12.8 KB). The real directory holds 54 files, and the site fails
  closed (abort before flashing).
- **dm release.yml:165 and check-install-links:65.** A non-`-q` grep or `sed -n`
  reads the whole producer.
- **ms plan-build-gate-ms:124.** grep reads all of nextest's output.
- **ms plan-build-gate-ms:140-141.** `$OUT` is a short decode error.
- **"No -e" verdicts.** I confirmed plan-cite-gate, plan-fold-sweep and
  verify-releases (`set -uo pipefail`), and seedhammer chain-mutation-check and
  tk doc-flag-lint.test.sh (`set -uo pipefail`).
- **"No pipefail" verdicts.** tk install.sh, install-verify, install-msrv-guard
  and install-assets are `#!/bin/sh` + `set -eu`, with 0 occurrences of
  `pipefail`.

### Nit: "the docs-only `run:` block, extracted, is shellcheck-clean"

Extracted literally, it gives 1 finding: SC2296 on `"${{ github.event.before }}"`.
That is Actions template syntax, identical at origin (1 → 1). The claim holds
only once expressions are substituted, and the record should say so.

## Settled, not re-derived

The 20-site before/after reproduction, the byte-identity of the five
push-via-staging copies, and the mk smoke containment were all independently
re-run above rather than taken from the implementer's report.

ready to ship: yes
