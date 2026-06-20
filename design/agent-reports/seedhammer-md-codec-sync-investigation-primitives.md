# SeedHammer fork md-codec sync — template-engraving primitives investigation

Date: 2026-06-20
Mode: READ-ONLY recon (multi-agent). No source modified.

Repos / pins:
- Fork (Go/TinyGo): `/scratch/code/shibboleth/seedhammer` @ `39cb5cf`
- Rust reference: `/scratch/code/shibboleth/descriptor-mnemonic` @ `54dd765` (md-codec v0.37.0)
- mk-cli selector: `/scratch/code/shibboleth/mnemonic-key` @ `1279ef9`
- Toolkit mutations: `/scratch/code/shibboleth/mnemonic-toolkit` @ `6de53879`

---

## HEADLINE VERDICT (load-bearing correctness question)

**The fork ALREADY has a standalone Go tree serializer (`writeNode`, `md/encode.go:159`),
and it is BYTE-FAITHFUL to Rust `tree::write_node` (`tree.rs:79-176`) across every shape
the fork supports.** It is already exercised in production by `WalletPolicyId`
(`md/walletpolicyid.go:42`). Therefore the `WalletDescriptorTemplateId` port is NOT
net-new tree serialization — it reuses the existing, already-correct `writeNode`. The
net-new surface is small: a thin `WalletDescriptorTemplateId` wrapper (use-site bits ‖
writeNode ‖ UseSitePathOverrides-TLV bits), a one-line `isWalletPolicy` predicate, and a
form-aware selector to rewire `bundle/verify.go:116`.

YES — the fork's existing tree-encoding is byte-faithful to Rust `tree::write_node`.
(Detailed shape-by-shape proof in §2.)

---

## 1. Primitives the fork must ADD for template engraving

### (a) Go port of `compute_wallet_descriptor_template_id`  — NET-NEW (thin wrapper)

Rust source: `identity.rs:71-104`. Preimage =
`SHA-256( use_site_path bits ‖ tree::write_node(tree) bits ‖ UseSitePathOverrides-TLV bits )[0..16]`.
Excludes header, origin-path-decl, Fingerprints TLV, HRP, BCH checksum → invariant to
origin-path / fingerprint changes.

Building blocks ALREADY in the fork (all reusable, all confirmed byte-faithful — see §2/agent-1):
- `writeNode(&w, tree, kiw)` — `md/encode.go:159` (the tree bits)
- `writeUseSitePath(&w, us)` — `md/encode.go:134` (the use-site bits)
- `kiw(n)` — `md/encode.go:34`
- `writeVarint` — `md/encode.go:52` (for the TLV length prefix)
- `reEmitBits` — `md/bits.go:156` (re-emit override sub-bitstream into the outer writer)
- `sha256First16` — `md/identity.go:20`
- `tlvUseSitePathOverrides = 0x00` — `md/md.go:495` (matches Rust `TLV_USE_SITE_PATH_OVERRIDES`,
  written as a 5-bit tag at `identity.rs:87`; fork TLV tag width is 5 bits, `md/encode.go:344`)

Net-new code: a ~30-40 line `WalletDescriptorTemplateId(d *descriptor) ([16]byte, error)` that
mirrors `identity.rs:74-103`:
  1. `width := kiw(d.???)` — see kiw-source note below.
  2. `d.useSite` → `writeUseSitePath`.
  3. `writeNode(&w, d.tree, width)`.
  4. If `d.tlv.useSitePresent` (the Go analog of Rust `tlv.use_site_path_overrides.is_some()`):
     build a sub-bitstream `for each (idx, path): write idx@kiw ‖ writeUseSitePath(path)`,
     then emit `tag(5b)=0x00 ‖ varint(sub.bitLen()) ‖ reEmitBits(sub)` into the outer writer.
  5. `sha256First16(w.intoBytes())`.

NOTE the Rust override re-emit (`identity.rs:90-97`) reads the sub-payload back through a
`BitReader` in ≤8-bit chunks and re-`write_bits` into `w`. The fork's `reEmitBits`
(`md/bits.go:156`) does the equivalent MSB-first re-emit in one call — confirmed equivalent by
agent-1. A direct `reEmitBits(&w, sub.intoBytes(), sub.bitLen())` is the faithful Go form.

kiw-SOURCE SUBTLETY (must get right): Rust `identity.rs:76` uses `d.key_index_width()` which is
`ceil(log2(descriptor.n))`. The fork's existing `WalletPolicyId` uses `kiw(dc.pathDecl.n)`
(`md/walletpolicyid.go:37`). Post-canonicalize these are guaranteed equal (encode-path guards
`errPathDeclNMismatch`, `md/encode.go:401`). For the WDT-Id port, follow the WalletPolicyId
precedent (canonicalize first if matching WalletPolicyId's contract; OR hash the descriptor
as-decoded if matching the Rust WDT-Id contract — see open question below). Either way compute
kiw from the SAME n the rest of the preimage uses.

OPEN QUESTION for the plan phase (not a divergence, a spec-contract choice): Rust
`compute_wallet_descriptor_template_id` does NOT canonicalize its input (unlike
`compute_wallet_policy_id`, which clones+canonicalizes at `identity.rs:173-177`). The Go port
must match Rust here: WDT-Id hashes the descriptor's tree/use-site AS GIVEN (no canonicalize),
relying on the wire already being canonical post-decode. Confirm against the decode invariant
(decoded descriptors are canonical) before locking. This is a behavioural-parity item, NOT a
byte-encoding divergence.

### (b) Go `is_wallet_policy()` predicate  — NET-NEW (one line)

Rust source: `encode.rs:50-52` →
`matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())`.

Fork building block: the decoded `tlvSection` carries `pubPresent bool` + `pubkeys []idxPub`
(`md/md.go:528-529`), set from the wire at `md/md.go:603`. So the Go predicate is:

```go
func (d *descriptor) isWalletPolicy() bool { return d.tlv.pubPresent && len(d.tlv.pubkeys) > 0 }
```

CONFIRMED: no `isWalletPolicy` / wallet-policy-mode predicate exists anywhere in the fork today
(grep over all `*.go`, NONE FOUND). This is net-new but trivial.

IMPORTANT mode-asymmetry to record: the fork's md1 ENCODERS hard-force `pubPresent: true`
(`md/encode_singlesig.go:73`, `md/encode_multisig.go:147`) — i.e. anything the FORK emits is
keyed wallet-policy mode and would return `isWalletPolicy()==true`. But DECODE reads `pubPresent`
off the wire (`md/md.go:603`), so an INGESTED keyless template md1 decodes with
`pubPresent==false` → `isWalletPolicy()==false`. The selector MUST dispatch on the predicate of
the DECODED ingested descriptor, never on an encoder-built one. This matches mk-cli exactly
(`mk-cli/src/cmd/mod.rs:73` decodes the supplied md1 string first, then branches).

### (c) Form-aware binding-stub selector to rewire `bundle/verify.go:116`  — NET-NEW (small)

Reference design = mk-cli `derive_stub_from_md1` (`mk-cli/src/cmd/mod.rs:72-82`):
```
decode md1 → if is_wallet_policy() { WalletPolicyId[0:4] } else { WalletDescriptorTemplateId[0:4] }
```
Current fork state (`bundle/verify.go:109-126`, `checkStubBinding`): unconditionally calls
`md.WalletPolicyIDStubChunks(b.MD1)` (`bundle/verify.go:116`) — i.e. it ALWAYS derives the
WalletPolicyId stub, even for a keyless template md1. For a template bundle this would compute the
WRONG stub (WalletPolicyId of a keyless descriptor ≠ the toolkit-emitted WalletDescriptorTemplateId
stub), so on-device verify would REJECT a legitimate template key-card binding (mis-binding /
false-negative). This is the security-relevant rewire.

Net-new: a form-aware `WalletBindingStubChunks(strs []string) ([4]byte, error)` (Go analog of
`derive_stub_from_md1`) that reassembles, branches on `d.isWalletPolicy()`, and returns
`WalletPolicyId[:4]` or `WalletDescriptorTemplateId[:4]`. Rewire `bundle/verify.go:116` to call it.
Existing keyed callers (`gui/multisig_derive.go:42`, `gui/singlesig_derive.go:67`,
`md/encode_multisig.go:158`) emit keyed wallet-policy cards and can stay on `WalletPolicyIDStub*`
(their inputs are always pubPresent=true), but the verify path must become form-aware to match the
toolkit/mk-cli identity contract.

---

## 2. CRITICAL — tree-encode fidelity (Go writeNode vs Rust write_node), shape-by-shape

The fork has a STANDALONE tree serializer: `writeNode` at `md/encode.go:159-232`, the faithful
inverse of the shipped decoder `readNodeDepth` (`md/md.go:330-490`). It is the exact Go analog of
Rust `tree::write_node` (`tree.rs:79-176`). Both write the 6-bit TAG first on every arm, then the
body. Shape-by-shape byte comparison:

| Shape | Rust (tree.rs) | Go (encode.go) | Bytes match? |
|---|---|---|---|
| TAG (every node) | `node.tag.write(w)` → 6-bit primary, no ext in v0.30 (`tag.rs:140-146`) | `writeTag` → `w.write(tag,6)` (`encode.go:46`) | YES — all 36 tag codes 0x00..0x23 identical (Rust `tag.rs:100-135` vs Go `md.go:40-75`) |
| KeyArg | `write_bits(index, kiw)` (`tree.rs:82-84`) | `w.write(index, kiw)` (`encode.go:162-163`) | YES (kiw=0 emits 0 bits both sides) |
| Children (1/2/3-ary wrappers, and/or/andor, TapTree) | recurse each child, no length prefix (`tree.rs:85-89`) | recurse each child (`encode.go:164-169`) | YES — child ordering preserved by slice order |
| Variable (Thresh only) | `(k-1)@5b ‖ (n-1)@5b ‖ children`; guards k,n∈1..32 & k≤n (`tree.rs:90-114`) | same (`encode.go:170-187`) | YES |
| MultiKeys (Multi/SortedMulti/MultiA/SortedMultiA) | `(k-1)@5b ‖ (n-1)@5b ‖ N×index@kiw`; same guards (`tree.rs:115-139`) | same (`encode.go:188-203`) | YES — bit-packing `5bits(k-1)\|5bits(N-1)` identical; raw kiw-width indices, not child nodes |
| Tr | `is_nums@1 ‖ [key_index@kiw iff !is_nums] ‖ has_tree@1 ‖ [subtree]` (`tree.rs:140-159`) | same (`encode.go:204-215`) | YES — NUMS suppresses the kiw field identically |
| Timelock (After/Older) | u32 @32b (`tree.rs:160-162`) | `w.write(uint32, 32)` (`encode.go:216-217`) | YES |
| Hash256 (Sha256/Hash256) | 32×byte@8b (`tree.rs:163-167`) | 32×byte@8b (`encode.go:218-221`) | YES |
| Hash160 (Hash160/Ripemd160/RawPkH) | 20×byte@8b (`tree.rs:168-172`) | 20×byte@8b (`encode.go:222-225`) | YES |
| Empty (False/True) | no body (`tree.rs:173`) | no body (`encode.go:226-227`) | YES |

Key sub-checks:
- **Tag codes**: byte-identical across all 36 operators. Rust `Tag::codes()` (`tag.rs:98-137`) ↔
  Go const block (`md.go:40-75`): Wpkh=0x00 … True=0x23, in the same order. Both write the primary
  in 6 bits; v0.30 allocates no extension subcodes, so neither side ever emits the 0x3F prefix.
- **k/N bit-packing**: both encode `(k-1)` then `(n-1)` each in 5 bits, MSB-first, for BOTH the
  Thresh (Variable) and Multi-family (MultiKeys) bodies. Identical.
- **kiw / key-index width**: both `ceil(log2(n))`, clamp 0 at n≤1 (Rust `encode.rs:37-41`,
  Go `encode.go:34-39`). At n=1 → kiw=0 → key-arg / multi indices / Tr key_index emit ZERO bits on
  both sides. (BitWriter `write(_, 0)` is a no-op on both — confirmed by agent-1.)
- **Child ordering**: both iterate children/indices in slice order; recursion is left-to-right.
- **NUMS handling**: `is_nums=true` suppresses the key_index field on both sides identically.

(Supporting-writer / bit-ordering fidelity for the use-site + TLV portions of the WDT-Id preimage:
see agent-1 findings folded in below.)

### Tree-encode fidelity verdict: **YES — byte-faithful.**
Same tag values, same `5bits(k-1)|5bits(N-1)` packing, same kiw key-index width, same raw-index
(kiw) handling for multi-family, same child ordering, same NUMS/has_tree bits, same hash/timelock
widths. No divergence found in any shape the fork supports.

---

## 3. Template strip/mutation semantics (toolkit synthesize.rs)

(Findings from agent-2, reading `mnemonic-toolkit` synthesize.rs @ `6de53879`. Note: the file
resolves under `crates/mnemonic-toolkit/src/synthesize.rs`; line numbers below are that file's.)

`synthesize_template_descriptor()` (≈ lines 1158-1283) clones the keyed descriptor and applies
exactly FOUR template-ifying mutations (toolkit SPEC §3.2):
1. `template.tlv.pubkeys = None`        (≈ L1182) — strip all 65-byte xpubs.
2. `template.tlv.fingerprints = None`   (≈ L1183) — strip all 4-byte fingerprints.
3. CONDITIONAL origin elision (≈ L1195-1196): IFF the wrapper has a `canonical_origin`
   (pkh/wpkh/tr-keypath/wsh(multi|sortedmulti)/sh(wsh(...))), set
   `path_decl.paths = Shared(OriginPath{ components: vec![] })` (empty). For non-canonical
   (general-policy) wrappers, KEEP the source origins verbatim (≈ L1198).
4. No `is_wallet_policy()` assert on the way out (template is keyless by construction).
NOT mutated: `use_site_path`, `use_site_path_overrides`, `origin_path_overrides`, `path_decl.n`,
the `tree` (threshold/shape/sorted-vs-multi preserved), and any unknown TLV entries.

`template_admissible()` (≈ L1113-1122) is the pre-mutation shape gate on the KEYED input:
- n==1: accept only the three canonical-elidable single-sig types (pkh→BIP44, wpkh→BIP84,
  tr-keypath-only→BIP86); reject everything else.
- n≥2: accept any shape that renders via `to_miniscript_descriptor` AND has no hardened use-site;
  reject `tr(sortedmulti_a)`, sortedmulti-in-combinator, hardened use-site.
Refusal → `ToolkitError::TemplateFormUnsupportedShape`.

Host-side vs on-device: synthesize is PURELY HOST-SIDE (toolkit `bundle --md1-form=template`,
called from `synthesize_descriptor` only when `md1_form.is_template()`). The engraver/firmware
RECEIVES the already-stripped keyless template md1 (`md_codec::chunk::split(&template)`); it does
NOT strip on-device. This matches the recon's locked decision DD1 = ingest-supplied, not
on-device emit.

Firmware replication requirement: **NONE.** Because the device only ingests an already-stripped
template, it never needs to replicate pubkey/fingerprint stripping or origin elision. The device's
only jobs are: decode/validate the keyless md1, compute `WalletDescriptorTemplateId` over it, and
use its top-4 bytes as the binding stub. The wire bytes are the source of truth.

KEY LINKAGE confirmed: after stripping, `tlv.pubkeys == None` → `is_wallet_policy()` returns
FALSE (toolkit pins this with `assert!(!decoded.is_wallet_policy(), "template is keyless")`,
≈ L2414). So a properly-synthesized template md1 will, on ingestion in the fork, decode with
`pubPresent == false` and route the form-aware selector down the WDT-Id branch — exactly the
intended behaviour. Conversely, the fork's OWN encoders force `pubPresent: true`, so they never
emit a template; the fork only ever obtains a template by ingesting a toolkit-produced one.

`synthesize.rs` itself does compute identities (host-side): `compute_wallet_descriptor_template_id`
on the mutated template (≈ L1206-1207, the template stub source) and `WalletPolicyId` variants for
the keyed disambiguator. These are the host counterparts the fork's on-device selector must agree
with byte-for-byte.

---

## 4. Supporting-writer & bit-ordering fidelity (use-site / varint / origin / bitwriter / kiw)

(Findings from agent-1. ALL items MATCH — zero byte divergence. The WDT-Id preimage uses the
use-site writer, the tree writer, varint, reEmitBits and the bitwriter; every one is byte-faithful.)

1. BitWriter MSB-first + into_bytes zero-pad + count==0 no-op — MATCH.
   Rust `bitstream.rs:29-83` (`write_bits`/`into_bytes`/`bit_len`, `re_emit_bits` 220-230) ↔
   Go `bits.go:102-149` (`write`/`intoBytes`/`bitLen`, `reEmitBits` 156-172). Both pack MSB-first,
   both zero-pad the final byte's low bits, both treat a 0-count write as a no-op (matters at
   kiw=0). `reEmitBits` re-emits exactly `bitLen` bits with no inserted padding on both sides.
2. use_site_path — MATCH. Rust `use_site_path.rs:80-96` (+ `Alternative::write` 28-32) ↔
   Go `encode.go:134-151` (+ `writeAlternative` 129-132): same has-multipath bit, same
   alt_count = (len - 2) in 3 bits, same per-alt (hardened bit + varint), same trailing
   wildcard_hardened bit, same ordering, same 2..9 bound.
3. varint (LP4-ext) — MATCH. Rust `varint.rs:15-42` ↔ Go `encode.go:52-74`: same 4-bit L,
   same 14-bit threshold, same extension (L=15, lHigh@4, low@14, high@lHigh), same value==0 →
   bitsNeeded 0 special case.
4. origin_path — MATCH. Rust `origin_path.rs:28-131` ↔ Go `encode.go:85-125`: same 4-bit depth,
   same per-component (hardened + varint), same `(n-1)@5b` path-decl, same max-depth 15.
5. kiw — MATCH + invariant safe. Rust `encode.rs:37-41` (= `ceil(log2(n))`, clamp 0 at n≤1) ↔
   Go `encode.go:34-39`. Rust WDT-Id uses `d.key_index_width()` (descriptor.n); the existing Go
   WalletPolicyId uses `kiw(dc.pathDecl.n)`. The encode path guards `dc.pathDecl.n == dc.n`
   (`errPathDeclNMismatch`, `encode.go:401`) and `canonicalize` keeps them in lockstep, so a Go
   WDT-Id port computing kiw from pathDecl.n is safe and equal. (Plan note: keep this guard /
   invariant in the WDT-Id port, or compute kiw from descriptor.n directly to match Rust exactly.)

---

## CLOSING VERDICT

### Net-new primitives + size estimate
1. `WalletDescriptorTemplateId(d) ([16]byte, error)` + a `*Chunks([]string)` wrapper — NET-NEW
   but a THIN composition of existing, already-byte-faithful pieces (`writeUseSitePath`,
   `writeNode`, `writeVarint`, `reEmitBits`, `sha256First16`, TLV tag 0x00). ~40-60 LOC incl.
   the override-TLV sub-bitstream branch. NOT net-new tree serialization.
2. `isWalletPolicy()` predicate = `d.tlv.pubPresent && len(d.tlv.pubkeys) > 0` — NET-NEW, ~1-3 LOC
   (mirrors Rust `encode.rs:50-52`; `pubPresent`/`pubkeys` already on the decoded `tlvSection`).
3. Form-aware binding-stub selector (Go analog of mk-cli `derive_stub_from_md1`) + rewire of
   `bundle/verify.go:116` from the unconditional `WalletPolicyIDStubChunks` to a branch on
   `isWalletPolicy()`. ~15-25 LOC + the one-line call-site swap. This is the security-relevant fix:
   today verify ALWAYS uses the WalletPolicyId stub, so it would mis-reject a legitimate keyless
   template key-card binding.

Total net-new on-device surface: roughly 60-90 LOC of straightforward composition over existing
correct primitives, plus tests. No new serialization machinery, no canonicalize changes, no
on-device stripping (DD1: ingest-only).

### Byte-faithfulness of the existing tree-encoding to Rust `tree::write_node`: **YES.**
The fork's `writeNode` (`md/encode.go:159-232`) emits byte-identical output to Rust
`tree::write_node` (`tree.rs:79-176`) for every shape the fork supports: identical tag values
(all 36 codes 0x00..0x23), identical `5bits(k-1)|5bits(N-1)` packing for both Thresh and the
multi-family, identical raw kiw-width key-index handling, identical child ordering, identical
Tr `is_nums`/`has_tree` framing and NUMS kiw-suppression, identical hash/timelock widths. All
supporting writers feeding the WDT-Id preimage (use-site, varint, origin, bitwriter MSB-order +
zero-pad, reEmitBits, kiw) are likewise byte-faithful (agent-1, zero divergence). The WDT-Id port
therefore reuses a verified-correct serializer; the load-bearing correctness risk for this cycle
is satisfied. No divergence found that would yield a different hash / wrong template ID / mis-bound
card.

One behavioural-parity item to LOCK in the plan (NOT a byte divergence): Rust
`compute_wallet_descriptor_template_id` does NOT canonicalize its input (contrast
`compute_wallet_policy_id`, which does). The Go WDT-Id port must match — hash the descriptor's
tree/use-site as-decoded, relying on the decode-side canonical invariant. Confirm and pin this in
the R0 gate.
