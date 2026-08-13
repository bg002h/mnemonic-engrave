# Lens: timelock semantics — the degrade2 "pathological example"

**Slug:** `timelocks` · **Date:** 2026-08-11 · **Subject:**
`/scratch/code/shibboleth/mnemonic-toolkit/.examples-build/degrade2.desc`
(`wsh(...)#4ld0crxa`, 11 xpubs, 4 tiers, `or_i` nested 3 deep)

Every number below was **measured**, not derived. The probe crate is at
`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/22fd28a4-d68a-47d6-82b1-8a8570fb5417/scratchpad/tlprobe`
(4 binaries: `tlprobe`, `plan`, `exhaust`, `interp`), built against the on-disk
`/scratch/code/shibboleth/rust-miniscript-fork` (miniscript 13.0.0, bitcoin 0.32.8,
bitcoin-units 0.1.3 — `Cargo.lock:43-45`, `:77-79`).

---

## 0. The four tiers, decoded

| Tier | Branch | Quorum | Lock fragment | Kind | Field decode | Real duration / moment |
|---|---|---|---|---|---|---|
| T1 | outer `or_i` left | 3-of-3 + sha256 preimage | `after(1000000)` | CLTV **absolute height** | `1000000 < 500000000` ⇒ height | block 1,000,000 (est. ~2027-04, see §5) |
| T2 | 2nd `or_i` left | 2-of-3 + same preimage | `after(1893456000)` | CLTV **absolute time** | `≥ 500000000` ⇒ Unix time | **2030-01-01 00:00:00 UTC** (measured: `date -u -d @1893456000`) |
| T3 | 3rd `or_i` left | 2-of-2 | `older(65535)` | CSV **relative blocks** | `0x0000ffff`: bit31=0, bit22=0, low16=**65535** | 65,535 blocks ≈ **455.10 d** at 600 s/blk |
| T4 | 3rd `or_i` right | 1-of-3 | `older(4255898)` | CSV **relative 512-s** | `0x0040f09a`: bit31=0, bit22=**1**, low16=**61594** | 61594 × 512 s = **31,536,128 s = 365.0015 d** |

Witness script is **498 bytes** (measured, `Miniscript::script_size()` and
`explicit_script().len()`), and each lock sits inside its own `OP_IF` arm — the
untaken arms are never executed:

```
OP_IF  <40420f> OP_CLTV OP_VERIFY … OP_CHECKMULTISIG            <- T1 (after 1000000)
OP_ELSE OP_IF  <80d8db70> OP_CLTV OP_VERIFY … OP_CHECKMULTISIG  <- T2 (after 1893456000)
OP_ELSE OP_IF  <ffff00> OP_CSV OP_VERIFY … OP_CHECKMULTISIG     <- T3 (older 65535)
OP_ELSE        <9af040> OP_CSV OP_VERIFY … OP_CHECKMULTISIG     <- T4 (older 4255898)
```

All four operands are minimally-encoded CScriptNums (`ffff00` carries the sign
byte because `0xffff` alone would read negative; `9af040` is LE `0x40f09a`).
No MINIMALDATA or negative-locktime hazard.

---

## 1. (a) Does any single satisfaction path mix incompatible lock kinds? — **NO. Clean.**

Established three independent ways:

**AST walk of `ExtData::timelock_info`** (`tlprobe`): every node reports
`contains_combination = false`, and every leaf-bearing branch carries exactly one
of the four flags:

```
or_i             csv_h=1 csv_t=1 cltv_h=1 cltv_t=1 COMBO=false
  and_v (T1)     csv_h=0 csv_t=0 cltv_h=1 cltv_t=0 COMBO=false
  or_i
    and_v (T2)   csv_h=0 csv_t=0 cltv_h=0 cltv_t=1 COMBO=false
    or_i
      and_v (T3) csv_h=1 csv_t=0 cltv_h=0 cltv_t=0 COMBO=false
      and_v (T4) csv_h=0 csv_t=1 cltv_h=0 cltv_t=0 COMBO=false
```

**Planner** (`plan`): all four tiers plan, each with exactly one lock and never
two:

```
T1 3-of-3 + preimage + after(H 1000000)   PLANNED abs=Some(1000000 blocks)     rel=None                       wu=260
T2 2-of-3 + preimage + after(T ...)       PLANNED abs=Some(1893456000 seconds) rel=None                       wu=188
T3 2-of-2 + older(65535 blocks)           PLANNED abs=None                     rel=Some(Blocks(Height(65535))) wu=156
T4 1-of-3 + older(4255898 = 512s)         PLANNED abs=None                     rel=Some(Time(Time(61594)))     wu=82
```

Each tier also refuses to plan when handed the *wrong-unit* lock asset (4 negative
cases, all `no plan`) — so the asset→branch mapping is unit-correct.

**`mnemonic compare-cost`** (v0.97.0, `--max-conditions 100000`) enumerates 12
spending conditions and labels each with exactly one lock: `after(height)` ×2,
`after(time)` ×6, `older(blocks)` ×1, `older(512s)` ×3. No row carries two.

**Exhaustive 2^32 proof** (`exhaust`, release build) over the whole nLockTime and
nSequence domains, using the library's own `Satisfier` impls:

```
nLockTime domain = 2^32
  satisfies T1 after(1000000) only        : 499000000      (= 500,000,000 − 1,000,000 ✓)
  satisfies T2 after(1893456000) only     : 2401511296     (= 2^32 − 1,893,456,000 ✓)
  satisfies BOTH T1 and T2                : 0
nSequence domain = 2^32
  satisfies T3 older(65535 blocks) only   : 16384          (= 2^14 free reserved bits ✓)
  satisfies T4 older(4255898 = 512s) only : 64585728       (= 3942 × 2^14 ✓)
  satisfies BOTH T3 and T4                : 0
```

The BIP-68/BIP-65 hazard is a *conjunction* hazard. Here all four locks meet under
`or_i` disjunctions, so each tier needs one field value of one type. No branch is
unspendable.

## 2. (b) What rust-miniscript's mixed-timelock lint says about **this** descriptor — **it stays silent, correctly**

```
desc.sanity_check()          : Ok(())
ms.has_mixed_timelocks()     : false
ms.ext_check(ExtParams::sane): Ok(())
top-level timelock_info      : TimelockInfo { csv_with_height: true, csv_with_time: true,
                                              cltv_with_height: true, cltv_with_time: true,
                                              contains_combination: false }
```

Mechanism, from source: `TimelockInfo::combine_threshold`
(`src/miniscript/types/extra_props.rs:55-86`) only sets `contains_combination`
when `k > 1` (`:71`), i.e. under a conjunction, and only for the pairs
`csv_height×csv_time` / `cltv_height×cltv_time` (`:72-77`). `or_i` combines with
`k = 1` (`combine_or`, `:50-52`), which propagates the flags upward but never
raises the combination bit. `has_mixed_timelocks` reads exactly that bit
(`src/miniscript/analyzable.rs:198`, `:42`), and `sanity_check` maps it to
`AnalysisError::HeightTimelockCombination` (`analyzable.rs:141`, `:234-235`).

The lint is not asleep — controls prove it fires on genuine conjunctions:

```
and_v(v:older(65535), and_v(v:older(4255898), pk(K)))   REFUSED: Contains a combination of heightlock and timelock
and_v(v:after(1000000), and_v(v:after(1893456000),...)) REFUSED: Contains a combination of heightlock and timelock
and_v(v:after(1000000), and_v(v:older(4255898), ...))   accepted, mixed=false   <- correct: nLockTime and nSequence are independent fields
```

## 3. (c) Is `older(4255898)` a correctly-formed BIP-68 time lock? — **YES**

`4255898 = 0x0040F09A`:

| Field | Value | Source |
|---|---|---|
| bit 31 disable (`SEQUENCE_LOCKTIME_DISABLE_FLAG`, `0x80000000`) | **0** | `bitcoin-0.32.8/src/blockdata/transaction.rs:354` |
| bit 22 type flag (`SEQUENCE_LOCKTIME_TYPE_FLAG`, `0x00400000`) | **1** ⇒ 512-second units | `transaction.rs:356` |
| low 16 value (`SEQUENCE_LOCKTIME_MASK`, `0x0000FFFF`) | **61594** | `transaction.rs:499-500` ("BIP-68 only uses the low 16 bits") |
| bits outside `{low16, bit22}` | **0x00000000 — clean** | measured |

Measured round-trip: `Sequence::from_consensus(4255898).to_relative_lock_time()`
→ `Some(Time(Time(61594)))`, `is_time_locked = true`, `is_height_locked = false`.

**Real duration: 61594 × 512 s = 31,536,128 s = 365.0015 days** — i.e. one
365-day year plus 128 seconds. (Consensus measures this against MTP, and BIP-68
compares against the MTP of the block *preceding* the UTXO's confirming block, so
the wall-clock delay is marginally longer. Immaterial at this magnitude.)

## 4. (d) Is `older(65535)` at or over the relative-BLOCK ceiling? — **exactly AT it. The repo's claim checks out.**

The BIP-68 value field is the low 16 bits, so **65535 is the maximum** — at the
ceiling, not over it. Verified rather than repeated:

```
older(65535)  -> 0x0000ffff  bit22=0 low16=65535  clean   to_relative_lock_time() = Some(Blocks(Height(65535)))
older(65536)  -> 0x00010000  bit22=0 low16=0      bits outside {low16,bit22}: 0x00010000
                                                  to_relative_lock_time() = Some(Blocks(Height(0)))   <- MASKS TO ZERO
older(65537)  -> low16=1                          to_relative_lock_time() = Some(Blocks(Height(1)))
older(4194304)-> bit22=1 low16=0                  to_relative_lock_time() = Some(Time(Time(0)))
```

The masking is not only in `CalculateSequenceLocks` — BIP-112's `CheckSequence`
masks the **script operand itself** with `SEQUENCE_LOCKTIME_TYPE_FLAG |
SEQUENCE_LOCKTIME_MASK = 0x0040FFFF` before comparing, so `older(65536)` becomes a
0-block lock inside the opcode. The repo's own authoritative constants table
(`mnemonic-toolkit/cycle-prep-recon-cycle6-timelock-decay.md:105-108`, recorded as
confirmed against `bip-0068.mediawiki`) matches the rust-bitcoin constants cited
above. **Claim verified: 65535 is the largest safe block value; 65536 masks to
zero.**

Neither operand in degrade2 is masked (both `clean` above), so this hazard **does
not apply to this descriptor**. What the measurement does show is that
rust-miniscript will not protect you: `older(65536)`, `older(65537)` and
`older(4194304)` all **parse and pass `sanity_check()`**, because
`RelLockTime::try_from<Sequence>` only rejects bit-31-set and zero
(`src/primitives/relative_locktime.rs:70-79`). `mnemonic-toolkit`'s authoring gate
(`descriptor_builder/gate.rs:257-299`, predicate `timelock_advisory.rs:41-65`) is
carrying that weight alone.

---

## 5. Findings

### F1 — T3/T4 relative tiers are **inverted**: the 1-of-3 tier matures ~90 days *before* the 2-of-2 tier · IMPORTANT · operational (not consensus, not standardness)

A degrading vault should relax the quorum monotonically *later*. Measured:

| Tier | Quorum | Delay from UTXO confirmation |
|---|---|---|
| **T4** | **1-of-3** (single signature) | 31,536,128 s = **365.00 days** — exact, unit is wall-clock |
| **T3** | **2-of-2** | 65,535 blocks ≈ **455.10 days** at the 600 s target |

T4 — the weakest tier in the whole policy, satisfiable by one key — opens roughly
**90 days earlier** than T3. For T3 to open first, the sustained average
interblock time would have to fall below **481.21 s = 8 min 01 s** (measured:
`31536128 / 65535`), a ~20 % overshoot of the difficulty target that retargeting
erases inside two weeks. In practice the inversion is certain.

The cost model reinforces it: given every asset, the planner picks T4 at **82 WU**
versus T3's 156 WU (`plan`, measured) — the weakest tier is also the cheapest and
the earliest.

This is not a bug in miniscript and not a chain-level failure; the script does
exactly what it says. It is a **policy-authoring defect** in the example wallet.

### F2 — The two CLTV tiers can never be spent in the same transaction · IMPORTANT · operational

`nLockTime` is a single per-transaction field, and BIP-65 requires the operand and
`nLockTime` to be the same type. T1's operand is a **height** (1,000,000 <
`LOCK_TIME_THRESHOLD` = 500,000,000, `bitcoin-units-0.1.3/src/locktime/absolute.rs:26`)
and T2's is a **time**. Exhaustively measured over all 2^32 candidates: **0**
values satisfy both. `is_same_unit(after(1000000), after(1893456000)) = false`;
both directions of `is_implied_by` are `false`
(`bitcoin-0.32.8/src/blockdata/locktime/absolute.rs:247-255`).

Consequence for a wallet: a coin-selection pass that picks one T1-eligible UTXO
and one T2-eligible UTXO into a single transaction produces an **unsignable**
transaction. The asymmetry is worth stating precisely — `nSequence` is *per input*,
so **T3 and T4 inputs batch fine together, and either batches fine with a T1 or a
T2 input**. The only forbidden pair is **T1 + T2**.

### F3 — rust-miniscript's *interpreter* false-PASSes both BIP-65 and BIP-112 side conditions · IMPORTANT · TOOLING (false GREEN, not a chain failure)

`Interpreter::from_txdata` (`src/interpreter/mod.rs:143-152`) takes only
`sequence` and `lock_time`. There is **no transaction-version parameter anywhere
in the interpreter** (`grep -n version src/interpreter/*.rs` → empty), and
`evaluate_after` (`src/interpreter/stack.rs:205-224`) never reads `self.sequence`.
Both consensus side conditions are therefore structurally unreachable. Measured
with real signed spends (`interp`):

```
### BIP-65: CLTV must fail when the input's nSequence is FINAL (0xFFFFFFFF)
  after(1000000)+pk, nSeq=0xfffffffe  [consensus OK]    -> SATISFIED (2 constraints)
  after(1000000)+pk, nSeq=0xffffffff  [consensus FAIL]  -> SATISFIED (2 constraints)   << FALSE PASS

### BIP-112: CSV must fail when the spending tx nVersion < 2
  older(65535)+pk,   v2               [consensus OK]    -> SATISFIED (2 constraints)
  older(65535)+pk,   v1               [consensus FAIL]  -> SATISFIED (2 constraints)   << FALSE PASS
  older(4255898)+pk, v1               [consensus FAIL]  -> SATISFIED (2 constraints)   << FALSE PASS
```

Controls in the same run prove the interpreter *does* enforce everything numeric
and unit-related, so this is a narrow gap rather than a broken component:

```
  older(65535) blocks vs 512s-unit nSeq   -> Err(RelativeLockTimeNotMet(Blocks(Height(65535))))
  older(65535), nSeq one short            -> Err(RelativeLockTimeNotMet(Blocks(Height(65535))))
  after(1000000) height vs time nLockTime -> Err(AbsoluteLockTimeComparisonInvalid(1000000 blocks, 1893456000 seconds))
  older(65535), nSeq disable bit set      -> Err(RelativeLockTimeDisabled(Blocks(Height(65535))))
```

**Affects all four tiers** — T1/T2 via the BIP-65 gap, T3/T4 via the BIP-112 gap.
The impact is a lying offline verifier ("this spend is valid") for a transaction
consensus rejects; it does not lose funds by itself, but it defeats a
pre-broadcast check.

The **PSBT finalizer path is safe** and shows what the correct check looks like:
`PsbtInputSatisfier::check_after` calls `enables_lock_time()`
(`src/psbt/mod.rs:329-332`; `TxIn::enables_lock_time` is `sequence != Sequence::MAX`,
`bitcoin-0.32.8/src/blockdata/transaction.rs:247`), and `check_older` rejects
`version < transaction::Version::TWO` (`src/psbt/mod.rs:339-347`). Use the
finalizer, not the interpreter, to validate a degrade2 spend.

This is upstream rust-miniscript behaviour — the fork's last three commits under
`src/interpreter/` are the SortedMulti refactors, not timelock changes.

### F4 — No tier-ordering lint reaches this descriptor; the one that exists is archetype-only · MINOR · tooling gap

`mnemonic-toolkit` already implements exactly the check F1 needs.
`descriptor_builder/archetype.rs:305-338` refuses a **cross-unit** `--older` /
`--recovery-older` pair ("a block delay and a 512-second delay cannot be totally
ordered without baking in a block-interval assumption"), and `:349-380` (same `if` block) refuses an
already-past absolute `--after`. Both sit inside `if def.id ==
"decaying-multisig"` (`archetype.rs:305`), so they run **only** for that preset's
parameters. A `--spec`-authored or externally-supplied descriptor — which is what
degrade2 is — gets neither.

degrade2's T3/T4 pair (`older(65535)` blocks vs `older(4255898)` 512-second) is
*precisely* the cross-unit shape D-decay-rel was written to refuse. Had the same
wallet been built from the preset it would have been rejected at authoring time.

### F5 — `me` (the engraving path) has no timelock awareness at all · NOTE · informational

`grep -rniE '\bolder\b|timelock|bip.?68' /scratch/code/shibboleth/mnemonic-engrave/crates/`
returns **nothing**. `me-cli` works at the bech32/chunk/header level
(`md_codec::codex32::unwrap_string`, `chunk::ChunkHeader`) and never walks the
descriptor node tree, so `mnemonic-toolkit`'s BIP-68 masked-`older()` advisory
(`timelock_advisory.rs:102-127`) has no counterpart on the engrave path. Harmless
for degrade2 (both operands clean, §4), but a masked `older()` would reach a
plate with no warning. Note also that `md-codec` itself does not screen: it reads
a raw 32-bit operand (`crates/md-codec/src/tree.rs:293-296`) and hands it to
`RelLockTime::from_consensus` (`crates/md-codec/src/to_miniscript.rs:591-594`),
which accepts 65536.

### F6 — `Descriptor::from_str` runs no sanity check for `wsh(...)` · MINOR · tooling trap

`FromStr for Descriptor` only calls `sanity_check()` inside the `Descriptor::Tr`
arm (`src/descriptor/mod.rs:1141-1153`), and `Wsh::from_tree`
(`src/descriptor/segwitv0.rs:189-198`) calls `Miniscript::from_tree` — which,
unlike `Miniscript::from_str` (`src/miniscript/mod.rs:1063-1071`, `ExtParams::sane()`),
performs no `ext_check`. So parsing a `wsh` descriptor successfully proves
**nothing** about timelock mixing; a caller must call `sanity_check()` explicitly.
degrade2 passes when you do call it (§2), so this does not bite here — but any
tool that treats "it parsed" as "it is sane" is wrong for every `wsh` descriptor.

### F7 — A hand-rolled tuple `Satisfier` over this descriptor would violate the library's own stated contract · MINOR · tooling

`src/miniscript/satisfy/mod.rs:97-108` warns: *"If a descriptor mixes time-based
and height-based timelocks, the implementation of this method MUST only allow
timelocks of either unit, but not both. Allowing both could cause miniscript to
construct an invalid witness."* degrade2 is exactly that descriptor — all four
kinds. The tuple `Satisfier` impl ORs across its components
(`satisfy/mod.rs:524-542`), so a natural-looking
`(seq_blocks, seq_512s, lt_height, lt_time)` satisfier violates the MUST.

The library's own providers are safe by construction, and this is worth knowing
rather than fixing: `Assets` holds **one** `Option` per class
(`src/plan.rs:515-519`) with last-write-wins semantics — measured,
`.older(blocks).older(512s)` → `Some(Time(Time(61594)))`,
`.after(height).after(time)` → `Some(1893456000 seconds)` — and
`PsbtInputSatisfier` reads the single real `nSequence`/`nLockTime` off the
transaction. Only a bespoke satisfier can get this wrong.

---

## 6. Risks checked that do NOT apply

- **Unspendable branch from mixed locks.** None. Every tier carries exactly one
  lock of one kind (§1); `contains_combination = false` at every node.
- **Consensus failure.** None found. The script is consensus-valid and each tier
  is satisfiable by a well-formed transaction.
- **Standardness/relay failure attributable to timelocks.** None. Nothing here
  triggers `IsFinalTx`/`CheckSequenceLocks` beyond the ordinary "not yet mature"
  behaviour every timelocked output has. (Script *size* and the 73,728-condition
  `compare-cost` refusal belong to the resource-limits lens, not this one.)
- **BIP-68 consensus masking.** Does not apply: both `older()` operands are clean
  (no bits outside `{low16, bit22}`, non-zero value) — §3, §4.
- **bit-31 disable flag / no-op CSV.** Not present; both operands have bit 31
  clear.
- **`older(65535)` over the ceiling.** No — it is exactly at the ceiling.
- **Zero-value or out-of-range absolute locktimes.** Both `after()` operands are
  within `MIN_ABSOLUTE_LOCKTIME=1` .. `MAX_ABSOLUTE_LOCKTIME=0x7FFFFFFF`
  (`src/primitives/absolute_locktime.rs:10`, `:17`).
- **Non-minimal / negative CScriptNum operands.** All four pushes are minimal
  (§0); none can read negative.
- **Absolute-tier ordering.** T1 (height 1,000,000) opens before T2 (2030-01-01);
  that pair is correctly ordered. Using the repo's own anchor
  `ABS_HEIGHT_PAST_FLOOR = 900_000` documented as "~mid-2025"
  (`mnemonic-toolkit/.../timelock_advisory.rs:88-93`), block 1,000,000 lands
  ~100,000 blocks ≈ 694 days later, i.e. **~2027-04 (estimate)**. Both absolute
  locks are still in the future, so `D-decay-abs` would not fire either.
- **`RelLockTime` ordering across the mask.** `Ord` compares the raw `u32`
  (`src/primitives/relative_locktime.rs:95-101`), so a masked operand would sort
  wrongly — but no operand here is masked.

## 7. One-line consequence

The descriptor is **timelock-correct at the consensus level and clean under
rust-miniscript's mixed-timelock lint**; the real defects are that its two
relative tiers unlock in the wrong order (**F1**, the 1-of-3 tier ~90 days before
the 2-of-2 tier), that its two absolute tiers can never share a transaction
(**F2**), and that the library's interpreter will tell you a v1 or final-sequence
spend of it is valid when consensus will not (**F3**).
