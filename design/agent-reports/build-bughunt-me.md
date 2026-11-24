build: 8f91ff54 gate PASS; 0 STOP

## Commits (branch fix/bughunt, from 286c3bf9, local only)
- 82b366ef test: bughunt me RED row 01b (FIFO guard); rows 01-05d red on base
- 05c57012 fix(me): to_uf2 refuses a blob over the 64 KiB seal region (FIX L5, R4)
- 12357ca5 fix(me): pack with passphrase and no secrets emits the unsealed container (FIX #3, R3)
- 2c6587d4 fix(me): render_plate removes a stale plate before spawning (FIX #2, R2)
- e387a82d fix(mnemonic-io-lib): write_private truncates after chmod, regular files only (FIX #1, R5)
- 3a41a14c spec: delta for FIX L5, #2, #1 (FIX section 2)
- 8f91ff54 fix(me): refuse argv records together with --in; spec row (FIX #4, R1)

## RED evidence
Row 01b was absent from bughunt.rs (triage had 13 rows); added per FIX §3, committed with the RED run.
RED run (/scratch/cache/bughunt/me/red.log): 14 run, 3 passed (01b guard, 06, 07 FALSE POSITIVEs), 11 failed, 693 skipped.
Failing rows: 01, 02, 02b, 03, 03b, 04, 04b, 04c, 04d, 05, 05b; 02b failed with the FIX's stated EmptyOutput text.
Per-item GREEN runs after each commit: 05/05b, 03/03b, 02/02b, 01/01b, 04-04d all passed.

## Gate (logs in /scratch/cache/bughunt/me/)
1. g1.log nextest workspace: 705 tests run: 705 passed, 2 skipped
2. g2.log doc: exit 0, 2 x "0 passed; 0 failed"
3. g3.log fmt --check: exit 0
4. g4.log clippy -D warnings: exit 0
5. g5.log bughunt_me_: 14 tests run: 14 passed, 693 skipped

Spec delta (FIX §2) applied in 2 commits; no FOLLOWUPS entries were given by the FIX, none added. Toolkit untouched.

0 Critical, 0 Important
