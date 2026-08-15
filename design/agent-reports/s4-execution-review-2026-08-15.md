# S4 execution review — the slot-assignment model and the seed↔key gate

Independent adversarial execution review of stage **S4**, worktree
`/scratch/code/shibboleth/seedhammer-s4`, branch `s4-slot-gate`, commits
`6bbe6d2 · bca9133 · 27547a1 · ecb1245` on `main` at `6922b43`.

**Verdict: 0 Critical / 0 Important · 2 Minor · 3 Nit. The loop CLOSES.**

## THE ONE QUESTION

> Could a wallet be engraved whose slots do not hold the keys the operator
> believes they hold — and would S4's gate fail to say so?

**No, for every shape S4's flow can reach.** The strongest evidence is not a
test but the artifacts: I drove the walk's PASS arm myself, took the **seven
plate strings the emulator actually cut**, and decoded them cold. The engraved
mk1 holds exactly the key the engraved md1 places in the operator's own slot @0;
it does **not** hold slot @1's key (non-vacuity); the engraved ms1 seed derives
slot @0's key at the shared origin; and the ms1 is fixture master A. Evidence in
clean probe **C-11**.

The two seams where a wrong key could enter without the gate noticing were
checked directly rather than reasoned about:

- The gate compares the **same bytes** assembly writes into the policy —
  `bothSlotKey` and `cosignerFromCard` both read the key through one
  `decodeXpubBytes(card.Xpub)` (C-4).
- The gate's `Card` index and `assembleBuildPolicy`'s `gi` walk the same skip
  condition, so they agree for **any** `SelfSlot` — verified at `SelfSlot = 1`,
  which **no shipped test literal exercises** (C-10).

---

## FINDINGS

### Minor 1 — the `Card contradicts seed` screen is wired by nothing

`gui/multisig_build.go:201-204` (and the generic fallback at `:206`).

**Scenario.** A card carrying the operator's key under a contradicting master
fingerprint reaches SPEC 4.3 row 3. The gate refuses correctly, but the arm that
routes that refusal to its own screen is executed by **no test and no walk**: a
future reordering of the `errors.As` chain would silently downgrade a specific,
actionable refusal to `"Couldn't check your key against your seed."` No funds
exposure — both arms `return` and nothing is engraved.

**Evidence RAN.** Grep over the whole tree for the three dispatch strings:

```
gui/multisig_build_gate_test.go:343:  buildFingerprintContradictsMessage(fc, ...)
gui/multisig_build_gate_test.go:491:  buildFingerprintContradictsMessage(errBuildFingerprintContradicts{
gui/multisig_build.go:202:            showError(ctx, th, "Card contradicts seed",
gui/multisig_build.go:206:            showError(ctx, th, "Build Policy", "Couldn't check your key against your seed.")
```

Both test hits construct the **message**; neither reaches the **dispatch**. The
walk drives only `NEEDLE_GATE_FAIL` (`"Key does not match seed"`).

I then replicated the dispatch over errors produced by the real gate and
confirmed it is currently correct, so this is a missing regression guard and not
a live defect (`TRUE EXIT: 0`):

```
mismatch (gui.errBuildSeedKeyMismatch)          -> "Key does not match seed"
fingerprint (gui.errBuildFingerprintContradicts) -> "Card contradicts seed"
bad assignment (gui.errBuildSlotAssignment)      -> "Build Policy (generic)"
```

**Minimal fix.** One assertion in the existing gate-FAIL scrub subtest, or a
sibling subtest driving a fixture card whose `Fingerprint` is doctored while its
`Xpub` is master A's, asserting the frame reads `Card contradicts seed`.

### Minor 2 — the refusal does not name the cause its own negative control produces

`gui/multisig_build_slots.go:520-535` (`buildSeedKeyMismatchMessage`).

**Scenario.** In S4's flow the `both` slot is filled by whichever card sits at
gather position @S, and the operator chooses that ordering in the card picker.
The single most likely way to trip this gate is therefore **selecting the wrong
payload card into your own slot** — which is literally how the walk's FAIL arm
is produced (`decisions: ["skip:1","skip:2"]`, nothing else differs). The screen's
"Likely causes" list is *a mistyped passphrase, one skipped where the card used
it, a different seed, or a card from another wallet*, and its remedies are
*check the passphrase and the seed, or rewrite the payload on the host with
`me sysw pack`*. Neither names the remedy that actually applies here: build
again and put your own card at @S. An operator who follows the screen rewrites a
payload that was never wrong.

The plan's literal MUST ("a mistyped or wrong-seed passphrase; a card from a
different wallet") **is** met, which is why this is Minor and not Important; it
is also a refusal path, so no funds move either way.

**Evidence RAN.** The FAIL arm's rendered first frame, read off `shScreen()`:

```
Yousaidslot@0(payloadcard3)holdsYOURkey,butthatcard'skeyisnotwhatyourseed
derivesatm/48h/0h/0h/2h.Nothingwasengraved.Likelycauses:amistypedpassphrase,
oneskippedwherethecardusedit,adifferentseed,oracardfromanotherwallet.
ReassigningthisslotSUPPRESSESthecheckratherthanfixingit.Checkthepassphrase
andtheseed,orrewritethepayloadonthehostwith`mesyswpack`.Keydoesnotmatchseed
```

with `decisions: ["skip:1","skip:2"]` — i.e. the refusal was caused purely by a
picker choice the message never mentions.

**Minimal fix.** Add one clause to the causes and one to the remedies, e.g.
"…or the wrong card chosen for this slot" / "Build again and choose your own
card for @N". Measured headroom exists: the message is 422 chars and the fold
sits between 563 and 623 (see C-8), and the worst legal input already renders at
530.

### Nit 1 — the drawn-refusal guard has a real margin; the log says it has none

`design/agent-reports/s4-impl-log-2026-08-15.md` §4 and follow-up 5 record
"a +65-character mutation still fits … so this test pins *it draws today* and
not a margin." Measured, the margin is substantially larger and comfortably
covers the worst legal input, so follow-up 5 is filed against a number that
understates the guard.

### Nit 2 — "one different tap" is loose

Impl log §5. The arms differ in the card-1 decision **and** in the number of
downstream picker decisions (`use:1 skip:2 skip:3` vs `skip:1 skip:2`), because
the picker short-circuits when the remaining cards equal the remaining slots.
The negative-control property is unaffected — one driver, one payload, one
decisive branch — but the phrase overstates it.

### Nit 3 — the one-scrub-site invariant has one three-line hole

`gui/multisig_build.go:379-383`. `buildSeedForSlot` calls the hook and then
`reg.add`; if `reg.add` returned an error the mnemonic would be live, observed,
and **not** in the registry, so `defer reg.scrub()` could not reach it. The code
comment states the invariant absolutely ("registers before it returns, so there
is no window in which a live seed is owned by nobody"). Probed unreachable —
`reg.add` succeeded on all-zero 12/15/18/21/24-word mnemonics, and
`deriveAccountXpub` with a nil path can only fail on a seed outside 16–64 bytes,
while `bip39.MnemonicSeed` is always 64. Record the exception in the comment
rather than changing code.

---

## CLEAN PROBES — what was run, and what it showed

Every mutation was applied to a **compiling** tree, run, reverted, and the
worktree confirmed clean. Final state: `git status --porcelain` **empty**, HEAD
`ecb1245`.

**C-1 · M1 re-applied independently (the plan's named mutation).** Replaced
`origin, err := bip32.ParsePath(card.Path)` in `bothSlotKey` with
`origin := multisigSharedOrigin()`. The tree compiled (`go test` built it).
`TRUE EXIT: 1`, **both** arms red — the PROCEED arm going red is the half that
matters:

```
--- FAIL: TestGateDerivesAtTheCardsOwnOrigin/PROCEED_when_the_key_is_genuinely_derived_there
    a card declaring m/48h/0h/1h/2h, carrying the key that seed derives AT
    m/48h/0h/1h/2h, was refused: ... A gate deriving at the shared origin
    instead would fail exactly here
--- FAIL: TestGateDerivesAtTheCardsOwnOrigin/FAIL_LOUDLY_when_the_key_was_derived_at_the_shared_origin_instead
    a card declaring m/48h/0h/1h/2h while carrying the key from m/48h/0h/0h/2h
    was ACCEPTED.
```

**C-2 · M2 re-applied (delete the ONE scrub site).** Removed
`defer reg.scrub()`. `TRUE EXIT: 1`, **all four** exit classes red, each with its
own message — so no subtest is riding on another:

```
--- FAIL: .../Back_at_the_passphrase_prompt   slot-review Back: seed 0 word 11 is still 3
--- FAIL: .../the_gate_FAIL_screen            gate FAIL: seed 0 word 11 is still 3
--- FAIL: .../Back_at_the_EXPERIMENTAL_warning tail abort: seed 0 word 11 is still 3
--- FAIL: .../ctx.Done_unwind                 ctx.Done unwind: seed 0 word 11 is still 3
```

**C-3 · Every exit from the build flow enumerated.** `grep -n "^\t*return"` over
`buildMultisigPolicyFlow` gives 12 returns at lines 43–162 and 12 at 183–345;
`reg := &seedRegistry{}` / `defer reg.scrub()` sit at **179/180**. So every
return after a seed can exist runs the scrub, and every return before it precedes
the registry's creation entirely. That includes the exits S4 *adds* and the plan
names: the gate FAIL screens (`:198`, `:204`, `:207`), the slot-source review
Back (`:212`), the **plate-census Back** (`:314`, the last free moment before
steel, and the one exit class the test file does not name), the tail abort, and
the `ctx.Done` unwind. Normal completion and panic unwinding run deferred
functions too. Registry copies are safe by type: `bip39.Mnemonic` is
`[]Word` (`bip39/bip39.go:24`), so the flow-local `mnemonic` shares the backing
array the registry zeroes — an array type would have left an unscrubbed copy.

**C-4 · `findUserSlot` reuse, and the byte-identity of what is compared.**
`buildSlotGate` calls
`findUserSlot(seed.Mnemonic, seed.Passphrase, net, []md.ExpandedKey{k})`
verbatim — no reimplementation, no second comparison. The wrapper does not
change the origin (`OriginPath` comes from `bip32.ParsePath(card.Path)` and
nothing else), the account (`Account` is an input to nothing on a `both` slot),
or the strictness (`XpubPresent: true`, so the `!k.XpubPresent { continue }` arm
cannot silently skip the only key). Crucially, `bothSlotKey`
(`gui/multisig_build_slots.go:277`) and `cosignerFromCard`
(`gui/multisig_build.go:869`) both obtain the key through the same
`decodeXpubBytes(card.Xpub)`, so the gate validates precisely the bytes assembly
writes into the policy. A derive error inside `findUserSlot` is a `continue`,
which yields *no match* — i.e. it fails **closed**, into a refusal.

**C-5 · The false-positive case proceeds.** `TestGateIgnoresUnassignedCosigners`
green, in both halves (seed + two foreign cards; and a checked own-slot beside
two foreign cards). Its flow-level equivalent already exists and is green: the
scrub test's `Back at the EXPERIMENTAL warning` subtest drives master A's seed
plus one unrelated cosigner card through `Key sources` all the way to the
warning, which is the ordinary 2-of-n shape proceeding end-to-end.

**C-6 · The legitimate multi-account shape proceeds.**
`TestGateAcceptsSameSeedAtDistinctOrigins` green, including its second half —
the same seed at the **same** origin twice emits no "that is allowed" notice, so
the assertion is not passing for want of any check. The distinctness key is the
**parsed** origin, so `m/48'/0'/1'/2'` and `m/48h/0h/1h/2h` are not counted as
two origins.

**C-7 · The FAIL screen, verified by rendering rather than by reading the
string.** Two independent instruments.

*The emulator.* `emu.wasm` deleted and rebuilt (`9875322 → 9875322` bytes,
`BUILD TRUE EXIT: 0`), served on a **fresh port 8947**, staleness checked via
`typeof window.shNFC.presented === "function"`. The FAIL arm's **first** frame
carries the entire refusal, host route included — quoted in Minor 2 above.

*The Go raster test's clipping sensitivity, proven not assumed.*
`op.Drawer.draw` (`gui/op/op.go:415-427`) appends a rune to `d.text` only **after**
`clip := state.clip.Intersect(dst.Bounds()); if clip.Empty() { break }`, so
`ExtractText` excludes fully-clipped glyphs — which is what makes
`uiContains(first, …)` a genuine on-screen assertion here rather than the
"submitted for drawing" false PASS `gui/raster_test.go`'s own header warns about.

**C-8 · The fold margin, measured rather than asserted.** Padding injected into
the mismatch body, one run per size, `TRUE_EXIT` per row:

```
PAD=20  exit 0   443 chars   PAD=100 exit 0   523 chars
PAD=40  exit 0   463 chars   PAD=140 exit 0   563 chars
PAD=60  exit 0   483 chars   PAD=200 exit 1   623 chars   <- red
PAD=80  exit 0   503 chars
```

So the base message is 422 chars and the fold lies between 563 and 623. I then
rendered the **worst legal input**: `mk/mk.go:126` caps a path at
`maxPathComponents = 10`, each component at most `2147483647` plus a hardening
mark, giving a longest legal `Declared` of `"m" + 10×"/2147483647h"` = 121 chars
against today's 14. This matters because the gate runs *before*
`assembleBuildPolicy`'s shared-origin check, so a divergent-origin card really
does reach this screen. Result (`TRUE EXIT: 0`):

```
shared origin (today)                    422 chars, ink 21528 px   PASS
longest legal mk1 path, 2-digit card     530 chars, ink 27091 px   PASS
fingerprint row, 2-digit card            479 chars, ink 23639 px   PASS
```

Both refusals draw their remedy on the first frame for every input the format
permits.

**C-9 · The walk asserts rather than reaches, and both arms reproduce.** Every
term of `ok` is emulator-produced: `proven` entries come from `window.shScreen()`,
`refusal.*` from the rendered frame, `census` from `window.shToolpath.strings()`,
`presented` from `window.shNFC.presented()`. `ok` **names** the decisive needle
in each arm and requires the other arm's absent — a bare `proven.length === 9`
would be satisfied by the wrong arm. The FAIL arm is a genuine negative control:
one driver, one payload, one decisive branch (`arm === "pass" ? [...] : [...]`),
subject to Nit 2. Reproduced independently against the freshly rebuilt binary:

```
FAIL arm  ok true, 16 s   outcome "Key does not match seed"
          namesSlot/saysNothingEngraved/saysSuppresses/namesHostRoute all true
          cutCount 0   unattributed 0   presented 0   censusHeld true
PASS arm  ok true, 201 s  outcome "Where each key comes from:"
          decisions use:1 skip:2 skip:3   cardsGathered 4 > openSlots 2
          reviewSaysChecked true   censusClaim 7 == cutCount 7   unattributed 0
          digests 0de338b49038dfc5 69769ac3909067b6 b216012dccb0f13a
                  644d808bc407827e 53fa28918b15ea8e ebecd9cd212d7689
                  63b08ac45f2b8c2c
```

All seven digests match the implementer's log character for character.

**C-10 · Gate ↔ assembly index agreement at `SelfSlot > 0`.** `grep -rn
"SelfFromCard" gui/*_test.go` returns **one** literal, at `SelfSlot: 0` — where
slot equals gather index trivially, so the seam that decides THE ONE QUESTION is
untested for every other self slot. Probed at `N=3, SelfSlot=1,
SelfFromCard=true` with cards `[C@0, A@0, B@0]` (`TRUE EXIT: 0`): `sources[1]` is
`slotFromBoth` with `Card == 1`; the gate accepts; the assembled md1 holds A@0's
key at @1 and C@0/B@0 at @0/@2 in order; and reordering so a foreign card lands
at @1 refuses, naming it — `multisig build: slot @1's card key does not derive
from its seed at m/48h/0h/0h/2h`. Both loops share the identical skip condition
(`slot == p.SelfSlot && !p.SelfFromCard`) and increment, so agreement holds for
all `SelfSlot`.

**C-11 · The engraved artifacts agree with each other.** The seven plate strings
the PASS arm actually cut, decoded cold (`TRUE EXIT: 0`):

```
md1 template: {N:2 K:2 Keys:[@0 m/48h/0h/0h/2h, @1 m/48h/0h/0h/2h] Renderable:true}
mk1 path=m/48h/0h/0h/2h fp="73c5da0a"
ms1 seed master fp 73c5da0a; mk1 declares "73c5da0a"
the engraved ms1 IS fixture master A
```

with assertions that the mk1's key **equals** md1 slot @0's key, does **not**
equal slot @1's (non-vacuity), and that the ms1 seed derives slot @0's key at the
shared origin. The engrave tail derives the mk1 from the **seed** at
`multisigSharedOrigin()` independently of the card, and nothing in the walk or
the Go suite compares the two artifacts — this probe closes that gap.

**C-12 · Why C-11 cannot be luck.** `originIsShared`
(`gui/multisig_build.go:774`) compares parsed components exactly, and on the
`both` arm the self card is no longer skipped by that loop. So a card that
survives assembly declares *exactly* the shared origin, the gate derived at that
same parsed path, and `deriveMultisigLeg`'s shared-origin derivation therefore
cannot diverge from the card key the gate blessed.

**C-13 · Needle single-site claims, machine-counted** over `gui/*.go` excluding
tests:

```
[key on a card?]             -> multisig_build_slots.go
[Where each key comes from:] -> multisig_build_slots.go
[Key does not match seed]    -> multisig_build.go
[Plate Count]                -> multisig_build.go
[This engraves]              -> bip85.go  multisig_build_census.go   (2 sites)
```

All four needles unique; the rejected census needle is ambiguous exactly as
claimed. `TestWalkOkContainsNoDriverSuppliedPlateCount` globs `walk_*.js`, so it
covers `walk_s4_gate.js` without being named.

**C-14 · S4's confirmation surfaces do not carry the defect the walk found.**
The `Key sources` review and the `Plate Count` census both go through
`confirmReviewScreen` (`gui/multisig_build.go:1095`), which is a **pager**: it
lays out only what fits, tracks `shown` exactly, and draws the right-arrow nav
button precisely when `start > 0 || shown < len(lines)`. That is materially
different from `ErrorScreen`'s silent scroll, so overflow at larger `n` is
signposted rather than invisible. The PASS arm's review rendered whole on one
frame at n=2:

```
KeysourcesWhereeachkeycomesfrom:@0yours:payloadcard1,checkedagainstyourseed
for@0@1acosigner:payloadcard4,takenassupplied1slotischeckedagainstaseedon
thisdevice.Therestaretakenassupplied-thisdevicehasnothingtocheckthemagainst.
```

**C-15 · The three widened structural guards are tightenings, not relaxations.**
`TestNoVerifyFlowCanReachAPayloadSecret` moved from one identifier to a
four-identifier set that includes both `…Titled` variants while deliberately
excluding the safe `…TypedOnly` ones;
`TestEverySyswConsumptionSiteNamesAnAdmittedClass` and
`TestTheSeamPassphraseOfferReachesOnlyProgramsThatAdmitIt` moved from equality to
prefix matching, which matches strictly more. Each catches a door that the exact
match would have left open once the `…Titled` wrappers existed.
`TestTheChecksumGateIsOnForSeedEntry` moved from the whole struct literal to the
field, which is the correct granularity now that `titlePrefix` is a sibling.

**C-16 · The under-supply dead-end the `both` question could have created does
not exist.** The question is gated on `len(supply) >= p.N` pre-gather while the
outcome is classified on `len(mk1s)` post-gather; both run the same
`mk1CosignerCards` filter (`gui/multisig_build_payload.go:156-161`), and the
gather can only *add* cards, so the pre-gather guard is conservative.

**C-17 · Toolchain and final state.** `emu.wasm` rebuild proven by byte size
(`9875322 → 9875322`); served on fresh port 8947 and shut down; both walk arms
reported `shNFC.presented() === 0`. On the restored tree:

```
git status --porcelain      (empty)          HEAD ecb1245
go test ./... -count=1      TRUE EXIT: 0     51 ok, 0 FAIL
gofmt -l ./                 TRUE EXIT: 0     0 bytes of output
```

`go vet`'s 40 test-only findings were taken as the given baseline per the brief
and not re-derived.

---

## What this review did NOT cover

- S0b, S1, S2, S3, S3b and the plan itself — out of scope by the brief.
- The plan's own gate. S4's walk stands on scaffolding still carrying the S0b
  execution review's open items, and the byte-identity half of every stage gate
  is skipped in CI. This review covers S4's work and S4's gate, exactly as the
  implementer scoped it.
- S5's re-proof. Every S4 gate test is synthetic by construction because S2's
  interim foreign-origin refusal makes the card's origin and the shared origin
  indistinguishable through the flow; the plan schedules
  `TestGateStillFiresAfterOriginsDiverge` for S5, and C-1 is what keeps the
  wrapper honest until then.
