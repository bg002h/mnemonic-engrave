# Independent verification — composer S4 walk W-6 / W-7 fix (`composer-s4e` @ `05466727`)

Reviewer: independent verification agent (opus). Base `70008da`, tip
`05466727c5589ddcedf6c38b05855da0cac17ac3`, worktree
`/scratch/code/shibboleth/wt-composer-s4e`. All mutations run in a `cp -r` copy at
`/scratch/code/shibboleth/.s4e-verify`; the worktree was never dirtied and nothing was
committed.

**VERDICT: DO NOT MERGE AS IS. 1 Critical / 1 Important / 1 Minor / 0 Nit.**

W-6 is closed, cleanly and on real hardware-shaped screens. The W-7 fix closes the
*wrapper* door and leaves a second one open that the same commit creates: the preset
rows it newly makes reachable from the path list carry every seat across a renumbered
slot mapping, with no §8j confirm. That is W-7's exact failure class, reproduced end to
end on the operator's own route.

---

## C-1 (CRITICAL) — the Back leg's PRESET rows carry every seat across a renumbered mapping, with no §8j

**This is introduced by this commit.** At `70008da`, `composerPresetPick` was called from
one place only — the ENTRY loop, before the main flow loop — so it could never run with a
seat held:

```
$ git show 70008da:gui/composer_flow.go | grep -n "composerPresetPick\|composerWrapperPick\|for !ctx.Done"
52:		w, ok = composerWrapperPick(ctx, th)      <- entry loop
57:		list, ok := composerPresetPick(ctx, th, w) <- entry loop, st.assigned empty
68:	for !ctx.Done {                                 <- main loop starts here
74:			w, ok := composerWrapperPick(ctx, th)  <- the old Back leg: NO preset pick
```

`composerStartStep` now enters at the preset screen on the Back leg (`fromPaths=true`),
which is W-6's fix and is right. But the guard it added is conditioned on
`composerShapeSignature`, and **that signature is incomplete under `tr`.**

### The mechanism

`composerShapeSignature` (gui/composer_discard.go:27) captures the wrapper, the path
count and each path's key count `N` — and nothing else:

```go
fmt.Fprintf(&b, "w%d/", list.Wrapper)
for _, p := range list.Paths { ... fmt.Fprintf(&b, "%d,", n) }   // n = p.Keys.N
```

But `lowerTr` (md/compose.go:615) numbers slots from the *internal key*, which it picks
by a predicate the signature cannot see:

```go
// lowerTr extracts the FIRST-LISTED unlocked, unhashed single key as the
// internal key (else NUMS)
for i, p := range list.Paths { if p.isBareSingle() { ik = i; break } }
numbered, slots := numberSlots(list, ik)   // `first` goes before listed order
```

`isBareSingle()` is `Keys != nil && Keys.N == 1 && Hash == nil && Lock == nil`. So under
`tr`, **which path holds the bare single moves slot @0**, and two lists with an identical
signature can number their slots completely differently. `composerApplyShapeEdit` then
compares signatures, sees no move, and keeps every seat.

### Reproduced END TO END, on the operator's own device route

No lock screen, no hash screen, no synthetic input — only `Add a spend path` and the key
count pickers. Walk (`walletPolicyFlow`, keyed payload, real screens):

```
Build a new policy -> Taproot (tr) -> Build my own paths
  Path 1: Keys, 2 keys, 2 must sign        -> "Path 1: 2-of-2"
  Path 2: Keys, 1, 1                       -> "Path 2: 1 key"
  Path 3: Keys, 1, 1                       -> "Path 3: 1 key"
Done -> stub -> seat @0 and @1 from the payload's two key: records, leave @2/@3
§8p "Unfilled:" -> "What now?" -> "Back to the paths"
BACK at the path list -> "Start from?"     <- the screen this commit created
pick `decaying-multisig`
```

Result (`TestE2EBackLegPresetCarriesSeats`, run on the copy):

```
NO §8j: a preset replaced the hand-built shape while two slots were seated,
and under tr the preset numbers @0/@1/@2 onto different paths.

path list after the preset:
  "Spend paths slots:4 / keys available:2
   Path 1: 2-of-2 +13140 blocks
   Path 2: 1 key  +26280 blocks
   Path 3: 1 key  +block 1000000 ..."

STUB AFTER THE PRESET:
  "... Slot @0: 73c5da0a m/48h/0h/0h/2h   Slot @1: 73c5da0a m/48h/0h/1h/2h"

A SEAT SURVIVED: the stub still shows a seated slot after the path list was
replaced by a preset that renumbers it.
```

The operator's whole hand-built shape was silently replaced by a three-tier timelocked
policy **and both seats stayed attached to slot indices that now serve different paths.**
Measured mapping for that pair:

```
hand-built [{2of2},{1of1},{1of1}]  sig="w0/2,1,1,"  ik=1
  slots = [{@0 p1 o0} {@1 p0 o0} {@2 p0 o1} {@3 p2 o0}]
decaying-multisig preset            sig="w0/2,1,1,"  ik=-1
  slots = [{@0 p0 o0} {@1 p0 o1} {@2 p1 o0} {@3 p2 o0}]
  @0, @1, @2 all MOVED
```

The key the operator seated as "Path 2's sole spending key" is now Path 1 key 1 of a
2-of-2 locked for 13140 blocks. This is precisely the harm `composer_backleg_test.go`'s
own comment names: *"a misassignment does not fail — it derives another wallet's address
and shows it to the operator as proof."*

### Also reproduced on the production function directly

`composerStartStep(ctx, th, st, true)` with every slot seated, real screens, picking
`decaying-multisig`:

```
BEFORE: sig="w0/2,1,1," slots=[{0 p2 o0} {1 p0 o0} {2 p0 o1} {3 p1 o0}] assigned=[0 1 2 3]
NO §8j CONFIRM was drawn ...
AFTER : ret=true sig="w0/2,1,1," slots=[{0 p0 o0} {1 p0 o1} {2 p1 o0} {3 p2 o0}] assigned=[0 1 2 3]
slot @0 still seated (src=0) ... ({Index:0 Path:2 Ordinal:0} -> {Index:0 Path:0 Ordinal:0})
slot @1 still seated (src=1) ... slot @2 ... slot @3 ...
```

### How wide is it

Exhaustive over the six presets offered under `tr`, against the lock-free, hash-free
hand-built list with the same per-path key counts (i.e. what an operator gets touching no
lock screen at all):

```
plain-multisig                 sig=w0/3,      ik hand=-1 preset=-1  moved=0/3  safe
simple-timelocked-inheritance  sig=w0/1,1,    ik hand= 0 preset= 0  moved=0/2  safe
kofn-recovery                  sig=w0/3,1,    ik hand= 1 preset=-1  moved=4/4  *** RENUMBERS, NO §8j ***
tiered-recovery                sig=w0/2,2,    ik hand=-1 preset=-1  moved=0/4  safe
hashlock-gated                 sig=w0/1,1,    ik hand= 0 preset=-1  moved=0/2  safe
decaying-multisig              sig=w0/2,1,1,  ik hand= 1 preset=-1  moved=3/4  *** RENUMBERS, NO §8j ***
```

Two of six, both reachable with nothing but path-add and key-count taps. With a lock on
the hand-built list, `simple-timelocked-inheritance` and `hashlock-gated` join them
(measured: hand `[{1of1,older},{1of1}]` → `[{@0 p1}{@1 p0}]` vs preset `[{@0 p0}{@1 p1}]`
at identical signature `"w0/1,1,"`).

`wsh`/`sh`/`sh(wsh)` are safe — `numberSlots(list, -1)` is listed order there, so equal
per-path `N` implies an equal mapping. **The defect is `tr`-only, and `tr` is row 0 of the
wrapper picker.**

### What would close it

The root cause is one line of `composerShapeSignature`: it must also capture what
`lowerTr` keys on. Adding the internal-key path (or, equivalently, each path's
`isBareSingle()` under a `tr` wrapper) to the signature closes C-1 **and** I-1 below in
one place, and needs no change to `composerStartStep`. Not prescribing the remedy —
reproducing the defect; a signature that asks `md` for the numbering rather than
re-deriving it would be stronger still.

---

## I-1 (IMPORTANT, PRE-EXISTING — not introduced here) — a lock edit permutes `tr` slots with no guard and no discard

Same root cause, second door, already on `70008da` and outside this diff.
`composerPathEdit` (gui/composer_shape.go, `case 1`) runs `composerLockEdit` with neither
`composerShapeGuard` nor `composerApplyShapeEdit` — deliberately, on §7d's "a lock or hash
edit moves no slot". Under `tr` that premise is false:

```
before: sig="w0/1,1," slots=[{Index:0 Path:1 Ordinal:0} {Index:1 Path:0 Ordinal:0}]
after : sig="w0/1,1," slots=[{Index:0 Path:0 Ordinal:0} {Index:1 Path:1 Ordinal:0}]
A LOCK EDIT PERMUTES SLOTS UNDER tr AT AN UNCHANGED SIGNATURE: @0 served path 1, now serves path 0
```

Reachable by: build `[{1 key, time lock}, {1 key}]` under `tr`, seat a slot, then
`Path 1 -> Time lock -> None`. Flagged because the same fix closes it, and because the
comment asserting the premise is load-bearing in three places. **This does not gate this
merge** — it was already broken — but it should not be lost.

---

## M-1 (MINOR) — nothing pins the wrapper-picker exit, and it drops the composition silently

`composerStartStep` returning false is now the only exit from the leg, and it is correct
(verified below), but no test in the suite asserts it: `grep` over `gui/*_test.go` for
"leaves the composer" / "opening-screen" finds only the two comments in
`composer_flow.go` (lines 150, 303). Behaviourally, three Backs from a fully built path
list exits the composer and drops the wrapper, every path, every lock and every seat with
no confirm. Pre-existing (the old leg did it in two Backs), and the fix makes it *harder*,
not easier — recording it only because the leg is now the sole exit and is untested.

---

## Per-item results

### 1. Diff scope — VERIFIED

```
$ git diff 70008da..05466727 --stat
 gui/composer_backleg_test.go | 276 +++++++++++++++++++++++++++++++++++++++++++
 gui/composer_flow.go         | 103 +++++++++++-----
 gui/composer_flow_test.go    |  27 +++--
 gui/composer_presets.go      |  22 ++--
 4 files changed, 384 insertions(+), 44 deletions(-)
```

Exactly the four expected files. **No hunk touches seating, the codec, the stub screen,
the census or the engrave path.** Every hunk read. `composer_flow_test.go`'s change is the
shipped Back test rewalked — and its old comment ("The re-pick after the Back below does
NOT pass through it") was the defect encoded as intent; the rewrite is correct.
`composerPresetPick`'s new `replace` return is the right shape for W-6's second pass.

### 2. W-6 closed — VERIFIED, and the inverse claim proved

Forward, on the fix: `TestComposerBackFromThePathListReturnsToTheStartFromScreen` and
`TestComposerBackLegWrapperChangeAsksBeforeDiscardingSeats` and
`TestComposerWrapperChangePermutesSlotsAtEqualCount` all PASS unmutated.

**MUTATION A — the shipped (pre-fix) Back leg restored** (`composerWrapperPick` alone).
First run with `panic("MUTATION-A-LINE-RAN")` on the mutated line, to prove it executes
rather than assuming it:

```
panic: MUTATION-A-LINE-RAN [recovered, repanicked]
FAIL	seedhammer.com/gui	0.006s
```

Then with the panic removed:

```
--- FAIL: TestComposerBackFromThePathListReturnsToTheStartFromScreen
    composer_backleg_test.go:106: Back at the path list did not return to "Start from?";
      it landed here instead, and the preset picker was then unreachable for the life of
      the composition (W-6).
      Last frame: "Whichscript?Taproot(tr)Segwit(wsh)Nested(sh-wsh)Legacy(sh)Newpolicy"
--- FAIL: TestComposerBackLegWrapperChangeAsksBeforeDiscardingSeats
    composer_backleg_test.go:226: Back at the path list did not return to "Start from?".
```

Caught by its own named assertion, naming W-6. The line provably ran.

### 3. W-7 (the wrapper leg) closed — VERIFIED; counterexample hunt FOUND C-1

(a) §8j drawn before acceptance, (c) accepting clears every seat, (d) the stub shows no
seated slot afterwards: all three are asserted by the shipped test and confirmed by
mutation:

**MUTATION B — the §8j condition dropped** (`if false` in place of the signature +
guard test):

```
--- FAIL: composer_backleg_test.go:239: the wrapper changed on the Back leg without
    asking §8j, which §7d requires before an edit that moves slot numbering is accepted
    (W-7). The path list's own "Change the script" row asks it.
```

**MUTATION D — `st.list = next` without `composerApplyShapeEdit`** (the shipped defect's
exact shape):

```
--- FAIL: composer_backleg_test.go:266: a seat survived a wrapper change on the Back leg
    Frame: "... Slot @0: 73c5da0a m/48h/0h/0h/2h  Slot @1: 73c5da0a m/48h/0h/1h/2h
            Slot @2 expects a key at m/48h/0h/0h/3h"
```

(Note the mutated frame's own tell: `@0`/`@1` still carry `/2h` wsh origins while `@2`
reports `/3h` — the carried seat made visible.)

**MUTATION C — the blank row blanks the list** (`replace=true` for row 0):

```
--- FAIL: composer_backleg_test.go:131: "Build my own paths" discarded the composition on
    the way back; §7b's rule is that going back loses nothing.
    Last frame: "Spendpathsslots:0AddaspendpathChangethescriptDone"
```

All four mutations caught, each by the assertion written for it. **The three new tests can
fail, and fail for their own reasons.**

(b) *Declining leaves the composition and the seats exactly as they were* — the shipped
test only walks the accept. Verified separately:

```
TestVerifyDeclineKeepsEverythingAndReturnsTrue: ret=true sig before="w1/2,1,"
  after="w1/2,1," assigned=[0 1 2]   PASS
```

**Counterexample hunt** (the brief's "is there ANY route through `composerStartStep` that
ends with a seat held across a moved shape signature?"):

| route | result |
| --- | --- |
| wrapper change + blank row | SAFE — wrapper is in the signature, §8j fires |
| wrapper change + a preset in one pass | SAFE — same reason |
| blank row, same wrapper | SAFE — no change at all |
| Back at the wrapper picker with seats held | SAFE — exits the flow (see item 4) |
| `ctx.Done` mid-leg | SAFE — see item 4 |
| **a preset row, same wrapper, equal signature** | **C-1 — every seat carried** |

### 4. What the fix might have broken — VERIFIED (all three clean)

- **Back at the wrapper picker still leaves the flow.**
  `TestVerifyBackAtWrapperPickerLeavesTheLeg`: `returned false` → `composerFlow` returns.
  PASS. (Unpinned in the shipped suite — M-1.)
- **No live-lock when `ctx.Done` goes true mid-leg.** Driven at three screens by
  `quit()`, which is the only way this harness sets `Done` (`iter.Pull`'s stop runs the
  coroutine to completion, so a live-lock hangs the test):
  ```
  --- PASS: .../on_the_preset_screen_(the_Back_leg's_entry)
  --- PASS: .../on_the_wrapper_picker_(after_Back)
  --- PASS: .../on_the_§8j_confirm
  ```
  *Method note, so it is not re-derived:* setting `ctx.Done = true` directly from a test
  goroutine does **not** work in this harness and is not evidence of a live-lock —
  `runUI` (gui_test.go:632) does `ctx.Done = ctx.Done || !yield(content)`, which reads
  `Done` *before* suspending and writes the stale value back on resume, clobbering it. A
  control on `composerWrapperPick` alone (the pre-fix leg) showed identical behaviour, so
  it is the harness, not the fix.
- **The decline path cannot strand the operator.** `return true` → `composerFlow`
  `continue`s into `composerShapeFlow`, i.e. the path list the operator kept. Confirmed
  by the decline test above returning `ret=true` with the composition intact.

### 5. The confirm's placement — VERIFIED, defensible

Asking after the choice and before `composerApplyShapeEdit` satisfies §7d's "told so
before the edit is accepted" literally, and gating it on a signature move makes it
strictly quieter than the path-list row's on-entry ask. It cannot fire when nothing is at
stake:

```
TestVerifySameScriptBlankRowAsksNothing: ret=true wrapper=1 assigned=[0 1 2]  PASS
  (no "CLEARS THE KEYS" frame within 12 frames; no seat discarded)
```

The two placements now differ within one flow (on-entry for "Change the script",
conditional for the Back leg). That is deliberate and documented. **The defect is not the
placement — it is the condition**, which is C-1.

### 6. Gates, as CI runs them — VERIFIED, every one reproduced

Run on the worktree (`/scratch/code/shibboleth/.tmp/verify-gates.log`,
`verify-shard.log`, `verify-fw.log`):

| gate | controller's claim | measured | |
| --- | --- | --- | --- |
| `gofmt -l cmd/` | clean | clean | ✓ |
| `gofmt -l gui/` | 3 pre-existing files | `transaction.go`, `transaction_golden_test.go`, `transaction_txrecord_test.go` | ✓ |
| `go vet ./gui/ ./cmd/...` | only the 2 `ArtifactDir` lines | exactly those 2, nothing else | ✓ |
| `go test ./...` | 0 FAIL | 0 FAIL, 75 ok/no-test-files | ✓ |
| sharded gui (24) | all 1195 ran | `partition verified exhaustive: 1195 == 1195`, `RESULT: ok -- all 1195 tests ran across 24 shards`, wall 59s | ✓ |
| `./scripts/test-32bit.sh` | 386 + arm exit 0 | `GOARCH=386 test: exit 0`, `GOARCH=arm build: exit 0` | ✓ |
| `go build ./cmd/...` | exit 0 | exit 0 | ✓ |
| firmware (tinygo) | 1,581,428 / 62,800 | `1549632 code 31796 data 31004 bss \| 1581428 flash 62800 ram` | ✓ |

**Shard count accounted for:** `git diff 70008da..05466727 \| grep -c "^+func Test"` = **3**,
all three in the new `composer_backleg_test.go`; `composer_flow_test.go` gained no test
function (its shipped test was rewalked in place). 1192 + 3 = **1195**. ✓

**Firmware delta: +224 B flash, +0 B RAM** against the stated `70008da` baseline of
1,581,204 / 62,800. Appropriate for the change: a GUI-only edit that adds one function
(`composerStartStep`), widens one return signature and deletes an inline loop should move
flash by a few hundred bytes and must not move RAM, since it adds no static allocation —
and it did not. *One caveat, stated rather than hidden:* I reproduced the tip figure but
did **not** re-measure `70008da` myself (that needs a second checked-out tree); the
baseline is the controller's number, and it is the only figure in this report I did not
derive.

### 7. The emulator, with taps — VERIFIED

**My own geometry walk**, independent of the shipped drivers, on `emu.wasm` built from
the copy, driving by `window.shTargets()` hit regions (never synthetic key events —
W-2's lesson), one PNG per step:

```
door:                          'Nokeysloaded...ScancardsBuildanewpolicyWalletPolicy'
which-script:                  'Whichscript?Taproot(tr)Segwit(wsh)Nested(sh-wsh)Legacy(sh)'
start-from(1st pass):          'Startfrom?Buildmyownpaths...decaying-multisig'
path-list (preset seeded):     'Spendpathsslots:3Path1:2-of-3AddaspendpathChangethescriptDone'
BACK #1 -> start-from:         'Startfrom?Buildmyownpaths...'          <- W-6 half 1
BACK #2 -> which-script:       'Whichscript?Taproot(tr)...'
re-pick script -> start-from:  'Startfrom?Buildmyownpaths...'          <- W-6 half 2
blank row -> paths kept:       'Spendpathsslots:3Path1:2-of-3...'      <- §7b, nothing lost
WALK OK
```

**I looked at the PNGs** (W-3's lesson — `shScreen()` cannot see clipping or
overprinting). `w4-back-to-start-from.png`: title "New policy", all seven rows rendered
inside the panel with "Build my own paths" as the highlighted default row, lead "Start
from?" at the foot, Back and confirm chevrons unobscured — nothing clipped, nothing
overprinted. `w7-paths-kept.png`: "Spend paths / slots: 3 / Path 1: 2-of-3 (selected) /
Add a spend path / Change the script / Done" — the composition legibly intact after the
round trip. Shots at `/scratch/code/shibboleth/.tmp/w6shots/` (scratch — regenerate with
the script inlined below if needed).

**Shipped drivers, all against the copy** (`EMU=/scratch/code/shibboleth/.s4e-verify/cmd/emu`):

```
RC_capture_composer.py=0        ("all legs matched the host", keyed A+B and keyless,
                                 engraved strings byte-for-byte across the air gap)
RC_capture_walletpolicy.py=0    (wallet id + 4 addresses MATCHED against the host)
RC_capture_seating.py=0
RC_capture_tr_pathological.py=0
```

No journey regression.

---

## Reproduction (inlined — the scratch copies do not survive)

Copy the worktree, drop this in `gui/`, run it. It is the whole of C-1 in one file:

```go
package gui

import (
	"testing"
	"testing/synctest"

	"seedhammer.com/md"
)

func TestC1BackLegPresetRowCarriesSeats(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		ks := func(k, n uint8) *md.KeySet { return &md.KeySet{K: k, N: n, Sorted: true} }
		st := &composerState{list: md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			{Keys: ks(2, 2)}, {Keys: ks(1, 1)}, {Keys: ks(1, 1)},
		}}}
		st.assigned = make([]composerAssignment, composerSlotCount(st.list))
		for i := range st.assigned {
			st.assigned[i].src = i // every slot seated
		}
		before, _ := md.Compose(st.list)

		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		done := make(chan struct{})
		frame, quit := runUI(ctx, func() {
			composerStartStep(ctx, &descriptorTheme, st, true) // the Back leg
			close(done)
		})
		defer quit()

		if got, ok := pumpUntil(frame, "Start from?", 24); !ok {
			t.Fatalf("Back leg did not open on the preset screen.\n%q", got)
		}
		click(&ctx.Router, Down, Down, Down, Down, Down, Down) // decaying-multisig
		click(&ctx.Router, Button3)
		if got, ok := pumpUntil(frame, "CLEARS THE KEYS", 24); !ok {
			t.Errorf("NO §8j for a preset that renumbers a seated composition.\n%q", got)
		}
		for i := 0; i < 400; i++ {
			select {
			case <-done:
				i = 400
			default:
				frame()
			}
		}
		after, _ := md.Compose(st.list)
		for i := range st.assigned {
			if st.assigned[i].src >= 0 && (before.Slots()[i] != after.Slots()[i]) {
				t.Errorf("slot @%d still seated (src=%d) but now serves %+v, was %+v",
					i, st.assigned[i].src, after.Slots()[i], before.Slots()[i])
			}
		}
	})
}
```

Environment: `PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH`,
`TMPDIR=/scratch/code/shibboleth/.tmp`, `CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`,
`go test -mod=readonly ./gui/ -run TestC1 -v`.

The emulator walk script is at `/scratch/code/shibboleth/.tmp/w6walk.py` (scratch); it is
a ~120-line standalone playwright driver modelled on `capture_composer.py`'s `drive()`
plus `shots_composer.js`'s `chooseRow`/`waitFor`/`goTo`, serving
`.s4e-verify/cmd/emu` on port 8861.

---

## Counts

| severity | n | |
| --- | --- | --- |
| Critical | 1 | C-1: Back-leg preset rows carry seats across a `tr` renumbering, no §8j — **introduced by this commit** |
| Important | 1 | I-1: lock edit permutes `tr` slots unguarded — **pre-existing**, same root cause, does not gate this merge |
| Minor | 1 | M-1: the leg's sole exit is untested and drops the composition silently — pre-existing |
| Nit | 0 | |

**W-6: closed.** **W-7: closed for the wrapper, open for the preset rows.** Each of the
three new tests can fail, and fails for its own named reason. Every CI gate reproduced.
No hunk outside the two findings' scope. The merge should wait on C-1, whose fix is one
line in `composerShapeSignature` and closes I-1 with it.

*F-470 not re-opened. No secret-handling defect observed.*
