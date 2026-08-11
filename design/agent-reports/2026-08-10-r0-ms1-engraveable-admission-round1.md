# R1 review — fold of round 0 on §10.2.1a `ms1` engraveable-admission

- **Artifact:** `design/SPEC_encrypted_payload_delivery.md` @ `b193301` (fold),
  responding to round 0 @ `0323b9b`.
- **Reviewer:** independent, round 1. Author ≠ reviewer.
- **Scope:** the ONE question — did the fold fix each round-0 finding, and did
  it introduce a new defect? Not a fresh audit. Operator ruling not reopened.
- **Trees read:** `/scratch/code/shibboleth/seedhammer-b2b` @ `75233b8`
  (branch `b2b`, clean before and after); `mnemonic-engrave` @ `b193301`.

## VERDICT

**NOT GREEN — 0 Critical, 1 Important, 2 Minor, 1 Nit.**

I1–I5 and M1–M3, M5 are all genuinely fixed, several verbatim to round 0's own
suggested text. The one Important is in text round 0 never reviewed: the new
"Scope: `ms1` only" paragraph makes three specific, checkable claims about
`backup.EngraveText` (the md/mk engrave path), and I measured all three false
by direct execution — not off by a little, off by more than double.

---

## FOLD-VS-FINDINGS

### I1 — FIXED. New claim checked against code and confirmed true.

Current text (§10.2.1a "Why at admission"): *"Refusing at admission does
**not** avoid the KDF — an encrypted record cannot be classified before it is
decrypted, and `AdmitSection` runs after `Open`. What it buys is twofold: it
collapses the seed's residency from the whole session to the duration of
`AdmitSection`, which then wipes every copy it made and builds no plate list;
and it produces a **precise** diagnosis instead of a false one."*

This is round 0's own recommended replacement, used almost verbatim. I
independently re-traced the code rather than trusting that provenance:
`seal/unlock_key.go:81` `Open` → `:87` `defer clear(plaintext)` →
`:99-101` `SplitSection` → `:106` `admitted, err := AdmitSection(recs,
SectionEncrypted)` → `:107-109` `if err != nil { return err }` (the `defer`
fires here). `AdmitSection`'s two existing failure paths (`seal/record.go:215,
221`, pass 1 and pass 2) both call `wipe(out)` before returning. So: the
decrypted buffer's lifetime is bounded by `UnlockWithKey`'s own return, which
happens immediately after `AdmitSection` returns; `AdmitSection`'s own heap
copies are wiped on the existing failure paths, and the spec's new "Where it
runs" clause (below) normatively requires the same for the length check. "The
duration of `AdmitSection`" is a fair characterization — there is no
intervening KDF or long-lived state between `Open` and the wipe. §2.2 item 15
was fixed in lockstep (no longer claims "before the KDF runs"; now "refused
immediately after the KDF"). **Both places agree.** This is exactly the
failure mode the brief warned about (a fix in one of two places) — checked,
not present here.

### I2 — FIXED. Normative distinguishability requirement added, matches §6.4's own `ErrTooManyRecords` precedent that round 0 cited.

### I3 — FIXED. "Where it runs" pins the per-record pass, forbids the post-loop block, and requires `wipe()` — verbatim round 0's smallest fix, and consistent with the actual `wipe(out)` pattern I re-read in `seal/record.go:214-224`.

### I4 — FIXED, with one Minor (below). Literal kept, pinning test now required, boundary sensitivity to `qr.Q` (67) restated.

### I5 — FIXED. `biptool seed -seedlen 64` → 127 chars, `cmd/biptool/main.go:312`, `EncodeMS1` corrected 74→75 in the same edit (M1 folded in).

### M1 (74→75), M2 (version/module conflation), M3 (test vectors) — FIXED, verified below. M5 (Rust home) — FIXED (`record::validate_record` named; the "new `RecordError` variant" phrasing from round 0 didn't make it in, but that was M5's low-order detail, not its ask).

### M4 — FIXED in substance, citation dropped. Round 0 asked for "the 24-word cap cited to `bip39/bip39.go:290`"; the fold's new scope note states the fact (QR 29) but without the citation. Folded into the Important finding below since it's the same new paragraph.

---

## IMPORTANT

### New "Scope: `ms1` only" paragraph — three specific claims about `backup.EngraveText`, all false when executed.

**Exact text (§10.2.1a):** *"md/mk plates take a different engrave path
entirely (`backup.EngraveText`), which **caps at QR 37**, **scales modules at
2 stroke widths rather than 3**, and **degrades TEXT+QR → TEXT-ONLY →
QR-ONLY** rather than failing."*

I built a minimal `md1`-shaped string, grew it one char at a time, and called
the real `validateMdmk` (`gui/gui.go:2095`, the actual `backup.EngraveText`
caller) at each QR-size transition, in the `b2b` tree:

| QR dim | `TEXT + QR` | `TEXT ONLY` | `QR ONLY` |
| --- | --- | --- | --- |
| 21–57 | yes | yes | yes |
| 61–85 | **no** | yes | yes |
| 89 | no | yes | **no** |
| ≥93 | no | no | no — `"backup: data does not fit plate"` |

1. **"caps at QR 37" is false.** `TEXT + QR` still succeeds at dim 57 (more
   than half again past 37); `QR ONLY` alone succeeds all the way to dim 85.
   Nothing fails at 37 — 37 sits in the middle of the range where all three
   variants still fit. The real ceiling is dim 89–93, more than double the
   stated number.

2. **"scales modules at 2 stroke widths rather than 3" is false for the
   actual callers.** `EngraveText`'s `p.QRScale == 0 → qrScale = 2` default
   (`backup/backup.go:343-346`) is real, but it is dead code for md/mk: the
   only two callers of `backup.EngraveText` (`validateMdmk`, `gui/gui.go:2100`;
   `validateDescriptor`, `gui/gui.go:464`) both declare `const qrScale = 3`
   and pass it explicitly as `QRScale: qrScale`. Every md/mk QR paragraph is
   built with scale **3**, identical to the seed plate, never 2. `2` is
   `freeTextQRScale` (`backup/fit.go:19`), which belongs to a different
   feature (the free-text plate, `backup/freetext.go`) that does not go
   through `EngraveText` at all.

3. **The degrade order is backwards from what's measured.** The chain
   `TEXT+QR → TEXT-ONLY → QR-ONLY` reads as: lose the QR, then lose the text.
   Measured, it's the opposite pairing that survives longest: at dim 89, `TEXT
   ONLY` still fits and `QR ONLY` does not — the QR-only variant is the
   **first** single-mode variant to fail, not the last, and the two single
   modes are offered *together* (dim 61–85) rather than as a strict sequence.

**Why this matters beyond the three numbers.** The paragraph's own conclusion
— "`ms1` is the one format that can neither chunk nor degrade, which is why it
alone needs this rule" — is still correct: `EngraveSeedString` genuinely has
no fallback path, unlike md/mk. But the specific mechanics offered as evidence
for it are fabricated, in a **NORMATIVE** section, about a path (`backup/`)
this same paragraph cites by name. This is the same defect class as round 0's
I5: a false, specific, checkable claim that happens to sit next to a true
conclusion. A future reader has no way to tell the false parts from the true
one without re-measuring, and the false "37" is exactly the kind of number
someone would reuse later (e.g. reasoning about a future plate-size change)
and get burned by.

**Smallest fix.** Replace the clause with what's measured: *"and md/mk plates
take a different engrave path entirely (`backup.EngraveText`), which offers
`TEXT+QR` up to QR dim 57, then narrows to `TEXT ONLY`/`QR ONLY` (dim 61–85),
then `TEXT ONLY` alone (dim 89), failing outright only past dim 93 — using the
same scale-3 QR as the seed plate (`gui/gui.go:2100`), not a smaller one."* Or,
if the paragraph doesn't need the specific numbers to make its point, drop
them and keep only the true, load-bearing claim: *"md/mk plates degrade
through multiple engrave variants instead of failing outright; `ms1` has no
such fallback."* Also restore round 0's citation `bip39/bip39.go:290` for the
24-word claim (M4's original ask), since it's in the same paragraph and the
fold otherwise dropped it.

---

## MINOR

### New-M1 — I4's "90 MUST be pinned by a test" paragraph names `qrScale` as one of three things that move the 90/91 boundary; it doesn't.

**Text:** *"It is derived from three things that can each move independently:
`qrScale`, the `qrc.Size > 33` cap, and the error-correction level."*
`EngraveSeedString` (`backup/backup.go:125-138`) computes `qrc, err :=
qr.Encode(seed, qr.M)` then `if qrc.Size > 33 { return nil, errors.New(...) }`
— the 90/91 boundary is decided here, entirely as a function of the ECC level
and the `33` cap. `qrScale` (the local const `=3` in the *separate*, lowercase
`engraveSeedString` renderer) is read only *after* this check has already
passed, to size the physical QR on the plate — it cannot move where the cap
fires. Round 0's own phrase here was "the plate layout" (vaguer, and at least
not a named variable); the fold's swap to a specific, wrong variable name is a
regression in precision, though the *paragraph's actual requirement* — pin `90`
by testing `EngraveSeedString` at 90/91 directly — is unaffected and still
correct. **Fix:** drop `qrScale`, keep the other two (ECC level, the `33`
cap), or say "the plate's physical layout" if a third factor is wanted.

### New-M2 — M4's citation to `bip39/bip39.go:290` was dropped.

Round 0's M4 fix explicitly asked for the 24-word cap "cited to
`bip39/bip39.go:290`." The fold's new scope note states the fact (*"A 24-word
mnemonic plate tops out at QR 29"*) but carries no citation. Low-cost to
restore since it's a single file:line in the same sentence.

---

## NIT

### N1 — Two mid-sentence line breaks orphan single words.

Diff shows *"...cannot produce one — it\ncaps\nentropy at BIP-39..."* and
*"...the plate's\nQR would need 41 modules..."* — `caps` alone on its own
source line. Renders fine as prose (soft line break), and the brief's
already-verified wrap-gate covers line length, not break placement. No
operator-facing or reviewer-facing consequence — purely cosmetic.

---

## WHAT I MEASURED (independent, not re-derived from the report)

- `AdmitSection` call order and `wipe`/`defer clear` pattern: read
  `seal/unlock_key.go:81-110` and `seal/record.go:204-269` directly.
- md/mk degrade boundary: ran `validateMdmk` (`gui/gui.go:2095`) at every QR
  size transition from dim 21 to dim 129 via a scratch test in `gui/`
  (`TestZZScratchMdmkBoundary`); removed after, `git status` clean.
- md/mk QR scale: grepped every `backup.Paragraph{...QRScale...}` construction
  site; both real callers (`gui/gui.go:464`, `:2100`) use `const qrScale = 3`.
- Test-vector constructibility: ran `codex32.NewSeed` + `codex32.New` over
  payload byte lengths 1–65 via a scratch test in `codex32/`
  (`TestZZScratchLengths`); confirms 90/91/93/125/127 all pass `New`, 92 never
  appears as an output length, 124 is produced by `NewSeed` but rejected by
  `New` ("invalid length"). Removed after, `git status` clean.
- `seedhammer-b2b` tree left clean (`git status --short` empty except one
  pre-existing untracked file, `gui/f103_scratch_test.go`, timestamped before
  this review started and not touched by it).

No files modified in `mnemonic-engrave` other than this report.
