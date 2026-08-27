# IMPLEMENTATION — P1 rows 7 through 13

**Agent:** implementer for `IMPLEMENTATION_PLAN_P1_mt_adopts.md`, rows 7–13.
**Date:** 2026-08-27.
**Worktree:** `/scratch/code/shibboleth/_work/p1impl/mnemonic-transaction`,
branch `impl/p1`.
**Baseline:** `a4cdefa` (rows 1–6 built and green, 245 tests).
**Head:** `a0a70a843df084e423e8306184bf42d074b6356b`.
**Nothing was pushed.**

---

## 1. RESULT — all seven rows built, all seven gates green

| row | name | RED first? | state |
| --- | --- | --- | --- |
| 7 | the pin | regression-gated (per plan) | **done** |
| 8 | the purge swap | RED, watched | **done** |
| 9 | the fd adoption | RED on the fifo half, regression-gated on the differential | **done** |
| 10 | the out channel | RED, watched | **done** |
| 11 | the wording | RED, watched | **done** |
| 12 | the decline | regression-gated (per plan) | **done** |
| 13 | the decode warning | RED, watched | **done** |

Plus one unplanned commit: **a gate fix**, between rows 8 and 9. Section 4.

Final gate run, each command run **separately** from every other and from the
commit, exit codes read directly from `$?` and never through a pipe:

```
cargo build --locked                                  0
cargo nextest run --locked                            0
cargo clippy --all-targets --locked -- -D warnings    0
cargo fmt --check                                     0
./scripts/check-refusal-coverage.sh                   0
./scripts/mutate-refusals.sh                          0
./scripts/journeys.sh                                 0
```

The `nextest` line, verbatim:

```
     Summary [  10.532s] 275 tests run: 275 passed, 0 skipped
```

`git diff --stat a4cdefa..HEAD`:

```
 .github/workflows/ci.yml                    |  26 ++
 Cargo.lock                                  |   6 +
 crates/mt-cli/Cargo.toml                    |  11 +
 crates/mt-cli/src/blocks.rs                 |  54 +++-
 crates/mt-cli/src/main.rs                   | 148 ++++++++--
 crates/mt-cli/src/validate.rs               | 403 ++++++++++++++++++++++----
 crates/mt-cli/tests/declined_crate_items.rs | 246 ++++++++++++++++
 crates/mt-cli/tests/decode_verify.rs        | 204 +++++++++++++
 crates/mt-cli/tests/encode.rs               | 304 ++++++++++++++++++++
 crates/mt-cli/tests/history_purge.rs        | 431 ++++++++++++++++++++++++++++
 crates/mt-cli/tests/refusals.rs             | 325 +++++++++++++++++++++
 crates/mt-cli/tests/refusals.toml           |  12 +
 crates/mt-cli/tests/tx_record.rs            |  22 +-
 design/SPEC_mt_v0_1.md                      | 141 ++++++++-
 scripts/journeys.sh                         |  22 ++
 scripts/mutate-refusals.sh                  |  24 +-
 16 files changed, 2272 insertions(+), 107 deletions(-)
```

Commits:

```
a0a70a8 P1 row 13 (the decode warning): F-275 -- WARN, DO NOT REFUSE
b2b9494 P1 row 12 (the decline): what mt does NOT take from the crate, asserted
2effa8b P1 row 11 (the wording): both mode messages are DERIVED FROM THE MODE -- F-260
35266f3 P1 row 10 (the out channel): mt encode gains --out, written by the crate
75c4cdc P1 row 9 (the fd adoption): both mode sites ask the crate what was MEASURED
71fa4bb gate fix: mutate-refusals.sh restored the SOURCE and left a MUTATED BINARY
a0d702e P1 row 8 (the purge swap): mt's §8.2f remedy is the shared crate's, and it RUNS
8ab8251 P1 row 7 (the pin): mt takes mnemonic-io-lib by GitHub rev
```

---

## 2. ROW 7 — the cold dependency resolve, and the method used

The dispatch asked, specifically, which method forced a resolve that a warm
`~/.cargo/git` could not serve.

**A scratch `CARGO_HOME` — an empty directory — plus a scratch
`CARGO_TARGET_DIR`.** Not a targeted deletion of the rev's checkout: an empty
`CARGO_HOME` cannot serve *anything* from a cache, so there is no residue to
argue about.

```
CARGO_HOME=<empty dir> CARGO_TARGET_DIR=<empty dir> cargo build --locked
```

→ **rc 0 in 26 s**, having printed

```
    Updating crates.io index
    Updating git repository `https://github.com/bg002h/mnemonic-engrave`
    Updating git submodule `https://github.com/seedhammer/seedhammer.git`
```

and left `git/db/mnemonic-engrave-4ee619818a86bc36` (52 MB) where there had been
nothing at all.

Independently of the dispatch, before pinning:

```
git ls-tree -d 6c24e628... crates/     -> crates/me-cli AND crates/mnemonic-io-lib
git merge-base --is-ancestor 6c24e628... origin/master  -> true
git rev-parse origin/master            -> 6c24e628... (it IS the tip)
```

**The submodule line is the finding.** Cargo fetches a git dependency's
submodules recursively, so `mt`'s CI now clones `third_party/seedhammer` on every
cold runner and acquires `github.com/seedhammer/seedhammer` as a build-time
dependency. Filed as **F-320**, because it is invisible from `mt`'s side —
nothing in `mnemonic-transaction` mentions seedhammer.

---

## 3. WHAT EACH ROW COST, in one line, with the measurement

- **Row 8.** `purge_command()` DELETED. `mt`'s §8.2f remedy is now
  `mnemonic_io_lib::remedy::history_purge_block`, emitted through `Refusal`'s
  `verbatim` channel rather than its `remedy` channel — because the remedy is
  re-wrapped at 68 columns by splitting on whitespace, which breaks the `sed`
  recipe across lines. The purge **surface** is `mt` plus the verb *only if one
  was typed*, which is `me`'s `argv_surface` rule reflected; the fallback is bare
  `mt` and not `mt encode`, because §8.2f runs before clap and `mt <transaction>`
  leaks a line with no verb in it.
- **Row 9.** Both mode sites take the crate's `fd`; `0o077` stays at `mt`'s call
  sites. The input warning loses its `is_file()` keying, which `mt`'s own source
  had recorded as measured false a few dozen lines below the site that still used
  it.
- **Row 10.** `--out` on `encode` alone, through the crate's `write_private`,
  routed by `channel::destination`. The §8.2h remedy gains `--out` and loses
  *"mt has no --out: stdout IS the strings, by design (§3b)"*.
- **Row 11.** `mode_grants` / `grants_read` / `grants_write` in `validate.rs`;
  both verdict lines and both hazard sentences derive from the observation.
- **Row 12.** `tests/declined_crate_items.rs` — three tests, each mutated to
  prove it notices.
- **Row 13.** `stdout_mode_warning` in `validate.rs`, called from `decode`;
  data inside the document under `--json`, prose otherwise, never suppressed by
  `--quiet`.

---

## 4. THE UNPLANNED COMMIT — a gate that had never run

**This is the most consequential thing found, and no row asked for it.**

`scripts/mutate-refusals.sh` neuters a check, runs one test, restores `src/` from
a byte copy and touches it. It never rebuilt. So the **last** entry's
`cargo nextest` leaves `target/debug/mt` linked from the mutated tree, and
nothing after it relinks: the working tree is clean, `git status` agrees, and
`./target/debug/mt` is a program with a refusal deleted.

Found by walking into it while measuring row 9's differential. The last entry in
`refusals.toml` names `world_readable_stdout_guard`, so straight after a GREEN
gate run:

```
mt encode --in <finalized psbt> > <a 0644 file>  ->  rc 0, 796 bytes, no refusal
mt encode --in <finalized psbt> > <a 0620 file>  ->  rc 0, 796 bytes, no refusal
```

which reads exactly like a shipped §8.2h defect on a tree whose whole suite is
green. About fifteen minutes went into looking for a bug in correct code.

**It was also concealing a real RED.** `mt`'s CI order is refusal-coverage →
refusal-mutation → **journeys**, so `scripts/journeys.sh` had been running
against that mutated binary. Journey A's first line is
`"$MT" encode … >"$WORK/a.out"`, and under the default umask 022 that is a 0644
destination §8.2h correctly refuses; under `set -e` the script dies there.
**The journeys gate had never once run against an unmutated binary** — not
locally and not in CI.

Reproduced rather than reasoned: stashing row 9's source changes, rebuilding at
row 8 and running `./scripts/journeys.sh` gives exit 1 at the same line.

Both fixed. `cleanup` now rebuilds (in the trap, so the FAILED path and an
interrupt are covered; its status discarded so a build failure is not mistaken
for a refusal-test failure), and `journeys.sh` sets `umask 077` — which is the
**first remedy `mt`'s own refusal offers**, so the script does what the tool
tells an operator to do rather than overriding it with `--allow-world-readable`.
All three journeys now run: **60 assertions ok, 0 FAIL**, where before the script
produced two lines of output and stopped.

The class is filed as **F-322**, because `mnemonic-engrave`'s own
`scripts/mutation-run.py` restores from a file copy the same way and likewise
never rebuilds. It is not in CI here, so nothing downstream is currently fooled.

---

## 5. THE ENUMERATED TEST DIFF — §5 condition 10

Every edit to `mt`'s tests, each justified by a named §6 ruling or a numbered
finding. Counts from `grep -c '^#\[test\]'` against `a4cdefa`, not by hand.

| file | before | after | row | justification |
| --- | --- | --- | --- | --- |
| `tests/history_purge.rs` | 0 | 8 | 8 | §6d / §6h; F-264, F-273. 2 of the 8 are harness CONTROLS. |
| `tests/refusals.rs` | 59 | 65 | 9, 11 | +4 for R0-round-0 finding I3 reaching its second site and for the 0620 differential; +2 for F-260 at both mode sites. **No test changed.** |
| `tests/encode.rs` | 17 | 23 | 10 | §6b, `--out`. **No test changed.** |
| `tests/decode_verify.rs` | 43 | 48 | 13 | F-275, the operator's ruling. **No test changed.** |
| `tests/declined_crate_items.rs` | 0 | 3 | 12 | the decline, §2.2 / §2.3 / §2.4 of the plan. |
| `src/validate.rs` (unit) | 0 | 2 | 11 | F-260's clause table, ten modes, strings pinned. |
| `tests/tx_record.rs` | 9 | 9 | 10 | **the one test that CHANGED** — see below. |
| `tests/refusals.toml` | 33 entries | 34 | 9 | +1 §8.2h entry for the 0620 differential. None removed or retargeted. |

**245 → 275. No test was deleted. Exactly one was edited:**

`the_world_readable_refusal_names_the_artifact_this_run_made` dropped the pinned
substrings `stdout IS the record` and `stdout IS the strings`. Those are
fragments of the sentence §6b's ruling retired — *"mt has no --out: stdout IS the
strings, by design (§3b)"* — and a grep of the suite for `--out` does not find
them, so they were located by fragment, exactly as the plan said they would have
to be. Justified by §6b's *"`mt` gains `--out` (I-2)"*, which states in as many
words that `mt`'s mode-0644 refusal text changes and that the test asserting it
changes with it.

**The edit narrowed nothing.** The form-specific noun survives where it is still
form-specific — in the MECHANISM (`This record IS the engraving` vs `These
strings ARE the engraving`) — and each form now *also* asserts the other form's
noun is ABSENT, which it did not before for the strings case. The remedy, which
is the same four lines whatever was emitted, is asserted as naming `--out`.

`refuses_a_world_readable_stdout` is unchanged, as the plan predicted: it asserts
only that the override is named.

---

## 6. MUTATIONS RUN — the gates were checked for vacuity, not assumed

Seven mutations were applied to shipped code and reverted. Each is recorded in
the commit message of the row it belongs to.

| mutation | test that went RED |
| --- | --- |
| purge surface `None => format!("mt {verb}")` | `the_emitted_zsh_recipe_purges_the_line_that_leaked_it` — the recipe ran to completion with the material still in `HISTFILE` |
| purge block through `with_remedy` instead of `with_verbatim` | same test — no intact `zsh:` line to extract |
| `Destination::Terminal` arm returns a refusal | `mt_paints_the_strings_across_a_real_terminal_and_exits_0` |
| the crate's no-records wording swapped in | `every_reading_verb_still_refuses_empty_input_in_mts_own_words` |
| reader made one-record-per-line | `mts_own_reader_still_does_what_split_record_stream_cannot`, on the single-line-blob row |

The two `with_remedy` / fish rows are worth one note: **the fish tests stayed
green under the re-wrap mutation**, because `history clear-session` has nothing
for a re-wrap to damage. The zsh recipe is what makes that claim checkable, and
that is why the `verbatim` comment cites it.

---

## 7. THINGS THE PLAN SAID THAT TURNED OUT DIFFERENT

Stated plainly, because each was a number the plan asserted and the tree
disagreed with. **None of them changed a design decision** — they changed a
figure.

| plan says | measured |
| --- | --- |
| row 9: stdout 0600 "writes **682** bytes" | **796** bytes, against `fixtures/p5_base.json`'s `finalized_psbt_b64`. Pinned as 796, with an instruction to re-measure rather than relax the assertion. |
| row 12: the pty run is "**1198** bytes, rc 0" | rc 0, and a **4264-byte** typescript carrying all 11 `mt1` strings. The typescript SIZE is not asserted — a pty rewrites line endings and the stderr card shares the stream — but every string is. |
| row 13: `mt decode` writes "**679** bytes, rc 0, stderr empty" | rc 0, **445** bytes, stderr empty. The shape is exactly as described; the figure is a different fixture's. |
| row 10 gate: "`--out` suppresses the §8.2h stdout gate entirely, since **`me`** creates the file owner-only" | reads as a typo for `mt`. Built as: `mt` creates the file, so §8.2h has nothing to say. |
| §2.3: `me`'s refusal at `main.rs:1259` | **`:1235`** — re-derived by symbol. |
| row 9: `me`'s `0o044` mask at `main.rs:1117` | **`:1093`** — re-derived by symbol. |

Every `mt` site was located by symbol and re-measured before being quoted, per
F-279's warning. The one that mattered: the plan's `is_file()`-keying citation
now lands on `fn looks_like_a_transaction`, exactly as the anchor warning says.

**One RED was watched and was the wrong RED.** Row 8's `MATERIAL` constant was
first written as 98 hex characters; `looks_like_a_transaction`'s raw-hex arm
requires **100 or more**, so every gate failed with clap echoing the material at
exit 2 — which is the *leak*, not the missing recipe. Corrected to 120, and the
length is now measured in the constant's own doc comment rather than eyeballed.

---

## 8. SPEC EDITS — `mnemonic-transaction`'s copy only

Four sections of `design/SPEC_mt_v0_1.md` were made false by this diff and were
corrected in the same commits: §8.2f (the shell-detected purge command), §8.2g
(the warning's wording), §8.2h twice (the `--out` paragraph, and the `decode`
half). **A fifth was already false before P1 started** — §8.2g still asserted
*"readable by every user on this machine"*, the F-252 reachability claim removed
from the code on 2026-08-25 and never from the paragraph, which `mt`'s own suite
has forbidden since.

**This repository's copy of the same file was NOT edited**, deliberately: it is a
second copy of the same document, four other agents are working here, and an
implementation worktree is the wrong place to decide which copy wins. Filed as
**F-321**, owned by P1's merge.

---

## 9. CI — the trap row 8 walks into, handled

`mnemonic-transaction`'s workflow installed **no shells**. Row 8's tests fail
rather than skip when one is missing — a skipped gate prints ok and exit 0 — so
the required check would have gone red the first time CI saw them.

Added, before `install nextest`, and modelled on the step this repo added an hour
earlier:

```yaml
- name: Install the shells the history-purge gates execute
  run: |
    sudo apt-get update
    sudo apt-get install -y zsh fish
    for b in /usr/bin/zsh /usr/bin/fish /usr/bin/script /usr/bin/timeout; do
      test -x "$b" || { echo "MISSING: $b"; exit 1; }
      echo "ok $b"
    done
    /usr/bin/zsh --version
    /usr/bin/fish --version
```

The **exact paths are asserted**, not the package names, because an install that
succeeds while putting the binary elsewhere fails identically to no install at
all, one CI round later. The YAML was parsed and the step order checked
mechanically, not by reading.

**Fish version skew was respected.** Local fish is 4.8.1, CI's is 3.7.0. The
gate asserts the **invariant** — the session finished, the material is gone, the
neighbour is gone — and never the mechanism. Nothing in `mt`'s suite asserts a
hang or a prompt.

**CI has not run.** Nothing was pushed, so this step is verified only by parsing
and by the local runs. That is the one claim in this report a machine here could
not close.

---

## 10. CONSULTS

**None dispatched.** The one genuine design fork — what string `mt` should hand
`history_purge_block` to match on — was settled by *reading the sibling* rather
than by asking: `me`'s `argv_surface` (`crates/me-cli/src/main.rs:411`) already
solves it, allowlist and bare-command fallback included, and its doc comment
states the reason for both halves. Reproducing that shape was cheaper and more
defensible than a consult, and the two-invocation-shape gate proves the fallback
is load-bearing rather than decorative.

---

## 11. WHAT WAS NOT DONE

- **Nothing was pushed**, and `mnemonic-transaction`'s CI has therefore not run
  on any of this.
- **This repo's `design/SPEC_mt_v0_1.md` was not edited** — F-321.
- **`mnemonic-engrave/scripts/mutation-run.py` was not fixed** — F-322. It is
  outside the dispatch's write scope and is not in CI here.
- **`mt decode --json --quiet` was not fixed** — F-323. It emits no JSON at all,
  pre-existing and measured at `a4cdefa` too; the remedy is a ruling (`--json`
  wins, or the pair is refused) and §6c places `--json` in P2.
- **`F-314`–`F-319` were left free** rather than claimed. The dispatch allocated
  F-301, and F-301–F-304 and F-311–F-313 were taken by other agents while these
  rows were built, so the four filed here are **F-320, F-321, F-322, F-323**.
  Two commit messages (rows 7 and 8) had already cited the stale numbers and were
  rewritten with `git filter-branch --msg-filter`; the trees are byte-identical
  before and after, verified with `git diff <old-head> HEAD` returning empty.

---

## 12. THE ONE THING A REVIEWER SHOULD LOOK AT FIRST

Not the rows. **§4** — a gate that had never run, and the binary it left behind.
Everything in this cycle was measured against `target/debug/mt`, and for about
fifteen minutes that binary was not the program the source described. The habit
that caught it was rebuilding before a behavioural measurement; the habit that
would have prevented it is the one now written into `mutate-refusals.sh`.
