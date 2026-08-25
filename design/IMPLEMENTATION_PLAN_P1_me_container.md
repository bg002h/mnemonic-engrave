# IMPLEMENTATION PLAN — P1: `me`'s transaction container

**Status:** DRAFT v3, pre-R0. **Round 1 returned 3 Critical / 11 Important / 8
Minor on v2**, and scored round 0's eighteen at **13 FIXED / 4 PARTIAL / 1 NOT
FIXED**. This is the **FOLD**, not a second rewrite.

**The reports are `design/agent-reports/R0-P1-plan-round0.md`** (5C/13I/5M on v1)
**and `design/agent-reports/R0-P1-plan-round1.md`** (the live one), both persisted
before any of this was written. Findings are marked inline — round-0 markers as
`(C1)`, `(I7)`; round-1 markers as `(r1-C2)`, `(r1-I9)` — so a reader can trace
each one to the report that raised it.

> **SECTION-REFERENCE CONVENTION, because three documents here number their
> sections `2.3` and `6`.** A bare **`§n`** is **this plan**. **`spec §n`** is
> `design/SPEC_engrave_transaction.md`. **`EPD §n`** is
> `design/SPEC_encrypted_payload_delivery.md`. Anything cited as `SPEC_mt_v0_1.md`
> carries its filename. v2 left `§2.3`, `§2.4` and `§6` meaning two different
> documents in adjacent sentences.

> **WHAT v1 GOT WRONG, kept at the top because it is the argument for gating
> plans at all.** v1's §1.1 called the txid byte order *"the most likely defect
> in this plan"* — and then **stated the losing answer as normative**, with an
> escape clause deferring to a decision `mt-codec` does not make. So **V4, the
> vector designed to pin the dangerous thing, was pinned to the wrong axis and
> could not have caught it.** Had this been implemented straight from §6's scope
> line (spec §6), the disagreement would have surfaced when the Go port was written — or
> when a plate was cut and R15 refused a correct record.

> **AND WHAT THE v2 REWRITE BROKE, kept beside it because all three of round 1's
> Criticals came from the EDIT, not from the design.** v2 was a wholesale
> rewrite, and a wholesale rewrite fails in three ways this document now has to
> be read against:
>
> 1. **Partial propagation (r1-C1).** §1.1's prose was corrected to *display*
>    order and **§1's layout table was left saying `INTERNAL`** — and the table
>    is what an implementer transcribes. One fact, two places, one fix. §6 now
>    gates it with `plan-fold-sweep.sh --terms`.
> 2. **Destructive restructuring (r1-C2).** Turning v1's body table into the
>    E1–E10 rules **deleted the only statement of what the body contains**, and
>    left §1.1 citing the deleted section. Restored as **§1.4**, and nothing in
>    this fold is deleted without the fact landing somewhere and every citation
>    being repointed.
> 3. **Building a thing without wiring it in (r1-C3).** The plan specified a
>    record format completely and never once wrote `ClassTransaction`, so §6
>    could close green while `me sysw pack` returned `Unclassifiable` for every
>    `tx:` record. Wired in **§2.4**, with its own TDD step (§4, step 6) and its
>    own closure condition.

## 0. Why this plan exists at all

**Spec §6's** P1 row is a **scope statement**, not a plan. And **spec §2.1b** makes P1's *first*
obligation a design act: **define the `tx:` record's wire layout**, which nothing
has done. Four sections read that layout (spec §3.4's asserted column, R4′, R15, and
P1's own vectors), and the Go port must reproduce it exactly. **A format defined
in code first is a format nobody reviewed.**

---

## 1. THE `tx:` RECORD LAYOUT — normative, Rust-primary

A `tx:` record is `tx:` followed by **lowercase hex** (the reserved-prefix rule).
**(M2)** v1 and v2 both cited `sysw/record.go:41-51` for that rule — but that is
**Go**, and P3's. The Rust site P1 edits is
`crates/me-cli/src/sysw/record.rs:24-28`: the doc comment stating the rule, plus
`TEXT_PREFIX` and `PASS_PREFIX`, which is exactly where `TX_PREFIX` goes (§2.4).

**An odd number of hex characters is already refused (M4).** `unhex_lower`
(`crates/me-cli/src/sysw/record.rs:201-207`) returns `None` on an odd length
before decoding anything, so R1 covers it and this plan adds no rule for it.

The hex decodes to:

```
off  size  field       notes
 0    4    magic       "MTX1" = 4D 54 58 31
 4    1    version     0x01
 5    1    form        0x01 = RAW (transaction bytes) | 0x02 = CHUNKS (mt1 strings)
 6   32    txid        DISPLAY order (byte-reversed), witness-STRIPPED -- see §1.1
38    1    n_fields    count of legend TLVs, 0..=255
39    ..   fields      n_fields x { tag:u8, len:u16 BE, value:len bytes }
 ..   4    body_len    u32 BE
 ..   N    body        body_len bytes -- WHAT IS IN IT is §1.4
```

**All multi-byte integers are BIG-ENDIAN.** Stated once, here, because a wire
format with mixed endianness is a defect generator.

> **(r1-C1) THE TABLE IS NORMATIVE AND SO IS §1.1, AND THE TWO MAY NEVER
> DIVERGE.** This row said `INTERNAL` in v2 while §1.1 said `DISPLAY` sixteen
> lines below — round 0's C1, half-folded, which is the same document carrying
> the same contradiction. **A Go porter transcribes the table**, because it is
> the only part of this plan shaped like a struct. So: the txid's byte order and
> its witness treatment are stated in **exactly these two places**, they must say
> the same thing, and §6 gates that by grep rather than by care.

**The magic is not redundant with the `tx:` prefix.** The prefix says how the
record is framed; the magic says the *body* is this format. A hex-valid body of
the wrong shape is caught at byte 0 instead of somewhere deeper.

### 1.1 THE TXID FIELD — display order, witness-stripped. TWO errors were here.

**NORMATIVE: the `txid` field holds the 32 bytes of the transaction's txid in its
STANDARD DISPLAY ORDER — the byte-reversed form a user reads — computed over the
transaction with marker, flag and witnesses STRIPPED.** So the first byte on the
wire is the first byte of the 64-hex string `mt inspect` prints, and
`hex(txid_field) == txid_display_string` character for character.

Two independent mistakes lived in this one field in v1, and each is separately
sufficient to make two implementations disagree.

**(C1) The order. v1 said INTERNAL; the constellation uses DISPLAY.** Verified in
`mt-codec` itself — the answer is in the function's name. Quoted **byte-exact**
this time **(M5)**; v2 compressed the blank `///` line and the final clause away,
in a section whose whole argument turns on quoting this function accurately:

```rust
// mnemonic-transaction/crates/mt-codec/src/string_layer/pipeline.rs:17-27
/// Top 20 bits of a txid **in its display form** — the content id (§10.13 c).
///
/// The display form is the byte-reversed one a user reads, and "which 20 bits,
/// from which end" is exactly where two implementations diverge silently. So
/// this takes the display string rather than raw bytes, and takes it as the
/// caller already has it.
pub fn content_id_from_txid_display(txid_hex: &str) -> Result<u32>
```

It is the sole producer of a `chunk_set_id` — `mnemonic-transaction/crates/mt-codec/src/string_layer/pipeline.rs:54`
is `let set_id = content_id_from_txid_display(txid_display)?;` — and
`design/SPEC_mt_v0_1.md:3546-3549` already ruled it. **`mt-codec`'s author
anticipated this exact trap and took a display STRING to defeat it. v1 walked
into it anyway** — and because `mt-codec` has no txid *field*, v1's "defer to
`mt-codec`" escape clause pointed at nothing.

**Shipped as written, R15 would have refused every byte-perfect chunks record.**

**(C2) txid, not wtxid.** v1 said *"the raw `double-SHA256` result"* while the
body definition (**§1.4**, restored — v2 deleted it, which is r1-C2) says the RAW
body carries the **witness**. Double-SHA256 over a witness-carrying serialization
is the **wtxid**. `design/SPEC_mt_v0_1.md:680` is explicit: the txid is
*"double-SHA-256 of the decoded transaction **with marker, flag and witnesses
stripped** … **Not** a hash of the engraved bytes."*

**Both halves, demonstrated on the pinned corpus** rather than argued — the
`even` vector of
`mnemonic-transaction/crates/mt-codec/src/test_vectors/mt1_v1.json`, recomputed
for this fold:

```
raw_hex          02000000 0001 01 ...          222 B, segwit (marker 00, flag 01)
dSHA256(raw_hex) reversed
                 d5717c03...ed836f51           == the vector's WTXID
vector txid      2dcf2b97...72ebf630           != the above, in all 32 bytes
top-20 of txid   0x2dcf2                       == the vector's set_id
top-20 of the same txid, INTERNAL order
                 0x30f6e                       R15 refuses a byte-perfect record
```

**So both mistakes are demonstrable on one 222-byte transaction**, and §3 uses
exactly that transaction for every vector so neither can hide.

**In Rust this is not a hand-rolled hash (I5).** `bitcoin 0.32`'s
`Transaction::compute_txid` encodes version, inputs, outputs and locktime and
**nothing else** — witnesses are excluded by construction, not by a flag
(`bitcoin-0.32.9/src/blockdata/transaction.rs:780-787`) — and `Txid`'s `Display`
is documented as *"serialized in reverse byte order when converted to a hex
string"* (`bitcoin-0.32.9/src/blockdata/transaction.rs:44-50`). **That reversal
is the trap in code as well as on the wire:** `Txid::to_string()` gives the
display hex, while `Txid::as_raw_hash().to_byte_array()` gives the INTERNAL
bytes. The field is the former's bytes. A test that compares
`hex(field) == tx.compute_txid().to_string()` catches the swap; one that compares
byte arrays without saying which does not.

### 1.2 Legend field tags

| tag | field | value | `len` | refused when |
| --- | --- | --- | --- | --- |
| `0x01` | `TO` label | UTF-8, operator's own words (spec §3.4, asserted) | `1..=64` | not valid UTF-8 (E14); `len = 0` (E6); `len > 64` (E15) |
| `0x02` | fee | **u64 satoshis**, big-endian | **exactly 8** | `len != 8` (E16) |
| `0x03` | `FROM` wallet | the master fingerprint | **exactly 4** | `len != 4` (E16) |

**The fee is satoshis, not BTC, and not a float.** F-236 closed exactly this in
`mt`. A wire format repeating it would be the same bug somewhere harder to change.

**(I3, r1-I11) "exactly 8 bytes" IN A TAG TABLE IS NOT A REFUSAL.** Round 0's I3
said this in those words — *"a description of the encoder, not a refusal binding
the decoder"* — and v2 folded only its zero-length half. The layout gives every
TLV its own `u16 len`, so `tag=0x02, len=2, value=0x03E8` is expressible and four
implementers answer differently: refuse; read 2 bytes BE as `1,000 sat`; left-pad
to 8 (`1,000 sat` by another route); right-pad (`281,474,976,710,656,000 sat`).
**The fee is engraved in spec §3.4's asserted column.** So the widths above are a
column of the table *and* **E16**, and the near-miss `len = 7` / `len = 9` is
vectored (V17), not just `len = 0`.

**(r1-I10) THE `TO` LABEL NEEDED A UTF-8 VERDICT AND A BOUND, AND RUST AND GO
DISAGREE BY DEFAULT.** Feed both a TLV whose value is `74 6f ff 21`: Rust's
`String::from_utf8` returns `Err` and the record is refused — `me` already has
that posture at `crates/me-cli/src/sysw/record.rs:93`
(`RecordError::NotUtf8`) — while Go's `string(b)` never fails, so the record is
accepted and the label reaches steel with a replacement character. Hence **E14**.

**The 64-byte bound is a P1 DECISION, not a derivation, and is stated as one.**
A `u16 len` makes a 65,535-byte label expressible, bounded only incidentally by
the section cap. 64 is chosen because the plate's `TO` line is budgeted at **34
characters** (`design/SPEC_mt_v0_1.md:1767`, which covers the whole
`TO <wallet id, fp or label>  <amount>` line), so 64 bytes is comfortably above
anything that can be cut and far below anything that can inflate a record. **If
the operator wants a different number this is the field to change**, and nothing
else moves with it.

### 1.3 ENCODING RULES — every one of these is a way two implementations diverge

v1 stated a layout and no rules. A layout without rules is a family of formats.

| # | rule | why |
| --- | --- | --- |
| E1 | **TLVs appear in ASCENDING TAG ORDER.** | **(C4)** v1 left order undefined and V2 pinned an *instance*, not a rule — two conforming encoders emit different bytes for the same input |
| E2 | **A tag appears AT MOST ONCE. A duplicate is REFUSED.** | **(I1)** undefined, a map-insert keeps the last and a `find` keeps the first — *different `TO` text on steel* |
| E3 | **The record ends where the body ends. Trailing bytes are REFUSED.** | **(C5)** without it, 32 bytes appended after a genuine transaction pass every other check |
| E4 | **`39 + Σ(3 + len) + 4 + body_len` MUST equal the decoded length exactly.** | makes E3 checkable rather than aspirational |
| E5 | **`body_len` is validated against the remaining length BEFORE any allocation.** | **(I2)** an 89-character record otherwise declares 4 GiB; EPD §6.2 sets the precedent — bound before you trust |
| E6 | **A zero-length TLV value is REFUSED.** | **(I3)** absence is *omission* (E7); a present-but-empty field is a second spelling of nothing |
| E7 | **An absent optional field is OMITTED from the list.** There is no empty encoding and no sentinel. | spec §2.1b asked what absence looks like |
| E8 | **An unknown tag is REFUSED, not skipped.** | skipping is how two implementations silently diverge |
| E9 | **A bad `magic`, an unknown `version`, or a `form` outside {0x01, 0x02} is REFUSED, each with its own message.** | **(I4)** v1 gave a verdict for unknown *tags* only |
| E10 | **`n_fields` MUST equal the number of TLVs actually parsed.** | a disagreement is a malformed record, not a hint |
| **E11** | **RAW: re-serialising the decoded transaction MUST reproduce the body BYTE FOR BYTE.** | **(r1-C2)** §1.4 says the RAW body carries the witness — and *nothing else in the plan can tell*. Both serialisations deserialise, and the txid strips witnesses anyway, so **only re-serialisation equality separates them**. Without it one conforming record carries a transaction with every signature removed |
| **E12** | **CHUNKS: the body is the `mt1` strings joined by a single `\n` (0x0A), with NO trailing separator and no empty element.** | **(r1-C2)** a trailing `\n` yields an empty final element on split: one implementation accepts what the other refuses |
| **E13** | **CHUNKS: every element is lowercase ASCII with no leading or trailing whitespace.** | **(r1-C2)** `mt-codec`'s `to_symbols` does `s.trim().to_ascii_lowercase()` (`mnemonic-transaction/crates/mt-codec/src/string_layer/pipeline.rs:66`), so it **accepts** uppercase and padded strings — different `body_len`, different record hex, a different EPD §6.6 public-data hash, and `mt_codec::decode` still green. The tolerance is the decoder's; the record layer may not inherit it |
| **E14** | **Tag `0x01`'s value MUST be valid UTF-8. Invalid is REFUSED.** | **(r1-I10)** Rust refuses by default and Go accepts by default — one implementation refuses what the other engraves |
| **E15** | **Tag `0x01`'s value is `1..=64` bytes.** | **(r1-I10)** a `u16 len` otherwise admits a 65,535-byte label into a plate legend budgeted at 34 characters |
| **E16** | **A fixed-width tag MUST carry exactly its width: `0x02` → 8, `0x03` → 4. Any other `len` is REFUSED.** | **(I3, r1-I11)** E6 refuses only `len = 0`; `len = 1..7` is the actual gap, and the fee is engraved |

**Every one of E1–E16 gets a vector (§3) and a test that goes RED without its
check.** A rule with no negative test is a comment. **E1's negative is V16**, not
V2 — **(r1-I1)** V2 is a *positive* vector whose bytes are ascending by
construction, so deleting the ordering check entirely left it green and the
closure condition unsatisfiable for the one rule it was written for.

### 1.4 (r1-C2) THE BODY — what `body_len` bytes actually contain

**v2 DELETED this and left §1.1 citing it.** v1 had it as a two-row table; the
rewrite replaced the section wholesale with E1–E10 and dropped the two rules it
already had, so v2 stated the record's framing completely and **never said what
the record carries**. Restored, as rules rather than as a table cell:

| `form` | the body is |
| --- | --- |
| `0x01` RAW | **the serialized signed transaction, WITH WITNESS** — the BIP-141 form: version, marker `0x00`, flag `0x01`, inputs, outputs, each input's witness, locktime |
| `0x02` CHUNKS | **the `mt1` strings, LF-separated, lowercase ASCII**, no trailing LF |

**RAW: why "with witness" needs E11 and cannot rest on this sentence.** Walk a
witness-STRIPPED body through every other check in this plan: `magic`, `version`,
`form` pass; **every encoding rule except E11 passes**, because the rest of them
constrain the TLVs and the lengths and not the body's shape; §2.2's *"deserialise
the body as a Bitcoin transaction"* passes, because a witness-free serialisation
**is** a valid transaction; and §2.2's txid equality passes, **because the txid
strips witnesses anyway.** Two conforming records, different bytes, and one plate
carries a transaction with every signature removed — unbroadcastable, which is
the one thing this artifact exists to make possible. E11 is the only check that
fires, and it is cheap: `bitcoin`'s `Transaction::consensus_encode`
(`bitcoin-0.32.9/src/blockdata/transaction.rs:1239-1260`) emits the segwit form
whenever any input carries a witness, so E11 is one `==` on two byte slices.

**MEASURED, on §3's transaction, with the exact dependency §2.2 chooses** — a
scratch crate built and run while writing this fold, and reproducible from §6.1:

```
body (with witness)            222 B     serialize(&tx) == body   ->  true
same tx, witnesses stripped    113 B     deserializes             ->  true
                                         SAME txid as the body    ->  true
trailing 32 bytes appended               deserialize rejects      ->  true
```

**109 bytes of signatures, silently absent, with every other check in this plan
green.** That is what E11 costs one `==` to prevent, and V18 is the vector.

**CHUNKS: the separator, the trailing separator and the case are all rules
(E12, E13), never vector cells.** v2 left `LF separation` living in V3's *"pins"*
column — and this plan's own C4 lesson is that **a vector is an instance, not a
rule**.

### 1.5 (r1-I2) WHAT "REFUSED" MEANS HERE — layer, scope, stream, exit, and what runs first

**"REFUSED" appears in E2–E16 and §2.3 and v2 defined it nowhere**, so step 8's
and step 10's assertions could not be written. Spec §5 is explicit that *"Where a
refusal RUNS is part of the refusal"* and *"For every refusal above, name what
runs BEFORE it"*. Four answers, once, binding every use of the word in this plan:

| question | NORMATIVE answer |
| --- | --- |
| **which layer** | the record codec returns `Err(TxRecordError::…)`; **`me sysw pack` maps it to a refusal**. The codec never prints and never exits |
| **what scope** | **the whole pack aborts and nothing is written** — five records, one malformed `tx:`, zero output. This follows `split`'s existing shape for `Class::Unknown` (`crates/me-cli/src/sysw/mod.rs:255`), **not** `report_unconfirmed`'s warn-and-pack (`crates/me-cli/src/main.rs:1141`) |
| **what the operator sees** | **one line on stderr naming the record's index and the rule**, nothing on stdout, **exit 4** — `me`'s existing pack-failure path (`crates/me-cli/src/main.rs:975-983`) |
| **what runs BEFORE it** | for a record already in hand: `classify` (§2.4). **For R2 the answer is clap**, and that is the whole point of the refusal — `mt`'s §8.2f was bypassed because the arg parser ran first *and clap's error echoed the bearer transaction*. **R2's test asserts the record text does not appear in stderr**, not merely that the exit code is non-zero |

**E9's "three distinct messages" therefore means three distinct stderr lines at
exit 4**, and V13 asserts the text, the stream and the code.

## 2. WHAT `me` MUST DECODE — and what decoding can and cannot prove

**A `tx:` record's body is NOT opaque to `me`, and no body ever has been** —
`me` calls `md_codec::reassemble` on the `md1`/`mk1` records it packs today
(`crates/me-cli/src/sysw/record.rs:155-159`).

> **(r1-I8) v2 OPENED THIS SECTION BY REFUTING A SENTENCE THE SPEC DOES NOT
> CONTAIN, and then made correcting it a condition of closing the phase.** The
> quoted *"the record body is opaque to `me`"* was a **spec §1** table cell added at
> `f935316` and **removed by `2dce797`, the journey-walk fold, before the spec
> reached R0 GREEN**; `grep -ni opaque design/SPEC_engrave_transaction.md`
> returns nothing. So §6's *"spec corrections this phase owns: §1's 'opaque'
> sentence"* had no referent — an unsatisfiable close condition, and whoever
> burned it would have ticked it falsely or edited
> `design/JOURNEY_WALK_engrave_transaction.md:574`, which is a different document
> making a different claim about where `--chunks` belongs. **The argument below
> stands and is still needed; the citation and the closure item are gone.**

EPD §6.3: a public-section record *"MUST additionally **DECODE**."* The rationale
is anti-smuggling: BCH is publicly computable, so arbitrary bytes wrap into
something that *classifies* right, and a non-conforming sealer could put secret
bytes in cleartext where `picotool save` reaches them with **no passphrase**.

### 2.1 (C3) `mt_codec::decode` PROVES NOTHING ON ITS OWN

v1 said the CHUNKS decode satisfies EPD §6.3. **It does not.** Round 0 executed it:
32 bytes of entropy → one valid `mt1` string → exact round-trip, with an
attacker-chosen `set_id` so **R15 passes too**. Confirmed independently:

```
grep -rn "bitcoin::" crates/mt-codec/src/   ->  0 hits
Cargo.toml                                  ->  bitcoin = "0.32"   (DECLARED, UNUSED)
```

`mt_codec::decode` is a **BCH verifier**. It proves a string is well-formed
`mt1`; it says nothing about whether the bytes are a transaction. **So the
smuggling channel v1 claimed to close was still wide open.**

### 2.2 NORMATIVE — both forms end at the same proof

| form | decode |
| --- | --- |
| RAW | deserialise the body as a Bitcoin transaction, **and E11: re-serialise it and require the bytes back** |
| CHUNKS | **reassemble via `mt-codec`, THEN deserialise the result** as a Bitcoin transaction — the elements having satisfied E12 and E13 first |

**and in both cases the transaction's txid (display order, witness-stripped, §1.1)
MUST equal the carried `txid` field.**

**The E11 clause is not decoration (r1-C2).** Without it the RAW row admits two
different byte strings for the same transaction, and §1.4 measures exactly how
far apart: **222 bytes with the witness, 113 without, the same txid, and both
deserialise.**

**Chaining the parse onto the reassembly is what closes the CHANNEL C3 measured**,
and it costs nothing extra: `me` needs a transaction parser for the RAW form
regardless, so the CHUNKS path reuses it. The two forms become **symmetric** —
each ends at *the bytes are a real transaction, and it is the one the record
claims*.

> **ACCEPTED COST (r1-I3) — DECODE RAISES THE ATTACKER'S PRICE; IT DOES NOT CLOSE
> THE CHANNEL.** v2 said this gate *"closes C3"* full stop, and that is a
> load-bearing overstatement in the one section that is this plan's whole
> anti-smuggling argument. **A valid transaction is itself an arbitrary-byte
> container.** Build a real, deserialisable transaction with one output whose
> `scriptPubKey` is `OP_RETURN <32 bytes of seed>` — or simply *is* the seed,
> since nothing evaluates it — compute its txid honestly, carry it: the body
> deserialises, the txid matches, E1–E16 pass, and `picotool save` reaches those
> 32 bytes **with no passphrase**. Nothing here is signed, nothing is checked
> against a UTXO set, and nothing bounds what a `scriptPubKey` may contain.
>
> **The true bound is "the bytes are a well-formed transaction", not "the bytes
> are not a secret."** The cost of smuggling goes from *any 32 bytes* to *any 32
> bytes wrapped in a syntactically valid transaction* — a few lines of code — and
> that is the honest claim. It is written out because EPD §6.3's requirement is the
> only reason `tx:` gets a decode at all, and the P3 porter will otherwise read
> DECODE as *the* gate. Spec §3.5 and spec §2.2a state their equivalent limits in
> exactly this voice; this section now does too.

**(I5) `me` gains `bitcoin = { version = "0.32", default-features = false,
features = ["std"] }`.** It has
none today — its deps are the three sibling codecs, clap, zeroize, serde,
aes-gcm, pbkdf2, sha2, bip39, rand, rpassword — and v1 required a transaction
parse without saying where it comes from. **v2 named the gap and still made no
choice, so this fold makes it (r1-I5):**

| decision | value | why |
| --- | --- | --- |
| crate | `bitcoin` | the API P1 needs is `consensus::deserialize`, `consensus::serialize` and `Transaction::compute_txid`, all in one crate |
| version | **`0.32`** | matches what `mt-codec` already declares, so the constellation resolves one copy rather than two |
| features | **`default-features = false, features = ["std"]`** | `bitcoin`'s `default` is `["std", "secp-recovery"]`, and P1 needs no key recovery, no `rand` and no `serde`-on-transactions. **`std` must be kept explicitly** — dropping it too puts a host CLI in `no_std`. **Built and run, not inferred** (§6.1) |
| **added in** | **§4, step 4** | v2 added it in no step at all, which is why this was only PARTIAL |
| **NOT via `mt-codec`** | | `mt-codec` declares `bitcoin` and **uses it nowhere** (0 `bitcoin::` in `src/`), so depending on it buys the compile cost and none of the API. §5 removes that dead declaration before publishing (M8) |

**All three API facts are verified against the vendored crate source, not its
docs:** `compute_txid` excludes witnesses by construction
(`bitcoin-0.32.9/src/blockdata/transaction.rs:780-787`); `Txid: Display` is
reverse byte order (`bitcoin-0.32.9/src/blockdata/transaction.rs:44-50`); and **`consensus::deserialize`
already refuses trailing bytes** — *"Fail if data are not consumed entirely"*,
`bitcoin-0.32.9/src/consensus/encode.rs:163-172` — which makes E3's record-level
rule and the body-level one agree instead of fighting.

### 2.3 (I8) A `tx:` record that fails DECODE is REFUSED, not warned

The two containers already differ and v1 said which applied to neither: `seal`
**refuses** (`crates/me-cli/src/seal/mod.rs:171`), while `sysw` **warns and packs
anyway** (`crates/me-cli/src/main.rs:1141` — *"then the container is built
anyway"*).

**NORMATIVE: `tx:` follows `seal`'s posture — refuse.** The `sysw` warn-and-pack
posture is defensible for a record whose worst case is an unengraveable plate. It
is not defensible for the one record class whose failure mode is **secret bytes
riding in cleartext**, which is the whole reason EPD §6.3 exists.

### 2.4 (r1-C3) WIRING — the class, the prefix, the classifier. A codec nobody calls is not a feature

**v2 specified this record format completely and never wrote `ClassTransaction`
once.** Not in a TDD step, not in a closure condition. Implement §4 exactly as v2
wrote it — the constant raised, the codec built, v2's V1–V15 green, clippy clean,
`mt-codec` published — and **every bullet of §6 is satisfied**, and then:

```
$ mt encode --record --raw < tx.final.psbt | me sysw pack
me: record 0 ... unclassifiable
```

because `classify` (`crates/me-cli/src/sysw/mod.rs:124`) tests `PASS_PREFIX`,
then `TEXT_PREFIX`, then BIP-39, then `seal::record::validate_record`, and returns
`Class::Unknown` for everything else; `split` (`crates/me-cli/src/sysw/mod.rs:255`)
turns that into `SyswError::Unclassifiable` and packs nothing. **The gate would
have closed green on the defect it exists to prevent** — spec §7's own words for
this shape: *"a close condition that could pass on the defect it exists to
catch."*

**NORMATIVE — five sites, and none of them is the codec:**

| # | site | change |
| --- | --- | --- |
| W1 | `crates/me-cli/src/sysw/record.rs:27-28` | add `pub const TX_PREFIX: &str = "tx:";` beside `TEXT_PREFIX` and `PASS_PREFIX` |
| W2 | `crates/me-cli/src/sysw/record.rs:31-40` | add `Transaction` to `enum Class` (today eight variants, none of them this) |
| W3 | `crates/me-cli/src/sysw/record.rs:50-55` | **`Class::Transaction` is NOT in `is_secret`'s `matches!`** — spec §2.1's *"no new secrecy class"*, and round 0's M3 |
| W4 | `crates/me-cli/src/sysw/mod.rs:124` | `classify` gains a `TX_PREFIX` branch: hex-decode, then §2.2's DECODE, then `Class::Transaction`; a failure is §1.5's refusal, **never** `Class::FreeText` |
| W5 | `crates/me-cli/src/sysw/mod.rs:108-115` | `unknown_reason` iterates `[PASS_PREFIX, TEXT_PREFIX]` only, so today a `tx:` record with a bad body reports `Unrecognised` **without naming its prefix**. Add `TX_PREFIX` |

**W3 is the one that costs the operator if it is wrong.** `is_secret` is a
`matches!` over three variants in a file whose doc comment argues both ways about
a borderline class (*"`Class::FreeText` is deliberately NOT secret even though an
operator may put anything in it"*). Get it wrong and **every transaction payload
seals**: a 12-word passphrase to store, those 12 words typed on the device's
on-screen keyboard, and ~31 s of KDF — to protect a payload whose purpose is to
become a plate anyone can read. That is the exact cost spec §2.4 exists to
remove, and it is an operator ruling in spec §8.

**W4 is where spec §2.1a's lesson lands in Rust.** The spec's C3 was
*adding a prefix without adding a branch*; W1 without W4 is that defect in `me`
rather than in `gui/scan.go`. **The branch is the work; the prefix is not.**

**§4's step 6 is these five sites, and its test is END TO END** — `me sysw pack`
on a real `tx:` record, read back with `me sysw show`. V1–V22 are record-level
vectors: they exercise the codec directly and stay green with `classify`
untouched, so **no vector in §3 can substitute for that test.**

## 3. THE VECTORS

Rust-primary means these are what the Go port is judged against. They must pin
every choice a second implementer could make differently.

**(r1-I7) EVERY VECTOR USES ONE TRANSACTION, AND IT IS SEGWIT.** v2 required
segwit **for V4 only**, and named no form for V4 at all — `chunk_set_id` exists
only for the chunks form, so V4 read as CHUNKS while the row never said, which
alone blocks construction. Worse, it left **V1 and V2 unconstrained**: built from
a convenient small *legacy* transaction, they pass under an implementation that
computes `dSHA256` over the with-witness RAW body, which then refuses **every
honest segwit RAW record** at §2.2's equality — and V8 (*carried txid ≠ body's* →
assert refusal) passes in that world too, because it also refuses. **That is C2
surviving in the half of the format V4 does not cover.**

**NORMATIVE: the transaction under every vector below is the `even` vector of
`mnemonic-transaction/crates/mt-codec/src/test_vectors/mt1_v1.json`** — 222 bytes,
segwit (`raw_hex` begins `02000000 0001 01`), `txid_is_wtxid: false`, six `mt1`
chunks of 37 B, and these three values, recomputed for this fold and written out
in the vector file (§6):

```
txid  (display)  2dcf2b973d52044b1e58c988a5a59d388073ff05598b0a1e93eeb04c72ebf630
wtxid            d5717c031917116bbd4fcaff0bcc3abe9d456899991414f2177a5281ed836f51
chunk_set_id     0x2dcf2      == top 20 bits of the DISPLAY txid
                 0x30f6e      == top 20 bits of the INTERNAL one -- the C1 defect
```

**It also satisfies §3.2 for free:** that corpus was generated by
`scripts/gen-mt1-vectors.py` **in this repo**, not by the crate it judges.

| # | vector | pins |
| --- | --- | --- |
| V1 | **RAW (segwit)**, no optional fields | the fixed layout; `n_fields = 0` |
| V2 | **RAW (segwit)**, all three optional fields | E1's ascending tag order as an *instance*, `u16 BE` lengths, u64 fee. **Not E1's negative — that is V16 (r1-I1)** |
| V3 | CHUNKS, multi-chunk body | `form = 0x02`; E12's single `\n` and absent trailing LF; E13's lowercase ASCII |
| **V4a** | **RAW**, with txid AND wtxid written out | **§1.1's display order AND txid-not-wtxid, on the RAW path.** Segwit is required: for a legacy transaction txid == wtxid and the vector passes in both worlds |
| **V4b** | **CHUNKS**, with txid AND wtxid AND `chunk_set_id` written out | the same two facts on the chunks path, plus R15's positive case. **(r1-I7)** v2 had one V4 and never said which form it was |
| V5 | absent optional field | absence is omission (E7) |
| V6 | unknown tag | REFUSED (E8) |
| V7 | body at **16,322 B minus the fields present** | the **record framing** ceiling — see §3.1 |
| V8 | RAW whose carried txid ≠ the body's | the §2.2 consistency refusal |
| **V9** | **duplicate tag** | E2 |
| **V10** | **trailing bytes after the body** | E3/E4 |
| **V11** | **`body_len` larger than the bytes remaining** | E5, before allocation |
| **V12** | **zero-length TLV value** | E6 |
| **V13** | **bad magic / unknown version / `form = 0x03`** | E9, three distinct stderr lines at exit 4 (§1.5) |
| **V14** | **`n_fields` ≠ the TLVs present** | E10 |
| **V15** | **R15 NEGATIVE: a chunks record whose carried txid's top 20 bits ≠ its chunks' `chunk_set_id`** | **(I13)** delete R15's comparison and every *other* vector stays green. **(M1) Its v2 justification was false and is retracted:** V15 constructs a deliberate mismatch, which is still a mismatch under the wrong byte order, so the refusal fires either way. **V4a/V4b catch C1; V15 catches a deleted R15.** Both are required |
| **V16** | **TLVs in DESCENDING tag order** → REFUSED | **(r1-I1)** E1's negative. Without it, a decoder that *accepts* `0x03, 0x02, 0x01` conforms to every vector, so the Go port — which §3 says is judged **against the vectors** — admits a record `me` refuses. E1's own divergence, one layer out |
| **V17** | **`tag=0x02, len=2`; and the near-misses `len=7` and `len=9`** | **(I3, r1-I11)** E16. E6 refuses only `len = 0`; `1..7` is the gap, and the fee is engraved |
| **V18** | **RAW body = the same transaction serialized WITHOUT witness**, carried txid correct | **(r1-C2)** E11. This is the vector that separates the two conforming records: it deserialises, and its txid matches, and it must still be REFUSED |
| **V19** | **CHUNKS body with a trailing `\n`** | E12 — the empty final element one implementation accepts and the other refuses |
| **V20** | **CHUNKS body with an UPPERCASE `mt1` string** | E13 — `mt_codec::decode` accepts it, the record layer must not |
| **V21** | **`tag=0x01` whose value is `74 6f ff 21`** | **(r1-I10)** E14. Rust refuses by default, Go accepts by default |
| **V22** | **`tag=0x01, len=65`; near-miss `len=64` must PASS** | E15, and the near-miss rule (§6) |

### 3.1 (I7) V7's number, and a THIRD ceiling nobody had

v1 said *"body at `MaxSectionLen` boundary"*. **Unconstructible.** A record is
`tx:` plus hex, so:

```
32,734 section chars − 3 for "tx:"  = 32,731 hex chars
                                     -> 16,365 whole bytes (one hex char SPARE)
                       − 43 framing  = 16,322 bytes of body, minus the fields
```

**(M4) The middle line is a FLOOR, not an equality.** 16,365 bytes is 32,730
characters; the odd 32,731st cannot start a byte and is simply unusable. An odd
run of hex is refused before it is decoded (§1, `unhex_lower`), so nothing turns
on it — but the derivation said `=` where it meant `->`, and a Go porter
reproducing the arithmetic would have hunted for the missing character.

**So there are THREE ceilings and the spec states only two:**

| ceiling | value | where | binds |
| --- | --- | --- | --- |
| container section | 16,367 B | spec §2.3 **as written** | nothing — it is the uncorrected figure |
| **record framing** | **16,322 B of BODY** | **this plan — new** | both forms |
| engraveable | 14,560 B | spec §4.1a, at 0.60 mm | the **raw** form only (QR plates) |

**(M7) THE 16,322 FIGURE ASSUMES THE `tx:` RECORD IS ALONE IN THE SECTION.**
Records are joined with `\n` and no trailing LF
(`crates/me-cli/src/sysw/mod.rs:260`, asserted by
`joins_with_lf_and_no_trailing_lf`), spec §3.6 contemplates several transactions
in one payload, and a payload may also carry `md1`/`mk1` records. For `k` records
the bound is `Σ(record chars) + (k − 1) ≤ 32,734`, so **V7 is explicitly the
single-record vector** and the plan claims nothing about a full payload.
(`MAX_RECORD_LEN = 512` and `MAX_RECORDS = 24` are `seal`'s; `sysw::split`
applies neither.)

**THREE SPEC NUMBERS ARE FALSIFIED BY §1'S FRAMING, NOT ONE.** Round 0's I7 named
two and v2's §6 owned only one (r1-I7); the third is the framing arithmetic
applied to **spec §2.3's** own table. All recomputed for this fold as
`3 + 2 × (43 + N) ≤ cap`:

| spec §2.3 says | recomputed | verdict |
| --- | --- | --- |
| *"a **16,367-byte** raw transaction"* | 16,322 B of body, minus the fields | **wrong by the framing** — 45 B plus the legend |
| *"5/2 … ✅ (**raw-only at 8191, by 31 chars**)"* | `3 + 2×(43 + 4,080) = 8,249` | **false — 58 chars OVER 8191.** The 31 counted 8,160 hex chars against the old cap and ignored both `tx:` and the framing |
| *"10/2 … 18,583 … ✅"* (chunks) | `3 + 2×(43 + 18,583) = 37,255` | **false — 4,521 chars OVER 32,734.** The chunks body is ASCII **inside** the record's hex, so every `mt1` character costs **two**; the column compares `mt1` characters against `MaxSectionLen` directly |

**That third row is the one with teeth**, and it is new: **the framing ceiling
binds the chunks form at 16,322 CHARACTERS of `mt1` text**, and the worst measured
pathological spend produces 18,583. Under §1's framing the 10/2 chunks payload
**does not fit a section**, while its raw form (16,223 chars) does. The full
recomputation, for the correction §6 owns:

```
in/out  rawB  raw record chars      chunk chars  chunks record chars
 1/1     852     1,793  OK              2,001       4,091  OK
 1/2     893     1,875  OK              2,092       4,273  OK
 2/2   1,692     3,473  OK              3,955       7,999  OK
 5/2   4,080     8,249  OK              9,383      18,855  OK
10/2   8,067    16,223  OK             18,583      37,255  OVER by 4,521
```

> **AND THE THIRD MAY BE A DESIGN QUESTION, NOT A NUMBER — flagged, not
> answered.** A wrong figure in a table is a correction. *"The worst measured
> pathological spend cannot be backed up in chunk form"* is a capacity fact, and
> it arrives from arithmetic nobody had done, in a document that is R0 GREEN. It
> is stated here because P1 is where the framing that causes it is defined, and
> **P1 does not resolve it**: the raw form of the same transaction fits with
> 16,511 characters to spare, so nothing is lost today that the operator cannot
> route around by choosing `--raw`. Whether the chunks form should avoid paying
> hex twice — it is the one body that is already ASCII — belongs to whoever
> reopens spec §2.1b, not to this fold.

**P1 must land all three corrections** (§6). Round 3 already corrected spec §2.3's
headline once for the engraving ceiling; this is the second and third pass over
the same paragraph, which is itself the argument for computing a table rather
than editing a sentence.

### 3.2 (I12) THE VECTORS MAY NOT BE PRODUCED BY THE CODE THEY JUDGE

`mt-codec`'s own `mnemonic-transaction/crates/mt-codec/src/lib.rs:9-14` rules
against exactly that, and `mt`'s vectors are
generated by `scripts/gen-mt1-vectors.py`, which **re-implements the format
independently**. P1's vectors follow that precedent: **hand-constructed or
independently generated, never dumped from the encoder under test.** A vector
derived from the implementation cannot falsify it — that is how a wrong NUMS
constant once launders itself into looking correct.

## 4. TDD ORDER

Each step: failing test first, watched fail **for the stated reason**, minimal
code, full suite green.

| step | test first | then |
| --- | --- | --- |
| 1 | **`sysw::wire::MAX_SECTION_LEN` is 32,734** — and `seal::wire::MAX_SECTION_LEN` is **still 8191** | raise **only** `crates/me-cli/src/sysw/wire.rs:42` |
| 2 | `me sysw pack` reads from **stdin**; **empty stdin refused** (R7); **a TTY with no `--in` and no argv refuses instead of blocking**; **`--in` still wins over argv, and stdin is read only when neither is given** (§4.2) | the stdin path |
| 3 | **a payload with NO `Class::is_secret()` record packs UNSEALED; one with any packs SEALED** (spec §2.4, the base rule); an explicit passphrase-mode flag **wins**; **`--allow-weak` is not one** (§4.3); **stderr says which way and why, every time** | content-based sealing |
| 4 | V1–V3 round-trip | the layout codec — **and this is the step that adds `bitcoin = { version = "0.32", default-features = false, features = ["std"] }` to `crates/me-cli/Cargo.toml`** (I5, §2.2) |
| 5 | **V4a and V4b** | the txid field: display order, witness-stripped, on **both** forms |
| **6** | **`Class::Transaction` exists and `is_secret()` is FALSE; `me sysw pack` on a real `tx:` record puts it in the PUBLIC section; `me sysw show` reads it back; a `tx:` record with a bad body names its PREFIX, not `Unrecognised`** | **(r1-C3) the wiring — W1–W5 of §2.4.** End to end, because no record-level vector can see this |
| **7** | **a `tx:` record that fails DECODE aborts the whole pack: nothing on stdout, one stderr line naming the index and the rule, exit 4 — with four other valid records present** | **§2.3's refusal, at the layer and scope §1.5 rules.** The anti-smuggling gate, tested as a `me sysw pack` outcome rather than as a codec `Err` |
| 8 | V5–V6, V9–V14, **V16–V22** | every rule in **E1–E16** |
| 9 | V7 at **16,322 − F** | the framing ceiling, single-record (§3.1) |
| 10 | V8 and **V15** | the txid-consistency refusals, both forms |
| 11 | a `tx:` record on **argv** refused (R2), **and its text does not appear in stderr** | the argv guard — §1.5's *what runs before it*: clap must not echo the record |

### 4.1 (I11) STEP 1 SAYS WHICH CONSTANT, BECAUSE THERE ARE TWO

v1 said *"the constant"* and named `boundBlob` — which has **0 hits in this
repo's Rust**; it is Go, and belongs to **P3**. Meanwhile **two** Rust constants
carry this name:

```
crates/me-cli/src/seal/wire.rs:21   MAX_SECTION_LEN: u32   = 8191   <- FROZEN. Do not touch.
crates/me-cli/src/sysw/wire.rs:42   MAX_SECTION_LEN: usize = 8191   <- this one
```

**Step 1's test asserts BOTH** — the new value on `sysw`, and that `seal` is
unchanged. A test that only checks the raise would pass if someone edited the
frozen container instead.

### 4.2 (I10) STEP 2 MUST NOT TURN `me sysw pack` INTO A HANG

Adding a stdin path makes a bare `me sysw pack` block on a TTY with no prompt —
**a new user's first action looks like the tool has crashed.** The `mt` cycle
found this exact defect (*"stdin doesn't mean from the command line?"*), and it
was the operator's own confusion that surfaced it.

**NORMATIVE: if stdin is a TTY and neither `--in` nor argv records are given,
refuse with a message naming both real inputs.** And note **R7's test as written
(`printf '' | me sysw pack`) passes in both worlds** — it never touches the TTY
path, so it cannot catch this. The TTY case needs its own test.

**(M6) AND STEP 2 MAKES A THIRD RECORD SOURCE, SO THE PRECEDENCE MUST BE RULED.**
Today `--in` wins over argv silently and filters blank lines
(`crates/me-cli/src/main.rs:1211-1227`). v1 and v2 both added stdin without saying
what `me sysw pack rec1 --in f.txt < g.txt` does.

**NORMATIVE: `--in` > argv > stdin.** Stdin is read **only** when neither `--in`
nor argv records are given — which is exactly the branch that returns
`no records: pass them on argv or with --in` today
(`crates/me-cli/src/main.rs:1223-1225`), so the new path *replaces a refusal*
rather than pre-empting a working input. **Stdin filters blank lines exactly as
`--in` does**, so a record's index is its position among the non-blank lines in
both cases; anything else makes `me: record 3` mean two things.

### 4.3 (I9) (r1-I9) STEP 3: THE BASE RULE, THEN PRECEDENCE — AND `--allow-weak` IS NOT A MODE

`me sysw pack` already has `--passphrase-words`, `--passphrase-ask`,
`--no-passphrase` and `--allow-weak`. v1 said "seal by content" and never said
what happens when the operator also passes a flag — so step 3's test could not be
written.

**NORMATIVE, in this order, because v2 kept only the second half (r1-I5):**

1. **THE BASE RULE, which is spec §2.4 and an operator ruling in spec §8:** a
   payload holding **no** `Class::is_secret()` record packs **UNSEALED**; one
   holding any packs **SEALED**. v2's step-3 test asserted only *precedence* —
   what happens when a flag is present — and **never asserted the outcome in the
   case spec §2.4 actually rules**, so a `me` in which `Class::Transaction` is secret,
   or in which "content decides" defaults to sealing, passed it. That is not
   hypothetical plumbing: it is W3 of this plan's §2.4.
2. **An explicit passphrase-MODE flag wins** — `--passphrase-words`,
   `--passphrase-ask`, `--no-passphrase`. These three are already mutually
   exclusive in clap (`crates/me-cli/src/main.rs:177-185`), so at most one is
   ever present.
3. **`--allow-weak` IS NOT A PASSPHRASE MODE and does NOT count as "an explicit
   flag" for rule 2 (r1-I9).** It is *accepted and ignored*
   (`crates/me-cli/src/main.rs:189-192`) and says so on stderr
   (`crates/me-cli/src/main.rs:893-895`). v2's *"an explicit flag always wins"*
   listed it among the four and therefore admitted a literal reading in which
   `me sysw pack --allow-weak < tx.txt` **seals a transaction-only payload** —
   precisely what spec §2.4 rules against. **Test: `--allow-weak` on a
   transaction-only payload packs UNSEALED**, with the existing ignore-warning
   still printed.
4. **stderr states which rule applied and why, on every run.** A
   content-dependent default that is silent is worse than the default it
   replaced.

## 5. `mt-codec` IS PUBLISHED FIRST — the v1 deadlock is dissolved

**(I6) v1 deadlocked**: §5 said publish before step 5, §5 also said publish after
V4 pinned the byte order, and §6 forbade a path dependency. All three cannot hold.

**It dissolves because the premise was false.** v1 wanted to wait for V4 to
resolve the byte order — but **`mt-codec` already implements it**, has done since
before this plan existed, and §1.1 now quotes the source. **There is nothing left
to learn before publishing.**

So: **publish `mt-codec` 0.1.0 to crates.io, then depend on the pinned published
version.** No path dependency, no git dependency, no ordering knot.

**`cargo publish` remains irreversible** — a version can be yanked, never
replaced — so it happens once **this plan is GREEN**, not before. The thing that
made it risky was uncertainty about the byte order, and that uncertainty is gone.

**(M8) AND THE DEAD `bitcoin = "0.32"` COMES OUT OF `mt-codec` FIRST.** Its
manifest declares it and `src/` uses it **nowhere** (0 occurrences of `bitcoin::`).
Publishing 0.1.0 as it stands bakes an unused dependency into an artifact that
can be yanked but never replaced, and pulls `bitcoin` into every downstream build
of `me` **twice over** — once uselessly through `mt-codec`, once for real through
§2.2's own declaration. **Remove the line, land it in `mnemonic-transaction`,
then publish.** That is a change to another repo, so it is named here as a
precondition of §5's publish step rather than as work P1 does in this one.

**Two things §5 asserts and does not prove**, named so a reviewer spends budget
elsewhere: `cargo publish --dry-run` has **not** been run on `mt-codec`, and its
manifest being workspace-inherited with no path dependencies is *necessary*, not
sufficient. **The dry run is a precondition of the publish step**, not of this
plan closing.

## 6. WHAT MUST BE TRUE TO CLOSE P1

- This plan is **0C/0I** under the R0 loop, over lenses enumerated up front.
- **The document gates below have RUN on this document** — §6.1, which is a
  different sentence from the one v2 had, and the difference is the point.
- **V1–V22 pass**, and **V4a/V4b write out txid, wtxid and `chunk_set_id`
  explicitly** in the vector file, not only in code.
- **Every rule E1–E16 has a test that goes RED without its check**, and **so does
  every refusal P1 adds** — §6.2. A rule with no negative test is a comment.
- **`ClassTransaction` is WIRED, not merely specified** (§2.4): `grep -c
  'Class::Transaction' crates/me-cli/src/sysw/` is non-zero at **all five** sites
  W1–W5, `Class::Transaction::is_secret()` is **false**, and **`me sysw pack`
  followed by `me sysw show` round-trips a real `tx:` record** — the end-to-end
  assertion no record-level vector can substitute for.
- **A `tx:` record that fails DECODE aborts the pack** with nothing on stdout and
  exit 4 (§1.5), asserted with four other valid records present.
- `cargo nextest run --locked` green; `cargo clippy --all-targets` clean.
- **The vectors were not produced by the code they judge** (§3.2) — satisfied by
  construction, since §3's transaction comes from a corpus generated by
  `scripts/gen-mt1-vectors.py`.
- **`mt-codec`'s dead `bitcoin` declaration is removed (M8), `cargo publish
  --dry-run` is clean, `mt-codec` is published**, and `me` depends on the
  **pinned published version**.
- **Spec corrections this phase owns — THREE, not one (r1-I7).** All three are
  **spec §2.3's**, all three are recomputed in §3.1, and the *"opaque"* item v2 listed is
  **struck: it has no referent** (r1-I8, §2):
  1. *"a **16,367-byte** raw transaction"* → 16,322 B of body, minus the fields.
  2. the 5/2 row's *"raw-only at 8191, by 31 chars"* → **8,249 chars, 58 OVER**.
  3. the 10/2 chunks row's *"18,583 … ✅"* → **37,255 chars, 4,521 OVER 32,734**;
     the chunks body is ASCII inside the record's hex, so each `mt1` character
     costs two.
- **The near-miss rule**: every guard here is tested against its nearest
  *legitimate* input as well as the hostile one — **(M3) v2 said "seven instances
  this cycle" against spec §5's six and named no seventh.** The count is dropped;
  the rule is not. P1's own near-miss pairs are named where they live: V22
  (`len = 65` refused / `len = 64` passes), V17 (`len = 7` and `len = 9` refused /
  `len = 8` passes), V18 (witness-stripped refused / with-witness passes), V19
  (trailing `\n` refused / no trailing `\n` passes), V20 (uppercase refused /
  lowercase passes), V10 (trailing bytes refused / exact end passes), and §4.2's
  TTY refusal (a TTY refused / a pipe passes).

### 6.1 (r1-I6) THE GATES THAT ACTUALLY READ THIS DOCUMENT — and the one that does not

**v2's first bullet was *"the build gate has run on it"*, and that is a
STRUCTURAL FALSE PASS.** Round 1 executed it:

```
$ ./scripts/plan-build-gate.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
test result: ok. 77 passed; 0 failed
   PASS: the CLI tests compile
   clippy clean
EXIT=0
```

**It extracted nothing.** The extractor accepts a ```rust block only when a
preceding anchor names `src/seal/*.rs` or `tests/seal_cli.rs`
(`scripts/plan-build-gate.sh:70-74`). This plan has neither anchor, and its only
```rust block is a quoted signature from another crate. So the script copied the
pristine crate, applied **zero** edits, and built, tested and linted **`me` as it
already is** — green, for an artifact it never read a line of. Its honesty clause
misfires the same way: the result block names *"src/main.rs and src/validate.rs
arrive as fragments"*, which are the **encrypted-payload** plan's files.

**NORMATIVE: `plan-build-gate.sh` is NOT a close condition for P1**, and this
plan does not cite it. What would have to change for it to apply, stated
precisely rather than left as "someday": the extractor's anchor test at
`scripts/plan-build-gate.sh:70-74` would have to accept `src/sysw/*.rs` and
`tests/sysw_cli.rs`, **and** this plan would have to carry assemblable whole-file
```rust blocks under those anchors — which it deliberately does not, because P1's
code is written test-first in §4, not transcribed from a plan.

**THREE gates DO read this document, and P1 runs all three** — before a
reviewer, and again before committing any fold, because **a fold is authorship
and re-earns the gate**. Each row states what a PASS looks like on *this*
document, measured on this fold, so a future run has something to diff against:

| gate | what it reads here | PASS on this document | what it does NOT cover |
| --- | --- | --- | --- |
| `./scripts/plan-cite-check.sh` | every `path:line` citation, resolved against the real tree | **45 of 53 resolve; the 8 dangling are exactly the 4 into the vendored `bitcoin` crate and the 4 into `mnemonic-transaction`** — see below. Any ninth is a defect | **interpretation** — it proves the line exists, never that this plan reads it right; and it cannot check absence claims |
| `./scripts/plan-table-check.sh` | every table row against its header's cell count | **84 rows checked, 0 malformed, exit 0** | cell **content**; a right-width row with wrong values passes |
| `./scripts/plan-fold-sweep.sh <doc> --terms <the six below>` | **terms this fold removed that survive elsewhere** | **exactly 6 hits, one per term, ALL of them inside the list block below — the self-reference. A seventh hit anywhere else is a real finding** | it flags candidates, not defects; and terms nobody named |

**`plan-glyph-check.sh` is NOT a close condition here either, and the reason is
not the same as the build gate's.** It runs and it reads this document — but it
scans for glyphs **the SeedHammer II display font** cannot draw, and **P1 emits
no device strings**: its refusals go to a host terminal via `eprintln!` (§1.5).
The device-facing strings of this cycle are **P4's and P5's**, and the gate is
theirs. Run against this plan it reports **10 undrawable of 86**, every one an em
or en dash in this document's own *prose blockquotes and table cells* — a
limitation the script states about itself (*"it only inspects what it can RECOGNISE as an operator-facing
string: markdown blockquotes, and backticked spans of 40+ characters"*). **A gate
that is red for non-defects trains a reader to ignore it just as surely as one
that is green for everything.**

**The `--terms` list is fixed, because the explicit mode is the one that works
and the fold author is the only one who knows what was superseded.** These six,
each of which this fold removed and each of which must survive **only** in this
block:

```
'INTERNAL byte order'          the v2 layout-table row      (r1-C1)
'§1.3 says'                    the dangling body citation   (r1-C2)
'V1–V15 pass'                  the superseded vector range
'Every rule E1–E10 has'        the superseded rule range
'16,367-byte raw transaction'  the uncorrected spec figure
'Seven instances'              the unexplained near-miss count (M3)
```

**The fold sweep is in this list because of r1-C1.** The `INTERNAL`/`DISPLAY`
split was *exactly* the shape it exists to catch — a fact corrected in the prose
and left standing in the table three sections away — and reading the diff cannot
find it, because by construction the defect lives in the text the diff did not
touch.

**WHAT `plan-cite-check.sh` CANNOT REACH, named because a gate that hides its
blind spot is worse than no gate.** Its `ROOTS` list
(`scripts/plan-cite-check.sh:72-79`) covers the fork, this repo,
`descriptor-mnemonic`, `mnemonic-toolkit`, `mnemonic-key` and `mnemonic-secret`.
It does **not** cover `mnemonic-transaction`, and it cannot cover the vendored
`bitcoin` crate at all. **Those are the 8 dangling: 4 into `mnemonic-transaction`**
(`pipeline.rs` x3 for §1.1's ruling, its sole consumer and E13's decoder
tolerance; and its `lib.rs` for §3.2) **and 4 into `bitcoin`** (`transaction.rs` x3
and `encode.rs` x1, the API facts §1.4 and §2.2 rest on). They are
**verified by command instead** — run these, they are the P1 equivalent of the
build gate for the facts the gate cannot reach:

```sh
MT=/scratch/code/shibboleth/mnemonic-transaction
B=$(echo ~/.cargo/registry/src/*/bitcoin-0.32.9)

  ## §1.1's byte-order ruling, its sole consumer, and E13's decoder tolerance
sed -n '17,27p;54p;66p' $MT/crates/mt-codec/src/string_layer/pipeline.rs

  ## §1.1's and §2.2's bitcoin API facts: Txid display order, witness-free
  ## compute_txid, segwit consensus_encode, deserialize refusing trailing bytes
sed -n '44,50p;780,787p;1239,1260p' $B/src/blockdata/transaction.rs
sed -n '163,172p' $B/src/consensus/encode.rs

  ## §3's corpus: its independent generator and the even vector's three values
python3 -c "import json; d=json.load(open('$MT/crates/mt-codec/src/test_vectors/mt1_v1.json')); print(d['generator']); print({k: v for k, v in d['vectors'][0].items() if k in ('size_bytes','txid','wtxid','set_id','txid_is_wtxid')})"

  ## §2.2's and M8's absence claim: bitcoin declared, never used -- prints 0
grep -rn 'bitcoin::' $MT/crates/mt-codec/src/ | wc -l

  ## §2.2's feature choice: bitcoin's default is ["std", "secp-recovery"]
grep -A 4 '^default = ' $B/Cargo.toml
```

**And §1.4's measurement is a scratch crate, reproducible in four minutes.**
`bitcoin = { version = "0.32", default-features = false, features = ["std"] }`
plus `hex`, a `main` that deserialises the corpus `raw_hex`, prints
`compute_txid()` and `compute_wtxid()`, compares `serialize(&tx)` to the body,
then clears every `TxIn::witness`, re-serialises and prints the stripped length,
whether it still deserialises and whether its txid is unchanged. **Run while
writing this fold, and it built clean at those exact features** — which is how
§2.2's dependency row is a measurement rather than a guess.

**Every command above was executed while writing this fold and its output is what
§1.1, §1.4, §2.2, §3 and §5 state.** That is the standard the build gate would
have enforced if it could read this document, applied by hand because it cannot.

**Adding `/scratch/code/shibboleth/mnemonic-transaction` to that `ROOTS` list is
the one-line change that would bring the FOUR `mnemonic-transaction` citations
inside the gate**, leaving 4. The `bitcoin` four cannot be gated at all: they
point into a registry cache whose path carries a version hash, and a gate that
resolved them would be asserting a *local build artifact*, not a source of truth.
The change is named rather than made, because this plan does not edit tooling.

### 6.2 (r1-I4) REFUSAL COVERAGE IS EVERY REFUSAL P1 ADDS, NOT JUST E1–E16

v1's closure list said *"every refusal added has a test that goes RED without its
check"*. **v2 narrowed it to *"every rule E1–E10"*** — the record-codec rules —
which silently dropped **four of this phase's own refusals**, including the
anti-smuggling gate the whole plan is built around:

| refusal | where | covered by E1–E16? |
| --- | --- | --- |
| **R2** — a `tx:` record on argv | §4 step 11 | **no** |
| **R7** — empty stdin | §4 step 2 | **no** |
| **the TTY refusal** | §4.2, NORMATIVE | **no** |
| **§2.3's decode-failure refusal** | §4 step 7 | **no** — and it is the reason EPD §6.3 exists |

**NORMATIVE: all four get a test that goes RED when its check is removed**, the
same standard as E1–E16, verified by deleting the check by hand and watching the
test fail **for the stated reason**.

**And the tooling citation v1 carried is corrected rather than restored.**
`refusals.toml`, `check-refusal-coverage.sh` and `mutate-refusals.sh` are **`mt`'s
and live in `mnemonic-transaction/scripts/`** — `ls scripts/` in *this* repo has
none of the three, and spec §5 says so in its own words (*"`mt` has this machinery … the
fork side needs its equivalent"*). So P1 owns the **per-refusal RED test**, done
by hand; spec §6's **P6** row owns building this repo's equivalent of the
**bijection sweep**. Naming P1 as the owner of the sweep would move a gate onto a
phase that has no tool for it.

## 7. OUT OF SCOPE

P2–P6. The device. The plate. `mt encode --record` (that is P2, and it is what
*produces* these records — this phase only reads and packs them).

**Named out of scope because this fold touched them and stopped:**

- **`scripts/plan-cite-check.sh`'s `ROOTS` list** (§6.1). The one-line change is
  specified; making it is tooling work, not P1's.
- **This repo's equivalent of `check-refusal-coverage.sh` / `mutate-refusals.sh`**
  (§6.2) — spec §6's **P6** row owns it. P1 owns the per-refusal RED tests, by
  hand.
- **Removing `mt-codec`'s dead `bitcoin` declaration** (§5, M8) — a change in
  `mnemonic-transaction`, and a precondition of the publish step rather than work
  in this repo.
- **`bitcoin 0.32`'s no-`std` story.** §2.2 keeps `std` explicitly and says why;
  whether a `no_std` `me` is ever wanted is not a question P1 opens.
