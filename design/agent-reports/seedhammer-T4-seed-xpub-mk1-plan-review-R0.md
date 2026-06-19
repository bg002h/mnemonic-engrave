# R0 Gate Review — IMPLEMENTATION_PLAN_seedhammer_T4_seed_xpub_mk1 (commit 5eda122)

**Reviewer:** opus-architect (materialize + build + run)
**Spec:** SPEC_seedhammer_T4_seed_xpub_mk1.md (3b15251) — GREEN
**Base fork:** a4d669d (detached worktree, removed after review; fork left clean at a4d669d)
**Go:** go1.26.4 (/home/bcg/.local/go/bin/go)
**Method:** Implemented the plan task-by-task in a throwaway worktree, inverting the shipped `mk.Decode`,
reusing the `codex32` engine for the BCH-generate, implementing the scrub-complete derivation, the 6-site
program lockstep, and a working `deriveXpubFlow`. Ran every test, plus targeted oracles (golden-vector
byte/round-trip parity, C-1 init trap, fragment-bracket exhaustion, aliasing reproduction).

---

## Verification Results

### Task 1 — `mk.Encode` + `codex32.MKChecksumSymbols` — GREEN
- **`TestEncodeRoundTrip` (the oracle): PASS for ALL listed paths** — `m/84'/0'/0'`, `m/44'/0'/0'`,
  `m/48'/0'/0'/2'` (multisig), `m/87'/0'/0'`, `m/84'/1'/0'` (testnet). Each: `mk.Decode(mk.Encode(card)) == card`,
  every emitted chunk passes `codex32.ValidMK`, ≥2 chunks, deterministic (identical strings across runs).
- **Golden-vector parity (`TestEncodeGoldenRoundTrip`): PASS for all 7 `mk/mk_test.go` parity vectors**
  (decode golden → re-encode → re-decode → same Card; ValidMK on every re-encoded chunk). This exercises
  fp-present, 3-stub, explicit-path (`0xFE`+LEB128), testnet, and the long 105/131-byte multi-chunk cards
  the synthetic test omits.
  - **IMPORTANT discovery (not a defect, but it invalidates one suggested test):** the mk-codec golden
    vectors were generated with **arbitrary, sequential explicit chunk_set_ids** (`0x12345, 0x23456,
    0x34567, 0x45678, 0x56789, 0x67890, 0x78901` — confirmed by parsing each golden chunk-0 header), NOT
    a SHA-256-derived csid. So **byte-equality of emitted chunks against the golden vectors is impossible**
    for any deterministic SHA-256-based csid. The decoder does not validate the csid value (only
    consistency), so the correct cross-check is decode→re-encode→re-decode (which the spec §6 actually
    states as the primary option). The plan's wording "OR if a golden card's csid is known, byte-equality"
    is moot — the goldens' csids are not the SHA-256 value. Folded into the test as a documented note.
- **C-1 (BCH-init trap): empirically confirmed.** `codex32.MKChecksumSymbols` builds the engine with mk's
  `POLYMOD_INIT=0x23181b3` residue + mk targets (regular `0x1/0x62435f91072fa5c`, long
  `0x418/0x90d7e441cbe97273`). `TestMKChecksumC1Trap` shows a checksum computed with codex32's init (1)
  **self-fails `ValidMK`**, while the POLYMOD_INIT checksum validates. Round-trip + ValidMK on every chunk
  is the standing proof the right init is used.
- **Regular/long selection: confirmed empirically.** For a real ~84-byte 2-chunk card,
  **chunk 0 = long (data-part len 108, the max of [96,108]); chunk 1 = regular (len 71)** — exactly the
  spec's prediction ("chunk 0 typically long, trailing chunk regular"). `TestAllFragmentSizesValid`
  exhaustively proves every fragment size 1..53 bytes maps to **exactly one** valid bracket (none lands in
  the reserved [94,95] gap or out of range; no size is ambiguously valid for both codes), so the
  "try regular first, else long" rule is total and unambiguous.
- **`MKChecksumSymbols` placement:** lives in package `codex32` (uses the unexported `engine`/`unpackSyms`/
  targets), called by `mk.Encode`. No import cycle (`mk` already imports `codex32`). Pure-stdlib,
  TinyGo-safe (uint64-only, no math/big). A `codex32.SymbolRune` exported helper renders 5-bit→lowercase
  bech32 (the encoder cannot reach the unexported `fe.rune()`).
- Depth/child encode invariant (§2.2) enforced: `key.Depth()==len(comps)` AND
  `key.ChildIndex()==comps[last]`, else `errEncodeXpub`. No panic on bad xpub / empty stubs / bad fp hex /
  depth mismatch (typed errors — §2.9).

### Task 2 — `deriveAccountXpub` — GREEN **only after fixing a Critical in the plan's reference code**
- **Golden derivation: PASS.** The canonical "abandon…about" mnemonic (seed
  `5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4`,
  the published BIP-39 vector) → m/84'/0'/0' account xpub =
  `xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V`.
  Cross-validated independently: same seed → known root xprv `xprv9s21ZrQH143K3GJpoapnV8SFf…` and the famous
  first receive pubkey `0330d54fd0dd420a6e5f8d3624f5f3482cae350f79d5f0753bf5beef9c2d91af3c`. No `xprv` in the
  output; `.Neuter()` confirmed public.
- `(*ExtendedKey).Zero()`, `Derive`, `Neuter`, `ECPubKey`, `ChildIndex`, `Version`, `ChainCode`,
  `ParentFingerprint`, `NewKeyFromString`, `NewExtendedKey` all exist (btcutil/v2 v2.0.0,
  hdkeychain/extendedkey.go) with the signatures the plan assumes. `bip32.Fingerprint(*secp256k1.PublicKey)
  uint32` and `bip32.ParsePath` confirmed. Master-FP captured before zeroing master; seed buffer
  `wipeBytes`'d via defer; master + every intermediate `.Zero()`'d.
- **See CRITICAL-1 below** — the plan's exact Task 2 code corrupts the xpub.

### Task 3 — program lockstep (6 sites) — GREEN
All 6 edits implemented and verified:
1. enum (`gui.go:148`): `engraveXpub` inserted between `backupWallet` and `qaProgram`.
2. Right clamp (`:1637`): `> backupWallet` → `> engraveXpub`.
3. `layoutMainPlates` (`:1843`): `case engraveXpub:` added (reuses `assets.Hammer`) — no `panic("invalid page")`.
4. both page-count consts (`:1834 npage`, `:1857 npages`): `int(engraveXpub)+1`.
5. title switch (`:1655`): `case engraveXpub: titleTxt = "Account Xpub"`.
6. `uiFlow` dispatch (`:1493`): `case engraveXpub: deriveXpubFlow(ctx, th); continue` (no fall-through to
   `engraveObjectFlow`).
- **`TestEngraveXpubProgramNavigable`: PASS** — Right reaches "Account Xpub", Right again wraps to
  "Backup Wallet"; `qaProgram` stays out of the navigable range. **No `panic("invalid page")`.** Left
  navigation also panic-free and in-range (verified). `qaProgram` is unreachable by nav (debug-command only),
  so the title switch needs no `default`.
- **`TestAllocs`: PASS (0 allocs)** — the new program reuses the existing plate-image draw path, so the
  alloc gate is intact.

### Task 4 — `deriveXpubFlow` (GUI) — GREEN
- **`TestTwoStagePicker`: PASS** — stage-1 (6 script types) + stage-2 (2 networks) resolves correctly for
  BIP-44/84/48-multisig/87 mainnet and BIP-84 testnet (coin-type 0'→1' swap verified). The 6-entry stage-1
  `ChoiceScreen` renders without crash.
- **`TestStubWarningUnskippable`: PASS** — the warning ("…placeholder policy stub (00000000) and is NOT
  bound to a wallet policy") is shown; Back/Cancel → does NOT proceed; only the explicit "Engrave anyway"
  confirm proceeds.
- **Security spine: clean.** The only `.String()` on a key is on the **neutered** account key (xpub, never
  xprv). The only thing fed to the engraver (`validateMdmk`→`NewEngraveScreen`) is `mk.Encode(card)` output
  (public mk1 chunks). No `engraveSeed`/`backup.EngraveSeed`/`engraveObjectFlow` on this path. Mnemonic
  scrubbed via `wipeMnemonic` after derivation and on picker-back; seed buffer `wipeBytes`'d in the helper.
- Multi-plate sequencing ("Plate i of N") + set-level abort warning ("Incomplete: i of N…discard partials")
  implemented; NFCReader nil throughout.

### Build / vet / fmt / full suite
- `go build ./...` — PASS.
- `go vet ./mk/ ./codex32/ ./gui/ ./bip32/` — clean.
- `gofmt -l mk/ codex32/ gui/ bip32/` — empty (clean).
- **`go test ./...` — ALL PASS** (gui 5.3s, mk, codex32, md, engrave, stepper, all drivers, nfc, etc.).
- `go test -count=1 -run TestAllocs ./gui/` — PASS.
- **TinyGo controller build — NOT RUN (tinygo not installed in this env).** Low risk: new code is
  pure-stdlib (`crypto/sha256`, `encoding/binary`) + in-tree pkgs; `codex32` stays uint64-only/no-math-big;
  the controller already imports `gui`/`mk`/`codex32`. CI must confirm (carried as MINOR-1).

**Worktree removed; fork left clean at a4d669d (verified below).**

---

## Findings

### CRITICAL-1 — Plan's Task 2 reference code corrupts the engraved xpub (use-after-zero via hdkeychain aliasing)
- **Location:** Task 2, Step 3 implementation block (plan lines 161-164):
  ```go
  acct, err := k.Neuter() // public-only
  k.Zero()
  if err != nil { return "", 0, err }
  return acct.String(), masterFP, nil
  ```
- **Why (empirically proven — `TestPlanOrderingAliasing`):** `hdkeychain.(*ExtendedKey).Neuter()` for a
  private key returns `NewExtendedKey(version, k.pubKeyBytes(), k.chainCode, k.parentFP, …)`, and
  `NewExtendedKey` stores those slices **by reference (no copy)**. So `acct.chainCode`, `acct.parentFP`, and
  `acct.key` (= the memoized `k.pubKey`) **alias `k`'s backing arrays**. The plan calls `k.Zero()` (which
  does `zero(k.chainCode); zero(k.parentFP); zero(k.pubKey)`) **before** `acct.String()`, so the neutered
  account key is serialized from **zeroed** chainCode/parentFP/pubkey. Result: a **silently wrong but
  structurally valid xpub** gets returned and engraved onto a permanent steel backup.
  - Concrete proof for the abandon mnemonic m/84'/0'/0':
    - correct (serialize-before-zero): `xpub6CatWdiZiodmU…PC7PW6V` (the known BIP-84 vector)
    - plan ordering (zero-before-serialize): `xpub6BemYiVNp19Zz…TWooGf` (**corrupted**)
  - This defeats §2.1/§2.5/§2.7: the operator's watch-only backup would track the **wrong wallet**, with no
    error surfaced.
- **Whether the plan's own TDD catches it:** partially. The Task 2 test (Step 1) asserts
  `xpub == knownTestVectorXpub84`, which would FAIL with the buggy ordering — *if* the golden vector is the
  true one. The risk is an implementer "fixing" the failure by pinning the golden to the corrupted output.
  The reference code itself is wrong and must be corrected in the plan.
- **Concrete fix (verified GREEN):** serialize the neutered key **before** zeroing the pre-neuter private
  key, and check the `Neuter` error first:
  ```go
  acct, err := k.Neuter()
  if err != nil { k.Zero(); return "", 0, err }
  xpub = acct.String() // serialize BEFORE zeroing k
  k.Zero()
  return xpub, masterFP, nil
  ```
  (Zeroing `k` after serialize still fully scrubs the secret: the private scalar is `k.key`, which `Neuter`
  does NOT alias — `acct.key` is the *public* key bytes. The only aliased arrays are public data, harmless
  to zero post-serialize.) The fixed helper produces the correct golden xpub and the whole suite is GREEN.

### IMPORTANT-1 — "byte-equality vs golden vectors" sub-option is unachievable; pin the achievable parity check
- **Location:** Plan Task 1 Step 4 / spec §6 ("OR if a golden card's csid is known, byte-equality"); plan
  source-of-truth bullet on deterministic csid (lines 19, 25).
- **Why:** The 7 mk-codec golden vectors use **explicit arbitrary csids** (`0x12345…0x78901`, confirmed by
  header-parsing), not the SHA-256-derived csid the encoder must use deterministically. Therefore
  byte-identical chunk strings are **impossible** for the T4 encoder against these goldens, and any plan
  step or reviewer expectation phrased as "emit byte-identical golden chunks" is unsatisfiable and would
  block GREEN spuriously. The decoder does not validate the csid value (only cross-chunk consistency), so
  this is harmless to correctness — but the plan must not gate on byte-equality.
- **Concrete fix:** state explicitly in Task 1 that the golden cross-check is **decode→re-encode→re-decode
  → same Card** (+ ValidMK per chunk), NOT byte-equality; drop the "if a golden card's csid is known,
  byte-equality" clause (the goldens' csids are not the SHA-256 value). The deterministic
  `top20(SHA-256(bytecode))` csid remains a valid internal choice (round-trip + determinism cover §2.3).
  *(Implemented and GREEN as `TestEncodeGoldenRoundTrip` over all 7 vectors.)*

### MINOR-1 — TinyGo controller build unverified here
- CI must compile `./cmd/controller` (TinyGo) to confirm `mk.Encode` + `codex32.MKChecksumSymbols`/
  `SymbolRune` + the `gui` additions build for the device. Low risk (pure-stdlib, no new math/big), but
  carry it as the plan's Task 5 Step 2 and run it in CI.

### MINOR-2 — `card.Path` carries `h`-form in the GUI flow (cosmetic display only)
- **Location:** `deriveXpubFlow` builds `card.Path = path.String()`, and `bip32.Path.String()` emits the
  `h` hardened form (`m/84h/0h/0h`), whereas `mk.Decode`/`mk.pathString` emit the apostrophe form
  (`m/84'/0'/0'`). The pre-engrave `mk1DisplayFlow` therefore shows `m/84h/0h/0h`. This is purely cosmetic:
  `mk.Encode` parses the path via `bip32.ParsePath` (accepts both `h` and `'`) and matches on the uint32
  components, so encoding and the standard-table indicator are unaffected; a later `mk.Decode` of the
  engraved card shows the apostrophe form. No round-trip break. Optionally normalize the displayed/stored
  path to apostrophe form for consistency with the decoded card; not blocking.

### MINOR-3 — `qaProgram` has no title and Left does not wrap up to `engraveXpub`
- The start-screen title switch has no `default` (so `qaProgram` would render an empty title), and `Left`
  from `backupWallet` stays at `backupWallet` (doesn't wrap up to `engraveXpub`). Neither is a defect:
  `qaProgram` is unreachable by navigation (debug command only) and the Right clamp keeps `m.prog` ∈
  {backupWallet, engraveXpub}; Right reaches `engraveXpub` and wraps. Cosmetic/UX nit only.

---

## Verdict

**NOT GREEN — 1C / 1I**

(CRITICAL-1: plan's Task 2 reference derivation corrupts the engraved xpub via hdkeychain Neuter/Zero
aliasing — the security-critical output is silently wrong; fix is a 2-line reorder, verified GREEN.
IMPORTANT-1: the "byte-equality vs golden vectors" parity sub-option is unachievable (goldens use arbitrary
csids) and must be replaced with the decode→re-encode→re-decode cross-check. Everything else — the encoder
round-trip across all paths + all 7 golden vectors, the C-1 BCH-init trap, the regular/long bracket
selection, the program lockstep/no-panic/allocs, the picker + unskippable warning + security spine — is
GREEN. Fold the two findings, re-persist, and re-dispatch.)
