# S6a WHOLE-DIFF ADVERSARIAL EXECUTION REVIEW
**Branch:** s6a-singlesig-truth @ b2301d6, vs main b8a23bf
**Scope:** 21 files, +3157 -73, build-order steps 2-8

## VERDICT: RED — 0 Critical, 1 Important (+ 3 filed)

The F-198 fix itself is sound. The eleven-exit mapping is implemented **exactly**
as step 1 specified — measured, not read. `suppliedCosigners` cannot under-report.
The pass write is unreachable without a passing comparison. The scoping line is
derived inside both document flows and cannot be forgotten on the third. Every
new operator string is byte-identical to the plan text that authorises it, and
every one is pure ASCII. The three walks that gained a press still reach the
screens they are named for, and no source assertion was loosened — all four were
*strengthened*.

The one blocking finding is a string this diff newly places on the two **multisig**
restore documents, which names a value those documents do not contain.

---

### I-1 — the zero-cell status line names a "master fingerprint below" that the multisig restore documents do not have `[MECHANICAL]`

**Severity: Important.**

`verifyStatusNotFullyCheckedLine` (`gui/verify_status.go:133-135`) reads:

> `These plates were not fully checked. Confirm they restore this wallet (master fingerprint below) before relying on this backup.`

Before this diff that line existed nowhere. This diff adds a `status` parameter to
**`multisigRestoreDocFlow`** (`gui/multisig_restore.go:107`) and places the status
at slice index 0 of both multisig restore documents. On a run that skipped or never
entered the verify — which §4.7f itself calls the **modal** occupant of this cell —
that is the line those documents now carry.

**The multisig restore documents contain no master fingerprint.** Measured over the
whole package:

    $ grep -rn "Master fp" --include='*.go' gui/ | grep -v _test
    gui/singlesig_restore.go:107:  fmt.Sprintf("Master fp: %08x", masterFP),

    $ grep -rn 'fingerprint' --include='*.go' gui/ | grep -v _test | grep '"'
    ...
    gui/multisig_build_census.go:324:  return fmt.Sprintf(" (master fingerprint %08x)", fp)
    gui/verify_status.go:134:          "Confirm they restore this wallet (master fingerprint below) before " +
    ...

Exactly two producers can put a labelled fingerprint on a restore document:

1. `gui/singlesig_restore.go:107` — the **single-sig** document only. There the
   line is correct: `Master fp:` is the first document line under the status.
2. `gui/multisig_build_census.go:324` `seedFingerprintSuffix`, reached only from
   `buildPassphraseInventoryLines`' per-seed arm, which is gated on
   `len(seeds) >= 2` (`gui/multisig_build_census.go:283`) **and** at least one
   passphrased seed.

`multisigRestoreLines` (`gui/multisig_restore.go:22-69`) emits, on either branch,
`Type:` / `Wallet policy (read-only):`, the chunked descriptor or `desc4Display`,
and the addresses. No fingerprint line on either.

**Reproduction — the multisig SUPPLY document, always.**
`supplyMultisigPolicyFlow` (`gui/multisig.go:375-376`) passes
`oneSeedPassphraseFact(...)`, which is a **one-element** slice
(`gui/multisig_build_census.go:334-336`). `len(seeds) < 2`, so
`buildPassphraseInventoryLines` returns at `:284` before the per-seed loop and
**no fingerprint is ever rendered on that document**. Engrave a supplied policy,
answer *Skip* at the verify offer, and page the restore document: the first line
tells the reader to confirm against a master fingerprint below, and there is none
on any page. On the display-only branch (`expandedToDescriptor != expandOK` — a
policy shape bip380 cannot render) there is not even an unlabelled one, because
that branch prints `desc4Display(tpl)` = `scriptName + policyLine` (e.g.
`P2WSH 2-of-3 multisig`) instead of a descriptor.

**Reproduction — the multisig BUILD document, common case.**
`buildMultisigPolicyFlow` (`gui/multisig_build.go:493`) passes
`reg.passphraseFacts()`. A build with one registered seed, or one where no seed
carries a passphrase, takes the same early return at `:284` / the
`len(passphrased) == 0` arm at `:267`, and prints no fingerprint either.

**Why this is Important and not Critical.** On the `expandOK` branch the master
fingerprints *are* physically below, unlabelled, inside the descriptor's
`[fp/origin]` prefixes — a determined reader can extract them. Nothing here
vouches for plates, claims a check that did not run, or strengthens a
verification claim, so neither G1 nor G2 is breached in its own terms. What is
breached is the only actionable instruction on the weakest and most common line,
on the durable artifact, on two of the three flows.

**Why NG1 does not shield it.** NG1 protects against *expanding what the document
reports about the device's knowledge state*. This finding is the opposite: the
document points at a value it does not print. Correcting it neither adds an
epistemic dimension nor touches the 2x2.

*Remedy — UNVERIFIED.* Either make the parenthetical conditional on the document
that carries it (a second constant, or a `firstDocLine`-style hint threaded from
the two doc flows), or drop the parenthetical so the line reads
`...Confirm they restore this wallet before relying on this backup.` The second is
one edit and loses nothing the single-sig document does not already show on the
line immediately below.

---

### M-1 — clause B stays singular on a multi-leg full multisig verify, where the device's own screen says "for each seed" `[JUDGEMENT]`

**Severity: Minor (filed).**

`verifyStatusMS1Clause` (`gui/verify_status.go:149`) is
`"The ms1 secret you typed matched this seed."`, rendered iff `p.full`
(`gui/verify_status.go:216-220`). It is not keyed on `p.legs`.

A full-mode multisig verify across two masters (Trace B, build path) types **two**
ms1s — one per seed, `gui/multisig_verify.go:926-950`, "ONE PER SEED" — and on the
success return records `{full: true, legs: 2}`. The pass line then reads:

> `2 key plates were read back and matched what this run engraved. The ms1 secret you typed matched this seed. Other cosigners' keys are taken as supplied.`

Clause A agrees in number; clause B does not. The precedent is in this same diff's
neighbourhood and is already reviewed: `multisigVerifyOKMessage`
(`gui/multisig_verify.go:1133-1136`) says, for exactly this case,
`"All %d operator key plates verified, and the ms1 you typed for each seed."` So
the on-screen notice and the durable document describe the same run with different
grammar, and the document is the one that under-counts.

**Not gating**: the singular *under*-claims (one comparison named, two ran), which
is the direction G2 permits, and §4.7c fixes clause B's text verbatim, so the
implementation transcribed its authority faithfully. Filed because §4.3 fixed the
identical singular/plural wobble in the seed-handling ruling one section earlier,
and this clause was not swept.

---

### M-2 — §4.7f's "NOT widened to statusNotFullyChecked" is guarded by nothing `[MECHANICAL]`

**Severity: Minor (filed).**

`verifyStatusScopeLines` (`gui/verify_status.go:189-194`) renders the scoping line
iff `status == verifyStatusDidNotPassLine`. The code is **correct**: the identity
is against a package constant, `buildVerifyStatusLine` emits that constant for
exactly one cell, and neither pass cell nor the zero cell can produce it.

What is missing is the test. `verifyStatusScopeLine` is read in exactly one place
in the whole test tree:

    $ grep -rn 'verifyStatusScopeLine' --include='*_test.go' gui/
    gui/singlesig_truth_test.go:1715:  gi := s6aUIIndex(doc, verifyStatusScopeLine)

and `s6aAssertStatusFirstAndScope` has three call sites, whose statuses are the
**pass** line (`:1274`, wantScope=false), **didNotPass** (`:1319`, wantScope=true)
and the **retry** line (`:1352`, wantScope=false). The fourth cell —
`statusNotFullyChecked`, the one §4.7f and R7 argue about by name — is never driven
through this assertion. The two other tests that render a document under the zero
cell (`gui/multisig_nested_name_test.go:233`,
`gui/singlesig_truth_test.go:508` with a synthetic needle) assert nothing about the
scoping line.

*Remedy — UNVERIFIED.* Add one direct row on the pure function:
`verifyStatusScopeLines(verifyStatusNotFullyCheckedLine)` must be nil and
`verifyStatusScopeLines(verifyStatusDidNotPassLine)` must be the one line. That
costs no walk time and closes the cell nothing currently covers.

*(I did not run the widening mutation, because the brief forbids modifying either
tree. The coverage fact above is measured; the survivability inference is not.)*

---

### N-1 — `singleSigReadbackCards` silently ignores a card of any third kind `[MECHANICAL]`

**Severity: Nit (filed, pre-existing).**

`gui/singlesig_verify.go:23-42` switches on `cardMK1`/`cardMD1` only. A readback
pile containing one mk1, one md1 and a stray `cardMS1` (or any future kind)
returns `ok`, the extra card is dropped, and the run can reach the success exit.
The multisig analogue refuses that shape (`errVerifyPlateUnclaimed`, adverse).

Unchanged by this diff and **produces no false claim**: clause A says
`1 key plate was read back and matched what this run engraved`, which stays true;
nothing on the document asserts the pile was fully accounted for. Recorded only
because this diff is what first turns that function's `ok` into a printed pass.

---

## PART 1 — THE ELEVEN EXITS vs STEP 1's MAPPING

**Measured in the shipped file, not quoted.**

    $ awk 'NR>=89 && NR<=203 && /return/{print NR}' gui/singlesig_verify.go
    97 106 118 126 140 (144*) 152 160 165 173 183 (186*)
      (*) 144 and 186 are comment lines containing the word "return"

Ten explicit `return` statements at **97, 106, 118, 126, 140, 152, 160, 165, 173,
183**, plus the fall-through at the closing brace, **203**. **Eleven exits**, the
same eleven step 1 mapped at their pre-diff line numbers (69, 78, 90, 98, 112,
117, 125, 130, 138, 146, 149).

    $ grep -n 'rec\.' gui/singlesig_verify.go
    150:  rec.adverse = true
    181:  rec.adverse = true
    198:  rec.pass = &passRecord{

| step 1 row | old | new exit | required | shipped |
| --- | --- | --- | --- | --- |
| 1 seed-entry Back | :69 | :97 | neither | **neither** |
| 2 pick-flow Back | :78 | :106 | neither | **neither** |
| 3 re-derive fails | :90 | :118 | neither | **neither** |
| 4 templateize fails | :98 | :126 | neither | **neither** |
| 5 gather Back / Done | :112 | :140 | neither | **neither** |
| 6 readback not accounted for | :117 | :152 | **adverse** | **adverse** (:150) |
| 7 ms1 keyboard Back | :125 | :160 | neither | **neither** |
| 8 not a codex32.String | :130 | :165 | neither | **neither** |
| 9 DecodeMS1 rejects | :138 | :173 | neither | **neither** |
| 10 comparator disagreed | :146 | :183 | **adverse** | **adverse** (:181) |
| 11 fall-through | :149 | :203 | **pass** | **pass** (:198) |

**2 adverse / 8 neither / 1 pass. Exact match, no drift.**

**The dangerous direction — a benign exit writing `adverseRecorded` — does not
occur.** Only two `rec.adverse` writes exist in the file, and both sit under a
guard that has already read something off the plates: `:150` follows
`singleSigReadbackCards(cards)` returning false, reachable only after
`bundleGatherFlow` returned `ok` (complete cards, operator pressed Done); `:181`
follows `verifySingleSig` returning a comparator error. Neither is reachable
without a plate having been read.

**The reverse direction — an adverse observation writing nothing — does not occur
either.** The eight silent exits are a Back (×3), a refusal at the keyboard (×2),
and three failures over objects that never touched steel: `deriveSingleSigBundle`
on the **re-typed** seed (:118 — byte-identical to the multisig site §4.7b
classifies benign), `templateizeBundle` on the **re-derived** bundle (:126), and
a type assertion / decode over a **hand-typed** string (:165, :173).

**`statusVerifiedOnRetry` remains unreachable from single-sig.** Both adverse
sites are terminal `return`s, the flow has no loop, and the single production
call site is a one-shot `if` (`gui/singlesig.go:190-192`) with `rec` declared
outside it at `:188`.

`TestSingleSigVerifyRecordsWhatItObserved` drives three of the eleven — one per
class, including the benign arm that keeps the other two honest — through the real
flow. All three pass.

---

## PART 2 — THE PASS WRITE

`rec.pass` is written at **one** place, `gui/singlesig_verify.go:198`, in the
fall-through after `showNotice("Verify OK")`. The only control path that reaches
it falls through
`if err := verifySingleSig(reDerived, ms1Readback, mk1, md1); err != nil { ...; return }`
at `:179-184`. **There is no path to the success exit that did not pass the
comparison.**

Vacuity of the comparison itself is closed upstream: `singleSigReadbackCards`
guarantees `len(mk1) > 0 && len(md1) > 0` (`:38-40`) before the comparator sees
them, and `reDerived` is a real derive from a **re-typed** seed
(`seedEntryFlowTypedOnly`), never the session payload — the §7.4 self-comparison
hazard the flow's own comment at `:130-137` names.

**`full` is RECORDED, not inferred** — `full: full` at `:199`, the flow's own
parameter, captured where it is in scope. This is R9's C-1 and it holds: a
watch-only run (`full=false`) sets `ms1Readback = ""`, `verifySingleSig` drops the
derived MS1 (`:51-55`) so `bundle.Verify` skips the ms1 leg, the record carries
`full: false`, and `buildVerifyPassLine` emits clause **B2**
(`No secret seed share was read back or compared.`) rather than merely omitting
B. Driven end to end by the real single-sig verify walk at
`gui/singlesig_truth_test.go:1265-1287`.

`legs: 1` is correct on a template engrave: `templateizeBundle` re-stubs the mk1
rather than removing it, so one real key plate is compared in both forms.
`suppliedCosigners: 0` is by construction — single-sig has no policy key it did
not itself derive and compare.

**Ordering.** `buildVerifyStatusLine(rec)` is an argument at
`gui/singlesig.go:222`, evaluated after the verify at `:191` returns. The record
is written before it is read on all three flows.

**Multisig pass write** (`gui/multisig_verify.go:1057-1061`) is likewise the sole
site and sits after `verifyMultisigLegs` returns nil, i.e. after the full
bijection including the reverse "every plate claimed" sweep. `legs: len(legs)` is
what was compared; on that path `len(legs) == len(expectedSlots) ==
len(readbackMk1s)` (guarded at `:788`), so the count is plates, not slots.

---

## PART 3 — `suppliedCosigners`

Shipped at `gui/multisig_verify.go:680-688`:

    func countUncoveredPolicyKeys(keys []md.ExpandedKey, covered map[int]bool) int {
        n := 0
        for i := range keys {
            if !covered[i] { n++ }
        }
        return n
    }

**It iterates the keys and asks whether each is covered** — the direction step 1
specified and the one that cannot under-report. An entry in `covered` outside
`[0, len(keys))` is *ignored*; a wrongly-cleared entry *inflates* the count. Every
defect in the map can only make the number larger, and a larger number renders a
clause saying **less** was checked. The rejected `len(keys) - len(covered)` form
does not appear anywhere in the tree.

**The `covered` indexing is sound.** In the shipped file:

    $ grep -n 'covered\[' gui/multisig_verify.go
    330:  if covered[s] || !slices.Contains(expected, s) {
    900 → 960:  covered[s] = true
    1034: if !covered[s] {

`:960` is the only write in the file, and it is `covered[s] = true` executed
**immediately after** `legs = append(legs, verifyLeg{Slot: s, ...})` at `:959`, in
the same loop body — so every covered entry has a leg behind it, which is what
makes under-reporting structurally impossible. The domain matches: `s` comes from
`fresh` ⊆ `slots` = `allUserSlots(..., keys)`, which builds its result as
`append(matches, i)` over `for i := range keys`.

`keys` (`:775`) and `covered` (`:828`) are both function-body bindings of
`multisigVerifyFlow` and both live at the success return (`:1057`). No signature
changed to reach them.

**T27 asserts non-vacuity rather than assuming it**
(`gui/singlesig_truth_test.go:1952-2014`): the fixture is one slot of a
three-key policy, so the flow computes 2 and the test fails if the count is ≤ 0.
It also pins `suppliedCosigners == policyKeys - legs` and pins the two lines to
differ by **exactly** the cosigner clause. Both halves run through the **real**
flows, not a stub. Verified green (`0.06s`).

Single-sig writes the literal `0`, and `grep -n -i "cosign\|policy\|covered"
gui/singlesig_verify.go` returns nothing — there is no policy key on that path for
the count to miss.

---

## PART 4 — PRINTED STRINGS vs §4.7c

Every rendered line reconstructs from §4.7c's clause table, byte for byte.

| §4.7c | shipped | site |
| --- | --- | --- |
| zero cell | identical | `verify_status.go:133` |
| didNotPass | identical | `verify_status.go:141` |
| **A** `<N> key plate(s) was/were read back and matched what this run engraved.` | identical | `verify_status.go:212-214` |
| **B** `The ms1 secret you typed matched this seed.` | identical | `:149` |
| **B2** `No secret seed share was read back or compared.` | identical | `:154` |
| **C** `Other cosigners' keys are taken as supplied.` | identical | `:156` |
| **D** `An earlier check did not pass; a later full check passed.` | identical | `:147` |
| §4.7f scope line | identical | `:169-170` |

**Number agreement, clause A.** `plateWord(n, "key plate", "key plates")`
(`gui/multisig_build_census.go:19-24`) returns `1 key plate` / `N key plates`, and
the verb is chosen alongside it (`verb := "were"; if p.legs == 1 { verb = "was" }`,
`verify_status.go:208-211`). `1 key plates` is not constructible. `legs == 0` is
unreachable — multisig refuses `len(expectedSlots) == 0` at `:715` before anything
else, single-sig writes the literal 1. Clause B does **not** agree in number; that
is M-1.

**No clause without a recorded observation.** A→`legs`, B/B2→`full`,
C→`suppliedCosigners`, D→`adverse`. There is no descriptor-plate clause, matching
§4.7c's deliberate deletion. `TestVerifyPassLineClausesAreEachBackedByARecord`
makes that audit executable — it checks each named field **exists by reflection**
and compares the whole rendered line against exactly the entitled clauses in both
the pass and the retry cell.

**Mode-blind literals.** One found, and it is the blocking finding: the zero-cell
line's `(master fingerprint below)` is blind to *which document* it is printed on
(I-1). The mode axis proper (full/watch-only) is handled correctly everywhere.

**ASCII.** Every new operator string in the diff was extracted and swept; the only
non-ASCII bytes on changed non-comment lines are pre-existing trailing-comment em
dashes. `TestVerifyPassLineClausesAreEachBackedByARecord` additionally rejects
`— – · ' ' " " …` in every clause under test.

**Byte-identity of the S5-reviewed sentence holds.** `seedCapacityMany` +
seed-on-plates reassembles the shipped ruling exactly, as §4.3 claims;
`TestSeedResidencyRulingDescribesTheMultiSeedReality` passes with every assertion
it had, now handed `seedCapacityMany`.

---

## PART 5 — §4.7f's SCOPING LINE (implementation stood in for spec)

`verifyStatusScopeLines(status string) []string`
(`gui/verify_status.go:189-194`) returns the scoping line iff
`status == verifyStatusDidNotPassLine`, and is called from **both** document
flows — `gui/singlesig_restore.go:142` and `gui/multisig_restore.go:113` — as

    head := append([]string{status}, verifyStatusScopeLines(status)...)
    restoreDocScreen(ctx, th, append(append(head, lines...), extra...))

**Does it render under `statusCheckDidNotPass` and only there?** Yes, and the
identity is sound rather than fragile. `buildVerifyStatusLine` returns
`verifyStatusDidNotPassLine` — the same package constant — for exactly that cell
(`:259-260`). The zero cell returns a different constant. The two pass cells return
a *generated* string that always begins with clause A's
`fmt.Sprintf(...)` and can never equal the constant. Rewording the status
necessarily reworks the comparison in the same edit.

**Can it be forgotten on the third flow?** No. There are three production flows
and **two** document functions; both derive the line for themselves, so a flow
cannot reach a document without passing through one of them.

    $ grep -rn 'restoreDocScreen(' --include='*.go' .
    gui/multisig_restore.go:114
    gui/singlesig_restore.go:143   (+ the declaration at :150)

Both are the two-argument `append(append(head, lines...), extra...)` form. No third
site exists, and no call site can omit the line the way an added parameter could.

**Position.** The scope line lands at index 1, between the status and the first
document line, which is what §4.7f asks. `s6aAssertStatusFirstAndScope` checks
`si < gi < di` on a rendered supply document, and the supply walk drives the real
`statusCheckDidNotPass` cell.

**The one gap is on the negative side**, and it is M-2: nothing asserts the line's
absence under the zero cell — the cell R7 argued about by name.

**No aliasing in the head-building.** `append([]string{status}, ...)` allocates
fresh; `lines` and `extra` are copied in, never retained or mutated; neither
caller's slice is aliased by the result.

---

## PART 6 — WHAT THE TESTS CANNOT SEE

**Nil-pointer paths.** Three guards, all present and all reachable-by-design
rather than decorative:

- `buildVerifyStatusLine` returns the **weakest line** rather than dereferencing
  when a pass cell is derived with `rec.pass == nil` (`verify_status.go:250-252`).
  Today unreachable; the failure it forecloses is a SIGSEGV on the device where
  the design calls for the safest string.
- `singleSigVerifyFlow` (`:91-93`) and `multisigVerifyFlow` (`:707-709`) both
  **discard** a nil record rather than dereferencing it. Every production caller
  passes one; `s6aRecordingStub` fails the test loudly if a caller ever passes nil
  (`singlesig_truth_test.go:1425-1429`).

**Status line at index 0.** `buildVerifyStatusLine` returns a `string`, not a
`[]string`, and every arm returns a non-empty constant or a generated line whose
clause A is unconditional — `""` is not constructible. All three production call
sites pass it (`gui/singlesig.go:222`, `gui/multisig.go:375`,
`gui/multisig_build.go:492`), and
`TestRestoreDocStatusIsBuiltFromTheRecordOnEveryFlow` pins each of them **over
comment-stripped source** (`s6aCodeOf`), which closes the comment-satisfaction
trap for this assertion by construction.

**Ordering.** Record declared → written → read, on all three flows. On the two
multisig flows the declaration is **outside** the re-offer loop
(`gui/multisig.go:335`, `gui/multisig_build.go:451`), which is what makes
`adverse` sticky; `gui/multisig_build.go`'s is also outside the
`if !template && len(legs) > 0` gate, so a template build or a legless build still
renders the zero cell rather than skipping the status. The comment says so and the
code does so.

**No exit assigns a `verifyStatus`.**

    $ grep -rn 'statusVerified|statusCheckDidNotPass|statusNotFullyChecked|verifyStatusFor' \
        --include='*.go' gui/ | grep -v verify_status.go | grep -v _test.go
    gui/singlesig_verify.go:86   (comment)
    gui/multisig_verify.go:702   (comment)
    gui/singlesig.go:186,203     (comments)
    gui/multisig.go:332          (comment)

Five hits, **all in comments**. No production code outside `verify_status.go`
names a status value, let alone assigns one. `TestVerifyStatusDerivationReadsNoVerdict`
enforces the converse over comment-stripped source: `verify_status.go` names none
of the five verdict constants nor the callers' `res` variable, and *does* read both
`rec.pass` and `rec.adverse`.

**Build gate.** `go build ./...` exit 0. `gofmt -l` over all 21 changed files:
empty. `go vet ./gui/` reports only the two known pre-existing go1.26
`t.ArtifactDir()` failures.

---

## PART 7 — MULTISIG REGRESSIONS

**Arity change, counted rather than described.**

    $ grep -rn 'multisigVerifyFlow(' --include='*.go' .
    → 1 declaration + 10 call sites (8 pre-existing, 2 new in singlesig_truth_test.go)
    $ grep -rn 'multisigVerifyFn' --include='*.go' .
    → 2 production dispatches, 4 source-assertion needles, 2 test stubs
    $ grep -rn 'singleSigVerifyFlow(' --include='*.go' .
    → 1 declaration + 1 production call site + 2 test call sites

Every site carries the new parameter; the tree builds. Nothing was deleted.

**No source assertion was loosened.** All four were **strengthened** — each needle
gained `, &rec)`:

- `multisig_verify_flow_test.go:389` `multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1, &rec)`
- `multisig_verify_flow_test.go:410` `...suppliedMd1, &rec)`
- `multisig_verify_report_test.go:1079,1081` — the same two, in the re-offer table

None is substring-weakened, and none can be satisfied by a comment: `funcBody`
(`multisig_build_title_test.go:87`) does **not** strip comments, so I checked the
two enclosing functions directly — `gui/multisig.go:326` and
`gui/multisig_build.go:441` mention `multisigVerifyFn` in prose but neither quotes
the parenthesised call, so the needle can only match the code line. (The newer
S6a assertion uses `s6aCodeOf`, which strips comments outright.)

**The three walks that gained a press still reach the screen they are named for.**
Each now asserts `Plates To Cut` *before* pressing, so the added `click` cannot
silently answer a screen that did not appear:

    --- PASS: TestEngraveSingleSigFlowFull        → still reaches "Card 1 of 3"
    --- PASS: TestEngraveSingleSigFlowWatchOnly   → still reaches "Card 1 of 2"
    --- PASS: TestEngraveSingleSigFlowTemplate    → still reaches "Card 1 of 3"

**Multisig adverse classification matches §4.7b row for row.** Fifteen
`return verify*` sites (717, 727, 743, 753, 773, 780, 798, 854, 957, 998, 1000,
1026, 1042, 1051, 1063); six write `adverse` (751→753, 771→773, 778→780, 792→798,
1024→1026, 1049→1051) and one writes `pass` (1057). Mapped to §4.7b's pre-diff
line numbers: `:701`, `:719`, `:724`, `:738`, `:963`, `:984` — **exactly the
adverse column**. §4.7b's seventh adverse entry, `:394 errVerifyLegHasNoPlate`, is
inside `verifyMultisigLegsPartial` (now `:400`) and reaches the flow as an error at
both comparator call sites, which both set the bit. The benign column — `:897`
re-typed seed will not derive (now `:957`), `:938`/`:940` zero legs (now
`:998`/`:1000`), `:696` abandon (now `:743`), `:670`/`:680`/`:794` structural
refusals (now `:717`/`:727`/`:854`), `:979` partial-verify-all-matched (now
`:1042`) — writes nothing at all of the eight.

`TestMultisigVerifyRecordsWhatItObserved` drives one site of each class through the
real flow and passes.

**Other regression-sensitive tests, all green:**

    --- PASS: TestRestoreDocNamesEveryPassphrasedSeed        (signature update only)
    --- PASS: TestRestoreDocMergesOneSeedHeldAtTwoSlots
    --- PASS: TestRestoreDocSaysWhichSeedsNeedNoPassphrase
    --- PASS: TestSeedResidencyRulingDescribesTheMultiSeedReality
    --- PASS: TestRestoreDocSaysThePassphraseIsNotOnThePlates
    --- PASS: TestRestoreDocNestedNameIsActuallyDrawn        (now rasterises a real status line)
    --- PASS: TestBothEngraveFlowsReOfferTheVerify
    --- PASS: TestBuildPassesTheTailsSlotsToTheVerify
    --- PASS: TestSupplyPassesTheEngravedPolicyToTheVerify
    --- PASS: TestSingleSigShowsThePlateCensusBeforeTheEngrave
    --- PASS: TestSingleSigAbortIsTheLastScreenOfTheProgram
    --- PASS: TestVerifyStatus* / TestVerifyPassLine* (all)
    --- PASS: TestSingleSigVerifyRecordsWhatItObserved (3 subtests)
    --- PASS: TestMultisigVerifyRecordsWhatItObserved  (2 subtests)
    --- PASS: TestBundleAbortJustificationNamesEveryTailCarryingCaller
    ok  seedhammer.com/gui  0.207s / 0.403s   (two scoped -run batches)

**Step 8's own factual corrections were re-measured; all four hold.**

    seedEntryFlowTypedOnly                     → gui/derive_xpub.go:140   ✓ (was cited :124)
    deriveXpubFlow's multiPlateEngrave call     → gui/derive_xpub.go:390   ✓
    NEEDLE_SLOT walk drivers                    → 4: walk_build_policy, walk_trace_b,
                                                  walk_s4_gate, walk_s3_nested  ✓ (was "three")
    "Which slot is your key?" production sites  → 1 (gui/multisig_build.go:894)  ✓
    bundleEngrave call sites                    → 4; three gate on bundleEngraveDone
                                                  (singlesig.go:177, multisig.go:291,
                                                  multisig_build.go:402), bundle_flow.go:39
                                                  returns on the next line  ✓ (was "both")
    bundleGatherFlow (nil,false) returns        → exactly 2  ✓ (the "empty bundle" claim is gone)

I found no fifth false comment.

---

**Nothing in either tree was modified.** `git status --porcelain` in
`/scratch/code/shibboleth/wt-s6a` returns 0 lines; HEAD is
`b2301d6183073d9f6a307729b587fee7047aba38`, `main` is
`b8a23bf3dcf45f0b996bedf8b17f7141f092d282`.
