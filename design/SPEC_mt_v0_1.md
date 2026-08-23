# SPEC — `mt`, the mnemonic-transaction format (v0.1 draft)

Status: **DRAFT, pre-R0. No code may be written against this until it passes an
architect R0 review at 0 Critical / 0 Important.** This is risk-set work by the
project's own definition — it touches funds, addresses and a new normative
format — so the gate is not optional.

Written 2026-08-22 from a brainstorm with the operator. Every number in it was
measured; the probes and raw results are in `design/measurements/`.

---

## 0. What this is, in one paragraph

`mt` covers the three steps between "I want to store a spend on steel" and a
plate. It **produces** an unsigned transaction from inputs and outputs it is
given; it **presents** that transaction for hand-off to a signing device, on a
screen or in a file; and it **engraves** the signed result — deciding how many
QR symbols, at what error-correction level, across how many plates, and what is
engraved beside them so a human in 2040 knows what they are holding.

It does not sign, holds no private key, does not choose which UTXOs to spend or
what fee to pay, and does not define a new encoding for the transaction itself:
the bytes go into the QR in a form the wider Bitcoin ecosystem already reads.
The three verbs are pulled apart in §1a, because conflating them caused two
reversals while this was being written — and only signed transactions are ever
engraved.

## 1. The operator's decisions, recorded

Each of these is a ruling, with the reasoning that produced it. Several
overturned an earlier assumption and are marked.

1. **Produce, present and engrave — three verbs, all in v0.1**, but applying to
   different artifacts. `mt` builds unsigned transactions, presents them for
   hand-off to a signing device, and engraves only the signed result. It never
   signs and holds no private key. See §1a.
2. **Its own repository**, `mnemonic-transaction`, with `mt-codec` and an `mt`
   CLI — not a subcommand of `me`. **This overrules the recommendation in
   §Section 1 of the brainstorm**, which argued `mt` had no wire format left to
   define and belonged next to `me bundle`. See §2 for what the codec does in
   fact specify; the objection was answered rather than ignored.
3. **The QR carries the standard form, never a codex32 string** (F-234).
4. **Reed-Solomon density is the highest that still minimises plate count.**
5. **Provenance rides in the engraved legend, not in the wire format.**
6. **A future locktime is required by default**, with an explicit flag to
   override.

## 1a. Present, produce, engrave — three different things

Conflating these caused two reversals during the brainstorm, so the spec names
them separately and rules on each.

| verb | means | in v0.1? |
| --- | --- | --- |
| **engrave** | commit bytes to steel | **yes — signed transactions only** |
| **present** | render a PSBT *it was handed* for hand-off to a signing device | **yes** |
| **produce** | build an unsigned transaction from chosen inputs | **yes — §6a governs the amounts** |

**Presenting is not producing, and the difference decides who owns the input
amounts.** A PSBT `mt` is handed already carries them, and whoever built it took
responsibility. When `mt` produces one, `mt` owns them — and getting them wrong
is the most dangerous thing in this spec. §6a governs it.

**Presenting is a screen and a file, never a plate.** The medium is `ur:psbt`,
the same UR machinery §3 specifies, which is what Sparrow, Keystone, Passport
and Specter already consume as an animated QR. An unsigned transaction is
worthless as a backup — it cannot be broadcast — so it has no business on steel,
and §8 refuses it there.

## 2. What `mt-codec` actually specifies

The payload is a raw Bitcoin transaction, specified by Bitcoin. The container is
a QR, specified by ISO/IEC 18004. Neither needs `mt`. What is unspecified — and
what this codec is for — is everything between them:

- how one transaction maps onto **one or more** QR symbols;
- how a recoverer reassembles them, and how they know a fragment is missing;
- which (module size, QR version, ECC level, tiling) configuration is chosen for
  a given transaction, deterministically, so two encoders agree;
- what is engraved **beside** the symbols, so the plate is self-describing;
- what `mt` refuses to engrave at all.

That is a real format. It is a *plate* format rather than a *string* format,
which is why it has no bech32 HRP and no BCH checksum.

## 3. The envelope: UR

Fragmentation uses **UR (Uniform Resources, BCR-2020-005)**, type `ur:bytes`.

**Why not the alternatives.** QR's own Structured Append is unavailable in
practice: the `qrcode` Rust crate knows it only as a mode indicator with no
encoder, the SeedHammer fork has none, and Bitcoin wallets do not read it. An
`mt`-specific envelope would reintroduce exactly the dependency F-234 exists to
remove — a recoverer would need `mt`-aware software, and the QR's entire purpose
is to be the escape hatch for someone who has none of our tools.

**Why UR is positively good here, not merely available.** It is already vendored
and device-tested in the fork (`bc/ur`, `bc/bytewords`, `bc/fountain`), it is
what Sparrow, Keystone, Passport and Specter already read, and `ur.Split` is
**fountain-coded** — the transaction recovers from any sufficient subset of
fragments rather than requiring every one. On steel that directly answers the
per-block Reed-Solomon concern recorded in F-234: losing one symbol to a deep
scratch becomes survivable instead of fatal.

**Its cost, measured.** Bytewords minimal is exactly 2 characters per byte plus
an 8-character CRC32 (`bc/bytewords/bytewords.go:17-31`, read from source).
Uppercased, `ur:bytes/N-M/…` is fully QR-alphanumeric — `:` and `/` are both in
the alphanumeric set — so it costs **11 bits per payload byte against raw
binary's 8**, a 37.5% expansion. Under the §4 rule that costs an ECC level on
mid-size artifacts and a whole plate on 9-of-11.

> **NOT YET MEASURED.** The per-fragment CBOR header that UR adds (seqNum,
> seqLen, message length, checksum) is not in the 2-chars-per-byte figure. Every
> plate count in this spec is therefore a **floor** for the UR path. Measure
> before any of these numbers reach an implementation plan.

## 4. Choosing the configuration

> **Rule (operator, 2026-08-22): the Reed-Solomon density is the highest that
> minimises plate count.**

Plate count is the real cost — one plate per string today, ~21 minutes each
(F-225) — so the search minimises plates first and spends every leftover byte on
error correction. Never trade a plate for redundancy; never leave redundancy
unbought.

    search space:  module size x QR version (1..40) x ECC (L,M,Q,H) x k*k tiling
    objective:     minimise plates
                   then maximise ECC
                   then minimise symbol count
    plate:         85 x 85 mm, outerMargin 3 mm => 79 mm usable
    quiet zone:    4 modules per side, per symbol

The rule turns out to be nearly free. Measured, at the conservative 0.60 mm
module (`RESULTS_ecc_selection_2026-08-22.txt`):

| artifact | raw bytes | UR |
| --- | --- | --- |
| RCW `tr` key-path, 1-in | 1 plate, v13, **ECC H** | 1 plate, v16, **ECC H** |
| RCW `tr` tier 4, 1-in | 1 plate, v22, **ECC H** | 1 plate, v26, **ECC H** |
| 3-of-5 signed, 1-in | 1 plate, v24, **ECC H** | 1 plate, v25, ECC Q |
| RCW `tr` tier 1, 1-in | 1 plate, v25, **ECC H** | 1 plate, v26, ECC Q |
| RCW `wsh` tier 1, 1-in | 1 plate, v26, ECC Q | 1 plate, v26, ECC M |
| 9-of-11 signed, 1-in | 1 plate, v24, ECC L | **2 plates**, v22, ECC M |

A 162-byte key-path spend would fit at ECC L in a v13 with room to spare; the
rule spends that room on H instead. Same plate, same 21 minutes, four times the
damage tolerance. It also degrades in the right order — H → Q → M → L *before*
it gives up a plate.

**Module size.** 0.30 mm is one engraved stroke: the theoretical floor and
**optically unvalidated**. Whether a camera reads 0.30 mm modules off brushed
steel is a hardware question, gated on the test plate in F-234. **Until that
plate exists, `mt` must not select a module below 0.60 mm** (two strokes). The
0.30 mm results are recorded for when it does.

## 5. The plate legend

Everything constellation-specific lives here, in engraved text, never in the QR.

| field | why |
| --- | --- |
| `BEARER — anyone holding this plate can spend it` | the plate is spendable; this is not a backup in the sense the other formats are |
| source wallet: **4-byte policy-id stub** | the transaction does not say what it spends *from* (§6) |
| **txid** | the transaction's own identity; lets a recoverer confirm the QR decoded to the right thing without trusting the QR |
| destination address(es) and amounts | already in the transaction; shown so a human need not decode it |
| **input outpoints** (`txid:vout`) | the only actionable mitigation for silent invalidation (§7) |
| block hash + height per input, when known | lets a future recoverer verify inclusion with one node command (§6c) |
| `input existed not before <MTP>` | a date bound a human can act on; median-time-past, not the header's own stamp (§6d) |
| locktime, in height or time | says when the plate becomes live |
| fee rate **and the date it was chosen** | makes staleness visible |
| symbol index / total, if more than one plate | so a missing plate is obvious |

**The stub is a hint, never an authority.** It is the top 4 bytes of a canonical
md1 identity, form-aware — WalletPolicyId for a keyed wallet, the key-stable
WalletDescriptorTemplateId for a keyless template — reusing `mk1`'s existing
derivation (`POLICY_ID_STUB_BYTES = 4`, `mk-codec/src/key_card.rs:24-32`,
`derive_stub_from_md1`), so one convention spans the constellation. If the
legend says wallet X and the transaction spends wallet Y's UTXOs, **the
transaction wins.** The stub exists to help a human find the right plates, not
to validate anything, and nothing may branch on it.

## 6. Why provenance is asymmetric

**"Goes to" is already in the transaction.** Outputs carry scriptPubKeys; any
standard decoder yields addresses and amounts. Encoding destinations into the
wire format would create a second source of truth that can disagree with the
transaction — and on disagreement a recoverer would have to guess which to
believe. That is a funds-safety hazard, not a feature. It is displayed, never
encoded.

**"Comes from" is genuinely absent.** A signed transaction references inputs as
outpoints only; the source scriptPubKeys live in the *previous* transactions.
Without the UTXO set you cannot tell which wallet it spends. Hence the stub —
and hence the stub living in text, because it is the one constellation-specific
fact on the plate and F-234 forbids that inside the QR.

## 6a. Producing: where the input amounts come from

**This is the most dangerous section in the spec.** Building a transaction
requires each input's **amount**, and getting it wrong is bad in both
directions:

- **Underestimate** an input's value and the real fee — `inputs − outputs` — is
  larger than intended. The operator overpays, possibly enormously.
- **Overestimate** and the outputs exceed the inputs. The transaction is simply
  invalid, and useless for the recovery it was built for.

**Signing catches an honest mistake.** BIP-341's sighash commits to
`sha_amounts`, *"the SHA256 of the serialization of all input amounts"*, and to
`sha_scriptpubkeys`, for every input unless `SIGHASH_ANYONECANPAY` is set. The
BIP states the purpose plainly: *"This eliminates the possibility to lie to
offline signing devices about the fee of a transaction."* So a wrong amount does
not merely misprice the fee — it produces a signature that will not verify.
Segwit v0 commits to the signed input's own amount via BIP-143.

**But it does not catch a CONSISTENT mistake, and that is the real hazard.** If
an operator certifies a wrong value, the signer signs against it and §8.2's
libbitcoinconsensus check verifies against the same asserted prevout. Every
local check passes and the plate is cut. The transaction then fails on the
network — years later, in precisely the situation it was engraved for.
**Self-certified amounts are checked against themselves.**

### The circularity is breakable without a node

Requiring the **full previous transaction** rather than an asserted amount binds
the amount cryptographically: hash the supplied transaction, require its txid to
equal the input's `previous_output.txid`, then read the value out of
`output[vout]`. Forging that needs a txid collision. This is exactly the
distinction PSBT draws between `non_witness_utxo` (the whole previous
transaction) and `witness_utxo` (a bare amount and scriptPubKey), and it is why
wallets hardened toward the former after the segwit fee-lying attack.

So there are four tiers, and `mt` must treat them as different:

| source | amount trustworthy? | unspent? | verdict |
| --- | --- | --- | --- |
| **`bitcoind` reachable** | yes, authoritative | **yes** | **preferred — fetched automatically, §6b** |
| full previous transaction **+ `gettxoutproof`** | **yes — txid, anchored to proof-of-work** | unknown | **accepted**, strongest offline tier |
| full previous transaction alone (`non_witness_utxo`) | **yes — bound by txid** | unknown | **accepted**, caveat stated on screen and in the legend |
| bare asserted amount (`witness_utxo` alone, operator-typed) | **no — self-certified** | unknown | **refused** by default; `--i-certify-amounts` overrides and the legend records it |

### 6b. When `bitcoind` is reachable, fetch the prevouts — do not ask

**Operator ruling 2026-08-22.** If a node is available, `mt` resolves every input
itself and the operator is asked for nothing.

The call is **`gettxout <txid> <vout> false`**, verified against a live Core
v25.0.0 node while writing this. It is the right RPC for three reasons:

- it returns `value` and `scriptPubKey` together — everything an amount needs;
- **it answers unspentness in the same call**, because it queries the UTXO set
  rather than the chain. A spent or nonexistent output returns `null`, which is
  a clean, unambiguous refusal;
- it needs **no `-txindex`**, unlike `getrawtransaction`.

`include_mempool` is passed **false** deliberately. The default is `true`, and
mempool state is the wrong basis for an artifact meant to sit in a drawer for
years — an input that is unspent only until someone else's transaction confirms
is not a foundation for a backup.

A `null` from `gettxout` is a **hard refusal**: the input is already spent or
never existed, and no flag overrides it. `mt` records which tier supplied each
amount, so the provenance of the numbers is auditable after the fact.

### 6c. Proving an input existed, without a node

*"Is there a way we can prove a transaction is in a block with a costly
header?"* — operator, 2026-08-22. Yes: a Merkle inclusion proof, which is what
`gettxoutproof` emits and `verifytxoutproof` checks. Demonstrated end to end
against the live node: a transaction in a 4,886-transaction block produced a
**538-byte** proof — an 80-byte block header, the transaction count, and a
13-hash Merkle branch — and verification independently recovered the txid.

The security argument is the operator's: forging the header requires redoing its
proof-of-work.

**What it establishes, and what it does not.** Three limits, all load-bearing:

1. **It proves inclusion, never unspentness.** A perfectly valid proof can
   describe an output spent years ago. This is the same gap as §7's silent
   invalidation, and it is disclosed the same way — the outpoints go on the
   plate.
2. **A lone header proves little.** Eighty bytes of header can be produced
   cheaply *in isolation*; the work only means something once the header is
   known to sit on the real chain with cumulative work behind it.
   `verifytxoutproof` gets that for free by checking against a node's own chain.
   **Offline, `mt` has no such anchor**, so the proof is only as strong as the
   verifier's chain context — which is precisely what the offline case lacks.
3. Therefore the proof is most useful **deferred**. `mt` stores the block hash
   and height and puts them in the legend, so a future recoverer — who will
   almost certainly have a node, since they are about to broadcast — verifies in
   one command. Verification moves to the moment when chain context exists.

**The 538 bytes never reach the steel.** The proof is an *input* to `mt`, not an
output: only the block hash and height go in the legend. Plate counts in §4 are
unaffected.

### 6d. Telling the operator what the header cost, and when it was made

*"There is a cost to producing each header. Can we convey to the user how costly
the header was to create?"* and *"And the timestamp of the block?"* — operator,
2026-08-22. Both come out of the same 80 bytes, and they answer different
questions: **how expensive** this evidence was to forge, and **how old** it is.

**Work, from `nBits`.** The header's compact target expands to
`target = mantissa x 2^(8 x (exponent - 3))`, and the expected number of hashes
is `2^256 / (target + 1)`. Self-contained: no price feed, no network statistics,
nothing that decays. Computed on the block proved in §6c:

| | |
| --- | --- |
| `nBits` | `17023cc1` |
| difficulty | 125,807,076,547,198 |
| **expected hashes for this ONE header** | **5.403 x 10^23** |
| chainwork at that height | 9.952 x 10^28 cumulative hashes |

**Date, from `nTime` — with a caveat that must not be dropped.** The header says
`2026-08-23T00:56:49Z`, but a block's `nTime` is only loosely constrained: it
must exceed the median timestamp of the previous 11 blocks, and must not exceed
network-adjusted time by more than two hours. It is therefore **not monotonic** —
a block's stamp can precede its parent's — and can run up to ~2 h fast.

**The median-time-past is the honest figure**, being monotonic and
consensus-enforced. For this block it is `2026-08-22T23:22:33Z`, **94 minutes
behind** the header's own stamp. So the legend states a bound, *"input existed
not before <MTP>"*, never an exact time.

> Both timestamp rules are consensus behaviour quoted from general knowledge, not
> read out of Bitcoin Core here. **Confirm them against the source before
> implementation.**

**How to convey it.** `5.403 x 10^23` is unreadable. Three framings, in
descending order of durability:

1. **Network-time equivalent** — *"about 10 minutes of the entire Bitcoin
   network at the difficulty of the day"*. Self-contained, since that is what
   difficulty retargeting means, and immediately intuitive.
2. **Expected hashes** — exact and permanent, but meaningless to most readers.
   Show it as the precise backing for (1), not as the headline.
3. **Economic** — the block's own revenue, which the node reports: subsidy
   3.1250 BTC + fees 0.0110 BTC = **3.1360 BTC**. Under competition, miner
   revenue approximates the cost of production. **Quote BTC, never fiat**: a
   dollar figure engraved in 2026 is misinformation by 2040.

**Rejected: the energy framing.** *"Roughly $X of electricity"* needs a J/TH
figure and an electricity price, both of which go stale and neither of which is
in the header. It is the most intuitive framing and the least durable, so it may
appear on screen but must never be engraved.

**Where each belongs.** The cost figures are a **decision aid at encode time**,
shown on screen so the operator knows how good the evidence is before committing
to steel. The plate carries only the durable anchor — block height, block hash,
and the MTP bound — because a future recoverer with a node recomputes every one
of these numbers from the hash alone.

The middle tier matters because it keeps `mt` usable offline, which is the
constellation's whole posture. What it cannot tell you is whether the outpoint
is still **unspent** — only a chain view can — so a transaction built this way
may be perfectly valid and still unbroadcastable because someone spent an input.
That is the same silent-invalidation hazard as §7, and it is disclosed the same
way: the input outpoints go on the plate so a holder can check.

## 7. Threat model

An `mt` plate is unlike every other plate in the constellation. `md1` and `mk1`
are watch-only public material: losing one costs privacy, not money. `ms1` is a
secret, and `me` refuses to push it over NFC at all. **An `mt` plate is
spendable by whoever holds it.** In hazard terms it sits nearer `ms1` than
`md1`, and the existing tooling's assumption that "public string" means "safe to
engrave" does not hold here.

| hazard | mitigation |
| --- | --- |
| **Bearer** — holder can broadcast | required future locktime (§8) bounds it in *time*, not in space; legend states it plainly |
| **Pinned destination** — a 2040 recoverer pays a 2026 address whose keys may be lost | cannot be fixed; legend names the destination so the operator sees what they commit to |
| **Pinned fee** — a 2026 fee rate may be unbroadcastable in 2040 | cannot be fixed; legend states rate and date so staleness is visible |
| **Silent invalidation** — one ordinary spend of any input voids the plate, and nothing on it says so | legend carries the input outpoints so a holder can check they are still unspent |

## 8. Refusals

All are machine-checkable before a single plate is cut.

1. **Not finalized** → refuse. Every input must carry a witness or scriptSig. An
   unbroadcastable transaction on steel costs ~21 minutes per plate for a dud.
2. **Script-invalid** → refuse, when prevouts are supplied. Real
   libbitcoinconsensus verification: `bitcoin` 0.32.101 ships the
   `bitcoinconsensus` feature and `consensus/validation.rs`.
3. **An unsigned transaction offered for ENGRAVING** → refuse. It cannot be
   broadcast, so it is not a backup. Present it instead (§1a).
3a. **Input amounts asserted without proof** → refuse unless
   `--i-certify-amounts` is given (§6a). Self-certified amounts are verified
   against themselves.
3b. **`gettxout` returns `null` for any input** → refuse, **no override**. The
   output is spent or never existed, and a node said so authoritatively (§6b).
4. **Broadcastable today** → refuse **by default**. The use case is dormant
   recovery at a future height; a transaction that can be broadcast now should
   be broadcast, not engraved. `--allow-immediate` overrides, and the legend
   then reads `IMMEDIATELY SPENDABLE` instead of the locktime line.
5. **Over the plate budget** → refuse, naming the exact plate count and what
   would fit.
6. **Module below 0.60 mm** → refuse until the F-234 optical test plate exists.
7. **Secrets** → refuse, as `me` already does for `ms1`.

Every refusal names the number that caused it. A refusal that says only "too
large" costs the operator a round trip.

## 9. Out of scope for v0.1

Signing; broadcasting; RBF or CPFP; watching the chain to detect invalidation;
any machine-readable provenance (ruled: legend only); sealed or encrypted
plates.

**Coin selection and fee estimation are deliberately excluded even though
producing is in scope.** `mt` builds a transaction from inputs and outputs it is
*given*; it does not choose which UTXOs to spend or what fee rate is
appropriate. Those are wallet decisions with their own failure modes, and
folding them in would make `mt` a wallet.


## 10. Open questions

1. **UR fragment overhead is unmeasured** (§3). This gates every plate count on
   the UR path.
2. **The F-234 optical test plate has not been cut.** It gates §4's module
   floor, and should test 0.30/0.45/0.60/0.90 mm modules *and* raw-vs-base45-vs-
   UR payloads in one cycle.
3. **How does a recoverer learn the fountain parameters?** UR carries seqNum and
   seqLen, but a fountain-coded set needs the decoder to know when it has
   enough. Confirm the vendored `Decoder` reports this, rather than assuming it.
4. **Does `mt` verify the transaction against the source wallet** when both the
   md1 card and prevouts are supplied — i.e. can it prove the stub is honest at
   encode time, even though nothing may branch on it at decode time?
5. **Plate legend text budget.** `PLATE_TEXT_BUDGET = 300` characters
   (`me-cli/src/lib.rs:48`) and §5 lists nine fields, several of them long
   (txid is 64 characters, each outpoint 66). This may not fit, and the legend
   competes with the QR for plate area. **Measure before speccing the layout.**

## 11. Provenance of the numbers

Everything measured is in `design/measurements/`, with the probe sources:
transaction sizes from real signed transactions (built, signed, finalized,
extracted, serialised — never estimated); QR capacities gated against the
published v40 limits at every mode and ECC level, a gate that caught three
wrong payload constructions before these numbers were trusted; plate and module
constants read from the fork (`backup/backup.go:45,99-102`,
`cmd/controller/platform_sh2.go:188`).
