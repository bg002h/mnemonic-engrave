# DESIGN-io-seam — attack on the policy/mechanism split for `mnemonic-io-lib`

**Date:** 2026-08-26
**Worktree:** `/scratch/code/shibboleth/_work/seam/mnemonic-engrave` @ `990f75a` (branch `design/io-seam`)
**Object:** the 11 functions in `crates/me-cli/src/main.rs` named in the brief. The spec was not read.
**Task:** construct a counterexample to the policy/mechanism seam, or report failing to.

---

## VERDICT

**SOUND WITH AMENDMENTS** — the cut *line* is in the right place, but the proposal has the
two halves' value backwards: **policy is the part the binaries already disagree about, and
mechanism is the part that is byte-identical.** Putting policy in the shared crate ships six
divergent rules under one name while leaving the actually-duplicated `unsafe` fd code
copy-pasted.

---

## COUNTEREXAMPLE 1 (strongest) — the second consumer already implements this rule, and diverges from `me` on the POLICY side while matching it on the MECHANISM side

The proposal's load-bearing claim is that policy is *"the part all six binaries must agree
on."* `mt` is one of the six. It shipped its own version of this exact gate, and it is
`crates/mt-cli/src/validate.rs:627`:

```rust
pub fn world_readable_stdout_guard(
    allow: bool,
    form: crate::blocks::Form,
) -> Result<(), Refusal> {
    if allow { return Ok(()); }
    #[cfg(unix)]
    {
        use std::mem::ManuallyDrop;
        use std::os::unix::fs::PermissionsExt;
        use std::os::unix::io::FromRawFd;
        // ManuallyDrop: fd 1 belongs to the process; dropping the File would
        // CLOSE stdout out from under the strings we are about to write.
        let f = unsafe { ManuallyDrop::new(std::fs::File::from_raw_fd(1)) };
        let md = match f.metadata() {
            Ok(md) => md,
            // Unreadable stdout is not evidence of exposure. Fail OPEN rather
            // than refusing for a reason we cannot state.
            Err(_) => return Ok(()),
        };
        use std::os::unix::fs::FileTypeExt;
        if md.file_type().is_char_device() { return Ok(()); }
        let mode = md.permissions().mode() & 0o777;
        if mode & 0o077 == 0 { return Ok(()); }
```

against `me`'s `crates/me-cli/src/main.rs:896`:

```rust
    let f = unsafe { ManuallyDrop::new(std::fs::File::from_raw_fd(1)) };
    match f.metadata() {
        Ok(md) => {
            if md.file_type().is_char_device() { return None; }
            let mode = md.permissions().mode() & 0o777;
            (mode & 0o044 != 0).then_some(mode)
        }
        Err(_) => None,
    }
```

The two are the **same code, including the same two comment sentences**, up to and including
`let mode = md.permissions().mode() & 0o777;`. They diverge on the very next line, and the
divergence is entirely policy:

| | `me` | `mt` |
|---|---|---|
| mask | `mode & 0o044` (read only) | `mode & 0o077` (read, **write, exec**) |
| terminal gate | refuses (`Destination::Terminal`) | none, deliberately — *"mt has no --out: stdout IS the strings, by design (§3b)"* |
| `--out` short-circuit | `destination(out_given, …) -> File` skips both gates | no `--out` exists; the parameter is unsupplyable |
| remedy text keyed on | `me`'s `REGION_ADDR`, `picotool`, BOOTSEL | `mt`'s `blocks::Form::{Strings,RawRecord}` |

**Machine-checked, with a control** (binaries by absolute path, `me` built from this worktree
at `target/debug/me`, `mt` at `/scratch/code/shibboleth/mnemonic-transaction/target/debug/mt`):

| stdout mode | `me sysw pack --no-passphrase text:68690a` | `mt encode --in psbt.txt` |
|---|---|---|
| **0620** | **exit 0, 63 bytes written** | **exit 1, REFUSED §8.2h "mode 0620"** |
| 0600 (control) | exit 0 | exit 0, 796 bytes |

Two constellation binaries, the same threat, the same source comments, **opposite answers on
the same file**. That is not a rule six binaries must agree on; it is a rule two binaries have
already been ruled to disagree about. A shared `is_world_readable(mode) -> bool` in
`mnemonic-io-lib` must either pick a mask and silently change one binary's shipped behaviour,
or take the mask as a parameter — at which point it is `mode & mask != 0` and the crate is
holding a bit test.

Note in passing: on 0620 `mt`'s message says *"its permissions grant read to group or
others"*, which is false — 0620 grants **write**. `mt`'s mask and `mt`'s message already
disagree with each other. Not in scope, but it is precisely the kind of reconciliation a
"shared policy" crate would have to adjudicate and has no basis to.

**What this does NOT refute:** the ~12 lines above the divergence are *identical in two
binaries today*. That is real duplication of genuinely tricky code (`from_raw_fd(1)` +
`ManuallyDrop`, char-device exemption, fail-open on `Err`), and it is the **mechanism**. The
seam is worth cutting; the shared half is the other one.

---

## COUNTEREXAMPLE 2 — where the seam makes something WORSE: it would freeze a live defect into the crate all six binaries import

`write_block`'s signature carries no notion of *what the payload is*. `me sysw wipe` needs
that fact, so it smuggles it through the flag-shaped parameter (`main.rs:1385`):

```rust
            // NOT GATED, and deliberately. `emit` is shared, so F-244's guard
            // reached `wipe` too -- and a wipe image is 65,536 bytes of
            // random/zeros/ones with NOTHING in it. Its purpose is to DESTROY a
            // payload, so it is the opposite of bearer: refusing it buys no
            // safety and costs the operator a working command. Caught by asking
            // what else the new guard would catch; there is a test.
            const WIPE_IMAGE_CARRIES_NO_SECRET: bool = true;
            emit(
                &sysw::overwrite::region_image(f),
                out.as_ref(),
                WIPE_IMAGE_CARRIES_NO_SECRET,
            )
```

It only half-works, because `allow_world_readable` does not reach the terminal arm:

```rust
    match destination(out_given, stdout_is_tty) {
        Destination::File => WriteBlock::None,
        Destination::Terminal => WriteBlock::Terminal,      // <-- allow_* never consulted
        Destination::Stream => match world_readable_mode {
            Some(mode) if !allow_world_readable => WriteBlock::WorldReadable(mode),
            _ => WriteBlock::None,
        },
    }
```

**Measured on a pty** (`script -qec "$ME sysw wipe --fill zeros" /dev/null`):

```
me: stdout is a TERMINAL, and this payload is BEARER.

Writing it here would paint 65536 bytes of raw binary across your scrollback ...

Give it a file, then flash that file:

  me sysw pack --region --out payload.bin  ...
```

exit **2**. The operator asked to *wipe*; they are told their wipe image is BEARER — the code
three files away declares in a named constant that it is not — and are handed the `pack`
command. (`picotool erase` does appear four lines later, so the message is not useless, only
false in its diagnosis and wrong in its lead remedy.)

**Nothing catches it.** `world_readable_output.rs:200
does_not_refuse_a_wipe_image_which_carries_no_secret` uses `Stdio::from(f)` — a real file, not
a tty — so the terminal arm never runs. And the unit test in `main.rs` *pins the wrong answer
as correct*:

```rust
        // A terminal is refused, and --allow-world-readable does NOT buy past
        // it: that flag is about a FILE's permissions.
        assert_eq!(write_block(false, false, true, None), W::Terminal);
        assert_eq!(write_block(false, true, true, None), W::Terminal);
```

This is the concrete "worse than leaving it" case the brief asked for. The proposed policy
side is defined as *"given a file mode, is it world-readable; what remedy text to emit"* —
payload class is not among its inputs. Extracting `write_block` as-is moves a function whose
signature is **already missing the parameter that would make it correct** into a crate that
six binaries import, and takes with it a passing unit test that asserts the defect. The rule
does not become two-things-to-keep-in-sync; it becomes one wrong thing with five more
consumers.

Within `me` alone there are already **three** policies over the one mechanism, and the third
is this bug:

1. `emit` / `sysw pack` — terminal-refuse + `0o044` + `--out` short-circuit.
2. the converter — `main.rs:433` calls `stdout_world_readable_mode()` and
   `refuse_world_readable_stdout()` **directly, bypassing `write_block` entirely**: mode gate
   only, no terminal gate. Verified: `me --in card.txt --stdout` on a pty writes raw NDEF into
   scrollback at exit 0. (Defensible — `md1` is watch-only, `Class::is_bearer` excludes
   `MdMk` — but it means `write_block`'s doc claim to be *"the single decision"* is true only
   within `sysw pack`.)
3. `sysw wipe` — intends no gate, gets the terminal gate.

Add `mt`'s and that is **four policies over one mechanism, across two of the six binaries.**

---

## `read_records` — disposition, argued from contents

**It does not belong to either side. It is a dispatcher with a 100-line pure policy function
inlined into one of its three branches, and the pure part has never been named.** Measured
spans (`main.rs`):

| part | lines | span | IO? |
|---|---|---|---|
| signature | 5 | 1921–1925 | — |
| `--in` branch | 6 | 1926–1931 | `std::fs::read_to_string` |
| **argv branch** | **100** (50 code, 50 comment) | **1932–2031** | **none** |
| stdin branch | 21 | 2032–2052 | `IsTerminal`, `eprintln!`, `read_to_string` |
| total | 132 | 1921–2052 | |

Grepping the argv branch for `std::fs|stdin|stdout|eprintln|println|File::|IsTerminal|
read_to_string` returns two hits, **both inside comments**. The branch calls only
`trim_start`, `to_ascii_lowercase`, `classify`, `format!`, and `return Err`. It is 100% pure,
returns `Result<_, (String, i32)>`, and constructs every byte of its refusal as a value rather
than printing it — it is already written in the shape the policy side wants.

So: **argv branch → policy** (`refuse_argv_records(argv, allow_argv_secret) -> Result<(),
(String,i32)>`); **`--in` + stdin branches + signature (32 lines) → mechanism.** The 132/431
figure that makes `read_records` look like the seam's hard case is an artifact of three
channels sharing one `fn`.

**This is where the seam actually pays, and it is not the write gate.** The argv gate is the
most safety-critical pure function in the extraction set — it is what stands between a seed
phrase and `/proc/<pid>/cmdline` — and it has **13 test functions in
`crates/me-cli/tests/sysw_cli.rs` and zero unit tests**: `a_tx_record_on_argv_is_refused`,
`argv_refuses_every_bearer_class`, `argv_refuses_every_secret_class_too`,
`the_argv_refusal_echoes_neither_the_transaction_nor_a_passphrase`,
`a_tx_record_anywhere_on_argv_is_refused_and_located`,
`the_argv_refusal_is_scoped_to_the_transaction_class`,
`a_bearer_record_on_argv_outranks_the_write_gate`, `allow_argv_secret_proceeds`,
`the_argv_refusal_names_the_command_that_purges_history`,
`argv_still_accepts_watch_only_and_free_text`, and three more. Every one spawns the binary to
exercise a function that touches no file descriptor. That is the measured motivation
(`12 tests, 13 spawns, 0 library calls`) applied to the function where it is actually true.

By contrast — and this is the amendment to the brief's own premise — the 12 spawns in
`world_readable_output.rs` do **not** convert to unit tests under this seam. Read what they
assert: `out_tightens_a_preexisting_world_readable_target` (chmod-after-open),
`refuses_a_world_readable_named_fifo`, `does_not_refuse_dev_null`, `does_not_refuse_a_pipe`,
`does_not_refuse_an_owner_only_stdout_redirect`. Every one is about a **real file type on a
real fd** — the part the proposal itself concedes cannot be abstracted. They stay spawns after
the seam. And the policy those spawns sit on top of is **already extracted and already
unit-tested** in `main.rs`'s `#[cfg(test)] mod tests` (`a_terminal_is_never_a_destination_for_
the_container`, `write_block_decides_both_gates_once`). The write gate has no testability debt
for this seam to pay off.

---

## Per-function disposition

| fn | lines | side | note |
|---|---|---|---|
| `read_records` | 132 | **does not split as a unit** | 100 pure / 32 mechanism, see above. Extract the argv gate; leave the dispatcher. |
| `emit` | 44 | mechanism | composition root: gathers `IsTerminal` + `stdout_world_readable_mode`, calls policy, writes. Stays in the binary. |
| `write_private` | 40 | mechanism | one policy constant (`0o600`) inside `OpenOptions`; splitting it out is a named constant, not a function. Genuinely shareable *as mechanism* — every binary that writes a secret wants exactly this open+chmod pair. |
| `refuse_write_block` | 34 | splits cheaply | `WriteBlock -> (String, i32)` policy + one `eprintln!`. Low cost, low value. |
| `destination` | 31 | policy | pure, already unit-tested. `out_given` is `me`-only (`mt` has no `--out`) — **no shared value.** |
| `refuse_terminal_destination` | 31 | **does not split** | see below. |
| `split_record_stream` | 29 (6 code) | policy | the **only** one of the 11 that is both pure and binary-agnostic. |
| `stdout_world_readable_mode` | 25 | **mechanism is the shareable half** | splits at exactly the line where `me` and `mt` diverge. Extract `stdout_file_mode() -> Option<u32>` (`None` = char device or unreadable); leave the mask with each binary. |
| `no_records_guard` | 25 | policy | pure. Message names `mt encode --qr > rec.txt` and `--in` — `me`-shaped, but portable-ish. |
| `write_block` | 21 | policy | pure, unit-tested, **and missing the payload-class parameter** — counterexample 2. |
| `refuse_world_readable_stdout` | 19 | **does not split** | see below. |

### The two that do not split: `refuse_terminal_destination` (31) and `refuse_world_readable_stdout` (19)

By the proposal's own definition these are 100% policy — *"what remedy text to emit"* — and
0% mechanism apart from a single `eprintln!`. Yet they cannot cross into a shared crate,
because the text is the `me` binary:

```rust
fn refuse_terminal_destination(len: usize) {
    use mnemonic_engrave::sysw::wire::{REGION_ADDR, REGION_LEN};
    ...
           me sysw pack --region --out payload.bin  ...\n\
           picotool load --verify payload.bin -t bin -o 0x{REGION_ADDR:08X}\n\
    ...
         Do NOT pipe into picotool: it sizes its input with fstat, a pipe \
         reports 0 bytes, and the load exits 0 having written nothing.",
        REGION_ADDR as usize + REGION_LEN
```

`mnemonic-io-lib` would have to depend on `me`'s `sysw::wire` constants — a shared library
importing from one of its consumers — or the remedy becomes an injected `&str`/closure
parameter, which is the "abstraction that costs more than it saves": the caller supplies the
whole message and the library supplies a `match` arm. `mt`'s counterpart has the same shape
from the other end, keyed on `blocks::Form` and containing the sentence *"mt has no --out:
stdout IS the {} , by design (§3b)"*. Neither remedy is portable, and remedy text is 50 of the
431 extracted lines before counting the ~35 message lines inside `read_records` and
`no_records_guard`.

This is the second-order version of counterexample 1: **remedy text is the least shareable
thing in the set, and the proposal lists it first among the policy examples.**

---

## The seam I would make instead

Not a different cut line — the same one, with the halves' destinations swapped and one
parameter added.

**`mnemonic-io-lib` holds (a) mechanism and (b) vocabulary. Policy stays per binary.**

- **(a) mechanism.** `stdout_file_mode() -> Option<u32>` (the `from_raw_fd(1)` +
  `ManuallyDrop` + char-device exemption + fail-open block, identical in `me` and `mt`
  today), and `write_private(path, bytes)` (open `0o600` + chmod-the-open-fd, F-244). ~40
  lines, `unsafe` in one place, one set of fifo//dev/null/pipe integration tests instead of
  six. This is the only code in the set that is *already duplicated*.
- **(b) vocabulary.** `Class`, `is_secret`, `is_bearer`, `is_argv_forbidden` — the three
  already in `sysw/record.rs` plus the enum. These *are* the part all six must agree on, and
  they were ruled so on **2026-08-26**: *"we want uniform behavior with secret bearing between
  ms1 and passwords and mt1 to the extent we can."* Plus the argv gate extracted out of
  `read_records`, which is pure, safety-critical, and consumes exactly this vocabulary.
- **(c) per-binary.** `write_block`, `destination`, both `refuse_*` printers, the masks. `me`
  keeps its terminal gate and `--out`; `mt` keeps `0o077` and no terminal gate.

**And `write_block` gains the input it is missing** — not `allow_world_readable: bool` doing
double duty, but the payload's class. Then `sysw wipe` says what it means instead of
`WIPE_IMAGE_CARRIES_NO_SECRET: bool = true`, the converter path stops needing to bypass
`write_block` to get "mode gate, no terminal gate", and the tty defect above is expressible.

### Evidence that distinguishes the two allocations

Not preference — three independent measurements, all pointing the same way:

1. **Duplication is in the mechanism.** `me:896` and `mt-cli/src/validate.rs:641–652` are the
   same statements *and the same comment sentences*. Nothing in the policy halves is
   duplicated anywhere.
2. **Divergence is in the policy, and it is deliberate.** `0o044` vs `0o077`, terminal gate vs
   none, `--out` vs no `--out` — each documented in its own repo as a ruling, not a bug. Run
   against a 0620 file the two binaries disagree, with a 0600 control passing in both.
3. **Three call sites in two binaries already reach for a parameter the proposed policy side
   does not have.** `me`'s wipe fakes it with a `const bool`; `me`'s converter avoids
   `write_block` entirely to get a different combination; `mt`'s guard takes `form` as its
   second argument. When three independent authors thread the same missing parameter three
   different ways, the axis that varies is *what the payload is* — and policy/mechanism is
   orthogonal to it.

---

## What I could not refute

The cut line itself. Every one of the 11 functions has an unambiguous policy/mechanism
boundary; none of them required inventing an abstraction to find it, and none of them needed
duplicated state to cross it. `stdout_world_readable_mode` is the closest call — the fail-open
`Err(_) => None` is a policy decision inside the mechanism's error path, conflating "no
metadata" with "not exposed" — but `io::Result<(bool, u32)>` splits it cleanly and I could not
make that cost more than it saves. **I did not find a function where policy and mechanism are
genuinely entangled.**

What I found is that the seam is right and its **justification** is measurably backwards, and
that extracting `write_block` as specified would multiply an existing defect by six.

---

## Findings, as a list

| # | sev | finding |
|---|---|---|
| 1 | **Important** | The proposal's premise — policy is "the part all six binaries must agree on" — is false against the only other consumer that has implemented it. `me` and `mt` diverge on mask, terminal gate, `--out`, and remedy keying, and give opposite answers on a mode-0620 file (measured, with control). The **mechanism** is what is duplicated, near-byte-identically including comments. Swap which half the shared crate holds. |
| 2 | **Important** | `write_block` lacks a payload-class input, so `sysw wipe` smuggles one through `allow_world_readable` (`const WIPE_IMAGE_CARRIES_NO_SECRET: bool = true`) and the terminal arm ignores it. Extracting it as specified freezes this into the shared crate, together with a unit test asserting the wrong answer. Add the class parameter before extracting. |
| 3 | **Important** (live defect, new — not in `design/FOLLOWUPS.md`) | `me sysw wipe --fill zeros` on a terminal exits 2 with *"this payload is BEARER"* for a 65,536-byte fill image the code itself declares carries no secret, and leads with `me sysw pack --region --out payload.bin` — the command the operator did not ask for. Invisible to all 12 tests in `world_readable_output.rs` (they redirect to files) and to both unit tests. |
| 4 | Minor | `write_block`'s doc calls itself *"the single decision, so the early check and `emit`'s cannot drift apart"*. True within `sysw pack`; `main.rs:433` (the converter) implements a second, `WriteBlock`-less policy over the same mechanism. Three policies in one binary, four counting `mt`. |
| 5 | Minor | `refuse_terminal_destination` and `refuse_world_readable_stdout` (50 lines) are pure policy that cannot leave the binary: the text imports `me`'s `REGION_ADDR`/`REGION_LEN` and names `picotool`. Remedy text is the least portable item in the set and the proposal lists it first. |
| 6 | Minor | `read_records` is 100 pure / 32 mechanism, not 132 of either. Its pure half is the argv gate — 13 spawn tests, 0 unit tests — and is where this seam actually pays. The 12 write-gate spawns do **not** become unit tests: they assert fifo//dev/null/pipe/chmod behaviour on real fds, and the policy under them is already unit-tested in `main.rs`. |
| 7 | Nit (other repo) | `mt`'s refusal on a 0620 file says *"its permissions grant read to group or others"*; 0620 grants **write**. `mt`'s own mask and message disagree — exactly the reconciliation a shared policy crate would inherit with no basis to settle. |

## Reproduction

```sh
# built in this worktree
cd /scratch/code/shibboleth/_work/seam/mnemonic-engrave && cargo build
ME=$PWD/target/debug/me
MT=/scratch/code/shibboleth/mnemonic-transaction/target/debug/mt   # already built

# Finding 3 — wipe at a terminal claims BEARER
script -qec "$ME sysw wipe --fill zeros; echo EXIT=\$?" /dev/null

# Finding 1 — a 0620 stdout: me accepts, mt refuses
touch /tmp/a && chmod 620 /tmp/a
$ME sysw pack --no-passphrase text:68690a > /tmp/a ; echo "me exit=$?"
python3 -c 'import json;print(json.load(open("/scratch/code/shibboleth/mnemonic-transaction/crates/mt-cli/tests/fixtures/p5_base.json"))["finalized_psbt_b64"])' > /tmp/p.txt
chmod 600 /tmp/p.txt; touch /tmp/b && chmod 620 /tmp/b
$MT encode --in /tmp/p.txt > /tmp/b ; echo "mt exit=$?"   # control: chmod 600 /tmp/b -> exit 0

# Finding 4 — converter has no terminal gate
printf 'md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3' > /tmp/c.txt
script -qec "$ME --in /tmp/c.txt --stdout; echo EXIT=\$?" /dev/null
```

No source file was modified. Nothing outside the worktree was written.
