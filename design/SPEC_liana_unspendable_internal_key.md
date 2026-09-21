# SPEC — the Liana unspendable internal key (F-449)

**Status:** r5, folded from nine independent reviews plus four author-found
findings. R0: opus 2C/6I/5M/2N, fable adversarial 0C/4I/9M. r1: fold-check
1 unaddressed + 2 defects, new-design 1C/4I/2M. r2: fold-check 0 unaddressed +
1 cosmetic, **journey walk 2C/6I/4M**. r3: fold-check 4 partial + 0 new
defects, new-design **0C**/6I/4M. r4: closing review **0C**/3I/5M, all eight of them
fold-propagation defects rather than design defects. Awaiting final check.
**Reports:** `design/agent-reports/f449-spec-r0-{opus,fable-adversarial,self}.md`,
`f449-spec-r1-{fold-check,new-design}.md`, `f449-spec-r2-{fold-check,journey}.md`,
`f449-spec-r3-{fold-check,new-design}.md`, `f449-spec-r4-closing.md`

**Both r2 Criticals were the author's own folds**, found only by a journey walk
after four correctness-shaped lenses had closed: a choice-screen predicate that
fired on the one preset where the choice is meaningless and stayed silent on the
two this cycle exists for, and an identity ruling written for Rust that left the
Go half — the half the operator reads off the screen and off an mk1 card —
unscheduled and ungated.
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
`gui/composer_consent.go:261`). r2 fixes the plan, not the promise. Two pieces of device work, not one:

1. **A choice screen** — see §0b, which specifies it completely. r1 gave it one
   sentence and that sentence was measurably inverted.
2. **Device address derivation for kind 1**, which r1 omitted entirely and
   which is the larger half. See §7a.

### 0b. The choice screen (normative)

r1 specified this screen in one sentence — *"fired only when
`composerLianaOutsideModelClass` (class 2 skipped for the new kind) returns
`""`"* — and a journey walk measured that rule to be **inverted**:

| preset under `tr` | `shape.KeyPath` | classifier | r1's rule fires? | should fire? |
| --- | --- | --- | --- | --- |
| `simple-timelocked-inheritance` | `KeyPathSpendable` | `""` | **yes** | **no** |
| `kofn-recovery` | `KeyPathNUMS` | `"NUMS key path"` | **no** | **yes** |
| `tiered-recovery` | `KeyPathNUMS` | `"NUMS key path"` | **no** | **yes** |

It fired on the one tr preset where the choice is meaningless and stayed silent
on the two this cycle exists for — which would have silently deleted §0's whole
measured claim. Skipping class 2 alone does not fix it: with class 2 skipped
`simple-timelocked-inheritance` still returns `""`, and it has a **real**
internal key (`md compose … --json` → `internal_key_path: 0`) where kind 1 is
unrepresentable in §3f's sum type.

**FIRING PREDICATE — both conjuncts are required:**

```
fire  ⟺  the tr internal key is NUMS today          (internal_key_path == null)
     AND composerLianaOutsideModelClass(root, shape), with class 2 skipped
         for the new kind, returns ""
```

Conjunct 1 is what r1 dropped. Conjunct 2 keeps the screen off
`plain-multisig`, `hashlock-gated` and `decaying-multisig`, where §0a's measured
table says no encoding change helps and kind 1 is a strict downgrade (different
addresses, different `WalletPolicyId` and 12-word phrase, Liana still refusing,
Nunchuk acceptance falling to 1/24 for four keys) — and where, for
`plain-multisig`, §6 row 1 would refuse the option the screen offered.

**PLACEMENT.** The screen sits **between `composerShapeFlow` and the first
`composerStubFlow`** — measured, `composerShapeFlow` closes at
`gui/composer_flow.go:95` and the first `composerStubFlow` is at `:129`, so the
window is **`96-128`**. Within it the screen must come **before**
`composerTemplateChunksFor`, since that is what turns the kind into the chunks
the stub screen displays. The kind changes the tree,
hence the template chunks, hence the Template-ID and `mk1 stub` the operator
**copies onto steel and mints cosigner cards from**. r1 pointed twice at
`gui/composer_consent.go:261` as "where the predicate already ships", which aims
an implementer at the *last* screen before steel. If the choice is ever placed
after the stub screen it MUST return through `composerStubFlow` and raise the
existing changed-id banner (`composerStubDelta`), whose own doc comment says a
false statement there is worse than a missing one.

**RESET — expressed as a predicate, not as an edit event.** r3 said the kind
"resets on any shape, wrapper or path-list edit that re-enters
`composerShapeFlow`", which is a granularity the composer cannot observe: it
does not diff path lists, and the one existing change detector is measurably
blind to the edits that flip §0b's predicate.

So do not detect the edit. **Re-evaluate the predicate.** Recompute §0b's two
conjuncts against the current shape **immediately before
`composerTemplateChunksFor` (`gui/composer_flow.go:98`)** — the same window
PLACEMENT puts the screen in, and the point at which the kind becomes the
template chunks. If either conjunct is false, the kind is 0.

**How the screen gets a shape before the chunks exist.** Conjunct 2 needs
`composerLianaOutsideModelClass(root, shape)`, and the fork's only producer of a
`md.PolicyShape` is `md.PolicyShapeChunks(strs []string)`
(`md/policy_shape.go:107`) — it needs chunks. That is not a contradiction with
placing the screen before `composerTemplateChunksFor`, because that function is
**pure** (`md.ComposeWith(st.list, …).Chunks()`,
`gui/composer_flow.go:267-273`): the screen calls it itself to evaluate the
predicate, and the flow calls it again afterwards to build the chunks the stub
screen shows. Say so explicitly, or the ordering reads as impossible.

r4 first hooked this at `composerStubFlow` (`:129`), which is **after**
`composerTemplateChunksFor` (`:98`). The chunks — and therefore the Template-ID
and `mk1 stub` that the stub screen exists to be **copied onto steel** from —
would already have been computed at kind 1 for a wallet the reset then made
kind 0. Re-evaluating before the chunks are built closes that window entirely. A kind-1 selection is therefore never carried
into a shape that cannot represent it, no edit needs to be detected, and the
Back invariant is preserved in the sense that matters: nothing is lost while it
is still applicable.

**Be precise about why a drop is legitimate, because a port implementer will
reason from it.** Conjunct 1 failing means the kind is genuinely
*unrepresentable* — a real internal key has no seat in §3f's sum type. Conjunct
2 failing does **not**: kind 1 over a `decaying-multisig` tree is encodable
(§3d), renderable (§4) and not refused by §6. Conjunct 2 is a Liana-model
**usefulness** test, not a representability one. The reset is still right —
offering a kind that changes the wallet and buys nothing is a downgrade — but it
is a product judgement, so a dropped choice must be **signalled**, not silent.
`composerStubDelta`'s cause-free banner is not sufficient alone, the same
insufficiency `composerKeyOrderStep`'s comment already records.

This also makes the screen's reachability total: the predicate that shows the
screen is the same predicate that keeps the value, so there is no state where
the kind is set but the screen cannot be reached to unset it.

**DEFAULT ROW — seeded once, from the current value.** `Initial` is the NUMS
row **on first entry only**; on any re-entry it is the kind currently set. The
zero-value trap is real (a default of "Liana xpub" would silently change the
wallet form for an operator who pressed through), but "always NUMS" is wrong on
every pass after the first: because PLACEMENT puts this screen before the stub
screen, an operator who sets kind 1 and then steps Back would find the screen
proposing NUMS again, and pressing through would silently revert their choice.
The composer has closed this exact defect twice — a picker that opens on row
zero *proposes* a setting, so looking at it changes it. Seed the initial **once
per screen entry** from current state, never from a constant.

**COPY.** The screen names, for each row: which coordinators import the result,
and that the two rows are **different wallets with different addresses** that
cannot be interchanged after engraving. This is the one screen in this spec that
decides which of two wallets is cut into steel; every other operator-facing
string here carries a copy requirement and r1 gave this one none.

It is **not** the target-selection mode (menu narrowing, picker constraints,
multi-coordinator intersection), which remains a separate later cycle.

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

**THIS RULING BINDS THE GO PORT IDENTICALLY.** r2 stated it for Rust only, and
the fork mirrors the structure 3-for-3 with `writeNode` equally version-less
(`func writeNode(w *bitWriter, n node, keyIndexWidth uint8) error`):

| Go site | what it computes |
| --- | --- |
| `md/encode.go:417` | the wire payload |
| `md/template_id.go:53` | `WalletDescriptorTemplateId` |
| `md/walletpolicyid.go:42` | `WalletPolicyId` |

Leaving these at a constant version reintroduces §3e's collision **on the
surface where the plates physically are**, because these are exactly the values
the operator reads and trusts:

- `gui/composer_consent.go:216-229` → `md.FormAwareIdChunks` /
  `FormAwareStubChunks` — the consent screen's id and **mk1 stub**;
- `policyIDHeader` → `md.WalletPolicyIdChunks` — the `Policy id:` line on the
  inspect screen, the only line there that could distinguish the two kinds;
- `md/template_id.go:112` `FormAwareStub` → `WalletPolicyIDStub` **or**
  `WalletDescriptorTemplateIdStub` — the **mk1 KEY card's** `policy_id_stub`,
  which binds a cosigner's key card to a policy. (The journey report cited
  `:116`, the `else` branch; verified, the dispatcher is `:112` and **both**
  stub flavours route through the version-less `writeNode`, so the exposure is
  slightly wider than reported.)

**Consequence if missed:** an mk1 KEY card minted for the kind-0 wallet seats
and verifies against the kind-1 plates of the same tree, and the reverse. Same
stub, different addresses — a positive false match on the screen whose entire
job is to prove two artifacts belong together.

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
`is_nums` occurrences across 14 files** in `md-codec/src`, plus the NUMS
references in the fork's `md/` + `gui/` — reproducible as
`grep -rn "isNums\|IsNUMS\|KeyPathNUMS\|NUMS" md/*.go gui/*.go | wc -l`, which
gives **87 in `md/*.go` and 49 in `gui/*.go`, 136 across 32 files** (r1 said 58,
from a grep that used the Rust spelling `is_nums` against Go source) — plus
**four** non-test md-cli sites
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

**Ruling, and it differs per surface — r1 treated three surfaces as one.**

| surface | can it recompute §2? | ruling |
| --- | --- | --- |
| `md decompose` (a concrete descriptor) | **yes** — it holds the real leaf keys | recompute; on a match, kind 1 and no slot |
| the template grammar | n/a — the marker carries no key binding | accept `UNSPENDABLE(liana)` via a substitution rule |
| `md encode` with a **literal xpub** | **no** | refuse, with a message that names the marker |

`md encode` cannot do it, and r1's ruling was unimplementable there.
`parse_template` → `substitute_synthetic` (`md-cli/src/parse/template.rs:1047-1084`)
replaces every `@i` with a domain-separated placeholder derived from
`sha256(b"md-v0.15" ‖ i ‖ depth)` **before** `walk_tr` ever sees the tree, so at
the moment the kind must be decided the leaf keys are synthetic. The recipe over
synthetic leaves cannot equal the recipe over the wallet's real leaves, so the
match can never succeed and r1's prescribed *"refusal naming the mismatch"*
would have told a Liana user that their genuine Liana key is not their genuine
Liana key.

So `md encode` **refuses a literal xpub in the tr internal-key position** with a
message naming the two spellings that do work — `UNSPENDABLE(liana)` in a
template, or handing the concrete descriptor to `md decompose`. That replaces
the internal-error leak, which is the actual defect.

**A non-matching origin-less internal key keeps TODAY'S behaviour.** r1 said
"never a phantom slot", which over-refuses: the property that makes a slot
phantom is **no origin**, not "not Liana's". `md decompose` today accepts such a
descriptor, annotates the origin-less slot, and refuses only `--emit commands` —
deliberate shipped behaviour. Two real classes would lose a working verb under
r1's rule: libnunchuk's PR-1746 form (a real wallet Nunchuk builds, for which
md1 has no wire encoding), and **a real spendable internal key whose owner
recorded no origin**, where `@0` is the correct answer and not phantom at all.

Ruling: match §2 → kind 1, no slot. Otherwise → today's annotated slot,
unchanged. This cycle adds a recogniser; it removes no existing capability.

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
| `kind = 1` with a **`sortedmulti_a` leaf anywhere in the tree** | REFUSE, as a **belt against a port error**. §2 hashes the leaves' *account-level* 33-byte pubkeys, which are index-independent, so a correct implementation's chain code does not vary with the address index — r1 gave that as the reason and it was wrong. The real risk is that `MultiALeafScript` (`address/taproot_script_path.go:261-270`) sorts the **serialized derived** x-only keys when building the script, and a port that feeds that already-sorted list into the recipe silently diverges (fable M-8). Reachability: the device composer emits `sortedmulti_a` only for `plain-multisig` (out of scope, §0a), but **`md encode` accepts one in any tree shape** (RUN), so the refusal is not vacuous. Cost is near-zero — Liana emits `multi_a`, never `sortedmulti_a` |
| `kind = 1` with a use-site path other than `<0;1>` | REFUSE. `md encode` accepts `<2;3>`, `<0;1;2>` and `<0;1>/*h` under `tr(H,…)` today (RUN). Liana pairs alternatives positionally and would derive from `0/i` while a use-site-following device derives from `2/i` (fable I-2b). Refusing is narrower than reconciling, and keeps §8.3 honest |
| `kind = 1` with a real internal key extracted | unrepresentable in §3f's sum type. `--unspendable liana` on a path list containing a bare single is a **no-op today**; it must WARN, not silently ignore (fable M-6) |
| `kind = 1` **nested** under `sh`/`wsh` | REFUSE. r0's "any wrapper other than `tr`" was ambiguous between vacuous and this (opus M1); this is the non-vacuous reading and the one that needs a check |
| a version-8 payload reaching a version-4 decoder | `WireVersionMismatch` — single-string via `Header::read` (the dispatch routes it correctly at 8), chunked via `chunk.rs:70` with `got: 8` |
| version 8 on a descriptor whose root `Tag::Tr` (there is at most one, `decode.rs:97-104`) is at `kind = 0` — or which has no `tr` at all | REFUSE at **encode** (minimum-version rule). Accepted at decode so old payloads never become invalid — but note this admits a hand-crafted second encoding of a kind-0 wallet with a different `WalletDescriptorTemplateId` (opus/fable M-4). Documented, not reachable through any encoder |

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

### 6a. What an older toolchain says about a version-8 plate

Version 8 fails closed everywhere (§3d). But *failing closed* and *saying
something true* are different properties, and a journey walk over a mixed
two-board fleet found the messages are wrong in three places. All three are
**this cycle's** to fix, because this cycle is what makes a version-8 plate
exist.

| surface | today | required |
| --- | --- | --- |
| old device, chunked plate | `md.ParseChunkHeader` errors → `gatherIgnored` → **"Not an md1 descriptor chunk."** | a **false statement** about a plate the constellation cut. `gatherIgnored` must be split so a well-formed md1 at an unsupported version says so, and names the version. **This crosses a package boundary:** `errWireVersion` is unexported (`md/md.go:22`), so package `md` must first expose a sentinel or typed error before `gui` can distinguish the case — the sentinel is a `md/` change (stage 3); `gatherIgnored` itself is `gui/` (`gui/mk1_inspect.go:36`, returned at `gui/md1_gather.go:33,39`, printed at `:121`), so BOTH packages have work at that stage |
| `md repair`, older binary | the BCH correction loop **succeeds**, then `decode_with_correction` rejects version 8 and `repair.rs:88-96` returns `Ok(2)`, **discarding the successful correction** — and exit 2 is the atomic-fail code | distinguish "BCH capacity exceeded" from "corrected fine, but this wire version is unsupported"; never discard a correction that succeeded |
| `WireVersionMismatch` Display | `"wire-format version mismatch: got 8, expected 4"` (`error.rs:33`) | "expected 4" becomes false the moment the decoder accepts `{4, 8}`; the message must name the accepted set |

The host error names the version and the device error does not, so §3d's *"loud
and correctly named"* is true of Rust and false of the fork until this is done.

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
- **The device inspect screen names no internal key at all.** `md1Summary`
  (reached via `gui/md1_gather.go` → `md1PolicyFlow`) does not switch on
  `KeyPath`, so §7's two-print-site enumeration does not reach the screen an
  operator uses a year later to ask "which kind are these plates?". It must name
  the kind, or the only answer on the device is the `Policy id:` line — which
  answers correctly only once C2's Go fix lands.
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

### 7a. Device address derivation (the half r1 omitted)

`gui/policy_address.go:132-158` has exactly **two** internal-key branches, and
`md.EmitTapLeavesChunks` returns a two-state `(keyIndex, isNums)`
(`md/tapleaves.go:188`, `:204`). Per §5 the derived xpub takes no slot, so
`byIndex` can never hold it. A kind-1 wallet therefore lands in one of two
wrong places:

| naive port | what the device does |
| --- | --- |
| kind 1 → `isNUMS = true` | derives the **raw H point** — addresses of a *different wallet*, shown on the consent screen and at plate verify. **This is the silent-wrong-address failure this whole cycle exists to prevent.** |
| kind 1 → `isNUMS = false` | `byIndex[ikIndex]` misses, `policyAddressSource` returns `nil, false`, and the operator consents to steel for a wallet the device cannot address |

**Required work, scheduled in §9 and not merely gated:**

1. `md.EmitTapLeavesChunks` returns a **three-state** internal-key kind
   (stage 3, with the rest of the Go port).
2. A **third branch** in `gui/policy_address.go` that recomputes §2 over the
   collected leaf keys and derives at `0/i` / `1/i` (stage 4).
3. **A refusal, not a fallback.** If the device meets an internal-key kind it
   cannot derive, it REFUSES to show an address and must never fall back to the
   NUMS branch. Without this rule a future fourth kind reintroduces row 1 above.

   **Scope of the engrave refusal.** The address-refusal half already ships:
   `complexAddressSource` (`gui/policy_address.go:88`, the entry point every
   screen goes through) probes `src(0, false)` and returns `nil, false` rather
   than falling back — the probe itself is in `complexAddressDeriver` at
   `:188-190`, which is where an implementer should grep. The engrave half is new, and it applies to a policy whose
   internal-key kind *this firmware cannot derive* — not to the shipped D3/D4
   paths that deliberately engrave without an address. State it that way, or it
   reads as a blanket "no address, no engrave" and retires working behaviour.

r1 asserted one choice screen was "the minimum that makes §0 true". It was not:
composing a wallet the device cannot address does not deliver §0's promise.

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
5. **Identity distinctness and stability, in BOTH languages.** Same tree at kind
   0 vs kind 1 → different `WalletPolicyId`, different
   `WalletDescriptorTemplateId`, different 12-word phrase, and a different
   **mk1 `policy_id_stub`**. Every existing v4 id byte-preserved. r0 had no id
   vector at all; r2 had one but scheduled it in Rust only, leaving the Go half
   — the half the operator actually reads — with no owner. **Rust leg: stage 1b.
   Go leg: stage 3.**
6. **Structure-independence pin.** All three of `kofn-recovery`,
   `tiered-recovery` and `decaying-multisig` over the same four keys derive one
   internal key — the third is the nested case.
7. **Render-reparse fixpoint extended to `tr` kind 1**, since
   `format/text.rs:269-274`'s corpus is `wsh`-only.
8. **A live `harnesses/liana` install run at v15.0 — REQUIRED, not a bonus.**
   It is the only measurement of the install path for any unspendable shape.
9. **The operator-facing rulings get vectors too — a rule with no gate is not a
   rule.** r3 stated §0b, §6a and stage 4a as prose and gated only §0b's
   predicate, which is the same shape of gap this spec has already been caught
   on twice (§8.8 declared REQUIRED and scheduled nowhere; §8.5 stated for Rust
   only). Each of these is a test, with its owning stage:

   | what | gate | stage |
   | --- | --- | --- |
   | §0b PLACEMENT | the screen is reached inside `composer_flow.go:96-128` and **before `composerTemplateChunksFor`**; if placed later, the changed-id banner fires | 4 |
   | §0b RESET | **the predicate is re-evaluated, not the edit detected.** Set kind 1, Back-edit to a shape where a conjunct is false (e.g. give the primary a bare single key, making the internal key real), and assert the kind is 0. **And the converse, which is the half that catches an over-eager reset:** Back-edit in a way that keeps both conjuncts true and assert the kind is **still 1** | 4 |
   | §0b DEFAULT ROW | two assertions, because one cannot fail: on FIRST entry the widget opens on the NUMS row (asserted on the first page), and **on RE-ENTRY after choosing kind 1 it opens on the kind-1 row** | 4 |
   | §0b COPY | both rows name their coordinators and say the two are different wallets | 4 |
   | §6a old-device message | a version-8 chunk yields "unsupported wire version", never "Not an md1 descriptor chunk." | 3 |
   | §6a `md repair` | a v8 chunk with a correctable BCH error KEEPS the correction and reports an unsupported wire version, distinctly from the atomic-fail exit | 2 |
   | §6a error Display | `WireVersionMismatch`'s message names the accepted set, not "expected 4" | 1b |
   | §8b `me` fail-open | a bundle never states a plate count it did not compute | 4a |
   | `me` record confirmation | an unsupported wire version is REPORTED, never silently reduced to "unconfirmed" (`sysw/record.rs:251-252`) | 4a |

10. **Mutation testing.** Every property above must be shown to FAIL when the code
   it guards is broken: flip the concat order, sort the keys, deduplicate them,
   drop the version bump, use kind 0's hex at kind 1, pass a constant version to
   the identity hashes, and group the new kind with `KeyPathSpendable`.

### 8b. `me`'s silent skip — a fail-open that version 8 ACTIVATES

`crates/me-cli/src/bundle.rs:371`:

```rust
if let Ok(d) = md_codec::decode::decode_md1_string(s) {
    hashlock_kinds.extend(descriptor_hash_kinds(&d));
    key_slots = key_slots.max(d.n as usize);
    keyless_template |= !d.is_wallet_policy();
}
plates.push(PlateEntry { … });          // pushed regardless
```

On a decode error the block is skipped **silently** while the plate is still
recorded, so `key_slots`, `keyless_template` and `hashlock_kinds` keep their
zero values. The comment three lines above says why that decode is there:
*"a check that covered only the chunked shape would be a completeness claim
with a hole in it."*

This is latent today and **version 8 activates it**: an older `me` — and
`crates/me-cli/Cargo.toml:26` pins `md-codec = "0.42"` from crates.io, which
this cycle cannot bump there because md-codec is unpublished — fails the decode
on every kind-1 plate and then emits a bundle whose *"backup needs N plates"*
claim was computed from nothing.

**Ruling.** A decode failure on a plate that feeds a completeness claim is
reported, never skipped. `me bundle` either refuses the payload naming the
version, or emits the bundle with the completeness claim explicitly marked
unknown — it must not state a count it did not compute. Owned by stage 4a,
gated by §8.9.

### 8a. The last mile, for the runbook rather than the gate

§0's promise ends at a Liana import, and nothing in this spec describes how the
operator gets from plates to that import. Measured, it works and needs no
change: `md encode` over the four keys emits 8 chunk strings, and
`md descriptor` over those 8 emits the concrete descriptor **with the BIP-380
checksum** §8.2 compares (`…#xnta28tv`, RUN). Recorded here so the acceptance
runbook has the two commands; not a gate.

## 9. Sequencing

| stage | content | gate |
| --- | --- | --- |
| **1a** | behaviour-preserving `InternalKey` refactor, wire untouched (§3f) | suite green at 1400+, no wire bytes changed |
| **1b** | version 8, the kind bit, §2 derivation, §4 rendering, §4a's `md decompose` recogniser and `md encode` refusal, §6 refusals | §8 vectors **1, 2, 5, 6, 7, 10** — every leg runnable in Rust alone. (§8.9 is a table of per-stage rows, not a single-stage item; §8.10 is the mutation pass and was item 9 before r3b inserted the gate table) |
| **2** | `md compose --unspendable liana\|nums` (default `nums`), `md descriptor` kind 1, the `UNSPENDABLE(liana)` template substitution rule, the JSON schema version bump (§4a) | §8.2 **and §8.8, the live `harnesses/liana` install run** — the first stage that can render the descriptor the harness consumes |
| **3** | Go port in the fork's `md/`: `EmitTapLeavesChunks` returning a **three-state** internal-key kind (§7a.1), **the version-derived identity ruling at `md/encode.go:417`, `md/template_id.go:53`, `md/walletpolicyid.go:42` (§3e)**, §6a's `gatherIgnored` split, provenance pin bumped | §8 vectors in Go, **including §8.4's `ParseChunkHeader`/`Decode` leg and §8.5's Go identity leg** |
| **4** | device: §7's `KeyPathKind`, the class-2 + unlocked-path ruling, the print-site arms including `md1Summary`, F-633 copy, **§7a.2's third address branch and §7a.3's refusal**, and **§0b's choice screen — predicate, placement, reset, default row and copy** | **§8.3's device leg**, §7's constructed shape, an address test for a kind the device cannot derive, and **§0b's firing predicate exercised on all six `tr` presets, firing on exactly `kofn-recovery` and `tiered-recovery`** |
| **4a** | `me` (this repo): §9a's four pieces — the unpin plus its `[patch.crates-io]` override, §3f's type-change repairs to `me`'s own source, §6a's message at `sysw/record.rs:251-252`, and **§8b's fail-open fix at `bundle.rs:371`** | **§8.9's `me` rows.** NOT "me round-trips a version-8 payload": measured, that already passes on the pinned 0.42 with no change at all, because `me convert` validates only the codex32/BCH layer (`me-cli/src/lib.rs:75-83` → `validate.rs:95-100`), which is version-agnostic. A gate the status quo satisfies is not a gate |
| **5** | rebuild `demo/sh2/` and deploy to quantoshi.xyz/SH2/ with `demo/sh2/update.sh` | site 200, `application/wasm`, and the emulator reaches §0b's screen |

**Every §8 item has an owning stage, and no item is left unowned** (opus I3 and
the r1 fold-check both found §8.8 declared REQUIRED and scheduled nowhere; opus
M2 found stage 1b claiming a gate whose device and Go legs it cannot run). Two
items are deliberately gated twice, at different layers — that is a double gate,
not a duplicate:

| §8 item | owning stage(s) |
| --- | --- |
| 1 recipe vectors, 6 structure pin, 7 fixpoint, **10 mutation** | 1b |
| **9 operator-facing gate table** | each row carries its own owning stage (1b, 2, 3, 4, 4a) |
| 2 descriptor equality | 1b (codec) **and** 2 (through the CLI) — deliberate |
| 5 identity distinctness | 1b (Rust) **and** 3 (Go) — deliberate; the Go half is the one the operator reads |
| 8 live Liana install | 2 |
| 4 dispatch round trip — Go leg | 3 |
| 3 address equality — device leg | 4 |

**Stage 1a/1b do NOT stand alone as r0 claimed** (opus I5). `md-cli` pins
`md-codec = { path = "../md-codec", version = "=0.45.1" }`
(`crates/md-cli/Cargo.toml:28`) in the same workspace, so bumping to 0.46.0
makes resolution fail **before rustc runs**, and §3f breaks four non-test md-cli
sites at compile time (§3f names them). The pin bump and the call-site repairs ship **in the same
commit** as the change that requires them.

### 9a. Stage 4a is four pieces of work, not one

r3 described stage 4a as "unpin and carry the new version". Measured it is
four — three invisible from that description, plus §8b's fail-open, which r3b
had assigned here separately and r4 then dropped from the content column:

1. **The unpin needs a `[patch.crates-io]` override.** Pointing
   `crates/me-cli/Cargo.toml:26` at the local md-codec 0.45.1 and running
   `cargo check -p mnemonic-engrave --all-targets` does **not** compile: the
   graph also needs a `miniscript` patch. Stage 4a names the pin and not the
   override, so an implementer hits a wall that reads like a broken workspace.
2. **§3f's type change breaks `me`'s own source.** §3f enumerates its blast
   radius carefully — 88 md-codec sites, the fork's `md/`+`gui/`, four non-test
   md-cli sites, and it even clears a fifth — and `me` is on none of those
   lists. It is a `md_codec` consumer like any other and its `Body::Tr` uses
   break with them.
3. **§8b's fail-open at `bundle.rs:371`** — a decode failure that leaves the
   bundle's completeness claim computed from nothing. Filed as F-635.
4. **The real version-sensitive surface is `sysw/record.rs:251-252`**, not
   `convert`. It reduces `md_codec::reassemble` / `decode_md1_string` to
   `.is_ok()`, so an unsupported wire version silently classifies a record as
   **unconfirmed** with no message naming anything — the §6a defect shape,
   on `me`. That is what stage 4a's gate points at.

**`me` is downstream and its pin is not bumpable the usual way** (journey I6).
`crates/me-cli/Cargo.toml:26` pins `md-codec = "0.42"` from **crates.io**
(`Cargo.lock`: `source = "registry+…crates.io-index"`), while md-codec is
developed unpublished. `me` is the tool in *this* repo that carries md1 to a
SeedHammer II, so a version-8 payload reaches it — and r2's downstream list
named only `mnemonic-toolkit`. Stage 4a owns it.

**`mnemonic-toolkit` is downstream** (opus M5): `render.rs:9-12` records that its
`inspect` renders the same `template:` line via `descriptor_to_template`,
"guaranteeing byte-identical output across both binaries". §4 changes that
function's output for kind 1. The toolkit pins md-codec by git tag, so it is not
broken by stage 1 — but it is **in scope for this cycle** and gets its pin bump
and a golden refresh after stage 2.
