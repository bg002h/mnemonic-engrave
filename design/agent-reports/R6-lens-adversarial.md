# R6 — adversarial lens: can `mt` produce a confidently WRONG answer about money?

Target: `design/SPEC_mt_v0_1.md` (3,082 lines, read in full).
Lens: **one question only** — is there a path where `mt` reports success, or reports
a fact, and that report is wrong? Refusals and stated `UNKNOWN`s are treated as safe.
Read-only; the spec was not edited.

**Count: 4 Critical, 8 Important, 6 Minor.**

Two of the Criticals (C-1, C-2) are the same root fact seen from two sides: **the
bytes `mt` engraves are the witness-bearing network serialization, and the txid is
not a hash of them.** I verified that against the spec's own measurement rather than
from memory — §3b line 1077 measures the `tr` key-path 1-in/1-out artifact at
**162 bytes**, which is exactly the network serialization
(`4 + 2 + 1 + 41 + 1 + 43 + 66 + 4 = 162`); the txid preimage for the same
transaction is **94 bytes**. The payload is witness-bearing, confirmed by the
spec's own number.

---

## CRITICAL

### C-1 — `TX` is specified as the wtxid and labelled the txid

**Severity: Critical** (a stated fact that is wrong, printed to a recoverer as the
thing to look up, and a wire-format fork risk).

**Lines: 492**, and **1554**.

> 492: `` | `TX` | **always** | double-SHA-256 of the decoded bytes; needs no node and no network (§6a) | ``
>
> 1554: *"the txid is the double-SHA-256 of the very bytes `decode` emits"*

**The scenario.** The bytes `decode` emits are the raw signed transaction in network
serialization — that is what `sendrawtransaction` accepts (§1.1a, line 549) and what
§3b measures. For any segwit input that serialization carries the marker, flag and
witnesses. Double-SHA-256 of those bytes is the **wtxid** (BIP-141). The **txid** is
double-SHA-256 of the same transaction with marker, flag and witness stripped. For
the spec's own smallest artifact those are hashes over 162 bytes and 94 bytes
respectively — different preimages, different values. Every artifact measured in §3b
is `tr` or `wsh`, i.e. segwit, so this is the normal case and not an edge.

Concrete wrong output: a recoverer runs `mt inspect` offline, gets the report at
line 480, and follows §6a's own instruction at lines 1538-1539 — *"look this txid up
in any block explorer"* — pasting a **wtxid**. The explorer returns nothing. The
recoverer concludes the transaction was never broadcast (a PENDING-shaped
conclusion) when it may have confirmed years ago, and broadcasts a duplicate or
files a live plate as dead.

Second consequence, worse and silent: §10.13(a2) line 2816 derives `chunk_set_id`
from *"the extracted txid"*. An implementer who follows the §1.1 row — which line
471 declares **normative and "the only place the layout appears"** — computes the
set id from the wtxid instead. Plates from the two implementations then carry
different set ids for the same transaction and are mutually unreadable. This is
byte-for-byte the defect class the spec already caught twice (HRP `"mt"` vs `"mt1"`,
lines 2799-2806; `count` vs `count − 1`, line 2820).

**Minimal fix.** In the line-492 row and at line 1554, say **txid = double-SHA-256
of the transaction serialized WITHOUT marker, flag and witness (BIP-141)**, and add
one sentence naming the distinction explicitly, because "the bytes we emit" is the
natural and wrong reading. See C-2 for why the wtxid should nonetheless appear —
as the content id, not as `TX`.

---

### C-2 — the "funds-load-bearing invariant" is blind to ~92% of the engraved payload

**Severity: Critical** (`mt verify` prints OK on a transaction that cannot confirm;
a recoverable-looking plate is certified and is scrap).

**Lines: 2816** (`chunk_set_id` = top 20 bits of the extracted **txid**), **2852**,
**2857-2864**; the claims it falsifies are at **358**, **537** and **2872-2875**.

> 358: *"**The content id is the only thing that can** [detect miscorrection] — which is what makes it the funds-load-bearing invariant"*
>
> 537: *"**prove the result is the right transaction** | re-derive the content id from the decoded transaction and compare"*
>
> 2872: *"A collision cannot yield a wrong transaction, because reassembly re-derives the id from what it decoded and a mismatch is caught."*

**The scenario.** The txid does not commit to witness data. For the spec's largest
measured artifact (`wsh` tier 1, 5-in/2-out, 3,538 B) the non-witness portion is
**301 bytes — 8.5%**. **91.5% of the engraved payload is witness data over which the
content id says nothing at all.**

So take the failure §1 spent lines 348-374 designing for: a chunk takes more than
`t = 4` symbol errors and BCH miscorrects it onto a different valid code word. If
the affected bytes lie in the witness — a 91.5% chance by byte position on that
artifact — then:

- every string parses ✔
- every BCH checksum holds ✔ (miscorrection produces a valid code word)
- the set is complete ✔
- every chunk carries the same `chunk_set_id` ✔
- **the reassembled transaction re-derives that id ✔ — with probability 1, because
  the txid never saw the bytes that changed**

`mt verify` prints `OK — 14 chunks, set 0x0e17e, transaction re-derives.` The plate
carries a transaction with a mangled signature that no node will accept. The operator
files it. In 2040 the recoverer broadcasts and the transaction is rejected.

**The brief asked whether miscorrection past the content id requires a hash
collision. It does not, on two independent grounds.**

1. **Witness-region damage: probability 1.** No collision, no grinding, no luck —
   the check structurally cannot see it.
2. **Non-witness-region damage: 1 in 2^20 ≈ 1 in 1,048,576.** The comparison is
   20 bits wide, not 256. That is a filter, not a proof, and lines 358/537/2872
   describe it as a proof.

**The adversarial version needs no luck at all.** The set id is *published*: §10.10
line 2715 has `mt encode` print `All 14 strings begin mt1qzrf8x` and §0a line 152
tells the recoverer to group by that prefix, so anyone who photographs **one** plate
learns the set id. An attacker who can insert one string into the recoverer's
typed-back file, or substitute one plate in a drawer, mints a chunk with the correct
version/`chunked`/set id/`count`/`index` whose payload lands in the witness region.
Every check above passes. `verify` says OK. This defeats line 535's stated property
(*"reject chunks from a different transaction | the 20-bit `chunk_set_id`"*) and
line 831's *"symbols from two different transactions **cannot** be combined"* at
zero computational cost.

**Bound on the harm, stated so the severity is not overstated:** the attacker cannot
*steal* — every accepted input is `SIGHASH_ALL`/`SIGHASH_DEFAULT` (§8.6), so
rewriting an output invalidates the signature and the transaction cannot confirm.
The reachable outcomes are (a) a plate certified OK that is dead, and (b) a
confidently wrong `inspect` report about destination and amount. (a) is
"a recoverable plate discarded / an unmet guarantee" and is Critical on its own.

**Minimal fix, and it is one line.** Derive the content id from the **wtxid** — the
double-SHA-256 of the bytes actually engraved — instead of the txid. Everything §10.13(c)
argues for the txid (lines 2847-2850: *"the thing actually engraved, actually
broadcast, and actually re-derivable by a recoverer who has decoded the plate and
holds nothing else"*) is **more** true of the wtxid, and the spec has already written
that formula down at line 492 — it merely filed it under the wrong name. This makes
the invariant cover 100% of the payload instead of 8.5%, and it composes with C-1:
`TX` row = txid (BIP-141 stripped), content id = wtxid. Separately, state plainly at
lines 358/537 that the comparison is **20 bits** and is therefore detection, not
proof.

---

### C-3 — DEAD is reported when the parent is merely unconfirmed

**Severity: Critical** (a live plate is declared scrap — the spec's own named worst
error, line 448).

**Lines: 437**, against **442-443**.

> 437: `` | **DEAD** | `null`, **and** `getrawtransaction` finds the parent | the input was spent by someone else. **The plate is scrap** | ``
>
> 442-443: *"`getrawtransaction` **"only returns a transaction if it is in the mempool. If `-txindex` is enabled"** it resolves any confirmed transaction."*

**The scenario.** The spec quotes, three lines below the table, the exact sentence
that falsifies the table. `getrawtransaction` finding a transaction proves it exists
**in the mempool or in a block** — it does not prove it confirmed.

Chain state: the plate's transaction spends an output of parent P. P has been
broadcast and is sitting **unconfirmed in the mempool** (spending unconfirmed change,
or a CPFP chain, or simply a congested hour). Then:

- `gettxout <P> <vout> false` → **null**. `include_mempool` is `false` by ruling
  (line 1599), so an unconfirmed parent's output is not in the UTXO set.
- `getrawtransaction <P>` → **found**. It is in the mempool. This holds **with or
  without `-txindex`** — without the index, the mempool is precisely the one place
  it does look.

Table verdict: `null` + parent found = **DEAD — "the input was spent by someone
else. The plate is scrap."**

The truth is exactly the PENDING row's own words at line 438 — *"the parent
transaction was never confirmed. The plate may still become live"*. The plate is
perfectly good and becomes spendable the moment P confirms. `mt` tells the recoverer
their money was taken by a third party and their steel is worthless. Line 448-450
names this the worst error available: *"Telling a recoverer their plate is scrap
when it is merely early is the worst error available here, because it is the one
that gets a live plate thrown away."* The rule as written produces it.

**The same missing distinction also fires at §8.5** (line 2174): *"`gettxout`
returns `null` for any input → refuse... The output is spent or never existed."*
With an unconfirmed parent that is a **false statement of fact** in a refusal
message — the output exists and is unspent.

**Minimal fix.** DEAD requires the parent to be **confirmed**, not merely found:
call `getrawtransaction <parent> true` and require `blockhash` / `confirmations ≥ 1`.
A parent found only in the mempool is **PENDING**. Amend §8.5's refusal text the
same way, or downgrade it to a warning for the mempool case.

---

### C-4 — the BEARER "cannot redirect" guarantee does not hold, and it chose the permanent wording

**Severity: Critical** (a stated guarantee that does not hold, used to pick text cut
into steel).

**Lines: 1308-1312**, against the spec's own admission at **2230-2237** and **1671**.

> 1308-1312: *"§8.6 refuses any input whose satisfaction does not bind the outputs, so a holder **cannot redirect the money**: the destination is fixed by signatures they cannot alter."*
>
> 2233-2237: *"A crafted witness carrying a signature-shaped element that the script never checks would pass. This is a structural heuristic, not a proof, and the spec should not claim more than that."*

**The scenario.** §8.6(b) requires *"every input must carry at least one signature,
and every signature must be (a)-clean"*, and §8.2's removal means `mt` recognises a
signature only by **shape**. A 64-byte witness element is read as a Schnorr signature
with implicit `SIGHASH_DEFAULT`; a DER-shaped element ending in `0x01` is read as
ECDSA `SIGHASH_ALL`.

Take a taproot script-path leaf whose satisfaction is a **64-byte hash preimage** —
e.g. `OP_SHA256 <H> OP_EQUALVERIFY OP_TRUE`, spent at depth 1. The witness is
`[preimage(64), script, control_block(65)]`. §8.6's shape rule (line 2224) strips the
last two elements as control block and leaf script and *"counts signatures only among
the remaining elements"* — leaving the 64-byte preimage, which is counted as a
`SIGHASH_DEFAULT` signature. `mt` accepts. The satisfaction commits to **nothing**:
any holder of the plate re-satisfies the same leaf with entirely different outputs
and keeps the money. That is strictly the failure §8.6(b) was added to close (line
2193: *"any holder can rewrite every output and re-satisfy it"*), reached through the
recognizer rather than around it.

The general form is the one the spec itself states at 2233: `mt` cannot tell that the
script *requires* the element it counted. Any PSBT producer — a co-signer, a service,
a compromised or exotic wallet — can pad a witness with a signature-shaped element
that no `OP_CHECKSIG` ever consumes.

**Why this is Critical rather than a known limitation.** The spec is honest about the
heuristic in two places (2233-2237, and §7's row at 1630 which says *"structurally,
since §8.2's removal left no script engine"*), and then asserts the unqualified
guarantee at 1310 — and **uses that guarantee to decide what is engraved
permanently**. Lines 1314-1319 rule out `SPEND` on the grounds that it *"overstates
the holder's power, implying theft that §8.6 exists to prevent"*. A holder reading
`BEARER - ANYONE HOLDING THIS CAN BROADCAST IT` therefore concludes — exactly as the
spec intends — that broadcasting is the worst that can happen to them. On any input
whose satisfaction §8.6 mis-recognised, that conclusion is false and the steel cannot
be corrected.

**Minimal fix.** Two edits, no design change. (1) At 1310, replace *"cannot redirect
the money"* with the conditional the spec already knows to be true: *"cannot redirect
the money **so long as each input's script actually requires the signature `mt`
recognised — which `mt` checks structurally, not by evaluation (§8.6)**"*, and drop
the *"implying theft that §8.6 exists to prevent"* clause at 1315. (2) Add to §7's
"Non-`ALL` sighash" row (1630) that the recognizer is grindable by a hostile PSBT
producer, so the hazard is recorded rather than mitigated — which is the posture §7
already takes for the last two rows and defends at 1634-1637.

---

## IMPORTANT

### I-1 — `verify --transaction` claims to "prove identity" and compares 20 bits

**Lines: 404-408.**

> *"because the content id **is** the txid (§10.13 c), comparing a supplied transaction against the set's id is a cryptographic round-trip rather than a structural comparison. `md verify` can only re-encode and diff; `mt verify` can prove identity."*

**Scenario.** The operator runs `mt verify --transaction wrong.psbt < plates.txt` to
confirm the plate holds the transaction they think it does. As written, `mt` compares
the supplied transaction against **the set's id** — 20 bits. Any supplied transaction
whose txid shares those 20 bits (1 in 1,048,576 by accident; 2^20 double-SHA-256 ops,
i.e. under a second, to construct) is reported as a **match**. `mt verify` says the
plate holds a transaction it does not hold, and says it in the words *"can prove
identity"*.

Second defect in the same clause: the flag accepts `<psbt|hex>`, and §10.13(c)
line 2841 establishes that a PSBT holds **two** transactions whose txids differ for
every legacy and `sh(wsh(…))` input. The comparison basis is unstated, so
`--transaction` can report a **mismatch** on the correct transaction.

**Minimal fix.** Compare the full 32-byte txid (or the decoded bytes) — `mt` holds
the whole reassembled transaction, so nothing forces a 20-bit compare — and state
that a supplied PSBT is compared against its **extracted** transaction, per (c).
Delete *"can prove identity"* or make it accurate.

### I-2 — `decode`'s failure behaviour is unspecified, and the documented pipeline cannot see a failure

**Lines: 531-537** (what `decode` must do) and **557** (the pipeline).

> 557: `mt decode < plates.txt | xargs bitcoin-cli sendrawtransaction`

**Scenario.** §1.1a lists five things `decode` must do, ending with *"prove the
result is the right transaction"* — and never says what `decode` **does when that
check fails**. The only specified failure output for a content-id mismatch belongs to
`verify` (lines 362-374). So an implementer may reasonably print the hex anyway with
a warning on stderr, which is consistent with §8.2e's posture of never refusing bytes
(line 1874: *"`mt` never refuses the bytes"*).

Now the spec's own flagship one-liner runs. `xargs` consumes **stdout only** and is
blind to both the exit code and stderr. The recoverer broadcasts a transaction that
failed `mt`'s own integrity check. Same hole for a `STATUS DEAD` or a §8.5-class
result: nothing says stdout is withheld.

This directly contradicts the justification given for the stderr report at line 578:
*"no path through this tool broadcasts a transaction the operator was never shown"* —
the tool ships a documented path that does exactly that.

**Minimal fix.** State normatively: `decode` writes **nothing to stdout** unless
every check in §1.1a's table passes, and exits non-zero otherwise. Replace the
documented one-liner with one that respects failure, e.g.
`mt decode < plates.txt > tx.hex && bitcoin-cli sendrawtransaction "$(cat tx.hex)"`.

### I-3 — LIVE says "broadcast it" for an input already spent in the mempool

**Lines: 436**, against **1599-1608**.

**Scenario.** `include_mempool` is `false` by ruling. Someone has already broadcast a
conflicting spend of input 0, unconfirmed. `gettxout` queries the UTXO set only, so
it returns a **value**. The table's LIVE row fires; the report prints
`STATUS LIVE — every input is unspent` and the action column says **"broadcast it"**.
The recoverer broadcasts, loses the race, and has been told by `mt` that the input
was unspent.

The spec knows the mechanism — line 1604 records *"a mempool-spent input reads as
*unspent*, which is the opposite of the caution this section argues for"* — but it
records it in §6a's **encode** context, as an unresolved limitation, and the
four-state table added later carries no trace of it. LIVE's action column is
unqualified.

**Minimal fix.** Either query with `include_mempool=true` for the *liveness report*
(keeping `false` for §8.5's refusal, where the drawer-years argument at line 1600
actually applies), or qualify LIVE: *"unspent in the UTXO set; a conflicting spend
may already be in a mempool this node did not consult."*

### I-4 — the normative report prints a `PASSED` verdict that §8.4 forbids and that is false under `OP_CSV`

**Line: 484**, against **1971-1982** and **2155-2173**.

> 484: `LOCKTIME  block 1383520, ~FALL 2034   current height 1402887 — PASSED`
>
> 1971-1975: *"**`mt` states the two facts and stops.** … the `stderr` report is a statement of what was read, **not a verdict**"*

**Scenario.** §8.4 enumerates the five permitted spellings at lines 1978-1982; none
contains `PASSED`. §1.1 declares itself *"normative and the only place the layout
appears"* (line 471) and prints one. Worse, `PASSED` is a claim about **spendability**,
which §8.4 spends lines 2155-2173 establishing that `mt` cannot make: a BIP-68
relative timelock lives in `OP_CSV` inside the witness script, a relative-locked
spend has `nLockTime = 0`, and reading it *"means evaluating the sending wallet's
script, which is out of scope by ruling"*. This project's own RCW fixture has exactly
such a leaf — `OP_CSV` with 32,768 blocks, ~7 months.

Concrete wrong output: a transaction spending that RCW leaf, `nLockTime` set to a
height already reached. The report prints `— PASSED`, the STATUS row prints `LIVE`,
and the recoverer broadcasts a transaction that is non-final and will be rejected for
months. §8.4 calls this *"false reassurance … the worst failure available here"*
(line 2150) and closed it in §8.4's own text while §1.1 reopened it.

**Minimal fix.** Delete `— PASSED` from line 484 and bind §1.1's `LOCKTIME` row to
§8.4's five normative spellings by reference, so the two cannot drift.

### I-5 — a PSBT-sourced FEE is presented in the chain-verified column and nothing checks it

**Lines: 494**, **505-510**, against **1469-1473**.

> 505-508: *"**Read and verified are visually distinct, always.** `TX`, `OUT` and `LOCKTIME` come off the plate; `FEE`, the input values and `STATUS` come off the chain."*
>
> 494: `` | `FEE` | a node is reachable, **or the input was a PSBT carrying values** | ``

**Scenario.** Offline `mt encode`, air-gapped — the constellation's own posture
(line 1585). The PSBT carries `witness_utxo` for a segwit input claiming 1.0 BTC.
No node, so §6a's chain comparison does not run. Not legacy, so §8.2d's txid binding
does not apply and §8.2c's warning **does not fire** — it fires *"when, and only
when, the value is bound by nothing: no `non_witness_utxo`, no chain fetch"*
(line 1741), and the spec treats a segwit amount as bound by the signature
(line 1470). **But §8.2's removal means `mt` verifies no signature**, so that binding
is asserted and never computed — the same defect §8.2d was created to fix for legacy
(line 1810-1815).

Result: the report prints `FEE 0.00012000 BTC` in the column rule 2 tells the reader
is chain-verified, with no warning anywhere, on a number nothing checked. Rule 2's
two-way split is wrong: §10.10 line 2650 already enumerates **three** provenance
classes — *"chain-fetched (§6a), txid-bound (§8.2d), or operator-asserted (§8.2c)"* —
and the report collapses them to two.

(Honest bound: a wrong `witness_utxo` also makes the signature invalid, so the
transaction cannot confirm — §7's accepted hazard. The wrong *number* stands
regardless, and it is the number the operator uses to decide whether to spend
21 minutes a plate.)

**Minimal fix.** Rule 2 names three classes, and the `FEE` row carries the weakest
provenance of any input inline, e.g.
`FEE 0.00012000 BTC   (CLAIMED — no input value verified)`.

### I-6 — "or the total across all inputs" has no rule for mixing, so the fee can be off by a whole input

**Lines: 1729-1730.**

> *"Where a record is absent, `mt` requires the operator to supply that input's value — **or the total across all inputs** — since §8.2b cannot check the value balance without it."*

**Scenario.** Two inputs. Input 0 carries `non_witness_utxo` and is txid-bound at
1.0 BTC (§8.2d). Input 1 carries nothing. The operator, following the sentence
literally, supplies **the total: 2.0 BTC**. Outputs total 1.99 BTC.

- Reading A (the supplied total *is* the input sum): fee = 0.01 BTC.
- Reading B (the supplied total is *added* to the bound inputs): fee = 1.01 BTC.

Both are stated by the same sentence. One of them is a confidently wrong fee figure
off by an entire input, and `AbsurdFeeRate` (§8.2b, 25,000 sat/vB) fires or does not
fire depending on which an implementer chose. The same ambiguity decides whether
§8.2b's `inputs ≥ outputs` refusal triggers.

**Minimal fix.** Delete the "or the total" alternative — values are supplied
**per input**, which is what §8.2b, §8.2d, §6a's per-input comparison, and the report's
per-input `INPUTS` rows all already require. If the alternative is kept, state that a
supplied total replaces the sum of all inputs and refuse it when any input is
independently bound.

### I-7 — duplicate resolution is defined for exactly two candidates, and row 1 hands an inserted forgery a silent win

**Lines: 213-217.**

**Scenario A — three chunks at one index.** The table is written for *"two chunks,
same index"* and the ≥3 case is undefined. An attacker (or a drawer holding a
re-cut plate *and* its two predecessors) supplies three candidates at index 7:
`{genuine, forgeA, forgeA}`. A pairwise implementation finds `forgeA == forgeA`
(row 2: *"accept silently"*) and `genuine ≠ forgeA` (row 3: *"refuse loudly"*) — the
two rows return opposite verdicts on the same set, and any implementer resolving that
by majority vote hands the decision to whoever can add the most strings. Nothing in
the spec forbids it.

**Scenario B — row 1 is steerable without any BCH work.** Row 1 reads *"one passes
BCH, one fails → **use the good one, and say so.** No operator decision is needed;
`mt` has proof"*. `mt` has proof that one **checksum** holds — not that the passing
chunk is the one that was engraved. The genuine string fails BCH whenever the
operator mistyped more than `t = 4` characters in a 90-character string, which is an
ordinary event (it is the entire reason §1's margin report exists). Combined with
C-2's published set prefix, an attacker who adds one string wins that index
**silently** — row 1 explicitly promises no operator decision and the log line is the
only trace.

**Minimal fix.** (1) State the rule over **n candidates**, not two: partition by
BCH-validity, then by exact bytes; accept only if exactly one distinct valid byte
string exists; refuse otherwise. Explicitly forbid majority vote. (2) Make row 1
**announce and require acknowledgement rather than proceed silently** when the
discarded candidate is not byte-identical to a peer, and — with C-2's fix in place —
note that the set's payload digest is what actually adjudicates it.

### I-8 — DEAD's explanation is printed for a transaction that already confirmed

**Line: 437.**

**Scenario.** The plate's own transaction was broadcast in 2029 and confirmed. In
2040 the recoverer runs `mt inspect`. `gettxout` on each input returns **null** (they
were spent — by this very transaction), and `getrawtransaction` finds each parent
(confirmed). The table fires **DEAD**, and prints *"the input was spent by someone
else. **The plate is scrap**"*.

Both halves are wrong. The inputs were spent by **this** transaction; the payment
succeeded. The recoverer is told their funds were taken by a third party. A plausible
next action is to go hunting for a theft that never happened, or to treat a completed
payment as a loss.

`mt` can answer this exactly and at zero cost — §6a line 1553 already establishes
that it can compute the transaction's own id with no node — yet §6a's offline warning
lists *"was this transaction already broadcast? UNKNOWN"* (line 1527) as a question
worth asking and the four-state table never asks it when a node **is** present.

**Minimal fix.** Add a fifth state, checked **first**: `getrawtransaction <this
txid>` → if found and confirmed, report **CONFIRMED — this transaction is already on
chain at block N**. Only if that misses do the four existing states apply.

---

## MINOR

- **M-1 — `verify` does not check that all chunks agree on `count`.** Line 193
  enumerates the checks: parse, BCH, completeness, same `chunk_set_id`, id
  re-derivation. Two chunks with the same set id and *different* `count` fields leave
  "how many chunks am I expecting" undefined. Fix: add `count` agreement to the
  enumerated list.

- **M-2 — the last chunk's expected length is not derivable at decode time.** §1e
  line 633 makes the length check *"the one damage class BCH cannot"* catch, and
  line 654 has `mt` compute *"both lengths"*. At encode time it knows both. At
  decode/verify time nothing carries the total payload length, so
  `last_len = total − (count−1)·bpc` is circular and the last chunk has no expected
  length to check against. Damage there fails BCH (a refusal, so safe) but is
  diagnosed as *"damaged beyond correction"* rather than as the missing character it
  is. Fix: say so, or note that the last chunk's length is unchecked.

- **M-3 — the set-prefix rule `mt` prints is stated as an absolute.** Lines 152 and
  2715: *"Strings sharing that prefix belong to this transaction; strings that do
  not, do not."* The first clause is a 20-bit, publicly visible, forgeable claim, and
  §1 line 391-396 already acknowledges the colliding-set case. Fix: *"Strings that do
  not share it are certainly not part of this set; strings that do share it are
  probably part of it — `mt verify` decides."*

- **M-4 — §10.10 still concludes PSBT-only, and §8.2c/§8.2e disagree on raw+no-node.**
  Lines 2601-2607 end *"`mt` therefore requires a PSBT"*, superseded by §8.2e
  (line 1875) and by §10.10's own table at 2583. Separately §8.2c (1729)
  **requires** the operator to supply values when records are absent, while §8.2e's
  warning box (1869) proceeds with *"The fee is UNKNOWN"* — so on the default Core
  workflow (`finalizepsbt` returns hex by default, line 1835) with no node, §8.2b's
  `inputs ≥ outputs` and `AbsurdFeeRate` refusals either run or do not depending on
  which section is read. Fix: delete the stale prose block and state one rule.

- **M-5 — the BIP-341 annex is unmodelled in §8.6's shape recognizer.** Line 2224
  fixes the shape as *"last element is the control block, second-last the leaf
  script"*. A witness carrying an annex (last element, first byte `0x50`) shifts both,
  so the recognizer counts the wrong elements. Fix: strip a trailing `0x50`-prefixed
  element before applying the shape rule, and note that a 2-element taproot witness is
  key-path-plus-annex or script-path-with-empty-stack, discriminated by `0x50` vs
  `0xc0/0xc1`.

- **M-6 — `STATUS` aggregation over mixed per-input states is unspecified.** Line 497
  makes `STATUS` always present and line 487 shows `LIVE — every input is unspent`,
  but nothing says what a set of `{LIVE, PENDING, UNKNOWN}` inputs prints. Fix: state
  the precedence (any DEAD → DEAD; else any UNKNOWN → UNKNOWN; else any PENDING →
  PENDING; else LIVE).

---

## What I checked and did NOT find

Stated plainly so a later reader knows these were examined rather than skipped.

- **Funds theft via miscorrection or chunk substitution: not reachable.** Every
  accepted input is `SIGHASH_ALL`/`SIGHASH_DEFAULT`, so any change to an output
  invalidates the signature. The reachable damage is a dead plate reported as OK
  (C-2), not a redirected payment. C-4 is the one path to redirection, and it goes
  through §8.6's recognizer rather than through the codec.
- **20-bit `chunk_set_id` collisions between unrelated transactions: not a
  wrong-answer path on its own.** Two colliding sets in one drawer produce duplicate
  indices with differing bytes → §1's row 3 refuses loudly. The operator's stated
  scale (*"at most a few dozen"*) gives a birthday probability under 10^-3, and the
  spec's decision to keep 20 bits is sound. What is **not** sound is the reasoning
  offered for it (lines 2872-2875) — that is C-2/I-1, not a width problem.
- **`mt decode`'s stdout/stderr split itself: sound.** stdout carrying only hex is
  correct and the stderr report is a genuine improvement. The defect is the missing
  failure rule, not the split (I-2).
- **The offline report's `UNKNOWN` discipline (rule 1, line 501): sound.** Rows read
  `UNKNOWN` rather than vanishing, and §6a's recovery warning enumerates them with
  two resolution paths. The read/verified *separation* is the part that does not hold
  (I-5), not the `UNKNOWN` handling.
- **§1e's autocorrect ordering (lines 662-691): sound.** "Try the string as written
  first" is correctly identified as a safety rule, and the positional `1`-at-index-2
  argument is right.
- **§8.4's `nSequence` rule and the `LOCK_TIME_THRESHOLD` branch: sound**, and the
  over-warning direction on `OP_CSV` is the safe one. The defect is §1.1 reintroducing
  a verdict §8.4 had removed (I-4).
