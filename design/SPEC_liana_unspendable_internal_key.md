# SPEC — the Liana unspendable internal key (F-449)

**Status:** r1, folded from two independent R0 reviews (opus 2C/6I/5M/2N, fable
adversarial 0C/4I/9M) plus four author-found findings. Awaiting re-review.
**Reports:** `design/agent-reports/f449-spec-r0-{opus,fable-adversarial,self}.md`
**Owning cycle:** its own constellation cycle in `descriptor-mnemonic`, per F-449.
**Rust-primary:** normative codec behaviour. Lands in `md-codec` **first**, with
test vectors, then the fork's Go port. The port may never lead.
**Baseline:** descriptor-mnemonic `6cbd49d8`, seedhammer `7b6f2fb`,
mnemonic-engrave `49964db1`. Suite at baseline: 1400 passed / 3 skipped.

## 0. What this delivers

An SH2 operator can compose a wallet under `tr` whose internal key is an
**unspendable xpub built by Liana's own recipe**, so Liana imports it.

Measured effect: the Liana-importable `tr` preset set goes from **1 of 6** to
**3 of 6** — `simple-timelocked-inheritance` (already works, real internal key),
plus `kofn-recovery` and `tiered-recovery`.

**The operator reaches it ON THE DEVICE.** r0 promised this in §0 and then
scheduled only verdict plumbing in §9, so no device path to a kind-1 wallet
existed (fable I-4: `md/compose.go:961` `isNums: ik < 0` is unconditional, and
`composerLianaOutsideModelClass` is called only from the consent screen at
`gui/composer_consent.go:261`). r1 fixes the plan, not the promise: stage 4
carries **one choice screen** — when a `tr` policy falls back to NUMS, the
composer asks which spelling, showing the compatibility consequence of each.
That is the minimum that makes §0 true. It is **not** the target-selection mode
(menu narrowing, picker constraints, multi-coordinator intersection), which
remains a separate later cycle.

**Rejected alternative:** defaulting the device to kind 1 for every NUMS `tr`.
It would silently change the wallet form for every existing operator and make
§8f's Nunchuk copy wrong in a new way (fable M-5).

### 0a. Not in scope

Making the other three tr presets import. Measured, they fire Liana's own
classes for reasons that are not about our encoding:

| preset under `tr` | class behind the NUMS one | why no encoding change helps |
| --- | --- | --- |
| `plain-multisig` | 3 `no locked path` | Liana requires a recovery path; a plain k-of-n has none |
| `hashlock-gated` | 4 `a hash lock` | Liana's policy model has no hash |
| `decaying-multisig` | 5 `an absolute lock`, then 7 `no unlocked path` | every tier timelocked, and one lock is `after()` |

Their deliverable is the **verdict**, which the coordinator-compat work
(plans 1a/1b/3) already builds. This spec must not weaken that refusal — see §7.

## 1. Measured baseline

| fact | measurement |
| --- | --- |
| the composer extracts a real internal key iff some path is a bare, unlocked, unhashed single key | `compose/mod.rs:292` `is_bare_single`; `compose/tr.rs:45` `is_nums: ik.is_none()` |
| a multi-key **primary** under `tr` emits `multi_a`, not `sortedmulti_a` | `md compose --wrapper tr --path 2of3 --path 2of3,older=26280`, RUN |
| `sortedmulti_a` appears only in the single-path bare-multi case | `md compose --wrapper tr --preset plain-multisig,2of3`, RUN |
| 5 of 17 measured `tr` shapes import into Liana today, all with a 1-of-1 unlocked primary | `IMPORTABILITY_composer_shapes.md`, counted mechanically |
| rewriting a NUMS tree with Liana's unspendable xpub makes 4 of 8 ACCEPT | `design/evidence/composer-fable-r0/fable-liana-parse-{in,out-v15}.jsonl`, variant `liana-unspendable-xpub` |
| v8.0 and v15.0 agree on all of it | `fable-liana-parse-out.jsonl` vs `…-v15.jsonl` |

## 2. The recipe (normative)

Liana's own, `liana/src/descriptors/analysis.rs:398-430`.

Given the taproot leaves' key expressions in **descriptor left-to-right order** —
which is **wire/slot order**, NEVER the derived-key-sorted order the device's
address builder uses for `sortedmulti_a` (fable M-8; this is the single line a
port implementer would otherwise get wrong, and §6 refuses the shape that would
expose it):

1. Take each leaf key's **33-byte compressed public key**. In md-codec that is
   bytes **`[32..65]`** of the 65-byte TLV entry, whose layout is
   `chain code ‖ compressed pubkey` (`validate.rs:331-335`, `:348`). It is
   **not** a base58 payload and md1 carries **no version bytes** — r0 said
   `[45..78]` of a base58 decode, which is the *descriptor* representation, not
   md's (opus I2).
2. Concatenate in that order. **Not sorted. Not deduplicated.**
3. `chain_code = sha256(concatenation)`.
4. Build an extended public key with `public_key =
   0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`
   (BIP-341 `H`, compressed), the chain code from step 3, `depth = 0`,
   `parent_fingerprint = 00000000`, `child_number = 0`.
5. **Version bytes come from the render-time network selector**
   (`md descriptor --network`, `md address --network`;
   `crates/md-cli/src/main.rs:1081, 1120`), not from the wire. Mainnet → `xpub`,
   testnet/signet/regtest → `tpub`.
6. Render with `origin: None`, derivation paths `<0;1>`, unhardened wildcard.

**Derivation, not just rendering (fable M-9).** The internal key derives at
`0/i` for receive and `1/i` for change **in every port**, independent of the
wallet's use-site path. §6 refuses kind 1 when the use-site is not `<0;1>`, so
the two can never disagree.

**VERIFIED INDEPENDENTLY, and scoped honestly.** The recipe was reimplemented
from scratch and reproduced the golden xpub byte-for-byte for the **four shapes
Liana accepted** — all **mainnet `xpub`**, all **one-level** taptrees with
`multi_a`/`pk` leaves. Three gaps the evidence does NOT close, which §8 must:

- the **`tpub` branch** of step 5 is transcribed, not measured (opus I2);
- the **nested taptree** order (`{A,{B,C}}`) appears only in
  `preset-decaying-multisig-tr`, which Liana **refused on policy shape**, so no
  ACCEPT backs it. It is right by reading rust-miniscript's `TapTreeIter`
  (left-first DFS) but is not measured (fable M-2);
- no evidence shape has a **`sortedmulti_a` leaf** (fable I-2a).

**A property worth pinning.** The chain code depends on the ordered multiset of
leaf pubkeys and **not** the tree structure: `kofn-recovery`, `tiered-recovery`
and `decaying-multisig` over the same four keys in the same order all share one
internal key (verified across all three; the third is the nested case and is the
stronger pin — opus N2). Harmless, because the output key commits to the tree
through the taproot tweak and `SkeletonKey` carries the template. A test must
pin it so nobody "fixes" it.

**Only Liana's recipe imports into Liana.** Any other xpub there is treated as a
spendable key and, having no origin, is `InvalidKey`. The abandoned bitcoin/bips
PR #1746 recipe (sorted, deduplicated; closed unmerged 2025-09-17) is a
different xpub.

## 3. The wire kind

### 3a. The version budget, stated up front

**Usable WF-redesign versions are `{4, 8, 12}` — three generations, total**
(`header.rs:4`, `:26`; `decode.rs:171`; `SPEC_v0_30_wire_format.md:129`). Four
is spent. **This cycle spends 8, leaving 12 as the last one**; after 12 a break
requires widening the version field, which itself moves the discriminator.

r0 reasoned "version 5, then 6 for a third recipe" against an unbounded budget
(opus I1). The real cost of this feature is **one of the two remaining wire
generations**. The cycle accepts that cost because §3c shows there is no
fail-closed alternative — but any *other* pending wire want should be batched
into version 8 rather than spending 12 separately.

### 3b. Why it cannot be additive — all four alternatives tested

| alternative | why it fails | verified |
| --- | --- | --- |
| a new bit after `is_nums` | a v4 decoder reads it as `has_tree` and desyncs — silent wrong addresses | by construction |
| a new TLV type | unknown TLVs are **preserved and skipped**, so the kind is silently ignored and raw `H` addresses are derived | `tlv.rs:273-274` "Unknown — buffer and skip per D6 forward-compat", RUN |
| a reserved `key_index` sentinel | `key_index_width = ceil(log2(n))`; n=4 → 2 bits → 0..3, no spare. Widening the field IS the wire change | `decode.rs:88` |
| **a header version bump** | **fail-closed everywhere; chosen** | §3c |

### 3c. The version is 8, not 5 — the dispatch constrains it

**r0's version 5 was unimplementable** (opus C1, fable I-1, both independently;
reproduced by the author).

md1 auto-dispatches single-payload vs chunked on **bit 0 of the first symbol**,
before any header is parsed (`decode.rs:191-193`; again at `chunk.rs:650-651`;
Go at `md/chunk.go:193`, `md/md.go:1235`). For a single payload the first symbol
is `[divergent][v3][v2][v1][v0]`, so **bit 0 is `v0`**. Therefore every usable
single-payload version must be **even**. Computed:

```
ver div  first-symbol  byte0  chunked_flag  routes-to
  4   0     0b00100    0x20        0        single-payload Header::read   (correct)
  4   1     0b10100    0xa0        0        single-payload Header::read   (correct)
  5   0     0b00101    0x28        1        CHUNK reassembler             (WRONG)
  5   1     0b10101    0xa8        1        CHUNK reassembler             (WRONG)
  8   0     0b01000    0x40        0        single-payload Header::read   (correct)
  8   1     0b11000    0xc0        0        single-payload Header::read   (correct)
 12   0     0b01100    0x60        0        single-payload Header::read   (correct)
 12   1     0b11100    0xe0        0        single-payload Header::read   (correct)
```

At version 5 a single-string plate routes to the chunk reassembler, which reads
bits 4..1 as its version and reports `WireVersionMismatch { got: 2 }` — the
error `header.rs:100-110` reserves for a *pre-redesign v0.x card*. The device
(`gui/multisig_verify.go:834`, `gui/md1_gather.go:31`, `sysw/confirm.go:123`)
and `md repair` go through the dispatch; host `md decode`/`descriptor`/`verify`
call `decode_md1_string` directly. So a v5 plate would **decode on the host and
be refused on the device that cut it**, naming a version no encoder ever emitted.

**Ruling: `Header::WF_UNSPENDABLE_VERSION = 8`.**

### 3d. The encoding

At **version 8**:

```
Tag::Tr | is_nums(1) | [kind(1) iff is_nums] | [key_index(kiw) iff !is_nums] | has_tree(1) | [tree]
```

`kind`: `0` = BIP-341 raw `H` (what version 4 means), `1` = the Liana
unspendable xpub of §2. At **version 4** the stream is unchanged and no `kind`
bit is read or written.

- The encoder emits the **minimum version that can express the tree**: 8 only
  when some `Body::Tr` carries `kind = 1`; otherwise 4. Existing plates keep
  their bytes and keep meaning raw `H`.
- The decoder accepts `{4, 8}` and rejects everything else with
  `WireVersionMismatch`.
- At version 4, `Body::Tr` decodes with `kind = 0` implicitly.
- **The chunk writer must agree with its payload** (opus M2): `chunk.rs:279`
  hardcodes `version: Header::WF_REDESIGN_VERSION`, as does `encode.rs:174`.
  Both take the derived version, so a chunked kind-1 set carries version 8 in
  both the chunk headers and the embedded payload header. A v4 decoder then
  refuses at `chunk.rs:70` with `got: 8` — loud and correctly named.

**Why one bit and not two.** A 1-bit field has no undefined value, so no
"unknown kind" error path exists to forget to write. A future third recipe would
require version 12 (§3a), which v8 decoders reject by exact match.

### 3e. Version threading — and the two identity hashes it lands on

`read_node`/`write_node` (`tree.rs:196`, `:79`) gain an explicit wire version.
`write_node` has exactly **three** non-test callers, and r0 named none of them
(opus C2):

| caller | what it computes |
| --- | --- |
| `encode.rs:181` | the wire payload |
| `identity.rs:90` | `WalletDescriptorTemplateId` |
| `identity.rs:200` | `WalletPolicyId` → `to_phrase()`, the 12-word identity phrase |

Both identity sites hash the tree with **no header**, and `Descriptor` carries
no version field — so the version they pass is a free choice with a funds-
relevant consequence.

**Ruling: the version is DERIVED FROM THE TREE, never a constant.** Add
`Descriptor::wire_version()` returning the minimum version that can express the
tree (§3d), and pass it at all three sites.

Rejected: passing `WF_REDESIGN_VERSION`. At version 4 no kind bit is written, so
`NumsPoint` and `LianaUnspendable` would emit identical bytes — two wallets with
**different addresses** sharing one `WalletPolicyId`, one 12-word phrase and one
`WalletDescriptorTemplateId`. That inverts the documented contract at
`identity.rs:120-127` ("two engravings of the same logical wallet produce
identical IDs") exactly for the pair this cycle creates. §4's own
false-evidence-match reasoning applies here and more sharply.

`compute_md1_encoding_id` → `chunk_set_id` is unaffected: it goes through
`encode_payload_for_identity`, which includes the header (`identity.rs:45`).

Read-side version checks to thread: `chunk.rs:70`, `:373`; write side
`chunk.rs:279`, `encode.rs:174`.

### 3f. The type, and how it lands

`Body::Tr`'s `is_nums: bool` + `key_index: u8` become:

```rust
pub enum InternalKey {
    /// A real, spendable key at slot `key_index`.
    Slot(u8),
    /// BIP-341 NUMS H-point, spelled as raw x-only hex. Wire kind 0.
    NumsPoint,
    /// Liana's unspendable xpub, derived from the leaf keys (§2). Wire kind 1.
    LianaUnspendable,
}
```

This retires the `debug_assert!` at `tree.rs:148` by construction.

**It ships as TWO commits, refactor first** (SELF-3). Measured blast radius: **88
`is_nums` occurrences across 14 files** in `md-codec/src`, **58** NUMS
references in the fork's `md/` + `gui/`, plus **four** non-test md-cli sites
(`format/json.rs:348`, `parse/reuse.rs:515`, `parse/template.rs:1604`, `:1629`).
The opus review listed a fifth, `seat/compose.rs:148`; verified, it does not
break — it matches `Body::Tr { tree: Some(t), .. }`, naming only `tree`, so the
`..` absorbs the field change. A mechanical ~146-site rename in the same diff as a
funds-relevant wire change produces a diff nobody can review, and the wire
change hides in the noise. So:

1. behaviour-preserving `is_nums`/`key_index` → `InternalKey`, wire untouched,
   full suite green;
2. the version-8 kind bit on top. `git diff` over commit 2 is then exactly the
   wire change.

## 4. Rendering

| mode | function | kind 0 | kind 1 |
| --- | --- | --- | --- |
| keyed descriptor | `md descriptor` | raw x-only NUMS hex (`render.rs:195`) | the **derived literal xpub** of §2, `/<0;1>/*`, no origin |
| keyless template | `descriptor_to_template` | raw x-only NUMS hex | `UNSPENDABLE(liana)` |
| abstract template | `descriptor_to_abstract_template` | raw x-only NUMS hex | `UNSPENDABLE(liana)` |

The derived xpub **cannot** appear in a keyless form — its chain code is a
function of the seated leaf keys, which a template-only card does not carry.
The marker is the only correct spelling, not a shortcut.

**Row 3 is safe; row 2 is not, and r0 conflated them** (opus I3).
`descriptor_to_abstract_template` is consumed only by `skeleton.rs:203` as a
string component and by `tests/render_abstract.rs` — never re-parsed. But
`descriptor_to_template`'s output is a **documented input**
(`README.md:129`, `md descriptor --template`), emitted by `md compose`
(`compose/mod.rs:632`) and printed by `md decode`/`md inspect`. It re-enters
through `parse_template` → `substitute_synthetic` → `Descriptor::from_str` →
`walk_tr` (`md-cli/src/parse/template.rs:1589-1632`), which accepts exactly `H`
or `@N`. So §4a is mandatory, not optional.

Note `format/text.rs:269-274` ships a render-reparse **fixpoint assertion** whose
corpus is `wsh`-only — it would stay green while the invariant it names is false
for `tr` kind 1. §8 must extend that corpus.

### 4a. The input side (mandatory)

**`md` must RECOGNISE the form, not merely emit it.** Three separate defects
converge here (opus I3, fable I-3, SELF-4):

- `md encode` on a literal xpub in the internal-key position fails with an
  **internal error** leaking to the user — `"internal: synthetic key … not found
  in key map"` (RUN). Whatever the scope, that string must never reach a user.
- `md decompose` on a real Liana taproot descriptor makes the unspendable xpub
  **slot `@0`** — a five-slot template for a four-key wallet, with a phantom slot
  no card can be minted for (RUN: `--emit commands` refuses, "1 key(s) state no
  origin"). Two md1 spellings of one wallet then exist, with different
  `WalletDescriptorTemplateId`s and different `SkeletonKey`s.
- The coordinator-compat conformance gate is `chunks -> key == descriptor -> key`
  for **every** evidence row (`DESIGN_coordinator_compatibility.md:620-621`). For
  the `liana-unspendable-xpub` rows, `descriptor -> key` goes through that
  phantom-slot parse and then fails `expand_per_at_n` (`skeleton.rs:186-193`), so
  the very ACCEPT evidence this cycle exploits is unkeyable.

**Ruling.** `md encode`, `md decompose`, and the template grammar recompute §2
over the parsed leaf keys and accept the descriptor as kind 1 **only when the
xpub matches**. A non-matching xpub in that position gets a real refusal naming
the mismatch, never the internal error and never a phantom slot.

This turns the recipe from something md emits into something md **verifies**,
and it makes the natural opposite journey work: a wallet created in Liana can be
carried onto plates.

The template grammar gains a substitution rule for `UNSPENDABLE(liana)`
(opus I3, fable M-1), so `md compose | md encode` round-trips.

**Published contract change:** `md decode --json`'s `is_nums: bool`
(`format/json.rs:326`, `docs/json-schema-v1.md`) and `md compose --json`'s
internal-key field need a third state. This is a **breaking change to a
published v1 schema** and must be versioned as one, not slipped in.

### 4b. Reconciling the coordinator-compat design

`DESIGN_coordinator_compatibility.md:155-158` states an unspendable xpub is *not*
a variant because recognising one "means re-deriving a specific coordinator's own
function — so that distinction lives in a rule, not in the key", and
`policy_shape.rs:119-133` records the same ruling in source, ending **"Do not
'restore fidelity' with the Go name here"**.

This spec makes it codec-observable, which is a legitimate reversal — but both
texts must be **retired in the same change**, or the source actively instructs
the next reader not to do what this spec requires (opus M3, fable I-3).

## 5. Slot numbering — the derived key takes no slot

**Ruling.** `tr(<derived xpub>,{pk(@0),pk(@1)})`, exactly as kind 0 spells
`tr(<NUMS hex>,{pk(@0),pk(@1)})`.

1. The operator never seats it, so a slot would be a phantom plate and card.
2. It would consume one of 32 slots for a value nobody holds.
3. Kind 0 sets the numbering precedent, so kind 1 is a drop-in for
   `policy_shape`, `skeleton`, the slot map and `@i` numbering.
4. Liana itself emits a literal xpub there, not a placeholder.

Independently confirmed sound by both reviews: `key_index_width`, seat counts,
mk1 counts and `@i` numbering are untouched, and `duplicate_keys.go:185-199`
already skips the NUMS internal key and will skip kind 1 identically.

The BIP-388 concession is real but unchanged from today: kind 0 is already a
literal in a position BIP-388 wants to hold a placeholder.

## 6. Refusals and invariants

| condition | outcome |
| --- | --- |
| `kind = 1` with a **`sortedmulti_a` leaf anywhere in the tree** | REFUSE. The device's address builder sorts that leaf's keys **by derived key, per index** (`address/taproot_script_path.go`), while §2 hashes them in wire order — a chain code that changes with the address index. The only shape that produces it (`plain-multisig`) is out of scope anyway (§0a), so this costs nothing and closes fable I-2a |
| `kind = 1` with a use-site path other than `<0;1>` | REFUSE. `md encode` accepts `<2;3>`, `<0;1;2>` and `<0;1>/*h` under `tr(H,…)` today (RUN). Liana pairs alternatives positionally and would derive from `0/i` while a use-site-following device derives from `2/i` (fable I-2b). Refusing is narrower than reconciling, and keeps §8.3 honest |
| `kind = 1` with a real internal key extracted | unrepresentable in §3f's sum type. `--unspendable liana` on a path list containing a bare single is a **no-op today**; it must WARN, not silently ignore (fable M-6) |
| `kind = 1` **nested** under `sh`/`wsh` | REFUSE. r0's "any wrapper other than `tr`" was ambiguous between vacuous and this (opus M1); this is the non-vacuous reading and the one that needs a check |
| a version-8 payload reaching a version-4 decoder | `WireVersionMismatch` — single-string via `Header::read` (the dispatch routes it correctly at 8), chunked via `chunk.rs:70` with `got: 8` |
| version 8 with every `Body::Tr` at `kind = 0` | REFUSE at **encode** (minimum-version rule). Accepted at decode so old payloads never become invalid — but note this admits a hand-crafted second encoding of a kind-0 wallet with a different `WalletDescriptorTemplateId` (opus/fable M-4). Documented, not reachable through any encoder |

**INVARIANT, not a refusal (SELF-2).** r0 required REFUSE when `kind = 1` has no
leaf keys, to stop `sha256("")` becoming a shared constant. That state cannot be
constructed: the template parser requires at least one `@i` placeholder anywhere
(RUN: `md encode "tr(<NUMS hex>)"` → *"template contains no @i placeholders"*),
and under `tr` with a NUMS-family internal key the internal key is not a
placeholder, so every placeholder lives in a leaf and the concatenation is
non-empty. **A refusal that cannot fire is not a guard** — it reads as safety and
its test passes vacuously. It ships as a pinned invariant with a test
demonstrating *why* it is unreachable, so if the placeholder rule ever changes
the pin fails instead of admitting an empty concat.

## 7. Verdict integration

- **Rust:** `KeyPathKind` gains a fourth value beside `NotTaproot | Nums | Xpub`.
  Name it `NumsXpub` rather than `Unspendable` — `Unspendable` sitting next to
  `Nums` implies `Nums` is spendable (opus M3). `skeleton_key` already appends
  `key_path_kind_label` (`skeleton.rs:313-320`), so kind 0 and kind 1 produce
  different `SkeletonKey`s from the fourth value alone.
- **Go:** `md.KeyPathNUMS` gains a sibling.
- **The class-2 ruling, stated precisely** (opus I6). The new kind is
  **excluded from class 2's refusal** (`composer_consent.go:397`) — that is what
  it exists for — **and is still NOT counted as an unlocked path**, because an
  unspendable key spends nothing. These are two separate decisions and r0 stated
  only the first.

  **Constructed failure if conflated:** `tr(<derived xpub>, and_v(v:pk(@0),older(26280)))`
  — a single timelocked leaf. Correct verdict is class 7 *no unlocked path* →
  REFUSE. If the sibling is grouped with `KeyPathSpendable`, `unlocked` becomes
  1, class 7 never fires, and the composer tells the operator the wallet is
  Liana-compatible **before the plates are cut**. A device test must pin this
  exact shape.
- **Fork consumers switch on `KeyPath` with no default** (fable M-3):
  `gui/template_engrave.go:159-164` ("THE KEY-PATH LINE COMES FIRST AND IS NEVER
  OMITTED") and `gui/composer_consent.go:205-214` print nothing for a fourth
  value, silently breaking the engrave summary's own stated invariant. Both need
  a kind-1 arm.
- **F-633 becomes gating for this cycle**, and its copy fix must also say that
  §8f's *"Nunchuk cannot import a NUMS policy at all"* is **false for kind 1 some
  of the time** (fable M-5): libnunchuk re-renders the key path with the PR-1746
  recipe and requires the re-render to equal the input
  (`descriptor.cpp:640-648`), so it accepts a kind-1 descriptor exactly when the
  leaf pubkeys already sit in sorted, unique order — 1/2 for two keys, 1/6 for
  three, 1/24 for four — and then imports the *same* wallet.

## 8. Acceptance

r0 claimed byte-equality against previously-accepted descriptors was equivalent
to an import test. It is not: the evidence measures `LianaDescriptor::from_str`
(**parse**), while §0's promise is **import** (opus I4). The install path is
recorded in `fable-liana-lianad.out` for six shapes and for **zero**
unspendable variants, so r0's "the live run re-measures what the evidence
already records" was false.

**Required, all of them:**

1. **Recipe vectors.** md-codec derives the §2 xpub for all eight evidence
   shapes, byte-identical. Plus **new vectors** for the three unmeasured gaps in
   §2: a `tpub` wallet, a nested taptree that Liana ACCEPTS, and — if §6 did not
   refuse it — a `sortedmulti_a` leaf.
2. **Descriptor equality**, for the four ACCEPT shapes, **including the
   descriptor checksum** (`#8jc8gq6v` etc. appear in the evidence). r0 left this
   undefined, which is the difference between a gate and a formatting failure
   (opus I4/M4).
3. **Address equality, three-way.** Receive and change at indices 0..2 agree
   between **md**, **the evidence's recorded Liana addresses**, and **the
   device**. The device leg is F-449's own wording and r0 dropped it (fable I-2):
   the Go `md/` port never derives an address — `gui/policy_address.go:132-160`
   and `address/taproot_script_path.go` do — so a `md/` vector cannot reach it.
   Addresses must also **differ** from the same tree at kind 0, which is what
   proves kind 0 and kind 1 are different wallets.
4. **Round trip through the DISPATCH, not just the payload decoder.** r0's
   "encode → decode → render" written against `decode_payload` would have passed
   while version 5 was unusable. The gate runs `decode_with_correction` **and**
   the Go `ParseChunkHeader`/`Decode` pair, single-string **and** chunked, at
   both versions, plus a v8 payload refused by a v4 decoder.
5. **Identity distinctness and stability.** Same tree at kind 0 vs kind 1 →
   different `WalletPolicyId`, different `WalletDescriptorTemplateId`, different
   12-word phrase. And every existing v4 id is byte-preserved. r0 had no id
   vector at all and would not have caught §3e's collision.
6. **Structure-independence pin.** All three of `kofn-recovery`,
   `tiered-recovery` and `decaying-multisig` over the same four keys derive one
   internal key — the third is the nested case.
7. **Render-reparse fixpoint extended to `tr` kind 1**, since
   `format/text.rs:269-274`'s corpus is `wsh`-only.
8. **A live `harnesses/liana` install run at v15.0 — REQUIRED, not a bonus.**
   It is the only measurement of the install path for any unspendable shape.
9. **Mutation testing.** Every property above must be shown to FAIL when the code
   it guards is broken: flip the concat order, sort the keys, deduplicate them,
   drop the version bump, use kind 0's hex at kind 1, pass a constant version to
   the identity hashes, and group the new kind with `KeyPathSpendable`.

## 9. Sequencing

| stage | content | gate |
| --- | --- | --- |
| **1a** | behaviour-preserving `InternalKey` refactor, wire untouched (§3f) | suite green at 1400+, no wire bytes changed |
| **1b** | version 8, the kind bit, §2 derivation, §4 rendering, §4a input side, §6 refusals, §8 vectors 1-7, 9 | §8 |
| **2** | `md compose --unspendable liana\|nums` (default `nums`), `md descriptor` kind 1, the JSON schema version bump (§4a) | §8.2 |
| **3** | Go port in the fork's `md/`, same vectors, provenance pin bumped | §8 vectors in Go |
| **4** | device: §7's `KeyPathKind`, the class-2 + unlocked-path ruling, the two switch arms, F-633 copy, **and the one choice screen (§0)** | §8.3 device leg, §7's constructed shape |
| **5** | rebuild `demo/sh2/` and deploy to quantoshi.xyz/SH2/ | site 200 + emulator reaches the new screen |

**Stage 1a/1b do NOT stand alone as r0 claimed** (opus I5). `md-cli` pins
`md-codec = { path = "../md-codec", version = "=0.45.1" }`
(`crates/md-cli/Cargo.toml:28`) in the same workspace, so bumping to 0.46.0
makes resolution fail **before rustc runs**, and §3f breaks four non-test md-cli
sites at compile time (§3f names them). The pin bump and the call-site repairs ship **in the same
commit** as the change that requires them.

**`mnemonic-toolkit` is downstream** (opus M5): `render.rs:9-12` records that its
`inspect` renders the same `template:` line via `descriptor_to_template`,
"guaranteeing byte-identical output across both binaries". §4 changes that
function's output for kind 1. The toolkit pins md-codec by git tag, so it is not
broken by stage 1 — but it is **in scope for this cycle** and gets its pin bump
and a golden refresh after stage 2.
