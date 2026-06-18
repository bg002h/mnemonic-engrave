<!--
Persisted verbatim. opus-architect R0 gate of the Seed XOR combine spec
(SPEC_seedhammer_seedxor.md @ ad8dcba). Reviewer agentId a7b741015a6009517. Verdict: NOT GREEN
0C/1I+2m. I1: spec specified a non-existent inputWordsFlow(title,bool) signature (conflated with
SLIP-39/codex32) + a false panic-safety invariant — real inputWordsFlow is pre-sized, returns a
partial slice on Back, so Entropy() could panic with no interposed validation. M1: the Coldcard
phrase vectors aren't a copyable artifact (re-derive from authoritative source). M2: order-indep
test is authored fresh. Everything else VERIFIED (pre-sized-entry reconciliation correct; gate
mandatory+drained; {16,24,32} guard load-bearing; no-ambiguity; dispatch; port faithful).
Disposition: folded. The text below is the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — Seed XOR combine (spec)

**Reviewer:** opus architect (adversarial R0 gate, read-only)
**Spec under review:** `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_seedxor.md` (commit `ad8dcba`)
**Base / ground-truth commits:** SeedHammer fork `main` `bc63caa` (verified: `git rev-parse HEAD` = `bc63caa814e20d2…`, "Merge feat/slip39-trezor-routing"); toolkit port source `mnemonic_toolkit::seed_xor`; consult `seedhammer-seedxor-design-consult.md`; recon `cycle-prep-recon-seedxor.md` (`4b3b8db`/`4affb17`).
**Date:** 2026-06-18

---

## Verification Results

Each load-bearing claim was checked against real source (not just the draft), per the per-phase external-fact rule.

**1. Pre-sized entry → part-length pick is mechanically necessary — TRUE (the central reconciliation holds).**
`inputWordsFlow(ctx, th, mnemonic bip39.Mnemonic, selected int)` (gui.go:580) fills the caller-supplied fixed-length slice and returns *only* when full: `selected++; if selected == len(mnemonic) { return }` (gui.go:644–646). The title is hardcoded `"Word %d of %d"` over `len(mnemonic)` (gui.go:701). `emptyBIP39Mnemonic(nwords)` (gui.go:552) allocates a fixed `make(bip39.Mnemonic, nwords)`. The existing 12/24 menu must know the length up front: `emptyBIP39Mnemonic([]int{12, 24}[choice])` (gui.go:2024). There is **no variable-length / "done-early" / "add-word" affordance.** This is directly corroborated by the sibling `slip39LengthPick` whose own comment states the same rationale: *"inputSLIP39Flow fills a pre-sized slice, so the length must be known at allocation"* (slip39_polish.go:38–39). **The architect's "no length picker, first part fixes L" was premised on a variable-length entry path that does not exist; the spec's reconciliation (add the pick, mechanically necessary) is correct.** The added 18-word option is a justified Coldcard-interop superset of the current `{12,24}` menu — `inputWordsFlow`/`emptyBIP39Mnemonic` are word-count-generic, so 18 works.

**2. The fingerprint gate (safety crux) is unskippable on the only engrave path — TRUE.**
The spec's `combineSeedXORFlow` (§4.2) returns `(seed, true)` only at step 6, after the mandatory step-5 `confirmSeedXORFingerprint` (Back → `(nil,false)`). The dispatch then routes the mnemonic to `backupWalletFlow`. This mirrors the verified SLIP-39 BIP-39 arm where `if !confirmSLIP39Fingerprint(...) { return false }` precedes `backupWalletFlow(ctx, th, m)` (slip39_polish.go:423–426). The gate template `confirmSLIP39Fingerprint(ctx, th, mfp uint32) bool` (slip39_polish.go:433) has Button1=Back→false, Button3/Center=Engrave→true, and the unconditional `drainBtn.Clicked(ctx) // drain Button2 (no-hang)` (slip39_polish.go:445). The spec faithfully clones this with stronger "no built-in check" wording and the Button2-drain (§4.3, invariant §2.5). No path delivers a recovered seed to `backupWalletFlow` without the gate.

**3. `Combine` is a faithful port + the interop guard is load-bearing — TRUE.**
`seed_xor_combine` (seed_xor.rs:161) = `validate_share_count(>=2)`, equal-length check, `validate_entropy_len`, byte-wise XOR fold. The spec's algorithm (§4.1) matches. The guard is load-bearing: `bip39.New` accepts any 16–32 B mult-of-4 (`if len(entropy) < 16 || 32 < len(entropy) { panic }; if len(entropy)%4 != 0 { panic }`, bip39.go:229–233) — it does **NOT** restrict to {16,24,32}, and the toolkit's `VALID_ENTROPY_LENGTHS = {16,20,24,28,32}` (seed_xor.rs:24) is *looser still*. So the spec's tighter `{16,24,32}` guard correctly rejects 20/28-byte (15/21-word) and is the only thing enforcing Coldcard interop. `Entropy()` panics on an invalid mnemonic (bip39.go:159–160) — but on the *completed-accept* path the last word is constrained to `bip39.LastWordCandidates` (checksum-valid; gui.go:611–619, 593), so a fully-entered part is `Valid()`. CLI checksum-exclude/recompute confirmed: `m.to_entropy()` then `Mnemonic::from_entropy_in` (cmd/seed_xor.rs:325, 347).

**4. No interpretation ambiguity — TRUE.** Input type = output type = `bip39.Mnemonic`; `bip39.New(result)` recomputes a valid checksum. Unlike the SLIP-39 master-secret BIP-39-vs-BIP-32 ambiguity that drives `engraveRecoveredSLIP39`'s fork (slip39_polish.go:392–413), there is exactly one engrave path. Dropping the two-way fork + interpretation hold-to-confirm is correct.

**5. Menu/dispatch — TRUE.** Input `ChoiceScreen` is `{"12 WORDS","24 WORDS","CODEX32","SLIP-39"}` (gui.go:2015); `engraveObjectFlow`'s `case bip39.Mnemonic: backupWalletFlow(ctx, th, scan)` (gui.go:1849–1850) routes a returned mnemonic with **no new dispatch case**. `masterFingerprintFor(m bip39.Mnemonic, network *chaincfg.Params, password string)` (gui.go:479) matches the spec's call exactly; `chaincfg` is already imported (gui.go:21) — "no new import" for the gui.go diff is accurate.

**6. No-auth-tag advisory parity — TRUE.** The toolkit emits *"Seed XOR has no authentication tag; verify the recovered wallet's expected derived address before trusting; if a share was substituted with a wrong-but-valid one, the result will validate but derive the wrong wallet"* (cmd/seed_xor.rs:374). The spec's §4.3 wording matches this register.

---

## Findings

### CRITICAL
None.

### IMPORTANT

**[I1] §4.1 panic-safety invariant + §4.2 `inputWordsFlow` call are both wrong against real source → a latent partial-fill panic on the only data path into the crypto.**
The spec §4.2 step 3 specifies `inputWordsFlow(ctx, th, m, 0, title="Part i of n")` and states "Back at any prompt → (nil,false)". The **real** BIP-39 entry is `inputWordsFlow(ctx, th, mnemonic, selected)` — it takes **no title** and **returns no bool** (gui.go:580, 701). (The title+bool signature belongs to the *SLIP-39/codex32* siblings: `inputSLIP39Flow(...title string) bool` at gui.go:796 / slip39_polish.go:225, and `inputCodex32Flow(...title) (…, bool)` at codex32_polish.go:171 — the spec conflated them.) Two consequences:

1. **Will not compile as written**, and more importantly there is **no return-value back-out signal.** Back inside `inputWordsFlow` returns immediately at the current `selected` (gui.go:631–632), leaving a *partially-filled* slice containing `-1` entries. The existing 12/24 menu tolerates this because it detects only full back-out via `isEmptyMnemonic` (gui.go:2026) and a partial fill is later caught by `SeedScreen.Confirm`'s `isMnemonicComplete`/`mnemonic.Valid()` gate (gui.go:2106, 2120) *before* any crypto. **The Seed XOR flow has no such interposed validation** between collection and `seedxor.Combine`→`parts[i].Entropy()`.
2. The spec §4.1 asserts *"callers pass only entry-validated/parsed mnemonics, so it's safe."* This is **false** for `inputWordsFlow` output on the Back path: a partially-entered part is not checksum-valid, and `Entropy()` panics (bip39.go:159–160). An unhandled panic in a seed-bearing firmware flow is a robustness/availability defect, exactly the class an R0 gate must close before code.

**Required fix:** §4.2 must (a) use the *actual* `inputWordsFlow(ctx, th, m, selected)` signature, (b) specify the real back-out mechanism — after each `inputWordsFlow` return, treat the part as cancelled-flow unless it passes `isMnemonicComplete(m)` *and* `m.Valid()` (reuse gui.go:2185 / bip39.go:107); a partial or non-empty-but-invalid fill → `(nil,false)`, never a collected part — and (c) decide the "Part i of N" progress title: the real `inputWordsFlow` hardcodes `"Word %d of %d"`, so either accept that (no per-part progress) or add a `title` parameter to `inputWordsFlow` (a signature change touching the existing 12/24 caller at gui.go:2025 and the `SeedScreen` edit caller at gui.go:2102 — must be added to the file manifest §7). §4.1's panic-safety sentence must be rewritten to state that validity is enforced *by the flow's own per-part `Valid()` check*, not assumed from "entry-validated" input.

### MINOR

**[M1] The "embedded Coldcard vectors" do not exist as a copyable artifact (test-soundness).** §6 instructs the implementer to embed "`silent toe … indoor`" (24w) and "`cannon … trade`" (12w) as `testdata`, citing them as "captured." Verified: these literal phrases appear **only inside this spec** (and a one-line description in the recon, "arithmetic reproduced") — there is no file in `mnemonic-engrave`, `mnemonic-toolkit`, or `seedhammer` containing the full part mnemonics + expected result. The toolkit's `lib_seed_xor.rs` is byte-layer/property/G1-byte-pin only (no phrase vectors). The implementer cannot "embed captured vectors"; they must **re-derive them from an authoritative source** (Coldcard `testing/test_seed_xor.py`, or regenerate via the toolkit CLI / Coldcard device) and persist them. Recommend §6 say so, and pin the result against the toolkit G2 round-trip arithmetic as the independent oracle. Not blocking (the algorithm is independently testable), but the spec over-claims the vectors' provenance.

**[M2] Order-independence test must be authored fresh — note it, don't imply a port.** §6 asks for a shuffle test; the toolkit has *no* explicit shuffle/order-independence test (relies on XOR commutativity in property round-trips). Fine to write one, but the spec's "the toolkit G1 byte-pin as oracle" framing shouldn't imply this test is ported. Trivial.

---

## Verdict

**NOT GREEN — 0 Critical / 1 Important.**

The design is sound and the safety crux is solid: the fingerprint gate is genuinely mandatory and Button2-drained, the {16,24,32} interop guard is correctly load-bearing over a permissive `bip39.New`, the no-ambiguity simplification is justified, dispatch/signatures match, and the pre-sized-entry reconciliation (which overrides the architect's mistaken "no picker") is verified correct against real source. The single blocker is **[I1]**: the spec specifies a non-existent `inputWordsFlow` signature and asserts a panic-safety invariant that is false on the Back/partial-fill path, leaving a latent `Entropy()` panic on the only data path into the crypto with no interposed validation. Fold **[I1]** (correct the signature, specify the `isMnemonicComplete`+`Valid()` per-part guard and the back-out mechanism, fix the §4.1 wording, and update §7 if `inputWordsFlow` gains a title param), address minors **[M1]/[M2]**, persist this review verbatim to `design/agent-reports/`, and re-dispatch the R0 gate. No code before GREEN.
