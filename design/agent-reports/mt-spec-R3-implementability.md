# R3 — `mt` v0.1 spec, coherence and implementability lens

Artifact: `design/SPEC_mt_v0_1.md` at `9907348` (1585 lines, read in full).
Question answered: **could a competent implementer build exactly one thing from
this spec, and could a recoverer in 2040 get their money back?**

Read first, per the brief: `design/agent-reports/mt-spec-R2-implementability.md`
(593 lines), then the spec, then the sources the folds now rest on.

Cross-repo sources read and executed against:
`descriptor-mnemonic/crates/md-codec/src/chunk.rs:20-80`;
`descriptor-mnemonic/crates/md-cli/src/main.rs:105-118`;
`mnemonic-engrave/crates/me-cli/src/sysw/{record.rs,mod.rs,wire.rs}`;
`mnemonic-engrave/crates/me-cli/src/seal/record.rs:117-129`;
`mnemonic-engrave/design/SPEC_encrypted_payload_delivery.md:806-840`;
`seedhammer/seal/{wire.go,container.go,open.go,record.go}`;
`mnemonic-engrave/design/measurements/mt-size-probe/src/bin/select.rs:195-282`
and `RESULTS_ecc_selection_2026-08-22.txt:14-42`.

Per brief: numbering, citations, table shape and the measured numbers in
`design/measurements/` are settled and are not re-derived. Operator rulings are
decisions; every finding below is about **incoherence or unimplementability**,
never about a ruling. §10.1/§10.2/§10.10/§10.13(scope)/§10.14/§10.17/§10.20/§10.21
are known-open and are not restated as findings — where one is cited it is
because a *different* section contradicts it or because the gap sits outside its
stated scope.

## Verdict

**6 Critical / 8 Important / 5 Minor / 1 Nit.**

The R2 fold closed two of six Criticals outright (C-4 content id, C-6 plate
budget) and half of two more. The four remaining ones are unchanged in substance:
what a `mt qr` chunk is, how it becomes bytes, what `version` it carries, and
what bytes reach the `sysw` transport. On top of them the fold introduced two new
Criticals of its own — the base45→bech32 reversal did not reach three sections
that still rule base45 normatively (§3a, §10.3, §10.8), and the replacement
encoding collides with the *same* EPD §6.4 rule that killed base45, on a
different clause.

The single sharpest result of this round is machine-checked rather than argued:
the measurement that §3's encoding ruling and §8.7c's ceiling both rest on models
the largest `mt qr` artifact as **96 chunks** (`select.rs:205,254`), and the
header §3a shares "verbatim" caps `count` at **64** (`chunk.rs:28,37-39`). The
ruled encoding cannot be written by the ruled header, and §3b and §8.7b both
state that `mt qr` has no such limit.

`mt string` is one Important (its stdout form) and one version constant away from
buildable. `mt qr` is not buildable. The recoverer walk completes for neither.

---

## R2 Critical disposition

### R2 C-1 — `mt qr` payload content undefined; base45 inadmissible — **PARTIALLY CLOSED, and the closed half re-opened**

The **space** limb is genuinely closed: §3 reverses base45 for bech32 uppercase
and quotes EPD §6.4 correctly on the collision.

Everything else is open, and the fold added a new failure on the same rule:

- **Record syntax: still none.** §10.9 reads exactly as R2 quoted it — *"There is
  no transaction class, which is what R0 lens 4 found. Adding one is the
  work."* No prefix, no class name, no record grammar. Carried as **C-2**, now
  with the four device-side gates measured.
- **§4's four parameters still have no channel.** I grepped `tiling`, `across`,
  `render`, `firmware`: §2 promises the configuration is chosen *"deterministically
  and with every tie broken, so two encoders agree"*, and no section says how
  (module size, QR version, ECC, across × rows) travels. §10.17 covers the
  firmware's *inability to engrave* what §4 selects; it does not cover `mt`'s
  inability to *serialize* it. Carried in **C-2**.
- **Legend delivery: still unstated.** Carried in **C-2**.
- **New:** bech32 **uppercase** violates EPD §6.4's *all-lowercase* clause exactly
  as base45 violated its *no-space* clause. **C-4**.

### R2 C-2 — the size of a `mt qr` chunk is never stated — **OPEN, and now three-ways contradictory**

R2 found two readings. The fold added a third and the measurement supplies a
fourth. §3:180 now says *"What a symbol carries: `mt1` **chunks**"* (plural);
§4:445 says *"the 37-bit `mt1` chunk header **per symbol**"* (one); §10.8:1266
says each symbol is labelled *"for **the chunk** it holds"* (one); the probe
models 96 chunks across 5 symbols. Carried as **C-3**, upgraded — the probe's
model is now unwritable by the ruled header.

### R2 C-3 — byte-domain framing, and the `version` field's value — **OPEN, both limbs**

I re-ran R2's search over the current text: `grep -n "byte\|pad\|align\|MSB\|framing"`
returns no framing rule for `mt qr`, and `grep -n "version"` with the QR/draft
senses excluded returns **zero** hits assigning a value to the 4-bit field.
§10.13 rules (a) NUMS, (b) HRP, (c) content id and is titled *"RULED, ready to
build"* — `version` is not among the three. Carried as **C-5** and **C-6**.

### R2 C-4 — which txid, which 20 bits, which end — **CLOSED**

§10.13(c) now reads *"the id derives from the EXTRACTED transaction's txid"* and
*"**The top 20 bits of the txid in its standard display form** — the big-endian
hex a user reads. Stated to that precision because 'which 20 bits, from which
end' is exactly where two implementers diverge silently"*. Both limbs named and
resolved. This is the cleanest fold in the round. (R2's M-5 — one transaction,
two chunk sets, same id — was not folded; **C-16**, Minor.)

### R2 C-5 — who chooses the module size — **PARTIALLY CLOSED (floor yes, role no)**

The floor limb is closed: §4:461-465 now says *"0.60 mm (two strokes) is what
`mt` SUGGESTS — not a floor it enforces"*, with the missed-supersession recorded.

The **role** limb is untouched. §4:392-399 still lists `module size` inside the
search space and still carries *"4. TIE-BREAK: maximise MODULE SIZE"*, and §4's
own correction box still cites the measurement that **41 configurations tie once
the floor lifts** — a measurement that is meaningless if the operator supplies
the module size. Carried as **C-8**.

### R2 C-6 — "plate budget" undefined; the real ceiling unmentioned — **CLOSED as to definition; the fold's arithmetic is wrong**

Both halves were answered. §8.7 now defines the term: *"'Plate budget' means the
operator's stated maximum plate count… there is no fixed number"*. §8.7c is new
and cites `MAX_SECTION_LEN = 8191` correctly (verified: `sysw/wire.rs:42`).

But §8.7c's worked figure compares the wrong quantity and its headroom claim is
2.6× too generous, which moves the refusal boundary by ~2 KB of PSBT. **C-7**.

### R2 Importants — disposition in one line each

| R2 | verdict |
| --- | --- |
| I-1 `n/m` geometry vs quiet zone | **OPEN** — §10.8 still says only *"beside it"*; §5 still says only *"beside each QR symbol"*. **C-14** |
| I-2 `FROM WALLET` mandatory vs optional | **OPEN** — §5:570-573 still reads *"`FROM WALLET` is a **mandatory field** sized into §4's reservation"*. **C-13** |
| I-3 `TO … <amount>` — which amount | **OPEN** — `<amount>` still occurs once (§5:486) and is never defined. **C-12** |
| I-4 §7 credits an engraved reminder §5 does not engrave | **CLOSED in §7** (row rewritten, *"Nothing reaches the steel for `mt qr`"*), **but the same sentence survives in §10.16:1480**. **C-15**, Minor |
| I-5 no decoder for either verb | **CLOSED as disclosure** — §9 now states it plainly and §10.21 records the missing format tag |
| I-6 `mt string` stdout form | **OPEN** — no delimiter, grouping or casing rule anywhere. **C-11** |
| M-1 78 vs 89 chunks | **CLOSED** — §8.7b now reads 89 |
| M-2 §5 "the UR part" | **CLOSED** — now *"naming the `mt1` chunk it carries"* |
| M-3 130 vs 136 characters | **CLOSED** — 41+20+29+34+12 = 136, and §8.4 records the 130→136 move |
| M-4 §7 three-cell row | **CLOSED** — every §7 row now has exactly 3 pipes |
| M-5 two chunk sets, one id | **OPEN**, **C-16** |
| M-6 QR mode never fixed | **OPEN and now load-bearing**, because the ruled encoding only reaches alphanumeric mode when uppercased. Folded into **C-4** |
| M-7 symbol ordering | **OPEN**, unchanged, still Minor — not re-reported |

---

### C-1 — three sections still rule the QR payload as base45, including the CLOSED open-question that decides it

**Severity: Critical. Section: §3a:262, §10.3:1176-1181, §10.8:1248 vs §3.**

`grep -n "base45"` returns 9 hits. Six are §3's own retraction, correctly framed
in the past tense. Three are live normative statements:

1. **§3a:262**, the pipeline block that §3a exists to fix as the format's
   definition:

       mt qr:      chunk header + payload -> base45 -> QR (Reed-Solomon) -> modules

2. **§10.3:1176-1181** — the open question titled *"What goes in the QR?"*, marked
   **CLOSED**, whose whole content is the answer:

   > *"UR is dropped (§3), and the QR payload is **`mt1` chunks, base45-encoded** —
   > operator ruling 2026-08-23… base45 was chosen over 3%-denser raw binary for
   > scanner compatibility and its ~28% intrinsic detection of corrupted triples.
   > **§10.1's test plate should still confirm scanners read base45 off engraved
   > steel** — the choice is made, the optical validation is not."*

3. **§10.8:1248** — *"for `mt qr` it rides in the base45 payload."*

**The divergence.** §10 is where this spec puts its rulings, and §10.3 is the one
question whose subject is precisely this decision. Implementer A reads §3 and
emits bech32 uppercase; implementer B reads §10.3 — a CLOSED ruling naming an
operator date, with a residual optical-validation action item attached to it —
and emits base45. Different bytes, different QR capacity (86.0% vs 80.7%,
`RESULTS_ecc_selection_2026-08-22.txt:41-42`), different plate counts on 2 of 5
measured artifacts, and B's payload is refused by the transport for the space
collision §3 was written to fix. A reader who resolves the conflict by date
cannot: both carry 2026-08-23.

This is not a stale-wording nit. §10.3 also **schedules work** against base45
(the test plate), so the residue propagates into the task list.

---

### C-2 — what bytes `mt qr` writes is still undefined, and every obvious framing is refused by a device-side gate the spec never names

**Severity: Critical. Section: §10.9 (with §3, §4, §5).**

§10.9 states the remaining work as *"There is no transaction class… **Adding one
is the work**"*, and §10.17 repeats it. I traced the actual admission path for an
**unencrypted public-section** payload — which is what §10.9's ruling produces —
and it is four gates, not one. All four are executed code, cited by line:

| gate | where | what it requires |
| --- | --- | --- |
| `SplitSection` | `seal/container.go:58,78`; consts `seal/wire.go:57-58` | `MaxRecords = 24`, `MaxRecordLen = 512`, no empty record, no CR |
| public allow-list | `seal/record.go:465-467` | *"the allow-list… admits only ClassMDMK into the public section"* |
| `cardKey` | `seal/record.go:444-472` | dispatches on `codex32.ValidMD` / `ValidMK` **only**; anything else → `ErrUndecodableCardSet` |
| `decodePublicSet` | `seal/record.go:475,494-517` | *"every public record belongs to a card set that REASSEMBLES AND DECODES"*, grouped by `(HRP, chunk_set_id)` |

The host side agrees: `sysw/mod.rs:124-148` classifies by prefix → BIP-39 →
`seal::record::validate_record`, and that function rejects **any uppercase
character** outright (`seal/record.rs:122-128`, `RecordError::NotLowercase`).

None of `MaxRecords`, `MaxRecordLen`, the public allow-list or EPD §6.3's
card-set rule appears anywhere in the spec — `grep -n "512\|24 record\|card set"`
returns nothing. The spec cites exactly one transport constraint (§8.7c's 8191)
and one transport rule (§6.4's no-space clause).

**The divergence, and it is not hypothetical.** The spec gives an implementer no
record boundary, so both natural choices are refused:

- **One record per chunk** — the constellation's own convention, and the only
  framing that can satisfy EPD §6.3's grouping by `(HRP, chunk_set_id)`. The
  largest §4 artifact is **96 chunks** (`select.rs:205`), and `MaxRecords = 24`
  refuses it. The cap binds at **24 chunks ≈ 960 B of PSBT** — smaller than five
  of §4's seven measured artifacts.
- **One record for the whole stream** — 6,863 characters for that artifact
  (arithmetic in C-7), against `MaxRecordLen = 512`. Refused.
- **The stream sliced into ≤512-byte records** — 14 records, inside both caps,
  but the slices have no HRP and no chunk header, so `cardKey` returns
  `ErrUndecodableCardSet` and `decodePublicSet` cannot run.

Implementer A ships one-record-per-chunk and every artifact over ~960 B fails to
load. Implementer B ships 512-byte slices and *every* artifact fails to load.
Implementer C invents a fifth thing. All three write a file; none of them
produces steel.

**And §4's four selected parameters and §5's legend still have no channel.** A
`sysw` record is a canonical string. §2 lists *"which (module size, QR version,
ECC level, tiling) configuration is chosen"* as one of the five things this codec
exists to specify, and no section says whether that configuration travels in the
payload (in what field?) or is re-derived on the device (in which case §2's
"so two encoders agree" buys nothing and §4's search is host-side computation
that never reaches the machine). Same for the five legend lines and
`PLATE n OF m`.

**Why this is not §10.17.** §10.17's scope is *"the firmware cannot yet engrave
what §4 selects"* — a device capability. The gap here is the byte string `mt`
writes, which §10.9's own Rust-primary ruling says lands **first**, in `me-cli`,
with test vectors. There is nothing to write a test vector against.

---

### C-3 — how many chunks a `mt qr` symbol holds: three normative statements, three answers, and the measured one is unwritable

**Severity: Critical. Section: §3:180 vs §4:445 vs §10.8:1266, vs the measurement.**

| source | says |
| --- | --- |
| §3:180 | *"What a symbol carries: `mt1` **chunks**, bech32 UPPERCASE"* — plural |
| §4:445 | *"the **37-bit `mt1` chunk header per symbol**"* — one header per symbol |
| §10.8:1266, **normative** | *"every engraved symbol carries its own human-readable `n/m` beside it, **for the chunk it holds**"* — one chunk per symbol |
| `select.rs:205,254` | `chunks = raw.div_ceil(40)` → **96 chunks** for the 3809 B artifact, which §4's table places in **4 symbols** |

The measured reading is the one §3's encoding ruling and §8.7c's ceiling both
rest on — and it **cannot be encoded**. `ChunkHeader::write` validates
`(1..=64).contains(&count)` and returns `ChunkCountOutOfRange` otherwise
(`chunk.rs:37-39`); `count` is a 6-bit field (`chunk.rs:28,55`). 96 > 64.

Meanwhile §3b:299 says *"**The 64-chunk ceiling is a hard limit `mt qr` does not
have**"* and §8.7b:1078 refuses `mt string` while *"pointing at `mt qr`, which
has no such limit"*. Both are false under §3a's own rule that the header is
shared *"verbatim"*.

**The divergence.** Implementer A sizes a `mt qr` chunk at the selected symbol's
capacity — one chunk per symbol, ≤64 symbols, §4's table reproduces, §10.8's
label is well-defined. Implementer B reuses the forked 40-byte chunker as the
measurement does — and `mt qr` **refuses the 2769 B artifact §4's own table sizes
at four plates**, because 70 > 64. Implementer C packs many chunks per symbol to
dodge the cap, and then §10.8's normative label has no referent: a symbol holding
19 chunks cannot carry *"`n/m` for the chunk it holds"*.

The recoverer consequence of C is the one that bites: the per-symbol label is the
mechanism §10.8 introduces so that *"a recoverer must be able to inventory what
they hold and name what is missing without decoding anything"*. Under C there is
nothing to write on the label.

---

### C-4 — bech32 UPPERCASE collides with EPD §6.4's all-lowercase rule, the same rule that killed base45

**Severity: Critical. Section: §3:212-221 vs EPD §6.4.**

§3 justifies the replacement encoding with a three-row table:

| constraint | bech32 uppercase |
| --- | --- |
| **EPD §6.4** — no interior spaces; every character inside the checksum | ✓ 32-character alphabet, no space |
| **EPD §6.6** — records hashed in canonical **lowercase** | ✓ case-insensitive by design; uppercase→lowercase is **lossless**, verified 1:1 |
| **QR alphanumeric** — for 11-bits-per-2-characters packing | ✓ when uppercased |

Row 1 states EPD §6.4 as *"no interior spaces"*. That is one clause of it.
`SPEC_encrypted_payload_delivery.md:806-825` lists §6.4's normative constraints,
*"all checked before any record is acted on"*, and the clause after the space
rule is:

> **All-lowercase.** … *"Verified: `md1qqqsyqcyq5rq…` and `MD1QQQSYQCYQ5RQ…` both
> return `ValidMD = true` and hash differently… **Pinned here at §6.4, not inside
> §6.6**, so the engraved artefact and the hash agree by construction."*

The spec attributes lowercase to §6.6 (hashing) and therefore treats it as a
property that *survives* rather than a rule that *binds*. It binds, it is
enforced (`seal/record.rs:122-128` rejects any uppercase character with
`NotLowercase`), and it was pinned in §6.4 precisely so that it could not be
argued away as a hashing detail.

**The divergence, and it changes the plate.** The spec never says which case the
record carries:

- **Uppercase on the wire** — the measured 80.4% holds and the QR reaches
  alphanumeric mode, and the record is refused by the transport for the same
  class of reason base45 was.
- **Lowercase on the wire** — admissible, but then something must uppercase the
  string before QR encoding, and nothing says what. If the device encodes the
  record as received, the QR falls out of alphanumeric mode into **byte mode**:
  a bech32 character carries 5 data bits in 8 QR bits (62.5%) instead of 11 bits
  per 2 characters (90.9%). Every capacity figure in §3's table and every row of
  §4's plate table is then wrong by ~1.45×.

R2's M-6 ("the QR encoding mode is never fixed in §4's search space") was rated
Minor when the alphabet was base45, which is *natively* the QR alphanumeric set.
Under bech32 uppercase the mode is no longer a library default question — it
depends on a case transform that has no owner in the spec. That is why it is
Critical here rather than Minor.

---

### C-5 — the byte/bit framing of the 37-bit header is still unspecified, and the measurement quietly picked one

**Severity: Critical. Section: absent (§3, §3a, §10.13 all assume it).**

Unchanged from R2 C-3's first limb, with one addition: the probe now **commits**
to a convention that no sentence in the spec states.

    // (2) header+payload as BYTES, no BCH. 37 bits rounds to 5 bytes/chunk.
    let bin_bytes = raw + chunks * 5;                    // select.rs:263-264

That is R2's implementer B — pad the 37-bit header to 40 bits and append the
payload byte-aligned. Implementer A follows `md-codec`'s `BitWriter`, which is
what §10.13 tells the implementer to fork: bit-packed MSB-first, payload bytes
straddling a 5-bit offset, pad bits at the end. Same length, completely different
bytes, mutually unreadable.

bech32 adds a second, independent padding question that base45 did not have:
bech32 is a **5-bit** domain and 37 mod 5 = 2, so the bit-count handed to the
encoder needs a rule as well. `md-codec` solves this for the string form by
passing an explicit `chunk_bit_count = 37 + 8·len` to `wrap_payload`; the QR form
has no such call site and no stated equivalent.

**The recoverer consequence.** A 2040 recoverer holding A's steel and B's decoder
gets bytes that fail §10.13's content-id compare — which §10.13 calls *"the
content-id oracle; funds-load-bearing invariant"* — with nothing on the plate to
say why. The failure mode is a refusal of a perfectly intact plate, not
degradation.

---

### C-6 — the 4-bit `version` value for `mt1` is unassigned, and the decoder hard-refuses a wrong one

**Severity: Critical. Section: §10.13.**

§10.13 is titled *"`mt1`'s own encoding, NUMS constant and content id — RULED,
ready to build"* and rules exactly three things: (a) NUMS constant, (b) HRP,
(c) content id. `grep -n "version"` over the whole spec, with the QR-version and
draft-version senses removed, returns **no** assignment for the header's 4-bit
`version` field.

`ChunkHeader::read` does not treat it as advisory:

```rust
let version = r.read_bits(4)? as u8;
if version != Header::WF_REDESIGN_VERSION {
    return Err(Error::WireVersionMismatch { got: version });
}
```
(`chunk.rs:69-73`)

Implementer A forks `md-codec` and inherits `WF_REDESIGN_VERSION` (the doc
comment at `chunk.rs:24` records v0.30 = 4). Implementer B treats `mt1` as a new
format at version 0 or 1. Each decoder rejects the other's chunks on the first
four bits read, before the HRP or the NUMS constant ever matters — so the two
distinguishing features §10.13 *does* rule cannot save it.

This is one constant. It is Critical only because §10.13 declares itself complete
and an implementer therefore has no prompt to ask.

---

### C-7 — §8.7c's ceiling check compares QR capacity against a record-length cap; the headroom is 16%, not 40%

**Severity: Important. Section: §8.7c.**

§8.7c:1068-1074:

> *"**Over the `sysw` section ceiling (`mt qr`)** → refuse. `MAX_SECTION_LEN =
> 8191` bytes… §4's largest measured artifact — RCW `wsh` tier 1 at five inputs,
> **4,719 B once chunked and bech32-encoded** — sits inside it with **roughly 40%
> headroom**."*

4,719 is not the record length. It is **QR capacity consumed**, and the results
file says so in words: `bytes + bech32U   4719 B in QR  eff 80.7%`
(`RESULTS_ecc_selection_2026-08-22.txt:42`). The probe computes it as
`(b32_chars.div_ceil(2) * 11).div_ceil(8)` — alphanumeric packing, 11 bits per 2
characters (`select.rs:271-272`). `MAX_SECTION_LEN` counts **bytes of record
text**, one byte per character.

Re-run from the probe's own model:

| quantity | value |
| --- | --- |
| PSBT | 3,809 B |
| chunks (`select.rs:254`) | 96 |
| header+payload bytes (`select.rs:264`) | 4,289 |
| **bech32 characters = record bytes** (`select.rs:271`) | **6,863** |
| QR capacity (`select.rs:272`) | 4,719 |
| headroom vs 8191, using 4,719 | 42.4% |
| **headroom vs 8191, using 6,863** | **16.2%** |
| records needed at `MaxRecordLen = 512` | 14 of the 24 permitted |

**The divergence.** Implementer A reproduces §8.7c's worked example and refuses
above the point where the **QR-capacity** figure crosses 8191 — first PSBT
refused at **6,617 B**. Implementer B compares record bytes — first PSBT refused
at **4,550 B**. A 5,000-byte PSBT is engraved by A and refused by B; A's operator
gets a payload the device will not load, discovered at the machine.

Two further consequences of the same error. §8.7c asserts *"sits inside it with
roughly 40% headroom"*, which reads as reassurance that the working range is
clear — it is 16%, before any record framing, LF separators or class prefix is
counted (C-2), all of which only add. And the ceiling §8.7c names is not the
binding one: `MaxRecords = 24` binds first under the only framing that satisfies
EPD §6.3, at ~960 B of PSBT.

---

### C-8 — module size: §4 searches it and tie-breaks on it; §8.8 and §10.1 give it to the operator

**Severity: Important. Section: §4:392-399 vs §8.8/§10.1.**

R2 C-5's floor limb is closed. The role limb is not, and the fold's own edit made
the two halves of §4 disagree with each other.

§4:392-399, unchanged:

    search space:  module size x QR version (1..40) x ECC (L,M,Q,H)
                   x rectangular tiling (across x rows)
    objective:     … 4. TIE-BREAK: maximise MODULE SIZE

§4:462-464, new: *"**Operator ruling 2026-08-23 (§10.1, §8.8): the operator picks
from every size `mt` can engrave, with 0.60 mm suggested.**"*

If the operator picks it, module size is an **input**, the search runs over
version × ECC × tiling only, and objective step 4 is dead — along with §4's
correction box, which justifies steps 4 and 5 by the measurement that *"**41
tie** once the floor lifts"*, a count that only exists if module size is being
searched across.

**The divergence.** Implementer A takes the module size as an input defaulting to
0.60 mm; every symbol is engraved at the operator's chosen size. Implementer B
searches it and returns the largest module among tied configurations; a 162 B
payload comes back at whatever the top of the ladder is, not at 0.60 mm.
Physically different plates for the same transaction and the same flags, and B
can hand back a size the operator never chose while §8.8 says the choice is
theirs.

---

### C-9 — the legend's no-timelock line is spelled two ways, both normatively, in the same section

**Severity: Important. Section: §8.4:903 vs §5:485 and §8.4:981.**

Three statements about what is **engraved**, permanently, on steel:

- §5:485, the legend table: *"Reads **`NO BLOCK TIMELOCK`** when there is no
  enforced `nLockTime`."*
- §8.4:981, the closing note of the section that owns this rule: *"The **legend
  now reads `NO BLOCK TIMELOCK`**: precisely true about the fields `mt` read, and
  silent about scripts it did not."*
- §8.4:902-903, the bullet **labelled `Legend:`**, rewritten by the
  height/timestamp fold: *"`LOCKED TO BLOCK <n> ~<year>` for a height,
  **`LOCKED UNTIL <time>`** for a timestamp, or **`NO TIMELOCK`**."*

The `stderr` report at §8.4:889 also shows `NO TIMELOCK`, and that one is
correct — it is the terminal form. The defect is that the bullet naming itself
`Legend:` uses the terminal form for the engraved one.

**The divergence.** Implementer A engraves `NO BLOCK TIMELOCK` (17 characters);
implementer B engraves `NO TIMELOCK` (11). Different steel, forever, on the field
§5 calls *"the single most actionable fact"* — and the two strings do not mean
the same thing to a 2040 reader: `NO BLOCK TIMELOCK` is a claim scoped to
block-height fields, which is exactly the scoping §8.4:981 argues for at length,
and `NO TIMELOCK` is the unscoped claim that paragraph refuses.

---

### C-10 — the unlock estimate has no rule for a target below the reference height, and the spec's own worked example of that case drops the year the legend mandates

**Severity: Important. Section: §8.4:913-954.**

The estimate is specified as one line:

    estimated unlock  =  reference_time + (target_height − reference_height) × 600 s

and the legend form as `LOCKED TO BLOCK <n> ~<year>` (§5:485, §8.4:902).

Four things are unstated, and the fourth diverges observably:

1. **Where the pair comes from.** *"`mt`'s binary embeds a reference
   `(height, unix_time)` pair **at build time**"*. No source, no update rule.
   Implementer A hardcodes a constant; implementer B queries a node in a build
   script and gets a non-reproducible, network-dependent build. (Minor on its own
   — the engraved year is unchanged whenever the chain ran near target.)
2. **What `reference_time` is when a node is reachable.** *"When a node is
   reachable, `mt` uses the live height instead"* — the live *height*. The paired
   time is unstated: host wall clock, the node's MTP (which §8.4 already fetches),
   or the best block's header time. (Minor; the three differ by hours.)
3. **Which calendar the year is read in.** UTC or local. (Nit.)
4. **The already-passed case, which is the divergence.** `(target − reference)` is
   negative whenever the lock has passed — and §8.4:951-954 raises exactly that
   case:

   > *"**A lock that has already passed is reported the same way**, because the
   > two numbers say so: `LOCKED TO BLOCK 900000, current height 963663` is a
   > plate that is live now."*

   That worked example carries **no `~<year>`**, while the legend form the same
   section defines eleven lines earlier requires one. So implementer A engraves
   `LOCKED TO BLOCK 900000 ~2022`, implementer B follows the worked example and
   engraves `LOCKED TO BLOCK 900000`, and implementer C suppresses the estimate
   whenever it lands in the past on the reasoning that a "projected" past date is
   absurd. Three different permanent legends for the same transaction.

   The arithmetic limb compounds it: `target_height` and `reference_height` are
   block heights, naturally `u32`, and the subtraction underflows. In a debug
   build that is a panic on a transaction §8.4 explicitly expects to see; in a
   release build it is a year roughly 81,000 years out, engraved. §4's own
   correction box records that this spec has already shipped one *"permanent
   falsehood on steel"* from a missing branch on a locktime value
   (`nLockTime = 1800000000` → `LOCKED TO BLOCK 1800000000`); this is the same
   shape, on the field that fold introduced.

---

### C-11 — `mt string`'s stdout form is the verb's entire artifact and is still unspecified

**Severity: Important. Section: §3b / §10.10.**

Unchanged from R2 I-6; re-verified by grep. `stdout` occurs four times
(§0:25, §3b:365-368, §10.10:1314), always as *"the codex32 string on stdout"*,
singular. §3b:346 says *"**`mt string` emits a string. That is the whole of its
output**"*. Every artifact in §3b's own table is **5 to 63 chunks**, i.e. 5 to 63
separate codex32 strings. No delimiter, no grouping rule, no casing rule.

The sibling precedent is the wrong default and is machine-checked:
`md-cli`'s `encode` carries `--group-size` with `default_value_t = 5` and
`--separator` defaulting to `space` (`md-cli/src/main.rs:110-116`). So an
implementer who mirrors the sibling prints `mt1fv 9wjpq pqpm6 …`.

**The divergence, and it is funds-relevant rather than cosmetic.** The operator
hand-engraves what they are shown, and EPD:794-796 states the cost in the same
words §3 quotes to justify dropping base45: *"a record carrying separator
characters the BCH checksum never covered turns a scratch on the operator's only
copy into silently-absorbed damage rather than a detected error."* §3 applied
that reasoning to a transport rule and did not apply it to the one output this
verb has. For a verb whose stated reason to exist is BCH fault tolerance (§1.1b),
grouping characters outside the checksum's coverage is the specific failure it
was built to prevent.

Casing is the second half: EPD §6.4 pins lowercase for records and records
*"Lowercase is what `mnemonic bundle --group-size 0` emits"*, while the device's
own keyboard path emits uppercase. §3b says nothing, so A prints lowercase and B
prints uppercase, and a recoverer typing one into a tool expecting the other is
a case-fold away from a support problem.

---

### C-12 — `TO <wallet id, fp or label>  <amount>`: which amount

**Severity: Important. Section: §5:486.**

Unchanged from R2 I-3, re-verified: `grep -n "amount"` returns 14 hits, of which
exactly one is this field (§5:486) and one is its budget (§10.4:1215, *"34
characters including the amount"*). Every other hit is about **input** amounts,
the PSBT's UTXO records, or BIP-143's commitment.

`mt` cannot tell a payment output from a change output — §0 removed construction,
§8.2 removed script evaluation, and §5 forbids branching on the stub. So for the
RCW's own 1-in/2-out and 5-in/2-out fixtures the candidates are: the total of all
outputs, the largest single output, the sum of outputs not matching `FROM`, or
the first output. Implementer A engraves `TO ALICE 0.51000000`; implementer B
engraves `TO ALICE 0.25000000`. Both permanent; they differ by the change.

§7's *"Pinned destination"* row leans on this line — *"the plate carries a
summary"* — so an undefined summary weakens a named mitigation.

---

### C-13 — §5 still calls `FROM WALLET` mandatory while its own table and §10.4 make it optional, and nothing says whether the reservation shrinks

**Severity: Important. Section: §5:570-573 vs §5:485 and §10.4.**

Unchanged from R2 I-2. §5:485's table row: *"**Optional — loudly warned when
absent** (§10.4)"*. §5:570-573, eighty lines later:

> *"**Where the stub comes from is unspecified, and that is an open question**,
> not a settled design: `FROM WALLET` is a **mandatory field** sized into §4's
> reservation, and nothing says what supplies it or what happens when it is
> absent. See §10.4."*

§10.4 is `CLOSED` and closes it the other way, with the operator's words.

**The live consequence is the reservation.** `FROM` and `TO` are both optional;
§4:401 reserves *"6 lines… on plate 1"* flat, and `grep -n "reserv"` returns
nothing that conditions the reservation on which fields are populated.
Implementer A always reserves 6 lines / 25.5 mm; implementer B reserves only the
lines it will engrave. §4:453-455 prices that difference in its own words:
reserving the legend *"drops small artifacts by two or three ECC levels and
doubles the plate count on the larger ones"*. Same transaction, same flags,
different ECC level and possibly a different plate count.

---

### C-14 — the normative `n/m` label has no placement rule, and the one geometric constraint that matters is the quiet zone

**Severity: Important. Section: §10.8:1266 / §5:490-493.**

§10.8 is explicitly **normative** and settles existence, 1-basedness and the
degenerate case (*"A lone symbol reads `1/1`"*). It does not settle where, and §5
adds only *"beside **each QR symbol**"*.

§4:400 reserves `quiet zone: 4 modules per side, per symbol`. A label inside that
margin — the natural reading of "beside", and the cheapest in plate area — puts
engraved glyphs where the QR standard requires blank, and can cost the scan on
the artifact that has no other reader (§9: v0.1 ships no decoder). A label
outside it consumes area §4 does not reserve. Implementer A engraves `3/11` in
the quiet zone and keeps §4's plate counts; implementer B reserves a line under
each symbol and gets a different tiling.

Font size is unstated too, and §10.8 prices the labels only by character count
(*"3–5 characters"*), which does not determine area.

The **pricing** is §10.8/§10.14's known-open item and is not re-reported. The
**placement rule and its interaction with the quiet zone** are a different thing,
and §10.14's regeneration cannot resolve them because it is a measurement task
and this is a design decision.

---

### C-15 — §10.16 still credits the engraved out-of-band reminder that §8.2c says cannot exist

**Severity: Minor. Section: §10.16:1480 vs §8.2c:776-786 and §7:656.**

The fold corrected §7's row (*"Nothing reaches the steel for `mt qr`"*) and added
§8.2c's retraction box, which is emphatic: *"**`mt` CANNOT put that reminder on a
`mt qr` plate**… For **`mt qr`** the legend is `mt`-controlled and full, so the
warning reaches the operator on `stderr` before they cut and nothing reaches the
steel."*

§10.16:1478-1480 was not swept: *"The residual risk is handled by §8.2c's
**engraved out-of-band reminder** and recorded in §7."*

No behavioural divergence — §8.2c and §7 agree and are unambiguous — but this is
the **fourth** recurrence of the class §5's own retraction box names (*"A diff
falsifies text it never touches"*), and it sits in the ruling that accepts legacy
inputs, i.e. in the justification for the hazard the reminder was supposed to
cover.

---

### C-16 — one transaction now has two chunk sets carrying identical `(version, chunk_set_id)`

**Severity: Minor. Section: §3:163-170 / §10.13(c).**

R2's M-5, unfolded. §3 argues the header is *"strictly stronger than UR… so
symbols from two different transactions cannot be combined."* True. But once the
id is the extracted txid (§10.13(c)), the **same** transaction's `mt qr` set
(PSBT payload) and `mt string` set (raw-tx payload) carry identical `version` and
identical `chunk_set_id` and differ only in `count`. The header has no field
distinguishing payload type. A recoverer who owns both forms and feeds both to
one reassembler is relying on a count mismatch to be detected rather than on the
set id to separate them.

Minor because in practice the PSBT magic `psbt\xff` distinguishes them, and
because §10.13's re-derivation catches a bad assembly rather than yielding a
wrong transaction.

---

### C-17 — §4's plate table models raw binary, and its "three unmodelled inputs" note omits the ruled encoding

**Severity: Minor. Section: §4:424-450.**

§4's table is captioned *"the **RAW** column"* and its caveat enumerates exactly
three additive inputs the rows do not model: the 37-bit header per symbol,
§10.8's labels, and §10.14's font correction. The **encoding** is not among them,
and it is the largest of the four: §3:224-228 measures it at *"one extra plate on
RCW `wsh` tier 1 at five inputs (5 → 6)"*, confirmed by
`RESULTS_ecc_selection_2026-08-22.txt:42` (`6 pl, 5 qr, v22 ECC L` for bech32U
against §4's tabled `5 plates, 4 qr, v22, ECC L`).

No build divergence — the search runs at encode time and the table is
documentation — so Minor. But an implementer or reviewer treating §4's rows as
"a lower bound plus three small additions" is off by a plate on the artifact that
matters most, and §10.14's regeneration brief should name the encoding as a
fourth input or it will be regenerated in the wrong form.

---

### C-18 — §5 budgets the locktime row at 29 characters; the timestamp form is 30

**Severity: Minor. Section: §5:485 vs §8.4:888.**

§5's row is `LOCKED TO BLOCK <n> ~<year>` / `LOCKED UNTIL <t>` at **29**
characters. `LOCKED TO BLOCK 1383520 ~2034` is 29; `LOCKED UNTIL
2027-03-14T00:00Z` — the format §8.4:888 uses — is **30**. §5's budget takes the
max of alternatives elsewhere in the same table, so 29 is one short.

Nothing downstream moves (§4's reservation is derived from the line count, and
136 sits inside a 300-character allowance), which is why it is Minor rather than
a defect. It is reported because §5's budget is a measured figure cited to
`RESULTS_legend_budget_2026-08-22.txt`, and a measured figure that is one short
of its own worked example is a marker that the legend regeneration §10.14
requires has not yet taken the height/timestamp split into account.

Related and unstated: `<t>`'s engraved format. §5 writes `<t>`, §8.4:903 writes
`<time>`, and only the `stderr` example fixes it to ISO-8601 with a `Z`. A minute
of precision is engraved forever; `LOCKED UNTIL 2027-03-14` and
`LOCKED UNTIL 2027-03-14T00:00Z` are both consistent with the spec.

---

### C-19 — §3's description of the shared header omits the `chunked` flag bit

**Severity: Nit. Section: §3:164-166 and §3a:253-255.**

Both places enumerate the header as *"`version`, a 20-bit `chunk_set_id`, `count`
and `index`"*. The real layout is `[version:4][chunked:1][chunk_set_id:20]
[count-1:6][index:6]` = 37 bits (`chunk.rs:3-6, 51-56`), and `read` refuses a
chunk whose `chunked` bit is clear (`ChunkHeaderChunkedFlagMissing`,
`chunk.rs:74-76`). The spec's own 37-bit figure only adds up with the flag
included, so an implementer who counts the listed fields (4+20+6+6 = 36) and
trusts the total will find the discrepancy immediately. Nit, not Minor, for
exactly that reason — but `mt1` must also decide whether the flag is always 1
(the only value `md-codec` accepts) or whether a single-chunk `mt1` artifact is
unchunked, and §10.13 does not say.

---

## The `mt qr` walk

Operator runs `mt qr <finalized.psbt>` (file or stdin, §10.10).

1. **Parse and refuse** — §8.1 (finalized, both vocabularies), §8.2b (inputs ≥
   outputs, `AbsurdFeeRate`, duplicate outpoints, non-empty `vin`), §8.2c (missing
   input values, legacy warning), §8.2d (`non_witness_utxo` bound by txid), §8.4
   (locktime fields on stderr), §8.5 (`gettxout`), §8.6 (satisfaction binds
   outputs, both spending structures, taproot control-block shape), §8.9
   (secrets). **This is buildable and is the strongest part of the spec** —
   stronger than at R2, because §8.2d and §8.6's structural-recognizer paragraph
   are new and both name their mechanism precisely. The known-open §10.10 residue
   (how a missing input value is supplied, how the node is addressed) still
   applies.
2. **Choose a configuration** — §4. **Breaks at C-8** (module size: input or
   search dimension), and depends on **C-13** (does the 6-line reservation shrink
   when `FROM`/`TO` are blank) and **C-4** (which QR mode the capacity model
   assumes, which now turns on an unowned case transform rather than a library
   default).
3. **Fragment** — **breaks at C-3.** Three normative sentences give three chunk
   counts, and the one the measurements use produces 96 chunks against a header
   that caps at 64, while §3b and §8.7b both state the cap does not apply.
4. **Serialize each chunk to bytes** — **breaks at C-5** (no framing convention;
   the probe silently picked byte-alignment) and **C-6** (no `version` value, and
   the decoder hard-refuses a wrong one).
5. **bech32-encode** — mechanical, *except* for the case (**C-4**), which decides
   both admissibility and QR mode. Also **C-1**: three sections still say encode
   base45 here.
6. **Frame for the transport** — **breaks at C-2.** No record syntax, no class
   name, no prefix; `MaxRecords = 24`, `MaxRecordLen = 512`, the public
   allow-list and EPD §6.3's card-set decode are all unnamed in the spec and all
   refuse every obvious framing; §4's four parameters and §5's legend have no
   channel. **Breaks again at C-7** if the implementer uses §8.7c's worked figure
   to decide what fits.
7. **Engrave** — §10.17, known open, correctly scoped as firmware work.

**Verdict on the walk:** the refusal engine could be built today and is close to
excellent. Everything from "how many bytes in a chunk" to "what file `mt qr`
writes" still cannot be built, and the fold moved the blockage rather than
removing it: base45's transport collision was replaced by an uppercase transport
collision, and the missing record syntax is now measurably harder than §10.9
describes, because the public section admits only md1/mk1 card sets.

## The `mt string` walk

Operator runs `mt string <finalized.psbt>`.

1. **Parse and refuse** — same §8 set, plus §8.7b's 64-chunk refusal, now citing
   the corrected 89. Buildable.
2. **Extract the raw transaction** — §10.10 states it plainly and gives the reason
   (§8 needs PSBT vocabulary; the payload wants the smaller form). Clean.
3. **Fragment at 40 payload bytes** — specified, cited, corrected, traceable to
   `chunk.rs:224,253-254`. Still the strongest passage in the spec.
4. **Header** — **inherits C-6** (no `version` value) and **C-19** (the `chunked`
   flag). **C-5's padding limb does not bite here**: codex32 is a 5-bit domain and
   `wrap_payload` takes an explicit bit count. **C-4 does not bite here** either —
   the string form has an HRP and a checksum and follows the constellation's
   lowercase convention.
5. **Content id** — **closed** by §10.13(c). Extracted transaction, top 20 bits,
   display form. An implementer can write this today.
6. **BCH + codex32 with an `mt1` HRP and NUMS constant** — ruled in §10.13(a)(b);
   the machinery is real and the values are scoped implementation work.
7. **Print** — **breaks at C-11.** The delimiter, grouping and casing of a
   5-to-63-chunk set is the whole artifact of this verb, is unstated, and the
   sibling CLI's default (`--group-size 5`, space) is the display form EPD
   documents as hazardous to engrave — in the same words §3 quotes to justify
   dropping base45.
8. **Warn on stderr** — §3b's bearer warning, well argued, well placed, and §7
   records the asymmetry as an accepted risk rather than claiming a mitigation.

**Verdict on the walk:** `mt string` is one Important and one constant from
buildable. Fix C-11, assign the version value (C-6), and a competent implementer
builds exactly one thing.

## The recoverer walk

2040. Someone opens a drawer and finds steel.

**What they hold is now a known-open question rather than an unknown one.** §9
states plainly that *"a plate cut by `mt` v0.1 **cannot be read back by `mt`
v0.1**"* and §10.21 records that no legend field names the format, the tool or
the encoding. R2's finding is closed as *disclosure*; the recoverer's situation is
unchanged, which is the operator's call to make.

**`mt qr` path.** Scan the symbols → decode to bytes. **C-1** decides whether they
try base45 or bech32; **C-4** decides whether the symbols are even in the mode
the encoder assumed; **C-5** decides how the 37-bit header is unpacked; **C-6**
decides whether the version check passes at all. Order by `index`, concatenate,
parse as PSBT, extract, broadcast. §10.13's content-id compare is now well-defined
(R2 C-4 closed), which removes the failure mode R2 called most likely to bite —
but only *after* the four decisions above land the same way at both ends.

**`mt string` path.** Read characters off the plate → type them → no verb accepts
them (§9, disclosed). The BCH correction that justifies the entire verb is
unreachable from the shipped CLI. If the plate was engraved from a grouped
stdout (**C-11**), the separators sit outside the checksum's coverage and a
scratch on one is absorbed silently rather than corrected — the exact failure
this verb exists to prevent.

**Inventory and completeness.** §10.8's mechanism is the right one and is stated
normatively, and it is the part of the recoverer walk that most nearly works. But
it depends on the thing that is least decided: under the chunk model the
measurements use, a symbol holds ~19 chunks and *"`n/m` for the chunk it holds"*
has no referent (**C-3**). And where the label goes relative to the 4-module quiet
zone is unstated (**C-14**), on the artifact for which there is no second reader.

**Where they end up.** With a complete set of `mt qr` plates, a correct decoder,
and an encoder that made the same four undecided choices, the money comes back —
and the content-id compare now works in their favour rather than against them.
With a partial set they can name what is missing, provided C-3 resolved toward
one chunk per symbol. Without `mt`'s own reader they hold a plate that is
provably intact and unreadable, which the spec now says out loud.
