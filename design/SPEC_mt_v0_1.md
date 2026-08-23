# SPEC — `mt`, the mnemonic-transaction format (v0.1 draft)

Status: **DRAFT, in R0.** Round 0 ran four independent lenses and did not close;
this is the fold responding to it. No code may be written against this until a
re-review closes it at 0 Critical / 0 Important. This is risk-set work by the
project's own definition — it touches funds, addresses and a new normative
format — so the gate is not optional.

Written 2026-08-22 from a brainstorm with the operator; folded 2026-08-23 after
R0 round 0. Every number in it was measured; the probes and raw results are in
`design/measurements/`, and the reproduce path is a command, not a memory.

---

## 0. What this is, in one paragraph

`mt` engraves a **signed Bitcoin transaction** on steel. It is handed a
transaction that is already built, already signed and already finalized, and it
renders it in one of **two forms**, because two different people engrave two
different ways:

| verb | form | engraved how | payload | size limit |
| --- | --- | --- | --- | --- |
| **`mt qr`** | QR symbols + legend | **machine** (SeedHammer II) | `ur:psbt` (§3) | the plate budget |
| **`mt string`** | `mt1` chunked codex32 | **by hand**, or machine | raw signed transaction (§3b) | **64 chunks** |

`mt qr` decides how many symbols that takes, at what error-correction level,
across how many plates, and what is engraved beside them. `mt string` emits a
character string with **BCH error correction**, so a hand engraver who cuts a
character wrong can still recover the transaction.

That is the whole of it. It does not build transactions, does not sign, holds no
private key, does not choose which UTXOs to spend or what fee to pay, and does
not invent an encoding for the transaction itself.

> **Scope ruling, operator, 2026-08-23 — transaction construction is removed.**
> An earlier draft had three verbs: produce, present, engrave. **Produce and
> present are gone; engrave split in two.** *"Constructing transactions is really a wallet function, and we
> want users to test their wallets in wallet software before going through all
> the work to hand engrave or machine engrave durable backups."* Presenting a
> PSBT to a signing device falls to the same argument — the wallet already does
> it, and doing it here would add a second path that nobody exercises on the way
> to a plate.
>
> This deleted the section the spec itself called *"the most dangerous section in
> this spec"* (the four-tier input-amount trust ladder), and with it two of the
> Criticals R0 round 0 found. **A tool that never builds a transaction can never
> get an input amount wrong.**

**What that shifts, and it is not a pure simplification.** `mt` is now a *pure
receiver* of transactions built elsewhere. Everything it can still get wrong is
a failure to inspect what it was handed, so §8 — the refusals — carries the
entire safety argument. It gets stricter in this fold, not looser.

## 1. The operator's decisions, recorded

Each of these is a ruling, with the reasoning that produced it. Several
overturned an earlier assumption and are marked.

1. **Two verbs, both engraving: `qr` and `string`.** Signed, finalized
   transactions only. Transaction construction and PSBT presentation are wallet
   functions and are out of scope (§9). **This overrules the previous draft's
   produce/present/engrave triple**, which split on *stage of the transaction*;
   these two split on *how the steel is cut*.
1b. **`mt string` exists so that short transactions can be HAND engraved, with
   fault tolerance.** Operator ruling 2026-08-23: *"For some shorter
   transactions, users will want codex32 style fault tolerant hand engraving."*
   Without it, the only route onto steel is a machine — which makes `mt`
   unusable for anyone without a SeedHammer, and gives up the human-readable,
   error-correcting property the rest of the constellation is built on.
2. **Its own repository**, `mnemonic-transaction`, with `mt-codec` and an `mt`
   CLI — not a subcommand of `me`. **This overrules the recommendation in
   §Section 1 of the brainstorm**, which argued `mt` had no wire format left to
   define and belonged next to `me bundle`. See §2 for what the codec does in
   fact specify; the objection was answered rather than ignored.
3. **The QR carries the standard form, never a codex32 string** (F-234).
4. **The envelope is `ur:psbt`, carrying a fully finalized PSBT.** **This
   overrules the previous draft's `ur:bytes`**, which R0 found is forbidden for
   production use by BCR-2020-005 itself. See §3.
5. **Reed-Solomon density is the highest that still minimises plate count.**
6. **Provenance rides in the engraved legend, not in the wire format.**
7. **The operator chooses between a timelocked and an immediately-spendable
   transaction**, with a loud warning on the second. **This replaces the
   previous draft's "future locktime required by default"**, which R0 found was
   unenforceable as written. See §8.
8. **Redundancy is zero. `mt` protects against damage to a plate, not against a
   missing plate.** The operator is free to engrave duplicate copies. **This
   closes §10.6, the previous draft's largest open question.** See §3.

## 2. What `mt-codec` actually specifies

The payload is a Bitcoin transaction, specified by Bitcoin. The container is a
QR, specified by ISO/IEC 18004. The envelope is UR, specified by Blockchain
Commons. None of the three needs `mt`. What is unspecified — and what this codec
is for — is everything between them:

- how one transaction maps onto **one or more** QR symbols, and onto plates;
- how a recoverer reassembles them, and how they know a fragment is missing;
- which (module size, QR version, ECC level, tiling) configuration is chosen for
  a given transaction, **deterministically and with every tie broken**, so two
  encoders agree;
- what is engraved **beside** the symbols, so the plate is self-describing;
- what `mt` refuses to engrave at all.

...and, for `mt string`, **the string format itself**: an `mt1` HRP, the chunk
header, and the BCH checksum that makes hand engraving fault-tolerant (§3b).

> **CORRECTION — the previous draft said the opposite, and it is worth saying
> why it was wrong.** It read: *"It is a plate format rather than a string
> format, which is why it has no bech32 HRP and no BCH checksum."* Adding
> `mt string` falsifies that sentence outright. `mt-codec` now defines a bech32
> string format with an HRP and a BCH checksum, exactly like `md-codec` and
> `mk-codec`.
>
> This **strengthens decision 2** rather than embarrassing it. The brainstorm's
> objection was that `mt` had no wire format left to define and belonged as an
> `me` subcommand. That objection is now decisively answered: `mt` defines a
> normative string encoding, which is precisely what earns a codec crate of its
> own in this constellation.

## 3. The envelope: `ur:psbt`

Fragmentation uses **UR (Uniform Resources, BCR-2020-005)**, type **`psbt`**.
The payload is a **fully finalized PSBT**, from which any standard tool extracts
the identical raw signed transaction.

> **CORRECTION — the previous draft specified `ur:bytes`, and that was wrong.**
> R0 lens 3 read BCR-2020-005 directly. Its "Types" section states:
>
> > The `bytes` type exists only for testing and validation of UR
> > implementations and **MUST NOT** be used for any other purpose.
>
> RFC-2119 keywords are declared normative in that same document. I then
> enumerated the companion registry BCR-2020-006 — **58 rows** — and the
> complete Bitcoin set is `seed`, `hdkey`, `keypath`, `coin-info`, `eckey`,
> `address`, `output-descriptor`, `sskr`, `psbt` and `account-descriptor`.
> **There is no registered UR type for a raw signed transaction.** `psbt` is the
> only transaction-shaped entry, and it requires a valid BIP-174 PSBT.
>
> So the compliant envelope and "raw signed transaction support" are the same
> artifact, reached through `psbt`.

**"Fully finalized" is exactly what `mt` already requires.** A PSBT holds a
transaction at any stage. Signatures accumulate as loose items in a scratch area
(`PSBT_IN_PARTIAL_SIG`); a *finalizer* then assembles them into the real witness
stack (`PSBT_IN_FINAL_SCRIPTWITNESS` / `PSBT_IN_FINAL_SCRIPTSIG`), which is the
only form the network understands. Three states matter:

| state | signatures live in | broadcastable? | `mt` engraves it? |
| --- | --- | --- | --- |
| partially signed | scratch, some inputs missing | no | **refused** |
| fully signed, not finalized | scratch, all inputs present | **no** | **refused** |
| **finalized** | assembled witness stack | **yes** | **yes** |

The middle state is not bureaucracy: assembling the witness stack requires
understanding the *script*. A P2WSH multisig witness must open with an empty
element — the `OP_CHECKMULTISIG` off-by-one workaround — and the signatures must
follow the pubkey order in the script. For the RCW the finalizer must also
decide **which of four tiers** is being satisfied, which is why `rcw.rs`
withholds keys per scenario and why `finalize_mut` can fail outright.

> **A hazard this envelope CREATES, recorded because the compliance fix is what
> introduced it.** A raw transaction cannot represent an unsigned one: if it
> serializes with witnesses, it is finished, and the format makes the mistake
> impossible. A PSBT can represent all three states above. So "is it finalized?"
> moves from a property physics enforced to a check `mt` must perform and must
> never skip. §8.1 is that check, and it is why §8.1 is stated first.

**What the payload is, exactly — the MIN form.** Measured in
`RESULTS_psbt_envelope_2026-08-23.txt`, three forms of a finalized PSBT, each
**asserted** to extract to a byte-identical transaction rather than assumed to:

| form | what it is | standard `extract_tx()` works? |
| --- | --- | --- |
| full | what BIP-174's finalizer leaves, untouched | yes |
| **MIN** | UTXO records kept, **output maps cleared** | **yes — this is the payload** |
| lean | UTXO records also stripped | **no** |

The `lean` row is a measured negative that corrects an assumption of mine:
`rust-bitcoin`'s `extract_tx()` runs a fee check needing each input's value, so
a UTXO-stripped PSBT fails with `MissingInputValue`. The bytes are all present
and `extract_tx_unchecked_fee_rate()` returns the right transaction, but the
*safe* API a recoverer reaches for refuses it — the wrong property for a plate
read in 2040.

**Clearing the output maps is what matters.** The change output's descriptor
metadata is what an updater wrote for a signer; BIP-174's finalizer strips only
*input* fields, so it rides along unless told otherwise. On two-output artifacts
it is the entire cost: `tr` tier 1, 1-in/2-out is **+1202 B** as finalized and
**+61 B** with the output maps cleared. Measured overhead of MIN over the raw
transaction: **+58 to +61 bytes at one input, +261 to +271 at five** (~54 B per
input).

**Why UR at all, now that redundancy is zero.** Decision 8 removed the fountain,
so every emitted part is a pure singleton — which is what
`bc/fountain/fountain.go:242` already returns for `seqNum <= seqLen`. What UR
still buys is a **type tag** and **fragmentation**, and for multi-symbol
artifacts there is no alternative that satisfies F-234: QR's own Structured
Append is unavailable in practice (the `qrcode` Rust crate knows it only as a
mode indicator with no encoder; the fork has none; wallets do not read it), and
an `mt`-specific envelope would reintroduce exactly the dependency F-234 exists
to remove. UR is the only fragmentation the Bitcoin ecosystem implements.

It is also cheap to adopt: the fork's encoder takes the type as a parameter and
its one live use is `ur:crypto-output` (`bc/ur/ur.go:111`) — an on-label,
*registered* type, so `ur:bytes` was the anomaly rather than the precedent. The
`psbt` type is a plain CBOR byte string, so wrapping costs 2–5 bytes.

**Redundancy is zero — ruling, operator, 2026-08-23.** Emitting exactly `seqLen`
parts costs least and tolerates no lost symbol. That is deliberate: **`mt`
protects against damage to a plate, which is what error correction does, and not
against a missing plate, which is what a duplicate plate does.** The operator is
free to engrave copies. Two consequences follow and are load-bearing:

- `PLATE n OF m` stays honest, because all `m` really are required. Any
  redundancy above zero would have made it misleading — a holder would need `k`
  of `m` and the plate would not say so.
- §4 is right to spend leftover capacity on ECC. ECC addresses marks, scratches
  and corrosion on a symbol, which is the failure this artifact is being
  hardened against.

**Its cost, measured.** Bytewords minimal is exactly 2 characters per byte plus
an 8-character CRC32 (`bc/bytewords/bytewords.go:17-31`, read from source).
Uppercased, `ur:psbt/N-M/…` is fully QR-alphanumeric — `:` and `/` are both in
the alphanumeric set — so it costs **11 bits per payload byte against raw
binary's 8**, a 37.5% expansion.

**Per-fragment overhead, measured** (`RESULTS_ur_overhead_2026-08-22.txt`). A
multi-part fragment is a 5-element CBOR array (`cbor:",toarray"`,
`bc/fountain/fountain.go:74-80`) of SeqNum, SeqLen, MessageLen, Checksum and
Data, deterministically encoded, then bytewords-encoded and prefixed. Read from
the fork's source, not the BCR paper:

| component | cost |
| --- | --- |
| CBOR array head + 4 scalars | **12–14 bytes**, by message size |
| Data payload | `ceil(messageLen / seqLen)`, identical in every part |
| bytewords | 2 chars per byte, **+ 8 chars** of CRC32 |
| `ur:psbt/<n>-<m>/` prefix | ~12 characters |

So each fragment costs about **49 characters** of overhead beyond its share of
the payload. A **single-part** UR skips the fountain wrapper entirely
(`bc/ur/ur.go:118`) and pays only the prefix and CRC — and therefore carries
**no MessageLen and no Checksum**, which §10.3 records as a trap for the
recoverer's tooling.

## 3b. The string form: `mt1`, for hand engraving

**`mt string` emits a chunked codex32 string with BCH error correction**, in the
same string layer `md1` and `mk1` already use. This is the constellation-native
form: human-readable, hand-engravable, and — the point — **fault tolerant**.

**The machinery exists and is proven; `mt1` is a new payload in it, not a new
codec.** `md-codec` ships a syndrome-based BCH *corrector*, not merely a
detector: `decode_with_correction` and `CorrectionDetail` in
`md-codec/src/lib.rs:48`, Berlekamp–Massey over `GF(1024)` in
`md-codec/src/bch_decode.rs`, on the `BCH(93,80,8)` regular-code variant of
BIP-93. A hand engraver who cuts a character wrong gets it corrected rather than
discovering years later that the plate is scrap.

**The payload is the raw signed transaction, NOT the PSBT — deliberately, and
for a different reason than §3.** F-234 binds the *QR*, because the QR is the
escape hatch for a recoverer holding no `mt`-aware software; it must therefore
carry a form the wider ecosystem reads, which is `ur:psbt`. An `mt1` string is
the opposite case: **nothing but `mt`-aware software will ever parse it**, so
F-234's argument does not apply and size is what matters. Dropping the PSBT
wrapper saves the **+58 to +61 bytes per input** measured in §3 — which at 5 bits
per character is real engraving time by hand.

### What fits

Measured (`RESULTS_envelope_2026-08-22.txt`, `RESULTS_rcw_2026-08-22.txt`),
against the **64-chunk** container:

| artifact | raw bytes | chunks | fits? |
| --- | --- | --- | --- |
| RCW `tr` key-path, 1-in/1-out | 162 | **4** | yes |
| RCW `tr` tier 4, 1-in/1-out | 405 | **9** | yes |
| RCW `tr` tier 1, 1-in/1-out | 535 | **12** | yes |
| RCW `wsh` tier 1, 1-in/1-out | 742 | **17** | yes |
| RCW `tr` tier 1, 5-in/2-out | 2498 | **56** | yes, barely |
| RCW `wsh` tier 1, 5-in/2-out | 3538 | **78** | **NO — refused** |

**The 64-chunk ceiling is a hard limit `mt qr` does not have**, and one real
wallet already exceeds it. That is the size asymmetry the two verbs exist to
span: `mt string` for short transactions, `mt qr` for anything.

The theoretical ceiling is **2,904 B** (64 chunks x (80 data symbols − 37 header
bits)). Treat the chunk counts above as a **floor**: `md`'s chunker balances
rather than fills, so a real chunk measures ~85 characters where a filled one
would be 96. **A new `mt1` codec could choose to fill**, which would raise the
ceiling — undecided, §10.12.

### How many chunks fit a plate — resolved from the fork's real geometry

The constellation convention is one string per plate (F-225), which would read
the table above as *"12 chunks = 12 plates"*. **That convention comes from
machine engraving and does not bind a hand engraver.** The fork's own text grid
says so.

`CharsPerLine` is `(plateSize − 2·outerMargin) / fixedCharWidth` and
`LinesPerPlate` is `(plateSize − 2·outerMargin) / fontMM`
(`backup/backup.go:87-97`), over the six-rung ladder `FontSizes`
(`backup/backup.go:82`). That yields:

| font | chars/line | lines/plate | **chars/plate** | **~85-char chunks/plate** |
| --- | --- | --- | --- | --- |
| 6.0 mm | 22 | 13 | 286 | **3** |
| 5.0 mm | 26 | 15 | 390 | **4** |
| 4.4 mm | 30 | 17 | 510 | **6** |
| 3.8 mm | 34 | 20 | 680 | **8** |
| 3.4 mm | 38 | 23 | 874 | **10** |
| 3.0 mm | 44 | 26 | 1144 | **13** |

So `mt string` plate counts, against `mt qr` for the same transaction:

| artifact | chunks | plates @6.0 mm | plates @3.8 mm | `mt qr` plates |
| --- | --- | --- | --- | --- |
| RCW `tr` key-path, 1-in | 4 | 2 | **1** | 1 |
| RCW `tr` tier 4, 1-in | 9 | 3 | **2** | 2 |
| RCW `tr` tier 1, 1-in | 12 | 4 | **2** | 2 |
| RCW `wsh` tier 1, 1-in | 17 | 6 | **3** | 2 |
| RCW `tr` tier 1, 5-in | 56 | 19 | **7** | 5 |

**This vindicates scoping the verb to short transactions and refutes the "4–6x
worse" reading.** At a middling font a short transaction costs the same one or
two plates as the QR form, while being human-readable and BCH-correctable. It
degrades sharply with size — 19 plates at 5 inputs and the largest font — which
is exactly why §8.7b refuses past the 64-chunk ceiling and points at `mt qr`.

> **PROVENANCE, and its one weakness.** The six rungs above are pinned by the
> fork's own `TestFontSizeLadder` (`backup/sizes_test.go:29-56`), whose comment
> calls them *"the basis of every capacity number in the spec"*. **Go is not
> installed on this machine, so that test was NOT executed here.** What I have
> is two agreeing derivations: the fork's committed pins, and my own arithmetic
> from the source formulas above, which reproduces all six rungs exactly. Two
> agreeing derivations are not an execution. **Run `go test ./backup/ -run
> TestFontSizeLadder` before this table is relied on** (§10.11).
>
> Two further limits: these are **unobstructed** lines — `CharsPerLine`'s own
> comment notes that lines crossing a screw-hole band hold fewer, per the
> `widthAt` predicate in `fit.go` — so every figure is an **upper bound**. And
> whether a legend is engraved beside a hand-cut string, consuming some of this
> budget, is undecided (§10.11).

> **This also casts doubt on §5's legend budget, and the doubt is not resolved
> here.** `legend.rs` takes `CHARS_PER_LINE = 35.0` and `LINES_FULL_PLATE = 20.0`
> from a **doc comment** in `crates/me-cli/src/lib.rs`, not from the fork's font
> metrics — and this project's own rule forbids describing code from its doc
> comment. Those two numbers correspond to the **3.8 mm rung** (34 chars, 20
> lines), one of six, treated as if universal. §4's 4.25 mm line pitch is
> `85/20`, which uses the **full** plate height where §4 uses 79 mm everywhere
> else, and 4.25 mm is not a rung of `FontSizes` at all. The nearest real rungs
> put 6 lines at 26.4 mm (4.4 mm) or 22.8 mm (3.8 mm) against §4's 25.5 mm.
> **The magnitude is small — under a millimetre — but §4's whole plate table
> rests on it.** Filed as §10.14.

## 4. Choosing the configuration — `mt qr` only

**This section governs `mt qr` and nothing else.** `mt string`'s layout is
undecided and is §10.10.

> **Rule (operator, 2026-08-22): the Reed-Solomon density is the highest that
> minimises plate count.**

Plate count is the real cost — ~21 minutes per plate (F-225) — so the search
minimises plates first and spends every leftover byte on error correction. Never
trade a plate for redundancy; never leave redundancy unbought.

    search space:  module size x QR version (1..40) x ECC (L,M,Q,H)
                   x rectangular tiling (across x rows)
    objective:     1. minimise plates   <- a plate holds the QR(s) AND the legend
                   2. maximise ECC
                   3. minimise symbol count
                   4. TIE-BREAK: maximise MODULE SIZE
                   5. then minimise QR version
    plate:         85 x 85 mm, outerMargin 3 mm => 79 mm usable
    quiet zone:    4 modules per side, per symbol
    legend:        6 lines reserved on plate 1 (25.5 mm at a 4.25 mm pitch),
                   1 line on every later plate for "PLATE n OF m"

> **Two corrections from R0 round 0, both in the objective above.**
>
> **The tiling is rectangular, not square.** The previous draft's search space
> said `k*k tiling` while the search that produced its own table returned 2-, 3-
> and 6-symbol configurations. None of those is a perfect square. The prose was
> wrong; `across x rows` is what the reference implementation does and what the
> numbers describe.
>
> **The objective was not a total order, and its implicit tie-break ran
> backwards.** Module size was in the search space and absent from the
> comparison key, and replacement was strict `<` against the incumbent, so ties
> resolved to whichever module the loop reached first — and the loop ascends
> from 0.30 mm. Ties therefore broke toward the **smallest and least legible**
> symbol, and would have broken toward the optically **unvalidated** 0.30 mm
> module once F-234 lifts the floor. Measured on the reference search: **4
> configurations tie** on (plates, ECC, symbols) for a 162 B payload at the
> 0.60 mm floor, and **41 tie** once the floor lifts. Steps 4 and 5 above make
> the order total, and make it break toward legibility, which is the direction
> the artifact's purpose demands.

**Measured, at the conservative 0.60 mm module, with the legend reserved, for
the actual `ur:psbt` payload** (`RESULTS_ecc_selection_2026-08-22.txt`):

| artifact | PSBT bytes | plates, symbols, version, ECC |
| --- | --- | --- |
| RCW `tr` tier 3, 1-in/1-out | 391 | **1 plate**, 1 qr, v16, ECC L |
| RCW `tr` tier 4, 1-in/1-out | 465 | **2 plates**, 1 qr, v24, ECC Q |
| RCW `tr` tier 1, 1-in/1-out | 595 | **2 plates**, 1 qr, v23, ECC M |
| RCW `wsh` tier 3, 1-in/1-out | 626 | **2 plates**, 1 qr, v24, ECC M |
| RCW `wsh` tier 1, 1-in/1-out | 802 | **2 plates**, 1 qr, v24, ECC L |
| RCW `tr` tier 1, 5-in/2-out | 2769 | **5 plates**, 4 qr, v22, ECC L |
| RCW `wsh` tier 1, 5-in/2-out | 3809 | **6 plates**, 5 qr, v23, ECC L |

> **This table replaces the previous draft's, which described the wrong
> payload.** That one was computed for raw transactions under `ur:bytes`. These
> are the finalized-PSBT sizes under `ur:psbt`, which is what will actually be
> engraved. The envelope change costs **one extra plate on three of seven
> artifacts and one ECC level on the other four** — the price of compliance,
> stated rather than buried.
>
> Ordinary-wallet comparisons (single-sig, 3-of-5, 9-of-11) are **not** in this
> table because their finalized-PSBT sizes have not been measured; only their
> raw-transaction sizes have. Filed as a follow-up rather than estimated.

**What the legend costs, stated plainly**: reserving 25.5 mm drops small
artifacts by two or three ECC levels and doubles the plate count on the larger
ones. The rule still degrades in the right order — H → Q → M → L before it gives
up a plate — and the smallest artifact still gets meaningful ECC free.

**Module size.** 0.30 mm is one engraved stroke: the theoretical floor and
**optically unvalidated**. Whether a camera reads 0.30 mm modules off brushed
steel is a hardware question, gated on the test plate in F-234. **Until that
plate exists, `mt` must not select a module below 0.60 mm** (two strokes). The
0.30 mm results are recorded for when it does.

## 5. The plate legend

Everything constellation-specific lives here, in engraved text, never in the QR.

**The legend carries only what a human needs BEFORE the QR is decoded.** Five
fields, 136 characters, 6 lines — measured,
`RESULTS_legend_budget_2026-08-22.txt`:

| field | chars | why |
| --- | --- | --- |
| `BEARER - ANYONE HOLDING THIS CAN SPEND IT` | 41 | the plate is spendable; this is not a backup in the sense the other formats are |
| `FROM WALLET <8 hex>` | 20 | the 4-byte policy-id stub. The transaction does **not** say what it spends *from* (§6) |
| `SPENDABLE AFTER BLOCK <n>` | 29 | the single most actionable fact: whether this plate is live yet. Reads `IMMEDIATELY SPENDABLE` when the operator chose that (§8.4) |
| `TO <truncated addr>  <amount>` | 34 | so a human sees where the money goes without a scanner |
| `PLATE n OF m` | 12 | a missing plate must be obvious, and all `m` are required (§3) |

### What was dropped, and why

The first draft listed ten fields and measured **474 characters at one input**,
growing to 1,066 at five — against a 300-character budget
(`crates/me-cli/src/lib.rs:48`). It could never have fitted.

Four fields were cut on one principle: **everything derivable from the decoded
transaction is duplication.** The txid, the input outpoints and the full
destination address are all *in* the transaction. Engraving them buys nothing —
and in the one case where it might seem to, an unreadable QR, the duplicate is
useless anyway because you still have no transaction to broadcast.

| dropped | recoverable how |
| --- | --- |
| txid | hash the decoded transaction |
| input outpoints | they are the transaction's inputs |
| fee rate and date | inputs − outputs, and the PSBT carries the input amounts |

> **§7's mitigations were written against the ten-field legend and were not
> re-read when it became five. R0 round 0 found this from two independent
> directions.** Four sections went on promising fields that no longer existed,
> and two of §7's four hazard mitigations named them. §7 below is corrected: it
> now claims only what §5 actually engraves. **A diff falsifies text it never
> touches** — the legend rewrite made those sentences false without editing
> them.

**The stub is a hint, never an authority.** It is the top 4 bytes of a canonical
md1 identity, form-aware — WalletPolicyId for a keyed wallet, the key-stable
WalletDescriptorTemplateId for a keyless template — reusing `mk1`'s existing
derivation — all three citations below are in the **`mnemonic-key` repo**, not
this one, so `plan-cite-check.sh` cannot resolve them and they were checked by
hand: `POLICY_ID_STUB_BYTES = 4` at `mk-codec/src/consts.rs:60`, the form-aware
rule documented at `mk-codec/src/key_card.rs:25-33`, and the derivation
`derive_stub_from_md1_card` at `mk-cli/src/cmd/mod.rs:126`. So one convention
spans the constellation. If the legend says wallet X and the transaction spends
wallet Y's UTXOs, **the transaction wins.** The stub exists to help a human find
the right plates, not to validate anything, and nothing may branch on it.

**Where the stub comes from is unspecified, and that is an open question**, not
a settled design: `FROM WALLET` is a mandatory field sized into §4's
reservation, and nothing says what supplies it or what happens when it is
absent. See §10.4.

## 6. Why provenance is asymmetric

**"Goes to" is already in the transaction.** Outputs carry scriptPubKeys; any
standard decoder yields addresses and amounts. Encoding destinations into the
wire format would create a second source of truth that can disagree with the
transaction — and on disagreement a recoverer would have to guess which to
believe. That is a funds-safety hazard, not a feature. It is displayed, never
encoded.

**"Comes from" is partly absent.** A signed transaction references inputs as
outpoints only; the source scriptPubKeys live in the *previous* transactions.
Without them you cannot tell which wallet it spends. The finalized PSBT closes
part of this by carrying each input's UTXO record — value and scriptPubKey — so
the engraved payload does describe what it spends, which the bare raw
transaction did not. It still does not name the *wallet*, hence the stub, and
hence the stub living in text, because it is the one constellation-specific fact
on the plate and F-234 forbids that inside the QR.

> **Do not lean on the PSBT's input amounts as trusted without a rule.** For
> segwit inputs they are committed to by the signature (BIP-341 `sha_amounts`,
> BIP-143), so they are cryptographically bound. **For legacy inputs nothing
> commits to them at all** — R0 lens 2's finding, and it survives the scope cut
> even though the section it was filed against is gone. §8.6 is the rule.

### 6a. When `bitcoind` is reachable, check the inputs are still unspent

**Operator ruling 2026-08-22, rescoped 2026-08-23.** If a node is available,
`mt` resolves every input itself and the operator is asked for nothing.

This section used to source input *amounts* for transaction construction. `mt`
no longer constructs, and the amounts now arrive inside the payload, so its job
is narrower and sharper: **before you spend ~21 minutes a plate, is this
transaction still worth engraving?**

The call is **`gettxout <txid> <vout> false`**, verified against a live Core
v25.0.0 node. It is the right RPC for three reasons:

- it returns `value` and `scriptPubKey` together, so the PSBT's claimed UTXO
  records can be checked against the chain rather than trusted;
- **it answers unspentness in the same call**, because it queries the UTXO set
  rather than the chain. A spent or nonexistent output returns `null`;
- it needs **no `-txindex`**, unlike `getrawtransaction`.

`include_mempool` is passed **false** deliberately. The default is `true`, and
mempool state is the wrong basis for an artifact meant to sit in a drawer for
years.

> **Known limitation, from R0 lens 2, not yet resolved.** `false` also means a
> mempool-spent input reads as *unspent*, which is the opposite of the caution
> this section argues for. And a `null` cannot distinguish "already spent" from
> "this node is still syncing, or is on the wrong chain". §8.5 states the rule
> that follows; whether `mt` should additionally require the node to be out of
> IBD is **§10.5**.

## 7. Threat model

An `mt` plate is unlike every other plate in the constellation. `md1` and `mk1`
are watch-only public material: losing one costs privacy, not money. `ms1` is a
secret, and `me` refuses to push it over NFC at all. **An `mt` plate is
spendable by whoever holds it.** In hazard terms it sits nearer `ms1` than
`md1`, and the existing tooling's assumption that "public string" means "safe to
engrave" does not hold here.

**Every mitigation below names a field §5 actually engraves.** Where there is no
mitigation, the row says so instead of inventing one.

| hazard | mitigation |
| --- | --- |
| **Bearer** — holder can broadcast | a timelock bounds it in *time*, not in space, and only when §8.4's `nSequence` condition holds; the `BEARER` line states it plainly, and it is the first line on the plate |
| **Pinned destination** — a 2040 recoverer pays a 2026 address whose keys may be lost | **cannot be fixed**; the `TO` line names the destination so the operator sees what they commit to before cutting |
| **Pinned fee** — a 2026 fee rate may be unbroadcastable in 2040 | **cannot be fixed, and is NOT on the plate.** Fee rate and date were cut from the legend (§5). `mt` displays both at encode time so the operator can judge staleness *before* engraving; a holder in 2040 recovers them by decoding, since the PSBT carries the input amounts |
| **Silent invalidation** — one ordinary spend of any input voids the plate, and nothing on it says so | **not mitigated on the plate.** The input outpoints were cut from the legend (§5), so a holder cannot check unspentness from the plate alone — they must decode the QR first. `mt` checks it at encode time (§6a, §8.5); after that the hazard is open and undisclosed on steel |
| **Non-`ALL` sighash** — an input signed with `SIGHASH_NONE` or `SIGHASH_SINGLE` leaves outputs unbound, so a plate-holder can redirect the funds and the `TO` line becomes a lie | refused at encode time, §8.6 |

> The last two rows are the honest state of this design. R0 lens 2 found that the
> previous draft claimed the plate carried outpoints when §5 had removed them,
> which turned an *undisclosed* hazard into a *falsely mitigated* one. Recording
> "not mitigated" is worse-looking and more useful.

## 8. Refusals

**This section now carries the whole safety argument.** `mt` builds nothing, so
everything it can get wrong is a failure to inspect what it was handed. All are
machine-checkable before a single plate is cut. **Every refusal below binds BOTH
verbs** unless it names one — a hand-engraved plate is exactly as bearer, and
exactly as permanent, as a machine-engraved one.

1. **Not fully finalized** → refuse. Every input must carry a populated
   `PSBT_IN_FINAL_SCRIPTSIG` or `PSBT_IN_FINAL_SCRIPTWITNESS`. A PSBT can
   represent partially-signed and fully-signed-but-unfinalized states (§3), and
   neither is broadcastable. **The raw-transaction format made this impossible;
   the PSBT envelope makes it merely illegal, so this check is mandatory and may
   not be skipped or overridden.**
2. **Script-invalid** → refuse. Real libbitcoinconsensus verification: `bitcoin`
   0.32.101 ships the `bitcoinconsensus` feature and `consensus/validation.rs`
   (verified against the crates.io source). The finalized PSBT carries each
   input's UTXO record, so — unlike the previous draft, where this refusal was
   conditional on prevouts being supplied separately — **the data needed to run
   it always arrives with the payload.** A PSBT whose UTXO records are missing
   is refused under (1)'s sibling rule: `mt` requires the MIN form of §3.
3. **An unsigned or unfinalized transaction offered for engraving** → refuse. It
   cannot be broadcast, so it is not a backup.
4. **Locktime: the operator chooses, and the second choice warns loudly.**
   **Ruling, operator, 2026-08-23**, replacing the previous draft's "refuse
   anything broadcastable today":

   - `--timelocked` (default): `mt` verifies the transaction is **actually**
     timelocked, and the legend reads `SPENDABLE AFTER BLOCK <n>`.
   - `--immediate`: the operator accepts an immediately-spendable plate. The
     legend reads `IMMEDIATELY SPENDABLE`, and `mt` prints a prominent warning
     that a transaction without an enforced timelock **may be broadcast by
     anyone who holds or photographs the plate, from the moment it is cut.**

   **Verifying "actually timelocked" requires reading `nSequence`, and the
   previous draft never mentioned it.** `nLockTime` is enforced only when at
   least one input has `nSequence != 0xFFFFFFFF`. A transaction with all inputs
   final ignores its locktime entirely — so a plate could have satisfied the old
   "required future locktime" rule on paper and been spendable the moment it was
   cut. Under `--timelocked`, `mt` refuses unless **both** hold: `nLockTime` is
   in the future, **and** at least one input is non-final. The warning under
   `--immediate` says *might* be spendable, not *is*, because relay also depends
   on fee and on the inputs still being unspent.
5. **`gettxout` returns `null` for any input** → refuse, when a node is
   reachable. The output is spent or never existed. See §6a's limitation and
   §10.5 for the IBD case.
6. **Any input not signed with `SIGHASH_ALL` (or taproot's `SIGHASH_DEFAULT`)**
   → refuse. R0 lens 2's finding. A `SIGHASH_NONE` input leaves the outputs
   unbound, so a holder — or anyone who photographs the plate — can redirect the
   funds while the signature stays valid, and the legend's `TO` line becomes
   false. `SIGHASH_SINGLE` and `SIGHASH_ANYONECANPAY` are refused on the same
   grounds. **Additionally, any legacy (non-segwit) input is refused**, because
   nothing in a legacy sighash commits to the input amount, so the PSBT's UTXO
   record for it is unverifiable (§6).
7. **Over the plate budget (`mt qr`)** → refuse, naming the exact plate count
   and what would fit.
7b. **Over the 64-chunk container (`mt string`)** → refuse, naming the chunk
   count and the ceiling, and pointing at `mt qr`, which has no such limit. Real
   wallets hit this: RCW `wsh` tier 1 at 5 inputs needs 78 chunks (§3b).
8. **Module below 0.60 mm** → refuse until the F-234 optical test plate exists.
9. **Secrets** → refuse, as `me` already does for `ms1`.

Every refusal names the number that caused it. A refusal that says only "too
large" costs the operator a round trip.

## 9. Out of scope for v0.1

**Transaction construction, and PSBT presentation to a signing device** — both
removed by operator ruling 2026-08-23 (§0). Coin selection, fee estimation,
change handling and input selection go with them: they are wallet decisions with
their own failure modes, they are better tested in wallet software before
anything is engraved, and folding them in would make `mt` a wallet.

Also out: signing; broadcasting; RBF or CPFP; watching the chain to detect
invalidation after engraving; any machine-readable provenance (ruled: legend
only); sealed or encrypted plates; and Merkle inclusion proofs of input
existence (`gettxoutproof`) with the block-work and timestamp framings that went
with them — removed 2026-08-23, since they existed to establish trust in input
amounts offline for *construction*, and the amounts now arrive bound inside a
signed PSBT.

## 10. Open questions

1. **The F-234 optical test plate has not been cut.** It gates §4's module
   floor, and should test 0.30/0.45/0.60/0.90 mm modules *and*
   raw-vs-base45-vs-UR payloads in one cycle. It should now **also** answer
   items 2 and 3 below — one plate cycle, three answers.
2. **Will a wallet reassemble multi-part `ur:psbt` from STATIC symbols?** Every
   wallet that reads multi-part UR does so from an *animated* QR on a screen.
   `mt` engraves static symbols on separate plates, scanned minutes apart. R0
   lens 3 found a concrete report of a static multi-part `ur:bytes` QR failing
   to scan into Sparrow at all. **This is the single most load-bearing unverified
   assumption in the spec**: if it does not hold, no envelope satisfies F-234 for
   multi-plate transactions, and the recoverability premise holds only for
   single-symbol artifacts. Test before relying on it; §3 does not currently
   claim ecosystem readability for the multi-plate case.
3. **Is UR worth its 37.5% expansion for SINGLE-symbol artifacts?** There it
   buys only a type tag, and a PSBT is already self-identifying by its `psbt\xff`
   magic. Measured cost of that tag: one extra plate on RCW `tr` tier 4. Item 1's
   plate is already scoped to compare raw against UR payloads.
4. **Where does `FROM WALLET <8 hex>` come from?** It is a mandatory legend
   field sized into §4's reservation, and nothing specifies what supplies the
   md1 card, nor what the legend does when it is absent. Related: should `mt`
   verify the transaction against the source wallet when both are supplied?
   §5 forbids branching on the stub at decode time, so such a check can only
   warn — which makes it deferrable, but the *input* question is not.
5. **Should `mt` require the node to be out of IBD before trusting `gettxout`?**
   §8.5's refusal cannot currently distinguish "spent" from "this node does not
   know yet", and routing a tired operator around a false refusal is its own
   hazard.
6. ~~How much fountain redundancy?~~ **CLOSED**, operator ruling 2026-08-23:
   zero. `mt` protects against plate damage (ECC), not plate loss (duplicate
   plates, the operator's choice). See §3.
7. **Would back-side engraving recover the 25.5 mm?** It would restore ECC levels
   and reduce plate counts. But there is **no back-side path in the fork**:
   `backup/backup.go:247` defines `frontSideSeed`, called once at
   `backup/backup.go:134`, and there is a single `Engraving` per plate with
   nothing that engraves a reverse. Firmware work, not a free option.
8. ~~How does a recoverer learn the fountain parameters?~~ **ANSWERED** from
   source by R0 lens 4, and it is a gap rather than a question. Every multi-part
   UR carries `SeqLen`, `MessageLen` and `Checksum` in its CBOR
   (`bc/fountain/fountain.go:74-87`); `Result()` returns non-nil only at
   `len(completed) == SeqLen`; and `Progress()` is a `x1.75` UI heuristic that
   can reach 1.0 while `Result()` is still nil — so **nothing may gate on
   `Progress()`**. Two further traps: the URI prefix is parsed then discarded,
   and single-part URs skip the fountain wrapper entirely, carrying no
   MessageLen and no Checksum.
9. **How does the engraving reach the machine, and can the machine engrave what
   §4 selects?** R0 lens 4 and lens 1 both found this and it is the largest
   remaining gap: the spec stops at "choose a configuration" and never says how
   an engraving is produced or conveyed. The fork's only arbitrary-payload QR
   path is fixed at `freeTextQRScale = 2` (`backup/fit.go:19`) with a
   compile-time ECC level and one code per plate, and `sysw/record.go` has no
   transaction class. §4 may be selecting from a space the machine cannot reach.
   **This blocks implementation and must close before code.**
10. **There is no CLI surface.** Two verbs (`mt qr`, `mt string`) and two flags
    (`--timelocked` / `--immediate`) are now named, but nothing specifies the
    input convention (file? stdin? PSBT or raw hex or both?), the output
    convention, or the exit codes — and §8 promises *"every refusal names the
    number that caused it"*, which is an output contract with no format.
    **Blocks implementation.**
11. ~~How many codex32 characters fit a hand-engraved plate?~~ **ANSWERED in
    §3b** from the fork's text grid: 286 characters at the largest font rung to
    1144 at the smallest, i.e. 3 to 13 chunks per plate. Two residues remain:
    **(a)** Go is not installed here, so the fork's `TestFontSizeLadder` was
    never executed — run it before the table is relied on; **(b)** is a legend
    engraved beside a hand-cut string, and does §5's 6-line reservation apply to
    `mt string` at all? The plate counts in §3b assume it does not.
12. **Should `mt1` FILL its chunks rather than balance them?** `md`'s chunker
    balances, so a real chunk is ~85 characters against a filled 96. Filling
    would raise the 64-chunk ceiling meaningfully — possibly enough to bring
    RCW `wsh` tier 1 at 5 inputs (78 chunks) under it. But it diverges from the
    chunker every other constellation format uses, and **the Rust-primary rule
    means any such change lands in the Rust codec first, with test vectors.**
14. **§5's legend budget rests on a doc comment, not on the fork's font
    metrics.** `legend.rs` hardcodes `CHARS_PER_LINE = 35.0` /
    `LINES_FULL_PLATE = 20.0` "per `crates/me-cli/src/lib.rs:46`"; the fork's real ladder has six
    rungs and those two values are the 3.8 mm one. §4's 4.25 mm pitch is `85/20`
    — full plate height, where §4 uses 79 mm everywhere else — and is not a rung
    of `FontSizes`. Small in magnitude, but §4's plate table and §5's 6-line
    reservation both stand on it. **Re-derive both from `CharsPerLine` /
    `LinesPerPlate` and regenerate §4's table if they move.**

13. **Does `mt1` reuse the `md1` chunk header verbatim, or need its own?** §3b
    assumes the existing string layer takes a new payload type cleanly. That is
    an assumption about `md-codec`'s header (37 header bits, chunk-set id,
    ordering) and has not been checked against a transaction-shaped payload.

## 11. Provenance of the numbers

Everything measured is in `design/measurements/`, with the probe sources and a
reproduce path that is a command rather than a memory. Transaction sizes come
from real transactions — built, signed, finalized, extracted, serialised, never
estimated. QR capacities are gated against the published v40 limits at every
mode and ECC level, a gate that caught three wrong payload constructions before
these numbers were trusted. Plate and module constants are read from the fork
(`backup/backup.go:45,99-102`, `cmd/controller/platform_sh2.go:188`).

All twelve probe binaries were rebuilt and re-run on 2026-08-22 before R0, and
nine of eleven results files reproduced **byte-identically**; the two exceptions
differ only by capture artifacts, documented in `design/measurements/README.md`.
`psbtfinal.rs` and the PSBT section of `select.rs` were added 2026-08-23 for this
fold.

§3b's chunk counts come from `RESULTS_envelope_2026-08-22.txt` and
`RESULTS_rcw_2026-08-22.txt`, which measure the **raw signed transaction**
against the 64-chunk container. They are a floor, for the balancing reason
stated in §3b. The BCH corrector's existence was read from
`md-codec/src/bch_decode.rs` and `md-codec/src/lib.rs:48` in the
`descriptor-mnemonic` repo — a sibling, so `plan-cite-check.sh` has no root for
it and those two were checked by hand.

> The previous draft's §11 claimed *"everything measured is in
> `design/measurements/`"* while the §6c/§6d block figures had no results file
> behind them. Those sections are now out of scope (§9), so the claim is true
> again by subtraction rather than by generating the missing evidence.
