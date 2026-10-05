# E3a plan R0 round 3, reviewer A (proportional re-review of draft 3)

Reviewed: `design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md` draft 3 at mnemonic-engrave `ebcc2b2`
(cited **P:line**). Round-2 reports: `e3a-plan-r1-a.md`, `e3a-plan-r1-b.md`. Draft 2 for diffing:
`git show 33d4c11:design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md`. R = `scripts/pico2-bootkey-rehearsal.sh`.
Scope: did draft 3 fix each round-2 finding, and did the fold add a defect. Settled facts (F14, `--ser`
249, `otp set -s` computes from copy 0 and writes all copies, F10, F18) were taken as given and not
re-derived. Read-only apart from this file.

**Counts: 0 Critical / 3 Important / 5 Minor / 2 Nit**

## Round-2 finding → fixed?

| finding | fixed? | evidence |
|---|---|---|
| A2-I1 / B2-I1 heal admits stray target-field bits | **Yes for the scenario reviewed; the fold opened a sibling gap (R3A-I1) and its mutation row has no real killer (R3A-I2)** | P:230-238 conditions 1-3 refuse the copy-0-only bit 8/9 case: copy 0 = 0x103, copies 1-2 = 0x003, E = 0xC03 → cond 1 (0x103 ⊄ 0xC03) and cond 2 (0x103 ≠ 0x003 outside T) both refuse. R9 walk still heals: 0x803/0x003/0x003 ⊆ 0xC03, all equal 0x003 outside T, 0x803\|0xC00 = 0xC03. |
| B2-I2.1 WARNING-trap mutation | **No** | case 3c (P:335) is not a killer; see R3A-I2(a). |
| B2-I2.2 `--ser` mutation | **Yes** | `FAKE_REQUIRE_SER=1` always set (P:322-323) makes every device read exit 99 under the mutation, so case 1's PASS goes red; case 13 (P:361) asserts the argv log. |
| B2-I2.3 red runs in the report | Yes | P:363-364. |
| A2-M1 80-column wrap | Yes | F18 (P:67), fake wraps (P:308), probe joins lines (P:386-387). |
| A2-M2 CRIT0 first copy | Yes | P:172 `-c 1` of 0x038; case 3 CRIT0 copy 0 (P:333-334). |
| A2-M3 stock values | Partly | P:403-406; the list lags the profile (R3A-M3). |
| A2-M4 slot-1 key flag | Partly | flag added P:131, P:143-147; the bench commands do not use it consistently (R3A-M2). |
| A2-M6 BOOT_FLAGS1 other bits | Yes | P:176. |
| A2-M7 alias probe detail | Yes | P:251-257; R3A-M4 on the CLI surface. |
| A2-M8 blinky at R0 | Yes | P:399-401. |
| A2-N1 G2 | Yes | F16 P:65, P:74. |
| B2-M1 F10 before R8 | Partly | cross-check in heal (P:237-238) and inject-copy (P:275-277); not applied to the equal-copy branches (R3A-I3); R8's fallback now unreachable (R3A-M1). |
| B2-M2 pre-state, branches | Partly | full-check pre-state and ordered branches (P:214-223); the derived-flag wording regressed (R3A-I1). |
| B2-M3 probe gating | Yes | P:252-257, case 8 (P:350-351). |
| B2-M4 compared-row set | Yes | P:115-117. |
| B2-M5 same entry | Yes | P:190-196. |
| B2-M6 deny-list spelling | Yes | P:150-152, case 7b. |
| B2-M7 case 12 | Yes | `BENCH_RUN` marker P:358-360, §8 step 6. |
| B2-M8 CI gating | Yes | steps in `test (rust + go)` (P:381-382; that job exists, release.yml:68); `contents: read`; builder print mode; ⊆ check (R3A-M5 on its definition). |
| B2-M9 `_TEST_ONLY` | Yes | P:198-200. |
| B2-M10 exit-3 guidance | Yes | P:121-123, P:294-296. |
| B2-N1 / B2-N2 | Yes | P:110-111; P:323-324. |

---

## Important

### R3A-I1 — the fold loosened the derived flags to "whatever equal value they hold", and E has no precise definition (§3.3 P:214-218, P:230-235; regression from draft 2)

**Evidence.** Draft 2 bounded the derived values: "KEY_INVALID either 0 or 0xC with equal copies"
(33d4c11 §3.3). Draft 3 says: "DISABLE_OTP_BOOT and KEY_INVALID take whatever equal value they hold"
(P:215). E is defined as "the row value the write produces" (P:218). Read literally, that is copy 0 | T,
which is the bound A2-I1 rejected. Cond 1's parenthetical ("a KEY_INVALID bit 8 or 9 is refused",
P:232-233) and cond 3 ("copy 0 | T equals E", P:235) only make sense if E is the *expected* post-write
value, independent of the board. So the implementer has to guess which one is meant.

**Consequence.**
1. A retail unit with BOOT_FLAGS1 = 0x000103 in all three copies (KEY_INVALID bit 8: SeedHammer's slot 0
   invalidated, for example by an earlier mistaken write) gets the derived flag `--key-invalid 0x1`.
   The pre-state check compares KEY_INVALID with that value, so it passes. `disable-otp-boot` then burns
   DISABLE_OTP_BOOT. Its post-check, "the full check with `--disable-otp-boot 1`" and the derived
   KEY_INVALID, returns **PASS, exit 0**, on a state the expected-state spec forbids. The same applies
   to bit 9: slot 1, Refugium's own key, is invalidated, yet the joint slot-1 table (P:163-167) still
   says `valid`, because it judges KEY_VALID only.
2. `invalidate-spare-keys` on that unit: under the literal E (0xD03), branch 2's first clause ("every
   copy has T clear and equals E & ~T") holds. 0xD03 is written to all three copies, the post-check with
   `--key-invalid c` fails, and the tool exits 3 on a board that should have been refused with exit 2
   before any write.

No new *effective* key bit is created, because the stray bit already wins the vote. That is why this is
Important and not Critical. It is still an irreversible write, and a false exit-0, on a board the tool
should refuse. No case covers it: case 2 is `check`-only, and cases 4-6 start from clean or
copy-0-only states.

A second gap in E: under both profiles, BOOT_FLAGS0 bit 11 is "either value" (P:177). The plan does not
say which value E carries. If an implementer fixes E's bit 11 to 0, then on a bench board where the boot
ROM has set ROLLBACK_REQUIRED, cond 1 and cond 2 refuse R6 and branch 2 refuses. That makes the R6 gate
unsatisfiable on silicon.

**Fix.**
- P:215: the derived flags must be legal. DISABLE_OTP_BOOT ∈ {0, 1} and KEY_INVALID ∈ {0, 0xC}, each
  with equal copies, or (for the target row only) the heal rule. Anything else is exit 2, no write.
- P:218: define **E** := the expected value of the target row after the write. Take it from the profile
  and the matched retail entry, plus the legal derived flag of the other write, plus T. "Either value"
  bits (BOOT_FLAGS0 bit 11) take the value the copies agree on, which cond 2 already forces to be
  common. E never takes a bit from copy 0 alone.
- Add cases: KEY_INVALID 0x1 (and separately 0x2) in all three copies → `disable-otp-boot` and
  `invalidate-spare-keys` each exit 2, state byte-identical. Add a heal and a no-op case with BOOT_FLAGS0
  bit 11 set in all copies → admitted.

### R3A-I2 — three §6.3 mutation rows still survive their named killers (§6.3 P:366-377, §6.1 P:319-320, §6.2 P:335, P:339-342)

**(a) "drop the WARNING trap (b) | case 3c" (P:370).** B2-I2.1 is not actually fixed.

Case 3c is copy 0 odd under `COPIES_IGNORED=1`. In that state:
- picotool still prints `RAW_VALUE=` with the odd copy 0 (F9);
- the `-c 1` read returns the vote.

Copy rule (d) (P:185-187) requires RAW_VALUE's copies to "equal the bare and `-c 1` reads copy for
copy". RAW_VALUE[0] ≠ the `-c 1` read, so (d) refuses. With (b) deleted, case 3c still exits 2, and the
mutation survives.

Given picotool's real output, (b) is redundant with (d): F9 prints WARNING and RAW_VALUE together.

**(b) "heal … bounded by copy0 | T instead of E | case 5 stray bit 8 / 9" (P:375).**

The named case puts the stray bit in copy 0 only (P:340-341). Under the mutation, E' = copy0 | T, and
cond 2 (P:234: every copy & ~T = E' & ~T = copy 0 & ~T) still refuses, because copies 1 and 2 lack
bit 8. The mutation survives.

What does distinguish the two bounds: a stray bit common to all copies, plus a partial T. Take copies
0x903 / 0x103 / 0x103 for `invalidate-spare-keys`:
- the true E refuses (0x903 ⊄ 0xC03);
- the mutant admits, then writes 0xD03 to all copies and exits 3.

The "judged by field" half of the row is an equivalent mutant while conds 1 and 3 use the true E. Field
bits outside T are 0 in E, so cond 1 already bounds them. Drop it from the table or say so.

**(c) "drop the RAW_VALUE cross-check | case 5 heal under `COPIES_IGNORED=1`" (P:376, P:342).**

The case does not say which copy is odd. Two variants pass the case without killing the mutant:
- **A non-0 copy odd.** `-c 1` = vote = the true copy 0, the heal is legitimately admitted, and the
  case's expected "refused" is wrong.
- **Copy 0 lacking T.** Under the mutant the tool sees three copies equal to E (the vote), takes branch
  1 (no write), and the post-check's copy rule (d) fails. That gives exit 2 with the state identical,
  which is exactly the case's assertion, so the mutant survives.

Only copy 0 *holding* a bit the others lack kills it. For example, copy 0 = 0x103 and the others 0x003:
the mutant reads three clean copies and writes.

**Also, `SUPPRESS_WARNING` (P:319).** It must state whether the `RAW_VALUE=` line is suppressed too. If
RAW_VALUE is still printed, the kill of "drop the `-c 1` read from (a)" (P:369) depends on whether
(d)'s implementation compares RAW_VALUE[0] with anything.

**Fix.**
- Redefine `SUPPRESS_WARNING=1` as "the named read prints only the vote, with no `RAW_VALUE=` and no
  WARNING", which is the realistic picotool regression.
- Add a fault mode `SUPPRESS_RAW_VALUE=1` (WARNING printed, no RAW_VALUE). Make case 3c copy 0 odd under
  `COPIES_IGNORED=1` + `SUPPRESS_RAW_VALUE=1`, so only (b) refuses.
- Make (b)'s killer case 3c as redefined.
- Add case 5d: copies 0x903/0x103/0x103 → `invalidate-spare-keys` exit 2, state identical. Make it the
  killer of the copy0|T bound, and delete or annotate the field half.
- Pin case 5's `COPIES_IGNORED` variant to "copy 0 holds a bit the others lack (0x103/0x003/0x003)".

### R3A-I3 — the RAW_VALUE cross-check (cond 4) and the WARNING are applied only inside the heal rule, not to branches 1 and 2's equal-copy test (§3.3 P:214-223, P:237-238)

**Evidence.** The target row is "judged by the heal rule instead of the copy rule" (P:216). The plan
does not say how the copies are established as equal in branch 1 ("every copy already equals E") or in
branch 2's first clause ("every copy has T clear and equals E & ~T"). Cond 4 and the WARNING trap are
stated only for "a row whose copies are unequal" (P:231). Literally, then, the branch-1 and branch-2
tests run on the per-copy reads (bare reads of the unnamed rows plus `-c 1`).

**Consequence.** Take F10 false on silicon (the case `COPIES_IGNORED` models, unmeasured until R8),
with copy 0 = 0x103 and copies 1-2 = 0x003:
- the `-c 1` read returns the vote (0x003);
- all three copies "equal" 0x003, so branch 2's first clause admits the row;
- `otp set -s` computes from the real copy 0 (F19) and writes 0xD03 to every copy.

Slot 0 is invalidated in all copies. This is exactly the A2-I1 harm, reached through the branch the
heal rule does not guard. The named read printed both the WARNING and RAW_VALUE, and nothing in the
branch looked at them. This is Important rather than Critical only because R8 measures F10 before E3b.

**Fix.** P:218-222: the target row's per-copy values are established in **every** branch by copy rule
(a) together with (d), with cond 4 being (d) restated. A WARNING or a `RAW_VALUE=` line means "unequal"
and routes the row to the heal rule. Branches 1 and 2's first clause require that the named read printed
neither. Add a case: 0x103/0x003/0x003 under `COPIES_IGNORED=1` → `invalidate-spare-keys` exit 2, state
identical. This is the same state as R3A-I2(c)'s pinned case, so one case serves both.

---

## Minor

- **R3A-M1 (§7 R7/R8, P:414-417; §3.6 P:275-277).** If F10 is false on silicon, R7's `inject-copy`
  post-verification already fails: its `-c 1` read returns the vote 0x003, and RAW_VALUE[0] is 0x803.
  It exits 3 with "stop the rehearsal; do not re-run", so the bench never reaches R8. R8's fallback ("If
  it shows the vote … the check still refused in R7 through the WARNING; the plan is re-opened") is
  therefore unreachable, and R7's "`check` must refuse" never runs.
  **Fix:** in R7, say that an inject-copy exit 3 naming the RAW_VALUE[0] disagreement *is* the F10-false
  finding. In that event run R8's raw `otp get` only, to record the reading, then stop and re-open the
  plan. (R5 and R6 do not reveal it: copy 0 is not the odd copy there.)
- **R3A-M2 (§7 R3, R5, R7, R10; P:408, P:414, P:420).** The bench commands are not executable as
  written:
  - R3 and R10 omit `--ser` and `--rehearsal-key`. Both are required on every device command (P:141,
    P:143-145).
  - R10 also omits `--rehearsal-slot1-key`.
  - R3's path `rehearsal-work/<CHIPID>/my-key.pem` does not exist. R keeps both keys flat in
    `$WORKDIR` = `rehearsal-work/` (R:67, R:1066 `"$WORKDIR/my-key.pem"`, R:937 `"$WORKDIR/factory-key.pem"`).

  The tool would refuse, so the failure is visible, not vacuous, but the bench walk has never been
  executed. **Fix:** spell R3 in full once (`--ser S --rehearsal-key rehearsal-work/factory-key.pem
  --rehearsal-slot1-key rehearsal-work/my-key.pem …`) and have R5, R7 and R10 say "R3's identity flags".
- **R3A-M3 (§7 R1, P:403-406, vs §3.2 rehearsal column P:171-181).** R1's stock-value list omits cells
  the rehearsal profile fixes, each of which can fail R3 after R2's irreversible phases:
  - FLASH_DEVINFO_ENABLE = 0 (A2-M3 named it);
  - BOOT_FLAGS1 bits 4-7 and 12-15 (P:176 now requires every bit outside KV/KI to be 0);
  - KEY_VALID and KEY_INVALID = 0 pre-R2;
  - page locks LOCK1 0x040404 and LOCK0 0;
  - slots 1-3 zero;
  - the BOOT_FLAGS0/1, CRIT0 and CRIT1 copies equal.

  **Fix:** R1 compares the capture against every fixed rehearsal cell of §3.2, plus the copies-equal
  rule, and lists them. Ideally the tool does this itself (`check` cannot run before slot 0 holds a key,
  so a capture-side comparison in the implementer's code, or an explicit list).
- **R3A-M4 (§3.1 P:136 vs §3.4 P:256, §7 R4 P:409).** `erase-range --probe-only` is not in the §3.1
  synopsis. Its R4 positive control (aliasing on 4 MB) is, by §3.4, a *refusal* ("refuse, condemned",
  exit 2), but R4 does not state the expected exit code or message. An operator could read exit 2 as a
  failed step. **Fix:** add `--probe-only` to the synopsis. In R4: "expect exit 2 with the alias message
  naming +4 MiB; any other result stops the bench".
- **R3A-M5 (§6.4 P:388-389).** "every argv shape in the fake's argv log is in the probed set" needs a
  defined shape function, or the ⊆ check is either always-false or vacuous. For example: subcommand,
  options and their order kept; row numbers, register names and values kept; `--ser` value, file paths,
  addresses and hex values replaced by placeholders. The e2e must obtain the probed set from the same
  builder print mode the probe job uses, not from a list in the probe job. **Fix:** state the
  normalisation and the source of the set.

## Nit

- **R3A-N1 (P:9-10).** "Fold tables: §10" — round 2's table is §11.
- **R3A-N2 (§3.1 P:144-145).** `--rehearsal-slot1-key` is required on "every command that judges slot 1
  (all but `capture`, `save-range` and `inject-copy`)". That includes `erase-range`, which judges only
  slot 0 (§3.4). Either list `erase-range` among the exceptions, or say that the write steps and `check`
  are the commands that judge slot 1. Case 7b's "also with `--rehearsal-slot1-key` given" then reads
  naturally.

## Checked, no finding

- §3.3 branch order: branch 1's no-write exit 2 cannot produce exit 3, and a re-run after exit 3 is
  bounded by "non-zero for any reason, stop" (P:122-123, P:295-296). No exit-code loop remains, given
  R3A-M1 for inject-copy.
- Interrupted writes (case 6): after `FAIL_SET_AFTER=1` on either write, copy 0 holds T. The re-run
  satisfies conds 1-4 with E = the expected value, and picotool writes copy 0's value to all copies.
- The alias probe on 16/8/4 MB: a random 4 KiB marker read at +4/+8/+12 MiB detects 8 MB (+8 aliases)
  and 4 MB (+4 aliases). It runs only after `--execute` and confirmation. Rehearsal erase skips it. The
  dry run records no `load` (case 8).
- §6.4: `test (rust + go)` exists on `ubuntu-latest` (release.yml:68-69), so adding steps to it gates
  merges. The probe job's `contents: read` and builder print mode answer B2-M8.
- inject-copy `bf1-copy0` uses `otp set -c 1 -s 0x04b <copy0|0x800>` with copies pre-required equal, so
  its `-c 1` value read is the true copy 0 even if F10 were false (vote = copy 0 when copies are equal).

VERDICT: 0C / 3I / 5M / 2N
