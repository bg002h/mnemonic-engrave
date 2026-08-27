# PROBE — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` §4 step 1, EXECUTED

**Throwaway feasibility probe, 2026-08-26.** Worktree
`/scratch/code/shibboleth/_work/probe/mnemonic-engrave`, branch `probe/p0-step1`,
off `2757e80`. Nothing here is meant to be merged. The question was not "is this
code good" but **"does step 1 execute as written, and what does the plan get
wrong when you try".**

---

## VERDICT

**YES WITH DEVIATIONS.** The move compiles, `clippy` is clean, and all 388 tests
pass — but step 1 as written cannot be executed without **three things the plan
does not mention**, one of which drags the exit-code integers into the shared
half three steps before the plan expects them.

```
     Summary [  12.785s] 388 tests run: 388 passed, 1 skipped
```

```
 crates/me-cli/src/io.rs   | 478 +++++++++++++++++++++++++++++++++++++++++++++
 crates/me-cli/src/lib.rs  |   1 +
 crates/me-cli/src/main.rs | 479 +---------------------------------------------
 3 files changed, 488 insertions(+), 470 deletions(-)
```

**What was actually done:** the 11 + the `#[cfg(not(unix))]` stub + `Destination`
+ `WriteBlock` were moved verbatim (two contiguous slices of `main.rs`,
`843..1073` and `1863..2095`, 464 lines) into a new `crates/me-cli/src/io.rs`
declared `pub mod io;` in `lib.rs`. `mnemonic_engrave::` self-references were
rewritten to `crate::`; every moved item was made `pub`. **The `& 0o044` mask
moved intact**, as step 1 requires. Behaviour was then differentially verified
(§Q4 below), not assumed.

---

## Q1 — IS THE CLOSURE REALLY 11 (+1 STUB)?

**As a count of `fn`s: YES, exactly.** No 12th or 13th function was pulled in.
The move compiled with `cargo build` exit 0 and **zero warnings** on the first
attempt.

```
$ cargo build
   Compiling mnemonic-engrave v0.7.0 (…/crates/me-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.89s
EXIT=0
```

**As a closure: NO — three constants had to move with it, and the plan lists
none of them.** `cargo nextest run` then failed on a fourth thing the plan does
not mention. See Q2.

Machine-checked against the plan's own §1 claims about the closure, on the
extracted 464 lines:

| §1 claim | measured | verdict |
| --- | --- | --- |
| `0` hits for `std::process::exit`, `Cli`, `clap` | 0 | TRUE |
| `std::env::args` is 0 | 0 | TRUE |
| 4 `eprintln!`, **0** bare `println!` (M1) | 4 / 0 | TRUE |
| the closure's only stdout write is `emit`'s `write_all` | 1 | TRUE |
| `Admission` "crosses with them" (§3 I5) | **0 hits** | **FALSE — see I-2** |

---

## Q2 — DOES §1's / §3's TYPE-AND-CONSTANT RULE HOLD?

**No.** I did the enumeration §3's I5 assigns to step 1 — mechanically, over the
moved code with comments stripped — and the plan's list is wrong in both
directions.

```
$ python3 …  # regex over crates/me-cli/src/io.rs, comment lines removed
=== crate:: paths referenced by the closure (code only):
   crate::sysw::classify
   crate::sysw::record::Class
   crate::sysw::record::TX_PREFIX
   crate::sysw::wire::
=== bare type/const idents from me's own vocabulary:
   Class::Mt   Class::Tx   EXIT_OK   EXIT_REFUSED   EXIT_USAGE
   REGION_ADDR   REGION_LEN   TX_PREFIX   is_argv_forbidden   is_bearer
```

**The complete list is 11 external symbols. §3's I5 names four, of which one
does not cross at all:**

| symbol | kind | in I5's list? |
| --- | --- | --- |
| `WriteBlock` | enum, moved with the closure | yes |
| `Destination` | enum, moved with the closure | yes |
| `sysw::record::Class` (+ `Class::Mt`, `Class::Tx`) | enum in `me`'s lib | yes |
| `Class::is_argv_forbidden`, `Class::is_bearer` | **inherent methods** | implied by N-C2 |
| `sysw::record::TX_PREFIX` | const | **NO** |
| `sysw::classify` | fn | **NO** |
| `sysw::wire::REGION_ADDR` | const | **NO** |
| `sysw::wire::REGION_LEN` | const | **NO** |
| `EXIT_OK` | const, **was in `main.rs`** | **NO** |
| `EXIT_USAGE` | const, **was in `main.rs`** (×6 uses) | **NO** |
| `EXIT_REFUSED` | const, **was in `main.rs`** | **NO** |
| `Admission` | — | **listed, but 0 references** |

### C-1 — the three `EXIT_*` constants are forced into the shared half AT STEP 1

`EXIT_OK`, `EXIT_USAGE` and `EXIT_REFUSED` were defined in `main.rs`
(lines 295–297 of the pre-move file). **A library module cannot see a binary's
items.** So moving the 11 "intact" is only possible if the constants move too —
there is no third option, because keeping them behind would require changing the
moved functions' signatures, which step 1 forbids ("intact … no behaviour
change").

What that means concretely, after the move:

```rust
// crates/me-cli/src/io.rs — now PUBLIC API of the lib half
pub const EXIT_OK: i32 = 0;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_REFUSED: i32 = 3;
```
```rust
// crates/me-cli/src/main.rs
use mnemonic_engrave::io::{EXIT_OK, EXIT_REFUSED, EXIT_USAGE};
const EXIT_INVALID: i32 = 4;   // not in the closure — stays private to the bin
```

The four exit codes were one coherent table in one file. Step 1 splits them
across two halves with two different visibilities, and publishes three of them.
**This is I2's own problem — "a constant is a mapping" — arriving five steps
before the plan schedules it**, and §4's own note that "steps 1 and 2 are ordered
this way on purpose … at no point does a masked function sit inside the crate"
has no counterpart for this. There is no ordering that avoids it: it is a
language rule, like N-C2.

### C-2 — step 1's I5 check is unexecutable as a gate

I5 requires step 1 to "confirm each is either moved or **reachable WITHOUT an
inherent impl in the crate**". **At step 1 there is no crate**, so the compiler
enforces nothing: `impl Class { … }` in `me`'s lib and a caller in `me`'s lib are
the same crate. My move compiles clean *while* `read_records` calls
`class.is_argv_forbidden()` and `class.is_bearer()` and names
`crate::sysw::record::Class` directly — the exact dependency I5 exists to catch.

So I5's check at step 1 is a **reading exercise wearing a step gate's clothes**,
and the plan's own N-C2 paragraph says reading is what let the earlier draft
through ("step 1's own check would have passed it"). The plan diagnosed the
failure mode and then re-assigned the check to the step that cannot perform it.

---

## Q3 — DOES `refuse_write_block` RETURNING A DECISION RIPPLE FURTHER?

**No. It is a 17-line change across 2 call sites, and nothing outside the 11
depends on the `i32`.** I implemented it to be sure, then reverted it.

Call sites, exhaustively:

```
$ grep -n "refuse_write_block(" crates/me-cli/src/io.rs crates/me-cli/src/main.rs
crates/me-cli/src/io.rs:163:pub fn refuse_write_block(b: WriteBlock, len: usize) -> Option<i32> {
crates/me-cli/src/io.rs:452:    if let Some(code) = refuse_write_block(      # inside `emit`
crates/me-cli/src/main.rs:969:                if let Some(code) = refuse_write_block(   # inside `run_sysw`'s pack arm
```

With `refuse_write_block -> Option<WriteRefusal>` and both call sites mapping the
decision onto `EXIT_USAGE`:

```
$ cargo nextest run --locked
     Summary [  12.216s] 388 tests run: 388 passed, 1 skipped
$ diff -u io.step1.rs io.rs   | grep -c '^[+-][^+-]'   → 14
$ diff -u main.step1.rs main.rs | grep -c '^[+-][^+-]' →  3
```

Both call sites already consumed the `i32` as the enclosing function's own
return (`emit -> i32`, `run_sysw -> i32`), and both enclosing signatures are
unchanged. No test asserts `refuse_write_block`'s return type; the exit codes are
asserted end-to-end by spawn tests, which are indifferent.

**But the change does not do what §3 says it does — see I-1 below.** It is
cheap, and it is inert.

---

## Q4 — DOES THE STATED GATE HOLD?

**The test count: YES, exactly as stated.**

```
BASELINE (2757e80, untouched):
     Summary [  14.912s] 388 tests run: 388 passed, 1 skipped
AFTER THE MOVE:
     Summary [  12.785s] 388 tests run: 388 passed, 1 skipped
```

The 1 skipped is `sysw::vectors::tests::regenerate`
(`#[ignore = "regenerates the fixture; run deliberately"]`,
`crates/me-cli/src/sysw/vectors.rs:128`) — deliberate, not a gap.

`cargo clippy --all-targets` exits 0 with no warnings, before and after.
`cargo fmt --all --check` is **red on the untouched baseline** (65 diffs, nightly
rustfmt 1.9.0) — pre-existing, not a step-1 finding, and no CI workflow runs it.

**The line count: NO.** `main.rs` went **2226 → 1765 = 461 lines**, not ~431.

**The pty assertion (I1): it passes, and the plan is right that nothing else
covers it.** All 12 tests in `world_readable_output.rs` reach stdout through
`--out`, a `File`, a pipe, a fifo or `/dev/null`; none uses a pty. Run against
the moved binary:

```
$ /usr/bin/script -qec "$ME sysw pack --no-passphrase text:6869" /dev/null
me: stdout is a TERMINAL, and this payload is BEARER.
…
script_exit=2
```

### The move was verified differentially, not by the suite alone

I built both binaries (pre-move `7c63c531…`, post-move `05648fda…` — confirmed
distinct) and ran 11 probes against each, covering every arm of the closure:

| probe | rc | note |
| --- | --- | --- |
| pty terminal refusal | 2 | F-253 arm, unreachable by the suite |
| `>` onto a 0644 file | 2 | mode=644 measured, F-252 arm |
| `tx:` on argv | 3 | prefix arm |
| `ms1` on argv | 3 | class arm |
| empty `--in` file | 2 | R7, `--in` channel |
| empty stdin | 2 | R7, stdin channel |
| `--out` | 0 | mode 0600, payload sha equal |
| `> /dev/null` | 0 | char-device exemption intact |
| blank lines in `--in` | 0 | payload sha equal |
| `--allow-argv-secret` | 4 | override honoured (gets past the guard) |
| stdin-is-a-TTY hint | 2 | `read_records`'s own `eprintln!` |

`diff before.txt after.txt` is **empty apart from three `mktemp -d` paths**.
Every gate fired — none of these is a false pass.

---

## WHERE THE PLAN IS WRONG

### CRITICAL

**C-1. Step 1 cannot be executed without publishing `EXIT_OK` / `EXIT_USAGE` /
`EXIT_REFUSED` into the shared half — and §1, §3-I5 and §4-step-1 all omit
this.** Detail above. It is not a style choice: `main.rs` items are invisible to
`lib.rs`, and step 1 forbids the signature changes that would avoid it. The plan
spends a page (I2) on why "the crate publishes NO shared numeric exit constant at
all", then schedules a first step that publishes three of them. Whoever executes
step 1 will either notice this and stop, or — far likelier, since the tests stay
green — commit `pub const EXIT_USAGE: i32 = 2;` into the donor's public API and
carry it forward. **That is C1's shape in a third file**, and unlike the mask it
arrives *before* the step meant to guard against it.

**C-2. `read_records` is a second SPLIT, and §1 says there is exactly one.**
§1: *"ONE OF THE 11 IS A SPLIT, NOT A MOVE — and it is the whole of C1."*
§3's table then rules that *"`read_records`'s class-keyed arm"* **stays in `me`**.
Measured: that arm is **80 of `read_records`'s 131 lines — 61 %**. The largest
function in the closure, the one §2.4 calls *"where the seam actually pays"*, is
majority-stays-behind. Two of the 11 are splits, and the plan's risk framing —
which puts the entire split risk on the mask — is calibrated to one.

### IMPORTANT

**I-1. §3's I2 remedy is aimed at a function §3's own table keeps in `me`, so it
proves nothing.** §3: *"So `refuse_write_block` returns the DECISION, not
`Some(i32)` — that signature change belongs to P0, and it is what makes this
boundary real rather than asserted."* Four paragraphs later, the same section's
table: *"**stays in `me`** | **every `refuse_*`** — `refuse_write_block`, …
(N-I1)."* A function that stays in `me` returning `me`'s own `EXIT_USAGE`
crosses no boundary at all. Measured (Q3): the change is 17 lines, 2 call sites,
and one of those call sites is **inside `emit`, which the table also keeps in
`me`** — so after the change `EXIT_USAGE` is still named in exactly the same
two places.

Meanwhile the functions that **do** carry an exit integer across the boundary the
table draws are `no_records_guard` and `read_records`, both returning
`Result<Vec<String>, (String, i32)>` — **4 of the closure's 8 `EXIT_*`
references** — and §3's table places `no_records_guard` in `records.rs`, inside
the crate. I2 does not mention either. The remedy is applied to the safe
function and withheld from the two that need it.

**I-2. §3-I5's type/constant enumeration is 3 right, 1 wrong, 7 missing.** Table
in Q2. `Admission` is named and has **zero** references in the 464 lines
(`grep -c 'Admission'` → 0 in both slices); `TX_PREFIX`, `classify`,
`REGION_ADDR`, `REGION_LEN` and the three `EXIT_*` are unnamed. This is the
paragraph that exists *because* §1 "enumerates functions only, which understates
it" — the correction has the same defect as the thing it corrects.

**I-3. `read_records` is given no home in the table §3 says exists so a reader
can tell where each function lands (M6).** Walking the table: `destination`,
`stdout_world_readable_mode` + stub, `split_record_stream`, `no_records_guard`,
`write_block`, `refuse_write_block`, `refuse_terminal_destination`,
`refuse_world_readable_stdout`, `emit`, `write_private` — **ten**. The eleventh,
`read_records`, appears only as *"`read_records`'s class-keyed arm"* in a
stays-in-`me` row. The other 51 lines have no assigned file.

**I-4. §3's N-I1 justification is factually wrong: only 2 of the 4 `eprintln!`
live in the `refuse_*` functions.** §3: *"All four `eprintln!` in the closure
live in them."* Measured, per containing function:

```
  line 206: inside fn refuse_terminal_destination
  line 229: inside fn refuse_world_readable_stdout
  line 422: inside fn read_records      ← the stdin-is-a-TTY hint
  line 474: inside fn emit              ← "me: {e}" on a failed write
```

This matters because it is the *stated reason* the `refuse_*` functions stay in
`me`, and because the surviving violation is `read_records` — whose argv
machinery §3 puts in `records.rs`, i.e. **inside the crate**, still writing to
process stderr unconditionally. §3's ruling (*"a library six binaries share must
not write to stdio unconditionally … functions return what should be said; the
caller emits it"*) is therefore not satisfied by the table that claims to satisfy
it. The probe confirms the hint really fires (Q4 probe 11, rc=2).

**I-5. §1's line-count table does not reproduce under any rule I can
construct — and its total is the number step 1's gate is written against.**
Recomputed from the source, both with and without doc comments and attributes:

```
function                          plan  code-only  with-doc
write_private                       40         21        34
stdout_world_readable_mode          25         23        41
destination                         31          9        12
write_block                         21         19        27
refuse_write_block                  34         13        14
refuse_terminal_destination         31         30        50
refuse_world_readable_stdout        19         17        17
split_record_stream                 29          6        10
no_records_guard                    25         22        44
read_records                       132        131       133
emit                                44         43        43
TOTAL                              431        334       425
```

`split_record_stream` is 6 lines of code / 10 with its doc comment; the table
says 29. `destination` is 9 / 12; the table says 31 — the same number it gives
`refuse_terminal_destination`. This is the identical defect §2.4 already
retracted once as M4 (*"not reproducible under any rule"*), left standing in the
table that produces 431.

### MINOR

**M-1. The step-1 gate's shrink figure is wrong by ~7 %.** ~431 claimed;
**461 measured** (2226 → 1765). The contiguous slice actually moved is 464 lines
— the 431 excludes the `#[cfg(not(unix))]` stub, `enum Destination`,
`enum WriteBlock` and the blank separators, all of which the plan elsewhere says
must move (N2, I5). Recompute it, or drop the number from the gate.

**M-2. Step 1 necessarily edits `me`'s tests, and the plan's gate is worded as if
it does not.** The first `cargo nextest run` after the move failed:

```
error[E0432]: unresolved imports `super::destination`, `super::Destination`, `super::WriteBlock`
    --> crates/me-cli/src/main.rs:1703:17
     |
1703 |     use super::{destination, is_plate_artifact, write_block, Destination, WriteBlock};
```

Two of the 388 — `a_terminal_is_never_a_destination_for_the_container` and
`write_block_decides_both_gates_once` — live in `main.rs`'s own `#[cfg(test)] mod
tests` and reach the moved items through `super`. Step 1's gate is *"the 388 still
pass"*; §6 condition 1 is *"unchanged in meaning"*; step 7 alone carries the
"enumerate the diff to the tests" discipline. Step 1 needs that allowance too, or
the first executor of this plan discovers it as a compile error. (The edit is
one import line; the finding is that the plan does not anticipate it.)

**M-3. `refuse_terminal_destination`'s wording is `me`-hardware-specific, and
§3 says the crate holds "the wording of each refusal".** Its 30-line message
names `picotool load --verify`, `picotool erase`, BOOTSEL, and interpolates
`sysw::wire::REGION_ADDR` / `REGION_LEN` — the SeedHammer II flash region. Five of
the six consumers have no such region. Whatever `remedy.rs` holds, it cannot hold
this one; the plan should say so rather than let an implementer discover it.

**M-4. §I1's "all 12 tests … redirect to files" is loose.** Of the 12, three use
`Stdio::piped()`, one a named fifo, one `/dev/null`. The *conclusion* — none
reaches the terminal arm — is correct and I verified it; only the reason is
imprecise.

---

## WHAT DID NOT GO WRONG

Worth saying, because the plan is right about most of it:

- The **function** closure is exactly 11 + the stub. No 12th `fn`.
- The closure really is library-shaped: 0 `process::exit`, 0 `clap`, 0 `Cli`.
- `std::env::args` really is 0 — C2's diagnosis stands.
- M1's `eprintln!`/`println!` correction is right (4 / 0).
- The 388/1-skipped baseline is exact, and the skip is deliberate.
- Steps 1-then-2 ordering works: the mask moved intact, and nothing changed.
- The pty gap I1 names is real, and the assertion it prescribes passes.

## COMMANDS

```
cargo build                  # exit 0, no warnings
cargo clippy --all-targets   # exit 0, no warnings
cargo nextest run --locked   # 388 tests run: 388 passed, 1 skipped
/usr/bin/script -qec "$ME sysw pack --no-passphrase text:6869" /dev/null   # rc 2
```

All binaries were invoked by absolute path; no exit code was read through a pipe.
