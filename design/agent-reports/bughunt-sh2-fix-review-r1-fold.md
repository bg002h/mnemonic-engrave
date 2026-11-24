# Fold of FIX review r1 (sh2) — commit 85231b1b

No STOP. Edited only design/FIX_bughunt_sh2.md; pushed to origin.

| I | landed at line |
|---|---|
| I-1 | 226 (§2.14 var text); closing sentence in §7 at 452 |
| I-2 | 459 (§8 item 1); item 4 struck at 464 |
| I-3 | §7 rows 448 and 449; TestDescriptors nonstandard row deleted; closing sentence 452 |
| I-4 | §2.7 paragraph 155; §6.1 row 376 |
| I-5 | §2.5 text; §6.1 row 373; file body under §8 item 4 (464) |
| I-6 | §8 item 0 at 458 |
| I-7 | §8 item 5 at 505 |

M-1, M-2, N-1 filed in §9.2 (new "FIX review r1 minors" table), owner "end-of-project minors".

Judgement calls a reviewer should check (the brief gave no exact rule):
- I-1: the fix text is a sentence, so the old function-var phrase became "function vars (declared below)" and the verbatim text follows the Behaviour sentence.
- I-1 + I-3 both edit the §7 closing sentence; combined into one (I-3 sentence, then I-1 parenthetical).
- I-2: kept the abbreviated `ed3706e2…` exactly as the fix text has it (the full hash is no longer in the doc).
- I-3: unescaped \" and \\n from the report to plain Go text; `||` written `\|\|` for the table cell.
- I-5: §8 has no free slot for a file body, so it sits under item 4 (no new item, no new §9.1 line).
- Counts: 23 -> 25 in §6.2 Counts and the §8 Gate. §3 code-delta row for nonstandard/parse.go not extended with fix-01 (not in a fix text).
- Owner for M-1/M-2/N-1 chosen as the sh2 default "end-of-project minors" (brief named none).
