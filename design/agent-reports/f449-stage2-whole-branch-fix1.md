# F-449 stage 2 fix wave — scoped re-review

Reviewer: sonnet, independent of the implementer. 2026-09-23.
Scope: dm `bcaa251b..6f2fb760` and me `6ff86336..77cc1386`. Built dm at `6f2fb760`
with the pinned 1.85.0 toolchain, `CARGO_TARGET_DIR=/scratch/code/shibboleth/dm-worktrees/f449-stage2-review-target`
(not /tmp). Both worktrees left clean. Did not re-run the full gate (settled:
nextest 1535 passed / 4 skipped, all six phase-gate steps).

**Verdict: 1 new Important, 0 new Critical.** I-1 is only partially closed: the
literal reported counterexample is fixed, but a structurally identical
counterexample survives using two genuinely-chunked (not misread) cards from
unrelated chunk sets. All other findings (M-1, M-3 through M-9, the three
Nits) are fully addressed by direct measurement, each below.

## Per-finding table

| Finding | Status | Evidence |
|---|---|---|
| I-1 | **PARTIALLY ADDRESSED** — see new finding below | Reviewer's exact counterexample now exits 2, empty stdout (measured). A structurally analogous case still exits 5 with a false claim. |
| M-1 | ADDRESSED | `md repair md1q4frpqq9q2tvyyy5jmpprj5qqcyxppgqwcudeey7atgd5` (chunked, header version 9) exits 5, corrects to `md1n4f...`, stderr says "pre-v0.30 or misread", not "newer md". Measured directly (not just via the shipped test). |
| M-3 | ADDRESSED | `compose_unspendable_liana_reproduces_the_nested_accept_byte_exact` exists in `liana_evidence_legs.rs` and passes (`cargo test -p md-cli --test liana_evidence_legs`: 3/3 ok). |
| M-4 | ADDRESSED | `cmd_verify.rs:280-284` now asserts `code == 1` and `err.contains("MISMATCH")`, not just `code != 0`. `cargo test -p md-cli --test cmd_verify`: 6/6 ok. |
| M-5 | ADDRESSED | `#vexl6448` (checksum of the marker text) on `tr(UNSPENDABLE(liana),pk(@0/<0;1>/*))` → exit 0. `#6vcyxrvp` (old synthetic-hex checksum) → refused, "invalid checksum 6vcyxrvp; expected vexl6448", no leak of the synthetic hex. A third, genuinely wrong checksum (`#aaaaaaaa`) is also refused, naming the same expected value — no wrong-checksum acceptance, no silent drop. A marker template with no checksum, and a non-marker template, are both unaffected (measured). Malformed-length checksum still refused with its own message. |
| M-6 | ADDRESSED | `liana-live-gate.sh` now compares `$commit` to the expectation's `liana_commit` **before** the harness build (before `rm -rf "$src"`). Verified live: built a throwaway git repo tagged `v15.0` at a different commit (`c4568734…`) and ran the gate against it via `LIANA_CHECKOUT=<throwaway>` — failed immediately, exit 1, "FAIL -- … is v15.0 at c4568734…, but the expectation was recorded at 4684d5cb…", naming the fix. No build was attempted (fails before `cp -r harnesses/liana/src`). The real shared checkout is independently confirmed to sit at the expectation's exact commit `4684d5cb…`, so the check does not false-positive on the normal path. |
| M-8 | ADDRESSED | `main.rs:834` exit table, `md-cli/README.md`'s repair row and exit table, and `SPEC_wallet_policy_composer.md:216` all now describe the unsupported-version exit-5 meaning and `--unspendable`. Confirmed by diff. |
| M-9 | ADDRESSED | `apply_corrections` now preserves input case. `MD15PFDSSSJJVVYYW2SQRQSCY9ZSN0MKDW0FZR7` → repair prints `MD15PFDSSSJJTVYYW2SQRQSCY9ZSN0MKDW0FZR7` (exit 5), and `md decode` of that exits 0. Lowercase control unchanged. Mixed-case input (`Md15pfd...Vvyy...`) still refused pre-repair exactly as before (exit 2, "mixes upper and lower case") — the M-9 fix does not touch that refusal. |
| Nit 1 (`main.rs:317` "(Task 1b)") | ADDRESSED | Removed (diff confirmed). |
| Nit 2 (D26 comment names F-642) | ADDRESSED | `repair.rs` module doc now says "F-642, owned by the toolkit's post-stage-2 pin bump" (diff confirmed). |
| Nit 3 (F-639 body path) | ADDRESSED | `design/FOLLOWUPS.md` F-639 body now cites `crates/md-cli/src/cmd/verify.rs:62` (diff confirmed). |
| M-2, M-7 | Not fixed, per brief — filed F-643/F-644 | Confirmed both exist as OPEN entries in `design/FOLLOWUPS.md` with owning phases (stage 3, stage 4). Out of this re-review's scope. |

## New finding

### NEW-1 (Important) — the I-1 guard checks "chunked", not "same chunk set"; two genuine, unrelated chunk headers still produce a false "take the corrected card to a newer md" at exit 5

The fix requires, for `strings.len() > 1`, that every corrected string be
individually chunked (bit 0 of its first data symbol). That blocks the
reported mechanism (a single-payload string misread as a chunk header). It
does **not** verify that the chunked strings belong to the *same* chunk set
(same `chunk_set_id`, `count`, consistent `version`) before treating the
`WireVersionMismatch` as "the card's own version" — because
`reassemble`/`ChunkHeader::read` bails on the **first** header whose version
is unsupported, before the set-consistency check (`chunk.rs:374`-area) ever
runs, exactly as I-1 described for the misread case.

Counterexample, measured on the `6f2fb760` binary (both cards forced-chunked,
genuinely, at their own true chunk-of-1 headers — not misread):

```
$ md encode --force-chunked "wsh(pk(@0/48'/0'/0'/2'/<0;1>/*))"
md1fp5e0qqpqztvyyy4qqxpzsh2vk39lr744nl   # chunk-set-id 0x0d32f, v4

$ md encode --force-chunked "wsh(pk(@0/48'/1'/9'/2'/<0;1>/*))"
md1f8v7jqqpqztvywjv4qqxpzsqa49qwd4lpae92   # chunk-set-id 0x3b3d2, v4, UNRELATED message
```

Take the first card, rewrite its chunk-header version field to 12 (still a
valid, individually-chunked header) and introduce one BCH-correctable error:
`md1ep5e0pqpqztvyyy4qqxpzs7j5uasr0kygh6`. Then:

```
$ md repair md1ep5e0pqpqztvyyy4qqxpzs7j5uasr0kygh6 md1f8v7jqqpqztvywjv4qqxpzsqa49qwd4lpae92
# Repair report
#   md1 chunk 0: 1 correction at position 5: 'p' -> 'q'
md1ep5e0qqpqztvyyy4qqxpzs7j5uasr0kygh6
md1f8v7jqqpqztvywjv4qqxpzsqa49qwd4lpae92
md: repair: corrected, but this build cannot read wire version 12 (accepted: 4, 8)
md: repair: take the corrected card to a newer md, which may read wire version 12
exit 5
```

Both stderr claims are false in the same way I-1's were: no md, old or new,
will ever successfully reassemble these two strings — they carry different
`chunk_set_id`s and (post-correction) different header versions (12 vs 4),
so **any** md (including a hypothetical v12-capable one) would fail
`ChunkSetInconsistent` on this exact pair the instant it tried. Confirmed
independently: `md decode` of the two clean cards together already refuses
with "wire-format version mismatch: got 12" before reaching that
consistency check, i.e. no build, old or new, treats this pair as one
descriptor.

**Why Important, not Critical:** same reasoning the I-1 report itself used —
no wrong content is emitted (each printed string is a genuine BCH correction
of its own input), so this is not a funds/restore Critical. But it is
exactly the class I-1 named ("the tool's claim about what it found is
false", exit 5 implying "stdout is the corrected [restorable] card" when it
is two incoherent unrelated fragments) — the same severity bucket the
original reviewer assigned to I-1 itself.

**Plausibility:** requires the operator to submit chunks from two different
*genuinely multi-chunk* wallets in one `repair` call (the reported I-1
scenario used the far more common default single-payload cards). Narrower
than the original, but not contrived — multi-chunk cards exist specifically
for large/high-value multisig policies, and mixing plates from two such
backups during a restore is a plausible operator mistake.

**Suggested direction (not authoritative):** before accepting the
"card's own version" framing for a multi-string set, also require the
corrected strings' `chunk_set_id` (and ideally `count`) to agree — i.e. parse
each corrected string as a chunk header (not just its bit-0 flag) and check
mutual consistency up to, but not including, the version check that already
failed, before trusting a single `got` value as authoritative for the whole
set.

## Machine-checked

- dm build at `6f2fb760`: `cargo build --locked -p md-cli` — clean.
- `cargo test -p md-cli --test cli_repair_unsupported_version`: 10/10 ok
  (includes the shipped mixed-set, real-multi-chunk, parity, and uppercase
  tests).
- `cargo test -p md-cli --test cmd_verify --test liana_evidence_legs --test liana_input_side`: 6/6, 3/3, 17/17 ok.
- `cargo test -p md-codec --test correct_chunks`: 6/6 ok.
- `liana-live-gate.sh` fail-closed path exercised live against a throwaway
  git repo (not the shared checkout); real checkout independently confirmed
  at the expectation's exact commit.
- Both worktrees confirmed clean (`git status --porcelain`, empty) before and
  after; no scratch files left (a temporary `crates/md-codec/tests/zz_scratch_review.rs`
  used to derive the NEW-1 fixture was removed and the clean status re-verified).

ready to ship: no
