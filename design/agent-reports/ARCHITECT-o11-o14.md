# ARCHITECT — O11 and O14, decided

Role: architect, routed two design decisions by the operator via the controller.
Date: 2026-08-24. Spec: `design/SPEC_engrave_transaction.md` (§2.2, §3.6/§3.6a,
§4.2a, §4.5, §6, §7, §8, §9). Every factual claim below was checked against the
tree, not quoted from the spec; citations inline.

---

## DECISION 1 — O11: the picker keys on a CARRIED txid; R14 is retired

**The decision.** The chunks form of the `tx:` record carries a **mandatory,
`mt`-computed 32-byte txid field**, defined by P1 in the wire layout (§2.1b —
which is not yet frozen, so this costs a field, not a version bump); the device
keys the picker and R10 on that carried txid, displays it **marked as carried,
never derived**, and cross-checks it against every chunk header's
`chunk_set_id`, which can only **refute**, never confirm. A new refusal (call it
R15) refuses the payload on mismatch. R14 is retired.

### Why

**The §3.6a candidate table conflates two provenance classes.** It files
"legend fields" as *asserted by the operator* — correct for the `TO` label,
which is typed free text. But a txid computed by `mt encode --record --chunks`
is **derived at origin from the very bytes being chunked, in the same
invocation, by the same tool**. Its trustworthiness is *identical* to the
chunks' own: any corruption or adversary that can falsify the field can equally
falsify the chunks beside it. §2.2 already accepts, in bold, that "a chunks
plate is cut with the device making no claim whatever about its content" — so
carrying the txid adds **zero new trust surface** over the accepted cost. It is
a third class — *derived-then-carried* — and the spec's objection to keying on
an **asserted** field does not reach it.

**The license already exists in §8's ruled table:** *"The txid is for
recognition and never claimed as proof"* (walk K). Recognition is precisely and
only what the picker does. Even on the raw form, the txid on screen is
recognition-not-proof; the chunks form now differs only in *who computed it*,
and the screen says so.

**The 20-bit binding check is real, decoder-free, and refutation-only.**
Verified in `design/SPEC_mt_v0_1.md`:

- the set id **is** the top 20 bits of the reassembled transaction's txid
  (`:943` — "the set id **is** the top 20 bits of the reassembled transaction's
  txid (§10.13 c)"; also `:3464`);
- the header prefix `version + chunk_set_id + count` is exactly 8 characters,
  `index` exactly 3 (`:907`) — readable off the string with a 32-entry charmap,
  which §3.6a's own table already concedes is not a decoder ("yes — read off
  the string").

So the device checks: every chunk in a record shares one `chunk_set_id`
(`mt`'s own rule, `SPEC_mt_v0_1.md:780`), and that id equals the carried
txid's top 20 bits. **Mismatch → R15, refuse the payload.** Match → no claim.
This respects `mt`'s refusal verbatim — *"1 in 1,048,576 by accident, and under
a second to construct deliberately"* refuses 20 bits **in the
identity/confirmation role**; here 20 bits never confirms anything. It catches
the accidental class (wrong file packed, record-assembly bug) with
p = 1 − 2⁻²⁰, and the deliberate class was already conceded: an attacker who
constructs a colliding set id controls the whole record and could as easily
write a matching txid — that attacker is inside §2.2's accepted cost with or
without this decision, and with or without R14.

**R10 becomes evaluable for chunks:** two records with the same carried txid
are the same transaction packed twice — refuse or collapse, per §3.6. Set ids
are checked only against their **own** record's txid, never across records, so
a legitimate top-20 collision between two *different* transactions still yields
two distinguishable picker rows (full txid on screen 2, §3.6's prefix rule
unchanged).

**The rejected candidates, and why:**

- **Partial decode — it is the decoder, plus a parser.** The txid is
  double-SHA256 over the **witness-stripped** serialisation (§3.6a). Reaching
  that preimage requires: charmap-decoding every data character, repacking
  5→8 bits, reassembling the *complete* chunk set in index order, then
  deserialising the transaction far enough to strip witnesses. That is the
  whole `mt1` decoder minus the checksum, plus a transaction parser applied to
  its output — strictly *more* than what §2.2 ruled out. There is no partial
  anything here; the candidate is rejected by inspection.
- **`chunk_set_id` as the key** — 20 bits in exactly the role `mt` refuses it
  for; strictly dominated by the carried txid, under which the set id still
  earns its keep in the only role where 20 bits is sound (refutation).
- **Legend fields** — asserted, collide, already rejected by §3.6a's own logic.
- **R14 standing** — honest but it forecloses the stated real use, and it buys
  no safety the carried txid does not: under R14 a single-transaction chunks
  payload whose chunks mismatch the operator's intent is *equally*
  undetectable. R14's protection was against **selection** error only, and the
  carried txid restores selection.

### What it costs, named

- §2.2's accepted-cost sentence must be reworded: the device makes **no claim
  of its own** about chunk content; it **relays `mt`'s claim, marked as
  carried**. One normative display rule lands in §3.6: screen 2 distinguishes
  *derived* (raw form) from *carried* (chunks form) — one word suffices.
- The wire layout gains a mandatory 32-byte field for the chunks form (raw form
  omits it; the device derives). Size is a non-issue against the 8 KB NFC
  buffer. It is **not** a second form of the transaction — a digest is not
  invertible — so R4′'s per-record XOR is untouched.
- P4 gains R15 and its tests (hostile input caught, nearest-legitimate input
  passing, per §5's rule).

### What it forecloses

- The layout commitment: every future chunks-form record must carry the field
  or version-bump the record. Deciding **before P1 freezes the layout** is
  exactly on time; after P1 it would be a migration.
- The absolutist reading of "the record makes no verifiable claims" is gone
  permanently.
- **Bind it now for v2:** if a later version ever adds an on-device decoder,
  derived ≠ carried is a refusal, never a preference for either value. Write
  that today so v2 does not invent a tiebreak.

### What would falsify it

- A journey walk (or field incident) showing an operator treating the carried
  txid as **content verification** — e.g. skipping the host-side `mt inspect`
  compare "because the device showed the txid". That is the carried label
  failing at the human layer, and it would argue for stronger wording or, at
  the limit, R14's return. This is checkable in P6's journey phase for the cost
  of one walk.
- P1's vector work discovering the field cannot ride the layout — no such
  constraint is visible (32 bytes vs 8 KB), so treat any appearance of one as a
  layout defect, not a reason to drop the field.

### §7 consequence

The closure line "O11 is resolved, OR R14 refuses…" is satisfied by the first
arm. Sequencing note: the carried-txid picker, R15, and R10-for-chunks are all
P4 work (the field itself is P1/P2). If P4 were ever to close without them, R14
is the already-specified interim refusal — but that is a fallback ordering, not
a hedge on this decision.

---

## DECISION 2 — O14: S0 cuts HAND-BUILT Structured Append symbols from an
independent, committed generator; P5 builds SA and must reproduce the S0
fixture as its gate

**The decision.** S0 keeps the SA pair and cuts **hand-built, standard-
conformant SA symbols** produced by an independent generator committed as
`scripts/gen-sa-fixture.py` (house style: `gen-mt1-vectors.py` already lives
there), validated **off-screen first**; Structured Append stays owned by P5,
whose gate becomes (a) the same software decoder and the same phone scanner(s)
used at S0 reassemble P5's output off a rendering, and (b) for the pinned S0
vector — same data, version, ECC, **mask** — P5's encoder reproduces the
fixture **module-for-module**, making the S0 fixture the cross-implementation
oracle.

### Why

**§7's two gates decompose the question, and only one of them needs steel.**

- **Gate 2 (physics): does the scanner+steel channel carry SA?** This is a
  property of the SA *format*, the steel, and the scanner ecosystem — not of
  the shipping encoder. Conformance to the standard (mode 0011: index, count,
  parity) is precisely what makes any two conformant encoders interchangeable
  over that channel; interchangeability is what a standard *is*. Any conformant
  symbols answer it.
- **Gate 1 (mechanism): does the shipping encoder emit conformant SA?** This is
  a property of P5's code and is machine-checkable **without steel** — decode a
  rendered image.

So the objection "the thing tested is not the thing shipped" dissolves under
the decomposition: S0 tests the channel with conformant symbols; P5's gate
proves the shipped encoder emits conformant — indeed **byte-identical** —
symbols. Byte-identity is achievable, verified: `coding.NewPlan(version, level,
mask)` takes the mask as an explicit parameter
(`kortschak-qr@v0.3.2/coding/qr.go:484`), and segno pins masks too, so the
vector is deterministic end to end. This is the cross-language-vectors pattern
that caught F-212 — the fixture stops being throwaway and becomes P5's
acceptance asset.

**The closure principle lands on gate 2, and gate 2 is the one that can
actually fail.** SA reassembly requires the *scanner* to hold state across
multiple decodes, and consumer phone apps routinely auto-complete on the first
symbol. If real scanners cannot reassemble SA off brushed steel at feasible
module sizes, that refutes F-234's ordinary-scanner promise for multi-symbol
jobs and re-opens the design (bespoke header + an `mt` reader, or re-ranking
the §4.5 objective to minimise symbols). That answer must arrive **before P4
and P5 exist**, at ~2 s per cut — not after the search, the encoder, the
computed reservation and the emission reorder are all built. Route 3 (drop the
pair) does not remove the gate — §7 still forbids cutting any multi-symbol job
until it holds — it merely schedules the gate's first execution at the most
expensive possible moment. That is the exact defect shape the closure rule
records.

**Route 1 (build SA before S0) inverts the dependency graph.** S0 resolves O1
(byte encoding), O2 (module size) and O13 (legend face) — inputs the P5 search
and encoder *consume*. Building P5's encoder first both delays every other S0
answer and builds against unknown parameters, and it puts fork code on the
critical path of a plate whose entire value is preceding code.

**The bridge into the cut path is small and verified.** `qr.Code` is a plain
public 4-field struct — `Bitmap []byte, Size, Stride, Scale`
(`kortschak-qr@v0.3.2/qr.go:84`) — constructible from any module bitmap, and
`engrave.QR(strokeWidth, scale, qr *qr.Code)` takes it
(`seedhammer/engrave/engrave.go:277`). S0 already needs a harness to place
blocks at four module sizes on one plate; a ~20-line bitmap loader is marginal
harness code, not product code. The "S0 has no code dependency" purity was
never literally true, and this does not meaningfully erode it.

### Normative details of the route

1. **Generator:** `scripts/gen-sa-fixture.py` over **segno** (supports SA via
   `make_sequence`, mask pinnable). segno is **not installed** on this box
   (verified) — one `pip install`. If the box must stay offline, say so now;
   that blocks the cheap tool, not the decision.
2. **Validate the fixture off-screen before cutting** — both with a software
   decoder and with the actual phone scanner(s), from the screen or paper.
   Otherwise a broken fixture spends the cut and, worse, mislabels a fixture
   bug as a physics failure — the "a control can test the wrong layer" lesson.
3. **Cut the pair small (v2–v5) at the module sizes bracketing the search's
   likely choice** — SA state-holding, not density, is what is under test; the
   single-symbol blocks already probe density.
4. **Record which scanner apps reassembled.** Gate 2's answer is
   per-ecosystem, and §4.3's operator instruction ("test the whole set after
   plate n") depends on naming a scanner that actually can.
5. **P5's gate pins geometry too** — module size, stroke width, quiet zone —
   to S0's chosen values. The equivalence argument rides on the module bitmap
   being the entire optical content *at fixed geometry*; pin the geometry and
   the argument is airtight.
6. **§4.3's already-mandated per-plate scan covers the first real P5 job** on
   steel — no new machinery needed for the residual.

### What it costs, named

- S0 gains ~20 lines of harness and one committed generator script — and the
  script must be maintained (reproduction paths decay silently; committing it
  is the mitigation, not the burden).
- One external tool install (segno) on the workstation.
- P5's gate gains a byte-identity vector, which pins mask and segmentation
  **for that vector only** — production jobs remain free.

### What it forecloses

- P5 may not choose an SA header layout, segmentation or mask policy that
  cannot reproduce the S0 vector — which is to say it may not be
  non-conformant; that is foreclosed deliberately.
- If S0's gate 2 FAILS (scanners will not reassemble off steel), the SA ruling
  itself (§8, operator 2026-08-24) comes back to the operator — this decision
  schedules that confrontation early; it does not avoid it.

### What would falsify it

- **P5 output that decodes off-screen but fails on steel where the hand-built
  symbols succeeded.** Given pinned geometry the module bitmap is the entire
  optical content, so this observation would mean the geometry pin was not
  honoured — treat it as a gate defect (pin violated), and tighten the gate,
  not the route.
- A finding that segno's SA output is itself non-conformant (decoders
  disagreeing between segno and a second independent decoder at the off-screen
  validation step). That is exactly what step 2 exists to catch before any
  steel is spent; its cost is minutes.

---

## Summary of spec edits these decisions imply

| § | edit |
| --- | --- |
| §2.1b/P1 | chunks form carries a mandatory 32-byte txid field; layout states it; raw form omits it |
| §2.2 | accepted-cost sentence reworded: no claim of its own; relays `mt`'s, marked carried |
| §3.6/§3.6a | picker keys on carried txid; display distinguishes derived vs carried; O11 closed |
| §5 | R14 retired; R15 added (set-id vs carried-txid binding, refutation-only); R10 evaluable for chunks |
| §6 S0 | SA pair = hand-built fixture from committed generator; off-screen validation precedes the cut |
| §6 P5 | gate gains fixture reproduction (byte-identical, pinned version/level/mask/geometry) + same-scanner decode-off-rendering |
| §7 | "O11 resolved" arm satisfied; SA gates unchanged in force, now executable in order |
| §9 | O11 closed → this report; O14 closed → this report; v2 rule recorded: derived ≠ carried is a refusal |
