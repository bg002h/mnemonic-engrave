# Fold r1: FIX_bughunt_me.md

Commit: 7a892cef (branch bughunt/triage, local, not pushed). Only design/FIX_bughunt_me.md edited.

- M1 -> FIX line 174 (13 run, 2 passed, 11 failed, 693 skipped).
- I1 item 1 -> §1 #1 step 2 (lines ~86-90).
- I1 item 2 -> §2 row #1 (line ~160).
- I1 item 3 -> §3 table row 01b plus a "Why" bullet (01b) with the test body from the report.
- I1 item 4 -> §6 expected 1 (705, 691+7+7), 5 (14/14, 693 skipped), "before #4" line.

STOP: none blocking. Two unquoted row-format fields, filled minimally: the 01b "RED on base" cell says PASSES on base (regression guard) plus the report's first-written failure text (no bughunt.rs line number given in the report for that row, omitted); "GREEN after" cell reads "#1 (stays green)". Also §6 "6 new rows" became "7 new rows" per the report's "+ 7 new rows".

0 Critical / 0 Important
