# S6a STEP-1 GATE REVIEW
**Artifact:** design/S6A_STEP1_EXIT_MAPPING.md
**Plan:** design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md (GREEN at R17)
**Code:** /scratch/code/shibboleth/seedhammer, main = b8a23bf3dcf45f0b996bedf8b17f7141f092d282

## VERDICT: GREEN — 0 Critical, 0 Important (+ 3 filed)

All eleven exit classifications are correct under G2. Both artifacts meet §4.8's
stated acceptance criteria. **Step 2 may begin.**

Every `path:line` the document cites was opened and read at the pinned SHA. I
found **no** citation that did not resolve, and **no** classification I would
change. The three filed items are wording/completeness notes on the document's
*rationale* prose; none of them changes a bit in the mapping table, and none
gates.

---

### N-1 [JUDGEMENT] — Nit. The decidable rule's two nouns are narrower than the sentence it is derived from.

§1 states the rule as:

> **ADVERSE iff the device ran a comparison or an accounting over what it read
> back off the plates, AND that check produced a negative result.**

The line it is derived from (§4.7c `statusCheckDidNotPass`) has **three** verbs:
*"a comparison did not match, or a plate **could not be read** or accounted for."*

The multisig site `gui/multisig_verify.go:724` is classified **adverse** by §4.7b
(*"readback will not decode"*), and it is a **decode** failure —

    721	_, keys, err := md.ExpandWalletPolicyChunks(readbackMd1)
    722	if err != nil {
    723		showError(ctx, th, "Verify Bundle", "Couldn't decode the read-back wallet policy.")
    724		return verifyFailed

— which is "could not be read", and is neither a *comparison* nor obviously an
*accounting*. The rule reaches the right verdict only under a broad reading of
"accounting". **This does not disturb any of the eleven single-sig rows**: no
single-sig exit is a decode failure over readback (`:117` is a genuine kind
accounting, `:146` a genuine comparison), so the rule is sound as applied here.

*Remedy — UNVERIFIED:* widen the rule's wording to "read, compared or accounted
for" so it mirrors the sentence it is decidability-shorthand for.

### N-2 [MECHANICAL] — Nit. U2's argument 2 omits `inputCodex32Flow`'s error-correction arm.

U2 argues: *"A mis-engraved character fails the checksum, so the operator never
gets past the keyboard and exits at `:125` (benign)."* `inputCodex32Flow` has a
third arm the argument does not mention:

    1042		if !valid && inWin && clicked3 {
    1043			res, ok := codex32.Correct(frag)
    ...
    1046			} else if confirmCorrectionFlow(ctx, th, res, strings.ToLower(parsed.HRP)) {
    1047				kbd.Fragment = res.Corrected // accept; next frame re-validates → OK

An invalid fragment within 4 changes can be corrected and accepted, so "never
gets past the keyboard" is too strong. **It does not change the verdict, and it
points the same way U2 already argues**: a corrected fragment is a valid `ms`,
so it reaches `:135`/`:144`, not `:130`/`:138` — the correction arm makes the
residual bad-plate world at `:130`/`:138` *rarer*, not commoner.

*Remedy — UNVERIFIED:* add one clause naming `gui/gui.go:1042-1049` and note it
narrows rather than widens the bad-plate world-set.

### N-3 [JUDGEMENT] — Nit. U4 is already settled inside the plan; U4 does not cite where.

See Part 5. The plan's §5 T27 row (plan line 1199) already states the U4
observation verbatim and already imposes the remedy. U4 re-derives it without
citing it, which invites a second decision on a decided question.

*Remedy — UNVERIFIED:* cite plan line 1199 in U4 and mark it "already ruled at
R15; no step-1 decision needed".

---

## PART 1 — THE TWO ADVERSE ROWS (`:117`, `:146`)

**Both are correct. Neither prints a false sentence on any reachable path.**

### `:146` — the comparator ran and disagreed. ADVERSE, uncontested.

    144	if err := verifySingleSig(reDerived, ms1Readback, mk1, md1); err != nil {
    145		showError(ctx, th, "Verify Failed", "The read-back bundle does NOT match the seed. Check the engraved plates.")
    146		return

`verifySingleSig` (`gui/singlesig_verify.go:49-58`) builds
`bundle.Bundle{MS1: ms1Readback, MK1: mk1, MD1: md1}` from the **read-back**
cards and calls `bundle.Verify`. Reaching `:146` requires a non-nil error from
that comparator. §4.7b binds it independently: *"any `bundle.Verify` error"* is
in the adverse column, and §4.7b states all eleven of its errors classify
identically. A check ran; it produced a negative. The sentence is true.

*One case I chased and rejected as a finding.* At `gui/singlesig_verify.go:82-86`
the passphrase prompt is `if sel, ok := ppChoice.Choose(...); ok && sel == 1 {
if pass, ok := passphraseFlow(ctx, th); ok { passphrase = pass } }` — a Back
inside `passphraseFlow` silently leaves `passphrase == ""`, so an operator who
*has* a passphrase can re-derive the wrong wallet and land on `:146` with
innocent plates. The adverse line is still **true** (a comparison ran and did
not match), so G2 holds; distinguishing this on the *document* is NG2 (the
multisig flow does it on a *screen*, via `multisigVerifySeedIsInnocent`,
`gui/multisig_verify.go:120`). Not a finding, not in scope.

### `:117` — the author's U1, the hardest row. ADVERSE is correct.

The brief's test: *if `:117` can be reached on a path where nothing was read
back, the adverse classification prints a false sentence.* **It cannot.** I
verified the two ways that could happen and both are closed:

**(i) `ok == true` from `bundleGatherFlow` implies at least one complete card.**
The flow (`gui/bundle_flow.go:143-241`) has exactly four returns:

    177: 			return nil, false
    207: 					return scr.g.cards, true
    210: 				return scr.g.cards, true
    241: 	return nil, false

`:210` is the `bundleDoneProceed` arm, and `bundleDoneDecision`
(`gui/bundle_flow.go:256-264`) returns `bundleDoneProceed` only after
`len(g.cards) == 0` has been excluded. `:207` is the `bundleDonePending` arm and
is guarded by `if len(scr.g.cards) > 0`. So `ok == true` ⟹ `len(cards) >= 1`.
Row 6's claim is exact.

**(ii) Those cards came off the reader, not out of the session.** This is the
one attack the document does *not* make, and it is the one that would have
falsified the row. `bundleGatherFlow` drains a session payload into the same
accumulator before scanning:

    161	for _, seed := range ctx.syswBundleSeeds {
    162		if seed == "" {
    163			continue
    164		}
    165		scr.g.offer(mdmkText(seed))
    166	}
    167	ctx.syswBundleSeeds = nil

If `ctx.syswBundleSeeds` could be non-empty when `singleSigVerifyFlow` runs, the
"read-back" cards would be the engrave source compared against itself — which
would break row 6's rationale *and* row 11's pass write. It cannot. There are
exactly three production writers, and **each one calls `bundleGatherFlow`
immediately afterwards**, which nils it at `:167`:

    gui/bundle_flow.go:26      ctx.syswBundleSeeds = []string{body}   → :29  bundleGatherFlow
    gui/multisig.go:97         ctx.syswBundleSeeds = []string{body}   → :99  bundleGatherFlow
    gui/multisig_build.go:136  ctx.syswBundleSeeds = records          → :137 bundleGatherFlow

`engraveSingleSigFlow` (`gui/singlesig.go:38-140`) never writes the field, and
`singleSigVerifyFlow` reaches its gather at `:110` with the field empty. The
readback is genuinely off steel.

**So at `:117` the device had read at least one complete card off the plates and
had run an accounting over it** — `singleSigReadbackCards`
(`gui/singlesig_verify.go:23-42`) — which failed at `:28` (two mk1s), `:33` (two
md1s) or `:38-40` (`len(mk1) == 0 || len(md1) == 0`). A check ran, over read-back
data, with a negative result.

**The innocent world does not make the sentence false.** On the
forgot-the-md1-in-a-drawer case the printed clause is *"or a plate could not be
read or accounted for"* — literally what happened — and the follow-on *"Do NOT
rely on this backup until a full check passes"* is also true: no full check
passed. G2 forbids claiming a check that did not happen; the check happened.
There is no G2 violation here in either direction.

**The bindings agree.** §4.7b lists the positional twin
`gui/multisig_verify.go:701` (`extractReadbackMd1AndMk1s` fails → `verifyRefused`)
in the **adverse** column. Same position in the flow, same kind of check, same
failure shape.

**Strongest sub-case, which the document does not name and which strengthens it:**
the `bundleDonePending` arm at `gui/bundle_flow.go:203-210` *drops* a half-read
chunk set and returns the remaining complete cards. An operator whose md1 tag is
damaged reaches `:117` having had a plate that literally *could not be read*.
That is the clause verbatim.

## PART 2 — THE EIGHT BENIGN ROWS

I looked for the opposite error — a row where the device observed something
adverse about the plates and the mapping says benign. **I found none.** Rows in
the document's numbering:

| row | exit | verified | verdict |
| --- | --- | --- | --- |
| 1 | `:69` | `seedEntryFlowTypedOnly` at `:67`; the gather is 43 lines later at `:110` | **benign, correct.** Nothing read. |
| 2 | `:78` | `singleSigPickFlow` at `:76`, still pre-gather | **benign, correct.** |
| 3 | `:90` | derive failure on the re-typed seed | **benign, correct and BOUND.** §4.8 (plan lines 1115-1119) names this row's classification explicitly via the byte-identical twin at `gui/multisig_verify.go:896-897`, which §4.7b lists benign. I re-confirmed the identity: `:89` and `:896` are the same `showError(ctx, th, "Verify Bundle", "Couldn't re-derive the bundle from the seed.")`. |
| 4 | `:98` | `templateizeBundle` (`gui/template_engrave.go:24-38`) operates only on the freshly re-derived bundle; still pre-gather | **benign, correct.** |
| 5 | `:112` | `bundleGatherFlow` `!ok` ⟹ `:177` (Back) or `:241` (`ctx.Done`) — measured above; the `bundleDoneEmpty` arm shows a screen and **loops** | **benign, correct.** See the note below. |
| 7 | `:125` | `inputCodex32Flow` returns `!ok` only via the Back `break` at `gui/gui.go:1033-1035` falling to `return nil, false` at `gui/gui.go:1113`, or `ctx.Done`; it returns `(obj, true)` only at `gui/gui.go:1040` under `if valid && clicked3` | **benign, correct.** Readback accounting had already *passed*; no negative. |
| 8 | `:130` | type assertion fails ⟹ `validateMStar` (`gui/codex32_polish.go:263-282`) returned a valid non-`codex32.String`, which is only `mdmkText(frag)` for HRP `md`/`mk` | **benign, correct.** See U2 below. |
| 9 | `:138` | `codex32.DecodeMS1` (`codex32/mspayload.go:34-59`) rejected a `New`-valid `ms` string with `errMSBadPrefix` / `errMSBadLength` / `errMSBadLanguage` (declared `codex32/mspayload.go:15-17`) | **benign, correct.** |

### U2 (`:130`, `:138`) — the author chose benign. I agree, and it is the *only* safe answer.

This is the exact failure mode the cycle exists to prevent. At `:130`/`:138`
`verifySingleSig` has **not been called**; nothing was compared against anything.
Printing *"A verification check ran and did not pass"* would claim a check the
device never performed — §4.8 (plan lines 1104-1110) names that in so many words
as the G2 violation. Unlike `:117`, **no clause of the adverse line is true
here**: no comparison was made, and both plates *were* read and accounted for
successfully at `:114`.

The multisig treatment agrees, and more strongly than the document claims.
`multisigVerifyMS1Entry` (`gui/multisig_verify.go:1004-1022`) shows the same two
screens verbatim (`:1012`, `:1017`) and returns `rejected = true`; the caller
`break`s at `:887`. That break lands on `:938` `verifyIncomplete` when
`len(legs) == 0` — §4.7b's **benign** column — and on the partial path at
`:962-979` when legs already exist, whose terminal `:979` is **also** in §4.7b's
benign column. Both destinations are benign.

### U3 (`:112`) — benign is right; the author's own worry is the correct one to record.

`bundleGatherFlow` can hold accepted cards in `scr.g.cards` before the Back at
`gui/bundle_flow.go:176-178`, so "nothing observed" is loose. But observation is
not the bit — a **negative result from a check** is, and the accounting at
`:114` never ran. §4.7b's twin `gui/multisig_verify.go:696` is benign with the
in-code rationale I confirmed at `gui/multisig_verify.go:694-695`: *"It is an
ABANDON rather than an incomplete: nothing was compared."*

I also tested the sharpest version of this the document does not raise:
`bundleDoneDecision` **can** return `bundleDoneEmpty` — an accounting that
produced a negative ("No complete cards yet") — before the operator presses
Back. It does not make `:112` adverse: that arm `showError`s and **continues the
loop**, so the exit actually taken is the Back at `:177`, and the record is
written at the exit. Classifying an empty-reader walkaway as *"a verification
check ran and did not pass"* would be closer to the false-sentence failure mode
than to a true one. Benign stands.

## PART 3 — THE DECIDABLE RULE vs §4.7b's TABLE

The document claims its rule *"reproduces §4.7b's multisig table row for row."*
**I tested every row of that table against the code and the claim holds.** Line
citations in §4.7b all resolve at the pinned SHA.

| §4.7b adverse row | what the code does | comparison/accounting over readback? | negative? | rule says |
| --- | --- | --- | --- | --- |
| `:719` foreign-or-garbled md1 | `if !slices.Equal(readbackMd1, engravedMd1)` (`:717`) | yes — exact chunk equality | yes | ADVERSE ✓ |
| `:724` readback will not decode | `md.ExpandWalletPolicyChunks(readbackMd1)` err | "could not be read" (see N-1) | yes | ADVERSE ✓ |
| `:394` `errVerifyLegHasNoPlate` | leg/plate bijection | yes — accounting | yes | ADVERSE ✓ |
| `:738` plate count ≠ engraved count | `len(readbackMk1s) != len(expectedSlots)` | yes — accounting | yes | ADVERSE ✓ |
| any `bundle.Verify` error | comparator | yes | yes | ADVERSE ✓ |
| `:701` readback filter drops cards | `extractReadbackMd1AndMk1s` `!ok` | yes — accounting | yes | ADVERSE ✓ |
| `:963` `verifyMultisigLegsPartial` mismatch | comparator | yes | yes | ADVERSE ✓ |
| `:984` `verifyMultisigLegs` mismatch | comparator | yes | yes | ADVERSE ✓ |

| §4.7b benign row | what the code does | rule says |
| --- | --- | --- |
| `:897` re-typed seed will not derive | `deriveMultisigLeg` err inside the leg loop. The md1 equality check at `:717` had already **passed** — so a check ran with a *positive* result, and the rule's second conjunct fails | BENIGN ✓ |
| `:938` zero legs, correctable | no comparison ran | BENIGN ✓ |
| `:940`, `:696` abandons | no comparison ran | BENIGN ✓ |
| `:670`, `:680`, `:794` structural refusals | `len(expectedSlots)==0`, `len(engravedMd1)==0`, `verifyFreshSlots` → `errVerifyNoExpectedSlots` (`gui/multisig_verify.go:318-321`) — all facts about the *obligation list*, not about readback | BENIGN ✓ |
| `:979` partial verify, everything matched | `verifyMultisigLegsPartial` returned **nil** (`:962`); the every-plate-claimed sweep was **skipped**, not failed | BENIGN ✓ |

**The two the brief warned about are exactly the two the rule handles cleanly.**
`:897` and `:979` both look adverse and are not, and in both the rule's
*negative-result* conjunct is what excludes them — `:897` because the check that
ran passed, `:979` because the check that would have failed never ran. That is
the property that makes the rule fail safe rather than merely fail.

**No disagreement found.** The rule is sound, and the single-sig rows derived
from it inherit that soundness. N-1 is the only wording gap, and it does not
change a verdict anywhere.

## PART 4 — ARTIFACT (b)

**Accepted against all three of §4.8's criteria.**

### Names in scope at `:987`

Verified against the file, not against the document:

    662	func multisigVerifyFlow(ctx *Context, th *Colors, full bool, expectedSlots []int, engravedMd1 []string) multisigVerifyResult {
    721		_, keys, err := md.ExpandWalletPolicyChunks(readbackMd1)
    768		covered := make(map[int]bool, len(keys))
    987		return verifyComplete

`:987` is the last statement of the function (`:988` is `}`), at function-body
scope. `grep -nE "\bkeys\b" gui/multisig_verify.go` shows `keys` bound **once**
inside `662-988` (at `:721`) and read at `:768`, `:790`, `:820`, `:832`, `:894`
— **no shadowing rebind**, confirming the document's claim. Criterion 1 met.

### Indexing

`covered`'s domain is indices into `keys`, and this is the load-bearing fact:
`allUserSlots` (`gui/multisig_match.go:78-97`) builds `matches` as
`matches = append(matches, i)` over `for i, k := range keys`, and `:894` indexes
`keys[s].OriginPath` with the same `s`. `verifyFreshSlots`
(`gui/multisig_verify.go:318-330`) only filters that set. So
`for i := range keys { if !covered[i] { n++ } }` indexes correctly.

### It cannot under-report — the G2 direction

Three independent reasons, all checked:

1. **`covered` has exactly one write in the file** — confirmed by
   `grep -n "covered\[" gui/multisig_verify.go` → `:324` (a read, on
   `verifyFreshSlots`' own parameter), `:900` (`covered[s] = true`), `:971` (a
   read). Nothing sets `false`, nothing deletes.
2. **Every covered slot at `:987` corresponds to a leg that actually verified.**
   `covered[s] = true` at `:900` is written next to `legs = append(...)` at
   `:899`, at which point the leg has only been *derived*. But `:987` is reached
   only past `if err := verifyMultisigLegs(legs, readbackMk1s, readbackMd1); err
   != nil { ... return verifyFailed }` at `:983-985`. So on the success return,
   every leg — and therefore every covered index — has passed the plate
   comparison.
3. **The iteration direction is the safety property.** Iterating `keys` and
   asking `!covered[i]` means a stray entry in `covered` outside
   `[0, len(keys))` is *ignored* (cannot shrink the count), and a wrongly-absent
   entry *inflates* it. Every defect in `covered` therefore over-reports supplied
   cosigners, which renders a clause saying *less was checked*. Criterion 3 met.

I also checked the direction on a **template** multisig md1: `allUserSlots`
skips `!k.XpubPresent` keys, so a keyless slot can never be covered and is
counted as uncovered. Over-report — the safe direction.

**The rejected one-liner is rightly rejected, and for the right reason.**
`len(keys) - len(covered)` is arithmetically identical at `:987` today (at that
point `len(legs) == len(expectedSlots)`, since the `len(legs) <
len(expectedSlots)` branch at `:969` returns and the loop's own exit condition is
`>=`; and `covered` gains exactly one entry per leg in the same iteration). Its
correctness rests on a cardinality invariant held elsewhere, and it fails in the
**wrong** direction: an extra entry in `covered` silently shrinks the count. The
document's stated reason is exactly this, and it is correct.

### Single-sig writing literal `0` — correct

Criterion 2 met, and by construction rather than by omission.
`deriveSingleSigBundle` (`gui/singlesig_derive.go:37`) derives from **one** seed
at **one** path, and the readback demands exactly one mk1 and one md1
(`gui/singlesig_verify.go:114-118`). There is no key in the descriptor this run
did not itself derive and compare, so there is nothing for the count to miss —
it cannot under-report because the true value is 0. The controller's grep
(`cosign|policy|covered` in `gui/singlesig_verify.go` → 0 hits) is consistent.

And 0 is the value that makes the clause **absent**, which §4.7b-seam requires:
`Other cosigners' keys are taken as supplied.` renders iff
`rec.pass.suppliedCosigners > 0` (§4.7c, plan line 938), and
`TestMultisigVerifyNoticeIsHonest` (`gui/multisig_verify_test.go:171`) pins the
split on the screen side.

### One check the document makes that I re-ran because it is load-bearing for G1

`legs: 1` on a **template** engrave. `templateizeBundle`
(`gui/template_engrave.go:24-38`) returns
`bundle.Bundle{MS1: b.MS1, MK1: mk1, MD1: tmplMD1}` where `mk1` comes from
`reStubMk1(b.MK1, stub)` at `:33` — the key card is **re-stubbed, not removed**.
The claim is true; `legs: 1` holds in both forms.

## PART 5 — U4, AND A RECOMMENDATION

**U4's observation is correct — and the plan already ruled on it at R15.**

The observation checks out. `open := p.N - len(p.SelfSlots)` at
`gui/multisig_build.go:96`, so a build where the operator holds every policy
slot yields `open == 0`; `buildSlotSources` (called at
`gui/multisig_build.go:240`) and `buildEngraveTail` (called at
`gui/multisig_build.go:384`, declared `gui/multisig_build_tail.go:53`) then
engrave every slot, `expectedSlots` covers every index of `keys`, all legs
verify, `covered` covers every key, and `countUncoveredPolicyKeys` returns **0**
on a *multisig* run. Both of U4's cited call-site line numbers are correct.

**U4 is also right that this makes `suppliedCosigners` a coverage axis rather
than the "path axis" §4.7b-seam calls it** (plan lines 828-841), and right that
this is not a defect: on such a run nothing *was* taken as supplied, so omitting
the clause is the truthful line, and §4.7b-seam's stated harm — *"omitting it is
an unscoped claim on the multisig document"* — does not arise when there is no
unverified cosigner to scope. Under G2 the field still behaves correctly:
it can only over-report, and 0 here is a true 0.

**It does not break T27.** The plan's §5 T27 row (plan line 1199) already
carries this exact case, verbatim:

> **NON-VACUITY (R15): the multisig half needs a fixture where at least one
> policy key is NOT covered by a verified leg, so `suppliedCosigners > 0`. The
> self-multisig "operator holds every slot" fixture yields `open == 0`
> (`gui/multisig_build.go:96`) and therefore `suppliedCosigners == 0`, on which
> T27 passes while asserting nothing.**

T27's assertion that a multisig full pass line *includes* the cosigner clause is
therefore already scoped to a fixture with an uncovered key, and the plan already
requires naming or building one at step 7. U4 rediscovered a resolved finding.

**Recommendation: no change to either artifact, and no decision is owed before
step 2.** Keep the expression as written, keep single-sig's literal `0`, and
treat U4 as a pointer to the T27 non-vacuity obligation the plan already owns —
adding a citation to plan line 1199 inside U4 (N-3) so the next reader does not
re-open it. Anything beyond that — a `template`/`multisig` discriminator, a
distinct "path" field, or a document line reporting *why* the count is 0 — would
add a field describing the device's epistemic state and is **NG1, out of scope
by default even if correct**.

---

## SCOPE NOTE

Nothing in this review proposes adding a state, a field, or a finer distinction
to what the document reports about the device's knowledge. The three filed items
are wording and citation notes on rationale prose. No file was modified; nothing
was committed; `/scratch/code/shibboleth/seedhammer` was read-only throughout.
