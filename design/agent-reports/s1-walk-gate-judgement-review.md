# Review — is C1 fatal to S1–S5's walk gates, and what should replace them

## Verdict on C1

**CONFIRMED for all five.** No stage's `"by emulator walk"` clause is satisfiable
today, and none survives on its test half, because §4.5 is a REQUIRED spec clause
carrying a verbatim operator requirement and R-4 says in terms that a green unit
suite does not close a stage — so "by test" is not a fallback the plan is free to
take. S3 is the only partial: half of its gate is a `grep` that needs no walk at
all, and the line its behaviour change touches (`gui/multisig_restore.go:51`) is
reachable from `supplyMultisigPolicyFlow` (`gui/multisig.go:174`) as well as from
the build flow, so a cheaper walk would carry the same evidence if the gate's word
"build" were relaxed.

## Per-stage table

| stage | its gate text, quoted | needs the build flow? | survives on tests alone? | verdict |
| --- | --- | --- | --- | --- |
| S1 | "Trace A reaches the gather with both cards, by test and by emulator walk. Then: **either the flow completes an engrave, or D-1 reproduces and is captured as a failing test**" | yes — the gather it names is `buildMultisigPolicyFlow`'s, and the sibling flow's gather proves nothing about `takeAll` | no | C1 confirmed. Needs F-169 only; census inert, no byte comparison |
| S2 | "Trace A completes end to end: engrave, by test and by emulator walk. The md1 is compared by **production**… the current primary BUILDS an md1 from the same inputs and the strings are equal" | yes | no | C1 confirmed, and under-stated: even with a walk the comparison has no mechanism (C4/F-171) |
| S3 | "Emulator walk of an **`sh(wsh)` build** … showing `P2SH-P2WSH` on the restore doc, and **`grep -rn TYPED-ONLY --include='*.go' gui/` returns 0**" | as written yes; the grep half needs no walk, and the renderer is shared with the supply path | half — grep + the two unit tests carry the naming change | C1 confirmed as written; the one gate where a cheaper walk is equivalent evidence |
| S4 | "Every failing row demonstrated failing. Emulator walk of the `both` happy path and of one loud failure" | yes | no | C1 confirmed but not actionable: the screens do not exist until S4 |
| S5 | "Trace B completes: correct descriptor, by test and by emulator walk. **The §4.5 comparison extends to every mk1 and to EVERY ms1, byte for byte**" | yes | no | C1 confirmed; needs F-169 + F-170 + F-171 |

## Findings

### C-1 (Critical) — Trace A cannot complete on the payload S0 delivered, once S1's `takeAll` lands

Measured, `nix develop --command go test ./cmd/emu -run TestSyswCardsPayloadCoversEveryStagesWalk -v`:

    sysw_cards_payload_host_test.go:115: 9 md1/mk1 records, 1 seed(s)

Nine `ClassMDMK` records = **four** cosigner cards (`cmd/buildpayloadcards/main.go:50-55`
lists A@0, A@1, B@0, C@0). S1's implementation note is "replace the single
`syswOffer` seeding … with **every** `ClassMDMK` record fed through
`bundleGatherFlow`'s `offer()`", and there is no per-card decline: the gather has
only `dropPending` for an incomplete chunk set (`gui/bundle_flow.go:124-127`), no
removal of an added card. Then `gui/multisig_build.go:61` calls
`buildCosignerCards(cards, p.N-1)`, whose last check is `if len(out) != want {
return nil, false }` (`:270`), and the flow shows *"Gather exactly %d cosigner key
cards (and no md1)."*

So for Trace A (2-of-3, `p.N-1 == 2`) the payload feed delivers four cards into two
open slots and the build refuses. Consequences, both inside this brief's question:

- **S2's gate is unsatisfiable** — "Trace A completes end to end: engrave" cannot
  happen at n=3 on this payload, with or without a walk.
- **S1's gate answers its own disjunction ambiguously**: the walk ends on a
  legitimate over-supply refusal that is *not* D-1, and S1's own test 6
  (`TestBuildRefusesMoreCardsThanOpenSlots`) makes that refusal the specified
  behaviour. A gate whose two arms are "engrave" and "D-1 reproduced" has a third
  outcome nobody assigned.
- The only n this payload admits under an unconditional feed is **n=5**, which is
  not Trace A and is not Trace B.

This is independent of C1 and survives every one of the three options. **Fix (one
ruling, before S0b or S1 is scheduled):** decide whether the payload feed is
per-card accept/skip — which is S1 scope, one screen, and restores every n — or
whether the walks run n=5 and §2's Trace A shape is restated. Silence here buys a
walk that cannot pass and a second discovery round at S2.

### I-1 (Important) — the recon's own remedy prescribes the harness substitution that makes S1's gate pass without S1's feature

`RECON_S1_S6_walk_gates.md` §5 row 3: *"`"First card from where?"` + gather |
`shSysw` + `shNFC.present`, **as today**"*. A build walk that completes its gather
by presenting chunks over the emulated NFC reader is green whether or not
`takeAll` exists — it is the "zero scans" clause of S1's test 3 deleted from the
walk half of the same gate. The emulated reader is a harness affordance; phase 1
hardware has none.

**Fix:** the S1-and-later build walk asserts **zero** `shNFC.present` calls (count
them in the harness, assert the count, and let that assertion be one of the
seen-to-fail mutations). An NFC-fed build run is a driver smoke test and must be
labelled as one — never a stage gate.

### I-2 (Important) — the plan's §3 preamble exempts S3 from the derived census on a false premise

Plan `:194-196`: *"where a stage's walk produces **no** artifact — S1 and S3 end at
a screen, not an engrave — the census is inert."* Measured in
`gui/multisig_build.go`: `bundleEngrave(ctx, th, cardsOut)` is `:168`;
`multisigRestoreDocFlow(ctx, th, tpl, keys)` is `:191`. The restore doc S3's gate
reads is **after** the engrave, so any walk that satisfies S3 has cut plates. S3
therefore inherits exactly the C3 defect the preamble exists to prevent: it may
engrave wrong artifacts and pass on a screen string.

**Fix:** strike S3 from the preamble's exemption (S1 alone is artifact-free) and
add F-170 to S3's owning set alongside F-172.

### I-3 (Important) — C1's supporting sentence is measurably false for S2, and it discards a regression gate that already exists

Recon `:66-68`: *"the only walk that exists cannot execute one line any of them
changes."* S2 edits `gui/bundle_flow.go:155`,
`layoutTitle(ctx, dims.X, th.Text, "Engrave Bundle")` — **inside the shared
gatherer**, which the existing bundle walk renders and waits on
(`waitFor("Scanacard,orDone")`). That is S2's riskiest edit by the plan's own
account: one shared file and five call sites across four flows that have nothing
to do with multisig build. The conclusion (the gates' *subjects* are unreachable)
is unaffected; the absolute is wrong, and it throws away the only automated
coverage D-4's blast radius has.

**Fix:** correct the sentence to "cannot execute the flow any of their gates name",
and add to S2's gate: **the S0 bundle walk is re-run green after D-4 lands**, since
that walk is now a regression check rather than a feature gate.

### I-4 (Important) — a flow-identifying needle exists TODAY; F-169 says it arrives only after S2

F-169: *"After S2 fixes D-4 the gather title becomes that discriminator; before then
it is a decoy."* Measured with `git grep -F … -- 'gui/*.go' | grep -v _test`, each a
single production site:

    gui/multisig_build.go:300  Lead: "Choose policy type"
    gui/multisig_build.go:376  Lead: "How many keys (n)?"
    gui/multisig_build.go:394  Lead: "Which slot is your key?"

plus `gui/multisig.go:44` `Lead: "Supply or build a policy?"`, unique to
`engraveMultisigFlow`. And a decoy the recon does not name: `Title: "Engrave wallet
policy"` / `Lead: "Which md1?"` is **two** sites — `gui/multisig_build.go:121-122`
and `gui/singlesig.go:94-95` — so a stage author picking the obvious form screen as
the discriminator picks a shared one.

Left uncorrected, this premise is an argument for deferring the scaffolding past S2,
which is the opposite of what the rest of the recon concludes.

**Fix:** put the measured needle list (and the `singlesig.go` decoy) into F-169.

## Recommendation

**Option 2, with its scope and its rationale both rewritten**, and gated on C-1's
ruling landing first.

Option 3 is not the plan's to take: §4.5 is REQUIRED and quotes the operator
directly, so weakening it is an operator decision, not a planning one. Option 1 and
option 2 cost the same wall clock — the concurrency ceiling is 1 either way — so the
choice between them is about *where the machinery can first be seen to fail*, not
about size.

The continuity doc's stated reason for 2 is wrong and should not be the reason of
record: *"splitting them across stages means the byte comparison arrives at S2
having never been exercised at S1."* S1's walk produces **no artifact** (plan
`:194`), so the byte comparison cannot be exercised at S1 under any of the three
options. The real reason is stronger: **F-170 and F-171 have exactly one target
that exists today with known-correct expected outputs — S0's committed bundle
record** (`oracle/gaterecords/S0-trace-a.record.json`, six mk1 plates whose expected
strings are reproducible from `go run ./cmd/buildpayloadcards`). Build the census
derivation and the oracle comparison there, mutate an expected string, watch it go
red. Do it at S2 instead and the harness's first execution is also its first
verdict, which is the never-run-gate this cycle has now been burned by twice.

**S0b, scoped to what is shared AND exercisable now — nothing else:**

1. **F-169 driver + needle.** A reusable build-flow driver reaching the gather via
   `Engrave Multisig → Build policy`, asserting one of the measured single-site
   needles (I-4). Zero `shNFC.present` in any stage-gate run (I-1).
2. **F-170 census derivation** from a recorded input tuple, replacing
   `plates = 6`, exercised against the S0 record.
3. **F-171 oracle comparison** — `oracle` shells out to the pinned `md`/`mk`/`ms`
   and compares census strings byte for byte, exercised against the S0 record's six
   mk1s.
4. Its own gate, seen to fail: mutate one expected string, one plate digest, and one
   needle; each must go red, then green on restore — the same three-way proof S0's
   record carried.

**Explicitly NOT S0b:** the five per-stage walk scripts. Each stage still writes its
own, because the tail of the build flow cannot be walked before the code that makes
it walkable exists — today the payload seeds one chunk, S3's branch choice is F-172,
and S4's and S5's screens are not written. "S0b owns the scaffolding for all five"
as the continuity doc phrases it over-promises; owning the three shared mechanisms
does not.

### The case against my own recommendation

- It is a stage of pure test infrastructure inserted ahead of the feature, which is
  exactly what the plan's §4 refuses when it says letting the test sweep into S1 "is
  how S1 becomes a test-infrastructure project". Renaming it S0b does not change
  what it is, and it delays the first repair of a **field-observed** defect (D-1,
  still unreproduced) by a stage.
- Some of the driver will be rewritten at S1 anyway: the card source moves from NFC
  to payload, the needle set shifts when S2 fixes D-4, and the census shape changes
  from six mk1s to md1 chunks + mk1 + ms1. The genuinely shared surface may be
  thinner than three follow-ups make it look — F-171 is shared by S2 and S5 only.
- Option 1's real advantage is that scaffolding written at S1 is written against the
  flow it must drive, and cannot be built to a guessed shape. My recommendation buys
  the seen-to-fail proof by accepting exactly that risk for the driver half.

### Why I recommend it anyway

The two halves fail differently. The **driver** can be guessed wrong cheaply and
fixed at S1; the **comparison** cannot be exercised at all until something correct
has been engraved, and the only such thing that exists is S0's record. Split them
across S1 and S2 and F-171 debuts at S2 as a hypothesis judging the stage that
introduced it. There is also a schedulable bonus: an NFC-fed build run at S0b (a
smoke test by I-1's rule, not a gate) is the cheapest route anyone has to
**reproducing D-1**, which S2's test 1 requires to fail on unfixed code and which
`SPEC §2.2` still records as NOT YET REPRODUCED.

If C-1 is ruled the other way — the walks run n=5 and Trace A's shape is restated —
the recommendation does not change, only S0b's input tuple does.

## Out of scope, noted

- The cards payload holds one `ClassMnemonic` byte-equal to master A *and* card A@0
  derived from master A. Intended for S4's `both` case, but every walk that takes
  `FROM PAYLOAD` at seed entry then engraves master A's ms1 in full mode; S5's
  per-master ms1 comparison must expect that, not treat it as a duplicate.
- `oracle/gaterecords/S0-trace-a.inputs.json` records `"k": 0` and
  `"fp_choice": "unset"` for a flow that picks neither. Harmless now, load-bearing
  the moment F-170 derives a census from that tuple.
