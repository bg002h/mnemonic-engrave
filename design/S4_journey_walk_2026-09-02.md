# S4 journey walk WITH the operator — 2026-09-02, on the device (bgb77449d)

Method: the operator role-plays; at each step three questions (what is in
hand exactly, what the device does, what ELSE they might do). Each divergence
is classified refusal / warning / default / not-our-concern / documentation
only, and earns a change only when the wrong outcome is worse than saying
nothing. Fixes batch into one S4 fold on the fork after the walk.

## Part A: no payload -> keyless template -> engrave -> decode back

| step | in hand | device did | divergence | class |
| --- | --- | --- | --- | --- |
| 1 | main menu, no payload | Wallet Policy door: "Scan card", "Build a new policy"; key state stated | none ("everything looks fine and is clear") | -- |
| 2 | door | wrapper choice, then "New policy / Start from?" with the six presets by name | Back on the preset list moves FORWARD into the blank "Spend paths" editor (Add a spend path / Change the script / Done); there is no row for a blank list | **W-1 default -> CHANGE** |

### W-1 — the blank route is undiscoverable

`gui/composer_presets.go` `composerPresetPick`: the ChoiceScreen offers only
the six preset names; declining (Back) IS the blank route ("declining here is
the BLANK route, not an error"). Spec §7b says "preset or blank". A user who
wants their own shape sees six presets and no way forward; the one key that
works is the one they expect to go backwards, and Back on the next screen
does not return to the wrapper choice. Wrong outcome: the operator never finds
the blank route, or reaches it by accident and cannot tell how. Worse than
telling them nothing.

Fix: fork branch `composer-s4` bc9dd63 -- the blank route is a row, FIRST ("Build my
own paths", so the default selection commits to nothing), returning the empty
list; Back returns to the wrapper choice. Regression test
`TestComposerPresetPickerOffersBlankFirstAndBackReturnsToTheWrapper` drives the
real screens and fails under both named mutations (pasted in the commit). The
label passes `assertChoiceLabelFits`. Spec §7b carries the row.

Walk paused at step 3 (the operator was tired); resumes at "Add a spend path".

## Emulator regression of the S3 door fix — 2026-09-03, fork main 60bee002 (controller)

The S3 plan's Task C2 Step 5 changed the three shipped Wallet Policy walk
drivers (`cmd/emu/shots_walletpolicy.js`, `shots_seating.js`,
`shots_tr_pathological.js`: a second `await tap(CONFIRM)` for the composer's
door) and could only count the edit, not run it -- "the walks need a browser
and playwright, which no gate in this stage has". They have now run, against a
fresh `emu.wasm` (10,788,612 bytes, Go 1.26.7) of fork main `60bee002`, from
`/scratch/code/shibboleth/.tmp/s4-emu-regression.sh` (log
`s4-emu-regression.log` beside it):

| driver | exit | shots | wall |
| --- | --- | --- | --- |
| `capture_walletpolicy.py` | 0 | 8 | 15 s |
| `capture_seating.py` | 0 | 8 | 9 s |
| `capture_tr_pathological.py` | 0 | 9 | 12 s |

Each capture is a comparison (the host's ids and addresses must appear on the
device's consent screen), so all three passing means the door did not move
what those journeys prove. The shots and `out/` intermediates are untracked
(`git ls-files design/journeys/shots` is empty), so the run left the tree
clean.
