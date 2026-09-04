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

Third instance, the plate's own screen (`shots/k02-stub-p0.png`, the keyless
arm): `Template-ID: e0863d3ccac31a64d3b5e14b85ccd6` -- TWO hex digits (`c0`)
under the Back button, and `--policy-id-stub e0863d3` under the page button.
This is the screen the operator photographs before cutting the S4 plate
(plan Task 4 step 1), so the fix precedes the plate. The seating pick list
(`c07-seat-slot0.png`) and the consent are unaffected.

## The emulator journey is EXECUTED (2026-09-03); the device walk resumes at step 3

Spec §12 items 2, 3 and 9 ran on the emulator (fork main `6fb90cb`, engrave
master `e3ee51c9`; `design/journeys/SeedHammer-II-composer-journey.pdf`). The
walk with the operator resumes at "Add a spend path" on the Taproot 2-of-3
shape once fork main (`1ae0ffcb` or later: W-2 + W-3) is flashed at the
operator's word; then ONE plate whose string must read
`md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3` byte for byte.

## Device confirmation 2026-09-03 — bg6fb90cb boots; W-1, W-2, W-3 confirmed on hardware by the operator

| fix | the operator saw |
| --- | --- |
| W-1 | "Build my own paths" is item 1 on the screen titled "New policy" whose lead "Start from?" sits at the bottom |
| W-2 | the threshold screen offers 1, 2, 3 after tapping 3 on "how many keys?" |
| W-3 | the Template screen shows all 32 hex digits of the Template-ID |

(Operator note recorded as observed, not as a divergence: a ChoiceScreen's
lead is drawn in the band at the BOTTOM of the panel, under the rows.)
The walk resumes at step 3.

## Part A walked to the cut on bg6fb90cb — 2026-09-03, WITH the operator (Taproot 2-of-3, key-less)

| step | in hand | device did | divergence | class |
| --- | --- | --- | --- | --- |
| boot | the payload offer | SKIP taken | none | -- |
| carousel | Wallet Policy, 8th entry | entered | none | -- |
| door | title Wallet Policy; rows Scan cards, Build a new policy | Build taken | the operator reported the rows and not the lead line (see below) | documentation |
| script | Which script? | Taproot (tr) | none | -- |
| preset | Start from? | Build my own paths already selected; confirmed | none (W-1 holds) | -- |
| shape | Add a spend path -> Keys -> 3 -> 2 -> Done -> Sorted (usual) | as the plan's keyless arm | none (W-2 holds: 3 and 2 taken by tap) | -- |
| Template | `Template-ID: e0863d3ccac31a64d3b5e14b85ccd6c0`, `mk1 stub (template): e0863d3c` | equals the host's `md inspect` (plan §2) | none (W-3 holds: all 32 digits read) | -- |
| seat / review / §8l / modal | Engrave a key-less template; Review paged; hold; "No slot is seated..." | as the plan | none | -- |
| census | `1 plate (key-less wallet policy)` | as the plan | none | -- |
| variant | Choose engraving | TEXT + QR confirmed | none | -- |
| cut | hold to start | NOT CUT: no blank plate available | -- | deferred |

The device, the emulator and the host now agree on the Template screen for
this shape (`e0863d3c...`); the plate itself (`md1fkzyyqq...h5wvl3`, 56
chars) waits for a blank. Nothing on the device holds state for it: the walk
can be repeated from the door in under a minute when a blank is on hand.
Open question for the record: what the door's lead line read on a machine
whose flash still carries the Load Payload journey's region (expected "A
payload is in flash but not loaded. Load it from the carousel first.").

Door lead, confirmed by the operator 2026-09-03 on bg6fb90cb: "A payload is
in flash but not loaded. Load it from the carousel first." -- read, and
called "that very helpful reminder". The documentation row above closes;
§7a's lead does its job on a machine whose region is present but skipped.

## W-4 — the digit pad overprints its prompt and its range line (found 2026-09-03 by the operator on bg6fb90cb)

| step | in hand | device did | divergence | class |
| --- | --- | --- | --- | --- |
| Build a new policy -> tr -> decaying-multisig -> Path 1 -> Time lock -> After a wait -> Blocks | the digit entry for the wait | between the entry box and the 0-9 keyboard, two lines of text overlap with no newline between them: "How many blocks?" and "1 to 65535 (blocks)" drawn over each other | the prompt and the live range line share one row | **W-4 CHANGE (Important: the range hint is unreadable on the screen that exists to state it)** |

Operator's words: "that screen has two overlapping lines of text between the
text entry box and the keyboard. It looks like it reads 'How many blocks?'
and '1 to 65535' but there is no newline in between. The keyboard is wisely
a 0-9 type keyboard." Reached through a PRESET (decaying-multisig), then the
path's Time lock. Reproduction and cause below.

Cause, read from `gui/composer_digitpad.go` `composerDigitEntry`: the info
lines (`lead`, then the echo `line`) are laid out under the entry box at
`lineY`, and EACH is clamped on its own -- `if lim := top.Max.Y - sz.Y; y >
lim { y = lim }` -- to the band above the keyboard. When the second line does
not fit it is moved up to the band's bottom, which is where the first line
already is; the two are drawn over each other. Every pad shares the code, so
"How many blocks?", "How many days?", "Date as YYYYMMDD" and "Block height"
all overprint their range line, as the operator saw on the blocks and the
date-or-height routes. The S4 capture typed 12960 through this pad and read
its text through `shScreen()`, which cannot see an overprint (the W-3
lesson, again). Fix on fork branch `composer-s4d` (brief
`composer-S4-W4-fix-brief.md`): the box and its lines are one vertically
centred group inside the band, never clamped line by line; a rasterising
geometry test over all four pads, empty and filled, asserts no two text
rectangles intersect and fails on `6fb90cb`.

Filled state, operator 2026-09-03: typing does produce the echo, partially
visible, but the overlap with the prompt remains -- so both the empty and the
filled pad overprint, which is what the fix brief's test asserts for every
pad in both states.
