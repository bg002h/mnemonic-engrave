# R2 re-review: SPEC_constellation_exit_codes.md (DRAFT, revision 2, 2026-11-26)

Reviewer: independent, read-only. Spec:
`/home/user/mnemonic-engrave/design/SPEC_constellation_exit_codes.md` ("S:n" =
spec line n). Scope, per the proportional re-review rule: the five round-1
Importants, defects the revision-2 fold introduced, and spot-checks of the new
factual claims. Evidence: the surveys, `review-r0.md`, `review-r1.md`, the
sources under `/home/user/wt/<repo>`, and about 25 fresh runs of the built
binaries (stdin `</dev/null`, scratch `/home/user/wt/ec/tmp-review2/`). No
repository was edited. Not re-litigated: the bit-flag direction, the six bit
names, 64 for usage, C1's over-warning, decisions framed as recommendations.

Verdict: **NOT GREEN: 0 Critical, 3 Important, 8 Minor, 3 Nit.** Two of the
five round-1 Importants are fully fixed (R1-I3, R1-I5). The other three are
mostly fixed, and each leaves one place where the spec gives two answers. All
three folds are a few sentences in §2 and §3 plus one or two §7 rows.

---

## 1. Round-1 Importants: is each fold a real fix?

| finding | fold | verdict |
| --- | --- | --- |
| R1-I1 (§7 rows against the tests) | S:229, S:228, S:238, S:232; §7a S:493-494, §7f S:601-603, §7g S:637 | **Real for the four named rows.** The published-vector root and the `export-wallet` secret slot are 2, "feature not compiled in" is 32, and the digest-shaped phrase is 8 on a flag that exists (`ms-cli/src/cmd/hashlock.rs:276`). A new mismatch between the tests and §7 came in with test 4 (R2-I1). |
| R1-I2 (closed list) | test 1 S:208-209, decision 2 S:91-100, S:239-241 | **Partial.** "The command line alone" is a clear test and it places the three named cases. Decision 2 and §7a then send flag values that the command line alone decides to 2 (R2-I2). |
| R1-I3 (NO vs INVALID; operator stop) | NO row S:184, test 4 S:213-214, decision 9, S:217, S:242-243 | **Real.** The 1/2 boundary is stated, a spent input is 1, a stop has a code, and the prompt case is worked. Leftovers are Minor (R2-M3, R2-M6, R2-M7). |
| R1-I4 (incomplete set: three codes) | decisions 6 and 7, R1 S:294-296, bit-4 row S:186 | **Partial.** `mk`, `mr` and `mnemonic` now agree on 16, "an incomplete set" is gone from the bit-4 row, and a repair that sets 2 is said to print. Decision 7's claim about `md` is false, and `md` is left at a different code (R2-I3). |
| R1-I5 (`restore` with no reference; bit 4 by printed text) | S:202-204, S:399, §7f S:593-595 | **Real.** `restore` with no reference stays 0, so the perverse pair with R4 is gone, and bit 4 has a definition that does not turn on wording ("a check that ran and could not confirm"). §10's round-0 row still says the opposite (R2-M1). |

---

## 2. Important

### R2-I1. Test 3 runs before test 4 and catches every overridable "no" (new with the five-test fold)

The tests are "asked in this order" (S:206). Test 3 (S:211-212): "Did the tool
choose not to proceed, when the same verb with a different flag, network or
channel would? ... Then 8." Test 4 (S:213-214) is the NO test.

- **`mnemonic restore --expect-fingerprint` on a mismatch.** Re-run:
  `restore --from phrase=- --template bip84 --expect-fingerprint deadbeef` ->
  4, nothing on stdout; the same line with `--allow-mismatch` -> 0, 402 bytes
  out. Help: "Mismatch → exit 4 (unless `--allow-mismatch`)". The tool chose
  not to proceed and the same verb with a different flag proceeds, so test 3
  gives **8** and test 4 is never asked. §3's NO row lists "`--expect` not met"
  (S:184) and §7f sends "`restore` expectations" to **1** (S:590-591). The same
  holds for `--expect-xpub`, `--cosigner` and `--expect-wallet-id`.
- **R4 uses this exact flag as its example** (S:311-313) and says an override
  "clears the bit its refusal or check would have set". Which bit that is, 1 or
  8, is what the two readings disagree on.
- **Strict BSMS.** §7f sends the strict signature or MAC mismatch to 1
  (S:591-592). The same verb without `--bsms-verify-strict` proceeds (lenient,
  4). Test 3 gives 8.
- **The other direction.** Decision 2 sends `mr member` "with no keys for a
  seat the card does not key" to 2 (S:96-97). The same verb with `--keys`
  proceeds, so test 3 gives 8. Test 3 as worded catches any refusal that adding
  a flag fixes, including a flag that supplies missing data.

Swapping tests 3 and 4 does not fix it by itself: a network mismatch is also
"well-formed, and a comparison against something else came out no", and it
must stay 8.

Fix (one sentence, plus the owner's choice): narrow test 3 to a refusal whose
flag *waives a safety rule*, not one that supplies data or that overrides a
comparison's verdict, and say which class an overridable mismatch takes. The
spec's tables imply 1; the tests say 8. A script that separates "a check said
no" from "refused" depends on the answer.

### R2-I2. Test 1 against flag values: decision 2 contradicts itself, and §7 splits the same kind of value between 64 and 2 (R1-I2 partly open)

Decision 2 (S:91-93): 64 covers "what can be judged from the command line
without reading any input, file or environment variable: the parser's errors,
and the tool's own checks on flags and their values". Seven lines later
(S:98-100): "a malformed hot-family value is still not a usage error. It is an
input refusal, now 2", repeated in §7g (S:635-636).

The hot family is flag values, judged from the command line alone:
`--hot-child Z:24`, `--retire-child C/18/0`, `--hot-on 12345`, "which the
parser takes as text and `mr` refuses once it reads it"
(`SPEC_refugium_mr1.md`, the E45 paragraph, about `:27541-27547`). Test 1
gives 64. The same paragraph's closed list holds "a malformed or repeated
`--hot-parent-fp`", which stays usage, so `--hot-parent-fp zz` is 64 and
`--hot-child Z:24` is 2, and only the old list says why.

The same split across tools, each a value the tool checks without reading
anything:

| case | today (re-run) | spec says | test 1 says |
| --- | --- | --- | --- |
| `me seal --iterations 5` | 2 | 64 (S:239) | 64 |
| `mnemonic seed-xor split --shares 1`, `word-card --raid 9` | 1 | 64 (S:587-588, "parser-shaped") | 64 |
| `md compose --wrapper bogus`, `--path 5of3` | 1 | **2** (S:483) | 64 |
| `mr --hot-child Z:24` | 1 | **2** (S:98-100, S:635-636) | 64 |
| `md encode --fingerprint @0=zz`, `--key @0=notanxpub` | 1 | not placed | 64, or 2 as "a malformed value the tool reads" (S:185) |
| `ms split` bad K or N | 1 | not placed (§7c) | 64 |
| `mk --xpub <xprv>` | 64 | 2 (S:503-504) | 2 if a key on argv is "input", else 64 |

Round 1 put the user-visible question as "a bad flag value is 64 or 2". The
fold answers it in test 1 and then keeps two exceptions without calling them
exceptions. The missing piece is what counts as "input" when it arrives on
argv: a card or key is clearly input; a count, a wrapper name or a child
selector is clearly not.

Fix: define "input" in §3 next to "channel" (for example: an artifact, key,
phrase or file content, wherever it is given; a parameter such as a count, a
name, a path syntax or a selector is not input). Then either move `md compose`
values and the hot family to 64, which reverses "Keep 1" and so belongs in
decision 2 as a stated reversal, or keep them at 2 as named exceptions with
the reason. Place `--fingerprint`, `--key` and `ms` K/N in §7.

### R2-I3. Decision 7 says `md` gives 5 for a correction in an incomplete set; it gives 2 and prints nothing (R1-I4 partly open)

Decision 7 (S:148-149): "D26 puts 'verifiable by reassembly later' under
`REPAIR_APPLIED`, and `md`, `mk` and `mr` follow it (5 today)."

Re-run, with the md survey's three-chunk card:

- `md repair C1 C2BAD` -> **2**, stdout empty, "chunk set incomplete: got 2
  chunks, expected 3".
- `md repair C2BAD` (one damaged chunk) -> **2**, stdout empty.
- `md repair C1` (one clean chunk) -> **2**.
- `md repair C1 C2BAD C3` -> 5.
- `mk repair --in v2c1only.txt` -> 5; `mr repair <chunk 0, one error>` -> 5;
  `mr repair <chunk 0, clean>` -> 0 with a warning.

`md` is atomic here by its own D28 (`md-cli/src/cmd/repair.rs:98-112`; survey
`md.md:86` lists "incomplete set" under the atomic 2). So:

- The owner is told that "16 everywhere" changes only the toolkit (S:152-153).
  It also changes `md`: from 2 with no output to 16 with the corrected chunks,
  and a clean lone chunk from 2 to 0 (decision 6, S:142-143: "A set that is
  merely incomplete is not a failure").
- §7a's repair bullet (S:484-487) does not list either change.
- An implementer who keeps `md` atomic under R1 (S:292-294: "A verb that fails
  atomically reports only its failure bits") leaves `md` at 2 where `mk` and
  `mr` give 16, which is the split decision 7 exists to end.

Fix: correct the sentence (`mk` and `mr` follow D26; `md` refuses atomically, 2
today), say in decision 7 whether `md` moves to 16 and emits, and add the row
to §7a. If `md` keeps its atomic refusal, say so in decision 6 as the
exception.

---

## 3. Minor

- **R2-M1.** §10's round-0 rows were not updated by the round-1 fold and now
  contradict the body: C2 "the four ordered tests" (five, S:206-215); I7 "test
  1 (the parser or a documented closed list)" (now "the command line alone");
  I8 "`restore` with no reference sets it" (it stays 0, S:202-204, S:788).
- **R2-M2.** S:12 says `review-r1.md` is in
  `design/agent-reports/exit-codes-2026-11-26/`. That directory holds
  `review-r0.md` and the surveys only; the round-1 report is at
  `/home/user/wt/ec/review-r1.md`.
- **R2-M3.** Decision 9 against R12 and the script. An operator's stop is 130,
  and R12 (S:361-363) tells a GUI to treat anything outside 0..125 as "ended
  abnormally" and to key on no whole number but 0, 64 and 101. `mr-gui` starts
  `mr-sitting` (S:708-709), where a stop is an ordinary outcome. The shell
  example (S:254) prints "usage, internal error or signal". Add 130 to R12's
  list. Decision 9 also offers no alternative (keep a bit, for example 8, so a
  parent can tell a stop from a kill), unlike decisions 5 to 7.
- **R2-M4.** Test 1 (S:208-209) does not carry R2's exception. A secret on
  argv is a command line that is wrong on its face, so test 1 gives 64; the
  REFUSED row and R2 give 8. In `mr` the seed-words refusal is a post-parse
  `Failure::usage`, not the pre-parser guard R2 names (round 1's point, still
  true), and §7g sends it to 8 (S:638). Say in test 1 "except a secret on
  argv, which is 8 in every tool".
- **R2-M5.** Test 5 is "otherwise ... 2", so by the tests an internal failure
  (a serialise error, an invariant) is INVALID. R13 and the 101 row say 101.
  Add a line before test 1, or to test 5: a failure that is the tool's own
  fault is 101.
- **R2-M6.** Not placed, and decided by test 4 against today's code:
  `mr-sitting` "hand-back not this plan's" (2 today; a well-formed file whose
  comparison with the plan says no, so 1; §7h S:650 keeps "a malformed plan,
  hand-back or kit file" at 2 and is silent on this one). Still not placed from
  round 1: `mnemonic`'s `Bip388VerifyDistinctness` and
  `DescriptorReparseFailed` (4 today).
- **R2-M7.** The mainnet prompt has three endings and the spec names two.
  `mr-sitting` already separates `Stop::Refused` from `Stop::EndOfInput`
  (`sit.rs:359-362`); `root` and `child` fold both into 1 (`main.rs:306-309`).
  S:243 gives end of input 2 and S:655 gives the refusal 8. An answer that is
  not the override sentence is then 8, and whether a typed "stop" at that
  prompt is 8 or 130 is not said.
- **R2-M8.** Decision 6's parenthesis "(22 for a corrected non-card)" (S:140)
  holds for a single string with no oracle (16 + 4 + 2). A corrected chunk of
  a set that fails its per-string check is 18. One clause would stop a reader
  taking 22 as the only sum.

## 4. Nit

- **R2-N1.** Decision 4 (S:113-115): "`mt` takes it from crates.io (0.1.3)".
  Five do (`md`, `mk`, `ms`, `mt`, `mnemonic`, each `mnemonic-io-lib =
  "0.1.3"`); `me` uses a path dependency. So the step is a publish plus five
  pin moves, and `me` moves in-tree.
- **R2-N2.** Decision 5 (S:120-122) says all the listed verbs "print a note on
  stderr". The measured `mk` runs print the chunk-set-id warning first; the
  correction note was not re-checked for `address` and `derive` separately
  (`mk address --in v2c1.txt` -> 0 confirmed; `derive` needs `--path` and was
  not run to completion).
- **R2-N3.** §3's 128+N row (S:192) says 130 "also means the operator stopped
  it"; R10 (S:345-349) says the same in its last sentence. The R11 legend
  (S:354-359) says "128 and up = killed by a signal" only.

## 5. New factual claims spot-checked

Right:

- `SPEC_mt_v0_1.md:3384-3385` ("`0 = every check passed`") and `:810-827` in
  `/home/user/mnemonic-engrave`.
- 66 `Refusal::new` sites outside tests in `mt-cli/src` (`main.rs` 32, plus
  one in a comment; `validate.rs` 20; `input.rs` 7; `read_strings.rs` 7).
- `mt` `main.rs:374-376` (any `Err` -> 1) and `out.rs:91-99` (0 -> 1);
  `mr` `app/common.rs:42-57`.
- `--phrase-looks-like-digest-ok` (`ms`), `--allow-unsigned-inputs` (`me`) and
  `--max-conditions` (`mnemonic compare-cost`) exist.
- `mk decode`, `verify`, `inspect` and `address` on a corrected set -> 0.
- `mk repair` on one chunk of two -> 5; `mr repair` on a lone damaged chunk ->
  5, clean -> 0, `decode` -> 1.
- `restore --allow-mismatch` -> 0 with output; without it -> 4, no output.
- `me seal --iterations 5` -> 2 today; `md compose --wrapper bogus` -> 1.
- mnemonic-gui's deliberate `process::exit(130)` (`app_window.rs:193,207`).
- The network gate's check 5 (`scripts/spec-network-rule-gate.sh:35-40`).
- mnemonic-refugium's CLAUDE.md hazard line ("its exit code stays the
  checksum's").
- `mr-sitting` stop and end of input -> 1 (`session.rs:2166-2179`,
  `sit.rs:361`); mainnet not overridden -> 1 (`main.rs:306-309`).

Wrong:

- Decision 7: "`md` ... follow it (5 today)" (R2-I3).
- S:12: the location of `review-r1.md` (R2-M2).

---

## Summary

- Critical: 0. Important: 3 (R2-I1 test 3 catches overridable mismatches before test 4; R2-I2 flag values split between 64 and 2 against test 1; R2-I3 decision 7's `md` claim is false and `md` is left at 2).
- Minor: 8. Nit: 3.
- Round-1 Importants: R1-I3 and R1-I5 are real fixes; R1-I1 is fixed for its named rows; R1-I2 and R1-I4 are partial and carry R2-I2 and R2-I3.
- Verdict: **NOT GREEN** (0 Critical, 3 Important). The folds are text in §2 decisions 2 and 7, §3 tests 1 and 3, and three §7 rows; no survey needs redoing.
