# R0 Gate — IMPLEMENTATION PLAN review (round 0)

- **Artifact:** `design/IMPLEMENTATION_PLAN_seedhammer_template_engrave.md`
- **Derives from:** `design/SPEC_seedhammer_template_engrave.md` (R0 GREEN @ round 2)
- **Reviewer:** adversarial opus architect, verifying every load-bearing symbol against source.
- **Source of truth verified against:** fork `/scratch/code/shibboleth/seedhammer` @ `39cb5cf` (confirmed HEAD); `descriptor-mnemonic@54dd765` (confirmed); `mnemonic-key@1279ef9` (confirmed); `mnemonic-toolkit` — plan pins `6de53879`, repo HEAD is `2f5d088` but `synthesize.rs` is **byte-identical** between them (`git diff 6de53879 HEAD -- …/synthesize.rs` empty), so the stale pin is benign for this plan.

**Verdict: NOT GREEN — 4 Critical / 4 Important.** Blocking: C1, C2, C3, C4, I1, I2, I3, I4.

The plan's headless cryptographic core is *mostly* right (the WDT-Id preimage byte order, the override-TLV layout, the kiw-from-n choice, the no-canonicalize pin, and the four-mint-site enumeration all check out against source). But several load-bearing symbols are mis-named or phantom, one strip mutation is missing (would hard-error at encode), the origin-elision step is under-specified to the point of being un-implementable as written, and Task 8's entire premise (a "template-parser refusal" of `tr(sortedmulti_a)`) does not exist in the fork. These must be fixed before code.

---

## CRITICAL

### C1 — Strip omits `fpPresent = false`; `StripToTemplate` would hard-error `errEmptyTLVEncode` (Task 4)
- **Location:** plan Task 4, Step 4, lines 185–186:
  ```go
  d.tlv.pubkeys = nil; d.tlv.pubPresent = false
  d.tlv.fingerprints = nil
  ```
- **Problem:** The plan clears `pubPresent` alongside `pubkeys` (correct — this is the I1 fix), but for fingerprints it nulls **only the slice** and leaves `fpPresent == true`. The Go encoder rejects exactly this state: `writeTLVSection` returns `errEmptyTLVEncode` when `s.fpPresent && len(s.fingerprints) == 0`.
- **Evidence:** `md/encode.go:271-273`:
  ```go
  if s.fpPresent {
      if len(s.fingerprints) == 0 { return errEmptyTLVEncode }
  ```
  The Go TLV model splits presence (`fpPresent bool`) from value (`fingerprints []idxFP`) — `md/md.go:526-527`. The Rust reference cannot exhibit this because `TlvSection.fingerprints` is a single `Option<Vec<…>>`; `synthesize_template_descriptor` sets `template.tlv.fingerprints = None` (`synthesize.rs:1183`), which clears presence and value atomically. The Go port MUST mirror that atomicity.
- **Fix:** Add `d.tlv.fpPresent = false` alongside `d.tlv.fingerprints = nil` (and add a strip-of-a-fp-bearing-policy test that would catch the omission — the wsh-sortedmulti fixture has fingerprints, so the golden test will surface it, but the plan code as written is wrong). This is the exact `pubkeys`-I1-class bug the SPEC warned about, reappearing on the fingerprints axis.

### C2 — WDT-Id "mirror `walletpolicyid.go:30-64`" cites the WRONG preimage structure (Task 2)
- **Location:** plan Task 2 "Reference" (line 85) and the File-structure table (line 23): "mirror `md/walletpolicyid.go:30-64` STRUCTURE … MINUS `canonicalize(d)` at `:32`."
- **Problem:** `walletpolicyid.go:30-64` is **not** a `useSitePath ‖ writeNode ‖ overrides-TLV` preimage. The actual `WalletPolicyId` preimage is `writeNode(tree)` **‖ per-@N records** (each record = `presence_byte ‖ varint(pathBitLen) ‖ pathBits ‖ varint(usBitLen) ‖ usBits ‖ fp? ‖ xpub?`), with **no leading `useSitePath`** and **no overrides-TLV block** — see `md/walletpolicyid.go:30-101`. An implementer told to "mirror `:30-64` minus canonicalize" would port the per-@N-record machinery (the wrong preimage) and never produce `b02b4403…`. The WDT-Id preimage the plan's *prose* gives (lines 106–116) is correct, but it contradicts the structural-mirror instruction, and removing only `canonicalize` from `WalletPolicyId` does NOT yield WDT-Id.
- **Evidence (correct preimage, Rust `compute_wallet_descriptor_template_id`, `descriptor-mnemonic/crates/md-codec/src/identity.rs:71-104`):**
  ```rust
  let kiw = d.key_index_width();
  d.use_site_path.write(&mut w)?;
  crate::tree::write_node(&mut w, &d.tree, kiw)?;
  if let Some(overrides) = &d.tlv.use_site_path_overrides { /* tag(5b) ‖ varint(bitlen) ‖ payload */ }
  ```
  There is no canonicalize, no per-@N origin/fp/xpub records. The plan should mirror the **forward `encodePayload` serialization fragments** — `writeUseSitePath` (`md/encode.go:134`), `writeNode` (`md/encode.go:159`), and the use-site-override TLV branch of `writeTLVSection` (`md/encode.go:250-268, 343-351`) — NOT `WalletPolicyId`'s body.
- **Fix:** Replace the "mirror `walletpolicyid.go:30-64`" anchor with "compose `writeUseSitePath` + `writeNode` + the use-site-override TLV entry exactly as `encodePayload`/`writeTLVSection` emit them; do NOT reuse `WalletPolicyId`'s per-@N-record loop." Keep the (correct) prose preimage at lines 106-116 as the spec.

### C3 — Origin-elision step is a `...` placeholder; under-specified and as-written wrong (Task 4)
- **Location:** plan Task 4, Step 4, lines 187–189:
  ```go
  if _, ok := canonicalOrigin(d.tree); ok {
      // elide: set the shared/use-site origin to the empty/canonical form (mirror synthesize.rs:1185-1198)
  }
  ```
- **Problem:** (a) The comment is a placeholder — there is no concrete mutation, so the task is not executable as written. (b) The hint "set the shared/**use-site** origin" is wrong: the toolkit elides the **origin path-decl**, never the use-site. (c) The toolkit's concrete elision is `template.path_decl.paths = Shared(OriginPath{components: vec![]})` (`synthesize.rs:1196`) — i.e. it replaces the path-decl with a **Shared, empty** path; it does NOT touch `tlv.origin_path_overrides`. In Go terms the strip must set `d.pathDecl.shared = &originPath{} (empty); d.pathDecl.divergent = nil` so that `encodePayload`'s header recomputation `divergentPaths: dc.pathDecl.divergent != nil` (`md/encode.go:408`) flips Divergent→Shared. A naive port that nulls an origin-override TLV instead — or that leaves `pathDecl.divergent` set — diverges from the toolkit wire.
- **Evidence:** toolkit elision verified at `mnemonic-toolkit/.../synthesize.rs:1195-1198` (set `path_decl.paths = Shared(empty)`, condition `canonical_origin(&descriptor.tree).is_some()`); Go `pathDecl` struct `md/md.go:209-213` (`shared *originPath` / `divergent []originPath`); Go header divergent-bit recompute `md/encode.go:408`. `canonicalOrigin(tree)` exists with the matching shape table (`md/md.go:1097`).
- **Fix:** Specify the exact mutation: on the canonical branch set `d.pathDecl.shared = &originPath{} ; d.pathDecl.divergent = nil` (leave `tlv.originOverrides`/`originPresent` untouched — the toolkit does not clear them). On the non-canonical branch leave `pathDecl` verbatim. Add the assertion that the stripped wire's header divergent bit matches the toolkit golden (the §5 keep-origins vector covers the non-canonical branch).

### C4 — Task 8's premise is factually wrong: the fork has NO template-parser refusal of `tr(sortedmulti_a)` / combinator-`sortedmulti`
- **Location:** plan Task 8, Step 1 (lines 245): "`tr(sortedmulti_a)` + `sortedmulti`-in-combinator → refused at the **template parser** with a clear message."
- **Problem:** No such refusal exists in the fork at the wire/template layer. `validateTapScriptTree` forbids only `wpkh/tr/wsh/sh/pkh/multi/sortedmulti` as tap leaves — it **permits** `multi_a`/`sortedmulti_a` (tags `0x08`/`0x09`). So a `tr(sortedmulti_a …)` taptree **decodes and re-encodes fine** at the wire level (consistent with the SPEC's own O1 finding that depth-≥2 taptrees are wire-encodable). `multi_a`/`sortedmulti_a` shapes are instead "not bip380-expressible — display-only, never verified" at the ADDRESS layer, not refused at parse. The SPEC's "refused because rust-miniscript lacks sortedmulti_a" reasoning is an OFF-DEVICE-recovery property, not an on-device parser refusal — the Go fork doesn't run rust-miniscript. Task 8 therefore cannot be "assertion-only" against an existing parser refusal that isn't there; and engraving a `tr(sortedmulti_a)` template is currently NOT blocked anywhere on-device.
- **Evidence:** `md/md.go:1003-1020` (`validateTapScriptTree` recurses tapTree, rejects only `isForbiddenLeafTag`); `md/md.go` `isForbiddenLeafTag` returns true only for `tagWpkh/tagTr/tagWsh/tagSh/tagPkh/tagMulti/tagSortedMulti` — `tagMultiA (0x08)`/`tagSortedMultiA (0x09)` are NOT in the set; `gui/md1_expand.go:118` ("Unsorted multi / multi_a / sortedmulti_a / taptree … not bip380-expressible (D2) — display-only, never verified"). Hardened-use-site IS refused, but at the derive/address path (`useSiteToChildren` reports `!ok` for a hardened wildcard, `gui/md1_expand.go:~128`), consistent with the SPEC — that half of Task 8 is sound.
- **Fix:** Either (a) make Task 8 ADD a genuine refusal of `tr(sortedmulti_a)`/`sortedmulti`-in-combinator at the chosen layer (decide: template-build entry vs. an explicit pre-engrave admissibility gate) with new code + a fail-then-pass test that genuinely fails on `39cb5cf`; or (b) if the SPEC's intent is that these shapes are simply never *offered* on-device (the device only BUILDS the shapes it can construct, and a SUPPLIED `tr(sortedmulti_a)` is engraved verbatim + flagged display-only), re-write Task 8 to assert that boundary instead. As written, Task 8 cannot fail-then-pass against a non-existent parser refusal — resolve the Open-item, do not leave it open.

---

## IMPORTANT

### I1 — `decodeChunksToDescriptor` is a phantom symbol; the real entry is `Reassemble` (Tasks 3 & 4)
- **Location:** plan Task 3 line 154 and Task 4 line 184 invent `decodeChunksToDescriptor(strs)`.
- **Problem:** No such function exists anywhere in the fork. The chunks→`*descriptor` decode that `WalletPolicyIDStubChunks` actually uses is `Reassemble(strs []string) (*descriptor, error)` (called by `WalletPolicyIdChunks`). Using the right symbol is what guarantees `FormAwareStubChunks` is byte-consistent with `WalletPolicyIDStubChunks` (Open-item #2).
- **Evidence:** `md/chunk.go:207` (`func Reassemble`); `md/walletpolicyid.go:119-124` (`WalletPolicyIdChunks` → `Reassemble(strs)`). Grep for `decodeChunksToDescriptor` across the fork: zero hits.
- **Fix:** Rename to `Reassemble` in Tasks 3 and 4 (and in `StripToTemplate`). This also resolves Open-item #2 affirmatively: yes, `Reassemble` is the exact shared decode path.

### I2 — GUI flow function names are wrong (`engraveSingleSig`, `engraveMultisig` are enum constants, not functions) (Tasks 6 & 7)
- **Location:** plan Task 6 (line 223: "Inner `ChoiceScreen` on `engraveSingleSig`"), Task 7 (multisig BUILD), File table (line 28-29).
- **Problem:** `engraveSingleSig`/`engraveMultisig` are `program`-enum constants, not functions. The real insertion points are: single-sig flow `engraveSingleSigFlow` (`gui/singlesig.go:30`), which derives `b` via `deriveSingleSigBundle` then engraves via `singleSigEngraveCards`→`bundleEngrave` (`gui/singlesig.go:85-86`); multisig BUILD `buildMultisigPolicyFlow` (`gui/multisig_build.go:38`), engraving via `multisigEngraveCards`→`bundleEngrave` (`gui/multisig_build.go:120-121`). The opt-in inserts before the engrave call on `b.MD1`.
- **Evidence:** `gui/gui.go:145-164` (program enum + the lockstep guard — the no-new-`program` claim is CORRECT; an inner ChoiceScreen on an existing flow does not trip `:164`, confirmed); `gui/singlesig.go:30,77,85`; `gui/multisig_build.go:38,120`; `gui/multisig.go:64` (`supplyMultisigPolicyFlow`); `gui/multisig_supply.go:72` (`allSlotsHaveXpub` — exists at that exact line; supply path separability confirmed → D1 untouched-supply claim is sound).
- **Fix:** Correct the function names in Tasks 6/7 and the File table.

### I3 — Task 6's `PolicyComplex` single-sig assertion is impossible (vacuous) (Task 6)
- **Location:** plan Task 6 line 221: "For a `classifyPolicy`→`PolicyComplex` shape … use the complex fixture if reachable single-sig …".
- **Problem:** `PolicyComplex` is unreachable on the single-sig engrave path by construction. The single-sig flow MINTS md1 from `md.EncodeSingleSig` with a `ScriptKind` ∈ {wpkh, pkh, sh(wpkh), tr-single-key}; every such root classifies as `PolicySingle,0,0` in `classifyPolicy`. `PolicyComplex` (the fall-through default) is only reachable via multisig / general-miniscript / taptree shapes — i.e. Task 7's path. So the Task 6 honest-minimal-consent (C3/DD7) assertion has no satisfiable single-sig fixture.
- **Evidence:** `md/md.go:1266` (`classifyPolicy`); single-sig arms return `PolicySingle` (`md/md.go:1268-1271, 1282-1284, 1296-1300`); `PolicyComplex` is the default at `md/md.go:1315`; existing test `md/md_test.go:491-493` asserts `sh(wpkh)→PolicySingle`. `PolicyComplex` fixtures live on the multisig/general path (`md/md_test.go:458-460`).
- **Fix:** Drop the complex/honest-minimal assertion from Task 6 (single-sig) and place it solely in Task 7 (multisig-BUILD / general), where a `PolicyComplex` fixture is reachable. Single-sig Task 6 keeps only the depth-≥2-tr EXPERIMENTAL gate if a tr-with-taptree single-key fixture is reachable there (note: a single-key `tr` with a script tree is NOT "single-sig" in the flow sense — confirm the fixture path before asserting). This resolves Open-item #3.

### I4 — Golden-generation command `md-cli identity --template` does not exist (Task 2)
- **Location:** plan Task 2 line 90: `cargo run -q -p md-cli -- identity --template '…' --path m/48h/0h/0h/2h`.
- **Problem:** `md-cli` has no `identity` subcommand. The WDT-Id is emitted by `md inspect` (`crates/md-cli/src/cmd/inspect.rs:19` calls `compute_wallet_descriptor_template_id`; printed as `wallet-descriptor-template-id`). The golden value itself is CORRECT and reproducible: the committed snapshot `crates/md-cli/tests/snapshots/json_snapshots__inspect@wsh_sortedmulti.snap:60` shows `wallet_descriptor_template_id = b02b44037119e6b6fd1d82f61aa17e21` for a `pubkeys: null` `wsh(sortedmulti)` (its `wallet_policy_id` is the distinct `80f18935…`, confirming key-independence). So `b02b4403…` is a valid golden; only the generation command is wrong.
- **Fix:** Replace with `md inspect <keyless-wsh-sortedmulti-md1>` (or cite the snapshot fixture directly). Use the full 16 bytes `b02b44037119e6b6fd1d82f61aa17e21` in the test, not a truncated literal.

---

## MINOR

### M1 — `kiw(d.n)` is correct, but state WHY it equals `WalletPolicyId`'s `kiw(pathDecl.n)`
- The plan's `kiw(d.n)` (Task 2 line 111) matches Rust `d.key_index_width()` = `⌈log₂(n)⌉` from `self.n` (`encode.rs:37-41`). The Go `WalletPolicyId` uses `kiw(dc.pathDecl.n)` on a canonicalized clone — these agree only because post-decode `d.n == d.pathDecl.n` (`md/md.go:858` sets `n: pd.n`) and canonicalize preserves it. The plan's `pathDecl.n == d.n` guard (line 110) enforces this. Fine as-is, but add a one-line note that `kiw(d.n)` is chosen to match Rust's `key_index_width()` (which reads `n`, not `pathDecl.n`), and the guard makes the two definitions provably equal.

### M2 — Override-TLV emptiness gate differs Rust vs Go (harmless on the strip path, document it)
- Rust WDT-Id writes the override block on `if let Some(overrides)` with no non-empty check (`identity.rs:79`); the Go `writeTLVSection` use-site branch returns `errEmptyTLVEncode` on present-but-empty (`md/encode.go:251-252`). On every descriptor reaching WDT-Id (decoded, `useSitePresent ⇒ non-empty`), they agree. If the WDT-Id port reuses the `writeTLVSection` use-site branch verbatim it inherits the (never-triggered-here) empty-guard — acceptable. Just note the WDT-Id helper must emit the override entry with the SAME byte layout (`tag 0x00 (5b) ‖ varint(bitLen) ‖ idx(kiw) ‖ writeUseSitePath`, ascending idx) — verified to match Rust `identity.rs:79-97` exactly (`tlvUseSitePathOverrides = 0x00` == `TLV_USE_SITE_PATH_OVERRIDES = 0x00`).

### M3 — "guard INSIDE WDT-Id" is a safe addition, not a Rust mirror — say so
- The plan carries the `pathDecl.n == d.n` guard inside `WalletDescriptorTemplateId` (Task 2 line 110, R0 pin #2). Rust `compute_wallet_descriptor_template_id` has NO such guard (it just calls `d.key_index_width()`). This is a defense-in-depth ADDITION (mirrors the Go `encodePayload` guard at `encode.go:401`), not a Rust-parity requirement. Harmless on the decode path (always equal). Label it as a deliberate Go-side hardening so a future reader doesn't "fix" it to match Rust by removing it.

### M4 — `canonicalize`-on-re-emit is consistent, not interfering (Task 4 scrutiny #3)
- The scrutiny question "will `encodePayload`'s canonicalize interfere with byte-identity vs the toolkit strip?" resolves NO: the toolkit re-emits through the SAME `encode_payload` that canonicalizes (`synthesize.rs:1212` → `chunk::split` → `encode_payload` → `canonicalize_placeholder_indices`, `encode.rs:67`). Both sides canonicalize on a clone before emit, so byte-identity holds. No action beyond noting it.

---

## NITS

- **N-a:** File table (line 27) cites `md/encode_multisig.go:158` as `WalletPolicyIDStub(d)` — confirmed exact (`md/encode_multisig.go:158`, takes the unexported `*descriptor`; `FormAwareStub(d)` is the right shape). Good.
- **N-b:** `Template.N` (Task 6 line 223) is real and exported (`md/md.go:1206`, populated `int(d.n)` at `md.go:1380`); the GUI already reads it (`gui/md1_inspect.go:60`). The plan's "Surface `Template.N`" is correct — no new accessor needed. Good.
- **N-c:** `bundleFlow`/`bundleEngrave` (`gui/bundle_flow.go:24,327`) confirmed; `bundleEngrave` engraves card strings VERBATIM (`bundlePlatePlan`, `str: s` "VERBATIM gathered chunk string (I-4)", `bundle_flow.go:296-312`) with only `validateMdmk` (not re-encode). The N1 engrave-verbatim-vs-form-aware-verify split (Task 5/7) is sound.
- **N-d:** `isWalletPolicy` (Task 1) = `d.tlv.pubPresent && len(d.tlv.pubkeys) > 0` matches Rust `is_wallet_policy` = `matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())` (`encode.rs:50-52`). Correct. The I1 desync test (line 64-65) is valid.
- **N-e:** Four-mint-site enumeration is EXHAUSTIVE and exact (verify.go:116 `b.MD1 []string`; singlesig_derive.go:67 `md1 []string`; multisig_derive.go:42 `suppliedMd1 []string`; encode_multisig.go:158 `d *descriptor`). No missed production stub-mint or display-id site. All four carry FULL policies in normal operation, so the form-aware selector must (and does, per `isWalletPolicy`) route them to `WalletPolicyId` byte-identically — the I5 regression pin is well-placed.

---

## Ruling on the plan's own "Open for plan-R0" list (lines 273-277)
1. **UseSitePathOverrides-TLV inclusion/byte-order vs `identity.rs:71-104`:** RULED — reproduced exactly by mirroring `writeTLVSection`'s use-site branch (`tag 0x00 (5b) ‖ varint(bitLen) ‖ idx(kiw) ‖ writeUseSitePath`, ascending). See M2. The blocker is C2 (cite the right serializer), not the byte-order itself.
2. **`decodeChunksToDescriptor` correctness:** RULED — it is `Reassemble` (I1). The chunks helpers ARE byte-consistent once renamed.
3. **Task 6/7 `PolicyComplex` reachability:** RULED — NOT reachable on single-sig (I3); assert only on the multisig/general path (Task 7).
4. **Task 8 refusals present vs new code:** RULED — the `tr(sortedmulti_a)`/combinator refusal is NOT present in the fork (C4); hardened-use-site IS present (address layer). Task 8 needs new refusal code or a re-scoped boundary assertion — it cannot be assertion-only as written.

---

## VERDICT

**NOT GREEN — 4C / 4I.** Blocking IDs: **C1** (missing `fpPresent=false` → encode error), **C2** (WDT-Id mirrors wrong preimage `:30-64`), **C3** (origin-elision unspecified/wrong-target), **C4** (Task 8 template-parser refusal does not exist), **I1** (`decodeChunksToDescriptor`→`Reassemble`), **I2** (GUI flow function names), **I3** (Task 6 PolicyComplex vacuous), **I4** (`md-cli identity` command does not exist). Fold all eight, persist this review, re-dispatch.
