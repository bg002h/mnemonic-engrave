# R3 re-review: SPEC_constellation_exit_codes.md (DRAFT, revision 3, 2026-11-26)

Reviewer: independent, read-only. Spec:
`/home/user/mnemonic-engrave/design/SPEC_constellation_exit_codes.md` ("S:n" =
spec line n). Scope, per the proportional re-review rule: the three round-2
Importants, and defects the revision-3 fold introduced (the seven ordered
tests, decision 2's setting-versus-input line, decisions 7, 9 and 11, R12's
130). Evidence: the surveys, `review-r0.md` to `review-r2.md`, the sources
under `/home/user/wt/<repo>`, and about 40 fresh runs of the built binaries
(stdin `</dev/null`, scratch `/home/user/wt/ec/tmp-review3/`). No repository
was edited. Not re-litigated: the bit-flag direction, the six bit names, 64
for usage, C1's over-warning, decisions framed as recommendations.

Verdict: **NOT GREEN: 0 Critical, 2 Important, 5 Minor, 3 Nit.** R2-I3 is
fully fixed. R2-I1 and R2-I2 are fixed for every case round 2 named, and each
leaves one place where the spec still gives two answers for a measured case.
Both folds are one sentence in §2 or §3 plus one or two §7 rows. No survey
needs redoing.

---

## 1. Round-2 Importants: is each fold a real fix?

| finding | fold | verdict |
| --- | --- | --- |
| R2-I1 (test 3 caught overridable "no" answers) | tests reordered S:243-257; decision 11 S:205-212; test 6's `mr member` example S:254-256 | **Real for the named cases.** `restore --expect-fingerprint` (re-run: 4 today, no stdout) now reaches test 4 before the refusal test and is 1; a network mismatch is caught first by test 3 and stays 8; `mr member` with no keys is 2, because test 5 now needs a *waived safety or policy rule*, not a flag that supplies data. The reorder was not carried into §7g and §7h (R3-I2). |
| R2-I2 (flag values split between 64 and 2) | decision 2 S:95-105, S:110-112 | **Real for values the tool checks itself.** The setting-versus-input line places every row of round 2's table: `me seal --iterations 5`, `--shares 1` and `ms` K/N are 64; `md --key`, `--fingerprint`, `compose --wrapper`/`--path` and the hot family are 2, and "Keep 1" survives as "not usage". It does not say what happens when the *parser* rejects an input value (R3-I1). |
| R2-I3 (decision 7's `md` claim) | decision 7 S:161-176; §7a S:526-532 | **Real.** Re-run: `md repair C1 C2BAD`, `md repair C2BAD` and `md repair C1` all give 2 with nothing on stdout; `md repair C1 C2BAD C3` gives 5; `mk repair` on one damaged chunk of two gives 5; `mnemonic repair --mk1` on the same string gives 4. Decision 7 now says all of this, says what changes in `md`, and §7a lists both changes. One "today" value is incomplete (R3-M4). |

---

## 2. The seven tests, re-run

### 2a. Against §3's worked table (S:265-285)

All 19 rows come out as the table says when the tests are asked in order,
with two notes:

- `mr-sitting practice --dir` whose files exist, 8 (S:277): right by test 5's
  example ("an overwrite"), not by its condition (R3-M1).
- "an `ms1` to `me --hex`", 2 (S:267), and "an xprv given as an xpub", 2
  (S:268): right for a file or stdin. On argv, test 1's exception runs first
  and gives 8 (R3-M2).

### 2b. Against refusals from the surveys

"Today" is a fresh run unless marked (s) for the survey's value.

| # | refusal | today | by the tests | §7 says | agree? |
| --- | --- | --- | --- | --- | --- |
| 1 | `mnemonic restore --expect-fingerprint deadbeef` | 4 | test 4: 1 | 1 (S:635) | yes |
| 2 | `mnemonic convert … --network testnet` on a mainnet xpub | 2 (s) | test 3: 8 | 8 (S:645) | yes |
| 3 | `mnemonic compare-cost … --max-conditions 1` | 3 (s) | test 5 by its "limit" example: 8 | 8 (S:646) | yes, see R3-I2 on test 4's width |
| 4 | `mnemonic export-wallet` given a phrase slot | 2 (s) | test 5 no (nothing waives), test 6: 2 | 2 (S:646-647) | yes |
| 5 | `mnemonic import-wallet --blob /nonexistent` | 1 (s) | test 2: 32 | 32 (S:654) | yes |
| 6 | `mt encode` to a 0644 stdout | 1 (s) | test 5: 8 (`--allow-world-readable`) | 8 (S:592) | yes |
| 7 | `mt verify --json --quiet` | 1 (s) | test 1: 64 | 64 (S:585) | yes |
| 8 | `me seal` of a seed without `--seal-secret` | 3 (s) | test 5: 8 | "by the §3 tests" (S:607) | yes |
| 9 | `me --hex --in ms.txt` | 3 | test 6: 2 | 2 (S:267) | yes |
| 10 | `md encode … --fingerprint @0=zz`, `--key @0=notanxpub` | 1, 1 | decision 2: 2 | 2 (S:104) | yes |
| 11 | `md compose --wrapper bogus`; `--path 5of3` | 1, 1 | decision 2: 2 | 2 (S:525) | yes |
| 12 | `mr member` given no keys on a card with a seat it does not key | 2 (E45's closed list, mr spec `:27511-27512`; not run) | test 5 no (a flag that supplies data waives nothing), test 6: 2 | 2 (S:108-109, S:254-256) | yes |
| 13 | `mr … decode <an md1>` | 2 | test 6: 2 | 2 (S:680-681) | yes |
| 14 | `mr … verify … --hot-parent-fp zz` | 2 (parser) | "the parser's errors": 64; "a fingerprint" is input: 2 | 64 (S:676-677) | **two answers, R3-I1** |
| 15 | `mnemonic restore --template bogus` | 64 (parser) | parser: 64; "template", "wallet shape": 2 | 64 (S:631) | **two answers, R3-I1** |
| 16 | `mr create --template-preset nosuch` | 1 | input: 2 | not placed | by who checks it, R3-I1 |
| 17 | `mr` plate whose policy-id stub is not the card's (`--accept-stub-mismatch`) | 1 (source: `plates.rs:1094-1114`) | test 4 and decision 11: 1 | 8 (S:683-684) | **no, R3-I2** |
| 18 | `mr-sitting` hand-back that is another plan's | 2 (s) | test 4: 1 | 2 (S:695-696) | **no, R3-I2** |
| 19 | `mr-sitting practice --dir` whose files exist | 2 (s) | test 5 by example: 8 | 8 (S:702) | yes, R3-M1 |
| 20 | `mnemonic restore --expect-fingerprint zz` | 4 ("MISMATCH") | malformed input: 2 | "expectations" to 1 (S:635) | not placed, R3-M5 |

---

## 3. Important

### R3-I1. Decision 2 does not say which rule wins when the parser rejects an input value (R2-I2, one case left)

Decision 2 gives two rules:

- S:92-94: 64 covers "the parser's errors, and the tool's own checks on its
  *settings*".
- S:97-101: an input is "a card string, a key, a fingerprint, template text,
  a wallet shape's parameters ... the hot family's values", and "a bad one is
  2".

Several binaries validate input values in the parser (a clap `value_parser`
or value enum). For those values both rules apply and they disagree. Re-run:

| command | today | who checks | "parser's errors" | "input" |
| --- | --- | --- | --- | --- |
| `mr … verify … --hot-parent-fp zz` | 2 | clap (`create.rs:305`, `verify.rs:75`) | 64 | 2 (a fingerprint) |
| `md encode … --fingerprint @0=zz` | 1 | `md` | | 2 (S:104) |
| `mnemonic restore --template bogus` | 64 | clap | 64 | 2 (a wallet shape) |
| `md compose --wrapper bogus` | 1 | `md` | | 2 (S:104) |
| `mr create --template-preset nosuch` | 1 | `mr` | | 2 |
| `mr … verify … --digest zz` | 2 | clap | 64 | 2 if a digest is input |

So after the cutover a malformed fingerprint is 64 in `mr` and 2 in `md`, and
an unknown wallet shape is 64 in `mnemonic` and 2 in `md` and `mr`. This is
the "bad value: 1 or 2 by who checks it" collision the mr survey recorded
(`mr.md:104`), with the numbers moved to 64 and 2. Round 2 named the
`--hot-parent-fp` and `--hot-child` pair explicitly; revision 3 defines
"input" so that both are input, and §7g (S:676-678) and §6 (S:487-489) keep
`--hot-parent-fp` at 64 as a closed-list entry "on the command line alone".

The owner is told in decision 2 that a malformed key or fingerprint "gives
2". He is not told that this holds only where the tool checks it after the
parse. That is a hidden trade-off, and the per-binary table test R13 asks for
needs one answer per row.

Fix (one sentence in decision 2, and the owner's choice):

- either "a value the argument parser rejects is 64 whatever it is; the
  setting-versus-input line applies only to checks the tool makes after the
  parse", which accepts the cross-tool split and says so;
- or "an input value is 2 in every tool", which means the parser stops
  validating those values (`--hot-parent-fp`, `--hot-record`, `--hot-key`,
  `--digest`, `--template`) and §7f, §7g and §6's E45 bullet list them.

### R3-I2. Test 4 now runs before tests 5 and 6, and §7g and §7h were not re-run against it (R2-I1, mirror case; new with the reorder)

Test 4 (S:249-250): "Is the input well-formed, with a comparison against
something else coming out no? Then 1, even when an override exists (decision
11)." Decision 11 (S:205-209): "A mismatch that an override can accept is NO
(1)".

- **`mr`'s `--accept-stub-mismatch`.** Its help: "Take a plate whose
  policy-id stub is not this card's". That is an overridable mismatch against
  a reference, so test 4 and decision 11 give **1**. §7g sends "the hot-cold
  rules and seating guardrails that `--accept-*` flags clear" to **8**
  (S:683-684), and this flag is a seating input
  (`mr-codec/src/seating.rs:310-311`, `plates.rs:1094-1114`). §7k then tells
  refugium-wallet to use "the class §7g gives that refusal" (S:747-749).
- **`mr-sitting` hand-back from another plan.** Revision 3 places it at **2**
  (S:695-696). The file is well-formed and its comparison with the plan came
  out no, so test 4 gives **1**. Round 2 raised this as unplaced (R2-M6); it
  is now placed against the test.
- **Width.** Test 4 does not carry the NO row's list ("a reference, a phrase,
  a template, the chain, an expectation", S:219). Read alone, "a comparison
  against something else" also covers a count against a cap
  (`--max-conditions`), a deadline against the tip (`--accept-deadline`,
  `--accept-tip`, `--accept-anchor`) and a delta against a floor
  (`--small-delta`). Test 5's examples ("a guardrail, a limit") are reached
  only after test 4 has said no, so the worked table's 8 for the cap rests on
  the reader knowing the intent.

Round 2's defect was that overridable mismatches fell to 8 by the tests and 1
by the tables. Now the tests give 1 and one §7 sentence gives 8, with
`mr`'s 1,700 assertions on 1 as the place where it will be decided row by row.

Fix: one clause in test 4 saying what kind of comparison it means (the verb's
own answer, or the user's material matched against a reference, phrase,
template, chain state or expectation; a threshold, date or limit rule is test
5; an input of the wrong kind or owner is test 6 if that is the intent for the
hand-back). Then split §7g's `--accept-*` line into the mismatches (1) and the
guardrails (8), and give the hand-back row its reason.

---

## 4. Minor

- **R3-M1.** Test 5's condition is a rule "which a flag, network or channel
  would waive" (S:251-252). Three of its own examples have no such waiver:
  `practice --dir` whose files exist (`mr-sitting practice --help` offers no
  overwrite flag; the cure is another folder), the same file as input and
  output (`mt`, `mk`, `md --out`), and a world-readable destination in `me`.
  "Channel" is defined as argv, stdin, a file or a terminal (S:234-236), so
  another path is not another channel. Read by the condition these fall to
  test 6 and then to 101. The examples carry them to 8; add "or another
  destination" to the condition.
- **R3-M2.** Two worked rows depend on the channel and do not say so. An ms1
  given to `me --hex` on argv is a secret on argv, so test 1's exception gives
  8; from a file it is 2 (S:267). Today both are 3 (re-run). The same holds
  for "an xprv given as an xpub" (S:268) wherever the tool's argv guard knows
  an xprv. Add "from a file or stdin" to the two rows.
- **R3-M3.** Decision 9 and R12, three loose ends.
  - The tests have no place for an operator's stop: none of tests 1 to 6
    fits, so test 7 reads it as 101. One line before the tests ("a signal or
    an operator's stop is not classified here; R10") closes it.
  - R12 keys on the number 130 (S:403-405). A GUI that starts the child
    itself sees no number when the child is killed by a signal on Unix
    (Rust's `ExitStatus::code()` is `None`), so a Ctrl-C kill is "ended
    abnormally" and only `exit(130)` is "stopped". Say which is meant.
  - The R11 legend (S:396-401) and the shell example (S:296) still call 130
    "killed by a signal" (R2-N3, not folded). For `mr-sitting`, a typed
    "stop" is an ordinary outcome and its help would describe it as a kill.
- **R3-M4.** Decision 7 and §7f describe the toolkit's incomplete-set case as
  4 today (S:165-166, S:650). That is the mk1 path. Re-run:
  `mnemonic repair --md1 <one damaged chunk of three>` gives **2**
  ("post-correction decode failed: chunk set incomplete"), the same atomic
  refusal `md` has. "16 everywhere" decides the new value, so only the
  "today" side and §7f's row are incomplete.
- **R3-M5.** Not placed, each decided by the tests against today's code:
  - `mnemonic restore --expect-fingerprint zz` and `--expect-xpub notanxpub`
    print "MISMATCH" and exit 4 today (re-run). A malformed expectation is a
    bad input by decision 2, so 2, not the 1 that §7f gives "`restore`
    expectations".
  - `Bip388VerifyDistinctness` and `DescriptorReparseFailed` (4 today),
    carried from R2-M6.
  - `mr create` and `upgrade` missing mandatory inputs other than the tip
    (exit 1 by E52, test `jf2_54_missing_inputs_exit_codes`). Decision 3 says
    a missing required argument is 64; §6's E45 bullet does not list the
    change.
  - `md compose --wrapper wsh --unspendable liana` (1 today): a flag clash
    judged from the command line (64 by test 1), or a "bad `compose` value"
    (2 by S:525).
  - `mnemonic derive-child --application rsa` (2 today).

## 5. Nit

- **R3-N1.** Decision 4 (S:125-127): "a publish plus six pin and `vendor/`
  moves". Five binaries take mnemonic-io-lib 0.1.3 from crates.io (`md`,
  `mk`, `ms`, `mt`, `mnemonic`); `me` uses a path dependency, and `mr` does
  not depend on it. R2-N1, re-checked in the `Cargo.toml` files, not folded.
- **R3-N2.** §2's lead-in (S:69-70): "The first four shape the scheme. The
  other six ..." There are eleven decisions, so seven.
- **R3-N3.** R12 (S:403-405) names 0, 64, 101, 130 and "outside 0..125". It
  does not say what a GUI shows for 65 to 125 other than 101, which §3 calls
  a bug (S:230).

## 6. Checked and sound

- **Decision 7.** True as written for `md`, `mk`, `mr` and the toolkit's mk1
  path (runs in §1). It agrees with decision 6 (per-string check; an
  incomplete set is not a failure) and with R1 (an uncorrectable string is
  still atomic 2).
- **Decision 9.** A typed "stop" is 130 at every prompt including the mainnet
  confirmation; end of input is 2; a wrong answer at the mainnet prompt is 8
  (§7h). The three endings round 2 asked for are all named, and the
  alternative is stated.
- **Decision 11.** Consistent with test 4, R4, the NO row and §7f. The
  alternative (every overridable refusal is 8) is stated with its cost.
- **Test order.** System failure (2) before the verdict tests, network (3)
  before NO (4), and "the first failure wins" (S:259) give one answer for the
  mixed cases tried: a missing file plus a bad card (32), a wrong-network key
  that also mismatches (8), a corrected card that then mismatches (16 + 1).
- **Test 1's exception** now carries the argv guard (R2-M4), and test 7 sends
  the rest to 101 (R2-M5).
- **§10.** The round-0 note is present; `review-r1.md` and `review-r2.md` are
  in `design/agent-reports/exit-codes-2026-11-26/` and match the working
  copies.

---

## Summary

- Critical: 0. Important: 2 (R3-I1 parser-rejected input values are 64 by one rule of decision 2 and 2 by the other; R3-I2 test 4 gives 1 where §7g gives 8 for `--accept-stub-mismatch` and §7h gives 2 for a foreign hand-back, and test 4's wording is wider than the NO row).
- Minor: 5. Nit: 3. Round-2 Importants: R2-I3 fixed; R2-I1 and R2-I2 fixed for their named cases, each with one residual carried as R3-I2 and R3-I1.
- Verdict: **NOT GREEN** (0 Critical, 2 Important). Both folds are one sentence in §2 or §3 and two §7 rows.
