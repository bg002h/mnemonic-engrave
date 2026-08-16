# S5 fold re-review — LENS: what did the FOLD break?

**Artifact:** the fold only — `git diff 7da66bd..830aaf7` on branch `s5-multislot`
(worktree `/scratch/code/shibboleth/wt-s5`, frozen at `830aaf7310b4d8870a5dd893d818afa625699e04`,
`git status --porcelain --branch` → `## s5-multislot` only, before and after this review).
Every probe ran in a `cp -a` copy at
`/tmp/claude-1000/.../scratchpad/probe`; the frozen tree was never written to.
**Date:** 2026-08-16.

---

## VERDICT

| | count |
| --- | --- |
| Critical | **0** |
| Important | **2** |
| Minor | 2 |
| Nit | 1 |

Both Importants are in text the FOLD authored, not in the base. Neither is a
re-litigation of a settled decision.

### Half 1 — did the fold CLOSE each finding, or MOVE it?

**Closed, all 17.** I re-derived the closure for the ones with real control-flow
or identity changes; the rest I confirmed by reading the shipped code.

| finding | closed? | how I checked |
| --- | --- | --- |
| C-1 | yes | The incomplete branch calls `verifyMultisigLegsPartial` **before** it reports (`gui/multisig_verify.go:874-877`), and the count is `len(checked)` where `checked` comes from the legs the comparator matched. Drove the real screens: a Trace-B readback with @0 replaced by a foreign `m/44'` mk1 now ends `"Noread-backkeyplatecarriesslot@0'skey…VerifyFailed"`, and an honest partial ends `"Checked2keyplates:@0and@1…"`. |
| C-2 | yes | `bound[seed.MasterFP]` (`gui/multisig_build_slots.go:428,487`). `MasterFP` is always populated: `seedRegistry.add` (`:177`) derives it and refuses the seed if it cannot, and `bindPassphrase` (`:189`) re-derives it. Ran the real projection on Trace B: `notices=["Slots @0 and @1 all come from your seed, at different key origins…"]`, and the same sentence is in the re-minted `oracle/gaterecords/S5-trace-b.walk.json` with `multiAccountNotice: true`. Zero-value `MasterFP` is a 2⁻³² event; collision is fail-safe (spurious notice), as documented. |
| C-3 | yes | `gui/multisig.go:217` `buildFullModeLabel(passphrase != "")` and `:359` `buildPlateInventoryLines(cardsOut, oneSeedPassphraseFact(passphrase != ""))`; `passphrase` is bound at `:139-149` and is not rebound before either site. |
| I-1, I-2, I-3, I-9, I-11, I-13, I-14 | yes | Read the shipped code and the new tests. `oracle/record_test.go:381-411` makes S5 a named `requiredStages` row; `multisigVerifyOKMessage`'s single-leg arm is now `full`-aware (`gui/multisig_verify.go:942-953`). |
| I-4 / I-12 | yes | `bundleEngrave` returns `bundleEngraveResult`; **all four call sites resolved**: `gui/multisig_build.go:402` and `gui/multisig.go:291` gate on it, `gui/bundle_flow.go:39` returns immediately after it so nothing downstream vouches for the set, and `gui/singlesig.go:127` is the already-filed F-197 (out of scope). `multisigVerifyFlow` has exactly two callers and both loop on the verdict. |
| I-5 | **closed for the case it was filed on, but the fold's own replacement has a new defect — see F-1 below.** |
| I-6 | yes, and it does not MOVE the dead end | Carried the newly reachable shape all the way past the point the review stopped at: a 2-of-2 held entirely, `chosen=nil`, `cosigners=nil` → `buildSlotSources` → `buildSlotGate` (nil err) → `assembleBuildPolicy` (stub `1613c69e`, 2 slots) → `buildSlotKeyStrings` → `buildEngraveTail` (2 legs, 4 cards, 10 plates). Nothing downstream mishandles the empty cosigner set. |
| I-7 | yes | `buildOriginAnnouncement(script, held)` reads origins off the assembled bytes via `buildSlotKeyStrings`' second return. Measured: `["Your key origins: @0 at m/48h/0h/0h/2h and @1 at m/48h/0h/1h/2h, the BIP-48 path for native segwit."]`. The re-minted walk record's `reviewScreen` carries the three-origin enumeration, and `cmd/emu/walk_trace_b.js` now **throws** on a missing origin rather than recording a flag. |
| I-8 | yes | Both operator-supplied strings landed **byte-identical** to `design/agent-reports/s5-i8-seed-residency-decision.md` §1 and §2 (diffed). `grep -rn "holds exactly one seed\|A seed you entered" gui/` → hits only in `gui/multisig_build_prose_test.go`, which asserts their **absence**. |
| I-10 | yes | The comment cites F-196; `grep -n "F-196" design/FOLLOWUPS.md` → `6979:### F-196 …`. |
| F-189 | yes | `multisigEngraveCards` gone; `findUserSlot` is `(int, bip32.Path, bool)`. Both compile-enforced. |

### Half 2 — did the fold INTRODUCE a defect?

Two Importants, below. Both are in code written **in this fold**, both are in an
operator-facing surface on the funds path, and both are invisible to the suite.

---

# IMPORTANT (2)

## F-1 — the restore document lists ONE seed as TWO seeds needing "DIFFERENT passphrases", on Trace B's own shape

**Site.** `gui/multisig_build_slots.go:259-267` (`seedRegistry.passphraseFacts`),
consumed at `gui/multisig_build_census.go:146-168`
(`buildPassphraseInventoryLines`), called from `gui/multisig_build.go:475`.
All three are new in commit `7a23bb5` (I-5's fold).

**Defect (one sentence).** `passphraseFacts()` emits one fact **per registry
entry** — i.e. per *held slot* — while the same fold re-keyed the gate on
`MasterFP` because a registry entry is **not** a seed identity, so a build where
one master fills two slots prints two "Needs a passphrase" lines carrying the
**same master fingerprint** and telling the reader they "may be DIFFERENT
passphrases".

**Concrete failing input (RUN).** Trace B, the branch's flagship shape: the
operator holds @0 and @1 from master A and @2 from master B, and types the same
passphrase at both of master A's per-slot prompts (`buildSeedForSlot` asks once
per held slot, `gui/multisig_build.go:619-635`). Driven through the production
chain — `reg.add` ×3 → `bindPassphrase` on ids 0 and 1 → `buildSlotSources` →
`buildSelfKeys` → `assembleBuildPolicy` → `buildEngraveTail` →
`buildPlateInventoryLines(cardsOut, reg.passphraseFacts())`:

```
$ nix develop --command go test ./gui/ -run TestZZProbeTraceBRestoreDoc -count=1 -v
FACT: label="your seed" fp=ca2c62d2 uses=true
FACT: label="your seed" fp=ca2c62d2 uses=true     <- ONE master, TWO facts
FACT: label="your seed" fp=b8688df1 uses=false

CENSUS: ms1 secret share 1 of 2: 1 plate           <- the tail cut TWO ms1s, one per master

INV: Needs a passphrase: your seed for @0 (master fingerprint ca2c62d2). If more than one
     is listed here they may be DIFFERENT passphrases; record each one against its fingerprint.
INV: Needs a passphrase: your seed for @1 (master fingerprint ca2c62d2). If more than one
     is listed here they may be DIFFERENT passphrases; record each one against its fingerprint.
INV: Needs NO passphrase: your seed for @2 (master fingerprint b8688df1).

gate notices=["Slots @0 and @1 all come from your seed, at different key origins…"]
```

(labels read `your seed` in the probe because `s5Registry` uses that label; the
flow's own label is `"your seed for @" + slot`, `gui/multisig_build.go:614`, and
that is what the INV lines above are rendered with.)

**Why it is Important.** The restore document is the artifact this diff itself
says is "read years later, alone, often by someone who was not the operator". It
now says **three seeds**, two of which "may be DIFFERENT passphrases", over a
census on the same page that says **two ms1 plates** and a Key-sources screen two
steps earlier — the notice C-2's fix just made fire — that says @0 and @1 come
from **one** seed. Two surfaces authored in the same fold state opposite
cardinalities for the same secrets. The instruction "record each one against its
fingerprint" is unfollowable when the two lines carry the same fingerprint, and a
reader who cannot find a second passphrase has to decide whether a fully
recoverable backup is unrecoverable. It is I-5's own harm — "silence about the
bare ones reads as *all of them need it*" — running in the other direction.

It is also, precisely, the review's **defect-class 1 standing remedy**: *"whenever
a dedupe or grouping key is chosen, grep every other site that groups the same
entities and prove they use the same identity."* The fold applied `MasterFP` to
`buildSlotGate` and left `passphraseFacts` keyed on registry position.

**Why nothing caught it.** `TestRestoreDocNamesEveryPassphrasedSeed`
(`gui/multisig_build_perseed_passphrase_test.go:52`) *Fatals* if the two entries
share a fingerprint (`:62-65`) — it is built specifically on the
distinct-fingerprint case. `TestRestoreDocSaysWhichSeedsNeedNoPassphrase` uses two
different masters, and `TestSingleSeedInventoryIsUnchanged` uses one
registration. The duplicate-master cell has no test. The committed Trace B walk
cannot see it either: `params.picks` is `["skip","skip","skip"]`, so no
passphrase is entered and the inventory takes the "No BIP-39 passphrase was used"
arm.

**Suggested fix (resolved against the call graph, not prescribed blind).** Group
`passphraseFacts()` on `MasterFP`, exactly as `buildSlotGate` now does — one fact
per distinct pair, carrying the labels of the entries in the group (e.g. `"your
seed for @0 and @1"`). That also fixes the `len(seeds) < 2` test at
`gui/multisig_build_census.go:146`, which currently sends a genuine ONE-seed
multi-account build down the multi-seed enumeration arm. The change is confined
to `passphraseFacts` plus that length test; `oneSeedPassphraseFact`
(`gui/multisig_build_census.go:200`) and the supply path are unaffected because
they already pass exactly one fact.

---

## F-2 — the incomplete screen's new instruction cannot reach a clean verify: the retry it names does not carry the first pass's coverage

**Site.** `gui/multisig_verify.go:459-472` (`multisigVerifyIncompleteText`, the
sentence at `:466`), against the retry loops the same fold added at
`gui/multisig_build.go:441-453` and `gui/multisig.go:325-337`, and against
`gui/multisig_verify.go:706-707` (`legs` and `covered` are locals of
`multisigVerifyFlow`, re-created on every invocation). All new in `9f93362`.

**Defect (one sentence).** The new screen tells the operator *"Choose VERIFY
AGAIN on the next screen and type **the remaining seed**"*, but each invocation
of `multisigVerifyFlow` starts with an empty `covered` map, so an operator who
does exactly that is told the slots they verified a minute ago are **NOT
verified**, and the loop cannot terminate in `Verify OK` by following its own
instruction.

**Concrete failing input (RUN).** Trace B, 3 key plates for {0,1,2}, honest
readback. First pass: type master A (covers @0, @1), STOP HERE → *"Checked 2 key
plates: @0 and @1 … 1 slot is NOT verified: @2 … Choose VERIFY AGAIN on the next
screen and type the remaining seed."* The operator presses **VERIFY AGAIN** and
types the remaining seed, master B. Driven through the real screens with the
fold's own driver:

```
$ nix develop --command go test ./gui/ -run TestZZProbeRetryTypingOnlyTheRemainingSeed -count=1 -v
verdict=1                                       (verifyIncomplete)
FINAL SCREEN: "Checked1keyplate:@2.Comparedagainsttheplatesyoupresented,andtheymatch.
               2slotsareNOTverified:@0and@1.Nothinghasbeenprovedaboutthoseplates.
               ChooseVERIFYAGAINonthenextscreenandtypetheremainingseed,ordonotfund
               thiswalletuntilyouhave.VerifyIncomplete"
```

`verifyIncomplete` re-arms the loop, so the operator is offered VERIFY AGAIN a
third time under the same instruction, with the same result. The only way out is
to re-type **every** seed in a single pass — which no screen says.

**Why it is Important.** It is a defect in the exact text the fold wrote to close
I-4, on the last screen before funding. It does not produce a false GREEN — the
report under-claims, which is the safe direction — but the device now issues an
instruction it cannot honour, and the operator's third reading of *"do not fund
this wallet until you have"* over plates that are in fact good is the state I-4
was filed to remove. The unsound assumption is in the new control flow: the
retry loop was built as a re-offer of the whole flow, and the sentence describes
it as a resume.

**Why nothing caught it.** Every new I-4/I-2/C-1 driver invokes
`multisigVerifyFlow` **once**. `TestBothEngraveFlowsReOfferTheVerify` asserts at
the call-site level that a loop exists and that the verdict is read; nothing
drives the loop round twice, so the discontinuity between iterations is unowned.

**Suggested fix (two options, both resolved against the call graph).** Either
(a) reword `:465-468` to say what the retry actually is — *"Choose VERIFY AGAIN
and type **every** seed, including the ones already checked; a new attempt starts
from nothing"* — which is a one-string change with no control-flow risk; or
(b) carry `covered`/`legs` across iterations, which means hoisting them out of
`multisigVerifyFlow` into the caller and is a real change to a funds-path
function. (a) is the honest minimum and is what the abort text's own I-11 fix
did for the same class of over-promise.

---

# MINOR / NIT (recorded, not blocking)

| # | Sev | Finding | Site |
| --- | --- | --- | --- |
| M-1 | Minor | I-6 made the zero-cosigner build reachable, and the Key-sources review it now reaches ends *"No slot claims to be both a seed and a card here, so nothing was cross-checked. **The cosigner keys are taken as supplied.**"* on a build that has no cosigner keys. Measured on the 2-of-2-held-entirely probe. The prose is pre-existing; the fold is what made it reachable, and the `else` arm at `:699-701` is unconditional. | `gui/multisig_build_slots.go:699-701` |
| M-2 | Minor | `multisigVerifyIncompleteText` is the only new modal body from this fold with **no** `assertModalBodyFits` guard — its two siblings from the same commit got one (`gui/multisig_verify_report_test.go:482`, `:569`). It fits today, measured: headroom 244 chars (2 checked / 1 outstanding), 201 chars (1 checked / 8 outstanding), against the 80-char margin. What is missing is the guard, not the fit. | `gui/multisig_verify.go:459` |
| N-1 | Nit | The enumerated origin announcement still ends in the singular: *"Your key origins: @0 at m/48h/0h/0h/2h and @1 at m/48h/0h/1h/2h, **the** BIP-48 **path** for native segwit."* The `MultisigShWsh` arm reads worse: *"…so this build derives your keys at @0 at m/… and @1 at m/…"*. Measured on the wsh path; the shWsh arm read from source. | `gui/multisig_build.go:1629-1648` |

**Out of scope, one line each, not investigated:** `gui/singlesig.go:127` ignores
the new `bundleEngraveResult` (already filed F-197) and `gui/singlesig.go:80`
hard-codes the mode label (already filed F-198).

---

# What I did NOT re-derive

* The machine baseline (`go test` exit 0 / 51 ok, `gofmt` clean, `go vet` exit 1
  with 40 test-only findings, oracle-live 7/7, emu 9972075 bytes) — taken as
  settled, not re-run, and no finding above depends on it.
* R-1 stays refuted. `git status --porcelain --branch` on the frozen tree printed
  the branch line only at the start of this review and again at the end.
* The 10 commits at or below `7da66bd`.
* I-8's ruling and I-6/I-13's departures — the decisions were not relitigated;
  only their implementations were checked, and all three landed as described.

**Gate position:** 0 Critical / 2 Important. Per the project's rule the branch
does not merge and nothing it produces is engraved until F-1 and F-2 are closed
and a re-review returns 0C/0I. Both fixes are small and neither needs a new
mechanism; F-1's is the one that touches shipped logic, and it should be pinned
by a test whose two registry entries **share** a fingerprint — the cell every
existing I-5 test excludes by construction.
