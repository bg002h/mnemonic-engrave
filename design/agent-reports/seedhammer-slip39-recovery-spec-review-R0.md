<!--
Persisted verbatim. opus-architect R0 gate of the Cycle D spec (SPEC_seedhammer_slip39_recovery.md @ 9d1dfd6). Reviewer agentId af0f23a44ea985f6b. Verdict: NOT GREEN — 0C/2I (I1 all-lengths-unreachable, I2 idx-9 sentinel) + 6 minors. Crypto port verified faithful, panic-safe, digest gate unbypassable.
The text below is the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — Cycle D SLIP-39 recovery (spec)

**Reviewer:** opus architect (adversarial R0 gate, read-only)
**Spec:** `design/SPEC_seedhammer_slip39_recovery.md` @ `9d1dfd6`
**Recon:** `design/cycle-prep-recon-slip39-recovery.md` @ `3c1e8ce`
**Base:** fork `main` `20fa4c4` (Slice 3 merged); oracle = `mnemonic-toolkit::slip39` Rust
**Date:** 2026-06-18

---

## Verification Results

I read all five Rust oracle files, the four firmware integration points, and the official vector fixture, and recomputed every numeric/protocol claim. The crypto port spec (§4) is faithful to the Rust.

**Crypto fidelity (§4) — all confirmed against the Rust oracle:**
- **GF(256):** spec `reductionPoly=0x11b`, `generator=3`, table loop `x=(x<<1)^x; if x&0x100 {x^=0x11b}`, `expTbl[255]=1` — verbatim match to `gf256.rs:14-18,41-48`. `gfMul` `>=255?-255` form: max `log_sum=508 → 253 < 255`, single subtract is valid (`gf256.rs:65-71`). `gfInv` `(255-log)%255` matches `gf256.rs:80`; inv(1): `255%255=0→exp[0]=1` correct.
- **Lagrange (§4.2):** XOR-subtraction basis `L_i=Π(x⊕xj)/(xi⊕xj)`, per-byte interpolation, `secretIndex=255`/`digestIndex=254`/`digestLen=4` — match `lagrange.rs:37-60`, `mod.rs:38-45`.
- **Feistel decrypt (§4.5):** round order `[3,2,1,0]`, body `L^=F; swap(L,R)`, output `R||L`, `password=[]byte{i}||passphrase`, `salt=saltPrefix||R`, `itersPerRound=(10000<<e)/4`, `saltPrefix= "shamir"||be16(id)` (ext=0) / empty (ext=1) — verbatim match to `feistel.rs:120,134-166,175-206`. The two-`shamir`-string trap is correctly avoided (`"shamir_extendable"` is RS1024-only; firmware `share.go:106` already uses it for the checksum). `(10000<<15)/4 = 81,920,000` and `10000<<15 = 327,680,000` both fit int32 (RP2350-safe).
- **recoverSecret (§4.3):** `T==1` → single value (no digest); `T≥2` → `S=interp(255)`, `D=interp(254)`, `digest=D[:4]`, `R=D[4:]`, `HMAC-SHA256(key=R,msg=S)[:4]==digest` — verbatim match to `mod.rs:439-457`. Digest order `digest||R` correct.
- **Two-level combine (§4.3):** empty→error; per-share length∈{16,20,24,28,32}; six cross-share sentinels; group-by-index (sorted); per-group uniform memberThreshold + distinct member indices + exact count; exact groupThreshold groups; group recover → EMS → feistel decrypt — matches `mod.rs:210-330` step-for-step (uses `shares[0]`'s id/ext/iter for decrypt, justified by step-3 consistency).
- **§4.6 length table** (recomputed): {20,23,27,30,33} words → {16,20,24,28,32} bytes, every one satisfies `padBits=(10(W−7))%16 ≤ 8` (W=27 is the `==8` boundary, correctly accepted by `>8` reject). These are exactly the valid SLIP-39 counts. Matches `share.rs:220-226`.
- **Firmware header decode reuse:** 2000-sample fuzz confirms `share.go:111-121` header decode is bit-identical to `share.rs:229-246`.
- **`bip39.New` (§3):** accepts entropy 16–32 (mult of 4); {16,20,24,28,32}→{12,15,18,21,24} words; won't panic given Combine's length guarantee (`bip39.go:228-234`).

**Panic-safety (§4.4) — sufficient.** Enumerated every combine-path panic site and confirmed each is gated: `gfInv(0)`/`gfDiv(_,0)` (`gf256.rs:78,86`) — denominators are `Πxi⊕xj` over share x-coords, non-zero given distinct member/group indices validated before interpolation; `interpolate_at` dup-x (`lagrange.rs:47`) — gated by distinct-index checks (§4.3 step 4 + BTreeMap grouping); `interpolate_secret_at` empty/length-mismatch (`lagrange.rs:71,74`) — gated by empty-check (step 1) + equal-length check (step 3, preserved through both interp layers); `feistel_run` length/exp asserts (`feistel.rs:110-121`) — length validated (step 2), `iterationExp` structurally 4-bit from parse; `split_at(4)` on D (`mod.rs:447`) — D length ≥16 > 4. No malformed input reaches a panic.

**Security invariants (§2) — honored.** Digest gate cannot be bypassed via Recover: Recover is offered only when `MemberThreshold>1 || GroupThreshold>1`, which forces at least one threshold≥2 layer → digest runs (verified the 1-of-1-both-layers case is the lone-share path that never enters Recover). Wrong-passphrase-no-error correctly preserved (digest gates Shamir reconstruction, not decryption; UI must not claim verification — §2.3/§5.4 comply). SLIP-39 vs BIP-39 passphrase distinction maintained (distinct stages/algorithms/labels). No `math/big` in new crypto; secret/passphrase never engraved-except-confirmed / never over NFC.

**Engrave model (§3) — correct.** Recon independently verified `from_seed(MS)` reproduces vector #1's `bip32_xprv` (MS = BIP-32 seed in the standard); the constellation MS = BIP-39-entropy convention governs; engraving via `bip39.New → backupWalletFlow` is sound under that documented assumption.

**Vector set (§7) — accurate.** Confirmed against `slip39_vectors.json` (45 entries): idx 0 = 1-of-1/128 (T==1 no-digest), idx 3 = 2-of-3/128, idx 17 = group-threshold, idx 35 = 256-bit/33-word multi-group, idx 42 = extendable/ext=1, idx 12 = invalid digest, idx 1/2/5/13 as stated. All VALID use `"TREZOR"`. Rust oracle test suite is green.

**GUI (§5):** `confirmSLIP39Flow` bool→enum change is safe for existing tests (`TestConfirmSLIP39Render` discards the return; `TestEngraveSLIP39BackoutRecognized` Back→true preserved). Button2-drain pattern correctly mirrors `codex32_polish.go:107-111`. `backupWalletFlow(ctx,th,m)` is reusable (same `gui` package). `inputSLIP39Flow` title param threads cleanly; gui.go needs no new import.

---

## CRITICAL

**None.** The crypto port is faithful, panic-safe, and the digest gate is unbypassable on the Recover path.

---

## IMPORTANT

**I1 — "All valid lengths" (§1 in-scope) is unreachable on-device; contradicts §5 + the deferred entry FOLLOWUP.**
The only path to a `slip39words.Share` reaching `engraveSLIP39`/Recover is menu case 3, which calls `inputSLIP39Flow(ctx, th, emptySLIP39Mnemonic(20), 0)` — hardcoded to **20 words** (`gui.go:2035-2036`). SLIP-39-over-NFC is disabled. The first share can therefore only be 128-bit. §5.2 step 2 reuses `inputSLIP39Flow` for subsequent shares with no length selection and no stated sizing, so the natural `len(first.Mnemonic)`=20 makes the entire collection loop 20-word-bound. Net: the ported 256-bit/33-word crypto and the §7 idx-35 path are **dead on-device** until the explicitly-out-of-scope `seedhammer-slip39-cycleC-all-lengths` FOLLOWUP lands. §1 ("All valid share lengths") and §5 (20-word entry) directly contradict each other.
**Required fix:** either (a) pull first-share-length widening into this cycle (the spec itself calls it "trivial"); or (b) re-scope: state plainly that on-device recovery is 20-word/128-bit this cycle (crypto + ParseShare support all lengths for the future widening + cross-check oracle), reword the §1 headline, and remove/relabel the GUI 33-word expectation since the GUI recover test cannot drive a non-20-word set. Also specify in §5.2 that each collected share is sized to `len(first.Mnemonic)`.

**I2 — §7 idx-9 ("group-thr>count") has no matching sentinel in §4/§6 (spec-internal contradiction; undefined test expectation).**
The Rust rejects `group_count < group_threshold` at **parse** via `GroupThresholdExceedsCount` (`share.rs:250`). The firmware's current `ParseShare` has no such check, and the spec's §4.6 (`ParseShare` changes) and §4.3 (Combine steps 2–7) do **not** add it. §6's taxonomy has only the cross-share "group count mismatch" sentinel, not a structural "threshold-exceeds-count" one. So idx 9 will fall through to whatever Combine produces (most likely group-level `errInsufficientShares`), and the §7 TDD vector asserts against an expectation that doesn't exist in the design — the test will either be written wrong or weakened to pass.
**Required fix:** pick one and make §4/§6/§7 agree — either add the `groupCount >= groupThreshold` structural check + a dedicated sentinel to `ParseShare` (Rust-faithful), or change §7's idx-9 expected error to the actual fallback (`errInsufficientShares`, group-level) and document that the firmware catches this class at combine, not parse.

---

## MINOR

- **M1 — `slip39_polish.go` needs a new `bip39` import** (for the `bip39.Mnemonic` return type and `bip39.New`). §5/§8 emphasize gui.go import hygiene but omit this; not wrong, just unstated. (`backupWalletFlow` needs no import — same `gui` package.)
- **M2 — §5.2 doc-comment says `("", false)` on abort, but the signature returns `(bip39.Mnemonic, bool)`** whose zero value is `nil`, not `""` (copy-paste from codex32's `string` return). Fix the comment to `(nil, false)`.
- **M3 — §4.5 pseudocode lists the hash first** (`pbkdf2.Key(hmacSHA256, password=…, …)`); the real API (already used at `bip39.go:225`) is `pbkdf2.Key(password, salt, iter, keyLen, h)`. Pseudocode only — clarify ordering in the plan.
- **M4 — §7 "existing guards stay green" should explicitly note `slip39/share_test.go` inverts:** the current 33-word→`errUnsupportedSize` assertion (`share_test.go:65-66`) and the `{errUnsupportedSize, "256-bit not supported"}` Describe case (line 87) must be updated/removed (33 words now parses; symbol dropped per §4.6). §8's `slip39/*_test.go` "modify" covers it, and §7's stay-green list correctly excludes these, but call it out so the TDD pass doesn't trip on the removed symbol failing to compile.
- **M5 — §5.2 collection stop-condition is under-specified.** "Collect until sufficient" (exactly groupThreshold complete groups) is correct, but the spec doesn't define behavior when a user enters a share from a new/extra group, or over-fills a group, before the needed groups complete. Combine's strict-equal counts will reject over-collection, but the flow must stop precisely at sufficiency to avoid it. The implementation plan should pin the exact per-frame stop logic. (Plan-level, not spec-blocking.)
- **M6 — note `bip39.New`'s panic precondition** is satisfied only by Combine's length guarantee (value∈{16,20,24,28,32}); worth a one-line defense-in-depth note since `bip39.New` panics on invalid length.

---

## Verdict

**NOT GREEN — 0 Critical / 2 Important.**

Required before GREEN: resolve **I1** (the "all valid lengths" headline is unreachable through the only on-device entry path — either widen first-share entry this cycle or re-scope §1/§5/§7 to 20-word and size the collection loop to `len(first.Mnemonic)`) and **I2** (§7's idx-9 expectation has no matching sentinel in §4/§6 — make ParseShare add the structural check + sentinel, or correct the test expectation to the actual combine-level fallback). The crypto port itself (§4) is faithful, panic-safe, and the digest gate is unbypassable — no Critical findings. Fold I1/I2 (+ the minors), persist this review verbatim, and re-dispatch.
