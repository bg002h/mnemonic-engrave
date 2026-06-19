# R0 DELTA-REVIEW — T6a WalletPolicyId port + mk1 policy-bound stub (wire-fidelity addition)

**Reviewer:** opus architect (focused delta-review, R0). **Date:** 2026-06-19.
**Scope:** ONLY the NEW bundle-composition addition to the two docs — the `md.WalletPolicyId` port (SPEC Phase-A bullet + plan Task 2W), the policy-bound mk1 stub (SPEC Phase B "Derive the 3 legs", invariant I-6b, comparator stub-check, template-OUT), and the plan deltas (Task 2W, the Task 3 stub-binding check, FuzzWalletPolicyId, acceptance + self-review). Plus a no-drift check on the already-R1-GREEN remainder. **The rest of both docs is NOT under review.**

**Documents reviewed:**
- `design/SPEC_seedhammer_T6a_singlesig_flagship.md`
- `design/IMPLEMENTATION_PLAN_seedhammer_T6a1_headless.md`
- `design/agent-reports/seedhammer-T6-recon-bundle-composition-stub.md` (grounding)

**Authoritative sources verified (NOT the docs' prose):**
- RUST md-codec @ pinned `c85cd49` (v0.36.0): `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/identity.rs` (confirmed `git rev-parse HEAD == c85cd49`, `version = "0.36.0"`), `…/src/canonicalize.rs`.
- TOOLKIT: `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs`, `…/src/cmd/bundle.rs`.
- mk-codec: `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/key_card.rs`, `…/design/SPEC_mk_v0_1.md`.
- GO fork @ pinned `e4013a8` (confirmed `git rev-parse HEAD == e4013a88011284c71f6da1b5629555bdc52c7e88`): `md/identity.go`, `md/expand.go`, `md/canonicalize.go`, `md/bits.go`, `md/encode.go`, `md/md.go`, `mk/mk.go`, `gui/derive_xpub.go`, `codex32/mspayload.go`.

---

## VERDICT: NOT GREEN — 0 Critical / 2 Important / 3 Minor

The conceptual additions are CORRECT and source-verified — the stub IS `WalletPolicyId[..4]` (not the chunk-id, not the stale bytecode hash), full-policy-only is right, the T6a-1/T6a-2 split is clean, the comparator check is sound. But the **byte-fidelity prose describing the `WalletPolicyId` preimage is materially incomplete in all three docs** (it omits the per-@N path/use-site record bytes), and **neither doc flags the live Go reuse trap** (the existing `ExpandWalletPolicy`/`ExpandedKey` is a display accessor whose origin-path serialization diverges from Rust's `expand_per_at_n`). Both are byte-divergence hazards on a golden-byte-locked package and must be corrected in the plan before code, per the external-protocol-fact rule. Neither rises to Critical because the Task 2W Step-1 toolkit differential, if built against the full id, mechanically catches a wrong preimage — but the prose actively misdirects the implementer and the auxiliary gates (presence-significance, stability) do NOT catch the omission, so the spec/plan text must be fixed rather than relying on the gate alone.

---

## Explicit rulings requested

### (a) WalletPolicyId preimage byte-fidelity to `identity.rs:172-240` — INCOMPLETE (see I1)
The docs describe the preimage as `placeholder-tree ‖ per-@N {presence_byte + fp[4] + xpub[65]}`. The ACTUAL preimage (identity.rs:188-228) per record is `presence_byte ‖ record_bytes ‖ fp? ‖ xpub?`, where `record_bytes` (identity.rs:203-211) = `into_bytes( varint(path_bit_len) ‖ path_bits ‖ varint(use_site_bit_len) ‖ use_site_bits )`. The docs OMIT `record_bytes` entirely. The in-source golden (identity.rs:463) is explicit: `record total = 1 + 8 + 4 + 65 = 78 bytes` — the "8" is `record_bytes`, which the docs' "presence_byte+fp+xpub" (1+4+65 = 70) drops. **NOT byte-faithful as written.**

### (b) Stub = WalletPolicyId[..4] (not the chunk-id) — CORRECT
- Toolkit sets stub = `compute_wallet_policy_id(descriptor).as_bytes()[..4]` at every synth site: `synthesize.rs:179-181, 215-217, 272-274, 453-455, 625-627`; pinned read-back assertion `decoded_mk1.policy_id_stubs[0] == policy_id.as_bytes()[..4]` (`synthesize.rs:1089-1090, 1134-1135, 1558-1562`).
- `Md1EncodingId` = `SHA-256(encode_payload)[0:16]` (identity.rs:39-45) = Go `computeEncodingID` (md/identity.go:11-16) — the md1 chunk-set-id source (md/chunk.go:126,285 → deriveChunkSetID). Confirmed DISTINCT primitive; NOT the stub.
- The stale-doc callout is ACCURATE: mk-codec `key_card.rs:25-27` still reads "Each stub is the top 4 bytes of the policy's `SHA-256(canonical_bytecode)`" — the OLD formula. Authoritative `SPEC_mk_v0_1.md:186` (§3.3): stub = top 4 bytes of `md_codec::compute_wallet_policy_id(descriptor)`, "**NOT** the md1 bytecode hash (`Md1EncodingId`)"; §9 Q-2 table (`:385`) records the 2026-06-10 audit-I1 supersession. Both docs cite this correctly.
- Go gap confirmed: `grep WalletPolicyId` over the fork's `md/` + `mk/` returns ZERO definitions — only `computeEncodingID`. The port is genuinely net-new.

### (c) T6a-1 (port) vs T6a-2 (stub-setting + warning-drop) split — CORRECT, no leakage
- The PORT (`md.WalletPolicyId` + `WalletPolicyIDStub`) and the comparator stub-binding check are headless → correctly in T6a-1 (plan Task 2W, Task 3). No GUI dependency.
- The stub-SETTING (`mk.Encode` with `Stubs:[WalletPolicyIDStub(md1)]`) + the "Unbound Key Card" warning-drop are correctly deferred to T6a-2/GUI (SPEC §Phase-B "mk1 stub binding"; plan self-review l.105). Verified the GUI surface: `mk.Encode(card)` consumes `Card{… Stubs [][4]byte}` (mk/mk.go:133,137,286); T4 today sets `Stubs: [][4]byte{{0,0,0,0}}` (gui/derive_xpub.go:142) and shows "Unbound Key Card / placeholder policy stub (00000000)" (gui/derive_xpub.go:157,239-241). T6's bound flow replacing stub-0 with `WalletPolicyIDStub(md1)` and dropping the warning is the right GUI delta — and is correctly NOT in the T6a-1 plan. No leakage either direction.

### (d) Full-policy-only / template-OUT — CORRECT
- `is_wallet_policy()` = `pubkeys Some(non-empty)` (recon Q1, encode.rs:50-52). Toolkit `debug_assert!(descriptor.is_wallet_policy())` at every synth site (synthesize.rs:195,231,346,472,644). Bundle md1 is always full-policy.
- The template-OUT rationale is ACCURATE and source-confirmed: `WalletPolicyId` is key-presence-significant (identity.rs:217-219 presence_byte + 223-227 conditional fp/xpub append; gated by `walletpolicyid_template_only_differs_from_full_cell_7`, identity.rs:610-617), so a template-only md1's id ≠ the full-policy id → a full-policy-computed stub would not match → binding breaks. SPEC §OUT and I-6b lock full-policy-only correctly.

---

## Findings

### IMPORTANT

**I1 — The WalletPolicyId preimage prose omits the per-@N `record_bytes` (path + use-site bits) in all three docs. (byte-divergence hazard)**
- **Where:** SPEC l.25 ("placeholder-tree ‖ per-@N `presence_byte`+fp[4]+xpub[65]"); PLAN l.66 ("each record = `presence_byte (fp_present | xpub_present<<1)` + fp[4] + xpub[65] when present"); recon l.13.
- **Source:** identity.rs:188-228. Each record is `presence_byte` (1B, l.221) ‖ **`record_bytes`** (l.211,222) ‖ `fp?` (l.223-225) ‖ `xpub?` (l.226-228). `record_bytes` is built at l.206-211: `BitWriter` ⇐ `write_varint(path_bit_len)` ‖ `re_emit_bits(path_bytes, path_bit_len)` ‖ `write_varint(use_site_bit_len)` ‖ `re_emit_bits(us_bytes, use_site_bit_len)`, then `.into_bytes()`. The in-source golden (identity.rs:460-465) proves it: `record total = 1 + 8 + 4 + 65 = 78`; the docs' description sums to 70 (drops the 8-byte `record_bytes`).
- **Why it matters:** An implementer following the prose literally would hash `tree ‖ {presence_byte ‖ fp ‖ xpub}` per @N and produce a WRONG id for EVERY descriptor — the engraved mk1 stub would never match a toolkit-recomposed bundle. The presence-significance gate (Task 2W Step 1) and the stability gate would BOTH still pass under this omission (nulling fp+xpub still flips presence_byte and drops fp/xpub → different id; elision-stability holds because the resolved record is identical) — so ONLY the toolkit differential catches it, and only if pinned on the full id (see M2). Do not rely on the gate to backstop wrong prose on a byte-locked package.
- **Fix:** Replace the preimage description in SPEC l.25 and PLAN l.66 (Task 2W) with the exact construction:
  `hash_input = canonical_template_tree_bytes ‖ Σ_@N[ presence_byte ‖ record_bytes ‖ fp[4]? ‖ xpub[65]? ]`, where
  `record_bytes = into_bytes( varint(path_bit_len) ‖ path_bits ‖ varint(use_site_bit_len) ‖ use_site_bits )` (lengths in BITS, single byte-boundary pad), `presence_byte = (fp_present | (xpub_present<<1)) & 0b0000_0011`, `canonical_template_tree_bytes = into_bytes(write_node(tree, key_index_width))`. SHA-256[:16]. Cite identity.rs:180-239 and the golden at :460-465.

**I2 — Neither doc flags that the existing Go `ExpandWalletPolicy`/`ExpandedKey` is a DISPLAY accessor that diverges from Rust `expand_per_at_n` for the hash preimage. (reuse trap → byte-divergence)**
- **Where:** PLAN Task 2W Step 3 ("Implement `md/walletpolicyid.go` — the canonical-expanded preimage … byte-exact vs `identity.rs:172-240`") gives no guidance on the expansion source; the prose ("per-@N records") points the implementer straight at the existing `ExpandedKey`.
- **Source:** The Go `ExpandedKey` (md/expand.go:56-64) carries `OriginPath bip32.Path` — hardening encoded IN-BAND as `value + HardenedKeyStart` (componentsToPath, expand.go:148-160), NOT the raw `originPath{components:[{hardened,value}]}` form. Rust's preimage (identity.rs:193) calls `e.origin_path.write()` on the RAW `OriginPath` (the `{hardened,value}` component form via `writeOriginPath`/`writePathComponent`, fork md/encode.go:84-100). A `bip32.Path` re-serialization is a different bitstream. Worse, `resolveOriginPath` (expand.go:116-144) applies a **deliberate Go divergence**: for an elided shared path it falls back to `canonicalOrigin(tree)` (l.140-142, self-documented as "the deliberate Go divergence from Rust" at l.75-80). Rust `expand_per_at_n` (canonicalize.rs:437-455) does NOT do this — it resolves origin from `path_decl.paths` only and raises `MissingExplicitOrigin` when empty; it consults `canonical_origin` solely for the error gate (l.452), never as the path VALUE.
- **Why it matters:** If the port builds the preimage from `md.ExpandWalletPolicy(...).OriginPath` (the obvious reuse), the origin-path bits diverge from Rust → wrong id. For the 4 T6a-1 single-sig goldens (all explicit-origin, full-key), the resolved VALUES coincide, so the toolkit differential would still flush the bit-serialization error out — but the port is general (FuzzWalletPolicyId feeds random decoded descriptors, Task 4) and the divergence becomes a live wrong-answer on any elided-origin or in-band-vs-raw path. The fork already HAS the right primitives — `bitWriter` (bits.go:102-149), `reEmitBits` (bits.go:156, "port of bitstream.rs:220-230"), `writeVarint`/`writeOriginPath`/`writeUseSitePath`/`writeNode` (encode.go:51,89,133,158) — so the correct implementation is cheap; the trap is purely "don't reuse the display accessor."
- **Fix:** Add to PLAN Task 2W Step 3 an explicit directive: the preimage MUST mirror Rust `expand_per_at_n` (canonicalize.rs:420-474) operating on the descriptor's RAW `originPath`/`useSitePath` component form and re-emit via `writeOriginPath`/`writeUseSitePath` — NOT via the GUI `ExpandWalletPolicy`/`ExpandedKey` (which converts to `bip32.Path` in-band hardening AND applies the `canonicalOrigin` fallback Rust does not). Either build a private `expandPerAtN`-equivalent inside `walletpolicyid.go`, or refactor the raw-form expansion out of the display path; canonicalize a clone first (identity.rs:175-176). Add a fuzz/differential case that exercises an elided-shared-origin descriptor so the divergence is provably caught, not merely avoided by the explicit-origin goldens.

### MINOR (non-blocking)

**M3 — Pin the ready-made Rust golden id, not just a toolkit-captured stub.** identity.rs:547-550 ships the full 16-byte cell-7 wpkh id `6650b980 3b3c6621 0140540d a8d765a0` (with the byte-by-byte preimage hex at :540-543). PLAN Task 2W Step 1 should cite this as a direct, authoritative full-id pin for the wpkh golden (stronger and SHA-stable vs re-deriving from a toolkit run). Note this golden uses the test's `deterministic_xpub` (chain code `0x11`×32 ‖ `02` ‖ `0x22`×32), not a real abandon-seed xpub — so it is a preimage-construction pin, complementary to the abandon-seed toolkit differential.

**M4 — Pin the FULL 16-byte id in the differential, not only the 4-byte stub.** PLAN Task 2W Step 1 says "capture from the toolkit, e.g. the engraved mk1 stub = `WalletPolicyId[0:4]` — pin it." A 4-byte stub catches a structural omission (I1) probabilistically-certainly, but pinning the full `compute_wallet_policy_id` 16-byte id for each of the 4 goldens is strictly stronger and costs nothing (the toolkit/Rust expose `as_bytes()`). Recommend asserting the full id.

**M5 — State explicitly that presence-significance + stability gates do NOT catch the I1 omission.** PLAN Task 2W Step 1 lists presence-significance (identity.rs:610-617) and stability (identity.rs:572-605) as if they corroborate byte-fidelity. They are necessary but insufficient: both pass under a `record_bytes`-omitting preimage. The doc should mark the toolkit/Rust-golden differential (M3/M4) as THE byte-fidelity gate and the other two as property checks only.

---

## No-drift check (already-R1-GREEN parts) — CLEAN
The addition does not appear to perturb the already-GREEN remainder:
- **EncodeSingleSig / 4 AST shapes / ScriptKind:** confirmed the Go enum is `ScriptWpkh, ScriptPkh, ScriptSh, ScriptWsh, ScriptTr` (md/md.go:1166-1170) with NO `ScriptShWpkh` — PLAN Task 2 R0-M2's "APPEND `ScriptShWpkh` after `ScriptTr`" remains accurate and untouched by the WalletPolicyId addition.
- **EncodeMS1 recipe:** confirmed prefix `0x00`/`msPrefixEntr` (codex32/mspayload.go:9), id "entr" (mspayload.go:6-7), `NewSeed` present (codex32.go:279), `DecodeMS1` present (mspayload.go:34), no `EncodeMS1` — Task 1 unaffected.
- **Chunked output / DecodeChunks / ExpandWalletPolicyChunks:** present (expand.go:102) — Task 0/2 unaffected.
- **Comparator (Task 3):** the added stub-binding check (`mk1.policy_id_stub == WalletPolicyIDStub(md1)`) composes onto the existing fp/xpub/path/md1/ms1-entropy set without altering it; matches the toolkit's own self-check (recon Q3: bundle.rs:2157-2192 recomputes `compute_wallet_policy_id(md1)[..4]` and asserts membership in `mk1.policy_id_stubs`). Sound and sufficient as a bundle-integrity check.
- **T6a-1 stays headless-only:** the port + comparator are pure functions; the stub-setting/warning-drop are explicitly fenced to T6a-2. Confirmed no GUI symbol enters the T6a-1 plan.
- **Scrub schedule / typed-only seed / mainnet-only / picker:** untouched by this delta (GUI-tier, T6a-2).

---

## Required to reach GREEN
Fold **I1** (correct the preimage description in SPEC l.25 + PLAN Task 2W to include `record_bytes`) and **I2** (PLAN Task 2W Step 3: mandate raw-form `expand_per_at_n` mirror, forbid reusing the display `ExpandWalletPolicy`, add an elided-origin differential case). Minors M3-M5 are recommended in the same fold (they harden the gate that backstops I1). Re-dispatch after the fold (folds can drift). No code before GREEN.
