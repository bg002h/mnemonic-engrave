# S6a step 5 (+ step 6) — implementation report

**Agent:** single implementer, S6a build order step 5.
**Worktree:** `/scratch/code/shibboleth/wt-s6a`, branch `s6a-singlesig-truth`.
**Parent:** `7d317ff` (step 4). **Commit:** `1a35b663f47ef48a1ea9ac6fb03337196c76a3e2`
— one commit, steps 5 **and** 6 together.
**Verdict:** DONE. Full suite green, every mutation check genuinely red.

`/scratch/code/shibboleth/seedhammer` was not touched: still clean at `b8a23bf`.

---

## 1. Files changed

| file | what |
| --- | --- |
| `gui/singlesig.go` | **production.** The mode label (§4.1), the pre-engrave census (§4.6), the abort gate (§4.5), the restore-document inventory (§4.2) |
| `gui/singlesig_truth_test.go` | **+545 lines, appended.** T1, T2, T3, T5, T6 (step 5) and T7c (step 6), plus the walk harness they share |
| `gui/singlesig_flow_test.go` | step 6: `TestEngraveSingleSigFlowFull`, `TestEngraveSingleSigFlowWatchOnly` — one press each |
| `gui/template_engrave_test.go` | step 6: `TestEngraveSingleSigFlowTemplate` — one press |

Not touched, deliberately: `gui/bundle_flow.go` (the false comment is **step 8**),
`gui/multisig.go` / `gui/multisig_build.go` (no verify-status wiring — **step 7**),
`design/SPEC_seedhammer_T6a_singlesig_flagship.md` (**step 9**).
`verifyStatusNotFullyCheckedLine` stays at the single-sig call site as step 4 left it.

The production diff is four hunks:

```go
Choices: []string{buildFullModeLabel(passphrase != ""), "Watch-only (keys)"},
...
cards := singleSigEngraveCards(b, full)

if !confirmReviewScreen(ctx, th, "Plates To Cut", buildPlateCensusLines(cards)) {
    return
}

if bundleEngrave(ctx, th, "Engrave Single-Sig", cards) != bundleEngraveDone {
    return
}
...
restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path,
    verifyStatusNotFullyCheckedLine,
    buildPlateInventoryLines(cards, oneSeedPassphraseFact(passphrase != ""), seedCapacityOne))
```

---

## 2. THE F-198 FIX, DEMONSTRATED

**The premise, re-measured on this tree** (not transcribed from the plan — a
throwaway test called `deriveSingleSigBundle` twice with the same twelve words at
BIP-84/mainnet, and was deleted afterwards):

```
passphrase=""              masterFP=73c5da0a ms1=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
passphrase="abandon about" masterFP=fc60c6df ms1=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
```

**Byte-identical ms1, different wallet.** Both fingerprints then appear
independently in the executing walks below (`fc60c6df` on the passphrase run,
`73c5da0a` on the bare one), so the premise and the fix are observed on the same
artifacts.

### What a passphrase single-sig run says NOW that it did not say BEFORE

| surface | before step 5 | after step 5 |
| --- | --- | --- |
| engrave-mode row 0 | `Full (seed + keys)` | `Full (seed + keys, NOT passphrase)` |
| pre-engrave | *(nothing — no count at all)* | `Plates To Cut` + `This engraves 6 plates.` + per-card census |
| after an abort | verify offer, then a restore document headed `This backup is …` | the abort modal is the last screen; the program ends |
| restore document | master fp, descriptor, 2 addresses. **No plate count, no completeness claim, no passphrase fact.** | +8 lines (§3 below), including `A BIP-39 passphrase WAS used.` and `Without it, these plates do not reach the money.` |

The pre-step document is not a claim: it is the exact rendered text captured when
T2's own mutation (`extra == nil`) was applied —

```
Restore Doc  These plates were not fully checked. …  Master fp: fc60c6df
Descriptor: wpkh([fc60c6df/84h/0h/0h]xpub6DV2HFGDNQvb…)#mjlfsmvg
First receive: bc1q53z9ajuc9hnqjww7r3rcqyycz8cjtm9hc0urc2
First change:  bc1q69e6e2uqdve2f5nzupw09ag85ux9p4632dggx5
```

— and nothing else. §1.1's measurement of the shipped document is exactly right.

---

## 3. EXACT STRINGS EMITTED, per mode and per path

All measured by calling the production builders over
`singleSigEngraveCards(b, full)`, and all four combinations are also reached
through the real screens by the walks in §5. ASCII only; no glyph the body face
lacks.

### 3.1 The mode label — `gui/singlesig.go`, engrave-mode picker

| run | row 0 | row 1 |
| --- | --- | --- |
| no passphrase | `Full (seed + keys)` | `Watch-only (keys)` |
| passphrase | `Full (seed + keys, NOT passphrase)` | `Watch-only (keys)` |

Row 1 is byte-unchanged. The label draws in full on the real panel
(`sh2DisplaySize`) — see the finding in §7.2.

### 3.2 The census — title `Plates To Cut`, body `buildPlateCensusLines(cards)`

**FULL (ms1 + mk1 + md1):**

```
This engraves 6 plates.
ms1 secret share: 1 plate (secret seed backup)
mk1 key: 2 plates (account key card)
md1 descriptor: 3 plates (wallet policy descriptor)
Each plate takes minutes to cut. Have that many blanks ready before you start: a set is only a backup when all of it exists.
```

**WATCH-ONLY (mk1 + md1):**

```
This engraves 5 plates.
mk1 key: 2 plates (account key card)
md1 descriptor: 3 plates (wallet policy descriptor)
Each plate takes minutes to cut. Have that many blanks ready before you start: a set is only a backup when all of it exists.
```

Title is `"Plates To Cut"`, matching the other front-door path (§3.1.5), **not**
the build path's — see §7.1.

### 3.3 The restore-document inventory — `extra`, appended at the tail

**FULL, no passphrase (8 lines):**

```
This backup is 6 plates:
ms1 secret share: 1 plate (secret seed backup)
mk1 key: 2 plates (account key card)
md1 descriptor: 3 plates (wallet policy descriptor)
If any of them is missing, this backup is incomplete.
Seed: this set contains YOUR seed, on the plate marked 'ms1 secret share'. Treat that plate as the secret itself.
No BIP-39 passphrase was used, so no passphrase is needed to spend from this wallet.
Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- stays in device memory until the build ends, and on a full build the words are also on the plates as they are cut. Do not leave a mid-build machine unattended: the plates are the secret. Power the device off when you are done.
```

**FULL, passphrase (9 lines)** — lines 0–5 identical, then:

```
A BIP-39 passphrase WAS used. It is not on these plates and cannot be recovered from them: nothing this device engraves carries a passphrase.
Without it, these plates do not reach the money. Keep it somewhere separate, and make sure whoever needs this backup can also get the passphrase.
Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- stays in device memory until the build ends, and on a full build the words are also on the plates as they are cut. Do not leave a mid-build machine unattended: the plates are the secret. Power the device off when you are done.
```

**WATCH-ONLY, no passphrase (7 lines):**

```
This backup is 5 plates:
mk1 key: 2 plates (account key card)
md1 descriptor: 3 plates (wallet policy descriptor)
If any of them is missing, this backup is incomplete.
Seed: this set contains NO seed. It is watch-only: it records the wallet, but it can never spend. If funds must be recovered, the seed words must come from somewhere else -- no plate in this set holds them.
No BIP-39 passphrase was used, so no passphrase is needed to spend from this wallet.
Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- stays in device memory until the build ends. Do not leave a mid-build machine unattended: it is still holding seed material. Power the device off when you are done.
```

**WATCH-ONLY, passphrase (8 lines)** — lines 0–4 identical, then the two
`WAS used` lines, then the same watch-only ruling.

Note both axes of §4.3 firing as designed: the **subject** is
`The seed you entered -- this build holds exactly one --` in all four (capacity =
the path), and the **plates clauses** appear only when `bundleSetCarriesASecret`
(a fact of the run). The watch-only document never says "the plates are the
secret" four lines under "no plate in this set holds them".

### 3.4 The abort gate — no new string

F-197 adds no text. It makes the **existing** `bundleAbortWarning` modal
(`Bundle Incomplete`) the last screen of the program. `bundleAbortWarningText`
already keys on `bundleSetCarriesASecret(cards)`, so:

- **full** set → `… A re-run RE-CUTS the seed plate. Any plate with your seed on it that you are not keeping must be DESTROYED, not binned: cut it up or grind the words off.`
- **watch-only** set → `… No plate in this set carries a seed.`

What changed is what comes *after* it: nothing.

### 3.5 What each PATH now hands `buildPlateInventoryLines` (T7c's subject)

| path | call site | capacity | ruling subject |
| --- | --- | --- | --- |
| single-sig | `gui/singlesig.go` (the ninth site, new) | `seedCapacityOne` | `The seed you entered` |
| multisig supply | `gui/multisig.go` (step 3) | `seedCapacityOne` | `The seed you entered` |
| multisig build | `gui/multisig_build.go` (step 3) | `seedCapacityMany` | `Every seed you entered` |

---

## 4. Did step 6's walks land here, and why

**Yes — same commit.** Step 5 alone leaves the tree **RED**, and the brief
forbids committing a red tree.

The mechanism is exactly as §5.1(b) predicted: `confirmReviewScreen`
(`gui/multisig_build.go:1727`) loops `for !ctx.Done` until Button1/Button3/Center,
and `pumpUntil` (`gui/slip39_polish_test.go:353`) **only pumps frames — it never
presses**. So each walk parks on the census for its whole frame budget and hits
its `t.Fatalf`. Confirmed by running T6's own mutation, which showed the reverse
case: with the census removed the flow runs straight into
`Choose engraving … Card 1 of 3 | Plate 1 of 1` for 96 consecutive frames.

**The repair is one press each, and nothing else:**

```go
if c, ok := pumpUntil(frame, "Plates To Cut", 64); !ok {
    t.Fatalf("the plate census was not shown before the engrave; got %q", c)
}
click(&ctx.Router, Button3)
// then the existing pumpUntil("Card 1 of N")
```

**Nothing was weakened.** All three still reach the screen they were written to
reach, with their needles byte-unchanged:

| walk | needle after repair | result |
| --- | --- | --- |
| `TestEngraveSingleSigFlowFull` | `Card 1 of 3` | PASS 0.05s |
| `TestEngraveSingleSigFlowWatchOnly` | `Card 1 of 2` | PASS 0.04s |
| `TestEngraveSingleSigFlowTemplate` | `Card 1 of 3` (after `TEMPLATE-ONLY md1` + `sortedmulti`) | PASS 0.05s |

The `Card 1 of 3` / `Card 1 of 2` distinction — §5.1(b) calls it the only
executing assertion in the tree that full mode puts the seed plate on steel and
watch-only does not — is untouched. The census press is a **confirm** (Button3),
not a Back, so the route past it is the one the walks were driving.

The fourth `engraveSingleSigFlow` driver, `TestEngraveSingleSigFlowSeedScrubbed`,
aborts at the wallet-type picker and never reaches the engrave — unaffected, and
verified PASS. `gui/singlesig_program_test.go` never enters the flow.

**T7c landed here too**, per §4.8 step 6.

**Step 7 was NOT done**, per the brief. Note for the controller: §4.8's paragraph
"steps 5+6 without 7 … are exactly C-1's harm" **no longer holds as written**,
because step 4 placed `verifyStatusNotFullyCheckedLine` at all three call sites.
The document at this commit carries a full inventory *and* the conservative
status line — it does not vouch for an unverified set. The harm §4.8 describes is
"no verification status line on it", and there is one. This is a plan text that
step 4's own design superseded; recorded, not acted on.

---

## 5. Per-test evidence

Every row was proved by applying **the mutation its §5 row names** to an
otherwise-complete tree and watching that row go red — then reverting. In every
case the mutated line is shown to have **run**, by the rendered text in the
failure output.

### T1 — the passphrase run's mode screen says `NOT passphrase`

`TestSingleSigPassphraseRunTellsTheOperatorWhatIsMissing`. Drives
`engraveSingleSigFlow` with a payload-borne passphrase (`s5PassphraseRecord`),
FULL mode, through **all 6 plates**, to the restore document.

- **GREEN:** PASS 34.97s. Mode screen: `"Whattoengrave?Full(seed+keys,NOTpassphrase)Watch-only(keys)EngraveMode"`.
- **MUTATION** (`revert :80 to the literal`) → **FAIL 34.83s**:
  ```
  the single-sig engrave-mode picker calls a PASSPHRASE build "Full (seed + keys)":
      "Whattoengrave?Full(seed+keys)Watch-only(keys)EngraveMode"
  ```
  The mutated line ran: the reverted literal is what the picker drew.

### T2 — the document carries the passphrase fact and the inventory

Same walk. Asserts `BIP-39 passphrase WAS used`, `This backup is`,
`do not reach the money`, `this set contains YOUR seed`, on **every page** of the
pager (`s5PageForNeedle`) — the inventory is at the tail, past the descriptor
chunks and both addresses.

- **GREEN:** the 9-line inventory renders (full text in §3.3).
- **MUTATION** (`pass nil as extra`) → **FAIL 35.29s** on **four** assertions;
  the rendered document collapses to exactly the four pre-cycle lines quoted in
  §2. The mutated line ran: the document was drawn, master fp `fc60c6df`.

### T3 — non-vacuity: the bare run does not cry wolf

`TestSingleSigBareRunDoesNotCryWolf`. Drives the bare run to the mode picker (no
plates cut), then asserts the document half on `buildPassphraseInventoryLines`
per §5.2.

- **GREEN:** PASS 0.02s. Mode screen: `"Whattoengrave?Full(seed+keys)Watch-only(keys)EngraveMode"`.
- **MUTATION** (`make buildFullModeLabel always return the passphrase arm`) →
  **FAIL 0.07s**:
  ```
  a single-sig build with NO passphrase is labelled as though a factor were missing:
      "Whattoengrave?Full(seed+keys,NOTpassphrase)Watch-only(keys)EngraveMode"
  ```
  **This row initially had a FALSE-PASS half — see §7.2.** It is reported there
  because the fix is a test-harness lesson, not a code change.

### T5 — after an abort, the program ends

`TestSingleSigAbortIsTheLastScreenOfTheProgram`. Presses through the census,
reaches `Choose engraving`, presses Back, **asserts it saw `Bundle Incomplete`**
(blind spot 3), dismisses it, then asserts none of
`Verify the engraved plates?` / `This backup is` / `Descriptor:` is drawn and the
flow returns. No engraver needed.

- **GREEN:** PASS 0.04s.
- **MUTATION** (`drop the != bundleEngraveDone guard`) → **FAIL 0.19s**:
  ```
  the program did not end after the abort; it drew:
      "Verifytheengravedplates?VerifynowSkipVerifyBundle || …"
  ```
  That is F-197's harm, verbatim: an aborted set reaching the verify offer.

### T6 — the census comes before the engrave

`TestSingleSigShowsThePlateCensusBeforeTheEngrave`. Keeps **every** frame
(`s6aPumpCollecting`), so the claim is ordering rather than presence: no frame
before the census may carry `Choose engraving` or `Card 1 of`. Then asserts
`This engraves 6 plates` on the screen, and pages (Button2) for
`have that many blanks ready` — which is **not on page 1**; the census is a pager
and page 1 ends mid-inventory.

- **GREEN:** PASS 0.04s.
- **MUTATION** (`remove the census call`) → **FAIL 0.03s**: the collected frames
  are 96 × `ChooseengravingTEXT+QRTEXTONLYQRONLYCard1of3|Plate1of1`. The mutated
  line's position ran; the flow went straight past it into the engrave.

### T7c — the capacity WIRING, per path (step 6)

`TestSeedHandlingRulingMatchesEachPathsCapacity`, three sub-tests, each driving a
**production flow to a rendered restore document**:

| arm | flow | plates cut | asserts |
| --- | --- | --- | --- |
| single-sig | `engraveSingleSigFlow`, watch-only | 5 | `The seed you entered` present, `Every seed you entered` absent; plus `this set contains NO seed` present and `the plates are the secret` absent |
| multisig-supply | `s5SupplyPassphraseWalk` | 7 | `The seed you entered` present, `Every seed you entered` absent |
| multisig-build | `buildMultisigPolicyFlow` (raster harness) | 9 | `Every seed you entered` present, `The seed you entered` absent |

- **GREEN:** PASS 125.82s (31.5 / 46.9 / 47.5).
- **MUTATION** (swap the capacity argument at **all three** call sites) →
  **FAIL 119.58s**, and **all three arms fail on both halves**. Each failure
  prints the rendered document, so each mutated call site is shown to have run:
  the single-sig doc carries `Masterfp:73c5da0a`, the supply doc
  `P2WSH2-of-2multisig(sorted)`, the build doc `P2WSH2-of-3multisig(sorted)`.

This is the row §8.4 says is required and that nothing else covers.
`TestSeedResidencyRulingDescribesTheMultiSeedReality` passes
`seedCapacityMany` **explicitly** and is therefore blind to a mis-wired call site
— it stayed green under all three swaps.

### Assertions are behavioural, not source-text

Per the step-4 warning, **no new row asserts by searching source**. Every one
drives screens and reads what was drawn. The one exception in the file
(`TestRestoreDocStatusPlaceholderCannotStrengthenTheDocument`, step 4's) already
strips comments before matching and is untouched.

---

## 6. The build gate

Run from the worktree at the committed bytes, `PATH` re-exported, **stdout and
stderr in separate files**:

| command | exit | stdout | stderr |
| --- | --- | --- | --- |
| `go build ./...` | **0** | empty | only `warning: Git tree … is dirty` |
| `go test ./... -count=1` | **0** | every package `ok`; no `FAIL`, no `---` line | only the nix dirty-tree warning |
| `gofmt -l .` | **0** | **empty** | only the nix warning |
| `./cmd/emu/build.sh` | **0** | `built emu.wasm (9981389 bytes)` | only the nix warning |

**Baseline, measured before any edit at `7d317ff`:** `go test ./... -count=1`
exit 0, every package ok, stderr empty, `gui 311.247s`.
**After:** exit 0, every package ok, stderr empty, `gui 455.491s`.

**Cost: +144 s on `gui`**, essentially all of it T7c's three end-to-end walks
(126 s) plus T1/T2's passphrase walk (35 s). Flagged rather than hidden: T7c is
the most expensive single test this cycle has added, and §8.4 is why it exists.

`go vet ./...` still reports the **pre-existing, unrelated**
`gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires
go1.26 or later` and nothing else from the touched packages. Left alone, as
instructed.

---

## 7. Findings

### 7.1 A COMMENT quoting the build path's census title turned `cmd/emu` RED (found by the gate, fixed here)

The first draft of the census comment in `gui/singlesig.go` spelled the build
path's census title in prose, to explain why single-sig does **not** use it. The
full suite came back:

```
--- FAIL: TestBuildFlowNeedlesHaveExactlyOneProductionSite
    needle_test.go:166: needle "Plate Count" has 2 production site(s), want exactly 1:
      gui/multisig_build.go
      gui/singlesig.go
```

`cmd/emu/needle_test.go`'s counter matches **source bytes, comments included**
(F-184). So a comment costs a needle its uniqueness exactly as a second screen
does. `gui/multisig.go` carries that warning **verbatim**, I read it while
writing this step, and I committed the error anyway. Fixed by rewording the
comment so it does not spell the string, and the note about *why* it may not is
now in the comment itself. `cmd/emu` re-run: `ok 1.484s`, and the whole suite was
re-run at the fixed bytes.

Worth the controller's attention for two reasons: (a) it is the second
independent confirmation that the "don't quote a needle" rule is not learnable by
being written down next door; (b) `needle_flow_test.go`'s AST counter would
**not** have caught it (comments vanish under `go/parser`), which is exactly why
the plan's own §6 keeps both counters side by side.

### 7.2 T3 had a FALSE-PASS half until the walk moved onto the real panel

T3's primary claim is that a bare run's label does **not** contain
`NOT passphrase`. Driven on `newPlatform()`'s default (smaller) display, the
mutated label draws as:

```
"Whattoengrave?seed+keys,NOTpassphWatch-only(keys)EngraveMode"
```

— **truncated mid-word**, because `widget.Label` does not wrap. `uiContains(mode,
"NOT passphrase")` therefore could not see the clause it exists to forbid, and
the mutation was caught only by the *other* assertion in the row. The negative
assertion was vacuous.

Found by running the row's own mutation — not by reading. Fixed by driving T3 on
`sh2DisplaySize`, the real machine, which is also the panel
`assertChoiceLabelFits` budgets the label against
(`gui/multisig_build_prose_test.go:508-520`). Re-run under the mutation, the row
now fails on its own named assertion.

**No production defect:** the SH2 is the machine, the label fits there, and the
existing fit gate already measures it there. But the trap generalises — *any*
label assertion driven on `newPlatform()`'s default display can be silently
truncated to nothing, and it will look like a passing negative.

### 7.3 §4.5 and §4.8 disagree about who corrects the `bundle_flow.go` comment

§4.5 says the false comment at `gui/bundle_flow.go` (*"both engraving callers now
gate it…"*) *"is corrected in the same change"* as the abort gate — i.e. here.
§4.8 step 8 and §4.8b schedule it to **step 8**, with T8. I followed §4.8 and the
brief, and left `gui/bundle_flow.go` untouched. **The comment is still false at
this commit** and step 8 still owns it.

One knock-on, so step 8's T8 is not tripped by my own text: the new comment in
`gui/singlesig.go` explaining this gate deliberately does **not** contain the
phrase `both engraving callers`; it says "a fix described as covering every
engraving caller covered two of the three".

### 7.4 §8.2's blind spot is now discharged, cheaply

§8.2 says no test asserts the *last* page of the single-sig document is
reachable, and invites the implementer to close it if cheap. Both new walks page
the document with `s5PageForNeedle` until the pager wraps and assert on content
that is the **last line** (the seed-handling ruling, T7c) and the tail block
(T1/T2). So the last page is now proved reachable on the single-sig document
specifically. Recorded rather than claimed as new work.

### 7.5 §8.7's template branch is now VISIBLE on the document — reported, not built

`singleSigEngraveCards` hard-codes `summary: "wallet policy descriptor"`
(`gui/singlesig_engrave.go`). With the inventory wired, a **template-only**
single-sig engrave now prints, on the census *and* the durable document:

```
md1 descriptor: 3 plates (wallet policy descriptor)
```

over a **keyless** template md1, and prints a full restore document built from
the live xpub — where the build path skips the document entirely for a template.
§8.7 named both consequences as non-funds-losing before this step; step 5 makes
the first one *printed* rather than latent. `templateWarningLines` still states
the recovery dependency, and `TestEngraveSingleSigFlowTemplate` passes.

**Not fixed here.** The plan specifies no wording for a template card summary,
and inventing one is authorship outside the step. Filed for the controller.

### 7.6 §8.6 — the inventory is still dropped on `restoreDocFlow`'s error returns

`gui/singlesig_restore.go` `showError`s and returns before `restoreDocScreen` on
either error, so on that path the operator gets no plate count, no seed statement
and **no passphrase statement**, after every plate is on steel. §8.6 says hoist
it if cheap, file it if it needs restructuring.

**It needs restructuring**, so it is filed. The inventory is built by the *caller*
and arrives as `extra`; hoisting it means changing what `restoreDocFlow` renders
on its error arms, and `multisigRestoreDocFlow` has the identical shape — a
shared-seam behaviour change, in a step whose scope is single-sig wiring.
Reachability is unchanged from before this cycle.

### 7.7 Things I thought of and did not build (NG1's guard)

- **The verify-OK notice on a passphrase run** (§8.8) is still
  `"Verify OK" / "The engraved bundle matches the seed."` — true and incomplete,
  and it is now bracketed by two corrected surfaces. Adding a passphrase clause
  there would be new epistemic reporting; §0.1's guard says report, not build.
- **A "which plate is which" claim.** The inventory names `ms1 secret share` as
  the secret plate; it says nothing about whether the mk1+md1 alone can restore
  watch-only. True and useful, specified nowhere. Not built.
- **Asserting the census count against `bundlePlatePlan`** rather than the
  literal `6`. T6 pins `This engraves 6 plates` because a derived expectation
  computed in the test is a term the walk supplied (F-170's shape). Recorded so
  the literal is not read as laziness.
- **F-203** (`Plate Count` vs `Plates To Cut`) is untouched, as §7 of the plan
  requires. Single-sig now makes `Plates To Cut` a **two-site** string
  (`gui/multisig.go`, `gui/singlesig.go`); it is in no needle list and no test
  pins its count, and the full suite is green with it.

---

## 8. What a reviewer should check first

1. §3.3's four line-sets against §4.2/§4.3/§4.4 — the whole document is there.
2. The three repaired walks (§4): one press added, needles unchanged.
3. T7c's mutation output — three arms, both halves each, three rendered documents.
4. §7.1 and §7.2 — the two defects this step found in its own work, both by
   executing rather than reading.
