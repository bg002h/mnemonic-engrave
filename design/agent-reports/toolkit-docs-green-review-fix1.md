# Re-review: mnemonic-toolkit `docs-green` fix round 1 (6ce7e464..aafd9d27)

Reviewer: independent (sonnet), 2026-09-23. Scratch clones under
`/scratch/code/shibboleth/.tmp/dgfix1/` (bare origin, work clones, a synthetic
CI event JSON, and a scratch `mnemonic` build + a reused pinned-tier CLI root
from the prior review's `/scratch/code/shibboleth/.tmp/dgreview/root/`: md
0.20.2, ms 0.19.0, mk 0.13.0). Worktree at
`/scratch/code/shibboleth/tk-worktrees/docs-green` (HEAD `aafd9d27`) left
clean throughout — verified before and after.

**Scope.** Findings from
`design/agent-reports/toolkit-docs-green-review.md` (I-1..I-4, M-1, N-1, N-2;
M-2 deferred to F-677, not re-reviewed here). The fix diff is
`6ce7e464..aafd9d27` (4 commits: `df2dd32f` guard fixes, `c4bd589a` reverse
lint, `333b6745` M-1/N-1, `aafd9d27` wires the self-tests into CI). Stayed
inside this diff.

**Question:** is each finding closed, and can the doc gates still report green
without doing the work?

**Answer:** I-1, I-2, I-3 and I-4's original counterexample are genuinely
closed — each independently re-executed against a fresh scratch clone,
separate from the implementer's own `ci/doc-gate-guard.test.sh` /
`ci/doc-flag-lint.test.sh` (both of which were also run and both genuinely
exercise the fixed code, confirmed by committing a guard mutation and watching
the self-test redden). But the fix's own mechanism — deriving the watched set
from what still **exists** in the checked-out tree, and exempting a flag by
**row** rather than by **flag** — opens two new false-green paths in the same
class the review just closed. Neither was named in the review.

Counts: **0 Critical, 2 Important (new), 0 Minor, 1 Nit (new).**

---

## Important

### NEW-1: deleting the shared `verify-examples.sh` un-watches it for the two books that only reach it by symlink (false green)

`ci/doc-gate-paths.py`'s symlink resolution only adds a target when
`os.path.exists(t)` is true (`scan_links`, `ci/doc-gate-paths.py:274-278`). A
commit that **deletes** `docs/manual/tests/verify-examples.sh` — the file
`docs/technical-manual/tests/verify-examples.sh` and
`docs/quickstart/tests/verify-examples.sh` symlink to — leaves those symlinks
dangling, and the deleted target is no longer discoverable by the walk: it
drops out of both books' derived watched sets.

**Counterexample (run).** In a scratch clone at `aafd9d27` (origin/master also
seeded at `aafd9d27`, so this isn't an artifact of a stale base): `git rm
docs/manual/tests/verify-examples.sh`, commit, push to `ci/staging`
(`before=000…`). Guard output, all four workflows' exact `--book`/`--also`
lines as written in the workflow files:

| workflow | `relevant` |
|---|---|
| manual | true (docs/manual/ is its own book dir) |
| quickstart | **false** |
| technical-manual | **false** |
| manual-gui | true |

technical-manual's derived set for this commit:
`docs/manual/FOLLOWUPS.md`, `docs/manual/pandoc/filters/include-transcript.lua`,
`docs/technical-manual/` — no `docs/manual/tests/` entry at all. Same for
quickstart. `changed` contains exactly
`docs/manual/tests/verify-examples.sh`, which matches neither the derived set
nor the `--also` ERE, so both report `relevant=false (no watched path
changed)` and skip every heavy step, including the one that would trip over
their own now-broken symlink. This is a required context reporting fast-green
on a commit that breaks the book's own build input.

manual-gui escapes **by accident**, not by design: its derived set includes
`docs/manual/tests/` as a **directory** (not just the file), because
`doc-gate-paths.py`'s reference scanner (step 3) picks up the literal string
`../../manual/tests/` out of a **code comment** in
`docs/manual-gui/tests/verify-examples-gui.sh:9` ("`verify-examples.sh` in
this dir is a SYMLINK to `../../manual/tests/`, shared …") and resolves it to
the still-existing directory. Reword that comment to drop the literal
relative path and manual-gui loses the same protection technical-manual and
quickstart already lack.

This is the same defect class I-1 closed (a cross-book dependency the guard
doesn't watch) with the opposite trigger: I-1 was a **modify**, this is a
**delete**, and the fixed-point derivation needs the dependency to still exist
to discover it — a delete erases the evidence in the same commit that needs
catching. It contradicts the guard header's claim verbatim: "the guard may
only ever err toward RUNNING the gate... can over-include but never miss a
change on the head side" (`ci/doc-gate-guard.sh:24-30`) — a deletion is a
head-side change, and it is missed.

**Fix direction:** `doc-gate-paths.py` should also watch a symlink's
lexical target (the `readlink` string resolved from the symlink's own
directory, without requiring existence) whenever the literal symlink still
exists in the tree, not only a target that currently resolves to something on
disk — a broken symlink is itself evidence of a dependency, arguably a
stronger signal that the gate needs to run.

### NEW-2: the `(unreleased: …)` marker exempts by ROW, not by flag — a second, permanently-nonexistent flag in the same cell rides along forever

`docs/manual/tests/lint.sh`'s reverse-direction check extracts **every**
`--flag` token from a row's first-column cell
(`grep -oE -- '--[a-z][a-z0-9-]+' <<<"$cell"`) and applies **one** marker
verdict to all of them. If any flag in the cell is undefined and the row
carries a valid `(unreleased: <crate> after <X.Y.Z>)` marker, **every**
undefined flag in that cell is counted as exempt — there is no check that the
marker was meant for that specific flag.

**Counterexample (run).** Take the real, correctly-marked row in
`44-mk-cli.md` line 45 (`` `--in <FILE>` | … (unreleased: mk-cli after
0.13.0) ``) and add a second, permanently fictitious flag to the same cell:
`` `--in <FILE>`, `--bogus-forever` ``. `make lint` / `docs/manual/tests/lint.sh`
step 4 against the pinned tier (md 0.20.2 / ms 0.19.0 / mk 0.13.0) prints:

```
[lint] exempt: --bogus-forever in 44-mk-cli.md (mk encode) (unreleased: mk-cli after 0.13.0)
...
[lint] flag-coverage: 11 row exemption(s) for unreleased flags
```

Exit 0. `--bogus-forever` will never be defined by any future mk-cli release
(it doesn't exist), so unlike a real "after 0.13.0" flag, this exemption never
expires and the stale-marker check (which only fires when the pinned binary
**does** define the flag) never catches it. It is indistinguishable in the
output from the 10 legitimate exemptions already there. This is the exact
outcome I-4 was written to close ("a flag a sibling release removes or
renames stays documented, and the required `manual` gate stays green") —
reached by piggybacking on an existing, legitimate marker rather than I-4's
bare unmarked row, which R1 does catch (re-verified below).

**Fix direction:** key the exemption to the specific flag, not the row — e.g.
require one marker per flag (`--flag` `(unreleased: … after …)` pairs), or
require every flag in a multi-flag cell to be independently defined or
individually markered.

---

## Re-verified: the four original findings

All four run independently in a fresh scratch clone (origin.git seeded from
the `docs-green` worktree at `aafd9d27`; a separate work clone per scenario;
`mnemonic` built from `aafd9d27` with the pinned 1.85.0 toolchain;
MD/MS/MK_BIN = the pinned-tag root reused from the prior review), **not**
solely by trusting `ci/doc-gate-guard.test.sh` / `ci/doc-flag-lint.test.sh`:

| finding | independent repro | result |
|---|---|---|
| I-1 | commit touching only `docs/manual/tests/verify-examples.sh`, pushed to fresh `ci/staging` | manual/quickstart/technical-manual/manual-gui **all** `relevant=true` |
| I-2 | `git mv docs/manual/transcripts/22-first-bundle.cmd attic/`, `--no-renames` diff shows both paths | manual, quickstart **both** `relevant=true` |
| I-3 | commit C breaks `42-md.md`, pushed to `ci/staging`; commit D (README-only) on top, `before=C`; run **twice**, once with `origin/master` stale at the pre-branch base and once with `origin/master` already advanced to `aafd9d27` | both cases `relevant=true` — confirmed the fix diffs push-to-non-default-branch against `merge-base(HEAD, origin/master)` and **ignores `before` entirely** off the default branch, closing the class, not just the literal scenario |
| I-4 | `docs/manual/tests/lint.sh` run directly (not via the wrapper) on a copy of `docs/manual` with the reviewer's exact `--no-such-flag` row added to `md compose` | `[lint] FAIL: flag --no-such-flag is documented for \`md compose\` in 42-md.md but the pinned binary does not define it` |

## Test suites

- `ci/doc-gate-guard.test.sh`: **44 passed, 0 failed**, matching the
  implementer's report. Worktree left clean.
- `ci/doc-flag-lint.test.sh` (pinned tier, `mnemonic` rebuilt from `aafd9d27`):
  **7 passed, 0 failed**.
- **Mutation-killed the guard test.** First attempt at forcing
  `ci/doc-gate-guard.sh` to always answer `relevant=false` inserted the
  `decide false …` call *before* the `decide()` function was defined —
  `bash`'s sequential execution meant the call was a silent
  `command not found` under `set -uo pipefail` (no `-e`), so the script fell
  through to its real logic and the test suite reported 44/0 unchanged. This
  was **my own construction error**, not a defect in the product; noted so
  the "mutant had zero effect" false alarm doesn't get mistaken for a finding.
  Re-placed the same `decide false "MUTATED"` line immediately after the
  `decide()` definition, committed it (the test seeds its synthetic origin
  from `git … HEAD`, so an uncommitted edit is invisible to it — this also
  needs to be committed to be picked up, which is itself worth knowing if this
  test is ever hand-run against a dirty tree), and reran:
  **26 passed, 18 failed** — every scenario that must answer `true` (S1, S3,
  P1, P2, F1-F5) correctly reddened. The self-test is not vacuous.

## Marker robustness (task item 4)

Beyond NEW-2 above: a flag spelled with any non-lowercase character (e.g.
`` `--No-Such-Flag` ``) is invisible to **both** lint directions, because the
flag-token regex (`--[a-z][a-z0-9-]+`) requires a lowercase letter
immediately after `--` — the same regex the **forward** direction has always
used to read `--help` output, so it is a pre-existing, symmetric limitation,
not something round 1 introduced. Every real flag across md/ms/mk/mnemonic is
lowercase-hyphenated (clap convention), so this isn't a plausible authoring
mistake the way a bare `--no-such-flag` is. **Nit, not blocking** — log as a
follow-up (case-fold the regex, or flag any non-matching token in a flag cell
as its own lint error) rather than fixing inline.

## Reachability (task item 5)

Read `.github/workflows/manual.yml` in full. The job `manual` has no
job-level `if:`. Its steps in order: Checkout (unconditional) → "Self-test the
doc-gate guard" (**no `if:`**, runs before `id: guard` even exists) → "Detect
relevant changes" (`id: guard`, no `if:`) → every heavy step gated on
`steps.guard.outputs.relevant == 'true'`. No `continue-on-error:` anywhere in
the file. The guard self-test step therefore runs, and can fail the required
`manual` context, on every push (master/main/`ci/**`), every tag, every PR and
every `workflow_dispatch`.

`ci/doc-flag-lint.test.sh`'s step **is** behind
`steps.guard.outputs.relevant == 'true'`, by design (it tests the lint, which
needs the pinned CLIs built — expensive — and only the lint's own inputs, not
the guard's correctness). Checked the stated rationale rather than taking it
on faith: the CLI version pins (`mk-cli-v0.13.0`, `descriptor-mnemonic-md-cli-v0.20.2`,
`ms-cli-v0.19.0`) are literal strings inside `.github/workflows/manual.yml`
itself (`grep` confirmed, lines 110/118/123), which is in `manual`'s `--also`
ERE — so a pin bump alone still flips the guard to `relevant=true` and reruns
the flag-lint self-test against the new pin. Holds.

---

## What would make it ship

Fix NEW-1 (watch a symlink's lexical target even when the target no longer
exists — or, more simply, always add `docs/manual/tests/` and
`docs/manual/pandoc/filters/` to technical-manual's and quickstart's `--also`
directly, sidestepping existence-dependent derivation for the two files that
matter) and NEW-2 (key the marker to the flag, not the row). Both are the same
size of fix as the ones this round already made — a few lines each in
`ci/doc-gate-paths.py` / `docs/manual/tests/lint.sh`. Then re-run this
report's two counterexamples: both must flip to the safe answer.

ready to ship: no
