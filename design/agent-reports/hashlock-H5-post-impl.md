# Hashlock H5 — independent adversarial execution review (post-implementation)

**Verdict: NOT GREEN — 0 Critical / 1 Important / 2 Minor / 4 Nit.**
The one Important is a single false clause in an un-merged FOLLOWUPS closure header
(`h5-records`), fixable in the same edit that fills `<FORK_MERGE_SHA>`, requiring no code
change and no gate re-run. **Every code, copy, geometry, gate and report claim I checked at
the tips held.**

Trees under review: fork `hashlock-h5` `8e605e1` (base main `b9a9a30`), engrave `h5-records`
`4e2cf01f`, toolkit `h5-manual` `b48af1c1`. All work done in my own detached worktrees
(`/scratch/code/shibboleth/.tmp/h5-review` at `8e605e1`, `.tmp/h5-base` at `b9a9a30`,
`/tmp/h5a|h5b|h5c` at the three implementer branches), each restored to a clean tree after
every mutation and removed at the end. Nothing was modified in the branch worktrees or the
fork checkout, nothing committed, no sub-agents, no `.jsonl` read, no phrase or preimage
bytes in anything kept. Go `1.26.7` at `/scratch/code/shibboleth/.toolchain/go/bin/go`;
TinyGo `0.41.1` via `nix develop`.

---

## 1. The question, answered

> Can you construct a state, input or sequence for which the device shows a digest that
> differs from the stored one, loses the operator's work, names a backup wrongly, draws copy
> the spec does not say, or lets a test report PASS on a defect it names — and does every
> claim in the four implementation reports hold at the tips?

**No, on all five, and yes on the reports.** I could not construct any such state.

* **Displayed vs stored is identical by construction, not by agreement.** Both the confirm
  modal (`gui/composer_hashlock.go:65`) and the reconcile screen (`:83`) are handed
  `hashlockFirst8Last8(h)`, `m.String()` and `len(phrase)` from the *same* three locals, and
  the stored value is `d := h` — a copy of the very `[32]byte` the token abbreviates
  (`:68-69`). There is no second derivation to disagree with. The only other place a digest
  reaches the panel is `composer_consent.go:95`, which reads `b.Sha256Digests` off the decoded
  `md.Branch` — what was *compiled*, not what state holds — and `composerDigestShort`
  (`:61-64`) is `hashlockFirst8Last8` spelled with `h[56:]` instead of `s[len(s)-8:]`. Both
  are 64-hex, so those are the same function.
* **No lost work.** The only lifecycle change is `defer st.reg.scrub()` becoming
  `defer composerFlowExit(st)`, which runs the identical scrub and then clears the hook. Same
  defer, same exits, same panic coverage. The three deleted `composerHashByPhraseSync` sites
  only ever cleared a bool.
* **Backup naming is right on every shape I could build** — see the banner table in §4,
  including the two shapes the spec deliberately overcounts.
* **Copy is byte-exact against the spec** on all five bodies (§3).
* **Every test the diff adds or edits bites** — 24 mutations, all fatal, in §5.

---

## 2. Findings

### I-1 — the F-485 closure header claims a gate that does not exist, and contradicts spec §4.3

`h5-records` `4e2cf01f`, `design/FOLLOWUPS.md`. Verbatim:

```
### F-485 — ~~`hashlock-walk-does-not-assert-hold-order-or-stored-vs-displayed`~~ **CLOSED
2026-09-05 by fork `<FORK_MERGE_SHA>`** (hashlock H5 -- the next device code cycle these five
were owned to, not overdue; gate the rewritten `cmd/emu/walk_hashlock_phrase.js` plus
`gui/composer_state_hook_test.go` / `ComposerPathHashes()`, asserting the post-hold stored
hash against the displayed token and picking the row by label):
```

The last clause is false, and it is false about a decision the spec makes explicitly in the
opposite direction. `design/SPEC_hashlock_H5_device_polish.md` §4.3:

```
3. Row picking stays by INDEX with the landing assertion (`chooseRow(i, expect, label)`):
   `shTargets` exposes rectangles only -- `frameTargets` returns bare `image.Rectangle`s and
   `gui/screen.go:95-98` drops the tag on purpose -- so a label pick would need a second gui
   seam for no safety gain the landing assertion does not already give (fidelity I-2, tests
   I-1, journey I-5: the original §4.3 is withdrawn). F-485's index note is recorded as
   answered this way.
```

Measured in the shipped walk:

```
$ grep -c "chooseRow(" cmd/emu/walk_hashlock_phrase.js
10
$ grep -n "targets\[i\]" cmd/emu/walk_hashlock_phrase.js
218:  await tap([targets[i].cx, targets[i].cy], 300);
$ grep -n "label" cmd/emu/walk_hashlock_phrase.js | grep -v "^.*//"
208:async function chooseRow(i, expect, label, settle = 350) {
215:    throw new Error(`choosing ${label}: the frame offers ${targets.length} tappable row(s) …
224:    throw new Error(`choosing ${label} (row ${i} of ${targets.length}) did not land on …
```

Every pick is `chooseRow(<integer>, …)` tapping `targets[i]`; `label` is a message string
only. F-485's body lists three defects, one of which is *"it picks the phrase row by INDEX"* —
so the closure header tells a future auditor that the item was closed by fixing the very
sub-defect the spec ruled would NOT be fixed, and named the *reason* it would not be. An
auditor checking that closure looks for a label pick, does not find one, and either re-opens a
correctly-closed item or stops trusting the closure headers.

The prose is implementer D's own (report D deviation 3: *"the exact sentence form is mine"*);
the plan does not contain the string (`grep "picking the row by label"` over the plan returns
nothing), and report D itself never repeats the claim, so this is confined to the record.

**Why Important rather than Minor.** The brief's severity table puts records at Minor, and if
this were wording I would file it there. It is not wording: it is a specific, checkable claim
about what the shipped gate does, on the one line that closes the follow-up that decision
belongs to — the "a defect in what a tool *claims* to have done" class. The remedy is free:
the controller must edit this line anyway to replace `<FORK_MERGE_SHA>`.

**Remedy (one clause).** Replace *"and picking the row by label"* with something true, e.g.
*"and keeping the index pick with its landing assertion (spec §4.3; F-485's index note is
answered, not fixed)"*. No code changes; no gate re-runs.

The other four closure headers are true as written and I checked each:

| follow-up | claimed gate | verified |
| --- | --- | --- |
| F-480 | "`gui/composer_provenance_test.go`'s six tests" | `grep -c "^func Test"` = **6** ✓ |
| F-484 | "`gui/composer_hashlock_geometry_test.go`, asserting the lead's ink stays out of the nav button rects" | ✓ (M10 below) |
| F-487 | the `composerCopyHashlockReconcile` row + `assertModalBodyFits`; H2 §4.5/§4.7 folded in the same commit | ✓ (M1/M2 below; §6) |
| F-488 | `TestUnlockNotPermittedBodyNamesTheRecordAndTheKind`, `gui/unlock_preimage_test.go` | ✓ (M12–M14 below) |

### M-1 — nothing in the tree machine-checks the code against `SPEC_hashlock_H2_device.md`

Spec §1.4 says the H2 fold is gated: *"the copy-verbatim test
`TestComposerCopyIsVerbatimFromTheSpec` diffs the code against `SPEC_hashlock_H2_device.md`"*.
It does not. `composerCopyTable()`'s `verbatim` column is a Go string literal in
`gui/composer_copy_test.go`, and no test in the fork reads any markdown:

```
$ grep -rn "\.md\"" --include=*_test.go gui/ cmd/
(no output)
```

So the test proves the code matches *a literal a human typed*, and the H2 spec fold is bound
to the shipped copy by nobody. The literals and the spec do agree here — I checked it
mechanically in both directions (§6) — but the property is a coincidence of this cycle's
diligence, not a gate. Worth a follow-up: a test that reads the spec markdown (the fork would
need the file, or the check lives on the engrave side).

### M-2 — the H2 spec fold carries a citation that decays inside its own commit

`h5-records` `4e2cf01f`, `design/SPEC_hashlock_H2_device.md`, §4.7:

```
(the shipped text now ends *"Back up every preimage separately."*, still
naming only "the preimage" -- an artifact this route cannot produce;
`composerCopyHashEveryPath` at `gui/composer_copy.go:169-173`). The
```

```
$ grep -n "func composerCopyHashEveryPath()" .tmp/h5-base/gui/composer_copy.go   # b9a9a30
173:func composerCopyHashEveryPath() string {
$ grep -n "func composerCopyHashEveryPath()" .tmp/h5-review/gui/composer_copy.go # 8e605e1
182:func composerCopyHashEveryPath() string {
```

`:169-173` was right at `b9a9a30` and is wrong at the tip this very commit describes, with no
baseline SHA attached (the H5 spec, by contrast, writes *"`composer_copy.go:169-173` at
`b9a9a30`"*). Records only; costs one `at b9a9a30` or one re-grep.

### N-1 — spec §4.2 names the wrong assertion for one of its two independence directions

§4.2: *"The two are then independent: a perturbed stored digest fails only the first, and a
screen and a policy AGREEING on a digest the corpus does not hold fails only the second."*

The first half is exactly right and the controller's run (c) demonstrated it. The second half
is not: if the screen drew a non-corpus digest, `must(hardened, ANCHOR_HARD_H, …)` — which
runs on the same frame *before* `drawnToken` is even called (`walk_hashlock_phrase.js:405`) —
throws first, and neither the stored-versus-displayed nor the corpus assertion is reached. The
walk still fails, loudly and with the corpus digest named, so nothing is weakened; the spec
sentence just attributes the failure to the wrong line. No action needed beyond noting it.

### N-2 — the H2 §4.7 parenthetical reads oddly after its own fold

Same paragraph as M-2: *"still naming only \"the preimage\""* now sits immediately after
quoting *"Back up every **every** preimage separately."* The point being made is about the
artifact **kind** (a preimage plate rather than a phrase), which is still true; the word
"only" now reads as a count claim that the fold just removed. Wording.

### N-3 — the settled "gofmt/vet clean" is true only at CI scope

Measured at `8e605e1`:

```
$ gofmt -l .            → gui/transaction.go, gui/transaction_golden_test.go,
                          gui/transaction_txrecord_test.go, mt/mt.go, mt/mt_test.go
$ git diff --name-only b9a9a30..8e605e1 | grep '\.go$' | xargs gofmt -l   → (empty)
$ go vet ./...          → 8 × "testing.ArtifactDir requires go1.26 or later (file is go1.25)"
$ GOOS=js GOARCH=wasm go vet ./cmd/emu/   → clean
```

All five gofmt files and all eight vet findings are in files H5 does not touch, with `go.mod`
unchanged, so they are identical at `b9a9a30`. `.github/workflows/test.yml:94` says so
explicitly and deliberately does not run `go vet ./...` (*"vet reports 40 pre-existing
findings … so a vet step here would fail on day one"*). Nothing here blocks; recorded only so
the phrase "vet clean" is understood as CI-scoped. Report B and report C D-5 both characterise
this correctly; the dispatch summary is the loose one.

### N-4 — the phrase screen's readout budget is exactly at its floor

`TestHashlockPhraseScreenKeepsTheReadoutBudget` logs, unmutated:

```
MaxHeight=209 grid=(340,182) gap=8 -> readout budget 19 px; one line is 19 px
```

19 ≥ 19 passes, with zero slack. That is F-481's floor met exactly, not a regression H5
introduced (the lead is 2 lines / 44 px at both the old 464 px and the new 411 px band, so
`MaxHeight` is unchanged), and the gate does bite — M11 drops it to 11 px and the test fails.
Recorded because the next pixel spent on this screen breaks it.

---

## 3. Copy, byte-for-byte against the spec

| spec | normative text | shipped | verdict |
| --- | --- | --- | --- |
| §1.1 | `hash  <first8>..<last8>\nmethod: <m>   chars: <n>\nBefore you cut plates, run ms hashlock with this phrase and method on the host and check the digest matches. If they differ, do not fund this wallet: build it again.` | `composerCopyHashlockReconcile`, `composer_copy.go:485-491` | **byte-exact** |
| §1.2 | `Write down this phrase, the method and this digest now.` / `The phrase and method are not on this device.` / `Without both, this path can never be spent.` | `composerCopyHashlockConfirm`, `:434-436` | **byte-exact** |
| §2.5 | `Back up every phrase and its method, and every preimage plate, separately.` | `composerCopyHashEveryPathPhrase`, `:520-524` | **byte-exact** |
| §2.6 | `Back up every preimage separately.` | `composerCopyHashEveryPath`, `:182-186` | **byte-exact** |
| §5 | `Remove that record -- and any others like it -- (records count from 0) on the host and seal the payload again.` | `unlockNotPermittedBody`, `unlock_kdf.go:415-418` | **byte-exact** |

The reconcile and confirm headers are *spelled* alike, not merely equal after normalisation:
`TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal` requires the literal
`"hash  " + tok + "\nmethod: " + method + "   chars: 28\n"` prefix of both, against a third
literal (so a shared drift cannot satisfy it). M21 below proves it is the *only* gate on the
three-space separator — `normalizeDrawn` hides it from the copy table and the flow test.

### Fit, re-measured (`assertModalBodyFits`, `modalBodyMargin = 80`)

| body | spec claims | measured at `8e605e1` |
| --- | --- | --- |
| reconcile, `hardened`, `chars: 100` | 181 drawn / headroom 339 | **181 / 339** ✓ |
| confirm modal, longest variant | 343 drawn / headroom 107 | **343 / 107** ✓ |
| §8h phrase form | 165 drawn / headroom 378 | **165 / 378** ✓ |
| §8h plain form | 133 drawn / headroom 397 | **133 / 397** ✓ |
| unlock, longest noun, two-digit index | 175 drawn / headroom 378 | **175 / 378** ✓ |

I also measured the arms the table does *not* row, in case "longest noun" was mis-chosen:
`"a hashlock preimage, not a seed"` and `"not a format this machine reads"` are both 31
characters and both give **175 / 378** at a two-digit index; `"an output descriptor"` gives
167 / 378. No arm is tighter than the gated row.

### Geometry, re-measured by raster (not by text)

```
band left=8 width=411; lead (407,44) = 2 line(s) of 23 px      (unmutated)
band left=8 width=411; lead (440,44) = 2 line(s) of 23 px      (M10: panel-wide wrap restored)
scanner sees ink at (435,55)                                    (probe control, unmutated run)
MaxHeight=209 grid=(340,182) gap=8 -> readout budget 19 px; one line is 19 px
```

§3.2(a) no lead ink under any nav rect — passes; (b) budget ≥ one line — passes at exactly 19;
(c) at most two lines — 2. §3.3's fallback copy correctly never fires. The test lays the lead
out through the *production* function `hashlockPhraseLead` at the production `top`
(`layout.Rectangle{Max: dims}.CutTop(leadingSize).Min.Y`, identical to `composer_hashlock.go:170`),
so it measures what the screen draws rather than its own arithmetic.

---

## 4. The §8h banner on every wallet shape

Built with a throwaway test in my own worktree (deleted; tree left clean), driving
`composerCopyHashEveryPathFor` on states constructed as `composerFlow` constructs them.

| wallet shape | §8h fires? | form | final sentence drawn | spec form | verdict |
| --- | --- | --- | --- | --- | --- |
| MIXED: one phrase path + one plate path | yes | PHRASE | `Back up every phrase and its method, and every preimage plate, separately.` | §2.5 | ✓ |
| ALL-PHRASE: two paths, two phrase digests | yes | PHRASE | same | §2.5 (overcounts the plate, RECORDED as deliberate) | ✓ |
| ALL-PHRASE: two paths sharing ONE phrase digest | yes | PHRASE | same | §2.4 "both are by-phrase" | ✓ |
| ALL-PAYLOAD: two different plate digests | yes | PLAIN | `Back up every preimage separately.` | §2.6 | ✓ |
| phrase path REMOVED, survivor given a hex hash | yes | PLAIN | same | §2.2/§2.3 (predicate walks paths) | ✓ |
| phrase digest RE-TYPED as 64 hex (same value, new pointer) | yes | PHRASE | phrase form | §2.4 "still by-phrase" | ✓ |
| one phrase path + one path with NO hash | **no** (guard closed) | — | not drawn | `composerEveryPathHashed` | ✓ |
| single phrase path | yes | PHRASE | phrase form | §2.5 (overcount, recorded) | ✓ |
| single plate path | yes | PLAIN | plain form | §2.6 | ✓ |

The two overcounting rows are the ones §2.5 names and rules on in writing, with the reasoning
carried in a comment beside the sentence (`composer_copy.go:512-519`) exactly as §2.5 requires
("recorded so it is not re-opened"). No shape produces an *under*count.

`hashByPhrase` has no code residue:

```
$ grep -rn "hashByPhrase" . --include=*.go
gui/composer_copy_test.go:158:// It exists because H5 §2 replaced composerState.hashByPhrase with a value set
gui/composer_provenance_test.go:12:// composerState.hashByPhrase was one bool for a whole composition: set at HOLD,
```

Both are historical prose. `composerHashByPhraseSync` and both call sites are gone.

---

## 5. Mutation table — 24 mutations, all fatal

Each was applied to my own detached worktree, run, and reverted with
`git checkout -- <file>` + `git diff --quiet` (verified `[restored]` every time; final tree
clean). Whole-`gui` runs used `gui-shard-test.sh ./gui/ 24`.

| # | mutation | test(s) that died | quoted failure |
| --- | --- | --- | --- |
| M1 | reconcile returns the old one-sentence body | `TestHashlockReconcileScreenCarriesTheDigestMethodAndChars`, `TestComposerCopyIsVerbatimFromTheSpec`, `TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal` | `the reconcile screen does not carry "hash  3cf5d421..b70a4c12"` |
| M2 | drop ONLY the mismatch sentence | same first two | `does not carry "If they differ, do not fund this wallet: build it again."` |
| M3 | restore the old write-down line | `TestHashlockPhraseRouteSetsTheCorpusDigest` (both subtests), `TestComposerCopyIsVerbatimFromTheSpec` | `the confirm modal's write-down line does not name the digest` |
| **M4** | **the hash assigned BEFORE the confirm** (order defect) | `TestHashlockBackContractKeepsThePath` | `composer_hashlock_test.go:464: hash assigned before HOLD` |
| M5 | `composerNotePhraseDigest` drops the nil-map check | 4+ tests | `panic: assignment to entry in nil map [recovered, repanicked]` |
| M6 | predicate returns `len(st.phraseDigests) > 0` | `TestComposerHashEditDispatchesByRowLabel`, `TestRemovePathThenAHexHashDrawsThePlainBanner`, `TestComposerAnyPathByPhraseIsPerDigest` (2 rows), `TestComposerHashEditToAPayloadRowDropsThePhraseForm` | `§8h names a phrase this composition no longer has` |
| M7 | predicate compares `p.Hash` POINTERS | 7 tests incl. all 4 positive predicate rows | `composerAnyPathByPhrase = false, want true` ×4 |
| M8 | restore the CHOICE (`or the preimage plate`) | `TestComposerMixedWalletBannerNamesEveryPhraseAndEveryPlate`, `TestComposerCopyIsVerbatimFromTheSpec` | `§8h's phrase form still offers a CHOICE of backups` |
| M9 | restore `Back the preimage up separately.` | `TestTwoPlateWalletBannerCountsEveryPreimage`, `TestComposerCopyIsVerbatimFromTheSpec` | `§8h's plain form still names ONE preimage on a two-plate wallet` |
| M10 | restore the panel-wide lead wrap | `TestHashlockPhraseLeadIsDrawnInsideTheBand` | `the phrase screen's lead is drawn UNDER a navigation button` (lead measured 440 px wide) |
| M11 | restore `CutBottom(8)` (F-481) | `TestHashlockPhraseScreenKeepsTheReadoutBudget`, `TestHashlockPhraseScreenDrawsTheMaskedReadout` | `readout budget is 11 px and one line needs 19` |
| M12 | drop the whole new unlock sentence | `TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable` (3 asserts), `TestUnlockNotPermittedBodyNamesTheRecordAndTheKind` (4 rows) | `the screen must say what to do next` |
| M13 | drop ONLY `(records count from 0)` | same two | `the screen must say the index is 0-based` |
| M14 | drop ONLY `-- and any others like it --` | same two | `the screen must say there may be more than one` |
| M15 | `walkOkAssignments` reads only the FIRST match | `TestWalkOkGuardReadsEveryAssignment/the_verdict_is_the_last_assignment` | `walkOkDriverSupplied found 0 caller-supplied term(s) [] in ["false"], want 1` and `allConst = true over ["false"], want false` — **byte-identical to the message the code comment predicts** |
| M16 | `composerFlowExit` moved between `composerFlow`'s doc comment and `composerFlow` | `TestComposerHelpersDidNotStealADocComment` | `composerFlow has NO doc comment` + `composerFlowExit's doc comment … opens "composerFlow is …"` |
| M17 | `composerFlowExit` no longer clears the hook | `TestComposerStateHookIsInstalledOnlyWhileAFlowRuns` | `the hook survived the composition it was installed for: []` |
| M18 | `composerFlow` never installs the hook | same | `the hook is not installed while composerFlow is running` |
| M19 | the hook hands out `st`'s own pointers | `TestComposerStateHookReportsEachPathAndHandsOutCopies` | `writing through the hook's pointer changed the POLICY: ff0102…` |
| M20 | the hook collapses nil holes | same | `the hook reports 1 entries for a 2-path composition` |
| M21 | reconcile header separator → one space | **only** `TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal` | `want prefix: "hash  b867db87..edbc96cb\nmethod: sha256   chars: 28\n"` |
| M22 | the tinygo stub exports `ComposerPathHashes` | `TestBuildTaggedHooksAreAbsentFromTheFirmwareImage` | `composer_state_hook_tinygo.go exports ComposerPathHashes -- that file IS the firmware` |
| M23 | `ComposerPathHashes` named in a second gui file | same | `composer_flow.go uses ComposerPathHashes in code but is not composer_state_hook.go` |
| M24 | the walk closes `out.ok = out.plates === 3` | `TestWalkOkContainsNoDriverSuppliedPlateCount` | `walk_hashlock_phrase.js's \`ok\` contains \`plates\`, which the CALLER supplies` |

**No survivors.** In particular M4 — the Critical-class order defect — is caught by
`TestHashlockBackContractKeepsThePath` in CI *as well as* by the walk's pre-hold `null` read,
so the seam is a second witness rather than the only one, which is what the spec claims for it.

M21 is worth calling out: it is the only gate on the three-space separator, because
`normalizeDrawn` collapses whitespace and so the copy table and the flow test both survive it.
The test's own doc comment says exactly this. It is right.

I did not separately mutate `TestHashlockPhraseLeadGeometryProbeCanSeeInk`; it *is* the control
for M10's scanner, it reports `scanner sees ink at (435,55)` on the passing run, and it carries
its own negative control (a label at the left margin must NOT be reported).

---

## 6. Records, mechanically checked

**H2 spec fold (`h5-records` `4e2cf01f`).** Compared against the shipped Go strings with a
throwaway whitespace-normalising test (deleted; tree clean), in both directions:

```
OK  §4.5 write-down sentences            OK  superseded text absent: "Write down this phrase and the method now"
OK  §4.5 reconcile body                  OK  superseded text absent: "They are not on this device and not on your plates"
OK  §4.5 reconcile header                OK  superseded text absent: "Before you fund this wallet, run ms hashlock"
OK  §4.7 phrase form                     OK  superseded text absent: "Back up the phrase and its method, or the"
OK  §4.7 plain form quote                OK  superseded text absent: "Back the preimage up separately"
```

**FOLLOWUPS (`4e2cf01f`).** Five closure headers, `<FORK_MERGE_SHA>` placeholder present in all
five as expected; two new follow-ups filed (F-491 H2 §4.5 reuse-block drift, F-492 the missing
manual unlock section). Four headers verified true (§2 table); F-485's is I-1.

**Toolkit manual (`h5-manual` `b48af1c1`).** The re-quoted confirm modal and reconcile screen
are byte-exact against the shipped bodies. The new claim *"the same run's engraving-card stderr
prints `phrase:          28 characters`"* resolves against the real source —
`mnemonic-secret/crates/ms-cli/src/cmd/hashlock.rs:352` emits `"phrase:          {n} characters
-- …"`, ten spaces, matching. The new Back-table row *"the reconcile screen | the spend-path
list | dropped; the hash is already assigned"* is true: `showError` dismisses on Back and OK,
`hashlockPhraseRoute` then returns `hashlockAssigned`, `composerHashEdit` returns `true`
(`composer_hash.go:204-206`), and the walk confirms the destination (`waitFor("Spend paths")`).

```
$ cd /scratch/code/shibboleth/tk-worktrees/h5-manual/docs/manual && make lint
…
[lint] === 5/6 glossary-coverage ===
[lint] === 6/6 index bidirectional ===
[lint] OK
```

**Firmware size, measured myself** (`nix develop -c tinygo build -size short -o /dev/null
-target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`):

| tree | flash | ram |
| --- | --- | --- |
| `8e605e1` (shipped) | **1,599,208** | 62,856 |
| `b9a9a30` (baseline) | **1,597,404** | 62,856 |
| `8e605e1` with the stub file and both call sites deleted | **1,599,224** | 62,856 |

Stage delta **+1,804 B flash / 0 B RAM**. Hook's share **−16 B**, i.e. no measurable cost —
reproducing `composer_state_hook_tinygo.go`'s two quoted numbers *to the byte*, and confirming
that "no measurable cost, not 0 B" is the honest form of the claim. My b9a9a30 measurement also
independently confirms report C's cross-check (C's Task-5-only tree measured 1,597,404 with the
hook, equal to the pristine baseline).

**Plan blocks vs tree:** `./scripts/h5-plan-blocks-vs-tree.sh … /scratch/code/shibboleth/.tmp/h5-review`
→ `55 blocks checked, 0 FAIL`.

**Whole gates at `8e605e1`,** all run by me:

```
gui: 1239 top-level tests, partition verified exhaustive 1239 == 1239, 24 shards, ok (30 s wall)
rest of tree: 54 ok, 0 FAIL
CGO_ENABLED=0 go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/ ./cmd/emu/  → 4 × ok
./scripts/test-32bit.sh   → GOARCH=386 exit 0; GOARCH=arm build exit 0
GOOS=js GOARCH=wasm go vet ./cmd/emu/  → clean
./cmd/emu/build.sh        → built emu.wasm (10873125 bytes)
gofmt -l over every H5-touched .go file → empty
```

---

## 7. The four implementation reports — every claim checked at the tips

| report | claim | measured | verdict |
| --- | --- | --- | --- |
| A | branch tip `6cd4f1331bc335031fe08950c94a1c7b5b78a0e2` | matches `git rev-parse h5-a` | ✓ |
| A | `git diff --stat b9a9a30 HEAD` = 12 files, 787 insertions, 94 deletions | `12 files changed, 787 insertions(+), 94 deletions(-)` | ✓ **exact** |
| A | `55 blocks checked, 18 FAIL`, every failure at plan line **1477 or later** | `55 blocks checked, 18 FAIL`; lowest failing line **1477** | ✓ **exact** |
| A | all three commits carry Signed-off-by + Co-Authored-By + Claude-Session | 3/3 trailers on `7b0868e`, `0e2d9ad`, `6cd4f13` | ✓ |
| B | tip `c1f0237b83c72daca7c51100e198ee59ae68fe63`; 2 files, 66 insertions, 4 deletions | `2 files changed, 66 insertions(+), 4 deletions(-)` | ✓ **exact** |
| B | whole gui on `h5-b`: `1225 == 1225`, all ok | `1225 top-level tests … RESULT: ok -- all 1225 tests` | ✓ **exact** |
| B | vet warnings pre-existing in untouched files | confirmed, `go.mod` unchanged | ✓ |
| C | tip `122a121c6ac2f30295004657de8d3a0ab8ee2816` | matches | ✓ |
| C | whole gui on `h5-c`: **1227 tests**, exhaustive | `1227 top-level tests … 1227 == 1227 … ok` | ✓ **exact** |
| C | rest of tree **54 packages ok, 0 FAIL** | `grep -c "^ok"` = 54, `grep -c "^FAIL"` = 0 | ✓ **exact** |
| C | the `cmd/emu` baseline was RED at `b9a9a30`, "8 walk scripts checked where 6 were" | at `b9a9a30`: `INCONCLUSIVE` ×2, `6 walk script(s) checked`, FAIL. At `8e605e1`: `8 walk script(s) checked`, PASS | ✓ **exact, including the line numbers** |
| C | hook share −32 B on the Task-5-only tree; with-hook 1,597,404 = the `b9a9a30` baseline | I measure `b9a9a30` = **1,597,404** | ✓ consistent |
| C | mutation anchors `h := hashlock.Digest(&x)` at `:64` and `d := h` at `:68`, one occurrence each, unmoved by the merge | at `8e605e1`: `:64` and `:68`, `grep -c` = 1 and 1 | ✓ **exact** |
| D | engrave `4e2cf01f…`, toolkit `b48af1c1…`; engrave diff 2 files, 32 insertions, 13 deletions | `2 files changed, 32 insertions(+), 13 deletions(-)` | ✓ **exact** |
| D | `make lint` → `[lint] OK` | reproduced | ✓ |

**No false count in any of the four reports.** Every number I could measure reproduced exactly.

### Deviations, one verdict each

| # | deviation | verdict |
| --- | --- | --- |
| A D-1 | plan had no block updating `composerCopyHashEveryPath`'s copy-table row; A updated the `verbatim` column and nothing else | **CORRECT AND NECESSARY.** M9 shows the row is what bites; without it Task 1's mutation 5 has nothing to catch and `TestComposerCopyIsVerbatimFromTheSpec` would have been red at Step 9. Minimal. |
| A D-2 | six RED line numbers 2/8/78 lines below the plan's quote (plan captured RED on the fully gated tree) | **ACCEPT.** Records only, disclosed with the mechanism; error identities, files and count identical. |
| B D-1 | plan says "three existing rows", table at `b9a9a30` has four; B read the gated tree to decide which row stays unmodified | **CORRECT.** Shipped table has 5 rows; 4 carry the sentence and the "record 0 -- records count from 0" row stays unmodified, which is what isolates the 0-based question. `TestUnlockNotPermittedBodyNamesTheRecordAndTheKind` passes on all 5 with headroom ≥ 378. |
| C D-1 | `gui/composer_doc_comment_test.go` deliberately NOT on `h5-c` (it cannot be green without Task 3), left for the controller | **CORRECT, AND THE CONTROLLER DID IT.** The file lands in `8e605e1` — the merge tip's own commit — the blocks checker passes it (55/55), and M16 proves it bites. |
| C D-2 | placed `nonInterfaceHookPairs` ABOVE the test's doc block rather than beneath it as the gate tree had it, because the gate tree reproduced fidelity I-3 a third time | **CORRECT, and it was the right call.** Verified at `8e605e1` with `go/ast`: `var nonInterfaceHookPairs` opens *"nonInterfaceHookPairs names every //go:build pair…"* and `func TestBuildTaggedHooksAreAbsentFromTheFirmwareImage` opens *"The guard over every optional hook…"*. Both own their own comment. C's suggestion to add that test to `composerDocOwners` remains open and is a judgement call; not a finding. |
| C D-3 | commit subject keeps the plan's "(0 firmware bytes)" while the measured share is −32 B | **ACCEPT, disclosed.** The shipped code comment says "no measurable cost", the numbers are in the commit body, and I reproduce −16 B at the merged tip. A commit subject is not a normative record here. |
| C D-4 | the `var (okPropRe, okAssignRe, okSetRe)` block has no plan block; taken verbatim from the gated tree | **ACCEPT.** The blocks checker is 55/55 at the merged tip, the three fragments that *do* have blocks pass byte-exact, and M15 proves the resulting guard bites in exactly the way its comment predicts. |
| D D-1 | implemented the manual's third edit (the `#### What Back does` qualifier + table row), which the brief summary omitted but the plan specifies | **CORRECT.** The added row is true (traced through `composer_hash.go:204-206`), and the qualifier *"before the hold"* is needed once a screen exists after the hold. |
| D D-2 | wrote the one connective sentence between the two H2 §4.5 fenced blocks, reusing established phrases | **ACCEPT.** The sentence asserts nothing the spec does not already state, and the two fenced blocks are byte-exact (§6). |
| D D-3 | closure-header prose is D's own | **THIS PRODUCED I-1.** Four of the five headers are true; F-485's is not. |

---

## 8. Counts and verdict

**0 Critical / 1 Important / 2 Minor / 4 Nit — NOT GREEN.**

The single Important is one false clause in an un-merged FOLLOWUPS closure header on
`h5-records`. It requires no code change, no re-implementation, and no gate re-run: replace
*"and picking the row by label"* with a true description of the index pick, in the same edit
that fills `<FORK_MERGE_SHA>`, and this review closes. Everything else — the copy, the
provenance model, the banner on nine wallet shapes, the geometry, the seam, the walk, the fit
budgets, the firmware delta, all 24 mutations, all 55 plan blocks, all four implementation
reports, the H2 spec fold and the toolkit manual — is GREEN.

*Scratch worktrees `/scratch/code/shibboleth/.tmp/h5-review`, `.tmp/h5-base`, `/tmp/h5a`,
`/tmp/h5b`, `/tmp/h5c`, `/tmp/h5emucheck` removed after this report was written.*
