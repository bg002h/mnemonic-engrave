# Brief — verification review of F-678 (descriptor-mnemonic `f678-musl`)

**One question:** do the two new checks actually fail when they should, and
can nothing reach a release except on a tag? Do not audit the rest of the repo.

## What to review

Worktree `/scratch/code/shibboleth/dm-worktrees/f678`, branch `f678-musl`,
`git diff origin/main..d2c8488a` (2 files, ~108+/19−; `man-pages.yml` and
`Cross.toml`). Implementer report (claims to re-run, not trust):
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f678-impl.md`.

## Settled — do not re-derive

- Root cause: the musl legs used a two-block `[source]` config with no git
  redirect for miniscript. Controller-verified: dispatch run 36086241898 at
  `d2c8488a` concluded success; both `musl-binary` legs and `repro-x86_64-musl`
  green; `repro-aarch64-musl` skipped because the caller passes
  `run_aarch64: false`. Whether to enable aarch64 repro is an operator decision,
  not a finding.

## Check, and show your evidence for each

1. **The `prep` step fails closed:** feed it a Cargo.lock with an empty rev, and
   one with an extra git source it doesn't redirect. It must exit non-zero.
   Run it locally (extract the script from the YAML).
2. **The dep-info check can fail:** construct dep-info that shows miniscript
   compiled from a git checkout (or not from `vendor/`) and show it goes red;
   show that empty or missing dep-info doesn't pass vacuously.
3. **Release safety:** confirm `ensure-release` and `upload` (and anything else
   that writes to a GitHub release) are still gated on a tag ref, so a
   `workflow_dispatch` or branch push can't publish. Quote the `if:` conditions.
4. **Nothing else regressed:** the tag path still produces the same asset names
   (`md-<ver>-{x86_64,aarch64}-linux-musl.tar.gz`, `SHA256SUMS.{x86_64,aarch64}`,
   `PROVENANCE.*`); `actionlint` is clean.
5. Stale comments: the report says it fixed some and left the
   `vendor-freshness` headers alone. Flag any comment in the diff that is now false.

Scratch work goes under `/scratch/code/shibboleth/review-f678-scratch/`;
delete it at the end with `find … -delete` (an `rm -rf` hook blocks the usual
way). Do not commit, push or edit the branch.

## Output

Critical / Important / Minor / Nit, each with evidence. End with
`ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f678-review.md`**
and return only a short summary plus that path.
