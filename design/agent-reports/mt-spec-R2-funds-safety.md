# R2 — adversarial funds-safety review of `design/SPEC_mt_v0_1.md`

Artifact: `design/SPEC_mt_v0_1.md` at `b1790a4` (HEAD, tree clean, 1357 lines).
Lens: **the one question** — following this spec exactly, in what sequence of
realistic events does an operator lose money, or cut a plate that cannot be
broadcast when they need it?

Operator rulings are treated as decisions, not defects. Every finding below is
either (i) a place where the spec *implements* a ruling unsafely, or (ii) a
consequence of a ruling the spec fails to disclose, or (iii) a claim the spec
makes about its own safety that its own text falsifies.

**Machine-checked before writing** (so a reader does not have to re-derive them):

| claim | how checked | result |
| --- | --- | --- |
| `LOCK_TIME_THRESHOLD = 500_000_000` | `bitcoin-units-0.1.101/src/locktime/absolute.rs:27` | confirmed; "values _below_ the threshold are interpreted as block heights, values _above_ (or equal to) the threshold are interpreted as block times" (`:18-19`) |
| `DEFAULT_MAX_FEE_RATE = 25,000 sat/vB` | `bitcoin-0.32.101/src/psbt/mod.rs:136` | confirmed |
| `extract_tx` refuses on 3 counts | `bitcoin-0.32.101/src/psbt/mod.rs:197-215` | confirmed: `MissingInputValue`, `SendingTooMuch`, `AbsurdFeeRate` |
| BIP-174 finalizer retains the UTXO record | BIP-174 text | *"All other data except the UTXO … should be cleared from the PSBT. The UTXO should be kept to allow Transaction Extractors to verify the final network serialized transaction."* |
| BIP-341 signature is 64 B (DEFAULT) / 65 B (explicit sighash) | BIP-341 text | confirmed |
| BIP-341 control block is `33 + 32m` bytes | BIP-341 text | confirmed — **65 bytes for an ordinary 2-leaf tree (m=1)** |
| BIP-68 relative locks live in `nSequence`, consensus-enforced for nVersion ≥ 2, independent of `OP_CSV` | BIP-68 text | confirmed; BIP-112 (`OP_CSV`) is the separate script-level feature |
| GFM ignores table cells beyond the header count | GFM spec, tables extension | *"If there are greater, the excess is ignored"* |
| §3b chunk table arithmetic at 40 B/chunk | recomputed | 162→5, 405→11, 535→14, 742→19, 2498→63, **3538→89**; ceiling 64×40 = 2560 ✓ |
| §5's five legend fields sum | recomputed | 41+20+23+34+12 = **130** |
| §7 threat-model table cell counts | `tr -cd '|'` per row | every row 2 cells **except "Pinned fee", which has 3** |
| §8.6's stated mechanism | `grep -n "scriptSig" SPEC` | `scriptSig` appears at lines 647, 1125, 1301 only — **never in §8.6** |

## Verdict

**4 Critical, 8 Important, 4 Minor, 1 Nit.**

Two of the Criticals (S-2, S-4) are **recurrences of findings already filed in
R0 round 0 and round 1 and recorded NOT FIXED** by
`mt-spec-R0-round1-fold-check.md` (`S-12`/`R-12`, `R-10`). They are re-filed here
rather than treated as known, because this fold *escalated* both: it removed the
check that partially covered one, and added an explicit safety claim the other
falsifies. Two more (S-1, S-3) are **created by this fold** — they are the direct
interaction of "legacy inputs are accepted" and "there is no script engine",
which no round has reviewed together.

The single sharpest observation: **§8.2's removal did not just delete a check, it
made three surviving sections say things that are no longer true.** §8.2b claims
the absurd-fee refusal covers §8.2c's hazard (it structurally cannot), §8.6's
mechanism reaches only witnesses (and legacy inputs no longer have one), and
§7's "Wrong input value" row names an engraved mitigation that §5 has no field
for.

---

### S-1 — §8.6 is specified over the **witness**, and legacy inputs put the signature in the `scriptSig`. A `SIGHASH_NONE` legacy input is never examined.

**Severity: Critical.  Sections: §8.1, §8.6, §7, §10.10.**

Two rulings landed in the same fold and were not composed:

- §10.16: *"Do not exclude legacy inputs. It is user responsibility to know
  their inputs for such edge cases."* — every input type is now accepted.
- §8.2: script verification removed, so §8.6's box states the **only** mechanism
  it has left: *"Without a script engine `mt` inspects the witness
  **structurally** — it can tell that a stack element is *shaped* like a
  signature (a 64-byte Schnorr element, or a DER-encoded ECDSA one with a
  trailing sighash byte)"*.

§10.10's summary table says the same thing twice: `| §8.6 satisfaction binds
outputs? | parses the witness | parses the witness — **works** |`. The string
`scriptSig` occurs three times in the whole spec (lines 647, 1125, 1301) and
**not once in §8.6**.

A legacy P2PKH input has an **empty witness**; its signature and sighash byte are
a push inside the `scriptSig`. A legacy P2SH-multisig input is the same. §8.1's
admission rule is a disjunction — *"every input carries a populated
`PSBT_IN_FINAL_SCRIPTSIG` or `PSBT_IN_FINAL_SCRIPTWITNESS`"* — so such an input
is admitted, and then §8.6's only stated mechanism has nothing to look at.

**Scenario.** A wallet holds an old P2PKH UTXO of 4 BTC. It builds a spend to the
operator's cold-storage wallet and, through a coordinator bug or a deliberately
hostile PSBT, signs input 0 with `SIGHASH_NONE`. The finalized PSBT reaches `mt`.

- §8.1 — `PSBT_IN_FINAL_SCRIPTSIG` is populated. **Passes.**
- §8.2 — removed.
- §8.2b — inputs ≥ outputs, no duplicate outpoints, `vin` non-empty. **Passes.**
- §8.2c — legacy input present, so the warning fires; the operator reads it as
  being about the *amount*, which is what it is about. **Passes.**
- §8.4 — `nLockTime` read, warning printed. **Passes.**
- §8.5 — `gettxout` non-null. **Passes.**
- **§8.6 — inspects the witness. The witness is empty.** The spec gives an
  implementer two readings and does not choose:
  - *skip inputs with no witness* → the `SIGHASH_NONE` input is never examined,
    and the plate is engraved;
  - *no signature-shaped element found → refuse under §8.6b* → **every legacy
    input is refused**, silently reversing §10.16's ruling and §8.6's own
    retraction box (*"every input type is accepted"*).
- §8.7, §8.9 — pass.

Under the first reading the plate is cut. Its legend engraves `TO <cold-storage
id> 4.00000000`. Anyone who holds the plate — or photographs it in a safe-deposit
box, or is handed it by an heir — re-signs nothing: they simply replace every
output with their own address and rebroadcast, because `SIGHASH_NONE` leaves the
outputs unbound. The signature stays valid. §7 states the opposite: *"**Non-`ALL`
sighash** … refused at encode time, §8.6 — **structurally**"*.

**Why it matters.** This is the hazard §8.6 exists for, and it is open for
exactly the input class the same fold decided to accept. Note it is not really
"legacy" that is the trigger — it is §8.1's disjunction: **any** input presented
with a `scriptSig` and no witness escapes §8.6 entirely.

*Non-authoritative sketch, not a prescription:* the defect is the mismatch
between §8.6b's stated scope (*"the rule is over the **satisfaction**, not the
signature"*) and its stated mechanism (witness-only). Either the mechanism or the
admission rule has to reach the other place.

---

### S-2 — §6a fetches the chain's true input value and §8.5 throws it away, while §7 and §8.2c positively assert the value is undetectable

**Severity: Critical.  Sections: §6a, §8.5, §8.2c, §7, §8.6.**
**Recurrence of R-10 (round 1, Important, recorded NOT FIXED). Escalated here
because §8.2's removal deleted the only other check on the number, and this fold
added text that asserts the check is impossible.**

§6a gives three reasons for choosing `gettxout`, the first of which is:

> it returns `value` and `scriptPubKey` together, **so the PSBT's claimed UTXO
> records can be checked against the chain rather than trusted**;

§8 then contains exactly one refusal built on that call:

> 5. **`gettxout` returns `null` for any input** → refuse, when a node is
>    reachable.

The `value` field is parsed and discarded. Separately, §8.6's own retraction box
establishes a **second**, offline mechanism:

> BIP-174 requires `non_witness_utxo` for a legacy input — the **whole previous
> transaction** — so hashing it and matching the txid binds the amount without
> any help from the sighash.

I verified that this is not a stale claim: BIP-174's Input Finalizer says *"All
other data **except the UTXO** … should be cleared from the PSBT. The UTXO should
be kept to allow Transaction Extractors to verify the final network serialized
transaction."* So a conforming finalized PSBT with a legacy input **carries the
whole previous transaction**, and matching its txid against the outpoint binds
the amount with one hash, no node and no script engine.

§8 mandates neither mechanism. Meanwhile §8.2c prints, unconditionally, to the
operator:

>     mt CANNOT VERIFY THAT VALUE.

and §7 records:

> **Wrong input value** … **not detectable by `mt`.**

**Scenario.** The operator has a synced node (the case §6a is written for). The
PSBT spends one legacy input; its `non_witness_utxo` is present and the outpoint
it names really holds **10 BTC**. A stale UTXO snapshot in the sending wallet
recorded it as **1 BTC**, so the wallet built outputs totalling 0.99 BTC.

- §8.5 calls `gettxout <txid> <vout> false`. It returns non-null, with
  `"value": 10.00000000`. **The refusal passes and the value is discarded.**
- §8.2b computes the fee from the *claimed* 1 BTC: 0.01 BTC, ≈ 5,000 sat/vB on a
  ~200 vB transaction. Not below 10 sat/vB, not above 25,000 sat/vB. **No
  warning, no refusal.**
- §8.2c fires the legacy warning, whose text tells the operator `mt` cannot
  verify the value — which, on this invocation, is false twice over: the node
  just told it, and the PSBT contains the prevout.
- Plates are cut. The transaction is **valid**: a legacy sighash never committed
  to the amount, so the signature is fine.

On broadcast — today or in 2040 — the miner takes **9.01 BTC**.

**Why it matters.** The spec's honesty about this hazard is what makes the
operator ruling defensible; but the honesty is misplaced. `mt` is not blind here.
It is holding the answer in two places and the refusal list looks at neither,
while the artifact it prints teaches the operator that the number is
unverifiable. That last part is worse than silence: an operator who does mostly
legacy work learns the warning is boilerplate.

---

### S-3 — §7's mitigation for the catastrophic-fee hazard is an **engraved reminder that §5 has no field for**, and `mt qr`'s legend is `mt`'s to control

**Severity: Critical.  Sections: §7, §5, §8.2c, §4.**
**This is the exact defect class the brief flags as having recurred twice: a §7
mitigation naming legend content §5 does not engrave. It has recurred a third
time, in the row added by this fold.**

§7, "Wrong input value" row:

> Mitigated only by §8.2c's warning — which states the arithmetic, `(real input
> value) − (output total)`, since the output total is the one term `mt` knows for
> certain — **plus the engraved out-of-band reminder**

§8.2c closes the same way — *"The warning **and the engraved reminder** are the
whole mitigation"* — and its warning text instructs: *"Verify the input value out
of band, and **engrave a reminder** to re-check it before broadcasting."*

§5 engraves five fields and nothing else: `BEARER …`, `FROM WALLET <8 hex>`,
`LOCKED TO BLOCK <n>`, `TO <wallet id, fp or label>  <amount>`, `PLATE n OF m` —
130 characters over **6 lines**, and §4 reserves exactly those 6 lines. There is
no reminder field, no free line, and no flag that adds one. §10.4's free-text
flag applies only to `TO`, which §10.4 itself measures at *"roughly 16"*
characters after the amount.

The asymmetry is inverted from what §7 assumes:

| verb | who controls the steel | can the reminder be engraved? |
| --- | --- | --- |
| `mt qr` | **`mt`** — legend is 5 fixed fields in a 6-line reservation | **No.** Half of §7's stated mitigation cannot exist |
| `mt string` | the operator — *"Font size, characters per plate … are all the user's decisions"* (§3b) | Yes, but `mt` *"cannot verify that any warning reached the plate"* and *"takes no further interest in the steel"* |

**Scenario.** The operator engraves a `mt qr` plate for a spend with one legacy
input. `mt` prints the §8.2c warning and instructs them to engrave a reminder.
They want to comply. `mt qr` emits a `sysw` payload whose legend `mt` composed;
there is no flag, no field and no reserved line for the sentence they were just
told to add. They cut the plate as-is, because that is the only plate the tool
produces.

2040. An heir scans the QR, decodes the finalized PSBT, and reads the fee
straight out of it — the PSBT carries the *claimed* 1 BTC, so their wallet
displays a **0.01 BTC fee** and a green light. Nothing on the plate, and nothing
in the payload, says the number was never verified. They broadcast; 9.01 BTC goes
to a miner.

**Why it matters.** §7's second-to-last row is the one the spec itself calls
*"the honest state of this design"*. Half of the mitigation it names is not
merely unimplemented — it is **unimplementable in the verb `mt` controls**, and
`mt qr`'s payload actively supplies the wrong fee to a future decoder, which §7's
"Pinned fee" row presents as a *benefit* of that verb (*"A holder in 2040 recovers
the fee by decoding — only for `mt qr`, whose PSBT payload carries the input
amounts"*). For a legacy input, that recovery is confidently wrong.

---

### S-4 — `nLockTime` ≥ 500,000,000 is a Unix timestamp; §8.4 and §5 have one unit, and the fold's new "false reassurance is closed" claim is falsified by it

**Severity: Critical.  Sections: §8.4, §5, §7.**
**Recurrence of S-12 (round 0) and R-12 (round 1), both recorded NOT FIXED.
Re-filed because this fold introduced an explicit safety claim that this path
falsifies, and because the legend text changed underneath the old finding.**

Verified from source: `LOCK_TIME_THRESHOLD = 500_000_000`
(`bitcoin-units-0.1.101/src/locktime/absolute.rs:27`), documented in the same
file at `:18-19` as *"values **below** the threshold are interpreted as block
heights, values **above** (or equal to) the threshold are interpreted as block
times (UNIX timestamp, seconds since epoch)"*. `grep` over the whole spec for
`timestamp|median|MTP|Unix` returns **no hit inside §8.4 or §5**.

§8.4's input table asserts:

> | `nLockTime` | transaction field | **yes** |

and its whole design rests on *"Fields are certain; scripts are somebody else's
job."* But this field is certain only as 32 bits — its **meaning** depends on a
threshold the spec never mentions. §5 hardcodes one unit:

> | `LOCKED TO BLOCK <n>` | 23 | the single most actionable fact.

23 characters is `"LOCKED TO BLOCK "` (16) + 7 digits. A timestamp is **10
digits**, so the field also overflows its own budget.

And §8.4 now makes a positive safety claim that did not exist in the drafts where
this was previously filed:

> The unsafe direction, **false reassurance, is closed by the `nSequence` rule
> above.**

**Scenario.** The operator is building an inheritance spend: *"my heirs can
broadcast this after 1 January 2040."* Timelocked-inheritance wallets encode
calendar dates the way calendars work — miniscript `after(2208988800)`, a
timestamp, not a height. `nSequence` on the single input is `0xFFFFFFFE`, so the
locktime **is** enforced and §8.4's `nSequence` rule correctly says so.

`mt` prints, and engraves permanently:

    LOCKED TO BLOCK 2208988800   current height 963663

The operator, tired, reads a lock roughly 24,000 years out, concludes the plate
is inert for any practical purpose, and files it in an unlocked desk drawer
rather than the safe — the plate is, after all, *"not spendable"*. It becomes
spendable on 1 January 2040, and the `BEARER` line is the only thing standing
between it and whoever opens the drawer.

2040. The heir reads the same line against a chain height near 1,650,000,
concludes the plate is junk or mis-cut, and sets it aside. The transaction was
live the whole time.

**Why it matters.** This is precisely the failure §8.4 names as *"the worst
failure available here"* — *"engrave `LOCKED TO BLOCK 900000` on a plate anyone
can broadcast today. That is **false reassurance on steel**"* — reached by a
second, independent road that the `nSequence` rule does not touch. It needs no
script reading, no `OP_CSV`, and no node: it is one comparison against one
constant on a field `mt` already parses. And the inheritance/dormant-spend case
is not a corner — it is the artifact's headline use case.

---

### S-5 — §7's "Pinned destination" mitigation names a legend line that does not exist for `mt string`, and §7 has no verb convention

**Severity: Important.  Sections: §7, §5, §3b.**

§8's preamble establishes a convention and states it:

> **Every refusal below binds BOTH verbs** unless it names one

§7 has no such convention, and it is not verb-agnostic. Two of its rows name a
verb explicitly (`Bearer (mt qr)`, `Bearer (mt string)`), one names both inside
the cell (`Indistinguishable`), and the rest are unmarked. An unmarked row
therefore reads as "both verbs" — which is false for:

> | **Pinned destination** … | **cannot be fixed; partly disclosed.** **§5's `TO`
> line** names the destination **wallet** (id or fingerprint), which does not
> degrade with output count as the old truncated-address form did |

`mt string` has no §5 legend. §3b is explicit: *"`mt string` emits a string. That
is the whole of its output. … It does not require a legend, does not reserve
space for one, and cannot verify that any warning reached the plate."*

**Scenario.** An operator reviewing this threat model to decide which verb to use
for a 5 BTC dormant spend reads the Bearer rows, sees `mt string` honestly
labelled *"accepted risk"*, and concludes the remaining rows apply equally. They
choose `mt string` because the transaction is short and they want BCH
hand-engraving fault tolerance. The plate they cut carries codex32 characters and
whatever text they chose — no `BEARER` line, no `TO` line, no `FROM WALLET`, no
`LOCKED TO BLOCK`. Three of §7's eight rows silently lose their mitigation, and
the threat model told them only about one.

**Why it matters.** §7's own preamble is *"Every mitigation below names a field §5
actually engraves"* — the guard installed after this defect class was found twice.
The guard checks the wrong thing: it verifies the field exists in §5, not that §5
applies to the verb the row is claimed for.

---

### S-6 — §7's "Pinned fee" row has three cells in a two-column table, so its `mt string` disclosure never renders

**Severity: Important.  Section: §7.**

Measured (`tr -cd '|' | wc -c` per row of the §7 table): every row is 2 cells
except **Pinned fee**, which is **3**. GFM's tables extension: *"If there are
greater, the excess is ignored."*

The ignored cell is not filler. It is the only place in the spec that states the
fee-recoverability asymmetry between the verbs:

> Fee rate and date were cut from the legend (§5). `mt` displays both at encode
> time so the operator can judge staleness *before* engraving. A holder in 2040
> recovers the fee by decoding **only for `mt qr`**, whose PSBT payload carries
> the input amounts; an `mt string` plate carries a raw transaction, from which
> the fee is **not** recoverable without the prevouts.

**Scenario.** Every reviewer, implementer and operator who reads this spec
rendered — which is how a `.md` design artifact is read — sees the Pinned fee row
end at *"…too low for the parent to reach a mempool at all"*. The `mt string`
fee-blindness disclosure is present in the source and absent from the document.
An implementer building `mt string` therefore never learns that its artifact
cannot answer "what fee does this pay?" at recovery time, and does not flag it in
the CLI or the docs §10.17 asks for.

**Why it matters.** §6 makes the same point once (*"a raw transaction carries
outpoints only, so a string plate is silent about both the input amounts and the
source scripts"*), but §7 is the section a threat-model reader consults, and the
row that is supposed to carry it is truncated by the renderer. A disclosure that
does not render is not a disclosure. This is machine-checkable and would be caught
by extending `scripts/spec-structure-check.sh` to compare cell counts against the
header row — the structure gate currently checks headings, item sequence and
cross-references, not table arity.

---

### S-7 — §8.6's structural recognizer cannot distinguish a 65-byte Schnorr signature from a 65-byte taproot control block

**Severity: Important.  Sections: §8.6, §7.**

§8.6's box specifies the recognizer as *"a **64-byte** Schnorr element, or a
DER-encoded ECDSA one with a trailing sighash byte"*. BIP-341, verified: a
taproot signature is 64 bytes **only** for `SIGHASH_DEFAULT`; *"When a sighash
byte is included, the signature becomes 65 bytes long."* BIP-341 also fixes the
control block at *"length 33 + 32m, for a value of m … between 0 and 128"* — so
**m = 1, an ordinary two-leaf tree, produces a 65-byte control block.**

The recognizer as written therefore has two failure modes and the spec chooses
neither:

- **Strict (64 bytes only).** Every taproot spend carrying an explicit sighash
  byte — including a perfectly good `SIGHASH_ALL` one — has no recognizable
  signature. §8.6b (*"every input must carry at least one signature"*) refuses it.
  The operator cannot cut a plate for a valid transaction, and §8's promise that
  *"every refusal names the number that caused it"* has no number to name.
- **Lenient (64 or 65 bytes, last byte read as the sighash).** A 65-byte control
  block is now signature-shaped, and its "sighash byte" is the last byte of a
  32-byte merkle node — effectively random.

**Scenario (lenient reading).** The wallet is a taproot vault with two leaves:
`and_v(v:pk(A),after(N))` and `and_v(v:sha256(H),older(M))`. The spend uses the
hash-preimage leaf. The witness is `[preimage(32 B), script(N B), control(65 B)]`
— **no signature anywhere**, which is exactly the case §8.6b was added for in R0
round 1 (R-4): *"An input satisfied by preimage alone commits to **nothing**: any
holder can rewrite every output and re-satisfy it."*

`mt` scans the witness, finds the 65-byte control block, reads its last byte. One
time in 256 that byte is `0x01` and the input passes §8.6 as a clean `SIGHASH_ALL`
input. Worse, if the tree was supplied by a counterparty (a collaborative-custody
template, a vendor's wallet policy), grinding the second leaf's script until the
merkle branch ends in `0x01` costs seconds — and then it passes **always**. The
plate is cut, and any holder rewrites every output and re-satisfies the input with
the same preimage.

**Why it matters.** §8.6's box does disclose the class — *"A crafted witness
carrying a signature-shaped element that the script never checks would pass"* —
but presents it as a residual abstraction. It is not abstract: the collision is
with a structure that BIP-341 makes **mandatory** in every script-path spend, at
a length that is the single most common one. §7 still summarises §8.6 as
*"refused at encode time"*.

---

### S-8 — §10.13's content id is "the txid", but the two verbs decode to different objects and legacy / `sh(wsh(…))` inputs give them different txids

**Severity: Important.  Sections: §10.13, §3, §3b, §10.20, §8.6.**

§10.13(c) rules:

> **A content id — RULED: the transaction id.** … **Reassembly re-derives it from
> the decoded transaction and compares**, giving `mt1` the same invariant `md1`
> has.

and §10.13 quotes `md-codec` calling this *"the content-id oracle;
funds-load-bearing invariant."* But the two verbs do not decode to the same
object:

- `mt qr`'s payload is *"a fully finalized PSBT"* (§3). Decoding yields a **PSBT**,
  whose `unsigned_tx` has **empty `scriptSig`s** by construction.
- `mt string`'s payload is *"the raw signed transaction, NOT the PSBT"* (§3b).
  Decoding yields a **finalized transaction**, with `scriptSig`s populated.

For a native-segwit-only transaction these have the same txid, because the txid
excludes the witness and the `scriptSig` is empty either way. For a **legacy**
input, or a **`sh(wsh(…))`** input, the finalized `scriptSig` is non-empty and
`unsigned_tx.txid() != final_tx.txid()`. §8.6's retraction box explicitly admits
both classes: *"`sh(wsh(…))` is therefore no longer an unclassified case:
wrapped-segwit inputs are segwit inputs, and **every input type is accepted**."*
P2SH-wrapped segwit is not exotic; it is the dominant pre-2021 wallet shape.

The spec never says which txid. §10.13 says "the transaction id"; §3 says one verb
carries a PSBT; §3b says the other carries a transaction; §10.8 asserts *"One
mechanism, both media."*

**Scenario.** The operator engraves a `sh(wsh(2-of-3))` spend with `mt qr`. The
encoder derives `chunk_set_id` from the PSBT's `unsigned_tx` txid, because a PSBT
is what it was handed. Two years later the reader (§10.2's static-scan verb, built
by someone else against §10.13's one-line ruling) reassembles the chunks, extracts
the transaction, re-derives the txid from **the extracted transaction** — the
"decoded transaction" §10.13 names — and gets a different 20 bits. The
content-id oracle reports a mismatch. Under §10.13 that mismatch is
*funds-load-bearing*: the reader's correct behaviour is to refuse the reassembly.
**A correctly engraved plate set is unreadable by the correct reader**, and the
failure is deterministic for the entire class of transactions with any non-empty
`scriptSig`.

The mirror case is worse for a human: the same transaction engraved by both verbs
produces two different `chunk_set_id`s, so an operator who cuts a `mt string`
backup of a `mt qr` plate holds two artifacts that the tool says are different
transactions.

**Why it matters.** §10.20 already notes that a legacy txid is malleable and
concludes *"§10.13's content id is sound"* because the engraved bytes have exactly
one txid. That reasoning is right about *malleation* and silent about
*extraction*: the engraved bytes for `mt qr` are a PSBT, and a PSBT has two
candidate txids the moment any input is not native segwit. §10.20 is filed as a
disclosure item; this is a different defect wearing the same clothes.

---

### S-9 — the operator-supplied input value is consumed at encode time and never engraved, so the plate fails the very API §8.2b names as its standard of care

**Severity: Important.  Sections: §8.2c, §8.2b, §3, §6.**

§8.2c:

> Where a record is absent, `mt` requires the operator to supply that input's
> value — **or the total across all inputs** — since §8.2b cannot check the value
> balance without it.

Nothing anywhere says the supplied value is written into the payload. §3 says
*"The payload remains a fully finalized PSBT"* — the one that arrived, still
missing the record. Two consequences:

**(a) The plate is refused by the recoverer's default API.** §8.2b's own box makes
`extract_tx()` the standard of care: *"§3 rejected the `lean` PSBT form on the
grounds that 'the safe API a recoverer reaches for refuses it'. That API is
`extract_tx()`, and it refuses on three counts — `MissingInputValue`,
`SendingTooMuch` and `AbsurdFeeRate`."* Verified in
`bitcoin-0.32.101/src/psbt/mod.rs:197`: a missing input value is exactly
`ExtractTxError::MissingInputValue`. So `mt` engraves, onto steel, a PSBT that the
API it cites as safe **refuses** — and the recoverer's escape is
`extract_tx_unchecked_fee_rate()` (`:175`), i.e. extracting with the fee check
switched off, on an artifact whose whole hazard is the fee.

**(b) The two supply forms compose wrongly with partially-present records, and the
wrong composition understates the fee.** Consider a two-input spend: input 0 is
segwit with a `witness_utxo` of 1.0 BTC; input 1 is legacy and its record was
stripped. Outputs total 1.45 BTC. The operator means *"input 1 holds 1.5"* and
reaches for the flag that takes **"the total across all inputs"** — because that is
the phrasing they remember and §10.10 leaves the flag names *"still
unspecified"*. If `mt` reads it as the grand total, it computes inputs = 1.5, fee
= **0.05 BTC** — plausible, no `AbsurdFeeRate`, no sub-10-sat/vB warning. The
true input total is 2.5 and the true fee is **1.05 BTC**. Plates are cut; a miner
takes 1.05 BTC on broadcast.

**Why it matters.** The supplied value is the *only* thing standing between the
operator and both of §8's fee outcomes — the low-fee warning and the
`AbsurdFeeRate` refusal both consume it — and the spec offers two mutually
incompatible ways to provide it, defines neither flag, and states no rule for how
either composes with records that are already present. Then it discards the number
rather than binding it to the artifact.

---

### S-10 — §8.2b's absurd-fee refusal is computed from claimed values, so it catches only the harmless direction of §8.2c's error

**Severity: Important.  Sections: §8.2b, §8.2c.**

§8.2b:

> - **an absurdly HIGH fee** — `rust-bitcoin`'s own ceiling is
>   `DEFAULT_MAX_FEE_RATE = 25,000 sat/vB`, raised as `AbsurdFeeRate`. **This is
>   the direction that loses money, and it is what a wrong input value produces
>   (§8.2c)**.

(`DEFAULT_MAX_FEE_RATE = 25,000 sat/vB` verified at
`bitcoin-0.32.101/src/psbt/mod.rs:136`.)

The refusal is computed on the values `mt` was given. A wrong claimed value moves
`mt`'s arithmetic and the chain's arithmetic in **opposite** directions:

| operator/PSBT claims | `mt` computes | reality | outcome |
| --- | --- | --- | --- |
| **too high** (says 10, holds 1) | huge fee → `AbsurdFeeRate` | fee is small and the transaction is fine | **`mt` refuses a good transaction** |
| **too low** (says 1, holds 10) | small, plausible fee → passes | fee is 9.01 BTC | **`mt` engraves it, and the miner takes it** |

So the refusal fires exactly when nothing is wrong, and is structurally incapable
of firing in §8.2c's scenario — the one the sentence cites it for.

**Scenario.** Identical to S-2's, minus the node. The PSBT claims 1 BTC on a
legacy input holding 10; outputs total 0.99. `mt` computes 0.01 BTC ≈ 5,000
sat/vB, five times under the ceiling. Every refusal passes. The plate is cut.

**Why it matters.** An implementer reading §8.2b will believe the absurd-fee
refusal is the backstop for §8.2c, and will not look for another. It is not a
backstop; it is a mirror that only ever shows the safe case. §8.2c is honest
about this in isolation (*"the fee absorbs the entire difference"*), but §8.2b
tells the reader the check covers it, and §8.2b is the normative list.

---

### S-11 — §6a discloses only the `null` side of `gettxout`'s ambiguity; a stale or wrong-chain node returns a value for an output spent long ago

**Severity: Important.  Sections: §6a, §8.5, §7. (§10.5's ruling that `mt` does not
vouch for the node's sync state is respected — this is a disclosure gap, not a
request for an IBD check.)**

§6a discloses one direction:

> a `null` cannot distinguish "already spent" from "this node is still syncing, or
> is on the wrong chain".

and `include_mempool=false` is disclosed as making a mempool-spent input read as
unspent. The **non-null** direction is not disclosed anywhere: a node whose tip is
behind the network returns the *old* UTXO set, so an output spent after that tip
reads as unspent. §8.5's refusal only fires on `null`, so it passes.

**Scenario.** The operator's node was restored from a two-week-old snapshot and is
still catching up; `mt` is given its RPC endpoint. Ten days ago the operator swept
the input in question into a different wallet. `gettxout` answers from the stale
UTXO set: non-null, `value` intact. §8.5 passes. `mt` engraves — 4 plates at
~21 minutes each (F-225), a `wsh` tier-1 5-input artifact per §4's table. The
plate is void the moment it leaves the machine and nothing on it or in the tool
says so.

§7 records the hazard as checked: *"**Silent invalidation** … `mt` checks it at
encode time (§6a, §8.5); after that the hazard is open"*. The check is weaker
than the row claims, in the direction that wastes the artifact.

**Why it matters.** §6a's stated purpose is *"before you spend ~21 minutes a
plate, is this transaction still worth engraving?"* Both disclosed failure modes
(`null`-ambiguity, mempool exclusion) fail toward caution. The undisclosed one is
the only one that fails toward cutting a dead plate, and §7 counts the check as a
mitigation for exactly that hazard.

---

### S-12 — `TO <wallet id>  <amount>` has no defined semantics for a multi-output transaction; R-14 fixed the identity half of the line and left the amount half

**Severity: Important.  Sections: §5, §7, §10.4.**
**Related to S-14 (round 0, recorded NOT FIXED) and R-14 (round 1, Critical, folded
— but folded only over the destination, not the amount).**

§5's field is `TO <wallet id, fp or label>  <amount>`, 34 characters. The spec
explains at length why the *identity* half changed:

> It was `TO <truncated addr>` until 2026-08-23, showing **one** output and
> truncated — so a transaction with change named one destination, silently
> omitted the rest … R0 round 1 (R-14) filed that as a Critical

`<amount>` is never defined. For a transaction with more than one output — which
is nearly all of them, since change is an output — it could mean the total to the
named wallet, the total of all non-change outputs, the largest output, or the sum
of all outputs. §5 and §10.4 both discuss the field at length and neither says.

**Scenario.** A transaction pays 0.5 BTC to `ACME`, 5.0 BTC to a second
counterparty, and 0.2 BTC change. The implementer picks "the amount to the named
wallet" — the reading §5's own phrasing (*"names the destination **wallet**"*)
most supports. The plate engraves `TO ACME 0.50000000`.

2040, an heir reads the plate: a bearer instrument that moves half a bitcoin to
ACME. They broadcast it to settle what they believe is a small obligation. It
moves **5.7 BTC**, 5.0 of it to a counterparty the plate never names.

**Why it matters.** This is R-14's defect — *"showed one output of several and
could not be checked by eye"* — surviving in the other half of the same line. The
fold replaced the address with a wallet id because *"it does not degrade with
output count"*; the amount beside it degrades with output count exactly as the
address did. §7's pinned-destination row leans on this line as its *"partly
disclosed"* mitigation.

---

### S-13 — §8.7b cites 78 chunks where §3b's corrected table says 89

**Severity: Minor.  Sections: §8.7b, §3b, §11.**

§8.7b: *"Real wallets hit this: RCW `wsh` tier 1 at 5 inputs needs **78 chunks**
(§3b)."* §3b's table: `| RCW wsh tier 1, 5-in/2-out | 3538 | **89** | **NO —
refused** |`. Recomputed: `ceil(3538 / 40) = 89`; every other row in §3b's table
agrees with 40 B/chunk (162→5, 405→11, 535→14, 742→19, 2498→63). `3538 / 45.375 =
78` — 78 is the **superseded filled-chunk model** the §3b correction retired, and
it survived in §8.7b.

**Scenario.** No funds outcome — both numbers exceed 64 and the refusal fires
either way. But this is precisely the *"folds fail by incomplete propagation"*
pattern the correction box in §3b describes for its own predecessor (*"The caveat
existed and was lost in transit"*), reappearing in the same fold that fixed it,
and it is the number a test vector for §8.7b's refusal message would be written
from.

---

### S-14 — §8.4's "`mt` will therefore OVER-WARN" and §1.7's "warns if immediate" both describe output the operator overruled

**Severity: Minor.  Sections: §8.4, §1.7, §7.**

§8.4's rule, after the ruling, is *"**`mt` states the two facts and stops** …
the `stderr` report is a **statement of what was read, not a verdict**"*. There is
no warning left to be over- or under-. Yet:

- §8.4's own box, two paragraphs later: *"**`mt` will therefore OVER-WARN on such
  transactions**, which is the safe direction: **it says a plate might be
  spendable when it is not**"* — quoting the `may be immediately spendable`
  verdict the ruling in the same section deleted.
- §1.7: *"**`mt` does not offer a locktime CHOICE. It reads the transaction and
  warns if the plate would be immediately spendable.**"* — the same deleted
  verdict, in the ruling list.

**Scenario.** An implementer works from §1's decision list (which is where a
reader looks for "what does this tool do") and builds the immediate-spendability
verdict. It reaches the operator as `MAY BE IMMEDIATELY SPENDABLE` on stderr —
precisely the output §8.4 rejects as *"true of almost any transaction and tells
the operator nothing they can act on"*, and the phrasing the same section calls
unsubstantiable now that scripts are out of scope.

The safety *conclusion* the box draws still holds under the fact-only output, for
`OP_CSV`-style locks — but by a different argument than the one written, and the
argument as written is about text that no longer exists.

---

### S-15 — BIP-68 relative locks live in `nSequence`, a field `mt` already reads; §8.4's stated reason for not reporting them is a wrong protocol fact

**Severity: Minor.  Sections: §8.4, §5.**

§8.4's disclosure box:

> A BIP-68 **relative** timelock **lives in the witness script as `OP_CSV`** …
> **Reading it means evaluating the sending wallet's script**, which is out of
> scope by ruling.

Verified against BIP-68: *"If bit (1 << 31) of the sequence number is not set,
then the sequence number is interpreted as an encoded relative lock-time"*, for
*"transactions with an nVersion greater than or equal to 2"*, and enforcement
*"operates at the consensus transaction level rather than requiring script
operations"*. `OP_CSV` is BIP-112 — the separate, script-level feature that lets a
script *require* such a lock.

So the relative lock's magnitude and unit are in `nSequence` (bits 0-15 plus the
bit-22 type flag) and its applicability is in `nVersion` — two transaction
**fields**, on the same footing as `nLockTime`, and §8.4 already mandates reading
one of them. §8.4's own principle is *"Fields are certain; scripts are somebody
else's job."* The spec never mentions `nVersion` at all
(`grep -n "nVersion" SPEC` → no hit).

**Scenario.** The operator engraves a spend from the RCW's own `older(32768)`
leaf — the spec's cited example, `nSequence = 0x00008000`, ≈ 7 months. `nLockTime`
is 0, so §5 engraves **`NO BLOCK TIMELOCK`** on steel. That statement is false in
the sense a human reads it: the transaction has a 32,768-**block** relative lock,
and `mt` was holding the number. The heir who scans it inside the window
broadcasts and is rejected with `non-BIP68-final`, with nothing on the plate or in
the tool to explain why.

**Why it matters.** Not a loss path — the plate becomes live on schedule and the
misreading errs toward treating it as bearer. But §8.4's box is the spec's
statement of what `mt` structurally cannot see, and it is wrong about where the
data lives, which means the boundary between "field" and "script" that the whole
section rests on is drawn in the wrong place.

---

### S-16 — "the MIN form (§3)" and "the `lean` PSBT form (§3)" point at text §3 no longer contains, and nothing states what the engraved PSBT may carry

**Severity: Minor.  Sections: §8.2b, §8.2c, §3.**
**Related to R-11 (round 1). The structure gate cannot catch this: it checks that
a cross-reference *resolves*, and states plainly that it does not check whether it
points at the right thing.**

- §8.2c: *"A finalized PSBT in **the MIN form** normally carries every input's
  UTXO record (§3)"*.
- §8.2b's box: *"§3 rejected the **`lean` PSBT form** on the grounds that…"*.

`grep -n "MIN form|\`lean\`" SPEC` returns exactly those two lines. §3 was rewritten
when UR was dropped and defines neither term. So the spec has no statement of
which PSBT fields the engraved payload must contain (§8.2c's requirement rests on
it) or must not contain.

**Scenario.** A real multisig PSBT from Sparrow or Specter carries
`PSBT_GLOBAL_XPUB` entries. With no admission or normalisation rule, they ride
into the payload: the plate permanently carries the operator's entire watch-only
wallet — every address, past and future — on an artifact §7 already establishes
is bearer and photographable. §7's hazard table has no privacy row. The extra
bytes also push the artifact past the plate counts §4 predicts, since every figure
in §4's table came from a probe-built PSBT with no global map.

---

### S-17 — §5 says "130 characters" and then "the 136-character budget above"

**Severity: Nit.  Section: §5.**

`41 + 20 + 23 + 34 + 12 = 130`, matching §5's *"Five fields, **130 characters**, 6
lines — measured"*. Eleven lines later: *"Plus, **not part of the 136-character
budget above**…"*. One of the two is a leftover from a superseded field set.

---

## What §8 no longer catches

`mt` now performs no signature verification and no script evaluation. The
surviving refusals are §8.1 (finalized), §8.2b (value-blind acceptance), §8.2c
(input values present + legacy warning), §8.3 (unsigned), §8.4 (locktime warning,
never refuses), §8.5 (`gettxout` null), §8.6 (satisfaction binds outputs,
**structural heuristic**), §8.7/§8.7b (size), §8.8 (module size — not a refusal),
§8.9 (secrets).

Enumeration of what a **finalized-but-defective** transaction can look like,
against that list. "Engraved" means every surviving refusal passes.

| # | shape of the defect | caught by | outcome |
| --- | --- | --- | --- |
| 1 | **Bad signature** — wrong key, corrupted bytes, signed over the wrong sighash message | **nothing** | Engraved. Fails at broadcast, years later. **Disclosed** (§7 "Well-formed but INVALID", §8.2's box) |
| 2 | **Segwit input signed over a wrong amount** — signature commits to the false value, so it is invalid on chain | **nothing** | Engraved, permanently unbroadcastable. Covered by row 1's disclosure; §8.2c's table says such a value is *"caught by anyone who verifies"* — no longer true of `mt` |
| 3 | **Legacy input with a wrong claimed value** — valid transaction, catastrophic fee | §8.2c **warns**; §8.2b's `AbsurdFeeRate` fires only on the *over*-claim direction (**S-10**); the node's true `value` and the PSBT's `non_witness_utxo` are both in hand and unused (**S-2**) | Engraved. **Funds loss on broadcast** |
| 4 | **Witness script / redeem script does not hash to the scriptPubKey** — a pure hash check, no script evaluation | **nothing** (was §8.2) | Engraved, permanently unbroadcastable. **Not disclosed anywhere**; §7's row 1 disclosure is about *signatures*, and this is not one |
| 5 | **Taproot control block does not commit to the leaf** (wrong merkle path / wrong parity) — a tweak check | **nothing** (was §8.2) | Engraved, permanently unbroadcastable. Not disclosed |
| 6 | **k-of-n multisig finalized with k−1 signatures**, or signatures in the wrong order | **nothing** — §8.6 requires *"at least one signature"*, not *enough* | Engraved, permanently unbroadcastable. Not disclosed |
| 7 | **Non-`ALL` sighash on a legacy input** | **nothing** — §8.6 inspects only the witness (**S-1**) | Engraved. **Any holder redirects the funds** |
| 8 | **Satisfaction with no signature at all** (preimage-only, timelock-only) | §8.6b — but its recognizer collides with the 65-byte control block (**S-7**) | Engraved when the collision is hit or ground for. **Any holder redirects the funds** |
| 9 | **Dust outputs / non-standard scriptPubKey / oversize scriptSig / >80 B OP_RETURN** — consensus-valid, will not relay | **nothing** — and was never caught: `verify_script` is consensus, not policy, so §8.2's removal did not change this (R-9, still open) | Engraved. Broadcastable only out of band, which §7's fee row already names as the escape hatch |
| 10 | **Empty `vout`** — consensus-invalid | **nothing** — §8.2b requires `vin` non-empty and says nothing about `vout` | Engraved (the whole input value becomes fee, so `AbsurdFeeRate` will usually but not always fire first) |
| 11 | **Immature coinbase input** | **nothing** — `gettxout` returns `"coinbase": true` and `"confirmations"`, both discarded (R-18, still open) | Engraved. Self-resolving in ≤ 100 blocks; low harm for a dormant artifact |
| 12 | **Input already spent** | §8.5, **only as strong as the node** — a stale/pruned/wrong-chain node answers non-null (**S-11**), and `include_mempool=false` hides a mempool spend (disclosed) | Engraved. Dead plate |
| 13 | **Locktime not yet satisfied** | §8.4 **warns**, never refuses — ruled | By design. But the warning has one unit (**S-4**) and ignores relative locks (**S-15**) |
| 14 | **Sighash `ANYONECANPAY`** | §8.6a, by whitelist (only `SIGHASH_ALL` and taproot `SIGHASH_DEFAULT` accepted) — sound as written, and only for inputs §8.6 can actually reach (see row 7) | Refused ✓ |

**The pattern.** Rows 4, 5 and 6 are the substantive new gap: they are
*commitment* checks, not script *evaluation*, and each is one hash or one
comparison. §7's accepted-hazard row covers only *"a transaction with a bad
signature"*, so a spec reader would conclude that a witness which fails to match
its own scriptPubKey is caught by something. Nothing catches it. That is a
disclosure gap under the operator's ruling rather than a challenge to it — the
ruling is *"We don't care if transaction is valid for initial version"*, and the
consequence the spec has not written down is that **`mt` cannot tell a signed
transaction from a structurally mismatched one, and its only stated example of
what it misses is the narrowest case in the class.**

Rows 3, 7 and 8 are the funds-loss rows, and are S-2/S-10, S-1 and S-7 above.
