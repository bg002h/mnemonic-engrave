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
