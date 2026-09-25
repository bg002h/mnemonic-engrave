# Brief — F-676 implementer (resume after a power outage)

You are the implementer for **F-676** in mnemonic-toolkit. Read the F-676 entry in
`/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md` (search `### F-676`).

## Where you are

Worktree: `/scratch/code/shibboleth/tk-worktrees/f675-677`, branch `f675-f677`.
Its two commits (`327f9a16` F-675, `9b04e874` F-677 toolkit) are finished and
reviewed — do not rewrite them.

On top of them sits **uncommitted, interrupted work** by a previous F-676
implementer whose session died in a power outage at ~03:57 on 2026-09-24,
three minutes into edits. It was never built, tested or gated, and no report
exists. `git diff --stat` shows `scripts/install.sh` (large rewrite),
`docs/manual/src/20-quickstart/21-install.md`, `.github/workflows/rust.yml`,
`scripts/install-msrv-guard.test.sh`, plus untracked `scripts/install-assets.test.sh`.

**Treat that tree as a draft of unknown quality.** Read all of it before
building on it. Keep what is sound, fix or rewrite what is not. You own the result.

## The design it chose (you may keep it)

`install.sh` installs each CLI (`mnemonic`, `md`, `ms`, `mk`, optional
`mnemonic-gui`) from the **prebuilt binary attached to its pinned GitHub
release**, verified against that release's published SHA-256 checksum file;
it refuses a digest mismatch or an asset no checksum covers; `--from-source`
(and platforms with no binary) builds the **same pinned tag** with
`cargo install --locked --git … --tag …`. Never crates.io.

If you conclude this design is wrong, stop and say why in your report rather
than silently switching.

## Facts the controller already measured (2026-09-24, `gh api`)

Assets differ per repo — do not assume one naming scheme:

- mnemonic-toolkit `mnemonic-toolkit-v0.104.0`: `mnemonic-0.104.0-{x86_64,aarch64}-linux-musl.tar.gz`,
  `-macos-{amd64,arm64}.tar.gz`, `-windows-amd64.zip`; `SHA256SUMS.{x86_64,aarch64,portable}`.
- descriptor-mnemonic `descriptor-mnemonic-md-cli-v0.20.2`: `md-0.20.2-aarch64-linux-musl`,
  **`linux-amd64`**, `linux-arm64`, `macos-{amd64,arm64}`, `windows-amd64.zip`;
  `SHA256SUMS.{aarch64,portable}` — **no `SHA256SUMS.x86_64`**.
- mnemonic-secret `ms-cli-v0.19.0` and mnemonic-key `mk-cli-v0.13.0`: same shape as toolkit.
- mnemonic-gui `mnemonic-gui-v0.61.0`: `mnemonic-gui-v0.61.0-{x86_64,aarch64}-linux{,-musl}.tar.gz`,
  `-{x86_64,aarch64}-macos.tar.gz`, `-x86_64-windows.zip`; a single `SHA256SUMS`.

Verify which checksum file actually lists which asset — do not infer it from names.

## Known traps (from this project's memory)

- **The install pin table and the `examples` golden move together.** The golden
  embeds the pin table; regenerate it in the same commit or the required
  `examples` check goes red (happened 3× on 2026-09-19/20).
- The manual's documented versions must equal the installer's pins
  (md 0.20.2, ms 0.19.0, mk 0.13.0, toolkit 0.104.0). Check every version string
  the diff introduces against `gh` — do not trust the draft.
- A tool's printed recipe is an untested claim: run the commands the manual and
  `--help` tell the user to run.
- `/tmp` is a 32 GB tmpfs. Put scratch install roots and target dirs under
  `/scratch/code/shibboleth/tk-worktrees/f676-scratch/`, and delete it when done.
- Use `cargo nextest run --locked`, not `cargo test`. Never `--release` for tests.
- Stage paths explicitly (no `git add -A`). Do not push, tag or merge.

## Gates you must actually execute (a gate never run is not a gate)

1. Every `scripts/install*.test.sh` passes, and each new test **fails** when the
   check it guards is removed (mutate, prove red, restore).
2. **Real end-to-end on this box (x86_64 Linux):** run the installer for real
   into a scratch `--root`, all four CLIs + GUI; each installed binary's
   `--version` equals its pin. Also run `--from-source` for at least one CLI.
3. **Refusal is real:** corrupt one downloaded asset (or point at a wrong digest)
   and show the installer refuses and installs nothing for that CLI.
4. `--dry-run` for every other platform the installer claims (macOS amd64/arm64,
   aarch64 Linux, Windows if claimed) resolves every URL and every checksum file
   to one that exists (HEAD/`gh api`).
5. The toolkit's full validation surface as CI runs it — including the
   `examples` golden and whatever `rust.yml` now runs — green.

## Deliverable

Commit the work on branch `f675-f677` (one commit, or a few with clear
messages; end each message with
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`).

**As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f676-impl.md`**:
what you kept from the draft and what you changed, the commits, each gate above
with its actual command and output, mutation results, and concerns. Then return
only a short summary plus that path.
