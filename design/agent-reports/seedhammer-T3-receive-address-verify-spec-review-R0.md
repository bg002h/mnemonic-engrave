<!--
Persisted verbatim. opus-architect R0 GATE review of SPEC_seedhammer_T3_receive_address_verify.md
(commit 8c76661) BEFORE code. Reviewer agentId a2b991453855d6c22. Verdict: NOT GREEN — 0C/3I. Strong
empirical verification (DecodeAddress round-trip identical to address.go encoder → no false-positive
hole; network handling safe; PassphraseKeyboard a-z/A-Z/0-9 pages cover the full bech32+base58 charset;
measured 20x2 3-of-3 multisig scan ~17ms host → seconds on RP2350, acceptable one-shot). 3 Importants
(all in Find's headless contract + GUI wiring): I-1 Find/Supported PANIC on a keyless descriptor (no
length guard in addressAt) → must guard len(desc.Keys)==0 first; I-2 Find must propagate Receive/Change
errors (not let ""==wantStr silently be a non-match); I-3 affordance offered a "new button" but no Button4
exists + all 3 taken → pin the ChoiceScreen route. 6 MINORs: M-1 unmask the public-address readout
(PassphraseKeyboard masks); M-2 wildcard-less descriptor → degenerate (0,0); M-3 no "Verifying…" frame
before the sync scan; M-4 gap cap must be a new `address`-pkg const (addrMaxIndex is in gui, cycle); M-5
state engraveObjectFlow has NO addressText case; M-6 fix lifecycle wording. Disposition: fold I-1/I-2/I-3
+ MINORs → re-dispatch R1. Text below verbatim (entities un-escaped: &lt;→<, &gt;→>, &amp;→&).
-->

# R0 GATE REVIEW — SPEC_seedhammer_T3_receive_address_verify.md

Reviewed at spec commit `8c76661`, verified against fork `/scratch/code/shibboleth/seedhammer` @ `d334861` (confirmed HEAD). All probes run against live source; baseline `go test ./address/ ./gui/` is GREEN.

## Verification Results (per numbered item)

**1. `address` pkg feasibility — CONFIRMED with one structural caveat.**
- `Receive(desc,i)(string,error)` (address.go:24), `Change` (:20), `Supported` (:28), `addressAt` (:35) all confirmed; addresses are plain strings, `==` comparable. `addressAt` emits P2PKH/P2SH/P2WPKH/P2WSH/P2TR (+ wrapped P2SH-P2WPKH/P2SH-P2WSH), driven by `(index, change)`. `Find` as a Receive/Change string-compare loop is feasible.
- **Receive/change convention (the recon's `address.go:137-146`):** CONFIRMED. `derivePubKey` maps `change=false→RangeDerivation.Index`, `change=true→RangeDerivation.End`, with the guard `End == Index+1` (line 138). A bare `wpkh(xpub)` defaults to `<0;1>/*` (lines 119-130), so receive=.../0/i, change=.../1/i. Test vector `<5;6>` confirms receive=5/change=6.
- **DISCREPANCY (subtle, real):** A descriptor whose path has **no RangeDerivation and no WildcardDerivation** (e.g. `wpkh(xpub/0/0)`) is `Supported` and derivable, but `Receive(i)`/`Change(i)` IGNORE both `index` and `change` — every index/chain yields the SAME address (empirically confirmed). `Find`'s reported `(chain,index)` is then degenerate (always `(0,0)`). Not a false positive (the address IS controlled), but invariant 2.1's "for some idx" claim hides this; the spec never addresses the wildcard-less-with-keys case (only "no keys" is scoped out).

**2. Canonical compare soundness — CONFIRMED (no false-positive hole found).**
- `btcd/address/v2.DecodeAddress(addr, *chaincfg.Params)` exists (go.mod:6; address.go:148). Both the fork's `addressAt` (`addr.String()`) and `DecodeAddress(...).String()` route through the IDENTICAL `EncodeAddress`/`encodeSegWitAddress`/`encodeAddress` code in the same package. **Empirically verified**: `DecodeAddress(Receive/Change(...)).String() == derived` for ALL emitted types (P2PKH/P2WPKH/P2SH-P2WPKH/P2TR/P2WSH-multi/P2SH-P2WSH-multi/P2SH-multi). No casing/checksum/witness-version drift → no false negative from encoding mismatch.
- **Network handling — CONFIRMED safe.** Empirically: mainnet bech32 + testnet param → decodes but `IsForNet(testnet)=false` (caught by spec's `!IsForNet` guard → ErrAddrWrongNetwork); mainnet base58 + testnet param → `DecodeAddress` errors directly (→ ErrAddrUnparseable). Both sides of the compare always use the descriptor's own net, so no cross-network false POSITIVE is possible. Mixed-network multisig is rejected by `addressAt` (lines 46-48) → `Supported=false`.
- Minor: a *wrong-network base58* surfaces as "Invalid address" while a wrong-network *bech32* surfaces as "Different network" (asymmetric `DecodeAddress` behavior). Cosmetic only.

**3. Case-preserving keyboard — CONFIRMED.**
- `Keyboard.rune()` force-uppercases: `k.Fragment + string(unicode.ToUpper(r))` (gui.go:1216). `PassphraseKeyboard.commit` preserves case: `k.Fragment += string(key.r) // NO ToUpper` (passphrase_keyboard.go:182). Building on `PassphraseKeyboard` is the right base.
- **Charset CONFIRMED complete:** empirically, the pp pages (a-z, A-Z, 0-9) cover the ENTIRE bech32 + base58 character set — 0 missing chars. "No charset restriction; DecodeAddress validates" is sound: a single base58 typo hits a 32-bit checksum (collision into a *different valid in-range address* is cryptographically implausible); bech32 has BCH error-detection. No misleading-match risk.

**4. Alloc gate — CONFIRMED.**
- `BenchmarkAllocs`/`TestAllocs` (gui_test.go:50-98) covers exactly `StartScreen.Flow` + `DescriptorScreen.Confirm`. `address.Supported` hoist confirmed at gui.go:2366 (spec's `~2366` exact). `Find`/`Receive`/`Change` run in the verify sub-flow's own loop (post-click), not the benchmarked path. A Button2→ChoiceScreen change allocates only inside the click branch, not per-frame. The hoist is the right precedent. The verify/keyboard screens are correctly not alloc-gated.

**5. Scanner-shell + affordance wiring — CONFIRMED (one infeasible alternative flagged).**
- `mk1GatherFlow` (mk1_inspect.go:156-256) is a reusable scanner-shell (own NFCReader + goroutine, `NFCReader()!=nil` gate, type-filtered `scans` channel, Back exit). Lifecycle CONFIRMED: `uiFlow→StartScreen.Flow` (defer-closes its reader, gui.go:1525-1529) returns BEFORE `engraveObjectFlow→descriptorFlow→DescriptorScreen.Confirm` runs → a fresh `NFCReader()` in the verify shell is safe (no two readers open). Adding a `DecodeAddress` branch to `scanner.Scan` (after existing probes) is sound; the shell type-asserts the new value (as `mk1GatherFlow` asserts `mdmkText`), so routing to verify (not `engraveObjectFlow`) works.
- **DISCREPANCY:** only `Button1/2/3` exist (event.go:21-31) — there is **no Button4**. `DescriptorScreen.Confirm` already uses all three (Back/addresses/Confirm, gui.go:2402-2406). The spec's affordance resolution offers "a ChoiceScreen OR the verify gets its own affordance (new button)" — the **"new button" alternative is physically infeasible**; the ChoiceScreen route (Button2 → "Show addresses"/"Verify") is effectively forced. The spec defers "the exact wiring" to the plan while presenting an impossible option as live.

**6. Gap limit + cost — CONFIRMED acceptable.**
- 20/chain, cap ≤49, bounded-with-clear-no-match: sound. **Measured** (host i7): full 20×2 3-of-3 multisig scan = ~17.3 ms; single singlesig Receive = ~0.37 ms. On RP2350 (Cortex-M33 @150 MHz, TinyGo, software `decred/secp256k1/v4`) this is ~100-1000× slower → worst-case seconds. Acceptable as a one-shot behind an explicit action (`Find` early-exits on first match). No TinyGo build-constraint issue in `address`/`bip380` (none present; same secp256k1 lib used by the working engrave path).
- **Gap UX gap (Minor):** `Find` runs synchronously with no frame rendered during the scan → UI freezes for seconds with no "Verifying…" feedback. The spec ("render after") does not mandate a pre-scan progress frame.
- **Cross-package const (Minor):** `addrMaxIndex=49` lives in package `gui` (address_polish.go:15); `address.Find` CANNOT import it (would cycle). The plan must define a separate cap constant in package `address`.

**7. Testability — CONFIRMED, with one keyboard-readout caveat.**
- `runUI`/`uiContains`/`ExtractText`/`click`/`press` all exist; `testPlatform.NFCReader()==nil` (gui_test.go:408). `Find` headless, result-flow via direct candidate string, scan recognizer headless, NFC routing code-reviewed — all feasible.
- **Caveat:** `PassphraseKeyboard.Layout` MASKS the Fragment (`*`×len unless `revealed`, passphrase_keyboard.go:341-344). For a PUBLIC address this is wrong UX AND breaks the `ExtractText`-based case-preservation assertion (readout shows `***`). The address keyboard must render an UNMASKED readout (default `revealed=true` or a dedicated variant); tests can read `kbd.Fragment` in-package as a fallback.

## Findings

### CRITICAL
None.

### IMPORTANT

**I-1 — `Find`'s stated error contract is unachievable as written; keyless descriptor PANICS.** §4.1 has `Find` derive `net := desc.Keys[0].Network` as step 1 and documents "err if the descriptor is unsupported." **Empirically, `Supported(keyless)` itself PANICS** (`index out of range [0]` — `addressAt` does `desc.Keys[0]` / iterates `desc.Keys` with no length guard, and `Receive` doesn't recover). A keyless descriptor (the recon's "template-only md1") passed to `Find` panics at `desc.Keys[0].Network` before any error can be returned. The GUI is safe (only `Supported` scanned descriptors reach it), but `Find`'s public contract claims robustness it doesn't have. **Fix:** `Find` MUST first `if len(desc.Keys) == 0 { return 0,0,false, ErrUnsupported }` BEFORE touching `desc.Keys[0]` AND before calling `Supported` (since `Supported` also panics on keyless). Add a headless test passing a keyless descriptor → expects the error, not a panic.

**I-2 — `Find` must propagate `Receive`/`Change` errors, not silently treat them as non-matches.** §4.1's algorithm shows `if Receive(desc,i) == wantStr` with no handling of `Receive`'s `error` return. If a derivation errors mid-scan (mixed-net multisig that slipped past, unsupported range element, HD-derive failure), `Receive` returns `("", err)`; comparing `"" == wantStr` silently records a non-match and continues, masking a real failure (false-negative + swallowed error). **Fix:** spec the loop as `got, err := Receive(desc,i); if err != nil { return 0,0,false, err }; if got == wantStr {...}` (same for `Change`). Add an invariant + test.

**I-3 — affordance wiring presents an infeasible option.** §4.2 offers "ChoiceScreen OR a new button" as live alternatives, but no Button4 exists (event.go) and all three buttons are occupied in `DescriptorScreen.Confirm`. **Fix:** pin the ChoiceScreen route (Button2 → "Show addresses"/"Verify an address") in the spec, or explicitly state Center-as-AltButton is the only other option; remove the infeasible "new button" alternative so the plan isn't seeded with a dead end.

### MINOR

**M-1 — Masked readout for a public value.** Building the address keyboard on `PassphraseKeyboard` inherits `*`-masking (passphrase_keyboard.go:341-344). Spec an unmasked address readout (default `revealed=true` or a variant); also unblocks the `ExtractText` case-preservation test.

**M-2 — Wildcard-less/range-less descriptor not addressed.** A `Supported` descriptor with only `ChildDerivation` path elements (e.g. `wpkh(xpub/0/0)`) makes `Find`'s `(chain,index)` degenerate (always `(0,0)`; index/chain ignored). Not a safety hole, but add an invariant note (and ideally a test) so the reported chain/index semantics are honest. Consider noting it in §2.1.

**M-3 — No "Verifying…" frame before the synchronous scan.** A multisig 40-set scan blocks for seconds with a frozen UI. §4.2 should mandate rendering a progress/"Verifying…" frame before the blocking `Find` call.

**M-4 — `Find`'s gap cap is a new `address`-package constant.** `addrMaxIndex` is in package `gui` and cannot be imported by `address` (cycle). Spec/plan should name the new cap constant in `address` (the §3 reference to "the addrMaxIndex=49 precedent" is fine as rationale only).

**M-5 — Top-level address scan behavior should be stated.** Adding `DecodeAddress` to the shared `scanner.Scan` means a bare address scanned at StartScreen now becomes `addressText`; `engraveObjectFlow` has no such case → `default: return false` → "unknown format" (same user-visible outcome as today). Spec should explicitly state `engraveObjectFlow` intentionally has NO `addressText` case, to preserve invariant 2.6 and pre-empt a "missing case" reviewer flag.

**M-6 — §4.2 lifecycle wording.** "safe — DescriptorScreen returned before this runs" is mis-stated; the correct reason is "StartScreen.Flow returned and closed its reader before DescriptorScreen ran." Conclusion (no concurrent readers) is correct; fix the wording.

## Invariant assessment
The 8 invariants are largely the right ones and individually verifiable. Gaps: (a) **no invariant that `Find` is panic-safe / total** for keyless or unsupported descriptors (I-1); (b) **no invariant on `Receive`/`Change` error propagation** (I-2); (c) the receive/change-convention and the **wildcard-less degenerate** case are not captured (M-2). 2.1's "no false positive" is empirically supported; 2.2's network safety is empirically supported. Scope (Phase A `Find`+result, Phase B inputs) is coherent; ~550 LOC is realistic. Citations in §3 all verified accurate (address.go:20/24/28/35, gui.go:1216/2052/2366, passphrase_keyboard.go:182, scan.go:66, address_polish.go:26).

## Verdict

`NOT GREEN — 0C/3I`

The match logic is sound (no false-positive hole found; canonical compare and network handling empirically verified). The three Important findings are all in `Find`'s headless contract and the GUI wiring — fix I-1 (panic-safe/keyless guard), I-2 (error propagation), and I-3 (drop the infeasible button alternative; pin ChoiceScreen), fold the Minors, persist this review, and re-dispatch.
