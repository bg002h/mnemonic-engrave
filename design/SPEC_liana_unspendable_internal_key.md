# SPEC — the Liana unspendable internal key (F-449)

**Status:** DRAFT r0, awaiting architect review.
**Owning cycle:** its own constellation cycle in `descriptor-mnemonic`, per F-449.
**Rust-primary:** this is normative codec behaviour. It lands in `md-codec`
**first**, with test vectors, and only then in the fork's Go port. The Go port
may never lead (project CLAUDE.md, Rust-primary rule).

## 0. What this delivers, and what it does not

An SH2 operator can compose a wallet under a `tr` wrapper whose internal key is
an **unspendable xpub built by Liana's own recipe**, so that Liana imports the
resulting descriptor.

Measured effect, from `design/IMPORTABILITY_composer_shapes.md` and the lens-5
evidence: the Liana-importable `tr` preset set goes from **1 of 6** to **3 of 6**
— `simple-timelocked-inheritance` (already works, its internal key is a real
key), plus `kofn-recovery` and `tiered-recovery`.

**Not in scope.** Making the other three tr presets import. Measured, they fire
Liana's own refusal classes for reasons that are not about our encoding:

| preset under `tr` | class that fires behind the NUMS one | why no encoding change helps |
| --- | --- | --- |
| `plain-multisig` | 3 `no locked path` | Liana requires a recovery path; a plain k-of-n has none |
| `hashlock-gated` | 4 `a hash lock` | Liana's policy model has no hash |
| `decaying-multisig` | 5 `an absolute lock`, then 7 `no unlocked path` | every tier is timelocked, and one lock is `after()` |

Those are not Liana wallets by shape. Their deliverable is the **verdict** —
telling the operator before the plates are cut — which the coordinator-compat
work (`DESIGN_coordinator_compatibility.md`, plans 1a/1b/3) already builds.
This spec must not weaken that refusal.

Also not in scope: the target-selection mode in the composer. That is a
separate, later cycle and this spec must not foreclose it.

## 1. Measured baseline

Every claim below was run against shipped code during the 2026-09-21 brainstorm.

| fact | measurement |
| --- | --- |
| the composer extracts a real internal key iff some path is a bare, unlocked, unhashed single key | `compose/mod.rs:292` `is_bare_single`; `compose/tr.rs:45` `is_nums: ik.is_none()` |
| a multi-key **primary** under `tr` emits `multi_a`, not `sortedmulti_a` | `md compose --wrapper tr --path 2of3 --path 2of3,older=26280`, RUN |
| `sortedmulti_a` appears only in the single-path bare-multi case | `md compose --wrapper tr --preset plain-multisig,2of3`, RUN |
| 5 of 17 measured `tr` shapes import into Liana today, all with a 1-of-1 unlocked primary | `IMPORTABILITY_composer_shapes.md`, counted mechanically |
| rewriting a NUMS tree with Liana's unspendable xpub makes 4 of 8 ACCEPT | `design/evidence/composer-fable-r0/fable-liana-parse-{in,out-v15}.jsonl`, variant `liana-unspendable-xpub`: ACCEPT for `preset-kofn-recovery-tr`, `preset-tiered-recovery-tr`, `same-seed-two-paths-tr`, `X19-tr-kofn-nums-older5`; REFUSE for the three hash shapes and `preset-decaying-multisig-tr` |
| v8.0 and v15.0 agree on all of it | `fable-liana-parse-out.jsonl` vs `fable-liana-parse-out-v15.jsonl` |

## 2. The recipe (normative)

Liana's own, at `liana/src/descriptors/analysis.rs:398-430`.

Given the taproot leaves' key expressions in **descriptor left-to-right order**:

1. Take each leaf key's **33-byte compressed public key** — for an xpub in
   `[origin]xpub.../<0;1>/*` form, that is bytes `[45..78]` of the base58-decoded
   78-byte payload, i.e. the pubkey of the xpub **itself**, not a derived child,
   and **after** the origin prefix is stripped.
2. Concatenate them in that order. **Not sorted. Not deduplicated.**
3. `chain_code = sha256(concatenation)`.
4. Build an extended public key with:
   - `public_key` = `0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`
     (the BIP-341 NUMS point `H`, compressed, `02` prefix)
   - `chain_code` from step 3
   - `depth = 0`, `parent_fingerprint = 00000000`, `child_number = 0`
   - version bytes copied from the leaf xpubs (mainnet `xpub` / testnet `tpub`)
5. Render it with `origin: None`, derivation paths `<0;1>`, unhardened wildcard —
   i.e. `xpub.../<0;1>/*`.

**VERIFIED INDEPENDENTLY.** This recipe was reimplemented from scratch in Python
during the brainstorm and reproduced the golden xpub **byte-for-byte for all four
shapes Liana accepted** (`preset-kofn-recovery-tr`, `preset-tiered-recovery-tr`,
`same-seed-two-paths-tr`, `X19-tr-kofn-nums-older5`). It is measured, not
transcribed from source comments.

**A property worth stating.** The chain code depends on the ordered multiset of
leaf pubkeys and **not** on the tree structure. `preset-kofn-recovery-tr` and
`preset-tiered-recovery-tr` have different trees over the same four keys in the
same order, and they share the same internal key — confirmed in the evidence.
This is harmless (the output key still commits to the tree through the taproot
tweak) but it is surprising, and a test must pin it so nobody "fixes" it.

**Only Liana's recipe imports into Liana.** Lens 5 measured that any other xpub
in that position is treated as a spendable key and, having no origin, is
`InvalidKey`. The abandoned bitcoin/bips PR #1746 recipe (sorted, deduplicated;
closed unmerged 2025-09-17) is a *different* xpub and does not import.

## 3. The wire kind

### 3a. Today

`tree.rs:43` — `Tag::Tr | is_nums(1) | [key_index(kiw) iff !is_nums] | has_tree(1) | [tree iff has_tree]`.

`is_nums` is a single bit; when set, no internal-key bits follow.

### 3b. Why this cannot be additive

- **A new bit after `is_nums` desyncs old decoders.** A v4 decoder would read it
  as `has_tree` and misparse the rest of the stream. That is the
  silent-wrong-addresses failure, not a clean one.
- **A TLV is worse.** F-417 makes unknown TLVs preserve-and-ignore, so an
  ignored kind would render raw `H` for a Liana wallet and derive a *different
  wallet's* addresses. This is precisely the trap F-449 names.
- **A reserved `key_index` sentinel does not fit.** `key_index_width =
  ceil(log2(n))` (`decode.rs:88`), so for n=4 the field is 2 bits with values
  0..3 and no spare. Widening the field is itself the wire change being avoided.

### 3c. The seam that works: the header version, which already fails closed

`header.rs:42` is an **exact-match** check — `version != WF_REDESIGN_VERSION` →
`Error::WireVersionMismatch`. So a version bump makes every existing decoder
refuse the new form **loudly**, with no misparse and no wrong addresses. That is
exactly the property F-449 asks for, and it already exists.

### 3d. The encoding

Add `Header::WF_UNSPENDABLE_VERSION = 5`.

At **version 5**:

```
Tag::Tr | is_nums(1) | [kind(1) iff is_nums] | [key_index(kiw) iff !is_nums] | has_tree(1) | [tree]
```

`kind`: `0` = the BIP-341 raw `H` point (what version 4 means), `1` = the Liana
unspendable xpub of §2.

At **version 4** the stream is unchanged and no `kind` bit is read or written.

**Rules:**

- The encoder emits the **minimum version that can express the tree**: version 5
  only when some `Body::Tr` carries `kind = 1`; otherwise version 4. Existing
  plates keep their bytes and keep meaning raw `H`.
- The decoder accepts `{4, 5}` and rejects everything else with
  `WireVersionMismatch`, unchanged in spirit from today.
- At version 4, a `Body::Tr` decodes with `kind = 0` implicitly.

**Why one bit and not two.** With a 1-bit field both values are defined, so a
v5 decoder can never meet an undefined kind — an entire error path does not
exist. A future third recipe would require version 6, and v5 decoders would
reject it with `WireVersionMismatch`: fail-closed by construction rather than by
a reserved-value check somebody has to remember to write. Plates are expensive
and md1's narrow paths are deliberate (F-417).

### 3e. Structural consequence

`read_node(r, key_index_width)` and `write_node(w, node, key_index_width)`
(`tree.rs:196`, `:79`) have no version parameter. The wire version must be
threaded into both, and into `chunk.rs`'s version checks (`:70`, `:373`). This
is mechanical but it touches every caller, and the plan must enumerate them
rather than leave it to be discovered.

### 3f. The type

`Body::Tr`'s `is_nums: bool` becomes a three-state internal key. Proposed:

```rust
pub enum InternalKey {
    /// A real, spendable key at slot `key_index`.
    Slot(u8),
    /// The BIP-341 NUMS H-point, spelled as raw x-only hex. Wire kind 0.
    NumsPoint,
    /// Liana's unspendable xpub, derived from the leaf keys (§2). Wire kind 1.
    LianaUnspendable,
}
```

This replaces the `is_nums: bool` + `key_index: u8` pair, whose invariant
("`is_nums = true` implies `key_index = 0`, no wire representation otherwise")
is today enforced by a `debug_assert!` at `tree.rs:148`. Making it a sum type
retires that assertion by construction — a funds-relevant simplification, not a
cosmetic one.

## 4. Rendering

| mode | today (kind 0) | kind 1 |
| --- | --- | --- |
| keyed descriptor (`md descriptor`) | literal x-only NUMS hex, `render.rs:195` | the **derived literal xpub** of §2, with `/<0;1>/*` and no origin |
| keyless template (`md compose` stdout) | literal x-only NUMS hex | the marker `UNSPENDABLE(liana)` |
| abstract template (`descriptor_to_abstract_template`) | literal x-only NUMS hex | the marker `UNSPENDABLE(liana)` |

The derived xpub **cannot** appear in a keyless form: its chain code is a
function of the seated leaf keys, so it does not exist until keys are seated.
The marker is therefore not a shortcut, it is the only correct spelling.

**The abstract-template marker is safe.** `descriptor_to_abstract_template` is
consumed only by `skeleton.rs:203` as a string key and by
`tests/render_abstract.rs` — verified by grep, it is never parsed as a
descriptor. The marker must differ from the raw-`H` hex so that kind 0 and kind
1 produce **different `SkeletonKey`s**; two policies identical but for the
internal-key kind are different wallets with different addresses, and a shared
skeleton key would be a false-evidence-match.

## 5. Slot numbering — the derived key takes no slot

**Ruling.** The derived internal key does **not** occupy a numbered placeholder.
`tr(<derived xpub>,{pk(@0),pk(@1)})`, exactly as kind 0 spells
`tr(<NUMS hex>,{pk(@0),pk(@1)})` today.

Reasons:

1. The operator never seats it, so a slot would be a phantom plate and a phantom
   card.
2. It would consume one of the 32 slots for a value nobody holds.
3. The numbering precedent is already set by kind 0, so kind 1 is a drop-in for
   every downstream consumer — `policy_shape`, `skeleton`, the slot map, the
   composer's `@i` numbering (spec §5, C19-C23) are all unchanged.
4. **Liana itself emits a literal xpub there, not a placeholder** — the eight
   evidence descriptors are literal-xpub form, and Liana accepted four of them.

The cost is BIP-388 strictness: BIP-388 wants the internal key to be a
placeholder backed by an xpub (bip-0388 l.139, 150-153, 310). md already
documents that the NUMS form is not BIP-388-registrable, and kind 1 is no worse
in that respect than kind 0 is today. It is strictly better for Liana, which is
the target.

## 6. Refusals

| condition | outcome |
| --- | --- |
| `kind = 1` on a `tr` whose tree has **no** leaf keys | REFUSE at encode and validate — the chain code is `sha256("")`, a constant, which would make every such wallet share one internal key |
| `kind = 1` under any wrapper other than `tr` | REFUSE — there is no internal key to be unspendable |
| `kind = 1` with a real internal key extracted (`InternalKey::Slot`) | unrepresentable by construction in the §3f sum type |
| a version-5 payload reaching a version-4 decoder | `WireVersionMismatch`, already the behaviour |
| version 5 with every `Body::Tr` at `kind = 0` | REFUSE at encode (the encoder must emit the minimum version); ACCEPT at decode, so the rule is enforced on one side only and old payloads never become invalid |

## 7. Verdict integration

- Rust: plan 1a's `KeyPathKind` gains a fourth value alongside
  `NotTaproot | Nums | Xpub` — `Unspendable`. A `SkeletonKey` built over kind 1
  must differ from the same tree at kind 0.
- Go (fork): `md.KeyPathNUMS` gains a sibling. `composerLianaOutsideModelClass`
  (`gui/composer_consent.go:381`) must stop returning `"NUMS key path"` for kind
  1 — that is class 2, and it is precisely what kind 1 exists to clear. The
  classes **behind** it must then fire normally, which is what keeps the three
  out-of-model presets refused.
- Spec §8f's NUMS notice (`composer_copy.go`) is written in the unqualified
  present tense about Liana; F-633 already tracks that. Kind 1 makes it wrong
  for kind-1 wallets, so F-633 is now **gating for this cycle** rather than
  deferred.

## 8. Acceptance

**The gate is byte-equality against descriptors Liana has already accepted.**

The eight `liana-unspendable-xpub` descriptors in
`design/evidence/composer-fable-r0/fable-liana-parse-in.jsonl` were fed to
`LianaDescriptor::from_str` at **both v8.0 and v15.0**, and four came back
ACCEPT. If md-codec emits a byte-identical descriptor string for those four
shapes, then Liana accepts md-codec's output. That is a deduction from a
measurement, not an inference from reading Liana's source.

Required, all of them:

1. **Recipe vectors.** md-codec derives the §2 xpub for all eight shapes and it
   equals the evidence byte-for-byte — including the four Liana refused, since
   the recipe is correct there too and only the *policy shape* is out of model.
2. **Descriptor equality.** `md descriptor` over the four ACCEPT shapes at kind
   1 emits a string byte-identical to the evidence input.
3. **Address equality.** Receive and change addresses at indices 0..2 agree
   between `md` and the evidence's recorded Liana addresses, and differ from the
   same tree at kind 0 — the second half is what proves kind 0 and kind 1 are
   different wallets, which is F-449's whole reason for being a wire item.
4. **Round trip.** encode → decode → render is byte-stable at both versions, and
   a version-5 payload is refused by a version-4 decoder with
   `WireVersionMismatch`.
5. **Structure-independence pin.** `kofn-recovery` and `tiered-recovery` over
   the same four keys derive the same internal key (§2).
6. **Mutation testing.** Every property above must be shown to *fail* when the
   code it guards is broken — per this project's standing rule, a proof in a
   transcript is not a gate. At minimum: flip the concat order, sort the keys,
   deduplicate them, drop the version bump, and use kind 0's hex at kind 1.

**Bonus, not required:** a live `harnesses/liana` run at v15.0. It needs a Liana
checkout and a long build, and it re-measures what the evidence already records.
Run it if the cycle has room.

## 9. Sequencing

1. **md-codec** (Rust primary): §3 wire, §3f type, §2 derivation, §4 rendering,
   §6 refusals, §8 vectors. Breaking — `md-codec` 0.46.0.
2. **md-cli**: `md compose --unspendable liana|nums` (default `nums`, so nothing
   changes for existing callers), and `md descriptor` rendering kind 1.
3. **Go port** (fork `md/`): behaviour-faithful port with the same vectors, plus
   the provenance pin bumped.
4. **Fork device/GUI**: §7's `KeyPathKind`, class 2, and the F-633 copy fix.
5. **Demo**: rebuild `demo/sh2/` and deploy to quantoshi.xyz/SH2/.

Each stage gates independently. Stage 1 is the only one that can proceed without
the ones above it.
