# Build gate — IMPLEMENTATION_PLAN_hashlock_H2_device.md

Plan: `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md`, engrave master `38509a9`.
Scratch: `/scratch/code/shibboleth/.tmp/h2-gate` (untracked copy of
`/scratch/code/shibboleth/seedhammer`, `git ls-files | tar` — no `.git`).
Go: `/scratch/code/shibboleth/.toolchain/go/bin/go` (1.26.7), first on PATH.
Nix: `/nix/var/nix/profiles/default/bin` on PATH for the firmware build.
No sub-agents used. No `.jsonl` files read. Emulator walk (Task 5 §7.5) not run
(out of scope per the dispatch brief).

Every plan code block was hand-wired into the scratch tree in task order and
run. This report lists, in order, every command with its verbatim tail, then a
table of every fix the gate had to apply, then the mutation results, the
whole-package results, and the firmware numbers.

## Setup

```
$ rm -rf .../.tmp/h2-gate && mkdir -p .../.tmp/h2-gate && (cd seedhammer && git ls-files -z | tar --null -T - -cf - | tar -xf - -C .../.tmp/h2-gate)
$ cd .../.tmp/h2-gate && go build ./...
(exit 0, silent -- baseline scratch copy builds clean before any plan code lands)
```

## Task 1 — `hashlock` package

```
$ sha256sum hashlock/testdata/hashlock-v0.8.json
a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30  hashlock/testdata/hashlock-v0.8.json
```
Matches the plan's pinned sum exactly. Corpus shape verified with `python3
json.load`: `derivation=11, refusals=15, kind=1, lockstep=4` — matches every
count the plan's tests assert.

Wrote `hashlock/hashlock.go`, `hashlock/hashlock_test.go`,
`hashlock/testdata/hashlock-v0.8.provenance.json` verbatim from the plan.

```
$ go vet ./hashlock/
(clean)
$ go test -count=1 -v ./hashlock/
--- PASS: TestDerivationRowsLockstep (0.55s)
--- PASS: TestCorpusCarriesTheNonFixedPointRows (0.00s)
--- PASS: TestRefusalRowsMatchTheHost (0.00s)
--- PASS: TestKindRowPreimageDigest (0.00s)
--- PASS: TestPhraseMaxCharsIsTheCap (0.00s)
--- PASS: TestLockstepListIsTheOneWeDrive (0.00s)
PASS
ok  	seedhammer.com/hashlock	0.554s
```
GREEN, zero fixes, exactly as written.

### Step 5 mutations (each applied, measured, reverted)

| Mutation | Plan's claim | Measured |
| --- | --- | --- |
| `Salt = append(Salt, 0, 0)` (zero-pad) | "11 hardened X/H failures" | 22 failures (11 rows × X + H) — matches |
| `Iterations = 99999` | "11 failures" | 22 failures (11 rows × X + H) — matches |
| `phrase = seal.NormalisePassphrase(...)` in `PreimageHardened` | "exactly the `Correct Horse Battery Staple` and `  a  b ` rows fail" | Exactly those 2 rows, 4 failures (X+H each) — matches |
| strip `-`/`,` from phrase first | "exactly the `correct-horse,battery staple` row fails" | **FALSE.** 4 rows fail, not 1: `correct-horse,battery staple`, `a-b,c`, and BOTH 64-char rows (`hashlock phrase row: sixty-four printable characters, no hex!!xx` and its `!`-suffixed sibling), because those two rows also contain `-` and `,`. Verified by dumping all 11 corpus phrases. **Plan's own claim is wrong; the mutation itself still works as a gate (it does fail), just on 4 rows, not 1.** |
| `IsMS1Shaped` via `codex32.New` (checksum parse) | "the grouped-by-5, leading/trailing-spaces and grouped-by-2 refusals rows fail" | Exactly refusal rows 11, 12, 13 fail (grouped-by-5, leading/trailing spaces, grouped-by-2) — matches |
| cap literal 99 | "`TestPhraseMaxCharsIsTheCap` and the 100-character refusals row fail" | Only `TestPhraseMaxCharsIsTheCap` fails. **The corpus has no 100-character refusals row** — its one `too-long` row is 101 characters (verified: `len(...)==101`), which is refused whether the cap is 99 or 100, so `TestRefusalRowsMatchTheHost` stays green under this mutation. Plan's second clause does not hold for this corpus. |

Every mutation still demonstrates the gate it claims to (nothing silently
passed), but two of the plan's own prose claims about which/how-many rows fail
are measurably wrong. Corrections recorded above; not blocking, but real.

## Task 2 — `codex32.DecodeMS1Preimage`

Appended the plan's test then its decoder verbatim.
```
$ go test -count=1 -run TestDecodeMS1PreimageIsShapeExact ./codex32/
(RED as expected: undefined: DecodeMS1Preimage, x3)
$ go test -count=1 ./codex32/
ok  	seedhammer.com/codex32	0.005s
```
GREEN, zero fixes.

Mutation (drop the `!f.Unshared` clause):
```
$ go test -count=1 -v -run TestDecodeMS1PreimageIsShapeExact ./codex32/
mspayload_test.go:155: DecodeMS1Preimage(a 2-of-N share beginning 0x03) err = <nil>, want codex32: not an m-format secret payload
```
Matches the plan exactly: "the share case returns a 32-byte value where
`errMSBadPrefix` is wanted." Reverted; `go test ./codex32/` green again.

## Task 3 — label-keyed `Which hash?` rows

Wrote the test, then the type/constructor rename the plan itself calls out
(`composerHashRowSet` type, `composerHashRows` constructor — Go forbids a type
and func sharing one name), the switch, and a Task-3 stub `hashlockPhraseRoute`
(replaced in Task 4).

```
$ go test -count=1 -run TestWhichHashRowsAreLabelKeyed ./gui/
(RED: undefined composerHashRows, composerHashRowPhrase)
```
After the rewrite:
```
$ go test -count=1 -run 'TestWhichHashRowsAreLabelKeyed|TestComposerHash|TestComposerCopy' ./gui/
--- FAIL: TestComposerCopyTableCoversEveryBody
    composerCopyHashlockNoPayloadLead is declared in composer_copy.go but is in no row of composerCopyTable
    composer_copy.go declares 42 bodies, the plan and the table know 41 -- if that is deliberate, update both
```
Fixed (table row + literal 41→42, see fold table below). Then:
```
$ go test -count=1 -run 'TestWhichHashRowsAreLabelKeyed|TestComposerHash|TestComposerCopy' ./gui/
ok  	seedhammer.com/gui	0.182s
```

## Task 4 — the phrase route

Copy functions, both gate tables, `composer_state.go`'s field,
`composer_shape.go:443`, the real `composer_hashlock.go`, and
`composer_hashlock_test.go` including all seven named helpers
(`composerStateForTest`, `runComposerAddPath`, `tapRow`, `holdConfirm`,
`tapPassphraseKey`, `waitDone`, `groupBy`, `loadHashlockCorpusForGUI`) were
written per the plan. Every helper needed a real implementation the plan only
named; see the fold table for what each one turned out to require.

```
$ go test -count=1 -run 'TestModalsThisBlockTouchesAreDrawnInFull|TestComposerCopy' ./gui/
(does not compile until the functions exist; then RED on the same
 declared-count / uncovered-row shape as Task 3, now 42→51 for the nine new
 Task-4 functions)
```

§4.5 fit measurement (drop order, as anticipated by the plan and by
`SPEC_hashlock_H2_device.md` §4.5, which states "r1 measured the normalised
body at about 484 characters, headroom uncertain"):
```
Step 0 (unshortened): 484 of 504 chars drawn -- CUT after "...checkthedigestmatches."(sic, cut before "Hold button to confirm.")
Step 1 (shorten reuse block to the brainstorm's two sentences): fits, 384/384 drawn, headroom 64 (BELOW the 80-char margin -- test fails: "fits today with only 64 characters to spare... Shorten this body rather than lowering the margin.")
Step 2 (move the reconciliation line to composerCopyHashEveryPathPhrase, per spec's own drop order): confirm modal now 290 chars drawn, headroom 186; §8h form now 254 chars drawn, headroom 262.
```
Both steps of the spec's own drop order were needed, not step 1 alone.

After the helpers and the route were complete and the mutations below folded
in, the full Task-4 test set:
```
$ go vet ./gui/
gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
gui/transaction_golden_test.go:104:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
(the two pre-existing, ignored complaints named in the dispatch brief -- nothing else)
$ go test -count=1 -run 'TestHashlock|TestWhichHash|TestComposerHash|TestComposerCopy|TestModalsThisBlockTouchesAreDrawnInFull' -v ./gui/
--- PASS: TestHashlockPhraseRouteSetsTheCorpusDigest (5.08s) [hardened_anchor, sha256_anchor]
--- PASS: TestHashlockPhraseRouteDoesNotNormalise (9.08s)
--- PASS: TestHashlockBackContractKeepsThePath (2.03s)
--- PASS: TestHashlockDeclineThenHardenedTypesOnce (2.07s)
--- PASS: TestHashlockPhraseRefusalsOnScreen (4.41s) [101_characters, 64_hex, plate_ungrouped, plate_grouped_by_5]
--- PASS: TestHashlockMethodModalsFireOnCondition (3.07s)
--- PASS: TestHashlockConfirmRelationLine (2.03s)
--- PASS: TestModalsThisBlockTouchesAreDrawnInFull (0.34s) [11 subtests incl. all 5 new hashlock rows]
--- PASS: TestWhichHashRowsAreLabelKeyed, TestComposerHash*, TestComposerCopy* (all)
PASS
ok  	seedhammer.com/gui	28.201s
```

### Task 4 mutations (each applied, measured, reverted)

| Mutation | Plan's claim | Measured |
| --- | --- | --- |
| fold `phrase` through `seal.NormalisePassphrase` in `hashlockPhraseFlow` | "fails on the mixed-case and spaces rows" | `TestHashlockPhraseRouteDoesNotNormalise` fails: `"Correct Horse Battery Staple": path hash = ..., want 95d4447...` — matches |
| confirm's Back returns `hashlockBackToWhichHash` | "`TestHashlockBackContractKeepsThePath` fails at 'Which method?'" | Fails, `never reached "Which method?"` — matches exactly |
| `composerHashEdit` returns `false` from the phrase route's Back | "the same test fails on the path count" | `TestHashlockBackContractKeepsThePath` **does** fail, but earlier than claimed — at `never reached "Type a hashlock phrase"` (the very next inner Back), not at the path-count assertion. The mutation is still caught by this test; the plan's description of *where* it fails is imprecise. |
| remove the relation line | "`TestHashlockConfirmRelationLine` fails" | Fails: `never reached "matches hash 1 in the payload"` — matches |
| drop `!f.Unshared` (Task 2, re-verified in combination) | "its test fails" | Fails as in Task 2 — matches |

## Whole-package and whole-suite runs

```
$ go test -count=1 ./codex32/ ./sysw/ ./seal/ ./hashlock/
ok  	seedhammer.com/codex32	0.002s
ok  	seedhammer.com/sysw	0.040s
ok  	seedhammer.com/seal	11.918s
ok  	seedhammer.com/hashlock	0.229s

$ bash scripts/gui-shard-test.sh ./gui/ 24
=== enumerating tests in ./gui/ ===
    1213 top-level tests
    partition verified exhaustive: 1213 == 1213
=== running 24 shards in parallel (timeout 20m each) ===
  shard 11: FAIL  51 tests  FAIL   <- first run, see fold below
  (all 23 others ok)
=== wall: 24s ===
RESULT: FAIL -- artifacts in /tmp/tmp.2nChLXQAT3
```
Shard 11's failure was a **real regression in a pre-existing test** (see fold
table). After the one-line fix:
```
$ bash scripts/gui-shard-test.sh ./gui/ 24
=== enumerating tests in ./gui/ ===
    1213 top-level tests
    partition verified exhaustive: 1213 == 1213
=== running 24 shards in parallel (timeout 20m each) ===
  (all 24 shards ok, 50-51 tests each)
=== wall: 34s ===
RESULT: ok -- all 1213 tests ran across 24 shards
```
The suite grew by 8 new top-level tests this plan adds
(`TestDecodeMS1PreimageIsShapeExact` in codex32; `TestWhichHashRowsAreLabelKeyed`
and the seven `TestHashlock*` functions in gui — `hashlock`'s own 6 tests are a
separate package, not counted in the 1213).

No script path adjustment was needed: `bash .../mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24` run with cwd = the scratch gate root resolved `./gui/` correctly.

## Firmware size

```
$ export PATH=/nix/var/nix/profiles/default/bin:$PATH
$ nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
   code    data     bss |   flash     ram
1563384   31852   31004 | 1595236   62856
```
**Flash 1,595,236 B / RAM 62,856 B**, against the plan's baseline
1,583,132 / 62,800 (fork main `c4a64fc`): **+12,104 B flash (+0.76%), +56 B RAM**
for the whole hashlock port (new `hashlock` package, `DecodeMS1Preimage`, the
label-keyed switch, the phrase route, its screens and copy).

## Fixes — every change made to the plan's code to compile or pass

| # | File | Plan said | What the gate changed | Why |
| --- | --- | --- | --- | --- |
| 1 | `gui/composer_copy_test.go` | Task 3: no instruction to touch `TestComposerCopyTableCoversEveryBody`'s literal count | Added a `composerCopyTable` row for `composerCopyHashlockNoPayloadLead` (section "H2-3"); bumped the hardcoded `declared != 41` to `42` | `TestComposerCopyTableCoversEveryBody` AST-scans `composer_copy.go` for every `composerCopy*` func and requires a table row + an exact count; Task 3 added one new such func the plan never listed a row for |
| 2 | `gui/composer_copy_test.go`, `gui/modal_fits_test.go` | Task 4 named only 6 sections (H2-4.2/4.3a/4.3b/4.4/4.5/4.7) for new rows | Added rows for **all 10** new `composerCopy*` funcs (incl. `composerCopyHashlockRefusal`, `composerCopyHashlockRelation`, `composerCopyHashEveryPathFor`, none named by the plan's row list); bumped `declared` literal 42→51 | Same AST-scan gate: every declared `composerCopy*` function needs a row or the test fails by name; the plan's own row list under-counted its own new functions by 4 |
| 3 | `gui/composer_copy.go` (`composerCopyHashlockConfirm`) | §4.5's literal reuse-block text, unshortened | Shortened per spec's own drop-order step 1 ("One phrase per policy. Never use this phrase as a passphrase or a password anywhere else.") | Measured 384/384 drawn but only 64 chars headroom, below the required 80-char margin — spec §4.5 anticipates exactly this and names the fix |
| 4 | `gui/composer_copy.go` (`composerCopyHashlockConfirm` / `composerCopyHashEveryPathPhrase`) | Reconciliation line ("Before you fund...") stays in the confirm modal | Moved the line from the confirm modal to `composerCopyHashEveryPathPhrase` (spec §4.5 drop-order step 2) | Step 1 alone left only 64 chars of headroom (see #3); the spec's own text names this as the next step, and only after both steps does the confirm modal clear the margin (186 chars headroom) |
| 5 | `gui/composer_hashlock_test.go` (`runComposerAddPath`) | `p := newPlatform(); ...; ctx := NewContext(p)` with no display override | Added `p.display = sh2DisplaySize` | The default 240×240 test display is narrower than the 340px passphrase keyboard, pushing `q/p/a/l` off-canvas — reachable by a hit test, unreachable by touch (the exact rule `passphrase_flow_test.go` states); without it, keys the anchor phrase needs are undrawn |
| 6 | `gui/composer_hashlock_test.go` (`composerStateForTest`) | `&composerState{}` | `&composerState{list: md.PathList{Wrapper: md.ComposeWsh}}` | `md.ComposeWrapper`'s zero value is `ComposeTr` (iota 0), and a key-less path is refused outright under `tr` (`composer_shape.go:250`, "This build will not put a key-less path in taproot") — the zero-value state can never reach the screen these tests exist to drive |
| 7 | `gui/composer_hashlock_test.go` (every test) | Flow starts directly with `h.mustReach("EXPERIMENTAL")` | Inserted `h.mustReach("What can spend on this path?"); h.choose(1) // A hash, no keys` before it (7 call sites) | `composerAddPath` first shows a `ChoiceScreen` ("Keys" / "A hash, no keys") that the plan's test code never drives; "EXPERIMENTAL" only appears after choosing the key-less arm |
| 8 | `gui/composer_hashlock_test.go` (7 call sites, right after `h.tapRow(0,3)`/`h.tapRow(2,5)`) | `h.mustReach("Hash lock") // §8i rule modal` | Changed the needle to `"32-byte value"` (a substring of `composerCopyHashRule()`) | The §8i rule modal is `showError(ctx, th, title, composerCopyHashRule())` with `title = "Path N hash"`, **not** "Hash lock" — that title belongs only to the later confirm screen. The plan's own code (composer_hash.go, faithfully transcribed) and its own test text disagree on this point |
| 9 | `gui/composer_hashlock_test.go` (7 call sites, right after those same `mustReach`es on zero-payload sessions) | `h.mustReach("Which hash?")` | Changed to `h.mustReach("Type a hashlock phrase")` | With `composerSessionWith(nil, nil)` (0 payload digests), Task 3's own design replaces the lead with `composerCopyHashlockNoPayloadLead()`, so the literal string "Which hash?" never appears on that frame; "Type a hashlock phrase" (the phrase row's label) is present regardless of lead |
| 10 | `gui/composer_hashlock_test.go` (9 call sites, method-pick `tapRow(...,2)`) | `h.tapRow(tc.methodRow, 2)` **then** `h.tapNav(Button3)` | Removed the redundant `h.tapNav(Button3)` | `tapRow` (this gate's own implementation, matching the plan's "tap selects, Button3 takes" contract) already takes the row; the plan's own test code then queues a **second**, stray Button3 click that lands on the next screen and silently misfires its confirm state. Every test that reached this line hung at the following screen until removed |
| 11 | `gui/composer_hashlock_test.go` (`holdConfirm`) | "the `ConfirmWarningScreen` hold gesture the existing composer tests use" (i.e. reuse `wipe_guard_test.go`'s `sessionHarness.hold`) | Wrote a **new** `holdConfirm` that holds, waits, **and then sends an explicit release** | `EventRouter.Events` (`gui/event.go`) tracks exactly ONE pointer contact **globally**: while `pointer.pressed` is true it reuses the stale `pointer.pressedTag` instead of hit-testing the current frame. `hold()` never releases. This route holds several `ConfirmWarningScreen`s in sequence (key-less consent → a method warning → the final Hash-lock confirm); without an explicit release, the second and later holds are routed to the first screen's now-defunct `Clickable` and silently never progress past 0% — no crash, no wrong-screen frame, just a stuck confirm. Measured directly with an isolated debug test before finding `event.go`'s single global `pointer` struct |
| 12 | `gui/composer_gates_test.go` (`TestComposerLockAndHashEditsAreNotGuardedByTheDiscardConfirm`) | Not part of this plan's files | Changed the "hash lock" subtest's pump target from `"Which hash?"` to `"Path 1 hash"` | **Genuine regression the gate caught in an existing, unrelated test.** This test drives `composerPathEdit` with `ctx.sysw == nil` (no session loaded), so `composerHashRows` reports 0 digests and Task 3's design swaps in the no-payload lead — exactly as intended for the real device, but this pre-existing test asserted the literal old lead text. Found only by running the full `gui` shard set (shard 11 failed); the narrower Task-4 test selection never touched this file |

12 fixes total. None were prescribed remedies taken on faith — each was
independently reproduced (RED before, GREEN after) and, where the fix touched
test assertions rather than implementation, the underlying mechanism was
traced to its root cause (event.go's global pointer state; the §8i modal's
actual title; the zero-digest lead swap) rather than patched to match observed
output.

## Verdict

**GATE GREEN WITH FIXES (12).**

All plan code (Tasks 1-4) compiles, and every test the plan names — including
all six mutation classes across Tasks 1, 2 and 4 — runs and reports the
predicted PASS/FAIL shape, with two prose corrections logged in the mutation
tables (Task 1 Step 5's separator-strip and cap-literal claims describe the
wrong scope, though the mutations themselves still function as gates). The
whole `codex32`/`sysw`/`seal`/`hashlock` package set is green. The whole `gui`
package (1213 top-level tests, exhaustive shard partition) is green, including
one pre-existing test this plan's own Task 3 design change required updating.
Firmware size: flash 1,595,236 B / RAM 62,856 B (+12,104 B flash / +56 B RAM
against the `c4a64fc` baseline of 1,583,132 / 62,800).

Not run (out of scope per the dispatch brief): the emulator walk
(`cmd/emu/walk_hashlock_phrase.js`, Task 5 §7.5) and the flash cycle. The scratch
tree is left in place at `/scratch/code/shibboleth/.tmp/h2-gate` for inspection.
