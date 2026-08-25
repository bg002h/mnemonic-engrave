# IMPLEMENTATION PLAN — P1: `me`'s transaction container

**Status:** DRAFT v2, pre-R0. **Round 0 returned 5 Critical / 13 Important / 5
Minor and this is the rewrite**, not a fold — three of the Criticals were in one
32-byte field and the rest of §1–§3 rested on them.

**The round-0 report is `design/agent-reports/R0-P1-plan-round0.md`**, persisted
before any of this was written.

> **WHAT v1 GOT WRONG, kept at the top because it is the argument for gating
> plans at all.** v1's §1.1 called the txid byte order *"the most likely defect
> in this plan"* — and then **stated the losing answer as normative**, with an
> escape clause deferring to a decision `mt-codec` does not make. So **V4, the
> vector designed to pin the dangerous thing, was pinned to the wrong axis and
> could not have caught it.** Had this been implemented straight from §6's scope
> line, the disagreement would have surfaced when the Go port was written — or
> when a plate was cut and R15 refused a correct record.

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

### 1.1 THE TXID FIELD — display order, witness-stripped. TWO errors were here.

**NORMATIVE: the `txid` field is the transaction's txid in its STANDARD DISPLAY
ORDER — the byte-reversed form a user reads — computed over the transaction with
marker, flag and witnesses STRIPPED.**

Two independent mistakes lived in this one field in v1, and each is separately
sufficient to make two implementations disagree.

**(C1) The order. v1 said INTERNAL; the constellation uses DISPLAY.** Verified in
`mt-codec` itself — the answer is in the function's name:

```rust
// crates/mt-codec/src/string_layer/pipeline.rs:17-27
/// Top 20 bits of a txid **in its display form** — the content id (§10.13 c).
/// The display form is the byte-reversed one a user reads, and "which 20 bits,
/// from which end" is exactly where two implementations diverge silently. So
/// this takes the display string rather than raw bytes.
pub fn content_id_from_txid_display(txid_hex: &str) -> Result<u32>
```

It is the sole producer of a `chunk_set_id` (`pipeline.rs:54`), and
`SPEC_mt_v0_1.md:3546-3549` already ruled it. **`mt-codec`'s author anticipated
this exact trap and took a display STRING to defeat it. v1 walked into it
anyway** — and because `mt-codec` has no txid *field*, v1's "defer to `mt-codec`"
escape clause pointed at nothing.

**Shipped as written, R15 would have refused every byte-perfect chunks record.**

**(C2) txid, not wtxid.** v1 said *"the raw `double-SHA256` result"* while §1.3
says the RAW body carries the **witness**. Double-SHA256 over a witness-carrying
serialization is the **wtxid**. `SPEC_mt_v0_1.md:680` is explicit: the txid is
*"double-SHA-256 of the decoded transaction **with marker, flag and witnesses
stripped** … **Not** a hash of the engraved bytes."*

**V4 must use a SEGWIT transaction** (§3), because for a non-segwit transaction
txid == wtxid and the vector would pass in both worlds.

### 1.2 Legend field tags

| tag | field | value |
| --- | --- | --- |
| `0x01` | `TO` label | UTF-8, operator's own words (§3.4, asserted) |
| `0x02` | fee | **u64 satoshis**, big-endian, exactly 8 bytes |
| `0x03` | `FROM` wallet | exactly 4 bytes, the fingerprint |

**The fee is satoshis, not BTC, and not a float.** F-236 closed exactly this in
`mt`. A wire format repeating it would be the same bug somewhere harder to change.

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
| E7 | **An absent optional field is OMITTED from the list.** There is no empty encoding and no sentinel. | §2.1b asked what absence looks like |
| E8 | **An unknown tag is REFUSED, not skipped.** | skipping is how two implementations silently diverge |
| E9 | **A bad `magic`, an unknown `version`, or a `form` outside {0x01, 0x02} is REFUSED, each with its own message.** | **(I4)** v1 gave a verdict for unknown *tags* only |
| E10 | **`n_fields` MUST equal the number of TLVs actually parsed.** | a disagreement is a malformed record, not a hint |

**Every one of E1–E10 gets a vector (§3) and a test that goes RED without its
check.** A rule with no negative test is a comment.

## 2. WHAT `me` MUST DECODE — and what decoding can and cannot prove

§1 of the spec says *"the record body is **opaque** to `me`."* **False, and never
true of `md1`/`mk1` either** — `me` calls `md_codec::reassemble` today.

EPD §6.3: a public-section record *"MUST additionally **DECODE**."* The rationale
is anti-smuggling: BCH is publicly computable, so arbitrary bytes wrap into
something that *classifies* right, and a non-conforming sealer could put secret
bytes in cleartext where `picotool save` reaches them with **no passphrase**.

### 2.1 (C3) `mt_codec::decode` PROVES NOTHING ON ITS OWN

v1 said the CHUNKS decode satisfies §6.3. **It does not.** Round 0 executed it:
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
| RAW | deserialise the body as a Bitcoin transaction |
| CHUNKS | **reassemble via `mt-codec`, THEN deserialise the result** as a Bitcoin transaction |

**and in both cases the transaction's txid (display order, witness-stripped, §1.1)
MUST equal the carried `txid` field.**

**Chaining the parse onto the reassembly is what closes C3**, and it costs
nothing extra: `me` needs a transaction parser for the RAW form regardless, so
the CHUNKS path reuses it. The two forms become **symmetric** — each ends at *the
bytes are a real transaction, and it is the one the record claims*.

**(I5) `me` gains a `bitcoin` dependency.** It has none today — its deps are the
three sibling codecs, clap, zeroize, serde, aes-gcm, pbkdf2, sha2, bip39, rand,
rpassword. v1 required a transaction parse and never said where it comes from.
`bitcoin` is a public crate; `mt-codec` already declares it.

### 2.3 (I8) A `tx:` record that fails DECODE is REFUSED, not warned

The two containers already differ and v1 said which applied to neither: `seal`
**refuses** (`seal/mod.rs:171`), while `sysw` **warns and packs anyway**
(`main.rs:1141` — *"then the container is built anyway"*).

**NORMATIVE: `tx:` follows `seal`'s posture — refuse.** The `sysw` warn-and-pack
posture is defensible for a record whose worst case is an unengraveable plate. It
is not defensible for the one record class whose failure mode is **secret bytes
riding in cleartext**, which is the whole reason §6.3 exists.

## 3. THE VECTORS

Rust-primary means these are what the Go port is judged against. They must pin
every choice a second implementer could make differently.

| # | vector | pins |
| --- | --- | --- |
| V1 | RAW, no optional fields | the fixed layout; `n_fields = 0` |
| V2 | RAW, all three optional fields | **E1's ascending tag order**, `u16 BE` lengths, u64 fee |
| V3 | CHUNKS, multi-chunk body | LF separation, `form = 0x02` |
| **V4** | **a SEGWIT transaction, with txid AND wtxid AND `chunk_set_id` all written out** | **§1.1's display order AND txid-not-wtxid, together.** Segwit is required: for a legacy transaction txid == wtxid and the vector passes in both worlds |
| V5 | absent optional field | absence is omission (E7) |
| V6 | unknown tag | REFUSED (E8) |
| V7 | body at **16,322 B minus the fields present** | the **record framing** ceiling — see below |
| V8 | RAW whose carried txid ≠ the body's | the §2.2 consistency refusal |
| **V9** | **duplicate tag** | E2 |
| **V10** | **trailing bytes after the body** | E3/E4 |
| **V11** | **`body_len` larger than the bytes remaining** | E5, before allocation |
| **V12** | **zero-length TLV value** | E6 |
| **V13** | **bad magic / unknown version / `form = 0x03`** | E9, three distinct messages |
| **V14** | **`n_fields` ≠ the TLVs present** | E10 |
| **V15** | **R15 NEGATIVE: a chunks record whose carried txid's top 20 bits ≠ its chunks' `chunk_set_id`** | **(I13)** without it, deleting R15's comparison leaves every other vector green — and this vector alone would have caught C1 on the first run |

### 3.1 (I7) V7's number, and a THIRD ceiling nobody had

v1 said *"body at `MaxSectionLen` boundary"*. **Unconstructible.** A record is
`tx:` plus hex, so:

```
32,734 section chars − 3 for "tx:"  = 32,731 hex chars
                                     = 16,365 bytes
                       − 43 framing  = 16,322 bytes of body, minus the fields
```

**So there are THREE ceilings and the spec states only two:**

| ceiling | value | where |
| --- | --- | --- |
| container section | 16,367 B | spec §2.3 **as written** |
| **record framing** | **16,322 B** | **this plan — new** |
| engraveable | 14,560 B | spec §4.1a, at 0.60 mm |

**Spec §2.3's "a 16,367-byte raw transaction" is wrong by the framing.** Round 3
already corrected it once for the engraving ceiling; this is a second correction
in the same sentence, and P1 must land it.

### 3.2 (I12) THE VECTORS MAY NOT BE PRODUCED BY THE CODE THEY JUDGE

`mt-codec`'s own `lib.rs:9-14` rules against exactly that, and `mt`'s vectors are
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
| 2 | `me sysw pack` reads from **stdin**; **empty stdin refused** (R7); **a TTY with no `--in` and no argv refuses instead of blocking** | the stdin path |
| 3 | explicit passphrase flags **win**; content decides **only** when none is given; **stderr says which way and why, every time** | content-based sealing |
| 4 | V1–V3 round-trip | the layout codec |
| 5 | **V4** | the txid field, display order, witness-stripped |
| 6 | V5–V6, V9–V14 | every rule in E1–E10 |
| 7 | V7 at **16,322 − F** | the framing ceiling |
| 8 | V8 and **V15** | the txid-consistency refusals, both forms |
| 9 | a `tx:` record on **argv** refused (R2) | the argv guard |

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

### 4.3 (I9) STEP 3 MUST RULE PRECEDENCE, BECAUSE FOUR FLAGS ALREADY EXIST

`me sysw pack` already has `--passphrase-words`, `--passphrase-ask`,
`--no-passphrase` and `--allow-weak`. v1 said "seal by content" and never said
what happens when the operator also passes a flag — so step 3's test could not be
written.

**NORMATIVE:** an explicit flag **always wins**; content decides **only** in
their absence; and **stderr states which rule applied and why, on every run**. A
content-dependent default that is silent is worse than the default it replaced.

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

## 6. WHAT MUST BE TRUE TO CLOSE P1

- This plan is **0C/0I** and the build gate has run on it.
- **V1–V15 pass**, and **V4 writes out txid, wtxid and `chunk_set_id` explicitly**
  in the vector file, not only in code.
- **Every rule E1–E10 has a test that goes RED without its check.** A rule with no
  negative test is a comment.
- `cargo nextest run --locked` green; `cargo clippy --all-targets` clean.
- **The vectors were not produced by the code they judge** (§3.2).
- `mt-codec` published; `me` depends on the **pinned published version**.
- **Spec corrections this phase owns:** §1's "opaque" sentence, and §2.3's
  "16,367-byte raw transaction" (§3.1's third ceiling).
- **The near-miss rule**: every guard here is tested against its nearest
  *legitimate* input as well as the hostile one. Seven instances this cycle.

## 7. OUT OF SCOPE

P2–P6. The device. The plate. `mt encode --record` (that is P2, and it is what
*produces* these records — this phase only reads and packs them).
