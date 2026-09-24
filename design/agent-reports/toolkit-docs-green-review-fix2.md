# Re-review: mnemonic-toolkit `docs-green` fix round 2 (2d71f8d4, 5978eaa2)

Reviewer: independent (sonnet), 2026-09-23. Scope: NEW-1 and NEW-2 from
`design/agent-reports/toolkit-docs-green-review-fix1.md`, plus the four
self-found fixes reported at the tail of `design/agent-reports/toolkit-docs-green-impl.md`
("Fix round 3"). Fix diff: `aafd9d27..5978eaa2` (2 commits: `2d71f8d4` guard,
`5978eaa2` lint). Scratch clones under `/scratch/code/shibboleth/.tmp/dgfix2/`
(bare origin + work clone seeded from `tk-worktrees/docs-green` HEAD `5978eaa2`,
five independent repro clones `repro1`..`repro5`, `mnemonic` release-built from
this tree, pinned md 0.20.2/ms 0.19.0/mk 0.13.0 reused from
`/scratch/code/shibboleth/.tmp/dgreview/root/bin/`, verified by `--version`).
Worktree `tk-worktrees/docs-green` never touched — `git status --short` empty
and HEAD still `5978eaa2` at the end.

**Question:** are NEW-1 and NEW-2 closed, and does round 2's own new code open
a gate that can report green without doing the work?

**Answer:** Both closed, independently reproduced outside the self-test
harness. Attacking the four new mechanisms (base∪head union, `$(TOOLKIT_ROOT)`
resolution, named-flag markers, subcommand completeness) found no live
false-green. Two structural blind spots exist but neither has a real
counterexample against the current repo content, and one is already disclosed
in the module's own docstring.

Counts: **0 Critical, 0 Important, 0 Minor, 2 Nit (informational, non-blocking).**

---

## NEW-1: closed (repro1, repro3)

Fresh clone (`repro1`) from `work`, `git rm docs/manual/tests/verify-examples.sh`,
committed. Ran each of the four workflows' exact `bash ci/doc-gate-guard.sh …`
lines directly (not via `ci/doc-gate-guard.test.sh`), event `push`,
`refs/heads/ci/staging`, `before=000…0`:

| workflow | `relevant` |
|---|---|
| manual | true |
| quickstart | true |
| technical-manual | true |
| manual-gui | true |

All four now answer true (fix1's counterexample had quickstart and
technical-manual false). Confirmed by output line, e.g. technical-manual:
`doc-gate-guard: relevant=true (docs/manual/tests/verify-examples.sh is watched)`.

**Merge-commit attack (repro3, new, not in the implementer's test suite).**
Branch A deletes the shared file; branch B makes an unrelated change; merged
with `--no-ff` into commit M; M pushed to `ci/staging`. All four workflows
answer true. The union's merge-base computation (`merge-base(HEAD,
origin/master)`) resolves correctly through a merge commit because the guard
diffs two trees directly (`git diff --no-renames --name-only base HEAD`), which
does not depend on parent/ancestry structure — a real difference between two
trees is caught regardless of how many parents HEAD has.

## NEW-2: closed (repro2)

Fresh copy of `docs/manual` (`repro2`), row 44 of `44-mk-cli.md`
(`` `--keys <FILE>` `` with a real `(unreleased: mk-cli after 0.13.0: --keys)`
marker) mutated to add a second, fictitious flag to the same cell:
`` `--keys <FILE>`, `--bogus-forever` ``. Ran `docs/manual/tests/lint.sh`
directly against the pinned tier (md 0.20.2 / ms 0.19.0 / mk 0.13.0):

```
[lint] FAIL: flag --bogus-forever is documented for `mk encode` in 44-mk-cli.md
but the pinned binary does not define it (no '(unreleased: mk-cli after
0.13.0: --bogus-forever)' marker naming it)
```

`--keys` is still correctly exempt in the same run. The marker now binds to
the flags it lists, not the row.

## Named-flag markers: a shipped flag falsely marked unreleased (repro4)

Independent of R3 in the self-test: took `--md-only` in `42-md.md` (a real
flag the pinned md 0.20.2 already defines) and added
`(unreleased: md-cli after 0.20.2: --md-only)` to its row. `lint.sh` step 4:

```
[lint] FAIL: stale marker in 42-md.md (md compose): --md-only is defined by
the pinned md-cli 0.20.2; remove it from the marker
```

## Subcommand completeness: reads real binaries, no live evasion found (repro5)

`leaves()` calls `--help` on the actual `$MNEMONIC_BIN`/`$MD_BIN`/`$MS_BIN`/`$MK_BIN`
(the pinned binaries in CI). Independent repro: deleted `md compose` from
`tests/cli-subcommands.list`, ran `lint.sh` directly:

```
[lint] FAIL: `md compose` is a subcommand of the pinned binary but is missing
from cli-subcommands.list
```

**Hidden/aliased subcommand check.** Grepped clap subcommand construction in
all four crates' CLI source (`mnemonic-toolkit`, `descriptor-mnemonic`
md-cli, `mnemonic-secret` ms-cli, `mnemonic-key` mk-cli) for
`hide = true`/`hide(true)` and `.alias(`/`visible_alias`/`alias =`: one hidden
*argument* (`ms-cli derive --phrase-stdin`, an arg not a subcommand) and zero
hidden or aliased *subcommands* exist today. `--help` on all four binaries
enumerated (printed above) matches `cli-subcommands.list`'s scope exactly.
**Mechanism gap, not a live defect:** `leaves()` parses clap's generated
`Commands:` block, which clap omits for a `hide = true` subcommand by design —
so if one existed, the completeness check would never require it to be
documented. No such subcommand exists in the pinned or locally-built binaries
to construct a running counterexample against.

**Nesting depth.** `leaves()` only descends two levels (`bin cmd sub`). Checked
every subcommand that has its own `Commands:` block today (`mnemonic seed-xor`,
`slip39`, `ms-shares`, `seedqr`) for a third level — none has one
(`mnemonic seed-xor split --help` prints no `Commands:` block). Same shape as
above: a real 3-level CLI would silently evade the check, but none exists to
demonstrate it against.

Both logged as Nit, not blocking — same bar the round-3 report itself used for
the lowercase-flag-regex limitation (documented, unexploited by current
content).

## Base∪head union: attacked, fails closed on every constructed case

Read `ci/doc-gate-guard.sh` in full (lines 88-150).

- **First commit of a branch** (`before=000…0`, default-branch push): line
  104-105 decides `true` unconditionally ("default-branch push without a
  previous tip") before the union is ever computed. Covered by the suite's F3
  and reconfirmed in the baseline run below.
- **Force-push:** for a push to *any branch other than the default*, `before`
  is never read at all — the case statement (lines 96-115) only inspects
  `before` inside the `refs/heads/$default|main|master` branch; every other
  ref always computes `base` via `merge_base_with "$default"`. A force-pushed
  `ci/staging` or topic branch cannot desync the base from this. For a
  force-pushed **default** branch, an unfetchable old `before` decides `true`
  (line 108, F4-equivalent, reconfirmed below); a still-fetchable old `before`
  yields a plain two-tree diff against it, which is content-correct
  regardless of ancestry (diff doesn't require a common history, only two
  tree objects) — not exploitable as a miss.
- **Merge commit:** empirically tested (repro3 above) — true on all four
  books.
- **Shallow clone / unfetchable base:** `merge_base_with` unshallows on
  failure and retries; if the branch can't be fetched at all it returns
  non-zero and the caller decides `true` (line 92-93, F5-equivalent,
  reconfirmed below). Every S1-S8 scenario in the suite already exercises the
  non-default-branch path (fetch + merge-base) against a real bare repo, so
  this is not merely a code-reading claim.

**Mutation (required by the brief).** Line 125 of `ci/doc-gate-guard.sh`:
```
-watched=$(python3 "$here/doc-gate-paths.py" --rev "$base" --rev HEAD "${books[@]}")
+watched=$(python3 "$here/doc-gate-paths.py" --rev HEAD "${books[@]}")
```
committed to the scratch `work` clone (`057f993`), then ran
`ci/doc-gate-guard.test.sh` against it: **61 passed, 1 failed** (baseline was
62/0). The one failure: `FAIL S7 quickstart, its imported ../manual/.cspell.json
deleted: want true, got false`. S4/S5/S5b/S6/S8 stayed green under the mutation
— not because they're vacuous, but because their defense (lexical symlink
resolution with no existence check, the other half of the NEW-1 fix) doesn't
need the base tree; only S7's defense (a `../`-reference whose target existed
in the base tree but not in HEAD, so step-3's existence check needs the base
tree specifically) depends on the union. The mutation lands exactly where the
design doc says it should and the suite catches it. `work` was reset back to
`5978eaa2` afterward (`git reset --hard`, verified clean).

## `$(TOOLKIT_ROOT)` resolution: one real gap, not currently live

Grepped every `docs/*/Makefile` for repo-root variable forms. `TOOLKIT_ROOT`
is always defined the same way (`$(abspath $(<BOOK>_DIR)/../..)`) and always
used as either `$(TOOLKIT_ROOT)/literal/path` (caught by the `ROOTED` regex,
confirmed by S8) or as a bare `-C $(TOOLKIT_ROOT)`/`-v $(TOOLKIT_ROOT):/work`
argument with no path suffix (not a file dependency to watch). No
`${TOOLKIT_ROOT}` curly-brace form and no `$(TOOLKIT_ROOT)/$(OTHER_VAR)/…`
nested-variable form exists anywhere in the four Makefiles today (both greps
empty). `WORKSPACE_ROOT` (sibling-repo root) is used only for cross-repo
`MD_BIN`/`MS_BIN`/`MK_BIN` `cargo run` invocations — out of scope for this
repo's gate by construction, matching the round-3 report's own note.

**The gap that would matter:** a Makefile line like
`TOOL := $(TOOLKIT_ROOT)/$(SUBDIR)/x.py` — the `ROOTED` regex requires
`[A-Za-z0-9_.]` immediately after `$(TOOLKIT_ROOT)/`, so a nested `$(...)`
there breaks the match and the dependency would be silently unwatched. This is
the same class of blind spot the derivation's own docstring already names
("What it still CANNOT see is a dependency spelled any other indirect way (a
shell variable, a computed path)"). No such line exists in any of the four
Makefiles as of `5978eaa2`, so there is no live counterexample to reproduce —
logged as Nit, disclosed rather than hidden.

## Test suites (independently run, pinned tier)

- `ci/doc-gate-guard.test.sh`: **62 passed, 0 failed** (matches the
  implementer's and fix1's counts).
- `ci/doc-flag-lint.test.sh` (`mnemonic` release-built from `5978eaa2`,
  `MD_BIN`/`MS_BIN`/`MK_BIN` = pinned md 0.20.2/ms 0.19.0/mk 0.13.0): **12
  passed, 0 failed** (matches the implementer's count).
- Mutation of the union logic: **61 passed, 1 failed** (S7 reddened) — proof
  the test suite is not vacuous for the union mechanism specifically.

---

## What would make it ship

Nothing blocking. The two Nits ($(TOOLKIT_ROOT) nested-variable indirection;
a hidden or 3-level-deep clap subcommand) are real mechanism boundaries but
have no counterexample against the repo's actual content today, and the first
is already disclosed in `ci/doc-gate-paths.py`'s own docstring. Log both as
follow-ups for future optimization; neither blocks this fold.

ready to ship: yes
