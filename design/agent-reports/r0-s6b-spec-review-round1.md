# R0 architect review — `SPEC_s6b_pre_flash_cycle.md`, round 1

**Artifact:** `design/SPEC_s6b_pre_flash_cycle.md` (DRAFT, ungated)
**Source under review:** fork `bg002h/seedhammer`, `main` = `b1479a1b38f6b045d27443764c858906e4e6e122`, clean (`git rev-parse` run).
**Lens:** will implementing this spec *exactly as written* produce a device that states something FALSE, omits something required, or ships a gate that cannot catch the defect it names? Nothing else.
**Date:** 2026-08-17.

**Verification basis.** Every Critical and every Important below was verified by reading the cited source in the fork at `b1479a1`. Findings resting on reasoning rather than execution are marked inline. No re-measurement of the facts declared settled in the brief (25-char `Text` budget, 42×2 band, 271/262/240 variant limits, panel/nav/arrow geometry, `verifyRefused`'s four sites, `passRecord.legs` semantics) was attempted or is implied.

---

## Findings

| id | sev | section | one line |
| --- | --- | --- | --- |
| **C1** | Critical | §2.3 | `"…DERIVED, NOT TYPED"` is false: the preloaded path preloads the *passphrase only*; the fingerprints are still hand-typed |
| **C2** | Critical | §2.4 | `md.WalletPolicyIDStub` is **not** form-aware; on a template engrave the plate would carry a policy id matching no card this run cut |
| **C3** | Critical | §1.3 / §1.1.3 | the marking is conditioned "at the call site", and neither named call site has the passphrase or the seed in scope |
| **I1** | Important | §1.3 | `bundle_flow.go:407` is **one** call site serving **four** flows, two of them multisig — marking there crosses R-B, and GATE 1.3 cannot see it |
| **I2** | Important | §1.2 | the Title condition drops the seed predicate, contradicting **R-A** on the reachable watch-only + passphrase path |
| **I3** | Important | §1.2 / §1.3 | a per-set marking at `bundle_flow.go:407` also marks the **ms1 secret plate**, which no directive covers |
| **I4** | Important | §4 / §5 | §5's mandatory background chip occludes body text, and §4's sweep reads the op tree, so it is structurally blind to it |
| **I5** | Important | §5.1 | the visibility predicate is required, never defined, and not listed in §7; §5.1 and §6 name two different GATE 5.1 tests |
| **M1** | Minor | §3.1 / GATE 3.1 | the `:854` arm of GATE 3.1 is unreachable by the spec's own argument, so no behavioural test can exist for it |
| **M2** | Minor | §2.4 / GATE 2.4a | gates behaviour the spec never specifies, for a case the device cannot produce |
| **M3** | Minor | §2.4 | `gui/singlesig_derive.go:28`'s doc comment is stale and is the likely origin of C2 |
| **M4** | Minor | §3.2 | the multisig sibling §3.2 cites has **three** arms; the spec specifies two, dropping the one that says "Your plates are fine" |

Also recorded, not a finding: **the spike's open item is closed** — see "Closed blind spot" at the end.

---

## C1 — Critical — §2.3: `"POLICY … DERIVED, NOT TYPED"` is an affirmative falsehood

**The defect.** §2.3 makes normative a new passphrase-plate footer,
`"POLICY <8 hex, grouped>  DERIVED, NOT TYPED"`, on the R-C preloaded path, on
the stated ground that *"the device **derived** the fingerprints"*. **It did
not.** The preloaded path preloads the **passphrase bytes only**. The two
fingerprints are still typed by the operator, through the same keyboard steps as
the standalone path.

**Verified against source.** `engravePassphraseFlowFrom(ctx, th, body []byte, src syswSource)`
(`gui/passphrase_flow.go:617`) copies `body` into `secret` (`:627`) — that is the
passphrase and nothing else. It then runs the unchanged step machine:

- `gui/passphrase_flow.go:665-671` — `ppStepSeedFP` → `fingerprintEntryFlow(ctx, th, ppSeedFP, seedFP, loadProof)`;
- `gui/passphrase_flow.go:672-678` — `ppStepCombinedFP` → `fingerprintEntryFlow(...)`.

The only other writer of those two variables is `ppPassProofLoader`
(`gui/passphrase_flow.go:655`, defined `gui/passphrase_passproof.go:234`), which
is the **test pattern** — it loads the fixed literals `ppPassProofSeedFP = "DEADBEEF"`
and `ppPassProofCombFP = "CAFEBABE"` (`gui/passphrase_passproof.go:90-91`).
Nothing derives a fingerprint on this path.

The code states it outright at `gui/passphrase_flow.go:628-629`:

> `// The payload PRE-FILLS the passphrase; the operator still walks the`
> `// fingerprint fields and the confirm screen.`

**The reachable case.** Every run of the R-C preloaded program. There is no arm
in which the device derives the plate's fingerprints today.

**Why it is wrong.** The plate would tell a reader, permanently, that this device
derived and therefore vouches for two fingerprints that a human typed and nothing
checked. That is an over-claim on an artifact read years later — the F-198 class
this cycle exists to close — and it is exactly what **R-D** forbids. It is
strictly worse than the string it replaces: `"FINGERPRINTS TYPED, NOT VERIFIED"`
is *true* on the preloaded path as the code stands.

**A second, quieter half of the same defect.** The footer is not unconditional.
`backup/passphrase.go:185-187` appends it only when
`plate.SeedFP != "" || plate.CombinedFP != ""`. So an implementation that
"solves" the falsehood by leaving the fingerprint fields empty on the preloaded
path **silently drops the policy id too** — R-H's whole deliverable disappears
with no error, on the path R-H was written for.

**Smallest fix (preferred).** Preload the fingerprints as well as the passphrase,
and skip `ppStepSeedFP`/`ppStepCombinedFP` on that path. The caller already holds
`masterFP` from `deriveSingleSigBundle` (`gui/singlesig.go:107`); the bare-seed
value is one more call to the same derivation with an empty passphrase. Only then
does `DERIVED, NOT TYPED` become true, and the plate gets *stronger* fingerprints
as a side effect.

**Smaller fix, if the fingerprint steps stay.** Keep the truth and shorten the
claim. `"POLICY 1A2B 3C4D  FPS TYPED, NOT VERIFIED"` is **41** characters against
the measured 42-character band — it fits, with one to spare. One character of
headroom is thin, so `"POLICY 1A2B 3C4D  FPS TYPED"` (27) is the safe form. Either
way, the spec must not assert a derivation that does not occur.

---

## C2 — Critical — §2.4 names the wrong stub function; the template form gets a false binding

**The defect.** §2.4 is normative:

> **"wallet policy id"** = `md.WalletPolicyIDStub` — top 4 bytes → 8 hex, grouped `XXXX XXXX`.

`md.WalletPolicyIDStub` (`md/walletpolicyid.go:106`) is **not form-aware**. It
always roots on `WalletPolicyId`. The form-aware function is `md.FormAwareStub`
(`md/template_id.go:112`):

```
func FormAwareStub(d *descriptor) ([4]byte, error) {
	if isWalletPolicy(d) {
		return WalletPolicyIDStub(d)
	}
	return WalletDescriptorTemplateIdStub(d)
}
```

and its doc comment says why it exists (`md/template_id.go:106-111`): *"Routed
through every stub-mint + verify site so a template binds (and the device's own
readback verifies)."*

**That routing is complete, and measured.** Every production stub site in the
fork calls the form-aware form, not the one the spec names:

| site | call |
| --- | --- |
| `gui/singlesig_derive.go:68` | `md.FormAwareStubChunks(md1)` |
| `gui/multisig_derive.go:43` | `md.FormAwareStubChunks(suppliedMd1)` |
| `gui/multisig_build.go:673` | `md.FormAwareStubChunks(tmplMd1)` |
| `gui/template_engrave.go:29` | `md.FormAwareStubChunks(tmplMD1)` |
| `md/encode_multisig.go:159` | `FormAwareStub(d)` |
| `bundle/verify.go:118` | `md.FormAwareStubChunks(b.MD1)` |

**The reachable case.** `singleSigEngraveFlow` offers **"Template-only md1"**
(`gui/singlesig.go:118-139`); choosing it sets `template = true` and replaces the
bundle via `templateizeBundle`. The mk1 this run cuts then carries a
`policy_id_stub` derived from `WalletDescriptorTemplateId`. A passphrase plate
built per §2.4 would engrave the top 4 bytes of `WalletPolicyId` instead — **a
different four bytes**, tying the passphrase plate to an identifier that appears
on none of the plates beside it.

**Why it is wrong.** The policy id's only job is to bind the passphrase plate to
the wallet it belongs to. On the template form, §2.4 as written engraves a value
that matches nothing this run cut, and nothing on the device or the host will
reconcile it. Under **R-D** the plate asserts a binding that does not exist. Its
severity is the R4 severity: an operator reconciling steel years later finds a
mismatch and cannot tell whether they have the wrong plate or the wrong tool.

**GATE 2.4b does not catch it.** GATE 2.4b is *"the policy-id **label** is true of
the template form too"* — a wording gate. It would pass with the wrong **value**
underneath it. This is a gate that names one half of a two-part defect and
certifies the artifact anyway.

**Smallest fix.** Two edits, both one line:

1. §2.4 names `md.FormAwareStubChunks` (the caller holds the md1 **chunk
   strings**, so the `Chunks` form is the one that applies) instead of
   `md.WalletPolicyIDStub`.
2. GATE 2.4b asserts the **engraved value equals the `policy_id_stub` carried by
   the mk1 this run cut**, on both forms — a value equality, not a label check.
   That gate subsumes the label question and cannot pass on the template form
   while C2 is live.

---

## C3 — Critical — §1.3 conditions the marking at call sites that cannot evaluate the condition

**The defect.** §1.1.3 and §1.3 together are the whole enforcement mechanism:
*"`validateMdmk` learns nothing about flows. The **caller** decides whether a
plate is marked, by supplying a string or not."* §1.3's table then names the four
`validateMdmk` call sites and marks two of them *"per §1.2"*.

§1.2's conditions are **"iff the set was derived with a BIP-39 passphrase"** and
**"iff the set contains a seed"**. Neither named call site has either fact in
scope.

**Verified against source.**

- `gui/gui.go:2344` is inside `mdmkFlow(ctx *Context, th *Colors, s mdmkText)`
  (`gui/gui.go:2342`). Its whole input is a **string**. It is reached from
  `engraveObjectFlow`'s `case mdmkText:` (`gui/gui.go:2261-2262`) — a scanned or
  typed md1/mk1. There is no seed, no passphrase, and no provenance anywhere in
  that call chain. Whether an md1/mk1 was passphrase-derived is **not decidable
  from the string**; that is precisely §2.1's measured point.
- `gui/bundle_flow.go:407` is inside
  `bundleEngrave(ctx, th, title string, cards []bundleCard)`
  (`gui/bundle_flow.go:404`). `bundleCard` is
  `{kind, label, strings, summary}` (`gui/bundle.go:33-38`) — no passphrase, no
  seed material, no derivation record.

The **only** place both facts exist is `singleSigEngraveFlow`
(`gui/singlesig.go`), where `passphrase` (`:97`, `:107`) and `full` (`:103`) are
in scope at the `bundleEngrave(ctx, th, "Engrave Single-Sig", cards)` call
(`:177`). **§1.3's table does not list it**, because it is not a `validateMdmk`
call site — it is a caller two frames above one.

**The reachable case.** All of them. Implemented literally, both conditions
evaluate to "unknown" at both marked sites, an implementer resolves that to
false, and **R3 and R4 never fire on any path** — the cycle's central deliverable
ships as a no-op that every gate in §6 passes. The alternative is worse: an
implementer invents a data path the spec does not authorise and has no guidance
for gating (see I1).

**Why it is wrong.** This is lens 4 exactly: a specified output owning no
mechanism. It is Critical rather than Important because the failure is silent in
both directions — a no-op marking passes GATE 1.1 (goldens do not move), GATE 1.2
(variant set unchanged) and GATE 1.3 (deriveXpubFlow unmarked) with a full green.

**Smallest fix.** Move the conditioning **one frame up**, where the facts are, and
say so: `bundleEngrave` grows two optional strings (`title`, `footer`) passed
through to `validateMdmk`; `gui/singlesig.go:177` is the **only** caller that
passes non-empty values, computed from `passphrase != ""` and `full`. The other
three `bundleEngrave` callers and `mdmkFlow` pass nothing and are byte-unchanged.
That keeps §1.1.2's optionality argument intact, keeps R-A/R-B structural, and
does not teach `validateMdmk` about flows.

---

## I1 — Important — `bundle_flow.go:407` is one call site serving four flows, two of them multisig

**The defect.** §1.3's table reads `gui/bundle_flow.go:407 | bundle engrave |
per §1.2`, as though that site belonged to one flow. It does not.
`bundleEngrave` has **four** production callers:

| caller | title | flow |
| --- | --- | --- |
| `gui/singlesig.go:177` | `"Engrave Single-Sig"` | single-sig — the one S6b means |
| `gui/multisig.go:291` | `"Engrave Multisig"` | **multisig** |
| `gui/multisig_build.go:402` | `"Build Policy"` | **multisig** |
| `gui/bundle_flow.go:39` | `"Engrave Bundle"` | scanned bundle, no derivation |

(Measured: `grep -rn "bundleEngrave(" --include='*.go'` over the fork, excluding
tests.)

**Why it is wrong.** Marking "per §1.2" at `bundle_flow.go:407` marks **multisig
plates**, crossing the **R-B** boundary that §1.3 exists to make *"structural
rather than a promise"*. §1.3 identifies exactly one leak (`derive_xpub.go:494`)
and misses the larger one directly beside it.

**The gate cannot catch it.** GATE 1.3 is *"a test asserting `deriveXpubFlow`'s
plates are **unmarked**"*. That test is green whether or not `bundleEngrave`
marks every multisig plate the device cuts. A gate scoped to one of two leaks
certifies the flow.

**Smallest fix.** C3's fix closes this by construction — if only
`gui/singlesig.go:177` passes a non-empty title/footer, the multisig callers
cannot be marked. Then widen GATE 1.3 to *"only the single-sig engrave marks:
`deriveXpubFlow`, `Engrave Multisig`, `Build Policy`, `Engrave Bundle` and
`mdmkFlow` all produce unmarked plates"*.

---

## I2 — Important — §1.2's Title condition contradicts R-A

**The defect.** §1.2's table:

| slot | when |
| --- | --- |
| **Title** | iff the set was derived **with** a BIP-39 passphrase |
| **Footer** | iff the set contains a seed (R-A) |

The seed predicate is attached to the **Footer row only**. R-A states the
opposite scope: *"watch-only sets are NOT marked … **So the marking predicate is
'the set contains a seed.'**"*

**The reachable case.** `singleSigEngraveFlow` with a passphrase and
`"Watch-only (keys)"` chosen — `gui/singlesig.go:97-103`,
`Choices: []string{buildFullModeLabel(passphrase != ""), "Watch-only (keys)"}`,
`full := modeSel == 0`. A passphrase-derived watch-only engrave is an ordinary,
supported run.

**Why it is wrong.** Under §1.2 read literally that set gets `PASSWORD REQUIRED`
and no footer. R-A says it gets neither. This is not a truth defect — the title
would be true — it is the spec contradicting the ruling it cites in the row
below, in the direction that quietly re-opens a decision the operator closed.

**Smallest fix.** One clause: the Title row's `when` becomes *"iff the set
contains a seed **and** was derived with a BIP-39 passphrase"*. Both rows then
read off R-A's single predicate.

---

## I3 — Important — the per-set marking also marks the **ms1 secret plate**

**The defect.** §1.2 conditions on properties of *"the set"*, and §1.3 locates the
decision at `validateMdmk` call sites. In full single-sig mode the set includes
an **ms1** card, and its string goes through the same `validateMdmk`.

**Verified against source.** `singleSigEngraveCards(b, full)`
(`gui/singlesig_engrave.go:20-28`) prepends `bundleCard{kind: cardMS1, …}` when
`full`; `gui/singlesig_engrave.go:15` states *"its codex32 string engraves as one
plate via validateMdmk, format-agnostic"*. `bundlePlatePlan`
(`gui/bundle_flow.go:358-373`) flattens **every** card's strings into the plan,
and `bundleEngrave` calls `validateMdmk(ctx.Platform, p.str)` on each
(`:406-407`). Nothing distinguishes the ms1 plate at that point.

**Why it is wrong.** R3 names *"mk1 and md1 plates"*; R4 names the same two.
Nothing in §1 or in the rulings decides anything about the ms1 plate, and a
per-set condition marks it by default. The consequence is a change to a **secret**
artifact that nobody chose: the ms1 plate would gain a master-fingerprint line
that ties the secret share to a specific wallet, on a plate whose entire design
posture (`gui/singlesig_engrave.go:17-19`) is that it never leaves owner-held
steel. §2.1's own measurement is that `ms1` is passphrase-**independent**, so the
combined fingerprint is also the one line on that plate that does not describe
what the plate encodes.

**Confidence.** High on reachability and on the mechanism (both read from
source). The *severity* of adding a fingerprint to a secret plate is a judgement I
am flagging rather than asserting — but the spec deciding it by silence is the
defect either way.

**Smallest fix.** One sentence in §1.2: the marking applies to `cardMK1` and
`cardMD1` plates only; `cardMS1` plates are never marked. Add it to GATE 1.3's
assertion list.

---

## I4 — Important — §5's mandatory chip occludes body text, and §4's sweep cannot see it

**The defect.** §5 NORMATIVE 3: *"**The chip is not optional** — the arrows sit
where body text currently draws, because `fadeClip` clips nothing (R-E)."* So the
spec makes an opaque background chip mandatory, at the top-centre and
bottom-centre of the body, over glyphs that are being drawn.

§4 is the spec's guarantee that modal bodies are readable in full, delivered by
the F-185 class check. **That check is structurally blind to occlusion.** It
compares the drawn frame's *op tree* against the source string:

- `bodyDrawnFully(drawn, body string)` — `gui/modal_fits_test.go:81-100` —
  `strings.Contains(normalizeDrawn(drawn), normalizeDrawn(body))` plus a binary
  search for the cut point;
- and the file says why that is the seam, at `gui/modal_fits_test.go:22-27`:
  *"ExtractText walks the op tree, so a body the panel renders as nothing still
  'appears' to uiContains."*

A glyph drawn **under** an opaque chip is still in the op tree. §4's sweep returns
green.

**The reachable case.** Every modal §4 sweeps, once §5 lands — including
`ConfirmWarningScreen`, which R-I itself describes as *"a screen whose purpose is
to stop a funds-losing mistake"*.

**Why it is wrong.** R-I's decoupling of F-192 from F-208 rests on *"Body width is
unchanged — 417 before, 417 after"* — an argument about **wrap**. Occlusion is a
different axis and the argument does not reach it. §5.2 carries the width
invariant forward as normative and says nothing about the chip covering text. The
result is F-185's own defect — a line of a safety modal the operator cannot read
and is not told exists — re-introduced under a gate that certifies the opposite.

**Smallest fix.** Two lines. (a) §5 states the chip's drawn bounds and requires
the body's first and last **drawn text rows** to clear them — the body already
starts at `bodyClip.Min.Y + scrollFadeDist` (`gui/gui.go:416`), i.e. inside the
16 px zone the arrows occupy, so this is a real constraint, not a formality.
(b) §4's GATE adds one pixel-level assertion on one representative long modal
with an arrow showing: the top and bottom drawn rows are not overlapped by a
chip. Nothing more general is needed — one screen proves the geometry.

---

## I5 — Important — §5.1 requires a predicate it never defines, and GATE 5.1 names two different tests

**The defect, part 1.** §5.1 is normative — *"the predicate is defined against
**actual visibility**"* — and then never defines it. It establishes what the
predicate is **not** (`maxScroll > 0`, `gui/gui.go:409`, verified: `maxScroll =
bodysz.Y - (bodyClip.Dy() - 2*scrollFadeDist)` with `scrollFadeDist = 16`,
`gui/gui.go:761`), states that the body is not clipped to `bodyClip` at all, and
stops. §7 "WHAT THIS SPEC DOES NOT SETTLE" does **not** list it, so an implementer
reads §5.1 as settled and reaches for the one expression the spec hands them —
`maxScroll > 0` — which §5.1 says is a false statement by the UI in the direction
R-D forbids.

**The defect, part 2.** GATE 5.1 is two different tests in two places:

- §5.1: *"a test that the two agree — `maxScroll > 0` on a screen where nothing is
  actually hidden is a **finding**, not a rounding error"* — a check on
  `maxScroll` vs. reality. By §5.1's own analysis this is **expected to fail**;
  R-E ordered it as a *discovery* pass whose failures are findings to file.
- §6 row 5.1: *"the arrow predicate agrees with actual visibility"* — a check on
  the **new** predicate, which must be green.

One number, two tests, opposite expected outcomes. A red result on the first reads
as "the gate failed, weaken it", which is how a false-PASS gate is born.

**Smallest fix.** (a) §5.1 states the predicate as an expression. The simplest
candidate the measured geometry supports is `bodysz.Y > bodyClip.Dy()` — drop the
`2*scrollFadeDist` reservation that is never drawn as fade — with the F-95
overdraw (body reaches y=317 against `bodyClip.Max.Y = 314`) named as the residual
and the R-E comment attached, as §5.1 already requires. (b) Split the gate: **5.1**
is the new predicate agreeing with actual visibility (must be green); **5.1b** is
R-E's divergence probe on `maxScroll` (findings expected, does not gate).

---

## M1 — Minor — GATE 3.1's `:854` arm cannot be tested

§3.1 argues, correctly, that `:854` is a defensive re-check of `:717`. **Verified:**
`verifyFreshSlots` has exactly one error return, `errVerifyNoExpectedSlots` on
`len(expected) == 0` (`gui/multisig_verify.go:324-327`); `expectedSlots` is the
function parameter and is **never reassigned** in `multisigVerifyFlow` (all
thirteen occurrences read it — `gui/multisig_verify.go:703, 715, 788, 797, 833,
851, 877, 937, 962, 967, 1020, 1032, 1033`); and `:715-717` returns before `:851`
can run. **`:854` is unreachable in-process.**

GATE 3.1 nevertheless demands *"a test asserting the other three do **not** loop"*.
Two of the three (`:717`, `:727`) are reachable by calling `multisigVerifyFlow`
with an empty slice; the third cannot be reached by any input. An implementer
either writes a test that cannot execute the line, or drops it and records the
gate as 3-of-3 covered.

**Smallest fix.** GATE 3.1 becomes: behavioural non-loop assertions for `:717` and
`:727`, plus a **source assertion** that `:854` returns `verifyRefused` — the
idiom `gui/singlesig_truth_test.go` already uses. Say in the gate which arm is
which, so "never executed" is visible rather than assumed.

---

## M2 — Minor — GATE 2.4a gates unspecified behaviour for an unreachable case

§2.4 constraint 1 ends *"**GATE 2.4a:** what the marking does in that case must be
specified and tested"* — and the spec never specifies it. §7 lists §2.4b's label
as unsettled but **not** §2.4a, so the hole reads as closed.

It is also unreachable on the paths this cycle marks. **Verified:** the header's
fingerprint flag is set iff `card.Fingerprint != ""`
(`mk/encode.go:70-80`), and every device-side `mk.Card` sets it —
`gui/singlesig_derive.go:77`, `gui/multisig_derive.go:51`,
`gui/derive_xpub.go:369`, all `Fingerprint: fmt.Sprintf("%08x", masterFP)`. **The
device cannot emit a fingerprint-less mk1.** A fingerprint-less mk1 can only
arrive from another tool, over the scanned paths that C3/I1 say must not be marked
at all.

Nothing false ships. The cost is a gate that will be recorded as covered without
ever running.

**Smallest fix.** Replace GATE 2.4a with the measured fact and a one-line
assertion: the mk1 this run cuts carries an `origin_fingerprint` (cite the three
construction sites); marking is scoped to device-derived sets, so the optional-fp
case is out of scope for S6b. Then say so in §7.

---

## M3 — Minor — the stale comment C2 probably came from

`gui/singlesig_derive.go:28` still documents the pipeline as
`md.WalletPolicyIDStubChunks(md1) → mk.Encode(stub)`, while the code forty lines
below at `gui/singlesig_derive.go:68` calls `md.FormAwareStubChunks(md1)`. This is
a pre-existing fork defect and I am filing it only because **the spec depends on
it**: it is the most likely source of C2's wrong function name, and it will
mislead the implementer again at fold time.

**Smallest fix.** One-word comment correction in the fork, in the same commit that
lands C2's spec fix.

---

## M4 — Minor — §3.2 specifies two arms where the sibling it cites has three

§3.2 quotes `multisigVerifyNoSlotBody` (`gui/multisig_verify.go:157`) and
prescribes a **two**-arm conditional (passphrase entered / not). The cited
function has **three** (`gui/multisig_verify.go:157-170`), and the arm the spec
drops is the strongest one:

```
case provedInnocent:
    return "That seed IS a cosigner of this policy, but not with the passphrase you " +
        "typed: this wallet's keys come from the seed with no passphrase. Your " +
        "plates are fine. Try again and skip the passphrase."
```

On the single-sig path this arm is cheap: the flow already re-derives from a
re-typed seed (`deriveSingleSigBundle`, `gui/singlesig_verify.go:115`), so
re-deriving once with an empty passphrase and comparing against the same readback
tells the device outright whether the plates are fine. Nothing false ships without
it — the two-arm copy is true, just weaker — which is why this is Minor and not
Important. Recording it so the choice is made rather than inherited.

---

## Closed blind spot — the spike's unmeasured bound is now measured

`SPIKE_s6b_q2_results.md` §2 and §4 flag as unmeasured: *"The maximum `md1`/`mk1`
chunk payload — §2's robustness argument rests on it."* It is a hard, code-enforced
constant:

- `ValidMD` (`codex32/mdmk.go:137-143`) rejects a data part longer than
  `mdRegularMaxLen = 93` (`codex32/mdmk.go:49`) → **md1 ≤ 96 characters** total;
- `ValidMK` (`codex32/mdmk.go:152-160`) admits only `[14,93]` and `[96,108]`
  (`mkRegularMinLen`/`mkRegularMaxLen`/`mkLongMinLen`/`mkLongMaxLen`,
  `codex32/mdmk.go:54-57`) → **mk1 ≤ 111 characters** total.

So the spike's *"longest in-repo 111"* is in fact the absolute maximum, and the
title+footer budget of **240** carries better than 2× margin against a **proven**
bound rather than an observed one. The spec should state this and retire the
spike's caveat; GATE 1.2 remains worth having for the variant-set question, but it
is no longer covering an open bound.

---

## Verdict

`RED 3C/5I`
