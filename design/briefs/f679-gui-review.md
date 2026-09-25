# Brief — adversarial review of mnemonic-gui `f679-pins` (v0.62.0, untagged)

**One question:** can this be tagged as v0.62.0? That means: does the GUI
drive today's CLIs correctly, does nothing it shows or runs disagree with what
the CLI will actually do, and does the new Linux build actually run where it
now claims to? Not a fresh audit of the GUI.

## Inputs

- Worktree `/scratch/code/shibboleth/gui-worktrees/f679`, branch `f679-pins`,
  `git log origin/master..123f09a` (6 commits).
- Implementer report (claims to re-run, not trust):
  `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-impl.md`.
- Release CLIs: install them into a scratch root with
  `sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`
  (mnemonic 0.104.0, md 0.20.3, ms 0.19.0, mk 0.13.0). Do not use
  `~/.cargo/bin/mk`, which is not the release.

## Settled — do not re-derive

- Controller-verified: CI build run 36100903163 and schema-mirror run
  36100903176 both succeeded at `123f09a`; pins name the four release tags.
- **Secret-handling severity rule (operator, 2026-08-27):** a failure to handle
  secret material secretly is never Critical or Important. Report such issues
  as Minor with a reproduction; don't argue them up. The GUI auto-adding
  `--allow-argv-secret` preserves pre-existing behaviour (older CLIs accepted
  secrets on argv), so judge it under this rule. But a GUI that **shows** one
  command and **runs** a different one is a correctness defect, not a
  secret-handling one.
- `origin/master` already fails `cargo fmt --check` in 82 files; don't report that.

## Where to push hardest

1. **Shown vs run:** for every form that can carry a secret, compare the
   confirm dialog's text, the Copy-command output, and the argv actually
   executed (log or intercept it). Any divergence other than the documented
   `--allow-argv-secret` handling is a finding. Can `--allow-argv-secret` ever
   be added to a command that carries no secret, or omitted from one that
   does (so the run fails)?
2. **Separator values:** the report says hyphen and comma were removed for
   mnemonic/md/ms and kept for mk, with a test against real binaries. Check the
   test actually runs each binary (mutate the offered list, prove red).
3. **New form fields:** for a sample of the report's per-surface decisions,
   run the field through the GUI against the release binary and confirm the
   CLI accepts what the GUI builds.
4. **Glibc floor:** the x86_64-linux-gnu asset is now built with `cross`. Build
   (or download the CI artifact of) the release-shaped binary and check
   `objdump -T` for the max GLIBC version. **Then actually start it on an old
   distro** — e.g. `docker`/`podman` isn't available to this user, so try
   `nix run` of an older glibc, a chroot, or report precisely what you could
   and couldn't prove. Also confirm the new CI floor step fails on a binary
   needing a newer glibc.
5. **Mutations:** spot-check 4 of the new tests (including the separator one
   and one tutorial transcript) — break, prove red and that it ran, restore.

## How to run

Rust: `cargo nextest run --locked` (never `--release`), clippy `-D warnings`,
MSRV 1.88.0 build. Scratch and any `CARGO_TARGET_DIR` under
`/scratch/code/shibboleth/review-f679-scratch/`; delete with `find … -delete`
at the end (an `rm -rf` guard blocks recursive forced rm). Do not commit, push,
tag or edit the branch; restore mutations and show `git status` clean.

## Output

Critical / Important / Minor / Nit, each with a reproduction. End with
`ready to tag: yes` or `ready to tag: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-review.md`**
and return only a short summary plus that path.
