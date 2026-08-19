# Pathological wallet — matched `wsh` / `tr` pair, and what round-tripping it proved

**Run 2026-08-18 by the controller.** Every value below is pasted command output.
User directive: *"The pathological wallet must roundtrip as wsh and tr wrapped
wallets."*

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

**Note the topology choice, because it is load-bearing.** Right-skewed
`{A,{B,{C,D}}}` mirrors `or_i(A,or_i(B,or_i(C,D)))` exactly. Recon established
that a **balanced depth-2** 4-leaf tree is what triggers the unreleased-#953
`Display` bug, while a right-skewed chain does **not** — the bug is
topology-dependent, not depth-dependent. So this faithful sibling *avoids* the
known defect, and a balanced variant would be a second, deliberately
defect-seeking fixture. Both are worth having; only the faithful one exists.

## 3. Round-trip results

Method: `md encode --group-size 0 --force-chunked --path bip84 <T>` (mirroring
`transcript_pathological.sh:34`), then `md decode <chunks>`, then `md address`.

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
  run only observed the behaviour.
- Whether the balanced depth-2 `tr` variant round-trips (not built).
- Anything about the Go/device side. This run is `md` only, tier T2 at most.
