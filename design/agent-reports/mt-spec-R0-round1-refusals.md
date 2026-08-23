# R0 round 1 — adversarial funds-safety pass on §8 (the refusals)

Artifact: `design/SPEC_mt_v0_1.md` at `1e74d4b`.
Lens: *following this spec exactly, in what sequence of realistic events does an
operator lose money, or cut a plate that cannot be broadcast when they need it?*
Reviewer: independent context, no authorship of the artifact.

Method: I enumerated what can be wrong with a finished transaction arriving from
arbitrary software, then checked §8 against that list — not the list against §8.
Every scenario below names concrete inputs and a concrete wrong outcome. Facts
about `rust-bitcoin`, the RCW fixture and the measurement probes were **executed
or read from source**, not taken from the spec; citations are given inline.

## Verdict

| severity | count |
| --- | --- |
| **Critical** | **7** |
| **Important** | **9** |
| Minor | 2 |
| Nit | 0 |

Not GREEN. The scope cut worked — `mt` really can no longer get an input amount
wrong — but §0's own framing is exact and unforgiving: *"Everything it can still
get wrong is a failure to inspect what it was handed, so §8 — the refusals —
carries the entire safety argument."* §8 currently inspects **the finalization
state and the scripts**. It does not inspect **the transaction**: no value
conservation, no fee, no standardness, no duplicate inputs. Four of the seven
Criticals are that one hole seen from four directions.

The second theme is the **two verbs**. §8 says refusals bind both; the spec's
*justifications* are written for `ur:psbt` only, and the string verb carries a
raw transaction. Where a justification depends on the payload, it silently fails
for `mt string` — §8.1, §8.2, §6 and §7 all do this.

Three ground facts I established by execution, used repeatedly below:

- **`bitcoin::consensus::verify_transaction` verifies scripts and nothing else.**
  Its whole body is a loop calling `verify_script_with_flags` per input
  (`~/.cargo/registry/src/index.crates.io-*/bitcoin-0.32.101/src/consensus/validation.rs:82-107`).
  No value conservation, no duplicate-outpoint check, no `nValue` range, no size
  or weight check. Its own doc even delegates the duplicate case back to the
  caller: *"The `spent` closure should not return the same [`TxOut`] twice!"*
- **`Psbt::extract_tx()` — the "safe API" §3 optimised the MIN form for — has
  two refusal paths §8 never checks**: `ExtractTxError::SendingTooMuch` when the
  fee is negative, and `ExtractTxError::AbsurdFeeRate` above
  `DEFAULT_MAX_FEE_RATE = 25_000` sat/vB (`bitcoin-0.32.101/src/psbt/mod.rs:136,
  196-216`).
- **The RCW's tier 2 is a *relative* timelock.** `older(32768)` in both
  `policy-tr.txt` and `policy-wsh.txt`, compiled to `OP_PUSHBYTES_3 008000
  OP_CSV` — printed by the probe itself
  (`RESULTS_rcw_2026-08-22.txt`, "tr leaf depth 2"). Tiers 3 and 4 are
  *absolute* (`after()` → `OP_CLTV`), tier 1 has no timelock.
  **Correction to my brief:** it is **one** of four tiers that is relative, not
  three. The finding stands on that one, and is sharpened by a second fact: no
  scenario in `rcw.rs` or `psbtfinal.rs` exercises tier 2 — all three measured
  scenarios use `Sequence(0xFFFF_FFFE)` (`rcw.rs`, the three-scenario list in `main`), so the one tier §8.4
  cannot express is also the one tier the spec never measured.

---

### R-1 — §8.2 verifies scripts, not transactions: a transaction that spends more than it holds passes every refusal

**Severity: Critical.  Sections: §8.2, §8 preamble.**

§8.2 says *"Script-invalid → refuse. Real libbitcoinconsensus verification"*, and
the §8 preamble promises *"All are machine-checkable before a single plate is
cut."* The spec treats per-input script verification as if it established
transaction validity. It does not (ground fact 1 above).

**Scenario.** The operator's wallet — or a coin-selection bug in it, or a
hand-assembled transaction from a script that read one UTXO's value from a stale
cache — produces a 2-input transaction whose outputs total more than its inputs.
Every signature is valid: `SIGHASH_ALL` commits to the outputs *as they are*, so
the signer signed exactly this. The transaction is finalized. Run §8 against it:

- §8.1 finalized — both inputs carry `PSBT_IN_FINAL_SCRIPTWITNESS`. **Pass.**
- §8.2 script-valid — `verify_transaction` loops the two inputs, both scripts
  verify. **Pass.**
- §8.4 `--immediate`, warning printed and accepted. **Pass.**
- §8.5 `gettxout` returns non-null for both. **Pass.**
- §8.6 native segwit, `SIGHASH_ALL`. **Pass.**

Two plates cut, ~42 minutes. The transaction is consensus-invalid
(`bad-txns-in-belowout`) and can never be mined by anyone, ever. In 2040 the
holder's own tooling refuses it before the network does:
`Psbt::extract_tx()` returns `SendingTooMuch` (ground fact 2).

**Why it matters.** §3 rejected the `lean` payload form for exactly this
property — *"the **safe** API a recoverer reaches for refuses it — the wrong
property for a plate read in 2040"*. The spec has already ruled that "the safe
API refuses this plate" is unacceptable. §8 then admits transactions that hit
two *other* refusal paths of the same API. This is not me asking for more
checks; it is the spec's own acceptance criterion, applied to what §8 lets
through.

Same hole, other instances that reach steel unrefused: **duplicate inputs**
(`bad-txns-inputs-duplicate` — a `CheckTransaction` rule, and script
verification will happily verify the same outpoint twice), **`nValue` out of
range**, and **an empty input list** (against which §8.1's "every input must
carry a final witness" and §8.3 are both *vacuously true* — zero inputs, zero
violations).

*Non-authoritative sketch:* the missing tier is transaction-level checks that
`verify_transaction` was never meant to cover. Where to draw the line is a
design question — not mine to settle.

---

### R-2 — nothing anywhere bounds the fee, in either direction

**Severity: Critical.  Sections: §8 (whole), §5, §7.**

`mt` refuses on finalization, scripts, unspentness, sighash, size and module
pitch. It never computes the fee. §5 dropped fee rate from the legend; §7 says
of it *"cannot be fixed, and is **NOT** on the plate"*; §8.4's own warning text
concedes *"relay also depends on fee"* — and then no refusal looks at it. The
fee is the one number in a signed transaction that is both trivially computable
from the payload (§3's MIN form keeps every input's value precisely so it is)
and unbounded in its consequences.

**Scenario A — funds burned.** Tired operator, RCW `tr` tier 1, 5-in/2-out. The
wallet is asked to sweep and the change output is dropped (a wallet bug, a
mis-set "subtract fee from amount", or the operator deleting a line in a hand-
edited PSBT). Inputs total 4.9 BTC; the single remaining output pays the
intended 0.00399 BTC. **Fee: 4.896 BTC.** §8 refuses nothing — every check in
R-1's list passes, because a 4.9 BTC fee is perfectly valid. The legend's one
`TO` line reads `TO bc1p8rrz...s6n0vcl  0.00399 BTC`, which is *true* and is
exactly what the operator expects to see (see R-7). Five plates, ~105 minutes.
In 2040 the holder broadcasts; `sendrawtransaction`'s `maxfeerate` (default 0.10
BTC/kvB) rejects it, so they pass `-maxfeerate=0` to make their own plate work,
and 4.896 BTC goes to a miner.

**Scenario B — dead plate.** The transaction is built at 1 sat/vB because the
mempool was empty that evening. Cut. In 2040 the network floor is above that.
§7 already calls this class unfixable and I am not disputing that — but
§7 frames it as a *future* problem. A fee **below the current
`minrelaytxfee`** is unbroadcastable *the day the plate is cut*, is machine-
checkable at encode time, and is unrefused. A zero-fee transaction passes every
refusal in §8.

**Why it matters.** §8.3 states the spec's own principle: *"It cannot be
broadcast, so it is not a backup."* A transaction whose fee is 4.9 BTC or 0 sat
is not broadcastable in any practical sense, and §8 applies that principle only
to the signature state.

---

### R-3 — `mt string` has no working finalization check, and the physics claim that excuses its absence is false

**Severity: Critical.  Sections: §3, §8.1, §8.2, §3b, §10.10.**

§3 records the hazard and then argues one verb is immune:

> **A raw transaction cannot represent an unsigned one: if it serializes with
> witnesses, it is finished, and the format makes the mistake impossible.**

**That sentence is false.** An unsigned transaction serialises perfectly well as
a raw transaction: no segwit marker, empty `scriptSig` on every input. It is
what `bitcoin-cli createrawtransaction` returns and is the single most common
raw-transaction artifact in existence; `rust-bitcoin` deserialises it without
complaint. The claim is also false at input granularity — a 2-input transaction
with one input finalized and one empty *does* "serialize with witnesses" and is
not finished.

The false claim is load-bearing. §3b rules *"The payload is the raw signed
transaction, NOT the PSBT"*, §10.10 leaves open whether the CLI even accepts a
PSBT for that verb (*"PSBT or raw hex or both?"*), and the two checks that would
catch it are written in PSBT vocabulary:

- §8.1: *"Every input must carry a populated `PSBT_IN_FINAL_SCRIPTSIG` or
  `PSBT_IN_FINAL_SCRIPTWITNESS`"* — there are no PSBT fields in a raw
  transaction, so the check is inapplicable rather than failed.
- §8.2: *"The finalized PSBT carries each input's UTXO record, so ... **the data
  needed to run it always arrives with the payload**"* — false for a raw
  transaction, which carries no prevout scripts or amounts at all.

§8.3 states the rule (*"An unsigned or unfinalized transaction ... → refuse"*)
and names no mechanism, and §8's preamble asserts refusals *"bind BOTH verbs"* —
which is a claim about scope, not a mechanism either.

**Scenario.** Operator exports "unsigned transaction (hex)" from wallet
software — an option most wallets offer next to the PSBT export, one line apart
in the same menu. `mt string <hex>`. §8.1 has no PSBT to inspect. §8.2 cannot
run: `verify_transaction`'s `spent` closure has nothing to return, giving
`TxVerifyError::UnknownSpentOutput` — so `mt` either refuses every raw-hex input
(the verb is unusable) or skips the check (fails open). §8.4's `nLockTime` and
`nSequence` checks read fine off a raw transaction and pass. The operator
hand-engraves it over an evening. The plate holds a transaction nobody can
broadcast, and §8.1's headline — *"The raw-transaction format made this
impossible"* — is the reason nobody looked.

---

### R-4 — an `mt string` plate never states how much money it moves, and §6 and §7 both claim it does

**Severity: Critical.  Sections: §3b, §6, §7, §8.6.**

The two verbs carry different payloads, and §6/§7 were written for one of them.

- §6: *"The finalized PSBT closes part of this by carrying each input's UTXO
  record — value and scriptPubKey — so the engraved payload does describe what
  it spends, **which the bare raw transaction did not**."*
- §7, Pinned-fee row: *"a holder in 2040 recovers them by decoding, **since the
  PSBT carries the input amounts**"*.

§3b then makes the bare raw transaction the payload for `mt string`
**deliberately, for exactly those bytes**: *"Dropping the PSBT wrapper saves the
+58 to +61 bytes per input measured in §3."* The verb's design decision removes
precisely the record §6 identifies as closing the provenance gap, and neither §6
nor §7 notices.

**Scenario.** A hand-cut `mt1` plate, found in 2040. The holder decodes it
correctly — the BCH corrector fixes their two mis-cut characters, everything
works. They now know: the outputs. `TO bc1q…  0.00399 BTC`. They do **not**
know: what the inputs are worth, therefore what the fee is, therefore whether
broadcasting moves 0.004 BTC or 4.9 BTC (R-2's scenario A, engraved by the
string verb). To learn it they must resolve five outpoints against a chain
source — which requires those transactions to still be findable, requires
trusting a third party with the whole transaction, and returns nothing useful if
an input was already spent. The plate itself is silent on magnitude.

**Why it matters beyond the two false sentences.** §8.6 refuses every legacy
input *"because nothing in a legacy sighash commits to the input amount, so the
PSBT's UTXO record for it is unverifiable"*. For `mt string` there is **no UTXO
record on the plate for any input**, segwit or not — so the verb pays the full
cost of that refusal (see R-15) to protect the integrity of a record it does not
carry. The safety argument and the payload disagree about which artifact exists.

---

### R-5 — relative timelocks: §8.4 refuses the genuinely timelocked case, and `SPENDABLE AFTER BLOCK <n>` cannot state it

**Severity: Critical.  Sections: §8.4, §5, §7.**

§8.4: *"Under `--timelocked`, `mt` refuses unless **both** hold: `nLockTime` is
in the future, **and** at least one input is non-final."* Both conditions are
about the **absolute** timelock. The RCW's tier 2 is `older(32768)` → `OP_CSV`
(ground fact 3): a **relative** timelock of 32768 blocks (~7.5 months) measured
from the input's own confirmation, enforced by BIP-68 via `nSequence`.

**Scenario A — the truthful plate is refused, and the operator is routed to a
lie.** The operator engraves a tier-2 recovery spend: two cosigners plus the
tier-2 preimage, gated by `older(32768)`. The signed transaction has
`nSequence = 32768` on the input (non-final ✓) and `nLockTime = 0` — there is no
absolute lock, because the policy does not use one. `mt --timelocked` (the
default) refuses: `nLockTime` is not in the future. The operator is tired, the
transaction is correct, the refusal looks like a bug, and the flag that clears
it is right there. They pass `--immediate`. The plate is engraved
`IMMEDIATELY SPENDABLE`, and `mt` prints the loud warning that it *"may be
broadcast by anyone who holds or photographs the plate, from the moment it is
cut."* Both statements are **false**: the transaction cannot be mined for ~7.5
months after the input confirmed. The operator now stores a genuinely
time-locked plate under a threat model it does not have, and the 2040 reader
believes a liveness claim that was never true.

**Scenario B — the passing plate lies the other way.** A 2-input transaction:
input A is a tier-3 spend (`after(1173520)`, `nSequence = 0xFFFFFFFE`), input B
is a tier-2 spend (`older(32768)`). §8.4 passes — `nLockTime = 1173520` is in
the future, and both inputs are non-final. The legend reads
`SPENDABLE AFTER BLOCK 1173520`. Input B was confirmed at height 1,170,000, so
it is not BIP-68-final until 1,202,768. The holder waits for the height the
plate names, broadcasts, and the node returns `non-BIP68-final`. The plate told
the truth about `nLockTime` and lied about spendability, by ~7 months.

**Why the legend cannot be repaired by arithmetic.** A relative deadline is
`confirmation_height(input) + n`. The confirmation height is on neither plate
nor payload: the raw transaction has only outpoints, and the PSBT's
`witness_utxo` carries value and `scriptPubKey` and **no height**. §5 calls this
field *"the single most actionable fact: whether this plate is live yet"* — and
for a relative-timelocked input it is not computable from anything `mt`
engraves. §7's Bearer row inherits the defect: it claims a timelock bounds the
hazard *"only when §8.4's `nSequence` condition holds"*, a condition that is
neither necessary (scenario A) nor sufficient (scenario B) for the timelock the
holder actually faces.

---

### R-6 — §8.6 is written over signatures; a satisfaction with no signature has no sighash to check and binds nothing

**Severity: Critical.  Sections: §8.6, §7.**

§8.6 refuses *"Any input not signed with `SIGHASH_ALL` (or taproot's
`SIGHASH_DEFAULT`)"*. The rule presumes every input is *signed*. Miniscript
satisfactions need not contain a signature at all — a path of the form
`after(N) AND sha256(H)` is satisfied by a preimage and a locktime, with no key
involved.

**This is not hypothetical in this repo.** The operator's own reference wallet
had exactly such a tier until three months ago. Commit `d1889e4`, in the
operator's words: *"Tier 4 was after(1383520) AND sha256(H3) with no key at all;
it is now after(1383520) AND sha256(H3) AND pk(@6)"*. The same commit records
that stock `rust-miniscript` 13.1 **accepted the keyless WSH form** and rejected
only the TR form — so a signature-free spending path passes the ecosystem's own
descriptor validation in the wrapping most multisig wallets use.

**Scenario.** A wallet with an inheritance tier of `after(N) AND sha256(H)` —
the classic "the heir has the passphrase and waits for the date" construction.
The operator engraves that spend. The finalized witness contains the preimage,
the leaf script and the control block; **no signature**. §8.6 iterates the
input's signatures, finds none, and has nothing to refuse — the check is
vacuously satisfied, exactly as R-1's is for an empty input list. The plate is
cut.

Because no signature exists, **nothing commits to the outputs, the inputs, or
anything else**. This is strictly worse than the `SIGHASH_NONE` case §8.6 was
written to stop: anyone who holds or photographs the plate has the preimage (it
is on the plate), the locktime is public, and they can build an entirely new
transaction paying themselves. §7's last row claims this whole class is
*"refused at encode time, §8.6"*.

**A second, quieter gap in the same rule.** Nothing specifies how a signature is
*identified* inside an arbitrary witness. The measured RCW witnesses are
`wit[64, 32, 79, 129]` (tr tier 4: Schnorr sig, 32-byte preimage, leaf script,
control block) and `wit[0, 71, 71, 71, 32, 1, 390]` (wsh tier 1: `CHECKMULTISIG`
dummy, three DER signatures, preimage, `OP_IF` selector, witnessScript) —
`RESULTS_rcw_2026-08-22.txt`. Deciding which items are signatures requires
interpreting the script, and the spec does not say whether an input whose
signatures cannot be located is refused or admitted. Given R-1's and this
finding's pattern, the default reading is fail-open.

---

### R-7 — one truncated `TO` line cannot discharge §7's pinned-destination mitigation for a transaction with change

**Severity: Critical.  Sections: §5, §7, §4.**

§5's legend has exactly one destination field, measured once:

> `TO <truncated addr>  <amount>` | 34 | so a human sees where the money goes
> without a scanner

and the measurement behind it is headed **"MINIMAL legend (any input count)"**,
rendering `TO bc1p8rrz...s6n0vcl  0.00399 BTC`
(`RESULTS_legend_budget_2026-08-22.txt`). The sweep varies **inputs**. It never
varies **outputs** — while §4's own plate table engraves `1-in/2-out` and
`5-in/2-out` artifacts, and every transaction with change has two.

Two problems, one line.

**Multi-output blindness.** With two outputs and one `TO` line, the plate shows
one destination and conceals the other, and the spec does not say which. If it
shows output 0 and the wallet randomised the change position — which Bitcoin
Core does by default — the plate names the operator's own change address, with
the change amount, as though it were the payment. §7's mitigation for the
pinned-destination hazard is *"the `TO` line names the destination so the
operator sees what they commit to before cutting"*: for any transaction with
change, the operator sees at most half of what they commit to, and an output
substituted by compromised wallet software has a 50% chance of being the half
that is not shown. There is no room to fix it by adding a line: the same
measurement shows the strip beside the QR holds **2 lines at v22 and 0 lines
at v25 and above**, against a 6-line reservation that is already the reason
§4's plate counts double.

**Truncation.** ~17 of a 62-character bech32m address are engraved. A prefix/
suffix match is cheap to grind, so the one field a human is told to check before
committing 21 minutes a plate is checkable only against an attacker's budget.

**Scenario.** Operator reviews the legend `mt` prints, sees `TO bc1p8rrz...
s6n0vcl  0.00399 BTC` — their intended recipient, correct amount — and cuts. The
transaction's second output sends 4.9 BTC to an address they have never seen.
Nothing in §8 refuses it, nothing in §5 displays it, and §7 records the hazard
as mitigated.

---

### R-8 — §8.4 sets no minimum horizon, so `--timelocked` can cut a bearer-live plate with the loud warning suppressed

**Severity: Important.  Section: §8.4.**

The two conditions are *"`nLockTime` is in the future"* and *"at least one input
is non-final"*. Neither bounds how far in the future, and the second is weaker
than it reads: `nSequence != 0xFFFFFFFF` is set by **every RBF-signalling
wallet transaction in existence**, for reasons having nothing to do with
timelocks. Every artifact the spec measures uses `Sequence(0xFFFF_FFFE)`
(the scenario lists in `rcw.rs`, `psbtfinal.rs` and `envelope.rs`). In practice the conjunction
reduces to *"`nLockTime` > tip"*.

**Scenario.** The operator intends an inheritance plate locked to block
1,900,001 and drops a digit: **900001**. The chain is at 900,000. §8.4 passes —
it is in the future. No warning is printed, because the loud warning is bound to
the `--immediate` *flag*, not to the horizon actually achieved. The legend
engraves `SPENDABLE AFTER BLOCK 900001`, which is true and reads exactly like
the intended plate. Engraving takes ~21 minutes per plate: **the timelock
expires while the machine is still cutting.** The operator files a bearer
instrument in the drawer where they intended to file a time-locked one, having
selected the safe flag and been told nothing.

This composes with R-5: the operator's only two options are a check that passes
on one block of horizon, and a flag that warns loudly about a hazard the
transaction may not have.

---

### R-9 — §8 checks consensus and never policy: dust, burns and non-standard transactions reach steel

**Severity: Important.  Section: §8.**

The word "dust" does not appear in the spec. Neither does "standard" in the
standardness sense, "relay policy", or "mempool". §8.2 verifies against
consensus rules only. A transaction can be perfectly consensus-valid and never
relay on any node, which for a plate is indistinguishable from invalid.

**Scenario (dust).** A payment transaction leaves a 400-sat remainder as change
rather than dropping it — plenty of software does this, and the operator has no
reason to look. The output is below Bitcoin Core's dust threshold (546 sat for
P2PKH, 294 for P2WPKH, at the default `dustRelayFee`). §8 refuses nothing. In
2040 every node the holder tries rejects the transaction as `dust`. The plate is
scrap, and the funds are recoverable only if the wallet's keys still are — which
is precisely the assumption an engraved signed transaction exists to avoid.

**Scenario (burn).** An output carries value to an `OP_RETURN`. Core's
`GetDustThreshold` exempts unspendable outputs, so the dust rule does **not**
catch it, and the value is destroyed on confirmation. §8 refuses nothing. Core
25 — the exact version §6a pins as verified — added `maxburnamount` to
`sendrawtransaction`, defaulting to 0, so the holder must explicitly opt in to
their own plate's burn.

Others in the same class, unrefused: a transaction **version** outside the
standard range (non-standard on every node, and the boundary moved in Core 28);
a **P2WSH** witness exceeding the standard stack limits (100 items, 80 bytes per
item, 3600-byte witnessScript) — the RCW's 390-byte witnessScript is far under,
but a larger policy is not; and a **taproot annex** or unknown leaf version,
both consensus-valid and non-standard.

---

### R-10 — §8.5 holds the chain's true UTXO record beside the payload's claimed one and never compares them

**Severity: Important.  Sections: §6a, §8.5.**

§6a chooses `gettxout` partly for this: *"it returns `value` and `scriptPubKey`
together, **so the PSBT's claimed UTXO records can be checked against the chain
rather than trusted**"*. §8.5 then refuses on exactly one condition — `null`.
The comparison §6a justifies the RPC with is never made.

**Scenario.** A PSBT arrives with a `witness_utxo` whose value is wrong (a buggy
updater, a stale UTXO snapshot, or an attacker feeding a signer a false amount —
the pre-BIP-143 fee attack, which BIP-143 converts from theft into an invalid
transaction). §8.2 verifies each input's script **using the claimed amount**, so
if the signer was fed the same false amount, verification passes. §8.5 sees
non-null and passes. The node's response held the true value the entire time and
nothing looked at it. Plates are cut. The transaction can never be mined: the
network verifies against the real UTXO and the signature does not check.

The engrave-time cost of the comparison is zero — the value is already in the
response §8.5 parses.

---

### R-11 — the MIN form is two operations on a probe-built PSBT, not an admission rule for a wallet's PSBT

**Severity: Important.  Sections: §3, §8.2, §4.**

§8.2 disposes of missing UTXO records with *"refused under (1)'s sibling rule:
`mt` requires the MIN form of §3"*. There is no such rule in §8 — §8.1 is about
finalization, and "the MIN form of §3" is a description of a payload, not an
admission criterion. What §3 actually defines is what the probe does:
`for o in min.outputs.iter_mut() { *o = Default::default(); }`
(`psbtfinal.rs`, `for o in min.outputs.iter_mut() { *o = Default::default(); }`). Outputs cleared; **the global map and any unknown or
proprietary fields are untouched**, and the input maps are clean only because
`rust-miniscript`'s finalizer already stripped them.

Nothing says whether `mt` **refuses** a PSBT that is not already MIN, or
**normalises** it — and the two answers have different failure modes.

**Scenario (normalise).** A real multisig PSBT from Sparrow or Specter carries
`PSBT_GLOBAL_XPUB` entries — the wallet's extended public keys. Clearing the
output maps does not touch them. The plate is engraved carrying the operator's
**entire watch-only wallet**: every address, past and future, permanently, on an
artifact §7 already establishes is bearer and photographable. §7's hazard table
has no privacy row at all. The extra bytes also push the artifact into more
plates than §4's table predicts, since every figure in that table came from a
probe-built PSBT with no global map.

**Scenario (refuse).** `mt` requires byte-exact MIN and rejects most real wallet
PSBTs, sending a tired operator to hand-edit a PSBT — the one operation on this
whole path most likely to produce R-1's or R-2's transaction.

---

### R-12 — `nLockTime` has two domains and the legend has one unit

**Severity: Important.  Sections: §8.4, §5.**

`nLockTime` below 500,000,000 is a block height; at or above it, a Unix
timestamp. §8.4 says only *"`nLockTime` is in the future"* without saying
against what, and §5 hardcodes the field as `SPENDABLE AFTER BLOCK <n>`.

**Scenario.** The operator wants "spendable after 1 January 2040" and their
wallet encodes it the way wallets encode dates — as a timestamp, 2,208,988,800.
`mt --timelocked` compares it against a block height around 900,000, finds it
enormous, and passes. The legend is engraved
`SPENDABLE AFTER BLOCK 2208988800`. In 2040 the chain is near height 1,650,000.
The heir reads a plate claiming it becomes live roughly ten thousand years from
now, concludes it is junk or mis-cut, and sets it aside. The transaction was
spendable the whole time.

The inverse costs a refusal rather than funds: a height-domain `nLockTime`
compared against a wall-clock time is always "in the past" and `--timelocked`
refuses a correct transaction, routing the operator to `--immediate` and its
false `IMMEDIATELY SPENDABLE` legend — R-5's scenario A by another road.

---

### R-13 — `mt string`'s accepted risk is disclosed to the encoder and invisible on the artifact, and the plate is indistinguishable from the constellation's watch-only plates

**Severity: Important.  Sections: §7, §3b.**

*Filed under the operator's ruling, not against it.* Hand-cut plates get a
stderr warning and nothing on the plate; that is a decision. Two things about
how the spec **states** it are still open.

**(a) §7's wording claims more than one verb delivers, and the Bearer row is not
the only one.** §7 currently reads *"the `BEARER` line states it plainly, and it
is the first line on the plate"*. If only that row is edited, three others still
name on-plate mitigations that a hand-cut plate does not carry: the
pinned-destination row rests on *"the `TO` line"*, the locktime row on
`SPENDABLE AFTER BLOCK <n>` (§5, §8.4), and `PLATE n OF m` is §5's stated answer
to *"a missing plate must be obvious"*. §8's preamble asserts symmetry —
*"Every refusal below binds BOTH verbs"* — which makes the asymmetry of the
**mitigations** easy to miss, since the two sections are read as a pair.

**(b) The undisclosed consequence: the warning and the risk have different
lifetimes and different audiences.** The stderr warning is seen once, at encode
time, by the person who already knows what the artifact is. The plate is read
in 2040 by someone else. What that reader holds is a steel plate bearing a
chunked bech32 character string — *visually identical to the `md1` and `mk1`
plates beside it in the same box*, differing in one character of the HRP. §7's
own opening states the confusion this creates: *"the existing tooling's
assumption that 'public string' means 'safe to engrave' does not hold here."*
For `mt qr` the plate resolves the confusion in 41 engraved characters. For
`mt string` the only thing distinguishing a bearer instrument from watch-only
public material — which the constellation has trained its users to treat as safe
to photograph, mail, and store casually — is `mt1` versus `md1`.

**Scenario.** The operator's heir inventories the box: eleven steel plates, all
carrying similar-looking strings. Following the constellation's documentation
for the plates they recognise, they photograph the set to a phone and email it
to the family's advisor for help identifying them. Two of those plates are an
`mt string` pair.

*Non-authoritative sketch, respecting the ruling:* the only thing that travels
with a hand-cut artifact rather than beside it is **the string itself** — its
HRP, or a field inside the BCH-checksummed header. Whether that is worth its
cost is a codec decision and, under the Rust-primary rule, lands in the Rust
codec with vectors first. The alternative that costs nothing is to state the
asymmetry in §7's table as a third *"not mitigated"* row, in the same register
the two existing ones already use.

---

### R-14 — a plate set has no identity: the txid was dropped as derivable, but deriving it requires the set already be paired

**Severity: Important.  Sections: §5, §3.**

§5 drops the txid on the principle *"everything derivable from the decoded
transaction is duplication"*, answering "recoverable how" with *"hash the
decoded transaction"*. That is circular for the case it needs to cover: to
decode you must first have the right plates together.

What remains to identify a plate set is `PLATE n OF m` and `FROM WALLET
<8 hex>` — and the stub is **identical for every transaction of the same
wallet**, which is the whole point of it.

**Scenario.** The operator does what an inheritance plan requires: engraves
three spending paths of the RCW — the tier-1 immediate spend, a tier-3 spend
and a tier-4 spend — two plates each. Six plates, all reading `FROM WALLET
fa568be0`, all reading `PLATE 1 OF 2` or `PLATE 2 OF 2`. They are stored
together because they belong to one wallet, and in 2040 they are found loose.
There are 48 orderings to try. Mispairing is *detected* — a multi-part UR
carries `MessageLen` and `Checksum` (§10.8), and `md1`'s header carries a
chunk-set id — so nothing silently decodes wrong, which is right. But the field
§5 says exists so *"a missing plate must be obvious"* cannot distinguish a
missing plate from a mixed one, and the heir cannot tell whether they are one
plate short or one pairing away.

---

### R-15 — §8.6's legacy refusal rests on a false premise, and its boundary for P2SH-wrapped segwit is undefined

**Severity: Important.  Sections: §8.6, §6.**

The stated reason is *"nothing in a legacy sighash commits to the input amount,
so the PSBT's UTXO record for it is unverifiable (§6)"*, and §6 states it
absolutely: *"**For legacy inputs nothing commits to them at all**"*.

The premise about the **sighash** is true; the conclusion about
**verifiability** is not. BIP-174 requires a legacy input to carry
`PSBT_IN_NON_WITNESS_UTXO` — the **entire previous transaction** — whose txid
the outpoint commits to. Hashing it verifies the amount cryptographically. That
is exactly why BIP-174 mandates the field and why hardware wallets demand the
full parent. A legacy input's amount is verifiable; it is just expensive.

The defensible reason for the refusal is one the spec does not give: §3's MIN
form is measured at **+58 to +61 bytes per input**, which is a `witness_utxo`
and cannot be a parent transaction, so MIN structurally cannot carry the record
BIP-174 requires for a legacy input. The rule may well be right. The reasoning
written down is not, and it is the reasoning a future editor will act on.

**The undefined boundary, and its cost.** "Legacy (non-segwit)" does not
classify **P2SH-wrapped segwit** — `sh(wpkh(…))` and `sh(wsh(…))`. Those inputs
have a non-empty `scriptSig` and a P2SH `scriptPubKey`, so they look legacy; but
they sign under BIP-143, so the amount **is** committed by the signature, and
the stated rationale admits them while the stated rule plausibly excludes them.
`sh(wsh(multi(…)))` is the dominant multisig form of 2017-2021 and the default
for a large installed base of hardware wallets.

**Scenario.** The operator brings their `sh(wsh(2-of-3))` vault. Under one
reading `mt` refuses every input and the tool is simply unusable for that
wallet — after the transaction has been built and signed, which is the point at
which a tired operator starts looking for a way around a refusal. Under the
other, `mt` engraves inputs whose UTXO record the spec believes is unverifiable.
Nothing in §8 says which.

---

### R-16 — nothing requires the emitted artifact to decode back to the transaction §8 checked

**Severity: Important.  Section: §8.**

Every refusal in §8 inspects the **input**. No refusal inspects the **output**.
`mt` chunks, wraps in UR, tiles across plates, and emits — and the spec never
requires it to parse its own emission back and compare it to the transaction it
verified.

*This is distinct from §10.9*, which asks how an engraving reaches the machine.
The gap here is inside `mt`: a fragment-boundary off-by-one, a chunker that
mis-sets a header field for a payload type it has never carried (§10.13 flags
exactly that uncertainty for `mt1`), or a tiling that drops the last symbol,
would leave every §8 check passing and every plate wrong.

**Scenario.** §10.2 already records that a static multi-part `ur:bytes` QR has
been reported failing to scan into Sparrow at all, and calls the static
multi-part question *"the single most load-bearing unverified assumption in the
spec"*. The operator cuts five plates at 21 minutes each; the artifact never
reassembles. Encode-time round-trip is the one check that turns that from a
2040 discovery into an encode-time refusal, and §8 does not have it.

---

### R-17 — §8.7's "plate budget" names no number, while §8 promises every refusal does

**Severity: Minor.  Sections: §8.7, §0.**

§8 closes: *"Every refusal names the number that caused it. A refusal that says
only 'too large' costs the operator a round trip."* §8.7 refuses *"Over the
plate budget (`mt qr`)"*, and the budget is defined in §0's table as "the plate
budget". §8.7b, by contrast, names 64 chunks. `mt qr` will therefore accept a
transaction of any size — the spec's own largest measured artifact is 6 plates,
about two hours of machine time — with no stated ceiling to refuse against.

---

### R-18 — coinbase maturity is not checked

**Severity: Minor.  Sections: §8.5, §6a.**

A transaction spending a coinbase output before 100 confirmations is
consensus-invalid. `gettxout` returns `coinbase` and `confirmations` in the same
response §8.5 already parses, and §8 does not look at either. Self-correcting
for a timelocked plate cut far in advance; a dead plate for an immediate one cut
by an operator who mines.

---

## What §8 does not cover

The enumeration promised in the brief: everything I could establish can be wrong
with a *finished, signed* transaction arriving from arbitrary software, checked
against §8's nine refusals. "Caught" means some refusal in §8 rejects it as
written.

| # | defect in the received transaction | consequence if engraved | caught by §8? |
| --- | --- | --- | --- |
| 1 | partially signed / not finalized (PSBT) | unbroadcastable | **yes** — §8.1 |
| 2 | unsigned or partly signed **raw** transaction (`mt string`) | unbroadcastable | **no** — R-3 |
| 3 | invalid signature / unsatisfied script | unbroadcastable | **yes** — §8.2 |
| 4 | **outputs exceed inputs** (negative fee) | unbroadcastable forever | **no** — R-1 |
| 5 | **duplicate inputs** | consensus-invalid | **no** — R-1 |
| 6 | `nValue` out of range / negative | consensus-invalid | **no** — R-1 |
| 7 | empty input or output list | consensus-invalid; §8.1 passes vacuously | **no** — R-1 |
| 8 | **absurd fee** (dropped change output) | **funds burned to miner** | **no** — R-2 |
| 9 | **fee below `minrelaytxfee` / zero fee** | never relays, from day one | **no** — R-2 |
| 10 | **dust output** | non-standard, never relays | **no** — R-9 |
| 11 | **value paid to `OP_RETURN`** | funds destroyed; dust rule exempts it | **no** — R-9 |
| 12 | output to an unspendable / future-witness-version script | funds unspendable | **no** — R-9 |
| 13 | non-standard tx **version** (0, or above the standard range) | never relays | **no** — R-9 |
| 14 | P2WSH witness over standard stack/script limits | never relays | **no** — R-9 |
| 15 | taproot **annex** present, or unknown leaf version | never relays | **no** — R-9 |
| 16 | tx weight over `MAX_STANDARD_TX_WEIGHT` | never relays | incidentally — the plate budget binds first, at ~4 KB |
| 17 | input already spent | unbroadcastable | **yes** — §8.5, *only when a node is reachable* |
| 18 | input spent **in the mempool only** | unbroadcastable | **no** — disclosed, §6a |
| 19 | node in IBD / wrong chain → false `null` | false refusal | **no** — disclosed, §10.5 |
| 20 | PSBT UTXO record disagrees with the chain | unbroadcastable; §8.2 verifies the lie | **no** — R-10 |
| 21 | immature coinbase input | consensus-invalid until 100 blocks | **no** — R-18 |
| 22 | `SIGHASH_NONE` / `SINGLE` / `ANYONECANPAY` | outputs or inputs unbound → redirectable | **yes** — §8.6 |
| 23 | **input satisfied with no signature at all** | outputs unbound → redirectable | **no** — R-6 |
| 24 | signature present but not locatable in the witness | unknown; fail-open | **no** — R-6 |
| 25 | legacy (non-segwit) input | UTXO record unverifiable under MIN | **yes** — §8.6, on a false premise (R-15) |
| 26 | P2SH-wrapped segwit input | classification undefined | **unclear** — R-15 |
| 27 | `nLockTime` future but only by one block | bearer before the engraving finishes | **no** — R-8 |
| 28 | `nLockTime` as a **timestamp**, not a height | legend states a nonsense block number | **no** — R-12 |
| 29 | **relative** (`OP_CSV`) timelock, no absolute one | truthful plate refused → `IMMEDIATELY SPENDABLE` engraved falsely | **no** — R-5 |
| 30 | relative **and** absolute timelocks together | legend names a height the tx cannot be mined at | **no** — R-5 |
| 31 | more than one output | half the destinations invisible on the plate | **no** — R-7 |
| 32 | destination matching only in truncation | operator's pre-cut check defeated | **no** — R-7 |
| 33 | PSBT carrying `PSBT_GLOBAL_XPUB` / unknown fields | whole watch-only wallet engraved; plate count wrong | **no** — R-11 |
| 34 | over the plate budget (`mt qr`) | — | **yes**, but the number is undefined (R-17) |
| 35 | over 64 chunks (`mt string`) | — | **yes** — §8.7b |
| 36 | emitted artifact does not decode back to the checked tx | plates that never reassemble | **no** — R-16 |

Read down the "caught" column: §8 catches **the signature state, the scripts,
the sighash flags, unspentness and the size**. It catches nothing about
**value**, nothing about **relay policy**, nothing about **the transaction as a
whole**, and nothing about **what it emits**. For a tool whose entire remaining
job is *"to inspect what it was handed"*, those are the four inspections.

---

*Reviewer's note on scope.* I did not re-derive citations, the `ur:psbt`
envelope choice, PSBT overhead, or the operator's rulings; I did not audit
§3b's string format or run a mechanical fold check. §10.9, §10.10, §10.12,
§10.13 and §10.14 are treated as known-open and are cited only where a safety
argument silently depends on one of them (R-3 on §10.10, R-16 on §10.13).
