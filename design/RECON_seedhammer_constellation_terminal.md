# RECON — "SeedHammer as the air-gapped constellation terminal": what else should SH do?

**Date:** 2026-06-18. **Method:** 5-agent parallel survey of the constellation (ms-codec / md-codec / mk-codec / mnemonic-toolkit / application-layer), each asking "should SH gain an on-device capability for this feature?" against SH's hardware + security constraints and today's shipped baseline. Verbatim reports: `design/agent-reports/seedhammer-constellation-survey-{ms,md,mk,toolkit,app}.md`.

## Strategic verdict: do NOT port the constellation wholesale

"Port everything except the GUI" is the wrong unit, for reasons independent of the GUI:

1. **It's a rewrite, not a port.** Constellation = Rust; SH = Go/TinyGo. Nothing lifts mechanically — every crate is hand-reimplemented + re-pinned against Rust-generated vectors, under the same R0/exec-review gate that made the *single* m\*1 BCH decoder a multi-phase cycle.
2. **~Half is categorically excluded** (see SKIP below): generate/split (no on-device CSPRNG), wallet/signer (off-mission; PSBT/miniscript confirmed *absent* from firmware), authoring (wrong I/O for a keypad), PC/file-shaped ops, host/CI tooling.
3. **Maintenance + crypto risk scale with surface** — each ported crate is a Go reimpl kept in lockstep with the evolving Rust source; each primitive is a new place to engrave the wrong secret.

**The reframe (what the surveys converge on):** the valuable subset is **SH as the air-gapped constellation *terminal*** — decode/verify/derive-and-engrave across all formats. Crucially, **most of the crypto it needs is already in the fork** (`bip32`, `bip39`, `bip380`, `codex32`, `slip39`, decred `secp256k1`, the `address` pipeline, SHA), so much of the "port" is **wiring existing in-tree primitives into the touchscreen UI**, not reimplementing Rust. The single sharpest finding: `address/address.go` (`Receive`/`Change`/`Supported`/`addressAt`/`derivePubKey`) **already exists + is tested in the fork and is NOT imported by `gui/`/`cmd/`** — on-device address derivation is a UI-wiring task.

**User decision (2026-06-18): "Save analysis. Build all of the above."** → execute the BUILD roadmap below, each item as its own full gated cycle, in dependency order.

---

## BUILD roadmap (PORT/BUILD candidates, dependency-ordered)

Each is its own gated cycle: cycle-prep recon → spec → R0(→GREEN) → plan → R0(→GREEN) → single-implementer TDD in a worktree → mandatory whole-diff execution review → merge no-ff signed+DCO → push `bg002h`. Reviews persisted verbatim.

| Cycle | Capability | Effort | Depends on | Source surveys |
|---|---|---|---|---|
| **T1** | **Address derivation + display** — recall/scan a card (mk1 xpub / md1 descriptor) → derive + show receive/change addresses ("does this card control these?"). Mostly WIRING the in-tree `address` pkg. | S–M | — (foundation) | app-layer #1; md/mk |
| **T2** | **Decode→display→verify-before-engrave**, per format: md1 → human-readable BIP-388 template; mk1 → xpub+origin-fp+path+stubs; ms1 → entropy/BIP-39 words (+ the `mnem` wordlist-language byte — kills the non-English silent-wrong-wallet failure); + `inspect` ("share k-of-N, need K"). | S–M each | — (parallel to T1) | ms #1/#2/#3; md #1; mk #1; toolkit #5 |
| **T3** | **Receive-address verification** — type/scan an address, gap-limit scan a derived range, confirm match + show chain/index. | M | T1 | app-layer #2; toolkit #4 |
| **T4** | **seed → account xpub (+ standard-path picker) → engrave as mk1** — hand-typed seed → watch-only xpub steel backup; deterministic (bip32 ckd); emits only the public xpub. | M–L | (bip32 already in-tree) | mk #2; ms #4; toolkit convert→xpub |
| **T5** | **Multi-chunk reassembly + integrity + guided bundle sequencing** — confirm a complete, self-consistent chunked md1/mk1 set on-device before engraving; device-side counterpart to `me bundle`. | M | T2 (decode) | md #2; mk #cross-chunk; app-layer #4 |
| **T6** | **`bundle` flagship + `verify-bundle` + `restore` doc** — derive ms1+mk1+md1 from one seed, engrave all three; read cards back and cross-check parity; show a watch-only restore document (fingerprint + first addrs + descriptors). | L | T1, T2, T4 | toolkit #1/#2/#3 |
| **T7** (opt) | Small niceties — `seedqr encode→engrave`, `final-word` (largely shipped), `convert phrase↔ms1` gap-fills, BIP-85 `derive-child` engrave. | S each | — | toolkit #6/#7 |

**Recommended sequencing:** T1 and T2 first (cheap, high-value, mostly wiring/deterministic, validate the verify-UX direction); then T3, T4, T5; then T6 (the flagship, which composes the rest). T7 opportunistic. Given **no hardware yet to validate the UX**, the T1/T2 wiring wins are the right place to start — they prove the direction cheaply before the heavy T6 engine.

---

## SKIP (explicit non-goals, confirmed by all 5 surveys)

- **Generate / split** — seed-xor split, slip39 split, ms-shares (codex32) split, fresh-wallet/seed generation. **HW-BLOCKED**: no app-accessible CSPRNG/TRNG (the reason on-device SPLIT was already deferred). (Also: SLIP-39 *sharing* math is a stub in the fork — word-list only, no GF(256)/Shamir — so a split mirror would need porting AND is blocked anyway.)
- **Wallet / signer** — PSBT construction/signing, transaction building, multisig coordination, child-xpub-for-spending, `shibboleth-wallet` (which is a *planning stub*, not running code). Off-mission; PSBT + a miniscript engine are confirmed **absent** from the firmware. SH never exports private keys.
- **Authoring** — descriptor construction, the miniscript policy compiler, template-text parsing, `build-descriptor`. Wrong I/O for a touchscreen keypad; host-tool concerns. (SH already engraves a *finished* descriptor.)
- **PC / file-shaped** — `import-wallet`/`export-wallet` (11 vendor formats), `electrum-decrypt`, `compare-cost`, `xpub-search`/passphrase brute-force, `nostr`, `verify-message` (base64-on-a-keypad is brutal). The device I/O model (engrave + NFC of public cards + hand-typed secrets) doesn't fit file blobs.
- **Host / CI tooling** — `vectors`, `gui-schema`, the `me` host bundle/preview pipeline.
- **The GUI** (mnemonic-gui) — desktop; SH has its own touchscreen UI.

---

## Per-component top candidates (distilled; see verbatim reports for full tables)

- **ms-codec:** decode-display (ms1→words) · `mnem` wordlist-language carry/read · inspect (share k-of-N) · derive (fp + account xpub) · verify. SKIP: split (CSPRNG), vectors/gui-schema.
- **md-codec:** decode→BIP-388 template display (top) · multi-chunk reassembly+integrity · policy/template fingerprint display · intake hygiene (separator-strip, mixed-case). SKIP: address derivation (heavy miniscript — but see app-layer correction), policy compiler, template parsing, re-encode.
- **mk-codec:** decode→xpub+fp+path+stubs display (top) · seed→account-xpub→mk1 engrave · field-assembly encode. ZERO HW-blocked (all public/deterministic). SKIP: address rendering, child-xpub, md1-stub cross-binding, SLIP-132 normalize.
- **mnemonic-toolkit:** `bundle` (seed→3 cards, flagship) · `verify-bundle` · `restore` watch-only doc · `addresses` · `inspect`/`convert mk1→xpub`. SKIP: all split, import/export/build-descriptor, electrum-decrypt, compare-cost, xpub-search, nostr.
- **application-layer:** address derivation+display (top — *wiring*, crypto in-tree+tested) · receive-address verification · descriptor/xpub verification display · on-device bundle sequencing · plate preview. SKIP: PSBT/signing (absent), multisig coordination, BIP-85/nostr/silent-payment, wallet export.

---

## Cross-cutting hardware/security constraints (the answer-space boundary)

- **No on-device CSPRNG/TRNG** for app code → only DETERMINISTIC transforms (decode/encode/correct/combine/derive-public/verify/display) are feasible; anything that mints fresh entropy is hardware-blocked.
- **Secrecy spine:** secret material (BIP-39 / ms1 / SLIP-39 shares / Seed-XOR parts / xprv) is hand-typed and NEVER over NFC/RF; md1/mk1/xpub/descriptor/address are PUBLIC → NFC-safe; passphrase strings are never engraved or transmitted.
- **SH is a backup/ENGRAVING device, not a signer** — public derivation (xpub/fingerprint/address) is in scope; anything touching spending/signing is not.
- **No camera** — address/signature *input* is hand-typed or NFC; QR is output-only.
- **Footprint** — RP2350 flash/RAM bounded; prefer reusing in-tree primitives over porting heavy stacks (miniscript address-eval, multi-language wordlists, etc.).
