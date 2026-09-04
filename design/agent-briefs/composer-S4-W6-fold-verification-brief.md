You are the INDEPENDENT verifier of a FOLD, round 2, on the SeedHammer II composer. Round 1 (`design/agent-reports/composer-S4-W6-verification.md`, opus) returned **1C/1I/1M** against `composer-s4e` at `05466727` and said DO NOT MERGE. This brief is about the controller's response to it.

ONE QUESTION: does the fold close C-1, I-1 and M-1 without introducing a new defect — and is there a THIRD door of the same kind still open?

Branch `composer-s4e`, worktree `/scratch/code/shibboleth/wt-composer-s4e`. The fold is `818220d8991e084ab6c8a4a3a6c44ebc7ff310a7`; the reviewed tip was `05466727c5589ddcedf6c38b05855da0cac17ac3`; the base is fork main `70008da5f935b36635a442cb2738f8dcc2fce7f1`. Read round 1's report first — it is on disk and it is the specification for this round.

Read-only: copy the worktree with `cp -r` to `/scratch/code/shibboleth/.s4e-verify2/` for every mutation (never dirty the worktree; do not run `git checkout` in a copied worktree — it shares the gitdir). Go: `/scratch/code/shibboleth/.toolchain/go/bin` on PATH, `CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`, `-mod=readonly`, `TMPDIR=/scratch/code/shibboleth/.tmp`. Sharded gui: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`. Firmware needs `/nix/var/nix/profiles/default/bin` on PATH first. Do NOT spawn sub-agents; commit nothing; read no `.jsonl` file.

## What the fold does, so you can attack it rather than rediscover it

Round 1's root cause: `composerShapeSignature` re-derived md's numbering rule (wrapper + path count + per-path `N`) while `lowerTr` picks the internal key with `isBareSingle()` and numbers it first, so a lock or hash moves `@0` under tr invisibly to the signature.

The fold stops re-deriving and asks the codec:

1. `composerShapeSignature` now appends `md.Composed.Slots()` — the mapping itself — after the structural terms, which are kept only as the fallback for a list `md.Compose` refuses (so an edit into or out of a refused shape reads as a move, which discards).
2. `composerEditCanRenumber(list, idx)` answers "can a lock/hash edit on this path move the mapping?" by composing the list with idx's lock+hash cleared and with a lock set, and comparing signatures. It deliberately does NOT restate `isBareSingle()`.
3. `composerPathEdit`'s lock arm (case 1) and hash arm (case 2) are now wrapped in `composerApplyShapeEdit`, and ask `composerShapeGuard` only when `composerEditCanRenumber` is true.
4. New tests: `TestComposerShapeSignatureSeesTheCodecsNumbering`, `TestComposerBackLegPresetAsksBeforeDiscardingSeats` (C-1, walked from a keyed payload), `TestComposerLockEditUnderTrDiscardsTheSeatsItMoves` (I-1), `TestComposerBackAtTheWrapperPickerLeavesTheComposer` (M-1).
5. Spec §7d in `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_wallet_policy_composer.md` replaces its enumeration with the codec's answer; §7b's Back sentence is refined.

Already run by the controller — reproduce if cheap, do not re-derive at length: all four mutations caught by their own named assertion (signature back to structural-only; never ask on the lock arm; ALWAYS ask on the lock arm, which the shipped wsh test `TestComposerLockAndHashEditsAreNotGuardedByTheDiscardConfirm` catches; lock arm outside `composerApplyShapeEdit`). Round 1's own C-1 reproduction was re-run on this harness and is red before the fold, green after.

## Verify

1. **C-1 closed.** Walk round 1's exact reproduction (`[2-of-2, 1 key, 1 key]` under tr, seat two, §8p → "Back to the paths" → Back → `Start from?` → `decaying-multisig`) and confirm §8j is drawn before the shape is replaced, that declining leaves shape AND seats untouched, and that accepting clears every seat. Then try the OTHER five presets and the blank row, and both other wrappers.
2. **I-1 closed, and not over-closed.** A tr lock edit that moves `@0` must ask and discard; a wsh lock edit, and a tr lock edit on a path whose bare-singleness cannot matter, must ask NOTHING and keep every seat. Construct both. A confirm that fires where nothing is at stake is a defect in its own right (§7g calls a lock edit DEFAULT) — round 1's M-C mutation is the guard against it, so check it still bites.
3. **THE THIRD DOOR — the highest-value item.** The class is "the GUI decides something the codec decides". Hunt for another instance: is there any remaining path where `st.assigned` survives a change to `st.list` that moves `md.Composed.Slots()`? Enumerate every production write to `st.list` and every caller of `composerApplyShapeEdit`, and test the ones the fold did not touch — `composerMoveUp`, `composerAddPath`, the keys arm, "Remove path", the wrapper row, and the Back leg itself. Report a counterexample or state plainly that you found none and how you looked.
4. **`composerEditCanRenumber`'s probe.** It sets `older(1)` as its trial lock and clears the hash. Attack that choice: a path with `Keys == nil` and a hash; a path already at the slot cap; a shape where the probe's own variants fail to compose while the real edit would not; anything where the probe answers false and the real edit still moves the mapping. This function is new, it is the guard's whole condition, and nothing outside these tests exercises it.
5. **The signature's fallback.** Two DIFFERENT refused shapes must not compare equal in a way that keeps seats. Find a pair if one exists.
6. **Gates, as CI runs them:** `gofmt -l cmd/`, `go vet ./gui/ ./cmd/...` (two pre-existing `testing.ArtifactDir` lines are expected and are on `70008da` already), `go test ./...`, the sharded gui runner (report the count; `05466727` was 1195), `./scripts/test-32bit.sh`, `go build ./cmd/...`, firmware size against `05466727`'s `1,581,428 B flash / 62,800 B RAM` and `70008da`'s `1,581,204 / 62,800` — **measure the `70008da` baseline yourself**, round 1 took it on trust and said so.
7. **The four capture drivers** (`design/journeys/capture_composer.py --arm both`, `capture_walletpolicy.py`, `capture_seating.py`, `capture_tr_pathological.py`): exit 0.
8. **The spec fold.** §7d's new text must describe what the code does — not what it should do. Check the measured claim in it (the hand-built/preset pair) against the code, and check §7b's sentence too.

## Severity

A carried seat across a moved mapping, a §8j that can be skipped, a confirm that fires where nothing is at stake, a test that cannot fail, a probe that answers wrongly, or a hunk outside this fold = **Critical or Important**. Wording = Minor/Nit. Do not re-open F-470. A finding you cannot reproduce is not a finding — say what you tried.

## Report (your final action)

Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-S4-W6-fold-verification.md` (create; must not exist): per item VERIFIED / NOT VERIFIED with the output that shows it; C-1, I-1 and M-1 each explicitly FIXED / PARTIAL / NOT FIXED; every mutation you ran; the shard count and both firmware numbers; closing counts. Return a two-line summary plus the path.
