# E3a plan R0 round 5, reviewer A (mechanical fold verification of draft 5)

Reviewed: `design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md` draft 5 at mnemonic-engrave `3b044f8`
(cited **P:line**), diff `1ed979a..3b044f8`. Previous report: `e3a-plan-r3-a.md` (0C/1I/3M/2N).
Scope: (1) each R4A finding fixed as asked; (2) every §6.3 row walked against §3.3 and the §6.1
fault modes; (3) §6.3 renders as one table; (4) no contradiction with §3.3 / §7. No fresh audit.

**Counts: 0 Critical / 0 Important / 2 Minor / 1 Nit**

## (1) Round-4 findings

| finding | fixed? | evidence |
|---|---|---|
| R4A-I1 equivalent mutants | Yes | P:246-247 states cond 2 implies cond 1 (T ⊆ E). P:399 respells the bound row "E replaced by copy 0 \| T in conditions 1-3". P:406-408 lists "superset" and "field" as equivalent, no row. |
| R4A-M1 source of per-copy values | Yes | P:253-254 conds 1-3 use per-copy reads, RAW_VALUE only via cond 4. Case 0x803/0x003/0x003 under `COPIES_IGNORED` at P:363-364; row at P:400. |
| R4A-M2 bit 11 untested | Partly | Case 5f exists (P:364-367) and kills the mutant. Not folded: bit 11 in one copy only, and the derived-row case. See R5-M2. |
| R4A-M3 slot-1 key scope | Yes | P:445-446 names the three commands; matches P:144-146 and the P:148 parenthetical. Residual R5-N1. |
| R4A-N1 table broken | **No** | R5-M1. |
| R4A-N2 runbook / R7 flags | Yes | P:306 has `--ser S`; P:453-454 spells R7. |

## (2) Mutation walk (E, T, equal-copy test, heal conds 1-4 as in §3.3; BOOT_FLAGS1 E = 0xC03, T = 0xC00)

| mutant | named case | walk | killed? |
|---|---|---|---|
| drop copy rule (a) | case 3, copy 2 odd, `SUPPRESS_WARNING=1` | No WARNING, no RAW_VALUE, so (b) and the cross-check are silent. Vote = copies 0,1 = expected, so (c) passes. Only the copy-2 bare read differs. Mutant PASS, real exit 2. | Yes |
| drop `-c 1` from (a) | case 3, copy 0 odd, `SUPPRESS_WARNING=1` | Bare reads of copy 0 resolve to the named register (F16); only `-c 1` sees copy 0 (`COPIES_IGNORED` unset). Without it copies 1,2 agree, vote = expected: mutant PASS, real exit 2. | Yes |
| drop WARNING trap (b) | case 3c | `COPIES_IGNORED`: `-c 1` = vote = expected; bare copies 1,2 expected; `SUPPRESS_RAW_VALUE`: no RAW_VALUE. Only the WARNING refuses. | Yes |
| drop `--ser` | `FAKE_REQUIRE_SER=1`, case 13 | exit 99 / argv log | Yes |
| `-n` before `-c` | fake argv order | exit 99 | Yes |
| drop post-write check | case 6 `FAIL_READ_AFTER_WRITE=1` | real exit 3, mutant exit 0 | Yes |
| E replaced by copy 0 \| T in conds 1-3 | case 5d, 0x903/0x103/0x103 | Real: 0x903 has bit 8 outside E, and 0x903 & ~T = 0x103 ≠ 0x003: refused, exit 2. Mutant E' = 0xD03: cond 1 ok, cond 2 (0x103 = E'&~T) ok, cond 3 (0x903 \| 0xC00 = E') ok, cond 4 ok (`-c 1` = RAW_VALUE[0] = 0x903). `otp set` writes 0xD03 to every copy; post-check against true E fails: exit 3 vs 2. | Yes |
| drop cond 4 alone | 0x803/0x003/0x003, `COPIES_IGNORED=1` (P:363-364) | Per-copy reads all 0x003 (`-c 1` returns vote). WARNING and RAW_VALUE print (copies differ), so the row is unequal and goes to the heal rule. Conds 1-3 on 0x003 x3: subset of E, &~T = 0x003, copy0 \| T = 0xC03: pass. Cond 4: RAW_VALUE [0x803,0x003,0x003] ≠ per-copy: real exit 2, state identical. Mutant writes (picotool sets from true copy 0: 0x803 \| 0xC00 = 0xC03 everywhere), post PASS, exit 0, state changed. | Yes |
| E's bit 11 fixed at 0 | case 5f | BOOT_FLAGS0 0x800 x3, `disable-otp-boot`. Real: E = 0x2800, copies equal to E & ~T, branch 2, write, post PASS, bit 11 kept. Mutant: E = 0x2000, E&~T = 0, copies 0x800 match neither branch 1 nor 2, exit 2. The test asserts a write. | Yes |
| drop RAW_VALUE cross-check (cond 4 and the equal-copy test) | case 5, 0x103/0x003/0x003, `COPIES_IGNORED=1` | Per-copy reads 0x003 x3, no RAW_VALUE/WARNING test: "equal" = E&~T, branch 2. Picotool writes 0x103 \| 0xC00 = 0xD03 everywhere; post-check fails: exit 3 vs real 2. | Yes |
| derived flag accepts any equal value | case 5e | KEY_INVALID 0x1 x3: mutant derives `--key-invalid 1`, pre-check passes, BOOT_FLAGS0 write, post-check with derived 1 passes: exit 0. Real exit 2. | Yes |
| identity gate skipped for erase | case 7 | slot 0 ≠ hash of rehearsal key on a retail-shaped state: real exit 2 before any argv; mutant records an erase argv. | Yes |

No unkilled mutation. The two equivalent mutants are correctly excluded (T ⊆ E gives cond 2 ⇒ cond 1).

## (3) Rendering and (4) contradictions

### R5-M1 (Minor) — §6.3 still renders as two tables plus a run-on paragraph (P:393-395)

**Evidence.** The note was removed but its leading blank line was not: P:394 is empty between the
`-c 1` row (P:393) and the WARNING-trap row (P:395). GFM ends the table at the blank line; rows from
P:395 to P:404 have no header and render as pipe text. That is the gate list for §9, including every
row this fold added. The fold table's R4A-N1 line ("note moved below the table") is therefore not
true in effect.
**Fix.** Delete the blank line at P:394.

### R5-M2 (Minor) — case 5f: contradictory second half, and part of R4A-M2 not folded (P:365-367)

**Evidence.** "the same with copy 3 lacking bit 13 after `bf0-copy3`": §3.6 (P:286) and R5 (P:451)
make `bf0-copy3` put bit 13 **in** copy 3 only, so after it copy 3 has the bit and copies 1,2 lack it.
"Lacking" reads the other way. The first half of 5f already kills the mutant, so no mutation survives,
but an implementer may build 0x2800/0x2800/0x0800 (a post-interruption state) instead of
0x0800/0x0800/0x2800 (the bench state). R4A-M2 also asked for bit 11 set in one copy only →
`disable-otp-boot` exit 2, and `invalidate-spare-keys` passing with bit 11 in the derived row; neither
is in 5f. The first is a stated rule (P:223-224) with no case; both fail safe.
**Fix.** Write "the same with bit 13 in copy 3 only (the `bf0-copy3` state) and bit 11 in all copies".
Add: "bit 11 in one BOOT_FLAGS0 copy only → `disable-otp-boot` exit 2, state identical;
`invalidate-spare-keys` with bit 11 set in all BOOT_FLAGS0 copies passes".

### R5-N1 (Nit) — `--rehearsal-slot1-key` on the other commands (P:371-372 vs P:144-148)

Case 7b gives `--rehearsal-slot1-key` to the identity-gate commands (`erase-range`, `inject-copy`).
§3.1 lists the flag for `check` and the two write commands only, and says nothing about the others
accepting it. R3 (P:445-446) now correctly limits it to three commands. One clause fixes it: either
7b names `check`/the write commands, or §3.1 says the flag is accepted and ignored on the others.

No contradiction found between the fold and §3.3 or §7. R6/R9 on the bench states still heal under the
cond 1-4 text; R9's 0x803/0x003/0x003 heals on silicon (F10 true) and is refused only under
`COPIES_IGNORED`, which is consistent with the new mutation row.

VERDICT: 0C / 0I / 2M / 1N
