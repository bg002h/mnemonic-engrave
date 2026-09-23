# F-449 stage 3 — whole-branch adversarial review (post-implementation)

Reviewer: independent opus agent, 2026-09-23. Scope: dm `cf35d61a..430ea478`, fork `7b6f2fb..43294c6` (5 commits), engrave `6d4c2a6d..607a9429`. All measurements were run in scratch copies under `/scratch/code/shibboleth/.tmp/f449rev/` (`git archive` of both fork commits, md-cli built from the dm worktree into its own `CARGO_TARGET_DIR`). All three worktrees were left clean (0 porcelain lines each, HEADs unchanged).

**Verdict: GREEN, 0 Critical / 0 Important / 3 Minor / 2 Nit.** I tried to build every failure the brief named: a wrong address, a wrong id, a Liana key read as NUMS or as a slot, a v4 card the old firmware read and this one refuses, a Go/Rust split on v8, and a false-PASS gate. None of them came out.

## Findings

### Critical
None.

### Important
None.

### Minor

**m1. Neither reader cross-checks the chunk-header version against the payload-header version.** This is pre-existing, and Go matches Rust here. I forged chunk sets with Go's own writers at the canonical csid:
- chunk header 4 with a v8 payload carrying a kind-1 tree: 32 sets;
- chunk header 8 with a v4 payload: 132 sets.

Rust `md inspect` and Go `Reassemble` **both accept all 164 sets, with identical `md1_encoding_id`, `wallet_policy_id` and `wallet_descriptor_template_id`**. The tree is read at the *payload* header's version on both sides (`md/md.go` `readNode(r, kiw, h.version)`, Rust `decode_payload`), so no kind is misread. The only effect is that a set whose chunk header says 4 still reads as a kind-1 wallet. This is not a Go defect. If it is ever tightened, the change goes into Rust first (Rust-primary rule). Owning phase: none, so it batches to the end.

**m2. Neither reader enforces SPEC §6 row 6 (minimum wire version).** Rust calls `validate_minimal_wire_version` only in `encode_payload_inner` (`encode.rs:252`), and decode never calls it. Go has no equivalent check. Forged non-minimal cards (a v4 tree written at header 8, with a kind bit of 0 on NUMS roots): **48 single cards and 132 chunk sets are accepted by both readers, with all three ids equal between Go and Rust.** Those ids equal the canonical v4 twin's, because both re-encode at the derived version. The two readers agree and nothing wrong is shown, so this is recorded rather than blocking. It is the read-side twin of F-654's mint refusals.

**m3. `errors.As(err, &*md.WireVersionError)` under TinyGo is exercised only by host tests.** `gui/md1_version.go:13-18` is the only route to the §6a message on all three surfaces. The firmware builds (the implementer measured +2,896 B), but no host test runs the TinyGo runtime. The stage-5 device walk should include one v12 plate scan. Owning phase: stage 5.

### Nit

**n1.** `policyIDHeader` computes a `Policy id:` for kind-1 sets. It did so for 2 of 2 keyed kind-1 sets called directly, with values equal to Rust's `cb913ed2…` and `3bebff4b…`. It is suppressed on screen only because `complexAddressSource` refuses kind 1 first (`gui/md1_gather.go:220`). I measured it absent in every kind-1 `gatheredDescriptorFlow` capture. When stage 4 enables addresses the line appears automatically, which is the intent. It is noted so stage 4 does not rediscover it.

**n2.** Pre-existing: `DecodeChunks`/`ExpandWalletPolicyChunks` refuse single-string cards (`chunk header chunked-flag not set`). So a single v8 card such as `liana_taproot` reaches the display through `md.Decode` only, never through the ids or address path. The behaviour is unchanged from `7b6f2fb`.

## What I verified sound, by measurement

**Lens 1: regression on existing cards.** I collected every `md1…` string in both trees' `md/`, `gui/`, `cmd/` and `testdata/`: 588 groups, 80 of them chunk sets, grouped by csid. Each group went through two harnesses built identically at 7b6f2fb and at 43294c6.

The **md harness** compared these outputs:

`Decode`, `ParseChunkHeader`, `Reassemble`, `computeEncodingID`, `WalletPolicyId`, `WalletDescriptorTemplateId`, `FormAwareStub`, re-`split`, `DecodeChunks`, `ExpandWalletPolicyChunks`, `PolicyShapeChunks`, `TapLeavesChunks`, `DuplicateKeySlotChunks`, `FormAwareIdChunks`, `WalletPolicyIdChunks`, `TapTreeDepthChunks`, `StripToTemplate`, `TemplateEngraveShapeGuardChunks`, `FormAwareStubChunks`.

The **gui harness** compared these outputs:
- `policyAddressAt`: receive and change addresses 0..2;
- `policyIDHeader`;
- bundle `offer` statuses;
- `gatheredDescriptorFlow` screen text over three input steps;
- the single-card `mdmkFlow` Inspect screens.

Results:
- **0 differences on any v4 card** outside the renamed error text. 84 groups decode, and 60 derive addresses; every address is byte-identical between the two builds.
- The only differences are the ones the plan authorised:
  1. Every v8 group is now read: 19 md groups, 2 keyed sets reach `addrOK=false`.
  2. The error text changed from `md: wire version mismatch` to `md: wire version N is not one this build reads (accepted: 4, 8)`.
  3. On screen, single cards at versions 0 and 10 now show "This firmware cannot read md1 version N." where they used to show "Can't decode this descriptor." or "Captured 0 of 0".
  4. Bundle status for those cards is now `bundleUnsupportedMD1Version`, not `bundleDropped`.

No card that 7b6f2fb read is refused by 43294c6.

**Lens 2: cross-language agreement on v8.** md-cli 0.19.0 was built from dm 430ea478. I generated a 288-case matrix: 9 tree shapes × both kinds × shared/divergent origins × keyed/keyless × auto/`--force-chunked` × three `--path` values. The shapes include `multi_a`, a pk pair, kofn-recovery, a depth-4 chain, a balanced 4-leaf tree, a hashlock, reordered indices, 6 keys, and `sortedmulti_a`.
- **Encoded:** 204 cases.
  - Refused at encode: 72 by Rust's same-origin rule and 12 by the `sortedmulti_a` kind-1 rule. Both refusals are expected.
- **Read by both:** 136 cases, 49 singles and 87 chunk sets.
  - `md1_encoding_id`, `wallet_policy_id` and `wallet_descriptor_template_id` are equal on all 136.
  - Go's re-`split` is byte-identical to Rust's chunks on all 87 sets.
  - The internal-key kind is correct on all 94 checkable cases (kind 1 is `InternalKeyLianaUnspendable`, kind 0 is `InternalKeyNUMS`).
- **Refused by Go:** the other 68, all `missing explicit origin`, while Rust partial-decodes them with exit 4. This is the same pre-existing origin rule on both sides.
- **Addresses:**
  - Kind 0: Go's addresses equal `md address` on 18 of 18 keyed sets.
  - Kind 1: Go refuses the address on 16 of 16 keyed sets, and no kind-1 screen capture contains `bc1` or `Policy id`.
- **Non-minimal and forged inputs:** 440 cards in total. 164 are m1's mismatched-header sets; 180 are m2's v4 trees at v8 (48 singles, 132 sets); 96 have a header-4 payload carrying a kind-1 tree or a v8 tree with no kind bit. Go and Rust agree on accept or refuse for **all 440**, and on all three ids wherever both accept.

**Lens 3: mint refusals reaching decode.**
- On the path `Reassemble → computeEncodingID → encodePayload`, the branch added only `version := dc.wireVersion()` and a `writeNode` call that takes it. `errLianaNeedsVersion8` cannot be reached from any caller, because every `writeNode` call site passes the version derived from the same tree. The call sites are `encode.go:469`, `template_id.go:53` and `walletpolicyid.go:42`, and `needsV8` covers `trBody`, `childrenBody` and `variableBody`.
- `canonicalize` keeps `ik` (`canonicalize.go:146`, `:323`).
- I built flip-to-kind-1 cards with Go's own encoder, including `sortedmulti_a` trees (90 cards). Rust and Go **agree on all 90**, with equal ids.
- `StripToTemplate` keeps kind 1 or kind 0 on 34 of 34 keyed sets, re-read by Rust.
- **I could not construct a card that Rust reads and Go refuses.**

**Lens 4: false passes.** I re-applied three mutations in the scratch copy and restored each one with `cp` then `touch`; each restore was green.
- **X2** (the kind bit is also read for a Slot key at v8): reds `TestKindBitPolarityIsPinnedOnTheWire` only. The corpus has no Slot-rooted v8 card, so that test is the only guard.
- **G6** (`case md.InternalKeyNUMS, md.InternalKeyLianaUnspendable:`): reds `TestEveryKeyedVectorReachesAnAddress` on both kind-1 vectors ("now derives via the complex route").
- **M2a** (identity writers pass `wfRedesignVersion`): reds `TestKeyedConformanceAgreesWithRust`, `TestKind0AndKind1TwinsNeverShareAnIdentity` and `FuzzWalletPolicyId`, as the plan says.
- The conformance gate **asserts** `md1_encoding_id` (`md/conformance_keyed_test.go:125-131`, `got != rec.Md1EncodingID` → `t.Errorf`); it does not only parse it.
- The Rust read-side goldens decode correctly by hand: `[06 00]` is tag 1, nums 1, kind 0, no tree; `[07 00]` has kind 1; `[04 80]` is Slot(1) at kiw 2.

**Lens 5: the §6a surfaces.**
- **Inspect, single card:** at versions 0 and 10, "This firmware cannot read md1 version N." on screen, measured.
- **Inspect, chunked card:** the gather flow's first chunk (`md1_gather.go:100`).
- **Gather, later chunks:** `md1_gather.go:136`.
- **Bundle:** the chunked card via `classify → clsDrop → default → md1StringVersionRefusal`, and the single card via `offerStandaloneMD1` (`bundle.go:244`, `:262`).
- None of these surfaces says "Not an md1" for a well-formed card at a version this firmware does not read. The four `md1_version_test.go` tests cover v12 on all three surfaces, and G1–G5 red them.

**Lens 6: provenance and landing.**
- **Vendored bytes:** all 260 pinned files in `md/testdata/vectors` at 43294c6 are **byte-identical** to `430ea478:crates/md-codec/tests/vectors/`, and every sha256 matches the pin. The 46 unpinned files present in both repos are also equal. `liana_cases.json` equals dm `430ea478:crates/md-codec/tests/fixtures/liana/cases.json` (sha256 `2a7c20e0…`).
- **`md/bits.go`:** the tag `descriptor-mnemonic-md-cli-v0.19.0` resolves to `cf35d61a`, which is md-codec 0.47.0.
- **Is the 430ea478 pin stable?** dm main is still `cf35d61a`, the branch base. That means:
  - A `--no-ff` merge keeps 430ea478 reachable as the second parent, and a fast-forward keeps the SHA itself.
  - Only a squash or rebase would orphan the pin.
  - dm's history lands with `Merge …` commits, so `--no-ff` is the repo's own convention.
- **Records:** 26 new or changed file:line citations in the spec sweep were re-resolved at 43294c6, and every one names the claimed line.

ready to ship: yes
