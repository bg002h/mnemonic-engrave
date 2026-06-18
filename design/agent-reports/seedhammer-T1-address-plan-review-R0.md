<!--
Persisted verbatim. opus-architect R0 gate of the T1 address-display implementation plan
(IMPLEMENTATION_PLAN_seedhammer_T1_address_display.md @ 75de617). Reviewer agentId a5b242d09f0a736f6.
Verdict: GREEN 0C/0I (2 minor). The reviewer materialized the plan's EXACT code in a throwaway
worktree off 384547d and RAN it: BenchmarkAllocs = 0 B/op 0 allocs/op (TestAllocs PASS), all 4 plan
tests pass verbatim, engrave path byte-identical (invariant 4), fixtures exact + valid (receive≠change
for the custom-children vector), unsupported/StyleNone path inert + non-shifting, paging/toggle/back
terminate, full suite + vet + gofmt clean, no new dependency. Minors (non-blocking): M-1 icon choice
cosmetic; M-2 add a Supported=false inertness regression test. Disposition: GREEN — proceed to
single-implementer TDD; instruct the implementer to also add the M-2 inertness test. The text below is
the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — T1 address display (plan)

**Reviewer:** opus architect (read-only, adversarial). **Plan:** `design/IMPLEMENTATION_PLAN_seedhammer_T1_address_display.md` (`75de617`). **Spec:** `design/SPEC_seedhammer_T1_address_display.md` (GREEN R1). **Base:** fork `main` `384547d`. **Go:** 1.26.4 at `/home/bcg/.local/go/bin/go`.

Method: materialized the plan's *exact* code (`gui/address_polish.go`, `gui/address_polish_test.go`, the `Confirm` import + body edit) in a detached throwaway worktree off `384547d`, built it, and ran the full suite + the 0-alloc gate + targeted probes, then removed the worktree. The real fork tree was never modified (verified clean at `384547d`).

## Verification Results

**Build:** `go build ./gui/... ./address/...` → exit 0 (clean compile of the plan's verbatim code).

**1 — The 0-alloc gate (THE risk): GREEN.**
```
BenchmarkAllocs-24    45122    35329 ns/op    0 B/op    0 allocs/op
=== RUN   TestAllocs --- PASS: TestAllocs (1.11s)
```
- (a) **Confirmed.** `BenchmarkAllocs` (`gui_test.go:50-91`) drives `ds.Confirm` through `iter.Pull`: the iterator body calls `s(ctx)` (i.e. `Confirm`) **exactly once**; each `next()` resumes to the next `ctx.Frame` callback. The hoisted `supported := address.Supported(s.Descriptor)` (allocating secp256k1 derivation) runs in that one-time pre-loop entry, **outside** the measured `b.Loop()`, so it amortizes to 0/op.
- (b) **Confirmed, with measurement.** The per-frame body — three `Clickable.Clicked` + the **fixed 3-element** non-escaping `[]NavButton{...}` literal + `layoutNavigation` + `s.Draw` + `ctx.Frame` — measures **0 B/op, 0 allocs/op**. The benchmark descriptor is a supported P2WSH sortedmulti, so `supported=true`: the styled (StyleSecondary) 3rd-button path is the one exercised, and it stays 0-alloc. The 3rd `.Clicked` and the 3-element-vs-2-element literal introduce no per-frame allocation. `layoutNavigation` only ranges over `btns` and never stores the slice (gui.go:1788), so the literal is stack-allocatable.

**2 — Compile + behavior: GREEN.** All four plan tests pass verbatim:
```
--- PASS: TestDescriptorAddressFlowRendersReceive
--- PASS: TestDescriptorAddressFlowToggleChange
--- PASS: TestDescriptorAddressFlowBackExits
--- PASS: TestDescriptorConfirmAddressAffordance
```
- Signatures all correct: `showError(ctx, th, "Address", err.Error())` matches `slip39_polish.go:22 showError(ctx,th,title,msg string)`; `runUI` returns `(func()(string,bool), func())` (gui_test.go:467); `uiContains` (gui_test.go:480); `layoutTitle(ctx, dims.X, th.Text, title)` matches gui.go:1694 `(ctx,width,col color.RGBA,title)` (`th.Text` is `color.RGBA`, theme.go:32); `widget.Labelw(&ctx.B, ctx.Styles.body, dims.X-2*8, th.Text, ln)` matches label.go:16; `layout.Rectangle.CutTop/CutBottom` return `(top,bottom)` (layout.go:96/101). The render block is byte-pattern-identical to the proven `confirmCodex32Flow` (codex32_polish.go:129-137) and `slip39_polish.go:128`.
- **Fixtures are exact + valid.** `tvXpub` == `address_test.go:11` `xpubs[0]`; `descWPKH` == row `:26`; `descCustomChildren` == row `:46`. `nonstandard.OutputDescriptor([]byte(descStr))` accepts both — this is the *same* constructor the on-device NFC path uses (`gui/scan.go:66`) and `address_test.go:62`. Probe confirmed receive[0]=`bc1qt77623mm…` and change[0]=`bc1qc8gz4sw…` match `address_test.go:47-48` and are **distinct** — so the toggle test genuinely distinguishes receive≠change (not idx0-vs-idx0).

**3 — Wiring correctness: GREEN.** `git diff gui/gui.go` confirms the engrave branch (`confirmBtn.Clicked` → `validateDescriptor` → `showErr`/`ChoiceScreen` → `return e, true`) is **byte-identical**; the `(Plate,bool)` contract and trailing `return Plate{}, false` are unchanged (invariant 4 holds; `descriptorFlow` at gui.go:2018 consumes `(plate,ok)` unaffected). Button2 was genuinely free on `Confirm` (only Button1/Button3 bound before) and is drained every frame via `addrBtn.Clicked(ctx)`, with the action gated by `&& supported` (queue-head idiom — `Clicked` loops `Next` to drain). A probe with `Supported=false` (Script 99) confirmed Button2 is **inert** on an unsupported descriptor (no crash, address view never opens). `descriptorAddressFlow` matches the spec: Button1 back / Button2 toggle (`start=0` reset, verified) / Button3 page (`start += 5`, cap `start+5 ≤ 50`), recompute-on-event-only, terminates on Back (verified including Back-after-toggle, no hang), never engraves/mutates. The `showError`-mid-`recompute`-then-`return` pattern is sound: `showError` is a self-contained modal that returns on dismiss (slip39_polish.go:22), so calling it then returning from the flow is non-re-entrant.

**4 — layoutNavigation with 3 buttons + StyleNone: GREEN.** Button position is keyed by `ys[idx]` where `idx := int(clk.Button - Button1)` (gui.go:1790-1793) — Button1→top, Button2→middle, Button3→bottom — **independent of slice order or whether earlier buttons rendered**. A `StyleNone` middle button returns the empty `op.Op{}` (gui.go:1726) and cannot shift back/confirm. The 3-button-nav precedent (`confirmCodex32Flow`, codex32_polish.go:116-123) renders the same way.

**5 — Scope / no regression: GREEN.** Full suite clean with only the plan's code:
```
ok  seedhammer.com/gui    (all pre-existing tests pass)
ok  seedhammer.com/address
go vet → exit 0 (only the pre-existing gui/op/draw_test.go go1.26 note, not ours)
gofmt -l gui/address_polish.go gui/address_polish_test.go → silent
```
No new dependency (`address` already imports the btcsuite/decred deps in `go.mod`); display-only; descriptor case only; single-sig + sortedmulti + paging — exactly the GREEN spec.

## Findings

**CRITICAL:** none.
**IMPORTANT:** none.

**MINOR (non-blocking, for the implementer — do not gate on these):**
- **M-1 (cosmetic, plan Task 1 Step 3 / spec §4.1):** The plan uses `assets.IconEdit` for the toggle button in `descriptorAddressFlow` and `assets.IconInfo` for the affordance, whereas spec §4.1 suggests `IconRight`/`IconInfo`. All three icons exist (embed.go) and tests pass; icon choice is a UX detail, not a correctness issue. No change required.
- **M-2 (test-strength, plan Task 2 Step 1):** `TestDescriptorConfirmAddressAffordance` asserts only the *supported* path opens the view; the plan's parenthetical leaves the unsupported-inertness assertion optional. My probe (Script 99) confirmed inertness holds, but the committed test suite does not lock it in. Optional: the implementer may add a `Supported=false` inertness case for regression protection. Not required for GREEN.

## Conclusion

The plan's exact code compiles, all four plan tests pass verbatim, the engrave path is byte-identical, the unsupported/StyleNone path is inert and non-shifting, paging/toggle/back all terminate correctly, and — the load-bearing check — `BenchmarkAllocs` measures **0 B/op, 0 allocs/op** with the modified `Confirm`, keeping `TestAllocs` green. No per-frame alloc, no compile error, no engrave regression, no hang.

**GREEN — 0 Critical / 0 Important.**
