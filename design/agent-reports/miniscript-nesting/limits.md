# Lens: consensus & standardness SIZE / RESOURCE limits — the `degrade2` pathological vault

Agent: `miniscript-nesting/limits`. Date: 2026-08-11.

Subject: `/scratch/code/shibboleth/mnemonic-toolkit/.examples-build/degrade2.desc`
(`wsh(...)`, 4-tier degrading vault, `or_i` three deep, 11 xpubs, checksum `#4ld0crxa`).

---

## Verdict

**On the size/resource lens this descriptor is CLEAN.** No consensus limit, no
standardness limit and no rust-miniscript refusal is anywhere near binding. The
tightest *ratio* against any cap is 90% — and that one (72-byte ECDSA signature vs
the 80-byte per-item standardness cap) is a property of ECDSA, identical for every
P2WSH multisig on the network, and is not reachable by anything this descriptor
does.

Everything below is measured, not derived. Two real defects were found, but both
are **TOOLING**, in the repo's own cost estimator — not in the descriptor.

---

## How the numbers were obtained

A scratch crate built against the on-disk fork (`miniscript` v13.0.0, path dep on
`/scratch/code/shibboleth/rust-miniscript-fork`, HEAD `2092faa`). Harness source is
persisted next to this report:

- `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/miniscript-nesting/limits-harness.rs`
- `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/miniscript-nesting/limits-harness-Cargo.toml`

Per-path witnesses are **real serialized witness stacks** produced by
`Descriptor::get_satisfaction()` with a worst-case *standard* (low-S, BIP-146) ECDSA
signature — 33-byte DER `r`, 32-byte DER `s`, 72 bytes on the wire. Each stack was
then fed back through `miniscript::interpreter::Interpreter::iter_assume_sigs()` to
confirm the branch actually executes (see "run-check" below). Static opcodes were
counted by walking the encoded script instruction-by-instruction with Bitcoin Core's
own rule (`opcode > OP_16` counts, executed or not), not read off a struct field —
and the walk agrees with `ms.ext.static_ops` exactly (29 == 29).

---

## The measured numbers

### Script (identical for both multipath branches and every derivation index)

| quantity | value | source |
| --- | --- | --- |
| descriptor string | 1868 chars | measured |
| scriptPubKey | `0020aa3d…6673` (P2WSH) | measured |
| **witnessScript** | **498 bytes** | `ms.script_size()` == `ms.ext.pk_cost` == real `explicit_script().len()` |
| script instructions | 56 | measured walk |
| **static opcodes (> OP_16)** | **29** | measured walk == `ms.ext.static_ops` |
| miniscript tree height | 5 | `ms.ext.tree_height` |
| accurate sigops (BIP-141) | **11** | `Script::count_sigops()` |
| legacy sigops (inaccurate) | 80 | `Script::count_sigops_legacy()` — not the rule that applies to P2WSH |

Verified index/branch invariance: `498 bytes / 29 ops` at branch 0 and 1, indices
0, 1, 1000, 2147483647. All 33-byte compressed keys, so the script cannot grow.

### Per spending path

`multi()` nodes in pre-order confirm the tier mapping: T1 `multi(3,n=3)`,
T2 `multi(2,n=3)`, T3 `multi(2,n=2)`, T4 `multi(1,n=3)`.

| tier | witness items (excl. script) | item sizes | max item | witness bytes (incl. script + varints) | executed op count |
| --- | --- | --- | --- | --- | --- |
| T1 3-of-3 + sha256, after HEIGHT 1000000 | 6 | `[0, 72, 72, 72, 32, 1]` | 72 | **757** | **32** (29 static + 3 multisig keys) |
| T2 2-of-3 + sha256, after TIME 1893456000 | 6 | `[0, 72, 72, 32, 1, 0]` | 72 | 685 | 32 |
| T3 2-of-2, older BLOCKS 65535 | 6 | `[0, 72, 72, 1, 0, 0]` | 72 | 653 | 31 (29 + 2) |
| T4 1-of-3, older TIME 4255898 | 5 | `[0, 72, 0, 0, 0]` | 72 | 579 | 32 |

Op count per path = the 29 static opcodes (Core counts *every* opcode in the script,
executed or not — `interpreter.cpp`: `if (opcode > OP_16 && ++nOpCount > MAX_OPS_PER_SCRIPT)`
sits outside the `fExec` guard) **plus** the executed `CHECKMULTISIG`'s key count
(`nOpCount += nKeysCount`, which *is* inside the `fExec` guard). Cross-checks
against the library: `ms.ext.static_ops + sat_data.max_exec_op_count = 29 + 3 = 32`,
matching the measured worst path.

Library aggregates (worst case over all paths), all agreeing with the measurement:

```
ms.max_satisfaction_witness_elements() = 7     (T1 measured: 7, incl. witnessScript)
ms.max_satisfaction_size()             = 255   (T1 measured: 255 bytes of non-script items)
d0.max_weight_to_satisfy()             = 756 WU (= 3 varint + 498 script + 255 items)
ms.sanity_check()                      = Ok
Descriptor::sanity_check()             = Ok
```

### Headroom against every applicable limit

| limit | class | cap | this descriptor | used | source |
| --- | --- | --- | --- | --- | --- |
| `MAX_SCRIPT_SIZE` | **consensus** | 10 000 | 498 | 5.0% | `src/miniscript/limits.rs:15` |
| `MAX_STANDARD_P2WSH_SCRIPT_SIZE` | **standardness** | 3 600 | 498 | **13.8%** | `src/miniscript/limits.rs:18`; Core `policy.h` `{3600}` |
| `MAX_OPS_PER_SCRIPT` | **consensus** | 201 | 32 (T1/T2/T4) | **15.9%** | `src/miniscript/limits.rs:9` |
| `MAX_STANDARD_P2WSH_STACK_ITEMS` | **standardness** | 100 | 6 (excl. script) | 6% | `src/miniscript/limits.rs:12`; Core `policy.h` `{100}` |
| `MAX_STANDARD_P2WSH_STACK_ITEM_SIZE` | **standardness** | 80 | 72 | **90%** | Core `policy.h` `{80}` — **not modelled in the fork** |
| `MAX_SCRIPT_ELEMENT_SIZE` (witness items) | **consensus** | 520 | 72 | 13.8% | `src/miniscript/limits.rs:22` |
| `MAX_STACK_SIZE` (stack + altstack) | **consensus** | 1 000 | ≤ 9 (6 initial + `max_exec_stack_count` 3) | 0.9% | `src/miniscript/limits.rs:29` |
| `MAX_PUBKEYS_PER_MULTISIG` | **consensus** | 20 | 3 | 15% | `src/miniscript/limits.rs:35` |
| `MAX_STANDARD_TX_SIGOPS_COST` | **standardness** | 16 000 (= `MAX_BLOCK_SIGOPS_COST`/5) | 11 per input → 1454 inputs | — | Core `policy.h`, `consensus.h` `{80000}` |
| `MAX_STANDARD_TX_WEIGHT` | **standardness** | 400 000 | 921 WU per input (164 base + 757 witness) → 434 inputs | — | Core `policy.h` `{400000}` |
| `DEFAULT_BYTES_PER_SIGOP` vsize bump | **policy/fee** | `max(weight, sigops·20)` | 11·20 = 220 vs 921 WU — never binds | — | Core `policy.h` `{20}` |

Core constants for the last five rows were resolved against
`bitcoin/bitcoin` v28.0 `src/policy/policy.h` and `src/consensus/consensus.h`
(fetched verbatim), because the fork does not carry them. The first six rows were
read out of the fork's own `limits.rs`, which cites the same Core headers by commit.

### Run-check: do these witnesses actually execute?

`Interpreter::iter_assume_sigs()` on the real script:

- **T3** — 3 conditions satisfied, no error.
- **T4** — 2 conditions satisfied, no error.
- **T1 / T2** — fail at `Expected Satisfied Boolean at stack top for VERIFY`, i.e.
  the `sha256` VERIFY, because the preimage for `a84dce40…08ad` is not recorded
  anywhere in the repos (`gen_spec2.py:10` keeps only the hash). To close this,
  the same descriptor was rebuilt with `sha256(SHA256([0xab;32]))` substituted —
  a **498-byte** witnessScript, byte-identical in length — and both branches then
  execute clean: **T1 5 conditions, T2 4 conditions, no error.** Branch selection,
  IF nesting, dummy-element placement and multisig arity are therefore all
  confirmed by execution, not just by construction.

Witness item bytes were also inspected for `SCRIPT_VERIFY_MINIMALIF` (a *policy*
flag for segwit v0): the only `OP_IF` condition items produced are the empty vector
and the single byte `01`. Compliant.

### Scaling probe (how much room is actually left)

To find which cap binds *first* for this family of shapes, synthetic
`or_i`-chained vaults of `and_v(v:older(N),multi(2,pk,pk))` tiers were parsed until
rust-miniscript refused:

```
tiers=30: script=2339  ops=179  wit_items=33  sanity=Ok
tiers=34: script=2655  ops=203  wit_items=37  sanity=Err("At least one spend path
          exceeds the resource limits(stack depth/satisfaction size..)")
```

`MAX_OPS_PER_SCRIPT` (201) binds first, at ~34 tiers, with the standardness script
budget still only 74% used and the stack-item budget at 37%. **This is a probe on a
related shape, not on `degrade2` itself** — but it establishes the ordering of the
caps and shows the real descriptor sits at roughly one-eighth of the ceiling.

---

## Findings

### F1 — TOOLING, **important**: `mnemonic compare-cost` omits the witnessScript from its wsh column, while the tr column includes its tapleaf script

Not a property of the descriptor — a defect in the repo's own estimator, which this
descriptor makes large enough to be unmissable.

`crates/mnemonic-toolkit/src/cost/enumerate.rs:267` reads
`Ok(plan) => Some(plan.witness_size())`. For a **wsh** descriptor,
`Plan::witness_size()` (`rust-miniscript-fork/src/plan.rs:258-266`) sums only the
placeholder template, and `Wsh::plan_satisfaction`
(`src/descriptor/segwitv0.rs:164-169`) is just `self.ms.build_template(provider)` —
the witnessScript is *not* in the template; `Plan::satisfy` pushes it separately
afterwards (`src/plan.rs:294-301`). For **tr**, `best_tap_spend`
(`src/descriptor/tr/mod.rs:500-501`) explicitly does
`wit.push(Placeholder::TapScript(script)); wit.push(Placeholder::TapControlBlock(control_block));`
— so the leaf script *is* counted.

Measured on this descriptor (my independent reconstruction reproduces the shipped
tool's output byte-for-byte in both columns):

| tier | tool's wsh vB | true wsh vB | tr vB | tool's Δ | true Δ |
| --- | --- | --- | --- | --- | --- |
| T1 | 105 | **231** | 232 | +127 | **+1** |
| T2 | 87 | **213** | 216 | +129 | **+3** |
| T3 | 79 | **205** | 208 | +129 | **+3** |
| T4 | 61 | **186** | 192 | +131 | **+6** |

`Plan::witness_size()` returns 256/184/152/78 for T1..T4; the real witnesses are
757/685/653/579 bytes. The gap is exactly **501 bytes = 498-byte witnessScript +
3-byte varint**, every row. Converting through the tool's own
`witness_bytes_to_vbytes` (`cost/format.rs:55`, `(164 + w + 3)/4`) reproduces
105/87/79/61 exactly, confirming the diagnosis rather than guessing at it. On the tr
side I rebuilt `tr(NUMS, M')` with `multi`→`multi_a` and measured
762/699/666/602 bytes → 232/216/208/192 vB, matching the tool exactly, with
`TapScript=true TapControlBlock=true` in every template.

Two aggravating factors:

1. `design/SPEC_compare_cost_v0_26_0.md:213` states the opposite in a comment:
   `let wsh_witness_bytes = wsh_plan.witness_size();  // includes scriptCode (the witnessScript)`.
   The claim is false; the implementation faithfully implements the false claim.
2. `crates/mnemonic-toolkit/src/cost/mod.rs:173` prints
   `"…absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct"`.
   The absolute numbers are off by 125–126 vB, not ±1, and the Δ is off by the same
   amount — ~98% of the reported wsh-vs-tr gap is an artifact. The stated conclusion
   ("wsh saves 127 vB per spend") inverts to "wsh and tr are within 1–6 vB" for this
   policy, which is a materially different wallet-design answer.

Direction of error is *understatement of wsh cost*, i.e. fee-optimistic.

### F2 — TOOLING/OPERATIONAL, **minor**: `compare-cost` refuses this descriptor by default, and takes 3¼ minutes when forced

```
$ mnemonic compare-cost --descriptor "$(cat .examples-build/degrade2.desc)"
error: compare-cost: spending conditions exceed --max-conditions cap (73728 > 4096);
       raise the cap or simplify the policy
```

73 728 = `n_abs × n_rel × 2^(11 signers + 1 preimage)`. With
`--max-conditions 100000` it completes but burns **195 s of CPU (3:15 wall)** to emit
12 rows. Not wrong, but this is exactly the descriptor the tool exists for, and it
is off by 18× from the default cap. Worth a follow-up on either the enumeration
strategy or the default.

### F3 — TOOLING/INFORMATIONAL, **note**: the fork does not model two Core limits

- `MAX_STANDARD_P2WSH_STACK_ITEM_SIZE` (80) has **no constant at all** in
  `src/miniscript/limits.rs` and is never checked. Harmless in practice: the largest
  item miniscript can put on a segwit-v0 witness stack is a 73-byte ECDSA signature
  (`src/util.rs:24` — `EcdsaSigPk => 73`, incl. the length prefix), so no
  miniscript-constructible P2WSH witness can breach 80. Measured here: 72.
- `MAX_STACK_SIZE` (1000) is checked **only in the Tap context**
  (`src/miniscript/context.rs:651`); `Segwitv0::check_local_consensus_validity`
  (`context.rs:520-533`) checks op count only. Also harmless here (≤ 9 elements),
  but it is a gap in the fork's model, not a deliberate exemption I could find
  documented.

### F4 — INFORMATIONAL, **note**: the fork's 100-item check is off by one in the safe direction

`Segwitv0::check_local_policy_validity` (`context.rs:547-564`) compares
`max_satisfaction_witness_elements()` — which **includes** the witnessScript
(`src/miniscript/mod.rs:428-433`, `… + 1`) — against `MAX_STANDARD_P2WSH_STACK_ITEMS`.
Core's `IsWitnessStandard` pops the script first and checks
`stack.size() - 1 > 100`. The fork is therefore conservative by exactly one item.
No action; recorded so nobody "fixes" it in the unsafe direction.

### F5 — INFORMATIONAL, **note**: `max_satisfaction_size()` is the same trap as F1

`ms.max_satisfaction_size()` returns **255** — witness items only, no witnessScript.
`Descriptor::max_weight_to_satisfy()` returns **756 WU** and *does* add
`varint_len(498) + 498`. Any caller that fee-estimates off `max_satisfaction_size()`
directly will undercount by 501 bytes per input on this descriptor. Prefer
`max_weight_to_satisfy()`.

### F6 — INFORMATIONAL, **note**: 498 bytes is 22 short of `MAX_SCRIPT_ELEMENT_SIZE`

Irrelevant for this descriptor: in `VerifyWitnessProgram` Core pops the witnessScript
off the stack *before* applying the 520-byte per-item check, so the script is bounded
by 10 000 (consensus) / 3 600 (standardness), not 520. Recorded only because it is a
near-miss that would become load-bearing if the wrapper ever changed to a bare
`sh(M)` — one more key in any tier (+34 bytes) would cross it. `sh(wsh(M))` is
unaffected (its redeemScript is 34 bytes).

---

## Explicitly clean — risks checked that do NOT apply

- **witnessScript vs the 3 600-byte P2WSH standardness cap** — 498 bytes, 13.8%. Not close.
- **witnessScript vs the 10 000-byte consensus cap** — 5.0%. Not close.
- **`MAX_OPS_PER_SCRIPT`** — worst path 32 of 201. The `or_i` nesting adds static
  opcodes on *every* path (Core counts unexecuted branches), which is the trap here,
  and it is still only 16% consumed.
- **`MAX_STANDARD_P2WSH_STACK_ITEMS`** — 6 items excluding the script (5 for T4), of 100.
- **`MAX_STANDARD_P2WSH_STACK_ITEM_SIZE`** — 72 bytes max, of 80. Tightest ratio in the
  table, but it is the ECDSA signature; nothing about the nesting or the 32-byte
  sha256 preimage moves it.
- **520-byte push limit on witness items** — 72 bytes max.
- **1 000-element stack/altstack limit** — ≤ 9 at peak.
- **`MAX_PUBKEYS_PER_MULTISIG`** — max arity 3 of 20.
- **Sigop cost** — 11 accurate sigops per input; 1 454 inputs before the 16 000
  standardness cap. The `DEFAULT_BYTES_PER_SIGOP` vsize bump (`max(weight, 220)` per
  input vs 921 WU actual) never fires.
- **`MAX_STANDARD_TX_WEIGHT`** — 434 worst-case inputs would fit in one standard tx.
- **`MAX_SCRIPTSIG_SIZE`** — scriptSig is empty (native segwit); measured 0 bytes.
- **`SCRIPT_VERIFY_MINIMALIF`** — only `<empty>` and `01` appear as IF conditions.
- **Derivation-index / multipath growth** — script is 498 bytes and 29 static ops at
  branches 0 and 1, indices 0 / 1 / 1000 / 2147483647. No index-dependent size drift.
- **rust-miniscript admission** — `from_str` (runs `check_global_validity`: consensus
  10 000 + standardness 3 600) and `sanity_check()` (runs `within_resource_limits()`
  → `check_local_validity`: op count 201 + witness items 100) both return `Ok`. No
  library refusal.
- **`me` / `md` side** — grepped `mnemonic-engrave/crates` and `descriptor-mnemonic/src`
  for `3600 | MAX_STANDARD | MAX_OPS | script_size | max_satisfaction | witnessScript`:
  no hits. Neither codec imposes or depends on a script-size bound, so there is
  nothing on the engraving path for this lens to catch.

**Out of scope for this lens, flagged for the owning lens:** the mixed
HEIGHT-locktime (T1) and TIME-locktime (T2) absolute timelocks in one script, and
`older(4255898)` decoding as the BIP-68 time flag with 61 594 × 512 s. The library's
`has_mixed_timelocks()` returns false (they sit under `or_i`, never under an `and`),
so no size consequence — but the semantics belong to the timelock lens.

---

## Appendix — raw harness output

Reproduce with:

```sh
mkdir -p /tmp/limits/src
cp design/agent-reports/miniscript-nesting/limits-harness.rs      /tmp/limits/src/main.rs
cp design/agent-reports/miniscript-nesting/limits-harness-Cargo.toml /tmp/limits/Cargo.toml
cd /tmp/limits && cargo run --release
```

```
== descriptor ==
chars (incl. checksum): 1868
parsed OK; desc_type = Wsh
Descriptor::sanity_check() = Ok
max_weight_to_satisfy() = Ok(Weight(756))
into_single_descriptors() -> 2 descriptors

== witnessScript (receive branch, index 0) ==
scriptPubkey       : 0020aa3d4d54c6696f2d1ee077299a50f80753785bf91d86fec12200741e03d56673
witnessScript bytes: 498
script instructions: 56
static opcodes >OP_16 (measured by walking the script): 29

== rust-miniscript ExtData (Segwitv0) ==
ms.script_size()                       = 498
ms.ext.pk_cost                         = 498
ms.ext.static_ops                      = 29
ms.ext.tree_height                     = 5
sat_data.max_exec_op_count             = 3
=> sat_op_count (static + exec)        = 32
sat_data.max_witness_stack_count       = 6
sat_data.max_witness_stack_size        = 255
sat_data.max_exec_stack_count          = 3
ms.max_satisfaction_witness_elements() = Ok(7)
ms.max_satisfaction_size()             = Ok(255)
d0.max_weight_to_satisfy()             = Ok(Weight(756))
ms.sanity_check() (analyzable)         = Ok(())

== limits (miniscript::miniscript::limits) ==
MAX_OPS_PER_SCRIPT             = 201
MAX_SCRIPT_SIZE                = 10000
MAX_STANDARD_P2WSH_SCRIPT_SIZE = 3600
MAX_STANDARD_P2WSH_STACK_ITEMS = 100
MAX_SCRIPT_ELEMENT_SIZE        = 520
MAX_STACK_SIZE                 = 1000

== multi() nodes in pre-order ==
tier 1: multi(3, n=3)
tier 2: multi(2, n=3)
tier 3: multi(2, n=2)
tier 4: multi(1, n=3)

== per-path measured satisfaction (real witness built by Descriptor::get_satisfaction) ==

-- T1 3-of-3 + sha256, after HEIGHT 1000000
   scriptSig len                : 0
   witness stack items (total)  : 7
   ... excluding witnessScript  : 6
   item sizes (excl. script)    : [0, 72, 72, 72, 32, 1]
   max item size (excl. script) : 72
   last item (witnessScript) len: 498
   total witness bytes (w/ varints, excl. per-input marker): 757
   executed op count            : 32 (= 29 static + 3 multisig keys)

-- T2 2-of-3 + sha256, after TIME 1893456000
   scriptSig len                : 0
   witness stack items (total)  : 7
   ... excluding witnessScript  : 6
   item sizes (excl. script)    : [0, 72, 72, 32, 1, 0]
   max item size (excl. script) : 72
   last item (witnessScript) len: 498
   total witness bytes (w/ varints, excl. per-input marker): 685
   executed op count            : 32 (= 29 static + 3 multisig keys)

-- T3 2-of-2, older BLOCKS 65535
   scriptSig len                : 0
   witness stack items (total)  : 7
   ... excluding witnessScript  : 6
   item sizes (excl. script)    : [0, 72, 72, 1, 0, 0]
   max item size (excl. script) : 72
   last item (witnessScript) len: 498
   total witness bytes (w/ varints, excl. per-input marker): 653
   executed op count            : 31 (= 29 static + 2 multisig keys)

-- T4 1-of-3, older TIME 4255898 (bip68 time flag)
   scriptSig len                : 0
   witness stack items (total)  : 6
   ... excluding witnessScript  : 5
   item sizes (excl. script)    : [0, 72, 0, 0, 0]
   max item size (excl. script) : 72
   last item (witnessScript) len: 498
   total witness bytes (w/ varints, excl. per-input marker): 579
   executed op count            : 32 (= 29 static + 3 multisig keys)

== sigops (rust-bitcoin Script::count_sigops, BIP141 accurate counting) ==
witnessScript.count_sigops()        = 11
witnessScript.count_sigops_legacy() = 80

== script size stability across derivation index / multipath branch ==
branch0 idx0: 498 bytes / 29 static ops   branch0 idx1: 498 bytes / 29 static ops   branch0 idx1000: 498 bytes / 29 static ops   branch0 idx2147483647: 498 bytes / 29 static ops
branch1 idx0: 498 bytes / 29 static ops   branch1 idx1: 498 bytes / 29 static ops   branch1 idx1000: 498 bytes / 29 static ops   branch1 idx2147483647: 498 bytes / 29 static ops

== witness item bytes (MINIMALIF / standardness inspection), T1 ==
  [0] 0 bytes =
  [1] 72 bytes = 3045022100ff0000000000000000000000000000000000000000000000000000000000000102207f0000000000000000000000000000000000000000000000000000000000000101
  [2] 72 bytes = 3045022100ff0000000000000000000000000000000000000000000000000000000000000102207f0000000000000000000000000000000000000000000000000000000000000101
  [3] 72 bytes = 3045022100ff0000000000000000000000000000000000000000000000000000000000000102207f0000000000000000000000000000000000000000000000000000000000000101
  [4] 32 bytes = abababababababababababababababababababababababababababababababab
  [5] 1 bytes = 01
  [6] witnessScript, 498 bytes

== interpreter run-check: does each witness actually satisfy the script? ==
-- T1 3-of-3 + sha256, after HEIGHT 1000000
   satisfied conditions = 1, error = Some("Expected Satisfied Boolean at stack top for VERIFY")
-- T2 2-of-3 + sha256, after TIME 1893456000
   satisfied conditions = 1, error = Some("Expected Satisfied Boolean at stack top for VERIFY")
-- T3 2-of-2, older BLOCKS 65535
   satisfied conditions = 3, error = None
-- T4 1-of-3, older TIME 4255898 (bip68 time flag)
   satisfied conditions = 2, error = None

== interpreter run-check on a hash-substituted twin (proves T1/T2 branch structure) ==
twin witnessScript bytes = 498 (same length as the real one: true)
-- T1 3-of-3 + sha256, after HEIGHT 1000000
   satisfied conditions = 5, error = None
-- T2 2-of-3 + sha256, after TIME 1893456000
   satisfied conditions = 4, error = None

== wsh Plan::witness_size() vs the REAL witness (does it include the witnessScript?) ==
(this is what mnemonic-toolkit `compare-cost` reads: cost/enumerate.rs:267)
-- T1 3-of-3 + sha256, after HEIGHT 1000000
   Plan::witness_size() =  256  -> compare-cost reports 105 vB
   real witness bytes   =  757  -> true cost           231 vB   (delta 501 bytes = witnessScript 498 + varint 3)
-- T2 2-of-3 + sha256, after TIME 1893456000
   Plan::witness_size() =  184  -> compare-cost reports 87 vB
   real witness bytes   =  685  -> true cost           213 vB   (delta 501 bytes = witnessScript 498 + varint 3)
-- T3 2-of-2, older BLOCKS 65535
   Plan::witness_size() =  152  -> compare-cost reports 79 vB
   real witness bytes   =  653  -> true cost           205 vB   (delta 501 bytes = witnessScript 498 + varint 3)
-- T4 1-of-3, older TIME 4255898 (bip68 time flag)
   Plan::witness_size() =   78  -> compare-cost reports 61 vB
   real witness bytes   =  579  -> true cost           186 vB   (delta 501 bytes = witnessScript 498 + varint 3)

== tr(NUMS, single-leaf) twin: does Plan::witness_size() include the tapleaf script? ==
tapleaf script bytes = 494
-- T1 3-of-3 + sha256, after HEIGHT 1000000
   tr witness_size() = 762 -> 232 vB | template has TapScript=true TapControlBlock=true
-- T2 2-of-3 + sha256, after TIME 1893456000
   tr witness_size() = 699 -> 216 vB | template has TapScript=true TapControlBlock=true
-- T3 2-of-2, older BLOCKS 65535
   tr witness_size() = 666 -> 208 vB | template has TapScript=true TapControlBlock=true
-- T4 1-of-3, older TIME 4255898 (bip68 time flag)
   tr witness_size() = 602 -> 192 vB | template has TapScript=true TapControlBlock=true

== scaling probe: how many MORE tiers of this shape fit before a limit binds? ==
  tiers= 5: script=Ok(387) ops=29 wit_items=8 sanity_err=None
  tiers=10: script=Ok(777) ops=59 wit_items=13 sanity_err=None
  tiers=15: script=Ok(1167) ops=89 wit_items=18 sanity_err=None
  tiers=20: script=Ok(1557) ops=119 wit_items=23 sanity_err=None
  tiers=25: script=Ok(1947) ops=149 wit_items=28 sanity_err=None
  tiers=30: script=Ok(2339) ops=179 wit_items=33 sanity_err=None
  tiers=34: script=Ok(2655) ops=203 wit_items=37 sanity_err=Some("At least one spend path exceeds the resource limits(stack depth/satisfaction size..)")

== per-path Plan (rust-miniscript planning module) ==
-- T1 3-of-3 + sha256, after HEIGHT 1000000
   template items = 6, witness_size() = 256, satisfaction_weight() = 260, abs_lock = Some(1000000 blocks), rel_lock = None
-- T2 2-of-3 + sha256, after TIME 1893456000
   template items = 6, witness_size() = 184, satisfaction_weight() = 188, abs_lock = Some(1893456000 seconds), rel_lock = None
-- T3 2-of-2, older BLOCKS 65535
   template items = 6, witness_size() = 152, satisfaction_weight() = 156, abs_lock = None, rel_lock = Some(Blocks(Height(65535)))
-- T4 1-of-3, older TIME 4255898 (bip68 time flag)
   template items = 5, witness_size() = 78, satisfaction_weight() = 82, abs_lock = None, rel_lock = Some(Time(Time(61594)))
```

### `compare-cost` output referenced in F1/F2

```
$ mnemonic compare-cost --descriptor "$(cat .examples-build/degrade2.desc)"
error: compare-cost: spending conditions exceed --max-conditions cap (73728 > 4096); raise the cap or simplify the policy

$ time mnemonic compare-cost --descriptor "$(cat .examples-build/degrade2.desc)" --max-conditions 100000
Condition                                               | wsh vB | tr vB |  Δ vB
--------------------------------------------------------+--------+-------+-------
key[0] + key[1] + key[2] + preimage(h0) + after(height) |    105 |   232 |  +127
key[3] + key[4] + preimage(h0) + after(time)            |     87 |   216 |  +129
key[6] + key[7] + older(blocks)                         |     79 |   208 |  +129
key[8] + older(512s)                                    |     61 |   192 |  +131
note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1
      from real-tx accounting, Δ values are correct
… 194.95s user 0.07s system 99% cpu 3:15.67 total
```
