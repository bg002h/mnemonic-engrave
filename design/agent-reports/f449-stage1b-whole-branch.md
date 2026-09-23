# F-449 stage 1b — whole-branch adversarial review (final gate before merge)

**Branch:** `/scratch/code/shibboleth/dm-worktrees/f449-stage1b`, tip `482a07e6`, base `37367c1f`
**Reviewer:** independent whole-branch pass (opus), 2026-09-22
**Method:** construct-and-run. Every claim below was executed, not read off the source.
**Verdict: NOT READY — 1 Critical, 1 Important, 3 Minor, 2 Nit.**

Tree left clean (`git status --porcelain` empty). Nothing in the worktree was mutated.

---

## What I ran (search width)

| probe | result |
| --- | --- |
| full suite, `cargo nextest run --locked --all-features` | **1495 passed / 0 failed / 3 skipped** (the 3 are pre-existing `#[ignore]`: partition timing, BCH exhaustive sweep, bitcoind differential) |
| `cargo fmt --all --check`, `cargo clippy --all-targets --all-features --locked -D warnings` | both clean |
| **base-vs-tip CLI differential**, all 65 vendored vectors × 8 command shapes (`decode`, `inspect`, `descriptor --network mainnet`, `descriptor --network testnet`, `address` receive, `address` change, `decode --json`, `inspect --json`) — base binary built from `git archive 37367c1f` | 520 comparisons, **130 diffs, every one of them the single line `"schema": "md-cli/1"` → `"md-cli/2"`. Zero non-schema differences.** This is brief item 4 answered end-to-end rather than by reading the guard. |
| **full evidence journey, all 8 vendored Liana cases**: `md decompose` → template → `md decompose --emit commands` → `md encode` → v8 chunks → `md descriptor --network mainnet` | **8/8 reproduce Liana's own `descriptor_with_checksum` BYTE-FOR-BYTE, checksum included** — including `preset-decaying-multisig-tr`, the nested taptree §2 flagged as "right by reading `TapTreeIter` but not measured", and `X20-tr-hashlock-known` |
| **address equality vs Liana's recorded addresses**, the 4 ACCEPT shapes, receive 0..2 + change 0..2 | **24/24 MATCH** |
| placeholder-renumbering invariance (`@3,@2,@1,@0` out of order with keys permuted to match) | identical chunk bytes and identical descriptor — canonicalisation does not disturb §2's leaf order |
| kind-0 vs kind-1 twin of one tree | different `md1-encoding-id`, `wallet-descriptor-template-id`, `wallet-policy-id`, different addresses; `md verify` MISMATCHes across the pair (2511 vs 2512 bits) |
| old (v4-only) binary on a v8 card | refuses loudly and names the version, both single-string and chunked: `wire-format version mismatch: got 8, expected 4` |
| single-payload dispatch at v8 | a keyless kind-1 card is one string `md1gppqq…`; first symbol `g` = `0b01000` → divergent 0, version 8, chunked-flag 0 — exactly §3c's table row, exercised through the real CLI |
| BCH repair of a corrupted v8 chunk | corrected and byte-identical to the original |
| mixed-version chunk set (4 v8 + 4 v4) | refused: "chunks in the set disagree on version, chunk-set-id, or count" |
| §6 refusals through the CLI | row 1 (`sortedmulti_a`), row 2 (`<2;3>` use-site), row 4 (nested), row 6 (non-minimal version) all fire |
| corpus-test vacuity check | 23 root-`tr` vectors, **10 NUMS-rooted**, **1** skipped by the §6 row-1 tolerance → **9 vectors actually exercised** by the kind-1 corpus tests. The helper measures its own population; not vacuous. |
| adversarial marker placements (leaf, doubled, under `wsh`, bare, with a path suffix) | see I-1 |
| adversarial internal-key spellings through `md decompose` (`<2;3>`, `/0/*`, `<1;0>`, `<0;1;2>`, `/*h`, `/0`, `/<0;1>/5/*`, origin-prefixed) | see C-1 |

Everything the brief listed as already established was taken as given and not re-derived.

---

## C-1 (Critical) — `md decompose` silently normalises a recognised Liana internal key's OWN derivation path, minting plates for a different wallet with different addresses

**Site:** `crates/md-cli/src/decompose/walk.rs`, `liana_internal_key_match`.

The recogniser compares **only the extended key**:

```rust
let actual_xpub = match internal {
    DescriptorPublicKey::XPub(x) => x.xkey,
    DescriptorPublicKey::MultiXPub(x) => x.xkey,
    DescriptorPublicKey::Single(_) => return None,
};
…
if actual_xpub == recomputed { Some(internal.to_string()) } else { None }
```

`x.origin`, `x.derivation_path` / `x.derivation_paths` and `x.wildcard` are never
looked at. On a match, `decompose/mod.rs` `retain`s that occurrence **out** of the
slot set and `build_template` substitutes the fixed marker `UNSPENDABLE(liana)`,
whose semantics are hard-coded to `origin: None`, `<0;1>`, unhardened wildcard
(`to_miniscript.rs::build_liana_internal_key`). Everything the input said about
that key's derivation is discarded, and because the occurrence is removed before
`check_depth_consistency` / the origin-less note ever see it, the existing input
refusals are bypassed too.

**SPEC §2 step 6 defines those properties as part of the recipe's output**
("Render with `origin: None`, derivation paths `<0;1>`, unhardened wildcard"), so
a key at `<2;3>` does **not** match §2. The implementation narrowed "match §2" to
the xkey bytes alone.

### Reproduction (RUN, exit 0, no note, no warning)

Input: `preset-kofn-recovery-tr`'s own descriptor with the internal key moved to `<2;3>`:

```
tr(xpub661MyMwAqRbcFswVugWFBxmD7r3bQLsmHovc3p3wTFfgk3EWb36m3QfsezgaR6h5cXXgPG3R2XmctBn55sAt35wzLnrYy82sLKYF8CRsak7/<2;3>/*,{multi_a(2,…),and_v(v:pk(…),older(26280))})
```

```
$ md decompose "<that>" --emit template        # rc=0, no note, no warning
tr(UNSPENDABLE(liana),{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,…),and_v(v:pk(@3/…/<0;1>/*),older(26280))})
$ md decompose "<that>" --emit commands        # rc=0
$ <run the emitted route-1 command>            # rc=0, 8 md1 chunks
$ md address <chunks> --network mainnet --index 0
bc1pj6davmeetfe2uutrjlgxcjytq50ggvyd82xtxx2eazm24hsdhlnqxvh4v4
```

That address is the **`<0;1>` wallet's** address — it is byte-identical to
`cases.json`'s `liana_receive[0]` for the *canonical* descriptor. The input
descriptor described the `<2;3>` wallet. Those are different wallets: BIP-32
CKDpub over that xpub gives internal key `03d0626c4c…` at `0/0` and
`02d4075001…` at `2/0` (computed independently in pure secp256k1 arithmetic), and
the taproot output key is that point tweaked by the same tap-tree commitment, so
the output keys — and the addresses — differ.

**One invocation of `md decompose` emits two mutually contradictory answers:**

```
$ md decompose "<that>" --emit descriptor
tr(xpub661MyMwAqRbc…/<2;3>/*,{…})       # the input wallet
$ md decompose "<that>" --emit template
tr(UNSPENDABLE(liana),{…})              # a DIFFERENT wallet
```

`md verify` cannot catch it: template and card are both derived from the
already-normalised template.

### It is a regression, not pre-existing

Base binary (`37367c1f`) on the identical input:

```
$ md decompose "<that>" --emit template
tr(@0/<2;3>/*,{multi_a(2,@1/…,…)})                       # path PRESERVED
note: 1 key(s) state NO origin … `--emit commands` refuses.
$ md decompose "<that>" --emit commands ; echo $?
1                                                          # REFUSED
```

So the branch turns a preserved-and-refused input into a silently-rewritten,
mintable one.

### Every non-canonical spelling is swallowed

Measured, all `rc=0` and all recognised:

| internal key as written | recognised as kind 1? | consequence |
| --- | --- | --- |
| `…/<2;3>/*` | yes | different wallet |
| `…/<1;0>/*` | yes | receive/change swapped |
| `…/<0;1;2>/*` | yes | third alternative dropped |
| `…/0/*` | yes | single-path silently made multipath |
| `…/<0;1>/*h` | yes | hardened wildcard silently made unhardened |
| `…/<0;1>/5/*` | yes | extra level dropped |
| `…/0` (no wildcard at all) | yes | a fixed key silently made a wildcard key |
| `[deadbeef/48'/0'/9'/2']…/<0;1>/*` | yes | origin silently dropped; the base binary **refused** this with a depth-inconsistency error |

### Why per-task review could not see it

Task 6 built the recogniser; Task 7 built §6's refusals. §6 row 2 exists for
exactly this hazard ("Liana pairs multipath alternatives positionally…") but it
checks `d.use_site_path` and `d.tlv.use_site_path_overrides` — the **leaves'**
use-site. Per §5 the kind-1 internal key takes **no slot and no use-site entry**,
so the encode-side refusal is structurally incapable of seeing the internal key's
own path. The only place that information exists is the decompose recogniser, and
it discards it. Neither task's brief covered the seam.

### Fix (small, and it keeps all 8 evidence cases working)

Require §2's full output shape before matching: `MultiXPub` **and**
`origin.is_none()` **and** `derivation_paths == [m/0, m/1]` **and**
`wildcard == Wildcard::Unhardened`. Anything else falls through to §4a's stated
fallback — "Otherwise → today's annotated slot, unchanged" — which is the base
binary's safe behaviour. All eight vendored evidence descriptors carry
`/<0;1>/*` with no origin, so they still match (verified: all 8 parse as the
multipath form). Add the `<2;3>` / origin-prefixed inputs as negative vectors.

---

## I-1 (Important) — `UNSPENDABLE(liana)` outside the internal-key position leaks `internal: synthetic key …`, the exact string SPEC §4a forbids

**Site:** `crates/md-cli/src/parse/template.rs`, `substitute_synthetic` —
`desugared.replace(LIANA_UNSPENDABLE_MARKER, liana_synthetic_internal_key_hex())`
is an unanchored global string replace. `walk_tr` recognises the reserved hex back
only in the `tr()` internal-key position; anywhere else the key-map lookup misses
and the internal-error message escapes.

```
$ md encode "tr(@0/<0;1>/*,{pk(UNSPENDABLE(liana)),pk(@1/<0;1>/*)})" --key @0=xpub… --key @1=xpub…
md: template parse error: internal: synthetic key fa1446b119da8e010be1a88d3340f68fe4f21c439270c652c5434d04d3e92c98 not found in key map (rendered: fa1446b1…)     # rc=1
```

Same for the marker in both positions at once.

SPEC §4a, first bullet: *"`md encode` on a literal xpub in the internal-key
position fails with an **internal error** leaking to the user — `"internal:
synthetic key … not found in key map"` (RUN). **Whatever the scope, that string
must never reach a user.**"* The branch closed that for the internal-key position
(a good, clear refusal naming the marker) and, in the same task, opened a new
route to the identical string — one the user reaches by mis-placing a marker this
stage itself invented, and whose message names a 64-char hex constant that appears
nowhere in their input.

**Regression, not pre-existing:** at base `37367c1f` the nearest analogue
(a literal x-only hex in a leaf) gives a clean
`miniscript parse failed: malformed public key`. `grep` confirms **no test places
the marker anywhere but the internal-key position**.

`design/FOLLOWUPS.md:7098-7112` already records this message class as the repo's
own worked example of why a confusing error message is not cosmetic.

**Fix:** refuse the marker outside the `tr()` internal-key position with a message
naming its only legal position — either by anchoring the substitution, or by
detecting a surviving `liana_synthetic_internal_key_hex()` in `walk_tap_tree` /
`lookup_key`'s fallback and mapping it to that refusal.

---

## Minor

**M-1 — `md descriptor --network testnet|signet|regtest` on a kind-1 card emits a mixed-network descriptor.**
`derive.rs:57` hardcodes `network: NetworkKind::Main` for every leaf key rebuilt
from the `Pubkeys` TLV, so leaves always render `xpub`. This branch makes the
kind-1 internal key the only key in the output that follows `--network`:

```
$ md descriptor <kind-1 card> --network testnet
tr(tpubD6NzVbkrYhZ4Xg8MMsVLPt6aSy9FPjNpqSWuQN75oARaVKuDfFwrBrLhu2mMwbeKQS4azqZHZdbfaNGgUj27vXQHQPbreeh6vH8etAiKLXC/<0;1>/*,{multi_a(2,[73c5da0a/…]xpub…
```

1 `tpub` + 4 `xpub` in one descriptor. `md descriptor --help` documents
`--network` as *"Network for xpub validation"*, and at v4 it changes the rendered
descriptor for **none** of the 65 vendored vectors (measured). **No address
divergence** — the version bytes are cosmetic, and `md address --network testnet`
returns `tb1p…` with the same witness program as the `bc1p…` form (verified).
Record it; md has no correct testnet rendering for any wallet today.

**M-2 — the branch vendors Liana's own descriptors and 24 addresses and no test compares any of them.**
`cases.json` carries `descriptor_with_checksum`, `liana_receive[3]` and
`liana_change[3]` for 8 shapes. `expected_xpub` is asserted (§8.1, all 8);
`descriptor_with_checksum` is read only as *input* to `md decompose`;
`liana_receive`/`liana_change` are read by **nothing** (`grep`: only the struct
fields and two comments saying so). Stage 1b's §8.2 leg is deliberately md-vs-md
("the C3 ruling"), and §9 assigns the evidence legs to stages 2 and 4 — so this is
spec-compliant, not a gate violation. Recorded because it is the single most
funds-relevant property of the stage and it is one assertion away with data
already committed. **I executed it by hand: 8/8 descriptors byte-exact including
checksum, 24/24 addresses match.** Risk is regression-only between now and
stage 4.

**M-3 — three items §9 assigns to stage 2 shipped in 1b.**
`md descriptor` kind 1 (`cmd/descriptor.rs`), the `UNSPENDABLE(liana)` template
substitution rule, and the JSON schema version bump are stage 2's content column
in SPEC §9; the plan assigns them to Tasks 5/6 and they are in this branch. The
effect is that they cross ahead of §8.2's CLI gate and §8.8's **REQUIRED** live
`harnesses/liana` install run. Not a correctness problem — I ran §8.2's CLI leg
and §8.3's md↔evidence address leg by hand (above) — but stage 2's brief must not
now treat §8.8 as already discharged.

## Nit

**N-1** — `liana_synthetic_internal_key_hex()` is a reserved magic constant that
`walk_tr` also accepts **literally**: writing that exact 64-char hex as a `tr()`
internal key mints kind 1. Cryptographically unreachable by accident and no funds
path, but it is an undocumented second spelling of the marker with no test.

**N-2** — `a_tpub_wallet_derives_a_tpub_internal_key` /
`a_tpub_wallet_renders_a_tpub_internal_key` are named for "a tpub wallet" but pass
`Network::Testnet` over mainnet leaf keys; md cannot represent a tpub wallet's
leaves at all (M-1). The doc comments are honest about this; the names overclaim.

---

## What I looked for and did NOT find (stated so the negative has a scope)

- **No cross-task invalidation in the codec.** `Descriptor::wire_version()`
  reaches every node that can hold a `Body::Tr` (`Children`, `Variable`, `Tr`;
  `MultiKeys` holds no nodes), and it is the single source of the version at all
  five write sites (`encode.rs`, `chunk::split`, both `identity.rs` hashes, and
  `write_node`'s own parameter). Encode/decode of the kind bit are symmetric and
  golden-pinned.
- **No v4 drift anywhere reachable from the CLI** — 520 base-vs-tip comparisons,
  only the deliberate schema string moved.
- **No kind-0/kind-1 confusion surface.** Template text, all three ids, the
  12-word phrase, the mk1 stub top-4, `md verify`, `policy_shape`'s
  `KeyPathKind`, `SkeletonKey`'s label and `--json`'s `unspendable_kind` all
  separate them; `md bytecode` emits no bit annotation that could mislabel a v8
  stream.
- **No false-positive route in the recogniser other than C-1's.** The match is
  full `Xpub` struct equality against a recomputed value, so a descriptor that
  matches re-renders to itself; the near-miss vector covers the
  pattern-match weakening.
- **No vacuous tests from Task 7's refusals.** The two corpus tests tolerate only
  `UnspendableWithSortedMultiA`, panic on any other refusal, and assert a non-zero
  exercised count; measured 9 of 10 NUMS-rooted vectors actually run.
- **`md repair`'s discarded-correction defect (§6a row 2) is real but is stage 2's.**
  Reproduced on the base binary (`rc=2`, correction thrown away); the tip binary
  repairs a corrupted v8 card exactly. Out of 1b scope by §9.
- **`verify.rs:62`'s `Admission::Enforce` re-encode (F-639)** stays unreachable for
  the four §6 rules — a minted card has already passed them.
- Secret-handling: nothing found; none would have gated anyway.

## Merge recommendation

**NOT READY.** C-1 is a reachable, silent, funds-relevant wrong result that the
branch *introduced* by retiring a refusal the base binary raised, and its fix is a
tightening of one predicate plus negative vectors. I-1 is a one-site refusal gap
against an explicit spec sentence. Everything else recorded above is Minor or Nit
and does not hold the gate. Re-review scope after the fold should be exactly
"does `liana_internal_key_match` now require §2's full output shape, do all 8
evidence cases still match, and does the marker refuse outside the internal-key
position" — not a fresh audit.
