# Round 4 — Spec-coverage sweep, `SPEC_engrave_transaction.md`

**Lens:** completeness only — is anything RULED but UNBUILT, or BUILT but UNRULED?
Not a correctness pass (four rounds already ran that). All findings below were
checked against the document as it stands (1,615 lines) using `grep -n` for the
literal anchors (`NORMATIVE`, `MUST`, section numbers), not by re-reading prose
and trusting memory.

**Already machine-verified, not re-derived (per brief):** all 16 refusals appear
in §5 with no numbering gaps; all 14 open items have a non-empty owner cell;
every `§n.n` reference in §6 resolves to a real section; zero placeholders/TBDs;
zero dangling internal references. §9 O8's stale "next walk" owner (Journey B
was skipped 2026-08-24) is known and not re-reported here.

---

## 1. Rulings without a section

Checked all 30 rows of §8's table (lines 1543–1572) against the body. **29 of 30
are carried by an identifiable section.** One is not:

- **"Overwriting the region is intended — it is a courier, not a vault" (walk H,
  §8 line 1558).** `grep -in "courier\|overwrit"` over the whole document returns
  exactly two hits: the §8 ruling itself, and **§9's O10**, which reads *"the
  courier model is nowhere written down (§ walk H) | documentation"* (line 1593).
  No section in §§1–7 states or explains this ruling. It is not a silent hole —
  the document already flags its own gap via O10 — but the ruling→section mapping
  is not total for this row. **Severity: Minor** (self-tracked, not invisible;
  and the ruling is permissive — "you need not protect against overwrite" — so it
  gates no phase's deliverable, unlike a missing wire format).

One further row is process, not design, and is intentionally not counted as a
gap: **"The journey walk is the review of this spec" (operator, §8 line 1551)**
is carried by the document's own front-matter (the review table and narrative at
lines 6–44), not a numbered §, which is the right place for a meta-statement
about how the spec was reviewed.

## 2. Obligations without an owning phase

`grep -n "NORMATIVE"` returns 18 hits; `grep -n "MUST\b"` returns 9 more not
already covered by a NORMATIVE tag. Cross-checked each against §6's six phase
rows (S0, P1–P6, lines 1497–1503). **16 of 18 NORMATIVE items and all 9 MUST
items trace to an explicit phase citation** (either by section-number citation in
the phase's `what` cell, or by a named mechanism the phase cell names verbatim —
e.g. "ALL FOUR lockstep sites", "R15's carried-txid cross-check"). Two do not:

- **§2.2a's NORMATIVE, line 300: "the chunks path engraves TEXT ONLY … a
  text-only plate builder."** §6's P5 row is the only phase that owns "the
  plate," and its full text (line 1502) is: search w/ 16-symbol cap, QR
  Structured Append over `coding`, computed legend reservation, legend-emission
  reorder, test-the-plate, plate count — all QR/raw-form concerns. Nothing in P5
  (or any other row) says "text plate builder," "chunks plate," or cites §2.2a.
  This is a **new deliverable** — a chunks-form plate needs different code than a
  raw-form plate does, and §2.2a is explicit that it is "not nothing." No phase
  row claims it. **Same shape as the O11-no-phase-row precedent this lens was
  briefed against.**
- **§3.2's NORMATIVE, line 557: "REPLACEMENT, not an addition"** — remove the
  `me sysw pack` line from the compare screen, put `me sysw show` beneath the
  digest. `grep -n "§3\.2\b"` finds this section cited three times total, all in
  §1/§1.2's pipeline description (lines 20, 93, 145) — **never in §6.** P4's row
  explicitly ranges "the program (§3.4–3.7)," which by its own stated bounds
  excludes §3.2 and §3.3's sibling compare screen. §3.3 (the payload menu) *is*
  separately named in P4's row; §3.2 (the compare screen fix) is not named
  anywhere.

Lower-confidence, listed for completeness (**Minor**, not counted in the
Important tally below): **R4/§2.1b's "both forms in one record" refusal**
(NORMATIVE at line 1450) has no phase explicitly cited as implementer — P1's row
names the record format and "with vectors" but not this specific validation.
Plausibly folds into P1's general record-definition scope, unlike the two items
above which name entirely new, uncited deliverables.

## 3. Close conditions nothing produces

§7 has 11 distinct bullets (lines 1512–1537). **8 trace cleanly** to a phase or
gate named elsewhere (S0's test plate; §4.2c's two SA gates → S0 + P5; legend
reservation → P5; legend emission order → P5; `check-provenance.sh` — pre-existing
repo tooling, not a build item, so "green" is the right verb and no phase need
build it; refusal coverage → P6; carried txid + R15 + R14 retirement → P1 + P4).
Three do not:

- **"The mode-segmentation gate is green. Any QR sizing MUST assert measured v40
  capacity against numeric 7089 / alnum 4296 / byte 2953 at L" (lines
  1514–1515).** `grep -n "mode-segmentation\|7089\|4296\|2953"` returns **exactly
  these two lines and nothing else in the entire 1,615-line document.** No
  section defines what the "mode-segmentation gate" is, no phase in §6 owns
  building or running it, and the numbers are never explained or connected to
  anything the design actually does. (§4.2 rules that the QR always carries raw
  transaction **bytes**, byte-mode only — never numeric or alphanumeric segments
  — so it is not obvious why a numeric/alnum/byte mode-segmentation capacity
  check would even apply to this design; it reads like an orphaned holdover,
  possibly from `SPEC_mt_qr_DEFERRED.md`'s different, codex32-text design.) A
  gate with no definition and no owner cannot be run, let alone closed.
- **"All THREE program-keyed lockstep sites carry the new program" (§3.1a) —
  line 1534.** §3.1a itself documents **FOUR** sites: the original three
  (`uiFlow`'s dispatch, `StartScreen.draw`, `layoutMainPlates`) plus, "found
  R0 round 2, I4," a fourth — `engraveObjectFlow`'s type switch — with the text
  at line 514 stating explicitly "**NORMATIVE: all four are lockstep sites**."
  §6's own P4 row (line 1501) correctly says "**ALL FOUR** lockstep sites." Only
  §7's close condition, and the §3.1a sub-heading itself (line 457, "THREE
  program-keyed sites"), were never updated after round 2 found the fourth. As
  written, §7's gate is **satisfiable by fixing 3 of 4 sites** — and the
  document itself calls the fourth "**the silent one**, and it is the program's
  front door" (line 502), the worst of the four to leave unfixed. A close
  condition that can pass while the worst defect in the section is still open is
  exactly the failure shape §3.3's own commentary warns about (line 594–597: "a
  gate that can pass while its own sentence is false is the shape the closure
  rule exists to catch") — here it recurs one section over, unnoticed.
- **"Both pipeline invariants are asserted as pipeline properties (§1.1)"**
  (line 1537). §1.1 names two invariants (lines 129–132): (1) `mt` puts nothing
  on stdout on any failure path, (2) `me sysw pack` refuses empty stdin.
  Invariant 2 is **R7** in §5's table, explicitly owned via §1.1 and covered by
  P6's "refusal coverage." Invariant 1 is **not a refusal** (nothing to refuse —
  it is a property of `mt`'s existing, "measured today" behavior) and has **no
  R-number, no §6 phase citation, and no other named test mechanism** anywhere
  in the document. It plausibly falls under P2's general `mt` scope by
  proximity, but unlike invariant 2 it is never named the way the other
  half is.

## 4. Open items that are stale or already resolved

Checked all 14 rows of §9 (O1–O14; already-known O8 excluded per brief). One
additional candidate, offered with lower confidence than O8's (which is a plain
date contradiction):

- **O6 — "multi-symbol recovery without `mt`'s reader," owner
  `SPEC_mt_qr_DEFERRED.md:169`** (an external document). §4.2a's own text (lines
  871–873), ruled the same day as this open item's context, states: *"it keeps
  F-234's promise **intact for multi-symbol jobs** — a recoverer with an
  ordinary scanner still gets the transaction, with no constellation
  knowledge — which no bespoke header could do."* That is the SA ruling
  directly answering "can you recover a multi-symbol job without `mt`'s
  reader" — yes, a standard QR decoder reassembles it, `mt` is not required to
  get the raw bytes back. §9 still lists O6 open, owned by a different,
  external spec, with no cross-reference to §4.2a's own resolution. It is
  possible O6 asks a narrower question this excerpt doesn't fully settle (e.g.
  recovery of the **chunks** form specifically, which genuinely has no
  non-`mt` reader) — the table row is one line and does not say which. Flagged
  for reconciliation rather than asserted resolved. **Severity: Minor.**

No other open item shows date-of-ruling contradictions, resolved-but-still-open
status, or an owner that stopped existing.

---

## Coverage tables

### Table 1 — §8 rulings → section (30 rows)

| # | ruling (short) | source | section | covered? |
|---|---|---|---|---|
| 1 | QR carries standard form only | F-234 | §4.2 | Y |
| 2 | comprehend before cut | brainstorm | §3.4 | Y |
| 3 | plate default QR+legend | operator | §4.1 | Y |
| 4 | raw XOR chunks | operator | §2.2 | Y |
| 5 | mt emits / me packs | operator | §1 | Y |
| 6 | no new secrecy class | operator | §2.1 | Y |
| 7 | MaxSectionLen 32,734 / NFC 8191 | operator | §2.3 | Y |
| 8 | byte encoding stays a parameter | F-243 | §4.2b | Y |
| 9 | journey walk is the review | operator | front-matter, not a §-body item | Y (not a hole) |
| 10 | me sysw pack gains stdin | walk A | §1.1 | Y |
| 11 | tx: on argv refused | walk B | §2.1 | Y |
| 12 | no --record default | walk C | §2.2 | Y |
| 13 | chunks verbatim, no decoder | walk C | §2.2 | Y |
| 14 | world-readable refused+override | walk E | §2.5 | Y |
| 15 | sealing by content | walk F | §2.4 | Y |
| 16 | courier, not vault | walk H | **none** (only §8, §9 O10) | **N — Minor finding** |
| 17 | me sysw show under digest | walk I | §3.2 | Y |
| 18 | txid for recognition only | walk K | §3.5 | Y |
| 19 | show total, allow skip | walk L | §3.5 | Y |
| 20 | total never a destination | walk M | §3.5 | Y |
| 21 | device says test, never tests | walk N | §4.3 / §3.7 | Y |
| 22 | mt inspect gains raw-tx subject | walk O | §1 owner table, §6 P2 | Y |
| 23 | carousel payload-independent | walk P | §3.1 | Y |
| 24 | payload menu after boot load | walk P | §3.3 | Y |
| 25 | picker keyed on txid | walk Q | §3.6 | Y |
| 26 | legend last, no resume | walk R | §4.4 | Y |
| 27 | text+QR never for a transaction | operator 08-24 | §2.2 | Y |
| 28 | multi-symbol uses SA | operator 08-24 | §4.2a | Y |
| 29 | legend packed/computed, 3.0mm floor | operator 08-24 | §4.5a | Y |
| 30 | symbols outrank ECC, ECC floor M | operator 08-24 | §4.5b | Y |

### Table 2 — NORMATIVE/MUST obligations → phase (27 anchors)

| line | obligation (short) | owning phase | explicit? |
|---|---|---|---|
| 201 | tx: branch beside PassPrefix | P3 | Y |
| 226 | P1 defines record layout | P1 | Y |
| **300** | **chunks path: text-only plate builder** | **none** | **N — Important** |
| 393 | sealing stderr message every time | P1 (content-based sealing) | implicit |
| 498 | P4 owns txScan case | P4 | Y |
| 514 | all four lockstep sites | P4 | Y |
| **557** | **compare screen: replace, don't add** | **none** | **N — Important** |
| 590 | payload menu + boot-path call | P4 | Y |
| 629 | address row states network params | P4 (§3.4–3.7) | Y |
| 652/655 | total never a destination label | P4 (§3.4–3.7) | Y |
| 658 | txid never claimed as proof | P4 (§3.4–3.7) | Y |
| 732 | P4 owns chunk_set_id extraction | P4 | Y |
| 734 | R15 refutes, never confirms | P4 | Y |
| 813 | R16 empty-config refusal | P4 | Y |
| 891/920 | search: 16-symbol hard bound | P5 | Y |
| 967 | SA over `coding` package | P5 | Y |
| 1027 | S0 cuts SA pair; P5 gate two halves | S0 + P5 | Y |
| 1128 | per-plate instruction = f(plate) | P5 ("test-the-plate", plausible) | implicit |
| 1139/1151 | scan-order irrelevant / no partial mt inspect | P2 (raw-tx subject covers the reader) | Y (reasoned, see body) |
| 1192 | legend-emission reorder | P5 | Y |
| 1326 | reservation computed per plate | P5 | Y |
| **1450** | **R4′: XOR is per-transaction** | **none named** | **N — Minor** |
| 1470 | R11′ two distinct messages | P4 (§3.3) | Y |

### Table 3 — §7 close conditions → producer (11 bullets)

| condition | producer | reachable? |
|---|---|---|
| 0C/0I over enumerated lenses | the R0 process itself | Y |
| **mode-segmentation gate green, v40 7089/4296/2953** | **none — undefined anywhere else** | **N — Important** |
| test plate cut and read | S0 | Y |
| §4.2c's two SA gates | S0 (physics) + P5 (software) | Y |
| legend reservation computed | P5 | Y |
| P5 gate asserts emission order | P5 | Y |
| check-provenance.sh green | pre-existing tooling | Y (not a build item) |
| refusal coverage bijection | P6 | Y |
| carried txid + R15 + R14 retired | P1 + P4 | Y |
| **all THREE lockstep sites** | P4 — but P4/§3.1a's own body say FOUR | **N — Important (stale count)** |
| **both pipeline invariants asserted** | invariant 2→R7→P1/P6; invariant 1→**none named** | **partial — Important** |

### Table 4 — §9 open items (14 rows)

| # | status | note |
|---|---|---|
| O1–O5, O7, O9, O10, O13 | open, consistent | owners check out against body text |
| O6 | open | **possibly resolved by §4.2a; not cross-referenced — Minor** |
| O8 | open | **stale (already known — Journey B skipped 2026-08-24), not re-reported** |
| O11, O12, O14 | resolved, correctly marked closed | cross-references to §3.6b/§4.2c/O14 check out |

---

## Verdict

**Counts (this round only):**

- Rulings without a section: **0 Important, 1 Minor**
- Obligations without an owning phase: **2 Important, 1 Minor**
- Close conditions nothing produces: **3 Important**
- Open items stale/resolved (beyond the known O8): **0 Important, 1 Minor**

**Total: 5 Important, 3 Minor.**

One line per Important:

1. §2.2a (line 300): the chunks-form "text-only plate builder" is a named,
   non-trivial NORMATIVE deliverable with no phase in §6 that owns building it.
2. §3.2 (line 557): the compare-screen fix (replace the `me sysw pack` pointer
   with `me sysw show`) is NORMATIVE and appears nowhere in §6's phase table.
3. §7 (lines 1514–1515): the "mode-segmentation gate" and its v40
   7089/4296/2953 numbers are defined nowhere else in the document and owned by
   no phase — an orphaned close condition.
4. §7 (line 1534): the close condition says "THREE" lockstep sites; §3.1a's own
   body and §6's P4 row both correctly say FOUR — the gate as worded is
   satisfiable while leaving `engraveObjectFlow`, the document's own "silent…
   front door" defect, unfixed.
5. §7 (line 1537): of the two pipeline invariants (§1.1) the close condition
   requires "both" asserted, only one (R7, empty-stdin refusal) has a named
   phase/test path; the other (`mt`'s clean stdout on failure) has none.

**What this round did NOT examine:** correctness of any claim, number, or
citation (four prior rounds' job); wording/style/length; whether any §8 ruling
was the *right* call; anything in `design/agent-reports/` (record, not
artifact); and it did not re-verify the four facts the brief marked as already
machine-checked (16/16 refusals, 14/14 owners, all `§n.n` links, zero
placeholders) — those were taken as given per the brief.
