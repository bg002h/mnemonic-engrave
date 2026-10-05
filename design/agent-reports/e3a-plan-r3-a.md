# E3a plan R0 round 4, reviewer A (proportional re-review of draft 4)

Reviewed: `design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md` draft 4 at mnemonic-engrave `1ed979a`
(cited **P:line**), diffed against draft 3 (`ebcc2b2`). Previous report: `e3a-plan-r2-a.md`
(0C/3I/5M/2N); fold table P:548-561. R = `scripts/pico2-bootkey-rehearsal.sh`.
Scope: did draft 4 fix each round-3 finding, and did the fold add a defect. Settled facts (F14,
`--ser` 249, `otp set -s` from copy 0 to all copies F11/F19, F9, F10, F18) taken as given. One
further fact checked against picotool 2.3.1 source (`main.cpp` at tag commit `2041936`,
lines 9236-9282): `RAW_VALUE=` and the WARNING are printed **only** when the redundant copies differ
(`if (diff)`), and `-c 1` sets `redundancy = 1` so the named read returns row 0's raw value. So
"no WARNING and no RAW_VALUE" is a sound equality signal for §3.3, and a no-RAW_VALUE equal-copy
read cannot be confused with a RAW_VALUE-bearing one. Read-only apart from this file.

**Counts: 0 Critical / 1 Important / 3 Minor / 2 Nit**

## Round-3 finding → fixed?

| finding | fixed? | evidence |
|---|---|---|
| R3A-I1 derived flags loosened; E ambiguous | **Yes** (one fix item not folded, R4A-M2) | P:214-219 legal values only (KEY_INVALID ∈ {0,0xC}, DISABLE_OTP_BOOT ∈ {0,1}), derived row by copy rule; P:220-224 E from expected state, never copy 0; bit 11 from agreed copies, disagreement exit 2. Walk of the round-3 harm: 0x103×3 → `disable-otp-boot` derived KEY_INVALID 0x1 → exit 2, no write (case 5e). `invalidate-spare-keys` on 0x103×3: copies equal, ≠ E (0xC03), ≠ E&~T (0x003) → branch 3, exit 2. |
| R3A-I2 three mutations survive | **Mostly**; the walk found a pre-existing row with the same defect (R4A-I1) | see the mutation walk below |
| R3A-I3 cross-check only inside heal | **Yes** | P:225-228: equal only if no WARNING, no RAW_VALUE and the three per-copy reads agree, in every branch; cond 4 (P:248-250) requires RAW_VALUE. Walk: 0x103/0x003/0x003 under F10-false → WARNING → unequal → heal → cond 4 (RAW_VALUE[0] 0x103 ≠ `-c 1` 0x003) refuses, exit 2. |
| R3A-M1 R8 unreachable | Yes | P:444-448: R7 inject-copy exit 3 is the F10-false signal; R8 read still run; R9/R10 skipped. R5/R6 cannot reveal F10-false (copy 0 is not odd there, vote = copy 0), so R7 is indeed the first place. |
| R3A-M2 bench flags, key paths | Yes (Minor residue R4A-M3) | P:433-436; R:67, R:1066 paths. |
| R3A-M3 R1 stock list | Yes | P:428-431 lists every rehearsal-column cell at pre-R2 value. R2 writes only slots 0/1, KEY_VALID and CRIT1 (R:960-970, R:1074-1082), so R3's rehearsal column is reachable from that list. |
| R3A-M4 `--probe-only` | Yes | P:136; R4 P:437-439 expects exit 2. |
| R3A-M5 shape normalisation | Yes | P:410-412. |
| R3A-N1 / R3A-N2 | Yes | P:9-10; P:143-147. |

## Mutation walk (§6.3, P:384-399)

| mutant | named case | walk | killed? |
|---|---|---|---|
| drop copy rule (a) | case 3, copy 2 odd, `SUPPRESS_WARNING=1` | no WARNING, no RAW_VALUE (redefined, P:331-332), so (b) and (d) are silent; vote = copies 0,1 = expected, (c) passes. Without (a), PASS; with (a), bare read of the copy-2 row differs, exit 2. | Yes |
| drop the `-c 1` read from (a) | case 3, copy 0 odd, `SUPPRESS_WARNING=1` | `0x048`/`0x04b` bare resolve to the named register (F16), so only `-c 1` sees copy 0. With it dropped, (a) compares copies 1,2 only (equal), (b)/(d) silent, (c) passes: PASS. Real tool: `-c 1` (F10 true in the fake by default) differs, exit 2. | Yes |
| drop the WARNING trap (b) | case 3c, copy 0 odd, `COPIES_IGNORED=1` + `SUPPRESS_RAW_VALUE=1` | (a): bare copies 1,2 and `-c 1` (= vote) all equal; (d): no RAW_VALUE; (c): vote = expected. Only (b) refuses. | Yes |
| drop `--ser` | `FAKE_REQUIRE_SER=1`, case 13 | unchanged from round 3 | Yes |
| `-n` before `-c` | fake exit 99 | unchanged | Yes |
| drop post-write check | case 6 `FAIL_READ_AFTER_WRITE` | real exit 3; mutant exit 0 | Yes |
| heal accepts a superset copy | case 5 superset | **equivalent mutant**, see R4A-I1 | **No** |
| heal bounded by copy0 \| T | case 5d 0x903/0x103/0x103 | true E 0xC03: cond 1/2 refuse, exit 2. Mutant with E′ = copy0\|T = 0xD03 **in conds 1-3**: all pass, cond 4 passes, `otp set -s` writes 0x903\|0xC00 = 0xD03 to all copies (copies 1,2 are subsets, healed), post-check KEY_INVALID 0xD ≠ 0xC, exit 3. **Only if the mutant replaces E in cond 2 as well**; a cond-1-only mutant is equivalent (R4A-I1). | Yes, as defined in R4A-I1's fix |
| drop RAW_VALUE cross-check (cond 4 and equal-copy test) | case 5, 0x103/0x003/0x003, `COPIES_IGNORED=1` | mutant: per-copy reads 0x003×3, no RAW_VALUE/WARNING test → "equal" = E&~T → branch 2 → picotool writes from true copy 0: 0xD03 all copies → post-check fails, exit 3. Real: exit 2, state identical. | Yes (see R4A-M1 for cond 4 alone) |
| derived flag accepts any equal value | case 5e, KEY_INVALID 0x1 ×3, `disable-otp-boot` | mutant: derived `--key-invalid 1` passes pre-check; BOOT_FLAGS0 equal = E&~T → write → post-check with derived 1 → PASS exit 0. Real exit 2. | Yes |
| identity gate skipped for erase | case 7 | unchanged | Yes |

## Bench walk R5-R10 against the new §3.3 (rehearsal; X = BOOT_FLAGS0 bit 11 as the ROM left it, same in all copies)

- R5 `bf0-copy3`: copies X, X, X|0x2000. `check --disable-otp-boot 0` → (a)/(b)/(d) refuse. ✓
- R6 `disable-otp-boot`: derived KEY_INVALID 0 (legal). E = 0x2000|X. Unequal (WARNING+RAW_VALUE) → heal: each copy ⊆ E; copy&~T = X = E&~T; copy0\|T = E; RAW_VALUE = bare + `-c 1` (copy 0 = vote here, so even F10-false passes). `otp set` writes X|0x2000 everywhere. Post PASS. R's phase 5 needs only KEY_VALID = 3 (R:1095). ✓
- R7 `bf1-copy0`: copies 0x803/0x003/0x003; `check --disable-otp-boot 1 --key-invalid 0` refuses. ✓
- R8: `-c 1` 0x803, vote 0x003, RAW_VALUE[0] 0x803 (picotool source above). ✓
- R9 `invalidate-spare-keys`: derived DISABLE_OTP_BOOT 1 (legal). E = 0xC03. Heal: 0x803, 0x003 ⊆ 0xC03; &~T all 0x003; 0x803\|0xC00 = 0xC03; cond 4 ✓. Write computes 0xC00 \| 0x003 \| 0x803 = 0xC03 (F19) to all copies. Post PASS; phase 5 KEY_VALID still 3. ✓
- R10 PASS. ✓

No write path found that burns a bit outside E: branch 2 writes copy0\|T = (E&~T)\|T = E; the heal's cond 3 forces copy0\|T = E; F19's "every bit copy 0 holds" is bounded by cond 2. No exit-code loop: branch 1's post-check failure is exit 2; exit 3 only after `otp set`. No contradiction found among §3.0/§3.2/§3.3/§6/§7 beyond the items below.

---

## Important

### R4A-I1 — the "superset" mutation row is an equivalent mutant, and the copy0|T row is killed only if its mutant replaces E in cond 2 (§3.3 P:242-247, §6.3 P:395-396, §9 P:474)

**Evidence.** T ⊆ E for both writes (bit 13 ⊆ E for BOOT_FLAGS0; 0xC00 ⊆ E for BOOT_FLAGS1, P:220-224).
Cond 2 (P:245) says every copy & ~T = E & ~T. Then every copy = (copy & T) | (E & ~T) ⊆ T | E = E. That is
cond 1 (P:243). So **cond 2 implies cond 1**, and any mutation confined to cond 1 cannot change behaviour.

- "heal rule accepts a superset copy | case 5 superset" (P:395). The natural mutant drops or relaxes
  cond 1. A superset copy (holding a bit outside E, and therefore outside T) still fails cond 2, so
  the case still exits 2. The mutant survives every possible case. §9's "every §6.3 mutation is
  killed by its named case" (P:474) cannot be met. The implementer must either guess a different
  mutant or report a red run that does not exist.
- "heal bounded by copy0 | T instead of E | case 5d" (P:396). "Bounded" reads as cond 1, the bound
  condition. With E′ = copy0|T substituted in cond 1 only, case 5d (0x903/0x103/0x103) is still
  refused by cond 2 against the true E: 0x903 & ~T = 0x103 ≠ 0x003. The mutant survives. It dies only
  when E′ replaces E in cond 2 as well (walk above: exit 3 vs 2).

This is the same class as the round-3 "field" half, which draft 4 correctly annotated as equivalent
(P:389-390). Its two siblings were not re-examined.

**Fix.**
- §3.3: state that cond 1 is implied by cond 2 (because T ⊆ E). Keep it as the readable safety
  statement, or fold it into cond 2.
- §6.3: delete the superset row, or annotate it as equivalent next to the "field" note. Respell the
  bound row as "E replaced by copy 0 | T in every heal condition (1-3)", killed by case 5d.
- The superset *case* in case 5 stays as a behaviour test.

## Minor

- **R4A-M1 (§3.3 P:242-250, §6.2 case 5 P:356-357).** The plan does not say which copy values conds
  1-3 evaluate: the per-copy reads (bare plus `-c 1`) or RAW_VALUE's list. Case 5's parenthetical
  "the RAW_VALUE cross-check is the only refusal" is true only for the per-copy source. With a
  RAW_VALUE source, cond 1/2 also refuse 0x103.
  - The combined row (cond 4 + equal-copy test) is killed under either source.
  - A *cond-4-alone* mutant is killed by 0x103 only under the per-copy source. Under the RAW_VALUE
    source it survives every listed case.
  
  Both sources are safe, since picotool's write reads true copy 0 (F19). The gap is test coverage
  and an implementer guess, not an unsafe write. **Fix:** say that conds 1-3 use the per-copy reads
  and that cond 4 is their cross-check against RAW_VALUE. Add a case: 0x803/0x003/0x003 under
  `COPIES_IGNORED=1` → `invalidate-spare-keys` exit 2, state identical. This is a legitimate heal
  state, refused only by cond 4 under either source, so it kills cond-4-alone. Add a row for it.
- **R4A-M2 (§6.2, R3A-I1 fix bullet 3 not folded).** No case puts BOOT_FLAGS0 bit 11
  (ROLLBACK_REQUIRED) in the state.
  - An implementation that fixes E's bit 11 at 0 passes every case. It then refuses R6 on a bench
    board whose ROM has set bit 11; R1 allows bit 11 (P:429).
  - The opposite slip, taking bit 11 from copy 0 alone, is also untested.
  
  Both fail safe (exit 2), but the first blocks the bench. **Fix:** add case 5f.
  - Bit 11 set in all three BOOT_FLAGS0 copies: `disable-otp-boot` writes and passes. Re-run is
    branch 1 (no write), exit 0. `invalidate-spare-keys` passes with bit 11 in the derived row.
  - Bit 11 in one copy only → `disable-otp-boot` exit 2, state identical.
- **R4A-M3 (§3.1 P:143-147 vs §7 R3 P:435-436, case 7b P:365-366).** R3 says "every later step passes
  the same `--ser` and key flags". That includes `--rehearsal-slot1-key` on `erase-range` (R4) and
  `inject-copy` (R5, R7). §3.1 requires that flag only on `check` and the two writes, and does not say
  whether the other rehearsal commands accept it. Case 7b ("also with `--rehearsal-slot1-key` given")
  implies they do. If an implementer treats it as a usage error (exit 1), R4/R5/R7 fail at the bench.
  That is visible, not unsafe. **Fix:** §3.1: "accepted and unused on `erase-range`, `save-range`,
  `inject-copy`", or have R3 say which flags each later step takes.

## Nit

- **R4A-N1 (§6.3 P:384-399).** The "Heal judged by field" note sits between the second and third
  table rows, after a blank line. GFM ends the table there, so the remaining nine mutation rows,
  the actual gate list for §9, render as one run-on paragraph of pipes. Move the note below the table.
- **R4A-N2 (§5 P:303, §7 R7 P:443).** The RUNBOOK's E3b `check` line omits `--ser S`, which §3.1
  requires everywhere. R7's `check` shows only the two changed flags (`--slot1 valid` and the key
  flags implied by R3). Spell `--ser S` in the RUNBOOK line and write R7 as "R3's flags with
  `--disable-otp-boot 1`".

VERDICT: 0C / 1I / 3M / 2N
