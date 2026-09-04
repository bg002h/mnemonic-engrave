You are the INDEPENDENT verifier of a FOLD, round 3, on the SeedHammer II composer. This is a TARGETED check, not a fresh audit — do not re-open closed findings and do not re-audit the composer at large.

Round 2 (`design/agent-reports/composer-S4-W6-fold-verification.md`, opus) returned **0C/2I/2M/1N** against `818220d8` and said DO NOT MERGE. This brief is about the controller's response to it, on fork branch `composer-s4e` at `177b490679228f25142f020e9b67851dcedd0fe8` (worktree `/scratch/code/shibboleth/wt-composer-s4e`), base fork main `70008da5f935b36635a442cb2738f8dcc2fce7f1`.

ONE QUESTION: did the fold close I-2, I-3, M-2 and M-3 without introducing anything — and **can its new test actually fail?**

Read-only: copy the worktree with `cp -r` to `/scratch/code/shibboleth/.s4e-verify3/` for mutations (never dirty the worktree; no `git checkout` in a copy — it shares the gitdir). Go: `/scratch/code/shibboleth/.toolchain/go/bin` on PATH, `CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`, `-mod=readonly`, `TMPDIR=/scratch/code/shibboleth/.tmp`. Sharded gui: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`. Firmware needs `/nix/var/nix/profiles/default/bin` on PATH first. Do NOT spawn sub-agents; commit nothing; read no `.jsonl` file.

## Already settled — do not re-derive

C-1, I-1 and M-1 were verified FIXED in round 2, including the structural closure of the signature (0 equal-signature renumbering pairs over 4,828 composable lists). `git diff 818220d8..177b4906` is the only thing under review here.

## What the fold does

`composerEditCanRenumber` now takes a `composerShapeField` and varies ONLY that field — lock cleared vs `older(1)` for the lock arm, hash cleared vs a fixed digest for the hash arm — instead of clearing the hash in both variants while toggling the lock. Each arm in `composerPathEdit` passes its own field. M-2 and M-3 are comment corrections. N-1 was filed as F-471, not fixed.

## Verify

1. **I-2 closed.** Round 2's reproduction: wsh, `[{2-of-2}, {key-less + hash}]`, both slots seated, `Path 2` → `Hash lock` → `No hash lock`. §8j must be drawn, and **declining must keep both seats and the hash**. Then confirm the ACCEPT path also behaves (seats discarded, as the safe direction). Run round 2's own pre-fold comparison if it is cheap.
2. **I-3 closed.** tr, `[{1 key + lock + hash}, {1 key}]` and round 2's `[{1 key + hash lock}, {1 key}]` shape: a lock edit on a path carrying a hash must ask NOTHING, and the lock must remain editable. A confirm that fires here is the defect returning.
3. **THE NEW TEST IS THE MAIN ITEM — hunt a false PASS.** `TestComposerEditCanRenumberIsExactOverEveryReachableShape` claims 3,708 cases with 0 false negatives and 0 false positives, using an "independent oracle". Attack that claim:
   - Is the oracle genuinely independent of the probe, or does it reduce to the same two-point comparison? Read both. If the oracle called `composerEditCanRenumber`, or varied the field the same way the probe does, the test would pass vacuously **and would have passed on the broken probe too** — check by running the test against the round-2 probe restored in a copy: the controller measured 156 false negatives / 288 false positives there, so a test that stays green on that probe is a false PASS and is Critical.
   - Does the corpus actually reach the shapes that mattered? Confirm key-less paths, tr paths with hashes, and multi-key paths are all present and that `checked` is in the thousands. Shrink the corpus in a copy and confirm the `checked < 1000` guard fires.
   - Are the oracle's value sets the ones the SCREENS produce? Compare against `composerLockEdit` and `composerHashEdit`: if a screen can produce a value the oracle never tries, the census is narrower than it claims.
4. **The call sites.** The census tests the function, not the wiring. Swap the two field constants at the two arms in a copy, one at a time, and confirm a named test fails each time.
5. **M-2 / M-3 are true statements now.** Read them against the code. M-3 re-asserts that a swap of equal-key-count paths leaves the signature identical (`w1/1,1,|0.0/1.0/`) — reproduce that, and confirm `composerMoveUp`'s unconditional discard is still load-bearing.
6. **Nothing else moved.** `git diff 818220d8..177b4906 --stat` should be `gui/composer_discard.go`, `gui/composer_shape.go`, `gui/composer_backleg_test.go` and nothing more. Any hunk beyond the four findings is Important.
7. **Gates as CI runs them:** `gofmt -l cmd/`, `go vet ./gui/ ./cmd/...` (two pre-existing `testing.ArtifactDir` lines expected), `go test ./...`, sharded gui (report the count; `818220d8` was 1199), `./scripts/test-32bit.sh`, `go build ./cmd/...`, firmware against `70008da`'s 1,581,204 / 62,800 (round 2 measured that baseline itself). Then `design/journeys/capture_composer.py --arm both` and the three shipped drivers: exit 0.

## Severity

A false PASS in the new test, a seat discarded without §8j, a confirm that fires where nothing is at stake, or a hunk outside the fold = **Critical or Important**. Wording = Minor/Nit. Do not re-open F-470 or F-471. A finding you cannot reproduce is not a finding — say what you tried. If it is clean, say MERGE plainly; a clean round closes this loop.

## Report (your final action)

Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-S4-W6-fold-r2-verification.md` (create; must not exist): I-2, I-3, M-2, M-3 each FIXED / PARTIAL / NOT FIXED with the output that shows it; the false-PASS hunt's result including the run against the restored round-2 probe; every mutation you ran; the shard count and firmware delta; closing counts and a plain MERGE / DO NOT MERGE. Return a two-line summary plus the path.
