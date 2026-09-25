# F-677 md + me branches: adversarial review

Reviewer: Opus 5.5, single agent, 2026-09-24. Scope is the brief's one question: do the two branches do what they claim without weakening a funds check, and can each new test fail? I did not audit anything else.

| repo | range reviewed | HEAD at review |
|---|---|---|
| descriptor-mnemonic `f677-shapekey` | `7b03a408..1bc2331a` | `1bc2331aeff4640ebe9920b048828db88db5e612` |
| mnemonic-engrave `f677-me` | `1ed5dc08..f4fbd7b1` | `f4fbd7b1a5d398106f506b5a916ecef56b8494db` |

Both worktrees reported a clean `git status` after all the mutations were restored. Nothing was committed, pushed or edited.

**Verdict: 0 Critical, 0 Important, 2 Minor, 3 Nit.**

## Gates I re-ran

All gates used toolchain 1.85.0 with its bin directory first on PATH. That gives `clippy 0.1.85 (4d91de4e48)`, the pinned clippy rather than the system one. `CARGO_TARGET_DIR` was under `review-target-f677/`.

| repo | nextest `--locked --workspace` | clippy `--all-targets -D warnings` | fmt `--check` | other |
|---|---|---|---|---|
| dm (`--all-features`) | 1573 passed, 4 skipped | exit 0 | exit 0 | `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps --document-private-items --all-features`: exit 0 |
| me | 690 passed, 2 skipped | exit 0 | exit 0 | — |

Both match the implementer's counts.

## md: `without_parent_fingerprints`, attacked

The one question was whether the normalization accepts an input that differs from the reconstruction in anything that matters. I found no such input.

Structural reason:

- Key material (chain code and point), origin fingerprint and origin path all come from **chain 0 parsed**.
- `got` is rendered from those values. The comparison is still exact text equality against `want`.
- The normalization only rewrites the 4 parent-fingerprint bytes of origin-bearing xpubs, via `Xpub::to_string()` substring replacement. A base58check xpub is a fixed 111-character canonical string, so it cannot match inside another key.
- The checksum is recomputed only after `from_str` has verified the original.

### Probes

These ran as a temporary integration test calling `descriptor_from_text`, `skeleton_key_of_text` and `descriptor_from_chains` directly. The test was deleted afterwards.

| probe | result |
|---|---|
| Chain 1 with **different key material** than chain 0, same metadata, via the pub `descriptor_from_chains` | refused |
| Chain 1 with a **different chain code** only | refused |
| Chain 1 that differs from chain 0 **only in parent fp** | accepted, n=1 (harmless: the field is not in CKDpub) |
| **Child number** disagreeing with the origin (84' origin, 85' child) | refused (`round-trip chain 0`) |
| **Depth** disagreeing | refused (the implementer's test) |
| **Mixed network** (a tpub beside an xpub), real or zero fps | refused both ways |
| **Origin-less** xpub | refused (`carries no [fingerprint/path] origin`) |
| **Bad chain-1 checksum** | refused (the implementer's unit test; confirmed by mutation M3) |
| Key reuse distinguished only by parent fp (same key twice, fps 1111/2222) | accepted, n=2. **Not new**: the zero/zero form of the same text was already accepted pre-fix with the identical result. |

### Corpus: re-run, not trusted

I built the pre-fix md-cli 0.20.2 from `git archive 7b03a408` and the post-fix 0.20.3 from the branch. My own extractor covered every balanced `wsh|wpkh|tr|sh|pkh(...)` text containing `<0;1>` and an xpub or tpub, under the same three roots. It reproduced the implementer's numbers exactly:

- 414 texts.
- 274 keyed before, and **0 of those changed** (stdout, stderr and exit code all compared).
- 99 newly keyed, 93 of them into the before-keyset.
- 41 still refused, 5 of them as round-trip refusals (the same 5 shapes the report names).

Stronger check on the 99 newly keyed texts: for **every one**, post-fix `md shape-key --descriptor <text>` equals pre-fix `md shape-key --descriptor <text with every origin-bearing xpub's parent fp zeroed>`, byte for byte (99/99). So the fix is exactly "treat the parent fp as zero" and nothing more.

## me: digest-shaped phrase stop, parity with `ms hashlock`

**The predicate is shared.** me calls `ms_codec::hashlock::looks_like_digest` itself. The function is byte-identical at the pinned rev `e5dff6f` and at mnemonic-secret HEAD `a6ce215`. The only ms commit since touches the hashlock.rs `md compose` fragment, not the rule.

**The bytes checked are the bytes hashed**, on both sides:

- ms checks `bytes` after `validate_phrase`, then hashes `bytes`.
- me checks `p.phrase` from `composer_records::parse`. `preimage_of` (main.rs:2930) hashes that same `p.phrase`, and `Class::Phrase` is assigned only when that same parse returns `Ok(Phrase)`.

**The flag has the same name and semantics** in both tools: `--phrase-looks-like-digest-ok`, a confirmation and not a wall.

**Measured parity table.** ms-cli was built at HEAD and fed through `--hashlock-phrase-stdin`. me was the branch binary (`sysw pack --no-passphrase --pack-preimage`, a `phrase:hardened,…` record). The "stop" column means stderr contains "width of a digest".

| phrase | ms stop | me stop |
|---|---|---|
| 64 hex lower / 40 hex / 64 UPPER / 64 mixed case / 40 zeros | yes (rc 1) | yes (rc 4) |
| 63, 65, 39, 41, 32, 48, 56, 80 hex; leading space + 64; 64 + trailing space; `0x`+62; 63+`g`; the anchor phrase | no (rc 0) | no (rc 0) |
| 128 hex | no (rc 1, over the 100-character cap) | no (rc 4, same cap) |

19 of 19 rows agree. With the override, 64 and 40 both pass in both tools. A record from `ms hashlock --phrase-looks-like-digest-ok --emit-record` stops in me without the flag (rc 4) and packs with it (rc 0), as the CHANGELOG states. The refusal exits 4 and writes no output file.

**All admission goes through `admit_check`.** `split` calls it. The CLI calls it before the ceremony (main.rs:1682). No other path hashes a `phrase:` record into a payload.

## Mutations

I ran every mutation below. Each was applied by exact-match replacement, with the match count asserted to be 1, then the tests ran and the file was restored. `git status` was clean after each.

| # | mutation | result |
|---|---|---|
| M1 | dm: `got != want` (revert the fix) | **red**: `shape_key_keys_a_wallet_export_like_its_card`, `md_descriptor_round_trips_through_shape_key`, `a_wallet_export_with_a_bad_chain1_checksum_is_still_refused` (its positive half) |
| M2 | dm: also set `depth` to the origin length | **red**: `shape_key_still_refuses_an_xpub_whose_depth_disagrees_with_its_origin` |
| M3 | dm: parse `text` with the `#checksum` stripped | **red**: `a_wallet_export_with_a_bad_chain1_checksum_is_still_refused` |
| M5 | dm: keep the original checksum (no recompute) | **red**: the same 3 as M1 |
| M4 | dm: drop `x.origin.is_some() &&` | **survives the full suite (1573/1573)**, see N1 |
| M6 | dm: also set `child_number` to the origin's last step | **survives the full suite (1573/1573)**, see m1 |
| ME1 | me: drop the `PhraseLooksLikeDigest` return | **red**: 3 tests (`a_digest_shaped_phrase_warns_and_stops`, `the_stop_precedes_the_passphrase_ceremony`, `admit_check_names_the_record_and_the_width`) |
| ME2 | me: ignore `phrase_looks_like_digest_ok` | **red**: 3 tests (`the_override_admits…`, `the_stop_precedes…`, `admit_check_names…`). The report said 2; see N3. |
| ME3 | me: `len >= 40 && all hex` | **red**: `only_the_digest_widths_stop` |
| ME4 | me: lowercase-only hex at the digest widths | **red**: `a_digest_shaped_phrase_warns_and_stops` (the uppercase row) |
| ME5 | me: report the width as a constant 64 | **red**: `admit_check_names_the_record_and_the_width`, `a_digest_shaped_phrase_warns_and_stops` |

In every red case the failing assertion depends on the mutated line's output, which shows the line ran. The ordering test is not vacuous: its confirmed half asserts that the ceremony does appear.

## Findings

### Critical

None.

### Important

None.

### Minor

**m1: dm, the "child number is still compared" claim has no test.** The CHANGELOG, the module doc and the helper's doc all say depth and child number are still compared.

- That is true today. My probe with an 85' child under an 84' origin is refused.
- But normalizing `child_number` in `without_parent_fingerprints` (M6) leaves all 1573 tests green.
- It is not a funds defect, because child number does not enter CKDpub, the script or the key. It is a documented guarantee that nothing pins.
- Fix: a sibling of `shape_key_still_refuses_an_xpub_whose_depth_disagrees_with_its_origin` with `x.child_number` changed.

Reproduction: apply M6 (`child_number: x.origin.as_ref().unwrap().1.into_iter().last().copied().unwrap_or(x.xkey.child_number),` after `parent_fingerprint: Default::default(),`), then `cargo nextest run --locked --workspace --all-features`. Result: 1573 passed.

**m2: dm, the corpus sentence in the CHANGELOG has no committed reproduction.** The md-codec 0.48.3 entry says: "Over 414 corpus descriptors, all 274 that keyed before give byte-identical keys, and 99 wallet-exported texts key where they were refused."

- The numbers are **true**: I reproduced 414/274/0/99 independently, and the 99 match pre-fix-on-zeroed-text 99/99.
- But the extractor and the two-binary diff exist only in the implementer's session and in mine.
- The durable part of the claim is already a test: `md_descriptor_round_trips_through_shape_key` covers the keyable evidence rows.
- Either drop the counts from the CHANGELOG or record the method. Not blocking.

### Nit

**N1: dm, the `x.origin.is_some()` guard is semantically inert (M4 survives).** An origin-less key reaches the comparison only as Liana's recognised internal key. That key is recognised by full `Xpub` equality with a recipe whose parent fp is already `0` (nums.rs:64). Any other origin-less key is refused earlier with `KeyWithoutOrigin`. The guard is harmless defence in depth; the doc sentence "an origin-less xpub … is untouched" is true but untestable through this route.

**N2: dm, the depth-1 consistency case is now silently accepted.** At depth 1 the parent fp *is* the master fingerprint, so `[f/84']xpub…` with a parent fp ≠ `f` is internally inconsistent evidence that the origin fp is wrong. Post-fix this keys (probe C1).

- Pre-fix, every real-fp export was refused, including the consistent ones, and a wrong origin fp with a zero parent fp was accepted anyway (probe H0).
- So nothing that used to be caught is lost, and the origin fp is not verifiable in general. Recorded only because it is the one place the input could reveal a wrong origin fp.

**N3: implementer report, ME2 undercount.** The report says "Ignore the override: 2 tests red". Measured, it is 3: `admit_check_names_the_record_and_the_width` also reds. The report under-reports test strength, so the error goes the harmless way.

(Observed, pre-existing, not this diff: the me refusal is preceded by the H6 §3.3 "bearer material" and "no `hash:` record" warnings, because main.rs deliberately prints those before `admit_check`. That affects every admission refusal, not just the new one.)

## Release hygiene

- dm: md-codec 0.48.2 → 0.48.3 and md-cli 0.20.2 → 0.20.3. md-cli pins `=0.48.3`. Cargo.lock changes only those two version lines. Both CHANGELOG entries match the measured behaviour. `git grep` finds no stale `0.20.2`/`0.48.2` outside the CHANGELOG, the lock and design/ (only vendored bitcoin doc URLs).
- me: 0.11.0 → 0.12.0. Cargo.lock changes only that line. The CHANGELOG's "Breaking (library)" entry is accurate: a new `SyswError` variant, and a new `Admission` field on a struct with public fields. The minor bump follows the repo's 0.11.0 precedent. No stale `0.11.0` references.

## Housekeeping

The brief asked me to delete `/scratch/code/shibboleth/review-target-f677/` (14 GB), but the `block-rm-rf` hook refused `rm -rf`. It is still on disk, and the controller or operator needs to remove it.

ready to ship: yes
