# CONSULT — P0 row 6: does the pre-parser argv guard cover `me seal`?

**Consulted 2026-08-27.** One question, one answer. Read-only except this file.

## Answer: **B.** Guard `seal` too, and declare `--allow-argv-secret` on `seal`.

## Reasoning

1. **C's clean version does not exist, and that is decisive.** The gate's
   observable is stderr, and `seal` IS in its scope: `seal`'s `--in` error path
   is `eprintln!("me: cannot read {}: {e}", path.display())`
   (`crates/me-cli/src/main.rs`, `Seal` branch) — the same
   name-the-token mechanism F-266 records for `sysw show`'s `<FILE>` — so a
   whole-surface exemption reopens a real F-266-class leak on
   `me seal --in <ms1>` the moment the cross-product is corrected to include
   `seal`, while a positional-only exemption forces the pre-parser guard to
   reimplement clap's grammar, exactly the hand-list fragility the plan spent
   two rounds (round-9 C-1, round-10 I-1) eliminating.
2. **A reverses a recorded decision that is not the implementer's to reverse:**
   the positional is a deliberately retained affordance with a follow-up number
   attached (F-102, cited at the call site), and B preserves its substance while
   strictly *tightening* today's behavior — warn-and-proceed becomes
   refuse-unless-flagged — whereas A would also make the downstream
   defense-in-depth refusals (e.g. the secret-in-`--plaintext` test at
   `seal_cli.rs:193`) unreachable end-to-end, since the guard would pre-empt
   them on every argv shape.
3. **B is the sibling precedent, not new design:** `sysw pack` has the identical
   shape — positional records that may be secret — and §6d already designed the
   per-surface override mechanism for exactly this; declaring the flag on `seal`
   is applying the plan's own mechanism to a surface the plan's list forgot
   (the list is short by two subcommands regardless, so the fold must amend the
   plan under any option).
4. **The positive-controls principle lands on B:** the `seal` row of the
   cross-product gets a positive control exactly parallel to the one the gate
   already asserts for `sysw pack` — `me seal --allow-argv-secret <fixture>
   --out …` proceeds past the guard, the F-102 warning still fires downstream —
   so the guard is uniform across all surfaces with no exemption hole and no
   deleted path.
5. B's cost is one flag declaration plus ~7 test invocations gaining
   `--allow-argv-secret`; A's cost is a deleted documented channel plus doc and
   F-102 record updates; C's cost is a shipped stderr leak — and shipping a leak
   is worse than both.
6. The plan's surface list `{bare, bundle, sysw, sysw pack, sysw show,
   sysw wipe, help, sysw help}` must be corrected to enumerate all five real
   top-level subcommands, with `seal`'s override rows added to the gate — file
   that as part of the row-6 fold, not as silent guard behavior.

## `me hash` — one line

The guard breaks `me hash` **worse** than `seal` and it cannot inherit `seal`'s
answer: its positional legitimately carries argv-forbidden *bearer* classes
(`tx:`/`mt1` are the public records the §6.6 hash exists to cover) and `Hash`
declares no `--in` and reads no stdin, so the guard makes hashing any
transaction-bearing payload impossible — measure it and schedule its own
resolution (most likely an `--in` channel, since `--allow-argv-secret` is a
misnomer for bearer-public records).
