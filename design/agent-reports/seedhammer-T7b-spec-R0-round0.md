# T7b SPEC — R0 review (round 0) — VERBATIM agent report

**Agent:** `aef172c817302cd8d` (adversarial opus architect; RAN a live BIP-85 derivation in the fork module reproducing the canonical 12-word test vector). **Fork HEAD:** `82d46b3`. **Spec commit:** `b77db90`. **Date:** 2026-06-19.
**Verdict:** NOT GREEN (0C / 2I / 3M). Persisted before folding per the R0 gate discipline.

---

# T7b SPEC — R0 review (round 0)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** 82d46b3  **Spec commit:** b77db90  **Verdict:** NOT GREEN (0C / 2I / 3M)

## BIP-85 protocol re-verification (MANDATE #1)
Re-derived against the actual in-tree code (not the recon) by executing a throwaway program inside the fork module that walks the canonical BIP-85 master xprv and prints the leaf entropy/mnemonic. Throwaway removed; tree clean.

(a) **Root = m/83696968'** — canonical. In-tree: `bip85.PathRoot = 83696968 + 0x80000000` (`bip85/bip85.go:11`). Spec §3/§6-I1 restate `m/83696968'`. MATCH.
(b) **BIP-39 app path = m/83696968'/39'/{lang}'/{words}'/{index}', English=0'** — canonical. In-tree validation: `len(path)!=5 || path[1]!=39+h || path[2]!=0+h || path[3]!=uint32(n)+h` (`cmd/biptool/main.go:183-184`). Spec §2/§3/§6-I1 restate `m/83696968'/39'/0'/{words}'/{index}'`, English=0'. MATCH.
(c) **Every element hardened** — canonical. In-tree: rejects any `p < hdkeychain.HardenedKeyStart` (`main.go:142-145`). Spec §2/§4/§6-I1 say "FULLY hardened every element (≥ HardenedKeyStart)". MATCH.
(d) **HMAC-SHA512, key "bip-entropy-from-k", over the 32-byte leaf privkey** — canonical. In-tree: `hmac.New(sha512.New, []byte("bip-entropy-from-k")); mac.Write(privkey)` with `panic` if `len!=32` (`bip85/bip85.go:13,16-22`). Spec §3 restates exactly. MATCH.
(e) **entLen=16/24/32 for 12/18/24; child = LEADING entLen bytes** — canonical. In-tree: `bip39.New(seed[:entLen])` (`main.go:189`) = leading bytes. Spec §2/§3/§5/§6-I1 emphasize `hmacOut[:entLen]`, "NOT trailing". MATCH. **Live proof:** in-tree walk of the canonical master at `m/83696968'/39'/0'/12'/0'` yields `ent[:16] = 6250b68daf746d12a24d58b4787a714b` → mnemonic `GIRL MAD PET GALAXY EGG MATTER MATRIX PRISON REFUSE SENSE ORDINARY NOSE` — byte-identical to the canonical bip-0085.mediawiki 12-word test vector.
(f) **entLen=(words*11-words/3)/8 → 16/24/32** — verified by execution: 12→16 (128b), 18→24 (192b), 24→32 (256b). MATCH.

The in-tree `bip85_test.go` vector (privkey `cca20ccb…` → entropy `efecfbcc…`) is the canonical standalone Entropy vector and reproduces verbatim. No divergence anywhere. The spec's protocol restatement is fully canonical — there is NO silent-wrong-child risk in the specified algorithm.

## Critical
None. The derivation algorithm, path, truncation direction, HMAC, and entLen are all canonical and in-tree-verified. The two-secret scrub spine, steel-only channel, typed-only master, and deterministic-no-CSPRNG invariants are all sound and correctly grounded in shipped patterns.

## Important

**I-A — `engraveSeed`'s `mfp` argument for the CHILD plate is unspecified; left as-is it engraves a WRONG/misleading fingerprint.**
`engraveSeed(params, m, mfp uint32)` (`gui/gui.go:461`) stamps `MasterFingerprint: mfp` onto the steel (`:475`). In `backupWalletFlow` the mfp is computed from the seed being engraved via `masterFingerprintFor(mnemonic, MainNet, "")` (`gui.go:2020`), with an optional passphrase-fingerprint picker (`:2026-2047`). The SPEC §2 says "engrave the child via engraveSeed (reuse verbatim)" and §3 cites the signature, but NOWHERE states which `mfp` to pass for the child. The child is a *bare* BIP-39 mnemonic with no passphrase, so the only correct value is the CHILD's own fingerprint: `masterFingerprintFor(childMnemonic, &chaincfg.MainNetParams, "")`. If the implementer passes the parent/master fingerprint (the value already in scope from the typed master), the plate gets a fingerprint that does NOT match the engraved child words — a permanently-wrong identifier on the backup, and a recurrence of the exact "wrong key on a permanent backup" class the codebase already burned an R0-Critical on (`derive.go:46-49`). The spec must pin: compute and engrave the CHILD's own bare-seed fingerprint; do NOT reuse the master fp. (Also decide explicitly whether the child plate offers the passphrase-fp picker at all — it should NOT, since the child is bare; §2-OUT already defers "passphrase ON THE CHILD," so the engrave step must skip the `backupWalletFlow` passphrase branch and call `engraveSeed` with the bare child fp directly. The spec's "reuse engraveSeed verbatim" wording is fine; the surrounding `backupWalletFlow` is NOT reusable verbatim because of this branch.)

**I-B — Index-entry widget is OPEN, but NO reusable numeric-entry widget exists in the tree; this is a real scope hole the spec leaves to the plan without bounding the fallback.**
§3 says "confirm an existing numeric-entry/stepper widget (the `stepper` pkg, or the xpub picker's path-component entry) before assuming." Verified: there is NO such widget. Every `stepper`/`Stepper` hit is the engraving MOTOR config (`engrave.PlanEngraving(params.StepperConfig, …)`, `gui.go:2805`, `qa.go:19`) — unrelated to UI. The xpub picker has NO path-component numeric entry (`derive_xpub.go`/`singlesig.go` only offer `ChoiceScreen` wallet-type picks; "Advanced" is a fixed-set choice, not free numeric entry). So the only shipped input primitives are `ChoiceScreen` (fixed small set) and the BIP-39 word keypad. The spec defers this to the plan, which is acceptable for the algorithm, BUT it must bound the fallback NOW so the plan can't over-build: either (1) a bounded `ChoiceScreen` index set (e.g. 0–9 or 0–23) — minimal, no net-new widget, recommended; or (2) accept that a free-form numeric keypad is net-new UI (~80-150 LOC, contradicting the "nicety / least net-new" framing and the S–M sizing). The spec's §7-risk-4 names this but the §7 framing ("don't over-build") understates it: there is literally nothing to reuse, so leaving the bound unstated risks the plan defaulting to a new keypad. Pin "default 0, bounded ChoiceScreen small-set index" as the spec-level decision (full free index can be a FOLLOWUP). This is Important because an unbounded index entry is the one place the picker could still mint an out-of-spec child if a custom keypad mis-validates — §6-I2 only covers word-count/app bounds, not index-entry validation.

## Minor

**M-1 — Helper must handle `ECPrivKey()`'s error return; spec §3 paraphrases biptool as `pkey=xkey.ECPrivKey().Serialize()` (no error).** Actual biptool: `pkey, err := xkey.ECPrivKey(); if err != nil {…}` then `bip85.Entropy(pkey.Serialize())` (`main.go:153-157`). `ECPrivKey()` returns `(*PrivateKey, error)` and errors only on `ErrNotPrivExtKey` (`hdkeychain/extendedkey.go:546-549`); since the T7b walk is master+hardened-only it can't fire, and `Serialize()` is always 32 bytes so `bip85.Entropy`'s panic is unreachable — but the helper must still propagate the error (don't `.Serialize()` a nil), matching the reference. Note in §3/acceptance.

**M-2 — `seedEntryFlow` offers only 12/24-word MASTER entry (`derive_xpub.go:83`, `[]int{12,24}`), no 18.** This is correct and intended (the master can be 12 or 24; the CHILD word-count axis {12,18,24} is independent), but the spec conflates the two `{12,18,24}` lists rhetorically — §2 IN says "word count ∈ {12,18,24}" for the picker while reusing `seedEntryFlow` (12/24) for the master. Add one sentence clarifying these are two independent axes so the implementer doesn't try to add 18-word master entry.

**M-3 — `bip39.New` panic bounds are satisfied but unstated.** `bip39.New` panics if `len(entropy) < 16 || 32 < len || %4 != 0` (`bip39/bip39.go:228-234`). entLen ∈ {16,24,32} all pass. Worth a one-line acceptance note (the §5.1 leading-bytes test implicitly covers it).

## Verified-correct
- All 8 lockstep sites accurate at HEAD `82d46b3`: enum `gui.go:147-154`; dispatch `:1492-1514` (`act.scan==nil` guard); left-wrap `:1640-1643`; right-wrap `:1648-1651`; title `:1667-1678`; `npage` `:1852`; `layoutMainPlates` case+`panic("invalid page")` `:1860-1867`; `npages` `:1871`. Sites left/right/npage/npages all reference `engraveMultisig` as the upper bound → repoint to `bip85Derive`. Inserting between `engraveMultisig` and `qaProgram` keeps `qaProgram` non-navigable (absent from title switch, layoutMainPlates case, and both wrap bounds; reachable only via the NFC debug command). Correct move.
- Nav-test precedent: `gui/multisig_program_test.go` is exactly 2 tests (`…ProgramNavigable` + `…LeftWrap`); the spec's "2 new nav-tests + repoint prior-program carousel counts" is accurate.
- `engraveSeed(params, m, mfp) (Plate, error)` → `qr.Encode(string(seedqr.QR(m)), qr.M)` + `backup.EngraveSeed` (`gui.go:461-482`). Steel-only, no extended key in the artifact — confirmed (I-4 channel sound).
- `bip39.New(entropy []byte) Mnemonic` (`bip39/bip39.go:228`).
- `seedEntryFlow` typed-only (`derive_xpub.go:82`); never touches `gui/scan.go`. The scan dispatch returns a `bip39.Mnemonic` (`scan.go:62`) only into `engraveObjectFlow` with `act.scan != nil` — the new `bip85Derive` runs only under `act.scan == nil`, so scan→derive is structurally unreachable (I-3 typed-only / I-4 no-NFC sound).
- `deriveAccountXpub` scrub discipline incl. the capture-before-zero R0-C1 note (`derive.go:19-52`) — correct pattern to mirror; BUT note it walks to a PUBLIC neutered key, so the T7b PRIVATE-leaf walk (`ECPrivKey().Serialize()`) is genuinely net-new, not a `deriveAccountXpub` call. Spec §1/§2 correctly treat it as re-created-from-biptool, not reused.
- `wipeBytes` (`slip39_polish.go:330`), `ChoiceScreen.Choose(ctx,th) (int,bool)` (`gui.go:1363`).
- Picker bounds match biptool's guard `n<12||24<n||n%3!=0` (`main.go:179`) exactly → {12,18,24} (I-2 sound for word-count; see I-B for the index axis).
- entLen formula and the leading-bytes child both reproduced live against the canonical spec vector.

## Bottom line
NOT GREEN — 0 Critical, 2 Important (I-A: unspecified/likely-wrong child `mfp` on the engraved plate, a wrong-identifier-on-permanent-backup risk; I-B: no reusable numeric-entry widget exists, so the OPEN index-entry decision must be bounded at spec stage to a `ChoiceScreen` small-set or it risks both over-build and an unvalidated index path). The BIP-85 protocol is fully canonical and in-tree-verified — the algorithm core is correct and carries zero divergence risk. Fold I-A (pin: engrave the CHILD's own bare fingerprint, skip the passphrase-fp branch) and I-B (pin: bounded small-set index `ChoiceScreen`, default 0, free numeric → FOLLOWUP), address M-1..M-3, persist this review, and re-dispatch. The 3 Minors should be folded in the same pass.
