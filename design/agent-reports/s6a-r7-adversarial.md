# S6a round 7 — INDEPENDENT ADVERSARIAL review of the fold

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md` @ HEAD
**Code:** `/scratch/code/shibboleth/seedhammer` @ `b8a23bf` (fork `main`)
**Question:** is this plan now safe to implement — and if not, what breaks?
**Answers:** `s6a-r6-pre-verify.md`, `s6a-r6-reader.md`, `s6a-blindspot-pass.md`
**Focus:** §4.7c–§4.7f (all new, adversarially unreviewed), with §4.7e's
observation table as the primary target.

Every `file:line` below was resolved by `sed` against the working tree during
this review, not transcribed from the plan or from an earlier report.

---

## VERDICT: RED — 3 Critical, 3 Important

---

### C-1 — the two PASS lines claim the ms1 plate was read back. It never is, on any run, by design

**Where:** §4.7d line-table rows 1 and 2 (plan `:949`, `:950`); §4.7d
knowledge-state row 1 (`:790`); §4.7e observation-table row 2 (`:1011`).
Code: `gui/singlesig_verify.go:23-42` (`singleSigReadbackCards`),
`:120-142`, `gui/singlesig_engrave.go:20-45`, `gui/multisig_verify.go:184-187`,
`bundle/verify.go:76-104`.

**The defect.** The plan writes, verbatim, as the highest-stakes line on a
durable document:

> `Plates VERIFIED: each plate was read back and matched.`
> `Plates VERIFIED on a repeat check. An earlier read-back disagreed; a later check read every plate and matched.`

A full single-sig run cuts **three** plates — `singleSigEngraveCards(b, full)`
prepends `cardMS1` at `gui/singlesig_engrave.go:22-29`, and the reader-lens
document (Case 1) lists all three. The verify reads back **two**:
`singleSigReadbackCards` switches only on `cardMK1` and `cardMD1`
(`gui/singlesig_verify.go:25-36`), and the ms1 is **hand-typed, never NFC**
(`:120-142`, the flow header at `:14-15`). The comparand for the ms1 leg is
therefore the string the operator typed, compared against entropy re-derived
from the seed they typed (`bundle/verify.go:82-96`). Nothing about the ms1
**plate** is observed. Identical on the multisig path (`multisigVerifyMS1Entry`,
`gui/multisig_verify.go:1004`; `verifyMultisig` at `:184-187`).

So the observation "every leg matched its plate" has

    W = { (a) all three plates encode this run's intent;
          (b) the mk1 and md1 plates do, and the ms1 plate is blank, miscut, or
              a different share — the operator typed a correct ms1 from the
              wallet rather than from that plate }

**|W| = 2, not 1**, and §4.7e row 2 carries a line asserting member (a). That is
a direct violation of the table's own mechanical rule two paragraphs below it
("a row with `|W| > 1` may not carry a line that asserts a single member of W"),
committed by the row that defines a clean pass.

**This is the plan's own provenance insight applied on one side only.** §4.7e
row 7 (`:1016`) marks the hand-typed ms1 `|W| = 2` *"solely because its comparand
was typed by a human rather than read from a plate"*, and §4.7e `:1028-1032`
calls provenance "a first-class column, not a detail". The plan then forgets that
column on the **agreement** side, where the same provenance means the ms1 plate
was never evidence at all.

**The codebase already refuses to make this claim, at screen level.**
`multisigVerifyOKMessage`'s doc comment (`gui/multisig_verify.go:1034-1038`) is
explicit: *"IT DOES NOT CLAIM THE SEED PLATES WERE COUNTED. Only what the
operator typed was compared: a secret is never read back over the reader (§7.4),
so the device cannot know how many ms1 plates exist and must not imply it checked
them all."* The shipped screen therefore says *"All %d operator key plates
verified, and the ms1 **you typed** for each seed."* The plan re-commits at
**document** level the exact error the code warns about at **screen** level —
which is §4.7d's own argument (plan `:834-838`), verbatim, at a different line.
The plan even notices the neighbouring case in §8 blind spot 8 (`:1566-1577`,
"True and incomplete, on the most vouching sentence in the flow") and declines to
fix it because it is inherited — but the status line is **new text this cycle
authors**, and it over-claims harder: "each plate was read back".

**The harm.** A full single-sig run whose ms1 plate is blank, half-cut, or
carries the wrong share produces a durable page reading `Plates VERIFIED: each
plate was read back and matched.` directly above `This backup is 3 plates: ms1
secret share: 1 plate …`. The one plate that holds the money is the one plate the
device never looked at, and the document says it did. That is F-198's shape at
the highest-value plate, authored by the cycle that exists to close F-198. It is
also the only status under which §4.7f renders no scoping line, so nothing later
on the page walks it back.

**Suggested remedy (UNVERIFIED):** scope the two pass lines to what was actually
read — mirroring `multisigVerifyOKMessage`'s split — e.g. `Plates VERIFIED: the
key and descriptor plates were read back and matched. The ms1 you typed matched
this seed; no ms1 plate was read.` A correctly **scoped** line is true in both
members of W and satisfies P4 without needing `statusUnaccounted`. §4.7e row 2's
`|W|` becomes 2 and §4.7d row 1's *"a comparison over plate-derived bytes
completed for every plate"* must lose "every plate". Note this also falsifies
§4.7e `:1024-1026` ("it is the only line that can be true of a two-world
observation") — scoping is the other way, and it is the right one here.

---

### C-2 — two reachable adverse observations are absent from §4.7e, and §4.7d affirmatively routes them to `statusDidNotComplete`

**Where:** §4.7e (`:1008-1018`), §4.7d row 4 (`:793`), §4.7a's switch (`:744-749`),
§4.7f's condition (`:985`). Code: `gui/multisig_verify.go:698-702`, `:732-739`;
`gui/singlesig_verify.go:114-118`.

**The defect.** The table claims one row per **return path**. Two return paths
that observe the plates have no row:

| line | returns | screen |
| --- | --- | --- |
| `gui/multisig_verify.go:701` | `verifyRefused` | `Read back one wallet-policy md1 AND the operator key card(s) (mk1).` |
| `gui/multisig_verify.go:738` | `verifyIncomplete` | `Read back N key plates, but this run engraved M. Present exactly the plates this run cut.` |

(single-sig carries the first at `gui/singlesig_verify.go:116`.)

Both are **adverse and ambiguous**, and `:738`'s W is *the same W the plan
already declares two-world at another site*:

    :738  W = { (a) a plate was not presented (mislaid, forgotten);
                (b) an extra plate from another run is on the bench;
                (c) a plate THIS RUN CUT is miscut/garbled and did not classify
                    as an mk1, so the reader produced fewer cards than plates }

World (c) is a bad-plate world. Compare §4.7e row 4 (`errVerifyLegHasNoPlate`,
`:394`): *"(a) a plate was not presented; (b) it is not one this run cut"* →
`|W| = 2` → `statusUnaccounted`. The count precheck is the **upstream** instance
of the identical observation, and the code's own comment says so
(`gui/multisig_verify.go:726-731`: *"an operator who has mislaid a plate, or
brought one from an earlier run, learns it before typing a seed"*). `:701` is the
same class one level up: zero md1 or zero mk1 cards came back, which is
either the operator not presenting them or a plate too badly cut to classify.

**This is not merely a missing row — the plan routes them wrongly on purpose.**
§4.7d row 4 defines `statusDidNotComplete` as *"an attempt ran and ended with no
adverse observation about the plates — seed typo, refusal, abandon,
**incomplete**"*. Naming `incomplete` sweeps `:738` in by verdict. That is the
exact proxy failure §4.7d `:840-848` says was superseded: `verifyIncomplete` has
**three** return sites (`:738`, `:938`, `:979`) and they are not uniform. The
plan moved `verifyFailed` from verdict → site → return-path/error/provenance and
never repeated the analysis for `verifyIncomplete`. The defect regenerated one
verdict over.

**The harm.** Both paths reach the restore document. `:738` returns
`verifyIncomplete`, which re-arms both retry loops (`gui/multisig.go:337`,
`gui/multisig_build.go:453`); the operator presses CONTINUE and falls through.
`:701`/`:116` return `verifyRefused`/void and fall through immediately. Under
§4.7a's switch, no sticky fact is set (no row ⇒ no `obs`), so the document prints

> `Plate verification DID NOT COMPLETE. Confirm they restore before relying on this backup.`

with **no scoping line** (§4.7f fires only on `statusDisagreed` /
`statusUnaccounted`), over an inventory reading `This backup is 3 plates … If any
of them is missing, this backup is incomplete.` The device has just told the
operator it could not account for a plate this run cut, and the document does not
carry it. **P2 is violated** ("an adverse observation is never lost … never
`DID NOT COMPLETE`"), and §4.7d row 4's universal is false — which is Q4's
answer: no, `statusDidNotComplete`'s "no adverse observation about the plates" is
not true for every observation routed to it.

**Suggested remedy (UNVERIFIED):** add rows for `:701` and `:738` with
`|W| = 2` → `statusUnaccounted`; strike `incomplete` from §4.7d row 4's
enumeration and replace it with the one incomplete that *is* benign — `:979`, the
partial pass where every compared leg matched and the rest were never compared
(also currently missing from §4.7e, though its classification is correct). Add
the corresponding single-sig exit. T15b should gain `:738`.

---

### C-3 — the provenance split P4 rests on has no mechanism: `bundle.Verify` returns untyped errors

**Where:** §4.7e rows 3 and 7 (`:1012`, `:1016`), §4.7e `:1028-1032`, P4
(`:883-889`), T15/T15b (`:1266`, `:1269`). Code: `bundle/verify.go:32-104`,
`gui/multisig_verify.go:424-441`, `:961-963`, `:982-984`,
`gui/singlesig_verify.go:144-146`.

**The defect.** Rows 3 and 7 are outcomes of the **same** return path — `:963`
and `:984` both surface whatever `verifyMultisig` → `bundle.Verify` returned —
distinguished *only* by which leg diverged:

- an mk1 fingerprint / origin-path / stub-binding / md1 divergence is
  **plate-derived** → `statusDisagreed` (row 3, `|W| = 1`);
- an ms1 entropy or wordlist divergence is **operator-typed** →
  `statusUnaccounted` (row 7, `|W| = 2`).

`bundle.Verify` erases that distinction. Every divergence is a bare
`errors.New` / `fmt.Errorf` with **no sentinel and no typed error**
(`bundle/verify.go:53`, `:56`, `:58`, `:65`, `:78`, `:95`, `:102`, and
`checkStubBinding` at `:107-125`). `errors.Is` and `errors.As` cannot separate
them. `multisigVerifyFailureText` (`gui/multisig_verify.go:424-441`) demonstrates
the ceiling: it uses `errors.As` for the two structural errors it owns and falls
back to `err.Error()` text for everything else, with a comment that leans on
*"bundle.Verify's ms1 arms already say 'ms1'"* (`:419-420`) — i.e. a **substring
match on an untested error string** is the only discriminator that exists today.

The plan asserts the split is decidable ("per comparand provenance … what a
per-site or per-verdict split can never express") and never says how the
implementer obtains provenance from `err`. §3 Scope and §4.8's nine build steps
budget no change to `bundle/verify.go`.

**The harm.** Whichever way the implementer resolves it, one of the two prior
Criticals returns:

- collapse to `statusDisagreed` → a one-character hand-typed ms1 typo prints
  `WARNING: … Do NOT rely on this backup: engrave a fresh set` over perfect steel.
  That is R5's I-1 verbatim, on the durable artifact.
- collapse to `statusUnaccounted` → a genuine plate divergence (wrong
  fingerprint, lying origin, broken stub binding) loses its condemning line.

A string match is not a fix either: it makes a funds-bearing classification
depend on an unexported message never asserted by any test, in a package the
Rust-primary rule keeps in sync with an upstream crate.

**Suggested remedy (UNVERIFIED):** scope a step into §4.8 that gives
`bundle.Verify` sentinel or typed errors (e.g. `ErrMS1Entropy`,
`ErrMS1Language`, `ErrMS1Presence` vs the plate-derived set) and require §4.7e
rows 3 and 7 to name the sentinel in their `where` column, so T15's fixture is a
type rather than a substring. Behaviour is unchanged (same failures, same
ordering), so the Rust-primary rule is not engaged — but confirm that before
building.

---

### I-1 — §4.7e row `:724`'s W is impossible: both members are excluded by the byte-equality three lines above it

**Where:** §4.7e row 6 (`:1015`). Code: `gui/multisig_verify.go:717-725`.

**The defect.** The row reads *"readback will not decode | `:724` | (a) a miscut
plate; (b) an NFC read error | 2 | `statusUnaccounted`"*. But `:721` is only
reached when `slices.Equal(readbackMd1, engravedMd1)` **passed** at `:717` —
the readback md1 is byte-identical to the md1 this run engraved. A miscut plate
or a bad read would differ and return at `:719`. So

    real W = { the md1 THIS RUN ENGRAVED does not expand }

— a device-side fault, not a plate fault and not a read fault. Neither declared
member of W is consistent with the observation. `md.ExpandWalletPolicyChunks` is
a pure function of the chunks, so the path is additionally unreachable on the
supply caller, which already expanded that exact slice successfully at
`gui/multisig.go:110` before offering the verify.

**The harm.** The assigned line asserts *"Either a plate was not presented, or it
is not one this run cut. … if this repeats, re-cut the set."* — false in the only
world of the real W, and it sends the operator to re-cut a set whose md1 will
fail identically forever. Under P4 ("every factual claim … true in every member
of W") this row fails its own property. Bounded by reachability, which is why it
is Important rather than Critical.

**Suggested remedy (UNVERIFIED):** correct W to the device-fault world, drop
`|W|` to 1, and route to `statusDidNotComplete` (nothing was observed about the
plates) — or mark the row unreachable and say why, with the `:717` equality as
the reason.

---

### I-2 — §4.7c's "each of the eleven exits maps to exactly one status" is false, at the one exit that matters

**Where:** §4.7c (`:1121-1127`), §4.8 step 1 (`:1175`, "Step 1 is a gate, not a
task"). Code: `gui/singlesig_verify.go:144-146`.

**The defect.** `singleSigVerifyFlow` has exactly one comparison exit:

    if err := verifySingleSig(reDerived, ms1Readback, mk1, md1); err != nil {
        showError(ctx, th, "Verify Failed", "The read-back bundle does NOT match the seed. Check the engraved plates.")
        return
    }

That single exit spans both provenances: an mk1/md1 divergence (plate-derived →
`statusDisagreed`) and an ms1 entropy/language divergence (operator-typed →
`statusUnaccounted`, §4.7e row 7). A **per-exit** mapping cannot express it, so
§4.7c's instruction — *"Each one maps to exactly one status"* — is false, and it
is the instruction §4.8 makes the plan's **first build step** and its one
delegated decision.

**The harm.** An implementer obeying the instruction literally picks one status
for `:145` and is wrong half the time, in the way C-3 describes — on the
single-sig path, which is this cycle's entire subject. The gate the plan relies
on (step 1 is reviewed) cannot catch what the plan told the author to produce.

**Suggested remedy (UNVERIFIED):** restate step 1 as "exit → **observation** →
status", require the mapping to split `:145` by provenance, and name it as the
single-sig instance of C-3's mechanism gap.

---

### I-3 — the scoping line has no seam, and T9 and T16 contradict each other as written

**Where:** §4.7b (`:1042`), §4.7f (`:984-988`), T9 (`:1260`), T11 (`:1262`),
T16 (`:1268`), §4.7 (`:697-700`).

**The defect.** §4.7b specifies one seam:

    func buildVerifyStatusLines(v verifyStatus) []string   // exactly one line

§4.7f then requires a **second** line under two statuses. The plan never says
where it comes from. Both readings break something already written:

- the scope line comes out of `buildVerifyStatusLines` → the `// exactly one
  line` contract is false, and **T9** ("every rendered document carries **exactly
  one** — over `buildVerifyStatusLines`", mutation: "return an empty slice")
  fails on every adverse document unless "exactly one" is silently redefined to
  mean "exactly one *status* line", which the plan does not say and no test can
  measure without a distinguisher;
- a second helper appears → §4.7b's "There is one [seam]" is falsified, nothing
  specifies who calls it, and §4.7c's own warning (`:1107-1113` — the seam is
  `[]string`, so a flow that forgets the call renders silence) now applies to a
  second uncalled function nobody has tested for.

This is the fold's own named failure mode (§4.2 `:422-426`, "folds fail by
incomplete propagation — the facts get corrected and the duplicates are left
standing") reproduced inside the fold, in the section whose whole existence is a
previous under-specified seam (R2 C-1).

**Suggested remedy (UNVERIFIED):** state the contract explicitly —
`buildVerifyStatusLines` returns **one or two** lines, index 0 is always the
status line, index 1 is the scope line iff the status is adverse — and reword T9
to "index 0 is the status line for that status, and no other element of the
returned slice contains a status line", so it still bites the empty-slice
mutation.

---

## THE OBSERVATION TABLE — COMPLETENESS AND |W| AUDIT

Every return statement in `multisigVerifyFlow` (`gui/multisig_verify.go:662-988`)
plus `singleSigVerifyFlow` (`gui/singlesig_verify.go:65-149`), resolved by `sed`:

| line | verdict | row in §4.7e? | |W| audit |
| --- | --- | --- | --- |
| `:670` empty obligation | `verifyRefused` | ~row 9 ("abandoned / refused") | ok; "where: loop exit" is wrong (entry guard, and it is the flow refusing, not the operator) |
| `:681` no engraved md1 | `verifyRefused` | ~row 9 | as above |
| `:696` Back at gather | `verifyAbandoned` | row 9 | **correct**, `|W| = 1` |
| `:701` readback filter | `verifyRefused` | **ABSENT** | **C-2** — `|W| ≥ 2`, holds a bad-plate world |
| `:719` md1 ≠ engraved | `verifyFailed` | row 5 | **correct**, `|W| = 2` |
| `:724` md1 won't expand | `verifyFailed` | row 6 | **I-1** — both members of W excluded by `:717` |
| `:738` plate count ≠ | `verifyIncomplete` | **ABSENT** | **C-2** — `|W| ≥ 2`, same W as row 4 |
| `:794` no expected slots | `verifyRefused` | ~row 9 | unreachable (guarded at `:668`); ok |
| `:797-861` no fresh slot | break → `:938`/partial | ~row 9 | **correct** — the policy is this run's own, so nothing is observed about plates |
| `:887` ms1 entry declined | break | ~row 9 | correct |
| `:897` derive failed | `verifyFailed` | row 8 | `|W| = 1` correct, **narrative wrong** — see M-1 |
| `:938` correctable, 0 legs | `verifyIncomplete` | ~row 9 | correct |
| `:940` abandon, 0 legs | `verifyAbandoned` | row 9 | correct |
| `:963` partial comparator | `verifyFailed` | rows 3/4/7 | rows exist; **C-3** — 3 vs 7 not separable. Row 3's `where` names only `verifyMultisigLegs`, not `…Partial` (M-4) |
| `:979` partial pass | `verifyIncomplete` | **ABSENT** | classification (`statusDidNotComplete`) is right; row still owed |
| `:984` full comparator | `verifyFailed` | rows 3/4/7 | as `:963`; row 2 **miscites this line as "success"** (M-2) |
| `:987` clean pass | `verifyComplete` | row 2 | **C-1** — `|W| = 2`, not 1 |
| singlesig `:69/:78/:90/:98/:112/:125/:130/:139` | void | none | single-sig has **no rows at all**; deferred to §4.8 step 1, see I-2 |
| singlesig `:116` | void | **ABSENT** | **C-2**, single-sig instance |
| singlesig `:145` | void | none | **I-2** — one exit, two provenances |
| singlesig `:148` clean | void | row 2 analogue | **C-1** |

**`|W|` verdicts on the rows as written:** row 2 under-claims (`1`, should be `2`
— C-1). Row 6's W is not merely mis-sized, it is falsified (I-1). Rows 4, 5 and 7
are **right**, and row 7 is the sharpest thing in the fold. Rows 3, 8 and 9 have
the right `|W|`; rows 3 and 8 describe the wrong worlds (C-3, M-1).

**One apparent gap that is not one:** `errVerifyPlateUnclaimed`
(`gui/multisig_verify.go:355-359`) is correctly absent. The `:732` precheck forces
`len(readbackMk1s) == len(expectedSlots)`, the derive loop caps `legs` at
`len(expectedSlots)` (`:773`, `:902`), and `:982` runs only when
`len(legs) == len(expectedSlots)`. So on the full path there are exactly as many
legs as plates and each claims a distinct one — the reverse sweep is vacuous and
the error is unreachable from this flow. Noted so a future reviewer does not read
its absence as a hole.

---

## THE SCOPE-LINE CONDITION — YOUR RULING

**The condition is CORRECT as written — conditional on C-2 being fixed.** Do not
extend it to `statusNotVerified` or `statusDidNotComplete`.

Three reasons, in order of weight:

**1. The scope line resolves a CONTRADICTION, and the contradiction exists only
under 5 and 6.** Under `statusDisagreed` the page says "Do NOT rely on this
backup" and then, nine lines later, "make sure whoever needs **this backup** can
also get the passphrase" — the reader lens's "single worst moment". Under
`statusUnaccounted` the page says the device could not account for plates and
then describes a complete set. Under `statusNotVerified` and
`statusDidNotComplete` the status line does not **forbid** reliance, it
**conditions** it — *"Confirm they restore before relying on this backup"* — and
the downstream inventory is consistent with that reading. There is nothing to
walk back. The discriminant is sharper than "adverse": it is *does the status
line withhold reliance, or condition it*.

**2. Extending it to `statusDidNotComplete` but not `statusNotVerified` inverts
the incentive the whole section is built on.** Status 3 is Skip; status 4 is
"ran and stopped". If the document is more alarming after an attempt than after
no attempt, an operator who wants a clean-looking page learns to skip the verify
— §4.7 `:679-684`, "Never make running the check the way to lose something." So
4 cannot take the line without 3.

**3. Extending it to both puts it on the modal path, where it is noise.** §4.7c
(`:1085-1086`) measures Skip as *"the single most common outcome"*. A scope line
on statuses 3, 4, 5 and 6 renders on the large majority of documents ever
printed, and stops distinguishing the two where the device holds actual adverse
evidence. That is the cry-wolf trap, and it would *reduce* the signal §4.7f was
written to add.

**The dependency, and it is load-bearing.** This ruling holds *only if every
adverse-and-ambiguous observation actually lands in `statusUnaccounted`*. Today
it does not: C-2 routes the plate-count mismatch and the readback-filter refusal
into `statusDidNotComplete`, and for those the scope line's absence **is** wrong.
Fix C-2 and the condition is right; leave C-2 open and the condition is a second
bug rather than a design. The condition should be expressed against the *class*
("adverse observation present"), not against the two status names, so that adding
a row cannot silently leave the page unscoped again.

---

## Q3 — DOES THE SCOPE LINE CLOSE THE READER FINDING?

**Substantially yes, on the half it was written for; not on the half the reader
lens said would actually bite.**

It does close the structural complaint. R6 I-1's core was *"nothing downstream is
conditioned on the status … placement alone does not stop the rest of the
document from vouching for itself."* One line that re-frames the entire remainder
as **intent** is a genuine conditioning of everything below it, it is true under
every status (so it cannot drift out of agreement with the blocks it re-frames),
and it avoids the six-statuses × N-blocks combinatorial rewrite that would have
been the worse fix. §4.7f's P4-cleanliness argument is sound.

It does **not** close the failure mode R6 named as plausible: a reader who is
handed a transcription beginning at "This backup is 4 plates", or who retains the
gist of page 1 but not its force. The scope line sits at slice index 1 — on the
same page as the status it restates — so it travels with the warning, not with
the inventory. And the specific sentence the reader called the worst moment
("make sure whoever needs **this backup** can also get the passphrase") survives
verbatim beneath it.

I do not gate on the residue, for a reason worth recording: that sentence is
**true and correct advice under every status**, including DISAGREED — a passphrase
must be preserved whether the plates are good, re-cut, or still unproved. It
reads badly; it does not misinform. R6 filed I-1 as Important on comprehension
grounds and the fold has answered the part a design can answer. The remainder is
a typographic/paging question for the hardware read-through (§8 blind spot 1),
not a plan defect. **Minor, not gating.**

---

## MINORS AND NITS — recorded, not gating

- **M-1 — §4.7e row 8's W is unreachable.** `:897` is credited to *"the operator
  mistyped the seed"*. A mistyped seed cannot reach `:897`: `seedEntryFlowTypedOnly`
  runs `inputWordsFlow` with `checksumGate: true`
  (`gui/derive_xpub.go:157`), and the derive loop runs only when `fresh` is
  non-empty, which requires `allUserSlots` (`gui/multisig_match.go:78-97`) to have
  already derived that seed successfully **at that slot's own origin** and matched
  the policy xpub. A wrong seed or passphrase exits at `:819`/`:796`. The real W is
  `md.FormAwareStubChunks` / `mk.Encode` / `codex32.EncodeMS1` failing —
  device-side. `|W| = 1` and `statusDidNotComplete` survive; the narrative does
  not. Same error in §4.7d's retained five-site table (`:829`), which also makes
  *"Two of the three non-comparisons are ordinary operator mistakes at verify
  time"* (`:833-834`) a count of one, not two.
- **M-2 — §4.7e row 2 cites `:984` as "success".** `:984` is
  `return verifyFailed`; the success return is `:987`. §4.7d's own five-site table
  (`:831`) reads `:984` correctly as the `verifyMultisigLegs` **mismatch**, so the
  plan contradicts itself in one document. The citation gate resolved it happily,
  which is its declared blind spot working as advertised.
- **M-3 — T15, T15b and T16 appear in no §4.8 build step.** Step 2 lists T14;
  step 7 lists T9, T10, T11, T12, T13a, T13b; step 8 lists T8. The three tests
  that enforce P4 and §4.7f — the newest and least-reviewed properties in the
  plan — have no owning step. T15's cell also points at *"the §4.7d observation
  table"*; the observation table is §4.7e.
- **M-4 — §4.7e row 3's `where` names only `verifyMultisigLegs`.** The partial
  path (`:961-963`) surfaces the same plate-derived divergences through
  `verifyMultisigLegsPartial`. Add it, or the row reads as full-path-only.
- **M-5 — §4.7e `:1024-1026` over-claims.** *"[statusUnaccounted] is the only
  line that can be true of a two-world observation"* is false: a line **scoped**
  to what was observed is also true in every member of W, which is exactly C-1's
  remedy. Ambiguity is one way to satisfy P4; scope is the other, and the stronger
  one where the device knows *which* thing it did not look at.
- **Nit —** §4.7d row 1's *"a comparison over plate-derived bytes completed for
  every plate"* is the same over-claim as C-1 in the knowledge-state table; it
  must move with the line.

---

## WHAT I CHECKED AND FOUND SOUND

- **§4.7a's switch is correct and produces every row of the §4.7d sequence
  table** for the observations that *do* have rows. I executed all twelve rows by
  hand against the code: `status` is assigned only inside the loop body so Skip
  keeps the safe zero value (R3 C-2 is structurally closed); `sawDisagreement` is
  sticky in the one direction that matters (R1 C-1 closed); there is no ordering,
  no `max`, no accumulator seed, so R3 C-1 cannot recur. Adding `sawUnaccounted`
  as a second boolean does not resurrect the lattice — the arm order
  (`complete && sawDisagreement` → `complete` → `sawDisagreement` → default) is
  total and unambiguous with two booleans.
  **One gap in `obs`'s definition, already filed as C-2:** the plan says `obs` is
  "the classified observation … see the 4.7e observation table", so `obs` is
  well-defined *exactly to the extent the table is complete*. A caller cannot
  bypass it — the classification happens inside the verify flow at the return
  site, where the error and provenance are still in hand, which is the only place
  it can happen — but an observation with no row silently classifies as neither
  sticky fact, which is a **fail-open** default. Recommend the classifier be
  exhaustive over return paths (a `default:` that panics in tests, or a status
  constant reserved for "unclassified" that renders the unaccounted line), so a
  future return path added to the flow cannot inherit `DID NOT COMPLETE` by
  omission — the same argument §4.7c makes for the zero value, applied to the
  classifier.
- **§4.7c's zero-value argument is sound and correctly scoped.** The plan is
  explicit that the zero value protects the *variable*, not the *document*, and
  moves the document guarantee onto T9 against a rendered document — the only
  formulation that catches the `nil`-passing flow at
  `gui/multisig_nested_name_test.go:230`. That distinction was drawn correctly.
- **§4.7b's seam and index-0 requirement are right.** `restoreDocScreen` opens at
  `start := 0` with `doneBtn` live on the first frame, so index 0 is the only
  thing that means "page 1"; `append(lines, extra...)` provably cannot place
  anything there. The two-parameter signature and the three-call-site blast
  radius are correct. (The scope line's placement inside that slice is the
  unspecified part — I-3.)
- **§4.7d's five-site measurement table is accurate** on all five rows as a
  description of what each site *is* (I re-derived it from `:719`, `:724`, `:897`,
  `:963`, `:984`), and the decision not to condemn on `verifyFailed` is right. Its
  `:897` gloss is wrong (M-1) but the conclusion does not rest on it.
- **The retry-loop condition change is genuinely control-flow-neutral.** Both
  loops read `if res != verifyIncomplete && res != verifyFailed { break }`
  (`gui/multisig.go:337`, `gui/multisig_build.go:453`, read from source), and all
  five `verifyFailed` sites loop today.
- **P4 is the right property**, and its diagnosis of why the previous two
  Criticals regenerated (world-fact antecedents implemented through proxies) is
  correct. C-1, C-2 and C-3 are not arguments against P4 — all three are P4 being
  applied *further* than the fold applied it: to the agreement side (C-1), to
  `verifyIncomplete` (C-2), and to the mechanism the split needs (C-3).
- **Statuses 2, 3, 5 and 6's lines** are true of their intended observations and
  share no leading token; R6 I-2 and I-3 are genuinely closed by the rewording.
  Status 5's line does state both readings and both actions, as claimed.
- **Not re-opened, per the brief:** the always-render decision, the one-cycle
  scope, the six-status frame, F-199's re-loop question. I found no reason to
  disturb any of them — the frame is right; three of its rows are wrong.

---

**Bottom line.** The frame change was the correct move and P4 is a property worth
having. What the fold did not do is finish applying it: the table stops at the
failure side of the comparator, at `verifyFailed`, and at multisig. The clean
pass over-claims a plate that is never read (C-1), two adverse observations are
routed to the status that means "nothing adverse" (C-2), and the provenance split
the table's centrepiece depends on has no mechanism in the code it cites (C-3).
Not safe to implement as written.
