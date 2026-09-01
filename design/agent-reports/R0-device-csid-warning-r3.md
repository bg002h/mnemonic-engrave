# R0 — SPEC_device_csid_warning.md, round 3 (micro fold-check)

**Artifact:** `design/SPEC_device_csid_warning.md` @ `88f0c71` (fold of r2's sole finding)
**Diff checked:** `git diff 0a4b545..88f0c71 -- design/SPEC_device_csid_warning.md`
**Prior review:** `design/agent-reports/R0-device-csid-warning-r2.md` (0C/1I/0M/0N) @ `0a4b545`
**Ground truth:** `/scratch/code/shibboleth/seedhammer` @ `origin/main` `2337ed3`
**Scope:** single-finding fold-check per project rule — NOT a fresh audit. r2's other
verified-sound items (I2–N1, all CLOSED) taken as settled, not re-derived.

## Verdict

**0 Critical / 0 Important / 1 Minor / 0 Nit — CLOSES.**

r2's sole open finding (I1: the sixth `offerChunkedMK1` consumer, `gui/multisig.go:102`
"Engrave Multisig", missing from Contract 3's enumeration) is discharged correctly and without
introducing a new inconsistency of blocking weight. One Minor gap remains: the Acceptance
section was not updated to name the new disposition's pinning test, breaking the exact-match
property r2 itself verified for the other five surfaces.

## Disposition of r2's I1

**Fixed, and the added text matches r2's traced behavior exactly.** The fold's new clause
(inserted between Build Policy and Verify readbacks, matching the six consumers' original file
order):

> "Engrave Multisig (`multisig.go:102`): NO marker, NO modal — its `extractSuppliedMd1` refuses
> unconditionally on ANY mk1 presence before a card could render (verified r2), so a csid
> warning is unreachable there; silence is correct by prior refusal, and a test pins that
> refusal so the reason cannot rot silently."

Checked against r2's trace, verbatim from r2's report:

> "`extractSuppliedMd1` (`gui/multisig_supply.go:24`) unconditionally refuses when any `cardMK1`
> is present — `case cardMK1, cardMS1: return nil, false // a stray key/secret card pollutes the
> supply.` — regardless of csid match."

and r2's own suggested remedy wording:

> "most likely 'no marker/modal: any mk1 is refused here regardless of csid status
> (`extractSuppliedMd1`), so the mismatch is moot' — mirroring the reasoning already given for
> the verify readbacks."

The fold's text is that remedy, adopted near-verbatim, plus one addition r2 did not ask for: "a
test pins that refusal." I independently re-verified this addition is TRUE, not aspirational —
`gui/multisig_supply_test.go:40-44`, subtest `"any mk1 present -> refuse"` inside
`TestExtractSuppliedMd1`, already exists at baseline `2337ed3` and asserts exactly this:

```go
t.Run("any mk1 present -> refuse", func(t *testing.T) {
    if _, ok := extractSuppliedMd1([]bundleCard{md1A, mk1}); ok {
        t.Fatal("ok=true with a stray mk1, want false (polluted supply)")
    }
})
```

Also re-traced the call sequence at `gui/multisig.go:101-108`: `bundleGatherFlow` returns cards,
`extractSuppliedMd1(cards)` is the immediate next call, and on refusal the function returns via
`showError` before anything past that point runs. No modal-insertion point exists between gather
completion and the unconditional refusal, so "unreachable" and "before a card could render" are
accurate, not overstated.

**All six consumers now enumerated with a disposition:** Engrave Bundle (`bundle_flow.go:45`),
Wallet Policy (`wallet_policy.go:125`), Build Policy (`multisig_build.go:184`), Engrave Multisig
(`multisig.go:102`, new), and the two verify readbacks (`multisig_verify.go:781`,
`singlesig_verify.go:145`) — matching r1's original six-caller table, reconfirmed live in r2 by
grep (`offerChunkedMK1`/`bundleGatherFlow` sites unchanged at `2337ed3`).

## Minor — Acceptance section not updated to name the new pinning test

r2 explicitly checked (its "new-defect/contradiction check #1"): *"Does Acceptance's test list
match Contract 3's per-surface decisions exactly?"* — answered **Yes**, but only because both
sections consistently omitted the same sixth consumer at that time. That parity no longer holds
in a matching way now that Contract 3 has six dispositions and one of them (Engrave Multisig)
carries an explicit test claim ("a test pins that refusal").

The Acceptance section (unchanged by this fold) still reads:

> "gui tests mirror the md1 R0-C1 pattern: the corpus's pinned row strings fire the warning in
> the inspect flow AND in each of the four modal/marker-bearing bundle consumers (Engrave
> Bundle, Wallet Policy, Build Policy incl. census/inventory/payload lines, and the two verify
> readbacks' line-markers); the clean twin is silent everywhere; `clsSingleMK1Refuse` pinned;
> the notice answers BACK and proceeds."

This names `clsSingleMK1Refuse` pinned (Contract 4's unreachable-consumer test) explicitly, but
has no equivalent clause for Contract 3's new Engrave-Multisig disposition's test
(`TestExtractSuppliedMd1`'s `"any mk1 present -> refuse"` subtest). Both tests are pre-existing
at baseline, not new work required by this cycle — so this is a documentation-completeness gap,
not a missing implementation obligation, and not evidence of anything actually silent. Grepped
`design/SPEC_device_csid_warning.md` for `extractSuppliedMd1` and `TestExtractSuppliedMd1`:
the only occurrence is the Contract 3 clause itself; Acceptance never names it.

Not Important: the underlying test already exists, already passes, and Contract 3's own prose
already states the pinning fact inline — an implementer following Contract 3 has the information
needed. It is a checklist-completeness gap in Acceptance, parallel in kind to (and smaller than)
r1/r2's own already-resolved class, so it is recorded rather than gating.

**Suggested remedy (not required for closure):** append to the Acceptance bullet, after
`` `clsSingleMK1Refuse` pinned; `` something like: `` `TestExtractSuppliedMd1`'s any-mk1-refuses
subtest (pre-existing) pins the Engrave Multisig disposition; ``.

## Machine-check log

- `git diff 0a4b545..88f0c71 -- design/SPEC_device_csid_warning.md`: confirmed the fold is
  exactly one clause insertion (5 added lines), no other text touched.
- `grep -n "extractSuppliedMd1\|multisig.go\|refus\|pin" design/SPEC_device_csid_warning.md`:
  located every relevant mention; confirmed Acceptance has no `extractSuppliedMd1` /
  `TestExtractSuppliedMd1` hit.
- `grep -rn "clsSingleMK1Refuse" gui/*.go` (seedhammer @ `2337ed3`): confirmed
  `gui/bundle_test.go:112-116` already pins it — Contract 4's acceptance entry names a
  pre-existing test, same category as the one now missing for Contract 3.
- `grep -rln "extractSuppliedMd1" gui/*.go` + read `gui/multisig_supply_test.go:1-57` in full:
  confirmed `TestExtractSuppliedMd1`'s `"any mk1 present -> refuse"` subtest exists at baseline
  and asserts the exact behavior the fold's new clause claims.
- Read `gui/multisig.go:90-120` in full: confirmed `extractSuppliedMd1(cards)` is the immediate
  next call after `bundleGatherFlow` returns, with unconditional `showError`-and-return on
  refusal — no modal-insertion window exists between gather completion and refusal.
- Tree state: read-only checks only (grep/sed/read), no probe files added; no `git status`
  changes made in the seedhammer fork.
