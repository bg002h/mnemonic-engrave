# F-687e: macOS stdin identity (wrong wallet), fix, CI and patch releases

Agent-written (Opus 5.5), 2026-09-25.

## Outcome

- The fix is committed in both repos, and `test (macos-latest)` passes on it in both, with the real test running.
- Patch releases are prepared: ms-cli **0.20.1** and mnemonic-toolkit **0.105.1**, with the toolkit installer pins in a separate commit, as last time.
- **Nothing is tagged, merged, or pushed to master.**

| repo | branch | commits |
|---|---|---|
| mnemonic-secret | `f687e-macos-stdin` (also on `ci/f687e-macos-stdin`), worktree `ms-worktrees/f687e` | `150a014e` fix → `8c6535bd` release 0.20.1 |
| mnemonic-toolkit | `f687e-macos-stdin` (also on `ci/f687e-macos-stdin`), worktree `tk-worktrees/f687e` | `f1466f40` fix → `b0801244` release 0.105.1 (pins NOT moved) |
| mnemonic-toolkit | `release-01051-pins` | `1402d547` pins → toolkit 0.105.1 + ms 0.20.1. **This is the commit to tag.** |

Both branches start from origin/master: ms `15ce93a` (the ms-cli-v0.20.0 commit) and toolkit `c61acfc5`.

## Root cause

`path_is_stdin` (F-687 fold 1, M1) decides whether an input path *is* stdin. It does this by comparing `metadata(path)` with `std::fs::metadata("/dev/stdin")`, matching on dev and inode.

- **Linux:** `/dev/stdin` → `/proc/self/fd/0`, a magic link to fd 0's open file. `stat` follows it, so the two agree.
- **macOS:** `/dev/stdin` → `/dev/fd/0` on the **fdesc** filesystem. Stat-ing that node does not report the dev/inode of the file fd 0 is open on, so the check never matched.

The consequence: with stdin redirected FROM a file, and the same file named as an input path, the one-stdin guard let both readers through. The file's bytes then became the passphrase, at exit 0: a wrong wallet.

**Verified, not inferred**, from CI logs:
- toolkit run 36188069667, job `test (macos-latest)`: `silent-payment --secret-file seed.txt --passphrase - < seed.txt` exited 0 and printed an sp address (`cli_f687_passphrase_channels.rs:867`).
- ms master run 36184571116: `ms derive --in card.ms1 --passphrase - < card.ms1` exited 0 with fingerprint `bf05dda1` (that is, the card's text used as the passphrase).
- **After the fix**, the same tests pass on macos-latest (below). The only code change is how fd 0 is identified.

**It was already red, unnoticed, since the F-687 merges.** `test (macos-latest)` on master:

| repo | green through | red since |
|---|---|---|
| ms | `4b029064` (the 10 masters before F-687 were all green) | `d1ab4477`: 5 masters in a row red (d1ab447, f1a1792, b3ea04a, 8a352b7, 15ce93a) |
| toolkit | `b72dbdfa` (the 10 before were all green) | `9da32e2f`: 5 red (9da32e2f, 30afb101, f40212e2, 9fc271b2, 743b9a6c) |

The job isn't a required check, so nothing blocked.

## Fix (identical in both repos, `passphrase_input::path_is_stdin`)

fd 0 is now identified by `fstat` on fd 0 itself: a dup of it, taken with `std::io::stdin().as_fd().try_clone_to_owned()` and then `File::metadata()`. Its dev and inode are compared with `metadata(path)`. It uses std only, no `unsafe`. If fd 0 is closed, the answer is false.

The by-name matches (`/dev/stdin`, `/dev/fd/0`, `/proc/self/fd/0`) are unchanged.

**Other places that stat `/dev/stdin` or `/proc/self/fd/0`:** none. I grepped the `.rs` and `.go` sources of mnemonic-secret, mnemonic-toolkit, mnemonic-engrave (including mnemonic-io-lib), descriptor-mnemonic, mnemonic-key, mnemonic-transaction and mnemonic-gui; the only occurrences were these two functions.

Toolkit callers now fixed on macOS:
- `silent-payment --secret-file`
- `bundle --descriptor-file` / `--import-json`
- `verify-bundle --bundle-json` / `--descriptor-file`
- `import-wallet --blob`
- `electrum-decrypt --decrypt-password-file`

ms caller: `derive --in`.

**Local gates** (Linux) on each branch tip:

| gate | ms | toolkit |
|---|---|---|
| `nextest --workspace --locked` | 646/646, 11 skipped | 4085/4085, 20 skipped |
| clippy 0.1.85 `-D warnings` | clean | clean |
| fmt 1.95.0 `--check` | clean | clean |

Also: ms `vendor-freshness.sh` OK. On the toolkit release commit: Examples regen matches the commit; `make audit` OK; `check_cli_pins` OK; install-verify, msrv-guard, man-step and install-assets OK.

The Linux suite **cannot** catch this defect. On Linux, `stat("/dev/stdin")` and `fstat(0)` agree, so a regression back to the old code stays green there. Only the macOS job detects it; see the proposal below.

## CI (pushed to `ci/f687e-macos-stdin` in each repo; all runs green)

| repo | SHA | workflow runs (all `success`) |
|---|---|---|
| ms | `150a014e` (fix) | rust **36191200092** (macOS job 108256460826), release 36191200129 |
| ms | `8c6535bd` (0.20.1) | rust **36191618677** (macOS job 108257826056), release 36191618550, vendor-freshness 36191618533 |
| toolkit | `f1466f40` (fix) | manual 36191202546, manual-gui 36191202525, quickstart 36191202589, technical-manual 36191202526, examples 36191202552, sibling-pin-check 36191202528, release 36191202554. rust.yml did **not** trigger on this first push of a new ci ref, for no reason I could determine: the ref matches `ci/**` and the commit touches `crates/**`. |
| toolkit | `b0801244` (0.105.1) | rust **36191616051** (macOS job 108257817478), examples 36191615816, manual 36191615843, manual-gui 36191616009, quickstart 36191616079, technical-manual 36191616078, sibling-pin-check 36191615789, release 36191615948 |

**The real test ran, not a skip.** From the macOS job logs:
- ms: `test an_in_path_that_is_stdin_is_a_second_stdin_reader ... ok`, in both runs.
- toolkit: `test a_path_that_is_stdin_is_a_second_stdin_reader ... ok`.

The macOS job uses `cargo test`, which stops at the first failing binary. These runs had no failures: toolkit 237 `test result: ok` blocks and 0 FAILED, so every binary ran.

On both SHAs, every master-required context is green: ms `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)`; toolkit `examples`, `test (ubuntu-latest)`, `clippy`, `manual`, `quickstart`, `technical-manual`, `manual-gui`. So pushing these exact SHAs to master is satisfied, not bypassed.

**Not covered on macOS:** the pty tests (prompt, echo, Ctrl-C, drain) are `#[cfg(target_os = "linux")]`, so they don't run there. `posix_openpt` and `TIOCSCTTY` exist on macOS, so enabling them is a candidate follow-up.

## Release commits

- **ms `8c6535bd`:** `crates/ms-cli/Cargo.toml` + `Cargo.lock` 0.20.0 → 0.20.1; CHANGELOG `[Unreleased]` → `ms-cli [0.20.1] — 2026-09-25`, with a Fixed entry. ms-codec is unchanged at 0.10.0.
- **toolkit `b0801244`:** Cargo.toml + Cargo.lock → 0.105.1; both README toolkit-version markers; `.examples-build/gen.sh` (5 lines); Examples golden regenerated (4 lines, version only); CHANGELOG `[Unreleased]` → `[0.105.1] — 2026-09-25`, with a Fixed entry. **`scripts/install.sh` is not moved**: users fetch it from raw master.
- **toolkit `1402d547` (`release-01051-pins`), the tag commit.** It is gated like last time: manual, quickstart and technical-manual lint + verify-examples OK against ms 0.20.1 built locally; manual-gui `make html` + `make lint` 13/13 against mnemonic-gui-v0.62.0; `check_cli_pins` and sibling-pin-check OK; nextest 4085/4085. **install-assets fails as expected**: the only missing assets are `mnemonic-0.105.1-*` and `ms-0.20.1-*`. It changes:
  - `scripts/install.sh`: `mnemonic-toolkit-v0.105.1`, `ms-cli-v0.20.1`;
  - the ms pin mirrored in `manual.yml`, `quickstart.yml`, `technical-manual.yml`;
  - the Examples golden (the `--list`/`--dry-run` pin lines);
  - `docs/manual-gui/pinned-upstream.toml` `[installer-ahead]` → the two new tags, plus §82's two tag mentions and a manual-gui CHANGELOG line (the GUI stays at v0.62.0);
  - the CHANGELOG "Installer pins" subsection under 0.105.1.

## Exact publish sequence (not executed)

1. **ms master.** `8c6535bd` already carries green required contexts (run 36191618677), so push it directly. It is a fast-forward from `15ce93a`.
   ```sh
   git -C ms-worktrees/f687e push origin 8c6535bd:master   # expect NO "Bypassed" line
   ```
2. **Tag ms and publish.**
   ```sh
   git -C ms-worktrees/f687e tag -a ms-cli-v0.20.1 8c6535bd -m "ms 0.20.1 — macOS: a file that IS stdin (< file with --in file) is refused as a second stdin reader instead of deriving with the file as the passphrase (F-687e)."
   git -C ms-worktrees/f687e push origin ms-cli-v0.20.1
   ```
   Then wait for `release` (and `man-release`) on the tag. Confirm the release carries `ms-0.20.1-{x86_64,aarch64}-linux-musl.tar.gz`, `-macos-{amd64,arm64}.tar.gz`, `-windows-amd64.zip`, `ms-man.tar.gz` and `SHA256SUMS.*`, as 0.20.0 did.
3. **toolkit master.** `b0801244` has green required contexts (run 36191616051); it is a fast-forward from `c61acfc5`.
   ```sh
   git -C tk-worktrees/f687e push origin b0801244:master   # expect NO "Bypassed" line
   ```
4. **Tag the toolkit pin commit and publish.**
   ```sh
   git -C tk-worktrees/f687e tag -a mnemonic-toolkit-v0.105.1 1402d547 -m "mnemonic-toolkit 0.105.1 — macOS: a file that IS stdin is refused as a second stdin reader instead of becoming the passphrase (F-687e); installer pins ms 0.20.1."
   git -C tk-worktrees/f687e push origin mnemonic-toolkit-v0.105.1
   ```
   `install-pin-check.yml` then checks the self-pin against the tag, and `release` publishes the `mnemonic-0.105.1-*` assets.
5. **Merge the pins.** Once both releases' assets exist, open a PR `release-01051-pins` → master (as PR #89 did), let `rust` run, and merge.
   - install-assets and g6 go green only now: g6 reads the ms tag from install.sh.
   - Nothing that moves `scripts/install.sh` may reach master before step 4 completes, because users fetch it from raw master.
6. Clean up: `git push origin --delete ci/f687e-macos-stdin` in both repos. Then delete the `f687e-macos-stdin` / `release-01051-pins` branches after the merge.

## Proposal: make `test (macos-latest)` a required check in both repos

**Recommended, yes**, added to the existing required contexts: ms `test (ubuntu-latest), clippy, test (ms-codec), clippy (ms-codec)`; toolkit `examples, test (ubuntu-latest), clippy, manual, quickstart, technical-manual, manual-gui`. The case for it:
- **It is the only gate for a real class of defect.** This one was a funds-relevant wrong wallet that Linux cannot see by construction; the Linux suite stays green on the buggy code.
- **It is reliable.** Over the last 15 master runs in each repo, the macOS job's only failures are this defect: 10 of 10 green before F-687, 5 of 5 red after. There are no flakes in that window.
- **It is already paid for.** The job runs on every push and PR anyway. Making it required adds no CI minutes; it only makes the red count.

Costs to accept:
- **Wall time.** It is the slowest leg: ms 107–192 s, toolkit 376–642 s. The `ci/staging` push ritual waits for it, so toolkit pushes take about 10 min.
- **Runner-queue delays.** macOS runners occasionally queue. When a queue is stuck, the maintainer can still use the `enforce_admins:false` hatch.
- **One wrinkle in the toolkit.** rust.yml's push trigger is path-filtered, and I saw one ci-ref push not trigger it. Making another rust.yml context required does not change that behaviour, since `test (ubuntu-latest)` is already required from the same workflow.

Scope for the controller: this is a branch-protection change the operator should make or approve. I did not change any protection settings.
