# SPEC — Back must mean "back one step", not "abandon the program"

**Status: DRAFT, 2026-08-19.** Operator directive: *"We need to make the back
button act like a back button for all our programs other than sealed payload.
Going back should lose nothing."* Clarified: **Sealed Payload is a carousel
program**, so the exclusion is at program granularity.

## The defect

Back is `Button1`. Every screen wires its own `&Clickable{Button: Button1}` and
returns `false`; every CALLER does:

```go
p, ok := buildParamPickFlow(ctx, th)
if !ok {
    return          // ← exits the whole program
}
```

Back at **any** step unwinds to the carousel and discards everything entered.
There is no back-stack: `grep` for `Push`/`Pop`/`navStack` across `gui/` returns
nothing, and 21 files each wire their own Button1.

**Observed 2026-08-19 driving the emulator**, not inferred: in the verify flow,
Back while typing the ms1 did not return to the passphrase prompt. It abandoned
the verification and jumped to the Restore Doc — discarding twelve typed seed
words and a completed NFC readback for one tap.

## The rule

**Back moves one step backwards and preserves every value already entered.**
Only Back on a program's FIRST step leaves the program.

## The shape of the fix

State lives OUTSIDE the loop; the loop indexes the step:

```go
step := 0
for step >= 0 && !ctx.Done {
    switch step {
    case 0:
        p, ok = buildParamPickFlow(ctx, th)
        if !ok { return }             // first step: Back leaves the program
        step++
    case 1:
        src, ok = buildSelfSourceFlow(ctx, th, p.SelfSlots)
        if !ok { step--; continue }   // Back → step 0, `p` still set
        step++
    }
}
```

Sub-flows keep their existing `(T, bool)` signatures — **no screen changes**.
Only composition changes, which keeps the diff mechanical.

A second, smaller change is needed per screen where a sub-flow holds partial
input (the seed keyboard, the ms1 keyboard): re-entering must restore what was
typed rather than starting blank. Listed separately because it is per-screen
work, not composition.

## Scope — the carousel programs to convert

| program | entry | status |
| --- | --- | --- |
| singlesig | `gui/singlesig.go` | **DONE** |
| bip85 | `bip85DeriveFlow` (`gui/bip85.go`) | **DONE** |
| engrave bundle | `gui/bundle_flow.go` | **DONE** |
| multisig build | `buildMultisigPolicyFlow` (`gui/multisig_build.go`) | **DONE** — seed loop + prefix |
| multisig verify | `multisigVerifyFlow` (`gui/multisig_verify.go`) | **DONE** |

**All five are converted.** Sealed Payload stays excluded per the directive.

## A resume costs nothing — but only since `wordEntryOpts.resuming`

The first four conversions shipped with a defect none of their tests caught,
because every one of them asserted *control flow* — where Back lands — and none
asserted *cost*.

Measured: `inputWordsFlow` commits a word, then skips already-filled slots and
confirms on running off the end (`gui/gui.go:1069-1077`). So an operator
returning to a complete 12-word entry could only leave it **forwards** by
retyping word 1. Eleven words survived the Back and one did not.

`wordEntryOpts.resuming` draws the `done` affordance while every slot is filled,
so the count is zero. It does not weaken the rule that `done` cannot appear
where a length is already known: here the length is known **and satisfied**, and
a partial resume still never draws it.

**The general lesson: assert the cost, not just the destination.** "Back returns
to step N" is satisfied by a step N that has forgotten everything.

## Two defects that only tests could find

1. **A shadowed variable in the verify step machine.** `fresh, ferr :=
   verifyFreshSlots(...)` inside a step shadowed the leg's `fresh` — legal,
   because `ferr` is new — so the derive loop ran over a nil slice and appended
   no legs. A clean build, and a verify that silently proves nothing. Caught by
   `TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts`.
2. **A test that could not see an undrawn button.** The first version of the
   affordance test used `click()`, which dispatches by *button*, so it reached a
   handler whose `Clickable` was never drawn — the exact historical defect
   `gui.go` carries a paragraph about, where `done` was accepted in code and
   unreachable on the machine. Tapping a *position* through the drawer catches
   both halves.

## Still open — the review and picker Backs

Inside the build's gather step, the Backs in `buildPayloadReviewFlow` and
`buildCosignerPickFlow` still return. Their predecessor is the gather, and
re-entering `bundleGatherFlow` without resume would discard cards already
collected — on reader-equipped hardware, scanned ones. Wiring
`bundleGatherFlowResume` through is a separate change with its own test.

## What the multisig-build conversion established (2026-08-19)

The per-slot seed loop converted cleanly (`gatherSlotSeeds`, seedhammer
`4c7f170`) and turned out to carry a **funds-relevant** invariant that the
UI-level test could not reach: the ids are **positional**, since
`buildSlotSources` reads `seedIDs[hi]` indexed by `SelfSlots`. So a step back
must **truncate**. Left to grow, the re-entered seed shifts to the next slot —
slot 0 keeps the seed the operator just *replaced*, slot 1 inherits the
replacement, and both get engraved with a key the operator never chose for
them. A wrong key in a multisig backup, not a UX defect.

That is the general lesson for the two flows left: **in this codebase Back is
not only a UX affordance, it re-indexes arrays that later bind keys to slots.**
Convert the loop and the binding together, or not at all.

The existing walk test (`TestBuildHoldingEverySlotReachesTheSeed`) asserted the
*old* contract verbatim — *"the flow did not return after Back at the second
seed entry"* — and even once updated it did **not** cover the truncation,
because a test that backs out and leaves never re-enters. Mutation-testing is
what exposed that; the covering tests are unit tests over the extracted helper.

### The prefix Backs, once deferred, are done

They were reverted rather than half-landed on the first attempt: `chosen`,
`cosigners` and `origins` looked like they needed hoisting. They did not — they
were **already** hoisted into a `var` block, and the failed wrap had simply
started *above* it. The real seam was an entry point that resumes the parameter
pickers (`buildParamPickFlowFrom`), since `buildParamPickFlow` was already a step
machine internally. Reverting cost one attempt; landing it half-built would have
cost more.

## EXCLUDED — Sealed Payload (a carousel program)

`gui/unlock_flow.go` (`unlockTitle = "Sealed Payload"`) keeps today's behaviour
per the directive. Recorded rather than assumed: that program involves
passphrase entry and a KDF (`gui/sysw_session.go:45`), and stepping backwards
through an unlock is not the same act as stepping backwards through data entry.

## Acceptance

- Per converted program: enter data through step N, press Back, go forward again
  — the SAME values are shown. Asserted by a `gui`-package walk test, not by
  inspection.
- Back on step 0 still leaves the program.
- Sealed Payload unchanged, pinned by a test.
- **The regression test is the observed defect:** in verify, Back during ms1
  entry returns to the passphrase step with the seed still entered.

## Open question for the operator

**Does Back step backwards past a step with PHYSICAL effect?** Once plates are
engraved, Back cannot un-engrave them. Proposal: Back is disabled or relabelled
after a step with physical effect, rather than stepping into a state the machine
cannot actually return to. That is a UX ruling, not an engineering one.

## Implementation note — verify is the WRONG place to start (2026-08-19)

Attempted `multisigVerifyFlow` first, since its ms1 Back is the observed defect,
and backed out. Recorded so the next attempt does not rediscover it.

The leg loop (`for len(legs) < len(expectedSlots)`) is seed → passphrase →
derive → ms1. Making Back at ms1 return to the passphrase needs an inner loop
around those three, and that inner loop **silently retargets three existing bare
`break`s** that today mean "stop verifying":

| site | meaning today |
| --- | --- |
| the derive switch's `correctable = true; break` | seed is not in the policy — stop |
| the ms1 `correctable = correctable \|\| rejected; break` | stop, correctable only if REJECTED |
| `if len(legs) >= len(expectedSlots) { break }` | coverage complete — stop |

All three must become `break legLoop`, and `passphrase`/`slots`/`fresh` must be
hoisted out of the new loop because the leg-append below consumes them. That is
a real refactor of the most carefully-reasoned control flow in the file — whose
own comments already document **three** prior defects from exactly this class,
including "a `break` in a Go switch breaks the SWITCH, not the loop".

**So: do the simple programs first and land the pattern, then return here with
the acceptance tests written FIRST.** `bip85DeriveFlow` is the smallest and is
the right reference implementation.

Also worth knowing before touching it: this flow's Back semantics are
DELIBERATE, not accidental. It reads *"a Back at the ms1 entry ... is them
declining to type one"* and *"ONLY A REJECTION IS CORRECTABLE (B3), AND A BACK
IS NOT."* The directive overrides that reasoning, but the conversion should say
so explicitly rather than appear to have missed it.
