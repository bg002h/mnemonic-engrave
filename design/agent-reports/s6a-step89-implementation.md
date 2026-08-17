# S6a steps 8 and 9 — implementation report

**Implementer:** single implementer for steps 8+9, the last two of the S6a build order (plan §4.8).
**Base:** worktree `/scratch/code/shibboleth/wt-s6a`, branch `s6a-singlesig-truth`, at `bf453671b82ed066785e1bde1bc064660a25ccea` (step 7), clean.
**Result:** DONE. Suite green, `gui` 405.798 s against the 600 s ceiling.

---

## 1. Commits

| step | repo | SHA | subject |
| --- | --- | --- | --- |
| **8** | `/scratch/code/shibboleth/wt-s6a` (`s6a-singlesig-truth`) | `b2301d6183073d9f6a307729b587fee7047aba38` | S6a step 8: the comments say what the code does, and T8 keeps one of them honest |
| **9** | `/scratch/code/shibboleth/mnemonic-engrave` (`master`) | `fa7a29a094e5a1ef72a5e06176067224601b15d4` | S6a step 9: the spec's restore document gains what the code now prints |

**A third commit, in the design repo, ahead of step 9:** `a7b27a10e4128a0e9fffeb4f2186b916c65a63a1` — *reports: persist the S6a step-8 comment re-sweep, verbatim*. It carries the dispatched sweep agent's report, in its own commit, per the standing rule. §4.8 requires step 9 to be its own commit and it is; the report commit is a separate artifact and does not mix with either.

Nothing was pushed, merged, or rebased. `/scratch/code/shibboleth/seedhammer` was not touched — still `main` at `b8a23bf`, clean.

---

## 2. Step 8 — the three false comments §4.8b named

Plan line numbers are from `main` (`b8a23bf`). **All three happened to be un-moved by steps 3–7**, so the plan's numbers were still live in the worktree; I re-located each by CONTENT anyway, and both the pre-edit and post-edit lines are given below.

### Comment 1 — `gui/bundle_flow.go`, edited at **:535** (pre) → **:544–561** (post)

The false clause sat on line 535 before the edit and the corrected count clause is on line 545 after it.

**BEFORE**

```
// AND THIS IS WHERE IT HAS TO BE SAID. The restore document carries the set
// inventory, and both engraving callers now gate it on this function's own
// caller returning bundleEngraveDone -- so an operator whose engrave died really
// does not reach it, and this modal really is the only screen they get. (Until
// I-12 that was an ASSERTION rather than a fact: the abort did not propagate,
// and the restore document printed after every abort.)
```

**AFTER**

```
// AND THIS IS WHERE IT HAS TO BE SAID. The restore document carries the set
// inventory, and all THREE callers that carry a post-engrave tail now gate it on
// this function's own caller returning bundleEngraveDone: engraveSingleSigFlow
// (gui/singlesig.go), supplyMultisigPolicyFlow (gui/multisig.go) and
// buildMultisigPolicyFlow (gui/multisig_build.go). So an operator whose engrave
// died really does not reach it, and this modal really is the only screen they
// get. bundleFlow, at the top of this file, is the fourth bundleEngrave call site
// and needs no gate: it returns on the very next line, so nothing downstream of
// it vouches for the set.
//
// THIS SENTENCE SAID "both" UNTIL S6a STEP 8, AND IT WAS FALSE ON THE DAY IT WAS
// WRITTEN. S5's I-12 fold gated the two multisig callers and generalised from the
// two it was looking at; engraveSingleSigFlow already existed, ungated, with a
// tail of its own, so a single-sig abort still fell through to the verify offer
// and the restore document. S6a step 5 gated it. Counting the callers is not
// pedantry here: this comment is the justification a reviewer inherits rather
// than re-derives, which is how the gap survived a whole cycle. (Until I-12 it
// was an ASSERTION rather than a fact for the multisig pair too: the abort did
// not propagate, and the restore document printed after every abort.)
```

**The measurement behind it**, re-run in the worktree rather than quoted from §1.5:

```
$ grep -rn "bundleEngrave(ctx" --include="*.go" gui/ | grep -v _test
gui/bundle_flow.go:39      bundleFlow                 no gate, NO TAIL (returns at :40)
gui/singlesig.go:177       engraveSingleSigFlow       gates, tail (verify offer :190, restore doc below)
gui/multisig.go:291        supplyMultisigPolicyFlow   gates, tail
gui/multisig_build.go:402  buildMultisigPolicyFlow    gates, tail
```

Enclosing functions were resolved by command, not read off:

```
$ printf '%s\n' gui/singlesig.go:177 gui/multisig.go:291 gui/multisig_build.go:402 gui/bundle_flow.go:39 \
  | while IFS=: read f l; do awk -v L=$l 'NR<L && /^func /{last=$0} END{print last}' $f; done
func engraveSingleSigFlow(ctx *Context, th *Colors) {
func supplyMultisigPolicyFlow(ctx *Context, th *Colors) {
func buildMultisigPolicyFlow(ctx *Context, th *Colors) {
func bundleFlow(ctx *Context, th *Colors) {
```

### Comment 2 — `gui/multisig_verify.go`, edited at **:78** (pre and post)

**BEFORE**

```
// FOUR OUTCOMES, NOT A BOOL, because the callers owe three different things.
```

**AFTER** (headline corrected in place; a provenance paragraph added at :85–89)

```
// FIVE OUTCOMES, NOT A BOOL, because the callers owe three different things.
...
// THE HEADLINE SAID "FOUR" UNTIL S6a STEP 8, while the paragraph under it named
// all five and the const block below declares all five. Three obligations over
// five outcomes is the whole point of the type, so the two numbers were never the
// same number -- and S6a round 0 repeated the "four" straight out of this
// headline, which is what a miscount in a comment costs.
```

The type declares five constants at `gui/multisig_verify.go:87–100`: `verifyComplete`, `verifyIncomplete`, `verifyFailed`, `verifyRefused`, `verifyAbandoned`. **"three different things" is TRUE and was left alone** — three obligations (fall through / re-offer / do not re-offer) over five outcomes is the point of the type. Only the numeral was wrong.

### Comment 3 — `gui/bundle_flow.go`, edited at **:126** (pre) → **:125–137** (post)

**BEFORE**

```
// bundleGatherFlow accumulates distinct verified cards via NFC, returning them
// on "Done adding cards" (Button3) or (nil,false) on Back / an empty bundle. It
// owns its own scanner goroutine (clone of mk1GatherFlow's shell). With
// testPlatform.NFCReader()==nil the goroutine doesn't run; the gatherer +
// review flow are driven directly in tests.
```

**AFTER**

```
// bundleGatherFlow accumulates distinct verified cards via NFC, returning them
// on "Done adding cards" (Button3), or (nil,false) on Back or on the context
// being done. Those two are the ONLY (nil,false) returns in the function.
//
// PRESSING DONE ON AN EMPTY BUNDLE IS NOT ONE OF THEM, and this comment said it
// was until S6a step 8. bundleDoneEmpty shows an error screen and LOOPS back to
// the gatherer, so the operator carries on scanning rather than being dropped out
// of the program; bundleDonePending with nothing complete behind it does the same.
// The distinction is load-bearing rather than cosmetic -- an exit classification
// taking the old wording at face value would have argued a row on an exit that
// does not exist.
//
// It owns its own scanner goroutine (clone of mk1GatherFlow's shell). With
// testPlatform.NFCReader()==nil the goroutine doesn't run; the gatherer +
// review flow are driven directly in tests.
```

Verified by reading the function body (`gui/bundle_flow.go:143–242`): the only `(nil,false)` returns are `:177` (Back) and `:241` (loop exit when `ctx.Done`). `bundleDoneEmpty` calls `showError` at `:189` and falls through to the frame loop; `bundleDonePending` with `len(scr.g.cards) == 0` does the same.

---

## 3. THE RE-SWEEP — §4.8b's instruction, and what it found beyond the three

§4.8b: *"Step 8 must re-sweep rather than trust this list of three."* I ran two passes: my own targeted resolution of count/call-site/return claims, and an **independent sonnet sweep** with a one-question brief over the ten production files this cycle touched (plus `bundle_flow.go`), with the three known comments declared settled so the budget went elsewhere. Its verbatim report is at `design/agent-reports/s6a-step8-comment-resweep.md`, committed at `a7b27a1`.

**Four candidates returned. I machine-checked all four before acting on any. Three confirmed and fixed; one rejected.**

### (4) CONFIRMED and fixed — `gui/multisig_build.go:873`

Claimed *"three walk drivers anchor on it"*; **four** do.

```
$ grep -rl 'NEEDLE_SLOT = "Which slot is your key?"' --include="*.js" cmd/emu/
cmd/emu/walk_build_policy.js
cmd/emu/walk_s3_nested.js
cmd/emu/walk_s4_gate.js
cmd/emu/walk_trace_b.js
```

**"three" was TRUE when written, and I checked rather than assumed.** My first draft of the fix blamed `walk_s4_gate` for the drift; git says otherwise:

```
$ git log -S "three walk drivers anchor on it" --format="%h %ad" --date=short -- gui/multisig_build.go
4b10319 2026-08-15
$ git log --diff-filter=A --format="%ad" --date=short -- cmd/emu/walk_trace_b.js
2026-08-16
```

`walk_trace_b` landed the day *after* the comment. The correction names the four drivers instead of tallying them, and records that the count went stale by other people's work rather than by its author's mistake. **The wrong attribution was in my own draft and was caught by running the check** — it is exactly the defect class step 8 exists for, reproduced by the fold.

The same comment's other claim, *"exactly one production site"*, is TRUE — `gui/multisig_build.go:889` is the only non-test `.go` occurrence — and was left alone.

### (5) CONFIRMED and fixed — `gui/bundle_flow.go:367` (pre) → `:377` (post)

Cited `deriveXpubFlow`'s `multiPlateEngrave` call at `derive_xpub.go:162`. Measured: the call is at `:390`, inside `deriveXpubFlow` (`:330–397`); line 162 is inside `seedEntryFlowTypedOnlyTitled` (`:149–185`).

- BEFORE: `// multiPlateEngrave (R0-M2: Go has no default params; deriveXpubFlow's call site` / `// at derive_xpub.go:162 stays BYTE-UNCHANGED), reusing the same per-plate`
- AFTER: `// multiPlateEngrave (R0-M2: Go has no default params; deriveXpubFlow's own call` / `// to it, gui/derive_xpub.go:390, stays BYTE-UNCHANGED), reusing the same per-plate`

### (6) CONFIRMED and fixed — `gui/multisig.go:34` and `gui/singlesig.go:25` (mirrored text)

Cited `seedEntryFlowTypedOnly` at `gui/derive_xpub.go:124`. Measured: it is declared at `:140`; `:124` is inside `seedEntryFlowTitled` (`:104–139`).

- BEFORE (both files): `//     seedEntryFlowTypedOnly (gui/derive_xpub.go:124), which the VERIFY flows`
- AFTER (both files): `//     seedEntryFlowTypedOnly (gui/derive_xpub.go:140), which the VERIFY flows`

### (7) CHECKED AND **REJECTED** — `gui/multisig_restore.go:44`

The sweep flagged *"§2.2 D-3 names gui/multisig_restore.go:51 as the call site that matters"* as a stale citation, since `desc4Display` is now at `:29` and `:64`.

**Not a false claim, and I left it.** That sentence *quotes what SPEC §2.2 D-3 said*, and the very next sentence already corrects the reader — *"MEASURED 2026-08-15, running S3's own gate for the first time: that call site is desc4Display, which sits on the display-only branch ABOVE, and a full-policy build never reaches it."* Editing the quotation would misquote the spec to make the code look tidier. Recorded rather than silently dropped.

---

## 4. T8

`TestBundleAbortJustificationNamesEveryTailCarryingCaller`, `gui/singlesig_truth_test.go:2257`, with helper `guiDocComment` at `:2317`.

**Shape.** Negative half is **file-wide** on `gui/bundle_flow.go` (`both engraving callers` must be absent). Positive half is **scoped to one doc comment block** — the one immediately above `func bundleAbortWarningText(` — and asserts it names `engraveSingleSigFlow`, `supplyMultisigPolicyFlow` and `buildMultisigPolicyFlow`, plus the count phrase.

**Why scoped, and the trap it closes.** The brief named a false-PASS class already paid for twice: a source assertion satisfied by *another* comment carrying the string. `gui/bundle_flow.go` is ~900 lines of dense expository comment; a positive `Contains` over the whole file proves only that the words appear *somewhere*. `guiDocComment` walks backwards from the declaration over contiguous `//` lines, and **fatals** (rather than returning `""`) if the declaration is missing, appears more than once, or has no doc comment — because each of those states would otherwise let every `Contains` below it assert against the empty string and report a pass.

### RED, before the fix

```
--- FAIL: TestBundleAbortJustificationNamesEveryTailCarryingCaller (0.00s)
    singlesig_truth_test.go:2263: gui/bundle_flow.go still justifies the abort warning's placement with "both engraving callers". THREE bundleEngrave call sites carry a post-engrave tail -- engraveSingleSigFlow (gui/singlesig.go), supplyMultisigPolicyFlow (gui/multisig.go) and buildMultisigPolicyFlow (gui/multisig_build.go) -- so "both" undercounts the callers the gate has to hold for, and it undercounted them when it was written
    singlesig_truth_test.go:2281: the justification above bundleAbortWarningText does not name engraveSingleSigFlow, which carries a post-engrave tail and gates on bundleEngraveDone:
        [the whole doc block, echoed]
```

The echoed block also proves the helper read the right comment.

### GREEN, after the fix

```
$ nix develop --command go test ./gui/ -run 'TestBundleAbortJustificationNamesEveryTailCarryingCaller' -count=1
ok  	seedhammer.com/gui	0.007s
```

### MUTATION — *restore the old comment* (§5's named row)

Applied by script, verified present in the file (`grep -n "both engraving callers" gui/bundle_flow.go` → `545:`), then run:

```
--- FAIL: TestBundleAbortJustificationNamesEveryTailCarryingCaller (0.00s)
    singlesig_truth_test.go:2263: ... still justifies ... with "both engraving callers" ...
    singlesig_truth_test.go:2281: ... does not name engraveSingleSigFlow ...
    singlesig_truth_test.go:2281: ... does not name supplyMultisigPolicyFlow ...
    singlesig_truth_test.go:2281: ... does not name buildMultisigPolicyFlow ...
    singlesig_truth_test.go:2295: ... does not state the count as "all THREE callers that carry a post-engrave tail" ...
    singlesig_truth_test.go:2300: ... still claims "both engraving callers" ...
FAIL	seedhammer.com/gui	0.007s
```

**All five assertions fire. The mutation genuinely fails.** The file was restored from a byte copy and re-verified green afterwards.

### A FALSE PASS FOUND INSIDE T8 ITSELF — the reason the mutation was worth running

The first draft of the count assertion was `strings.Contains(strings.ToUpper(doc), "THREE")`. **Under the mutation it PASSED**, while the four other assertions failed: the same doc block already says *"has exactly three options"* about something else, four paragraphs above. A needle chosen for looseness was satisfied by unrelated prose in the very block it was policing — the precise defect T8 exists to forbid, reproduced in T8. The assertion now pins the phrase that carries the claim, `all THREE callers that carry a post-engrave tail`, and the comment above it records why.

This was not visible from the green run, from the red run, or from reading the test. Only the mutation showed it.

---

## 5. Step 9 — the spec

Target: `design/SPEC_seedhammer_T6a_singlesig_flagship.md`, **line 36**, in `/scratch/code/shibboleth/mnemonic-engrave`. Committed separately at `fa7a29a`, per §4.8 (*"the spec follows the behaviour, and is not mixed with it"*). No code in that commit; no other file touched.

Line 36 keeps every word it had and gains §4.9's three additions as a numbered list, plus the unchanged-constraint clause:

1. **the verification status line** — always exactly one, first on the page, derived from what the verify RECORDED (a 2×2 over two booleans) and never from a returned verdict; a skipped verify renders the weakest of the four lines rather than silence; the pass line is generated per mode; one scoping line follows the did-not-pass cell alone;
2. **the plate inventory** — count, per-card census, completeness sentence;
3. **the seed statement and the passphrase statement**, both passphrase arms speaking, with the capacity-keyed seed-handling ruling closing the section.

Descriptions were written from the shipped code (`gui/verify_status.go:128–267`, `gui/multisig_build_census.go:59–74, 205–229, 258–279`), not from the plan's prose.

### §4.9's gate — the grep pair, run

```
$ grep -c "verification status" design/SPEC_seedhammer_T6a_singlesig_flagship.md   # expect >= 1
1
$ grep -c "xprv"                design/SPEC_seedhammer_T6a_singlesig_flagship.md   # expect unchanged
3
```

And the same pair against the pre-step-9 file, so "unchanged" is a measurement rather than an assertion:

```
$ git show fa7a29a~1:design/SPEC_seedhammer_T6a_singlesig_flagship.md | grep -c "verification status"
0
$ git show fa7a29a~1:design/SPEC_seedhammer_T6a_singlesig_flagship.md | grep -c "xprv"
3
```

**`verification status` 0 → 1 (≥ 1 ✓). `xprv` 3 → 3 (unchanged ✓).**

**How the second half was kept honest.** §4.9 requires `greps clean of any xprv/private material` to hold and *not* be weakened. The obvious way to write the closing clause — restating the quoted constraint — would have pushed the `xprv` count to 4 and broken the very gate that polices it. The added prose therefore refers to *"the grep-clean constraint stated above"* rather than requoting the token, so the count could not be inflated by the edit meant to preserve it. The three surviving `xprv` occurrences are at lines 36, 47 and 70, the same three as before.

---

## 6. Validation gate — run on the committed tree

Run from `/scratch/code/shibboleth/wt-s6a` at `b2301d6`, working tree clean, **streams separated** (`> out 2> err`).

| check | result |
| --- | --- |
| `gofmt -l .` | clean (no files listed) |
| `go build ./...` | `GO_BUILD_RC=0` |
| `go test ./... -count=1` | **`SUITE_RC=0`**, `grep -c FAIL` → **0**, total wall **408 s** |
| **`seedhammer.com/gui`** | **`ok  405.798s`** |
| `./cmd/emu/build.sh` | `built emu.wasm (9991127 bytes)` — `go test` does not compile the emulator (§6) |
| `go vet ./...` | pre-existing only — see below |
| stderr stream | empty |

### The 600 s ceiling

| measurement | value |
| --- | --- |
| baseline, step-7 tree (`bf45367`), before T8 | **434.845 s** |
| after steps 8 (`b2301d6`) | **405.798 s** |
| ceiling | 600 s |
| **headroom** | **194 s — `gui` sits at 67.6% of the ceiling** |

**T8 costs nothing measurable.** It is a source assertion with no walk: scoped, it runs in 0.007 s. The post-change number is *below* the baseline, which is machine variance (the baseline ran concurrently with my scoped iteration runs), not a speed-up — the honest reading is "unchanged, ~406–435 s, well clear of 600 s". No timeout occurred at any point.

### `go vet`

Two classes, both pre-existing, and **no file this commit touches appears in the output**:

```
gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
gui/op/draw_test.go:176:24:                 testing.ArtifactDir requires go1.26 or later (file is go1.25)
backup/…, bspline/…, engrave/…             struct literal uses unkeyed fields
```

**One note against the brief:** it named the known go1.26 failure as being in `gui/freetext_sizeproof_golden_test.go` — correct, and there is **a second instance** in `gui/op/draw_test.go:176` that the brief did not mention. Both are the same `testing.ArtifactDir` / go1.25 mismatch, both pre-existing, both left alone.

---

## 7. Things that contradict the plan, or that the plan did not anticipate

1. **§4.8b comment 1's framing is pre-step-5.** It says *"THREE callers carry a tail … and only two gate."* True on `main`. In the worktree, **step 5 gated the third**, so the live state is *three carry a tail and all three gate*. The corrected comment says the latter. If it had transcribed §4.8b's sentence it would have shipped a fresh false comment inside step 8.

2. **§1.5's table names `gui/multisig.go:291` without its enclosing function**, and the natural guess is wrong: it is **`supplyMultisigPolicyFlow`**, not `engraveMultisigFlow`. The corrected comment names functions rather than files-plus-line-numbers precisely because the latter is what decayed in findings (5) and (6); resolving it by command rather than by name was what caught it.

3. **The three §4.8b comments had NOT moved.** The brief warned that steps 3–7 would have shifted them and that the plan's line numbers were from `main`. Measured at `bf45367`: comment 1 at `:535`, comment 2 at `:78`, comment 3 at `:126` — identical to `main`. Re-locating by content was still the right method; it simply confirmed rather than corrected.

4. **§6's validation gate says to run from `/scratch/code/shibboleth/seedhammer`.** I ran from the worktree, as the dispatch brief directed, and left `seedhammer` untouched.

---

## 8. Considered and deliberately NOT built (NG1: report, do not build)

1. **`gui/multisig_verify.go:53` — *"multisigVerifyRetryLead is the RE-offer both engraving callers make"*.** This is a second live *"both engraving callers"* in the tree, and it caught my eye immediately. **It is not false.** Measured, the constant has exactly two production sites (`gui/multisig.go:349`, `gui/multisig_build.go:464`), and the comment self-scopes on the next line — *"ONE STRING, TWO SITES, so the build path and the supply path cannot drift"*. It is now mildly *under-descriptive*, since S6a gave single-sig a post-engrave tail too and single-sig makes no re-offer (it has no retry loop by construction, `gui/singlesig.go:190`). Rewording it is a judgement call about clarity, not a correction of a false claim, and T8 polices `bundle_flow.go` only. **Left alone; flagged here.**

2. **`TestBothEngraveFlowsGateOnACompletedSet` / `TestBothEngraveFlowsReOfferTheVerify`** (`gui/multisig_verify_report_test.go:937`, `:1076`), under a section header reading *"the two flows' post-engrave wiring"*. Three flows now gate, so the names read as undercounts. But the doc comments are explicitly historical accounts of I-12, the assertion tables genuinely cover exactly two flows, and the single-sig gate has its own coverage in T5 (`TestSingleSigAbortIsTheLastScreenOfTheProgram`). Renaming tests is not correcting a false comment. **Left alone; flagged here.**

3. **A test for `guiDocComment`'s own fatal guards.** The helper fatals on a missing declaration, a duplicated declaration, and an absent doc comment. None of the three is exercised. The plan schedules no such row and the guards are defensive-only, so I did not add one. If a future round wants it, it is three sub-tests against a temp file.

4. **Making T8's count assertion tolerant** (case-insensitive, or accepting any "three"). Considered for brittleness, **rejected after the mutation proved the loose form false-PASSes.** Brittle-and-precise is the right trade for a test whose subject is a claim.

5. **Nothing was added to the verification's epistemic surface.** No new states, fields, or distinctions. Step 8 changed comments and added one source assertion; step 9 changed prose. NG1 holds.

---

## 9. Process deviation, recorded

The standing rule is: the agent's report lands in **its own commit BEFORE** the fold responding to it, so `git diff <report>..<fold>` reads in the right direction. Here the sweep report reached disk when the agent finished, but I **folded findings (4), (5) and (6) into step 8 and committed that first** (`b2301d6`), then committed the report (`a7b27a1`). The order is inverted for that pair.

Mitigating facts, stated rather than glossed: the report file was written by the agent and **never opened for editing by me**, so the text is byte-exact; each folded finding carries its own verifying command in `b2301d6`'s message; and finding (7) was rejected in writing rather than silently dropped. The inversion is also noted in `a7b27a1`'s own commit message so a future reader does not have to infer it from timestamps.
