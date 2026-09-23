# F-449 stage 2: whole-branch adversarial review

Reviewer: opus, independent of the implementer and of the plan's author. 2026-09-23.
Scope: descriptor-mnemonic `25acb33c..bcaa251b` (10 commits, md-codec 0.47.0 / md-cli 0.19.0) and
mnemonic-engrave `fa8bba6f..6ff86336` (7 commits). Built at bcaa251b with the pinned 1.85.0 toolchain,
`CARGO_TARGET_DIR=/scratch/code/shibboleth/dm-worktrees/f449-stage2-review-target`. I built the
0.18.0 base (`25acb33c`) in a temporary detached worktree for differential runs, then removed it.
Both worktrees are clean at their heads. I applied one mutation and reverted it.

**Verdict: 0 Critical, 1 Important, 9 Minor, 3 Nit.** The funds and restore surface is sound. Omitting the flag
leaves `md compose` byte-identical to 0.18.0 (69,888 + 135 differential invocations). `--unspendable liana` reproduces
Liana's descriptor B byte-exact. No new mint refusal reaches decode. The one blocking finding is in
`md repair`'s new exit-5 branch: it treats a `WireVersionMismatch` that comes from mixing strings as
the card's own version. For a set of readable cards it then exits 5 and says "take the corrected
card to a newer md", where 0.18.0 exited 2.

---

## Important

### I-1 — `md repair` exits 5 and blames the wire version when the "version" is a misread from mixing single-string cards in one call

`crates/md-cli/src/cmd/repair.rs:96` routes every `WireVersionMismatch` to
`corrected_but_unsupported` (`:166`, returns `Ok(5)` at `:211`). With two or more strings,
`decode_with_correction` always goes through `reassemble`. `reassemble` reads each string as a
**chunk header** (`chunk.rs:351`, `ChunkHeader::read`), and that read comes before any set-consistency
check (`chunk.rs:374`). A **single-payload** card read as a chunk header gives a bit-shifted
"version" (`[div][v3][v2][v1]`). So a v4 or v8 card that this build reads fine reports version 2
or 10. The branch assumes the mismatch is a property of the card. For multi-string input that
contains a non-chunked string, the assumption is false.

Counterexample, measured on the bcaa251b binary. Two cards that each decode alone at 0.19.0:
- A = `md15pfdsssjjtvyyw2sqrqscy9zsn0mkdw0fzr7` (`md encode "wsh(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))"`; `md decode A` exits 0)
- Ae = A with one substitution at data position 9 (`t`→`v`): `md15pfdsssjjvvyyw2sqrqscy9zsn0mkdw0fzr7`
- B = `md1gppqqxq799p20d5hxuzu2c9la` (`md encode "tr(UNSPENDABLE(liana),{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})"`)

```
$ md repair Ae B            # 0.19.0 (bcaa251b)
# Repair report
#   md1 chunk 0: 1 correction at position 9: 'v' -> 't'
md15pfdsssjjtvyyw2sqrqscy9zsn0mkdw0fzr7
md1gppqqxq799p20d5hxuzu2c9la
md: repair: corrected, but this build cannot read wire version 10 (accepted: 4, 8)
md: repair: take the corrected card to a newer md, which may read wire version 10
exit 5
$ md repair Ae B            # 0.18.0 (25acb33c)
md: repair: wire-format version mismatch: got 10; accepted versions: 4, 8
exit 2
```

Both statements on stderr are false: this build reads both cards, and no md reads "version 10"
because no card has that version. Exit 5 is REPAIR_APPLIED, which scripts read as "stdout is the
corrected card". Here stdout is two unrelated cards, not one set. More measured variants, all
0.18.0 exit 2 → 0.19.0 exit 5: `[v12 1-err, v4 clean]` → "newer md … version 14";
`[v4 clean, v12 1-err]` and `[v4 1-err, v12 clean]` → "version 2 … pre-v0.30 or misread".

No wrong string is emitted: each corrected string is a genuine BCH correction of its own input.
So this is not a funds or restore Critical. It is the class the brief names ("return 5 … where it
should return 2"), and the tool's claim about what it found is false. The plan and SPEC §8.9 scope
the branch to "a card whose wire version this build does not support". The mixed-set case is
outside that scope, and the tests (`cli_repair_unsupported_version.rs`) run single strings only.

Fix, a few lines: take the exit-5 branch only when the mismatched version is the card's own. For
`strings.len() > 1`, require every corrected string's chunked flag (bit 0 of its first symbol, as
`decode_with_correction` reads it at `chunk.rs:660`ff.) to be 1, and fall through to today's exit 2
otherwise. Add one mixed-set CLI test asserting exit 2 and empty stdout.

---

## Minor

- **M-1 — the parity half of repair's advice is untested; the mutation survives.** I applied
  `if got % 2 == 0 && got > newest` → `if got > newest` (`repair.rs:201`), and
  `cli_repair_unsupported_version` stayed **6/6 PASS**. Reverted. No test uses an odd version above 8
  (for example 9, 11, 13 or 15, which would wrongly get the "newer md" advice under the mutation).
  Add one fixture.
- **M-2 — the new branch drops decode as a second check behind BCH miscorrection.** Before this
  change, a word with more than 4 errors that BCH mis-corrected onto another codeword was still
  refused by the decode (exit 2). Now, if the mis-corrected header lands outside {4, 8} (14 of 16
  values), repair exits 5 with a **wrong** "corrected" string. Measured rate, 60,000 random trials on
  a v4 card: 5 errors 0 miscorrections, 6 errors 1 (caught by decode at a later field), 9 errors 0,
  20 errors 0. That is roughly 1e-5 or less, and the only consequence is a string no md decodes.
  Log it; not blocking.
- **M-3 — no committed test ties `md compose --unspendable liana` to Liana's own record.** The
  goal's end-to-end claim rests on the Task 5 transcript plus a chain of tests: the golden,
  `liana_changes_the_internal_key_and_nothing_else` (kofn only), and the evidence legs, which use
  decompose templates. I re-ran the transcript's claim and it holds today:
  `md compose --wrapper tr --path 2of2 --path 1of1,older=26280 --path 1of1,older=52560 --unspendable liana`
  gives the same template as `md decompose` of case `nested-2of2-two-recoveries-tr`. Rendered with
  the case's keys and fingerprints, it equals Liana's `descriptor_with_checksum` byte-exact. Pin
  this as a test in `liana_evidence_legs.rs`: compose, then descriptor, compared with the corpus.
- **M-4 — the F-639 mismatch leg accepts any non-zero exit.** `cmd_verify.rs:279`
  `assert_ne!(code, 0)` plus `!contains("is refused")` would also pass on an unrelated error (a
  template-parse refusal, say). Assert `MISMATCH` in stderr.
- **M-5 — a checksummed marker template can no longer be encoded with any checksum.** The F-641
  tree parse (`template.rs:1133`) checks `#…` over the marker text. The later `Descriptor::from_str`
  checks it over the substituted synthetic hex. Measured on `tr(UNSPENDABLE(liana),pk(@0/<0;1>/*))`:
  `#vexl6448` gets "expected 6vcyxrvp", and `#6vcyxrvp` gets "expected vexl6448". 0.18.0 accepted
  `#6vcyxrvp`, the checksum over synthetic text no operator writes. The first error names a checksum
  over text the operator never wrote, which is F-641's own leak class. No md output emits a
  checksummed marker template (`md decode` measured), so the impact is low.
- **M-6 — the live gate records the Liana commit but does not enforce it.** `liana-live-gate.sh`
  checks the tag (`describe --exact-match`) and the verdict diff. `liana_commit` is only printed
  (`:209-210`). `DESIGN_coordinator_compatibility.md:512` says the gate "pins the tag and commit".
  Compare `$commit` with `exp_commit`, or reword the design doc.
  (Separately, I exercised the uncovered wrong-tag branch with a throwaway git repo via
  `LIANA_CHECKOUT`, without touching the shared checkout. With the repo at `v14.0` the gate exits 1
  with the named fix. It is sound.)
- **M-7 — `--unspendable liana` composes several Liana-incompatible shapes without a warning.**
  Measured exit 0 with no warning: `--path 2of3,unsorted --path 2of2` (two unlocked primaries),
  `--path 2of2 --path 1of1,after=800000` (absolute-timelock recovery),
  `--path 2of2 --path 1of1,older=100 --path 1of1,older=100` (duplicate recovery timelock), and
  `--path 2of3,unsorted --experimental` (no recovery path). Liana refuses these at import, loudly
  and before funding, so the wrong outcome is not worse than silence. This is implementer concern
  (4), widened. **Follow-up, not blocking.** The live gate makes collecting harness evidence for
  these cheap, which is what the "harness is the oracle" rule needs.
- **M-8 — user-facing docs that this diff made incomplete.** `md repair --help`'s exit table
  (`main.rs:834`) and the md-cli README repair row still describe 5 as success-after-decode, with no
  mention of the unsupported-version meaning. The md-cli README compose row does not list
  `--unspendable`. `SPEC_wallet_policy_composer.md:216` still says "otherwise NUMS".
- **M-9 (pre-existing, not a regression; file a follow-up) — `md repair` on an all-uppercase card
  emits a mixed-case "corrected" string that md refuses to read.** `apply_corrections`
  (`repair.rs:219`) writes the lowercase `now` char into uppercase input. Measured on both 0.18.0 and
  0.19.0: `md repair MD15PFDSSSJJVVYYW2SQRQSCY9ZSN0MKDW0FZR7` prints
  `MD15PFDSSSJJtVYYW2SQRQSCY9ZSN0MKDW0FZR7`, and `md decode` of that exits 1 with "mixes upper and
  lower case". QR alphanumeric form is uppercase, so this is a plausible restore journey. It is not
  in FOLLOWUPS (grepped). The new exit-5 branch inherits it (measured on the v12 fixture uppercased).

## Nit

- `md compose --help` says "(Task 1b)" (`main.rs:317`): plan jargon in the user help.
- `repair.rs`'s D26 divergence comment names "a follow-up" without F-642 (the implementer noted this).
- F-639's filed body cites `crates/md-codec/src/verify.rs:62`. The file is
  `crates/md-cli/src/cmd/verify.rs`. The Status line is right.

---

## Verified sound (measured, not read)

- **Flagless compose is unchanged.** `scripts/gen-compose-golden.sh`, run against the 0.18.0 binary,
  regenerates `compose_pre_unspendable.json` **byte-identical** (14 rows, 10 non-zero cells). The
  golden commit `c9d926fe` precedes the flag commit `6e918a8f`. Wider differential, 0.18.0 vs
  0.19.0, comparing exit code, stdout and stderr: **69,888** `--path` invocations (16 path specs in
  every 1-, 2- and 3-tuple, × 4 wrappers × {plain, `--json`, `--experimental`, both}) and **135**
  preset invocations, including bad wrappers and presets. **0 differences.**
- **The Step 1 floor test is non-vacuous.** It asserts `golden.len() == 14`, a 4-wrapper and
  6-preset SET, and exit 0 plus byte equality per row, and it reads the golden at runtime.
- **Liana key correctness.** The descriptor B round trip is byte-exact against the corpus (M-3).
  The evidence legs mint through the wire and compare with Liana's own strings, and the ACCEPT set
  is asserted non-empty.
- **Read path.**
  - `validate_unspendable_shape` has exactly two callers: encode under `Admission::Enforce`
    (`encode.rs:251`) and `md compose` under `--unspendable liana`. F-638 changes only the variant
    payload.
  - F-636's new branch sits inside the `len >= 2` group loop, and every such group already
    returned `Err`, so the refusal set is unchanged.
  - F-641: I ran 21 marker templates through `md encode` on both builds (origins, `h`/`'`, nested
    braces, hashlocks, `after`, whitespace, `<2;3>`, keypath-only, `TR(`, malformed). No template
    that 0.18.0 accepted is refused, except the synthetic-hex checksum (M-5).
  - `md decode --json` of a kind-1 card is identical across builds.
- **F-639 cannot pass a mismatched card.** Both sides use the same serialiser, whose bytes are
  admission-independent (`encode_payload_inner`: the policy block is pure refusal, and the version
  is `d.wire_version()` on both sides). Measured: the §6 card verifies OK against its own template,
  gives MISMATCH against its `multi_a` twin (same size, byte 9), and gives MISMATCH against its
  NUMS twin.
- **Repair, single string.**
  - A clean v12 card exits 2 with empty stdout.
  - v12 with 1 to 4 errors exits 5 with the correct string, in text and JSON.
  - v12 with 5 errors exits 2 with empty stdout (D28 holds).
  - `[v12 1-err, v12 5-err]` exits 2 with empty stdout (atomic).
  - `correct_chunks` is the old loop verbatim. The single-string re-parse uses the same
    `parse_chunk_symbols` on either the verbatim input or its own re-encoding, so there is no new
    error path in `decode_with_correction`.
- **Mutation claims.** M4 is inert, and the cause is confirmed: `policy_shape.rs:289-302` pushes a
  real key as its own unlocked `Branch`. The M1b replacement control is a genuine test of
  refusals gated on shape rather than on the flag.
- **Records.**
  - Every file:line the spec re-pointed resolves at bcaa251b: `main.rs:1102,1141` `network_str`;
    `chunk.rs:666` `chunked_flag`; `compose/mod.rs:673` `template_with_origins`;
    `template.rs:1738` `walk_tr`.
  - Every FOLLOWUPS closure (F-636, F-638, F-639, F-640, F-641) names a commit that carries the
    change.
  - Every test named in a Status line exists: 4 of 4 grepped.
  - The CHANGELOG entries match the code.

ready to ship: no
