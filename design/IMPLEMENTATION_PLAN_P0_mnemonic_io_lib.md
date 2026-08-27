# IMPLEMENTATION PLAN — P0: `mnemonic-io-lib`, the shared IO + safety crate

**Status:** DRAFT v1, written 2026-08-26. **NOT reviewed.** No code may be
written until this closes an R0 round at 0C/0I.

**Source spec:** `SPEC_constellation_cli_uniformity.md` §5a (the crate and its
four boundary lines), §5b (the four verbs), §6b/§6d/§6f/§6h (the rules being
hoisted), §7 P0 (this phase's row and gate), **and §8 — *"What is NOT verified,
and must be before the plan is trusted"* (M7)**, whose items this plan
inherits rather than supersedes.

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

**`me-cli` has both `lib.rs` and `main.rs`** — as does `mnemonic-toolkit`
(M2: an earlier draft called `me-cli` the only such crate, from a search that
looked at the toolkit's repo root, where its crate does not live). That reads
like a head start on extraction. **It is not.**

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

**ONE OF THE 11 IS A SPLIT, NOT A MOVE — and it is the whole of C1.**
`stdout_world_readable_mode` is not mechanism. It applies **`& 0o044` INSIDE the
function** (`crates/me-cli/src/main.rs:912`) and returns `Option<u32>` — a
verdict, not a measurement. Moving it intact would publish `me`'s mask as the
crate's *mechanism* at the irreversible step, after which a later phase could
delete `mt`'s stricter refusal while citing a real §6 ruling. **The risk is not
that the two masks keep disagreeing; it is that they get reconciled silently in
favour of the weaker one, on the path where the artifact is cut into metal.**

**The closure is library-shaped.** Measured across all 431 lines: **0** hits for
`std::process::exit`, the `Cli` struct, or `clap`. Nothing reaches for the
binary's argument parser or for process exit, so the move is mechanical.

**But `std::env::args` is 0 too, and that is a GAP, not an achievement (C2).**
§6d requires the argv guard to run **before `Cli::parse()`**, and a pre-parser
guard is *defined* by reading raw argv. Zero hits means **the closure does not
contain the guard the spec assigns to P0** — §7 P0's row says *"argv guard with
pre-parser ordering"* and §6d says *"Both layers run pre-parser (C-4)"*. An
earlier draft of this plan cited the 0 as evidence the work was easy. It is the
symptom of the work being absent.

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
> taking `allow: bool` and `form: crate::blocks::Form`), whose own mask is at
> **line 653**. **Line 585 is NOT the guard** — it carries the same `0o077`
> inside a different function returning `None` rather than a refusal, and an
> earlier draft named it here as one of the guard's two sites (M3). They are written without the usual `path:line`
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

132 lines, the largest by 3×, and **only 9 of them touch IO at all**
(`std::fs`, `std::io`, `stdin`, `File::`, `read_to_string`, `BufRead`). Its argv
gate performs no IO whatsoever, and it currently has **13 spawn tests and zero
unit tests**. **This is where the seam actually pays.**

*(An earlier draft claimed a "100 pure / 32 mechanism" split, transcribed from a
design report without recomputation. It is not reproducible under any rule —
M4. The 9-line figure is measured and the command is above.)*

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
  src/fd.rs           — MECHANISM only: see the contract below.
  src/observation.rs  — what was measured, as types (§2.3)
  src/records.rs      — record stream splitting, the argv gate, kind vocabulary
  src/exit.rs         — refusal DECISION types + wording + ordering. NO integers.
  src/remedy.rs       — purge/remedy text, FROM `me` ALONE (§6h)
```

**`mt`'s purge text is NOT a source.** It advises zsh operators `history -d`,
which does not delete on zsh 5.9.2, and tells fish operators to match on the
bearer material — typing the secret into history a second time.

**`records.rs` — the PRE-PARSER argv guard (C2).** Two layers, both running on
raw `std::env::args()` **before `Cli::parse()`**:

1. **FLAG-NAME.** Match known secret-bearing flag names as strings. The names are
   static, so this needs no parse. `mnemonic-toolkit` already proves the shape
   with `NodeType::is_argv_secret_bearing` and a lockstep parity test — **P0
   adopts that shape rather than inventing one.**
2. **VALUE-SHAPE, additive.** For material arriving positionally where no flag
   declares it: `tx:` by prefix, `mt1`/`ms1` by HRP, a BIP-39 mnemonic by
   wordlist. `mt` and `me` have this today and it stays.

**The ordering is NORMATIVE, and `mt`'s source records why.** When the check
lived inside the `encode` subcommand, clap rejected the unexpected positional
first — **and clap's error echoed the entire bearer transaction to stderr.** A
guard downstream of the parser has already lost. **The override's own parse must
also run pre-parser**, or `--allow-argv-secret` cannot be honoured without
parsing the very argv the guard exists to protect.

**`me` does not currently leak this way** — verified 2026-08-26,
`me sysw pack --nosuchflag <ms1…>` exits 2 naming only the flag, with the secret
absent from stderr. **That is the behaviour to preserve, and it is not the same
as having the pre-parser guard**; P0 must not regress it while adding one.

**`--expect` — THE KIND VOCABULARY, enumerated (C3).** `me sysw pack --expect
<kinds>` does not exist yet and is P0's content:

| kind | resolved by | note |
| --- | --- | --- |
| `descriptor` | **HRP `'d'`** (`md1`) | **NOT by `Class`** — see below |
| `cosigner` | **HRP `'k'`** (`mk1`) | **NOT by `Class`** — see below |
| `transaction` | `Class::Mt` ∪ `Class::Tx` | the union is deliberate |
| `mnemonic` | `Class::Mnemonic` | BIP-39 wordlist |
| `secret` | `Class::Codex32Secret` | `ms1` HRP |
| `passphrase` | `Class::Passphrase` | flag-declared only |

**`descriptor` and `cosigner` MUST NOT resolve through `Class` (N-C1).** `me`'s
`Class` has a **single `MdMk` variant covering both**, so a `Class`-keyed
`--expect descriptor,cosigner` cannot tell a descriptor card from a cosigner
card — and that is the funds case §6g exists for: `--expect descriptor` would be
satisfied by the `md1` records alone, so **a refusing `mk encode` still yields
exit 0 with the cosigner card missing**, and the operator believes a backup is
complete when it is not.

The discriminant is one level down and P0 uses it: `mdmk_unconfirmed` already
groups by `(hrp, chunk_set_id)` and switches on the HRP character — **`'d'`
reassembles through `md_codec`, `'k'` through `mk_codec`.** An earlier draft of
this table mapped `descriptor` to `Descriptor`+`MdMk` jointly and omitted
`cosigner` entirely, which made §10's acceptance command
`--expect descriptor,cosigner,transaction` **unsatisfiable** — and §10 says it
must be RUN.

**`address` is NOT in the vocabulary, deliberately.** `Class::Address` and
`Class::Descriptor` are never produced by `classify` — `me sysw pack` refuses an
address record at **rc=4**, *"Descriptors and addresses are not yet classifiable
here"*. A kind that can never be satisfied is worse than an absent one: it turns
a gate into a permanent refusal.

`Class` has ten variants — `Mnemonic`, `Codex32Secret`, `Passphrase`,
`FreeText`, `Descriptor`, `MdMk`, `Mt`, `Tx`, `Address`, `Unknown`. **`FreeText`
and `Unknown` are deliberately unnameable**: `--expect` states what must be
present, and neither can be required of a stream.

**`exit.rs` HAS C1's PROBLEM IN A SECOND FILE (I2), and it is worse there.**
Calling it "the exit-code table" implies one table. Measured on the same
invalid-artifact input: **`md` 1, `mk` 2, `ms` 1, `mnemonic` 2 (by input shape)**,
and for a clap usage error **`md` 2, `mk` 64, `ms` 64, `mt` 2, `me` 2**. The
binaries do not agree, §9b rules the usage-code split **out of scope**, and §6f
rules `mk`'s invalid-artifact 2 → 1 the only change this cycle makes. **So the
crate publishes NO shared numeric exit constant at all.**

**A CONSTANT IS A MAPPING, and "meanings, not a table" does not separate them
(I2).** `refuse_write_block` returns `Some(EXIT_USAGE)` where `EXIT_USAGE = 2`
(`crates/me-cli/src/main.rs:296`) — **the donor's number**. `ms` maps clap errors
to **64** deliberately, with a comment citing its own spec's carve-out. So a
published `EXIT_USAGE = 2` leaves `ms` two choices at P2: adopt it, which §9b
rules out of scope; or ignore it, which is round 0's decorative table verbatim.
**Enumerating the constants is what exposed this** — §6f's table shows the five
binaries agree on almost nothing.

| the crate holds | the crate does NOT hold |
| --- | --- |
| the refusal *decision* types (`WriteBlock` and kin) | any `EXIT_*` integer |
| the wording of each refusal | any binary→code mapping |
| the ordering rule — which gate outranks which | a "usage error" number |

Each binary maps a decision onto its own code. **So `refuse_write_block` returns
the DECISION, not `Some(i32)`** — that signature change belongs to P0, and it is
what makes this boundary real rather than asserted. Publishing an integer would
do to the exit codes exactly what publishing `0o044` would do to the mask.

**`fd.rs`'s CONTRACT, stated because "no policy" is not self-explaining:**

- Return the **raw `mode & 0o777`** for a regular file. **No disqualifying mask
  anywhere in the crate** — not `0o044`, not `0o077`.
- Return `None` for a **character device**, and `None` on a **failed `fstat`**
  (fail-open).

**Those two `None`s ARE shared mechanism, and saying so is load-bearing.** Both
binaries already implement both, identically, comment sentences included —
`me` (`crates/me-cli/src/main.rs:906-917`) and `mt` — in the sibling repo, at
**lines 645 to 655** of its `mt-cli` validate module — the guard's own mask is
at **line 653**, not 585; 585 carries the same `0o077` inside a *different*
function that returns `None` rather than a refusal (M3) — deliberately written
without citation punctuation per the cross-repo note in §2.1, and verified by
hand 2026-08-26. An implementer who reads "no policy" literally would push the
char-device exemption out to callers, where it is **load-bearing for `/dev/null`
(mode 0666)** and where a caller can forget it. Fail-open is the same: *unreadable
stdout is not evidence of exposure*, and both tools say so in those words.

**The mask is the only part that is not shared**, and it is the only part that
stays behind.

**THE 11 MAPPED ONTO THE FILES (M6)** — §3 lists seven files and §4 sequences
six; without this table a reader cannot tell where a function lands:

| file | functions |
| --- | --- |
| `channel.rs` | `destination` — classification only (N-I1) |
| `fd.rs` | `stdout_world_readable_mode` (**split**, C1), its `cfg(not(unix))` stub |
| `records.rs` | `split_record_stream`, `no_records_guard`, the string-level recognisers (prefix / HRP / wordlist), and the pre-parser argv machinery |
| `observation.rs` | the payload-kind and mode types (F-259, F-260) |
| `exit.rs` | `write_block` — the DECISION only (N-I1) |
| `remedy.rs` | the purge/remedy text |
| `lib.rs` | re-exports only |
| **stays in `me`** | `is_secret`, `is_bearer`, `is_argv_forbidden`, and `read_records`'s class-keyed arm (N-C2) |
| **stays in `me`** | **every `refuse_*`** — `refuse_write_block`, `refuse_terminal_destination`, `refuse_world_readable_stdout` (N-I1) |
| *(caller-side)* | `emit`, `write_private` — see below |

**THE `refuse_*` FUNCTIONS STAY IN `me` TOO, and for the same reason as §3's
stdio ruling (N-I1).** All four `eprintln!` in the closure live in them. An
earlier draft of this table mapped three of the four into the crate while §3,
one page above, ruled that **functions return what should be said and the caller
emits it** — the plan contradicting itself one table apart, which is N-C2's
shape a third time.

**The split is decision from announcement.** `write_block` decides; `destination`
classifies; the crate holds those and the *wording*. Announcing is the caller's,
because a library six binaries share cannot be tested without capturing process
stdio and cannot be redirected by a caller — doubly wrong in a crate whose whole
purpose is controlling what reaches stdout.

**THE THREE `Class` PREDICATES CANNOT MOVE — this is a language rule, not a
preference (N-C2).** `is_secret`, `is_bearer` and `is_argv_forbidden` are
**inherent methods** on `me`'s `Class` (`impl Class { … }`,
`crates/me-cli/src/sysw/record.rs:65`). A different crate cannot define an
inherent impl for a foreign type. Reproduced in a two-crate scratch project:

```
error[E0116]: cannot define inherent `impl` for a type outside of the
              crate where the type is defined
```

**An earlier draft of this table put all three inside the crate**, which both
fails to compile *and* contradicts §5a's boundary line four paragraphs above it
— *record classes, prefixes and payload grammar stay with `me`*. Worse, **step
1's own check would have passed it**: the earlier wording asked only whether a
type was moved or public, and `Class` *is* public — in `me`. The cheapest edit that makes such
a build succeed is to drag `me`'s container vocabulary into the shared crate at
the irreversible step, with step 7's gate written to accept it under a citation.
**That is C1's shape, one file over.**

**So the split is by REPRESENTATION.** The crate's recognisers work on strings —
a `tx:` prefix, an HRP character, a BIP-39 word — and return the crate's **own**
kind type. `me` maps that kind onto its `Class` and keeps the three predicates.
Nothing in the crate ever names a `Class` variant.

**`emit` and `write_private` stay in `me` for now.** `emit` writes the payload
and `write_private` creates the 0600 file; both are the *act*, not the decision,
and P0's value is in the decisions. Moving them is P1's question, not P0's.

**TYPES AND CONSTANTS ARE PART OF THE CLOSURE TOO (I5).** §1 enumerates
functions only, which understates it: `WriteBlock`, `Destination`, `Class`, and
the `Admission` flags all cross with them. **Step 1 must enumerate every type and
constant the 11 reference and confirm each is either moved or **reachable
WITHOUT an inherent impl in the crate** — "already public" is NOT sufficient
and is precisely what let N-C2 through: `Class` is public, and an inherent
impl on it still cannot compile outside `me`** —
a function that compiles only because its enum is in scope has not been moved,
it has been copied into a file that happens to see it.

**Boundary lines, from §5a.** No display grouping (`mnemonic-toolkit` already
owns it, and the four encoders' copies are checksum-gated). No record classes,
prefixes or payload grammar — those stay with `me` per §9a. Describing a
*measurement* is not owning the *grammar*.

**The stdio question, answered.** The 431 lines contain **4 `eprintln!` and ZERO
bare `println!`** (M1 — an earlier draft said 4 of each; the grep was matching
`println!` as a substring of `eprintln!`). **The closure's only write to stdout
is `emit`'s payload `write_all`, and that must STAY a write** — it is the
payload itself, not a message. A library six binaries share must not write to stdio
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
| 1 | move the 11 into `me`'s lib half **intact — including the mask**, no behaviour change | `me`'s existing tests still pass — **388 RUN, 1 skipped, out of 389 `#[test]` attributes (N1); the gate is the run count, not the attribute count**; `main.rs` shrinks by ~431 lines. **PLUS a pty assertion pinning the terminal arm (I1)** — see below |
| 2 | `fd.rs` — **SPLIT** `stdout_world_readable_mode`: the crate returns the raw mode, `me`'s call site regains `& 0o044` | `fd.rs` returns `Some(0o644)` for a 0644 regular file **and `Some(0o620)` for a 0620 one** — a masked implementation cannot do the second; `/dev/null` returns `None`; `me`'s end-to-end behaviour is **still** unchanged |
| 3 | `observation.rs` — types | a payload kind cannot be constructed from a permission bool (**F-259 cannot recur**) |
| 4 | `remedy.rs` | zsh remedy does **not** contain `history -d`; fish remedy does **not** contain the secret |
| 5 | `records.rs` layer 1 — **pre-parser** flag-name guard on raw argv | **`me` ships NO secret-bearing flag**, so the RED gate cannot be an end-to-end refusal in the donor. It is a unit test on the crate's flag-name table plus a lockstep parity assertion against `mnemonic-toolkit`'s `NodeType::is_argv_secret_bearing`, whose flags DO exist |
| 5b | `records.rs` layer 2 — value-shape, additive | the argv gate refuses by class, with the override, **as unit tests** (§2.4); `me sysw pack --nosuchflag <ms1…>` still does not echo the secret |
| 5c | `--expect <kinds>` — the flag and the vocabulary | `--expect descriptor,transaction` **refuses a stream with no transaction**, and **refuses an incomplete `md1` set** (§6g). Both fail today: the flag does not exist |
| 6 | `channel.rs` + `exit.rs` | `--out` overwrites; `-` reads stdin; codes match §6f |
| 7 | `me` consumes the crate | all 388 tests pass, **with the diff to them enumerated and each edit justified by a named finding** — the shape §7 P1 already uses for `mt` |
| 8 | publish `0.1.0` | **irreversible — §5** |

**I1 — STEP 1's GATE CANNOT FAIL FOR THE TERMINAL ARM, so it is not left as the
only proof.** "The 388 still pass" is green whether or not the terminal refusal
survives the move — and the terminal arm is one of the 11, and the one carrying
F-259's funds-adjacent behaviour. **All 12 tests in
`crates/me-cli/tests/world_readable_output.rs` redirect to files, so none of them
reaches it.** Step 1 therefore carries a **pty assertion** pinning the refusal
(the repo already has the technique — `script -qec` reproduces it), and without
that assertion step 1 proves nothing about the terminal path.

**M5 — steps 1 and 7 do NOT have a test that must fail first, and the column
header should not claim they do.** Both are refactors whose gate is *"the suite
still passes"*, which is a regression gate, not a RED-first one. That is
legitimate for a move; asserting it is TDD when it is not would hide which steps
carry real proof. Steps 2, 3, 4, 5, 5b, 5c and 6 are RED-first; **1 and 7 are
regression-gated**, and step 1's pty assertion is the one RED-first thing in it.

**Steps 1 and 2 are ordered this way on purpose.** Step 1 moves
`stdout_world_readable_mode` *with* its `& 0o044` so nothing changes; step 2
then splits it, pushing the mask back to `me`'s call site. Behaviour is
unchanged at **both** steps, and at no point does a masked function sit inside
the crate. An earlier draft ordered step 1 to "move with no behaviour change"
and step 2 to hold "no policy assertion" — **which cannot both be true of the
same function**, and the reading that satisfied step 1 published `me`'s mask as
the crate's mechanism.

**Step 1 is not a refactor to skip.** It is the step that proves the closure is
really 11 and not more.

**The `#[cfg(not(unix))]` stub of `stdout_world_readable_mode`
(`crates/me-cli/src/main.rs:921`) is a 12th definition and moves with its twin**
(N2). It is named here because it appears in neither table above and a reader
counting definitions will find it.

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
5. **The §6h in-memory-history question MEASURED** (I3). An earlier draft of this
   condition offered an escape hatch — measure it, **or** declare it
   unanswerable — which weakened a spec gate the spec states without one. (The
   wording is described rather than quoted: quoting a retracted phrase re-mints
   it, which is how this cycle has re-created nine of them, every one found by
   re-running a sweep and none by re-reading.) **The escape
   is exactly the failure mode `remedy.rs` exists to prevent** — `history -d`
   reports success and purges nothing, and "recorded as unanswerable" is the
   documentation-shaped version of the same thing. **Step 4 therefore carries a
   POSITIVE test: run the emitted recipe under an interactive shell and assert
   the entry is gone**, not that a command was printed.
6. **F-259 and F-260 cannot recur by construction** — a payload kind is a type,
   and a permission message is derived from the observed mode.

   **This REQUIRES test and signature changes, and condition 1 must not be read
   as forbidding them (C4).** `emit` and `write_block` change signature: the
   payload-kind fact stops travelling in the `allow_world_readable` bool that
   caused F-259. **`write_block_decides_both_gates_once`
   (`crates/me-cli/src/main.rs:2201`) is expected to change** — F-259's own
   analysis says that test locks the defect in, so a fold that leaves it
   untouched has not closed the finding. "Unchanged in meaning" governs
   condition 1; it does not govern the tests this condition exists to fix.
7. **§8's `CLOSED`-grep discipline observed (M7).** §8 directs P0 not to read an
   absent `CLOSED` marker as open work, and to **grep both markers**. The plan
   cited §8 but never carried this item. Concretely: before scheduling any
   follow-up, `grep -n 'CLOSED\|DONE' design/FOLLOWUPS.md` for it — F-259 and
   F-260 were both filed the day this plan was written, so the practical risk is
   near nil today and rises with every week that passes.
8. **The guard AND the override's own parse are both decided before
   `Cli::parse()`**, asserted at least in the donor (C2). A guard that reaches
   its decision by parsing first has reintroduced the leak §6d exists to stop.
9. An R0 round closing **0C/0I**.

---

## 7. OUT OF SCOPE

- **The §5c verb migration.** `split`, `combine`, `compile`, `derive` and
  `address` move to the toolkit — **decided, not scheduled**. D7 holds: this
  cycle relocates nothing. Every verb keeps its home through P0–P4.
- **Settling the `0o044` vs `0o077` disagreement.** §2.2. Each binary keeps its
  policy; the crate holds the mechanism. **And adoption never changes a
  consumer's mask** — a later phase that deletes or widens `mt`'s `0o077`
  refusal while citing a §6 uniformity ruling is out of scope for this cycle
  and must be refused at review, not accepted as tidying. Changing what either
  tool treats as a dangerous destination is a RULING, never a refactor.
- **`mnemonic-toolkit`'s own adoption.** It is the sixth consumer, not P0's
  work.
- **F-260 — REASSIGNED FROM P0 TO P1 (I4).** `mt`'s message calls mode 0620 one
  that *"grants read to group or others"* when no read bit is set outside owner.
  It was filed against P0, but **P0 does not touch `mt`** — §7 places `mt`'s
  adoption in P1, and a P0 that edited `mt`'s message would contradict its own
  scope. Under the constellation rule an item whose owning phase has passed is
  **overdue, not deferred**, so this is re-assigned rather than left to drift:
  **P1 owns it**, and `FOLLOWUPS.md` is updated in this same fold rather than
  later. F-259 stays with P0 — it is `me`'s, and `observation.rs` is where it is
  prevented.
- Anything the nine prior spec rounds closed.
