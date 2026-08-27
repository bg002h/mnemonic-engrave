# IMPLEMENTATION PLAN — P0: `mnemonic-io-lib`, the shared IO + safety crate

**Status:** DRAFT v1, written 2026-08-26. **NOT reviewed.** No code may be
written until this closes an R0 round at 0C/0I.

**Gates this plan is checked by** — run each **separately** from the commit:
`scripts/plan-stepref-check.sh` (prose may not name a step number),
`scripts/plan-table-check.sh`, `scripts/plan-cite-check.sh`,
`scripts/fold-propagation-check.sh`.

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

**`read_records` STAYS WHOLE IN `me` — it is not a split, and an earlier draft
saying so was what made "one signature change" arithmetically false (round-2
C-2).** Its three `EXIT_*` references sit at `main.rs` lines **1928, 2026 and
2048**, while the class-keyed arm §3 would have kept runs roughly **1932–2030**
— so **two of the three fall OUTSIDE it**. Split it and the moving set carries
three references, not one, and Variant B collapses. Keep it whole and the moving
set carries exactly one. **The simpler reading is also the correct one.**

**And the closure is larger than the six.** Those six call **five more**
`main.rs`-local functions. Each must move or the extraction does not compile:

| function | lines |
| --- | --- |
| `refuse_terminal_destination` | 31 |
| `split_record_stream` | 29 |
| `no_records_guard` | 25 |
| `refuse_world_readable_stdout` | 19 |
| *(`emit`, already counted above)* | — |

**Total: 11 functions plus the `cfg(not(unix))` stub. NO LINE-COUNT GATE — three probes measured the shrink at 461, 459 and 442 depending on the order taken, so the number is ordering-dependent and was deleted rather than corrected** — and its
per-function figures above reproduce under no rule I can state
(`split_record_stream` is 6 lines of 10, not 29). **Treat the per-function
column as indicative and the 461 as measured**; the probe's `git diff --stat`
is the number that was observed rather than computed.

**EXACTLY ONE OF THE MOVING SET IS A SPLIT, and it is the whole of C1.**
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

**THE FIRST WORK IS INSIDE `me`.** Nothing crosses a crate boundary until those 11
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
unit tests**. **An earlier draft named this the place the seam paid off, and proposed
splitting it. That is retracted** — two of its three `EXIT_*` references fall
outside the arm a split would keep, so splitting it makes the moving set carry
three references instead of one and collapses the single-signature-change
result. **It stays whole.**

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

**§6f IS NOT SUFFICIENT AS THE CHANNEL/EXIT WORK'S AUTHORITY (probe, Critical).** Its `me` row
listed only `2` and `4` and **never mentioned `3`** — the policy-refusal code
`me` returns for a seed on argv, a `tx:` record, and the `ms1`-over-NFC refusal,
used **three** times in its source (`crates/me-cli/src/main.rs:407`, `:599`, `:2026` — an earlier draft said four). **An implementer mapping `ArgvSecret =>
EXIT_USAGE` would have CONFORMED to §6f, collapsed the seed-on-argv refusal into
the usage code, and passed the gate.** That distinction survives today only
because it lives in code nobody was being asked to preserve. §6f is corrected in
the same fold; the gate is now **differential against the current binary**, not
a match against a table.

**AND THE HARD PART HOLDS — the first clean structural result of this cycle.**
The probe built `exit.rs` carrying refusal decisions, wording and the write-gate
ordering with **zero integers**, in `me`'s lib half where `main.rs`'s constants
are compiler-invisible. `me` reproduced **every current exit code byte-for-byte**
— a 30-case differential matrix diffing to three hunks, all intended. **No
`From<Decision> for i32` crept back**; the mapping is a 12-line
`fn exit_code(&Refusal) -> i32` in the *binary*. So §6f and "no shared constant"
do not contradict, and I2's ruling is executable.

**`-` READS STDIN NOWHERE IN `me` TODAY** — five surfaces, four different exit
codes — and §6b's permissive wording, which lets an implementation take the
flag and do nothing with it, makes the compliant
implementation **silently lossy**: `… | me sysw pack --out b.bin - text:6869`
packs **1 record instead of 2, at exit 0**, on the artifact that gets cut into
metal. **`-` is implemented, not merely accepted.**

**THE REMEDY WORK'S ORIGINAL GATE WAS FALSE AGAINST THE CORRECT TEXT (probe, Critical).**
It read *"the zsh remedy does not contain `history -d`"*. `me`'s refusal **does**
contain that literal string — deliberately, in the sentence warning that it does
not delete (`crates/me-cli/src/main.rs:2017`). So the gate goes **RED against
the correct donor**, and the only way to make it green is to **delete the
warning** — re-creating the exact defect this plan cites to disqualify `mt`'s
text.

**The codebase already documented the trap**, in the donor's own test file:

> *"NOT `!err.contains("history -d")` — the message deliberately NAMES that
> command in order to warn against it, so the naive negative fails on the
> warning itself. The requirement is that it is never OFFERED."*
> — `crates/me-cli/tests/sysw_cli.rs:2080`

**The gate was written without reading it.** The requirement is *never offered*,
which is a different assertion from *never mentioned*.

The fish half — *"does not contain the secret"* — is a **tautology**: `me`'s
remedy is static text with the secret never in scope, so it passes for any
string. A gate that cannot fail is not a gate.

**Condition 5 IS satisfiable** — the probe wrote it, 8 tests, control and
mutation both verified — **and running it found a live defect in `me`**, filed
as **F-264**. That is the argument for condition 5 in one line: the test was
worth writing because it failed.

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

**THE INCOMPLETE-SET CHECK NEEDS THREE WALKS, NOT ONE (probe I-1).** An earlier
draft said `--expect` needs no second walk because `mdmk_unconfirmed` already
groups by `(hrp, chunk_set_id)`. **Wrong twice:** that walk discards the HRP, and
it is **blind to `mt1` entirely** — it filters on `Class::MdMk` and an `mt1`
chunk is `Class::Mt`. Measured, not read:

```
mdmk_unconfirmed(3 of the 6 EVEN chunks) == []        ← "nothing wrong here"
mt_unconfirmed  (the same 3)             == [0, 1, 2]
```

So §6g's incomplete-set half also needs `sysw::mt::mt_unconfirmed`
(`crates/me-cli/src/sysw/mt.rs:207`) — a **third** walk, named in neither this
plan nor §6g. **An implementer who follows the old sentence literally ships an
`--expect transaction` that passes a half-transmitted transaction as complete**
— §6g's own failure mode surviving inside §6g's own remedy.

**THE HRP IS NOT DIRECTLY REACHABLE (probe I-2).** It comes only through
`seal::record::chunk_key`, whose `Ms` arm is `unreachable!()`, and `me`'s
defensive wrapper around it is module-private. **P0 must widen that access
deliberately** — reaching for an `unreachable!()` arm from a new call site is
how a panic ships.

**THE `passphrase` ROW CONTRADICTS ITSELF (probe I-3)** and is unsatisfiable on
the flag path. `--expect passphrase` is therefore **removed from the
vocabulary**, on the same rule that removed `address`: a kind that cannot be
satisfied turns a gate into a permanent refusal.

**`--expect` MUST CONSULT `Admission`, and omitting it creates a false refusal
on the funds path (probe C-2).** Built exactly as this table specified,
`me sysw pack --allow-unsigned-inputs --expect transaction` **refuses at rc=4**
saying *"NO record of that kind is in the stream"* — for a record the **same
invocation packs at exit 0 without `--expect`**. A false refusal carrying a
false message, which is C-1's shape reproduced inside the feature P0 is adding.
**One parameter fixes it: the kind test takes the `Admission` flags.**

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
| **stays in `me`** | `is_secret`, `is_bearer`, `is_argv_forbidden`, and **`read_records` WHOLE** — not just its class-keyed arm (round-3 C-2) |
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
— *record classes, prefixes and payload grammar stay with `me`*. Worse, **the move's own check would have passed it**: the earlier wording asked only whether a
type was moved or public, and `Class` *is* public — in `me`. The cheapest edit that makes such
a build succeed is to drag `me`'s container vocabulary into the shared crate at
the irreversible step, with the adoption gate written to accept it under a citation.
**That is C1's shape, one file over.**

**So the split is by REPRESENTATION.** The crate's recognisers work on strings —
a `tx:` prefix, an HRP character, a BIP-39 word — and return the crate's **own**
kind type. `me` maps that kind onto its `Class` and keeps the three predicates.
Nothing in the crate ever names a `Class` variant.

**`emit` and `write_private` stay in `me` for now.** `emit` writes the payload
and `write_private` creates the 0600 file; both are the *act*, not the decision,
and P0's value is in the decisions. Moving them is P1's question, not P0's.

**TYPES AND CONSTANTS ARE PART OF THE CLOSURE TOO (I5) — and the probe measured
which.** An earlier draft guessed at this list, naming `Admission`, which has
**zero references in the 464 moved lines**, and omitting **the symbols that actually cross**. That list was measured against the
**rejected 464-line move** and named `EXIT_OK`/`EXIT_USAGE`/`EXIT_REFUSED` as
crossing — **under Variant B they do NOT cross**, and citing them here pointed
the implementer straight at the forbidden publish (round-2 I-1). **The move**
enumerates the real set against the five-function move, in both directions. **Guessing at a closure
is what this plan exists to replace.** **The move must enumerate every type and
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
| 1 | **`no_records_guard` returns `Result<Vec<String>, String>`** — the ONE signature change (Variant B). Nothing moves yet. | `me`'s **388 RUN, 388 passed, 1 skipped**, unchanged in meaning; the `EXIT_*` count inside `no_records_guard` goes **1 → 0** |
| 2 | **Move the five + the stub** — carrying a **pty assertion pinning the terminal refusal BEFORE and AFTER — asserting the exit DIGIT, not `!success()`**, because this is F-265's own site #1 (`refuse_write_block`'s Terminal arm, proven respellable 2→3 with 388/388 green), since none of the 12 `world_readable_output.rs` tests reach it and it could otherwise be lost with every gate green (round-4 C-2) — into `me`'s lib half: `destination`, `stdout_world_readable_mode` (+ its `cfg(not(unix))` twin), `split_record_stream`, `no_records_guard`, `write_block`. **`read_records`, `emit`, `write_private` and every `refuse_*` STAY.** | builds with **`grep -c 'pub const EXIT' main.rs` == 0** and **`grep -c 'EXIT_' <lib module>` == 0** — a published constant fails the step. **Plus a real test, not only greps** (round-4 I-1: `cargo build` + two greps are already green on the untouched tree, so they cannot fail). Callers enumerated in BOTH directions. **What actually breaks is the private `Destination`/`WriteBlock` enums and `main.rs`'s `use super::` in `mod tests`; the "four callers outside the closure" figure was measured against the REJECTED move, and both those functions stay** |
| 3 | `fd.rs` — **SPLIT** `stdout_world_readable_mode`: the crate returns the raw mode, `me`'s call site regains `& 0o044` | `fd.rs` returns `Some(0o644)` for a 0644 file **and `Some(0o620)` for a 0620 one** — a masked implementation cannot do the second; `/dev/null` → `None`; `me`'s behaviour still unchanged |
| 4 | `observation.rs` — the payload-kind type **and its pty assertion** | **the assertion is the gate, not the type**: `script -qec 'me sysw wipe --fill zeros'` must NOT emit the word BEARER, asserted on the **emitted words**, pinning the **exit digit** (F-265: `!success()` cannot fail here). Mutation-checked in both directions |
| 5 | `remedy.rs` | the zsh remedy never **OFFERS** `history -d` — it must still NAME it to warn against it; and the emitted recipe, **RUN under a real interactive zsh**, actually removes the entry. **Blocked on F-264** — see §6 condition 9 |
| 6 | `records.rs` layer 1 — **pre-parser** flag-name guard on raw argv, **AND `me`'s `--allow-argv-secret` moved off clap** | **the observable is that no `ms1` appears in stderr for an argv clap would otherwise reject** — that is what pre-parser ordering means from outside, and it is the only gate here whose whole content is an ordering claim (round-3 M-7). Asserted end-to-end in the donor, not only as a crate unit test (round-1 N-I3); toolkit parity against `NodeType::is_argv_secret_bearing` in addition, never instead. **`--allow-argv-secret` must still PARSE afterwards** — it is a clap field today (`crates/me-cli/src/main.rs:252`, consumed at `:1116` and `:1127`), and moving the decision off clap without leaving the flag declared turns a valid invocation into a usage error, regressing the very flag P0 is told not to regress |
| 7 | `records.rs` layer 2 — value-shape, additive | the argv gate refuses by class, with the override, **as unit tests**; `me sysw pack --nosuchflag <ms1…>` still does not echo the secret |
| 8 | `--expect <kinds>` — the flag and the vocabulary | `--expect descriptor,cosigner` **refuses an `md1`-only payload**; `--expect descriptor,transaction` refuses a stream with no transaction; **refuses an incomplete `md1` set AND an incomplete `mt1` set** (three walks, §6g); `--allow-unsigned-inputs --expect transaction` **does NOT falsely refuse** |
| 9 | **`exit.rs` FIRST, then `channel.rs`** | **`--out` overwrite is asserted where it LIVES — `write_private`, which stays in `me`** (round-3 M-3); `channel.rs` holds only `destination`, so gating the overwrite on it would gate nothing. **`-` is IMPLEMENTED**; every code `me` produces today reproduced **byte-for-byte**, differentially against the pre-change binary — not by matching a table |
| 9b | **CREATE `mnemonic-io-lib` and move the lib-half modules into it.** Until here everything lives in `me`'s lib half; this is the crate boundary, and it is a step because no earlier one created it (round-3 M-4). | the crate builds standalone; `me` depends on it by path; **no `EXIT_*` and no `Class` in it** |
| 10 | `me` consumes the crate | **the 388 pre-existing tests still pass, plus every test added along the way** — an earlier draft said "all 388", which is wrong by construction once the intervening work adds tests (round-3 M-5); **with the diff to them enumerated and each edit justified by a named finding** |
| 11 | publish `0.1.0` | **irreversible — §5. Operator-gated; this plan does not authorise it.** |

**THE MOVE'S GATE IS BLIND TO 5 OF 8 EXIT DECISIONS (probe C-2), AND THE PTY
ASSERTION MUST PIN THE DIGIT.** A control run against the **unmodified** binary
showed `me` today can respell five refusals from **2 to 3** with every one of
388 tests still green — the lines proven to execute. So `!success()` is not
enough: **§4's pty assertion must assert the exit code itself**, or it misses
even the arm it is named for. (The hole is pre-existing, not introduced — filed
as **F-265**.)

**ENUMERATE CALLERS, NOT ONLY CALLEES** — the original closure was computed one
direction only. **The specific "four callers outside the closure" figure is
RETRACTED**: it was measured against the rejected 464-line move, and both
functions it named (`write_private`, `refuse_world_readable_stdout`) stay in
`me` under the adopted plan, so it contradicted the table's own cell twenty
lines below (round-5 I-1). What actually breaks the move is what that cell says:
the private `Destination`/`WriteBlock` enums, and the `use super::` in
`mod tests`.

**I1 — THE MOVE'S GATE CANNOT FAIL FOR THE TERMINAL ARM, so it is not left as the
only proof.** "The 388 still pass" is green whether or not the terminal refusal
survives the move — and the terminal arm is one of the 11, and the one carrying
F-259's funds-adjacent behaviour. **All 12 tests in
`crates/me-cli/tests/world_readable_output.rs` redirect to files, so none of them
reaches it.** **The move** therefore carries a **pty assertion** pinning the refusal
(the repo already has the technique — `script -qec` reproduces it), and without
that assertion the move proves nothing about the terminal path.

**M5 — TWO PIECES OF WORK ARE REGRESSION-GATED RATHER THAN RED-FIRST, and the
column header must not claim otherwise.** They are **the signature change** and
**the crate adoption**: both are refactors whose gate is *"the suite still
passes"*. That is legitimate for a refactor; asserting it is TDD when it is not
would hide which work carries real proof.

**Everything else is RED-first**, including the value-shape argv guard — an
earlier draft of this paragraph listed the RED-first work by number, omitted
that guard from the list, and so **de-gated it**. Naming work by number is what
made that mistake invisible. The table states each gate; this paragraph states
only the distinction.

**THE MOVE AS FIRST WRITTEN CANNOT BE DONE, and only executing it revealed why (probe
C-1).** The **four** exit constants are **private** in `main.rs`:

```
const EXIT_OK: i32 = 0;       const EXIT_USAGE: i32 = 2;
const EXIT_REFUSED: i32 = 3;  const EXIT_INVALID: i32 = 4;
```

**FOUR, not three** (`crates/me-cli/src/main.rs:295-298`). `EXIT_INVALID` is the code `--expect`'s
refusals return, and an earlier draft omitted it from every enumeration here
while leaning on those enumerations being exhaustive (round-3 M-1).

A lib module cannot see a binary's items, so moving the 11 into the lib half
**requires publishing them** — and an *"intact, no behaviour change"* move
forbids the signature change that would avoid it. So the move as first written
necessarily commits **`pub const EXIT_USAGE: i32 = 2`** into the donor's public
API: the exact thing §3 spends a page ruling out. **No ordering avoids it. It is
a language rule, like N-C2.**

**THE ORDERING IS SETTLED, AND THE ANSWER IS SMALLER THAN EITHER ATTEMPT.**
Three orderings were tried; a probe executed each.

| attempt | result |
| --- | --- |
| move intact, then split | **NO** — publishing `EXIT_*` is unavoidable |
| `refuse_write_block` returns a decision, then move | **NO** — it holds 2 of 8 refs; move still fails with 7 × `E0425` |
| **split exit-code production across four functions** | **YES** — 8 refs → 0, nothing published |
| **↳ and then: only ONE function needs it** | **ADOPTED** |

**The four-function version solves a one-function problem, and §3's own table
held the evidence.** Counted per function:

| §3 assigns to the crate | `EXIT_*` refs | | §3 keeps in `me` | refs |
| --- | --- | --- | --- | --- |
| `destination` | 0 | | `refuse_write_block` | 2 |
| `stdout_world_readable_mode` | 0 | | `emit` | 2 |
| `split_record_stream` | 0 | | `read_records` | 3 |
| `write_block` | 0 | | | |
| **`no_records_guard`** | **1** | | | |

**Seven of the eight live in functions that stay behind.** The moving set is
closed and holds exactly one, whose only callers stay in `me`.

**SO: ONE SIGNATURE CHANGE, NO NEW TYPE.**
`no_records_guard -> Result<Vec<String>, String>`. Executed and green: a
**160-line** move against 442, **8** public items against 15, and the decision
type deferred to the value-shape layer — the work that actually produces a second variant.
**A smaller correct answer beats a larger elegant one**, and the larger one was
about to be scheduled on the strength of a premise its own table refuted.

**The decision type is NOT inert** — that conclusion came from attempt 2, whose
type was inert *by construction* (both variants mapped to `EXIT_USAGE`). Here the
two-variant swap goes **RED by 9 tests**, with every mutation site proven to run
by observing the binary's exit code change.

**PROSE IN THIS PLAN NEVER NAMES A STEP NUMBER.** Twice now a fold has renumbered
one half and left the other asserting the old sequence — round 2 found the table
stale against the prose, round 3 found the prose stale against the table, **13
sites**, including a tie-break paragraph whose own example about the table had
become false. **The table is the only ordering of record**; rationale refers to
work by NAME — *the mask split*, *the move*, *the signature change* — so a
renumbering cannot falsify it.

**THE MASK SPLIT COMES AFTER THE MOVE, NOT BEFORE.** `stdout_world_readable_mode`
travels **with** its `& 0o044` so the move changes nothing, and the split then
pushes the mask back to `me`'s call site. Behaviour is unchanged at both points,
and **at no moment does a masked function sit inside the crate** — which is the
whole of C1. An earlier draft ordered the move to be "intact, no behaviour
change" and the split to hold "no policy assertion" *of the same function*,
which cannot both be true, and the reading that satisfied the first published
`me`'s mask as the crate's mechanism.

**The move is not a refactor to skip.** It is the step that proves the closure is
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
4. **`--expect` in full**: `descriptor,cosigner` refuses an `md1`-only payload;
   `descriptor,transaction` refuses a stream missing a transaction; it refuses an
   incomplete `md1` set **and an incomplete `mt1` set** (three walks); and
   `--allow-unsigned-inputs --expect transaction` does **not** falsely refuse.
   (An earlier draft certified less than the gate it pointed at — round-3 M-6.)
5. **The §6h in-memory-history question MEASURED** (I3). An earlier draft of this
   condition offered an escape hatch — measure it, **or** declare it
   unanswerable — which weakened a spec gate the spec states without one. (The
   wording is described rather than quoted: quoting a retracted phrase re-mints
   it, which is how this cycle has re-created nine of them, every one found by
   re-running a sweep and none by re-reading.) **The escape
   is exactly the failure mode `remedy.rs` exists to prevent** — `history -d`
   reports success and purges nothing, and "recorded as unanswerable" is the
   documentation-shaped version of the same thing. **The remedy work therefore carries a
   POSITIVE test: run the emitted recipe under an interactive shell and assert
   the entry is gone**, not that a command was printed.
6. **F-259 is caught by a TEST, not by construction — "by
   construction" was FALSE and a probe proved it.**

   An earlier draft of this condition claimed a type made F-259 impossible. The
   probe built those types in their **strongest** form —
   `WriteBlock::Terminal(PayloadKind)`, message derived from the carried kind —
   and then **re-wrote the bug** by changing one pattern to
   `WriteBlock::Terminal(_)`. Clean `cargo build`, clean
   `cargo clippy --all-targets`, **391/391 tests passing**, and
   `me sysw wipe --fill zeros` on a pty printed *"this payload is BEARER"* at
   exit 2 once more.

   > **A type stops a value being CONFUSED for another value. It cannot stop a
   > value being IGNORED.**

   Only the literal bool-in-the-kind-seat mistake is blocked by a type — and
   that is the only shape the old observation-types gate tested, so the gate would have
   gone green over a live recurrence.

   **What actually catches it: a pty assertion on the EMITTED WORDS**, with a
   positive control, mutation-checked. The probe verified both directions — the
   finding test FAILs on the bug and the control PASSes without it. **That
   assertion is the gate; the types are a convenience.**

   **F-260 is NOT part of this condition** — §7 reassigns it to P1 because P0
   does not touch `mt`, and requiring a test for it here would contradict that
   (round-5 I-3). The derived-message discipline is what stops it recurring
   when P1 adopts.

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
9. **F-264 fixed, because the remedy gate cannot pass otherwise.** `me`'s zsh
   recipe removes nothing when run immediately — the entry is still in memory
   and `sed -i` edits a file that does not contain it. **That gate demands the
   recipe actually work**, so P0 either fixes the recipe (`fc -W`, edit, `fc -R`)
   **The remedy must make the recipe WORK** — flush, edit, reload (`fc -W`, `sed
   -i`, `fc -R`). Merely rewording the message to say "exit the shell first"
   is honest but **cannot make the gate green**, and under "no step begins
   until the previous is green" that would stall everything after it
   (round-3). If the recipe genuinely cannot be made to work, that is a
   finding to raise, not a wording to settle for. Owning phase in `FOLLOWUPS.md` is already P0.
10. **F-265 fixed at ALL FIVE SITES, with a step that does it.** All five stay in
   `me` — `refuse_write_block` ×2, `read_records` ×2, `emit` — so this is work P0
   does **in the donor**. An earlier draft asserted both that they stay and that
   "P0 moves these functions" four lines apart, while no step edited any of them,
   so the condition could not close (round-5 I-2). Five refusals can swap exit
   **2 for 3** with all 388 tests green, proven against the unmodified binary.
   P0 moves these functions, and **a refactor over an untested distinction is
   how the distinction dies.** Every gate in §4 that asserts a refusal pins the
   **digit**.
11. An R0 round closing **0C/0I**.

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
