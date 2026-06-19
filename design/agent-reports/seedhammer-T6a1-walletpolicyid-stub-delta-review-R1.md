# R1 DELTA-REVIEW (RE-REVIEW) — T6a WalletPolicyId port + mk1 policy-bound stub (wire-fidelity fold)

**Reviewer:** opus architect (focused delta re-review, R1, after the R0 fold). **Date:** 2026-06-19.
**Scope:** ONLY the wire-fidelity fold to the two normative docs — the `md.WalletPolicyId` preimage prose (SPEC Phase-A `md.WalletPolicyId` bullet l.25; PLAN Task 2W l.60-73), the I2 display-accessor-trap mitigation, and the M3/M4/M5 gate hardening. Confirm the prior delta-R0's 2I/3m are CLOSED, verify no drift, check for anything new. The remainder of both docs is R1-GREEN and NOT re-opened here.

**Documents re-reviewed:**
- `design/SPEC_seedhammer_T6a_singlesig_flagship.md`
- `design/IMPLEMENTATION_PLAN_seedhammer_T6a1_headless.md`
- `design/agent-reports/seedhammer-T6a1-walletpolicyid-stub-delta-review-R0.md` (the prior NOT-GREEN this verifies folded)

**Authoritative sources re-verified (NOT the docs' prose):**
- RUST md-codec @ pinned `c85cd49` (v0.36.0): confirmed `git rev-parse HEAD == c85cd498c690d9f91c7884234cf25d0c39264608`, `crates/md-codec/Cargo.toml version = "0.36.0"`. `src/identity.rs` (the record layout + golden), `src/canonicalize.rs` (`expand_per_at_n`).
- GO fork @ pinned `e4013a8`: confirmed `git rev-parse HEAD == e4013a88011284c71f6da1b5629555bdc52c7e88`. `md/expand.go`, `md/encode.go`, `md/bits.go`, `md/md.go`, `md/canonicalize.go`.

---

## VERDICT: GREEN — 0 Critical / 0 Important / 0 Minor

Both R0 Important findings (I1 preimage completeness, I2 display-accessor trap) are CLOSED with byte-exact source fidelity, and all three Minors (M3 Rust golden pin, M4 full-16-byte differential, M5 gate-non-coverage note) are CLOSED. The folded prose now matches `identity.rs:172-240` field-for-field, the field ORDER and varint/bit-packing are correct, and the 78-byte (`1+8+4+65`) record is pinned in both docs. No drift introduced; the previously-confirmed rulings (a)-(d) and the broader R1-GREEN remainder are unperturbed. **The T6a-1 plan is CLEARED for single-implementer TDD.**

One non-blocking observation (recon-doc staleness) is recorded below; it is NOT a finding because the recon is a historical grounding artifact, not a normative implementation-driving doc, and both normative docs are correct.

---

## I1 — Preimage now includes the full per-@N `record_bytes` — **CLOSED**

**Verified against `identity.rs:172-240` (the function), :460-465 (the in-source golden comment), :515-553 (the byte-by-byte golden test).**

The authoritative per-record layout (`identity.rs:187-228`) is, in exact ORDER:
1. `presence_byte` (`l.219,221`) = `((fp_present as u8) | ((xpub_present as u8) << 1)) & 0b0000_0011` — bit 0 = fp, bit 1 = xpub, bits 2..7 masked to 0.
2. `record_bytes` (`l.206-211,222`) = `BitWriter ⇐ write_varint(path_bit_len) ‖ re_emit_bits(path_bytes, path_bit_len) ‖ write_varint(use_site_bit_len) ‖ re_emit_bits(us_bytes, use_site_bit_len)`, then `.into_bytes()` (single byte-boundary zero-pad). Lengths are in **BITS** (`l.194,200`).
3. `fp[4]?` (`l.223-225`) — appended iff `fp_present`.
4. `xpub[65]?` (`l.226-228`) — appended iff `xpub_present`.
The whole hash input (`l.232-235`) = `canonical_template_tree_bytes (write_node, l.180-182) ‖ Σ_@N record`, then `SHA-256 → [0..16]` (`l.236-238`).

The in-source golden (`identity.rs:463`) is explicit: `record total = 1 + 8 + 4 + 65 = 78 bytes` (presence 1 ‖ record_bytes 8 ‖ fp 4 ‖ xpub 65); the record_bytes is 60 bits → 8 bytes after pad (`l.457-458,513,518`).

**SPEC l.25 now reads** (verbatim): "each record = `presence_byte (fp_present | xpub_present<<1) ‖ record_bytes ‖ fp[4]? ‖ xpub[65]?` and `record_bytes = into_bytes(varint(path_bit_len)‖path_bits‖varint(use_site_bit_len)‖use_site_bits)` (`identity.rs:188-228`) — a fully-present record is `1+8+4+65 = 78` bytes (the in-source golden), NOT 70; the 8-byte path/use-site `record_bytes` must NOT be dropped. SHA-256 → [:16]."

**PLAN Task 2W l.66 now reads** (verbatim): "SHA-256 over `placeholder-form tree bytes ‖ (per-@N: presence_byte ‖ record_bytes ‖ fp[4]? ‖ xpub[65]?)`, where `presence_byte = fp_present | (xpub_present<<1)` and `record_bytes = into_bytes(varint(path_bit_len)‖path_bits‖varint(use_site_bit_len)‖use_site_bits)` (`identity.rs:188-228`). A fully-present record is `1+8+4+65 = 78` bytes (the in-source golden), NOT 70 — the 8-byte path/use-site `record_bytes` is the part the earlier prose dropped. Truncate SHA-256 → [:16]."

**Assessment — CLOSED.** Both docs now state the full record, the correct field ORDER (presence ‖ record_bytes ‖ fp ‖ xpub), the correct varint/bit-packing (`varint(path_bit_len)‖path_bits‖varint(use_site_bit_len)‖use_site_bits`, lengths in BITS), and the pinned 78-byte size with the explicit "NOT 70 / do not drop record_bytes" callout. This is byte-faithful to `identity.rs:188-228` and the :463 golden. No residual order or encoding error. (The SPEC adds the correct distinction from `Md1EncodingId` `identity.rs:39-45` and `WalletDescriptorTemplateId` `identity.rs:71-104` — confirmed correct.)

## I2 — Display-accessor trap flagged + forbidden; elided-origin differential added — **CLOSED**

**Verified against the Go display accessor and the Rust resolver.**

The trap is real and source-confirmed:
- Go `ExpandedKey.OriginPath` is a `bip32.Path` with hardening encoded IN-BAND as `value + HardenedKeyStart` (`expand.go:53-58`; `componentsToPath` `expand.go:148-160`) — NOT the raw `originPath{components:[{hardened,value}]}` form that Rust's `e.origin_path.write()` (`identity.rs:193`) serializes via the raw component writers.
- `resolveOriginPath` (`expand.go:116-144`) applies a `canonicalOrigin(tree)` fallback for an elided shared path (`l.137-142`), self-documented as "the deliberate Go divergence from Rust" (`l.75-80`).
- Rust `expand_per_at_n` (`canonicalize.rs:420-474`) resolves origin from `origin_path_overrides[idx]` else `path_decl.paths` ONLY (`l.437-444`); it consults `canonical_origin` SOLELY for the `MissingExplicitOrigin` error gate (`l.450-455`), NEVER as the path VALUE. Confirmed: there is no `canonical_origin`-as-value path in the Rust resolver.

**PLAN Task 2W now flags and forbids this** (verbatim l.67): "**R0-I2 — do NOT reuse the display accessor.** `md.ExpandWalletPolicy`/`ExpandedKey` (`md/expand.go`) is a DISPLAY path: it converts origin to a `bip32.Path` (in-band `+HardenedKeyStart` hardening) and applies a `canonicalOrigin` fallback that Rust `expand_per_at_n` does NOT apply to the path value (a deliberate Go divergence, `expand.go:75-80,140-142`). Reusing it for the id would diverge byte-wise. Instead, build `record_bytes` by mirroring `expand_per_at_n` on the RAW component form using the in-tree bit machinery (`bitWriter`/`reEmitBits`/`writeOriginPath`/`writeUseSitePath` from #10a) — the same primitives `encodePayload` uses." Step 3 (l.71) repeats: "incl. `record_bytes` via the raw bit machinery, NOT the display accessor."

**Elided-origin differential case ADDED** (verbatim Task 2W Step 1, l.69): "**Encoding-stability + elided-origin (R0-I2):** origin-elided vs explicit-origin forms of the same wallet yield the SAME id (`identity.rs:572-605`) — **add the elided-origin case** (proves `record_bytes` is built from the resolved path, exposing any display-accessor/canonicalOrigin divergence)." This matches the Rust stability tests `walletpolicyid_stable_across_origin_elision` (`identity.rs:571-588`) and `_use_site_elision` (`:592-605`).

**"Any OTHER place the port could pick up the display form?" — checked, and the fix is sufficient.** The fork's ONLY per-@N resolvers today are `resolveOriginPath` (`expand.go:116`) and `resolveUseSite` (`expand.go:165`), and BOTH return the display form (`bip32.Path` / exported `UseSite`) with the divergent fallback. There is no pre-existing raw-component per-@N resolver, so the implementer cannot accidentally reach for a "raw" helper that secretly routes through the display path — any raw resolver must be written fresh. The raw primitives the plan names all exist and are the correct ones: `bitWriter.write/bitLen/intoBytes` (`bits.go:102,136,147`), `reEmitBits` (`bits.go:156`), `writeVarint` (`encode.go:51`), `writePathComponent` (`encode.go:84`), `writeOriginPath` (`encode.go:89`), `writeUseSitePath` (`encode.go:133`), `writeNode` (`encode.go:158`). The raw `originPath{components}` / `useSitePath` types (`md.go:190,265`) and the raw override/baseline slices (`d.tlv.originOverrides` `md.go:530`, `d.pathDecl.shared/divergent`) are all in-package and accessible to the in-`md` port. So the port can build a raw `expandPerAtN`-equivalent reading `originOverrides[idx]` else `pathDecl.shared/divergent[idx]` (mirroring `canonicalize.rs:437-444`) and emit via the raw writers — exactly as the plan mandates. The one subtlety worth flagging to the implementer is benign and already covered: the port must NOT replicate the Go-display `canonicalOrigin` value-fallback (`expand.go:137-142`); Rust raises `MissingExplicitOrigin` instead, which the 4 T6a-1 goldens (all explicit-origin) never trigger and which the new elided-origin differential will catch if mishandled. This is precisely what l.67/l.69 already say. No additional plan change required.

**Assessment — CLOSED.** The trap is explicitly named, the display accessor explicitly forbidden, the raw-machinery mirror mandated, and the elided-origin differential that would expose a divergence is added with the right source citation. No other display-form leakage vector exists.

## M3 — Rust golden id pinned — **CLOSED**
PLAN Task 2W Step 1 (l.69) pins: "`WalletPolicyId(cell_7_wpkh_full) == 6650b980 3b3c6621 0140540d a8d765a0` (`identity.rs:547-550`)." Verified against `identity.rs:547-550` byte-for-byte: `[0x66,0x50,0xb9,0x80, 0x3b,0x3c,0x66,0x21, 0x01,0x40,0x54,0x0d, 0xa8,0xd7,0x65,0xa0]` — exact match. The SPEC l.25 also pins the same id. (This is the `deterministic_xpub` preimage-construction pin, complementary to the abandon-seed toolkit differential — correctly characterized.)

## M4 — Full 16-byte id pinned in the differential — **CLOSED**
PLAN Task 2W Step 1 (l.69) now reads "pin the FULL 16-byte id, not just the 4-byte stub … `WalletPolicyId(decoded)` byte-equals the toolkit's `compute_wallet_policy_id` (capture the full 16 bytes + the engraved mk1 stub = `[0:4]`)." The 4-byte stub is now a derived projection of the pinned full id, not the primary assertion. CLOSED.

## M5 — Gate-non-coverage note added — **CLOSED**
PLAN Task 2W Step 1 (l.69) ends: "**(R0-M5) NOTE:** the presence-significance + stability gates PASS even if `record_bytes` is wrongly omitted — only the toolkit/golden differential (M3/M4) catches that, so it is the load-bearing gate." Verified correct against the Rust property tests: `walletpolicyid_template_only_differs_from_full_cell_7` (`identity.rs:609-618`) flips presence + drops fp/xpub regardless of record_bytes, and the two `_stable_across_*_elision` tests (`:571-605`) compare two forms that BOTH share the same (possibly-omitted) record_bytes construction — so neither catches an omission. The toolkit/Rust-golden differential is correctly marked load-bearing. CLOSED.

---

## No-drift re-confirmation — CLEAN

All prior-confirmed rulings still hold (re-verified, no change introduced by the fold):
- **Stub = `WalletPolicyId[..4]` (not the chunk-id).** SPEC l.25, PLAN l.64,66,73 consistently cite SPEC_mk §3.3 / audit-I1 and the STALE `mk-codec key_card.rs:27` doc; `WalletPolicyId` (`identity.rs:172-240`) distinct from `Md1EncodingId` (`identity.rs:39-45`). Unchanged.
- **T6a-1 port / T6a-2 stub-set split.** Port (`WalletPolicyId`/`WalletPolicyIDStub`) + comparator are headless → T6a-1 (PLAN Task 2W, Task 3). Stub-SETTING (`mk.Encode` with `Stubs:[WalletPolicyIDStub(md1)]`) + "Unbound Key Card" warning-drop fenced to T6a-2/GUI (SPEC Phase-B l.32; PLAN self-review l.106). No leakage. Unchanged.
- **Full-policy-only / template-OUT.** SPEC §OUT l.39 + I-6b l.77; full-policy-only is correct because `WalletPolicyId` is key-presence-significant (`identity.rs:609-618`). Unchanged.
- **Comparator stub-binding (I-6/I-6b).** SPEC l.26,76,77 + PLAN Task 3 (l.81,83): read-back `mk1.policy_id_stub == md.WalletPolicyIDStub(decoded md1)`, FAIL "stub mismatch". Composes onto the existing fp/xpub/path/md1/ms1-entropy set without altering it. Unchanged.
- **Broader R1-GREEN remainder unperturbed:** EncodeSingleSig chunked output (`split`, not `encodeMD1String`), 4 AST shapes + `ScriptShWpkh`-append (no renumber), EncodeMS1 recipe (`NewSeed("ms",0,"entr",'s',[0x00‖entropy])`), scrub schedule, typed-only seed, mainnet-only — none touched by this fold (the fold is confined to the `md.WalletPolicyId` bullet/Task and its gate). T6a-1 stays headless-only (port + comparator are pure functions; no GUI symbol enters the plan). Confirmed.

## Non-blocking observation (NOT a finding)
The recon grounding doc `design/agent-reports/seedhammer-T6-recon-bundle-composition-stub.md:13,26` still carries the OLD incomplete preimage prose ("`presence_byte = fp_present | xpub_present<<1` + fp[4] + xpub[65] when present", omitting `record_bytes`). This does NOT gate: the recon is a historical recon artifact, not a normative implementation-driving doc, and the two docs that DO drive the implementer (SPEC + PLAN) are now byte-correct. The R0 fix scope correctly targeted "SPEC l.25 + PLAN Task 2W"; the recon is the source the R0 finding cited as evidence of where the wrong prose originated, not a fix target. Optionally annotate the recon with a "superseded by R0-I1 fold; see SPEC l.25 / PLAN Task 2W for the full record" pointer to prevent a future reader re-importing the 70-byte form — but this is housekeeping, not a gate item.

---

## Conclusion
**VERDICT: GREEN (0C/0I/0m).** I1 CLOSED, I2 CLOSED, M3/M4/M5 CLOSED; no drift, nothing new. The wire-fidelity addition to the two docs is byte-faithful to `md-codec/identity.rs@c85cd49:172-240` and `canonicalize.rs:420-474`, the gate is correctly load-bearing on the full-16-byte toolkit/Rust-golden differential, and the display-accessor trap is named, forbidden, and differentially tested. **The T6a-1 (headless) implementation plan is CLEARED for single-implementer TDD** in a worktree, followed by the mandatory whole-diff adversarial exec review per the project standard. No code before this GREEN was permitted; this GREEN now lifts that gate for T6a-1.
