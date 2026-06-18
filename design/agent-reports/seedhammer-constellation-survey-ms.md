<!--
Persisted verbatim. Constellation capability survey (2026-06-18), agent 1 of 5: ms-codec /
mnemonic-secret. Question: "for each ms-codec feature, should SH gain an on-device capability?"
Synthesis + roadmap: design/RECON_seedhammer_constellation_terminal.md. Agent's report below as
returned.
-->

## ms-codec / mnemonic-secret feature inventory → SeedHammer on-device verdicts

ms1 = codex32/BIP-93 secret-sharing strings over GF(32) with BCH(93,80,8) (t=4 correct / 8 detect). The component encodes **BIP-39 entropy** (`entr`, prefix `0x00`) or **entropy + wordlist-language hint** (`mnem`, prefix `0x02`, lang byte 0-9). Library API in `crates/ms-codec/src/{encode,decode,inspect,shares,payload}.rs`; CLI in `crates/ms-cli/src/cmd/*`.

| Feature | What SH could do on-device | Verdict | Rationale | Effort | Feasibility / security notes |
|---|---|---|---|---|---|
| **Encode** entr — `encode(Tag::ENTR, Entr)` / `ms encode --hex\|--phrase` | Hand-type a BIP-39 phrase/entropy on the device → engrave the ms1 string | **SHIPPED** | SH already does ms1 verbatim engrave + BIP-39 seed entry. Encoding from an entered seed is the existing engrave path. | — | Deterministic. Source seed hand-typed (security spine intact). |
| **Decode** entr → entropy/BIP-39 mnemonic — `decode()` / `ms decode` | Type the engraved ms1 → display the recovered entropy hex / BIP-39 words on screen | **GAP** | SH does ms1 typed entry + multi-share *recovery to the unshared secret*, but exposing the **single-string ms1 → show the entropy/mnemonic words** read is the highest-value missing deterministic transform. Read your own plate back. | **S** | Deterministic, no entropy. Display only, never NFC. Decode is the trivial K=1 case of interpolation SH already has. |
| **Mnem decode** — language byte on the wire (`Payload::Mnem`, `MNEM_LANGUAGE_NAMES`) | When decoding an ms1, read the embedded wordlist-language byte and show "language: japanese" | **GAP** | Core reason `mnem` exists: a non-English wallet recovered as English silently yields wrong addresses. Surfacing the on-wire language removes a catastrophic silent-failure mode. | **S** | Deterministic. One byte → table lookup (10 languages). Needs the non-English BIP-39 wordlists on-device (font/space cost). |
| **Mnem encode** — emit language-tagged ms1 for a non-English seed | When engraving an ms1 for a non-English seed, write the `mnem` form so the plate self-documents its wordlist | **GAP** | Symmetric; makes the plate self-correct rather than relying on a separate note. | **S-M** | Deterministic. Slightly longer string; engraver must accept the extra char. |
| **Error-correct** — `decode_with_correction` / `ms repair` (BCH t=4) | Type a damaged ms1 → repair up to 4 substitutions, show what it fixed | **SHIPPED** | SH already does codex32/ms1 BCH error-correction on typed entry. | — | (A standalone "here's what I corrected" report would be a small GAP, but the capability is shipped.) |
| **Validate / verify** — `Payload::validate()`, `ms verify` | After typing an ms1, show OK/structural verdict; optionally confirm round-trip to a re-typed phrase | **GAP** (lightweight) | A "is this plate still readable/valid?" check is a natural air-gapped maintenance ritual distinct from full recovery. | **S** | Deterministic. The verdict is already computed by SH's correcting parser. |
| **Inspect** — `inspect()` / `ms inspect` (lenient structural dump; detects "this is one share, needs K") | Show "this is a K-of-N share, threshold k, id ____, index ____" vs "single secret" and why a bad string fails | **GAP** (lightweight) | Strong recovery-UX win: tells the user what they're holding + how many more shares they need. | **S-M** | Deterministic, read-only. Screen real-estate is the constraint; "share k-of-N, need K" is the high-value subset. |
| **Combine / interpolate** K-of-N → secret — `combine_shares` / `ms combine` | Type K codex32 shares → interpolate the secret-at-S, show recovered entropy/phrase | **SHIPPED** | SH does "multi-share recovery (Interpolate to the unshared secret)." | — | Deterministic. SECRET shares hand-typed, never NFC. |
| **Split / generate shares** — `encode_shares(k,n)` / `ms split` | Take a secret → produce N fresh K-of-N shares | **HW-BLOCKED** | `encode_shares` calls `getrandom::fill` for the random share-set `id` AND the `k-1` random defining-share payloads. SH has no app-accessible TRNG/CSPRNG. | — | The deferred on-device SPLIT. Unblocked only by a hardware entropy source. |
| **Derive** — master fingerprint (+ account xpub via BIP-44/49/84/86 template) — `ms derive` | From a typed/decoded ms1, derive master fingerprint + account xpub for a chosen template/account/network; engrave or display the xpub | **GAP** | Read-only PUBLIC derivation (fingerprint + xpub only — no xprv, no signing). High value for verifying which wallet a plate belongs to and producing the watch-only xpub the mk1 sibling engraves. | **M** | Needs secp256k1 + BIP-32 on TinyGo. No private key leaves. Optionally engrave the xpub (public, NFC-safe) — bridges into the mk1 path. |
| **Test-vector corpus dump** — `ms vectors` | — | **OUT-OF-SCOPE** | Developer/CI artifact. | — | — |
| **GUI flag-schema emit** — `ms gui-schema` | — | **OUT-OF-SCOPE** | Tooling for mnemonic-gui's schema-mirror CI gate. | — | — |
| **`--json` / grouped-display / separators / stdin** | — | **LOW-VALUE** | Host-CLI I/O ergonomics; grouped-display (every-5-chars) is the only piece with a device analog, cosmetic. | S | — |
| **Secret-hygiene layer** — Zeroizing, mlock, no-echo | — | **OUT-OF-SCOPE** (as features) | Host-OS process-hardening; SH is bare-metal air-gapped firmware. The principles (scrub buffers, never echo) are good firmware discipline but not ms-codec capabilities. | — | — |

### Top SH candidates from ms-codec (ranked)
1. **Decode display (single ms1 → entropy / BIP-39 words)** — S. Biggest missing read: verify an engraved plate without a host. Trivial K=1 of the interpolation SH already runs.
2. **Mnem wordlist-language on decode (and encode)** — S. Eliminates the catastrophic non-English silent-wrong-wallet failure. Main cost = shipping non-English wordlists.
3. **Inspect ("share k-of-N, need K" / single vs share / failure reason)** — S-M. Tells the user what they hold + how many more shares to find.
4. **Derive (master fingerprint + account xpub, watch-only)** — M. Identify a plate's wallet + feed the mk1 xpub-engrave path; secp256k1/BIP-32 on TinyGo is the only real cost.
5. **Verify (valid-plate green/red, optional phrase round-trip)** — S. Lightweight maintenance check.

**HW-BLOCKED:** `split`/`encode_shares`. **OUT-OF-SCOPE:** `vectors`, `gui-schema`, host process-hardening. **SHIPPED:** encode (verbatim engrave), BCH correction, K-of-N combine/interpolate.
