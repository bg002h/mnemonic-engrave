# cycle-prep recon — 2026-06-19 — T6 flagship (seed → ms1+mk1+md1 → engrave + verify-bundle + restore doc)

**Fork HEAD:** `e4013a8` (T5 shipped). **Recon agents (parallel, source-verified):** `a744d5d7478925cb5` (device-side build feasibility — `design/agent-reports/seedhammer-T6-recon-build-feasibility.md`), `a3420a4920e4abbb4` (scope + security model — `design/agent-reports/seedhammer-T6-recon-scope-security.md`).

## HEADLINE
T6 is a **NEW on-device `program`** (parallel to T4's `engraveXpub`, NOT an extension of T5's public-only `engraveBundle`/`bundleFlow`): take a typed-only SECRET seed → deterministically derive the constellation (entropy→ms1, account-xpub→mk1, descriptor→md1) → engrave the set → verify-bundle read-back parity → watch-only restore doc. It ports the **`mnemonic-toolkit`** flagship (`bundle_unified.rs`/`restore`/`verify-bundle`), NOT host `me bundle` (which is a public-string validator that refuses ms1 and derives nothing).

## Two pivotal findings
1. **BUILD GAP (make-or-break): md1-from-seed needs NET-NEW md-package API.** The md1 encoder (#10a) is byte-faithful but ENTIRELY UNEXPORTED + test-only — it consumes the md-internal `*descriptor` AST (the `body` interface has an unexported `isBody()` → unconstructible outside package `md`), and the exported decode path (`Template`) is LOSSY (not round-trippable to the AST). Producing md1 from a self-derived descriptor requires a new exported entry — recommend a NARROW `md.EncodeSingleSig(xpub, origin, useSite, script) ([]string, error)` that builds the simplest AST (n=1, one key, one pubkey TLV) internally + calls `split`/`encodeMD1String`. New PUBLIC surface on a Rust-golden-byte-locked package → the DOMINANT T6 risk; must be R0-gated + external-protocol-fact-verified vs the Rust md-codec. Everything else (mk1 via T4 chain; ms1 via `codex32.NewSeed`+`bip39.Entropy()`; engrave via T5's `bundleEngrave([]bundleCard)` synthesized from derived strings; restore-doc via `address.Receive/Change` + a from-xpub `*bip380.Descriptor`; verify-bundle via T5 gather + a NEW deterministic comparator) is reuse-with-glue.
2. **SCOPE FORK: single-sig is the only scope where "one seed → complete ms1+mk1+md1" is literally true.** A multisig md1 references N cosigner xpubs; one seed produces ONE. The constellation HARD-REJECTED self-multisig ("no migration path"). Honest multisig = multi-source (scan the OTHER cosigners' xpubs → MultisigHybrid). → recommend single-sig-only this cycle, defer multisig.

## ms1 = the SECRET (not a public artifact)
ms1 encodes BIP-39 ENTROPY (16-32B, tag `entr`) as BIP-93 codex32; the seed is re-derived via wordlist+PBKDF2 (wordlist LANGUAGE matters). The fork's `backupWallet` engraves the seed TODAY as BIP-39 WORDS+SeedQR, NOT ms1 — so T6's ms1 engrave is NET-NEW + constellation-native (codex32, BCH-protected, sibling to mk1/md1), NOT a duplicate of backupWallet. Engraving ms1 onto owner-held steel is the device's core purpose (no spine tension); the invariant is ms1 typed/derived→engraved-only, NEVER NFC.

## Security spine (highest-exposure tier — holds the seed + derives EVERYTHING)
Extends the shipped T4 scrub discipline. NEW exposures: LONGEST secret residency (seed held across 3 derivations + addresses + possible verify re-hold → scrub every leg, re-zero after the LAST consumer); entropy (ms1) + seed (mk1/md1) co-resident (both scrubbed); verify re-holds the secret. **CRITICAL FOOTGUN (D12):** `gui/scan.go:61-70` CAN parse a bip39 mnemonic + codex32 secret from NFC today — T6 MUST assert its seed input is typed-only (`seedEntryFlow`/`newInputFlow`), never `act.scan`, + test it.

## Decision table (full table + citations in the scope-security recon)
- **D1 (USER, headline):** single-sig only (A) / +multisig-scan (B) / defer (C). **Rec: (A), defer (B), never self-multisig.**
- **D2/D3/D4 (USER):** engrave ms1? (rec yes) · ms1 (codex32) vs upstream word-plate for the secret (rec ms1, constellation-native; maybe offer choice) · offer a watch-only/skip-ms1 mode (mk1+md1 only)? (rec yes — lowest exposure, toolkit SingleSigWatchOnly).
- **D5 (lock):** restrict the path picker to the 4 single-sig types under (A).
- **D6 (lock):** verify-bundle parity = fp + xpub(s) + path + descriptor-checksum + ms1-entropy + wordlist-lang → PASS/FAIL.
- **D7 (minor USER):** verify inline / separate / both (rec both).
- **D8 (USER, security):** verify secret model re-type vs hold-through (rec re-type, shorter residency).
- **D9 (minor USER):** restore doc display-only vs engravable; #addrs (rec display + optional NFC; 1 recv + 1 change).
- **D10 (lock):** NEW program, reuse T4 derive + T5 sequencing.
- **D11 (lock, Critical):** per-leg scrub schedule (zero each of {entropy,seed,master,intermediates,mnemonic} after its LAST consumer).
- **D12 (lock, Critical):** seed typed-only; never consume scanned bip39/codex32.

## Effort + phasing (one cycle, after the scope call)
- **Phase A — headless:** the NEW `md.EncodeSingleSig` (+ R0 + golden parity vs Rust) — the make-or-break; the ms1-derive glue (`Entropy()`→`NewSeed`); the verify-bundle comparator (deterministic exact-match).
- **Phase B — GUI:** new `program` (8-site lockstep) + typed seed entry (reuse, typed-only) + single-sig path/script picker (4 types) + derive all 3 + the watch-only/skip-ms1 mode + engrave sequence (reuse `bundleEngrave`, T6 completion message) + verify-bundle flow + restore-doc screen.
- Likely the largest tier; R0 should assess whether to split (e.g. #T6a `EncodeSingleSig`+derive-core / #T6b GUI+verify+restore) — analogous to the #10a/#10b split.

## Gate reminder
`SPEC_seedhammer_T6_*` MUST pass opus R0 to 0C/0I before code. The NEW md API + the per-leg scrub schedule + the typed-only seed assertion are the must-verify items. External-protocol-fact rule applies to `EncodeSingleSig` (verify the md1 single-sig wire/canonicalization vs Rust md-codec). Fork-side only; no upstream PR.

## NEXT
Surface D1 + the ms1 policy (D2/D3/D4) + D8 to the USER (the rest defaulted per the recon recommendations), then write `SPEC_seedhammer_T6_*` → R0.
