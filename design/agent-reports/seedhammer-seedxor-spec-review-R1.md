<!--
Persisted verbatim. opus-architect R1 re-dispatch of the Seed XOR combine spec R0 gate
(SPEC_seedhammer_seedxor.md @ 0664fa7, base fork main bc63caa). Verdict: GREEN — 0C/0I.
Confirms the R0 I1 (inputWordsFlow signature + panic-safety invariant), M1 (Coldcard vectors
not a copyable artifact), and M2 (order-independence test authored fresh) are all folded with
no drift, and the R0-VERIFIED-correct elements are undisturbed. The text below is the agent's
report exactly as returned; do not edit.
-->

# R1 GATE REVIEW — Seed XOR combine (spec)

**Reviewer:** opus architect (adversarial R0 re-dispatch, read-only)
**Spec under review:** `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_seedxor.md` (commit `0664fa7`, "design(seedxor): fold spec R0 (0C/1I+2m)")
**Base / ground-truth commit:** SeedHammer fork `main` `bc63caa` (verified `git rev-parse HEAD` = `bc63caa814e20d2…`, "Merge feat/slip39-trezor-routing"); toolkit port source `mnemonic_toolkit::seed_xor` (`crates/mnemonic-toolkit/src/seed_xor.rs`).
**Prior round:** R0 `seedhammer-seedxor-spec-review-R0.md` — NOT GREEN, 0C/1I + 2 minors (I1 signature + panic invariant; M1 vectors; M2 order-indep).
**Date:** 2026-06-18

---

## Verification Results

Each fold was checked against shipped source on fork main `bc63caa`, not just the draft prose, per the per-phase external-fact rule. Corrected spec lines are quoted alongside the source they must match.

### Fold 1 — I1: real `inputWordsFlow` signature + load-bearing per-part guard + §4.1 panic-safety rewrite

**(1a) The real signature is now used — VERIFIED.** Shipped source: `func inputWordsFlow(ctx *Context, th *Colors, mnemonic bip39.Mnemonic, selected int)` (gui.go:580) — **no `title` param, no return value.** Spec §4.2 step 3 now reads: *"`inputWordsFlow(ctx, th, m, 0)` — the **real** signature (no title, no return value; it fills the pre-sized `m` and returns when full OR returns a *partially-filled* slice on Back)."* This matches the source exactly. The R0-flagged non-existent `inputWordsFlow(...title, bool)` is gone (grep for the 5-arg `title="Part…"` call → no hits). The spec correctly attributes the title+bool shape to the *siblings* (§4.2: "mirroring the SLIP-39/codex32 precedent") — confirmed real: `inputSLIP39Flow(…, selected int, title string) bool` (gui.go:796) and `inputCodex32Flow(ctx, th, title string) (codex32.String, bool)` (gui.go:713).

**(1b) `isMnemonicComplete` + `Valid()` both exist; `Entropy()` panics → guard is load-bearing — VERIFIED.**
- `func isMnemonicComplete(m bip39.Mnemonic) bool` exists at **gui.go:2185** (returns false if `slices.Contains(m, -1)` or empty) — spec cites "gui.go:~2185" ✓.
- `func (m Mnemonic) Valid() bool` exists at **bip39.go:107** — spec cites "bip39.go:~107" ✓.
- `func (m Mnemonic) Entropy() []byte` at bip39.go:158 **panics on an invalid mnemonic**: `if !m.Valid() { panic("invalid mnemonic") }` (bip39.go:159–160). The spec cites "bip39.go:159" / "~159" ✓ and states the panic makes the guard load-bearing.
- The Back/partial-fill hazard is real and the guard closes it: `inputWordsFlow`'s `if backBtn.Clicked(ctx) { return }` (gui.go:631–632) returns at the current `selected`, leaving `-1` entries; full entry returns only when `selected == len(mnemonic)` (gui.go:645–646). The existing 12/24 caller tolerates a partial via `isEmptyMnemonic` (gui.go:2026) + the downstream `SeedScreen.Confirm` `isMnemonicComplete`/`Valid()` gate (gui.go:2106, 2120) — the Seed XOR flow has **no** such downstream gate before `seedxor.Combine`→`Entropy()`, so the spec's interposed per-part guard is the only thing preventing the panic.

Spec §4.2 step 3 now mandates exactly that: *"after each return, validate the part — `if !isMnemonicComplete(m) || !m.Valid() { return nil, false }` … A partial fill (Back) or any non-valid part **aborts the whole flow** (never collected) — so only checksum-valid `L`-word parts reach `seedxor.Combine`/`Entropy()`, which otherwise panics on an invalid mnemonic."* Correct and complete.

**(1c) §4.1 panic-safety now ENFORCES rather than ASSUMES — VERIFIED.** R0 flagged the old §4.1 claim that "callers pass only entry-validated mnemonics, so it's safe" as false on the Back path. The folded §4.1 now reads: *"`bip39.Mnemonic.Entropy()` panics on an *invalid* mnemonic, so `Combine` must only ever receive valid parts. This is NOT assumed — it is *enforced* by the §4.2 flow's per-part `isMnemonicComplete && Valid()` guard before a part is collected (the GUI's `inputWordsFlow` can return a partial slice on Back), and by the unit tests parsing from checksum-valid vectors."* This is the correct framing: validity is enforced by the flow's guard, not assumed from input. ✓

**(1d) Title param is genuinely additive; the two callers are real — VERIFIED.** The title in `inputWordsFlow` is currently hardcoded: `layoutTitlef(ctx, dims.X, th.Text, "Word %d of %d", selected+1, len(mnemonic))` (gui.go:701). Adding a `title string` param is additive — existing callers pass the same string they render today, so behavior is preserved. The two callers the manifest §7 names are real and exact:
- 12/24 menu entry: `inputWordsFlow(ctx, th, mnemonic, 0)` at **gui.go:2025** (spec cites "~:2025") ✓.
- `SeedScreen` edit: `inputWordsFlow(ctx, th, mnemonic, s.selected)` at **gui.go:2102** (spec cites "~:2102") ✓.
These are the *only* two callers (grep `inputWordsFlow(` in gui.go → exactly lines 580/2025/2102; 580 is the def). §7 lists both. The precedent that justifies the param (SLIP-39/codex32 already take a `title`) is verified above. **The title-param addition is genuinely additive/behavior-preserving and both callers are real.** ✓

### Fold 2 — M1: Coldcard vectors are NOT a copyable artifact

**VERIFIED.** §6 no longer says "embed the captured vectors." Grep for "embed the captured vectors" / "captured vectors" → **no hits.** §6 now reads: *"the Coldcard phrase vectors are NOT a pre-existing copyable artifact (the `silent toe … indoor` 24-word / `cannon … trade` 12-word phrases live only in the recon/consult prose) — the implementer must **re-derive them from an authoritative source** (Coldcard `testing/test_seed_xor.py` / `testing/xor.py`, or regenerate via the toolkit CLI / a Coldcard device) and persist them under `seedxor/testdata/` with the source cited … and pin against the toolkit **G1 byte-pin** + **G2 round-trip** arithmetic (`tests/lib_seed_xor.rs`) as the independent oracle."* This matches the ground truth: a full-text search across `mnemonic-toolkit`, `mnemonic-engrave`, and `seedhammer` finds the `silent toe`/`cannon` phrases only in prose docs (recon/consult/spec/agent-reports), never as a complete phrase-pair test fixture; `tests/lib_seed_xor.rs` is G1 byte-pin (`--deterministic-from-master`) + G2 algorithmic round-trip only (header lines 3–6; the only multi-word run in the file is an English code comment at line 262, not a phrase vector). The over-claim is corrected. ✓

### Fold 3 — M2: order-independence test authored fresh

**VERIFIED.** §6 now reads: *"**order-independence** (shuffle parts → same result — authored fresh; XOR commutativity, no toolkit test to port)."* The toolkit has no explicit shuffle/order-independence test (round-trip relies on XOR commutativity), so "authored fresh, no port" is accurate and no longer implied to be ported. ✓

---

## No-drift check

Re-confirmed the fold touched only §4.1, §4.2 step 3, §6, §7 and introduced no new contradiction. The R0-VERIFIED-correct elements are all undisturbed:

- **Mandatory Button2-drained fingerprint gate (security crux):** §4.3 still clones `confirmSLIP39Fingerprint` (gui.go:slip39_polish.go:433) with `drainBtn := &Clickable{Button: Button2}` drained every frame (slip39_polish.go:445). §4.2 step 5 still places the mandatory `confirmSeedXORFingerprint` before step 6's `(seed, true)`, mirroring the verified SLIP-39 BIP-39 arm where `confirmSLIP39Fingerprint` (slip39_polish.go:423) precedes `backupWalletFlow` (slip39_polish.go:426). Undisturbed. ✓
- **{16,24,32} interop guard load-bearing over a permissive `bip39.New`:** `bip39.New` only enforces `len ∈ [16,32]` and `%4 == 0` (bip39.go:229–233), and the toolkit's own `VALID_ENTROPY_LENGTHS = {16,20,24,28,32}` (seed_xor.rs:24) is looser still — so the spec's tighter `{16,24,32}` guard in §4.1 / §2.3 remains the only enforcer of Coldcard interop. Undisturbed. ✓
- **No-ambiguity / no-fork simplification:** §3 unchanged; input type = output type = `bip39.Mnemonic`. ✓
- **Path A dispatch with no new case:** §4.4 still relies on `engraveObjectFlow`'s existing `case bip39.Mnemonic:` (gui.go:1849) → `backupWalletFlow`; `masterFingerprintFor(m bip39.Mnemonic, network *chaincfg.Params, password string)` (gui.go:479) matches §4.2 step 5's call; "no new import" holds. ✓
- **Port faithfulness:** §4.1 algorithm (≥2 parts, equal-length, `{16,24,32}` length, byte-wise XOR, `bip39.New`, wipe) matches `seed_xor_combine` (seed_xor.rs:161) including its `>= 2` and equal-length checks (seed_xor.rs:63). ✓

**Specifically re-confirmed the I1 fix did not break adjacent claims:**
- The *"every part is exactly L words (pre-sized) so length-mismatch is structurally impossible"* claim is **intact and consistent** with the I1 fix. The fix added a *validity* (checksum/completeness) guard, not a length mechanism; length is still fixed by `emptyBIP39Mnemonic(L)` allocation. §4.2 step 3 still states "Every part is exactly `L` words (pre-sized), so length-mismatch is structurally impossible," and §4.2 step 4 keeps the `errMismatchedLengths` handling as explicit defense-in-depth ("the pre-sized entry should make `errMismatchedLengths` unreachable; keep it"). No contradiction: structurally-impossible-but-defended is coherent. ✓
- The **part-length pick** (§4.2 `seedXORPartLength`, one pick that parts 2..N inherit) remains coherent and is correctly grounded in the R0-confirmed pre-sized-entry finding: `inputWordsFlow` fills a `make(bip39.Mnemonic, nwords)` slice whose length must be known before entry, so a single up-front length pick is mechanically necessary. The I1 fix did not alter this. ✓

**Stale-residue grep:** no leftover `title="Part i of n"` 5-arg `inputWordsFlow` call; no "embed the captured vectors"; the only §-numbers referenced by the I1 fold (§4.1 wording, §4.2 step 3, §7) are internally consistent. Clean.

---

## Findings

### CRITICAL
None.

### IMPORTANT
None.

### MINOR
None blocking. (Optional, non-gating: §6 lists `testing/xor.py` as a Coldcard source path; the implementer should confirm the exact upstream filename at vector-derivation time, since the corrected text already correctly frames these as to-be-re-derived-and-cited rather than asserted, this is an implementation note, not a spec defect.)

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

All three R0 items are folded correctly and verified against shipped source on fork main `bc63caa`. I1 is fully resolved: the spec now uses the real `inputWordsFlow(ctx, th, m, selected)` signature (no title/bool), mandates the load-bearing per-part `isMnemonicComplete(m) && m.Valid()` back-out guard before any part reaches `seedxor.Combine`/`Entropy()` (which genuinely panics on an invalid mnemonic at bip39.go:159–160), rewrites §4.1 to *enforce* rather than assume validity, and handles the per-part progress title via a genuinely-additive `title` param to `inputWordsFlow` with the two real existing callers (gui.go:2025, gui.go:2102) listed in §7. M1 (vectors re-derived, not "captured/embedded") and M2 (order-independence authored fresh) are correctly stated. The fold touched only §4.1/§4.2-step-3/§6/§7, introduced no new contradiction, and left the R0-verified-correct elements — the mandatory Button2-drained fingerprint gate, the load-bearing {16,24,32} guard over a permissive `bip39.New`, the no-ambiguity/no-fork simplification, the Path-A no-new-case dispatch, the structurally-impossible-length claim, the mechanically-necessary part-length pick, and the port faithfulness — undisturbed. The spec is cleared for the implementation-plan R0 gate. No code before that gate is GREEN.
