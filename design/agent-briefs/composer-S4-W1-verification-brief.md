You are the INDEPENDENT fold-verification reviewer (targeted) for one finding from the S4 journey walk on the SeedHammer II: W-1 in `/scratch/code/shibboleth/mnemonic-engrave/design/S4_journey_walk_2026-09-02.md`. The fix is fork branch `composer-s4`, commit `bc9dd6300676ba9970036a1b997eb453cce4e0b9` on `b77449db` (worktree `/scratch/code/shibboleth/wt-composer-s4`). One question: does the fix do what W-1 asks (the blank route is a visible FIRST row "Build my own paths"; Back on the preset picker returns to the wrapper choice, never forward into the path list), can its test fail, are the five retargeted walks exact old->new replacements, and did nothing else move?

Read-only: copy the worktree with `cp -r` to `/scratch/code/shibboleth/.s4-verify/` for every mutation (never dirty the worktree; do not run `git checkout` in a copied worktree -- it shares the gitdir). Go: `/scratch/code/shibboleth/.toolchain/go/bin` on PATH, `CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`, `-mod=readonly`, `TMPDIR=/scratch/code/shibboleth/.tmp`. Do NOT spawn sub-agents; commit nothing; read no `.jsonl` file.

## Verify
1. `git show bc9dd63`: hunks in `gui/composer_presets.go` (the row, first; `sel-1` indexing) and `gui/composer_flow.go` (the wrapper/preset loop; `ctx.Done` exit) do exactly the above; no other production file changed.
2. The new test `TestComposerPresetPickerOffersBlankFirstAndBackReturnsToTheWrapper` passes on the fix and FAILS under each named mutation (remove the blank-row append; make the loop `break` regardless of `ok`) with the W-1 messages the commit pastes; revert.
3. The five retargets (`composer_flow_test.go` x2, `composer_join_test.go` x2, `composer_gates_test.go`): each replaces exactly one `click(&ctx.Router, Button1)` immediately after the `"Start from?"` pump with `Button3`, and each test still passes; nothing else in those files changed.
4. `assertChoiceLabelFits` covers the new label; `go test -count=1 ./cmd/emu/` ok (the new string is no registered needle); `go test -timeout 20m ./...` all ok; sharded gui count (was 1185 + 1 new); `-run '^TestComposer'` ok; `scripts/test-32bit.sh`; oraclelive build; js vet.
5. The spec edit (mnemonic-engrave 34c92bf, §7b) states the row and Back semantics the code implements.

Severity: a hunk outside W-1, a test that cannot fail, or a retarget that changed more than the one click = Important; cosmetic = Minor/Nit. Do not pad.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-S4-W1-verification.md` (create; must not exist): per item VERIFIED / NOT VERIFIED with output; closing counts. Return a two-line summary plus the path.
