# S5 round-2 gate verdict — `s5-multislot`

**Verdict: RED — 0 Critical, 1 Important (1 blocking).**

Repo reviewed (read-only, untouched — `git status --short` empty at start and at
finish): `/scratch/code/shibboleth/wt-s5`, head `6088487` ("S5 fold (B4, B5): the
retry loop and the BUILD abort, driven by executing tests").

This is the last review gate before `s5-multislot` merges and its output is
engraved on steel plates holding real Bitcoin keys.

## Trajectory

| round | result |
| --- | --- |
| 0 | RED — 3 Critical, 14 Important (C-1..C-3, I-1..I-14) |
| fold | `da4fa98`, `750296f`, `6088487` |
| 1 | RED — 0 Critical, 5 Important (B1..B5) |
| fold | folded B1..B5 |
| **2 (this)** | **RED — 0 Critical, 1 Important** |

The shape is convergence: 17 blocking → 5 blocking → 1 blocking, no Criticals for
two rounds, and the one survivor is a **test-coverage pin**, not a live defect.
Production behaviour on the reported path is **correct today**. Nothing on this
branch mis-engraves, mis-verifies, or loses funds as it stands.

Two independent skeptics ran a refutation pass over the round-2 candidates. **1
blocking finding survived; 0 were refuted.** The refuted list below is therefore
empty, which is itself a datum: no candidate this round was speculative enough to
fall over, and no finding may be reinstated from it because there is nothing in
it.

## Dedupe applied before ranking

Two merges, both same-site:

1. The Nit *"Only one of the two ms1-rejection arms has a table row"*
   (`gui/multisig_verify_report_test.go:438-471`) is the test-side statement of
   the blocking Important at `gui/multisig_verify.go:1018`. **Merged into the
   blocking item at the higher severity** — the Nit under-rated it by reasoning
   "both arms return on the same line so no branch is unpinned", which is true of
   the *source line* and false of the *behaviour*: the two arms are reached by
   different inputs and only one input is driven.
2. The Minor *"B1's group key choice is confirmed unpinned"* and the Minor *"the
   dedupe KEY is pinned by no test"* are the same mutation at
   `gui/multisig_build_slots.go:298`. **Merged**, keeping the sharper of the two
   (the one that also shows the fold's stated reason for non-pinnability is
   wrong).

Post-dedupe ledger: **0 Critical, 1 Important, 3 Minor, 4 Nit.**

---

## BLOCKING (1)

### Important — B3's fix is pinned at ONE of its two ms1 rejection arms; reverting the other leaves the whole suite green

**Defect (one sentence):** `multisigVerifyMS1Entry` has two rejection arms that
both must report `rejected=true` for a correctable mistake to reach the retry
offer, and the suite drives only the first — so a one-token revert of the second
restores round-1 B3's dead end with the entire `gui` package still green.

**Site:** `gui/multisig_verify.go:1018` (production, correct as written) —
unpinned by `gui/multisig_verify_report_test.go:446-490`
(`TestVerifyRetriesAfterACorrectableFirstSeed`).

The two arms, both new in this diff (`gui/multisig_verify.go:1004-1021`):

```go
s, isStr := obj.(codex32.String)
if !isStr {
        showError(ctx, th, "Verify Bundle", "That isn't an ms1 secret share.")
        return "", false, true          // :1013 — PINNED
}
_, _, ent, err := codex32.DecodeMS1(s)
if err != nil {
        showError(ctx, th, "Verify Bundle", "That isn't a valid ms1 secret share.")
        return "", false, true          // :1018 — PINNED BY NOTHING
}
```

**Trigger (concrete):** on the built-policy path, after nine plates are cut, the
operator reaches "Type ms1" and hand-types a legitimate BIP-93 **k=2 SSS share**
— e.g. `ms12namea320zyxwvutsrqpnmlkjhgfedcaxrpp870hkkqrm` — instead of the
unshared secret. `codex32.New` accepts it (`validateMStar`,
`gui/codex32_polish.go:263-268`, filters on HRP only, never on threshold or share
index), so `isStr` is **true** and the `:1013` arm is not taken; `DecodeMS1`
refuses it at `:1015` (`codex32/mspayload.go:28-33` documents exactly this: a
K-of-N share yields `errMSBadPrefix`/length, and callers "MUST pass only the
unshared secret"). The operator reads "That isn't a valid ms1 secret share." —
a screen naming an input they can retype. If `:1018`'s `rejected=true` is ever
reverted, the flow returns `verifyAbandoned`, neither engrave caller re-offers
(both loop on `verifyIncomplete`/`verifyFailed` only), and the next screen is the
restore document headed "If any of them is missing, this backup is incomplete" —
with **zero plates verified**. That is round-1 B3's dead end verbatim, on a route
round 1 named explicitly.

**Measured, in `cp -a` copies (the frozen tree was never written):**

- Unmutated tree, a new row driven through the fold's own
  `s5DriveVerifyFirstSeedRefused` with `badMs1` = that share:
  last frame `"Thatisn'tavalidms1secretshare.VerifyBundle"`, **verdict 1**
  (`verifyIncomplete`) — PASS. Production is correct today.
- Only `:1018` mutated (`return "", false, true` → `false`): same row
  **verdict 4** (`verifyAbandoned`) — the dead end reproduces.
- Whole package with that same one-token revert and **no** new test present:
  `ok seedhammer.com/gui 249.862s` / `248.666s` (two independent runs), exit 0 —
  **nothing red**.
- Positive control that the route is drivable and the suite is not simply blind
  everywhere: the identical revert on the *sibling* arm at `:1013` goes RED at
  `TestVerifyRetriesAfterACorrectableFirstSeed/the_first_seed's_hand-typed_ms1_is_rejected`
  (verdict 4, want 1). One arm is pinned; the other is not.

**Why the fold missed it:** all three of the fold's B3 mutations act on the
**consumer** (`correctable = true` → `false` at `:859`, `_ = rejected` at `:886`,
`correctable := true` at `:767`). The consumer is a single boolean OR —
`correctable = correctable || rejected` — so `_ = rejected` disables *both*
producer sites at once and structurally cannot tell a helper that reports one arm
from one that reports two. The test's own doc comment
(`gui/multisig_verify_report_test.go:409-410`) claims coverage of both — "*(wrong
HRP, or a checksum-valid string DecodeMS1 refuses)*" — while its only fixture,
`notAnMs1 := plates[0][0]`, is a **mk1** and can only ever reach `:1013`. The
comment asserts a gate that has never run.

**Minimal fix — test-only, one table row. Resolved against the real call graph; I
checked each link myself, in this tree:**

- `gui/multisig_verify_report_test.go:446-451` — the table's element type is
  `struct{ name string; exit s5FirstSeedExit; want multisigVerifyResult; because string }`.
- `s5FirstSeedExit` (`:326-335`) already carries `badMs1` ("*when set, is typed at
  'Type ms1' and puts the flow in FULL mode*") and `needle`.
- The driver types it: `:341` `full := x.badMs1 != "" || x.backAtMs1`, `:371`
  pumps to "Type ms1", `:380` `runes(&ctx.Router, strings.ToLower(x.badMs1))`,
  then `:386-391` `pumpUntil(frame, x.needle, 128)` and dismiss.
- The needle discriminates the two arms rather than aliasing them: `uiContains`
  (`gui/gui_test.go:527-532`) lowercases and strips spaces from the *needle*, so
  `"isn't a valid ms1"` → `isn'tavalidms1` matches only the `:1018` screen, and
  the existing row's `"isn't an ms1"` → `isn'tanms1` matches only `:1013`.
  Copy-pasting the existing needle would make the new row vacuous.
- The fixture is real. I ran it independently rather than inheriting it —
  a probe test in the `codex32` package of a scratch copy:
  `New("ms12namea320zyxwvutsrqpnmlkjhgfedcaxrpp870hkkqrm")` → **OK**;
  `DecodeMS1` → **`codex32: not an m-format secret payload`**. So the row lands
  on `:1015-1019`, not `:1010-1013`.

So the fix is:

```go
{
        name: "the first seed's hand-typed ms1 is a k-of-n SHARE, which DecodeMS1 refuses",
        exit: s5FirstSeedExit{
                phrase: fixtureMasterA,
                // BIP-93 k=2 share 'a': codex32.New ACCEPTS it, DecodeMS1 refuses it,
                // so it reaches the second rejection arm and never the first.
                badMs1: "ms12namea320zyxwvutsrqpnmlkjhgfedcaxrpp870hkkqrm",
                needle: "isn't a valid ms1",
        },
        want:    verifyIncomplete,
        because: "a valid codex32 string that is the wrong KIND of ms1 is still an " +
                "input the operator can retype",
},
```

plus, if the fold wants the comment to stop over-claiming, a one-line amendment at
`:409-410` noting that both arms now have a row.

**Explicitly NOT the fix — and this is the part that matters most.** No
production change is warranted. Do **not** "unify" the two arms into one return,
do **not** tighten `validateMStar`/`inputCodex32Flow` to reject shares earlier,
and do **not** touch `correctable = correctable || rejected` at `:886`. The
shipped behaviour on this path is already correct on every reachable input; every
one of those edits would be an unjustified change on a funds path, and this cycle
has already seen a prescribed "fix" that would have introduced a Critical. The
whole defect is that a correct line is unguarded, so the whole fix is a guard.

**Closure evidence the fold must produce** (not a new gate — it is the same
mutation already run, in the other direction): the new row PASSES on the
unmutated tree, and FAILS with `verdict 4` when `gui/multisig_verify.go:1018` is
reverted to `false`. Both were run in scratch copies during this round, so the
fold is transcribing a measured result, not predicting one.

---

## REFUTED — do not reinstate

**Empty.** Zero round-2 candidates were refuted by the two-skeptic pass. There is
nothing here to reinstate, and no finding below or above may be justified by
appeal to this section. (For the record, the earlier candidate **R-1** was
refuted in a prior round and remains refuted; it is not part of this round's
ledger.)

## Non-blocking ledger (3 Minor, 4 Nit) — recorded, gates nothing

**Minor**

1. **`s5AssertRetryLoop` misattributes a stale-lead regression.**
   `gui/multisig_engrave_tail_walk_test.go:301-308`. Mutating `lead =
   multisigVerifyRetryLead` → `_ = ...` (`gui/multisig.go:340`,
   `gui/multisig_build.go:456`) correctly turns
   `TestBothEngraveFlowsDriveTheRetryLoop` RED, but the message reads "*after an
   INCOMPLETE verify the offer was not made again*" while its own quoted frame is
   `"Verifytheengravedplates?VERIFYAGAINCONTINUEVerifyBundle"` — the offer plainly
   *was* made again. A maintainer reads the loop, not the lead. Same class as
   **F-200**; fold into it rather than opening a new item.
2. **The dedupe KEY is pinned by no test, and the fold's stated reason is wrong.**
   `gui/multisig_build_slots.go:298`. Re-keying the merge from `(Mnemonic,
   Passphrase)` to `MasterFP` — the exact change the function's own comment
   forbids, because it merges in the funds-losing direction — leaves
   `go test ./gui -count=1` at exit 0 (`ok seedhammer.com/gui 250.510s`). The fold
   logged this as unpinnable "because exhibiting a collision costs 2^32 work";
   that holds only through `reg.add`, and `seedRegistry.seeds` is a plain
   in-package field, so a ~12-line test can simply *state* the collision. Such a
   test was written and run this round: green unmutated, RED under the mutation
   (`collapsed to 1 fact(s) … [{Label:your seed for @0 and your seed for @1 …}]`).
   Behaviour is correct on every reachable input — a 4-byte collision is not
   operator-suppliable — so this is a regression test against a future
   "unification", not an unmet guarantee. (Absorbs the separately-reported B1
   group-key item.)
3. **The retry loop's non-looping half is still pinned only by a source grep.**
   `gui/multisig_build.go:453`. Inserting `if res == verifyAbandoned || res ==
   verifyRefused { res = verifyIncomplete }` before the check at both call sites
   preserves every string `TestBothEngraveFlowsReOfferTheVerify` greps and leaves
   the package green (`ok seedhammer.com/gui 245.179s`, exit 0) while re-offering
   the verify to an operator who declined it. Nothing is mis-verified and CONTINUE
   still exits, hence Minor. ~10 lines against machinery B4 already built
   (`s5StubVerifyFn(t, verifyAbandoned)` + assert the offer is not redrawn).

**Nit**

4. **B2's new sentence says "ALL of this wallet's seeds"; the obligation is only
   the operator's own.** `gui/multisig_verify.go:481-489`. `expectedSlots` is the
   tail's own held-slot list on both paths (`gui/multisig.go:230` / `:336`,
   `gui/multisig_build.go:384` / `:452`), so it never names a slot the operator
   cannot fill; "all of YOUR seeds" is the exact statement. The achievable reading
   is also the correct one, so nothing is unobeyable.
5. **`TestVerifyIncompleteInstructionCanBeObeyed`'s positive tokens are
   individually weak.** `gui/multisig_verify_report_test.go:276-290`. Four prose
   tokens in any arrangement satisfy it. Mitigated: obeyability itself is pinned
   behaviourally by `TestVerifyFullModeTwoSeedsReportsTheFullSuccess` (`:624-647`).
6. **`correctable = correctable || rejected` has a dead left operand.**
   `gui/multisig_verify.go:886`. `correctable` is provably false there on every
   path (its only other assignment, `:859`, is immediately followed by `break`).
   Reads as if correctability accumulates across iterations; it cannot.
7. **`verifyIncomplete`'s doc contract ("what was compared MATCHED") is false at
   two return sites.** `gui/multisig_verify.go:91`. Pre-existing —
   `git show 830aaf7:gui/multisig_verify.go` already returns it from the mk1-count
   refusal with nothing compared; this diff widens it by one (`:938`). Both
   production consumers only branch the retry loop, so no behaviour depends on it.

## B1..B5 — closure statement

| finding | status |
| --- | --- |
| **B1** — restore document counts SECRETS, not held slots | **CLOSED.** Behaviour fixed and pinned. Residue: the merge *key* itself is unpinned (Minor 2) — a missing regression test, not an open defect. |
| **B2** — the verify's instruction can be obeyed | **CLOSED.** Residue is two Nits about wording precision and assertion strength (4, 5). |
| **B3** — the remedy screens reach the retry offer | **NOT CLOSED — MOVED.** The production fix is complete and correct on **both** rejection arms; what moved is the evidence. Round 1 named both routes ("wrong HRP, **or a checksum-valid string DecodeMS1 refuses**") and the fold's own test comment claims both, but only the wrong-object arm is driven. The DecodeMS1 arm's `rejected=true` is held up by nothing, and reverting it restores B3's dead end with a green suite. This is the round's one blocking finding. |
| **B4** — the retry loop | **CLOSED.** Both engrave flows now drive it through an executing test. Residues: a misattributed failure message (Minor 1) and the still-grep-pinned non-looping half (Minor 3). |
| **B5** — the BUILD abort | **CLOSED.** Driven by executing tests; no round-2 finding against it. |

**4 of 5 closed. B3 moved rather than closed.** Per this project's standing rule —
*a plan or a fix may not close while one of its own gates has never been run* —
a claimed-but-undriven arm is a hypothesis, not a gate, so B3 stays open.

## What remains

Settled this round and not re-litigated: the five gates pass; `go vet` exit 1 is
40 test-only findings and clean; **I-8** ruled (b); `singlesig.go` out of scope;
**F-199 / F-200 / F-201** filed; **R-1** refuted.

To reach GREEN, in order:

1. Fold the one Important — **one table row plus its fixture constant in
   `gui/multisig_verify_report_test.go`, no production edit.**
2. Run the fold's own closure evidence: the new row green on the tree, RED under
   the `:1018` revert, and `go test ./gui -count=1` green. That is a mechanical
   check (sonnet tier at most), not another design round — the fold touches no
   production code and no control flow, so the proportional-re-review rule does
   not call for a fresh adversarial pass.
3. That closes the gate. **No new gates are introduced by this report.**

Then the already-known remaining work, unchanged and not expanded here: merge
`s5-multislot` into the fork's unprotected `main` and push; push
`mnemonic-engrave` `master` via the `ci/staging` dance; and **S6 (hardware)**,
which carries the already-filed **F-198** Critical on the single-sig path.

## Honesty note

One blocking finding after two productive rounds, zero Criticals, zero
refutations, and a survivor that is a missing test rather than a wrong behaviour —
this is the boring shape of convergence, and it should be read as such. The round
is not RED because the branch is dangerous; it is RED because a funds-path
guarantee that round 1 paid for is currently held up by a line no test would miss.
That is a ten-line fix, and it is the difference between a fix that is protected
and a fix that merely happens to be present.
