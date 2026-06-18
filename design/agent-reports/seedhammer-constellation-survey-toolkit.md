<!--
Persisted verbatim. Constellation capability survey (2026-06-18), agent 4 of 5: mnemonic-toolkit
(25 subcommands). Synthesis: design/RECON_seedhammer_constellation_terminal.md. Agent's report below.
-->

# SeedHammer II on-device capability survey vs. mnemonic-toolkit

Verdict legend: GAP / SHIPPED / HW-BLOCKED (needs fresh entropy) / OUT-OF-SCOPE (PC/file-oriented) / LOW-VALUE. SH I/O: input = touchscreen keypad + NFC; output = NFC + engraving (text/QR) + screen. Secret material (seed words, ms1 entropy, SLIP-39 shares, Seed-XOR parts, xprv/WIF) hand-typed, NEVER over NFC; public (md1/mk1/xpub/descriptor/address) may go over NFC; passphrases never engraved/transmitted.

## Bundle / synthesis
| Operation | What SH could do | Verdict | Rationale | Effort | Notes |
|---|---|---|---|---|---|
| `bundle` full 3-card (seed → ms1+mk1+md1) | Hand-type a seed (+passphrase,+template), derive entropy→ms1, xpub→mk1, policy→md1, engrave all three | GAP | The toolkit's flagship; the raison d'être of `mnemonic-engrave`. Pure deterministic transform from a *supplied* seed — NOT entropy generation. | L | Needs the BIP-388/template engine + xpub derivation on TinyGo. Largest single GAP; engraving-native. |
| `bundle` watch-only 2-card (xpub → mk1+md1) | Enter xpub+origin+policy (public), engrave mk1+md1 | GAP (subset) | All-public, deterministic, NFC-friendly. Cleanest first slice. | M | No secret on device at all. |
| `verify-bundle` (re-derive + cross-card parity) | Read 3 cards back (NFC mk1/md1; type ms1), re-derive, confirm 9-point parity, show PASS/FAIL | GAP | Deterministic; high backup-integrity value ("did I engrave these correctly?"). | M | Pairs with the bundle GAP. |

## Convert / derive
| Operation | What SH could do | Verdict | Rationale | Effort | Notes |
|---|---|---|---|---|---|
| `convert phrase/entropy → ms1` | Type a seed, engrave as ms1 (codex32) | SHIPPED-adjacent / minor GAP | SH does BIP-39 entry AND ms1 entry+engrave separately; the seed→ms1 single flow may not exist. Both SECRET, off NFC. | S | Small on-brand GAP if not a one-step flow. |
| `convert ms1 → phrase/entropy` | Recover seed words from an ms1, display/engrave as BIP-39 | SHIPPED | codex32 entry+recovery shipped. | — | Confirm the recovered-as-BIP-39 display exists; if not, trivial GAP. |
| `convert phrase/entropy → xpub / fingerprint / xprv` | Derive account xpub or master fp from a typed seed; display (public) or engrave | GAP | Deterministic. Master-fingerprint = the passphrase-correctness oracle + verification primitive. Output PUBLIC (NFC-able). | M | High verification value. fingerprint+xpub display is the lightweight slice. |
| `convert phrase/entropy/xpub → address` | Derive receive address(es); display or engrave a verification QR | GAP | Deterministic public derivation. | M | See `addresses`. |
| `convert xprv → xpub/fingerprint` | Neuter a typed xprv to its public form | GAP (LOW-VALUE) | Niche on a keypad. | S | xprv hand-entry long/error-prone. |
| `convert wif↔xpub/fingerprint`, `wif↔bip38`, `→wif`, `minikey→wif`, `electrum-phrase↔entropy`, `xpub→xpub (SLIP-132)` | WIF/BIP-38/Casascius/Electrum/prefix handling | LOW-VALUE / OUT-OF-SCOPE | Single-key/legacy/PC artifacts tangential to a seed-backup device. | S–M | Mostly off-mission. |
| `convert mk1 → xpub/fingerprint/path` | Read an mk1 (NFC public), extract for display | GAP | Deterministic public decode; complements `inspect`. | S | Fold into inspect. |
| `convert → mk1` / sibling pivots (ms1↔md1) | (refused by toolkit) | OUT-OF-SCOPE | Toolkit refuses; mk1 needs a policy binding (→ bundle). | — | — |
| `derive-child` (BIP-85) | Type master seed, derive a BIP-85 child phrase/xprv/hex at an index, engrave the child | GAP (LOW-VALUE) | Fully *deterministic* (derives from master, no RNG). The child *phrase* is the engravable angle. Power-user niche. | M | — |

## Address / xpub derivation + display
| Operation | What SH could do | Verdict | Rationale | Effort | Notes |
|---|---|---|---|---|---|
| `addresses` (batch receive/change from xpub or seed) | Derive + display receive/change addresses (optionally engrave first-address QR) for verification | GAP | Deterministic watch-only; confirm "this card controls these addresses." | M | Display-only is the primary win. No camera needed (derivation on-device). |
| `restore` single-sig watch-only doc (seed+passphrase → fingerprint + first addrs + descriptors) | After engraving, show a watch-only restore document | GAP | Deterministic; NO private keys out; fingerprint = passphrase oracle. "Prove your backup restores." | M | All PUBLIC; screen or NFC out. |
| `restore` multisig (md1 alone → concrete watch-only multisig descriptor) | Read the shared md1 (NFC public), reconstruct the watch-only multisig descriptor for display | GAP | Deterministic; md1 PUBLIC. Exactly the verification a multisig backup needs. | M | md1→descriptor expansion engine; pairs with descriptor engrave (shipped). |

## Backup splitting (split vs combine)
| Operation | What SH could do | Verdict | Rationale | Effort | Notes |
|---|---|---|---|---|---|
| `seed-xor split` | — | HW-BLOCKED | Uses `OsRng` for N−1 random masks. No app TRNG. | — | The deferred SPLIT class. |
| `seed-xor combine` | Type N XOR shares, recover BIP-39 | SHIPPED | Coldcard Seed-XOR combine (N-of-N) shipped. | — | — |
| `slip39 split` | — | HW-BLOCKED | Needs a CryptoRng for the identifier + random shares. | — | — |
| `slip39 combine` | — | SHIPPED | SLIP-39 recovery shipped. | — | — |
| `ms-shares split` | — | HW-BLOCKED | `encode_shares` consumes CSPRNG. | — | — |
| `ms-shares combine` (K-of-N codex32 recombine) | Type ≥K codex32 shares, recombine, display/engrave secret | SHIPPED-adjacent / minor GAP | Single-card recovery shipped; K-of-N multi-share recombine may not be wired. | S–M | Verify against shipped codex32 scope. |
| `seedqr encode` (BIP-39 → SeedQR) | Convert a typed seed to a SeedQR payload, engrave as QR | GAP (LOW-VALUE) | Deterministic; compact alternative seed backup. SECRET QR (consistent with engraving seed words). | S | Decode moot without a camera; encode-and-engrave is usable. |

## Keys & messages
| Operation | Verdict | Note |
|---|---|---|
| `final-word` (N−1 → valid last words) | GAP (LOW-VALUE) | Deterministic entry-aid; small. |
| `nostr` (wrap npub/nsec) | OUT-OF-SCOPE | Cross-protocol; off-mission. |
| `silent-payment` (BIP-352 sp1 receiver) | GAP (LOW-VALUE) | sp1 address PUBLIC/engravable; early adoption; keep privkeys local. |
| `verify-message` (signmessage/BIP-322) | LOW-VALUE / OUT-OF-SCOPE | Secret-free but base64-on-a-keypad is brutal; off-mission. |

## Decrypt / repair / inspect
| Operation | Verdict | Note |
|---|---|---|
| `repair --ms1/--mk1/--md1` (BCH correct) | SHIPPED | codex32/md/mk correct shipped. |
| `inspect` (describe ms1/mk1/md1 contents) | GAP | Deterministic decode + display; high "what is this card?" value; defaults to withholding ms1 entropy. Fold in `convert mk1→…`. S–M. NFC-friendly for public cards. |
| `electrum-decrypt` / `compare-cost` / `gui-schema` | OUT-OF-SCOPE | Electrum interop / wallet-design analytics / GUI introspection. |
| `xpub-search` (path/account/address/passphrase) | LOW-VALUE / OUT-OF-SCOPE | Search loops; diagnostic/forensic; passphrase brute-force inappropriate on RP2350. The single useful slice (fingerprint-as-passphrase-oracle) is covered by `restore`. |

## Import / export / descriptor construction
| Operation | Verdict | Note |
|---|---|---|
| `import-wallet` (vendor blob → bundle) | OUT-OF-SCOPE | Multi-KB file input; PC-side ingestion. |
| `export-wallet` (11 watch-only formats) | OUT-OF-SCOPE | PC-import files, not engraving artifacts. The descriptor engrave SH does is the relevant slice. |
| `build-descriptor` (JSON policy-tree → wsh) | OUT-OF-SCOPE / LOW-VALUE | Authoring a JSON tree on a keypad is impractical; build on PC, engrave on SH. |

## Passphrase / fingerprint (cross-cutting)
| Operation | Verdict | Note |
|---|---|---|
| BIP-39 passphrase flow + master-fingerprint choice | SHIPPED | Reused by the bundle/restore/convert GAPs. Passphrase never engraved/over-NFC. |
| Forgotten-passphrase recovery / keyspace generation | OUT-OF-SCOPE | Deferred to btcrecover; brute-force inappropriate on RP2350. |
| Any GENERATE / fresh-entropy op | HW-BLOCKED | No on-device CSPRNG. Covers seed-xor/slip39/ms-shares split + new-wallet generation. |

## Top SH candidates from mnemonic-toolkit (ranked)
1. **`bundle` (full + watch-only 2-card)** — derive 3 cards from a seed/xpub and engrave them. Flagship, engraving-native, deterministic. (L; watch-only 2-card first slice M.)
2. **`verify-bundle`** — read cards back, cross-check 9-point parity. (M)
3. **`restore` watch-only document** (single-sig + multisig). (M)
4. **`addresses` + `convert →address`** — derive/display receive addresses for verification. (M)
5. **`inspect` (+ `convert mk1→xpub/fingerprint/path`)** — decode + display a card's contents. (S–M)
6. **`convert phrase↔ms1`, `ms-shares combine`** — small deterministic gap-fills (if not shipped). (S–M)
7. **`convert →xpub/fingerprint`, `seedqr encode→engrave`, `final-word`, `derive-child`** — small niceties. (S–M each)

**Hard exclusions:** all split/generate (HW-BLOCKED); PC/file-shaped (import/export/build-descriptor/electrum-decrypt/compare-cost/gui-schema/nostr) OUT-OF-SCOPE; xpub-search not ported (the useful slice = fingerprint oracle, covered by `restore`). **Correction to intake:** `bundle` full mode is NOT entropy-generation — it deterministically derives from a *supplied* seed, so it is feasible on SH.
