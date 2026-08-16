# S6a C-1 decision: the restore document at the verify tail

Operator stand-in decision, 2026-08-16. Scope: what happens to the restore
document when the verify did not cleanly pass, and whether the fix extends to
the multisig paths this cycle. Not a review; not a plan edit.

## DECISION

Option (b), uniformly across all three states: the restore document always
renders, and it always carries exactly ONE verification status line as
normative document content -- VERIFIED after a clean pass, NOT VERIFIED when
the verify was skipped or never offered (state i), DID NOT COMPLETE when it ran
but was incomplete/refused/abandoned (state iii), and a strong DISAGREED
warning when a comparison failed (state ii). No state gates the document.

## WHY

The Critical is that the document vouches against the device's own evidence.
There are two cures: silence (gate the doc) or honesty (put the evidence on the
doc). The operator's standing directive picks honesty -- permissive on input,
expressive on output -- and the incentives agree. Any gate keyed to a FAILED
verify makes the honest path strictly worse than the lazy one: the operator who
wants the document (the only screen with descriptor, fingerprint, and
addresses) learns to SKIP the verify to keep it. Never make running the check
the way to lose something. A hard gate (a) is worse still: it deletes an
existing capability for the common skip case and forces a full seed re-type --
itself error-prone -- as the toll for seeing the descriptor.

For the operator at the machine: a FAILED verify is evidence, not proof. The
comparison seed was re-typed by hand and the plates were read over NFC; a typo
or a bad read produces FAILED on good steel. Gating on a possibly spurious
signal blocks a true document; printing the signal on the document loses
nothing and misleads no one. The on-screen FAILED + CONTINUE choice remains
the acknowledgment; the document is where the acknowledgment is remembered.

For the stranger reading the record years later: a document that says the
plates were never verified, or that a read-back DISAGREED, is strictly more
useful than no document (which says nothing) and categorically safer than a
vouching document (which lies). The wallet facts on the page -- descriptor,
fingerprint, first addresses -- derive from the seed the operator typed, not
from the plates, so they stay true even when the plates are wrong; destroying
them along with the vouch throws away the part of the page that could still
rescue the restore. The inventory lines stay too, framed by the status line.

The status line is always present, including VERIFIED on a clean pass. A
document with no status line must be unrepresentable, so silence can never be
mistaken for a pass, and a pass is positive information the stranger deserves.

## TEXT (verbatim, ASCII only)

Exactly one of these renders on every restore document:

- Clean pass:
  `Plates VERIFIED: each plate was read back and matched the seed.`
- Skipped or never offered:
  `Plates NOT VERIFIED. Confirm they restore before relying on this backup.`
- Incomplete, refused, or abandoned:
  `Plate verification DID NOT COMPLETE. Confirm they restore before relying on this backup.`
- Failed comparison:
  `WARNING: a read-back check DISAGREED with these plates. Do NOT rely on this backup. Re-verify or re-engrave.`

## SCOPE DECISION

The multisig paths are fixed in this cycle, not filed as a follow-up. Two
reasons. First, the next phase is a hardware flash where an operator cuts real
backups, so there is no later phase this item could be parked on without
shipping a known funds hole into exactly the territory it endangers -- under
the project's own rule it is not deferrable past that boundary. Second, choice
(b) makes the multisig cost small: multisig already has the 5-value verdict
type, so the fix is threading the last verdict (or "skipped") into its restore
document and rendering the same four lines -- no restructuring of the retry
loop -- plus correcting the false comment at gui/multisig.go:323, which must
not survive a cycle that fixes the behavior it misdescribes. If the multisig
side balloons anyway, the fallback is that it becomes a named gate on the
hardware flash, never a batched follow-up.

## WHAT I AM ASSUMING

1. The descriptor, master fingerprint, and addresses on the restore document
   are derived from the operator-typed seed held in memory, not from reading
   the plates back. If they were plate-derived, the FAILED case would need
   rethinking.
2. A FAILED verify can be caused by a re-typed-seed typo or an NFC misread as
   well as by miscut plates; it is evidence of mismatch, not proof of bad
   steel.
3. The restore document persists only through the operator transcribing or
   photographing it, so the status line must live ON the document; a transient
   screen warning is not remembered by the artifact.
4. The status line is normative content: every restore document renders exactly
   one of the four lines, and a document with none is a defect, not a default.
5. The on-screen FAILED + CONTINUE prompt remains the operator's explicit
   acknowledgment; the document records the outcome but does not replace that
   choice.
6. The multisig in-cycle scope assumes the existing multisigVerifyResult
   verdict can be carried to the restore document without touching the retry
   loop's control flow.
7. The hardware-flash phase may include real multisig backups, which is why
   the multisig hole cannot defer past this cycle.
8. F-197 stands untouched: an aborted engrave still ends with no verify offer
   and no restore document; this decision covers only fully cut sets.
