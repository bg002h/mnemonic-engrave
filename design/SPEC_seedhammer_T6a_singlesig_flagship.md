# SPEC — T6a: single-sig flagship (seed → ms1+mk1+md1 → engrave + verify-bundle + restore doc)

**Status:** for opus R0 gate (must reach 0C/0I before any code).
**Fork base:** `e4013a8` (T5 shipped). **Fork-side only; no upstream PR.**
**Feeds from:** `design/cycle-prep-recon-T6-flagship.md` + `design/agent-reports/seedhammer-T6-recon-build-feasibility.md` + `…-recon-scope-security.md` + `…-architect-scope-multisig.md` (the architect consult — the authoritative scope/split + the D1-revised + D13-D17). **USER decisions:** secret = ms1 + watch-only (option 1); multisig via supply-md1 (T6b, NOT here). **Cite Rust source SHAs** for the encoder: `md-codec` v0.36.0 (`descriptor-mnemonic` @ `c85cd49`), `ms-codec`.

## 1. Why / context
T6 = the constellation flagship: from ONE hand-typed seed, derive + engrave the full single-sig constellation backup — the **secret** card `ms1` (the seed's BIP-39 entropy as codex32), the **key** card `mk1` (the account xpub), and the **policy** card `md1` (the single-sig descriptor) — then read the cards back to confirm parity (verify-bundle) and show a watch-only restore doc. It ports the `mnemonic-toolkit` flagship to-device. **This is a NEW `program`** (parallel to T4's `engraveXpub`, NOT an extension of T5's public-only `engraveBundle`). T6a is the SINGLE-SIG foundation (the architect-split first cycle); **multisig (via supplied md1) is T6b; the on-device policy picker is the deferred T6c.** Single-sig is the only scope where "one seed → complete ms1+mk1+md1" is literally true.

**The make-or-break:** the md1 encoder (#10a) is byte-faithful but ENTIRELY UNEXPORTED + consumes the md-internal AST (the `body` interface's `isBody()` is unexported → unconstructible outside `md`). T6a's dominant risk + deliverable is a **NEW exported `md.EncodeSingleSig`** that builds the simplest AST internally (n=1, one `keyArgBody`, one pubkey TLV) and calls the shipped `split`/`encodeMD1String`. It is new PUBLIC surface on a Rust-golden-byte-locked package → R0-gated, external-protocol-fact-verified vs Rust md-codec, golden-parity tested.

## 2. Scope

### IN (T6a)
- **`md.EncodeSingleSig(...)` (NET-NEW md API):** an exported single-sig md1 encoder. Signature shape (R0 to confirm): `EncodeSingleSig(accountXpub *bip380.Key-or-equiv, script ScriptKind) ([]string, error)` (or taking the xpub bytes + origin + use-site + script). Builds the md-internal `*descriptor` (`n=1`, `tree=node{tag: tagWpkh/tagPkh/tagTr, body: keyArgBody{0}}`, one pubkey TLV from the account xpub, the origin path, `<0;1>/*` use-site) and calls `split`/`encodeMD1String`. PRIMARY gate = byte-exact parity vs the constellation's single-sig md1 goldens (the same `.bytes.hex`/`.phrase.txt` vendoring discipline #10a used: `wpkh_basic`, `pkh_basic`, `tr_keyonly`).
- **A new top-level `program` (8-site lockstep, inserted BEFORE `qaProgram` per T5's R0-I-A precedent):** typed seed entry → single-sig path/script picker → derive → (watch-only choice) → engrave sequence → verify-bundle → restore doc.
- **Derive the 3 legs from the typed seed:** ms1 via `m.Entropy()` (`bip39/bip39.go:158`) → `codex32.NewSeed("ms", 0, id, 'S', entropy)` (`codex32/codex32.go:279`; the 4-char `id` is a fixed deterministic value — R0 to lock; e.g. derived from the fingerprint or a constant); mk1 via the T4 chain (`deriveAccountXpub` `gui/derive.go:19` + `mk.Encode`); md1 via the new `EncodeSingleSig`.
- **Operator-choosable watch-only / skip-ms1 mode** (user option-1): full mode engraves ms1+mk1+md1; watch-only engraves only mk1+md1 (no secret).
- **Single-sig path/script picker restricted to the 4 single-sig types** (BIP-44 pkh / BIP-49 sh-wpkh / BIP-84 wpkh / BIP-86 tr) × network — NOT T4's 6-type picker (which offers BIP-48/87 multisig paths T6a can't complete).
- **Engrave sequence:** synthesize `[]bundleCard` from the derived strings and reuse T5's `bundleEngrave` (`gui/bundle_flow.go:327`) — verbatim plates, "Card X of Y · Plate P of Q", set-level abort. A T6-specific completion message (NOT T5's "hand-engrave your ms1" reminder, since T6 engraved it — or in watch-only mode, DO remind).
- **verify-bundle (deterministic read-back parity):** re-type the seed (D8 re-type, shorter residency) → read back mk1/md1 over NFC (PUBLIC) + ms1 HAND-TYPED (SECRET, never NFC) → re-derive + compare on: master fingerprint, account xpub, path, descriptor (md1 string exact-match — deterministic), ms1 entropy, wordlist language → PASS/FAIL + which field diverged. Offered inline after engrave AND as a re-enterable standalone (D7).
- **restore doc (watch-only, display-only + optional NFC export, D9):** master fingerprint + the concrete descriptor + first receive + first change address (via a from-xpub `*bip380.Descriptor` + `address.Receive`/`Change`). NO secret.

### OUT (deferred)
- **Multisig** → T6b (supply a complete md1 over NFC + slot-cross-match + verbatim engrave; NO new encoder). NOT here.
- **On-device multisig policy picker + `md.EncodeMultisig`** → T6c (deferred/demand-gated).
- Producing md1 for anything but single-sig; self-multisig (constellation hard-rejected).

## 3. Verified facts (cite source)
- Build feasibility (the build-feasibility recon): mk1 leg EXISTS (T4); ms1 lib EXISTS (`codex32.NewSeed`) + `bip39.Entropy()`; md1 = GAP → `EncodeSingleSig` net-new; verify-comparator net-new (deterministic exact-match); restore-doc = assemble `address.Receive/Change` + from-xpub `*bip380.Descriptor` (constructible literal, `gui/md1_expand.go:60-77`); engrave reuse `bundleEngrave` (`[]bundleCard`-driven, not scan-bound).
- `EncodeSingleSig` AST shape: `descriptor{n:1, tree:node{tag,body:keyArgBody{0}}, tlv:{one pubkey}}` (`md/md.go:119,816`); calls unexported `split`/`encodeMD1String` (`md/chunk.go:121`, `md/encode.go:451`).
- ms1 = BIP-39 ENTROPY (16-32B) as codex32 (`ms-codec/src/lib.rs:29`); wordlist language matters (`mnem`).
- Security spine (T4): `wipeBytes` (`gui/slip39_polish.go:330`); `.Zero()` master+intermediates (`gui/derive.go:28-51`, serialize-before-Zero); mnemonic scrub (`gui/derive_xpub.go:113`). **Footgun (D12):** `gui/scan.go:61-70` can parse bip39/codex32 from NFC — T6a seed input MUST be the typed `newInputFlow`/`seedEntryFlow`, never `act.scan`.

## 4. Faithfulness / security spine (the most sensitive tier)
- **Seed/mnemonic/passphrase SECRET → typed-only, NEVER NFC** (D12, Critical): assert the T6a program's seed input is typed; never consume a scanned bip39/codex32 object. ms1 (the secret card) is engraved onto owner-held steel only, NEVER NFC.
- **`.Neuter()` everything before serialization** — no xprv ever serialized/displayed/engraved/NFC'd; the restore doc greps clean of private material.
- **Per-leg scrub schedule (D11, Critical):** zero the BIP-39 ENTROPY buffer after `NewSeed`; `wipeBytes` the 64-byte PBKDF2 seed + `.Zero()` master + EVERY intermediate `*ExtendedKey` + zero the mnemonic `[]Word` — each after its LAST consumer (T6a holds the seed across entropy→ms1, xpub→mk1, descriptor→md1, AND restore-doc address derivation → re-zero after the LAST, not the first). Capture the fingerprint BEFORE zeroing master. verify-bundle RE-TYPES the seed (fresh residency window, its own scrub).
- **mk1/md1/xpub/addresses PUBLIC**; restore doc carries NO secret. Passphrase never engraved/transmitted (no-pp vs pp fingerprint follows `backupWalletFlow`).
- **Set-level all-or-nothing** (reuse T5's abort): a partial bundle is NOT a usable backup → incomplete warning; re-entry re-derives deterministically (no half-state).

## 5. Acceptance gate (TDD)
1. **`EncodeSingleSig` byte-exact parity (PRIMARY):** for each single-sig golden (`wpkh_basic`/`pkh_basic`/`tr_keyonly`, vendored in `md/testdata`), building the descriptor from the golden's account key → `EncodeSingleSig` → equals the golden md1 string(s); each chunk `ValidMD`; `md.DecodeChunks`/`md.Decode` round-trips.
2. **Derive parity:** for the abandon-test seed at m/84'/0'/0', the derived mk1 == T4's known card; the derived md1 (wpkh over that xpub) decodes to the expected single-sig descriptor; the derived ms1 decodes (via the shipped ms1 decoder) back to the original entropy + wordlist language.
3. **Watch-only mode:** full mode → 3 cards (ms1+mk1+md1); watch-only → 2 cards (mk1+md1), no ms1 engraved, ms1-reminder shown.
4. **verify-bundle:** re-derive + compare a correct read-back set → PASS; a mutated card (wrong xpub/descriptor/entropy) → FAIL naming the field; ms1 hand-typed leg (never NFC).
5. **restore doc:** master fp + descriptor + first receive/change address match the derived wallet; greps clean of any xprv/private material.
6. **Security:** seed input is typed-only (a test asserts the T6a flow never routes a scanned object to derivation); buffers scrubbed on all exit paths incl. abort/error; fuzz `EncodeSingleSig` + the verify comparator (0 panics).
7. **Program nav + no-regression:** the new program reachable/titled/wrap-correct (before `qaProgram`, both title+layout arms), `TestAllocs` re-run green; single-card flows + `deriveXpubFlow` + `backupWalletFlow` + T5 `bundleFlow` byte-unchanged; full suite green.

## 6. Invariants (R0 must confirm)
- **I-1 (Critical):** `md.EncodeSingleSig` is byte-faithful to the Rust single-sig md1 goldens (canonicalization included); proven by §5.1 before any GUI work.
- **I-2 (Critical, D12):** seed/mnemonic/passphrase are typed-only; the T6a flow NEVER consumes a scanned bip39/codex32 object; ms1 engraved-only, never NFC.
- **I-3 (Critical, D11):** complete per-leg scrub — entropy after `NewSeed`, seed/master/intermediates/mnemonic after their LAST consumer; fingerprint captured before zeroing; verify-bundle re-types (own scrub).
- **I-4:** `.Neuter()` — no private material ever serialized/displayed/engraved/NFC'd (restore doc clean).
- **I-5:** watch-only mode engraves only mk1+md1 (no secret); full mode all 3; correct completion/reminder per mode.
- **I-6:** verify-bundle is deterministic re-derive-and-compare (fp + xpub + path + descriptor + ms1-entropy + wordlist-lang → PASS/FAIL); mk1/md1 read via NFC, ms1 hand-typed.
- **I-7:** restore doc display-only (+optional NFC), no secret; addresses via `address.Receive/Change`.
- **I-8:** the single-sig path picker offers only the 4 single-sig types (not BIP-48/87).
- **I-9:** new `program` coherent across all 8 lockstep sites (before `qaProgram`, both title+layout arms, no reachable panic, nav-test updated); `TestAllocs` intact.
- **I-10 (no-regression):** single-card flows, `deriveXpubFlow`, `backupWalletFlow`, T5 `bundleFlow`/`bundleEngrave`, and the codecs are byte-unchanged.

## 7. Biggest risks (lock in R0)
1. **`md.EncodeSingleSig` — new public API on a Rust-golden-byte-locked pkg** (dominant). Verify the single-sig md1 wire + canonicalization vs Rust md-codec (external-protocol-fact rule), golden-parity gate. R0 must confirm the signature + that single-sig is genuinely the simplest AST (no multi-key/threshold complexity).
2. **Longest secret residency in the constellation** (per-leg scrub, D11) — the seed is held across 3 derivations + addresses; a missed scrub on any leg/branch is Critical.
3. **Typed-only seed (D12)** — the `scan.go` footgun; assert + test.
4. **ms1 `id` determinism** — the 4-char codex32 id must be fixed/deterministic (no CSPRNG); R0 to lock its derivation.
5. **Program lockstep drift** (T5's exact surface) — before `qaProgram`, both arms, derived consts.
6. **No hardware** to validate the multi-leg derive→engrave→verify UX.

## 8. Phasing / split note for R0
T6a is itself large (new encoder + new program + 3-leg derive + verify + restore). R0 should assess whether to split further (e.g. T6a-1 `EncodeSingleSig`+derive-core headless / T6a-2 GUI program+engrave+verify+restore) — analogous to #10a/#10b. The author's lean: keep T6a whole if reviewable, else split at the headless/GUI boundary (the encoder + derive + verify-comparator are headless-testable; the program/screens are GUI).

## 9. Gate reminder
This spec MUST pass opus R0 to 0C/0I before code; fold → persist verbatim → re-dispatch until GREEN. Then implementation plan → its own R0 → single-implementer TDD → mandatory whole-diff adversarial exec review → merge no-ff (signed+DCO) → push bg002h. The seed-handling spine (typed-only, per-leg scrub) + the `EncodeSingleSig` golden parity are the must-verify items. T6b (multisig-via-supply) follows; T6c (picker) deferred.
