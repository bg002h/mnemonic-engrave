# S5 block C — adversarial review of the multi-select picker and the per-leg verify

**Reviewer:** independent adversarial reviewer (did not author the code).
**Subject:** `4b10319` (picker + per-leg verify) and `40eb3bc` (pairing guard) on
`s5-multislot`, worktree `/scratch/code/shibboleth/wt-s5`.
**Question asked:** do they deliver what they claim, and can either produce a
FALSE GREEN on the operator's readback?

**Verdict on the headline question: NO FALSE GREEN.** I attacked the bijection
from six directions and could not construct a path to a `Verify OK` screen over
a plate that went unchecked. That property is sound and I confirmed the happy
path end to end. The block has **two Important defects in the opposite
direction**: it reports failure, or nothing at all, on honest readbacks — one of
which is a **regression against shipped behaviour**.

**Counts: 0 Critical / 2 Important / 3 Minor / 0 Nit.**

Everything below was reached by driving the **real `multisigVerifyFlow`** under
`synctest` + `runUI` (the package's own harness), not by reasoning about it. The
probe file was deleted; `git status --porcelain` is empty and
`go test ./gui/ -count=1` exits 0 after cleanup.

---

## Important

### I-1 — an honest, COMPLETE readback is reported as `Verify Failed` whenever the operator's seed accounts for more slots than were engraved. CONFIRMED. Regression.

`gui/multisig_verify.go:138-142` (the leg→plate direction of the bijection),
driven by the seed loop at `gui/multisig_verify.go:258-301`.

**The defect.** The flow derives a leg for **every slot the typed seed accounts
for in the read-back policy** (`allUserSlots`), and then requires each of those
legs to find a plate. But the set of slots a seed *fills* is not the set of
slots that were *engraved*. Where the second is a strict subset of the first,
every plate is present and correct and the verify still fails.

Coverage — the property this rewrite exists to establish — is carried entirely
by the **unclaimed-plate sweep** at `:148-152`. The leg→plate requirement at
`:140-142` adds nothing to it (success already implies
`len(legs) == len(mk1s)` with each plate verified) and is what manufactures the
false failure.

**Concrete scenario, CONFIRMED by driving the flow.** The SUPPLY path
deliberately engraves exactly one leg when the seed matches several slots, and
says so on screen — `gui/multisig.go:141-149`:

```
This key is reused at slots @0 and @1; engraving the first (@0).
```

That behaviour is deliberate, pinned by `TestFindUserSlot`'s
`"ambiguous @0 and @2 -> first-by-index + notice"` subtest, and it produces a
one-plate engrave for a seed that fills two slots. Feeding the verify exactly
what that engrave produced — **1 md1 + 1 mk1, the complete and correct set** —
and typing the same seed:

```
GATHER:       "Engrave Bundle  md1 descriptors: 1  mk1 keys: 1  Done when you have reviewed these."
FINAL SCREEN: "The read-back bundle does NOT match the seed. Check the engraved plates.  Verify Failed"
```

The operator is told their correct steel is wrong, on the one screen that exists
to tell them it is right. Pre-S5 this same input **passed**: the old body took
`findUserSlot`'s first match, derived one leg, and compared it against the one
plate.

**It is also reachable from the BUILD path.** The operator picks a strict subset
of the slots their seed fills — e.g. holds `@0` and lets `@1` arrive as a
payload cosigner card that is their own master's account-1 key. The fixture
roster in this very tree models that card (`cosignerCardRoster`'s `A@1`, and
`multisig_build_walk_test.go:31-35` calls it out by name), so it is not a
contrived shape.

**The commit's own claim is falsified by this.**
`gui/multisig.go:175-178`, added by `4b10319`, states:

> the SUPPLY path engraves exactly one leg, so the per-leg verify runs with one
> plate and one seed and behaves exactly as the single-leg verify did

It does not, in exactly the case the supply path prints a notice about.

**Minimal fix shape** (stated because it is small and does not weaken the
guarantee): drop the leg→plate requirement — skip a leg whose key no read-back
plate carries instead of erroring — and keep the unclaimed sweep as the sole
coverage rule. The `Verify Incomplete` gate at `:327-333` would then need to
count *claimed plates* rather than *legs*.

**Severity note.** Not Critical only because it fails LOUD: no funds move on a
`Verify Failed`. It is Important on this block's own published standard —
`40eb3bc`'s message: *"an operator who is told their honest plates are wrong has
no way to tell that from plates that really are."*

### I-2 — Back at the ms1 entry on the second or later seed exits the verify with NO result screen, bypassing `Verify Incomplete`. CONFIRMED.

`gui/multisig_verify.go:284-290` (`if !ok { return }`), bypassing `:327-333`.

**The defect.** In full mode the ms1 is requested once per seed, before that
seed's legs are derived. `multisigVerifyMS1Entry` returns `ok=false` on Back,
and `inputCodex32Flow` (`gui/gui.go:1033-1035`) breaks out on Back **without
drawing anything**. The flow then `return`s directly, skipping the
`len(legs) < len(readbackMk1s)` report. On the first seed this is the documented
zero-leg abandon; on the **second** seed, legs have already been verified and
plates are still outstanding.

**Concrete scenario, CONFIRMED by driving the flow.** Trace B, full mode, 3
plates read back. Type master A → ms1 A → `"1 key plate is not checked yet. Next
seed?"` → `TYPE THE NEXT SEED` → type master B → skip passphrase → **Back** at
`Type ms1`:

```
AT SECOND ms1 ENTRY:        "…0 chars  Type ms1"
returned=true   LAST SCREEN AFTER BACK: "…0 chars  Type ms1"     <- no result screen drawn
```

`multisigVerifyFlow` returned with `len(legs) == 2`, `len(readbackMk1s) == 3`,
and said nothing. On the build path the very next screen the operator sees is
the **restore document** (`gui/multisig_build.go:360-373`) — the build's final
artifact — with no statement anywhere that one of three plates was never
checked. Compare Back one screen earlier, at the seed entry, which correctly
`break`s to `Verify Incomplete` (also confirmed, below).

**The function's own text asserts this cannot happen** —
`gui/multisig_verify.go:315-318`: *"It is the only way to reach zero legs,
because every other exit above returns on its own screen"* — and the rule at
`:323-326`: *"NOT A PASS, AND NOT SILENCE."* This exit is silence with plates
outstanding.

**Severity note.** Not Critical: no `Verify OK` is shown and the operator did
press Back. Important because the `Verify Incomplete` report is precisely the
safety artifact this block added, and one button press one screen deep skips it.
The same `return` is taken by the two ms1 *rejection* arms (`:355`, `:361`),
which at least name a problem — but name the ms1's, not the unchecked plates'.

---

## Minor

### M-1 — a watch-only single-leg verify claims a secret was verified

`gui/multisig_verify.go:385-395`. `multisigVerifyOKMessage(legs<=1, full)`
returns `multisigVerifyOKBody` = *"Operator key **and secret** verified…"*
regardless of `full`, while the multi-leg branch two lines below correctly
branches on `full` and omits the secret claim in watch-only mode. Single-leg
watch-only is the dominant case (every supply-path watch-only verify), and no
ms1 is asked for or compared on that path. The constant is pre-existing and
pinned by `TestMultisigVerifyNoticeIsHonest`, so this is not a regression — but
the block had `full` in hand, used it for the sibling branch, and left the
common branch over-claiming.

### M-2 — every typed seed stays resident until the whole flow returns

`gui/multisig_verify.go:230-237`. `typed` accumulates each mnemonic and the
single deferred scrub runs only when `multisigVerifyFlow` returns — so on a
two-master verify both seeds are live through the ms1 entry, the comparator and
the success notice. Each mnemonic is dead after its own derive loop at
`:292-301` (the legs carry derived values; `deriveMultisigLeg` does not retain
it). The pre-S5 flow held exactly one seed. Wiping in-loop and clearing the
slice entry keeps the one-scrub-site design with strictly less residency. The
mechanism itself is correct: `bip39.Mnemonic` is `[]Word` (`bip39/bip39.go:24`),
so `append` shares the backing array and the deferred wipe does reach the
original — I checked, because an array type here would have scrubbed a copy.

### M-3 — the slot-naming failure errors are built and then discarded

`gui/multisig_verify.go:335-337`. `verifyMultisigLegs` returns
`errVerifyLegHasNoPlate{Slot}`, `errVerifyPlateUnclaimed{Plate}` and
`fmt.Errorf("slot @%d: %w", …)`, and the doc comments say why — *"It names the
SLOT, because that is the only thing the operator can act on"*, *"so 'which
plate do I re-cut' has an answer"*. The flow renders a fixed string that carries
none of it. The message is unchanged from shipped, so this is not a regression;
but the naming machinery this block added is unreachable by the operator, and it
is what would let someone hitting I-1 tell a mis-cut plate from a slot with no
plate.

---

## Probed and found CLEAN (no finding)

Recorded so the next reviewer does not re-derive them.

1. **The bijection cannot report success over an unchecked plate.** Traced end
   to end: each leg claims one *distinct unclaimed* index, so success implies
   `count(claimed) == len(legs)`; the sweep at `:148-152` then implies
   `len(legs) == len(mk1s)` and that `verifyMultisig` ran on every (leg, plate)
   pair. Attacked with duplicate plates, a foreign-policy plate, more plates
   than legs, more legs than plates, an undecodable plate and zero plates —
   every one produces a named failure, none a pass. An undecodable plate cannot
   even reach `mk1s`: `offerChunkedMK1` (`gui/bundle.go:195-201`) runs
   `mk.Decode` before appending, so the decode-skip in `verifyClaimPlate` is
   defensive only.
2. **No path reaches the PASS screen with plates uncovered.** PASS is gated on
   `len(legs) >= len(readbackMk1s)` (`:327`) *and* the bijection (`:335`).
3. **`Verify Incomplete` is reachable and reads unambiguously.** Driven: 3
   plates, one seed covering 2, `STOP HERE` →
   `"1 key plate is not checked yet. Next seed?  TYPE THE NEXT SEED / STOP HERE"`
   then `"Checked 2 of the 3 key plates read back. The rest were NOT verified.
   Run verify again with the remaining seeds before funding this wallet.  Verify
   Incomplete"`. It renders (no zero-pixel glyph), it names both counts, and it
   cannot be read as a pass.
4. **The picker.** Cannot produce an empty set (the first pick is mandatory),
   duplicates (`multisigRemainingSlotChoices` excludes `held`), an out-of-range
   slot (labels and slots are built over `0..n-1` and index-aligned) or a set
   larger than `n` (`for len(held) < n`). Back at all three surfaces returns
   `ok=false`; all-defaults yields `{@0}`. The tests covering these are
   non-vacuous — `TestSelfSlotPickerSelectsASet` picks `{@2, @0}` so it cannot
   pass by returning a range or the taps verbatim, and `TestSelfSlotSetReaches‐
   Params` drives the whole stage loop rather than the picker alone.
5. **Walk needles intact.** `"Which slot is your key?"` and `"key on a card?"`
   each have exactly **one** production site
   (`gui/multisig_build.go:728`, `gui/multisig_build_slots.go:530`); measured by
   grep over `*.go`/`*.js` excluding tests.
6. **`findUserSlot`'s contract is unchanged for its other callers.** All
   production callers: `gui/multisig.go:141` (supply cross-match, uses `idx`,
   `origin`, `reused`) and `gui/multisig_build_slots.go:384` (the gate, uses
   only the `matched` bool). The diff moves the loop verbatim into
   `allUserSlots` and reconstructs first-match + `reused` identically.
7. **The gather-first reordering does not leak the session into the readback.**
   `bundleGatherFlow` seeds from `ctx.syswBundleSeeds` and nils it
   (`gui/bundle_flow.go:161-167`); both producers (`gui/multisig_build.go:103`,
   `gui/multisig.go:85`) set it immediately before their own gather, so the
   verify's gather always sees nil. The `syswOffer` payload prompt lives in
   `bundleFlow` (`gui/bundle_flow.go:25-27`), a different program. The "NO
   PAYLOAD OFFER HERE" claim at `:207-214` holds. (My probes set the field
   directly — the package's own test idiom.)
8. **`extractReadbackMd1AndMk1s` cannot admit what it should refuse.** An ms1 is
   refused twice (at `classify`, `gui/bundle.go:66-72`, and again in the filter);
   a duplicate of the same plate cannot arrive because `offerChunkedMK1` dedupes
   on `chunk_set_id`, which is payload-derived (`mk/encode.go:237`,
   `csid := top20(bytecode)`); a foreign-policy plate is admitted at the filter
   and caught by the unclaimed sweep. The md1 stays exactly one.
9. **The `both`-question decision.** Left alone per the brief. Its stated
   justification checks out: `buildSlotSourceLines` does print the resulting
   per-slot assignment before anything is derived, and `buildSelfSourceLead`
   does name every held slot in the plural form.

## Positive control

Driven end to end, so the happy path is measured and not assumed — honest Trace B,
FULL mode, three plates, two masters:

```
NEXT-SEED PROMPT: "1 key plate is not checked yet. Next seed?  TYPE THE NEXT SEED / STOP HERE"
FINAL SCREEN:     "All 3 operator key plates verified, and the ms1 you typed for each seed.
                   Other cosigners' keys are taken as supplied.  Verify OK"
```

The multi-seed loop, the per-seed ms1 binding, the bijection and the scoped
success copy all work as claimed. The block's central deliverable is sound; both
Importants are on the failure side of it.

## Scope and hygiene

Out of scope per brief and not audited: the model/tail (`7910e00`), `oracle/**`,
S0-S4, the screens/prose and walk blocks. The three un-run walk drivers, the
deletion of `extractSuppliedMd1AndMk1`, and the pre-existing all-slots-held S1
refusal were disclosed by the implementer and are not re-reported. The pairing
rule and its new guard were settled by the controller and were not re-derived.

No mutation was left applied. Probe test file deleted;
`git status --porcelain` empty at `40eb3bc`; `go test ./gui/ -count=1` exit 0
after cleanup.
