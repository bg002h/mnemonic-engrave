# IMPLEMENTATION PLAN — P0: `mnemonic-io-lib`, the shared IO + safety crate

**Status:** DRAFT v1, written 2026-08-26. **NOT reviewed.** No code may be
written until this closes an R0 round at 0C/0I.

**Source spec:** `SPEC_constellation_cli_uniformity.md` §5a (the crate and its
four boundary lines), §5b (the four verbs), §6b/§6d/§6f/§6h (the rules being
hoisted), §7 P0 (this phase's row and gate).

**Prior art this plan is downstream of:**
`design/agent-reports/DESIGN-io-seam.md` — the review that inverted the seam.

---

## 0. Why this plan exists at all

Six binaries — `md`, `mk`, `ms`, `mt`, `me`, `mnemonic` — each solve the same
IO and safety problems their own way. `me` solves them most completely, so `me`
is the donor. The crate is **`mnemonic-io-lib`** (operator-approved 2026-08-26,
confirmed free on crates.io).

**What makes this phase risky is not the code. It is that the code already
exists twice and the two copies disagree** — see §2. Extracting the wrong half
would freeze a disagreement into a shared dependency.

---

## 1. THE INVENTORY — measured, not described

`me-cli` is the **only** crate in the constellation with both `lib.rs` and
`main.rs` (5 of 5 `-codec` crates are lib-only; 4 of the 5 `-cli` crates are
main-only). That reads like a head start on extraction. **It is not.**

**Six of the nine named IO/safety functions are in `main.rs` — the binary
half:**

| in `main.rs` (BINARY half) | lines | | in the lib half (`sysw/record.rs`) |
| --- | --- | --- | --- |
| `read_records` | 132 | | `is_secret` (`crates/me-cli/src/sysw/record.rs:73`) |
| `emit` | 44 | | `is_bearer` (`crates/me-cli/src/sysw/record.rs:89`) |
| `write_private` | 40 | | `is_argv_forbidden` (`crates/me-cli/src/sysw/record.rs:105`) |
| `refuse_write_block` | 34 | | |
| `destination` | 31 | | |
| `stdout_world_readable_mode` | 25 | | |
| `write_block` | 21 | | |

**And the closure is larger than the six.** Those six call **five more**
`main.rs`-local functions. Each must move or the extraction does not compile:

| function | lines |
| --- | --- |
| `refuse_terminal_destination` | 31 |
| `split_record_stream` | 29 |
| `no_records_guard` | 25 |
| `refuse_world_readable_stdout` | 19 |
| *(`emit`, already counted above)* | — |

**Total: 11 functions, 431 lines, 19% of `main.rs`'s 2,226.**

**The closure is library-shaped.** Measured across all 431 lines: **0** hits for
`std::process::exit`, the `Cli` struct, `clap`, or `std::env::args`. Nothing
reaches for the binary's argument parser or for process exit, so the move is
mechanical rather than a rewrite.

**STEP ZERO IS INSIDE `me`.** Nothing crosses a crate boundary until those 11
are a library. Treating this as "donate three functions" discovers the other
eight one compile error at a time.

---

## 2. THE SEAM — mechanism is shared, policy is not

**This section reverses the first draft of this plan, on evidence.** The
original assumption was that *policy* — which modes disqualify, what remedy to
offer — was the valuable half to share. It is backwards.

### 2.1 The measurement that settles it

`mt` already ships this rule as `world_readable_stdout_guard`.

> **CROSS-REPO NOTE — these two live in `mnemonic-transaction`, not here.**
> `crates/mt-cli/src/validate.rs`, at **line 627** (`world_readable_stdout_guard`,
> taking `allow: bool` and `form: crate::blocks::Form`) and **line 585** (the
> `if mode & 0o077 == 0` mask). They are written without the usual `path:line`
> punctuation on purpose: `plan-cite-check.sh` resolves only against this repo
> and the fork root, so a citation-shaped sibling path would be reported
> DANGLING forever, training a reader to skim the gate's output. **Both were
> verified by hand on 2026-08-26** and must be re-verified by hand when this
> plan is next folded. It is the
same code as `me`'s `stdout_world_readable_mode`, comments included, down to
extracting the mode. Then every policy decision diverges. Run with a valid
transaction on stdin:

| stdout mode | `me sysw pack` | `mt encode` |
| --- | --- | --- |
| 0600 — **control** | exit 0, 733 bytes | exit 0, 796 bytes |
| **0620** | **exit 0, 733 bytes written** | **exit 1, REFUSED** |

| | `me` | `mt` |
| --- | --- | --- |
| disqualifying mask | `0o044` — read bits only (`crates/me-cli/src/main.rs:912`) | `0o077` — every group/other bit — see the cross-repo note below |
| terminal gate | yes | none |
| `--out` | yes | none; stdout **is** the strings, by design |

**The control is load-bearing.** Both tools exit 0 at 0600 with the same input,
so the 0620 divergence is the mode and nothing else.

### 2.2 What follows

- **The mechanism is duplicated near-byte-identically.** `fstat` the fd, read
  `permissions().mode() & 0o777`, hand back the number. **That is what the crate
  holds.**
- **The policy is a deliberate disagreement.** `mt` refuses a group-*writable*
  destination because someone else could alter the strings before they are cut;
  `me` permits it. **`mt` is arguably right, and this plan does not settle
  it** — hoisting policy would force a decision neither tool has agreed to make,
  inside a shared dependency where the argument is hardest to have.

### 2.3 The crate DOES hold the vocabulary for what was measured

Two shipped defects are the argument, both found 2026-08-26, in two repos:

- **F-259** — `me sysw wipe --fill zeros` on a terminal exits 2 saying *"this
  payload is BEARER"* for a 65,536-byte zeros image the code itself declares
  carries no secret (`WIPE_IMAGE_CARRIES_NO_SECRET`, smuggled through the
  `allow_world_readable` parameter, which the terminal arm never consults).
- **F-260** — `mt encode` refuses mode 0620 saying its permissions *"grant read
  to group or others"*. `0620 & 0o044 == 0`. No read bit is set outside owner.

**Both are messages hard-coded to a rule's NAME rather than derived from the
observation.** A message computed from the observed mode cannot say "read" about
a write-only mode. A payload kind carried as a type cannot be read as a
permission override. **F-259 traces to a `bool` two callers read differently —
exactly what a shared type prevents.**

### 2.4 `read_records` is not what its line count suggests

132 lines, the largest by 3× — but it is **100 pure lines and 32 of
mechanism**. Its argv gate performs no IO at all, and it currently has **13
spawn tests and zero unit tests**. **This is where the seam actually pays.**

**And a correction to an earlier claim in this cycle:** the 12 tests in
`world_readable_output.rs` do **not** convert to unit tests. They exercise real
file-descriptor types, which is irreducibly a process-level concern. The seam
buys unit-testability for the *argv and record* half, not the *fd* half.

---

## 3. WHAT THE CRATE CONTAINS

```
mnemonic-io-lib/
  src/lib.rs          — re-exports; no logic
  src/channel.rs      — --in / --out / `-`, destination classification
  src/fd.rs           — MECHANISM: fstat, extract mode. No policy.
  src/observation.rs  — what was measured, as types (§2.3)
  src/records.rs      — record stream splitting, the argv gate, kind vocabulary
  src/exit.rs         — the exit-code table
  src/remedy.rs       — purge/remedy text, FROM `me` ALONE (§6h)
```

**`mt`'s purge text is NOT a source.** It advises zsh operators `history -d`,
which does not delete on zsh 5.9.2, and tells fish operators to match on the
bearer material — typing the secret into history a second time.

**Boundary lines, from §5a.** No display grouping (`mnemonic-toolkit` already
owns it, and the four encoders' copies are checksum-gated). No record classes,
prefixes or payload grammar — those stay with `me` per §9a. Describing a
*measurement* is not owning the *grammar*.

**The stdio question, answered.** The 431 lines contain 4 `eprintln!` and 4
`println!`. A library six binaries share must not write to stdio
unconditionally — it cannot be tested without capturing process stdio and a
caller cannot redirect it, which is doubly wrong in a crate whose purpose is
controlling what reaches stdout. **Functions return what should be said; the
caller emits it.** This is a decision, recorded here so it is not rediscovered
mid-implementation.

---

## 4. TDD ORDER

Each step is RED first. No step begins until the previous is green.

| # | step | the test that must fail first |
| --- | --- | --- |
| 1 | move the 11 into `me`'s lib half, no behaviour change | `me`'s existing 388 tests still pass; `main.rs` shrinks by ~431 lines |
| 2 | `fd.rs` — mechanism only | a mode is extracted from a real fd; **no** policy assertion |
| 3 | `observation.rs` — types | a payload kind cannot be constructed from a permission bool (**F-259 cannot recur**) |
| 4 | `remedy.rs` | zsh remedy does **not** contain `history -d`; fish remedy does **not** contain the secret |
| 5 | `records.rs` | the argv gate refuses by class, with the override, **as unit tests** (§2.4) |
| 6 | `channel.rs` + `exit.rs` | `--out` overwrites; `-` reads stdin; codes match §6f |
| 7 | `me` consumes the crate | all 388 tests still pass, unchanged |
| 8 | publish `0.1.0` | **irreversible — §5** |

**Step 1 is not a refactor to skip.** It is the step that proves the closure is
really 11 and not 12.

---

## 5. THE IRREVERSIBLE STEP

`cargo publish mnemonic-io-lib 0.1.0` cannot be undone. Before it:

```
curl -s -o /dev/null -w '%{http_code}' -A 'name-check' \
  https://crates.io/api/v1/crates/serde              # CONTROL — must be 200
curl ... -A 'name-check' .../mnemonic-io-lib         # must be 404
curl ... -A 'name-check' .../mnemonic_io_lib         # must be 404
```

**`-A` is mandatory and the control is the gate.** crates.io answers a
user-agent-less request with **403 for every name, taken or free** — so without
`-A` the check cannot distinguish `mnemonic-io-lib` from `serde`. **If the
control does not return 200, the request is not being answered and no 404
beneath it means anything.** Both spellings are checked because crates.io treats
`-` and `_` as colliding.

**A 404 is availability at a moment, not a reservation.** Re-run immediately
before publishing; do not trust a check from an earlier session.

**Publication is operator-gated.** This plan does not authorise it.

---

## 6. WHAT MUST BE TRUE TO CLOSE P0

1. All of `me`'s tests pass, unchanged in meaning.
2. **§5b's invariant**: `encode`, `decode`, `verify`, `inspect` present on `md`,
   `mk`, `ms`, `mt` — **16 checks**, passing as of 2026-08-26.
3. **§6f's `mnemonic` invalid-artifact cell re-measured under a verb that
   EXISTS** — `inspect`, not the non-existent `decode` whose 64 was clap's
   unrecognised-subcommand code. Expect **2** for a bad HRP, **1** for an `md1`
   HRP that fails to decode.
4. `--expect descriptor,transaction` refuses a stream missing a transaction,
   **and** refuses an incomplete `md1` set. Both asserted.
5. The §6h in-memory-history question measured, or explicitly recorded as
   unanswerable.
6. **F-259 and F-260 cannot recur by construction** — a payload kind is a type,
   and a permission message is derived from the observed mode.
7. An R0 round closing **0C/0I**.

---

## 7. OUT OF SCOPE

- **The §5c verb migration.** `split`, `combine`, `compile`, `derive` and
  `address` move to the toolkit — **decided, not scheduled**. D7 holds: this
  cycle relocates nothing. Every verb keeps its home through P0–P4.
- **Settling the `0o044` vs `0o077` disagreement.** §2.2. Each binary keeps its
  policy; the crate holds the mechanism.
- **`mnemonic-toolkit`'s own adoption.** It is the sixth consumer, not P0's
  work.
- Anything the nine prior spec rounds closed.
