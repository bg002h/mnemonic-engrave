# IMPLEMENTATION LOG — P0, `mnemonic-io-lib`

Executed 2026-08-27 on `impl/p0`, branch point `0c31395`.
Plan: `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`.

**Baseline:** `388 tests run: 388 passed, 1 skipped`.
**Final:** `423 tests run: 423 passed, 1 skipped`. `cargo clippy --all-targets --locked`: 0 warnings.
**Rows 1–10 complete. Row 11 (`cargo publish`) NOT run** — irreversible and operator-gated.

---

## Rows, in the table's order

| # | row | gate result |
| --- | --- | --- |
| 1 | `no_records_guard -> Result<Vec<String>, String>` | `EXIT_*` refs inside it **1 → 0**; suite unchanged at 388 |
| 2 | move the five + the stub into `me`'s lib half | both greps 0; **a real pty test that CAN fail**, mutation-verified |
| 3 | `fd.rs` — SPLIT the mask | RED first on the `0o620` row; `me`'s behaviour unchanged at 0600/0620/0644//dev/null |
| 4 | `observation.rs` — `PayloadKind` + the pty assertion | RED first (F-259 reproduced); mutation-checked **both** directions |
| 5 | `remedy.rs` | the emitted recipe RUN under interactive zsh actually purges; 3 mutations RED |
| 6 | the pre-parser argv guard | **450 generated rows, 0 leaking**; 6 mutations RED |
| 7 | F-265 — the digit at all five sites | all five mutated 2→3, all five turn the suite RED |
| 8 | `--expect <kinds>` | 4 design rulings, 4 mutations, 4 REDs; every refusal pins digit 4 |
| 9 | `exit.rs` then `channel.rs`; `-` implemented | spliced container **byte-identical** to the explicit form; 4 mutations RED |
| 9b | CREATE `mnemonic-io-lib` | builds + tests standalone; 0 `EXIT_`/`Class` on code lines; **empty dependency table** |
| 10 | `me` consumes the crate | **0 pre-existing tests lost**; 388 + 35 = 423 |
| 11 | `cargo publish 0.1.0` | **NOT RUN.** Stopped, as instructed. |

---

## What failed first, and for the stated reason

Several rows had gates the plan said fail today. Each was confirmed failing
before being fixed — the plan documents seven defects from its own history where
that step was skipped.

- **Row 3.** `0o620 & 0o044 == 0`, so a masked implementation returns `None`:
  `left: None, right: Some(400)`. The 0644 row **passed** at that moment and
  proves only that something is returned; the 0620 row is the gate.
- **Row 4.** F-259 reproduced on a pty: exit 2, *"this payload is BEARER"*, for
  a 65,536-byte zeros image. Its positive control passed at the same moment, so
  the RED was the defect and not the harness.
- **Row 6.** 225 of 450 cross-product rows leaked with the guard removed.
- **Row 9.** `printf 'text:6162\n' | me sysw pack --out b.bin - text:6869` →
  exit 4, nothing written.

Three pieces of work are **regression-gated rather than RED-first**, and the
plan says so: the signature change, the crate adoption, and the dash
differential's unchanged-elsewhere half.

---

## Mutation matrix — every gate proven able to fail

Each mutation was applied to the shipped code, run against the suite, and
reverted.

| mutation | result |
| --- | --- |
| row 2: `refuse_write_block` Terminal arm 2→3 | 389/390, only the new pty test fails — **the 388 pre-existing stayed green**, reproducing F-265 site 1 |
| row 3: mask left inside `fd.rs` | the 0620 assertion fails |
| row 4A: `Terminal(_)` + hard-coded `Bearer` | `a_wipe_image_is_never_called_bearer` fails, 393 others green |
| row 4B: delete the BEARER label | `a_real_container_is_still_called_bearer` + the terminal test fail |
| row 5A: shipped `sed -i` recipe | the purge gate fails — **F-264 reproduced by the gate** |
| row 5B: OFFER `history -d` | the never-offered gate fails |
| row 5C: DROP the `history -d` warning | the same gate fails, for the opposite reason |
| row 6 M1: guard removed | **225/450 leak** (75 canonical, 75 leading-space, 75 UPPERCASE) |
| row 6 M2: no trim+lowercase | **90/450 leak — 0 canonical**, exactly the plan's prediction |
| row 6 M3: no `=` split | the `--in=X` rows leak |
| row 6 M4: guard after `Cli::parse()` | ordering + cross-product + override fail |
| row 6 M5: refuse-everything guard | **the positive controls fail** |
| row 6 M6: override honoured everywhere | the scope test fails |
| row 6 F-270: raw token back into `classify` | the post-parse unit test fails |
| row 7 ×5: each site 2→3 | 3 FAIL, then 1 FAIL each for the other four |
| row 8: `Class`-keyed card kinds | the cosigner gate fails (N-C1) |
| row 8: drop `Admission` | the false-refusal gate fails (C-2) |
| row 8: drop walk 3 / drop walk 2 | the mt1 / md1 completeness gates fail |
| row 9: accept `-` and drop it | the byte-equality gate fails |
| row 9: APPEND instead of splice | same count, same `pub_len`, **caught only by byte equality** |
| row 9: pack the rest on empty stdin | the R7 gate fails |
| row 9: stop re-tightening `--out` | `out_tightens_a_preexisting_world_readable_target` fails |

**One mutation silently did not apply.** Row 6's `=`-split mutation was written
inline and the shell ate the quotes around `'='`; it reported *5 passed*. A
mutation that did not apply is not a result. It was re-run from a file, with an
assertion that the replacement matched exactly once, and every later mutation
used that shape.

---

## Consults dispatched, and what each decided

Three, all `fable`, all persisted before being folded.

1. **`CONSULT-P0-row4-f259-refusal.md`** — the exit digit and message for
   `me sysw wipe` refusing a terminal. **Verdict: digit 2**, with a 571-byte
   replacement message folded verbatim. It rejected 0 and 3 with reasons,
   **dropped** *"terminal sessions are often logged"* rather than rephrasing it
   (logging is a secrecy rationale, and keeping it would re-create the defect),
   and elided the fill as `--fill ...`. Machine-checked before folding: `--fill`
   really does default to `random`, `REGION_ADDR` is `0x10D0_0000`,
   `REGION_LEN` is `65_536`.
   *It independently reached the same fill-elision conclusion I had, which I
   found by running the other two fills rather than by reading the text.*

2. **`CONSULT-P0-scope-followups.md`** — my brief forbids fixing F-264, F-265,
   F-266, F-267, F-268 and F-270, but the plan's §4 schedules four of those six
   as row content and §6 makes P0 unclosable without three of them.
   **Verdict: BUILD F-264/F-265/F-266/F-270; leave F-267 and F-268.** The list
   means *"FOLLOWUPS.md is not your work queue — build the rows, nothing beyond
   them."* Its decisive citation, which I had not read and machine-checked
   afterwards, is in F-266's own entry: *"OPERATOR RULING 2026-08-27: deferred,
   not fixed now"* … *"It is still what condition 8 is FOR, and P0 fixes it as a
   side effect"* … *"deferred in the sense of not interrupting the cycle, not in
   the sense of unowned."*

3. **`CONSULT-P0-row6-seal-surface.md`** — the pre-parser guard breaks
   `me seal`, whose positional is a documented *"FIXTURES AND TESTS only"*
   channel. **Verdict: guard `seal` AND declare `--allow-argv-secret` on it**,
   so the channel is gated rather than deleted. Its load-bearing claim held:
   `me seal --in <ms1>` really did echo the secret to stderr on the pre-guard
   binary, so exempting `seal` would have shipped a live leak.
   **Its `me hash` claim did not hold**, and I measured rather than folding it —
   see below.

---

## Where the plan or a consult was wrong, and how it was caught

- **The plan's prescribed F-264 fix does not work.** §6 condition 9 and F-264
  both specify *flush, edit, reload* — `fc -W`, `sed -i`, `fc -R`. Measured on a
  real pty under stock zsh 5.9.2, that still leaves the secret on disk, because
  `fc -R` **appends** the file to the in-memory list rather than replacing it.
  The shipped recipe zeroes `HISTSIZE` to empty memory, restores the operator's
  own value, then re-reads. **bash had the identical defect** and needed the
  identical shape. *Prescribed fixes are not authoritative — reproduce the
  defect, not the remedy.*
- **The plan's argv-guard surface list is short by two.** `me` has five
  top-level subcommands; the plan enumerates neither `seal` nor `hash`. `seal`
  was not cosmetic: the `--in` shape leaked. Filed and closed as **F-272**.
  Notably the POSITIONAL shape on `seal` is clean — `seal` accepts it, so clap
  never errors — which is exactly why a hand-written surface list missed it:
  **the shape that leaks is not the shape that looks dangerous.**
- **The consult's `me hash` claim was false.** It predicted the guard would
  break `hash` *"worse than `seal`"* because its positional carries `tx:`/`mt1`
  legitimately. Measured against the pre-guard binary, `me hash` already refuses
  **all five** argv-forbidden classes with its own messages, and all four `hash`
  invocations in the suite pass md1/mk1 only. Nothing was needed. **A consult is
  not a measurement.**
- **My own first draft of the F-259 message hard-coded `--fill zeros`**, which
  measurably told a `--fill ones` operator to re-run with a different fill —
  F-260's exact shape one line over, inside the fix for F-259. Caught by running
  the other two fills, not by re-reading the text.
- **My first F-264 harness recorded no history at all** and reported *"purged"*
  for the shipped recipe and the fix alike. The `.zshrc` was misnamed. **The
  control caught it**, and it now runs first in the test file for that reason.
- **My row 2 gate prose tripped its own lexical grep.** The plan's gate is
  `grep -c 'EXIT_' io.rs == 0`, and my new comments used the token. I reworded
  the comments rather than loosening the grep.

---

## Deviations from the plan, and why

1. **`seal` declares `--allow-argv-secret`** (consult 3). A CLI surface change
   the plan does not schedule, taken because the alternatives were deleting a
   documented channel or shipping a leak.
2. **The row 6 cross-product covers ten surfaces, not the plan's eight.**
3. **The fish purge recipe was not written.** F-264 documents three fish
   defects, but I could not build a fish history harness with a working control
   — the session wrote no history file at all — and I will not ship a recipe I
   cannot verify. What *was* measured: `history delete --prefix` blocked for two
   minutes on a planted history and deleted nothing. The message now **describes**
   that instead of offering it, which is the idiom the file already uses for
   `history -d`. Filed as **F-271**, owned by P1.
4. **Row 9 created `records.rs` early**, so row 9b's crate move was a pure
   `git mv` — git recorded all six files as renames with zero content change.

---

## Follow-ups

**Filed:** F-271 (fish purge advice unverified; **P1**), F-272 (the plan's
surface list; **P0**, closed in the same commit).

**Closed, each with the mutation that kills it:** F-259, F-264, F-265, F-266,
F-270, F-272.

**Deliberately left open:** F-267 (a secret embedded in a path — out of the
guard's reach by construction, and pinned as a test so the residue is a fact
rather than a hope) and F-268 (flag-name layer, **P3**). The plan and my brief
agree on both.

---

## §6's eleven closure conditions

| # | condition | result |
| --- | --- | --- |
| 1 | all tests pass, unchanged in meaning | 423/423; **0 pre-existing tests lost or renamed** |
| 2 | §5b's invariant — 4 verbs × 4 binaries | **16/16 present, 0 absent** (measured by absolute path: `md` is aliased to `mkdir -p` in this shell) |
| 3 | `mnemonic`'s cell under a verb that EXISTS | `inspect` bad HRP → **2**; `md1` HRP that fails to decode → **1**. And `mnemonic decode` → **64**, confirming the old cell's 64 was clap's |
| 4 | `--expect` in full | all four cases gated and mutation-checked |
| 5 | §6h in-memory history MEASURED | measured; the escape hatch was not taken |
| 6 | F-259 caught by a TEST, not by construction | pty assertion on emitted words, both directions |
| 7 | §8's `CLOSED`-grep discipline | both markers greppable, verified per entry |
| 8 | guard AND override parse both pre-`Cli::parse()` | `me --nosuchflag <ms1>` exits via the guard (3), not clap (2) |
| 9 | F-264 fixed | fixed — and the prescribed fix was measured insufficient |
| 10 | F-265 at ALL FIVE sites | five mutations, five REDs |
| 11 | an R0 round closing 0C/0I | **NOT MINE TO RUN.** Outstanding. |

---

## §5's pre-publish name check (read-only; publish NOT run)

```
serde (CONTROL)   200      <- the gate: without it no 404 beneath means anything
mnemonic-io-lib   404
mnemonic_io_lib   404
```

**A 404 is availability at a moment, not a reservation.** Re-run immediately
before publishing. **Publication is operator-gated and was not performed.**
