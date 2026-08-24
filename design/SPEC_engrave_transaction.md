# SPEC — Engrave a Transaction (SeedHammer II program)

**Status:** DRAFT, pre-R0. Written 2026-08-24 from the brainstorm in this
session. **Not yet reviewed, and the operator journey walk has not been run** —
by ruling, that walk is the *review* of this document (§9), and §5's refusal
list is deliberately thin until it has happened.

**Goal 1 of the two set by the operator 2026-08-24** (`CONTINUITY_mt_2026-08-24.md`).
The device engraves seeds, descriptors and a BIP-39 password. It has no notion of
a transaction. This closes that gap from the firmware end; Goal 2 closes it from
the operator's.

---

## 0. Scope

**In:** a new device program that accepts an already-signed Bitcoin transaction,
parses it, shows the operator what it is, and engraves it — plus the container
and host-side plumbing that gets the transaction to the device.

**Out:** transaction construction, signing, broadcasting, script evaluation,
fee estimation from the device, and any change to the Sealed Payload
(`seal/`, region `0xE1000000`), which stays frozen.

**Risk set.** Fork-native firmware touching funds-adjacent material and changing
normative container behaviour. The R0 gate applies: **no code before 0C/0I**.

---

## 1. The pipeline, and who owns each stage

```
tx.hex ──▶ mt encode --record ──▶ tx: record ──▶ me sysw pack ──▶ container
 (signed)     (mnemonic-transaction)              (mnemonic-engrave)     │
                                                                         ├─▶ --region ─▶ picotool ─▶ 0x10D00000
                                                                         └─▶ NFC tag
                                                                                          │
                                                                                          ▼
                                                             SeedHammer: Engrave Transaction
                                                             parse ─▶ comprehend ─▶ confirm ─▶ cut
```

Four owners. **None of them learns another's job**, and that is the point:

| owner | owns | gains |
| --- | --- | --- |
| `mt` (private repo) | the transaction and the `mt1` codec | one output form, `--record`. Not a new competence |
| `me` (this repo) | the `sysw` container | `ClassTransaction`. The record body is **opaque** to it, exactly as `text:` bodies already are |
| fork `sysw/` (Go) | reading the container | a port of the above, provenance-pinned |
| fork `gui/` | comprehension and the plate | the program |

**`me` packs; `mt` does not** — operator ruling 2026-08-24. `sysw/wire.go:10-12`
is explicit that the device never packs, *"so the device cannot disagree with the
host about how to build a container it should never build."* The same argument
forbids a **second host-side packer**. `mt` emits a record; `me` puts records in
containers; they compose over a pipe.

**Rust-primary.** The container is normative. It lands in `me` with test vectors
**first**, and reaches the fork as a port. The fork may never lead.

---

## 2. The container

### 2.1 One record, not siblings

`ClassTransaction` is carried by **one framed record** under a new reserved
prefix `tx:`, holding:

- the transaction, **or** its `mt1` chunks — never both (§2.2)
- the **legend fields**: `TO` label, fee, and whatever else `mt encode` already
  computes for its stderr legend

One record rather than three, so the legend stays **bound to** the transaction it
describes rather than adjacent to it and separately corruptible. It also makes
EPD's `1..24` record-count cap moot — though note `sysw` itself does not enforce
that cap (it splits on `\n`, `sysw/open.go:74`); the cap is `seal`'s.

`tx:` inherits the reserved-prefix rule (`sysw/record.go:41-51`,
`gui/scan.go:56-80`): a `tx:` record whose body is not valid lowercase hex is
`ClassUnknown` and **refused before any sniffer sees it**. A malformed
transaction record can therefore never fall through to free text and become an
engraved plate.

**No new secrecy class** — operator ruling 2026-08-24. A transaction rides as an
ordinary non-secret class beside `ClassMDMK` and `ClassFreeText`. The operator
and the machine handle the payload privately.

### 2.2 Raw transaction XOR chunks

**Operator ruling 2026-08-24: the payload carries the raw transaction or its
`mt1` chunks, never both. The operator picks at encode/pack time.**

This supersedes an earlier decision in this same brainstorm that the device
should *"always decode the chunks and compare them to the raw transaction."*
**With one form present there is nothing to compare**, and a spec carrying a
requirement its own architecture made unreachable is how a check that does not
exist gets asserted — the exact defect the `mt` review found in `verify` (§1.1's
content-id re-derivation, which printed a claim with no code behind it).

What replaces it is sharper, because **the payload's form decides the plate's
form**:

| payload carries | device needs | device can cut |
| --- | --- | --- |
| **raw transaction** (hex, 2.0 chars/byte) | a transaction parser only — **no `mt1` codec at all** | QR plates + legend |
| **`mt1` chunks** (≈2.3 chars/byte) | `mt1` **decoder** + parser | text plates, **and** QR plates (decode → bytes → QR) |

Note the asymmetry, and preserve it: chunks-only is strictly more capable on the
device but costs ~15% more container. **Raw-only can never produce text**, which
would need an *encoder* nobody has ported. The device does not choose the plate's
representation; the operator already did, at pack time.

**When chunks are present, decoding them IS the check.** A chunk set that does
not reassemble into a parseable transaction is refused — not because a second
representation disagrees, but because the device cannot comprehend what it was
handed, and §3 forbids cutting what it cannot show.

### 2.3 The section cap

**`sysw.MaxSectionLen` rises from 8191 to 32,734**, derived rather than picked:

```
MaxSectionLen = (RegionLen − HeaderLen − TagLen) / 2
              = (65536 − 52 − 16) / 2
              = 32,734
```

**Why 8191 exists.** It is the **NFC scan buffer minus one**. `gui/scan.go:31`
allocates `make([]byte, 8*1024)` and flags overflow at `s.n == len(s.buf)` —
*exactly full*, not over — so 8192 is unreachable. EPD §6.2 ruled it
(`SPEC_encrypted_payload_delivery.md:569`): a spec-legal 8192-byte payload would
pass every bound, burn the full KDF, authenticate, and only then die in the
classifier. `sysw/wire.go:39-41` then **inherited it unchanged** — its own comment
says so — so the flash path is capped at one-eighth of the region it physically
has, for a reason belonging to NFC.

**Why the formula and not a round number.** It preserves the property the current
8191 has and that `boundBlob`'s 32-bit no-wrap argument rests on: **two maxed
sections plus header plus tag still fit the region.** Today that is
`52 + 8191 + 8191 + 16 = 16,450` — the figure `gui/gui.go` quotes. A round 32,768
would break it by 34 bytes.

**What it buys.** 32,734 characters ⇒ a **16,367-byte** raw transaction, **2× the
worst measured pathological spend**. Measured (`RESULTS_2026-08-22.txt`,
pathological wallet — 11 keys, 3 masters — `wsh`, tier 1 = 3-of-3 + hash, the most
expensive spend path):

| inputs/outputs | signed tx | as hex | chunks | bytes/chunk | as chars | fits 32,734? |
| --- | --- | --- | --- | --- | --- | --- |
| 1-in/1-out | 852 B | 1,704 | 22 | 39 | 2,001 | ✅ either form |
| 1-in/2-out | 893 B | 1,786 | 23 | 39 | 2,092 | ✅ either form |
| 2-in/2-out | 1,692 B | 3,384 | 43 | 40 | 3,955 | ✅ either form |
| 5-in/2-out | 4,080 B | 8,160 | 102 | 40 | 9,383 | ✅ either form (**was raw-only at 8191, by 31 characters**) |
| 10-in/2-out | 8,067 B | 16,134 | 202 | 40 | 18,583 | ✅ raw; ✅ chunks |

Every cell above is computed, not estimated: `chunks = ceil(bytes/40)`,
`bytes_per_chunk = ceil(bytes/chunks)` — **balanced, not filled**, which is why
the two smallest cases carry 39 — and `chars = ceil((bytes_per_chunk×8 + 55)/5)
+ 16` per chunk, plus one newline between chunks. The character formula
reproduces the three shipped vector lengths (79 / 85 / 87 at 32 / 36 / 37 bytes)
exactly.

Chunk counts are `ceil(bytes/40)` (`SPEC_mt_v0_1.md:1527`) and match the results
file. A full 40-byte chunk is **91 characters**, not the `~96` at
`SPEC_mt_v0_1.md:1562` — see F-242, and note `SPEC_mt_v0_1.md:1308` already says
91.

**Four things the raise touches, all of which the plan must carry:**

1. `boundBlob`'s comment names `8191` in its no-wrap argument and goes stale the
   moment this lands. Comments outlive their conditions.
2. **NFC stays at 8191.** `gui/scan.go`'s buffer genuinely bounds it. So a large
   transaction is **picotool-only**, and `me sysw pack` MUST state which
   transports its output fits rather than letting the operator discover it at the
   tag.
3. Rust-primary: `me`'s `sysw` first, with vectors.
4. **`seal` is untouched** and keeps its own 8191. It is frozen.

---

## 3. The device program

### 3.1 Placement

**`engraveTransaction`, INSERTED mid-enum** before `loadPayload` — not appended.
The house rule in `gui/gui.go` is explicit and compile-time-enforced
(`var _ [1]struct{} = [qaProgram - unlockPayload]struct{}{}`): unconditional
programs go in the middle so `bip85Derive` stays the bound `lastNav()` returns
and `unlockPayload` stays the conditional last. Appending would put an
unconditional program after a conditional one and break that bound.

Placed **beside the other engrave programs**, per `walletPolicy`'s own reasoning:
*"the carousel reads as a list of things to make, and a utility sitting between
two of them is a seam an operator has to step over."*

### 3.2 Two entry points, matching the two transports

- **the carousel** — payload already loaded by `syswLoadFlow`
- **`engraveObjectFlow`** gains a `txScan` case, exactly as `freeTextScan` and
  `passScan` did

Both pass through `syswSourceAccept`, which names the source (F3) before anything
is entered.

### 3.3 The confirm screen IS the program

**Operator ruling 2026-08-24: comprehend, then cut.** The device parses the
transaction with `btcd/wire/v2` (already an indirect dependency) and shows the
operator what it is. The organising rule:

> **The screen separates what the transaction PROVES from what the operator
> ASSERTED, and never renders them in one voice.**

| | source | examples |
| --- | --- | --- |
| **Derived** | the transaction bytes; the device stands behind it | txid, input count, each output's address and amount, locktime, `nSequence` state |
| **Asserted** | the payload's legend fields; the operator's own words | the `TO` label, the fee |

**This split is forced, not stylistic.** Fee is **not in a signed transaction** —
it needs input values, which is why `mt` takes `--input-value` or asks a node.
And F-235 already established that the legend's `TO` comes from `--to` /
`--to-label`, the operator's assertion, never from the transaction. A screen
rendering both in one voice would have the device vouching for numbers it cannot
see.

**Network.** A `scriptPubKey` carries no network, so the device cannot know it
either (F-235). The address row MUST say which parameters it rendered under
rather than printing an address that may not exist on the transaction's network.
F-235's own guidance applies: do **not** fix this by suppressing the address —
it is the single most useful row for a recoverer deciding whether to broadcast.

### 3.4 What the device does NOT do

- **No script evaluation.** There is no consensus engine. Signatures are
  recognised **by shape**, as in `mt`, and the device says so. It cannot detect a
  bad signature; this is an accepted, recorded hazard, inherited from `mt` and
  restated here so it is not rediscovered as a defect.
- **No transaction construction, no signing, no broadcast.**
- **No fee derivation.** See §3.3.

---

## 4. The plate

### 4.1 Composition

**Default: QR + legend.** Text plates only when the payload carries chunks
(§2.2). The device states the plate count and cut time **before** the operator
commits — ~21 minutes per plate (F-225), and the difference between the two forms
is not marginal: the pathological 10-in/2-out spend is **~9–11 QR plates** or
**202 text plates**.

### 4.2 What the QR carries

**The raw transaction bytes** — F-234, operator directive 2026-08-22, and not
re-litigable. A plate carries two representations with two audiences:

| | engraved TEXT | engraved QR |
| --- | --- | --- |
| format | `mt1` codex32 | **the standard form** — the transaction |
| audience | a human with eyes and a keyboard | anyone with a camera and standard tooling |
| error correction | BCH — transcription slips | Reed-Solomon — physical damage |
| needs constellation knowledge? | yes | **no** |

**The ENCODING of those bytes is a PARAMETER, not a ruling** — operator decision
2026-08-24, recorded as F-243. The candidates are raw octets, base45 (RFC 9285)
and bech32-uppercase. The argument that previously settled this **does not bind
here**:

- `SPEC_mt_v0_1.md:1362` rejected base45 because **its alphabet contains SPACE**
  (index 36) and EPD §6.4 forbids interior whitespace in a `sysw` record. In that
  architecture *"the record stores lowercase; `mt` uppercases only when encoding
  the QR symbol"* — **the record's string WAS the QR's string**, so EPD §6.4
  propagated into the QR.
- **In this architecture they are decoupled.** The record carries hex (or
  chunks); the QR is generated **on-device** from parsed bytes and never passes
  through a record. EPD §6.4 does not reach it.

**And the case against raw octets is currently unproven.** F-243: F-234's claim
that *"many QR scanners assume UTF-8 and mangle octets >= 0x80"* is **not
measured anywhere** — `grep -ri "scanner|utf-8|mangl" design/measurements/` finds
no test, no result, no apparatus. What *is* measured is that our encoder emits an
**ECI header** for high bytes, ~0.5% (`qrplate.rs:28-29`). Since raw octets are
the only candidate that actually delivers F-234's stated promise, an untested
assertion is currently the reason that promise is unmet. **The test plate settles
it (§6 S0).**

### 4.3 The configuration search

**The DEVICE runs it, not the host.** §4 of `SPEC_mt_qr_DEFERRED.md` was written
for a host verb `mt qr` that this design does not have. Its objective is stated in
**plates and minutes** — machine facts — and only the device holds
`EngraverParams`. The device already does plate layout (`validateMdmk`,
`backup.Paragraph`, `toPlate`); this extends that rather than adding a
competence.

```
search space:  module size × QR version (1..40) × ECC (L,M,Q,H)
               × rectangular tiling (across × rows)
objective:     1. minimise plates      ← a plate holds the QR(s) AND the legend
               2. maximise ECC
               3. minimise symbol count
               4. TIE-BREAK: maximise MODULE SIZE
               5. then minimise QR version
plate:         85 × 85 mm, outer margin 3 mm ⇒ 79 mm usable
quiet zone:    4 modules per side, per symbol
legend:        6 lines reserved on plate 1 (25.5 mm at 4.25 mm pitch),
               1 line on every later plate for "PLATE n OF m"
```

*Never trade a plate for redundancy; never leave redundancy unbought.*

**Both R0 corrections MUST be carried into the port, because they are easy to
lose:**

1. **Tiling is `across × rows`, not `k × k`.** The previous draft's prose said
   square while its own search returned 2-, 3- and 6-symbol configurations, none
   of which is a perfect square.
2. **The objective must be a TOTAL ORDER, breaking toward the LARGEST module.**
   The original comparison key omitted module size and used strict `<` against a
   loop ascending from 0.30 mm, so ties resolved to the **smallest and least
   legible** symbol. **4 configurations tie** at the 0.60 mm floor for a 162 B
   payload; **41 tie** once the floor lifts.

### 4.4 The plate table must be regenerated

`SPEC_mt_qr_DEFERRED.md`'s measured table **does not apply to this design** and
must be regenerated. Three inputs, one job:

1. it measures **PSBTs**; this design carries **signed transactions**, which
   F-234 puts at 53–91% of PSBT size;
2. it corrects for a **49-bit** chunk header — a superseded draft layout; the
   ruled header is **55 bits** (F-242, third site);
3. §10.14's **font-metric correction** to the legend reservation, already owed.

**Until it is regenerated, no plate count in this spec is load-bearing** beyond
the order-of-magnitude comparison in §4.1.

### 4.5 Module size

**0.60 mm (two engraved strokes) is the default and what the device suggests** —
not a floor. 0.30 mm is the theoretical minimum and is **optically unvalidated**:
whether a camera reads 0.3 mm engraved modules off brushed steel is a *hardware*
question, and the font work's two-stroke minimum was established for **glyphs**,
where a QR module (a solid square) genuinely differs. **Design against the
0.60 mm column until the test plate exists.**

---

## 5. Refusals — DELIBERATELY INCOMPLETE

**This section is a stub, and saying so is the point.** By operator ruling
2026-08-24 the Goal 2 **journey walk is the review of this spec** (§9), and that
method exists precisely because *"most refusal lists get written by imagining a
hostile input; a journey walk generates them from a plausible user, which is
where real losses come from."*

Filling this in from imagination before the walk would produce a list the walk
then has to argue with. What is already forced by §1–§4:

| # | refusal | why |
| --- | --- | --- |
| R1 | a `tx:` record whose body is not lowercase hex | reserved-prefix rule; else it falls through to free text and becomes a plate |
| R2 | a chunk set that does not reassemble into a parseable transaction | §2.2 — the device cannot comprehend it, and §3 forbids cutting what it cannot show |
| R3 | a payload carrying **both** raw and chunks | §2.2 is XOR; both present means one was built by something that does not know this format |
| R4 | a transaction the parser rejects | §3.3 |
| R5 | a payload whose section exceeds `MaxSectionLen` | §2.3, and it must name the transport, since NFC's bound is lower |

**Every refusal gets a test, and every refusal test must go RED when its check is
removed.** `mt` has this machinery already — `refusals.toml`,
`check-refusal-coverage.sh` (a bijection over every suite),
`mutate-refusals.sh` (30/30 go red). The fork side needs its equivalent; this
is not optional, and it is the gate that caught the most in the `mt` cycle.

**A guard downstream of the parser has already lost.** `mt`'s §8.2f was bypassed
by the very invocation it existed to refuse, because clap rejected the positional
argument first — **and clap's error echoed the bearer transaction**. Every
refusal here must be checked against *where in the pipeline it actually runs*.

**And every guard must be tested against its NEAREST LEGITIMATE INPUT.** Five
separate fixes in the `mt` cycle broke on the near miss — the input that merely
resembles the one the finding named. A finding hands you a hostile X and never
the legitimate near-X. **Before committing any fold that adds or widens a guard:
run the hostile input (must be caught) AND the nearest legitimate one (must
pass), and keep both as tests.**

---

## 6. Sequencing

| | where | what |
| --- | --- | --- |
| **S0** | this repo | **Cut the test plate.** QR blocks at 0.3 / 0.45 / 0.6 / 0.9 mm, plus one raw-octet and one base45 symbol, scanned off brushed steel. ~2 s per cut using the single-character technique. Resolves module size (§4.5) **and** the encoding parameter (§4.2, F-243) |
| **P1** | `me` (Rust) | `ClassTransaction`, the framed record, `MaxSectionLen` → 32,734, **with test vectors** |
| **P2** | `mt` (Rust) | `mt encode --record`; states which transports its output fits |
| **P3** | fork (Go) | Port P1, provenance-pinned |
| **P4** | fork | The program: parse, comprehend, confirm (§3) |
| **P5** | fork | The plate: configuration search, legend, plate count (§4) |
| **P6** | both | Journeys and refusal coverage (§5) |

**S0 comes first, and that is the closure rule applied rather than quoted.** Two
of this design's gates are hypotheses, and one of them is **two seconds of
machine time**. Cutting it before the spec freezes removes two unknowns from the
document instead of carrying them through every review round. *A gate that has
never executed is a hypothesis, not a gate.*

---

## 7. What must be true to close

- **0C / 0I** under the R0 loop, over the lenses enumerated up front — not until
  a round comes back clean. *Closure is lens-closure.*
- **The mode-segmentation gate is green.** Any QR sizing MUST assert measured
  v40 capacity against the published limits — **numeric 7089 / alnum 4296 / byte
  2953 at L**. A QR encoder performs optimal mode segmentation and will silently
  re-encode part of a payload in a denser mode: an all-`0x41` payload measured
  *alphanumeric* capacity while claiming byte, a high-byte payload paid an ECI
  header, and a mixed payload read **6.6% low**. Every one produced a plausible
  number, and only this gate caught them.
- **The test plate is cut and read** (S0).
- **`check-provenance.sh` green** across both repos. It is **not in CI** — it
  needs a second repository — so it will not catch itself.
- **Refusal coverage is a bijection, and every refusal test goes red without its
  check** (§5).
- **The plate table is regenerated** (§4.4).

---

## 8. Ruled, and not to be re-litigated

| ruling | date | source |
| --- | --- | --- |
| Every QR carries the STANDARD form, never a codex32 string | 2026-08-22 | operator, F-234 |
| The device comprehends before it cuts | 2026-08-24 | operator, this brainstorm |
| Plate default is QR + legend; text is optional | 2026-08-24 | operator |
| Payload carries raw tx **XOR** chunks; operator picks at pack time | 2026-08-24 | operator |
| `mt` emits the record, `me` packs the container | 2026-08-24 | operator |
| No new secrecy class for transactions | 2026-08-24 | operator |
| `MaxSectionLen` rises for flash; NFC keeps 8191 | 2026-08-24 | operator |
| The QR's byte ENCODING stays a parameter until the test plate | 2026-08-24 | operator, F-243 |
| The journey walk is the review of this spec | 2026-08-24 | operator |

---

## 9. Open, and owned

| # | open question | owner |
| --- | --- | --- |
| O1 | **The QR's byte encoding** — raw octets, base45, or bech32-uppercase | S0's test plate (F-243) |
| O2 | **Module size below 0.60 mm** — optically unvalidated | S0's test plate (F-234) |
| O3 | **The refusal list** (§5) | the Goal 2 journey walk |
| O4 | **The network the address row renders under** | F-235's unresolved half; §3.3 |
| O5 | **`validateMdmk`'s four callers** engrave an `md1`/`mk1` codex32 string as QR content — a live F-234 violation, found 2026-08-24 | **NOT this spec.** New scope on F-234; needs its own ruling, because for an `md1`/`mk1` card the "standard form" is not obvious the way transaction bytes are |
| O6 | Whether a **multi-symbol** plate can be recovered without `mt`'s reader | carried from `SPEC_mt_qr_DEFERRED.md:169`, which states plainly that F-234's promise holds only for single-symbol artifacts |

---

## 10. Provenance of the numbers in this document

Every measured figure here was resolved against its source during the brainstorm
rather than carried from prose, because **three separate facts turned out to be
stale in the process** — the 64-chunk cap (retracted; `mt1` uses 15 bits,
32,768 chunks), F-234's chunk counts (~13% low), and the `~96`-character chunk
(actually 91). See F-241, F-242, F-243, and commits `d6c735a` / `0c0d11e`.

- `sysw` constants — `third_party/seedhammer/sysw/wire.go`, read at `a91df84`
- scan buffer — `gui/scan.go:31`
- pathological wallet sizes — `design/measurements/RESULTS_2026-08-22.txt`
- RCW sizes — `design/measurements/RESULTS_rcw_2026-08-22.txt`
- QR density by representation — `design/measurements/RESULTS_qr_modes_2026-08-22.txt`
- chunk rule and header — `design/SPEC_mt_v0_1.md` §3
- QR search and plate geometry — `design/SPEC_mt_qr_DEFERRED.md` §4

> **A caution attached to two of those files.** `RESULTS_2026-08-22.txt` and
> `RESULTS_rcw_2026-08-22.txt` mark rows `fits` / `OVER` against `ch(n) <= 64`
> (`signed.rs:209`, `rcw.rs:189`) — **the retracted 64-chunk cap**. Under `mt1`'s
> real 32,768-chunk ceiling **everything in both files fits**. The byte counts and
> chunk counts are sound; the verdict column is not. Do not cite it.
