# R0 round 1 — fold review: §2.2 item 12 (responds to round 0)

**Artifact reviewed:** fold commit `4014fe2` on `design/SPEC_encrypted_payload_delivery.md`.
**Round 0 report reviewed against:** `design/agent-reports/2026-08-10-r0-spec-program-scope-round0.md` (`bd09394`).
**Scope:** did the fold fix I1–I4, the Minor, and the Nit; did it introduce a new defect. Not a
fresh audit — round 0's six settled facts and the operator ruling are taken as given.

## VERDICT — NOT GREEN: 0 Critical, 1 Important, 2 Minor, 1 Nit

I1, I2, I3 and I4 are each fixed correctly and precisely — three of them essentially verbatim to
round 0's own suggested wording, independently re-verified against the code (two `ctx.wipe =`
production sites, `SeedScreen`'s two non-test constructors, item numbering 1–12 contiguous before
`### 2.2a`, `§2.2a`'s moved body byte-identical to before). The one Important is a piece of the Nit
round 0 flagged that the fold's commit message claims was "corrected with I1" but was not: a
second, unedited occurrence of the same single-bracket claim sits two paragraphs below the fixed
carve-out, now contradicting it.

---

## IMPORTANT 1 — item 12's own "boundary is the PROGRAM" paragraph still asserts ONE bracket, unedited, two paragraphs below the now-fixed two-bracket carve-out

**Exact text at fault** (§2.2 item 12, lines 157–162, **unchanged by this fold** — confirmed via
`git show 4014fe2^:design/SPEC_encrypted_payload_delivery.md`, byte-identical):

> **The boundary is the PROGRAM, not the data's provenance.** A legacy program that reads an
> encrypted payload does **not** inherit this discipline; conversely, anything inside the Sealed
> Payload session carries it regardless of how the bytes arrived. This is not a new constraint on
> the implementation — it is what `gui/wipe_guard.go` already does, since the guard's lifetime *is*
> `unlockSecretSession`'s own first and last act.

**The fact.** This is the exact sentence round 0's Nit quoted as one of *two* occurrences of the
same error (`gui/wipe_guard.go` cited as installing the bracket; it only defines the type). The
fold's commit message says "Nit: … corrected with I1" — but I1's fix landed only in the SCOPE note
eight lines into §10.2.4, which now correctly names **two** installers (`unlockSecretSession` for
rows 1–3, `unlockPassphraseFlow` for row 4 — re-verified against `gui/unlock_session.go:87-91` and
`gui/unlock_kdf.go:135-144`, whose comments state explicitly "these two guards never nest today").
This occurrence, sitting in item 12 itself, was not touched and still names only
`unlockSecretSession`'s bracket as the entirety of "what `gui/wipe_guard.go` already does" for
"anything inside the Sealed Payload session" — which two paragraphs above it (the now-fixed
carve-out 2) explicitly includes "an in-flight passphrase." Item 12 now asserts both a one-bracket
and a two-bracket model of its own boundary, eleven lines apart.

**Concrete scenario.** This is round 0's own I1 scenario, relocated. The F-112 burndown (owning
phase: post-B2b, before the release tag) reads item 12's own explanatory paragraph — the one that
answers "why is this not a new constraint" — rather than the SCOPE note eleven hundred lines away,
and takes "it is what `gui/wipe_guard.go` already does, since the guard's lifetime *is*
`unlockSecretSession`'s own first and last act" at face value: one bracket, one installer. A later
refactor of `unlockSealedFlow`'s retry loop drops `gui/unlock_kdf.go:136`'s guard install as
apparently-redundant machinery not accounted for by the item's own authoritative explanation. The
operator is on the twelve-word passphrase keyboard at word 7, is interrupted, and walks away —
no timer, no scrub, indefinitely, with the sealed blob beside them. This is the identical harm I1
was written to close, reopened by the one occurrence the fold missed.

**Smallest fix.** Match this paragraph to the SCOPE note's now-correct wording:

> This is not a new constraint on the implementation — it is what the two `wipeGuard` installs
> already do: `unlockSecretSession`'s for the secret-record window (rows 1–3) and
> `unlockPassphraseFlow`'s for the in-flight-passphrase window (row 4), each as that function's own
> first and last act.

---

## Fold-vs-findings, in full

- **I1** (SCOPE note named one bracket, timer has two) — **FIXED**, and re-verified against code:
  `gui/unlock_session.go:89` / `gui/unlock_kdf.go:136` are the only two production `ctx.wipe =`
  sites, matching the new text "installed by `unlockSecretSession` for rows 1–3 and by
  `unlockPassphraseFlow` for row 4" (lines 1373–1376) exactly. See Important 1 above for the one
  place this fix needed to also land and didn't.
- **I2** (carve-out 2 promised a discipline row 3 forbids) — **FIXED**. Current text (lines
  171–177): "Inside that program the discipline follows the **secret**, not the screen: any screen
  reached while a secret record or an in-flight passphrase is resident is inside the bracket,
  however that screen is shared with a legacy flow. Once §10.2.2 has wiped the last secret, §10.2.4
  row 3 applies and there is nothing left to time." This is true and implementable: §10.2.2 forces
  every secret record to be offered and wiped *before* the plate list is built ("plate list:
  mk1/md1 only, no secret resident"), so an Inspect/plate-path screen is by construction reached
  only after residency has already gone to zero — row 3 — never while the timer's precondition
  holds. It answers F-76 unambiguously without arming a timer on the plate list during a legitimate
  multi-minute steel swap, closing exactly the harm round 0 identified.
- **I3** ("the backup/inspect screens" listed among OTHER programs) — **FIXED**. Current
  enumeration (lines 137–138) is the 7-entry list with the five struck words gone; grepped for
  "backup/inspect" across the file post-fold — zero hits.
- **I4** (item 12 physically in §2.2a, not §2.2) — **FIXED**. Item numbering 1–12 is contiguous
  (grepped) and item 12 (lines 134–183) now precedes `### 2.2a` (line 185), which precedes
  `### 2.3` (line 213). `git diff` of the §2.2a body between parent and fold commit is empty except
  for item 12's removal — the moved block is otherwise byte-identical, so no content was lost or
  altered in transit. The back-reference "As **§2.2a** says, the other delivery routes…" (line
  144–145) is accurate: §2.2a's closing paragraph (lines 209–211) does say exactly that. Every
  `§2.2 item 12` citation elsewhere in the file (lines 223, 1379) now resolves correctly since item
  12 is genuinely inside §2.2.
- **Minor** (three divergent enumerations, 8/7/7) — **ADDRESSED, differently than suggested**. The
  fold did not adopt the DRY suggestion ("say 'the legacy programs listed in §2.2 item 12'
  rather than restating"); instead it made the three restated copies textually identical
  ("NFC scan, manual word entry, BIP-85, account xpub, SeedXOR, SLIP-39, free text" — verified
  word-for-word identical at lines 137–138, 1377, and 225). Today's divergence is gone; the
  structural risk round 0 flagged (three hand-maintained copies, next addition updates one or two)
  is unchanged. Not blocking.
- **Nit** (`gui/wipe_guard.go` cited as installing the bracket) — **HALF-FIXED**. The SCOPE-note
  occurrence is corrected (now: "`gui/wipe_guard.go` defines the type"). The item-12 occurrence is
  not — see Important 1, which is this Nit's other half, elevated because it now actively
  contradicts fixed text eleven lines above it rather than merely mis-citing a file.

## New, minor formatting regression (not part of round 0's findings)

**MINOR — the fold merged two wrapped lines into one 172-character line, breaking the paragraph's
established wrap.** §2.3, line 225: `legacy programs — NFC scan, manual word entry, BIP-85,
account xpub, SeedXOR, SLIP-39, free text — leave seed material resident with no timer behind
them, so with those the` is 172 characters; every other prose line in §2.2/§2.3 wraps at roughly
78–84 characters (e.g. lines 137–140, 220–221). Introduced when the fold synced this line's
enumeration wording without re-wrapping. Purely cosmetic — no meaning changes, `plan-cite-gate.sh`
does not check wrap and reports unaffected (2 unresolvable, both pre-existing F-115). Smallest fix:
re-wrap the paragraph at the file's usual width.

## Independently re-verified for this round (not re-derived from round 0)

- `ctx.wipe = ` production sites and the "never nest today" code comments at
  `gui/unlock_session.go:83-91` and `gui/unlock_kdf.go:131-144` — match the fold's I1 text exactly.
- Item numbering 1–12 contiguous, `### 2.2a` immediately follows item 12, `### 2.3` immediately
  follows `### 2.2a`.
- `§2.2a`'s relocated body is byte-identical to its pre-fold content (diffed directly).
- All six `§2.2 item 12` / `§2.2a` cross-references in the file resolve to the correct section
  post-move.
- `./scripts/plan-cite-gate.sh design/SPEC_encrypted_payload_delivery.md`: 2 unresolvable, both
  pre-existing (F-115), matching the commit message and the brief's already-verified note.
- No leftover "backup/inspect" text anywhere in the file.

## Re-review scope if Important 1 is folded

The fix is a single-sentence swap mirroring already-accepted I1 wording elsewhere in the same
file. A re-review should ask only "does item 12's 'boundary is the PROGRAM' paragraph now name both
brackets, and does it still agree with the SCOPE note and carve-out 2" — everything else in this
report is settled and should not be re-derived.
