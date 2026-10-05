# E3a plan R0, lens B: failure states, operator error, false PASS, and whether the gates run

Reviewer: independent adversarial R0 (lens B), 2026-10-05. Target: `design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md`
draft 1 (commit 6b0d1ba), cited **P:line**. R = `scripts/pico2-bootkey-rehearsal.sh`; FK = `scripts/test/fake-picotool`;
E2E = `scripts/test/run-e2e.sh`; OF / PT / RM = the e3a recon reports. Read-only apart from this file. shellcheck 0.x
was run locally on R, E2E and FK; nothing else was executed.

**Counts: 1 Critical / 7 Important / 10 Minor / 2 Nit**

---

## B-C1 (Critical): the pre-check rule for the irreversible writes contradicts itself, so R7 and R9 cannot run as written and the implementer has to invent the precondition

**Sections:** §3.1 (P:121-124), §3.2 (P:142-143, 148-153), §3.3 (P:168-175), §7 R5/R7/R8/R9 (P:272-282).

**Scenario 1, heal vs equality.** §3.1 says a full `check --stage pre` must pass immediately before every write.
§3.2's copy rule (a) requires all raw copies to be equal. §3.3 adds "If a copy holds a bit the new value lacks … the
pre-check has already refused it (copies unequal), so the partial-burn case never starts". At the bench, R5 leaves
BOOT_FLAGS0 as 0x048=0, 0x049=0, 0x04a=0x002000, and R5 itself requires `check --stage pre` to refuse that state.
R7 then runs `disable-otp-boot --execute` and expects that "the write succeeds and heals the copy". By §3.1 and
§3.3 the in-run pre-check refuses with exit 2 before any write. R8 and R9 repeat the same contradiction on
BOOT_FLAGS1 0x04b.

**Scenario 2, a binary stage over a three-state sequence.** `--stage` is pre|post, and the table defines
`DISABLE_OTP_BOOT 0 (pre) or 1 (post)` (P:143). The planned order is disable then invalidate (P:176-177, R7 then R9).
`invalidate-spare-keys` runs "a full `check --stage pre`", which requires DISABLE_OTP_BOOT = 0. After R7 it is 1, so
R9 is refused. In the other order the failure is worse. If invalidate runs first, its "full check of the expected
post-state" (P:123) includes DISABLE_OTP_BOOT = 1 (stage post), which is false at that point. The post-check fails
and the tool prints "do not use this board for seeds" and exits 3 on a correctly written board. Also,
`disable-otp-boot` has no `--expect-key-invalid` argument (P:109), so after an earlier invalidate its pre-check
(KEY_INVALID default 0) refuses forever. The tool explicitly does not enforce the order (P:177-178).

**Why Critical:** the precondition for an irreversible OTP write is the safety core of this plan, and as written it
is undefined. Whichever way the implementer resolves it at code time, nobody reviews that choice before code exists.
"Relax equality when the copies look healable" is exactly the change that creates a false-pre-PASS. The Done-when
(P:311, "the §7 rehearsal passes") cannot be met as written.

**Fix:**
1. Replace `--stage pre|post` with an explicit expected-state vector per step. Each write command computes its own
   expected pre-state and post-state from (profile, current KEY_INVALID expectation, current DISABLE expectation).
   Give `disable-otp-boot` an `--expect-key-invalid`, or have each write derive "the other step's bit" as
   *either 0 or the final value, with equal copies*.
2. Define a **heal pre-state** in the plan as a rule and test it. For the target row, every copy satisfies
   `copy & ~target_final == 0` (no copy holds a bit outside the final value), and the copies differ **only** within
   the target field's bits. All other bits are equal across copies and equal the expected values. Copy 0 (as read
   with `-c 1`) must not hold any bit that the others lack outside the target (F11: `otp set` computes from copy 0).
   Only then is the write allowed. Anything else is refused.
3. Add e2e cases: heal from each of the three single-copy partials, for both rows, and refusal when a copy holds a
   bit outside the target.
4. Rewrite R7 and R9 to say which rule admits them. R5 and R8 keep their "must refuse" against the *non-write*
   `check`.

---

## B-I1 (Important): an interrupted or failed write has no defined outcome, and the rerun path is refused forever

**Sections:** §3.1 (P:121-126), §3.3 (P:168-175); OF P3; prior art `bootkey-round5-lens-failure-injection.md` C1
("re-run the identical `otp set -s`, which is OR-only and safe to repeat").

**Scenario A: unplug or power loss during `otp set -s BOOT_FLAGS0.DISABLE_OTP_BOOT 0x1 --ser X`.** The boot ROM burns
rows one at a time (OF P3), so the board can end with copy 0 only (vote 0, harmless), or copies 0+1 (vote 1,
effective), at the same time as picotool exits non-zero. The plan does not say what the script does when the write
command itself fails. Under `set -euo pipefail` (R:62) the script dies with picotool's status, or with `die` → exit 1
(R:83). The operator sees no post-check, no exit 3 and no "do not use" text. On the rerun, the pre-check finds
unequal copies and refuses with exit 2 "state refused". On SeedHammer #1 there is no tool path left. The operator
either condemns a device that only needed the documented OR-only rerun, or improvises raw `picotool otp set` without
the tool's checks. That improvisation is the failure these tools exist to remove.

**Scenario B: write completes, then unplug before or during the post-check.** The post-check's reads `die` inside
the shared helpers (`otp_field`/`read_rows`/`read_row_raw24`, R:176, 210, 367), giving exit 1 ("usage or tool
error", P:125) instead of exit 3. The "do not use this board for seeds" text is not printed. The rerun works
(already-set skip, P:168-169), but the operator was never told to rerun rather than retry blindly.

**Scenario C:** the same between the two bench injections and their heal (R5→R7, R8→R9). These are rehearsal-only,
but R7/R9 are the recovery path, so B-C1's fix covers them.

**Fix:**
- State that a non-zero exit from the write command, and any read failure after a write was issued, both go to the
  exit-3 path with a fixed message. The message says: "the write may be partial; re-run this same command (it is
  OR-only and heals a missing copy); if the re-run refuses, do not use this board for seeds".
- Make the heal pre-state of B-C1 the rerun's admission rule.
- Add e2e cases: the fake fails `otp set` after N of 3 rows for N = 1 and 2 → exit 3 with the message, then a rerun
  heals and passes; the post-check read fails → exit 3.
- The runbook E3b section lists these outcomes for the operator.

---

## B-I2 (Important): the profile is trusted from the command line for the commands with no OTP pre-check, which lets a SeedHammer be under-erased or injected

**Sections:** §3.1 (P:111-113, 116-127), §3.4 (P:182-188), §0 item 4.

**Scenario A: `erase-range --profile rehearsal --ser <SH2 #1 CHIPID> --execute`** (operator typo, or a pasted
bench command). §3.1 applies the full pre-check and post-check only to "write commands", and its post-check is an OTP
expected state. Nothing says erase-range runs the profile identity check (slot 0 = rehearsal factory key ≠
`SH_SIGNKEY_HASH`). If erase-range only takes its *range* from the profile, it erases 0x10000000-0x10400000. The
following `save-range` reads the same 4 MB and finds every byte 0xFF, so the step reports PASS. The upper 12 MB,
including the payload region at 0x10E00000, is never touched. That is a false PASS of a provisioning step on a seed
device, against UI spec §4.4 ("Every erase, read and verify of the flash uses the explicit 16 MB range").

**Scenario B: `inject-copy --profile rehearsal --ser <SH2 CHIPID> --row 0x04b --bits 0x000800 --execute`.** The only
stated guard is "Under `retail`, `inject-copy` is refused" (P:127), and the profile is whatever the operator typed.
inject-copy cannot obey the general write rule: its post-state is by design one that the check refuses, so "a full
check of the expected post-state must pass immediately after" (P:123-124) would always exit 3. An implementer will
therefore exempt it from pre/post checks. That exemption leaves the typed flag as the only thing between a
deliberate unequal-copy burn and a seed engraver.

**Fix:**
- Every device command (check, capture excepted only for the profile), erase-range, save-range and inject-copy,
  runs the profile identity gate before any device write. Under `rehearsal`: slot 0 = hash of the rehearsal factory
  key, slot 0 ≠ `SH_SIGNKEY_HASH`, and the CHIPID is not one of the SH2 CHIPIDs in HARDWARE_INVENTORY (a constant
  list in `otp-read.sh`). Under `retail`: slot 0 = `SH_SIGNKEY_HASH`.
- Give inject-copy its own explicit pre-rule and post-rule: the target row changed by exactly `--bits` and the
  other copies are unchanged; see B-I3.
- Add e2e cases: `erase-range --profile rehearsal` on a retail-shaped state → refused before any `erase` argv is
  recorded; `inject-copy --profile rehearsal` on a retail-shaped state → refused with no write.

---

## B-I3 (Important): an inject-copy into the named first copy can write all three copies, which makes R8 a vacuous "refused" while burning a real KEY_INVALID bit

**Sections:** §1 G2 (P:69), §3.1 (P:113), §6 fake (P:223-224), §7 R8 (P:278-280).

**Scenario:** F-619 (measured on silicon, quoted at R:373-376) shows that the numeric selector `0x04b` resolves to
the **named** register BOOT_FLAGS1 (RBIT-3); 0x048 and 0x040 will behave the same way. PT D11 says `otp set` on a
redundant register writes **all** copies, computed from copy 0. So `otp set 0x04b 0x000800`, or `otp set -s`
without `-c 1`, very plausibly writes 0x800 into 0x04b, 0x04c and 0x04d. R8 then runs `check
--expect-key-invalid 0`, which refuses because KEY_INVALID votes 0x8 ≠ 0. The step is recorded as "R8 refused", so
the F-619 first-copy blind spot and F10 are recorded as settled when no unequal copy ever existed.

A second variant applies to unnamed rows such as 0x04a. If the write is not explicitly raw, picotool may apply ECC
and burn ECC parity into bits 16-21. On BOOT_FLAGS0 those are DISABLE_WATCHDOG_SCRATCH and the DISABLE_BOOTSEL_*
interface bits (OF 2d). R7's heal then fails on that row (F11), leaving a permanent stray copy.

G2 asks how a one-row write is expressed and assumes "that it writes only that row". Nothing *verifies* this on the
board after the injection. The plan also says only that the fake implements "raw single-row writes per G2", so the
fake will model whatever G2 assumes. Finally, `--bits` accepts any hex value (P:113).

**Fix:**
- inject-copy accepts only the two bench injections (0x04a/0x002000 and 0x04b/0x000800), plus whatever the e2e
  needs behind a test-only fake.
- The write uses an explicitly raw, single-copy form (`-r` and `-c 1`, per G2).
- Immediately afterwards, read 0x049/0x04a or 0x04c/0x04d bare, plus the target with `-c 1`, and require: the target
  copy equals old|bits, and every other copy is unchanged. Otherwise exit 3 and stop the rehearsal.
- The fake must model (i) numeric 0x040/0x048/0x04b resolving to the named, voted register with WARNING, (ii) `otp
  set` on a named numeric selector writing every copy unless `-c 1`, and (iii) a non-raw write of an unnamed row
  applying ECC. A test must show that inject-copy without `-c 1`/`-r` is caught by the post-injection verification.

---

## B-I4 (Important): the e2e gates cannot run as written (old-fake version line, case 9 toolchain, fixture UF2)

**Sections:** §6 case 9 (P:247-248), CI (P:252-254), §9 (P:308-309), §4 (P:203).

1. **"R's existing e2e passes under both fakes … the old fake's run stays as is."** §4 adds `require_pinned_picotool`
   to R ("exactly 2.3.1", "no override flag", P:82-83). FK:22 prints `picotool v2.2.0-a4 (stub)`. Every R invocation
   in E2E sections A-E therefore dies before doing anything, and the Done-when line P:309 is unsatisfiable unless FK
   changes, which contradicts "stays as is". The old run also needs a *real* 2.3.1 picotool for `seal` and `info
   <file>` (FK:13, 23, 26) plus TinyGo (E2E:120-121). §8 names no shell that has both: the fork's shell has 2.2.0-a4,
   and `.#otp` has no tinygo.
2. **Case 9 ("R's sections A-E under the new fake … fixture UF2 so TinyGo is not needed") runs in CI** (run-e2e-otp.sh
   is the CI job). R itself requires `tinygo` and `go` on PATH for phases 0, 3, 4, 5 and 6 (R:132-136). It requires
   `SEEDHAMMER_DIR` to exist (R:145-146). Phases 3/5 call `sign-firmware.sh`, which runs the fork's `picosign` via Go
   and uses `xxd`. The CI job provides "bash + python3 + jq + openssl" only (P:252-253).
3. **A fixture UF2 cannot serve `seal`'s OTP JSON.** R generates fresh keys at phase 0 (R:913-921 per RM) and
   `make_otp_json` cross-checks the JSON against the openssl hash of *that* key (R:459-477). A static fixture cannot
   match. The fake would have to compute the key hash itself, which is the same method as the code under test, so
   the cross-check becomes tautological in the e2e. The plan should say so.

**Fix:**
- Update FK's version line to 2.3.1 and its ECC VALUE width (PT condition 3), and say so. Alternatively, give R's
  version gate an env override that only the test harness sets, and state that.
- Define case 9's stubs explicitly: stub `tinygo`, `go` and `sign-firmware.sh` on PATH; set
  `SEEDHAMMER_DIR=$TMP/fake-fork`; have the fake's `seal` compute bootkey0 from the key with an explicit "tautological
  in e2e" note. Or drop case 9 from CI and run it at §8 step 1.
- Name the shell in which "R's existing e2e under the old fake" runs, and run it once before dispatch (§8 step 1),
  per CLAUDE.md's never-run-gate rule.

---

## B-I5 (Important): the mutation gate is unsatisfiable, because the three copy rules overlap in a fake that always warns

**Sections:** §3.2 copy rule (P:148-151), §6 mutations (P:249-251), §9 (P:308).

**Scenario:** the fake implements P1 faithfully: whenever copies differ, the named read prints RAW_VALUE and
WARNING. Then:

| Mutation | Why no §6 case turns red |
|---|---|
| Remove rule (a), the raw copy comparison | Every unequal-copy case also trips (b), the WARNING trap. That includes the first-copy case, and it holds even for copies 1 and 2, because the named read reads all copies. |
| Remove rule (d), the `-c 1` comparison | (d) only adds something when copy 0 is odd *and* no WARNING is printed. The fake never produces that state. Case 3's "WARNING-only" variant has equal values, so (d) is silent there too. |

"Every mutation turns a case red" (P:308) can then be met only by fake modes the plan does not list. In practice the
implementer will add them ad hoc, or record the mutations as passing.

**Fix:** add two named fake fault modes, each modelling a real hazard, and map each mutation to the case that
kills it:

- `SUPPRESS_WARNING=1`: unequal copies with no WARNING. This models a picotool output regression and kills
  mutation (a).
- `COPIES_IGNORED=1`: `-c 1` returns the vote. This models F-619's measured no-op and pairs with a copy-0-odd case
  that kills mutation (d) only while WARNING is also suppressed.

Record the mapping in the report. If (d) cannot be killed by any realistic mode, say that it is defence in depth,
and drop it from the "must turn red" list rather than leaving an unsatisfiable Done-when.

---

## B-I6 (Important): the bench rehearsal cannot execute R0/R1/R4 as written

**Sections:** §7 R0, R1, R4 (P:264-271), §2 (P:82-90), §8 (P:293-304).

1. **R1 skips phase 3, so R4 dies.** R1 runs "R's phases 0-2 and 4". R4 runs phase 5, which requires
   `$WORKDIR/blinky-mykey.signed.uf2` from **phase 3 --execute**: "no phase-3 SIGNED image found -- run phase 3
   --execute first" (R:1099). Phase 5 also runs 5b, the real firmware from `$SEEDHAMMER_DIR/seedhammerii-*.uf2`,
   unless `ACCEPT_BLINKY_ONLY=1` (R:1134-1140).
2. **The shell has neither tool set.** R0 has the operator enter `nix develop .#otp`, whose package list is
   picotool, openssl, jq, python3, bash and coreutils (P:80). R's phases 0/3/4/5 die on `tinygo`/`go` (R:132-136) and
   need the fork (R:145). The fork's own shell has 2.2.0-a4, which R's new `require_pinned_picotool` refuses. So no
   shell runs R's phases until the fork thread's PR consumes this flake. That PR is "its own PR" (P:89-90) and
   appears nowhere in §8's order or §9's Done-when.
3. **picosign versus the 2.3.1 seal.** PT D23 says 2.3.1's `seal --sign` adds a VECTOR_TABLE item and the
   EXTRA_SECURITY bit, and `sign-firmware.sh` re-signs with the fork's `picosign`. The plan assigns the re-test to the
   fork thread (P:86-87) but makes the bench depend on it. If picosign mis-parses the image, phase 5 shows "no blink"
   and dies with the M3 high-s advice (R:1121-1124), which sends the operator in the wrong direction.

**Fix:**
- R1 runs phases 0-4 (including phase 3 --execute) and states the `ACCEPT_BLINKY_ONLY` choice.
- §8 gains an explicit prerequisite: "the fork's devshell consumes this flake and picosign is re-tested on a 2.3.1
  seal". Alternatively, `devShells.otp` includes tinygo and go, and R1/R4 run with `SEEDHAMMER_DIR` set.
- Add that prerequisite to §9.

---

## B-I7 (Important): the post-write exit-code contract conflicts with the die-based shared helpers it reuses

**Sections:** §3 (P:97-101, "move is mechanical and behaviour-preserving"), §3.1 (P:124-126), §3.2 (P:131-133).

**Scenario:** the moved helpers `die` with exit 1 (R:83) on every read failure and every WARNING (R:176-186, 210-214,
367-379). The plan promises three things those helpers cannot deliver:

1. An unreadable row → exit 2 (P:126).
2. "Nothing short-circuits" (P:132).
3. Exit 3 plus "do not use" for any failed read-back (P:124).

In particular, a WARNING on the *post-write* read (the very case the copy rule exists for) exits 1, "tool error",
with no "do not use". A caller that keys on exit codes (the runbook's E3b text, and lane S, which "reuses these
facts") treats a degraded post-write state as a retry-able tool fault. There is a further trap. To catch `die`
without short-circuiting, the implementer will reach for `( … )` or `$( … )`. That loses the globals
`ROWVALS`/`SLOT_HEX`/`CHIPID_HEX` (R:204-206). A FAIL counter incremented in a subshell is also lost, which turns
"N FAIL lines printed, RESULT: PASS" into a reachable outcome.

**Fix:**
- Specify the mechanism in the plan. Either add non-dying variants (`try_read_*` that return a status and set
  globals) used by `check`, or add a `DIE_HOOK`/`trap`-based mapping, so that inside a post-write check any helper
  failure maps to exit 3 and inside `check` maps to exit 2.
- RESULT is computed from a global FAIL count **and** an asserted count of rows compared. A row that was never
  compared is a FAIL.
- Add e2e cases: a WARNING on the post-write read → exit 3 with "do not use"; an unreadable row in `check` → exit 2
  with every other row still listed.

---

## Minor

**B-M1. Optional R10 breaks R11 and is refused by R.** R10 burns slot 2 and sets KEY_VALID bit 2, so R11's
"slots 2, 3 all zero" and "KEY_VALID 0x3" fail (P:141-142, 283-286). R's `--make-otp-json` also refuses a "third
rehearsal key": `third-party-key.pem` is on its rejection list, and any key inside `rehearsal-work/` is refused
(R:646, loop at R:654ff). The "must NOT boot" result has no positive control (R's own header, R:16-23), so a bad
image is indistinguishable from enforcement. The `otp set` for bit 2 is a raw bench command with no `--ser`.
**Fix:** move R10 after R11. Use a key generated outside WORKDIR, or `ALLOW_UNCHECKED_KEY=1` stated explicitly. Show
the `--ser` on its command, and record it as evidence of "not booting", not proof.

**B-M2. CRIT1's first-copy blind spot is not listed.** `0x040` resolves to the named CRIT1 (any-3-of-8). The CRIT1
row of the table says only "raw, each copy | all 8 equal" (P:138), and the copy rule (P:148) names only "RBIT rows".
**Fix:** state that rules (b) and (d) apply to CRIT1, and add a fake case where copy 0 is the odd copy.

**B-M3. Joint KEY_VALID and slot-1 states are ambiguous.** "KEY_VALID 0x1 (pre, slot 1 empty) or 0x3" (P:142) next
to "slot 1 pre: empty or fork key; post: fork key" (P:140) reads as two independent sets. A natural implementation
then passes KEY_VALID 0x3 with an empty slot 1 (pre), and KEY_VALID 0x1 with the fork key unmarked (post).
**Fix:** give a joint truth table: pre ∈ {(empty, 0x1), (fork, 0x1), (fork, 0x3)}; post = (fork, 0x3) only. Add
e2e cases for the two bad combinations.

**B-M4. `--rehearsal-key` is optional, and the write commands do not take it.** It appears in brackets at P:107 and
is absent from P:109-113. If it is absent, the slot-0 comparison has nothing to compare against, and a
"skip if absent" implementation leaves only the ≠ `SH_SIGNKEY_HASH` test. The rehearsal-PASS evidence for F-701 is
weakened accordingly. **Fix:** required under `rehearsal` for every device command, with no default, or a default
of `$WORKDIR/factory-key.pem` that fails closed when missing. Neither profile's RESULT exits 0 without its key
comparison having run.

**B-M5. The G1 drift guard runs late and blind to the formats that matter.** The R1 diff happens after merge (§8
step 4 before step 5) and *after* phases 1 and 4 have burned OTP with parsers that have not been validated on 2.3.1.
It also happens on equal-copy state, so the WARNING/RAW_VALUE text, the multi-device `info` text, the `--ser` miss
exit status (G4) and the erase refusal (G3) are never compared. **Fix:** run `capture` (read-only) at R0.5, before
phase 1, and diff it. Also diff the R5/R8 refusal output. Commit the real transcripts as fixtures that CI replays
through the parsers, so future fake edits cannot drift unnoticed. Key the erase/save refusal on a non-zero exit, not
on G3 text.

**B-M6. The CHIPID spelling has no test vector.** The moved `chipid()` yields the row-order form
(`6f463e8d0bf609f5`); `--ser` needs CHIPID3..0 in uppercase (`09F50BF63E8D6F46`, HARDWARE_INVENTORY). If the fake
derives its serial with the same conversion as the script, the e2e stays green while every real `--ser` call
misses (exit 249). That fails safe but strands the bench at R2. **Fix:** pin that inventory pair as a shared test
vector in the e2e (the UI spec already asks for one). Uppercase or reject a lowercase operator `--ser` explicitly,
because picotool `strcmp`s it (PT D20).

**B-M7. A test entry could be committed into `retail-otp.json`.** If case 1's "test retail-otp.json entry" (P:229)
lives in, or is overridable into, `design/hardware/retail-otp.json`, a fabricated revision entry makes "a retail
check cannot pass before H0" (P:164) false. A missing field read through `jq … // empty` becomes a vacuous
comparison. **Fix:** keep fixtures under `scripts/test/fixtures/`; allow the path override only through a variable
named `…_TEST_ONLY` and print it loudly. Validate the schema of each entry (every row present, hex shape, capture-file
path and sha256 that exist in the repo) before comparing.

**B-M8. A bit that varies with the unit's history is compared as a constant.** OF 2d: the boot ROM sets
ROLLBACK_REQUIRED (BOOT_FLAGS0 bit 11) itself. H0 captures from SeedHammer #1, a unit that has run fork firmware. A
"every other bit equals recorded retail" rule (P:143) can then condemn a healthy unit, or bless a non-pristine
reference. **Fix:** record which BOOT_FLAGS0 bits can change during normal boot, compare them by an explicit rule,
and note each capture's unit history.

**B-M9. Retail rows are read with ECC rather than raw.** The check table says "raw / ECC" for
USB_WHITE_LABEL_ADDR and the table (P:145). PT condition 4 requires `-r` for a raw read of an ECC row. **Fix:** read
and compare every ECC row raw (24-bit), as `capture` already says (P:192), so a corrected single-bit addition is
visible.

**B-M10. Other loose ends:**
- *Masked exit status.* Operators saving "full output" (P:262) with `| tee` lose the exit status. Add `--log <file>`
  written by the tool itself, and make the RESULT line plus exit code authoritative.
- *shellcheck severity.* shellcheck on R today reports SC2010 (warning, R:1134) and SC2006 (R:851). Sourcing
  `lib/otp-read.sh` adds SC1091. The CI job (P:253) is red until these are addressed. Name the severity
  (`-S warning`) and `-x`.
- *Version gate.* A version-string gate cannot detect a 2.3.1 build against SDK 2.2.0 (PT D1). Add an `otp list`
  fingerprint (MAC0 present, DISABLE_BOOTSEL_EXEC2 absent).
- *Undersized flash.* erase-range's all-0xFF check passes on a unit whose physical flash is smaller than 16 MB
  (XIP aliasing; HARDWARE_INVENTORY 4 MB note). Also check `picotool info` "flash size" against the profile.

---

## Nits

- **B-N1.** P:39 says R6 "settles" F10, but §5 (P:214-215) defers F-619's status line "until §7 R6". These are
  consistent, but say where the R6 result is written (`HARDWARE_RESULT_<date>_e3a.md`, and FOLLOWUPS F-619).
- **B-N2.** P:106 lists `--board-rev`, while P:161-163 key the entries by the revision string read from the
  white-label table. Say which wins, and that a mismatch between them is a refusal.

---

## Scope

- The new script, rather than write modes in R, is **sound**. R's "no write path" promise is load-bearing in five
  places (RM §7 gotchas), and keeping it true is worth the extraction cost.
- The plan does not drop any E3a/F-701 element: DISABLE_OTP_BOOT, KEY_INVALID 0xC in R, profiles, three-copy raw
  reads, explicit-range erase/save, the injected unequal copy and the picotool pin are all present.
- It **widens** in one operationally significant way. R gains a hard 2.3.1 gate with no override, so R's SH2 modes
  stop working from the fork's shell until an unscheduled fork PR lands (B-I4, B-I6). Either schedule that PR as a
  prerequisite or defer R's gate until it lands.
- The four-system flake (including darwin) is harmless but unverified; Done-when covers Linux only.
