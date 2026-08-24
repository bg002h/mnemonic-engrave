# R9 — out-of-scope sweep, SPEC_mt_v0_1.md

**Question:** does the spec still contain live material describing something
`mt` v0.1 does not do (mt qr behaviour, QR configuration, sysw/SH2/machine
engraving, plate counts/budgets/costs, transaction construction, script
evaluation, redundancy/fountain coding, a future shared BCH crate) — as
distinct from §12 history, narrated corrections, or legitimate
contrast/deferral-explanation?

**Method.** The document (3,729 lines) was read in full: lines 1–400 and
1500–2699 read directly by the orchestrating agent; lines 400–1500 and
2600–3729 swept by two independent forked agents inheriting the same task
definition and exclusion rules. Findings below were then independently
re-verified by the orchestrating agent against the live file (not taken on
either sub-report's word) before being written up. The gap between the
manually-read range (…2699) and the second fork's range (2600–3729) overlaps
by 99 lines; no discrepancy was found there.

**Scope already excluded**, per the task brief, and not re-reported: §4 (QR
config), §8.7/§8.7c refusals, the `mt qr` CLI-surface row, `PLATE n OF m` in
the legend, and the plate-mapping claims removed from the offline warning,
success report, §0's opening and `inspect`'s appended rows (all from the
prior out-of-scope sweep).

---

## Findings

### Finding 1 — Important — §8 item 8, module size presented as live v0.1 behaviour

**Lines 2797–2802:**

> 8. **Module size is the operator's choice, defaulting to 0.60 mm** — not a
>    refusal. Ruling 2026-08-23 (§10.1): `mt` offers every size it can engrave and
>    suggests 0.60 mm (two engraved strokes). Sizes below that are **optically
>    unvalidated**, and `mt` says so at the point of choice rather than refusing.
>    A scan that succeeds today is evidence about one plate on one machine on one
>    day, not a property of the size (§10.1).

**Why out of scope.** This sits inside §8 ("Refusals"), a numbered list whose
own framing (line 2067: *"Every refusal below binds BOTH verbs unless it names
one"*) leads a reader to expect every entry to be live v0.1 behaviour, exactly
like its neighbours 1–7b and 9. Its two immediate siblings in the same list —
item 7 (`mt qr` plate-budget refusal) and item 7c (`sysw` section-ceiling
refusal) — were both correctly converted to `**MOVED — see
design/SPEC_mt_qr_DEFERRED.md**` stubs by the prior sweep. Item 8 was not: it
still asserts a concrete default (0.60 mm), a specific mechanism ("offers
every size it can engrave"), and a QR-specific concept ("optically
unvalidated," "a scan that succeeds"). `mt encode`, the only live v0.1 verb,
has no engraving geometry at all — §0a states plainly that *"`mt encode`'s
layout is the operator's by ruling"* and *"the realistic plate has NO legend
on it."* There is no module size concept for a verb that emits a bare
character string.

Worse, its own citation (§10.1) is itself the deferred material: §10.1 (line
2881) reads `**MOVED — see design/SPEC_mt_qr_DEFERRED.md.** The F-234 optical
test plate has not been cut — and the module size is now. mt qr material,
deferred with the verb (§0a)`. So item 8 cites a moved, deferred open
question as live authority for a present-tense v0.1 ruling — a direct
inconsistency, not merely thematically adjacent material.

Confirmed by inspection that the identical content (0.60 mm floor, "two
strokes," "offers every size `mt` can engrave," "not a property of the module
size," the optical-validation caveat) already exists verbatim in
`design/SPEC_mt_qr_DEFERRED.md` lines 111–147, attributed there to *"§8.8's
hard refusal below 0.60 mm becomes a default and a recommendation."`* The
material was copied forward to the deferred file but never removed or
stubbed in the live spec — the other two siblings got the stub treatment and
this one was missed.

**Disposition:** delete/reword to the same `**MOVED — see
design/SPEC_mt_qr_DEFERRED.md**` stub pattern used at items 7 and 7c, since
the content is already present, verbatim, in the deferred file.

### Finding 2 — Important (linked to Finding 1) — §10.10 input table cites module size as a live v0.1 input

**Line 3028**, in the "inputs `mt` needs, and which section needs them" table:

> | module size | §8.8 | default 0.60 mm |

**Why out of scope.** This table's own adjacent row shows the correct
pattern for a deferred input — `| **plate budget** | §8.7 | **§8.7 cannot
run** |` (line 3023) — because §8.7 was properly stubbed as MOVED. The module
size row instead states a genuine, working default ("0.60 mm") as if `mt`
v0.1 actually consumes this input, propagating Finding 1's error into a table
whose explicit purpose (line 3031: *"Naming them is a prerequisite for
implementation, not a nicety"*) is to tell implementers what `mt` v0.1 needs.
An implementer following this table literally would build a module-size input
path into `mt encode`/`decode`/`verify`/`inspect`, none of which has any
concept of engraving geometry.

**Disposition:** reword to `**§8.8 cannot run**` (matching the plate-budget
row), or delete the row outright, once Finding 1 is fixed upstream.

### Finding 3 — Minor — §1 decision list, item 3: `mt qr` payload form stated as a flat ruling

**Line 1124**, inside "## 1. The operator's decisions, recorded" (a numbered
list of 8 items following the long decision 1):

> 3. **The QR carries the standard form, never a codex32 string** (F-234).

**Why flagged, and why only Minor.** This is a single declarative sentence
about the deferred `mt qr`'s payload format, with no "(deferred)" or
"(§0a)" qualifier, sitting in a list that otherwise mixes genuinely
cross-verb rulings (item 4: UR dropped, applies to both verbs' shared header;
item 8: zero redundancy, applies to both) with QR-only ones. Unlike the rich
contrast/justification framing found elsewhere in the document (§3, §3a,
§0a), this line offers no signal to a reader that it binds only the deferred
verb. However, it is terse, cites its own decision number (F-234) for
traceability, and specifies no operative detail an implementer could act on
for v0.1 — so it is unlikely to cause a reader to build the wrong thing. Not
covered by the prior sweep (which touched §4, §5, §8.7/8.7c, the CLI table,
and the legend — not this list).

**Disposition:** reword with an explicit "(mt qr, deferred — §0a)" tag, or
move to `SPEC_mt_qr_DEFERRED.md` alongside its sibling QR-payload-form
material already retained in §3.

### Finding 4 — Minor — §1 decision list, item 5: Reed-Solomon/plate-count ruling stated flatly

**Line 1131**, same list:

> 5. **Reed-Solomon density is the highest that still minimises plate count.**

**Why flagged, and why only Minor.** Reed-Solomon is exclusively the
machine-engraved QR error-correction layer — §3a's own contrast table (around
line 1392) names Reed-Solomon as the QR-native correction versus BCH for
hand engraving. This item packs together two categories the task brief names
explicitly as out of scope: QR configuration (ECC level/density) and a plate
count claim ("minimises plate count"). Like Finding 3, it carries no
deferral qualifier in a list that otherwise mixes live and QR-only rulings.
Kept at Minor rather than Important because it states a design principle,
not a concrete number or mechanism — nobody could implement `mt qr`'s ECC
level from this sentence alone, and it does not describe `mt encode`/`decode`/
`verify`/`inspect` behaviour that a v0.1 reader could mistakenly rely on.

**Disposition:** same as Finding 3 — tag as deferred or move to
`SPEC_mt_qr_DEFERRED.md`.

---

## Explicitly checked and NOT flagged (to save a future sweep re-deriving these)

- §0, §0a, §1 decision 1 (verb naming, chunk numbering, BCH correction
  mechanics, duplicate-chunk resolution) — all live v0.1 material, correctly
  scoped.
- §2, §3, §3a, §3b (codec specification, envelope/UR history, chunking math,
  "layout on steel is the user's, not `mt`'s") — correctly framed as either
  live wire-format material or explicitly-retained contrast/history per §0a's
  own statement that this retained material "binds nothing in v0.1."
- §5 (the plate legend) — correctly headed *"LIVE for `mt encode`, sized for
  the deferred `mt qr`"*, with an explicit account of which of its five
  fields are live (printed on stderr by `mt encode`) versus which
  measurements are retained for the deferred cycle only.
- §6, §6a, §7 (provenance asymmetry, node-liveness checks, threat model) — all
  live v0.1 material; `mt qr`-specific table rows (e.g. the Bearer row split
  in §7) are properly paired contrast rows, consistent with the prior sweep's
  treatment of similar splits.
- §8 items 1–7b, 9 and all of items 2b–2g, 3–6 (refusals) — live v0.1
  material; every `mt qr`-only reference inside them (e.g. the asymmetry note
  at lines 2198–2206 about the legacy-input reminder not reaching an `mt qr`
  plate) is legitimate contrast, correctly framed.
- §9's "out of scope" declarations — these are explicitly *about* what is
  deferred, not specifications of it.
- §10's open-questions list — items 1, 2, 3, 9, 17 (and others) carry the
  `**MOVED — see design/SPEC_mt_qr_DEFERRED.md**` stub correctly; items 4–8,
  10+ marked SETTLED/CLOSED/RULED are live-scoped or properly historical.
- §10.10's CLI-surface table and "inputs `mt` needs" table — clean except for
  Finding 2 above; the `mt qr` row is already absent (fixed by the prior
  sweep) and the "plate budget" row is correctly marked unrunnable.
- §10.13 (`mt1` header/encoding), §11 (provenance) — in-scope wire format,
  with legitimate QR contrasts.
- **§12, "Appendix — the settled questions," lines 3441–3729** — confirmed as
  a historical record preserved verbatim throughout, explicitly framed with
  `~~CLOSED~~`/`~~SETTLED~~`/`**CLOSED**` markers on every entry, including
  ones restating retired QR/plate/redundancy/shared-crate material (e.g.
  §12.6 on fountain redundancy, §12.11 on plate character counts, §12.17 on
  the retired shared-crate plan). None of it is a finding.

---

## Counts

- **Important: 2** (Findings 1 and 2 — linked; both trace to the same
  un-stubbed §8 item 8)
- **Minor: 2** (Findings 3 and 4)
- **Not a finding (checked, excluded correctly): the rest of the document**
