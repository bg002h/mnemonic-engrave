# miniscript-nesting / LENS: typecheck

**Question:** does the 4-tier degrading vault parse, type-check, and pass
rust-miniscript's own sanity/lint surface? What does the library warn about, and
how close is it to the parse recursion-depth limit?

**Answer, up front: it is clean.** Every lint rust-miniscript has returns the
safe value, `sanity_check()` is `Ok(())`, and the descriptor sits at ~1/80th of
the depth limit and ~1/7th of the standardness script-size cap. The library emits
**zero** warnings on this descriptor. The interesting results are the three
things the library is silent about that are nonetheless real, and one
library-sizing footgun whose magnitude is caused by exactly this nesting shape.

Everything below was **run**, not derived. Probe crate:
`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/22fd28a4-d68a-47d6-82b1-8a8570fb5417/scratchpad/msprobe`
(bins `msprobe`, `probe2`..`probe7`) and `.../scratchpad/ms13` (crates.io control).

## Provenance of the implementation under test

| | |
|---|---|
| `/scratch/code/shibboleth/rust-miniscript-fork` | `git describe` = **`13.0.0-102-g2092faa`** |
| mnemonic-toolkit pins | `[patch.crates-io] miniscript = git rev 95fdd1c` (`mnemonic-toolkit/Cargo.toml:29`, `Cargo.lock:706`) |
| descriptor-mnemonic pins | crates.io `miniscript 13.0.0` (`descriptor-mnemonic/Cargo.lock:529-532`) — **no patch** |
| fork HEAD vs the pinned rev | `git diff --stat 95fdd1c..HEAD` = `src/descriptor/wallet_policy/mod.rs \| 113 +++-` and nothing else |
| mnemonic-engrave | does **not** depend on miniscript at all (no hit in `crates/me-cli/Cargo.toml`) |

So two different miniscripts are in play across the constellation. I ran the full
analysis against **both**. They agree exactly: same address
`bc1q4g7564xxd9hj68hqwu5e558cqafhsklerkr0asfzqp6puq74veesrp6qss`, same 498-byte
witnessScript, same `max_weight_to_satisfy() = 756`, same
`tree_height = 5`, same five lint booleans. **No version skew on this
descriptor.**

## 1. Parse + type-check + lint — measured

Input: `/scratch/code/shibboleth/mnemonic-toolkit/.examples-build/degrade2.desc`
(1868 chars incl. `#4ld0crxa`).

```
Descriptor::<DescriptorPublicKey>::from_str  = Ok
  is_multipath                               = true
  to_string() == input (byte-identical)      = true
Descriptor::sanity_check()                   = Ok(())
into_single_descriptors()                    = 2, both sanity_check = Ok(())
derive_at_index(0).address(Bitcoin)          = bc1q4g7564xxd9hj68hqwu5e558cqafhsklerkr0asfzqp6puq74veesrp6qss
```

Root type of the inner miniscript:
`Correctness { base: B, input: Any, dissatisfiable: false, unit: true }`,
`Malleability { dissat: Unknown, safe: true, non_malleable: true }`.

Every analysis API in `src/miniscript/analyzable.rs`, whole script and per tier:

| API (`analyzable.rs`) | root | tier 1 | tier 2 | tier 3 | tier 4 |
|---|---|---|---|---|---|
| `requires_sig()` :187 | true | true | true | true | true |
| `is_non_malleable()` :190 | true | true | true | true | true |
| `within_resource_limits()` :195 | true | true | true | true | true |
| `has_mixed_timelocks()` :198 | **false** | false | false | false | false |
| `has_repeated_keys()` :201 | false | false | false | false | false |
| `contains_raw_pkh()` :212 | false | false | false | false | false |
| `sanity_check()` :225 | `Ok(())` | `Ok(())` | `Ok(())` | `Ok(())` | `Ok(())` |
| `ext_check(&ExtParams::sane())` :242 | `Ok(())` | `Ok(())` | `Ok(())` | `Ok(())` | `Ok(())` |

`from_str_ext` with `sane()` / `insane()` / `allow_all()` all return `Ok` — i.e.
**no `ExtParams` relaxation is needed**; none of the six insanity allowances is
being consumed. `lift()` succeeds; `normalized()` produces a clean
`or(and(...), and(...), and(...), and(...))`; `n_keys() = 11`;
`minimum_n_keys() = Some(1)`.

## 2. Recursion depth — name, value, file:line, and distance

**Constant:** `const MAX_RECURSION_DEPTH: u32 = 402;` —
`rust-miniscript-fork/src/lib.rs:503`, with the comment at `:502`
(`// https://github.com/sipa/miniscript/pull/5 for discussion on this number`).
Upstream, not fork-modified: `git log -S "MAX_RECURSION_DEPTH: u32"` → commit
`6438f10` (sanket1729, 2020-06-18, "Improve Miniscript robustness").

There are **two independent** checks against it:

1. **String-level paren/brace nesting**, before any AST is built —
   `expression::Tree::parse_pre_check`, `src/expression/mod.rs:592-597`, error
   `ParseTreeError::MaxRecursionDepthExceeded { actual, maximum }`.
2. **AST height** — `Miniscript::from_ast`, `src/miniscript/mod.rs:333-335`,
   error `Error::MaxRecursiveDepthExceeded`, on `ExtData::tree_height`
   (`src/miniscript/types/extra_props.rs:161`).

Measured on this descriptor:

| | value | limit | headroom |
|---|---|---|---|
| max paren/brace nesting of the string | **6** | 402 | 396 |
| `ms.ext.tree_height` | **5** | 402 | 397 (≈80×) |

Per-node `tree_height` multiset (all 25 AST nodes):
`[0×10, 1×6, 2×4, 3×3, 4, 5]`.

**The depth limit is unreachable for any Segwitv0 `wsh`** — something else always
bites first. Measured with synthetic `or_i` chains of `pk()`:

```
or_i depth   93  -> Ok
or_i depth   94  -> Err: The Miniscript corresponding Script cannot be larger than 3600 bytes, but got 3607 bytes.
or_i depth  401  -> Err: (same 3600-byte error)
or_i depth  402  -> Err: maximum recursion depth exceeded (max 402, got 403)   <- string pre-check
```

i.e. the 3600-byte standardness cap fires at ~1/4 of the depth cap even for the
cheapest possible fragment. Distance from the depth limit for the real
descriptor is therefore not a meaningful risk at any scale this wallet could grow to.

## 3. Resource limits — measured against the constants the fork itself cites

All constants from `rust-miniscript-fork/src/miniscript/limits.rs`, each carrying
its Bitcoin Core permalink in the source.

| limit | file:line | value | this descriptor | class |
|---|---|---|---|---|
| `MAX_SCRIPT_SIZE` | limits.rs:15 (Core `script.h:32`) | 10000 | **498** (5.0%) | consensus |
| `MAX_STANDARD_P2WSH_SCRIPT_SIZE` | limits.rs:18 (Core `policy.h:44`) | 3600 | **498** (13.8%) | standardness |
| `MAX_OPS_PER_SCRIPT` | limits.rs:9 (Core `script.h:26`) | 201 | **32** (15.9%) | consensus |
| `MAX_STANDARD_P2WSH_STACK_ITEMS` | limits.rs:12 (Core `policy.h:40`) | 100 | **7** (7.0%) | standardness |
| `MAX_PUBKEYS_PER_MULTISIG` | limits.rs:35 (Core `script.h:30`) | 20 | **3** | consensus |
| `MAX_SCRIPT_ELEMENT_SIZE` | limits.rs:22 (Core `script.h:23`) | 520 | **N/A** | see below |

**The 520-byte limit does not apply.** The `pk_cost > MAX_SCRIPT_ELEMENT_SIZE`
check at `src/miniscript/context.rs:401` sits inside `impl ScriptContext for
Legacy` (block opens at `context.rs:360`) and yields
`MaxRedeemScriptSizeExceeded` — it is the P2SH redeemScript push limit. The
`Segwitv0` impl uses `MAX_SCRIPT_SIZE` (`context.rs:507`) and
`MAX_STANDARD_P2WSH_SCRIPT_SIZE` (`context.rs:538`) instead. A 498-byte
witnessScript is 22 bytes under 520 — that near-miss is a coincidence, not a
constraint.

**Op count, independently recomputed off the serialized bytes** (not read from
the library): walking `explicit_script().instructions()` and counting only
opcodes with `opcode > OP_16` (Bitcoin Core's `EvalScript` increments `nOpCount`
for every such opcode *whether or not the branch executes*):

```
opcodes with opcode > OP_16       = 29
largest CHECKMULTISIG key count   = 3    (+n only for the branch that executes)
worst-case nOpCount               = 32   of 201
```

rust-miniscript's own numbers: `ext.static_ops = 29`,
`sat_op_count = static_ops + sat_data.max_exec_op_count = 29 + 3 = 32`. Exact
agreement with the independent count. Full opcode histogram: 4×`OP_CHECKMULTISIG`,
4×`OP_EQUALVERIFY`, 4×`OP_VERIFY`, 3×`OP_IF`, 3×`OP_ELSE`, 3×`OP_ENDIF`,
2×`OP_CLTV`, 2×`OP_CSV`, 2×`OP_SHA256`, 2×`OP_SIZE`, plus 11×33-byte key pushes,
2×32-byte hash pushes, and the number pushes.

Satisfaction: `max_satisfaction_witness_elements() = 7`
(doc at `miniscript/mod.rs:421-423`: "including the witness script itself"),
`max_satisfaction_size() = 255`, `Descriptor::max_weight_to_satisfy() = 756 WU`.
Verified arithmetic against `descriptor/segwitv0.rs:70-84`:
`(VarInt(7).size() − VarInt(0).size()) + VarInt(498).size() + 498 + 255`
= `0 + 3 + 498 + 255` = **756** ✓ (VarInt sizes measured, not assumed).

**Growth headroom, measured:** appending further
`or_i(…, and_v(v:older(n),multi(2,k,k)))` tiers to the real inner miniscript,
**39** extra tiers still parse (script = 3579 bytes); the 40th fails at 3658
bytes. The 4-tier vault is nowhere near any wall.

**Where the 3600 cap is enforced matters:** it is checked in
`Miniscript::from_ast` → `Ctx::check_global_validity` →
`check_global_policy_validity` (`context.rs:538-545`), i.e. **at construction
time**. rust-miniscript will not even build a `Segwitv0` miniscript whose script
exceeds 3600 bytes; you get a *parse error*, not a lint. Classify as
**TOOLING refusal at the standardness bound**, sitting far below the 10000-byte
consensus bound.

## 4. The timelock lint: why it does NOT fire, and why that is correct

The descriptor uses all four timelock kinds. Decoded via rust-bitcoin's own
types (measured, not asserted):

| tier | fragment | decoded | duration |
|---|---|---|---|
| 1 | `after(1000000)` | `1000000 blocks` — `is_block_height=true` | absolute HEIGHT |
| 2 | `after(1893456000)` | `1893456000 seconds` — `is_block_time=true` | absolute TIME (`LOCK_TIME_THRESHOLD = 500000000`) |
| 3 | `older(65535)` | `Blocks(Height(65535))` | ≈ **455.1 days** at 10 min/block |
| 4 | `older(4255898)` | `Time(Time(61594))` — BIP-68 flag set | 61594 × 512 s = 31 536 128 s = **365.00 days** |

Root `timelock_info`:
`{ csv_with_height: true, csv_with_time: true, cltv_with_height: true, cltv_with_time: true, contains_combination: false }`.

All four flags are set, yet `has_mixed_timelocks()` is **false**, because
`contains_unspendable_path()` returns `contains_combination` alone
(`extra_props.rs:42`), and `contains_combination` is only ever set inside
`combine_threshold` when `k > 1` (`extra_props.rs:71-77`). `or_i` combines via
`combine_or` → `combine_threshold(1, …)` (`extra_props.rs:50-51`), so mixing
*across* `or_i` branches can never set it. This is right: a given input takes
exactly one branch.

**Control (run, both directions):**

```
and_v(v:after(1000000),and_v(v:after(1893456000),pk(...)))
   has_mixed_timelocks = true,  sanity_check = Err(HeightTimelockCombination)
or_i(and_v(v:after(1000000),pk(..)),and_v(v:after(1893456000),pk(..)))
   has_mixed_timelocks = false, sanity_check = Ok(())
```

The lint is live and would fire on the mixed shape. It genuinely does not apply
to this descriptor. **Clean lens result — no finding.**

## 5. What the library is silent about (three real things)

### 5a. Cross-input nLockTime unit conflict, tier 1 vs tier 2 — CONSENSUS

`nLockTime` is a *transaction* field, one per transaction. Tier 1 needs it
interpreted as a **height**, tier 2 as a **time**. rust-miniscript encodes the
consensus rule itself in `Interpreter`: `evaluate_after`,
`src/interpreter/stack.rs:212-216`, returns
`Error::AbsoluteLockTimeComparisonInvalid(n, lock_time)` whenever the CLTV
operand and `nLockTime` are on opposite sides of the threshold
(rust-bitcoin's `LockTime::is_implied_by`, `bitcoin-0.32.8/src/blockdata/locktime/absolute.rs:247-255`,
`_ => false // Not the same units`).

Consequence: **a single transaction can never spend one UTXO of this wallet via
tier 1 and another via tier 2.** One of the two inputs fails script verification —
a consensus failure, not a relay one. The wallet must batch tier-1 and tier-2
spends into separate transactions.

Tier 3 vs tier 4 do **not** have this problem: `nSequence` is per-input, so
mixing relative-blocks and relative-time inputs in one transaction is fine.

No lint models this, and none can — `contains_combination` is a per-script,
per-branch property, and this is a per-*transaction* property. It belongs in the
PSBT/coin-selection layer.

### 5b. The sha256 hashlock is shared between tier 1 and tier 2 — funds-safety

Measured: 2 `Terminal::Sha256` nodes in the AST, **1 distinct target**
(`a84dce40…9b9a08ad`, 2 occurrences in the descriptor string).

`has_repeated_keys()` returns **false** because it is implemented over
`iter_pk()` (`analyzable.rs:201-210`, `iter_pk()` at :204), which yields `Pk`; hash terminals hold
`Pk::Sha256`, a different associated type. There is no `has_repeated_hashlocks`
and no `AnalysisError` variant for it (`analyzable.rs:133-146` enumerates all
six).

Consequence: the first tier-1 spend publishes the preimage on-chain, which
permanently converts tier 2 from "2-of-3 **and** preimage" into plain "2-of-3"
for every remaining UTXO of the wallet. That is a deliberate design choice or a
mistake — the library will never tell you which. **Not consensus, not
standardness; a design property invisible to every lint.**

### 5c. Unlock order is not tier order

`minimum_n_keys()` = **`Some(1)`**. The *weakest* branch (tier 4, 1-of-3) is
gated on a **relative** 365.00 days from each UTXO's confirmation, while tier 3
(2-of-2) needs 65535 blocks ≈ 455 days relative, and tiers 1 and 2 are gated on
**absolute** points (height 1 000 000; unix time 1 893 456 000). So for any UTXO
confirmed today, the 1-of-3 branch is the first to open. Additionally, tier 1 is
a height and tier 2 is a time, so their relative order is not fixed at design
time — it depends on future hashrate.

I did not resolve the current chain tip, so I am not asserting *when* tier 1
opens; the point is only that the naming order and the unlock order are
independent, and rust-miniscript computes `minimum_n_keys` without commenting on
it. A semantics lens should own this; recording it here because it is a direct
output of the library's own `lift()`/policy analysis.

## 6. Two library-behaviour findings for the tooling side

### 6a. `Plan::satisfaction_weight()` under-reports a `wsh` spend by the whole witnessScript — TOOLING

Measured:

| | template items | `witness_size()` | `satisfaction_weight()` |
|---|---|---|---|
| tier 1 plan | 6 | 256 | **260** |
| tier 2 plan | 6 | 184 | **188** |
| tier 3 plan | 6 | 152 | **156** |
| tier 4 plan | 5 | 78 | **82** |
| `Descriptor::max_weight_to_satisfy()` | — | — | **756** |

`Plan::witness_size` (`src/plan.rs:258-264`) sums only `self.template` via
`util::witness_size` (`src/util.rs:50-52`). The witnessScript is pushed onto the
stack **only inside `Plan::satisfy()`** (`src/plan.rs:294-301`), never as a
template placeholder — which is why the templates have 6 items where
`max_satisfaction_witness_elements()` says 7. The omitted term is
498 (script) + `VarInt(498).size()` = 3, i.e. **501 WU**.

So the Plan API under-states the true input weight by **66%** for a tier-1 spend
and **86%** for a tier-4 spend. This is generic to `wsh`, but the *magnitude* is
caused by exactly the shape under review: a 4-deep `or_i` nest puts all four
unused branches into the witnessScript, so the omitted term dominates the
reported one. Any fee estimator built on `Plan::satisfaction_weight()` would
badly underpay on this wallet.

Not currently exposed in the constellation: `mnemonic-toolkit`'s gate documents
that the per-branch `plan()` step "was CUT"
(`crates/mnemonic-toolkit/src/descriptor_builder/gate.rs:13-17`), and
`mnemonic-engrave` does not depend on miniscript. The rule to carry forward is:
**size spends with `Descriptor::max_weight_to_satisfy()`, never with
`Plan::satisfaction_weight()`.**

### 6b. `Descriptor::from_str` does not run the sanity lints — TOOLING

Control, run: `Descriptor::<DescriptorPublicKey>::from_str("wsh(and_v(v:after(1000000),after(1000001)))")`
returns **`Ok`**, and only the explicit call surfaces it —
`.sanity_check() = Err(AnalysisError(SiglessBranch))`.

`Miniscript::from_str` *does* apply `ExtParams::sane()`
(`src/miniscript/mod.rs:1069-1070`), but the descriptor path
(`Wsh::from_tree` → `Miniscript::from_tree`, `src/descriptor/segwitv0.rs:195`)
does not. `mnemonic-toolkit` already knows this and calls sanity as a separate
gate step, documenting it as "F1: `from_str` is LENIENT on the funds-footgun
rules" (`descriptor_builder/gate.rs:7-8`, step 3 at `:176`). Recorded so any
*new* consumer in the constellation does not assume parsing implies safety.

## 7. BIP-388 template form (`@0..@10`)

The abstract form in the brief is a BIP-388 wallet policy, not a descriptor.
Measured:

```
Descriptor::<DescriptorPublicKey>::from_str(template)  -> Err: key too short
WalletPolicy::from_str(template)                       -> Ok
WalletPolicy::from_descriptor(&concrete)               -> Ok
  template Display: wsh(or_i(and_v(v:after(1000000),... multi(3,@0/**,@1/**,@2/**)) ...))
  into_descriptor() round-trips byte-identical to degrade2.desc  = true
```

`WalletPolicy` is re-exported at `src/descriptor/mod.rs:62`. Its
`validate()` (`src/descriptor/wallet_policy/mod.rs:173-189`) enforces in-order,
non-skipped key placeholders and disjoint multipath expressions — this
descriptor satisfies both. The `/<0;1>/*` suffixes canonicalize to `/**`.
`WalletPolicy::from_str` on the bare template succeeds; `into_descriptor()` on it
errors with `KeyInfoInvalidKeyIndex(8)` only because no key info is attached,
which is expected.

Note that this round-trip exercises the **fork-only** code path — the 110-line
`wallet_policy/mod.rs` delta at fork HEAD is the sole difference from
mnemonic-toolkit's pinned rev, and hash terminals in the wallet-policy translator
are the very thing fork HEAD `2092faa` adds. crates.io 13.0.0 has no
`WalletPolicy` at all.

## 8. Clean — risks checked that do NOT apply

Each of these was evaluated and returned the safe value; none is a finding.

- `AnalysisError::SiglessBranch` — `requires_sig()` true on every tier.
- `AnalysisError::Malleable` — `is_non_malleable()` true; `Malleability { safe: true, non_malleable: true }`.
- `AnalysisError::BranchExceedResouceLimits` — `within_resource_limits()` true.
- `AnalysisError::HeightTimelockCombination` — false; mixing is across `or_i` only (§4, with a positive control).
- `AnalysisError::RepeatedPubkeys` — 11 distinct xpubs, no repeats.
- `AnalysisError::ContainsRawPkh` — no raw-pkh fragments.
- `MAX_RECURSION_DEPTH` 402, both the string pre-check and the AST-height check — 6 and 5 respectively.
- `MAX_OPS_PER_SCRIPT` 201 — 32, independently recounted off the serialized script.
- `MAX_SCRIPT_SIZE` 10000 / `MAX_STANDARD_P2WSH_SCRIPT_SIZE` 3600 — 498.
- `MAX_STANDARD_P2WSH_STACK_ITEMS` 100 — 7.
- `MAX_SCRIPT_ELEMENT_SIZE` 520 — Legacy-only (`context.rs:401` inside `impl ScriptContext for Legacy`); does not bind a native `wsh` witnessScript.
- `MAX_PUBKEYS_PER_MULTISIG` 20 — largest `multi` is 3.
- `ScriptContextError::MultiANotAllowed` / x-only keys in `Segwitv0` — no tapscript fragments; all keys are 33-byte compressed.
- BIP-389 multipath length mismatch (`Error::MultipathDescLenMismatch`) — all 11 keys use `<0;1>`, `into_single_descriptors()` yields exactly 2.
- BIP-380 checksum — `to_string()` reproduces `#4ld0crxa` byte-identically.
- Hash-preimage size malleability — the `sha256` fragment compiles with its `OP_SIZE <0x20> OP_EQUALVERIFY` guard (2×`OP_SIZE` in the histogram), so a non-32-byte preimage cannot be substituted.
- Version skew between crates.io `13.0.0` (descriptor-mnemonic) and the pinned fork rev (mnemonic-toolkit) — ran both; identical on every measured value.

**Caveat on one constant I could not resolve:** Bitcoin Core's
`MAX_STANDARD_P2WSH_STACK_ITEM_SIZE` (the per-witness-item standardness bound)
does **not exist anywhere in rust-miniscript** — `grep -rn STACK_ITEM_SIZE src/`
returns 0 hits, and it is not in bitcoin-0.32.8 either. So the library models no
per-item size limit at all. It is not triggered here regardless: the largest
non-script witness element in any tier's satisfaction is a 73-byte ECDSA
signature, and the 32-byte preimage and 1-byte `or_i` selectors are far smaller.
I am flagging the *library gap*, not asserting the Core value, because I could
not resolve it against source in these repos.

## Verdict

`wsh(...)` degrade2 **parses, type-checks, lifts, plans, and passes every
rust-miniscript lint with no `ExtParams` relaxation**. Depth is 5 against a limit
of 402, and that limit is structurally unreachable for `Segwitv0` because the
3600-byte standardness cap is enforced at construction time and bites ~4× sooner.
Script size, op count and witness-element count all sit under 16% of their caps,
with room for ~39 more tiers.

Nothing blocks. Three things the lint surface cannot see are worth carrying to
other lenses: the tier-1/tier-2 `nLockTime` unit conflict across inputs
(consensus, wallet-layer), the shared sha256 hashlock (funds-safety, design), and
`Plan::satisfaction_weight()`'s 501-WU omission (tooling, fee estimation).
