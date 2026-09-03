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

## W-2 — the composer's pick lists cannot be operated by touch (found 2026-09-03 by the S4 emulator driver, before the device walk reached it)

| step | in hand | device did | divergence | class |
| --- | --- | --- | --- | --- |
| 3 (emulator, `Path 1: how many keys?`) | the count picker `1 2 3 4 5` | a tap on any row changes nothing; only Button2 (page) moves the cursor, to the first row of the next page | on the SH2 only the first row of each page of a `composerPickScreen` can be taken: `n = 2`, `n = 3`, `Done` once a path exists, every hash row and every seating row but the first are unreachable | **W-2 CHANGE (Critical: a state the operator cannot complete)** |

`gui/composer_paged.go` `composerPickScreen` moves its cursor only on
`ButtonFilter(Up)` / `ButtonFilter(Down)` and registers no `op.Input` hit
area per row; the SeedHammer II has no directional buttons (its only
production input is the ft6x36 panel's `PointerEvent`s;
`cmd/controller/debug_sh2.go` is the sole non-test source of `Down`). Four
production call sites: `composerCountPick` (keys, threshold), the `Spend
paths` list, `Which hash?`, `Seat keys`. Measured by the S4 implementer on the
emulator at fork `05d903b` (`composer-S4-implementation-report.md` Task 3):
a 205-tap sweep over the `n` picker never moved the cursor and the take gave
`n = 1`; the positive control (a row-2 tap on `Which script?`, a
`ChoiceScreen`, which starts the legacy wrapper's picker at 2) landed; paging
reaches exactly the first row of each page; with one path on the list, four
pages moved nothing and the take opened Path 1's editor, so `Done` is
unreachable. Every composer test drives these screens with synthetic `Down`
events -- the harness has an input the machine does not, the "a control can
test the wrong layer" shape -- which is why 1186 green tests, three R0 lenses
and the whole-diff review never saw it, and why the device walk paused one
step short of it.

Why the S4 walk record and not a plain bug: it is precisely the class the
journey exists for (the operator at step 3 would have tapped `3` and watched
nothing happen), and the emulator driver hit it first only because the
operator was tired at step 2.

Fix: fork branch `composer-s4b` (brief `composer-S4-W2-fix-brief.md`): every
drawn row of `composerPickScreen` gets a `Clickable` hit area, as
`ChoiceScreen`'s rows have, so a tap selects the row and Button3 takes it;
Up/Down stay. Regression test on the TOUCH harness (`runUITouch` + `tap`,
`gui/start_screen_touch_test.go`) through the real flow.

## W-3 — the Template screen's longest line runs under the navigation buttons (found 2026-09-03 in the S4 capture's screenshots, not by the driver)

| step | in hand | device did | divergence | class |
| --- | --- | --- | --- | --- |
| Template (stub) screen, keyed and keyless arms | `Template-ID: 531ab9e1777f018ae53694387dd0d128` | draws the line centred across the full panel width, so its tail (`...dd0d12` **8**) lies under the Back button; the `mk encode ... --origin-fingerprint <f` line and `--policy-id-stub 531ab9e` (the stub's last hex) lie under the page button on the keyless-arm frame | the ONE screen whose lines exist to be copied (§7c) hides the last character(s) of the id and of the stub argument | **W-3 CHANGE (Important)** |

Measured on the emulator's own framebuffer (`shots/c06-stub-p0.png`,
`c10-stub2-p0.png`, `k02-stub-p0.png` from `capture_composer.py --arm both`
at fork `a6eb44e`): `composerPageLines` (`gui/composer_paged.go`) centres each
`widget.Labelw` line on the panel and wraps at a width that overlaps the
navigation column (Back at the top right, page/confirm below), so a line long
enough reaches under a button. The shipped `confirmReviewScreen` (the consent,
`c11-consent-p0.png`) lays its `Policy-ID:` line below the buttons and shows
all 32 hex; the composer's own paged widget does not. **The driver could not
see it**: `shScreen()` extracts the drawn text, including text under a button,
so every needle and byte comparison passed -- only the pixels show the loss.

Wrong outcome: an operator comparing the Template-ID against the host by eye
compares 31 digits and passes a mismatch in the 32nd; an operator copying the
`--policy-id-stub` argument from the screen copies seven hex and `mk encode`
refuses (recoverable, but the screen taught them the wrong value). Worse
than saying nothing.

Fix: fork branch `composer-s4c` (brief `composer-S4-W3-fix-brief.md`): the
composer's paged widgets wrap and centre their lines inside the band LEFT of
the navigation column (the same right bound the W-2 hit areas use), so no
glyph is drawn under a button; a regression test asserts, from the frame's
own layout, that every text op of the stub screen lies outside the nav
rectangles -- a GEOMETRY test, since a text-presence test cannot fail on this.

Second instance, same widget (`shots/c09-mapping-p0.png`): the mapping review's
wrapped line "This device cannot confirm a key was derived at the origin it
declares." puts "origin" under the page button, so the sentence reads "...
derived at the [ ] it declares." The fix brief's item 3 drives every
`composerPageLines` screen, not only the Template screen.
