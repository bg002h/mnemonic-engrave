# R0 Gate — IMPLEMENTATION PLAN review (round 1, FOCUSED on the 8 folds)

- **Artifact:** `design/IMPLEMENTATION_PLAN_seedhammer_template_engrave.md` (DRAFT v2)
- **Round 0 verdict:** NOT GREEN — 4C/4I (C1,C2,C3,C4,I1,I2,I3,I4); author folded all 8 → this is round 1.
- **Reviewer:** adversarial opus architect. Round 0 already validated the crypto core; this round verifies each fold is correctly resolved, hunts for fold-introduced drift, and gives the final 0C/0I check.
- **Source of truth verified against (READ-ONLY):**
  - Fork `/scratch/code/shibboleth/seedhammer` @ `main`. (Plan/SPEC pin `39cb5cf`.)
  - `descriptor-mnemonic` working tree @ `54dd765` (confirmed `git rev-parse HEAD` = `54dd765a…`, the v0.37.0 release commit).
  - `mnemonic-toolkit` HEAD `959af85`; plan pins `6de53879`. `git diff 6de53879 HEAD -- …/synthesize.rs` is **EMPTY** → byte-identical; the stale pin is benign (re-confirmed this round).
  - `mnemonic-key` `mk-cli/src/cmd/mod.rs` (`derive_stub_from_md1`).

---

## Per-fold status

### C1 — strip must set `tlv.fpPresent = false` (+ `pubPresent = false`) — **RESOLVED**
- Plan Task 4 Step 4 (lines 186-187) now reads `d.tlv.pubkeys = nil; d.tlv.pubPresent = false` AND `d.tlv.fingerprints = nil; d.tlv.fpPresent = false` with the inline note "C1: MUST clear fpPresent too, else errEmptyTLVEncode (encode.go:271-273)".
- Both flags exist: `tlvSection.fpPresent`/`pubPresent` at `md/md.go:527,529`. Present-but-empty genuinely hard-errors: `md/encode.go:271-273` (`if s.fpPresent { if len(s.fingerprints)==0 { return errEmptyTLVEncode } }`) — and the identical guard for pubkeys at `:292-294`. Rust mirror: `synthesize.rs:1182-1183` clears `tlv.pubkeys=None` + `tlv.fingerprints=None` atomically (Option = presence+value in one). Go port now mirrors that atomicity on both axes. **RESOLVED.**

### C2 — WDT-Id from forward serializers, NOT by editing `WalletPolicyId` — **RESOLVED**
- Plan Task 2 (lines 85, 104-119) + File table (line 23) now explicitly forbid cloning `WalletPolicyId`: "`walletpolicyid.go:30-64` is `WalletPolicyId`'s per-@N-record preimage — a DIFFERENT, key-dependent computation that will NOT yield `b02b4403…`. Construct the WDT-Id preimage `useSitePath ‖ writeNode(tree) ‖ UseSitePathOverrides-TLV` DIRECTLY via `writeUseSitePath` / `writeNode` / the use-site-override branch of `writeTLVSection`, matching `identity.rs:71-104` byte-for-byte." File table line 23 is now "Read-only ref … mirror `:30-64` MINUS canonicalize" — re-worded; the body prose overrides any residual mirror hint and the C2 reference paragraph is unambiguous that it is the WRONG preimage.
- Verified the Rust preimage `compute_wallet_descriptor_template_id` at `identity.rs:71-104`: `kiw = d.key_index_width()` → `d.use_site_path.write(&mut w)` → `tree::write_node(&mut w, &d.tree, kiw)` → optional `if let Some(overrides) = &d.tlv.use_site_path_overrides { tag(TLV_USE_SITE_PATH_OVERRIDES=0x00, 5b) ‖ varint(bitlen) ‖ payload }`. NO canonicalize, NO keys/fp, NO per-@N records. This is exactly the plan's prose preimage (lines 106-116) and is structurally distinct from `walletpolicyid.go:30-101` (which canonicalizes at `:32` then emits `writeNode ‖ per-@N {presence ‖ varint(pathLen)‖path ‖ varint(usLen)‖us ‖ fp? ‖ xpub?}`). The plan no longer instructs cloning that body. Golden `b02b4403…` confirmed reproducible (see I4). **RESOLVED.**

### C3 — origin-elision targets the PATH-DECL (`pathDecl.shared=&empty; divergent=nil`) — **RESOLVED**
- Plan Task 4 (lines 188-193): on the `canonicalOrigin(d.tree)` ok branch, `empty := originPath{}; d.pathDecl.shared = &empty; d.pathDecl.divergent = nil`.
- Field names verified real: `pathDecl{ n uint8; shared *originPath; divergent []originPath }` at `md/md.go:209-213`. Rust target `synthesize.rs:1196`: `template.path_decl.paths = PathDeclPaths::Shared(OriginPath { components: vec![] })` — i.e. Shared + empty components. Match.
- **CRITICAL init-subtlety check (does `originPath{}` zero-value serialize as depth-0 shared?): YES, no subtlety.** `originPath struct{ components []pathComponent }` (`md/md.go:190`); `originPath{}` ⇒ `components == nil`. `writeOriginPath` writes the depth as `w.write(uint64(len(p.components)), 4)` (`md/encode.go:94`) ⇒ `len(nil)==0` ⇒ a 4-bit zero depth + no components = exactly Rust `vec![]`. `writePathDecl` shared branch (`md/encode.go:118-124`) routes a non-nil `shared` through `writeOriginPath`. Header divergent bit recomputes from `divergentPaths: dc.pathDecl.divergent != nil` (`md/encode.go:408`); setting `divergent=nil` flips Divergent→Shared on re-emit. And `canonicalize` (the encode-path clone) takes its identity fast-path for a normal decoded descriptor and leaves `pathDecl` untouched (`md/canonicalize.go:61-63`), and only re-permutes `divergent` when `divergent != nil` (`:68`) — so a nil-divergent/shared-empty path-decl survives canonicalize intact. The `tlv` origin overrides are correctly left untouched (Rust mutates only pubkeys/fingerprints/path_decl). **RESOLVED.**

### C4 — Task 8 ADDS refusal code; the fork lacks the refusal today — **RESOLVED**
- Plan Task 8 (lines 245-253) now states "C4: the fork does NOT currently refuse `tr(sortedmulti_a)` / `sortedmulti`-in-combinator … this task ADDS refusal code on the TEMPLATE-ENGRAVE path"; Step 2 expects FAIL "(the refused shapes engrave today)".
- Verified the fork genuinely permits them: `validateTapScriptTree` (`md/md.go:1005-1020`) recurses tapTree and refuses only `isForbiddenLeafTag`; `isForbiddenLeafTag` (`:1022-1028`) returns true ONLY for `tagWpkh/tagTr/tagWsh/tagSh/tagPkh/tagMulti/tagSortedMulti`. `tagMultiA(0x08)`/`tagSortedMultiA(0x09)` (`md/md.go:48-49`) are NOT in that set → a `tr(…sortedmulti_a…)` tap leaf decodes + re-encodes today. So the RED test is genuinely red. The detection ("tree-tag walk for `tr(...sortedmulti_a...)` + `sortedmulti`-under-combinator from the decoded tree") is feasible: the decoded tree carries these exact tag constants and the tap-tree structure (`childrenBody`/`trBody`/`multiKeysBody`), so a tag-walk guard is implementable. Hardened-use-site correctly remains a derive/address-layer refusal (the plan no longer asserts a parser refusal that doesn't exist). **RESOLVED.**

### I1 — decode entry is `Reassemble` (`md/chunk.go:207`), not the phantom — **RESOLVED (with NIT, see N-1)**
- Plan Tasks 3/4 now route through `reassembleToDescriptor(strs)` annotated inline "the Reassemble-based decode (md/chunk.go:207)". `Reassemble(strs []string) (*descriptor, error)` exists at `md/chunk.go:207` and is the exact entry `WalletPolicyIdChunks` uses (`md/walletpolicyid.go:120`), yielding a `*descriptor` usable by the stub helpers. The phantom `decodeChunksToDescriptor` is gone. **RESOLVED** — but the plan introduces a NEW helper *name* `reassembleToDescriptor` it never defines; the real symbol is `Reassemble` (see N-1).

### I2 — `engraveSingleSigFlow:30` / `buildMultisigPolicyFlow:38`; no `gui.go:164` trip; distinct from supply — **RESOLVED**
- `engraveSingleSigFlow` is a real func at `gui/singlesig.go:30` (engraves via `bundleEngrave(ctx,th,cards)` at `:86`); the opt-in inserts before that call. `buildMultisigPolicyFlow` is a real func at `gui/multisig_build.go:38` (engraves via `bundleEngrave(ctx,th,cardsOut)` at `:121`). Both already use inner `ChoiceScreen`s heavily, so a new inner ChoiceScreen adds no `program` enum. The `gui.go:164` guard `var _ [1]struct{} = [qaProgram - bip85Derive]struct{}{}` (`gui/gui.go:145-164`) only trips when a new `program` constant is inserted between `bip85Derive` and `qaProgram` — not affected. `buildMultisigPolicyFlow` (`multisig_build.go:38`) is DISTINCT from `supplyMultisigPolicyFlow` (`gui/multisig.go:64`), whose `allSlotsHaveXpub` gate (`gui/multisig_supply.go:72`) the plan leaves untouched (D1). **RESOLVED.**

### I3 — complex/depth consent in Task 7 only; Task 6 single-sig cannot reach `PolicyComplex`/depth-≥2 — **RESOLVED**
- Plan Task 6 (line 225) now states single-sig always classifies `PolicySingle` and the complex/depth consent is "exercised in Task 7 … NOT here — I3". Verified: `singleSigPickFlow` offers exactly {BIP-84 wpkh, BIP-44 pkh, BIP-49 sh(wpkh), BIP-86 tr-single} (`gui/singlesig_pick.go:28-35`); `singleSigTree` builds `ScriptTr → trBody{isNums:false, tree:nil}` (`md/encode_singlesig.go:90,98`). `classifyPolicy` (`md/md.go:1266-1316`) returns `PolicySingle` for wpkh/pkh keyArg (`:1268-1271`), tr with `!isNums && tree==nil` (`:1282-1283`), and sh(wpkh) (`:1296-1298`); `PolicyComplex` is the fall-through default (`:1315`) reachable only via multisig/general/script-tree shapes. A single-key tr has no script tree ⇒ no depth-≥2. So Task 6 has NO vacuous assertion and Task 7 (multisig-BUILD/general) is the sole reachable site for `PolicyComplex` + depth-≥2. **RESOLVED.**

### I4 — golden `b02b44037119e6b6fd1d82f61aa17e21` via `md inspect` (not `identity --template`) — **RESOLVED**
- Plan Task 2 (lines 90, 97) + line 278 now use `md inspect` and cite the full 16-byte golden. Verified: `crates/md-cli/src/cmd/inspect.rs:9 pub fn run` calls `compute_wallet_descriptor_template_id` (`:19`) and prints `wallet-descriptor-template-id` (`:36,57`); there is NO `identity.rs` under `crates/md-cli/src/cmd/` (the subcommand never existed). The golden value is the committed snapshot `crates/md-cli/tests/snapshots/json_snapshots__inspect@wsh_sortedmulti.snap:60` = `b02b44037119e6b6fd1d82f61aa17e21` for a `pubkeys:null` `wsh(sortedmulti)`. Self-consistent; not re-derived per instruction. **RESOLVED.**

---

## NEW findings introduced by the folds

### N-1 (NIT, NOT C/I) — `reassembleToDescriptor` / `splitToChunks` are undefined plan-local helper names; the real symbols are `Reassemble` and `split`
- **Location:** Task 3 line 154 + Task 4 lines 184, 194 (`reassembleToDescriptor(strs)`, `splitToChunks(d)`).
- **Detail:** Neither helper exists in the fork. The real decode entry is `Reassemble` (`md/chunk.go:207`) and the real `encodePayload`-backed chunker is `split(d)` (`md/chunk.go:121-178`, line 122 calls `encodePayload`). This is the SAME phantom-name axis round 0 flagged as I1 — the decode-side phantom (`decodeChunksToDescriptor`) was renamed, but DRAFT v2 introduces two new author-coined wrapper names on the same axis.
- **Why this is only a NIT, not a re-opened I1:** unlike round-0's bare `decodeChunksToDescriptor` (cited with no pointer to any real symbol), BOTH new names are annotated inline with the exact real symbol + line — `reassembleToDescriptor` → "the Reassemble-based decode (md/chunk.go:207)"; `splitToChunks` → "encodePayload-backed; same chunker the encoders use" (= `split`, `chunk.go:121`). An implementer cannot be misled: the real symbols are named adjacent. Both `Reassemble` and `split` already return precisely what the plan needs (`*descriptor` / `[]string`), so the cleanest resolution is to drop the wrappers and call `Reassemble`/`split` directly (or define them as one-line aliases). No code-correctness risk. **Recommend the implementer substitute the real symbol names; does not block the gate.**

### No other new Critical/Important
- **Cross-task consistency of `reassembleToDescriptor`:** used identically in Tasks 3 and 4 (same signature `(strs) (*descriptor, error)`) — internally consistent (just under a non-canonical name; N-1).
- **Task 6/7 split:** clean — Task 6 (single-sig, `PolicySingle` only) and Task 7 (multisig-BUILD, `PolicyComplex`+depth-≥2 consent) are non-overlapping and each has a reachable fixture class; the supply flow (`supplyMultisigPolicyFlow`/`allSlotsHaveXpub`) is explicitly out of scope (D1) in both.
- **Four mint sites (Task 5) re-verified unchanged:** `bundle/verify.go:116` `WalletPolicyIDStubChunks(b.MD1)`; `gui/singlesig_derive.go:67` `WalletPolicyIDStubChunks(md1)`; `gui/multisig_derive.go:42` `WalletPolicyIDStubChunks(suppliedMd1)`; `md/encode_multisig.go:158` `WalletPolicyIDStub(d)` — all four present at the cited lines; the form-aware swap is exact and the I5 regression pin (keyed → `WalletPolicyId`) holds because `isWalletPolicy` routes keyed policies unchanged.
- **`canonicalOrigin(d.tree)` two-value signature** (`md/md.go:1097` `func canonicalOrigin(tree node) (originPath, bool)`) matches Task 4's `if _, ok := canonicalOrigin(d.tree); ok`.
- **`FormAwareStub` branch** matches mk-cli `derive_stub_from_md1` (`mk-cli/src/cmd/mod.rs:72-74`: `if descriptor.is_wallet_policy() { WalletPolicyId } else { WalletDescriptorTemplateId }`).
- **No round-0-resolved item regressed.** All eight folds are present, source-faithful, and mutually consistent. The minor naming wrappers (N-1) are the only residue and are non-blocking.

---

## VERDICT

**GREEN — 0 Critical / 0 Important.**

All 8 round-0 folds (C1, C2, C3, C4, I1, I2, I3, I4) verified RESOLVED against source. No new Critical/Important introduced; no cross-task inconsistency; no regression of any round-0-resolved item. One NIT (N-1: substitute the real symbols `Reassemble`/`split` for the author-coined `reassembleToDescriptor`/`splitToChunks` wrappers) — non-blocking, the real symbols are cited inline. The plan clears the R0 gate and may advance to implementation (single subagent, TDD, worktree; then the mandatory whole-diff adversarial exec review).
