# PROBE — exit-code production as a FOURTH SPLIT, EXECUTED

**Throwaway feasibility probe, 2026-08-26.** Worktree
`/scratch/code/shibboleth/_work/probe7/mnemonic-engrave`, branch
`probe/p0-fourthsplit`, off `9c60992` (*"fold the swap probe: my fix was aimed at
the wrong function, and the ordering is OPEN"*). Nothing here is meant to be
merged.

The question: **§4's direction says "exit-code production is a SPLIT, not a
step" — give each of the four integer-carrying functions a return type that
carries the DECISION, keep the `i32` naming at `main.rs`'s call sites, then move
the 11. Does that build with no `EXIT_*` published?**

Downstream of `PROBE-P0-step1.md` and `PROBE-P0-swap.md`, whose measurements
(the 11 + the `cfg(not(unix))` stub, the 8 references across 4 functions, the
differential technique) were reused rather than re-derived.

---

## VERDICT

**YES.** The split executes, publishes **zero** exit-code constants, and the
suite stays green. Two of the three decision-carrying sites the suite CAN see
are caught by mutation; the swap that was inert in probe 2 is caught by **9**
tests here.

```
     Summary [  12.163s] 388 tests run: 388 passed, 1 skipped
```

```
 crates/me-cli/src/io.rs   | 495 +++++++++++++++++++++++++++++++++++++++++++++
 crates/me-cli/src/lib.rs  |   1 +
 crates/me-cli/src/main.rs | 504 +++-------------------------------------------
 3 files changed, 527 insertions(+), 473 deletions(-)
```

**But the honest answer to the plan is smaller than the thing it asked me to
test.** A second variant — move only the five functions **§3's own table
assigns to the crate** — needs **no decision type at all**, publishes nothing,
and is a **160-line** move instead of a 442-line one. See Q5. Simplicity wins,
and the plan already contains the argument for it.

---

## WHAT WAS ACTUALLY DONE

**Step A — the split (all four at once, in `main.rs`, before any move).** One
new type, defined beside `WriteBlock` so it travels with the closure:

```rust
enum Refusal {
    /// The invocation or the environment is wrong — try again. (`me`: 2.)
    Usage,
    /// Understood, well-formed, and refused on purpose — stop. (`me`: 3.)
    Policy,
}
```

| function | before | after |
| --- | --- | --- |
| `refuse_write_block` | `Option<i32>` | `Option<Refusal>` |
| `no_records_guard` | `Result<Vec<String>, (String, i32)>` | `Result<Vec<String>, (String, Refusal)>` |
| `read_records` | `Result<Vec<String>, (String, i32)>` | `Result<Vec<String>, (String, Refusal)>` |
| `emit` | `i32` | `Result<(), Refusal>` |

and **the naming stays in `me`**, beside the constants it names:

```rust
fn exit_code(r: Refusal) -> i32 {
    match r { Refusal::Usage => EXIT_USAGE, Refusal::Policy => EXIT_REFUSED }
}
fn emit_code(r: Result<(), Refusal>) -> i32 {
    match r { Ok(()) => EXIT_OK, Err(x) => exit_code(x) }
}
```

**Step B — the move.** The same two contiguous slices as the earlier probes
(now 251 + 237 = 489 lines) into `crates/me-cli/src/io.rs`, `pub mod io;` in
`lib.rs`, `mnemonic_engrave::` rewritten to `crate::`, every moved item `pub`.
The `& 0o044` mask moved intact, as step 1 requires.

Behaviour was verified **differentially**, not assumed (below).

---

## Q1 — DOES IT BUILD WITH NOTHING PUBLISHED?

**Yes. Both gates the brief names are satisfied.**

```
$ grep -c 'pub const EXIT' crates/me-cli/src/main.rs
0
$ grep -c 'EXIT_' crates/me-cli/src/io.rs
0
$ grep -rn 'EXIT_' crates/me-cli/src/ --include='*.rs' | grep -v '/main.rs' | wc -l
0
$ grep -n 'const EXIT' crates/me-cli/src/main.rs
299:const EXIT_OK: i32 = 0;
300:const EXIT_USAGE: i32 = 2;
301:const EXIT_REFUSED: i32 = 3;
302:const EXIT_INVALID: i32 = 4;
```

All four constants stay **private in the binary**, one coherent table in one
file — the property step 1 destroyed in probe 1 and probe 2.

The closure went from **8** `EXIT_*` references to **0**, measured over the
exact slices before and after:

```
BASELINE closure (464 lines):  EXIT_OK 1   EXIT_USAGE 6   EXIT_REFUSED 1   → 8
AFTER STEP A     (488 lines):  0
```

Compare probe 2, whose one-function fix took 8 → 7.

```
$ cargo build          # exit 0, 0 warnings
$ cargo clippy --all-targets   # exit 0, 0 warnings
$ cargo nextest run --locked
     Summary [  12.163s] 388 tests run: 388 passed, 1 skipped
```

### Behaviour is unchanged — 13 differential probes, byte-identical

Baseline binary `4ea13396…` and post-move binary `0765d44d…` (confirmed
distinct). `diff probe.baseline.txt probe.after.txt` is **empty**.

| probe | rc | probe | rc |
| --- | --- | --- | --- |
| pty terminal refusal | 2 | `--out` (mode 0600) | 0 |
| `>` onto a 0644 file | 2 | `> /dev/null` | 0 |
| `tx:` on argv | 3 | blank lines in `--in` | 0 |
| a seed phrase on argv | 4 | `--allow-argv-secret` | 4 |
| empty `--in` file | 2 | tty-hint path | 2 |
| empty stdin | 2 | `--version` | 0 |
| `me --stdout --in /dev/null` | 2 | | |

Every arm of the closure fired, including both refusal digits. All binaries were
invoked by absolute path; no exit code was read through a pipe.

---

## Q2 — HOW LARGE IS THE CHANGE REALLY?

### Step A, the split alone — 84 changed lines, 16 hunks

```
$ diff -u main.baseline.rs main.stepA.rs | grep -c '^[+-][^+-]'
84          #  61 added, 23 removed
$ grep -c '^@@' stepA.diff
16
```

16 of the 61 added lines are comment/doc, so **45 added / 23 removed of code**.
`main.rs` GROWS by 41 lines (2226 → 2267) before it shrinks.

### Call sites: 5 outside the closure, 1 inside

| function | call sites edited |
| --- | --- |
| `read_records` | 1 (`run_sysw`) |
| `refuse_write_block` | 1 outside (`run_sysw`'s early F-246 gate) + 1 inside `emit` |
| `emit` | 3 (`run_sysw`: `pack` region arm, `pack` arm, `wipe` arm) |
| `no_records_guard` | 0 outside — both callers are inside `read_records` |

No ripple past those. Both enclosing functions (`run_sysw`, `emit`) keep their
own signatures.

### Step B, the move — `main.rs` 2226 → 1784 = **442 lines**

**And that number is ORDERING-DEPENDENT, which is the real finding here.** Three
probes, three shrink figures for the same step:

| probe | order | shrink |
| --- | --- | --- |
| step1 | move only | **461** |
| swap | one signature, then move | **459** |
| fourthsplit | four signatures, then move | **442** |

The plan's step-1 gate says *"`main.rs` shrinks by ~431 lines"*. It is not one
number that is wrong; **a line-count gate cannot be written before the ordering
is fixed**, because the split moves lines INTO `main.rs` first.

### Is there a FIFTH function that produces codes? No — but two others cross

Within the 11, exactly four produce exit codes; nothing else in the closure
returns or names one. `grep -c 'EXIT_'` on `io.rs` is 0 after the move, which is
the proof.

**But the move exposes two functions the plan never mentions as having callers
outside the closure**, and the build fails without them:

```
error[E0425]: cannot find function `write_private` in this scope
   --> crates/me-cli/src/main.rs:445:25      # run_bundle_cli
   --> crates/me-cli/src/main.rs:517:25      # manifest write
   --> crates/me-cli/src/main.rs:641:21      # run_seal_cli, the uf2
error[E0425]: cannot find function `refuse_world_readable_stdout` in this scope
   --> crates/me-cli/src/main.rs:455:13      # run_bundle_cli's own F-244 gate
```

`write_private` has **3** external call sites and `refuse_world_readable_stdout`
has **1**, all in `run_bundle_cli` / `run_seal_cli` — i.e. the bundle and seal
verbs re-use two of the eleven. §1, §3 and §4 all describe the closure purely as
what the six IO functions *call*; nothing records what calls THEM.

---

## Q3 — DOES THE PUBLISHED SURFACE GROW?

**It grows by one enum and shrinks by three constants — and the enum is the
thing §3 says the crate should hold.**

```
$ grep -c '^pub ' crates/me-cli/src/io.rs
15          # 12 fn (11 + the cfg-stub twin), 3 enum
```

| | probe 1 (step1) | probe 2 (swap) | **this probe** |
| --- | --- | --- | --- |
| `pub const EXIT_*` | **3** | **3** | **0** |
| public enums beyond the closure's own | 0 | 1 (`WriteRefusal`) | 1 (`Refusal`) |
| public enums total | 2 | 3 | 3 |

**`Refusal` cannot stay private, exactly as `WriteRefusal` could not.** Measured:

```
$ sed -i 's/^pub enum Refusal {/enum Refusal {/' crates/me-cli/src/io.rs && cargo build
warning: type `Refusal` is more private than the item `emit`
warning: type `Refusal` is more private than the item `no_records_guard`
warning: type `Refusal` is more private than the item `read_records`
warning: type `Refusal` is more private than the item `refuse_write_block`
error[E0603]: enum `Refusal` is private
```

So probe 2's I-2 recurs in form — **but not in substance.** Probe 2 traded 0
constants for 1 enum and still published 3 constants: net `+1`. This trades
**3 published integers for 1 published non-numeric decision**, which is
precisely what §3's table asks for (*"the crate holds the refusal decision
types … the crate does NOT hold any `EXIT_*` integer, any binary→code
mapping"*). `ms` can map `Refusal::Usage` to **64** and `me` to **2** without
either ignoring or adopting the other's number.

---

## Q4 — ARE THE DECISION TYPES ACTUALLY TESTED?

**One decision type was introduced: `Refusal`.** It has **7 production sites**
in the moved code. Every one was mutated individually, plus the literal
two-variant swap, plus `emit`'s `Ok` arm. Each mutation was built and run
against the full suite with `--no-fail-fast`.

**Every mutation reported below was PROVEN TO RUN** — for each one the mutated
binary was invoked on the exact input that reaches that line and observed
returning a different exit code. A green suite on a line that never executed
would not be evidence of anything.

| # | site | mutation | suite | proven to run |
| --- | --- | --- | --- | --- |
| **MSWAP** | both live sites | `Usage` ⇄ `Policy` | **RED — 9 tests** | yes (implied by 9 reds) |
| M1a | `read_records`, argv gate (`io.rs:422`) | `Policy` → `Usage` | **RED — 1 test** | yes |
| M1b | `no_records_guard` (`io.rs:311`) | `Usage` → `Policy` | **RED — 5 tests** | yes |
| M7 | `emit`, success arm (`io.rs:487`) | `Ok(())` → `Err(Usage)` | **RED — 58 tests** | yes |
| M2 | `refuse_write_block`, Terminal arm (`io.rs:181`) | `Usage` → `Policy` | **GREEN — MISSED** | **yes: pty rc 2 → 3** |
| M3 | `refuse_write_block`, WorldReadable arm (`io.rs:185`) | `Usage` → `Policy` | **GREEN — MISSED** | **yes: `>` 0644 rc 2 → 3** |
| M4 | `read_records`, `--in` read error (`io.rs:324`) | `Usage` → `Policy` | **GREEN — MISSED** | **yes: `--in /nonexistent` rc 2 → 3** |
| M5 | `read_records`, stdin read error (`io.rs:444`) | `Usage` → `Policy` | **GREEN — MISSED** | **yes: stdin=a directory rc 2 → 3** |
| M6 | `emit`, write failure (`io.rs:492`) | `Usage` → `Policy` | **GREEN — MISSED** | **yes: `--out` into a 0500 dir rc 2 → 3** |

**The swap that was inert in probe 2 is not inert here.**

```
$ cargo nextest run --locked --no-fail-fast     # with the two variants SWAPPED
     Summary [  12.475s] 388 tests run: 379 passed, 9 failed, 1 skipped
        FAIL  cli::the_exit_code_vocabulary_is_one_vocabulary
        FAIL  sysw_cli::a_bearer_record_on_argv_outranks_the_write_gate
        FAIL  sysw_cli::a_tx_record_anywhere_on_argv_is_refused_and_located
        FAIL  sysw_cli::a_tx_record_on_argv_is_refused
        FAIL  sysw_cli::an_empty_in_file_is_the_exit_2_path_too
        FAIL  sysw_cli::empty_stdin_is_the_exit_2_path_not_an_empty_container
        FAIL  sysw_cli::the_argv_refusal_echoes_neither_the_transaction_nor_a_passphrase
        FAIL  sysw_cli::the_empty_in_refusal_names_the_file
        FAIL  sysw_cli::the_no_records_message_names_stdin
```

Compare probe 2: `388 tests run: 388 passed` on the equivalent swap.

**Why the difference, and it is not luck.** Probe 2's `WriteRefusal::Terminal`
and `::WorldReadable` **both mapped to `EXIT_USAGE`** — the type had two names
for one observable, so `-> bool` would have been byte-identical and no test
could possibly distinguish them. `Refusal`'s two variants map to **2 and 3**,
digits `tests/cli.rs::the_exit_code_vocabulary_is_one_vocabulary` pins as a
table. **A decision type is testable exactly insofar as its variants produce
different observable behaviour**; the design rule that follows is *do not give a
decision type more variants than the caller can distinguish.*

### THE FIVE MISSES ARE PRE-EXISTING, AND A CONTROL PROVES IT

The five green mutations are not a defect this direction introduces. The same
mutation class applied to the **UNMODIFIED baseline** — `Some(EXIT_USAGE)` →
`Some(EXIT_REFUSED)` in `refuse_write_block`'s Terminal arm, and `EXIT_USAGE` →
`EXIT_REFUSED` at `read_records`'s `--in` error — is equally invisible:

```
$ cargo nextest run --locked --no-fail-fast    # BASELINE code, exit ints changed 2 -> 3
     Summary [  12.964s] 388 tests run: 388 passed, 1 skipped
$ /usr/bin/script -qec "$ME sysw pack --no-passphrase text:6869" /dev/null
control pty(terminal arm) rc=3          # baseline 2 — the line RAN
$ $ME sysw pack --no-passphrase --in /nonexistent/nope.txt --out x.bin
control --in-unreadable rc=3            # baseline 2 — the line RAN
```

So `me` today can have its terminal refusal, its world-readable refusal, its
`--in` error, its stdin error and its write failure all silently respell
themselves as a **policy refusal (3)** instead of **usage (2)** and report
388/388 green. That is a live hole in the exit-code vocabulary the plan's own
§2/§6f treats as normative — **the property is asserted by a table in one test
that covers 3 of the 8 sites.**

---

## Q5 — IS THERE A SIMPLER ANSWER? YES, AND IT WAS EXECUTED TOO

**VARIANT B: move only the five functions §3's own table assigns to the crate.
No decision type at all. It builds, it is green, and it publishes nothing.**

§3's table (*"THE 11 MAPPED ONTO THE FILES (M6)"*) assigns to the shared crate:
`destination` (channel.rs), `stdout_world_readable_mode` + stub (fd.rs),
`split_record_stream` + `no_records_guard` (records.rs), `write_block`
(exit.rs). It keeps `read_records`'s class arm, **every `refuse_*`**, `emit` and
`write_private` in `me`.

That set is **closed** (`write_block` calls `destination`; nothing else escapes)
and contains **exactly one** `EXIT_*` reference — `no_records_guard`'s. And
`no_records_guard`'s only two callers are inside `read_records`, which stays in
`me`. So the entire fix is:

```rust
fn no_records_guard(recs: Vec<String>, from: Option<&std::path::Path>)
    -> Result<Vec<String>, String>            // was Result<Vec<String>, (String, i32)>
```
```rust
// read_records, staying in `me`, names the number:
no_records_guard(split_record_stream(&raw), Some(p)).map_err(|m| (m, EXIT_USAGE))
```

Measured, executed in this same worktree:

```
$ cargo build ; cargo clippy --all-targets ; cargo nextest run --locked
BUILD_EXIT=0   CLIPPY_EXIT=0   (0 warnings)
     Summary [  12.087s] 388 tests run: 388 passed, 1 skipped
$ grep -c 'pub const EXIT' crates/me-cli/src/main.rs   → 0
$ grep -c 'EXIT_' crates/me-cli/src/io.rs              → 0
```

| | variant A (the four-way split) | **variant B (the minimal closure)** |
| --- | --- | --- |
| functions moved | 11 + stub | **5 + stub** |
| `main.rs` shrink | 442 | **160** |
| lines in the new module | 495 | **168** |
| public items | 15 | **8** |
| new public types | 1 (`Refusal`) | **0** |
| published `EXIT_*` | 0 | **0** |
| signature changes | 4 | **1** |
| `git diff --stat` | 527 ins / 473 del | **177 ins / 170 del** |

**Variant B is strictly smaller and answers the same question.** What it does
not do is discharge §4's stated reason for step 1 — *"it is the step that proves
the closure is really 11 and not more"* — but **three probes have now measured
that closure**, so that proof already exists on disk and does not need a code
step to re-establish it.

**And variant B sequences the decision type to the step that earns it.** The
crate needs a two-variant `Refusal` only once `records.rs`'s **argv gate** moves
(step 5b), because that gate is the sole producer of the `Policy` variant.
Introducing the type at step 1, when the only crossing refusal is a single
`Usage`, is a type with one reachable meaning — and that is exactly the shape
probe 2 measured as inert.

### On "just publish the constants and let I2 be wrong"

**Do not.** It is not needed — two orderings now work without it — and the
publication is not as harmless as "it's only the donor's own lib": step 1 moves
into `me`'s **lib half**, whose crate is `mnemonic-engrave` and whose `[lib]`
is `mnemonic_engrave`. `pub const EXIT_USAGE: i32 = 2;` there is a public API
item on a versioned crate that step 7 must then remove, and probe 1 already
named the likelier outcome: the tests stay green, so it gets carried forward.

---

## WHERE THE PLAN IS WRONG

### CRITICAL

**C-1. §4's direction is CORRECT but its arithmetic still says "four", and the
plan should record that ONE is enough.** §4: *"exit-code production is a fourth
split … the four functions above each decide something and then name a
number."* True of step 1 as written (all 11 move). **False of the crate §3
describes**: of the four, `refuse_write_block` and `emit` are on §3's
**stays in `me`** rows, and `read_records`'s `EXIT_REFUSED` is in the
class-keyed arm §3 also keeps in `me`. So the number of signatures that must
change for the CRATE boundary is **one — `no_records_guard`** — and it needs no
new type, because its only caller stays behind. Executed and green (Q5). The
plan is about to schedule a four-function refactor to solve a one-function
problem, and it contains the evidence for that in its own §3 table.

**C-2. Step 1's gate cannot see 5 of the 8 exit-code decisions — and neither can
the baseline.** Measured by mutation with a control (Q4). `me` can respell its
terminal refusal, world-readable refusal, `--in` error, stdin error and write
failure from 2 to 3 and report **388 passed, 1 skipped**. §4's I1 correctly
identifies the terminal arm as unreachable by the suite and prescribes a pty
assertion — **but I1 names only that one arm, and the assertion must pin the
DIGIT.** `assert!(!status.success())` passes under M2. Three more sites
(`--in`, stdin, write failure) are invisible even with I1 satisfied. This is a
gap in `me` today, not in the extraction; it should be a named finding with an
owning phase rather than a surprise discovered when a shared crate starts
carrying the vocabulary.

### IMPORTANT

**I-1. §4's conclusion from probe 2 — *"the decision type is inert, and nothing
tests it"* — does not generalise, and carrying it forward as a ruling would be
wrong.** Probe 2's `WriteRefusal` was inert **by construction**: both variants
mapped to `EXIT_USAGE`, so no observable distinguished them. A `Refusal` whose
variants map to 2 and 3 is caught by **9** tests under the identical swap
mutation (Q4). The correct general rule is narrower and more useful: **a
decision type is testable exactly insofar as its variants produce different
observable behaviour** — so do not give one more variants than a caller can
distinguish, and mutate it to find out which.

**I-2. Nothing in §1/§3/§4 records that two of the 11 have callers OUTSIDE the
closure**, and the move does not compile without knowing it. `write_private` (3
sites) and `refuse_world_readable_stdout` (1 site) are used by `run_bundle_cli`
and `run_seal_cli` (Q2). The plan describes the closure only downward — what the
six IO functions call — so an implementer meets this as four `E0425`s. It also
sharpens §3's *"`emit` and `write_private` stay in `me` for now … moving them is
P1's question"*: `write_private` is not only P0's question, it is shared by
three verbs today.

**I-3. §4's step-1 shrink figure cannot be written before the ordering is
fixed.** *"`main.rs` shrinks by ~431 lines"* has now been measured three times
at three values — **461 / 459 / 442** — because each candidate ordering adds a
different number of lines to `main.rs` before removing the slices (this
probe's split adds 41). Probe 1's M-1 and probe 2's M-3 both asked for the
number to be corrected; the finding is that it should be **deleted from the
gate**, not corrected. The gate that carries real information is the pair of
greps in Q1 plus `388 passed, 1 skipped`.

### MINOR

**M-1. Probe 1's M-2 recurs a third time: the move edits `main.rs`'s own test
module and step 1's gate is still worded as if it does not.** One line:

```
-    use super::{destination, is_plate_artifact, write_block, Destination, WriteBlock};
+    use super::is_plate_artifact;
+    use mnemonic_engrave::io::{destination, write_block, Destination, WriteBlock};
```

The gate says *"the move necessarily edits 2 of them"* — accurate as a count of
affected **tests** (`a_terminal_is_never_a_destination_for_the_container`,
`write_block_decides_both_gates_once`), and it is **one** line of diff. Worth
stating that way so an executor does not go looking for two edits.

**M-2. §3's I5 list is still one short even after its own correction.** It names
seven crossing symbols (`EXIT_OK`, `EXIT_USAGE`, `EXIT_REFUSED`, `WriteBlock`,
`Destination`, *"and the two enums travelling with them"* — which double-counts
`WriteBlock`/`Destination`). Probe 1 measured **11**: the three constants, the
two enums, `sysw::classify`, `sysw::record::Class`, `Class::is_argv_forbidden`,
`Class::is_bearer`, `sysw::record::TX_PREFIX`, `sysw::wire::REGION_ADDR`,
`sysw::wire::REGION_LEN`. The fold picked up the constants and dropped the other
four. Re-verified here: `grep -o` over the moved 488 lines still finds
`TX_PREFIX`, `classify`, `REGION_ADDR`, `REGION_LEN`.

**M-3. `Destination` is used by nothing outside the test module once the move is
done.** `grep -c 'Destination'` in the post-move `main.rs` non-test code is 0;
its four uses are all in `#[cfg(test)] mod tests`. It is published solely
because `destination()` returns it — fine, but worth knowing that step 7's
"enumerate the diff to the tests" work will find the `Destination`/`WriteBlock`
unit tests are the *only* consumers and should move into the crate with them.

---

## WHAT DID NOT GO WRONG

- The direction §4 proposes **works**. Two orderings have now failed and the
  third succeeds; the plan's diagnosis (*"only the naming stays in `me`"*) is
  right.
- The closure is still exactly **11 fns + the `cfg(not(unix))` stub**. No 12th
  function, in either variant.
- Behaviour is genuinely unchanged: 13 differential probes, byte-identical
  output between two confirmed-distinct binaries, every gate observed firing at
  rc 0/2/3/4.
- `cargo clippy --all-targets` is clean at **every** stage — baseline, post-split,
  post-move, and variant B. No `is_some()` lint fight this time, because the
  call sites still bind the value.
- `388 tests run: 388 passed, 1 skipped` is exact at baseline, post-split,
  post-move and in variant B. The skip is the deliberate
  `sysw::vectors::tests::regenerate`.
- The two earlier probes' measurements reproduce: 8 `EXIT_*` references across
  4 of the 11, `refuse_write_block`'s 2 call sites, the `use super::` breakage.

---

## COMMANDS

```
cargo build                          # exit 0, 0 warnings
cargo clippy --all-targets           # exit 0, 0 warnings
cargo nextest run --locked           # 388 tests run: 388 passed, 1 skipped
cargo nextest run --locked --no-fail-fast    # per-mutation, to enumerate reds
grep -c 'pub const EXIT' crates/me-cli/src/main.rs    # 0
grep -c 'EXIT_' crates/me-cli/src/io.rs               # 0
/usr/bin/script -qec "$ME sysw pack --no-passphrase text:6869" /dev/null   # rc 2
```

All binaries were invoked by absolute path; no exit code was read through a
pipe; every suite was captured once to a file and grepped.

## FINAL STATE OF THIS WORKTREE

Variant A (the four-way split) is what is checked in here, because it is what
the brief asked to be tested. **Variant B is the recommendation.**

```
     Summary [  12.163s] 388 tests run: 388 passed, 1 skipped
```
```
 crates/me-cli/src/io.rs   | 495 +++++++++++++++++++++++++++++++++++++++++++++
 crates/me-cli/src/lib.rs  |   1 +
 crates/me-cli/src/main.rs | 504 +++-------------------------------------------
 3 files changed, 527 insertions(+), 473 deletions(-)
```
