<!--
Persisted verbatim. opus-architect R1 re-dispatch of the Cycle D spec R0 gate
(SPEC_seedhammer_slip39_recovery.md @ 6a87269). Reviewer agentId a7aa600ded8691aab.
Verdict: GREEN — 0C/0I. Both R0 Importants resolved Rust-faithfully; 6 minors
addressed; panel folds no-drift; crypto fidelity re-verified post-rewrite. One
plan-level note: leave backupWalletFlow's own text unchanged. Cleared to D1 planning.
The text below is the agent's report exactly as returned; do not edit.
-->

# R1 GATE REVIEW — Cycle D SLIP-39 recovery (spec)

**Reviewer:** opus architect (adversarial R1 re-dispatch of the R0 gate, read-only)
**Spec:** `design/SPEC_seedhammer_slip39_recovery.md` @ `6a87269` (folded against R0 `…-spec-review-R0.md` + the 3-lens architect panel)
**Base:** fork `main` `20fa4c4` (Slice 3 merged); oracle = `mnemonic-toolkit::slip39` Rust
**Date:** 2026-06-18

---

## Verification Results

I re-read all five Rust oracle files, the firmware integration points (current `20fa4c4`), the official vector corpus, the R0 review, and all three architect-panel briefs, and recomputed the load-bearing numeric/protocol claims. I did **not** assume the R0 crypto verification still held after the rewrite — I re-derived it.

### R0 folds — both Important findings resolved (confirmed)

**I1 (all-lengths reachable) — RESOLVED.** The R0 root cause is real and exact: `gui.go:2035-2036` `case 3:` allocates `emptySLIP39Mnemonic(20)` then `inputSLIP39Flow(ctx, th, mnemonic, 0)` — the 20-word pin lives at the **call site allocation**, not inside `inputSLIP39Flow`. I confirmed `inputSLIP39Flow` (`gui.go:796`) already iterates `selected` against `len(mnemonic)` and lays out against `len(mnemonic)` — it is **length-agnostic in its body**, so the spec's §5.2 widening (add a `title string` param + pass a variable-length `Mnemonic`, size each collected share to `len(first.Mnemonic)`) is structurally supported by the current signature. §0 (D2 bullet), §1 in-scope ("D2 widens `inputSLIP39Flow`"), §5.2, §8 manifest (`gui.go` "title param + variable share length"), and §7 (the 33-word entry test) are mutually consistent. The only two surviving "20-word" mentions (lines 52, 233) are both in the I1-resolution narrative; **no 20-word-only contradiction survives**. Cross-share value-length consistency (§4.3 step 3, `mod.rs:245`) backs the "all shares same length as `first`" sizing rule.

**I2 (idx-9 sentinel) — RESOLVED and Rust-faithful.** I verified against ground truth, not the draft:
- `share.rs:248-256` rejects `group_count < group_threshold` with `GroupThresholdExceedsCount` at **parse**, as **step 5 — before** `decode_value` (step 6). The spec cites `share.rs:250` — the exact line of the `if` guard.
- Official vector **idx-9 = "Mnemonics with greater group threshold than group counts (128 bits)"** with empty expected-secret (error-expected) — confirmed by reading `slip39_vectors.json` (45 entries).
- §4.6 adds `errGroupThresholdExceedsCount` to `ParseShare` (reject `GroupThreshold > GroupCount`, equivalent to Rust's `group_count < group_threshold`), §6 lists it tagged "(idx-9)", §7 maps idx-9 → `errGroupThresholdExceedsCount` @ParseShare. §4/§6/§7 now agree. Placement (at parse, structural) is faithful.

### Minors M1–M6 — all addressed
- **M1:** §5 / §8 add the `bip39` import to `slip39_polish.go` (lines 218, 350); correctly notes `backupWalletFlow` needs no import (same package). ✓
- **M2:** §5.3 signature returns `(bip39.Mnemonic, bool)`; abort documented as `(nil, false)` (line 247) — `nil`, not `""`. ✓
- **M3:** §4.5 pseudocode now writes `pbkdf2.Key(password=…, salt=…, itersPerRound, half, sha256.New)` — correct arg order, matching `bip39.go:225`. ✓
- **M4:** §4.6 (line 196), §7 (line 348), §8 explicitly call out inverting/removing `share_test.go:65-66` (33-word→`errUnsupportedSize`) and the `TestDescribe` `{errUnsupportedSize,"256-bit not supported"}` case (line ~87). I confirmed both exist in the current test file. ✓
- **M5:** §5.3 step 3 pins the exact stop-condition (`satisfied==GT`, each satisfied group at exactly its memberThreshold; stop-and-offer-Continue; start-over for the dead-end). Matches `mod.rs:278-309` strict-equal semantics. ✓
- **M6:** §2.6 + §5.3 note `bip39.New`'s panic precondition is satisfied only by Combine's length guarantee (value ∈ {16,20,24,28,32}). I confirmed `bip39.New` (`bip39.go:228-234`) panics on `len<16 || >32 || %4!=0`; Combine's §4.3 step-2 guarantee covers it. ✓

### Panel folds — no drift, no phantom symbols
- **§0 decomposition (D1/D2)** is internally consistent with §8 manifest (D1 = the four crypto files + `share.go` + tests + fixtures; D2 = GUI) and §9 process (two gated phases, D1 merges between). Forced order (D1→D2) is justified by D2 writing against `Combine`'s frozen contract. ✓
- **§3 hold-to-confirm:** `ConfirmWarningScreen` is a real type (`gui.go:312`, `Layout(...)→(op.Op, ConfirmResult)`), used exactly as the "Discard Seed?" hold pattern inside `SeedScreen.Confirm` (`gui.go:2079`). The cited line is the canonical usage site. (Note: it's a struct, not a `func` — the spec correctly calls it "the existing `ConfirmWarningScreen` hold pattern.") ✓
- **§5.4 fingerprint:** `masterFingerprintFor(m bip39.Mnemonic, network *chaincfg.Params, password string) (uint32, error)` (`gui.go:479`) matches the spec's `masterFingerprintFor(m, net, "")` invocation exactly. ✓
- **§5.7 `backupWalletFlow` reuse:** `backupWalletFlow(ctx, th, mnemonic bip39.Mnemonic)` (`gui.go:1929`) is generic (Confirm → optional BIP-39 passphrase → fingerprint choice → engrave) and not recovery-aware — exactly as §5.7 mandates. ✓
- **§4.8 scrubbing, §5.5 passphrase-labeling, §5.6 off-thread/high-e warn:** internally consistent; the firmware-resource facts (RP2350 software-SHA, `(10000<<15)/4` int32-safe, `-scheduler tasks`) are consistent with the panel brief and the oracle.

### Crypto fidelity (§4.1–4.6) — re-verified intact after the rewrite (NOT assumed)
- **GF(256) §4.1:** `reductionPoly=0x11b`, **generator 3**, table loop `x=(x<<1)^x; if x&0x100 {x^=0x11b}`, `expTbl[255]=1`, `gfMul` `≥255 → −255`, `gfInv` `(255-log)%255` — verbatim vs `gf256.rs:14-18,33-48,60-82`. ✓
- **Lagrange §4.2:** XOR-subtraction basis, per-byte interpolation, `secretIndex=255`/`digestIndex=254`/`digestLen=4` — vs `lagrange.rs:37-60,70-91`. ✓
- **Feistel §4.5:** rounds `[3,2,1,0]`, `L⊕=F; swap`, output `R||L`, `password=[]byte{i}||passphrase`, `salt=saltPrefix||R`, `itersPerRound=(10000<<e)/4`, `saltPrefix="shamir"||be16(id)` (ext=0) / empty (ext=1) — verbatim vs `feistel.rs:101-166,175-206`. `"shamir_extendable"` correctly noted as RS1024-only (`share.rs:71`, `share.go:106`). ✓
- **recoverSecret / two-level combine §4.3:** empty→error; per-share len∈{16,20,24,28,32}; six cross-share sentinels; group-by-index sorted; per-group uniform mt + distinct indices + **exactly** mt; **exactly** GT groups; group recover → EMS → feistel decrypt using `shares[0]`'s id/ext/iter; `recoverSecret` T==1 (no digest) / T≥2 (`S=interp(255)`, `D=interp(254)`, `digest=D[:4]`, `R=D[4:]`, `HMAC-SHA256(R,S)[:4]==digest`) — vs `mod.rs:206-331,430-458`, step-for-step. The §4.3 use of `subtle.ConstantTimeCompare` where Rust uses `!=` is the crypto-lens free-hygiene fold — an **improvement**, not a divergence. ✓
- **§4.6 byte-oriented unpack:** spec mandates bit-at-a-time MSB-first into `[]byte` with a **per-byte (8-bit) accumulator, no value-wide accumulator** — matches `decode_value` (`share.rs:338-369`, `get_bit`/per-byte packing). Correct for 33-word/256-bit on 32-bit TinyGo. Length/padding table recomputed: W∈{20,23,27,30,33} → padBits {2,0,8,6,4} (all ≤8; W=27 is the `==8` boundary) → bytes {16,20,24,28,32}, matching `share.rs:220-226`. ✓
- **Panic-safety §4.4:** still sufficient — all input-violable preconditions (steps 1–5) checked-and-returned before any `gfDiv`/interpolation; distinct x-coords guaranteed; mandatory never-panics test retained. ✓

---

## CRITICAL
**None.**

## IMPORTANT
**None.** Both R0 Importants (I1, I2) are resolved and Rust-faithful; the rewrite did not corrupt the verified crypto.

## MINOR
- **(observation, non-blocking) §5.5 vs §8 latent tension.** §5.5 says "*If* `backupWalletFlow` later offers a BIP-39 passphrase, it is labeled by function." `backupWalletFlow`'s prompt is `ChoiceScreen{Lead:"Add a BIP-39 passphrase?"}` (`gui.go:1941`), and §8 lists `gui.go` BIP-39/Slice-3 logic as "Unchanged." If the implementer reads §5.5 as requiring a retitle of that prompt, it would touch supposedly-unchanged code. The existing prompt is already function-named ("BIP-39 passphrase"), and §5.5 is satisfiable via the §5.7 disambiguation brackets (§3 hold-to-confirm + §5.4 fingerprint) without editing `backupWalletFlow`. The D2 plan should state explicitly that `backupWalletFlow`'s own text stays unchanged. Plan-level, not spec-blocking.

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

The R0 folds are correct and complete: I1 (widen `inputSLIP39Flow` + size collected shares to `len(first.Mnemonic)`; §1/§5/§7/§8 reconciled with no surviving 20-word contradiction) and I2 (`errGroupThresholdExceedsCount` added to `ParseShare`, matching `share.rs:250` `GroupThresholdExceedsCount`, with vector idx-9 confirmed as the greater-group-threshold-than-count case and the §4/§6/§7 expectation reconciled). All six minors (M1–M6) are addressed. The three architect-panel folds (D1/D2 decomposition, §3 hold-to-confirm, §4.8 scrubbing, §5.4 fingerprint, §5.5 passphrase labeling, §5.6 off-thread/high-e warn, §5.3 roster UX) introduced no drift and reference only symbols that exist with the cited signatures (`ConfirmWarningScreen`, `masterFingerprintFor`, `backupWalletFlow`, `bip39.New`, `inputSLIP39Flow`). The crypto fidelity (§4.1–4.6) survived the rewrite intact and was re-verified against the Rust oracle, not assumed. The lone minor observation is plan-level. Persist this review verbatim to `design/agent-reports/`; the spec may proceed to D1 planning.
