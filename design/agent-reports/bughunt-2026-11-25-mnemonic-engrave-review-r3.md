# Review r3 (fold reviewer): mnemonic-engrave PR #68 (`claude/project-thread-utm1aa`, head 8ad7ad7)

Scope: the fold of r2 I-1 (cd269bc), plus a check that db03395 and 8ad7ad7 change nothing else. I edited, committed and pushed nothing on the branch. I used one temporary detached worktree, in the scratchpad, for the pre-fold RED run. It has been removed and pruned.

**Verdict: 0 Critical, 0 Important, 1 Minor, 2 Nit. r2 I-1 is fixed.** The new origins are true: I re-derived all 16 children from the parent key. Regeneration changed only the three fields it claims to. The new column predicate fails on the pre-fold file and passes now.

## Evidence

- **Test run.** `CARGO_INCREMENTAL=0 cargo test --locked -j 2 -p mnemonic-engrave --no-fail-fast --test descriptor_seam --test descriptor_refusals --test descriptor_as --test record_corpus` (the package is `mnemonic-engrave`, not `me-cli`). Results: descriptor_as 30 ok, descriptor_refusals 45 ok, descriptor_seam 19 ok, record_corpus 6 ok, 0 failed.
  - **Environmental note (not a finding).** My first run failed `the_file_carries_one_named_test_per_section_6_row`: `Row::ALL` had no `md1-origin-mismatch`. The cause was stale artifacts. r2 built a 67c3f1c worktree into the shared `target/`. Cargo hashes path packages by their workspace-relative path, so that build overwrote the same artifacts, and their newer mtimes made them look fresh.
  - `cargo clean -p mnemonic-engrave` fixed it.
  - Anyone who reuses this clone's `target/` after a worktree build should clean the package first.

### 1. Are the 16 keys really children 0..15 of the account key? Yes.

- I wrote a stdlib Python check (scratchpad `r3/derive.py`): base58check decode, then a full BIP-32 CKDpub of the parent with HMAC-SHA512 and secp256k1 point addition.
- Parent: `K0` (`xpub6DiYrfRw…hpan`, the `dc567276` fixture cosigner). It is depth 4 with child number `0x80000002` (2h), which is consistent with `48h/0h/0h/2h`.
- Each `children21.txt[i]` for i = 0..15 matched on every field:
  - depth 5;
  - child number i (unhardened);
  - parent fingerprint `5eb7b5c6` = HASH160(K0)[:4];
  - chain code = I_R;
  - pubkey = point(I_L) + K_par.
- All 16 are OK. The origins `48h/0h/0h/2h/<i>` are therefore true for depth, child number and path, not just consistent with the refusal predicate.
- The new row is also now a positive witness that E3 admits depth-5 keys with unhardened last steps.
- The master fingerprints are still the synthetic `00000001..00000010` (N-2).

### 2. Did regeneration change only that row? Yes.

- `git diff cd269bc~1 cd269bc` on the JSON: two hunks, 3 lines.
- A structural diff over all 74 `vectors` (scratchpad parse): only `accepted/sh-wsh-sortedmulti-16-keys` differs, and only in `input`, `sha256` and `canonical`.
  - `address_0` (`39rQdU…d3Q`) and `address_1` are unchanged.
  - No field was added or dropped.
  - `refusal_rows` (36) and `_comment`/`invariant` are unchanged.
- In the new row:
  - `sha256` = sha256(`input`);
  - `canonical` = `input` + `#khry96w4`;
  - all 16 origins carry `/<i>`.
- The file's sha is `e2a2308d…0596b6`, equal to the pin at `crates/me-cli/tests/descriptor_seam.rs:46`.
- Population: host_admits_true 19, md1_admits_true 15, rows 74. All are unchanged and equal to `POP` (`descriptor_seam.rs:151-160`).
- The cited allowlist (`md1-fixed-index`/`md1-no-wildcard`, `cited == 6`) is untouched and passes. Fix option (a) needed no widening.
- The fork copy is already in step: `/home/user/seedhammer` at c4529d4 holds a byte-identical file (`cmp`), and `seamVectorsSHA256` = `e2a2308d…` (`nonstandard/descriptor_seam_test.go:42`).

### 3. Does the new column test fail before the fold and pass after? Yes.

- **RED.** In the worktree at HEAD I restored the cd269bc~1 JSON and the old pin `683ccd3d…`, leaving the new predicate. Result: `the_md1_column_matches_the_representability_rules` FAILED at `descriptor_seam.rs:928`, "accepted/sh-wsh-sortedmulti-16-keys: md1_admits", left false, right true. The other 18 passed.
- **GREEN** at HEAD (above).
- **Coverage.** The test asserts on every row (`checked == POP.rows`, 74).
  - `md1_refusals(p, "", "")`'s two string arguments are only remedy text. They gate no check (`admit.rs:387-427`).
  - `admit(Md1)` plus `md1_refusals` is exactly the CLI's md1 route before `md1::build` (`as_flag.rs:104-115`).
  - The `md1::build` error branch is not mirrored. It is "unreachable", and `the_md1_route_derives_every_carried_address` runs `build` on every `md1_admits=true` row, so a false PASS there would still fail that test.
- **No false PASS** for the column found.
- **Caveat.** After the fold, no admitted row has a mismatched origin. So a regression that drops E3 is no longer caught by the seam file. It is caught by `descriptor_refusals::row_md1_origin_mismatch` (green), which was r2's scenario 2. See M-1 for the other half.

### 4. Is the normative text consistent with the file? Yes.

- SPEC_descriptor_input §7's population table (`:1913`, `accepted-extreme` 1) is unchanged.
- The `md1_admits` definition (`:1703-1711`) and the §5.3-citing clause (`:1934-1940`) all still hold.
- No row now refuses md1 for a non-§5.3 reason once it is admitted.
- The spec quotes no origin for this row.
- rows.py's `source` ("16 unhardened children of the dc567276 fixture key") is still true.

### db03395 and 8ad7ad7

- **db03395:** `design/FOLLOWUPS.md` gains 7 lines, a pure append at :21448 with no `-` lines. It adds the r2 report, byte-identical to the scratchpad original.
- **8ad7ad7:** FOLLOWUPS gains 98 lines, a pure append at :21455. It adds the seedhammer r1/r2 reports, each byte-identical to its scratchpad original.
- Neither touches code, tests, vectors or spec.

## Findings

### Minor

**M-1. The fold removed the seam's only host-admitted witness for "canonical keeps an xpub's own depth and child number when its origin disagrees" (`sh2-me-canonical-depth`). Nothing records the loss.**
- Where: `scripts/descriptor-seam-vectors/rows.py:276-281`; `crates/me-cli/testdata/descriptor_seam_vectors.json` row `accepted/sh-wsh-sortedmulti-16-keys`.
- `FIX_sec_canonical_keeps_xpub_depth.md:9-10, 187-191` made this row the cross-language pin for that fix. Its canonical was where me's and the fork's `xpubMeta` encoders had to agree on a depth-5 key under a 4-step origin.
- My scan (scratchpad `r3/scan.py`) found 8 explicit-origin mismatch rows before the fold and 7 after. All 7 remaining are `host_admits=false` and carry no `canonical`. `the_encoder_produces_every_canonical_the_file_carries` (`descriptor_seam.rs:1097`) checks only rows with a `canonical`.
- The shape is still live: `--as descriptor` packs a mis-originated multisig, and only md1 refuses it.
- What still covers it:
  - me-side unit tests (`cascade_tests.rs:698-730`);
  - `tests/cross_lang.rs:125` `descriptor_canonical_keeps_xpub_depth_cross_lang`, which is single-key `pkh` only and is skipped where the seedhammer submodule is absent.
- So a multisig-shaped divergence between me's canonical and the fork's would no longer fail the shared vectors.
- Fix: add a separate accepted row that keeps the old account-only origins, with `host_admits=true` and `md1_admits=false`. That needs the cited allowlist widened to `md1-origin-mismatch`, plus `POP` and spec count moves. Or file a FOLLOWUP (e.g. alongside F-436, "next vector-file byte change") and note the loss in the README re-pin paragraph.

### Nit

- **N-1.** `scripts/descriptor-seam-vectors/README.md:71-72`: "E3 added the refusal row `md1-origin-mismatch` to `refusal_rows.json` (sha256 `683ccd3d…20e5cd`)". The parenthetical reads as the sha of `refusal_rows.json`, but `683ccd3d…` is the sha of `descriptor_seam_vectors.json` (the old pin). Suggest "…; the vectors file's sha256 became `683ccd3d…`".
- **N-2.** cd269bc's subject ("carries true origins") and the rows.py comment (`rows.py:272-275`) say the origins are now true. That is right for path, depth and child number. But each key still cites a synthetic master fingerprint (`00000001`…`00000010`, one per key) for what is one account under one master. This predates the fold and is harmless to every assertion, but the wording slightly overstates it.

## Out of scope, noted only

- `scripts/fix-sec-canonical-keeps-xpub-depth-gate.sh:45` still pins the pre-E3 sha `71bba6f2…`. It went stale with E3, before this fold.
