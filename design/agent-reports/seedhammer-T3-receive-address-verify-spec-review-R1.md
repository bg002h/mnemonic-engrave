<!--
Persisted verbatim. opus-architect R1 re-dispatch of the T3 spec R0 gate, after folding R0 (0C/3I).
SPEC commit 70f885e. Reviewer agentId afe0812c79389c638. Verdict: GREEN 0C/0I. All three R0 Importants
verified CLOSED against fork source d334861 with accurate citations: I-1 (keyless panic — confirmed real:
zero-value MultisigType=Singlesig → addressAt does desc.Keys[0] with no guard → Find now guards
len(desc.Keys)==0 before desc.Keys[0] AND no longer calls Supported; inv 2.1a + keyless test); I-2
(Receive/Change error propagation; inv 2.1b + test); I-3 (no Button4 in event.go, all 3 taken → ChoiceScreen
route pinned + confirmed wireable inside the existing click branch, alloc gate undisturbed). All 6 MINORs
folded. No drift; invariant numbering 2.1/2.1a/2.1b/2.2..2.8 contiguous, all R0-tag refs resolve. Two new
doc-only MINORs: M-7 (Find has two distinct "unsupported" sentinels — new exported ErrUnsupported keyless
+ existing unexported errUnsupported wrapped; no collision, no bug) and M-8 (stale "or new button" phrase
in §5 manifest). Both non-blocking; folded into the GREEN commit. Disposition: GREEN — cleared to
implementation-plan authoring. Text below verbatim (entities un-escaped: &lt;→<, &gt;→>, &amp;→&).
-->

# R1 RE-DISPATCH — SPEC_seedhammer_T3_receive_address_verify.md (commit `70f885e`)

Reviewed the folded spec in full against fork `/scratch/code/shibboleth/seedhammer` @ `d334861` (confirmed HEAD: `d334861032d9daaba3ec2693fe62f0146a5a27ff`). Method per the R1 brief: verify each fold closed correctly against source with accurate citations, and skim the whole spec for drift. The R0-cleared facts (canonical-compare no-false-positive proof, network handling, charset coverage, gap-cost timing, alloc gate) were NOT re-derived beyond confirming the folds did not disturb them.

## Fold verification

**I-1 — keyless-descriptor panic guard — CLOSED.**
- Source: panic is real. Zero-value `bip380.MultisigType` is `Singlesig` (`bip380/bip380.go:93`, `Singlesig MultisigType = iota`). A keyless descriptor therefore hits the `Singlesig` branch in `addressAt`, which does `k := desc.Keys[0]` (`address/address.go:76`) with NO length guard → `index out of range [0]` panic. `Supported` (`address.go:28-31`) calls `Receive(desc,0)` → `addressAt` → same panic, with no recover. So the panic is genuine and `Find` must guard before BOTH `desc.Keys[0]` AND `Supported`.
- Fold: spec §4.1 places `if len(desc.Keys) == 0 { return 0,0,false, ErrUnsupported }` as the FIRST statement, before `net := desc.Keys[0].Network` and before any `Supported` call (the folded algorithm in fact no longer calls `Supported` at all — it relies on the keyless guard plus propagated `Receive` errors, which is sound and cleaner). New invariant 2.1a (spec line 27) states the panic-safety/totality contract and names the keyless headless test. Matches source. CLOSED.
- **Sentinel name `ErrUnsupported`: no collision.** The existing fork symbol is unexported `errUnsupported` (`address.go:33`); the proposed exported `ErrUnsupported` is a distinct Go identifier (export status differs → different symbol, both legally coexist in package `address`). No compile collision, no shadowing. See MINOR M-7 below for a documentation nicety (two distinct "unsupported" sentinels reachable), which is non-blocking.

**I-2 — derivation-error propagation — CLOSED.**
- Source: `Receive`/`Change` return `(string, error)` (`address.go:20-26`); a mid-scan failure (e.g. `"unsupported range path element"` at `address.go:139`, or HD-derive error at `address.go:150-152`) returns `("", err)`. A bare `"" == wantStr` would silently record a non-match.
- Fold: spec §4.1 (line 56) specifies `got, derr := Receive(desc,i); if derr != nil { return 0,0,false, derr }; if got == wantStr { return 0,i,true,nil }`, and "the same for `Change`". New invariant 2.1b (spec line 28) mandates propagation, not silent `"" == wantStr`, and names the inject-an-error test. Matches source semantics. CLOSED.

**I-3 — affordance wiring (no Button4) — CLOSED.**
- Source: `gui/event.go:28-31` defines `Button1`, `Button2`, `Button3`, then `MaxButton` — no Button4. `DescriptorScreen.Confirm` (`gui/gui.go:2348`) occupies all three: `backBtn=Button1` (2360), `addrBtn=Button2` (2361), `confirmBtn=Button3` (2362).
- Fold: spec §4.2 (line 59) now states "there is no Button4 — `Button1/2/3` all exist and `DescriptorScreen.Confirm` uses all three" and pins the ChoiceScreen route ("the only feasible route; a 'new button' is impossible"). The infeasible "new button" alternative is removed. CLOSED.
- **Wireability confirmed:** the existing `if addrBtn.Clicked(ctx) && supported { descriptorAddressFlow(...) }` branch (`gui.go:2372-2374`) is the insertion point — replacing the direct call with a `ChoiceScreen` "Show addresses"/"Verify an address" lives entirely inside the click branch, NOT in the per-frame layout (lines 2402-2406 unchanged), so the `TestAllocs` 0-alloc gate is undisturbed and the existing show-addresses affordance is preserved (it becomes one of the two choices). An adjacent `ChoiceScreen.Choose` pattern already exists in the same function (lines 2382-2387) to mirror. The `address.Supported` hoist (line 2366) is untouched. Sound.

**M-1 — unmasked public-address readout — CLOSED.** Source: `PassphraseKeyboard.Layout` masks with `*` when `!revealed` (`passphrase_keyboard.go:342-343`). Fold: spec §4.3 cites `passphrase_keyboard.go:341-344`, mandates UNMASKED readout, notes masking would break the `ExtractText` case-preservation test. CLOSED.

**M-2 — degenerate range/wildcard-less path — CLOSED.** Source: `derivePubKey` only varies by `index`/`change` via `RangeDerivation`/`WildcardDerivation` (`address.go:137-146`). Fold: §2.1 caveat + §4.1 note + §4.2 step-4 "✓ Controlled by this descriptor" phrasing. CLOSED.

**M-3 — "Verifying…" frame — CLOSED.** §4.2 step 3 mandates the frame before the synchronous `Find`. CLOSED.

**M-4 — `addrFindMaxGap` new `address`-pkg const — CLOSED.** Source: `addrMaxIndex=49` in package `gui` (`gui/address_polish.go:15`) → cycle if imported by `address`. Fold: §4.1 + §5 name `addrFindMaxGap` in package `address`. CLOSED.

**M-5 — `engraveObjectFlow` no `addressText` case — CLOSED.** Source: `engraveObjectFlow` `default: return false` (`gui.go:1881-1882`). Fold: §4.4 states it intentionally has no `addressText` case → unchanged "unknown format", preserving inv 2.6. CLOSED.

**M-6 — lifecycle wording — CLOSED.** Source: `StartScreen.Flow` defer-closes its reader before returning (`gui.go:1525-1529`). Fold: §4.2 step 2 corrected wording. CLOSED.

## Drift check
Skimmed the whole spec; no blocking drift. Invariant labels 2.1/2.1a/2.1b/2.2…2.8 contiguous + unique; all `2.x` / `R0-I*` / `R0-M*` references resolve. §4.1 realizes 2.1a/2.1b; §4.2 ChoiceScreen ↔ §4.4 no-engraveObjectFlow-case consistent; §5 manifest maps to §4. Cited line numbers spot-checked accurate (`address.go:20/24/28/35`, `gui.go:1216/2361/2366`, `passphrase_keyboard.go:182/341-344`, `address_polish.go:15`). One residual stale phrase (M-8) in §5.

## Findings
- **CRITICAL:** None.
- **IMPORTANT:** None. (I-1/I-2/I-3 all verified CLOSED against source, no regression.)
- **MINOR (non-blocking):** M-7 (`Find` exposes two distinct "unsupported" sentinels — new exported `ErrUnsupported` keyless + existing unexported `errUnsupported` wrapped via propagated `Receive` errors; no collision, no logic bug since the GUI only reaches `Find` for `Supported` descriptors; plan could note it). M-8 (§5 manifest retained a stale "or new button" phrase that §4.2 pinned away; cosmetic, manifest is "indicative; plan pins").

## Verdict
`GREEN — 0 Critical / 0 Important`

All three R0 Importants closed correctly against fork source `d334861` with accurate citations; all six MINORs folded; no new contradiction or numbering drift. The two new MINORs (M-7 sentinel-doc, M-8 stale manifest phrase) are documentation-only and non-blocking. The spec passes the R0 gate and may proceed to `IMPLEMENTATION_PLAN_seedhammer_T3_receive_address_verify.md` and its own R0.
