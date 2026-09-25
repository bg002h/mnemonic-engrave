# Brief — ms 0.19.1: argv-admitted secrets trip the "both from stdin" guard

## The defect (controller-reproduced on ms 0.19.0, no GUI)

```
P="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"   # BIP-39 test vector, no funds
MS1=$(ms encode --allow-argv-secret --phrase "$P" | grep -m1 '^ms1')
ms verify --allow-argv-secret --phrase "$P" "$MS1" </dev/null   # exit 1: cannot read both ms1 and --phrase from stdin
printf '%s\n' "$MS1" > card.ms1
ms verify --allow-argv-secret --in card.ms1 --phrase "$P" </dev/null   # exit 0
```

Nothing is read from stdin. Reviewer's diagnosis (verify it, don't trust it):
ms rewrites each argv value admitted by `--allow-argv-secret` to `-` before
dispatch, so `cmd/verify.rs:69-75`'s "both from stdin" check fires. Source:
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-review.md` (I1).

`cmd/derive.rs:432` has the same shape ("cannot read both the entropy source
and --passphrase from stdin"), and `combine.rs` / `hashlock.rs` also handle
`"-"`. **Treat it as a class:** find every verb where an admitted argv secret
can be mistaken for a stdin read, and fix them all.

## Where you work

`/scratch/code/shibboleth/mnemonic-secret`, `master` (clean, = origin). Make a
worktree at `/scratch/code/shibboleth/ms-worktrees/f679-stdin` on branch
`f679-stdin` off `origin/master`. Read the repo's CLAUDE.md first.

## Requirements

- The fix must keep the guard's real purpose: two inputs that BOTH genuinely
  come from stdin must still be refused. Distinguish "the user passed `-`" or
  "the value is read from stdin" from "an argv value that was admitted".
- Tests first (TDD): for each affected verb, a failing test for the argv+argv
  combination, a test that the genuine stdin+stdin case is still refused, and
  the `--in` path still working. Mutation-check the guard (remove it, prove
  the stdin+stdin test goes red and that the line ran).
- This is normative CLI behaviour in the primary Rust repo; any Go port in the
  seedhammer fork that mirrors this guard needs a convergence note, but don't
  edit the fork.
- Release: bump ms-cli to 0.19.1 and write a CHANGELOG entry. **Do not tag,
  push to master, or merge.** Push the branch only if you need CI to run.

## Gates

`cargo nextest run --locked --workspace` (never `--release`), clippy
`-D warnings` with the repo's pinned toolchain, `cargo fmt --check`, and
`ci/repro/vendor-freshness.sh` if Cargo.lock changes (it's a non-required check
that the push ritual won't stop for). Scratch or target dirs go under
`/scratch/code/shibboleth/f679-ms-scratch/`; delete with `find … -delete`.

## Deliverable

Commits on `f679-stdin` ending with
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. **As your final
action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-ms-stdin-impl.md`**:
root cause with evidence, every verb affected and how, tests and mutations,
gates. Return a short summary plus the path.
