# R0 — rewrite-fidelity review: `SPEC_s6b_pre_flash_cycle.md` clean rewrite

**Artifact under review:** `design/SPEC_s6b_pre_flash_cycle.md` at commit `6c85d28`.
**Previous version:** `git show 6c85d28~1:design/SPEC_s6b_pre_flash_cycle.md` (795 lines).
**Diff:** `git show 6c85d28` — 417 insertions, 661 deletions.
**Source (fork):** `bg002h/seedhammer`, `main` = `b1479a1b38f6b045d27443764c858906e4e6e122`
(re-verified: `git rev-parse HEAD` matches, tree clean).

**Lens — the two questions only.** (1) Did the rewrite DROP anything settled,
required, or measured? (2) Did the rewrite INTRODUCE a contradiction or an
unsupported claim? Not a fresh correctness audit; R-A…R-M are operator decisions
and are not re-opened.

**Machine-checked before this review, independently re-run (not trusting the
rewrite commit's own claims):**

```
./scripts/plan-cite-check.sh design/SPEC_s6b_pre_flash_cycle.md
  → citations resolved: 87 / 87 ; dangling: 0

./scripts/plan-table-check.sh design/SPEC_s6b_pre_flash_cycle.md
  → table rows checked: 46 ; malformed: 0
```

Both match the rewrite commit's stated numbers exactly. **Neither gate covers
prose citations of the form `Document.md §N`** (only `path/file.ext:N`) — that
blind spot is exactly where finding M4 below lives, and it is why it survived
the gate.

---

## 1. R-A … R-M — honoured / lost, where

| ruling | status | where in the rewrite |
| --- | --- | --- |
| **R-A** — watch-only sets not marked | honoured | §1.2 ("Both rows read off R-A's predicate — the set contains a seed... a passphrase-derived watch-only engrave... is unmarked") |
| **R-B** — multisig marking → new phase, not gated here | honoured | §0 ("Multisig marking and F-205 → phase `key & password custody refinement` (R-B). Not closed here, even incidentally"); §1.3 table marks multisig unmarked |
| **R-C** — R2 is the existing passphrase program, preloaded | honoured | §2.1 ("the device runs the existing dedicated passphrase-plate program with values already in hand rather than building a new offer flow") |
| **R-D** — "all things said must be true" | honoured (substance everywhere; explicit citation count reduced 6→3, see note) | §2.2, §2.6, §3.3 explicit; substance also drives §2.3's provenance field, §3.2a's forbidden alternative, §5.1's predicate correction, without repeating the label every time |
| **R-E** — `fadeClip` stays stubbed | honoured | §0 ("Out: Restoring `fadeClip`'s clip mask (R-E)"); §5.1 ("While `fadeClip` stays stubbed... Against the panel, which is what R-E required") |
| **R-F** — optional `Title`/`Footer` on `backup.Text` | honoured | §1.1 ("Per R-F... Text gains two optional string fields") |
| **R-G** — no golden churn is the assertion | honoured | GATE 1.1 in §7; "Golden policy (R-G)" paragraph closing §7, verbatim in substance |
| **R-H** — policy id in the footer, preloaded path only | honoured | §2.3 (`POLICY <8 hex, grouped> DERIVED, NOT TYPED`, 36 chars, matches R-H exactly); §8 records R-H's `bottomLines`/`topLines` asymmetry as a recorded, not re-opened, choice |
| **R-I** — arrows float over body edges | honoured | §5 ("Per R-I: arrows draw at the top-centre and bottom-centre of the body..."), geometry numbers match verbatim |
| **R-J** — device preloads fingerprints too | honoured | §2.1 ("Per R-J it preloads the fingerprints too, not only the passphrase"); presented as **settled from the first sentence**, not reopened as OPEN anywhere (see XC1 below) |
| **R-K** — the threat model (sealed payload is the security program) | honoured | §2.6 (verbatim threat-model paragraph, "R-K does not relax R-D") |
| **R-L** — F-204 copy approved as specified | honoured by content (not cited by name — same as old spec, see note) | §3.2 (conditional copy, passphrase-entered / not, matches R-L's approval) |
| **R-M** — `provedInnocent` arm must say "not a passphrase-protected wallet" | honoured | §3.2a, wording block byte-identical to `REQUIREMENTS` §2bis's "ADOPTED WORDING" (`diff` run, exit 0) |

**Note on R-D citation density:** the old spec named "R-D" 6 times; the new spec
names it 3 times. Checked each of the 3 missing citations individually — in
every case the *substantive constraint* (a footer/screen/document may not
assert something false) is still stated in the same place, just without
repeating the ruling's label. Not a drop; a citation-density style choice.

**Note on R-L:** neither the old spec nor the new spec cites "R-L" by name
anywhere (confirmed by grep on both). §3.2's content matches R-L's approval in
both versions. No regression.

---

## 2. The 19 comprehension findings — incorporated / not, where

| id | sev | incorporated | where |
| --- | --- | --- | --- |
| XC1 | Critical | **yes** | R-J stated as settled from §2.1's first mention; grep for the old "operator's call / Until that is ruled / Until then" language returns nothing |
| XC2 | Critical | **yes** | new §6, "THE RESTORE DOCUMENT (R1)" — conditional clause, keyed on whether *this run* cut a plate |
| XC3 | Critical | **yes** | new §6.1 — passphrase plate named on a separate line, `len(plan)` unchanged, matching the finding's own prescribed fix almost verbatim |
| XI1 | Important | **yes** | §2.1 NORMATIVE 1–3 — preloaded entry carries seed FP/combined FP/policy-id hex as parameters; steps elided from the sequence, not skipped inside it |
| XI2 | Important | **yes** | no "OPEN" section remains anywhere (grep confirms); GATE 2.3 and GATE 2.3c sit in §7's normal table |
| XI3 | Important | **yes** | no orphaned 27-char footer text remains (grep for "FPS TYPED" / "27 char" returns nothing); only the 36-char preloaded / 32-char standalone forms appear |
| XI4 | Important | **yes** | §1.3 — "every other caller passes `""`, `""` explicitly. Three tests assert the call text as a source string and must be updated in the same commit" — matches the three cited test lines exactly |
| XI5 | Important | **yes** | §5.1 NORMATIVE predicate is `bodyClip.Min.Y + scrollFadeDist + bodysz.Y > dims.Y`, i.e. fires at `bodysz.Y > 260` — the corrected number, not the old 270 |
| XI6 | Important | **yes** | §1.1.2 — "The title is plate row 0; the footer is the last plate row. Both are screw-hole rows. This is normative and load-bearing"; GATE 1.2b added |
| XI7 | Important | **yes** | §2.3 — "What selects between them is a recorded PROVENANCE, not the policy id. `backup.Passphrase` gains a field stating whether the fingerprints were derived or typed" |
| XM1 | Minor | **yes (moot)** | no correction blocks exist in the rewrite by design; the superseded-text-under-NORMATIVE-heading problem cannot recur |
| XM2 | Minor | **yes** | "Cross-document references are always qualified by document" (line 16); `SPEC_seedhammer_systemwide_payloads.md §7.4` now names its document; remaining bare `§2.x` refs are same-document self-references (no collision) |
| XM3 | Minor | **yes** | §1.2 splits the footer into `COMB FP` (passphrase-derived) vs `SEED FP` (bare-seed), removing the mislabeled case entirely |
| XM4 | Minor | **yes** | §3.3 — replacement clause written out: `` The ms1 you typed for each seed matched. `` |
| XM5 | Minor | **yes** | GATE 3.3 states explicitly it is "a flow-level assertion driving verify with two seeds" and names the middle case as "the one that kills the filed remedy" |
| XM6 | Minor | **yes** | §1.2 — "The marking renders on all three engraving variants... Title and footer are plate rows, not paragraph content, and render in every variant"; GATE 1.2 extended to assert this |
| XM7 | Minor | **yes** | §2.4 — "It is computed from the FINAL `b.MD1`, at the offer site"; GATE 2.4c added |
| XN1 | Nit | **yes** | §3.2a — "`multisigVerifyNoSlotBody`'s doc comment... describes the arm being replaced and is updated in the same commit" |
| XN2 | Nit | **yes** | §8 — R-H's `bottomLines`/`topLines` asymmetry recorded verbatim, "recorded so it is a choice rather than a discovery" |

All 19 incorporated. None weakened relative to the finding's own prescribed fix.

---

## 3. Round 1 (C1–3, I1–5, M1–4) and round 2 (N1–3) fixes — carried forward / lost

All 15 findings' fixes were re-checked directly against the new spec's text
(not inferred from the old spec having had them, since the old spec was itself
what round 3 closed GREEN before the rewrite).

| id | fix | carried forward |
| --- | --- | --- |
| C1 | footer must not claim derivation while typed | **yes** — §2.3 + GATE 2.3b |
| C2 | `md.FormAwareStubChunks`, not `WalletPolicyIDStub` | **yes** — §2.4 + GATE 2.4b (value equality) |
| C3 | condition moved one frame up to `gui/singlesig.go:177` | **yes** — §1.3 |
| I1 | widened GATE 1.3 to name every unmarked flow | **yes** — GATE 1.3 names `deriveXpubFlow`, `Engrave Multisig`, `Build Policy`, `Engrave Bundle`, `mdmkFlow`, `cardMS1` |
| I2 | Title row reads off R-A's seed predicate too | **yes** — §1.2 table |
| I3 | `cardMS1` never marked (wording) | **yes** — §1.2 |
| I4 | chip-bounds requirement + GATE 5.3 | **yes** — §5 point 3 + GATE 5.3 |
| I5 | predicate stated as expression; GATE 5.1/5.1b split | **yes** — §5.1, GATE 5.1 / 5.1b |
| M1 | `:854` gated by source assertion, not behavioural | **yes** — §3.1 table + GATE 3.1 |
| M2 | GATE 2.4a replaced with fingerprint-construction-site assertion | **yes** — §2.4 + GATE 2.4a |
| M3 | stale doc comment flagged for same-commit fix | **yes** — §8 |
| M4 | third multisig arm recorded as scope choice | **yes** — §8 |
| N1 | offer located inside `engraveSingleSigFlow`, between verify offer and `restoreDocFlow`; GATE 2.3e (offer-appears predicate) | **yes** — §2.6 + GATE 2.6; insertion-point line ranges (`:188-192`, `:221-223`) independently re-verified against fork source, exact |
| N2 | `bundlePlate` gains `kind`, conditions title/footer pass-through on `p.kind != cardMS1` | **yes** — §1.2 "Mechanism" paragraph |
| N3 | GATE 2.3d/3.2a added to the collected table | **yes, and exceeded** — every gate in the rewrite is now a table row (29 rows, all inline gates included); the old inline-vs-table distinction that caused N3 cannot recur |

**Round 3's own Nit** (function name `singleSigEngraveFlow` → `engraveSingleSigFlow`)
is also carried correctly: new spec §2.6 says `engraveSingleSigFlow`
(`gui/singlesig.go:38`) — confirmed against source, exact function exists at
that line.

No round 1/2/3 fix was lost or weakened.

---

## 4. Gates — old vs new, anything missing

Old table: 22 rows (`1.1, 1.2, 1.2a, 1.3, 2.2, 2.3, 2.3b, 2.3c, 2.3d, 2.3e,
2.4a, 2.4b, 2.5, 3.1, 3.2, 3.2a, 3.3, 4, 5.1, 5.1b, 5.3, —`).

New table: 29 rows (`1.1, 1.2, 1.2a, 1.2b, 1.3, 2.1, 2.2, 2.3, 2.3b, 2.3c,
2.3d, 2.4a, 2.4b, 2.4c, 2.5, 2.6, 3.1, 3.2, 3.2a, 3.3, 4, 5.1, 5.1b, 5.3, 6,
6.1, —`).

Reconciled every old row against the new table:

- **`2.3d`** in the old spec bundled two clauses ("no fingerprint-entry step"
  + "KDF doesn't run unless offered"). The new spec **splits** these into
  **GATE 2.1** (no fingerprint-entry step, plus a new clause: Back from
  `ppStepQR` lands on a real prior step — addresses XI1) and **GATE 2.6**'s
  second clause (KDF doesn't run unless offered). Both original clauses
  present; one addition.
- **`2.3e`** in the old spec bundled "offer appears only when `passphrase !=
  ""`" and "restore doc's inventory reflects whether a plate was engraved."
  The first clause is now **GATE 2.6**'s first clause. The second clause —
  which round 1's comprehension review (XC3) found had **no mechanism and an
  obvious-but-wrong implementation** — is now **GATE 6** and **GATE 6.1**,
  materially stronger than the gate it replaces (asserts the specific
  separate-line requirement and `len(plan)` invariance, not just "reflects").
- **New rows with no old counterpart** (`1.2b`, `2.4c`, `6`, `6.1`): all are
  additions closing comprehension findings (XI6, XM7, XC2, XC3respectively) —
  net gate coverage increased, not decreased.

**No gate present in the old table is absent from the new one.** Every old
gate's assertion maps onto a new gate, at the same or greater precision.

---

## 5. What the rewrite introduced without support

No contradictions and no requirements-nobody-ruled were found. Four Minor
items, none gating:

### M-a — the "closed blind spot" (proven `md1`≤96/`mk1`≤111 bound) is gone

The old spec's §6 (post-table) carried round 1's finding that
`codex32/mdmk.go`'s `ValidMD`/`ValidMK` constants make `md1 ≤ 96 chars` /
`mk1 ≤ 111 chars` a **code-enforced, proven** maximum — closing
`SPIKE_s6b_q2_results.md` §4's own stated blind spot ("the maximum `md1`/`mk1`
chunk payload... the spike did not measure"). That paragraph, and its
conclusion ("the spike's caveat is retired"), does not appear anywhere in the
rewrite. `grep -n "96\|111\|ValidMD\|ValidMK" design/SPEC_s6b_pre_flash_cycle.md`
returns nothing relevant. **The SPIKE doc itself was never updated**, so this
closure now lives nowhere in the currently-readable design artifacts except
git history. Non-gating: round 1's own report explicitly filed this as "also
recorded, not a finding" (i.e. informational, not a requirement), and GATE
1.2's pinned representative-example test is unaffected either way.

### M-b — the raw-width-vs-layout-based method caveat is gone

Old spec §1.2a: "the two do not agree at the 6.0 mm rung — raw width admits
only 16 characters there where the shipped cap is 18... the implementation's
gate must be the layout-based form, not this one," repeated in old §7 as an
open item for future readers. Not present anywhere in the rewrite (grep for
"6.0 mm" / "rung" / "discrepancy" returns nothing). Non-gating by the old
text's own admission ("does not touch the 3.8 mm result used here"), and
GATE 1.2a already mandates the layout-based form regardless, so the practical
protection survives without the caveat's restatement.

### M-c — GATE 6's mechanism for "cut" vs "merely offered" is not named

§6's NORMATIVE text is careful and correctly scoped — "conditional on whether
**this run cut** a passphrase plate — not on the flow" (correctly distinct
from "whether the offer merely appeared," since `passphrase != ""` alone is
necessary but not sufficient: the operator can decline). No data path (e.g. a
boolean threaded from §2.6's offer into `buildPlateInventoryLines`, analogous
to the existing `rec` pattern for verify status two lines above at
`gui/singlesig.go:189`) is named. This is the same *shape* of gap as C3/N1/N2
in the document's own history, but at materially lower risk here: (a) this is
brand-new text never yet reviewed by any round — the comprehension review
found the underlying problem (XC2/XC3) but did not review this fix text; (b)
GATE 6 as an outcome-based assertion would still likely catch a wrong
implementation, unlike the historical cases where the gate only tested one
arm. Recording for implementation-time attention, not gating this review.

### M-d — citation error: `SPIKE_s6b_q2_results.md §1.2a` does not exist

New spec line 102: `"...416000 units (`SPIKE_s6b_q2_results.md` §1.2a)."`
**`SPIKE_s6b_q2_results.md` has no §1.2a section** — confirmed by direct grep
of the SPIKE file (sections present: 1, 2, 3, 3b, 4; no 1.2a anywhere). The
underlying number (25 characters / 416000 device units / 3.8 mm) is correct
and matches the old spec exactly — but its **true provenance** was the old
spec's *own* §1.2a subsection ("This was §7's outstanding gate. Run
2026-08-17, not inferred") — a measurement the SPIKE run explicitly did
**not** perform (SPIKE §4: "What remains unmeasured here is the *title*
band's horizontal fit on an `md1`/`mk1` plate... which is a different face
and a different inset from the passphrase plate's band"). The old spec never
made this mistake — it correctly cited only `SPIKE §2` and `SPIKE §3b`
(both real sections), never `SPIKE §1.2a`. This is a genuine rewrite-introduced
citation error, invisible to `plan-cite-check.sh` (which only resolves
`path/file.ext:N` citations, not `Document.md §N` prose citations — a
documented blind spot of that script). Non-gating: the number itself is right
and consistent with GATE 1.2a's requirement; only the document attribution is
wrong.

---

## Verdict

`GREEN 0C/0I`

(4 Minors recorded above: M-a, M-b — dropped non-gating context/caveats;
M-c — a new-text mechanism gap at low risk, worth implementation-time
attention; M-d — a citation misattribution. None block. All R-A…R-M rulings
are honoured, all 19 comprehension findings are incorporated, all round 1–3
fixes are carried forward with gate coverage equal or greater, and both
machine-checkable claims in the rewrite's own commit message —
`87/87` citations resolved and `46/46` table rows well-formed — were
independently re-run and confirmed true.)
