# R0 round 3 — Plan A @ `6e5c0bd`, fold verification BY EXECUTION

Reviewer: opus. All 9 tasks written into a pristine scratch crate and run.
Verdict: **2 Critical / 3 Important / 1 Minor / 1 Nit — BLOCKED.**

**This round is different: the round-2 folds introduced the defects.** Five
compile errors; the plan as written stops at Task 4. Graded per the brief's
pre-commitment that a surviving mutation is Critical.

## Verified clean
Per-task counts wire 8 · crypto 5 · passphrase 5 · record 7 · pubhash 5 ·
container 4 · uf2 2 · `seal_cli` 9. clippy `-D warnings` clean after Tasks 7 and
9. **All seven vectors A–G and both §6.6 literals independently recomputed in
Python** (no Rust involved) — every length and sha256 matches; A–F untouched by
the fold. Passes under MSRV 1.85.0 `--locked`. Vector G groups into four cards
(`d`/841149 ×6, `k`/153720, 153721, 153723 ×2 each), each decoding standalone.
Mutations (a), (b), (d) all killed by exactly the named test.

## CRITICAL 1 — `me seal` printed a hash that did not describe the blob it wrote
M-1 removed the CLI trim, so `public` held raw argv, but the §6.6 hash was still
computed from it verbatim while the blob's public section is what
`encode_section` emits — trimmed. **Measured:** one leading/trailing space gives
a **byte-identical blob** (`pub_len=203`, same bytes) and a different hash:
`fcc7 217e …` clean vs `2aee 5d32 …` padded. For an unsealed payload the hash is
the ONLY integrity control, and the CLI's own text says a mismatch means the
payload was altered. No test covered it.
**Folded:** trim in the hash refs, matching `check_public` and `run_hash_cli`,
plus a CLI killer asserting `me seal`'s printed hash equals `me hash`'s.

## CRITICAL 2 — the lowercase guard had no killer (mutation (c) SURVIVED)
Deleting `record_or_mnemonic`'s `is_uppercase` check left the **entire 166-test
suite green**. The mutation table listed the killer as a manual `me seal "BACON…"`
invocation — while every other row names a real test function, and the plan's own
rule is that a mutant with no killer is a gap.
**Folded:** `refuses_an_uppercase_bip39_mnemonic` as a real lib test; row
re-pointed.

## IMPORTANT 1 — `decode_public_set` did not compile
The match unified `Result<_, md_codec::Error>` with `Result<_, mk_codec::Error>`
→ E0308. Round 2's HRP version used two separate `?` statements and never hit it.
**Folded:** stringify inside each arm.

## IMPORTANT 2 — the `AsRef<str>` fold did not compile, and could not reach its goal
Three sites: `r[pos..]` cannot index `&S`; four test call sites lost inference;
and `Payload { secret }` got `Vec<Zeroizing<String>>` against a `Vec<String>`
field. The fold had deleted the only line that made it type-check without
changing `Payload`. The generic was dead weight — `encode_section` is never
called with anything else — so M-4's stated goal was unreachable as specified.
**Folded by RETRACTION:** reverted to `&[String]`, and the Global Constraint now
says what zeroize actually covers instead of claiming a guarantee not delivered.
argv already exposes these records via `/proc/$PID/cmdline`.

## IMPORTANT 3 — `plaintext.clone()` on `&[String]` yields `&[String]` (E0308)
**Folded:** `to_vec()`.

## MINOR / NIT — counts still wrong
"50 at Task 7, 52 after Task 8" measured as **52 / 54**; `mod` is 18, not 16 —
the fold subtracted uf2's 2 but ignored the 2 tests the round-2 folds added.
**Folded** to 54 / 56 (this round adds two more), and the stale "164 tests"
comment removed.

## Observation folded from the reviewer
The spec's §6.3 printed mk1 csids `852310 / 852311 / 852308`; the measured values
are `153720 / 153721 / 153723`. Controller then verified **both sides
independently** — `mk.ParseHeader` on the device and
`StringLayerHeader::from_5bit_symbols` on the host return identical values, and
the records reproduce byte-identically across `mnemonic bundle` runs. The wrong
figure had been copied out of the earlier §6.3 review report rather than
measured. Corrected in the spec with its provenance recorded.
