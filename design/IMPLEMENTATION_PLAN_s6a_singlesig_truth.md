# IMPLEMENTATION PLAN — S6a: the single-sig path says what it does not contain

**Status:** DRAFT — not yet through R0. No code may be written against this until
it closes GREEN at 0 Critical / 0 Important.

**Owning phase:** pre-S6. Every follow-up here is owned by
`SPEC_multisig_build_repair.md` S6 (hardware validation), and S6 flashes firmware
an operator engraves real backups with. An item scheduled to a phase is not
deferrable past it.

**Repos.** All code changes land in the fork,
`/scratch/code/shibboleth/seedhammer` (Go, fork-native GUI). This plan and every
review report live in `mnemonic-engrave/design/`.

---

## 0. The operator's design directive, which this plan is an application of

Verbatim, 2026-08-16:

> **permissive on input, expressive on output, and loudly declare assumptions we
> make to fulfill user requests.**

Read onto this cycle, that is three rules and they resolve every design question
below:

1. **Permissive on input.** A passphrase single-sig engrave is a thing the
   operator may ask for, and this plan does not refuse it, does not force
   watch-only, and does not add a confirmation gate in front of it. Nothing here
   removes a capability.
2. **Expressive on output.** Every screen and every document states what is on
   the plates *and what is not*. The label the operator reads before pressing,
   and the document a stranger reads in five years, both.
3. **Loudly declare assumptions.** Where the device assumes the operator holds a
   factor it never engraved — a BIP-39 passphrase, the seed words behind a
   watch-only set — the artifact says so out loud rather than going silent.

The defect this cycle exists to kill is the opposite of all three: a flow that
takes an input, silently drops a required spending factor, and then vouches for
the result.

---

## 0.1 WHAT THIS CYCLE IS TRYING TO ACHIEVE — and what it is not

**Written at round 9, after the operator asked "maybe the goal of the design is
simply wrong — what are we trying to achieve?" The absence of this section is how
a third, unstated goal crept in and produced every Critical of the cycle.**

### The two goals

**G1 — THE DEVICE NEVER MISDESCRIBES WHAT IT ENGRAVED.**
The mode label names what is and is not on the plates; the document states how
many plates the set is and what it contains; a required spending factor that is
on no plate is named out loud; and an aborted set produces no document at all.
*This is F-198, verified by bytes. Its fix has produced **zero Criticals in nine
rounds**.*

**G2 — THE DEVICE NEVER VOUCHES FOR PLATES IT HAS EVIDENCE AGAINST, AND NEVER
CLAIMS A CHECK IT DID NOT PERFORM.**
The document asserts a verification only when one cleanly happened; otherwise it
says so conservatively. Omission must **weaken** a claim, never strengthen it.
*This is C-1, added by review in round 0.*

### The non-goal that cost this cycle nine rounds

**NG1 — REPORTING THE EPISTEMIC STATUS OF THE VERIFICATION.**
Six knowledge states, per-observation world-sets, a monotonicity property, an
enforcement artifact, a coverage script. **Nobody ever stated this goal.** It was
inferred while fixing G2, and **every Critical since round 0 came from it.**

**The structural reason it was unaffordable, and it generalises:**

> **G2 is a PROHIBITION. NG1 is an OBLIGATION.**

*"Never claim more than you know"* is satisfiable with one conservative default
and requires **no enumeration** — if you do not know, you say less. *"Always say
exactly what you know"* requires a **complete and correct partition of everything
the device can observe.** That is what P4 needed and could not get, and what
P5(b) needed and could not get. **Two properties failed for one reason:
obligations over incomplete knowledge are not dischargeable.**

A prohibition fails safe by construction. An obligation fails open — which is
exactly what the observation table did.

**NG2 — DISTINGUISHING FAILURE MODES ON THE DOCUMENT.** Diagnosis has a reader:
the operator standing at the machine, at verify time. The fork's screens already
do it well, and have already fixed bugs this plan then re-committed **four
times**. The document's reader is a stranger, years later, who cannot act on a
taxonomy.

### The test any line must pass to belong on the document

> Does a stranger holding a pile of steel need it to answer **"is this
> everything?"** or **"can I trust it?"**

If neither, it belongs on a screen or nowhere.

### The lens that was never run

Nine rounds applied adversarial, executability, test-falsifiability,
fold-vs-findings, spec-coverage, comprehension, disclosure, reader-comprehension
and two blind-spot passes. **Every one asks whether the design is CORRECT. None
asked whether it should EXIST.** That lens is this section.

---

## 1. MEASURED FACTS — what a reviewer need not re-derive

Everything in this section was read out of the fork at `main` = `b8a23bf` by the
controller, with the command that produced it. **A reviewer should spend no
budget re-checking these**; spend it on design, on the threat model, and on
whether the tests in §5 can actually fail.

Where this plan contradicts a follow-up's description, the follow-up is wrong and
the fact below is what was measured.

### 1.1 The Critical (F-198) is real, and it is broader than F-198 says

`gui/singlesig.go`:

| line | what is there |
| --- | --- |
| `:64-74` | takes an optional BIP-39 passphrase via `syswPassphraseFlow` |
| `:80` | `Choices: []string{"Full (seed + keys)", "Watch-only (keys)"}` — the raw literal |
| `:90` | passes `passphrase` into `deriveSingleSigBundle(...)` — a live derivation input |
| `:127` | `bundleEngrave(ctx, th, "Engrave Single-Sig", cards)` — **result discarded** |
| `:136` | `restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path)` |

**F-198 understates the restore-document half.** The follow-up says the document
"cannot mention a passphrase". Measured, it is worse: `restoreDocFlow`
(`gui/singlesig_restore.go:119`) takes **no inventory parameter of any kind**. Its
screen (`singleSigRestoreLines`, `gui/singlesig_restore.go:97-113`) renders
exactly four things — master fingerprint, descriptor, first receive address,
first change address. The single-sig restore document therefore states **no plate
count, no completeness claim, and no passphrase fact**. It is not a document with
a missing sentence; it is a document with no inventory at all.

    grep -c passphrase gui/singlesig_restore.go   # → 0

### 1.2 Both multisig paths already do this correctly; single-sig is the only holdout

    grep -rn "buildPlateInventoryLines" --include="*.go" gui/ | grep -v _test

- `gui/multisig_build.go:479` — BUILD path, passes `reg.passphraseFacts()`
- `gui/multisig.go:362` — SUPPLY path, passes `oneSeedPassphraseFact(passphrase != "")`
- single-sig — **passes nothing; does not call it**

Same for the label:

    grep -rn "buildFullModeLabel" --include="*.go" gui/ | grep -v _test
    # gui/multisig_build.go:373, gui/multisig.go:217, and the definition. Not singlesig.go.

`buildFullModeLabel` (`gui/multisig_build_census.go:248-253`) returns
`"Full (seed + keys, NOT passphrase)"` when true and `"Full (seed + keys)"` when
false. The correct string already exists and is simply not called from
`singlesig.go`.

### 1.3 The landmine in the obvious fix

`buildPlateInventoryLines` (`gui/multisig_build_census.go:75-92`) ends with a
seed-handling ruling containing the clause **"this build can hold several"**
(seeds). That is **false on the single-sig path**, which holds exactly one seed
by construction (one `seedEntryFlow` seam, `gui/singlesig.go:41`).

Reusing the shared function verbatim would print a false sentence on a document
read years later by someone who was not the operator. The naive fix introduces a
new defect of the same family as the one it closes.

### 1.4 Single-sig has no PRE-engrave plate census either

    grep -rn "buildPlateCensusLines" --include="*.go" gui/ | grep -v _test
    # gui/multisig_build.go:394 and gui/multisig.go:271 only.

The single-sig operator commits to a 2- or 3-plate cut — minutes per plate — with
no count shown. **This is not a filed follow-up.** The controller found it during
this recon; it is filed as **F-202** and its disposition is §3.

### 1.5 F-197 is a defect AND a false claim in the code that outlives it

`gui/bundle_flow.go:535` states, as justification for where the abort warning's
text lives:

> `// inventory, and both engraving callers now gate it on this function's own`
> `// caller returning bundleEngraveDone -- so an operator whose engrave died really`
> `// does not reach it, and this modal really is the only screen they get.`

Measured, `bundleEngrave` has **four** production call sites:

    grep -rn "bundleEngrave(ctx" --include="*.go" gui/ | grep -v _test

| site | gates on `bundleEngraveDone`? | has a post-engrave tail? |
| --- | --- | --- |
| `gui/multisig.go:291` | **yes** | yes |
| `gui/multisig_build.go:402` | **yes** | yes |
| `gui/singlesig.go:127` | **no** | **yes — verify offer at `:130`, restore doc at `:136`** |
| `gui/bundle_flow.go:39` | no | **no** — it `return`s on the next line, so nothing vouches for the set. Correct as written; needs no gate. |

So three callers carry a tail, two of them gate, and the comment says "both". The
claim **was false when it was written** (S5's I-12 fold): `gui/singlesig.go:127`
already existed, ungated, with a tail. The fold generalised from the two callers
it was looking at.

This matters beyond tidiness. The sentence is exactly the class of assertion a
reviewer inherits as a given — F-196's lesson, in the same codebase, one stage
earlier. Correcting it is part of F-197's fix, not a cleanup.

### 1.6 F-195's sentence already exists in this codebase, on a different screen

`gui/bundle_flow.go:555` — inside `bundleAbortWarningText`, gated on
`bundleSetCarriesASecret(cards)`:

    return msg + "No plate in this set carries a seed."

So the *abort warning* already tells a watch-only operator there is no seed in
the set. The **restore document does not**, and the restore document is the
artifact read years later, alone. F-195 is not a missing concept; it is a true
sentence that exists on the transient screen and not on the durable one.

### 1.7 The test harness that already drives all of this

The tests this plan needs have exact prior art, which the implementer should
mirror rather than invent:

| what | where |
| --- | --- |
| driving `engraveSingleSigFlow` end to end | `gui/singlesig_flow_test.go:51` (`TestEngraveSingleSigFlowFull`), `:91` (watch-only) |
| asserting a label + a restore document over a passphrase run | `gui/multisig_supply_passphrase_test.go:261` |
| the non-vacuity arm (bare run must NOT cry wolf) | `gui/multisig_supply_passphrase_test.go:305` |
| "abort is the last screen of the program" | `gui/multisig_verify_report_test.go:969` |
| delivering a passphrase to a walk via the payload | `s5PassphraseRecord` / `s5Passphrase`, `gui/multisig_supply_passphrase_test.go:41-43` |

`syswPassphraseFlow` is `gui/sysw_source.go:84`. An explicitly-bound **empty**
passphrase is *no* passphrase — the predicate throughout is `passphrase != ""`,
matching `seedPassphraseFact.Uses`
(`gui/multisig_build_census.go:225-229`).

### 1.8 Blast radius of the signature change

`restoreDocFlow` has **exactly one** production call site
(`gui/singlesig.go:136`) and **zero** test call sites:

    grep -rn "restoreDocFlow(" --include="*.go" gui/
    # the definition at gui/singlesig_restore.go:119, and gui/singlesig.go:136. Nothing else.

`multisigRestoreDocFlow` (`gui/multisig_restore.go:100`) takes `extra []string`
and ends `restoreDocScreen(ctx, th, append(lines, extra...))`.

**That is the CURRENT shape, and it is explicitly NOT the shape to mirror** — an
earlier draft of this line said it was. §4.7b measures that a trailing parameter
cannot reach slice index 0, which is what "page 1" means on this pager, so
**both** functions gain a leading `status []string` as well (§4.2, §4.7b). This
sentence is kept as a *measured fact about today's code* and is flagged so no
reader takes it as the target design.

### 1.9 No test pins the literal single-sig label

    grep -rn '"Full (seed + keys)"' --include="*.go" gui/

Every hit is either the multisig census/label code, a comment, or a multisig
test. `TestEngraveSingleSigFlowFull` selects the row by **index** (`Button3` =
choice 0), not by text, so changing the label breaks no existing test. **This is
also why the label change needs a new test of its own: nothing currently fails
if it regresses.**

---

## 2. The Rust-primary check — run, negative, settled

The standing rule: whenever a defect is found in a Go port, check whether the
same defect exists in the primary Rust implementation, and fix it there first if
it does. **The check was run. It is negative**, and this is measured, not assumed:

- `me-cli` never derives a wallet from a mnemonic. Its only BIP-39 use is a
  *generated* 12-word phrase used as an encryption passphrase — the module's own
  words, `crates/me-cli/src/seal/passphrase.rs:9`:
  `//! Used ONLY as a passphrase: never seed entropy, never derives a wallet.`
- `me bundle` emits a **build manifest**, not a restore document read years
  later. There is no "Full (seed + keys)" label and no completeness claim in the
  Rust surface:

      grep -rni "restore doc\|full (seed\|watch-only" --include="*.rs" crates/   # → no hits

So F-198's defect class has **no Rust counterpart**. This is fork-native
GUI/UX code with no Rust sibling — exemption (b) of the Rust-primary rule. This
cycle leads nothing.

---

## 3. Scope

Decided by the operator stand-in; full reasoning in
`design/agent-reports/s6a-scope-and-design-decisions.md`.

**S6a contains four items:**

| id | severity | what |
| --- | --- | --- |
| **F-198** | **Critical** | the single-sig passphrase label, and a restore document that has no inventory at all |
| **C-1** | **Critical (new, R0)** | a restore document that vouches for plates the device has just said are wrong — **on all three paths** (§4.7) |
| F-197 | Important | the single-sig engrave does not stop on an aborted set — plus the false claim in `bundle_flow.go` that says it does |
| F-195 | Important | no document states outright that a watch-only set contains no seed |
| **F-202** | Minor (new) | single-sig shows no pre-engrave plate census |

All three filed items are the same sentence said three ways — *the single-sig
path never learned what S4/S5 taught the multisig paths* — and they land in the
same fifteen lines of the same function. Splitting them would make a reviewer
read that function three times and would leave the Critical's own fix
half-delivered (F-198's label without F-198's document).

**F-202 enters through the R0 gate, not around it.** It was found during this
recon and is unfiled; it is named here *before* review precisely so it is not a
fold-time addition. If it had arrived mid-cycle it would have been filed instead.

**F-199 is NOT in S6a**, and this is scheduling, never deferral. It is a
different file, a different concern, and its own follow-up says it "needs a
decision, not a reflex" — bundling an open design question with a Critical is how
the Critical ends up waiting. It stays S6-owned and gating: **it gets its own
cycle (S6b), opening with a decision pass, and the hardware flash does not happen
until S6b also closes GREEN.** If schedule pressure ever forces a choice, the
flash waits.

### 3.1 ASSUMPTIONS THIS PLAN MAKES — declared loudly

Per the operator directive's third rule. Each is a place where this plan chose an
answer the request did not specify.

1. **The supply path's restore document CHANGES, and that is a correction.**
   §4.3 keys the seed-handling ruling on the path's seed *capacity*. Measured,
   the multisig SUPPLY path holds exactly one seed by construction
   (`gui/multisig.go:355`), so it moves onto the one-seed arm. Its document
   today says "this build can hold several", **which is already false there** —
   S5 wired the shared function to a path it did not fit. So S5-reviewed output
   does change: the BUILD path is byte-identical, the SUPPLY path is corrected.
   The operator stand-in's assumption 8 ("multisig documents do not churn") is
   **wrong on the supply path**, and this plan overrides it, having measured the
   capacity the stand-in was not given.
2. **Capacity is a property of the PATH, not of the run.** A multisig build that
   happened to take one seed still truthfully "can hold several"; two otherwise
   identical builds must not print different documents because of runtime
   happenstance.
3. **`passphrase != ""` is the predicate**, everywhere. An explicitly-bound empty
   passphrase is *no* passphrase — `syswPassphraseFlow` can return `("", true)`,
   and a build engraving a plain seed must not be labelled as though a factor
   were missing (`seedPassphraseFact.Uses`, `gui/multisig_build_census.go:225`).
4. **Nothing is refused, and no capability is removed.** Rule 1 of the directive.
   A passphrase single-sig engrave still proceeds; it is only labelled and
   documented truthfully. The one new *stop* (F-197) fires on an abort the
   operator themselves initiated.
5. **The census title is "Plates To Cut"**, matching the other front-door path
   (`gui/multisig.go:279`). The build path says "Plate Count"
   (`gui/multisig_build.go:394`) — a pre-existing inconsistency this plan does not
   fix; filed as **F-203**, ownerless Nit.
6. **Watch-only single-sig does hold a seed in device memory during the build —
   but that settles only HALF of the seed-handling sentence.** The measurement is
   sound: `gui/singlesig.go:41` obtains the mnemonic and `:90` derives from it
   **unconditionally**, so the *device-memory* clause is true in both modes.
   **The first draft then wrote the assumption as though it covered the whole
   sentence, and it does not** (R0 I-2). The same string ends "the plates are the
   secret", which is **false on every watch-only run of every path**, and §4.4
   would have placed a new line saying the opposite four lines above it. §4.3 now
   conditions that clause on `bundleSetCarriesASecret(cards)` instead. Recorded
   as an assumption that was *wrong*, not one that held: it is the §1.3 landmine
   class committed by the plan that named the class — one clause of a shared
   string audited, the clause beside it not.
7. **THIS PLAN DEPARTS FROM `SPEC_seedhammer_T6a_singlesig_flagship.md`, AND
   THAT SPEC MOVES** (R2 spec-coverage). That spec governs the single-sig restore
   document and describes it **exhaustively** — `:36`, *"restore doc (R0-M2):
   display-only + optional NFC; master fp + the concrete descriptor + first
   receive/change address … greps clean of any xprv/private material"*. Four
   fields, and §1.1 measured the shipped code as still matching it exactly.
   §4.2, §4.4 and §4.7 add a plate inventory, a seed statement and a
   verification status line. **The first two drafts of this plan cited that spec
   zero times** while carrying a section built to declare exactly this kind of
   thing.
   Not Critical: nothing in T6a forbids additional non-secret content, so no
   normative MUST is broken, and the "greps clean of private material" constraint
   is **preserved** — every added line is public (a plate census, a passphrase
   fact, a verify outcome). What breaks is the spec's *exhaustiveness*. So the
   spec is updated in this cycle, in its own commit, separate from the plan and
   from the code — a spec left describing the old artifact is the trap this
   project has been caught by before.
8. **The presence-arm sentence names the inventory label `ms1 secret share` as a
   PREFIX, not an exact string.** Measured: single-sig labels the card exactly
   `"ms1 secret share"` (`gui/singlesig_engrave.go:25`), multisig numbers it via
   `numberedLabel("ms1 secret share", i, n)` (`gui/multisig_engrave.go:37`) →
   `"ms1 secret share 1 of 2"`. The wording "the plate marked 'ms1 secret share'"
   is true of both. A future relabel must propagate into these lines; that
   coupling is deliberate and greppable.
### 3.2 THE R0 ROUND GREW THIS CYCLE — declared, not absorbed quietly

The first draft was a single-sig cycle. **C-1 pulls both multisig paths in**, and
that is a scope increase the operator should see rather than find in a diff.

**Why it could not be filed.** The same fall-through exists on the multisig
supply and build paths: the retry loop breaks on `!ok || sel != 0`, so an
operator who reads a FAILED verify and presses CONTINUE reaches
`multisigRestoreDocFlow` anyway. The next phase is a **hardware flash where an
operator cuts real backups**, and the hardware gate itself requires a multisig
engrave-and-restore (plan §3, S6 items 1, 2 and 4). Filing this would park a
known funds hole on exactly the territory the next phase enters — and under this
project's own rule an item scheduled to a phase is not deferrable past it.

**What it costs — corrected twice, so here it is measured rather than argued.**
`multisigVerifyResult` has **five** constants (`gui/multisig_verify.go:88-100`;
round 0 said "four" and was wrong — as does the type's own doc comment, §4.7c).

**It is NOT a superset of what the status line needs** (R2 I-1): it has no value
for *skipped* or *never offered*, which is the commonest outcome of all, since
the operator may simply press Skip. §4.7c therefore defines a separate
`verifyStatus` whose zero value is the safe one.

**And `multisigRestoreDocFlow` DOES change signature** (R2 C-1). Round 1 claimed
it did not, reasoning that `extra []string` already existed. The document needs
**two** insertion points — status first, inventory last — and one trailing
parameter cannot express that.

**THREE call sites change, not two (R3 I-5 / comprehension I-5).** Measured:

    grep -rn "multisigRestoreDocFlow(" --include="*.go" gui/ | grep -v "^.*func "
    gui/multisig_build.go:478
    gui/multisig.go:361
    gui/multisig_nested_name_test.go:230      <- passes nil, renders a REAL document

The third is a test, and it is the kind that matters: it drives a real restore
document, so it does not merely need re-compiling — it needs a status argument
that is *correct for what it drives*, and it is a place the new line can be
asserted for free.

What genuinely does **not** change: the retry loop's control flow, and the set of
verdicts. The engrave, derive and encode paths are untouched, and no plate
content moves.

**One place multisig differs, and §4.7 is scoped around it:** the BUILD path
skips the restore document entirely for a template engrave
(`gui/multisig_build.go:464`, `if !template`). So the claim is *"wherever a
document renders, it carries a status line"* — **not** "a status line always
renders". The first fold wrote the stronger claim and it is false there.

**The fallback, stated in advance so it is a decision and not a surprise:** if
the multisig side turns out to need structural change, it becomes a **named gate
on the hardware flash** — never a batched follow-up.


---

## 4. Per-item design

### 4.1 F-198a — the mode label (`gui/singlesig.go:80`)

    Choices: []string{buildFullModeLabel(passphrase != ""), "Watch-only (keys)"},

That is the whole change. The correct string already exists and the passphrase is
already in scope at that point (`:64-74` precedes `:77`). This mirrors
`gui/multisig.go:217` exactly.

### 4.2 F-198b — the restore document gets an inventory

`restoreDocFlow` (`gui/singlesig_restore.go:119`) gains **two** parameters — a
**leading** `status []string` and a **trailing** `extra []string` — exactly
matching the shape §4.7b specifies for `multisigRestoreDocFlow`:

    func restoreDocFlow(ctx *Context, th *Colors, xpub string, masterFP, parentFP uint32,
        script md.ScriptKind, path bip32.Path, status []string, extra []string) {
        ...
        restoreDocScreen(ctx, th, append(append(status, lines...), extra...))
    }

and the one call site (`gui/singlesig.go:136`) supplies both:

    restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path,
        buildVerifyStatusLines(status),
        buildPlateInventoryLines(cards, oneSeedPassphraseFact(passphrase != ""), seedCapacityOne))

`cards` is already in scope from `:126`. Blast radius is one production call site
and zero test call sites (§1.8).

**WHY TWO PARAMETERS AND NOT ONE, on this path too.** The first three drafts of
this section specified only the trailing `extra`, and the R2 fold corrected that
for multisig **and left this section behind** — while §4.7b went on to reference
"the leading parameter §4.2 adds", which §4.2 did not add. So the plan asserted a
design it did not contain, on the *more travelled* of the two paths.

That is R2 C-1's own defect class, reintroduced **by omission**, in the fold that
closed it: `append(lines, extra...)` cannot place anything at slice index 0, and
index 0 is what §4.7b measured "page 1" to mean. **Folds fail by incomplete
propagation — the facts get corrected and the duplicates are left standing** —
and reading the diff does not find it, because the defect is in the text the diff
did *not* touch. It was caught by a cheap pre-review pass grepping for the stale
shape, which is precisely the job that pass exists to do.

### 4.3 The capacity-keyed seed-handling ruling

New named type in `gui/multisig_build_census.go`, so a call site reads as an
intent rather than as an opaque boolean:

    // seedCapacity is how many seeds a PATH can hold, which is what the
    // seed-handling ruling describes. It is deliberately not the runtime count:
    // a build that happened to take one seed can still hold several, and two
    // identical builds must not print different documents.
    type seedCapacity int

    const (
        // seedCapacityOne: one seed seam by construction — the single-sig flow
        // and the multisig SUPPLY path.
        seedCapacityOne seedCapacity = iota
        // seedCapacityMany: the multisig BUILD path's registry, one seed per
        // held slot.
        seedCapacityMany
    )

**THE RULING HAS TWO INDEPENDENT AXES, NOT ONE (R0 I-2).** The first draft keyed
the whole sentence on capacity and audited only the clause it had come to fix.
The sentence contains a second conditional claim:

> Do not leave a mid-build machine unattended: **the plates are the secret.**

That is **false on every watch-only run**, on all three paths — and under §4.4 it
would sit four lines below a new sentence asserting *"no plate in this set holds
it"*. The document would contradict itself about the one thing it exists to
settle. §3.1.6 measured that the device holds a seed in memory in watch-only mode
and wrote the assumption as though it covered the *plates* half too. It does not.

So the ruling is assembled from two selectors:

| axis | source | what it selects |
| --- | --- | --- |
| **path capacity** | the `seedCapacity` argument | "The seed you entered -- this build holds exactly one --" vs "Every seed you entered -- this build can hold several --" |
| **seed on the plates** | `bundleSetCarriesASecret(cards)` — a fact of THIS run | whether the "words are also on the plates" / "the plates are the secret" clauses appear at all |

    base := "Seed handling: this build does not time out. " + subject +
            " stays in device memory until the build ends"

    subject = "The seed you entered -- this build holds exactly one --"    // seedCapacityOne
            = "Every seed you entered -- this build can hold several --"   // seedCapacityMany

    // seed IS on the plates:
    base + ", and on a full build the words are also on the plates as they are cut. " +
           "Do not leave a mid-build machine unattended: the plates are the secret. " +
           "Power the device off when you are done."

    // seed is NOT on the plates (watch-only, any path):
    base + ". Do not leave a mid-build machine unattended: it is still holding " +
           "seed material. Power the device off when you are done."

**"seed material" — and the two rejected drafts are recorded, because each was
wrong in an instructive way.** The shipped singular *"your seed"* is a
singular/plural wobble on the multi-seed BUILD path. The R1 fold replaced it with
*"the words you typed"*, which fixed the wobble and **introduced a falsehood**
(R2 I-5): `seedEntryFlow` is the *source picker*, not a keyboard — the flow's own
security spine says so at `gui/singlesig.go:20-23`, and §3.3.2 admits a
payload-borne `ClassMnemonic` on purpose. On a payload-sourced run **nothing was
typed**. "seed material" is number-neutral and provenance-neutral, so it is true
on every path and every source.

Worth naming as a pattern rather than a one-off: a **wording** fix for a
**wording** defect introduced a **factual** one, on a document read years later.
Any replacement string is new text and inherits the full truth obligation.

**Byte-identity check, and what deliberately churns.** `seedCapacityMany` +
seed-on-plates reassembles the shipped string **byte for byte**, so the multisig
BUILD path's full-mode document is unchanged. Two documents *do* change, both
because they are wrong today:

- **multisig SUPPLY, any mode** → the one-seed subject (§3.1.1).
- **any path, WATCH-ONLY** → loses the "plates are the secret" pair, which is
  false there.

**"on a full build" is kept verbatim inside the seed-bearing arm** even though
that arm now fires only on seed-bearing builds, making the qualifier vestigial.
It is retained because removing it would re-word an S5-reviewed sentence for no
gain in truth, and byte-identity with reviewed text is worth more than tidiness.
Flagged here so a reviewer does not read it as an oversight.

**Call sites (all 8, measured):** production `gui/multisig_build.go:479` →
`seedCapacityMany`; `gui/multisig.go:362` → `seedCapacityOne` (see §3.1.1);
`gui/singlesig.go:136` (new) → `seedCapacityOne`. Tests:
`gui/multisig_build_prose_test.go:369,424,425` and
`gui/multisig_build_perseed_passphrase_test.go:134,246,304` → all
`seedCapacityMany` (they are build-path tests).

**An existing test already guards this wiring.**
`TestSeedResidencyRulingDescribesTheMultiSeedReality`
(`gui/multisig_build_prose_test.go:364`) asserts the ruling contains `"Every seed"`.
If the implementer wires the build path to the wrong arm, that test fails. It
must be updated to pass `seedCapacityMany` **and keep every assertion it has** —
weakening it to accommodate the new signature is a Critical.

### 4.4 F-195 — the seed statement, both arms

New function beside the passphrase one, same file. **It takes no `seedCapacity`
(R0 I-1):**

    func buildSeedInventoryLines(cards []bundleCard) []string

**WHY NOT CAPACITY — the two sentences answer two different questions.** The
first draft keyed this on the same `seedCapacity` as §4.3, and that is wrong.
Capacity is a property of the **path** (how many seeds the device can hold), and
§3.1.2's argument for it holds only for the seed-handling ruling. The seed
statement is a claim about **what is on the plates in front of the reader** — a
fact of the *run*. Keying it on capacity makes the ordinary one-slot multisig
build (the common case: the operator holds one key in a 2-of-3) print
*"this set CONTAINS seeds … Each plate marked 'ms1 secret share'"* over **one**
plate — which `numberedLabel` leaves **unnumbered** at n=1
(`gui/multisig_engrave.go:37`, whose own comment says a one-leg build "reads
exactly as it always did"). A reader counting one plate against a document
saying *each* concludes a plate is missing from a complete set, and stops. That
is the self-vouching defect run backwards, and this document's own prose
(`gui/multisig_build_census.go:110-114`) names giving-up-on-a-recoverable-backup
as the failure mode it exists to prevent.

So the discriminant is the **ms1 card count in `cards`**, counted here. Absence
still uses the existing `bundleSetCarriesASecret(cards)`
(`gui/bundle_flow.go:482`) — one definition of "this set holds a seed", so the
abort warning and the restore document can never disagree.

**Absence arm** (no ms1 card — watch-only, any path):

> Seed: this set contains NO seed. It is watch-only: it records the wallet, but it can never spend. If funds must be recovered, the seed words must come from somewhere else -- no plate in this set holds them.

**Presence arm, exactly one ms1 plate:**

> Seed: this set contains YOUR seed, on the plate marked 'ms1 secret share'. Treat that plate as the secret itself.

**Presence arm, several ms1 plates:**

> Seed: this set contains YOUR seeds, on the plates marked 'ms1 secret share'. Treat each of those plates as the secret itself.

**Three things this wording does on purpose:**

1. **"YOUR seed", not "THE seed" (R0 I-3).** The definite article, sitting
   directly under *"If any of them is missing, this backup is incomplete"*,
   answers the reader's one question — *is this everything?* — with **yes**. On a
   2-of-3 that answer is false and costs the recovery. `"your seed"` is true in
   all six modes and is already the codebase's own word for this fact
   (`oneSeedPassphraseFact`, `gui/multisig_build_census.go:198`, which feeds
   *"Needs a passphrase: your seed"*).
2. **No sufficiency claim in either direction.** It does not say the seed plates
   alone can spend (false on k-of-n), and it does not say they cannot (false on
   single-sig). It states presence and consequence, and stops. How many keys must
   sign is the descriptor's job, on the same page.
3. **No address claim (R0 M-1).** The earlier draft said the plates "can rebuild
   the wallet's addresses". On a supplied policy that `expandedToDescriptor`
   cannot render, the same document already says *"Addresses unavailable for this
   policy shape."* (`gui/multisig_restore.go:26-31`) — a visible contradiction on
   the page. "It records the wallet" is true in both cases.

Passphrase facts stay out of these lines entirely; that is the passphrase lines'
job, immediately after.

**THE ADVERSARY SIDE, STATED — because arguing only one side is the gap** (R5
disclosure, Minor). This statement is a genuinely NEW disclosure: today's
single-sig document says nothing about mode, so a reader holding the **document
but not the plates** learns a definite yes/no on whether a seed-bearing plate
physically exists for this wallet. That is help to a thief deciding whether the
steel is worth hunting for.

**The trade-off still favours saying it, for three measured reasons:**

1. The same document already carries the **descriptor and both first addresses**,
   which identify the wallet and its balance to anyone who reads them. That is a
   far stronger incentive to go hunting than the one bit added here.
2. **The identical fact already exists** on the abort-warning screen
   (`gui/bundle_flow.go:555`, *"No plate in this set carries a seed."*). This
   moves it from a transient screen to the durable artifact, which is the entire
   point of F-195 — it does not invent the disclosure.
3. The reader this document exists for — a stranger, years later, holding a pile
   of steel and asking *is this everything?* — cannot act without it, and
   silence is the state in which recoverable backups get abandoned.

Recorded here rather than left implicit, because §4.7's incentive argument states
both sides and this one did not. Under the operator's directive to declare
assumptions loudly, a one-sided argument in the same document is the
inconsistency.

**Placement inside `buildPlateInventoryLines`**, so the document reads
what-it-is → what-it-is-not → how-to-handle-it:

1. the plate list and `"If any of them is missing, this backup is incomplete."` *(unchanged)*
2. **the seed statement** *(new)*
3. the passphrase statement *(unchanged)*
4. the seed-handling ruling *(capacity-keyed, §4.3)*

**ASCII ONLY.** `TestSeedResidencyRulingDescribesTheMultiSeedReality:395` fails
the build if the ruling contains any of `— – · ' ' " " …` — the body face lacks
those glyphs, so such a line does not draw. Every new string above uses `--` and
straight quotes. The new lines get the same guard (§5, T7).

### 4.5 F-197 — the abort gate, and the claim that said it was already there

`gui/singlesig.go:127`:

    if bundleEngrave(ctx, th, "Engrave Single-Sig", cards) != bundleEngraveDone {
        return
    }

**And the comment at `gui/bundle_flow.go:535` is corrected in the same change.**
It currently reads "both engraving callers now gate it … so an operator whose
engrave died really does not reach it". Measured (§1.5), three callers carry a
post-engrave tail, two gate, and the claim was **false when written**. It must
name all three, and say why `gui/bundle_flow.go:39` needs no gate (it `return`s
immediately; nothing after it vouches for the set). Leaving a false justification
in place is how the next reviewer inherits it as a given — F-196's lesson, one
stage earlier, in this codebase.

### 4.6 F-202 — the pre-engrave census

Immediately before the engrave in `gui/singlesig.go`, mirroring
`gui/multisig.go:279`:

    if !confirmReviewScreen(ctx, th, "Plates To Cut", buildPlateCensusLines(cards)) {
        return
    }

Back here aborts before anything is cut, which is the last moment that is free.

---

### 4.7 G2 — the document must not vouch for plates it has evidence against

**REWRITTEN AT ROUND 9, AFTER NINE RED ROUNDS. §0.1 explains why.** The previous
apparatus — six knowledge states plus a reserved seventh, per-observation
world-sets, a lattice-free switch over a classified observation, an enforcement
artifact and a coverage script — was chasing **NG1**, an unstated goal. Every
Critical of this cycle came from it. What follows serves **G2 only**, and G2 is a
prohibition, which is why it is discharged by construction rather than by
enumeration.

#### 4.7a FOUR STATES — the 2×2 of TWO RECORDED BOOLEANS

No lattice. No ordering. No observation enum. No reserved status. No
world-set table to keep complete.

    // Both are RECORDED at return sites this plan owns, never inferred from a
    // verdict. A verdict is the proxy two rounds proved unsound; it is not read
    // here at all.
    fullPassRecorded := false   // written AT THE SUCCESS RETURN, with `full` in
                                // scope, recording WHICH comparisons ran and
                                // matched in this mode.
    adverseRecorded  := false   // STICKY. Written at any return site whose world
                                // -set contains a bad-plate world.

    // ...and the document line is the cell, not a search:
    switch {
    case fullPassRecorded && adverseRecorded: status = statusVerifiedOnRetry
    case fullPassRecorded:                    status = statusVerified
    case adverseRecorded:                     status = statusCheckDidNotPass
    default:                                  status = statusNotFullyChecked
    }

**The `default` is the ZERO CELL and it is where everything unclassified lands** —
skip, incomplete, a benign refusal, an abort, a path nobody enumerated, a path
added next year. **Monotonicity is now structural rather than promised:** a fact
not recorded cannot set a bit, and an unset bit can only move the cell *toward*
`statusNotFullyChecked`. There is no arm that an omission can strengthen, which
is precisely what the previous design got backwards.

#### 4.7b WHICH SITES ARE ADVERSE — the whole classification

The only classification left is one bit per return site, and R9 already verified
it row by row:

| adverse (world-set contains a bad-plate world) | benign (nothing observed about the plates) |
| --- | --- |
| `gui/multisig_verify.go:719` foreign-or-garbled md1 | `gui/multisig_verify.go:897` re-typed seed will not derive |
| `gui/multisig_verify.go:724` readback will not decode | `gui/multisig_verify.go:938` zero legs, correctable |
| `gui/multisig_verify.go:394` `errVerifyLegHasNoPlate` | `gui/multisig_verify.go:940`, `gui/multisig_verify.go:696` abandons |
| `gui/multisig_verify.go:738` plate count ≠ engraved count | `gui/multisig_verify.go:670`, `gui/multisig_verify.go:680`, `gui/multisig_verify.go:794` structural refusals |
| **any `bundle.Verify` error** | *(loop exits, skip)* |
| `gui/multisig_verify.go:701` readback filter drops cards | `gui/multisig_verify.go:979` partial verify, everything compared matched |
| `gui/multisig_verify.go:963` `verifyMultisigLegsPartial` mismatch | |
| `gui/multisig_verify.go:984` `verifyMultisigLegs` mismatch | |

**`gui/multisig_verify.go:987` is neither** — it is the **success return**, and it
is where `fullPassRecorded` is written **with `full` in scope**. It sets no
adverse bit; it records *what was compared and matched in this mode*. That single
site is the whole of P5(a) and the whole of R9's C-1 fix: the pass line is
generated from what this record contains, so on watch-only it cannot claim an ms1
comparison the record does not hold.

**All fifteen `return verify*` sites are now classified**, and
`./scripts/verify-returnsite-sweep.sh` is what proves it rather than care —
though note its declared scope warning: single-sig contributes zero sites until
`singleSigVerifyFlow` gains a verdict at build-order step 1, so this covers
multisig only.

**`bundle.Verify` NEEDS NO CHANGE, and this is what kills P5(b)'s unenforceable
instance genuinely rather than hiding it.** All eleven of its errors classify
**identically** — adverse — at the gui call site. The ms1-versus-plate provenance
distinction that lived only inside it, and that P5(b) could not reach, turns out
to be a distinction **nothing consumes**. There is no sub-classification within
adverse.

#### 4.7c THE FOUR LINES

    type verifyStatus int
    const (
        statusNotFullyChecked verifyStatus = iota  // THE ZERO VALUE. Safe, and true.
        statusCheckDidNotPass
        statusVerifiedOnRetry
        statusVerified
    )

| cell | status | line (verbatim, ASCII only) |
| --- | --- | --- |
| pass, no adverse | `statusVerified` | *generated from the pass record* — names exactly the comparisons this mode ran, and states what was not read |
| pass, adverse | `statusVerifiedOnRetry` | the generated pass line, plus `An earlier check did not pass; a later full check passed.` |
| no pass, adverse | `statusCheckDidNotPass` | `A verification check ran and did not pass: a comparison did not match, or a plate could not be read or accounted for. Do NOT rely on this backup until a full check passes. Check again with every plate this run engraved; if this repeats, engrave a fresh set.` |
| no pass, no adverse | `statusNotFullyChecked` | `These plates were not fully checked. Confirm they restore this wallet (master fingerprint below) before relying on this backup.` |

**THE PASS LINE IS GENERATED, NOT A LITERAL — this is what closes R9's C-1.**
`buildVerifyStatusLines` takes the **pass record**, which carries the mode. On a
full run it names the key and descriptor plates AND the typed-ms1 comparison; on
watch-only it names only what watch-only actually compares, and **the ms1 clause
is absent because the record does not contain it.** A mode-blind literal is what
claimed "the ms1 you typed matched this seed" on a run where no ms1 is typed —
the bug `multisigVerifyOKMessage` had already found and fixed, in all four of its
arms, before this plan re-committed it.

#### 4.7d THE MEMBERSHIP TEST — why four and not two, six, or seven

A distinction earns a line **iff both prongs hold**:

1. **ENFORCEABILITY** — its generating facts are values in scope at return sites
   **in code this plan owns**. The boundary lies *on* the return-site partition,
   never through a callee's interior.
2. **CONSUMPTION** — across the boundary the stranger's required action differs,
   **or** a settled property forbids the merge.

| distinction | prong 1 | prong 2 | verdict |
| --- | --- | --- | --- |
| pass vs not-pass | success return, `full` in scope | rely-with-scope vs do not | **kept** |
| adverse vs benign | all gui-local (§4.7b) | "confirm before relying" vs "do not rely until a check passes" | **kept** |
| VERIFIED vs on-repeat | same two bits | action identical — but **P2 forbids the merge** | **kept, and free** |
| DISAGREED vs UNACCOUNTED | **fails** — interior to `bundle.Verify` | — | **dropped** |
| skip vs incomplete | — | **fails** — same action | **dropped** |

**Two states would have been wrong**, and this is the part the controller's own
hypothesis got wrong: on **adverse → clean retry**, a sticky adverse bit violates
P1 and kills the re-verify incentive, while a non-sticky one violates P2 on the
ms1 class, which R9's I-1 proved is not retro-explainable. Neither arm of a
two-state collapse is available. Four is the smallest set that is both
enforceable and sufficient.

#### 4.7e WHAT THIS DELIBERATELY GIVES UP

**`DISAGREED`'s condemnation.** The old design could say *a read-back check
DISAGREED with these plates*. The new one says only *a check ran and did not
pass*. That is a real loss of specificity **and it was a promise, not a
capability** — the device could not reliably earn the stronger claim, which is
what three consecutive Criticals were about. §0.1's test applies: a stranger can
only rely, confirm, or re-cut, and both lines route to the same action.

#### 4.7f THE STATUS MUST SCOPE THE PAGE BENEATH IT (R6 reader I-1)

Under `statusCheckDidNotPass`, a **scoping line** renders immediately after the
status line, because nothing below it is conditioned on it:

> `Everything below describes what this run INTENDED to engrave. Until the check above is resolved, do not assume the plates match it.`

**Not under `statusNotFullyChecked`**, whose own line already tells the reader to
confirm before relying — R7 ruled that widening it to the modal Skip path cries
wolf on a backup that is probably fine.

#### 4.7g P6 — THE PASS PATH IS AUDITED LIKE THE FAILURE PATHS

R9's root diagnosis was that **the pass path was never held to the discipline the
failure paths were**: five verdict sites audited for one failure verdict, three
for another, and the success return got one row, added last, carrying the only
unscoped positive claim in the plan.

> **P6 — every positive line is audited BY CLAIM, PER MODE.** For each claim a
> pass line makes, in each mode the flow supports, the reviewer names **which
> recorded observation says so**. A claim with no naming observation is deleted.
> Guards are **entitlement, never inference** -- a mode guard says *this record
> entitles this clause*, it never infers a fact from a verdict. And enumerations
> are **outcome-blind**: every list of return sites includes the success return,
> or it is not a list of return sites.

**P5 survives in reduced form** — (a) claims generated from records, and (c)
monotone under omission, both now structural rather than promised. **P5(b) is
retired**: §4.7b shows the distinction it was written to enforce is one nothing
consumes.


## 4.8 BUILD ORDER — what to do first, and what can land on its own

The comprehension review found the plan gave none, and that its single ordering
sentence was wrong. Nine steps.

**NOT every step leaves the tree green, and an earlier draft claimed it did
(R4 I-2).** Steps **1–4** and **8–9** each leave it green and are independently
landable. **Steps 5, 6 and 7 land TOGETHER, as one commit.** Two reasons, and the
second is the serious one:

- Step 5 alone leaves the suite **red** — step 6 exists to repair the three walks
  step 5 breaks, and step 6's own cell already said "must accompany step 5",
  contradicting the header above it.
- **Steps 5+6 without 7 are green AND landable AND are exactly C-1's harm**: a
  restore document carrying a full inventory and completeness claim, with no
  verification status line on it. A state the suite would call healthy is the
  precise defect this cycle exists to close, so the build order must make it
  unreachable rather than merely undesirable.

| # | step | why here |
| --- | --- | --- |
| 1 | **Write the single-sig exit → `verifyStatus` mapping** (§4.7c) and get it reviewed | eleven exits, and every later step depends on it. Nothing else starts until it is agreed. |
| 2 | `verifyStatus` + `buildVerifyStatusLines` + **T14 only** | pure function, no callers yet. **T9, T13a and T13b do NOT land here** — §5.2 requires them on a rendered document and on a multisig walk, so they move to step 7 |
| 3 | `seedCapacity` + the two-axis ruling + `buildSeedInventoryLines` (§4.3, §4.4), updating the six existing call sites | shared census; still no flow changes |
| 4 | `restoreDocFlow` and `multisigRestoreDocFlow` gain `status` + `extra` (§4.2, §4.7b), **all three call sites** | signature change; the tree must stay green across it |
| 5 | Wire single-sig: label (§4.1), inventory, census (§4.6), abort gate (§4.5) | the F-198/F-195/F-197/F-202 body of work |
| 6 | Update the three walks that the census screen stops (§5.1b) | must accompany step 5, not follow it |
| 7 | Wire the verify status into all three flows, plus T9, T10, T11, T12, T13a, T13b | needs 2, 4 and 5 in place. **T9/T13a/T13b need a rendered document and a multisig retry loop**, neither of which exists at step 2 |
| 8 | Correct the three false comments (§4.7c) + T8 | independent; deliberately last so it cannot mask a behavioural regression |
| 9 | Update `SPEC_seedhammer_T6a_singlesig_flagship.md` (§3.1.7), **in its own commit** | the spec follows the behaviour, and is not mixed with it |

**Step 1 is a gate, not a task.** It produces a table of eleven rows, it is
reviewed before step 2 begins, and it is the one place this plan deliberately
delegates a decision — so it does not get made silently inside an
implementation.

### 4.9 The spec update — what it says, and how it is checked (R3 I-8)

§3.1.7 declares this plan departs from `SPEC_seedhammer_T6a_singlesig_flagship.md`
and that the spec moves. **It named no content and no check**, which is the same
"filed rather than smuggled in" shape F-196 was written about: a promise that
resolves to nothing.

**What changes.** That spec's line 36 currently describes the restore document
exhaustively:

> `restore doc (R0-M2): display-only + optional NFC; master fp + the concrete descriptor + first receive/change address ... greps clean of any xprv/private material`

It gains the three additions this cycle makes, and keeps the constraint that
still binds:

1. a **verification status line**, always exactly one, first on the page (§4.7);
2. a **plate inventory** — count, per-card census, completeness sentence (§4.2);
3. a **seed statement** and a **passphrase statement** (§4.4, §4.3);
4. **unchanged and still normative:** display-only, no secret material. Every
   added line is public — a plate count, a passphrase fact, a verify outcome —
   so `greps clean of any xprv/private material` holds and is **not** weakened.

**How it is checked.** The spec is prose, so the check is a grep pair rather than
a test, run as step 9's gate:

    grep -c "verification status" design/SPEC_seedhammer_T6a_singlesig_flagship.md   # expect >= 1
    grep -c "xprv"                design/SPEC_seedhammer_T6a_singlesig_flagship.md   # expect unchanged

**And it lands in its OWN commit** (§4.8 step 9), separate from plan and code, so
a future reader can see what the spec said before this cycle and what it says
after, without the behaviour change mixed in.

## 5. Test plan

**Every test below must be shown to FAIL against ITS OWN MUTATION** — the one
named in its row — **not against the pre-cycle tree.** A green suite proves
nothing on its own: 9 of round 0's 17 blocking findings in S5 were reproduced by
mutating the tree and watching a green suite stay green. The implementer reports,
per test, the mutation applied and the failure message observed — **and proves
the mutated line RAN**, not merely that the edit landed.

**The earlier wording said "FAIL against the unfixed tree", and that was wrong
for eight of these rows** (R3 comprehension I-6). Two distinct reasons, and the
distinction matters because the sloppy version is unsatisfiable rather than
merely loose:

- **T3 is a NON-VACUITY test and PASSES on the unfixed tree by design.** It
  asserts a bare run does *not* say "NOT passphrase" — which is exactly what
  today's code does. Demanding it fail pre-cycle would demand it be wrong.
- **A test of a function that does not exist yet does not "fail"; it does not
  COMPILE.** T9, T13a, T13b and T14 target `buildVerifyStatusLines` and
  `verifyStatus`, which this cycle introduces. "Red" from a missing symbol proves
  nothing about the assertion.

So the standard is per-row and adversarial: **revert or mutate the specific
behaviour the row names, on an otherwise-complete tree, and watch that row go
red.** That is what actually demonstrates the assertion bites.

New file: `gui/singlesig_truth_test.go`. Prior art to mirror is in §1.7.

| id | asserts | mutation that must break it |
| --- | --- | --- |
| **T1** | a passphrase single-sig run's engrave-mode screen contains `NOT passphrase` | revert `:80` to the literal |
| **T2** | that run's restore document contains `BIP-39 passphrase WAS used` **and** `This backup is` (the inventory reached the document at all) | pass `nil` as `extra` |
| **T3** | *non-vacuity:* a **bare** single-sig run's label does NOT contain `NOT passphrase`, and its document says `No BIP-39 passphrase was used` | make `buildFullModeLabel` always return the passphrase arm |
| **T4** | watch-only single-sig document contains the absence line; full contains the presence line | swap the arms of `buildSeedInventoryLines` |
| **T5** | after aborting the single-sig engrave, the program ENDS: neither `Verify the engraved plates?` nor `This backup is` nor `Descriptor:` is drawn afterwards | drop the `!= bundleEngraveDone` guard |
| **T6** | the single-sig run reaches `Plates To Cut` before the engrave picker | remove the census call |
| **T7** | `seedCapacityOne` yields `The seed you entered` and not `Every seed`; `seedCapacityMany` yields `Every seed`; **and every new operator string is ASCII-clean** (the glyph set at `gui/multisig_build_prose_test.go:395`) | swap the capacity arms; insert an em dash |
| **T8** | `gui/bundle_flow.go` no longer claims `both engraving callers` (source assertion, mirroring the `readGuiFile` pattern at `gui/multisig_build_prose_test.go:402`) | restore the old comment |
| **T20** | **the 2x2.** Each of the four §4.7c cells renders **its own** line, byte-exact; every rendered document carries **exactly one** | return the same string for two cells; return an empty slice |
| **T21** | **the ZERO CELL is the default.** An observation matching no recorded bit yields `statusNotFullyChecked` — including a return path added with no classification at all | make any other status the `default:` arm. This is monotonicity, and it must be structural rather than promised |
| **T22** | **the pass line is GENERATED PER MODE (R9 C-1).** On watch-only the ms1 clause is **absent**, because the pass record does not contain it; on full it is present | use a mode-blind literal — the exact bug `multisigVerifyOKMessage` had already found and fixed in all four arms |
| **T23** | **stickiness.** adverse, then no full pass → `statusCheckDidNotPass`, never `statusNotFullyChecked` | make `adverseRecorded` non-sticky |
| **T24** | **P2 on the retry path.** adverse, then a full pass → `statusVerifiedOnRetry`, never bare `statusVerified` | drop the `&& adverseRecorded` arm — a two-state collapse, which R9's I-1 proved loses the ms1 class |
| **T25** | **no verdict is read.** The status derivation references neither `res` nor any `verify*` constant — only the two recorded booleans | key a pass arm on `res == verifyComplete`, which is R9's C-3 and the proxy two rounds proved unsound |
| **T26** | **P6 — every positive claim is named per mode.** For each clause of each pass line, in each mode, a recorded observation is named; a clause with none is deleted | add an unbacked clause to a pass line |
| **T11** | the status line is at **slice index 0** of what `restoreDocScreen` receives, and the §4.7f scope line renders **only** under `statusCheckDidNotPass` — asserted **through a production flow**, not on a helper | pass it via the trailing `extra` parameter, as the round-1 fold specified |
| **T7c** | **the capacity WIRING**, per path: drive each of the three flows to its restore document and assert the seed-handling subject clause matches that path's capacity (build → `Every seed`; supply and single-sig → `The seed you entered`) | swap either call site's capacity argument — a mutation no compiler and no current test detects |

**T23 AND T24 CANNOT RUN ON THE SINGLE-SIG PATH, and §5 put every test there
(R3 I-3).** Single-sig has **no retry loop** — its verify is a one-shot
`if sel == 0 { ... }` — so `failed → abandoned` and `incomplete → complete` are
unreachable by construction. Those rows must be driven on a **multisig**
flow, which is the only place a second attempt exists. Written as-is against
`engraveSingleSigFlow` they would pass vacuously, never reaching the sequence
they name — the exact vacuity §5.2 warns about for T5, one section later.

Consequence for §4.8's build order: step 7 is where these land, and it needs a
multisig walk, not the single-sig harness §1.7 lists as prior art.

**NO SUBSTRING ASSERTIONS ON STATUS LINES — they do not bite (R5 I-3).**
Measured over the six lines:

    Plates VERIFIED: ...                  contains "VERIFIED"
    Plates VERIFIED on a repeat check...  contains "VERIFIED"
    No check was run on these plates. ... contains NEITHER   (was the trap)
    WARNING: ... DISAGREED ...            contains "DISAGREED"

**Recounted after the §4.7d rewording: two of six contain `VERIFIED`, one
contains `DISAGREED`.** The original trap — `Plates NOT VERIFIED` matching
`Contains("VERIFIED")`, which let T13a's own named mutation pass — is **gone**,
because the reader lens forced status 3 to stop sharing a token with the pass
lines (§4.7d, R6 I-2). The rule survives its own fix: even at two of six, a
substring assertion cannot distinguish status 1 from status 2, so assertions
still compare whole strings. Every status assertion compares the **entire string**
against the §4.7d table, which also makes a reworded line a deliberate test
update rather than a silent pass.

**THREE shipped tests pin the retry-loop condition, and §5.1 named none (R5
I-2).** The `verifyMismatch` split changes that condition, so all three must be
updated **in the same commit** or the atomic 5+6+7 landing is red:

| test | what it pins |
| --- | --- |
| `gui/multisig_verify_report_test.go:166` | the flow returns `verifyFailed` so the caller can re-offer |
| `gui/multisig_verify_report_test.go:759` | same, second site |
| `gui/multisig_verify_report_test.go:1093` | the loop condition's **source text**, `res != verifyIncomplete && res != verifyFailed` |

The third is the dangerous one: it is a **source assertion**, so it fails the
moment the condition gains `&& res != verifyMismatch`, and the obvious repair —
relaxing the string — is exactly the weakening §5.1 forbids. It must be updated
to the new condition verbatim, not loosened.

**T9–T14 must pin a PRODUCTION CALL SITE, not just the pure functions (R2 I-3).**
Round 1's C-2 was that the Critical had no test; round 1 answered it with tests
that would all still pass if the status line were wired into two of the three
document flows and forgotten on the third. At least T11 and one of T10/T12 drive
a real flow to a real restore document — the same standard §5.1's T5 is held to,
and for the same reason: *a call-site assertion alone is what let the multisig
instance ship.*

**T5 is the one that carries the class.** F-197's own follow-up says it: *"A
call-site assertion alone is not enough — that is exactly what let the multisig
instance ship."* T5 must drive the real screens end to end, not assert on the
source.

**T9–T12 exist because the R1 round found the cycle's Critical had NO TEST.**
The first fold rewrote this section twice — adding §5.1 and §5.2, refining T2
through T8 — and added not one row for §4.7. The remedy for the Critical was the
only item in the plan with nothing that could fail if it regressed, while F-202,
a Minor, had a row.

**T10 is the sharpest test in this plan, and it is a regression test against the
plan itself.** Its mutation is not hypothetical: *implement the status as
last-wins* is **exactly what the first fold specified**, in writing, and it
silently downgrades a `DISAGREED` to `DID NOT COMPLETE` on the two paths that
same fold had just pulled into scope. If T10 does not fail against that
implementation, it is not testing anything.

### 5.1 EXISTING TESTS THAT MUST BE UPDATED, NOT WEAKENED

Both R0 reviewers found this list incomplete, from different lenses. It is now
two lists.

**(a) The six `buildPlateInventoryLines` call sites** in §4.3 — all gain a
capacity argument, all `seedCapacityMany`.

**(b) THREE END-TO-END WALKS THAT §4.6's CENSUS SCREEN STOPS DEAD.** The first
draft named none of them, and one of them is in a file the plan did not mention
at all:

| walk | file:line | where it breaks |
| --- | --- | --- |
| `TestEngraveSingleSigFlowFull` | `gui/singlesig_flow_test.go:51` | `:82` `click(Button3)` → `:83` `pumpUntil("Card 1 of 3", 64)` |
| `TestEngraveSingleSigFlowWatchOnly` | `gui/singlesig_flow_test.go:91` | `:121` `click(Button3)` → `:122` `pumpUntil("Card 1 of 2", 64)` |
| `TestEngraveSingleSigFlowTemplate` | `gui/template_engrave_test.go:79` | `:128` `click(Button3)` → `:129` `pumpUntil("Card 1 of 3", 64)` |

`pumpUntil` (`gui/slip39_polish_test.go:353`) **only pumps frames — it never
presses**, and `confirmReviewScreen` (`gui/multisig_build.go:1729`) loops
`for !ctx.Done` until Button1/Button3/Center. So each walk parks on the census
for its whole frame budget and hits its `t.Fatalf`.

**The required repair is one extra press**, mirroring the in-tree positive
control at `gui/multisig_verify_report_test.go:1009-1013`:

    pumpUntil(frame, "Plates To Cut", N)
    click(&ctx.Router, Button3)
    // then the existing pumpUntil("Card 1 of N")

**AND THE `Card 1 of 3` / `Card 1 of 2` DISTINCTION IS NOT WEAKENABLE.** It is
the **only executing assertion in the tree** that full mode puts the ms1 seed
plate on steel and watch-only does not. Relaxing either needle to `"Card 1 of"`,
or deleting the assertion to get green, retires that proof — and would do so in
good faith, since the first draft's not-weakened rule reached only list (a).
Doing so is a **blocking finding**, not a style note.

**Bounded, so this list is not itself incomplete:** exactly **four** tests drive
`engraveSingleSigFlow` (`grep -rn "engraveSingleSigFlow" --include="*_test.go" gui/`).
The fourth, `TestEngraveSingleSigFlowSeedScrubbed`
(`gui/singlesig_flow_test.go:141`), aborts at the wallet-type picker and never
reaches the engrave, so three is the count. `gui/singlesig_program_test.go` walks
the start-screen carousel and never enters the flow — **not** affected.

### 5.2 Per-test refinements the R0 round added

Recorded so the implementer does not rediscover them:

- **T2 has two unstated costs.** (a) `restoreDocScreen` is a **pager** and §4.2
  appends `extra` *after* the descriptor chunks and both addresses, so the
  inventory lands on the last page(s) — a single-frame assertion misses it. Use
  `s5PageForNeedle` (`gui/multisig_build_s5_flow_test.go:119`). (b) The restore
  doc sits past `bundleEngrave`, so the walk must cut **every** plate: that needs
  `p.engraver = newEngraver()`, `p.display = sh2DisplaySize`, and a per-plate
  driver (`s5EngraveOnePlate`, `gui/multisig_supply_passphrase_test.go:110`),
  none of which the current single-sig walks set up. Both omissions fail loudly;
  neither can produce a false green.
- **T3 must say which half it drives.** The stated mutation exercises the label
  only. Assert the document half directly on `buildPassphraseInventoryLines`, as
  the prior art does (`gui/multisig_supply_passphrase_test.go:305-323`).
- **T4 at unit level proves nothing about the single-sig document** — that seam
  is carried by T2 alone. Fine as designed; stated so it is a choice, not an
  oversight.
- **T5 must now press through the census** (§5.1b). Also: `"This backup is"` is
  absent from the single-sig document until §4.2 lands, so the pair that actually
  bites the stated mutation is `"Verify the engraved plates?"` and
  `"Descriptor:"`. T5 needs **no** engraver — the shipped walks already reach
  `"Card 1 of 3"` on a plain `newPlatform()`.
- **T7's absence arm is covered by nothing else.** Three existing tests already
  run the ASCII guard over ms1-**bearing** inventories; the seedless fixture is
  T7's alone and must be built on purpose.
- **T8 needs a POSITIVE half.** As written it is a bare negative that a wholesale
  deletion — or a differently-false replacement — satisfies. Its own cited prior
  art pairs `!Contains` with a positive `Contains`
  (`gui/multisig_build_prose_test.go:402-411`) for exactly this reason. T8 must
  also assert the corrected comment **names all three tail-carrying callers**.

---

## 6. Validation gate

Run from `/scratch/code/shibboleth/seedhammer`, with
`export PATH="/nix/var/nix/profiles/default/bin:$PATH"`:

    nix develop --command go test ./... -count=1
    nix develop --command ./cmd/emu/build.sh      # go test does NOT compile the emulator
    nix develop --command gofmt -l .
    nix develop --command go vet ./...

**Reading these gates, from S5's own scars:**

- `gofmt` and `go vet` output must be captured with **stderr separated from
  stdout**. Nix prints `Git tree is dirty` on stderr; a `2>&1` capture made
  `gofmt` appear to report 1 file and `go vet` 41 findings when both were clean.
  Judge on true exit codes.
- `go vet` needs a **COLD `GOCACHE`** or it prints nothing, exits 0, and proves
  nothing. **Exit 1 with 40 test-only findings IS the clean baseline** — that is
  the number to compare against, not zero.
- Bare `go` does not exist on `PATH`. A "command not found" proves nothing.

**Baseline, measured before any edit**, at fork `main` = `b8a23bf`:

    nix develop --command go test ./... -count=1   # EXIT=0, every package ok, stderr empty

So the suite is green *before* this cycle. Any red after an S6a edit is this
cycle's, not inherited.

### 6.1 The citation gate — run before a reviewer is engaged

This plan cites ~40 `file:line` locations in the fork. Those are exactly the
class of fact that the S5 cycle measured as unreliable: **16 of 16 gated code
citations were true, and 5 of 22 ungated facts were false.** Citations also decay
on every merge.

`scripts/plan-cite-check.sh` resolves every `path:line` citation in this plan
against the fork's working tree and prints the actual line, so a reviewer reads
verified anchors rather than re-deriving them:

    ./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
    # R0 fold: citations resolved: 76 / 76 ; dangling: 0   (exit 0)

### 6.2 The glyph gate — every operator string this plan writes must DRAW

`scripts/plan-glyph-check.sh` scans this plan's operator-facing strings for the
glyphs the device's body face does not carry (`— – · ' ' " " …`). A string
containing one does **not draw** on the machine, on screens whose entire job is
to say what the backup does and does not contain.

    ./scripts/plan-glyph-check.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
    # R0 fold: operator strings scanned: 41 ; undrawable: 0   (exit 0)

**It earned itself on the first fold.** It caught a string this plan had copied
verbatim out of the R0 report, where the reviewer had joined `showNotice`'s two
ASCII arguments with an em dash — so the plan carried a misquote of shipped code.
Both gates are proven with **positive controls** (a bad citation and a bad glyph
each produce exit 1), because a gate that has never failed is a hypothesis.

**What it does NOT cover, stated so the gate does not hide its blind spot:**

- It proves a cited line **exists and says what the plan quotes**. It does not
  prove the plan's *interpretation* of it is right.
- It cannot check the prose claims in §1 that are about **absence** ("no test
  pins the literal label", "zero test call sites"). Those are grep-negatives, and
  an empty result is also what a broken query looks like — each one in §1 carries
  the command that produced it so a reviewer can re-run it with a positive
  control.
- It does not compile the Go snippets in §4. They are fragments against a package
  the implementer will edit, so they remain a reviewer's execution pass.

---

## 7. What is NOT in this plan

- **F-199** — its own cycle, S6b, before the flash (§3).
- **F-196** — a model change; it earns its own R0 against the spec.
- **F-200, F-201, F-203** — ownerless Minors/Nits, batched to the end.
- **The hardware flash itself.** S6a is the software burndown that unblocks S6;
  it cuts no plates and touches no machine.
- **Any refusal or new confirmation gate in front of a passphrase build.** Rule 1
  of the operator directive. The passphrase run remains fully available; it is
  labelled and documented, not obstructed.
- **The "Plate Count" / "Plates To Cut" title inconsistency** on the two multisig
  paths (F-203).

---

## 8. Known blind spots of this plan's own gates

Stated because a gate that hides its blind spot is worse than none.

1. **Nothing here is exercised on hardware.** Every gate is `go test` and
   emulator walks. The restore document is *drawn* in a test, never *read off a
   screen by a person*, and the plates are never cut. That is S6's job, and it is
   the reason S6a is a precondition for S6 rather than a substitute.
2. **The new document lines lengthen an already-long paged screen.**
   `restoreDocScreen` (`gui/singlesig_restore.go:137`) pages with Button2 and the
   existing multisig documents already carry these lines, so overflow is not a
   new risk — but no test asserts the *last* page is reachable on the single-sig
   document specifically, and this plan does not add one. If the implementer can
   assert it cheaply, they should; if it needs a harness, it is filed, not folded.
3. **T5 depends on the abort route staying reachable.** It drives Back at the
   first engrave-style picker. If a future change moves the abort, T5 starts
   passing vacuously by never reaching the abort at all — which is why T5 must
   assert it *saw* `Bundle Incomplete` before asserting what came after, exactly
   as `TestSupplyAbortIsTheLastScreenOfTheProgram` does.
4. **The capacity parameter is a new way to be wrong, and T7 does NOT cover the
   wiring** (R3 comprehension I-7 — an earlier draft of this very item claimed it
   did, which made the blind spot worse than silence). Nine call sites carry an
   argument no compiler can check for *correctness*, only for presence. T7
   asserts `buildPlateInventoryLines` produces the right text **when handed a
   given capacity** — it says nothing about whether any *call site* hands it the
   right one.
   The build path is covered by accident, because
   `TestSeedResidencyRulingDescribesTheMultiSeedReality` asserts `"Every seed"`
   and fails loudly if wired to the one-seed arm. **The supply path and single-sig
   have no such guard**, and the supply path is precisely where §3.1.1 says the
   capacity changes S5-reviewed output. So **T7c is required**: drive each of the
   three flows to its restore document and assert the ruling's subject clause
   matches that path's capacity. Without it, a mis-wired capacity is invisible.
5. **`bundleSetCarriesASecret` is reused as the seed-presence detector.** If a
   future card kind carries seed material without being `cardMS1`, both the abort
   warning and the restore document go wrong together. That is the intended
   trade — one definition, one place to fix — but it is a single point of
   failure and is named here rather than discovered later.
6. **`restoreDocFlow`'s two error returns drop the ENTIRE inventory after a
   fully cut set** (R0 M-2). `gui/singlesig_restore.go:122` and `:127` `showError`
   and `return` *before* `restoreDocScreen`, and §4.2 rides the inventory in as
   `extra`, appended only on the success path. So on either error the operator
   gets no plate count, no seed statement and — the half F-198 is Critical for —
   **no passphrase statement**, after every plate is already on steel.
   Reachability is low (all four `md.ScriptKind` values map, and the xpub is
   device-derived) and `multisigRestoreDocFlow:103` has the identical shape
   today, which is why this plan names it rather than fixing it. **If the
   implementer can hoist the inventory ahead of the descriptor build cheaply,
   they should; if it needs restructuring, it is filed, not folded.**
7. **The template-only single-sig branch is in none of §4's edits** (R0 M-3).
   Two consequences, neither funds-losing: `singleSigEngraveCards` hard-codes
   `summary: "wallet policy descriptor"` (`gui/singlesig_engrave.go:41`), so
   §4.2's inventory will call a **keyless template plate** the wallet policy; and
   single-sig prints a restore document built from the live `xpub` for a template
   engrave, where the build path skips the document entirely
   (`gui/multisig_build.go:464`). The mk1 is in the set either way and
   `templateWarningLines` already states the recovery dependency. Named because
   §3's item inventory did not mention the template branch at all.
8. **The verify-OK notice stays as-is on a passphrase run** (R0 M-4).
   `gui/singlesig_verify.go:148` is
   `showNotice(ctx, th, "Verify OK", "The engraved bundle matches the seed.")` —
   **two** strings, both pure ASCII. (The R0 report rendered them as one string
   joined by an em dash, and the first fold copied that verbatim before the
   glyph gate caught it. Quoted from the source here. It is the
   never-describe-code-from-an-earlier-report rule, committed against a report
   that was otherwise right.)
   True and incomplete, on the most vouching sentence in the flow. It is left
   alone because §4.1's truthful label precedes it and §4.2's truthful document
   now follows it, so the operator is bracketed by corrections — but §0 rule 2
   says *every* screen, and this one is not yet compliant.
