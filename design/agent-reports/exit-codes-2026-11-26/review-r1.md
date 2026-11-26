# R1 re-review: SPEC_constellation_exit_codes.md (DRAFT, revision 1, 2026-11-26)

Reviewer: independent, read-only. Spec: `/home/user/mnemonic-engrave/design/SPEC_constellation_exit_codes.md`
("S:n" = spec line n). Scope: the round-0 folds, defects the folds introduced,
and decisions still made silently. Evidence: the six surveys and `review-r0.md`,
the cited specs, and about 40 fresh runs of the built binaries (stdin
`</dev/null`, scratch `/home/user/wt/ec/tmp-review1/`, surveyors' fixtures read
only). No repository was edited. Not re-litigated: the bit-flag direction, 64
for usage, the six bit names, C1's over-warning.

Verdict: **NOT GREEN: 0 Critical, 5 Important, 12 Minor, 3 Nit.** Most folds
are real. What is left is one family: the four ordered tests were added, but §7
was not re-derived from them, the "closed list" in test 1 exists for one tool
only, and three boundaries the tests do not cover (NO vs INVALID, the
incomplete set, UNVERIFIED vs warning) still give two answers.

---

## 1. Round-0 findings: is each fold a real fix?

| finding | fold | verdict |
| --- | --- | --- |
| C1 | S:205-214 states the over-warning | Real (accepted as stated; settled). |
| C2 | four tests S:146-153, worked cases S:163-174 | **Partial.** The tests decide the worked cases. §7 still contradicts them (R1-I1), and test 1 leans on a list that mostly does not exist (R1-I2). |
| I1 | S:216-217, S:411-412 | **Partial.** `mk` is 16 in the prose, 20 by §3's own row for bit 4, 18 by decision 6, and `mnemonic` stays 20 for the same input (R1-I4). |
| I2 | R1, S:221-225 | Real for `md`. Open for a repair that sets 2 but still prints (R1-I4). |
| I3 | decision 5, S:104-115 | Real. Cost stated. Verb list incomplete (R1-M6). |
| I4 | R4, S:236-241 | Real for the flag vocabulary. Creates a perverse pair with the I8 fold (R1-I5). |
| I5 | §6 S:324-338 | Real. Three small gaps (R1-M3, R1-M5). |
| I6 | R2, S:226-232 | Real for `--chain` and the pre-parser guard. `mr`'s argv refusals are not that guard (R1-I2). |
| I7 | test 1, S:158-161 | **Partial** (R1-I2). |
| I8 | S:135, R5 S:242-250 | **Partial.** R5 is now clear. Bit 4 is defined by what the tool prints (R1-I5). |
| I9 | R13, S:289-293 | Real. The stated conformance check is weak (R1-M7). |
| I10 | R9, S:265-269; §7d S:450-451 | Real; the two now agree (re-run: `mt inspect --bitcoin-cli ./cli-fail.sh` -> 0 today). One consequence to show the owner (R1-M10). |
| I11 | §6 S:339-363 | Real. |
| I12 | §7, R14 | **Partial**, Minor now (R1-M5). |
| I13 | S:577-581, S:617-630, R12 | Real. |
| I14 | R7, S:253-259 | Real: 2 in both modes, no behaviour change. |
| I15 | R1, S:224-225 | Real. Leaves the exact sum implementation-defined (R1-M8). |
| I16 | §7 counts | Real for `md`, `mk`, `ms`, `mt`, `mr` (checked against mk-ms.md:316-325, mt-me.md:186-190, mr.md:142). Not for `me` (R1-M4). |

Minor and Nit from round 0 that §10's last row does not list and that are
still in the text: M10, M11, M13, M14, N1, N4 (see R1-I3, R1-M9, R1-M6).

---

## 2. Important

### R1-I1. §7 still places refusals against the four tests (C2 fold incomplete)

Round 0 asked for "§5 re-derived from" the decision procedure. Three rows were
not, and each is the exact analogue of a worked case in S:165-174:

- **`mr` published-vector root, S:526 ("to 8").** Text
  (`mnemonic-refugium/crates/mr-codec/src/error.rs:5046`): "its words are
  guessable from the spec; make a new root from new draws". Raised at
  `root.rs:62,184`; no `--accept-*` flag guards it (grep). Test 3 fails, test 4
  gives **2**. Round 0 named this row (review-r0.md:71).
- **`mnemonic export-wallet` given a secret slot, S:494 ("to 8").** Text
  (`mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_export/mod.rs:67-68`):
  "watch-only by definition ... use 'mnemonic bundle'". No flag of the same verb
  accepts it. This is "xprv given as an xpub: 2" (S:168). Tests give **2**.
- **`md` "feature not compiled in", S:397-398 ("to 64").** Re-run: `md compile
  'pk(A)' --context tap` -> 2, "rebuild with --features cli-compiler". The
  parser accepted the line and `md` has no closed list, so test 1 fails; test 3
  ("a different ... environment would proceed") gives 8, and test 2 arguably
  32. The spec's 64 is a fourth answer.

Also S:431-432: `ms`'s digest-shaped phrase is 8 "since an override exists or
is proposed (CLI-08)". A class cannot depend on an unbuilt flag; S:171 sends
an unsigned transaction to `mt` to 2 precisely because "`mt` has no such flag".

Fix: re-derive §7's refusal rows from the tests, or state each exception with
its reason next to the tests.

### R1-I2. Test 1's "documented closed list" exists for `mr` only, and `mr`'s list contradicts the test and §7g (I7 fold incomplete)

S:148-149 and S:139 make 64 "the parser, or the tool's own documented closed
list, ... before any input was read". S:158-161: "Each tool keeps its own
closed list ... `mr` E45's list ... stay".

1. **E45's list holds entries §7g sends elsewhere.** The list
   (`mnemonic-refugium/design/SPEC_refugium_mr1.md:27510-27534`) ends: "a card
   argument refused ... by any verb taking an `mr1` card as seed words or an
   ms1, mk1 or md1 string". Re-run: `mr --network testnet decode md1yqpq...` ->
   2 (usage). By test 1 with the list kept, that is **64**. S:523-524 sends
   "another format's string" to **2**, and S:525 sends "a secret on argv (2
   today)" to **8**. In `mr` the argv seed-words refusal is a post-parse
   `Failure::usage` (mr.md:51, `common.rs:923,1018`), not the pre-parser guard
   that R2's exception names (S:227-229). So §3 says 64, §7g says 2 and 8.
2. **E45's list holds input-dependent entries.** "`member` or `address` given
   no keys or seeds on a card with a seat its Pubkeys TLV does not key" is
   decided after the card is decoded; `root --from coins` with zero flips is 2
   "usage" today on an empty stdin (mr.md:99, C8). Test 1 requires "before any
   input was read", R2 says an input-dependent limit "is not usage" (S:229-231),
   and decision 3 makes empty input 2. S:521 moves the whole list to 64.
3. **No other tool has a list.** §7 uses "parser-shaped" (S:376, S:482-483),
   which is not defined. Real cases that two implementers split on:
   - `me seal --iterations 5` (re-run: 2). `me`'s own rule says usage
     (`me-cli/src/main.rs:493-494`); §3's INVALID row says "a value the tool
     itself rejects" (S:134). 64 or 2.
   - `ms derive - --passphrase-stdin`, two readers on stdin: S:433 says 64, but
     the parser accepted it and `ms` documents no list. 64 or 2.
   - `md --fingerprint @0=zz`, `ms` bad K/N, a bad BIP-39 phrase (all 1 today;
     review-r0.md:109-112): 2 by the tests, "parser-shaped" by S:376.
   - `@env:` variable unset (`ms`, `mnemonic`: 1 today, toolkit.md:88): 2, or 32
     by test 2 ("something needed from the system ... is missing").

Fix: say that each tool's list is written down in its PR and what may be on it
(only checks made before any input is read), then either reopen E45's list for
the three kinds of entry above or exempt it by name. This is an owner decision
(§4 below, item 4).

### R1-I3. The tests do not separate NO (1) from INVALID (2), and an operator's stop has no class

The four tests choose among 64, 32, 8 and 2. Nothing chooses between 1 and 2,
and the two rows of §3 overlap ("`--expect` not met" against "an input that
breaks the verb's correctness rules").

- **`mt`, a node that proves an input spent.** S:445 keeps "a node's 'this
  check failed'" at 1; S:446-447 sends "the ten §6i correctness refusals" to 2,
  and "an input absent from the UTXO set" is one of the ten. Round 0's M13;
  not folded, not in §10.
- **`mnemonic`** `Bip388VerifyDistinctness`, `DescriptorReparseFailed` (4
  today, toolkit.md:62): not placed (round 0's M14).
- **`mr-sitting`** "hand-back not this plan's" (2 today, mr.md:71): a check
  that said no, or an invalid file. Not placed.
- **Stops.** S:541 keeps "stops" at 1, but no check ran; by S:133 that is not
  NO, and by R13 an unclassified failure is 101, "a bug". S:543 moves "end of
  input at a prompt" to 2. Re-run: `mr-sitting root --network main --letters B
  --from coins </dev/null` -> 1: the mainnet-on-a-network refusal (8 by S:542)
  is delivered as a typed-sentence prompt that then hits end of input (2 by
  S:543). 8, 2, or 1.

Fix: one sentence for the 1/2 boundary (for example: NO only when every input
was valid and the verb's question was answered "no"), a class for "the
operator stopped or declined", and a rule for a refusal delivered as a prompt.

### R1-I4. The incomplete set and decision 6 give three codes (I1 fold incomplete; new with decision 6)

Re-run, same one-chunk-of-two input (`$MK1BAD`): `mk repair` -> 5,
`mnemonic repair --mk1` -> 4; both print "correction UNVERIFIED".
`mk repair --in v2c1only.txt` -> 5 while `mk decode --in v2c1only.txt` -> 1
("received 1 chunks, header declares total_chunks = 2").

- S:216-217 and S:411-412: `mk` incomplete set is **16**.
- S:135, the bit-4 row itself: "the tool marked part of it as not verified: a
  correction with no oracle, a partial decode, **an incomplete set**". `mk`
  marks it UNVERIFIED. By the table it is **20**.
- Decision 6, S:121-126: "`repair` decode-checks its result, in every tool,
  and sets 2 when the result is not a valid artifact ... `mk repair` gets the
  same rule". The corrected incomplete set does not decode. **18.**
- S:497-498: `mnemonic` "unverified or ambiguous from 4 to 20". So the same
  input is 16 in `mk` and 20 in `mnemonic`, which is the split I1 was about.

The same hole in `mr`: a lone chunk (`mr repair <chunk 1>` -> 0 with a warning,
`decode` -> 1; mr.md:44, C5). Clean: 0 or 2 under decision 6. Corrected: 16
(D26's "verifiable by reassembly later"), 20 (S:531 "a single string"), or 22.

Also unstated: when decision 6 sets 2, does `repair` still print the corrected
strings? `mr` prints today (re-run: 5, corrected string on stdout). If it
prints, 22 is consistent with R1. If the failure is atomic, R1 (S:222-224)
makes it 2, as `md` already does for a dead card or an incomplete set
(md.md:86). "22 ... in every tool" then is not what `md` does.

Fix: say whether decision 6's check treats an incomplete set as "not a valid
artifact"; remove "an incomplete set" from the bit-4 row or keep it and make
`mk` 20; give `mnemonic repair --mk1` the same answer as `mk repair`; say
whether a repair that sets 2 emits output.

### R1-I5. R4 plus the I8 fold: an overridden mismatch is cleaner than no reference, and `restore` 0 -> 4 is an undeclared decision

- S:239-241 (R4): `restore --expect-fingerprint X --allow-mismatch` on a
  mismatch exits **0**, "judged as if that check had passed".
- S:135, S:489: `restore` with **no** reference moves from 0 to **4**.

So a script gets 0 by passing any wrong fingerprint with `--allow-mismatch`,
and 4 by passing none. The overridden run is at least as unverified.

Separately, 0 -> 4 on every reference-less `restore` (re-run in round 0: 0
today) is the same kind of cost as decision 5: every `set -e` script and `&&`
chain around a plain restore starts failing. Decision 5 was made a numbered
decision for that reason; this one appears only in §3's table row and one
bullet of §7f.

The rule behind it ("the tool marked part of it as not verified", S:135) turns
on the tool's wording. `mnemonic` prints "UNVERIFIED:" and gets 4. `mt` with
no node prints "could confirm NOTHING about it against the chain" (re-run) and
R9 keeps it 0. `mr`'s "verify: passed, but ..." and `md encode`'s "will
partial-decode" warning stay 0. Two implementers will not agree on which
existing warnings are "marks".

Fix: make it decision 7 (with the R4 interaction: does an override leave bit 4
set?), and list which existing banners set bit 4.

---

## 3. The four tests applied to real refusals

Each row is a refusal from the surveys, re-run where marked M. "Same class?"
asks whether two careful implementers, reading §3 and §7, land together.

| # | refusal (today) | tests give | §7 says | same class? |
| --- | --- | --- | --- | --- |
| 1 | `mt encode $(cat tx.hex)`, argv material (1) | R2 + test 3: 8 | 8 (S:448) | yes |
| 2 | `mt encode --in tx.hex > wr.txt`, mode 0644 (1) | test 3 (`--allow-world-readable`): 8 | 8 | yes |
| 3 | `me --hex --in ms.txt` (3, M) | no flag on the verb: 2 | "by the tests" (S:462) | yes, if "channel" is not read as "use `sysw pack`" (R1-M12) |
| 4 | `me seal --in ms.txt` (3, M; "Re-run with --seal-secret") | 8 | by the tests | yes |
| 5 | `me seal --iterations 5` (2, M) | 64 or 2 | silent | **no** (R1-I2) |
| 6 | `mr decode <md1 string>` (2, M; on E45's list) | test 1: 64 | 2 (S:523-524) | **no** (R1-I2) |
| 7 | `mr verify $C --seeds "<words>"` (2; on E45's list) | test 1: 64; R2's guard: 8 | 8 (S:525) | **no** (R1-I2) |
| 8 | `mr child`, published-vector root (1) | 2 | 8 (S:526) | **no** (R1-I1) |
| 9 | `mnemonic export-wallet --slot @0.phrase=-` (2) | 2 | 8 (S:494) | **no** (R1-I1) |
| 10 | `mnemonic convert` xpub to xprv (2) | 2 | 2 | yes |
| 11 | `mnemonic compare-cost --descriptor "wpkh(...)"` (3, M) / over the cap (3) | 2 / 8 | 2 / 8 | yes |
| 12 | `mr-sitting root --network main` on a networked computer, stdin at end (1, M) | 8 or 2 | 8 (S:542) and 2 (S:543) | **no** (R1-I3) |
| 13 | `mr-sitting kit-wipe`, no RAM folder (2) | test 2: 32 | 32 | yes |
| 14 | `mr-sitting practice --dir <files exist>` (2) | test 3: 8 (the IO row's "an output path" suggests 32) | silent | probably (R1-M12) |
| 15 | `md compile` without the feature (2, M) | 8 or 32 | 64 (S:397) | **no** (R1-I1) |
| 16 | `mt`, node proves an input spent (1) | not covered | 1 (S:445) and 2 (S:446) | **no** (R1-I3) |
| 17 | `ms derive - --passphrase-stdin`, two readers (1) | 2 | 64 (S:433) | **no** (R1-I2) |
| 18 | `mk address` on a multisig card (64) | 2 | 2 | yes |
| 19 | `me sysw pack --as descriptor --in md.txt` (3, M; "not a wallet descriptor") | 2 | by the tests | yes |
| 20 | `mr create` with no template (1; E52 "stays exit 1", not usage) | 2 | "Keep 1" meaning kept | yes |

Twenty refusals: 11 agree, 1 probably, 8 do not. All eight trace to R1-I1,
R1-I2 or R1-I3.

---

## 4. Decided silently; the owner should decide

1. **`restore` with no reference: 0 -> 4**, and whether an override leaves bit
   4 set (R1-I5).
2. **`inspect` on a string that would not decode: 0 -> 2** (R5, S:243-245).
   Round 0's decision 8. It reverses `ms`'s locked table; §8 (S:608) lists it
   as a defect "whatever is decided".
3. **Which reading of D26 wins for an incomplete set**: `mk`'s (16) or the
   toolkit's (20). §3 picks `mk`'s in one sentence; §7f keeps the toolkit's.
4. **What a closed list may contain, and whether E45's list is reopened**
   (R1-I2). "A bad flag value is 64 or 2" is the user-visible question.
5. **The code for "the operator stopped or declined"** (1 today; R1-I3).
6. **"Feature not compiled in" is 64** (S:397-398).
7. **A HARD network mismatch stays 8 although nothing clears it** (R1-M3). It
   follows from the 2026-11-23 ruling; it should be said, since §3's row tells
   the operator to "change how it is run".
8. **`mr`'s cost and timing.** Round 0's item 7. §7g and §9 now state the size
   ("two thirds", "the long pole"), but whether `mr` renumbers in this cutover,
   against a plan frozen at M15 and about 900 TRACE lines, is still assumed.
9. **`me bundle --preview DIR` with no sidecar exits 0** (R1-M10).

---

## 5. Minor

- **R1-M1.** S:110, S:112, S:360, S:453: `SPEC_mt_v0_1.md:3649-3660` and
  `:819-836` are the line numbers of the *copy* in
  `/home/user/wt/mnemonic-transaction/design/` (3659; 820-836). In the
  normative file, in this repo, "`0 = every check passed`" is at `:3384-3385`
  and the one-liner passage at `:810-827` (`/home/user/mnemonic-engrave`, HEAD
  `9010adb`; `:3470-3471` and `:813-830` in the `c667773` worktree).
- **R1-M2.** S:177, S:441: "`mt`'s 56 `Refusal::new` sites". Counted: 66
  outside test modules in `mt-cli/src` (`main.rs` 32, `validate.rs` 20,
  `input.rs` 7, `read_strings.rs` 7); no survey gives 56 (mt-me.md:42 gives 94
  rows in `tests/refusals.toml`). S:443-444: "the 19 §10.10 flag-combination
  refusals" move to 64. The 19 rows include two pasted-txid rows
  (`refusals.toml:263-274`, `txid_paste_guard`: the input's content, so 2) and
  four non-UTF-8 argv rows (already 64).
- **R1-M3.** S:155-156: network mismatch "passes test 3 anyway: `--network`
  changes the result". False for the HARD class: TX-H2 (key against key), and
  TX-H4C, whose text is "No --network repairs a file that contradicts itself"
  (`SPEC_network_rule.md:208,212`); the gate says "hard -> 8 and override never
  helps" (`scripts/spec-network-rule-gate.sh:31-32`). The ruling still makes
  them 8; the sentence, and the legend's "an override exists" (S:282), should
  not claim otherwise. §6 also omits that gate's check 5 (`:35-40`), which
  fails any repo that uses 8 (review-r0.md:306).
- **R1-M4.** S:456: `me` "about 110 pins" still includes 22 on 0 and 2 on 101
  (mt-me.md:194-196); on S:369-370's basis it is about 80. Round 0 named it.
  S:535 (`mr-sitting` "about 270") is "code assertions", basis not stated
  (mr.md:146).
- **R1-M5.** I12 is partly folded. Still unmentioned: toolkit
  `verify-examples.sh`, `root-shaped-test.sh`; `mr`'s
  `spec-gate-sitting-kit-day.sh`, `golden/diff-wrapper.py`,
  `create-key-kind-fix-gate.sh`; mnemonic-engrave `policy-differential.py`;
  `ci/toolkit-manual.rev`; `mr`'s usage-quiet trailer, which is keyed on code 2
  (mr.md C12); and the cross-repo tests that run a sibling binary
  (`jf8_pr64.rs`, `jf8_b_advice.rs`, `ms_remedy_runs.rs`). The last matter to
  §9's "the others can land in any order" (S:632-634).
- **R1-M6.** Decision 5 (S:104-106) and §7b (S:414-415) omit `mk inspect`
  (re-run: `mk inspect --in v2c1.txt` -> 0 after a correction) and `mt
  verify`/`inspect`. "Silently" is wrong for `mk` and `mt`, which print a
  stderr note (round 0's N1). The decision offers no alternative (a `--strict`
  flag; REPAIRED on `repair` only), which round 0 asked for.
- **R1-M7.** R13's conformance check (S:292-293) greps for
  `ExitCode::FAILURE` and `exit(1)`. It would pass the three mappings that
  matter: `mr`'s `From<mr_codec::Error> for Failure` -> 1
  (`app/common.rs:42-57`), `mt`'s "any `Err` -> 1" (`main.rs:374-376`), and
  the 0 -> 1 write filters (`mt` `out.rs:91-99`).
- **R1-M8.** R1's fail-fast clause (S:224-225) makes the exact sum depend on
  evaluation order (network refusal plus a corrected card: 8 or 24, S:328).
  Say that vectors and tests assert masks, not sums; the network rule's
  `vectors.tsv` has an exact `exit` column.
- **R1-M9.** Carried from round 0 and not in §10: R11's legend and R12 lack the
  1..63 range guard (M11; Windows NTSTATUS values in mnemonic-gui); R12's GUI
  codes omit mnemonic-gui's deliberate 130 and mr-gui's 1 (M10); `ms verify`
  prints "OK: valid future format" while exiting 2 (M14, S:424-425).
- **R1-M10.** R9 defines "asked for" as "the user named the helper"
  (S:267-269). So `me bundle --preview DIR`, where the user asked for the
  preview but did not name the sidecar, exits 0 with no preview (S:472). That
  is consistent, and worth showing the owner.
- **R1-M11.** Decision 4 and §9 step 1 (S:622-623) do not say that six
  binaries already depend on mnemonic-io-lib and that `mt` takes it from
  crates.io (0.1.3), so step 1 is a publish plus six pin and `vendor/` moves
  (review-r0.md:344-350).
- **R1-M12.** "Channel" in S:136 and S:151 is not defined (argv against stdin,
  or another verb such as `sysw pack`). `mr-sitting`'s "output path exists" is
  not placed, and the IO row's "an output path" (S:138) pulls against REFUSED's
  "an output that would overwrite".

## 6. Nit

- **R1-N1.** S:514: `45-mr.md:52-59`; the table is at `:53-60`.
- **R1-N2.** S:382-384 lists the wrong-network key among refusals that move to
  2, then excepts it to 8 in the next sentence.
- **R1-N3.** Decision 6 also changes the hazard line in mnemonic-refugium's
  `CLAUDE.md` ("its exit code stays the checksum's"); §6 names E45 only. R10
  still says a killed process "exits 128+N" (round 0's N4).

## 7. New factual claims checked and found right

`mr repair mr1qqpqqqqqqqqqqqqq5saqe93dj7z0t` -> 5 and the clean string -> 0
(`decode` -> 1); `mk repair --in N12.txt` -> 0 against `mk decode` -> 3;
closed stdout -> 0 in all eight binaries (`md`, `mk`, `ms`, `mt`, `me`,
`mnemonic`, `mr`, `mr-sitting`); `me --hex $'\xff\xfe'` -> 101; `ms inspect`
future tag -> 0 with FAIL; uniformity spec `:35` ("refuse at exit 2");
network rule `:400-401`, N9, N10; `mk` and `ms` pin counts; 147 `mr` test
files, 46 vector files; `create/mod.rs:4031`, `create/sitting.rs:1181,1321`;
pin `25051a06`; `me-cli/src/main.rs:495-500`; `Row` in
`descriptor/refusal.rs:38`; `mt` has no unsigned-input override; CLI-08 and
CLI-17 exist; §7j's old-code readings (5 = 1+4, 3 = 1+2).

---

## Summary

- Critical: 0. Important: 5 (R1-I1 §7 rows against the tests; R1-I2 the closed list; R1-I3 no NO/INVALID test and no class for a stop; R1-I4 incomplete set and decision 6; R1-I5 `restore` 0 -> 4 against R4). Minor: 12. Nit: 3.
- Round-0 folds: 12 of 18 are real fixes (C1, I2-I6, I9-I11, I13-I15); C2, I1, I7, I8 are partial and carry the five Importants; I12 and I16 are partial at Minor.
- The four tests on 20 real refusals: 11 agree, 1 probably, 8 do not; every miss comes from R1-I1, R1-I2 or R1-I3.
- Nine choices are still made silently (§4), led by `restore` 0 -> 4, `inspect` 0 -> 2, the D26 reading for an incomplete set, and what a closed list may hold.
- Verdict: **NOT GREEN** (0 Critical, 5 Important). The folds needed are text in §3, §2 and §7; no survey needs redoing.
