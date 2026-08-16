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

`multisigRestoreDocFlow` (`gui/multisig_restore.go:100`) is the shape to mirror:
it takes `extra []string` and ends `restoreDocScreen(ctx, th, append(lines, extra...))`.

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
6. **Watch-only single-sig does hold a seed in device memory during the build.**
   The stand-in flagged this as an untested premise; it is now measured, not
   assumed: `gui/singlesig.go:41` obtains the mnemonic and `:90` derives from it
   **unconditionally** — `full` affects only which cards are built at `:126`. So
   the seed-handling ruling is true in both modes.
7. **The presence-arm sentence names the inventory label `ms1 secret share` as a
   PREFIX, not an exact string.** Measured: single-sig labels the card exactly
   `"ms1 secret share"` (`gui/singlesig_engrave.go:25`), multisig numbers it via
   `numberedLabel("ms1 secret share", i, n)` (`gui/multisig_engrave.go:37`) →
   `"ms1 secret share 1 of 2"`. The wording "the plate marked 'ms1 secret share'"
   is true of both. A future relabel must propagate into these lines; that
   coupling is deliberate and greppable.

---

## 4. Per-item design

### 4.1 F-198a — the mode label (`gui/singlesig.go:80`)

    Choices: []string{buildFullModeLabel(passphrase != ""), "Watch-only (keys)"},

That is the whole change. The correct string already exists and the passphrase is
already in scope at that point (`:64-74` precedes `:77`). This mirrors
`gui/multisig.go:217` exactly.

### 4.2 F-198b — the restore document gets an inventory

`restoreDocFlow` (`gui/singlesig_restore.go:119`) gains a trailing
`extra []string`, mirroring `multisigRestoreDocFlow` (`gui/multisig_restore.go:100`):

    func restoreDocFlow(ctx *Context, th *Colors, xpub string, masterFP, parentFP uint32,
        script md.ScriptKind, path bip32.Path, extra []string) {
        ...
        restoreDocScreen(ctx, th, append(lines, extra...))
    }

and the one call site (`gui/singlesig.go:136`) supplies it:

    restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path,
        buildPlateInventoryLines(cards, oneSeedPassphraseFact(passphrase != ""), seedCapacityOne))

`cards` is already in scope from `:126`. Blast radius is one production call site
and zero test call sites (§1.8).

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

`buildPlateInventoryLines(cards, seeds, capacity)` selects the arm:

- `seedCapacityMany` → **byte-identical to today's string.**
- `seedCapacityOne` → the same sentence with the clause corrected:

> Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- stays in device memory until the build ends, and on a full build the words are also on the plates as they are cut. Do not leave a mid-build machine unattended: the plates are the secret. Power the device off when you are done.

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

New function beside the passphrase one, same file:

    func buildSeedInventoryLines(cards []bundleCard, capacity seedCapacity) []string

Presence is detected with the **existing** `bundleSetCarriesASecret(cards)`
(`gui/bundle_flow.go:482`) rather than a second `any(kind == cardMS1)` walk — one
definition of "this set holds a seed", so the abort warning and the restore
document can never disagree.

**Absence arm** (watch-only, all paths):

> Seed: this set contains NO seed. It is watch-only: these plates can rebuild the wallet's addresses but can never spend. If funds must be recovered, the seed must come from somewhere else -- no plate in this set holds it.

**Presence arm, `seedCapacityOne`:**

> Seed: this set CONTAINS the seed. The plate marked 'ms1 secret share' in the inventory above is the seed backup -- treat that plate as the secret itself.

**Presence arm, `seedCapacityMany`:**

> Seed: this set CONTAINS seeds. Each plate marked 'ms1 secret share' in the inventory above is a seed backup -- treat each one as the secret itself.

Deliberately **not** claimed on the presence arm: that the seed plates *alone*
suffice to spend (true on single-sig, false on a k-of-n multisig set), and
anything about a passphrase (that is the passphrase lines' job, immediately
after).

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

## 5. Test plan

**Every test below must be shown to FAIL against the unfixed tree.** A green
suite proves nothing on its own: 9 of round 0's 17 blocking findings in S5 were
reproduced by mutating the tree and watching a green suite stay green. The
implementer reports, per test, the mutation applied and the failure message
observed — **and proves the mutated line RAN**, not merely that the edit landed.

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

**T5 is the one that carries the class.** F-197's own follow-up says it: *"A
call-site assertion alone is not enough — that is exactly what let the multisig
instance ship."* T5 must drive the real screens end to end, not assert on the
source.

**Existing tests that must be updated, not weakened:** the six
`buildPlateInventoryLines` call sites in §4.3. Any assertion deleted rather than
re-parameterised is a blocking finding.

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
4. **The capacity parameter is a new way to be wrong.** Eight call sites now
   carry an argument that no compiler can check for *correctness* — only for
   presence. §4.3's existing-test guard catches the build path; the supply path
   and single-sig are covered only by T7 and by review.
5. **`bundleSetCarriesASecret` is reused as the seed-presence detector.** If a
   future card kind carries seed material without being `cardMS1`, both the abort
   warning and the restore document go wrong together. That is the intended
   trade — one definition, one place to fix — but it is a single point of
   failure and is named here rather than discovered later.
