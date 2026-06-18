<!--
Persisted verbatim. opus-architect R0 gate of the Seed XOR combine PLAN
(@ 2c105eb). Reviewer agentId a02147f0c21ee885a. Verdict: NOT GREEN 0C/2I+2m. Both Important are
in the inputWordsFlow title refactor: I-1 (it renders a dynamic 'Word N of M' title pinned by
TestWordFlowProgressTitle — the plan's 'render like inputSLIP39Flow, pass ""' would blank it);
I-2 (10 callers, not 2 — 8 are in gui_test.go — so the param add won't compile). M-1 testdata
network-dependent (add the offline toolkit G1/G2 oracle). M-2 redundant copy (harmless, keep).
Everything else verified correct (seedxor package, I1 panic guard, gate, port fidelity, dispatch).
Disposition: folded. The text below is the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — Seed XOR combine plan

**Reviewer:** opus architect (adversarial R0 gate, read-only)
**Plan:** `design/IMPLEMENTATION_PLAN_seedhammer_seedxor.md` (commit `2c105eb`)
**Spec:** `design/SPEC_seedhammer_seedxor.md` (R1 GREEN, `0664fa7`)
**Base verified against:** fork `main` `bc63caa814e20d2de6140e40026015456c35a2b0`
**Port oracle:** `mnemonic_toolkit::seed_xor::seed_xor_combine` (`seed_xor.rs:161`)
**Date:** 2026-06-18

---

## Verification Results

### 1. Compile / signature reality — mostly GOOD, one import nuance
- **`bip39.Mnemonic` is a slice** — `type Mnemonic []Word` (`bip39.go:24`). So `Combine` returning `nil` as `bip39.Mnemonic` is valid. ✔
- **`Entropy()` returns `[]byte`** and `panic`s on `!m.Valid()` (`bip39.go:158-164`). The inline `append([]byte(nil), parts[0].Entropy()...)` and `e := p.Entropy()` typecheck. ✔ (The defensive copy is technically redundant — `splitMnemonic` freshly allocates `entBytes` each call, no caller alias — but harmless and good hygiene.)
- **`bip39.New`** panics if `len < 16 || 32 < len` or `len%4 != 0` (`bip39.go:228-234`) — i.e. it accepts 16/20/24/28/32. The plan's `interopLen(n) == 16||24||32` guard before `New` is genuinely **load-bearing** and the comment is correct. ✔
- **Import path** `github.com/btcsuite/btcd/chaincfg/v2` matches `gui.go:21` and `slip39_polish.go:8`; `&chaincfg.MainNetParams` (pointer to a package value) matches existing usage (`gui.go:2136`, `slip39_polish.go:418`). ✔
- **`masterFingerprintFor(m bip39.Mnemonic, network *chaincfg.Params, password string) (uint32, error)`** (`gui.go:479`) — the plan's `masterFingerprintFor(seed, &chaincfg.MainNetParams, "")` matches exactly. ✔
- **`ChoiceScreen{Title, Lead, Choices}`** are the real exported fields (`gui.go:1323-1328`); **`Choose(ctx, th) (int, bool)`** confirmed (`gui.go:1337`). `sel+2` over `{"2","3","4","5"}` → 2..5 ✔; `[]int{12,18,24}[sel]` over `{"12","18","24"}` ✔. The `slip39LengthPick` precedent (`slip39_polish.go:40`) matches the picker idiom.
- **Module path** `seedhammer.com` (`go.mod`) → `seedhammer.com/seedxor`, `seedhammer.com/bip39` imports correct. ✔
- `seedxor/` does not yet exist (clean create). ✔
- **One nuance (MINOR):** `seedxor_polish.go` imports `chaincfg/v2` for `&chaincfg.MainNetParams`, but the package-qualified selector is `chaincfg.` while the import path ends in `/v2`. This is correct only because btcd's `chaincfg/v2` package is still **named** `chaincfg` (as gui.go/slip39_polish.go already prove). No aliasing needed. Noted, not a defect.

### 2. The I1 guard (panic safety crux) — SUFFICIENT ✔
`inputWordsFlow` mutates the caller's pre-sized `m` in place and `return`s on Back (`gui.go:631-633`), leaving unentered slots at `-1` (`emptyBIP39Mnemonic`, `gui.go:552-558`). The guard `if !isMnemonicComplete(m) || !m.Valid() { return nil,false }`:
- `isMnemonicComplete` returns false iff any slot is `-1` or `len==0` (`gui.go:2185-2190`) → catches every partial/Back path.
- `m.Valid()` (`bip39.go:107-115`) does **not** panic (it guards `len%3` before `splitMnemonic`); it returns false on bad checksum.
- `Entropy()` panics only on `!Valid()` — and the guard returns before `Combine`/`Entropy()` whenever not complete-AND-valid. Words are only ever set to in-range `bip39.Word` via `completeWord`, so `Valid()` cannot panic either.
- **No residual partial/invalid path reaches `Combine`.** The crux holds.

### 3. The mandatory fingerprint gate — CORRECT ✔
`combineSeedXORFlow` returns `(seed, true)` only after `confirmSeedXORFingerprint(...) == true` (plan lines 245-248); every earlier branch returns `(nil,false)`. The clone template `confirmSLIP39Fingerprint` (`slip39_polish.go:433`) has the unconditional `drainBtn.Clicked(ctx)` Button2-drain at line 445 — the plan instructs keeping it. Seed-XOR-specific "no built-in check" wording specified. The gate is on the only success path, drained, and worded. ✔

### 4. `inputWordsFlow` title-param ripple — **NOT additive as written; two real defects** ✘

Ground truth: `inputWordsFlow` today takes **no** title (`gui.go:580`) and renders a **hardcoded dynamic** title `layoutTitlef(ctx, dims.X, th.Text, "Word %d of %d", selected+1, len(mnemonic))` (`gui.go:701`). By contrast `inputSLIP39Flow` renders **only** a free-form `layoutTitle(..., title)` (`gui.go:868`) and shows **no** word-position line.

- **(a) Behavior change / test break.** The plan (Task 2 Step 1) says "Render `title` as the screen title exactly as `inputSLIP39Flow` does" and (Step 2) "pass the current title… Likely `""`." This is factually wrong: the current effective title is the **dynamic** `"Word N of M"`, not `""` and not any static string. A dedicated test pins it — `TestWordFlowProgressTitle` asserts `uiContains(content, "Word 1 of 24")` (`gui_test.go:487-500`). If the implementer follows the plan literally and replaces the dynamic `layoutTitlef` line with `layoutTitle(..., title)`, then passing `""` (or any caller string) at the two callers will **blank/replace** the "Word N of M" line and break `TestWordFlowProgressTitle` — i.e. the change is *not* behavior-preserving, contradicting the plan's own Step 3 claim. The plan gives no scheme that simultaneously (i) preserves "Word N of M" for the wallet-backup/SeedScreen callers and (ii) shows "Part i of n" for Seed XOR.

- **(b) Caller undercount → compile break.** The plan claims exactly "2 callers" (`gui.go:2025`, `:2102`). There are **10** call sites: 2 in `gui.go` and **8 in `gui_test.go`** (`:285, :491, :507, :604, :625, :642, :662, :681`), all using the current 4-arg signature. Adding a `title` param is a hard compile error across `gui_test.go`; "existing tests stay green" (Step 3) cannot even build until those 8 sites are updated. The plan does not mention them.

The spec (§4.2) shares the same flaw — it asserts the param is "additive, behavior-preserving" and cites only the two `gui.go` callers — but the spec is already R1-GREEN; the gate I'm holding is the **plan**, and the plan must carry an implementable, test-green instruction. This is the load-bearing defect of the review.

### 5. Port fidelity — FAITHFUL ✔ (with one intentional, documented tightening)
Against `seed_xor_combine` (`seed_xor.rs:161-178`):
- **N-of-N / `len<2`→err:** Rust `validate_share_count(>=MIN_SHARES=2)`; Go `len(parts)<2 → errTooFewParts`. ✔
- **Equal length:** Rust checks all lengths equal → `MismatchedShareLengths`; Go checks each later `len(e)!=len(out) → errMismatchedLengths`. ✔
- **Length validation:** Rust `validate_entropy_len` allows `{16,20,24,28,32}`; Go **intentionally narrows to `{16,24,32}`** — this is the documented Coldcard-interop guard (spec §2.3, rejecting 20/28), not a port infidelity. The Rust source's own header (`seed_xor.rs:5-7`) confirms 20/28 are "toolkit-only extensions." ✔
- **XOR fold:** both `out[i] ^= share[i]` over all parts (Go folds part 0 via the copy, then parts 1..N). ✔
- **Order independence:** XOR commutes; Go's fold is order-independent. ✔
- **No caller mutation:** Go's `append([]byte(nil), …)` copies part 0's entropy; later parts are read-only. ✔
- **`Combine([]bip39.Mnemonic)` vs raw entropy:** sound — it derives entropy via `Entropy()` after the I1 validity guarantee; the Rust oracle takes raw `&[&[u8]]` because checksum recompute is the CLI's job, whereas here `bip39.New(out)` does the recompute (matches spec §1 "the result is itself a BIP-39 mnemonic"). ✔

### 6. Vectors / tests — sound in principle, one executable-as-written gap
- **Testdata (M1):** plan Step 1 says fetch Coldcard `docs/seed-xor.md` / `testing/test_seed_xor.py` and persist with a `SOURCE.md` cite. This is a runtime fetch of an external artifact not in-repo; **executable but network-dependent** and not pinnable from this review. Acceptable as a step, but the plan should name the **toolkit G1/G2 byte-pin** (`tests/lib_seed_xor.rs`, cited in spec §6) as the *in-repo, offline* oracle so the test does not hard-depend on a network fetch. MINOR.
- **Negatives → sentinels:** `Combine(parts[:1])→errTooFewParts`; 12+24 mix→`errMismatchedLengths`; 15-word (20-byte) part→`errBadLength`. Correct mapping to the three sentinels. ✔
- **Order-independence test:** real and well-founded. ✔
- **GUI drive:** `driveShare` (`slip39_polish_test.go:217`), `pumpUntil` (`:329`), `runUI`/`click`/`runes` exist and are the right harness. A `driveWord`-style BIP-39 analog is needed (none exists yet) but is trivially adaptable from `driveShare` + the per-word `runes`+`Button3` loop already shown at `gui_test.go:282-283`. Drivable. ✔
- **NOTE (consequence of 4b):** the new GUI tests must use the *new* `inputWordsFlow` signature; fine, but they're new code.

### 7. Scope / consistency — GOOD ✔
- No new `engraveObjectFlow` dispatch: confirmed `case bip39.Mnemonic: backupWalletFlow(...)` already exists (`gui.go:1849-1850`); returning a `bip39.Mnemonic` from `newInputFlow case 4` rides it. ✔
- `newInputFlow` switch is a plain `switch choice` over indices 0..3 (`gui.go:2022-2056`); adding `"SEED XOR"` at index 4 + `case 4:` is structurally correct (the switch is index-based, not range-based). ✔
- No interpretation fork / hold-to-confirm — justified (spec §3: input type = output type = `bip39.Mnemonic`). ✔
- S-sizing, commit hygiene (SSH-sign + DCO + Co-Authored-By), explicit paths — all consistent with CLAUDE.md. ✔

---

## Findings

### CRITICAL
None.

### IMPORTANT
- **I-1 — Task 2 (`inputWordsFlow` title) is not behavior-preserving as specified, and will break `TestWordFlowProgressTitle`.** `inputWordsFlow` renders a hardcoded dynamic title `"Word %d of %d"` (`gui.go:701`), pinned by `gui_test.go:498`. The plan's instruction to "render `title` exactly as `inputSLIP39Flow` does" (which shows *only* a free-form title, no word-position line) plus "pass the current title… likely `""`" would replace/blank the "Word N of M" line for the wallet-backup and SeedScreen-edit callers — a behavior change that fails the existing test and contradicts the plan's own "behavior-preserving / tests stay green" claim.
  **Required fix:** Specify a title scheme that *keeps* the dynamic "Word N of M" line for the existing callers byte-identically (e.g. `title==""` ⇒ render the current `layoutTitlef("Word %d of %d", …)` unchanged; non-empty `title` ⇒ render it, or render it as an *additional* line above the word-position line). Make the existing-caller rendering provably unchanged and keep `TestWordFlowProgressTitle` green by passing `""` at both `gui.go` callers.

- **I-2 — Caller enumeration is wrong (2 claimed, 10 actual), so the signature change won't compile under the existing tests.** Beyond `gui.go:2025` and `:2102`, there are 8 call sites in `gui_test.go` (`:285, :491, :507, :604, :625, :642, :662, :681`) using the current 4-arg signature. The plan must instruct updating all 8 test call sites (pass `""`) as part of Task 2, or the `./gui/` build/test breaks before any new test runs.

### MINOR
- **M-1 — Testdata sourcing is network-dependent.** Plan Task 1 Step 1 fetches Coldcard docs at implementation time. Add the in-repo toolkit G1/G2 byte-pin (`mnemonic-toolkit .../tests/lib_seed_xor.rs`, already cited in spec §6) as the offline oracle so the test isn't hard-blocked on a live fetch; keep the Coldcard vectors as the interop cross-check with a `testdata/SOURCE.md` cite.
- **M-2 — Redundant entropy copy.** `append([]byte(nil), parts[0].Entropy()...)` duplicates an already-fresh slice (`splitMnemonic` allocates per call). Harmless and arguably good defensive hygiene; no change required.

---

## Verdict

**NOT GREEN — 0 Critical / 2 Important.**

Required fixes before implementation:
1. **(I-1)** Rewrite Task 2 (and align the inline guidance) so the `title` param is genuinely additive: preserve `inputWordsFlow`'s existing dynamic `"Word N of M"` title for the two `gui.go` callers byte-identically (empty `title` ⇒ unchanged rendering), while letting Seed XOR supply a per-part progress string — without deleting the word-position line that `TestWordFlowProgressTitle` (`gui_test.go:498`) pins. State the exact render contract.
2. **(I-2)** Enumerate and update **all 10** call sites — including the 8 in `gui_test.go` (`:285, :491, :507, :604, :625, :642, :662, :681`) — in Task 2; "existing tests stay green" must be achievable as a compile precondition.

Recommended (non-blocking): fold M-1 (add the offline toolkit byte-pin oracle alongside the Coldcard fetch).

The `seedxor` package, the I1 panic-safety guard, the mandatory Button2-drained fingerprint gate, port fidelity, menu wiring, and dispatch reuse are all correct against `bc63caa`. The only blockers are in the `inputWordsFlow` title refactor (Task 2). Re-dispatch after the fold.
