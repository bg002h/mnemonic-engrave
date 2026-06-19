# RECON (T6, agent a3420a4920e4abbb4, 2026-06-19) — scope + security model (source-verified)

Recon only. The flagship "derive ms1+mk1+md1 from one seed → engrave all + verify-bundle + restore doc."

## 0. FRAMING CORRECTION (verify before speccing)
T6 is NOT host `me bundle`, and NOT the shipped device `engraveBundle`/`bundleFlow` (T5). Three distinct things:
- **Host `me bundle`** (`crates/me-cli/src/bundle.rs:1,49`): a public-string VALIDATOR (ingests md1/mk1, proves chunk-set integrity, manifest); HARD-REFUSES ms1 (exit 3); NEVER derives from a seed; no restore/verify/derive subcommand (`main.rs:38` — `Bundle` is the only one). The host has NO seed→3-card equivalent.
- **Device `engraveBundle` (T5, shipped):** `gui/bundle_flow.go:16` — gather PUBLIC md1/mk1 over NFC, engrave verbatim; ms1 REFUSED; derives nothing.
- **The seed→3-card flagship** lives in `mnemonic-toolkit` (`bundle_unified.rs`, `cmd/bundle.rs`, `restore`, `verify-bundle`). **THIS is what T6 ports to-device.** So "the host equivalent" = the TOOLKIT, not `me`.
⇒ T6 is a NEW `program` (parallel to T4's `engraveXpub`), NOT an extension of `bundleFlow`. It introduces a new device capability: take a SECRET seed, derive the full constellation.

## 1. THE SCOPE FORK — single-sig vs multisig
Constellation modes (`bundle_unified.rs:14-26`): SingleSigFull (N=1 secret @0), SingleSigWatchOnly (N=1 watch-only), MultisigMultiSource (N≥2 all secret), MultisigWatchOnly (N≥2 all watch-only), MultisigHybrid (N≥2 mix). Toolkit assembles multisig from per-slot `--slot @idx.xpub=…` inputs (`cli_bundle_multisig.rs:35-42`); one seed fills ONE slot. **Self-multisig (one seed → all N cosigners) was HARD-REJECTED in v0.4.0, "no migration path"** (`cli_bundle_multisig.rs:7-9`). Only honest multisig is multi-source (need the OTHER cosigners' xpubs).
Options: **(A) single-sig only** (one seed → complete ms1+mk1+md1 over the one key; = SingleSigFull; no external input; clean/self-contained); **(B) single-sig + multisig-with-scanned-cosigners** (your seed = your slot; scan co-signers' mk1/policy over NFC [PUBLIC→safe]; assemble multisig md1; = MultisigHybrid); **(C) defer multisig**.
**RECOMMENDATION: (A) single-sig only this cycle; explicitly DEFER (B); never self-multisig.** Rationale: (1) single-sig is the ONLY scope where "one seed→complete ms1+mk1+md1" is literally true; (2) cosigner-input is a real 2nd feature (NFC-gather foreign mk1/md1 + slot-assignment UI + threshold/template picker + "which slot am I?"), dwarfs (A), no hardware to validate UX; (3) clean composition path later (MultisigHybrid + scanned watch-only slots) — (A) is the spanning subset, not a dead-end; (4) toolkit's own "cleanest first slice" = single-sig-full + watch-only-2-card.
**SPEC must lock (A); restrict the path picker to the 4 single-sig types** (T4 ships a 6-type picker incl. BIP-48/87 multisig — T6(A) must NOT offer multisig paths it can't complete). **USER CALL — headline.**

## 2. ms1 = the secret; the "engrave the seed" question
**ms1 IS the secret** (`ms-codec/src/lib.rs:3-4,29`): "ms1 = backup format for BIP-39 ENTROPY, layered on BIP-93 codex32." Tag `entr`, payload = BIP-39 ENTROPY (16-32 bytes), NOT the 64-byte seed (master-seed/xpriv payloads "reserved-not-emitted v0.1"). Chain (`lib.rs:35-36`): phrase→entropy→ms1→engrave→recover→mnemonic→PBKDF2→seed. So ms1 stores entropy; seed re-derived via wordlist+PBKDF2. Wordlist-LANGUAGE matters (`mnem` field; a Japanese `mnem` → different fingerprint, `cli_restore.rs:31-33`; T2a handles on-device).
**SHARPEST FINDING — ms1 vs backupWallet:** the fork's `backupWallet` engraves the seed TODAY as BIP-39 WORDS + SeedQR, NOT ms1 (`gui/gui.go:459` `engraveSeed`→`backup.Seed{Mnemonic,QR:seedqr…}`; `backup.go:15`). So the device's existing secret backup = upstream word-plate. **T6 adding ms1 engrave does NOT duplicate backupWallet** — it adds a CONSTELLATION-FORMAT secret backup (codex32, BCH-protected, sibling-consistent with mk1/md1).
Spine consistency: engraving ms1 onto OWNER-HELD steel is the device's core purpose — secret never on a wire. NO spine tension (tension would be NFC-transmitting it, which T6 must not). 
**Sub-decisions:** (2a, USER) ms1 vs word-plate for the secret card (ms1 = constellation-native; offer choice/both); (2b, RECOMMEND yes) offer a watch-only/skip-ms1 mode (mk1+md1 only; = toolkit SingleSigWatchOnly `cli_bundle_multisig.rs:62`; lowest exposure; pairs with "seed already backed up").

## 3. verify-bundle — cross-check parity
Host (`cli_verify_bundle_seedqr_slot.rs:30-72`): `verify-bundle --slot @0.phrase=… --template … --ms1 … --mk1 … --md1 …` — re-derives the constellation from the secret + confirms supplied cards match (byte-equal whether secret via phrase or seedqr). Toolkit "9-point parity" (entropy/fp/xpub/path/descriptor across 3 card types) → PASS/FAIL.
**Device behavior:** a confidence check before trusting the steel: (1) re-type seed (SECRET, typed-only); (2) read engraved cards back — mk1/md1 over NFC (PUBLIC, T1/T2 decode shipped), ms1 HAND-TYPED (SECRET, T2a entry shipped, never NFC); (3) re-derive + compare on master-fp, account-xpub(s), path(s), descriptor/template, ms1-entropy (+wordlist-lang); show PASS/FAIL + which field diverged. Deterministic recompute-and-diff. **SPEC lock the parity field set.** Edge: watch-only mode → skip ms1 leg. **USER (minor):** inline-after-engrave vs separate program (recommend both; re-entry re-derives deterministically).

## 4. restore doc — watch-only document
Host (`cli_restore.rs:1-9,86-126`): `restore --from phrase=… [--template …]` → watch-only doc, NEVER private material (negative test greps for xprv/tprv absence). Fields: master fingerprint (path-independent, `73c5da0a`); a CONFIRM line; per script-type the concrete descriptor w/ `#checksum` + multipath `<0;1>` (e.g. `DESC_BIP84=wpkh([73c5da0a/84'/0'/0']xpub…/<0;1>/*)#hpg6d6w2`); first receive addr (`bc1qcr8te4…` = BIP-84 m/84'/0'/0'/0/0 vector); optional `--expect-*` reference → mismatch exit 4.
**Device:** display-only (NO secret) screen/QR: master fp + concrete descriptor(s) + first receive+change addr (via `address.Receive`/`Change`, T1/T3 wired). All PUBLIC → screen + NFC-exportable. **SPEC lock:** display-only vs also-engravable (descriptor already on the md1 plate); 1 addr vs small gap-range. **USER (minor):** own engraved plate or screen/NFC-only (recommend screen + optional NFC).

## 5. Security spine (highest-exposure tier — holds seed + derives everything)
MUST-HOLD (each Critical if violated), extending the shipped T4 spine (`SPEC_T4 §2.5`, `gui/derive_xpub.go:98-113`):
1. Seed/mnemonic/passphrase SECRET → typed-only, NEVER NFC. **NEW FOOTGUN FLAG:** `gui/scan.go:61-70` CAN parse a `bip39.Mnemonic` + `codex32.New` secret from NFC TODAY. **T6 spec MUST explicitly assert its seed input is typed-only and never consumes a scanned bip39/codex32 object** — verify the T6 program's `uiFlow` dispatch (`gui.go:1500`) takes the typed `seedEntryFlow`/`newInputFlow` path, NOT `act.scan`. (D12, Critical)
2. `.Neuter()` every key before any serialization — no xprv ever serialized/displayed/engraved/NFC'd (host restore greps xprv-absent).
3. Scrub-complete (T4 §54): `wipeBytes` the 64-byte PBKDF2 seed; `.Zero()` master + EVERY intermediate `*ExtendedKey`; zero the mnemonic `[]Word`; capture fp BEFORE zeroing master.
4. ms1 engraved onto owner-held steel only; never NFC; scrub the entropy buffer after encode.
5. mk1/md1/xpub/addresses PUBLIC → NFC/screen/engrave safe.
6. restore doc carries NO secret.
7. passphrase never engraved/transmitted; no-pp vs pp fingerprint follows `backupWalletFlow` (`gui.go:2013-2034`).
8. set-level all-or-nothing + abort warning (T4 §67/T5): partial bundle NOT usable; re-entry re-derives from scratch, no half-state.
**NEW EXPOSURE beyond T4/T5:** (a) LONGEST secret residency — T6 holds the seed across THREE derivations (entropy→ms1, xpub→mk1, descriptor-key→md1) + addresses + possible verify re-hold → scrub must cover EVERY leg, re-zero seed after the LAST consumer (restore-doc addr derivation), not the first. (b) entropy (ms1) AND seed (mk1/md1) co-resident — entropy is the seed's pre-image, a 2nd copy of the root secret → BOTH scrubbed. (c) verify-bundle re-holds the secret (fresh window each pass). (d) multi-plate engrave holds derived PUBLIC material across frames (not secrecy, but the set-abort/incompleteness UX is a backup-INTEGRITY risk). **USER (security):** verify-bundle re-type-each-time vs hold-through (recommend re-type — shorter residency).

## Decision table (LOCK / USER)
| # | Decision | Recommendation | USER? |
|---|---|---|---|
| D1 | single-sig (A) / +multisig-scan (B) / defer (C) | (A) only; defer (B); never self-multisig | **YES headline** |
| D2 | engrave ms1 as the secret card? | yes, operator-choosable | YES |
| D3 | ms1 (codex32) vs upstream word-plate for the secret | ms1 = constellation-native; offer choice/both | YES |
| D4 | watch-only/2-card mode (mk1+md1, skip secret)? | YES (toolkit SingleSigWatchOnly; lowest exposure) | YES |
| D5 | picker: restrict to 4 single-sig types under (A)? | YES | spec-lock |
| D6 | verify-bundle parity field set | fp + xpub(s) + path + descriptor-checksum + ms1-entropy + wordlist-lang; PASS/FAIL | spec-lock |
| D7 | verify: inline / separate / both | both; re-entry re-derives | minor USER |
| D8 | verify secret model: re-type vs hold-through | re-type (shorter residency) | YES security |
| D9 | restore doc: display-only vs engravable; #addrs | display + optional NFC; 1 recv+1 change | minor USER |
| D10 | T6 = NEW program, NOT bundleFlow extension | new program; reuse T4 derive + T5 sequencing | spec-lock |
| D11 | per-leg scrub schedule (longest-residency) | zero each of {entropy,seed,master,intermediates,mnemonic} after its LAST consumer | spec-lock Critical |
| D12 | assert seed typed-only; never scanned bip39/codex32 (scan.go can parse) | explicit assertion + test | spec-lock Critical |

## Bottom line
T6 = a NEW on-device `program` taking a typed-only seed → deterministically derive the SINGLE-SIG constellation (entropy→ms1, account-xpub→mk1, descriptor→md1) → engrave the set (operator-choosable watch-only/skip-ms1) → deterministic verify-bundle read-back parity (mk1/md1 NFC, ms1 hand-typed) → watch-only restore doc. Reuses T4's scrub-complete derivation + T5's multi-plate sequencing; NOT the public-only bundleFlow. Multisig → MultisigHybrid follow-on. Headline user calls: D1 (scope), D2/D3/D4 (ms1 policy), D8 (verify residency). Build gap (other recon): md1-from-seed needs new md API.
