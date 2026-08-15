# RECON — what S1–S6's walk gates assume, and what the walk actually proves

**Date** 2026-08-14. **Scope** `IMPLEMENTATION_PLAN_multisig_build_repair.md`
§2–§3 against `cmd/emu/walk_trace_a.js` and `gui/` at fork `88d43c7`.
**Question, and only this one:** every stage gate from S1 on says *"by test and
by emulator walk"* — does the walk assert what those gates assume?

**Answer: no, and the gap is structural rather than a matter of degree. The only
walk that exists drives a different program than the one all five stages edit.**

Everything below is measured. The command that produced each fact is given, for
the reason this project keeps re-learning: 16 of 16 *gated* code citations in
the previous cycle were true and 5 of 22 ungated ones were false.

---

## 0. What the walk does, exactly

    $ grep -n 'goTo("' cmd/emu/walk_trace_a.js
    169:  await goTo("LoadPayload");
    180:  await goTo("EngraveBundle");

Two programs, from the nine on the start carousel
(`gui/text_program_test.go:19-29`). Every screen it waits on:

    $ grep -nE 'waitFor\("|match: "' cmd/emu/walk_trace_a.js
    SeedHammer · PayloadDigest · PayloadWarnings · Keepthispayloadloaded? ·
    Firstcardfromwhere? · Scanacard,orDone · Droppedanincompletecard ·
    Cardadded · cardsverified · Chooseengraving ·
    Holdbuttontostarttheengravingprocess · Engravingcompletedsuccessfully

What it concludes:

    $ grep -n "ok:" cmd/emu/walk_trace_a.js
    274:    ok: census.strings.length === plates && census.unattributed === 0,

That is the whole gate: **a plate count and an anomaly counter.**

---

## C1 — the walk never enters the function all five stages edit

`buildMultisigPolicyFlow` has exactly one production caller:

    $ git grep -n "buildMultisigPolicyFlow(" -- 'gui/*.go' | grep -v _test
    gui/multisig.go:55:	buildMultisigPolicyFlow(ctx, th)
    gui/multisig_build.go:39:func buildMultisigPolicyFlow(ctx *Context, th *Colors) {

`gui/multisig.go:55` sits inside `engraveMultisigFlow`, behind the choice
`{"Supply policy (md1)", "Build policy"}` at `gui/multisig.go:45`. And
`engraveMultisigFlow` is dispatched from the program switch at
`gui/gui.go:1822-1823` — the case *next to* the one the walk takes:

    gui/gui.go:1816-1817   case engraveBundle:   bundleFlow(ctx, th)
    gui/gui.go:1822-1823   case engraveMultisig: engraveMultisigFlow(ctx, th)

`goTo("EngraveBundle")` matches `shScreen().startsWith("EngraveBundle")`;
"Engrave Multisig" squashes to `EngraveMultisig` and does not match. So the walk
takes `bundleFlow` — the standalone bundle program — and `buildMultisigPolicyFlow`
is never on its path.

That function is `gui/multisig_build.go:39-193`. Per
`design/agent-reports/plan-wide-file-touch-matrix.md` (`bb45d7d`), **all five of
S1–S5 edit it**; it is the reason the concurrency ceiling is 1.

**So S1, S2, S3, S4 and S5 each close on "by emulator walk", and the only walk
that exists cannot execute the flow any of their gates name.** Not "asserts too
little" — does not reach.

**That sentence read "cannot execute one line any of them changes" until the
review (I-3), and the absolute was false.** S2 edits
`layoutTitle(ctx, dims.X, th.Text, "Engrave Bundle")` at `gui/bundle_flow.go:155`
— **inside the shared gatherer**, which the existing bundle walk does render and
wait on. That is S2's riskiest edit by the plan's own account: one shared file,
five call sites, four flows that have nothing to do with multisig build. So the
existing walk is **the only automated coverage D-4's blast radius has**, and
writing it off would have discarded it. The conclusion is unchanged — each
gate's *subject* is unreachable — but the S0 walk is now a regression check for
S2 rather than nothing.

## C2 — S0's own evidence line calls this walk Trace A, and it is not Trace A

The plan defines Trace A at §2:173-177 as

    boot → SKIP/LOAD payload → digest compare → Engrave Multisig → Build policy
      → template(wsh) → n=2..5 → k → self-slot → fp
      → cosigner review → seed entry → policy review → form → EXPERIMENTAL
      → mode → engrave → restore doc

The walk is: boot → LOAD payload → digest → **Engrave Bundle** → gather three
cards → engrave six plates. **Zero** of the eleven build-flow screens appear.
The file is named `walk_trace_a.js`, its header opens "Trace A as a script", and
S0's gate row 3 records it as "a six-plate Trace A run in ~165 s"
(plan:355, and row 3 of the gate table at plan:405).

This is F-164's defect one level up: not a stale *identifier*, a stale *claim
about which journey ran*. Anyone verifying S1 by "the Trace A walk is green"
verifies a bundle engrave.

**S0 D3 itself is not in question.** The shapes it delivered — `shTap`,
`shPress`/`shRelease`, `shPace`, `shSysw`, `shScreen`, `shNFC.present`,
`shToolpath.strings()` — are sufficient to drive the build flow (see §5). What is
wrong is the label on the walk, and the inference every later gate draws from it.

## C3 — the walk asserts a count where §4.5 requires a byte comparison

`CARDS` (walk_trace_a.js:66-76) holds the exact six strings the walk expects.
It is used once, to *present* chunks at line 198 (`for (const [name, ...chunks]
of CARDS)`). It is never compared to the census. `ok` is
`strings.length === plates`.

A walk that engraved six *wrong* strings is green today. The header
acknowledges this and defers it to a human: "compare run()'s census against
`go run ./cmd/buildpayloadcards`".

The plan's §3 preamble (:193-201) is explicit that this is not enough — *"A
walk's expected artifact census MUST derive from the recorded input tuple, never
from what the walk produced… The script computes how many md1 chunks, mk1s and
ms1s the inputs REQUIRE and fails when fewer arrive."* Measured: no such
computation exists. `plates` is a function parameter defaulting to `6`
(walk_trace_a.js:151).

## C4 — no byte-comparison harness against the primary toolchain exists at all

S2's gate: *"the current primary BUILDS an md1 from the same inputs and the
strings are equal"*. S5's: *"each engraved ms1 must equal `ms encode --hex <that
master's entropy>`"*.

    $ git grep -nE 'exec\.Command\("(md|mk|ms)"|cargo/bin/(md|mk|ms)' -- '*.go'
    (no production hit)

`oracle` resolves the three binaries to source commits and stops; nothing
invokes them. Both gates are **unimplemented**, not merely unrun. The two `gui`
tests that read `md/testdata/vectors` (`bundle_testdata_test.go:43`,
`md1_gather_test.go:30`) are fixtures, and `oracle.CheckDataSource` refuses
`testdata` as a comparison source by design — correctly, since agreeing with
vendored vectors is agreeing with a subset of ourselves.

## I1 — every screen needle the walk uses is ambiguous across flows

    $ git grep -n "First card from where" -- 'gui/*.go'
    gui/bundle_flow.go:25       (Engrave Bundle)
    gui/multisig.go:76          (Supply policy)
    gui/multisig_build.go:54    (Build policy)   <- the one S1 edits

Three production flows, one string. `"Choose engraving"` has six sites
(`gui/gui.go:2323`, `gui/gui.go:2830`, `gui/bundle_flow.go:307`,
`gui/derive_xpub.go:461`, `gui/unlock_platelist.go:227`, and
`gui/unlock_plates_test.go` waits on it too). And the gather title is
`layoutTitle(…, "Engrave Bundle")` at `gui/bundle_flow.go:155` — **inside the
shared gatherer**, so the gather screen says "Engrave Bundle" even when reached
from Build policy. That is exactly S2's D-4.

Consequence, and it is the trap: a stage walk written by editing the `goTo`
target of this script would produce *identical* assertions and no needle would
notice it was in the wrong program. The only program identification in the whole
script is the carousel match at entry. **Every per-stage walk must assert a
screen that exists in one flow only.**

**Three such needles exist TODAY — corrected after review (I-4).** This section
said the gather title becomes the discriminator only after S2 fixes D-4, which
would have been an argument for deferring the scaffolding past S2. Measured with
`git grep -F … -- 'gui/*.go' | grep -v _test`, each a single production site:

    gui/multisig_build.go:300   Lead: "Choose policy type"
    gui/multisig_build.go:376   Lead: "How many keys (n)?"
    gui/multisig_build.go:394   Lead: "Which slot is your key?"

plus `gui/multisig.go:44` `Lead: "Supply or build a policy?"`, unique to
`engraveMultisigFlow`. **And a decoy this report missed:** `Title: "Engrave
wallet policy"` / `Lead: "Which md1?"` is **two** sites —
`gui/multisig_build.go:121` and `gui/singlesig.go:94` — so a stage author
reaching for the obvious form screen reaches for a shared one.

## I2 — S3's restore-doc gate is readable, but only on one of two branches

`multisigRestoreDocFlow` renders to a screen (`gui/multisig_restore.go:54-58`),
so `shScreen()` can read `P2SH-P2WSH`. But `gui/multisig_build.go:185` skips the
restore doc entirely when the operator picked the **template-only** form at
`:120-142`. S3's gate — *"showing `P2SH-P2WSH` on the restore doc"* — is
therefore satisfiable only if that stage's walk picks **"Full policy md1"**, and
nothing says so. A walk that took the other branch would reach the end with the
gate's subject never drawn, which reads as "the screen did not say it".

## I3 — a per-stage walk needs a per-stage plate count, and nobody has one

Trace A on the build path is 2-of-3 with the operator holding one key, so the
engrave is md1 chunks + the self mk1 + (in full mode) an ms1 — not the six mk1
chunks this walk cuts. Trace B is 6–9 plates (plan:757). Each stage's `plates`
and expected census must be **derived from the input tuple** (§3 preamble), and
today `plates` is a literal default. This is the same defect as C3, restated at
the place it has to be fixed.

## I4 — the census is cumulative, so a second walk in one page session is poisoned

`engravedRecorder.Strings()` is cumulative for the session and deliberately has
no reset (`cmd/emu/engraved.go`, and its comment explains why hanging one on
`shToolpath.reset()` would be a trap). Two walks on one page load accumulate.

Currently this fails closed twice over, by luck rather than design:
`strings.length === plates` is an equality, and `oracle.ParseWalk` requires
`len(digests) == len(strings)` while `digests` is per-run. Both refuse the
second walk. **Record it as a standing constraint — one walk per page load — so
a future `>=` does not quietly convert it into a fail-open.**

## N1 — S4 and S6 are correctly gated on work that does not exist yet

S4's `both` slot is S4's own new model, so no walk can exist before its
implementation; that is a gate following its stage, not a defect. The payload is
already ready for it — measured against the blob, not read off the inventory
comment: card `A@0` is master A at account 0
(`cmd/buildpayloadcards/main.go:34-38,53-58`), and the payload holds **exactly
one** `ClassMnemonic` record whose body is byte-equal to master A's words. That
is the honest `both` case S0 D2 promised, and it is there. S6 is hardware and
names no walk.

---

## 5. What is NOT wrong — so the fix is not scoped wider than it needs to be

The harness shapes are sufficient. A real Trace A walk needs, in order:

| screen | how the harness reaches it |
| --- | --- |
| template picker (`multisigTemplatePick`) | `shTap` on a ChoiceScreen |
| n / k / self-slot / fp (`multisig_build.go` `buildParamPickFlow`) | four ChoiceScreens |
| "First card from where?" + gather | `shSysw` — and **zero `shNFC.present`**, see below |
| **seed entry** | **`FROM PAYLOAD` is choice 0** — see below |
| passphrase, review, form, EXPERIMENTAL, mode | ChoiceScreens |
| engrave | the existing cut loop |
| restore doc | `shScreen()` |

The seed step is the one that looked expensive and is not. The build flow's
`(3) TYPED-ONLY self seed` comment at `gui/multisig_build.go:67` is **one of the
nine stale `TYPED-ONLY` comments S3 deletes** — it calls `seedEntryFlow`
(`gui/derive_xpub.go:88`), which opens `syswSeedPicker`, which puts
`FROM PAYLOAD` **first** whenever the payload carries a `ClassMnemonic`. Ours
does. So the walk takes one confirm, not twelve words through a keyboard.

Reading that comment instead of the call it guards would have costed a
seed-typing harness nobody needs. It is the third time this cycle a doc comment
described a retired mechanism.

**The gather row said "`shSysw` + `shNFC.present`, as today" until the review
caught it (I-1), and that was a harness substitution that would have made S1's
gate pass without S1's feature.** S1 delivers *"the payload supplies the whole
cosigner set"*, and its own test 3 says **zero scans**. A build walk that
completes its gather by presenting chunks over the emulated reader is green
whether or not `takeAll` exists — and phase-1 hardware has no reader at all, so
the affordance exists only in the harness. **A stage-gate build walk must assert
`shNFC.present` was called ZERO times**, with that assertion itself one of the
seen-to-fail mutations. An NFC-fed build run is a driver smoke test and must be
labelled as one.

---

## 6. What this means for the plan

Two things follow, and they are different sizes.

**Small: the labelling.** `walk_trace_a.js` is a bundle-engrave walk. Rename it,
or state in one line what journey it is, and correct S0's gate row 3 so it does
not certify a Trace A that did not happen. S0's D3 deliverable stands on its own
merits — the shapes work, and the six-plate run proves it.

**Large: five walks and one comparison.** Each of S1–S5 needs its own script
against its own program, with (a) a needle that identifies the flow it is in,
(b) a plate count and census derived from that stage's input tuple, and (c) the
byte comparison §1a asks for, which needs a harness that shells out to the pinned
`md`/`mk`/`ms` — `oracle` already resolves them and nothing calls them.

**The rule this cycle keeps paying for applies here too:** a gate must have
executed at least once and been *seen to fail once*. Of the gates audited above,
the only one that has now done both is S0's gate record (three mutations, red
then green, `88d43c7`). Every "by emulator walk" clause from S1 on is a
hypothesis.
