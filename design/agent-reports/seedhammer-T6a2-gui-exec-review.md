# T6a-2 (GUI) — whole-diff adversarial EXECUTION review

**Scope:** the single-sig flagship GUI flow (`engraveSingleSig` program: typed seed → pick → derive ms1+mk1+md1 → engrave full/watch-only → verify-bundle → restore doc).
**Branch:** `feat/t6a2-gui` @ `f1c0b88` off `bfff857`. Worktree `/scratch/code/shibboleth/seedhammer-wt-t6a2`.
**Diff reviewed:** `bfff857..f1c0b88` (8 commits, 22 files, +1922/-22).
**Reviewer:** opus architect. Reviewer-only — no tracked source modified (temp probe tests written, run, then removed; `git status` clean).

---

## VERDICT: GREEN (0 Critical / 0 Important)

The feature is correct, deterministic, and secret-safe. The vendored-golden drift the implementer flagged is a **stale hand-crafted vector**, NOT an encoder bug, and does not affect T6a-2's own derived mk1. The restore-doc xpub byte-matches the engraved mk1 for all 4 scripts incl. sh-wpkh (independently re-derived). No secret leak; typed-only seed; per-leg scrub on all exit paths incl. abort. T4/T5/codecs byte-unchanged. Full suite + TestAllocs green; both fuzzers >2.8M execs with 0 panics.

Findings below are all **Minor** (non-blocking).

---

## (a) ROOT CAUSE of the wpkhMK1 vendored-golden drift — STALE VECTOR, not an encoder bug

I decoded the vendored `wpkhMK1` golden (`bundle/verify_test.go:17-20`), re-encoded its decoded card with the shipped `mk.Encode`, and diffed at the bytecode + header level.

```
DECODED CARD: Network="mainnet" Path="m/84'/0'/0'" Fingerprint="73c5da0a" Stubs=[1c0170fe]
              Xpub="xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3Xyuv…7PW6V"
VENDORED[0]: mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4
REENCODE[0]: mk1qph25epqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qasghpexqvjkydy7
VENDORED csid = 0x1c017   REENCODE csid = 0xbaa99
VENDORED bytecode (84 B): 04011c0170fe73c5da0a030488b21e…2da34f5f3a09a9b
REENCODE bytecode (84 B): 04011c0170fe73c5da0a030488b21e…2da34f5f3a09a9b   ← BYTE-IDENTICAL
top20(SHA-256(bytecode)) = 0xbaa99 (== REENCODE csid)
DECODE(REENCODE) == DECODE(VENDORED)  → card round-trips identically
```

**Findings:**
1. The decoded **bytecode is byte-for-byte identical** (84 B) between the vendored golden and the re-encode. The two strings differ ONLY in the 4 csid header symbols (and the consequent BCH checksum).
2. The shipped `mk.Encode` derives the csid deterministically as `top20(SHA-256(bytecode)) = 0xbaa99` (`mk/encode.go:236,314-319`). The re-encoded string carries `0xbaa99` and is self-consistent.
3. The vendored golden carries csid `0x1c017`. I confirmed `0x1c017 == top20(stub_bytes 1c0170fe)` exactly — i.e. the vendored vector was produced by an EARLIER/hand-crafted encoder that mis-derived the csid from the **stub bytes** instead of `SHA-256(bytecode)`.
4. `mk.Decode` only checks that all chunks share the SAME csid (`mk/mk.go:192-193`); it never validates the csid against any canonical value. So the stale vector decodes fine and round-trips faithfully (`decode(encode(decode(vendored))) == decode(vendored)`).

**Authoritative cross-check (external-protocol-fact rule).** In the canonical Rust `mk-codec`, `key_card::encode` draws the `chunk_set_id` **from the system CSPRNG** (`/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/key_card.rs:97-98`), with `encode_with_chunk_set_id` as the deterministic override "for vector regeneration, conformance tests" (`:104-110`). The cross-chunk binding is the integrity hash `SHA-256(canonical_bytecode)[0..4]` (`src/string_layer/chunk.rs:4-5,67`), NOT the csid. **Therefore the csid is a non-load-bearing chunk-grouping nonce** — it carries no wallet semantics, the decoded card is independent of it, and a wallet/verifier never depends on its value. The Go fork's choice to make it deterministic (`top20(SHA-256(bytecode))`, "NO CSPRNG") is a stricter-than-Rust, legitimate design that yields byte-stable output.

**Conclusion:** (a) STALE/hand-crafted vendored vector. There is NO non-determinism and NO decode→encode-non-identity bug in the shipped `mk.Encode`/`mk.Decode` (the bytecode is identity-preserved; only a cosmetic nonce header differs). No latent T4/T2b regression.

**Is T6a-2's OWN derived mk1 correct/deterministic? YES.** `deriveSingleSigBundle` (`gui/singlesig_derive.go:73-82`) calls `mk.Encode`, so its mk1 always carries the spec-correct deterministic csid `0xbaa99` for the wpkh abandon card — NOT the stale `0x1c017`. The verify-bundle path re-derives via the same `mk.Encode`, so both sides are byte-identical; verify compares DECODED fields + stub binding (`bundle/verify.go`), which are csid-independent anyway. `TestDeriveSingleSigBundleMatchesGoldenWpkh` correctly asserts the mk1 by decoded-fields + bound stub (R0-m1), not raw-string vs the stale golden — exactly the right call.

**Minor M-1 (optional hygiene, non-blocking):** the vendored `wpkhMK1` golden in `bundle/verify_test.go:17-20` is a stale-csid vector. It is harmless for these comparator tests (they decode it, never re-encode it, and `mk.Decode` ignores the csid value), but regenerating it from the current `mk.Encode` would remove the latent confusion. Not required for merge.

---

## (b) Restore-doc xpub byte-matches the engraved mk1 — ALL 4 SCRIPTS, independently re-derived

Re-derived the abandon seed for all 4 purposes and built the restore descriptor directly (`gui/singlesig_restore.go:60-91`):

```
pkh   (44) masterFP=73c5da0a parentFP=155bca59  xpub==mk1.Xpub=true  enc⊇mk1.Xpub=true
            pkh([73c5da0a/44h/0h/0h]xpub6Bosf…9nMdj/<0;1>/*)#kw28l7md
            recv0=1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA
sh-wpkh(49) masterFP=73c5da0a parentFP=3d05ff75  xpub==mk1.Xpub=true  enc⊇mk1.Xpub=true
            sh(wpkh([73c5da0a/49h/0h/0h]xpub6C6nQ…UaJa7/<0;1>/*))#zmygnj3e
            recv0=37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf   (canonical BIP-49 abandon vector ✓)
wpkh  (84) masterFP=73c5da0a parentFP=7ef32bdb  xpub==mk1.Xpub=true  enc⊇mk1.Xpub=true
            wpkh([73c5da0a/84h/0h/0h]xpub6CatW…PW6V/<0;1>/*)#qf45pmyh
            recv0=bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu   (canonical BIP-84 abandon vector ✓)
tr    (86) masterFP=73c5da0a parentFP=035270da  xpub==mk1.Xpub=true  enc⊇mk1.Xpub=true
            tr([73c5da0a/86h/0h/0h]xpub6BgBg…ReUsQ/<0;1>/*)#xf07c0qd
            recv0=bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr
```

- `desc.Encode()` xpub BYTE-MATCHES the engraved mk1 xpub verbatim for all 4 scripts.
- The `parentFP` is the REAL, non-zero, per-script-distinct value (155bca59 / 3d05ff75 / 7ef32bdb / 035270da) — NOT 0. R0-I1 is correctly threaded: the descriptor's `Key.ParentFingerprint` comes from the derive-time `decodeXpubBytes` (`gui/singlesig_derive.go:53,121`) and is forwarded through the orchestrator (`gui/singlesig.go:77,95`).
- **`address.Receive(wpkh, 0) == bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`** — the known BIP-84 m/84'/0'/0'/0/0 abandon vector. Also confirmed sh-wpkh #0 = `37Vuc…` (canonical BIP-49 vector), independently corroborating the sh-wpkh classifier-bypass path.
- Explicit `<0;1>/*` children present (R0-m4). Descriptor master-fp origin uses masterFP. No xprv/tprv (greps clean; `FuzzSingleSigRestoreDescriptor` asserts on the key-string version prefix, 3.79M execs clean).
- sh-wpkh works (the classifier would have dropped it; the direct build keeps it) — confirmed by the produced `sh(wpkh(...))` descriptor + valid P2SH address.

---

## (c) Security spine — NO secret leak, typed-only, per-leg scrub on abort

- **Typed-only (D12).** `engraveSingleSigFlow` obtains the seed via `seedEntryFlow` ONLY (`gui/singlesig.go:32`); no `act.scan`/`assembleScan`/scanner path to derivation. Structural test `TestEngraveSingleSigFlowTypedOnly_Structural` strips comments then asserts presence of `seedEntryFlow` and absence of the forbidden scan primitives. The verify flow re-types via `seedEntryFlow` too (`gui/singlesig_verify.go:66`). The NFC read-back gatherer REFUSES a scanned ms1 (codex32 secret → `clsMs1Refuse`, `gui/bundle.go:46,56-69`) and only ever yields `cardMK1`/`cardMD1`; `singleSigReadbackCards` ignores any other kind.
- **Per-leg scrub on ALL exit paths incl. abort (D11).** The mnemonic `[]Word` is zeroed by a `defer` registered immediately after seed entry (`gui/singlesig.go:41-45`), so abort at the picker/passphrase/mode-choice all scrub. `TestEngraveSingleSigFlowSeedScrubbed` drives the abort-at-picker path and asserts every word is 0. `seedEntryFlow` allocates a fresh slice per call (`gui/derive_xpub.go:89`), so the orchestrator and verify flow each own a distinct, separately-scrubbed buffer. Entropy: gated on `m.Valid()` first (`gui/singlesig_derive.go:40`; `Entropy()` panics on invalid, `bip39.go:159`), returns a FRESH buffer (not aliasing the mnemonic — `bip39.go:158-164,177-194`), and is `wipeBytes`'d unconditionally before the error check (`gui/singlesig_derive.go:85-88`). Seed/master/intermediates scrubbed inside the byte-unchanged `deriveAccountXpub` (`gui/derive.go:19-53`, serialize-before-Zero preserved).
- **No private material engraved/displayed/NFC'd.** `decodeXpubBytes` refuses an xprv (`gui/singlesig_derive.go:104-106`). md1/mk1 carry only the neutered xpub. Restore doc is public (greps clean of xprv across all 4). The ms1 (secret) is engraved onto steel only (`bundleEngrave` → `NewEngraveScreen.Engrave`, never NFC); in verify it is HAND-TYPED, never NFC-read. Passphrase is an immutable Go string (accepted un-wipeable residual per spec §4, identical to T4).

---

## (d) T4 / T5 / codecs BYTE-UNCHANGED

- T4: `gui/derive_xpub.go`, `gui/derive.go` — NOT in the diff (empty `git diff`). `deriveAccountXpub` reused verbatim.
- T5: `bundleEngrave` signature unchanged (`func bundleEngrave(ctx *Context, th *Colors, cards []bundleCard)`); its call site at `gui/bundle_flow.go:36` (bundleFlow) is byte-unchanged. The only `bundle_flow.go` change is the internal reminder gate (`bundleShowMs1Reminder(cards)` — cards-derived, R0-I2), plus the new helper. The only `bundle.go` change is the additive `cardMS1` enum value.
- Codecs: `md/`, `codex32/`, `mk/`, `address/`, `bip380/`, `bip32/`, `bip39/` — NONE in the diff. The only headless change is the `bundle/verify.go` watch-only extension (placed AFTER stub-binding/mk1/md1 legs, BEFORE the ms1-entropy leg; both-empty → skip, one-sided → error). The sole non-test caller of `bundle.Verify` is the new `verifySingleSig`; no other caller exists to break.
- `engraveSingleSig` enum sits between `engraveBundle` and `qaProgram`; all 8 lockstep bounds moved (left/right wrap, npage, npages, dispatch, title, layoutMainPlates, layoutMainPager); both nav-tests updated; `qaProgram` reachable only via the debug command "FOREVERLAURA!" (`gui.go:1606`), excluded from the carousel.

---

## Test / vet / gofmt / fuzz output (clean tree, temp probes removed)

```
go test -count=1 ./...                     → ALL ok (gui 5.2s, bundle, mk, md, codex32, … all ok)
go test -run TestAllocs ./gui/             → PASS (1.28s)   (restore doc NOT via DescriptorScreen)
go vet ./gui/... ./bundle/...              → only gui/op/draw_test.go:176 ArtifactDir go1.26 note
                                              (PRE-EXISTING: t.ArtifactDir() present identically at
                                               bfff857; gui/op/ untouched by this diff)
gofmt -l . (excl third_party/)             → clean
go test -fuzz FuzzVerifyWatchOnly  45s     → 2,833,025 execs, 0 crashes, PASS
go test -fuzz FuzzSingleSigRestoreDescriptor 45s → 3,785,978 execs, 0 crashes, PASS
```

---

## Minor findings (non-blocking)

- **M-1** — stale-csid vendored `wpkhMK1` golden (`bundle/verify_test.go:17-20`). Harmless (decode-only; csid not validated by Decode). Optionally regenerate from the current `mk.Encode`. See (a).
- **M-2** — `singleSigVerifyFlow(ctx, th, derived bundle.Bundle, full bool)` (`gui/singlesig_verify.go:64`) never USES its `derived` parameter: the body correctly re-derives `reDerived` from the re-typed seed (spec §4 semantics) and compares THAT against the NFC read-back. The passed-in engraved bundle is dead. Drop the param (and the orchestrator arg at `gui/singlesig.go:91`) to remove the confusion. Pure clarity; no behavioral effect. (`go vet` does not flag unused params.)
- **M-3** — the verify PASS result uses `showError(ctx, th, "Verify OK", …)` (`gui/singlesig_verify.go:126`) — a success message routed through a function named `showError`. Cosmetic naming wart; the screen renders correctly. Consider a neutrally-named `showMessage` alias.

All three are clarity/hygiene only and do not gate merge.

---

## Bottom line

VERDICT: **GREEN**. The flow derives the correct deterministic single-sig constellation, binds mk1↔md1 with the policy-bound non-zero stub, engraves the secret ms1 only to steel (full) or reminds (watch-only), verifies by deterministic re-derive + public NFC read-back + hand-typed ms1, and produces a canonical, sh-wpkh-safe, secret-free restore doc whose xpub byte-matches the engraved key. The vendored-golden drift is a stale vector, not a defect. Fold the 3 Minors at the implementer's discretion; none block merge.
