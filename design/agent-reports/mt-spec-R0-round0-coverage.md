# R0 gate, lens 4 of 4 — SPEC COVERAGE AND IMPLEMENTABILITY

Artifact: `design/SPEC_mt_v0_1.md` @ `099a516` (534 lines, read in full).
Reviewer: independent agent, coverage lens only.
Question answered: *what must an implementer decide that this spec does not decide
for them, and where would two competent implementers, both faithfully following
it, produce different artifacts?*

Everything below was checked against source, not inferred. The fork was read at
`/scratch/code/shibboleth/seedhammer`, `me` at `crates/me-cli`, the measurement
probe at `design/measurements/mt-size-probe/`. I did **not** recompute plate
counts, byte counts or character budgets — where a number appears below it is
quoted from the spec or from source.

## Verdict

**3 Critical / 11 Important / 5 Minor / 2 Nit**

---

### C-1 — Critical — §0, §4 (and absent everywhere else): the spec never says how an engraving is produced, and §4 selects from a space the machine cannot reach

**What I searched.** `grep -n -i` over the whole file for: `manifest`, `firmware`,
`machine`, `SeedHammer`, `toolpath`, `NFC`, `NDEF`, `seal`, `sysw`, `me `, `file`,
`JSON`. Results: `firmware` appears once (line 526, about back-side engraving),
`SeedHammer` once (line 93, about Structured Append), `NFC` once (line 444, about
`me` refusing `ms1`), `machine` never in the sense of the engraver. `toolpath`,
`NDEF`, `sysw` never appear at all.

**The gap.** §0 says `mt` "**engraves** the signed result — deciding how many QR
symbols, at what error-correction level, across how many plates, and what is
engraved beside them". Deciding is where the spec stops. There is no statement of
what `mt engrave` *emits*, no interface to anything that drives the SeedHammer II,
and no named consumer of the decision.

**And the decision space does not exist on the machine.** Read from the fork:

- The plate model carries **exactly one** QR per paragraph: `backup.Paragraph`
  has a single `QR *qr.Code` and `QRScale int` (`backup/backup.go:67-68`), and
  the engraver draws that one code (`backup/backup.go:419-423`). There is no
  `k*k`, no tiling, no second symbol on a plate anywhere in the package.
- The only arbitrary-payload path — free text — is **fixed at one point of §4's
  search space**: `freeTextQRScale = 2` (`backup/fit.go:19`, i.e. 0.6 mm modules
  against the 0.3 mm stroke) and `qr.Encode(text, qr.L)` (`backup/fit.go:154`),
  so ECC **L**, one symbol, one scale. §4's UR column selects ECC H, M, M, L, M,
  L across its seven rows and lists plates carrying `3 qr` and `2 qr`.
- The device's record classes are enumerated at `sysw/record.go:19-27`:
  `ClassUnknown, ClassMnemonic, ClassCodex32Secret, ClassPassphrase,
  ClassFreeText, ClassDescriptor, ClassMDMK, ClassAddress`. **There is no
  transaction, UR, or raw-symbol class**, and `classifyConstellation`
  (`sysw/classify.go:33-49`) has no branch that could produce one.
- Module size is quantised: `engrave.QR(strokeWidth int, scale int, qr *qr.Code)`
  (`engrave/engrave.go:277`) takes an **integer** scale against
  `strokeWidth = 0.3 * mm` (`cmd/controller/platform_sh2.go:213`). Only integer
  multiples of 0.30 mm are engravable.

**The divergence.** Implementer A treats `mt engrave` as terminating in a printed
report plus a preview image (the `me bundle --preview` shape) and hands the
operator nothing the machine accepts. Implementer B invents a payload record
class and a fork patch. Implementer C emits an SVG. None of them can be wrong
against the spec, and only one of the three ends with metal cut. A fourth,
following §4 literally, selects `v16 ECC H at 0.60 mm` for the smallest artifact
and discovers at the machine that the only path takes ECC L.

**Why it matters.** This is the project's own measured failure mode — *"plans list
components and omit the call that joins them; six green stages shipped an inert
feature."* A user cannot do the thing end to end with only what this spec
specifies. Every other finding in this report is downstream of this one: plate
counts, ECC levels and tiling are all being optimised for a consumer that does
not exist and whose constraints are not stated.

*Non-authoritative sketch.* Either (a) name the artifact — "`mt engrave` writes
`plate-<n>.svg` plus a manifest, and the operator engraves via <path X>" — and
add §4 a constraint line saying which of (scale, ECC, symbols-per-plate) the
existing path fixes; or (b) declare the firmware work (a transaction/UR record
class, multi-symbol plates, selectable scale and ECC) as an in-scope dependency
with its own gate, and say §4's search space is *aspirational until that lands*.
Option (b) makes §10.7's back-side question one item in a list rather than the
lone firmware caveat.

---

### C-2 — Critical — §4: the stated search space (`k*k tiling`) contradicts the search that produced §4's own normative table

§4 line 160:

>     search space:  module size x QR version (1..40) x ECC (L,M,Q,H) x k*k tiling

and §2 line 79 promises this is decided *"deterministically, so two encoders
agree"*.

**The search that produced the table does not tile `k*k`.** From
`design/measurements/mt-size-probe/src/bin/select.rs:91-99`:

```rust
let across = (USABLE_MM / fp).floor() as usize;
let rows_first = (h_first / fp).floor() as usize;
let rows_rest  = (h_rest  / fp).floor() as usize;
...
let first_cap = across * rows_first;   // may be 0: QR too tall to share with the legend
let per_plate = across * rows_rest;
```

That is a free `across × rows` rectangle with **different row budgets on plate 1
and on later plates** — three distinct capacities (`across`, `rows_first`,
`rows_rest`), not one `k`.

**The table proves the contradiction on its own.** §4 line 186 lists RCW `wsh`
tier 1 raw as `2 plates, 6 qr`, and line 187 lists 9-of-11 as `3 plates, 2 qr`.
Six is not a square; two is not a square. No value of `k` yields either.

**The divergence.** Implementer A implements `k*k` as written and, for any
artifact needing 6 symbols, must round up to `k=3` (9 slots) or split across more
plates — producing different plate counts and therefore, under the objective
"minimise plates then maximise ECC", **a different ECC level for the same
transaction**. Implementer B reproduces the probe and matches the table.
Two encoders do not agree, which is precisely what §2 line 79 promises they will.

*Non-authoritative sketch.* State the tiling rule in the spec in the currency the
probe uses (`across = floor(usable / footprint)`, `rows` computed separately for
plate 1 and later plates), and delete `k*k`. The probe is the normative
description; §4's prose is the stale one.

---

### C-3 — Critical — absent: there is no CLI surface, and §8's refusals do not say which verb they bind

**What I searched.** `grep -n -i` for `CLI`, `command`, `flag`, `exit`, `stdin`,
`JSON`, `file`. `CLI` appears twice and both are about *which repo* the tool
lives in (lines 39, 225). `stdin`, `exit`, and any command name (`mt engrave`,
`mt produce`, `mt present`) appear **zero** times in 534 lines.

**What is undecided:**

1. **Command names and shapes.** The three verbs are named as concepts (§1a) and
   never as invocations.
2. **Inputs.** For `engrave`: a hex transaction? a finalized PSBT? a file or
   stdin? For `produce`: what carries the inputs, outputs, previous transactions
   and `gettxoutproof` blobs of §6a's four tiers? §9 says `mt` builds "from
   inputs and outputs it is *given*" — given how, in what format, is never said.
3. **Outputs.** Nothing, anywhere. See C-1.
4. **Exit codes.** Absent, for a tool whose §8 is seven distinct refusals.
5. **The two named flags attach to nothing.** `--i-certify-amounts` (lines 317,
   468) and `--allow-immediate` (line 474) are the only two flags in the
   document and neither has a command.

**And §8 is verb-ambiguous in a way that changes whether the tool is usable.**
§8's preamble is *"All are machine-checkable before a single plate is cut"* —
engrave-time. But §8.3a (*"Input amounts asserted without proof → refuse unless
`--i-certify-amounts`"*) and §8.3b (*"`gettxout` returns `null` for any input →
refuse, **no override**"*) are §6a/§6b rules, and §6a is explicitly about
**producing**. §8.2 hedges the other way: *"Script-invalid → refuse, **when
prevouts are supplied**"*, which presupposes they may not be.

**The divergence.** Implementer A: `mt engrave signed.hex` works standalone —
§8.1/§8.4/§8.5/§8.6/§8.7 apply, §8.2 is skipped because no prevouts were given,
and §8.3a/§8.3b are produce-only rules. Implementer B: `mt engrave` demands
prevout evidence for every input before it will cut, because §8 says these are
checks made "before a single plate is cut". **B's tool cannot engrave a
transaction someone else signed unless the operator also supplies every previous
transaction; A's tool will happily engrave a partially-signed multisig** — §8.1
only requires *"a witness or scriptSig"*, and a 1-of-3-signatures-collected
`wsh` input carries a non-empty witness. Only §8.2's consensus check catches
that, and A skips it.

*Non-authoritative sketch.* A short §8.0 table: rule × verb (`produce` /
`present` / `engrave`), with §8.2 promoted from "when prevouts are supplied" to
either mandatory-at-engrave or explicitly-optional-with-a-named-consequence.
Then a §12 naming each command, its inputs, its outputs and its exit codes.

---

### I-1 — Important — §5: `FROM WALLET <8 hex>` is a mandatory legend field with no specified input and no absent-case rule (this is also §10.4's answer)

§5's table lists five fields and the whole of §4's reservation (`6 lines … 25.5
mm`) is sized to them. One of the five is `FROM WALLET <8 hex>`, described as
*"the 4-byte policy-id stub … reusing `mk1`'s existing derivation"*.

**The derivation takes an input the spec never lists.** In `mnemonic-key`:
`pub fn derive_stub_from_md1_card(card: &[&str]) -> Result<[u8; 4]>`
(`crates/mk-cli/src/cmd/mod.rs:126`). It takes an **md1 card** — a slice of
strings, i.e. potentially several chunks. Nothing in §0, §1a, §6a or §8 mentions
an md1 card as an input to `mt`; §5 introduces the stub as a legend field and
moves on.

**The divergence.**
- A makes the card **required** for `engrave`; an operator holding only a signed
  transaction cannot cut a plate at all.
- B makes it **optional** and omits the line when absent — dropping the legend to
  4 fields, which changes §4's reserved height and therefore the selected ECC and
  possibly the plate count.
- C makes it optional and engraves a placeholder (`FROM WALLET UNKNOWN`), keeping
  6 lines but at a different character width.

All three are faithful readings. Two of them change the *plate count* for the
same transaction.

*Non-authoritative sketch.* Say where the 8 hex digits come from (an md1 card
supplied alongside the transaction), say whether it is required, and if optional
say exactly what occupies the line and whether §4 still reserves it.

---

### I-2 — Important — §5: the legend field templates are not specified — truncation, amount format, wrapping and overflow

§5 gives five templates and a character count each. It does not give the
rendering rules for any of the variable parts.

- **`TO <truncated addr>  <amount>` | 34.** Truncated *how*? The measurement file
  shows `TO bc1p8rrz...s6n0vcl  0.00399 BTC` — 8 leading + `...` + 7 trailing —
  but that string lives in
  `design/measurements/RESULTS_legend_budget_2026-08-22.txt`, **not in the
  spec**. An implementer reading the spec alone chooses first-N, first-and-last,
  or a checksum-preserving split. All fit 34 characters; all engrave differently.
- **Amount format.** BTC or sats? How many decimals? `0.00399 BTC` in the
  measurement is 5 decimals, which is not the 8 a BTC amount can carry — so
  amounts are being *rounded on a bearer instrument* by an unstated rule.
  Implementer A engraves `0.00399 BTC`, B engraves `0.00399000 BTC` (3 chars
  wider, over budget), C engraves `399000 SATS`.
- **Six lines, five fields — which one wraps?** §5 says *"Five fields, 136
  characters, 6 lines"*. `BEARER - ANYONE HOLDING THIS CAN SPEND IT` is 41
  characters against the ~35 chars/line the budget assumes
  (`crates/me-cli/src/lib.rs:45-48`), so it must wrap — but the spec never says
  it wraps, never says where the break goes, and never says whether the strings
  are fixed or templated.
- **Overflow.** My brief asks what happens when a field overflows its line. The
  spec has no answer: §8.5 refuses when *"over the plate budget"*, which is about
  plate count, not about a legend line that grew (e.g. a block height crossing
  10,000,000, or a taproot address whose truncation rule differs).

**Why it matters.** These are engraved permanently on a bearer instrument, and
the character counts in §5 are what §4's whole objective is optimised against. A
rendering rule that costs 3 characters can cost a line, and a line costs 4.25 mm
of the QR's budget.

---

### I-3 — Important — §5, §6: one `TO` line, N outputs, no selection rule

§5's legend has exactly one `TO <truncated addr>  <amount>` line, justified as
*"so a human sees where the money goes without a scanner"*. §6 says *"'Goes to'
is already in the transaction. Outputs carry scriptPubKeys; any standard decoder
yields addresses and amounts"* — plural, correctly.

A transaction has ≥1 outputs and typically ≥2 (payment plus change). §9 excludes
coin selection but **not** change; §0 says `mt` builds from "inputs and outputs it
is given", so multi-output is plainly in scope.

**The divergence.** A engraves the **first** output. B engraves the **largest**.
C engraves the largest **non-change** output, which requires knowing which is
change — a fact `mt` does not have without the wallet. D adds a line per output,
blowing §5's 136-character budget and therefore §4's reservation and therefore
the plate count.

**Why it matters.** The legend is the only thing a human reads before scanning.
Under A, a transaction that pays 0.001 BTC to a counterparty and returns 4.9 BTC
to change engraves `TO <counterparty> 0.001 BTC` and says nothing about the 4.9 —
or the reverse, depending on output order. This is the field that exists so an
operator *"sees what they commit to"* (§7's pinned-destination row).

---

### I-4 — Important — §8.4, §5: the "broadcastable today" refusal has no chain-tip source, no time-locked case, and misses the all-sequences-final case that makes nLockTime unenforced

§1 decision 6: *"**A future locktime is required by default**, with an explicit
flag to override."* §8.4: *"**Broadcastable today** → refuse **by default**."*
§5's legend: `SPENDABLE AFTER BLOCK <n>`.

Three cases the rule does not cover:

1. **No tip source offline.** Deciding "broadcastable today" needs the current
   height. §6b's `gettxout` path assumes a node; §6c's whole argument is that
   `mt` must stay usable **offline** (*"the middle tier matters because it keeps
   `mt` usable offline, which is the constellation's whole posture"*). Offline
   there is no tip. A refuses to run offline; B adds a `--height` flag; C uses
   the highest block height it saw in a `gettxoutproof`; D skips the check
   offline. Four different tools.
2. **Time-based nLockTime.** A locktime ≥ 500,000,000 is a Unix timestamp, not a
   height. §5's field is `SPENDABLE AFTER BLOCK <n>` with no representation for
   it, and §8.4 has no rule. A refuses time-locked transactions; B engraves a
   ten-digit timestamp into a field labelled `BLOCK`.
3. **The case that defeats the ruling.** nLockTime is only enforced when at least
   one input has `nSequence != 0xFFFFFFFF`. A transaction with a future
   nLockTime and every sequence final is **final now** — broadcastable today —
   and passes §8.4's check as written, which looks only at the locktime. The
   plate then carries `SPENDABLE AFTER BLOCK <n>` as an engraved falsehood on a
   bearer instrument, and §1's default guarantee is not delivered.

Case 3 is arguably Critical under this gate's own definition ("an unmet
guarantee"); I have graded it Important because it is a missing case in a
refusal rule rather than an unimplementable spec, and because lens 2 (funds
safety) may hold the same finding. It should not close without a decision.

---

### I-5 — Important — §4: the legend-only plate is in the measurement and not in the spec

§4's plate model, line 166-167:

>     legend:        6 lines reserved on plate 1 (25.5 mm at a 4.25 mm pitch),
>                    1 line on every later plate for "PLATE n OF m"

The probe implements a third case the spec does not mention
(`select.rs:97, 105-111`):

```rust
let first_cap = across * rows_first;   // may be 0: QR too tall to share with the legend
...
else if first_cap == 0 {
    // legend cannot share a plate with this symbol
    // size: it needs a plate of its own
    1 + symbols.div_ceil(per_plate)
}
```

**This case is reachable and it is in the table.** With `LEGEND_LINES_FIRST *
LINE_PITCH_MM = 25.5`, plate 1's symbol budget is 79 − 25.5 = 53.5 mm. Any symbol
taller than that gets `rows_first == 0`, so plate 1 carries **the legend and no
QR at all** — a text-only plate — while the symbols start on plate 2. §4's own
table (line 184) reports 3-of-5 signed / UR as `2 plates, v21, ECC M`; that is
one legend plate plus one symbol plate, not two symbol plates.

**The divergence.** A implements §4's prose, finds the legend and the symbol
mutually unsatisfiable on plate 1, and backs off to a lower ECC or a finer split
until they co-fit — a *different config and possibly a different plate count*.
B reproduces the probe and cuts a text-only plate 1. And `PLATE n OF m` now
counts a plate that carries no fragment, which changes what a recoverer counts.

---

### I-6 — Important — §4: the plate geometry is not the machine's plate geometry

§4 models a plate as: 85×85 mm, `outerMargin 3` ⇒ 79 mm usable, `quiet zone: 4
modules per side`, `4.25 mm` line pitch. Four features of the real plate are
absent from that model:

| §4 says | the fork says |
| --- | --- |
| 79 mm usable, symbols tile it freely | **screw-hole bands**: `innerMargin = 10` (`backup/backup.go:74`), which narrows the first and last text rows by `holeChars` per side (`backup/wrap.go:141-149, 231`). §4 places symbols into those corners. |
| quiet zone = 4 modules/side | the layout adds a **fixed 2 mm** border: `qrBorder := params.I(2)` (`backup/wrap.go:199`), and the vertical band is rounded up to whole text rows: `qrLines := (qrsz + 2*qrBorder + fontSize - 1) / fontSize` (`wrap.go:202`) |
| 4.25 mm line pitch | **3.8 mm**: `plateFontSizeUR = 3.8` (`backup/backup.go:176`), and the line advance is the font size (`backup.go:388, 467`). The 4.25 figure is `85/20`, taken from an advisory *comment* in `crates/me-cli/src/lib.rs:45-47`, not from a constant the machine uses — `select.rs:20` says so: `const LINE_PITCH_MM: f64 = 85.0 / 20.0;` |
| "module size" as a free dimension | integer stroke multiples only: `engrave.QR(strokeWidth int, scale int, …)` (`engrave/engrave.go:277`) against `strokeWidth = 0.3 * mm` (`platform_sh2.go:213`) |

**The divergence.** The spec never enumerates the module candidate set. A
enumerates `{0.60, 0.90, 1.20, …}` (stroke multiples ≥ the §8.6 floor). B copies
the probe's `MODULES_MM = [0.30, 0.45, 0.60, 0.90]` (`select.rs:25`) and can
select **0.45 mm, which is 1.5 strokes and cannot be engraved**. C reads "module
size" as continuous and selects 0.72 mm. Separately, an implementer who honours
the screw holes and the 2 mm border gets fewer symbols per plate than §4's table
promises, and one who does not places a symbol over a screw hole.

I did not recompute any plate count; the point is that the *model* omits physical
features of the plate, not that a particular number is wrong.

---

### I-7 — Important — §2, §3, §5: the recoverer's 2040 walk is never written down, and nothing on the plate says what it is

§2 promises the codec specifies *"how a recoverer reassembles them, and how they
know a fragment is missing"* and that the legend makes the plate
*"self-describing"*.

**What I searched.** `grep -n -i` for `recoverer`, `scan`, `order`, `version`,
`mask`. Every `recoverer` hit (lines 78, 95, 242, 244, 245, 264, 367, 430, 452,
504) is an argument *about* a recoverer; none is a procedure *for* one. `scan`
appears once (line 218) and only as "without a scanner". `order` appears twice,
both about ECC degradation and framing durability, never about plates or symbols.

**Undecided, all of it:**

- What order the plates go in (the UR parts are self-describing, so it may not
  matter — but the spec never says that, and §10.3 explicitly leaves the question
  open).
- What order symbols sit in *within* a plate under tiling, or where they sit on a
  partially-filled last plate.
- Whether the seqNum of a fragment maps to its plate or its position at all.
  §5's legend has `PLATE n OF m`; the fragments are UR parts numbered `1..seqLen`;
  **the mapping between the two indices appears nowhere**, so a recoverer holding
  plates 1 and 3 of 3 cannot say which fragments they are missing.
- What they scan with, and what they do with the strings afterwards.
- **Nothing on the plate identifies the format.** The five legend fields are a
  bearer warning, a wallet stub, a locktime, a destination and a plate index.
  None says "Bitcoin transaction", "UR", "mt", or a format version. A finder in
  2040 gets `UR:BYTES/3-7/…` off a scanner and a plate that never tells them what
  the bytes are or what version of the plate format they hold. There is no
  forward-compatibility story — `version` appears in the spec only as *QR*
  version and as "the first version of this spec".

**The divergence.** A engraves a sixth legend line (`MT PLATE V1 — SCAN ALL,
DECODE AS UR:BYTES`) and pays a line out of §4's budget. B engraves nothing and
ships the procedure as documentation that will not be in the drawer. C numbers
symbols `SYMBOL k OF s` instead of plates. Different plates, different budgets,
different recovery odds.

*Non-authoritative sketch.* One line of the answer is free: see **§10.3 answered**
below — every UR part carries its own `seqLen`, so a decoder knows when it is
done and order does not matter. That fact belongs in §3 as a normative statement,
which then makes "plate order is irrelevant" a written rule rather than a hope.

---

### I-8 — Important — §3: the bytes that go into the UR are not specified

§3 rules the envelope (*"UR (Uniform Resources, BCR-2020-005), type `ur:bytes`"*)
and measures its overhead. It does not say what the **message** is.

- **CBOR wrapping.** BCR-2020-005 registers `bytes` as a CBOR byte string, so a
  conformant `ur:bytes` message is the transaction wrapped in a CBOR byte-string
  header, not the raw serialization. The fork's encoder does no wrapping —
  `ur.Encode` bytewords-encodes whatever it is handed (`bc/ur/ur.go:117-123`),
  because its callers hand it CBOR already. The probe likewise treats the message
  as raw bytes (`select.rs:53-58`). §3 never rules. **A emits `ur:bytes` of the
  raw tx; B emits `ur:bytes` of `h'<tx>'`.** The strings differ, the symbol sizes
  differ, and a strict third-party decoder accepts only one.
- **Bytewords style.** §3 says *"Bytewords minimal is exactly 2 characters per
  byte"* — a measurement, not a requirement. Standard style is 4 characters per
  byte and would double every symbol. Nothing in the spec **requires** minimal.
- **QR mode and case.** §3 observes the uppercased form *"is fully QR-alphanumeric"*
  and the probe encodes in alphanumeric mode (`select.rs:139`). Neither uppercasing
  nor alphanumeric mode is stated as a requirement, and lowercase would fall to
  byte mode and change every capacity in §4's table.

**Why it matters.** These three choices are the difference between "a Bitcoin
wallet in 2040 reads this plate" and "only `mt` reads this plate" — which is
F-234's entire point, quoted in §3: *"the QR's entire purpose is to be the escape
hatch for someone who has none of our tools."*

(Whether the BCR registry *requires* the CBOR wrapper is lens 3's call. What is
mine: the spec does not decide it, and the two decisions produce different steel.)

---

### I-9 — Important — §1a: `present` is a verb with no artifact

§1a rules *"Presenting is a screen and a file, never a plate. The medium is
`ur:psbt`, the same UR machinery §3 specifies, which is what Sparrow, Keystone,
Passport and Specter already consume as an animated QR."* That is the entire
specification of one of the three v0.1 verbs.

Undecided: whether the output is a single static QR when the PSBT fits one;
what `seqLen` is chosen when it does not; whether fountain parts beyond `seqLen`
are emitted (the animated-QR consumers cycle indefinitely, which is exactly the
case where redundancy is free — §10.6 discusses redundancy only for plates); the
frame rate; the QR version/ECC/module for a *screen* (§4's whole objective is
plate count, which is meaningless here); and what "a file" contains — a text file
of UR strings, a directory of PNGs, an animated GIF?

**The divergence.** A writes `psbt.txt` with one UR string per line and leaves
animation to the operator. B writes `frames/*.png`. C opens a window. The signing
device sees a different thing in each case, and in A's case may see nothing it can
consume.

Separately: §1a says `ur:psbt` while §3 specifies `ur:bytes`, so "the same UR
machinery §3 specifies" is not literally the same type; the fork's only UR type
string is `crypto-output` (`bc/ur/ur.go:111`). Whether the wallets named in §1a
read `ur:psbt` or `ur:crypto-psbt` is lens 3's question — but the spec must pick
one, because a wrong pick makes the verb inert.

---

### I-10 — Important — absent: no test vectors, no reference artifact, no conformance surface

**What I searched.** `test vector` — zero hits. `vector` — zero hits. `fixture` —
zero hits.

`mt` is a new normative format whose stated purpose is to be decoded by an
independent implementation after a long dormancy. §11 documents the provenance of
the *sizing* numbers; nothing pins the *format*.

**The divergence.** Every ambiguity in I-2, I-6 and I-8 stays undetected. A and
B ship, both green against their own tests, and produce different QR strings and
different legends for the same transaction — with nothing in the repo that would
fail. This is the project's own "a corpus can be uniformly wrong" and
"cross-language vectors see what no repo test can" lessons applied one layer
earlier: the vectors must exist *before* there are two implementations to
disagree.

*Non-authoritative sketch.* A `vectors/` directory keyed on the measured
transactions already in `design/measurements/`: for each, the exact UR strings,
the selected (module, version, ECC, symbols, plates), and the exact legend text —
so §4's table becomes executable rather than descriptive.

---

### I-11 — Important — §5: "the manifest" is load-bearing and never specified

§5 line 241-245, on the block anchor and MTP bound dropped from the legend:

> They survive as **`mt` output at encode time** — printed, and available in the
> manifest — just not on steel …

`manifest` occurs **exactly once** in 534 lines (verified by grep). It has no
definition, no schema, no fields, no filename, no lifetime.

This is not cosmetic: §6c's whole security argument is that verification is
*deferred* to a recoverer with a node, and §6d says *"The plate carries only the
durable anchor — block height, block hash, and the MTP bound"* — except §5 has
just removed all three from the legend and parked them in the manifest. **The
manifest is now the only durable home of the evidence §6c and §6d spend two
sections justifying**, and it is undefined. If it is a stdout print that nobody
saves, the deferral in §6c.3 fails silently.

**The divergence.** A prints the anchors to stdout and calls it done. B writes
`manifest.json` next to the plates. C reuses `me bundle`'s manifest shape
(`crates/me-cli/src/manifest.rs`, a serde `Manifest`/`PlateEntry` type keyed on
`md1`/`mk1-chunk`/`ms1` plate kinds, which has no field for any of this). Only B
leaves the recoverer anything.

---

### M-1 — Minor — §4: the table has a `raw bytes` column and no ruling in §4 about when it applies

§3 rules the envelope is UR. §4's table then presents `raw bytes` and `UR` side by
side for seven artifacts, with the raw column consistently cheaper (1 plate vs 2
for 3-of-5, ECC H vs ECC L for tier 4). §4 itself never says the raw column is
non-normative. A reader who starts at §4 — the section titled "Choosing the
configuration" — can reasonably take it as a choice. The probe encodes them as
two different functions (`best` in byte mode, `best_ur` in alphanumeric,
`select.rs:86, 126`) and prints both. One sentence in §4 closes it.

### M-2 — Minor — §11: the provenance citations for the plate and module constants point at unrelated lines

§11 line 536-538: *"plate and module constants read from the fork
(`backup/backup.go:45,99-102`, `cmd/controller/platform_sh2.go:188`)"*.

The constants are elsewhere: `outerMargin = 3` is `backup/backup.go:73`,
`plateSize = 85` is `backup/backup.go:77`, and `strokeWidth = 0.3 * mm` is
`cmd/controller/platform_sh2.go:213`. Line 45 of `backup.go` is a comment inside
the `Text` struct; lines 99-102 are inside `fixedCharWidth`; line 188 of
`platform_sh2.go` is StallGuard commentary about TCOOLTHRS.

I am not re-checking the 16 gated anchors — I found these while looking for the
constants themselves. `scripts/plan-cite-check.sh` states this blind spot in its
own header (*"It proves a cited line EXISTS … It does NOT prove the doc's
INTERPRETATION of that line is correct"*), so the gate reporting `ok` here is the
gate working as documented. The values in §4 (85, 3, 0.30) are correct; only the
pointers are not, and an implementer following them finds nothing.

### M-3 — Minor — §8: "every refusal names the number that caused it" has no content and no exit codes

§8's closing promise is testable in principle and untestable as written: no
message template for any of the seven refusals, and no exit code for any of them.
§8.5 comes closest (*"naming the exact plate count and what would fit"*) and still
does not say what "what would fit" means — a byte count? a smaller ECC? fewer
outputs?

### M-4 — Minor — §6b: how `mt` reaches `bitcoind` is unspecified

§6b rules the RPC (`gettxout <txid> <vout> false`) but not the connection: cookie
file vs `rpcuser`/`rpcpassword`, host/port, `-rpcwallet`, and — materially —
**which network**. A mainnet stub, a signet transaction and a testnet address all
render identically in §5's legend. Implementers will invent different flags; the
engraved artifact is the same, so Minor.

### M-5 — Minor — §3/§10.3: `Progress()` is not a completion test and the spec should say so

Detail in **§10.3 answered** below. Recorded here because an implementer porting
the fork's decoder to Rust could reasonably take `Progress() == 1` as "done" and
ship a decoder that declares success on an incomplete message.

### N-1 — Nit — §0, §1a heading and §1a table give the three verbs in three different orders

§0: "produces … presents … engraves". §1a's heading: "Present, produce, engrave".
§1a's table rows: engrave, present, produce. Harmless, but the section exists
because *"conflating them caused two reversals"*, so a fixed order is cheap.

### N-2 — Nit — §5: `SPENDABLE AFTER BLOCK <n>` does not say whether `<n>` is nLockTime or nLockTime + 1

A transaction with `nLockTime = N` is includable in a block of height `> N`.
"AFTER BLOCK N" reads correctly for `n = nLockTime`, which is presumably the
intent, but it is one sentence to pin and a one-block engraved error to get
wrong.

---

## §10.3 answered

> **3. How does a recoverer learn the fountain parameters?** UR carries seqNum and
> seqLen, but a fountain-coded set needs the decoder to know when it has enough.
> Confirm the vendored `Decoder` reports this, rather than assuming it.

**Answered from source. The vendored decoder does report it, and the parameters
ride in every part.** Read at `/scratch/code/shibboleth/seedhammer`:

1. **Every part self-describes.** `part` is a 5-element CBOR array
   (`cbor:",toarray"`, `bc/fountain/fountain.go:74-81`) of `SeqNum` plus an
   embedded `partHeader` — `SeqLen`, `MessageLen`, `Checksum`
   (`fountain.go:83-87`) — plus `Data`. So `seqLen` and the total message length
   are in **every** fragment, not only the first. A recoverer holding any single
   plate already knows how many fragments exist.
2. **The decoder latches and enforces the header.** `Decoder.Add`
   (`fountain.go:110-116`) takes `d.header = p.partHeader` from the first part and
   returns `"fountain: incompatible fragment"` for any later part whose header
   differs — so fragments from two different transactions cannot be mixed.
3. **`Result()` is the authoritative "have enough" signal.**
   `fountain.go:197-200`: `if len(d.completed) != d.header.SeqLen { return nil, nil }`.
   It returns non-nil only when all `seqLen` pure fragments have been recovered
   (directly or by XOR reduction), then concatenates in fragment order, truncates
   to `MessageLen` (`:215`) and verifies CRC32 against `Checksum` (`:216-219`).
   `ur.Decoder.Result()` (`bc/ur/ur.go:139-148`) surfaces the same contract.
4. **`Progress()` is a UI heuristic and must not be used as the test.**
   `fountain.go:89-96` divides received parts by `float32(d.header.SeqLen) * 1.75`
   and clamps at 1. With `seqLen = 4`, seven mixed parts give `Progress() == 1`
   while `Result()` is still `(nil, nil)`. §3's CORRECTION block already says the
   1.75 figure *"describes random reception, not the deterministic set an engraver
   emits"*; the operational consequence — **`Result() != nil` is the only
   completion test** — is not yet written down anywhere.
5. **Corruption is caught before the fountain sees it.** `bytewords.Decode`
   verifies the trailing CRC32 and returns `"crc32 checksum mismatch"`
   (`bc/bytewords/bytewords.go:48-53`), so a misread fragment is rejected rather
   than XOR'd into the message.
6. **One thing an implementer would get wrong.** `ur.Decoder.Add`
   (`bc/ur/ur.go:177-184`) parses the `<n>-<m>` prefix with `Sscanf` and then
   **discards both values**, passing only the bytewords payload to
   `fountain.Add`. The authoritative `seqNum`/`seqLen` are the CBOR ones; the URI
   prefix is validated for syntax only. A plate whose engraved prefix disagrees
   with its body decodes by the body.
7. **Single-part URs are a different code path.** `ur.Decoder.Add` at
   `bc/ur/ur.go:185-187` sets `d.data = enc` directly — no fountain wrapper, no
   `MessageLen`, no `Checksum`. Integrity rests entirely on the bytewords CRC32.
   `ur.Encode` mirrors this at `ur.go:118-119`. §3 already notes the size effect
   (*"A single-part UR skips the fountain wrapper entirely"*); the *integrity*
   difference is unremarked.

**Verdict on the question itself: this is a gap, not an open question.** It was
answerable in a fifteen-minute read of two files already vendored in a sibling
repo, and the answer is normative — it decides whether §2's promise (*"how a
recoverer reassembles them, and how they know a fragment is missing"*) is
delivered by delegation or must be specified. It should be closed in §3 before
the gate, with points 3, 4, 6 and 7 stated as rules, not left as `Decoder`
behaviour an `mt-codec` Rust port might not reproduce.

---

## §10.4 answered

> **4. Does `mt` verify the transaction against the source wallet** when both the
> md1 card and prevouts are supplied — i.e. can it prove the stub is honest at
> encode time, even though nothing may branch on it at decode time?

**Not deferrable as posed — but the deferrable part and the blocking part are
different, and the spec conflates them.**

**Deferrable: the verification itself.** §5 already rules the stub *"is a hint,
never an authority … If the legend says wallet X and the transaction spends wallet
Y's UTXOs, **the transaction wins** … nothing may branch on it."* Given that
ruling, an encode-time check can only ever produce a **warning**, never a refusal
— it cannot change what is engraved unless the spec adds a refusal, which §5
forbids. So "does `mt` cross-check the stub against the prevout scriptPubKeys" is
a genuine v0.1-vs-later choice, and deferring it does not make the spec
unimplementable. The machinery exists in the constellation (the md1 card resolves
to a descriptor, and prevout scriptPubKeys come from §6a's tiers), so it is
cheap later.

**Blocking: where the stub comes from at all.** That is I-1 above, and it is not
the same question. §10.4 presupposes "when both the md1 card and prevouts are
supplied" — but **the spec never establishes that an md1 card is an input to
`mt`**. `derive_stub_from_md1_card(card: &[&str])`
(`mnemonic-key/crates/mk-cli/src/cmd/mod.rs:126`) needs one, §5 mandates the
`FROM WALLET <8 hex>` legend line, and §4 reserves height for it. So the spec
requires an output it has no specified input for. That must be closed before
implementation.

**One consequence worth stating explicitly**, because §5's "the transaction wins"
ruling is doing more work than it looks like it is: with no verification and no
required input, `FROM WALLET <8 hex>` is an **operator-asserted string engraved
permanently on a bearer instrument**, structurally identical to the
self-certified amounts §6a refuses by default. §6a's own argument —
*"Self-certified amounts are checked against themselves"* — applies verbatim to a
self-certified wallet stub. That asymmetry (amounts refused, stub accepted
unchecked) is not wrong, but it is undefended, and defending it is one sentence.

*Non-authoritative sketch.* Close §10.4 as: "the md1 card is an optional `mt`
input; when supplied with prevouts, `mt` cross-checks the derived scriptPubKeys
and **warns** on mismatch (never refuses, per §5); when not supplied, the legend
line reads `<X>` and §4 reserves/does not reserve its height accordingly."
The last clause is the one that matters — it is the only one that changes plate
counts.
