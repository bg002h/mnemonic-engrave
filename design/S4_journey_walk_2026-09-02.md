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

## W-5 — typing a hashlock's 64 hex on the device is very hard (operator, 2026-09-03)

| step | in hand | device did | divergence | class |
| --- | --- | --- | --- | --- |
| Path -> Hash lock -> Which hash? with NO payload loaded | rows `Type 64 hex`, `No hash lock` | the fallback keyboard for 64 hex characters | the primary route (§6c, C25: a `hash:` payload record picked as one row) is invisible on a machine with no payload, and the screen does not say it exists | DEFAULT + DOCUMENTATION: the host path exists and works (measured below); the screen should name it -- filed F-465 with the host helper; on-device preimage entry is C25's deferred item, re-raised by the operator -- filed F-466 for a ruling |

Operator's questions: (1) is there a host method to type a preimage, hash it
doubly and pack it? (2) can the operator enter the preimage on the SH2
directly? Measured on the host 2026-09-03 (`me` 0.8.0): `X = sha256(passphrase)`
is the 32-byte preimage, `H = sha256(X)` the digest;
`printf 'hash:%s\n' "$H" | me sysw pack --no-passphrase --no-now --in - ...`
packs and `me sysw show` prints `sha256 hashlock (hash:) — b867db87..edbc96cb`;
the device's `Which hash?` then offers it as `hash 1  b867db87..edbc96cb`.
No dedicated command exists yet (two `sha256sum` calls and `xxd`); the
preimage X is what must be backed up (F-132). On-device entry: spec §14 row
"on-device preimage derivation, storage or engraving" (C25, adversarial C-5)
-- deferred, not refused; a ruling to revisit it is the operator's.

W-5 rulings (operator, 2026-09-03): `ms` owns the hashlock command (F-465);
on-device hashlock-phrase entry is REQUIRED, Rust first (F-466 becomes a
change request for the next cycle). Term: "hashlock phrase" (two words).

W-4 SHIPPED: fork main `70008da5` (merge of `composer-s4d` `bb50775`; CI green;
verification 0C/0I, `composer-S4-W4-verification.md`). Firmware 1,581,204 B
flash / 62,800 B RAM. The device runs the branch build `bgbb50775`, the same
gui bytes; the operator's re-check of the blocks and date pads is the record
still owed.

## W-6 -- "Start from?" cannot be returned to; re-picking the script skips it (operator, 2026-09-03, on bgbb50775)

Operator's observation, verbatim: "In the wallet policy program on sh2, after
build new policy, which script (tr or wsh), any selection on the 'start from'
screen and not be returned to via the back button. Going back jumps to the
which scripts screen and picking a script (tr or wsh) skips over the 'start
from' screen."

| step | in hand | device did | divergence | class |
| --- | --- | --- | --- | --- |
| Build a new policy -> script (tr/wsh) -> `Start from?` -> any row -> ... -> Back | a chosen start (blank or a preset) | Back lands on the script choice, not on `Start from?` | W-1's fold (`composer-s4` bc9dd63) made Back FROM `Start from?` return to the script choice deliberately; Back from the NEXT screen apparently skips `Start from?` too | to classify (DEFAULT vs defect): a screen the operator cannot return to is the [[can-a-user-do-the-thing]] class if the only way to change the start is to discard the policy |
| ... -> script choice -> pick tr or wsh again | the same policy state | `Start from?` is NOT shown; the flow continues past it | the start choice persists in the composer state and the picker is not re-offered on re-entry | to classify: if the operator wanted a different preset they now cannot reach it without Back-to-discard |

**CLASS: W-6 CHANGE (Important: a shipped screen becomes unreachable, and Back
skips a step it should return to).**

MEASURED 2026-09-04 on fork main `70008da` by a flow-level walk that drives the
real `composerFlow` (`gui/composer_backleg_test.go`), both halves reproduced:

| step | frame drawn |
| --- | --- |
| forward: script -> `Start from?` | `"Startfrom?Buildmyownpathsplain-multisig...decaying-multisig"` |
| forward: pick a preset -> paths | `"Spendpathsslots:3Path1:2-of-3AddaspendpathChangethescriptDone"` |
| **Back at the path list** | `"Whichscript?Taproot(tr)Segwit(wsh)Nested(sh-wsh)Legacy(sh)"` |
| **pick a script again** | `"Spendpathsslots:3Path1:2-of-3Addaspendpath..."` -- `Start from?` never drew |

Cause, read from the code and confirmed by the walk: the site is NOT
`gui/composer_shape.go` (the earlier controller note's guess) but
`gui/composer_flow.go`'s Back leg out of `composerShapeFlow`, which ran
`composerWrapperPick` ALONE and then `continue`d straight into the path list.
The preset picker sits in the ENTRY loop only, so it was passed exactly once
per composition, in one direction. The shipped test
`TestComposerBackAtThePathListKeepsTheComposition` had encoded that as
intended, in a comment: "The re-pick after the Back below does NOT pass
through it, which is why this step appears once."

Why it earns a change rather than a documentation line: the six archetypes S0b
shipped (a whole Rust-first cycle, F-453) are reachable only on that one
screen, and the operator's only route back to it was to discard the whole
composition -- the [[can-a-user-do-the-thing]] class, and the exact inverse of
W-1, where Back meant forward. Spec §7b states Back's rule for the preset
screen and is SILENT about Back at the path list, which is why no gate caught
it: nothing was wrong in a section, a step was merely unreachable.

FIXED on fork branch `composer-s4e` (`05466727`, on `70008da`): `composerStartStep` walks §7b's opening
pair and IS the Back leg, entered at the preset screen. Back is now the
inverse of the way in -- paths -> `Start from?` -> script -- and a script
picked on the way out walks forward through `Start from?` again. The blank row
("Build my own paths") KEEPS the current paths on re-entry rather than
blanking them: it is the default row of a screen the operator reaches by
pressing Back, and blanking there would have been a new data-loss trap of my
own making (mutation-tested: making it blank the list fails the test that says
so).

## W-7 -- the Back leg changed the wrapper with seats held, unguarded (found 2026-09-04 while measuring W-6)

| step | in hand | device did | divergence | class |
| --- | --- | --- | --- | --- |
| ... seat @0 and @1, leave @2 -> §8p -> "Back to the paths" -> **Back** -> pick a different script | a policy with two slots seated | the wrapper changed with NO §8j confirm, and both seats were CARRIED into the new numbering | the path list's own "Change the script" row asks §8j and discards; the Back leg one function away did neither | **W-7 CHANGE (Critical: an unmet guarantee, §7d/§8j; keys seated into slots they were never chosen for)** |

MEASURED on fork main `70008da`, walked from a keyed payload:

- Reachability: §8p's "What now?" offers "Back to the paths", which lands on
  the path list with every seat still held. Back there, then a script, and the
  edit is accepted with no confirm -- the frames after the change are the path
  list, never `"EDITINGTHESHAPECLEARSTHEKEYS"`.
- The seats survive it. The stub screen after the change reads
  `"Slot@0:73c5da0am/48h/0h/0h/2h Slot@1:73c5da0am/48h/0h/1h/2h Slot@2expectsakeyat..."`.
- And the numbering PERMUTES. `md.Composed.Slots()` for the shape
  [Path 1: 2-of-2, Path 2: a single key], measured:
  `wsh -> [{@0 path0 ord0} {@1 path0 ord1} {@2 path1 ord0}]`,
  `tr  -> [{@0 path1 ord0} {@1 path0 ord0} {@2 path0 ord1}]`.
  Same slot COUNT (3), so `composerSizeAssignments` left `st.assigned`
  untouched -- and the key the operator seated as "Path 1 key 1 of 2" became
  slot @0, which under tr is Path 2's sole spending key.

Nothing on any screen says so: `composerMappingLines` prints a slot's index and
origin, never its path, so the mapping review after the change shows the same
two lines it showed before. This is the failure `gui/key_card_seating.go:24-27`
refuses to allow anywhere on this device -- "a misassignment does not fail, it
derives a different wallet's address and shows it to the operator as proof."

Spec §7d states the rule the leg broke, verbatim: "Any change that moves slot
NUMBERING (the wrapper, the path count, or a path's key count) after at least
one slot has been assigned discards ALL assignments; the operator is told so
before the edit is accepted (§8j)." The rule was met by the path list's
"Change the script" row (`gui/composer_gates_test.go` walks it) and unmet by
the Back leg, which assigned `st.list.Wrapper` directly -- bypassing both
`composerShapeGuard` and `composerApplyShapeEdit`.

FIXED with W-6, in the same commit (`05466727`) and the same `composerStartStep`: the choice is applied through
`composerApplyShapeEdit`, and §8j is asked first whenever the shape signature
would move. The confirm is asked AFTER the choice and BEFORE it is accepted --
§7d's own wording -- rather than on entry as the path-list row asks it,
because an operator on this leg is usually navigating, and re-picking the same
script with the blank row moves no slot at all.

Both fixes carry failing-first tests in `gui/composer_backleg_test.go`, and
four mutations of the fix were each caught by their own named assertion:
dropping §8j, blanking the list on the blank row, running the wrapper picker
alone (the shipped defect), and assigning the list without
`composerApplyShapeEdit`.

### The verification found the first fix incomplete, and a third door open

`design/agent-reports/composer-S4-W6-verification.md` (opus, 1C/1I/1M) closed
W-6 and closed W-7 **for the wrapper only**, and reported that the fix had
opened a second door to W-7's own failure class. Both findings are one defect,
and it is the same shape as W-7 itself: **the GUI was re-deriving the codec's
numbering rule instead of asking it.**

`composerShapeSignature` carried the wrapper, the path count and each path's
key count — §7d's own enumeration — while md's `lowerTr` numbers slots from an
internal key chosen by `isBareSingle()` (one key, no lock, no hash) and places
it ahead of listed order. So a LOCK or a HASH decides which path owns slot `@0`
under tr. Re-derived by the controller before folding:

| shape | §7d signature | slots |
| --- | --- | --- |
| hand-built `[2-of-2, 1 key, 1 key]` | `w0/2,1,1,` | `[{@0 p1 o0} {@1 p0 o0} {@2 p0 o1} {@3 p2 o0}]` |
| `decaying-multisig` preset | `w0/2,1,1,` | `[{@0 p0 o0} {@1 p0 o1} {@2 p1 o0} {@3 p2 o0}]` |

Identical signatures, three of four slots moved. So:

- **C-1 (Critical, introduced by the W-6 fix):** the preset rows that fix newly
  makes reachable from the path list replaced a seated shape with **no §8j**
  and carried every seat. Reproduced end to end on the operator's own route,
  with no lock or hash screen involved — `Add a spend path` and the key-count
  pickers alone. Pre-fix this was unreachable, because `composerPresetPick` was
  entry-only.
- **I-1 (Important, pre-existing on `70008da`):** the path editor's lock arm
  runs `composerLockEdit` with neither guard nor `composerApplyShapeEdit`,
  deliberately, on §7d's "a lock or hash edit moves no slot" — a premise that
  is true under wsh and false under tr.

FOLDED at `818220d8`: `composerShapeSignature` now carries
`md.Composed.Slots()`, the mapping itself, keeping the structural terms only as
the fallback for a list the codec refuses (so an edit into or out of a refused
shape reads as a move, which discards — the safe direction). The lock and hash
arms are wrapped in `composerApplyShapeEdit`, and §8j is asked on them exactly
when `composerEditCanRenumber` — which asks the CODEC, with the lock cleared
and set, rather than naming `lowerTr`'s predicate a second time — says the edit
can move the mapping. Spec §7d corrected: the enumeration is replaced by the
codec's answer.

Four mutations of this second fold, each caught by its own named assertion: the
signature back to structural-only (C-1 and I-1 both return), never asking on
the lock arm (I-1 returns), **always** asking on the lock arm (the shipped wsh
test `TestComposerLockAndHashEditsAreNotGuardedByTheDiscardConfirm` fails,
which is what keeps §8j from firing where nothing is at stake), and the lock
arm applying outside `composerApplyShapeEdit`. M-1 closed too: the leg's sole
exit is now pinned by a test.

**The lesson, recorded:** W-7 and C-1 are the same mistake one level apart. A
GUI that restates a codec's rule will drift from it, and the drift shows up as
keys seated onto the wrong paths. Ask the codec.

### Round 2: the fold's own new probe was wrong in both directions

`design/agent-reports/composer-S4-W6-fold-verification.md` (opus, 0C/2I/2M/1N)
found C-1, I-1 and M-1 all FIXED, and the root cause closed structurally — 0
equal-signature renumbering pairs over 4,828 composable lists, 0 over 28,948
preset × hand-built pairs. Both Importants were in the one function the fold
added, and the class had moved *inside my own fix*: `composerEditCanRenumber`
cleared the HASH in both of its variants while varying only the lock, so it
answered a question about a path it had already changed.

- **I-2:** on a key-less path both variants collapse to the same refused
  empty-path shape, so the probe said "no move", the hash arm asked nothing —
  and the `composerApplyShapeEdit` wrapper the fold had just added then
  discarded **every seat with no §8j and no chance to decline**. A regression
  against `05466727`, where the same walk kept both seats.
- **I-3:** on a tr path carrying a hash no lock can affect `isBareSingle`, yet
  §8j drew "Every key you seated will be cleared", cleared nothing, and
  declining it left the lock uneditable — verbatim the failure the function's
  own comment says it exists to remove.

Measured over 14,092 `(list, idx)` pairs: **1,200 false negatives, 288 false
positives**.

FOLDED at `177b4906`: the probe varies ONLY the field its arm edits
(`composerFieldLock` / `composerFieldHash`), the two arms pass their own field,
and the reviewer's census is committed as
`TestComposerEditCanRenumberIsExactOverEveryReachableShape` — an enumeration
over 3,708 `(list, path, field)` cases whose oracle is independent of the probe
(it sweeps the values each SCREEN can produce, rather than comparing two
points). It reports 0 and 0 on the fix and **156 false negatives / 288 false
positives** on the probe it replaced; the 288 match the reviewer's count
exactly. The call-site wiring, which the census cannot see, is pinned by
`TestComposerHashEditOnAKeylessPathAsksBeforeItDiscards`, and both field-swap
mutations are caught. M-2 and M-3 (two comments the fold had falsified) are
corrected, `composerMoveUp`'s unconditional discard re-measured as still
load-bearing (`w1/1,1,|0.0/1.0/` before and after a swap), and N-1 filed as
F-471 rather than fixed inside a merge already two rounds deep in funds-relevant
code.

**The lesson sharpens:** the first fold moved the class from the GUI's
signature into the GUI's *probe of* the codec. Asking the codec is not enough —
you have to ask it the question the operator's screen actually poses. A probe
that varies a field the arm does not edit is the same defect wearing the
remedy's clothes.
