# Review: mnemonic-toolkit `docs-green` (642300b7..6ce7e464)

Reviewer: independent (opus), 2026-09-23. Worktree
`/scratch/code/shibboleth/tk-worktrees/docs-green` at `6ce7e464`, left clean.
Scratch: `/scratch/code/shibboleth/.tmp/dgreview/`.

**Question:** after this lands, are the four doc gates real gates, and is the
documentation it adds true?

**Answer:** The documentation is true. Ten flags and sections, including the
three that were red and the verdict prose, match the binaries when run, and every
golden replays green against the tag-built CI tier. The gates are **not yet real**.
`ci/doc-gate-guard.sh` has three constructible false greens, and the flag lint
cannot catch a documented flag that does not exist.

Counts: **0 Critical, 4 Important, 2 Minor, 2 Nit.**

---

## Important

### I-1: the guard's path scopes miss the shared files that technical-manual and manual-gui execute (false green)

`docs/technical-manual/tests/verify-examples.sh` and
`docs/manual-gui/tests/verify-examples.sh` are symlinks to
`../../manual/tests/verify-examples.sh`.
`docs/technical-manual/pandoc/filters/include-transcript.lua` is a symlink into
`docs/manual/pandoc/filters/`. Neither guard regex contains `docs/manual/`:

- technical-manual: `^(docs/technical-manual/|docs/tools/render-mermaid-cache\.py$|\.github/workflows/technical-manual\.yml$|ci/doc-gate-guard\.sh$|crates/|Cargo\.(toml|lock)$)`
- manual-gui: `^(docs/manual-gui/|\.github/workflows/manual-gui\.yml$|ci/doc-gate-guard\.sh$)`

**Counterexample (run).** A commit that changes only
`docs/manual/tests/verify-examples.sh` makes both guards answer
`relevant=false (no gated path changed)`, so both contexts report success.
Only `manual` runs the changed script, and only against manual's transcripts.
For example, if the change breaks the `EXAMPLES_DIR` / cargo-example path that
only the technical manual uses, `technical-manual` still goes green and lands red
on master.

The implementer widened quickstart for exactly this reason (its `transcripts/` is
a symlink), but did not widen the other two books. This hole predates the branch
(the old `paths:` had it too), but the branch's purpose is to make these contexts
required.

**Fix:** add `docs/manual/tests/verify-examples\.sh$` to both regexes, and
`docs/manual/pandoc/filters/` to technical-manual's. Better, derive the list from
`find docs/<book> -type l`.

### I-2: renaming a gated file out of scope is invisible to the guard (false green)

The guard runs `git diff --name-only "$base" HEAD`. Porcelain `git diff` detects
renames by default, so a rename prints only the **new** path.

**Counterexample (run).** `git mv docs/manual/transcripts/22-first-bundle.cmd attic/`
and commit. `git diff --name-status` shows
`R100 docs/manual/transcripts/22-first-bundle.cmd attic/22-first-bundle.cmd`, the
guard lists only `attic/22-first-bundle.cmd`, and it answers `relevant=false`.
`git mv docs/manual attic/manual` would likewise skip the manual gate entirely.

A pure delete is caught, because D rows print the old path. A rename is not. This
contradicts the header's claim that the guard "can run the gate unnecessarily but
never miss a change".

**Fix:** `git diff --no-renames --name-only`.

### I-3: a push diffs against `before`, which is not a gated base (false green through a leftover `ci/staging`)

For a push, the guard uses the event's `before` SHA when it is non-zero. The two-dot
diff is safe only if `before` itself passed the gate, and on `ci/staging` nothing
guarantees that:

- `scripts/push-via-staging.sh` pushes `HEAD:refs/heads/ci/staging` without deleting a
  pre-existing branch first.
- It leaves `ci/staging` in place on the "tip moved" abort, on the "bypass detected"
  abort, and whenever the script is interrupted during its up-to-30-minute wait.
  A 10-minute Bash-tool timeout on an agent-run push is one way to interrupt it.

**Counterexample (run in a scratch bare repo):**

1. Commit C adds an undocumented flag to `42-md.md`, which would turn `manual` red.
2. C is pushed to `ci/staging`, and the window is interrupted, so `ci/staging`
   stays at C.
3. Commit D (touching only `README.md`) goes on top of C, and
   `push-via-staging.sh` runs again. The push is a fast-forward with
   `before` = C.
4. The guard prints `changed files (vs C): README.md` and answers `relevant=false`,
   so `manual` reports success on D.
5. The script sees every required context green and pushes D, which contains C's
   red manual, to master.
6. The master push then runs the full gate and goes red, but only after the
   commit has landed.

The same D pushed to a fresh `ci/staging` (`before` = 000…) answers
`relevant=true`. `strict: false` accepts the SHA's staging-run context.

**Fix:** for every push to a non-default branch, diff against
`origin/<default>`, ignoring `before`. Use `before` only for pushes to the
default branch itself, whose previous tip is gated.

### I-4: the flag lint is one-directional; a documented flag that does not exist passes

The brief expected this mutation to fail. It passed.

**Counterexample (run).** Adding the row ``| `--no-such-flag` | a flag md compose does not have |`` to
the `md compose` flag table gives `make lint` exit **0** with the pinned tier. The
lint checks that each flag in `--help` appears in the doc, never that each flag in
the doc appears in `--help`.

Consequences:

- A flag a sibling release removes or renames stays documented, and the required
  `manual` gate stays green.
- The six mk `--in` rows are documented while CI's pinned `mk-cli-v0.13.0` has no
  `--in` on those verbs (`mk decode --help | grep -c -- --in` gives 0). They are
  honestly annotated "(mk-cli after 0.13.0)", but no gate knows that.

The implementer never claimed the reverse direction (M1–M5 are all help→doc), so
this is a scope gap, not a mis-stated test. It matters because this lint is what
makes the `manual` context a *truth* gate.

**Fix:** for each first-column `` `--flag` `` in a verb section's flag table, require
it in that verb's `--help`. Rows marked with an explicit "after <ver>" token go on
a named allowlist, so the mk rows become a visible, counted exception.

---

## Minor

**M-1: the verdict prose omits the fourth verdict.** 42-md.md says "Each line says
one of three things" (refuses / imports / unproven). md-codec at the md-cli 0.20.2
tag also renders `Verdict::ImportsAltered`: "`{name} {span}: imports the {form} form,
but reads it as {kind}{k-of-n} -- not the wallet as built (...)`"
(`crates/md-codec/src/coordinator/mod.rs:504`). It is reachable today: Liana
8.0/15.0 reads a measured X24 shape as 2-of-4, and Nunchuk reads an unsorted
2-of-3 wsh as MINISCRIPT (tests at `crates/md-codec/tests/coordinator.rs:338,348`).
The tool line is self-explanatory, so this is not blocking, but the "three things"
sentence is false and the funds-relevant case goes unexplained. Add a sentence for
it.

**M-2: the `examples` required context has the same push-filter hole.** The
implementer's report already notes that `examples.yml` is `paths:`-filtered on
`push`, so it cannot be earned by a docs-only staging push. Not this branch's
change. File it.

## Nit

- **N-1:** the `md descriptor` worked example in 42-md.md drops the final stderr
  line `note: stdout is watch-only — public keys only, cannot spend` (verified by
  running it). The other lines match exactly.
- **N-2:** a `[skip ci]` commit message suppresses every workflow, so the now-required
  contexts never appear and `push-via-staging.sh` stops with "NEVER RAN". This is
  fail-closed and deserves one line in the guard header.

---

## Verified clean (no finding)

**Gate structure**
- Job ids and contexts are `manual`, `quickstart`, `technical-manual` and
  `manual-gui`, each unique across `.github/workflows/`. None has a job-level `if`.
- All four trigger on `push` to `[main, master, 'ci/**']`, on `pull_request`, and on
  `workflow_dispatch`, with no `paths:`. Every staging push and every PR therefore
  yields the context, apart from N-2.

**manual-gui aggregate**
- It is `if: always()` with needs `[changes, lint, verify-examples, verify-examples-gui, build]`.
- It requires `changes == success`. When relevant, it requires every gate to be
  `success`; otherwise it requires `skipped`.
- A failed or cancelled gate fails it, and a guard crash fails it through
  `CHANGES`. I found no way for it to succeed while a gate failed.

**Install pins**
- Pins agree across `scripts/install.sh` (md 0.20.2 / ms 0.19.0 / mk 0.13.0),
  manual / quickstart / technical-manual, and cross-tool-differential.
- `sibling-pin-check.yml` couples install.sh to the workflow pins, so an install.sh
  bump must touch a gated workflow file.
- manual-gui's separate GUI-pinned tier is deliberate.

**Lint (pinned tier: tag-built md 0.20.2, ms 0.19.0, mk 0.13.0, plus mnemonic 0.104.0)**
- `make lint` is OK.
- M-A: renaming every bare `--unspendable` in the compose section to
  `--unspendable-key` fails with "flag --unspendable for `md compose` is not
  documented in its section", so the whole-flag match works.
- The compose section carries all 7 flags; `md shape-key` and the verdict output
  are documented.

**Goldens against the tag-built tier**
- manual: `verify-examples` OK, 62 transcripts pass.
- quickstart: OK, 62 pass.
- technical-manual: with `EXAMPLES_DIR`, OK, 18 pass.
- The goldens are what the CI-pinned binaries produce.
- No prose in manual, quickstart, technical-manual or README teaches an argv-secret
  `ms` command (grep over all ms verbs).

**Truth checks (each run)**

| Item | Result |
|---|---|
| `restore --recalibrate-threads` | Matches `resolve_threads`: the stderr line `measuring search threads on this machine`, `[search] threads` in `~/.mnemonic/mt.conf`, fallback on write failure, ~0.6 s/rung. Warm-up 1.2 s and a 10 s tick; `scan complete in` is printed. |
| `ms hashlock --kind ripemd160` | Prints `hash:ripemd160:09e7bb50…946b`, byte-equal to the doc. |
| `ms hashlock`, `--kind` omitted | Prints the sha256 record plus the four-kind table on stderr, and under `--json` `digests_by_kind` and `kind_specified:false`. `--kind SHA256` exits 64 with "never folded". |
| `--phrase-looks-like-digest-ok` | A 40-hex phrase and a 64-hex uppercase phrase both exit 1, and the message names `md compose --path` and `me sysw pack`. With the flag the phrase is hashed, exit 0. |
| ms argv guard | Refuses before the parser, names the argument by position, exit 1. `ms encode --in` of a hex file refuses and points at `--hex - <`. |
| ms combine grouping | `ms combine` has no grouping flags. |
| `ms split --out` | Writes a 0600 file with unbroken shares; stdout keeps only the labels. |
| ms separators | `ms split` / `ms encode` `--separator` accept space, hyphen and comma. mnemonic's convert, ms-shares and `--separator` are whitespace-only, per their help. |
| `md compose` worked examples | Both are byte-exact. The keyless `--experimental` case exits 1 with nothing on stdout, and `--md-only` then prints the template. |
| Verdict ranges | `Liana 8.0-15.0: refuses (NUMS key path)` without `--unspendable liana`. `Bitcoin Core 24.2-25.2: refuses (miniscript under tr)` implies miniscript-in-tr from Core 26.0. The descriptor example's `Bitcoin Core 24.2-28.4: refuses (the <0;1> multipath spelling …)` implies multipath from 29. The zero parent fingerprint in `md descriptor` output is confirmed (xpub6CatW… in, xpub6BemY… out). |
| `md shape-key` | `md1yzpqqxppsgsc8dua4tu0kekyl` gives `wsh(multi(2,…))\|[[]][]\|NotTaproot`, as documented. `--descriptor` accepts md's own output and refuses the wallet form with "does not round-trip chain 0". |
| `md verify --path` | An origin-less `sh(multi)` card refuses with "non-canonical wrapper requires explicit origin for @0" without the flag, and gives `OK` with it. |

The doc does not use the phrase "only by chance" for Nunchuk; the Nunchuk lines it
shows match the tool.

## What would make it ship
Fix I-1, I-2 and I-3 in `ci/doc-gate-guard.sh` and the two regexes (a few lines),
and either add the reverse lint (I-4) or have the controller explicitly re-rank I-4
as a filed follow-up with an owning phase. Then re-run the three guard
counterexamples above: each must answer `relevant=true`.

ready to ship: no
