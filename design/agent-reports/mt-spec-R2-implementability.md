# R2 — `mt` v0.1 spec, coherence and implementability lens

Artifact: `design/SPEC_mt_v0_1.md` at `b1790a4` (1357 lines, read in full).
Question answered: **could a competent implementer build exactly one thing from
this spec, and could a recoverer in 2040 get their money back?**

Cross-repo sources read: `descriptor-mnemonic/crates/md-codec/src/chunk.rs`,
`bitstream.rs`, `identity.rs`; `mnemonic-engrave/crates/me-cli/src/sysw/{record.rs,mod.rs,wire.rs}`,
`crates/me-cli/src/main.rs:791-900`; `design/SPEC_encrypted_payload_delivery.md:765-850`;
`design/SPEC_systemwide_payloads.md:515-545`; `design/measurements/mt-size-probe/src/bin/{select.rs,qrmodes.rs}`.

Per brief: numbering, citations, superseded terminology and the measured numbers
in `design/measurements/` were taken as settled and are not re-derived. Operator
rulings are treated as decisions; findings below are about **incoherence or
unimplementability**, not about the rulings themselves.

## Verdict

**6 Critical / 6 Important / 7 Minor / 0 Nit.**

Two implementers cannot build the same `mt qr` from this text — not because the
firmware is behind (§10.17, known and excluded), but because the bytes `mt`
itself writes are undefined at four independent layers: what a chunk is, how a
chunk becomes bytes, how those bytes are framed for the ruled transport, and how
the content id is derived. `mt string` is much closer to buildable; its gaps are
its stdout form and the absence of any decoder. The recoverer walk does not
complete for either verb in v0.1.

---

### C-1 — `mt qr`'s payload content is undefined, and base45 is inadmissible in the ruled transport as written

**Severity: Critical. Section: §10.9 (with §3, §4, §5).**

§10.9 rules the transport: *"That format is **`sysw`**, the system-wide payload
already used for every other constellation artifact"*, and names the gap it sees
as *"There is no transaction class, which is what R0 lens 4 found. Adding one is
the work."* That is not the whole gap, and the rest of it blocks the `mt` side
today, independently of §10.17's firmware work.

A `sysw` payload is **not** a typed byte blob. Its public section is
`record[0] LF record[1] LF … LF record[n-1]` (EPD §6.4,
`design/SPEC_encrypted_payload_delivery.md:765-775`), and class is not a tag on
the wire at all — it is **inferred by parsing each record string**
(`crates/me-cli/src/sysw/mod.rs:124-148`: prefix match, then `bip39` parse, then
`seal::record::validate_record`; anything else is `Class::Unknown`, and the
module doc at `record.rs:23-26` says `Unknown` is *"refused"*). So "adding a
`Class`" means **defining a record syntax**, and the spec defines none.

Three concrete divergences follow, each of which changes the emitted bytes:

1. **base45 collides with EPD §6.4 and the spec never notices.** EPD §6.4 is
   normative and quoted in `sysw`'s own spec: *"Every record MUST be the
   canonical, unbroken string — **no interior spaces, no hyphens, no grouping of
   any kind**."* The base45 alphabet (RFC 9285) is exactly the QR alphanumeric
   set — which is *why* §3 chose it — and that set **contains the space
   character** (value 36). A space therefore occurs about once per 45 output
   characters; §4's largest artifact is a 3809 B PSBT ≈ 5714 base45 characters,
   so a space is a certainty, not an edge case. Fed to `classify()` verbatim, the
   record is `Class::Unknown` and refused.
   `SPEC_systemwide_payloads.md:534-545` already hit this collision for free text
   and resolved it by **hex-encoding the body, explicitly refusing to exempt the
   rule** (*"the two new classes collide with EPD§6.4 — and are ENCODED, not
   exempted"*). Implementer A follows that precedent (`tx:` + lowercase hex,
   1.5× the base45 length on the wire); implementer B exempts the rule for the
   new class; implementer C strips the spaces — which EPD:786-795 documents as
   the specific mistake that turns a scratch into silently-absorbed damage.
   Three different payloads, one of which the device refuses.
2. **§4's selection has no channel.** §4 chooses module size, QR version, ECC
   level and an `across × rows` tiling. A `sysw` record is a canonical string; there
   is no field for any of those four values. So either the payload carries a
   *rendered* engraving (for which `sysw` has no encoding at all) or it carries
   the base45 text and the *device* re-derives the configuration — in which case
   §4's search is host-side computation that never reaches the machine, and the
   spec's own promise that the choice is made *"deterministically and with every
   tie broken, so two encoders agree"* (§2) buys nothing. The spec never says
   which. This is **not** §10.17: §10.17 says the firmware cannot *engrave* what
   §4 selects; this says `mt` cannot *serialize* it.
3. **The legend has no delivery mechanism either.** §5 engraves five fields on
   plate 1 and `PLATE n OF m` on later plates. Nothing states whether those
   strings travel as additional records (there is a `FreeText` class), as part of
   a transaction record, or not at all.

**Why it matters:** these are the bytes `mt qr` writes to a file. Until they are
specified there is nothing to implement, no test vector to write, and nothing for
the Rust-primary rule §10.9 invokes to bind the Go port *to*.

*Non-authoritative sketch:* one `tx:`-prefixed hex record per symbol, carrying
the chunk bytes, plus a one-line parameter record naming (module_mm, version,
ecc, across, rows) and the legend lines as `text:` records — chosen so every
record stays EPD §6.4-canonical.

---

### C-2 — for `mt qr`, the size of a chunk is never stated, and the obvious reuse contradicts §4's own table

**Severity: Critical. Section: §3 / §3b / §4.**

§3 fixes the pipeline as `mt qr: mt1 chunk -> bytes -> engraved as a QR symbol`,
and §4's caveat confirms one chunk per symbol (*"the **37-bit `mt1` chunk header
per symbol**"*). What is never stated is **how many payload bytes a `mt qr`
chunk carries.**

The only chunk-size rule in the spec is §3b's: *"A chunk carries **40 payload
bytes**, and the container holds **64 chunks**"* — the corrected
`CHUNK_PAYLOAD_BITS = 320` figure, presented in the string section but written as
a property of chunks. §10.13 then tells the implementer to fork `md-codec`'s
machinery, whose `split()` applies exactly that rule
(`chunk.rs:250-256`: `payload_bytes.len() * 8`, `div_ceil(SINGLE_STRING_PAYLOAD_BIT_LIMIT)`).

- Implementer A reads "one chunk per symbol" and sizes the chunk at the selected
  symbol's capacity: the 2769 B PSBT becomes **3 chunks in 3 symbols**, matching
  §4's table (4 plates, 3 qr, v21, ECC L).
- Implementer B reuses the forked chunker: the same PSBT becomes **70 chunks**,
  which is over the header's own limit and returns `ChunkCountExceedsMax`
  (`chunk.rs:257-261`) — so `mt qr` **refuses an artifact §4's table says takes
  four plates**. If B instead packs multiple chunks per symbol, chunk index and
  symbol index stop coinciding and §10.8's per-symbol `n/m` label becomes
  ambiguous (n of what?).

The 64-chunk ceiling is **not** a codex32-container property, as §3b implies
(*"the container holds 64 chunks"*) and §8.7b asserts (*"pointing at `mt qr`,
which has no such limit"*). It is in the shared header: `count` is a 6-bit field,
`1..=64`, validated in `ChunkHeader::write` (`chunk.rs:36-40, 53`). Since §3a
shares the header *"verbatim"*, `mt qr` inherits the cap; it is only harmless
under reading A, and the spec never picks A.

---

### C-3 — the byte-domain framing of a chunk is assumed, never specified (37 bits is not a byte boundary)

**Severity: Critical. Section: absent (§3, §3a, §10.13 all assume it).**

I searched the spec for a byte-framing rule: `grep -n "byte"`, `"pad"`,
`"align"`, `"MSB"`, `"framing"`. The only hits are §11's note that `md-codec`
chunks a *framed* payload and the probe models zero framing overhead. Nothing
defines how a chunk becomes bytes.

The header is **37 bits** — `[version:4][chunked:1][chunk_set_id:20][count-1:6][index:6]`
(`chunk.rs:1-10, 51-56`) — and 37 mod 8 = 5. In the string form this never
surfaces, because codex32 is a 5-bit domain and `wrap_payload` is handed an
explicit `chunk_bit_count = 37 + 8 * len` (`chunk.rs:288-292`). base45 is a byte
domain, so somebody must choose:

- Implementer A follows `BitWriter`: bit-packed MSB-first, every payload byte
  straddling a 5-bit offset, three zero pad bits at the end, `L = n + 5` bytes.
- Implementer B pads the header to 40 bits (5 bytes) and appends the payload
  byte-aligned — same length, **completely different bytes**, and trivially
  easier to implement without md-codec's bitstream.

Both are self-consistent; neither can read the other's plates. A 2040 recoverer
holding A's steel and B's decoder gets garbage that fails the §10.13 content-id
compare, with nothing on the plate to say why.

The same section-shaped hole covers the **4-bit `version` field's value for
`mt1`**. `ChunkHeader::read` hard-refuses any version other than
`Header::WF_REDESIGN_VERSION` (`chunk.rs:70-73`). §10.13 rules that `mt1` gets
its own NUMS constant and its own HRP but says nothing about the version field,
so A writes `4` (md1's) and B writes `0` or `1` for a new format, and each
decoder rejects the other's chunks outright.

---

### C-4 — the content id is "the txid" and nothing more: which txid, which 20 bits, which end

**Severity: Critical. Section: §10.13(c).**

§10.13(c) rules: *"`mt1`'s analogue is the **txid**… **Reassembly re-derives it
from the decoded transaction and compares**, giving `mt1` the same invariant
`md1` has"*, with *"Width stays at 20 bits."* The reduction from 256 bits to 20
is left unstated, and there is no default to fall back on that both implementers
would reach:

- `md1` takes *"the **top 20 bits** of the underlying 16-byte hash, MSB-first"*
  (`chunk.rs:170-180`), i.e. `((b0<<12) | (b1<<4) | (b2>>4))` over
  `sha256(payload)[0..16]`. Applying that to the txid's **internal byte order**
  and to its **display (reversed) form** — the form a human reads and the form a
  recoverer would compare against a block explorer — gives two different ids from
  the same transaction.
- Low 20 bits instead of high is a third reading, and hashing the txid again with
  a tag (the way `md1` hashes before extracting) is a fourth.

**And "the decoded transaction" is ambiguous for `mt qr`**, whose payload is a
PSBT. `txid(psbt.unsigned_tx)` and `txid(psbt.extract_tx())` are equal for segwit
inputs and **unequal for legacy inputs**, whose `scriptSig` is inside the txid
preimage and is empty in the unsigned tx. §10.20 already reasons about legacy
txid malleability but assumes the derivation is settled; §8.6's ruling accepts
legacy inputs, so this case is live rather than hypothetical.

**Why it matters:** the source calls this *"the content-id oracle;
funds-load-bearing invariant"*. A mismatch is a hard failure, so an encoder and
an independently written decoder that disagree here do not degrade gracefully —
the decoder **refuses a perfectly good plate**. This is the single most
likely-to-bite divergence in the recoverer walk, because the two implementations
are separated by years by construction.

---

### C-5 — who chooses the module size: §4 searches it and forbids 0.30 mm; §8.8 and §10.1 give it to the operator

**Severity: Critical. Section: §4 vs §8.8/§10.1.**

§4 puts module size **inside the search** and makes it the primary tie-break:

    search space:  module size x QR version (1..40) x ECC (L,M,Q,H) x rectangular tiling
    objective:     … 4. TIE-BREAK: maximise MODULE SIZE

and then, four paragraphs later: *"**Until that plate exists, `mt` must not
select a module below 0.60 mm** (two strokes)."*

§8.8 rules the opposite: *"**Module size is the operator's choice, defaulting to
0.60 mm** — not a refusal… `mt` offers every size it can engrave and suggests
0.60 mm."* §10.1 states the supersession but names only §8: *"So **§8.8's** hard
refusal below 0.60 mm becomes a **default and a recommendation**, not a floor."*
§4 was not updated and still says `must not`.

Two divergences, both of which change the steel:

1. **Floor.** A engraves at 0.30 mm when the operator asks (§8.8/§10.1); B
   refuses or clamps to 0.60 mm (§4). At half the module size a symbol needs a
   quarter of the area — this is a plate-count difference, not a rounding one.
2. **Role.** If the operator supplies the module size, it is an *input* and §4's
   tie-break 4 is dead code; if §4 searches it, the operator's choice is at most
   a floor and the tool may return 0.90 mm modules the operator never asked for.
   §4's measured table is captioned *"at the conservative 0.60 mm module"*, which
   reads as an input; the objective reads as a search dimension.

---

### C-6 — §8.7 refuses "over the plate budget" and no plate budget exists; the real ceiling is in the transport and is unmentioned

**Severity: Critical. Section: §8.7 (and §0's table).**

§8.7: *"**Over the plate budget (`mt qr`)** → refuse, naming the exact plate
count and what would fit."* §0's table gives `mt qr`'s size limit as *"the plate
budget"*. I grepped the whole spec for `plate budget` — **two hits, both of them
the ones above.** No number, no rule, no derivation. §4 minimises plate count but
imposes no cap.

Implementer A never refuses (there is no threshold), so a 40-input transaction
silently produces a 40-plate job — ~14 hours of engraving at §4's ~21 min/plate.
Implementer B invents a cap (4? 8? 16?) and refuses artifacts A engraves. §8's
own closing rule — *"Every refusal names the number that caused it"* — cannot be
satisfied by a refusal that has no number.

Worse, the ceiling that **does** exist is not the plate count and is not
mentioned anywhere in the spec: the ruled `sysw` transport caps a section at
`MAX_SECTION_LEN = 8191` bytes (`crates/me-cli/src/sysw/wire.rs:40-42`), enforced
today with the operator-facing message *"these records are too long for one
payload: a section caps at {} bytes. Split them across two payloads."*
(`crates/me-cli/src/main.rs:884-893`). At base45's 1.5× that binds at roughly a
5.4 KB PSBT — and at 2× (hex, per C-1's precedent) at roughly 4.0 KB, which is
**just past §4's largest measured artifact at 3809 B**. So `mt qr` has a hard
size limit, it is close to the working range, §3b and §8.7b both assert it does
not exist (*"a hard limit `mt qr` does not have"*, *"pointing at `mt qr`, which
has no such limit"*), and no refusal covers it.

---

### I-1 — the per-symbol `n/m` label has no geometry, and the one geometric constraint that matters is the quiet zone

**Severity: Important. Section: §10.8 / §5.**

§10.8 is explicitly **normative**: *"every engraved symbol carries its own
human-readable `n/m` beside it, for the chunk it holds… A lone symbol reads
`1/1`."* That settles existence, 1-basedness and the degenerate case. It does not
settle **where**, and §5 adds only *"beside **each QR symbol**"*.

§4 reserves `quiet zone: 4 modules per side, per symbol`. A label placed *inside*
that margin — the natural reading of "beside", and the cheapest in plate area —
puts engraved glyphs in the region the QR standard requires to be blank, and can
cost the scan. A label placed outside it consumes area §4 does not reserve.
Implementer A engraves `3/11` in the quiet zone at 2.6 mm and keeps §4's plate
counts; implementer B reserves a 4.4 mm line under each symbol and gets a
different tiling and possibly an extra plate. Both satisfy every sentence in the
spec.

(The *pricing* of the labels is §10.8/§10.14's known-open item and is not
re-reported here; the **placement rule and its interaction with the quiet zone**
are a separate and unstated thing.)

Format is also open — `3/11`, `3 OF 11`, `PART 3/11` — which matters only because
the recoverer must not confuse it with `PLATE n OF m` on the same plate.

---

### I-2 — §5 calls `FROM WALLET` mandatory and open while §10.4 closes it as optional, and nothing says whether the reservation shrinks

**Severity: Important. Section: §5 vs §10.4.**

§5's table row already carries the ruling: *"`FROM WALLET <8 hex>` … **Optional —
loudly warned when absent** (§10.4)"*. Sixty lines later the same section says:
*"**Where the stub comes from is unspecified, and that is an open question**, not
a settled design: `FROM WALLET` is a **mandatory field** sized into §4's
reservation, and nothing says what supplies it or what happens when it is
absent. See §10.4."* §10.4 is `CLOSED`, and closes it the other way.

The live consequence is not the wording: **`FROM` and `TO` are both optional, and
the spec never says whether §4 still reserves their lines when they are blank.**
Implementer A always reserves 6 lines / 25.5 mm, so a plate with two blank fields
wastes area that §4's objective would otherwise spend on ECC; implementer B
reserves only the lines it will engrave and gains one to two ECC levels — §4's
own note says reserving the legend *"drops small artifacts by two or three ECC
levels and doubles the plate count on the larger ones"*. Same transaction, same
flags, different ECC and possibly a different plate count.

---

### I-3 — `TO <wallet id, fp or label>  <amount>`: which amount?

**Severity: Important. Section: §5.**

`<amount>` appears exactly once in the spec (§5's legend table, line 458) and is
never defined. I grepped `amount` across all 1357 lines: every other hit is about
*input* amounts and the PSBT's UTXO records.

`mt` has no wallet knowledge — §0 removed construction, §8.2 removed script
evaluation, and §5 forbids branching on the stub — so it **cannot tell a payment
output from a change output**. For the RCW fixtures' 1-in/2-out and 5-in/2-out
shapes the candidates are: total of all outputs; the largest output; the sum of
outputs not matching `FROM`; or the first output. Implementer A engraves
`TO ALICE 0.51000000` (total including 0.26 change), implementer B engraves
`TO ALICE 0.25000000` (largest single output). Both are permanent, both are the
recoverer's headline fact about the plate, and they differ by the change.

§7 leans on this line as the *"Pinned destination"* mitigation (*"the plate
carries a summary"*), so an undefined summary weakens a named mitigation.

---

### I-4 — §7 credits an "engraved out-of-band reminder" that §5 does not engrave

**Severity: Important. Section: §7 vs §5.**

§7's "Wrong input value" row: *"Mitigated only by §8.2c's warning… **plus the
engraved out-of-band reminder**."* §8.2c's warning text ends *"Verify the input
value out of band, and **engrave a reminder** to re-check it before
broadcasting."*

§5's legend is **five fields, 130 characters, six lines**, all allocated, and none
of them is a reminder. There is no free-text legend mechanism except the flagged
`TO` label (§10.4), whose budget §10.4 itself computes at *"roughly 16"*
characters. For `mt string` there is no engraving at all (§3b), so the row's
mitigation cannot exist there even in principle.

Implementer A reads §8.2c as operator advice and engraves nothing — leaving §7
claiming a mitigation the artifact lacks. Implementer B adds a legend line, which
changes §4's reservation and therefore the ECC level and plate count.

This is the exact defect class §5's own retraction box says was swept: *"§7's
mitigations were written against the ten-field legend and were not re-read when
it became five… §7 below is corrected: it now claims only what §5 actually
engraves."* One row still does not.

---

### I-5 — v0.1 ships no decoder for either verb, so the recoverer walk cannot complete

**Severity: Important. Section: §10.10 / §10.2.**

§10.10's ruled CLI surface is two verbs: `mt qr`, `mt string`. §10.2 rules that a
reader arrives later — *"We will add another verb in the **next subversion** to
accept static scan data"* — and that ruling is about **scan** data only. Nothing
in the spec provides a path from **hand-typed `mt1` characters** back to a
transaction, for the verb whose entire purpose is hand engraving. §10.13
specifies reassembly semantics at the codec layer, so the library has the
machinery and the CLI never exposes it.

Consequently a plate cut by `mt` v0.1 is not recoverable by `mt` v0.1. That is a
scope decision the operator may well accept for the QR form, but no ruling covers
the string form, and §3b's ceiling analysis, §10.12's BCH-budget argument and
§10.13's content-id invariant all presuppose a decoder that the CLI does not
have.

**Related, and load-bearing for the same walk:** §10.2 keeps the claim *"F-234's
promise — that a recoverer with none of our tools can still read the plate — now
holds only for artifacts that fit **one** symbol."* Under §3's ruled encoding
that is false for one-symbol artifacts too: a single symbol carries
base45(37-bit `mt1` header ‖ PSBT bytes), which no wallet, no PSBT parser and no
UR decoder will accept. The promise now holds for **no** artifact, which is a
defensible cost of dropping UR but is recorded in the spec as if a residue
survived.

---

### I-6 — `mt string`'s stdout form is the verb's entire artifact and is unspecified

**Severity: Important. Section: §3b / §10.10.**

§0 says *"a `mt1` chunked codex32 string, **on stdout**"* (singular), §3b says
*"**`mt string` emits a string. That is the whole of its output**"*, and §10.10's
table says *"the **codex32 string on stdout**"*. But every measured artifact is
5–63 chunks, i.e. 5–63 separate codex32 strings, and nothing says how they are
delimited or whether they are grouped.

The constellation's precedent cuts both ways, which is why this diverges rather
than resolving itself: `md-cli` carries `--group-size` and `--separator`
(`crates/md-cli/src/main.rs:443-449`), and EPD:775-780 records that *"`mnemonic
bundle` defaults to `--group-size 5` and prints `md1fv 9wjpq pqpm6 …`, which is a
**display** form"* — and that the device *"rejects it outright"*.

Implementer A prints one canonical unbroken chunk per line; implementer B follows
the sibling CLI's default and prints grouped display form. The operator
**hand-engraves what they are shown**, and EPD:786-795 states exactly what that
costs: *"a stripped-then-engraved plate would carry separator characters the BCH
checksum never covered… a scratch or mis-strike that alters a separator would
then be silently absorbed rather than detected."* For a verb whose whole reason
to exist is BCH fault tolerance (§1.1b), that is the wrong end to be loose at.

---

### M-1 — §8.7b's chunk count is the pre-correction number

**Severity: Minor. Section: §8.7b vs §3b.**

§8.7b: *"Real wallets hit this: RCW `wsh` tier 1 at 5 inputs needs **78 chunks**
(§3b)."* §3b's table gives **89** for that artifact (3538 B). 78 is exactly the
retracted model: 3538 × 8 ÷ 363 = 77.97. The §3b correction did not propagate
into the refusal that cites it. No behavioural effect (both exceed 64), but it is
a false number inside a normative refusal, and it is the marker of an incomplete
fold.

### M-2 — §5 still describes the per-symbol label as naming "the UR part"

**Severity: Minor. Section: §5 vs §10.8/§3.**

§5:462 — *"one `n/m` label engraved beside **each QR symbol**, naming the **UR
part** it carries (§10.8's ruling)"*. §3 dropped UR entirely and §10.8 defines the
label as *"for the chunk it holds"*. An implementer reading §5 alone looks for a
UR part index that no longer exists.

### M-3 — §5's legend budget is 130 characters in one sentence and 136 in the next

**Severity: Minor. Section: §5.**

*"Five fields, **130 characters**, 6 lines"* (41+20+23+34+12 = 130), then *"Plus,
**not part of the 136-character budget above**"*. One of the two is wrong; §4's
reservation is derived from the line count rather than the character count, so
nothing downstream moves.

### M-4 — §7's "Pinned fee" row has three cells in a two-column table

**Severity: Minor. Section: §7.**

The table header is `| hazard | mitigation |` (line 619). The "Pinned fee" row
(line 625) carries a third cell, and it is not filler — it holds the only
statement in §7 that *"an `mt string` plate carries a raw transaction, from which
the fee is **not** recoverable without the prevouts"*. Every conforming Markdown
renderer drops it, so the rendered spec silently loses a real asymmetry between
the verbs.

### M-5 — the chunk-set id now identifies a transaction, and one transaction has two chunk sets

**Severity: Minor. Section: §3 / §10.13(c).**

§3 argues the header is *"strictly stronger than UR… so symbols from two
different transactions cannot be combined."* True. But once the id is the txid
(§10.13(c)), the **same** transaction's `mt qr` chunk set (PSBT payload, N
symbol-sized chunks) and `mt string` chunk set (raw-tx payload, M 40-byte chunks)
carry **identical** `(version, chunk_set_id)` and different counts. A recoverer
who owns both forms and feeds both to one reassembler is relying on a count
mismatch to be detected rather than on the set id to separate them. The header
has no field distinguishing payload type; in practice the PSBT magic `psbt\xff`
does.

### M-6 — the QR encoding mode is never fixed in §4's search space

**Severity: Minor. Section: §4.**

§4's search space is `module size x QR version x ECC x tiling`. Mode is absent.
§3's prose says base45 *"is pure QR alphanumeric text"* and the probe models it as
`Class::Alnum` (`design/measurements/mt-size-probe/src/bin/qrmodes.rs:52`), but no
normative sentence requires alphanumeric mode. An encoder library left to
auto-select generally picks alphanumeric here; one that defaults to byte mode
loses roughly a third of the capacity and changes plate counts.

### M-7 — symbol ordering within the tiling and across plates is unspecified

**Severity: Minor. Section: §4.**

§4 fixes the tiling as `across x rows` and nothing fixes which chunk lands where —
row-major, column-major, or column-then-plate. Implementer A's chunk 3 is
top-right, B's is bottom-left. Recoverability is unaffected *because* §10.8's
per-symbol labels exist, which is what keeps this Minor rather than Important; the
artifacts still differ visibly for the same input.

---

## The `mt qr` walk

Operator runs `mt qr <finalized.psbt>` (file or stdin, §10.10).

1. **Parse and refuse** — §8.1 (finalized), §8.2b (inputs ≥ outputs, absurd fee,
   duplicate outpoints, non-empty `vin`), §8.2c (missing input values, legacy
   warning), §8.4 (locktime facts on stderr), §8.5 (`gettxout` when a node is
   reachable), §8.6 (satisfaction binds outputs), §8.9 (secrets). **This part is
   buildable.** It is the most complete section of the spec: every check names
   its field and its message. Two soft spots, both charged to the known-open
   §10.10: how the operator supplies a missing input value, and how the node is
   addressed.
2. **Choose a configuration** — §4. **Breaks at C-5**: the implementer cannot
   tell whether the module size is an input or a search dimension, or whether
   0.30 mm is available. Also depends on I-2 (does the legend reservation shrink
   when optional fields are blank) and M-6 (which QR mode the capacity model
   assumes).
3. **Fragment** — **breaks at C-2**: nothing says how many payload bytes a `mt qr`
   chunk carries, and the one chunk-sizing rule in the spec refuses artifacts
   §4's own table sizes at four plates.
4. **Serialize each chunk to bytes** — **breaks at C-3**: 37 bits is not a byte
   boundary and no padding convention is given; the `version` field has no value
   assigned for `mt1`.
5. **base45-encode** — mechanical (RFC 9285), and the only step in the walk with
   no gap.
6. **Reach the machine** — **breaks at C-1**: the `sysw` record syntax for a
   transaction does not exist, base45 output violates the transport's normative
   canonical-record rule (spaces), §4's four selected parameters have no field to
   travel in, and the legend has no stated delivery. **Breaks again at C-6** if
   the payload exceeds the 8191-byte section cap the spec never mentions.
7. **Engrave** — §10.17, known open, correctly scoped as firmware work.

**Verdict on the walk:** the refusal engine could be built today. Everything from
"how many bytes in a chunk" to "what file `mt qr` writes" cannot. The gap is
`mt`-side, not firmware-side, and it is not covered by §10.9 or §10.17 as
written.

## The `mt string` walk

Operator runs `mt string <finalized.psbt>`.

1. **Parse and refuse** — same §8 set, plus §8.7b's 64-chunk refusal. Buildable
   (M-1 is a wrong number in the message, not in the rule).
2. **Extract the raw transaction** — §10.10 states this plainly and gives the
   reason (§8 needs PSBT vocabulary; the payload wants the smaller form). Clean.
3. **Fragment at 40 payload bytes** — specified, cited, corrected, and the
   correction is traceable to `chunk.rs:224,253-254`. The strongest passage in the
   spec.
4. **Header** — **inherits C-3's version-field gap and C-4's content-id gap.** The
   framing question (C-3's padding limb) does not bite here: codex32 is a 5-bit
   domain and `wrap_payload` takes an explicit bit count.
5. **BCH + codex32 with an `mt1` HRP and an `mt1` NUMS constant** — ruled in
   §10.13(a)(b), and the machinery is real (`bch.rs`, `bch_decode.rs`,
   `decode_with_correction`). The constants' *values* are implementation work, and
   the spec is right that it is scoped rather than open.
6. **Print** — **breaks at I-6**: the delimiter/grouping of a 5-to-63-chunk set is
   the whole artifact of this verb and is unstated, with the sibling CLI's default
   pointing at the display form EPD documents as hazardous to engrave.
7. **Warn on stderr** — §3b's bearer warning, well argued and well placed.

**Verdict on the walk:** `mt string` is close. Fix I-6, assign a version value,
and pin the txid reduction (C-4), and a competent implementer builds one thing.

## The recoverer walk

2040. Someone opens a drawer and finds steel.

**They cannot tell what they are holding.** §2 promises *"what is engraved
**beside** the symbols, so the plate is self-describing"*, but §5's five legend
fields are `BEARER…`, `FROM WALLET`, `LOCKED TO BLOCK`, `TO … <amount>` and
`PLATE n OF m`. **None of them names the format, the tool, the encoding or a
version.** Every other plate in the constellation self-identifies, because the
engraved string *starts with its own HRP* (`md1…`, `mk1…`). A `mt qr` plate
carries QR symbols and five human sentences, and not one character says "base45",
"PSBT", "mt", or "`mt1`". This is not on the open-questions list and is the first
thing the walk hits. (I looked for it: grep for `HRP`, `version`, `format`,
`identif` in §5 and §10.8 — the only HRP discussion is §10.13's `mt1` for the
string form.)

**`mt qr` path.** Scan the symbols → they must know it is base45 (nothing says so)
→ decode to bytes → they must know a 37-bit header precedes the fragment and how
it is padded (**C-3**) → order by `index` → concatenate → PSBT → extract →
broadcast. If they wrote their own decoder, **C-4** decides whether the content-id
compare passes or refuses their own plate. If they have `mt` itself, **I-5** says
v0.1 has no verb to do any of this; §10.2 promises the scan verb in the next
subversion and it is, as §10.2 itself says, *"what keeps multi-plate transactions
recoverable at all"*.

**`mt string` path.** Read characters off the plate → type them → **no verb
accepts them** (**I-5**). The BCH correction that justifies the entire verb
(§1.1b, §10.12) is unreachable from the shipped CLI. `md repair`-style correction
exists in the machinery and is not exposed.

**Inventory and completeness.** This is the part that works. §10.8's ruling is
the right mechanism and is stated normatively: per-symbol `n/m` plus
`PLATE n OF m` lets a recoverer name what is missing without decoding. Its
geometry is unstated (**I-1**), which is a divergence between implementers rather
than a hole in the recoverer's procedure.

**Where they end up.** With a complete set of `mt qr` plates and a correct
decoder, the money comes back. With a partial set, they can name what is missing
— genuinely better than UR would have given them. Without `mt`'s own reader, or
with a decoder that reduces the txid differently from the encoder, they hold a
plate that is provably intact and unreadable. The spec should say, on the plate,
what the plate is.
