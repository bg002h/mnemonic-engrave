You are the INDEPENDENT fold-verification reviewer (targeted) for two findings from the S4 journey walk on the SeedHammer II, recorded in `/scratch/code/shibboleth/mnemonic-engrave/design/S4_journey_walk_2026-09-02.md`:

- **W-6** (the operator, on the device): after `Build a new policy -> script -> Start from?`, the `Start from?` screen could never be returned to -- Back landed on the script choice, and re-picking a script skipped `Start from?`.
- **W-7** (the controller, found while measuring W-6, **Critical**): that same Back leg changed the wrapper by assigning `st.list.Wrapper` directly, bypassing `composerShapeGuard` (§8j's confirm) and `composerApplyShapeEdit` (the discard), so seats were CARRIED across a wrapper change that permutes slot numbering.

The fix is fork branch `composer-s4e`, tip `05466727c5589ddcedf6c38b05855da0cac17ac3` on `70008da` (worktree `/scratch/code/shibboleth/wt-composer-s4e`). There is no implementer report: the controller measured, fixed and gated this one, so **you are the only independent read of it** -- re-derive everything, trust no sentence below that you can run instead.

ONE QUESTION: does the fix close both findings, can each of its tests actually fail, and did it break any other Back path or the discard rule anywhere else?

Read-only: copy the worktree with `cp -r` to `/scratch/code/shibboleth/.s4e-verify/` for every mutation (never dirty the worktree; do not run `git checkout` in a copied worktree -- it shares the gitdir). Go: `/scratch/code/shibboleth/.toolchain/go/bin` on PATH, `CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`, `-mod=readonly`, `TMPDIR=/scratch/code/shibboleth/.tmp`. Sharded gui runs: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`. Do NOT spawn sub-agents; commit nothing; read no `.jsonl` file.

## Already settled -- do NOT re-derive, and do not re-audit the composer at large

Machine-checked by the controller; spend your budget past these:

1. The permutation is real. `md.Composed.Slots()` for `[Path 1: 2-of-2, Path 2: a single key]`: `wsh -> [{@0 p0 o0} {@1 p0 o1} {@2 p1 o0}]`, `tr -> [{@0 p1 o0} {@1 p0 o0} {@2 p0 o1}]` -- equal COUNT, permuted mapping. Pinned by `TestComposerWrapperChangePermutesSlotsAtEqualCount`.
2. Both new tests were RED on `70008da` before the fix, each naming its own finding.
3. Four mutations of the fix were each caught by their own named assertion: (a) dropping the §8j condition, (b) making the blank row blank the list, (c) running the wrapper picker alone on the Back leg, (d) assigning `st.list = next` without `composerApplyShapeEdit`.
4. Every other production assignment to `st.list` goes through `composerApplyShapeEdit`, except `composerMoveUp`, which discards unconditionally on purpose (review r0 I-1). Grepped, not assumed.
5. `go vet ./gui/` prints two `testing.ArtifactDir requires go1.26` lines on `70008da` ALREADY -- pre-existing, outside the diff.
6. Spec §7b was folded to state the Back rule, and F-470 filed for a question deliberately left open (a preset row replacing hand-built paths with nothing seated is not confirmed).

## Verify

1. `git diff 70008da..05466727c5589ddcedf6c38b05855da0cac17ac3 --stat` and every hunk. Expected: `gui/composer_flow.go` (the entry and the Back leg both call the new `composerStartStep`), `gui/composer_presets.go` (`composerPresetPick` returns `(list, replace, ok)`), `gui/composer_flow_test.go` (the shipped Back test rewalked), `gui/composer_backleg_test.go` (new). **A hunk touching seating, the codec, the stub screen, the census or the engrave path is Important** -- say so and name it.
2. **W-6 closed.** On the fix: Back at the path list draws `Start from?`; Back there draws `Which script?`; picking a script draws `Start from?` again; the blank row keeps the paths. Then the inverse claim: in a copy, restore the old leg (call `composerWrapperPick` alone) and confirm the test fails naming W-6. **Prove the mutated line RAN.**
3. **W-7 closed, and this is the one that matters.** Construct the failure rather than reading for it: drive a keyed payload to the path list with seats held (§8p's "What now?" -> "Back to the paths" is the shortest route), take the Back leg, change the wrapper, and show that (a) §8j is drawn BEFORE the change is accepted, (b) declining it leaves the composition and the seats exactly as they were, (c) accepting it clears every seat, and (d) the stub screen afterwards shows no seated slot. Then hunt a counterexample: **is there ANY route through `composerStartStep` that ends with a seat held across a moved shape signature?** Try the preset rows (a preset replaces the whole list), a wrapper change plus a preset in one pass, `ctx.Done` mid-leg, and Back at the wrapper picker with seats held.
4. **What the fix might have broken.** `composerStartStep` returning false is now the only way the composer exits from that leg: confirm Back at the wrapper picker still leaves the flow (and that leaving is what the shipped tests expect), confirm the flow cannot loop forever when `ctx.Done` goes true mid-leg, and confirm the decline path (`return true`) cannot strand the operator on a screen with no way out. A live-lock or an unreachable exit is Important.
5. **The confirm's placement.** The path list's "Change the script" row asks §8j on ENTRY; `composerStartStep` asks it after the choice and before accepting it. Judge whether that is defensible against §7d's "told so before the edit is accepted" -- and whether it can now fire when NOTHING is at stake (re-picking the same script with the blank row must draw no confirm; check it).
6. **Gates, as CI runs them**, on the worktree: `gofmt -l cmd/`, `go vet ./gui/ ./cmd/...`, `go test ./...`, the sharded gui runner (the count on `70008da` was 1192; report yours and account for the difference), `./scripts/test-32bit.sh`, `go build ./cmd/...`, and the firmware size against `1,581,204 B flash / 62,800 B RAM` at `70008da` -- report the delta and say whether a GUI-only change should have moved it.
7. **The emulator, with taps.** Build `emu.wasm` from the copy and drive the operator's own route by geometry (`window.shTargets()`, as the shipped drivers do, not synthetic key events -- W-2's lesson): Wallet Policy -> Build a new policy -> a script -> `Start from?` -> a preset -> Back -> confirm `Start from?` is on screen -> Back -> a script -> confirm `Start from?` returns. Screenshot each step and LOOK at the PNGs (W-3's lesson: `shScreen()` cannot see a clipped or overprinted line). Then `design/journeys/capture_composer.py --arm both` against the copy: exit 0, and the three shipped drivers (`capture_walletpolicy.py`, `capture_seating.py`, `capture_tr_pathological.py`) exit 0.

## Severity

A carried seat across a moved signature, a §8j that can be skipped, a Back that loses the composition, a test that cannot fail, or a hunk outside W-6/W-7 = **Critical or Important**. Copy, wording and test-coverage gaps = Minor/Nit. A finding you cannot reproduce is not a finding -- say what you tried. Do not pad, and do not re-open F-470 (filed, the operator's call).

## Report (your final action)

Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-S4-W6-verification.md` (create; must not exist): per item VERIFIED / NOT VERIFIED with the command output that shows it; every mutation you ran and what it proved; the shard count and the firmware delta; closing counts (C/I/M/N). Return a two-line summary plus the path.
