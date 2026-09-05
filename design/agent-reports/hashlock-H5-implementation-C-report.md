# Hashlock H5 — implementer C report (Task 5)

**Branch `h5-c`, tip `122a121c6ac2f30295004657de8d3a0ab8ee2816`**, one commit off fork
main `b9a9a30`, worktree `/scratch/code/shibboleth/.tmp/seedhammer-h5-c`. Nothing
pushed. Working tree clean at the tip.

Plan: `design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md` at engrave master
`5b77367` (`git diff 5b77367..HEAD` over the plan and the spec is empty, so the
plan was executed at the SHA the brief names). Spec `e03d8e7`.

Every number below comes from a run at this tip, captured to a file under
`/scratch/code/shibboleth/.tmp/h5-c-evidence/` and quoted from there; the filename
is given with each.

---

## 1. Result

| gate | result | evidence |
| --- | --- | --- |
| `gui` whole shard set, 24 shards | **ok — all 1227 tests**, partition verified exhaustive (`1227 == 1227`), wall 33 s | `30-gui-shards.txt` |
| the rest of the tree (`go list ./...` minus `/gui`) | **54 packages ok, 0 FAIL** | `31-rest-of-tree.txt` |
| `cmd/emu` (the baseline RED) | **fixed** — 8 walk scripts checked where 6 were | `00-baseline-emu-RED.txt`, `10-step8-emu-all.txt` |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | clean | `05-vet-js.txt` |
| `./cmd/emu/build.sh` | `built emu.wasm (10868902 bytes)` | `23-emu-build.txt` |
| firmware size, hook's share | **−32 B flash / 0 B RAM — no measurable cost** | `20/21/22-size-*.txt` |
| `gofmt -l` over my files | clean (three pre-existing dirty files are `gui/transaction*.go`, identical at `b9a9a30`) | `32-gofmt.txt`, `35-baseline-gofmt.txt` |
| `scripts/h5-plan-blocks-vs-tree.sh` over this tree | **13 of 13 Task 5 blocks PASS byte-exact**; the 14th is the file described in §5 | `40-blocks-check.txt` |
| mutations | 7 of the plan's 8 re-run and each bit with the predicted message; the 8th has no gate on this branch (§5) | `M1..M8.txt` |

Steps 1–11 done. **Step 12's three walk runs were NOT run** — the brief reserves
them for the controller at the merged tip. Their recipes are in §6, re-verified
against this tip and against the plan author's fully-merged gate tree.

---

## 2. RED before GREEN, per step

**The baseline RED (Step 6), reproduced on this worktree before any edit** —
`00-baseline-emu-RED.txt`, `CGO_ENABLED=0 go test -count=1 ./cmd/emu/`:

```
--- FAIL: TestWalkOkContainsNoDriverSuppliedPlateCount (0.00s)
    needle_test.go:525: INCONCLUSIVE: walk_h0_preimage.js has no `ok:` property this test can read, so nothing was checked for it — the walk's return shape changed and this guard did not
    needle_test.go:525: INCONCLUSIVE: walk_hashlock_phrase.js has no `ok:` property this test can read, so nothing was checked for it — the walk's return shape changed and this guard did not
    needle_test.go:563: 6 walk script(s) checked; no driver-supplied plate count in any `ok`
FAIL
FAIL	seedhammer.com/cmd/emu	1.096s
```

Byte-identical to the plan's quoted measurement, including the line numbers. Not
introduced by H5.

**Step 3, the seam's own gate, written first** — `01-step3-RED.txt`:

```
gui/composer_state_hook_test.go:30:13: undefined: ComposerPathHashes
gui/composer_state_hook_test.go:50:13: undefined: ComposerPathHashes
gui/composer_state_hook_test.go:69:13: undefined: ComposerPathHashes
gui/composer_state_hook_test.go:94:2: undefined: setComposerStateHook
gui/composer_state_hook_test.go:95:12: undefined: clearComposerStateHook
gui/composer_state_hook_test.go:97:9: undefined: ComposerPathHashes
FAIL	seedhammer.com/gui [build failed]
```

GREEN after Steps 1–2 — `02-step3-GREEN.txt`: both tests PASS, `ok 0.002s`.

**Step 5's RED comes for free the moment the pair exists**, exactly as the plan
predicts — `03-step5-RED.txt`, run after Steps 1–2 and before touching
`tinygo_split_test.go`:

```
--- FAIL: TestBuildTaggedHooksAreAbsentFromTheFirmwareImage (0.03s)
    tinygo_split_test.go:202: composer_state_hook.go declares no exported interface, so nothing about it is checked below -- if this pair is not an interface hook, say so here rather than leaving the scan silently vacuous
```

GREEN after Step 5 — `04-step5-GREEN.txt`, with the `t.Logf` the plan describes.

**Step 6's table test, written before its helpers** — `06-step6-tabletest-RED.txt`:

```
cmd/emu/needle_test.go:539:11: undefined: walkOkAssignments
cmd/emu/needle_test.go:540:21: undefined: walkOkDriverSupplied
FAIL	seedhammer.com/cmd/emu [build failed]
```

**Step 8's GREEN** — `09-step8-okguard.txt`, matching the plan's quoted output
including both line numbers:

```
=== RUN   TestWalkOkContainsNoDriverSuppliedPlateCount
    needle_test.go:647: walk_hashlock_phrase.js assigns `ok` nothing but the constant(s) true, so it restates no assertion (H5 §4.4)
    needle_test.go:693: 8 walk script(s) checked; no driver-supplied plate count in any `ok`
--- PASS: TestWalkOkContainsNoDriverSuppliedPlateCount (0.00s)
```

`ls cmd/emu/walk_*.js | wc -l` is **8**, unchanged by this task.

---

## 3. Every mutation, run once and reverted

Each was applied to the task state, run, and the file restored from a byte-compare
backup (`diff -q` confirmed the revert each time; the final `git status` is clean).

| # | mutation | measured failure | file |
| --- | --- | --- | --- |
| 1 | delete `clearComposerStateHook()` from `composerFlowExit` | `composer_state_hook_test.go:70: the hook survived the composition it was installed for: []` | `M1.txt` |
| 2 | delete `setComposerStateHook(st)` from `composerFlow` | `composer_state_hook_test.go:52: the hook is not installed while composerFlow is running` | `M2.txt` |
| 3 | the hook hands out `st`'s own pointers (`out[i] = p.Hash`) | `composer_state_hook_test.go:117: writing through the hook's pointer changed the POLICY: ff0102…1e1f, want 000102…1e1f` | `M3.txt` |
| 4 | the hook skips paths with no hash instead of leaving a hole | `composer_state_hook_test.go:99: the hook reports 1 entries for a 2-path composition` | `M4.txt` |
| 5 | `composer_state_hook_tinygo.go` exports `ComposerPathHashesOnDevice` | `tinygo_split_test.go:271: composer_state_hook_tinygo.go exports ComposerPathHashesOnDevice -- that file IS the firmware, so the host-only surface of this pair is in the image` | `M5.txt` |
| 6 | `ComposerPathHashes` named in another `gui` file (`_ = ComposerPathHashes` in `composer_flow.go`) | `tinygo_split_test.go:307: composer_flow.go uses ComposerPathHashes in code but is not composer_state_hook.go -- the interface and the type assertion that reaches it belong in the one file the firmware build drops` | `M6.txt` |
| 7 | `composerFlowExit` put back BETWEEN `composerFlow`'s doc comment and `composerFlow` | **no gate on this branch** — mechanism proven instead, see below | `M7-mechanism.txt` |
| 8 | `walkOkAssignments` reads only the first match (`FindStringSubmatch`) | `needle_test.go:607: walkOkDriverSupplied found 0 caller-supplied term(s) [] in ["false"], want 1` and `needle_test.go:611: allConst = true over ["false"], want false`, on the row `the_verdict_is_the_last_assignment` | `M8.txt` |

Every one matches the plan's Step 8 table verbatim, message for message.

**Mutation 7 has no test on this branch** because Step 3a's
`gui/composer_doc_comment_test.go` is not in my file list (§5). The *mechanism* is
proven directly with a `go/ast` reader over `gui/composer_flow.go`
(`M7-mechanism.txt`). With the block moved into the sandwich:

```
func composerFlowExit: doc opens "// composerFlow is \"Build a new policy\" (SPEC_wallet_policy_composer.md §7),"
func composerFlow: doc opens "<none>"
```

— the two conditions Step 3a's test reports as failures. At the task state, each
declaration owns its own comment:

```
func composerFlowExit: doc opens "// composerFlowExit is everything one composition must undo, in one deferred"
func composerFlow: doc opens "// composerFlow is \"Build a new policy\" (SPEC_wallet_policy_composer.md §7),"
```

---

## 4. Firmware size (Step 9)

`export PATH=/nix/var/nix/profiles/default/bin:$PATH`, then
`nix develop -c tinygo build -size short -o /dev/null -target pico-plus2
-stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`
(tinygo 0.41.1). Three builds on this tree, this box:

| build | code | data | bss | flash | ram | file |
| --- | --- | --- | --- | --- | --- | --- |
| **with the hook, one defer** | 1,565,552 | 31,852 | 31,004 | **1,597,404** | **62,856** | `20-size-with-hook.txt` |
| **the hook deleted from the tinygo view** (`composer_state_hook_tinygo.go` removed, `composerFlow`'s `setComposerStateHook` call and `composerFlowExit`'s `clearComposerStateHook` call removed) | 1,565,584 | 31,852 | 31,004 | **1,597,436** | **62,856** | `21-size-without-hook.txt` |
| positive control: the stub's `setComposerStateHook` given `println("hook")` | 1,565,728 | 31,852 | 31,004 | 1,597,580 | 62,856 | `22-size-positive-control.txt` |

**The hook's share is −32 B of flash and exactly 0 B of RAM.** The image WITHOUT
the hook is 32 bytes LARGER; nothing can cost negative flash, so what the pair
measures is that the hook contributes nothing the compiler does not reclaim
elsewhere, to within the granularity of a whole-image build. The **positive
control moves the image by +176 B**, so edits to that stub do reach it and the
zero is not an artefact of a build that ignored the edit.

Two cross-checks worth having:

* **The with-hook number equals the plan's `b9a9a30` baseline exactly** —
  1,597,404 B flash / 62,856 B RAM. This branch carries Task 5 only, and Task 5
  puts nothing else in the image, so the whole-image build agreeing to the byte
  with the pristine baseline is independent evidence for the same claim the
  subtractive pair makes.
* **The plan's "no measurable cost, not 0 B" fold is corroborated and extended.**
  The same pair has now measured **0 B** (pre-fold tree), **−16 B** (the plan
  author's post-fold full-stage tree) and **−32 B** (here, Task 5 alone). Three
  trees, three different values, all at or below zero: it is layout noise, exactly
  as `composer_state_hook_tinygo.go` says. A spec asserting an exact 0 would have
  been asserting the noise, and the fold that removed that assertion was right.

The brief asked for the hook's share to be **0 B**. Measured, it is 0 B of RAM and
**no measurable flash** (−32 B). I did not run the +96 B second-defer row or the
`b9a9a30` baseline build: the pair above plus the positive control settles the
question the brief asks, and the baseline is reproduced for free by the first row.

---

## 5. Deviations

**D-1 (the only one that needs the controller to act).
`gui/composer_doc_comment_test.go` — Step 3a — is NOT in this branch.**

The plan's File Structure and Task 5 Step 3a both put it in Task 5. **No
implementer brief lists it** — I grepped all four (`hashlock-H5-implementer-*.md`)
and the string does not appear in any of them; my brief's file list and its
one-line scope summary both omit it. That omission looks deliberate rather than an
oversight, because the file **cannot be green on a Task-5-only branch**: its
`composerDocOwners` map names `composerPageLines` and `composerTextBand`, which
`composer_paged.go` only declares after **Task 3** (implementer A). On `h5-c` the
test would emit `composer_paged.go declares no func composerTextBand; this list is
stale` and then `t.Fatalf("checked 3 of 4 symbols")`.

So I did not create it, and the blocks checker reports it as the one Task 5 block
absent from my tree (`40-blocks-check.txt`):

```
FAIL ...:1960  whole  gui/composer_doc_comment_test.go  -- no such file in the tree
```

**The controller must add it at the merged tip**, where Tasks 3 and 5 are both
present. The block is at plan line 1960 and is `mode=whole`, so it can be lifted
verbatim; both symbols it names on my side (`composerFlow`, `composerFlowExit`)
are already in the shape it demands, proven in §3's mutation 7.

**D-2. `nonInterfaceHookPairs` is placed ABOVE
`TestBuildTaggedHooksAreAbsentFromTheFirmwareImage`'s doc comment, not below it.**

The plan gives the fragment's text but not its placement. The plan author's gated
tree (`/scratch/code/shibboleth/.tmp/h5-gate`) puts the var and its comment
directly beneath that test's doc comment with no blank line — **which is r0
fidelity I-3 happening a third time.** Measured on the gate tree with the same
`go/ast` reader as §3:

```
var nonInterfaceHookPairs: doc opens "// The guard over every optional hook this package keeps OUT of the firmware"
func TestBuildTaggedHooksAreAbsentFromTheFirmwareImage: doc opens "<none>"
```

The var has taken the test's doc comment — the long "WHY THE PROPERTY MATTERS /
WHY IT IS A TEST AND NOT THE COMPILER" record, including the Critical that test
was written for. Step 3a's own gate cannot see it: `composerDocOwners` is a named
list of four `composer*` symbols and this is not one of them.

So I placed the var (byte-exact) above the test's doc block with a blank line
between, which is the fix Step 3a's own MUTATION note prescribes for the same
shape. Measured at my tip:

```
var nonInterfaceHookPairs: doc opens "// nonInterfaceHookPairs names every //go:build pair in gui whose host file"
func TestBuildTaggedHooksAreAbsentFromTheFirmwareImage: doc opens "// The guard over every optional hook this package keeps OUT of the firmware"
```

The fragment is still a verbatim substring, so the checker passes it
(`40-blocks-check.txt`, plan line 2153). **Two suggestions for the controller,
neither of them mine to make on this branch:** the gate tree carries the defect
and should not be the merge source for this file, and
`"TestBuildTaggedHooksAreAbsentFromTheFirmwareImage": "tinygo_split_test.go"`
would be a cheap fourth entry in `composerDocOwners` — except that the map's own
comment scopes it to `composer*` helpers, so it is a judgement call, not a
mechanical fix.

**D-3. The commit subject keeps the plan's "(0 firmware bytes)".** The measured
share is −32 B, i.e. no measurable cost. I kept the plan's message verbatim as
instructed and put the three measured numbers, the delta and the positive control
in the commit body, where they are the record.

**D-4. `cmd/emu/needle_test.go`'s new `var (…)` regex block has no plan block.**
The plan's three `needle_test.go` fragments reference `okPropRe`, `okAssignRe` and
`okSetRe` but no block declares them, and the rename `okExprRe` → `okPropRe` is
likewise uncarried. I took that block verbatim from the plan author's gated tree
(the brief permits reading it to resolve unclear context). The result:
`diff -u cmd/emu/needle_test.go /scratch/code/shibboleth/.tmp/h5-gate/cmd/emu/needle_test.go`
is **empty** — my file is byte-identical to the gated one. Same for
`cmd/emu/walk_hashlock_phrase.js`. Nothing was copied wholesale: the three plan
fragments were applied from the plan, and only the un-blocked var declaration came
from the tree.

**D-5. Pre-existing, not mine.** `gofmt -l gui/ cmd/emu/` lists
`gui/transaction.go`, `gui/transaction_golden_test.go`,
`gui/transaction_txrecord_test.go`, and `go vet ./gui/` reports two
`testing.ArtifactDir requires go1.26 or later (file is go1.25)` findings. Both
sets are **identical on a pristine `b9a9a30` export** (`35-baseline-gofmt.txt`,
`36-baseline-vet.txt`) and none of the files is in my task. Not fixed, since they
are outside my file list. `go test` is green regardless — the vet finding is a
tooling-version artefact of `go vet`'s per-file language version, not a
compilation failure.

---

## 6. Step 12 — the two mutation recipes, for the controller

Stated exactly as the plan states them. Both are one-line edits to
`gui/composer_hashlock.go`, each rebuilt with `./cmd/emu/build.sh`, each reverted
with `git checkout -- gui/composer_hashlock.go && git diff --quiet
gui/composer_hashlock.go` **before the next mutation and again after run (c)**,
with a final `./cmd/emu/build.sh` so the served `emu.wasm` is the unmutated one.

| run | edit | must |
| --- | --- | --- |
| (a) unmutated | — | PASS, `ok: true` |
| (b) the assignment moved before the confirm | replace `\t\t\th := hashlock.Digest(&x)` with `\t\t\th := hashlock.Digest(&x); st.list.Paths[idx].Hash = &h` | FAIL at §4.2's pre-hold read: `the path ALREADY holds a hash while the confirm modal is up` |
| (c) the stored hash perturbed by one byte | replace `\t\t\t\td := h` with `\t\t\t\td := h; d[0] ^= 1` | FAIL at the stored-versus-displayed assertion — `the stored digest does not abbreviate to the token the confirm modal drew: the screen showed one digest and the policy holds another.` — and at NO earlier one: the confirm modal still draws `3cf5d421..b70a4c12` from the unperturbed `h`, so `displayed` is that token, the pre-hold `null` read still passes, and the CORPUS assertion below is never reached |

**Both anchors verified to be unique and to survive the merge.** At my tip
`h := hashlock.Digest(&x)` is `gui/composer_hashlock.go:64` and `d := h` is `:68`,
one occurrence each; on the plan author's gated tree with all six tasks applied
they are at **the same two line numbers**, so Task 1's edits to that file do not
disturb either recipe.

Serving reminder from the walk itself: the browser caches `emu.wasm` and a
cache-buster on `index.html` does not help — **serve on a FRESH port**. The walk
throws `shComposerPathHashes is missing -- STALE emu.wasm` if the old binary is
served, so a stale run cannot be mistaken for a pass.

---

## 7. What landed

`git show --stat 122a121c`: 9 files changed, 720 insertions(+), 16 deletions(-).

```
A  cmd/emu/composer_js.go              (58 lines, whole, byte-exact)
M  cmd/emu/needle_test.go              (3 fragments + the un-blocked regex var)
M  cmd/emu/platform.go                 (1 fragment: installComposerAPI())
M  cmd/emu/walk_hashlock_phrase.js     (481 lines, whole, byte-exact; was 331)
M  gui/composer_flow.go                (2 fragments: composerFlowExit, the install)
A  gui/composer_state_hook.go          (86 lines, whole, byte-exact)
A  gui/composer_state_hook_test.go     (120 lines, whole, byte-exact)
A  gui/composer_state_hook_tinygo.go   (52 lines, whole, byte-exact)
M  gui/tinygo_split_test.go            (2 fragments; placement per D-2)
```

Trailers on the commit, all three verified present by `git log -1 --format=%B`:
`Co-Authored-By: Claude Fable 5.1`, `Claude-Session: …session_01Fs3bg7TRfuSaFcCEkskwXA`,
`Signed-off-by: Brian Goss <goss.brian@gmail.com>` (`-s`, DCO).

No file outside the brief's list was touched. No `main` commit. Nothing pushed.
No sub-agents. No `.jsonl` read. No phrase or preimage bytes are in any captured
file — the walk's corpus constants are public digests and the only hex quoted
anywhere above is the test's synthetic `0x00..0x1f` fixture.
