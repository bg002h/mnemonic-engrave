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
| **`mt qr`** | QR symbols + legend, as a **SH2 payload** | **machine** (SeedHammer II) | `mt1` chunks, **base45** (§3) | the plate budget |
| **`mt string`** | `mt1` chunked codex32, **on stdout** | **by hand** | raw signed transaction (§3b) | **64 chunks** |

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
4. **UR is dropped entirely. Both verbs share the `mt1` chunk header and NOTHING
   ELSE** — each medium carries the error correction native to it (§3a). The QR
   payload is **base45**.
   **This overrules the previous draft's `ur:psbt`, which itself overruled
   `ur:bytes`** — three positions in one cycle, and §3 records why each fell.
   The payload remains a fully finalized PSBT. See §3.
5. **Reed-Solomon density is the highest that still minimises plate count.**
6. **Provenance rides in the engraved legend, not in the wire format.**
7. **`mt` does not offer a locktime CHOICE. It reads the transaction and warns
   if the plate would be immediately spendable.** Operator ruling 2026-08-23:
   *"Timelocking happens by user at their wallet software. We do not create
   transactions. We merely read transaction and warn if immediate."* **This
   overrules the previous draft's `--timelocked` / `--immediate` flags**, which
   made `mt` a party to a decision it does not own. See §8.4.
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

## 3. The envelope: none — `mt1` chunks, both verbs

There is **no UR**, and no third-party envelope of any kind. Both verbs
fragment with the **`mt1` chunk header** (§3b), and `mt qr` puts the resulting
chunks into QR symbols directly.

> **CORRECTION, and this is the third envelope position in one cycle. All three
> are recorded because the reasoning matters more than the answer.**
>
> 1. **`ur:bytes`** — the original draft. R0 round 0 killed it: BCR-2020-005
>    states the `bytes` type *"exists only for testing and validation of UR
>    implementations and MUST NOT be used for any other purpose."*
> 2. **`ur:psbt`** — the compliant replacement, on the operator's *"don't go
>    off-label"* ruling. Correct as far as it went: the BCR-2020-006 registry has
>    58 rows and `psbt` is its only transaction-shaped type, so this was the one
>    conformant way to carry a transaction under UR.
> 3. **No UR at all** — operator ruling 2026-08-23: *"I don't think UR wrapper
>    complexity is worth it."*
>
> **What made (3) available was §10.2**, ruled the same day. UR's real defence
> was never conformance — it was that UR is the only fragmentation the Bitcoin
> ecosystem implements, so a recoverer could reassemble engraved symbols with
> off-the-shelf wallet software. Once `mt` ships its own static-scan reader, the
> ecosystem reassembles nothing and that defence is void.
>
> **What it costs, measured** (`RESULTS_ecc_selection_2026-08-22.txt`, at
> 0.60 mm, same finalized-PSBT artifacts): dropping UR **saves a whole plate on
> 3 of 7 artifacts** — `tr` tier 4 goes 2 plates to 1, `tr` tier 1 at 5 inputs
> goes 5 to 4, `wsh` tier 1 at 5 inputs goes 6 to 5 — **and buys one to two ECC
> levels on the other 4.** Under §1.8, which spends slack on damage tolerance,
> UR was spending the exact currency the artifact needs.
>
> **The efficiency numbers, which decide this on their own**
> (`RESULTS_qr_modes_2026-08-22.txt`, gated against published v40 limits):
> raw binary 100%, **base45 97%**, bech32 uppercase 91%, base64 75%, and **UR
> bytewords ~73%** — the same density as plain uppercase hex. UR was paying a
> 27% tax for a wrapper whose benefit §10.2 removed.

**Fragmentation: the `mt1` chunk header, for both verbs.** Operator ruling
2026-08-23. `md-codec`'s `ChunkHeader` carries `version`, a 20-bit
`chunk_set_id`, `count` and `index` — n-of-m **plus a set identifier**, so
symbols from two different transactions cannot be combined. That is strictly
stronger than UR, which has a payload checksum but no set identity, and it means
**one fragmentation scheme to specify, test, teach a recoverer, and get wrong
only once.**

    mt string:  mt1 chunk -> BCH + codex32 text -> engraved as characters
    mt qr:      mt1 chunk -> bytes              -> engraved as a QR symbol
                ^ identical header both ways

**Consequence: §10.13 now gates both verbs, not one.** Whether `md-codec`'s
header and reassembly take a transaction-shaped payload cleanly was already
open; it is now load-bearing for everything `mt` emits.

**What a symbol carries: `mt1` chunks, base45-encoded.** Operator ruling
2026-08-23. Three candidates were measured
(`RESULTS_ecc_selection_2026-08-22.txt`, `qr_payload_forms`), all carrying the
same chunk header:

| form | efficiency | worst plate cost |
| --- | --- | --- |
| codex32 string inside the QR | **63–65%** | **+2 plates** (`wsh` tier 1, 5-in: 5 → 7) |
| bytes + **base45** | **85.5–86%** | — **chosen** |
| bytes, raw binary | 88.4–88.8% | — |

**Why not base45's 3%-denser rival.** Binary is marginally smaller and produces
**identical plate counts in 4 of the 5 measured artifacts**, so the choice was
never about size. base45 (RFC 9285) wins on two other grounds: it is pure QR
alphanumeric text, so no scanner is asked to accept arbitrary bytes — a real
failure mode at the application layer even though QR byte mode is standard — and
it carries **intrinsic error detection**, because 45³ = 91,125 exceeds the 65,536
values two bytes can hold, so roughly **28% of corrupted 3-character groups are
detectably invalid**. Raw binary has none: every byte sequence is legal.

> **Why the codex32 string does NOT go in the QR, measured rather than argued.**
> Operator question: *"are you suggesting we first codex32 style encode the
> transaction and then qr encode that? Does that massively increase the plate
> count?"* It does. At **63–65%** it is **worse than UR's ~73%**, which §3 had
> already dropped for waste, and it costs one extra plate on two artifacts and
> **two** on `wsh` tier 1 at five inputs.
>
> It pays twice: a 65-bit BCH checksum per 40-byte chunk, and then bech32's five
> data bits per 5.5 character-bits through alphanumeric mode. That is far below
> the 91% a *bare* bech32 payload measures, because at 40 payload bytes per chunk
> the header, checksum and HRP are a large fraction of every chunk.

## 3a. The medium-appropriate ECC principle

**Each medium carries exactly one error-correction layer: the one native to it.**
This is the rule that rejected codex32-in-QR, and it generalises.

1. **One layer per medium**, chosen to match **how that medium physically
   fails**:

   | medium | fails as | native correction |
   | --- | --- | --- |
   | hand-engraved string | **per character** — a miscut stroke, a wrong glyph, one scratched letter | **BCH** over 5-bit symbols, `t = 4` per chunk |
   | machine-engraved QR | **per region** — a scratch across modules, corrosion, a dent | **Reed-Solomon** + QR codeword interleaving, which spreads a local blot across many RS blocks |

2. **Never stack them, because a redundant layer is paid for in the same
   currency as the native one: plate area.** §4's objective spends every leftover
   byte on ECC, so carrying BCH inside a QR does not add protection *on top of*
   Reed-Solomon — it buys BCH parity **with area that would otherwise have bought
   RS parity**, at a worse rate. Measured above: 64% against 88.8%, up to two
   extra plates, and a lower ECC level everywhere else. **Stacking made the
   artifact strictly less damage-tolerant**, which is the opposite of what a
   second checksum intuitively promises.

3. **What legitimately crosses both media is FRAMING, not correction.** The
   chunk header — version, `chunk_set_id`, `count`, `index` — is about identity
   and assembly, so it is shared verbatim. Damage is medium-specific; identity is
   not.

So the split is clean, in the operator's own words: **QR is for machine
engraving, codex32 is for hand engraving.**

    mt string:  chunk header + payload -> BCH + codex32 -> engraved characters
    mt qr:      chunk header + payload -> base45 -> QR (Reed-Solomon) -> modules
                ^ identical header, medium-appropriate correction

## 3b. The string form: `mt1`, for hand engraving

**`mt string` emits a chunked codex32 string with BCH error correction**, in the
same string layer `md1` and `mk1` already use. This is the constellation-native
form: human-readable, hand-engravable, and — the point — **fault tolerant**.

**The machinery exists and is proven; `mt1` is a new payload in it, not a new
codec.** `md-codec` ships a syndrome-based BCH *corrector*, not merely a
detector: `decode_with_correction` and `CorrectionDetail` in
`crates/md-codec/src/lib.rs:48`, Berlekamp–Massey over `GF(1024)` in
`crates/md-codec/src/bch_decode.rs`, on the `BCH(93,80,8)` regular-code variant of
BIP-93. A hand engraver who cuts a character wrong gets it corrected rather than
discovering years later that the plate is scrap.

**The payload is the raw signed transaction, NOT the PSBT — deliberately, and
for a different reason than §3.** F-234 binds the *QR*, because the QR is the
escape hatch for a recoverer holding no `mt`-aware software; it must therefore
carry a form the wider ecosystem might read. An `mt1` string is
the opposite case: **nothing but `mt`-aware software will ever parse it**, so
F-234's argument does not apply and size is what matters. Dropping the PSBT
wrapper saves the **+58 to +61 bytes per input** measured in §3 — which at 5 bits
per character is real engraving time by hand.

### What fits

A chunk carries **40 payload bytes**, and the container holds **64 chunks**, so
the hard ceiling is **2,560 B**. Measured
(`RESULTS_envelope_2026-08-22.txt`, `RESULTS_rcw_2026-08-22.txt`):

| artifact | raw bytes | chunks | fits? |
| --- | --- | --- | --- |
| RCW `tr` key-path, 1-in/1-out | 162 | **5** | yes |
| RCW `tr` tier 4, 1-in/1-out | 405 | **11** | yes |
| RCW `tr` tier 1, 1-in/1-out | 535 | **14** | yes |
| RCW `wsh` tier 1, 1-in/1-out | 742 | **19** | yes |
| RCW `tr` tier 1, 5-in/2-out | 2498 | **63** | yes, barely |
| RCW `wsh` tier 1, 5-in/2-out | 3538 | **89** | **NO — refused** |

**The 64-chunk ceiling is a hard limit `mt qr` does not have**, and one real
wallet already exceeds it. That is the size asymmetry the two verbs exist to
span: `mt string` for short transactions, `mt qr` for anything.

> **CORRECTION — every number above was ~13% low until 2026-08-23, and so was
> the ceiling.** R0 round 1 (S-1) found that the probe helper feeding all of
> them modelled a chunk as `(bytes*8).div_ceil(363)`. 363 = 80 codex32 symbols
> x 5 bits − 37 header bits, i.e. what a chunk *could* carry if the chunker
> **filled** to long-form capacity. **It does not.** `md-codec` sizes chunks by
> `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64 * 5 = 320` bits
> (`crates/md-codec/src/chunk.rs:224`), applied over `payload_bytes.len() * 8`
> (`crates/md-codec/src/chunk.rs:253-254`) — a flat **40 bytes per chunk**.
>
> The old model claimed 2,904 B where the real ceiling is **2,560 B**, leaving a
> **344-byte band** in which a transaction the table called "fits" would in fact
> return `ChunkCountExceedsMax`. That band *was* §8.7b's refusal boundary.
>
> The defect was **one shared helper replicated across seven probe binaries**,
> so every chunk count in every results file was wrong the same way — a corpus
> can be uniformly wrong and still look perfectly self-consistent. It is now the
> named constant `CHUNK_PAYLOAD_BITS = 320` carrying the citation and the
> history, and **all thirteen binaries were rebuilt and re-run**. Capacity
> conclusions moved: single-sig `tr` key-path from a 26-input ceiling to **23**,
> RCW `wsh` tier 1 from 4 inputs to **3**.
>
> The measurements README *did* label the old counts "a floor", which was the
> right hedge. §3b dropped the label and presented them as counts, and §8.7b
> refused against them. **The caveat existed and was lost in transit.**
> Whether `mt1` should instead FILL its chunks, raising the ceiling, is §10.12.

### Layout on steel is the user's, not `mt`'s

> **Scope ruling, operator, 2026-08-23.** *"How many codex32 characters fit a
> hand engraved plate? As many as a user wants. It is not our concern."*

**`mt string` emits a string. That is the whole of its output.** Font size,
characters per plate, how many plates, what order they are laid out in, whether
the string is cut by hand or by machine, and whether anything is engraved beside
it are all the user's decisions. This spec does not constrain any of them, and
§4's configuration search does not apply to this verb.

An earlier version of this section derived a chars-per-plate table from the
fork's font ladder and drew plate counts from it. **That was out of scope and is
deleted.** What survives from it is the one part that *is* a property of the
codec rather than of anyone's steel: the **64-chunk ceiling** above, which binds
regardless of how the string is engraved, and which §8.7b refuses against.

> **The distinction that decides what belongs here:** what `mt` *emits* is this
> spec's concern; what a user does with steel is not. `mt qr` is the exception
> only because it emits an engraving, so plate geometry is part of its output.

### The one thing `mt string` does say about the plate

> **Ruling, operator, 2026-08-23:** *"Hand cut plates get a warning on stderr.
> And that's it."*

`mt string` prints a warning at encode time that the artifact is **bearer** —
anyone holding the resulting plate can spend it — and takes no further interest
in the steel. It does not require a legend, does not reserve space for one, and
cannot verify that any warning reached the plate.

**On `stderr` specifically, and this is load-bearing rather than incidental.**
The `mt1` string goes to **stdout**, so the ordinary invocation pipes it to a
file or another tool. A warning on stdout would be captured by that redirection
and silently swallowed; on stderr it reaches the operator's terminal either way.
This is the first fixed point of §10.10's CLI contract: **stdout carries the
artifact, stderr carries everything the human must see.**

**The accepted risk, stated plainly rather than buried.** That warning is seen
**once**, by the person doing the encoding. The person holding the plate in 2040
is a different person, and the plate itself says nothing. This is a deliberate
asymmetry between the two verbs — `mt qr` engraves `BEARER - ANYONE HOLDING THIS
CAN SPEND IT` as the first line of a legend `mt` controls, and `mt string` has
no such mechanism because it emits no engraving. §7 records it as an accepted
risk, not as a mitigation.


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

**Measured, at the conservative 0.60 mm module, with the legend reserved**
(`RESULTS_ecc_selection_2026-08-22.txt`, the **RAW** column — the payload is now
PSBT bytes, not bytewords):

| artifact | PSBT bytes | plates, symbols, version, ECC |
| --- | --- | --- |
| RCW `tr` tier 3, 1-in/1-out | 391 | **1 plate**, 1 qr, v15, ECC M |
| RCW `tr` tier 4, 1-in/1-out | 465 | **1 plate**, 1 qr, v15, ECC L |
| RCW `tr` tier 1, 1-in/1-out | 595 | **2 plates**, 1 qr, v23, ECC Q |
| RCW `wsh` tier 3, 1-in/1-out | 626 | **2 plates**, 1 qr, v24, ECC Q |
| RCW `wsh` tier 1, 1-in/1-out | 802 | **2 plates**, 1 qr, v23, ECC M |
| RCW `tr` tier 1, 5-in/2-out | 2769 | **4 plates**, 3 qr, v21, ECC L |
| RCW `wsh` tier 1, 5-in/2-out | 3809 | **5 plates**, 4 qr, v22, ECC L |

> **This table has now been regenerated twice and is STILL provisional.** The
> first version described raw transactions under `ur:bytes`; the second,
> finalized PSBTs under `ur:psbt`; this one, finalized PSBTs with UR dropped
> (§3). Compare the second against this one for what UR cost: a plate on three
> of seven artifacts, one to two ECC levels on the rest.
>
> **Three inputs are still unmodelled here**, all of them additive, so treat
> every row as a lower bound: the **37-bit `mt1` chunk header per symbol**
> (§3), §10.8's **per-symbol `n/m` labels**, and §10.14's **font-metric
> correction** to the legend reservation. §10.14 already requires the
> regeneration; this note names all three inputs it must take.
>
> Ordinary-wallet comparisons (single-sig, 3-of-5, 9-of-11) are **not** here
> because their finalized-PSBT sizes have not been measured.

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
fields, **130 characters**, 6 lines — measured,
`RESULTS_legend_budget_2026-08-22.txt`:

| field | chars | why |
| --- | --- | --- |
| `BEARER - ANYONE HOLDING THIS CAN SPEND IT` | 41 | the plate is spendable; this is not a backup in the sense the other formats are |
| `FROM WALLET <8 hex>` | 20 | wallet id or seed fingerprint. The transaction does **not** say what it spends *from* (§6). **Optional — loudly warned when absent** (§10.4) |
| `LOCKED TO BLOCK <n>` | 23 | the single most actionable fact. Reads **`NO BLOCK TIMELOCK`** when there is no enforced `nLockTime`. **A statement about the transaction's fields, never about spendability** — `mt` does not evaluate scripts, so it reports the lock it read and lets the reader conclude (§8.4) |
| `TO <wallet id, fp or label>  <amount>` | 34 | names the destination **wallet**, not one truncated address — operator ruling, §10.4. **Optional — loudly warned when blank.** A free-text label is allowed **only behind an explicit flag**, since nothing can check it against the transaction |
| `PLATE n OF m` | 12 | a missing plate must be obvious, and all `m` are required (§3) |

Plus, **not part of the 136-character budget above**, one `n/m` label engraved
beside **each QR symbol**, naming the UR part it carries (§10.8's ruling). A
plate may hold several symbols, so `PLATE n OF m` alone cannot tell a recoverer
which *part* is missing. These labels are per-symbol and their area is not yet
reserved in §4 — see §10.8 and §10.14.

> **This budget rests on a DOC COMMENT, not on the fork's font metrics, and the
> doubt is not resolved here.** `legend.rs` hardcodes `CHARS_PER_LINE = 35.0`
> and `LINES_FULL_PLATE = 20.0` taken "per `crates/me-cli/src/lib.rs:46`" — and
> this project's own rule forbids describing code from its doc comment. The
> fork's real grid is `CharsPerLine = (plateSize − 2·outerMargin) /
> fixedCharWidth` and `LinesPerPlate = (plateSize − 2·outerMargin) / fontMM`
> (`backup/backup.go:87-97`) over a **six-rung** ladder `FontSizes`
> (`backup/backup.go:82`), pinned by the fork's own `TestFontSizeLadder`
> (`backup/sizes_test.go:29-56`) at 22/13, 26/15, 30/17, 34/20, 38/23 and 44/26
> characters-per-line / lines-per-plate. **`legend.rs`'s two values are the
> 3.8 mm rung of six, treated as universal.**
>
> §4's 4.25 mm line pitch compounds it: that is `85/20`, using the **full** plate
> height where §4 uses 79 mm everywhere else, and 4.25 mm is not a rung of
> `FontSizes` at all. The nearest real rungs put 6 lines at 26.4 mm (4.4 mm) or
> 22.8 mm (3.8 mm) against §4's 25.5 mm.
>
> Magnitude is under a millimetre, but **§4's entire plate table and this
> section's 6-line reservation both stand on it.** Filed as §10.14 rather than
> patched, because regenerating §4's table is a measurement task, not a wording
> one. Note the fork test was **not executed** — Go is absent from this machine —
> so the six rungs above are the fork's committed pins cross-checked against an
> independent derivation from the source formulas, which is two agreeing
> derivations and not a run.

> **What the `TO` line does NOT do.** It was `TO <truncated addr>` until
> 2026-08-23, showing **one** output and truncated — so a transaction with
> change named one destination, silently omitted the rest, and offered an
> address that could not be checked by eye. R0 round 1 (R-14) filed that as a
> Critical against §7's pinned-destination mitigation. The operator's ruling
> replaces it with a **wallet identity**, which names the counterparty instead
> of one of its scripts and does not degrade with output count.
>
> It is still not a full disclosure: it is one line, it is optional, and it says
> nothing when the destination is not a known wallet (§10.4). `mt` prints every
> output in full at encode time; the plate carries the summary.

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
hand: `POLICY_ID_STUB_BYTES = 4` at `crates/mk-codec/src/consts.rs:60`, the form-aware
rule documented at `crates/mk-codec/src/key_card.rs:25-33`, and the derivation
`derive_stub_from_md1_card` at `crates/mk-cli/src/cmd/mod.rs:126`. So one convention
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
Without them you cannot tell which wallet it spends. **`mt qr`'s** finalized
PSBT closes part of this by carrying each input's UTXO record — value and
scriptPubKey — so that payload does describe what it spends. **`mt string`'s
does not**: a raw transaction carries outpoints only, so a string plate is
silent about both the input amounts and the source scripts. It still does not name the *wallet*, hence the stub, and
hence the stub living in text, because it is the one constellation-specific fact
on the plate and F-234 forbids that inside the QR.

> **`mt`'s INPUT is always a finalized PSBT (§10.10), even for `mt string`,
> whose engraved payload is the extracted raw transaction.** Input format and
> payload format are separate decisions; requiring a PSBT is what keeps §8.2 and
> §8.2b runnable at all.
>
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
| **Bearer** — holder can broadcast (`mt qr`) | a timelock bounds it in *time*, not in space, and only when §8.4's `nSequence` condition holds; the `BEARER` line is the first line of a legend `mt` controls |
| **Bearer** — holder can broadcast (`mt string`) | **accepted risk, not mitigated on the plate.** `mt` emits a string, not an engraving, so it has no mechanism to put a warning on hand-cut steel (§3b). It warns once on `stderr` at encode time, to the person encoding — who is not the person holding the plate later. The timelock bound still applies |
| **Pinned destination** — a 2040 recoverer pays a 2026 address whose keys may be lost | **cannot be fixed; partly disclosed.** §5's `TO` line names the destination **wallet** (id or fingerprint), which does not degrade with output count as the old truncated-address form did — but it is **optional**, and says nothing when the destination is not a known wallet (§10.4). `mt` displays every output in full at encode time; the plate carries a summary |
| **Indistinguishable from a watch-only plate** — an `mt1` plate sits in the same drawer as `md1` and `mk1` plates, in the same script, differing in **one HRP character**, and is the only one of the three that is spendable by whoever picks it up | for `mt qr` the `BEARER` legend line carries the difference. For `mt string` there is **no mitigation** — see the bearer row above and §3b. R0 round 1 (R-13) |
| **Pinned fee** — a 2026 fee rate may be unbroadcastable in 2040 | **cannot be fixed by `mt`, and is NOT on the plate.** `mt` warns below 10 sat/vB (§8.2b) and names two things a future holder can try, guaranteeing neither: **CPFP** — spending one of this transaction's outputs with a high-fee child, which needs no key from the original signer, unlike **RBF**, which requires signing a replacement and is therefore useless to a plate holder — and **out-of-band submission** straight to a miner, which bypasses relay policy and is the escape hatch when a fee is too low for the parent to reach a mempool at all | Fee rate and date were cut from the legend (§5). `mt` displays both at encode time so the operator can judge staleness *before* engraving. A holder in 2040 recovers the fee by decoding **only for `mt qr`**, whose PSBT payload carries the input amounts; an `mt string` plate carries a raw transaction, from which the fee is **not** recoverable without the prevouts |
| **Silent invalidation** — one ordinary spend of any input voids the plate, and nothing on it says so | **not mitigated on the plate.** The input outpoints were cut from the legend (§5), so a holder cannot check unspentness from the plate alone — they must decode the QR first. `mt` checks it at encode time (§6a, §8.5); after that the hazard is open and undisclosed on steel |
| **Non-`ALL` sighash** — an input signed with `SIGHASH_NONE` or `SIGHASH_SINGLE` leaves outputs unbound, so a plate-holder can redirect the funds and the `TO` line becomes a lie | refused at encode time, §8.6 — **structurally**, since §8.2's removal left no script engine |
| **Wrong input value** — a legacy input whose claimed value is wrong yields a valid transaction, and **the fee absorbs the entire difference** | **not detectable by `mt`.** §8.2's removal means no signature is verified, and a legacy sighash never committed to the amount anyway. Mitigated only by §8.2c's warning — which states the arithmetic, `(real input value) − (output total)`, since the output total is the one term `mt` knows for certain — plus the engraved out-of-band reminder |
| **Well-formed but INVALID** — a transaction with a bad signature engraves cleanly and fails at broadcast, years later | **accepted, not mitigated.** Operator ruling 2026-08-23 removed script verification from v0.1 (§8.2). §8.1 sees a witness, §8.2b sees balanced values, §8.6 sees correct sighash flags — none of them verifies the signature. `mt` may add this someday |

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

1. **Not fully finalized** → refuse, **on both payloads, by their own
   vocabulary.** For a PSBT: every input carries a populated
   `PSBT_IN_FINAL_SCRIPTSIG` or `PSBT_IN_FINAL_SCRIPTWITNESS`. For a raw
   transaction: every input carries a non-empty `scriptSig` **or** a non-empty
   witness. Neither format makes an unfinalized transaction unrepresentable —
   §3's retraction — so this check is mandatory on both verbs and may not be
   skipped or overridden.
2. **Script validity is NOT checked in v0.1.** **Operator ruling 2026-08-23:
   *"We don't care if transaction is valid for initial version. We might never
   care but we might add it someday."*** The previous draft ran real
   libbitcoinconsensus verification here; that is removed, and with it `mt`'s
   only dependency on a consensus engine.

   > **What this costs, stated plainly because nothing else in §8 covers it.**
   > `mt` no longer detects a transaction that is **well-formed but invalid** —
   > most importantly one carrying a **bad signature**. Such a transaction has a
   > witness present (passes §8.1), balances (passes §8.2b), and carries correct
   > sighash flags (passes §8.6). It engraves cleanly and **fails at broadcast**,
   > which for this artifact means years later, in exactly the situation it was
   > cut for. §7 records this as an accepted hazard.
   >
   > **It also weakens §8.6.** That refusal reasons about whether an input's
   > *satisfaction binds the outputs*, and without a script engine `mt` can only
   > inspect the witness **structurally** — it can see that a stack element is
   > shaped like a signature, not that the script requires one. See §8.6.
   >
   > The upside is real and is why the ruling is defensible: `mt` becomes a tool
   > that parses a PSBT, checks structure and arithmetic, reads two locktime
   > fields and asks a node two questions. That is a far smaller thing to get
   > right than one embedding a consensus engine, and this artifact's other
   > failure modes — the plate being bearer, the destination being stale, the
   > inputs being spent — are ones validity checking never addressed anyway.

2b. **Value-blind acceptance** → refuse. **Now one of the few checks `mt` runs,
   since §8.2 is gone.** `verify_transaction` is a per-input
   *script* loop — read from `consensus/validation.rs` in the `bitcoin` 0.32.101 crate (lines 82-107 of the registry source),
   it iterates `tx.input` calling `verify_script_with_flags` and returns — so it
   never compares input value against output value. Outputs exceeding inputs,
   duplicate inputs and an empty `vin` all pass every other refusal here.
   `mt` must therefore check, at minimum:

   - **inputs ≥ outputs** (`SendingTooMuch`);
   - **an absurdly HIGH fee** — `rust-bitcoin`'s own ceiling is
     `DEFAULT_MAX_FEE_RATE = 25,000 sat/vB`, raised as `AbsurdFeeRate`. This is
     the direction that loses money, and it is what a wrong input value produces
     (§8.2c);
   - **NO minimum fee — but a WARNING below 10 sat/vB.** Operator rulings
     2026-08-23. A refusal floor would hardcode today's relay policy into an
     artifact meant to be broadcast in 2040, the same mistake as engraving a
     dollar figure (§9). `mt` reports the rate and warns:

           WARNING: fee rate is 3.2 sat/vB.

           This transaction may be engraved and then sit for years. A fee has
           to be high enough to motivate a miner AT THE TIME IT IS BROADCAST,
           and nobody knows what that will be. If it turns out too low, the
           holder may need CPFP -- spending one of this transaction's outputs
           with a high-fee child, which needs no key from the signer -- or
           out-of-band submission directly to a miner, which bypasses relay
           policy entirely.

     **The 10 sat/vB threshold is a heuristic and will age**, which is fine here
     for a reason worth stating: it is consumed **at encode time, by a human who
     is present**, and is never engraved. A number that ages is only dangerous
     on steel;
   - **no duplicate outpoints**, and **`vin` non-empty**.

   > **The spec convicts itself here.** §3 rejected the `lean` PSBT form on the
   > grounds that *"the safe API a recoverer reaches for refuses it"*. That API
   > is `extract_tx()`, and it refuses on **three** counts — `MissingInputValue`,
   > `SendingTooMuch` and `AbsurdFeeRate`. §8 adopted the first and ignored the
   > other two while citing the same API as its standard of care.

2c. **Input values: require them when the PSBT lacks them, and WARN whenever a
   legacy input is present.** Operator rulings 2026-08-23: *"Only require user to
   supply utxo values if not part of the psbt"*, and *"Just warn users legacy
   input exists and they will pay a fee equivalent to what is really present at
   the input minus sum of outputs. Explain that this could be very large if they
   are wrong about what the value of the input is."*

   A finalized PSBT in the MIN form normally carries every input's UTXO record
   (§3), so `mt` computes the fee itself and asks for nothing. Where a record is
   absent, `mt` requires the operator to supply that input's value — or the total
   across all inputs — since §8.2b cannot check the value balance without it.

   **The legacy warning fires whenever any input is legacy**, whether the value
   came from the PSBT or from the operator, because `mt` verifies neither. It
   states the mechanism rather than a caution:

       WARNING: input 0 is a legacy (pre-SegWit) input.

       The fee you will pay is:   (what is REALLY at that input) - 0.99000000 BTC
       You have told mt it holds:  1.00000000 BTC
       So mt shows a fee of:       0.01000000 BTC

       mt CANNOT VERIFY THAT VALUE. A legacy signature does not commit to the
       input's amount, so a wrong value still produces a perfectly valid
       transaction -- and the fee absorbs the entire difference. If that input
       actually holds 10 BTC, this transaction pays 9.01 BTC in fees and a
       miner will simply take it.

       Verify the input value out of band, and engrave a reminder to re-check
       it before broadcasting.

   **The output total is the anchor and `mt` knows it with certainty** — it is in
   the transaction. Everything uncertain sits on the other side of the
   subtraction, which is what makes the warning stateable as arithmetic rather
   than as advice.

   > **Why this is the residual hazard §8 cannot close.** The value is not in the
   > transaction; it lives in the already-confirmed previous output. **No miner
   > can alter it and no attacker can inflate the fee that way** — a miner would
   > have to rewrite a block. The entire risk is that the claimed value is wrong,
   > and whether anything catches that depends on the input type:
   >
   > | input | sighash commits to the amount? | a wrong value produces |
   > | --- | --- | --- |
   > | SegWit v0 (BIP-143) | **yes** | an invalid signature — caught by anyone who verifies |
   > | Taproot (BIP-341) | **yes** | an invalid signature — caught |
   > | **legacy** | **no** | **a valid signature and a catastrophic fee** |
   >
   > This is exactly what BIP-143 was written for: *"eliminates the possibility
   > to lie to offline signing devices about the fee of a transaction."* And
   > §8.2's removal widened it — `mt` verifies no signatures at all now, so for a
   > legacy input the claimed value is checked against **nothing**. The warning
   > and the engraved reminder are the whole mitigation.

3. **An unsigned or unfinalized transaction offered for engraving** → refuse. It
   cannot be broadcast, so it is not a backup.
4. **Read the locktime FIELDS, compare against the chain if a node is there,
   and warn. Never refuse, never on a flag, and never by reading scripts.**
   Operator rulings 2026-08-23: *"Timelocking happens by user at their wallet
   software… We merely read transaction and warn if immediate"*, and — the scope
   line that decides how this is implemented — *"we can know with certainty if a
   transaction is locked to a specific block. And we can ask `bitcoind`, if
   available, what the current block is. But we are not in the business of
   handing the transaction to `bitcoind` to check validity or reading scripts to
   evaluate for timelocks in the sending wallet's descriptor."*

   **So `mt` reads two FIELDS and asks one question.** Fields are certain;
   scripts are somebody else's job.

   | input | source | certain? |
   | --- | --- | --- |
   | `nLockTime` | transaction field | **yes** |
   | `nSequence`, per input | transaction field | **yes** |
   | current block height | `bitcoind` if reachable, else absent | yes when present |

   **The rule:**

**`mt` states the two facts and stops.** Operator ruling 2026-08-23:
   *"'may be immediately spendable' is accurate but incomplete. Just say whether
   the transaction is locked to block x and current height is y."*

   So the `stderr` report is a statement of what was read, not a verdict:

       LOCKED TO BLOCK 1383520   current height 963663
       NO BLOCK TIMELOCK         current height 963663
       nLockTime 900000 present but NOT ENFORCED (all inputs final)
       LOCKED TO BLOCK 900000    current height unknown (no node)

   **Why facts beat a verdict here.** *"May be immediately spendable"* is true of
   almost any transaction and tells the operator nothing they can act on — it
   cannot distinguish a lock that has already passed from one that was never
   enforced from one still years away, and all three want different responses.
   Two numbers side by side let the operator see which case they are in. It also
   keeps `mt` inside its own scope: a height comparison is arithmetic on fields,
   whereas *"spendable"* is a claim about a transaction's fate that depends on
   scripts, fees and unspent inputs — none of which `mt` evaluates.

   - **Legend:** `LOCKED TO BLOCK <n>`, or `NO BLOCK TIMELOCK`.
   - **Height comes from `bitcoind` when reachable**, and is reported as unknown
     otherwise. This is the whole of `mt`'s use of the chain here — it never
     hands the transaction to the node for validation.
   - **A lock that has already passed is reported the same way**, because the
     two numbers say so: `LOCKED TO BLOCK 900000, current height 963663` is a
     plate that is live now, and the operator can read that without `mt`
     concluding it for them.

   **`nSequence` is not optional, and omitting it causes the dangerous error.**
   `nLockTime` is enforced only when at least one input has
   `nSequence != 0xFFFFFFFF`. A transaction with every input final ignores its
   locktime — so reading `nLockTime` alone would engrave `SPENDABLE AFTER BLOCK
   900000` on a plate anyone can broadcast today. That is **false reassurance on
   steel**, the worst failure available here, and it is a field read rather than
   a script read, so it stays in scope. `nSequence` appeared nowhere in the
   534-line draft that first specified this rule.

   > **What `mt` therefore CANNOT see, disclosed rather than glossed.** A BIP-68
   > **relative** timelock lives in the witness script as `OP_CSV`, and a
   > relative-locked spend has **`nLockTime = 0`**. Reading it means evaluating
   > the sending wallet's script, which is out of scope by ruling. One of the
   > RCW's own taproot leaves is exactly this — `OP_CSV` with `008000` = 32,768
   > blocks, roughly seven months (`RESULTS_rcw_2026-08-22.txt`).
   >
   > **`mt` will therefore OVER-WARN on such transactions**, which is the safe
   > direction: it says a plate might be spendable when it is not, and the
   > operator — who chose the wallet — can disregard it. The unsafe direction,
   > false reassurance, is closed by the `nSequence` rule above.
   >
   > **This is why the legend states an OBSERVATION, not a conclusion.** An
   > earlier draft engraved `IMMEDIATELY SPENDABLE`, which is a positive claim
   > about spendability that `mt` can no longer substantiate — engrave it on a
   > `OP_CSV`-locked transaction and the steel permanently asserts something
   > false. A `stderr` warning is disposable; a legend line is forever. The
   > legend now reads **`NO BLOCK TIMELOCK`**: precisely true about the fields
   > `mt` read, and silent about scripts it did not.
5. **`gettxout` returns `null` for any input** → refuse, when a node is
   reachable. The output is spent or never existed. See §6a's limitation and
   §10.5 for the IBD case.
6. **Any input whose satisfaction does not bind the outputs** → refuse. Two
   cases, and the previous draft caught only the first:

   a. **A signature with a non-`ALL` sighash.** A `SIGHASH_NONE` input leaves
      the outputs unbound, so a holder — or anyone who photographs the plate —
      can redirect the funds while the signature stays valid, and the legend's
      `TO` line becomes false. `SIGHASH_SINGLE` and `SIGHASH_ANYONECANPAY` are
      refused on the same grounds. Accepted: `SIGHASH_ALL`, and taproot's
      `SIGHASH_DEFAULT`.

   b. **NO signature at all.** R0 round 1 (R-4). The previous rule was written
      over *signatures* and silently assumed every input has one. **A miniscript
      satisfaction need not.** This project's own RCW fixture is the proof: its
      tier 4 was `after(N) AND sha256(H)` — a timelock and a hash preimage, no
      key — until commit `d1889e4` added one, and stock rust-miniscript accepted
      the `wsh` form throughout. An input satisfied by preimage alone commits to
      **nothing**: any holder can rewrite every output and re-satisfy it. That
      is strictly worse than the `SIGHASH_NONE` case (a), which at least binds
      the inputs.

      So the rule is over the **satisfaction**, not the signature: every input
      must carry at least one signature, and every signature must be (a)-clean.

      > **Limited by §8.2's removal.** Without a script engine `mt` inspects the
      > witness **structurally** — it can tell that a stack element is
      > *shaped* like a signature (a 64-byte Schnorr element, or a DER-encoded
      > ECDSA one with a trailing sighash byte), but not that the script it
      > satisfies actually **requires** one. A crafted witness carrying a
      > signature-shaped element that the script never checks would pass. This
      > is a structural heuristic, not a proof, and the spec should not claim
      > more than that.

   > **Legacy inputs are ACCEPTED. Operator ruling 2026-08-23:** *"Do not
   > exclude legacy inputs. It is user responsibility to know their inputs for
   > such edge cases."* The previous draft refused them, and its stated reason
   > was false: it claimed a legacy amount is unverifiable because the sighash
   > does not commit to it. The first clause is true; the conclusion does not
   > follow, since BIP-174 requires `non_witness_utxo` for a legacy input —
   > the **whole previous transaction** — so hashing it and matching the txid
   > binds the amount without any help from the sighash.
   >
   > `sh(wsh(…))` is therefore no longer an unclassified case: wrapped-segwit
   > inputs are segwit inputs, and every input type is accepted.

7. **Over the plate budget (`mt qr`)** → refuse, naming the exact plate count
   and what would fit.
7b. **Over the 64-chunk container (`mt string`)** → refuse, naming the chunk
   count and the ceiling, and pointing at `mt qr`, which has no such limit. Real
   wallets hit this: RCW `wsh` tier 1 at 5 inputs needs 78 chunks (§3b).
8. **Module size is the operator's choice, defaulting to 0.60 mm** — not a
   refusal. Ruling 2026-08-23 (§10.1): `mt` offers every size it can engrave and
   suggests 0.60 mm (two engraved strokes). Sizes below that are **optically
   unvalidated**, and `mt` says so at the point of choice rather than refusing.
   A scan that succeeds today is evidence about one plate on one machine on one
   day, not a property of the size (§10.1).
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

1. **The F-234 optical test plate has not been cut — and the module size is now
   the USER's choice.** **Operator ruling 2026-08-23: "User picks from all
   available options, suggesting 0.6."**

   So §8.8's hard refusal below 0.60 mm becomes a **default and a
   recommendation**, not a floor: `mt` offers every module size it can engrave,
   suggests 0.60 mm, and the operator decides. The test plate still wants
   cutting — it is how anyone learns what 0.30 mm actually does on steel — but it
   no longer gates the tool.

   > **The operator's second point, which generalises past this question:**
   > *"just because one size engraves and scans today doesn't guarantee in the
   > future the engraving will scan due to maintenance issues."*
   >
   > A successful scan is evidence about **one plate, one machine, one day** —
   > not a property of the module size. Machine wear, stylus condition, plate
   > stock and lighting all drift, and the artifact must survive decades of that
   > drift. This is why a test plate can **license** a size and can never
   > **certify** it, and it is an independent argument for spending slack on ECC
   > rather than on smaller modules: error correction is the only margin that
   > keeps paying after the machine has changed.

2. ~~Will a wallet reassemble multi-part UR from STATIC symbols?~~ **OUT OF
   SCOPE, operator ruling 2026-08-23: "We will add another verb in the next
   subversion to accept static scan data."**

   `mt` will ship its own reader rather than depend on third-party wallets
   stitching engraved symbols together. That retires the spec's most
   load-bearing unverified assumption by removing the dependency instead of
   testing it.

   **It also removes the main argument for UR**, and §10.3 turns on this. UR was
   defended as the only fragmentation the Bitcoin ecosystem implements; if `mt`
   supplies the reader, the ecosystem is not reassembling anything and that
   defence is void for every multi-symbol artifact.

   **What it costs, stated plainly:** F-234's promise — that a recoverer with
   none of our tools can still read the plate — now holds only for artifacts
   that fit **one** symbol. Multi-symbol recovery requires `mt`'s reader. The
   next subversion's verb is therefore not a convenience; it is what keeps
   multi-plate transactions recoverable at all, and it should be specified
   before anyone engraves a multi-symbol artifact.

3. ~~Is UR worth its expansion? What goes in the QR?~~ **CLOSED.** UR is
   dropped (§3), and the QR payload is **`mt1` chunks, base45-encoded** —
   operator ruling 2026-08-23. Codex32-in-QR was measured and rejected at 63–65%
   efficiency, worse than the UR it would replace and up to two extra plates
   (§3). base45 was chosen over 3%-denser raw binary for scanner compatibility
   and its ~28% intrinsic detection of corrupted triples. **§10.1's test plate
   should still confirm scanners read base45 off engraved steel** — the choice is
   made, the optical validation is not.

4. ~~The legend's FROM and TO fields.~~ **CLOSED**, operator rulings
   2026-08-23: *"we use walletid or seed fp for the from: field and to: field.
   Optional but loudly warn if either not supplied"*, and — for the third-party
   case — *"warn if blank but allow, allow arbitrary text if user passes a
   flag."*

   Both fields are **wallet identities**, not addresses: a wallet id or a seed
   fingerprint. `FROM` is what §6 says a transaction cannot tell you on its own.
   `TO` names the counterparty rather than one of its scripts, which is why it
   replaced the truncated address R0 round 1 filed as a Critical (R-14) — a
   truncated address showed one output of several and could not be checked by
   eye.

   **Three states for `TO`, and paying a third party is the reason for the
   third:**

   | state | behaviour |
   | --- | --- |
   | wallet id or fingerprint | engraved as given |
   | **blank** | **allowed, loudly warned** on `stderr` — a plate with no destination named is legal and worse |
   | **arbitrary text, behind a flag** | engraved as given, e.g. `TO ALICE` |

   **The flag is the point, not a convenience.** A free-text label cannot be
   derived from or checked against the transaction, so requiring an explicit flag
   makes it an **act of assertion by the operator** rather than something that
   quietly appears. It is the same posture as the stub: a human-orientation aid,
   never an authority, and §5 already forbids branching on any of it. If the
   label disagrees with the transaction, the transaction wins.

   **Still to specify (§10.10's CLI work, not a design question):** the flag's
   name, and what `mt` does with a label too long for the field — §5's budget
   gives `TO` 34 characters including the amount, so a label has roughly 16.
   Refusing with the limit named fits §8's rule that every refusal names its
   number; silent truncation does not.

5. ~~Should `mt` require the node to be out of IBD before trusting
   `gettxout`?~~ **CLOSED — OUT OF SCOPE**, operator ruling 2026-08-23. `mt`
   asks the node it is given and reports what it is told; vouching for the
   node's sync state is not `mt`'s job. §8.5's refusal stands as written, and
   §6a already records that a `null` cannot distinguish "spent" from "this node
   does not know yet".

6. ~~How much fountain redundancy?~~ **CLOSED**, operator ruling 2026-08-23:
   zero. `mt` protects against plate damage (ECC), not plate loss (duplicate
   plates, the operator's choice). See §3.
7. **Back-side engraving — CLOSED for v0.1**, operator ruling 2026-08-23:
   *"yes, but probably better left to user to manage physically."* It would
   recover the 25.5 mm the legend costs and reduce plate counts, but there is no
   back-side path in the fork (`backup/backup.go:247` defines `frontSideSeed`,
   called once at `:134`, with a single `Engraving` per plate), so it is
   firmware work. An operator who wants both sides used can flip the plate and
   run a second job — a physical workflow rather than a `mt` feature. §4's plate
   counts therefore stand as one-sided.

8. ~~How does a recoverer learn the fountain parameters?~~ **ANSWERED, and the
   operator has ruled on what follows from it.**

   > **Ruling, operator, 2026-08-23: "each piece should say something like
   > n of m."**

   Machine-readably this already holds at both layers, verified from source:

   - **`mt string`** — `ChunkHeader` carries `count` and `index`
     (`crates/md-codec/src/chunk.rs`), inside the BCH-protected header, plus a 20-bit
     `chunk_set_id` shared across a set so pieces of different transactions
     cannot be combined. This is the model; nothing to add.
   - **`mt qr`** — every multi-part UR carries `SeqLen`, `MessageLen` and
     `Checksum` in its CBOR (`bc/fountain/fountain.go:74-87`), and the string
     itself reads `ur:psbt/<n>-<m>/…` (`bc/ur/ur.go:122`).

   **Three traps, all load-bearing for the ruling:**

   1. **The visible `<n>-<m>` prefix is NOT authoritative.** `bc/ur/ur.go:179`
      parses it with `Sscanf` into locals `seq` and `n` — and then never uses
      them, calling `d.fountain.Add(enc)` and letting the CBOR decide. The
      prefix a human reads and the field a decoder obeys are different data. They
      agree when `mt` writes both, but nothing enforces that.
   2. **A single-part UR says nothing at all.** `bc/ur/ur.go:118`: at
      `seqLen == 1` the encoder emits `ur:psbt/<data>` with **no `n-m` prefix**
      and skips the fountain wrapper entirely, so there is no SeqLen, no
      MessageLen and no Checksum. A lone symbol cannot state that it is
      complete.
   3. **`PLATE n OF m` is not `part n of m`, and §5 offers only the former.**
      Under a multi-symbol tiling, plate 2 of 3 may carry parts 5–8 of 11.
      Nothing in the spec maps symbols to plates or fixes their order, so a
      recoverer who scans out of sequence, or misses one symbol *on* a plate,
      cannot tell which part is absent. This is the gap the ruling closes.

   **Normative, from the ruling:** every engraved symbol carries its own
   human-readable `n/m` beside it, in engraved text, for the UR part it holds —
   independent of, and in addition to, the plate's `PLATE n OF m`. A recoverer
   must be able to inventory what they hold and name what is missing **without
   decoding anything**. For the single-part case that label reads `1/1`, which is
   the only way a lone symbol can say it is whole.

   **Unpriced.** These labels consume plate area that §4's table does not
   reserve, exactly as the legend did before it was measured — see §10.14, which
   already requires that table's regeneration. Cost per label is small (3–5
   characters) but it is per *symbol*, not per plate, and the worst artifact here
   carries 5 symbols. **Measure before §4's numbers are treated as final.**

   Also unchanged: `Progress()` is a `x1.75` UI heuristic that can reach 1.0
   while `Result()` is still nil, so **nothing may gate on `Progress()`**.
9. ~~How does the engraving reach the machine?~~ **ANSWERED, operator ruling
   2026-08-23: "send via payload unencrypted. We have a format for transferring
   data to SH2 via USB."**

   That format is **`sysw`**, the system-wide payload already used for every
   other constellation artifact. It is Rust-primary in this repo
   (`crates/me-cli/src/sysw/`) and ported to the fork (`sysw/record.go`). A
   payload carries a **`Class`**, and the existing set is `Mnemonic`,
   `Codex32Secret`, `Passphrase`, `FreeText`, `Descriptor`, `MdMk`, `Address`,
   `Unknown` (`crates/me-cli/src/sysw/record.rs:31-40`).

   **There is no transaction class**, which is what R0 lens 4 found. Adding one
   is the work, and **the Rust-primary rule binds**: the new class lands in
   `me-cli`'s Rust `sysw` first, with test vectors, and only then ports to the
   fork's Go.

   **Unencrypted, by ruling.** Note `me` has an encrypted-payload path and this
   deliberately does not use it. The reasoning is consistent with §7: the plate
   the payload produces is **bearer** and sits in a drawer, so the wire is not
   where this artifact's secrecy lives. What the ruling does accept is that
   anyone with access to the USB link sees the transaction before it is cut.

   **Still open underneath this ruling, and it still blocks:** §4 selects an ECC
   level, a module size and a multi-symbol tiling, and the fork's only
   arbitrary-payload QR path is fixed at `freeTextQRScale = 2`
   (`backup/fit.go:19`) with a compile-time ECC level and one code per plate.
   A `sysw` class says how the bytes *arrive*; it does not make the firmware able
   to engrave what §4 chose. **That gap is now §10.17.**

10. **The CLI surface — RULED.** Operator rulings 2026-08-23.

    | | |
    | --- | --- |
    | verbs | `mt qr`, `mt string` |
    | **input** | **a finalized PSBT, and nothing else** — from a file or stdin, equivalently |
    | `mt qr` output | a **SH2 payload** (`sysw`) carrying the QR — machine engraving |
    | `mt string` output | the **codex32 string on stdout** — hand engraving |
    | stderr | every warning and refusal a human must see (§3b) |
    | flags | **none for locktime** (§8.4) |

    **Why PSBT-only, when `mt string`'s PAYLOAD is a raw transaction.** Input
    format and payload format are independent, and conflating them would have
    cost two refusals. §8 is written in PSBT vocabulary and degrades unevenly
    without one:

    | refusal | finalized PSBT | raw signed transaction |
    | --- | --- | --- |
    | §8.1 finalized? | reads `PSBT_IN_FINAL_*` | reads scriptSig/witness — **works** |
    | ~~§8.2 script-valid?~~ | *removed from v0.1* | *removed from v0.1* |
    | §8.2b value balance? | UTXO records give input values | **cannot run** — no input amounts |
    | §8.6 satisfaction binds outputs? | parses the witness | parses the witness — **works** |

    So accepting raw hex would **silently disable two refusals**, including the
    only check that inputs ≥ outputs, while the artifact looked identical. `mt`
    therefore requires a PSBT, runs the full refusal set against it, and then —
    for `mt string` — extracts the raw transaction as the payload. Nothing is
    lost: a PSBT is what wallet software emits at the point this workflow
    starts, which is exactly the *"test it in your wallet first"* flow §0 is
    built around.

    **Still unspecified:** exit codes, and the format of the refusal messages
    §8 promises will *"name the number that caused it"*.

11. ~~How many codex32 characters fit a hand-engraved plate?~~ **CLOSED — OUT
    OF SCOPE**, operator ruling 2026-08-23: *"As many as a user wants. It is not
    our concern."* `mt string` emits a string; what a user does with steel is
    theirs. See §3b. The 64-chunk ceiling is unaffected — that is a property of
    the codec, not of anyone's plate.

12. ~~Should `mt1` FILL its chunks rather than balance them?~~ **CLOSED — NO.
    Filling would reduce error recoverability, which is the one thing this
    format exists for.** Operator question 2026-08-23: *"does increased packing
    reduce error recoverability?"* Answered from source, and the answer is yes,
    by two independent mechanisms:

    **BCH correction is PER CHUNK, and it is `t = 4`.** `decode_regular_errors`
    returns `None` for any pattern above *"t = 4 errors"*, against a 13-symbol
    checksum (`REGULAR_CHECKSUM_SYMBOLS`) over a codeword of at most 93 symbols
    (`crates/md-codec/src/bch_decode.rs`). Each chunk therefore carries its **own
    independent 4-error budget**.

    1. **Fewer chunks means less total correction.** For a fixed payload,
       filling packs the same bytes into ~12% fewer chunks — and the budget
       scales with chunk *count*. A 535 B transaction balanced at 40 B/chunk is
       14 chunks = **56 correctable symbol errors**; filled at ~45 B/chunk it is
       12 chunks = **48**. Same data, 8 fewer errors survivable.
    2. **Each chunk is longer under the same `t`.** Filling raises the symbols
       at risk per chunk while the per-chunk budget stays at 4, so the
       probability that any single chunk exceeds its budget rises.

    Both effects push the same way. **Balancing is not a limitation of `md`'s
    chunker — it is error-correction budget bought with plate area**, and for a
    hand-engraved artifact whose entire purpose is surviving a miscut character,
    trading it for ~340 bytes of ceiling is the wrong trade. The 2,560 B ceiling
    stands, and §8.7b refuses past it.

13. **`mt1`'s own encoding, NUMS constant and content id — RULED, ready to
    build.** Operator rulings 2026-08-23.

    R0 round 1 (S-2) read `md-codec` directly: the header *layout* (37 bits),
    chunk ordering, gap detection and missing-chunk checks are payload-agnostic
    and take a transaction cleanly. Three things do not transfer, and all three
    are now decided:

    **(a) Its own NUMS constant.** `MD_REGULAR_CONST` is hardcoded into checksum
    create and verify (`crates/md-codec/src/bch.rs`). Every constellation format
    gets its own; without a distinct one **an `mt1` chunk would verify as a valid
    `md1` chunk**, which for a bearer plate sitting in a drawer beside `md1`
    plates is a real hazard, not a theoretical one.

    **(b) Its own HRP**, `mt1`, currently hardcoded at four sites in `md-codec`.

    **(c) A content id — RULED: the transaction id.** `derive_chunk_set_id`
    hashes a *descriptor*, and reassembly re-derives it from the decoded object
    as what the source calls *"the content-id oracle; funds-load-bearing
    invariant."* `mt1`'s analogue is the **txid**: already a canonical hash of
    exactly this content, already present, already what a recoverer would use to
    name the transaction. **Reassembly re-derives it from the decoded
    transaction and compares**, giving `mt1` the same invariant `md1` has.

    **Width stays at 20 bits.** Operator: *"1 in a million is more than unique
    enough. User only needs to distinguish between at most a few dozen engraved
    transactions… 1 in 1000 only saves 2 characters from 1 in 1000000, so 20
    bits is probably not too burdensome."* The arithmetic holds — 20 bits is 4
    codex32 symbols against 10 bits' 2, so narrowing saves 2 characters **per
    chunk** (~24 on a 12-chunk transaction). Worth adding: **the re-derivation in
    (c) is what makes the width non-critical.** A collision cannot yield a wrong
    transaction, because reassembly re-derives the id from what it decoded and a
    mismatch is caught. The 20 bits buy human discrimination and early detection,
    not integrity.

    > **WHERE THIS LANDS — and an earlier statement of mine was wrong.** I said
    > this "lands in `descriptor-mnemonic`". It does not. The constellation's
    > precedent is **forking, not sharing**: `md-codec`'s own BCH decoder says
    > *"Forked from `mk-codec` v0.3.1… The algorithm is constant-agnostic — the
    > caller XORs the polymod residue against the per-HRP target constant"*, and
    > `md-codec` has **no dependency on `mk-codec`**. So `mt1` forks the same
    > machinery into **`mt-codec`, in the new `mnemonic-transaction` repo**, with
    > its own constants. **`descriptor-mnemonic` is untouched.**
    >
    > A future `mc-codex32` shared crate is planned to retire these forks; its
    > stated trigger is *"both formats v1.0 with cross-validated conformance
    > vectors"*, so `mt1` should be built to be absorbed by it later, not to
    > block on it now.

    **What Rust-primary means for this format**, since it binds later rather
    than now: `mt-codec` in Rust is the primary and only implementation today.
    When SH2 learns to read `mt1` — §10.2's static-scan reader and §10.17's
    firmware work — the **Go decoder is written as a PORT**, bound to the Rust
    conformance vectors, and may never lead. If the two ever disagree, Rust is
    right by definition and Go is the bug. That is not theoretical in this
    constellation: Go and Rust once computed **different `WalletPolicyId`s**
    while 887 fork tests passed either way, and only cross-language vectors
    caught it.

    **No longer blocking as a design question** — it is now scoped
    implementation work with every decision made. It still blocks *code* for
    both verbs, since both fragment with this header.

14. **§5's legend budget rests on a doc comment, not on the fork's font
    metrics. DEFERRED** by operator ruling 2026-08-23. `legend.rs` hardcodes
    `CHARS_PER_LINE = 35.0` / `LINES_FULL_PLATE = 20.0` per a doc comment at
    `crates/me-cli/src/lib.rs:46`; the fork's real ladder has six rungs and
    those are the 3.8 mm one. §4's 4.25 mm pitch is `85/20` — full plate height,
    where §4 uses 79 mm elsewhere — and is not a rung of `FontSizes`. Magnitude
    is under a millimetre. **Deferred, not closed:** §4's table must be
    regenerated before implementation anyway, for the three unmodelled inputs
    named there, and this correction rides along with that regeneration.

15. ~~§8.4 sets no minimum timelock horizon, and cannot tell a timelock from
    RBF signalling.~~ **CLOSED — OUT OF SCOPE**, operator ruling 2026-08-23:
    *"not our concern. User handles this by their own wallet, or we later create
    our own wallet utilities."* Consistent with §0: `mt` does not build
    transactions, so how long a timelock ought to be is a wallet decision. `mt`
    still verifies that the timelock it was handed is **enforced** (§8.4) — it
    simply does not judge whether the horizon is wise.

16. ~~Should `mt` refuse legacy (non-segwit) inputs at all?~~ **CLOSED — NO**,
    operator ruling 2026-08-23: *"Do not exclude legacy inputs. It is user
    responsibility to know their inputs for such edge cases."* See §8.6. The
    original refusal's premise was false (`non_witness_utxo` binds a legacy
    amount by txid), and `sh(wsh(…))` is no longer unclassified since every
    input type is accepted. The residual risk is handled by §8.2c's engraved
    out-of-band reminder and recorded in §7.

17. **The firmware cannot yet engrave what §4 selects — and will be taught.**
    Operator ruling 2026-08-23: *"we will later teach SH2 how to handle
    transactions."* So this is scheduled firmware work rather than an unresolved
    design question, and §4 keeps its full search space.

    What stands today: the fork's only arbitrary-payload QR path is
    `freeTextQRScale = 2` (`backup/fit.go:19`) with a compile-time ECC level and
    one code per plate, and `sysw`'s `Class` enum has no transaction member
    (`crates/me-cli/src/sysw/record.rs:31-40`). **Until that work lands, `mt qr`
    can produce a payload that no shipped firmware will engrave.** That is a
    real limitation on the verb, not on the spec, and it should be stated
    wherever `mt qr` is documented as usable. **The Rust-primary rule binds the
    new `Class`:** it lands in `me-cli`'s Rust `sysw` with test vectors first,
    then ports to the fork's Go.


18. ~~Does §8.2's consensus-engine check survive the scope line?~~ **CLOSED —
    NO. Script validity is out of v0.1**, operator ruling 2026-08-23: *"We don't
    care if transaction is valid for initial version. We might never care but we
    might add it someday."* §8.2 is removed, `mt` drops its consensus-engine
    dependency, and §7 carries the accepted hazard: a transaction with a bad
    signature engraves cleanly and fails at broadcast. Reopen if it is ever
    added.


19. ~~Does CPFP still require the parent to reach the mempool?~~ **CLOSED — the
    spec no longer needs the answer.** Operator ruling 2026-08-23: *"We don't
    care about rbf or cpfp… we can't control the future but cpfp is a well known
    standard that will help user in future if they picked a bad fee."*

    `mt` neither implements nor checks either mechanism. §8.2b's low-fee warning
    **names** CPFP and out-of-band miner submission as things a future holder can
    try, and guarantees neither — so the mempool question stops being
    load-bearing. Out-of-band submission is itself the answer to the case that
    prompted this: a fee too low for the parent to reach a mempool at all
    bypasses relay policy by going straight to a miner.

20. **Legacy inputs are txid-malleable, and the content id is the txid.** A
    legacy `scriptSig` can be re-encoded by a third party in relay without
    invalidating the signature, changing the txid — what SegWit fixed. The
    engraved bytes still have exactly one txid and a recoverer re-derives it
    deterministically, so §10.13's content id is sound. But **if a malleated
    version confirms first, the confirmed txid will not match the plate's** —
    the plate is not wrong, it is superseded, and the original can no longer
    confirm. Worth a sentence somewhere a recoverer will read.

## 11. Provenance of the numbers

Everything measured is in `design/measurements/`, with the probe sources and a
reproduce path that is a command rather than a memory. Transaction sizes come
from real transactions — built, signed, finalized, extracted, serialised, never
estimated. QR capacities are gated against the published v40 limits at every
mode and ECC level, a gate that caught three wrong payload constructions before
these numbers were trusted. Plate and module constants are read from the fork
(`backup/backup.go:45,99-102`, `cmd/controller/platform_sh2.go:188`).

The probe crate has been re-run twice, and the counts differ because the crate
grew between them:

- **2026-08-22, before R0** — all **12** binaries then in the crate rebuilt and
  re-run; **9 of 11** results files reproduced **byte-identically**, the two
  exceptions differing only by capture artifacts documented in
  `design/measurements/README.md`.
- **2026-08-23, for the chunk-size correction** — `psbtfinal.rs` had since been
  added, so all **13** binaries were rebuilt and re-run and all **12** results
  files regenerated. This is the current state of every number in this spec.

§3b's chunk counts come from `RESULTS_envelope_2026-08-22.txt` and
`RESULTS_rcw_2026-08-22.txt`, which measure the **raw signed transaction**
against the 64-chunk container.

> **They remain a LOWER BOUND, but not for the reason an earlier draft gave.**
> That draft called them "a floor, for the balancing reason stated in §3b" —
> i.e. because `md`'s chunker balances rather than fills. **That reason is now
> void**: §3b's correction established that chunk sizing is a flat 40 payload
> bytes (`crates/md-codec/src/chunk.rs:224,253-254`), so the count is *exact* for a
> given payload size.
>
> They are a lower bound because of what is fed in. `md-codec` chunks the output
> of `encode_payload`, which is a **framed** payload — canonicalization plus TLV
> sections — not raw bytes. The probe feeds the **raw transaction length**
> straight in, modelling **zero framing overhead** for `mt1`. Whatever header
> `mt1` ends up carrying adds to the payload and can therefore add chunks. That
> is precisely open question §10.13, and it must close before these counts are
> treated as final.

The BCH corrector's existence was read from `crates/md-codec/src/bch_decode.rs` and
`crates/md-codec/src/lib.rs:48` in the `descriptor-mnemonic` repo — a sibling, so
`plan-cite-check.sh` has no root for it and those two were checked by hand.

> The previous draft's §11 claimed *"everything measured is in
> `design/measurements/`"* while the §6c/§6d block figures had no results file
> behind them. Those sections are now out of scope (§9), so the claim is true
> again by subtraction rather than by generating the missing evidence.
