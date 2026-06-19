# T7b SPEC — R0 review (round 1) — VERBATIM agent report

**Agent:** `ab0ef0801671277c2` (adversarial opus architect; re-verified the BIP-85 vector LIVE again this round). **Fork HEAD:** `82d46b3`. **Spec commit reviewed:** `ff13dbe`. **Date:** 2026-06-19.
**Verdict:** NOT GREEN (0C / 1I / 3M). The 1 Important (I-A') was a fold-introduced citation defect; folded in round 2. Persisted per the R0 gate discipline.

---

# T7b SPEC — R0 review (round 1)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** 82d46b3  **Spec commit:** ff13dbe  **Verdict:** NOT GREEN (0C / 1I / 3M)

## Round-0 resolution check

- **I-A (child fingerprint on the plate): RESOLVED (with one residual citation defect → see new I-A').**
  The fold is present and correct in all four required places: §2 engrave bullet (pins `mfp` MUST be `masterFingerprintFor(childMnemonic, &chaincfg.MainNetParams, "")`, NEVER the master's, and states `backupWalletFlow` is NOT reusable verbatim because of the passphrase-fp picker branch); §3 fact; §5.4 acceptance (asserts `== child fp` AND `!= master fp`); §6-I5. **Verified against source:** `masterFingerprintFor` EXISTS (`gui/gui.go:485`) — the spec cited `:2020`, WRONG (see I-A'/M-4). `engraveSeed` stamps `MasterFingerprint: mfp` at `gui/gui.go:475` — CORRECT. `masterFingerprintFor(childMnemonic, MainNet, "")` IS the bare-seed fp: `deriveMasterKey(m, net, "")` → `bip39.MnemonicSeed(m, "")` → pbkdf2 = standard empty-passphrase BIP-39 seed; `bip32.Fingerprint(mk.ECPubKey())` = the BIP-32 master fp of that bare seed; `backupWalletFlow` itself uses the identical bare call (`gui.go:2019`). Passphrase-fp picker branch is real at `gui.go:2026-2049` (spec cited `:2026-2047`, off by 2, substantively correct). Algorithmic resolution sound; only the citation is defective.

- **I-B (index widget): RESOLVED.** Consistent across §2 (bounded `ChoiceScreen` 0–9, default 0; free/large → FOLLOWUP), §3 (RESOLVED note), §5.3, §6-I2, §7-risk-4b. **Verified:** NO reusable free-form numeric-entry widget exists — every `stepper`/`Stepper`/`StepperConfig` hit is the engraving MOTOR (`gui/gui.go:2805`, `gui/qa.go:19`); `seedEntryFlow` + xpub picker are `ChoiceScreen`-only. Bounded `ChoiceScreen` index is validated-by-construction and minimal. Sound.

- **M-1 (ECPrivKey error): RESOLVED.** §2/§3 use `pkey, err := xkey.ECPrivKey()` and propagate. Verified `func (k *ExtendedKey) ECPrivKey() (*btcec.PrivateKey, error)` (`hdkeychain/extendedkey.go:546`); `ErrNotPrivExtKey` unreachable on master+hardened walk.

- **M-2 (master vs child word-count axes): RESOLVED.** §2 clarifies `seedEntryFlow` = 12/24 MASTER axis; {12,18,24} = independent CHILD axis; "Do NOT add 18-word master entry." Verified `seedEntryFlow` offers `[]int{12, 24}` (`gui/derive_xpub.go:89`).

- **M-3 (bip39.New bounds): RESOLVED.** §5.1 notes `entLen∈{16,24,32}` satisfies `bip39.New`'s bounds (`bip39/bip39.go:228-234`). Verified.

## New findings (this round)

## Critical
None. The derivation algorithm is canonical and re-verified LIVE this round: a throwaway test walking the canonical master xprv `m/83696968'/39'/0'/12'/0'` through the in-tree `bip85.Entropy` + `hdkeychain` yields entropy `6250b68daf746d12a24d58b4787a714b` → `GIRL MAD PET GALAXY EGG MATTER MATRIX PRISON REFUSE SENSE ORDINARY NOSE`, byte-identical to the canonical BIP-85 vector and to round-0's live proof. Throwaway removed; tree clean. The I-A child-fp resolution is algorithmically correct (bare-seed fp, never the master's). No wrong-child / wrong-identifier / secret-leak / build-break.

## Important

**I-A' — `masterFingerprintFor`'s signature is mis-cited as error-free, and its line number is wrong; the load-bearing child-fp call as written in §3/§5.4 would not compile, and the omission is internally inconsistent with the spec's own M-1 fold.**
Actual signature: `func masterFingerprintFor(m bip39.Mnemonic, network *chaincfg.Params, password string) (uint32, error)` (`gui/gui.go:485`) — a TWO-value `(uint32, error)` return. The spec §3 cites it as `masterFingerprintFor(…) uint32` (single value) AND at the wrong location `gui/gui.go:2020` (actual `:485`). §5.4 / §6-I5 write the equality as if it returns a bare `uint32`. This is the SAME `(value, error)`-must-propagate class the round-0 review caught for `ECPrivKey` and folded as M-1 — but the fold pinned error-propagation ONLY for `ECPrivKey` and left `masterFingerprintFor` presented as infallible. An implementer following §3 literally writes a non-compiling call (Go forbids dropping the second return without `_`) or blindly discards the error on the very fp path §5.4 makes load-bearing. Like `ECPrivKey`, the error is effectively unreachable (the child mnemonic is freshly minted by `bip39.New`, always valid; `deriveMasterKey` fails only at ~1-in-2^127), but the spec must pin the same discipline: `mfp, err := masterFingerprintFor(childMnemonic, &chaincfg.MainNetParams, ""); propagate err`. Fix: correct the signature to `(uint32, error)`, the line to `gui/gui.go:485`, add the M-1-style error-propagation note in §3 / §5.4 / §6-I5. (Important, not Minor: a cited API on the security-critical child-fp path, wrong enough to derail a literal implementation and contradicting an already-accepted fold; trivial to fix.)

## Minor

**M-4 — Stale line citations after the fold (non-load-bearing).** Besides I-A's `:2020`→`:485`: §3 cites the passphrase-fp picker at `gui.go:2026-2047` (actual `2026-2049`); the enum at `gui.go:148-154` (actual const block 147–154, members 148–153); `seedEntryFlow`'s set implied at `:83` (actual `:89`); title switch `:1667-1678` (actual body 1668–1678). All reference the correct symbols and the correct `engraveMultisig` upper bound; none change the plan. Tighten on the same pass as I-A'.

**M-5 — `masterFingerprintFor` reuse inherits unscrubbed internal seed/master, undocumented in §4's two-secret spine (informational; not a defect).** Computing the child fp via `masterFingerprintFor(childMnemonic, …)` internally allocates `seed := bip39.MnemonicSeed(child, "")` and `mk := NewMaster(seed)` in `deriveMasterKey` (`gui.go:194-203`) and neither is scrubbed — but this is PRE-EXISTING shipped behavior the reused Backup-Wallet path (`backupWalletFlow` → `masterFingerprintFor(mnemonic, MainNet, "")`, `gui.go:2019`) already exhibits for EVERY seed backup. Scrubbing would require forking the helper (net-new, against "reuse the primitive"). §4's spine is correct for the buffers the flow OWNS. Recommend one sentence in §4 acknowledging the helper's internals are out of the flow's scrub scope, identical to the shipped Backup-Wallet path. Documentation polish only.

**M-6 — `seedEntryFlow` cite `:82` vs `:83`/`:89`.** §3 cites the func at `derive_xpub.go:82` (signature correct); the `[]int{12,24}` set is at `:89`. The §2 M-2 note attributes the set to `:83` (off by 6). Immaterial; fix on the same pass.

## Verified-correct (fresh checks)
- **Live BIP-85 vector re-confirmed this round** (throwaway test, removed): canonical master → `m/83696968'/39'/0'/12'/0'` → `6250b68d…` (leading 16 B) → `GIRL MAD PET GALAXY…NOSE`. Byte-identical to canonical + round-0. `bip39.New(ent[:16])` did not panic.
- **Bare-seed fp confirmed:** `masterFingerprintFor(child, MainNet, "")` = bare BIP-32 master fp of the child (`deriveMasterKey`→`MnemonicSeed(child,"")`→pbkdf2; `bip32.Fingerprint(ECPubKey)`). I-A's resolution is the right call; no subtlety with `password==""`.
- **"reuse engraveSeed PRIMITIVE, NOT backupWalletFlow verbatim" is now crisp and CORRECT:** `engraveSeed` (`gui.go:461`) is a pure primitive stamping `MasterFingerprint: mfp` (`:475`) with no passphrase logic; `backupWalletFlow` (`:2014`) wraps it with the bare-fp→passphrase-picker loop (`:2018-2061`). Calling `engraveSeed` directly with the bare child fp + skipping the picker is exactly right.
- **0–9 bound sensible + consistent** across §2/§3/§5.3/§6-I2/§7-4b; covers the common BIP-85 child-index range, validated-by-construction, zero net-new widget — consistent with §1 "least net-new". (0–23 equally sound; wider/free → FOLLOWUP.)
- **All 8 lockstep sites accurate at HEAD `82d46b3`**, all referencing `engraveMultisig` as the upper bound: enum `gui.go:147-153`; dispatch `:1492-1514` under `act.scan==nil`/`obj==nil` (`:1491`); left-wrap `m.prog=engraveMultisig` `:1642`; right-wrap `m.prog>engraveMultisig` `:1649`; title switch `:1668-1678`; `npage := int(engraveMultisig)+1` `:1852`; `layoutMainPlates` case+`panic("invalid page")` `:1860-1867`; `npages` `:1871`. Inserting between `engraveMultisig` and `qaProgram` keeps `qaProgram` non-navigable. Correct.
- **Scan→derive structurally unreachable:** new program dispatches only under `obj==nil`; `seedEntryFlow` typed-only, never touches `gui/scan.go`. I-3/I-4 sound.
- **biptool reference** (`cmd/biptool/main.go:142-189`): unhardened-reject (`:142-145`), `pkey, err := xkey.ECPrivKey()` (`:152`), `bip85.Entropy(pkey.Serialize())`, word guard `n<12||24<n||n%3!=0` (`:179`), path validation 5-elem (`:183`), `entLen=(n*11-n/3)/8` (`:188`), `bip39.New(seed[:entLen])` (`:189`). Matches spec §2/§3 exactly.
- **I-1..I-8 → §5 test mapping intact**; no duplication/contradiction after the fold; the {12,18,24} child vs {12,24} master split disambiguated everywhere.
- `wipeBytes` (`slip39_polish.go:330`), `showError` (`slip39_polish.go:22`), `ChoiceScreen.Choose(ctx,th)(int,bool)` (`gui.go:1363`), `bip85.Entropy`/`PathRoot`/`macKey`/panic-if-not-32 (`bip85/bip85.go:11,13,15,17`) — present at cited lines.

## Bottom line
NOT GREEN — 0 Critical, 1 Important, 3 Minor. The folds for I-A, I-B, M-1..M-3 are all genuinely resolved, introduced no algorithmic drift, and the BIP-85 derivation remains canonical (re-verified live). The ONE remaining blocker is I-A' — a fold-introduced inconsistency: the spec mis-cites `masterFingerprintFor` as a single-return `uint32` at `:2020`, when it is `(uint32, error)` at `gui/gui.go:485`. Because §5.4/§6-I5 make this call load-bearing for child-fp correctness and the spec ALREADY pins the identical `(value,error)`-propagate discipline for `ECPrivKey` (M-1), the gap is internally inconsistent and would derail a literal implementation. Fix is trivial: correct the signature/line, add the M-1-style error-propagation note; sweep the stale cites (M-4/M-6); add the one §4 sentence (M-5). Fold → persist → re-dispatch for the GREEN confirmation. No code before GREEN.
