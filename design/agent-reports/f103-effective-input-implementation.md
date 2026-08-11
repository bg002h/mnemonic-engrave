# F-103 implementation — §10.2.4's idle clock keys on EFFECTIVE input

**Implementer report.** Worktree `/scratch/code/shibboleth/sh-wt-f103`, branch
`f103-effective-input`, from `main` @ `5831335`. One commit: **`8975111`**.
Nothing merged, nothing pushed. `design/FOLLOWUPS.md` untouched.

Files changed (staged by path, no `git add -A`):

| file | change |
| --- | --- |
| `gui/run_flow.go` | the fix: `effectiveInput`, the `a.pressed` field, the call site |
| `gui/idle_effective_input_test.go` | NEW — three tests |
| `gui/run_harness_test.go` | additive `poll`/`polls` seam on `deadlinePlatform`, nil in every existing test |

`git diff --stat`: 3 files changed, 402 insertions(+), 1 deletion(-).

---

## 1. What was wrong and what changed

The defect, verbatim from the tree at the branch point (`gui/run_flow.go:251`):

```go
if len(evts) > 0 || (ctx.keepAwake && !armed) {
    a.idle.start = now
}
```

Arrival is not presence. `processTouch`
(`cmd/controller/platform_sh2.go:398-417`) dedupes on **exact equality** of
`(touching, pos)`, so a panel holding contact — the factory protective film —
delivers a fresh event on every poll whose reported position drifts by one
pixel. `a.idle.start` is refreshed forever, `a.idle.active` never becomes true,
and because §10.2.4's warning is nested inside `if a.idle.active` the operator
gets no countdown, no wipe, and nothing on screen. Silent, permanent, on a
machine holding a decrypted seed.

The change, at the same site:

```go
effective := effectiveInput(evts, &a.pressed)
if effective || (ctx.keepAwake && !armed) {
    a.idle.start = now
}
```

plus the predicate, which is the whole of the new logic:

```go
func effectiveInput(evts []Event, pressed *bool) bool {
	effective := false
	for _, e := range evts {
		pe, ok := e.AsPointer()
		if !ok {
			if _, isFrame := e.AsFrame(); isFrame {
				continue
			}
			effective = true
			continue
		}
		if pe.Pressed != *pressed {
			*pressed = pe.Pressed
			effective = true
		}
	}
	return effective
}
```

`a.pressed` is a new field on the existing `a` struct, which lives **above** the
session loop. That placement is deliberate: contact is physical and a wipe does
not lift a finger, so resetting it per session would manufacture a spurious
press edge on the first reading after every wipe — the one moment the clock
most needs to be honest.

The predicate is bound to its own variable rather than inlined as the left
operand of `||`, because it also **advances** `a.pressed`. Inlined, a later
reordering of the two terms would short-circuit past the tracking and lose the
contact state with no compile error and no test failure at the moment of the
edit.

---

## 2. The definition of effective input, and why

**Effective input is input that resolves to a change in state.** Per event kind
the router handles:

| kind | effective? | why |
| --- | --- | --- |
| **pointer** | only on a **contact-state change** (released→pressed, pressed→released) | see below |
| **rune** | always | discrete, self-terminating operator action |
| **button** | always | same; and there is no keypad on the SH2 at all |
| **frame** | never | a camera image is machine output, not operator input |

**Pointer — why a position-only move does not count.** The task asked for this
one to be justified explicitly. There is no cursor and no hover on this
hardware. `processTouch` emits a position *only while contact is asserted*, and
zeroes `Pos` on release. So a position-only event means exactly one thing: *the
contact point moved while still held*. That is precisely and only what an object
resting on the panel produces — thermal and electrical jitter around a fixed
contact — and it is what a human produces only in the middle of a drag, which is
always bracketed by the down and up edges that **do** count. The cost to a
genuine operator is therefore a single uninterrupted drag lasting longer than
3:30 with no press or release inside it. This UI has no drag gestures; its
longest hold is `confirmDelay`, **1 second** (`gui/gui.go:323`, read from the
source, not from memory).

**Rune / button — why they count unconditionally.** The SH2 has neither a keypad
nor a keyboard. The only producer of either kind in the whole tree is
`cmd/controller/debug_sh2.go:74-83`, which synthesises them from the debug serial
line, in press/release pairs. Nothing on this machine can emit them as a
continuous stream, so the "arrival is not presence" argument has no purchase
here. (Verified by grepping every `.Event()` construction outside tests: the only
other producers are `platform_sh2.go:375,393` and `cmd/emu/platform.go:92`, both
pointer.)

**Frame — why it is excluded even though nothing produces it.** A `FrameEvent`
carries a camera image. A source delivering frames at 30 Hz is *precisely* the
shape of the defect being fixed. No platform in this tree produces one today, so
the line costs nothing behaviourally; it is written down and tested so that a
future scan path cannot silently re-open F-103.

### The design alternative I rejected, and why

The follow-up's Option 2 has two readings: "a completed press/release" and "a
router-consumed event". **Router-consumed does not work**, and the reason is
structural rather than aesthetic:

```go
if !a.idle.active {
    ctx.Router.Events(d, evts...)
}
```

While the machine is idle — which is exactly when the screensaver and the
§10.2.4 warning are on screen — **the router is not run at all**. No event
during the warning is ever consumed by anything. So a router-consumed predicate
would break §10.2.4's touch-to-keep affordance outright ("Touch the screen to
keep it" is drawn by `wipe_warning.go:72`, and the spec makes "any touch resets
it" normative). It also breaks four existing tests that use untargeted
`p.tap()` purely as an idle refresh (`run_flow_test.go:51,231,276,540`), which
the harness documents as deliberate (`run_reentry_test.go:76`).

Contact **edges** are visible in both states, which is what lets one predicate
serve the running UI and the parked warning without a special case.

### What this does NOT close — stated rather than left for a reviewer

A panel whose contact **flickers** — repeatedly crossing the ft6x36 detection
threshold rather than holding — produces genuine press and release edges, and
would still hold the clock off. This is strictly narrower than the behaviour it
replaces (which any non-identical reading tripped, *including pure position
jitter*), and it is the shape the 2026-08-10 mechanism report listed second
("the touch/no-touch boundary flickering"), not the shape it reproduced. It is
**not closed**.

Closing it needs the follow-up's Option 3 — a plausibility bound on how fast a
human can tap, or a maximum credible contact duration. That is a tunable
constant in the middle of a funds-safety control, and picking it blind (with no
instrumented capture of what a filmed panel actually reports on this hardware)
would be guessing. **Recommendation to the controller: if F-103 is to be closed
rather than narrowed, the missing input is a bench capture of the ft6x36 event
stream under a film — the same "free bench check" the pre-hardware preflight
recorded and never ran.**

---

## 3. TDD — the tests, and that they can fail

### Fails first

The regression test was written and run **before** any change to `run_flow.go`.
Verbatim, against the unmodified branch point:

```
=== RUN   TestSpuriousTouchDoesNotHoldOffTheWipe
    idle_effective_input_test.go:99: Run exceeded 100000 ticks without terminating -- flow is probably parked (screensaver?). 100001 frames drawn, last = "CUTTHISPLATE"
--- FAIL: TestSpuriousTouchDoesNotHoldOffTheWipe (13.15s)
=== RUN   TestGenuineTapsStillHoldOffTheWipe
--- PASS: TestGenuineTapsStillHoldOffTheWipe (0.16s)
FAIL
FAIL	seedhammer.com/gui	13.327s
FAIL
EXIT=1
```

That failure **is** the 2026-08-10 experiment, now committed: 100,000 spurious
touch polls at a 10 ms floor ≈ 1000 s of fake time, 4.8× past the 3:30
deadline, and the last frame drawn is still the content screen — zero warnings,
zero wipes. `mustFinish`'s `maxRunFrames` cap is 100,000 and one poll costs one
tick, so the cap and the experiment are the same number by construction.

`TestGenuineTapsStillHoldOffTheWipe` **passing** pre-fix is correct and
intended: it is the guard on property 2, and its job is to fail if the predicate
is tightened too far. M3 below confirms it does.

After the fix, same command:

```
=== RUN   TestSpuriousTouchDoesNotHoldOffTheWipe
--- PASS: TestSpuriousTouchDoesNotHoldOffTheWipe (4.16s)
=== RUN   TestGenuineTapsStillHoldOffTheWipe
--- PASS: TestGenuineTapsStillHoldOffTheWipe (0.12s)
PASS
ok  	seedhammer.com/gui	4.316s
EXIT=0
```

### The three tests

1. **`TestSpuriousTouchDoesNotHoldOffTheWipe`** — the F-103 regression. A
   platform that asserts contact forever with a 9-long cycle of distinct
   positions (so `processTouch`'s equality dedupe suppresses none of them) and
   never releases. Asserts the warning appears, the wipe fires (session 2), and
   — crucially — that it happens **on schedule**: `warnAt - guardInstalledAt`
   within 5 s of `idleTimeout`. A fix that merely lengthened the window would
   still leave a secret resident for an unbounded time, and a presence-only
   assertion would not notice.
2. **`TestGenuineTapsStillHoldOffTheWipe`** — property 2. A tap (press+release)
   every 20 s of fake time across **4× the window**; no warning, no wipe.
3. **`TestEffectiveInputClassification`** — 14 table cases naming each kind
   and each transition, including the two properties the 3-minute window is too
   coarse to see: that the **whole batch** is scanned so `*pressed` is never left
   stale, and that a camera frame is not input.

**Every absence assertion is paired with a positive control** (`p.polls ≥
21000`, `*taps ≥ 30`, `elapsed ≥ runFor`), each phrased as `INCONCLUSIVE:` and
`t.Fatal`. An event generator that silently stopped producing would otherwise
pass "no wipe fired" for a reason having nothing to do with the fix — the
"empty output is not absence" failure mode.

### Existing tests

**None was weakened, deleted, or adjusted.** The three tests the brief singled
out as encoding hard-won behaviour —
`TestIdleWindowIsNotDoubledByALateArmEdge` and both
`TestCutEnding*StartsAFreshWindow` (F-106's 6:00 doubling) — pass unchanged,
and **all three die under M1 and M2** below, so they are demonstrably still
biting on the line I touched. Nothing had to be reported under the "stop and
report" clause.

The one edit to an existing test file is additive: `deadlinePlatform` gains an
optional `poll func() []Event` and a `polls` counter, `nil`/unused in every
pre-existing test, following the file's own `onDirty` seam pattern. It exists
because the device's "don't starve touch input" fast path
(`platform_sh2.go:371`) returns *before* it ever arms the deadline timer, so a
continuously-asserting panel cannot be modelled by a platform that blocks on a
deadline.

---

## 4. Mutation table

Each mutant was applied to `gui/run_flow.go`, and **the mutated line was grepped
out of the file and printed before the run was judged** — a no-op mutant and a
surviving mutant produce identical output, which has produced a false result in
this project before. After each run the file was restored from a saved copy and
the restore confirmed by `md5sum` (`d1288ffd12836da1f441fbbf1904e811`).

Every count is `grep -c '^--- FAIL'` over the saved run output. **No number in
this table was counted by hand** — the first draft of the commit message did
contain two hand-counted values (15 and 16); both were wrong, both were caught
by re-running with a machine count, and the commit was amended before this
report was written.

| # | mutation | applied line, verified | result | top-level tests killed |
| --- | --- | --- | --- | --- |
| **M1** | invert the predicate | `342: effective := !effectiveInput(evts, &a.pressed) // MUTANT M1` | **KILLED** | **14** |
| **M2** | always true — `effective := true` | `64: effective := true // MUTANT M2` | **KILLED** | **15** |
| **M3** | always false — `return false` | `80: return false // MUTANT M3` | **KILLED** | **4** |
| **M4** | revert call site to `len(evts) > 0` | `343: _ = effective // MUTANT M4`<br>`344: if len(evts) > 0 \|\| (ctx.keepAwake && !armed) {` | **KILLED** | **1** |
| **M5** | early return on first effective event | `76: return true // MUTANT M5` | **KILLED** | **1** |

Detail on the ones that matter:

**M1 (14 killed)** — `TestSpuriousTouchDoesNotHoldOffTheWipe`,
`TestIdleWindowIsNotDoubledByALateArmEdge`,
`TestCutEndingDuringTheParkStartsAFreshWindow`,
`TestCutEndingAfterTheDeadlineStartsAFreshWindow`,
`TestRunHarnessHonoursDeadline`, `TestRunWarningThenWipe`,
`TestRunWarningCountdownIsReal`,
`TestRunTapDuringWarningResetsAndReturnsContent`,
`TestRunPostCutWindowRestartsFromCutEnd`, `TestRunWarningBufferDoesNotGrow`,
`TestRunKeepAwakeCannotPostponeAnArmedWipe`,
`TestRunSealedPayloadReentryAfterWipe`,
`TestUnlockPassphraseWarningShowsTheRow4Subject`,
`TestWipeZeroesEveryPinnedBufferAtRunLevel`.

**M3 (4 killed)** — `TestEffectiveInputClassification`,
**`TestGenuineTapsStillHoldOffTheWipe`**,
`TestRunTapDuringWarningResetsAndReturnsContent`,
`TestRunPostCutWindowRestartsFromCutEnd`. The second of those is the
property-2 guard: an over-tightened predicate that never lets anything refresh
the clock is caught by a test whose failure message says an operator "would lose
it mid-entry".

**M4 (1 killed)** — exactly `TestSpuriousTouchDoesNotHoldOffTheWipe`. This is
the cleanest result in the table: M4 *is* the shipped defect, and the new
regression test is the only thing in a 48-package suite standing between the
tree and it. Before this commit, nothing did.

**M5 (1 killed)** — exactly
`TestEffectiveInputClassification/press_and_release_in_one_batch`. This is the
subtle one the doc comment warns about: returning at the first effective event
leaves `*pressed` stale, so the next genuine press edge is invisible. It is
invisible to both behavioural tests, which is why the classification table
exists.

M3's first form (`return false` with `effective` left declared) failed to
compile — `effective declared and not used`. That is recorded rather than
hidden: the mutant was re-formed as `_ = effective; return false` so the run
being judged was a real one. A mutant that does not compile is not evidence.

---

## 5. Gate — verbatim

Run against the committed tree (`git status --short` empty), exit statuses read
directly from `$status`, never through a pipe.

```
=== gofmt -l gui/ ===
GOFMT_EXIT=0
gui/bip85_test.go
gui/md1_expand_fuzz_test.go
gui/multisig_build_test.go
gui/multisig_match.go
gui/multisig_testhelpers_test.go
=== go test ./... ===
GOTEST_EXIT=0
ok=48  FAIL=0
=== tinygo ===
TINYGO_EXIT=0
2632704 bytes
```

**On the five `gofmt -l` names:** they are pre-existing and were present at the
branch point. None is a file this commit touches — `git status --short` before
staging listed exactly `gui/run_flow.go`, `gui/run_harness_test.go`,
`gui/idle_effective_input_test.go`, and none of those appears in the list. Not
fixed here: unrelated files in a security fix's diff hide the security fix.

Full `go test ./...` output (all 48 `ok`, 0 `FAIL`, `seedhammer.com/gui
43.014s`) is reproducible with the command in the brief; the branch-point
baseline measured identically (exit 0, 48 ok, 0 FAIL) before any edit.

Device build:

```
tinygo build -o /tmp/f103.uf2 -target pico-plus2 -stack-size 16kb \
    -gc precise -opt 2 -scheduler tasks ./cmd/controller
→ exit 0, 2,632,704 bytes
```

---

## 6. Unresolved / for the controller

1. **The flicker shape is narrowed, not closed** (§2 above). This is the only
   substantive gap and it is deliberate. It needs a bench capture of what a
   filmed ft6x36 actually reports before a constant can be chosen honestly.
2. **This is a normative change to §10.2.4.** The spec currently says *"any
   touch resets it"*; after this commit it is *"any contact-state change resets
   it"*. The affordance an operator sees is unchanged — touching the screen
   during the warning still keeps the session, because a finger arriving is a
   press edge — but the spec sentence is now inexact and F-103's own entry
   flagged this as needing the R0 loop rather than a patch. **The spec text was
   not edited** (not in scope, and `design/` is the controller's).
3. **Operator documentation is still required and still not written.** F-103's
   candidate fix 1 ("remove the screen film" in the setup runbook) is
   independent of this change: with the film on, the panel is unusable as an
   input device regardless of what the idle clock does. This commit only ensures
   the machine *wipes* rather than sitting there holding a seed forever.
4. **`design/FOLLOWUPS.md` was not edited**, per the brief. F-103's entry still
   reads as open.
5. **Not merged, not pushed.** Branch `f103-effective-input` @ `8975111`.
