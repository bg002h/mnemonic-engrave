# PROBE — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` steps 3 and 5c, EXECUTED

**Throwaway feasibility probe, 2026-08-26.** Worktree
`/scratch/code/shibboleth/_work/probe3/mnemonic-engrave`, branch
`probe/p0-steps35`. Built **inside `me`** — no `mnemonic-io-lib` crate was
created, per the brief. Nothing here is meant to be merged.

Downstream of `design/agent-reports/PROBE-P0-step1.md`, whose findings are taken
as settled and not re-derived.

---

## VERDICTS

- **TARGET A — step 3, `observation.rs`: BUILDABLE AS SPECIFIED. The plan's
  CLAIM about it is FALSE.** The types build in 99 lines and fix F-259. But
  **the F-259 bug can still be written**, against the strongest typed
  construction I could produce, with `cargo build` clean, `cargo clippy
  --all-targets` clean, and **403/403 tests passing**. "Cannot recur by
  construction" (§6 condition 6) is not true. See C-1.
- **TARGET B — step 5c, `--expect <kinds>`: BUILDABLE WITH DEVIATIONS.** All
  four questions answer YES on real records, and §10's acceptance command RUNS
  in both halves. Three deviations were forced: the `Admission` parameter the
  plan's table omits (**C-2, a false refusal on the funds path**), a second
  completeness walk the plan says is not needed (I-1), and the HRP discriminant
  being reachable only through a `pub(crate)` function whose `Ms` arm is
  `unreachable!()` (I-2).

```
     Summary [  11.749s] 403 tests run: 403 passed, 1 skipped
```

```
 crates/me-cli/src/lib.rs                     |   1 +
 crates/me-cli/src/main.rs                    | 142 ++++++++---
 crates/me-cli/src/observation.rs             |  99 ++++++++
 crates/me-cli/src/sysw/expect.rs             | 363 +++++++++++++++++++++++++++
 crates/me-cli/src/sysw/mod.rs                |   1 +
 crates/me-cli/tests/world_readable_output.rs |  62 +++++
 6 files changed, 638 insertions(+), 30 deletions(-)
```

Baseline 388 → 403. The 15 new tests are 3 in `observation.rs`, 10 in
`sysw/expect.rs`, and 2 pty tests in `world_readable_output.rs`. The 1 skipped is
`sysw::vectors::tests::regenerate`, deliberate, as before.

---

# TARGET A — step 3, the observation types

## What was built

`crates/me-cli/src/observation.rs`:

```rust
pub enum PayloadKind { Bearer, ScrubImage }      // F-259's half
pub struct ObservedMode(u32);                    // F-260's half
```

Then wired through the whole write gate, in the strongest form the plan's words
permit — the observation is carried **inside the decision**, not merely handed
to the function that makes it:

```rust
enum WriteBlock { None, Terminal(PayloadKind), WorldReadable(ObservedMode) }

fn write_block(out_given, kind: PayloadKind, allow_world_readable: bool,
               stdout_is_tty: bool, world_readable_mode: Option<ObservedMode>) -> WriteBlock
fn emit(bytes, out, kind: PayloadKind, allow_world_readable: bool) -> i32
fn refuse_terminal_destination(len: usize, kind: PayloadKind)
```

**F-259 is fixed by this.** Before and after, on a real pty:

```
$ /usr/bin/script -qec "$ME sysw wipe --fill zeros" /dev/null
BEFORE: rc=2  me: stdout is a TERMINAL, and this payload is BEARER.
AFTER:  rc=2  me: stdout is a TERMINAL, and this payload is a SCRUB IMAGE (it carries no secret).
```

Control, unchanged — a real container is still called bearer:

```
$ /usr/bin/script -qec "$ME sysw pack --no-passphrase text:6869" /dev/null
rc=2  me: stdout is a TERMINAL, and this payload is BEARER.
```

F-260's half is `mt`'s and out of P0's scope, but the mechanism was built and
unit-tested here: `ObservedMode::new(0o620).grants()` returns `"write to group"`
and asserts it contains no `"read"`; `0o644` returns
`"read to others, read to group"`.

## CAN THE F-259 BUG STILL BE WRITTEN? — YES. Three attempts, two succeed.

### Attempt 1 — the literal mechanism. **BLOCKED.**

Put the bool back in the kind's seat:

```rust
const WIPE_IMAGE_CARRIES_NO_SECRET: bool = true;
emit(&sysw::overwrite::region_image(f), out.as_ref(),
     WIPE_IMAGE_CARRIES_NO_SECRET, false)
```
```
error[E0308]: mismatched types
    --> crates/me-cli/src/main.rs:1398:17
1398 |                 WIPE_IMAGE_CARRIES_NO_SECRET,
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `PayloadKind`, found `bool`
```

This is the only attempt the plan's step-3 gate anticipates — *"a payload kind
cannot be constructed from a permission bool"*. It is also the least interesting
one: it is a type error, and nobody was going to write it twice.

### Attempt 2 — the PLAN-MINIMAL reading of §6 condition 6. **SUCCEEDS.**

§6 condition 6 says only *"a payload kind is a type"*. Satisfy exactly that:
`PayloadKind` exists, `emit` and `write_block` take it — and leave
`WriteBlock::Terminal` a unit variant, so the terminal arm never consults it:

```rust
Destination::Terminal => WriteBlock::Terminal,
// `kind` IS consulted here, so rustc does not warn it is unused.
Destination::Stream => match world_readable_mode {
    Some(mode) if !allow_world_readable && kind == PayloadKind::Bearer => {
        WriteBlock::WorldReadable(mode)
    }
    _ => WriteBlock::None,
},
```

```
cargo build                → clean
cargo clippy --all-targets → clean, no warnings
cargo nextest run --locked → 391 tests run: 391 passed, 1 skipped
$ /usr/bin/script -qec "$ME sysw wipe --fill zeros" /dev/null
rc=2  me: stdout is a TERMINAL, and this payload is BEARER.
```

**F-259, verbatim, with the type in place.** This is not a strawman: it is
the *original bug's exact shape*, with a type substituted for the bool. The
original `allow_world_readable` was likewise used in the Stream arm and ignored
in the Terminal arm — which is why there was no compiler signal then either.

### Attempt 3 — against the STRONG construction. **ALSO SUCCEEDS.**

Keep `WriteBlock::Terminal(PayloadKind)` — the version that actually fixes
F-259 — and discard the carried kind at the consumer:

```rust
WriteBlock::Terminal(_) => {
    refuse_terminal_destination(len, PayloadKind::Bearer);
    Some(EXIT_USAGE)
}
```

```
cargo build                → clean
cargo clippy --all-targets → clean, no warnings
cargo nextest run --locked → 391 tests run: 391 passed, 1 skipped
$ /usr/bin/script -qec "$ME sysw wipe --fill zeros" /dev/null
rc=2  me: stdout is a TERMINAL, and this payload is BEARER.
```

`_` in a pattern is a legal, idiomatic, warning-free way to throw the
observation away. There is **no construction in Rust that forces a caller to
read a value it has been handed.**

### What DOES catch it

Only an assertion on the emitted words, through a pty. I added two to
`crates/me-cli/tests/world_readable_output.rs`:

- `a_wipe_image_refused_at_a_terminal_is_not_called_bearer` — asserts the
  terminal arm fired **and** that `BEARER` is absent.
- `a_container_refused_at_a_terminal_is_still_called_bearer` — the control,
  without which deleting the word entirely would pass the first.

Mutation-checked, not assumed. Re-applying attempt 3 with those tests present:

```
     Summary 2 tests run: 1 passed, 1 failed, 392 skipped
        FAIL mnemonic-engrave::world_readable_output a_wipe_image_refused_at_a_terminal_is_not_called_bearer
```

The control passed while the finding test failed — so the pair discriminates.

---

# TARGET B — step 5c, `--expect <kinds>`

Built as `crates/me-cli/src/sysw/expect.rs` (363 lines with tests) plus a
`--expect <KINDS>` flag on `me sysw pack`. Vocabulary exactly as plan §3's
table: `descriptor` by HRP `'d'`, `cosigner` by HRP `'k'`, `transaction` =
`Class::Mt` ∪ `Class::Tx`, and `mnemonic`/`secret`/`passphrase` by `Class`.

All commands below used the binary by **absolute path**; no exit code was read
through a pipe. Records are the repo's own vectors — `MD1_A/B/C` (a complete
3-chunk set, `chunk_set_id` 398802), `MK1_A/B` (a complete 2-chunk set), and
`EVEN` (mt-codec's pinned 6-chunk `mt1` corpus vector).

## Q1 — does the HRP discriminant work on real records? **YES.**

The premise first, in a unit test: `classify(MD1_A)` and `classify(MK1_A)` are
**both** `Class::MdMk`, and `kind_of` separates them into `Descriptor` and
`Cosigner`. So the plan's N-C1 ruling is not theoretical — `Class` genuinely
cannot do this and the HRP genuinely can.

End to end. The baseline is the C-1 defect itself:

```
### md1 + mt1, NO mk1, no --expect  (the defect §6g exists for)
rc=0  payload=783 bytes           ← a backup missing its cosigner card, exit 0

### same stream, --expect descriptor,cosigner
rc=4  payload=NONE
me: --expect named cosigner, and NO record of that kind is in the stream. A
    producer that refused leaves exactly this behind, and the pipeline still
    exited 0.
me: nothing was written.

### CONTROL — complete stream, --expect descriptor,cosigner,transaction
rc=0  payload=976 bytes
```

## Q2 — does `--expect descriptor,transaction` refuse a stream with no transaction? **YES.**

```
$ me sysw pack --no-passphrase --in no-tx.txt --expect descriptor,transaction --out payload.bin
rc=4  payload=NONE
me: --expect named transaction, and NO record of that kind is in the stream. …
me: nothing was written.
```

## Q3 — does it refuse an INCOMPLETE `md1` set (§6g)? **YES.**

Two of the three chunks of set 398802, alongside a complete `mk1` set and a
complete `mt1` set:

```
### with --expect
rc=4  payload=NONE
me: --expect named descriptor, and the descriptor card in this stream is
    INCOMPLETE — record(s) [0, 1] (counting from 0) do not reassemble. A
    partial set cannot restore anything.
me: nothing was written.

### BASELINE, same stream, no --expect  — §6g's reproduction, still live
rc=0  payload=908 bytes
me: record 0, as given (records count from 0): an md1/mk1 this tool could not
    decode; the device will treat it as a SECRET
me: record 1, as given (records count from 0): …
```

## Q4 — is `transaction = Class::Mt ∪ Class::Tx` sufficient? **YES — both forms, on real records.**

```
### the mt1 form: all 6 EVEN chunks classify Class::Mt, --expect transaction satisfied
rc=0  payload=579 bytes

### the `tx:` / --qr form §10's acceptance command actually uses
$ me sysw pack --no-passphrase --in <md1 + mk1 + tx:…> \
      --expect descriptor,cosigner,transaction --out payload.bin
rc=0  payload=896 bytes

### an INCOMPLETE mt1 set (3 of 6) must NOT satisfy it
rc=4  payload=NONE
me: --expect named transaction, and the transaction card in this stream is
    INCOMPLETE — record(s) [5, 6, 7] (counting from 0) do not reassemble. …
```

The union is necessary and it is enough. But see **C-2**: `Class::Tx` is not a
property of the record alone, and the plan's table is written as if it were.

## §10's acceptance command, RUN — both halves

Positive, in §10's own brace-group shape (producers replaced by `cat` of their
outputs, since `md`/`mk`/`mt` producer surface is P3's):

```
$ { cat wallet.md1; cat cosigner.mk1; cat tx.mt1; } \
    | me sysw pack --no-passphrase --expect descriptor,cosigner,transaction --out payload.bin
rc=0  payload=976 bytes   digest 855a 337d 56d6 3556 c4ac 22db 46ca 27b5
```

Negative — §10's second half, *"the same pipeline with one producer made to
refuse must FAIL and write no payload"*:

```
$ { cat wallet.md1; true; cat tx.mt1; } \
    | me sysw pack --no-passphrase --expect descriptor,cosigner,transaction --out payload.bin
rc=4  payload=NONE
me: --expect named cosigner, and NO record of that kind is in the stream. …
```

Both halves pass. **§10's acceptance criterion is satisfiable.**

---

# WHERE THE PLAN IS WRONG

## CRITICAL

### C-1. §6 condition 6 is false: F-259 CAN recur with the observation types in place, and the step-3 gate cannot see it.

Evidence in full above. Two of three attempts reproduce F-259 verbatim with
`cargo build` clean, `cargo clippy --all-targets` clean and the entire suite
green. Attempt 3 defeats the *strongest* construction, by writing `_`.

The plan's step-3 gate is *"a payload kind cannot be constructed from a
permission bool"*. That gate is satisfied by attempt 2 and attempt 3 — it tests
only attempt 1. So the plan's own gate passes while the defect it exists to
close is live and on a pty.

**What "by construction" actually buys is one thing, and it is worth having:**
the *smuggling* mechanism dies (a bool cannot enter the kind's seat). What it
does not buy is the *false message*, which is what F-259 IS — the follow-up says
so in its own words: *"The refusal is arguably RIGHT; the stated reason is
FALSE."* A type prevents a value being confused for another value. It cannot
prevent a value being ignored.

**The correct wording, and the correct gate.** Condition 6 should say the
message is **asserted**, not that it is impossible: step 3's gate must be a pty
assertion on the emitted words with a positive control, of the shape added here
and mutation-checked here. The plan already knows this technique — I1 prescribes
exactly it for step 1 — and then does not carry it into step 3, which is the one
step whose subject is *what the refusal says*.

### C-2. The `--expect` vocabulary table omits `Admission`, and the omission produces a FALSE REFUSAL with a FALSE MESSAGE on the transaction path.

The plan's table: `transaction | Class::Mt ∪ Class::Tx`. But `Class::Tx` is not
a function of the record — `classify_with(record, adm)` demotes a `tx:` record
with an unsigned input to `Class::Unknown` unless `--allow-unsigned-inputs` is
set (`crates/me-cli/src/sysw/mod.rs:198`). `classify()` is
`classify_with(record, Admission::default())`.

Built exactly as the table says, resolving through `classify()`:

```
### CONTROL — the same invocation WITHOUT --expect
$ me sysw pack --no-passphrase --allow-unsigned-inputs --in all-unsigned.txt --out payload.bin
rc=0  payload=678 bytes
me: WARNING — record 5 … input 0 … passed --allow-unsigned-inputs.

### WITH --expect, built from the plan's table
$ me sysw pack --no-passphrase --allow-unsigned-inputs --in all-unsigned.txt \
      --expect descriptor,cosigner,transaction --out payload.bin
rc=4  payload=NONE
me: --expect named transaction, and NO record of that kind is in the stream.
```

The record is right there; `pack` accepts it and writes 678 bytes without the
flag. **`--expect` refuses it and says it is not in the stream.** That is a
false refusal on the funds path, and the message asserts something untrue about
the operator's data — **C-1's shape, in the feature this plan is adding.**

The fix is one parameter — `unmet(records, want, adm)` resolving through
`classify_with` — and it is in the probe's build. Verified after:

```
rc=0  payload=678 bytes                      # with --allow-unsigned-inputs
rc=4  "…NO record of that kind…"             # without it, still refuses
```

The plan says the vocabulary *"must map onto exactly one of those two
discriminants per kind"*. There is a **third** input — the admission — and no
row of the table mentions it.

## IMPORTANT

### I-1. "It does not need a second walk" is wrong twice, and the second way loses the mt1 case entirely.

Plan §3, quoting §6g: *"`mdmk_unconfirmed` already computes the incomplete-set
predicate the third bullet needs. `--expect` escalates its report to a refusal
for named kinds; it does not need a second walk, and a second walk would be a
second thing to drift."*

**(a) The HRP never leaves it.** `mdmk_unconfirmed(records: &[String]) ->
Vec<usize>` returns indices and nothing else
(`crates/me-cli/src/sysw/record.rs:168`). It groups by `(hrp, csid, uniq)`
internally and discards the HRP. Since `descriptor` and `cosigner` are
**defined** by that character, `--expect` must re-derive it — a second walk, in
the probe's `kind_of`/`mdmk_hrp`. Every `md1`/`mk1` record has its card identity
decoded twice per invocation.

**(b) It is blind to `transaction`, which is a named kind.** Its first statement
is `if super::classify(r) != Class::MdMk { continue; }`, and an `mt1` chunk is
`Class::Mt`. Measured, not read — `sysw::expect::probe_second_walk::mdmk_unconfirmed_is_blind_to_an_incomplete_mt1_set`:

```
mdmk_unconfirmed(3 of the 6 EVEN chunks) == []        ← "nothing wrong here"
mt_unconfirmed  (the same 3)             == [0, 1, 2]
```

So the incomplete-set half of §6g needs `sysw::mt::mt_unconfirmed`
(`crates/me-cli/src/sysw/mt.rs:207`) as well — a **third** walk, named in
neither the plan nor §6g. An implementer who follows the sentence literally
ships an `--expect transaction` that passes a half-transmitted transaction as
complete, which is §6g's own failure mode surviving inside §6g's own remedy.

### I-2. The HRP discriminant is reachable only through a function whose `Ms` arm is `unreachable!()`, and `me`'s defensive wrapper for exactly that is module-private.

`--expect` needs `chunk_key`. There are two:

- `crate::seal::record::chunk_key` — `pub(crate)`, reachable. Its last arm is
  `RecordKind::Ms => unreachable!("secret records are refused by the caller")`
  (`crates/me-cli/src/seal/record.rs:282`).
- `crate::sysw::record::chunk_key` — the defensive wrapper that maps `Ms` to
  `None`, and it is a **private `fn` in `sysw/record.rs`**
  (`crates/me-cli/src/sysw/record.rs:236`), unreachable from a sibling module.

`sysw/record.rs`'s own comment says why the wrapper exists: *"an `ms1` reaching
it is a disagreement between `classify` and `validate_record`, not an
impossibility. Delegating that arm would turn a defensive `None` into a panic on
the device."*

So the natural `--expect` implementation — the one this probe wrote — calls the
panicking version. Not reachable today (both routes go through the same
`validate_record`, so they cannot disagree), which is precisely why nothing
will catch it later. The plan mandates the HRP discriminant and says nothing
about which `chunk_key` provides it or that the safe one must be made
reachable first.

### I-3. The `passphrase` row of the vocabulary table contradicts itself, and the kind it produces is unsatisfiable on the ordinary path.

The table: `passphrase | Class::Passphrase | flag-declared only`. Those two
cells disagree. `Class::Passphrase` is produced by a `pass:` **record**;
"flag-declared" describes `--passphrase-words` / `--passphrase-ask`, which
create the container's sealing passphrase and put **no record in the stream**.

Measured:

```
$ me sysw pack --passphrase-words 12 --in all.txt \
      --expect descriptor,cosigner,transaction,passphrase --out payload.bin
rc=4  payload=NONE
me: --expect named passphrase, and NO record of that kind is in the stream. …

### CONTROL — a pass: record DOES satisfy it
$ me sysw pack --no-passphrase --in md-pass.txt --expect descriptor,passphrase --out payload.bin
rc=0  payload=275 bytes
```

This is the trap the plan explicitly rules out one paragraph earlier, for
`address`: *"A kind that can never be satisfied is worse than an absent one: it
turns a gate into a permanent refusal."* `passphrase` is that kind for every
operator who declares one with a flag. Either drop the row or state that it
checks for a `pass:` record and says nothing about sealing.

## MINOR

### M-1. The plan never says which exit code `--expect` uses, and §6f publishes three with scripted meanings.

`me`'s vocabulary is 2 usage / 3 policy refusal / 4 invalid input, and §6f's
whole argument is that *"every site returns one of these BY NAME"* because
scripts read them. `--expect` is a new refusal site and the plan assigns it
nothing. The probe chose **4** (well-formed invocation, input is not what the
operator said), and **2** for an unknown kind word. Either is defensible; the
plan should rule it, because §10's acceptance turns on the group's exit status.

### M-2. `--expect`'s position relative to the F-246 write gate is unruled, and it changes which refusal the operator sees.

`main.rs:1186` states at length that the write gate's position is load-bearing
and that getting it wrong pre-empted the argv refusal. `--expect` is a new gate
in that same window. Placed after `read_records` (the probe's choice), it
pre-empts the write gate:

```
### terminal/world-readable stdout AND a missing cosigner
rc=4, --expect fires first        (the probe's placement)
```

Both are refusals so nothing unsafe follows, but the plan inherits a section
that says this ordering must be decided deliberately, and does not decide it.

### M-3. Splitting the F-259 bool into a type without carrying its OTHER meaning regresses `wipe`, and the plan's step-3 description does not mention that the bool had two consumers.

`WIPE_IMAGE_CARRIES_NO_SECRET` in the `allow_world_readable` seat was doing real
work in the arm it *did* reach: it exempted a scrub image from the mode gate,
and there is a shipped test for it —
`does_not_refuse_a_wipe_image_which_carries_no_secret`
(`crates/me-cli/tests/world_readable_output.rs:200`). The obvious split —
`emit(&image, out, PayloadKind::ScrubImage, false)` — fails it:

```
FAIL [0.005s] (366/391) mnemonic-engrave::world_readable_output does_not_refuse_a_wipe_image_which_carries_no_secret
```

The fix is for the Stream arm to consult the kind too
(`… && kind == PayloadKind::Bearer`), which the probe's build does. Worth
stating in the plan, because F-259's write-up frames the bool as a fact the
terminal arm *never received*, which reads as though it were doing nothing
elsewhere.

---

## WHAT THE PLAN GOT RIGHT

Stated plainly, because most of it holds:

- **N-C1 is correct and load-bearing.** `Class::MdMk` really does cover both
  cards; the HRP really does separate them; `--expect descriptor,cosigner` on
  `md1`-only records really does refuse. The funds case §6g names is real and
  the remedy works.
- **The `transaction` union is right, and both halves are needed.** A real
  6-chunk `mt1` set and the `tx:` `--qr` form both satisfy it, and §10's command
  uses the second.
- **§6g's incomplete-set requirement is satisfiable**, for `md1`, `mk1` and
  `mt1`, and the reproduction it cites is still live at exit 0 without
  `--expect`.
- **§10's acceptance command RUNS**, positive and negative halves.
- **`address` is correctly excluded.** Verified: `classify` never returns
  `Class::Descriptor` or `Class::Address` (0 occurrences in
  `crates/me-cli/src/sysw/mod.rs`), and an address record is refused at rc=4
  with the quoted message.
- **The observation types are cheap and they DO fix F-259.** 99 lines, and the
  pty message goes from a false "BEARER" to a true "SCRUB IMAGE". The plan is
  right that this is the shape of the fix; only its claim about what the shape
  guarantees is wrong.

---

## COMMANDS

```
cargo build                    # exit 0, no warnings
cargo clippy --all-targets     # exit 0, no warnings
cargo nextest run --locked     # 403 tests run: 403 passed, 1 skipped
/usr/bin/script -qec "$ME sysw wipe --fill zeros" /dev/null
$ME sysw pack --no-passphrase --in <records> --expect <kinds> --out <file>
```

Every binary was invoked by absolute path. No exit code was read through a pipe.
