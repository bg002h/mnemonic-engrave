# CORRECTION — the g6 fix is not what I said it was, and the two g6 jobs are ASYMMETRIC

**Date:** 2026-08-15
**Author:** controller
**Status:** nothing was pushed. `mnemonic-toolkit` `master` is unchanged at `c14b1e21`.

## What I claimed, and why it was wrong

I reported that fixing the g6 breach was **easy** — "apply two line-wraps to the
toolkit's `mlock.rs`, one small commit, no tag bump needed" — and that the
exemption comments claiming a frozen pin were stale.

**That was wrong.** I had it backwards, and acting on it would have traded one
red gate for another.

## The actual mechanism: the two g6 jobs do NOT mirror each other

| repo | its g6 job compares its own `mlock.rs` against | how |
| --- | --- | --- |
| `mnemonic-secret` | **`mnemonic-toolkit` @ `master`** | `actions/checkout` with `ref: master` |
| `mnemonic-toolkit` | **`mnemonic-secret` @ a TAG** — currently `ms-cli-v0.14.1` | resolves the canonical ms-cli tag **dynamically from `scripts/install.sh`**; the step is literally named `g6-sibling-master-not-pin` |

The toolkit's side was *deliberately changed* at some point from tracking the
sibling's master to tracking the pinned tag. Its name records the decision.

## Measured consequence

`ms-cli-v0.14.1`'s `mlock.rs` differs from current `ms-cli` master by **exactly
the two hunks `de593ca` introduced** (6 insertions, 2 deletions) — so the pinned
tag is **unformatted**, and toolkit master is **unformatted**, which is why the
toolkit's g6 is **green today**.

Therefore:

| action | `mnemonic-secret` g6 | `mnemonic-toolkit` g6 |
| --- | --- | --- |
| today (after `de593ca`) | **RED** | green |
| format the toolkit (what I proposed) | green | **RED** |
| revert `de593ca` | green | green |

**My proposed fix would have broken the toolkit's g6.** I verified it against
`mnemonic-secret`'s g6 test only — which passed, genuinely, red→green — and never
ran the toolkit's own g6, which uses a different reference entirely.

## Two compounding process failures, both mine

1. **I branched from a stale `master` without fetching.** Local `master` was
   `8051af16` (v0.91.0); real `origin/master` was `c14b1e21` (**v0.97.0**), 11
   commits ahead. I checked `git log --oneline origin/master..master | wc -l`,
   got `0`, and read it as "in sync". **It means "behind or equal."** The
   symmetric check (`master..origin/master`) would have said 11. My
   `origin/master` ref was itself stale because I never fetched.
2. **I read the exemption comment from that stale checkout.** At v0.91.0 it said
   g6 pins the frozen `ms-cli-v0.7.0` tag; I saw the sibling's `ref: master` job,
   concluded the comment was stale, and said so. On **current** master the
   comment reads *"tracks the canonical ms-cli tag read DYNAMICALLY from
   `scripts/install.sh` (field 3; currently `ms-cli-v0.14.1`)"* — accurate, and
   describing the mechanism I then contradicted. **The comment was right and my
   reading of it was stale.** That is the inverse of this cycle's usual failure,
   and it has the same cause: judging a record without re-deriving it.

## The real options

**(A) Revert `de593ca` in `mnemonic-secret`** — one commit, restores `ms-cli`
master's `mlock.rs` to the unformatted form matching both toolkit master and tag
`ms-cli-v0.14.1`. **Both g6 jobs green.** Cost: a local
`cargo +1.95.0 fmt --all --check` will again report that file, which is exactly
what the CI `fmt` job's exemption exists to tolerate. This is the easy fix, and
it is the one I should have proposed.

**(B) Bump the pin — genuinely harder than I said.** It requires, in order:
cutting a **new `ms-cli` release tag** from a 1.95.0-formatted commit; updating
the toolkit's `scripts/install.sh` pin to it (which `sibling-pin-check` also
watches); reformatting the toolkit's `mlock.rs`; then dropping both exemptions.
The tag is a release action — irreversible and publish-adjacent — so this is an
operator decision, not a mechanical fix.

The end state of (B) is still worth wanting: with both copies formatted by one
pinned formatter, the formatter *produces* byte-equality instead of threatening
it, and both exemptions can go. But it is a release-coordination task, not a
two-line change.

## One more thing not to do

While reworking this I removed `examples.yml`'s push-side `paths:` filter, by
analogy with the primaries. **That was also wrong** and has been reverted. That
filter is a deliberate documented ruling — the file states *"push (master/main):
WITH a `paths:` filter. This run is LOAD-BEARING, not redundant — it is the sole
gate for direct-to-master pushes"* — with an authoritative config at
`design/agent-reports/examples-pdf-branch-protection-ruling.md §7` in that repo.
Its PR side stays unfiltered and uses a fail-safe internal guard to no-op
cheaply, so the required context still reports on every PR. Any future `ci/**`
work on the toolkit must read that ruling first.
