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

| program | entry |
| --- | --- |
| multisig build | `buildMultisigPolicyFlow` (`gui/multisig_build.go`) |
| multisig verify | `multisigVerifyFlow` (`gui/multisig_verify.go`) |
| singlesig | `gui/singlesig.go` |
| bip85 | `bip85DeriveFlow` (`gui/bip85.go`) |
| engrave bundle | `gui/bundle_flow.go` |

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
