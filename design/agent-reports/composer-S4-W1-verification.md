# S4 W-1 fold verification (targeted)

Reviewer: independent, targeted (sonnet). Scope: does fork commit `bc9dd6300676ba9970036a1b997eb453cce4e0b9`
(branch `composer-s4`, on `b77449db`) do what S4 journey-walk finding W-1 asks, can its test
fail, are the five retargeted walks exact old->new replacements, and did nothing else move.

All mutation testing was done in a `cp -r` copy of `/scratch/code/shibboleth/wt-composer-s4`
under `/scratch/code/shibboleth/.s4-verify/` (no `git checkout` ever run there; the copy was
deleted after verification). The worktree itself was never touched (`git status --porcelain`
before and after: clean).

## 1. `git show bc9dd63`: hunks do exactly what W-1 asks; no other production file changed

**VERIFIED.**

- `gui/composer_presets.go`: `composerPresetPick` now prepends `composerPresetBlankRow`
  ("Build my own paths") as `choices[0]`; `sel == 0` returns the empty `md.PathList{Wrapper: w}`,
  otherwise `presets[sel-1].list` (the `sel-1` indexing named in the brief).
- `gui/composer_flow.go`: the former single-shot `w, ok := composerWrapperPick(...)` /
  `if list, ok := composerPresetPick(...); ok { st.list = list }` is now a `for {}` loop:
  wrapper-pick -> preset-pick; on preset-pick `ok` it breaks with `st.list = list`; on
  preset-pick `!ok` (Back) it checks `ctx.Done` (returns if the context is tearing down) and
  otherwise loops back to `composerWrapperPick` -- i.e. Back on the preset picker returns to
  the wrapper choice, never falls forward.
- `git show --stat bc9dd63` lists exactly 6 files: `gui/composer_flow.go`,
  `gui/composer_flow_test.go`, `gui/composer_gates_test.go`, `gui/composer_join_test.go`,
  `gui/composer_presets.go`, `gui/composer_presets_test.go`. Of these, exactly two are
  non-test production files (`composer_flow.go`, `composer_presets.go`); the other four are
  `_test.go`. No other production file changed.

## 2. New test passes on the fix and fails under both named mutations, with the pasted messages

**VERIFIED.**

- On the unmodified copy: `go test -run '^TestComposerPresetPickerOffersBlankFirstAndBackReturnsToTheWrapper$' -v ./gui/` -> `PASS`.
- Mutation A (remove the blank-row append, i.e. delete
  `choices = append(choices, composerPresetBlankRow)` in `composer_presets.go`): same test ->
  `FAIL`, `composer_presets_test.go:308: the preset picker offers no blank row (W-1).` --
  exact match to the message pasted in the commit body. Reverted; copy diffed byte-identical
  to the worktree's `composer_presets.go` afterward.
- Mutation B (make the flow loop break regardless of `ok`, i.e. replace the `if ok { st.list =
  list; break }` / `if ctx.Done { return }` pair in `composer_flow.go` with an unconditional
  `st.list = list; break`): same test -> `FAIL`,
  `composer_presets_test.go:317: Back on the preset picker did not return to the wrapper choice (W-1).`
  -- exact match. Reverted; copy diffed byte-identical to the worktree's `composer_flow.go`
  afterward.

## 3. The five retargets are exact single-click old->new replacements; nothing else moved

**VERIFIED.**

`git diff bc9dd63~1 bc9dd63 -- gui/composer_flow_test.go gui/composer_gates_test.go
gui/composer_join_test.go` shows exactly 5 one-line hunks, each `- click(&ctx.Router, Button1)`
/ `+ click(&ctx.Router, Button3) // row 0 = Build my own paths (W-1)`, each immediately after a
`pumpUntil(frame, "Start from?", 24)` block: `composer_flow_test.go` x2
(`TestComposerNoPayloadWalkEngravesAKeylessTemplate`,
`TestComposerBackAtThePathListKeepsTheComposition`), `composer_gates_test.go` x1
(`TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit`), `composer_join_test.go` x2
(`TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen`,
`TestComposerBackAtTheMappingReviewKeepsTheSeatedKeys`). No other line in any of the three
files changed. All five named tests pass on the fix
(`go test -run '^(TestComposerNoPayloadWalkEngravesAKeylessTemplate|TestComposerBackAtThePathListKeepsTheComposition|TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit|TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen|TestComposerBackAtTheMappingReviewKeepsTheSeatedKeys)$' -v ./gui/`
-> all `PASS`).

## 4. Build-gate surface

**VERIFIED**, all items:

- `assertChoiceLabelFits(t, composerPresetBlankRow)` present at `composer_presets_test.go:283`,
  calling the pre-existing `assertChoiceLabelFits` (`multisig_build_prose_test.go:508`).
- `go test -count=1 ./cmd/emu/` -> `ok  seedhammer.com/cmd/emu  1.067s`; confirmed the new
  string `Build my own paths` has zero hits under `cmd/emu/`, so it is not a registered needle
  (as the commit claims).
- `go test -timeout 20m -count=1 ./...` -> all 54 test-bearing packages `ok`, zero `FAIL`/`panic`
  lines (grepped the captured log once); `seedhammer.com/gui` itself: `ok  110.088s`.
- Sharded gui count: `go test ./gui/ -list '.*' | grep -E '^(Test|Example|Fuzz)' | wc -l` = 1186
  on the fix. `git diff bc9dd63~1 bc9dd63 | grep -E '^\+func (Test|Example|Fuzz)'` shows exactly
  one addition (`TestComposerPresetPickerOffersBlankFirstAndBackReturnsToTheWrapper`) and zero
  removals -- so pre-fix was 1185, matching the brief's "was 1185 + 1 new" exactly.
  `scripts/gui-shard-test.sh ./gui/ 24 20m` run against the copy: "1186 top-level tests",
  "partition verified exhaustive: 1186 == 1186", all 24 shards `ok`, `RESULT: ok -- all 1186
  tests ran across 24 shards" (wall 23s).
- `go test -run '^TestComposer' -count=1 ./gui/` -> `ok`.
- `scripts/test-32bit.sh` (default `./sysw/`) -> `GOARCH=386 test: exit 0`,
  `GOARCH=arm build: exit 0`, script exit 0.
- oraclelive build: `go vet -tags oraclelive ./oracle/ ./gui/ ./sysw/` reproduces exactly the
  two pre-existing `testing.ArtifactDir requires go1.26 or later` findings named in the commit
  message (`gui/freetext_sizeproof_golden_test.go:111`, `gui/transaction_golden_test.go:104`);
  both files were last touched by commits `2ffb38c` / `4cee75d`, predating `bc9dd63` -- these
  findings are pre-existing, not introduced by the fix.
- js vet: `GOOS=js GOARCH=wasm go vet ./cmd/emu/` -> exit 0, no output.
- `git status --porcelain` in the mutation copy after all mutation/revert cycles: empty.
  `gofmt -l` on all 6 changed files: empty (gofmt clean).

## 5. Spec edit states the row and Back semantics the code implements

**VERIFIED.** `design/SPEC_wallet_policy_composer.md` §7b (commit `34c92bf`, this repo):
"The preset screen's FIRST row is the blank route, 'Build my own paths' ... Back on that screen
returns to the wrapper choice." Matches the code (row 0 = blank; Back loops to
`composerWrapperPick`) and matches `design/S4_journey_walk_2026-09-02.md`'s W-1 entry, updated
in the same commit, which cites the same fix commit `bc9dd63` and the same test name.

## Closing counts

0 Critical / 0 Important / 0 Minor / 0 Nit.

Every checked claim in the commit message and the brief reproduced exactly: hunk scope, both
named mutations' fail messages verbatim, all five retargets exact single-click replacements
with nothing else moved in those files, full gate surface (unit tests, sharded gui count,
`cmd/emu`, `-run '^TestComposer'`, `test-32bit.sh`, oraclelive vet, js vet, gofmt) green or
matching a documented pre-existing exception, and the spec edit matches the implemented
semantics.
