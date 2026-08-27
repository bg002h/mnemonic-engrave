# FOLD — R0 round 0 on `IMPLEMENTATION_PLAN_P3_md_mk_mnemonic.md`

**Folder:** independent agent (not the plan's author, not the reviewer).
**Date:** 2026-08-27. **Branch:** `fold/p3`, worktree
`/scratch/code/shibboleth/_work/foldp3/mnemonic-engrave`.
**Report folded:** `design/agent-reports/R0-P3-plan-round0.md` @ `d5a6c45`
(0C / 4I / 6M / 3Nit).

## Gate results

Each run as its own command, exit code read directly, never chained with the
commit:

| gate | exit |
| --- | --- |
| `./scripts/plan-table-check.sh` | **0** |
| `./scripts/plan-cite-check.sh` | **0** — 71/71 resolved, 0 dangling, 0 ambiguous |
| `./scripts/plan-stepref-check.sh` | **0** |

The cite gate cannot see *what* is on a cited line, so every citation added by
this fold was read back off the gate's own output against the claim it supports.

## Disposition

| # | finding | disposition |
| --- | --- | --- |
| I-1 | entry 11's exit-code census misses two thirds of the assertions; two missed sites pin the arm being changed | **FIXED** |
| I-2 | entry 16's "zero GUI test failures" is false; no step can make it true | **FIXED** |
| I-3 | entries 13+14 break 23 committed CI-byte-compared goldens the plan's green cannot see | **FIXED** |
| I-4 | entry 5 covers four verbs, its gate covers three; §1.1's `verify` baseline is wrong | **FIXED** |
| M-1 | §3's E0425 list omits `write`, the module `md`+`mk` adopt | **FIXED** |
| M-2 | F-293 names 2 of 4 trailing-space sites; residue count wrong | **FIXED** |
| M-3 | "`--out` exists on no verb" is wider than its search | **FIXED** |
| M-4 | entry 9's card-boundary justification is falsified by duplicate key records | **FIXED** |
| M-5 | entry 2's control cannot catch header deletion | **FIXED** |
| M-6 | entry 1's RED baseline went green during the review | **FIXED** |
| N-1 | "5 lines, 1 blank" is fixture-dependent | **FIXED** |
| N-2 | the `chunk-set-id:` design count is 35/36, not 34 | **FIXED** |
| N-3 | no closure condition checks that the filed follow-ups landed | **FIXED** |

**13 of 13 landed. Nothing declined.** No finding was folded on the reviewer's
word alone; each was reproduced first, and two were reproduced *against* what
the dispatch brief asserted.

## Where I pushed back, and on whom

### The dispatch brief was wrong about I-1's citations. The report was right.

The brief stated that the report's citations were wrong — that the first cited
line holds a `let valid = &chunks[0];` binding, that the second holds a bare
`1,` inside an `assert_eq!(chunks.len(), 1, …)`, and that **"the named test
`repair_hrp_mismatch_exits_2` does not exist anywhere in
`descriptor-mnemonic`."** *(The brief's two line references are described here
rather than reproduced: written unqualified in `path:line` form they dangle,
because the file they were resolved against is not the file the report cited —
which is the whole finding.)*

That last clause contains the error. The report cited
**`crates/mk-cli/tests/cli_repair.rs`**, which is in **`mnemonic-key`**. The
counter-check was run against `descriptor-mnemonic`'s file of the same name — a
different file in a different repo that happens to share a basename, and whose
lines 152 and 185 really do hold the text the brief quotes.

Measured in `mnemonic-key`:

```
crates/mk-cli/tests/cli_repair.rs:140  fn repair_beyond_t4_capacity_exits_2() {
crates/mk-cli/tests/cli_repair.rs:152      assert_eq!(   ... code, 2
crates/mk-cli/tests/cli_repair.rs:172  fn repair_hrp_mismatch_exits_2() {
crates/mk-cli/tests/cli_repair.rs:185      assert_eq!(   ... code, 2
```

Both tests exist, at the cited lines, asserting what the report said they
assert. The report's only slip is `(:171)` for a test whose `fn` is at `:172`.

**This is the "negatives inherit the search scope" failure**: "does not exist
anywhere in `descriptor-mnemonic`" was true and irrelevant, because the claim
was never about `descriptor-mnemonic`.

### Both prior censuses missed the same third family

The report named the second family `assert_eq!(code, N)` / `assert_eq!(status.code(), Some(N))`
and lumped them; the dispatch brief split out `.code(), Some(N)` and concluded
`mk` had no additional 2s. **There are three distinct families**, and the one
the brief's regex could not see is the one that matters.

Recounted by parsing every tracked `.rs` file under each test directory:

| family | `descriptor-mnemonic` `crates/md-cli/tests` (39 files) | `mnemonic-key` `crates/mk-cli/tests` (17 files) |
| --- | --- | --- |
| `assert_cmd` `.code(N)` | 25 — `{0:3, 1:13, 2:9}` | 12 — `{0:4, 2:4, 5:3, 64:1}` |
| `assert_eq!(out.status.code(), Some(N))` | 18 — `{0:6, 2:2, 4:8, 5:2}` | 6 — `{0:3, 4:1, 5:1, 64:1}` |
| `assert_eq!(code, N)` over a bound `let code` | 14 — `{0:8, 1:1, 2:2, 5:3}` | 16 — `{0:6, 2:2, 4:1, 5:4, 64:3}` |
| **total** | **57** — `{0:17, 1:14, 2:13, 4:8, 5:5}` | **34** — `{0:13, 2:6, 4:2, 5:8, 64:5}` |

**How I counted, and how I checked the count.** A Python pass over
`git ls-files <testdir>` filtered to `.rs`, with three multiline regexes, then a
**residue pass**: every textual occurrence of `.code(` not covered by a matched
span was printed and read by hand. All 32 residue lines were bindings
(`let code = out.status.code()...`) or argument lines feeding a matched
assertion — no fourth family. I also dumped every family-D binding name to
confirm all 30 are the identifier `code`, not an unrelated variable.

**Reconciliation with the two prior counts:**

- The report's `.code(N)` = 12 for `mk` is **exactly right**. Its "24 / 36
  total" over-counts by 2 (mine: 22 / 34), but its **load-bearing** figure — six
  sites asserting 2 — is **exactly right**.
- The dispatch brief's `md` figures for families A and B match mine
  **digit for digit**. Its totals (43 md / 18 mk) omit family D, so its
  conclusions "`md` has 11 sites asserting 2" and "`mk` has 4, and its `Some(N)`
  family contains none" are wrong: **md has 13, mk has 6.**

### The census was then settled by mutation, not by counting

Counting establishes what a grep can see. **Mutation establishes what the suite
can catch**, which is the claim entry 11 actually made. On a scratch copy of
`mnemonic-key` (the subject repo was never written to):

| experiment | result |
| --- | --- |
| naive one-line edit — `crates/mk-cli/src/error.rs:111`, `=> 2` becomes `=> 1` | `cargo nextest run --locked --no-fail-fast`: **337 tests, 335 passed, 2 FAILED** — `repair_beyond_t4_capacity_exits_2` and `repair_hrp_mismatch_exits_2`, exactly the two sites the third family holds |
| the same edit **plus** an `md`-shaped bypass wrapping `crates/mk-cli/src/cmd/repair.rs:113` | **337 tests, 337 passed, 0 failed** |

Under **both** mutations the four `cli_mk1_repair_reverify.rs` funds-safety
tests stayed **green** — an independent confirmation of the dispatch brief's
`SetReassemblyMismatch` result, obtained by execution rather than by reading two
match arms.

So the plan's *"the change would ship unnoticed by the suite in either
direction"* is false in a way that matters twice: the suite does catch it, and
it catches it by reddening tests the plan told the implementer do not exist.

### The §6f ruling the plan did not contain

The report declined to prescribe a remedy — correctly, since a prescribed fix is
not authoritative. I reproduced the defect and ruled it from the source.

The plan said `repair` gains `md`'s bypass *"on an **uncorrectable** input"*.
**`md` does not do that.** `crates/md-cli/src/cmd/repair.rs:124` sits in a bare
`Err(e) =>` arm over the whole correcting decode, and the function's own doc
comment at `:107` says *"On atomic-fail (any md_codec error from
`decode_with_correction`)"*. Measured:

| invocation | `md` | `mk` before | `mk` after |
| --- | --- | --- | --- |
| `repair <HRP swapped to ms1>` | 2 | 2 | 2 |
| `repair <BCH-uncorrectable>` | 2 | 2 | 2 |
| `decode <HRP swapped to ms1>` | 1 | 2 | 1 |
| `decode <garbage>` | 1 | 2 | 1 |

**Ruling folded in: `mk repair` adopts the bypass at `md`'s actual width — any
codec error out of the correcting decode.** The narrow reading passes the
plan's stated gate and reds `repair_hrp_mismatch_exits_2`; the plan now asserts
the HRP-mismatch cell explicitly, which is the only assertion that separates the
two candidate implementations.

**A comment falsified by a diff that never touches it**, folded as work:
`crates/mk-cli/src/cmd/repair.rs:112` says *"route to exit 2 via
`CliError::Codec(_) => 2` in error.rs"*, which stops being true.

### M-2: I disagreed with the report, then found my own regex was wrong

My first count of trailing-space advisory sites returned **0**, against the
report's 4 and the plan's 2. The report was right and I was wrong:
`secret_in_argv_warning`'s **first** argument is the writer and the flag name is
the **second** (`crates/mnemonic-toolkit/src/secret_advisory.rs:40`). My regex
matched a string literal immediately after the paren, which never occurs.

Re-derived from the correct position — 4 sites, matching the report exactly:

```
crates/mnemonic-toolkit/src/cmd/electrum_decrypt.rs:101   "--decrypt-password "
crates/mnemonic-toolkit/src/cmd/import_wallet.rs:507      "--decrypt-password "   <- not in the plan
crates/mnemonic-toolkit/src/cmd/import_wallet.rs:2331     "--decrypt-password "
crates/mnemonic-toolkit/src/cmd/seedqr.rs:157             "--digits "             <- not in the plan, different flag
```

Residue is **44**, not 46. **F-292's figures were re-derived and are correct**:
50 calls under `src/` across 21 files, minus the 2 in `secret_advisory.rs`'s own
unit tests (confirmed by reading them) = 48 call sites across 20 files.

Recorded here because it is the same class of error as the two it corrects: a
regex over a call site that reads the wrong argument, three times in one cycle,
by three different agents.

## The other three Importants, as verified

### I-2 — proven by running the gate, not by reading it

The plan's "zero GUI test failures" rested on *"a pin far behind the CLI's
current version"*, which rests on a **stale in-file comment**
(`tests/schema_mirror_defaults_drift.rs:36`, *"the pinned v0.75.0 binary"*).
Both real pins say `mnemonic-toolkit-v0.97.0` — `pinned-upstream.toml:22` and
the load-bearing dependency pin at `mnemonic-gui/Cargo.toml:76` — and the
measured toolkit is `mnemonic 0.97.0`. **Exactly current.**

Executed the counterexample: a scratch copy of `mnemonic-gui` with the four
`--group-size` defaults in `src/schema/mnemonic.rs` flipped `"5"` → `"0"`, with
`MNEMONIC_BIN` at the pinned-version binary:

```
test md_ms_mk_choices_and_defaults_match_pinned_gui_schema ... ok
test mnemonic_defaults_and_choices_match_pinned_gui_schema ... FAILED
  bundle --group-size :: mirror=Some("0") gui-schema=Some("5")
  convert --group-size :: mirror=Some("0") gui-schema=Some("5")
  ms-shares-split --group-size :: mirror=Some("0") gui-schema=Some("5")
  ms-shares-combine --group-size :: mirror=Some("0") gui-schema=Some("5")
```

Four violations, red. The `md`/`ms`/`mk` test stayed green **in the same run**,
which confirms the plan's other two blindness reasons by execution rather than
leaving them as reading. Independently measured: `md gui-schema` and
`mk gui-schema` are `version: 1` and emit `default_value: null` and
`choices: null`, so the one-sided guard never arms.

**Folded as a new entry**, not a note: the toolkit release + the bump of both
pins, placed strictly *before* the mirror.

### I-3 — reproduced with an independent detector

169 tracked files under `mnemonic-toolkit/docs/manual/transcripts/`, **62
`.cmd`**, and `git grep -l 'secret material on argv'` returns **19** — my list
matches the report's file for file.

For the grouping half I did **not** reuse the report's list. I matched grouped
artifact lines (an HRP-prefixed token followed by ≥2 further whitespace-separated
bech32 groups) across every tracked golden, and got **exactly 4**:
`22-first-bundle.out`, `41-bundle-inheritance-cards.out`,
`cross-format-recipes/recipe-2-bitcoin-core-to-bundle.out`, `qs-23-bundle.out` —
the same four, reached independently.

The two halves are folded with **different remedies**, which the report was
right to separate: the 19 need their `.cmd` files **rewritten** (the commands
must now refuse) and the 4 genuinely regenerate.

### I-4 — measured on all four verbs

`md bytecode -` → **1**, the identical defect at the identical site, in the
plan's work column and in no gate. And bare `md verify -` → **2** from clap's
missing `--template`, reaching 1 only when the template is supplied. Both
folded; closure condition 4 now names four verbs and the `--template`
requirement.

## The parallel-split answer the operator was waiting on

**Three branches run in parallel. Four joins are sequential, in this order:**

```
[md branch]  ─┐
[mk branch]  ─┼─→ toolkit release ─→ GUI pin bump + mirror ─→ journey goldens
[mnemonic branch, INCLUDING its 23 doc transcripts] ─┘            (waits on P2)
```

| what must serialise | why | owner |
| --- | --- | --- |
| **toolkit release + pin bump, BEFORE the GUI mirror** | the drift gate is a lockstep gate against a pinned binary and the mnemonic branch moves one side of it; measured, the mirror alone reds with 4 violations | the join agent, as its own entry preceding the mirror |
| **the GUI mirror, alone on its branch** | one repo, one branch; P2 owns its `ms` schema file. Files disjoint, branch not | one agent, after all three branches are green |
| **the 23 doc transcripts, INSIDE the mnemonic branch** | they live in the toolkit but are invisible to `cargo nextest`; 19 need rewriting, not regenerating | the mnemonic branch, before it calls itself green |
| **the journey goldens, last** | drivers need all three P3 binaries **and** P2's `ms` from `target/release` | one agent, and it waits on P2 |

**The prerequisite is gone.** `crates/mnemonic-io-lib` is on `origin/master` at
`6c24e62` with all three adopted items present, so the three branches start
immediately.

## Follow-ups filed

| id | subject | owning phase |
| --- | --- | --- |
| **F-311** | `mk encode --keys` accepts a duplicate BIP-380 record at exit 0, emitting two byte-identical cards sharing one chunk-set-id | **NOT P3** — a `mk` admission ruling outside this row |
| **F-312** | the stale `v0.75.0` comment in the GUI drift gate, which propagated a false premise into the plan | **P3**, with the toolkit release |
| **F-313** | a runner-shaped definition of green is structurally blind to the toolkit's 62 doc transcripts | ownerless residue |

**F-296 marked DONE** — `plan-cite-check.sh` gained the `mnemonic-gui` root,
leading-dot handling and `.tsv`. The plan's three citation workarounds are
**retired**, and the header now describes the retracted forms **without
reproducing them in `path:line` form**, which is what previously made a gate
flag the prose explaining it.

**F-293 amended in place** from 2 sites to 4, residue 46 → 44, with the
wrong-argument-position cause recorded so the next reader does not repeat it.

## Method

- `cargo build --locked` in `descriptor-mnemonic`, `mnemonic-key` and
  `mnemonic-toolkit` before any behavioural measurement.
- `md` invoked **only** by absolute path (`target/debug/md`); the login shell
  aliases `md` to `mkdir -p`.
- Every exit code captured to a file and read from the file, never through a
  pipe. Mutation runs used `--no-fail-fast` after a fail-fast run truncated at
  71/337 and would have under-reported the blast radius.
- **No subject repo was written to.** Both mutation experiments ran on `cp -a`
  copies under the session scratchpad. Verified afterwards: `descriptor-mnemonic`
  and `mnemonic-gui` are fully clean, and `mnemonic-key` and `mnemonic-toolkit`
  carry **only** the untracked cycle-prep files the plan's own §1 inventory
  already records. **Zero tracked files modified in any of the four.**
- Consults dispatched: **none.** No question required a tie-breaker — every one
  was settled by running something.

## Scope of my negatives

"13 of 13 landed" covers the findings in R0 round 0. This fold did **not**
re-audit the plan for new defects, and three of its claims are inherited rather
than re-derived: §2's boundary verdicts, §10's acceptance measurements, and the
`--from-md1-set` equivalence. The report checked those and found them clean;
I did not re-run them.

**The three assertion families are exhaustive for the two directories I
parsed** — `crates/md-cli/tests` and `crates/mk-cli/tests` — verified by the
residue pass. I did not census `mnemonic-toolkit` (595 sites per the dispatch
brief, unverified by me and not load-bearing for any folded finding).
