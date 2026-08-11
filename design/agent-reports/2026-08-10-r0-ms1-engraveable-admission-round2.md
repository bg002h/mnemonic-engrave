# R2 review — fold of round 1 on §10.2.1a `ms1` engraveable-admission scope note

- **Artifact:** `design/SPEC_encrypted_payload_delivery.md` (current, HEAD),
  fold commit `246398f`, responding to round 1's report
  (`design/agent-reports/2026-08-10-r0-ms1-engraveable-admission-round1.md`).
- **Reviewer:** independent, round 2. Author ≠ reviewer.
- **Scope:** the ONE question — did the fold fix round 1's Important and its
  Minors, and did it introduce a new defect? Not a fresh audit. Rounds 0/1 and
  the operator ruling not reopened.
- **Trees read:** `/scratch/code/shibboleth/seedhammer-b2b` @ `c0c958d`
  (branch `b2b`, clean, `git status --short` empty except untracked
  `biptool`); `mnemonic-engrave` @ working tree (`246398f` HEAD).

## VERDICT

**GREEN — 0 Critical, 0 Important.**

Round 1's single Important (three fabricated `backup.EngraveText` mechanics)
is fixed by removal, not correction — the false numbers are gone from every
normative sentence and survive only inside a parenthetical explicitly labeled
"measured false." The replacement prose makes five new checkable claims; all
five verified true against `seedhammer-b2b` @ `c0c958d`. §10.2.1a, §2.2 item
15, §2.3, and §10.2.1's table row still agree. One Minor and one Nit found,
both cosmetic/audit-trail, neither blocking.

---

## FOLD-VS-FINDINGS

### Important (three false `EngraveText` claims) — FIXED by removal.

Grepped the whole file for `caps at QR 37`, `2 stroke widths`, `scales
modules at 2`, and `TEXT-ONLY.*QR-ONLY` / `TEXT+QR.*TEXT-ONLY`: the only hits
are inside the new parenthetical (`design/SPEC_encrypted_payload_delivery.md:1367-1373`),
which frames them as *"An earlier draft of this paragraph asserted [...] All
three were measured false in R0 round 1"* — not asserted as fact anywhere.
No second, unfixed copy (the failure mode round 1's own brief warned about).

**Five new claims in the replacement text, each checked against
`seedhammer-b2b` @ `c0c958d`:**

1. *"A descriptor too long for one record is split across records that
   reassemble by `(HRP, chunk_set_id)` — §10.2.1 already requires the device
   to verify that."* — TRUE. §10.2.1's table row (`design/SPEC_encrypted_payload_delivery.md:1309`)
   already states exactly this requirement, verbatim-consistent.
2. *"`gui/gui.go:2106-2108` builds 'TEXT + QR' and 'QR ONLY'"* — TRUE, exact
   lines: `gui/gui.go:2106` `{"TEXT + QR", ...}`, `:2107` `{"TEXT ONLY",
   ...}`, `:2108` `{"QR ONLY", ...}`, inside `validateMdmk`.
3. *"`backup.EngraveText`, which renders its QR through `engrave.QR`"* —
   TRUE, `backup/backup.go:357`: `qr = engrave.QR(params.StrokeWidth,
   qrScale, p.QR)`.
4. *"`backup.EngraveSeedString` has exactly one shape, returning 'seed too
   long to engrave QR'"* — TRUE. `backup/backup.go:125-138` is one linear
   path (no plate-shape branches, unlike `validateMdmk`'s three), and the
   error string at `:132` is `errors.New("seed too long to engrave QR")`,
   verbatim match to the spec's quote.
5. *"`engrave.ConstantQR` — the CONSTANT-TIME encoder used because a
   secret's QR pattern must be content-independent"* — TRUE per the code's
   own comments: `engrave/engrave.go:416` *"ConstantQR is like QR that
   engraves the QR code in a pattern independent of content"*; `:619`
   *"ConstantQRCmd represents the constant time plan"*. Its only callers
   (`EngraveSeed`, `EngraveSeedString`) both handle seed/secret material
   (`gui/gui.go:556`, `:2243`, `gui/unlock_session.go:196`,
   `gui/slip39_polish.go:496`), supporting the "secret" framing.

**Conclusion still supported.** *"`ms1` has neither: a seed share is atomic
and cannot be chunked, and `backup.EngraveSeedString` has exactly one
shape."* Checked independently: `grep -rln chunk codex32/*.go` hits only
`mddata.go`, `mdencode.go`, `mkencode_test.go`, `mkdata_test.go` — no
chunk/`chunk_set_id` machinery in the `ms1`-only files (`codex32.go`,
`msencode.go`). The conclusion rests on the two new bullets plus this
independently-confirmed fact, not on thin air.

### New-M1 (`qrScale` wrongly listed as moving the 90/91 boundary) — FIXED correctly.

`design/SPEC_encrypted_payload_delivery.md:1412-1416` now lists only the
`qrc.Size > 33` cap and the ECC level, with an explicit parenthetical: *"`qrScale`
is NOT among them — the boundary is decided by `qr.Encode` before `qrScale`
is ever read."* Matches round 1's suggested fix exactly.

### New-M2 (`bip39/bip39.go:290` citation dropped) — NOT restored; underlying claim deleted instead. Commit message inaccurate.

The commit message claims *"M4's `bip39/bip39.go:290` citation restored."*
This is false: `grep -n "bip39.go" design/SPEC_encrypted_payload_delivery.md`
returns nothing, and the sentence the citation belonged to — *"A 24-word
mnemonic plate tops out at QR 29"* — is gone from the file entirely (it was
part of the paragraph replaced wholesale by the two-bullet restructure).
Functionally the round-1 concern (an uncited quantitative claim in a
normative section) is moot because the claim itself no longer exists — same
resolution-by-removal pattern as the Important above, and not a spec defect.
But it means the commit message misdescribes its own diff: nothing was
restored. **Severity: Minor** (audit-trail/record accuracy, not spec
correctness — no orphaned reference to the deleted claim exists anywhere
else in the file; checked §2.2 item 15, §2.3, and found no dependency on the
QR-29 figure).

### N1 (line-wrap orphans) — pre-existing instance untouched (expected, Nit); one new instance introduced by this fold's reflow.

The original N1 instances (`caps` alone on its line, `:1405`; the plate's /
QR would need 41 modules split, §2.2 item 15 `:236-237`) sit outside this
fold's diff hunks and are unchanged — consistent with round 1 scoring N1 a
non-blocking Nit that doesn't obligate a fix. New instance: comparing
pre-fold (`b193301:design/SPEC_encrypted_payload_delivery.md:1397`, *"at
`qr.Q` instead of `qr.M` the"* all on one line) against current
(`:1416-1417`, reflowed so *"`qr.M` the"* now sits alone on its own line) —
this fold's paragraph edit introduced a fresh orphan-word wrap, same
cosmetic class as N1, no operator/reviewer-facing consequence. **Severity:
Nit.**

---

## REMOVAL DAMAGE — none found

- Checked every other reference point the brief named:
  `design/SPEC_encrypted_payload_delivery.md:234-246` (§2.2 item 15) — talks
  about the 125–127-char long form and >90-char short form; no dependency on
  the deleted md/mk mechanics or the QR-29 figure.
  `:276-297` (§2.3) — no reference to `EngraveText` mechanics either.
  `:1309-1310` (§10.2.1's table row) — `(HRP, chunk_set_id)` requirement and
  "`ms1` MUST additionally be engraveable per §10.2.1a" both agree with the
  current §10.2.1a text.
- Ran `scripts/plan-cite-gate.sh design/SPEC_encrypted_payload_delivery.md
  /scratch/code/shibboleth/seedhammer-b2b`: every `file:line` and
  `pkg.Symbol` citation in the whole spec resolves (`RESULT: every citation
  resolves`). Note: the gate's regex only matches single-line citations
  (`path.go:NNN`), so `gui/gui.go:2106-2108` (a range) isn't mechanically
  covered by it — verified that one by hand instead (see claim 2 above).
- Checked wrap width on the touched paragraph (`design/SPEC_encrypted_payload_delivery.md:1349-1421`):
  max line length 81 chars, consistent with the rest of the doc; no line
  over the wrap, matching the commit message's gate claim.

---

## WHAT I MEASURED (independent, not re-derived from prior reports)

- `grep`/`Read` over current `design/SPEC_encrypted_payload_delivery.md` for
  every false-claim string and every new claim's citation.
- Direct `Read`/`grep` of `seedhammer-b2b` @ `c0c958d`:
  `gui/gui.go:2080-2120` (plate-variant construction),
  `backup/backup.go:100-145,325-370` (`EngraveSeed`, `EngraveSeedString`,
  `EngraveText`), `engrave/engrave.go:410-420,619-621` (`ConstantQR`
  comments), `grep -rln chunk codex32/*.go` (chunking confined to md/mk
  files).
- `scripts/plan-cite-gate.sh` run against the current spec, pointed at
  `seedhammer-b2b`, output captured above.
- `git show b193301:design/SPEC_encrypted_payload_delivery.md` vs. current,
  to isolate the new wrap orphan from the pre-existing N1 instances.
- `seedhammer-b2b` tree confirmed clean before and after (`git status
  --short`); no files modified in `mnemonic-engrave` other than this report.
