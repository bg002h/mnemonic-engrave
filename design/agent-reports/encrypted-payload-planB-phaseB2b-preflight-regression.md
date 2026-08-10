# Phase B2b — pre-flight regression review (blast radius outside the encrypted-payload path)

**Reviewer:** independent agent (Opus 5, 1M), read-only over
`/scratch/code/shibboleth/seedhammer-b2b` at `b2b`.
**Diff under review:** `a01b666..b2b`, 6 commits / 18 files / +1753 −135.
**Brief:** *Does this diff change anything outside the encrypted-payload path?*
Explicitly out of scope and not re-derived: plan correctness (4 R0 rounds), the
mutation rows (16/16 KILLED), the build/TinyGo green, the unwind design.

**Verdict on the assigned question: NO regression found outside the
encrypted-payload path.** Both claims in the brief hold. Three findings are
recorded below; the first is real and material but lands *inside* the encrypted
path — it surfaced while verifying Task 1/Task 3's structural claims, and it
falsifies a comment this diff introduces, so it is reported rather than dropped.

---

## 1. Claim: "Task 1's move of `Run`'s body into `runWithFlow` is a PURE MOVE"

**VERIFIED — pure, with exactly one unlisted delta, which is compiled out of
production firmware.**

Method: mechanical. Extracted `Run`'s body from `a01b666:gui/gui.go` and
`fbe31ab:gui/run_flow.go`, stripped comments and blank lines, and diffed. The
entire result is 4 hunks:

| delta | assessment |
| --- | --- |
| `version :=` → `versionText :=` (the inner shadow renamed) | identifier only; the outer `version` param is read once in the same expression, exactly as before |
| `uiFlow(ctx, …)` → `flow(ctx, …)` | the declared new parameter; `Run` passes `uiFlow` |
| the `d.Reset()` … `d.Draw()` block lifted into a `draw := func(content op.Op)` closure | statement-for-statement identical inside; call site is `draw(content)` at the same point |
| `if onDraw != nil { onDraw(content) }` appended to `draw` | the declared new parameter; `Run` passes `nil` |

Everything below the first hunk — the inner event loop, `AppendEvents`,
`idle.start` refresh, `ctx.Reset()`, `Router.Events` gating, the `idle.active`
edge, `saver.State{}` reinit, `saver.State.Draw`, `minFrameTime`, both
`WakeupAt` calls, `break`, `startTime = time.Now()` — is byte-identical.

**The one unlisted delta (Nit, finding 3):** `layoutTime := time.Since(startTime)`
moved. Old order was `d.Reset()`; `dirty := image.Rectangle{Max: pl.DisplaySize()}`;
`layoutTime := …`; `pl.Dirty(dirty)`. New order measures `layoutTime` *before*
`draw(content)`, i.e. before `d.Reset()` and before `pl.DisplaySize()`.
`layoutTime` is consumed only by `if debug { stats.Dump(drawTime, layoutTime) }`,
and `debug` is a build-tag const — `gui/nodebug.go:5` `const debug = false` is the
production selection — so the shipped firmware compiles this out entirely.

**Orphans after the `saver` import removal:** none.
- `grep -n "saver\." gui/gui.go` → no hits. The import removal is exactly
  balanced by `gui/run_flow.go`'s own `saver` import.
- `idleTimeout` survives in `gui/gui.go:2949` and is read from `gui/run_flow.go:188`.
- `runtimeStats` / `stats.Dump` / `EngraverStats` / `NewContext` all retain
  callers.
- `Run`'s signature is unchanged; both production callers
  (`cmd/controller/main.go:34`, `cmd/emu/main.go:33`) are untouched.

---

## 2. Claim: "When `armed` is false, the event loop behaves as it did before, with
one deliberate exception (`ctx.keepAwake`)"

**VERIFIED.** Same method: normalized diff of `fbe31ab:gui/run_flow.go` against
`b2b:gui/run_flow.go`. Inside the inner event loop the additions are, in order:

1. `armed := ctx.wipe.armed()` — `ctx.wipe` is `nil` on every non-secret path
   (grep: the only production assignment is `gui/unlock_session.go:82`, inside
   `unlockSecretSession`, with `defer func() { ctx.wipe = nil }()`).
   `(*wipeGuard).armed` has an explicit `if g == nil { return false }`, so this
   is two nil checks and `false`.
2. `if len(evts) > 0 || (ctx.keepAwake && !armed)` — **the declared exception.**
   `ctx.keepAwake` has exactly one production setter: `ctx.KeepAwake()` at
   `gui/unlock_kdf.go:302`, inside `unlockDerive`. No plaintext flow can set it,
   so on those paths this term is a constant `false` and the condition reduces
   to `len(evts) > 0`, verbatim as before.
3. `if armed != a.armed { … }` — `false != false`, dead on unarmed paths.
4. `if wipeNowHook != nil && wipeNowHook()` — `wipeNowHook` is a package var,
   `nil` in production; both test call sites register `t.Cleanup(func() { wipeNowHook = nil })`
   (`gui/run_flow_test.go:199`, `:262`) and `armWipe` self-clears on first fire.
5. `if armed { … warning … }` nested inside the existing `if a.idle.active` —
   skipped, so the `else` path is the pre-existing
   `a.idle.state.Draw(pl)` / `WakeupAt(now.Add(minFrameTime))` / `continue`.

Point by point against the brief's three sub-questions:

- **The screensaver still covers a running 21-minute cut.** Yes. The saver branch
  is reached identically: nothing on the plaintext engrave path sets
  `ctx.keepAwake`, and `armed` is `false`, so `a.idle.state.Draw(pl)` runs on
  exactly the same condition and with the same 40 ms throttle. This also holds
  for a *secret* cut: `wipeGuard.armed()` returns `false` while
  `j.Status().State` is `engraveRunning`/`engraveStopping`, so the saver, not
  the warning, takes the screen for the whole cut.
- **`Router.Events` gating is unchanged.** `ctx.Reset()` then
  `if !a.idle.active { ctx.Router.Events(d, evts...) }`, in that order, with no
  new statement between them. `EventRouter` itself is untouched by the diff.
- **`ctx.WakeupAt` scheduling is unchanged for the unarmed case.** Both existing
  calls (`now.Add(minFrameTime)` in the saver branch, `idleWakeup` at the tail)
  are at the same points with the same arguments. The only new `WakeupAt`
  (`now.Add(time.Second)`) is inside `if armed`.

**The `FrameCallback` rewrite is provably equivalent on unarmed paths.** Old:
`ctx.Done = ctx.Done || !yield(op)`. New: `if ctx.Done { return }; if !yield(o) { ctx.Done = true }`.
Case analysis:
- `Done` true on entry: `||` short-circuits, so the old form did *not* call
  `yield` either, and left `Done` true. New returns early. Identical.
- `Done` false, `yield` returns true: old writes `false`, new writes nothing.
  Identical *unless the body mutated `Done` during the call* — which on an
  unarmed path nothing does (the only mutators are the two wipe sites, both
  gated on `armed`/`wipeNowHook`, and both `break` rather than return true).
- `Done` false, `yield` returns false: both set `Done = true`. Identical.

**Session-loop re-entry does not disturb shared state.** `uiFlow` re-probes
`ctx.Platform.PayloadReader()` and rebuilds `StartScreen` on every entry
(`gui/gui.go:1599-1606`); there is no `sync.Once` anywhere in `gui/`. A wipe
cannot fire while an engrave job is live (`armed()` returns `false` for
`engraveRunning`/`engraveStopping`, and `errs` is only sent *after*
`runEngraving`'s deferred `d.Close()`), so the unwind can never abandon an open
engraver device.

**`ctx.Done` in the shared flows is untouched.** `git grep -n "\.Done"` over
`gui/*.go` at both revisions returns the identical set of `for !ctx.Done`
sites across all 24 flow files (`bundle_flow`, `codex32_polish`, `slip39_polish`,
`multisig_build`, `verify_address`, `passphrase_flow`, `bip85`, `qa`, …); only
`gui/gui.go`'s line numbers shift, by the size of the removed `Run` body.

---

## 3. `SecretsResident` → `RecordsResident`

**Identifier-only. No behavioural change.** The method body is unchanged
(`for _, r := range p.Secret { if !IsSecret(r.Class) { continue } … }`); the rest
of the diff in `seal/session.go` is doc prose. There are **zero production
callers** at either revision — every `RecordsResident` hit outside
`seal/session.go` is a comment or a `_test.go` assertion
(`seal/session_test.go`, `gui/unlock_session_test.go`, `seal/unlock_key_test.go`,
`bip39/bip39.go` comment, `bip39/bip39_test.go` comment,
`gui/unlock_session.go:297` comment, `gui/wipe_guard.go:6` comment,
`gui/run_flow.go:137` comment). The `bip39` diff is comment text only.

## 4. `Context` gaining two fields

**No effect.** `Context` is constructed in exactly one place — `gui/gui.go:92`,
`c := &Context{Platform: pl, Styles: NewStyles()}` — and is used as `*Context`
universally: there is no value-typed `Context` parameter, receiver, field,
slice, or map in the tree, and no dereference-copy. It has never been comparable
(it embeds `op.Buffer`, which holds two slices), so `==` on it would not compile
before or after. Both new fields are unexported, so no out-of-package composite
literal can break. Zero values (`nil`, `false`) are the disarmed/no-op case.

---

# Findings

## F1 — Important — the wipe leaves the last secret frame's runes in the abandoned `op.Buffer`, and the session tail asserts the opposite

**Location:** `gui/run_flow.go`, the session-loop tail comment (the block
beginning `// NOTHING to scrub here, and that is worth stating…`), together with
`gui/op/op.go:374-378`.

**The claim under test**, quoted verbatim from the diff:

> `clear(b.refs)` (gui/op/op.go:376) runs on the last frame drawn — then again
> after every discard-guarded Frame during the unwind. **The abandoned Context's
> buffer is already zeroed by the time control reaches this line.**

**It is false.** `op.Buffer.Reset` is:

```go
func (b *Buffer) Reset() {
	b.args = b.args[:0]   // TRUNCATED, not zeroed
	clear(b.refs)
	b.refs = b.refs[:0]
}
```

`clear` covers `refs` (the `[]any` holding faces and images) only. Glyphs encode
the **rune itself into `args`**: `gui/op/op.go:132`,
`return MaskOp{encodeOp(b, opMask, 0, []any{glyphImage, face}, uint32(r))}`.
So every character of every label drawn survives `Reset` in `args`' backing
array.

**Machine-checked, not argued.** A throwaway module in the scratchpad (`replace`
onto the worktree; the repo was not modified) drove the production path —
`widget.Label(&b, poppins.Regular16 style, …)` — then `Reset`, then read the
backing array through the slice's `cap`:

```
before Reset: len(args)=127 len(refs)=48
after  Reset: len(args)=0   len(refs)=0
args backing array cap=128
lowercase runes recoverable from the args BACKING ARRAY after Reset: "abandonzoohammer"
```

The three words went in as three `widget.Label` calls and came back out of the
"zeroed" buffer verbatim and in order.

**Why this diff makes it worse rather than merely inheriting it.** Before
`a01b666..b2b` there was one `Context` for the whole process, so `args` was
appended into and truncated every frame and the backing array was continuously
overwritten by later screens. This diff moves `ctx := NewContext(pl)` *inside*
the session loop: after a wipe the old `Context` — and its `op.Buffer` — is
abandoned, and the restarted session allocates a fresh zero-value `Buffer`. The
old array is therefore **frozen holding the last frame drawn before the wipe**,
which on `SeedScreen.Confirm` is the twelve words, and nothing ever writes over
it again. It persists until TinyGo's precise GC happens to recycle that
allocation, which is neither prompt nor guaranteed on an idle machine sitting at
the main menu.

**Operator impact:** the feature's stated purpose is that decrypted seed
material is *erased* after 3 minutes idle. After the wipe the seal-owned records
and the flow locals are gone, but a complete, ordered, rune-level copy of the
twelve words remains in RAM in an unreferenced heap array. Nothing in the phase
detects this: `RecordsResident()` scans `p.Secret` and cannot reach an
`op.Buffer`, and the assertion that would have caught it is the comment that
states the opposite. Note this is *not* an outside-the-path regression — it is
inside the encrypted path — and the exposure needs physical RAM access on a
secure-boot device. But it is a machine-checkable claim in the shipped source
that is wrong, in the one place the phase exists to be right about, and the
operator is about to flash it.

**Suggested fix (one line, plus a decision):** `clear(b.args)` in
`Buffer.Reset()` before the truncation. `Reset` is per-frame, so if zeroing a
few-hundred-element `uint32` slice every frame is unwanted on the device, scrub
only where it matters: keep a handle to the outgoing `ctx` at the session tail
and zero its buffer through a new `op.Buffer.Scrub()` before dropping it — the
tail is exactly the "nothing to scrub" comment's location, so the comment
becomes true instead of being deleted. Either way the comment must be corrected:
as written it will stop the next reviewer from looking.

## F2 — Minor — `wipeGuard.armed()` reads as a pure predicate but calls a mutating accessor, now on every Run tick including behind the screensaver

**Location:** `gui/wipe_guard.go:46` (`switch j.Status().State`), against
`gui/engraver.go:126-149`.

**Claim:** `engraveJob.Status()` is not a query. It non-blockingly receives from
`e.progress` (mutating `e.status.Completed`), receives from `e.errs` (setting
`e.errs = nil` and transitioning `e.status.State`), and ends with
`if e.status.State == engraveRunning { e.Start() }` — a call that can spawn the
engraving goroutine. Before this diff there was exactly one caller family, the
`EngraveScreen` frame loop (`gui/gui.go:2722, :2728, :2766, :2838, :2884`) plus
`gui/qa.go:29`. This diff adds a second, structurally different one: Run's event
loop, every tick.

**Verified benign as it stands, and I want that on the record so it is not
"fixed" by accident:** Run's loop and the flow execute on the *same* goroutine
(the loop is the range-over-func body of `it`, which is driven by
`ctx.FrameCallback` inside `ctx.Frame`), so there is no race on `e.status`; both
drains accumulate into `e.status`, so the screen loses no progress and no
completion; and `Start()`'s `if e.errs != nil { return }` guard makes the
auto-restart unreachable from this call site, because the only way to have
`errs == nil` is to have just drained it, which leaves `State` at
`engraveDone`/`engraveStopped`/`engraveFailed`, never `engraveRunning`.

**Operator impact if it later stops being benign:** the new call site polls a
running secret job roughly every 40 ms for the entire screensaver-covered cut —
on the order of 30,000 extra `Status()` calls across a 21-minute plate, at a
point where the screen is parked and no one is watching. Any future change that
makes `Status()` consume a one-shot signal, or that makes the restart branch
reachable, would fire from Run's loop with the needle in the metal and no frame
on the display to show it.

**Suggested fix:** documentation, not code — `Status()` is load-bearing here
(reading `j.status.State` directly would miss the running→done transition while
the screen is parked, which is precisely §10.2.4 row 2's trigger). Add one line
to `armed()`'s doc comment saying it calls a mutating accessor deliberately, and
one line to `Status()`'s saying it now has a second caller on Run's tick.

## F3 — Nit — Task 1's "pure move" also moved the `layoutTime` measurement point

**Location:** `gui/run_flow.go`, `layoutTime := time.Since(startTime)` at the
top of the range body.

**Claim:** `a01b666:gui/gui.go` measured `layoutTime` after `d.Reset()` and after
`pl.DisplaySize()`; the new code measures it before both. Task 1's commit
describes the move as verbatim "plus two parameters and one corrected line", and
this is neither.

**Operator impact:** none on shipped firmware. `layoutTime` feeds only
`if debug { stats.Dump(drawTime, layoutTime) }`, and production selects
`gui/nodebug.go:5`, `const debug = false`, so the statement is compiled out. The
only effect is that a `-tags debug` build reports a layout time a few
microseconds smaller. Recorded solely so the "pure move" claim is accurate in
the record, since a future reviewer will diff against it.

**Suggested fix:** none needed; amend the claim wording if the commit message is
ever revised.

---

## Checks run (all clean, no finding)

- `Context` value-copy / comparison sweep — none exist.
- `SecretsResident` → `RecordsResident`: zero production callers at either revision.
- `saver` import removal: no orphaned symbol in `gui/gui.go`.
- `ctx.Reset()` production callers: one (`gui/run_flow.go:173`); the added
  `c.keepAwake = false` is read-before-clear at `gui/run_flow.go:150`.
- `ctx.KeepAwake()` production callers: one (`gui/unlock_kdf.go:302`).
- `ctx.wipe` production assignments: one install + one deferred clear, both in
  `unlockSecretSession`.
- Test-hook hygiene (`wipeNowHook`, `warnBufHook`, `unlockMnemonicParsedHook`):
  every call site registers `t.Cleanup`; `armWipe` additionally self-clears.
- `gui/op/buffer_len.go`: new exported `Len` on `op.Buffer`; no production
  caller, no interface in the tree it could accidentally satisfy
  (it returns two ints), dead in the TinyGo build.
- `ExtractText`/`Drawer.text`: test-only in production, so the `Drawer` shared
  across sessions retains no rune buffer. (`Drawer.Reset` clears only `inputs`
  and `skipInputOps` — worth knowing, but `maskStack`/`jumpStack` are drained by
  `draw` itself and `text` is nil.)
