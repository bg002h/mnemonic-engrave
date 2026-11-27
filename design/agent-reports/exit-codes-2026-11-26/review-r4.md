# R4 final review: SPEC_constellation_exit_codes.md (DRAFT, revision 4, 2026-11-26)

Reviewer: independent, read-only. Spec:
`/home/user/mnemonic-engrave/design/SPEC_constellation_exit_codes.md` ("S:n" =
spec line n). Read fresh, whole document, as an implementer. Evidence: the six
surveys, `review-r0.md` to `review-r3.md`, `check-r3fold.md`, the sources under
`/home/user/wt/<repo>`, and about 90 fresh runs of the built binaries (stdin
`</dev/null`, scratch `/home/user/wt/ec/tmp-review4/`). No repository was
edited and no directory was created outside the scratch folder. Not
re-litigated: the bit-flag direction, the six bit names, 64 for usage, C1's
over-warning, decisions as recommendations.

Verdict: **NOT GREEN: 0 Critical, 1 Important, 9 Minor, 5 Nit.** The one
Important is a wrong "today" fact in decision 7 about `mr`, with a missing
§7g row behind it. The fold is two sentences. Nothing needs re-surveying.

---

## 1. Important

### R4-I1. Decision 7 says `mr` already gives 5 for a correction in an incomplete set. It does so only for one lone chunk; for two or more chunks of an incomplete set it refuses with 2 and prints nothing (S:172-188, S:720-722)

Decision 7 (S:175): "`mk` and `mr` follow D26 and give 5." Only "The toolkit
changes" and "`md` changes more" are listed as costs (S:183-186). §7g's repair
row (S:720-722) lists no incomplete-set change. Round 3 ran `md`, `mk` and the
toolkit for this decision, not `mr`. Measured now, on the 4-chunk corpus card
(`BAD0` = chunk 0 with two substitutions):

| command | today | stdout |
| --- | --- | --- |
| `mr repair BAD0` (one damaged chunk alone) | 5 | corrected chunk, with "chunks 1-3 are missing" warning |
| `mr repair C1` (one clean chunk alone) | 0 | echoed, with the same warning |
| `mr repair BAD0 C1` (2 of 4) | **2** | nothing: "the chunk set lacks chunk 2 of 4" |
| `mr repair BAD0 C1 C2` (3 of 4, one damaged) | **2** | nothing |
| `mr repair C0 C1 C2` (3 of 4, all clean) | **2** | nothing |
| `mr repair BAD0 C1 BAD2 C3` (complete) | 5 | corrected set |

So `mr` behaves as `md` does for every incomplete set except a single string.
(`mk` does follow D26 for partial sets: `mk-cli/src/cmd/repair.rs:50-52`,
"Candidate", exit 5 with an advisory.)

What this leaves undecided:

- **The owner's cost picture.** "16 everywhere" also changes `mr repair`: it
  must correct and print each chunk of a partial set, as S:184-186 says for
  `md`. That is a behaviour change under `mr`'s frozen plan, its E45 and its
  TRACE gates, in the repo decision 10 already calls the long pole. Decision
  7 tells him `mr` needs nothing.
- **Two implementers.** One reads "16 everywhere" and changes `mr repair
  BAD0 C1 C2` from 2 to 16 with output. The other reads §7g, finds no row,
  and keeps today's 2 (it is `REPAIR_FAILED`, "framing"). Same for the clean
  partial set: 0 by decision 7's `md` rule, 2 by §7g's silence.
- **The lone damaged chunk.** §7g says "A single string moves from 5 to 20"
  (S:720). `mr repair BAD0` is a single string that is a chunk of a set.
  Decision 7 gives it 16; §7g's sentence, read literally, gives it 20.

Fix: in decision 7, move `mr` out of "follow D26" ("`mr` gives 5 for one lone
chunk and refuses any other incomplete set, 2, with no output"), and add it to
the list of tools that change. In §7g's repair row say "a single-string card
moves from 5 to 20; a chunk or chunks of an incomplete set are corrected and
printed, 16 if any was corrected and 0 if clean (2 today for two or more
chunks)". Add the E45 line to §6.

---

## 2. Minor

- **R4-M1. Decision 5's list of silently correcting verbs is short (S:143-147,
  S:677-679).** It names `mnemonic inspect --mk1` as the toolkit's one case.
  Measured: `mnemonic convert --from "mk1=<damaged chunk> <chunk>" --to xpub`
  exits 0 with byte-identical stdout and stderr to the clean card, and so
  does `mnemonic word-card --from <damaged> --from <chunk>`. The help of
  `bundle`, `verify-bundle`, `export-wallet` and `restore` also takes mk1
  (not run). R6 ("any verb") gives the right answer, 16. But §7f sends only
  `inspect --mk1` there, and R13's table test is built from error variants
  and refusal rows, so nothing finds a silent correction that returns `Ok`.
  Say "at least" in decision 5, list `convert` and `word-card` in §7f, and
  ask each binary's conformance test for one corrected-input row per verb
  that reads a card.
- **R4-M2. Decision 8's comparison is not true (S:192-193); R5 inherits it
  (S:377-379).** "It exits 2 and still prints its whole report, as `md` and
  `mk inspect` already do." Measured: `md inspect` on an undecodable set
  exits 1 with empty stdout; `mk inspect` exits 1 (or 3 for version 1) with
  empty stdout. Neither prints a report. `ms inspect` itself prints the FAIL
  report only for the format-violation class; on a bad checksum it exits 1
  with nothing. What `md` and `mk` already do is exit non-zero. Reword the
  clause. R5's "it prints its whole report and sets 2" then needs "where it
  prints one", or §7a and §7b need a row saying `md`/`mk inspect` gain a
  report.
- **R4-M3. §7f's malformed `--from` row contradicts R2 unless the guard
  changes (S:659-660 against S:260-262, S:360-363).** `mnemonic convert
  --from bogus=1 --to xpub` exits 2 today because the pre-parser argv guard
  calls the value "secret key material". R2 and test 1 say the guard runs
  first and a secret on argv is 8. §7f says this case becomes 2. Both cannot
  hold while the guard fires on unknown node names, and it fires on them for
  a reason: `--from phrsae="<12 real words>"` (measured: refused, 2) is a
  secret on argv. State which: 8 whenever the guard fires, and 2 only when
  it does not (under `--allow-argv-secret`, where clap gives 64 today).
- **R4-M4. §3's own example of 20 is a case `md` refuses today (S:342-343).**
  "A chunked md1 with no origin: it reassembles (16) and decodes only partly
  (4)." The only `md` verb that corrects is `repair`. Measured on the
  origin-less card `md1yppqqxqj2s4dk6hk0wrt5n3`: `md decode` 4; `md repair`
  **2**, clean or with one substitution, nothing printed ("requires explicit
  origin"). So for `md repair` of an origin-less card the spec gives three
  answers: 4 or 20 (this example), 2 (decision 6's "adds 2 for any that
  fails", and today), 0 or 16 (it decodes as a string of its format). §7a
  keeps partial decode at 4 "on `decode` and `inspect`" only (S:556). Pick
  one and list it in §7a, or use another example.
- **R4-M5. Decision 7 does not say that `mk` calls this case unverified in
  its own text (S:175, S:181-188).** `mk repair <one damaged chunk of two>`
  exits 5 and prints "warning: correction UNVERIFIED — reassemble the full
  card ... a >4-error correction can alias to a different card". Under the
  recommendation the code is 16 with no bit 4 while the tool says
  UNVERIFIED, and R5 says a message must agree with its code (S:380). The
  owner should see that sentence beside "the alternative is 20, which is
  safer", and §7b should say the warning's wording changes if 16 is ruled.
- **R4-M6. N10 is opt-in, and §6's substitution breaks its trigger (S:81-83,
  S:477-479).** Decision 1 says the N10 line "is what tells [a network
  mismatch] apart from the other refusals". N10 prints only when
  `MNEMONIC_NETWORK_JSON=1` (`SPEC_network_rule.md:145-146`), so a script
  that does not set it cannot tell them apart; say so in decision 1. N10's
  trigger is "a CLI that exits 8 prints one extra line". Applying §6's
  "'Exits 8' becomes 'sets bit 8' everywhere it appears" makes every argv
  refusal print `{"kind":"network_mismatch",...}`. §6 needs one bullet: N10
  prints for a network mismatch, not for bit 8.
- **R4-M7. `ME_PREVIEW_BIN` naming a missing file: 0 or 32 (S:402-406,
  S:639-644).** Measured 2 today. R9: a helper the user named that fails is
  32. §7e has two rows, "found but fails" (32) and "absent ... stays 0, even
  with `--preview`"; a named path that does not exist is absent and named.
  One clause in §7e ("`ME_PREVIEW_BIN` set counts as naming it").
- **R4-M8. A wipe that fails during a stop or a signal has no code (S:195-199,
  S:407-411, S:733-737).** The survey records that a failed `mr-sitting` wipe
  "prints ST-HARD-02 and keeps the stop's code" (`wipe.rs:300-313`). Under
  decision 9 and R10 that run exits 130 or 128+N, neither of which combines,
  and R12 shows it as "stopped". Secret material left in the RAM folder is
  the one I/O failure of this tool an operator must not miss. Say whether a
  failed wipe keeps 130 (and why that is acceptable) or exits 32.
- **R4-M9. Cases the tests do not place, tried this round.** Each gave two
  defensible answers:
  - `mr current … --time 1` ("before this network's genesis block", 1 today)
    and `--height abc` (clap, 2 today). Decision 2's lists (S:98-104) name
    neither the tip nor reference values. Setting gives 64; "a value `mr`
    refuses after the parse" gives 2. The same for `mr verify --digest zz`
    and a malformed `--set-id` (clap, 2 today): 2 by analogy with
    `--expect-fingerprint zz` (S:112), 64 as a parser error on something the
    input list does not name.
  - `mnemonic derive-child … --application rsa` (2 today, "out-of-scope:
    deferred"): 64 as a bad setting, 32 as "a feature this build lacks", or
    2. Carried from R3-M5. It also shows that test 1's condition does not
    separate itself from the worked row "`md compile` in a build without the
    feature, 32" (S:300): both are known from the command line alone.
  - `Bip388VerifyDistinctness` inside `verify-bundle` (4 today): 1 or 2.
    Carried from R2-M6 and R3-M5.
  - R12 keys "stopped" on the number 130 (S:424-426). A GUI that starts the
    child sees no number when a signal kills it on Unix, so Ctrl-C is "ended
    abnormally" and only `exit(130)` is "stopped". Carried from R3-M3, second
    bullet; S:328-329 still says 130 covers Ctrl-C for a GUI.

---

## 3. Nit

- **R4-N1.** R2 (S:362-363): "a secret on argv plus a bad flag gives 8" holds
  where the guard is pre-parser (`ms`, `mt`, `me`, `mnemonic`; re-run). `mr`
  checks `--seeds <words>` after the parse, so `mr --bogus … --seeds
  "<words>"` is clap's error (2 today, 64 new). Add "where the tool has
  one".
- **R4-N2.** §3's REFUSED row (S:234) says "a flag, network or channel would
  waive"; test 5 (S:270-271) and round 3's fold add "or a different
  destination". Make the row match.
- **R4-N3.** Decision 6 (S:167-168): "a *complete* set that fails reassembly
  is 2". Say "exits 2 and prints nothing (R1)". `mk` suppresses that output
  on purpose (`repair.rs:53-55`, "a batch success must never carry a
  miscorrection"), and the sentence before it says `repair` "still prints
  the corrected strings".
- **R4-N4.** `SPEC_mt_v0_1.md:3384-3385` and `:810-827` (S:148-150, S:520,
  S:621-622) are right for mnemonic-engrave `master`. At the fix-PR head the
  surveys measured, the same text is at `:3470-3471` and `:813-830`. Quote
  the anchor text ("`0 = every check passed` is fixed here").
- **R4-N5.** The worked row "`mt` input the node proves spent, 1" (S:304)
  names no verb. R5 keeps `mt inspect`'s STATUS DEAD at 0 for the same fact
  (S:383-385). Write "`mt encode`/`verify`". The shell example (S:316) still
  calls 130 a signal (R3-M3, third bullet).

---

## 4. Decidability: the cases tried

"Today" is a fresh run unless marked (s) for the survey's value. Cases 1 to 16
were not tried by an earlier round.

| # | outcome | today | by the spec | one answer? |
| --- | --- | --- | --- | --- |
| 1 | `mr repair BAD0 C1 C2` (3 of 4) | 2 | 16 (decision 7) or 2 (§7g silent) | **no, R4-I1** |
| 2 | `mr repair BAD0` (lone damaged chunk) | 5 | 16 (decision 7) or 20 (S:720) | **no, R4-I1** |
| 3 | `md repair` of an origin-less card | 2 | 2, 4 or 0 | **no, R4-M4** |
| 4 | `mnemonic convert --from mk1=<damaged>` | 0, silent | 16 (R6) | yes; not listed, R4-M1 |
| 5 | `mnemonic word-card --from <damaged mk1>` | 0, silent | 16 (R6) | yes; not listed, R4-M1 |
| 6 | `mnemonic convert --from bogus=1` | 2 (guard) | 2 (§7f) or 8 (R2) | **no, R4-M3** |
| 7 | `ME_PREVIEW_BIN=/nonexistent me bundle --preview` | 2 | 32 (R9) or 0 (§7e) | **no, R4-M7** |
| 8 | `mr current … --time 1` | 1 | 64 or 2 | **no, R4-M9** |
| 9 | `mnemonic derive-child --application rsa` | 2 | 64, 32 or 2 | **no, R4-M9** |
| 10 | `mr-sitting` stop with a failed wipe | 1 (s) | 130, or 32 | not placed, R4-M8 |
| 11 | `mr export <card>` with no tip or range | 1 | test 6, 2: the need depends on the card | yes |
| 12 | `mr member … --epoch 999999` | 1 | R2, 2 | yes |
| 13 | `mr procedure nosuchpage`; `mr-sitting glossary nosuchterm`; `practice nosuchcase` | 2, 2, 2 | test 1, 64: not about the wallet | yes |
| 14 | `mr-sitting practice full-1 --dir /nonexistent` | 2 | test 2, 32 | yes |
| 15 | `me sysw pack --as descriptor --in md.txt` | 3 | test 6, 2 | yes |
| 16 | `me sysw pack --expect nonsense`; `mnemonic verify-message --signature zz` | 2; 1 | 64; 2 | yes |
| 17 | `mnemonic restore --expect-fingerprint deadbeef`, and with `--allow-mismatch` | 4, 0 | 1, 0 (decision 11, R4) | yes |
| 18 | `mnemonic restore --expect-fingerprint zz` | 4 | 2 (S:112) | yes |
| 19 | `mnemonic restore --template bogus` | 64 | 2 (S:111) | yes |
| 20 | `mr verify … --hot-parent-fp zz` | 2 | 2 (S:111, S:510-511) | yes |
| 21 | `mnemonic build-descriptor --archetype kofn-recovery` (no keys) | 2 | 64, a missing argument | yes |
| 22 | `mnemonic bundle --network mainnet --template bip84` (no slots) | 2 | 64 | yes |
| 23 | `md descriptor … --verify-against X --chain 7` | 1, verdict printed first | 2, before the verdict (R2) | yes |
| 24 | `mk repair` of a version-1 card (`N12`) | 0 | 2 with the string printed (decision 6) | yes |
| 25 | `ms inspect` of a reserved-tag string; `ms verify` of it | 0; 3 | 2; 2 (R5) | yes |
| 26 | `mt inspect --bitcoin-cli /nonexistent` | 0 | 32 (R9) | yes |

Eighteen of the 26 have one answer from the text alone (two of them, rows 4
and 5, are not listed where an implementer would look). Of the other eight,
rows 1 and 2 are R4-I1, and each of the remaining six is a single row in §7 or
one clause in §2 or §3.

## 5. Checked and sound

- **Revision 2 to 4 facts re-run and true:** every binary exits 0 with stdout
  closed (all eight run); `me --hex $'\xff\xfe'` panics, 101; `mr repair
  mr1qqpqqqqqqqqqqqqq5saqe93dj7z0t` 5 and the clean no-card string 0; `mk
  repair` 0 where `mk decode` gives 3; `mnemonic repair --mk1` 4 against `mk
  repair` 5 on the same string; `mnemonic repair --md1` on one chunk 2;
  `restore --expect-fingerprint` 4 with empty stdout, 0 with output under
  `--allow-mismatch`; `ms verify` "OK: valid future format" at 3; `mnemonic
  inspect --mk1` corrects with no note; `--hot-parent-fp zz` is rejected by
  clap; `md` prints the NOT spend-equal verdict and then fails on `--chain
  7`.
- **Pointers that hold at the surveyed heads:** `md` `main.rs:930-961`; `mk`
  `error.rs:135`; `ms` `error.rs:60`; `me` `main.rs:495-500`; toolkit
  `error.rs:641`, `cli.rs:173`, `out_sink.rs:89`; `mr` `app/common.rs:42-57`;
  `mt` `main.rs:297-378` and `out.rs:91-99`; mnemonic-gui
  `app_window.rs:1316-1356`; refugium-wallet `mr_panel.rs:240-246`,
  `create/mod.rs:4031`, `create/sitting.rs:1181,1321`, pin `25051a06`; the
  network rule's `:400-401`, N9, `nr0_exit_code_8_is_network_mismatch` and
  the gate's check 5; every script and test file §7 and §9 name exists.
- **Decision 4:** five binaries take mnemonic-io-lib 0.1.3 from crates.io,
  `me` by path, `mr` not at all. Matches S:135-140 and S:815-817.
- **Internal agreement:** tests 0 to 7 against the 19 worked rows; §5 against
  §7a to §7h; R4 against decision 11 and the network rule's "overridden SOFT
  exits 0"; decision 9 against R10, R11, R12, §5 and §7h; decision 2 against
  test 1, §6's E45 bullet, §7f and §7g (the round-3 fold and its check are
  both in the text); the arithmetic of S:773-774 and of 18 and 22.
- **Fairness of the eleven decisions:** each states its cost and, where one
  exists, its alternative. The exceptions are the facts in R4-I1 (decision
  7), R4-M5 (decision 7), R4-M2 (decision 8), R4-M1 (decision 5) and R4-M6
  (decision 1). None of them hides an argument against the recommendation
  except R4-I1 and R4-M5, which both bear on decision 7.

---

## Summary

- Critical: 0. Important: 1 (R4-I1: decision 7 says `mr` gives 5 for a correction in an incomplete set; measured, `mr repair` refuses every incomplete set of two or more chunks with 2 and no output, so "16 everywhere" changes `mr` too, §7g has no row for it, and a lone damaged chunk reads as 16 or 20).
- Minor: 9 (decision 5's list misses `mnemonic convert` and `word-card`; decision 8's "as `md` and `mk inspect` already do" is false; §7f against R2 on a malformed `--from`; §3's 20 example against `md repair`; `mk`'s UNVERIFIED text; N10; `ME_PREVIEW_BIN`; a failed wipe; unplaced values). Nit: 5.
- Decidability: 26 real outcomes tried, 16 new to this round; 18 have one answer, 8 do not (2 are R4-I1, the other 6 are one row or clause each).
- Verdict: **NOT GREEN** (0 Critical, 1 Important). The fold is two sentences in decision 7 and one row in §7g.
