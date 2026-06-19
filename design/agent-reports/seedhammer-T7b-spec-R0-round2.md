# T7b SPEC — R0 review (round 2) — VERBATIM agent report

**Agent:** `ad8b51ad335cb068d` (adversarial opus architect; re-verified the BIP-85 vector LIVE a third time). **Fork HEAD:** `82d46b3`. **Spec commit reviewed:** `c629c87`. **Date:** 2026-06-19.
**Verdict:** GREEN (0C/0I). 2 cosmetic Minors swept post-GREEN. Persisted per the R0 gate discipline; cleared for the implementation-plan phase.

---

# T7b SPEC — R0 review (round 2)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** 82d46b3  **Spec commit:** c629c87  **Verdict:** GREEN (0C/0I)

## Round-1 resolution check

- **I-A' (masterFingerprintFor signature/line + error propagation): RESOLVED.**
  Source confirmed: `func masterFingerprintFor(m bip39.Mnemonic, network *chaincfg.Params, password string) (uint32, error)` at `gui/gui.go:485` — a genuine TWO-value `(uint32, error)` return (returns `0, err` on derive/pubkey failure, else `bip32.Fingerprint(pkey), nil` at `:494`). `engraveSeed` stamps `MasterFingerprint: mfp` at `gui/gui.go:475` (confirmed). All FOUR required locations now state the `(uint32, error)` two-value return + error propagation, and NO place implies a bare `uint32`:
    - §2 (line 18): "`mfp, err := masterFingerprintFor(childMnemonic, &chaincfg.MainNetParams, "")` (signature `(uint32, error)`, `gui/gui.go:485` — propagate `err` per R0-A1/M-1)".
    - §3 (line 32): full signature `masterFingerprintFor(m bip39.Mnemonic, network *chaincfg.Params, password string) (uint32, error)` (`gui/gui.go:485`) — "note the `(uint32, error)` return: propagate the error … `mfp, err := masterFingerprintFor(childMnemonic, &chaincfg.MainNetParams, "")`".
    - §5.4 (line 47): "the value from `masterFingerprintFor(childMnemonic, MainNet, "")` (a `(uint32, error)` call — handle the err; the CHILD's own bare fp)".
    - §6-I5 (line 58): "`masterFingerprintFor(childMnemonic, MainNet, "")` — `(uint32, error)`, propagate the err".
  Grep for `masterFingerprintFor` across the whole spec returns exactly 5 mentions (the 4 above + the §4 M-5 note); all consistent on `(uint32, error)` / `:485` / propagate-err. The stale `:2020` cite is fully gone. Mirrors the M-1 `ECPrivKey` discipline exactly. No new drift.

- **M-4/M-6 (stale cites): RESOLVED / no residual drift worth blocking.**
    - enum const block `:147-153` (members `148–153`, `engraveMultisig`@`:152`, `qaProgram`@`:153`) — spec cites `:147-153` — CORRECT.
    - title switch body `:1668-1678` (cases `1668`→`engraveMultisig`@`1677`, blank `1678`) — spec cites `:1668-1678` — CORRECT.
    - passphrase-fp picker: §3 (line 32) cites `gui.go:2026-2049` — CORRECT. §2 (line 18) still cites `gui.go:2026-2047` — a stale ±2 from round 1; references the right symbol/branch, non-load-bearing. MINOR (see below).
    - `seedEntryFlow` set: the round-1 M-6 `:83`/`:89` complaint is GONE — the §2 M-2 note no longer cites a line for the set; `seedEntryFlow`@`derive_xpub.go:82` (§3) correct, `[]int{12,24}`@`:89` confirmed.

- **M-5 (scrub-scope note): RESOLVED.** §4 now carries the note that `masterFingerprintFor`→`deriveMasterKey`'s internal seed/master ExtendedKey are out of the flow's scrub scope, explicitly "identical to the shipped Backup-Wallet path" and "not a regression."

## New findings (this round)

## Critical
None.

## Important
None.

## Minor
- **m1 (residual cite drift, §2 vs §3 self-inconsistency):** §2 cites the passphrase-fp picker branch as `gui.go:2026-2047`; §3 cites the same branch as `gui.go:2026-2049`. Source: the picker block ends at `:2049`. The §2 `:2047` is the round-1 stale value; §3 is correct. Both point at the correct symbol/branch that MUST be skipped; the plan is unaffected. Cosmetic only — does NOT block. [Swept post-GREEN.]
- **m2 (off-by-1, non-load-bearing):** §3 cites `ChoiceScreen.Choose` at `gui/gui.go:1362`; actual `func (s *ChoiceScreen) Choose(...) (int, bool)` is `:1363` (`:1362` is the doc-comment line). Immaterial; correct symbol/signature. [Swept post-GREEN.]

Neither minor rises to Important: both are correct-symbol / ±2-line citations with no effect on implementability or correctness.

## Verified-correct
- **BIP-85 derivation canonical — RE-VERIFIED LIVE this round** (throwaway module against the in-tree `bip85.Entropy` + `hdkeychain` + `bip39.New`, then removed): canonical master → `m/83696968'/39'/0'/12'/0'` → entropy leading-16 `6250b68daf746d12a24d58b4787a714b` → `GIRL MAD PET GALAXY EGG MATTER MATRIX PRISON REFUSE SENSE ORDINARY NOSE`. Byte-identical to the spec vector (§5.1) and to round-0/round-1. `bip39.New(ent[:16])` did not panic. The fold did NOT touch the algorithm text (path / LEADING-bytes truncation / `entLen=(n*11-n/3)/8` all unchanged); algorithm remains canonical.
- **`engraveSeed`** (`gui.go:461`) is a pure primitive stamping `MasterFingerprint: mfp` (`:475`) via `backup.EngraveSeed`, no passphrase logic — confirmed. "Reuse the PRIMITIVE, skip `backupWalletFlow`'s passphrase-fp picker" is correct: `backupWalletFlow` (`:2014`) wraps it with the bare-fp call (`:2020`) + the picker loop (`:2026-2049`).
- **Bare child fp is the right call:** `masterFingerprintFor(child, MainNet, "")` = BIP-32 master fp of the bare child seed (`deriveMasterKey`→`MnemonicSeed(child,"")`→pbkdf2; `bip32.Fingerprint(ECPubKey)`); `backupWalletFlow` itself uses the identical bare call at `:2020`. No `password==""` subtlety.
- **All 8 lockstep sites accurate @ `82d46b3`**, all referencing `engraveMultisig` as the upper bound: enum `:147-153`; dispatch switch `:1492-1514` under `obj==nil` (`:1491`); left-wrap `m.prog=engraveMultisig` `:1642`; right-wrap `m.prog>engraveMultisig` `:1649`; title switch `:1668-1678`; `npage:=int(engraveMultisig)+1` `:1852`; `layoutMainPlates` case + `panic("invalid page")` `:1860-1867`; `npages:=int(engraveMultisig)+1` `:1871`. Inserting between `engraveMultisig`/`qaProgram` keeps `qaProgram` non-navigable. Correct.
- **biptool reference matches §2/§3 exactly:** unhardened-reject `:142-145`, `pkey, err := xkey.ECPrivKey()` `:153-156`, `bip85.Entropy(pkey.Serialize())` `:157`, word guard `n<12||24<n||n%3!=0` `:179`, 5-elem path validation `:183`, `entLen=(n*11-n/3)/8` `:188`, `bip39.New(seed[:entLen])` `:189`.
- **`ECPrivKey() (*btcec.PrivateKey, error)`** at `hdkeychain/extendedkey.go:546` — M-1 fold sound.
- **`bip85.Entropy`** (`bip85/bip85.go:16`): HMAC-SHA512, key `"bip-entropy-from-k"` (`:13`), `PathRoot=83696968+0x80000000` (`:11`), panic-if-not-32 (`:17-18`). `bip39.New` bounds `16≤len≤32 && len%4==0` (`bip39/bip39.go:228-234`); `entLen∈{16,24,32}` all satisfy → no panic.
- **Helpers present at (effectively) cited lines:** `seedEntryFlow`@`derive_xpub.go:82` (typed-only, `[]int{12,24}`@`:89`), `deriveAccountXpub` scrub discipline `derive.go:19-58` (`defer wipeBytes(seed)`@`:21`, `.Zero()` each intermediate, capture-before-zero), `wipeBytes`@`slip39_polish.go:330`, `showError`@`slip39_polish.go:22`, `bip39.New` usages `ms1_decode.go:33` / `slip39_polish.go:292`.
- **End-to-end implementability walk — no ambiguity:** program entry (insert `bip85Derive` between `engraveMultisig`/`qaProgram`, 8 lockstep sites pinned) → typed master via `seedEntryFlow` (12/24, typed-only, never scan) → child-param picker (app fixed BIP-39; word-count `ChoiceScreen` {12,18,24}; bounded index `ChoiceScreen` 0–9 default 0) → derive helper (fully-hardened path walk → `pkey, err := xkey.ECPrivKey()` propagate → `bip85.Entropy(pkey.Serialize())` → LEADING `entLen` bytes → `bip39.New`) → child-fp (`mfp, err := masterFingerprintFor(child, MainNet, "")` propagate, child's-own-fp not master's) → unskippable child-seed warning → `engraveSeed(params, child, mfp)` direct (skip passphrase-fp picker) → scrub master + child + privkey-serialization + HMAC-output on every exit. Every step has a pinned source symbol/signature, a TDD acceptance test (§5.1–§5.8), and an invariant (§6 I-1…I-8). A competent engineer can implement with no further questions.

## Bottom line
GREEN — 0 Critical / 0 Important. The round-1 I-A' fold is cleanly resolved: `masterFingerprintFor` is now correctly cited as `(uint32, error)` at `gui/gui.go:485` with error-propagation pinned in all four load-bearing locations (§2/§3/§5.4/§6-I5), mirroring the M-1 `ECPrivKey` discipline, with no place still implying a bare `uint32` and no new internal inconsistency introduced by the fix. M-4/M-6 stale cites are corrected; M-5's scrub-scope note is present and accurate. The BIP-85 derivation remains canonical (re-verified LIVE this round, byte-identical vector) — the fold did not touch the algorithm. Two trivial cosmetic ±2-line cites remain (since swept) — correct-symbol, non-load-bearing, Minor only, NOT blocking. The spec is fully implementable end-to-end. This clears the T7b spec for the implementation-plan phase.
