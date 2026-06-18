<!--
Persisted verbatim. opus-architect R0 gate of the Cycle D D2 (GUI recover) plan
(IMPLEMENTATION_PLAN_seedhammer_slip39_recovery_D2.md @ bedecc1). Reviewer agentId
aa03d4bca54744c2c. Verdict: NOT GREEN 1C/2I+5m. C1 wrong-passphrase test assertion
(empirically run), I1 two-level assembly admits a stray partial group into Combine,
I2 missing chaincfg import. Watchdog/PBKDF2 confirmed adequately handled (no active WDT).
Disposition: folded. The text below is the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — Cycle D D2 plan (SLIP-39 GUI recover)

**Reviewer:** opus architect (adversarial R0 gate, read-only)
**Plan:** `design/IMPLEMENTATION_PLAN_seedhammer_slip39_recovery_D2.md` @ `bedecc1`
**Spec:** `design/SPEC_seedhammer_slip39_recovery.md` §0/§5 (R1 GREEN)
**Base (fork main):** `f0092d5` (verified: `git rev-parse HEAD` = `f0092d53077…`, "Merge feat/slip39-recovery-crypto … Phase D1")
**Date:** 2026-06-18
**Baseline:** `go test ./gui/ ./slip39/ ./bip39/` → all `ok` (green before changes).

---

## Verification Results (source-confirmed)

- **D1 crypto is real and untouched by this plan.** `slip39.Combine(shares []Share, passphrase []byte) ([]byte,error)` (combine.go:39), `ConsistentShares([]Share) error` (combine.go:148, count-agnostic, checks id/ext/iterExp/groupThr/groupCount/value-len + dup `(GroupIndex,MemberIndex)` — confirms the plan's eager-validation reliance), `Describe` (share.go:167), and `Share` fields (share.go:21–32: `GroupThreshold/GroupCount/MemberThreshold/MemberIndex/GroupIndex/Mnemonic/Value/IterationExp/Identifier/Extendable`) all match the plan. D2 adds no crypto. ✔
- **`Combine` exact-count preconditions confirmed (combine.go:81–109):** every group present in the slice must have `len(gs) == mt` and `len(groupShares) == first.GroupThreshold`. Over-collection (extra member, or an extra/partial group) errors. ✔ (drives Critical/Important below).
- **Codex32 template is real:** `codex32ConfirmAction` enum (codex32_polish.go:70–76), unconditional Button2 drain w/ the R0-C1 comment (lines 106–111), `recoverCodex32Flow` returning `(codex32.String,bool)` (line 161), `inputCodex32Flow(ctx,th,title)` (gui.go:713), the `engraveCodex32` dispatch `for{ switch … }` loop (lines 198–217). ✔
- **`inputSLIP39Flow(ctx,th,mnemonic slip39words.Mnemonic, selected int) bool`** (gui.go:796) fills a pre-sized slice and returns `true` when full — the picker/`emptySLIP39Mnemonic(n)` model is compatible. **But its title is the hard-coded literal `"Input Words"` (line 868), not a "SLIP-39 title"** — the plan's wording ("replace the current fixed SLIP-39 title") is inaccurate; the mechanism (param-ize the literal) is sound. ✔ (minor wording).
- `emptySLIP39Mnemonic(n)` (gui.go:544), menu `case 3:` using `emptySLIP39Mnemonic(20)` + `inputSLIP39Flow(ctx,th,mnemonic,0)` (gui.go:2034–2051), `ConfirmWarningScreen`+`Layout`→`ConfirmResult` hold pattern (gui.go:312–341, `ConfirmYes` on 1s hold), `ChoiceScreen.Choose`→`(int,bool)` (gui.go:1337), `backupWalletFlow(ctx,th,mnemonic bip39.Mnemonic)` (gui.go:1929), `masterFingerprintFor(m,*chaincfg.Params,password) (uint32,error)` (gui.go:479), `bip39.New([]byte) Mnemonic` (bip39.go:228), `bip39.Mnemonic.Entropy()` (bip39.go:158). All signatures match. ✔
- **`engraveObjectFlow` (gui.go:1847–1862)** is the real dispatcher; `case slip39words.Share: return engraveSLIP39(...)`. The existing `TestEngraveSLIP39BackoutRecognized` drives **`engraveObjectFlow`**, not `engraveSLIP39` directly (slip39_polish_test.go:41). ✔
- **No active hardware watchdog.** The only `rp.WATCHDOG.*` use (platform_sh2.go:413–425) stages the BOOTSEL reboot vector via SCRATCH registers; there is **no** `WATCHDOG.CTRL/LOAD` enable or periodic feed anywhere in `cmd/` or `driver/`. A blocking `Combine` at e=0/1 (~0.5–1.9 s) will **not** trip a reset. The plan's "verify the watchdog / fall back to off-thread" is a *correctly-scoped impl-time verify with a safe default*, not a deferred Critical hazard. ✔
- **Fixture reality:** `slip39/testdata/slip39_vectors.json` idx 3 = "Basic sharing 2-of-3 (128 bits)", 2 mnemonics (the threshold subset), master `b43ceb7e57a0ea8766221624d01b0864`. Decoded share-0 header: `GroupThreshold=1, GroupCount=1, MemberThreshold=2, IterationExp=2`. So the Recover-offer predicate `MemberThreshold>1 || GroupThreshold>1` correctly fires, and e=2 is below the e≥4 high-e gate. ✔
- **Test harness:** `runUI`/`uiContains` (gui_test.go:467/480), `click`/`runes`/`press` (event_test.go:42/68/57), `newPlatform`/`NewContext`/`descriptorTheme` all real. Direct-call confirm tests (`click` then call, no `runUI`) are the established pattern (`TestConfirmCodex32ShareOffersRecover`, codex32_polish_test.go:209). ✔
- `showError(ctx,th,title,msg)` (slip39_polish.go:19) takes a title — `showError(ctx,th,"SLIP-39",…)` works. ✔

---

## CRITICAL

**C1 — `TestRecoverSLIP39`'s asserted entropy is the wrong-passphrase value; the test as written cannot pass.**
The plan (Task 3 Step 1 / SPEC §7) drives the passphrase choice as **Skip** (empty) yet asserts `hexOfEntropy(m) == "b43ceb7e57a0ea8766221624d01b0864"`. I executed D1 `Combine` on idx-3's two shares:
```
pp=""       -> 61cf4d6c0d8a07d8c2fd3cff22432664
pp="TREZOR" -> b43ceb7e57a0ea8766221624d01b0864
```
`b43c…0864` is the **"TREZOR"-passphrase** recovery (the official corpus uses passphrase "TREZOR" throughout; `slip39/vectors_test.go:114-116` Combines idx 3 with `[]byte("TREZOR")`). With Skip/empty the master secret is `61cf4d6c…2664` (SLIP-39's deliberate plausible-deniability, pinned by `TestCombinePassphraseDistinguishes`). The planned test would therefore **fail**. Fix: either (a) drive the SLIP-39 passphrase keyboard to type `TREZOR` and keep the `b43c…` assertion, or (b) keep Skip and assert `61cf4d6c0d8a07d8c2fd3cff22432664`. (Note: 16-byte secret → `bip39.New` 12-word mnemonic; `Entropy()` returns the 16 bytes, so `hexOfEntropy` is realizable once the expected value is corrected.)

---

## IMPORTANT

**I1 — Two-level share-set assembly is under-specified and admits a wrong `Combine` input (latent recovery failure on a valid pile).**
`Combine` requires that *every group present in the passed slice* be at *exactly* its memberThreshold and that the number of present groups *equals* GroupThreshold (combine.go:83–109). The plan accumulates **every** consistent share into a flat `shares` slice (Task 3 Step 3.2) and passes that whole slice to `Combine` (Step 5). "Stop exactly at sufficiency" only stops *prompting* once `satisfied == GT` — it does **not** prevent a stray **partially-filled group** from remaining in `shares`. Example (GT=2, groups A/B/C each mt=2): enter A1, C1 (wrong pile), A2, B1, B2 → satisfied groups A,B == GT → loop stops, but `shares` still contains the lone **C1** → `Combine` returns `errInsufficientShares` despite a sufficient A+B set being present. The single-group idx-3 fixture (the only crypto fixture exercised here) cannot catch this; codex32's flat template has no analogue. Fix: specify that the slice fed to `Combine` is built from **exactly the GT satisfied groups' members** (prune partial/extra groups), or refuse appending a share that would start an uncompletable group. The plan must state the assembly rule, and a multi-group GUI test (e.g. idx 17/35-style topology) should cover it.

**I2 — Missing `chaincfg` import for the §5.4 fingerprint, in a file that doesn't import it.**
Task 4 Step 3.2 calls `masterFingerprintFor(m, &chaincfg.MainNetParams, "")` from inside `slip39_polish.go`. That file currently imports `backup/constant/assets/layout/op/widget/slip39words/fmt/image` only — **not** `github.com/btcsuite/btcd/chaincfg/v2`. The plan/spec list only "add `bip39`" (plan File-structure line 62; SPEC §5 line 218); **neither lists `chaincfg`**, which is load-bearing (the §5.4 always-on fingerprint is a mandated pre-engrave gate and cannot be satisfied by `backupWalletFlow`'s internal fingerprint, which only appears embedded in the passphrase choice). Add `chaincfg` to the `slip39_polish.go` import list in the plan. (gui.go's "no new import" claim is independently verified correct: it already has fmt/strings/chaincfg/bip39/slip39words.)

---

## MINOR

- **M1 — `driveRecover`/SLIP-39 word-entry driving is an unproven stub.** No existing test drives `inputSLIP39Flow` word entry via `runes`; the codex32 recover test drives a *different* keyboard. SLIP-39 entry requires, per word, a disambiguating prefix (`completeSLIP39Word`, gui.go:963: complete when `nvalid==1` or exact) **then** `click(Button3)` to accept (inputSLIP39Flow:821–839) — ~20 words × (runes+Button3) per share. Feasible via the harness, but the plan leaves `driveRecover`/`vec3Share`/`parseFixtureShare`/`hexOfEntropy` as sketches; budget TDD time and pin the per-word accept sequence.
- **M2 — `TestConfirmSLIP39MultiOffersRecover` has a spurious `Down`.** `click(&ctx.Router, Down, Button2)` — the confirm screen has no list to navigate; the codex32 analogue uses just `click(Button2)`. `Down` is harmlessly drained, but drop it for clarity/parity.
- **M3 — Fingerprint format-string inconsistency.** Plan uses `%08X`; `backupWalletFlow` uses `%.8X` (gui.go:1952). Identical output for uint32; cosmetic — prefer matching `%.8X` for consistency.
- **M4 — `slip39LengthPick` host-file ambiguity.** Task 2 header says "**Files:** `gui/gui.go`" but the File-structure table lists `slip39LengthPick` under `slip39_polish.go`. Same package, so no compile impact; pick one and state it.
- **M5 — Wording: `inputSLIP39Flow` title.** The "fixed SLIP-39 title" to replace is the literal `"Input Words"` (gui.go:868), not a SLIP-39-specific string. Correct the plan's description.

---

## Verdict

**NOT GREEN — 1 Critical / 2 Important.**

**Required fixes before implementation:**
1. **(C1)** Correct `TestRecoverSLIP39`: align the passphrase the test drives with the asserted entropy — either type `TREZOR` and keep `b43ceb7e…0864`, or keep Skip and assert `61cf4d6c0d8a07d8c2fd3cff22432664`. Verified empirically against D1 `Combine`.
2. **(I1)** Specify the two-level share-set assembly: feed `Combine` exactly the GT satisfied groups' members (prune partial/extra groups) or refuse uncompletable-group shares; add a multi-group GUI test (single-group idx-3 cannot catch the bug).
3. **(I2)** Add `github.com/btcsuite/btcd/chaincfg/v2` to the `slip39_polish.go` import list in the plan (required by the §5.4 fingerprint call).

The watchdog/PBKDF2 concern is **adequately handled** (verified: no active WDT; blocking e≤1 is safe; e≥4 warn gate + off-thread fallback are specified, not hand-waved). The word-count picker design is sound and composes with `emptySLIP39Mnemonic`/`inputSLIP39Flow`. The Cycle-C all-lengths resolution claim, D1-untouched claim, Button2-drain/no-hang, Back-returns-recognized, passphrase isolation/labeling, and `backupWalletFlow` reuse are all consistent with source. Fold C1+I1+I2 (and ideally the minors), persist this review verbatim to `design/agent-reports/`, and re-dispatch.
