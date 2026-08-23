# R0 round 0 — lens 1: DESIGN AND ARCHITECTURE

Artifact: `design/SPEC_mt_v0_1.md` (538 lines) at commit `099a516`.
Question answered: *is this design sound and internally consistent, and could a
competent implementer build exactly one thing from it?*

Answer: **no, not yet.** The transaction-handling half of the spec (§0, §1a, §6,
§6a–§6d, §7, §9) is coherent and unusually well-reasoned. The *plate* half — §4's
selection, §5's legend, and the boundary to the machine — does not close: §4's
stated objective is not a total order, §5's legend is contradicted by four other
sections that spend it, and §4 selects a configuration that nothing in the
constellation can currently be told to cut.

Everything I assert about the fork is cited against **`third_party/seedhammer`**
(the pinned submodule, upstream v1.4.2), which is the tree §11's own citations
resolve against (`backup/backup.go:45` is `const outerMargin = 3`, exactly as
§11 says). Where I checked, the bg fork working copy at
`/scratch/code/shibboleth/seedhammer` (HEAD `a91df84`) agrees on every fact,
at different line numbers.

Machine-checked before writing: I re-ran §4's reference search
(`design/measurements/mt-size-probe/src/bin/select.rs`) in a scratch crate,
modified only to *enumerate* every candidate tying the winner instead of keeping
one. Output is quoted in D-1. No repo file was edited.

## Verdict

**3 Critical / 5 Important / 4 Minor / 2 Nit**

---

### D-1 — §4's objective is not a total order, and the un-ordered dimension is the one that decides legibility

**Severity: Critical** (unmet guarantee) — §2, §4

§2 states, as one of the five things that make `mt-codec` "a real format":

> - which (module size, QR version, ECC level, tiling) configuration is chosen for
>   a given transaction, deterministically, so two encoders agree;

§4 gives the search space and the objective:

>     search space:  module size x QR version (1..40) x ECC (L,M,Q,H) x k*k tiling
>     objective:     minimise plates      <- a plate must hold the QR AND its legend
>                    then maximise ECC
>                    then minimise symbol count

Four dimensions in the search space; three terms in the objective. **Module size
is never ordered at all**, and the *set* of admissible module sizes is never
enumerated anywhere in §4 — §4 gives only a floor ("`mt` must not select a module
below 0.60 mm"), and the only enumeration in the document is in §10.2, as a list
of things a *test plate* should try. So the objective is a partial order over an
undefined domain.

This is not theoretical. Enumerating every candidate that ties the winner on
§4's own three terms, using §4's own reference search:

```
=== 162 B raw, 0.60mm floor ===
  winning objective tuple: plates=1 ECC=H symbols=1
  candidates TIED on the spec's objective: 4
    v13 ECC H @0.60mm   v14 ECC H @0.60mm   v15 ECC H @0.60mm   v16 ECC H @0.60mm

=== 162 B raw, 0.30mm floor (AGGRESSIVE, i.e. after F-234) ===
  winning objective tuple: plates=1 ECC=H symbols=1
  candidates TIED on the spec's objective: 41
    v13..v38 ECC H @0.30mm ; v13..v23 ECC H @0.45mm ; v13..v16 ECC H @0.60mm
```

162 B is the most ordinary artifact in the table — a single-sig `tr` key-path
spend, one input. Today, four configurations satisfy §4 equally. After F-234
lifts the floor, forty-one do, spanning three different module sizes.

Two consequences, in ascending order of seriousness:

1. **Two conforming encoders produce visibly different plates**, which is exactly
   what §2 promises they will not. §4's published table reports `v13 ECC H` and,
   in the aggressive tier, `v13 ECC H @0.30mm` — but nothing in §4 selects those.
   They win because the reference implementation iterates modules ascending,
   versions ascending, and keeps the first of an equal tuple (`if better` on a
   strict `<`). The tie-break is loop order, and loop order is not in the spec.
2. **Loop order resolves the tie in the worst physical direction.** Among the 41
   tied candidates, the one the reference picks is the *smallest* module (0.30 mm,
   the optically unvalidated one) at the *smallest* version — i.e. the physically
   smallest, least legible, most damage-sensitive symbol in the tie set, when a
   0.60 mm / v16 symbol was available at identical plate count, identical ECC and
   identical symbol count. §4 is visibly careful about module size elsewhere
   ("optically unvalidated", a hard floor, a refusal in §8.6) and then lets an
   accident of iteration order spend that care.

The floor in §8.6 masks (1) and (2) only while it is in force. F-234 exists in
order to lift it, and lifting it takes the tie set from 4 to 41.

*Non-authoritative sketch:* enumerate the module set explicitly, and add terms
that make the order total — e.g. after `minimise symbols`, `maximise module size`,
then `maximise version`. Note that "maximise module size" is not a tie-break of
convenience: it is the term that makes §4 agree with its own stated worry about
optical legibility.

---

### D-2 — §5's legend is contradicted by §6a, §6c, §6d and §7, which spend fields §5 deleted

**Severity: Critical** (internal contradiction; §4's entire plate table rests on
the losing side) — §5 vs §6a, §6c, §6d, §7

§5 fixes the legend at exactly five fields:

> **The legend carries only what a human needs BEFORE the QR is decoded.** Five
> fields, 136 characters, 6 lines

and enumerates what it removed, including:

> | input outpoints | they are the transaction's inputs |
> | block hash + height per input | from the outpoints, with any node (§6c) |
> | `input existed not before <MTP>` | from the block, with any node (§6d) |
> | fee rate and date | inputs − outputs, once prevouts are known |

> They survive as **`mt` output at encode time** — printed, and available in the
> manifest — just **not on steel**

Four later sections then engrave those exact fields. Eight distinct claims:

| where | what it says the legend carries | §5 status |
| --- | --- | --- |
| §6a tier 3 | "accepted, caveat stated on screen **and in the legend**" | no such field |
| §6a tier 4 | "`--i-certify-amounts` overrides and **the legend records it**" | no such field |
| §6c limit 1 | "it is disclosed the same way — **the outpoints go on the plate**" | dropped |
| §6c limit 3 | "`mt` stores the block hash and height and **puts them in the legend**" | dropped |
| §6c close | "only the block hash and height **go in the legend**" | dropped |
| §6d | "**the legend states a bound**, *"input existed not before <MTP>"*" | dropped |
| §6d | "**The plate carries** only the durable anchor — block height, block hash, and the MTP bound" | dropped |
| §6d close | "**the input outpoints go on the plate** so a holder can check" | dropped |
| §7 | "legend carries **the input outpoints** so a holder can check they are still unspent" | dropped |
| §7 | "legend **states rate and date** so staleness is visible" | dropped |

Why this is Critical rather than an editing slip:

- **§4's entire published table depends on the losing side.** §4 reserves "6 lines
  ... (25.5 mm at a 4.25 mm pitch)" *because* §5 is five fields / 136 characters,
  and §4 says plainly what that reservation costs: "reserving 25.5 mm drops small
  artifacts by two or three ECC levels ... and doubles the plate count on
  everything from RCW `wsh` tier 1 upward." Restoring even one of the dropped
  fields moves every number in the table. Input outpoints alone are ~70 characters
  each — five inputs is 350 characters against a 136-character legend, which is
  precisely the failure §10.5 records as already having happened once
  ("the ten-field legend was 474 characters against a 300 budget").
- **§7's mitigations do not exist.** Two of the four hazards in the threat model
  are mitigated *only* by legend fields §5 deleted. Silent invalidation's entire
  mitigation is the outpoints; pinned fee's is rate and date. On the current §5,
  those rows mitigate nothing. (The funds-safety grading of that is lens 2's; I
  report it as an internal contradiction because §7 and §5 cannot both be true.)
- **The direction of the fix is not obvious**, which is why I am not prescribing
  one. §5's argument for dropping is sound on its own terms ("everything derivable
  from the decoded transaction is duplication") and §7's argument for keeping is
  sound on its own terms (a holder must be able to check unspentness *without*
  decoding). Both cannot hold. Deciding which wins changes §4's table.

---

### D-3 — §4 selects a configuration the target machine cannot be told to use, and no section says who closes the gap

**Severity: Critical** (the spec's central engineering result is unbuildable as
scoped) — §4 vs §1.2, §9, §10.7

§4's chosen configuration has four degrees of freedom: module size, QR version,
ECC level, and a tiling of several symbols on one plate. The SeedHammer engraving
path — the only path in the constellation that cuts a plate — supports none of
them as a parameter.

| §4 selects | what the machine does | citation (pinned submodule) |
| --- | --- | --- |
| ECC L/M/Q/H, maximised | ECC is a **compile-time constant** per plate type: `qr.Encode(desc.EncodeCompact(), qr.L)`, `qr.Encode(seed, qr.M)`. Never a parameter, never host-supplied. | `gui/gui.go:401`, `backup/backup.go:77` |
| QR version 1..40 | Chosen by the encoder as the smallest that fits the text. There is no version input. | same call sites |
| module size (continuous) | `qrsz = qrc.Size * StrokeWidth * qrScale` with `qrScale` an **integer** and `strokeWidth = 0.3 * mm`. Realisable modules are 0.3, 0.6, 0.9, 1.2 mm — and `qrScale` is a `const` in the caller (`const qrScale = 3`), not an input. | `cmd/controller/platform_sh2.go:188`, `gui/gui.go:405` |
| k*k tiling, up to "6 qr" on a plate | **One code per plate.** Both production callers build `Paragraphs: []backup.Paragraph{e.Paragraph}` — a one-element slice — and the placement pins the single code to the right margin: `qrx := plateDims.X - qrsz - margin - qrBorder`. There is no horizontal tiling primitive. | `gui/gui.go:431`, `backup/backup.go:325` |

Two further geometry mismatches in the same direction:

- §4 models the usable plate as **79 mm** (85 − 2×3 outerMargin) in both axes. The
  machine additionally keeps the code clear of the **screw-hole band** —
  `qry := offy + holeLines*fontSize + …` where `holeLines = ceil((innerMargin −
  margin)/fontSize)` and `innerMargin = 10` — and adds a 2 mm `qrBorder` on each
  side of the code's band. §4's model has neither. `innerMargin` is on the very
  line after the `outerMargin` §11 cites (`backup/backup.go:45,46`).
- §4 reserves the legend as a **full-width horizontal band** above/below the code
  (`h_first = USABLE_MM − 6 × pitch`). The machine flows text **beside** the code,
  on the same rows, at a narrowed budget: `charPerQRLine = (width − 2*qrBorder −
  qrsz) / charWidth`. So §4's model is not a conservative approximation of the
  machine's layout — it is a *different* layout, and its plate counts are
  therefore neither an upper nor a lower bound on the real ones.

What makes this Critical rather than "firmware work, obviously": **the spec's own
scope excludes it.** §1.2 delivers "`mt-codec` and an `mt` CLI". §9's out-of-scope
list is "Signing; broadcasting; RBF or CPFP; watching the chain to detect
invalidation; any machine-readable provenance; sealed or encrypted plates" — no
firmware item, in or out. And §10.7 proves the spec knows how to flag exactly this
class of gap when it sees it:

> But there is **no back-side path in the fork** ... This is firmware work, not a
> free option — but it is the single highest-value change to these numbers, so
> cost it before accepting the doubled plate counts.

The identical sentence is owed to §4's ECC axis, its module axis, and its tiling —
and §4's numbers are far more load-bearing than §10.7's. As written, an
implementer builds `mt-codec`, gets a configuration, and discovers at the last
step that there is no way to ask for it.

---

### D-4 — no layering statement: what belongs to `mt-codec`, to the `mt` CLI, to the firmware, and what crosses the boundary

**Severity: Important** (missing case) — §1.2, §2, §5, §6b

The spec never says which component does what, nor what artifact `mt` hands over.
Specifically unassigned:

- **Plate geometry has no named owner.** §4's constants (79 mm usable, 4.25 mm
  pitch, 4-module quiet zone, 25.5 mm reservation) are re-derived inside the
  selection algorithm. The constellation already has an answer for this and it is
  the opposite one: `me` does **not** re-derive SeedHammer geometry: `me bundle
  --preview` shells out to the `me-preview` Go sidecar, whose `preview/layout.go`
  imports `seedhammer.com/backup` and calls the fork's real fit/layout code. D-3
  is what re-deriving costs — four independent drifts from the machine, none of
  which a sidecar call could have produced.
- **The artifact that crosses the `mt` → machine boundary is never named.** Is it
  UR fragment *strings* (in which case the firmware's own encoder picks version
  and ECC, and §4's selection is advisory at best — see D-3), a pre-encoded
  `*qr.Code` per symbol, a rendered toolpath, or an NDEF payload on the `me` path?
  Each answer implies a different `mt-codec` API and a different firmware change.
- **"The manifest" is undefined but load-bearing.** §5 justifies deleting four
  legend fields partly because "they survive as `mt` output at encode time —
  printed, and available in the manifest". §6b adds more: "`mt` records which tier
  supplied each amount, so the provenance of the numbers is auditable after the
  fact." No section defines a manifest — its format, where it is written, whether
  it is part of `mt-codec` or the CLI, or how it stays associated with a plate set
  that by construction outlives the machine that produced it. `me` already has a
  `manifest` module (`crates/me-cli/src/manifest.rs`) with a different meaning
  (the plates a wallet backup needs), so the word is already taken in this
  constellation.

An artifact whose entire point is a 2040 recovery cannot leave "where the durable
record of the encode lives" to the implementer.

---

### D-5 — "k*k tiling" is not the tiling that produced §4's table

**Severity: Important** (the normative algorithm is not the algorithm behind the
numbers) — §3, §4

§4 says the search space is `module size x QR version (1..40) x ECC (L,M,Q,H) x
k*k tiling`. The model that produced §4's table is a **rectangular** grid whose
capacity differs between plate 1 and later plates:

```
across     = floor(79 / footprint)
rows_first = floor((79 - 6*4.25) / footprint)     // plate 1: legend reserved
rows_rest  = floor((79 - 1*4.25) / footprint)     // later plates
first_cap  = across * rows_first
per_plate  = across * rows_rest
```

Three things follow that §4 never states, and that an implementer building "k*k"
would get wrong:

1. **The grid is not square.** For the RCW `wsh` tier 1 RAW row, `across = 2`,
   `rows_rest = 2`, `rows_first = 1` — plate 1 holds 2 symbols and plate 2 holds
   4. §4's table reports "**2 plates**, 6 qr" and 6 is not `k*k` for any k.
2. **Plate 1's capacity is smaller than every later plate's**, because the legend
   is reserved there and not elsewhere. §4 states the *reservation* but never
   states the consequence, which is that the first plate is not interchangeable
   with the others.
3. **Plate 1 can hold zero symbols** — the legend alone occupies a whole plate.
   The model has an explicit branch for it (`else if first_cap == 0 { 1 +
   symbols.div_ceil(per_plate) }`), and my enumeration confirms the published
   9-of-11 RAW row is exactly that case: *"v24 ECC L @0.60mm (plate-1 capacity 0
   symbols)"*. Several rows in §4's table read "**2 plates**, 1 qr" — one symbol
   costing two plates — and that is unexplainable from §4's text, which says only
   that "a plate must hold the QR AND its legend".

(3) also has a design consequence nobody has ruled on: a legend-only plate costs a
full plate cycle — "one plate per string today, ~21 minutes each (F-225)" — to
engrave six lines of text, and adds a physical object to the set that carries no
recoverable data. Whether that is the right trade against, say, allowing a
slightly smaller symbol so the legend can share, is a decision the spec makes by
accident.

---

### D-6 — §4's first rule already answers §10.6 in the negative, while §10 defers it

**Severity: Important** (contradiction on the spec's own most consequential
question) — §3, §4, §10.6

§4:

> Plate count is the real cost ... so the search minimises plates first and spends
> every leftover byte on error correction. **Never trade a plate for redundancy;
> never leave redundancy unbought.**

§10.6:

> **How much fountain redundancy should `mt` emit?** Parts beyond `seqLen` are real
> fountain parts, each tolerating one more lost symbol at the cost of one more
> symbol — **often one more plate** (§3) ... **The most consequential undecided
> question in this spec**, trading directly against §4's plate counts.

Read together: §10.6 asks whether to spend a plate on redundancy, and §4 has
already answered "never". The spec simultaneously forbids the trade and defers it.

Two contributing defects:

- **"Redundancy" names two different things in two sections.** §3 uses it
  exclusively for fountain parts ("Redundancy is therefore a choice with a price,
  not a property"); §4 uses it for the Reed–Solomon level inside a symbol. They are
  drawn from the same slack and traded against each other, so one word for both is
  not survivable in a normative rule. An implementer can read "never leave
  redundancy unbought" as an instruction to emit fountain parts.
- **Redundancy is not a dimension of the search space.** §4 searches over module,
  version, ECC and tiling, and derives symbol count from the payload. A redundancy
  count `r` changes the symbol count, hence the tiling, hence the plate count —
  so resolving §10.6 is not a parameter change, it is a re-specification of §4.
  There is a small trap here too: an extra part has `seqNum > seqLen`, so its CBOR
  `SeqNum` scalar and its `<n>-<m>/` prefix can each be one character wider than a
  pure fragment's, and §3's per-fragment sizing is written for `seqNum <= seqLen`
  ("§4's selection now models this **exactly**"). The exactness claim does not
  survive redundancy.

See the recommendation at the end.

---

### D-7 — the objective's second and third terms encode a damage model the spec never states, and it is the wrong one for the failure that matters

**Severity: Important** (unsound assumption) — §3, §4

`maximise ECC` before `minimise symbol count` means: at equal plate count, prefer
more symbols at a higher ECC level over fewer symbols at a lower one. §4's own
table shows the rule doing exactly that — the RCW `wsh` tier 1 RAW row is
"**2 plates**, 6 qr, ECC Q", chosen over configurations at the same two plates
carrying **one** symbol at ECC L (v24 ECC L holds the whole 742 B payload; my
enumeration reaches it).

That preference is only correct under a damage model the spec never writes down:

- **ECC is per-symbol and intra-symbol.** It recovers a symbol from *distributed*
  damage — scratches, pitting, partial corrosion.
- **It cannot recover a lost symbol at all**, and under §3's finding that parts
  `1..seqLen` are singletons, *every* symbol is an all-or-nothing dependency:
  "one unreadable plate and the transaction is gone."

So the rule buys deeper protection against the failure that is already partly
covered, by *multiplying* the number of single points of total failure — six of
them instead of one, for the same plate count and the same steel. Whether ECC Q on
six symbols beats ECC L on one depends entirely on whether damage on brushed steel
is distributed (favours many-small-at-high-ECC) or localised/structural, i.e.
killing finder and timing patterns (favours few-large). That is the F-234 optical
question, and §4 has committed to an answer before F-234 has been cut — the same
mistake §8.6's module floor exists to prevent.

Note also that the *first* term is justified purely by production cost — "one plate
per string today, ~21 minutes each" — on an artifact whose stated purpose is to
survive decades in a drawer, and that §7's hazard table contains no
production-cost hazard. Minimising plates is a defensible top term, but the spec
never argues for it against survivability; it argues for it against *time*.

---

### D-8 — §8's refusals are not scoped to a verb, so the §1a split leaks

**Severity: Important** (missing case in the safety-critical list) — §1a vs §8,
§6a, §6b

§1a is the spec's answer to a problem it says caused two reversals, and it holds
cleanly in §0, §6a, §6b and §9. It leaks in §8. §8's preamble scopes the whole
list to engraving — "All are machine-checkable **before a single plate is cut**" —
and refusal 3 is explicitly verb-scoped ("An unsigned transaction offered for
ENGRAVING"). The rest are not, and three of them are defined in terms of
information only the *produce* path holds:

- **3a — "Input amounts asserted without proof → refuse unless
  `--i-certify-amounts` (§6a)".** At engrave time `mt` is handed a signed,
  finalized transaction, which contains **no** input amounts — only outpoints
  (§6, "A signed transaction references inputs as outpoints only"). So the
  predicate is either undefined at engrave time or silently requires prevouts —
  which refusal 2 treats as *optional* one line earlier ("Script-invalid → refuse,
  **when prevouts are supplied**"). Two adjacent refusals disagree about whether
  prevouts are mandatory.
- **3b — "`gettxout` returns `null` for any input → refuse, no override".**
  Stated flatly, it makes a reachable node mandatory for engraving, which
  contradicts §6b's conditional framing ("**If** a node is available") and §6d's
  "keeps `mt` usable offline, which is the constellation's whole posture". If the
  refusal is vacuous when no node is present, the spec must say so — a refusal
  that silently disappears is worse than one that never existed.
- **4 — "Broadcastable today → refuse by default".** Deciding "today" requires the
  chain tip (height or MTP). Offline there is none, and the spec names no
  operator-supplied substitute. This is also the enforcement point for §1's
  ruling 6, "A future locktime is required by default" — the spec's one temporal
  safety property has no defined input.

Also unstated: whether §6a's tier table binds `present`. §6a is titled
"**Producing**: where the input amounts come from" and §1a says a handed PSBT's
amounts are someone else's responsibility ("whoever built it took responsibility")
— yet the tier table's refused tier is "bare asserted amount (`witness_utxo`
alone)", which is precisely what a Sparrow- or Specter-produced segwit PSBT
normally carries. If §6a binds `present`, `mt` refuses to display the mainstream
case; if it does not, §8.3a re-imposes it at engrave time anyway. The spec needs
one sentence per refusal naming the verb it binds.

---

### D-9 — §2's five bullets do not establish a codec, so the §1.2 objection is not in fact answered

**Severity: Minor** (the ruling is the operator's; the justification is what is
weak) — §1.2, §2, §3

§1.2 records the ruling and claims the objection was met:

> **This overrules the recommendation in §Section 1 of the brainstorm**, which
> argued `mt` had no wire format left to define ... See §2 for what the codec does
> in fact specify; **the objection was answered rather than ignored.**

Taking §2's five bullets in order: bullets 1 and 2 (how a transaction maps onto
symbols; how a recoverer reassembles and detects a missing fragment) are
**delegated wholesale to UR** by §3 — "Fragmentation uses **UR**" — and §10.3 even
lists half of bullet 2 as still unconfirmed ("Confirm the vendored `Decoder`
reports this, rather than assuming it"), so §2 claims as specified something §10
lists as open. Bullets 3 and 4 (configuration selection; what is engraved beside
the symbols) are plate geometry. Bullet 5 (what `mt` refuses) is transaction
policy requiring `bitcoin` + `bitcoinconsensus` + an RPC client, not a codec. §2
concedes the shape of this itself:

> It is a *plate* format rather than a *string* format, which is why it has no
> bech32 HRP and no BCH checksum.

I am not challenging the ruling — a separate repo may well be right for release
cadence, dependency weight (`bitcoinconsensus`, an RPC client) and blast radius.
The defect is that §2 is offered as the *reason* and does not carry it, which
matters because the load-bearing consequence — a second plate-geometry
implementation with no stated relationship to `me`'s — is left unexamined (D-4),
and has already drifted (D-3).

---

### D-10 — §10.2's 0.45 mm module is not producible on this machine

**Severity: Minor** — §4, §10.2

§10.2 says the F-234 plate "should test 0.30/0.45/0.60/0.90 mm modules", and
§4's reference search includes 0.45 mm (it appears in eleven of the 41 tied
candidates in D-1). A module is `StrokeWidth * qrScale` with `qrScale` an integer
(`gui/gui.go:405`) and `strokeWidth = 0.3 * mm`
(`cmd/controller/platform_sh2.go:188`), so realisable modules are integer
multiples of 0.3 mm. 0.45 mm is 1.5 strokes and has no representation. Cutting
that rung of the test plate would require a different tool or a sub-stroke
strategy the spec does not describe — and a test plate that cannot cut one of its
four rungs answers a smaller question than §4 needs.

---

### D-11 — the legend grid is `me`'s explicitly advisory TEXT-ONLY constant, not the machine's layout

**Severity: Minor** — §5, §11

§5 measures the legend against "a 300-character budget
(`crates/me-cli/src/lib.rs:48`)", and the measurement file derives the grid as
"35 chars/line x 20 lines, **TEXT-ONLY plate**". That constant's own doc comment
reads:

> Conservative single-plate text budget. SeedHammer's 85x85mm text layout wraps
> ~35 chars/line over ~20 usable lines; **with a QR present, far less.**
> **This is an advisory pre-check** — the firmware still backstops with ErrTooLarge.

Every `mt` plate carries a QR by construction. So §5's line count and §4's 4.25 mm
pitch descend from an approximation whose source explicitly disclaims itself for
this case, while §11 states that "plate and module constants [were] read from the
fork". The real grid is computable — `charPerLine = width / charWidth` and
`charPerQRLine = (width − 2*qrBorder − qrsz) / charWidth`, both in
`backup/backup.go:288` and reachable from the `me-preview` sidecar. Small numbers,
but they set the 25.5 mm reservation that §4 says "doubles the plate count".

---

### D-12 — §5's locktime field cannot express a time-typed locktime

**Severity: Minor** — §1.6, §5, §8.4

§1's ruling 6 is "**A future locktime is required by default**", and §5's legend
renders it as one fixed field, `SPENDABLE AFTER BLOCK <n>` (29 characters,
measured as `SPENDABLE AFTER BLOCK 1383520`). `nLockTime` has two forms and only
one of them is a block height; a transaction using the other satisfies §1.6 and
§8.4 but cannot be described by §5's only temporal field. §8.4 already contemplates
substituting the line (`IMMEDIATELY SPENDABLE` under `--allow-immediate`), so the
mechanism exists; the case is just missing. (The protocol detail belongs to
lens 3; I record it as an internal completeness gap between §1.6 and §5.)

---

### D-13 — "signed" and "finalized" are used interchangeably

**Severity: Nit** — §0, §1a, §8.1

§1a's table, the normative statement of the three verbs, says engrave is "**yes —
signed transactions only**", and §0 says `mt` "engraves the signed result". §8.1
requires more: "Not finalized → refuse. Every input must carry a witness or
scriptSig." A 2-of-5 PSBT carrying two signatures is signed and not finalizable;
§8.1 correctly refuses it and §1a appears to admit it. One word in §1a's table
fixes it.

---

### D-14 — §7 says the legend "names the destination"; §5 truncates it

**Severity: Nit** — §5, §7

§7's mitigation for the pinned-destination hazard is "legend **names the
destination** so the operator sees what they commit to"; §5's field is
`TO bc1p8rrz...s6n0vcl  0.00399 BTC`. A prefix-and-suffix elision does not name an
address uniquely to a human. Recorded here only as wording that overstates what §5
engraves — whether the truncation is *adequate* is a funds-safety question and
belongs to lens 2.

---

## §10.6 recommendation

**Reframe the unit first: the plate is the unit of loss, not the symbol. Then emit
enough parts to survive losing one plate, make `r` an input to §4's search rather
than an addition after it, and default `r = 0` for single-plate artifacts —
because for those, the correct redundancy is a second plate, not a second symbol.**

Reasoning, in the order that decides it:

**1. The spec is costing redundancy in the wrong currency.** §10.6 states the
trade as "one more symbol — often one more plate". But the events this artifact
must survive over decades do not destroy symbols, they destroy *plates*: a plate is
lost in a house move, taken in a burglary, separated from its siblings, sheared,
or left behind. §7's own hazard framing is bearer-and-decades, not
scratch-and-a-week. A part beyond `seqLen` tolerates the loss of any one **symbol**
— so to survive the loss of any one **plate** you need `r >= max symbols on any
plate`. §4's tiling routinely puts 2, 4 or 6 symbols on a plate (the "(4 up)" and
"(6 up)" rows), so "one extra symbol" buys plate-level tolerance only in the
special case where every plate carries exactly one symbol. §10.6's arithmetic is
optimistic by exactly the tiling density that §4's minimise-plates-first objective
works to maximise. These two decisions are coupled and the spec treats them as
independent.

**2. For a single-plate artifact, in-plate redundancy is nearly worthless.** If the
whole transaction is one plate, `r = 1` protects against that plate's symbol being
scratched beyond ECC — a case ECC H already covers well — and protects against
nothing that actually takes plates away, since the redundant symbol burns with the
plate. The strictly better use of the same one plate of cost is **a second,
identical plate stored somewhere else**: it survives fire, flood and theft at one
location, it needs no fountain-aware decoder, and it is what every other format in
this constellation already does. So the honest default for the 1-plate case is
`r = 0` plus an operator-facing recommendation to cut two copies — not a fountain
part. That covers the majority of §4's table.

**3. For a multi-plate artifact, redundancy is the only thing that helps, and ECC
is currently eating its budget.** Above one plate, loss of any one plate is total
loss with probability 1 under `r = 0`, and no ECC level touches that mode. Because
§4 maximises ECC at fixed plate count, the slack a fountain part would occupy is
already spent — on deeper protection against a mode that is partly covered, to buy
none against the mode that is uncovered (D-7). The allocation to change is not
"plates → redundancy", it is "**ECC above the optical requirement → redundancy**".
That is often free in plate terms: at a fixed plate count, dropping ECC H to M
frees capacity comparable to an extra part.

**4. Therefore the shape of the answer, not just the number.**

- Make `r` a **term in §4's search space and objective**, not a post-processing
  step, since it changes symbol count → tiling → plate count. My suggested order,
  offered as illustration and not as the deliverable: `minimise plates` →
  `maximise plate-loss tolerance up to 1` → `maximise ECC` → `minimise symbols` →
  the tie-breakers D-1 needs. Note that "tolerance up to 1 plate" may be cheaper
  to buy by *loosening* the tiling (fewer symbols per plate, so `r = 1` suffices)
  than by adding parts — which is a possibility the current objective cannot even
  represent.
- Size symbols for `seqNum = seqLen + r`, not `seqLen` (D-6's off-by-one trap).
- Keep the parameter explicit and recorded: `--redundancy <n>`, its value printed
  and stored wherever the manifest turns out to live (D-4), and refused rather than
  silently reduced if it would push over a plate the operator did not ask for.
  §8's rule — "Every refusal names the number that caused it" — applies.

**5. Do not close §10.6 on a number before F-234 is cut, but do close it on a
mechanism now.** The ECC-versus-`r` split depends on the optical read margin on
brushed steel, which is exactly what the test plate measures, and choosing a
number first would repeat the error §8.6's module floor exists to prevent. But the
*architecture* — `r` as a search-space dimension, tolerance measured in plates,
`r = 0` plus a duplicate plate for the single-plate case — does not depend on
F-234 and should land in v0.1. Then the test plate changes a default rather than a
design, which is the difference between one round and a rewrite of §4.

**One caveat I could not settle and am not asserting either way:** whether
fountain parts (`seqNum > seqLen`, XOR mixtures, decodable only by a
fountain-aware decoder that has collected enough other parts) are an acceptable
carrier at all under F-234's posture, given §3's stated purpose for the QR — "to
be the escape hatch for someone who has none of our tools". Every wallet that
reads multipart `ur:bytes` implements the fountain decoder, so the practical loss
looks small; but a duplicate of an existing pure part is decodable by anything and
costs the same one symbol, trading strictly weaker coverage (it tolerates losing
*that* part, not any part) for strictly stronger decodability. The spec has not
considered duplicate-parts-versus-fountain-parts as a fork at all, and it should,
because point 2 above says duplication is the right answer for the commonest case.
