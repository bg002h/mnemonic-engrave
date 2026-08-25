# IMPLEMENTATION PLAN — P1: `me`'s transaction container

**Status:** DRAFT v7, pre-R0. **Round 4 returned 2 Critical / 5 Important / 1
Minor on v6**, and scored round 3's eight remaining findings at **5 FIXED, 2
PARTIAL, 1 WRONGLY FIXED**. This is the **FOLD of round 4**, and both Criticals
were **defects the previous fold introduced**: v5 answered round 3 by ADDING
§2.5a and never retracting the §2.4 rows it replaced, so §4 and §6 still pointed
an implementer at the superseded edit (**C1**); and §1.4a's ruling turned V3 into
a payload while §4's step rows were carried across unexamined, leaving step 4's
gate unable to go green at all (**C2**, measured: `exit 4`). **A replacement that
does not retract is an alternative**, and that is the lesson this fold is named
for.

**The reports are `design/agent-reports/R0-P1-plan-round0.md`** (5C/13I/5M on v1),
**`R0-P1-plan-round1.md`** (3C/11I/8M on v2), **`R0-P1-plan-round2.md`**
(2C/7I/4M on v3) **and `R0-P1-plan-round3.md`** (the live one, 2C/7I/3M on v4),
all persisted before any of this was written. Findings are marked
inline — round-0 markers as `(C1)`, `(I7)`; round-1 as `(r1-C2)`, `(r1-I9)`;
round-2 as `(r2-C1)`, `(r2-I3)`; round-3 as `(r3-I1)`, `(r3-M2)` — so a reader
can trace each one to the report that raised it.

> **THE TWO THINGS THIS FOLD CHANGES STRUCTURALLY, stated once at the top
> because everything below is downstream of them.**
>
> 1. **A `wtxid` field is added at offset 38 (r2-C1).** Round 2 built a scratch
>    crate and measured that **E11 does not fire on V18**: a witness-stripped
>    body re-serialises to itself, so the rule invented to catch it could not.
>    The txid cannot see the difference either — stripping the witness does not
>    change it. A carried **wtxid** can, and the reason is exact: for a
>    transaction with no witness data the wtxid **equals** the txid, so a
>    stripped body announces itself. **The framing goes 43 → 75 bytes and every
>    ceiling in §3.1 moves with it.**
> 2. **The chunks form rides as BARE RECORDS (§1.4a, operator ruling
>    2026-08-24).** `form = 0x02`'s body becomes **empty**; the `mt1` strings are
>    sibling records in the same section, exactly as `md1`/`mk1` already ride.
>    This is what makes the pathological 10-in/2-out spend fit at all.

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
> be read against — **and round 2 found a fourth, which is why item 4 is here:**
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
>    own closure condition. **Round 2 found the same shape one layer in
>    (r2-C2):** the five wiring sites were correct and *insufficient*, because
>    none of them could carry WHICH rule a record broke. §2.4 now names **ten**.
> 4. **A rule that does not enforce what it claims (r2-C1).** E11 was invented to
>    close r1-C2 and was *reasoned*, not executed. Round 2 executed it and it
>    returned `true` on the vector written to prove it fires. **NORMATIVE FOR
>    THIS DOCUMENT: no rule is added here without a stated input that would fail
>    it, and that input measured.** Every new rule below (E17–E20) carries its
>    measurement or names the command that produces it.

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
 5    1    form        0x01 = RAW (transaction bytes) | 0x02 = CHUNKS (metadata only)
 6   32    txid        DISPLAY order (byte-reversed), witness-STRIPPED  -- see §1.1
38   32    wtxid       DISPLAY order (byte-reversed), over the CANONICAL
                       serialization (witness INCLUDED when there is one) -- §1.1a
70    1    n_fields    count of legend TLVs, 0..=255
71    ..   fields      n_fields x { tag:u8, len:u16 BE, value:len bytes }
 ..   4    body_len    u32 BE
 ..   N    body        RAW: body_len bytes.  CHUNKS: body_len == 0 and the body is
                       EMPTY -- the chunks are SIBLING RECORDS (§1.4a)
```

**The framing is 75 bytes** — `4 + 1 + 1 + 32 + 32 + 1 + 4`, with `n_fields = 0`.
v3's was 43. **Every ceiling in §3.1 is recomputed against 75**, and the figure
that moved furthest is the record-framing ceiling: **16,322 → 16,290 bytes**.

**All multi-byte integers are BIG-ENDIAN.** Stated once, here, because a wire
format with mixed endianness is a defect generator.

> **(r1-C1) THE TABLE IS NORMATIVE AND SO ARE §1.1 AND §1.1a, AND THEY MAY NEVER
> DIVERGE.** This row said `INTERNAL` in v2 while §1.1 said `DISPLAY` sixteen
> lines below — round 0's C1, half-folded, which is the same document carrying
> the same contradiction. **A Go porter transcribes the table**, because it is
> the only part of this plan shaped like a struct. So: each identifier's byte
> order and its witness treatment are stated in **exactly two places** — this
> table and its own subsection — they must say the same thing, and §6 gates that
> by grep rather than by care. **The `wtxid` row inherits the whole of that
> rule**, and it inherits the trap too: `Wtxid` is declared in the *same*
> `hash_newtype!` block as `Txid` (`bitcoin-0.32.9/src/blockdata/transaction.rs:44-55`),
> so it is `DISPLAY_BACKWARD` in exactly the same way and fails in exactly the
> same way.

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

### 1.1a (r2-C1) THE WTXID FIELD — NEW, and it is the rule E11 was supposed to be

**NORMATIVE: the `wtxid` field holds the 32 bytes of the transaction's wtxid in
its STANDARD DISPLAY ORDER**, computed over the transaction's **canonical
consensus serialization** — witnesses INCLUDED when the transaction has any,
excluded when it has none. In `bitcoin 0.32` it is exactly
`tx.compute_wtxid().to_string()`, which hashes `consensus_encode`
(`bitcoin-0.32.9/src/blockdata/transaction.rs:804-808`).

**WHY IT EXISTS: because E11 does not fire, and round 2 proved it by execution.**
E11 says *"re-serialising the decoded transaction MUST reproduce the body byte for
byte"*, and V18 — the witness-stripped body — was written to be the record E11
refuses. It is not refused. Re-measured for this fold on the exact dependency
§2.2 chooses, on §3's transaction:

```
with-witness body      : 222 B
  serialize(tx)==body  : true
  txid  (display)      : 2dcf2b97...72ebf630
  wtxid (display)      : d5717c03...ed836f51
  txid == wtxid        : false
witness-stripped body  : 113 B
  deserialises         : true
  E11 serialize(deser(s))==s          : TRUE   <- E11 does NOT fire
  txid unchanged                      : true   <- the txid cannot see it either
  *** wtxid == txid                   : TRUE
  *** carried wtxid matches this body : FALSE  <- E17 fires
```

**E11 is a CANONICALITY check, not a witness-presence check.** The stripped body
decodes to a transaction whose inputs carry no witness, so
`uses_segwit_serialization()` is false
(`bitcoin-0.32.9/src/blockdata/transaction.rs:1057-1065`), `consensus_encode`
takes the legacy branch (`bitcoin-0.32.9/src/blockdata/transaction.rs:1244-1246`),
and it reproduces the stripped body
exactly. E11 rejects a non-canonical encoding of a transaction; it cannot see
that a *different, canonical* transaction was substituted. **And the txid cannot
help, because stripping the witness is precisely the operation the txid is
defined to ignore.**

**The wtxid can, and the reason is a definition rather than an accident.** For a
transaction with no witness data, `compute_wtxid` and `compute_txid` hash the
same bytes — `bitcoin` documents it in those words at `:801-802` (*"For non-segwit
transactions which do not have any segwit data, this will be equal to
[`Transaction::txid()`]"*) — so:

| the body is | its computed wtxid | verdict against a carried wtxid ≠ txid |
| --- | --- | --- |
| the honest segwit serialization | the real wtxid | **matches — accepted** |
| the same transaction, witnesses stripped | **equal to its txid** | **≠ carried — REFUSED (E17)** |

**THE ACCEPTED COST, stated in the plan's own voice because the last version of
this argument over-claimed and cost a round.** A record whose txid AND wtxid are
*both* recomputed from a stripped body is internally consistent, and **nothing in
the record can tell it from an honest witness-free transaction** — because it
*is* one. That is not a gap E17 closes and it is not a gap any field can close:
the record is the whole artifact, and both identifiers are asserted by whoever
built it. **What E17 buys, exactly:** an encoder that strips the witness while
computing its identifiers from the real transaction is refused — the honest-bug
case, which is the interop risk between P1's Rust and P3's Go — and the operator
gains a **second value to compare against `mt inspect`**, one that the txid
provably cannot substitute for. **V18 and V26 are the same 113 bytes and differ
only in the carried wtxid**, which is the sharpest near-miss pair in this plan
and is what makes the residual visible rather than argued.

**In Rust this is one call, and it is the SAME trap as §1.1's.**
`Wtxid` is declared in the same `hash_newtype!` block as `Txid`
(`bitcoin-0.32.9/src/blockdata/transaction.rs:44-55`), so `Wtxid::to_string()` is
the display hex and `as_raw_hash().to_byte_array()` is the internal bytes. The
field is the former's bytes, and the test compares
`hex(field) == tx.compute_wtxid().to_string()`.

**`mt` already computes it, so P2 has nothing to invent.** Both vectors of
`mnemonic-transaction/crates/mt-codec/src/test_vectors/mt1_v1.json` carry a
`wtxid` alongside their `txid`, and a `txid_is_wtxid` boolean — read from the
corpus for this fold, not inferred.

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
| E4 | **`71 + Σ(3 + len) + 4 + body_len` MUST equal the decoded length exactly.** | makes E3 checkable rather than aspirational. **The constant is 71, not v3's 39** — `4 + 1 + 1 + 32 + 32 + 1`, the fixed part up to and including `n_fields`, which the `wtxid` field moved (§1.1a). A Go porter transcribes this arithmetic |
| E5 | **`body_len` is validated against the remaining length BEFORE any allocation.** | **(I2)** a 153-character record — the shortest legal one — otherwise declares 4 GiB; EPD §6.2 sets the precedent — bound before you trust |
| E6 | **A zero-length TLV value is REFUSED.** | **(I3)** absence is *omission* (E7); a present-but-empty field is a second spelling of nothing |
| E7 | **An absent optional field is OMITTED from the list.** There is no empty encoding and no sentinel. | spec §2.1b asked what absence looks like |
| E8 | **An unknown tag is REFUSED, not skipped.** | skipping is how two implementations silently diverge |
| E9 | **A bad `magic`, an unknown `version`, or a `form` outside {0x01, 0x02} is REFUSED, each with its own message.** | **(I4)** v1 gave a verdict for unknown *tags* only |
| E10 | **`n_fields` MUST equal the number of TLVs actually parsed.** | a disagreement is a malformed record, not a hint |
| **E11** | **RAW: re-serialising the decoded transaction MUST reproduce the body BYTE FOR BYTE.** | **(r1-C2, CORRECTED r2-C1)** this is a **CANONICALITY** rule and nothing more — it rejects a non-canonical encoding of a transaction, and it **cannot** see a different, canonical transaction substituted. v3 claimed it enforced witness presence; **measured false**, see §1.1a. It is retained for the Go port, and §6 records that it has **no RED test in Rust** — `bitcoin`'s decoder already refuses both things it could catch |
| **E12** | **CHUNKS: the chunk records are separated by the CONTAINER's own record separator — a single `\n` (0x0A), no trailing separator, no empty record.** | **(r1-C2, RE-HOMED §1.4a)** the chunks are no longer elements inside a body, so this is no longer a body rule: it is the container's, and `me` already has it — `payload.public.join("\n")` at `crates/me-cli/src/sysw/mod.rs:260` — but **(r3-I1) the ASSERTION v4 cited is `seal`'s, not `sysw`'s.** `joins_with_lf_and_no_trailing_lf` lives at `crates/me-cli/src/seal/container.rs:85` and asserts on `seal`'s `encode_section`, the frozen container whose `MAX_RECORDS = 24` and `MAX_RECORD_LEN = 512` §3.1 insists are **not** `sysw`'s; `grep -rn 'joins_with_lf_and_no_trailing_lf' crates/` returns that **single** hit, and `sysw`'s four joins (`crates/me-cli/src/sysw/mod.rs:192`, `:260`, `:278`, `crates/me-cli/src/sysw/pubhash.rs:27`) are reached by it nowhere — change `:260`'s separator and it stays GREEN. **The fact did leave the plan, and it landed somewhere nothing asserts it.** So P1 OWES E12 a RED test in `sysw`: **§4 step 12**. This is §4.1's own shape — *"a test that only checks the raise would pass if someone edited the frozen container instead"* — applied to the rule one section earlier |
| **E13** | **Every `mt1` chunk RECORD is lowercase ASCII with no leading or trailing whitespace and no interior whitespace or `-`.** | **(r1-C2, RE-SCOPED §1.4a)** `mt-codec`'s `to_symbols` does `s.trim().to_ascii_lowercase()` (`mnemonic-transaction/crates/mt-codec/src/string_layer/pipeline.rs:66`), so it **accepts** uppercase and padded strings — a different record, a different EPD §6.6 public-data hash, and `mt_codec::decode_chunk` still green. The tolerance is the decoder's; the record layer may not inherit it. `me` already refuses exactly this for md1/mk1 — `first_noncanonical` plus the uppercase scan at `crates/me-cli/src/seal/record.rs:118-128` — and `mt1` joins them. **BOTH halves are vectored (r2-I7)**: V20 the case, V23 the whitespace |
| **E14** | **Tag `0x01`'s value MUST be valid UTF-8. Invalid is REFUSED.** | **(r1-I10)** Rust refuses by default and Go accepts by default — one implementation refuses what the other engraves |
| **E15** | **Tag `0x01`'s value is `1..=64` bytes.** | **(r1-I10)** a `u16 len` otherwise admits a 65,535-byte label into a plate legend budgeted at 34 characters |
| **E16** | **A fixed-width tag MUST carry exactly its width: `0x02` → 8, `0x03` → 4. Any other `len` is REFUSED.** | **(I3, r1-I11)** E6 refuses only `len = 0`; `len = 1..7` is the actual gap, and the fee is engraved. **BOTH halves are vectored (r2-I7)**: V17 the fee tag, V17b the fingerprint tag |
| **E17** | **The decoded transaction's WTXID (display order, §1.1a) MUST equal the carried `wtxid` field — on BOTH forms.** | **(r2-C1)** this is the rule E11 was believed to be. **What fails it, measured (§1.1a):** V18, the witness-stripped body, whose computed wtxid equals its txid and so differs from the carried one in all 32 bytes. **What passes it:** V26, the same 113 bytes with the wtxid recomputed honestly — the near-miss, and the accepted cost §1.1a states |
| **E18** | **`form = 0x02` (CHUNKS) MUST carry `body_len == 0` and an EMPTY body. A non-empty CHUNKS body is REFUSED, and so is `form = 0x01` with `body_len == 0`.** | **(§1.4a)** the chunks are sibling records now, so a CHUNKS record with a body is either v3's superseded framing or a record carrying **both** forms — which is exactly what **spec §5's R4′** refuses, made checkable in one comparison instead of left to a reader. Vectored by V19 |
| **E19** | **An `mt1` chunk record MUST verify AS WRITTEN: `mt_codec::decode_chunk` reporting ZERO BCH corrections.** | **(§1.4a)** `decode_chunk` error-corrects up to `t = 4` before reporting success (`mnemonic-transaction/crates/mt-codec/src/string_layer/pipeline.rs:160`), so without this `me sysw pack` silently repairs a damaged chunk and then packs the repaired string as if the operator had supplied it. **This is not a new posture** — `me` already refuses a non-pristine `mk1` record in these words: *"not pristine: required N BCH correction(s)"* (`crates/me-cli/src/seal/record.rs:137-142`). Vectored by V24 |
| **E20** | **Every `mt1` record in the payload MUST belong to exactly one CHUNKS `tx:` record's set, and every CHUNKS `tx:` record MUST have a COMPLETE set: `count` chunks, indices `0..count-1`, no gap and no duplicate.** | **(§1.4a)** the binding is R15's — `chunk_set_id` == the top 20 bits of that record's carried txid — so an orphan chunk, a missing index or a second copy of one is a payload nobody can reassemble. `mt1`'s header is `version(5), chunk_set_id(20), count−1(15), index(15)` — 55 bits (`mnemonic-transaction/crates/mt-codec/src/string_layer/header.rs:4`), so **all three facts are read off each record without reassembling anything**. Vectored by V25 |

**Every one of E1–E20 gets a vector (§3) and a test that goes RED without its
check — WITH TWO NAMED EXCEPTIONS, because a completeness claim that is false is
worse than a narrower one that is true.** A rule with no negative test is a
comment. **E1's negative is V16**, not V2 — **(r1-I1)** V2 is a *positive* vector
whose bytes are ascending by construction, so deleting the ordering check
entirely left it green and the closure condition unsatisfiable for the one rule
it was written for.

**THE TWO EXCEPTIONS, and their owners (r2-M1, r2-C1):**

| rule | why it has no RED test in Rust | who owns it |
| --- | --- | --- |
| **E7** — absence is omission, no empty encoding, no sentinel | it is an **encoder** rule. Its one checkable clause (*no empty encoding*) is E6, vectored by V12; *"no sentinel"* is semantic and no decoder check can reach it | nobody — it constrains what P1 EMITS, and V5 is its positive |
| **E11** — RAW re-serialisation equality | **measured for this fold, not argued:** `bitcoin::consensus::deserialize` already refuses everything E11 could catch. A non-minimal `VarInt` input count is refused `non-minimal varint` on **both** the segwit and the legacy body, and a trailing byte is refused `parse failed: data not consumed entirely`. Delete E11's `==` in Rust and every vector stays green | **P3.** The Go port's decoder is hand-written and has no such guarantee; E11's RED test is a **Go** test against a decoder that accepts a non-minimal `VarInt` |

**Neither exception is a licence.** E7 and E11 are the only two, they are named
here and again in §6, and **any future rule that cannot be made to fail must join
this table or be deleted.** v3's completeness claim over its sixteen rules was
false for both of them and for E13's and E16's second halves (r2-I7), and it read
as a stronger claim than the plan could support.

### 1.4 (r1-C2) THE BODY — what `body_len` bytes actually contain

**v2 DELETED this and left §1.1 citing it.** v1 had it as a two-row table; the
rewrite replaced the section wholesale with E1–E10 and dropped the two rules it
already had, so v2 stated the record's framing completely and **never said what
the record carries**. Restored, as rules rather than as a table cell:

| `form` | the body is |
| --- | --- |
| `0x01` RAW | **the transaction's CANONICAL CONSENSUS SERIALIZATION** — the BIP-144 segwit form (version, marker `0x00`, flag `0x01`, inputs, outputs, each input's witness, locktime) **when any input carries a witness**, and the legacy form (version, inputs, outputs, locktime) **when none does.** Exactly what `bitcoin`'s `consensus_encode` emits |
| `0x02` CHUNKS | **EMPTY. `body_len == 0`** (E18). The `mt1` strings are SIBLING RECORDS in the same section — **§1.4a**, the ruling of 2026-08-24 |

**(r2-I1) THE RAW ROW USED TO ENUMERATE marker+flag, AND THAT LAYOUT IS
UNDECODABLE FOR HALF OF BITCOIN.** v3 wrote the body as *"WITH WITNESS — the
BIP-141 form: version, marker `0x00`, flag `0x01`, …"*. Enumerated like that it
is a struct layout, and a Go porter transcribes struct layouts. For an ordinary
P2PKH/P2SH-only spend — a signed transaction with **no** witnesses — that layout
cannot be decoded at all. Measured:

```
witness-free tx serialises to 113 B, first 6 bytes [02, 00, 00, 00, 01, 7c]
  marker 00 flag 01 at offset 4?                   false
  a segwit-FLAGGED body with all-empty witnesses:
    "parse failed: witness flag set but no witnesses present"
```

The refusal is structural and BIP-144 requires it
(`bitcoin-0.32.9/src/blockdata/transaction.rs:1280-1282`). So the row now states
the **rule the encoder actually follows** —
`uses_segwit_serialization()` is *"any input carries a witness"*
(`bitcoin-0.32.9/src/blockdata/transaction.rs:1057-1065`), and
`consensus_encode` branches on it
(`bitcoin-0.32.9/src/blockdata/transaction.rs:1244-1246`) — and E11 and this row agree
instead of disagreeing for every legacy transaction. **V26 is the witness-free
vector, and v3 had none** (§3's own NORMATIVE sentence pinned one segwit
transaction for every vector, which closed the txid axis and left this one
untested).

**RAW: WHAT SEPARATES A HONEST BODY FROM A STRIPPED ONE IS E17, NOT E11
(r2-C1).** Walk a witness-STRIPPED body through every other check in this plan:
`magic`, `version`, `form` pass; **every TLV rule passes**, because they
constrain the fields and the lengths and not the body's shape; §2.2's
*"deserialise the body as a Bitcoin transaction"* passes, because a witness-free
serialisation **is** a valid transaction; §2.2's txid equality passes, **because
the txid strips witnesses anyway**; and **E11 passes too** — measured, §1.1a.
Two conforming records, different bytes, and one plate carries a transaction with
every signature removed — unbroadcastable, which is the one thing this artifact
exists to make possible. **E17 is the check that fires**, and it is one `==` on
two 32-byte slices.

**MEASURED, on §3's transaction, with the exact dependency §2.2 chooses** — a
scratch crate built and run while writing this fold, and reproducible from §6.1.
**The fifth row is the one v3 did not measure, and it is the one that decided the
claim:**

```
body (with witness)            222 B     serialize(&tx) == body       ->  true
same tx, witnesses stripped    113 B     deserializes                 ->  true
                                         SAME txid as the body        ->  true
                                         E11: reserialise == body     ->  TRUE   (!)
                                         its wtxid == its own txid    ->  true
                                         its wtxid == CARRIED wtxid   ->  false  (E17)
trailing 32 bytes appended               deserialize rejects          ->  true
```

**109 bytes of signatures, silently absent, with every other check in this plan
green — including the one written to catch it.** That is what E17 costs one `==`
to prevent, and V18 is the vector.

**CHUNKS: the separator, the trailing separator and the case are all rules
(E12, E13), never vector cells.** v2 left `LF separation` living in V3's *"pins"*
column — and this plan's own C4 lesson is that **a vector is an instance, not a
rule**. §1.4a moves both rules to the layer that now owns them; neither is
dropped.

### 1.4a THE CHUNKS FORM RIDES AS BARE RECORDS — operator ruling, 2026-08-24

**THE DEFECT THAT FORCED IT.** An `mt1` chunk is bech32 — already printable
ASCII — and the reserved-prefix rule hex-encodes a record body, so **every
character costs two**. Compounded with the chunk text's own ~2.3 characters per
byte, the worst measured pathological spend could not enter the container at all
in the form it was designed for, while the *same transaction* fitted as raw
bytes with room to spare. Under v3's one-record framing:

```
pathological 10-in/2-out, 8,067 B      (v3 framing, 43 B)
  RAW     record    16,223 chars   fits    (16,511 spare)
  CHUNKS  record    37,255 chars   OVER the 32,734 cap by 4,521
```

**The hex was carrying the SEPARATORS, not the data.** Its only job was to keep
LF bytes out of a record, because records are LF-separated.

**RULED (operator, 2026-08-24): follow `md1`/`mk1`.** They already solved this,
and the fork already ships the answer: a chunked constellation format does **not**
ride as one hex-encoded record. **Each chunk is its own bare record** — no
prefix, no hex — and the container's own separator separates them.
`gui/scan.go:91-92` is the shipped precedent: `codex32.ValidMD(string(buf)) ||
codex32.ValidMK(string(buf))` → `return mdmkText(buf), nil`.

| | v3 — one hex record | RULED — bare records |
| --- | --- | --- |
| `form = 0x02` body | the LF-joined `mt1` text, hex-encoded | **empty**, `body_len == 0` (E18) |
| the chunks | inside that body | **sibling records** in the same section |
| what binds them | position | **content: R15** — see below |
| 10/2 pathological | **37,255 chars, 4,521 OVER** | **18,737 chars, 13,997 spare** |

**Recomputed against the 75-byte framing this fold introduces**, not against the
ruling's own worked example, which used v3's 43:

```
metadata record   3 + 2x75                      =     153 chars
202 bare chunks   202 x 91 + 201 separators     =  18,583 chars
one separator between the metadata and the set  =       1
                                          total =  18,737   (13,997 spare)
```

**§2.1's "one record, not siblings" REOPENS, and the binding gets STRONGER.**
The reason that sentence existed was *"so the legend stays bound to what it
describes"* — a positional worry. The bare-record framing replaces position with
**content**: the metadata record carries the txid, every chunk carries
`chunk_set_id` = the **top 20 bits of that same txid**, so the association is
derivable from what the records *say*, not from where they sit. **R15 is
therefore the BINDING MECHANISM, not merely a cross-check** — spec §3.6b and
spec §5 are edited to say so, and **E20** is the rule that enforces it.

**SAFE BECAUSE OF A BECH32 PROPERTY, VERIFIED RATHER THAN ASSUMED.** The data
charset is `qpzry9x8gf2tvdw0s3jn54khce6mua7l`, which excludes `1`, `b`, `i` and
`o` — checked, all four absent, 32 distinct symbols. So `1` occurs in an `mt1`
string **only** as the HRP separator, and the three characters `mt1` can only
ever mark a chunk boundary. A sniffer keyed on the prefix cannot be fooled by a
chunk's payload.

**WHAT THE RULING COSTS, named because every ruling in this document names its
cost.**

1. **`me` must classify a bare `mt1` record, and today it REFUSES one.** Measured
   on the current tree, on a chunk string from the pinned corpus:

   ```
   $ me sysw pack 'mt1p9h8jqq9qqqqgqq...' --no-passphrase
   me: record 0 (records count from 0) is not a form this container can place:
       not a BIP-39 mnemonic, not an md1/mk1/ms1 string, and not a `text:`/`pass:`
       record.                                                        (exit 4)
   ```

   `crates/me-cli/src/classify.rs:40-53` matches `md`/`mk`/`ms` and returns
   `UnknownHrp("mt")` for anything else, so `sysw::classify` falls through to
   `Class::Unknown`. **The `ValidMT` branch is W5 of §2.4, and it is the work.**
   Spec §2.2a already ruled the shape of it: *"a new `ValidMT` over the shared GF
   engine, not a call to an existing predicate."*
2. **The record count goes from 1 to 203** for the pathological payload — one
   `tx:` record under v3's framing, one metadata record plus 202 chunks now. `sysw`
   applies no record cap — `MAX_RECORD_LEN = 512` and `MAX_RECORDS = 24` are
   `seal`'s and `sysw::split` applies neither — so nothing refuses it, but
   §2.4's W9 has to summarise a set rather than print 202 lines.
3. **A payload-level rule appears where there was none.** Set membership,
   completeness and orphan detection are not decidable one record at a time, so
   `split` — the only place the whole record list is in hand — gains a pass
   (W10). That is a site, not a sentiment, and V25 is its negative.

### 1.5 (r1-I2) WHAT "REFUSED" MEANS HERE — layer, scope, stream, exit, and what runs first

**"REFUSED" appears in E2–E20 and §2.3 and v2 defined it nowhere**, so step 8's
and step 10's assertions could not be written. Spec §5 is explicit that *"Where a
refusal RUNS is part of the refusal"* and *"For every refusal above, name what
runs BEFORE it"*. Four answers, once, binding **every RECORD-CODEC refusal** in
this plan — and the scope of that phrase is itself a correction, see below:

| question | NORMATIVE answer |
| --- | --- |
| **which layer** | the record codec returns `Err(TxRecordError::…)`; **`me sysw pack` maps it to a refusal**. The codec never prints and never exits |
| **what scope** | **the whole pack aborts and nothing is written** — five records, one malformed `tx:`, zero output. This follows `split`'s existing shape for `Class::Unknown` (`crates/me-cli/src/sysw/mod.rs:255`), **not** `report_unconfirmed`'s warn-and-pack (`crates/me-cli/src/main.rs:1141`) |
| **what the operator sees** | **one line on stderr naming the record's index and the RULE**, nothing on stdout, **exit 4** — `me`'s existing pack-failure path (`crates/me-cli/src/main.rs:975-983`). *"The rule"* is a channel that does not exist today and **§2.4's W11/W8 (r4-C1: W7 is retracted) build it** (r2-C2) |
| **what runs BEFORE it** | for a record already in hand: `classify` (§2.4). **For R2 the answer is clap**, and that is the whole point of the refusal — `mt`'s §8.2f was bypassed because the arg parser ran first *and clap's error echoed the bearer transaction*. **R2's test asserts the record text does not appear in stderr**, not merely that the exit code is non-zero |

**(r2-I2) ONE EXIT CODE FOR "EVERY USE OF THE WORD" WAS WRONG, AND IT BROKE AN
R0-GREEN SPEC RULE.** v3 closed this section with *"binding every use of the word
in this plan"* and then gave that class one code, **exit 4** — while §6.2 puts
R2, R7 and the TTY refusal under the same word. `me` already has a named
vocabulary and v3 never mentioned it (`crates/me-cli/src/main.rs:225-228`):
`EXIT_OK = 0`, `EXIT_USAGE = 2`, `EXIT_REFUSED = 3`, `EXIT_INVALID = 4`.
**NORMATIVE, per refusal:**

| refusal | exit | why that one, and what already does it |
| --- | --- | --- |
| **E1–E20 and §2.3's decode failure** | **4** `EXIT_INVALID` | it arrives through `pack → split → Err` and that path already returns 4 (`crates/me-cli/src/main.rs:983`). "Invalid/integrity" is what `me` calls this (`crates/me-cli/tests/cli.rs:162`) |
| **R7 — empty stdin** | **2** `EXIT_USAGE` | **spec §5 rules it normatively**: *"must join the existing exit-2 path, not bypass it"*, and the path is real — `read_records` fails at `crates/me-cli/src/main.rs:1223-1225` and its caller returns 2 at `crates/me-cli/src/main.rs:903`. An implementer following v3 would have written 4 and broken an R0-GREEN spec rule and a shipped exit code |
| **the TTY refusal (§4.2)** | **2** `EXIT_USAGE` | §4.2 says the new path *"replaces a refusal"* — that same `crates/me-cli/src/main.rs:1223-1225` branch. Two sibling refusals at one site exiting 2 and 4 is not a vocabulary, it is a coin toss |
| **R2 — a `tx:` record on argv** | **3** `EXIT_REFUSED` | a **policy** refusal on bearer material, the same shape as *"refusing to seal seed material … without `--seal-secret`"* at `crates/me-cli/src/main.rs:509-515`, which returns `EXIT_REFUSED`. Nothing is invalid; the operator is being told not to do it this way |

**So this section binds E1–E20 and §2.3 — the record-codec refusals — and the
other three are ruled in the row above rather than inherited.** V13, step 2,
step 7 and step 11 each assert their own code.

**E9's "three distinct messages" therefore means three distinct stderr lines at
exit 4**, each naming a **different rule name** (`magic`, `version`, `form`), and
V13 asserts the text, the rule, the stream and the code.

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
| RAW | deserialise the **body** as a Bitcoin transaction, **and E11: re-serialise it and require the bytes back** |
| CHUNKS | gather the record's set (E20: `chunk_set_id` == the top 20 bits of the carried txid), require it COMPLETE and PRISTINE (E19), **reassemble via `mt_codec::decode`, THEN deserialise the result** as a Bitcoin transaction — every record having satisfied E13 first |

**and in both cases BOTH identifiers must match: the transaction's txid (display
order, witness-stripped, §1.1) MUST equal the carried `txid`, and its wtxid
(display order, §1.1a) MUST equal the carried `wtxid` (E17).**

**The E17 clause is what is load-bearing, and E11 is not (r1-C2, CORRECTED
r2-C1).** v3 said the E11 clause was *"not decoration"* and that it was *"the
only check that fires"* on a witness-stripped body. **Executed, it does not fire
at all** — §1.1a's measurement block, five rows, the fifth of which v3 never
took. The RAW row still admits two different byte strings for the same
transaction — **222 bytes with the witness, 113 without, the same txid, and both
deserialise, and both re-serialise to themselves** — and it is the wtxid
equality, not the re-serialisation equality, that tells them apart.

**The CHUNKS row is now the longer of the two, and that is the ruling's cost.**
v3's CHUNKS decode was *"split the body, reassemble"*; §1.4a's bare records make
the input a **set gathered from the payload**, so the chain gains a gathering
step that can fail three new ways — orphan chunk, missing index, duplicate index
— all of them E20 and all of them vectored by V25. **The gathering is not
`classify`'s**: `classify` sees one record at a time and none of these is
decidable from one record. It is `split`'s, which is W10.

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
> deserialises, both identifiers match, E1–E20 pass, and `picotool save` reaches those
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

**(r2-I6) AND `me` GAINS `mt-codec` — WHICH v3 REQUIRED IN THREE PLACES AND
ADDED IN NO STEP.** This is r1-I5's defect one dependency over: §2.2's CHUNKS row
requires it, §5's whole publish ceremony exists to make it available, §6 closes
on *"`me` depends on the pinned published version"*, and `grep -n 'Cargo.toml'`
over v3 returned exactly **one** manifest edit — the `bitcoin` one. Verified:
`me-cli` declares no `mt-codec` today (its deps are `md-codec`, `mk-codec`,
`ms-codec`, `clap`, `zeroize`, `serde`, `serde_json`, `aes-gcm`, `pbkdf2`,
`sha2`, `bip39`, `rand`, `rpassword`).

| decision | value | why |
| --- | --- | --- |
| crate | `mt-codec` | the API P1 needs is `decode_chunk` (per-record classify and the grouping key), `decode` (set reassembly) and `HRP` |
| version | **`= "0.1.0"`**, the pinned published version | §5 publishes it once this plan is GREEN, which is before step 1 runs. No path dependency, no git dependency |
| **added in** | **§4, step 4** — the SAME manifest edit as `bitcoin` | step 4's test is *"V1, V2 and V3's metadata record, round-tripped at the codec"* (**r4-C2** moved V3's whole-payload round-trip to step 10) and **V3 is the CHUNKS vector**, so the first step that needs it is the step that must add it |
| **first load-bearing at** | **§4, step 6** | `classify`'s `ValidMT` branch (W5) is `decode_chunk`; the set gathering (W10) is `decode`. Step 4 exercises the codec, step 6 exercises the classifier |

**The three functions, read at source rather than from a doc comment:**
`decode_chunk(s: &str, plan: Option<Chunking>) -> Result<DecodedChunk>` yields a
`ChunkHeader { chunk_set_id, count, index }` **from a single ungrouped record**
(`mnemonic-transaction/crates/mt-codec/src/string_layer/pipeline.rs:160`, header at `mnemonic-transaction/crates/mt-codec/src/string_layer/header.rs:26`), which is
exactly what E20's grouping needs and is the same accessor shape EPD §6.3 already
requires for `md1`/`mk1`; `decode(strings: &[String]) -> Result<DecodedSet>`
returns `DecodedSet { bytes, … }`, the reassembled transaction bytes
(`mnemonic-transaction/crates/mt-codec/src/string_layer/pipeline.rs:148`); and `DecodedChunk::corrected` is the BCH
correction count E19 requires to be zero (`mnemonic-transaction/crates/mt-codec/src/string_layer/pipeline.rs:93`).

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

> **(r2-C2) v3 NAMED FIVE SITES AND THEY WERE CORRECT AND INSUFFICIENT — the
> same shape, one layer in.** §1.5 rules that a refusal is *"one line on stderr
> naming the record's index and the rule"*; E9 requires *"each with its own
> message"*; V13 asserts the text. **The channel that carries a rule identity
> does not exist, and none of v3's five sites creates it.** `classify` returns a
> bare `Class` with no `Result` and no payload (`crates/me-cli/src/sysw/mod.rs:124`);
> `unknown_reason` re-derives the reason **from the record string alone**
> (`crates/me-cli/src/sysw/mod.rs:107-115`), the classification already discarded; `UnknownReason` has
> exactly two variants (`crates/me-cli/src/sysw/mod.rs:96-105`) and `sysw_error` renders exactly two
> messages (`crates/me-cli/src/main.rs:1263-1280`). So W5 as v3 wrote it yields **one** message
> for every `tx:` failure and it is the wrong one — a record whose body is
> perfect lowercase hex but whose magic is `MTX2` would be told *"its body is
> **not lowercase hex** … `printf '%s' 'your text here' | xxd -p -c 256`"*: a
> false statement about the record, plus an instruction that would corrupt it.
> **The table below names fourteen sites.** §2.4's own words applied to itself.

**NORMATIVE — FOURTEEN sites, and none of them is the codec.** **(r4-C1) W6 and
W7 are RETRACTED and W11–W13 replace them.** v5 answered round 3's Criticals by
adding §2.5a and left this table prescribing the edit §2.5 proves wrong — so an
implementer following §4 step 6 built the retracted version, and §6's closure
table passed them on it. **A replacement that does not retract is an
alternative, and the reader executes whichever they reach first:**

| # | site | change |
| --- | --- | --- |
| W1 | `crates/me-cli/src/sysw/record.rs:27-28` | add `pub const TX_PREFIX: &str = "tx:";` beside `TEXT_PREFIX` and `PASS_PREFIX` |
| W2 | `crates/me-cli/src/sysw/record.rs:31-40` | add **`Transaction`** AND **`MtChunk`** to `enum Class` (today eight variants, neither of them this). Two variants because they are two record shapes with two decode paths — a `tx:` framed record and a bare `mt1` chunk (§1.4a) |
| W3 | `crates/me-cli/src/sysw/record.rs:50-55` | **NEITHER `Class::Transaction` NOR `Class::MtChunk` is in `is_secret`'s `matches!`** — spec §2.1's *"no new secrecy class"*, and round 0's M3 |
| W4 | `crates/me-cli/src/sysw/mod.rs:124` | `classify` gains a `TX_PREFIX` branch: hex-decode, then §2.2's DECODE, then `Class::Transaction`; a failure is §1.5's refusal, **never** `Class::FreeText` |
| **W5** | `crates/me-cli/src/sysw/mod.rs:124` | **`classify` gains a `ValidMT` branch** for a **bare `mt1`** record — §1.4a's ruling. Spec §2.2a already ruled its shape: *"a new `ValidMT`, not a call to an existing predicate."* In Rust it is `mt_codec::decode_chunk(r, None)` plus E13 and E19; success is `Class::MtChunk`. **Measured: without it `me sysw pack` refuses a corpus chunk today at exit 4** (§1.4a) |
| ~~W6~~ | ~~`crates/me-cli/src/sysw/mod.rs:107-115`~~ | **RETRACTED (r4-C1). DO NOT add `TX_PREFIX` to `unknown_reason`'s loop.** That edit makes §6's own **W8** assertion RED: an `MTX2` record whose body IS valid lowercase hex matches the prefix and reports `NonHexBody("tx:")`, whose message (`crates/me-cli/src/main.rs:1263-1272`) tells the operator to re-hex their text — **a false statement plus a corrupting instruction.** §2.5 states this outcome and v5 left this row prescribing it anyway. **Superseded by W11**, which decides the reason from the PARSE RESULT, not the string |
| ~~W7~~ | ~~`crates/me-cli/src/sysw/mod.rs:96-105`~~ | **RETRACTED (r3-C1, r4-C1). `UnknownReason::TxRule(&'static str)` has NO PRODUCER** — `unknown_reason` receives only the record string and cannot know which rule failed. Declaring the variant satisfies a compile-time existence check and **nothing has to produce it**, so §6's W7 row was a gate that could not fail. **Superseded by W11–W13.** The `&'static str` reasoning survives in W11: the enum doc at `crates/me-cli/src/sysw/mod.rs:89-95` rules it *"Carries NO operator data, and that is load-bearing"*, because a `pass:` body is a passphrase |
| **W8** | `crates/me-cli/src/main.rs:1263-1280` | **`sysw_error` gains the PER-RECORD arm for W11's `TxRecordError`** — *"record {i} (records count from 0) is a `tx:` record that fails rule {rule}"* — and E9's three rule names (`magic`, `version`, `form`) become its three distinct lines. **(r4-C1) NOT a `TxRule` arm: that variant is W7's and W7 is RETRACTED.** The rule name reaches here from the parse (W11), which is the only place that knows it |
| **W9** | `crates/me-cli/src/main.rs:1156-1181` | **`print_mdmk_confirmation` gains a `tx:` / `mt1` arm** (r2-I5). Today its second statement is `if classify(r) != Class::MdMk { continue; }`, so a `tx:` record produces **no line at all** and §6's *"`me sysw show` reads it back"* names a capability `show` does not have. It prints one line per `tx:` record (**form, carried txid AND carried wtxid** — r3-I4) and **one line per chunk SET, not per chunk** — 202 chunks must not become 202 lines. **The wtxid is not decoration here:** §1.1a buys the field partly for *"a second value to compare against `mt inspect`"*, and V26 is the case that needs it — a stripped body whose txid EQUALS the honest record's, so an operator comparing the txid alone gets a **match** on a payload with its signatures removed. Print the value that separates them |
| **W10** | `crates/me-cli/src/sysw/mod.rs:251-259` | **`split` gains a payload-level pass for E20** — set membership, completeness, orphans. `classify` sees one record at a time and **none of E20 is decidable from one record**; `split` is the only place the whole list is in hand. This is the site §1.4a's ruling creates |
| **W11** | `crates/me-cli/src/sysw/record.rs` (new) | **`TxRecordError`, produced BY THE PARSE.** §2.5a. The `tx:` parse returns `Result<TxRecord, TxRecordError>`, and the error names the rule that failed (`magic`, `version`, `form`, `body_len`, …) because **the parse is the only place that knows**. Rule names are `&'static str` literals from this crate — never operator data. **This is what W6/W7 could not do** |
| **W12** | `crates/me-cli/src/sysw/mod.rs:96-105` | **`SyswError` gains a SET-LEVEL variant.** It may NOT carry a bare `usize`: E20's failures are *"chunk 7 of set `0x2dcf2` is missing"* and *"record 12 is an orphan"*, R17's is *"two sets share top-20 bits"* — **none of which is one index**, so §1.5's *"index and the rule"* is unsatisfiable for them. Produced by W10 |
| **W13** | `crates/me-cli/src/main.rs:1256-1300` | **the SET-LEVEL printer arm (W12's variant), plus `sysw_error`'s OUTER match** — W8 is the per-record half, this is the rest. Round 3 found this required by W10/R17 and named by no row; **round 4 found §4 and §6 still pointing at W6/W7 instead** |
| **W14** | `crates/me-cli/src/sysw/coverage.rs` and `crates/me-cli/src/sysw/vectors.rs` | **(r4-I3) the vector fixture's two build-failing assertions.** `every_required_vector_exists` (`crates/me-cli/src/sysw/vectors.rs:147`) and `assert_every_named_test_is_placed` (`crates/me-cli/src/sysw/coverage.rs:230`) **fail the build** when a required vector is absent or a named test is unplaced, so P1's 29 vectors cannot land in `testdata/sysw_vectors.json` (§3.3) without extending both. **No version of this plan before round 4 mentioned this file at all** |

**W3 is the one that costs the operator if it is wrong.** `is_secret` is a
`matches!` over three variants in a file whose doc comment argues both ways about
a borderline class (*"`Class::FreeText` is deliberately NOT secret even though an
operator may put anything in it"*). Get it wrong and **every transaction payload
seals**: a 12-word passphrase to store, those 12 words typed on the device's
on-screen keyboard, and ~31 s of KDF — to protect a payload whose purpose is to
become a plate anyone can read. That is the exact cost spec §2.4 exists to
remove, and it is an operator ruling in spec §8. **W2 adding a second variant
doubles the surface**: `Class::MtChunk` must be non-secret for the same reason,
and step 6 asserts both.

**W4 and W5 are where spec §2.1a's lesson lands in Rust — twice.** The spec's C3
was *adding a prefix without adding a branch*; W1 without W4 is that defect in
`me` rather than in `gui/scan.go`, and **§1.4a's ruling creates a second instance
of it in the same function**. **The branch is the work; the prefix is not.**

**These fourteen sites are split across THREE steps, not one (r4-C1, r4-C2):**
**step 4** builds W14, **step 6** builds W1–W5 and W8–W9, **step 7** builds
W11–W13, and **step 10** builds W10. **Step 6's test is END TO END** — `me sysw
pack` on a real `tx:` record, read back with `me sysw show`, which **W9 is what
makes possible**. **Most** of V1–V27 are record-level vectors: they
exercise the codec directly and stay green with `classify` untouched, so **no
vector in §3 can substitute for that test.** **(r3-M3) FIVE are not, and siting
their tests at the codec would site them where their input never arrives:** V20
(uppercase `mt1`), V23 (padded) and V24 (BCH-repaired) are **bare chunk records
with no `tx:` framing**, refusable only by `classify`'s W5 branch; V25 and V27 are
**whole payloads**, decidable only in `split` (W10). The conclusion stands — step 6
is still irreplaceable — but it rests on the other **twenty-one**.

> **(r4-I5) BOTH operands of v6's arithmetic were wrong, and this is the SEVENTH
> enumeration in this cycle to be wrong on first count.** The table has **29 rows**,
> not 27 — `V4a`, `V4b` and `V17b` are separate vectors — and **eight** are not
> record-level, not five. v6 named V20, V23, V24, V25, V27 and missed three that
> the plan's own cells declare: **V3** (*"the whole payload is the vector"*),
> **V4b** (R15's positive case, which needs the chunks present) and **V15** (whose
> perturbation is applied to *the chunks' embedded* `set_id`). 29 − 8 = **21**.
> Re-derived mechanically, not read.

§6 states the per-site assertion for each of the **fourteen**, because a `grep -c`
cannot see a site and cannot assert an absence (r2-I4).

## 2.5 THE ERROR PATH — r2-C2 was named, not plumbed (r3-C1, r3-C2)

**Round 2 asked for the sixth wiring site. v4 answered with ten sites, and all
ten resolve — but naming where the code changes is not the same as saying how a
failure TRAVELS from where it is detected to where it is printed.** Round 3's two
Criticals are one defect: **there is no channel.** Measured, not read:

```rust
// crates/me-cli/src/sysw/mod.rs:107-115 -- SOLE producer on the production path
fn unknown_reason(record: &str) -> UnknownReason {      // <- only the STRING
    for prefix in [record::PASS_PREFIX, record::TEXT_PREFIX] {
        if record.starts_with(prefix) { return UnknownReason::NonHexBody(prefix); }
    }
    UnknownReason::Unrecognised
}
// crates/me-cli/src/sysw/mod.rs:255
Class::Unknown => return Err(SyswError::Unclassifiable(i, unknown_reason(&r))),
```

**Three mismatches between that channel and the rules E1–E20 need:**

| the channel is | the new rules are | consequence |
| --- | --- | --- |
| **string-only** — `unknown_reason` never sees a parse result | **parse-result-shaped** — E9 must say *which* of magic/version/form failed | `UnknownReason::TxRule` **can never be produced** (r3-C1) |
| **prefix-shaped** | E13/E19 apply to **bare `mt1`** records, which match no prefix | they reach `Unrecognised`, whose message is **false** for V20/V23/V24 (r3-C2) |
| **per-record, carrying an index** — `Unclassifiable(usize, …)` | E20 and R17 are **set-level** | a *missing* chunk has **no index**; §1.5's "index and the rule" is unsatisfiable |

**And W6's prescribed edit is actively wrong.** Adding `TX_PREFIX` to that loop
makes an `MTX2` record with valid hex report *"its body is not lowercase hex …
`xxd -p -c 256`"* — advice for a defect it does not have, and **RED against §6's
own W8 assertion.**

### 2.5a NORMATIVE — what P1 builds instead

**A rule failure must carry the rule.** The parse returns its own error type; the
container error carries it; the printer names it. Three additions, replacing W6's
edit:

| | |
| --- | --- |
| **W11** | a `TxRecordError` enum whose variants are the rules — one per E-number that can fail a **single record**. Produced by the parse, where the failure is known. |
| **W12** | `SyswError` gains a **set-level** variant. It may not carry a bare `usize`: E20's failures are *"chunk 7 of set 0x2dcf2 is missing"* and *"record 12 is an orphan"*, and R17's is *"two sets share top-20 bits"* — none of which is one index. |
| **W13** | the printer arm for both, at `sysw_error` **and its outer match** (`crates/me-cli/src/main.rs:1257-1300`), which round 3 found required by W10/R17 and named by no row. |

**`unknown_reason` is NOT the place.** It is reached only after `classify` has
already returned `Unknown`, by which point the reason is gone. **The parse must
fail with its reason, not be re-interrogated afterwards** — that is the shape
that produced r3-C1.

**The near-miss, and it has a test:** a `tx:` record whose body genuinely is not
lowercase hex must **still** report `NonHexBody`. W11 must not swallow the case
the existing channel gets right.

### 2.5b E13's PRECEDENT IS FALSE — measured, and the truth is worse (r3-I2)

E13 (no padding / whitespace) cited `seal`'s `validate_record` as precedent.
**Executed, and the citation does not hold:**

```rust
// crates/me-cli/src/seal/record.rs:118
pub fn validate_record(s: &str) -> Result<RecordKind, RecordError> {
    let s = s.trim();                                   // <- TRIMS FIRST
    if let Some((pos, ch)) = first_noncanonical(s) {    // <- then checks
```

Padding never reaches the canonicality check. **And the container does not merely
tolerate it — it preserves it.** Run against the shipped binary:

```
me sysw pack --no-passphrase "<md1 string> "     -> exit 0
the packed record's last byte                    -> b' '   (the space, verbatim)
```

**So a record carrying trailing whitespace is packed into the public section with
the whitespace intact** — which is the hazard EPD §6.4 states in its own words:
records engrave **verbatim**, so a character outside the BCH checksum's coverage
*"turns a scratch on the operator's only copy into silently-absorbed damage
rather than a detected error."*

**NORMATIVE for P1:** E13 stands on its own reasoning, **not** on a precedent
that does not exist. The false citation is struck.

**AND THIS IS A LIVE `me` DEFECT WIDER THAN P1**, on the `md1`/`mk1` path, which
P1 neither introduces nor is scoped to fix. **Filed, not folded** — see
`FOLLOWUPS.md` F-245. Naming it here so a reader does not mistake E13's
correctness for the neighbouring path being safe.



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
`mnemonic-transaction/crates/mt-codec/src/test_vectors/mt1_v1.json`, WITH ONE
NAMED EXCEPTION — V7 (r2-I3).** 222 bytes, segwit (`raw_hex` begins
`02000000 0001 01`), `txid_is_wtxid: false`, six `mt1` chunks of 37 B, and these
values, recomputed for this fold and written out in the vector file (§6):

```
txid  (display)  2dcf2b973d52044b1e58c988a5a59d388073ff05598b0a1e93eeb04c72ebf630
wtxid            d5717c031917116bbd4fcaff0bcc3abe9d456899991414f2177a5281ed836f51
chunk_set_id     0x2dcf2      == top 20 bits of the DISPLAY txid
                 0x30f6e      == top 20 bits of the INTERNAL one -- the C1 defect
```

**It also satisfies §3.2 for free:** that corpus was generated by
`scripts/gen-mt1-vectors.py` **in this repo**, not by the crate it judges —
verified by reading the corpus's own `generator` field, which is that path.

> **(r2-I3) THE EXCEPTION, AND WHY THE SENTENCE COULD NOT BIND V7.** v3 scoped
> this rule to *"every vector below"*. Round 1's I7, which it was written for,
> was about the **txid axis** — V1 and V2 built from a convenient small legacy
> transaction pass in a world that refuses every honest segwit record. Scoped to
> every vector it also binds **V7, where the axis is SIZE**, and there the
> constraint is impossible: a 16 KB body cannot be built from a 222-byte
> transaction, padding is unavailable because E11 and E17 both require the body
> to be a real transaction's real serialisation, and the corpus's other vector is
> 284 bytes. §6's *"the vectors were not produced by the code they judge —
> satisfied by construction, since §3's transaction comes from that corpus"* then
> leaned on the same sentence and did not cover V7 either.
>
> **NORMATIVE: V7's transaction is a SECOND, SIZE-ONLY vector**, and it is
> generated by **`scripts/gen-tx-record-vectors.py` — new in this repo, committed
> in §4 step 9**, which builds a synthetic 1-in/N-out transaction and serialises
> it independently of `bitcoin` and of `me`. It satisfies §3.2 the same way
> `gen-mt1-vectors.py` does: **a different implementation of the format, in a
> different language.** Byte-exact sizing is available because an `OP_RETURN`
> output's `scriptPubKey` length is a free parameter, so the body can be tuned to
> any target length to the byte — which is what makes *"16,290 − F exactly, and
> 16,291 − F refused"* a constructible near-miss pair rather than an aspiration.
> **The transaction is not a valid spend and does not need to be**: §2.2 requires
> only that it deserialise, re-serialise identically, and carry identifiers that
> match — and §2.2's own ACCEPTED COST block already states that a well-formed
> transaction is all this gate can ever require.
>
> **V7 and V27 are the only vectors this exception covers (r3-I3).** Every other
> vector below — including V18 and V26, which are the `even` vector's own bytes
> with the witnesses cleared — is the corpus transaction.
>
> **V27's exception is FORCED, and v4's scoping sentence made its vector
> UNBUILDABLE.** V27 needs two records whose txids **share their top 20 bits
> without being equal**, and that is two DIFFERENT transactions. The corpus holds
> two and they do not collide: `even`'s `set_id` is **`0x2dcf2`** and `uneven`'s is
> **`0x3b426`**, both read from `mt1_v1.json` while writing this fold, each equal to
> the top 20 bits of its own display-order txid. Built the only way v4's rule
> allowed — the same transaction twice — the two records carry **IDENTICAL** txids,
> which is **R10**; the payload is refused with R17's comparison deleted, so **V27
> would have had no RED test.** That is r2-M4's failure mode exactly.
>
> **So V27 GRINDS a second transaction** whose display-order txid begins `2dcf2` —
> locktime is the free knob, and `mt`'s own help puts a 20-bit grind at *"under a
> second"*. §4 step 9's generator produces it and writes it into the vector file
> beside the corpus pair, so the collision is a **committed input**, not a
> re-grind at test time.

| # | vector | pins |
| --- | --- | --- |
| V1 | **RAW (segwit)**, no optional fields | the fixed layout; `n_fields = 0` |
| V2 | **RAW (segwit)**, all three optional fields | E1's ascending tag order as an *instance*, `u16 BE` lengths, u64 fee. **Not E1's negative — that is V16 (r1-I1)** |
| V3 | **CHUNKS: a metadata record with `body_len = 0` PLUS its six BARE `mt1` records** | **(§1.4a)** `form = 0x02`; E18's empty body; E12's single `\n` between records and absent trailing LF; E13's lowercase ASCII; E20's complete set (`count = 6`, indices 0..5). **The whole payload is the vector**, not one record — that is what the ruling changed |
| **V4a** | **RAW**, with txid AND wtxid written out | **§1.1's display order AND txid-not-wtxid, on the RAW path.** Segwit is required: for a legacy transaction txid == wtxid and the vector passes in both worlds |
| **V4b** | **CHUNKS**, with txid AND wtxid AND `chunk_set_id` written out | the same facts on the chunks path, plus **R15's positive case — which is now the BINDING, not a cross-check** (§1.4a). **(r1-I7)** v2 had one V4 and never said which form it was |
| V5 | absent optional field | absence is omission (E7) |
| V6 | unknown tag | REFUSED (E8) |
| V7 | **RAW body at exactly `16,290 − F` bytes, `F` = the fields present; near-miss `16,291 − F` REFUSED** | the **record framing** ceiling under the 75-byte framing — see §3.1. **The only vector not built on the corpus transaction (r2-I3)**; its source is `scripts/gen-tx-record-vectors.py` |
| V8 | RAW whose carried txid ≠ the body's | the §2.2 consistency refusal |
| **V9** | **duplicate tag** | E2 |
| **V10** | **trailing bytes after the body** | E3/E4 |
| **V11** | **`body_len` larger than the bytes remaining** | E5, before allocation |
| **V12** | **zero-length TLV value** | E6 |
| **V13** | **bad magic / unknown version / `form = 0x03`** | E9, three distinct stderr lines at exit 4, **each naming a DIFFERENT rule name** (`magic`, `version`, `form`) — §1.5, and W11/W8 are what make the rule name expressible (r2-C2) |
| **V14** | **`n_fields` ≠ the TLVs present** | E10 |
| **V15** | **R15 NEGATIVE: a chunks record whose carried txid's top 20 bits ≠ its chunks' `chunk_set_id`.** **NORMATIVE, because it decides whether the vector can go RED (r2-M4): the perturbation is applied to the CHUNKS' EMBEDDED `set_id`, leaving the carried txid HONEST** | **(I13)** delete R15's comparison and every *other* vector stays green. Perturb the **txid** instead and §2.2's full txid equality refuses the record on its own, so V15 would stay green with R15 deleted and R15 would have no RED test — v3's row named the txid first and left the choice to the builder. **(M1) Its v2 justification was false and is retracted:** V15 constructs a deliberate mismatch, which is still a mismatch under the wrong byte order, so the refusal fires either way. **V4a/V4b catch C1; V15 catches a deleted R15.** Both are required |
| **V16** | **TLVs in DESCENDING tag order** → REFUSED | **(r1-I1)** E1's negative. Without it, a decoder that *accepts* `0x03, 0x02, 0x01` conforms to every vector, so the Go port — which §3 says is judged **against the vectors** — admits a record `me` refuses. E1's own divergence, one layer out |
| **V17** | **`tag=0x02, len=2`; and the near-misses `len=7` and `len=9` refused, `len=8` passes** | **(I3, r1-I11)** E16's FEE half. E6 refuses only `len = 0`; `1..7` is the gap, and the fee is engraved |
| **V17b** | **`tag=0x03, len=3` and `len=5` REFUSED; `len=4` passes** | **(r2-I7)** E16's FINGERPRINT half, which v3 never vectored at any width. Delete the `0x03 → 4` clause and every v3 vector stayed green — while a 3-byte value for the master fingerprint reached the plate's `FROM` line |
| **V18** | **RAW body = the same transaction serialized WITHOUT witness (113 B), carried txid correct, carried wtxid the REAL transaction's** | **(r1-C2, RE-ASSIGNED r2-C1)** **E17, not E11.** This is the vector that separates the two conforming records: it deserialises, its txid matches, **and E11 passes on it** — measured, §1.1a. The wtxid is what refuses it, because the stripped body's wtxid equals its own txid |
| **V19** | **`form = 0x02` with a NON-EMPTY body** (and its sibling: `form = 0x01` with `body_len = 0`) | **(§1.4a)** E18. v3's V19 was *"CHUNKS body with a trailing `\n`"*; the CHUNKS body is empty now, so that case moved to the container and is covered by **§4 step 12's new `sysw` test** (r3-I1: the `joins_with_lf_and_no_trailing_lf` v4 named here is `seal`'s and does not reach `sysw`'s join). **The rule did not go away and neither did the vector** — it is re-cut against the framing that replaced it, and it is also spec §5's R4′ made checkable |
| **V20** | **a bare `mt1` RECORD in UPPERCASE** | **(§1.4a)** E13's case half — `mt_codec::decode_chunk` lowercases before it verifies, so it accepts what the record layer must not |
| **V21** | **`tag=0x01` whose value is `74 6f ff 21`** | **(r1-I10)** E14. Rust refuses by default, Go accepts by default |
| **V22** | **`tag=0x01, len=65`; near-miss `len=64` must PASS** | E15, and the near-miss rule (§6) |
| **V23** | **a bare `mt1` record with a TRAILING SPACE** (and one with a leading space) | **(r2-I7)** E13's WHITESPACE half, which v3 never vectored — delete it and every v3 vector stayed green, while `to_symbols`'s `s.trim()` swallowed the padding and `decode_chunk` returned Ok. A padded record is a different record and a different EPD §6.6 public-data hash |
| **V24** | **a bare `mt1` record with ONE symbol corrupted — BCH-correctable, `corrected == 1`; near-miss: the pristine string passes** | **(§1.4a)** E19. `decode_chunk` REPAIRS it and reports success, so without E19 `me sysw pack` launders a damaged chunk into the payload. The near-miss pair is what shows E19 refuses damage and not `mt1` |
| **V25** | **E20's three negatives on one payload: (a) an ORPHAN `mt1` record whose `chunk_set_id` matches no `tx:` record; (b) a set MISSING index 3 of 6; (c) a set with index 3 present TWICE.** Near-miss: the complete set of V3 passes | **(§1.4a)** E20, the rule the ruling creates. All three are read off the chunk headers without reassembling anything, and **none is decidable one record at a time** — which is why W10 exists |
| **V26** | **the SAME 113 witness-free bytes as V18, with the carried wtxid recomputed honestly (== its txid) — MUST PASS** | **(r2-C1, r2-I1)** two things at once. It is E17's **nearest legitimate input**, and V18/V26 differ in exactly 32 bytes. And it is the **witness-free transaction class** v3 had no vector for at all — every ordinary P2PKH/P2SH spend — which §1.4's old marker+flag enumeration would have refused |
| **V27** | **two CHUNKS `tx:` records whose txids share their top 20 bits** → REFUSED | **(§1.4a)** the collision R15-as-binding creates: 20 bits is *"1 in 1,048,576 by accident, and under a second to construct deliberately"* (`mt`'s own help), and two sets sharing a `chunk_set_id` are unassignable. **spec §5's R17**, added by this ruling. **NORMATIVE (r4-I1), because it decides whether the vector can go RED — the same clause V15 carries:** the bare refusal is NOT the assertion. Two colliding sets make **every chunk match both `set_id`s**, so no chunk belongs to *exactly one* set and **E20 refuses the payload on its own**; delete R17's comparison and V27 would stay green, which is r3-I3's failure with a different mask. **V27 asserts R17's RULE NAME on stderr**, and goes RED when E20's set-completeness message appears in its place |

### 3.1 (I7) V7's number, and a THIRD ceiling nobody had

v1 said *"body at `MaxSectionLen` boundary"*. **Unconstructible.** A record is
`tx:` plus hex, so:

```
32,734 section chars − 3 for "tx:"  = 32,731 hex chars
                                     -> 16,365 whole bytes (one hex char SPARE)
                       − 75 framing  = 16,290 bytes of body, minus the fields
```

**THE FRAMING IS 75 BYTES, NOT v3's 43 — the `wtxid` field (§1.1a, r2-C1).**
`4 + 1 + 1 + 32 + 32 + 1 + 4`. Every figure in this section is recomputed against
it, and the body ceiling moves **16,322 → 16,290**.

**(M4) The middle line is a FLOOR, not an equality.** 16,365 bytes is 32,730
characters; the odd 32,731st cannot start a byte and is simply unusable. An odd
run of hex is refused before it is decoded (§1, `unhex_lower`), so nothing turns
on it — but the derivation said `=` where it meant `->`, and a Go porter
reproducing the arithmetic would have hunted for the missing character.

**So there are THREE ceilings and the spec states only two:**

| ceiling | value | where | binds |
| --- | --- | --- | --- |
| container section | 16,367 B | spec §2.3 **as written** | nothing — it is the uncorrected figure |
| **record framing** | **16,290 B of BODY** | **this plan — new** | **the RAW form ONLY** |
| engraveable | 14,560 B | spec §4.1a, at 0.60 mm | the **raw** form only (QR plates) |

**(§1.4a) THE RECORD-FRAMING CEILING NO LONGER BINDS THE CHUNKS FORM, AND THAT
IS THE WHOLE POINT OF THE RULING.** In v3 it bound both, and that is what put the
pathological chunks payload 4,521 characters over. With the chunks riding as
bare records the metadata record is a **fixed 153 characters** (`3 + 2×75`, plus
its legend fields), and what bounds the chunk set is the **section cap directly**
— 32,734 characters, against which 202 chunks cost 18,583. One ceiling was
removed from the path; none was raised.

**(M7) THE 16,290 FIGURE ASSUMES THE `tx:` RECORD IS ALONE IN THE SECTION.**
Records are joined with `\n` and no trailing LF
(`crates/me-cli/src/sysw/mod.rs:260`, asserted by **§4 step 12**, not by
`joins_with_lf_and_no_trailing_lf` — that one is `seal`'s, r3-I1), spec §3.6 contemplates several transactions
in one payload, and a payload may also carry `md1`/`mk1` records. For `k` records
the bound is `Σ(record chars) + (k − 1) ≤ 32,734`, so **V7 is explicitly the
single-record vector** and the plan claims nothing about a full payload.
(`MAX_RECORD_LEN = 512` and `MAX_RECORDS = 24` are `seal`'s; `sysw::split`
applies neither — **which is what makes §1.4a's 203-record payload legal at
all**, and it is stated here because a reader who assumed `seal`'s caps applied
would conclude the ruling was unimplementable.)

**THREE SPEC NUMBERS ARE FALSIFIED BY §1'S FRAMING, NOT ONE.** Round 0's I7 named
two and v2's §6 owned only one (r1-I7); the third is the framing arithmetic
applied to **spec §2.3's** own table. **All three are recomputed AGAIN for this
fold**, because the `wtxid` field moved the framing and §1.4a moved the chunks
form. The arithmetic is `3 + 2 × (75 + N)` for a RAW record and
`153 + 1 + chunk_chars` for a chunks transaction:

| spec §2.3 says | recomputed | verdict |
| --- | --- | --- |
| *"a **16,367-byte** raw transaction"* | **16,290 B of body**, minus the fields | **wrong by the framing** — 77 B plus the legend |
| *"5/2 … ✅ (**raw-only at 8191, by 31 chars**)"* | raw record `3 + 2×(75 + 4,080) = 8,313`; the chunks form in the container `153 + 1 + 9,383 = 9,537` | **false in BOTH halves — the raw record is 122 chars OVER 8191, so at the old cap NEITHER form fitted.** The 31 counted 8,160 hex chars against the old cap and ignored both `tx:` and the framing. At the raised cap of 32,734 both forms fit, so the parenthetical is **struck** rather than re-derived |
| *"10/2 … 18,583 … ✅"* (chunks) | `153 + 1 + 18,583 = 18,737`, **13,997 spare** | **the ✅ is CORRECT and the number is INCOMPLETE.** 18,583 is the chunk text alone; the container also carries the metadata record and one separator. The column needs the container figure beside it — that is the correction §6 owns |

> **THE THIRD ROW IS WHY §1.4a EXISTS, and the history has to stay readable in
> exactly one place, which is this one.** Under **v3's** framing — one
> hex-encoded record holding the LF-joined chunk text — that row computed to
> `3 + 2×(43 + 18,583) = **37,255**` characters, **4,521 OVER** the 32,734 cap,
> while the *same transaction* fitted as raw bytes at 16,223 with 16,511 to
> spare. v3 flagged it as *"a design question, not a number"* and deferred it to
> *"whoever reopens spec §2.1b"*. **The operator reopened it the same day
> (§1.4a): the chunks ride as bare records, the hex disappears, and the row fits
> with 13,997 spare.** The 37,255 figure was correct under the framing that
> produced it and is retained here as the REASON for the ruling — it is not a
> live ceiling and nothing below may cite it as one.

The full recomputation, for the correction §6 owns — **every figure produced by
running the arithmetic, none transcribed**:

```
in/out  rawB   RAW record   chunk chars   CHUNKS in container   fits 32,734
 1/1     852      1,857        2,001        153+1+ 2,001 =  2,155      yes
 1/2     893      1,939        2,092        153+1+ 2,092 =  2,246      yes
 2/2   1,692      3,537        3,955        153+1+ 3,955 =  4,109      yes
 5/2   4,080      8,313        9,383        153+1+ 9,383 =  9,537      yes
10/2   8,067     16,287       18,583        153+1+18,583 = 18,737      yes
                                                      spare at 10/2 = 13,997
```

**Both forms of every measured spend now fit, and the raw form still has more
room** — 16,447 spare at 10/2 against the chunks form's 13,997 — so §2.2's XOR
stays a real choice rather than a forced one.

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

### 3.3 (r4-I3) WHERE THE TWENTY-NINE VECTORS PHYSICALLY LIVE — and the refresh command that would void §3.2

**v6 discussed the vectors as pins and never as a file.** Three sites said *"the
vector file"* and none named it; §4 assigned the construction of V1–V6 and
V8–V26 to no step. **The repo already ships exactly this artifact, and no version
of this plan mentioned it** — `grep` for `vectors.rs`, `coverage.rs`, `testdata`
and `sysw_vectors` over v6 returned **zero hits each**:

```
crates/me-cli/testdata/sysw_vectors.json      the fixture
crates/me-cli/src/sysw/vectors.rs:25          pub const PATH = "testdata/sysw_vectors.json"
crates/me-cli/src/sysw/vectors.rs:147         fn every_required_vector_exists()   <- FAILS THE BUILD
crates/me-cli/src/sysw/coverage.rs:230        fn assert_every_named_test_is_placed() <- FAILS THE BUILD
```

**NORMATIVE — P1's vectors extend `crates/me-cli/testdata/sysw_vectors.json`.**
They are `sysw` container vectors and that file is the `sysw` container's vector
home; `vectors.rs`'s module doc is §3's own sentence in different words — *"the Go
implementation reads the SAME file … a vector both sides are missing is a defect
neither will ever notice"*. A new file would be invisible to the cross-language
check, which is the one thing these vectors exist to feed. **This makes
`coverage.rs` a FOURTEENTH wiring site (W14)**, because its `COVERAGE` table
fails the build when a required vector is absent.

> **REFUSAL — the documented refresh command MUST NOT be run for P1's vectors.**
> `crates/me-cli/src/sysw/vectors.rs:10-14` ships
> `cargo test -p mnemonic-engrave --lib sysw::vectors::regenerate -- --ignored --nocapture`,
> which **rewrites the fixture from today's code** — precisely what §3.2 forbids.
> An implementer who files P1's vectors in the obvious place and runs the
> documented refresh has voided §3.2 **while every test still passes**, and §6's
> closure bullet would report satisfied, because its stated evidence is only that
> the *transaction* comes from `gen-mt1-vectors.py`. The corpus supplies the
> transaction; **every byte of the record framing — magic, version, form, the two
> identifiers, `n_fields`, the TLVs, `body_len` — is assembled by something else**,
> and the bullet is true no matter what that something was. Same false-PASS shape
> as the `grep -c` struck under r2-I4.

**§4 step 4 constructs V1–V6 and V8–V26 and commits them**, by the same
independent route §3.2 requires — `scripts/gen-tx-record-vectors.py`, which step 9
already commits, extended to emit the framing rather than only V7 and V27.

## 4. TDD ORDER

Each step: failing test first, watched fail **for the stated reason**, minimal
code, full suite green.

| step | test first | then |
| --- | --- | --- |
| 1 | **`sysw::wire::MAX_SECTION_LEN` is 32,734** — and `seal::wire::MAX_SECTION_LEN` is **still 8191** | raise **only** `crates/me-cli/src/sysw/wire.rs:42` |
| 2 | `me sysw pack` reads from **stdin**; **empty stdin refused at EXIT 2** (R7, spec §5); **a TTY with no `--in` and no argv refuses at EXIT 2 instead of blocking**; **`--in` still wins over argv, and stdin is read only when neither is given** (§4.2). **Both exit codes are asserted, not just non-zero** (r2-I2) | the stdin path |
| 3 | **a payload with NO `Class::is_secret()` record packs UNSEALED; one with any packs SEALED** (spec §2.4, the base rule); an explicit passphrase-mode flag **wins**; **`--allow-weak` is not one** (§4.3); **stderr says which way and why, every time** | content-based sealing |
| 4 | **V1, V2 and V3's METADATA RECORD, round-tripped AT THE CODEC — and (r4-I3) this is the step that CONSTRUCTS AND COMMITS V1–V6 and V8–V26 into `crates/me-cli/testdata/sysw_vectors.json` (§3.3), extending W14's two build-failing assertions** — `encode`/`decode` on the 75-byte framing, no `me sysw pack` in the loop. **(r4-C2) V3's WHOLE-PAYLOAD round-trip is step 10's, not this step's**: V3 is *"a metadata record plus six bare chunks"*, and `me sysw pack` REFUSES a bare `mt1` record until W5 lands at step 6 — measured, `exit 4`, *"record 0 … is not a form this container can place"*, the same refusal §1.4a's cost 1 already quotes. A gate that cannot go green is a gate that cannot fail | the layout codec — **and this is the step that adds BOTH manifest lines to `crates/me-cli/Cargo.toml`: `bitcoin = { version = "0.32", default-features = false, features = ["std"] }` (I5) and **`mt-codec = "=0.1.0"`** — **the `=` is required (r3-M1): a bare `"0.1.0"` is `^0.1.0`, and a `0.1.1` publish could change `decode_chunk`'s tolerance under `me` with no manifest edit** — the pinned published version (r2-I6, §2.2)** |
| 5 | **V4a, V18 and V26, plus V4b's RECORD half** — the txid/wtxid/`chunk_set_id` a CHUNKS metadata record carries, read straight off the framing. **(r4-C2) V4b's R15-POSITIVE half is step 10's**: R15 binds the carried txid's top 20 bits to *the chunks'* `chunk_set_id`, and gathering the set is W10's, six steps later — §2.2 says so itself (*"is not `classify`'s … It is `split`'s, which is W10"*) | the two identifier fields: txid (display order, witness-stripped) **and wtxid** (display order, canonical serialization), on **both** forms. **V18/V26 belong here and not in step 8**, because they are the pair that proves the wtxid field does work the txid cannot (§1.1a) |
| **6** | **`Class::Transaction` AND `Class::MtChunk` exist and `is_secret()` is FALSE for BOTH; `me sysw pack` on a real `tx:` record puts it in the PUBLIC section; a bare `mt1` chunk classifies instead of being refused — **and (r4-I5) V20, V23 and V24 are asserted HERE, not at step 8**: uppercase refused, padded refused, BCH-repaired refused, each against its pristine near-miss; `me sysw show` reads BOTH back (one line per `tx:` record, ONE line per chunk SET); a `tx:` record with a bad body names its PREFIX, not `Unrecognised`; a `tx:` record with a bad MAGIC names the MAGIC RULE, not "not lowercase hex"** | **(r1-C3, r2-C2, r2-I5) the wiring — W1–W5 and W8–W9 of §2.4. NOT W6/W7, which are RETRACTED (r4-C1)**; the last two clauses are W11's and W13's, built in step 7. End to end, because no record-level vector can see this. **The last clause is the r2-C2 test**: with v4's retracted W6 it reports a false statement plus a corrupting instruction, which is why that row is struck |
| **7** | **a `tx:` record that fails DECODE aborts the whole pack: nothing on stdout, one stderr line naming the index and the rule, exit 4 — with four other valid records present. AND (r4-C1) a record with magic `MTX2` and a VALID lowercase-hex body names the MAGIC rule — the string *"not lowercase hex"* MUST NOT appear** | **§2.3's refusal, at the layer and scope §1.5 rules — and THE ERROR PATH: W11, W12 and W13 of §2.4 (§2.5a).** The anti-smuggling gate, tested as a `me sysw pack` outcome rather than as a codec `Err`. **The second clause is the one v4's W6 could not pass** |
| 8 | V5–V6, V9–V14, **V16–V17b, V19, V21, V22** — **(r4-I5) V20, V23 and V24 are NOT here: they are BARE `mt1` records with no `tx:` framing, and the layout codec never sees one.** They move to **step 6**, where W5's `classify` branch is what refuses them; sited here they would pass vacuously against a codec whose input is a framed record | every rule in **E1–E19 EXCEPT E12, E17 and E20** — **(r4-M1)** E12's RED test is **step 12's**, E17's is **step 5's** (*"V18 and V26 belong here and not in step 8"* — step 5's own words), and E20 is **step 10's**, because it is a payload-level rule and its vectors are payloads |
| 9 | V7 at **16,290 − F**, and the near-miss `16,291 − F` REFUSED | the framing ceiling, single-record (§3.1) — **and this is the step that writes and commits `scripts/gen-tx-record-vectors.py`** (r2-I3), because V7 has no other input |
| 10 | V8, **V15**, **V25**, **V27** — **and (r4-C2) the two halves steps 4 and 5 cannot hold: V3's WHOLE-PAYLOAD round-trip and V4b's R15-positive binding.** Both need `split`'s set pass, which is this step | the identifier-consistency refusals on both forms, **and E20's set binding** (§1.4a). **This is the first step at which a payload containing bare `mt1` records can be packed and read back at all**, because W5 (step 6) and W10 (here) are both in place |
| 11 | a `tx:` record on **argv** refused (R2) **at EXIT 3**, **and its text does not appear in stderr** | the argv guard — §1.5's *what runs before it*: clap must not echo the record. **Exit 3, not 4** (r2-I2): it is a policy refusal, the shape of `crates/me-cli/src/main.rs:509-515` |
| **12** | **(r3-I1) E12's RED test, in `sysw`** — pack a CHUNKS `tx:` set, read the public section back as bytes, and assert it is the records joined by a single `0x0A` with **no trailing LF and no empty record**. It goes RED when `crates/me-cli/src/sysw/mod.rs:260`'s separator changes | E12, which until this step is a rule the plan states and nothing in `sysw` asserts. **Not** `seal`'s `joins_with_lf_and_no_trailing_lf`, which tests a different container |

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
Today `--in` wins over argv silently and filters **empty** lines
(`crates/me-cli/src/main.rs:1211-1227`). v1 and v2 both added stdin without saying
what `me sysw pack rec1 --in f.txt < g.txt` does.

**NORMATIVE: `--in` > argv > stdin.** Stdin is read **only** when neither `--in`
nor argv records are given — which is exactly the branch that returns
`no records: pass them on argv or with --in` today
(`crates/me-cli/src/main.rs:1223-1225`), so the new path *replaces a refusal*
rather than pre-empting a working input. **Stdin filters EMPTY lines exactly as
`--in` does** — `read_records` (`crates/me-cli/src/main.rs:1217-1221`) tests
`!l.is_empty()`, so **a line holding a single space SURVIVES as a record** (r3-M2),
becomes no class, and lands in §1.5's refusal path via W11 — so a record's index is its position among the non-empty lines in
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

**AND `me` DECLARES IT IN §4 STEP 4 (r2-I6).** v3 named the publish in three
places and added the dependency in no step, so the first step that needed
`mt-codec` was the step that did not have it. The manifest line is
**`mt-codec = "=0.1.0"`** — the exact-pin operator, not the caret a bare `"0.1.0"`
would mean (r3-M1) — and §2.2's second decision table owns it. **The publish
therefore happens BEFORE step 1 runs**, not between steps — this plan going GREEN
is the gate, and step 4 is only where the manifest line is written.

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
- **V1–V27 pass**, and **V4a/V4b write out txid, wtxid and `chunk_set_id`
  explicitly** in the vector file, not only in code.
- **Every rule E1–E20 has a test that goes RED without its check — EXCEPT E7 and
  E11, which §1.3 names, explains and assigns.** Anything else without a RED test
  is a defect, not an exception. And **so does every refusal P1 adds** — §6.2. A
  rule with no negative test is a comment; a completeness claim that is false is
  worse than a narrower one that is true.
- **`ClassTransaction` and `ClassMtChunk` are WIRED, not merely specified**
  (§2.4). **(r2-I4) THE `grep -c` THAT USED TO CLOSE THIS IS STRUCK — it was a
  false PASS**, and the plan already names the shape twice (*"a close condition
  that could pass on the defect it exists to catch"*). `grep -c
  'Class::Transaction' crates/me-cli/src/sysw/` returns **per-file** counts over
  nine files; **(r3-I6, re-derived r4) W1–W3 and W11 live in `record.rs`; W4, W5,
   W10 and W12 live in `mod.rs`; W8, W9 and W13 live in `main.rs`; W14 lives in
   `coverage.rs`/`vectors.rs` — and that grep's path (`crates/me-cli/src/sysw/`)
   sees NEITHER `main.rs` NOR the two W14 files**, so it sees **two files, not
   fourteen sites**. **W6 and W7 are retracted (r4-C1) and are not sites at all.**
   Several sites carry no such token: W1 is a `const`, W11 is an error type, and
   **W3 requires the token to be ABSENT**, so a non-zero count there is the defect
   rather than the proof. **An absence is not assertable by any non-zero grep.** Each site
  gets a test instead, and each test can fail:

  | site | the assertion that closes it, and it is a TEST |
  | --- | --- |
  | W1 | `sysw::record::TX_PREFIX == "tx:"` |
  | W2 | `Class::Transaction` and `Class::MtChunk` both construct (compile-time), and `Class` has ten variants |
  | W3 | `assert!(!Class::Transaction.is_secret())` **and** `assert!(!Class::MtChunk.is_secret())`, plus step 3's *a transaction-only payload packs UNSEALED* |
  | W4 | `classify(<a valid tx: record>) == Class::Transaction` |
  | W5 | `classify(<a corpus mt1 chunk>) == Class::MtChunk` — **measured RED on the current tree** (§1.4a) |
  | W14 | **(r4-I3)** `every_required_vector_exists` and `assert_every_named_test_is_placed` both pass with P1's 29 vectors present in `testdata/sysw_vectors.json` — and the `regenerate` command was NOT run (§3.3) |
  | **W11** | **(r4-C1, replacing v4's W6/W7 rows)** a `tx:` record with magic `MTX2` and a **valid lowercase-hex body** is refused naming the **magic** rule, and the string *"not lowercase hex"* does **not** appear. **This is the assertion v4's W6 row made unsatisfiable** — and unlike *"the variant exists and is `Copy`"* it cannot pass by declaration |
  | **W12** | a payload with chunk 7 of set `0x2dcf2` missing is refused naming **the set and the missing index** — not a bare record index, which E20 does not have |
  | **W13** | both of the above reach stderr through `sysw_error` **and its outer match**; neither prints a bare `Unclassifiable` |
  | W8 | a `tx:` record with magic `MTX2` produces a line naming the **magic** rule, and **the string "not lowercase hex" does NOT appear in it** |
  | W9 | `me sysw show` on a packed `tx:` payload prints the carried txid **and the carried wtxid** (r3-I4), and on a 202-chunk payload prints **one** set line. **(r4-I2) the assertion is NOT *"V18 and V26 differ in 32 positions"* — v6 wrote that and it CANNOT RUN, because V18 is a vector E17 REFUSES, so `show` never emits a line for it.** The runnable assertion, which delivers the same operator benefit: on the honest 222-byte corpus record the printed **txid and wtxid DIFFER**; on **V26** — which the plan requires to PASS — they are **EQUAL**, because a witness-free body's wtxid is its own txid. That equality is the signal that the payload carries no signatures, and it is visible in `show`'s output or it is visible nowhere |
  | W10 | V25's three negatives all refuse; V3's complete payload packs |
- **(r2-I5) `me sysw show` CAN ACTUALLY DO THIS, BECAUSE W9 MAKES IT SO.** v3's
  read-back gate named a capability `show` does not have: `show`
  (`crates/me-cli/src/main.rs:1045-1088`) prints `sealed:`, `pub_len:`, `ct_len:`,
  `identity:`, the digest, and then `print_mdmk_confirmation`, whose second
  statement is `if classify(r) != Class::MdMk { continue; }` — so a `tx:` record
  produces **no line at all**, before or after the wiring. There is no
  `--records` flag and no other subcommand that lists them. **The gate is kept
  and the capability is built**; weakening it to `pub_len`/digest is exactly the
  substitutability §2.4 says no vector may have.
- **A `tx:` record that fails DECODE aborts the pack** with nothing on stdout and
  exit 4 (§1.5), asserted with four other valid records present. **The other
  three refusals P1 adds exit 2, 2 and 3** — §1.5's table, asserted per refusal
  rather than as "non-zero" (r2-I2).
- `cargo nextest run --locked` green; `cargo clippy --all-targets` clean.
- **The vectors were not produced by the code they judge** (§3.2) — satisfied by
  construction for V1–V6 and V8–V27, since §3's transaction comes from a corpus
  generated by `scripts/gen-mt1-vectors.py`, **and for V7 by
  `scripts/gen-tx-record-vectors.py`, which §4 step 9 writes and commits**
  (r2-I3). v3 claimed *"by construction"* over all of them while V7's input could
  not come from that corpus and no other source was named.
- **`mt-codec`'s dead `bitcoin` declaration is removed (M8), `cargo publish
  --dry-run` is clean, `mt-codec` is published**, and `me` depends on the
  **pinned published version**.
- **Spec corrections this phase owns — TWO still open (r3-I5). v4 said THREE and
  correction 3 LANDED IN THE SAME FOLD**, in spec §2.3's new container column;
  the spec's own amendment block already enumerates the remaining **two**, and v4's
  plan-side list was never reconciled with it. All three are
  RE-DERIVED against the 75-byte framing and §1.4a's bare records.** All three are
  **spec §2.3's**, all three are recomputed in §3.1, and the *"opaque"* item v2
  listed is **struck: it has no referent** (r1-I8, §2):
  1. *"a **16,367-byte** raw transaction"* → **16,290 B of body**, minus the fields.
  2. the 5/2 row's *"raw-only at 8191, by 31 chars"* → **struck**: the raw record
     is **8,313 chars, 122 OVER 8191**, so at the old cap *neither* form fitted.
     At 32,734 both do.
  3. ~~the 10/2 chunks row~~ — **DONE, this fold's predecessor.** Verified in the
     artifact rather than assumed: `design/SPEC_engrave_transaction.md` now carries
     the container column and the 10/2 row reads **18,737** with **13,997 spare**.
     Items 1 and 2 are what the phase still owes, and the spec's amendment block
     names exactly those two.

  **These three MOVED between v3 and v4**, and that is itself the argument for
  computing them: correction 2's number changed and its verdict widened,
  correction 3 inverted from *"false — 4,521 OVER"* to *"true but incomplete"*
  because §1.4a changed the framing underneath it. A correction transcribed from
  the previous fold would now be wrong.
- **The SPEC EDITS this fold already made are spec §2.1, spec §2.3, spec §3.6b
  and spec §5**, and
  they are the ruling's — **and spec §2.3 carried correction 3 with them**, which is
  why the list above is now two. **Items 1 and 2** are still owed by the phase.
- **(r4-I4) §6.3's FIVE STALE SPEC STATEMENTS are corrected before P1 closes.**
  Distinct from the two numeric corrections above: these are prose falsified by
  §1.4a's ruling — spec §3.6's *"R15 validates it only within a single record"*
  (R15 is now the BINDING across a metadata record and up to 202 siblings), spec
  §6's **P1** row, spec §2.1b's R4′ dependency row, spec §6's **P3** row, and spec
  §1's ownership table. **§6.3 enumerates them; this bullet is what burns them
  down.** Left ungated, P1 closes GREEN while spec §3.6 still tells P4's picker
  that R15 is a within-record check.
- **The near-miss rule**: every guard here is tested against its nearest
  *legitimate* input as well as the hostile one — **(M3) v2 said "seven instances
  this cycle" against spec §5's six and named no seventh.** The count is dropped;
  the rule is not. P1's own near-miss pairs are named where they live: V22
  (`len = 65` refused / `len = 64` passes), V17 (`len = 7` and `len = 9` refused /
  `len = 8` passes), **V17b (`len = 3` and `len = 5` refused / `len = 4` passes,
  r2-I7)**, **V18/V26 (a stripped body with the real wtxid refused / the SAME 113
  bytes with an honest wtxid passes — the sharpest pair here, and the one that
  makes §1.1a's accepted cost visible)**, V19 (a non-empty CHUNKS body refused /
  `body_len = 0` passes), V20 (uppercase refused / lowercase passes), **V23 (a
  padded chunk refused / the trimmed one passes, r2-I7)**, **V24 (a BCH-repaired
  chunk refused / the pristine one passes)**, **V25 (an incomplete set refused /
  V3's complete set passes)**, V10 (trailing bytes refused / exact end passes),
  **V7 (`16,291 − F` refused / `16,290 − F` passes)**, and §4.2's TTY refusal
  (a TTY refused / a pipe passes).

### 6.1 (r1-I6) THE GATES THAT ACTUALLY READ THIS DOCUMENT — and the one that does not

**v2's first bullet was *"the build gate has run on it"*, and that is a
STRUCTURAL FALSE PASS.** Round 1 executed it and got:

```
$ ./scripts/plan-build-gate.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
test result: ok. 77 passed; 0 failed          <- ROUND 1's RUN. This output
   PASS: the CLI tests compile                   NO LONGER REPRODUCES; see
   clippy clean                                  the note below.
EXIT=0
```

> **(r2-M3) THAT OUTPUT IS HISTORICAL, AND THE HOLE IT DEMONSTRATES IS ALREADY
> CLOSED.** `c8f8557` — *"scripts: plan-build-gate must FAIL when it extracts
> nothing (R0 P1 round 1, I6)"* — made the script refuse on an empty extraction.
> **Re-run while writing this fold, the same command now prints *"Refusing rather
> than reporting a pass on an empty extraction"* at `EXIT=3`.** The round-1 quote
> is correctly attributed and the conclusion below is unaffected, but a reader
> who ran the command to check this section's premise got a different answer than
> the section showed. The structural false PASS is a FIXED defect; **the
> remaining reason this gate does not apply to P1 is the anchor filter alone**,
> which the NORMATIVE paragraph below states.

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
code is written test-first in §4, not transcribed from a plan. **Since `c8f8557`
the script also refuses rather than reporting a pass**, so citing it here would
now turn the gate red for a non-defect — which §6.1's own last paragraph rules
against.

**THREE gates DO read this document, and P1 runs all three** — before a
reviewer, and again before committing any fold, because **a fold is authorship
and re-earns the gate**. Each row states what a PASS looks like on *this*
document, measured on this fold, so a future run has something to diff against:

| gate | what it reads here | PASS on this document | what it does NOT cover |
| --- | --- | --- | --- |
| `./scripts/plan-cite-check.sh` | every `path:line` citation, resolved against the real tree | **90 of 107 resolve; the 17 dangling are exactly the 8 into the vendored `bitcoin` crate and the 9 into `mnemonic-transaction`** — see below. Any eighteenth is a defect. **(r3 AND r4 folds) This gate has now caught THREE bare-path citations in EACH of three successive folds — nine in total** — two `sysw/mod.rs` citations and one `main.rs` citation, written in the REPORT's shorthand (no `crates/me-cli/src/` prefix) and unresolvable as written. **They are not reproduced here with their line numbers, because doing so mints fresh dangling citations — measured, twice.** The r4 fold repeated the defect a third time in §3.3 and the gate caught it again in seconds. **No reading has ever caught this class; the gate has caught it every time.** The six hand-checks that fold ran did not include this gate. **That is the argument for the row, and it is why the number moved from 90 to 98 rather than staying put** | **interpretation** — it proves the line exists, never that this plan reads it right; and it cannot check absence claims |
| `./scripts/plan-table-check.sh` | every table row against its header's cell count | **137 rows checked, 0 malformed, exit 0** | cell **content**; a right-width row with wrong values passes |
| `./scripts/plan-fold-sweep.sh <doc> --terms <the twenty-two below>` | **terms this fold removed that survive elsewhere** | **exactly 27 hits, one per term, ALL of them inside the block below — the self-reference. A twenty-eighth hit anywhere else is a real finding.** **(r3 fold)** Its ten new terms were swept BEFORE they were written down and **nine were absent**; the tenth (the `five sites` entry) survives twice in prose and **both are HISTORICAL** — *"none of v3's five sites"*, *"three of v3's five sites"* — where a present-tense survivor would be a finding. **The terms live ONLY in the block: quoting them in this cell instead made all ten self-hit from a table, measured one edit ago** | it flags candidates, not defects; and terms nobody named |

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
and the fold author is the only one who knows what was superseded.** These **twenty-seven**,
each of which this fold removed and each of which must survive **only** in this
block:

```
'INTERNAL byte order'          the v2 layout-table row          (r1-C1)
'§1.3 says'                    the dangling body citation       (r1-C2)
'V1–V15 pass'                  the superseded vector range
'Every rule E1–E10 has'        the superseded rule range
'16,367-byte raw transaction'  the uncorrected spec figure
'Seven instances'              the unexplained near-miss count  (M3)
'E11 is the only check'        v3's false claim about E11       (r2-C1)
'16,322 B of BODY'             the 43-byte framing ceiling      (r2-C1)
'E1–E16'                       the superseded rule range        (§1.3)
'NORMATIVE — five sites'       the superseded wiring header     (r2-C2)
'the serialized signed transaction'
                               the undecodable RAW body row     (r2-I1)
'strings, LF-separated'        the superseded chunks body       (§1.4a)
'THREE, not one'               spec corrections owed            (r3-I5)
'These six'                    the terms list's own count       (r3-I6)
'not five sites'               the ten-site wiring count        (r3-I6)
'filters blank lines'          they are EMPTY, not blank        (r3-M2)
'non-blank lines'              they are EMPTY, not blank        (r3-M2)
'V7 is the only vector'        V27 needs the exception too      (r3-I3)
'mt-codec = "0.1.0"'           a caret requirement, not a pin   (r3-M1)
'asserted by `joins_with_lf'   that test is `seal`'s            (r3-I1)
'all three are still'          correction 3 had already landed  (r3-I5)
'FOUR `mnemonic-transaction`'  measured NINE                    (r3-I6)
'W1–W10'                       step 6 is not all the sites      (r4-C1)
'TEN sites'                    fourteen, and two retracted      (r4-C1)
'V1–V3 round-trip'             step 4 cannot pack a payload     (r4-C2)
'other twenty-two'             29 rows, 8 exceptions, so 21     (r4-I5)
'W7/W8'                        W7 is RETRACTED                  (r4-C1)
```

**The last FIVE are r4's and the ten before them are r3's**, each a fact its fold
RETRACTED. A hit for any of the twenty-seven **outside the block above** is a real finding — that is the
whole mechanism, and r1-C1 is why: a fact corrected in the prose and left
standing in a table three sections away is invisible to a diff, because by
construction it lives in the text the diff did not touch.

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
`bitcoin` crate at all. **Those are the 17 dangling: 9 into
`mnemonic-transaction`** (`pipeline.rs` ×6 — §1.1's ruling, its sole consumer,
E13's decoder tolerance, `DecodedSet`, `decode_chunk` and `DecodedChunk::corrected`;
`header.rs` ×2 for E20's inputs; `lib.rs` ×1 for §3.2) **and 8 into `bitcoin`**
(`transaction.rs` ×7 and `encode.rs` ×1, the API facts §1.1a, §1.4 and §2.2 rest
on). **The count rose from v3's 8 because this fold added facts, not because it
added slack** — every one of the 17 is a *real* file:line in a repo the gate has
no root for, and each is **verified by command instead**. Run these; they are the
P1 equivalent of the build gate for the facts the gate cannot reach:

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

  ## §1.4a's bech32 property: the data charset excludes 1, b, i and o, so `mt1`
  ## can only ever mark a chunk boundary -- prints four Falses, then 32
python3 -c "cs='qpzry9x8gf2tvdw0s3jn54khce6mua7l'; print([c in cs for c in '1bio'], len(set(cs)))"

  ## §1.4a's cost 1: `me` REFUSES a bare mt1 record today -- exit 4, Unrecognised
cargo run --quiet --bin me -- sysw pack --no-passphrase \
  'mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax' \
  >/dev/null

  ## §1.4a's cost 1, at source: classify() matches md/mk/ms and nothing else
sed -n '40,53p' crates/me-cli/src/classify.rs

  ## E19's precedent: `me` already refuses a non-pristine mk1 record
sed -n '134,144p' crates/me-cli/src/seal/record.rs

  ## E20's inputs: mt1's header is set_id(20) | count-1(15) | index(15)
sed -n '1,20p;26,40p' $MT/crates/mt-codec/src/string_layer/header.rs

  ## §2.2's mt-codec API surface: decode_chunk, decode, DecodedChunk::corrected
sed -n '85,100p;148,160p;230,236p' $MT/crates/mt-codec/src/string_layer/pipeline.rs

  ## §1.1a's wtxid API facts: compute_wtxid hashes consensus_encode, and Wtxid is
  ## declared in the SAME hash_newtype! block as Txid -- so DISPLAY_BACKWARD too
sed -n '44,55p;800,808p' $B/src/blockdata/transaction.rs

  ## §3.1's whole table, recomputed rather than transcribed
python3 -c "
import math
CAP=32734; FR=75; meta=3+2*FR
cc=lambda b: math.ceil((b*8+55)/5)+16
for name,rawB,n,bpc in [('1/1',852,22,39),('1/2',893,23,39),('2/2',1692,43,40),('5/2',4080,102,40),('10/2',8067,202,40)]:
    ch=n*cc(bpc)+(n-1); print(name, 3+2*(FR+rawB), ch, meta+1+ch, CAP-(meta+1+ch))
print('body ceiling', (CAP-3)//2-FR, 'metadata record', meta)"
```

**And §1.1a's and §1.4's measurements are a scratch crate, reproducible in four
minutes.** `bitcoin = { version = "0.32", default-features = false, features =
["std"] }` plus `hex`, a `main` that deserialises the corpus `raw_hex`, prints
`compute_txid()` and `compute_wtxid()`, compares `serialize(&tx)` to the body,
then clears every `TxIn::witness` and, on the stripped body, prints: its length,
whether it still deserialises, **whether `serialize(deserialize(stripped)) ==
stripped`** — that is E11, and it is `true` — whether its txid is unchanged,
**whether its wtxid now equals its own txid**, and whether its wtxid still
matches the carried one. **Run while writing this fold, and it built clean at
those exact features** — which is how §2.2's dependency row is a measurement
rather than a guess.

> **(r2-C1) THE FIFTH AND SIXTH ROWS ARE THE POINT, AND v3's CRATE DID NOT PRINT
> THEM.** v3's measurement block had four rows and every one was accurate —
> round 2 reproduced all four. The row it did not contain is the one that decided
> the claim: it measured `serialize(&tx) == body` for the **with-witness** body
> only and never measured re-serialisation equality on the **stripped** body.
> **Four minutes of the crate this very paragraph described would have found it.**
> The lesson is not *measure more*; it is that a rule's measurement must be taken
> on **the input the rule is supposed to refuse** — which §1.3 now makes a
> standing requirement for every rule added here.

**Two further things this crate settles, both run, both load-bearing in §1.3:**
a segwit-flagged body with all-empty witnesses is refused by the decoder
(*"witness flag set but no witnesses present"*), which is r2-I1's other half; and
**E11 has nothing left to catch in Rust** — a non-minimal `VarInt` input count is
refused `non-minimal varint` on the segwit body *and* on the legacy one, and a
trailing byte is refused `parse failed: data not consumed entirely`.

**Every command above was executed while writing this fold and its output is what
§1.1, §1.4, §2.2, §3 and §5 state.** That is the standard the build gate would
have enforced if it could read this document, applied by hand because it cannot.

**Adding `/scratch/code/shibboleth/mnemonic-transaction` to that `ROOTS` list is
the one-line change that would bring the **NINE** `mnemonic-transaction` citations
inside the gate**, leaving **eight**. The `bitcoin` **eight** cannot be gated at all: they
point into a registry cache whose path carries a version hash, and a gate that
resolved them would be asserting a *local build artifact*, not a source of truth.
The change is named rather than made, because this plan does not edit tooling.

### 6.2 (r1-I4) REFUSAL COVERAGE IS EVERY REFUSAL P1 ADDS, NOT JUST E1–E20

v1's closure list said *"every refusal added has a test that goes RED without its
check"*. **v2 narrowed it to *"every rule E1–E10"*** — the record-codec rules —
which silently dropped **four of this phase's own refusals**, including the
anti-smuggling gate the whole plan is built around:

| refusal | where | exit (§1.5) | covered by E1–E20? |
| --- | --- | :-: | --- |
| **R2** — a `tx:` record on argv | §4 step 11 | **3** | **no** |
| **R7** — empty stdin | §4 step 2 | **2** | **no** |
| **the TTY refusal** | §4.2, NORMATIVE | **2** | **no** |
| **§2.3's decode-failure refusal** | §4 step 7 | **4** | **no** — and it is the reason EPD §6.3 exists |
| **R17** — two chunk SETS sharing top-20 bits | §4 step 10, **V27** | **4** | **no** (r3-I3) — it has no E-number, so §6's E1–E20 sweep never reached it. This table is the mechanism that does |

**NORMATIVE: all four get a test that goes RED when its check is removed**, the
same standard as E1–E20, verified by deleting the check by hand and watching the
test fail **for the stated reason** — **and each at the exit code §1.5's table
gives it, asserted rather than "non-zero" (r2-I2).**

**And the tooling citation v1 carried is corrected rather than restored.**
`check-refusal-coverage.sh` and `mutate-refusals.sh` are **`mt`'s and live in
`mnemonic-transaction/scripts/`**; **`refusals.toml` is `mt`'s too but lives at
`mnemonic-transaction/crates/mt-cli/tests/refusals.toml`** — **(r2-M2)** v3 put
all three in `scripts/`, and the slip was in the one sentence whose whole job was
to correct a path. `ls scripts/` in *this* repo has none of the three, and spec §5 says so in its own words (*"`mt` has this machinery … the
fork side needs its equivalent"*). So P1 owns the **per-refusal RED test**, done
by hand; spec §6's **P6** row owns building this repo's equivalent of the
**bijection sweep**. Naming P1 as the owner of the sweep would move a gate onto a
phase that has no tool for it.

## 6.3 SPEC STATEMENTS THIS RULING LEFT STALE — FIVE, not three (r3-I7)

The v4 fold named three and was right that they were out of its scope. **Round 3
found five, and one of the three was right for the wrong reason.** All verified
at source:

| # | where | why stale |
| --- | --- | --- |
| 1 | spec §3.6 | *"R15 validates it only within a single record"* — now one metadata record **plus** its bare chunks |
| 2 | spec §6's **P1** row | describes the framed record without the **wtxid** or the chunk class |
| 3 | spec §2.1b | **not** the wtxid omission the fold gave as the reason — its **R4′ dependency row is now outright false**, because the XOR is no longer expressible inside one record |
| **4** | spec §6's **P3** row | says *"the `tx:` branch in `gui/scan.go`"* — **singular**. The ruling needs a **second** branch (`ValidMT`, bare `mt1`), which this plan's own §7 already assigns to P3 |
| **5** | spec §1's ownership table | `me` owns *"`ClassTransaction`; a stdin path; content-based sealing"* — no chunk class, no wtxid |

**P1 owns correcting all five**, and **(r4-I4) §6's closure list did NOT carry
them — v5 appended this section and never edited §6 twenty lines above, so the
sentence described an edit the fold intended and did not make.** `grep -n '§6.3'`
over v6 returned **one hit: this heading**. The bullet now exists in §6, and the
two lists are deliberately distinct: §6's *"TWO still open"* are the **numeric**
corrections to spec §2.3; **these five are PROSE**, in spec §3.6, §2.1b, §6's P1
row, §6's P3 row and §1's ownership table. **Both misses
are the same shape**: a row that describes P1/P3's work in a *sentence* rather
than a *list*, so widening the work did not visibly widen the row.

> **This is the sixth enumeration this cycle to be wrong on first count** — three
> lockstep sites that were four, five wiring sites that were ten, ten that needed
> thirteen, three stale statements that were five. **An enumeration in this
> document is a hypothesis until something re-derives it.**

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

**AND FIVE THINGS §1.4a's RULING TOUCHES OUTSIDE THIS PLAN.** Each is named with
its owner, because a ruling whose consequences are unowned is a ruling that
half-lands:

| consequence | owner | what has to happen |
| --- | --- | --- |
| **`gui/scan.go` must route a bare `mt1` record** | **P3** | `codex32.New` fails on `mt1` (different BCH target), `ValidMD`/`ValidMK` fail on the HRP, `btcaddr.DecodeAddress` fails — so a bare `mt1` reaches `errScanUnknownFormat` at `gui/scan.go:97`. It needs the Go half of W5, beside the `mdmkText` branch at `:91-92` it is modelled on |
| **`SPEC_systemwide_payloads` §3.3.2's admission table gains columns** | **P4** | the table is `program × class` and there are now two more classes. Engrave Transaction is a new row; every existing row refuses both by absence, which is that section's own stated mechanism |
| **`SPEC_systemwide_payloads` §5.3's "the public section admits `ClassMDMK` only"** | **P3/P4** | it is a statement about what the container admits, and `mt1` chunk records widen it. **P1 does not edit it** — the sentence is about the *device*'s admission, and P1 is the host codec |
| **`SPEC_systemwide_payloads` §5.3.2's card-set DECODE check is a FLAG, not a refusal** | already consistent | that rule is the **device at load time**; §2.3's refusal is **`me sysw pack` at creation time**. Refuse when writing, flag when reading, is not a contradiction — but it is close enough to one that it is written down here |
| **`me sysw show` on a 202-chunk payload** | **P1, W9** | in scope, and named here only because it is the one place the record count is visible to a human |
