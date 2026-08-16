# S6a R4 — independent adversarial review of `IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`

Reviewer: independent adversarial agent, 2026-08-16.
Artifact: `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
Code read: `/scratch/code/shibboleth/seedhammer` @ `b8a23bf` (working tree clean before and after; **no file was modified** — the one execution probe was injected with `go test -overlay`, whose source lives in the session scratchpad).

Question answered: **is this plan now safe to implement, and if not, what specifically breaks?**

---

## VERDICT: RED — 1 Critical, 2 Important

---

### C-1 — `verifyFailed` does not mean "these plates disagreed", and §4.7a's DISAGREED line says it does, permanently

**Where:** §4.7a (the `sawDisagreement` sticky fact and the `statusDisagreed` line), read against `gui/multisig_verify.go`.

**The defect.**

§4.7a's entire condemning path is keyed on one predicate:

```go
if res == verifyFailed {
    sawDisagreement = true       // STICKY. A later attempt cannot un-see it.
}
```

and it prints, on the durable artifact:

> `WARNING: a read-back check DISAGREED with these plates. Do NOT rely on this backup. Re-verify or re-engrave.`

Measured, `multisigVerifyFlow` returns `verifyFailed` from **five** sites, and **only two of them are a comparison that ran against this run's steel and disagreed**:

| site | what actually happened | is it a plate disagreement? |
| --- | --- | --- |
| `gui/multisig_verify.go:717-719` | `!slices.Equal(readbackMd1, engravedMd1)` — the operator presented **a different wallet's plates** | **no** |
| `gui/multisig_verify.go:721-724` | `md.ExpandWalletPolicyChunks` failed on a readback that line 717 has just proved **byte-identical** to the engraved md1 | **no** — the readback *agreed* |
| `gui/multisig_verify.go:895-897` | `deriveMultisigLeg` failed on the **re-typed seed**; no plate was ever compared | **no** |
| `gui/multisig_verify.go:960-963` | `verifyMultisigLegsPartial` disagreed | yes |
| `gui/multisig_verify.go:982-984` | `verifyMultisigLegs` disagreed | yes |

The first row is not hypothetical, and the codebase already says so in its own words, twenty lines above the return the plan reads from (`gui/multisig_verify.go:42-48`):

> `// It says the one thing the operator can act on. A generic "Verify Failed"`
> `// sends them to re-cut plates that are perfectly good; what actually happened`
> `// is that the steel in their hands belongs to a different wallet, and no`
> `// amount of re-presenting THESE plates will ever satisfy this run.`

§4.7a takes that verdict and prints **exactly the generic "Verify Failed"** the shipped code was written to avoid — on the artifact a stranger reads in five years instead of on a modal, and **sticky**, so there is no route to clear it. A later clean pass only downgrades it to `Plates VERIFIED on a repeat check, after an earlier read-back DISAGREED.` — a permanent record that this backup once failed a read-back, over steel that was never in question.

**This is a gap in the design's stated properties, not a wording slip.** P1 ("a clean pass always prints a pass line") and P2 ("a disagreement is never lost") are both constraints against *under*-warning. **Nothing in §4.7a constrains over-condemning**, and the plan's own premise says the verdict cannot bear the weight it is given:

> §4.7: "**A FAILED verify is evidence, not proof.** The comparison seed is re-typed by hand and the plates are read over NFC, so a typo or a bad read yields FAILED on good steel."

The plan states that, uses it to justify not gating the document — and then prints an unhedged imperative derived from that same weak evidence. The asymmetry is visible in the two lines side by side: the *vouching* direction is hedged twice (`VERIFIED on a repeat check` even carries "Confirm they restore before relying on this backup"), while the *condemning* direction is absolute and unclearable.

**The harm.**

- A stranger holding a complete, correct 6-plate set reads `Do NOT rely on this backup` and does not attempt the recovery. That is precisely the failure mode this document exists to prevent, named in the code the plan cites for its own wording (`gui/multisig_build_census.go:110-114`: *"that is the state in which people give up on a recovery that would have worked"*) and re-argued by the plan itself at §4.4 point 1.
- The operator's rational response to `Do NOT rely on this backup. Re-verify or re-engrave.` is to re-cut the set — hours per run, and on a full build it puts a second copy of the seed on steel for no reason.
- The trigger is an ordinary bench mistake on the path §3.2 pulled into scope: presenting a cosigner plate from another wallet during the readback. Single-sig is clean here (`singleSigVerifyFlow` has exactly one failure exit, `gui/singlesig_verify.go:144-146`, and it *is* a real comparison) — the defect lives on the two multisig paths, which is the scope increase §3.2 declared.

**Suggested remedy (UNVERIFIED — I did not resolve this against the call graph, and it is offered as a description of the gap, not as a prescription).**
The minimal-looking fix — stop treating `:717-719` as sticky — is the one to be most careful with: an operator who cannot tell their own plates apart and then abandons would print `DID NOT COMPLETE`, losing a signal that matters. A shape that keeps the stickiness while removing the false absolute would be to state the disjunction the device actually knows, e.g. *"A read-back check DISAGREED and was never cleared. This can mean the wrong plates were presented, or that a plate is wrong. Confirm these plates restore before relying on this backup."* — which also closes I-1. **Reproduce the defect, not the remedy.**

---

### I-1 — the DISAGREED line prescribes a device capability that does not exist

**Where:** §4.7a, the `statusDisagreed` line: `... Do NOT rely on this backup. Re-verify or re-engrave.`

**The defect.** The device has **no standalone bundle verify**. Measured against the program dispatch table, `gui/gui.go:1838-1875`: the rows are `qaProgram`, `engraveXpub`, `engraveBundle`, `engraveSingleSig`, `engraveMultisig`, `loadPayload`, `bip85Derive`, `unlockPayload`, `engravePassphrase`, `engraveText`, `backupWallet`. There is no verify program. `gui/plate_verify.go` is the word-plate menu and is explicitly not a bundle verify. Both bundle verifies are post-engrave offers inside a flow the operator has already left by the time the restore document renders — on single-sig there is no retry at all, and on multisig the document is reached only by pressing CONTINUE or Back out of the offer loop.

The codebase records this exact defect class as already-fixed, on the type the plan reads (`gui/multisig_verify.go:64-76`):

> `// That made "Run verify again with the remaining seeds" a prescription with no`
> `// implementation -- multisigVerifyFlow's only callers are one-shot post-engrave`
> `// offers ... and the program dispatch table carries no standalone bundle verify.`
> `// The operator's only route was to re-run the whole engrave and cut every plate`
> `// again over hours; the predictable response is to fund the wallet anyway.`

**The harm.** The one line that most needs to hand the reader an action hands them a non-existent one. Per the code's own analysis of the identical sentence, the predictable response to a remedy the device cannot perform is to do nothing and fund the wallet anyway. `re-engrave` is real; `Re-verify` is not, and offering them as alternatives implies a verify-only route exists.

**Suggested remedy (UNVERIFIED):** drop `Re-verify` and keep the two things a reader can actually do — restore-test the plates off-device (the other four lines already say `Confirm they restore before relying on this backup`), or re-engrave.

---

### I-2 — §4.8's build order opens C-1's harm at step 5 and closes it at step 7, while advertising the intermediate states as landable

**Where:** §4.8, the table header and steps 4-7.

**The defect.** Two parts.

**(a) The header claim is false for step 5, by the plan's own text.** The header says *"Each of 1-8 leaves the tree green, so the work is landable in pieces rather than all-or-nothing."* Step 5 adds §4.6's `confirmReviewScreen(ctx, th, "Plates To Cut", ...)` before the engrave — and §5.1b measures that this stops three shipped walks dead (`pumpUntil` only pumps frames, it never presses; `confirmReviewScreen` loops until Button1/Button3/Center, so each walk parks on the census for its whole frame budget and hits its `t.Fatalf`). Step 6's own cell concedes it: *"must accompany step 5, not follow it."* Step 5 as a discrete step leaves the tree **red**.

**(b) The state after steps 5+6 is exactly C-1's harm, and it is green.** Step 4 gives `restoreDocFlow` its `status` parameter with nothing to pass (the flows are wired at step 7), step 5 gives the single-sig document its **inventory**. After 5+6 the tree compiles, the suite passes, and the single-sig restore document renders

> `This backup is 3 plates: ... If any of them is missing, this backup is incomplete.`

with **no verification status line** — reachable immediately after `The read-back bundle does NOT match the seed` (`gui/singlesig_verify.go:145`). That is verbatim the state §4.7 opens by naming as the Critical (*"§4.2 is exactly what turns that document from silent into vouching"*), and §4.7c re-states it as a rule: *"a flow that never calls `buildVerifyStatusLines`, or passes `nil`, renders a document with no status line at all — silence, which §4.7 opens by declaring must never be mistakable for a pass."*

So the plan's build order says a state the plan calls a defect is *landable*. The asymmetry is telling: §4.8 noticed the must-accompany relationship for a **test-greenness** dependency (step 6) and did not apply it to a **Critical-openness** one (step 7).

**The harm.** An interrupted or partially-merged cycle leaves `main` — the tree S6 flashes — carrying a self-vouching single-sig restore document. This project's own record includes a session ending with work unpersisted, and §3's *"S6a ships as ONE cycle"* is a scope decision, not a commit-granularity one; §4.8's sentence is the only guidance an implementer has on granularity and it points the wrong way.

**Suggested remedy (UNVERIFIED):** mark steps 5-7 as one landing unit (the same treatment step 6 already gets), or move the status wiring ahead of the inventory. I did **not** resolve whether step 7 can precede step 5 — step 7's cell claims it "needs 2, 4 and 5 in place", and T11 plausibly needs step 6's census press-through rather than step 5 itself. Correct the header claim for step 5 either way.

---

## Minor (recorded, does not gate)

- **M-1 — §4.4 point 1 justifies "YOUR seed" with a sentence that never renders.** It says `"your seed"` *"is already the codebase's own word for this fact (`oneSeedPassphraseFact`, `gui/multisig_build_census.go:198`, which feeds 'Needs a passphrase: your seed')"*. Measured: `buildPassphraseInventoryLines` early-returns at `gui/multisig_build_census.go:146-148` whenever `len(seeds) < 2`, and `oneSeedPassphraseFact` returns a **one-element** slice and is always passed alone (`gui/multisig.go:362`, and §4.2's new single-sig site). The `Needs a passphrase: %s` line at `:160` is therefore unreachable from that constructor, and the `Label` is dead text. The wording choice stands on its own merits; the *justification* is a false reading of a real line — the citation gate's declared blind spot, hit again.
- **M-2 — the call-site count disagrees with itself.** Measured: `grep -rn "buildPlateInventoryLines(" --include="*.go" gui/` → **8** existing call sites (2 production: `gui/multisig.go:362`, `gui/multisig_build.go:479`; 6 test: `gui/multisig_build_prose_test.go:369,424,425`, `gui/multisig_build_perseed_passphrase_test.go:134,246,304`), 9 after §4.2's new one. §4.3's *"Call sites (all 8, measured)"* is right. **§4.8 step 3 says "the six existing call sites"** and **§5.1(a) says "The six `buildPlateInventoryLines` call sites in §4.3 — all gain a capacity argument, all `seedCapacityMany`"** — the six are the *test* sites only, and the two production sites take **different** capacities. The compiler catches a missed site, so this cannot ship a defect; it can send an implementer to §5.1(a)'s "all `seedCapacityMany`" and mis-wire `gui/multisig.go:362`, which §8.4 already names as the one call site no test guards.
- **M-3 — `DID NOT COMPLETE` discards a fact the device had.** `gui/multisig_verify.go:965-979` computes `checked` and `outstanding` and shows `multisigVerifyIncompleteText(checked, outstanding)` — which plates were proved, which were not. The durable line collapses that to *"Plate verification DID NOT COMPLETE."* This is §1.6's own pattern (a true sentence on the transient screen and not on the durable one), applied to the artifact §1.6 argues is the one that matters. The plan's "exactly one status line" is a deliberate constraint, so this is recorded rather than raised.

---

## THE ASSEMBLED DOCUMENT, MODE BY MODE

Read as it renders: `restoreDocScreen` opens at `start := 0` and draws `lines[0]` first with `doneBtn` live on that frame (`gui/singlesig_restore.go:137-160`), so `status` at slice index 0 is page 1. **Machine-verified by execution** (overlay probe, `go test -overlay`): with the longest status line prepended, `P2SH-P2WSH` is still on page 1 of the sh(wsh) multisig document at `sh2DisplaySize` (480x320) — page 1 goes `Restore Doc | <status> | Type: | P2SH-P2WSH 2-of-3 mu | ltisig (sorted) | Descriptor: | sh(wsh(sortedmulti(2`. So `TestRestoreDocNestedNameIsActuallyDrawn`'s `pumpUntil(frame, "P2SH-P2WSH", 64)` — which never presses — **survives** the change. That was the one silent test breakage I expected to find and it is not there.

Order on every page: **[status] → [wallet facts] → [plate list + "If any of them is missing"] → [seed statement] → [passphrase statement] → [seed-handling ruling]**.

**Single-sig, FULL** (cards: `ms1 secret share` / `mk1 key` / `md1 descriptor`, `gui/singlesig_engrave.go:20-45`; ms1 is one string = one plate).
Seed statement takes the one-ms1 presence arm; ruling takes `seedCapacityOne` + seed-on-plates.
- `VERIFIED` — coherent. `verifySingleSig` compares MS1 (hand-typed), MK1 and MD1 (NFC), so "each plate was read back and matched" is true of all three plates.
- `VERIFIED on a repeat check` — unreachable on this path (no retry loop). Correct.
- `NOT VERIFIED` / `DID NOT COMPLETE` — coherent.
- `DISAGREED` — coherent here, and only here: the single-sig flow's sole failure exit is a real comparison. **C-1 does not bite single-sig.** `Re-verify` (I-1) still does.

**Single-sig, WATCH-ONLY** (cards: `mk1 key` / `md1 descriptor`).
Absence arm + `seedCapacityOne` + no-seed-on-plates arm:
> `Seed: this set contains NO seed. It is watch-only: it records the wallet, but it can never spend. ... no plate in this set holds them.`
> `Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- stays in device memory until the build ends. Do not leave a mid-build machine unattended: it is still holding seed material. Power the device off when you are done.`
No contradiction: the first sentence is about the **plates**, the second about the **device**. Checked the underlying fact — `gui/singlesig.go:41` obtains the mnemonic and the `defer` scrub at `:50-54` has not run when `:136` renders, so "still holding seed material" is true. Also checked that no idle wipe contradicts "does not time out": `wipeGuard` is armed only from `unlockSecretSession` / `unlockPassphraseFlow` (`gui/unlock_session.go:88`, `gui/unlock_kdf.go:136`), never from an engrave flow, so `ctx.wipe` is nil and `armed()` is false. Sound.
Status crossings are the same as full mode.

**Multisig SUPPLY, FULL** (`gui/multisig.go:361`; `supplyEngraveTail` dedupes ms1 by minted string, so one seed → one ms1 however many slots it fills).
Presence arm fires with **one** ms1 card, and `numberedLabel` leaves it unnumbered at n=1 (`gui/multisig_engrave.go:59-68`), so *"the plate marked 'ms1 secret share'"* matches the label exactly. `seedCapacityOne` per §3.1.1 — correct, `supplyEngraveTail` takes one mnemonic.
- `VERIFIED` — true of every plate: md1 by exact chunk equality (`:717`), every mk1 by the bijection, the ms1 by hand entry.
- `DISAGREED` — **C-1 bites here.** Reachable by presenting another wallet's md1 at the readback.

**Multisig SUPPLY, WATCH-ONLY** — absence arm + `seedCapacityOne` + no-seed-on-plates. Coherent; the device does hold the seed. Same C-1 exposure.

**Multisig BUILD, FULL** (`gui/multisig_build.go:478`, `seedCapacityMany`).
Ruling reassembles the shipped string **byte for byte** — machine-verified by execution, exact `==` against the literal at `gui/multisig_build_census.go:86-90`: `true`. Several-ms1 arm's *"the plates marked 'ms1 secret share'"* is a prefix of `ms1 secret share 1 of 2` etc., per §3.1.8. Coherent with `the plates are the secret` in the ruling below it.
- `VERIFIED` — checked the multi-master case: a seed covering two slots yields one engraved ms1 (dedupe at `gui/multisig_build_tail.go:117-126`) and the verify attaches the one typed ms1 to both legs, which re-derive the same MS1. Every plate is covered. Sound.
- `DISAGREED` — **C-1 bites here.**
- `NOT VERIFIED` — I checked whether it is reachable with the document rendered but the verify never offered: `gui/multisig_build.go:444` gates on `!template && len(legs) > 0`, and `buildEngraveTail` returns `errBuildNoHeldSlot` when no slot is held (`gui/multisig_build_tail.go:131-133`), so `len(legs) > 0` always holds when reached. The two gates collapse to `!template`, matching the document's own `if !template` at `:464`. **No mode renders a document past an un-offered verify.** Sound, and §3.2's narrower claim ("wherever a document renders, it carries a status line") is the right one.

**Multisig BUILD, WATCH-ONLY** — absence arm + `seedCapacityMany` + no-seed-on-plates:
> `Seed handling: ... Every seed you entered -- this build can hold several -- stays in device memory until the build ends. Do not leave a mid-build machine unattended: it is still holding seed material.`
I specifically chased the zero-seed build (all cosigners supplied from cards), where "still holding seed material" would be flatly false. It is **unreachable** — `errBuildNoHeldSlot` refuses it. Sound.

**Multisig BUILD, TEMPLATE** — no document, no status line, no verify. Consistent.

**Cross-mode contradiction sweep:** no pair of §4.7a's line, §4.4's seed statement, §4.3's ruling and §4.2's inventory contradicts in any of the twelve renderable combinations I walked. The only false statements I found are C-1's, and they are false against the *device's evidence*, not against another line on the page.

---

## WHAT I CHECKED AND FOUND SOUND

- **§4.7a is structurally correct, and its two "structurally impossible" claims hold.** There is no ordering, no `severity()`, no `max`, no seeded accumulator, so R3 C-1 has nothing to be wrong about. `res` is declared inside the loop body at `gui/multisig.go:336` / `gui/multisig_build.go:452` and is never hoisted, so R3 C-2 cannot arise; the two hoisted variables are `status` (zero = `statusNotVerified`) and `sawDisagreement` (zero = false), both safe. Verified by tracing the real loops rather than the plan's excerpt.
- **The reachable sequence space is exactly what the table covers.** `if res != verifyIncomplete && res != verifyFailed { break }` makes `complete`, `refused` and `abandoned` terminal, so every run is `{incomplete, failed}*` followed by an optional terminal or an operator exit at the offer. The two rows the table does not spell out — `incomplete → refused` and `failed → refused` — land on `DID NOT COMPLETE` and `DISAGREED` respectively, consistent with their neighbours. P1 and P2 both hold.
- **`verifyComplete` is self-contained per attempt**, so `incomplete → complete` printing bare `VERIFIED` is **not** an overclaim: `legs`, `covered` and `typed` are locals of `multisigVerifyFlow`, freshly zeroed on every call, and `verifyMultisigLegs` runs the full bijection. The second attempt independently proves the whole obligation. This was the brief's lead hypothesis and it is clean.
- **"each plate was read back and matched" does not overclaim in any of the six modes** — traced above, including the multi-master dedupe case.
- **`DID NOT COMPLETE` does not under-state a structurally broken set.** All four `verifyRefused` sites are checked: `:670` and `:680` are defensive and **unreachable in production** (both tails refuse an empty set — `errBuildNoHeldSlot`, `errSupplyNoMatchedSlot`), and `:701` / `:794` are "you didn't present the right cards" / "that seed owes nothing here", neither of which is a verdict on the steel.
- **§4.4's two discriminants cannot disagree.** Absence uses `bundleSetCarriesASecret`, which is `!bundleShowMs1Reminder` = "no card has `kind == cardMS1`" (`gui/bundle_flow.go:457-464, 482-484`); the presence arm's count is the same ms1 card count. One question, two spellings, no divergence.
- **§4.3's byte-identity claim: VERIFIED by execution**, exact string equality against `gui/multisig_build_census.go:86-90`.
- **§4.7b's "page 1 means slice index 0": VERIFIED by execution**, and the status line does not displace the descriptor off page 1 (see the mode section).
- **All five status lines are ASCII-clean** against the body face's missing glyph set, and the longest renders and wraps within the pager.
- **§4.2's call site compiles as written in principle**: `cards` is in scope from `gui/singlesig.go:126`, `passphrase` from `:64`, and `md` / `bip32` are already imported in `gui/singlesig_restore.go`.
- **F-197's gate is correctly shaped**: `bundleEngraveResult` has exactly two values (`gui/bundle_flow.go:442-450`), and `!= bundleEngraveDone` is the right predicate.
- **§1.5's count of `bundleEngrave` call sites and their tails** matches the source.
- **Rust-primary check (§2)**: nothing in this cycle touches normative codec behaviour; it is fork-native GUI text. Agreed, exemption (b).

---

## SCOPE NOTE

I did not audit the codebase for pre-existing defects, did not review prose or markdown, and did not re-derive anything listed as mechanically verified in the brief. Every remedy above is marked **UNVERIFIED**: I reproduced the defects, not the fixes, and C-1's minimal-looking fix is one I specifically believe could reopen a Critical if applied without its own reproduction pass.
