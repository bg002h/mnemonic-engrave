# R5 — `mt` v0.1 spec, pre-implementation readiness: both layers

Artifact: `design/SPEC_mt_v0_1.md` at **`322bbb5`** (1,893 lines, read in full).
`git log -1` confirmed `322bbb5f7ebc04dca3ef95b92c3be86f2a2bd026`, working tree
clean; the spec did not move during this round.

Scope: codec/format **and** CLI/IO/transport. Question answered: *an implementer
starts building tomorrow — where must they still GUESS, and would two
implementers produce plates that cannot read each other?*

Prior rounds read first: `mt-spec-R4-codec-assumptions.md` (A-1…A-10) and
`mt-spec-R4-cli-assumptions.md` (B-1…B-23), both folded at `01b280d`;
`322bbb5` settled `MT_REGULAR_CONST`.

### What is machine-verified in this report

Run, not inferred, and never read from a doc comment:

- `SHA-256("shibbolethnumstransaction")` = `d17e43bfca946be09034ac97e7950cdd50d3b5a3e3cf4bad5cb65516897978f6`;
  top 65 bits = `0x1a2fc877f9528d7c1`; bit length exactly 65; distinct from
  `md1`'s `0x815c07747a3392e7` and `mk1`'s `0x1062435f91072fa5c`. **§10.13(a) is
  exactly right.**
- `descriptor-mnemonic/crates/md-codec/src/bch.rs`: `bch_create_checksum_regular(hrp, data)`
  calls `hrp_expand(hrp)`; `POLYMOD_INIT = 0x23181b3`; `GEN_REGULAR` is five
  65-bit coefficients. I **recomputed** the polymod fold of `hrp_expand("ms")`
  from init `1` and got `0x23181b3` — the doc comment's claim is true.
- `md-codec/src/codex32.rs:15` `const HRP: &str = "md"`; `chunk.rs:453`
  `const HRP_PREFIX: &str = "md1"`. Two different strings.
- `md-codec/src/chunk.rs`: `write()` validates `(1..=64)`, writes `count-1` in 6
  bits, `index` plain; `read()` refuses `chunked == 0`; `split()` at `:267`
  computes `bytes_per_chunk = payload_bytes.len().div_ceil(count)` and slices
  sequentially; `reassemble_with_opts` recovers length as
  `(symbol_aligned_bit_count - 37) / 8`, does **no** de-duplication.
- `design/measurements/mt-size-probe/src/bin/select.rs:248-281` — the model
  behind §4's table: `chunks = raw.div_ceil(40)`, `bin_bytes = raw + chunks*5`,
  `b32_chars = (bin_bytes * 8).div_ceil(5)` over the **whole stream**.
- `41 + 320 = 361` bits → **73** data symbols ≤ `REGULAR_DATA_SYMBOLS_MAX = 80`;
  codeword `86 ≤ 93`. The widened header still fits, with 7 symbols of margin.
- Balancing divergence computed on §3b's own payloads (see R-5).
- Framing divergence computed for the 3,809 B artifact: **7,054** characters
  (per-chunk base32) vs **7,016** (whole-stream base32); both under 8,191.
- `grep -n -- "--[a-z]"` over the spec: **1** hit, still the *deleted* locktime
  pair. `grep -ci "timeout"`: **0**. `grep -io "rpc[a-z]*\|cookie"`: one hit,
  the word `RPC` in §6a prose.

I recomputed nothing that `design/measurements/` establishes, and did not
re-check the citation or structure gates.

---

## Verdict

| severity | count | items |
| --- | --- | --- |
| **Critical** — plates mutually unreadable or the verb unbuildable | **3** | R-1, R-2, R-3 |
| **Important** | **16** | R-4 … R-19 |
| **Minor** | **4** | R-20 … R-23 |
| **Nit** | **1** | R-24 |
| **total** | **24** | |

**The fold was good on the codec's constants and bad on its geometry.** Every
R4 finding that could be closed by naming a *value* — the NUMS domain, the
version, `count-1`, the `chunked` bit, bit order, the content id's end — is
closed, correctly, and I verified the arithmetic. What is not closed is
everything about **shape**: how a chunk becomes characters, how characters
become symbols, and what a record contains. §10.13 is titled *"RULED, ready to
build"* and `mt string` genuinely nearly is; **`mt qr` cannot be built at all**,
and the three Criticals are all on that side.

**The single highest-value finding is R-2**, and it is not a wording fix.
§10.8's normative *"every engraved symbol carries its own `n/m` … for the chunk
it holds"* and §4's measured *"5 plates, 4 qr"* for the 96-chunk artifact
describe **two incompatible artifacts**. One of them has to give, and whichever
gives moves numbers the spec currently presents as settled.

**R-1 is the cheapest Critical in the report**: R4 filed it as a Minor (A-9 ii),
the fold did not touch it, and it is one wrong word — §10.13(b) says the HRP is
`mt1` where the checksum wants `mt`. Two implementers who read that sentence
differently cut plates neither can verify.

---

## R4 disposition

### R4 codec lens (A-1 … A-10)

| # | R4 severity | verdict at `322bbb5` |
| --- | --- | --- |
| **A-1** NUMS constant not derivable | Critical | **CLOSED, verified.** §10.13(a) names `"shibbolethnumstransaction"` and `0x1a2fc877f9528d7c1`; I reproduced the digest and the shift independently. Residual (Minor, R-23): the drift-guard test and the `!=` assertion against `md1`/`mk1` that both siblings carry are not required anywhere in the spec |
| **A-2** `count` stored as `count-1`? | Critical | **CLOSED.** §10.13(a2): *"`count - 1`, matching `md-codec`'s offset convention: a set of 1 stores `0`, a set of 256 stores `255`"*, with the `index < count` predicate kept. Matches `chunk.rs:55,79` |
| **A-3** 4-bit `version` unassigned | Critical | **CLOSED.** §10.13(a2): `0b0001`. Distinct from `md-codec`'s `WF_REDESIGN_VERSION = 4`. Residual (Nit, R-24): the stated *reason* is false — with distinct NUMS constants a shared version could not let one format's chunk verify as the other's; the decision is right, the justification is not |
| **A-4** last chunk's length has no signal | Critical | **HALF-CLOSED, and the `mt qr` half got worse.** The `mt string` identity is still nowhere in the spec (→ **R-6**), and the new *"byte boundary (`mt qr`)"* clause opens an 11-bit padding window that breaks the identity outright (→ **R-6**) |
| **A-5** "flat 40 bytes" mis-describes the chunker | Important | **HALF-CLOSED.** §3b now retracts it — but **§11 still asserts it** (→ **R-4**), and §3b's replacement sentence describes a *different* algorithm from the line it cites (→ **R-5**) |
| **A-6** two chunk sets share one content id | Important | **OPEN, unchanged.** Still no discriminator, and the fold spent the one free bit on retention instead (→ **R-8**) |
| **A-7** `chunked` bit has no function or value | Important | **CLOSED.** §10.13(a2): *"`1`, always, and RETAINED"*, with the bit-shift hazard spelled out. Residual (Minor, R-23): what a decoder does on `0` is unstated |
| **A-8** reassembly semantics unwritten | Important | **OPEN, unchanged.** `grep` for duplicate/out-of-order/gap over the spec returns nothing (→ **R-7**) |
| **A-9(i)** bit order / symbol padding | Minor | **CLOSED for order** — §10.13(a2) states MSB-first, and *"no padding between"* header and payload, which matches `split()`. Residual (Minor, R-23): the padding **value** (zero) and whether canonical form is enforced on decode are unstated |
| **A-9(ii)** `hrp_expand("mt")` not `"mt1"` | Minor | **OPEN, unchanged — and I am raising it to Critical** (→ **R-1**). §10.13(b) still reads *"Its own HRP, `mt1`"* |
| **A-10** record the 73/80 margin and the identity | Nit | **OPEN.** Neither number is in the spec. I verified both (73 ≤ 80, codeword 86 ≤ 93) — folding them costs two sentences (→ **R-6**, **R-23**) |

### R4 CLI lens (B-1 … B-23)

| # | R4 severity | verdict at `322bbb5` |
| --- | --- | --- |
| **B-1** zero flags, seven operator inputs | Critical | **HALF-CLOSED.** §10.10 now carries the eight-row input table and states outright that §8.7 *"cannot run"* without a plate budget. Flag spellings remain absent by the spec's own admission (→ **R-9**) |
| **B-2** `sysw` record framing, four ceilings | Critical | **OPEN, ACKNOWLEDGED.** §10.9 and §8.7c both now name it as the prerequisite and refuse to state a threshold. **Confirmed genuine** — see **R-3** for the enumeration the brief asks for |
| **B-3** §4's config and §5's legend have no channel | Critical | **OPEN**, folded under §10.17 as firmware work — which hides the channel decision rather than deciding it (→ **R-14**) |
| **B-4** what `mt qr` writes, and where | Critical | **OPEN, unchanged** (→ **R-10**) |
| **B-5** node location, credentials, timeout | Critical | **OPEN, unchanged.** Zero hits for timeout/cookie/rpc-url (→ **R-11**) |
| **B-6** engraved `~<year>` depends on the network | Critical | **CLOSED.** §8.4 now rules the embedded constant *"and ONLY the embedded constant"*, with `MT_REF_HEIGHT`/`MT_REF_TIME`, MTP-not-header justification, and block provenance. This is the cleanest fold in the round |
| **B-7** refusal format; `7c` before `7b`; §8.1 ≡ §8.3 | Important | **OPEN, unchanged.** Both defects still present at lines 774/950 and 1223/1254 (→ **R-12**) |
| **B-8** exit codes | Important | **OPEN**, spec says so (→ **R-12**) |
| **B-9** input encoding, file vs stdin | Important | **OPEN, unchanged** (→ **R-13**) |
| **B-10** §8.9 "Secrets → refuse" has no subject | Important | **OPEN, unchanged** — still one sentence (→ **R-18**) |
| **B-11** success report format / change row / provenance | Important | **OPEN, unchanged** (→ **R-20**) |
| **B-12** negative block delta | Important | **CLOSED**, ruled: warn, and the legend drops the `~<year>`. But the ruling **collides with §5 on the legend string** and leaves the timestamp analogue unruled (→ **R-15**) |
| **B-13** §4's tie-break not a total order | Important | **OPEN, unchanged** (→ **R-16**) |
| **B-14** `Class::is_secret()` | Important | **OPEN, unchanged** (→ **R-19**) |
| **B-15** `me convert` on an `mt1` | Important | **OPEN, unchanged** (→ **R-19**) |
| **B-16** reference-constant refresh cadence | Minor | **PARTLY.** §8.4 now carries block provenance for whoever refreshes it; no cadence, no staleness signal (→ **R-23**) |
| **B-17** crate dependency direction / release ordering | Minor | **OPEN, unchanged** (→ **R-23**) |
| **B-18** module-size domain, "at the point of choice" | Minor | **OPEN, unchanged** (→ **R-16**) |
| **B-19** `--input-value` name, units, override | Minor | **OPEN** (→ **R-9**) |
| **B-20** `mt string` stdout framing, bearer text | Minor | **OPEN, unchanged** (→ **R-21**) |
| **B-21** legend rendering below the field list | Minor | **OPEN, unchanged** (→ **R-22**) |
| **B-22** severity markers / tool prefix | Nit | **OPEN** (→ **R-24**) |
| **B-23** passphrase on an `mt qr` payload | Nit | **OPEN** (→ **R-24**) |

**Score: of R4's 10 Criticals, 5 are closed (A-1, A-2, A-3, A-7 promoted to
closed, B-6), 2 are half-closed (A-4, B-1), 3 remain open (B-2 acknowledged,
B-3, B-4, B-5 — B-2 deliberately).** Of 13 Importants, 2 are closed (B-12
partially, plus A-7), the rest stand.

---

### R-1 — The BCH checksum construction is pinned only by its target constant. §10.13(b) names the HRP as `mt1` where the checksum wants `mt`.

**Severity: Critical. Section: §10.13(b), §3b.**

**What the spec says.** §10.13(a) gives `MT_REGULAR_CONST` and its derivation.
§10.13(b), in full: *"**Its own HRP**, `mt1`, currently hardcoded at four sites
in `md-codec`."* §3b adds that the machinery is *"the `BCH(93,80,8)`
regular-code variant of BIP-93"* and points at `bch_decode.rs`. Nothing else in
1,893 lines describes the checksum.

**What is actually true of `md-codec`** (read from source, not doc comments):

| element | value | site |
| --- | --- | --- |
| checksum input | `hrp_expand(hrp) ++ data ++ [0;13]` | `bch.rs:86-89` |
| the `hrp` passed | **`"md"`** | `codex32.rs:15` |
| the printed prefix | **`"md1"`** — BIP-173's separator, not part of the domain | `chunk.rs:453` |
| `POLYMOD_INIT` | **`0x23181b3`** | `bch.rs:43` |
| generator | `GEN_REGULAR`, five 65-bit coefficients | `bch.rs:7-13` |
| target XOR | `MD_REGULAR_CONST` | `bch.rs:90,104` |

**And `POLYMOD_INIT` is not the obvious value.** I recomputed it: `0x23181b3`
is *exactly* the polymod fold of `hrp_expand("ms")` starting from `1`. So
`md1`'s checksum domain is, in effect, `hrp_expand("ms") ++ hrp_expand("md") ++
data` — a double HRP expansion, an artifact of codex32's `ms32` lineage.
`bch.rs`'s own doc comment records that **this constellation already got this
question wrong once in writing**: *"Earlier notes here claiming md1
'deliberately deviates from codex32's init `1`' … were both wrong."*

**What an implementer must guess.** Three things, none derivable from the spec:
(i) the string fed to `hrp_expand` — `"mt"` or `"mt1"`; (ii) the polymod
initial residue — `1` (BIP-93 as written) or `0x23181b3` (what `md-codec`
actually seeds); (iii) whether the generator and the trailing 13 zeros are
inherited verbatim.

**Candidate guesses.**

1. `hrp_expand("mt")`, `init = 0x23181b3`, `GEN_REGULAR` verbatim — what a
   forker of `md-codec` produces, because they copy `bch.rs` and edit one
   `const`.
2. `hrp_expand("mt1")`, everything else inherited — **what §10.13(b) literally
   instructs**, and what a reader who has not opened `codex32.rs` writes.
3. `hrp_expand("mt")`, `init = 1` — what an implementer building "the
   `BCH(93,80,8)` regular-code variant of BIP-93" from the BIP writes, since
   BIP-93's published init for a bech32-style code is `1`. This is the Go
   porter's guess, and §10.13 explicitly anticipates a Go port.

**What the spec's logic implies: guess 1.** §10.13's *"WHERE THIS LANDS"* box
rules that `mt1` **forks** `md-codec`'s machinery — *"the algorithm is
constant-agnostic — the caller XORs the polymod residue against the per-HRP
target constant"* — so everything except the target constant is inherited, and
the HRP is the argument `md-codec` passes, which is `"md"` → `"mt"`. **This is
a one-word fix in §10.13(b) plus three constants written down**, and the spec's
own fork ruling already decides all four.

**The observable divergence.** A (guess 1) engraves `mt1qpzry…`; B (guess 2 or
3) computes over a different domain. Every one of B's `bch_verify_regular`
calls on A's chunk returns `false`, the syndrome is non-zero, and
`decode_regular_errors` returns `None` because the residue exceeds `t = 4`.
**The report the recoverer gets is "this plate is damaged beyond correction",
not "this plate was cut by a different tool"** — they conclude the steel
failed, on a plate that is physically perfect. Each implementation round-trips
flawlessly against itself, so neither's test suite catches it; only a
cross-implementation vector does, which is precisely the failure this
constellation already shipped once with `WalletPolicyId`.

---

### R-2 — How many `mt1` chunks does one QR symbol carry? §10.8 says one; §4's own table says twenty-four.

**Severity: Critical. Section: §10.8 vs §4, §3, §8.7c.**

**What the spec says.** §10.8, **normative**: *"every engraved symbol carries
its own human-readable `n/m` beside it, **for the chunk it holds** — independent
of, and in addition to, the plate's `PLATE n OF m`"*, and *"**A lone symbol
reads `1/1`**, which is the only way it can state that it is whole."* It prices
the labels at *"3–5 characters"* each. §4's note calls the unmodelled cost *"the
41-bit `mt1` chunk header **per symbol**"*.

**What §4's measured table says.** RCW `wsh` tier 1, 5-in/2-out: 3,809 PSBT
bytes → **5 plates, 4 qr**. §3 and §8.7c size the same artifact at **96
chunks**.

**Executed, from the probe that produced the table**
(`mt-size-probe/src/bin/select.rs:253-281`):

```
let chunks = raw.div_ceil(40);          // 96
let bin_bytes = raw + chunks * 5;       // headers byte-padded, concatenated
let b32_chars = (bin_bytes * 8).div_ceil(5);   // ONE base32 over the whole stream
```

The character stream is then packed into QR symbols by `best()`. **Chunk
boundaries and symbol boundaries are independent in the model that produced
every number in §4.** A symbol carries ~24 chunks and no whole number of them.

**What an implementer must guess.** Whether an `mt1` chunk is (i) a 40-byte
fragment of a byte stream that is sliced into symbols wherever capacity falls,
or (ii) the unit of one QR symbol.

**Candidate guesses.**

1. **40-byte chunks, symbols sliced by capacity** — §3/§8.7c/§4's arithmetic.
   Then §10.8's label has no referent: a symbol holds chunks 25–48, so the
   label must become a *range* (`25-48/96`, 7 characters, not 3–5), and *"a
   lone symbol reads `1/1`"* is only true of a transaction that fits 40 bytes.
2. **One chunk per symbol, chunk sized to the symbol** — §10.8's plain reading,
   §4's *"minimise symbol count"* objective, and the *"41-bit header per
   symbol"* note. Then the 3,809 B artifact is **4 chunks**, the header costs
   41 bits per ~1,100 bytes instead of per 40, and §3's measured **80.4%**
   bech32 efficiency (which is 91% minus exactly the 41-bits-per-320-bits
   overhead) is wrong by ~11 points.
3. **A hybrid**: 40-byte chunks for `mt string`, symbol-sized chunks for
   `mt qr`, with `count`/`index` meaning different things per verb — which
   §3's *"identical header both ways"* forbids.

**What the spec's logic implies, and it is not the one the numbers use.** The
40-byte figure is derived, in §3b's own words, from
`SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64 * 5 = 320` bits — **a property of the
codex32 container**. §3 itself applies exactly this reasoning to the other
limit: *"`mt string` keeps the **64-chunk limit** because that is a property of
the codex32 container it is engraved into (§3b), **not of the header**."* By
that same argument the 40-byte payload ceiling is a codex32 property and has no
business binding `mt qr`, which has no codex32 container — pointing at guess 2.
But guess 2 dissolves §3's stated reason for widening the header at all
(*"§3b's own table measures the largest `mt qr` artifact at 96"* — 4 chunks
needs 6 bits, not 8), and it changes §3's efficiency measurement and therefore
§4's plate table. **This is a live design question, not a wording one**, and it
is the one thing in the spec that cannot be settled by an editor.

**The observable divergence.** Same PSBT. A (guess 1) emits 96 chunks whose
concatenated base32 fills 4 symbols across 5 plates, each symbol labelled with a
range. B (guess 2) emits 4 chunks, one per symbol, each labelled `1/4`…`4/4`,
at a different QR version and ECC level because the payload is ~11% smaller.
**Neither decoder can read the other's plate**: A's expects `count = 96` and
finds `count = 4` with a 1,100-byte payload; B's expects one chunk per symbol
and finds a symbol whose first 41 bits say `index 0 of 96` followed by 24
chunks' worth of bytes it has no rule for splitting. The plates are also
*physically* different — different symbol count, different labels, possibly
different plate counts — so an operator can tell them apart by eye and still not
know which is authoritative.

---

### R-3 — The `mt qr` byte→character→record stage is unspecified, and §10.13(a2)'s "padding appears only once" is false for that verb

**Severity: Critical. Section: §10.13(a2), §3, §10.9, §8.7c.**

**What the spec says.** §10.13(a2): *"The 41-bit header is followed immediately
by the chunk payload with **no padding between them**; padding appears **only
once**, at the end of a chunk, to reach the next 5-bit symbol boundary
(`mt string`) or **byte boundary** (`mt qr`)."* §3's table chooses *"bytes +
bech32 UPPERCASE"* and rules that *"the `sysw` record stores **lowercase**, and
`mt` uppercases only when encoding the QR symbol."* §10.9 and §8.7c both name
the record framing as an unresolved prerequisite and decline to state a
threshold.

**Confirming the brief's question: yes, the record framing is genuinely the
blocker §10.9 and §8.7c describe.** But it is *narrower and wider* than they
say. Narrower, because §10.13(a2) has already ruled the one part they imply is
open — the header/payload bit layout — and it matches `split()`'s behaviour, so
that is settled. Wider, because three further decisions sit between a chunk and
a record and **none of them is named in either section**.

**What must be decided, precisely — the enumeration the brief asks for:**

| # | decision | candidates | consequence |
| --- | --- | --- | --- |
| **F1** | **base32 granularity** | per-chunk, or one conversion over the concatenated stream | changes every character after the first chunk boundary |
| **F2** | **record boundary** | one record per chunk; one record per QR symbol; one record for the whole artifact | drives F4, and drives whether a record can classify at all |
| **F3** | **record text** | bare base32 data; `mt1`-prefixed with BCH (forbidden by §3a's never-stack rule for a QR-bound artifact); a `tx:`-style tagged record | `me`'s `classify` needs an HRP before the first `1`, and the bech32 charset excludes `1` — a bare record is **unclassifiable**, which R4 executed and confirmed |
| **F4** | **who uppercases** | `mt` (but `mt` never encodes the QR — the device does); the firmware, reading a lowercase record | if nobody does, the QR falls out of alphanumeric mode into byte mode and **every plate count in §4 is wrong** |
| **F5** | **the last chunk's length rule** | `bytes = floor(chars*5/8)`, `N = bytes - 6`; or the `mt string` identity `(chars*5 - 41)/8` | the two disagree by one byte per chunk — see R-6 |
| **F6** | **the `MAX_SECTION_LEN` threshold** | falls out of F1–F3 | §8.7c's missing number |

**"Padding appears only once" cannot be true for `mt qr`.** Under §10.13(a2)
the chunk is padded to a byte boundary (41 + 8N → 48 + 8N bits), and then the
resulting bytes must be converted to base32 characters, which pads *again* to a
5-bit boundary. That is two paddings. The probe's model pads differently again
— it byte-pads the header alone (`chunks * 5` bytes for a 37-bit header) and
then runs **one** base32 over the whole stream. Three descriptions of one
pipeline, in one spec plus its own measurement crate.

**What an implementer must guess:** F1 through F5, all of them.

**What the spec's logic implies, where it implies anything.** F1: nothing
decides it. F3: §3a's *"never stack them"* forbids BCH inside a QR-bound
chunk, so the record is bare data — which is exactly the shape `classify`
cannot place, and that catch-22 is real and unresolved. F4: §3's sentence says
`mt` uppercases *"when encoding the QR symbol"*, but §10.9/§10.17 put QR
encoding on the **device**, so the sentence assigns the job to a component that
does not do it. That contradiction must be resolved before anyone writes either
side.

**The observable divergence, computed for the 3,809 B artifact** (96 chunks,
balanced 95×40 + 9, each chunk 41 bits + payload end-padded to a byte boundary
= 95×46 + 15 = 4,385 bytes):

- **F1 per-chunk base32:** 95 × 74 + 24 = **7,054 characters**.
- **F1 whole-stream base32:** `ceil(4385 × 8 / 5)` = **7,016 characters**.

Both fit under `MAX_SECTION_LEN = 8191`, so the ceiling does not adjudicate —
but the two strings **diverge from the first chunk boundary onward** and share
no character after position ~74. A's record is 38 characters longer than B's,
and B's decoder reading A's record mis-frames every chunk after the first. **A's
plate cannot be read by B's decoder, and the failure is silent** — the first
chunk parses, the header is valid, and the corruption shows up only at the
content-id compare, which reports "this is a different transaction."

---

### R-4 — §11 still asserts "a flat 40 payload bytes", which §3b now retracts. The fold edited one of the two sites R4 named.

**Severity: Important. Section: §11 vs §3b.**

**What the spec says.** §3b (folded): *"An earlier version of this box called it
'a flat 40 bytes per chunk', and that mis-describes the chunker — R4 lens 1.
`md-codec` computes `chunks_needed` against the 320-bit ceiling and then
**balances** the payload."* §11 (line 1873, unfolded): *"**That reason is now
void**: §3b's correction established that chunk sizing is **a flat 40 payload
bytes** (`crates/md-codec/src/chunk.rs:224,253-254`), so the count is *exact*
for a given payload size."*

§11 attributes to §3b the exact proposition §3b now retracts, and cites the
same two lines R4 showed were the wrong lines (the sizing happens at `:267`).

R4's A-5 named both sites explicitly: *"§3b's sentence and §11's repetition are
the two places to edit."* One was edited.

**What an implementer must guess:** which section is current. Nothing marks
§11 as stale; both are in the same document at the same status.

**Candidate guesses:** flat-40 with a short tail (§11); `div_ceil` balancing
(§3b); *"balanced at 40 B/chunk"* (§10.12's phrasing, which is a third wording
for the same thing and reads like flat-40).

**What the spec's logic implies: balance.** §10.12 is a closed operator ruling
that filling reduces error recoverability, §3b's correction is dated later and
is specific, and `md-codec` balances. **§11 is simply stale text and the fix is
to delete the clause** — but see R-5, because "balance" is itself not enough.

**The observable divergence.** A (reading §11) emits 40,40,40,40,2 for a 162 B
payload; B (reading §3b) emits 33,33,33,33,30. Both are 5 chunks, both
reassemble correctly in either decoder (chunk lengths are self-describing), so
**this is a determinism break, not a readability break** — but it invalidates
any conformance corpus pinned by one implementation, including the Go port
§10.13 explicitly anticipates, and a hand engraver who re-runs `mt string` to
check their plate gets a total character mismatch with nothing telling them
which is authoritative.

---

### R-5 — "Balances" is not an algorithm. §3b's replacement sentence describes a different split from the line it cites.

**Severity: Important. Section: §3b.**

**What the spec says.** §3b: *"`md-codec` … **balances** the payload across that
many chunks (`crates/md-codec/src/chunk.rs:267`), **so the last chunk is not a
short remainder** and no chunk is padded to 40."* And: *"**`mt1` balances
too**."*

**What `chunk.rs:267` actually does** (read from source):

```rust
let bytes_per_chunk = payload_bytes.len().div_ceil(count as usize);
```

followed by sequential `start_byte = index * bytes_per_chunk` slicing. **The
last chunk IS a short remainder** — it is merely less short than flat-40 would
make it. The fold's clarifying clause states the opposite of the code it cites,
and it is the clause an implementer will act on, because it reads like a
specification while the citation reads like provenance.

**What an implementer must guess:** the split function.

**Candidate guesses,** computed over §3b's own table:

| payload / count | `div_ceil` (what `chunk.rs:267` does) | even-spread (what "not a short remainder" describes) |
| --- | --- | --- |
| 162 B / 5 | 33, 33, 33, 33, **30** | 33, 33, **32, 32, 32** |
| 405 B / 11 | 37 ×10, **35** | 37 ×one, then 36s — tail **36** |
| 535 B / 14 | 39 ×13, **28** | 39 ×the remainder, tail **38** |
| 742 B / 19 | 40 ×18, **22** | 40, then 39s — tail **39** |

**All four differ.** A third candidate — flat 40 with a short tail — differs
again on three of them (R4's A-5 measured this).

**What the spec's logic implies: `div_ceil`**, because §10.13 rules that `mt1`
forks `md-codec` and this is what the forked line does. **The fix is to write
the formula, not the adjective** — one line of pseudocode ends the question for
both the Rust primary and the Go port.

**The observable divergence.** Two encoders emit **different codex32 strings**
for the same 535 B transaction, differing from the first chunk onward.
Reassembly still works in both directions, so nothing catches it in either test
suite; it surfaces the first time a cross-language vector is generated, which is
exactly how the `WalletPolicyId` divergence surfaced in this constellation.

---

### R-6 — Nothing states how a decoder recovers the last chunk's payload length, and the two natural rules disagree by one byte for `mt qr`

**Severity: Important. Section: §10.13(a2), absent elsewhere.**

**What the spec says.** §10.13(a2) rules where padding goes. It does not say
how a reader recovers `N`, the chunk's payload byte count — and since R-4/R-5
make chunk lengths *unequal*, a reader now needs `N` per chunk, not once.
`grep -n "MessageLen"` over the spec returns one hit, inside the retracted UR
discussion.

**What `md-codec` does** (read from source, `reassemble_with_opts`):

```rust
let payload_byte_count = (symbol_aligned_bit_count - 37) / 8;
```

with a doc comment proving it: the symbol-aligned count is in `[37+8N, 37+8N+4]`,
so the floor recovers `N` exactly. **I verified the identity survives the
widening for `mt string`:** padding to a 5-bit boundary is ≤ 4 bits, so
`(sabc - 41) / 8` floors to `N` for all `N`.

**It does NOT survive for `mt qr`.** Under §10.13(a2) an `mt qr` chunk is padded
to a **byte** boundary (up to 7 bits) and then converted to base32 characters
(up to 4 more) — a padding window of up to **11 bits, which exceeds a byte**.
Worked example, `N = 40`: `41 + 320 = 361` → byte-pad → 368 → base32 → 74
characters → 370 bits. `(370 - 41) / 8 = 41.125` → **41, not 40.** The identity
silently over-counts by one byte per chunk.

**What an implementer must guess:** the recovery rule.

**Candidate guesses.** (1) `N = floor(chars*5/8) - 6` — strip to whole bytes
first, then subtract the 6-byte header footprint. Correct for `mt qr`. (2)
`N = (bits - 41)/8` — the inherited `mt string` identity. Correct for
`mt string`, **wrong for `mt qr`**. (3) An explicit length field — the
`MessageLen` §3 removed.

**What the spec's logic implies:** (1) for `mt qr`, (2) for `mt string`, and
**the spec must say both**, because the obvious move — reuse one rule across
"one fragmentation scheme" (§3's stated goal) — is the one that breaks.

**The observable divergence.** A (guess 1) reassembles 3,809 bytes from the
96-chunk set. B (guess 2) reassembles 3,905 — one spurious byte per chunk. B's
`extract_tx` fails or yields a different transaction, the content-id compare
refuses, and the recoverer is told the plates are from a different transaction.
Because guess 2 round-trips *within* B (B's writer emits what B's reader
expects only if B also mis-pads), this can also silently produce two
self-consistent implementations whose plates are mutually unreadable.

---

### R-7 — Reassembly semantics — duplicates, gaps, id mismatch — are still unwritten, and `mt qr`'s camera makes duplicates the normal case

**Severity: Important. Section: §2 (promise), §10.13(c), absent thereafter.**

**What the spec says.** §2 lists as a thing this codec exists to specify: *"how
a recoverer reassembles them, and how they know a fragment is missing."*
§10.13(c) says only *"Reassembly re-derives the id from the transaction it
decoded and compares"* — and does not say what happens when the comparison
fails. `grep` for duplicate / out-of-order / gap over the spec returns nothing.

**Inherited behaviour if `md-codec` is forked verbatim** (read from
`reassemble_with_opts`, confirmed at source this round):

| case | inherited behaviour |
| --- | --- |
| out of order | fine — `parsed.sort_by_key(index)` |
| **duplicate chunk** | **fatal.** `parsed.len() != expected_count` → `ChunkSetIncomplete { got: 4, expected: 3 }`. There is **no de-duplication anywhere in the function** |
| duplicate + one missing | passes the length check, then `ChunkIndexGap` |
| missing chunk | `ChunkSetIncomplete { got, expected }` — **does not name which index** |
| two sets mixed | `ChunkSetInconsistent` on differing `count` / `chunk_set_id` / `version` |
| content-id mismatch | `ChunkSetIdMismatch` — unconditional, called *"the content-id oracle; funds-load-bearing invariant"* |

**Why this is not an edge case for `mt`.** `md1` chunks are transcribed by a
human from a card. **`mt qr` chunks are scanned by a camera off up to five
plates**, and re-scanning a symbol is the normal operating mode. An inherited
decoder **refuses a complete, undamaged plate set** because the operator scanned
one symbol twice.

**Candidate guesses.** (a) De-duplicate identical chunks silently, refuse
*conflicting* ones at the same index. (b) Fork verbatim and refuse. (c) Accept
the first chunk seen at each index and ignore the rest.

**What the spec's logic implies: (a).** §10.2 rules that `mt` ships its own
static-scan reader, so the codec owns the scanner's semantics, and §10.8's
purpose is *"a recoverer must be able to inventory what they hold and name what
is missing"*, which presumes re-scanning. **(c) must be explicitly excluded** —
it is the only option that can hand back a transaction nobody engraved.

**One more, and it is normative already.** §8 closes: *"Every refusal names the
number that caused it."* `ChunkSetIncomplete { got: 95, expected: 96 }` **does
not name which index is missing** — and that is the one fact a recoverer can
act on, because §10.8's per-symbol labels let them walk to the right plate.

**Objection considered and answered.** §9 rules a decoder out of v0.1, so this
looks deferrable. It is not: §10.8's label design, the `n/m` cost §4 must
reserve, and whether the encoder must emit anything extra all depend on it, and
those are cut into steel now.

**The observable divergence.** A's reader ingests a re-scanned pile and
reassembles. B's refuses it with `ChunkSetIncomplete`, telling a recoverer their
intact plate set is incomplete. Both are "correct" against the spec, because the
spec says nothing.

---

### R-8 — One transaction still produces two chunk sets sharing one `chunk_set_id`, and the fold spent the free bit rather than the gap

**Severity: Important. Section: §10.13(c) vs §3b, §10.10.**

**What the spec says.** §10.13(c): the id is *"the top 20 bits of the EXTRACTED
transaction's txid"*, display form. §3b: `mt string`'s payload is *"the raw
signed transaction, NOT the PSBT"*. §10.10: `mt qr`'s payload is the finalized
PSBT. §3: *"identical header both ways"*.

**Consequence the spec still does not draw:** the same transaction produces
**two different chunk sets carrying the same `chunk_set_id`**, because the id is
derived from the transaction and the transaction is not the payload in one of
the two cases.

**And the fold closed the cheapest exit.** R4's A-7 offered the `chunked` bit as
a payload-type discriminator — *"free and strictly better … that option expires
at first engraving."* §10.13(a2) instead retains it as a constant `1`. The
reasoning given (dropping it shifts every field by one bit) is sound and I do
not dispute the retention; but retaining it *as a constant* rather than
*as a discriminator* was a choice made without the gap in view, and it is
irreversible once a plate exists.

**What an implementer must guess:** given a reassembled byte string, which
parser to apply, and what to hash for the §10.13(c) compare — a PSBT must be
`extract_tx()`'d first, a raw transaction hashed directly.

**Candidate guesses.** (1) Sniff: a PSBT begins `70 73 62 74 ff`, a raw
transaction with a 4-byte version. (2) Infer from the medium: codex32 ⇒ raw
transaction, QR ⇒ PSBT. (3) A header bit.

**What the spec's logic implies: (2)**, because §10.10 binds payload format to
verb and a reader always knows which medium it scanned. **Say so in one
sentence, and say what reassembly hashes in each case** — that is the whole
fix, and it costs nothing.

**The observable divergence, and the safety net worth naming.** A pile mixing a
string set and a QR set of the same transaction has one `chunk_set_id`.
Measured, `ChunkSetInconsistent` saves it: the MIN-form PSBT is +58 B minimum
over the raw transaction (`RESULTS_psbt_envelope_2026-08-23.txt`), and 58 > 40,
so the two sets' `count` always differs. **That is arithmetic luck, not
design** — nothing in the spec claims it, no test pins it, and it evaporates if
R-2 resolves toward symbol-sized chunks, because then the two verbs' chunk
counts are computed by different rules entirely and may coincide.

---

### R-9 — The eight-row input table names the inputs and no way to supply them; §8.7 is still an unrunnable refusal

**Severity: Important. Section: §10.10.**

**What the spec says.** §10.10 now carries the table R4's B-1 asked for — eight
rows, each with the section that needs it and the absent-behaviour — and
concedes: *"**Still unspecified:** the flag spellings themselves, exit codes,
and the format of the refusal messages."* It also argues the concession is
cheap: *"two implementers given this table will still choose different flag
*spellings*, but they will at least build the same tool."*

**That argument is right about six rows and wrong about one.** For `FROM`,
`TO`, the label, module size, input values and node location, the table fixes
the *behaviour* and only the spelling floats. **For the plate budget it does
not**, because the table's absent-behaviour cell says *"§8.7 cannot run"* —
which is a restatement of the problem, not a decision.

**What an implementer must guess:** whether the plate budget is **required**,
and what happens when it is absent.

**Candidate guesses.** (1) Required — `mt qr` refuses without it, so §8.7
always runs. (2) Optional, defaulting to unbounded — §8.7 never fires unless
asked. (3) Optional with a built-in default (5 plates? 10?).

**What the spec's logic implies: (1).** §8.7 is a *numbered refusal* in a
section that *"carries the whole safety argument"*, and §8's closing rule is
that every refusal names its number. A refusal whose threshold is optional is
not a refusal. But nothing states it, and (2) is what a `clap` derive with
`Option<usize>` produces by default.

**Two smaller residues in the same table.** `--input-value`'s **units** (§8.2c's
examples are BTC to eight places, which implies BTC) and its **shape** (indexed
per input vs a total — the spec says *"or"*, so both); and whether an operator
value that *conflicts* with a present `witness_utxo` is an override or a
refusal. §6's *"on disagreement a recoverer would have to guess which to
believe. That is a funds-safety hazard"* implies **refuse**; nothing says it.

**The observable divergence.** Implementer A ships `--max-plates` required;
implementer B defaults it to unbounded. Same PSBT: A refuses a 6-plate job, B
starts cutting steel for two hours. Every runbook and CI fixture is
incompatible between them, and the operator cannot tell which tool they have
from the artifact.

---

### R-10 — What `mt qr` writes, in what encapsulation, and how it reaches the machine

**Severity: Important. Section: §10.10, §3b.**

**What the spec says.** §10.10: *"`mt qr` output | a **SH2 payload** (`sysw`)
carrying the QR"*, and §3b's fixed point: *"stdout carries the artifact, stderr
carries everything the human must see."*

**What an implementer must guess.** Three separable things: the
**encapsulation** (bare container, `REGION_LEN`-padded region image, or UF2);
the **destination** (binary on stdout, or a required `--out`); and **how it
reaches the machine**, which no section states.

**Candidate guesses — the two siblings in this repo disagree, so both are
defensible precedent.** `me sysw pack` writes a bare container to **stdout by
default** with `--out` optional and `--region` padding to 65,536; `me seal`
writes **UF2** with `--out` **required**, *"never stdout"*. And `me`'s own
converter refuses to guess: *"choose an output mode: `--out`, `--stdout`,
`--hex`, or `--base64`."*

**What the spec's logic implies.** `me sysw pack`'s shape, since §10.9 names
`sysw` explicitly. But `mt qr`'s output is **binary** and §10.10's *"stdout
carries the artifact"* was written for `mt string`'s text; piping ~7 KB of
binary at an interactive terminal is exactly what `me`'s explicit-output-mode
rule exists to prevent. **The cheapest resolution is to require `--out` for
`mt qr` and say the stdout rule binds `mt string` only.**

**The observable divergence.** A writes a bare container to stdout; B writes a
65,536-byte region image; C writes UF2. **None of the three is loadable by the
other two's documented procedure**, and only the region image is directly
usable with the project's flashing script. An operator following A's runbook
against B's binary writes a 65 KB image where a 7 KB container was expected.

---

### R-11 — The node: no location, no credentials, no timeout, and no taxonomy of non-answers

**Severity: Important. Section: §6a, §8.5, §10.5.**

**What the spec says.** §6a pins the call — *"`gettxout <txid> <vout> false`,
verified against a live Core v25.0.0 node"* — and pins the no-node warning text
verbatim. §10.5: *"`mt` asks the node it is given and reports what it is told."*
Executed greps over the spec: `timeout` **0**, `cookie` **0**, `bitcoin.conf`
**0**, `rpc` one hit and it is the word "RPC" in prose.

**What an implementer must guess.** URL, port, network selection, credentials
(cookie file vs `rpcuser`/`rpcpassword`), timeout — and the classification of
every non-answer.

**Candidate guesses.** (A) Explicit `--rpc-url` / `--rpc-cookie`, no default,
so "no node" unless asked. (B) Auto-discovery of `~/.bitcoin/.cookie` and
`127.0.0.1:8332`, so it *usually* finds one. (C) `bitcoin.conf` parsing.

**What the spec's logic implies: (A).** §10.5's *"the node it is given"* is the
closest thing to a ruling, and §0's offline posture agrees — a tool that
silently reaches for localhost is doing network I/O the operator did not ask
for. **Say it in one sentence and the flag spelling is all that floats.**

**The unresolved sub-cases, each a separate guess and none covered by §6a's
*"an absent node is an absent answer, not a bad one"*:**

| situation | is it "no node"? |
| --- | --- |
| connection refused | yes, by §6a's plain reading |
| timeout | undecided — implied warn-and-proceed |
| 401 / bad credentials | undecided — the node exists and told you nothing |
| **partial failure**: inputs 0–2 answered, input 3 times out | **undecided, and the worst case** |
| `-28` loading block index | undecided |

**The observable divergence, and it is funds-relevant.** §8.5 is a refusal —
"one of your inputs is already spent, do not cut this plate." Under (A) it never
fires unless a flag is passed; under (B) it fires on any box running Core. Same
PSBT, same machine: A engraves 21 minutes of steel for a transaction that can
never confirm; B refuses. On the partial-failure row, if input 3 is the spent
one, A downgrades the whole run to the no-node warning and B reports per-input
`UNKNOWN` — different text, same silence about the thing that matters.

---

### R-12 — §8's numbering is still defective, the refusal format is unwritten, and exit codes are unassigned

**Severity: Important. Section: §8, §10.10.**

**What the spec says.** §8 closes: *"Every refusal names the number that caused
it. A refusal that says only 'too large' costs the operator a round trip."*
§10.10 concedes exit codes and the message format are unspecified.

**The numbering itself is still defective, unchanged from R4.** §8's markers in
document order are `1, 2, 2b, 2c, 2d, 3, 4, 5, 6, 7, 7c, 7b, 8, 9`:

1. **`7c` is printed before `7b`** (lines 1223 and 1254). An operator reading
   top-to-bottom meets the section ceiling before the chunk ceiling.
2. **Item 1 and item 3 are the same refusal.** Item 1 (line 774): *"**Not fully
   finalized** → refuse"*. Item 3 (line 950): *"An unsigned or **unfinalized**
   transaction offered for engraving → refuse."* A finalized-PSBT check that
   fails has **two** numbers, so §8's promise cannot be satisfied
   deterministically.

**What an implementer must guess:** the rendering, *which number to name for the
duplicate*, and the whole exit-code space.

**Candidate guesses.** Format: `mt: refused (§8.1): …` / `mt: refusal 8.1: …` /
`mt: E8.1: …` / prose. Duplicate: A prints `§8.1`, B prints `§8.3`, C prints
both. Exit codes: `0`/`1`; `me`'s four-code scheme; a code per refusal.

**What the spec's logic implies.** `me`'s house style is `me: <message>` on
stderr with the section in prose, so `mt: …` with a parenthesised `§8.N`. For
exit codes, `me`'s sibling constant block (`EXIT_OK 0` / `EXIT_USAGE 2` /
`EXIT_REFUSED 3` / `EXIT_INVALID 4`) already distinguishes exactly the two
things §8 needs and should be adopted verbatim — **what remains a genuine guess
even then is which §8 items are `REFUSED` and which are `INVALID`** (§8.1 "not
finalized" is arguably malformed input; §8.7b "over the container" is arguably a
refusal), and whether a **warning** leaves exit `0` (it must, by §8.4's *"warn,
never refuse"*, but a CI author needs it in writing). For the duplicate, the
cheapest ruling is to **delete item 3**.

**The observable divergence.** An operator runbook says *"if you see §8.3, your
wallet did not finalize"*; against A's build that string never appears.
`mt qr … || alert` fires on A and not on B for the same low-fee transaction.

---

### R-13 — Input encoding is unspecified, and the obvious implementation strands half the intended users

**Severity: Important. Section: §10.10.**

**What the spec says.** *"**a finalized PSBT, and nothing else** — from a file
or stdin, **equivalently**"*. `grep -ci base64` over the spec: 1 hit, and it is
§3's efficiency table, not the input.

**What an implementer must guess.** (i) binary / base64 / hex, or a sniff;
(ii) positional path or a flag; (iii) what happens with neither, or both.

**Candidate guesses.** (A) base64 only, `--in FILE` else stdin — `me`'s shape.
(B) Sniff the 5-byte magic `psbt\xff`, else base64; positional path. (C) All
three encodings sniffed, both path forms accepted.

**What the spec's logic implies: (B) or (C).** Sniffing is free and
unambiguous — a binary PSBT starts `70 73 62 74 ff` and its base64 always starts
`cHNidP8`, so the two cannot be confused. §0's whole flow is *"test it in your
wallet first"*, and wallets emit **both**: `bitcoin-cli finalizepsbt` returns
base64 text, Sparrow and Electrum save binary `.psbt` files. Refusing either
strands half the intended users.

**A concrete trap the silence hides.** `me`'s stdin read is `read_to_string`,
which **fails on non-UTF-8**. An implementer copying that shape gets base64-only
by accident, and the failure on a binary PSBT is `stream did not contain valid
UTF-8` — a message that sends the operator to look at their terminal, not at
their file format. Also unstated and cheap: trailing-newline tolerance on
base64 (every shell heredoc adds one), and whether `--in` plus piped stdin is a
usage error — `me seal` already rules the analogous case.

**The observable divergence.** An operator with Sparrow's `.psbt` file gets a
clean run from B and `not valid UTF-8` from A.

---

### R-14 — §4's chosen configuration and §5's legend still have no channel into the payload, and §10.17 hides the decision rather than deferring it

**Severity: Important. Section: §2, §4, §5, §10.9, §10.17.**

**What the spec says.** §2 lists as a thing this codec exists to specify:
*"which (module size, QR version, ECC level, tiling) configuration is chosen …
**deterministically and with every tie broken**, so two encoders agree"*. §5
specifies five legend fields; §10.8 adds a per-symbol `n/m`. §10.9 rules the
payload travels as `sysw` and then defers the gap: *"A `sysw` class says how the
bytes *arrive*; it does not make the firmware able to engrave what §4 chose.
**That gap is now §10.17**."* §10.17 schedules firmware work.

**§10.17 is on the known-open list, so I am not restating it — I am naming the
implementer decision it hides.** Firmware capability and *wire channel* are
different questions. Even after SH2 learns to engrave at an arbitrary ECC level
and tiling, **something must tell it which ones** — and nothing in the spec says
what carries `(module, version, ECC, across × rows)` or the 136 characters of
legend text. That decision binds the **host** side, which is being built now,
and it cannot wait for firmware.

**What an implementer must guess.** Whether §4's answer and §5's legend are
(i) carried *in* the payload, in what record and what field; or (ii)
re-derived on the device.

**Candidate guesses.** (A) A second `sysw` record holding a config/legend blob
— which needs either another new class or a `text:` record, and the legend text
contains **spaces** (`BEARER - ANYONE HOLDING THIS CAN SPEND IT`), so under EPD
§6.4 it must be hex-escaped exactly as `FreeText` is, at 2×. (B) Device
re-derivation. (C) A split: config travels, legend is re-derived.

**What the spec's logic implies: (A).** §2's justification for specifying the
search at all is *"so two encoders agree"*; if the device re-derived, §4's
search would be host-side computation that never reaches the machine and §2's
fifth bullet would buy nothing.

**And F4 from R-3 belongs here.** §3 says *"`mt` uppercases only when encoding
the QR symbol"* — but `mt` never encodes a QR symbol; §10.9 and §10.17 put that
on the device. So the sentence assigns the uppercasing to a component that does
not do it, and if nobody does it the QR falls out of alphanumeric mode into byte
mode, at which point **every plate count in §4 is wrong**.

**The observable divergence.** A sends a two-record payload (transaction +
config); B sends one and the device engraves at its compile-time defaults,
ignoring the module size and the ECC level §4 spent every leftover byte buying.
Same PSBT, physically different plates, B's silently discarding the damage
tolerance §1.5 exists to maximise — and neither implementation can read the
other's payload.

---

### R-15 — The legend's no-timelock string exists in two spellings, and a *timestamp* lock that has already passed gets no rule at all

**Severity: Important. Section: §5 vs §8.4.**

**What the spec says**, four sites, two strings:

| line | text | context |
| --- | --- | --- |
| 549 (§5) | *"Reads **`NO BLOCK TIMELOCK`** when there is no enforced `nLockTime`"* | the legend field table |
| 1014 (§8.4) | *"…or `NO TIMELOCK`"* | the bullet headed **Legend:** |
| 1074 (§8.4) | *"The legend then reads `NO TIMELOCK`"* | the negative-delta ruling |
| 1136 (§8.4) | *"The **legend** now reads **`NO BLOCK TIMELOCK`**"* | the `OP_CSV` disclosure |

§8.4 contradicts **itself** twice over, and §5 backs one side. The strings are
11 and 17 characters, so this is not only a wording choice — it moves §5's
136-character budget, which §8.4 elsewhere pins to the character
(*"the legend goes from **130 to 136** characters and stays at **6 lines**"*).

**What an implementer must guess:** which string is engraved, and whether the
two cases (`nLockTime` unenforced vs `nLockTime` below the reference height) get
the same string or different ones.

**Candidate guesses.** (1) `NO BLOCK TIMELOCK` everywhere — §5 and the §8.4
argument that names it *"precisely true about the fields `mt` read"*. (2)
`NO TIMELOCK` everywhere — §8.4's Legend bullet and its negative-delta rule.
(3) Both, split by case, which is what the spec literally says today.

**What the spec's logic implies: (1).** The §8.4 passage at line 1136 gives the
*reason* — `NO BLOCK TIMELOCK` is silent about the `OP_CSV` relative locks `mt`
cannot read, whereas `NO TIMELOCK` is a positive claim `mt` cannot
substantiate, and that is the exact false-reassurance failure §8.4 exists to
close. The other two sites are stale phrasing. **A permanent claim on steel
should not be decided by which paragraph an implementer read last.**

**A second gap in the same ruling.** §8.4's negative-delta rule is written
entirely over **heights**: *"When `target_height < MT_REF_HEIGHT`…"*. A
**timestamp** `nLockTime` already in the past gets no warning and no legend
rule — so the same hazard (a plate that is spendable today) is loud for one lock
type and silent for the other. §8.4's own *"compare like with like"* principle
supplies the fix (compare against `MT_REF_TIME`), but the spec does not draw it.

**The observable divergence.** Same PSBT with an unenforced locktime: A engraves
`NO BLOCK TIMELOCK`, B engraves `NO TIMELOCK` — permanently, on steel, with
different line lengths. Same PSBT with a 2020 timestamp lock: A warns and
engraves `NO BLOCK TIMELOCK`; B follows §8.4 literally, prints no warning, and
engraves `LOCKED UNTIL 2020-01-01T00:00Z` on a plate anyone can broadcast today.

---

### R-16 — §4's objective is still not a total order, and two of its three inputs have no domain

**Severity: Important. Section: §4, §8.8, §10.1, §10.8.**

**What the spec says.** The objective, after the R0 fix: *"1. minimise plates →
2. maximise ECC → 3. minimise symbol count → 4. TIE-BREAK: maximise MODULE SIZE
→ 5. then minimise QR version"*, over a search space including *"rectangular
tiling (across × rows)"*. §2 requires the result be *"deterministic, with every
tie broken, so two encoders agree."*

**Three residues, all unchanged from R4's B-13/B-18:**

1. **Tiling orientation is not in the key.** `2 across × 3 rows` and
   `3 across × 2 rows` tie on plates, ECC, symbol count, module size **and**
   version. The comparison key cannot separate them, so the winner is again
   *"whichever the loop reached first"* — the precise defect §4's own correction
   note was written to remove.
2. **The module-size domain is unenumerated.** §4 lists module size in the
   search space and §8.8 says *"`mt` offers **every size it can engrave**"* with
   no set anywhere. *"Maximise module size"* has no maximum over a continuum,
   so step 4 is not evaluable until the domain is discrete.
3. **Which plate a symbol lands on is unstated** under a multi-plate tiling —
   which determines what §10.8's labels say and which plate a recoverer is sent
   to.

**What the spec's logic implies.** For (1), §4's step-4 reasoning is *"break
toward legibility, which is the direction the artifact's purpose demands"*, and
a 6th key in the same spirit is available — prefer the tiling closest to
square, ties toward more rows, since a taller stack leaves the legend's six
reserved lines undisturbed. For (2), a discrete ladder: §8.8's own vocabulary
(*"two engraved strokes"* for 0.60 mm) implies multiples of the 0.30 mm stroke.
**The spec implies a direction in both cases; it does not supply the key.**

**Observable divergence.** Two implementations produce visibly different plates
from the same PSBT — a 2×3 grid versus a 3×2 grid, or a different module size
off a different ladder. The symbols still decode, so nothing catches it; §2's
stated goal is simply not met, and any conformance fixture pinned to one
implementation's geometry is wrong for the other.

---

### R-17 — `count`'s valid range is unstated, and nothing says where `mt string`'s 64-chunk limit is enforced

**Severity: Important. Section: §3, §3b, §8.7b, §10.13(a2).**

**What the spec says.** §3: 8-bit fields *"admitting 256 chunks"*. §10.13(a2):
*"a set of 1 stores `0`, a set of 256 stores `255`"*. §3b: *"`mt string` keeps
the **64-chunk limit** because that is a property of the codex32 container it is
engraved into, **not of the header**."* §8.7b refuses past 64 for `mt string`.

**What `md-codec` does, which is what a forker inherits.** `ChunkHeader::write`
validates `(1..=64).contains(&count)` and returns `ChunkCountOutOfRange`, and
`split()` returns `ChunkCountExceedsMax` above 64. **Both caps live inside the
codec, not in a verb.** A forker who widens the bit fields to 8 and does not
notice the `1..=64` predicate ships an `mt-codec` whose `write()` refuses at 65
chunks — for *both* verbs.

**What an implementer must guess:** the validation range, and which layer owns
the 64-chunk rule.

**Candidate guesses.** (1) Codec validates `1..=256`; the 64-chunk rule lives in
the `mt string` path as §8.7b. (2) Codec validates `1..=64` (inherited
unexamined); `mt qr` inherits the cap. (3) Codec takes the cap as a parameter.

**What the spec's logic implies: (1)**, explicitly — §3b says the 64-chunk limit
is a property of the container and not of the header, and §8.7b scopes the
refusal to `mt string` by name. **The spec already answers this; it just never
states the codec's range**, and the inherited predicate is the trap.

**Two smaller residues in the same field.** `md-codec`'s `count` is a `u8`,
which **cannot represent 256** — the fork must widen the in-memory type or make
the API take `count - 1`. And nothing in §8 refuses a `mt qr` artifact above
**256** chunks; §8.7 (plate budget) and §8.7c (section ceiling) would normally
bite first, but §8.7c has no threshold yet (R-3), so there is currently no
numbered refusal standing between a large PSBT and `ChunkCountOutOfRange`.

**The observable divergence.** A (guess 1) encodes the 96-chunk artifact §4's
table measures. B (guess 2) refuses it with `ChunkCountOutOfRange { count: 96 }`
— **B's `mt qr` cannot produce the largest artifact the spec measures**, and the
error names a limit no section of the spec states for that verb.

---

### R-18 — §8.9 "Secrets → refuse" has no subject on a PSBT, and the real hazard — a seed pasted into the engraved label — is unguarded

**Severity: Important. Section: §8.9, §5, §10.4.**

**What the spec says.** In full: *"**Secrets** → refuse, as `me` already does
for `ms1`."* One sentence, unchanged.

**What an implementer must guess.** What a "secret" *is* when the input is a
finalized PSBT. BIP-174 defines no private-key field; a finalized PSBT carries
`PSBT_IN_FINAL_SCRIPTSIG` / `_SCRIPTWITNESS`, xpubs and UTXO records — all
public. **The refusal has no obvious subject.**

**Candidate guesses.** (A) Dead code — nothing can trigger it; ship an
unreachable branch. (B) Scan the PSBT's proprietary/unknown key-value pairs for
things shaped like key material. (C) Apply it to the **operator-supplied
strings** — `FROM`, `TO`, and the free-text label.

**What the spec's logic implies: (C), and it is the only reading that protects
anything.** §5's `TO <free text>` is **engraved**, on a **bearer** plate, and
§10.4 rules the label *"an act of assertion by the operator"* typed on a command
line. An operator who pastes a BIP-39 mnemonic or an `ms1` string into that flag
engraves their seed onto a plate that already spends. `me` has exactly this
guard and names it honestly — a best-effort anti-footgun, not a boundary —
combining an `ms1` classification with a BIP-39 checksum test. Worth stating in
the spec that the BIP-39 check is a *checksum* test, so it catches a pasted
mnemonic and not a partial one.

**The observable divergence.** A ships §8.9 as an unreachable branch and the
label path is unguarded; C screens every operator-supplied string. Between them
sits a plate with a seed phrase cut into it, next to a transaction that spends.

---

### R-19 — The new `Class`: `is_secret()` is undecided, and teaching `me`'s shared classifier `mt1` silently re-arms `me convert`

**Severity: Important. Section: §10.9, §7.**

**What the spec says.** §10.9 rules the payload **unencrypted** — *"the plate
the payload produces is bearer and sits in a drawer, so the wire is not where
this artifact's secrecy lives"* — and that the new class *"lands in `me-cli`'s
Rust `sysw` first, with test vectors, and only then ports to the fork's Go."*
§7 places an `mt` plate *"nearer `ms1` than `md1`"*.

**Two decisions the spec never makes.**

**(a) `Class::is_secret()` for the new variant.** Candidates: `false` (it is not
key material, and §10.9 rules the payload unencrypted) or `true` (§7 puts it
near `ms1`, and `is_secret()` is what raises the device's plaintext-secret
flag). **The spec's logic implies `false`**, and the existing precedent settles
the tension rather than leaving it: `Class::FreeText` is documented as
deliberately *not* secret *"even though an operator may put anything in it: a
class states what the format **guarantees**, not what a human might do"*. A
transaction is public data by construction. **But the consequence must be
written down**, because it is exactly what §7's threat model cares about: with
`is_secret() == false`, a bearer transaction sitting unencrypted in flash raises
**no device flag at all**.

**(b) What `me convert` does with an `mt1` string once the shared classifier
knows it.** Recognition naturally lands in the shared HRP switch that
`sysw::classify` delegates to, and `Format` is matched exhaustively downstream,
so the compiler forces every consumer to answer — `me convert`, `me bundle`,
`me seal --plaintext`, `me sysw pack`. **The spec's logic implies `convert` must
refuse**, unambiguously: `me`'s crate description is *"refuses secret `ms1`"*,
and §7 places an `mt` plate nearer `ms1` than `md1` because it is spendable by
whoever holds it. Pushing a signed spendable transaction over NFC to an
unauthenticated tag is the shape of hazard `me` refuses `ms1` for. **Today
`me convert` on an `mt1` string fails with "unrecognized HRP" — a refusal by
accident. After the class lands, that accident is gone.**

**The observable divergence.** For (a): implementer B reads §7, sets `true`, and
the device warns; A reads §10.9, sets `false`, and it does not — same payload,
different device behaviour. For (b): A ships and `me convert` emits an NDEF
payload of a spendable transaction; B ships and it exits with a refusal. **The
divergence is in a sibling tool the spec does not mention**, which is exactly
how it gets missed.

---

### R-20 — The success report has no format, no ordering, one unsatisfiable row, and an incomplete provenance enumeration

**Severity: Minor. Section: §10.10.**

**What the spec says.** Seven rows on stderr *"before any plate is cut"*.

**(a) Format.** Stable `key: value` (greppable), table, or prose — nothing says.
§8.4 *does* pin one row exactly, the locktime line, with five literal example
forms including column alignment. No other row gets that treatment. Units are
half-decided: §8.2c prints `0.99000000 BTC` and §8.2b prints `3.2 sat/vB`, so
the examples imply BTC amounts and sat/vB rates — but *"the fee | **absolute**
and as sat/vB"* leaves the absolute fee's unit open.

**(b) Ordering versus the warnings.** The report *"goes to stderr, **with the
warnings**"*, and nothing orders them. §8.2c's eleven-line legacy block lands
above or below a forty-line report — i.e. on screen or scrolled off it. For a
warning whose whole purpose is to be read before 21 minutes of steel, that is
not cosmetic.

**(c) The change-detection row is unsatisfiable.** *"every output | … **which
are change if a wallet was supplied**"*. Identifying change requires deriving
the wallet's own scripts — a descriptor. The only wallet input the spec defines
is §5's `FROM WALLET <8 hex>`, which §5 itself calls *"a hint, never an
authority — nothing may branch on it."* You cannot derive a scriptPubKey from
4 bytes of a hash, and branching on it is forbidden by the same section. A drops
the row silently; B adds a `--descriptor` flag the spec never authorised, which
pulls descriptor parsing into a tool §0 says *"holds no private key."*

**(d) The provenance enumeration is incomplete.** *"per input: chain-fetched
(§6a), txid-bound (§8.2d), or operator-asserted (§8.2c)"*. **A segwit input
carrying `witness_utxo` and no node is none of the three** — its value is bound
by the signature (BIP-143/341, §8.2c's own table) but nothing fetched or hashed
it. That is the *common* offline case. A labels it "operator-asserted", which is
alarming and false; B adds a fourth label. The row exists to tell an operator
how much to trust a number, and the two builds tell them different things.

---

### R-21 — `mt string`'s stdout framing, and the one sentence carrying a whole row of the threat model

**Severity: Minor. Section: §3b, §10.10.**

**What the spec says.** §3b: *"`mt string` **emits a string. That is the whole
of its output.**"* — singular, for something that is up to 64 strings. §10.10:
*"the **codex32 string on stdout**"*. §3b also rules a stderr warning that the
artifact is **bearer**, with no text.

**What an implementer must guess.** How multiple chunks are framed on stdout
(one per line, space-separated, concatenated), whether there is a trailing
newline, and the warning's exact wording.

**What the spec's logic implies: one chunk per line.** The constellation's
convention is LF-separated records everywhere — `me sysw pack --in` reads
*"newline-separated records"*, `me bundle` reads *"newline-separated public
strings"*, and `sysw`'s own record separator is LF on the stated grounds that no
constellation string contains a newline.

**On the warning text**, the asymmetry is worth naming: §3b calls this warning
the *entire* mitigation for `mt string`'s bearer hazard and §7 records it as an
accepted risk with no plate-side mitigation — so one unwritten sentence carries
a whole row of the threat model, while §6a, §8.2b and §8.2c all get their text
pinned verbatim.

**Observable divergence.** A hand engraver's worksheet from A's output has 14
lines; from B's it has one 1,120-character line. Any chunk-counting script
differs.

---

### R-22 — Legend rendering below the field list: absent fields, label placement, case

**Severity: Minor. Section: §5, §10.8.**

Five fields, 136 characters, 6 lines, plus §10.8's per-symbol `n/m`. What the
spec implies versus what is genuinely open:

| question | status |
| --- | --- |
| field **order** | **implied** — §5's table order, and §3b confirms `BEARER` is *"the first line of a legend `mt` controls"* |
| **line breaks**, 5 fields over 6 lines | **implied by arithmetic** — `BEARER…` is 41 chars against ~35/line, so it is the field that wraps |
| **truncation** of a too-long label | **decided** — §10.4 rules a refusal naming the limit; needs a number under R-12 |
| **case** | implied — every §5 example is uppercase and the fork's engraving font is uppercase-only in practice; unstated |
| an **absent** optional field (`FROM`, `TO`) | **genuinely open** — omit the line (5 lines), print a bare label, or print `FROM WALLET UNKNOWN`. §5 reserves 6 lines in §4's budget either way, so omitting frees space §4 already spent |
| where the `n/m` **sits** relative to its symbol | **genuinely open** — §10.8 says only *"beside"*, and §10.8 itself calls the area *"unpriced"* |

**Observable divergence.** For a transaction with no `TO` supplied, A engraves a
5-line legend and B engraves 6 with a bare `TO`. Both legible, both permanent,
and A frees 4.25 mm that §4's search did not know it had. **Note this rides with
§10.14's regeneration and should not be settled separately.**

---

### R-23 — Spec-hygiene minors, each one line to fix

**Severity: Minor.** Grouped because none carries an independent design
decision, and all were found while checking something else.

1. **A-10's two numbers are still unrecorded.** The widened header leaves
   **73 of 80** data symbols used (codeword 86 of 93) — I verified it. The next
   person to widen a field or revisit §10.12's 320-bit budget has no marker
   telling them the cliff is 7 symbols away.
2. **The drift-guard test is not required.** Both siblings carry a test
   recomputing the constant from the domain string, and `mk-codec` carries a
   second asserting its domain differs from `md1`'s. §10.13(a) states the
   derivation but does not require either test — and the derivation is the only
   thing standing between `mt1` and the fork-mechanic failure §10.22 describes.
3. **Behaviour on a `chunked` bit of `0`, and on a version mismatch,** is
   unstated. The fork refuses both; say so, since §10.13(a2) went to the trouble
   of ruling the bit's *value*.
4. **The padding *value* is unstated.** `md-codec` zero-pads the low bits of a
   short final symbol and calls that *"the canonical form"*; whether a decoder
   **enforces** it is a real choice (an enforcing decoder refuses a
   non-enforcing encoder's plate).
5. **§3 mis-cites its own justification.** *"§3b's own table measures the
   largest `mt qr` artifact at **96**"* — §3b's table has no 96 in it; its
   largest is **89**. The 96 comes from applying 40-byte chunking to the
   3,809 B PSBT, which is §8.7c's arithmetic, not §3b's table. This matters
   because that sentence is the whole stated reason for widening the header.
6. **§5 still calls the stub's source an open question.** *"Where the stub comes
   from is unspecified, and that is an open question … See §10.4."* §10.4 is
   **CLOSED** and §10.10's input table now answers it (operator-supplied, warn
   and engrave blank when absent). Stale forward reference.
7. **§10.2's cost note is falsified by the envelope change.** It says F-234's
   promise *"now holds only for artifacts that fit **one** symbol."* Since §3
   dropped UR, even a one-symbol artifact carries an **`mt1` chunk in bech32**,
   which no wallet parses — so the promise holds for **nothing**, and decision 3's
   *"the QR carries the standard form"* is no longer true either. This hides a
   real decision: an implementer reading §10.2 may special-case a single-chunk
   artifact to emit a bare PSBT. **§3's *"Both verbs fragment with the `mt1`
   chunk header"* already forbids that** — say so, and retire the promise.
8. **Release ordering across two repos is unstated.** For `me`'s `classify` to
   place an `mt1` record, `me-cli` must be able to validate one, so
   `mnemonic-engrave` gains a dependency on `mt-codec` — which must publish
   first, then `mnemonic-engrave`, then `mt`. Acyclic, so nothing errors; it
   simply cannot be done in one cycle, and an implementer discovers it on day
   one of the `sysw` work.
9. **No refresh cadence or staleness signal for the reference pair.** §8.4 now
   carries block provenance for whoever refreshes it and concedes the pair ages,
   and already commits to *printing* it. Judging it is unspecified — §6a's own
   principle (*"'no node' alone tells the operator nothing they can act on"*)
   argues a bare date stamp is the same non-actionable shape.

---

### R-24 — Nits

**Severity: Nit.**

1. **§10.13(a2)'s justification for `version = 0b0001` is false.** *"a shared
   value would let one format's chunk verify as the other's under a colliding
   constant"* — with distinct NUMS constants, which §10.13(a) guarantees, a
   shared version cannot do that. The decision is right; the reason is not, and
   a wrong reason invites a future reader to relax the decision.
2. **No severity markers, no tool prefix.** Three warning bodies are pinned
   verbatim and each opens `WARNING:`; two more are described only as *"loudly
   warned"* and one (§3b's bearer warning) has no text. Nothing says whether the
   `mt: ` prefix applies or whether there is a NOTE/WARNING/ERROR ladder — and
   `me` itself is inconsistent between two styles, so there is no clean
   precedent to inherit.
3. **`mt` should offer no passphrase flag, and should say so.** §10.9 rules the
   payload unencrypted, but the sibling's default is the *opposite* —
   `me sysw pack` generates a passphrase unless told not to. An implementer
   mirroring that CLI produces a sealed payload by default, against the ruling;
   and if they offer one anyway it **silently no-ops**, because only secret
   classes are encrypted and R-19 implies `is_secret() == false`. The operator
   writes down a passphrase that protects nothing.

---

## The `mt qr` walk

End to end, marking each step **RULED** or **GUESS**. This verb **cannot be
built at `322bbb5`**: three of the seventeen steps are Critical guesses.

| # | step | status |
| --- | --- | --- |
| 1 | read a finalized PSBT from file or stdin | **GUESS** — encoding, path shape, both-supplied (R-13) |
| 2 | parse; run §8.1, 8.2b, 8.2c, 8.2d, 8.4, 8.5, 8.6 | **RULED**, and well — this is the strongest part of the spec |
| 3 | §8.7 plate-budget refusal | **GUESS** — no input exists; required or defaulted is undecided (R-9) |
| 4 | §8.7c section-ceiling refusal | **GUESS** — no threshold can exist until R-3 resolves |
| 5 | §8.9 secrets refusal | **GUESS** — no subject on a PSBT (R-18) |
| 6 | collect `FROM`, `TO`, label, module size, input values | **GUESS** — spellings and units only (R-9) |
| 7 | reach a node; classify a non-answer | **GUESS** — entirely (R-11) |
| 8 | `chunk_set_id` = top 20 bits of the extracted txid, display form | **RULED**, unambiguously |
| 9 | chunk count for the QR payload | **GUESS, Critical** — 40-byte chunks or symbol-sized (R-2) |
| 10 | chunk sizes | **GUESS** — "balances" is not an algorithm (R-5); §11 still says flat-40 (R-4) |
| 11 | per chunk: 41-bit header MSB-first, payload immediately after | **RULED** — and it matches `split()` |
| 12 | end-pad the chunk to a byte boundary | **RULED** — but see 13 |
| 13 | bytes → bech32 characters | **GUESS, Critical** — per-chunk or whole-stream; 7,054 vs 7,016 characters (R-3) |
| 14 | frame into `sysw` record(s), lowercase | **GUESS, Critical** — and the EPD-conformant shape cannot classify (R-3) |
| 15 | §4 search for module / version / ECC / tiling | **GUESS** — key not total, domain unenumerated (R-16) |
| 16 | assign chunks to symbols and symbols to plates | **GUESS** — follows from 9; plate assignment unstated (R-2, R-16) |
| 17 | per-symbol `n/m` labels | **GUESS** — unimplementable as written under step 9's measured model (R-2) |
| 18 | render the legend | **GUESS** — absent fields, case, `NO TIMELOCK` spelling (R-15, R-22) |
| 19 | get the config and the legend to the device; uppercase for alphanumeric | **GUESS** — no channel, no owner (R-14) |
| 20 | write the output | **GUESS** — container / region / UF2, stdout or `--out` (R-10) |
| 21 | success report + warnings on stderr; exit | **GUESS** — format, ordering, exit code (R-12, R-20) |

## The `mt string` walk

Substantially buildable. One Critical, and it is one word.

| # | step | status |
| --- | --- | --- |
| 1–7 | as above, minus §8.7 and §8.7c (neither binds this verb) | as above |
| 8 | `extract_tx()`; payload = the raw signed transaction | **RULED** |
| 9 | `chunk_set_id` = top 20 bits of that transaction's txid | **RULED** — and here the id is a hash of the payload itself, which is the cleaner case |
| 10 | `count = ceil(bytes × 8 / 320)`; refuse above 64 (§8.7b) | **RULED** in effect; where the cap lives is a fork trap (R-17) |
| 11 | chunk sizes | **GUESS** — same as `mt qr` step 10 (R-4, R-5) |
| 12 | per chunk: 41-bit header, payload, pad to the 5-bit symbol boundary | **RULED**; padding *value* unstated (R-23) |
| 13 | BCH checksum | **GUESS, Critical** — `hrp_expand("mt")` vs `"mt1"`, and the init/generator are unpinned (R-1) |
| 14 | emit `mt1` + data symbols + 13 checksum symbols | **RULED**, once R-1 separates HRP from prefix |
| 15 | frame N chunks on stdout | **GUESS** — lines, spaces, or concatenated (R-21) |
| 16 | bearer warning on stderr | **GUESS** — text unwritten, and it is the entire mitigation for a threat-model row (R-21) |
| 17 | success report; exit | **GUESS** (R-12, R-20) |

**Close R-1 and R-5 and this verb is buildable**, with R-4 and R-6 as
correctness follow-ons. That is a materially different state from `mt qr`, and
it argues for building `mt string` first.

## The recoverer walk

**§9 rules that there is none in v0.1** — *"a plate cut by `mt` v0.1 **cannot be
read back by `mt` v0.1**"*, with the reader arriving in §10.2's next
subversion. So this walk is not a gap to close now. **But five decisions are cut
into steel by the encoder and cannot be revisited by a later reader**, and those
are the ones this round cares about:

1. **The `n/m` labels are the recoverer's only pre-decode inventory** (§10.8's
   stated purpose: *"name what is missing **without decoding anything**"*). What
   they can say depends entirely on R-2. Under the 40-byte-chunk model a symbol
   holds no single chunk, so the label must become a range and §10.8's *"a lone
   symbol reads `1/1`"* stops being true. **This is engraved. It cannot be
   fixed by the future reader.**
2. **The missing-index diagnostic.** The inherited `ChunkSetIncomplete { got,
   expected }` does not name *which* index is absent — the one fact that makes
   §10.8's labels actionable, and a violation of §8's own closing rule (R-7).
3. **Duplicate tolerance.** A camera re-scanning a symbol is normal operation;
   the inherited decoder treats it as fatal (R-7). Deciding this now costs a
   sentence; discovering it after five plates are cut costs a recovery.
4. **Which parser to apply and what to hash for the content-id compare** —
   PSBT or raw transaction, extract-then-hash or hash-directly (R-8). The
   `chunked` bit that could have signalled it has been spent on retention.
5. **Nothing on the plate names the format** (§10.21, known open). For
   `mt string` the `mt1` HRP identifies it to someone who already knows the
   constellation; **for `mt qr` the symbols carry nothing at all**, and R-23.7
   shows the escape hatch §10.2 still claims — a wallet reading a single symbol
   — no longer exists, because even a one-symbol artifact is an `mt1` chunk in
   bech32 rather than a PSBT.

**The honest summary:** a recoverer in 2040 holding an `mt qr` plate has QR
symbols encoding a format no shipped software reads, labels whose meaning
depends on an unresolved question, a five-field legend that does not name the
tool, and — if they scan a symbol twice — a decoder that may tell them their
complete set is incomplete.

---

## Ranked decision list

Ordered by what must be settled first, not by severity alone. Items 1–4 gate
whether `mt qr` can be built at all; 5–8 decide what bytes reach steel; the rest
can be settled while implementation proceeds.

| # | item | what to settle | who decides |
| --- | --- | --- | --- |
| **1** | **R-2** | **Is an `mt1` chunk 40 payload bytes, or one QR symbol?** §10.8 and §4's measured table cannot both stand. Resolving toward symbol-sized chunks dissolves §3's reason for the 41-bit header and moves §3's efficiency figure and §4's plate table | **Operator / design** — this is the only item in the report that is not an editing task |
| **2** | **R-3** | The six framing decisions F1–F6: base32 granularity, record boundary, record text, who uppercases, the length rule, and the resulting §8.7c threshold. §10.9 and §8.7c already name this as the blocker — **confirmed genuine** | spec + a `classify` decision in `me-cli` |
| **3** | **R-1** | `hrp_expand("mt")`, prefix `mt1`; plus `POLYMOD_INIT = 0x23181b3`, `GEN_REGULAR`, and the trailing 13 zeros written down. **One word and three constants** | spec, one paragraph |
| **4** | **R-14** | What channel carries §4's configuration and §5's legend to the device, and who uppercases for alphanumeric mode. Distinct from §10.17's firmware capability | spec |
| **5** | **R-5** + **R-4** | The split formula (`bytes_per_chunk = len.div_ceil(count)`, sequential slices), and delete §11's surviving "flat 40" clause | spec, two sentences |
| **6** | **R-6** | The length-recovery rule, stated **per verb** — the `mt string` identity does not survive `mt qr`'s 11-bit padding window | spec, two sentences |
| **7** | **R-9** | Whether the plate budget is required. The other seven inputs need only spellings | spec, one ruling + a flag table |
| **8** | **R-15** | `NO BLOCK TIMELOCK` vs `NO TIMELOCK` — pick one, it is engraved; and extend the past-lock warning to timestamps | spec, one string |
| **9** | **R-7** + **R-8** | Duplicate/gap/mismatch behaviour, explicitly excluding first-wins; name the missing index; and say which parser a reassembler applies and what it hashes | spec |
| **10** | **R-17** | Codec range `1..=256`; the 64-chunk cap lives in the `mt string` path. **§3b already implies both** | spec, one sentence |
| **11** | **R-11** | Explicit `--rpc-*` per §10.5, plus the five-row non-answer table | spec |
| **12** | **R-10** | Require `--out` for `mt qr`; scope §10.10's stdout rule to `mt string` | spec |
| **13** | **R-13** | Sniff `psbt\xff` / `cHNidP8`; `--in FILE`, stdin otherwise | spec |
| **14** | **R-12** | Delete §8 item 3, reorder 7b/7c, pin `mt: refused (§8.N): …`, adopt `me`'s 0/2/3/4 and classify each item | spec |
| **15** | **R-16** | A 6th tie-break key, and a discrete module-size ladder | spec |
| **16** | **R-18** + **R-19** | Screen operator strings for seed material; `is_secret() == false` with the no-flag consequence stated; refuse `Format::Mt` in `me convert` | spec + `me-cli` |
| **17** | **R-20** … **R-24** | Report format, stdout framing, legend residues, the nine hygiene items, three nits | spec; **R-22 rides with §10.14's regeneration** |

**Items 3, 5, 6, 10 and most of R-23 are together about one page of spec text**,
and they convert one Critical and five Importants into settled facts. **Item 1
is the only one that cannot be written by a reviewer or an implementer**, and
nothing about `mt qr` is safe to build until it is answered.
