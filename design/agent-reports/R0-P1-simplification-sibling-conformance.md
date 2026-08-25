# Sibling conformance and simplification — P1 `tx:` container vs the shipped md1/mk1 path

**Lens:** not correctness — eight rounds of that have run and none asked this
question (measured: `grep -i 'simplif|proportion'` over
`R0-P1-plan-round0..7.md` returns nothing). The question here is scope: what
does the `tx:` plan build that the `md1`/`mk1` path, shipping through the same
container, never needed — and which requirements, if relaxed, buy the most
simplification per unit of risk.

**The comparator arithmetic, restated once.**
`IMPLEMENTATION_PLAN_systemwide_payloads.md` (611 lines, 32.5 KB) built the
*entire* container — wire format, sealing, crypto, classify/split, CLI, Go
port, device plumbing, eight programs' wiring, 13 stages. Its Stage 7 — the
md/mk decode-confirmation walk, the direct ancestor of everything `tx:` needs
at the set level — is **48 lines**: one signature, one posture sentence, one
vector, one green command. It shipped as `mdmk_unconfirmed()`,
`crates/me-cli/src/sysw/record.rs:99-170`, ~54 lines of implementation.
`IMPLEMENTATION_PLAN_P1_me_container.md` v10 is **2,007 lines / 163 KB** for
one more record type in that same, already-shipped container, host side only
(device work is P3/P4/P5). Five times the comparator's bytes; forty times its
Stage 7.

---

## Part 1 — the divergences

| # | dimension | md1/mk1 (shipped) | `tx:` (planned) | class | evidence |
| --- | --- | --- | --- | --- | --- |
| D1 | record form | bare bech32 records, no prefix, no framing | CHUNKS: bare `mt1` records (operator ruling 2026-08-24) **plus** a 75-byte framed `tx:` metadata record; RAW: one framed record | **FORCED** (mostly) | the metadata record is forced by device needs: the device cannot derive a txid for chunks (spec §3.6a) and the picker is keyed on the txid (walk Q, spec §8); the legend (`TO`, fee — spec §3.4 asserted column) must ride somewhere. RAW needs a byte carrier; bare hex would be sniffable free text (spec §2.1a). What is *not* forced is the metadata record's breadth — see D3, D4 |
| D2 | failure posture, host | **REPORT**: incomplete/non-decoding set marks members unconfirmed, warns, packs anyway (`record.rs:110` doc: "REPORTS instead of refusing"; spec §13 D6); device flags unconfirmed as SECRET at load | **REFUSE**: whole pack aborts, exit 4 (plan §1.5, §2.3) | **DELIBERATE — but justified by an argument that is wrong about the sibling, while the correct justification goes unstated** | Plan §2.3 argues refuse because tx: is "the one record class whose failure mode is secret bytes riding in cleartext, which is the whole reason EPD §6.3 exists". False as a distinction: that is *exactly* `MdMk`'s failure mode, and EPD §6.3 was written for md/mk — where the answer is report-and-treat-as-secret. The **real** difference in the problem: an `md1` card set that fails decode may be the operator's only copy of a backup (refusing loses data); a `tx:` payload that fails decode is regenerable by re-running `mt encode`. Refusal is cheap for `tx:` and expensive for md/mk. That is a sound reason to diverge, and the plan never states it. Note the device side does NOT diverge: `SPEC_systemwide_payloads` §5.3.2 stays flag-at-load (plan §7 row 4 — "refuse when writing, flag when reading") |
| D3 | carried identity | **none** — the record is self-describing, decode is the proof | 32-byte txid + 32-byte wtxid, display order, on **both** forms | txid-on-chunks **FORCED** (spec §3.6b, decided O11 — device can't derive it). wtxid **DELIBERATE** (plan-added r2-C1, then folded into spec §2.1). txid+wtxid **on the RAW form** is neither forced nor examined: for RAW the device derives the txid itself (spec §3.4 derived column) and `me` holds the bytes, so both carried identifiers on RAW are pure encoder-consistency checks against values the same producer computed from the same bytes | the device never displays a wtxid on either form (spec §3.4 lists none). E17's own "accepted cost" paragraph concedes an internally-consistent stripped record is undetectable by any field. What the carried pair buys is detection of an *inconsistent* encoder — an honest-bug/interop class — plus a second operator comparand in `me sysw show` |
| D4 | metadata encoding | n/a (no metadata record) | full TLV: `n_fields`, tag/len/value, ascending-order rule, duplicate rule, unknown-tag-refused, per-tag width rules | **DELIBERATE, and self-defeating** | TLV's one benefit over fixed fields is that unknown tags can be skipped by old readers. **E8 refuses unknown tags.** The plan pays TLV's entire divergence surface (9 of the 20 E-rules, ~11 of 30 vectors — see Part 3.1) and explicitly forfeits the only thing TLV buys. Three fields exist, all known, one producer (`mt`, P2, unbuilt), two consumers (`me`, device), a version byte available for any future change. No sibling anywhere in the constellation uses TLV; the container header itself is fixed-layout |
| D5 | failure vocabulary | none — `mdmk_unconfirmed` returns `Vec<usize>`; one warn line: "record 3: an md1/mk1 this tool could not decode; the device will treat it as a SECRET" | 26 normative rule names (19 per-record + 2 chunk + 5 set), a three-hand cross-language string contract (generator emits, `TxRecordError` maps, Go compares), three error channels (`TxRecordError`, `MtChunkError`, set-level `SyswError`), two W8 templates | **DELIBERATE, grown round-by-round** (r2-C2 → r3-C1/C2 → r4-C1 → r6-I4 → r7-C1/C2) | each individual finding was real (a channel must exist; a message must be true about the record it names). The *aggregate* — names as a cross-language wire contract — has no sibling precedent: md/mk Rust↔Go conformance is behavioural (same indices unconfirmed over the same shared vector file), not string-keyed. And the apparatus now generates its own findings — see Part 3.2 |
| D6 | set completeness | not separately checked — group by `(hrp, chunk_set_id)`, feed the group to the real decoder, report on failure. Collision of two sets in one 20-bit id ⇒ merged group ⇒ decode fails ⇒ reported. 54 lines total | E20 taxonomy (orphan / missing / duplicate, each read off headers, each named), R17 as a distinct named refusal, W10 + W15 as separate passes, V25 (three arms) + V27 + V28 | **partly FORCED, partly DELIBERATE** | forced: `mt1` headers make orphan/missing readable pre-decode, and the orphan case has a distinct remedy (r7-M1: "packed the chunks alone" must not read as "delete 202 records"). Deliberate: the split of E20-shape from W15-contents, and separate names for missing vs duplicate, where the sibling collapses everything into "the group does not decode". R17's *named* refusal is new; the sibling handles the same collision by merged-group decode failure |
| D7 | canonicality (case/whitespace) | **not enforced on the sysw path** — measured by the plan itself (§2.5b): a trailing space packs verbatim, exit 0. A live defect, filed F-245 | E13 refuses non-canonical chunk records | **the sibling is ACCIDENTALLY wrong; `tx:` is right** | divergence resolves by fixing F-245, not by relaxing E13. E19 (zero BCH corrections) has a true sibling precedent — `seal`'s "not pristine" refusal for `mk1` (`seal/record.rs:137-142`) |
| D8 | vectors | **one** container vector (S-J) + unit tests against real cards | 30 vectors, near-miss pairs, an independent generator, a new fixture + loader | **mostly FORCED, scaled by D4/D5** | md1/mk1's wire formats are pinned in their own crates; `me` only had to pin *classification*. P1 invents a wire format, and Rust-primary means it must be pinned here with vectors (spec §2.1b). The count, not the existence, is the discretionary part: ~8 of 30 exist only for TLV rules, and the fixture's rule-name arm exists only for D5 |
| D9 | plan spec-density | Stage 7: signature + posture + vector + green command, 48 lines | E13/E19/E20/W5/W10/W15/§2.5a/V20-V28 span several hundred lines | **ACCIDENTAL** | the plan's only references to the sibling machinery are `report_unconfirmed` (line 552, cited to *reject* its posture) and `mdmk_unconfirmed` as a JSON field name in §3.3's schema listing (line 1287). It is never cited as implementation precedent, although W10+W15 is `mdmk_unconfirmed` with `mt_codec::decode` in the decoder seat and refuse in place of report — structurally the same walk, minus the `uniq`-key subtlety (every `mt1` record carries a chunk header, so the non-chunked arm vanishes) |

---

## Part 2 — what can be simplified without relaxing any requirement

Pure waste: nothing below changes a rule, a refusal, a vector's meaning, or any
operator ruling.

**2.1 Evict the archaeology (~500–600 lines, 25–30% of the document).**
Measured: **338 lines** of the 2,007 carry a finding marker (`(C1)`,
`(r5-I4)`…), a superseded-version reference (`v1`…`v9`), or retraction language
(`retracted/superseded/struck/was wrong/used to`) — and those markers sit
inside blockquotes whose whole content is fold history ("what v1 got wrong",
"what the v2 rewrite broke", the r5-C1 narrative in §3.3, §6.3's running-tally
table, the header's round ledger). Every one of those narratives already exists
**verbatim** in `design/agent-reports/R0-P1-plan-round0..7.md` — persisted
before each fold, by the repo's own standing rule, precisely so the plan does
not have to be its own history. `git show`/`git diff report..fold` is the
constellation's stated mechanism for "what was found and what changed"; the
plan re-narrating it is the same defect shape as the persist-commit-message
finding of 2026-08-23 (21% of report volume re-narrated), one artifact over. A
v11 rewritten in present-tense normative voice, with a ~20-line
finding→section index replacing the inline narratives, drops ~500–600 lines
and — the larger saving — shrinks every future fold's sweep surface, since most
of the 44 fold-sweep terms police stale restatements of history that would no
longer exist. Keep: the *reasons* for current rules (why display order, why
wtxid, why bare records). Evict: what previous versions said.

**2.2 Cite `mdmk_unconfirmed` as the template for W10/W15 and specify them
Stage-7-style (~80–120 lines).** The shipped function already does grouping by
chunk-set identity, real-decoder arbitration, fail-closed ungroupables, and
per-caller-index reporting, with the anti-smuggling `uniq` reasoning in its
comments. W10+W15's normative content compresses to: one signature, "groups by
`chunk_set_id` as `mdmk_unconfirmed` groups by `(hrp, chunk_set_id)`; the
arbiter is `mt_codec::decode` then `bitcoin::consensus::deserialize` then the
identifier comparison; REFUSES where the sibling reports (reason: a `tx:`
payload is regenerable, an md1 card may be the only copy)" — plus the vectors.
That is also the correct place to state D2's real justification, currently
absent.

**2.3 Move the two committed-check blocks out of the plan into scripts (~110
lines, and it upgrades the checks).** §6.1 carries (a) the 44-term fold-sweep
list (~53 lines) and (b) the block of verification commands for the 17
citations the cite-gate cannot reach (~55 lines). The repo's own rule: "when an
artifact will be folded repeatedly, commit the extractor as a script so the
check is a command rather than a discipline." Both blocks are exactly that —
`scripts/p1-fold-sweep-terms.txt` and `scripts/p1-uncited-facts-check.sh` —
and the term list has already generated three findings *about itself* (r3-I6,
r5-M3, and the self-hit incident its own cell records). In a script, a term
list cannot self-hit prose.

**2.4 Delete §6.3's running-tally blockquote (~25 lines).** A table counting
the plan's own miscounts, which by its own admission "had itself become one of
the entries it counts". Meta-meta; the reports hold it.

**2.5 Tombstone W6/W7 in one line each (~15 lines).** "A replacement that does
not retract is an alternative" requires the tombstone to exist, not to carry
its full history — that lives in `R0-P1-plan-round4.md`.

Total Part 2: **~700–850 lines removed, zero requirements touched.** This
alone brings the plan to ~1,200–1,300 lines.

---

## Part 3 — THE RANKED LIST: requirements whose relaxation buys the most

Ranked by simplification bought per unit of risk accepted.

### 3.1 RELAX: "legend fields are extensible TLVs" → three fixed optional slots behind a presence byte

**The requirement.** Plan §1 layout (`n_fields` + tag/len/value TLVs) and the
rules it drags in. Spec §2.1b explicitly delegates this: P1 must state "how the
optional legend fields are delimited" — the spec does not mandate TLV, and
spec §8 carries no ruling on the encoding.

**What gets deleted.** Replace `n_fields + TLVs` with one flags byte (bit0
fee present, bit1 fingerprint present, bit2 label present) followed by fixed
`fee: u64 BE`, `fp: [u8;4]`, `label: u8 len + 1..=64 bytes`, in that fixed
order. Deleted outright: **E1** (no order to rule), **E2** (duplicates
inexpressible), **E8** (unknown tags inexpressible), **E10** (no `n_fields`),
**E16** (widths structural), and E6/E7 collapse into the flags byte (absent =
bit clear; no second spelling of nothing exists). E4's Σ-arithmetic becomes a
constant-plus-label-len equation. Rules **20 → ~13**; rule names **26 → 20**;
vectors **V6, V9, V14, V16, V17, V17b gone and V12 halved — 30 → ~24**, with
their near-miss arms; §1.2 and roughly nine rows of §1.3 removed; step 8's
list shrinks. **~150–200 plan lines**, and the same surface again in the Go
port (P3) and the device's legend read (P4) — this cut pays three times,
because every TLV rule is a way two implementations diverge, which is the
plan's own definition of the E-rules.

**The risk accepted.** A fourth legend field later requires a version-byte
bump instead of a new tag. That is the entire risk — and it is smaller than it
looks, because **E8 already refuses unknown tags, so the current design also
requires a coordinated upgrade for any new field.** The plan pays for
extensibility it has explicitly disabled. Single producer (`mt`, P2 —
unbuilt), no shipped code anywhere, `mt-codec` unpublished: sunk cost is zero.

**Who must agree.** The plan author + one re-review round (it reverses no
review finding — no round asked for TLV; v1 introduced it and the rounds only
hardened it). Operator sign-off as a courtesy since P2/P3/P4 transcribe the
layout; no §8 ruling is touched.

**Recommendation: TAKE IT.** Highest simplification-per-risk in the plan.

### 3.2 RELAX: "the rule name is a three-hand cross-language wire contract" → rule identity is Rust-internal; the fixture pins refuse/pass; messages must be true, not vocabulary-keyed

**The requirement.** §2.5a.1's 19-name table + §2.5a.2's 5-name table + the
`MtChunkError` channel + W14's flip-the-name-and-go-RED assertion + the
"generator emits it, `TxRecordError` maps it, the Go port compares it"
contract.

**What gets deleted.** §2.5a.1 and §2.5a.2 as normative tables (~90 lines);
the `MtChunkError` type and its third `SyswError` variant (a non-canonical or
damaged chunk refuses with "record {i} is not a canonical, pristine `mt1`
chunk" — index only, true about the record, never calling it a `tx:` record,
so r3-C2's core is preserved); the fixture's `refuse.rule` string as a
cross-language assertion (keep it as documentation if desired, asserted only
by the Rust loader against the Rust enum); W8 shrinks to two short templates.
Keep, verbatim: the two failures whose remedies are genuinely distinct and
non-obvious — the R17 collision message ("pack as separate payloads", both
txids in full — spec-required) and W15's not-a-transaction message (the
smuggling case). `TxRecordError` itself **stays** as an internal Rust enum, so
Rust tests still assert the variant, keeping "refused for the right reason"
machine-checked where it is cheapest. **~150 lines**, plus the entire defect
class that produced **r6-I4, r7-C1, r7-C2, r7-I3 and r7-I5 — including both
of round 7's Criticals**, all of which are internal inconsistencies of the
vocabulary apparatus, not of the format. The machinery built to close r2-C2
is now the plan's main generator of review findings; that is the signature of
apparatus past its budget.

**The risk accepted.** A Go port could refuse a negative vector for the wrong
reason and pass the fixture. This is narrower than it sounds: the plan already
requires every negative vector to be constructed so that **exactly one rule
can refuse it** (the delete-the-check-and-it-goes-green discipline, stated
normatively on V8, V15, V27, V28). Under that construction, refuse/pass over
the full vector set pins behaviour to the rule with high precision; the
residue is a Go bug that spuriously refuses one negative vector while leaving
every positive vector green — real, small, and the class the positive/near-miss
pairs exist to shrink.

**Who must agree.** Plan author + re-review. The underlying findings stay
honoured (a channel exists; every message is true about its record; §1.5's
"index and the rule" softens to "index and a true, actionable reason", which
is a plan-level sentence). No spec edit: spec §5 requires naming what runs
*before* a refusal and R17's two txids — neither requires a rule-name
vocabulary.

**Recommendation: TAKE IT**, or at minimum the halfway house (names stay in
the fixture, asserted only Rust-side; the Go port asserts refuse/pass).

### 3.3 RELAX: E20's enumerated shape taxonomy → sibling-shaped "the group does not decode", keeping the orphan check and the missing-index report

**The requirement.** Separate named failures for missing vs duplicate, split
from W15's contents pass.

**What gets deleted.** `chunk_missing` vs `chunk_duplicate` as distinct
species (both become "set for txid X does not reassemble: have indices …,
header declares count N" — the header makes the have/want line nearly free, so
keep it); V25 collapses from three arms to two (orphan; incomplete-set with
have/want detail); W10's prose collapses into 2.2's Stage-7-shaped spec.
**~40–60 lines.** Keep the orphan check and r7-M1's whole-set summarisation —
the packed-chunks-alone first mistake is real and its wrong-obvious-remedy is
worse than silence, which is the journey-walk bar.

**The risk accepted.** Essentially none — the same inputs are refused at the
same site; only the internal species count drops. R15's amended spec row
(orphan + incomplete refusals) remains satisfied.

**Who must agree.** Plan author + re-review only.

**Recommendation: TAKE IT** (it is half a Part 2 item; listed here because it
deletes two of the five set-level names, which interacts with 3.2).

### 3.4 RELAX: "both identifiers are carried on both forms" → drop the wtxid field (and/or carried identifiers on RAW)

**The requirement.** §1.1a, E17, the 75-byte framing.

**What gets deleted.** §1.1a (~90 lines with its measurement blocks), E17,
V18/V26, the wtxid halves of V4a/V4b and W9, 32 bytes of framing (75 → 43),
plus §3.1's ceilings reverting to the v3 numbers; spec §2.1's amended table
and two of §6.3's five stale-statement corrections shrink. **~200–250 lines**
— the largest single-line-count cut on the list.

**The risk accepted.** The one thing E17 detects — an encoder that strips
witnesses while carrying honest identifiers (the honest-bug/interop class) —
goes undetected at pack time. Mitigations that remain: the cross-language byte
vectors pin the exact body bytes (a Go encoder that stripped witnesses fails
V1's byte comparison), and `me` can *compute* and print both identifiers from
the body/reassembly for the operator's `mt inspect` comparison, which is the
real gate in both worlds — the plan's own accepted-cost paragraph concedes a
consistent stripped record is undetectable by any carried field. But: this
reverses a review **Critical** (r2-C1) whose fold is complete and measured;
the ongoing cost of keeping it is 32 bytes and one `==`; the deletion cost is
re-editing a settled layout, re-amending the spec, and a re-review round.

**Who must agree.** The operator (spec §2.1's normative table now carries the
wtxid) + a re-review that explicitly re-opens r2-C1.

**Recommendation: DO NOT take it in isolation.** The cheap insurance is
already paid for. Take it **only if** cut 3.1 is taken — the layout is being
re-cut anyway — *and* the operator wants the maximal shrink; otherwise keep.

### 3.5 RELAX: R17 as a distinct named refusal → collision surfaces as merged-group decode failure

**What gets deleted.** ~15 lines, one set-level name, V27's grinding
apparatus (the ground second transaction, the committed collision input).

**The risk accepted.** The one failure whose remedy ("pack the two
transactions as separate payloads — the txids cannot be changed") is genuinely
non-obvious loses its explanation, for an event that is 2^-20 by accident.
R17 is spec §5, added with the 2026-08-24 ruling; weakening it needs a spec
edit and arguably operator agreement.

**Recommendation: KEEP R17.** One comparison at pack time, and it is one of
only two failures in the whole plan whose message changes what the operator
does. Poor simplification-per-risk despite the nonzero saving.

### 3.6 NOT proposed: refuse → report (D2)

Adopting the sibling's report-and-pack posture would *not* simplify — report
needs every check refuse needs, plus partial-classification bookkeeping, plus
"treat as secret" semantics that collide with the operator's unsealed ruling
(spec §2.4). Refuse is both the simpler and the safer branch for a regenerable
artifact. The only change owed is Part 2.2's one sentence stating the real
reason for the divergence.

---

## Part 4 — what is load-bearing and must stay

- **The decode gate and its chain** (EPD §6.3, §2.2, W15, V28). The whole
  anti-smuggling posture; operator-ruled at EPD level; r6-C1 proved the chain
  can silently end up owned by no site, so it must stay explicitly owned. The
  honest accepted-cost statement (a valid transaction is itself an
  arbitrary-byte container) must stay with it.
- **Carried txid on the CHUNKS metadata record** (spec §3.6b, decided O11).
  The device cannot derive it and the picker is keyed on it. Forced.
- **Bare-record chunks, R15 as binding, E18** (operator ruling 2026-08-24).
  Forced by the measured 37,255-vs-32,734 overflow; the sibling-conformant
  half of the design.
- **E13 and E19** (canonical, pristine chunks). Records engrave verbatim;
  uncovered characters convert scratches into silent damage (EPD §6.4), and
  `decode_chunk`'s repair-then-report-success would launder damage into the
  payload. E19 has a true shipped precedent (`seal`'s mk1 pristine refusal).
- **R2 including the bare-`mt1` extension** (r6-I3). Genuinely earned: after
  the ruling, the bearer material is the siblings, and a prefix-shaped guard
  would protect the empty envelope. Plus the no-echo-in-stderr clause (the
  clap lesson).
- **The exit-code table** (§1.5): it binds to shipped codes and a spec-ruled
  R7; getting it wrong breaks an R0-GREEN spec rule.
- **Sealing by content, step 3, including the shipped-test reversal note**
  (operator-ruled spec §2.4; r6-I2's warning that a shipped test goes RED is
  exactly what a plan is for).
- **Steps 1, 2, 11, 12 and the two-constant assertion** (§4.1): each guards a
  measured failure on today's binary (frozen `seal` constant; empty `--in`
  packing at exit 0; the TTY hang; the unwatched `sysw` join).
- **§3.2 / the independent generator / no-regenerate rule**: constellation
  doctrine with a measured failure behind it (r5-C1 — the golden-file suite
  that could never have gone green).
- **The per-rule RED-test and near-miss discipline**: cheap per rule and the
  best false-PASS defence in the plan. The right way to shrink it is fewer
  rules (Part 3.1), never weaker testing of the rules that remain.
- **A cross-language fixture per se**: Rust-primary requires it (spec §2.1b).
  Only its rule-name arm is discretionary (3.2).

---

## Verdict

**The design core is proportionate; the document and two apparatus choices are
not.** What the format irreducibly needs — a framed record, the decode gate,
carried identity for chunks, canonical pristine chunks, set binding with an
orphan/collision story, the argv/stdin/sealing plumbing — is sound, mostly
operator-ruled, and sibling-conformant where a sibling exists. The excess is
concentrated in three named places: a TLV encoding whose only benefit the plan
itself refuses to honour (≈7 rules, ≈6 vectors, 6 names, paid again in Go and
on the device); a 26-name cross-language string vocabulary that has become the
plan's principal generator of its own review findings (both round-7 Criticals,
plus r6-I4/r7-I3/r7-I5, are defects of the apparatus, not the format); and
25–30% of the byte count re-narrating fold history that the persisted reports
already hold verbatim. Taking Part 2 plus cuts 3.1–3.3 — no operator ruling
touched, no funds-safety property weakened, one re-review round — yields a
plan of roughly **900–1,100 lines, ~13 rules, ~23 vectors, the same 13 wiring
sites and 12 steps**: about half the current size, and much closer to the
611-line plan that built the entire container this record rides in. The plan
is disproportionate by roughly 2×, and the disproportion is removable without
relaxing anything that protects the operator.
