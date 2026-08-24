# R6 — LENS: IMPLEMENTABILITY

**Artifact:** `design/SPEC_mt_v0_1.md` (3,082 lines)
**Date:** 2026-08-23
**Lens (the one question):** Could a competent implementer build `mt-codec` and
the `mt` CLI from this document **without inventing anything** — and would two
independent implementers produce **byte-compatible output**?
**Mode:** read-only. No spec edits were made.

**Answer: no.** Two implementers working only from this document produce
different chunk boundaries, different chunk counts, different-case strings,
and mutually-rejecting length checks. Below, every finding names the exact
decision that would have to be guessed.

**Verdict by severity: 2 Critical, 11 Important, 12 Minor, 2 Nit.**

---

## Method note — what was machine-checked before writing this

Per the brief, structure/numbering/citations were treated as settled and were
not re-examined. What I *did* run, because this lens turns on arithmetic:

- Reconstructed the string-length model from the spec's own numbers and
  confirmed it is self-consistent:
  `len(chars) = 2 + 1 + ceil((49 + 8·bytes)/5) + 13`. All six rows of §1e's
  table (79/74, 85/82, 89/71, 90/61, 90/90, 90/55) reproduce exactly, and
  `13×89 + 71 = 1,228` matches §1.1's `CUT` row. **The model is derivable —
  the problem is that the two inputs to it are not stated (C-1, C-2).**
- Confirmed `count = ceil(len/40)` reproduces every row of §3b's chunk table
  (162→5, 405→11, 535→14, 742→19, 2498→63, 3538→89).
- Proved the last chunk can never be empty under `bytes_per_chunk =
  ceil(len/count)` with `count = ceil(len/40)`: `bpc ≤ 40` ⇒
  `bpc·(count−1) ≤ 40(count−1) < len`. **No degenerate case — this is one
  thing the design gets right and nobody had shown it.**
- Confirmed a full `mt1` chunk is 74 data symbols (`ceil(369/5)`), inside
  `md-codec`'s `REGULAR_DATA_SYMBOLS_MAX = 80` and inside the `data ||
  checksum ≤ 93` order-of-β bound (74 + 13 = 87). **The 49-bit header does
  not overrun the code.** (I checked whether `hrp_expand` counts toward the
  93 — it does not; `bch_decode.rs:22` bounds `data_with_checksum.len()`.
  A finding I nearly filed and withdrew.)
- Measured the two worked correction examples column-by-column with a script,
  which is how I-5 below was found.

---

## CRITICAL

### C-1 — §11 and §3b specify two different chunk splits. One of them makes the other's plates read as damaged steel.

**The decision that must be guessed:** given a payload of `len` bytes split
into `count` chunks, how many bytes does chunk *i* carry?

**Line numbers.** §3b, lines 1100–1113:

> `md-codec` computes `chunks_needed` against the 320-bit ceiling and then
> splits the payload **`bytes_per_chunk = ceil(len / count)`**, each chunk
> taking that many bytes and the **last taking whatever remains** … **No chunk
> is padded to 40.**

§11, line 3062, still says the opposite:

> **That reason is now void**: §3b's correction established that chunk sizing
> is **a flat 40 payload bytes** (`crates/md-codec/src/chunk.rs:224,253-254`),
> so the count is *exact* for a given payload size.

§3b explicitly retracts "a flat 40 bytes per chunk" as an error (line 1100,
"An earlier version of this box called it 'a flat 40 bytes per chunk', and
that mis-describes the chunker"). §11 then cites §3b as the authority for the
retracted claim. Same document, 1,950 lines apart.

**Why two implementers diverge, concretely.** A 535-byte transaction, 14
chunks:

| rule | chunk bytes | string lengths |
| --- | --- | --- |
| §3b (balanced) | 39×13, then 28 | 89×13, then **71** |
| §11 (flat 40) | 40×13, then 15 | 90×13, then **50** |

Both sets decode to the same transaction — a decoder concatenates payload
slices in index order and neither boundary choice is visible to it. **That is
what makes this dangerous rather than obvious.** The break is at §1e's
*mandatory, pre-decode* length check (lines 633–660), which is normative and
runs before anything parses:

>     string 7: 88 characters (expected 89) — a character is MISSING, not
>               wrong. … Re-read the plate.

Implementation A, holding implementation B's fourteen plates, reports **every
string as damaged and points the operator at their steel.** The operator's
correct response to that message — re-cut — destroys nothing but costs ~21
minutes a plate and never converges, because the fault is in the software.

**The spec already knows this hazard and named it, in the paragraph directly
above the surviving contradiction** (lines 1111–1113): *"Two implementers, one
following the sentence and one following the code, produce different chunk
boundaries and therefore plates neither can read."* R5 readiness caught the
inverse of this defect in §3b and the fold never swept §11.

**Smallest addition that removes the ambiguity.** Delete the flat-40 clause at
line 3062 and replace §11's sentence with a pointer: *"chunk sizing is
`bytes_per_chunk = ceil(len / count)` per §3b; the count is exact for a given
payload size."* Then state the split once, normatively, in §10.13 beside the
header layout rather than only inside a §3b correction box:

>     count           = ceil(payload_len / 40)
>     bytes_per_chunk = ceil(payload_len / count)
>     chunk i         = payload[i·bytes_per_chunk .. min((i+1)·bytes_per_chunk, payload_len)]

---

### C-2 — The chunk-count formula is never stated as a rule for `mt1`. It exists only as prose *about `md-codec`* inside a correction box.

**The decision that must be guessed:** how many chunks does a payload of `len`
bytes become?

**Line numbers.** Every occurrence in the document is either descriptive of
another codec or a bare assertion the same section retracts:

- 1069: *"A chunk carries **40 payload bytes**"* — flatly stated, and
  contradicted 30 lines later.
- 1097: *"**40 bytes is the CEILING the chunk count is derived from**, not the
  size of each chunk"* — correct, but it is a statement about `md-codec`'s
  `SINGLE_STRING_PAYLOAD_BIT_LIMIT`, a constant the spec cites as a fact about
  a *sibling crate* and never adopts for `mt-codec`.
- 1118: *"This is the same error class as the 363-vs-320 correction above: **a
  limit read as a rule**."* — the document warns the reader against exactly the
  inference it requires them to make.

There is no line anywhere that says *"`mt-codec` chooses `count` as
`ceil(payload_bytes / 40)`."*

**Why two implementers diverge.** The obvious alternative reading is invited by
the spec's own justification at line 865:

> Both `41 + 320 = 361` and `49 + 320 = 369` fit the **400-bit capacity**.

An implementer told the binding constraint is the 400-bit / 80-data-symbol
single-string capacity sizes chunks against it: `(400 − 49)/8 = 43` bytes per
chunk. A 535-byte transaction becomes **13 chunks of 42/41 bytes**, not 14 of
39/28. Different count, different `chunk_set` headers in every string,
different `STRING n OF m` labels engraved on steel, different string lengths,
and the same false "your plate is damaged" verdict as C-1. I verified 43 bytes
is legal under the code (49 + 344 = 393 bits = 79 data symbols ≤ 80; 79 + 13 =
92 ≤ 93), so nothing downstream refuses it — **the wrong reading builds and
passes its own tests.**

§10.12 closes *"fill vs balance"* as NO-fill, which rules out 43 bytes for a
reader who reaches §10.12 and connects it — but §10.12 argues about error
budget, never states the count formula, and is 1,700 lines from §3b.

**Smallest addition.** The three-line normative block given at the end of C-1,
placed in §10.13 where an implementer building the codec is already reading,
with one sentence saying the 40 is `mt-codec`'s own constant and not inherited:
*"`MT_CHUNK_PAYLOAD_BYTES = 40` (320 bits). It is a sizing budget below the
400-bit codeword capacity, not a per-chunk fill target."*

---

## IMPORTANT

### I-1 — `mt encode`'s stdout case is never stated, and "normalise case" never says which direction.

**Guess required:** does `mt encode` write `mt1qzrf8x…` or `MT1QZRF8X…`?

**Lines.** Every case rule in the document governs an artifact that is
**deferred**: line 731 / 905 / 913 / 2425 are the *QR payload*; 969–970 are the
*`sysw` record*. Line 620's *"Engrave UPPERCASE; accept anything"* is advice to
a human about steel, not a statement about the byte stream. §0a (99–108) makes
stdout normative — *"stdout IS THE STRINGS AND NOTHING ELSE … the output exists
to be piped"* — and never says in which case.

The document's own examples split: lowercase at 152, 712 and 2715; uppercase at
293, 1338 and 2986.

**Why it matters more than style.** stdout is declared the artifact and is
piped into files, into `mt qr` when that lands, and into whatever the operator's
engraving path is. Two implementations emit different bytes for the same
transaction, so any byte comparison, hash or `diff` across tools fails while
both are "correct". §3's whole EPD §6.4 argument turns on lowercase being what
is *stored*.

**Second half of the same gap:** §1e's step 1 is *"strip whitespace, **normalise
case**"* (line 665) with no direction, and the autocorrect table immediately
below (681–687) is written in **mixed case** — `l`, `I`, `i` at position 2;
`1`, `i` at 3+; `o`, `b` at 3+. An implementer who normalises to uppercase
first has a table that matches nothing.

**Smallest addition.** One sentence in §1e: *"`mt encode` writes lowercase.
Input is normalised to lowercase before step 2; the correction table below is
read after normalisation."*

---

### I-2 — "Strip whitespace before doing anything else" makes `decode` and `verify` unbuildable: nothing says how strings are separated.

**Guess required:** given a file or a paste, how does `mt` split the input into
individual `mt1` strings?

**Lines 627–631:**

> Whatever grouping the operator chose, `mt decode` and `mt verify` **strip
> whitespace before doing anything else**.

and line 665, step 1: *"strip whitespace, normalise case"*.

Followed literally, fourteen 89-character strings become one 1,228-character
blob and the tool cannot parse its own output. The rule cannot mean what it
says, and the spec never states what it does mean.

**Why two implementers diverge.** A splits on newlines then strips intra-line
spaces — and refuses the single-line paste an operator produces by copying
three plates' worth of text out of a terminal. B scans for `mt1`/`MT1`
occurrences and slices between them — and accepts it. C strips everything and
re-splits by counting characters, which needs the length that I-3 shows is not
computable. The recovery path is where this bites, and a refusal there is
answered by an operator retyping 1,228 characters from steel.

**Smallest addition.** *"Input is split into candidate strings on any run of
whitespace containing a newline; spaces and tabs **within** a line are grouping
separators and are stripped. A line containing more than one `mt1` prefix is
split at each prefix."*

---

### I-3 — The mandatory length check has no stated way to compute the expected length at decode time.

**Guess required:** when `verify` holds only strings — possibly damaged,
possibly incomplete — what is "expected 89"?

**Lines 633–660.** *"Every string in a set has a KNOWN length, checked **before
decoding**"* and *"**`mt` computes both lengths and states them.**"* For
`encode` this is trivial: it holds the payload. For `decode`/`verify` it is
circular — the per-chunk length follows from `bytes_per_chunk`, which follows
from the *total* payload length, which is not known until every chunk is
assembled, which is what the check is supposed to gate.

**Why two implementers diverge.** A infers the expected length as the modal
length of the strings present, so a set where 2 of 14 are miscut still checks
correctly; B derives it from `count` plus an assumed 40-byte chunk (and lands
in C-1's failure); C skips the check when the set is incomplete, which is
exactly the case §1e wrote it for. The error message *"a character is MISSING …
Re-read the plate"* is a claim about the operator's steel, so a wrong expected
value is a wrong accusation.

**Smallest addition.** State the derivation from strings alone: *"All chunks
with `index < count − 1` carry the same payload length, so the expected full
length is the modal string length across the set; the final chunk's expected
length is `2 + 1 + ceil((49 + 8·(payload_len − (count−1)·bytes_per_chunk))/5) +
13` once the set is complete, and is not checked before then."*

---

### I-4 — Duplicate resolution never says whether "bytes identical" means as-typed characters or post-correction payload.

**Guess required:** in §1's three-row table, which bytes are compared?

**Lines 213–217.** Row 2 *"both pass, bytes identical → accept silently"*; row 3
*"both pass, bytes **differ** → refuse loudly."*

Two copies of chunk 7 that differ by one miscut character are, after BCH
correction, **identical payloads** and, as typed, **different strings**. So:

| implementer compares | verdict on the same two plates |
| --- | --- |
| as-typed characters | **refuse loudly** — "two valid chunks disagreeing" |
| corrected payload bytes | **accept silently** — the §1.8 duplicate case |

That is not a formatting difference. One implementation recovers the
transaction and the other tells the operator their two plates describe
different transactions and stops. §1.8's duplicate copy is *the spec's own only
mitigation for its largest accepted risk*, and the as-typed reading refuses it
whenever the second copy needed any correction at all — which is the normal
case for hand-cut steel.

Two smaller unstated cases in the same table: **three or more** chunks sharing
an index (the table is titled "two chunks, same index"), and the **precedence**
between duplicate resolution and §1's *"every chunk carries the same
`chunk_set_id`"* check when the duplicates disagree on the set id.

**Smallest addition.** *"Comparison is over the **corrected payload bytes**,
after BCH correction has succeeded on both. Two copies that correct to the same
payload are the §1.8 duplicate case regardless of how many symbols each needed.
With three or more candidates, the rule applies pairwise and any pair that
differs is a refusal."*

---

### I-5 — Correction positions: the two worked examples use opposite indexing conventions, and neither is stated.

**Guess required:** is `pos 34` the 34th character or the character at offset
34, and offset from where?

I measured both examples by column rather than by eye:

- **Line 293** — `MT1QZRF8XK2V[q>p]HQ9WRDG5S8XE7M2…` reported as `pos 12`.
  Exactly **12 characters** precede the bracket, so the corrected character is
  at **0-based index 12** of the whole string including `MT1`.
- **Lines 711–713** — `corrected \`b\` -> \`6\` at position 12`, with the caret
  under **0-based index 11**. That is **1-based**.

Same document, same concept, opposite conventions. (The second example is
doubly broken: index 11 of `mt1qzrf8xk2v` is `v` and index 12 is `.` — neither
is the `b` it claims to have corrected.)

**Why it matters beyond tidiness.** §1's entire design rests on this number
being checkable against steel (lines 318–320): *"if position 34 on the steel
reads `d`, they mistyped; if it reads `v`, they miscut."* An off-by-one sends
the operator to the wrong character, where they read a symbol that matches
neither value and learn nothing — and the spec has just told them this single
comparison is what resolves miscut-versus-mistyped.

**A third, invisible offset.** BCH error positions come out of the decoder as
indices into `data || checksum` (`md-codec/src/bch_decode.rs:22`), i.e. the
first data symbol is codeword index 0 and string index 3. Nothing in the spec
states the `+3` mapping, so an implementer wiring the corrector's output
straight into the report is off by three and will never notice, because the
report is prose.

**Fourth:** if the operator engraved with §1e's optional grouping, plate
positions are not string positions. `verify` strips whitespace before anything
else, so its positions are in the stripped string while the operator is counting
on grouped steel.

**Smallest addition.** *"Positions are **0-based indices into the
whitespace-stripped string, counting the HRP**: index 0 is `m`, index 2 is the
separator, index 3 is the first data symbol. A BCH codeword index `k` maps to
string index `k + 3`. Where the operator used grouping, positions do not count
the spaces."* Then regenerate the line 711–713 example so it agrees.

---

### I-6 — Chunk numbering is 0-based in the rules and 1-based in every report. "Chunk 7" is ambiguous, and the remedy is re-cutting a plate.

**Guess required:** does `chunk 7` in `verify`'s output mean `index == 7` (the
eighth string) or the seventh string (`index == 6`)?

**Lines.** Rules are 0-based: §1 line 192 *"indices `0..count-1`"*; §10.13(a2)
line 2818 *"`index` | **plain, zero-based**"*. Reports are 1-based: line 479
*"14 strings, **1..14** all present"*; line 248 *"strings **1-13** are 89
characters, string **14** is 71"*. The correction report sits between them
(lines 287–301) and says only `chunk 2`, `chunk 7`, `chunk 11`.

**Why two implementers diverge.** Both readings are supported by the document,
and the consequence is not cosmetic: the report's closing line is *"Chunk 7 is
at its correction limit … **Re-cut it.**"* An operator re-cutting the wrong
string spends ~21 minutes producing a duplicate of a good plate and leaves the
one-scratch-from-unrecoverable string on the shelf. The same ambiguity governs
the FAILED report's ranked suspect list (lines 369–374), whose entire value is
telling the operator *which three of fourteen* to retype.

**Smallest addition.** One sentence in §1: *"All human-facing output numbers
chunks from 1 (`chunk n` = `index n−1`). `index` is zero-based on the wire and
appears nowhere in output."* Then make §1's completeness rule read *"chunks 1
through `count` present"*.

---

### I-7 — Whether raw hex is accepted input is stated both ways. §8.2e says yes and supersedes; §10.10 and §6 still say PSBT-only.

**Guess required:** does `mt encode < tx.hex` work?

**Lines 1823–1875 (§8.2e)** accept it: *"**A raw signed transaction is
ACCEPTED, with a loud warning** … `mt` never refuses the bytes … **This
supersedes the earlier PSBT-only input ruling.**"*

**Lines 2589–2607 (§10.10)** refuse it, in the section titled *the CLI
surface* — the one place an implementer building the binary is certain to read:

> **Why PSBT-only, when `mt encode`'s PAYLOAD is a raw transaction.** … So
> accepting raw hex would **silently disable two refusals** … **`mt` therefore
> requires a PSBT**, runs the full refusal set against it …

**Line 1464 (§6)** repeats it: *"**`mt`'s INPUT is always a finalized PSBT
(§10.10)**, even for `mt encode`."*

§8.2e's own justification is that PSBT-only *"would have refused **the default
output of the reference implementation**"* — `finalizepsbt` defaults to
`extract=true`. So the two readings differ on whether the tool accepts what
Bitcoin Core hands the user by default at exactly the moment this workflow
starts.

The word "supersedes" resolves it for a reader who finds §8.2e first. §10.10's
block carries no retraction marker while every other superseded passage in this
document does, so it does not read as stale.

**Smallest addition.** Replace §10.10's "Why PSBT-only" block with a "Why a
PSBT is *preferred*" block carrying the same degradation table, and strike
*"`mt` therefore requires a PSBT"*. Fix line 1464 to *"`mt` prefers a finalized
PSBT (§8.2e) and accepts a raw signed transaction with §8.2e's warning."*

---

### I-8 — A locktime already in the past gets two different engraved legends.

**Guess required:** for `nLockTime = 900000` with the chain at 963,663, does the
suggested legend read `NO TIMELOCK` or `LOCKED TO BLOCK 900000`?

**Lines 2092–2104:**

> **A NEGATIVE subtraction means the lock is already behind us — warn.** …
> The legend then reads **`NO TIMELOCK`** rather than a `~<year>` …

**Lines 2141–2144, forty lines later:**

> **A lock that has already passed is reported the same way**, because the two
> numbers say so: `LOCKED TO BLOCK 900000, current height 963663` is a plate
> that is live now …

Same transaction, same section, two different strings. This is engraved, and
§8.4 has already established (lines 2033–2039) that a drifting engraved
spelling is a real defect: *"two `mt` versions would cut different plates for
the same transaction, and a recoverer matching against documentation would find
neither."* The `NO TIMELOCK` spelling was pinned to fix exactly this class, and
this instance survived the fix.

There is also a substantive question underneath: `NO TIMELOCK` is a claim about
the *fields*, and a transaction with an enforced past `nLockTime` **does** have
one. §8.4's own §10.23 argument — the height is the fact, the estimate is the
courtesy — argues for keeping `LOCKED TO BLOCK 900000` and dropping only the
`~<year>`.

**Smallest addition.** At line 2101: *"The legend keeps `LOCKED TO BLOCK <n>`
and **omits the `~<SEASON> <year>` estimate**; `NO TIMELOCK` is reserved for a
transaction with `nLockTime = 0` or with all inputs final."*

---

### I-9 — §8.5 refuses on any `gettxout` null; §1.1 says a null can mean PENDING. Whether `encode` refuses a child of an unconfirmed parent is unstated.

**Guess required:** the operator hands `encode` a transaction spending an output
of a parent still in the mempool. Refuse, warn, or proceed?

**Line 2174 (§8.5):** *"**`gettxout` returns `null` for any input** → refuse,
when a node is reachable. **The output is spent or never existed.**"*

**Lines 434–446 (§1.1):** the four-state table says a `null` **with the parent
findable** is DEAD, and a `null` with the parent **not** findable is PENDING —
*"the parent transaction was never confirmed. **The plate may still become
live**"* — and the section's closing warning is that *"telling a recoverer their
plate is scrap when it is merely early is the worst error available here."*

**Line 1599** compounds it: `include_mempool` is passed **false** deliberately,
so an unconfirmed parent's output *always* reads `null`. Under §8.5 as written,
**every transaction that spends an unconfirmed parent is refused at encode
time**, with a message asserting something false about the operator's coin.

Implementer A applies §8.5 literally and refuses. Implementer B applies §1.1's
table and proceeds with a PENDING row. Both cite the spec.

**Smallest addition.** *"§8.5's refusal applies to the **DEAD** state only —
`null` **and** the parent resolvable via `getrawtransaction`. `PENDING` and
`UNKNOWN` are warnings, not refusals, and the §6a enumeration names them."*

---

### I-10 — The CLI names zero flags for eight required operator inputs, and the spec says so itself.

**Guess required:** every flag spelling, every default, and every exit code.

**Line 2728, the spec's own words:** *"**Still unspecified:** the flag spellings
themselves, exit codes, and the format of the refusal messages §8 promises will
name the number that caused it."* **Line 2652:** *"A `grep` for `--[a-z]`
returns one hit, and it is the *deleted* locktime pair inside a retraction."* I
re-ran that grep: four hits, three of which are `md`'s `--template` (line 185),
the deleted `--timelocked`/`--immediate` pair (741), and `--transaction` (404);
`--quiet` appears exactly once, at line 579.

Named-but-unspecified behaviours that block a build:

| behaviour | line | what is missing |
| --- | --- | --- |
| `--quiet` | 579 | defined only for `decode`. Does it exist on `encode`/`verify`/`inspect`? Does it suppress the `stderr` **warnings** or only the inspection report? |
| grouping by N | 627–630 | no flag name, no default, and — critically — **is the grouped form what goes to stdout?** §0a rules stdout is the artifact; inserting spaces into it makes every downstream consumer strip them |
| `--transaction <psbt\|hex>` | 404 | the only fully-named flag, and it is optional on `verify` |
| free-text `TO` label | 2466 | *"Still to specify: the flag's name"* — by ruling this flag is what makes the label an act of assertion, so its absence removes the safety property |
| input values | 2668 | *"absent → refuse"*, with no way to supply them; per-input or total (line 1730) |
| `FROM` / `TO` identities | 2665–2666 | no input path |
| node location | 2670 | no input path |
| exit codes | 2728 | §1.1a's own pipeline `mt decode < plates.txt \| xargs bitcoin-cli sendrawtransaction` depends on them |

**Why this is a divergence and not just a gap.** The spec anticipates the
objection at line 2672 — *"two implementers given this table will still choose
different flag spellings, but they will at least build the same tool"* — which
is true of the *spellings* and false of the **grouping/stdout** and **`--quiet`
scope** questions, which are behaviour, not naming.

**Smallest addition.** A flags table in §10.10 with one row per input above,
plus two rulings: *"grouping affects `stdout`; the canonical artifact is the
ungrouped string and `decode`/`verify` accept both"* (or the reverse — either
is fine, but it must be one), and *"`--quiet` suppresses the inspection report
only; warnings and refusals are never suppressed."*

---

### I-11 — Input sniffing is a table of recognisers, not a decision procedure.

**Guess required:** given arbitrary bytes on stdin, which of the three forms is
this, and what normalisation happens first?

**Lines 1823–1832 (§8.2e)** give three recognisers — `psbt\xff` magic,
`cHNidP8` prefix, "bare hex, no magic" — and assert *"Each is distinguishable
by inspection, so `mt` sniffs rather than asking."* That is true of the three
canonical forms and silent about everything a user actually hands a tool:

- **Line-wrapped base64.** Many wallets and `openssl`-style exports wrap at 64
  or 76 columns. Is the `cHNidP8` test applied to the raw first bytes (works)
  or after whitespace removal (works) or per-line (fails)? Unstated. A accepts
  a wrapped `.psbt`, B refuses it.
- **Trailing newline / leading whitespace / CRLF.** A `.psbt` file written by
  a wallet, or a hex string pasted from a terminal, usually has one. Nothing
  says the input is trimmed. A bare-hex recogniser applied to `...ac00\n` sees
  a non-hex character.
- **Uppercase hex**, and a `0x` prefix — both plausible from a user, neither
  mentioned.
- **Hex-encoded PSBT** (`70736274ff…`) is simultaneously valid hex *and* a
  PSBT after decoding. It fails as a transaction, so the outcome is a refusal
  — but which refusal, and does the message name the real problem?

**Why it matters.** This is the tool's first contact with the user, in a
workflow the spec has gone to trouble to make forgiving (§8.2e exists because
refusing Core's default output *"was untenable"*). A refusal here is
indistinguishable from "your PSBT is bad".

**Smallest addition.** An ordered procedure: *"(1) Trim leading and trailing
ASCII whitespace. (2) If the first five bytes are `psbt\xff`, binary PSBT. (3)
Otherwise remove all interior whitespace; if the result matches
`^[0-9a-fA-F]+$` with even length, raw transaction hex; if it matches base64
and decodes to bytes beginning `psbt\xff`, base64 PSBT. (4) Otherwise refuse,
naming what it looked like."*

---

## MINOR

### M-1 — The set-prefix rule says 7 characters; every example shows 6.

Lines 144–153, 2705–2716: *"**the first 7 characters after `mt1` are the same on
every string**"*, illustrated with `mt1qzrf8x` — which is `mt1` plus **six**
characters (`q z r f 8 x`). Line 293's `MT1QZRF8XK2V…` agrees with six.

The rule is the correct one: `mt1`'s invariant fields are bits 0–36 and 7
symbols is 35 bits ⊂ invariant. **The cited evidence does not establish it,
though.** *"Verified on real `md1` output, where four chunks of one wallet all
read `md1fveszps…"* — `md1`'s invariant span is 31 bits, so only **6** symbols
are guaranteed; the 7th covers `index` bits 0–3, which are zero for every index
below 4. A four-chunk set is the one case where the test passes by accident.

Also unstated: does `mt` print a fixed 7 characters, or the *actual* maximal
shared prefix? For any set with `count ≤ 512` the true shared prefix is 8 or 9
characters, since the top 3 bits of the 12-bit `index` are also constant.

**Fix:** correct the example to 7 characters, mark the `md1` observation as
illustrative rather than verification, and state *"exactly 7, regardless of
`count`."*

### M-2 — The worked report's `SET` does not match its own `TX`.

Lines 479–480: `mt1 SET 0x0e17e` beside `TX
9a3f21c0d4e5b6a7…`. Under §10.13(c) — *"the top 20 bits of the txid in its
standard display form"* — the set id is `0x9a3f2`. `0x0e17e` corresponds to
nothing in the example. An implementer who checks their implementation against
the one worked report in the spec will conclude the truncation rule means
something other than what §10.13(c) says. The rule itself is stated precisely
enough; only the example is wrong.

### M-3 — Padding is located but not valued.

Line 2833–2837 says padding *"appears only once, at the end of a chunk, to
reach the next 5-bit symbol boundary"*. It does not say the pad bits are
**zero**, does not say they occupy the **low** bits of the final symbol
(derivable from "most-significant-bit first", but derivable is not stated), and
does not say whether a decoder must **reject** a non-zero pad. Encoders will
converge on zero; the divergence is in acceptance — A rejects a chunk with a
non-zero pad, B ignores it. `md-codec`'s `bits_to_symbols` left-justifies and
zero-pads the low bits and calls that *"the canonical form"*.

**Fix:** *"Pad bits are zero and occupy the low bits of the final symbol.
Decoders ignore their value."* (or "reject" — either, but pick one).

### M-4 — `~WINTER <year>` straddles a year boundary, and the estimate has no stated timezone.

Line 2016–2019 rules seasons northern-hemisphere and gives the mapping for one
of them: *"`~FALL 2034` means roughly September to November 2034"*. WINTER is
December–February, so `~WINTER 2034` is either Dec 2034 or Jan–Feb 2034 — a
twelve-month difference, engraved. The estimate's timezone is also unstated
(`MT_REF_TIME` is a Unix timestamp; the year presumably comes from UTC).

**Fix:** *"SPRING = Mar–May, SUMMER = Jun–Aug, FALL = Sep–Nov, WINTER =
Dec–Feb, all UTC; a December projection is labelled with the year it falls in."*

### M-5 — Unknown `version`, and `chunked = 0`, have no specified behaviour.

§10.13(a2) pins `version = 0b0001` and `chunked = 1, always`. Nothing says what
a decoder does with `version = 0b0010` (refuse and name the version? attempt
anyway?) or with `chunked = 0`. `md-codec` returns `WireVersionMismatch`; the
spec never adopts that. This is the field whose whole purpose is forward
compatibility, so the handling rule is the feature.

### M-6 — `decode` and `verify` disagree on foreign-set chunks.

Line 535 (`decode`): *"**reject** chunks from a different transaction"* — which
reads as *ignore them and carry on*. Line 192 (`verify`): *"every chunk carries
the same `chunk_set_id`"* — which reads as a refusal. An operator with two
transactions' plates mixed in one file gets a transaction from one verb and a
refusal from the other.

### M-7 — `inspect`'s accepted inputs are never enumerated, and the report table implies a caller with no strings.

§1.1 has `inspect` produce its report *"from an `mt1` string alone"* (singular),
while `TX` is marked *"present **always**"* (line 492) and is uncomputable from
an incomplete set. The row table's *"the caller had strings"* qualifier (line
491) implies a caller who does **not** — but all three named callers do.
Unstated: whether `inspect` also takes a PSBT or hex (it must, if `encode`
"invokes `inspect` on what it just produced" before any strings exist), and what
it reports for an incomplete set.

### M-8 — `STRING n OF m` and `n/m` are two spellings of one engraved string.

Lines 131–142 give both in the same table — *"per string | `n/m` — string `n` of
`m`"* in the row, `STRING n OF m` in the prose two lines below — while §10.8
(2521) mandates `n/m` for QR symbols. This is the exact defect §8.4 fixed for
`NO TIMELOCK` versus `NO BLOCK TIMELOCK` (lines 2033–2039), where the spec's own
reasoning is that a drifting engraved spelling means *"two `mt` versions would
cut different plates for the same transaction."*

### M-9 — The BCH generator, the polymod constants and the bech32 symbol ordering are never given.

The spec names `BCH(93,80,8)`, `t = 4`, 13 checksum symbols, Berlekamp–Massey
over `GF(1024)`, and `MT_REGULAR_CONST`. It never states the generator
polynomial, the polymod step constants, `hrp_expand`'s definition, or the
32-character alphabet ordering. An implementer forking `md-codec` (which
§10.13 directs) inherits all of them; an implementer working from the spec
must find them. The checksum construction itself —
`polymod(hrp_expand(hrp) || data || [0;13]) ⊕ CONST`, verified as
`polymod(hrp_expand(hrp) || data || checksum) == CONST` — is stated only by
citation to `chunk.rs:565,615` (line 2797), not in the document.

Compounding it: the plate legend says **`FORMAT: mt1 codex32`** and §5's
justification is that *"`codex32` is **BIP-93**, published and archived
independently of this project, so the tag stays findable."* A BIP-93 codex32
string carries a threshold character, a 4-character identifier and a share
index; an `mt1` string carries none of these and a BIP-93 parser will reject it.
The tag is durable and points at a specification that cannot read the artifact.

**Fix:** state the checksum construction in §10.13 as three lines of pseudocode,
and give the legend tag as `FORMAT: mt1 codex32-bch` or similar with one line
naming what it shares with BIP-93 (the BCH(93,80,8) code and the bech32
alphabet) and what it does not (the codex32 string layout).

### M-10 — §11 says the payload framing is still open; §10.13 says nothing is open.

Lines 3066–3072: *"`md-codec` chunks the output of `encode_payload`, which is a
**framed** payload — canonicalization plus TLV sections — not raw bytes. The
probe feeds the **raw transaction length** straight in, modelling **zero framing
overhead** for `mt1`. Whatever header `mt1` ends up carrying adds to the payload
and can therefore add chunks. **That is precisely open question §10.13, and it
must close before these counts are treated as final.**"*

§10.13 is closed (*"No longer blocking as a design question — it is now scoped
implementation work with every decision made"*, line 2901) and §10.13(a2)
answers it implicitly: 49-bit header, *"followed immediately by the chunk
payload with no padding between them."* An implementer reading §11 last
concludes a TLV layer is still to be designed and either stalls or invents one —
and an invented framing layer changes every string in the set.

**Fix:** one sentence in §10.13(a2): *"There is no payload framing. The chunk
payload is a byte-aligned slice of the raw transaction, with no TLV,
canonicalization or length prefix."* Then strike §11's last paragraph.

### M-11 — The absurd-fee refusal has no comparison operator or vsize rule.

Line 1689: *"an absurdly HIGH fee — `rust-bitcoin`'s own ceiling is
`DEFAULT_MAX_FEE_RATE = 25,000 sat/vB`"*, under a *"→ refuse"* heading.
Unstated: `≥` or `>`; how vsize is computed (`ceil(weight/4)` is standard but
unnamed); and whether the sat/vB figure is rounded before comparison. The same
applies to the 10 sat/vB warning threshold. Two implementations differ on
exactly one transaction each, which is small — but §8's closing rule is *"Every
refusal names the number that caused it"*, and a refusal whose comparison is
unstated cannot be reproduced by the operator.

### M-12 — §10.12's worked error-budget example uses the flat-40 model it elsewhere retracts.

Line 2752: *"A 535 B transaction **balanced at 40 B/chunk** is 14 chunks."* Under
balancing it is 39 B/chunk (§3b line 1110, same number 535). The chunk count and
the argument survive; the phrase is a third site carrying the flat-40 model and
should be swept with C-1.

---

## NIT

### N-1 — Decision 1b contains a duplicated sentence.

Lines 592–594: *"`mt qr` is deferred to its own cycle (§0a) because QR
conversion is a cross-format concern `md1` and `mk1` share."* appears twice,
the second copy starting mid-line 593.

### N-2 — The autocorrect example at lines 711–713 marks a character that is not in the string.

`corrected \`b\` -> \`6\` at position 12` against `mt1qzrf8xk2v...9d7b4...`,
with the caret at index 11. Index 11 is `v`, index 12 is `.`. Regenerate with
I-5's fixed convention.

---

## What this lens found that a correctness lens would not

Every Critical here is a case where **each section is internally right and the
document as a whole does not determine an output.** C-1 is two correct
descriptions of two different chunkers, 1,950 lines apart, each with a citation.
C-2 is a rule that is *stated nowhere* while being *described three times* —
correctness review reads the descriptions and finds them accurate. I-1 is a
document that rules on case four times and never for the stream it declares
normative. I-5 was invisible to reading and took a column-counting script.

The shape is consistent: **a correctness pass asks whether a sentence is true,
and every sentence here is true.** What is missing is the sentence that says
which of two true things is normative.

## Things I checked and did NOT file

Recorded so a later reader knows they were weighed:

- **Whether the 49-bit header overruns the BCH code.** It does not. A full
  chunk is 74 data symbols; the cap is 80 data / 93 codeword, and `hrp_expand`
  does not count toward the 93. Verified against `bch_decode.rs:22` and the
  `len > 93` floor at `:290`/`:420`.
- **Whether the last chunk can be empty.** It cannot, provably, under the
  §3b formula. See the method note.
- **Whether a chunk's payload byte count is recoverable from its string
  length.** It is, uniquely: `floor((5·data_symbols − 49)/8)`, and no two byte
  counts share a symbol count. Derivable from §10.13(a2) as written.
- **Bit order and field order of the header.** §10.13(a2) is exemplary — MSB
  first, order given, `count − 1` offset stated with its rationale, the dead
  `chunked` bit retained with an explicit warning against the optimisation that
  would break it. This is what the rest of §10.13 should look like.
- **`chunk_set_id` truncation.** *"The top 20 bits of the txid in its standard
  display form"* plus *"the internal byte order is the reverse of the displayed
  one"* is unambiguous. Only the worked example disagrees (M-2).
- **`MT_REGULAR_CONST`.** Derivation stated, recomputed in-document, width
  confirmed, distinctness from the sibling constants confirmed. Adequate.
- **§4 and §5 gaps** (module size, tiling, legend budget, `sysw` framing) —
  out of scope per the brief, deferred with `mt qr`.
- **Whether divergent chunk boundaries break *decoding*.** They do not — a
  decoder concatenates payload slices and is blind to the boundaries. This is
  why C-1 and C-2 are filed against the **length check**, which is where the
  incompatibility becomes visible, and why they would survive a round-trip test
  suite that did not cross implementations.

---

*Report written by the R6 implementability lens as its final action, per the
standing agent-persists-its-own-report rule. Nothing in the spec was edited.*
