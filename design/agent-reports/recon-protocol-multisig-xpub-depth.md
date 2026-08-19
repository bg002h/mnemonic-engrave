# RECON: Is `md` right to demand depth == 4 for an xpub in a multisig script context?

**Agent:** external-protocol research (read-only)
**Date:** 2026-08-19
**Scope:** authoritative BIP source text only; no code changes; no unrelated audit.

---

## VERDICT (up front)

**The exact-depth-4 requirement is WRONG. It is simultaneously TOO STRICT and TOO LOOSE.**

- **Too strict:** it rejects xpubs that appear *verbatim, in multisig script contexts, inside the
  normative test vectors of the very BIPs `md` implements* — BIP-383 (`sortedmulti`, depth 1) and
  BIP-388 (`tr(...sortedmulti_a...)`, depth 5). It also rejects the entirety of BIP-87 (depth 3),
  a Complete-status BIP whose sole subject is deterministic multisig hierarchy, and BIP-45
  (depth 1).
- **Too loose:** depth 4 does not imply BIP-48. BIP-388's own vectors reach depth 4 via
  `44'/0'/0'/100'`. Any arbitrary 4-level path (`m/1/2/3/4`) satisfies the check. So the check does
  not establish the property it was written to establish.

**Confidence: HIGH.** This is not inference. The specs are not silent in the ordinary "no one
addressed it" sense — they are silent on depth as a *constraint* (0 occurrences of the word across
all 11 non-BIP-32 BIPs read), while simultaneously publishing multisig test vectors at depths 1, 3,
4 and 5 that the check rejects. A rule that rejects a spec's own test vectors is falsified by that
spec, not merely unsupported by it.

**Answer to the framing question in the brief:** this is *not* a case of "the specs are silent, so
it's a policy choice". Depth-as-metadata **is** settled by BIP-32, and the permissibility of non-4
multisig cosigner xpubs **is** settled by BIP-87/BIP-383/BIP-388 test vectors. A policy choice
remains available *on top* (see Recommendation), but the current rule is not a defensible reading
of the standards.

---

## 1. What the code actually does

`/scratch/code/shibboleth/descriptor-mnemonic/crates/md-cli/src/parse/keys.rs`, lines 67–77, verbatim:

```rust
    let depth = bytes[4];
    let expected_depth = match ctx {
        ScriptCtx::SingleSig => 3,
        ScriptCtx::MultiSig => 4,
    };
    if depth != expected_depth {
        return Err(CliError::BadXpub {
            i,
            why: format!("expected depth {expected_depth} for this script context, got {depth}"),
        });
    }
```

Confirmed: an **exact equality** test on `bytes[4]`, the BIP-32 serialized depth byte. `ScriptCtx`
is a two-valued enum (lines 9–15) — there is no third case, and no scaling with tree size or leaf
count, exactly as the brief states.

The same constants are mirrored in template synthesis at
`/scratch/code/shibboleth/descriptor-mnemonic/crates/md-cli/src/parse/template.rs:681-684`.

### A false citation in the source comment

`template.rs:672-673` states:

> `/// Depth tracks BIP 388 expectation: depth 3 for single-sig (wpkh/pkh), depth`
> `/// 4 for multisig/taproot.`

**This attribution is false and machine-checked as false.** BIP-388 contains the word "depth" zero
times (`grep -ci depth bip-0388.mediawiki` → `0`), and its own test vectors use xpubs at depths
2, 3, 4 and 5. BIP-388 does not merely fail to state this expectation — its vectors contradict it.
This is a "comments outlive their conditions" / "records are the weak half" instance: the rule
acquired a spec-shaped justification it never had.

---

## 2. Method (so this is reproducible, not recalled)

Fetched raw source from `raw.githubusercontent.com/bitcoin/bips/master/` — not summaries, not
memory, not the wiki mirrors:

BIP-32, 43, 44, 45, 48, 49, 84, 86, 87, 380, 381, 382, 383, 386, 388.

Machine checks run (values pasted, not described):

| Check | Result |
| --- | --- |
| `grep -ci depth` across BIP-43/44/45/48/49/84/86/87/380/381/382/383/386/388 | **0 in every one** |
| `grep -ci depth bip-0032.mediawiki` | 5 (positive control — grep works) |
| `grep -ci key` in BIP-380/388/381/386/387 | 73 / 67 / 13 / 25 / 15 (positive control — files are readable) |
| Base58check-decoded depth byte of every xpub in BIP-383 | `{depth 1: 1, depth 4: 1}` |
| ... in BIP-382 | `{depth 1: 1}` |
| ... in BIP-388 | `{depth 2: 1, depth 3: 4, depth 4: 7, depth 5: 3}` |
| `grep -ci "taproot\|p2tr\|schnorr" bip48.mediawiki` | **0** |

Note on the negative results: per the "empty output is not absence" rule, every zero above is
paired with a positive control on the same file, so the zeros are genuine absence, not a broken
command or an unreadable file.

BIP statuses (from the documents' own headers), since "it's only a draft" is a predictable rebuttal:

- BIP-45 — **Complete** — *Structure for Deterministic P2SH Multisignature Wallets*
- BIP-48 — **Deployed** — *Multi-Script Hierarchy for Multi-Sig Wallets*
- BIP-87 — **Complete** — *Hierarchy for Deterministic Multisig Wallets*
- BIP-383 — **Deployed** — *Multisig Output Script Descriptors*
- BIP-388 — **Complete** — *Wallet Policies for Descriptor Wallets*

---

## 3. What the specs say

### 3.1 BIP-32: depth is a positional label, and is normative only at zero

The only definition of the field, from the `Serialization format` section:

> `* 1 byte: depth: 0x00 for master nodes, 0x01 for level-1 derived keys, ....`

That is a description of *where the key sits*, with no statement about what it may be used for.

BIP-32's Test vector 5 enumerates invalid extended keys. The **only** depth-related invalidity
conditions in all of BIP-32 are:

> `(zero depth with non-zero parent fingerprint)`
> `(zero depth with non-zero index)`

So depth participates in validity **exclusively at depth 0**, as an internal-consistency check
against the fingerprint and index fields. There is no rule at depth 3, 4, or any other value.

Worth noting by contrast — BIP-32 *does* impose a normative import check, and `md` correctly
implements it (`keys.rs:83`, the M11 secp256k1 point check):

> `When importing a serialized extended public key, implementations must verify whether the X
> coordinate in the public key data corresponds to a point on the curve. If not, the extended
> public key is invalid.`

This is the useful contrast: **the on-curve check is spec-mandated; the depth check is not.** The
two sit adjacent in the same function and have completely different standing.

### 3.2 The path BIPs define paths, not key-usage constraints

BIP-48 (`Path levels`):

> ```
> m / purpose' / coin_type' / account' / script_type' / change / address_index
> ```

This is where depth 4 comes from, and the rationale as stated in the brief is factually correct
*as a description of BIP-48*. But BIP-48 never says an xpub used in a multisig descriptor must be
at that level, never mentions the serialized depth byte, and — critically — **is not the only
multisig hierarchy.**

Also: **BIP-48 defines no taproot script type.** Its `Script` section says:

> `Currently the only script types covered by this BIP are Native Segwit (p2wsh) and
> Nested Segwit (p2sh-p2wsh).`

and the document mentions taproot/p2tr/schnorr zero times. So for `tr()` — which `md` classifies as
`MultiSig` and demands depth 4 of — the BIP-48 justification does not merely fail, **it does not
exist.** The taproot single-key standard is BIP-86, `m / 86' / coin_type' / account'`, depth 3.

BIP-44/49/84/86 all share `m / purpose' / coin_type' / account' / change / address_index`,
i.e. account level at depth 3. Confirmed by reading each.

### 3.3 BIP-87 — a Complete multisig BIP at depth 3, which explicitly rejects the depth-4 design

This is the most damaging single document for the current rule.

BIP-87 `Path levels`:

> `We should not be mixing keys and scripts in the same layer. The wallet should create extended
> private/public keys independent of the script type, whereas the descriptor language tells wallets
> to watch the multisig outputs with the specified public keys.`
>
> We define the following 5 levels in the BIP32 path:
> ```
> m / purpose' / coin_type' / account' / change / address_index
> ```

`purpose'` is `87'`. The account-level export is therefore `m/87'/coin_type'/account'` — **depth 3,
for multisig.**

And BIP-87 prints the resulting descriptors literally:

> `wsh(sortedmulti(2,[xfpForA/87'/0'/0']XpubA/**,[xfpForB/87'/0'/0']XpubB/**))`

expanding to

> `wsh(sortedmulti(2,[xfpForA/87'/0'/0']XpubA/0/*,[xfpForB/87'/0'/0']XpubB/0/*))`

**`md` rejects this descriptor.** It is a `wsh(sortedmulti(...))` — unambiguously `ScriptCtx::MultiSig`
— with cosigner xpubs at depth 3.

Note the force of the first quoted sentence: BIP-87's *stated reason for existing* is that BIP-48's
4th level (`script_type'`) is a design mistake, because descriptors already encode the script. The
depth-4 rule does not just fail to be normative — it hard-codes the side of a standards
disagreement that the later, purpose-built multisig BIP argues against.

### 3.4 BIP-45 — multisig cosigner xpub at depth 1

BIP-45 `Cosigner Index` section:

> `Note that the master public key is not shared amongst the cosigners. Only the hardened purpose
> extended public key is shared, and this is what is used to derive child extended public keys.`

The "hardened purpose extended public key" is `m/45'` — **depth 1**. This is a BIP-defined multisig
cosigner xpub three levels shallower than the check permits. (BIP-45 is legacy P2SH and arguably
obsolete in practice — I flag it as real but low-weight; BIP-87 and the descriptor vectors below
carry the argument on their own.)

### 3.5 BIP-380 — the descriptor key expression imposes no depth requirement, and origin is optional

`Key Expressions`:

> Key expressions consist of:
> * **Optionally**, key origin information, consisting of:
> ** An open bracket `[`
> ** Exactly 8 hex characters for the fingerprint of the key where the derivation starts (see BIP 32 for details)
> ** Followed by **zero or more** `/NUM` or `/NUMh` path elements to indicate the unhardened or hardened derivation steps between the fingerprint and the key that follows
> ** A closing bracket `]`
> * Followed by the actual key, which is either:
> ...
> ** `xpub` encoded extended public key or `xprv` encoded extended private key (as defined in BIP 32)

(emphasis mine on "Optionally" and "zero or more")

That is the complete normative content of a key expression. Depth is absent. Origin is optional.
The origin path may be **zero elements long**.

BIP-380 goes further and *builds in* the depth variability the check forbids —
`Normalization of Key Expressions with Hardened Derivation`:

> `The exporter should derive the extended public key at the last hardened derivation step and use
> that extended public key as the key in the descriptor.`

The exported xpub's depth is therefore **defined to be wherever the last hardened step falls**,
which is scheme-dependent by construction: depth 4 under BIP-48, depth 3 under BIP-87/BIP-84/BIP-86,
depth 1 under BIP-45. BIP-380 specifies a rule whose output depth is *necessarily* variable.

### 3.6 BIP-383 — the multisig descriptor BIP enumerates its key restrictions, and depth is not among them

BIP-383 states its restrictions explicitly:

> `Depending on the higher level descriptors, there may be restrictions on the type of public keys
> that can be included.`
>
> `Depending on the higher level descriptors, there are also restrictions on the number of keys that
> can be present, i.e. the maximum value of n.`

Two restriction classes: key **type** (compressed/uncompressed) and key **count**. Depth is not a
third. This is an enumerated list in the governing BIP for `multi()`/`sortedmulti()`.

**And BIP-383's own test vector, verbatim (line 80):**

> `sortedmulti(2,xpub6ERApfZwUNrhLCkDtcHTcxd75RbzS1ed54G1LkBUHQVHQKqhMkhgbmJbZRkrgZw4koxb5JaHWkY4ALHY2grBGRjaDMzQLcgJvLJuZZvRcEL/*,xpub68NZiKmJWnxxS6aaHmn81bvJeTESw724CRDs6HbuccFQN9Ku14VQrADWgqbhhTHBaohPX4CjNLf9fq9MYo6oDaPPLPxSb7gwQN3ih19Zm4Y/0/0/*)`

Decoded (machine-checked, base58check + field extraction):

| key | depth | parent_fp | child_index |
| --- | --- | --- | --- |
| `xpub6ERApfZwUNrhL...` | **4** | `78412e3a` | 4294967294 |
| `xpub68NZiKmJWnxxS...` | **1** | `41d63b50` | 2147483648 |

A `sortedmulti(2, ...)` in the Deployed multisig BIP, **mixing depth 4 and depth 1 in one script**.
`md` rejects the second key. Note also that depth is not even required to be *uniform across
cosigners* — an assumption the exact-match rule silently bakes in.

For completeness, BIP-382's `wpkh` vector uses `xpub69H7F5d8KSRgm...` at **depth 1**, which `md`'s
`SingleSig => 3` rule also rejects. The single-sig arm is unsupported by the same argument; the
brief scoped me to multisig, so I record this without pursuing it.

### 3.7 BIP-388 — the standard `md` actually implements — permits depths 2/3/4/5 and mandates no depth

BIP-388 is the source of `md`'s `@i` key-index syntax (`--key @i=XPUB`). Its `KI` definition:

> `A KI (key index) expression consists of:`
> `* a single character @`
> `* followed by a non-negative decimal number, with no leading zeros (except for @0)`

Its `Key information vector` definition:

> Each element of the key origin information vector is a `KEY_INFO` expression, containing an
> extended public key, and (**optionally**) its key origin.

Its `Additional rules` section is an **exhaustive enumeration** of the extra validity constraints
on exactly this data structure:

> - A wallet policy must have at least one key placeholder and the corresponding key.
> - The public keys obtained by deserializing elements of the key information vector must be pairwise distinct.
> - If two `KEY` are `KP/<M;N>/*` and `KP/<P;Q>/*` for the same key placeholder `KP`, then the sets `{M, N}` and `{P, Q}` must be disjoint. ...
> - Repeated `KI` expressions are not allowed inside a `musig` placeholder.
> - The key information vector should be ordered so that placeholder `@i` never appears for the first time before an occurrence of `@j` for some `j < i` ...

**Depth is not in the list.** For a spec that troubled itself to mandate pairwise-distinct pubkeys
and change-set disjointness, the omission is not an oversight.

**BIP-388 test vector — "Taproot wallet policy with `sortedmulti_a` and a miniscript leaf":**

> Descriptor template: `tr(@0/**,{sortedmulti_a(1,@0/<2;3>/*,@1/**),or_b(pk(@2/**),s:pk(@3/**))})`
> Keys info: `["[6738736c/48'/0'/0'/100']xpub6FC1fXFP1GXQpyRFfSE1vzzySqs3Vg63bzimYLeqtNUYbzA87kMNTcuy9ubr7MmavGRjW2FRYHP4WGKjwutbf1ghgkUW9H7e3ceaPLRcVwa", "xpub6Fc2TRaCWNgfT49nRGG2G78d1dPnjhW66gEXi7oYZML7qEFN8e21b2DLDipTZZnfV6V7ivrMkvh4VbnHY2ChHTS9qM3XVLJiAgcfagYQk6K", "xpub6GxHB9kRdFfTqYka8tgtX9Gh3Td3A9XS8uakUGVcJ9NGZ1uLrGZrRVr67DjpMNCHprZmVmceFTY4X4wWfksy8nVwPiNvzJ5pjLxzPtpnfEM", "xpub6GjFUVVYewLj5no5uoNKCWuyWhQ1rKGvV8DgXBG9Uc6DvAKxt2dhrj1EZFrTNB5qxAoBkVW3wF8uCS3q1ri9fueAa6y7heFTcf27Q4gyeh6"]`

- `@0` — depth **4**, origin `48'/0'/0'/100'`
- `@1` — depth **5**, **no key origin at all**
- `@2` — depth **5**, no origin
- `@3` — depth **5**, no origin

`@1` sits inside `sortedmulti_a(1,@0/<2;3>/*,@1/**)` — a multisig script context by any reading —
at **depth 5 with no origin**. `md` rejects it twice over.

A second BIP-388 vector puts multisig keys at depth 4 via a **BIP-44** purpose:

> `wsh(or_d(pk([6738736c/48'/0'/0'/100']xpub...),and_v(v:multi(2,[b2b1f0cf/44'/0'/0'/100']xpub...,[a666a867/44'/0'/0'/100']xpub...,[bb641298/44'/0'/0'/100']xpub...),older(65535))))`

Those `multi(2,...)` cosigners are at `44'/0'/0'/100'`. Depth 4 — but not BIP-48. **This is the
too-loose half of the verdict, demonstrated from spec text:** passing the depth-4 check is not
evidence of a BIP-48 key.

### 3.8 The reference BIP-388 implementation performs no depth validation

`github.com/bigspider/bip388` (the reference implementation by BIP-388's author, Salvatore Ingala).
Cloned and grepped: every occurrence of `depth` is descriptor-parser **recursion** depth
(`MAX_PARSE_DEPTH = 64`, "descriptor / tap-tree nesting depth"). There is **no** validation of the
BIP-32 depth byte anywhere. Key parsing is `Xpub::from_str(...)` — BIP-32 serialization validity
only — and the enumerated policy rules `B2`/`B3` mirror BIP-388's "Additional rules" (index
resolution, pairwise-distinct pubkeys).

Classification: this is **(b) common practice**, not (a) spec — but it is the practice of the
reference implementation of the exact standard `md` targets.

---

## 4. Answers to the four questions

**Q1 — What do the specs say?** Quoted above. Depth is defined once (BIP-32, as a positional
label) and constrained once (BIP-32, only at depth 0, only for internal consistency). The path BIPs
define paths; the descriptor BIPs define key expressions. None constrains the depth byte of a key
in a script context.

**Q2 — Is the depth byte normative for how a key may be used?**
**No.** Depth is conventional metadata. The normative identifying information is the **key origin
(fingerprint + derivation path)**, and even that is *optional* in BIP-380 and BIP-388. BIP-380's
normalization rule makes exported depth an explicit function of the scheme, so a fixed depth
expectation contradicts it. `(a) spec-stated.`

**Q3 — Is there a legitimate workflow producing a multisig cosigner xpub at depth != 4?**
**Yes — at least six, four of them spec-documented:**

| # | Case | Depth | Standing |
| --- | --- | --- | --- |
| 1 | **BIP-87** `m/87'/coin'/account'` | 3 | (a) Complete BIP, purpose-built for multisig; prints `wsh(sortedmulti(...))` |
| 2 | **BIP-383 test vector** `sortedmulti(2,...)` | 1 (mixed with 4) | (a) Deployed BIP's own vector |
| 3 | **BIP-388 test vector** `tr(...sortedmulti_a...)` | 5, no origin | (a) Complete BIP's own vector |
| 4 | **BIP-45** shared `m/45'` purpose xpub | 1 | (a) Complete BIP (legacy) |
| 5 | **Origin-stripped / bare xpub export** | any, incl. 0 | (a) BIP-380 "Optionally"; BIP-388 vector has three origin-less keys |
| 6 | **Single-sig-derived key reused in multisig**; HW exporting at account rather than script level | 3 | (b) practice — BIP-87 §"Account" explicitly anticipates per-account key records |

On the brief's specific sub-cases:
- **`tr()` multi-leaf trees** — BIP-48 defines no taproot script type at all (0 mentions), so
  depth 4 has no basis here; BIP-388's taproot vector uses depth 5.
- **Depth 0 / stripped origin** — permitted as a key expression, but note BIP-32 makes a *forged*
  depth-0 invalid: "zero depth with non-zero parent fingerprint" / "non-zero index". A genuine
  master xpub (depth 0, zero fp, zero index) is valid and legal in a descriptor.

**Q4 — Verdict.** **TOO STRICT and TOO LOOSE simultaneously.** Strict: rejects §3.3–§3.7. Loose:
depth 4 is satisfiable by `44'/0'/0'/100'` (BIP-388's own vector) or by any 4-level path whatsoever,
so it does not establish BIP-48 provenance. A check that both rejects valid input and accepts the
input it was meant to exclude is not a conservative check — it is a miscalibrated one.

---

## 5. Recommendation for this project

I am recommending, not implementing. Any change here is normative codec behaviour → **lands in
Rust first, with test vectors**, per the Rust-primary rule.

### 5.1 Steelman first: what the check was probably protecting against

For an **engraving** tool the stakes are asymmetric and irreversible: an operator who pastes a
*master* xpub, or an xpub from the wrong derivation level, engraves a plate that either cannot
recover the funds or recovers the wrong wallet. Metal is not re-flashable. That concern is real
and I do not want the fix to discard it.

But observe what the current rule actually delivers against that threat: it catches the master-xpub
paste (depth 0 != 4) — and so would any of the alternatives below — while *also* rejecting BIP-87
and BIP-388-conformant setups, and *still* waving through a wrong-level key that happens to land
at depth 4. It is a proxy for the real property, and a poor one.

### 5.2 Recommended: replace the depth equality with an origin-path check, warn-not-reject on depth

**Replace** the exact-depth test with validation of the thing the BIPs actually make meaningful —
**the key origin (fingerprint + path)** — and demote depth to a *consistency* check:

1. **Normative (reject):** keep BIP-32 validity — base58check, 78 bytes, version bytes, on-curve
   point (already correct, and the on-curve check is genuinely spec-mandated). Add BIP-32's real
   depth rule, which `md` currently omits: **depth 0 with non-zero parent fingerprint or non-zero
   child index is invalid** (BIP-32 Test vector 5).
2. **Normative (reject) when origin is supplied:** if a key origin path is present, require
   `depth == origin_path.len()`. This is a genuine internal-consistency invariant, it is
   scheme-agnostic, and it catches the actual paste-the-wrong-xpub error far more precisely than a
   constant does — because it detects *disagreement between the two things the operator supplied*.
3. **Advisory (warn, do not reject):** if the origin path is absent, or present but not one of the
   recognised multisig schemes (`48'/·/·/·`, `87'/·/·`, `45'`), emit a clear warning naming what
   was seen. Do not block. BIP-388 §"Implementation guidelines" explicitly contemplates
   non-standard policies and prescribes *care in backup*, not rejection:

   > `Any implementation in a software wallet that allows wallet policies not matching any of the
   > specifications in BIP-44, BIP-49, BIP-84, BIP-86 (especially if involving external cosigners)
   > should put great care into a process for backing up the wallet policy ...`

Point 2 is the load-bearing replacement. It is strictly stronger than the current check on the
threat that matters (it catches a wrong-level key at *any* depth, not just non-4), strictly weaker
on valid input (it accepts BIP-87, BIP-45, and origin-less keys), and it is derived from BIP-32
rather than from a guess about the user's wallet scheme.

Note this composes with something `md` already has: `cmd/partial.rs` defines
`ORIGIN_UNSPECIFIED_MARKER = "origin: «unspecified — supply on restore»"`, so the codebase already
models keys whose origin is unknown. The present situation is inverted — the *optional* thing
(origin) is treated as optional, while the *non-normative* thing (depth) is mandatory.

### 5.3 Minimum acceptable change, if appetite for churn is low

If the full origin-check is too large a cycle: **widen `MultiSig` from `== 4` to `>= 1`, or drop
the multisig depth check entirely.** Even this narrow change restores BIP-87, BIP-45 and the
BIP-383/BIP-388 vectors. Dropping it entirely matches the reference implementation's behaviour.
I would still fix the false BIP-388 attribution in the `template.rs` comment regardless of which
option is taken, since that comment is what makes the rule look researched.

### 5.4 Test vectors to land with the change (all from spec text, no invention required)

1. BIP-87 §"Address Discovery": `wsh(sortedmulti(2,[xfpForA/87'/0'/0']XpubA/**,[xfpForB/87'/0'/0']XpubB/**))` — must ACCEPT (depth 3, multisig).
2. BIP-383 line 80 `sortedmulti(2, <depth-4 xpub>/*, <depth-1 xpub>/0/0/*)` — must ACCEPT (mixed depths).
3. BIP-388 "Taproot wallet policy with `sortedmulti_a`" — must ACCEPT (depth-5, origin-less key inside `sortedmulti_a`).
4. BIP-388 `wsh(or_d(pk(...),and_v(v:multi(2,[.../44'/0'/0'/100']...))))` — must ACCEPT (depth 4 under purpose 44').
5. BIP-32 Test vector 5: `xpub661no6RGEX3uJkY4bNnPcw4URcQTrSibUZ4NqJEw5eBkv7ovTwgiT91XX27VbEXGENhYRCf7hyEbWrR3FewATdCEebj6znwMfQkhRYHRLpJ` (zero depth, non-zero parent fingerprint) — must REJECT.
6. **Negative control for the new rule:** origin `[fp/48'/0'/0'/2']` paired with a depth-3 xpub — must REJECT on origin/depth mismatch. This is the case the current rule cannot express and the new one can.

Vector 6 is the one that proves the replacement is not merely a loosening.

### 5.5 Caveat on the single-sig arm

`SingleSig => 3` fails by the identical argument (§3.6: BIP-382's own `wpkh` vector is depth 1) and
would be fixed by the same origin-consistency rule. I did not investigate it further because the
brief scoped me to multisig — flagging it so it is not mistaken for something I checked and cleared.

---

## 6. Evidence classification (per the brief)

- **(a) Spec-stated:** everything in §3, all quotations taken from raw `bitcoin/bips` source text.
  Every xpub depth reported was decoded from base58check by script, not read off a document.
- **(b) Common practice:** §3.8 (reference implementation performs no depth validation); Q3 row 6.
- **(c) My inference:** the verdict's "too loose" framing; the §5 recommendation and the claim that
  origin/depth consistency dominates a depth constant on the threat model; the reading of BIP-87's
  "should not be mixing keys and scripts in the same layer" as a deliberate repudiation of BIP-48's
  4th level (the sentence is quoted exactly; the characterisation of intent is mine).

**Nothing in this report is cited from memory.** No citation was manufactured; where a document is
silent I have said so and paired the silence with a positive control.
