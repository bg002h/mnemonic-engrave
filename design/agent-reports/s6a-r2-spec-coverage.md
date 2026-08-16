# SPEC-COVERAGE review — `IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`

**Lens:** does this plan contradict, or silently depart from, the specs that
govern the code it changes? First application of this lens to this artifact.

**Reviewer:** sonnet, mechanical/verification tier. Read-only — no files
modified.

## VERDICT: RED — 0 Critical, 1 Important

## SPECS EXAMINED

| spec | sections read | bearing on this plan |
| --- | --- | --- |
| `SPEC_multisig_build_repair.md` | §4.3 (383, per-slot model), §4.4 (D-3, restore-doc naming), §2.2/2.3 (D-3, D-5, restore-doc/verify-offer mentions), §9 (verify-before-funding surface), full-text grep for `watch-only`/`inventory`/`passphrase`/`verify offer`/`status line`/`SUPPLY` | owns S6 (cited by plan §3.2) and the multisig build/supply model the plan's §4.3/§4.7 edit |
| `SPEC_seedhammer_T6a_singlesig_flagship.md` | full file (91 lines): §3 Phase B scope, §4 security spine, §6 acceptance gate (B5), §7 invariants (I-7) | owns the single-sig restore document this plan's §4.2/§4.4/§4.7 changes — **the finding below** |
| `SPEC_systemwide_payloads.md` | §3.3.2 admission table (338-412), the Engrave Single-Sig row (349), the `ClassPassphrase` admission note (357-362) | governs passphrase admission; plan reuses `syswPassphraseFlow` unmodified and never cites this spec — checked for conflict, found none |
| `IMPLEMENTATION_PLAN_multisig_build_repair.md` | §S6 (1300-1321) | cross-check for plan §3.2's claim about what the S6 hardware gate requires |

## C-n / I-n / M-n

### I-1 — the plan changes the single-sig restore document's content without acknowledging its governing spec, which enumerates that content and does not include any of what the plan adds

**Spec says (`SPEC_seedhammer_T6a_singlesig_flagship.md:36`):**

> **restore doc (R0-M2):** display-only + optional NFC; master fp + the
> concrete descriptor + first receive/change address (from-xpub
> `*bip380.Descriptor`, `gui/md1_expand.go:60-77` + `address.Receive/Change`);
> greps clean of any xprv/private material.

This is not a stray comment — it is this spec's own scope-defining bullet for
Phase B ("Scope — IN"), restated as the acceptance gate at line 66 (`B5 —
restore doc: fp + descriptor + first recv/change addr match; greps clean of
xprv`) and as invariant I-7 at line 78 (`restore doc display-only (+optional
NFC), no secret`). All three describe the same four-field document and nothing
else — no plate inventory, no seed-presence statement, no passphrase fact, no
verification status.

**The plan under review confirms, independently, that shipped code still
matches this description exactly** (§1.1, its own measured fact):

> Its screen (`singleSigRestoreLines`, `gui/singlesig_restore.go:97-113`)
> renders exactly four things — master fingerprint, descriptor, first receive
> address, first change address.

**Plan says (§4.2, §4.4, §4.7):** the plan adds, to that same document: a
plate inventory + passphrase statement (§4.2, via `buildPlateInventoryLines`),
a seed-presence statement in three variants (§4.4, `buildSeedInventoryLines`),
and a five-state verification status line prepended ahead of everything else
(§4.7/§4.7a/§4.7b). This is exactly the content T6a's restore-doc bullet does
not mention, added to exactly the document that bullet describes exhaustively.

**The conflict:** the plan's own §0 second rule — "every screen and every
document states what is on the plates *and what is not*" — is the explicit
opposite of T6a's minimal, four-field restore doc. That is very plausibly the
*right* call (it is this cycle's whole point), but the plan never says so. It
cites zero lines of `SPEC_seedhammer_T6a_singlesig_flagship.md` anywhere —
confirmed by grep, zero hits for `T6a`, `SPEC_seedhammer`, or `flagship` in the
plan text — despite an otherwise careful §3.1 "ASSUMPTIONS THIS PLAN MAKES —
declared loudly" section that flags several much smaller departures (e.g. the
"Plate Count"/"Plates To Cut" title mismatch, filed as F-203, a pure wording
Nit). A four-field restore document becoming an N-field one, on the single-sig
flagship spec that this project's own R0/R1 process took to 0C/0I specifically
to lock that content, is a larger departure than F-203 and gets no equivalent
acknowledgment. Left as-is, `SPEC_seedhammer_T6a_singlesig_flagship.md:36/66/78`
reads as the current, accurate spec of the restore document to the next reader
who has not read this plan — exactly the "trap for the next reader" the review
brief describes.

**Not Critical:** nothing in T6a forbids additional non-secret content — I-7's
actual constraint ("no secret") is untouched, and B5's field-presence
assertions ("fp + descriptor + first recv/change addr match") are not falsified
by prepending/appending more true lines. So this is a documentation departure,
not a violated normative "MUST". **Important**, not Critical: it is a real,
measured, unacknowledged departure from a governing spec's own scope
definition for the exact document this plan edits, not merely wording that
will read slightly dated.

**Suggested remedy (not prescribing the shape, per this project's own rule):**
the plan should either (a) add one line to §3.1 naming that
`SPEC_seedhammer_T6a_singlesig_flagship.md:36/66/78` now needs updating to
reflect the enlarged restore document, or (b) explicitly file that spec update
as an owned follow-up, the same way it already does for F-203.

## WHAT I CHECKED AND FOUND CONSISTENT

- **F-196 / per-slot language (`SPEC_multisig_build_repair.md:383`).** The
  spec's normative table reads "Every slot `@0..@{n-1}` carries exactly one
  **source**, chosen by the operator and shown on a review screen before
  assembly" with `payloadKey`/`derived`/`both` as the three sources. F-196's
  entry in `FOLLOWUPS.md:7013-7016` quotes this line correctly and its
  conclusion ("the limit is in the PICKER, not the model... which is why the
  owning phase is the spec and not a stage") is faithful to what the spec
  actually says. The S6a plan's §7 one-line disposition ("F-196 — a model
  change; it earns its own R0 against the spec") is consistent with both.
- **Plan §3.2's claim that the S6 hardware gate requires a multisig
  engrave-and-restore.** Checked against
  `IMPLEMENTATION_PLAN_multisig_build_repair.md:1300-1321`. S6 items 1 and 2
  are literally "Engrave and restore a `wsh` multisig" / "an `sh(wsh)`
  multisig", and item 4 requires restoring "master B's mnemonic... from its
  ms1 plate" as part of the stage's own gate. The plan's citation ("S6 items
  1, 2 and 4") is accurate.
- **`SPEC_systemwide_payloads.md` §3.3.2 passphrase admission.** The admission
  table (line 349) admits `Passph` for "Engrave Single-Sig", consistent with
  the plan's unmodified reuse of `syswPassphraseFlow`. The plan does not cite
  this spec and does not need to — it changes no admission logic, only what
  the already-admitted passphrase fact is later labelled/documented as.
- **`SPEC_multisig_build_repair.md`'s own restore-document coverage.** Unlike
  T6a, this spec never gives an exhaustive content list for the multisig
  restore document — its restore-doc mentions (lines 21, 127, 191, 443) are
  about a naming defect (D-3, nested-segwit vs. legacy P2SH) and reachability,
  not a field inventory. So the plan's §4.7 changes to the multisig restore
  document (the prepended verify status line) have no equivalent governing
  text to fall out of step with. No finding on the multisig side.
- **`gui/multisig.go:321-322` / `gui/multisig_build.go:439`** ("Only
  verifyComplete falls through...") are code comments the plan itself
  identifies and corrects (§4.7b) — the spec makes no matching normative claim
  anywhere, so this is not a spec/plan conflict, just the plan fixing a false
  code comment as it already says it will.
- No spec text (`does not time out`, `mid-build machine`, `plates are the
  secret`, `this build can hold several`/`holds exactly one`) exists anywhere
  in the three governing specs — the seed-handling ruling §4.3 modifies is
  pure code prose with no spec counterpart, so no staleness there.

**Outside my lens** (per brief, not re-litigated): adversarial funds,
executability, test falsifiability, fold-vs-findings coverage were covered by
prior rounds and not re-examined here.
