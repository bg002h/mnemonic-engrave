---
name: push-master
description: Push master (any constellation repo) via the ci/staging ref so the required CI check is satisfied rather than bypassed; why, the manual ritual, and the freeze rule
---

# Pushing `master` via `ci/staging`

Run `scripts/push-via-staging.sh` — it discovers the required contexts from the
live branch-protection rule. The manual ritual and its rationale follow.

- **Push `master` via the `ci/staging` ref, so the required check is SATISFIED
rather than bypassed** (established and verified 2026-08-08). Branch
protection requires the `test (rust + go)` context, and a status check binds to
a **commit SHA**, not a branch — so a commit pushed straight to `master` has no
check when the rule is evaluated, reports "expected", and is bypassed. That
happened five times in a row; it is a chicken-and-egg in the rule, not a lapse.
`strict: false` on the rule is what makes it fixable — GitHub asks only whether
the commit carries a passing context. So let the SHA earn it first:

```sh
git push origin master:refs/heads/ci/staging   # builds this exact SHA
gh run watch <id>                              # wait for test (rust + go)
git push origin master                         # no bypass message = satisfied
git push origin --delete ci/staging
```

**FREEZE `master` FOR THE WHOLE WINDOW — the ritual assumes the tip does not
move.** 2026-08-16: a push agent staged a SHA, and the controller committed
twice while CI ran, so the final push carried a tip two commits past the gated
one. `strict: false` accepted it against the older gated ancestor and printed
"Bypassed rule violations"; two commits reached `origin/master` with zero CI
signal. The agent correctly refused to call that success. **No commits to
`master` between the staging push and the final push** — hold the work, or
re-stage the new tip afterwards and verify it.

`.github/workflows/release.yml` builds `ci/**` for this reason and explains it
at the trigger. `assemble + sign + release` is gated on `refs/tags/v*`, so a
`ci/**` push cannot sign or publish — verified: it reported `skipped`. A push
that prints "Bypassed rule violations" means the staging step was missed.
