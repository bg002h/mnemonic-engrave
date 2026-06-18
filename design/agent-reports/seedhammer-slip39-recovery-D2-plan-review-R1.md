<!--
Persisted verbatim. opus-architect R1 re-dispatch of the Cycle D D2 plan R0 gate
(@ ec8f9d1). Reviewer agentId a821abbcdb1119832. Verdict: GREEN — 0C/0I. All three R0
blockers (C1 passphrase assertion, I1 selectForCombine, I2 chaincfg import) + 5 minors folded
correctly, verified against shipped D1 source (combine.go exact-count preconditions,
empirically-derived idx-3 secrets, the group-2of3-over-2of3 fixture, gui.go chaincfg import).
No drift. Cleared for implementation. The text below is the agent's report exactly as returned; do not edit.
-->

# R1 GATE REVIEW — Cycle D D2 plan (SLIP-39 GUI recover)

**Reviewer:** opus architect (adversarial R1 re-dispatch of the R0 gate, read-only)
**Plan:** `design/IMPLEMENTATION_PLAN_seedhammer_slip39_recovery_D2.md` @ `ec8f9d1` ("design(cycle-d): fold D2 plan-R0 (1C/2I+5m)")
**R0 review folded:** `design/agent-reports/seedhammer-slip39-recovery-D2-plan-review-R0.md` (1C / 2I + M1–M5, base `bedecc1`)
**Spec:** `design/SPEC_seedhammer_slip39_recovery.md` §0/§5 (R1 GREEN)
**Base (fork main):** `f0092d5` (D1 merged; crypto shipped & trusted)
**Date:** 2026-06-18
**Empirical D1 check:** ran `slip39.Combine` on official vector idx "3" shares in a throwaway test; results quoted below.

---

## Verification Results (source-confirmed)

### C1 — wrong-passphrase test assertion — FOLDED ✔
I re-derived the secret empirically against shipped D1 `Combine` on the two idx-3 shares (`slip39/testdata/slip39_vectors.json["3"]` = "4. Basic sharing 2-of-3 (128 bits)"):
```
pp=""       -> 61cf4d6c0d8a07d8c2fd3cff22432664
pp="TREZOR" -> b43ceb7e57a0ea8766221624d01b0864
```
This matches the R0 reviewer's empirical finding exactly, and is pinned upstream by `slip39/vectors_test.go:126 TestCombinePassphraseDistinguishes` (TREZOR → `vectorSecretHex(3)` = `b43c…`; empty must differ). The plan now correctly assigns:
- `TestRecoverSLIP39` (plan L198–205) drives **Skip** and asserts `61cf4d6c0d8a07d8c2fd3cff22432664` (L203) — the empty-passphrase value. Correct.
- `TestRecoverSLIP39Passphrase` (plan L208–219) types **"TREZOR"** and asserts `b43ceb7e57a0ea8766221624d01b0864` (L216) — the corpus value. Correct.

The two values are not swapped, and `b43c…` no longer appears on any Skip path. The comment at L196–197 records both values for the reader; the self-review (L358–360) restates them consistently. Side facts confirmed: idx-3 share header is `MemberThreshold=2, GroupThreshold=1, IterationExp=2` — so the Recover-offer predicate `MemberThreshold>1 || GroupThreshold>1` fires and e=2 is below the e≥4 high-e gate. The 16-byte secret → 12-word `bip39.New` mnemonic, `Entropy()` returns 16 bytes, so `hexOfEntropy` is realizable.

### I1 — stray partial group admitted to Combine — FOLDED ✔
`combine.go` confirms the exact-count preconditions: per present group `if len(gs) != mt { return errInsufficientShares }` (L91 region), and after group reduction `if len(groupShares) != first.GroupThreshold { return errInsufficientShares }` (L104). A lingering partial/extra group in a flat slice therefore breaks `Combine` on a genuinely-sufficient pile — exactly the R0 bug.

The plan now specifies (Task 3 Step 3.3, L246–260) a pure helper:
```go
func selectForCombine(byGroup map[int][]slip39words.Share, groupThreshold int) (shares []slip39words.Share, ok bool)
```
returning "the flattened members of **exactly** groupThreshold satisfied groups … dropping partial/extra groups," and it feeds **that** result to `Combine`, not the raw accumulation (L257). Tracing against `combine.go`: every group in the selected slice has exactly `mt` members and exactly GT groups are present → both `errInsufficientShares` branches are avoided; the C1-class example (A1,C1,A2,B1,B2 with stray C1) prunes C → passes only A+B → recovers. The over-fill end is also closed at collection time: Step 2 (L242–244) rejects a share whose group is already satisfied (no group exceeds `MemberThreshold`; also avoids `errDuplicateMemberIndex`). Coverage added (Step 1bis, L272–277): `selectForCombine` unit test cases (a) single satisfied group, (b) **stray partial group pruned, ok=true**, (c) fewer than GT → ok=false; plus a multi-group GUI round-trip using the `group(2-of-3 over 2-of-3 groups, GT=2)` fixture. I confirmed that fixture exists: `slip39/testdata/slip39_fixtures.json` indices 2/5/8/11/14 are `group-2of3-over-2of3` topologies (idx 2: secret `101112…1e1f`, 4 mnemonics splitting into 2 groups of 2 members each, passphrase "TREZOR").

### I2 — missing chaincfg import — FOLDED ✔
The §5.4 fingerprint (Task 4 Step 3.2, L315) calls `masterFingerprintFor(m, &chaincfg.MainNetParams, "")` from `slip39_polish.go`. Confirmed: `slip39_polish.go` currently imports only `fmt/image/backup/constant/assets/layout/op/widget/slip39words` — **not** chaincfg. The plan's File table (L62) and self-review (L361) now require adding `github.com/btcsuite/btcd/chaincfg/v2` to `slip39_polish.go`. I verified that is the correct module path: `gui/gui.go:21` imports exactly `github.com/btcsuite/btcd/chaincfg/v2` and uses `&chaincfg.MainNetParams` / `masterFingerprintFor(mnemonic, &chaincfg.MainNetParams, "")` at gui.go:1935. The plan's "gui.go gains no new import" claim is independently correct (chaincfg/bip39/fmt/strings/slip39words already present there).

### Minors M1–M5
- **M1** ✔ — driveShare per-word realism note present (L279–288): per word `runes(<disambiguating prefix>)` then `click(Button3)`, citing `completeSLIP39Word` complete-when-`nvalid==1`/exact (`gui.go:821-839,963`); a reusable `driveShare(t, ctx, mnemonic)` helper is specified; TDD time budgeted.
- **M2** ✔ — `TestConfirmSLIP39MultiOffersRecover` is now `click(&ctx.Router, Button2)` with the explicit "no spurious Down" note (L92). The remaining `Down` (L157) is in the length-picker test where navigating to the 33-word option is legitimate.
- **M3** ✔ — only `%.8X` appears (L316–317), matching `backupWalletFlow` (gui.go:1935 uses `%.8X`); no `%08X` residue anywhere.
- **M4** ✔ — `slip39LengthPick` is unambiguously placed in `slip39_polish.go` (File table L62; Task 2 header L143–147 states Task 2 touches **both** files: `slip39LengthPick` in `slip39_polish.go`, the `inputSLIP39Flow` title param + `case 3:` in `gui.go`).
- **M5** ✔ — the title literal to param-ize is correctly named `"Input Words"` at `gui.go:868` (L64, L168); I confirmed that literal at gui.go:868. No "fixed SLIP-39 title" / "replace the current SLIP-39 title" residue remains.

---

## No-drift check

Grep of the plan for stale residue is clean: no `%08X`; no `b43c…`-on-Skip; no "fixed/current SLIP-39 title" phrasing. The folds are localized to the intended regions — File-structure table (L62/L64), Task 2 header + Step 3.1 (L143–147, L166–169), Task 3 Step 1 assertions + Step 3.3/Step 1bis (L192–219, L246–277), Task 4 Step 3.2 (L315–317), and the self-review (L346–369). No new internal contradiction: the self-review now matches the I1 assembly rule (L353–357: "built by `selectForCombine` = exactly the GT satisfied groups' members") and the C1 assertions (L358–360: `61cf…` Skip / `b43c…` TREZOR). Step 2's "reject already-satisfied group" composes with — does not contradict — Step 3.3's pruning (collection-time over-fill guard + assembly-time partial-group prune are complementary). The R0-confirmed-adequate items (no active WDT; e≤1 blocking safe; e≥4 warn gate; word-count picker; Cycle-C all-lengths; D1-untouched; Button2-drain/no-hang; Back-returns-recognized; `backupWalletFlow` reuse) are unchanged and remain consistent.

---

## Findings

- **CRITICAL:** none.
- **IMPORTANT:** none.
- **MINOR:** none. (M1–M5 all folded; nothing new surfaced.)

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

All three R0 blockers (C1 passphrase/secret assertion, I1 two-level assembly via `selectForCombine`, I2 chaincfg/v2 import) and all five minors are folded correctly and verified against shipped D1 source (`combine.go` exact-count preconditions, empirically-derived idx-3 secrets, the `group-2of3-over-2of3` fixture, and `gui.go`'s chaincfg import). No drift, no stale residue, no new internal contradiction. The plan is cleared for implementation.

Key paths for the record:
- Plan: `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_seedhammer_slip39_recovery_D2.md` @ `ec8f9d1`
- D1 source verified: `/scratch/code/shibboleth/seedhammer/slip39/combine.go`, `/scratch/code/shibboleth/seedhammer/slip39/vectors_test.go`, `/scratch/code/shibboleth/seedhammer/slip39/testdata/slip39_vectors.json` (idx "3"), `/scratch/code/shibboleth/seedhammer/slip39/testdata/slip39_fixtures.json` (idx 2 = group-2of3-over-2of3), `/scratch/code/shibboleth/seedhammer/gui/gui.go` (chaincfg L21, `%.8X` L1935, `inputSLIP39Flow` L796, "Input Words" L868, `case 3:` L2034), `/scratch/code/shibboleth/seedhammer/gui/slip39_polish.go` (imports, no chaincfg).
