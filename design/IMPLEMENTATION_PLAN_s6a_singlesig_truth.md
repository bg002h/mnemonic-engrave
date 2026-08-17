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

### 4.7 C-1 — the restore document must say whether the plates were verified

**This is the R0 round's Critical, and the first draft made it worse rather than
introducing it.** Decision recorded verbatim in
`design/agent-reports/s6a-c1-verify-tail-decision.md`.

**The defect.** `singleSigVerifyFlow` (`gui/singlesig_verify.go:65`) returns
**nothing**. Its caller offers it in a one-shot `if sel == 0 { ... }`
(`gui/singlesig.go:130-133`) and then runs `restoreDocFlow` **unconditionally**
on the next line. Skipped, passed, or failed — same next screen. §4.2 is exactly
what turns that document from *silent* into *vouching*, so without this section
the plan ships a device that says **"The read-back bundle does NOT match the
seed"** and then prints a document headed *"This backup is 3 plates … If any of
them is missing, this backup is incomplete."*

**"Mirror multisig" is not available, because multisig has the same hole.** It
does own a 5-value `multisigVerifyResult`, and it re-offers on
incomplete-or-failed — but the loop breaks on `!ok || sel != 0`, so an operator
who reads a FAILED verify and presses **CONTINUE** falls straight through to
`multisigRestoreDocFlow`. The comment at `gui/multisig.go:323` asserting *"Only
verifyComplete falls through to the restore document"* is **false in its own
file** (R0 M-6).

**The remedy is honesty, not silence — and the incentive argument is what
decides it.** Any gate keyed on a FAILED verify makes the honest path strictly
worse than the lazy one: the operator who wants the document — the only screen
carrying the descriptor, the master fingerprint and the first addresses — learns
to **skip the verify in order to keep it**. Never make running the check the way
to lose something. A hard gate is worse still: it deletes an existing capability
for the common skip case, which is rule 1 of the operator directive.

Two supporting facts, both measured rather than assumed:

- **A FAILED verify is evidence, not proof.** The comparison seed is re-typed by
  hand and the plates are read over NFC, so a typo or a bad read yields FAILED on
  good steel.
- **The document's wallet facts are SEED-derived, not plate-derived.**
  `gui/singlesig.go:90` derives `xpub`/`masterFP`/`parentFP` from the typed
  mnemonic and `:136` hands exactly those to the document. They stay true even
  when the steel is wrong, so destroying the page along with the vouch would
  throw away the part that could still rescue the restore.

**The design.** Wherever a restore document renders at all, it carries **exactly
one** verification status line, and that line is **the first thing on the page**.
A rendered document with no status line is a defect, not a default — silence must
never be mistakable for a pass.

#### 4.7a TWO STICKY FACTS AND A SWITCH (R1 C-1, R2 C-2, R3 C-1/C-2)

**The first fold said "hold the last verdict outside the retry loop", and that
reintroduces the exact harm C-1 exists to close.** `verifyFailed` is one of the
two verdicts that **keep the loop alive** — measured, `gui/multisig.go:337` and
`gui/multisig_build.go:453` both read

    if res != verifyIncomplete && res != verifyFailed { break }

So: a comparison **DISAGREES**, the operator presses **VERIFY AGAIN**, then backs
out at the gather. The last verdict is `verifyAbandoned`, and a last-wins document
prints *"DID NOT COMPLETE"* over plates the device has already said do not match.
The remedy would have re-opened its own Critical, on the two paths §3.2 had just
pulled into scope.

**THE RULE IS TWO STICKY FACTS, NOT A SEVERITY LATTICE.**

The R1 and R2 folds built a ranked ordering, an accumulator seeded at a zero
value, a `max` over it, and a "was the final attempt clean" check. Two
independent reviewers, on two different lenses, then found the same root defect:
`not-verified` had to rank ABOVE `verified` for the table to hold, and had to be
the accumulator's seed for the zero value to be safe. **One variable cannot be
both**, and no reading of the algorithm reproduced its own table.

That structure is deleted rather than patched. It was the fifth fold in a row to
carry a defect, and the last two rounds each found a Critical in the algorithm
written to fix the previous Critical. The requirement never needed a lattice:

    status := statusNotVerified      // zero value. No attempt has run.
    sawDisagreement := false         // zero value. Nothing has disagreed.
    sawUnaccounted  := false         // zero value. Nothing was left unchecked.

    // ...inside the existing offer loop, per attempt, changing no control flow.
    // The observation is classified per RETURN PATH / ERROR / PROVENANCE (P4),
    // never per verdict and never per site -- see the 4.7e observation table.
    res, obs := <verify>             // obs is the classified observation
    switch obs {
    case obsDisagreed:               // a plate-derived comparison diverged
        sawDisagreement = true       // STICKY. A later attempt cannot un-see it.
    case obsUnaccounted:             // adverse but AMBIGUOUS -- |W| > 1
        sawUnaccounted = true        // STICKY, and outranked by obsDisagreed.
    }
    switch {
    case res == verifyComplete && sawDisagreement: status = statusVerifiedOnRetry
    case res == verifyComplete:                    status = statusVerified
    case sawDisagreement:                          status = statusDisagreed
    case sawUnaccounted:                           status = statusUnaccounted
    case obs == obsBenign:                         status = statusDidNotComplete
    default:                                       status = statusUnclassified  // P5(c)
    }

**Why this is correct where the lattice was not:**

- **No ordering exists to get wrong.** There is no `severity()`, no `max`, no
  seed. R3 C-1 is structurally impossible here.
- **Zero attempts needs no special case.** `status` is assigned *only inside the
  loop body*, so Skip leaves the zero value, which is `statusNotVerified`. R3
  C-2 — where hoisting `res` outside the loop makes its zero value
  `verifyComplete` and Skip prints VERIFIED — cannot arise, because **no verdict
  variable is hoisted at all.** Both sticky facts are of types whose zero values
  are the safe ones.
- **`sawDisagreement` is the only sticky thing, and it is sticky in the only
  direction that matters.** A disagreement is evidence; a later abandon cannot
  erase it. That was R1 C-1's entire content, kept.
- **An incomplete first attempt is NOT an anomaly**, and the earlier design
  wrongly treated it as one. The repeat-check line exists to record that a
  *disagreement* happened and was later cleared — that is the anomaly a stranger
  needs to know about. `incomplete → complete` is simply a verify finished in two
  sittings, and prints `VERIFIED`. This closes R2 C-2 by correcting the
  requirement rather than by widening an exception.

#### 4.7d SIX STATUSES, ONE PER KNOWLEDGE STATE (R4 C-1, R5 C-1, R5 I-1)

**The classifier's DOMAIN was wrong, not its shape.** Keep classify-then-map and
the sticky-facts switch; classify **what the device knows**, not what the world
is. This is evidence reporting, not diagnosis — and the sting is that **the
screens already speak this frame and only the document lacked it** —
`gui/multisig_verify.go:427-429`, verbatim: *"Either that plate was not
presented, or it is not the one this run cut."* (An earlier draft of this
sentence cited `:42-48`, which is **confident single-world diagnosis** — *"what
actually happened is that the steel in their hands belongs to a different
wallet"* — the opposite of ambiguity framing. The claim was right and the
citation was wrong.) The codebase had the idea; two
folds of this plan failed to adopt it.

**Six statuses, and the number is DERIVED — one per distinguishable knowledge
state.** If the device gains or loses a way of knowing, the set moves with it.

| # | status | what the device actually knows |
| --- | --- | --- |
| 1 | `VERIFIED` | a comparison over plate-derived bytes completed for every plate and every leg matched |
| 2 | `VERIFIED on a repeat check` | as (1), after an earlier **true** disagreement — one a clean pass does not retro-explain |
| 3 | `NOT VERIFIED` | no attempt ran; the device observed nothing |
| 4 | `DID NOT COMPLETE` | an attempt ran and ended with **no adverse observation about the plates** — seed typo, refusal, abandon, incomplete. It knows only that it does not know |
| 5 | `DISAGREED` | a comparison of plate-derived bytes against this run's intent ran and diverged, **with no ambiguity of provenance**. The only status that may condemn |
| 6 | **`PLATES UNACCOUNTED FOR`** *(new)* | an **adverse but ambiguous** observation: W holds both an operator-procedure world and a bad-plate world |

**Status 6 is where the entire Critical-generating residue lands** —
`errVerifyLegHasNoPlate`, the foreign-or-garbled md1 at `:719`, the garbled mk1
skipped at claiming, and the hand-typed ms1 divergence (R5 I-1). *That every
defect of rounds 4 and 5 falls into the one status that did not exist is the
strongest evidence this is the right set.*

Its line states **both readings and both actions**, exactly as the `:963` screen
already does:

> `Some plates could not be checked against this run. Either a plate was not presented, or it is not one this run cut. Present every plate this run engraved and check again; if this repeats, re-cut the set.`

**Mechanically:** one more sticky fact — `sawDisagreement` and `sawUnaccounted`,
with explicit switch-arm order, `DISAGREED` outranking `UNACCOUNTED`. Same shape
as before; **not** a resurrected severity lattice, because both are booleans over
observations rather than a ranking over verdicts.

**The repeat-check wording is settled by P4 too.** After a **full** clean pass
every plate this run cut was presented and matched, so an earlier *pairing*
failure is retro-explained as procedural — W collapses to "plates fine" — and
plain `VERIFIED` is earned. An earlier **true** disagreement is not
retro-explained by a later pass and keeps its note.

##### The old §4.7d, kept because its measurement still holds — `verifyFailed` MEANS TWO DIFFERENT THINGS

**Keying the condemning line on `verifyFailed` stamps an unclearable "Do NOT rely
on this backup" onto a document describing PERFECTLY GOOD STEEL.** Measured, only
**two of five** `verifyFailed` sites are a comparison against this run's plates:

| site | what it is | a comparison? |
| --- | --- | --- |
| `gui/multisig_verify.go:719` | `!slices.Equal(readbackMd1, engravedMd1)` — the plates belong to a **different wallet** | no |
| `gui/multisig_verify.go:724` | the readback would not decode | no |
| `gui/multisig_verify.go:897` | the **re-typed seed** would not derive — a typo at verify time | no |
| `gui/multisig_verify.go:963` | `verifyMultisigLegsPartial` mismatch | **yes** |
| `gui/multisig_verify.go:984` | `verifyMultisigLegs` mismatch | **yes** |

Two of the three non-comparisons are ordinary operator mistakes *at verify time*
with nothing wrong with the backup at all. **The codebase already states this
lesson** at `gui/multisig_verify.go:42-48`: *"A generic 'Verify Failed' sends them
to re-cut plates that are perfectly good."* The plan re-committed at the
**document** level the error the code warns about at the **screen** level — and
the document is the durable artifact, so it is worse there.

**SUPERSEDED AS A FIX, RETAINED AS A MEASUREMENT.** The R4 fold responded by
adding a verdict `verifyMismatch` at `:963`/`:984` only. R5 showed the **site is
not uniform either** — `errVerifyLegHasNoPlate` (`gui/multisig_verify.go:394`)
returns from inside those very sites **before any comparison runs**. So the split
is now made **per return path, per error, and per comparand provenance** (P4),
and lands in the six statuses above. The five-site table below remains true and
remains useful; it is simply not fine-grained enough to be the classifier. Non-comparison failures therefore print
`DID NOT COMPLETE` — *"Confirm they restore before relying on this backup"* —
which is true, actionable, and does not condemn.

**Control flow is UNCHANGED, and that is load-bearing.** All five sites loop
today, so both retry-loop conditions become

    if res != verifyIncomplete && res != verifyFailed && res != verifyMismatch { break }

at `gui/multisig.go:337` and `gui/multisig_build.go:453`, which preserves current
looping exactly. **Whether a foreign-policy plate SHOULD loop** — the code's own
comment says re-presenting those plates can never satisfy the run — is a real
question and is **deliberately out of scope**: it is F-199's family, and changing
retry behaviour here would be an unreviewed control-flow change riding a
document fix. Named so a reviewer does not read it as missed.

**Single-sig is unaffected**: `singleSigVerifyFlow` has no comparison split of
this kind and no retry loop. The mapping produced at build-order step 1 must
state which of its eleven exits, if any, are genuine comparisons.

#### THE THREE PROPERTIES THIS MUST SATISFY — testable, unlike the old "invariant"

The R2 fold asserted an "incentive invariant" phrased as a ranking claim, and R3
showed it false on its own ordering. Ranking is not the property anyone cares
about. These two are, and each is directly assertable:

**P1 — a clean pass always prints a pass line.** Any sequence whose final attempt
is `verifyComplete` prints `VERIFIED` or `VERIFIED on a repeat check`, never
`DID NOT COMPLETE` and never `DISAGREED`. This is what makes running the verify
never worse than skipping it.

**P2 — an ADVERSE OBSERVATION is never lost.** (Was "a disagreement is never
lost"; repaired, because "disagreement" is a world-fact and the device holds
observations.) Any sequence containing an adverse observation prints a line that
carries it — `DISAGREED`, `PLATES UNACCOUNTED FOR`, or the repeat-check line —
never bare `VERIFIED` and never `DID NOT COMPLETE`.

**P5 — THE LINE IS CONSTRUCTED FROM RECORDED EVIDENCE, AND ENUMERATION FAILURES
MAY ONLY WEAKEN IT.** This is the *enforceable* property; **P4 below is now a
THEOREM of it**, not a separate obligation.

- **(a) Positive claims are GENERATED.** Every printed claim that a check
  occurred is generated from a **recorded observation** of that check, carrying
  that observation's provenance. A claim with no generating observation **cannot
  be rendered**. *(This makes "each plate was read back" unwritable: nothing
  records reading the ms1, because nothing reads it.)*
- **(b) CLASSIFICATION AT THE POINT OF KNOWLEDGE.** Every distinction the status
  map draws is made **at the code site where its distinguishing facts are values
  in scope**, and recorded there — never reconstructed downstream from a verdict,
  an error value, or an error string.
- **(c) MONOTONE UNDER OMISSION.** An unrecognized observation maps to the line
  making the **fewest claims**, and **a test asserts that default arm is
  unreachable** from every known return path. Incompleteness may only ever
  **weaken** the printed line, never strengthen it.

**WHY THE TABLE ALONE COULD NOT DISCHARGE P4.** It needed three conjuncts at
once — complete enumeration, a correct W per row, and code able to make every
distinction the rows demand — and R7 falsified **each independently**. Worse,
**it failed OPEN**: unrowed paths inherited `DID NOT COMPLETE`, and the
unexamined success row inherited full-strength `VERIFIED`. A safety mechanism
whose failure mode is *confident output* is worse than none, because it is
trusted. P5(c) inverts that: the failure mode is now silence about what was not
checked.

P5 splits P4's completeness into **the part a tool enforces** (row coverage via a
committed return-site sweep; classifier exhaustiveness via the tested default
arm) and **the part a human still judges** (W per row) — whose failure direction
is now safe.

**P4 (now a theorem) — A LINE MAY NEVER OUTRUN THE EVIDENCE.** For every reachable outcome —
enumerated **per return path, per error, and per comparand provenance**, never
per verdict and never per site — let **W** be the set of world-states consistent
with everything the device observed on the way there. Every factual claim and
every instruction in the printed line must be true in **every** member of W. If W
contains worlds demanding different operator actions, the line must **state the
ambiguity and cover both actions**; it may not pick one world and assert it.

**P4 replaces P3, which it subsumes** (P3 is P4 restricted to the `DISAGREED`
line), and it is why the previous two Criticals kept regenerating one level down.

**THE DIAGNOSIS, and it is the reason this round changed the frame rather than
the classifier.** P1, P2 and P3 all had **world-fact** antecedents — *"a
comparison disagreed"*. The device does not have world-facts. It has
**observations**, some of which are consistent with several worlds. A world-fact
can only ever be implemented through a **proxy**, and any proxy can conflate
worlds — so the defect regenerates forever: round 4 moved the proxy from the
verdict to the site, round 5 found the site's own error set was not uniform
either. **P4 is stated over the evidence, so there is no next level to move to.**

**P4 catches all three prior findings mechanically rather than case by case** —
which is the test of a property worth having. R4's C-1: at verdict granularity, W
for `verifyFailed` contains seed-typo worlds, so *"a read-back check DISAGREED
with these plates"* is false in a member of W. R5's C-1: at the
`errVerifyLegHasNoPlate` path, W contains the forgot-a-plate world — same
failure. R5's I-1, unprompted: a hand-typed ms1 divergence has a W containing
*"one-character transcription typo, plates perfect"*.

**THE ENFORCEMENT IS AN ARTIFACT, NOT A HABIT.** The review deliverable is a
table keyed by **observation**, each row naming its W and its line, and

> **a row with |W| > 1 may not carry a line that asserts a single member of W.**

That check cannot be dodged by moving down a level, because its unit is the
world-set rather than any proxy for it.

#### ENUMERATED — every sequence, and what it prints

`S` = skip / never offered. Derived by executing the switch above, not by
argument.

| sequence | sawDisagreement | final res | prints |
| --- | --- | --- | --- |
| `S` | false | *(none)* | `NOT VERIFIED` |
| `complete` | false | complete | `VERIFIED` |
| `incomplete` then stop | false | incomplete | `DID NOT COMPLETE` |
| `refused` / `abandoned` | false | that | `DID NOT COMPLETE` |
| `incomplete` → `complete` | false | complete | `VERIFIED` |
| `mismatch` then stop | **true** | mismatch | `DISAGREED` |
| `mismatch` → `abandoned` | **true** | abandoned | `DISAGREED` |
| `mismatch` → `incomplete` | **true** | incomplete | `DISAGREED` |
| `mismatch` → `complete` | **true** | complete | `VERIFIED on a repeat check` |
| `incomplete` → `mismatch` → `complete` | **true** | complete | `VERIFIED on a repeat check` |
| **`failed`** (foreign plates / undecodable / seed typo) then stop | false | failed | **`DID NOT COMPLETE`** — never condemns (§4.7d) |
| **`failed`** → `complete` | false | complete | **`VERIFIED`** — nothing was ever disagreed with |

The retry space is unbounded, but the switch depends only on `sawDisagreement`
and the final `res`, so these twelve rows are the complete image of it — an honest
statement of coverage, where the previous table's "every sequence" header
overclaimed.
**The repeat-check state is a controller decision derived from the persisted C-1
principles, not a new operator ruling** — flagged so the next reviewer reads it
as an addition rather than as something already blessed.

| # | status | line (verbatim, ASCII only) |
| --- | --- | --- |
| 1 | `statusUnclassified` **(reserved)** | `The device observed something it could not classify. Treat these plates as unchecked and confirm they restore before relying on this backup.` |
| 2 | `statusVerified` | `Key and descriptor plates VERIFIED: read back and matched. The ms1 you typed matched this seed -- no ms1 plate was read.` |
| 3 | `statusVerifiedOnRetry` | `Key and descriptor plates VERIFIED on a repeat check. An earlier read-back disagreed; a later one matched. No ms1 plate was read.` |
| 4 | `statusNotVerified` | `No check was run on these plates. Confirm they restore before relying on this backup.` |
| 5 | `statusDidNotComplete` | `Plate verification DID NOT COMPLETE. Confirm they restore before relying on this backup.` |
| 6 | `statusUnaccounted` | `Some plates could not be checked against this run. Either a plate was not presented, or it is not one this run cut. Present every plate this run engraved and check again; if this repeats, re-cut the set.` |
| 7 | `statusDisagreed` | `WARNING: a read-back check DISAGREED with these plates. Do NOT rely on this backup: engrave a fresh set and check it before use.` |

**Seven rows: six reachable statuses plus the reserved `statusUnclassified` (§4.7c).** An
earlier draft carried five here while the prose above already argued for six —
the frame had been changed in prose only, and the artifacts a reader would build
from still described the old design.

**I-2 and I-3, from the reader lens, are folded into the table above.**
`Plates NOT VERIFIED` and `Plates VERIFIED` differed by **one leading word** at
the highest-stakes position on the page — the very ambiguity §5 forbids in the
*test suite* (three of six lines contain `VERIFIED`), shipped to a human reader
who has no compiler. Status 3 now leads with `No check was run`, sharing no token
with the pass lines. And status 2, the one PASS that mentions a disagreement, no
longer closes with the same hedge as the two UNKNOWN statuses; it now states the
resolution instead of hedging it.

#### 4.7f THE STATUS MUST SCOPE THE PAGE BENEATH IT (R6 reader I-1)

**The status line leads the document, and nothing below it was conditioned on
it.** Under `DISAGREED` or `PLATES UNACCOUNTED FOR`, the inventory, seed,
passphrase and seed-handling text was **byte-identical to the healthy case** —
including a line reading *"make sure whoever needs **this backup** can also get
the passphrase"*, reusing the exact phrase the status line had just said not to
rely on.

**That is this cycle's own defect one level up.** A page that warns in its first
line and then describes a healthy backup for the rest of its length is a document
vouching against the device's own evidence — F-198's shape at document scale
rather than sentence scale.

**The fix is one conditional line, not a rewrite of every downstream string.**
When the status is adverse — `statusDisagreed` or `statusUnaccounted` — a
**scoping line** renders immediately after it:

> `Everything below describes what this run INTENDED to engrave. Until the check above is resolved, do not assume the plates match it.`

**Why a scope line rather than conditional rewording throughout.** Rewording the
inventory, seed and passphrase blocks per status multiplies six statuses across
every downstream line, and each variant is a new string nobody has read — the
combinatorial version of the defect this cycle keeps producing. One line that
**re-frames** the rest is true under every status, needs no per-block variants,
and cannot drift out of agreement with them.

**It is P4-clean:** it asserts only that the text below states *intent*, which is
true in every world consistent with any observation, and it prescribes no action
that could be wrong.

#### 4.7e THE OBSERVATION TABLE — a REVIEW PROJECTION, not the enforcement mechanism

**Enforcement is P5; this table is how a human READS what P5 produced, and T15 tests it against that.** One row per **return path /
error / comparand provenance** — never per verdict, never per site, because both
of those proxies have already failed. `W` is the set of world-states consistent
with what the device observed.

| observation | where | W — worlds consistent with it | \|W\| | status |
| --- | --- | --- | --- | --- |
| no attempt ran | operator pressed Skip | *(nothing observed)* | — | `statusNotVerified` |
| every leg matched its plate | `gui/multisig_verify.go:987` — the success return (`:984` is `verifyFailed`, and an earlier draft miscited it here) | key and descriptor plates match; **nothing was observed about the ms1 plate** | **2** | `statusVerified`, SCOPED |
| a plate-derived comparison diverged | `verifyMultisigLegs` mismatch | this plate is not what this run cut | 1 | `statusDisagreed` |
| **no plate carries a leg's key** | `errVerifyLegHasNoPlate`, `gui/multisig_verify.go:394` | **(a)** a plate was not presented; **(b)** it is not one this run cut | **2** | `statusUnaccounted` |
| **readback md1 ≠ engraved md1** | `:719` | **(a)** foreign wallet's plates; **(b)** this run's md1, garbled in the read | **2** | `statusUnaccounted` |
| **readback will not decode** | `:724` | **(a)** a miscut plate; **(b)** an NFC read error | **2** | `statusUnaccounted` |
| **hand-typed ms1 diverges** | ms1 comparand is **operator-typed**, not plate-derived | **(a)** the plate is wrong; **(b)** a transcription typo | **2** | `statusUnaccounted` |
| re-typed seed will not derive | `:897` | the operator mistyped the seed; nothing observed about the plates | 1 | `statusDidNotComplete` |
| operator abandoned / refused | loop exit | nothing observed about the plates | 1 | `statusDidNotComplete` |

**THIS TABLE IS A REVIEW PROJECTION, NOT THE ENFORCEMENT MECHANISM (R7 / P5).**
It was demoted after R7 found three Criticals inside it: a wrong `|W|` on the
success row, two reachable paths with **no row at all**, and a distinction the
code cannot make. Enforcement is P5; this table is how a human reads what P5
produced, and its coverage is checked by a **committed return-site sweep script**
rather than by care.

**MISSING ROWS ADDED (R7 C-2), and the method failure behind them named.** The
plan performed an exhaustive per-return-site analysis for `verifyFailed` — five
sites — **because a reviewer named that verdict**, and never repeated it for any
other. `verifyIncomplete` has **three non-uniform return sites**, unexamined:

| observation | where | W | \|W\| | status |
| --- | --- | --- | --- | --- |
| readback filter drops cards | `gui/multisig_verify.go:701`; single-sig `:116` | (a) not presented; (b) unreadable plate | **2** | `statusUnaccounted` |
| plate count ≠ engraved count | `gui/multisig_verify.go:738` | (a) a plate withheld; (b) a plate this run cut is unreadable | **2** | `statusUnaccounted` |
| partial verify, all compared matched | `gui/multisig_verify.go:979` | nothing adverse observed about the plates | 1 | `statusDidNotComplete` |

**"incomplete" is removed from §4.7d's row-4 enumeration**, which had
affirmatively swept the first two in — so they printed `DID NOT COMPLETE` with no
scope line, violating P2.

**THE REMAINING SEVEN SITES — enumerated BY THE SCRIPT, not from memory.**
`./scripts/verify-returnsite-sweep.sh` reported **7 of 15 verdict return sites
unrowed** the moment it was written — *immediately after this table had been
"fixed" in response to R7*. Three review rounds found some of them; a command
found all of them in one second. That is the method failure measured, and it is
why P5(c) matters more than any row below: **being wrong about one of these must
be safe, because being complete about them is not achievable by care.**

| site | verdict | W | \|W\| | status |
| --- | --- | --- | --- | --- |
| `gui/multisig_verify.go:670` | `verifyRefused` | no expected slots — a programmer-error refusal; nothing observed about plates | 1 | `statusDidNotComplete` |
| `gui/multisig_verify.go:680` | `verifyRefused` | no engraved md1 to compare against; nothing observed about plates | 1 | `statusDidNotComplete` |
| `gui/multisig_verify.go:696` | `verifyAbandoned` | the operator declined to present plates at all | 1 | `statusDidNotComplete` |
| `gui/multisig_verify.go:794` | `verifyRefused` | `verifyFreshSlots` failed — a structural refusal before any readback | 1 | `statusDidNotComplete` |
| `gui/multisig_verify.go:938` | `verifyIncomplete` | zero legs, correctable — the seed filled no slot this run engraved | 1 | `statusDidNotComplete` |
| `gui/multisig_verify.go:940` | `verifyAbandoned` | zero legs, not correctable | 1 | `statusDidNotComplete` |
| `gui/multisig_verify.go:987` | `verifyComplete` | **the success path** — every leg matched a read-back plate; **no ms1 plate was read** | **2** | `statusVerified`, whose line is SCOPED (§4.7d) |

**`:987` is the one that matters.** It is the success site, it was unrowed, and
its `|W| = 2` is exactly R7's C-1: the device knows the key and descriptor plates
matched and knows **nothing** about the ms1 plate, because nothing reads it. The
scoped line is what makes that row P5(a)-clean.

**The rule the projection is read against:**

> **A row with `|W| > 1` may not carry a line that asserts a single member of W.**

Every `|W| = 2` row above lands in `statusUnaccounted`, whose line states **both**
readings and **both** actions. That is why the sixth status is not a nicety: it is
a line that can be true of a two-world observation.

**Not the ONLY one, and an earlier draft claimed it was** (R7 M-5). The `:987`
success row ten lines above is `|W| = 2` and takes `statusVerified` with a
**scoped** line. Ambiguity is one way to be true in every world; **scope is the
other, and the stronger one when the device knows WHICH thing it did not look
at.** `statusUnaccounted` is for the case where it does not know.

**C-3 — THE CODE CANNOT SEPARATE ROWS 3 AND 7 AS SPECIFIED, AND P5(b) IS WHY
THAT IS NOW A DESIGN INSTRUCTION RATHER THAN A CONTRADICTION.** `bundle.Verify`
returns only **untyped errors**, so a plate-derived comparison failure and a
hand-typed ms1 divergence are indistinguishable *downstream*. Collapsing them
either way reproduces R5's I-1 or loses a real `DISAGREED`. **So the distinction
is not reconstructed downstream — it is recorded at the comparator, where mk1-vs-ms1
provenance is a value in scope.** That is P5(b), and it is a change to *where*
the classification happens, not to what the table says.

**Provenance is a first-class column, not a detail.** The ms1 row is
`|W| = 2` *solely* because its comparand was typed by a human rather than read
from a plate. Two identical-looking comparison failures classify differently on
provenance alone — which is exactly what "per comparand provenance" in P4 means,
and what a per-site or per-verdict split can never express.

**"matched", not "matched the seed" (R1 I-3).** The singular contradicted §4.4's
own *"YOUR seeds"* on a multi-master build, in the same document. Dropping the
object makes the line true in every mode and costs nothing.

#### 4.7b ONE SEAM, AND IT IS THE ONE THAT ALREADY EXISTS (R1 I-1, I-2)

The first fold described two different seams for the same line. There is one:

    func buildVerifyStatusLines(v verifyStatus) []string   // exactly one line

**IT MUST LAND AT SLICE INDEX 0, AND `extra` CANNOT PUT IT THERE (R2 C-1).**

The R1 fold said "prepend it into `extra`". That does not work, and the plan had
already written down why without noticing: re-read from source rather than
recalled,

    gui/multisig_restore.go:106   restoreDocScreen(ctx, th, append(lines, extra...))

`extra` is appended **after** `lines`, so prepending *within* `extra` moves the
status line from the end of the document to the middle of it. A reviewer
measured the real shape by running the shipped supply walk: the document is
**five pages**, and `extra` begins mid-**page 4**.

And `restoreDocScreen` (`gui/singlesig_restore.go:148-160`) opens at `start := 0`
and draws `lines[start]` first, with `doneBtn` live on that same frame. So
**"page 1" means slice index 0** — nothing weaker qualifies.

So the status line is passed **separately** and placed first:

    func multisigRestoreDocFlow(ctx *Context, th *Colors, tpl md.Template,
        keys []md.ExpandedKey, status []string, extra []string) {
        ...
        restoreDocScreen(ctx, th, append(append(status, lines...), extra...))
    }

**This CORRECTS a claim the R1 fold made in §3.2.** That fold asserted "no
signature change on the multisig side at all", reasoning that `extra` already
existed. Wrong — the document needs **two** insertion points, front and back:
the status line leads, the wallet facts follow, the inventory trails. One
trailing parameter cannot express that, so `multisigRestoreDocFlow` **does**
change signature, at all THREE call sites (§4.7b). §3.2 is corrected to match.

**Single-sig** gets the same `verifyStatus` (§4.7c) and threads it into the
leading parameter §4.2 adds to `restoreDocFlow`. It has **no retry loop**, so its
two sticky facts collapse to a single assignment at each of the eleven exits —
which is why §4.7c requires that mapping be written and reviewed *first*.

#### 4.7c THE STATUS TYPE, AND WHY ITS ZERO VALUE IS "NOT VERIFIED" (R2 I-1, I-2)

The R1 fold used the identifier `verifyStatus` without ever defining it, and
claimed `multisigVerifyResult` was "a superset of what the status line
distinguishes". **It is not a superset:** it has no value for *skipped* or *never
offered*, which is the single most common outcome — the operator picks "Skip".

So the status is its own small type, distinct from the verdict:

    type verifyStatus int

    const (
        // THE ZERO VALUE IS THE SAFE ONE, DELIBERATELY. A path that forgets to
        // set a status prints "NOT VERIFIED" -- conservative and true-ish -- and
        // can never print a vouch. Mirroring multisigVerifyResult's shape would
        // have made verifyComplete = iota = 0, so the SAME omission would print
        // "Plates VERIFIED" over plates nothing ever checked. That is the whole
        // Critical, reachable by forgetting one assignment.
        statusNotVerified verifyStatus = iota   // zero value; no attempt ran
        // RESERVED, P5(c). An observation the mapping does not recognise takes
        // the fewest-claims line. T17 asserts NO reachable path produces it --
        // it is scaffolding whose emptiness is continuously tested, not a
        // seventh knowledge state.
        statusUnclassified
        statusDidNotComplete
        statusUnaccounted        // adverse but AMBIGUOUS -- 4.7d status 6
        statusDisagreed
        statusVerifiedOnRetry
        statusVerified
    )

**`statusNotVerified` stays the ZERO VALUE, not `statusUnclassified`.** A
forgotten assignment should say *no check ran* — which is what actually happened
— rather than *something unclassifiable was observed*, which asserts an
observation that never occurred. Both are safe; only one is true.

**THE ZERO VALUE PROTECTS THE VARIABLE, NOT THE DOCUMENT (R3 I).** `verifyStatus`
failing safe means a forgotten *assignment* prints `NOT VERIFIED`. It does
nothing about a forgotten *call*: the seam is `[]string`, so a flow that never
calls `buildVerifyStatusLines`, or passes `nil`, renders a document with **no
status line at all** — silence, which §4.7 opens by declaring must never be
mistakable for a pass. The type-level argument and the document-level guarantee
are different claims, and the first fold conflated them.

So the document-level guarantee is carried by a test, not by a type: **T9's
"exactly one status line" must be asserted on the RENDERED DOCUMENT of each of
the three production flows**, not only on the helper's return value. That is the
only formulation that catches a flow wired to pass `nil` — which is precisely
what `gui/multisig_nested_name_test.go:230` does today.

**Single-sig's mapping must be written, not left to the implementer (R2 I-2).**
`singleSigVerifyFlow` (`gui/singlesig_verify.go:65`) has eleven exit points and
today returns nothing. Each one maps to exactly one status, and the plan owes
that table rather than a resemblance to a type on another path — "mirrors
`multisigVerifyResult`'s shape" is not a specification when the two types do not
have the same members. The implementer produces the mapping as the **first**
step of §4.7, and it is reviewed before the rest of the section is built.

**There are THREE false-comment sites, not two (R2 I-4).** Two consecutive folds
undercounted this same defect — round 0 said one (and cited the wrong file),
round 1 said two. Pasted from the source, not from a report:

| site | what it says now | why it is false |
| --- | --- | --- |
| `gui/multisig_build.go:439` | `Only verifyComplete falls through to the restore document.` | a CONTINUE after a failure falls through too |
| `gui/multisig.go:321-322` | `Only verifyComplete falls through; a refusal or an abandon does not loop` | same, differently worded |
| `gui/multisig_verify.go:78-79` | `FOUR OUTCOMES, NOT A BOOL` … `Only verifyComplete may fall through to the restore document.` | **doubly wrong**: the type has FIVE constants, and the fall-through claim is the same falsehood on the type's own doc comment |

The third is the worst of them, because it sits on the **type definition** — the
first thing anyone reads when they go looking for what the verdicts mean. It is
also the one both previous folds missed while explicitly hunting for this exact
defect, which is why this table is pasted from `sed` output rather than
transcribed.

Neither may survive a cycle that fixes the behaviour they misdescribe. **The
citation gate resolved the wrong line happily and was right to** — it states
plainly that it proves a line *exists*, never that the interpretation is right.
That is the gate's declared blind spot doing its job by being declared.

**F-197 is untouched by this.** An *aborted engrave* still ends the program with
no verify offer and no document, because the set on the bench is incomplete.
This section governs only sets that were fully **cut**.

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
| **T9** | each of the **six** §4.7c statuses renders **its own** line, and every rendered document carries **exactly one** — over `buildVerifyStatusLines` | return the same string for two statuses; return an empty slice |
| **T10** | **the stickiness.** `mismatch` → `abandoned` prints `DISAGREED`, **not** `DID NOT COMPLETE` | implement as last-wins — precisely what the round-0 fold specified |
| **T11** | the status line is at **slice index 0** of what `restoreDocScreen` receives — asserted **through a production flow**, not on a helper | pass it via the trailing `extra` parameter, as the round-1 fold specified |
| **T12** | `incomplete` → `complete` prints **bare `VERIFIED`** (nothing ever disagreed), and `mismatch` → `complete` prints the **repeat-check** line. Neither prints `DID NOT COMPLETE` | cover only `failed → complete` — precisely what the round-1 fold specified |
| **T7c** | **the capacity WIRING**, per path: drive each of the three flows to its restore document and assert the seed-handling subject clause matches that path's capacity (build → `Every seed`; supply and single-sig → `The seed you entered`) | swap either call site's capacity argument — a mutation no compiler and no current test detects |
| **T13a** | **P1 — a clean pass always prints a pass line.** Assert the **whole line, byte-exact**, against the six-line table in §4.7d — **never `strings.Contains`**. Table-driven over §4.7a's twelve rows: every sequence whose final `res` is `verifyComplete` prints `VERIFIED` or the repeat-check line | make `sawDisagreement` non-sticky, or reorder the switch arms so a disagreement outranks the final pass |
| **T13b** | **P2 — an ADVERSE OBSERVATION is never lost.** Every sequence containing one prints `DISAGREED` or the repeat-check line, never bare `VERIFIED`, never `DID NOT COMPLETE` | drop the `sawDisagreement` assignment — R1 C-1's defect exactly |
| **T15** | **P4 — the line does not outrun the evidence.** The §4.7d **observation table** is the test fixture: one row per return path / error / comparand provenance, each naming its world-set W and its line. Assert (a) every row's line is true in every member of its W, and (b) **no row with `|W| > 1` carries a line asserting a single member** | give `errVerifyLegHasNoPlate` the `DISAGREED` line — precisely what the R4 fold specified, and R5's Critical |
| **T17** | **P5(c) — the default arm is UNREACHABLE.** No known return path reaches `statusUnclassified`; the reserved line renders for no reachable observation | add a return path and omit its classification — the test flags it the day it is written, which is exactly what `:701` and `:738` needed |
| **T18** | **P5(a) — no pass line claims a check that was not recorded.** The `statusVerified` lines name the key and descriptor plates only, and state that no ms1 plate was read | restore "each plate was read back and matched" — R7's C-1, false on every full run |
| **T19** | **P5(b) — provenance is recorded at the comparator**, not reconstructed downstream: a hand-typed ms1 divergence and a plate-derived mismatch reach the status map already distinguished | classify from `bundle.Verify`'s untyped error downstream — impossible by construction, which is the point |
| **T16** | under `statusDisagreed` and `statusUnaccounted`, the scoping line (§4.7f) renders immediately after the status line; under the four non-adverse statuses it does **not** render | drop the condition and render it always, or never — the first cries wolf on a clean backup, the second is R6's I-1 |
| **T15b** | `errVerifyLegHasNoPlate`, the `:719` foreign-or-garbled md1, and a hand-typed ms1 divergence each yield `PLATES UNACCOUNTED FOR` — **never** `DISAGREED` and **never** `DID NOT COMPLETE` | route any of them to either neighbour; the first is R5's C-1, the second its I-1 |
| **T14** | the zero value of `verifyStatus` renders `NOT VERIFIED` | reorder the constants so `statusVerified` is 0 — the mutation that makes a forgotten assignment vouch |

**T10, T12, T13a and T13b CANNOT RUN ON THE SINGLE-SIG PATH, and §5 put every
test there (R3 I-3).** Single-sig has **no retry loop** — its verify is a one-shot
`if sel == 0 { ... }` — so `failed → abandoned` and `incomplete → complete` are
unreachable by construction. Those four rows must be driven on a **multisig**
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
