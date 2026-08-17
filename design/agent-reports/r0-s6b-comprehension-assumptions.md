# R0 — `SPEC_s6b_pre_flash_cycle.md`, COMPREHENSION + UNFOUNDED-ASSUMPTIONS lens

**Artifact:** `design/SPEC_s6b_pre_flash_cycle.md` (post round-3 fold).
**Source:** fork `bg002h/seedhammer`, `main` = `b1479a1b38f6b045d27443764c858906e4e6e122`
(`git rev-parse HEAD` re-verified, tree clean). All `gui/*.go`, `backup/*.go`,
`md/*.go` citations below are that repo, verified by direct read.

**Lens, per brief:** (1) could a competent implementer read this and build the
WRONG thing; (2) what does it take as true without establishing it. **Not** a
correctness round — rounds 1–3 are not relitigated, and R-A…R-M are operator
decisions, not re-opened.

Prefix `X` on every id so nothing collides with round 1's C1/C2/C3.

---

## 1. Findings

| id | sev | lens | section | one line |
| --- | --- | --- | --- | --- |
| **XC1** | Critical | comprehension | §2.3a, §7 | R-J is presented as an **OPEN, unruled** question in two places while §2.3/§2.3d implement it as NORMATIVE — the two readings ship different footers, one of them C1's falsehood |
| **XC2** | Critical | assumption | §0 / R1 | the restore document already says *"nothing this device engraves carries a passphrase"* — false on **exactly** the runs S6b's new offer fires; nothing in the spec touches it |
| **XC3** | Critical | comprehension | GATE 2.3e | "the inventory reflects whether a passphrase plate was engraved" has no mechanism, and the obvious one folds the passphrase plate into *"If any of them is missing, this backup is incomplete"* — the opposite of R1 |
| **XI1** | Important | assumption | §2.1 | *"the machinery is already shaped for it"* is false for R-J/R-H: neither `engravePassphraseFlowFrom` nor `ppBuildPlate` can carry preloaded fingerprints or a policy id |
| **XI2** | Important | comprehension | §2.3a | **GATE 2.3 and GATE 2.3c live inside a section headed "OPEN … Not folded"** — two live plate-safety gates parked where an implementer skips |
| **XI3** | Important | comprehension | §2.3e | the closing paragraph specifies a **27-character** preloaded footer; §2.3's live block specifies a **36-character** one. Two normative footers |
| **XI4** | Important | comprehension | §1.3 | *"Every other caller passes nothing and is byte-unchanged"* is impossible in Go; the readings that make it literally true are variadic or shared state, and shared state crosses R-B |
| **XI5** | Important | assumption | §5.1 | the normative predicate `bodysz.Y > bodyClip.Dy()` under-reports by exactly **10 px**, in the direction that hides content with no arrow |
| **XI6** | Important | assumption | §1.1 / §1.2a | nothing states the title is **plate row 0** and the footer the **last plate row**; the 25-char budget holds only for those rows |
| **XI7** | Important | assumption | §2.3 | what **selects** between the two footer strings is unspecified; `backup.Passphrase` has no notion of "preloaded" |
| **XM1** | Minor | comprehension | §2.3d | the section's opening sentence is superseded text sitting under a **NORMATIVE** heading |
| **XM2** | Minor | comprehension | §0, §1.2, §6, §2.1 | bare `§2.1 / §2.2 / §2.4 / §7.4` references point into **three different documents** and collide with the spec's own section numbers |
| **XM3** | Minor | assumption | §1.2 | `COMB FP:` is engraved on **no-passphrase** sets too, where no "combination" exists |
| **XM4** | Minor | comprehension | §3.3 | the replacement clause string is never written, and §7 does not list it as unsettled (unlike §3.2's arm, which it does) |
| **XM5** | Minor | comprehension | GATE 3.3 | "1 seed / 2 legs" and "2 seeds / 2 legs" are **indistinguishable** in `passRecord` — a unit test can only reach two of the three cases |
| **XM6** | Minor | comprehension | §1.2, GATE 1.2 | the marking's behaviour on the operator-selectable **QR-ONLY** variant is unstated |
| **XM7** | Minor | comprehension | §2.4, §2.3e | nothing says the policy stub must be computed from the **post-`templateizeBundle`** `b.MD1` |
| **XN1** | Nit | comprehension | §3.2a | `multisigVerifyNoSlotBody`'s doc comment (`gui/multisig_verify.go:150-156`) describes the arm R-M replaces — stale on landing |
| **XN2** | Nit | comprehension | §2.3 | `POLICY xxxx xxxx  DERIVED, NOT TYPED` puts its subject (the fingerprints) in the **opposite band** of the plate. R-H is an operator ruling — recorded, not re-opened |

---

## 2. Findings in full

### XC1 — Critical — comprehension — §2.3a and §7 say R-J is unruled; §2.3/§2.3d say it is normative

**What it is.** R-J (`REQUIREMENTS §2bis`) rules **yes**: the device preloads the
fingerprints, `ppStepSeedFP`/`ppStepCombinedFP` are skipped, and the footer may
say `DERIVED, NOT TYPED`. §2.3's live block and §2.3d both implement that. But
two later passages still present it as an open question:

- §2.3a is headed **"OPEN — should the device preload the fingerprints too?"**
  and opens *"Not folded, because it grows R-C's scope and that is the operator's
  call"*, closing with *"**Until that is ruled on, the normative text above
  stands**, because it is the only version that is true of the code as it
  exists."*
- §7 ("WHAT THIS SPEC DOES NOT SETTLE") repeats it as a live bullet: *"§2.3a —
  should the device preload the FINGERPRINTS as well as the passphrase? … it
  grows R-C's scope, so it is the operator's call. **Until then §2.3's
  truth-preserving footer stands.**"*

Both sentences are pre-R-J text. "The normative text above" and "§2.3's
truth-preserving footer" both denote the **superseded** `FINGERPRINTS TYPED, NOT
VERIFIED` version — the phrase "truth-preserving" is the C1-fold's own
description of it, and it is now attached to a section that says the opposite.

**What breaks.** An implementer who reads §7 first — the section whose entire job
is to say what is not decided — concludes the fingerprints are **not** preloaded,
leaves `fingerprintEntryFlow` on the path, and then still lands R-H's ruled
footer (R-H *is* a ruling, and §2.3 prints it). That is
`POLICY … DERIVED, NOT TYPED` engraved on steel while the operator typed the
fingerprints — round 1's C1 verbatim, on the cycle built to close that class,
permanent, on a plate read years later. GATE 2.3b is the backstop, but the
implementer writing that gate is reading the same contradiction.

**Cheapest fix.** Delete §7's bullet and re-head §2.3a as
`2.3a — RESOLVED by R-J` (or fold its two surviving gates into §2.3, see XI2).
One-line check that the staleness is gone:
`grep -n "operator's call\|Until that is ruled\|Until then" design/SPEC_s6b_pre_flash_cycle.md`
must return nothing about the fingerprints.

---

### XC2 — Critical — assumption — the restore document's shipped passphrase paragraph becomes false the moment R2 ships

**What is assumed.** `REQUIREMENTS §1` records R1 as *"already shipped —
`buildPassphraseInventoryLines`"*, and the spec inherits that: R1 appears nowhere
in §0's scope, in any section, or in any gate. The assumption is that R2
(offering to engrave the passphrase) leaves R1 intact.

**It does not.** The shipped text is, verbatim
(`gui/multisig_build_census.go:273-279`):

```
"A BIP-39 passphrase WAS used. It is not on these plates and cannot be
 recovered from them: nothing this device engraves carries a passphrase."
"Without it, these plates do not reach the money. Keep it somewhere
 separate, and make sure whoever needs this backup can also get the
 passphrase."
```

The reachability is **total**, not incidental. `engraveSingleSigFlow` passes
`oneSeedPassphraseFact(passphrase != "")` at `gui/singlesig.go:223`;
`buildPassphraseInventoryLines` (`gui/multisig_build_census.go:258-279`) prints
those two lines iff that flag is true. GATE 2.3e makes the passphrase-plate offer
appear **iff `passphrase != ""`** — the *same* predicate. So on every run where
S6b can cut a passphrase plate, the document printed minutes later asserts that
this device engraves no passphrase.

Stated honestly: `engravePassphraseFlow` is already on the carousel, so the
sentence is arguably already loose today. S6b converts that into a guaranteed
same-run contradiction — the operator is offered the engrave, cuts the plate, and
is then handed a page saying the device does not do that.

**What breaks.** R-D (*"all things said must be true"*) on the one artifact
designed to outlive everybody who could correct it — the F-198 class the cycle
exists to close.

**Cheapest way to establish it.**

```sh
grep -n "nothing this device engraves carries a passphrase" gui/multisig_build_census.go
grep -n "oneSeedPassphraseFact" gui/singlesig.go
```

Two hits, one predicate. The fix is a conditional clause on that paragraph plus a
gate; it is text-only and does not cross R-B (the multisig build path reaches the
same function, so the clause must be keyed on whether *this run* cut a passphrase
plate, not on the flow).

---

### XC3 — Critical — comprehension — GATE 2.3e's inventory requirement has no mechanism, and the obvious mechanism contradicts R1

**What it is.** GATE 2.3e: *"the restore document's plate inventory reflects
whether a passphrase plate was actually engraved."* §2.3e justifies the insertion
point with *"the document's inventory already takes
`oneSeedPassphraseFact(passphrase != "")` (`:223`) — so if a passphrase plate is
cut, the inventory must be able to say so."*

It cannot, as it stands. Verified signature
(`gui/multisig_build_census.go:59-73`):

```go
func buildPlateInventoryLines(cards []bundleCard, seeds []seedPassphraseFact,
	capacity seedCapacity) []string {
	plan := bundlePlatePlan(cards)
	lines := []string{
		fmt.Sprintf("This backup is %s:", plateWord(len(plan), "plate", "plates")),
	}
	...
	lines = append(lines, "If any of them is missing, this backup is incomplete.")
```

`seedPassphraseFact` carries `{Label, Uses}` — *whether a passphrase was used*,
never *whether one was engraved*. The passphrase plate is cut by a different
program (`engravePassphraseFlowFrom`) and produces no `bundleCard`. So the gate
requires a data path that does not exist and the spec never specifies.

**What breaks, and why the wrong guess is expensive.** The nearest mechanism —
append a `bundleCard` for the passphrase plate — puts it inside the plate count
and under *"If any of them is missing, this backup is incomplete."* That tells a
future reader the passphrase plate belongs **with** the set, which is the exact
inverse of R1's *"Keep it somewhere separate"* printed four lines below it, and
it would also flow into `bundleSetCarriesASecret(cards)`
(`gui/bundle_flow.go:492`). A document that contradicts itself about whether the
passphrase plate travels with the steel is a funds-safety defect, and the gate as
worded would report green either way — the inventory *did* reflect it.

**Cheapest fix.** Decide it in the spec, in one sentence: the passphrase plate is
**not** a member of the backup set and is named in a separate line that repeats
the separation instruction. Then GATE 2.3e asserts that specific line and asserts
`len(plan)` is unchanged. Cheapest check that the hazard is real:
`grep -n "This backup is\|is incomplete" gui/multisig_build_census.go`.

---

### XI1 — Important — assumption — §2.1's "the machinery is already shaped for it" does not survive R-J or R-H

**What is assumed.** §2.1: *"It runs the existing dedicated passphrase-plate
program with the passphrase already in hand, via
`engravePassphraseFlowFrom(ctx, th, body []byte, src syswSource)`
(`gui/passphrase_flow.go:617`), which **already takes a body and a
provenance**."* That is true and sufficient for R-C alone. It is asserted as the
reason no new mechanism is needed, and R-J and R-H then add three values it
cannot carry.

**Verified against source.**

- `engravePassphraseFlowFrom` (`gui/passphrase_flow.go:617`) takes exactly
  `(ctx, th, body []byte, src syswSource)` — no fingerprints, no policy id.
- `ppBuildPlate` (`gui/passphrase_flow.go:546`) takes
  `(params, secret []byte, seedFP, combinedFP string, qr bool)` and builds
  `backup.Passphrase{Passphrase, SeedFP, CombinedFP, QR, Font}`
  (`backup/passphrase.go:23-31`) — no policy-id field.
- `seedFP`/`combinedFP` are `var`-declared inside the function
  (`gui/passphrase_flow.go:645`) and written **only** by `ppStepSeedFP` /
  `ppStepCombinedFP` and `ppPassProofLoader`.
- The step machine is an integer loop with `step -= 2; break` as its Back
  transition and `step++` at the bottom (`gui/passphrase_flow.go:656-706`).
  "Skipping" two adjacent steps is not "don't run them": the Back arithmetic from
  `ppStepQR` lands on `ppStepCombinedFP`, and the spec does not say what happens
  then.

**What breaks.** This is the shape round 1's C3 and round 2's N1 both had — a
specified output owning no mechanism. It will not ship as a no-op (GATE
2.3b/2.3c/2.4b are outcome-shaped, so a missing policy id fails them), but an
implementer will invent a data path under time pressure, and the cheap inventions
here are bad ones: a package-level variable, or a field on `Context`, either of
which makes a **secret-adjacent** value outlive one flow.

**Cheapest fix.** One sentence in §2.1 naming what changes: the preloaded entry
carries seed FP, combined FP and the policy-id hex as parameters, and the two
fingerprint steps are elided from the step sequence rather than skipped inside it.
Cheapest check of the premise:
`grep -n "func engravePassphraseFlowFrom\|func ppBuildPlate" gui/passphrase_flow.go`.

---

### XI2 — Important — comprehension — GATE 2.3 and GATE 2.3c are defined inside a section headed "OPEN … Not folded"

**What it is.** Section order in the file is §2.3 → §2.3d → §2.3e → **§2.3a**.
GATE 2.3b sits at the end of §2.3e; **GATE 2.3 and GATE 2.3c sit inside §2.3a**,
whose heading is *"2.3a OPEN — should the device preload the fingerprints too?"*
and whose second line is *"Not folded, because it grows R-C's scope."*

Those two are not optional:

- **GATE 2.3** is the one preventing `FINGERPRINTS TYPED, NOT VERIFIED` (32) and
  a policy id co-occurring — **50 characters against a 42-character band**,
  rendered off both plate edges with no refusal. Verified: `band`
  (`backup/passphrase.go:228-235`) centres at `(plateX-w)/2` and has no refusal
  of any kind.
- **GATE 2.3c** is the one catching the policy id vanishing with an empty
  fingerprint. Verified: `backup/passphrase.go:185-187` appends the footer only
  when `plate.SeedFP != "" || plate.CombinedFP != ""`.

**What breaks.** A reader who treats a section marked OPEN/not-folded as
not-yet-implementable loses both gates, and §6's table gives them no protection —
the table rows point back at these definitions. The failure of GATE 2.3 is silent
and permanent: ink off both plate edges.

**Cheapest fix.** Move GATE 2.3 and GATE 2.3c under §2.3 (their subject), which
also resolves the ordering wobble; §2.3a then holds only the resolved-by-R-J note
from XC1. Verify with
`grep -n "^### 2\.3\|GATE 2\.3" design/SPEC_s6b_pre_flash_cycle.md` — every gate
should sit under the section it belongs to.

---

### XI3 — Important — comprehension — §2.3e's closing paragraph specifies a different footer from §2.3

**What it is.** §2.3's live NORMATIVE block (post-R-J):

```
preloaded    "POLICY <8 hex, grouped>  DERIVED, NOT TYPED"  36 chars -> fits (42 budget)
```

§2.3e's closing paragraph, four subsections later:

> *"The preloaded footer states the policy binding **and** the fingerprints'
> true provenance. **27 characters leaves 15 of headroom**; the denser
> `"POLICY 1A2B 3C4D  FPS TYPED, NOT VERIFIED"` is 41 against 42 and is
> **rejected for having one character of headroom**."*

Measured (`python3`, exact): `POLICY 1A2B 3C4D  DERIVED, NOT TYPED` = **36**;
`POLICY 1A2B 3C4D  FPS TYPED` = **27**; `POLICY 1A2B 3C4D  FPS TYPED, NOT
VERIFIED` = **41**. So the 27/15 paragraph is the **pre-R-J** fallback string —
the one §2.3 names only as a contingency (*"if the fingerprint entry steps
remain … this string reverts to `POLICY <8 hex, grouped>  FPS TYPED` (27
chars)"*) — orphaned into §2.3e and reading as settled.

**What breaks.** An implementer resolving §2.3 against §2.3e can land the 27-char
form. Under §2.3d the fingerprints are **derived**, so `FPS TYPED` is an
affirmative falsehood — R-D, and the same direction as round 1's C1. GATE 2.3b
tests the *derivation* claim, not the *typed* claim, so it does not catch this
direction.

**Cheapest fix.** Move that paragraph back under §2.3 and rewrite its numbers for
the 36-char string, or mark it as the contingency it is.

---

### XI4 — Important — comprehension — "passes nothing and is byte-unchanged" is not achievable in Go

**What it is.** §1.3: *"`bundleEngrave` grows two optional strings, passed
through to `validateMdmk`. `gui/singlesig.go:177` is the only caller that passes
non-empty values … **Every other caller passes nothing and is byte-unchanged.**"*

Go has no default parameters — a fact this very codebase records at the function
under discussion (`gui/bundle_flow.go:395-397`: *"R0-M2: Go has no default
params; deriveXpubFlow's own call to it … stays BYTE-UNCHANGED"*). Verified, the
four production callers are `gui/singlesig.go:177`, `gui/multisig.go:291`,
`gui/multisig_build.go:402`, `gui/bundle_flow.go:39`; `validateMdmk` has four
more (`gui/bundle_flow.go:407`, `gui/gui.go:2344`, `gui/unlock_platelist.go:222`,
`gui/derive_xpub.go:494`). A two-parameter addition edits all of them.

It is also contradicted by tests that pin the **call text as a source string**:

```
gui/multisig_verify_report_test.go:940  `if bundleEngrave(ctx, th, "Engrave Multisig", cardsOut) != bundleEngraveDone {`
gui/multisig_verify_report_test.go:942  `if bundleEngrave(ctx, th, "Build Policy", cardsOut) != bundleEngraveDone {`
gui/bundle_abort_prose_test.go:258      strings.Index(src, "bundleEngrave(ctx, th, ")
```

**What breaks.** The two readings that make "byte-unchanged" literally true are
(a) a variadic `...string` tail — order- and arity-unchecked on a value that
decides whether a plate says `PASSWORD REQUIRED`; or (b) shared state on
`Context` / a package var, which would let the marking reach `"Engrave
Multisig"` and `"Build Policy"` and cross the R-B boundary §1.3 exists to make
structural. GATE 1.3 catches (b) if it is written; nothing catches (a).

**Cheapest fix.** Replace "passes nothing and is byte-unchanged" with "passes
`""`, `""`" and note that the three source-text assertions above must be updated
in the same commit. `grep -rn 'bundleEngrave(ctx, th, ' --include="*_test.go" gui/`
enumerates them.

---

### XI5 — Important — assumption — §5.1's normative predicate under-reports actual visibility by 10 px

**What is assumed.** §5.1 hands over an expression as NORMATIVE:

```
show arrows  iff  bodysz.Y > bodyClip.Dy()
```

and names one residual only — F-95's few pixels of overdraw past
`bodyClip.Max.Y`.

**Measured against source** (`gui/gui.go:399-417`, `gui/theme.go:43`,
`gui/gui.go:761`):

| quantity | value |
| --- | --- |
| `bodyClip.Min.Y` = `leadingSize` | 44 |
| `bodyClip.Max.Y` = `dims.Y - boxMargin` | 314 |
| `bodyClip.Dy()` | **270** |
| body's drawn top: `bodyClip.Min.Y + scrollFadeDist` (`:416`, at `scroll == 0`) | **60** |
| panel bottom (`dims.Y`) | 320 |

Content is off the panel iff `60 + bodysz.Y > 320`, i.e. **`bodysz.Y > 260`**.
The spec's predicate fires at `bodysz.Y > 270`. The gap is
`scrollFadeDist - boxMargin` = `16 - 6` = **10 px**, constant in `dims`, and it
is a **false negative**: content is drawn below the panel edge and **no arrow is
shown**. That is F-185's own harm — a line of a safety modal the operator cannot
read and is not told exists — which is precisely what round 1's I4 objected to
elsewhere in §5.

The spec's stated residual points the other way (`bodyClip.Max.Y` = 314 vs a body
reaching 317, both **inside** the 320-px panel and therefore visible), so it does
not cover this.

**What breaks.** GATE 5.1 (*"the new predicate agrees with actual visibility —
MUST BE GREEN"*) will fail against a correctly written test. The dangerous
resolution is the one §5's own history warns about: writing the gate against the
handed-over expression instead of against visibility, which is exactly the
false-PASS gate GATE 5.1b was split off to prevent.

**Cheapest way to establish it.** One test that lays out a body with
`bodysz.Y` in `(260, 270]` and asserts whether the last drawn row's `y` exceeds
`dims.Y`. If it does, the expression is short by 10 and the spec should state the
predicate against the panel (`dims.Y`) rather than against `bodyClip.Dy()`, which
is what R-E already told it to do (*"define the arrow's predicate against what is
ACTUALLY VISIBLE — the panel, not `bodyClip`"*).

---

### XI6 — Important — assumption — the 25-character budget holds only if the title is plate row 0 and the footer the last plate row, and neither is stated

**What is assumed.** §1.2a derives the budget from
`[innerMargin, plateSize - innerMargin]` = 416000 units — *"the bound
`TestTitleCapFitsAtEveryRung` uses"*. That test
(`backup/freetext_test.go:105-135`) builds a **`Fitted`**, whose `Title`/`Footer`
are documented as *"The screw-hole rows. Title takes plate row 0 and Footer the
last row"* (`backup/fit.go:117-120`). §1.1 instead describes the new fields as
*"rendered through the layout helpers `EngraveText` **already shares** with
`Fitted`"*. Neither section says where on a `Text` plate the two rows sit.

**The mechanism does support it, and that is why this is a gap and not a false
number.** `lineLayout.at` (`backup/wrap.go:~145-165`) applies a `holeChars`
inset to any row whose y falls inside either `innerMargin` band, and `EngraveText`
starts its paragraphs at `offy := params.I(outerMargin)`
(`backup/backup.go:361`) — so a `Text` plate's **row 0** and its **last plate
row** do get exactly the screw-hole treatment the 25-char figure assumes.

**What breaks if the reading differs.** The other natural reading — "render a
footer row directly after the paragraph text" — puts the footer **mid-plate**,
where it is not a hole row (so 25 is the wrong bound) and where it falls inside
the QR keep-out band: `qrPlaceAt` (`backup/wrap.go:196-213`) sets
`Top = anchorY + holeLines*fontSize`, i.e. the QR begins at row 2 on these
plates, and `textLayout` narrows every line in `[qrTop, qrBottom)` by
`KeepOutX`. §2.4 has already established that **nothing in the render layer
bounds a title** — it is engraved, not refused — so a footer laid out against the
wrong span is ink over a QR or in a screw-hole band, permanently.

**Cheapest fix.** One clause in §1.1: *the title is plate row 0 and the footer the
last plate row.* Then GATE 1.2a's layout-based assertion is unambiguous about
which rows it measures.

---

### XI7 — Important — assumption — nothing says what selects between the two footer strings

**What it is.** §2.3 gives two footers keyed on *"preloaded"* vs *"standalone"*.
The renderer has no such concept: `passphraseLayoutFor`
(`backup/passphrase.go:185-187`) appends the single `const passphraseFooter`
(`:156`) whenever either fingerprint is non-empty, and `backup.Passphrase`
(`:23-31`) carries `{Passphrase, SeedFP, CombinedFP, QR, Font}` — nothing that
distinguishes the paths. §2.4 says only *"The standalone path has no descriptor,
passes `""`, and renders no line."*

**What breaks.** The cheapest discriminator an implementer reaches for is
`PolicyID != ""` — which couples two independent facts (the plate is
policy-bound; the fingerprints were derived). Any future path that preloads
fingerprints without a descriptor then prints `FINGERPRINTS TYPED, NOT VERIFIED`
over fingerprints the device derived: an affirmative falsehood in the direction
GATE 2.3b does **not** test (2.3b tests the derivation claim, not the typed
claim). The invariant "policy id present ⟺ fingerprints derived" is assumed and
nothing enforces it.

**Cheapest fix.** Name the discriminator in §2.3 — a separate field on
`backup.Passphrase` recording fingerprint provenance — and say that the policy id
and the provenance are independent inputs. `grep -n "passphraseFooter"
backup/passphrase.go` shows the two sites that must change together.

---

## 3. Minors and Nits

**XM1 — §2.3d's opening sentence is superseded text under a NORMATIVE heading.**
The heading reads *"2.3d NORMATIVE — where the two derivations happen (R-J)"* and
its first line is *"Both fingerprints are derived **back-to-back at
`gui/singlesig.go:107`**"*. Four paragraphs later: *"**NORMATIVE (revised):
derive the bare-seed fingerprint LAZILY**"*. A skimmer looking for the bold
NORMATIVE finds the wrong one first and pays a ~31 s KDF on **every** single-sig
engrave. Non-gating because GATE 2.3d asserts the revised behaviour directly.
Fix: strike the opening sentence.

**XM2 — bare section references resolve into three different documents.**
`§2.1`, `§2.2`, `§2.4` and `§2.5` are used to cite `REQUIREMENTS_s6b…`, while the
spec has its own §2.1 ("R2 — the program runs preloaded"), §2.2 ("A new
`syswSource` value is REQUIRED"), §2.4 ("The identifiers") and §2.5 ("R7 — the QR
stays password-only"). Every one of those collides. §6's *"§2.4 establishes that
nothing in the render layer bounds a title"* resolves to the **requirements**
§2.4, not the spec's. Worse, `§7.4` (§2.1 and §3.2) names a **third** document —
grep places it in `SPEC_seedhammer_systemwide_payloads` / its plan, and neither
S6b document defines it. Fix: qualify each with its document.

**XM3 — `COMB FP:` is engraved on no-passphrase sets.** §1.2's footer fires
*"iff the set contains a seed"* (R-A) — independent of the passphrase. On a
bare-seed full engrave the plate carries `COMB FP: 73C5 DA0A` with **no**
`PASSWORD REQUIRED` title and no `SEED FP` line to contrast against, so a reader
years later sees a label naming a combination that does not exist. Not false —
the value is the master fingerprint — but it names a distinction the plate does
not have. Worth deciding rather than discovering. (R-A itself is out of scope;
the *label* is not ruled.)

**XM4 — §3.3's replacement clause is never written down.** §3.3 says only *"The
document clause adopts that shape"* of `multisigVerifyOKMessage`
(`gui/multisig_verify.go:1134-1135`, verified: *"the ms1 you typed for each
seed"*). That message is a screen line; the document clause it replaces
(`verifyStatusMS1Clause`, `gui/verify_status.go:155`) also asserts a **match**.
So new operator-facing text under R-D is left to the implementer, and §7 does not
list it as unsettled — even though §7 *does* list §3.2's no-passphrase arm, which
is the same situation.

**XM5 — GATE 3.3's three cases are two.** `passRecord`
(`gui/multisig_verify.go:1057-1061`) is `{full, legs, suppliedCosigners}` — no
seed count, as §3.3 itself argues. So "1 seed / 2 legs" and "2 seeds / 2 legs"
produce byte-identical records and a unit test cannot separate them. That is the
*point* of the count-free fix, but the gate is worded as three cases; state that
it is a flow-level assertion (drive the verify with two seeds) or it silently
degrades to two and the middle case — *"the one that kills the filed remedy"* —
stops being exercised.

**XM6 — the marking's behaviour on the QR-ONLY variant is unstated.**
`validateMdmk` (`gui/gui.go:2288-2320`) offers three variants: `TEXT + QR`,
`TEXT ONLY`, `QR ONLY`, and the operator chooses in `bundleEngrave`'s
`ChoiceScreen`. At 67–111 characters all three fit, so `QR ONLY` — a plate whose
`Paragraph` has **no `Text` at all** — is reachable and selectable. Whether it
carries `PASSWORD REQUIRED` is not specified. GATE 1.2 already pins the offered
variant set with the title set; extend it to assert the marking **renders** in
each offered variant.

**XM7 — which `b.MD1` the policy stub comes from.** `templateizeBundle`
(`gui/template_engrave.go:24-38`) replaces the bundle and re-mints the stub via
`md.FormAwareStubChunks(tmplMD1)`; `gui/singlesig.go:139` assigns `b = tb`. The
policy id on the passphrase plate must therefore be computed from the **final**
`b.MD1`, at the offer site, not captured near `:107`. GATE 2.4b's value-equality
would catch a mistake, so this is Minor — but one clause in §2.4 removes the
guess.

**XN1 — a doc comment goes stale on landing.** `multisigVerifyNoSlotBody`'s
header (`gui/multisig_verify.go:150-156`) describes the three arms including the
one R-M rewrites. Per this project's own "comments outlive their conditions"
rule, update it in the same commit.

**XN2 — the preloaded footer's subject sits in the other band.** `POLICY xxxx
xxxx  DERIVED, NOT TYPED` lands in `bottomLines`; the fingerprints it describes
are in `topLines` (`backup/passphrase.go:176-180`), at the opposite edge of an
85 mm plate. The natural reading of the isolated line is that the *policy* was
derived. R-H is an operator ruling and is **not** re-opened here — recorded only
so it is a choice rather than a discovery.

---

## 4. Load-bearing assumptions checked and found SOUND

The next reader need not re-check these.

1. **A new `syswSource` value raises only `flagSource`.** `syswFlags`
   (`gui/sysw_admit.go:91-109`) gates `flagSecretInPlaintext` and
   `flagWeakPassphrase` on `src == srcPayload` and `flagNFCNoIntegrity` on
   `src == srcNFC`; only `if src != srcTyped` catches a new value. No spurious
   warning, no wrong provenance text beyond §2.2's own `default:` trap. Also
   `admits` (`gui/sysw_admit.go:47`) is keyed on program × class, **not** source
   — no admission table needs editing.
2. **§3.1's `:854`-is-unreachable argument.** `verifyFreshSlots`
   (`gui/multisig_verify.go:324-336`) has exactly one error return —
   `errVerifyNoExpectedSlots` on `len(expected) == 0` — `expected` is not
   reassigned inside it, and `gui/multisig_verify.go:715-717` returns before
   `:851` can run. The source-assertion gate is the right instrument.
3. **§1.2's one-field `kind` fix.** `bundlePlate` (`gui/bundle_flow.go:346-353`)
   has exactly six fields and no `kind`; `bundleCard` (`gui/bundle.go:33-38`)
   carries `kind bundleCardKind`; `bundlePlatePlan` (`:358-373`) builds each
   `bundlePlate` from `c`, so `c.kind` is in hand. `singleSigEngraveCards`
   (`gui/singlesig_engrave.go:20-28`) does prepend `cardMS1` when `full`.
4. **§1.1's shared-helper claim.** `WrapText` (`backup/wrap.go:31`), `qrPlaceAt`
   (`:196`) and `textLayout` (`:224`) are used by **both** `EngraveText`
   (`backup/backup.go:375,384,414`) and the `Fitted` path (`backup/fit.go:231,
   240,392,739`; `backup/freetext.go:121,153,200`).
5. **§1.3's flow census is complete.** Exactly four production `bundleEngrave`
   callers and exactly four production `validateMdmk` callers; the spec's table
   names all of them and no fifth exists.
6. **§2.3e's scope table.** Re-verified at the insertion point
   (`gui/singlesig.go:188-192` … `:221-223`): `passphrase` live, `masterFP` bound
   at `:107` and read at `:221`, mnemonic scrubbed only by the `defer` registered
   at `:50-54` which fires at **return**.
7. **§3.2a's body.** Extracted from both documents and compared: **byte-identical
   after whitespace normalisation**, **251 characters**, **no em dash**. The arm
   it replaces reads exactly as quoted (`gui/multisig_verify.go:157-162`).
8. **Every string length in §1.2a and §2.3 recomputes exactly.** 17 / 18 / 18 /
   30 / 27 / 36 / 41 / 32, measured. The `42` (band) and `25` (`Text`
   title/footer) budgets are never crossed between mechanisms in the spec's own
   text — the near-miss the brief warns about did not recur.
9. **§1.2a's method caveat errs in the safe direction.** Raw width admits 16 at
   the 6.0 mm rung where the layout-based cap is 18, so the raw method
   **under**-reports; 25 at 3.8 mm is therefore conservative rather than
   optimistic.
10. **§2.4's C2 correction.** `gui/singlesig_derive.go:68` calls
    `md.FormAwareStubChunks` (`md/template_id.go:122`); the stale doc comment
    naming `WalletPolicyIDStubChunks` is at `gui/singlesig_derive.go:28`, exactly
    as §7 records.
11. **§2.5.** `backup/passphrase.go:86` is `qr.Encode(plate.Passphrase, qr.L)`
    inside `passphraseQRCode` — the passphrase and nothing else.
12. **§3.3's premises.** `verifyStatusMS1Clause` (`gui/verify_status.go:155`) is
    appended on `p.full` alone (`buildVerifyPassLine`, `:211-231`);
    `multisigVerifyOKMessage` (`:1134-1135`) does carry the count-free shape.
13. **R-G's golden inventory.** `backup/testdata` holds **16** `.bin` goldens,
    including `text-{0,1,2}-shards-1.bin` and four `passphrase-*.bin` — "the
    frozen sixteen" and "the four" are both accurate.

---

## Verdict

`RED 3C/7I`
