# Release prep: ms-cli 0.20.0 + mnemonic-toolkit 0.105.0

Brief: `design/briefs/release-ms020-tk0105.md`. Nothing pushed, merged or tagged.

## Result

| step | repo / branch | commit | content |
|---|---|---|---|
| (a) | mnemonic-secret `release-f687` | `15ce93a46bf45410056702b96d6f3219067be3ea` | ms-cli 0.19.1 -> 0.20.0, Cargo.lock, CHANGELOG `## ms-cli [0.20.0] — 2026-09-25` |
| (b) | mnemonic-toolkit `release-f687` | `743b9a6ca28c4d625d2615ec27813d024ac0f6c5` | 0.105.0 version, lock, README markers, CHANGELOG, gen.sh + Examples golden, `check_cli_pins.py` `[installer-ahead]` support. **install.sh pins unchanged** |
| (c) | mnemonic-toolkit `release-f687-pins` (child of b) | `af986ee49b615a6cfc98bc7578a946f5bfdc712d` | install.sh self-pin `mnemonic-toolkit-v0.105.0` + `ms-cli-v0.20.0`, ms mirrors in 3 workflows, Examples golden, GUI-manual `[installer-ahead]` declaration + prose, CHANGELOG pin subsection |

Worktrees: `/scratch/code/shibboleth/ms-worktrees/release`, `/scratch/code/shibboleth/tk-worktrees/release` (the brief's worktrees did not exist, so I created them from origin/master: ms `8a352b7`, toolkit `9fc271b2`). The tk worktree is checked out on `release-f687`.

**Tags:** `ms-cli-v0.20.0` -> `15ce93a4`; `mnemonic-toolkit-v0.105.0` -> **`af986ee4` (c), not (b)**.

## Deviation: the toolkit tag goes on (c)

The brief asks for a toolkit release commit (b) without the new pins. But `install-pin-check.yml` runs on every `mnemonic-toolkit-v*` tag push and fails unless the **tagged commit's** `scripts/install.sh` self-pin equals the tag. Its documented recovery is to force-push the tag. So if the tag went on (b), that check would be red for good. I kept the brief's split: (b) goes to master, and the pins stay out of master until the assets exist. The **tag** goes on (c), and it is pushed **before** (c) reaches master. The tag push publishes the self-pinned assets, and only after that does master (which is what users `curl`) name them. Neither repo's release workflow checks whether the tagged commit is on master.

`design/RELEASE_CHECKLIST.md` items 2–3 ("self-pin in the release commit; push; master green; THEN tag") are older than F-676's asset-download installer, and this sequence contradicts them. They should be updated. I did not change them.

## Publish sequence

Master must stay frozen in each repo from its staging push through its final push.

1. **ms master.** Fast-forward `mnemonic-secret` master to `15ce93a4` (a child of origin/master `8a352b7`). Push via `ci/staging` (`scripts/push-via-staging.sh`). Required checks: `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)`.
2. **ms tag.** `git tag -a ms-cli-v0.20.0 15ce93a4` and push the tag. Wait for `release.yml` to publish. Confirm that `ms-0.20.0-{x86_64,aarch64}-linux-musl.tar.gz` (and the other platforms) plus the `SHA256SUMS*` files are attached.
3. **toolkit master = (b).** Fast-forward `mnemonic-toolkit` master to `743b9a6c` (a child of `9fc271b2`). Push via staging. All seven required checks should pass: install.sh still names 0.104.0 / 0.19.1, so `install-assets` passes too. This step does not depend on steps 1–2 and can run in parallel with them.
4. **toolkit tag on (c), tag only.** Once step 2's assets exist: `git tag -a mnemonic-toolkit-v0.105.0 af986ee4`, then `git push origin mnemonic-toolkit-v0.105.0`. **Do not push the branch.** `release.yml` publishes from `cargo test` + build only. `install-pin-check` and `changelog-check` pass on this commit. `rust.yml` also runs on the tag, and its `install.sh harnesses` job **will fail at install-assets** until the release is published. After publication, re-run it with `gh run rerun <id> --failed` (it is not a required check and does not gate the release).
5. **Verify assets.** Run `sh scripts/install-assets.test.sh` at `af986ee4`. It must print `OK`. Before publication it fails on exactly 22 asset fetches, all of them `mnemonic-0.105.0` or `ms-0.20.0`.
6. **toolkit master = (c).** Fast-forward master to `af986ee4` and push via staging. The required `manual`, `quickstart` and `technical-manual` jobs then `cargo install … --tag ms-cli-v0.20.0`, and `examples` checks the new golden. If anything landed on toolkit master after step 3, (c) is no longer a fast-forward and the tag would sit off master. So keep toolkit master frozen from step 3 to step 6.
7. Delete `release-f687` / `release-f687-pins` in both repos. Per the memory note, an ms/toolkit tag can redden mnemonic-engrave CI (`demo/sh2` install block), so check it after the tags.

## Version decisions

- **ms:** only `crates/ms-cli` and `CHANGELOG.md` changed since `ms-cli-v0.19.1` (`git diff --name-only`). ms-codec stays **0.10.0**; only ms-cli is bumped. The CHANGELOG heading follows the repo's convention, `## ms-cli [0.20.0] — 2026-09-25`.
- **toolkit:** only `mnemonic-toolkit` is bumped; wc-codec stays 0.1.1. The heading is `## mnemonic-toolkit [0.105.0] — 2026-09-25`, the form `changelog-check.yml` greps for.
- The (c) CHANGELOG subsection and the GUI-manual CHANGELOG entry describe the pin move. They live in (c), which is the tagged tree, so the release notes match what ships.

## GUI-manual gate: how it stays consistent

`check_cli_pins.py` (F-679) required `pinned-upstream.toml` to **equal** install.sh, with prose checked against install.sh. Moving the installer while GUI v0.62.0 still pins toolkit 0.104.0 / ms 0.19.1 would have forced one of two bad outcomes: the manual documenting CLIs the GUI was not built against, or the installer held back. I did this instead:

- **(b) gate change.** `pinned-upstream.toml` may carry an `[installer-ahead]` table, keyed like `[manual-gui]`. The check accepts an entry only if it equals install.sh's pin exactly and is strictly newer than the manual's tag. It rejects an entry that is stale (the installer equals the manual again), an unknown key, and any move of the GUI tag itself. While any entry exists, the manual must contain exactly one `{#installer-ahead}` section, and none otherwise. Prose is now checked against the manual's own pins, plus the declared newer tag. I ran 15 scratch mutation cases; every one fails with the intended message, and the declared states pass.
- **(c) declaration.** The `[installer-ahead]` table is `toolkit-tag-implied = mnemonic-toolkit-v0.105.0` and `ms-cli-tag-implied = ms-cli-v0.20.0`. A new §82 section, "When the installer is ahead of the GUI", names both releases. §12, §21 and §82 no longer claim the installer installs exactly the GUI's pins. `manual-gui.yml`'s verify-examples tier stays on the GUI's tags. sibling-pin-check does not see it: its grep needs `--git` and `--tag` on one line, and those installs are multi-line. I confirmed this by running the gate's own script.
- **Delete both the table and the §82 section when the GUI re-pins.** The check enforces this: a stale entry, or a section with no table, fails.

**One real behaviour difference for GUI users (documented, not fixed).** GUI v0.62.0 passes a typed passphrase as the separate argv value of `--passphrase` and runs the CLI with `Stdio::null()`. So a passphrase of exactly `-` (or one starting `@env:`) now means stdin (or an environment variable). Measured with abandon×11 about and passphrase `-`: ms 0.19.1 gives `66d564d1`; ms 0.20.0 gives `73c5da0a` (the empty-passphrase wallet) plus `warning: --passphrase from stdin is empty; proceeding with the EMPTY passphrase`. This follows from the reviewed F-687 decision. It is now stated in §82 and in both CHANGELOGs. The GUI's `--version` soft-check is a stub (its label is compiled in), so the GUI runs against the newer CLIs without complaint.

## "--from-source --dry-run capture"

No such capture is committed anywhere. The only committed installer captures are the Examples golden's `--list` and `--no-gui --dry-run` blocks, and both are regenerated. I checked `--no-gui --from-source --dry-run` live at (c): it prints `--tag mnemonic-toolkit-v0.105.0` and `--tag ms-cli-v0.20.0`. The CLI manual (`docs/manual`) names no ms or toolkit pin in prose. Its one `v0.104.0` mention, like gen.sh's depth-2 appendix line ("supported as of v0.104.0"), is history and stays unchanged.

## Gates run

The Rust toolchain is 1.85.0 (the pinned one) unless noted.

- **ms (a):** nextest `--workspace --locked` 646 passed, 11 skipped. clippy `--all-targets -D warnings` clean. fmt clean. `ci/repro/vendor-freshness.sh` OK.
- **toolkit (b):** nextest 4085 passed, 20 skipped. clippy clean. fmt `--check` with 1.95.0 (CI's canonical formatter) clean. Examples regen matches the committed golden (4 lines changed, version only). `docs/manual` `make audit` OK: lint, 62 transcripts, anchor-check at baseline. quickstart and technical-manual lint + verify-examples OK. install-verify / msrv-guard / man-step / install-assets OK. sibling-pin-check 11 OK. check_cli_pins OK. vendor-freshness OK.
- **toolkit (c):** ms is 0.20.0, built from `15ce93a4`. nextest 4085/4085. Examples regen matches the committed golden (pin lines only). `make audit` OK. quickstart and technical-manual OK. manual-gui `make html` + `make lint` all 13 phases OK (upstream root = a `mnemonic-gui-v0.62.0` checkout). check_cli_pins OK (it reports the two installer-ahead pins). sibling-pin-check 11 OK. The `rust.yml` g6 mlock invariant against the ms release commit passes 2/2. vendor-freshness OK. **install-assets fails, as expected**, until publication (see step 5).
- **Doc gates use tag-built siblings.** md, ms and mk were `cargo install --git … --tag`'d at their pins, as CI does. The locally installed `~/.cargo/bin/{md,ms,mk}` are path installs from working trees. The local `mk` reports 0.13.0 but defines flags (`--from-md1-set`) that the tag source lacks, and that made `make audit` fail spuriously.

## Minor, not fixed

- ms `[0.20.0]` has two `### Changed` headings, and toolkit `[0.105.0]` has two `### Changed` and two `### Fixed`. They come from the merged F-687 parts. I did not restructure reviewed text.
- `RELEASE_CHECKLIST.md` is stale on tag order (see above).
