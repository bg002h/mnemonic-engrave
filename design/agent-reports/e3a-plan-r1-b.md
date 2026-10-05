# E3a plan R0 round 2, lens B: failure states, operator error, false PASS, whether gates run

Reviewer: independent adversarial R0 (lens B, round 2), 2026-10-05. Target:
`design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md` draft 2 (commit 33d4c11), cited **P:line**. Round 1:
`e3a-plan-r0-b.md`. R = `scripts/pico2-bootkey-rehearsal.sh`. PT-src = picotool 2.3.1 source
(`2041936`, scratchpad clone). BIN = `/nix/store/5cxc3bjs…-picotool-2.3.1`. I ran BIN with no device
attached for every argv shape the plan names (all exit 249 with "with serial number ZZPROBE00000000"),
and `otp list` (exit 0, `OTP_DATA_MAC0` present, no device needed). Nothing else was executed. No file
was changed except this one.

**Counts: 0 Critical / 2 Important / 10 Minor / 2 Nit**

---

## Round-1 finding → fixed?

| round-1 finding | fixed? | evidence (draft 2) |
|---|---|---|
| **B-C1** heal vs equality; binary stage | **Yes** (but see B2-I1 on the new rule) | `--stage` is gone: "`check` takes the whole expected state explicitly (no stage, no defaults)" (P:143). Writes derive state: "Each write command reads the board, works out its own pre-state and post-state" (P:191-192). Order independence: disable keeps "KEY_INVALID either 0 or 0xC … (whichever is found is kept in the post-state)" (P:194-195); invalidate keeps "DISABLE_OTP_BOOT either value with equal copies (kept)" (P:200). Heal rule defined (P:203-209). The check never heals (P:208-209). Cases 4 ("Both orders of the two writes pass") and 5. R5→R6 and R7→R9 traced: the rule admits both injections and produces equal copies. |
| **B-I1** interrupted write | **Yes** | "3 a write was issued and anything after it failed: a non-zero exit of the write itself, any read failure, WARNING or mismatch" plus the fixed re-run text (P:111-115). The heal rule is "the admission rule for a re-run after exit 3" (P:203). Case 6 covers `FAIL_SET_AFTER=1/2` and `FAIL_READ_AFTER_WRITE`. PT-src `main.cpp:9711-9729` confirms that `otp set` always writes (no "unchanged" short-circuit), so a re-run does heal. |
| **B-I2** profile trusted for erase/inject | **Yes** | "Profile identity gate … for every command except `capture` … A failure exits 2 before any write or erase" (P:135-138). inject-copy is `--profile rehearsal` only (P:129). Case 7. |
| **B-I3** inject writes all copies | **Yes** | "Exactly two cases, no free `--bits`" plus a post-injection read of every copy (P:236-241). The fake models numeric-to-named resolution and all-copy `set` (P:270-272). Case 10 is the mutation that drops `-c 1`. The ECC sub-variant is moot: PT-src `:9718` gives `bEcc` = 0 for an unnamed row without `-e`. Residue at B2-M1. |
| **B-I4** gates cannot run | **Yes** | "old fake with its version line updated to 2.3.1 … not in CI … runs in `.#otp` at §8 step 1 and at the bench (R0)" (P:332-333). Case 9 is dropped (P:394). `.#otp` carries tinygo and go (P:81). |
| **B-I5** mutations unsatisfiable | **Partly** | Fault modes `SUPPRESS_WARNING`, `COPIES_IGNORED`, … (P:280-282) and a mapping table (P:313-322). Rows (a) and `-c 1` are now killable. Two rows still survive their named case: **B2-I2**. |
| **B-I6** bench cannot execute | **Yes** | "R2. R's phases 0, 1, 2, 3 (`--execute`), 4" (P:346). `ACCEPT_BLINKY_ONLY=1` and the BOOTSEL re-entry (P:348-349). The R0 picosign check comes before any write (P:340-342). R is not gated to 2.3.1 (P:85-87). |
| **B-I7** exit codes vs dying helpers | **Yes** | `try_<name>` readers set `ERR`; "Nothing is called inside `$(...)`"; main-shell `ROW_NAMES`/`ROW_RESULTS`; "a row never compared is a FAIL" (P:103-110). Residue at B2-M4 (a count, not a set) and B2-N1. |
| B-M1 R10/R11 | Yes | R11 now runs after R10, with a key from outside `rehearsal-work/`, `ALLOW_UNCHECKED_KEY=1`, `--ser` shown, and "evidence, not proof" (P:360-363). |
| B-M2 CRIT1 first copy | Yes | The copy rule covers "CRIT1 ×8" (P:169). Case 3 covers "copies 0 and 7 of CRIT1". |
| B-M3 joint states | Yes | Joint table (P:149-153). Case 2 covers both bad combinations. |
| B-M4 rehearsal key optional | Yes | "required under `rehearsal` (no default) and refused under `retail`" (P:134). |
| B-M5 G1 drift late | **Partly** | R1 runs `capture` before any write, and CI replays (case 12). The R5/R7 refusal output (the WARNING/RAW_VALUE text) is still neither diffed nor committed. Case 12 is also vacuous until the bench: **B2-M7**. |
| B-M6 CHIPID spelling | Mostly | Case 11 is the vector, and lowercase is uppercased (P:132, 306-307). The new CHIPID deny-list has no spelling rule: **B2-M6**. |
| B-M7 test entry in retail JSON | Mostly | `scripts/test/fixtures/`, the `_TEST_ONLY` override with a warning, and the schema check (P:178-182). Residue at B2-M9. |
| B-M8 history-dependent bit | Yes | "bit 11 (ROLLBACK_REQUIRED) either value" (P:163). Unit history is recorded (P:177). |
| B-M9 ECC rows raw | Yes | "ECC rows are always read raw (24-bit)" (P:172-173). |
| B-M10 tee / shellcheck / SDK / small flash | Yes | `--log` (P:116). `shellcheck -x -S warning` (P:327). R's warnings are fixed (P:250). The `otp list` fingerprint (P:83-84) works with no device (BIN). The alias probe (P:220-222) is new; see B2-M3. |
| B-N1 where R6's result goes | Yes | "Write R8's reading into the result file and F-619" (P:356-357, 261). |
| B-N2 `--board-rev` | Yes | Dropped (P:423). |

---

## Important

### B2-I1 — the heal rule is stated over the target *field*, so `invalidate-spare-keys` admits, and burns into all three copies, a stray KEY_INVALID bit for slot 0 or slot 1

**Sections:** §3.3 heal rule (P:203-209); invalidate-spare-keys (P:199-202); F11 (P:58); PT-src `main.cpp:9693-9702`.

**What picotool writes.** For `otp set -s BOOT_FLAGS1.KEY_INVALID 0xc`, PT-src computes
`value = (0xc<<8 & mask) | (old & ~mask)` and then `value |= old` (`-s`). Here `old` is copy 0's raw row
(`:9628`), and that value is written to all three copies. The write therefore propagates **every bit
copy 0 holds**, including bits inside the KEY_INVALID field (bits 8-11) that are not target bits.

**The rule, as written:**
- (1) "every copy … holds no bit outside … (copy 0 | target bits)". Copy 0 is a subset of itself, so (1)
  can never catch a stray bit that sits *in copy 0*.
- (2) "the copies differ only inside the target **field's** bits". For KEY_INVALID the field is bits
  8-11, not the target bits 10-11.
- (3) "every bit outside the target **field** equals the expected value". Bits 8 and 9 are inside the
  field, so (3) does not check them either.

**Scenario.** A SeedHammer has BOOT_FLAGS1 copy 0 = 0x000103 (KEY_VALID 0x3, plus KEY_INVALID bit 8
from an earlier interrupted, mistaken write) and copies 1 and 2 = 0x000003. The vote is 0x003. Slot 0 is
valid, and SeedHammer's retail firmware still boots. `check` refuses the board (unequal copies), which is
correct and harmless. Then `invalidate-spare-keys --execute` runs:
- pre-state: the identity gate passes; `--slot1 valid` passes; KEY_INVALID is "0 or heal rule"; the heal
  rule passes (1), (2) and (3) as quoted.
- write: 0x000D03 goes to all three rows. Every row is a superset of its old value, so the boot ROM
  accepts each one.
- post-check: KEY_INVALID 0xD ≠ 0xC, exit 3. A re-run is refused (0xD is neither 0, nor healable, nor
  0xC).

Slot 0, which is SeedHammer's own key, is now invalid on all three copies, irreversibly. Before the run,
the board was refused but recoverable to retail use. Afterwards, retail firmware can never boot it. The
plan's own sentence, "Any other unequal state is refused with exit 2 and no write" (P:208), is violated
by the tool's own write. This is exactly "a heal leaves the board worse". The same wording applies to bit
9 (slot 1, the fork key), which would also break the Refugium image.

**Fix.** Define the heal rule over the **target bits** T (DISABLE: 0x002000; invalidate: 0x000C00), not
the field:
(1) every copy, copy 0 included, ⊆ (E | T), where E is the expected value of the row *without* T. This
uses the expected value, not copy 0.
(2) every copy & ~T == E & ~T. All bits outside T, including KEY_INVALID bits 8-9 (expected 0), are equal
across copies and equal the expected value.
Add e2e case 5 variants: copy 0 alone holds KEY_INVALID bit 8 (and bit 9) → exit 2, state identical. Add
a §6.3 mutation, "heal rule judges the field instead of T", killed by that case.

### B2-I2 — two §6.3 mutation rows survive their named cases, so the Done-when "every §6.3 mutation is killed by its named case" is still unsatisfiable as written

**Sections:** §6.1 fault modes (P:280-282), §6.2 cases 3 and 4 (P:290-294), §6.3 (P:313-322), §9 (P:379).

1. **"drop the WARNING trap (b) | a case with equal printed values but WARNING (fault in the fake text)".**
   No such fault mode exists in §6.1, and no such case exists in §6.2. A realistic killer does exist:
   copy 0 odd with `COPIES_IGNORED=1`. Then (a) sees copies 1 and 2 equal, and `-c 1` returns the vote,
   which is also equal, so only (b)'s WARNING refuses. Dropping (b) turns the result into PASS. But
   `COPIES_IGNORED` is defined and **used by no case**: case 3 lists only the `SUPPRESS_WARNING` variant.
2. **"drop `--ser` from the builder | case 4 `--ser` mismatch … with `DEVICES=2` it refuses".** With
   `--ser` dropped, the fake runs on the one attached device. The tool then reads its CHIPID and compares
   it with `--ser` (P:132-133), and that refuses with exit 1, the same exit case 4 expects. With
   `DEVICES=2`, the "Exactly one RP2350 in BOOTSEL" check (P:133) refuses regardless of `--ser`. Both
   named cases stay green under the mutation. This is the guard for F14/A-C1, a round-1 Critical.

**Fix.**
- Add case 3c: "copy 0 odd, `COPIES_IGNORED=1` → `check` exit 2". Name it as the killer of mutation (b),
  and delete the "fault in the fake text" row.
- For `--ser`, make the fake refuse (exit 99) any device-touching call that lacks `--ser` when
  `FAKE_REQUIRE_SER=1` (which `run-e2e-otp.sh` always sets). Alternatively, have every case assert on
  the fake's argv log that each device call carries `--ser <S>`. Name that as the killer.
- Have the implementer's report show each mutation's red run (case id plus failing assertion), not just
  a mapping.

---

## Minor

**B2-M1. The heal rule and inject-copy's verification depend on F10 before R8 measures it.** Both read
"copy 0" with `-c 1` (P:205, 240). R6 (heal) and R7 (inject verification) run before R8, the first
silicon reading of F10 (P:353-355).
- If F10 is false on silicon, R7's verification sees `-c 1` = the vote (0x003) ≠ old|bit and exits 3
  ("re-run this same command") before R8 can record the reason. The re-run injects again as a no-op and
  loops.
- In the heal path, a false F10 can admit a copy-0-lacks-bit / copies-1,2-have-it state as "equal".
  picotool then computes from the real copy 0 and partially burns (F11).

**Fix:** in both places, also parse the named read's `RAW_VALUE` (F9 prints every copy) and require
`-c 1` = RAW_VALUE[0]; refuse on disagreement. Give inject-copy its own exit-3 text ("stop the rehearsal;
do not re-run"). Add a case: heal under `COPIES_IGNORED=1` → refused.

**B2-M2. The write pre-state's scope and its three-way disjunction are underspecified.**
- The bullets list only the identity gate, slot 1, KEY_INVALID and DISABLE (P:194-201). Draft 1's "full
  check" is gone, so an implementer can burn DISABLE on a retail unit whose CRIT1, page locks or
  white-label rows mismatch. Only the post-check would catch it, with exit 3 and a misleading "re-run".
- "already 1 in all three copies" (disable) vs. "already 0xC" (invalidate; copies not required equal),
  and the precedence among "0 / heal / already", are unstated.
- The no-write branch can fail its post-check and exit 3, although "a write was issued" is false. A
  re-run then repeats exit 3 forever, with no exit 2 to tell the operator to stop.

**Fix:** pre-state = the full §3.2 check with the derived flags, with the target row judged by the heal
rule instead of the copy rule. Order the branches: final value in all copies → no write; else zero in all
copies or heal-admitted → write; else exit 2. Use exit 3 only if `otp set` was invoked; otherwise exit 2.

**B2-M3. The alias probe writes flash, and nothing says it is gated by `--execute`.** "Under `retail`, an
alias probe first: write a 4 KiB marker at `0x10000000`" (P:220-222), while "Write commands are dry-run
unless `--execute`" (P:139). A dry-run `erase-range --profile retail` on SeedHammer #1, which is the
natural E3b preview, would destroy sector 0 of its firmware before any go-ahead, against "no SeedHammer
write before E4" (P:258). Nothing ever exercises the probe on silicon either: the rehearsal profile skips
it, although the 4 MB bench board is exactly the case it should detect.

**Fix:** run the probe only after `--execute` and the confirmation. Use a random marker and print it.
Add a §7 step that runs the probe logic against the 4 MB Pico 2 (e.g. `erase-range --profile rehearsal
--probe-only`, or a 16 MB range in rehearsal) and must report aliasing. Add a dry-run case asserting
that no `load` argv is recorded.

**B2-M4. "Number of rows compared" is a count, not a set, and for retail it can be derived from the
board.** A bug that compares one row twice and skips another passes the count (P:108-110). The
white-label string rows are "every valid STRDEF's string rows" (P:166), so a count taken from the board's
own valid bits and lengths is self-referential.

**Fix:** require that the set of row names compared equals the required set. Derive that set from the
profile and the matched retail entry, never from board reads.

**B2-M5. The retail entry's prose contradicts the per-row table, and some bits are unspecified.**
- "each with every row above … A unit passes only if it equals one entry in every recorded row"
  (P:175-178) includes CHIPID, slots, BOOT_FLAGS0 DISABLE and KEY_INVALID. Read literally, only H0's own
  unit in H0's own state passes, and every post-write check exits 3.
- The table also leaves retail BOOT_FLAGS1 bits 4-7, 12-15 and 20-23 uncompared (P:162).
- The rehearsal "KEY_INVALID = …; other bits 0" contradicts KEY_VALID 0x3 in the same row.
- "one entry" must mean the *same* entry for every row. Matching rows against different entries is a
  false-PASS route.

**Fix:** "equals the same entry in every cell the table marks 'recorded entry'". For BOOT_FLAGS1, compare
every bit outside KEY_VALID and KEY_INVALID against the entry (retail) or against 0 (rehearsal).

**B2-M6. The SeedHammer CHIPID deny-list has no spelling rule.** R's `chipid()` yields the row-order form
`6f463e8d0bf609f5` (R:309-323). HARDWARE_INVENTORY's key is `0x09f50bf63e8d6f46`. If the list in
`otp-read.sh` and the comparison use different forms, the deny-list never matches, and it fails silently
(P:137). Case 7 also does not isolate it: a retail-shaped state already fails on slot 0.

**Fix:** store the list in `--ser` form and compare after the same conversion. Case 7b: slot 0 = the
rehearsal key hash plus a listed CHIPID → exit 2.

**B2-M7. Case 12 is vacuous until after the bench, and the refusal formats are never replayed.** The
fixtures come from §7 R1 (P:308-309), which runs after merge (§8 step 5). So "run-e2e-otp.sh passes …
in CI" (P:379) is met with zero fixtures, and §8 has no follow-up PR that commits them. R1 sees only
equal copies. The WARNING/RAW_VALUE text from R5/R7 (B-M5) is still not diffed or committed.

**Fix:** case 12 fails if the fixture directory is empty once a marker file says the bench has run (or
list it as "activates in the bench PR"). Commit the R5/R7 refusal transcripts as well, and add the
bench-result PR to §8.

**B2-M8. The CI gates run but do not gate, and the probe has a blind spot.**
- Branch protection requires only `test (rust + go)` (release.yml:9). The new `otp tooling e2e` and
  `picotool argv probe` jobs are advisory, so a later PR or flake bump can merge red. That defeats the
  probe's purpose ("so a parser change in a future picotool turns CI red").
- release.yml grants `contents: write` / `id-token: write` workflow-wide (release.yml:43-46), and the
  probe job runs a third-party nix-install action under those permissions.
- Measured on BIN: the probe proves `--ser` parsed, but an option placed *after* `--ser` on `otp get` is
  swallowed as a selector and still yields 249 plus "with serial number" (A-C1 table row
  `--ser X -c 1 0x048`). If the probe's argv list is hand-written rather than generated by the builder, it
  can drift from the builder.

**Fix:**
- Make both jobs steps of `test (rust + go)`, or name the branch-protection change and who makes it.
- Set job-level `permissions: contents: read`.
- Generate the probe's argv from the builder functions (a `pt_*` print mode), and assert that the set of
  shapes the e2e records via the fake's argv log ⊆ the probed set.

**B2-M9. The `_TEST_ONLY` retail-JSON override is honoured on a real write.** A variable left exported
in the `.#otp` shell (the same shell runs the e2e at R0) is only "a warning line" (P:180).

**Fix:** refuse the override when `--execute` is given, and make RESULT read `RESULT: TEST ENTRY — not a
retail check` (not PASS) when it is set.

**B2-M10. Exit-3 guidance on a SeedHammer is incomplete.** "If the re-run refuses, do not use this board"
(P:115) does not cover a re-run that exits 3 again (B2-M2's loop, or a persistent write error such as a
page-lock refusal). The runbook also does not say whether Brian's per-step typed go-ahead (P:257-258)
covers the re-run.

**Fix:** "If the re-run exits non-zero for any reason, stop". State in the E3b runbook that one re-run is
pre-authorised by the step's go-ahead and that a second needs a new one.

---

## Nits

- **B2-N1.** The R wrapper sketch `otp_field() { try_otp_field "$@" || die "$ERR"; }` (P:105-106) prints
  nothing, but R calls `KV="$(otp_field …)"` (R:348, 704, 791…). The wrapper must `printf` the global.
  R's old e2e will catch this. Say it so the implementer does not "fix" R's call sites.
- **B2-N2.** "old fake with its version line updated to 2.3.1" (P:332). R will now call `version -s`
  (F17), and FK's `version)` arm (FK:22) prints the full line regardless of `-s`. Say that the stub
  honours `-s` and prints the bare `2.3.1`.

---

## Scope

- No scope creep beyond the R0 fold, apart from the alias probe, a new flash write on a seed device
  (B2-M3), and the nix CI job (B2-M8). Both are justified by round-1 findings, but each needs the gating
  stated.
- The new mechanisms otherwise hold up:
  - **Order independence.** Traced both orders through §3.3.
  - **Re-run after exit 3.** A re-run is admitted only for subset copies under F11, given the B2-I1
    correction and an accurate copy 0 (B2-M1).
  - **Identity gate.** No path runs `erase-range` or `inject-copy` on a SeedHammer under `rehearsal`,
    because slot 0 ≠ `SH_SIGNKEY_HASH` is decisive regardless of the CHIPID list.
  - **Two devices.** Refused.
  - **CI argv shapes.** All the shapes the plan names parse with `--ser` on BIN (exit 249). The
    fingerprint needs no device.
