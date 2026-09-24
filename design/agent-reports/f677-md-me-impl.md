# F-677, the md and me items: implementation report

Implementer: Opus 5.5, single agent, 2026-09-24. Nothing pushed, tagged or merged. `design/FOLLOWUPS.md` not edited.

| repo | branch | base | fix commit | release commit (HEAD) |
|---|---|---|---|---|
| descriptor-mnemonic (`dm-worktrees/f677`) | `f677-shapekey` | `7b03a408` | `5aa374f5` | `1bc2331a` (md-codec 0.48.3, md-cli 0.20.3) |
| mnemonic-engrave (`me-worktrees/f677`) | `f677-me` | `1ed5dc08` | `d799e141` | `f4fbd7b1` (me 0.12.0) |

## 1. md: `shape-key --descriptor` refused wallet exports

### Reproduction (pre-fix binary, md-cli 0.20.2)

The input was Liana's own re-render of `preset-simple-timelocked-inheritance-wsh`, taken from `design/evidence/composer-fable-r0/fable-liana-parse-out.jsonl`. It comes in two variants of one wallet (same chain codes and points):

- `md` variant (parent fp `00000000`): keyed, exit 0.
- `real-xpub` variant (parent fps `1cf29716`, `ee71f8c5`): refused with `md: shape-key: the reconstruction does not round-trip chain 0: got …xpub6DXuQW1Q2JpZxsEn…, want …xpub6DkFAXWQ2dHxq2va…`.

### Where the defect is

The defect is in shape-key's self-check, not in `md descriptor`. `descriptor_route::descriptor_from_chains` re-renders its reconstruction and demands the input back byte for byte, xpubs included. A re-render always carries parent fingerprint `00000000`, because a card stores chain code and point only, and the parent fingerprint is hash160 of the parent point, which is not on the wire (F-611, already documented in `md descriptor --help`). `md descriptor` cannot render a wallet's real value. That value also takes no part in CKDpub, the script or the SkeletonKey.

### Fix

The fix is in `crates/md-codec/src/descriptor_route.rs`. A new private helper, `without_parent_fingerprints`, zeroes the parent fingerprint of each **origin-bearing** input xpub. When the input carried a checksum, it recomputes it (`from_str` has already verified the original). The self-check compares against this normalized form. Nothing else is relaxed:

- Depth and child number are still compared, because both are recoverable from the origin.
- Origin-less xpubs are untouched. Liana's unspendable internal key is recognised by full byte equality, and that recognition still applies.
- Text that does not parse comes back unchanged, so it fails the comparison exactly as before.
- `RouteError::RoundTrip.want` still reports the original input.

Doc updates: the module doc now names the one exempt field, and the `--descriptor` help says parent fingerprints are not compared.

### Corpus diff: the SkeletonKey does not change for any input that already worked

The corpus was every balanced multipath descriptor text containing an xpub or tpub under `mnemonic-engrave/design/evidence`, `dm/crates` and `dm/design`: 414 texts in total. Each was run through `md shape-key --descriptor` with the pre-fix and post-fix binaries.

- **274 keyed before; all 274 are byte-identical after** (stdout, stderr and exit code).
- 99 are newly keyed. 93 of their keys were already in the before key set. The other 6 are shapes with no zero-fingerprint twin in the corpus (wpkh, sortedmulti, simple tr).
- 41 are still refused. 36 are unchanged-class refusals: keys without an origin, parse errors on `…`-elided fixture text, and similar. The other 5 are round-trip refusals, all checked by decoding:
  - 3 carry `/2'` xpubs (child 0x80000002) under `/3'` origins.
  - 2 mix an origin-less tpub internal key with mainnet xpubs.

  These should still refuse.

### Tests

In `crates/md-cli/tests/cli_coordinator_verdict.rs` and `descriptor_route.rs`. Every mutation below was run.

- `shape_key_keys_a_wallet_export_like_its_card`: the Liana wsh and tr wallet exports key identically to their `md` forms and to the card minted from them. Red pre-fix.
- `md_descriptor_round_trips_through_shape_key`: card → `md descriptor` → `md shape-key --descriptor` gives the same key as `md shape-key <card>`. It runs over every keyable evidence descriptor plus both wallet exports. Red pre-fix.
- `shape_key_still_refuses_an_xpub_whose_depth_disagrees_with_its_origin`: red under the mutation "normalize depth too".
- `a_wallet_export_with_a_bad_chain1_checksum_is_still_refused` (unit test): red under the mutation "strip the checksum before parsing".

### Go port

No counterpart. The fork (`seedhammer` @ `0287e3a`) has no SkeletonKey or descriptor-text route. `gui/multisig_match.go` compares xpubs while ignoring parentFP and depth, but that is a different feature. Nothing was changed in the fork.

### Gate (toolchain 1.85.0, `CARGO_TARGET_DIR=dm-worktrees/f677-target`)

`cargo nextest run --locked --workspace --all-features`: 1573 passed, 4 skipped. `cargo doc -D warnings`, `cargo clippy --all-targets -D warnings` and `cargo fmt --check` all return 0. The gate was run on both the fix commit and the release commit.

### Release

md-codec 0.48.3 and md-cli 0.20.3 are patch releases: the public API is unchanged and more inputs are accepted. The release commit carries the exact pin `=0.48.3`, the Cargo.lock entries (these two only) and both CHANGELOG entries.

## 2. me: a digest-shaped hashlock phrase packed without ms's stop

### Where me takes a phrase

The only place is `me sysw pack --pack-preimage`, through a `phrase:<hex of "method,phrase">` record. It is hashed by `HashlockMethod::preimage`. The other `Phrase` sites are the argv guard and advisories, and none of them admits a record.

### Fix

- `sysw::admit_check` is the one admission point. It runs before the passphrase ceremony (F-246).
- It now applies ms-codec's own predicate, `ms_codec::hashlock::looks_like_digest`, to the phrase as the record carries it: no trim, no case fold. The rule is present at the pinned rev `e5dff6f`, and ms has not changed it since.
- A match is refused with a new error, `SyswError::PhraseLooksLikeDigest(index, width)`, unless the new field `Admission::phrase_looks_like_digest_ok` is set. The CLI flag is `--phrase-looks-like-digest-ok`, the same name as in ms.
- The message reuses ms-cli's sentences verbatim: "that phrase is N hex characters, the width of a digest.", "Hashing it commits the wallet to the ASCII of those characters, NOT to the digest they spell.", "If you already hold a DIGEST, it is finished", the `md compose` and `me sysw pack 'hash:…'` remedy lines, and "If you really meant this as a phrase, re-run with --phrase-looks-like-digest-ok."
- Only the `--hex` parenthetical is dropped, because `me` has no `--hex`.
- The message prints the width, never the phrase. The exit code is 4.
- Classification is unchanged.

### Tests

Five new tests in `crates/me-cli/tests/sysw_pack_preimage.rs`:

- The stop fires at 40 and 64 hex characters, in uppercase, and under both methods. The phrase is not echoed.
- The override admits the phrase.
- Widths ±1, a non-hex 64-character phrase and a 64-character phrase with a space all pack.
- The stop precedes the ceremony. The same run, confirmed with the flag, does reach the ceremony, so the ordering assertion is not vacuous.
- `admit_check` returns the new error with index and width.

Mutations, each run and each red:

- Drop the rule: 3 tests red. This is also the pre-fix behaviour.
- Ignore the override: 2 tests red.
- Use `len >= 40` in place of the kinds' widths: 1 test red.

### Gate (toolchain 1.85.0, `CARGO_TARGET_DIR=me-worktrees/f677-target`)

`cargo nextest run --locked --workspace`: 690 passed, 2 skipped. `cargo clippy --all-targets -D warnings` and `cargo fmt --check` both return 0. The gate was run on both commits.

### Release: 0.12.0, not 0.11.1

This follows the repo's own convention. 0.11.0 shipped "Newly refused" behaviour together with a "Breaking (library)" section for new enum variants. This change does both:

- A record that packed at exit 0 now exits 4 without the flag.
- `SyswError` gains a variant, and it is not `#[non_exhaustive]`.
- `Admission`, a struct with all-public fields, gains a field. A struct literal without `..Default::default()` no longer compiles.

## Concerns

1. A record produced by `ms hashlock --emit-record --phrase-looks-like-digest-ok` needs the flag a second time at `me sysw pack`. That is the same rule applied on a second surface. It is stated in the CHANGELOG.
2. The device already has the equivalent confirmation (`seedhammer gui/composer_copy.go` `composerCopyPhraseLooksLikeDigest`, `hashlock/hashlock.go`), so it is not a third gap. I did not check that its widths match.
3. A wallet export that `md shape-key --descriptor` still cannot take is a single-chain export (`/0/*` and `/1/*` as two descriptors, as older Core `listdescriptors` emits). That is the existing `NotMultipath` refusal, is outside F-677's wording, and I did not change it.
4. `me` still depends on md-codec 0.47.0 by git rev. The md fix does not reach `me`, and it does not need to.
