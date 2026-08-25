# IMPLEMENTATION PLAN — P1: `me`'s transaction container

**Status:** DRAFT, pre-R0. Written 2026-08-24 after `SPEC_engrave_transaction.md`
reached **R0 GREEN**. **No code until this plan is 0C/0I** — the risk-set rule
gates plan-docs as well as specs, and P1's riskiest content is a **new normative
wire format** that Go must later match byte-for-byte.

**Scope: §6's P1 row and nothing else.** `ClassTransaction`, the framed record
including the mandatory carried txid, stdin, content-based sealing,
`MaxSectionLen` → 32,734 — with vectors.

---

## 0. Why this plan exists at all

§6's P1 row is a **scope statement**, not a plan. And §2.1b makes P1's *first*
obligation a design act: **define the `tx:` record's wire layout**, which nothing
has done. Four sections read that layout (§3.4's asserted column, R4′, R15, and
P1's own vectors), and the Go port must reproduce it exactly. **A format defined
in code first is a format nobody reviewed.**

---

## 1. THE `tx:` RECORD LAYOUT — normative, Rust-primary

A `tx:` record is `tx:` followed by **lowercase hex** (the reserved-prefix rule,
`sysw/record.go:41-51`). The hex decodes to:

```
off  size  field       notes
 0    4    magic       "MTX1" = 4D 54 58 31
 4    1    version     0x01
 5    1    form        0x01 = RAW (transaction bytes) | 0x02 = CHUNKS (mt1 strings)
 6   32    txid        INTERNAL byte order -- see §1.1, this is where bugs live
38    1    n_fields    count of legend TLVs, 0..=255
39    ..   fields      n_fields x { tag:u8, len:u16 BE, value:len bytes }
 ..   4    body_len    u32 BE
 ..   N    body        body_len bytes
```

**All multi-byte integers are BIG-ENDIAN.** Stated once, here, because a wire
format with mixed endianness is a defect generator.

**The magic is not redundant with the `tx:` prefix.** The prefix says how the
record is framed; the magic says the *body* is this format. A hex-valid body of
the wrong shape is caught at byte 0 instead of somewhere deeper.

### 1.1 THE TXID'S BYTE ORDER IS THE MOST LIKELY DEFECT IN THIS PLAN

Bitcoin displays a txid **reversed** from the internal `double-SHA256` output.
Every implementation that has ever touched txids has had this bug.

**NORMATIVE: the `txid` field is the raw `double-SHA256` result, INTERNAL byte
order, unreversed.** Display reverses it; the wire does not.

**And it interacts with §3.6b.** That section rules `chunk_set_id` **is the top
20 bits of the txid**. *Top 20 bits of which order?* The two answers differ
completely, and picking wrong makes R15 refuse every honest record — or, worse,
accept every dishonest one.

**This plan does not guess.** `mt-codec` already computes `chunk_set_id`, so the
answer is whatever `mt-codec` does, and **a vector must pin it** (§3, V4). If
`mt-codec`'s own choice turns out to be the display order, this field follows it
— consistency with the sibling beats consistency with a convention.

### 1.2 Legend field tags

| tag | field | value |
| --- | --- | --- |
| `0x01` | `TO` label | UTF-8, operator's own words (§3.4, asserted) |
| `0x02` | fee | **u64 satoshis**, big-endian, 8 bytes |
| `0x03` | `FROM` wallet | 4 bytes, the fingerprint |

**An absent optional field is simply not present in the TLV list.** There is no
"empty" encoding and no sentinel — §2.1b asked what absence looks like, and the
answer is *nothing at all*, which cannot be confused with a present-but-blank
value.

**The fee is satoshis, not BTC, and not a float.** F-236 closed exactly this
defect in `mt` (`--input-value` took BTC as an `f64`). A wire format repeating it
would be the same bug in a place harder to change.

**Unknown tags are REFUSED, not skipped.** Skipping is how a format silently
diverges between two implementations; refusing makes the Go port's disagreement
visible on the first vector.

### 1.3 The body

| form | body is |
| --- | --- |
| `0x01` RAW | the serialized signed transaction, **with witness** |
| `0x02` CHUNKS | the `mt1` strings, **LF-separated**, ASCII |

LF inside the body is safe: the body is inside the record's **hex**, so no LF
reaches `sysw/open.go:74`'s record splitter.

---

## 2. WHAT `me` MUST DECODE, AND WHY §1's "OPAQUE" IS WRONG

`SPEC_engrave_transaction.md` §1 says *"`me` owns the container. The record body
is **opaque** to it."* **That is false, and it was never true of `md1`/`mk1`
either.**

EPD §6.3: *"A record in the PUBLIC section MUST NOT classify as `ms1` or as a
BIP-39 mnemonic, and **MUST additionally DECODE**."* The rationale is
anti-smuggling — BCH is publicly computable, so arbitrary bytes wrap into
something that *classifies* correctly, and a non-conforming sealer could put
secret bytes in cleartext where `picotool save` reaches them with **no
passphrase**. `me` satisfies this today by calling `md_codec::reassemble` and
`mk_codec::decode`.

**`tx:<hex of a seed>` is exactly that smuggling channel.** So:

**NORMATIVE: `me` DECODES a `tx:` record before admitting it to the public
section.**

| form | decode means |
| --- | --- |
| RAW | the body deserialises as a Bitcoin transaction, **and its txid equals the carried `txid` field** |
| CHUNKS | the body's `mt1` strings reassemble as a chunk set via `mt-codec` |

**The RAW check is stronger than §6.3 requires and costs nothing** — `me` holds
both the bytes and the claim, so it can compare them. A mismatch means the record
is internally inconsistent, which is R15's shape applied to the raw form.

**§1's "opaque" sentence must be corrected as part of this phase.** The accurate
statement: `me` does not interpret the record's *meaning*, but it does prove the
record is what it claims.

---

## 3. THE VECTORS

Rust-primary means these vectors are what the Go port is judged against. They
must therefore pin every choice a second implementer could make differently.

| # | vector | pins |
| --- | --- | --- |
| V1 | RAW form, no optional fields | the fixed layout; `n_fields = 0` |
| V2 | RAW form, all three optional fields | TLV order, `u16 BE` lengths, u64 fee |
| V3 | CHUNKS form, multi-chunk body | LF separation, `form = 0x02` |
| **V4** | **a transaction whose txid and `chunk_set_id` are both stated** | **§1.1's byte order, and §3.6b's top-20-bits claim, together** |
| V5 | absent optional field | that absence is *omission*, not an empty TLV |
| V6 | unknown tag | that it is REFUSED, not skipped |
| V7 | body at `MaxSectionLen` boundary | the cap, exactly |
| V8 | RAW whose carried txid ≠ the body's txid | the §2 consistency refusal |

**V4 is the one that matters.** It is the only vector that can catch a byte-order
disagreement, and a byte-order disagreement is invisible to every other test —
both implementations produce 32 plausible bytes.

---

## 4. TDD ORDER

Each step: failing test first, watch it fail **for the stated reason**, minimal
code, full suite green.

| step | test first | then |
| --- | --- | --- |
| 1 | `MaxSectionLen` is 32,734 and `boundBlob`'s no-wrap argument still holds | raise the constant; **update the comment that names 8191** |
| 2 | `me sysw pack` reads records from **stdin**; **empty stdin is refused** (R7) | the stdin path |
| 3 | a payload with no `IsSecret()` record packs **unsealed**; one with any packs **sealed**; **stderr says which and why, every time** | content-based sealing (§2.4) |
| 4 | V1–V3 round-trip | the layout encoder/decoder |
| 5 | **V4** | whatever `mt-codec` says; pin it |
| 6 | V5, V6, V8 | absence, unknown-tag refusal, txid consistency |
| 7 | V7 | boundary |
| 8 | a `tx:` record on **argv** is refused (R2) | the argv guard |

---

## 5. THE ORDERING CONSTRAINT NOBODY CAN SKIP

**`mt-codec` 0.1.0 must be published to crates.io before step 5 can build.**
`mnemonic-transaction` went public 2026-08-24 for this reason; the crate is
already structured and marked publishable, like its three siblings.

**`cargo publish` is IRREVERSIBLE — a version can be yanked but never replaced.**
So it happens **after** this plan is GREEN and after V4 has pinned the byte
order, not before. Publishing a codec whose txid convention we have not yet
verified would put a wrong constant in an immutable artifact.

---

## 6. WHAT MUST BE TRUE TO CLOSE P1

- This plan is **0C/0I** and the build gate has run on it.
- V1–V8 all pass, and **V4's byte order is stated in the vector file itself**, not
  only in code.
- `cargo nextest run --locked` green; `cargo clippy --all-targets` clean.
- **Every refusal added has a test that goes RED without its check** — the
  `mutate-refusals.sh` discipline, applied to `me`'s new guards.
- `mt-codec` published, and `me` depends on the **pinned published version**, not
  a path or a git URL.
- **§1's "opaque" sentence is corrected** in `SPEC_engrave_transaction.md`.
- **The near-miss rule**: every guard added here is tested against its nearest
  *legitimate* input as well as the hostile one. Six instances this cycle.

---

## 7. OUT OF SCOPE

P2–P6. The device. The plate. `mt encode --record` (that is P2, and it is what
*produces* these records — this phase only reads and packs them).
