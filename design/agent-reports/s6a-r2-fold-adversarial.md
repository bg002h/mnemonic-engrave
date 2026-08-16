# S6a R2 — adversarial review of the R1 FOLD's new text

**Scope:** only what `git diff 6b32cb6..HEAD -- design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
added — §3.2 (revised), §4.3's watch-only arm, §4.7 (revised), §4.7a (new),
§4.7b (new), §5 rows T9–T12 and their two rationale paragraphs. Not a fresh
audit of the plan, the fork, or the earlier rounds' findings.

**Fork read at:** `/scratch/code/shibboleth/seedhammer`, branch `main`.

## VERDICT: RED — 2 Critical, 5 Important

---

### C-1 — prepending into `extra` does NOT put the status line on page 1; it moves it from page 5 to the middle of page 4. R1 I-2 is not closed, and §4.7 now asserts a guarantee the specified seam cannot deliver

**Where:** §4.7 ("that line is **the first thing on the page**"), §4.7b
("**PREPENDED** … to the lines already passed through the existing
`extra []string` parameter"), §5 T11.

**The defect.** `extra` is not the document. Both doc flows append it *after* the
whole descriptor block:

    // gui/singlesig_restore.go:130   restoreDocScreen(ctx, th, append(lines, extra...))
    // gui/multisig_restore.go:119    restoreDocScreen(ctx, th, append(lines, extra...))

so prepending *within* `extra` puts the status line first **in the inventory
block**, which is the tail of the document — not first on the pager's page 1.

**Measured, not reasoned.** I ran the shipped supply-path walk, which already
pages the whole restore doc and joins the pages with `" || "`:

    nix develop --command go test ./gui/ -run 'TestSupplyPassphraseRunTellsTheOperatorWhatIsMissing' -v -count=1
    --- PASS (42.57s)

The document is **five pages** at `sh2DisplaySize` (480×320):

| page | content |
| --- | --- |
| 1 | `Type:` / `P2WSH 2-of-2 multisig (sorted)` / `Descriptor:` / first 3 descriptor chunks |
| 2 | descriptor chunks |
| 3 | descriptor chunks + `First receive:` + receive address + `First change:` |
| 4 | change address, **then `extra` begins**: `This backup is 7 plates:` … `If any of them is missing…` |
| 5 | passphrase lines, seed-handling ruling |

`extra` starts **part-way down page 4 of 5**. A status line prepended into it
lands mid-page-4. Meanwhile `restoreDocScreen`'s Done key is live before any
paging happens — `doneBtn := &Clickable{Button: Button3, AltButton: Center}` and
`if backBtn.Clicked(ctx) || doneBtn.Clicked(ctx) { return }` sit at the top of
the frame loop, so the operator exits from page 1 without ever seeing pages 2–5.
The single-sig document is shorter but not short enough: `Master fp:` +
`Descriptor:` + ~7 chunks + `First receive:`/addr + `First change:`/addr is ≥13
lines against the ~6 lines page 1 held above, so `extra` cannot start before
page 3 there either.

**The plan's own §5.2 already says this**, in text this fold did not touch:
*"`restoreDocScreen` is a **pager** and §4.2 appends `extra` after the descriptor
chunks and both addresses, so the inventory lands on the last page(s)."* §4.7's
new "first thing on the page" contradicts §5.2 in the same document.

**The cited precedent refutes the remedy rather than supporting it.**
`gui/multisig.go:271` prepends the collapse note into `census`, and `census` **is
the entire body** of `confirmReviewScreen`, so there the prepend really is page 1.
The comment's own stated reason, quoted in the fold, is the argument *against*
this seam: *"a note on page three is a note the operator can commit past without
reading."* Here it is a note on page four.

**The harm.** This is R1 I-2's harm verbatim, delivered by I-2's own remedy. The
`WARNING: a read-back check DISAGREED … Do NOT rely on this backup.` line sits
three page-presses behind a Done key the operator can hit immediately. §4.7's
governing sentence is *"silence must never be mistakable for a pass"*; the
specified implementation is silence for any operator who does not page. The plan
now also states the false guarantee as settled fact, so the next reader inherits
it as a given.

**Second-order:** **T11 is unsatisfiable as written.** *"the status line is the
first line of the document"* fails against the specified implementation, and its
stated mutation (*append instead of prepend*) is undetectable by that assertion
because both spellings fail it. The only version of T11 that can pass is the
weakened one — "ahead of the plate inventory" — which is precisely the property
that does not buy the guarantee.

**Third-order:** §4.7b's "**This means NO signature change on the multisig side
at all**", and §3.2's newly-"measured" affordability claim, both ride on this
seam. If the status line must precede `lines`, at least one of the two doc flows
takes a new parameter and the affordability sentence is an estimate again.

**Suggested remedy (UNVERIFIED — I did not resolve the call graph for it):**
the status line has to enter *ahead of* `lines` inside `restoreDocFlow` /
`multisigRestoreDocFlow`, not through `extra`. Whatever shape that takes,
§4.7's "first thing on the page", §4.7b's "no signature change", §3.2's
affordability paragraph and T11 must be re-decided together — they are one
claim, and it is currently wrong in all four places.

---

### C-2 — the sticky rule prints `DID NOT COMPLETE` over a verify that DID complete, on the sequence the device itself instructs the operator to perform

**Where:** §4.7a's ranking (`disagreed > did-not-complete > not-verified >
verified-on-retry > verified`) and its single exception.

**The defect.** The exception is carved for **one** recovered sequence
(`failed → complete`). `verifyIncomplete` is the *other* verdict that keeps the
loop alive — `if res != verifyIncomplete && res != verifyFailed { break }` — so
`incomplete → complete` is equally reachable, and the rule as written takes
`max(did-not-complete, verified) = did-not-complete`.

That is not a corner. `multisigVerifyIncompleteText` (`gui/multisig_verify.go:482-489`)
is the screen the operator reads on an incomplete, and it says:

> *"Choose VERIFY AGAIN on the next screen and type ALL of this wallet's seeds in
> one pass; a new attempt keeps nothing from this one. Until then, do not fund
> this wallet."*

An operator who obeys that instruction reaches `verifyComplete`, which
`gui/multisig_verify.go:88-90` defines as *"the only clean pass: every expected
slot got a leg, every leg found its plate, and the bijection closed."* Their
document then reads **"Plate verification DID NOT COMPLETE. Confirm they restore
before relying on this backup."**

The commonest way in is the most mundane failure there is: the NFC read misses
one plate, `len(readbackMk1s) != len(expectedSlots)`
(`gui/multisig_verify.go:936`) returns `verifyIncomplete`, the operator taps all
the plates on the second pass and gets a full clean verify.

**The harm.**

1. **A false statement on the durable artifact.** The verification completed. The
   document says it did not. That is the class §4.3 and §4.4 spend two sections
   eliminating.
2. **It silently overrides the persisted operator ruling.**
   `s6a-c1-verify-tail-decision.md` binds *"Clean pass: `Plates VERIFIED: …`"*.
   The fold flags its **fifth state** as *"a controller decision derived from the
   persisted C-1 principles, not a new operator ruling"* — but does not flag that
   the same ranking silently re-assigns the VERIFIED state for this sequence.
   The undeclared departure is the one nobody was asked to bless.
3. **It re-creates the fold's own named trap.** §4.7a's justification for the
   fifth state is *"never make running the check the way to lose something."*
   Under this ranking, an operator who skips outright gets `NOT VERIFIED`, which
   the rule ranks **below** `did-not-complete` — so pressing VERIFY AGAIN and
   passing cleanly yields a *worse* line than never verifying at all.

**Suggested remedy (UNVERIFIED):** the exception is not "a disagreement cleared
by a clean pass" but "any non-clean observation cleared by a clean pass", and it
needs a decision about whether `incomplete → complete` prints bare `VERIFIED`
(the incomplete proved nothing adverse about any plate — what was compared
*matched*, per `multisigVerifyIncompleteText`) or the repeat-check line. Note
`refused → complete` and `abandoned → complete` are **unreachable** — both break
the loop — so this is the only other case to decide.

---

### I-1 — §3.2's "superset" is false: `multisigVerifyResult` has no state for "skipped / never offered", and §4.7b's `verifyStatus` is never defined

**Where:** §3.2 ("`multisigVerifyResult` already exists with **five** constants …
which is a superset of what the status line distinguishes"; "**neither multisig
call site changes signature**") vs §4.7b (`func buildVerifyStatusLines(v verifyStatus) []string`).

**The defect.** The status line distinguishes five *outcomes*; the verdict type
has five *constants*; they are not the same five. Measured —
`grep -rn "verifyStatus\|verifySkipped\|buildVerifyStatusLines" --include="*.go" .`
returns **zero hits**, so `verifyStatus` is entirely new, and
`multisigVerifyResult` (`gui/multisig_verify.go:87-100`) contains **no value for
"the verify was never run"** and none for "verified on retry". Both are
status states §4.7a's table requires. A five-constant enum missing two of the
five states it must express is not a superset of them.

The fold corrected "four constants" to "five constants" and kept the affordability
sentence resting on a count, when the load-bearing property was never the count —
it is whether the existing type can *represent the status*. It cannot.

**The harm.** An implementer told by §3.2 that the type already exists and nothing
changes shape will carry `multisigVerifyResult` to the document and then need a
value for the Skip button. The available ones are `verifyRefused` and
`verifyAbandoned`, both of which §4.7a maps to `DID NOT COMPLETE` — so an
operator who simply pressed **Skip** gets *"Plate verification DID NOT
COMPLETE"* on the artifact instead of `NOT VERIFIED`, inverting the rule's own
ranking on the single commonest path in the whole section. Nothing in T9–T12
distinguishes those two lines end-to-end.

---

### I-2 — the single-sig exit→verdict mapping is still unspecified, and "mirroring `multisigVerifyResult`'s shape" makes the PASS the zero value

**Where:** §4.7b, final paragraph ("**Single-sig** gets a verdict type mirroring
`multisigVerifyResult`'s shape, and `engraveSingleSigFlow` threads the worst-seen
outcome …").

**The defect.** R1 I-1 raised two things. §4.7b closes the first (one seam) and
leaves the second untouched: *"§4.7 says single-sig 'gets a result type mirroring
`multisigVerifyResult`'s shape' but gives no return→verdict mapping … the mapping
in between is where a FAILED becomes a NOT VERIFIED."* The fold rewrote the
sentence (`result type` → `verdict type`, `the outcome (or "skipped")` →
`the worst-seen outcome`) without adding the mapping.

`singleSigVerifyFlow` (`gui/singlesig_verify.go:65-149`) is a bare `func(...)`
with **eleven** exits and no verdict at all today: `:66` seed entry Back, `:71`
script pick Back, `:90` derive error, `:96` templateize error, `:112` gather
Back, `:118` readback-cards refusal, `:124` ms1 keyboard Back, `:130` not-an-ms1,
`:138` bad-ms1, `:145` **the failed comparison**, `:148` success. The plan says
nothing about which of the ten non-success exits is `failed` versus `refused`
versus `abandoned` — and `:145` is the one the whole cycle exists for:

    showError(ctx, th, "Verify Failed", "The read-back bundle does NOT match the seed. Check the engraved plates.")

If that exit is not mapped to the DISAGREED line, the single-sig document prints
`DID NOT COMPLETE` over plates the device just said do not match the seed —
verbatim the original C-1 harm, on the original C-1 path. No T-row in §5 asserts
this mapping (see I-3).

**And the mirrored shape's default is the dangerous one.** `verifyComplete` is
`iota` — **zero**. A single-sig type "mirroring that shape" makes the clean pass
the zero value, so any exit that misses an explicit assignment (a named result
parameter, a struct field, a var declared above the branch) yields a document
saying `Plates VERIFIED` over an unverified or failed set. The plan neither warns
about this nor specifies the constant order.

Also, "threads the **worst-seen** outcome" is stated for a path with **no retry
loop** — `if sel, ok := verifyChoice.Choose(...); ok && sel == 0 { singleSigVerifyFlow(...) }`
(`gui/singlesig.go:130-133`) runs at most once, so "worst-seen" ranges over one
observation. Harmless in itself; it is the tell that the rule was written for the
loop path and transcribed to the one that has none.

---

### I-3 — T9–T12 pin no production call site, and T9's stated assertion overclaims what its stated mechanism can prove

**Where:** §5, rows T9–T12; the "T9–T12 exist because…" paragraph.

**The defect.** T9's assert column is *"each of the five §4.7a outcomes renders
its own line, **and every rendered document carries exactly one status line** —
asserted over `buildVerifyStatusLines` **directly**"*. A unit assertion on the
builder proves the builder returns one distinct line per state. It cannot prove
anything about a *document*, because whether a document carries the line depends
on three production call sites — `gui/singlesig.go:136`, `gui/multisig.go:361`,
`gui/multisig_build.go:478` — none of which T9–T12 names. T10 and T12 are
sequence assertions over the sticky rule; T11 says "of the document" but names no
path and cites no pager helper.

R1 C-2's harm statement was explicit: *"an implementer who wires two of three
ships a green suite and a device that is silent on one path — which is exactly
the state F-197's own follow-up warns about."* That is still true after this
fold. Wiring single-sig and the build path and forgetting `gui/multisig.go:361`
passes T9, T10, T11 and T12.

The gap is cheap to close and the plan knows how: the supply path already has a
walk that pages the whole restore doc
(`TestSupplyPassphraseRunTellsTheOperatorWhatIsMissing`, via `s5PageForNeedle`),
and §5.2 exists precisely to record per-test costs of that kind. The fold added
four rows and **no §5.2 bullet for any of them**, so the costs a document-level
status test faces on the multisig paths (`newEngraver()`, `sh2DisplaySize`,
`s5EngraveOnePlate` per plate, the pager) are unstated for exactly the four tests
that would hit them.

**Also, on T10's own falsifiability:** T10 *does* detect its stated mutation —
under last-wins, `failed → abandoned` yields `DID NOT COMPLETE ≠ DISAGREED` —
but only if the test can drive the sequence. The retry loop is drivable through
`multisigVerifyFn`, the in-file seam; single-sig has **no loop and no seam**, so
`failed → abandoned` is unreachable there. The new tests go in
`gui/singlesig_truth_test.go` and T10 names no path.

---

### I-4 — "there are TWO of them" is wrong: a THIRD site carries the same false claim, on the very type the fix threads

**Where:** §4.7b, *"**Both false comments are corrected, and there are TWO of
them (R1-A).**"* and its two-row table.

**The defect.** Measured —
`grep -rn "falls through\|fall through" --include="*.go" gui/` — the claim
appears at **three** sites:

| site | text |
| --- | --- |
| `gui/multisig_build.go:439` | `Only verifyComplete falls through to the restore document.` *(in the table)* |
| `gui/multisig.go:322` | `verifyComplete falls through; a refusal or an abandon does not loop` *(in the table)* |
| **`gui/multisig_verify.go:79`** | **`Only verifyComplete may fall through to the restore document.`** *(not in the plan anywhere)* |

The missed one is the doc comment on `multisigVerifyResult` itself — the type
§3.2 says the whole multisig fix reuses. Its neighbouring line
(`gui/multisig_verify.go:78`) reads **`FOUR OUTCOMES, NOT A BOOL`** over five
constants: that is the exact sentence §3.2 says the first fold read instead of
counting, and the fold corrected the plan's copy of the error while leaving the
source of it in place.

**The harm.** §4.7b's own rule is *"Neither may survive a cycle that fixes the
behaviour they misdescribe."* An implementer working the two-row table
mechanically leaves the third standing, and the type's doc comment is the one a
future reader hits first. The fold diagnosed the prior round's failure as
citing from memory rather than measuring, then stated an exhaustive count without
running the grep that settles it — the same move, one round later. No T-row
covers this either; T8's source-assertion pattern exists and is applied only to
`gui/bundle_flow.go`.

---

### I-5 — §4.3's new watch-only clause "the words you typed" is false on every payload-sourced run

**Where:** §4.3's revised watch-only arm and the paragraph justifying it.

**The defect.** The fold replaced *"it is holding your seed"* with *"it is still
holding **the words you typed**"* to fix a singular/plural wobble, and audited
exactly one axis — count. The new clause adds a **provenance** claim the old one
did not make, and the seed is not always typed:

    // gui/singlesig.go:39-41
    // The seed, through the ONE seam (D12). seedEntryFlow offers every source this
    // machine has; it is not keyboard-only.
    mnemonic, ok := seedEntryFlow(ctx, th)

`seedEntryFlowTitled` (`gui/derive_xpub.go:104-125`) routes to
`seedEntryFlowTypedOnlyTitled` **only** when `src == srcTyped`; every other
source returns `m, true` directly — a sysw payload record, a scan. (Contrast
`singleSigVerifyFlow`, which deliberately calls `seedEntryFlowTypedOnly`; the
engrave path does not.) An operator who loads the seed from a payload card and
runs a watch-only engrave gets a durable document telling them the machine is
holding *words they typed*, which they did not.

The same sentence now carries two different provenance verbs four words apart —
the shipped subject clause says *"The seed you **entered**"* / *"Every seed you
**entered**"*, and the new tail says *"the words you **typed**"*.

**The harm.** It is a warning sentence, and the falsity is exactly the kind a
reader uses to decide the warning is not about them: *I loaded mine from a card,
so this does not apply* — and leaves a machine holding a seed unattended. Lower
stakes than C-1/C-2, but it is the §1.3 landmine class committed inside §4.3,
the section that exists to demonstrate auditing a shared string clause by clause
across every mode. The clause it replaced was true on every path.

**Suggested remedy (UNVERIFIED):** "it is still holding the seed you entered"
matches the subject clause's own verb, is count-neutral, and makes no provenance
claim the flow can falsify.

---

## Minor / Nit (recorded, not gating)

- **M-1 — the ranking is written as a total order over observations, but
  `verified-on-retry` is not an observable outcome.** No verdict, and no
  never-offered state, produces it; it is a predicate over the *sequence* (an
  earlier `failed` plus a final `complete`). An implementer who codes §4.7a's
  one-line spec literally — keep the max of the outcomes observed, over the
  five-element order — gets `failed → complete` = `disagreed` and the fifth state
  is dead code. T12 catches it, which is why this is Minor rather than
  Important, but the ranking and the exception are two mechanisms presented as
  one.
- **M-2 — the DISAGREED line prescribes a retry single-sig cannot perform.**
  *"Re-verify or re-engrave"* — single-sig offers the verify once
  (`gui/singlesig.go:130-133`, no loop), and per `gui/multisig_verify.go:100-106`
  the device has no standalone bundle verify at all, so "re-verify" means
  "re-engrave every plate". That is the shape of the defect I-4 fixed for
  multisig (a screen prescribing a retry with no implementation). The string is
  the operator's verbatim text and is not re-litigated here; recorded because the
  fold *did* revise the VERIFIED string in this round, so the strings are
  evidently in play.
- **Nit — §3.2's "neither multisig call site changes signature"** is true of the
  seam as specified and probably false after C-1 is fixed; noted so it is
  re-checked rather than inherited.

---

## THE SEQUENCE TABLE

Every path through the two retry loops (`gui/multisig.go:330-343`,
`gui/multisig_build.go:446-459`), plus the single-sig one-shot. `res` values are
`multisigVerifyResult`. The loop breaks on `!ok || sel != 0` (Skip / Back /
CONTINUE) and on any `res` other than `verifyIncomplete` / `verifyFailed`, so
**`complete`, `refused` and `abandoned` are terminal** and only `incomplete` and
`failed` re-offer.

| # | path | operator sequence | observations | §4.7a prints | ok? |
| --- | --- | --- | --- | --- | --- |
| 1 | both ms | Skip at the first offer | — | `NOT VERIFIED` | ✓ |
| 2 | both ms | Back at the first offer (`!ok`) | — | `NOT VERIFIED` | ✓ |
| 3 | both ms | Verify now → complete | complete | `VERIFIED` | ✓ |
| 4 | both ms | Verify now → refused (no md1 read back) | refused | `DID NOT COMPLETE` | ✓ |
| 5 | both ms | Verify now → abandoned (Back at the gather) | abandoned | `DID NOT COMPLETE` | ✓ |
| 6 | both ms | Verify now → **failed** → CONTINUE | failed | `DISAGREED` | ✓ **(R0 C-1's case — fixed)** |
| 7 | both ms | failed → VERIFY AGAIN → abandoned | failed, abandoned | `DISAGREED` | ✓ **(R1 C-1's case — fixed; T10)** |
| 8 | both ms | failed → VERIFY AGAIN → refused | failed, refused | `DISAGREED` | ✓ |
| 9 | both ms | failed → VERIFY AGAIN → incomplete → CONTINUE | failed, incomplete | `DISAGREED` | ✓ |
| 10 | both ms | failed → VERIFY AGAIN → **complete** | failed, complete | `VERIFIED on a repeat check…` | ✓ **(the fifth state; T12)** |
| 11 | both ms | incomplete → CONTINUE | incomplete | `DID NOT COMPLETE` | ✓ |
| 12 | both ms | incomplete → VERIFY AGAIN → abandoned | incomplete, abandoned | `DID NOT COMPLETE` | ✓ |
| 13 | both ms | incomplete → VERIFY AGAIN → failed → CONTINUE | incomplete, failed | `DISAGREED` | ✓ |
| 14 | both ms | **incomplete → VERIFY AGAIN → complete** | incomplete, complete | **`DID NOT COMPLETE`** | ✗ **C-2** |
| 15 | both ms | incomplete → VERIFY AGAIN → failed → VERIFY AGAIN → complete | incomplete, failed, complete | `VERIFIED on a repeat check…` | ✓ |
| 16 | both ms | failed → VERIFY AGAIN → incomplete → VERIFY AGAIN → complete | failed, incomplete, complete | `VERIFIED on a repeat check…` | ✓ |
| 17 | ms | refused → (terminal) → complete | — | unreachable: `refused` breaks the loop | n/a |
| 18 | ms | abandoned → (terminal) → complete | — | unreachable: `abandoned` breaks the loop | n/a |
| 19 | ms build | `template` engrave | — | no document at all (`if !template`) | ✓ per §3.2 |
| 20 | ms build | `!template && len(legs) == 0` | — | doc renders, verify never offered → `NOT VERIFIED` | ✓ (guard is `if !template && len(legs) > 0`; `errBuildNoHeldSlot` makes it unreachable today) |
| 21 | single-sig | Skip / Back at the one-shot offer | — | `NOT VERIFIED` | ✓ |
| 22 | single-sig | verify runs → `:148` success | — | `VERIFIED` | ✓ |
| 23 | single-sig | verify runs → **`:145` comparison fails** | — | **unspecified** — the plan never maps the exit | ✗ **I-2** |
| 24 | single-sig | verify runs → any of `:66, :71, :90, :96, :112, :118, :124, :130, :138` | — | **unspecified**; zero-value default is `VERIFIED` | ✗ **I-2** |
| 25 | any | the status line's **page** on any of the above | — | mid-page 4 of 5 (measured), not page 1 | ✗ **C-1** |

Two things this table says that the plan does not. First, the sticky rule is a
genuine fix: rows 6–9 and 13 are all the sequences R0 C-1 and R1 C-1 were about,
and every one of them now prints the warning. Second, the ranking's only wrong
answer is row 14, and it is wrong in the *understating* direction — which is why
it reads as safe and is not: it is a false sentence on the artifact, and it is
the one the device's own incomplete screen sends the operator toward.

---

## WHAT I CHECKED AND FOUND SOUND

- **The sticky rule closes R1 C-1, completely.** Every reachable sequence in
  which a comparison disagreed and was not later cleared prints the DISAGREED
  warning (rows 6–9, 13). Under the superseded "last verdict" spelling, rows 7,
  8, 9 and 13 would all have printed `DID NOT COMPLETE`. The diagnosis in §4.7a
  is exactly right about which verdicts keep the loop alive.
- **The two loops really are identical in the respect that matters.**
  `gui/multisig.go:330-343` and `gui/multisig_build.go:446-459` both break on
  `!ok || sel != 0` and both break on `res != verifyIncomplete && res != verifyFailed`,
  so one rule serves both. The build path's extra `len(legs) > 0` conjunct means
  the *variable* must be hoisted above the `if`, not merely above the `for` — not
  a defect in the fold's text, but the one place "hoist it out of the loop" is
  not literally sufficient.
- **The fifth state's trigger is well-defined and reachable.** Because
  `verifyComplete` breaks the loop, a clean pass is always the *last*
  observation, so "a disagreement followed by a clean pass on a repeat check"
  reduces to "the last observation is `complete` and an earlier one was `failed`"
  — decidable with one bool and no ordering subtlety. Its incentive argument is
  the operator decision's own, correctly applied.
- **The "clean pass on other plates" attack does not land.** I checked whether a
  repeat pass could clear a disagreement without being about the same steel:
  `!slices.Equal(readbackMd1, engravedMd1)` (`gui/multisig_verify.go:918`) pins
  the readback to this run's engraved md1 chunks by exact bytes, and
  `expectedSlots` is the tail's own return, so a `complete` on the second pass is
  a statement about the plates the document describes.
- **§3.2's template carve-out is right.** `gui/multisig_build.go` guards the
  whole doc block with `if !template`, so "wherever a document renders" is the
  correct form of the claim and the first fold's "always renders" was false
  there. Single-sig does *not* carry that guard — `restoreDocFlow` runs
  unconditionally after the verify offer — so a single-sig template engrave does
  render a document and does need a status line, which the revised wording
  admits.
- **§4.7a's "matched", not "matched the seed" (R1 I-3) is a clean fix.** It
  removes the singular that contradicted §4.4's "YOUR seeds" on a multi-master
  build, and it costs nothing in the other modes.
- **The one-seam decision keeps §5.1(a) honest.** Because the status rides
  `extra` rather than a third `buildPlateInventoryLines` parameter, §5.1(a)'s
  "all gain **a** capacity argument" (singular) stays true and the six existing
  test call sites do not each have to choose a verdict. That half of R1 I-1 is
  properly closed — it is only the *position* consequence (C-1) and the
  single-sig mapping (I-2) that are not.
- **The two false comments the plan names are genuinely false**, including
  `gui/multisig.go:321-322`, which is self-contradictory on its own terms:
  "Only verifyComplete falls through; a refusal or an abandon does not loop" —
  if a refusal does not loop, it falls through. The R1 fold-check's "arguably
  true as written" reading is the weaker one; the fold is right to correct both.
  The count is what is wrong (I-4), not the classification.
- **Gates re-run by me:** `./scripts/plan-cite-check.sh` and
  `./scripts/plan-glyph-check.sh` were not re-run — the brief states both are
  clean at 83/83 and 45/45 and that is not re-derived here. I did re-run the fork
  test named above, which passed.

I did **not** audit the plan's unchanged sections, the earlier rounds' findings,
the codebase at large, prose, or line numbers.
