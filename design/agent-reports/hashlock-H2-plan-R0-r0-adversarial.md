# H2 device plan — R0 round 0, adversarial / failure-states lens

**Artifact:** `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` at engrave `02abee6`
**Spec:** `design/SPEC_hashlock_H2_device.md` (GREEN `55ee7a4`)
**Fork:** `/scratch/code/shibboleth/seedhammer` main `c4a64fc`; gated tree read at `/scratch/code/shibboleth/.tmp/h2-gate`
**Host:** mnemonic-secret `504ff46` (corpus unchanged since `cd0a60f`)
**Lens:** construct the inputs, timings and states under which the device this plan builds fails. Read-only; nothing committed; no sub-agents.

**Counts: 2 Critical / 4 Important / 7 Minor / 5 Nit.**

---

## C-1 — The hardened derivation stalls for three minutes on its first frame and is then parked by the screensaver; `ctx.Frame` never returns

**Constructed state.** Any hardened derivation on the real SH2 or in the emulator. Add a key-less path, choose `Type a hashlock phrase`, type a phrase of 20 characters or more (so no §4.3 modal intervenes), pick `Hardened (about 10 s)`. Nothing else is required — no unusual input, no timing race.

**Trace.**

The plan's derive screen (plan Task 4 Step 3, `gui/composer_hashlock.go`, the `hashlockDeriveFlow` progress callback) builds its frame and calls:

```go
ctx.Frame(op.Layer(pctOp, leadOp, nav, titleOp, op.Color(&ctx.B, th.Background)))
return true
```

It calls neither `ctx.WakeupAt(...)` nor `ctx.KeepAwake()` anywhere.

`Context.Frame` (`gui/gui.go:95-100`) only invokes `FrameCallback` and resets the buffer. The yield is inside `runWithFlow`'s `FrameCallback` (`gui/run_flow.go:208-230`), which hands control to the inner loop at `gui/run_flow.go:287-410`:

```go
wakeup := ctx.Wakeup                              // run_flow.go:308
evts = pl.AppendEvents(wakeup, evts[:0])          // run_flow.go:309  -- BLOCKS
...
ctx.WakeupAt(idleWakeup)                          // run_flow.go:408
break
```

`ctx.Reset()` (`gui/gui.go:123-130`) clears `Wakeup` each tick, and the only value written back before control returns to the flow is `idleWakeup = a.idle.start + idleTimeout` (`run_flow.go:368, 408`). `idleTimeout = 3 * time.Minute` (`gui/gui.go:3584`). `Context.WakeupAt` takes the *minimum* (`gui/gui.go:110-114`), so a flow that never calls it leaves `ctx.Wakeup` at the three-minute idle deadline.

Both production platforms block on that deadline:

- `cmd/controller/platform_sh2.go`, `AppendEvents`: `p.timer.Reset(time.Until(deadline))`, then `select` on the timer, `p.wakeups`, stdin and the touch interrupt.
- `cmd/emu/platform.go:219-239`: `d := time.Until(deadline); ... t := time.NewTimer(d); select { <-t.C; <-p.wakeups; <-p.events }`.

Nothing in the derive frame produces a `Wakeup()`: `Platform.Wakeup()` is called by `ConfirmDelay.Progress` (`gui/gui.go:466`, only while a confirm hold is in progress) and by the engrave goroutine — neither is live here. `Clickable`'s auto-repeat `ctx.WakeupAt` (`gui/widget.go:64`) fires only for a *pressed* Up/Down/Left/Right, which this screen has none of.

So the first callback fires at `done = 501` (after one `Step(500)`, ~51 ms), draws one frame, and `AppendEvents` then blocks for the remainder of the three minutes. When it returns, `now.Sub(idleWakeup) >= 0` (`run_flow.go:369`) so `a.idle.active` flips true, and the unarmed branch at `run_flow.go:402-406` draws the screensaver and `continue`s — **it does not `break`**, so `ctx.Frame` never returns and `DeriveHardened` is frozen at 501 of 100,000 iterations.

This is the exact failure the fork already documents and fixed once. `gui/unlock_kdf.go:295-336` carries both calls with the reason spelled out verbatim:

> "So a WakeupAt placed AFTER Frame governs the NEXT frame, never this one -- and frame 1 then inherits whatever the preceding screen left, which is Run's own ctx.WakeupAt(idleWakeup) ... The derivation parks at 500/300,000 iterations, the screensaver takes the screen, and ctx.Frame never returns."

and, for `KeepAwake`:

> "Run refreshes a.idle.start only on `len(evts) > 0`, and a derivation produces no events, so without the KeepAwake call below a derivation longer than idleTimeout still trips the saver -- whose branch `continue`s without breaking, so ctx.Frame does not return and the KDF stops until a touch."

**Wrong outcome.** `Deriving` renders once at 0% or 1%, freezes for three minutes, then the screensaver takes the panel. A touch un-parks it and buys exactly one more 500-iteration slice. 200 slices are needed. The advertised 10-second derivation cannot complete; the operator's realistic response to a frozen screen mid-composition is a power cycle, and `composerState` is RAM (spec §4.4), so the whole policy composition is lost. `KeepAwake` alone would not repair it: with the deadline still three minutes out, each slice would still block for `idleTimeout`, giving ~200 x 180 s.

**Why no gate in the plan can see it.** `runComposerAddPath` (Task 4 Step 2) builds `newPlatform()`, whose `AppendEvents` **ignores the deadline entirely** — `gui/gui_test.go:430-434` appends any queued events and returns immediately. It also drives the flow through `runUITouch`, which never runs `runWithFlow` at all, so the idle clock, `idleWakeup` and the screensaver branch are not present in the harness. The fork has a platform that models this correctly — `deadlinePlatform` (`gui/run_harness_test.go:58-88`), whose own comment names `unlockDerive`'s `ctx.WakeupAt(time.Now())` as the reason it needs a tick floor — and it is used only by `run_flow.go`'s own tests. The F-93 regression test `TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver` (`gui/run_flow_test.go:672-693`) drives `unlockDerive` by name, so a second derivation screen inherits none of its protection. The one gate that *would* have caught this is the plan's own Task 5 Step 1 emulator walk ("Hardened once: assert `3cf5d421..b70a4c12` after the countdown (allow 30 s)"), which the plan itself records as **not run**.

**SUGGESTION.** In `hashlockDeriveFlow`'s progress callback, immediately before `ctx.Frame(...)`, add the two calls in `unlockDerive`'s order and with a comment pointing at `gui/unlock_kdf.go:295-336` and F-93:

```go
ctx.KeepAwake()
ctx.WakeupAt(time.Now())
ctx.Frame(op.Layer(pctOp, leadOp, nav, titleOp, op.Color(&ctx.B, th.Background)))
```

Order is load-bearing — Run reads and clears both from inside that very call. Then add a gate that can fail: a `synctest` test in the shape of `TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver` driving `hashlockDeriveFlow` (or the whole phrase route) on `newDeadlinePlatform()` with a tick floor, asserting the derivation completes; and record the mutation "delete `ctx.WakeupAt(time.Now())` -> the test reports `Run exceeded N ticks`" in the Task 4 mutation table. Run Task 5 Step 1 before this plan closes: a plan may not close while one of its own gates has never been executed.

---

## C-2 — `TestDecodeMS1PreimageIsShapeExact` cannot fail on a wrong preimage: an off-by-one in `DecodeMS1Preimage` ships green, and spec §7.1's kind-row lockstep is unimplemented

**Constructed mutation.** In the plan's Task 2 Step 3 decoder (`codex32/mspayload.go`), change the one line that extracts the payload:

```go
copy(preimage[:], d[1:])     // as planned
copy(preimage[:], d[:32])    // mutation: off by one, includes the kind byte
```

**Trace.** The corpus's kind row is `preimage_hex = "ab"*32` and `ms1 = "ms10hashsq...kzv2ncy60u7z9c"` (verified: the plan's hard-coded plate string is byte-identical to `kind[0].ms1`, both 75 characters). So `d = [0x03, 0xab x32]`, and under the mutation `preimage = [0x03, 0xab x31]`.

The plan's whole assertion on the decoded value is (Task 2 Step 1):

```go
if x[0] == 0 && x[31] == 0 {
    t.Fatalf("preimage looks zero: %x", x)
}
```

Under the mutation `x[0] == 0x03` and `x[31] == 0xab`, so the condition is false and the test proceeds. The remaining rows in that test — `DecodeMS1(s) == errMSBadPrefix`, the `entr single` / share / entr-id cases, and the 17-byte `errMSBadLength` case — are all decided before or independently of the `copy`, so none of them move. **The mutation ships green.**

The corpus carries exactly the constant that would kill it, and no test in the plan reads it. Spec §7.1 states the clause plainly:

> "the `kind` row: `DecodeMS1Preimage` on `kind[0].ms1` returns `kind[0].preimage_hex`; the entr-32 pair row -> `errMSBadPrefix`"

Neither half is implemented. The plan's `corpus` struct in `hashlock/hashlock_test.go` declares only `PreimageHex` and `MS1` for the kind rows — it does not parse the corpus's `digest` field or `entr32_pair_ms1` at all, and `grep` over the gated tree confirms `entr32_pair_ms1` appears only inside the vendored JSON, never in a test. The plan's substitute, `TestKindRowPreimageDigest`, asserts only `Digest(&x) != x` — that SHA-256 is not the identity function — while its own doc comment claims it establishes "the digest of that preimage is what the confirm modal must show for a `--hex` X". The corpus's `digest` constant `9a2db2e23f1504cd056606553ac049c5e718e8f9ce9233876df1a7a1821af885` is right there and unused.

**Wrong outcome.** A decoder that returns 32 bytes shifted by one reports PASS. No screen calls `DecodeMS1Preimage` in H2, so no operator meets it this stage — but the plan's stated purpose for the function is that "the kind has one decoder and one test" (spec §1 item 4), and the test does not test it. The next stage that wires a screen to it inherits a green gate over an unverified decoder, and the value it decodes is a spend preimage.

**SUGGESTION.** Add to `codex32/mspayload_test.go` a row driven from the vendored corpus, not from a literal: read `hashlock/testdata/hashlock-v0.8.json` (the pattern `loadHashlockCorpusForGUI` already establishes for a sibling package), assert `New(kind[0].ms1)` decodes to exactly `kind[0].preimage_hex`, and assert `DecodeMS1Preimage(New(kind[0].entr32_pair_ms1)) == errMSBadPrefix`. Record the mutation `d[1:] -> d[:32]` in the Task 2 mutation table alongside the existing `!f.Unshared` row. In `hashlock/hashlock_test.go`, add `Digest string \`json:"digest"\`` to the kind struct and make `TestKindRowPreimageDigest` compare `Digest(&x)` against that constant instead of against the identity, or delete the test and say so.

---

## I-1 — The reconciliation line was moved out of an always-shown modal into one whose guard is false for the ordinary wallet shape

**Constructed state.** A `wsh` policy with two spend paths: path 1 = keys, path 2 = key-less with a hash set through the phrase route. Complete it and tap `Done`.

**Trace.** §4.5's drop-order step 2 (folded as build-gate fix 4) moved "Before you fund this wallet, run ms hashlock with this phrase and method on the host and check the digest matches." out of `composerCopyHashlockConfirm` and into `composerCopyHashEveryPathPhrase` (plan Task 4 Step 1). That body is reachable only through `composerCopyHashEveryPathFor(st)` at `gui/composer_shape.go:443`, which the plan does not re-guard:

```go
if composerEveryPathHashed(st.list) {
    showError(ctx, th, "Spend paths", composerCopyHashEveryPathFor(st))
}
```

and `composerEveryPathHashed` (`gui/composer_state.go:239-249`) returns false the moment any path has `Hash == nil`:

```go
for _, p := range list.Paths {
    if p.Hash == nil {
        return false
    }
}
```

Path 1 is keyed and unhashed, so the modal never fires and the sentence is drawn nowhere in the firmware.

**Wrong outcome.** Spec §4.5 says of that line: "The reconciliation line converts a divergence discovered at spend time into a five-minute check." After the move it reaches only the wallet in which *every* spend path is hashed. For the mixed shape — keys on one path, a hashlock on another, which is the shape a hashlock is normally used for — the operator is never told to reconcile the device's digest against `ms hashlock` on the host. C-2 above shows the device's derivation is pinned by the corpus, but the host reconciliation is the operator's only cross-check that they typed the phrase they think they typed, and it is silent.

The spec sanctioned the destination; neither spec nor plan checked the destination's guard, and no test in Task 4 Step 2 constructs a mixed-shape policy and asserts the line appears.

**SUGGESTION.** Keep the line reachable independently of `composerEveryPathHashed`: either restore it to `composerCopyHashlockConfirm` (measured headroom on that body is 186 characters against a required 80, so a ~100-character sentence does not fit — so instead) show it as its own `showError` at the end of the phrase route immediately after HOLD, gated on `st.hashByPhrase`, and add it to both copy gates. Add a Task 4 test that builds a keyed path plus a phrase-hashed path and asserts the reconciliation text is drawn.

---

## I-2 — `st.hashByPhrase` is set once and never cleared, so §8h can instruct the operator to back up a phrase that guards nothing

**Constructed states, both reachable with three taps.**

(a) Add a key-less path, set its hash by phrase (`st.hashByPhrase = true`, plan Task 4 Step 3). Return to the path menu, choose `Hash lock` again, choose `Type 64 hex`, enter a digest read off a preimage plate. The path's hash is now the typed digest; `hashByPhrase` is still true.

(b) Add path 1 key-less, hash by phrase. `Remove path`. Add another key-less path and take a payload `hash:` record. `hashByPhrase` is still true.

**Trace.** `gui/composer_state.go` gains only `hashByPhrase bool`, with the comment "records that AT LEAST ONE path's hash was set through the phrase route (H2)". The single writer is `hashlockPhraseRoute`'s assign arm (`st.hashByPhrase = true`); there is no clear in `composerHashEdit`'s `noneRow` arm, none in the hex arm, none in the payload-digest arm, none in `composerMoveUp`, none on path removal, and none in `composerAddPath`'s two `st.list.Paths = st.list.Paths[:idx]` rollbacks (`gui/composer_shape.go:269, 277`). The field's stated invariant is therefore not maintained.

**Wrong outcome.** At Done, `composerCopyHashEveryPathFor(st)` returns the phrase form, telling the operator "Back up the phrase and its method, or the preimage plate, separately" and (per I-1's line) to run `ms hashlock` with "this phrase and method" — for a policy in which no live path's hash came from a phrase. In case (a) the correct backup artifact is the preimage plate the digest was copied from; the copy sends the operator to a phrase that no longer secures anything.

The related gap, same field: two paths set by two *different* phrases both set the flag, and §8h's text is singular ("the phrase and its method"). The confirm modal's advisory "One phrase per policy." is the only thing standing between that and one of the two phrases going unrecorded; nothing refuses or warns at the second phrase.

**SUGGESTION.** Make the predicate derived rather than latched: replace the field with a function over `st.list` — but a `*[32]byte` does not record its provenance, so instead keep the field and clear it wherever a hash leaves the phrase route: the `noneRow` arm, the hex arm, the payload arm, and both `composerAddPath` rollbacks. Better, store provenance per path (a parallel `hashByPhrase []bool` indexed with `Paths`, spliced by `Remove path`/`Move up`) so the predicate is "some *live* path was set by phrase". Add a Task 4 test for case (a): set by phrase, overwrite with hex, assert §8h draws the shipped form.

---

## I-3 — `Type 64 hex`'s Back behaviour changes and the plan claims a test that does not exist

**Constructed state.** Create a key-less path, hold through EXPERIMENTAL, choose `Type 64 hex`, press Back at the hex pad.

**Trace.** Today `composerHexEntry` returning false propagates out of `composerHashEdit` (`gui/composer_hash.go:166-169`), and at creation `composerAddPath` deletes the path (`gui/composer_shape.go:269`). The plan's replacement changes it (Task 3 Step 2):

```go
case sel == rows.hexRow:
    d, ok := composerHexEntry(ctx, th)
    if !ok {
        continue // Back from hex entry returns to `Which hash?`, path intact
    }
```

The plan states the change and then says: *"The test in Step 1 does not cover it; Task 4's harness tests do (Back from hex entry at creation keeps the path)."*

They do not. Grepping the gated tree's `gui/composer_hashlock_test.go` for the hex route returns only `encoding/hex`, `hashlockHashHex`, and the `{"64 hex", hashlockAnchorHardH, "Use the Type 64 hex row"}` *phrase-refusal* row — no test selects `rows.hexRow`, and `TestHashlockBackContractKeepsThePath` never enters the hex route. `TestWhichHashRowsAreLabelKeyed` (Task 3 Step 1) asserts row labels and indices only; it never drives a selection.

**Wrong outcome.** A behaviour change to a shipped funds-relevant route — whether Back at the hex pad destroys a path and its EXPERIMENTAL consent — lands with no coverage, and the plan's own text asserts coverage exists, so a reviewer or implementer checking the claim finds it satisfied. This is the shape §4.6 was written to prevent, applied to the sibling route rather than the new one.

**SUGGESTION.** Add the test the plan already names: `runComposerAddPath`, reach `Which hash?`, tap the hex row, dismiss the §8i modal, press Back at the pad, assert the frame returns to the `Path 1 hash` title with `len(st.list.Paths) == 1`. Add the mutation "`continue` -> `return false` in the hex arm -> the path count assertion fails".

---

## I-4 — The `Deriving` screen's zero-state lead is unreachable, so the operator's first sight of the screen is an estimate extrapolated from one 500-iteration slice

**Constructed state.** Any hardened derivation (once C-1 is fixed and the screen animates at all).

**Trace.** `hashlock.DeriveHardened` (plan Task 1 Step 4) calls `progress` only from inside `for !d.Step(500)`, and `seal.NewDeriver` sets `d.done = 1` after computing U_1 (`seal/pbkdf2.go:100-102`). The first callback therefore arrives with `done = 501`. The plan's callback reads:

```go
lead := composerCopyHashlockDerivingLead()
if elapsed := time.Since(start); done > 0 && elapsed > 0 {
    left := time.Duration(float64(elapsed) * float64(total-done) / float64(done))
    lead = fmt.Sprintf("About %d seconds left.", int(left.Seconds()+0.5))
}
```

`done > 0` is true on every call that can occur, and `elapsed > 0` is true after a 500-iteration slice, so the zero-state branch is dead code. No frame is drawn between entering `hashlockDeriveFlow` and that first callback, so `composerCopyHashlockDerivingLead()` — "Deriving. This takes about 10 seconds." — is never on the panel.

Spec §4.4 specifies it: "the countdown screen, title `Deriving`, zero-state lead *'Deriving. This takes about 10 seconds.'*, then `About N seconds left.`". `unlockDerive`'s equivalent is reachable because `unlockKDFLead` is called with the pre-step `d.Done()` and guards `done <= 0` at a point where the zero state exists (`gui/unlock_kdf.go:219-221`).

The plan's own test cannot see this: `h.mustReach("Deriving")` matches the screen **title**, which is drawn either way.

**Wrong outcome.** The first number the operator sees is extrapolated from a single 51 ms slice that includes HMAC construction and first-call setup, so it can read high and then fall — a countdown that starts wrong instead of the calibrated sentence the spec chose for exactly that first frame. `composerCopyHashlockDerivingLead` also occupies a row in `composerCopyTable` and a slot in the declared-count literal for copy no operator sees.

**SUGGESTION.** Either draw one frame with the zero-state lead before the first `Step` (mirroring `unlockDerive`, which draws after checking Back and before the frame's estimate is meaningful), or hold the zero-state lead until at least N slices have accumulated (`done >= 2000`, say) so the first estimate rests on more than one slice. Add an assertion on the lead text, not the title, in `TestHashlockMethodModalsFireOnCondition`'s hardened arm.

---

## M-1 — The fit gate measures two of the three confirm bodies 20 normalised characters short of what production draws, and labels all five rows with the wrong renderer

Production wraps both method warnings: `composerConfirmScreen(ctx, th, "Hardened", composerConfirmBody(composerCopyHashlockHardenedWarning()))`. The plan's fit rows do not:

```go
{ "the hashlock hardened warning (H2 §4.3)", composerCopyHashlockHardenedWarning() },
{ "the hashlock sha256 warning (H2 §4.3)",   composerCopyHashlockSHA256Warning()   },
```

while the §4.5 row *does* wrap (`composerConfirmBody(composerCopyHashlockConfirm(...))`) — an inconsistency inside the plan's own five rows. `composerConfirmBody` appends "\n\nHold button to confirm." (`gui/composer_copy.go:32-33`), 20 normalised characters. Measured in the gated tree (`go test -v -run TestModalsThisBlockTouchesAreDrawnInFull ./gui/`): hardened 169 drawn / headroom 397, sha256 206 / 360, confirm 290 / 186, refusal 91 / 476, §8h 254 / 262. So production's true headroom for the two warnings is ~377 and ~340 — far above the 80-character margin, and nothing fails today. The defect is that the gate is not measuring the string the panel draws.

Separately, `TestModalsThisBlockTouchesAreDrawnInFull` renders every row with `errorScreenBody` (`gui/modal_fits_test.go:359`), while three of the plan's five rows are `ConfirmWarningScreen` in production. That one is cosmetic: `ErrorScreen.Layout` and `ConfirmWarningScreen.Layout` both delegate to the same `(*Warning).Layout(ctx, th, dims, title, body)` over the same `warningBodyClip(dims)` (`gui/gui.go:409, 683, 472-473`), so capacity is identical — but the row names assert a screen the renderer is not.

**SUGGESTION.** Wrap the two warning rows in `composerConfirmBody(...)`, and either route the three confirm rows through `confirmWarningBody` (already defined at `gui/modal_fits_test.go:118`) by giving the table a per-row renderer, or add a one-line comment recording that the two shapes share `Warning.Layout` so capacity is the same.

## M-2 — `IsMS1Shaped` strips a narrower separator set than the host's, while its doc comment claims to be the host's predicate

Host (`crates/ms-cli/src/format.rs:12-14`): `is_display_separator(c) = c.is_whitespace() || c == '-' || c == ','` — *all* Unicode whitespace, so U+000B, U+000C, U+0085, U+00A0, U+2000-200A, U+3000 and the rest.

Plan (`hashlock/hashlock.go`, `IsMS1Shaped`): `if r == ' ' || r == '\t' || r == '\n' || r == '\r' || r == '-' || r == ','`. `\v` and `\f` are missing, as is every non-ASCII whitespace. `strings.ToLower` is also Unicode-aware where the host uses `to_ascii_lowercase`.

Unreachable this stage: `ValidatePhrase` runs the printable-ASCII rule first (both host and port, same order), so every byte reaching `IsMS1Shaped` is in `0x20..=0x7E`; and the keyboard cannot produce anything else (see the clean list below). But `IsMS1Shaped` is exported, the doc comment says it *is* `looks_like_ms1`, and on the host that predicate is also the argv guard's, where raw `\v` is reachable from a shell. `MIN_MS1_LEN = 48` and `BECH32_CHARSET` were checked and match (`crates/ms-cli/src/argv_guard.rs:98, 103`).

**SUGGESTION.** Either widen the strip to `unicode.IsSpace(r) || r == '-' || r == ','` and use `strings.ToLower` on ASCII only, or narrow the doc comment to "the host's predicate restricted to printable-ASCII input, which is all `ValidatePhrase` admits" and say why that is sufficient.

## M-3 — `hashlock.Salt` is a mutable package-level slice

`var Salt = []byte("ms-hashlock-v1")`. Nothing in production writes it, and the plan's own mutation table mutates it deliberately (`Salt = append(Salt, 0, 0)` -> 22 failures). But a 14-byte conversion lands in a 16-byte size class, so an `append(hashlock.Salt, x)` anywhere in the fork would write through the shared backing array and silently change every subsequent hardened digest, with the corpus test still green in a separate process.

**SUGGESTION.** `const saltStr = "ms-hashlock-v1"` and pass `[]byte(saltStr)` at each call site (twice: `PreimageHardened`, `DeriveHardened`). The mutation test still works — mutate the literal.

## M-4 — The §8i rule modal repeats on every re-entry to the phrase row, against §4.7's "once"

`composerHashEdit` is now a loop, and `showError(ctx, th, title, composerCopyHashRule())` sits inside it, gated on `taking`. Back at the phrase screen returns `hashlockBackToWhichHash`, `continue`s to the pick screen, and selecting the phrase row again redraws the rule modal. Spec §4.7: "The §8i rule modal fires at the pick (§5) as today, once". Under the shipped index-keyed code there was no loop, so it genuinely fired once. This is a modal the operator learns to tap through — the failure mode `composerShapeGuard`'s own comment names.

**SUGGESTION.** Latch it per `composerHashEdit` call: `shown := false` above the loop, `if taking && !shown { showError(...); shown = true }`.

## M-5 — The plan's size recipe drops the one signal the toolchain gives about `-stack-size 16kb`, and no gate exercises the new nesting depth on a 16 KB stack

Task 5 Step 2 runs `tinygo build -size short`. CI runs `-size full -print-stacks` (`.github/workflows/test.yml:139`), which is the only stack-depth report available — and nothing asserts its output either. The phrase route adds real depth on top of an already deep path: `composerAddPath` -> `composerHashEdit` -> `hashlockPhraseRoute` -> `hashlockDeriveFlow` -> `hashlock.DeriveHardened` -> the progress closure -> `layoutNavigation`/`widget.Labelw` -> `ctx.Frame` -> `runWithFlow`'s `FrameCallback` -> `draw` -> `op.Drawer.Draw` — with `showError`'s nested frame loop reachable from `hashlockPhraseFlow` on top of that. Every gate the plan defines runs on a host with an 8 MB goroutine stack. The precedent is good (`unlockDerive` draws from inside its own KDF loop and ships), which is why this is Minor rather than Important, but the plan asserts nothing about it.

Separately confirmed for the brief: there is **no** numeric flash ceiling anywhere in the tree (no partition or size assertion in `picobin/`, `cmd/controller/`, or CI), so the plan's "the acceptance is the delta against the named baseline" is accurate rather than an unsound assumption.

**SUGGESTION.** Use `-size full -print-stacks` in Task 5 Step 2, record the top stack consumers alongside the flash/RAM numbers, and state whether the deepest new chain appears in that report.

## M-6 (secret handling — non-gating per the 2026-08-27 ruling) — the phrase is never zeroed and survives in three places

`kbd.Fragment` is a Go `string` (immutable, unreachable to `clear`, freed only by GC); `phrase := []byte(kbd.Fragment)` is a fresh heap allocation dropped without `clear(phrase)` on every return path in `hashlockPhraseRoute`; and `hashlockPhraseFlow`'s re-entry copies it back into a second string via `kbd.Fragment = string(initial)`. `seal.NewDeriver`'s own comment (`seal/pbkdf2.go:60-84`) records that the HMAC ipad/opad pair stays key-equivalent for the Deriver's whole life and that `Wipe` cannot reach it. `ctx.B` retains rendered glyphs and is scrubbed only on a wipe (`gui/run_flow.go:425`); the readout is masked so those glyphs are `*`, but a `show`-revealed readout is not. The plan's L7/L15 claim, "The preimage lives on the stack here and is dropped when this function returns", is true of the digest and not of the phrase. Logged for future optimisation, per the ruling.

## M-7 — "Write down this phrase and the method now" is drawn on the one screen that does not show the phrase, and the phrase is masked by default everywhere else

`PassphraseKeyboard`'s readout is `strings.Repeat("*", utf8.RuneCountInString(k.Fragment))` unless `revealed` (`gui/passphrase_keyboard.go:435-438`), and `Clear()` — called at the end of `newPPKeyboard` — sets `revealed = false` (`:243-250`). `NewAddressKeyboard` exists precisely because a caller that wants cleartext must set it afterwards; `hashlockPhraseFlow` does not. The confirm modal then shows `hash <first8>..<last8>`, `method`, `chars`, and the instruction to write the phrase down — with the phrase nowhere on the frame.

The sharp case is the one §2 deliberately creates: a trailing or doubled space changes the digest and is **invisible in both the masked and the revealed readout**. Type `correct horse battery staple` and tap space once more; the confirm reads `chars: 29`, the operator writes 28 characters on paper, and `ms hashlock` on the host derives a different digest — the path is unspendable and nothing on the device said so. The designed mitigation is exactly the `chars: n` line (spec §4.5, journey M-5), so the class is handled; the residual is that the instruction is given at a moment when it cannot be followed without three Backs, and the counter is the only witness to whitespace.

**SUGGESTION.** Documentation-only would be enough for H3's manual, but one cheap change closes it at the moment: append `chars` to the phrase screen's counter band as it already is (`n/100`) *and* state on the confirm modal that `chars` counts spaces — e.g. `method: hardened   chars: 29 (spaces counted)`. Headroom on that body is 186, and the addition is ~17 normalised characters.

---

## Nits

- **N-1** `TestKindRowPreimageDigest`'s name and doc comment describe a check it does not perform (see C-2). Rename or delete.
- **N-2** `if composerPickScreenMaxRows < 2+3` is a tautology (`= 24`, `gui/composer_paged.go:224`). Spec §5 asks for a check "against the longest row set", and the longest row set is `len(payload hash: records) + 3`, unbounded by the payload rather than 5. No panic results — `composerPickScreen` bounds its hit areas by `j < len(rowHits)` and `start+j < len(lines)` and pages the rest — so the assertion is harmless and proves nothing.
- **N-3** `TestLockstepListIsTheOneWeDrive` asserts `len(c.Lockstep) == 4` and is named "the one we drive", but clause 3 ("kind: the entr32 pair; id/prefix mismatch both directions") and clause 4 ("the fork's pin test drives these rows in BOTH directions (encode and decode)") are undriven — nothing in the plan ever *encodes* a preimage string and compares it to `kind[0].ms1`. Clauses 1 and 2 are genuinely driven (derivation rows 5 and 6 are 100 and 101 characters; refusals rows 0-4, 6, 7 and 14 cover the rest).
- **N-4** The `default: panic(...)` arm in `composerHashEdit` is unreachable by construction: `composerPickScreen` clamps `sel` into `[0, len(lines)-1]` on the Down arm, the row-tap arm (`start+j >= len(lines)` breaks) and both pager clamps, and the four cases partition that range exactly. Spec §5 asks for it, so it stays — noted only so a future reader does not hunt for the test that kills it.
- **N-5** The harness's `hashlockKbdFor map[*sessionHarness]*PassphraseKeyboard` is written from the UI goroutine (via `passphraseWidgetHook`) and read from the test goroutine; safe only because `runUITouch` serialises on frame delivery and no `gui` test calls `t.Parallel()`. `t.Cleanup` also nils `passphraseWidgetHook` unconditionally, so nesting `runComposerAddPath` inside another helper that installs the hook (`gui/chain_walk_test.go:263`) would silently disarm it.

---

## Attack table

| Surface (brief) | Constructed case | Outcome |
| --- | --- | --- |
| Bytes — every refusals row | all 15 driven through `ValidatePhrase`; the 5 `<the kind[0].ms1 ...>` placeholders all handled by the test's switch, and an unhandled one would fail loudly rather than validate the placeholder text | clean |
| Bytes — order of the rule | refusals row 13 (grouped by 2, 112 chars) pins ms1-shape *before* the cap; matches `hashlock_phrase.rs:118-140` exactly (empty -> printable -> ms1 -> cap -> 64-hex) | clean |
| Bytes — 100 spaces / `0x7F` / empty / leading-trailing-double spaces | `0x7F` and `0x09` refused as `printable-ascii`; `  a  b ` derives the corpus constant with no fold; 100 spaces accepted (100 <= cap, not ms1-shaped, not 64-hex) | clean |
| Bytes — multi-byte UTF-8 from the keyboard | `ppPages` + space enumerate exactly the 95 bytes `0x20..=0x7E` (26+26+10+10+9+13+1); `RuneFilter` matches only keys in that table | not producible; `ErrNotPrintableASCII` is unreachable from the screen (pinned in the package test only, as §2 rule 2 intends) |
| Bytes — `ms1` + 45 bech32 chars | `len(t) >= 48` and prefix `ms1` -> refused as ms1-shaped, matching the host's `MIN_MS1_LEN = 48` | clean |
| Bytes — 64 hex with one uppercase | `isHex` accepts `A-F`; corpus refusals row 7 is the UPPERCASE 64-hex row and is driven | clean |
| Bytes — separator strip vs the host | host strips *all* Unicode whitespace; port strips 4 ASCII forms | **M-2**, unreachable through `ValidatePhrase` |
| KDF — iteration accounting | `NewDeriver` performs U_1 and sets `done = 1`; `Step` runs while `done < total`; total PRF applications = 100,000 exactly | clean |
| KDF — `Key()` length / after `Wipe` / second `Step` | `Key()` is read before every deferred `Wipe`; the loop can only exit on `done >= total` (never `dead`, since `Wipe` is deferred), so `copy(x[:], d.Key())` can never silently copy 0 bytes | clean |
| KDF — salt as a slice | 14-byte slice through `NewDeriver`; `seal.Header`'s `[16]byte` never involved; corpus mutation `Salt = append(Salt, 0, 0)` fails 22 rows | clean (but **M-3**) |
| KDF — scheduler / starvation / watchdog | derive frame sets no wakeup; `AppendEvents` blocks to `idleTimeout` on both production platforms; screensaver branch `continue`s without breaking | **C-1** |
| KDF — button press mid-derivation | `backBtn.Clicked` polled every 500 iterations (~51 ms); a Back during the final slice is missed (the callback is not invoked when `Step` returns true) and is absorbed by the confirm screen's own Back | acceptable |
| KDF — `progress false` abandons | returns the zero `[32]byte` with `ok = false`; `hashlockPhraseRoute` `continue`s without assigning | clean |
| State — `false` from the phrase route at creation vs edit | `hashlockBackToWhichHash` -> `continue`, never `return false`; only Back at `Which hash?` returns false and deletes at creation (`composer_shape.go:269`) | clean |
| State — `Hash` assigned before HOLD | assignment is inside `if composerConfirmScreen(...)` | clean |
| State — `hashByPhrase` set, path later changed or removed | never cleared on any of five paths | **I-2** |
| State — hex route's Back | behaviour changed, claimed test absent | **I-3** |
| State — the `default` panic arm | `sel` is clamped into range on every path in `composerPickScreen` | unreachable (**N-4**) |
| State — W-7 seat-discard interplay | the phrase route touches only `Paths[idx].Hash`, never key counts or the wrapper, so `composerApplyShapeEdit`'s signature is unmoved and no seat is discarded | clean |
| Display — longest legal confirm body | the tested variant *is* the longest: 18-char digest, `hardened` (8) > `sha256` (6), `chars: 100` (3 digits max), the `-1` relation line (38 normalised) > any `matches hash N` (max 27) | clean |
| Display — production body vs tested body | two warning rows measured without `composerConfirmBody` | **M-1** |
| Display — dynamic line count | `rel == ""` only shortens the body | clean |
| Firmware — 16 KB stack | no gate reaches it; plan's recipe drops `-print-stacks` | **M-5** |
| Firmware — flash budget | no ceiling exists in the tree; +12,104 B (+0.76%) against `c4a64fc` | claim accurate |
| Firmware — TinyGo-incompatible constructs | no reflection, no maps in production code (`hashlockKbdFor` is `_test.go`), `fmt.Sprintf` only where `unlock_kdf.go` already uses it | clean |
| Gate — could lockstep pass while `DeriveHardened` is wrong? | expectations are corpus constants, never recomputed; the corpus sha256 is pinned as a literal and checked on every load; `DeriveHardened` is cross-checked against `PreimageHardened` for all 11 rows | clean for the derivation |
| Gate — could the *kind* row pass while the decoder is wrong? | `copy(d[:32])` off-by-one ships green | **C-2** |
| Gate — vendored corpus divergence | `corpusSHA256` literal vs `sha256.Sum256(raw)` on every `loadCorpus`; provenance pin names ms `cd0a60f`; verified the vendored file still matches the host's at `504ff46` | clean |

---

## Checked and clean (so a later round need not re-derive)

- Iteration accounting is exactly 100,000 PRF applications; `Step`/`Done`/`Total`/`Key`/`Wipe` semantics all hold for both call shapes.
- `ValidatePhrase`'s rule order is byte-for-byte the host's, and refusals row 13 is the row that pins it.
- `MIN_MS1_LEN = 48` and `BECH32_CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"` match `argv_guard.rs:98, 103`.
- The keyboard produces exactly the 95 printable-ASCII bytes; no multi-byte rune and no control character is typeable, and `RuneFilter` cannot introduce one.
- The plan's hard-coded plate string is byte-identical to `kind[0].ms1` (both 75 characters).
- The `-1` relation variant is the longest; the §4.5 fit row therefore measures the true worst case (headroom 186 against a margin of 80, re-measured in the gated tree for this review).
- `done*100/total` peaks at 9,950,100 on the 32-bit target — no overflow, unlike the `seal.MaxIterations` case `unlock_kdf.go:290-297` documents.
- `left := time.Duration(float64(elapsed) * float64(total-done) / float64(done))` avoids `unlockKDFLead`'s integer-truncation trap by construction.
- No numeric flash ceiling exists anywhere in the fork or its CI.
- `composerPickScreen` cannot return an out-of-range `sel`, so the label-keyed switch is total.

---

**Closing counts: 2 Critical / 4 Important / 7 Minor / 5 Nit. Not GREEN.**

C-1 is a reachable stall on real hardware that every gate this plan defines is structurally blind to, and the one gate that would catch it (Task 5 Step 1) has never been run. C-2 is a test that reports PASS for a property it does not check, with the killing constant sitting unused in the vendored corpus.
