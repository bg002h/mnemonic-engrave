# R0 — P0 implementation plan, ROUND 1 (fold-check)

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` (444 lines)
**Worktree:** `/scratch/code/shibboleth/_work/p0r1/mnemonic-engrave` @ `09da392` (branch `review/p0-r1`)
**Date:** 2026-08-26. **Reviewer:** independent R0 agent, round 1. Scope: the two questions in the brief.
**Round 0:** 4C/5I/7M/2N, all 18 claimed folded across `b566e85`, `426809c`, `09da392`.

**Counts: 2 Critical / 4 Important / 5 Minor / 1 Nit. Verdict: NOT GREEN.**

Of the 18, **15 CLOSED, 3 PARTIAL** (I2, M3, M7). The gating weight is in Part B:
the fold's new text — the `--expect` vocabulary and the file-mapping table —
carries two Criticals and three Importants that did not exist in draft v1.

Method note: every binary invoked by absolute path with `< /dev/null`, exit code
captured into `rc=$?` before any substitution. Nothing in the already-machine-checked
list was re-derived.

---

## PART A — the 18

### Critical

| # | verdict | evidence |
| --- | --- | --- |
| **C1** | **CLOSED** | §1 carries "ONE OF THE 11 IS A SPLIT, NOT A MOVE"; §3's `fd.rs` contract states the raw `mode & 0o777`, **"No disqualifying mask anywhere in the crate — not `0o044`, not `0o077`"**, plus the char-device and fail-open `None`s as shared mechanism; §4 step 1 moves it "**intact — including the mask**" and step 2 splits it with a gate that a masked implementation cannot pass (`Some(0o620)`); §7 adds "**adoption never changes a consumer's mask**". The four sites round 0 named are all present, and the step-1/step-2 contradiction is explicitly retracted in §4. |
| **C2** | **CLOSED** | §1 now reads the 0 `std::env::args` hits as "a GAP, not an achievement"; §3 specifies both layers on raw `std::env::args()` before `Cli::parse()`, with the override's own parse named; §4 gains step 5; §6 gains condition 7. Matches §6d's two-layer text. (One §6d sub-ruling is still missing — see **N-M2**.) |
| **C3** | **CLOSED as an action** | The vocabulary is now a table in §3 and steps 5c exist, which is what round 0 asked for. **The table's contents are wrong — see N-C1 and N-I2.** |
| **C4** | **CLOSED** | Step 7's gate is now "all 388 tests pass, **with the diff to them enumerated and each edit justified by a named finding**"; §6 condition 6 states that `emit` and `write_block` change signature and that `write_block_decides_both_gates_once` **is expected to change**, and rules that condition 1's "unchanged in meaning" does not govern it. The two gates can now both hold. |

### Important

| # | verdict | evidence |
| --- | --- | --- |
| **I1** | **CLOSED** | Step 1's gate gains "**PLUS a pty assertion pinning the terminal arm (I1)**" and §4 carries the I1 paragraph naming the 12 file-redirecting tests and `script -qec`. |
| **I2** | **PARTIAL** | The ruling is stated ("the crate holds the constants and their MEANINGS … and never a mapping from binary to code") and both exclusions are named. **The constants are still not enumerated, and the fold's own file-mapping re-creates I2's concrete failure:** §3 places `refuse_write_block` in `exit.rs`, and `refuse_write_block` returns `Some(EXIT_USAGE)` where `EXIT_USAGE = 2` (`crates/me-cli/src/main.rs:296`) — the donor's. `ms` maps clap errors to **64** deliberately (`mnemonic-secret/crates/ms-cli/src/main.rs:184`, `_ => ExitCode::from(64)`, with a comment citing its own SPEC §6 carve-out). So at P2, `ms` either adopts a published `EXIT_USAGE = 2` — the change §9b rules out of scope — or ignores it, which is round 0's "decorative table" verbatim. A constant *is* a mapping; "meanings, not a table" does not separate them. **Enumeration was the part that would have exposed this**: §6f's table shows the five binaries agree on almost nothing. |
| **I3** | **CLOSED** | §6 condition 5 restores "MEASURED", describes (rather than re-quotes) the retracted escape hatch, and requires a **positive** test running the recipe under an interactive shell. §4's step-4 row was not updated to match — filed as **N-M1**, Minor, because §6 is the closure gate. |
| **I4** | **CLOSED** | §7 carries "**F-260 — REASSIGNED FROM P0 TO P1 (I4)**" with the reason and the overdue-vs-deferred rule; `FOLLOWUPS.md` updated in the same fold (machine-checked before dispatch, per the brief). |
| **I5** | **CLOSED as an action** | §3 gains "TYPES AND CONSTANTS ARE PART OF THE CLOSURE TOO (I5)", naming `WriteBlock`, `Destination`, `Class`, `Admission`, and requiring step 1 to enumerate every type and constant and confirm each is "either moved or **already public**". **That escape clause is what lets N-C2 through**: `Class` *is* already public — in `me`'s lib, which is where §3's own boundary line says it must stay. |

### Minor / Nit

| # | verdict | evidence |
| --- | --- | --- |
| **M1** | **CLOSED** | §3: "**4 `eprintln!` and ZERO bare `println!`**", with the substring-grep explanation and the ruling that `emit`'s `write_all` must stay a write. |
| **M2** | **CLOSED** | §1: "`me-cli` has both `lib.rs` and `main.rs` — as does `mnemonic-toolkit`", with the cause of the earlier error. |
| **M3** | **PARTIAL** | §3 (line 251) now says "the guard's own mask is at **line 653**, not 585; 585 carries the same `0o077` inside a *different* function". **§2.1's cross-repo note (line 102) is untouched** and still reads "…and **line 585** (the `if mode & 0o077 == 0` mask)" as one of the guard's two sites — and that note is the one instructing a future folder to "re-verify by hand", which is the failure M3 described. The plan now states both the right and the wrong citation, 149 lines apart. Incomplete propagation, not a wrong fix. |
| **M4** | **CLOSED** | §2.4 now gives "132 lines, and **only 9 of them touch IO**" with the grep terms, and retracts the 100/32 split by name. |
| **M5** | **CLOSED** | §4 gains the M5 paragraph marking steps 1 and 7 as **regression-gated** and naming steps 2, 3, 4, 5, 5b, 5c, 6 as RED-first. |
| **M6** | **CLOSED as an action** | The mapping table exists. **Its content is N-C2 and N-I1.** |
| **M7** | **PARTIAL** | The header now cites §8 and says the plan "inherits rather than supersedes" its items. §8's `me`-facing item is carried (condition 5 = §6h). §8's other P0-directed item — *"P0 must not read an absent `CLOSED` as open work: grep both"* — appears nowhere: `grep -n 'CLOSED\|DONE'` over the plan returns **0** hits outside that header line. A citation is an acknowledgement, not a reconciliation. Minor; the practical risk is near nil because both follow-ups P0 schedules were filed the same day. |
| **N1** | **CLOSED** | Step 1: "**388 RUN, 1 skipped, out of 389 `#[test]` attributes (N1); the gate is the run count, not the attribute count**". |
| **N2** | **CLOSED** | §4 names the `#[cfg(not(unix))]` stub at `crates/me-cli/src/main.rs:921` as a 12th definition moving with its twin. |

---

## PART B — defects the fold introduced

### N-C1 — CRITICAL. The `--expect` kind vocabulary drops `cosigner` and folds `mk1` into `descriptor`. §10's acceptance command cannot be run, and the funds case §6g was written for is unprotected.

**Site:** §3's `--expect` table (new in `426809c`).

**What the table says.**

| kind | admits `Class` | recognised by |
| --- | --- | --- |
| `descriptor` | `Descriptor`, `MdMk` | `md1`/`mk1` HRP |

**What the spec rules.** §6g: *"`--expect descriptor,cosigner` keyed on `Class` alone
**cannot distinguish a descriptor card from a cosigner card**"*, and the remedy is the
discriminant one level down — *"switches on the HRP character — `'d'` reassembles
through `md_codec`, `'k'` decodes through `mk_codec`"*. The plan cites that HRP
discriminant and then **unions the two HRPs into one kind**, which is the exact
collapse §6g exists to forbid. The kind `cosigner` does not appear anywhere in the
plan (`grep -in cosigner` → 0 hits).

**The concrete failure, two of them.**

1. **§10's acceptance criterion is unsatisfiable.** It runs, verbatim:
   `… | me sysw pack --expect descriptor,cosigner,transaction --out payload.bin`.
   `cosigner` is not in the vocabulary, so P4 meets either an unknown-kind refusal or
   a silently-ignored kind. §10 says *"P4 does not close until this has been RUN"*.
   This is the failure mode §6g predicted for the `transaction` row and the plan
   avoided there — reproduced one row down.
2. **The funds case is left open.** §6g's motivating scenario is *"Substitute `mk` for
   `mt` and the missing record is a cosigner card — a backup the operator believes is
   complete and that cannot restore the wallet."* Under this vocabulary,
   `--expect descriptor` is **satisfied by the `md1` records alone**: `mk encode`
   refuses, its cosigner card never reaches the stream, `pack` exits 0, and `--expect`
   — the feature built for this — reports nothing. Verified against the classifier:
   `classify_with` returns `Class::MdMk` for `md1` and `mk1` alike
   (`crates/me-cli/src/sysw/mod.rs:227`, `Ok(_) => Class::MdMk`).

**Why it is Critical rather than Important.** Step 8 publishes the vocabulary
(irreversible). Adding `cosigner` afterwards **narrows the already-shipped
`descriptor`** — a semantic break on a released crate, across six manifests. §6g
states the cost in its own words: *"Left unstated, this is first detectable at P4 —
after the vocabulary has shipped inside a released crate consumed by five CLIs."*

---

### N-C2 — CRITICAL. The file-mapping puts `me`'s record-class predicates and the class-keyed argv gate inside the crate. It contradicts the plan's own boundary line, and it does not compile.

**Site:** §3's mapping table, `records.rs` row (new in `09da392`).

> | `records.rs` | `read_records`, `split_record_stream`, `no_records_guard`, `is_secret`, `is_bearer`, `is_argv_forbidden` |

Those seven files are `mnemonic-io-lib/src/…` — the crate, not `me`'s lib half.

**(a) It cannot compile.** `is_secret`, `is_bearer` and `is_argv_forbidden` are
**inherent methods on `Class`** (`impl Class` at `crates/me-cli/src/sysw/record.rs:65`; the three at `:73`, `:89`, `:105`),
and `Class` is `me`'s. Machine-checked, not asserted — a two-crate scratch project
reproducing the shape:

```
error[E0116]: cannot define inherent `impl` for a type outside of the crate where the type is defined
 --> src/lib.rs:2:1
  | impl Class {
  = help: consider defining a trait and implementing it for the type or using a newtype wrapper
```

**(b) It contradicts §3, four paragraphs later.** §3's own boundary lines:
*"No record classes, prefixes or payload grammar — those stay with `me` per §9a."*
That restates §5a's second boundary line verbatim (*"No container vocabulary.
Record classes, prefixes and payload grammar belong to `me`; see §9a"*). The only
way to make the `records.rs` row compile is to move `Class` into the crate — which
is the thing both the plan and the spec forbid, and it would ship `me`'s container
vocabulary inside the shared crate at step 8, permanently.

**(c) It is larger than the three methods.** `read_records` is in the same row, and
its argv gate is class-keyed throughout: it calls `mnemonic_engrave::sysw::classify(r)`
(`main.rs:1978`), matches `Class::Tx` and `Class::Mt` (`main.rs:1995`, `1997`), and
reads `record::TX_PREFIX` (`main.rs:1958`). `classify` in turn reaches `bip39`,
`sysw::mt`, and `seal::record::validate_record`. So the crate's `records.rs` needs
either `me`'s classifier and its codec closure, or a seam — a caller-supplied
predicate or trait — **that the plan never names**. §6d assigns layer 2 (value-shape:
`tx:` by prefix, `mt1`/`ms1` by HRP, BIP-39 by wordlist) to P0, so this is not
deferrable to P1.

**Why the plan's own gate cannot catch it.** §3's I5 paragraph tells step 1 to confirm
each type is "either moved or **already public**". `Class` is already public — in `me`.
Step 1 moves into **`me`'s lib half**, where `Class` lives, so step 1 passes clean and
the wall arrives at step 5/5b, after four green steps, as a design question about the
published API rather than a compile error with an obvious fix.

**What it costs if resolved the cheap way.** The one edit that makes the table compile
is moving `Class` (and `classify`, and the prefixes) into `mnemonic-io-lib`. Step 7's
gate — "388 tests pass, with the diff enumerated and each edit justified by a named
finding" — **accepts that**, because "§3: the argv gate is `records.rs`'s" is a real
citation. That is C1's shape exactly: a boundary the plan states in prose and deletes
in a table.

---

### N-I1 — IMPORTANT. Three of the closure's four `eprintln!` functions are mapped into the crate, against §3's own ruling that the library must not write to stdio — and no step converts them.

**Site:** §3's mapping table vs. §3's "The stdio question, answered" (the ruling is
draft-v1 text; the mapping is new, and they were never reconciled).

The ruling: *"A library six binaries share must not write to stdio unconditionally —
it cannot be tested without capturing process stdio and a caller cannot redirect it,
which is doubly wrong in a crate whose purpose is controlling what reaches stdout.
**Functions return what should be said; the caller emits it.**"*

The mapping sends into the crate three of the four `eprintln!` sites round 0 measured:

| function | `eprintln!` at | mapped to |
| --- | --- | --- |
| `refuse_terminal_destination` | `main.rs:1035` | `channel.rs` |
| `refuse_world_readable_stdout` | `main.rs:1058` | `exit.rs` |
| `read_records` (the TTY hint) | `main.rs:2039` | `records.rs` |

`refuse_write_block` (mapped to `exit.rs`) exists only to call the first two and hand
back a code, so it is a fourth.

**The concrete failure.** Step 1's gate is "no behaviour change" and step 7's is "the
388 pass"; **both are green whichever way this goes**. So the implementer either (a)
moves them verbatim and publishes a library that writes to stderr unconditionally —
the thing the ruling forbids, in the crate whose purpose is controlling what reaches
stdio, at an irreversible step — or (b) converts them to return their text, which
changes `refuse_write_block`'s signature *and* `emit`'s body, with **no step and no
gate covering the conversion** and no statement of what the returned shape is. There
is also a third-party effect: `refuse_world_readable_stdout`'s message is remedy text
(`--out` / `umask 077` / `--allow-world-readable`), and §3 assigns remedy text to
`remedy.rs`, so the mapping splits one message across two files by accident.

---

### N-I2 — IMPORTANT. Two `--expect` kinds name `Class` variants `me` never produces. `--expect address` can never be satisfied.

**Site:** §3's `--expect` table, rows `descriptor` (`Descriptor`) and `address`.

**Measured.** `Class::Descriptor` and `Class::Address` occur **only** in a unit test
asserting they are not secret (`crates/me-cli/src/sysw/record.rs:348-349`). No
production code constructs or matches them. `classify_with`'s own doc says so:
*"**Descriptor and Address are deliberately absent**, and this is a known limitation
… classifying them needs a descriptor parser and an address decoder, neither of which
is a dependency of this crate."*

Run, absolute path, stdin at `/dev/null` (`me sysw pack --in addr.txt --no-passphrase --out /dev/null`
with one bech32 address in the file):

```
rc=4
me: record 0 (records count from 0) is not a form this container can place: not a
BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:`
record. Descriptors and addresses are not yet classifiable here — see sysw::classify
```

So the `address` row's "recognised by: **address parse**" describes a parser that does
not exist, and an address record is refused at exit 4 before `--expect` could observe
it. `--expect address` is a kind that always refuses; the `Descriptor` arm of
`descriptor` is dead. Both ship inside the published 0.1.0 vocabulary, and removing a
kind afterwards is a break.

**The rationale for the two exclusions is inverted.** §3 declares `FreeText` and
`Unknown` "**deliberately unnameable**" because "neither can be required of a stream".
That is true of `Unknown` and **false of `FreeText`**: a `text:` record classifies as
`Class::FreeText` (`sysw/mod.rs:212`) and is packed like any other. The two classes
that genuinely cannot be present are the two the table names as kinds.

---

### N-I3 — IMPORTANT. Step 5's RED gate has no site in the donor: `me` ships no secret-bearing flag for layer 1 to key on.

**Site:** §4 step 5 (new) and §6 condition 7 (new).

Step 5's gate: *"a known secret-bearing flag name is refused **before `Cli::parse()`**;
today nothing runs there, so the test fails for absence."* Condition 7 requires the
ordering "asserted **at least in the donor**".

**Measured** — every long option of `me sysw pack` / `sysw show` / `sysw wipe` / `seal` /
`bundle` whose name could carry secret material: `--seal-secret` is a **bool**;
`--passphrase-words` takes a **count**; `--passphrase-ask` is a bool; `--plaintext` is
documented "Never an ms1 or a BIP-39 mnemonic". `me`'s secret material arrives
**positionally** (`me seal <payload>`, `me sysw pack [RECORDS]…`), which is layer 2.

So layer 1's flag-name table in the donor is **empty**. The gate is unsatisfiable in
`me` and vacuous in the crate — the crate has no `Cli::parse()` for anything to run
before. The implementer writes a crate unit test against a synthetic flag table,
declares green, and P0 **publishes §6d's "primary layer" without one end-to-end
assertion in any binary**; the first real flag (`mnemonic bundle --passphrase`)
arrives at **P3**, two phases after the irreversible step. This is round 0's I1 shape —
a gate that cannot fail on the arm that matters — recreated for the layer C2 added.

There *is* a donor-testable half and the plan already owns it: §5a rules that `me`
ships the override as an ordinary clap flag (`crates/me-cli/src/main.rs:252`,
`#[arg(long)] allow_argv_secret`) and that "P0 owns it". An assertion on the
override's pre-parser move is writable today; the flag-name half is not.

---

### N-M1 — MINOR. §6 condition 5 prescribes a step-4 test that §4's step-4 row does not carry.

Condition 5: *"**Step 4 therefore carries a POSITIVE test: run the emitted recipe under
an interactive shell and assert the entry is gone**, not that a command was printed."*
§4's step 4 row still reads only *"zsh remedy does **not** contain `history -d`; fish
remedy does **not** contain the secret"* — both negative-content, neither able to reach
the §6h question. An implementer walking §4 does not measure it; only a reader of §6
does. One row, not a design change.

### N-M2 — MINOR. §6d's ruling on what happens to admitted material is not carried, and §3 calls the current exit code "the behaviour to preserve".

§6d rules: *"admitted material is passed to the tool through the same internal path as
`--in` content, and **never re-presented to clap as a positional**, because a later,
unrelated clap error would echo it."* The plan carries the pre-parser ordering and the
override's own parse, but not this. It shapes the crate's published entry point (the
guard must hand the admitted material back for routing, not let the caller re-feed
clap), so P1 discovers it after publish.

Adjacent, same paragraph: §3 verifies `me sysw pack --nosuchflag <ms1…>` exits 2 with
the secret absent — **reproduced here, rc=2, 0 occurrences of the string in stderr, so
the claim is true** — and calls it "the behaviour to preserve". After a pre-parser
guard that invocation must exit **3** (`EXIT_REFUSED`), because the guard now runs
first. Nothing says the code changes, and "preserve" plus step 1's "no behaviour
change" reads as forbidding it.

### N-M3 — MINOR. The `passphrase` row's discriminant is the argv guard's, not the stream's.

§3's table gives `passphrase` → "recognised by: **flag-declared only**". `--expect`
tests what is present in the **record stream**, where a passphrase is a `pass:` record
recognised by its prefix (`sysw/mod.rs:203`). "Flag-declared" is layer 1's
discriminant from the paragraph above, carried into the wrong table.

### N-N1 — NIT. §8's title is misquoted in the header.

The plan quotes §8 as *"What is NOT verified, and must be before the plan **is
trusted**"*. The spec's heading is *"What is NOT verified, and must be before the plan
**closes**"* (`SPEC_constellation_cli_uniformity.md:1483`). A reader grepping the
quoted string finds nothing.

---

## MACHINE-CHECKED THIS ROUND — command, result

| # | claim | how | result |
| --- | --- | --- | --- |
| 1 | an inherent `impl` of a foreign type is a compile error | two-crate scratch project, `cargo build` | **E0116**, verbatim above |
| 2 | `Class::Descriptor` / `Class::Address` are never produced | `grep -rn 'Class::Descriptor\|Class::Address' crates/` | 2 hits, both in one unit test (`record.rs:348-349`) |
| 3 | an address record is refused, not classified | `me sysw pack --in addr.txt --no-passphrase --out /dev/null` | **rc=4**, "Descriptors and addresses are not yet classifiable here" |
| 4 | `md1` and `mk1` share one `Class` | read `classify_with` | `Ok(_) => Class::MdMk` (`sysw/mod.rs:227`) |
| 5 | `me` ships no secret-bearing flag | `me <verb> --help` over 5 verbs | `--seal-secret` bool only; secrets are positional |
| 6 | §3's no-leak claim | `me sysw pack --nosuchflag <ms1…>` | **rc=2**, flag named, `grep -c` of the ms1 in stderr = **0** — claim TRUE |
| 7 | `emit` / `write_private` are not crate dependencies | `grep -n` call sites | `emit` called at 1358/1360/1386 (all `run_sysw`); `write_private` at 424/496/620 and inside `emit` — none from the other 9 |
| 8 | `EXIT_USAGE` is the donor's 2; `ms` uses 64 deliberately | `grep -n 'const EXIT_'` / `ms-cli/src/main.rs` | `me` `EXIT_USAGE = 2` (`main.rs:296`); `ms` `_ => ExitCode::from(64)` (`:184`) |
| 9 | `cosigner` appears in the plan | `grep -in cosigner` | **0 hits** |
| 10 | §8 reconciled beyond the header | `grep -n 'CLOSED\|DONE'` | 0 hits outside the header citation |
| 11 | the 585 mis-citation | `grep -n '585\|653'` | line 102 (wrong, uncorrected) and line 251 (right) |

**Not re-derived** (machine-checked before dispatch, per the brief): `plan-cite-check.sh`
8/8, `plan-table-check.sh` 41 rows, `fold-propagation-check.sh` exit 0, the 0-`println!`
count, the toolkit's lib+main, the 653 mask line, `read_records` 132/9, the exit-code
sweeps, and F-260's reassignment in `FOLLOWUPS.md`.

---

## NOT FILED — checked, and the fold is right

Recorded so round 2 does not re-derive them.

- **`emit` / `write_private` left in `me` is compile-consistent.** Nothing among the
  other nine calls either (claim 7). §1's "each must move or the extraction does not
  compile" is about the five callees, and the direction holds: `emit` calls *into* the
  closure, never the reverse. The residual is arithmetic — §1 totals the crate at
  11/431 while §3 gives it 9 of them (347 lines) — and is not worth a finding on its
  own.
- **`exit.rs`'s ruling does not forbid §6f's `mk` 2 → 1.** That change is **P3**'s
  (§7's phase table names it explicitly: *"and `mk`'s invalid-artifact 2 → 1, which
  §6f calls the only code this cycle changes and which no phase owned (I-4)"*), so no
  P0 artifact needs a per-binary table. §9b's 2-vs-64 split is named absent correctly.
  The exit.rs problem is `refuse_write_block`'s constant (I2 PARTIAL), not the ruling.
- **Condition 6 does not contradict step 7 or step 1.** The signature change lands in
  step 7, whose gate explicitly admits a justified test diff; step 1's "no behaviour
  change" is scoped to the move. C4's fold holds.
- **`transaction = Mt ∪ Tx` is right**, and stated the way §6g orders.
- **`Class` has exactly ten variants**, spelled as §3 lists them (`record.rs:44-62`).
- **§1's "six of the nine"** understates its own table (7 rows in the `main.rs` column,
  3 in the lib column) — **pre-existing in draft v1** and examined by round 0 without
  fault, so it is out of scope and not counted. Noted only because the same
  off-by-one is what makes §3's "THE 11 MAPPED" header describe a 14-name table.

---

## VERDICT

**2 Critical / 4 Important / 5 Minor / 1 Nit — NOT GREEN.**

The fold closed 15 of 18 findings cleanly and the four Criticals of round 0 are
genuinely answered. The gate stays shut on the fold's own new text.

**The single most important finding is N-C2.** §3 states the spec's boundary line —
record classes stay with `me` — and then, four paragraphs later, its mapping table
puts `Class`'s three predicates and the class-keyed argv gate inside the crate. It
cannot compile (E0116, reproduced), the plan's own step-1 check ("moved or already
public") passes it because `Class` *is* already public in `me`, and the cheapest edit
that makes it build is to move `me`'s container vocabulary into the shared crate — at
step 8, irreversibly, with step 7's gate written to accept it under a citation. It is
C1's shape one file over: a boundary asserted in prose and deleted in a table.
