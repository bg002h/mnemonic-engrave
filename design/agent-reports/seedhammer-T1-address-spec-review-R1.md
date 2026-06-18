<!--
Persisted verbatim. opus-architect R1 gate of the T1 address-display spec after folding R0
(SPEC_seedhammer_T1_address_display.md @ d7fc96a). Reviewer agentId a1bf819d54533cc1f.
Verdict: GREEN 0C/0I (2 minor, plan-level). The R0 0-alloc blocker is closed: Supported is now
unambiguously required hoisted out of Confirm's frame loop; the address list is a separate
non-benchmarked screen; TestAllocs/BenchmarkAllocs added as a must-stay-green gate. Key verified
nuance: adding a 3rd nav button to Confirm does NOT itself allocate per-frame — layoutNavigation
(gui.go:1723,1788-1800) only ranges over btns and never stores it, so a fixed 3-element composite
literal is stack-allocatable exactly like today's 2-element one. M-1 paging + M-2 custom-children
match source exactly; no drift. The 2 MINORs are plan-steering: MINOR-1 — do NOT copy codex32's
append-chain nav idiom (codex32's confirm isn't alloc-gated; DescriptorScreen.Confirm IS); use a
fixed literal + StyleNone for the conditional 3rd button. MINOR-2 — name the StyleNone mechanism.
Disposition: folded MINOR-1 into spec §4.1 (replaced the misleading codex32-append pointer with the
fixed-literal+StyleNone construction); proceed to the plan. The text below is the agent's report
exactly as returned; do not edit.
-->

# R1 GATE REVIEW — T1 address display (spec)

**Reviewer:** opus architect (adversarial, read-only). **Spec:** `design/SPEC_seedhammer_T1_address_display.md` @ `d7fc96a` (folded). **Prior R0:** `design/agent-reports/seedhammer-T1-address-spec-review-R0.md` (NOT GREEN, 0C/1I+2 minor). **Authoritative base:** fork `main` `384547d` at `/scratch/code/shibboleth/seedhammer` (HEAD confirmed `384547d`). No Go toolchain in env — alloc reasoning is from source + the (R0-verified, still-passing) `TestAllocs` gate, not a fresh benchmark run.

## Verification Results

### I-1 (the R0 blocker) — CLOSED, with one residual implementation hazard the plan must pin (see MINOR-1)

**(a) The gate is real and drives `Confirm`.** `gui/gui_test.go:50-91` `BenchmarkAllocs` runs exactly two screen frame-funcs — `m.Flow` (StartScreen) and **`ds.Confirm(ctx, &descriptorTheme)`** (`:68-70`) — frame-stepped via `iter.Pull`; `TestAllocs` (`:93-98`) fails on `AllocsPerOp() > 0`. Grep confirms these are the *only* benchmarked screens. So `DescriptorScreen.Confirm`'s frame loop is a hard 0-alloc path; the new `descriptorAddressFlow` is **not** in the set. The fold's "keep the address list a SEPARATE screen so its per-index `Receive/Change` allocations are off the benchmarked path" is sound by construction.

**(b) The hoist is unambiguous and feasible.** Folded invariant 2 (§2:29), invariant 6 (§2:30), §4.1 (:49), and §6 (:82) all now state `Supported` MUST be computed once — cached on the `DescriptorScreen` struct or before the `for !ctx.Done` loop — and never inside the frame loop. `DescriptorScreen` is a plain struct (`gui/gui.go:2306-2308`) trivially extensible with a `supported bool` field; the descriptor is already assigned there. `Supported`→`Receive(desc,0)` (secp256k1 derivation + formatting, `address/address.go:24-31, 116-157`) runs exactly once. Mechanism is correct and the wording leaves no per-frame reading open. **This closes the R0 Important.**

**(c) Does adding a 3rd nav button itself allocate per-frame? NO — *if built as a fixed composite literal* (the current pattern), YES if built via the codex32 append pattern.** This is the load-bearing check the prompt flagged, and the answer is nuanced:
- The existing 2-button nav in `Confirm` is `layoutNavigation(&ctx.B, th, dims, []NavButton{ {…back…}, {…confirm…} }...)` (`gui/gui.go:2347-2350`). `layoutNavigation(buf, th, dims, btns ...NavButton)` (`:1723`) only **ranges** over `btns` (`:1788-1800`) and never stores it — `btns` does not escape. So the variadic backing array of a *fixed composite literal* is stack-allocatable; that is precisely why the current 2-element call is 0-alloc under the live `TestAllocs` gate. Extending the fixed literal to **3 elements is still a non-escaping fixed-size composite literal → still 0-alloc.** Adding a 3rd nav button does **not**, by itself, allocate per-frame. **Not a residual Critical.**
- The 0-alloc-safe way to make the 3rd button *conditional* is to keep a fixed 3-element literal and set the address button's `Style = StyleNone` when `!supported`; `layoutNavigation`'s `button()` returns the zero `op.Op{}` for `StyleNone` (`:1726-1728`) — drawn empty, no alloc, and its `Clickable` is still constructed and can be drained every frame (satisfying the queue-head-block idiom, §4.1:49).
- **The hazard:** §4.1 tells the implementer to "mirror the gated-Button2 idiom of `confirmCodex32Flow`/…/seedxor". But that cited precedent (`gui/codex32_polish.go:116-123`) builds nav via `navBtns := []NavButton{…}; navBtns = append(…)` — an append chain that **can heap-allocate** and would break the gate. Codex32 gets away with it because its `Confirm` is **not** in the benchmarked set; `DescriptorScreen.Confirm` **is**. Copying codex32's slice-building verbatim into the alloc-gated `Confirm` would regress `TestAllocs`. The spec requires the *invariant* (and §6 makes `TestAllocs` a must-stay-green gate that would catch it red in TDD), so this is not a spec-level Critical — but the precedent it points at demonstrates the wrong construction for an alloc-gated screen, so the **plan must pin the fixed-literal + `StyleNone` approach**. Recorded as MINOR-1.

### M-1 (paging) — CORRECT and CONSISTENT
§4.2 (:55) pins Button1=Back, Button2=receive⇄change, **Button3=page-forward**, window `start..start+4`, `start+=5`, hard cap `start+5 ≤ 50`. Verified:
- **Button3 genuinely free on the address-list screen.** Button3 is used for Confirm/engrave only on `DescriptorScreen.Confirm` (`gui/gui.go:2323`); the address-list is a *different* screen, so Button3 is unbound there. Correct.
- **No auto-repeat.** `Clickable.Next` only applies repeat logic for `Up, Down, Right, Left` (`gui/widget.go:50-68`); `Button1/2/3` and `Center` are discrete. The spec's "avoid Up/Down Clickables (they auto-repeat), use discrete Button3" is exactly right against source.
- **Deterministic window/cap.** `start..start+4` with `start+=5` and `start+5 ≤ 50` is a bounded, testable progression.

### M-2 (custom-children test) — CORRECT and matches source exactly
§6 (:79) requires a custom-children descriptor so receive≠change, citing the `/1234/<5;6>/*` vector and noting default keys use `<0;1>/*` → receive=branch-0, change=branch-1. Verified:
- `derivePubKey` default-children: when `len(children)==0`, defaults to `<0;1>/*` (`address/address.go:118-130`); `RangeDerivation` uses `Index` for receive and `End` for change (`:137-144`) → receive=branch0, change=branch1. Exact.
- The `/1234/<5;6>/*` vector exists at `address/address_test.go:46-49` with distinct receive vs change golden addresses. Mirroring it makes the toggle test genuinely distinguish receive from change (not index0-vs-index0). Correct.

### Drift hunt — NONE
- **Button2 semantics across screens are NOT in conflict.** §4.1 (:49): on `DescriptorScreen.Confirm`, Button2 = "open address view" affordance. §4.2 (:55): on the address-*list* screen, Button2 = receive⇄change toggle. These are two **different** screens with independent button bindings — no contradiction. (The list screen is entered *from* Confirm via Button2, then has its own loop/bindings.)
- Cross-references intact: invariants 2↔6 reference each other consistently; §6's 0-alloc bullet references "R0 IMPORTANT-1" and matches `gui_test.go:50-98`; file manifest (§5) matches §4.3 wiring.
- No scope creep: still display-only, single confirm-screen affordance + one new flow + tests. Out-of-scope items (mk1/md1-native, T3 verification, engraving addresses) unchanged.

### Spot-confirm of R0-verified claims (unchanged) — STILL HOLD
- API signatures `Receive`/`Change`/`Supported`/`addressAt`/`derivePubKey` and the supported-type set unchanged in `address/address.go`.
- Wiring point `engraveObjectFlow case *bip380.Descriptor:` → `descriptorFlow` → `DescriptorScreen.Confirm`, Button2 free on Confirm (`gui/gui.go:2322-2323`), `(Plate,bool)` contract preservable.
- Reachability: NFC scan is the only on-device producer; **no `bip380.Parse` callsites in `gui/`** (grep empty) — T2 deferral correct.
- Footprint: `seedhammer.com/address` imported **nowhere** outside its own package (grep empty) — T1 is the first importer; gui→address acyclic; zero new module deps. No pre-existing `address_polish*` files and no `feat/address-display` worktree (clean pre-implementation state).

## Findings

### MINOR-1 — §4.1's cited nav-button precedent (codex32) uses an append chain that would break the 0-alloc gate on `Confirm`; the plan must pin the fixed-literal + `StyleNone` construction.
**Where:** §4.1 (:49) "mirror the gated-Button2 idiom of `confirmCodex32Flow`/…". **Evidence:** `gui/codex32_polish.go:116-123` builds nav via `[]NavButton{…}` + `append` (heap-allocatable); codex32 is *not* in `BenchmarkAllocs`, but `DescriptorScreen.Confirm` *is* (`gui/gui_test.go:68-70`). The 0-alloc-safe construction is a fixed 3-element composite literal (non-escaping → stack) with the address button's `Style=StyleNone` when `!supported` (rendered empty per `gui/gui.go:1726-1728`, its `Clickable` still drained). **Not a spec blocker** — the invariant is correctly required (inv. 2/6, §6) and `TestAllocs` would catch an append-based impl red in TDD — but the plan should explicitly steer to the fixed-literal/`StyleNone` pattern, not the cited append idiom, to avoid a predictable red-then-rework loop on a gated screen.

### MINOR-2 — Optional clarity: name the `StyleNone` mechanism for the hidden affordance.
§4.1 says the affordance is "shown only when Supported is true" and "Button2 is drained every frame even when hidden." The cleanest realization (fixed literal, `Style` toggled to `StyleNone` when not supported, Clickable always constructed+drained) is implied but not named. A one-line mention would remove ambiguity. Cosmetic; not blocking.

*(No CRITICAL: adding a 3rd nav button does not allocate per-frame when built as a fixed composite literal — the existing 2-button literal is 0-alloc precisely because `btns` doesn't escape `layoutNavigation`, and a 3-element literal inherits that. No Important: the R0 blocker (per-frame `Supported`) is closed by an unambiguous, feasible hoist; M-1/M-2 match source exactly; no drift.)*

---

**GREEN — 0 Critical / 0 Important.**

The fold closes R0 IMPORTANT-1 cleanly: `Supported` is now unambiguously required to be hoisted out of `Confirm`'s frame loop (cached on `DescriptorScreen` or pre-loop), the address list stays a separate, non-benchmarked screen, and `TestAllocs`/`BenchmarkAllocs` are added as a must-stay-green gate (§6). Critically, adding a 3rd nav button to `Confirm` does **not** itself allocate per-frame — `layoutNavigation` never lets `btns` escape (`gui/gui.go:1723,1788-1800`), so a fixed 3-element composite literal is as stack-allocatable as today's 2-element one; that hazard is not residual. M-1 paging (Button3 free on the list screen, discrete no-auto-repeat, deterministic `start+5≤50` cap) and M-2 (custom-children `/1234/<5;6>/*` so receive≠change) both match source exactly. No drift, no broken cross-references, no scope creep. The two MINORs are plan-level steering (use a fixed literal + `StyleNone`, not codex32's append chain, on this alloc-gated screen) — non-blocking. Clear to proceed to the plan / plan-R0 loop.
