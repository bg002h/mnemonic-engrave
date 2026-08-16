# S5 whole-diff review — GATE VERDICT (round 0)

**Artifact:** branch `s5-multislot` @ `7da66bd7841b96f879b7f5a957c66ad16744e3d2`
(worktree `/scratch/code/shibboleth/wt-s5`, confirmed clean at verdict time:
`git status --porcelain --branch` → `## s5-multislot` only, all 8 gate-record files present).
**Gate:** mandatory post-implementation independent adversarial execution review over the
whole diff, before merge and before the branch's output is engraved on steel plates holding
real Bitcoin keys.
**Date:** 2026-08-16.

---

## VERDICT: **RED**

**3 Critical / 14 Important remain open. The branch MUST NOT merge, and nothing it produces
may be engraved, until every item below is closed and the re-review returns 0C/0I.**

| | count |
| --- | --- |
| Critical | **3** |
| Important | **14** |
| **Blocking total** | **17** |
| Minor | 13 |
| Nit | 2 |

Raw survivor count from the two-skeptic refutation pass was 22; **5 collapsed as
cross-lens duplicates** of the same defect at the same site (see the dedupe map). One
further Minor was absorbed into Important I-12. One finding was refuted and is recorded,
with reasons, at the end — **it must not be reinstated.**

Machine baseline (settled, not re-opened): `go test ./...` exit 0 / 51 `ok`, `gofmt`
clean, `go vet` exit 1 with 40 test-only findings, oracle-live PASS 7/7, emu builds.
**A green suite is what makes this report necessary, not what makes it unnecessary** —
9 of the 17 blocking findings were reproduced *by mutating the tree and watching the
suite stay green.*

---

## Dedupe map (22 survivors → 17 blocking)

| Merged into | From lenses | Site |
| --- | --- | --- |
| **C-2** | `fixture-realism` + `model-origins` | `gui/multisig_build_slots.go:377` |
| **C-3** | `funds-identity` + `fixture-realism` + `operator-failure-states` | `gui/multisig.go:204,302` |
| **I-1** | `inert-fix` + `engrave-tail` | `gui/multisig_build_tail.go:92-99` |
| **I-2** | `inert-fix` + `verify-bijection` | `gui/multisig_verify.go:596-621` |
| **I-12** | `operator-failure-states` + a Minor from the same lens | `gui/bundle_flow.go:381` |

Where lenses disagreed on severity, the **highest** severity is kept (C-3 was filed
Important by two lenses and Critical by a third; it is Critical here).

---

# CRITICAL (3)

## C-1 — "Verify Incomplete: Checked N of the M key plates this run engraved" asserts a comparison that never ran

**Defect (one sentence).** On the incomplete path the device reports a count of *plates
checked* that is actually a count of *slots re-derived from a typed seed*, so plates that
were never decoded, never xpub-matched and never stub-bound are reported to the operator as
verified.

**Site.** `gui/multisig_verify.go:655-662` (the `len(legs) < len(expectedSlots)` branch),
against `:664` — the sole call site of `verifyMultisigLegs`, which is the only function in
the file that touches a read-back plate at all.

**Triggering state (reproduced).** Trace B engraves 3 key plates for slots {0,1,2} across
masters A and B. The operator presents the md1 and three mk1 cards, but the card presented
for @0 is a **foreign mk1 from a different wallet** (an earlier generation's plate, a
mis-cut, or in the probe a single-sig mk1 at `m/44'`). They type master A — which covers @0
and @1 — then choose **STOP HERE**. Upstream checks all pass, because all three are
cardinality or provenance and none is plate identity: the readback md1 equals the engraved
md1 (`:478`), the mk1 **count** matches (`:493`), and the typed seeds account for N expected
slots. The flow then `return`s at `:662`, so `:664` is structurally unreachable, and draws:

> "Checked 2 of the 3 key plates this run engraved. The rest were NOT verified."

The operator concludes @0 and @1 are proven good and only @2 is outstanding. @0's plate
belongs to a different wallet and was never looked at. On a 3-of-4 that is a plate the
wallet needs, believed verified. The full-mode Back-out-of-ms1 path (`:609`) lands on the
same screen with the same false claim.

**Evidence.** Both skeptics independently built the probe in `cp -a` copies (frozen tree
untouched, `git status` empty, HEAD still `7da66bd`) and drove the *real* screens:
`nix develop --command go test ./gui/ -run TestProbeIncomplete… -count=1 -v` →
`FINAL SCREEN: "Checked2ofthe3keyplatesthisrunengraved…VerifyIncomplete"` with the foreign
plate among the "checked" 2. `grep -n "verifyMultisigLegs(" gui/*.go | grep -v _test.go`
returns exactly two lines: the definition at `:303` and the single call at `:664`. The
existing test `TestVerifyReportsIncompleteAfterAMidLoopRefusal`
(`gui/multisig_verify_policy_test.go:238`) reaches this screen and **passes**, asserting only
that the substring `Verify Incomplete` appears — nothing pins the count's meaning.

**Minimal fix.** Do not let the message describe plates unless plates were compared. Two
edits, both in `gui/multisig_verify.go`:

1. Before the `:655` branch, run the comparator over the legs that *were* built and the
   plates they claim: hoist a `verifyMultisigLegs(legs, readbackMk1s, readbackMd1)` call
   that is tolerant of an under-sized `legs` set (i.e. run each leg's plate match + stub
   binding, and **skip only the "every plate must be claimed" sweep** while `len(legs) <
   len(expectedSlots)`). A leg whose plate fails must produce `Verify Failed`, not
   `Verify Incomplete`.
2. Reword `:656-661` so the number is the number of *plates compared and matched*, not
   `len(legs)`, and say plainly which slots remain: e.g.
   `"Checked %d of the %d key plates this run engraved (slots %s). Slots %s were NOT
   verified…"`.

If (1) is judged too large for this cycle, the **only** acceptable alternative is to strip
the verification claim from the string entirely — "Stopped after covering N of M slots.
**No plate was checked.** Nothing here has been verified." — because the present wording is
a false GREEN on an irreversible artifact and cannot ship in any form that implies a check.

---

## C-2 — SPEC 4.3 row-5's multi-account notice is dead in the shipped flow: the gate groups by `SeedID` while the flow mints one `SeedID` *per held slot*

**Defect (one sentence).** `buildSlotGate` keys its binding map on `s.SeedID`, but
`buildMultisigPolicyFlow` calls `buildSeedForSlot` once per held slot and `reg.add()`
unconditionally, so two slots filled from **one master** carry two **different** SeedIDs,
every group has exactly one binding, and the only surface that tells the operator two of
their slots share a secret can never fire.

**Site.** `gui/multisig_build_slots.go:377` (`bound[s.SeedID]`) and the notice loop at
`:444-469` (`if len(bs) < 2 { continue }`), against `gui/multisig_build.go:196-201`
and `:554`.

**Triggering state (reproduced).** A 2-of-3 (or Trace B's 3-of-4) built on-device where the
operator holds @0 and @1 and types the **same** seed for both. The two slots get SeedIDs 0
and 1 despite sharing master fingerprint `73c5da0a`; `buildSlotSources`' account counter is
keyed on `MasterFP` and therefore *correctly* diverges them to accounts 0 and 1 — so the
build is a genuine, legitimate multi-account wallet — but `bound[0]` and `bound[1]` each hold
one binding, `len(bs) < 2` skips both, and **zero notices** are produced. The Key-sources
review then reads `@0 yours: derived from your seed for @0` / `@1 yours: derived from your
seed for @1, account 1` — two labels that read as two secrets.

**Why it is Critical, not cosmetic.** The tail correctly engraves **one** ms1 and **two**
mk1s, and the census says only `ms1 secret share: 1 plate / mk1 key 1 of 2 / mk1 key 2 of 2`.
Years later the ms1 plate is lost. The operator believes one of two independent keys survives
and expects *1 of theirs + the cosigner = 2-of-3*. In truth **both** their keys are gone
(mk1 plates are public keys and cannot sign), one key remains, and the wallet is permanently
unspendable. The **supply** path — which merely *discovers* this shape — has a live loud
notice at `gui/multisig.go:180-196`; the **build** path, which *creates* it, is silent.

**Evidence.** Probe on a byte copy of the frozen tree, building the registry exactly as
`buildMultisigPolicyFlow`'s loop does:
`registry: 2 entries, seedIDs=[0 1], masterFPs=73c5da0a/73c5da0a` /
`source @0: kind=1 seedID=0 account=0` / `source @1: kind=1 seedID=1 account=1` /
`buildSlotGate -> notices=[] err=<nil>`. **Mutation:** replacing the whole notice loop with
`return nil, nil` and running `go test ./gui/ -count=1` fails **exactly one** assertion —
`multisig_build_gate_test.go:222 "got 0 notice(s) [], want exactly 1"` — from
`TestGateAcceptsSameSeedAtDistinctOrigins`, which **hardcodes `SeedID: 0` twice**, a
registry the flow cannot construct. **The repo's own committed walk confirms it in
production:** `oracle/gaterecords/S5-trace-b.walk.json`'s `keySourcesScreen` shows
`@1 yours: derived from your seed for @1, account 1` and no notice anywhere before the
closing "No slot claims to be both a seed and a card here…" sentence that
`buildSlotSourceLines` appends notices immediately before. `cmd/emu/walk_trace_b.js:616`'s
`claims.multiAccount` is a **false friend**: it greps the review text for `"account 1"`,
proving only that the account counter diverged — it never asserts the notice fired.

**Minimal fix.** Re-key the gate's grouping on the registered seed's **master fingerprint**,
which `seedRegistry.add` already captures (`gui/multisig_build_slots.go:169-180`) and which
is precisely the identity the *sibling* function `buildEngraveTail` was already fixed to use
(it keys its ms1 dedupe on the ms1 string, with a comment at
`gui/multisig_build_tail.go:57-67` recording this exact SeedID-non-identity mistake as a
shipped Critical). Concretely, in `buildSlotGate`:

```go
bound := map[uint32][]binding{}   // was map[int][]binding, keyed on s.SeedID
var order []uint32
// in both the slotFromSeed and slotFromBoth arms, after `seed, ok := reg.at(s.SeedID)`:
key := seed.MasterFP
if _, seen := bound[key]; !seen { order = append(order, key) }
bound[key] = append(bound[key], binding{…})
// and at the notice site, `seed, _ := reg.at(id)` becomes a lookup by MasterFP
// (or carry the label alongside the binding).
```

The `slotFromSeed` arm currently does not resolve the seed beyond an existence check; it
must now bind `seed` to read `MasterFP`. **This fix was measured to work:** re-keying on
`MasterFP` makes the notice fire for the real Trace B shape and the whole gui suite stays
green (`ok seedhammer.com/gui 85.021s`).

**Mandatory accompanying test.** `TestGateAcceptsSameSeedAtDistinctOrigins` must be rebuilt
so its `sources` come from `buildSlotSources` over a registry built by **two `reg.add()`
calls with the same mnemonic** — i.e. the shape the flow actually produces — and must assert
the notice fires. The current hand-built fixture is itself a blocking defect (see the
Defect-class section) and must not survive the fold.

---

## C-3 — The SUPPLY path labels a passphrase build "Full (seed + keys)" and its restore document never mentions a passphrase

**Defect (one sentence).** A supply-path run in which the operator entered a BIP-39
passphrase engraves an ms1 that encodes the words **only**, then labels the choice
"Full (seed + keys)" and prints a restore document with `extra == nil`, so no screen and no
plate anywhere states that a required spending factor is missing.

**Site.** `gui/multisig.go:141-150` (passphrase taken via `syswPassphraseFlow`), `:204`
(hard-coded `Choices: []string{"Full (seed + keys)", "Watch-only (keys)"}`), `:302`
(`multisigRestoreDocFlow(ctx, th, tpl, keys, nil)`).

**Triggering state (reproduced live).** Front-door picker → Multisig → **"Supply policy
(md1)"** (the first row) → scan a coordinator's md1 → type the seed → "Add passphrase" →
type one → pick row 0, `Full (seed + keys)` → `supplyEngraveTail` cuts ms1 + one mk1 per
matched slot + md1. The passphrase is a live derivation input on this path (it flows
`multisig.go:141` → `allUserSlots` at `:162` → `supplyEngraveTail` → `deriveMultisigLeg` →
`deriveAccountXpub`, `gui/multisig_derive.go:37`), so the plates are *correct* — and the
backup is *incomplete*, silently. Five years later the reader holds a complete-looking
"Full" set that does not reach the money, with nothing on the device or the steel that a
third spending factor was ever in play.

**Why Critical.** This is F-132's shape — a backup that is both wrong and trusted — and S5
**declares it must not ship**, in this very diff, in the code's own words at
`gui/multisig_build_census.go:126-128` ("…is a LIE for one with a passphrase") and
`gui/multisig_build_slots.go:219`. S5 built `buildFullModeLabel` and
`buildPassphraseInventoryLines` for exactly this harm and wired them to the **build** path
only. The result is an asymmetry S5 itself created: the path that tells the truth is behind
the mandatory EXPERIMENTAL warning, and the hardware-validated, front-door path is the one
that lies. The merge is immediately followed by engraving real keys.

**Evidence.** Live UI walk in a clean archive of `7da66bd`, driving the real
`supplyMultisigPolicyFlow` with a genuine passphrase-bearing payload record and an md1 whose
slot key was derived **with** that passphrase (verified: `allUserSlots(m,"abandon about",…)
== [0]` and `allUserSlots(m,"",…) == []`, so the fixture only matches when the passphrase is
supplied). Captured screen: `"What to engrave?Full(seed+keys)Watch-only(keys)EngraveMode"`
— zero occurrences of "passphrase". Statically: `grep -n "buildFullModeLabel|
buildPlateInventoryLines" gui/*.go | grep -v _test` → **one caller each**, both in
`gui/multisig_build.go` (`:340`, `:415`); `grep -c -i passphrase gui/multisig_restore.go` →
**0**; `multisigRestoreLines(tpl, keys)` has no passphrase parameter at all, so it is
structurally incapable of naming one. `buildFullModeLabel(true)` was executed and returns
`"Full (seed + keys, NOT passphrase)"` — the correct string exists and is unreachable from
this flow. **No test covers it:** the only supply-path UI driver, `s5DriveSupply`, always
clicks `Skip` at the passphrase prompt.

**Minimal fix — two lines**, both in `gui/multisig.go`:

```go
// :204
Choices: []string{buildFullModeLabel(passphrase != ""), "Watch-only (keys)"},
// :302
multisigRestoreDocFlow(ctx, th, tpl, keys, buildPlateInventoryLines(cardsOut, passphrase != ""))
```

`cardsOut` is already in lexical scope at `:302` (bound at `:217-219` from
`supplyEngraveTail`), so the restore-doc call site's own comment — "the supply path has no
set of its own and passes nil" — is factually wrong and should be deleted with the change.
Note this second edit also closes Minor M-15 (the supply restore doc carries no plate
inventory at all), which F-188 made bite by letting this path cut several plates.

**Adjacent, out of scope, file it:** `gui/singlesig.go:80` carries the same hard-coded
literal.

---

# IMPORTANT (14)

## I-1 — A `both` slot's engraved origin is unpinned: swapping the card's declared path for `derivedSlotOrigin` leaves the whole gui+oracle suite green

**Defect.** Nothing in the suite distinguishes "a `both` slot's mk1 is engraved at the
**card's** declared origin" (correct, SPEC M-B) from "…at `derivedSlotOrigin(script,
s.Account)`" (wrong), so the correctness of the shipped line is unobserved.

**Site.** `gui/multisig_build_tail.go:92-99` (the `slotFromBoth` arm, `origin = o`).

**Triggering state (reproduced).** With the **delivered** payload
(`cmd/emu/sysw_cards_payload.bin`): wsh 2-of-2, operator holds @0, answers "my key is on a
card", asserts roster card 3 = A@1 declaring `m/48h/0h/1h/2h`. The seed↔key gate **passes**
(measured `gate err = <nil>`, because S5 deleted S2's foreign-origin refusal and the gate
derives at the card's own origin), and the assembled policy declares `m/48h/0h/1h/2h` at @0.
`buildSlotSources` never sets `Account` on a `both` slot, so it is always 0 and
`derivedSlotOrigin(wsh,0)` is always `m/48h/0h/0h/2h`. Under the mutation the same input
engraves a plate declaring `m/48h/0h/0h/2h` carrying `xpub6DkFAXWQ2dHxq2va` while the
card/policy hold `xpub6DzhyrnFFYQ1HimD` — a key plate asserting membership of a wallet
whose @0 is a different key, at a path the policy never declares. The only downstream check
is behind a `Verify now / Skip` picker (`gui/multisig_build.go:396-401`), so on **Skip** the
wrong plate is the operator's only record of a slot they can no longer prove.

**Evidence.** Both skeptics minted private copies (`git archive 7da66bd | tar -x`) and ran
it independently. Baseline `go test ./gui/ ./oracle/ -count=1` → exit 0. With `origin = o`
replaced by `origin = derivedSlotOrigin(script, s.Account)` → **exit 0, zero failing tests**
(one run: `--- PASS` × 810, `--- FAIL` × 0, `ok gui 156.314s`, `ok oracle 0.211s`). Matches
the 28-mutation campaign's `M14-build-both-uses-derived-origin SURVIVED(green)`. The cause
is measurable: `grep -rn 'buildEngraveTail(' gui/*_test.go` shows **all 7** call sites use
`s5TraceB(t)`, whose 3 held slots are all `slotFromSeed`; every `slotFromBoth` fixture in the
tree reaches `buildSlotGate` and **never** the tail. Contrast: the `derived` arm **is**
pinned (M12 killed by 3 tests) and the §4.1 duplicate guard the tail relies on **is** pinned
(M11 killed by 9).

**Minimal fix.** Add one test that pins the `both` arm end-to-end, using the gate test file's
already-proven fixture. Both skeptics wrote and ran a version of it; land it as
`gui/multisig_build_tail_both_test.go`:

- build `reg := gateRegistry(t, fixtureMasterA, "")` and `card := gateCard(t, 3)` (master A
  at `m/48'/0'/1'/2'`, already proven to PROCEED `buildSlotGate` by
  `TestGateDerivesAtTheCardsOwnOrigin`);
- assemble a 2-of-2 wsh policy via `assembleBuildPolicy`, assert `buildSlotGate` returns nil
  (reachability), call `buildEngraveTail`;
- `mk.Decode` the @0 leg's mk1 and assert **both** that its `OriginPath` equals the card's
  declared path *and* that its public key bytes equal the assembled policy's slot-0
  `ExpandedKey`.

Measured: PASS on the frozen tree, FAIL under the mutation with
`"the @0 leg's mk1 carries a key the policy does NOT hold at @0"`. **No production change is
required** — the shipped line is correct; what is missing is the thing that holds it there.

---

## I-2 — The whole `full` half of `multisigVerifyFlow` is executed by no test; two mutations survive, including one that would fail every full-mode verify

**Defect.** No test drives `multisigVerifyFlow` with `full=true`, so the ms1 half of the
verify — the per-seed ms1 capture, its binding to that seed's legs, the ms1-entry Back
handling and the full-mode success string — is entirely unobserved, and two independent
mutations to it leave the suite green.

**Site.** `gui/multisig_verify.go:596-612` (the `if full` block), `:621`
(`MS1Readback: ms1Readback`), `:677-694` (`multisigVerifyMS1Entry`, whole body),
`:718-721` (the full-mode multi-leg success string).

**Triggering state (reproduced, two mutations).**

- **M25 — the ms1 readback is discarded.** Blanking the operator-typed ms1 so an empty
  string lands in the leg leaves the suite **GREEN**, even though in full mode the derived
  leg carries a non-empty MS1 and an empty readback is a hard `ms1 presence mismatch` at
  `bundle/verify.go:77-79` — i.e. **every full-mode verify would fail and nothing notices.**
  (Precision note: the literal edit at `:621` does not compile — Go rejects `ms1Readback` as
  declared-and-not-used once its only read is removed. The compiling equivalent, blanking at
  the assignment site, was run by both skeptics: `ok seedhammer.com/gui 57.094s` /
  `61.104s`, GREEN. The substance stands; the finding's stated diff text is imprecise about
  which line must carry the mutation.)
- **M19 — the I-2 fix reverts silently.** Changing the ms1-entry `break` at `:609` back to
  `return` leaves the suite **GREEN** (`ok seedhammer.com/gui 68.635s` / `84.427s`),
  reintroducing exactly what its own comment describes: Trace B full mode, 3 plates / 2
  masters; the operator verifies master A, is offered the next seed, types master B, presses
  **Back** at "Type ms1" — the flow exits silently with no "Verify Incomplete", and on the
  build path the next screen is the restore document, with 2 of 3 plates checked and no way
  for the operator to know. Its **sibling** `break` at `:590` **is** pinned (M20 killed by
  `TestVerifyReportsIncompleteAfterAMidLoopRefusal`); the two are described as the same fix
  for the same reason in the same function, and only one is held.

**Evidence.** Coverage, measured with a cold `GOCACHE`:
`go test ./gui/ -count=1 -coverprofile=…` → `ok, 83.4%`; then
`grep multisig_verify.go gui.cov | awk -F'[:,. ]+' '$NF==0'` shows **0 executions** for
`597.11,599.11`, `599.11,609.10`, `611.4,611.19` (the whole `if full` block including the
I-2 break), `677.70..693.25` (the entire body of `multisigVerifyMS1Entry`) and
`718.10,721.3` (the full-mode multi-leg success string). Cause:
`grep -rn 'multisigVerifyFlow(ctx' gui/` — **all five** test call sites pass the literal
`false` (`multisig_verify_policy_test.go:177`, `multisig_verify_flow_test.go:114/220/246`,
`multisig_supply_multislot_test.go:271`), including `s5DriveVerifyTwoSeeds`, the only
multi-seed driver — so the *multi-seed × full-mode* cell, **Trace B's shipping shape**, is
untested end to end. `grep -rn multisigVerifyMS1Entry gui/` → one reference outside its own
definition: the production call at `:598`. `TestVerifyCoversEveryMastersSecret` — the only
test of the load-bearing per-seed binding — hand-builds `verifyLeg` values via
`s5ReDerivedLegs` and calls `verifyMultisigLegs` directly, which proves the **comparator**
and not the **assignment** at `:621`. The same cause leaves `:519-520`, `:528-531`
(Add-passphrase), `:563-566` (the F-191 arm's wiring), `:571-573` and `:647-649` uncovered:
every driver clicks Skip.

**Dissent, recorded.** One skeptic on the second half of this pair voted `stands=false`,
arguing the flagship *illustrative* scenario (a hoist of `ms1Readback` above the loop
letting a mis-cut master-B plate pass) does not reproduce, because the slot-freshness gate
returns `fresh=[]` first and, failing that, `bundle.Verify` catches the entropy mismatch
(`slot @2: verify: ms1 entropy mismatch`). **That dissent is accepted on the illustration
and does not change the verdict**, because (a) the finding survives on the other skeptic's
vote and on its merged sibling, and (b) the *measured* facts — two green mutations and a
zero-execution `if full` block — are what block, not the narrative. Implementers should
therefore **not** spend time on the hoist scenario; fix the coverage gap, which is real and
independently confirmed.

**Minimal fix.** Add one flow-level driver that runs `multisigVerifyFlow` with `full=true`
over Trace B's two-master shape, and assert three things the mutations would break:

1. type master A + its ms1, type master B + its ms1 → the final screen is the **full-mode**
   multi-leg success string (kills the `:718-721` gap and M25, since a blanked readback
   fails `bundle/verify.go:77-79`);
2. type master A + its ms1, then press **Back** at master B's "Type ms1" → the final screen
   is **`Verify Incomplete`**, not the restore document (kills M19, mirroring the existing
   `:590` test);
3. type master A + **master B's** ms1 → `Verify Failed` (pins the per-seed binding through
   the flow, not just through `verifyMultisigLegs`).

No production change is required for (1) and (2). See I-3 for (3)'s message.

---

## I-3 — The verify's failure screen discards every diagnosis, including the slot the tests assert it names

**Defect.** `verifyMultisigLegs`' error is bound solely for a nil check and thrown away, so
an ms1 typo, a presence mismatch and a missing plate for slot @2 all collapse into one
generic sentence that tells the operator to distrust perfectly good steel.

**Site.** `gui/multisig_verify.go:664-667`.

**Triggering state.** Trace B, full mode, 3 key plates + 2 seed plates on the bench. The
operator re-types master A correctly, then mistypes the hand-typed 48-character ms1 in a way
that still decodes (a character swap, or the *other* master's ms1 — the presence and
language arms need no checksum luck at all). `bundle.Verify` returns
`verify: ms1 entropy mismatch` (`bundle/verify.go:96`, wrapped as `slot @%d: %w` at
`multisig_verify.go:315`); the flow discards it and shows **"Verify Failed — The read-back
bundle does NOT match the seed. Check the engraved plates."** The steel is perfect and the
device has just told the operator to distrust it, while holding a string that says the ms1
is the problem. The same collapse hits `errVerifyLegHasNoPlate{Slot}`, whose entire reason
to carry a slot number (`:186-188`) is that it is "the only thing the operator can act on",
and which **two tests assert on** with the rationale "so the operator cannot tell WHICH
plate to re-cut" (`multisig_verify_legs_test.go:153-157`, `:464-467`) — assertions no
production path ever exercises.

**Evidence.** `err` is bound and never read past the nil check (read directly). `showError`
(`gui/slip39_polish.go:36`) displays the literal string with no side channel and no logging.
`git show main:gui/multisig_verify.go` carries the identical string at `:126`, from an era
when exactly **one** plate existed and "the engraved plates" was unambiguous; **S5 makes a
run cut 3-9.** Coverage: `multisig_verify.go:212.49,215.2 1 0` —
`errVerifyPlateUnclaimed.Error()` has never been called by anything, test or product. The
one flow-level mismatch test, `TestVerifyStillFailsWhenTheENGRAVEDPlateIsWrong`
(`multisig_verify_flow_test.go:181-198`), passes while asserting only
`uiContains(last, "Verify Failed")`. This is F-191's class at a new site: the comparator
rather than the slot match.

**Minimal fix.** At `:664-667`, render the diagnosis:

```go
if err := verifyMultisigLegs(legs, readbackMk1s, readbackMd1); err != nil {
    showError(ctx, th, "Verify Failed", multisigVerifyFailureText(err))
    return
}
```

with a small `multisigVerifyFailureText(err error) string` that type-switches on
`errVerifyLegHasNoPlate` / `errVerifyPlateUnclaimed` (naming the slot / the plate) and
otherwise appends the wrapped comparator message to the generic lead — the ms1 arms of
`bundle.Verify` already say "ms1 …", which is the one word that stops an operator scrapping
good plates. Assert the slot number reaches the **screen** in the I-2 driver (3), not just
the error object.

---

## I-4 — "Verify Incomplete" instructs the operator to run verify again, and nothing on the device can run it again

**Defect.** The incomplete-verify screen prescribes a remedy — "Run verify again with the
remaining seeds before funding this wallet" — that has no implementation anywhere on the
device.

**Site.** `gui/multisig_verify.go:655-662` (the instruction), against
`gui/multisig_build.go:396-401`, `gui/multisig.go:296-298` and `gui/gui.go:1840-1870`.

**Triggering state.** Trace B build, 3 plates for slots {0,1,2}. The operator types master A,
covering @0 and @1; the flow offers "TYPE THE NEXT SEED"; they fumble one word of master B or
type master C by mistake (`TestVerifyReportsIncompleteAfterAMidLoopRefusal` drives exactly
this and PASSES), the refusal modal shows, the loop `break`s, and the screen prescribes a
re-run. Dismiss → the restore document → the program ends. `multisigVerifyFlow`'s only two
callers are one-shot post-engrave offers (a bare `if sel == 0 { … }`, no loop), and the
program dispatch table — `qaProgram, engraveXpub, engraveBundle, engraveSingleSig,
engraveMultisig, loadPayload, bip85Derive, unlockPayload, engravePassphrase, engraveText,
backupWallet` — contains **no standalone bundle verify**; `gui/plate_verify.go:22-25` states
outright that the bundle verifies are untouched by the word-plate menu. The operator's only
route is to re-run the whole engrave and cut three more plates over hours. The predictable
response is to fund the wallet anyway. The same trap sits under "Verify Failed" (I-3): one
mistyped character ends the only verify the run will ever get.

**Minimal fix — choose one, and it must be the first for a funds-bearing device.**

**(a) Preferred: make the remedy exist.** Wrap the verify offer in a retry loop at both call
sites, so an Incomplete or Failed verify re-offers "Verify again / Continue" instead of
falling through to the restore document:

```go
for {
    sel, ok := verifyChoice.Choose(ctx, th)
    if !ok || sel != 0 { break }
    if multisigVerifyFlow(ctx, th, full, engravedSlots, md1) == verifyComplete { break }
}
```

This requires giving `multisigVerifyFlow` a return value, which **I-12 requires anyway** —
land the two together.

**(b) Fallback, only if (a) slips:** reword `:656-661` to state the truth ("This run's verify
is over. To check the rest, re-run **Engrave Multisig** and choose Verify without engraving"
— which does not exist either) is **not acceptable**; the only honest fallback is to remove
the instruction and say plainly that the remaining plates are unverified and the wallet must
not be funded. Prescribing a nonexistent action on the screen that precedes funding is the
worse failure.

---

## I-5 — The restore document collapses N per-seed passphrases into one boolean and can silently omit a required spending factor

**Defect.** SPEC 4.1 makes the *(seed, passphrase)* pair the derivation unit and asks the
passphrase **per seed**, but S5 routes all of them through `reg.usesPassphrase()`, an
`any()`, and that single bool is the only passphrase signal reaching both operator-facing
surfaces.

**Site.** `gui/multisig_build_census.go:108-131` (`buildPassphraseInventoryLines`), with
`gui/multisig_build_slots.go:230` (`usesPassphrase`) and `gui/multisig_build.go:339`.

**Triggering state (reproduced).** `multisigSelfSlotPickFlow` is multi-select;
`buildSeedForSlot` asks a seed **and its own passphrase** per held slot and calls `reg.add`
unconditionally. The operator holds @0 and @1, types the **same twelve words** twice, with
passphrase `alpha` then `beta`. Measured result: **one** ms1 plate cut (correct — ms1 encodes
words only), **two** mk1 plates both at `m/48h/0h/0h/2h` differing only in a master
fingerprint (`8aaa4f4b` / `d70ed067`), and a restore document reading *"A BIP-39 passphrase
WAS used… Without it… Keep it somewhere separate"* — every reference **singular** —
immediately after asserting *"If any of them is missing, this backup is incomplete."*
Nothing on steel or on that page lets a reader learn a **second** passphrase exists. In a
3-of-4 holding three slots, the operator records "the passphrase", two legs of three recover
years later, and the funds are unreachable — silently, with the backup vouching for itself
throughout. The commoner and equally fatal form is the same bug: two different masters, one
passphrased, and the document cannot say **which** of `ms1 secret share 1 of 2` / `2 of 2`
needs it.

**Evidence.** Probe on a pristine `git archive 7da66bd` copy through the real chain
(`reg.add` ×2 → `buildSlotSources` → `buildSelfKeys` → `assembleBuildPolicy` →
`buildEngraveTail` → `buildFullModeLabel` / `buildPassphraseInventoryLines` /
`buildSlotSourceLines`):
`fp(@0,alpha)=8aaa4f4b fp(@1,beta)=d70ed067` / `ms1 plates cut = 1, mk1 plates cut = 2` /
`usesPassphrase() = true (ONE bool for 2 registered pairs)` / the singular restore text
verbatim. `git log -S usesPassphrase main..s5-multislot` → `023505c`;
`git show main:gui/multisig_build_slots.go | grep -c usesPassphrase` → **0** and
`git show main:gui/multisig_build_census.go | grep -c buildPassphraseInventoryLines` → **0**,
so the text is S5's own, not inherited. `buildSlotSourceLines` is no mitigation: both slots
print with **no account suffix** (`nextAccount` keys on `MasterFP`, which a passphrase
changes, so each distinct fingerprint starts its own counter at 0) and there is no
passphrase column.

**Minimal fix.** Replace the boolean with the per-seed facts the registry already holds.
Minimal viable version:

- add `func (r *seedRegistry) passphrasedSeeds() []registeredSeed` (or return
  `[]struct{Label string; MasterFP uint32; Passphrase bool}`);
- change `buildPassphraseInventoryLines(passphrase bool)` to take that slice and, when more
  than one seed is registered, emit **one line per passphrased seed naming its label and
  fingerprint** — e.g. `"Your seed for @0 (fp 8aaa4f4b) needs a BIP-39 passphrase. Your seed
  for @1 (fp d70ed067) needs a DIFFERENT one."` — and, when some seeds are bare, say so
  explicitly rather than implying all are;
- thread it through `buildPlateInventoryLines` and the `gui/multisig_build.go:339,415` call
  sites (and the new C-3 supply-path call site).

Pin it with a test asserting the two-passphrase build's document contains **two** distinct
passphrase statements.

---

## I-6 — Holding EVERY slot (`open == 0`) dead-ends on a self-contradictory refusal the multi-select picker made reachable

**Defect.** `classifyCosignerSupply` refuses whenever `state != cosignerSourceLoaded`
**regardless of `open`**, so a build needing zero cosigner cards is stopped by a demand for
a cosigner-card payload, with no forward route on the screen.

**Site.** `gui/multisig_build.go:91` (`open := p.N - len(p.SelfSlots)`) into
`gui/multisig_build_payload.go:204-213`.

**Triggering state (reproduced through the real screens).** An operator builds a 2-of-2 they
hold entirely and enters both seeds on the keyboard — no payload is needed for anything,
since `seedEntryFlow` offers the keyboard. `multisigSelfSlotPickFlow` returns `{0,1}` (pick
@0, answer "YES, ONE MORE"; at n=2 the last slot auto-adds — a path
`TestSelfSlotPickerNeverAsksAOneAnswerQuestion` already exercises and passes), so
`open = 2-2 = 0`. `classifyCosignerSupply`'s `case state != cosignerSourceLoaded, have < open:`
is a comma-**OR**, so it returns `cosignerRefuse`, and `buildSupplyRefusal` renders, via
`cardCount(0)`:

> "No payload is loaded, and this policy needs **no cosigner key cards**. This device has no
> card reader: pack the cards on the host with `me sysw pack`, load the payload, then build."

The message simultaneously says no cosigner cards are needed and demands a payload of them.
`showError` is dismiss-only, and dismissing returns straight out of
`buildMultisigPolicyFlow` — the whole build is abandoned. Pre-S5 this was unreachable
because `open` was always `p.N-1 >= 1`. Trace B does not catch it (3 held of n=4, so
`open == 1`).

**Evidence.** Whole flow driven through the screens:
`go test ./gui/ -run TestProbeHoldsEverySlotNoPayload -v -count=1` →
`screen after fp pick: ok=true content="Nopayloadisloaded,andthispolicyneedsnocosignerkeycards…"`,
`flow done=false`. Classification table measured: `state=noPayload have=0 open=0 → REFUSE`;
`state=uncompared open=0 → REFUSE`; `state=loaded open=0 → autoFill`.
`TestUnderSupplyRefusalNamesTheHostRoute`'s table only has `open=2`; the `open==0` cell is
untested. The stale premise that hid it is the comment at `gui/multisig_build.go:46` — "The
@S picker always sets exactly one held slot, so this cannot fire today" — false since commit
`4b10319`.

**Minimal fix.** In `classifyCosignerSupply` (`gui/multisig_build_payload.go:204-213`), let
a zero-demand build through before consulting the payload state:

```go
func classifyCosignerSupply(state cosignerSourceState, have, open int) buildCosignerOutcome {
    if open == 0 {
        return cosignerAutoFill   // nothing to supply; whatever `loaded` returns today for open==0
    }
    switch { … existing arms unchanged … }
}
```

Confirm the `open == 0` return value matches what `state == cosignerSourceLoaded, open == 0`
already yields (measured: `autoFill`), so the three states agree. Add the `open==0` row to
`TestUnderSupplyRefusalNamesTheHostRoute`'s table for all three states, and fix the stale
comment at `gui/multisig_build.go:46` in the same edit (see Minor M-5/M-6/M-12).

---

## I-7 — The §0.1a origin announcement states ONE origin for a build that derived the operator's keys at several accounts

**Defect.** `buildOriginAnnouncement` hardcodes account 0, and `buildReviewLines` prints no
per-slot origin, so the last confirmation screen before the EXPERIMENTAL warning and the
engrave contains **no correct statement** of where a non-account-0 slot's key lives.

**Site.** `gui/multisig_build.go:1541` (`base := derivedSlotOrigin(script, 0)`), rendered via
`buildReviewLines` at `:1403-1438`.

**Triggering state (reproduced, and already minted into a green gate record).** A multi-slot
build where the operator holds @0 and @1 from one master derives at `m/48h/0h/0h/2h` and
`m/48h/0h/1h/2h`; the review prints *"Your key origins: m/48h/0h/0h/2h, the BIP-48 path for
native segwit."* and, per slot, a fingerprint label plus raw xpub chunks — no origin. The
**very next screen** tells the operator to "compare the keys you just reviewed … against the
same wallet in your coordinator"; an operator taking the announced origin at face value
enters `m/48'/0'/0'/2'` for both keys, derives the wrong key for @1, and either blames the
device or mis-registers the wallet at a path the plate does not carry. §0.1a's requirement is
that the device says which path it stamped on **every** slot.

**Evidence.** Probe: assembled md1 decodes to slot origins `m/48h/0h/0h/2h`,
`m/48h/0h/1h/2h`, `m/48h/0h/0h/2h`, while `buildOriginAnnouncement` returns the single
account-0 line. The repo's **own committed S5.D gate record** confirms it in production:
`oracle/gaterecords/S5-trace-b.walk.json` has `keySourcesScreen` stating
`@1 yours: … account 1` and `reviewScreen` stating only
`Yourkeyorigins:m/48h/0h/0h/2h,theBIP-48pathfornativesegwit.` — while the same record's
`S5-trace-b.expect.json` carries both `--origin-path m/48'/0'/0'/2'` and
`--origin-path m/48'/0'/1'/2'`. `cmd/emu/walk_trace_b.js:357` is
`await tap(CONFIRM, 400); // past the Policy Review` and asserts nothing on that screen, so
**the wrong sentence was minted into a green gate record.**

**Minimal fix.** Make the announcement a function of the origins actually used. Change
`buildOriginAnnouncement(script md.MultisigScript)` to take the assembled slot origins (or
the `[]slotSource` + script) and emit either the single line when all held slots share one
origin, or an enumerated line per distinct origin (`"Your key origins: @0 at
m/48h/0h/0h/2h, @1 at m/48h/0h/1h/2h — the BIP-48 paths for native segwit."`). Cheapest
correct alternative if the announcement must stay scalar: append the origin to each slot's
line in `buildReviewLines`, which is where the operator is being asked to compare. Add a
needle for the origin text to `cmd/emu/walk_trace_b.js` so the gate record stops vouching
for the wrong sentence, and **re-mint `S5-trace-b`** after the fix.

---

## I-8 — The seed registry's own justification for having no wipe/idle bound is now false, and its re-decision was scheduled TO S5 and not made

**Defect.** The comment justifying the absence of an idle/wipe bound rests on a premise this
very diff falsified, and the re-decision it explicitly scheduled to S5 was never made.

**Site.** `gui/multisig_build_census.go:58-63`.

**Triggering state.** The comment reads, verbatim: *"The registry today holds exactly one
seed, which is what the shipped flow already held, so an idle limit would buy no reduction in
exposure over the state of the tree. S5 multiplies the masters in it; the bound is filed to
be re-decided there, when it would actually change something."* S5 **is** this diff and does
exactly that: `buildSeedForSlot` now runs once per held slot, so the registry holds up to
**n** live `bip39.Mnemonic` buffers (Trace B: three entries across two masters) for a build
that grew from a few plates to twelve. `seedRegistry.scrub()` runs **once**, via a single
`defer` at `gui/multisig_build.go:189-190`, so every registered master stays live through
`bundleEngrave` and the restore document. An operator who walks away between plates now
leaves two masters' word lists in RAM for the hours the engrave takes, where the shipped flow
left one. `buildPlateInventoryLines` still emits the unchanged non-wiping ruling to the
operator: *"Seed handling: this build does not time out. A seed you entered stays in device
memory until the build ends…"*

**Evidence.** Registry probe: `2 entries, seedIDs=[0 1]` for a two-slot build.
`git diff --stat main..s5-multislot -- gui/multisig_build_census.go` → `1 file changed, 68
insertions(+), 1 deletion(-)`, and the only edit to that function is the passphrase parameter
— the justification paragraph and the ruling sentence are byte-identical.
`git diff main..s5-multislot | grep -iE 'idle|wipeGuard|time.?out|re-decid'` returns only
those two unchanged lines plus unrelated JS `timeoutMs` helpers: **no re-decision exists
anywhere in the branch.** Per the project's own rule, an item scheduled *to* a phase is not
deferrable past that phase.

**Minimal fix — make the decision, in writing, in this diff.** Either:

- **(a) decide to bound it**: scrub each `registeredSeed` as soon as its key is derived in
  `buildSelfKeys` (the mnemonic is not needed after `deriveAccountXpub` unless the slot is
  `both`, which needs it only in the gate), leaving `defer reg.scrub()` as backstop; **or**
- **(b) decide not to**, and rewrite `gui/multisig_build_census.go:58-63` to state the *new*
  premise honestly — n masters resident for the length of a 12-plate engrave — and say why
  that is accepted.

Whichever is chosen, the operator-facing ruling in `buildPlateInventoryLines` must be updated
to describe the multi-seed reality, and the stale "holds exactly one seed" sentence must not
survive the fold.

---

## I-9 — S5 has no analogue of `TestS0GateHasARecord`: the flagship gate's continued presence is unenforced

**Defect.** No test asserts that stage S5 has a gate record, so the flagship walk's evidence
can be deleted and the whole suite stays green.

**Site.** `oracle/record_test.go:359` (`TestS0GateHasARecord`, the only stage-presence test).

**Triggering state (reproduced by deletion).** `oracle/gaterecords/S5-trace-b.record.json`
(or any of its `.expect.json` / `.walk.json` / `.inputs.json` siblings) is deleted, or never
re-minted after a future change to the build-policy flow.
`TestEveryGateRecordCensusMatchesItsCommittedExpectation`,
`TestEveryGateRecordOnDiskVerifies` and the cmd/emu anchor test all iterate
`Records(GateRecordsDir)` — a **listing of what already exists** — and Fatal only when the
directory is entirely empty. Since S0's four files remain, all of them still pass.

**Evidence.** One skeptic deleted all four `S5-trace-b.*` files and ran the suite:
`go test ./oracle/... -count=1 -v` → all PASS, including
`TestS0GateHasARecord: "S0 gate records: [S0-trace-a.record.json]"` and
`TestEveryGateRecordOnDiskVerifies: "verified 1 gate record(s)"`; then
`go test ./... -count=1` → `ok seedhammer.com/oracle 0.123s` and every other package `ok`.
Files restored; `git status --porcelain oracle/gaterecords/` empty. This is this cycle's
Critical #4 one layer up: not the walk's correctness, but **the evidence that a walk ran at
all.**

**Correction to the finding's evidence, recorded:** it cites
`TestGateRecordStringsAreRecordsOfTheCardsPayload`, which does not exist by that name; the
real generic iterators are the three named above. The reproduction is unaffected.

**Minimal fix.** Add, beside `TestS0GateHasARecord` in `oracle/record_test.go`, the same
clause for S5 — the mechanism already works (`StagesRecorded` is generic, and
`json.load(...)['stage']` on `S5-trace-b.record.json` returns `S5`):

```go
func TestS5GateHasARecord(t *testing.T) {
    stages, err := StagesRecorded(GateRecordsDir)
    if err != nil { t.Fatalf(...) }
    if len(stages["S5"]) == 0 { t.Fatalf("S5 has no gate record in %s (stages present: %v).%s", ...) }
    t.Logf("S5 gate records: %v", stages["S5"])
}
```

Better still, make it table-driven over a `requiredStages = []string{"S0", "S5"}` constant so
the next stage cannot forget.

---

## I-10 — The self-slot picker cannot express a mixed `both`/`derived` source, and the code's "filed rather than smuggled in" claim does not check out

**Defect.** One yes/no question sets a single `p.SelfFromCard` bool applied to the **whole**
held set, so the genuinely mixed configuration the model and gate already support is
unreachable through the shipped UI — and the in-code comment's claim that this limitation was
filed as a follow-up is false.

**Site.** `gui/multisig_build.go:80-92` (`buildSelfSourceFlow` called once on `p.SelfSlots`;
`p.SelfFromCard` is one bool) and `gui/multisig_build_slots.go:481-538` (`buildSlotSources`
applies it uniformly: `if isHeld && !p.SelfFromCard { … slotFromSeed … } else if isHeld {
kind = slotFromBoth }`).

**Triggering state.** The operator holds @0 (whose key is **also** on a payload card, so they
want the §4.3 cross-check) and @1 (freshly derived, no card exists). Answering **YES** forces
@1 into `slotFromBoth` too, which dead-ends on an under-supply refusal or a spurious
`errBuildSeedKeyMismatch` at the gate; answering **NO** silently skips the available
cross-check on @0. There is no third path. This contradicts SPEC §4.3's per-slot language,
verified at `design/SPEC_multisig_build_repair.md:383`: *"Every slot @0..@{n-1} carries
exactly one source, **chosen by the operator** and shown on a review screen before assembly."*

**Evidence.** `buildSlotGate`'s `slotFromBoth` arm returns `errBuildSeedKeyMismatch{Slot}` on
a per-slot mismatch, and `classifyCosignerSupply` refuses first when the now-required N cards
are not supplied. The code's own comment at `gui/multisig_build_slots.go:510-518` concedes the
gap ("a genuinely MIXED build — @0 on a card, @1 derived — is not expressible through the
screens") and claims it was "filed rather than smuggled in". It was not:
`grep` over `design/FOLLOWUPS.md` (6955 lines) for `SelfFromCard`, `buildSelfSourceFlow`,
`per-slot`, `mixed build`, `not expressible`, `genuinely mixed` returns **zero hits**, and
F-189..F-195 read verbatim (lines 6848-6955) contain no match. The trail is legible:
`design/agent-reports/s5-picker-and-verify-implementation.md` §4 has the implementer stating
*"I did not write to design/FOLLOWUPS.md"* and **drafting** an `F-new` entry for exactly this
gap — its sibling from the same round (F-191) was landed by the controller on the
implementer's behalf; this one was not. `TestSelfSourceQuestionNamesEverySlotItAnswersFor`
passes but only checks the screen *announces* the single-bool assumption; the mixed-source
gate tests hand-build `[]slotSource` slices and bypass the picker entirely.

**No funds-loss path** — every reachable branch either refuses loudly or falls back to an
all-derived (safe) configuration — which is why this is Important, not Critical.

**Minimal fix — two parts, and the second is not optional.**

1. **Land the follow-up.** Add an `F-196` (next free number) to `design/FOLLOWUPS.md` naming
   the gap, with an **owning phase**, so the code comment's claim becomes true. Use the
   implementer's drafted text in `design/agent-reports/s5-picker-and-verify-implementation.md`
   §4.
2. **Fix the comment or the code.** Either amend
   `gui/multisig_build_slots.go:510-518` to cite the follow-up ID (making it checkable), or —
   if the mixed shape is judged in-scope — ask the source question **per held slot** inside
   `multisigSelfSlotPickFlow`'s loop and carry a `[]bool` instead of `p.SelfFromCard`, which
   `buildSlotSources` and `buildSlotGate` already support unchanged.

A claim of the form "filed rather than smuggled in" is exactly the class of assertion a
reviewer inherits as a given; it must be a grep, not a promise.

---

## I-11 — The abort screen's recovery instruction has no mechanism: there is no way to skip a plate that is already cut

**Defect.** The abort modal promises a resumable re-run ("it cuts the same plates, byte for
byte, so you only cut the ones you are missing") that `bundleEngrave` cannot deliver — it
always walks the plan from index 0 and offers no skip.

**Site.** `gui/bundle_flow.go:489-492` (the promise) against `:348-363` (`bundlePlatePlan`,
always full) and `:381-410` (the loop).

**Triggering state.** Trace B, full mode, 9 plates. Power fails after plate 6, or the operator
runs out of blank steel at plate 7 (the census itself tells them to have that many blanks
ready). They re-run, give the same answers, and land on `Card 1 of 6 | Plate 1 of 1`, the ms1
seed plate already on the bench. Their options are exhaustively three, all bad:

1. cut a **second seed plate** — the exact outcome `buildEngraveTail` refuses to produce on
   purpose (`gui/multisig_build_tail.go:70-72`, "the second would be a duplicate secret on
   steel") and which contradicts the census and inventory the same run prints;
2. re-run the identical job onto the already-engraved plate — named nowhere on any screen;
3. press **Back**, which aborts and re-shows the same message — a closed loop.

There is no fourth: the per-plate `ChoiceScreen`'s only `Choices` are `validateMdmk`'s
`{"TEXT + QR","TEXT ONLY","QR ONLY"}` (`gui/gui.go:2299-2303`) with no skip row anywhere in
the tree; `EngraveScreen.Engrave` returns `true` only after `engraveDone` **and** an operator
confirm (`gui/gui.go:2977-2986`), every other exit re-showing the same picker for the **same**
plate; and `cs.Choose` returning `!ok` aborts the **whole set** (`gui/bundle_flow.go:398-402`).
No persisted per-plate state exists (the only `resume` machinery, `releaseResumeState`, is
mid-cut resume of a *single* plate). The sibling `multiPlateEngrave` still says the honest
thing: *"discard the partial plate(s) and start over"* (`gui/derive_xpub.go:523-527`).

**Evidence.** All four `bundleEngrave(` call sites pass the full `cards` slice with no
start-index. `TestReRunMintsByteIdenticalPlates` asserts only data-level determinism
(`assembleBuildPolicy`/`buildEngraveTail` byte-equality) and never touches `bundleEngrave`,
`ChoiceScreen` or `EngraveScreen` — the byte-identity property is real, but nothing turns it
into an operator-usable resume. `bundleAbortWarningText(bundlePlate{cardIdx:1,cardTotal:6,
label:"ms1 secret share 1 of 2"}, true)` was executed and printed the promise verbatim.

**Minimal fix — the cheap correct one is the text.** Change `gui/bundle_flow.go:489-492` to
say what the device can actually do, mirroring its own sibling: the plates are byte-identical
on a re-run, **and** a re-run starts from plate 1, so the operator must either finish this set
in one sitting or discard the partial set and start over. Explicitly warn that re-running
will re-cut the seed plate.

If a resume is wanted instead (larger, and not required to clear this gate), it needs a
`startAt` parameter on `bundleEngrave` plus a "Skip — already cut" row on the per-plate
picker with its own confirmation, and must never let a skipped plate count as verified.

---

## I-12 — An abort does not propagate: the verify offer and the restore-doc inventory run exactly as if the engrave had completed

**Defect.** `bundleEngrave` returns `void` and both abort paths are bare `return`s, so
neither caller can distinguish a completed run from an aborted one, and both walk straight on
to the verify offer and the full restore document.

**Site.** `gui/bundle_flow.go:381` (void signature), `:389` and `:402` (bare returns),
`:481-483` (the false premise in the comment), against `gui/multisig_build.go:364 → :397 →
:415` and `gui/multisig.go:272 → :295 → :302`.

**Triggering state (reproduced end-to-end through the real screens).** Trace B, full mode, the
operator aborts at card 4 of 6 out of blanks and reads *"Bundle Incomplete … This set is not
a usable backup yet."* Two screens later they are asked **"Verify the engraved plates?"** — a
verify that structurally cannot succeed, because the md1 card is emitted last and was never
cut, so it dies at `extractReadbackMd1AndMk1s` with *"Read back one wallet-policy md1 AND the
operator key card(s) (mk1)."*, which reads as *your plates are unreadable* rather than *you
never cut the md1*. (If enough plates exist to pass that, the length precheck instead prints
*"Read back 1 key plate, but this run engraved 3 key plates"* — a false statement about what
the run engraved, and an instruction the operator cannot satisfy.) Then the restore document
prints, headed *"This backup is 9 plates: … If any of them is missing, this backup is
incomplete."* — the artifact the diff itself says is read years later, alone, printed as the
last word of a run the device just said produced no usable backup, with nothing
distinguishing the two.

The comment at `:481-483` asserts the opposite and is false for **every** abort: *"The restore
document … is printed at the end of a SUCCESSFUL run — an operator whose engrave died never
reaches it. This modal is the only screen they get."*

**Evidence.** A probe drove the real `supplyMultisigPolicyFlow` to the first engrave picker
and clicked Back: `abort warning frame: "Stopped at card 1 of 4 (ms1 secret share). This set
is not a usable backup yet. … Bundle Incomplete"` immediately followed by
`post-abort frame 0: "Verify the engraved plates? Verify now Skip Verify Bundle"`, with zero
guard screens between. `TestBundleEngraveSetAbort` cannot see this: it calls `bundleEngrave`
directly and stops at the warning, never invoking the caller that contains the unguarded
continuation.

**Absorbed:** the Minor "An aborted engrave still offers the verify and prints the full
restore-doc inventory" is the same defect at the same site and is folded here.

**Minimal fix.** Give `bundleEngrave` a result and gate both callers on it:

```go
// gui/bundle_flow.go
type bundleEngraveResult int
const (bundleEngraveDone bundleEngraveResult = iota; bundleEngraveAborted)
func bundleEngrave(...) bundleEngraveResult { … return bundleEngraveAborted (at :389, :402) … }

// gui/multisig_build.go:364 and gui/multisig.go:272
if bundleEngrave(ctx, th, title, cardsOut) != bundleEngraveDone { return }
```

Delete the false premise at `:481-483` in the same edit. This also unblocks I-4's preferred
fix, which needs a verify result for the same reason — land the two together, and add a
flow-level test asserting that an abort is the **last** screen of the program.

---

## I-13 — A watch-only verify tells the operator "secret verified" when no seed plate exists and no ms1 was ever typed

**Defect.** `multisigVerifyOKMessage`'s single-leg arm ignores `full` and returns the
secret-claiming body, so a watch-only run — in which no ms1 is created, requested, typed or
compared — reports that a secret was verified.

**Site.** `gui/multisig_verify.go:714-717` (`if legs <= 1 { return multisigVerifyOKBody }`).

**Triggering state (reproduced).** The ordinary single-slot case — Build policy or Supply
policy, one held/matched slot — with **"Watch-only (keys)"** chosen (`full := modeSel == 0`
at `gui/multisig_build.go:345` / `gui/multisig.go:210`). No ms1 card is created,
`multisigVerifyMS1Entry` is never called (guarded by `if full` at `:597`), and
`bundle.Verify`'s ms1 leg is skipped on both sides by presence semantics. The final screen
nevertheless reads **"Operator key and secret verified."** — asserting a check of a secret
that was never engraved, never typed and never compared. `len(legs)==1` is the common case.

**Evidence.** Executed on a copy: `multisigVerifyOKMessage(1,false)` and
`multisigVerifyOKMessage(1,true)` return the **byte-identical** string *"Operator key and
secret verified. Other cosigners' keys are taken as supplied."*, while
`multisigVerifyOKMessage(3,false)` correctly omits the secret claim. The function's own doc
comment at `:709-710` states the rule it breaks: *"`full` is the mode, so a watch-only run
does not claim a secret it never asked for."* `TestMultisigVerifyNoticeIsHonest`
(`gui/multisig_verify_test.go:171`) cannot catch it — it drives `showNotice` with the
constant directly and never calls `multisigVerifyOKMessage`, so `full` is untested at
`legs<=1`.

**Minimal fix.** Make the single-leg arm `full`-aware, exactly as the multi-leg arms are:

```go
func multisigVerifyOKMessage(legs int, full bool) string {
    if legs <= 1 {
        if full { return multisigVerifyOKBody }
        return "Operator key verified. Other cosigners' keys are taken as supplied."
    }
    …
}
```

Pin it with a four-cell table test over `(legs ∈ {1,3}) × (full ∈ {false,true})` asserting
that the word "secret" appears **iff** `full`.

---

## I-14 — The verify's "already checked / different seed" arm asserts a foreign seed where a same-seed passphrase divergence is equally likely (new site of the F-191 class)

**Defect.** Two of the three arms of the verify's seed-classification switch assert a fact
the device cannot support — "The plates still outstanding belong to a different seed." —
while the routine that distinguishes the real cause is wired only into the third arm.

**Site.** `gui/multisig_verify.go:566-573` (the `default:` arm), against `:563-566` where
`multisigVerifySeedIsInnocent` is called.

**Triggering state (reproduced).** S5 makes the passphrase prompt **per held slot**
(`buildSeedForSlot` once per `p.SelfSlots` entry), with **no confirm-entry**
(`passphraseFlowTitled` returns on a single OK, `gui/gui.go:694-720`) and no comparison
between two registry entries holding the **same words**. The operator holds @0 and @1 from one
seed, types the words for @0 with passphrase `hunter2` and the same words for @1 with
`huntre2`. `buildSlotSources` keys the account counter on `seed.MasterFP` — the fingerprint of
the *(seed, passphrase)* **pair** — so the two entries look like two masters and **both get
account 0**, deriving at the same `m/48'/0'/0'/2'`; the keys differ, so §4.1's duplicate
refusal does not fire; the review screen suppresses the account suffix at account 0, so both
lines read identically. Two mk1 plates are cut and one ms1 (correctly deduped) encoding the
words and neither passphrase. At verify, seed+`hunter2` covers @0; the operator is asked for
the next seed, types the same words again, and is told:

> "That seed's slots have already been checked. The plates still outstanding belong to a
> different seed."

then *"Verify Incomplete: Checked 1 of the 2 key plates this run engraved."* The device sends
an operator holding the **only seed that exists** to look for a second one, and the real cause
— a passphrase divergence on the same words — is the one explanation it never names, despite
already owning the routine that distinguishes it.

**Evidence.** Reproduced through the production helpers on the frozen tree:
`MasterFP @0=ca2c62d2 @1=939cd11b (distinct, as buildSlotSources sees them)`;
`sources: @0 account=0, @1 account=0`; both origins `m/48h/0h/0h/2h` with different xpubs;
second leg `slots2=[0] everOwed=[0] covered=map[0:true]` → falls into the `default:` arm,
with `multisigVerifySeedIsInnocent` never consulted. `duplicateSlotPair` compares only
identical 65-byte key material, so §4.1 cannot fire. No existing test approaches this: the
F-191 tests exercise the `len(slots)==0` arm via empty-vs-nonempty passphrase on one seed,
and `TestVerifyReportsIncompleteAfterAMidLoopRefusal` uses a genuinely different master.

**Minimal fix.** Wire the existing routine into the other two arms and soften the claim.
In the `default:` arm at `:566-573`, before composing the message, call
`multisigVerifySeedIsInnocent(m, passphrase, net, keys)`; when it reports the words are a
cosigner's under some passphrase, say so instead of asserting a foreign seed:

> "That seed's slots have already been checked. The outstanding plates were built from
> **different words, or from these words with a different BIP-39 passphrase.**"

The unconditional half of the fix is one line of prose and costs nothing: **never assert
"a different seed" when the device cannot distinguish that from "the same seed, a different
passphrase."** Consider also, as a follow-up rather than a gate item, adding a confirm-entry
to the per-slot passphrase prompt — a single-press accept on a spending factor that is never
echoed anywhere is the upstream cause.

---

# Recurrence of this cycle's four known defect CLASSES

**All four recurred.** This is the single most important line in the report: the cycle's own
lessons did not hold across the S5 diff, and three of the four recurrences are among the
blocking findings.

### 1. Fail-safe dedupe — **RECURRED (Critical)**
`buildSlotGate` groups its bindings on `s.SeedID` (`gui/multisig_build_slots.go:377`) while
the flow mints a fresh `SeedID` **per held slot**, so the group size is always 1 and SPEC
4.3 row 5's notice can never fire (**C-2**). This is *verbatim* the identity mistake the
sibling function `buildEngraveTail` already shipped once and fixed — its own comment at
`gui/multisig_build_tail.go:57-67` records the diagnosis and the remedy (key on the minted
ms1, not the SeedID). **The fix was applied to the tail and never swept into the gate.**
A softer instance is **I-5**: `usesPassphrase()` is an `any()` that collapses N per-seed
passphrases into one bool, so a second required passphrase is deduped out of existence in the
restore document.

*Standing remedy:* whenever a dedupe or grouping key is chosen, grep every other site that
groups the same entities and prove they use the **same** identity. A per-slot-minted surrogate
ID is never a seed identity.

### 2. Unrealistic fixture — **RECURRED (Critical + 2 Important)**
- **C-2's only guard**, `TestGateAcceptsSameSeedAtDistinctOrigins`
  (`gui/multisig_build_gate_test.go:207-215`), hand-builds two `slotSource`s sharing
  `SeedID: 0` — **a registry `buildMultisigPolicyFlow` cannot construct.** Mutating the entire
  notice loop to `return nil, nil` fails exactly that one assertion, so the impossible fixture
  is the only thing pinning the mechanism, and the real shape has zero coverage.
- **I-1**: all 7 `buildEngraveTail` test call sites use `s5TraceB`, whose held slots are all
  `slotFromSeed`; every `slotFromBoth` fixture stops at the gate, so the tail's `both` arm is
  exercised by nothing.
- **I-2**: all 5 `multisigVerifyFlow` drivers pass `full=false`, so the mode that ships is
  driven by no test; `TestVerifyCoversEveryMastersSecret` hand-builds `verifyLeg` values and
  proves the comparator instead of the assignment.

*Standing remedy:* a fixture must be **produced by the production constructor**, not
hand-assembled. Where a test builds a struct literal that a flow builds elsewhere, it is a
unit test of a shape that may not exist — state that in the test, and pin the real shape
separately.

### 3. Cardinality-not-identity — **RECURRED (Critical)**
**C-1** is the flagship: `len(legs)` counts slots **re-derived from a typed seed** and is
reported as **plates checked**; every upstream gate on that path is a count or a provenance
(`:478` md1 equality, `:493` mk1 count) and none is plate identity; `verifyMultisigLegs`, the
only function that binds a plate to a slot, is structurally unreachable. Adjacent instances
in the Minor list — the `:493` precheck described in-code as "a courtesy, not the mechanism"
while it **is** the mechanism for one direction, and `verifyClaimPlate` pairing on
`got.Xpub == w.Xpub` while acceptance additionally requires fingerprint, origin and stub
binding — are the same class one step down.

*Standing remedy:* any operator-facing sentence containing a count must name the predicate
that produced it. "Checked N of M" must be produced by the comparator, not by the loop
counter that precedes it.

### 4. Unrunnable / never-executed gate — **RECURRED (Important ×3)**
- **I-9**: deleting all four `S5-trace-b.*` gate-record files leaves `go test ./...` fully
  green — **reproduced**. The presence of the flagship gate's evidence is enforced for S0 and
  not for S5.
- **I-2**: the entire `if full` block, all of `multisigVerifyMS1Entry`, and the full-mode
  success string have **zero executions** in the coverage profile. A gate that has never
  executed is a hypothesis, not a gate.
- **I-11 / I-12**: the abort path's promised recovery mechanism does not exist, and the abort
  itself does not propagate — the "did the operator actually get out of this state" question
  is asked by no test at flow level.

*Standing remedy:* before closing, enumerate every gate the artifact defines and prove each
one has **run at least once**, and that its absence would fail. Where a gate's subject is a
committed file, assert the file is required by name, not that the directory is non-empty.

**A fifth pattern, worth naming for the next cycle:** *comments outliving their conditions.*
Five separate stale claims are load-bearing in this diff — three asserting a single-select @S
picker that this same branch replaced in `4b10319` (one of which is the exact premise that
hid **I-6**), the seed-registry justification whose premise S5 falsified (**I-8**), and the
"filed rather than smuggled in" claim that `grep` refutes (**I-10**). Each is a reachability
or provenance assertion a reviewer inherits as a given.

---

# Minor / Nit (recorded, not blocking) — 13 Minor, 2 Nit

One Minor ("An aborted engrave still offers the verify and prints the full restore-doc
inventory", `gui/multisig_build.go:364-416`) has been **absorbed into I-12** and is not
repeated here.

| # | Sev | Title | Site |
| --- | --- | --- | --- |
| M-1 | Nit | The verify's plate-count precheck is inert — deleting it leaves the suite green (`if false && …` → `ok gui 78.244s`). Every *other* mutation in the battery failed at least one test. | `gui/multisig_verify.go:493-500` |
| M-2 | Minor | The readback-count precheck is unpinned; only comments say what it does (M17 SURVIVED). What regresses invisibly is the timing benefit — learning about a mislaid plate **before** typing a seed. | `gui/multisig_verify.go:493-500` |
| M-3 | Minor | Two named structural refusals are unreachable and unasserted: deleting `errBuildNoHeldSlot` (M18) or `errSupplyNoMatchedSlot` (M21) leaves the suite green. If a future caller loses the guarantee, the build engraves an md1 with **no key plate at all**. | `gui/multisig_build_tail.go:131-133`, `gui/multisig_supply_tail.go:160-162` |
| M-4 | Minor | Two production comments assert the @S picker is still single-select; false since `4b10319`. One **explicitly licenses the hand-built-fixture practice that caused C-2**. | `gui/multisig_build_slots.go:40-44`, `gui/multisig_build.go:46` |
| M-5 | Minor | (Same class, second sighting) two comments in this diff assert a single-select picker this same diff replaced; a reviewer reading `multisig_build_slots.go:40` alone would dismiss I-14's scenario as unreachable. | `gui/multisig_build_slots.go:40-44`, `gui/multisig_build.go:869` |
| M-6 | Nit | Engraved plate labels never name the slot (`mk1 key 1 of 3`) where the oracle's own labels do (`(mk1 key, slot 0)`). Usually recoverable by decoding the origin — **except in I-14's shape**, where two mk1s declare the same path. | `gui/multisig_engrave.go:78-83` |
| M-7 | Minor | `verifyClaimPlate` pairs on a predicate coarser than the one that accepts (`got.Xpub == w.Xpub` vs `bundle.Verify`'s fingerprint + path + stub binding), so greedy can pick the wrong plate. Unreachable today only because of the `:493` precheck the code invites relaxing. | `gui/multisig_verify.go:329-347` |
| M-8 | Minor | The unclaimed-plate sweep cannot fire in production and the comments assert the opposite arrangement; `errVerifyPlateUnclaimed.Error()` has zero executions. | `gui/multisig_verify.go:318-322`, `:157-159`, `:487-492` |
| M-9 | Minor | S2's ruled ordering (duplicate-key outranks the origin refusal) is silently reversed for an unparseable card origin: the operator sees the generic "Couldn't assemble the wallet policy" and is sent to fix the wrong input. | `gui/multisig_build.go:1166-1169` in the fill loop at `:1256`, before `duplicateSlotPair` at `:1278` |
| M-10 | Minor | The restore doc's descriptor carries no key origins, which S5 made load-bearing by giving each slot a different one. It is now the only kept surface that has **lost** information relative to the plates it describes. | `gui/multisig_restore.go:59-62` |
| M-11 | Minor | The verify readback gather screen is titled **"Engrave Bundle"** (should be "Verify Bundle"); the guarding test's file table never looks at `multisig_verify.go`. Verbatim in `main` too. | `gui/multisig_verify.go:455` |
| M-12 | Minor | Aborting at plate 1 issues DESTROY instructions for a seed plate that does not exist. A guard on `cardIdx==1 && plateIdx==1` keeps the warning for the runs where it matters. | `gui/bundle_flow.go:493-495` |
| M-13 | Minor | The supply path's restore document carries no plate inventory at all (`nil` extra) — **closed by C-3's second line.** | `gui/multisig.go:302` |
| M-14 | Minor | A TEMPLATE-form build shows no restore document, so its passphrase statement exists only on the mode label; the artifact meant to outlive the operator is not written at all. | `gui/multisig_build.go:412` |
| M-15 | Minor | Three comments still assert a single-select @S picker, one naming the retired singular field `@SelfSlot`; the premise that hid **I-6**. | `gui/multisig_build_slots.go:40-44`, `gui/multisig_build.go:46`, `:890` |

M-4, M-5 and M-15 are the same defect class at overlapping sites and should be fixed in **one**
sweep: `grep -n "always sets exactly one\|still single-select\|@SelfSlot\|selfSlot" gui/*.go |
grep -v _test`. Fixing them is cheap and directly reduces the chance of the next I-6.

---

# REFUTED — do NOT reinstate

## R-1 — "WORKTREE DAMAGE: I deleted 4 tracked files from the frozen tree; repair before proceeding" (filed Critical)

**Claim.** That the frozen worktree at `/scratch/code/shibboleth/wt-s5` currently has four
tracked files (`oracle/gaterecords/S5-trace-b.{expect,inputs,record,walk}.json`) deleted from
disk, evidenced by ` D` entries in `git status --porcelain`, and that this must be repaired
before the gate proceeds.

**Why refuted (both skeptics, independently).**

1. **The claim fails its own prescribed repro.** `git status --porcelain --branch` on the
   frozen tree prints the branch line only — no ` D` entries anywhere. The working tree is
   clean, `HEAD` matches the frozen SHA `7da66bd`, and all four `S5-trace-b` files are present
   on disk with content.
2. **Every corroborating check agrees.** `git diff --stat HEAD` is empty;
   `git diff --exit-code HEAD -- oracle/gaterecords/` exits 0; `ls -la` shows the four
   `S5-trace-b` files at 6935 / 1808 / 4420 / 6962 bytes alongside the four `S0-trace-a`
   files, matching `git ls-files` exactly (8/8 present). The `.git` file and worktree
   registration resolve correctly to the `wt-s5` path itself, and `git worktree list` shows no
   corruption.
3. **It is not a code-correctness finding at all.** Its `site` is `n/a`; it is a claim about
   the reviewer's own transient worktree state during a `cp -a` probe, not about the artifact
   under review.

**Confirmed again at verdict time** by this agent: `git status --porcelain --branch` →
`## s5-multislot` only; `git rev-parse HEAD` → `7da66bd7841b96f879b7f5a957c66ad16744e3d2`;
`ls oracle/gaterecords/` → all 8 files present. **There is nothing to repair.** Do not
reinstate, and do not spend a fold round on it.

*Process note worth keeping:* several skeptics reported transient dirt in this shared
worktree from concurrent agents. That is an artifact of running probes against a shared tree,
and it is why every reproduction in this report was performed in a `cp -a` copy or a
`git archive` extract. **Keep that discipline; the frozen tree is evidence, not a workspace.**

---

# What must happen before the next gate

1. **Fix all 3 Criticals and all 14 Importants.** No item here is deferrable: every one of
   them is either in the funds path, in an operator-facing claim about the funds path, or in
   the evidence that the funds path was checked.
2. **Land the follow-up I-10 requires** — the code currently claims a filing that does not
   exist, and a claim of that shape is exactly what a reviewer inherits as a given.
3. **Rebuild the fixtures named in defect-class 2 before re-review**, not after. A fold that
   fixes C-2's production line while leaving `TestGateAcceptsSameSeedAtDistinctOrigins`
   hand-building `SeedID: 0` twice has not closed the finding — it has moved it.
4. **Run the build gate on the fold.** A fold is authorship and re-earns it. In particular,
   C-2's re-key changes a map's key type and requires the `slotFromSeed` arm to resolve
   `seed`; I-12 changes a function signature with two call sites; I-6 changes a switch's
   arity. All three are a `go build` away and none is worth a review round.
5. **Re-mint `S5-trace-b`** after I-7 (and after any change that alters a recorded screen),
   and add the needle so the walk asserts the origin sentence instead of tapping past it.
   Land `TestS5GateHasARecord` (I-9) in the same commit so the re-minted record is protected.
6. **Re-review scope.** Per the proportional rule, this fold is **non-trivial** — new logic,
   changed control flow, and three Critical fixes — so it re-triggers the gate. Scope the
   re-review to *"did the fold fix each finding, and did it introduce a new defect"*, and
   state in the brief the facts already settled here (the machine baseline, the refuted R-1,
   and the mutation results) so they are not re-derived.
7. **Then, and only then**, consider the single highest-stakes pre-execution review before the
   first plate is cut on real key material.

**Nothing is engraved, and nothing is merged, until this returns 0C/0I.**
