# R0 round 2 — Plan A @ `60c5951`, fold verification by EXECUTION

Reviewer: opus. Executed all 9 tasks in a scratch crate; the user's tree was not
modified. Verdict: **1 Critical / 2 Important / 5 Minor / 4 Nit — BLOCKED.**

## The 15 round-1 folds landed, and were mutation-proved

Counts confirmed by execution: wire 8 · crypto 5 · passphrase 5 · record 7 ·
pubhash 5 · container 4 · uf2 2 · `seal_cli` 9 · **full suite 164 passed, 0
failed** · both clippy gates clean.

- **C1 is now a real assertion.** Copying the real twelve words into the UF2's
  padding (length unchanged) makes `seals_and_prints_the_passphrase_to_stderr_only`
  **FAIL**. Not cosmetic.
- C3, C4, I5 and the pubhash endpoint fix all mutation-proved dead.
- The gate is blocked by defects round 1 did not reach, **not by regressions**.

## CRITICAL — `decode_public_set` groups by HRP, refusing every multisig

Found independently of the concurrent §6.3 spec re-review, which reached the same
conclusion. Vector F's 12 public records (6 `mk1` = 3 cards, 6 `md1` = 1 card) →
`mk1: chunked-header malformed: received 6 chunks, header declares total_chunks
= 2`. So `me seal --plaintext` cannot seal a multisig public section, and
`me hash` cannot re-derive the §6.6 hash for any multisig wallet — **disabling
the downgrade control for exactly the wallets §6.4 exists to admit.**
Invisible because `decodes_a_complete_card_set`, D and E all use one card per
HRP, and F is `pub_len = 0`.

**Folded:** group by `(HRP, chunk_set_id)`, non-chunked records take the
single-string path, plus **vector G** (12 public records across six cards) and a
mutation row.

## IMPORTANT — the combined-cap test was a FALSE PASS

20 copies of one md1 chunk die in `check_public` with `chunk set incomplete: got
20 chunks, expected 3` long before the cap is reached, and the test asserted only
`.is_err()`. **Deleting the combined-cap check left all 52 lib tests green.**
Round 1's "combined 24-cap untested" Minor was not closed — it was replaced by a
test that passes for the wrong reason.

**Folded:** a public set that actually decodes (5 records) plus 20 secrets = 25,
matching `ContainerError::RecordCount(25)` specifically.

## IMPORTANT — `me seal` accepted and emitted an UPPERCASE BIP-39 mnemonic

`passphrase::is_valid` lowercases via `normalise` before parsing, but
`encode_section` emits verbatim. Executed: `me seal "BACON BACON …"` exits 0 and
the decrypted blob contains `BACON…`. §6.4's lowercase rule binds both sections
and §9 says refuse rather than emit. Only the mnemonic branch was affected —
uppercase `md1`/`mk1`/`ms1` were correctly refused.

**Folded:** lowercase check before the mnemonic branch.

## Minors / Nits folded
`run_seal_cli` trimmed argv before `encode_section`'s CR scan, so a
leading/trailing CR was normalised rather than refused (§9 says refuse) · Task 7
claimed 52 tests when `uf2.rs` arrives in Task 8 (50 at that point) · the
public-only `--iterations` guard had no killer · the zeroize fold was defeated by
its own next line, cloning the secret into a plain `Vec<String>` — now
`encode_section` is generic over `AsRef<str>` so no copy is made · a mnemonic
passed to `--plaintext` was refused with the `--group-size 0` message, which
misdiagnoses it · Task 4 Step 5's filter · a `cargo fmt` caveat: under nightly it
reformats pre-existing files the per-task `git add` lists do not stage, leaving a
dirty tree after every commit.

## M-5, confirming the spec re-review independently

"The md-codec bump is required, not optional" is **false**. Pinned back to
`=0.40.0`, **all 52 seal tests pass**, including `decodes_a_complete_card_set`,
`refuses_a_bch_valid_but_undecodable_record` and every vector sha256. Matches
`8fd30f9`'s finding that the "wire version 9" claim was a misread of
`decode_md1_string` applied to a chunked record.

**Folded:** the bump is retained only as routine currency, with the retracted
justification recorded in place.
