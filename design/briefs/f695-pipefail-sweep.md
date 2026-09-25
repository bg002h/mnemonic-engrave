# Brief — F-695: sweep `producer | grep -q` / `| head` under `pipefail`

Read F-695 in `/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md`.

## The class

Under `set -o pipefail` (or a GitHub Actions step with `shell: bash`, which is
`bash -eo pipefail`), `producer | grep -q PATTERN` can fail on a good
producer: `grep -q` exits at the first match, the producer is still writing,
and it dies of SIGPIPE, so the pipeline's status is non-zero. That gives a
flaky red, or, where the result feeds an `if`/`||`, a gate that takes the
wrong branch. The same applies to `| head -N`, `| grep -m1`, and any
early-exiting reader. Past occurrences: toolkit F-675 (residue check), mk
release smoke test (fixed in `65a526c`).

Safe forms: capture then test (`out="$(producer)"; grep -q PAT <<<"$out"`),
or `grep -q PAT <(producer)` only if you prove it's safe, or read the whole
input (`grep -c`, `awk` without early exit). A pipe is fine when no
`pipefail` applies **and** the pipeline's status is the reader's, but prefer
the capture form in gates anyway: someone will add `pipefail` later.

## Scope

All shell in the constellation's own repos: `scripts/`, `ci/`, `demo/`,
Makefiles, and `run:` blocks in `.github/workflows/*.yml` of
mnemonic-engrave, descriptor-mnemonic, mnemonic-secret, mnemonic-key,
mnemonic-toolkit, mnemonic-gui and the seedhammer fork (its own scripts only,
not upstream code). Enumerate by search and list every instance with a
verdict: **fix**, or **safe** plus the reason (no pipefail and the reader's
status is used; a builtin writing a small string entirely before the reader
can exit; the reader consumes all input). Don't hand-count: the list comes
from the search command, which you include.

## Special care: irreversible-operation scripts

`mnemonic-engrave/scripts/sh2-flash`, `scripts/pico2-bootkey-rehearsal.sh`
and `scripts/sign-firmware.sh` lead to board flashing and OTP writes. For each
instance there:
- say which way it fails today (open or closed) if the pipe spuriously fails;
- make the change minimal and behaviour-preserving;
- **do not run** any command that talks to a device, and don't flash or
  write OTP. Test the changed logic by extracting it into a harness with a
  stubbed producer: output larger than the pipe buffer (≥ 128 KB) with the
  match early, which reproduces the SIGPIPE; the match absent; empty output.

## Tests

For every **fix**, a reproduction that fails before and passes after, e.g. a
producer printing 1 MB with the match on line 1, under `pipefail`. The
push-via-staging copies must stay byte-identical across the 5 repos that carry
it (engrave, descriptor-mnemonic, mnemonic-secret, mnemonic-toolkit, seedhammer).

## Where and how

One branch `f695-pipefail` per repo you change, off origin's default branch,
in a worktree under `/scratch/code/shibboleth/<prefix>-worktrees/f695`.
seedhammer commits need `-s` (DCO) and are authored Brian Goss (that's the
repo's git config; just use `-s`). shellcheck clean. Don't push default
branches, merge or tag. Scratch under `/scratch/code/shibboleth/f695-scratch/`;
delete with `find … -delete`. The shell here is zsh; run bash snippets via
`bash -c` or files, and remember an `rm -rf` guard blocks recursive forced rm.

## Deliverable

Commits ending with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
**As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f695-impl.md`**:
the search command, the full instance table with verdicts, each fix with its
before/after reproduction, and the special-care section. Return a short
summary plus the path.
