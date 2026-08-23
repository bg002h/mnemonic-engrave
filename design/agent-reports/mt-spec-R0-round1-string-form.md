# R0 round 1 — `mt` two-verb design and §3b (the string form)

Artifact: `design/SPEC_mt_v0_1.md` @ `1e74d4b`
Lens: **is the two-verb design coherent, and is §3b sound enough to build from?**
Scope correction received mid-review and applied: plate layout for the string
form (§3b's "How many chunks fit a plate", both its tables, §10.11 in all parts,
and hand-engraving-as-plate-layout) is **out of scope** and is not reported
against. What `mt` *emits* is reported against; what a user does with steel is
not.

All source claims below were read from the files cited, at the line numbers
cited, on 2026-08-22. No file outside this report was modified.

---

## Verdict

**3 Critical, 4 Important, 2 Minor.** Does not close.

| id | sev | subject |
| --- | --- | --- |
| S-1 | **Critical** | §3b's capacity arithmetic contradicts the codec it claims to reuse; the 64-chunk ceiling is stated as a number no implementation can reach, and §8.7b's refusal boundary is therefore undefined over a 344-byte band |
| S-2 | **Critical** | "a new payload in it, not a new codec" is false — `mt1` needs a new BCH target constant, a new HRP through four hardcoded sites, a new chunk-set-id derivation, and it loses the reassembler's funds-load-bearing content-id check. §10.13 answered. |
| S-3 | **Critical** | §7's threat model is silently `mt qr`-only; its pinned-fee mitigation is false for `mt string`, which carries no PSBT. Same defect class R0 round 0 already found once in §7. |
| S-4 | Important | the F-234 exemption argument in §3b is self-fulfilling, and it ships one column of a deliberately two-column design standalone without saying so |
| S-5 | Important | whether `mt string`'s *input* is a PSBT decides whether §8.1/§8.2/§8.6 can be evaluated at all; §3b makes the answer load-bearing and §10.10 files it as a CLI question |
| S-6 | Important | the emitted string's alphabet, casing and separators are unspecified — and the case dimension is a hard reject, not a correctable error, which falsifies §3b's headline claim |
| S-7 | Important | the fault-tolerance budget is 4 symbols **per chunk**, does not pool, and one over-budget chunk fails the whole set. §3b states none of it. |
| S-8 | Minor | "`mt qr` for anything" / "which has no such limit" is falsified by §8.7 |
| S-9 | Minor | `mt1` is one character from `md1`, and §7 says the two artifacts sit at opposite ends of the hazard scale |

The headline question — *is the two-verb design coherent* — is answered by **S-3
and S-4** together: the split is defensible in intent, but the spec applies the
split to §3b only. §6, §7 and §8 were written for one artifact and were not
re-read against the second. §3b is **not** sound enough to build from as written,
principally because of S-1 and S-2: the numbers that justify the verb and the
machinery that is said to implement it do not match the source.

---

### S-1 — Critical — §3b, §8.7b, §10.12

**The defect.** §3b states:

> The theoretical ceiling is **2,904 B** (64 chunks x (80 data symbols − 37 header
> bits)). Treat the chunk counts above as a **floor**: `md`'s chunker balances
> rather than fills, so a real chunk measures ~85 characters where a filled one
> would be 96.

Three separate numbers are in play, and 2,904 B is the one **no implementation
can reach**.

*(a) What the existing codec actually does.* `md-codec` sizes chunks by a hard
constant, not by filling and not by balancing:

- `md-codec/src/chunk.rs:224` — `pub const SINGLE_STRING_PAYLOAD_BIT_LIMIT: usize = 64 * 5;`
  whose doc comment (`chunk.rs:213-218`) reads: *"Per-chunk payload sizing budget
  (in payload bits) that [`split`] uses to choose the chunk count:
  `count = ceil(padded_payload_bits / 320)`."*
- `md-codec/src/chunk.rs:253-254` — `let payload_bit_count_for_sizing = payload_bytes.len() * 8;`
  `let chunks_needed = payload_bit_count_for_sizing.div_ceil(SINGLE_STRING_PAYLOAD_BIT_LIMIT);`

That is **40 payload bytes per chunk**, so the 64-chunk container holds
**2,560 B**, not 2,904 B.

*(b) What a filling chunker could reach while still reusing this reassembler.*
Not 2,904 B either. `split` writes payload in whole bytes
(`chunk.rs:286-288`, `w.write_bits(u64::from(*byte), 8)`) and `reassemble`
recovers the count as `let payload_byte_count = (symbol_aligned_bit_count - 37) / 8;`
(`chunk.rs:355`) — byte-granular by construction. With `37 + 8N ≤ 400`, `N ≤ 45`,
so a byte-granular filling chunker tops out at **2,880 B**. 2,904 B assumes
fractional-byte chunk payloads, which requires a *different reassembler* — i.e.
exactly the "new codec" §3b says `mt1` is not.

*(c) Every chunk count in §3b's "What fits" table comes from the fill model.*
All six probes compute `(n * 8).div_ceil(363)`:
`design/measurements/mt-size-probe/src/bin/envelope.rs:109`, `rcw.rs:172`,
`signed.rs:192`, `baselines.rs:110`, `psbtfinal.rs:239`, `src/main.rs:102`.
Against `md-codec`'s real `split`, replicated exactly:

| artifact | raw B | §3b says | `split()` gives |
| --- | --- | --- | --- |
| RCW `tr` key-path 1-in/1-out | 162 | 4 | **5** |
| RCW `tr` tier 4 1-in/1-out | 405 | 9 | **11** |
| RCW `tr` tier 1 1-in/1-out | 535 | 12 | **14** |
| RCW `wsh` tier 1 1-in/1-out | 742 | 17 | **19** |
| RCW `tr` tier 1 5-in/2-out | 2498 | 56 ("yes, barely") | **63** |
| RCW `wsh` tier 1 5-in/2-out | 3538 | 78 (refused) | **89** |

**Why it matters.** §8.7b is normative behaviour on a bearer instrument:

> **Over the 64-chunk container (`mt string`)** → refuse, naming the chunk count
> and the ceiling

Two implementers reading this spec produce different refusals over a 344-byte
band. A 2,600-byte transaction: §3b's table says 58 chunks — fits, engrave it;
`md-codec`'s `split` returns `Error::ChunkCountExceedsMax { needed: 65 }`
(`chunk.rs:255-258`). The brief's question — *could a transaction the spec's
table says fits actually not fit?* — yes, for every payload in
(2560, 2904]. The "floor" caveat is directionally right and quantitatively
wrong: the gap is not a balancing rounding effect, it is a fixed 320-bit
budget, and it is 12–14% on every row, not noise.

Secondary: "yes, barely" is stated of a row with **8** chunks of headroom under
the spec's own model. Under the real chunker the same row has **1**.

*Non-authoritative sketch.* State all three numbers explicitly and say which one
`mt1` adopts: 2,560 B if `mt1` reuses `split` as-is; 2,880 B if `mt1` fills
byte-granularly (still reusing the reassembler); 2,904 B only with a
bit-granular chunk payload, which is a new reassembler. Regenerate the "What
fits" table from whichever is chosen, and make §10.12 a decision that gates
§8.7b rather than an optimisation.

---

### S-2 — Critical — §3b, §2, §10.13

**The defect.** §3b's load-bearing claim is:

> **The machinery exists and is proven; `mt1` is a new payload in it, not a new
> codec.**

Read against `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/`,
the 37-bit header *layout* is reusable and the ordering/gap machinery is
genuinely payload-agnostic (see "§10.13 answered" below). Four other things are
not, and the spec specifies none of them.

**(a) `mt1` needs its own BCH target residue, and the spec names none.** The
checksum primitives take `hrp` as a parameter but XOR a hardcoded md constant:

- `md-codec/src/bch.rs:86-90` — `pub fn bch_create_checksum_regular(hrp: &str, data: &[u8]) -> [u8; 13]` … `let polymod = polymod_run(&input) ^ MD_REGULAR_CONST;`
- `md-codec/src/bch.rs:100-105` — `pub fn bch_verify_regular(hrp: &str, …) -> bool` … `polymod_run(&input) == MD_REGULAR_CONST`

The constellation's documented rule, from `mk-codec/src/string_layer/bch.rs:200-202`:

> Domain separation is carried by the per-HRP target constants
> (`MK_REGULAR_CONST` / `MK_LONG_CONST`) + the HRP — never by this init.

Each format derives its constant from its own NUMS domain string —
`b"shibbolethnums"` for md1 (`md-codec/src/bch.rs:118-130`, drift-guarded) and
`pub const NUMS_DOMAIN: &[u8] = b"shibbolethnumskey"` for mk1
(`mk-codec/src/consts.rs:15-18`). **`mt1` has no domain string and no
`MT_REGULAR_CONST` anywhere in the spec.** Without one, two implementers emit
mutually unverifiable `mt1` strings — the single most normative value in the
format. If an implementer instead reuses `MD_REGULAR_CONST` because §3b told
them the codec is unchanged, the two-part separation above silently degrades to
HRP-only, and nothing in the repo would catch it.

**(b) the HRP is hardcoded, not a parameter, in the chunk layer.** Four sites:
`codex32.rs:15` (`const HRP: &str = "md";`), `chunk.rs:453`
(`const HRP_PREFIX: &str = "md1";`), and `crate::bch::hrp_expand("md")` at
`chunk.rs:565` and `chunk.rs:615`. `md-codec`'s chunk layer is not generic over
HRP the way `mk-codec`'s is; adopting it for `mt` is a refactor of the primary
Rust codec, which under the Rust-primary rule is work in `descriptor-mnemonic`,
not in `mt`.

**(c) the 20-bit chunk-set-id has no transaction-shaped derivation.** `split`
computes it from the *descriptor*:

- `chunk.rs:248-249` — `let md1_id = compute_md1_encoding_id(d)?;` `let chunk_set_id = derive_chunk_set_id(&md1_id);`
- `chunk.rs:176-180` — top 20 bits of the 16-byte `Md1EncodingId`.

A raw transaction has no `Md1EncodingId`. §3b does not say what supplies `mt1`'s
chunk-set-id. §10.13 correctly names "chunk-set id" as the open item; §3b
asserts the opposite in the same document.

**(d) and this is the funds-load-bearing one — the check that makes the 20-bit
field safe has no `mt1` analogue.** Reassembly step 7:

```
chunk.rs:406-415
    // Cross-chunk integrity check — UNCONDITIONAL regardless of `opts`
    // (the content-id oracle; P0.2 funds-load-bearing invariant).
    let md1_id = compute_md1_encoding_id(&descriptor)?;
    let derived_csid = derive_chunk_set_id(&md1_id);
    if derived_csid != expected_csid { return Err(Error::ChunkSetIdMismatch { … }) }
```

Twenty bits of chunk-set-id alone cannot stop a chunk from a *different* payload
splicing into a set; the re-derivation from the **decoded object** is what does.
`mt1` as specified inherits the header field and not the oracle. A hand-cut set
where one chunk is transcribed from the wrong plate — plausible for a 63-chunk
job — would reassemble into a byte-sequence with nothing structural to reject it.

**(e) the whole reassembly path is descriptor-typed.** `reassemble`,
`reassemble_with_opts` and `decode_with_correction` all return
`Result<Descriptor, …>` (`chunk.rs:311`, `:328`, `:655`) and route through
`decode_payload_with_opts` (`chunk.rs:404`). "A new payload in it" understates
what has to change.

**(f) the version field and the auto-dispatch bit are unspecified for `mt1`.**
`ChunkHeader::read` rejects any version ≠ `WF_REDESIGN_VERSION` (= 4)
(`chunk.rs:69-72`, `header.rs:27`). Auto-dispatch reads bit 0 of the first 5-bit
symbol as the chunked flag (`chunk.rs:650-652`), which only works because the
single-payload header places version's LSB there and 4 is even (`header.rs:31`).
The spec does not say what version `mt1` writes, nor whether `mt1` has a
single-payload (unchunked) form at all — note that the smallest real signed
transaction exceeds the 400-bit single-string cap, so `mt1` may never need one,
but that is a decision, not a silence.

*Non-authoritative sketch.* §3b should enumerate what `mt1` adds — NUMS domain
string, `MT_REGULAR_CONST`, HRP, chunk-set-id derivation (a truncated hash of
the raw transaction is the obvious candidate and gives step 7 back verbatim),
version value — and §2 should stop claiming the string layer is taken
unchanged. The step-7 analogue matters most: without it, `mt1` should at minimum
require the decoder to re-run §8.2's consensus check on the reassembled
transaction before returning it.

---

### S-3 — Critical — §7, §3b, §6

**The defect.** §7 is written for "an `mt` plate", singular, and its rows are
`mt qr` rows. One is now false. §7's pinned-fee row:

> **Pinned fee** — a 2026 fee rate may be unbroadcastable in 2040 | **cannot be
> fixed, and is NOT on the plate.** … a holder in 2040 recovers them by
> decoding, since **the PSBT carries the input amounts**

There is no PSBT on an `mt string` artifact. §3b: *"The payload is the raw
signed transaction, NOT the PSBT."* So for that verb the fee, the input values
and the input scriptPubKeys are **not** recoverable by decoding — ever, from that
artifact alone.

§6 states the same property in general terms and was likewise not re-read:

> A signed transaction references inputs as outpoints only; the source
> scriptPubKeys live in the *previous* transactions. Without them you cannot
> tell which wallet it spends. The finalized PSBT closes part of this by
> carrying each input's UTXO record — value and scriptPubKey — so the engraved
> payload does describe what it spends, **which the bare raw transaction did
> not**.

§3b then chooses "the bare raw transaction", on size grounds, without noting
that §6's closure is thereby reopened for that verb.

**Why it matters.** This is the exact defect class R0 round 0 already caught once
in this same section, recorded in §5:

> **§7's mitigations were written against the ten-field legend and were not
> re-read when it became five.** … **A diff falsifies text it never touches**

The fold fixed §7 against the *legend* change and then introduced a *second*
artifact class without re-reading §7 against it. §7 is the threat model of a
bearer instrument; a row that says a hazard is recoverable when it is not is
worse than a row that says "not mitigated", by §7's own stated standard.

Secondary consequence, and it is not stated anywhere: the two payloads are not
two encodings of one artifact. The string payload is derivable from the QR
payload (`extract_tx`); the QR payload is **not** derivable from the string
payload — the UTXO records are gone. §0's table presents the two verbs as
symmetric renderings of "a signed Bitcoin transaction"; they are not, and the
string form is strictly the lossy one.

*Non-authoritative sketch.* Give §7 a per-verb column, or state at its head which
verb each row describes. The pinned-fee and silent-invalidation rows need
separate `mt string` text.

---

### S-4 — Important — §3b

**The defect.** §3b's justification for the payload asymmetry:

> F-234 binds the *QR*, because the QR is the escape hatch for a recoverer
> holding no `mt`-aware software; it must therefore carry a form the wider
> ecosystem reads, which is `ur:psbt`. An `mt1` string is the opposite case:
> **nothing but `mt`-aware software will ever parse it**, so F-234's argument
> does not apply and size is what matters.

Two problems.

**(1) The premise is self-fulfilling.** "Nothing but `mt`-aware software will
ever parse it" is a *consequence* of choosing `mt1`, not an independent fact
about the world. The identical sentence licenses any bespoke encoding: pick a
proprietary format, observe that nothing else reads it, conclude the
interoperability requirement does not apply. F-234's cost row is a price the
design pays, not a permission to stop paying it.

**(2) It ships one column of a deliberately two-column design, standalone.**
F-234's own text (`design/FOLLOWUPS.md:9594-9605`) is not "QRs must be standard";
it is a **pairing**:

> **The principle.** A plate should carry two representations with two different
> audiences and two different failure modes:
>
> | | engraved TEXT | engraved QR |
> | needs constellation knowledge? | **yes** | **no** |
> | survives a dead decoder? | yes | no |
> | survives a scratched plate? | degrades gracefully | cliff |

F-234 already grants that the codex32 text column needs constellation knowledge
— because the QR column beside it does not. The recovery property comes from the
pair. `mt qr` alone is the column that dies on a scratched finder pattern;
`mt string` alone is the column that dies with the tooling. **Neither verb alone
delivers what F-234 designed**, and the spec presents them as alternatives:
§0's table lists two rows, and §3b closes *"`mt string` for short transactions,
`mt qr` for anything."* Nowhere does the spec say a transaction may or should be
engraved both ways, nor state the consequence that choosing `mt string` opts that
transaction out of F-234's guarantee entirely.

**The risk also runs backwards.** §1.1b says the string verb exists because
otherwise `mt` is *"unusable for anyone without a SeedHammer"*. The string user
is by construction the one with *less* infrastructure — and they are the one
handed the representation that requires a v0.1 tool, in a repository that does
not yet exist (decision 2), to still exist when the plate is read.

I stop short of Critical because F-234 as recorded is a directive about **QR
content** ("never let an `md1`/`mk1`/`ms1` string become QR content") and
`mt string` emits no QR, so nothing is literally violated. If the operator reads
F-234 as a guarantee binding every `mt` artifact rather than every `mt` QR, this
is Critical and §3b's exemption has to be withdrawn rather than reworded.

*Non-authoritative sketch.* The honest form of §3b's paragraph is not "F-234 does
not apply" but "F-234's recovery guarantee is **traded away** for this verb, in
exchange for hand-engravability and BCH transcription correction, on the
operator's ruling" — and then either say the two verbs may be combined on one
plate set, or say plainly that they may not.

---

### S-5 — Important — §3b, §8, §10.10

**The defect.** §0 says *"§8 — the refusals — carries the entire safety
argument."* §8 says *"Every refusal below binds BOTH verbs unless it names
one."* But three of the nine refusals are stated over PSBT structures:

- §8.1 — *"Every input must carry a populated `PSBT_IN_FINAL_SCRIPTSIG` or `PSBT_IN_FINAL_SCRIPTWITNESS`"*
- §8.2 — *"A PSBT whose UTXO records are missing is refused under (1)'s sibling rule: `mt` requires the MIN form of §3."*
- §8.6 — *"nothing in a legacy sighash commits to the input amount, so the PSBT's UTXO record for it is unverifiable"*

§3b makes the string form's *payload* a raw transaction; §10.10 leaves the
*input* convention entirely open (*"file? stdin? PSBT or raw hex or both?"*). The
two questions are now coupled, and the spec never couples them:

- if `mt string` accepts a raw signed transaction as input, §8.1, §8.2 and §8.6
  have nothing to read, and the safety argument does not bind the artifact that
  is hand-cut and permanent;
- if `mt string` always requires a PSBT input and merely *serialises* the
  extracted transaction, then §3b's "+58 to +61 bytes per input" is a
  serialisation saving, not an input choice, and §3b should say so — because as
  written it reads as an input choice.

**Why it matters.** §10.10 is filed as a CLI-surface gap ("nothing specifies the
input convention"). After §3b it is also a safety gap, and it is not recorded as
one. I am not auditing the refusals themselves — that is another reviewer's lane
— only reporting that §3b's payload decision silently made §10.10 load-bearing.

---

### S-6 — Important — §3b

**The defect.** §3b specifies nothing about the characters `mt string` emits.
Three concrete gaps, all with a wrong-by-default answer:

**Alphabet.** `qpzry9x8gf2tvdw0s3jn54khce6mua7l` (`chunk.rs:450`,
`codex32.rs` via `char_to_symbol`) — no `b`, `i`, `o`, or `1` outside the HRP
separator. This is a genuinely good property for a hand-cut artifact and §3b
never states it.

**Case, and this one falsifies §3b's headline claim.** Both decode paths reject
**mixed case** outright, *before* any correction is attempted:

- `codex32.rs:142-147` (strict path) — *"BIP-173: reject mixed-case input"*
- `chunk.rs:464-468` (correcting path), with the in-source rationale at
  `chunk.rs:459-463`: *"a case-flip is a zero-symbol-error event never in the BCH
  channel; a wholesale mixed-case string is a malformed encoding, not noise to
  correct."*

So a single stray capital in an otherwise-lowercase engraved string is **not** a
correctable one-symbol error — it is a hard reject of that chunk, which under
S-7's atomicity fails the entire set. §3b claims:

> A hand engraver who cuts a character wrong gets it corrected rather than
> discovering years later that the plate is scrap.

False for the case dimension. And the collision is live rather than theoretical:
everything the fork engraves it uppercases first (`backup/backup.go:49`, `:76`,
`:110`, `:150`, `:216`), while §5's legend is written in caps and `md-codec`
re-emits corrected strings in lowercase (`encode_chunk_string`,
`chunk.rs:496-503`) and reports `CorrectionDetail.was`/`.now` as lowercase
characters (`chunk.rs:606-608`). A tool telling an engraver "change `q` to `p`"
would be speaking a different case from the plate in front of them. All-upper
and all-lower both decode (`char_to_symbol` lowercases, `codex32.rs:72-78`), so
either is fine — the spec just has to pick one and say the corrector reports in
it.

**Separators.** Both paths silently strip whitespace and `-` *inside the data
part* but not inside the HRP (`codex32.rs:159-162`, `chunk.rs:477-479`). So `mt1`
strings may be broken up for legibility, which is exactly what a hand-cut string
wants — and §3b neither says the tolerance exists, nor says whether `mt` emits
separators, nor at what interval. Two implementers emit visually different
strings for the same transaction.

---

### S-7 — Important — §3b

**The defect.** §3b sells the verb on fault tolerance and states no budget:

> `mt string` emits a character string with **BCH error correction**, so a hand
> engraver who cuts a character wrong can still recover the transaction.

From source, the shape of that tolerance is:

- **4 symbols per chunk, maximum.** `bch_decode.rs:410` — *"`None` if the pattern
  is uncorrectable (> t = 4 errors)"*; `bch_decode.rs:433` — *"> 4 errors is above
  the BCH(93, 80, 8) / t = 4 capacity."*
- **It does not pool across chunks.** Each chunk is its own codeword; four errors
  in one chunk is the limit even if every other chunk is clean.
- **Failure is all-or-nothing across the whole set.** `chunk.rs:11-13` — *"Atomic
  per plan §1 D28: any chunk exceeding the BCH `t = 4` capacity fails the whole
  call without partial output."* One over-budget chunk returns
  `Error::TooManyErrors { chunk_index, … }` and the transaction does not decode,
  with no partial output to work from.
- **A missing chunk is total loss.** `Error::ChunkSetIncomplete { got, expected }`
  (`chunk.rs:378-383`). Consistent with decision 8 (zero redundancy), but §3b
  never says it, and the string form is where a human is most likely to lose one.

**Why it matters.** Corrected for S-1, a 2,498-byte transaction is 63 chunks and
roughly 5,500 engraved characters. "Four per chunk, non-pooling, atomic" is the
number that decides whether a person should start that job, and it is the number
§3b's justification for the verb rests on. Stating it is also what makes the
verb's scope ruling ("short transactions") mean something quantitative.

---

### S-8 — Minor — §3b, §8.7b, §0

§3b closes *"`mt string` for short transactions, `mt qr` for anything"*, and
§8.7b tells the refused operator to use *"`mt qr`, which has no such limit."*
§8.7 refuses `mt qr` *"Over the plate budget"*, and §0's table gives `mt qr`'s
size limit as "the plate budget". `mt qr` has a limit. As written, §8.7b's
refusal message promises an unbounded escape hatch, and an operator who has just
been refused at 65 chunks can be refused again.

---

### S-9 — Minor — §3b, §7

`mt1` differs from `md1` in one character. §7 places the two artifacts at
opposite ends of the constellation's hazard scale — *"`md1` and `mk1` are
watch-only public material: losing one costs privacy, not money … An `mt` plate
is spendable by whoever holds it"* — and the HRP is the only in-band signal that
distinguishes them. The machine layer separates correctly (per-HRP expansion plus
the per-format target constant of S-2a), so this is a human-handling concern
only, and plate layout is out of scope. Recording it because the HRP string is
itself a normative codec-level choice that §3b fixes without discussion, and
because a two-character HRP with more visual distance was available for free at
this point in the design.

---

## §10.13 answered

> **Does `mt1` reuse the `md1` chunk header verbatim, or need its own?** §3b
> assumes the existing string layer takes a new payload type cleanly. That is an
> assumption about `md-codec`'s header (37 header bits, chunk-set id, ordering)
> and has not been checked against a transaction-shaped payload.

Checked, against `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/`.
**The header layout and the ordering machinery take a transaction cleanly. The
identity, the checksum domain and the reassembly contract do not.** §10.13 should
close as *answered, and the answer is "needs its own"* — not as a question.

**Reusable verbatim, payload-agnostic — verified:**

| item | source | finding |
| --- | --- | --- |
| header is exactly 37 bits | `chunk.rs:52-56`, asserted at `chunk.rs:104-105` (`assert_eq!(w.bit_len(), 37)`) | **spec is correct.** 4 version + 1 chunked + 20 csid + 6 count−1 + 6 index |
| 64-chunk ceiling is real | `chunk.rs:38` `if !(1..=64).contains(&(self.count as u32))`, 6-bit count−1 field | **spec is correct** that the container is 64 chunks (but see S-1 on what 64 chunks *hold*) |
| ordering is in-band and order-independent | `chunk.rs:386-394` — sort by index, then reject any gap with `ChunkIndexGap` | chunks may be presented in any order; a duplicated index is caught by the gap check |
| a missing chunk is detected | `chunk.rs:378-383` `ChunkSetIncomplete { got, expected }` — `count` rides in every header | a decoder always knows how many it should have, from any single chunk |
| chunks from two sets are detected | `chunk.rs:370-377` `ChunkSetInconsistent` on differing version/csid/count | |
| payload length is recovered exactly | `chunk.rs:355` `(symbol_aligned_bit_count - 37) / 8` | byte-granular; a raw transaction is byte-aligned by nature, so md1's TLV-rollback tolerance is not even needed |
| the BCH corrector exists as §3b describes | `bch_decode.rs:1` "Syndrome-based BCH decoder", `:212-218` `berlekamp_massey`, `:75-93` `Gf1024`, `:286` `chien_search`, `:314` Forney; `lib.rs:48` re-exports `CorrectionDetail, decode_with_correction` | **every §3b citation resolves and is true**, including `md-codec/src/lib.rs:48` |
| over-length codewords fail closed | `chunk.rs:557-563`, `codex32.rs:174-179` | β has order 93, so >93-symbol words are rejected before correction rather than mis-corrected |

**Not reusable — needs `mt1`-specific specification (all of S-2):**

1. **BCH target residue.** `MD_REGULAR_CONST` is hardcoded into
   `bch_create_checksum_regular` and `bch_verify_regular` (`bch.rs:90`, `:105`)
   despite both taking `hrp`. Per-format NUMS domain strings exist
   (`b"shibbolethnums"`, `b"shibbolethnumskey"`); there is no `mt` one.
2. **HRP.** Hardcoded at `codex32.rs:15`, `chunk.rs:453`, and
   `hrp_expand("md")` at `chunk.rs:565` and `chunk.rs:615`.
3. **Chunk-set-id derivation.** `derive_chunk_set_id(compute_md1_encoding_id(d))`
   (`chunk.rs:176-180`, `:248-249`) — a descriptor hash. No transaction analogue
   is defined.
4. **The step-7 content-id oracle.** `chunk.rs:406-415`, described in-source as
   *"the content-id oracle; P0.2 funds-load-bearing invariant"* and enforced
   *"UNCONDITIONAL regardless of `opts`"*. This is the check that makes a 20-bit
   chunk-set-id safe, and `mt1` as specified has no equivalent.
5. **Return type / decode contract.** `reassemble`, `reassemble_with_opts` and
   `decode_with_correction` are `Descriptor`-typed end to end (`chunk.rs:311`,
   `:328`, `:404`, `:655`).
6. **Version value and the auto-dispatch bit.** `WF_REDESIGN_VERSION = 4` is
   enforced on read (`chunk.rs:69-72`); dispatch depends on bit 0 of the first
   symbol (`chunk.rs:650-652`, `header.rs:31`). Unspecified for `mt1`.
7. **Chunk sizing.** `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 320` (`chunk.rs:224`) —
   40 bytes per chunk, which is the whole of S-1.

**Nothing in the chunk layer assumes a descriptor-shaped payload at the *bit*
level** — the payload is opaque bytes between the header and the checksum, and a
transaction violates nothing there. Every incompatibility found is at the
*identity* layer (what names the set, what proves the set reassembled correctly)
and at the *domain-separation* layer (which HRP, which target constant). That is
the correct one-line answer for §3b to replace its current claim with.

---

## Method note

`md-codec`'s `split` was not executed against a transaction — it takes a
`Descriptor`, so no transaction can be fed to it without the changes S-2
describes. The chunk counts in S-1 were produced by replicating `split`'s
arithmetic exactly (`count = ceil(len·8 / 320)`, `bytes_per_chunk =
ceil(len / count)`, per-chunk `37 + 8N` bits) from `chunk.rs:253-293`, and
cross-checked against the fill model the probes use. Both models are pure
arithmetic over `payload_bytes.len()`, so this is a derivation rather than an
execution — worth one `cargo test` in `mt-codec` once it exists, and worth
pinning as a vector.
