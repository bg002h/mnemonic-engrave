# IMPL-F440 — BACK dismisses a modal

**Worktree** `/scratch/code/shibboleth/sh-worktrees/f440-modal-back`, branch
`f440/modal-back`, from fork main **a0c1615**. **Not pushed.**

**Commit** `9762542` — `gui: F-440 -- BACK dismisses a modal, and the modal says so`
(`gui/gui.go` +41/-2, `gui/modal_back_test.go` new).

## The change

`ErrorScreen.Layout` gains a second `Clickable` bound to **Button1** that
dismisses exactly as `ok` (Button3) does, and the nav row now draws
`assets.IconBack` beside the checkmark so the route is **visible and tappable**.
One `Layout`, **143** `showError`/`showNotice` call sites fixed at once.

A second Clickable rather than `ok.AltButton = Button1`: a `NavButton` is also
the **touch target** (`op.Input`), and an AltButton has no drawn region — an
invisible, untappable route is most of what F-440 was.

## Force-ack: no site needs one, and it was checked, not assumed

**Answer: none. No `ForceAck` variant was added.** Three independent reasons:

1. **The type split already exists.** Every screen in the firmware that must
   force an acknowledgment is a **`ConfirmWarningScreen`** — a different type
   with a cancel/confirm pair and a `ConfirmDelay` (hold-to-confirm). Enumerated:
   `bip85.go:225`, `derive_xpub.go:546`, `multisig_build.go:812`,
   `unlock_flow.go:227`, `gui.go:2908`, and `holdToConfirm` in
   `slip39_polish.go`. `ErrorScreen` is used by nothing of that shape.
2. **There is nothing to skip.** `ErrorScreen.Layout` returns `(op, dismissed
   bool)` — one boolean, carrying no "which button" — and **all five** of its
   caller loops are `if dismissed { return/break }`: `showModal`
   (`slip39_polish.go:23`, which is all 143 sites), `showCodex32Error`
   (`codex32_polish.go:160`), `showSeedError` (`gui.go:904`), `showErr` inside
   the seed flow (`gui.go:2940`) and `showErr` inside `DescriptorScreen.Confirm`
   (`gui.go:3128`). One exit, one destination. BACK reaches the same place OK
   does.
3. **The engrave-critical surfaces are not modals.** The mid-engrave consent,
   the EXPERIMENTAL warning and the plate confirm are `ConfirmWarningScreen` or
   `confirmReviewScreen`, none of which routes through this Layout.

## The regression I introduced, and what caught it

Worth recording because it is the interesting half. My first version drained
**both** Clickables unconditionally (`okd := …; backd := …; if okd || backd`).
That is wrong: the router is a **single queue** whose head is taken by the first
matching filter, so a frame that polled `ok` and then `back` **swallowed a
Button1 press queued behind the dismissing Button3** — a press meant for the
screen underneath.

`TestRecoverRejectsNonCodex32` hung forever. Its script is exactly
`Button3, Button3, Button1` — OK the entry, dismiss the modal, Back out of
recovery — and the modal ate the Back, so recovery never returned. **Found by
the gui shard, not by review**: two shards sat at 20-minute timeouts, and the
panic named the test.

Fixed by short-circuiting (`if s.ok.Clicked(ctx) || s.back.Clicked(ctx)`), so a
dismissal consumes **at most one click** and a pending event is left for whoever
comes next. `TestErrorScreenDismissalLeavesTheNextClickAlone` now pins that
property directly, so the next person who finds the short-circuit untidy learns
why from a failure rather than from a comment.

## The Minor: skipped, and why

**Not done: a shared line in the modal chrome saying the dismissal ends the
program.** It would be *false at most sites*. Same chrome, opposite outcomes,
measured in this tree:

- `bundleAbortWarning` → `bundleEngrave` returns aborted → `walletPolicyFlow`
  **returns**. The program does end.
- `bundleDoneEmpty` and `bundleDonePending` → `showError`, then **fall straight
  back into the same gather loop**. The program does not end.
- `showNotice` (e.g. "Verify OK") → the flow **continues**.

An honest sentence is therefore per-site prose, which is 143 strings and a
different change. Recorded rather than silently dropped.

## TDD and mutation proof

Written first, red first, against unmodified `a0c1615`:

```
--- FAIL: TestErrorScreenDismissesOnBackAndOnOK/Button1_(back)_dismisses
        b1 dismissed the modal = false, want true
--- FAIL: TestF440BundleIncompleteModalDismissesOnBack
        BACK did not dismiss the Bundle Incomplete modal; the flow is still
        drawing it after 12 frames. Last frame: "Stoppedatcard1of1(md1
        descriptor).Thissetisnotausablebackupyet…BundleIncomplete"
```

Mutation proof on the **final** shape — BACK's dismissal removed, the glyph left
drawn, so the tests are shown to measure the **binding** and not the icon:

```
=== MUTATION: back no longer dismisses ===
--- FAIL: TestErrorScreenDismissesOnBackAndOnOK/Button1_(back)_dismisses
--- FAIL: TestF440BundleIncompleteModalDismissesOnBack
=== restored ===  ok  seedhammer.com/gui
```

Four tests committed: the generic contract (Button1 and Button3 dismiss;
Button2, Center and Up do not), the stray-release guard (a modal must not be
dismissed by a release whose press it never saw), the one-click property above,
and the bench walk itself.

## Gate tails (`9762542`)

```
gui shard    1032 top-level tests, partition verified exhaustive: 1032 == 1032
             === wall: 31s ===  RESULT: ok -- all 1032 tests ran across 24 shards
non-gui      go test <all but ./gui>: exit 0, 52 ok
go vet ./... no new findings against an a0c1615 baseline taken the same way
             (the first baseline was from the older tree; its only diff was a
             line-number shift in an untouched file, so it was re-taken by
             stashing on THIS tree)
gofmt -l     clean on gui/gui.go and gui/modal_back_test.go
tinygo       pico-plus2, exit 0
             1197801  269247  31620  30956 | 1498668  62576 | total
```

**Worktree clean** — `git status --short --untracked-files=all` empty; one commit
above `a0c1615`; nothing pushed.

## Note for the merge

F-440 fixes a **dead button on a live screen**. It does **not** fix the
operator's other two reports: `design/agent-reports/BUG-wallet-policy-back-hang.md`
Appendix B identifies a **permanent, hardware-only device lock** on the same BACK
edge (`stopScanner` → `Poller.Close` → a dropped `st25r3916` cancel token →
an unbounded channel wait on the UI goroutine). That is a separate Critical, not
implemented, awaiting your go-ahead.

---

## Follow-on: F-441 landed on the same branch (`4698223`)

`Poller.Close` bounded (`ErrCloseTimeout`, device untouched on that path),
`stopScanner` abandons instead of blocking, `Device.Interrupt` drains before
signalling. Mutation-proved in both directions; the `d.cancel` sweep found no
second site. **Correction carried in `BUG-wallet-policy-back-hang.md` Appendix
C:** the dropped signal I named in Appendix B is *not* the cause — at capacity 1
a stale token cancels the next waiter exactly as a fresh one would, checked
executably — so the bound is the fix rather than a backstop.

Both commits are on `f440/modal-back` for one review: `9762542` (F-440, a dead
button on a live screen) and `4698223` (F-441, a live screen on a dead device).
