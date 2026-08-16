# S5 — round 2 re-review of the FOLD OF THE FOLD

**Artifact:** `git diff 830aaf7..s5-multislot` on `/scratch/code/shibboleth/wt-s5` — the three
commits `da4fa98`, `750296f`, `6088487` that answer round 1's B1..B5. The frozen tree was
**never written to**: every mutation ran in a `cp -a` copy under `/scratch/tmp-r2-mut/m1..m9`.

**Lens, and the only one:** *what did this fold break, and are its own mechanisms pinned?*
(a) trace every path through the changed verdict logic; (b) delete or invert each mechanism the
fold ADDED and confirm which test goes red.

---

# VERDICT: **1 Important, 0 Critical**

| | count |
| --- | --- |
| Critical | **0** |
| Important | **1** |
| Minor | 2 |
| Nit | 2 |

Question (a) came back **clean**: I found no new defect in the changed control flow. Question (b)
found one — B3's fix is correct in production at *both* of its ms1 arms and pinned at only *one*
of them, and the fold's own mutation set structurally could not tell them apart.

Nine mutant trees were built and run. **Two of the fold's five mechanisms have a hole; three are
pinned exactly as the fold claims, independently re-measured here rather than taken from its
report.**

---

# (a) The changed control flow — traced, nothing found

Read at `gui/multisig_verify.go:767` (`correctable := false`), `:859`, `:886`, `:937`, and the
two callers at `gui/multisig.go:330-342` / `gui/multisig_build.go:449-459`.

**`correctable` can only be set immediately before a `break`.** Its two assignment sites (`:859`
after the no-slot/covered-seed switch, `:886` at the ms1 exit) are each followed by `break` on the
next line, so no loop iteration can *continue* with it true. Consequences, each checked:

* **No genuinely abandoned verify can report incomplete.** The three abandon routes —
  Back at `bundleGatherFlow` (`:696`, `return verifyAbandoned`, never touches `correctable`),
  Back at `seedEntryFlowTypedOnly` (`:770`, `break` with `correctable` untouched), and Back at the
  ms1 keyboard (`:1008` returns `rejected=false`) — all still reach `verifyAbandoned` at `:940`.
  The third is asserted by `TestVerifyRetriesAfterACorrectableFirstSeed/Back_at_the_first_seed's_ms1_entry_still_abandons`,
  and I confirmed it is a real non-vacuity row, not decoration (below).
* **Every path that now returns `verifyIncomplete` with zero legs has drawn a remedy screen
  first.** All three arms of the `len(fresh)==0` switch call `showError` (`:830`, `:845`), and
  both `rejected=true` returns are preceded by a `showError` inside the helper (`:1012`, `:1017`).
  There is no path to `:938` that shows nothing.
* **The retry loop cannot re-offer with no way out.** Both loops are
  `sel, ok := verifyChoice.Choose(...); if !ok || sel != 0 { break }` — Back and the second row
  both leave, on every iteration, before the verify is re-entered. `lead`/`choices` are assigned
  after the first non-clean verdict and never reset, which is correct: from attempt 2 onward the
  screen is the retry offer.
* **No consumer branches wrongly.** `grep -rn --include='*.go' 'verifyIncomplete\|verifyAbandoned'`
  over non-test files returns exactly two production consumers, `gui/multisig.go:337` and
  `gui/multisig_build.go:453`, both of which only decide whether to loop.
* **The helper's new return value has exactly ONE caller.**
  `grep -rn --include='*.go' 'multisigVerifyMS1Entry' .` → 5 hits: the definition (`:1004`), the
  single call (`:868`), and three comments. No test calls it directly. So "check every caller of
  that helper" resolves to one site, and it consumes `rejected` correctly.

**B2 did NOT quietly implement the state-carrying version.** `legs`, `covered`, `typed` and
`correctable` are all function locals of `multisigVerifyFlow`; the only package-level `var`s the
whole file carries are `errVerifyNoLegs` (`:268`), `errVerifyNoExpectedSlots` (`:280`) and
`multisigVerifyFn` (`:660`) — a function value, not coverage state. The callers carry only `lead`
and `choices` across iterations. `bundleGatherFlow` is still called on every invocation at `:691`.
A "Verify OK" cannot be assembled from two readbacks.

---

# (b) Mechanism-by-mechanism, by mutation

Every row is a full `nix develop --command go test ./gui -count=1` in its own `cp -a` copy.

| # | mutation | result | caught by |
| --- | --- | --- | --- |
| M1 | `multisig_verify.go:1018` `return "", false, true` → `false` (the **DecodeMS1** rejection arm) | **GREEN, exit 0, `ok seedhammer.com/gui 154.884s`** | **nothing — FINDING R2-1** |
| M2 | both callers' `if res != verifyIncomplete && res != verifyFailed` → `if res == verifyComplete` | RED | `TestBothEngraveFlowsReOfferTheVerify` **only**, at `multisig_verify_report_test.go:1050`, in 0.00s — a `strings.Contains` over source |
| M3 | B1's group key `(Mnemonic, Passphrase)` → `MasterFP` | GREEN, exit 0 | nothing — but the fold **declared** this; see Minor 2 |
| M4 | `multisig_build.go:402-404` abort body → `_ = cardsOut` | RED | `TestBuildAbortIsTheLastScreenOfTheProgram` — B5 **confirmed** |
| M5 | `{"VERIFY AGAIN", "CONTINUE"}` → `{"CONTINUE", "VERIFY AGAIN"}`, both sites | RED | `TestBothEngraveFlowsDriveTheRetryLoop/supply` **and** `/build` — B4 **confirmed** |
| M7 | keep every string M2 broke; coerce `verifyAbandoned`/`verifyRefused` → `verifyIncomplete` before the check | **GREEN, exit 0, `ok seedhammer.com/gui 245.179s`** | nothing — Minor 1 |
| M8 | `multisig_verify.go:1013` `return "", false, true` → `false` (the **object-rejection** arm) | RED | `TestVerifyRetriesAfterACorrectableFirstSeed/the_first_seed's_hand-typed_ms1_is_rejected` — the positive control for M1 |

M4 and M5 are re-measurements of the fold's own claims, made here rather than inherited. Both
hold, and M5's failure message is the right one (`pressing the row LABELLED "VERIFY AGAIN" did not
run the verify a second time (entered 1 time(s))`, after `the supply run cut 14 plate(s)` /
`the build run cut 9 plate(s)`) — the walk really does reach the tail through a completed engrave.

---

# R2-1 (Important) — B3's fix is pinned at ONE of its two ms1 rejection arms; reverting the other leaves the suite green

**Defect.** `multisigVerifyMS1Entry` refuses a hand-typed ms1 in two different ways and B3's fix
sets `rejected = true` in both, correctly. Only the first is watched by an executing test.
Changing the second back to the pre-fold value restores round-1 B3's dead end verbatim, at a route
round 1 named explicitly, and **the whole `gui` suite stays green.**

**Site.** `gui/multisig_verify.go:1015-1018` —

```go
	_, _, ent, err := codex32.DecodeMS1(s)
	if err != nil {
		showError(ctx, th, "Verify Bundle", "That isn't a valid ms1 secret share.")
		return "", false, true          // <-- unwatched
	}
```

against the pinned twin twelve lines up at `:1010-1013` (`"That isn't an ms1 secret share."`) and
the consumer at `:886` / `:937`.

**The concrete failing input.** A BIP-93 k=2 SSS **share** typed at "Type ms1" on the first seed:

```
ms12namea320zyxwvutsrqpnmlkjhgfedcaxrpp870hkkqrm
```

(`codex32/codex32_test.go:58`.) `validateMStar` (`gui/codex32_polish.go:265-270`) hands
`inputCodex32Flow` a `codex32.String` for it, because `codex32.New` accepts it — checksum-valid,
valid share schema — so the **`isStr` arm is not taken**. `codex32.DecodeMS1` then refuses it, by
its own documented contract (`codex32/mspayload.go:31-33`: *"a K-of-N share carries an
SSS-evaluated point, not an m-format payload, and yields errMSBadPrefix/Length"*). This is the
route round 1's B3 named in its own trigger — *"a checksum-valid string DecodeMS1 refuses, e.g. a
k>0 SSS share"* — and it is a plausible operator error, since a shared backup's plates carry
shares and the unshared secret is a different string.

**Measured, in both directions.** I wrote one driver test using the fold's own
`s5DriveVerifyFirstSeedRefused` harness and ran it on an unmutated copy and on the M1 copy:

```
# /scratch/tmp-r2-mut/m9 (UNMUTATED)
nix develop --command go test ./gui -run TestZZR2Ms1ShareIsCorrectableToo -v -count=1
    last frame: "Thatisn'tavalidms1secretshare.VerifyBundle"
    verdict: 1  done=true                       <-- verifyIncomplete: the fix is CORRECT here
--- PASS

# /scratch/tmp-r2-mut/m1 (only :1018 reverted)
nix develop --command go test ./gui -run TestZZR2Ms1ShareIsCorrectableToo -v -count=1
    last frame: "Thatisn'tavalidms1secretshare.VerifyBundle"
    verdict: 4  done=true                       <-- verifyAbandoned: B3's dead end, back
--- FAIL
```

and the whole package with that same revert:

```
# /scratch/tmp-r2-mut/m1
nix develop --command go test ./gui -count=1
ok  	seedhammer.com/gui	154.884s     (exit 0)
```

**Why the fold's mutations could not see it, which is the brief's question.** The fold ran three
B3 mutations and all went red, but all three act on the **consumer**: `correctable = true` → false
at `:859`, `_ = rejected` at `:886`, and `correctable := true` at `:767`. `_ = rejected` disables
*both* production sites at once, so it cannot distinguish a helper that reports one arm from a
helper that reports two. The mutation that separates them is the one on the **producer**, and it
is the one nobody ran. Positive control that this is a real hole rather than a route no test
drives: **M8**, the identical revert on the sibling arm twelve lines up, goes RED
(`TestVerifyRetriesAfterACorrectableFirstSeed/the_first_seed's_hand-typed_ms1_is_rejected`). And
under M1 that same test still RAN and PASSED, all three rows:

```
# /scratch/tmp-r2-mut/m1
nix develop --command go test ./gui -run 'TestVerifyRetriesAfterACorrectableFirstSeed' -v -count=1
=== RUN   TestVerifyRetriesAfterACorrectableFirstSeed/the_first_seed_fills_no_slot
=== RUN   TestVerifyRetriesAfterACorrectableFirstSeed/the_first_seed's_hand-typed_ms1_is_rejected
=== RUN   TestVerifyRetriesAfterACorrectableFirstSeed/Back_at_the_first_seed's_ms1_entry_still_abandons
--- PASS: TestVerifyRetriesAfterACorrectableFirstSeed (0.08s)
```

**The harm, if it regresses.** An operator on the built-policy path who has just cut nine plates,
types their seed, and hand-types a share instead of the secret reads *"That isn't a valid ms1
secret share."* — an input they can retype — and is then handed the restore document headed
*"This backup is N plates … If any of them is missing, this backup is incomplete."* with no plate
verified and no retry offered. That is round-1 B3's shape exactly, and round 1 rated it Important.
The same reasoning that made B5 Important — *"the abort `return` is real and correct in
production, and nothing executes it"* — applies here without modification.

**Minimal fix, resolved against the real call graph.** Add a fourth row to the existing table in
`TestVerifyRetriesAfterACorrectableFirstSeed` (`gui/multisig_verify_report_test.go:420-490`); no
new harness, no production change. `s5FirstSeedExit` already carries everything needed:

```go
{
    name: "the first seed's hand-typed ms1 is a k>0 SHARE",
    exit: s5FirstSeedExit{
        phrase: fixtureMasterA,
        // BIP-93 k=2 share: codex32.New ACCEPTS it, so the isStr arm is not
        // taken; DecodeMS1 refuses it (codex32/mspayload.go:31-33).
        badMs1: "ms12namea320zyxwvutsrqpnmlkjhgfedcaxrpp870hkkqrm",
        needle: "isn't a valid ms1",
    },
    want:    verifyIncomplete,
    because: "the screen names the object as the wrong KIND of ms1, which is an " +
        "input the operator can retype",
},
```

I ran exactly this shape (as a standalone test with the same fields) on the unmutated tree: PASS,
verdict 1. On M1: FAIL, verdict 4. It kills the mutation and costs one table row. The premise
belongs beside it — assert `codex32.New(share)` succeeds and `codex32.DecodeMS1` errors, so the
row cannot silently degrade into a second copy of the `isStr` case if the vector is ever edited.

---

# Minor / Nit — recorded, not gating

**Minor 1 — the retry loop's *non*-looping half is still pinned only by a source grep, and B4's
own seam is what makes closing it cheap.** `gui/multisig.go:337`, `gui/multisig_build.go:453`.
The design decision stated at `gui/multisig_verify.go:82-86` — *"Refused and Abandoned must not
be [re-offered]"* — has no executing test. M2 looked like it was covered, but its only failure was
`TestBothEngraveFlowsReOfferTheVerify` at `multisig_verify_report_test.go:1050` in **0.00 s**, a
`strings.Contains` over the caller's source. M7 proves the point: preserving every grepped string
and coercing the verdict just before the check —

```go
		res := multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1)
		if res == verifyAbandoned || res == verifyRefused {
			res = verifyIncomplete
		}
		if res != verifyIncomplete && res != verifyFailed {
```

— leaves the tree GREEN (`ok seedhammer.com/gui 245.179s`, exit 0). An operator who presses Back
at the gather is then re-offered the verify on a loop. Harm is low, which is why this is Minor
rather than Important: nothing is mis-verified and CONTINUE still leaves. But this is the same
class B4 was raised for, half-closed: `s5StubVerifyFn(t, verifyAbandoned)` plus an assertion that
the offer is *not* redrawn is roughly ten lines against machinery this fold already built.
Owning phase: next cycle's implementation phase.

**Minor 2 — B1's key choice is confirmed unpinned, exactly as the fold declared.** M3 (group on
`MasterFP` instead of `(Mnemonic, Passphrase)`) is GREEN, exit 0. The fold said so in its own
report ("NOT PINNED, stated rather than implied") and the reasoning in the comment at
`gui/multisig_build_slots.go:274-283` is correct — the merge SUPPRESSES a sentence, so it must not
key on a 4-byte surrogate. Recorded as measured rather than as a finding, because exhibiting a
collision costs 2^32 work and the fold disclosed it. What would close it is a unit test on
`passphraseFacts` with two hand-built `registeredSeed`s sharing a `MasterFP` and differing in
`Mnemonic` — no derivation needed, the struct is assignable in-package.

**Nit 1 — `correctable = correctable || rejected` at `gui/multisig_verify.go:886` has a dead
left operand.** `correctable` is false at that point on every path, because its only other
assignment (`:859`) is followed immediately by `break`. Harmless and arguably defensive, but it
reads as though the flow can accumulate correctability across iterations, which it cannot.

**Nit 2 — `verifyIncomplete`'s doc contract at `gui/multisig_verify.go:91-92` is now wrong at two
sites, one of them new.** It reads *"a partial verify: what was compared MATCHED"*; B3's
`:938` returns it having compared nothing. **Pre-existing, not introduced:** `git show
830aaf7:gui/multisig_verify.go` already returns `verifyIncomplete` at its line 694 (the mk1-count
refusal) with nothing compared. The fold widens an imprecision rather than creating one. One
sentence on the const would settle it.

---

# Checked and CLEAN — recorded so a later round does not re-spend on them

* **B1's merged label is not a rendering hazard.** The joined label grows with held slots
  (`joinAnd`, `gui/multisig_build_payload.go:431`), and the restore doc pages by LINE and always
  draws a page's first line regardless of height (`gui/singlesig_restore.go:156-160`), so an
  over-tall line would have an unreachable tail. Measured with `restoreDocScreen`'s own geometry
  on `sh2DisplaySize`: page height **224 px**; the merged passphrase line is **77 px** at 2 and 3
  held slots and **95 px** at 4 and 5 (n is capped at 5 — `multisigNChoices`,
  `gui/multisig_build.go:803`). Pre-fold, the same 5-slot build drew five separate 77 px lines.
  The merge makes the document strictly shorter. No overflow at any reachable size.
* **B1 cannot merge two distinct secrets.** Two entries merge only on equal `Passphrase` and
  `slices.Equal` mnemonics, i.e. the same derivation unit, which necessarily shares `MasterFP`
  (`seedRegistry.add` derives it, `gui/multisig_build_slots.go:178-188`). The `groups` table
  aliases the registry's mnemonic slice rather than copying it, so `scrub()`
  (`gui/multisig_build_slots.go:322`) still reaches it, and the table does not escape the
  function — no new secret lifetime.
* **`buildPassphraseInventoryLines`' `len(seeds) < 2` early return composes correctly with the
  merge.** After merging, one fact means exactly one secret in the whole build, so the singular
  arm's prose is true; a bare seed is a different derivation unit and keeps the count at 2, so the
  enumeration arm still runs. Both cells are asserted, in different tests
  (`TestRestoreDocNamesEveryPassphrasedSeed` row 2 with `wantStatements: 0`, and
  `TestRestoreDocMergesOneSeedHeldAtTwoSlots`).
* **The B4 seam is not a concurrency hazard.** `grep -rn 't\.Parallel()' gui/` → **0**, and
  `s5StubVerifyFn` restores through `t.Cleanup` (`gui/multisig_engrave_tail_walk_test.go:114`).
* **`s5RowIndexOf` fails safe.** A label it cannot locate is a `t.Fatalf`; a mis-ranked label
  presses the wrong row, which lowers the call count and fails. It has no false-PASS shape.

---

# Facts inherited as SETTLED, and not re-derived

The five build gates on the current head; `go vet` exit 1 / 40 / 0-outside-`_test.go`; R-1
refuted; I-8 ruled (b); `gui/singlesig.go` out of scope with F-197/F-198 filed; F-199, F-200,
F-201 filed; the gate record needing no re-mint. None of these was re-opened, and R2-1 is not a
restatement of F-199 — F-199 is `verifyRefused` at `:701`, a *different verdict at a different
site*, and this is `multisigVerifyMS1Entry`'s second rejection arm, where the fold's fix is
already present and merely unwatched.

---

*Round 2. Nine mutant trees built and run; 7 mutations reported above. 1 Important, 0 Critical.*
