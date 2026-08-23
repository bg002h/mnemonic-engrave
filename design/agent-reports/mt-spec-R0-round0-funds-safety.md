# R0 round 0 — lens 2: funds safety and threat model

Artifact: `design/SPEC_mt_v0_1.md` @ `099a516` (538 lines, read in full).
Question answered: *following this spec exactly, in what sequence of realistic
events does an operator lose money, or cut a plate that cannot be broadcast when
they need it?*

Out of scope by brief, and not re-derived: citation anchors, the measured
arithmetic in `design/measurements/`, the three KNOWN-open questions as such, and
verification of BIP/RPC wording (lens 3). Findings that depend on a quoted
assertion I did not verify are marked **[depends on lens 3]**.

## Verdict

**6 Critical / 10 Important / 5 Minor / 1 Nit**

---

### S-1 — the amount-binding argument does not apply to legacy inputs at all

**Severity: Critical.** §6a (the tier table and the paragraph above it).

§6a's safety case rests on two sentences: *"BIP-341's sighash commits to
`sha_amounts` … for every input unless `SIGHASH_ANYONECANPAY` is set"* and
*"Segwit v0 commits to the signed input's own amount via BIP-143."* It then
concludes *"Signing catches an honest mistake."* Pre-segwit inputs — P2PKH, bare
multisig, non-segwit P2SH — are never mentioned, and for them the legacy sighash
commits to **no amount whatsoever**. The script interpreter does not see the
amount either, so §8.2's libbitcoinconsensus pass cannot notice.

**Scenario.** The operator is building exactly the transaction `mt` exists for: a
pre-signed sweep of an old cold-storage UTXO. The input is a 2013-era P2PKH
output. They are offline, so tier 1 is unavailable; they do not have the 2013
previous transaction to hand, so tiers 2 and 3 are unavailable; they pass
`--i-certify-amounts` and type the value from memory as `0.05 BTC`. It is
actually `5.0 BTC`. Outputs total `0.049`. Now, in order:

1. §8.3a is satisfied — the flag was given.
2. The signer signs. The legacy sighash does not commit to any amount, so the
   signature is **valid against the real 5.0 BTC output**. Nothing warns.
3. §8.2 runs libbitcoinconsensus against the asserted prevout and **passes** —
   legacy script evaluation never consults the value.
4. §8.1, §8.4, §8.5, §8.6, §8.7 all pass. The plate is cut.
5. Broadcast in 2040: the transaction is **consensus-valid** and confirms. The
   fee is `5.0 − 0.049 = 4.951 BTC`.

For a taproot or segwit-v0 input the same typo yields an *invalid* signature —
a dud plate, recoverable, cheap. For a legacy input it yields a **valid
transaction that pays away 99% of the input**, and there is no point between the
typo and the confirmation at which any check in this spec can fire.

**Why it matters.** §6a is titled "the most dangerous section in the spec" and
its entire tier structure is calibrated against a failure mode — "signing
catches an honest mistake" — that is simply absent for the input class most
likely to appear in a decade-dormant recovery spend. The tier table has one row
per *evidence source* and no column for *input type*, so it silently claims the
same protection for all three.

**Fix sketch (non-authoritative).** Make tier 4 unavailable for any input whose
prevout scriptPubKey is non-segwit — i.e. `--i-certify-amounts` refuses legacy
inputs outright, since for them there is no second check anywhere in the system.
Note this also means `mt` must know the prevout *script type* to police the
tiers, which it does not have in tier 4 either.

---

### S-2 — sighash flags are never inspected; a non-`ALL` input makes the legend a lie

**Severity: Critical.** §8 (refusals), §5 (`TO` line), §7 (threat model).

§6a names `SIGHASH_ANYONECANPAY` as the exception that breaks the cross-input
amount commitment, and then nothing anywhere in the spec ever looks at a sighash
byte. §8's seven refusals do not include one. `mt` engraves whatever the signer
returned.

**Scenario A — redirect.** The operator's coordinator or signing script produces
signatures with `SIGHASH_NONE|ANYONECANPAY` (a normal artifact of some
collaborative and coin-join-adjacent flows, and of a signer configured for
partial signing). `mt` sees a finalized transaction: every input carries a
witness, so §8.1 passes; libbitcoinconsensus verifies each input's script against
the supplied prevouts, so §8.2 passes. The legend is engraved
`TO bc1p8rrz...s6n0vcl  9.50000 BTC`. In 2039 a house guest photographs the
plate. `SIGHASH_NONE` commits to no outputs at all, so the photographer strips
the outputs, substitutes their own address, waits for the locktime (which *is*
committed) and broadcasts. The operator's coins are gone, and the plate they
still hold in the safe says the money went to their own recovery address.

**Scenario B — dud.** A `SIGHASH_SINGLE` input whose index exceeds the output
count, or an `ANYONECANPAY` set that made §6a's tier reasoning inapplicable
without anyone noticing, produces a plate whose transaction is unbroadcastable
or malleable in ways the operator never chose.

**Why it matters.** §5 justifies the `TO` line as the field that lets *"a human
see where the money goes without a scanner"*, and §6 argues at length that a
second source of truth about destinations is *"a funds-safety hazard, not a
feature."* With a non-`ALL` sighash the destination is not a property of the
signed artifact at all, and the legend states it as fact in steel. This is the
one hazard in the whole document where the plate actively misinforms the person
holding it.

**Fix sketch (non-authoritative).** Add a §8 refusal: every signature in every
input must be `SIGHASH_ALL` (or `SIGHASH_DEFAULT` for taproot), no exceptions,
no flag. Anything else is not "a spend to address X" and must not carry a `TO`
line.

---

### S-3 — the required locktime is inert when all inputs are final (`nSequence`)

**Severity: Critical.** §8.4, §7 (bearer row), §1.6.

The string `nSequence` does not appear in this spec. Consensus finality is not
"`nLockTime` is in the future" — a transaction is final, and therefore relayable
and mineable **today**, if every input carries `nSequence == 0xFFFFFFFF`,
whatever `nLockTime` says. `nLockTime` is only consulted when at least one input
is non-final.

**Scenario.** The operator uses a hardware wallet that sets `nSequence =
0xFFFFFFFF` on all inputs (the common non-RBF default on several signers), and a
`nLockTime` of block 1,383,520. `mt`'s §8.4 check reads `nLockTime` and compares
it to the tip: 1,383,520 is in the future, so the "broadcastable today" refusal
does **not** fire. The plate is cut with the legend
`SPENDABLE AFTER BLOCK 1383520`. That statement is false: the transaction is
final now and any node will relay it. A thief who takes the plate this evening
broadcasts it this evening. The legend told the operator, in engraved steel, that
they had nine years.

The plate cannot be repaired after the fact either: `nSequence` is committed to
by the sighash in every sighash type, so the value cannot be changed without
re-signing.

**Why it matters.** §7's hazard table gives exactly one mitigation for the
headline hazard of the entire format — *"**Bearer** — holder can broadcast |
required future locktime (§8) bounds it in *time*, not in space"* — and that
mitigation is void for a realistic and common transaction shape that §8's
refusal, as specified, cannot detect. §1's decision 6 ("A future locktime is
required by default") is likewise unenforced.

**Fix sketch (non-authoritative).** The §8.4 predicate is not `nLockTime >
tip`; it is `nLockTime > tip AND min(nSequence) < 0xFFFFFFFF`. When `mt`
*produces* the transaction it must set `nSequence` itself (0xFFFFFFFE keeps the
locktime live without signalling BIP-125 replaceability) — and the spec currently
does not say what `nSequence` a produced transaction gets, which is how this went
unnoticed.

---

### S-4 — nothing checks the value balance, so the classic forgotten-change loss is unguarded

**Severity: Critical.** §8 (all refusals), §6a, §9.

§6a states the hazard in its own words — *"Overestimate and the outputs exceed
the inputs. The transaction is simply invalid"* and *"Underestimate an input's
value and the real fee — `inputs − outputs` — is larger than intended. The
operator overpays, possibly enormously."* §8 then lists seven refusals, and
**none of them is `inputs − outputs`**. §8.2 is script verification;
libbitcoinconsensus evaluates the script interpreter for one input and does not
check the transaction's value balance, fee, or output sanity. So the arithmetic
that §6a calls the most dangerous thing in the spec is computed by nothing and
displayed by nothing.

**Scenario.** Tier 1, everything correct. The operator has a single 10 BTC input
and hands `mt` one output: 0.5 BTC to their recovery address. They intended a
second output — 9.49 BTC of change — and did not type it, because they are
building this at the end of a long session and the change output is the one
that is easy to forget when the tool does not ask for it. Amounts come from
`gettxout`, so they are authoritative. The signer signs a consistent, fully
valid transaction. §8.1–§8.7 all pass. The plate is cut, and in 2040 it confirms
with a **9.5 BTC fee**.

Every amount in this scenario is correct. There is no wrong number anywhere for
a tier to protect against — which is precisely why the tier table does not help,
and why the absence of a balance/fee check is a separate defect rather than a
restatement of §6a.

**Why it matters.** This is the single most common catastrophic error in
hand-built Bitcoin transactions, it is machine-checkable in one line, the artifact
is irreversible steel, and §5 additionally removed the fee from the legend so no
human sees it after the fact either. §9's exclusion — *"it does not choose which
UTXOs to spend or what fee rate is appropriate"* — is about **choosing**; nothing
in it excuses not **computing and refusing on** a fee that `mt` is in a position
to see.

**Fix sketch (non-authoritative).** Compute `fee = Σin − Σout` whenever every
input's amount is known (tiers 1–3), print it prominently with the implied
sat/vB before any plate is cut, and add a §8 refusal on `fee > N%` of input value
or `fee > absurd-fee` absolute, overridable by an explicit flag that names the
number. Refuse unconditionally on `Σout > Σin`.

---

### S-5 — the fifth tier is "no prevouts at all", and it passes every refusal in §8

**Severity: Critical.** §8.2, §8.3a, §1a, §6a (tier table).

The tier table enumerates four sources of an input amount. It omits the one the
engrave verb actually gets most often: **nothing**. §1a rules that `mt`
*"engraves only the signed result"*, and the natural form of a signed result is a
raw serialised transaction — the `hex` out of `finalizepsbt`, or whatever a
signer exports. A raw transaction carries no prevout amounts and no prevout
scripts. Follow the refusals for `mt engrave signed.hex`:

- §8.1 "Not finalized → refuse. Every input must carry a witness or scriptSig."
  Passes — this is a purely structural test that any non-empty witness satisfies,
  including a placeholder or a witness for the wrong script.
- §8.2 "Script-invalid → refuse, **when prevouts are supplied**." Does **not
  fire**. Not "refuses"; *silently skips*. Consensus verification is the only
  real validity check in the document and it is conditional on data this path
  does not have.
- §8.3a "Input amounts **asserted** without proof → refuse unless
  `--i-certify-amounts`." Does not fire either: nothing was asserted, so there is
  no assertion to refuse. The refusal is worded against tier 4 and does not
  cover tier ∅.
- §8.3b needs `gettxout` per input — reachable only in tier 1.
- §8.4 needs `nLockTime` only, so it passes; §8.5–§8.7 are unrelated.

**Scenario.** The operator produced and presented the transaction yesterday with
`mt` on the offline machine, signed it on the device, and today runs `mt engrave
signed.hex` — a different invocation, possibly a different day, plausibly a
different machine. The signer had a bug, or the operator grabbed the wrong file
from the transfer directory (the *unsigned*-then-hand-patched one, the one for
the other wallet, the one from the abandoned first attempt). Nothing in §8 can
tell. Twenty-one minutes per plate later they have steel that will never confirm,
and they will not find out until 2040, when the check that would have caught it
is even harder to run.

**Why it matters.** §8 opens with *"All are machine-checkable before a single
plate is cut."* On the most convenient path through the tool that sentence is
false for the only refusal that verifies anything cryptographic. Worse, the
checking is *weakest* exactly where `mt` knows least about the artifact —
`mt`-produced transactions get tiers and consensus verification, hand-supplied
ones get a structural glance.

Related, and part of the same defect: taproot script verification requires **all**
spent outputs, not just the one being verified, so "when prevouts are supplied"
is all-or-nothing for any transaction containing a taproot input. A 5-input
taproot spend where the operator supplies four previous transactions and certifies
the fifth cannot be script-verified **at all**, and §8.2 again degrades to
silence rather than to a refusal.

**Fix sketch (non-authoritative).** Make prevouts mandatory for engraving —
refuse to engrave any transaction whose inputs cannot all be resolved to a
(value, scriptPubKey) by tiers 1–4 — so §8.2 becomes unconditional. A refusal
that can be skipped by supplying less data is an invitation to supply less data.

---

### S-6 — four sections promise legend content the legend does not contain, and §4 has no room for it

**Severity: Critical.** §7, §6a, §6c, §6d vs. §5 and §4.

§5 fixes the legend at *"Five fields, 136 characters, 6 lines — measured"*, and
`RESULTS_legend_budget_2026-08-22.txt` shows 35 chars/line, so the 41-character
bearer warning already wraps and those 6 lines are **exactly full**. §4 then
reserves precisely that 25.5 mm and the measurement file shows the consequence: at
6 lines, every configuration above v13 already *"NEEDS A SECOND PLATE"*. There is
no slack for a seventh line at any price short of another plate.

Against that, four other sections state as fact that the legend carries something
it does not:

| section | what it promises | in §5's five fields? |
| --- | --- | --- |
| §7, silent-invalidation row | *"legend carries the input outpoints so a holder can check they are still unspent"* | **no — §5 cut "input outpoints"** |
| §6c, limit 1 | *"it is disclosed the same way — the outpoints go on the plate"* | no |
| §6d, closing paragraph | *"the input outpoints go on the plate so a holder can check"* | no |
| §7, pinned-fee row | *"legend states rate and date so staleness is visible"* | **no — §5 cut "fee rate and date"** |
| §6a, tier 3 | *"caveat stated on screen and **in the legend**"* | no such field |
| §6a, tier 4 | *"`--i-certify-amounts` overrides and **the legend records it**"* | no such field |

**Scenario.** The operator reads §7 — the threat model, the section whose entire
purpose is telling them what protects them — and concludes that a plate whose
inputs get spent will be diagnosable from the plate itself, and that fee
staleness will be visible on the plate. Neither is true of the artifact §5
specifies. In 2040 the holder has: a bearer warning, an 8-hex wallet stub, a
block height, one truncated destination, and a plate index. To learn whether the
inputs are still unspent they must first successfully decode the QR — the
scenario in which they did *not* need the legend. To learn whether the fee is
survivable they need the prevouts, which are the thing the plate does not carry.

Separately, §5's own justification for the fee/date cut is **false as written**:
the drop table gives the recovery route for "fee rate and date" as *"inputs −
outputs, once prevouts are known"*. That yields the fee. It does not yield the
**date**, which is not derivable from the transaction by any means — a Bitcoin
transaction contains no creation timestamp — and the date is what tells a 2040
holder how stale the fee is and which of two plates from one wallet is the newer.
§5's cutting principle, *"everything derivable from the decoded transaction is
duplication"*, therefore does not cover one of the four things it was used to
cut.

**Why it matters.** This is not a cross-reference tidy-up. The mitigation column
of the threat-model table is the spec's safety guarantee, and it is false for two
of its four rows; §6a's mitigation for the tier it calls the most dangerous thing
in the spec is unrenderable; and restoring any of it costs a plate under §4's
measured numbers, so the fix is a design decision, not an edit.

**Fix sketch (non-authoritative).** Three options, all with a price, and the
spec should pick explicitly rather than leave the contradiction: (a) accept the
7th/8th line and re-run §4's selection with the larger reservation; (b) delete
the promises from §7/§6c/§6d/§6a and state plainly that the plate cannot
self-diagnose invalidation or staleness — which makes §7's mitigation column
honestly empty for two hazards; (c) buy the space from §10.7's back-side
engraving, which is the only route that costs neither a plate nor the guarantee.
Note that the tier caveat (§6a) is the one item that is *not* derivable from the
transaction at all, so it has the strongest claim on a line.

---

### S-7 — `include_mempool false` gives the opposite of the behaviour §6b argues for

**Severity: Important.** §6b. **[depends on lens 3 for the exact RPC semantics.]**

§6b's stated reason for the flag value is: *"`include_mempool` is passed **false**
deliberately. The default is `true`, and mempool state is the wrong basis for an
artifact meant to sit in a drawer for years — an input that is unspent only until
someone else's transaction confirms is not a foundation for a backup."*

The hazard described in that sentence — an output that someone else's pending
transaction is about to consume — is exactly the one that `false` **stops `mt`
from seeing**. A chainstate-only view reports such an output as unspent, because
on-chain it still is. It is `include_mempool = true` that returns `null` for an
output already being spent in the mempool. `false` protects against a different
thing: treating an output *created* by an unconfirmed transaction as spendable.
Both are real, but the rationale as written names the first and the chosen value
handles only the second.

**Scenario.** The operator's own wallet broadcast a transaction 40 minutes ago
that spends UTXO `abc:0` — a routine payment they made before sitting down to
this task, or a partner's spend from a shared wallet. `mt` calls
`gettxout abc 0 false`, receives the value, marks the input **tier 1,
"authoritative", unspent: yes**, and cuts the plate. The pending transaction
confirms in the next block. The plate is dead before it cools, and §8.3b — the
refusal designed for precisely "this input is spent", with **no override**
because a node *"said so authoritatively"* — never fired.

**Why it matters.** Tier 1 is the top of the trust ladder and the only tier whose
"unspent?" column says **yes**. That yes is weaker than advertised in the exact
window where a careful operator is most active: building a recovery artifact on
the same day they used the wallet.

**Fix sketch (non-authoritative).** Call it twice and require agreement:
`false` catches the unconfirmed-parent case, `true` catches the
mempool-conflict case, and a disagreement is a refusal worth reporting by name
("this output is being spent by an unconfirmed transaction").

---

### S-8 — `gettxout`'s `null` conflates "spent" with "this node cannot see it", and the no-override rule pushes the operator down to tier 4

**Severity: Important.** §6b, §8.3b.

§6b asserts *"A spent or nonexistent output returns `null`, which is a clean,
unambiguous refusal"* and §8.3b makes it *"refuse, **no override**"*. `null` is
not unambiguous: it is also what a node returns when it has not yet built that
part of the UTXO set, or is building a different one.

**Scenario A — a syncing node.** The operator starts `bitcoind` on the offline
machine after months powered down. It is at 71% of initial block download; the
UI shows it running and the RPC answers. `mt` connects, calls `gettxout` on a
UTXO created two years ago in a block the node has not processed, and gets
`null`. §8.3b refuses, **with no override**, and reports the input as *"spent or
never existed"* — a statement that is both false and alarming. The realistic
next move by a tired operator at 11 p.m. is not "wait six hours"; it is to stop
the node and re-run with `--i-certify-amounts`, i.e. the hard refusal with no
escape hatch **routes the operator into the worst tier in the table**, where S-1
is waiting.

**Scenario B — the wrong chain.** The node is a signet or regtest instance left
over from testing, or a remote node the operator does not control. `gettxout`
answers confidently with the wrong chain's values, and tier 1 is labelled
*"yes, authoritative"* with no requirement anywhere that the node's chain, sync
state, or trustworthiness be established. Unlike tier 2, tier 1's authority is a
trusted third party's assertion; the table ranks it above the proof-of-work
anchored tier without saying so.

**Why it matters.** A refusal with no override must be *certain*, and this one is
not. Its false-positive mode degrades the operator to self-certification, which
is the failure the whole of §6a is built to prevent.

**Fix sketch (non-authoritative).** Before trusting `gettxout`, require
`getblockchaininfo` to report the expected `chain` and
`initialblockdownload == false`; treat `null` from a node failing either
condition as "cannot determine", which is a *different* outcome from §8.3b's
hard refusal and should fall back to tiers 2–3 rather than to tier 4.

---

### S-9 — tier 3 proves the previous transaction exists, not that the outpoint ever confirmed

**Severity: Important.** §6a (tier table, "The circularity is breakable without a node").

The tier table's columns are "amount trustworthy?" and "unspent?", and tier 3
reads *"yes — bound by txid"* / *"unknown"*. The txid binding proves the supplied
bytes hash to the outpoint's txid — that the *transaction* is the one referenced.
It proves nothing about whether that transaction was ever mined. The table has no
column for **existence**, so tier 3's real gap hides inside a cell that says
"unknown" about something else.

**Scenario.** The operator fee-bumped a transaction in 2026; the original was
replaced and the RBF replacement confirmed with a different txid. In 2029 they
build a recovery spend and pull "the funding transaction" out of their wallet's
own transaction log or a saved PSBT — the *replaced* one. Its bytes are real, its
txid is real, `output[vout]` has a real value, and `mt`'s check
(*"hash the supplied transaction, require its txid to equal the input's
`previous_output.txid`"*) passes cleanly. Tier 3 is *"accepted"*. The signature
is computed over that amount and verifies locally; §8.2 verifies scripts against
that prevout. The plate is cut. The outpoint has never existed on any chain, and
the transaction is unbroadcastable from the moment the steel cools — not
"invalidated later" as §7's silent-invalidation row describes, but **dead at cut
time**, with every check green.

The same applies to a transaction that was signed and never broadcast, and to one
that lost a same-block race.

**Why it matters.** §7's silent-invalidation hazard is disclosed and accepted;
this one is not disclosed at all, and it is worse, because it is detectable at
cut time by tier 2 (`gettxoutproof`) and simply is not required.

**Fix sketch (non-authoritative).** Give the tier table an explicit **"existed
on-chain?"** column: tier 1 yes, tier 2 yes (by proof-of-work), tier 3 **no**,
tier 4 no. That is what actually distinguishes tiers 2 and 3, and it makes the
gap visible instead of leaving it inside a cell about unspentness.

---

### S-10 — `--i-certify-amounts` has no stated scope, and a mixed-tier transaction is ungoverned

**Severity: Important.** §6a, §8.3a.

The spec never says whether `--i-certify-amounts` degrades **one input** or the
**whole transaction**, whether it must name the inputs it covers, or what happens
when different inputs arrive from different tiers. Nor does §6b's promise that
*"`mt` records which tier supplied each amount"* have anywhere to land: it is not
in the legend (§5), and the spec does not name the manifest field.

**Scenario.** A 5-input recovery spend: four inputs resolved from full previous
transactions (tier 3), one — an old exchange withdrawal whose previous
transaction the operator cannot find — typed by hand. §8.3a blocks, so the
operator passes `--i-certify-amounts`. It is a bare boolean, so it now suppresses
the refusal for **all five** inputs, including the one where they fat-fingered
the third previous transaction's `vout` and are reading `output[0]` (the
counterparty's change) instead of `output[1]`. That input is nominally tier 3 —
`mt` says "bound by txid" — but the binding is to the transaction, not to the
output index, and the index came from the operator.

And for taproot the damage does not stay in one input: `sha_amounts` covers
**every** input, so one wrong amount invalidates the signatures on all five. The
transaction's real trust level is the **minimum** over its inputs, and the tier
table is written per-input as if the tiers composed.

**Why it matters.** "Every flag that can be passed will eventually be passed",
and this one is a single global switch that turns off the spec's most important
refusal for inputs the operator never intended to certify. The per-input record
that would let anyone reconstruct what happened has no specified home.

**Fix sketch (non-authoritative).** Make the flag take the outpoints it covers
(`--i-certify-amounts abc:0,def:2`) so it cannot silently widen; state that a
transaction containing any taproot input inherits the lowest tier present, and
say where the per-input tier record is written.

---

### S-11 — the tier evidence is not carried into the PSBT `mt` presents, destroying the signer's independent check

**Severity: Important.** §1a, §6a.

§6a justifies its own tier structure by pointing at the wallet ecosystem's
response to the segwit fee-lying attack: *"This is exactly the distinction PSBT
draws between `non_witness_utxo` … and `witness_utxo` …, and it is why wallets
hardened toward the former."* Then §1a has `mt` **produce** a transaction and
**present** it to a signing device — and the spec never says what UTXO fields the
presented PSBT carries.

**Scenario.** The operator supplies full previous transactions (tier 3). `mt`
performs the txid binding locally, reads the amounts, and emits a PSBT carrying
only `witness_utxo` (amount + script) because that is smaller and the animated-QR
budget is tight — §4 measures the PSBT sizes, so size pressure is real. The
signing device receives bare asserted amounts. Its own hardening — the very
hardening §6a cites as the reason the tiers matter — is now inoperative, and the
device displays a fee derived from numbers it cannot check. The system has
**one** amount check (`mt`'s) where the operator believes it has two, and if
`mt` has a bug in its binding, or was handed the wrong previous transaction per
S-10, nothing downstream can catch it.

The reverse also matters: if `mt` emits `non_witness_utxo` for a tier-4
(certified) input, the device is shown evidence stronger than what `mt` actually
has.

**Why it matters.** The signer is the only independent verifier in the whole
pipeline. A tier system that stops at `mt`'s process boundary is a single point
of failure wearing four hats.

**Fix sketch (non-authoritative).** State that the presented PSBT must carry
`non_witness_utxo` for every input where `mt` holds the previous transaction —
so the device redoes the binding itself — and that tier-4 inputs are presented as
`witness_utxo` only, so the device's own hardening fires on them as designed.

---

### S-12 — §8.4 cannot be evaluated offline, and a time-based locktime prints as an absurd block number

**Severity: Important.** §8.4, §5 (locktime line), §6a ("usable offline … the constellation's whole posture").

§8 claims *"All are machine-checkable before a single plate is cut."* §8.4's
predicate — "broadcastable today" — requires the **current chain tip**, which is
exactly what the offline posture §6a defends does not have. The spec does not say
where the height comes from when `bitcoind` is unreachable: operator-typed, cached
from a previous run, or refused.

**Scenario A — a stale tip.** The operator types the height from memory, or `mt`
reuses a cached value from the last time it saw a node, and the figure is 30,000
blocks stale. A locktime that is already *in the past* passes §8.4's "future
locktime" test. The plate is cut and engraved `SPENDABLE AFTER BLOCK 1383520`
for a block that passed seven months ago. The bearer mitigation is not merely
absent — the legend asserts a protection that has already expired, which is worse
than saying nothing, because it is what a cautious operator will rely on when
deciding where to store the plate.

**Scenario B — a time-based locktime.** `nLockTime` values ≥ 500,000,000 are unix
timestamps, not heights, and `mt` also engraves transactions built elsewhere. A
signer that emits `nLockTime = 2208988800` yields the legend line
`SPENDABLE AFTER BLOCK 2208988800` — a height ~40,000 years out. The 2040 holder
reads it as "this plate is dead", and the plate is in fact live. The measured
budget is `SPENDABLE AFTER BLOCK <n>` at **29 characters** (7 digits), so a
10-digit value also overflows the line the measurement sized.

Additionally, a time-based locktime is compared against **median-time-past**, not
wall-clock — the same MTP §6d discusses carefully for a different purpose — so
"broadcastable today" needs the MTP, not the local clock.

**Why it matters.** The locktime is the only bearer mitigation in the document
(see also S-3), and both its enforcement check and its human-readable rendering
have modes in which they state the opposite of the truth.

**Fix sketch (non-authoritative).** Refuse to engrave when the tip is not
independently known, or require an explicit `--assume-height` that is printed in
the refusal-check output; branch the legend line on the height/time threshold and
render a time-based locktime as a date, not as a block number.

---

### S-13 — nothing checks whether the transaction will *relay*: dust and minimum feerate are machine-checkable and absent

**Severity: Important.** §8, §7 (pinned-fee row).

Consensus validity and relayability are different things, and only the second one
gets a transaction broadcast. §8.2 gives consensus script validity; nothing in
§8 touches standardness. Two cases are checkable at cut time in one line each:

**Scenario A — dust.** The recovery transaction has a small second output (a
tiny payment, a marker, a rounding remnant) of 400 sat to a P2WPKH address.
Below the dust threshold, this is non-standard: no default node will relay it and
no default miner template will include it. Every check in §8 passes, the plate is
cut, and in 2040 the holder gets an inexplicable rejection from every node they
try, with no way to change the transaction because it is signed.

**Scenario B — feerate.** The transaction is built at 1.2 sat/vB, a sensible 2026
choice for something that will not be broadcast for a decade. If the network's
minimum relay fee rises — a policy parameter that has moved before — the plate
becomes unrelayable everywhere, permanently, and §9 excludes CPFP so `mt` offers
no remedy. §7's mitigation for this hazard is *"legend states rate and date so
staleness is visible"*, which per S-6 is not a legend field at all; and even if
it were, visibility does not help a holder who cannot alter a signed transaction.
The genuine mitigation is to **deliberately overpay at cut time** — the fee is
the premium on a decade-long option — and the spec never says so, because §9
excludes fee policy from `mt`'s remit while the artifact's entire viability
depends on it.

**Why it matters.** "Cut a plate that cannot be broadcast when they need it" is
half of the question this lens exists to answer, and relay policy is the most
likely route to it after S-3 and S-9.

**Fix sketch (non-authoritative).** Add a §8 refusal for any output below the
dust threshold for its script type, and a *warning with an acknowledgement* (not
a silent pass) when the effective feerate is below some floor — plus one sentence
in §7 saying the fee is the option premium and should be paid generously,
which is guidance, not fee estimation, and so does not breach §9.

---

### S-14 — the legend describes one output of N, and change back to the source wallet is unflagged

**Severity: Important.** §5 (`TO` line), §7 (pinned-destination row), §6.

The legend has exactly one destination line, `TO <truncated addr>  <amount>`.
Transactions routinely have two or more outputs, and the spec does not say which
one is shown or what happens to the others.

**Scenario A — the invisible majority.** A 10 BTC input, 0.5 BTC to the recovery
address, 9.49 BTC elsewhere. The legend shows one line. A 2040 holder reads
`TO bc1p8rrz...s6n0vcl  0.50000 BTC` and broadcasts believing that is what the
plate does. §6's argument for showing destinations at all is *"so a human sees
where the money goes without a scanner"*; showing one of two outputs shows a
human where 5% of the money goes and implies it is all of it.

**Scenario B — change to the wallet you cannot reach.** This is the sharp one.
The premise of an `mt` plate is that in 2040 the operator may not have working
access to the source wallet — that is why the spend is pre-signed. If the
transaction carries a **change output back to the source wallet**, which is the
default shape of every transaction any wallet software builds, then broadcasting
the plate in 2040 moves the change into a wallet nobody can spend from. The
recovery succeeds for the destination amount and **permanently destroys the
change**. Nothing in §6a, §8, or §7 detects, refuses, or displays this, and
`mt` is in a position to see it: whenever prevouts are known it can compare each
output's scriptPubKey against the input scriptPubKeys, and open question 4
already contemplates having the source wallet's md1 card at encode time.

**Why it matters.** §7's pinned-destination row worries about *"a 2040 recoverer
pays a 2026 address whose keys may be lost"* for the destination and misses the
one output that is *guaranteed* to point at a wallet whose accessibility is in
doubt — the change, whose address belongs to the very wallet the plate exists to
escape.

**Fix sketch (non-authoritative).** Refuse (overridably) any output paying a
scriptPubKey that matches an input's scriptPubKey or derives from the supplied
md1 descriptor, with the message naming the amount at risk; and specify the
legend for `n > 1` outputs — a `TO … +k MORE` form, or a total-out figure, so the
line cannot imply completeness it does not have.

---

### S-15 — the legend never says what the artifact is

**Severity: Important.** §5.

The five fields are: bearer warning, `FROM WALLET <8 hex>`, `SPENDABLE AFTER
BLOCK <n>`, `TO <addr> <amount>`, `PLATE n OF m`. The words *Bitcoin*,
*transaction*, `mt`, and any format or version marker appear nowhere. The QR
decodes to `ur:bytes/…`, and `bytes` is by construction the UR type that says
nothing about its contents.

**Scenario.** 2040. The person holding the plate is an heir, an executor, or a
locksmith's client — the brief's "someone else". They have a steel plate that
says money is involved and that anyone holding it can spend it, a QR that yields
an opaque hex blob, and no indication of what software turns one into the other.
They do not know it is a Bitcoin transaction rather than a key, a share, a
descriptor, or one of this constellation's other four formats — all of which are
also engraved as steel plates with legends, and one of which (`ms1`) is a secret
that must **not** be broadcast anywhere. The most likely outcomes are inaction
(the money is never recovered) or pasting a blob into the wrong tool.

**Why it matters.** §5's cutting principle is *"everything derivable from the
decoded transaction is duplication"*, and for this one fact the principle is
circular: you must already know the artifact is a serialised Bitcoin transaction
in order to decode it as one. It is therefore the field with the strongest claim
to steel, and it is the one that was never considered. It is also cheap —
`BITCOIN TX - SCAN ALL PLATES` is 28 characters — although per S-6 it is a
seventh line and costs a plate under §4's current numbers, which is precisely the
trade §10.7 exists to break.

**Fix sketch (non-authoritative).** One field naming the artifact type and the
decode action, and a format version so a 2040 tool can tell `mt` v0.1 plates from
whatever v0.4 does.

---

### S-16 — the threat model starts at the finished plate; the payload is bearer long before it is steel, and there is no revocation story

**Severity: Important.** §7, §8.7, §9.

§7 correctly concludes *"In hazard terms it sits nearer `ms1` than `md1`, and the
existing tooling's assumption that 'public string' means 'safe to engrave' does
not hold here."* It then draws no operational consequence: §8.7's only secret
handling is *"**Secrets** → refuse, as `me` already does for `ms1`"*, i.e. `mt`
inherits `me`'s rule about refusing to *engrave* a mnemonic, and nothing at all
about how the spendable transaction itself is handled before it reaches steel.
Three gaps, all with the same root:

**Scenario A — the payload's own lifecycle.** The signed transaction exists as a
file on the operator's machine, in the shell history that named that file, in
`mt`'s stdout, in the manifest §6c/§6d say the block anchor is *"available in"*,
on the presenting screen, and in whatever transport carries it to the engraver.
For `md1`/`mk1` every one of those is harmless; for `mt` every one is a bearer
instrument. A cloud-synced project directory, a backup, or a screen-shared
session is a complete theft. `me` refuses to push `ms1` over NFC precisely
because of this class of exposure, and `mt`'s payload gets no equivalent rule.

**Scenario B — photographed, not stolen.** The brief's case, and the worst one:
a plate photographed at a family gathering or by a contractor in the safe room
leaves the operator with **no signal at all**. Every other hazard in §7 is at
least discoverable (a missing plate is missing). This one is silent until the
locktime passes.

**Scenario C — no revocation.** Once the operator suspects a plate is copied,
their only defence is to **spend one of its inputs**, which permanently voids the
plate. That is the same mechanism §7's table lists purely as a hazard ("Silent
invalidation — one ordinary spend of any input voids the plate"). It is the
operator's only lever and the threat model never tells them it exists, nor that
they should keep the ability to exercise it (i.e. keep at least one input
spendable, which conflicts with the "my keys will be gone" premise and is worth
saying out loud). Note also that §8.4's `--allow-immediate` removes the sole
bearer mitigation entirely, and §7 has no row for the overridden case.

**Why it matters.** §7 is the section that decides how the operator stores and
handles the artifact, and it models exactly one adversary: someone who physically
takes a finished plate. The realistic adversaries are a copy, a backup, and a
camera.

**Fix sketch (non-authoritative).** A §7 row per exposure surface with the
handling rule for each; a statement that the payload gets `ms1`-class treatment
in transport, logs and files; and one paragraph naming input-spend as the
revocation mechanism, with its cost.

---

### S-17 — the wallet stub is form-dependent and the legend does not record which form

**Severity: Minor.** §5 ("The stub is a hint, never an authority").

The stub is *"form-aware — WalletPolicyId for a keyed wallet, the key-stable
WalletDescriptorTemplateId for a keyless template"*. The legend prints
`FROM WALLET fa568be0` with no indication of which. Two plates cut from the same
wallet in different forms carry **different** eight-hex values, so an operator
sorting plates in 2040 concludes they came from different wallets — or, worse,
concludes two genuinely different wallets are the same. Nothing branches on it,
so this is not a correctness defect; it degrades the one job §5 gives the stub,
*"to help a human find the right plates"*.

---

### S-18 — §6b requires no minimum confirmations and does not consider coinbase maturity

**Severity: Minor.** §6b.

`gettxout` answers about the current chainstate with no depth qualification. An
input confirmed one block ago passes; a two-block reorg then removes it and the
plate is dead, which is the same class of hazard as the mempool reasoning §6b
already cares about. An immature coinbase output likewise reports as unspent
while being unspendable until 100 blocks. Cheap to close: require a minimum depth
and refuse coinbase inputs below maturity relative to the engraved locktime.

---

### S-19 — only the stub has a conflict rule; the destination, amount and locktime lines have none

**Severity: Minor.** §5, §6.

§5 rules for the stub: *"If the legend says wallet X and the transaction spends
wallet Y's UTXOs, **the transaction wins.**"* No such rule exists for the `TO`,
amount or `SPENDABLE AFTER BLOCK` lines, which are the three a human actually
acts on — and §6 argues that a second source of truth about destinations is
*"a funds-safety hazard"* immediately before §5 engraves one. A holder in 2040
whose scanner shows a different address from the plate has no stated rule, and
S-2 gives a concrete way for the two to legitimately diverge. (The measured
truncation is prefix-and-suffix — `bc1p8rrz...s6n0vcl` — which is strong enough
that accidental confusion is unlikely; the missing rule is the defect, not the
truncation.)

---

### S-20 — `PLATE n OF m` is budgeted for single digits

**Severity: Minor.** §5, `RESULTS_legend_budget_2026-08-22.txt`.

The field is measured at 12 characters (`PLATE 1 OF 1`). `PLATE 10 OF 12` is 14,
which is still inside the 35-char line but changes the 136-character total the
legend budget is pinned to. §4's table already reaches 4 plates and §10.6 may add
more. Worth stating the field's width at its maximum rather than its minimum.

---

### S-21 — a re-cut multiplies bearer instruments and there is no inventory rule

**Severity: Minor.** §7, §9.

If a plate is damaged or a symbol misreads, the operator cuts a replacement — and
now two complete bearer artifacts exist for the same spend. Destroying the
suspect one is not obviously mandatory to a tired operator, and nothing in §7
says so. Same for a test cut of the real payload (§10.2's optical test plate
should be explicit that it must never carry the real transaction).

---

### S-22 — "SPENDABLE AFTER BLOCK n" boundary

**Severity: Nit.** §5.

A transaction with `nLockTime = n` first becomes includable in block `n + 1`; a
holder who broadcasts while the tip is exactly `n` gets a `non-final` rejection
and may conclude the plate is defective. Whether the engraved `<n>` is the raw
`nLockTime` or the first spendable height is unspecified. One sentence.

---

## §10.6 recommendation

**From a funds-safety standpoint the question is not "how much redundancy" but
"which failure am I buying against", and §4's current objective answers the wrong
one.**

Three observations first, one of which I do not think has been noticed.

**1. §4 spends every spare byte on ECC, and ECC does not protect against the
failure that actually destroys an `mt` transaction.** The rule reads *"Never
trade a plate for redundancy; never leave redundancy unbought"* — but "redundancy"
there means error-correction *within* a symbol. The dominant loss mode for a
steel plate over 14 years is the **whole plate**: fire, flood, a house move, a
tidy-up, a relative who did not know what it was, a safe deposit box that was
closed. ECC H recovers a scratched symbol and does exactly nothing for a plate
that is not in the drawer. Fountain parts are the only mechanism in the design
that addresses whole-plate loss, and they are currently the only thing the
objective refuses to buy.

**2. Redundancy is not free in the security direction, and this cuts against
naive "more is better".** With zero redundancy a multi-plate artifact is an
`m`-of-`m` scheme: a thief who obtains any proper subset gets nothing, which is
a genuine (if accidental) security property, and it is the reason geographic
splitting works today. Emitting `r` extra fountain parts turns it into
`seqLen`-of-`seqLen + r`: **every additional part increases the number of subsets
that reconstruct a spendable transaction.** For a bearer instrument that is a
real cost, not a rounding error. Redundancy trades theft-resistance for
loss-resistance, and the spec should say so rather than treating `r` as a pure
durability dial.

**3. Any `r > 0` breaks the legend as specified.** `PLATE n OF m` tells a holder
they need all `m`. With redundancy the truth is "any `k` of `m`", and a holder
who has 3 of 4 plates and reads "3 OF 4" will conclude the artifact is dead and
stop — destroying the transaction that the redundancy was bought to save. **If
§10.6 answers non-zero, the legend must carry `k`**, which per S-6 is another
line in a budget that has none. This makes the redundancy decision and the §5/§4
legend decision one decision, not two.

**Recommendation.**

- **`seqLen == 1`: emit no fountain parts. Cut a second identical plate instead.**
  This is the case §4's table shows for most artifacts. A fountain part and a
  duplicate plate cost the same — one plate — and the duplicate is strictly
  better: either plate alone decodes, so it survives losing *either* one, it needs
  no decoder cleverness, it needs no `k` on the legend, and the two can be stored
  in different places. There is no configuration in which a fountain part beats a
  duplicate at `seqLen == 1`.
- **`seqLen >= 2`: default `r = 1`, and make it a flag with the number printed in
  the refusal/summary output.** One extra part tolerates any one lost or
  unreadable plate, which is the single most likely non-catastrophic loss, at the
  cost §3 states plainly (*"one more symbol … which, at §4's sizes, is frequently
  one more plate"*). Going beyond `r = 1` buys steeply less per plate while
  continuing to widen the thief's subset count, so `r > 1` should be operator-
  chosen, never default.
- **Make the default conditional on storage, and ask.** `r = 1` buys nothing
  against the failure that takes all plates at once (one fire, one safe). If the
  plates will live together, the honest advice is `r = 0` plus a second complete
  set stored elsewhere; if they will be split, `r = 1` is what makes the split
  survivable. This is an operator input `mt` can simply ask for, and it changes
  the right answer.
- **Do not ship any `r > 0` until the decoder has actually been shown to
  reconstruct from a deterministic `seqLen`-subset of `seqLen + r` parts** —
  §10.3 flags that the vendored `Decoder`'s behaviour here is assumed, not
  confirmed, and §3's own CORRECTION block records that the last confident claim
  about fountain behaviour was false. An `r` that does not decode is worse than
  `r = 0`: the operator has paid a plate for a protection that does not exist and
  will store the plates accordingly.
- **State the loss explicitly either way.** Whatever `r` is chosen, one sentence
  in §7's hazard table — *"unreadable or lost plate — with r = 0, the transaction
  is unrecoverable"* — because it is currently the largest funds-safety hazard in
  the design that the threat model does not list at all.

One framing that may help the decision: the plate is only load-bearing when the
keys are *not* available. If the operator will still hold the keys in 2040, an
unreadable plate costs convenience. If the plate exists because the keys will be
gone — inheritance, dead-man's switch, deteriorating access — then an unreadable
plate is **total, permanent loss of those coins**, and `r = 1` is cheap at one
plate. The spec should ask which of the two the operator is building, because the
answer changes `r`, and it is the only input to §10.6 that matters.
