# PROBE — the SWAPPED step order (`step 2 first, then step 1`), EXECUTED

**Throwaway feasibility probe, 2026-08-26.** Worktree
`/scratch/code/shibboleth/_work/probe2/mnemonic-engrave`, branch `probe/p0-swap`,
off `ab520d4` (*"fold the probe: step 1 as written CANNOT BE DONE"*). Nothing
here is meant to be merged. The question was **"the fold says swapping the steps
removes the need to publish `EXIT_*` — does it?"**

Downstream of `design/agent-reports/PROBE-P0-step1.md`, whose measurements
(464-line closure, 11 fns + stub, the differential technique) were reused rather
than re-derived.

---

## VERDICT

**NO.** The swapped order executes and the suite stays green — but it **does not
do the thing it was created to do**. After `refuse_write_block` returns the
DECISION, the move of the 11 still fails with **seven** `E0425: cannot find
value EXIT_*`, and step 1 still ends with `pub const EXIT_USAGE: i32 = 2;` in the
donor's public API. The fold's own words —

> **Step 1 second** — move the 11 into `me`'s lib half, now with **no constant
> left to publish.**

— are false by seven.

```
     Summary [  12.048s] 388 tests run: 388 passed, 1 skipped
```

```
 crates/me-cli/src/io.rs   | 482 +++++++++++++++++++++++++++++++++++++++++++++
 crates/me-cli/src/lib.rs  |   1 +
 crates/me-cli/src/main.rs | 483 ++--------------------------------------------
 3 files changed, 495 insertions(+), 471 deletions(-)
```

**What was actually done, in the fold's order:** (2) `refuse_write_block`'s
return type changed from `Option<i32>` to `Option<WriteRefusal>`, both call sites
mapping the decision onto `EXIT_USAGE`; build + clippy + 388 green. Then (1) the
same 11 + stub + `Destination` + `WriteBlock` + the new `WriteRefusal` moved into
`crates/me-cli/src/io.rs`, `pub mod io;` in `lib.rs`. Behaviour was verified
differentially against a baseline binary, not assumed.

---

## Q1 — DOES THE DECISION TYPE AVOID THE CONSTANTS CROSSING?

**No, and it drags a new public item across on the way.**

### The constants still cross — 7 of the original 8 references survive

`refuse_write_block` was never where the exit integers lived. Counted over the
exact closure slice, comments included, before and after step 2:

```
$ python3  # regex \bEXIT_x\b over the two contiguous closure slices
BASELINE (pre-step-2), 464 lines:   EXIT_OK 1  EXIT_USAGE 6  EXIT_REFUSED 1   → 8
POST-STEP-2,           472 lines:   EXIT_OK 1  EXIT_USAGE 5  EXIT_REFUSED 1   → 7
```

**Step 2 removes exactly ONE reference from the closure.** It takes two out of
`refuse_write_block` and puts one straight back into `emit` — which is itself one
of the 11. The other lands in `run_sysw`, outside.

The seven that remain, by containing function: **`no_records_guard` 1,
`read_records` 3, `emit` 3.** None of the three is mentioned by the fold.

### The move then fails, exactly as before

```
$ cargo build          # after step 2, with the 11 moved to io.rs
error[E0425]: cannot find value `EXIT_USAGE` in this scope
   --> crates/me-cli/src/io.rs:295:9      #  no_records_guard
error[E0425]: cannot find value `EXIT_USAGE` in this scope
   --> crates/me-cli/src/io.rs:308:60     #  read_records, --in channel
error[E0425]: cannot find value `EXIT_REFUSED` in this scope
   --> crates/me-cli/src/io.rs:406:21     #  read_records, argv gate
error[E0425]: cannot find value `EXIT_USAGE` in this scope
   --> crates/me-cli/src/io.rs:428:46     #  read_records, stdin channel
error[E0425]: cannot find value `EXIT_USAGE` in this scope
   --> crates/me-cli/src/io.rs:461:16     #  emit  <- ADDED BY STEP 2 ITSELF
error[E0425]: cannot find value `EXIT_OK` in this scope
   --> crates/me-cli/src/io.rs:470:19     #  emit
error[E0425]: cannot find value `EXIT_USAGE` in this scope
   --> crates/me-cli/src/io.rs:475:13     #  emit
error: could not compile `mnemonic-engrave` (lib) due to 8 previous errors
```

(The 8th is a `use std::io::Write;` the extraction header omitted — a harness
artefact, not a plan finding.)

The only way forward is the one the plan rules out:

```rust
// crates/me-cli/src/io.rs — PUBLIC API of the lib half, after the SWAPPED order
pub const EXIT_OK: i32 = 0;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_REFUSED: i32 = 3;
```

### And the decision type must itself be published

Making `WriteRefusal` private while `refuse_write_block` is `pub`:

```
warning: type `WriteRefusal` is more private than the item `refuse_write_block`
   --> crates/me-cli/src/io.rs:164:1
error: type `mnemonic_engrave::io::WriteRefusal` is private
   --> crates/me-cli/src/main.rs:969:20
```

So the swap's net effect on the published surface is **+1 public enum, −0
public constants**. No `From<WriteRefusal> for i32` was needed and no `match` in
`main` — because nothing reads the decision at all (see I-1).

---

## Q2 — ARE THE FOLD'S THREE CLAIMS TRUE?

| claim | measured | verdict |
| --- | --- | --- |
| **2 call sites** | 2 — `emit`, and `run_sysw`'s pack arm | **TRUE** |
| **nothing outside the 11 depends on the `i32`** | nothing ripples; but `run_sysw` **is** outside the 11 and *did* consume the `i32` as its own return | **MISLEADING** (M-2) |
| **17-line change** | **30** with a bare enum, **35** with doc comments + derives | **FALSE** (M-1) |

```
$ diff -u main.baseline.rs main.rs | grep -c '^[+-][^+-]'
30        # bare `enum WriteRefusal { Terminal, WorldReadable }`
35        # with the doc comments and #[derive(Debug, PartialEq, Eq)]
```

The terser call-site form that would get nearer 17 is rejected by the repo's own
lint gate:

```
$ cargo clippy --all-targets     # with `if let Some(_) = refuse_write_block(…)`
warning: redundant pattern matching, consider using `is_some()`
   --> crates/me-cli/src/io.rs:453:12
```

so the `.is_some()` + braced-block rewrite is **forced** at both call sites, and
that is where the extra lines come from. No test references
`refuse_write_block`; `grep -n "refuse_write_block(" src/*.rs tests/*.rs`
returns the definition and the two call sites and nothing else.

---

## Q3 — DOES STEP 1 THEN SUCCEED WITHOUT PUBLISHING ANY `EXIT_*`?

**No.** Q1 above. With the three constants published, step 1 completes exactly as
the first probe reported it — same three follow-on edits, same green suite:

```
$ cargo build                 # 0 warnings
$ cargo clippy --all-targets  # 0 warnings, 0 errors
$ cargo nextest run --locked
     Summary [  12.048s] 388 tests run: 388 passed, 1 skipped
```

`main.rs` **2226 → 1767 = 459 lines** (the plan's gate says ~431).

### Behaviour is unchanged — verified differentially, not assumed

Baseline binary `23886e31…` and post-swap binary `ce31d7fd…` (confirmed
distinct), 12 probes each, every arm of the closure:

| probe | rc | probe | rc |
| --- | --- | --- | --- |
| pty terminal refusal | 2 | `--out` (mode 0600) | 0 |
| `>` onto a 0644 file | 2 | `> /dev/null` | 0 |
| `tx:` on argv | 3 | blank lines in `--in` | 2 |
| `ms1` on argv | 4 | `--allow-argv-secret` | 4 |
| empty `--in` file | 2 | stdin-is-a-TTY hint | 2 |
| empty stdin | 2 | `--version` | 0 |

`diff probe.baseline.txt probe.step1.txt` differs on **one line only** — the
`mktemp -d` path echoed inside a refusal message. Every gate fired; none of these
is a false pass.

---

## Q4 — IS THERE A THIRD ORDERING PROBLEM THE SWAP CREATES?

**Yes — the plan now states three mutually inconsistent orderings within seven
lines, and §4's table was never renumbered.**

- **line 409** — table row 1: *"move the 11 into `me`'s lib half **intact —
  including the mask**, no behaviour change"*
- **line 410** — table row 2: *"`fd.rs` — **SPLIT** `stdout_world_readable_mode`"*
- **line 450** — *"**So step 1 and step 2 SWAP.** `refuse_write_block` returns
  the DECISION first"*
- **line 456** — *"**Steps are ordered this way on purpose.** Step 1 moves
  `stdout_world_readable_mode` *with* its `& 0o044` so nothing changes; **step 2
  then splits it**"*

The fold's "step 2" (`refuse_write_block` returns a decision) **is not step 2**.
It is not in the table at all. The table's step 2 is the `fd.rs` mask split, and
line 456 — six lines below the swap — still asserts the original 1-then-2 order
as deliberate.

Three readings, all supported by the text:

1. **Follow the table** (rows unrenumbered): execute the move first — the exact
   step the fold exists to declare impossible.
2. **Literally swap rows 1 and 2**: do the `fd.rs` mask split first. That does
   nothing about `EXIT_*`, and it makes step 1's own gate wording
   (*"intact — including the mask"*) unsatisfiable, because the mask is already
   at the call site. It also inverts line 456's *"at no point does a masked
   function sit inside the crate"* argument.
3. **Follow the prose**: insert an unnumbered new first step. This is what the
   author meant, and it is the only reading that is not self-contradictory — and
   it is the one that Q1 shows does not achieve its purpose.

**This is a worse ordering hazard than the one the fold repaired**, because the
original plan was merely impossible; this one is ambiguous and one of its three
readings is silently green.

---

## WHERE THE PLAN IS WRONG

### CRITICAL

**C-1. The swap's stated purpose is not achieved: `EXIT_OK` / `EXIT_USAGE` /
`EXIT_REFUSED` still must be published at step 1.** Step 2 removes **1 of 8**
references from the closure and relocates one of those into `emit`, itself one
of the 11. Seven `E0425` remain, in `no_records_guard`, `read_records` and
`emit`. Evidence: the compiler output in Q1. The fold was reasoned, not run, and
it reached the opposite of the measurable answer. **The remedy is aimed at the
one function in the closure whose integers do not matter.**

**C-2. §4 now contains three inconsistent orderings and the table was not
renumbered** (Q4). An implementer following §4's table — the only numbered thing
in the section — executes the impossible step first. An implementer taking
"swap" literally does the `fd.rs` split first and breaks step 1's own gate
wording. Neither is what the prose means.

### IMPORTANT

**I-1. The decision change is inert — the suite cannot tell the two decisions
apart.** Mutation: swap the two arms so `WriteBlock::Terminal` yields
`WriteRefusal::WorldReadable` and vice versa.

```
$ cargo nextest run --locked      # with the two decisions SWAPPED
     Summary [  13.101s] 388 tests run: 388 passed, 1 skipped
```

Both call sites consume the result with `.is_some()` and map every variant to
`EXIT_USAGE`; **no code anywhere reads which gate blocked**. `-> bool` would be
byte-identical in behaviour. This is the first probe's I-1 measured a second way:
the signature change "that makes this boundary real rather than asserted" makes
nothing real, and now there is a green mutation to prove it.

**I-2. The swap grows the published surface rather than shrinking it.** Before:
3 private constants. After both steps: **3 public constants + 1 public enum**.
`WriteRefusal` cannot stay private while `refuse_write_block` is `pub`
(`error: type … is private`, Q1).

**I-3. The first probe's I-1 was cited in the fold and then not acted on.** That
finding named the functions that actually carry an exit integer across the
boundary — `no_records_guard` and `read_records`, `Result<Vec<String>, (String,
i32)>` — and the fold changed `refuse_write_block` instead. Those two plus `emit`
hold **7 of 7** surviving references. A step 2 that worked would change
`no_records_guard` / `read_records` to `Result<Vec<String>, (String, Refusal)>`
and `emit` to `Result<(), Refusal>`; that, and only that, empties the closure of
integers.

### MINOR

**M-1. "a 17-line change" is not reproducible.** 30 lines minimal, 35 as written
with docs and derives (Q2). The 17 came from the first probe's measurement in the
**post-move** file, where `emit`'s call site sat in `io.rs` beside the function —
a different diff shape from the pre-move file the swapped order actually edits.
A number carried across a changed context.

**M-2. "nothing outside the 11 depends on the `i32`" is loose.** One of the two
call sites, `run_sysw`, **is** outside the 11 and consumed the `i32` as its own
return value; after the change it names `EXIT_USAGE` itself. The intended claim —
nothing ripples past the two call sites — is true, and I verified it.

**M-3. The first probe's M-1 was not folded: the ~431-line shrink figure is still
in the step-1 gate at line 409.** Measured here, in the swapped order:
**2226 → 1767 = 459**.

**M-4. The first probe's M-2 was not folded either, and recurs identically.** The
move breaks `main.rs`'s own `#[cfg(test)] mod tests`:

```
error[E0432]: unresolved imports `super::destination`, `super::Destination`, `super::WriteBlock`
    --> crates/me-cli/src/main.rs:1705:17
```

Step 1's gate is still worded as if the 388 are untouched.

---

## RECOMMENDATION — the ordering that would work

Not executed; stated because the brief asks for it if visible.

**There is no ordering of "change one function's return type" and "move the 11"
that avoids publishing the constants, because six of the seven references were
never in that function.** The choice is between two real options:

1. **Change all three integer-carrying signatures before the move.**
   `no_records_guard` and `read_records` → `Result<Vec<String>, (String,
   Refusal)>`; `emit` → `Result<(), Refusal>`; `refuse_write_block` →
   `Option<Refusal>`. Then the closure holds zero `EXIT_*` and step 1 publishes
   nothing. This is a real change with real call sites in `main.rs` — larger than
   17 lines, and it is the actual content of the boundary §3 argues for.
2. **Accept the publication as an explicitly temporary intermediate**, with step
   1's gate rewritten to say the constants are published *and* that step 7 must
   remove them, plus a check that they are gone. Cheaper, honest, and does not
   pretend the boundary exists before it does.

Option 1 is what §3's own argument implies. Either way, **§4's table must be
renumbered**, because the prose and the table currently disagree.

---

## WHAT DID NOT GO WRONG

- Both steps compile, `clippy --all-targets` is clean, and **388/388 + 1 skipped**
  is exact at every stage: baseline, post-step-2, post-move.
- The closure is still exactly 11 fns + the `cfg(not(unix))` stub; the swap pulls
  in no 12th function.
- Behaviour is genuinely unchanged — 12 differential probes, one `mktemp` path
  apart, with every gate observed firing.
- The first probe's 464-line closure measurement reproduces exactly.
- `refuse_write_block` really does have only 2 call sites and really does not
  ripple.

## COMMANDS

```
cargo build                   # exit 0, 0 warnings (final tree)
cargo clippy --all-targets    # exit 0, 0 warnings
cargo nextest run --locked    # 388 tests run: 388 passed, 1 skipped
/usr/bin/script -qec "$ME sysw pack --no-passphrase text:6869" /dev/null   # rc 2
```

All binaries were invoked by absolute path; no exit code was read through a pipe.
