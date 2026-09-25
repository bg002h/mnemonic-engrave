# Brief — adversarial review of mnemonic-toolkit branch `f675-f677`

**One question:** can this branch ship? Above all, can the new installer ever
put a binary on a user's machine that is not the one the pinned release
published, while reporting success? Do not audit the rest of the repo.

## What to review

Worktree `/scratch/code/shibboleth/tk-worktrees/f675-677`, branch `f675-f677`,
`git log origin/master..HEAD` — four commits:

- `327f9a16` F-675: vendor-freshness and repro-drift made green; residue-check pipefail race.
- `9b04e874` F-677 (toolkit): `verify-bundle --accept-search-time` help; `examples` check context.
- `8cf7759d` F-676: `install.sh` installs pinned, sha256-verified release binaries, never crates.io.
- `d3f043e7` F-676 fold 1: md pin 0.20.2 → 0.20.3; cli-compiler caveat dropped.

Implementer reports (claims to re-run, not trust), in
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/`:
`toolkit-docs-green-impl.md` (F-675/F-677 origin), `f676-impl.md`, `f676-fold1.md`.
FOLLOWUPS entries F-675, F-676, F-677 in `mnemonic-engrave/design/FOLLOWUPS.md`.

## Settled — do not re-derive

- Design: prebuilt binaries from pinned GitHub releases, checked against the
  release's own SHA256SUMS files; `--from-source` builds the same pinned tag.
  Releases are unsigned, so checksums prove "what the release published", not
  who built it. That limit is accepted and documented; don't re-argue it.
- md 0.20.3 is published with cli-compiler; the controller downloaded and ran it.
- md releases have no x86_64 musl asset (F-678, separate). GUI pins disagree
  (F-679, separate). Out of scope.

## Where to push hardest

1. **The install path's failure states.** Construct inputs that should refuse
   and check they do, installing nothing: a digest mismatch; an asset no sums
   file lists; a sums file listing the asset twice or with a different name;
   a truncated/404 download; an archive with zero, two, or a path-traversing
   binary; a binary whose `--version` isn't the pin; Ctrl-C mid-install; a
   partial install when one of five CLIs fails. Report which leave something
   installed, or report success, when they should not.
2. **Platform detection:** glibc vs musl, macOS arch, Windows, unknown OS. A
   wrong mapping that still passes its checksum is the worst case — it
   installs a working binary for the wrong libc.
3. **Shell safety:** `sh -c "$(curl …)"` means it runs under whatever `sh` is.
   Check POSIX-sh compatibility (dash), quoting, temp-dir creation and cleanup,
   and what `--root` does with odd paths.
4. **Tests that can't fail:** for the install harnesses and the F-675
   residue/repro checks, mutate the guarded line, prove the test goes red *and*
   the line ran, restore. The F-675 report itself found a gate that could not
   fail; look for more.
5. **Docs vs behaviour:** run the commands the manual and `--help` tell a user
   to run. A printed recipe is an untested claim.

## How to run

Put scratch dirs and any `CARGO_TARGET_DIR` under
`/scratch/code/shibboleth/review-f676-scratch/` (not `/tmp`, a small tmpfs).
Delete them at the end with `find … -delete` (an `rm -rf` hook blocks the usual
way). Use `cargo nextest run --locked`, never `--release`. Do not commit, push
or edit the branch; restore any mutation and show `git status` clean. Real
downloads from GitHub are fine; do not install into `~/.cargo/bin`.

## Output

Findings as Critical / Important / Minor / Nit, each with a reproduction.
End with the line `ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f675-677-toolkit-review.md`**
and return only a short summary plus that path.
