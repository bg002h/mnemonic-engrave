# H5 plan R0 round 1 — fold verification (sonnet)

**Scope.** Independent fold-verification reviewer for round 1 of the R0 gate on
`design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md`. Fold under review:
`a85c9fb` (plan) over `0c2b13e`, with companion spec fold `61a47e4` (both diffed
together per the brief: `git diff 0c2b13e..a85c9fb -- <plan> <spec>`). Round 0
inputs: `design/agent-reports/hashlock-H5-plan-R0-r0-{fidelity,tests,journey}.md`
(0C/3I/4M/4N, 0C/0I/1M, 0C/2I/5M/2N). Engrave HEAD at review time `9a04556`; the
plan/spec files there are byte-identical to `a85c9fb`/`61a47e4` (`git diff
a85c9fb HEAD -- <plan> <spec>` is empty).

**Method.** Own copy `/scratch/code/shibboleth/.tmp/h5-r1`, a `cp -a` of the
author's gated tree `/scratch/code/shibboleth/.tmp/h5-gate` (never modified; a
final `diff -rq` confirms `h5-r1` still matches it byte-for-byte after every
mutation below was applied and reverted). Go 1.26.7
(`/scratch/code/shibboleth/.toolchain/go/bin/go`), nix on `PATH` for the tinygo
builds. No sub-agents dispatched; no `.jsonl` read; nothing committed.

## Per-finding table

| # | finding (source) | change in the fold | verdict |
| --- | --- | --- | --- |
| 1 | fidelity I-1 = journey I-7 unmet — walk's stored-vs-displayed assertion was a tautology (compared `short8(stored)` against a constant that was `short8` of the same constant the corpus check used) | walk parses the token the confirm modal actually drew (`drawnToken`, regex on the squashed frame) into `out.displayed`, compares stored digest against **that** before the corpus check; reconcile screen's token assertion reads the frame too | **FIXED.** Diff read in full; independently replayed `drawnToken`/`squash` in Node against realistic frame text built from `composer_copy.go`'s actual format — correctly parses `3cf5d421..b70a4c12` from both the confirm-modal and reconcile-screen frame shapes; scenario (c) (stored perturbed, displayed unchanged) fails the screen comparison and not the corpus one; scenario (d) (screen and stored agree, corpus doesn't) fails only the corpus check — the two are independent, not restating each other |
| 2 | fidelity I-2 — `ok`-shape guard read only the FIRST `.ok =` assignment, so a walk's real (LAST) verdict could be a caller-supplied `plates` term and still clear the guard | `walkOkAssignments` now uses `FindAllStringSubmatch`; the `plates` check runs on every right-hand side; `TestWalkOkGuardReadsEveryAssignment` gates the blind spot with 5 rows | **FIXED, re-run.** `TestWalkOkGuardReadsEveryAssignment` PASS (5/5 subtests). Reverting `walkOkAssignments` to `FindStringSubmatch` (first match only) reproduces the plan's exact quoted failure: `walkOkDriverSupplied found 0 caller-supplied term(s) [] in ["false"], want 1` and `allConst = true over ["false"], want false` on the `the_verdict_is_the_last_assignment` subtest. Reverted; tree confirmed clean against gate. |
| 3 | fidelity I-3 — `composerFlowExit` and `composerTextBand` were inserted directly beneath `composerFlow`'s and `composerPageLines`' doc comments (no blank line), so Go merged the comment blocks and both original functions lost their doc text | each helper moved ABOVE the doc block it had captured; new `gui/composer_doc_comment_test.go` (`TestComposerHelpersDidNotStealADocComment`) gates all 4 symbols | **FIXED, re-run.** Test PASSES on the folded tree. Reconstructed the exact original defect (moved `composerFlowExit`'s comment+func back between `composerFlow`'s doc comment and `composerFlow`) — reproduces the plan's exact quoted failure, byte for byte: `composerFlow has NO doc comment in composer_flow.go` and `composerFlowExit's doc comment ... opens "composerFlow is \"Build a new policy\" ..."`. Reverted; tree confirmed clean against gate. |
| 4 | journey I-1 — write-down sentence: "They are not on this device and not on your plates" became false once "and this digest" joined the list (digest IS in the engraved md1); "Without both" got a 3-item antecedent | second sentence scoped to `"The phrase and method are not on this device."`; first/third sentences unchanged | **FIXED, re-run.** Reverting to the old sentence reproduces the plan's exact quoted `TestComposerCopyIsVerbatimFromTheSpec` failure (`composerCopyHashlockConfirm (SPEC §H2-4.5) does not match the spec`, both bodies quoted in full). Fit re-measured: **343 drawn / headroom 107** (claimed 343/107). Reverted; tree clean. |
| 5 | journey I-2 — §8h's PLAIN form kept the singular "Back the preimage up separately," the same undercount the phrase form's sibling sentence was fixed for | plain form's last sentence becomes "Back up every preimage separately."; new `TestTwoPlateWalletBannerCountsEveryPreimage` | **FIXED, re-run.** Reverting reproduces both named failures exactly (`TestComposerCopyIsVerbatimFromTheSpec`: `composerCopyHashEveryPath (SPEC §8h) does not match the spec`; `TestTwoPlateWalletBannerCountsEveryPreimage`: both quoted lines). Fit re-measured: **133 drawn / headroom 397** (claimed 133/397; unmutated old form independently measured at 131/397, matching "(was 131/397)"). Reverted; tree clean. |
| 6 | journey M-1 — Task 5 Step 12 never reverted a mutation between the 3 walk runs, so a contaminated run (c) would fail at the WRONG (pre-hold) assertion and record a false pass of §4.5(c) | `git checkout -- gui/composer_hashlock.go && git diff --quiet gui/composer_hashlock.go` added before each mutation and after run (c) | **FIXED (procedure), corroborated.** Applied mutation (b) then, WITHOUT reverting, mutation (c) to `gui/composer_hashlock.go`: both edits coexist and the file still `go build`s cleanly — nothing structural catches the stacking, confirming the risk M-1 names is real and only the walk's own assertion order (or the discipline of reverting) can prevent it. File-copy revert (the moral equivalent of `git checkout` in this non-git scratch tree) restored the file byte-identical to the gate tree. The added procedure text is present verbatim in Task 5 Step 12. |
| 7 | journey M-2 — reconcile screen named "before you fund", but the plates (which carry the digest) are cut ~21 min later, first | first sentence: "Before you cut plates, run ms hashlock ..."; mismatch sentence keeps funding as the deadline | **FIXED, re-run.** Reverting reproduces the plan's exact quoted failure (`composerCopyHashlockReconcile (SPEC §H2-4.5) does not match the spec`); `TestHashlockReconcileScreenCarriesTheDigestMethodAndChars` independently confirmed to stay GREEN under this mutation, exactly as the plan claims. Fit re-measured: **181 drawn / headroom 339** (claimed 181/339). Reverted; tree clean. |
| 8 | journey M-3 — unlock refusal named one record; `AdmitSection` refuses one at a time and the index moves between rounds | body gains "-- and any others like it --" | Diff-verified present in both `unlockNotPermittedBody` and the test table; not independently mutated (Minor, and covered by the whole-gui-shard run below), fit re-measured via the shard/targeted run: **175 drawn / headroom 378** at the longest noun (claimed 175/378) |
| 9 | journey M-4 — phrase form overcounts on the 2 pure wallets (all-phrase; re-typed-as-hex) | declined; recorded as a deliberate decision in a code comment | **Declined with a true reason**, verified present in `gui/composer_copy.go` beside the phrase-form sentence and in spec's Plan-round-fold table |
| 10 | journey M-5 / N-2 — manual's Back table has no row for the reconcile screen; lead sentence false there; `chars:` field has no documented host counterpart | Task 6 Step 4 gains the Back-table row + qualifies the lead sentence; one clause naming `ms hashlock`'s `phrase: N characters` stderr line | Diff-verified text present; **citations independently re-checked against toolkit `46b40bb`**: heading `:512` = `#### What Back does`, lead `:514-515`, table `:517-524` (line 525 is blank, confirming the fold's correction of the journey report's own `:514-525` off-by-one); `mnemonic-secret` `504ff46` `crates/ms-cli/src/cmd/hashlock.rs:329-331` (`phrase_chars` JSON field) and `:351-352` (`phrase: {n} characters` stderr line) — both exact |
| 11 | fidelity M-1 — "re-typed as 64 hex" test row was byte-identical to "one phrase path" | row uses a distinct pointer (`retyped := phrase`) holding the same bytes; new `TestReassigningTheSameDigestStaysByPhrase` | Diff-verified; both tests run GREEN as part of the shard run below |
| 12 | fidelity M-2 — spec §6 asked for the firmware delta "per change"; granularity doesn't support it | spec folded the other way: delta stated per STAGE with the reasoning; hook's share claim downgraded from "0 bytes" to "no measurable cost" | **Re-measured independently — confirms the correction was necessary.** Rebuilt "H5, this tree" (**1,599,208 B** flash, exact match) and "hook deleted from the tinygo view" (**1,599,224 B** flash, exact match) with the plan's own tinygo command. Hook's share = **-16 B**, matching the claim exactly; a build 16 B *larger* without the hook is why "0 bytes" would have been false and "no measurable cost" is the honest claim. |
| 13 | fidelity M-3 — `hashlockOtherPathLine` comment still said "that flag" after a prior fix stopped one clause short | extended one line further ("that set's own history") | Diff-verified text present |
| 14 | fidelity M-4 — spec §5 asked for "the copy-table row updated," but no such row can exist (`unlockNotPermittedBody` is not `composerCopy*`) | one-line erratum added to spec §5 naming the `assertModalBodyFits` row as what stands in its place | Diff-verified present in spec, matches `composerCopyTable`'s actual AST-scan restriction (unchanged code fact, previously verified by r0 fidelity) |
| 15 | fidelity N-1..N-4, journey N-1, tests M-1 | prelude branch name -> `hashlock-h5`; keyboard-grid probe note re-dated to H2/`e1bf137`; positive-control recipe stated exactly (`println("hook")`) and re-measured; `composer_hashlock.go`'s stale line-number citation dropped to match its sibling; ok-guard log line narrowed to what is checked; RED quotes re-captured complete with `-gcflags=-e` | All present in the diff; `-gcflags=-e` and the complete 20-line / 3-call-site RED quotes were spot-checked against the diff text (not re-executed — Minor, non-blocking, and the underlying build-failure mechanism is unchanged) |

**No Critical or Important finding across the three round-0 reports was left unfixed
or undeclined-with-a-reason.**

## Executed checks, with output

**1. The checker, folded plan against the gate tree** (default args, run from
the engrave repo):
```
$ ./scripts/h5-plan-blocks-vs-tree.sh
...
55 blocks checked, 0 FAIL
```
Matches the fold commit's own claim exactly.

**2. Whole `gui` shard set, once, on my own copy of the gate tree**
(`cd .tmp/h5-r1 && .../gui-shard-test.sh ./gui/ 24`):
```
    1239 top-level tests
    partition verified exhaustive: 1239 == 1239
=== wall: 29s ===
RESULT: ok -- all 1239 tests ran across 24 shards
```
All 24 shards `ok`. **`TestEngraveScreenReleasesResumeStateOnReturn` (F-490) did
NOT flake in this run** — no isolated re-run was needed.

**3. Fit numbers, re-measured with `assertModalBodyFits`** (or its direct
callers), on my own tree:
```
the hashlock reconciliation screen (H2 §4.5, H5 §1): 181 chars drawn in full, headroom 339 chars (margin 80)
HASH ON EVERY PATH, phrase-route form (H2 §4.7): 165 chars drawn in full, headroom 378 chars (margin 80)
the hashlock confirm modal, longest variant (H2 §4.5): 343 chars drawn in full, headroom 107 chars (margin 80)
the §8h every-path-hashed warning: 133 chars drawn in full, headroom 397 chars (margin 80)
[unlock refusal, longest noun/two-digit index]: 175 chars drawn in full, headroom 378 chars (margin 80)
```
All five match the plan/spec exactly: 181/339, 165/378, 343/107, 133/397, 175/378.

**4. Mutations applied, quoted, reverted** (each followed by a `diff -q`/`diff
-rq` confirming the file, then the whole tree, matched the gate tree again):

- Write-down line (journey I-1): reverting to `"They are not on this device and
  not on your plates."` → `TestComposerCopyIsVerbatimFromTheSpec` FAILs quoting
  both bodies verbatim, as claimed.
- Plain form's plural (journey I-2): reverting to `"Back the preimage up
  separately."` → `TestComposerCopyIsVerbatimFromTheSpec` and
  `TestTwoPlateWalletBannerCountsEveryPreimage` both FAIL with the claimed
  messages; `TestComposerEveryPathHashedWarns` logs 131/397 for the old text,
  matching "(was 131/397)".
- Reconcile screen (journey M-2): reverting the first sentence to `"Before you
  fund this wallet, ..."` → `TestComposerCopyIsVerbatimFromTheSpec` FAILs;
  `TestHashlockReconcileScreenCarriesTheDigestMethodAndChars` stays PASS, exactly
  as the plan claims (its needles don't touch the moved sentence).
- Step 12's revert discipline (journey M-1): applied mutation (b)
  (`h := hashlock.Digest(&x); st.list.Paths[idx].Hash = &h`) then, without
  reverting, mutation (c) (`d := h; d[0] ^= 1`) to the same file — both compile
  cleanly together (`go build ./gui/...` exit 0), confirming nothing structural
  would catch contamination between runs if the revert step were skipped; the
  file-copy revert (this scratch tree has no `.git`) restored it byte-identical
  to the gate tree, corroborating that the added `git checkout`/`git diff
  --quiet` procedure is the correct fix.
- fidelity I-2 (`walkOkAssignments` → `FindStringSubmatch`, first match only):
  `TestWalkOkGuardReadsEveryAssignment` fails exactly as the plan quotes.
- fidelity I-3 (`composerFlowExit` moved back between `composerFlow`'s doc
  comment and itself): `TestComposerHelpersDidNotStealADocComment` fails with
  the plan's exact quoted messages.

**5. Firmware sizes, two of the five builds re-measured with the plan's exact
command** (`nix develop -c tinygo build -size short -o /dev/null -target
pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks
./cmd/controller`):
```
H5, this tree:                        1567356  31852  31004 | 1599208  62856
hook removed (stub + both call sites): 1567372  31852  31004 | 1599224  62856
```
Both exact. Hook's share = **-16 B** (image without the hook is 16 B *larger*),
confirming the fold's "no measurable cost" correction was necessary and correct.
(The second-defer and `println`-control rows were not independently rebuilt;
non-blocking — they were already independently reproduced by the r0 fidelity
report and are not the number the fold corrected.)

**6. Whole-tree gates:**
```
$ go test ./hashlock/ ./codex32/ ./sysw/ ./seal/ ./cmd/emu/   -> 5x ok
$ gofmt -l .                                                  -> 5 pre-existing files (same as pristine b9a9a30)
$ go vet ./gui/ ./cmd/emu/                                    -> 2 pre-existing go1.26 findings (same as pristine)
$ GOOS=js GOARCH=wasm go vet ./cmd/emu/                        -> clean
$ ./cmd/emu/build.sh                                          -> built emu.wasm (10873125 bytes)
$ CGO_ENABLED=0 go test -timeout 20m ./...                    -> exit 0, 55 ok (grep -c '^ok'), 0 FAIL lines
```
All match the fold's claimed gate output exactly.

**7. Stored-vs-displayed assertion (fidelity I-1), independently replayed**
(Node, `drawnToken`/`squash` copied verbatim from `cmd/emu/walk_hashlock_phrase.js`
against synthetic frames built from `composer_copy.go`'s actual string format):
confirmed `drawnToken` parses `3cf5d421..b70a4c12` out of both the confirm-modal
and reconcile-screen frame shapes; confirmed scenario (c) (perturbed stored,
unperturbed displayed) fails only the screen comparison and scenario (d) (screen
and stored agree on a non-corpus value) fails only the corpus comparison — the
two assertions are independent, closing the "gate that cannot fail" defect.
`node --check cmd/emu/walk_hashlock_phrase.js` parses clean.

**8. Superseded phrasing, grepped** in both the plan and the spec: `hashlock-h2`,
the unscoped write-down sentence, the old reconcile "before you fund" opener, the
old plain-form singular, bare "0 bytes"/"0 B" hook claims, and the old firmware
numbers (1,599,164 / 1,599,276 / 1,599,388). Every hit is a deliberately-labelled
historical quote (contrasting old vs. new, or a find/replace target for the H2
spec fold) — none is a leftover normative claim. The actual walk file's runtime
string reads `hashlock-h5` (confirmed by direct grep of the tree, not just the
plan's quote of it).

**9. Plan/spec copy strings, byte-identical:** reconcile body, confirm-modal
write-down (all three sentences), phrase-form last sentence, plain-form last
sentence, and the unlock refusal's new clause were extracted from both documents
and compared — identical in every case.

## Closing counts

- 5 Important findings (fidelity I-1, I-2, I-3; journey I-1, I-2): **5/5 fixed**,
  each independently re-run under its named mutation (or, for I-1, independently
  replayed) and reproducing the plan's exact quoted output.
- 11 Minor/Nit findings across the three reports: all folded (with true reasons
  where declined — journey M-4; fidelity M-2's "per stage, not per change"
  reframe) or diff-verified present; none reopened.
- Checker: **PASS**, 55 blocks, 0 FAIL.
- Whole `gui` shard run: **1239/1239, 24/24 shards ok**; no F-490 flake this run.
- Whole-tree `go test ./...`: **55/55 packages ok, 0 FAIL, exit 0**.
- `gofmt`/`go vet`: same pre-existing findings as pristine `b9a9a30`, no new ones.
- Firmware: 2/5 builds independently reproduced exactly; hook's share **-16 B**,
  confirming the "no measurable cost" correction.
- No new false number, no contradiction, no superseded phrasing found.
- Own copy `/scratch/code/shibboleth/.tmp/h5-r1` matched
  `/scratch/code/shibboleth/.tmp/h5-gate` byte-for-byte (`diff -rq`) both before
  the mutation runs and after every one was reverted.

## Verdict

**GREEN.** The round-0 fold fixes every Critical/Important the three lenses
raised (0 Criticals existed), declines the rest with true reasons, the checker
and the whole test/build suite pass on the folded tree, every changed test fails
under its named mutation and passes clean otherwise, every re-measured number is
exact, and the plan and spec agree byte-for-byte on every copy string this round
touched. This closes R0 round 1 for the plan.
