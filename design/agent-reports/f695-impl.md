# F-695 implementation report — `producer | grep -q` / `| head` under `pipefail`

**Outcome.** 133 search hits across the seven repos; **20 are fixed**, 113 are
safe (reasons below). All 20 fixes are on branch `f695-pipefail` in five repos,
none pushed, merged or tagged. At every fixed site the reproduction goes from
**0/10 to 10/10** on a 1 MiB producer with the match on line 1. Absent, empty
and failing-producer outcomes are unchanged. No command talked to a device.

| repo | worktree | base (origin default) | commits |
|---|---|---|---|
| mnemonic-engrave | `/scratch/code/shibboleth/me-worktrees/f695` | `e5c0a52f` | `facb66f8` fixes, `448ef559` evidence |
| descriptor-mnemonic | `/scratch/code/shibboleth/dm-worktrees/f695` | `1bea51ec` | `6db19494` |
| mnemonic-secret | `/scratch/code/shibboleth/ms-worktrees/f695` | `cfbcfdd` | `9d26302` |
| mnemonic-toolkit | `/scratch/code/shibboleth/tk-worktrees/f695` | `af5cc1a5` | `35fde2ec` |
| seedhammer (fork) | `/scratch/code/shibboleth/sh-worktrees/f695` | `80b12c5` | `51a9e0d` (DCO `-s`, Brian Goss) |

mnemonic-key (`65a526c`) and mnemonic-gui (`3ca1d60`) need no change.

## Search command

`design/evidence/f695-pipefail/search.sh` (committed in engrave `448ef559`).
It runs `git show` on each repo's origin default branch. It selects `scripts/`,
`ci/` and `demo/` at any depth, Makefiles, `*.mk`, `.github/workflows/*.yml`,
and for seedhammer only the files the fork adds or modifies relative to
`upstream/main`, including `cmd/*/*.sh`. It excludes `vendor/`, `third_party/`,
`target/` and `design/`. Readers matched after a `|`:
`grep` with `-q/-m/-l/--quiet/--max-count/--files-with-matches`, `head`,
`sed …q`, `awk … exit`, and `read`. A second pass also checks multi-line
pipelines (a reader at the start of a line whose previous line ends in `|`);
it found none.

```sh
bash design/evidence/f695-pipefail/search.sh                          # origin: 133 hits
F695_AFTER=f695-pipefail bash design/evidence/f695-pipefail/search.sh  # branch: 123 hits
```

The outputs are committed as `hits-before.txt` and `hits-after.txt`. With line
numbers stripped, the diff has **20 removed** lines, which are exactly the 20
fixed sites, and **10 added** lines, which are the new F-695 comments that quote
the old shape. A second grep for other early-exit readers (`cmp`, `file -`,
`dd`, `perl … last`) found nothing in scope. Nothing matched in any Makefile.

## Measurements that set the "safe" bar

- **Bash builtin writing a captured string** (`echo/printf "$x" | grep -q`,
  match on line 1, pipefail), 200 runs per size: 0 failures at 16 KB or less,
  1/200 at 32 KB, 7–20/200 at 60 KB, 149–160/200 at 70 KB, 200/200 at 128 KB and
  above. The here-string form failed 0/200 at every size. So "a small builtin
  string" is safe here, with small meaning a few KB.
- **An external line-buffered producer is NOT safe when small.** A direct
  `mk encode --help | grep -q` with 3,851 bytes of output failed **1/200**
  locally under pipefail. This is the mk 65a526c class. Rust (and Go) write per
  line, so grep can exit between two writes. Any direct `<rust/go binary> | grep
  -q` whose status is used is therefore a **fix**, whatever its size.
- **GitHub Actions**: `run:` without `shell:` is `bash -e {0}`, with **no
  pipefail**. Only an explicit `shell: bash` gives `-eo pipefail`. I checked each
  workflow hit for step-level `shell:` and job/workflow `defaults:`.

## Instance table (all 133 hits, origin default branches)

| # | site | code | verdict |
|---|---|---|---|
| 1 | `mnemonic-engrave/.github/workflows/release.yml:93` | `if echo "$CHANGED" \| grep -qvE '^(design/\|[^/]+\.md$)'; then` | **fix (preventive)**: no pipefail today; under it a SIGPIPE'd echo on a large diff flips the step to docs_only=true and skips tests (fails OPEN) |
| 2 | `mnemonic-engrave/demo/sh2/check-install-links.sh:65` | `latest=$(gh_api "https://api.github.com/repos/$repo/releases/latest" \| sed -n 's/.*"tag_name": *"\([^"]*\)...` | safe: sed -n reads all of gh_api's output; head reads sed's single small line |
| 3 | `mnemonic-engrave/demo/sh2/deploy.sh:51` | `rrun "ls -1t ${NGINX_CONF}.bak-* 2>/dev/null \| head -1"` | safe: runs on the remote via ssh (remote login shell, no pipefail); ls output tiny; output only |
| 4 | `mnemonic-engrave/demo/sh2/deploy.sh:52` | `rrun "latest=\$(ls -1t ${NGINX_CONF}.bak-* 2>/dev/null \| head -1); \` | safe: runs on the remote via ssh (remote login shell, no pipefail); ls output tiny; output only |
| 5 | `mnemonic-engrave/demo/sh2/rehearse.sh:29` | `if ! ms split --help 2>&1 \| grep -q -- '--in'; then` | **fix**: Rust `ms split --help` piped straight to grep -q under pipefail; false 'crates.io build' refusal (closed) |
| 6 | `mnemonic-engrave/demo/sh2/rehearse.sh:60` | `[[ $(grep -o '^md1[a-z0-9]*' <<<"$out" \| head -1) == "$GOOD" ]] \` | safe: `$(...)` inside `[[ ]]`/message, no -e; status never read |
| 7 | `mnemonic-engrave/demo/sh2/rehearse.sh:61` | `&& ok "repaired string is byte-identical to the original" \|\| bad "repair output" "$(grep -o '^md1[a-z0-9]...` | safe: `$(...)` inside `[[ ]]`/message, no -e; status never read |
| 8 | `mnemonic-engrave/demo/sh2/rehearse.sh:150` | `\|\| bad "john->search pipeline" "$(grep -iE 'match\|no match\|error' <<<"$res" \| head -1)"` | safe: message text only; status never read |
| 9 | `mnemonic-engrave/demo/sh2/update.sh:77` | `prev=$("${SSH[@]}" "ls -1dt ${REMOTE_DIR}.prev-* 2>/dev/null \| head -1" \|\| true)` | safe: remote command, masked by `|| true`; value comes from head's output |
| 10 | `mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh:180` | `printf '%s' "$out" \| grep -qi 'WARNING' \` | **fix (special care)**: WARNING trap `printf | grep -q && die` — a SIGPIPE'd printf SKIPS the die (fails OPEN) |
| 11 | `mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh:187` | `printf '%s' "$v" \| grep -qE '^[0-9a-f]+$' \|\| die "unexpected field value '$v' for $sel"` | safe: `$v` is a parsed hex field we built (tens of bytes); builtin write completes before grep can exit; fails closed |
| 12 | `mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh:208` | `printf '%s' "$out" \| grep -qi 'WARNING' \` | **fix (special care)**: WARNING trap `printf | grep -q && die` — a SIGPIPE'd printf SKIPS the die (fails OPEN) |
| 13 | `mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh:266` | `printf '%s' "$errtxt" \| grep -qi 'ASN1 OID: secp256k1' \` | safe: `$errtxt` is openssl's stderr on a successful read (empty/tiny); a false result falls through to line 267, then die (closed) |
| 14 | `mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh:267` | `\|\| openssl ec -in "$1" -noout -text 2>/dev/null \| grep -qi 'ASN1 OID: secp256k1' \` | safe: the matched `ASN1 OID: secp256k1` line is openssl's LAST output line, so the producer has finished writing when grep exits; fails closed |
| 15 | `mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh:373` | `printf '%s' "$out" \| grep -qi 'WARNING' \` | **fix (special care)**: WARNING trap `printf | grep -q && die` — a SIGPIPE'd printf SKIPS the die (fails OPEN) |
| 16 | `mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh:1105` | `picotool info -a "$WORKDIR/blinky-mykey.signed.uf2" 2>/dev/null \| grep -qi 'signature: *verified' \` | **fix (special care)**: phase-5 `picotool info | grep -q verified || die` (fails closed) |
| 17 | `mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh:1128` | `FW_REAL="$(ls -1 "$SEEDHAMMER_DIR"/seedhammerii-*.uf2 2>/dev/null \| grep -v '\.signed\.uf2$' \| head -1 \|...` | safe: masked by `|| true`; ls output tiny; FW_REAL comes from head's output |
| 18 | `mnemonic-engrave/scripts/plan-build-gate-me.sh:60` | `PIN="$(grep -E '^\s*RUST_TOOLCHAIN:' "$SRC/.github/workflows/release.yml" 2>/dev/null \| head -1 \| sed -E ...` | safe: grep of a few RUST_TOOLCHAIN lines (block-buffered, one small write) into head |
| 19 | `mnemonic-engrave/scripts/plan-build-gate-me.sh:61` | `if [ -n "$PIN" ] && rustup toolchain list 2>/dev/null \| grep -q "^$PIN-"; then export RUSTUP_TOOLCHAIN="$P...` | safe: `rustup toolchain list` is a few lines; a false result only skips the toolchain pin (informational) |
| 20 | `mnemonic-engrave/scripts/plan-build-gate.sh:202` | `\| grep -E '^(warning\|error)' \| sort \| uniq -c \| sort -rn \| head -10 \` | safe: informational; head reads sort's output (small, one write); guarded by `|| echo none` |
| 21 | `mnemonic-engrave/scripts/plan-cite-gate.sh:65` | `` # F-115: the search used to be `... -print \| head -1`, which picks the FIRST `` | not code: comment text quoting the shape |
| 22 | `mnemonic-engrave/scripts/plan-cite-gate.sh:89` | `f="$(printf '%s\n' "$matches" \| head -1)"` | safe: no -e; assignment status never read |
| 23 | `mnemonic-engrave/scripts/plan-cite-gate.sh:122` | `loc=$(grep -rnE "$top" "$GOREPO/$pkg"/*.go \| head -1)` | safe: no -e; assignment status never read |
| 24 | `mnemonic-engrave/scripts/plan-cite-gate.sh:125` | `loc=$(grep -rnE "$grouped" "$GOREPO/$pkg"/*.go \| head -1)` | safe: no -e; assignment status never read |
| 25 | `mnemonic-engrave/scripts/plan-fold-sweep.sh:135` | `grep -nF -- "$bare" "$doc" \| head -4 \| sed 's/^/                 /' \| cut -c1-118` | safe: no -e; output only |
| 26 | `mnemonic-engrave/scripts/push-master.sh:162` | `if printf '%s' "$OUT" \| grep -qi 'bypass'; then` | **fix**: bypass detector `printf | grep -qi bypass` in an `if` (fails OPEN) |
| 27 | `mnemonic-engrave/scripts/push-via-staging.sh:154` | `if printf '%s\n' "${CR[@]}" \| awk -F'\t' '$2!="completed"{f=1} END{exit !f}'; then` | safe: awk's `exit` is in END, so awk reads all input |
| 28 | `mnemonic-engrave/scripts/push-via-staging.sh:213` | `if echo "$OUT" \| grep -qi "bypassed rule violations"; then` | **fix**: bypass detector `echo | grep -qi` in an `if` (fails OPEN); 5 copies kept byte-identical |
| 29 | `mnemonic-engrave/scripts/release-workflows/emit.py:32` | `"$BIN" encode --help \| grep -q -- '--policy-id-stub' \|\| {` | **fix**: generator still emits the shape mk fixed by hand in 65a526c (step is `shell: bash`); emits mk's fixed text now |
| 30 | `mnemonic-engrave/scripts/sh2-flash:255` | `-printf '%T@ %p\n' \| sort -rn \| head -1 \| cut -d' ' -f2-)"` | safe: sort's output (a few filenames) is one small write into head; set -e would abort BEFORE flashing (closed) |
| 31 | `mnemonic-engrave/scripts/sh2-flash:288` | `devshell picotool info -a "$SIGNED" 2>/dev/null \| grep -q 'signature:.*verified' \` | **fix (special care)**: `devshell picotool info | grep -q verified || die` (fails closed) |
| 32 | `mnemonic-engrave/scripts/sh2-flash:305` | `if ! lsusb 2>/dev/null \| grep -qi "$BOOTSEL_ID"; then` | **fix (special care)**: `lsusb | grep -qi` BOOTSEL check (fails closed) |
| 33 | `mnemonic-engrave/scripts/sign-firmware.sh:75` | `openssl ec -in "$KEY" -noout -text 2>/dev/null \| grep -qi 'secp256k1\\|ASN1 OID: secp256k1' \` | safe: `ASN1 OID: secp256k1` is openssl's last output line; fails closed |
| 34 | `mnemonic-engrave/scripts/sign-firmware.sh:101` | `elif printf '%s' "$HASH_ERR" \| grep -qiE 'missing SIGNATURE section\|missing HASH_DEF item'; then` | **fix (special care)**: HASH_ERR classifier; a false result goes to the `else die` (closed) |
| 35 | `mnemonic-engrave/scripts/sign-firmware.sh:180` | `printf '%s' "$INFO" \| grep -qi 'signature: *verified' \` | **fix (special care)**: final `printf "$INFO" | grep -q verified || die` (fails closed) |
| 36 | `mnemonic-engrave/scripts/test/run-e2e.sh:66` | `if printf '%s' "$out" \| grep -qiE "$want"; then` | **fix**: `printf "$out" | grep -qiE "$want"` in the e2e harness (false red) |
| 37 | `mnemonic-engrave/scripts/verify-releases.sh:60` | `(cd "$d" && sha256sum -c "$sums" --ignore-missing 2>&1 \| grep -v ': OK$' \| head -4 \| sed 's/^/        /')` | safe: `set -uo pipefail` with no -e; statuses never read (output/assignment only) |
| 38 | `mnemonic-engrave/scripts/verify-releases.sh:65` | `lin=$(find "$d" -name "*linux*amd64*.tar.gz" -o -name "*x86_64-linux*.tar.gz" \| head -1)` | safe: `set -uo pipefail` with no -e; statuses never read (output/assignment only) |
| 39 | `mnemonic-engrave/scripts/verify-releases.sh:71` | `b=$(find "$ex" -type f -name "$bin" \| head -1)` | safe: `set -uo pipefail` with no -e; statuses never read (output/assignment only) |
| 40 | `mnemonic-engrave/scripts/verify-releases.sh:76` | `v=$("$b" --version 2>&1 \| head -1)` | safe: `set -uo pipefail` with no -e; statuses never read (output/assignment only) |
| 41 | `descriptor-mnemonic/.github/workflows/bitcoind-differential.yml:94` | `"./bitcoin-${BITCOIND_VERSION}/bin/bitcoind" --version \| head -1` | safe: `shell: bash` (pipefail) but bitcoind --version is <1 KB written once by C++ stdio; informational line |
| 42 | `descriptor-mnemonic/.github/workflows/man-pages.yml:338` | `CROSS_IMG="$(grep -E '^image *=' Cross.toml \| head -1 \| sed -E 's/.*"([^"]+)".*/\1/')"` | safe: no `shell:` (no pipefail); grep of Cross.toml into head |
| 43 | `descriptor-mnemonic/.github/workflows/release.yml:165` | `POL="$("$BIN" encode --from-policy 'or(pk(@0),and(pk(@1),older(144)))' --context segwitv0 --group-size 0 \|...` | safe: `grep '^md1'` reads all of md's output; head reads one small line |
| 44 | `descriptor-mnemonic/ci/repro/vendor-freshness.sh:45` | `MINISCRIPT_REV="$(grep -oE 'rust-miniscript\?rev=[0-9a-f]{40}' Cargo.lock \| head -1 \| grep -oE '[0-9a-f]{...` | safe: masked by `|| true`; value comes from head's output; grep -o of Cargo.lock is a few short lines |
| 45 | `descriptor-mnemonic/scripts/gen-compose-golden.sh:34` | `if "$MD" compose --help 2>&1 \| grep -q -- '--unspendable'; then` | **fix**: `md compose --help | grep -q -- --unspendable` refusal; a false result SKIPS the refusal (fails OPEN) |
| 46 | `descriptor-mnemonic/scripts/push-via-staging.sh:154` | `if printf '%s\n' "${CR[@]}" \| awk -F'\t' '$2!="completed"{f=1} END{exit !f}'; then` | safe: awk's `exit` is in END, so awk reads all input |
| 47 | `descriptor-mnemonic/scripts/push-via-staging.sh:213` | `if echo "$OUT" \| grep -qi "bypassed rule violations"; then` | **fix**: bypass detector `echo | grep -qi` in an `if` (fails OPEN); 5 copies kept byte-identical |
| 48 | `mnemonic-secret/.github/workflows/man-release.yml:79` | `REV="$(grep -oE 'mnemonic-engrave\?rev=[0-9a-f]{40}' Cargo.lock \| head -1 \| grep -oE '[0-9a-f]{40}' \|\| ...` | safe: no `shell:` (no pipefail); status is the last grep's |
| 49 | `mnemonic-secret/.github/workflows/man-release.yml:267` | `IO_LIB_REV="$(grep -oE 'mnemonic-engrave\?rev=[0-9a-f]{40}' Cargo.lock \| head -1 \| grep -oE '[0-9a-f]{40}')"` | safe: no `shell:` (no pipefail); status is the last grep's |
| 50 | `mnemonic-secret/.github/workflows/man-release.yml:289` | `CROSS_IMG="$(grep -E '^image *=' Cross.toml \| head -1 \| sed -E 's/.*"([^"]+)".*/\1/')"` | safe: no `shell:` (no pipefail); grep of Cross.toml into head |
| 51 | `mnemonic-secret/.github/workflows/man-release.yml:327` | `IO_LIB_REV="$(grep -oE 'mnemonic-engrave\?rev=[0-9a-f]{40}' Cargo.lock \| head -1 \| grep -oE '[0-9a-f]{40}')"` | safe: no `shell:` (no pipefail); status is the last grep's |
| 52 | `mnemonic-secret/.github/workflows/rust.yml:301` | `/usr/bin/bash --version \| head -1` | safe: no `shell:` (no pipefail); informational |
| 53 | `mnemonic-secret/ci/repro/vendor-freshness.sh:40` | `IO_LIB_REV="$(grep -oE 'mnemonic-engrave\?rev=[0-9a-f]{40}' Cargo.lock \| head -1 \| grep -oE '[0-9a-f]{40}...` | safe: masked by `|| true`; value comes from head's output; grep -o of Cargo.lock is a few short lines |
| 54 | `mnemonic-secret/scripts/plan-build-gate-ms.sh:124` | `cargo nextest run -p ms-codec --locked --no-capture -E 'test(codeword_distance)' 2>&1 \| grep -E "codeword ...` | safe: grep reads all of nextest's output; its few matching lines are one small write into head |
| 55 | `mnemonic-secret/scripts/plan-build-gate-ms.sh:140` | `echo "$OUT" \| head -2; echo "exit=$RC"` | safe: `$OUT` is a short error message (builtin write, far under the ~16 KiB measured threshold) |
| 56 | `mnemonic-secret/scripts/plan-build-gate-ms.sh:141` | `[ "$RC" -eq 2 ] && echo "$OUT" \| grep -q "reserved-prefix byte was 0x03" \|\| { echo "DOWNGRADE ROW FAILED...` | safe: `$OUT` is a short error message; a false result exits 4 (closed) |
| 57 | `mnemonic-secret/scripts/push-via-staging.sh:154` | `if printf '%s\n' "${CR[@]}" \| awk -F'\t' '$2!="completed"{f=1} END{exit !f}'; then` | safe: awk's `exit` is in END, so awk reads all input |
| 58 | `mnemonic-secret/scripts/push-via-staging.sh:213` | `if echo "$OUT" \| grep -qi "bypassed rule violations"; then` | **fix**: bypass detector `echo | grep -qi` in an `if` (fails OPEN); 5 copies kept byte-identical |
| 59 | `mnemonic-key/.github/workflows/musl-binaries.yml:196` | `CROSS_IMG="$(grep -E '^image *=' Cross.toml \| head -1 \| sed -E 's/.*"([^"]+)".*/\1/')"` | safe: no `shell:` (no pipefail); grep of Cross.toml into head |
| 60 | `mnemonic-key/ci/repro/vendor-freshness.sh:51` | `\| head -1 \| grep -oE '[0-9a-f]{40}' \|\| true)"` | safe: masked by `|| true`; value comes from head's output; grep -o of Cargo.lock is a few short lines |
| 61 | `mnemonic-toolkit/.github/workflows/bitcoind-differential.yml:121` | `"./bitcoin-${BITCOIND_VERSION}/bin/bitcoind" --version \| head -1` | safe: `shell: bash` (pipefail) but bitcoind --version is <1 KB written once by C++ stdio; informational line |
| 62 | `mnemonic-toolkit/.github/workflows/gui-pin-drift-check.yml:72` | `PIN=$(grep -oE 'mnemonic-gui-v[0-9]+\.[0-9]+\.[0-9]+' scripts/install.sh \| head -n1)` | safe: no `shell:` (no pipefail); grep of a small file into head |
| 63 | `mnemonic-toolkit/.github/workflows/gui-pin-drift-check.yml:104` | `LOWER=$(printf '%s\n%s\n' "$PIN_V" "$LATEST_V" \| sort -V \| head -n1)` | safe: no `shell:` (no pipefail); two-line input to sort |
| 64 | `mnemonic-toolkit/.github/workflows/install-pin-check.yml:48` | `PIN=$(grep -oE 'mnemonic-toolkit-v[0-9]+\.[0-9]+\.[0-9]+' scripts/install.sh \| head -n1)` | safe: no `shell:` (no pipefail); grep of a small file into head |
| 65 | `mnemonic-toolkit/.github/workflows/man-pages.yml:295` | `CROSS_IMG="$(grep -E '^image *=' Cross.toml \| head -1 \| sed -E 's/.*"([^"]+)".*/\1/')"` | safe: no `shell:` (no pipefail); grep of Cross.toml into head |
| 66 | `mnemonic-toolkit/.github/workflows/manual-gui.yml:82` | `\| head -1 \` | safe: no `shell:` (no pipefail); grep of a small file into head |
| 67 | `mnemonic-toolkit/.github/workflows/manual-gui.yml:268` | `\| head -1 \` | safe: no `shell:` (no pipefail); grep of a small file into head |
| 68 | `mnemonic-toolkit/ci/doc-flag-lint.test.sh:52` | `grep -F '[lint] FAIL' <<<"$out" \| head -5 \| sed 's/^/     /'` | safe: no -e; failure-branch output only |
| 69 | `mnemonic-toolkit/ci/doc-flag-lint.test.sh:65` | `else fail=$((fail + 1)); echo "FAIL baseline: unmutated manual fails flag-coverage"; step4 "$out" \| grep -...` | safe: no -e; failure-branch output only |
| 70 | `mnemonic-toolkit/ci/doc-gate-guard.sh:134` | `` # Here-strings throughout, never `printf \| grep -q`: under pipefail, grep -q `` | not code: comment text quoting the shape |
| 71 | `mnemonic-toolkit/ci/doc-gate-guard.test.sh:48` | `grep -oE 'bash ci/doc-gate-guard\.sh .*$' <<<"$src" \| head -1` | safe: grep -o of a one-match here-string; one small write into head |
| 72 | `mnemonic-toolkit/ci/doc-gate-guard.test.sh:236` | `if git -C "$W" show 15443535:.github/workflows/examples.yml 2>/dev/null \| grep -qE "^    paths:"; then` | safe: `git show` of a fixed 9,431-byte historical blob, written in one write; measured 0/300 failures |
| 73 | `mnemonic-toolkit/ci/repro/remap-off-negative.sh:146` | `` # residue_hits (residue-lib.sh), never `grep \| head -1 \| grep -q`: under `` | not code: comment text quoting the shape |
| 74 | `mnemonic-toolkit/ci/repro/remap-off-negative.sh:180` | `if grep -aq "/build-a" "$WORK/out-a/$BIN" \|\| grep -aq "/build-b" "$WORK/out-b/$BIN"; then` | not a pipe: `||` matched the search pattern |
| 75 | `mnemonic-toolkit/ci/repro/residue-lib.sh:11` | `#     grep -aEo "$RE" "$file" \| head -1 \| grep -q .` | not code: comment text quoting the shape |
| 76 | `mnemonic-toolkit/ci/repro/residue-lib.sh:48` | `` # residue_hits, never `grep \| head -1 \| grep -q`: under pipefail that shape `` | not code: comment text quoting the shape |
| 77 | `mnemonic-toolkit/ci/repro/residue.test.sh:10` | `` # than a pipe buffers, which is what made the old `grep \| head -1 \| grep -q` `` | not code: comment text quoting the shape |
| 78 | `mnemonic-toolkit/ci/repro/residue.test.sh:66` | `if ! bash -c 'set -euo pipefail; grep -aEo "/project" "$1" \| head -1 \| grep -q .' _ "$WORK/heavy-project....` | safe (deliberate): reproduces the OLD shape on purpose as the F-675 control |
| 79 | `mnemonic-toolkit/ci/repro/residue.test.sh:70` | `echo "info - OLD 'grep \| head -1 \| grep -q' shape reported NO residue in $old_false/20 runs on heavy resi...` | not code: text in an echo |
| 80 | `mnemonic-toolkit/scripts/install-assets.test.sh:103` | `b=$(find "$x" -type f -name "$n" \| head -n1)` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 81 | `mnemonic-toolkit/scripts/install-assets.test.sh:106` | `if readelf -d "$b" 2>/dev/null \| grep -q '(NEEDED)'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 82 | `mnemonic-toolkit/scripts/install-msrv-guard.test.sh:74` | `if printf '%s' "$out_old" \| grep -q 'mnemonic-gui needs rustc'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 83 | `mnemonic-toolkit/scripts/install-msrv-guard.test.sh:79` | `if printf '%s' "$out_old" \| grep -q 'CARGO-INVOKED.*mnemonic-gui'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 84 | `mnemonic-toolkit/scripts/install-msrv-guard.test.sh:87` | `if printf '%s' "$out_new" \| grep -q 'mnemonic-gui needs rustc'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 85 | `mnemonic-toolkit/scripts/install-msrv-guard.test.sh:94` | `if printf '%s' "$out_bin" \| grep -q 'mnemonic-gui needs rustc'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 86 | `mnemonic-toolkit/scripts/install-msrv-guard.test.sh:96` | `elif printf '%s' "$out_bin" \| grep -q 'CURL-INVOKED.*mnemonic-gui-v'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 87 | `mnemonic-toolkit/scripts/install-verify.test.sh:88` | `&& printf '%s' "$out" \| grep -q 'verified sha256'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 88 | `mnemonic-toolkit/scripts/install-verify.test.sh:98` | `if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" \| grep -q 'REFUSED .*sha256 mismatch'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 89 | `mnemonic-toolkit/scripts/install-verify.test.sh:106` | `if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" \| grep -q 'REFUSED .*no checksum'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 90 | `mnemonic-toolkit/scripts/install-verify.test.sh:115` | `&& printf '%s' "$out" \| grep -q 'REFUSED .*no checksum'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 91 | `mnemonic-toolkit/scripts/install-verify.test.sh:123` | `if [ "$rc" -eq 0 ] && installed && printf '%s' "$out" \| grep -q 'SHA256SUMS.portable'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 92 | `mnemonic-toolkit/scripts/install-verify.test.sh:131` | `if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" \| grep -q 'download failed'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 93 | `mnemonic-toolkit/scripts/install-verify.test.sh:140` | `&& printf '%s' "$out" \| grep -q 'publishes no mk binary for platform'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 94 | `mnemonic-toolkit/scripts/install-verify.test.sh:148` | `&& ! printf '%s' "$out" \| grep -q 'publishes no'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 95 | `mnemonic-toolkit/scripts/install-verify.test.sh:154` | `if grep -v -- '--git' "$T/cargo.log" 2>/dev/null \| grep -q .; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 96 | `mnemonic-toolkit/scripts/install-verify.test.sh:170` | `if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" \| grep -q "REFUSED .*did not print 'mk $MK_VER'"; ...` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 97 | `mnemonic-toolkit/scripts/install-verify.test.sh:177` | `if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" \| grep -q "it printed: (exit 1) mk: .*GLIBC_2.34";...` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 98 | `mnemonic-toolkit/scripts/install-verify.test.sh:213` | `&& printf '%s' "$out" \| grep -q 'cannot create a temporary directory'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 99 | `mnemonic-toolkit/scripts/install-verify.test.sh:224` | `&& printf '%s' "$out" \| grep -q 'cannot create a temporary directory'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 100 | `mnemonic-toolkit/scripts/install-verify.test.sh:235` | `if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" \| grep -q 'temporary directory .* is gone' \` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 101 | `mnemonic-toolkit/scripts/install-verify.test.sh:243` | `if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" \| grep -q 'cannot create a temporary directory'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 102 | `mnemonic-toolkit/scripts/install-verify.test.sh:258` | `if printf '%s' "$n" \| grep -q 'md binary needs glibc >= 2.34; this host has 2.33'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 103 | `mnemonic-toolkit/scripts/install-verify.test.sh:264` | `if printf '%s' "$g" \| grep -q 'install  md (source'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 104 | `mnemonic-toolkit/scripts/install-verify.test.sh:273` | `if printf '%s' "$out" \| grep -q -- 're-run with: --from-source --only mk'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 105 | `mnemonic-toolkit/scripts/install-verify.test.sh:282` | `&& printf '%s' "$out" \| grep -q 'which no cargo record in .* claims (--force)'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 106 | `mnemonic-toolkit/scripts/install-verify.test.sh:298` | `if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" \| grep -q "expected exactly one 'mk'"; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 107 | `mnemonic-toolkit/scripts/install-verify.test.sh:305` | `if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" \| grep -q "did not print 'mk $MK_VER'"; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 108 | `mnemonic-toolkit/scripts/install-verify.test.sh:314` | `if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" \| grep -q "cannot unpack $ASSET"; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 109 | `mnemonic-toolkit/scripts/install-verify.test.sh:382` | `if [ "$rc" -ne 0 ] && [ ! -e "$T/root/bin/mk.exe" ] && printf '%s' "$out" \| grep -q "REFUSED $WZIP: 'mk.ex...` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 110 | `mnemonic-toolkit/scripts/install-verify.test.sh:398` | `if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" \| grep -q 'lists it with more than one digest'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 111 | `mnemonic-toolkit/scripts/install-verify.test.sh:407` | `if [ "$rc" -ne 0 ] && [ -z "$(ls -A "$T/root/bin/mk")" ] && printf '%s' "$out" \| grep -q 'is a directory; ...` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 112 | `mnemonic-toolkit/scripts/install-verify.test.sh:414` | `if [ "$rc" -eq 2 ] && [ "$rc2" -eq 2 ] && printf '%s' "$out" \| grep -q 'non-empty'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 113 | `mnemonic-toolkit/scripts/install-verify.test.sh:437` | `if [ "$rc" -eq 1 ] && printf '%s' "$out" \| grep -q 're-run with --exclude md,mnemonic-gui'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 114 | `mnemonic-toolkit/scripts/install-verify.test.sh:485` | `&& printf '%s' "$out" \| grep -q "belongs to cargo package 'fake-mk 0.1.0 (path+file:///home/user/src/fake-...` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 115 | `mnemonic-toolkit/scripts/install-verify.test.sh:486` | `&& printf '%s' "$out" \| grep -q 'cargo uninstall --root .* fake-mk'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 116 | `mnemonic-toolkit/scripts/install-verify.test.sh:499` | `&& printf '%s' "$out" \| grep -q "belongs to cargo package 'tools 1.0.0"; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 117 | `mnemonic-toolkit/scripts/install-verify.test.sh:506` | `&& printf '%s' "$out" \| grep -q "force given: replacing .*owned by cargo package 'fake-mk"; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 118 | `mnemonic-toolkit/scripts/install-verify.test.sh:528` | `&& printf '%s' "$out" \| grep -q 'could not be read with certainty; not forcing'; then :` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 119 | `mnemonic-toolkit/scripts/install-verify.test.sh:529` | `else bad "malformed .crates.toml '$(printf '%s' "$bad_toml" \| head -n1)': forced or no note: $out / $(cat ...` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 120 | `mnemonic-toolkit/scripts/install-verify.test.sh:552` | `&& printf '%s' "$out" \| grep -q 'which no cargo record in .* claims (--force)'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 121 | `mnemonic-toolkit/scripts/install.sh:273` | `elif ldd --version 2>&1 \| grep -qi musl \|\| ls /lib/ld-musl-* >/dev/null 2>&1; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 122 | `mnemonic-toolkit/scripts/install.sh:474` | `rustc_ver=$(rustc --version 2>/dev/null \| grep -oE '[0-9]+\.[0-9]+\.[0-9]+' \| head -n1)` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 123 | `mnemonic-toolkit/scripts/install.sh:515` | `if wget --help 2>&1 \| grep -q -- '--https-only'; then` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 124 | `mnemonic-toolkit/scripts/install.sh:638` | `_ran=$(printf '%s\n' "$_ran" \| head -n1)` | safe: `#!/bin/sh`, no pipefail anywhere in the file; status is grep's |
| 125 | `mnemonic-toolkit/scripts/install.sh:766` | `if [ ! -r "$_json" ] \|\| grep -Eq "\"$_cb(\\.exe)?\"" "$_json" 2>/dev/null; then` | not a pipe: `||` matched the search pattern |
| 126 | `mnemonic-toolkit/scripts/push-via-staging.sh:154` | `if printf '%s\n' "${CR[@]}" \| awk -F'\t' '$2!="completed"{f=1} END{exit !f}'; then` | safe: awk's `exit` is in END, so awk reads all input |
| 127 | `mnemonic-toolkit/scripts/push-via-staging.sh:213` | `if echo "$OUT" \| grep -qi "bypassed rule violations"; then` | **fix**: bypass detector `echo | grep -qi` in an `if` (fails OPEN); 5 copies kept byte-identical |
| 128 | `seedhammer/cmd/glyphtrace/variants.sh:67` | `awk -v g="$glyph" '$1==g' \| head -1 \|\| true` | safe: masked by `|| true`; awk reads all input; head reads awk's small output |
| 129 | `seedhammer/scripts/chain-mutation-check.sh:129` | `detail="$(sed -n 's/.*is NOT the plate [^ ]* pins: //p' "$work/$name.m2.txt" \| head -1)"` | safe: `set -uo pipefail` with no -e; assignment status never read |
| 130 | `seedhammer/scripts/chain-mutation-check.sh:142` | `dirty="$(git status --porcelain -- "$json" gui/testdata/ \| head -20)"` | safe: `set -uo pipefail` with no -e; assignment status never read |
| 131 | `seedhammer/scripts/oracle-live.sh:104` | `if ! printf '%s\n' "$mint_out" \| grep -q '^=== RUN   TestAssembledMd1MatchesThePrimaryByteForByte$'; then` | **fix**: `printf "$mint_out" | grep -q '^=== RUN ...'`; the match is line 1 of go test output (false red) |
| 132 | `seedhammer/scripts/push-via-staging.sh:154` | `if printf '%s\n' "${CR[@]}" \| awk -F'\t' '$2!="completed"{f=1} END{exit !f}'; then` | safe: awk's `exit` is in END, so awk reads all input |
| 133 | `seedhammer/scripts/push-via-staging.sh:213` | `if echo "$OUT" \| grep -qi "bypassed rule violations"; then` | **fix**: bypass detector `echo | grep -qi` in an `if` (fails OPEN); 5 copies kept byte-identical |

## Fixes: before → after, with reproduction

The harness is `design/evidence/f695-pipefail/repro.py` (engrave `448ef559`).
For each site it first checks that every line of the BEFORE fragment appears in
the file at the origin SHA, and every line of the AFTER fragment appears at
`f695-pipefail`, so it tests the real text and not a copy. It then evaluates both
under the file's own `set` options with a stub producer (`cat` of a fixture):
**big** = 1 MiB with the match on line 1; **absent** = 1 MiB with no match;
**empty**; **prodfail** (command producers only) = prints the match, then exits
1. Each cell counts how many of 10 runs took the TRUE branch.

Fix shapes used:
- Captured variable: `printf '%s' "$x" | grep -q P` → `grep -q P <<<"$x"`.
- Command producer: `cmd | grep -q P` → `{ v="$(cmd)" && grep -q P <<<"$v"; }`.
  The `&&` keeps "a failing producer fails the check", which is what pipefail
  gave the old form.

```
push-via-staging.sh blob on f695-pipefail in 5 repos: IDENTICAL 9bb4c963981a

REPS=10; cells = runs that took the TRUE branch (before -> after)

site                                                             big    absent     empty  prodfail  verdict
engrave release.yml:93 docs-only (-v)                          0->10      0->0    10->10         -  PASS
engrave demo/sh2/rehearse.sh:29                                0->10      0->0      0->0      0->0  PASS
engrave pico2-bootkey-rehearsal.sh:180/208/373 WARNING traps     0->10      0->0      0->0         -  PASS
engrave pico2-bootkey-rehearsal.sh:1105 verified               0->10      0->0      0->0      0->0  PASS
engrave push-master.sh:162 bypass                              0->10      0->0      0->0         -  PASS
push-via-staging.sh:213 bypass (5 copies)                      0->10      0->0      0->0         -  PASS
engrave release-workflows/emit.py:32 (emitted mk smoke)        0->10      0->0      0->0      0->0  PASS
engrave sh2-flash:288 verified                                 0->10      0->0      0->0      0->0  PASS
engrave sh2-flash:305 lsusb                                    0->10      0->0      0->0      0->0  PASS
engrave sign-firmware.sh:101 HASH_ERR                          0->10      0->0      0->0         -  PASS
engrave sign-firmware.sh:180 verified                          0->10      0->0      0->0         -  PASS
engrave scripts/test/run-e2e.sh:66                             0->10      0->0      0->0         -  PASS
dm scripts/gen-compose-golden.sh:34                            0->10      0->0      0->0      0->0  PASS
seedhammer scripts/oracle-live.sh:104                          0->10      0->0      0->0         -  PASS

OVERALL: PASS
```

`OVERALL: PASS` also asserts that the five `push-via-staging.sh` copies are
**byte-identical** on `f695-pipefail` (blob `9bb4c963981a`). The harness can
fail: its first run reported `FAIL` on emit.py because my model of that
two-statement step was wrong (inside an `if`, `-e` is suspended). I corrected
the model; the emitted text did not change. That emitted mk smoke text is also
checked to be **contained verbatim** in mnemonic-key's current `release.yml`
(the 65a526c fix).

**shellcheck** (0.11.0 via nix): every changed shell file shows the same
finding count before and after. pico2-bootkey-rehearsal.sh keeps its 2
pre-existing findings (SC2006 at line 850, SC2010 at the `ls | grep` on line
1133). I added no new finding: SC2015 on my first `A && B || die` shape was
removed by grouping it as `{ A && B; } || die`. The engrave `release.yml`
docs-only `run:` block, extracted, is shellcheck-clean. `bash -n` passes, the
YAML parses and emit.py parses.

## Special care: sh2-flash, pico2-bootkey-rehearsal.sh, sign-firmware.sh

I ran nothing that talks to a device: no flash, no OTP, no picotool, no lsusb,
no sh2-flash, no rehearsal. Every changed condition was run only inside the
harness, with stubs for `picotool`, `lsusb` and `devshell` (sh2-flash's
non-DRY `devshell`, minus nix). The failure direction below is what a
**spurious** pipe failure does on origin today.

| site | today on a spurious failure | change |
|---|---|---|
| sh2-flash:255 `find … \| sort -rn \| head -1` | set -e aborts before flashing: **closed** | none (safe: sort writes a few names in one write) |
| sh2-flash:288 `devshell picotool info -a "$SIGNED" \| grep -q verified \|\| die` | refuses to flash: **closed** | `{ SIGINFO="$(devshell picotool info …)" && grep -q … <<<"$SIGINFO"; } \|\| die` |
| sh2-flash:305 `lsusb \| grep -qi "$BOOTSEL_ID"` | dies "no RP2350 in BOOTSEL": **closed** | `if ! { USB_LIST="$(lsusb 2>/dev/null)" && grep -qi … <<<"$USB_LIST"; }` |
| rehearsal:180 / 208 / 373 `printf "$out" \| grep -qi WARNING && die` | `&& die` is skipped and a redundant-row/ECC warning is parsed as a clean OTP value: **OPEN** | `grep -qi 'WARNING' <<<"$out" && die` |
| rehearsal:187 `printf "$v" \| grep -qE '^[0-9a-f]+$' \|\| die` | die: closed | none (safe: `$v` is a short hex field) |
| rehearsal:266 `printf "$errtxt" \| grep -qi secp256k1` | falls through to 267, then die: closed | none (safe: empty/tiny stderr) |
| rehearsal:267 / sign-firmware:75 `openssl ec -text \| grep -qi secp256k1 \|\| die` | die: closed | none (safe: the matched line is openssl's last output) |
| rehearsal:1105 `picotool info -a … \| grep -qi verified \|\| die` | die: **closed** | `{ IMGINFO="$(picotool info …)" && grep … <<<"$IMGINFO"; } \|\| die` |
| rehearsal:1128 `ls \| grep -v \| head -1 \|\| true` | masked | none (safe) |
| sign-firmware:101 `elif printf "$HASH_ERR" \| grep -qiE 'missing SIGNATURE…'` | goes to `else die`: **closed** | `elif grep -qiE … <<<"$HASH_ERR"` |
| sign-firmware:180 `printf "$INFO" \| grep -qi verified \|\| die` | die: **closed** | `grep -qi … <<<"$INFO" \|\| die` |

Every change keeps behaviour: the same die message and the same exit path. A
producer that exits non-zero still fails the check (the harness's prodfail
column is 0→0). The three WARNING traps are the only fail-OPEN sites in the
irreversible scripts. In practice `picotool otp get` output is a few hundred
bytes, so they could not SIGPIPE today, but the fix removes the dependency on
output size.

## Notes for the controller

- **The fail-OPEN fixes are the ones that matter**: the push-via-staging bypass
  detector (5 copies), push-master.sh, the three OTP WARNING traps,
  gen-compose-golden's refusal, and (only if pipefail is ever added) engrave's
  docs-only detector. The rest are false reds.
- `scripts/release-workflows/emit.py` was still emitting the pre-65a526c mk
  smoke test, so any regeneration would have reintroduced the flake. It now
  emits mk's text byte-for-byte. The generator's MD smoke has separately
  drifted from descriptor-mnemonic's hand-edited `release.yml` (dm has a
  `--from-policy` row the generator lacks). That drift is not F-695 and I did
  not touch it.
- For review scope: the push-via-staging copies must be updated **in all five
  repos together** when these branches land, or the byte-identity breaks.
- Scratch lived in `/scratch/code/shibboleth/f695-scratch/` and was removed with
  `find … -delete`. I did not edit F-695's status in FOLLOWUPS.md.
