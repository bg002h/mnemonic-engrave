# SPEC: constellation exit codes

**Status:** revision 6, 2026-11-26. Implemented in all eight binaries and both
GUIs on the `fx/*` working branches, which are reviewed but not merged or
pushed (§10). The §2 decisions were built as recommended and still await
Brian's ruling. So do the coordinator rulings of §2a, which the implementations
follow, and the open questions of §2b, which nobody has decided.

**Revision 6 (2026-11-26).** This revision makes the spec say what the
implementations do. It records the coordinator rulings taken during the
implementation (§2a), lists the questions still open (§2b), folds the 27 spec
defects that the final review listed (§10), records the interim state of the
shared constants (decision 4, §9), replaces the GUI's version floors with
detection by the help legend (§7j, §9), and adds Appendix A (settings and
inputs, by tool). Revisions 1 to 5 were the review rounds below.

**Spec reviews** (Fable agents). §10 says how each finding was folded. The
implementation reviews are in §10 too.

| round | result | report |
| --- | --- | --- |
| 0 | NOT GREEN: 2 Critical, 16 Important, 14 Minor, 4 Nit | `design/agent-reports/exit-codes-2026-11-26/review-r0.md` |
| 1 | NOT GREEN: 0 Critical, 5 Important, 12 Minor, 3 Nit | `review-r1.md` in the same directory |
| 2 | NOT GREEN: 0 Critical, 3 Important, 8 Minor, 3 Nit | `review-r2.md` in the same directory |
| 3 | NOT GREEN: 0 Critical, 2 Important, 5 Minor, 3 Nit | `review-r3.md` in the same directory |
| 4 | NOT GREEN: 0 Critical, 1 Important, 9 Minor, 5 Nit | `review-r4.md` in the same directory |

**Scope.** Every binary in the constellation except seedhammer and
rust-miniscript: `md`, `mk`, `ms`, `mt`, `me`, `mnemonic`, `mr` and
`mr-sitting`. Two GUIs are covered as consumers: `mnemonic-gui`, and `mr-gui`
with refugium-view in refugium-wallet. The helper processes `me-preview`,
`gui_render` and `gen_ap2_grind` are out of scope. They are internal, and only
their parent's code is specified.

**Brian's direction (2026-11-26):** *"Do we have a spec for exit codes across
the constellation? Maybe we need one"*, and *"We should make the exit codes
summable like 1, 2, 4, 8 so any combination into one sum is unique."*

**Evidence.** The six survey reports, the brief they shared, and the round-0
review are in `design/agent-reports/exit-codes-2026-11-26/`. They measured
every binary at the heads of the 2026-11-25 fix PRs. A "today" cell is a
measured run unless its report marks it as source only. Through §1, §2, §5,
§7 and §8, "today" means before the implementation, as the surveys measured
it. Revision 6 keeps those words so that each before-and-after stays
readable; what the binaries do now is the "new" side.


## 1. Do we have one?

Only partly. Three earlier documents each cover a slice:

- **The uniformity spec, §6f** (`SPEC_constellation_cli_uniformity.md`). A
  table of four outcomes for six binaries: usage error, invalid artifact,
  repair applied, repair impossible.
- **D26**, quoted in §6f. It fixes what the repair codes *mean*.
- **The network rule** (mnemonic-toolkit `SPEC_network_rule.md`). It
  reserves 8 for a network mismatch in every tool.

Each tool's own spec or manual covers some of its codes. What is missing is one
table across tools, and the surveys show the cost of that:

| outcome | what the binaries returned before the implementation |
| --- | --- |
| usage error | 2 (`md`, `mt`, `me`, `mr`, `mr-sitting`) or 64 (`mk`, `ms`, `mnemonic`). Inside `mt` and `mnemonic`, the flag clashes the program checks itself get 1 or 2 instead |
| "the answer is no" | 1 (`md verify`, `mt`, `mr`, `mnemonic verify-message`) or 4 (`mk`, `ms`, the rest of `mnemonic`). `md descriptor --verify-against` gives 5 |
| invalid artifact | 1, 2, 3 or 4, depending on the tool and sometimes on the input's shape |
| empty input | 1, 2 or 64 |
| refusals | 1, 2, 3 or 64 |
| I/O failure | 1, 2 or 64 |

Two codes are badly overloaded:

- **5** means "repair applied" in four tools. It also means "NOT
  spend-equal" in `md`. In `mnemonic`, on a terminal only, it means
  "aborted, here is a repair report".
- **4** means "verify mismatch", "unverified repair" or "partial result" in
  `mk`, `ms` and `mnemonic`, and "invalid artifact" in `me`.

The toolkit manual has exit tables for `mnemonic`, `mk`, `mr` and
`mr-sitting`. The READMEs of `mt` and `me` list some codes, and there is no
cross-tool table.

## 2. Decisions for Brian

Each decision has a recommendation, and the rest of the spec is written
assuming it. The first four shape the scheme. The other seven each change an
existing behaviour that some script or document relies on.

1. **Seven codes fit, and network mismatch shares bit 8.**
   - An exit status is one byte. Shells use 126 and 127 themselves, and 128+N
     for a process killed by signal N, so every sum must stay at 125 or below.
   - Six combinable bits (1 to 32, sums up to 63) leave 64 free for a usage
     code that never combines. A seventh combinable bit would make sums reach
     127 and leave no free usage code.
   - The network rule (Brian, 2026-11-23) gave network mismatch the code 8.
     *Recommended:* bit 8 becomes **REFUSED**, the class of "the tool chose not
     to proceed", with network mismatch as one of its members. A network
     mismatch still sets bit 8. Its machine-readable stderr line (the network
     rule's N10) is what tells it apart from the other refusals.
   - A HARD mismatch (a source that contradicts itself) also sets 8, by that
     ruling, even though no flag clears it.
   - *The alternative* gives network its own bit and folds another class away.
   - §6 lists what this changes in the network rule.

2. **Usage is 64, it never combines, and it means the command line alone.**
   - Under this scheme 2 is a bit (INVALID), so clap's 2 cannot stay. 64 is
     the traditional `EX_USAGE` (BSD `sysexits.h`), and `mk`, `ms` and
     `mnemonic` already use it. `md`, `mt`, `me`, `mr` and `mr-sitting` change.
   - 64 covers what can be judged from the command line without reading any
     input, file or environment variable: the parser's errors (an unknown flag
     or verb, a missing argument, a conflict, a bad setting value), and the
     tool's own checks on its *settings*. A malformed *input* value is 2 even
     when the parser rejects it (below).
   - **Setting versus input on argv.** A setting tells the tool how to run:
     iterations, counts, share K and N, output form, ranges. It is judged from
     the command line, so a bad one is 64. An input is the data the verb works
     on, even when it arrives as an argument: a card string, a key, a
     fingerprint, template text, a wallet shape's parameters
     (`md compose --wrapper`, `--path`), the hot family's values. Its validity
     is about the wallet, so a bad one is 2.
   - So `me seal --iterations 5`, the toolkit's `seed-xor split --shares 1`,
     and `ms` with a bad K or N give 64. A malformed `md --key`,
     `md --fingerprint`, `md compose --path 5of3` or `mr --hot-child Z:24`
     gives 2.
   - **Who checks the value does not change its class.** A malformed input
     value is 2 even when clap's value parser is what rejects it. Examples:
     `mr verify --hot-parent-fp zz`, a toolkit scheme name such as
     `--template bogus`, and `--expect-fingerprint zz`. The implementation
     marks those arguments as input and maps clap's invalid-value errors on
     them to 2, so a fingerprint typo is 2 in every tool. E45's closed-list
     entry for a malformed `--hot-parent-fp` moves to 2 with them (§6).
   - This reverses the CLI review's recommendation of 2, which predates the bit
     scheme. It also narrows `mr`'s E45 closed list: its entries that read the
     card (a card that is another format's string, `member` with no keys for a
     seat the card does not key) become input errors, 2.
   - Brian's 2026-10-06 "Keep 1" ruling stays in substance: a malformed
     hot-family value is still not a usage error. It is an input refusal, now
     2.

3. **Empty input is INVALID (2), not a usage error.** In a pipeline, an empty
   stdin nearly always means the step before it failed, and 2 points at the
   input. A required argument that is *missing* stays 64. This reverses CLI-14's
   "empty input is a usage error".

4. **One definition of the numbers.**
   - mnemonic-io-lib publishes no exit integer on purpose (`src/exit.rs`): "a
     constant is a mapping", and the binaries agreed on nothing. This spec is
     that mapping, so the objection goes away.
   - *Recommended:* mnemonic-io-lib gains the constants and a small bit-set
     type, and each binary still does its own classification.
   - Six binaries already depend on the library, and `mt` takes it from
     crates.io (0.1.3). So this means a publish, then pin and `vendor/` moves
     in the five crates.io consumers; `me` uses a path dependency and moves
     with the source (§9).
   - mr-codec, which has a no-CLI-deps rule, keeps a copy with its own test
     against the §3 numbers.
   - **Interim state (revision 6).** The publish has not happened.
     mnemonic-io-lib 0.1.4 is written but unpublished. It holds the
     constants, `Bits`, `HELP_LEGEND`, and, since the final review,
     `exit::add` (adds a bit to a code, and leaves a code above 63 unchanged)
     and `Bits::from_code`. `me` takes it by path. Every other binary carries
     a local copy pinned to §3 by its own test, and the copies differ in their
     names: module `exit` or `exit_code`, `LEGEND` or `HELP_LEGEND`, `add` or
     `with_io_failure`, and 130 as `STOPPED`, `OPERATOR_STOP` or absent.
     Replacing the copies with the published crate is owed (§9 step 1).

5. **Every verb reports a correction it used (R6).**
   - Today these correct a damaged string, print a note on stderr, and exit 0:
     `mt decode`/`verify`/`inspect`, `mk decode`/`verify`/`inspect`/`address`/`derive`,
     and `mnemonic inspect --mk1`, `convert --from mk1=…` and `word-card
     --from …` (which print no note at all). Under R6 they
     exit with 16 set. The output is the same; a script now learns that the
     plate is damaged.
   - The cost: `SPEC_mt_v0_1.md:3384-3385` (mnemonic-engrave's normative copy)
     defines 0 as "every check passed". Its one-liner `mt decode … > tx.hex &&
     …` (`:810-827`) and any `set -e` script would stop on a corrected plate.
     A script that wants the old behaviour tests `[ $(( rc & ~16 )) -eq 0 ]`.
   - *Recommended: yes.* For a backup tool, silent damage is the worst thing to
     hide. *The alternatives* are 16 from `repair` only, or a `--strict` flag
     that opts in.

6. **`repair` checks that each corrected string decodes.**
   - mr's E45 rule says `repair`'s code is the checksum's alone. So `mr repair
     mr1qqpqqqqqqqqqqqqq5saqe93dj7z0t` exits 5 ("corrected") and the result is
     no card. The same no-card string with a clean checksum exits 0.
     `mk repair` exits 0 on strings that `mk decode` rejects.
   - *Recommended:* every `repair` decodes each corrected or clean string as a
     string or chunk of its format, and adds 2 for any that fails. It still
     prints the corrected strings with its warning, because a person
     repairing by hand needs to see them. A corrected string that fails its
     check would give 16 + 2 = 18, and with no oracle, as for a single string,
     16 + 4 + 2 = 22. Under the shapes below (ruling 3), `repair` prints a
     string that is no card only where it has no oracle for it, so it reports
     22. A set that has its oracle and fails is 2 with nothing printed, so
     `repair` does not report 18.
   - The check is per string. A set that is merely *incomplete* is not a
     failure (decision 7), and a *complete* set that fails reassembly is 2,
     with nothing printed.

   - This changes E45 and the hazard line in mnemonic-refugium's CLAUDE.md
     ("its exit code stays the checksum's").
   - **The shapes (ruling 3, §2a).** These are the same in `md`, `mk`, `ms`,
     `mr` and the toolkit's `repair`. `mk` (a malformed set read as
     incomplete) and `mr` (a complete set that fails, printed) were being
     folded onto them when this revision was written (§10).

     | input | status | printed |
     | --- | --- | --- |
     | an incomplete set or a lone chunk: fewer chunks than the total, distinct in-range indices, one total | 16 if a chunk was corrected, 0 if clean (decision 7) | yes, chunk by chunk |
     | a malformed set: a repeated index, more chunks than the total, an index out of range, or chunks that disagree on the total | 2 | nothing |
     | a complete set that fails reassembly or decode | 2 | nothing |
     | a chunked set of one | as a complete set | as a complete set |
     | a string that is not chunked, corrected, that decodes | 20 | yes |
     | a string that is not chunked, corrected, that is no card | 22 | yes, and never written to `--out` |
     | a string that is not chunked, clean, that is no card | 2 | nothing |
     | a string whose chunk header is malformed | clean: 2; corrected: 22 | clean: nothing; corrected: yes |
     | an `ms` K-of-N share | clean: 0; corrected: 20 | yes, and written to `--out` |

     A share is a valid string of its format, not a card that failed. `md`
     has one exception: a corrected single string at a wire version this
     build does not support is printed at 22 whatever its chunk flag, so a
     chunked set of one at a newer version is 22, not 2. That errs toward
     warning.
   - A string that is no card is shown, so that a person repairing by hand can
     see it, but it is never filed for an engraver: `--out` is not written.

7. **A correction in an incomplete chunk set is 16, not 20.**
   - D26 puts "verifiable by reassembly later" under `REPAIR_APPLIED`. Today
     the tools split three ways:
     - `mk` follows D26 and gives 5, though it prints "correction
       UNVERIFIED … can alias to a different card" beside that 5.
     - The toolkit demotes the case to VERIFY-ME: `mnemonic repair --mk1`
       gives 4 where `mk repair` gives 5 on the same strings. Its
       `repair --md1` refuses an incomplete set, as `md` does (2).
     - `md repair` refuses an incomplete set or a lone chunk outright, clean
       or damaged: 2, with no output.
     - `mr repair` accepts a lone chunk (5 if corrected, 0 if clean) but
       refuses a set of two or more chunks that is incomplete: 2, with no
       output ("the chunk set lacks chunk 2 of 4").
   - *Recommended: 16 everywhere*, which is D26 as written. The rest of the
     set will verify it when it is reassembled.
     - The toolkit changes.
     - `md` and `mr` change more. They correct and print each chunk of an
       incomplete set (16), and a clean one becomes 0 (§7a, §7g). That is what
       makes a partly damaged set repairable before every plate is found.
     - A lone chunk of a multi-chunk set counts as an incomplete set (16). Only
       a string that is not chunked at all has no oracle (20).
     - `mk`'s message changes to say the correction is verifiable when the
       rest of the set is reassembled (R5: the message agrees with the
       code).
   - *The alternative* is 20 everywhere, which is safer and reverses D26's
     wording.
   - What counts as incomplete, and what is malformed instead, is in decision
     6's table. Chunks that carry different set ids are an open question
     (§2b).

8. **`inspect` reports a string that would not decode (R5).** Today `ms
   inspect` prints "FAIL: would NOT decode" and exits 0, as its locked table
   says. *Recommended:* it exits 2. `md inspect` and `mk inspect` already
   exit non-zero on such a string, printing nothing on stdout; R5 does not
   require any tool to start printing a report it does not print today.

9. **An operator who stops at a prompt gets 130.** Today a `mr-sitting`
   session stops with 1 when the operator answers "stop" or input ends at a
   prompt, which would read as "a check said no". *Recommended:* a deliberate
   stop exits 130, the same as Ctrl-C, since both mean "the operator stopped
   it".
   - This covers "stop" typed at any prompt, including the mainnet
     confirmation.
   - End of input at a prompt is empty input, 2 (decision 3).
   - GUIs read 130 as "stopped" (R12).
   - *The alternative* is 2, which reads as "bad input" and is less true.

10. **`mr` renumbers in the same cutover.**
    - `mr` is about two thirds of the test churn (1,700 assertions on 1 and 230
      on 2), with about 900 TRACE lines, under a plan frozen at M15.
    - *Recommended: yes, as its own PR train*, with the change entering through
      the spec's build-impact notes and `owed-to-build` FOLLOWUPS, as the
      process gates require. A constellation where `mr` alone keeps the old
      numbers is the mixed state this spec exists to end, and refugium-wallet
      reads `mr`'s numbers directly.
    - *The alternative* is to defer `mr` and `mr-sitting` to their next minor
      release, with the GUIs decoding by version.

11. **A mismatch that an override can accept is NO (1), and the override
    clears it.** `mnemonic restore --expect-fingerprint X` exits 4 today with
    no output, and 0 with output under `--allow-mismatch`. The strict BSMS
    mismatch is the same kind of thing. A mismatch is a comparison that came
    out no, so *recommended:* 1, with the override clearing it (R4).
    *The alternative* treats every overridable refusal as 8, which puts a
    wallet that does not match the user's expectation in the same class as a
    secret typed on argv.

### 2a. Rulings taken during the implementation

The coordinator ruled on the questions the implementers raised. Each ruling
was taken as a recommendation, with the same status as the decisions above:
the implementations follow it, and it is pending Brian.

1. **Named helpers (R9).** A helper is named when the operator gave its path
   by flag or environment variable, even if the path equals the default. A
   named helper that is missing, cannot be started, fails, or gives a reply
   the tool cannot use sets 32. An unnamed default that is absent or failing
   is a warning and sets nothing. A helper's own answer (a null, a not-found)
   is not a failure. Naming a missing path is not an offline switch. An
   explicit `mt --offline` is open (§2b).
2. **16 survives a later failure (R1).** Once a verb has used a corrected
   value, bit 16 stays set when the run then fails (17, 18, 24 and so on).
   The exception is `repair` when it prints nothing.
3. **The repair shapes (decisions 6 and 7).** Uniform across `md`, `mk`,
   `ms`, `mr` and the toolkit, as decision 6's table gives them.
4. **Self-disagreeing input, comparisons and conflicts (tests 1, 4 and 6).**
   An input that disagrees with itself, or two parts of one input that
   disagree, is 2. Bit 1 is a comparison between the input and a separate
   reference the operator gave. A flag conflict judged without reading values
   is 64. A missing required input channel is 64, and a named channel that
   yields nothing is 2.
5. **Passphrases and passwords (§3).** A wrong passphrase or password proven
   by an authentication tag is 1. No tag, a missing passphrase, or a
   malformed container is 2. A failure while sealing with a key the tool
   derived is 101.
6. **The lock and the mainnet sentence (§7h).** A held lock is 32. A refused
   mainnet sentence is 130 (decision 9).
7. **A choice the tool declines (test 5).** When the tool declines to choose
   and a flag would choose, as `me` with `--as` omitted, the run is 8.
8. **GUI scheme detection (R11, R12, §7j, §9).** Detection by the help legend
   replaces version floors. 16 alone is shown amber, not green; the colour is
   open for Brian (§2b).
9. **Per-verb help (R11).** Each verb's `--help` ends with one line that
   states the bits it can set.
10. **JSON (R14).** The `kind` strings differ across tools and are renamed
    only when mnemonic-io-lib is published. The additive repair fields are
    recorded.
11. **Decision 4's interim state** is recorded (decision 4, §9).

### 2b. Open questions for Brian

Nobody has decided these. The implementations do what each item says today.

1. **An explicit offline switch for `mt`.** Under R9 naming a missing path is
   not an offline switch, so an operator with `bitcoin-cli` on `PATH` cannot
   keep `mt` from asking that node (which may be remote) about the
   transaction's txids, short of editing `PATH`. The question is whether to
   add `mt --offline` (or `--bitcoin-cli none`). Today there is none.
2. **A damaged duplicate plate that `mt` discards.** When a set holds a clean
   and a damaged copy of one chunk, `mt` keeps the clean copy and exits 0,
   while it prints "re-cut it" for the damaged one. Either a discarded copy
   does not set 16, as today, or 16 is set whenever the tool tells the
   operator to re-engrave.
3. **Groups with different set ids that cover every index.** Two `mk1`
   chunks with different `chunk_set_id`s that together cover every index
   exit 0 (16 if corrected) in `mk` and in the toolkit: each id forms its own
   incomplete group, which decision 7 says is no failure. `md` refuses chunks
   of two different cards (2). The choices are to say each set id is its own
   group and the tool names the split, or to treat such groups as a
   suspected miscorrection (2).
4. **The GUI colour of 16 alone.** Recommended amber, which the mnemonic-gui fold adopts:
   a lone corrected chunk is 16 and is verifiable only when the
   set is reassembled, and the GUI cannot tell that from a verified
   correction. The alternative is green with text that says "verified, or
   verifiable when the whole set is read".
5. **A toolkit path mismatch against a fingerprint mismatch.** In
   `mnemonic bundle`, an inline descriptor path that disagrees with the
   slot's path is 1, while the slot's fingerprint that disagrees with its
   annotation is 2. Ruling 4 decides it once it is settled whether the slot
   is a separate reference the operator gave or a second part of one input.

## 3. The scheme

| code | name | set when | what the operator does |
| --- | --- | --- | --- |
| 0 | OK | everything requested was done and every check passed | nothing |
| 1 | NO | the input is well-formed, and a comparison against something else came out no: a reference, a phrase, a template, the chain, an expectation. For example a verify mismatch, NOT spend-equal, not a member, `--expect` not met, an input the node proves spent | read the verdict |
| 2 | INVALID | an input is wrong for this verb whatever the flags: bad checksum, uncorrectable, wrong HRP or kind, a newer format, empty input, a malformed value the tool reads, an input that breaks the verb's correctness rules | fix or replace the input |
| 4 | UNVERIFIED | the tool checked part of the result and could not confirm it: a correction with no oracle, a partial decode, a signature accepted leniently | verify before trusting the result |
| 8 | REFUSED | the tool declined under a safety or policy rule that a flag, network, channel or different destination would waive, or declined to make a choice that a flag would make, or a network mismatch (decision 1). Examples: secret on argv, an unsafe output destination, an output that exists and would be overwritten, a limit a flag can raise | change how it is run, or pass the override knowingly |
| 16 | REPAIRED | the tool applied a checksum correction to an input and used the corrected value | the plate is damaged; re-engrave it |
| 32 | IO | something the tool needed from the system failed or is missing: an input file, an output that cannot be written, stdout, a helper program the user asked for, a device, a lock, a RAM folder, an environment variable, a feature this build lacks | fix the environment |
| 64 | USAGE | never combines: the command line alone is wrong (decision 2). Nothing ran | fix the command line |
| 101 | INTERNAL | never combines: a bug. This is Rust's panic code, and also the code for any failure no class covers (R13) | report it |
| 128+N | | the shell reports a process killed by signal N this way; 130 also means the operator stopped it (decision 9) | |

Codes 1 to 63 are sums of the six bits, and each sum is unique. 64 and 101 are
single codes. Any other value from 65 to 125 is a bug.

**Two words used above:**

- A **channel** is how the same verb reads or writes: argv, stdin, a file, a
  terminal. Using another verb (for example `sysw pack` in place of `--hex`)
  is not a different channel.
- **Not asking for a check is not a failure.** `restore` with no reference, or
  `mt` with no node, exits 0 and says what it did not check. Bit 4 is for a
  check that ran and could not confirm.

**The tests that pick a class,** asked in this order:

0. Did the operator stop it, at a prompt or with Ctrl-C? Then **130**
   (decision 9).
1. Is the command line wrong, judged without reading any input, file or
   environment: an unknown flag or verb, a missing argument, a conflict, or a
   bad *setting* (decision 2)? Then **64**. A malformed input value is not
   this test; it is 2 (test 6). A conflict here is one between flags, judged
   without reading their values. A missing required input channel (no input
   named where the verb has no default) is 64; a named channel that yields
   nothing is empty input, 2 (decision 3, ruling 4). Where a verb reads stdin
   when no input is named, as `ms` does, stdin is the named channel, so an
   empty stdin is 2. The exception is the argv
   guard of uniformity spec §6d, which runs first: a secret on argv is 8
   (R2).
2. Did something needed from the system fail or go missing? Then **32**.
3. Is it a network mismatch? Then **8** (decision 1).
4. Is the input well-formed, but does it fail to match something it should
   match: a reference the user gave, a phrase, a template, a policy-id stub, a
   plan it should belong to, the chain's state, or an `--expect` claim? Then
   **1**, even when an override exists (decision 11). The comparison is
   between the input and a separate reference the operator gave, or the
   world (ruling 4). A limit, cap, deadline
   or delta that the tool enforces is not a match; those are test 5.
5. Did the tool decline under one of its safety or policy rules, which a flag,
   a network, a channel or a different destination would waive? Examples:
   secret exposure, an unsafe or existing destination, the same file in and
   out, a guardrail, a limit. The same holds when the tool declines to make a
   choice that a flag would make, such as `me` with `--as` omitted on an
   input whose packing it will not guess (ruling 7). Then **8**.
6. Is the input itself wrong for this verb, or insufficient for it (for
   example `mr member` with no keys for a seat the card does not key)? Then
   **2**. An input that disagrees with itself, or two parts of one input
   that disagree (an origin fingerprint against its xpub, a `--key` bracket
   against the template's path), is this test, not test 4 (ruling 4).
7. Anything else is a bug: **101** (R13).

A run that ends on its first failure takes that failure's class. When order
can change the sum (a network refusal found before or after a correction),
tests and vectors assert the bits that must be set, not the exact sum.

**Applying the tests to cases that rounds 0 and 1 found ambiguous:**

| case | class | why |
| --- | --- | --- |
| an `ms1` to `mt`, or to `me --hex` | 2 | neither verb has a flag that takes one |
| an xprv given as an xpub | 2 | |
| `convert` from xpub to xprv | 2 | |
| `export-wallet` given a secret slot | 2 | it is watch-only by definition |
| `mr`'s published-vector root | 2 | the remedy is new draws |
| an unsigned transaction to `me sysw pack` | 8 | `--allow-unsigned-inputs` |
| an unsigned transaction to `mt` | 2 | |
| an `ms` digest-shaped phrase | 8 | `--phrase-looks-like-digest-ok` exists |
| `compare-cost` over the `--max-conditions` cap | 8 | |
| `compare-cost` with an unsupported wrapper | 2 | |
| `mr-sitting practice --dir` whose files exist | 8 | an output that would be overwritten |
| a directory that cannot be written | 32 | |
| `mr-sitting` with no RAM folder | 32 | |
| `md compile` in a build without the feature | 32 | |
| `me seal --iterations 5` | 64 | |
| `ms` reading two inputs from stdin | 64 | |
| `@env:NAME` with NAME unset | 32 | |
| `mt encode` on an input the node proves spent | 1 | `mt verify` never asks a node |
| `mt verify --bitcoin-cli …` | 64 | a node flag on a verb that never asks a node |
| `me` with `--as` omitted where the tool will not choose a packing | 8 | a flag would choose (ruling 7) |
| `me sysw hash` with no records | 64 | a missing required input, judged from argv |
| `mnemonic restore --from xpub=…` | 2 | not a seed source: the wrong kind of input for the verb |
| an origin fingerprint that disagrees with its xpub | 2 | one input disagrees with itself (ruling 4) |
| a wrong passphrase proven by an authentication tag | 1 | ruling 5 |
| `mr-sitting` with its lock held | 32 | ruling 6 |
| `mr-sitting`, the operator refuses the mainnet sentence | 130 | a stop at a prompt (decision 9, ruling 6) |
| `mr --time 1`, `--height abc`, `--digest zz`, `--set-id zz` | 2 | input values (decision 2) |
| `derive-child --application rsa` | 2 | a valid name the tool will not do; no flag waives it |
| a BIP-388 distinctness violation in `verify` | 2 | |
| `mr-sitting root --network main` where input ends first | 2 | the first failure wins |

**Passphrases and passwords** (ruling 5). An authenticated cipher cannot
tell a wrong passphrase from altered ciphertext. So a decrypt that fails its
authentication tag is 1, the passphrase compared with the container, and the
message says that the passphrase is wrong or the container was altered. A
decrypt with no tag to prove it, a missing passphrase, or a malformed
container is 2. A failure while sealing with a key the tool derived itself
cannot be caused by the input, so it is 101. `me` (`seal`, `sysw`) and the
toolkit (BIP-38, Electrum, BSMS) implement this.

`me`'s refusal table (`Row` in `me-cli/src/descriptor/refusal.rs`) and the 66

`Refusal::new` sites in `mt-cli/src` (outside tests) each get a class column,
decided by these tests.

**Reading the code in a script:**

```sh
md repair "$card"; rc=$?
if   [ "$rc" -eq 0 ];  then echo "clean"
elif [ "$rc" -ge 64 ]; then echo "usage, internal error, operator stop or signal: $rc"
else
  [ $((rc & 1))  -ne 0 ] && echo "a check said no"
  [ $((rc & 2))  -ne 0 ] && echo "an input is invalid"
  [ $((rc & 4))  -ne 0 ] && echo "verify before trusting"
  [ $((rc & 8))  -ne 0 ] && echo "refused"
  [ $((rc & 16)) -ne 0 ] && echo "a correction was applied"
  [ $((rc & 32)) -ne 0 ] && echo "file, device or helper failure"
fi
```

A GUI applies the same 1..63 guard. On Windows an exit status is 32 bits, and a
crash shows up as an NTSTATUS value. So anything outside 0..125, other than
130 (stopped by the operator or Ctrl-C), means "ended abnormally".

**D26 under this scheme.** D26 is a rule about meaning, not numbers
(uniformity spec §6f), so its meanings carry over and only the numbers move:

| D26 meaning | today | new |
| --- | --- | --- |
| clean, nothing to correct | 0 | 0 |
| `REPAIR_APPLIED`: verified now, or verifiable by reassembly later (decision 7) | 5 | **16** |
| `VERIFY-ME`: a correction with no self-oracle | 4 | **20** (16 + 4) |
| atomic fail, uncorrectable | 2 | **2** |

**What 20 does and does not say.** Round 0's C1 is right that 20 is also the
sum of a verified correction and an unrelated unverified result. One example is
`mnemonic verify-bundle` reporting a partial result (4) on a bundle in which
another card's correction was verified (16).

The scheme accepts this. Bit 4 is defined by what the operator must do,
"verify before trusting", and that action is the same either way. So the sum
can over-warn but never under-warns. A tool that needs to say *which* part is
unverified says so in its output, as `mnemonic`'s UNVERIFIED banner already
does.

## 4. Rules

- **R1. Sum what happened.** The exit status is the sum of the bits for what
  the reported outcome contains. A verb that fails atomically reports only its
  failure bits. `md repair` with one uncorrectable string emits nothing and
  exits 2, not 18. A verb that prints its result despite a failure (`repair`
  under decision 6, `inspect` under R5) sums every bit it reports. A verb may
  stop at the first failure (fail fast) and report only what it examined.
  Tests assert masks where order can change the sum (§3).
  - **Bit 16 survives a later failure** (ruling 2). Once a verb has used a
    corrected value, bit 16 stays set in whatever status the run ends with,
    including an atomic failure or a refusal: a verify mismatch on a
    corrected card is 17, a content-id failure after a correction 18, a
    network refusal 24, a stdout failure after a corrected repair 48. The
    plate is damaged whatever
    happened next, and the bit only adds a warning. The one exception is
    `repair` when it prints nothing (an uncorrectable string, a malformed
    set, a complete set that fails reassembly): it exits 2 alone, as in the
    `md repair` example above.
- **R2. The usage check comes first, except the argv guard.** A usage error
  exits 64 before any input is read or any output is written. One exception
  is the pre-parser argv guard of uniformity spec §6d: it is normative and runs
  first. So a secret on argv plus a bad flag gives 8 in the tools that have
  that guard. `mr` checks argv after parsing, so there a bad flag gives 64
  first; adopting the pre-parser guard in `mr` is uniformity-spec work, not
  this spec's. A limit that depends on
  the input, such as `md descriptor --chain 7` on a card whose limit is 2, is
  not usage. It is 2, and it must be checked before the verdict is printed.
  Before the implementation `md` printed the verdict first and then exited 1.
  Putting usage first has a cost: advice that needs the input cannot come
  before a usage error. `me --in wallet.txt` with no output mode is 64
  before `me` can say "this is a descriptor; use `sysw pack`", so `me`
  prints a static hint instead.
- **R3. Classify with the ordered tests of §3.** Uniformity spec §6i (posture
  versus correctness) is a taxonomy. It explains most REFUSED rows, but it is
  not the test.
- **R4. An override clears what it overrides.** An explicit override flag,
  such as `--allow-*`, `--accept-*`, `--experimental` or
  `--allow-unsigned-inputs`, clears the bit its refusal or check would have
  set, and the run is judged as if that check had passed. So `restore
  --allow-mismatch` keeps exiting 0, as its help says today. It still prints
  the mismatch. The rule follows the flag's function, not its prefix.
- **R5. A report verb reports in its code (decision 8).**
  - `inspect` is a report verb. When the artifact it inspects would not
    decode, or a share would not combine, it prints its whole report and
    sets 2. Before the implementation `ms inspect` printed "FAIL: would NOT
    decode" and exited 0.
  - A message must agree with its code. `ms verify` printed "OK: valid
    future format" and exited 3; under this spec it exits 2, so the message
    stops saying OK. A verb whose only bit is 16 may print its success line
    (`mt verify` and `mk verify` print `OK` and exit 16), and must name the
    correction beside it.
  - A field that only describes the world is not a failure and leaves the code
    alone. `mt inspect`'s STATUS DEAD (the transaction is spent on chain) is
    such a field.
  - A classify verb such as `gui-schema --classify-descriptor` exits 0
    whatever class it prints.
- **R6. Every correction sets REPAIRED (decision 5).** This applies in any
  verb that used a corrected value.
- **R7. The code must not depend on whether stdout is a terminal.** Today,
  on a terminal, `mnemonic`'s auto-repair replaces the verb's output with a
  repair report and exits 5; in a pipe it exits 1. Under this rule the verb has
  failed on an invalid input either way, so it exits 2 in both. The repair
  suggestion is advice, and the terminal may still decide whether it is
  printed. Bits 16 and 4 are set only by a verb that *used* a corrected value.
  No new auto-repair behaviour is introduced.
- **R8. Output failures add IO.** A stdout write error, a closed stdout or a
  full disk adds 32 to the other bits; it does not replace them. EPIPE (the
  reader went away) keeps the verb's own code, as before. Before the
  implementation every binary exited 0 with stdout closed (`md decode <card>
  >&-`, `mt decode >&-`, `me --hex >&-`, all measured), losing the output.
  - A closed stdout is detected on Linux, Android, macOS and the BSDs. On
    Windows it is not, and a run with stdout closed still exits 0 with the
    output lost.
  - A closed stdin reads as empty input, 2 (decision 3).
- **R9. An unnamed helper is optional; a named one is not** (ruling 1).
  - A helper is *named* when the operator gave its path by flag or
    environment variable, even if the path equals the default (`mt
    --bitcoin-cli bitcoin-cli`, `ME_PREVIEW_BIN`). A helper the verb needs
    (`mr-sitting`'s picotool) counts as named.
  - A named helper that is missing, cannot be started, exits with a failure,
    or gives a reply the tool cannot use sets 32. The tool says which helper
    failed and how.
  - An unnamed optional helper that is absent or failing is a warning and
    sets nothing: `mt` with no `bitcoin-cli` on `PATH`, or one found there
    that fails; `me` with no preview sidecar.
  - A reply that is the helper's own answer, such as a null or a not-found,
    is not a failure.
  - Naming a path that does not exist is therefore not an offline switch.
    Whether `mt` gains an explicit one is open (§2b).
  - A helper failure sets 32 even when the input caused it, such as a
    sidecar that cannot render the plate the input describes. The message
    says what the helper reported.
- **R10. Signals keep the shell meaning.** A process killed by signal N is
  reported by the shell as 128+N. A process that catches a signal to clean
  up, as the `mr-sitting` session's wipe hook does, re-raises the signal when it
  is done instead of exiting 1. A deliberate stop at a prompt exits 130
  (decision 9). If the wipe itself fails during a stop or a signal, the
  process exits 32 instead, after printing its wipe-failure message, because
  the operator must now act on the device.
- **R11. Help states the table.** Every binary's root `--help` ends with the
  same legend, and the toolkit manual gains one appendix holding §3. Each
  verb's `--help` ends with one line that states the bits that verb can set
  (ruling 9). `mt`, `me`, `mk`, `ms` and `md repair` have it. `md`'s other
  verbs, `mnemonic`, `mr` and `mr-sitting` still owe it. The legend:

  > Exit status: 0 = done, every check passed. 64 = bad command line, nothing
  > ran. 101 = internal error. 130 = stopped by the operator. Other codes from
  > 128 up = killed by a signal. Otherwise the
  > sum (1 to 63) of: 1 a check said no; 2 an input is invalid; 4 verify the
  > result before trusting it; 8 refused (network mismatch, secret on the
  > command line, unsafe output); 16 a correction was applied; 32 a file,
  > device or helper failed.

  The legend is the last text of the root `--help` (and of `-h` and `help`
  where they exist), compared with whitespace normalised, so a binary may
  wrap it. Each binary has a test for this. A consumer detects the scheme
  this way (ruling 8): an exact legend at the end means the bit codes; no
  paragraph beginning `Exit status:` means the old codes; anything else (a
  legend that differs by a word, or one that is not last) means the status
  cannot be decoded, and the consumer shows the number undecoded.
- **R12. GUIs decode the bits.**
  - A child killed by a signal has no exit code on Unix; a GUI shows
    "killed by signal N" from the platform's status.
  - A GUI decodes 1..63 bit by bit and names each bit it finds. It never keys
    on a whole number other than 0, 64, 101 and 130 ("stopped"). It treats
    anything else outside 0..125 as "ended abnormally" (§3).
  - A GUI decides whether output exists by looking at the output, never from
    the code.
  - A GUI shows 16 alone in amber, not green (§2a ruling 8; the colour is
    open, §2b). The code cannot tell a verified correction from one that is
    verifiable only when the rest of a set is read.
  - The GUIs' own processes exit 0, 64, 32 (cannot start: missing libraries,
    no display, a child that fails to start) or 101. `mr-gui`'s catch-all 1
    goes, and mnemonic-gui keeps its deliberate 130 when a clean shutdown
    after a signal times out.
- **R13. No failure goes unclassified.** Today an unclassified `Err` returned
  from `main`, a bare `ExitCode::FAILURE`, or an internal serialise error
  exits 1. That would read as "a check said no". Each such path is either
  classified or exits 101. The blanket mappings are where this hides:
  - `mr`'s `From<mr_codec::Error> for Failure` sends every library error to 1
    (`app/common.rs:42-57`).
  - `mt` maps any `Err` to 1 (`main.rs:374-376`).
  - The write filters turn 0 into 1 (`mt` `out.rs:91-99`).
  The conformance test for each binary is therefore a table test: each error
  variant, and each refusal row, maps to its §3 class. A grep for
  `ExitCode::FAILURE` alone would pass all three blanket mappings.
- **R14. Machine-readable output carries the same number.** The JSON
  envelopes of `mk`, `ms` and `mnemonic` that hold an `exit_code` field put the
  process's exit status in it, and their schemas say it is a bit sum.
  - **The `kind` strings are owed** (ruling 10). The tools named one class
    differently: an IO failure is `IoError` in `mk`, `Io` in `ms` and
    `IoFailure` in the toolkit, and an invalid input is `InvalidInput` in
    `mk` and `BadInput` in `ms`. `Refused` and `Internal` agree. Nothing is
    renamed now. One name per class is settled when mnemonic-io-lib is
    published, and the GUI keys on the class names from then on.
  - **Repair's additive fields.** A repair envelope may add `not_a_card` (at
    22, saying why the corrected string is no card), `incomplete: true` (an
    incomplete set or lone chunk, decision 7), `share` (an `ms` share), and
    `mk verify` adds `corrected`. Each is additive; an envelope without them
    keeps its old meaning. The three tools' repair shapes therefore no longer
    byte-match one another.


## 5. Today to new, by class

| class | before | new |

| --- | --- | --- |
| success | 0 | 0 |
| answer is no | `md` 1 (`--verify-against` 5); `mk` 4; `ms` 4; `mt` 1; `mnemonic` 4 (`verify-message` 1, strict BSMS mismatch 2); `mr`, `mr-sitting` 1; `me --expect` not met 4 | 1 |
| invalid artifact or value | 1 in `md` (`repair` 2), `mk`, `mt` and `mr`; `ms` 1 or 2 (reserved tag 3); `mnemonic` 1, 2 or 3; `me` 4; `mr-sitting` plan files 2 | 2 |
| usage | `md` 2; `mk` 64; `ms` 64; `mt` 2 (its own flag checks 1, non-UTF-8 argv 64); `me` 2 (non-UTF-8 argv panics: 101); `mnemonic` 64 (its own checks 1 or 2); `mr`, `mr-sitting` 2 | 64 |
| empty input | `md` 2; `mk` 64; `ms` 1 (`repair` 2); `mt` 1; `me` 2 (`sysw show` 4); `mnemonic` 1 (`build-descriptor` 2); `mr` 1 or 2 | 2 |
| refusal that an override, network or channel clears | `md` 1 or 2; `mk` 64; `ms` 1 or 64; `mt` 1; `me` 3 (destination refusals 2); `mnemonic` 2 (some 1, `compare-cost` 3); `mr` 1 or 2; `mr-sitting` 1 | 8 |
| repair applied, verified or verifiable | 5 (`md`, `mk`, `mr`, `mnemonic`; the toolkit gives 4 for an incomplete set); 0 with a stderr note in other verbs | 16 |
| repair applied, no oracle | `ms` 4, `mnemonic` 4 (`md` and `mr` say 5) | 20 |
| uncorrectable | 2 (`mt` 1) | 2 |
| I/O, device, named helper | `md` 1 or 2; `mk` 1 (input file 64); `ms` 1; `mt` 1 (named `--bitcoin-cli` failing: 0); `me` 1 or 2 (sidecar 2 or 4); `mnemonic` 1 (two flags 2); `mr` 1; `mr-sitting` 1 or 2 | 32 |
| unverified result | `md` partial decode 4; `mnemonic` `verify-bundle` partial 4, lenient BSMS 4 | 4 (`restore` with no reference stays 0, §3) |
| unclassified failure, panic | 1 or 101 | 101 |
| operator stop | `mr-sitting` session 1 | 130 |
| signal | 128+N; `mr-sitting` session 1 | 128+N |

## 6. Rulings this changes

This spec changes the following earlier rulings. Each one is listed so that
none changes silently.

- **D26.** The meanings stay and the numbers move (§3). `ms`'s "reasoned and
  load-bearing" 4 becomes 20. The toolkit's "4 = project-wide VERIFY-ME"
  becomes "bit 4 = verify before trusting".
- **Network rule** (mnemonic-toolkit `SPEC_network_rule.md`, Brian
  2026-11-23):
  - Exact-8 checks become bit checks. "Exits 8" becomes "sets bit 8"
    everywhere it appears: the test `nr0_exit_code_8_is_network_mismatch`, the
    vectors, and `mk`'s JSON envelope. Under R1 the status can be 24 or more.
  - The GUI rows at `:400-401` ("CLI exit 8 → red banner / Override") key on
    the N10 machine line, not on the number. Otherwise they would fire on every
    argv refusal. N10 prints only under `MNEMONIC_NETWORK_JSON=1`, so the GUIs
    set it when they run a CLI, and N10's trigger "a CLI that exits 8" becomes
    "a CLI that sets bit 8". A script that needs to tell a network refusal
    from other refusals sets the same variable.
  - N9 ("8 is reserved for `ms` and `mt` and unreachable") no longer holds.
    Both can set 8 for other refusals.
  - `me`'s `PromotionTestnetKey` "stays 3" becomes whatever the §3 tests give.
  - "Nothing was written" is not promised for every refusal: `me` refuses a
    destination after a successful conversion (mt-me report, C).
  - A HARD mismatch sets 8 though no flag clears it (decision 1). The legend
    does not promise an override.
  - The gate `scripts/spec-network-rule-gate.sh` check 5 fails any repo that
    uses 8 for anything else. It changes to check the N10 line instead.
  - Order: the network rule's units are not built yet, so they should be built
    straight onto bit 8 (§9).
  - **Not done yet (revision 6).** None of the network-rule changes above is
    made: `SPEC_network_rule.md` still says 8 is reserved and unreachable
    (`:143`), still says "a CLI that exits 8" (`:146`), and still keys the GUI
    rows on exit 8 (`:400-401`), and gate check 5 is unchanged. They are owed
    with the network rule's unbuilt units.
  - Among those units: `me`'s `MixedNetwork` (a file whose keys are of two
    networks) sets 8, but no binary prints the N10 line yet, so a script
    cannot tell it from an argv refusal. `md` and `mk` set 8 for a network
    mismatch and print no N10 line either.
- **Uniformity spec §6f.**
  - Its table is replaced.
  - Its "`mk` 2 → 1" was done, and `mk` now moves to 2 with everything else.
  - Its ruling that "`mnemonic`'s 1 or 2 by input shape stands" is replaced:
    an invalid input is 2 whatever its shape.
  - The collisions it recorded (`md encode` with no template at 2;
    `ms repair` 4 against `me sysw pack` 4) are resolved.
  - Its line 35 says `mt` refuses a world-readable stdout at 2; it measures 1.
- **Uniformity spec §6i.** Still a taxonomy. R3 uses §3's tests instead.
- **mnemonic-io-lib `exit.rs`.** "No integers" is reversed by decision 4.
- **mr spec E45 and D9.**
  - Usage moves from 2 to 64.
  - Refusals split into 2 and 8 by the §3 tests (E45's "1 on any other
    refusal").
  - "Repair's code is the checksum's either way" changes by decision 6.
  - The closed list narrows to checks on the command line alone (decision 2).
    Its entries that read the card become 2, and so does its malformed
    `--hot-parent-fp` entry (an input value; decision 2). The 2026-10-06 "Keep 1" ruling
    keeps its meaning: "not usage".
  - mnemonic-refugium's CLAUDE.md hazard line ("its exit code stays the
    checksum's") changes with decision 6.
- **`ms`'s locked exit table** (ms-cli and its spec; decisions 8 and 3), **`me` ruling T6** on
  its code vocabulary, **`md`'s reservation of 5** for repair, **the toolkit's
  argv-refusal test** (`2`), and **refugium-wallet spec §4.1.1** (0/1/2 from
  `Outcome::code()`) are all replaced by §3.
- **mt spec.** The normative copy is in mnemonic-engrave:
  `SPEC_mt_v0_1.md:3384-3385` and `:810-827`. It changes by decision 5, and
  the mnemonic-transaction copy re-syncs. Done: engrave `27234b9`,
  `9659335` and `946147a` (`mt verify --bitcoin-cli` is usage), with mt's
  re-sync in `ffa1f45` (the last line still to follow).
- **The toolkit's incomplete-set demote** (4 today) changes by decision 7.
- **CLI review CLI-14.** Replaced by this spec. Decision 2 reverses its usage
  recommendation, and decision 3 its empty-input rule.

## 7. Per-repo changes

Pointers are from the surveys. Each repo's tests, help text, manual chapter,
GUI mirror and printed text move in the same PR, per its CLAUDE.md lockstep
rules. "Pins" are the surveys' counts of test assertions on an exact non-zero
code. Assertions on 0 are mostly unaffected, except where decision 5 turns a
silent correction into 16.

### 7a. `md` (descriptor-mnemonic): about 110 pins (about 63 on 1, 20 on 2, 10 on 4 or 5)

- **Mapping.** `crates/md-cli/src/main.rs:930-961` maps clap errors to 64.
  `CliError::BadArg` is split: checks on the command line alone go to 64, and
  checks that read input go to 2 (decision 2). "No subcommand" and non-UTF-8 argv go to 64.
- **No.** `verify` MISMATCH stays 1. `descriptor --verify-against` NOT
  spend-equal moves from 5 to 1, which also fixes the GUI badge (§7j).
- **Invalid.** Decode failures in every verb move from 1 to 2. Empty input
  stays 2, and an empty-string argument moves from 1 to 2.
- **Correctness refusals move from 1 to 2:** BIP-388 key reuse, an
  unsupported shape, and an xprv where an xpub is expected.
- **Network.** A wrong-network key moves from 1 to 8, per the network rule.
- **Override refusals move to 8:** "needs `--experimental`" and `--md-only`
  (1 before), and `--out` naming the input (2 before). Bad `compose` values
  move from 1 to 2.
- **`--allow-invalid-miniscript`** is not in the `md` this was built against.
  It exists only on md PR 76 (open and unmerged, branch
  `claude/project-thread-utm1aa`, commit `fcefa83`), which refuses an
  invalid-miniscript card at 1 unless the flag is given. Revisions 1 to 5
  listed it here by mistake. When that PR lands, its refusal is 8 (an
  override clears it, test 5), and toolkit manual `42-md.md:184`, which says
  "exit 1", moves with it.
- **Repair.** An incomplete set or a lone chunk, refused today (2, no
  output), is corrected chunk by chunk and printed: 16 if a correction was
  applied, 0 if clean (decision 7). A chunked set that reassembles moves from
  5 to 16. A
  single-string correction moves from 5 to 20, which closes FOLLOWUP
  `md-cli-non-chunked-single-string-repair-demote` by the same rule.
  Uncorrectable stays 2, and the result is decode-checked (decision 6). A
  corrected single string at a wire version this build does not support is
  printed at 22 whatever its chunk flag (decision 6's exception).
- **Unverified.** Partial decode stays 4 on `decode` and `inspect`.
- **IO moves to 32:** `--in` missing (2), `--from-mk1-file` missing (1),
  `encode --out` unwritable (1), `gen-man`/`vectors` output dir (2), stdout
  errors (added, not replacing), and closed stdout (0).
- **R2.** `--chain` out of the card's range is 2, checked before the verdict.
- **Build feature.** "Feature not compiled in" moves from 2 to 32: this build
  lacks something the command needs (§3).
- **Other consumers.** `mnemonic-engrave/demo/sh2/rehearse.sh:60` expects
  `md repair` to give 5.

### 7b. `mk` (mnemonic-key): 64×48, 1×17, 4×12, 5×9 pins; about 30 rc rows and four gate scripts

- **Mapping.** `crates/mk-cli/src/error.rs:135`. Usage stays 64.
- **No.** Verify mismatch moves from 4 to 1.
- **Invalid moves to 2:** invalid artifacts (1 today), an unsupported version
  (3), a multisig card given to `address` (64), empty input (64), and an xprv
  given as `--xpub` (64).
- **Refusals move from 64 to 8:** an output that is an input, and the
  `address` network refusal.
- **Repair.** Moves from 5 to 16, including an incomplete set (decision 7).
  Uncorrectable stays 2. `repair` decode-checks each string (decision 6),
  which ends today's 0 on strings `mk decode` rejects.
- **Corrections in other verbs.** `decode`, `verify`, `inspect`, `address` and
  `derive` move from 0 to 16 (decision 5).
- **IO moves to 32:** a missing input file (64 today), an unwritable `--out`
  (1), and closed stdout (0).
- **JSON.** The JSON `exit_code` field follows (R14).

### 7c. `ms` (mnemonic-secret): 1×68, 64×40, 2×20, 4×11 pins

- **Mapping.** `crates/ms-cli/src/error.rs:60`. Usage stays 64.
- **No.** `verify --phrase` mismatch moves from 4 to 1.
- **Invalid moves to 2:** checksum or length (1), and a reserved tag or
  `verify`'s "valid future format" (3). A wrong HRP, tag, prefix or share
  stays 2.
- **Inspect.** "would NOT decode" moves from 0 to 2 (decision 8).
- **Repair.** Applied moves from 4 to 20, and uncorrectable stays 2. `ms`'s 4
  then stops meaning two things.
- **Empty input** moves from 1 to 2.
- **Refusals move to 8:** a secret on argv (1, `--allow-argv-secret`),
  `--out` refusals (64), and a digest-shaped phrase (1,
  `--phrase-looks-like-digest-ok`).
- **Two readers on stdin** moves from 1 to 64.
- **IO moves from 1 to 32:** a missing input file, an unwritable `--out`, no
  OS randomness, and closed stdout (0 today).
- **JSON.** The JSON `exit_code` field follows (R14).

### 7d. `mt` (mnemonic-transaction): about 36 non-zero pins (about 130 counting pins on 0)

- **Mapping.** `crates/mt-cli/src/main.rs:297-378` returns SUCCESS or FAILURE
  only, and gets the full mapping. Each of the 66 `Refusal::new` sites in
  `mt-cli/src` (outside tests) gets a class.
- **Usage moves to 64:** clap errors (2) and the flag-combination rows of
  §10.10 (1). Of that section's 19 rows, the two pasted-txid rows read the
  input, so they go to 2, and the four non-UTF-8 rows are already 64.
- **No.** Verify mismatch stays 1, and so does a node's verdict, including an
  input the node proves spent.
- **Invalid moves from 1 to 2:** an invalid set, garbage, an ms1, the ten §6i
  correctness refusals, and empty input.
- **Refusals move from 1 to 8:** argv material, a world-readable stdout, and
  the same file given as input and output.
- **IO moves from 1 to 32:** read and write failures. A named `--bitcoin-cli`
  that is missing, cannot start, fails, or gives a reply `mt` cannot use
  moves from 0 to 32 (R9), including a path that does not exist: naming one
  is not an offline switch. An unnamed `bitcoin-cli`, absent from `PATH` or
  found there and failing, stays 0 with its warning. Core's own answers
  (exit 5 from `getrawtransaction`, 89 from an old Core's `getindexinfo`)
  are answers, not failures. Whether to add `mt --offline` is open (§2b).
- **`mt verify --bitcoin-cli`** is usage, 64: `verify` never asks a node
  (`SPEC_mt_v0_1.md` (b)).
- **Corrections.** In `decode`, `verify` and `inspect`, these move from 0 to 16
  (decision 5). For `mt`, 16 means the reassembled transaction re-derives its
  id. Witness bytes are not covered by the id; a miscorrection there yields a
  transaction that cannot be broadcast, not a different one. So `mt` reports
  16, never 20. A transliterated HRP character is a reading, not a
  correction, and does not set 16. A damaged duplicate that `mt` discards for
  a clean copy is 0 today (open, §2b). The normative spec's
  one-liner (`SPEC_mt_v0_1.md:810-827` in mnemonic-engrave) and its exit
  status line (`:3384-3385`) change with this.

### 7e. `me` (mnemonic-engrave): about 80 non-zero pins; six gate scripts branch on the codes

- **Constants.** The private constants at `crates/me-cli/src/main.rs:495-500`
  give way to the shared ones (decision 4).
  - `EXIT_USAGE` moves from 2 to 64.
  - `EXIT_INVALID` moves from 4 to 2.
  - `EXIT_REFUSED` (3) splits into 2 and 8 by the §3 tests, row by row of the
    `Row` table.
- **Usage.** Non-UTF-8 argv, which panics with 101 today before the argv guard
  runs, moves to 64.
- **Invalid.** Empty input stays 2. `sysw show` on an empty file moves from 4
  to 2.
- **Refusals move from 2 to 8:** destination refusals (world-readable,
  terminal stdout, `--manifest` naming the plate file).
- **Expect.** `sysw pack --expect` not met moves from 4 to 1.
- **IO moves to 32:** read failures (2), write failures (1), and a sidecar
  that was found but fails (2 or 4).
- **Named sidecar.** `ME_PREVIEW_BIN` naming a file that does not exist moves
  from 2 to 32: the user named the helper (R9).
- **No sidecar.** An absent sidecar (none named) stays 0 with a warning, even with
  `bundle --preview DIR`. The user asked for a preview, not for the sidecar
  (R9). If Brian prefers, `--preview` can count as asking for the sidecar,
  which would make this 32.
- **Refusal rows.** `--as` omitted, where `me` will not choose a packing, is
  8: a flag would choose (ruling 7). `MixedNetwork` is 8 (decision 1). The
  representability rows (`PromotionTestnetKey`, `Md1FixedIndex`,
  `Md1NoWildcard`, `MultiUnderDescriptor` and similar) are 2.
- **Passphrases.** An authentication failure on open is 1, a missing
  passphrase or malformed container 2, and a sealing-side failure 101
  (ruling 5). An out-of-range iteration count is 64 as a flag and 2 when read
  from a container's header.
- **Sidecar.** A render failure in a sidecar that was found is 32 even when
  the input caused it (R9).
- **Scripts.** Six gate scripts branch on the codes, and each has a `--cli`
  mode that runs the binary: `fix-me-admission-canonical-gate.sh`,
  `fix-me-checklist-partial-keys-gate.sh`, `fix-me-cli-arg-contract-gate.sh`
  (it needs the seedhammer submodule and Go), `fix-me-descriptor-input-strict-gate.sh`,
  `fix-me-inspection-fail-open-gate.sh` and `fix-me-output-integrity-gate.sh`.
  All six pass in `--cli` mode at `3183c14`. They move in the same PR, and so
  does `policy-differential.py`. Revisions 1 to 5 said three.

### 7f. `mnemonic` (mnemonic-toolkit): about 470 pins in 221 files

- **Mapping.**
  - `ToolkitError::exit_code()`, `src/error.rs:641-702`.
  - The verb-level `Ok(n)` returns.
  - `out_sink.rs:89`.
  - The clap override at `cli.rs:173`, which already gives 64.
- **Usage moves to 64:** runtime flag-combination checks (2), and `BadInput`
  checks on settings (1), such as `seed-xor split --shares 1`.
- **Input values move to 2:** `BadInput` checks on input values, and a
  malformed scheme name (`--template bogus`, 64 from clap today). So does a
  *malformed* `--from` or `--slot` under `--allow-argv-secret` (64 from clap
  today) (decision 2). Without that flag the argv guard runs first and gives
  8 (R2), as it should: `--from phrsae="<words>"` is a real secret on argv.
- **Corrections.** `convert --from mk1=…` and `word-card --from …` move from 0
  to 16 when they used a corrected card (decision 5). The implementer audits
  every decode call site for R6, since R13's table test does not see an `Ok`
  path.
- **No moves to 1:** `verify-bundle` mismatch, `restore` expectations,
  `xpub-search`, and an `import-wallet` seed mismatch (all 4), and the strict
  BSMS signature or MAC mismatch (2). `verify-message` stays 1.
- **Unverified (4).** `verify-bundle` "partial" and lenient BSMS stay 4.
  `restore` with no reference stays 0 (§3: not asking for a check is not a
  failure).
- **Invalid moves to 2:** bad cards (1, 2 or 3), parse errors, a newer wire
  version (3), an undecodable card inside `verify-bundle` (4), empty input
  (1), the one-way `convert` refusals, the electrum-taproot, bip388-NUMS and
  wif-at-mint refusals, and an xprv inside an imported blob.
- **Restore.** `restore --from xpub=…` is 2: an xpub is not a seed source,
  the wrong kind of input for the verb, as `convert` from xpub to xprv.
- **Passwords.** A wrong BIP-38 passphrase or Electrum password proven by its
  authentication tag is 1, as the BSMS MAC; a decrypt with no tag is 2
  (ruling 5).
- **Refusals move to 8:** a secret on argv (2), network mismatch (per the
  network rule), and the `compare-cost` cap (3). `export-wallet` given a secret
  slot moves to 2: export is watch-only by definition, so no flag clears it.
  `compare-cost` with an unsupported wrapper is also 2.
- **Repair.** Verified moves from 5 to 16, and unverified or ambiguous from 4
  to 20. An incomplete set moves from 4 to 16 (decision 7).
- **Auto-repair.** In `inspect`, `convert` and `verify-bundle`, an invalid
  input is 2 on a terminal and in a pipe alike (R7). `inspect --mk1`'s silent
  correction moves from 0 to 16 (decision 5).
- **IO moves to 32:** today's 1, and the 2 behind `--descriptor-file` and
  `--spec`.
- **Other consumers.**
  - JSON `exit_code` fields (R14).
  - `prove-envelope-consistency.sh` (expects a refusal to be 1),
    `verify-examples.sh` and `root-shaped-test.sh`.
  - The toolkit manual and the GUI manual (§7i).

### 7g. `mr` (mnemonic-refugium): about two thirds of the whole renumber

- **Size.** The survey counts about 1,700 assertions on 1 and about 230 on 2
  across mr-cli's 147 test files, plus 46 corpus vector files that carry
  `"exit"`. The TRACE (about 900 lines), the spec text, the printed advice
  ("must exit 0 / exits 1"), the usage-quiet trailer keyed on code 2, and
  `45-mr.md:53-60` all move under the repo's quote and printed-command gates.
  So do the gate scripts `spec-gate-sitting-kit-day.sh` and
  `create-key-kind-fix-gate.sh`, and `golden/diff-wrapper.py`.
- **Constants.**
  - `crates/mr-codec/src/app/exit_codes.rs`.
  - `Failure::fail` and `Failure::usage` in `app/common.rs`.
  - The duplicate `mr_codec::repair::EXIT_*`.
  - `mr_codec::api::Outcome::code()`, which refugium-wallet reads (§7k).
- **Usage moves from 2 to 64** for the parser's errors on settings (input
  values are 2, decision 2), and for those of
  E45's closed-list checks that need only the command line (decision 2).
  The list's entries that read a card become 2.
- **Refusals ("1 on any other refusal") split by §3:**
  - to 2: a card that does not decode, a slot the card lacks, a malformed
    hot-family value ("Keep 1"), another format's string, and the
    published-vector root (the remedy is new draws, not a flag);
  - to 1 (test 4): the `--accept-*` refusals that are mismatches, such as a
    plate whose policy-id stub is not this card's (`--accept-stub-mismatch`);
  - to 8: a secret on argv (2 today), the hot-cold rules and the guardrails on
    deltas, lockups and deadlines that `--accept-*` flags clear, and network
    refusals.
  - Each `--accept-*` flag is placed by test 4 or 5 in the implementing PR,
    and refugium-wallet follows (§7k).
- **No.** Verify mismatch and `child --check` with no match stay 1.
- **IO moves from 1 to 32:** `--keys-file`, `--heir-file`, `vectors --out`,
  and stdout.
- **Repair.**
  - A string that is not chunked moves from 5 to 20.
  - A lone chunk of a multi-chunk set moves from 5 to 16.
  - An incomplete set of two or more chunks, refused today (2, no output), is
    corrected chunk by chunk and printed: 16, or 0 if clean (decision 7).
  - A set that reassembles is 16.
  - A corrected string that is not chunked and is no card is printed at 22,
    a clean one is 2 with nothing printed, and a complete set that fails
    reassembly or decode is 2 with nothing printed (decision 6's table).
    Uncorrectable stays 2.
  - E45's repair paragraph changes with this (§6).

### 7h. `mr-sitting` (mnemonic-refugium): about 270 code assertions (the survey's basis, including 0)

- **Usage** moves from 2 to 64.
- **Plan files.** A malformed plan, hand-back or kit file stays 2. A
  well-formed hand-back that belongs to another plan is a mismatch, so 1
  (test 4).
- **IO moves from 2 to 32:** an unreadable file, a held lock (ruling 6), no
  RAM folder, and picotool unusable. A failed picotool `load` or erase moves
  from 1 to 32.
- **No.** Verify mismatch stays 1.
- **Operator stops** move from 1 to 130 (decision 9).
- **Mainnet.** An operator who refuses the mainnet confirmation sentence
  stops the run, so it is 130 (decision 9, ruling 6), not 8.
- **Refusals move from 1 to 8:** a brought seed.

- **Existing output.** `practice --dir` whose files exist moves from 2 to 8.
- **End of input at a prompt** moves from 1 to 2.
- **Signals.** A session signal moves from 1 to 128+N, after the wipe (R10).
- **Parity.** The CI rule that `mr` and `mr-sitting` agree on `root`/`child`
  stays.

### 7i. Toolkit manual and GUI manual

- **Appendix.** One appendix holds §3, and every chapter's exit section
  points at it.
- **Manual claims that are wrong today,** and could be fixed now:

| where | the doc says | measured |
| --- | --- | --- |
| `42-md.md:802-807` | `gen-man` I/O errors exit 1 | 2 |
| `md` README and manual | `repair` I/O errors exit 1 | 2 for input |
| `44-mk-cli.md:622` | unwritable `mk encode --out` exits 64 | 1 |
| `41-mnemonic.md:393`, GUI manual `42-bundle.md:274`, `bundle --help` | out-of-range `--import-json-index` exits 2 | 1 |
| `39-cross-format-conversion.md:284` | multi-entry envelope with no index exits 2 | 1 |
| GUI manual `70-mk/*` (72, 74, 75, 77, 78) | invalid mk1 exits 2 | 1 |
| GUI manual `32-run-and-output.md:63-69` | `mk inspect` with no arguments exits 2 | 64 |
| GUI manual `4f-ms-shares-split.md:94,100` | bad K or N exits 64 | 1 |

### 7j. mnemonic-gui

- **The badge is wrong today.** `src/app_window.rs:1316-1356` shows any tab's
  4 as amber "VERIFY-ME" and 5 as green "Repair Applied", so `md descriptor
  --verify-against` NOT spend-equal (5) gets a green repair badge. This is
  from reading the source; it was not run. Under R12 the badge names each bit
  set.
- **Tests and scripts.** Tests and measurement scripts that treat `0 | 4` as
  "has output" check the output instead (R12).
- **Text that moves.** The tooltip at `src/schema/ms.rs:649` ("exit 64").
  About seven comments, one of which (`schema/mnemonic.rs:2358`) is already
  wrong. `tests/tutorial/manifest.rs:104`, and the recorded measurement JSON.
- **Cutover (§9).** Every old code is a valid new sum with a different
  meaning: old 5 reads as NO + UNVERIFIED, old 3 as NO + INVALID, and old 4
  from `mk verify` as UNVERIFIED. So the GUI must know which scheme each
  binary speaks before it reads a status. Revision 5 asked for a version
  floor on every run. The implementation detects the scheme instead, from the
  binary's root `--help` (R11, ruling 8):
  - the exact legend as the last text, whitespace normalised: the bit codes;
  - no paragraph beginning `Exit status:`: the old codes (Legacy);
  - anything else, or a `--help` that hangs or fails: Undecoded, and the
    status is shown as a bare number.
  The probe must run the same file the run spawns (on Windows the
  application's directory is searched before `PATH`). Version floors are gone,
  except G1's floor for secret runs. A legend that drifts by one word in one
  copy is therefore read as Undecoded, never as Legacy, whose 5 is green.
- **16 alone** is amber, not green (R12; open, §2b).

### 7k. refugium-wallet

- **Panel.** `refugium-view/src/mr_panel.rs:240-246` maps `Outcome::code()`
  as 0 success, 1 refused, else "usage, a bug". It moves to decoding bits.
- **Create.** Create's path call requires 1 (`create/mod.rs:4031`,
  `create/sitting.rs:1181,1321`); it becomes the class §7g gives that
  refusal. There are about 21 sites and about 170 test lines.
- **Spec.** §4.1.1 changes with them.
- **When.** All of this happens in the pin-bump PR that moves the mr-codec git
  rev (`25051a06` today) past the renumber, so nothing breaks before then.
- **mr-gui.** It gains `-h`/`--help` exiting 0 (CLI-17), usage 64, and 32
  for missing libraries or a child that fails to start. `sitting_spawn.rs`
  shows the decoded names of `mr-sitting`'s status.

## 8. Defects found along the way

These were true before the implementation. Each is fixed on the `fx/*`
branches except where noted.

- mnemonic-gui shows a green "Repair Applied" badge for `md descriptor
  --verify-against` NOT spend-equal (§7j; source reading).
- `me` panics with 101 on a non-UTF-8 argument, before its argv secret guard
  runs (`me --hex $'\xff\xfe'`).
- Every binary exits 0 with stdout closed, and the output is lost (R8).
  Still true on Windows.
- `mk repair` exits 0 on cards that `mk decode` rejects with 1 or 3.
- `ms inspect` exits 0 while printing "FAIL: would NOT decode". This one is
  by `ms`'s locked table, so it changes only through decision 8.
- `mnemonic`'s code for the same input depends on whether stdout is a
  terminal.
- `md descriptor --verify-against X --chain 7` prints its verdict and only
  then fails.
- A named `mt --bitcoin-cli` that fails exits 0. Fixed: 32, and a named path
  that does not exist is 32 too, so it is no longer an offline switch (R9).

## 9. Rollout

There are no existing card holders, and the codes are read by a few scripts
and two GUIs, so this is one coordinated cutover. A number cannot have an
alias, and every old code is a valid new sum, so no stage may let an old
binary and a new GUI meet without a version check. The order:

1. The shared constants and the bit-set type in mnemonic-io-lib (decision 4):
   a crates.io publish, then pin and `vendor/` moves in the five consumers
   that take it from crates.io (`me` has a path dependency). The mr-codec copy and its test land at the same time.
   - **Interim (revision 6).** This step did not come first. The binaries
     were built with local copies (decision 4), and 0.1.4 is unpublished.
     Before it is published it gains `exit::add` and `Bits::from_code`,
     which it now has, so that no consumer keeps a shim. After the publish,
     each binary replaces its copy, the JSON `kind` names are settled
     (R14), and the toolkit's legend test switches to a whitespace-normalised
     comparison, since the library's legend is one line.
2. The network rule's unbuilt units are built straight onto bit 8, so none of
   that work has to be redone.
3. The eight binaries, each with its tests, help legend (R11), JSON fields,
   manual chapter and gate scripts.
4. In the same window: mnemonic-gui (bit decoding, and scheme detection from
   the legend, §7j) and refugium-wallet (the mr-codec pin bump), plus the
   manual appendix.


`mr` is about two thirds of the test churn, so its PR is the long pole
(decision 10). The others can land in any order behind step 1, with two
limits. Step 4 must not trail them. And some tests run a sibling binary
(`jf8_pr64.rs`, `jf8_b_advice.rs`, `ms_remedy_runs.rs`, and the toolkit's
manual checks through `ci/toolkit-manual.rev`), so a repo's PR updates
those expectations to match whichever sibling version its CI builds.

## 10. Disposition of review findings

### Round 0

Some of these dispositions were refined in later rounds; where they differ, the round 1 and round 2 rows and the body text win.

| finding | disposition |
| --- | --- |
| C1 (20 is ambiguous) | Accepted as a design property and stated in §3: bit 4 is defined by the operator's action, and it over-warns but never under-warns. |
| C2 (REFUSED vs INVALID undecidable; §6i misread) | Replaced by the four ordered tests in §3, with the ambiguous cases worked through. §6i is a taxonomy, not the test (R3). |
| I1 (`mk` incomplete set) | It is 16, per D26 (§3, §7b). |
| I2 (atomic repair) | R1. |
| I3 (R6 reverses the mt spec) | Now decision 5, with the cost stated. |
| I4 (R4 too wide; the flag vocabulary) | R4 now keeps today's behaviour: an override clears the bit it overrides, whatever the prefix. |
| I5 (network-rule fallout) | Listed in §6; build order in §9. |
| I6 (R2's example; argv-guard precedence) | R2 rewritten. |
| I7 (usage vs a bad value; "Keep 1") | §3 test 1 (the parser or a documented closed list), with "Keep 1" kept. |
| I8 (unverified vs a warning; R5 scope) | Bit 4 is set only on a result the tool marks unverified. `restore` with no reference sets it; R5 is split into report, world-state and classify cases. |
| I9 (no class for other failures) | R13. |
| I10 (R9 against `mt`) | R9 and §7d agree: a named helper that fails sets 32. |
| I11 (reversals missing from §6) | §6 expanded. |
| I12 (consumers missed) | Added to §7, and R14 covers the JSON fields. |
| I13 (cutover; "has output") | §7j and §9 add a version floor on every run; R12 bans deriving output from the code. |
| I14 (R7 picks auto-repair semantics) | R7 now keeps behaviour and fixes only the code: 2 in both modes. |
| I15 (fail fast) | R1. |
| I16 (pin counts) | Corrected from the survey reports. |
| Minor and Nit | §1 rewritten (`me` usage 2 both ways; `mt` and `me` READMEs); closed stdout is every binary; `BadArg` is split in §7a; the evidence directory exists; decision 4 covers mr-codec's CI; §2 arithmetic corrected; the legend's first sentence covers 64, 101 and 128+. |

### Round 1

| finding | disposition |
| --- | --- |
| R1-I1 (refusals placed against the tests) | §3's worked table and §7 corrected: the published-vector root and the `export-wallet` secret slot go to 2, "feature not compiled in" to 32, and the digest-shaped phrase to 8 on its existing flag. |
| R1-I2 (closed list) | Test 1 is now "the command line alone", and decision 2 narrows E45's list. `me seal --iterations 5`, two readers on stdin, and an unset `@env:` are placed in §3. |
| R1-I3 (NO vs INVALID; operator stop) | NO is now "well-formed, and a comparison came out no" (test 4). A spent input is 1. Operator stop is decision 9 (130). "The first failure wins" is stated. |
| R1-I4 (incomplete set: three codes) | Decision 7 picks 16 everywhere. Decision 6 checks per string, so an incomplete set is not 2. R1 says a repair that prints its result sums its bits. |
| R1-I5 (`restore` with no reference; bit 4 by printed text) | `restore` with no reference stays 0. Bit 4 means "checked and could not confirm", and not asking for a check is not a failure (§3). |
| R1-M1 to M12, N1 to N3 | Folded: mt line numbers (normative copy); 66 `Refusal::new` sites and the composition of the 19 rows; HARD mismatch stated in decision 1 and §6, with the override claim removed from the legend; the gate-script check; `me` pins on a non-zero basis; the extra consumers (§7f, §7g, §7e, §9); `mk inspect` and `mt verify`/`inspect` in decision 5, with its alternatives; R13's table test in place of a grep; the range guard and Windows note (§3, R12); the GUIs' own codes (R12); `ms verify`'s message (R5); `me --preview` (§7e); "channel" defined, and `practice --dir` placed; the io-lib publish (decision 4, §9); `45-mr.md:53-60`; §7a's network line; the mnemonic-refugium CLAUDE.md hazard (decision 6, §6); R10's wording. |

### Round 2

| finding | disposition |
| --- | --- |
| R2-I1 (test 3 caught overridable "no" answers) | Tests reordered: network (3), comparison came out no (4), safety or policy rule (5), input (6), bug (7). An overridable mismatch is 1, as decision 11. `mr member` with no keys is 2. |
| R2-I2 (flag values) | Decision 2 defines setting against input on argv, with examples. "Keep 1" holds: hot-family values are input, so 2. |
| R2-I3 (`md` incomplete set) | Decision 7 states `md`'s refusal today and what changes; §7a lists it. |
| Minor | §10 round-0 note; review file present; 130 added to R12 and §3; test 1 names the argv-guard exception; test 7 sends the rest to 101; the foreign hand-back placed; the typed "stop" placed (decision 9, with an alternative); decision 6's sums (18 and 22). |

### Round 3

| finding | disposition |
| --- | --- |
| R3-I1 (parser rejects an input value) | Decision 2: the class does not depend on who checks. A malformed input value is 2 even when clap rejects it; E45's `--hot-parent-fp` entry moves with it. |
| R3-I2 (test 4 against §7g and §7h) | Test 4 now lists what counts as a match and excludes limits, caps, deadlines and deltas. §7g splits `--accept-*` into mismatches (1) and guardrails (8); §7h's foreign hand-back is 1. |
| Minor and Nit | Test 5 covers destinations (M1). Test 0 places the operator stop, and the legend separates 130 from signals (M3). The toolkit's `repair --md1` today value (M4). `--expect-fingerprint zz` placed by decision 2 (M5; the rest of M5's cases are placed by the tests in the implementing PRs). Five crates.io pins plus `me`'s path dependency; "seven" of eleven. M2 (an ms1 on `me --hex` argv is 8, from a file 2) is intended: the argv guard runs first. |

A mechanical check of the round-3 fold (Sonnet) found 1 Important (§7f's
"parser-shaped" wording) and 4 Minor (decision 2, §7g and test 1 lacked the
input-value carve-out; §5 had no operator-stop row). All five are folded.

### Round 4

| finding | disposition |
| --- | --- |
| R4-I1 (`mr`'s incomplete sets) | Decision 7 states `mr`'s behaviour today (lone chunk 5 or 0; two or more chunks, incomplete: 2, no output). `md` and `mr` both change. A lone chunk is 16, and only a string that is not chunked is 20. §7g lists all of it. |
| M1 to M9 | Folded. More silently correcting verbs listed (decision 5, §7f). Decision 8's false "as `md` and `mk` already do" removed. The argv guard wins for a malformed `--from` (§7f). §3's example of 20 replaced. `mk`'s "UNVERIFIED" message at 16 (decision 7). N10 and `MNEMONIC_NETWORK_JSON` (§6). A named `ME_PREVIEW_BIN` that is missing is 32 (§7e). A failed wipe during a stop exits 32 (R10). Tips, `--digest`, `--set-id`, `--application rsa` and distinctness placed (§3); signal kills in GUIs (R12). |
| N1 to N5 | `mr`'s post-parse guard (R2). "Destination" in REFUSED. "Nothing printed" (decision 6). `mt` verb named. The shell example says "operator stop". N4: the mt line numbers are master's (`:3384-3385`, `:810-827`); the fix-PR head has them at `:3470-3471` and `:813-830`. |

### Implementation reviews

Each repo's implementation had three review steps. An Opus agent that did not
write the code reviewed the whole diff, and the implementer folded it. A Fable
agent then ran a final independent round across repos. The final findings are
folded by Opus implementers, in the commits named below where they have
landed. Every branch is `fx/<repo>`, merged into each repo's
`claude/project-thread-o57s2k` branch for its open PR.

Opus reviews are at `design/agent-reports/exit-codes-impl-review-2026-11-26.md`
in each repo, except `me`'s, which is
`design/agent-reports/exit-codes-2026-11-26/impl-review-me.md` here. The Fable
final reviews are in this repo's `design/agent-reports/exit-codes-2026-11-26/`:
`final-review-md-mk-ms.md`, `final-review-mt-me-spec.md` and
`final-review-toolkit-gui.md`, `final-review-mr.md` and `final-review-rw.md`.

| repo | implementation | Opus review | Opus fold | Fable final review | final fold |
| --- | --- | --- | --- | --- | --- |
| descriptor-mnemonic (`md`) | `1a7063c`, `e2ae76f`, `09e9f29` | 0C, 2I, 7M, 7N | `b79bfa5`, `e178792` | md-mk-ms: NOT GREEN, 0C, 3I, 9M, 6N (the three binaries together) | `8c85bf9` |
| mnemonic-key (`mk`) | `506c95f`, `f3a17c8` | 0C, 2I, 6M, 4N | `55a6e68` | md-mk-ms, as above | `9a4e0aa`, `8f74a45` |
| mnemonic-secret (`ms`) | `ff7cb4e`, `d54e140` | 0C, 1I, 4M, 5N | `ce20d5a` | md-mk-ms, as above | `9da35f9` |
| mnemonic-transaction (`mt`) | `60151a4`, `56e6ebc`, `bb996d0`, `608c338` | 0C, 2I, 5M, 5N | `d49dabf` | mt-me-spec: 0C, 2I, 11M, 6N (with the spec) | `b7b5480`, SPEC_mt re-sync `ffa1f45` |
| mnemonic-engrave (`me`, mnemonic-io-lib) | `8ff5711`, `72273ea`, `82d2bb3`, `236c576`, `cf88af4` | 0C, 1I, 6M, 8N | `f0a9654`, `27234b9` | mt-me-spec, as above | `9659335`, `3183c14`, `946147a` |
| mnemonic-toolkit (`mnemonic`, manuals) | `e51607b8`, `71dc68ac`, `f8b8d19c`, `a32c3bd2` | 1C, 6I, 11M, 5N | `2a3ce852`, `5931f7a7`, `1cfa8950`, `a0ac6ad0` | toolkit-gui: NOT GREEN, 0C, 4I, 14M, 6N | `6205316f`, `1991f3e9`, `ed35d630`, `da439125`, `1c84f771` |
| mnemonic-gui | `a5fa42d`, `691f9ce`, `2545d0c` | 1C, 2I, 5M, 3N | `2168783` | toolkit-gui, as above | `70970d2` |
| mnemonic-refugium (`mr`, `mr-sitting`) | `26ee6dd42`, `6e6398d09`, `3e2a50c5e` | 0C, 4I, 7M, 3N | `f2a44d069` | `final-review-mr.md`: 0C, 2I, 7M, 9N | `26504288a`, `534a5d652`, `711373bef`, `27137cc6d`, `9131377b0` |
| refugium-wallet (mr-gui, refugium-view; mr-codec pin to `9131377b`) | `1225400a` | 0C, 1I, 5M, 5N | `8cea3377` | `final-review-rw.md`: 0C, 1I, 2M, 5N | `e76999a7`, `013f3ebf`: the pin also brings mr's E101 card-check row, so the FOLLOWUP `rw-mrgui-card-check-row` is done here; two scoped re-reviews moved the check before engraving (`0902150c`, `46509782`, `c7c8fea2`; Brian's ruling pending) |

Still owed across the work, from the reviews:

- `mr`'s own process gate's whole-diff review
  (`agent-reports/whole-diff-mr-<phase>.md`) is owed, and its three typed-quit
  pty rows (`w1t_pty.rs`) were changed to 130 but could not run here.
- Gates nobody ran: `mt`'s `scripts/mutate-refusals.sh`, `me`'s
  `fix-me-dev-gates-gate.sh` (no `--cli` mode), `mr`'s regtest rows, and
  mnemonic-gui's tutorial snapshots, `schema_mirror` and secret-channel
  re-measurement against the renumbered binaries.
- Toolkit manual `45-mr.md` and `46-mr-sitting.md` passages that still give
  old codes (toolkit-gui final review I1), and the manual lockstep FOLLOWUPS
  for `mk` and `ms`.
- The network-rule changes of §6, the mnemonic-io-lib publish (§9), and the
  per-verb help lines that R11 lists as owed.

### Revision 6: the final review's 27 spec items

The list is at the end of `final-review-mt-me-spec.md`. "Ruling" means a
coordinator ruling of §2a; "open" means §2b.

| item | disposition |
| --- | --- |
| 1 (worked table names `mt verify`) | §3: `mt encode` on a spent input is 1; `mt verify --bitcoin-cli` is 64. |
| 2 (R9: named, fails) | Ruling 1. R9 rewritten; §7d and §8 say a named missing path is 32 and not an offline switch. |
| 3 (`mt --offline`) | Open, §2b item 1. |
| 4 (R1: does 16 survive) | Ruling 2. R1 amended. |
| 5 (complete set that fails reassembly) | Ruling 3: 2, nothing printed. Decision 6's table; §7g. |
| 6 (groups with different set ids) | Open, §2b item 3. |
| 7 (§7d's oracle) | §7d: 16 means the transaction re-derives its id; witness bytes are not covered. |
| 8 (`OK` with 16) | R5: allowed, with the correction named beside it. |
| 9 (what counts as used; discarded duplicate) | Transliteration is not a correction (§7d). The duplicate is open, §2b item 2. |
| 10 (test 1 against self-contradicting values) | Ruling 4. Tests 1, 4 and 6. |
| 11 (settings and inputs list) | Appendix A, with the `@env:` rule. |
| 12 (wrong passphrase) | Ruling 5. §3 "Passphrases and passwords", §7e, §7f. |
| 13 (a flag the input makes necessary) | Ruling 7. §3 test 5 and REFUSED row; §7e. |
| 14 (missing argument against empty input) | Ruling 4. §3 worked table (`me sysw hash` with no records, 64). |
| 15 (§7h against decision 9; the lock) | Ruling 6. §7h: the mainnet sentence is 130, the lock 32. |
| 16 (R14 and `kind`) | Ruling 10. R14: owed for the io-lib publish; repair's additive fields recorded. |
| 17 (GUI detection) | Ruling 8. R11, §7j and §9. |
| 18 (colour of 16 alone) | Ruling 8: amber. Open for Brian, §2b item 4. |
| 19 (per-verb help) | Ruling 9. R11 names which binaries have it and which owe it. |
| 20 (closed stdout and stdin) | R8: the platforms; a closed stdin is 2. |
| 21 (helper failure the input caused) | R9 and §7e: 32. |
| 22 (R2 costs a referral) | R2: advice that needs the input cannot precede a usage error. |
| 23 (`--out` at 22) | Ruling 3. Decision 6: never written to `--out`. |
| 24 (`inspect` of a share) | R5: "or a share would not combine". |
| 25 (decision 4's order) | Ruling 11. Decision 4 and §9 step 1 record the interim state. |
| 26 (`MixedNetwork` and N10) | §6: listed with the unbuilt network-rule units, which are not done. |
| 27 (§7e's gate-script count) | §7e names the six. |

Other corrections in this revision: §7a's `--allow-invalid-miniscript`
exists only on md PR 76, which is unmerged (§7a); `restore --from xpub` is 2
(§3, §7f); `md`'s wire-version exception at 22 (decision 6); the status line;
and the "today" note under Evidence.

## Appendix A. Settings and inputs, by tool

The rule is decision 2's: a setting is judged from the command line, so a bad
one is 64; an input is the data the verb works on, so a bad one is 2, whoever
rejects it. A setting is checked before any input is read (R2). For `@env:`
references: a malformed variable name is 64 (judged from argv), an unset
variable is 32, and a value that is not UTF-8 is 2.

Each tool keeps its own list, and they agree on the principle. As implemented:

| tool | settings and other command-line checks (64) | inputs (2) |
| --- | --- | --- |
| `md` | `--chain`, `--index`, `--count`, `--group-size`, `--separator`, `--network`, `--emit` | cards, `--key`, `--fingerprint`, `--path`, `compose --wrapper`, `--seat`; a `--chain` the card does not have (R2) |
| `mk` | `derive --path`, `--index` | cards, `--xpub`, `--chunk-set-id`, `--policy-id-stub`, fingerprints, origin paths |
| `ms` | share K and N, two readers on stdin (including `--in` naming stdin's file) | cards, phrases, shares |
| `mt` | `--separator`, the flag combinations of `SPEC_mt` §10.10, `verify --bitcoin-cli` | the PSBT or transaction, `--input-value`, pasted txids |
| `me` | `seal --iterations`, `sysw pack --passphrase-words`, the output mode | cards, descriptors, records, an iteration count read from a container |
| `mnemonic` | `seed-xor split --shares`, `ms-shares split --threshold` and `--shares`, `convert` target settings | `--template` scheme names, `--expect-fingerprint`, `--expect-xpub`, `--from`, `--slot`, descriptors |
| `mr`, `mr-sitting` | `--network`, `--lead`, `--epochs`, `--letters`, `--horizon`, `--delta-blocks`, `--anchor-time`, unknown glossary terms and practice cases | `--time`, `--height`, `--epoch`, `--slot`, `--index`, `--digest`, `--set-id`, `--hot-parent-fp`, the hot family's values, cards, keys and records; the list is `INPUT_FLAGS` |
