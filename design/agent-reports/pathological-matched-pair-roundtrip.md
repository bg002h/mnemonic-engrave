# Pathological wallet — matched `wsh` / `tr` pair, and what round-tripping it proved

**Run 2026-08-18 by the controller.** Every value below is pasted command output.
User directive: *"The pathological wallet must roundtrip as wsh and tr wrapped
wallets."*

---

## PRIORITY 1 — `md encode --from-policy --context tap` is broken for ordinary taproot wallets

**Found while answering "is there a right way to nest?", not while looking for
it. This is a shipped command failing on unremarkable input, and it outranks
everything else in this report.**

### The failure

```
$ md encode --from-policy 'thresh(1,pk(@0),pk(@1),pk(@2),pk(@3),pk(@4))' --context tap
md: template parse error: miniscript parse failed: taptree branch must have 2 children, but found 1
```

**A plain 1-of-5 taproot wallet cannot be compiled by our CLI.** So cannot the
pathological wallet's own policy. Measured across six policies:

| policy | result | taptree built |
| --- | --- | --- |
| `or(pk(@0),pk(@1))` | OK | `tr(@0,pk(@1))` |
| `thresh(1,pk(@0),pk(@1),pk(@2))` | OK | `tr(@0,{pk(@1),pk(@2)})` |
| `or(4@pk(@0),1@or(pk(@1),pk(@2)))` | OK | `tr(@0,{pk(@1),pk(@2)})` |
| `or(1@or(pk(@0),pk(@1)),1@or(pk(@2),pk(@3)))` | OK | `tr(@0,{pk(@1),{pk(@2),pk(@3)}})` |
| `or(pk(@0),or(pk(@1),or(pk(@2),pk(@3))))` | **FAIL** | — |
| `thresh(1,pk(@0),…,pk(@4))` (1-of-5) | **FAIL** | — |

### Root cause, with file:line

`crates/md-cli/src/compile.rs:95-100` round-trips the compiled descriptor
**through a string**:

```rust
    .compile_tr(unspendable)
// Descriptor::to_string() includes a trailing #<8-char-checksum>
let rendered = desc.to_string();
```

That `to_string()` is rust-miniscript's pre-#953 `Display`
(`descriptor-mnemonic/vendor/miniscript/src/descriptor/tr/taptree.rs:92-113`), which tracks only the
depth *change between adjacent leaves*. Reading the algorithm gives the exact
condition, sharper than the "depth ≥ 2" proxy in use:

> **It is correct exactly when the leaf-depth sequence never decreases** — i.e.
> only for right-spine "caterpillar" trees. Traced: `[1,2,2] → {A,{B,C}}` ✓;
> `[2,2,2,2] → {{A,B,C,D}}` ✗; `[2,2,1] → {{A,B,C}}` ✗.

Every OK row above is a caterpillar; every FAIL is not. The bug is
**topology-dependent, not depth-dependent**.

### Why it was not caught before

Two reasons worth recording. The **template** route
(`md encode '<template>'`) never touches `Display` — it parses the string md was
handed — which is why the `tr` sibling in §2 encodes fine while the equivalent
policy does not. And `--from-policy` requires the non-default `cli-compiler`
feature, so a default build cannot reach the defect at all.

### Mitigating, and not mitigating

**It fails loudly** — a parse error, not silent corruption — because md re-parses
what it rendered. Any consumer that took `Descriptor::to_string()` *without*
re-parsing would get a malformed descriptor with no error at all. That is the
argument for treating this as urgent rather than cosmetic.

### Fix directions — not chosen here

1. **Stop round-tripping through a string.** Convert the `compile_tr`
   `Descriptor` into md's own AST directly, so `Display` is never invoked. This
   needs a `Descriptor → md AST` converter; only the *forward* direction
   (`to_miniscript.rs`) is known to exist, so the reverse may be new work.
   **Fixes it with no dependency change**, and would also remove the last
   production caller of the broken renderer.
2. **Advance the miniscript pin past #953.** But see
   `wallet-policy-pin-regime-differential.md`: `md-cli` does not currently
   compile against `ff4732e` — two PR #915 API breaks — so this is not the cheap
   option it looks like.
3. **Refuse early with a good message.** Detect a non-caterpillar tree before
   rendering and say so, instead of surfacing miniscript's parse error. Strictly
   worse than (1) but far better than today.

**(1) is the recommendation.** It is the only option that neither waits on an
upstream release nor accepts the failure.

---

## 1. Correction first — the pathological wallet is `wsh`, not `tr`

It was described in conversation as a "deeply nested tr" wallet. Measured
against `design/journeys/inputs-pathological/wallet-policy.txt`:

| fragment | count |
| --- | --- |
| `wsh(` | 1 |
| `or_i(` | 3 |
| `and_v(` | 6 |
| `after(` / `older(` | 2 / 2 |
| `sha256(` | 2 (identical digest both times) |
| `multi(` | 4 — **unsorted**; 3-of-3, 2-of-3, 2-of-2, 1-of-3 |
| **`tr(` / `multi_a(` / `taptree`** | **0 / 0 / 0** |

11 slots `@0…@10` from 3 masters (`master-A/B/C.seed`). It is genuinely
pathological and genuinely 11-key, but it contains **no taproot at all**, so
round-tripping it exercises zero `tr` behaviour. Hence the matched pair.

## 2. The `tr` sibling, as constructed

Faithful translation: each `or_i` branch becomes a **taptree leaf**, preserving
the right-skewed nesting, and each `multi` becomes `multi_a` (tapscript has no
`OP_CHECKMULTISIG` — BIP-342 uses `CHECKSIGADD`). Internal key is the BIP-341
NUMS point, so the key path is unspendable and every spend goes through a leaf.

```
tr(NUMS, { A , { B , { C , D } } })

A = and_v(v:after(1000000),    and_v(v:sha256(a84d…08ad), multi_a(3,@0,@1,@2)))
B = and_v(v:after(1893456000), and_v(v:sha256(a84d…08ad), multi_a(2,@3,@4,@5)))
C = and_v(v:older(65535),      multi_a(2,@6,@7))
D = and_v(v:older(4255898),    multi_a(1,@8,@9,@10))
```

Installed as `design/journeys/inputs-pathological/wallet-policy-tr.txt`
(502 bytes) with its keyless md1 at `backup-strings-tr.txt` (3 chunks).

**Note the topology choice, because it is load-bearing — and it is also the
economically correct one, which is a coincidence worth not relying on.**

Right-skewed `{A,{B,{C,D}}}` mirrors `or_i(A,or_i(B,or_i(C,D)))` exactly. It is
also a caterpillar, so it is immune to the #953 `Display` defect described in
PRIORITY 1 — but *that is an implementation accident, not a design reason*, and
the two must not be conflated.

**The design reason is cost.** Taptree topology is semantically free — any tree
over the same leaf set authorizes the same spends — but a BIP-341 control block
is `33 + 32×depth` bytes, so depth is charged in witness bytes per leaf. The
*right* tree is therefore the Huffman tree weighted by spend probability, which
is exactly what policy-language weights (`9@`, `1@`) and `compile_tr` are for.

| | A | B | C | D |
| --- | --- | --- | --- | --- |
| right-skewed `{A,{B,{C,D}}}` | 65 B | 97 B | 129 B | 129 B |
| balanced `{{A,B},{C,D}}` | 97 B | 97 B | 97 B | 97 B |

Right-skewed wins iff **P(A) > P(C) + P(D)**. Here A is the tier-1 normal path
while C and D sit behind `older(65535)` and `older(4255898)` — deep recovery — so
right-skewed is right on cost too. The `wsh` form already encodes the same
ordering, since `or_i` selector bytes likewise make earlier branches cheaper, so
both script types agree. **Balanced would be the correct choice for a wallet
whose branches are equally likely**, and such a wallet cannot currently be
compiled via `--from-policy --context tap` at all.

A balanced variant remains worth building as a deliberately defect-seeking
fixture. It does not exist yet.

## 3. Round-trip results

Method: `md encode --group-size 0 --force-chunked --path bip84 <T>` (mirroring
`design/journeys/transcript_pathological.sh:34`), then `md decode <chunks>`, then `md address`.

| | wsh | tr |
| --- | --- | --- |
| template length | 441 chars | **501 chars** |
| payload | 182 data symbols | **183** |
| chunks | 3 | 3 |
| **STRUCTURAL** `decode(encode(T)) == T` | **PASS** | **PASS** |
| **FUNCTIONAL** addresses, chain 0 and 1 | **FAIL** | **FAIL** |
| `WalletDescriptorTemplateId` | `5b48af35d4321a3ac18b43045e2523cc` | `44ad26a19b53048b6ff8957359a30c31` |
| `WalletPolicyId` | `d3dda0f3a9ef2eef1f1de404b8a352a5` | `b6cdb48c88d6c2be8603d86ab975f6ca` |

The `tr` sibling costs **one extra data symbol** over `wsh` — 183 vs 182 — which
is worth knowing before anyone assumes taproot templates are dramatically more
expensive to engrave. Both fit in 3 chunks.

## 4. Why FUNCTIONAL fails — two distinct findings

### 4.1 R4 reproduced on a real wallet, not inferred from `--help`

```
$ md address --template <T> --key @0=… --chain 0 --count 2
md: --key @0: expected depth 4 for this script context, got 3
```

`md address` has no `--path`, so it cannot be told the origin is bip84 and
infers depth from the script context. This is exactly recon's R4
(`wallet-policy-recon-rust-primary.md`), now with an independent reproduction.

### 4.2 The deeper one — this wallet has never been keyed, by anything, ever

Passing `--path bip84` **explicitly to `md encode`** fails identically, for both
script types:

```
$ md encode --path bip84 --key @0=… <wsh template>   → exit 1, expected depth 4, got 3
$ md encode --path bip84 --key @0=… <tr  template>   → exit 1, expected depth 4, got 3
```

So `--path` does not relax it; the **script context** demands depth 4. The
fixture's keys are bip84 **depth-3** (`m/84'/0'/0'`, per each key file's own
comment, e.g. `[73c5da0a/84'/0'/0']`) — the *single-sig* native-segwit
convention — while a multisig script context expects the bip48 depth-4 form
(`m/48'/coin'/account'/script'`).

And the committed artifact confirms no keying ever happened:

```
$ md inspect $(committed backup-strings)
wallet-policy-mode: false
note: stdout is a keyless descriptor template (no keys)
```

**Consequence.** No address has ever been derived for the pathological wallet by
any tool in the constellation, and none can be with the committed fixture. This
is the structural reason F-210's recon found the journey has no address-verify
step — it could not have had one. Under the round-trip definition
(`DRAFT_round_trip_journey_definition.md` §4), which requires a functional
equality alongside the structural one, **neither wallet can currently complete a
round trip**, and the blocker is fixture material, not codec behaviour.

## 5. Decision this forces — not taken here

The fixture is internally inconsistent: a bip84 single-sig origin declared on a
multisig policy. Options:

1. **Regenerate the 11 xpubs at bip48 depth 4** from the same three committed
   seeds. Cheap and re-derivable. **But it changes the template's origin, hence
   the md1 chunks and both wallet ids** — the committed `backup-strings.txt` and
   everything downstream of it move.
2. **Give `md address`/`md encode` a way to accept a declared origin depth**
   (R4, widened). Leaves the fixture alone; changes normative admission, so it is
   Rust-primary work with vectors.
3. **Accept the wallet as template-only forever** and derive its addresses
   nowhere. Consistent with today, but it means the one journey exercising
   timelocks, a hashlock and unsorted `multi` can never carry a functional
   assertion.

Option 1 is the smallest change that makes the pair round-trippable; option 2 is
the one that decides whether md is being correctly strict or over-strict. **They
are not mutually exclusive and the choice is the user's.**

## 6. Not established

- Whether md's depth-4 rule for multisig contexts is correct per BIP-388/BIP-48,
  or over-strict. Not researched — it is an external-protocol question and this
  run only observed the behaviour. **Confirmed by reading
  `crates/md-cli/src/parse/path.rs:29-31`:** `bip84 => m/84'/0'/0'` (depth 3),
  `bip86 => m/86'/0'/0'` (depth 3), `bip48 => m/48'/0'/0'/2'` (depth 4,
  hardened). So the fixture's keys are single-sig account keys and the depth-4
  demand is the BIP-48 multisig convention — but whether md is *right* to
  enforce it is still unresearched.
- Whether the balanced depth-2 `tr` variant round-trips (not built). Per
  PRIORITY 1 it cannot be produced via `--from-policy`; the template route
  should still reach it.
- Whether a `Descriptor → md AST` converter exists or must be written — the
  crux of PRIORITY 1 fix option (1). Only the forward direction was located.
- Anything about the Go/device side. This run is `md` only, tier T2 at most.
- The caterpillar characterization is **derived by reading the vendored
  formatter and is consistent with six CLI observations**; it has not been
  pinned by a direct per-topology unit test. That test is cheap and worth
  writing before the rule is relied on.
