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
| **`mt encode`** | `mt1` chunked codex32, **on stdout** | **by hand**, or by any tool the operator chooses | raw signed transaction (§3b) | **4,096 chunks / 164 KB** |
| ~~`mt qr`~~ | ~~QR symbols + legend~~ | **DEFERRED out of v0.1 — §0a** | | |

`mt qr` decides how many symbols that takes, at what error-correction level,
across how many plates, and what is engraved beside them. `mt encode` emits a
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

## 0a. `mt qr` is DEFERRED out of v0.1

**Operator ruling 2026-08-23.** v0.1 has **one verb**: `mt encode`. QR
conversion is deferred to its own cycle.

**The reason is that QR is a CROSS-FORMAT concern, not an `mt` one.** `md1` and
`mk1` will want the same conversion, and building it inside `mt` first would
mean either duplicating it for them or refactoring it out later. Where it
belongs — a shared crate, the toolkit, or `me` — is a design question that
deserves its own cycle rather than being settled as a side effect of shipping a
transaction format. The same instinct made `me` the constellation's single
`sysw` writer (§10.9).

**What it costs: NO artifact loses its path.** `mt1`'s ceiling is **4,096
chunks / 163,840 bytes** (§3), above Bitcoin's own ~100 KB standardness limit —
so every transaction that will relay can be encoded, including RCW `wsh` tier 1
at five inputs (89 chunks, **2.2%** of the ceiling).

> **An earlier version of this section said the cut "costs one artifact of
> seven", naming that same 89-chunk wallet as losing its path.** That rested on
> a 64-chunk ceiling `mt1` never had — `md-codec`'s 6-bit `count` field, which
> §3 corrected to 12 bits. The scope cut costs **machine engraving**, not
> transaction sizes.

**What it removes from v0.1**, all of it QR-only: §4's entire configuration
search, §5's plate legend, the `sysw` transaction `Class`, the record framing,
§8.7c's transport ceiling, and §10.17's firmware work. Four of the open
questions in §10 go with them. **Those sections are retained in this document
rather than deleted**, because the measurements behind them are real and the
next cycle starts from them — but nothing in them binds v0.1.

**What remains is small enough to state in a sentence:** read a finalized PSBT,
validate it per §8, and emit an `mt1` chunked codex32 string on stdout, with
warnings on stderr.

> **A consequence worth naming: a v0.1 plate carries the string and nothing
> else.** §5's legend is `mt qr`'s, and `mt encode`'s layout is the operator's
> by ruling (§3b) — so no `BEARER` line, no `FROM`/`TO`, no locktime line
> reaches the steel unless the operator puts it there.
>
> **`mt encode` therefore PRINTS suggested legend text on `stderr`**, which the
> operator may engrave beside their strings. `mt` does not control the layout
> and does not withhold the words.
>
> **It is NOT §5's five fields, and an earlier version of this section said it
> was — U-5.** §5's set was designed for a `mt qr` plate, where every symbol
> sits beside one legend. Hand-engraved strings split the text in two:
>
> | printed | text |
> | --- | --- |
> | **once** | `BEARER…`, `FROM`, `TO`, `LOCKED TO BLOCK n ~SEASON year` |
> | **per string** | `n/m` — string `n` of `m`, which `mt` knows exactly |
>
> **`PLATE n OF m` is dropped, because `mt` cannot compute `m`.** §3b rules that
> how many plates you use is the operator's decision, so the denominator would
> be invented — and **`PLATE 1 OF 1` cut onto each of five plates is a false
> completeness claim on permanent steel**, read by someone who then stops
> looking for the other four.
>
> **`STRING n OF m` is strictly better anyway**, not merely computable: it names
> which *data unit* is missing rather than which plate, and it survives any
> layout — three strings per plate, one per plate, all of them on one.
>
> **Set membership needs no legend field at all.** The header packs its
> invariant fields first — `version(4) + chunked(1) + chunk_set_id(20) +
> count(12)` — so bits 0–36 are identical across a set, and at 5 bits per symbol
> **the first 7 characters after `mt1` are the same on every string in it**.
> Verified on real `md1` output, where four chunks of one wallet all read
> `md1fveszps…`. A recoverer groups plates **by eye, without decoding**, so
> `mt encode` prints the shared prefix once and tells them the rule:
>
>     All 14 strings begin `mt1qzrf8x`. Strings sharing that prefix belong
>     to this transaction; strings that do not, do not.

## 1. The operator's decisions, recorded

Each of these is a ruling, with the reasoning that produced it. Several
overturned an earlier assumption and are marked.

1. **`mt` is the constellation's fourth format tool, with the same verbs as the
   others.** Operator ruling 2026-08-23: **`encode`**, plus **`decode`**,
   **`verify`** and **`inspect`**. `md` and `mk` both carry exactly this set —
   neither has a verb named `string`, and both call the emit path `encode`.

       md encode  ->  descriptor      ->  md1 string(s)
       mk encode  ->  key card        ->  mk1 string(s)
       mt encode  ->  finalized PSBT  ->  mt1 string(s)

   **This renames the previous draft's `mt string`.** That name only made sense
   as a contrast with `mt qr`; with the QR verb deferred (§0a) the contrast is
   gone, and `encode` is what a user who already drives `md` will reach for.

   **`decode` is not optional, and §9 said the opposite until 2026-08-23.** §9
   claimed v0.1 shipped no decoder, which was written when "reading a plate"
   meant the deferred static-scan verb. §9 now carries the retraction and the
   distinction that resolves it: **optical reading stays deferred; reassembling
   `mt1` strings does not**, because it needs no scanner and no camera. See
   decision 1a above.

   **`verify` and `inspect` follow the siblings, whose division is consistent
   across all three.** Read from their own help text:

   | | `verify` | `inspect` |
   | --- | --- | --- |
   | `md` | *"Verify backup strings re-encode to a given template"* — `--template` **required** | *"Decode + pretty-print everything the codec sees"* |
   | `mk` | *"BCH check + **optional** content match"* | *"structural commentary in addition to decode"* |
   | `ms` | *"is valid (and **optionally** round-trips against a phrase)"* | *"structural fields and decoder verdict"* |

   **`mt verify` is STRUCTURAL ONLY.** Operator ruling 2026-08-23, and it is
   what the siblings already do — **none of the three touches external state.**
   It checks: every string parses, every BCH checksum holds, the set is complete
   (`count` chunks, indices `0..count-1`, no duplicates), every chunk carries the
   same `chunk_set_id`, and the reassembled transaction re-derives that id.

   **`verify` REPORTS ITS MARGIN, not just its verdict.** Usability journey
   walk, U-2 — the one Critical it found, and five correctness rounds had missed
   it because nothing in the spec was *wrong*; a step was simply silent.

   BCH corrects up to **`t = 4` symbol errors per chunk** (§3a). A plate miscut
   in four places therefore **passes `verify` as OK** — while sitting **one
   scratch from unrecoverable**, with §1.8's zero redundancy behind it and no
   second copy unless the operator made one. A verdict that hides how much of
   its budget it just spent is telling the operator the opposite of what they
   need.

       mt verify: OK — 14 chunks, set 0x0e17e, transaction re-derives.

         CORRECTION APPLIED. 3 chunks needed repair:
           chunk  2   1 of 4 symbols
           chunk  7   4 of 4 symbols   <-- NO MARGIN LEFT
           chunk 11   2 of 4 symbols

         Chunk 7 is at its correction limit. One more damaged symbol in
         that string and this transaction is unrecoverable. Re-cut it.

   **`verify` still returns OK** — the transaction *is* recoverable today, and
   inventing a refusal would overrule the operator on their own plate. What
   changes is that the margin is **stated**, so re-cutting one string is a
   decision they can make rather than one they never knew was available.

   **It never asks a node.** A predicate whose answer changes between runs is not
   a predicate, and keeping `verify` offline means it runs on an air-gapped
   machine — which is this constellation's posture. Chain questions live in
   `inspect` (§6a), where they are reported as observations rather than folded
   into a verdict.

   **Optionally, `--transaction <psbt|hex>`** — the sibling round-trip. `mt`'s
   form is unusually strong: because the content id **is** the txid (§10.13 c),
   comparing a supplied transaction against the set's id is a cryptographic
   round-trip rather than a structural comparison. `md verify` can only re-encode
   and diff; `mt verify` can prove identity.

   **`inspect` reports what is IN the artifact**: chunk count and indices, the
   set id, and the decoded transaction's own facts — outputs, fee, locktime,
   per-input value provenance, and **plate liveness** (below).

   **`inspect` consults the local node automatically when one is reachable.**
   Operator ruling 2026-08-23, matching §6a's *"the operator is asked for
   nothing"*. This is what lets `inspect` produce its full report **from an
   `mt1` string alone** — the decoded transaction carries its inputs' outpoints
   but not their values, so without a node the fee and provenance rows are
   simply unavailable. With one, `gettxout` supplies both.

   > That repairs §1's claim rather than weakening it: *"the operator and the
   > 2040 recoverer see the same output"* holds whenever the recoverer has a
   > node, and when they do not, `inspect` **names the rows it could not
   > produce** exactly as §6a enumerates its skipped checks. Third use of the
   > same pattern — the node rescues a raw-transaction payload (§8.2e), the node
   > answers unspentness (§8.5), the node completes this report.

   **PLATE LIVENESS is its own row, and it has FOUR states, not two.** Operator
   ruling 2026-08-23: *"a transaction may be invalid because its input has been
   spent, which is different than its input hasn't been broadcast yet."* Those
   are opposite situations for a recoverer and `gettxout` alone conflates them —
   it returns a bare `null` for both.

   | state | how `mt` knows | what the recoverer does |
   | --- | --- | --- |
   | **LIVE** | `gettxout` returns a value | broadcast it |
   | **DEAD** | `null`, **and** `getrawtransaction` finds the parent | the input was spent by someone else. **The plate is scrap** |
   | **PENDING** | `null`, **and** the parent is not found | the parent transaction was never confirmed. **The plate may still become live** — find out what happened to the parent |
   | **UNKNOWN** | `null`, and no `-txindex` | `mt` cannot distinguish DEAD from PENDING and says so |

   **The parent lookup needs `-txindex`**, which most nodes do not run:
   `getrawtransaction` *"only returns a transaction if it is in the mempool. If
   `-txindex` is enabled"* it resolves any confirmed transaction. So `mt` uses
   the index when it is there and **reports UNKNOWN rather than guessing** when
   it is not — never printing DEAD on evidence that cannot distinguish it from
   PENDING.

   > **Telling a recoverer their plate is scrap when it is merely early is the
   > worst error available here**, because it is the one that gets a live plate
   > thrown away.

   **`inspect` OWNS the report; `encode` CALLS it.** Operator ruling
   2026-08-23. `encode` does not compose its own version of §10.10's report — it
   invokes `inspect` on what it just produced and appends the rows only it can
   know: **how many strings to cut, and how many characters in total.**
   `md inspect` cannot say how many plates an `md1` string takes either; that is
   not the codec's business.

   > **The point of the ownership rule is that the two CANNOT DRIFT.** If
   > `encode` composed its own report, the operator's pre-engraving view and the
   > recoverer's post-hoc view would be two implementations of the same thing,
   > free to disagree — and this artifact has already produced that defect twice
   > (§7's mitigations naming legend fields §5 had deleted; §11 asserting a chunk
   > rule §3b had retracted). With `inspect` as the single owner, **the operator
   > and the 2040 recoverer are looking at the same output**, and `inspect` is
   > independently testable in a way an inline report inside `encode` would not
   > be.

1a. **`mt decode` reads `mt` output and emits BROADCASTABLE HEX, and ships in
   v0.1.** Operator ruling
   2026-08-23: *"we need a decode to read mt output."* It takes `mt1` strings —
   from a file, from stdin, typed or pasted, in any order — and reassembles the
   transaction.

   **What it must do, and each of these is a property the format already
   provides rather than new machinery:**

   | step | what makes it possible |
   | --- | --- |
   | accept chunks in any order | `index` in every header (§10.13 a2) |
   | know when the set is complete | `count` in every header |
   | reject chunks from a different transaction | the 20-bit `chunk_set_id` |
   | correct a miscut character | BCH, `t = 4` per chunk (§3a) |
   | **prove the result is the right transaction** | re-derive the content id from the decoded transaction and compare (§10.13 c) |

   That last row is the one that matters. It is the *"funds-load-bearing
   invariant"* `md-codec`'s own source names, and it is what turns `decode` from
   a convenience into the check that the engraving round-trips at all.

   **Its output is raw transaction HEX on stdout — not a PSBT, not JSON, not a
   pretty-print.** Operator ruling 2026-08-23, settled by checking what the
   ecosystem's broadcast paths accept:

   | endpoint | accepts |
   | --- | --- |
   | `bitcoin-cli sendrawtransaction` | *"The **hex string** of the raw transaction"* |
   | Esplora `POST /tx` | *"The transaction should be provided as **hex** in the request body"* |
   | Esplora `POST /txs/package` | a JSON array of **hex** strings |

   **Hex is the only format that reaches all of them without conversion**, and
   the recoverer's last step is always a broadcast. So `decode` hands them
   exactly what the next command wants:

       mt decode < plates.txt | xargs bitcoin-cli sendrawtransaction

   This closes the pipe `mt` sits in the middle of: **hex or PSBT in
   (§8.2e), `mt1` strings onto steel, hex back out.** Everything human goes to
   stderr at both ends, so the pipe stays clean.

   **`decode` is also how `encode` gets tested.** A format whose encoder has no
   decoder can only be verified against itself; with both, every artifact in §3b
   becomes a round-trip vector.

1b. **One ENGRAVING form in v0.1.** `mt qr` is deferred to its own cycle
   (§0a) because QR conversion is a cross-format concern `md1` and `mk1` share. `mt qr` is deferred to its own cycle
   (§0a) because QR conversion is a cross-format concern `md1` and `mk1` share.
   The two-verb design below is retained as the eventual shape.
1c. ~~Two engraving verbs, `qr` and `string`.~~ **Superseded**: the engraving
   split is `encode` now and `qr` later (§0a); the verb set is `md`'s. Signed, finalized
   transactions only. Transaction construction and PSBT presentation are wallet
   functions and are out of scope (§9). **This overrules the previous draft's
   produce/present/engrave triple**, which split on *stage of the transaction*;
   these two split on *how the steel is cut*.
1d. **`mt encode` exists so a transaction can be HAND engraved, with fault
   tolerance.** Operator ruling 2026-08-23: *"For some shorter transactions,
   users will want codex32 style fault tolerant hand engraving."* It gives `mt`
   the human-readable, error-correcting property the rest of the constellation
   is built on, and makes it usable by someone with no SeedHammer.

   > **The original wording said "without it, the only route onto steel is a
   > machine" — and §0a inverted that.** With `mt qr` deferred, `mt encode` is
   > not the alternative to the machine, it is **the only route at all**, and
   > everything up to the 4,096-chunk ceiling goes through it. The ruling's word
   > *"shorter"* described a verb that had a sibling; it now describes the whole
   > tool. Nothing about the format changes — but nobody should read this item
   > as scoping `mt encode` to small transactions, because §8.7b's ceiling is the
   > only bound and it sits above Bitcoin's own relay limit.
1e. **The human text surface: what `mt` suggests engraving, and what it accepts
   back.** Operator rulings 2026-08-23. A string leaves `mt` as text and comes
   back typed by a person, and both ends need rules.

   **Engrave UPPERCASE; accept anything.** `mt` suggests uppercase because it is
   more legible on steel — fewer ascenders and descenders, more distinct
   letterforms under a scratch — and the fork's own keyboard path already emits
   it. **Input is case-insensitive**, because bech32 treats all-upper and
   all-lower as identical and normalising costs nothing. (Mixed case is invalid
   *bech32*; `mt` normalises before that rule bites.)

   **Spaces are stripped on input, and offered on output.** `mt encode` takes an
   optional grouping — every N characters, space-separated — **for hand
   engraving only**, since a person cutting 90 characters needs somewhere to
   keep their place. Whatever grouping the operator chose, `mt decode` and
   `mt verify` strip whitespace before doing anything else.

   **A full string is exactly 90 characters**, and that is checked *before*
   decoding, because it catches the one damage class BCH cannot:

       string 7: 89 characters (expected 90) — a character is MISSING, not
                 wrong. BCH repairs substitutions; an omission shifts every
                 symbol after it and cannot be corrected. Re-read the plate.

   **Confusable characters are autocorrected FIRST, and the order is the
   point.** bech32's data charset excludes `1`, `b`, `i` and `o` *precisely*
   because they are confusable, so a typed excluded character is not a wrong
   symbol — **it is not a symbol at all**, and BCH never sees it. Repairing it
   before decoding therefore **costs nothing from the `t = 4` budget**, which
   stays available for genuine substitution errors:

   | typed | meant | why |
   | --- | --- | --- |
   | `o` | `0` | excluded from the charset |
   | `b` | `6` | excluded |
   | `i` | `l` | excluded |
   | `1` **in the data** | `l` | `1` is the separator, never data |
   | `l`/`I` **in the prefix** | `1` | the separator, which every user types |

   That last row matters most: the prefix is `mt1`, so **every string a person
   types contains the single most confusable glyph in the set**, and `mtl…` or
   `mtI…` does not merely fail its checksum — it has no separator and will not
   parse at all.

   **Autocorrect announces itself, localises, and states its verdict.**
   Operator ruling: never silently. A silent fix means the operator never learns
   which engraved glyph reads badly, and so never re-cuts it before the next
   scratch lands there.

       string 3: corrected `o` -> `0` at position 41. Checksum now valid.
                 That character reads badly on your plate — consider re-cutting it.

       string 9: corrected `b` -> `6` at position 12. Checksum STILL INVALID.
                 mt1qzrf8xk2v...9d7b4...
                            ^ here            <- could not resolve

2. **Its own repository**, `mnemonic-transaction`, with **`mt-cli` and
   `mt-codec`** — matching the constellation's pattern exactly, and not a
   subcommand of `me`. Every normative format has this shape: `descriptor-mnemonic`
   is `md-cli` + `md-codec` for `md1`, `mnemonic-key` is `mk-cli` + `mk-codec`
   for `mk1`, `mnemonic-secret` is `ms-cli` + `ms-codec` for `ms1`. **`mt-cli`
   builds the `mt` binary**, as `md-cli` builds `md`. (An earlier draft said
   "`mt-codec` and an `mt` CLI", which named the binary where the siblings name
   the crate — a rename that is cheap now and annoying after a release.)
   `me` is the one repo with no codec, because it defines no format; that is
   precisely why `mt1` cannot live there. **This overrules the recommendation in
   §Section 1 of the brainstorm**, which argued `mt` had no wire format left to
   define and belonged next to `me bundle`. See §2 for what the codec does in
   fact specify; the objection was answered rather than ignored.
3. **The QR carries the standard form, never a codex32 string** (F-234).
4. **UR is dropped entirely. Both verbs share the `mt1` chunk header and NOTHING
   ELSE** — each medium carries the error correction native to it (§3a). The QR
   payload is **bech32 uppercase**, the constellation's own alphabet.
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

...and, for `mt encode`, **the string format itself**: the `mt` HRP (rendering as `mt1…`,
where `1` is bech32's separator — §10.13b), the chunk
header, and the BCH checksum that makes hand engraving fault-tolerant (§3b).

> **CORRECTION — the previous draft said the opposite, and it is worth saying
> why it was wrong.** It read: *"It is a plate format rather than a string
> format, which is why it has no bech32 HRP and no BCH checksum."* Adding
> `mt encode` falsifies that sentence outright. `mt-codec` now defines a bech32
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
2026-08-23. It carries `version`, a 20-bit `chunk_set_id`, `count` and `index` —
n-of-m **plus a set identifier**, so symbols from two different transactions
cannot be combined. That is strictly stronger than UR, which has a payload
checksum but no set identity, and it means **one fragmentation scheme to
specify, test, teach a recoverer, and get wrong only once.**

> **`mt1` WIDENS `count` and `index`, and an earlier version of this section said
> the header was shared "verbatim" with `md-codec`. That was unbuildable.**
> R3 lens 3 found it. `md-codec`'s header packs
> `version(4) + chunked(1) + chunk_set_id(20) + count−1(6) + index(6) = 37 bits`,
> against `mt1`'s
> `version(4) + chunked(1) + chunk_set_id(20) + count−1(12) + index(12) = 49 bits`
> (`crates/md-codec/src/chunk.rs`), and `write()` refuses any `count` outside
> `1..=64` with `ChunkCountOutOfRange`. **Six bits caps a set at 64 chunks** —
> while §3b's own table measures the largest `mt qr` artifact at **96**, and
> §3b and §8.7b both **stated at the time** that the 64-chunk ceiling was what
> distinguished `mt encode` from `mt qr`. The ruled encoding could not be
> written by the ruled header. (Both of those sentences are gone: the ceiling is
> now 4,096 for **both** verbs, and what distinguishes them is what a chunk
> *costs*, not how many are permitted.)
>
> **`mt1` therefore uses 12 bits each for `count` and `index`** — a **49-bit**
> header admitting **4,096 chunks = 163,840 bytes**, `mt1`'s ceiling for **both**
> verbs.
>
> **Why 12, and the bound is Bitcoin's rather than ours.** Operator ruling
> 2026-08-23. `MAX_STANDARD_TX_WEIGHT = 400,000` (verified in `bitcoin`
> 0.32.101's `policy.rs`) is 100,000 vbytes, so **a transaction above ~100 KB
> will not relay** and `mt` could never usefully engrave one. That is **2,500
> chunks**; 4,096 covers it with 1.6x headroom. An 8-bit field gave 10 KB and
> would have refused an ordinary 20-input multisig spend; 14 bits would give
> 640 KB, six times what any node accepts, bought for nothing.
>
> **The cost is ONE CHARACTER per engraved string.** `md-codec` sizes chunks
> against a 320-bit budget that sits *below* codex32's 400-bit capacity, so a
> wider header does not change the chunk count — it consumes slack. Measured: a
> chunk-string goes from **89 to 90 characters**, so a five-chunk transaction
> goes from 445 to ~450. Both `41 + 320 = 361` and `49 + 320 = 369` fit the
> 400-bit capacity.
>
> **Sizing this field for hand engraving would have been sizing the wrong
> constraint.** Nobody hand-cuts 2,500 strings whatever the format permits —
> *effort* limits that, not `count`. The header must serve the largest consumer,
> which is the machine path (§0a), and it is the one field that cannot be
> widened after v0.1 without breaking the wire format. That is consistent with §10.13, which already forks the
> codec with its own NUMS constant and HRP rather than reusing `md-codec`'s; the
> fork extends to the field widths. Cost is **4 bits per chunk**: 48 bytes on the
> 96-chunk artifact, which changes no plate count.
>
> **What is shared is `mt1`'s header, identically across both verbs** — not
> `md-codec`'s.
>
> **CORRECTION: an earlier version of this box said `mt encode` "keeps the
> 64-chunk limit because that is a property of the codex32 container". That is
> false, and it was mine.** codex32 limits a **single string** — 80 data symbols
> plus 13 checksum, `BCH(93,80,8)` — and says **nothing** about how many strings
> form a set. The 64 comes entirely from `md-codec` writing `count` into **6
> bits**, which `mt1` no longer shares. **`mt1`'s ceiling is 4,096 chunks for
> both verbs**, and every artifact measured in §3b fits it many times over.
>
> **Why 64 was right for `md1` and wrong for `mt1`, measured.** Encoding this
> repo's pathological wallet with the real `md` binary: the keyless template is
> **4 chunks**, and the keyed form carrying all **11 xpubs is 23 chunks** — about
> a third of 64. `md-codec`'s bound has ~3× headroom over the worst real
> descriptor. The same wallet's five-input **spend** needs **89 chunks**, because
> a transaction carries the witnesses, signatures and script paths a descriptor
> only describes. `mt1` is a different format with different sizing, which is
> why it has its own codec.

    mt encode:  mt1 chunk -> BCH + codex32 text -> engraved as characters
    mt qr:      mt1 chunk -> bytes              -> engraved as a QR symbol
                ^ identical header both ways

**Consequence: §10.13 now gates both verbs, not one.** Whether `md-codec`'s
header and reassembly take a transaction-shaped payload cleanly was already
open; it is now load-bearing for everything `mt` emits.

**What a symbol carries: `mt1` chunks, bech32 UPPERCASE.** Measured
(`RESULTS_ecc_selection_2026-08-22.txt`, `qr_payload_forms`), all four
candidates carrying the same chunk header:

| form | efficiency | worst plate cost | usable in a `sysw` record? |
| --- | --- | --- | --- |
| codex32 string inside the QR | 63–65% | +2 plates | yes |
| bytes + base45 — *rejected, see below* | 85.5–86% | — | **NO** |
| **bytes + bech32 UPPERCASE** | **80.3–80.7%** | **+1 plate** on one artifact | **yes — chosen** |
| bytes, raw binary | 88.4–88.8% | — | no |

> **base45 was chosen on 2026-08-23 and is REVERSED here, because it cannot
> reach the machine.** R2 lens 3 found the collision. **base45's alphabet
> contains SPACE** (index 36, RFC 9285), and EPD §6.4 — the `sysw` record rule —
> is normative and emphatic:
>
> > *"Every record MUST be the canonical, unbroken string — **no interior
> > spaces, no hyphens, no grouping of any kind**."*
>
> **The reason is about engraving, not parsing**, and it is why the rule does not
> bend: records engrave **verbatim**, so *"a record carrying separator characters
> the BCH checksum never covered turns a scratch on the operator's only copy into
> silently-absorbed damage rather than a detected error."* A character outside
> the checksum's coverage is a hole in the guarantee, cut into the only copy.
>
> **EPD §6.4 HAS A SECOND CLAUSE — ALL-LOWERCASE — AND I DENIED IT IN THIS
> SPEC.** R3 lens 3 reported that bech32 uppercase collides with it, citing
> `design/SPEC_encrypted_payload_delivery.md:806-825` by exact line range. In
> commit `52ad001` I **refuted that Critical**, writing that EPD §6.4 carries no
> lowercase clause and that the rule belongs to EPD §6.6's hashing. **That was
> wrong.** I checked `SPEC_systemwide_payloads.md` — a secondary document that
> quotes EPD §6.4 *in part* — found no lowercase clause in the fragment, and
> concluded the clause did not exist, without opening the file the reviewer had
> named. The primary source says:
>
> > *"**All-lowercase.** … without this the same wallet has two spec-legal
> > encodings — and therefore two different EPD §6.6 hashes. … **Pinned here at
> > EPD §6.4, not inside EPD §6.6**, so the engraved artefact and the hash agree by
> > construction."*
>
> It states the proposition I denied, in the terms I denied it, and its last
> sentence pre-empts my exact reasoning. **A partial quote in a secondary
> document is not the clause** — a negative inherits the scope of the search
> that produced it, and mine searched the wrong file.
>
> **The design survives; the justification did not.** The same commit ruled that
> the record stores lowercase and `mt` uppercases only for the QR, which
> satisfies EPD §6.4 as actually written. So bech32 remains correct — but for a
> reason the spec had stated falsely, and a reader would have learned that EPD §6.4
> has no case rule.
>
> **This is the third format to collide with EPD §6.4, and the precedent is
> settled.**
> `FreeText` and `Passphrase` hit it too, and *"the exemption is refused —
> relaxing EPD §6.4 for two classes would weaken the rule for all of them."* They
> were hex-encoded instead, at 2×. Hex-escaping base45 would land at **48.5%**,
> worse than raw binary and worse than the UR this cycle dropped for waste.

**Why bech32 uppercase satisfies all three constraints at once**, which is why it
is the constellation's alphabet rather than a stylistic choice:

| constraint | bech32 uppercase |
| --- | --- |
| **EPD §6.4** — no interior spaces; every character inside the checksum | ✓ 32-character alphabet, no space |
| **EPD §6.4 — ALL-LOWERCASE**, a second clause of the same rule | ✓ **only because the record stores lowercase.** bech32 is case-insensitive by design and uppercase→lowercase is lossless (verified 1:1), so the payload survives the constraint — but the *record* must be written lowercase, not merely be convertible |
| **which case is STORED** — the record and the QR are different artifacts | the `sysw` record stores **lowercase**; `mt` uppercases **only** when encoding the QR symbol, where alphanumeric mode needs it. The uppercase form never reaches a record |
| **QR alphanumeric** — for 11-bits-per-2-characters packing | ✓ when uppercased |

The rejected base45 satisfies only the third; hex satisfies the first two at
twice the cost.
`md1`, `mk1` and codex32 already store lowercase and uppercase for QR, so
`mt qr` and `mt encode` now share one alphabet.

> **Correction to a figure I quoted while recommending this.** The 91% measured
> for bech32 in `RESULTS_qr_modes_2026-08-22.txt` is for a **bare** payload. With
> `mt1` chunk headers added before encoding, the measured figure is **80.4%** —
> the overhead compounds. The plate consequence is one extra plate on RCW `wsh`
> tier 1 at five inputs (5 → 6) and no change on the other four artifacts.

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

    mt encode:  chunk header + payload -> BCH + codex32 -> engraved characters
    mt qr:      chunk header + payload -> bech32U -> QR (Reed-Solomon) -> modules
                ^ identical header, medium-appropriate correction
                ^ and PER-CHUNK conversion in both (below)

**The base32 conversion is PER CHUNK, never over the concatenated stream.**
Operator ruling 2026-08-23. `mt encode` has no choice here — codex32 is
per-chunk by construction, each chunk becoming a complete string with its own
HRP and checksum — so this rules the only verb where the question arises,
`mt qr`, **to follow the convention `mt encode` already has.**

**Why, and it is not the size.** Measured on the 3,809 B artifact: per-chunk is
**7,054 characters**, whole-stream **7,016** — a 0.5% saving for whole-stream.
What per-chunk buys instead:

- **One chunking rule across both verbs.** A recoverer's chunk 7 is byte-
  identical in either medium before the medium-specific encoding, which is what
  makes §3a's "identical header" claim true at the byte level rather than only
  at the field level.
- **Chunk independence, which is the point of chunking.** Whole-stream couples
  every chunk's characters to every byte before it, so a damaged chunk shifts
  its neighbours' alignment.

> **The failure mode if two implementers split here is silent and
> misdiagnosed** — R5 readiness computed it. The two strings **share no
> character after position ~74**, yet the first chunk still parses with a valid
> header. The corruption surfaces only at the content-id compare, which reports
> *"this is a different transaction"* — pointing the recoverer at the wrong
> plate rather than at the wrong software.

## 3b. The string form: `mt1`, for hand engraving

**`mt encode` emits a chunked codex32 string with BCH error correction**, in the
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

A chunk carries **40 payload bytes** and `mt1`'s header admits **4,096 chunks**,
so the ceiling is **163,840 B** — above Bitcoin's own ~100 KB standardness limit,
so `mt1` encodes any transaction that will relay (§3). (An earlier draft said 64 chunks / 2,560 B,
inheriting `md-codec`'s 6-bit `count` field that `mt1` does not use — see §3.) Measured
(`RESULTS_envelope_2026-08-22.txt`, `RESULTS_rcw_2026-08-22.txt`):

| artifact | raw bytes | chunks | fits? |
| --- | --- | --- | --- |
| RCW `tr` key-path, 1-in/1-out | 162 | **5** | yes |
| RCW `tr` tier 4, 1-in/1-out | 405 | **11** | yes |
| RCW `tr` tier 1, 1-in/1-out | 535 | **14** | yes |
| RCW `wsh` tier 1, 1-in/1-out | 742 | **19** | yes |
| RCW `tr` tier 1, 5-in/2-out | 2498 | **63** | yes, barely |
| RCW `wsh` tier 1, 5-in/2-out | 3538 | **89** | yes — 2% of `mt1`'s 4,096 |

**Both verbs share the 4,096-chunk ceiling**, because both use `mt1`'s header.
What differs is what a chunk *costs*: one chunk is one hand-cut string of ~96
characters, or about 1/24th of a machine-engraved QR symbol. **The same count is
two orders of magnitude apart in human effort**, which is why §8.7b warns in
characters and the deferred QR verb would warn in plates and minutes.

> **CORRECTION — every number above was ~13% low until 2026-08-23, and so was
> the ceiling.** R0 round 1 (S-1) found that the probe helper feeding all of
> them modelled a chunk as `(bytes*8).div_ceil(363)`. 363 = 80 codex32 symbols
> x 5 bits − 37 header bits, i.e. what a chunk *could* carry if the chunker
> **filled** to long-form capacity. **It does not.** `md-codec` sizes chunks by
> `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64 * 5 = 320` bits
> (`crates/md-codec/src/chunk.rs:224`), applied over `payload_bytes.len() * 8`
> (`crates/md-codec/src/chunk.rs:253-254`) — **40 bytes is the CEILING the chunk
> count is derived from, not the size of each chunk.**
>
> **An earlier version of this box called it "a flat 40 bytes per chunk", and
> that mis-describes the chunker — R4 lens 1.** `md-codec` computes
> `chunks_needed` against the 320-bit ceiling and then splits the payload
> **`bytes_per_chunk = ceil(len / count)`**, each chunk taking that many bytes
> and the **last taking whatever remains** (`crates/md-codec/src/chunk.rs:267-273`).
> No chunk is padded to 40.
>
> **An intermediate version of this box said "the last chunk is not a short
> remainder", and that describes a different split — R5 readiness.** Under
> `ceil` the last chunk *is* the remainder and is normally shorter: a 535-byte
> payload over 14 chunks gives `ceil(535/14) = 39` bytes each for the first
> thirteen and **28** for the last. Two implementers, one following the sentence
> and one following the code, produce different chunk boundaries and therefore
> **plates neither can read**. Correcting the flat-40 error introduced this one
> in the same paragraph. The **chunk
> counts in this spec are unaffected** — they derive from the ceiling, which is
> what `chunks_needed` uses — but the **per-chunk sizes** differ on any payload
> that is not a multiple of the chunk count. This is the same error class as the
> 363-vs-320 correction above: a limit read as a rule.
>
> **`mt1` balances too**, which §10.12 already implies by forbidding fill, and which
> §4's *"never leave redundancy unbought"* requires: a padded chunk spends plate
> area on nothing.
>
> **The error was per-chunk, and it is easiest to see there.** The old model
> put **45.4 payload bytes** in a chunk where the chunker puts **40** — about
> 13% too many — so every chunk count derived from it was that much too low.
> At the time the chunk count was capped at 64, so the mistake also showed up as
> a **2,904 B versus 2,560 B** total ceiling, and a transaction inside that
> 344-byte band would have been called "fits" and then returned
> `ChunkCountExceedsMax`. Those two totals are themselves now historical — the
> cap is 4,096 chunks (§3) — but the per-chunk figure is the durable part and it
> is what §3b's table rests on.
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

**`mt encode` emits a string. That is the whole of its output.** Font size,
characters per plate, how many plates, what order they are laid out in, whether
the string is cut by hand or by machine, and whether anything is engraved beside
it are all the user's decisions. This spec does not constrain any of them, and
§4's configuration search does not apply to this verb.

An earlier version of this section derived a chars-per-plate table from the
fork's font ladder and drew plate counts from it. **That was out of scope and is
deleted.** What survives from it is the one part that *is* a property of the
codec rather than of anyone's steel: the **4,096-chunk ceiling** above, which
binds regardless of how the string is engraved, and which §8.7b refuses
against.

> **The distinction that decides what belongs here:** what `mt` *emits* is this
> spec's concern; what a user does with steel is not. `mt qr` is the exception
> only because it emits an engraving, so plate geometry is part of its output.

### The one thing `mt encode` does say about the plate

> **Ruling, operator, 2026-08-23:** *"Hand cut plates get a warning on stderr.
> And that's it."*

`mt encode` prints a warning at encode time that the artifact is **bearer** —
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
CAN SPEND IT` as the first line of a legend `mt` controls, and `mt encode` has
no such mechanism because it emits no engraving. §7 records it as an accepted
risk, not as a mitigation.


## 4. Choosing the configuration — `mt qr` only, DEFERRED (§0a)

**This section governs `mt qr` and nothing else.** `mt encode`'s layout is
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
> every row as a lower bound: the **49-bit `mt1` chunk header per symbol**
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
plate exists, 0.60 mm (two strokes) is what `mt` SUGGESTS** — not a floor it
enforces. **Operator ruling 2026-08-23 (§10.1, §8.8): the operator picks from
every size `mt` can engrave, with 0.60 mm suggested.** `mt` says at the point of
choice that finer modules are optically unvalidated; it does not refuse them.
The 0.30 mm results are recorded for when the plate exists.

> **This paragraph stated a hard floor until 2026-08-23 and was missed when the
> ruling landed.** R2 lens 1 (F-3) found it: commit `fc4179c` rewrote the rule in
> §8 item 8 and §10.1 and never touched §4, so the spec carried both the new rule
> and the old prose. **No superseded-term sweep could have caught it** — every
> word in the sentence was still current, and only the modal verb changed.

## 5. The plate legend — `mt qr` only, DEFERRED (§0a)

> **Retained for the deferred QR cycle, and for one live purpose:** §0a rules
> that `mt encode` **prints these five fields on `stderr`** as suggested text
> the operator may engrave beside their string. The measurements and the field
> choices below are what that suggestion is made of.


Everything constellation-specific lives here, in engraved text, never in the QR.

**The legend carries only what a human needs BEFORE the QR is decoded.** Five
fields, **141 characters**, 6 lines — measured,
`RESULTS_legend_budget_2026-08-22.txt`:

| field | chars | why |
| --- | --- | --- |
| `BEARER - ANYONE HOLDING THIS CAN SPEND IT` | 41 | the plate is spendable; this is not a backup in the sense the other formats are |
| `FROM WALLET <8 hex>` | 20 | wallet id or seed fingerprint. The transaction does **not** say what it spends *from* (§6). **Optional — loudly warned when absent** (§10.4) |
| `LOCKED TO BLOCK <n> ~<SEASON> <year>` / `LOCKED UNTIL <t>` | 35 | the single most actionable fact. Reads **`NO TIMELOCK`** when there is no enforced `nLockTime`. **A statement about the transaction's fields, never about spendability** — `mt` does not evaluate scripts, so it reports the lock it read and lets the reader conclude (§8.4) |
| `TO <wallet id, fp or label>  <amount>` | 34 | names the destination **wallet**, not one truncated address — operator ruling, §10.4. **Optional — loudly warned when blank.** A free-text label is allowed **only behind an explicit flag**, since nothing can check it against the transaction |
| `PLATE n OF m` | 12 | a missing plate must be obvious, and all `m` are required (§3) |

Plus, **not part of the 141-character budget above**, one `n/m` label engraved
beside **each QR symbol**, naming the `mt1` chunk it carries (§10.8's ruling). A
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
scriptPubKey — so that payload does describe what it spends. **`mt encode`'s
does not**: a raw transaction carries outpoints only, so a string plate is
silent about both the input amounts and the source scripts. It still does not name the *wallet*, hence the stub, and
hence the stub living in text, because it is the one constellation-specific fact
on the plate and F-234 forbids that inside the QR.

> **`mt`'s INPUT is always a finalized PSBT (§10.10), even for `mt encode`,
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

**NO NODE IS A WARNING, NOT A SILENCE.** Operator ruling 2026-08-23:
*"bitcoind might not be available and we need a warning for that."* An earlier
draft made every check in this section conditional on a node being reachable and
said nothing when one was not — so the quietest possible run was also the
least-verified one, and the operator could not tell the difference. `mt` names
what it could not check:

    WARNING: no bitcoind reachable. These checks did NOT run:

      - are the inputs still unspent?        (§8.5)   UNKNOWN
      - do the PSBT's input values match
        the chain?                           (§6a)    UNKNOWN
      - has the locktime already passed?     (§8.4)   UNKNOWN
        locked to block 1383520, current height unknown

    The transaction may already be unspendable. A plate is ~21 minutes.
    Consider re-running with a node before cutting.

**Enumerating the skipped checks is the point.** *"No node"* alone tells the
operator nothing they can act on; a list of what is therefore unknown tells them
exactly what they are trading for convenience, and the plate-time reminder tells
them what it costs to be wrong. This is the same principle as §8.2c: state the
mechanism, not the caution.

**Not a refusal.** Offline operation is the constellation's posture (§0), and
§8.5 refuses only on a node's *positive* answer that an output is spent — an
absent node is an absent answer, not a bad one.

**Use the value it returns, not merely its null-ness.** `gettxout` returns
`value` and `scriptPubKey` — which is this section's stated reason for choosing
it over `getrawtransaction` — and an earlier draft acted only on whether the
result was `null`. R3's information lens (I-2) caught that: since §8.2's
removal, **the chain's own answer is the only value check `mt` has for a segwit
input**, and it was being thrown away. `mt` compares the fetched `value` against
the PSBT's UTXO record for that input and **refuses on mismatch**, naming both
numbers. This is a comparison of two integers, not script evaluation, so it sits
inside §8.4's scope ruling.

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
| **Bearer** — holder can broadcast (`mt encode`) | **accepted risk, not mitigated on the plate.** `mt` emits a string, not an engraving, so it has no mechanism to put a warning on hand-cut steel (§3b). It warns once on `stderr` at encode time, to the person encoding — who is not the person holding the plate later. The timelock bound still applies |
| **Pinned destination** — a 2040 recoverer pays a 2026 address whose keys may be lost | **cannot be fixed; partly disclosed.** §5's `TO` line names the destination **wallet** (id or fingerprint), which does not degrade with output count as the old truncated-address form did — but it is **optional**, and says nothing when the destination is not a known wallet (§10.4). `mt` displays every output in full at encode time; the plate carries a summary |
| **Indistinguishable from a watch-only plate** — an `mt1` plate sits in the same drawer as `md1` and `mk1` plates, in the same script, differing in **one HRP character**, and is the only one of the three that is spendable by whoever picks it up | for `mt qr` the `BEARER` legend line carries the difference. For `mt encode` there is **no mitigation** — see the bearer row above and §3b. R0 round 1 (R-13) |
| **Pinned fee** — a 2026 fee rate may be unbroadcastable in 2040 | **cannot be fixed by `mt`, and is NOT on the plate.** `mt` warns below 10 sat/vB (§8.2b) and names two things a future holder can try, guaranteeing neither: **CPFP** — spending one of this transaction's outputs with a high-fee child, which needs no key from the original signer, unlike **RBF**, which requires signing a replacement and is therefore useless to a plate holder — and **out-of-band submission** straight to a miner, which bypasses relay policy and is the escape hatch when a fee is too low for the parent to reach a mempool at all. **Neither is recoverable from an `mt encode` plate's own contents**, since a raw transaction carries no input amounts (§6) |
| **Silent invalidation** — one ordinary spend of any input voids the plate, and nothing on it says so | **not mitigated on the plate.** The input outpoints were cut from the legend (§5), so a holder cannot check unspentness from the plate alone — they must decode the QR first. `mt` checks it at encode time (§6a, §8.5); after that the hazard is open and undisclosed on steel |
| **Non-`ALL` sighash** — an input signed with `SIGHASH_NONE` or `SIGHASH_SINGLE` leaves outputs unbound, so a plate-holder can redirect the funds and the `TO` line becomes a lie | refused at encode time, §8.6 — **structurally**, since §8.2's removal left no script engine |
| **Wrong input value** — a legacy input whose claimed value is wrong yields a valid transaction, and **the fee absorbs the entire difference** | **not detectable by `mt`.** §8.2's removal means no signature is verified, and a legacy sighash never committed to the amount anyway. Mitigated only by §8.2c's `stderr` warning, which states the arithmetic `(real input value) − (output total)` since the output total is the one term `mt` knows for certain. **Nothing reaches the steel for `mt qr`** — §5's legend is full (§8.2c). An `mt encode` operator controls their own plate and may add a reminder; `mt qr`'s operator cannot |
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

   **The legacy warning fires only when the value is UNBOUND** — not on every
   legacy input. R3's information lens found the earlier rule actively harmful:
   it fired *"whenever any input is legacy"* while its body asserted `mt` could
   not bind the value by txid, **which §8.2d now does**. In the common case —
   a legacy input carrying `non_witness_utxo`, which BIP-174 requires — that
   printed a false, capitalised, eleven-line block, **training the operator to
   ignore the rare case where it is true.** A warning that cries wolf on the
   normal path has negative value.

   So it fires when, and only when, the value is bound by nothing: no
   `non_witness_utxo` (§8.2d), no chain fetch (§6a). It
   states the mechanism rather than a caution:

       WARNING: input 0 is a legacy (pre-SegWit) input.

       The fee you will pay is:   (what is REALLY at that input) - 0.99000000 BTC
       You have told mt it holds:  1.00000000 BTC
       So mt shows a fee of:       0.01000000 BTC

       NOTHING HAS VERIFIED THAT VALUE. This input carries no
       non_witness_utxo, so mt could not bind it by txid (see 8.2d), and a
       legacy signature does not commit to the amount either. A wrong value
       still produces a perfectly valid transaction -- and the fee absorbs the
       entire difference. If that input actually holds 10 BTC, this transaction
       pays 9.01 BTC in fees and a miner will simply take it.

       Verify the input value out of band before you cut this plate.

   > **`mt` CANNOT put that reminder on a `mt qr` plate, and an earlier draft
   > said it could — R2 lens 2 (S-3), the third recurrence of this class in this
   > artifact.** §7 named *"the engraved out-of-band reminder"* as the
   > mitigation, and §5's legend has **no such field**: it is five fields over
   > six lines, sized into §4's reservation, with no room for a sixth. So the
   > instruction only lands where the operator controls the plate — **`mt
   > string`**, whose layout is theirs by ruling (§3b). For **`mt qr`** the
   > legend is `mt`-controlled and full, so the warning reaches the operator on
   > `stderr` **before** they cut and nothing reaches the steel. §7 records that
   > asymmetry rather than claiming a mitigation `mt qr` does not have.

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
   > legacy input the claimed value is checked against **nothing by signature**.
   > **§8.2d closes part of this**: where the input carries `non_witness_utxo`,
   > `mt` binds the value by txid, which is a hash comparison rather than script
   > evaluation. The residue this warning exists for is an input whose value
   > arrives with **no** `non_witness_utxo` — supplied by the operator under this
   > refusal — where the warning and the engraved reminder are the whole
   > mitigation.

2d. **`non_witness_utxo` present but not matching the input's txid** → refuse.
   Where a PSBT input carries `non_witness_utxo` — the **whole previous
   transaction**, which BIP-174 requires for legacy inputs — `mt` hashes it and
   requires the result to equal that input's `previous_output.txid`, then reads
   the value from `output[vout]`. A mismatch is a refusal naming both txids.

   **This is a hash comparison, not script evaluation**, so it sits inside the
   2026-08-23 scope ruling (§8.4): `mt` never executes a script, never asks a
   node, and learns nothing about the wallet's policy. Forging a passing value
   would need a txid collision.

   > **Added by R2 lens 1 (F-1), which found the spec asserting this binding
   > without anyone performing it.** §8.6 accepts legacy inputs on the grounds
   > that `non_witness_utxo` *"binds the amount"* — true of the mechanism, and
   > false of `mt` until this refusal existed. An acceptance resting on an
   > unperformed check is the same defect as the original legacy refusal, whose
   > premise was also wrong.
   >
   > **This materially narrows §8.2c's hazard.** A legacy input carrying
   > `non_witness_utxo` now has its value bound by proof-of-work-anchored
   > history rather than by the operator's word. What remains unbound — and what
   > §8.2c's warning still exists for — is an input whose value arrives with
   > **no** `non_witness_utxo` at all, or by operator assertion under §8.2c.

2e. **Which serialisations `mt` accepts, and why all three.** Operator ruling
   2026-08-23, settled by checking what the tools actually hand a user:

   | form | recognised by | where a user gets it |
   | --- | --- | --- |
   | **binary PSBT** | the `psbt\xff` magic | a `.psbt` file from a wallet |
   | **base64 PSBT** | the `cHNidP8` prefix | what wallets export and display |
   | **raw transaction hex** | bare hex, no magic | **Bitcoin Core's default output** — see below |

   Each is distinguishable by inspection, so `mt` sniffs rather than asking.

   > **Core's canonical workflow ENDS in hex, which is why refusing it was
   > untenable.** `finalizepsbt` takes `extract` (boolean, **default `true`**):
   > *"If true and the transaction is complete, extract and return the complete
   > transaction in normal network serialization instead of the PSBT."* So the
   > moment a PSBT is finalized — the exact state `mt` requires — **Core stops
   > returning a PSBT and returns hex.** A user must pass `extract=false`
   > explicitly to keep the PSBT form.
   >
   > The earlier PSBT-only ruling would therefore have refused **the default
   > output of the reference implementation**, for the one transaction state
   > this tool exists to consume. That is a stronger reason than "refusing the
   > engraved bytes is unhelpful", and it is why §8.2e accepts hex rather than
   > tolerating it.

   **A raw signed transaction is ACCEPTED, with a loud warning.** Operator
   ruling 2026-08-23: *"we can't refuse raw hex signed tx. We
   have to warn loudly if they paste it and state what we can't verify."*

   **Refusing was the wrong response to someone holding the exact bytes that get
   engraved**, and it was never a special case: a raw transaction is simply the
   **no-UTXO-records** input §8.2c already covers. What degrades is narrow, and
   a node closes most of it:

   | check | PSBT | raw, no node | raw, **node** |
   | --- | --- | --- | --- |
   | §8.1 finalized | ✓ | ✓ | ✓ |
   | §8.6 satisfaction binds outputs | ✓ | ✓ | ✓ |
   | §8.2b value balance | ✓ | **✗** | **✓ via `gettxout`** |
   | the fee | ✓ | **unknown** | **✓** |

       WARNING: this is a raw signed transaction, not a PSBT.

         A raw transaction carries its inputs' OUTPOINTS but not their
         VALUES, so mt cannot compute the fee from it alone.

         [no node]   The fee is UNKNOWN. mt cannot tell you whether it is
                     0.0001 BTC or 9 BTC. Supply input values, or a node.
         [with node] mt fetched each input's value from the chain:
                     fee 0.00012 BTC, 3.2 sat/vB.

   **`mt` never refuses the bytes — it refuses to pretend it checked something
   it did not.** This supersedes the earlier PSBT-only input ruling.

2f. **A PSBT or transaction passed as a COMMAND-LINE ARGUMENT** → **refuse**,
   and tell the operator how to clean up. Operator ruling 2026-08-23.

   **A finalized transaction is a BEARER artifact** — anyone holding it can
   broadcast it, exactly like the plate it becomes. As an argument it lands in
   the shell's history file in plaintext and in `ps` output for every user on
   the machine. `mt` reads from a **file or stdin** only.

       mt encode: refusing a transaction passed as a command-line argument.

         It is now in your shell history and was visible in `ps` while this
         ran. A finalized transaction is BEARER: anyone who reads it can
         spend it.

         Remove it:  history -d 512 && fc -W        # zsh
         Then re-run: mt encode < tx.psbt

   The purge command is **specific to the operator's shell**, detected from
   `$SHELL`. Two limits stated rather than papered over: it cannot know who
   read the history before now, and it cannot reach backups.

   > **The siblings' precedent does not transfer, and the reason is the whole
   > point.** `md verify <STRINGS>...` and `mk verify [MK1_STRINGS]...` do take
   > their material as positional arguments — but `md1`/`mk1` strings are
   > **watch-only public material**, where a leak costs privacy. A finalized
   > transaction is bearer, where it costs the money. Same shape, different
   > hazard class.

2g. **The source file is readable by anyone but its owner** → **warn loudly.**
   Operator ruling 2026-08-23. `mt` checks `mode & 0o077 == 0` — no group bits,
   no other bits — accepting `600`, `400`, `700` and warning on `644`, `640`,
   `604`.

       WARNING: /home/bcg/tx.psbt is mode 0644 — readable by every user
                on this machine.

         A finalized transaction is BEARER. Anyone who can read this file
         can broadcast it. It is exactly as dangerous as the plate you are
         about to cut.

         chmod 600 /home/bcg/tx.psbt

   **It works in more cases than "a named file", which was worth checking.**
   Verified by experiment: with `mt encode < tx.psbt` an `fstat` on fd 0 still
   returns the underlying file's mode, so the redirect form is checkable too.
   Piped input (`cat … | mt`) gives a FIFO and typed input gives no file — in
   both `mt` says the permissions are **unknown** rather than silently skipping
   the check.

   Two honest limits: it says nothing about who read the file **before** now,
   and nothing about backups or directories it has passed through. It is the
   check that is available, not a guarantee.

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

   **`nLockTime` IS NOT ALWAYS A BLOCK HEIGHT, and an earlier version of this
   section assumed it was.** Verified against source:
   `LOCK_TIME_THRESHOLD: u32 = 500_000_000`. Below that value `nLockTime` is a
   **block height**; at or above it, a **Unix timestamp**. `mt` branches on the
   threshold before it compares anything or engraves anything.

   > **Two failures came from the missing branch — R2 lens 2 (S-4).**
   >
   > 1. **A permanent falsehood on steel.** A transaction with
   >    `nLockTime = 1800000000` would have engraved `LOCKED TO BLOCK
   >    1800000000` — a block number some thirty thousand years out, for a plate
   >    that actually unlocks in 2027. A holder could reasonably read that as
   >    "never" and discard it.
   > 2. **False reassurance, which this section had CLAIMED to close.**
   >    Comparing a *timestamp* against a *height* makes every timestamp look
   >    enormously distant, so `mt` would stay silent about a plate whose
   >    time-lock has **already passed** and which is spendable today. §8.4
   >    asserted that the `nSequence` rule closed false reassurance; this was a
   >    second road to it, needing no script read.

   **`mt` states the two facts and stops.** Operator ruling 2026-08-23:
   *"'may be immediately spendable' is accurate but incomplete. Just say whether
   the transaction is locked to block x and current height is y."*

   So the `stderr` report is a statement of what was read, not a verdict — with
   the units named, never mixed:

       LOCKED TO BLOCK 1383520          current height 963663
       LOCKED UNTIL 2027-03-14T00:00Z   current MTP 2026-08-23T03:00Z
       NO TIMELOCK                      current height 963663
       nLockTime 900000 present but NOT ENFORCED (all inputs final)
       LOCKED TO BLOCK 900000           current height unknown (no node)

   **Why facts beat a verdict here.** *"May be immediately spendable"* is true of
   almost any transaction and tells the operator nothing they can act on — it
   cannot distinguish a lock that has already passed from one that was never
   enforced from one still years away, and all three want different responses.
   Two numbers side by side let the operator see which case they are in. It also
   keeps `mt` inside its own scope: a height comparison is arithmetic on fields,
   whereas *"spendable"* is a claim about a transaction's fate that depends on
   scripts, fees and unspent inputs — none of which `mt` evaluates.

   **The block height is MANDATORY, and the estimate names a SEASON.** Operator
   ruling 2026-08-23: *"Estimate year and season (spring, summer, winter, fall)
   and mandate output of blockheight at unlock time."* So the legend always
   carries the raw unlock height — the one figure that is exact, consensus-
   defined, and re-derivable forever — and the estimate rides beside it as an
   orientation aid:

       LOCKED TO BLOCK 1383520 ~FALL 2034

   **The height is the fact; the season is the courtesy.** A height alone is
   meaningless to a human (§8.4's original problem) and a season alone is
   unverifiable, so the plate carries both and a reader can always fall back to
   the number.

   **Season precision is supported by the measured block rate, and this was
   checked rather than assumed.** Over three windows ending at height 963,759 the
   realised interval was **9.945 to 10.116 min/block** — within ±1.2% of the
   10-minute target — which over the 419,761 blocks of the worked example is
   **+16 to −34 days** of drift. A season is ~91 days, so the error sits inside
   one comfortably. **The exception is a projection landing near a season
   boundary**, which can tip; the `~` marks the whole estimate as approximate and
   the height beside it is what settles any dispute.

   **Seasons are NORTHERN-HEMISPHERE, by ruling.** Operator, 2026-08-23. So
   `SPRING` / `SUMMER` / `FALL` / `WINTER` are the meteorological quarters of the
   northern year — `~FALL 2034` means roughly September to November 2034 —
   regardless of where the plate is read.

   > **The residual, stated because a plate cannot be asked a question.** A
   > reader in Sydney sees `~FALL 2034` and, reading it locally, is wrong by
   > about six months. The harm is bounded and small for one reason: **the
   > mandatory block height sits beside it and is unambiguous everywhere.** The
   > height is the fact and the season is the courtesy, so a misread courtesy
   > costs an orientation, not a recovery. That asymmetry is exactly why the
   > height is mandatory and the estimate is not.

   - **Legend:** `LOCKED TO BLOCK <n> ~<SEASON> <year>` for a height,
     **`LOCKED UNTIL <time>`** for a timestamp, or **`NO TIMELOCK`** — that exact
     spelling, 11 characters, normative everywhere.

     > **This string existed in TWO spellings across four sites — `NO TIMELOCK`
     > and `NO BLOCK TIMELOCK`, 11 versus 17 characters — and §8.4 contradicted
     > itself twice (R5 readiness).** It is **engraved permanently**, so drifting
     > spelling is not a style question: two `mt` versions would cut different
     > plates for the same transaction, and a recoverer matching against
     > documentation would find neither. The 6-character difference also changes
     > what fits the line.

     **A timestamp
     is never presented as a height.**
   - **Compare like with like:** a height against the chain height, a timestamp
     against the chain's **median-time-past** — which §6a's node already
     reports, and which is the monotonic, consensus-enforced figure rather than
     the loosely-constrained header stamp.
   - **Height or MTP comes from `bitcoind` when reachable**, and is reported as
     unknown otherwise. This is the whole of `mt`'s use of the chain here — it never
     hands the transaction to the node for validation.
   **A height means nothing to a human; `mt` estimates the date.** Operator
   ruling 2026-08-23: *"estimate unlock date for time locked transactions
   assuming 10 minute block times. Will need to embed a timestamp in binary for
   reference."*

       estimated unlock  =  reference_time + (target_height − reference_height) × 600 s

   **The estimate uses the embedded constant, and ONLY the embedded constant.**
   Operator rulings 2026-08-23: *"Embed fallback timestamp blockheight in case
   bitcoind not available at compile time"*, then — simplifying — *"Use embedded
   timestamp above only ever. It's essentially constant and reasonably reliable
   as an estimate."*

       MT_REF_HEIGHT = 963_759
       MT_REF_TIME   = 1_787_507_701   // 2026-08-23T17:55:01Z

   **`mt` never consults a node for this.** An earlier draft branched — live
   height when a node was reachable, the constant otherwise — and that was
   removed as too complex. The simplification is worth more than the accuracy it
   costs, for three reasons:

   - **The answer is deterministic.** Two runs of `mt`, on any two machines,
     with or without a node, produce the **same engraved year** for the same
     transaction. Branching would have made a permanent number on steel depend
     on the operator's network.
   - **The accuracy difference is immaterial at this granularity.** The estimate
     is stated to the **year** (below), and a reference pair drifting by even a
     few months moves a projection years out by less than the rounding.
   - **It removes a whole class of question** — what if the node disagrees, what
     if it is syncing, what if it is on another chain — from a number that only
     ever orients a human.

   `MT_REF_TIME` is the tip's **median-time-past**, not its header `nTime`. MTP
   is monotonic and consensus-enforced, while a header stamp is only loosely
   constrained — it may run up to two hours fast and need not exceed its
   parent's. At capture the tip's `nTime` was `1787509876`, **36 minutes ahead**
   of its MTP; small here, unbounded in general, and baking that slack into a
   decades-long projection would be permanent.

   Provenance for whoever refreshes it: block 963,759,
   `00000000000000000000b7060d74b6540e3b2accc9cb50f2a0d428b55911a455`.

   **A NEGATIVE subtraction means the lock is already behind us — warn.**
   Operator ruling: *"If subtraction is negative, warn user transaction is not
   time locked."* When `target_height < MT_REF_HEIGHT` there is no future date to
   estimate, and `mt` says so rather than printing a past year:

       WARNING: nLockTime 900000 is BELOW this build's reference height 963759.
                This transaction is not meaningfully time-locked -- its lock
                height passed before mt was built. Treat it as spendable now.

   The legend then reads `NO TIMELOCK` rather than a `~<year>` that would be
   both meaningless and reassuring. Note this is a **separate** determination
   from §8.4's `nSequence` rule: a locktime can be unenforced *and* in the past,
   and either alone is enough to make the plate immediately spendable.

   Reporting the **current height** alongside (§8.4's two facts) is unaffected —
   that comes from the node when one is reachable and is a fact `mt` observed,
   not an input to this estimate.

   **Stated to the year, deliberately.** Three separate reasons, and they all
   point the same way:

   - **Ten minutes is a target, not a rate.** Difficulty retargeting holds the
     average near it over the long run, but the realised interval drifts with
     hashrate between adjustments. Month or day precision would claim accuracy
     the method does not have.
   - **The reference pair ages.** A binary built in 2026 and run in 2031 carries
     a five-year-old anchor, and the error grows with that gap. `mt` prints the
     reference pair alongside the estimate so the operator can see how fresh it
     is, and prefers a live node when one is there.
   - **It is engraved, and engraved numbers are forever.** The legend carries
     `~<year>` with the tilde, because a projection presented as a fact is the
     mistake §9 refuses for fiat figures. The difference that makes a year
     acceptable where a dollar amount is not: block rate is
     **consensus-targeted**, so it depends on nothing external, whereas a
     currency figure depends on everything.

   Measured cost: the legend goes from **130 to 136 characters** and stays at
   **6 lines** (`RESULTS_legend_budget_2026-08-22.txt`), so §4's reservation and
   plate table are unaffected.

   - **A lock that has already passed is reported the same way**, because the
     two numbers say so: `LOCKED TO BLOCK 900000, current height 963663` is a
     plate that is live now, and the operator can read that without `mt`
     concluding it for them.

   **`nSequence` is not optional, and omitting it causes the dangerous error.**
   `nLockTime` is enforced only when at least one input has
   `nSequence != 0xFFFFFFFF`. A transaction with every input final ignores its
   locktime — so reading `nLockTime` alone would engrave `LOCKED TO BLOCK
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
   > legend now reads **`NO TIMELOCK`**: precisely true about the fields
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

      > **BOTH SPENDING STRUCTURES, not just the witness — R2 lens 2 (S-1).**
      > An earlier version of this refusal named only the **witness**, written
      > when legacy inputs were refused. §10.16 now **accepts** them, and a
      > legacy input's signature lives in the **`scriptSig`**, which that
      > wording never examined — while §8.1 admits such an input by disjunction
      > (*"a non-empty `scriptSig` **or** a non-empty witness"*). So a
      > `SIGHASH_NONE` **legacy** input would have passed every refusal here
      > with its outputs unbound, making §7's *"refused at encode time"* false
      > and the plate redirectable by any holder. `mt` inspects **`scriptSig`
      > and witness alike**, applying (a) and (b) to whichever carries the
      > satisfaction.
      >
      > **The structural recognizer is AMBIGUOUS, and the fixture in this repo
      > proves it — R2 lens 2.** A Schnorr signature carrying an explicit sighash
      > byte is **65 bytes**; a BIP-341 control block is `33 + 32m`, so at
      > `m = 1` it is also **65 bytes**. They are indistinguishable by length.
      > The RCW's own taproot witness measures
      > `[64, 64, 64, 32, 143, 65]` (`RESULTS_rcw_2026-08-22.txt`) — three
      > signatures, a preimage, a 143-byte leaf script, and that trailing **65 is
      > a control block**, not a signature.
      >
      > So (b)'s *"every input must carry at least one signature"* is
      > **grindable**: a keyless leaf spent at depth 1 yields
      > `[preimage, script, control-block(65)]`, and a length-based recognizer
      > counts the control block as the signature it is looking for. `mt` must
      > therefore recognise a taproot script-path witness **by shape** — last
      > element is the control block, second-last the leaf script — and count
      > signatures only among the remaining elements. **This is still a
      > heuristic and the spec does not claim otherwise.**
      >
      > **Limited by §8.2's removal.** Without a script engine `mt` inspects the
      > spending structure **structurally** — it can tell that a stack element is
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
   > binds the amount without any help from the sighash. **§8.2d makes `mt`
   > actually perform that check**; without it, this justification would assert a
   > binding nobody computes, which is the same defect as the premise it
   > replaced.
   >
   > `sh(wsh(…))` is therefore no longer an unclassified case: wrapped-segwit
   > inputs are segwit inputs, and every input type is accepted.

7. **Over the plate budget (`mt qr`)** → refuse, naming the exact plate count
   and what would fit. **Deferred with the verb (§0a).** **"Plate budget" means the operator's stated maximum
   plate count**, which `mt` compares against §4's search result; there is no
   fixed number, because §4's answer depends on module size, ECC and tiling.

7b. **Over the 4,096-chunk ceiling** → refuse, naming the chunk count and the
   ceiling. Both verbs share it, since both use `mt1`'s header (§3).

   > **This refusal is deliberately unreachable for anything broadcastable.**
   4,096 chunks is 163,840 bytes, and Bitcoin's own standardness limit is
   ~100,000 vbytes — so a transaction large enough to trip this **could not be
   relayed even if `mt` engraved it** (§3). It exists for completeness, not as a
   working constraint. For scale: the largest artifact measured in §3b is
   **89 chunks, 2.2% of the ceiling.**
   >
   > **An earlier version of this refusal said "over the 64-chunk container"**
   > and cited that same 89-chunk artifact as a wallet that *"hit this"*. Both
   > were wrong: 64 was `md-codec`'s 6-bit field, never `mt1`'s (§3's
   > correction), and at 4,096 the artifact is nowhere near the limit. It also
   > pointed at `mt qr` *"which has no such limit"* — both verbs share it.
7c. **Over the `sysw` section ceiling (`mt qr`)** → refuse. **Deferred with the
   verb (§0a); no v0.1 behaviour depends on it.** `MAX_SECTION_LEN =
   8191` (`crates/me-cli/src/sysw/wire.rs:42`), inherited from EPD. **This is a
   hard transport limit §4's search knows nothing about**, so a transaction can
   pass every plate-count check and still be unsendable.

   **This refusal cannot carry a NUMBER until the record framing is chosen, and
   two earlier attempts to give it one were both wrong — R4 lens 2.** The
   ceiling counts **record text**, so the largest admissible PSBT depends
   entirely on how a chunk is framed into a record. Four candidate framings give
   **four different ceilings — 3,671 / 4,094 / 4,476 / 4,525 B** — and none is
   the 4,537 B computed here previously.
   **The only EPD-conformant candidate refuses §4's own largest artifact by
   322 B**, which would mean the biggest wallet this spec measures cannot reach
   the machine at all.

   > **Its two previous numbers, recorded because the pattern matters more than
   > either.** First *"roughly 40% headroom"*, from comparing QR-capacity
   > **bytes** against a cap counting **characters**. Then *"15.4%, ceiling
   > ~4,537 B"*, arithmetically sound but computed against a record framing the
   > spec had never chosen. Three numbers, three unstated assumptions. The fix is
   > not to compute more carefully — it is that **§10.9's record framing is a
   > prerequisite for this refusal**, and until it is settled the refusal is
   > stated as a rule with its threshold named as pending.

   > **An earlier version of this refusal said "roughly 40% headroom", and that
   > was wrong by a units error — R3 lens 3.** It compared the artifact's
   > **QR-capacity bytes** against a cap that counts **record text characters**.
   > The mistake is instructive because it flattered the design in the same
   > commit that discovered the ceiling: a 40% margin invites "no need to model
   > this", while 15% is close enough that §4's search and this refusal must be
   > reconciled rather than left independent (§10.14's regeneration).
8. **Module size is the operator's choice, defaulting to 0.60 mm** — not a
   refusal. Ruling 2026-08-23 (§10.1): `mt` offers every size it can engrave and
   suggests 0.60 mm (two engraved strokes). Sizes below that are **optically
   unvalidated**, and `mt` says so at the point of choice rather than refusing.
   A scan that succeeds today is evidence about one plate on one machine on one
   day, not a property of the size (§10.1).
9. **Secrets** → refuse, as `me` already does for `ms1`.

> **What §8 does NOT check, enumerated because §8.2's removal made the list
> longer and nothing else states it — R2 lens 2.** These are **commitment
> checks**: one hash each, no script engine needed, and `mt` performs none of
> them.
>
> | unchecked | what it would catch |
> | --- | --- |
> | **script-hash** — does the revealed `witnessScript` hash to the `scriptPubKey`? | a witness script that is not the one being spent |
> | **taproot tweak** — does the internal key + merkle root tweak to the output key? | a control block that does not belong to this output |
> | **k-of-n sufficiency** — are there enough signatures for the policy? | an under-signed multisig that will never validate |
>
> Each is cheap and none is script *evaluation* in the sense §8.4's scope ruling
> excludes — they are hashes over data already in the PSBT. They are listed here
> rather than implemented because adding refusals is the operator's call, and
> because §8.2's removal was itself a ruling that `mt` does not verify
> validity. **The consequence stands either way: a transaction can fail every
> one of these and still be engraved.**

Every refusal names the number that caused it. A refusal that says only "too
large" costs the operator a round trip.

## 9. Out of scope for v0.1

**Transaction construction, and PSBT presentation to a signing device** — both
removed by operator ruling 2026-08-23 (§0). Coin selection, fee estimation,
change handling and input selection go with them: they are wallet decisions with
their own failure modes, they are better tested in wallet software before
anything is engraved, and folding them in would make `mt` a wallet.

**`mt qr` IS OUT OF SCOPE FOR v0.1** — deferred to a cross-format QR cycle
(§0a), taking §4, §5, the `sysw` transaction `Class`, the record framing and
§10.17's firmware work with it.

> **CORRECTION — an earlier version of this section said "a decoder is out of
> scope for v0.1" and that a plate cut by `mt` v0.1 could not be read back by
> `mt` v0.1. Operator ruling 2026-08-23 reverses it: `mt decode` ships in
> v0.1.**
>
> The claim was written when "reading a plate" meant §10.2's **static-scan**
> verb — a camera pointed at engraved QR symbols. Two things make that framing
> wrong now. `mt qr` is deferred (§0a), so v0.1 engraves **characters**, not
> symbols; and reassembling `mt1` chunks into a transaction **needs no scanner
> and no camera at all** — it takes strings a human typed or pasted. The
> obstacle I described was never in the way of the thing that matters.
>
> **A format whose own tool cannot read its own output is not falsifiable**, and
> both siblings have a decoder — `md decode`, `mk decode`. `mt` shipping without
> one would have been the anomaly.

**What IS still out of scope: reading a plate OPTICALLY.** §10.2's static-scan
verb — camera, symbol detection, reassembly from images — is deferred with
`mt qr` (§0a), because there are no engraved symbols in v0.1 to scan. That
leaves one real gap, §10.21: **no legend field names the format, the tool, or
the encoding**, so a recoverer holding steel has nothing on it telling them what
software to look for. `mt1…` identifies the string to someone who already knows
the constellation; it says nothing to someone who does not.

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
   dropped (§3), and the QR payload is **`mt1` chunks, bech32 UPPERCASE** —
   operator rulings 2026-08-23. Codex32-in-QR was measured and rejected at
   63–65% efficiency, worse than the UR it would replace and up to two extra
   plates. **base45 was chosen first and then REVERSED**: its alphabet contains
   SPACE, which EPD §6.4 forbids in a `sysw` record, so it could never have
   reached the machine (§3). bech32 uppercase is the only candidate satisfying
   EPD §6.4, EPD §6.6 and QR-alphanumeric packing together.

   **§10.1's test plate should still confirm scanners read bech32-uppercase QR
   symbols off engraved steel** — the encoding is decided, the optical
   validation is not.

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

8. ~~How does a recoverer learn the fragment parameters?~~ **ANSWERED, and the
   operator has ruled on what follows.**

   > **Ruling, operator, 2026-08-23: "each piece should say something like
   > n of m."**

   **Machine-readably this holds for both verbs, because §3 made them share one
   header.** `mt1`'s header carries `count` and `index` — n-of-m — plus a 20-bit
   `chunk_set_id` so pieces of different transactions cannot be combined. **It is
   `mt1`'s own 49-bit header, not `md-codec`'s 37-bit one** (§3): the latter's
   6-bit `count` caps a set at 64 chunks, which `mt qr` exceeds. For `mt encode` that header sits inside the
   BCH-protected chunk; for `mt qr` it rides in the bech32-uppercase payload.
   **One
   mechanism, both media.**

   > **This item was answered TWICE, and the first answer is gone.** It
   > originally analysed UR's fountain encoding — `SeqLen`/`MessageLen`/
   > `Checksum` in CBOR, the `ur:psbt/<n>-<m>/` prefix, and three traps in the
   > vendored decoder (the prefix is parsed then discarded; a single-part UR
   > carries no length or checksum at all; `Progress()` is a `x1.75` heuristic
   > that reaches 1.0 while `Result()` is still nil). **All of that is moot: §3
   > dropped UR entirely.** The traps are recorded here only so a future reader
   > who finds UR attractive again knows what the vendored implementation does.

   **The gap the ruling closes, which survives the envelope change unaltered:**
   `PLATE n OF m` is **not** `part n of m`. Under a multi-symbol tiling, plate 2
   of 3 may carry parts 5–8 of 11, and §5's legend offers only the plate label.
   A recoverer who scans out of sequence, or misses one symbol *on* a plate,
   cannot tell which part is absent.

   **Normative:** every engraved symbol carries its own human-readable `n/m`
   beside it, for the chunk it holds — independent of, and in addition to, the
   plate's `PLATE n OF m`. A recoverer must be able to inventory what they hold
   and name what is missing **without decoding anything**. A lone symbol reads
   `1/1`, which is the only way it can state that it is whole.

   **Unpriced.** These labels consume plate area §4's table does not reserve,
   exactly as the legend did before it was measured — see §10.14, which already
   requires that regeneration. The cost is small per label (3–5 characters) but
   it is per **symbol**, not per plate, and the worst artifact here carries 5.
   **Measure before §4's numbers are treated as final.**

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
   is **necessary and not sufficient**, and **the Rust-primary rule binds**: the
   new class lands in `me-cli`'s Rust `sysw` first, with test vectors, and only
   then ports to the fork's Go.

   > **What "the work" actually is, and a correction to a claim I nearly
   > folded.** R3 lens 3 reported that a new `Class` must pass four gates
   > including `MaxRecords = 24` and `MaxRecordLen = 512`. **Those are `seal`
   > gates, not `sysw` gates** — R4 lens 2 caught the mis-attribution, and it
   > checks out: they are defined in `seal/wire.go`, while `sysw`'s own
   > `splitRecords` is a bare LF split with a UTF-8 check and no caps. The wrong
   > claim reached a persist commit and **never reached this spec**, which is
   > what persisting a report verbatim *before* folding it is for.
   >
   > **The real prerequisite is the RECORD FRAMING**, which nothing has chosen:
   > what a record's text actually contains, and how a multi-symbol artifact maps
   > onto records. Four candidate framings were costed and they give four
   > different transport ceilings (§8.7c), with the only EPD-conformant one
   > refusing §4's largest artifact. **§8.7c cannot state a threshold until this
   > is settled**, and no implementer can build `mt qr`'s output without it.

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
    | verbs | **`encode`**, `decode`, `verify`, `inspect` — matching `md` and `mk` |
    | **input** | a finalized PSBT (preferred) **or a raw signed transaction** (§8.2e) — from a **file or stdin**, never a command-line argument (§8.2f) |
    | `mt qr` output | a **SH2 payload** (`sysw`) carrying the QR — machine engraving |
    | `mt encode` output | the **codex32 string on stdout** — hand engraving |
    | stderr | every warning and refusal a human must see (§3b) |
    | flags | **none for locktime** (§8.4) |

    **Why PSBT-only, when `mt encode`'s PAYLOAD is a raw transaction.** Input
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
    for `mt encode` — extracts the raw transaction as the payload. Nothing is
    lost: a PSBT is what wallet software emits at the point this workflow
    starts, which is exactly the *"test it in your wallet first"* flow §0 is
    built around.

> **A row was removed here on 2026-08-23, and it had gone stale twice.** The
> report carried *"the headroom — chunks against 64 (`mt encode`) or characters
> against 8,191 (`mt qr`)"*. **Both ceilings it named are gone**: 64 was never
> `mt1`'s (§3's correction — it was `md-codec`'s 6-bit field), and 8,191 is
> `sysw`'s, deferred with `mt qr` (§0a). It survived both corrections because
> each fixed a *ceiling* and neither re-read the row that cited it.
>
> **It would not have earned its place even with the right number.** Against
> `mt1`'s real 4,096-chunk ceiling the worst measured artifact is **2.2%** and
> the pathological wallet's descriptor is 0.6%, so the row would report ~98%
> headroom every time — a figure that never varies and never informs. A ceiling
> is worth reporting only where it binds, and this one binds at sizes nobody
> hand-engraves. It returns for the QR cycle, where `sysw`'s 8,191 *does* bind
> at realistic sizes.

    **The SUCCESS-PATH REPORT is `inspect`'s, and `encode` calls it** (§1's
    verb rulings). What follows is the report's content; the ownership rule is
    that `encode` invokes `inspect` rather than composing a second copy, so the
    operator's pre-engraving view and a recoverer's later `mt inspect` cannot
    disagree.

    **`mt` was specified silent when nothing is wrong.** R3's information lens (I-1) found that stdout carries the artifact
    and stderr carries warnings and refusals, so the fee, the plate count, the
    configuration and **the outputs themselves had no channel at all** — while
    §5 and §7 both justify `TO` being an optional one-line summary on the
    grounds that *"`mt` prints every output in full at encode time."* Nothing
    defined that printing.

    **It goes to `stderr`, with the warnings**, because stdout is the artifact
    and writing a report there would corrupt `mt encode`'s output. Before any
    plate is cut, `mt` reports:

    | | |
    | --- | --- |
    | **every output** | address in full, amount, and which are change if a wallet was supplied |
    | **the fee** | absolute and as sat/vB — the number §8.2b's warning thresholds refer to, printed whether or not a warning fires |
    | **the locktime** | §8.4's two facts |
    | **the plate count** | and, since a plate is ~21 minutes (F-225), the **engraving time** |
    | **the configuration** | module size, QR version, ECC level, symbol count — §4's answer |
    | **the engraving size** | how many strings to cut and **how many characters in total** — the unit the person doing the cutting actually experiences |
    | **the set prefix** | the **first 7 characters after `mt1`**, shared by every string in this set, with the rule stated — see below |
    | **the value provenance** | per input: chain-fetched (§6a), txid-bound (§8.2d), or operator-asserted (§8.2c) |

    **THE SPEC NAMES ZERO FLAGS while requiring SEVEN operator inputs the PSBT
    cannot supply — R4 lens 2.** A `grep` for `--[a-z]` returns one hit, and it
    is the *deleted* locktime pair inside a retraction. Most consequentially
    **§8.7's plate budget has no input at all**, which makes that numbered
    refusal unrunnable as written: a refusal whose threshold cannot be supplied
    is not a refusal.

    The inputs `mt` needs, and which section needs them:

    | input | needed by | absent → |
    | --- | --- | --- |
    | the PSBT | everything | refuse |
    | **plate budget** | §8.7 | **§8.7 cannot run** |
    | `FROM` wallet id / fingerprint | §5 | warn, engrave blank |
    | `TO` wallet id / fingerprint | §5 | warn, engrave blank |
    | `TO` free-text label | §10.4 | **requires an explicit flag** by ruling |
    | input values | §8.2c, when the PSBT lacks them | refuse |
    | module size | §8.8 | default 0.60 mm |
    | node location | §6a | the no-node warning |

    **Naming them is a prerequisite for implementation, not a nicety**: two
    implementers given this table will still choose different flag *spellings*,
    but they will at least build the same tool. Given different tables they build
    different tools.

    **A TTY on stdin gets a welcome line, not silence.** Operator ruling
    2026-08-23. `mt encode` with nothing piped in **blocks waiting on stdin**,
    and to anyone who does not know the paste-then-Ctrl-D idiom that is
    indistinguishable from a hang: no output, no prompt, no cursor movement. The
    natural response is Ctrl-C and the conclusion that the tool is broken.

        mt encode: reading a transaction from stdin.
                   Paste it and press Ctrl-D, or Ctrl-C to abort.

    **The test is one line** — stdin is a TTY rather than a pipe — and it is the
    same check that tells `mt` a paste is coming rather than a redirect. The
    failure it prevents is not a wrong result but **a new user concluding the
    tool does not work and leaving**, which no other check catches.

    It is also the one place `mt` would otherwise stop doing what it does
    everywhere else: §8.2c states the fee arithmetic, §8.4 states two facts,
    §6a enumerates the skipped checks. **A tool that silently waits is the
    exception.**

    **Unrecognised input is NAMED, not merely rejected.** `me` already has a
    `classify` module and `md`/`mk` classify their input too, so this is the
    constellation's habit rather than a new idea. A txid is 64 hex characters
    and recognisable as such:

        mt encode: that is a transaction ID (a 64-character hash), not a
                   transaction. mt needs the transaction itself — a txid
                   identifies one, it does not contain one.

    **The SET PREFIX row, and why it is a row rather than a footnote.**
    Operator ruling 2026-08-23. `mt1`'s header packs its invariant fields first
    — `version(4) + chunked(1) + chunk_set_id(20) + count(12)` — so bits 0–36
    are identical across every chunk of a set, and at 5 bits per symbol **the
    first 7 characters after `mt1` are the same on all of them**. Only `index`
    varies.

    **Verified on real output rather than derived**: the four `md1` chunks of
    this repo's pathological wallet all read `md1fveszps…`.

        All 14 strings begin `mt1qzrf8x`. Strings sharing that prefix belong
        to this transaction; strings that do not, do not.

    **This is the only grouping rule a recoverer can apply without software.**
    They may hold plates from two transactions, or one plate from a set whose
    siblings are elsewhere, and the prefix separates them **by eye** — no
    decoding, no checksum, no tool. It costs one line at encode time and hands
    the 2040 reader a rule they would otherwise have to be told by someone who
    is not there.

    **Input and output serialisations are now settled** — three accepted input
    forms (§8.2e) and raw hex out of `decode` (decision 1a in §1).

    **Still unspecified:** the flag spellings themselves, exit codes, and the
    format of the refusal messages §8 promises will *"name the number that
    caused it"*.

11. ~~How many codex32 characters fit a hand-engraved plate?~~ **CLOSED — OUT
    OF SCOPE**, operator ruling 2026-08-23: *"As many as a user wants. It is not
    our concern."* `mt encode` emits a string; what a user does with steel is
    theirs. See §3b. The **4,096-chunk** ceiling is unaffected — that is a
    property of the codec, not of anyone's plate.

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
    trading it for ~340 bytes per chunk of capacity is the wrong trade. The
    **163,840 B** ceiling stands, and §8.7b refuses past it.

13. **`mt1`'s own encoding, NUMS constant and content id — RULED, ready to
    build.** Operator rulings 2026-08-23.

    R0 round 1 (S-2) read `md-codec` directly: the header *layout* (37 bits),
    chunk ordering, gap detection and missing-chunk checks are payload-agnostic
    and take a transaction cleanly. Three things do not transfer, and all three
    are now decided:

    **(a) Its own NUMS constant — RULED, operator 2026-08-23:**

        domain string    : "shibbolethnumstransaction"
        MT_REGULAR_CONST = 0x1a2fc877f9528d7c1

    the **top 65 bits of `SHA-256("shibbolethnumstransaction")`**, following the
    constellation's rule exactly — `md1` uses `"shibbolethnums"`, `mk1` uses
    `"shibbolethnumskey"`, each appending its distinguishing noun spelled out.
    **Recomputed independently before folding**: SHA-256 is
    `d17e43bfca946be09034ac97e7950cdd50d3b5a3e3cf4bad5cb65516897978f6`, the top
    65 bits are `0x1a2fc877f9528d7c1`, the value occupies exactly 65 bits, and it
    differs from both constants already in use.

    `MD_REGULAR_CONST` is hardcoded into checksum
    create and verify (`crates/md-codec/src/bch.rs`). Every constellation format
    gets its own; without a distinct one **an `mt1` chunk would verify as a valid
    `md1` chunk**, which for a bearer plate sitting in a drawer beside `md1`
    plates is a real hazard, not a theoretical one.

    **(b) Its own HRP — the string is `"mt"`, NOT `"mt1"`.** The `1` in a
    rendered `mt1…` string is bech32's **separator**, not part of the HRP.
    `md-codec` makes this explicit: `const HRP: &str = "md"`
    (`crates/md-codec/src/codex32.rs:15`) while its strings render as `md1…`,
    and the checksum is computed over `hrp_expand("md")`
    (`crates/md-codec/src/chunk.rs:565,615`).

    > **R4 filed this as a MINOR and the R4 fold skipped it; R5 found it makes
    > plates MUTUALLY UNVERIFIABLE.** An implementer reading "its own HRP, `mt1`"
    > would compute `hrp_expand("mt1")`, producing a different polymod residue —
    > so every plate written by one implementation fails the other's checksum,
    > and fails it with a *"damaged beyond correction"* diagnostic that points
    > the recoverer at their steel rather than at their software. **Triage by
    > severity label is what let this through**: the finding was correct and its
    > label was wrong, and I folded by label.

    **(a2) The header's exact layout, because R4 found five things an
    implementer would otherwise guess — and two of the guesses produce plates
    another implementation cannot read.** `mt1`'s 49 bits are, in order:

    | field | bits | value |
    | --- | --- | --- |
    | `version` | 4 | **`0b0001`** — `mt1` wire v1. Not inherited from `md1`; a shared value would let one format's chunk verify as the other's under a colliding constant |
    | `chunked` | 1 | **`1`, always, and RETAINED** even though `mt1` is always chunked — see below |
    | `chunk_set_id` | 20 | top 20 bits of the extracted txid, display form (c) |
    | `count` | **12** | **`count − 1`**, matching `md-codec`'s offset convention: a set of 1 stores `0`, a set of 4,096 stores `4095` |
    | `index` | **12** | **plain, zero-based**, `index < count` |

    **`count` stores `count − 1`.** §3's *"admitting 4,096 chunks"* and its
    `count(12)` are consistent only under the offset, and `md-codec` already does
    this (`chunk.rs`). An implementer choosing plain would produce plates whose
    every multi-chunk set is off by one — **unreadable by the other
    implementation, and sending a recoverer to hunt a plate that was never cut.**

    **The `chunked` bit is RETAINED, dead though it is.** `mt1` is always
    chunked, so a thoughtful implementer would drop it and reach a byte-aligned
    40 bits. That is precisely the danger: dropping it shifts every later field
    by one bit, so the two implementations disagree **silently** and produce
    nonsense rather than a clean refusal. Keeping a known-constant bit costs one
    bit per chunk and keeps the layout identical to the format `mt1` forked from.

    **Bit order and padding.** Fields are written most-significant-bit first in
    the order above, matching `md-codec`'s `BitWriter`. The 49-bit header is
    followed immediately by the chunk payload with **no padding between them**;
    padding appears only once, at the end of a chunk, to reach the next 5-bit
    symbol boundary (`mt encode`) or byte boundary (`mt qr`).

    **(c) A content id — the transaction id, and R2 lens 2 found the ruling
    AMBIGUOUS.** A PSBT holds **two** transactions that could be called "the"
    transaction: its `unsigned_tx`, and the one `extract_tx()` produces. **For
    every legacy and `sh(wsh(…))` input their txids DIFFER**, because a legacy
    `scriptSig` is part of the txid preimage while a witness is not. Two
    implementers picking differently would produce plates neither could
    reassemble from the other.

    **Resolved: the id derives from the EXTRACTED transaction's txid** — the
    thing actually engraved, actually broadcast, and actually re-derivable by a
    recoverer who has decoded the plate and holds nothing else. `unsigned_tx` is
    a PSBT-internal artifact a recoverer never sees.

    **The top 20 bits of the txid in its standard display form** — the
    big-endian hex a user reads. Stated to that precision because *"which 20
    bits, from which end"* is exactly where two implementers diverge silently,
    and the internal byte order is the reverse of the displayed one.

    Reassembly re-derives the id from the transaction it decoded and compares.
    `derive_chunk_set_id`
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
    input type is accepted. The residual risk is handled by §8.2c's `stderr`
    warning — which states the fee arithmetic — and by §8.2d, which binds any
    input carrying `non_witness_utxo` by txid. **Nothing reaches an `mt qr`
    plate**: §5's legend is full (§8.2c). Recorded in §7.

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


21. **Nothing on the plate names the format.** A recoverer in 2040 holds QR
    symbols or a codex32 string, a five-field legend, and no indication of which
    tool reads them. The `mt1…` prefix identifies the string form to someone who
    already knows the constellation; `mt qr`'s symbols carry nothing at all.
    Weigh a short format tag against §5's budget — which is 136 characters of a
    300-character allowance, so the room exists (§10.14's regeneration should
    price it).


22. ~~`mt1`'s NUMS domain string is undecided.~~ **CLOSED**, operator ruling
    2026-08-23: the domain string is **`"shibbolethnumstransaction"`**, giving
    **`MT_REGULAR_CONST = 0x1a2fc877f9528d7c1`**. Stated with its derivation in
    §10.13(a), and recomputed there before it became normative.

    The *rule* was always derivable — `MD_REGULAR_CONST` is verifiably the top
    65 bits of `SHA-256("shibbolethnums")` — but the **domain string is an
    arbitrary chosen name** no implementer could have inferred. That mattered
    because the fork mechanic makes the worst guess the most tempting: copy
    `md-codec`, change the HRP, leave the constant, and `mt1` chunks verify as
    `md1` chunks. **§10.13 now has no undecided input left.**


23. ~~Season names are hemisphere-relative on a permanent artifact.~~
    **CLOSED**, operator ruling 2026-08-23: seasons are **northern-hemisphere**
    and §8.4 says so. A southern reader misreads the estimate by about six
    months; the harm is bounded because the **mandatory block height beside it
    is unambiguous everywhere**, so a misread costs an orientation rather than a
    recovery. Alternatives considered and not taken: month ranges (`~SEP 2034`)
    or quarters (`~Q4 2034`), both hemisphere-neutral, both less legible to the
    majority of readers.

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
against a 40-byte chunk. `mt1`'s ceiling is 4,096 of them (§3).

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
> `design/measurements/`"* while the block figures in the old sections 6c and 6d
> — the Merkle-proof and header-cost material — had no results file
> behind them. Those sections are now out of scope (§9), so the claim is true
> again by subtraction rather than by generating the missing evidence.
