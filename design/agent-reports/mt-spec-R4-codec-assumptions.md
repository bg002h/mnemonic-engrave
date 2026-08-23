# R4 — `mt` v0.1 spec, pre-implementation readiness: where the codec layer must be GUESSED

Artifact: `design/SPEC_mt_v0_1.md` at `d2d1a58` (1,694 lines, read in full).
Scope: **codec and format layer only.** CLI/IO/transport belongs to the other reviewer.
Question answered: *an implementer sits down to build `mt-codec` tomorrow — where
must they guess, and what will they guess differently from the next implementer?*

This is not a defect hunt. Every item below is a place the spec **does not decide
something**, with the candidate decisions, the one the spec's own logic implies,
and the observable divergence between two implementers who guess differently.

## Verdict

| severity | count | items |
| --- | --- | --- |
| **Critical** — plates become mutually unreadable | **4** | A-1, A-2, A-3, A-4 |
| **Important** | **4** | A-5, A-6, A-7, A-8 |
| **Minor** | **1** | A-9 |
| **Nit** | **1** | A-10 |
| **total** | **10** | |

**Six of the ten are new to this round.** A-3 is R3 C-6 carried unfolded; A-9's
first limb is part of R3 C-5. The R3 fold (`52ad001`) responded to exactly one of
R3's header Criticals — the width — and **A-2 is a gap the fold itself opened**:
the pre-fold text said the header was shared *"verbatim"* with `md-codec`, which
answered the `count−1` question by reference. Widening the field deleted the
answer and did not replace it. *A fold is authorship, and its own defects are the
ones nobody has looked at yet.*

**The single highest-value fix is A-1**, and it is not a wording fix: `mt1`'s NUMS
constant **cannot be derived by an implementer**. §10.13 is titled *"RULED, ready
to build"*; the one input that would make (a) buildable is absent from the spec
and cannot be reconstructed from anything in the repo. Everything `mt string`
emits is blocked on the operator naming one ASCII string.

### What is machine-verified in this report

Every code fact below was **run or read from source**, never from a doc comment:
`descriptor-mnemonic/crates/md-codec/src/{bch,chunk,codex32,bitstream,header}.rs`,
`mnemonic-key/crates/mk-codec/src/consts.rs`,
`mnemonic-engrave/design/measurements/mt-size-probe/src/bin/select.rs`,
`bitcoin-0.32.101/src/blockdata/transaction.rs`. Chunk-layout divergences in A-5
and character counts in A-4 were **computed**, not estimated. I recomputed
nothing that `design/measurements/` already establishes.

---

### A-1 — The NUMS constant is not derivable. `mt1` cannot be built until the operator names a domain string.

**Severity: Critical. Section: §10.13(a).**

**What the spec says.** §10.13(a): *"Its own NUMS constant. `MD_REGULAR_CONST` is
hardcoded into checksum create and verify (`crates/md-codec/src/bch.rs`). Every
constellation format gets its own; without a distinct one **an `mt1` chunk would
verify as a valid `md1` chunk**."* The section is titled *"RULED, ready to build"*
and *"No longer blocking as a design question — it is now scoped implementation
work with every decision made."*

**What is actually true of the constant** (read from source, both siblings):

| format | constant | derived from |
| --- | --- | --- |
| `md1` | `MD_REGULAR_CONST = 0x0815c07747a3392e7` (`md-codec/src/bch.rs:33`) | top 65 bits of `SHA-256(b"shibbolethnums")` |
| `mk1` | `MK_REGULAR_CONST = 0x1062435f91072fa5c` (`mk-codec/src/consts.rs:18`) | top 65 bits of `SHA-256(NUMS_DOMAIN)`, `NUMS_DOMAIN = b"shibbolethnumskey"` (`consts.rs:15`) |

Both carry a drift-guard test that recomputes the constant from the domain string
(`md-codec/src/bch.rs::md_regular_const_reproduces_from_nums_domain`;
`mk-codec/src/consts.rs::nums_constants_reproduce_from_domain`), and mk-codec
carries a second test, `nums_string_differs_from_md1`, asserting the two domains
are not equal.

**So the derivation RULE is public and fixed — `hi = u128::from_be_bytes(digest[0..16]); const = hi >> 63` — and the DOMAIN STRING is a chosen name.** It is not a
function of the HRP, of the format name, or of anything in the spec. **An
implementer cannot compute `mt1`'s constant. They can only be told it.**

**What an implementer must guess:** the exact bytes of `MT_NUMS_DOMAIN`.

**Candidate guesses:**

1. `b"shibbolethnumstx"` — following mk1's `md` + suffix pattern with the short form.
2. `b"shibbolethnumstransaction"` — the same pattern with the long form.
3. `b"shibbolethnumsmt"` — suffixing the HRP rather than the concept.
4. **Reuse `MD_REGULAR_CONST` outright** — §10.13(a) says `mt1` *"gets its own"*
   NUMS constant but never says the numeric value must differ, and an implementer
   forking `bch.rs` who reads only *"its own HRP"* as the distinguishing change
   will leave the constant alone. This is the guess the fork mechanic actively
   invites: `bch.rs` is copied wholesale and only `codex32.rs`'s `HRP` looks like
   it needs editing.

**What the spec's logic implies:** the precedent is `"shibbolethnums"` + a word
naming the format's subject (`md` = the base, `mk` = `+"key"`), so guess 1 or 2 —
but **nothing in the spec or the sibling repos chooses between them**, and the
choice is not derivable. This is an operator ruling, not a review finding. It must
also ship the drift-guard test both siblings carry, and a `!=` assertion against
`md1`'s and `mk1`'s domains, because guess 4 is the failure the section exists to
prevent.

**The observable divergence.** Implementer A picks `"shibbolethnumstx"`, B picks
`"shibbolethnumstransaction"`. Both produce well-formed `mt1…` strings. **B's
decoder rejects every chunk A engraved**, and the way it rejects is the worst
diagnostic available: `bch_verify_regular` returns false, the syndrome is
non-zero, and `decode_regular_errors` returns `None` because the residue exceeds
`t = 4`. **The report a recoverer gets is "this plate is damaged beyond
correction", not "this plate was cut by a different tool."** They will conclude
the steel failed.

Under guess 4 the failure inverts and gets worse: an `mt1` chunk verifies as a
valid `md1` chunk, which §10.13(a) itself identifies as *"a real hazard, not a
theoretical one"* for a bearer plate sitting in a drawer beside `md1` plates.

---

### A-2 — Is `count` stored as `count−1`? The fold widened the field and deleted the answer.

**Severity: Critical. Section: §3 (the R3 fold block).**

**What the spec says.** §3: `version(4) + chunked(1) + chunk_set_id(20) + count(8) + index(8)`, and *"`mt1` therefore uses 8 bits each for `count` and `index` — a
41-bit header **admitting 256 chunks**."* It quotes `md-codec`'s layout as
`… + count−1(6) + index(6) = 37 bits` in the line immediately above.

**What `md-codec` does** (`chunk.rs`): `write()` validates `(1..=64).contains(&count)` (`:37-39`), then `w.write_bits((self.count - 1) as u64, 6)` (`:55`);
`read()` does `let count = (r.read_bits(6)? + 1) as u8` (`:79`). `index` is
written **plain and 0-based** (`:56`), validated `index < count` (`:41-46`).

**What an implementer must guess:** whether the widened 8-bit `count` carries the
same `−1` offset, and whether `index` picked one up by symmetry.

**Candidate guesses:**

1. **Plain 8-bit count.** `count(8)` is what §3's tally literally says; valid
   range `1..=255`, `0` invalid. This is what a reader who has never seen
   `md-codec` writes, and what a reader who has seen it may *still* write, because
   §3 changed the notation from `count−1(6)` to `count(8)` in the same sentence —
   which reads as a deliberate change of encoding, not just of width.
2. **`count−1` in 8 bits.** Valid range `1..=256`, all 256 values used.
3. `index−1` as well, by symmetry with (2).

**What the spec's logic implies: (2), and it says so arithmetically.** *"admitting
**256** chunks"* is true only under `count−1`; under a plain 8-bit field the
ceiling is 255. The fold's own number decides the question — it just does not
state the encoding that produces it. `index` stays plain and 0-based, because
`md-codec` writes it plain and because `index < count` is the validation the fork
inherits; (3) breaks that predicate at `index = count`.

**The observable divergence.** A (guess 2) writes `count = 96` as the byte `0x5F`.
B (guess 1) reads `0x5F` as `count = 95`, sees 96 chunks presented, and refuses.
Reverse the roles and B writes `0x60`, A reads 97, and refuses with
`ChunkSetIncomplete { got: 96, expected: 97 }`. **Every multi-chunk plate set is
unreadable in both directions, and the error tells the recoverer they are missing
a chunk that was never engraved** — so the failure sends them hunting for a plate
that does not exist. Single-chunk transactions (none of the seven measured
artifacts; the smallest is 5 chunks) would work fine, which is exactly the shape
that lets the bug ship.

---

### A-3 — The 4-bit `version` value is still unassigned. (R3 C-6, unfolded — and the fold made the obvious guess worse.)

**Severity: Critical. Section: §10.13.**

R3 lens 3 filed this as C-6 and the R3 fold did not close it. I re-ran the search
over the current text: `grep -n "version" SPEC_mt_v0_1.md` returns 17 hits, all
either QR-version, draft-version, or the field's *name* in the §3 tally. **No hit
assigns the 4-bit field a value.** §10.13 rules three things — NUMS, HRP, content
id — and declares itself complete, so an implementer has no prompt to ask.

I record it here rather than restating R3, because **the fold changed the answer's
balance**. Before the fold, §3 said the header was shared *"verbatim"* with
`md-codec`, so "inherit `WF_REDESIGN_VERSION = 4`" was at least a defensible
reading. After the fold, `mt1`'s header is structurally a *different* header —
different width, different field sizes — so inheriting md-codec's version number
is now the *least* defensible guess, while remaining the one a forking
implementer takes by default (the constant is right there in `header.rs:39`).

**Candidate guesses:** `4` (inherited from the fork source), `0` (a new format's
first version), `1` (a new format's first version, 1-indexed).

**What the spec's logic implies:** `mt1` has its own HRP and its own NUMS
constant, so it shares no version namespace with `md1` and is free to start at
`0` or `1`. Note `md-codec` deliberately *rejects* `0` and `2` as v0.x sentinels
(`chunk.rs:70-73`, `header.rs:57-62`) — that history is `md1`'s and does not bind
`mt1`, so the reservation is not inherited. **Whichever value is chosen, the point
is that it be written down**; there is no derivation to appeal to.

**The observable divergence.** `ChunkHeader::read` refuses on the **first four
bits** with `WireVersionMismatch { got }`, before the HRP or the NUMS constant is
consulted — so the two features §10.13 *does* rule cannot rescue it. A's plate is
rejected by B's decoder as "wrong wire format", which at least names the problem,
unlike A-1.

---

### A-4 — The last chunk's payload length has no signal, and there is no `MessageLen`. The two verbs need different answers and only one of them already has it.

**Severity: Critical. Section: §3b, §8.7c (arithmetic), absent everywhere else.**

**What the spec says.** §3b: *"A chunk carries **40 payload bytes**, and the
container holds **64 chunks**, so the hard ceiling is **2,560 B**."* §8.7c
computes the largest artifact as *"96 chunks, 34,656 bits, **6,932 bech32
characters**"*. `grep -n "MessageLen"` over the spec returns **one** hit
(line 1371) — inside the *retracted* UR discussion, describing what UR carried
and `mt1` does not.

`34,656 = 96 × 361 = 96 × (41 + 320)`. **The spec's own arithmetic pads every
chunk to exactly 40 payload bytes, including the last** — which for a 3,809 B
payload holds only 9 bytes (`3809 − 95×40`).

**What an implementer must guess:** how a decoder learns that the final chunk
carries 9 bytes and not 40.

**For `mt string` the spec's fork already answers it, and this is the cheapest
possible fix — write down what is already true.** `md-codec` never stores a
length: `reassemble_with_opts` recovers it from the codex32 symbol count,
`payload_byte_count = (symbol_aligned_bit_count - 37) / 8` (`chunk.rs`). **I
verified the identity survives the widening**: chunk bits = `41 + 8N`,
symbol-aligned = `ceil((41+8N)/5) × 5 ∈ [41+8N, 41+8N+4]`, so
`(symbol_aligned_bit_count − 41) / 8` floor-divides to exactly `N` for all `N`.
No length field is needed, no padding is needed, and the last chunk is simply
shorter. One sentence in §3b closes it.

**For `mt qr` nothing answers it**, because there is no per-chunk container to
carry the length — this is the half of the question that is genuinely open.

**Candidate guesses:**

1. **Zero-pad every chunk to 40 bytes** and let the transaction/PSBT parser stop
   at its own end. §8.7c's arithmetic commits to this.
2. **Length-delimit**: the final chunk is short, and its length is recovered from
   the total stream length (`total_bits − 95 × 361`).
3. **An explicit `MessageLen`** in every header — the field UR had and §3 removed.

**What the spec's logic implies: (2)**, for two reasons the spec argues elsewhere.
§4's objective is *"never leave redundancy unbought"* — under (1) the largest
artifact spends **31 bytes** of Reed-Solomon budget on zero padding, which is the
one currency §4 says may never be wasted. And (2) is the same rule `mt string`
already uses (length implied by the container), which keeps *"one fragmentation
scheme to specify, test, teach a recoverer, and get wrong only once"* (§3) true.
But §8.7c's number was computed under (1), so the spec currently asserts both.

**The observable divergence.** A (guess 1) reassembles 3,840 bytes; B (guess 2)
reassembles 3,809. Whether B's decoder can read A's plate depends on whether the
PSBT parser tolerates 31 trailing zero bytes — which is **itself unspecified**, so
the answer is "sometimes". A decoder that checks all-bytes-consumed refuses an
intact plate; one that does not, silently accepts, and then the two implementations
disagree about what the plate *contains* while agreeing on the transaction. The
character count differs too: I computed 6,882 (guess 2, bit-concatenated), 6,932
(the spec's §8.7c model), and 6,958 (per-chunk symbol alignment) for the same
artifact — three numbers for one quantity, from one spec.

---

### A-5 — "A flat 40 payload bytes per chunk" mis-describes `md-codec`, which balances. The two rules disagree on 3 of the 7 artifacts in §3b's own table.

**Severity: Important. Section: §3b (correction block), §11, vs §10.12.**

**What the spec says.** §3b: *"`md-codec` sizes chunks by
`SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64 * 5 = 320` bits (`chunk.rs:224`), applied
over `payload_bytes.len() * 8` (`chunk.rs:253-254`) — a **flat 40 bytes per
chunk**."* §11 repeats it: *"chunk sizing is a flat 40 payload bytes."*

**What `chunk.rs` does.** Lines 253-254 choose the **count**:
`chunks_needed = (payload_bytes.len() * 8).div_ceil(320)`. Line **267** then sizes
the chunks: `let bytes_per_chunk = payload_bytes.len().div_ceil(count as usize);`
— **equal-sized chunks with a short tail, not 40-byte chunks.** The spec's
citation is correct about the line it cites and the conclusion it draws is not a
property of the code.

**Measured** (computed over §3b's own table; counts agree everywhere, sizes do not):

| payload | count | flat-40 sizes | `md-codec` balanced sizes | diverge? |
| --- | --- | --- | --- | --- |
| 162 B | 5 | 40,40,40,40,**2** | 33,33,33,33,**30** | **YES** |
| 405 B | 11 | 40×10, **5** | 37×10, **35** | **YES** |
| 535 B | 14 | 40×13, **15** | 39×13, **28** | **YES** |
| 742 B | 19 | 40×18, 22 | 40×18, 22 | no |
| 2,498 B | 63 | 40×62, 18 | 40×62, 18 | no |
| 3,538 B | 89 | 40×88, 18 | 40×88, 18 | no |
| 3,809 B | 96 | 40×95, 9 | 40×95, 9 | no |

**What an implementer must guess:** whether `mt1` fills to 40 with a short tail,
or balances with `div_ceil`.

**What the spec's logic implies: balance,** and §10.12 already ruled it in that
vocabulary — *"A 535 B transaction **balanced** at 40 B/chunk is 14 chunks"*, and
its whole argument is that balancing buys error-correction budget. §10.13 also
tells the implementer to fork `md-codec`, and `md-codec` balances. So §3b's
sentence and §11's repetition are the two places to edit — **but note §10.12 uses
"balanced at 40 B/chunk" for what is really "count sized by the 320-bit budget,
bytes equalised", so all three sentences describe the same rule with two
incompatible words.**

**The observable divergence.** Two encoders emit **different codex32 strings** for
the same 535 B transaction — different characters from the first chunk onward. A
hand engraver who re-runs `mt string` to check their plate gets a total mismatch
with no indication which is authoritative. Reassembly still works in each
direction (chunk lengths are self-describing per A-4), so this is *not* a
readability break — it is a determinism break, and it invalidates any conformance
corpus pinned by one implementation, **including the Go port §10.13 explicitly
anticipates**. This constellation has already shipped exactly that failure once:
Go and Rust computed different `WalletPolicyId`s while 887 fork tests passed
either way.

---

### A-6 — One transaction now has two chunk sets sharing one content id, and no header bit says which payload is inside.

**Severity: Important. Section: §10.13(c) vs §3b and §10.10.**

**What the spec says.** §10.13(c): the content id is *"the top 20 bits of the
EXTRACTED transaction's txid"*. §3b: `mt string`'s payload is *"the raw signed
transaction, NOT the PSBT"*. §10.10: `mt qr`'s payload is the finalized PSBT.
§3: *"Both verbs share the `mt1` chunk header"*, *"identical header both ways"*.

**Consequence the spec does not draw:** the same transaction produces **two
different chunk sets carrying the same `chunk_set_id`** — because the id is
derived from the transaction, and the transaction is not the payload.

**What an implementer must guess:** given a reassembled byte string from a set of
`mt1` chunks, which parser to apply — and what to hash for §10.13's re-derivation
compare, since a PSBT must be `extract_tx()`'d first and a raw transaction hashed
directly.

**Candidate guesses:**

1. **Sniff the payload.** A PSBT begins `70 73 62 74 ff`; a raw transaction begins
   with a 4-byte version (`01`/`02` `00 00 00`). Decidable in practice.
2. **Infer from the medium**: codex32 string ⇒ raw transaction, QR ⇒ PSBT.
3. **Spend the functionless `chunked` bit** (A-7) as a payload-type discriminator.

**What the spec's logic implies: (2)**, because §10.10 binds payload format to
verb and the reader always knows which medium it scanned. **(3) is free and
strictly better**, but only if it is decided *now* — it is a wire-format bit, and
after the first plate is cut it cannot be reclaimed.

**The observable divergence, and an accidental safety net worth naming.** A pile
of `mt1` chunks mixing a string plate and a QR plate of the same transaction has
one `chunk_set_id` and would concatenate into garbage. **Measured, it does not:**
the MIN-form PSBT is **+58 B minimum** over the raw transaction
(`RESULTS_psbt_envelope_2026-08-23.txt`, per-input deltas +58…+61, five-input
+271), and 58 > 40, so the two sets' `count` always differs by at least one in
every measured case and `ChunkSetInconsistent` fires. **That is arithmetic luck,
not design.** Nothing in the spec claims it, no test pins it, and a future
tightening of the MIN form below 40 B/input silently removes it. Implementer A
relies on it; implementer B adds a discriminator; only B's decoder still refuses
correctly if the PSBT form ever shrinks.

---

### A-7 — The `chunked` bit has no function in `mt1`, no stated value, and no stated behaviour on `0`.

**Severity: Important. Section: §3 (the 41-bit tally).**

**What the spec says.** §3 enumerates `version(4) + chunked(1) + chunk_set_id(20) + count(8) + index(8)` and never mentions the bit again.

**What it is for in `md-codec`, and why that reason is gone.** The bit is in-band
auto-dispatch between two headers: the 5-bit single-payload header
(`header.rs:12-16`, first symbol `[divergent][v3][v2][v1][v0]`) and the 37-bit
chunk header (`chunk.rs:4-6`, first symbol `[v3][v2][v1][v0][chunked]`). A decoder
reads five bits and branches. `ChunkHeader::read` returns
`ChunkHeaderChunkedFlagMissing` when the bit is `0` (`chunk.rs:75-77`).

**`mt1` has no single-payload form.** §3: *"Both verbs fragment with the `mt1`
chunk header"* — even a one-chunk transaction is a chunk (§10.8: *"A lone symbol
reads `1/1`, which is the only way it can state that it is whole"*). **So nothing
dispatches, and the bit is a constant.**

**What an implementer must guess:** its value, and what a decoder does with the
other one.

**Candidate guesses:**

1. **Always `1`, refuse `0`** — the fork's behaviour, inherited unexamined.
2. **Reserved, must be `0`** — the natural reading of an unused bit.
3. **Drop it: a 40-bit header** — the natural reading of *"this field has no
   purpose"*, and 40 bits is a byte boundary, which is independently attractive
   given A-4's framing question.

**What the spec's logic implies: (1).** §3 counts the bit inside the 41-bit total,
so it exists on the wire; the fork sets it to 1 and refuses 0. One sentence.
**But (3) is the guess a thoughtful implementer makes** — the bit is provably
dead, and dropping it makes the header byte-aligned.

**The observable divergence, and it is the worst-behaved failure in this report.**
(1) vs (2): each decoder refuses the other's plates outright — clean. (1)/(2) vs
(3): **every subsequent field is shifted by one bit.** The 20-bit set id, the
8-bit count and the 8-bit index all read as noise. The most likely report is a
`count` in the hundreds or a `WireVersionMismatch` from the *next* chunk, and the
recoverer has no way to tell a one-bit framing disagreement from a destroyed
plate. Where A-3 fails loudly and A-1 fails as "damaged", this fails as
*nonsense*.

**Better than settling it: spend it.** Ruling the bit as A-6's payload-type
discriminator costs zero bits, keeps the header at 41 bits (matching §8.7c's
arithmetic), and closes a real gap. That option expires at first engraving.

---

### A-8 — Reassembly semantics — duplicates, gaps, mismatch — are unwritten, although §2 lists them as what the codec is for.

**Severity: Important. Section: §2 (promise), absent thereafter.**

**What the spec says.** §2, listing what `mt-codec` exists to specify: *"how a
recoverer reassembles them, and how they know a fragment is missing."* Nothing
later in the spec specifies either. §10.13 says only *"Reassembly re-derives the
id from the transaction it decoded and compares"* — and does not say what happens
when the comparison fails.

**Inherited behaviour if `md-codec` is forked verbatim** (`reassemble_with_opts`,
read from source):

| case | `md-codec` does |
| --- | --- |
| **out of order** | fine — `parsed.sort_by_key(index)` |
| **duplicate chunk** | **fatal.** `parsed.len() != expected_count` → `ChunkSetIncomplete { got: 4, expected: 3 }`. There is no de-duplication anywhere in the function |
| **duplicate + one missing** | passes the length check, then fails `ChunkIndexGap { expected, got }` |
| **missing chunk** | `ChunkSetIncomplete { got: 95, expected: 96 }` |
| **two sets mixed** | `ChunkSetInconsistent` on differing `count`/`chunk_set_id`/`version` — but only *consistency* is checked, not correctness |
| **content-id mismatch** | `ChunkSetIdMismatch { expected, derived }` — hard, and **unconditional** even under the partial-decode option, which the source calls *"the content-id oracle; funds-load-bearing invariant"* |

**What an implementer must guess, and the duplicate case is not an edge case
here.** `md1` chunks are transcribed by a human from a card. **`mt qr` chunks are
scanned by a camera off up to five plates**, and re-scanning a symbol is the
normal operating mode, not an error. An inherited decoder **refuses a complete,
undamaged plate set** because the operator scanned one symbol twice.

**Candidate guesses:** (a) de-duplicate identical chunks silently and refuse
*conflicting* ones at the same index — what a scanner needs; (b) fork verbatim and
refuse; (c) accept the first chunk seen at each index and ignore the rest — which
silently prefers whichever plate the operator happened to scan first, and is the
one guess that can produce a *wrong transaction* rather than a refusal.

**What the spec's logic implies: (a).** §10.2 rules that `mt` ships its own
static-scan reader, so the codec owns the scanner's semantics; and §10.8's whole
purpose is *"a recoverer must be able to inventory what they hold and name what is
missing"*, which presumes re-scanning. (c) must be explicitly excluded — it is
the only option that can hand back a transaction nobody engraved.

**One more, cheap and normative already.** §8 rules *"Every refusal names the
number that caused it. A refusal that says only 'too large' costs the operator a
round trip."* `ChunkSetIncomplete { got: 95, expected: 96 }` **does not name which
index is missing** — and the missing index is the one fact a recoverer can act on,
because §10.8's per-symbol `n/m` labels let them walk to the right plate. The
header carries enough to say it; the inherited error type does not.

**The observable divergence.** A's reader ingests a re-scanned pile and
reassembles. B's refuses it with `ChunkSetIncomplete`, telling a recoverer their
intact plate set is incomplete. Both decoders are "correct" against the spec,
because the spec says nothing.

---

### A-9 — Bit order is stated nowhere, and §10.13(b) names the HRP as `mt1` where the checksum wants `mt`.

**Severity: Minor. Section: absent; §10.13(b).**

Two one-line items, both free to fix, both load-bearing once the Go port §10.13
anticipates exists.

**(i) Bit order and symbol padding.** `grep -n "MSB\|LSB\|endian\|bit order\|padding"` over the spec returns **two** hits, both inside the retracted UR
discussion or §10.13's txid sentence. The conventions the fork actually carries:
bits packed **MSB-first**, first payload bit into the most-significant bit of the
first byte, final byte zero-padded (`bitstream.rs:3-5`); multi-bit fields written
MSB-first within themselves (`write_bits`); and — the part R3 C-5 did not name — a
**short final 5-bit symbol is left-justified, zero-padding the LOW bits**, which
`codex32.rs:49-53` calls *"the canonical form"*. An LSB-first implementer produces
a byte string sharing not one field with an MSB-first one. **The spec's logic
implies MSB-first** (§10.13 says fork `md-codec`), so this is transcription, not
design — but it is the transcription a Go porter gets wrong.

**(ii) `hrp_expand("mt")`, not `hrp_expand("mt1")`.** §10.13(b): *"**Its own
HRP**, `mt1`, currently hardcoded at four sites in `md-codec`."* In `md-codec` the
HRP and the prefix are **different strings**: `const HRP: &str = "md"`
(`codex32.rs:15`) is what `bch_create_checksum_regular(HRP, …)` feeds to
`hrp_expand`, while `HRP_PREFIX = "md1"` (`chunk.rs`) is the printed prefix — the
`1` is BIP-173's separator, emitted by `s.push('1')` in `wrap_payload`, and is not
part of the checksum domain. An implementer who reads §10.13(b) literally feeds
`hrp_expand("mt1")` and gets a **different checksum domain**: their strings
round-trip perfectly against themselves and fail every cross-implementation
verify, with the same "damaged beyond correction" diagnostic as A-1.

---

### A-10 — Record the codex32 capacity the widened header just consumed, and the length identity that survived it.

**Severity: Nit. Section: §3 (the R3 fold block).**

The fold widened the header from 37 to 41 bits and priced it as *"4 bits per
chunk: 48 bytes on the 96-chunk artifact, which changes no plate count."* True,
and it is not the constraint that mattered. **The binding constraint is the
codex32 regular code, and nobody checked it in the fold.** I did:

- `41 + 8×40 = 361` bits → `ceil(361/5)` = **73 data symbols**.
- `wrap_payload` refuses more than `REGULAR_DATA_SYMBOLS_MAX = 80`
  (`codex32.rs:25`, enforced at `:89-93`). **73 ≤ 80 — 7 symbols of margin.**
- Codeword length `73 + REGULAR_CHECKSUM_SYMBOLS(13) = 86 ≤ REGULAR_CODE_SYMBOLS_MAX = 93` (`codex32.rs:32`). Safe.

So the widening is sound. **But the margin is now 7 symbols and the spec records
neither the check nor the number**, so the next person to widen a field, or to
revisit §10.12's 320-bit budget, has no marker telling them a cliff is 7 symbols
away. Two sentences and a number.

Same paragraph should carry A-4's identity: `payload_byte_count = (symbol_aligned_bit_count − 41) / 8` recovers `N` exactly, because the symbol
padding is at most 4 bits. That is the whole answer to *"is there a `MessageLen`"*
for `mt string`, and it is already true — it just is not written down.

---

## Ranked decision list

Ordered by what must be settled first, not by severity alone: A-1 gates code
existing at all; A-2/A-3/A-7 are single constants that make plates mutually
readable; A-4/A-5 decide what bytes reach steel; the rest can be settled while
implementation proceeds.

| # | item | what to settle | who decides |
| --- | --- | --- | --- |
| **1** | **A-1** | The NUMS domain string for `mt1`, plus the drift-guard test and a `!=` assertion against `md1`/`mk1`. **Nothing in the string form can be built or tested before this exists.** | **Operator** — it is a name, not a derivation |
| **2** | **A-2** | `count` is stored as `count−1` (the fold's own "256 chunks" says so); `index` plain and 0-based; `index < count` | spec, one sentence |
| **3** | **A-3** | The 4-bit `version` value. Any value; `md-codec`'s `4` is now the weakest choice | spec, one constant |
| **4** | **A-7** | The `chunked` bit: `1`-and-refuse-`0`, **or** spend it as A-6's payload-type discriminator. **This option expires at first engraving** | spec — but see A-6 before choosing |
| **5** | **A-4** | Whether the last chunk is zero-padded to 40 B or length-delimited, for `mt qr`; and write down that `mt string` already answers it via the symbol count. §8.7c's arithmetic must then match the ruling | spec |
| **6** | **A-5** | Fill-to-40 or `div_ceil` balance. §10.12 already ruled balance; §3b and §11 say flat. Fix the two sentences, and settle on one word | spec |
| **7** | **A-6** | Which parser a reassembler applies, and what it hashes for the content-id compare. Depends on #4 | spec |
| **8** | **A-8** | Duplicate/gap/mismatch behaviour, explicitly excluding first-wins; and name the missing index in the refusal, per §8's own rule | spec |
| **9** | **A-9** | One sentence for MSB-first + low-bit symbol padding; correct §10.13(b) to `hrp_expand("mt")` with prefix `mt1` | spec, two sentences |
| **10** | **A-10** | Record 73/80 data symbols and the `(sabc − 41)/8` identity | spec, two sentences |

**Items 2, 3, 4, 9 and 10 together are roughly one paragraph of spec text**, and
they convert three Criticals and a Minor into settled facts. Item 1 is the only
one that cannot be written by a reviewer or an implementer.
