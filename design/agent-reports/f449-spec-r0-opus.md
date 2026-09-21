# R0 architect review — SPEC_liana_unspendable_internal_key.md (F-449)

**Reviewer:** independent architect (opus), R0 gate, 2026-09-21
**Artifact:** `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_liana_unspendable_internal_key.md` (DRAFT r0)
**Question asked:** is this spec correct, complete and unambiguous enough that a competent
implementer with no context can build it without making a load-bearing guess?

## VERDICT: 2 Critical / 6 Important / 5 Minor / 2 Nit — NOT GREEN

The measured half of this spec is strong: §2's recipe is genuinely pinned by the evidence
(including, as I verified below, the multi-key-leaf and nested-taptree orderings), and §4's
SkeletonKey argument is sound. What blocks the gate is §3: the chosen version number cannot
be decoded by the codec it is written for, and the version-threading §3e calls "mechanical"
lands on two funds-relevant identity hashes the spec never names.

---

## C1 — Critical. §3d: `WF_UNSPENDABLE_VERSION = 5` is an ODD version, and the in-band chunked-flag auto-dispatch makes every single-payload v5 string undecodable

**Location:** spec §3c, §3d (`Add Header::WF_UNSPENDABLE_VERSION = 5`; "The decoder accepts
`{4, 5}`").

**Reproduction.** The md1 wire has an in-band chunked/single auto-dispatch that reads **bit 0
(LSB) of the first 5-bit symbol**, *before* any header is parsed:

`crates/md-codec/src/decode.rs:191-193`
```rust
let chunked_flag = bytes.first().map(|b| (b >> 3) & 0x01).unwrap_or(0);
if chunked_flag == 1 {
    return crate::chunk::reassemble_with_opts(&[s], opts);
}
```
and the same test again at `crates/md-codec/src/chunk.rs:650-651` (`decode_with_correction`'s
single-string pre-pass).

The single-payload first symbol is `[divergent][v3][v2][v1][v0]`, so **bit 0 is `v0`**. The
chunk-header first symbol is `[v3][v2][v1][v0][chunked=1]`, so bit 0 is always 1 there. The
dispatch is only unambiguous while every usable single-payload version is **even**. That is
not incidental — it is the recorded constraint:

`crates/md-codec/src/header.rs:4` — *"4-bit version field (v0.30 = 4; usable WF-redesign set
{4, 8, 12} per §2.4)"*
`crates/md-codec/src/decode.rs:171-173` — *"The usable single-payload version set {4,8,12} is
all-even ⇒ every currently-valid single-payload string has first-symbol LSB = 0, so this
dispatch never diverts an input that decodes today."*
`descriptor-mnemonic/design/SPEC_v0_30_wire_format.md:129` §2.4 — *"The auto-dispatch reads bit
0 (`v0`); if `v0 = 1`, the symbol is interpreted as chunked... Therefore WF-redesign versions
must have `v0 = 0` (i.e. even values)."*

Version 5 = `0b0101`, so `v0 = 1`. Computed:

```
version=4 divergent=0: first-symbol=0b00100 byte0=0x20 -> chunked_flag=0  => single-payload (Header::read)  [today]
version=5 divergent=0: first-symbol=0b00101 byte0=0x28 -> chunked_flag=1  => reassemble; ChunkHeader::read version=2  -> WireVersionMismatch{got:2}
version=5 divergent=1: first-symbol=0b10101 byte0=0xa8 -> chunked_flag=1  => reassemble; ChunkHeader::read version=10 -> WireVersionMismatch{got:10}
```

(`ChunkHeader::read` takes the first **4** bits as the version — `chunk.rs:68-71` — so it reads
bits 4..1 of the single-payload symbol, i.e. 2 or 10.)

**Why it matters.**

1. §3d's rule *"The decoder accepts `{4, 5}`"* is **unimplementable at the string entry point.**
   Adding `5` to `Header::read` (`header.rs:42`) changes nothing: `decode_md1_string` never
   reaches `Header::read` for a v5 string. A brand-new v5 plate is undecodable by a v5 decoder.
2. The failure is **mis-diagnosed, not loud.** `WireVersionMismatch { got: 2 }` is the exact
   error the wire spec's own §2.5 trace reserves for *"v0.x chunked read as single-payload"*
   (`header.rs:100-110` pins it as a test). A v5 plate would tell the operator their card is a
   pre-redesign v0.x card. §3c's whole argument — *"a version bump makes every existing decoder
   refuse the new form **loudly**, with no misparse"* — does not hold at the number chosen.
3. It is **asymmetric in a way that will look like corruption.** A *chunked* v5 payload works
   (the chunked flag really is 1, and `chunk_payload` at `chunk.rs:268-289` embeds the full
   single-payload header, so `reassemble` → `decode_payload` → `Header::read` sees 5). So the
   same wallet decodes when it spans two plates and fails when it fits on one.

This is not a typo-level fix: §3d's "one bit, and version 6 for a future third recipe" reasoning
is built on a 4→5→6 numbering the format does not have (see I1).

---

## C2 — Critical. §3e/§3f: the two identity hashes call `write_node` directly, and the spec never says which wire version they serialize under — the default reading collides kind 0 and kind 1

**Location:** spec §3e ("`read_node` ... and `write_node` ... have no version parameter. The
wire version must be threaded into both ... This is mechanical but it touches every caller, and
the plan must enumerate them"); §3f (the `InternalKey` sum type).

**Reproduction.** `write_node` has exactly three non-test callers. §3e names none of them, and
two are identity algorithms:

| caller | what it computes |
| --- | --- |
| `crates/md-codec/src/encode.rs:181` | the wire payload |
| `crates/md-codec/src/identity.rs:90` | `WalletDescriptorTemplateId` (§8.1 γ-flavour) |
| `crates/md-codec/src/identity.rs:200` | `WalletPolicyId` (§5.3) → `to_phrase()`, the 12-word identity phrase |

Both identity calls hash the tree **without any header**:

`identity.rs:89-90`
```rust
d.use_site_path.write(&mut w)?;
crate::tree::write_node(&mut w, &d.tree, kiw)?;
```
`identity.rs:199-201`
```rust
let mut tree_w = BitWriter::new();
crate::tree::write_node(&mut tree_w, &d.tree, d.key_index_width())?;
let canonical_template_tree_bytes = tree_w.into_bytes();
```

There is **no `version` field on `Descriptor`** (`encode.rs` builds the header from a constant at
`:173-176`), so once `write_node` gains a version parameter the implementer must invent what
these two sites pass. Two readings, both defensible:

- **Reading A (pass `WF_REDESIGN_VERSION`).** The obvious choice, and the one that trivially
  preserves every already-computed id. At version 4 no `kind` bit is written (§3d: *"At version
  4 the stream is unchanged and no `kind` bit is read or written"*), so
  `InternalKey::NumsPoint` and `InternalKey::LianaUnspendable` emit **identical bytes**.
  Consequence: two descriptors that §8.3 requires to have **different addresses** share one
  `WalletPolicyId`, one 12-word identity phrase, and one `WalletDescriptorTemplateId`.
- **Reading B (derive the minimum version from the tree, per §3d's encoder rule).** Correct;
  kind-1 trees hash with the extra bit, every existing v4 id is byte-preserved.

**Why it matters.** `WalletPolicyId`'s documented contract (`identity.rs:120-127`) is that *"two
engravings of the same logical wallet produce identical IDs"* — its contrapositive is what the
operator relies on when they read the phrase back. Reading A silently breaks it for exactly the
pair of wallets this cycle creates. The spec itself flags this defect class one section earlier
— §4: *"a shared skeleton key would be a false-evidence-match"* — and then does not carry the
reasoning to the two hashes where it is worse. Nothing in §8's six required items would catch
it: there is no id-distinctness vector.

Note `compute_md1_encoding_id` (→ `chunk_set_id`) is **not** affected: it goes through
`encode_payload_for_identity`, which includes the header (`identity.rs:45`).

---

## I1 — Important. §3: the usable WF version set is {4, 8, 12}, not 4→5→6 — the bump spends one of the format's last two generations, and §3b ruled out the alternatives against the wrong cost

**Location:** §3b ("Why this cannot be additive"), §3d ("A future third recipe would require
version 6, and v5 decoders would reject it").

**Reproduction.** `design/SPEC_v0_30_wire_format.md:129-130` and
`design/agent-reports/spike-v0.30-q10-pre-spec.md:25`:

> **Usable WF-redesign versions: {4, 8, 12}.** v0.30 uses 4; future major breaks would use 8
> then 12. After 12 is consumed, the next break requires a format-layer change (e.g. widening
> the version field to 5 bits, which would itself break the discriminator placement). The
> 3-version lifetime is intentional.

So the real cost of §3d is not "the next integer". It is **one of the two remaining wire-format
generations, permanently**, spent on a single bit. And "version 6" in §3d's fail-closed argument
does not exist either — the third recipe would have to be 12, the last one.

**Why it matters.** §3b's conclusion ("this cannot be additive") is a cost comparison: a new bit
desyncs, a TLV is worse, a sentinel does not fit, *therefore* bump the version. That comparison
was made against a cost of ~zero. Against a cost of 1-of-2-remaining-generations, a reviewer of
§3b would reasonably want the rejected options re-argued — in particular whether this feature
should be batched with whatever else wants version 8, rather than consuming it alone. The spec
must at minimum state the real version budget; it currently implies an unbounded one.

---

## I2 — Important. §2 step 4 tells the implementer to copy a field the md1 wire does not carry, and §2 step 1 indexes a representation md-codec does not have

**Location:** §2 steps 1 and 4 (marked **normative**).

**Reproduction.** §2 step 1: *"for an xpub in `[origin]xpub.../<0;1>/*` form, that is bytes
`[45..78]` of the base58-decoded 78-byte payload"*. §2 step 4: *"version bytes copied from the
leaf xpubs (mainnet `xpub` / testnet `tpub`)"*.

md-codec's TLV pubkey entries are **65 bytes, `chain code ‖ compressed pubkey`** — there is no
base58 payload, no `[45..78]`, and **no version bytes at all**:

`crates/md-codec/src/validate.rs:331-335`
```rust
for (idx, xpub) in entries {
    if bitcoin::secp256k1::PublicKey::from_slice(&xpub[32..65]).is_err() {
```
`validate.rs:348` states it explicitly: *"The 65-byte `chain code ‖ compressed pubkey`"*.

Network is a **render-time CLI parameter**, not wire content — `md descriptor --network`,
`md address --network` (`crates/md-cli/src/main.rs:1081, 1120`).

**Why it matters.** A normative section directs the implementer to a nonexistent source. The
derivation itself is fully determined (sha256 over the 33-byte pubkeys in tree order — the md
representation already stores exactly those bytes at `[32..65]`), but the version-byte rule has
no mechanical translation, and the implementer must guess: the `--network` flag (correct,
inferrable) or a hardcoded mainnet (wrong for tpub wallets, and silently so — the derived
addresses would be right while the emitted descriptor string is the wrong network for Liana).

Related, and worth stating in the same place: §2's bolded **"VERIFIED INDEPENDENTLY ... measured,
not transcribed"** is scoped to *"all four shapes Liana accepted"*. All eight evidence
descriptors in `fable-liana-parse-in.jsonl` are **mainnet `xpub`**. The `tpub` branch of step 4
is transcribed, not measured, and §8's vectors add no testnet coverage.

---

## I3 — Important. §4 specifies an output spelling for the keyless template with no input spelling, breaking the documented `md compose` → `md descriptor --template` round trip and the renderer's own shipped fixpoint property

**Location:** §4, row 2 (keyless template, `md compose` stdout → the marker
`UNSPENDABLE(liana)`); §9 stage 2.

**Reproduction.** §4 defends the marker's safety by citing `descriptor_to_abstract_template`'s
consumers — correct for row 3, but row 2 is a **different function**,
`descriptor_to_template`, and its output is a documented *input*:

`descriptor-mnemonic/README.md:129`
```
md descriptor --template "<template>" --key "@0=[fp/path]xpub…" # T → descriptor
```
`md compose` emits it (`crates/md-codec/src/compose/mod.rs:632`), `md decode` and `md inspect`
print it (`cmd/decode.rs:66`, `cmd/inspect.rs:95`).

That input path runs the template through **rust-miniscript**: `parse_template` →
`parse_template_ext` (`crates/md-cli/src/parse/template.rs:2613, 2654`) → `substitute_synthetic`
(`:1045-1047`, which rewrites `@i/...` to a synthetic xpub) → `Descriptor::from_str` →
`walk_tr` (`:1589`), which only then compares the internal key string against
`NUMS_H_POINT_X_ONLY_HEX` (`:1601`). `UNSPENDABLE(liana)` has no substitution rule and is not a
valid descriptor key expression, so `Descriptor::from_str` fails before md's own code sees it.

The renderer also ships an explicit fixpoint property asserting this cannot happen —
`crates/md-cli/src/format/text.rs:269-274`:
```
"render emitted a template that does NOT re-parse\n  input: {candidate}\n  rendered: {r1}\n  error: {e}"
```
Its corpus is `wsh(...)`-only, so it will stay green while the invariant it names is false for
`tr` kind 1.

**Why it matters.** §9 stage 2 schedules *"`md descriptor` **rendering** kind 1"* and nothing
about parsing it. As written, the composer's own printed template is the one template md cannot
read back. Two readings the spec leaves open: (a) the marker is render-only and the template
path is simply closed for kind-1 wallets (which should be an explicit §6 refusal with a message,
not a rust-miniscript parse error), or (b) md-cli gains a substitution rule for the marker — a
stage-2 work item that is currently unscheduled.

---

## I4 — Important. §8's acceptance deduction proves *parse*, not *import*, and the "Bonus" clause dismisses the stronger test with a reason the evidence contradicts

**Location:** §8 ("The gate is byte-equality against descriptors Liana has already accepted…
then Liana accepts md-codec's output"); §8 "Bonus, not required".

**Reproduction.** The measurement cited is `LianaDescriptor::from_str`. The same evidence
directory separately records two *further* layers, and records them for **different shapes**:

- `design/evidence/composer-fable-r0/fable-liana-lianad.out` — live `lianad` 8.0.0 install +
  spend, for `X16-wsh-kofn-older5`, `X18-tr-inherit-older5`, `X02`, `X10`, `X13`, `X24`.
- `fable-liana-core-v25-import.out`, `fable-liana-core-v31-import.out` — Core
  `importdescriptors` (`import ok; descs=2 recv==md [...]`).

Neither covers **any** `liana-unspendable-xpub` variant. I checked: the eight unspendable
records in `fable-liana-parse-out-v15.jsonl` carry `{ok, error, receive, change, liana_desc,
receive_desc, is_taproot, primary, recovery}` — parse output and derived addresses only.

So §8's *"Bonus… it re-measures what the evidence already records"* is false: a live
`harnesses/liana` run would measure the **install** path, which the evidence records for zero
unspendable shapes.

**Why it matters.** §0's deliverable is stated as *"so that Liana **imports** the resulting
descriptor"*, and §1's baseline counts shapes that *"import into Liana"*. The gate measures
`from_str`. `from_str` is a strong proxy — Liana's policy-model refusals do come back through it
(`"Descriptor is not compatible with a Liana spending policy."` appears as the `error` on the
refused records) — but the spec asserts *equivalence* between the proxy and the promise, and then
uses that assertion to demote the one test that would close the gap. Either the promise should be
scoped to what was measured, or the live run stops being a bonus.

Secondary, same section: §8.2 requires md's output to be *"byte-identical to the evidence input"*.
The evidence inputs carry descriptor checksums (`#8jc8gq6v`, `#sg4a2yu7`, …). The spec does not
say whether the comparison includes the checksum, which is the difference between a real gate and
one that fails on formatting.

---

## I5 — Important. §9: stage 1 does not stand alone — the same-workspace exact version pin fails resolution before any compiler runs

**Location:** §9 ("Breaking — `md-codec` 0.46.0"; *"Each stage gates independently. Stage 1 is
the only one that can proceed without the ones above it."*).

**Reproduction.** `md-cli` and `md-codec` are one Cargo workspace
(`descriptor-mnemonic/Cargo.toml:3`), and md-cli pins md-codec **exactly**:

`crates/md-cli/Cargo.toml:28`
```toml
md-codec = { path = "../md-codec", version = "=0.45.1" }
```

Bumping the path crate to 0.46.0 makes that requirement unsatisfiable — the workspace fails to
resolve before rustc is invoked. Independently, §3f's `Body::Tr` field change breaks md-cli at
compile time in at least five non-test sites: `format/json.rs:348`, `parse/reuse.rs:515`,
`parse/template.rs:1604` and `:1629`, `seat/compose.rs:148` — plus ~10 test-site constructions in
`parse/template.rs`.

**Why it matters.** Under this project's gate a red suite is itself a blocking finding, so
"stage 1 gates independently" is not achievable as stated: stage 1 must carry the pin bump and
the md-cli call-site repairs to be green. The plan derived from this §9 would schedule a stage
that cannot close.

---

## I6 — Important. §7 specifies only that class 2 must stop firing, and leaves the new kind's effect on the *unlocked-path count* unspecified — one reading produces a false ACCEPT verdict

**Location:** §7, Go bullet.

**Reproduction.** `seedhammer/gui/composer_consent.go` makes "does this key path count as an
unlocked spend path?" a **separate, explicitly load-bearing** decision from the class-2 check.
Its own doc comment, `:373-380`:

> A REAL SPENDABLE TAPROOT KEY PATH COUNTS AS AN UNLOCKED PATH (lens 5 I-2): `md.KeyPathSpendable`
> means "a real key can spend directly, WITHOUT satisfying any leaf" … **KeyPathNUMS does not
> count: a NUMS key spends no path at all.**

Class 2 is at `:397` (`if shape.KeyPath == md.KeyPathNUMS`). §7 says only: *"`md.KeyPathNUMS`
gains a sibling … must stop returning `"NUMS key path"` for kind 1 … The classes **behind** it
must then fire normally."* It never says which side of the `KeyPathNUMS` / `KeyPathSpendable`
line the sibling falls on for the unlocked-path count.

**Constructed failing case.** `tr` at kind 1 whose taptree is a single timelocked leaf —
`tr(<derived xpub>,and_v(v:pk(@0),older(26280)))`. Correct verdict: Liana class 7, *no unlocked
path* → REFUSE. If the sibling is grouped with `KeyPathSpendable` (the reading §7's "stop
returning NUMS key path" most directly suggests — it moves the new kind *out* of the NUMS arm),
the key path is counted as unlocked, class 7 does not fire, and the composer tells the operator
the wallet is Liana-compatible **before the plates are cut**. That is precisely the deliverable
§0 names for the out-of-scope presets.

**Why it matters.** The three presets §0 lists stay refused either way (`plain-multisig` still
hits class 3, `decaying-multisig` still hits class 5 first, `hashlock-gated` still hits class 4),
so the spec's own examples do not expose the ambiguity — which is why it needs stating. The
general case flips.

---

## Minor

**M1 — §6 row 2 is unrepresentable, not a refusal.** *"`kind = 1` under any wrapper other than
`tr` | REFUSE"*. Under §3f, `kind` lives on `Body::Tr`, which only exists under `Tag::Tr` — so
this row is in the same class as row 3 ("unrepresentable by construction"), not a refusal to
implement. Two readings: (a) it is vacuous, or (b) it means a `tr` **nested** under `sh`/`wsh`,
which is a different and non-vacuous check. As written an implementer cannot tell which.

**M2 — §3e names two read-side version checks and omits the write side.** §3e cites
`chunk.rs:70` and `:373` (both in `ChunkHeader::read` / the reassembly consistency loop). The
chunk **writer** hardcodes the version too — `chunk.rs:279`, `version: Header::WF_REDESIGN_VERSION`
— as does `encode.rs:174`. Leaving `:279` at 4 makes a chunked v5 set carry chunk headers that
disagree with their own payload (still fail-closed, but the header lies); changing it makes a
v4 decoder reject at `:70` instead. The spec should pick one.

**M3 — §7's `KeyPathKind::Unspendable` contradicts a shipped ruling and reads ambiguously beside
`Nums`.** `crates/md-codec/src/policy_shape.rs:119-133` records a prior review's decision in the
source: *"recognising that needs re-deriving a specific coordinator's `unspendable_internal_key`-
style function over the whole descriptor (Liana's does exactly this), which makes it a coordinator
RULE, not a codec-observable property. **Do not "restore fidelity" with the Go name here**"*. This
spec makes it codec-observable, which is a legitimate change — but the doc comment must be
retired in the same change or the source actively instructs the next reader not to do what the
spec requires. Separately, `Unspendable` sitting next to `Nums` implies `Nums` is spendable.

**M4 — §8 does not define byte-equality w.r.t. the descriptor checksum.** See I4, second half.

**M5 — §9 omits `mnemonic-toolkit`.** `crates/md-codec/src/render.rs:9-12` states that *"the
`mnemonic` toolkit's `inspect` renders the same `template:` line by calling
[`descriptor_to_template`] — guaranteeing byte-identical output across both binaries."* §4 row 2
changes that function's output for kind 1, so the toolkit is a downstream of this cycle. It pins
md-codec by git tag (`mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml:34-36`), so it is not
*broken* by stage 1 — but §9's five stages should say whether it is in or out.

## Nit

**N1 — §6 row 5's "every `Body::Tr`" is vacuously true for non-`tr` descriptors.** A descriptor
has at most one root `Tag::Tr` (`decode.rs:97-104` allow-list), so the quantifier is over a set of
size 0 or 1; the rule as phrased says a `wsh` descriptor at version 5 must be refused at encode,
which is presumably intended but arrives by accident.

**N2 — §2's "surprising" property is under-cited.** §2 notes that `preset-kofn-recovery-tr` and
`preset-tiered-recovery-tr` share an internal key. `preset-decaying-multisig-tr` shares it too
(verified: same xpub `xpub661MyMwAqRbcFswVugWFBxmD7r3bQLsmHovc3p3wTFfgk3EWb36m3QfsezgaR6h5cXXgPG3R2XmctBn55sAt35wzLnrYy82sLKYF8CRsak7`
across all three). That third case is the stronger one for §8.5's pin, because it is the only
evidence shape with a **nested** taptree `{A,{B,C}}` — it is what proves the traversal is
depth-first left-to-right rather than per-branch.

---

## What I checked and found sound

- **§2's ordering rule is genuinely pinned by the evidence, including the two cases I expected
  to be gaps.** *Multi-key leaves:* `preset-tiered-recovery-tr` has `multi_a(2,A,B)` and
  `and_v(v:multi_a(1,C,D),older(...))` and derives the same internal key as
  `preset-kofn-recovery-tr`'s `multi_a(2,A,B,C)` + `and_v(v:pk(D),...)` — so the concatenation is
  a **flat left-to-right walk of key expressions**, not per-leaf. *Nested taptrees:*
  `preset-decaying-multisig-tr`'s `{A,{B,C}}` over the same four keys in the same order derives
  the same key. Both are the behaviours an implementer would have had to guess, and both are
  measured. §2 does not need a clarification here.
- **§4's SkeletonKey requirement is satisfied twice over and the abstract-template claim holds.**
  `skeleton_key` (`skeleton.rs:313-320`) already appends `key_path_kind_label(s.shape.key_path)`,
  so §7's fourth `KeyPathKind` alone makes kind 0 and kind 1 produce different keys even before
  the marker differs. `descriptor_to_abstract_template`'s only consumer in the skeleton path is
  `skeleton.rs:203`, as a string component — not re-parsed. Row 3 is safe. (Row 2 is not — I3.)
- **§4's "the derived xpub cannot appear in a keyless form" is correct and correctly reasoned.**
  The chain code is a function of the seated leaf pubkeys, and md1 stores those only in the
  optional `pubkeys` TLV; a template-only card has none.
- **§5's ruling (the derived key takes no numbered slot) is right, and I could not find a
  downstream it breaks.** `policy_shape`'s key-path branch is gated on "is there a real key"
  (`policy_shape.rs:267`, `if !*is_nums`), which the sum type expresses directly;
  `canonicalize_placeholder_indices` renumbers `@i` but does not reorder the tree, so the derived
  key is invariant under canonicalisation; `fp_partition` / `key_partition` are slot-indexed and
  unaffected. §5's BIP-388 concession is also accurate: kind 0 is already a literal (x-only hex)
  in a position BIP-388 wants to hold a placeholder, so kind 1 (a literal xpub) is genuinely "no
  worse".
- **§3b's three rejections are individually correct.** The trailing-bit desync, the F-417
  preserve-and-ignore TLV trap, and the `key_index_width = ceil(log2(n))` sentinel exhaustion
  (`decode.rs:88`) all hold as stated. What fails is the conclusion's cost model, not the
  premises (I1).
- **§3d's "one bit, not two" argument is sound on its own terms** — a 1-bit field has no
  undefined value, so no error path exists to forget to write. It is only the version *number*
  that is wrong (C1) and the version *budget* that is mis-stated (I2/I1).
- **§0's scope exclusions are correct and the refusal-preservation concern is real.** I confirmed
  the three excluded presets fire their stated classes ahead of anything kind 1 changes
  (`composer_consent.go`'s ordered class list), so §0's "this spec must not weaken that refusal"
  survives kind 1 for those three specifically. The general case is I6.
- **§8's items 1, 3, 4, 5 and 6 are runnable as written.** The evidence records `receive` and
  `change` address triples per shape (indices 0..2, mainnet `bc1p…`), so item 3 has something to
  compare against — I checked rather than assumed, since a gate that has never executed is a
  hypothesis. Item 6's mutation list (flip concat order, sort, dedup, drop the version bump, use
  kind 0's hex) targets the right five failure modes.
