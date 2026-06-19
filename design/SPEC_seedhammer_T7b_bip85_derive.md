# SPEC — T7b: on-device BIP-85 derive-child → engrave (BIP-39 words application)

**Status:** for opus R0 gate (0C/0I before code).
**Fork base:** `82d46b3` (T6 complete). **Fork-side only; no upstream PR.**
**Feeds from:** `design/cycle-prep-recon-T7-seedqr-bip85.md` (BIP-85 protocol facts re-verified vs canonical bitcoin/bips bip-0085.mediawiki + in-tree `bip85_test.go` vector — all accurate). **USER decision (2026-06-19):** "Do all remaining T7" — T7a (SeedQR) confirmed already shipped (Backup Wallet); T7b is the build.

## 1. Why / context
The last T7 nicety. BIP-85 turns ONE master seed into many DETERMINISTIC child seeds (`m/83696968'/…`). The fork already ships the crypto primitive (`bip85.Entropy` = HMAC-SHA512) and a CLI driver (`cmd/biptool`), but it is NOT wired into the touchscreen. This cycle adds a new on-device `bip85Derive` program: typed master seed → pick the child params → derive the child BIP-39 mnemonic → engrave it as a seed-backup plate (words + SeedQR) via the EXACT `engraveSeed` path Backup Wallet uses. Deterministic (no CSPRNG → on-device-feasible). **ZERO new crypto** — `bip85.Entropy` + `hdkeychain` + `bip39.New` + `engraveSeed` are all shipped; the net-new logic is the path-walk/validation/truncation/word-mapping (which lives in `biptool`, not the `bip85` package — it must be RE-CREATED in the flow) + the program/picker/lockstep.

## 2. Scope

### IN
- **A new top-level `bip85Derive` program** (inserted between `engraveMultisig` and `qaProgram`; 8-site lockstep + nav-tests; `qaProgram` stays the last non-navigable sentinel).
- **Typed-only master seed** (`seedEntryFlow`; NEVER scan→derive) + optional passphrase (consistent with T4/T6a).
- **Child-param picker** (`ChoiceScreen`-based): **application fixed to BIP-39 words** (the only engrave-as-words-faithful app); **word count ∈ {12, 18, 24}** (exactly biptool's `n∈{12,18,24}`, `n%3==0`); **index** a hardened non-negative integer (default 0; advanced selection). The picker bounds MUST match biptool exactly — an out-of-spec word count would mint a child that disagrees with biptool/other BIP-85 wallets (silent-wrong-backup class).
- **Derive helper (NET-NEW, re-creates biptool `derive bip39`):** walk `m/83696968'/39'/0'/{words}'/{index}'` — **FULLY hardened** every element (≥ `hdkeychain.HardenedKeyStart`), English=`0'` → at the leaf take the 32-byte EC privkey → `bip85.Entropy(privkey)` (64 B) → `entLen := (words*11 - words/3)/8` (12→16, 18→24, 24→32) → child entropy = the **LEADING** `entLen` bytes (`hmacOut[:entLen]`, NOT trailing) → `bip39.New(childEntropy)` → child `bip39.Mnemonic`.
- **Unskippable "this engraves a CHILD SEED" warning** before engrave (mirrors T4's stub-0 warning pattern): anyone with the child mnemonic controls the child wallet; engrave onto YOUR OWN steel only.
- **Engrave the child via `engraveSeed`** (reuse verbatim — the Backup-Wallet path): child words + standard SeedQR engraved onto steel. NO new engrave primitive.
- **Mainnet-only** (the BIP-85 path is network-agnostic; the child seed-backup artifact is words+SeedQR with no network — consistent with T4/T6).

### OUT (deferred → FOLLOWUPS if wanted)
- Other BIP-85 applications: `32'` (XPRV), WIF, raw hex, the `seed`/RSA apps — engrave-as-words only fits the `39'` (BIP-39) application; the others need a different artifact. CompactSeedQR (dead in the GUI). Non-English BIP-39 languages (biptool hardcodes English=`0'`). Passphrase ON THE CHILD (the child is a bare mnemonic).

## 3. Verified facts (cite source; full detail in the recon)
- **`bip85.Entropy(privkey []byte) []byte`** (`bip85/bip85.go:13,20-22`) — HMAC-SHA512(key=`"bip-entropy-from-k"`, msg=32-byte privkey), 64-byte output; PANICS if `len(privkey)!=32`. `PathRoot = 83696968 + 0x80000000` (`:11`). The package is a THIN primitive — NO path walk / app codes / truncation / word-mapping (those are biptool-side and must be re-created).
- **biptool reference flow** (`cmd/biptool/main.go:137-167` walk, `:174-190` bip39 app): `xkey=NewMaster(seed)`; for each hardened path elem `xkey=xkey.Derive(p)`; `pkey=xkey.ECPrivKey().Serialize()`; `seed=bip85.Entropy(pkey)`; validate path `m/83696968'/39'/0'/{words}'/{index}'` (`:183`); `entLen=(n*11-n/3)/8` (`:188`); `bip39.New(seed[:entLen])` (`:189`). Word guard `n<12||24<n||n%3!=0` (`:179`).
- **Protocol verification (recon B3, vs canonical BIP-85 — ALL ACCURATE):** root `m/83696968'`; bip39 path `…/39'/{lang}'/{words}'/{index}'`, English=`0'`; entropy/word 12→128b/18→192b/24→256b; truncation keeps the LEADING bytes; HMAC key `"bip-entropy-from-k"`; in-tree vector `cca20ccb…`→`efecfbcc…` matches the spec vector identically. **No divergence.**
- **`bip39.New(entropy []byte) bip39.Mnemonic`** (`bip39/bip39.go:228`; used at `gui/ms1_decode.go:33`, `gui/slip39_polish.go:292`). **`engraveSeed(params, m, mfp) (Plate, error)`** (`gui/gui.go:461`) → `qr.Encode(string(seedqr.QR(m)), qr.M)` + `backup.EngraveSeed` (words + SeedQR onto steel).
- **`seedEntryFlow`** (`gui/derive_xpub.go:82`, typed-only), **`deriveAccountXpub` scrub discipline** to mirror (`gui/derive.go:19-58`: `defer wipeBytes(seed)`, `.Zero()` each intermediate, capture-before-zero), **`wipeBytes`** (`gui/slip39_polish.go:330`), **`ChoiceScreen.Choose`** (`gui/gui.go:1362`), **`showError`** (`gui/slip39_polish.go:22`).
- **Lockstep (8 sites @ `82d46b3`):** enum `gui/gui.go:148-154`; dispatch `:1492-1514`; left-wrap `:1640-1643`; right-wrap `:1648-1651`; title `:1667-1678`; `npage` `:1852`; `layoutMainPlates` case+`panic("invalid page")` default `:1860-1867` (MANDATORY); `npages` `:1871`. Sites left/right/npage/npages reference `engraveMultisig` as the current upper bound → repoint to `bip85Derive`. Nav-tests: 2 new (`Navigable` + `LeftWrap`, mirror `gui/multisig_program_test.go`) + repoint the prior programs' carousel-count assertions.
- **Index-entry widget (OPEN — plan must resolve):** confirm an existing numeric-entry/stepper widget (the `stepper` pkg, or the xpub picker's path-component entry) before assuming; if none fits, a bounded small-set `ChoiceScreen` (e.g. 0–9) or a minimal numeric keypad. Default index 0.

## 4. Faithfulness / security spine
- **Two secrets to scrub (more than T4/T6a's one):** the typed MASTER mnemonic AND the derived CHILD mnemonic — both `defer`-scrubbed (`[]Word` zeroed) on EVERY exit (derive, warning-abort, engrave-abort, error). Plus the intermediate 32-byte privkey serialization and the 64-byte HMAC output → `wipeBytes` after `bip39.New`. Mirror the `deriveAccountXpub` capture-before-zero discipline for each `hdkeychain` intermediate.
- **Typed-only master, NEVER NFC** (the `gui/scan.go` footgun) — no scan→derive path in the flow.
- **Child engraved onto owner-held steel ONLY, never NFC** (`engraveSeed`/`backup.EngraveSeed` plan — same channel as Backup Wallet's seed plate).
- **No xprv ever serialized/engraved/NFC'd** — the flow serializes only the EC privkey internally (scrubbed) to feed the HMAC; the engraved artifact is the child WORDS + SeedQR, no extended key.
- **Deterministic-only:** BIP-85 is a pure function of (master seed, app, words, index) — no CSPRNG/TRNG. Re-running with the same inputs reproduces the same child.
- **Spec-faithful-or-nothing:** picker bounds = biptool's exactly (words∈{12,18,24}, hardened index, English) so the on-device BIP-85 can NEVER disagree with biptool/other wallets.

## 5. Acceptance gate (TDD)
1. **Derive (known vector):** the derive helper reproduces a canonical BIP-85 BIP-39 child — assert against the spec/biptool vector (master → `m/83696968'/39'/0'/12'/0'` → expected 12-word child); 18'/24' produce 24/32-byte entropy → 18/24-word children; the child is the LEADING `entLen` bytes (a trailing-bytes bug → different child → caught).
2. **Path is fully hardened + exact:** the walk uses `83696968'`, `39'`, `0'`, `{words}'`, `{index}'` all ≥ HardenedKeyStart; a non-hardened element or a wrong app/lang index → wrong child (guard test).
3. **Picker bounds:** words∈{12,18,24} only (13/15/0/27 rejected); index hardened ≥0; application fixed BIP-39.
4. **Engrave:** the child mnemonic is engraved via `engraveSeed` → plate carries the child WORDS + a standard SeedQR (reuse the Backup-Wallet path; assert the plate is built from the CHILD mnemonic, not the master).
5. **Warning gate:** the unskippable child-seed warning is shown before engrave; abort at the warning → no plate, secrets scrubbed.
6. **Security + scrub:** typed-only master (D12 structural — no scan→derive symbol in the flow); BOTH master and child mnemonics + the privkey/HMAC buffers scrubbed on all exit paths (derive/abort/error); grep clean of xprv serialization in the engraved artifact.
7. **Program nav:** `bip85Derive` reachable (between engraveMultisig/qaProgram, non-blank title, no render panic; 2 new nav-tests + repointed prior-program counts); `TestAllocs` green.
8. **No-regression:** Backup Wallet (`engraveSeed`), T4/T6/single-card flows + codecs byte-unchanged; fuzz the derive helper (0 panics).

## 6. Invariants (R0 must confirm)
- **I-1 (Critical, spec-faithful derivation):** the child is derived by the FULLY-hardened path `m/83696968'/39'/0'/{words}'/{index}'`, `bip85.Entropy` over the leaf's 32-byte privkey, child entropy = the LEADING `entLen=(n*11-n/3)/8` bytes; byte-identical to biptool/canonical BIP-85. A wrong path/truncation/word-mapping → a divergent child (silent-wrong-backup) — refuse via guard tests.
- **I-2 (Critical, picker bounds):** word count ∈ {12,18,24} only, index hardened ≥0, application fixed BIP-39 — the on-device picker can NEVER produce an out-of-spec child.
- **I-3 (Critical, secrets):** typed-only master (never scan→derive); BOTH master + child mnemonics + intermediate privkey/HMAC buffers scrubbed on every exit; deterministic (no CSPRNG).
- **I-4 (Critical, channel):** the child seed is engraved onto owner steel ONLY, never NFC; no xprv/extended-key serialized into the artifact.
- **I-5:** reuse `engraveSeed` verbatim (no new engrave primitive); the child plate = child words + standard SeedQR.
- **I-6:** mainnet-only (BIP-85 path network-agnostic; artifact carries no network).
- **I-7:** new `bip85Derive` program coherent across all 8 lockstep sites (no panic/blank title; qaProgram non-navigable; 2 new nav-tests + repointed prior counts); `TestAllocs` green.
- **I-8 (no-regression):** Backup Wallet / T4 / T6 / codecs byte-unchanged.

## 7. Biggest risks (lock in R0)
1. **Re-creating the biptool derive logic** (I-1) — the `bip85` package is a thin primitive; the path walk + truncation + word-mapping are net-new in the flow and MUST match biptool/canonical BIP-85 byte-for-byte. The single load-bearing correctness risk.
2. **Picker bounds drift** (I-2) — an out-of-spec word count/app → a child no other BIP-85 wallet reproduces.
3. **Two-secret scrub** (I-3) — master AND child mnemonics + privkey/HMAC buffers; scrub on warning-abort/engrave-abort/error.
4. **Index-entry widget** (§3 open) — confirm an existing widget before assuming; don't over-build.
5. **Program-lockstep drift** (8 sites + nav-tests) — the mechanical risk (the T6a/T6b dance).
6. **No hardware** to validate the picker + derive + engrave UX.

## 8. Gate
This spec MUST pass opus R0 to 0C/0I before code; fold → persist → re-dispatch until GREEN. Then implementation plan → its own R0 → single-implementer TDD → mandatory whole-diff adversarial exec review → merge no-ff (signed+DCO) → push bg002h. T7b is the last planned T7 item (T7a done; final-word + convert already shipped); shipping it completes the T1–T7 build-out.
