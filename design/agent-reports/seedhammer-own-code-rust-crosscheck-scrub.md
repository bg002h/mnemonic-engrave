# Cross-implementation SECURITY check — secret-scrub / zeroize cluster (Go fork → primary Rust)

**Task:** A recent adversarial bug hunt on the SeedHammer firmware fork's OWN Go code found 8 defects (since fixed). Project rule: when a defect is found+fixed in a Go port, check whether the SAME defect exists in the primary Rust implementation; if so, fix Rust FIRST. This is a READ-ONLY check of the four **scrub-cluster** findings (M2/M3/M4/L1).

**Authoritative finding source:** `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/seedhammer-fork-own-code-bughunt.md`
**Primary Rust source checked:** `/scratch/code/shibboleth/mnemonic-toolkit` @ `6de53879` (HEAD `6de5387945bf4f5bdc9b73f04fb06c0de1bf9f34`), crate `mnemonic-toolkit`. Also surveyed `mnemonic-key`, `descriptor-mnemonic`.
**Date:** 2026-06-20.

**Idiom note applied:** In Rust, a buffer wrapped in `Zeroizing<...>`, or owned by a type with a `Drop`/`Zeroize`/`ZeroizeOnDrop` impl, IS correct scrubbing on ALL paths (including error/early-return) by RAII — even with no explicit wipe call at the use site. A raw `Vec<u8>`/`[u8; N]`/`String` holding entropy that drops un-zeroized IS a gap.

**Crate-wide context:** `crates/mnemonic-toolkit/Cargo.toml:66` declares `zeroize = { version = "1.8", features = ["derive"] }`. The `zeroize` crate / `Zeroizing` wrapper is used pervasively across the secret modules (slip39, seed_xor, bip85, secrets, derive, etc.). This is an idiomatic-RAII-zeroization codebase, NOT a manual-wipe codebase like the Go fork.

---

## Per-finding verdict table

| Finding | Go gap (fork, since fixed) | Rust counterpart | VERDICT | Rust evidence |
|---|---|---|---|---|
| **M2** | slip39 `Combine`: recovered group-share secrets (`groupShares[].v`, `ems`, `d`) un-wiped on 3 error returns | `mnemonic_toolkit::slip39::{slip39_combine, recover_secret}` | **GO-ONLY** | All secret intermediates are `Zeroizing<Vec<u8>>` → RAII-scrubbed on every path incl. `?`-error early returns. `slip39/mod.rs:259,292,296,312,433,444,445` |
| **M3** | seedxor `Combine`: per-part BIP-39 entropy copies (`parts[0].Entropy()`, `e := p.Entropy()`) un-wiped | `mnemonic_toolkit::seed_xor::{seed_xor_combine, seed_xor_split, seed_xor_split_deterministic}` | **GO-ONLY** | `seed_xor_combine` takes `&[&[u8]]` (borrows; no per-part copies). Split returns `Vec<Zeroizing<Vec<u8>>>`; det-split's scratch `buf` explicitly `zeroize()`'d. `seed_xor.rs:86,95,98,140,161,171` |
| **M4** | bip85 leaf EC privkey object `pkey` never `Zero()`'d | `mnemonic_toolkit::bip85::derive_entropy` (+ `format_*` apps) | **GO-ONLY (different architecture; documented residual)** | Rust never materializes a standalone `*PrivateKey`; it reads `child.private_key.secret_bytes()` inline into an HMAC engine. 64-byte output is `Zeroizing<Vec<u8>>`. The `Xpriv`/`SecretKey` non-zeroize residual is a *known, separately-tracked* upstream-blocked FOLLOWUP, NOT the M4 gap. `bip85.rs:42-56,49-53,52` |
| **L1** | codex32 `DecodeMS1` probe entropy buffer discarded un-scrubbed (3 call sites) | `mnemonic_toolkit::cmd::bundle::self_check_bundle` ms1 self-check probe | **RUST-ALSO-AFFECTED (LOW)** | `cmd/bundle.rs:2473-2484`: `ms_codec::decode(ms)` → bare `Payload` used via `.as_bytes()` then **dropped un-`Zeroizing`-wrapped**; `ms_codec::Payload::Entr(Vec<u8>)` is explicitly NOT zeroize-wrapped and has NO Drop/Zeroize impl (`ms-codec-0.4.4/src/payload.rs:17-44`). All OTHER toolkit decode sites correctly move bytes into `Zeroizing` (slot_ms1.rs:50,74; silent_payment.rs:150-151; bundle.rs:2029,2038; overlay.rs:133,148). |

---

## Detail — M2 (slip39 group-share scrub on error paths)

**Go gap:** `Combine` wiped `groupShares[].v`/`ems` only on the success path; three error returns (digest fail, insufficient shares, group-layer digest fail) skipped the scrub, leaking fresh recovered-secret allocations. Fork fix = scrub all paths via defer + wipe `d`.

**Rust:** `slip39_combine` (`crates/mnemonic-toolkit/src/slip39/mod.rs:206-331`):
- `group_shares: Vec<(u8, Zeroizing<Vec<u8>>)>` (mod.rs:259) — the recovered group-share values (the Go `groupShares[].v` analogue) are held in `Zeroizing<Vec<u8>>`.
- Member-level point copies: `Zeroizing::new(s.value().to_vec())` (mod.rs:292-295) — intermediate share-value copies are Zeroizing too.
- `let gv = recover_secret(mt, &pts)?;` (mod.rs:296) returns `Zeroizing<Vec<u8>>`; pushed into `group_shares`.
- `let ems = recover_secret(gt, &group_shares)?;` (mod.rs:312) — `ems` is `Zeroizing<Vec<u8>>`.
- The strict-equal / consistency error returns (mod.rs:230-247, 264, 272-276, 282-287, 303-309) and the `?` propagation from `recover_secret`'s `DigestVerificationFailed` (mod.rs:453-455) all unwind through Drop, which zeroizes every live `Zeroizing<...>` (`group_shares`, the per-iteration `pts`, `gv`, `ems`).
- `recover_secret` (mod.rs:430-458): the interpolated `secret` (mod.rs:444) and `digest_payload` (mod.rs:445) — the Go `d` analogue — are BOTH `Zeroizing<Vec<u8>>`, so on the `DigestVerificationFailed` early return (mod.rs:453) BOTH are scrubbed by Drop. This is exactly the "also wipe `d`" the Go fix had to add manually.
- `Share`'s `value` field is `#[derive(Zeroize, ZeroizeOnDrop)]` (see share.rs evidence below), so the source shares are scrubbed too.

**VERDICT: GO-ONLY.** Rust already scrubs all recovered group-share secrets and the digest payload on every path (success + all three error returns) via `Zeroizing` RAII. No Rust action needed.

---

## Detail — M3 (seedxor per-part entropy intermediates)

**Go gap:** `Combine` scrubbed only the `out` accumulator; per-part `Entropy()` copies (fresh heap allocations) were never wiped, on success or on the mismatched-length error path. Fork fix = wipe each per-part entropy.

**Rust:** `crates/mnemonic-toolkit/src/seed_xor.rs`:
- `seed_xor_combine(shares: &[&[u8]])` (seed_xor.rs:161): the combine takes **borrowed** share slices — it does NOT make per-part owned entropy copies at all. There is no `Entropy()`-style fresh allocation per part; it XORs directly out of the borrowed slices into `out`. The Go gap's root cause (a fresh allocation per part) structurally does not exist here.
- `out` accumulator is `Zeroizing::new(vec![0u8; first_len])` (seed_xor.rs:171) → scrubbed on every return incl. the `MismatchedShareLengths`/`BadEntropyLength` error paths (those return before `out` is allocated anyway: seed_xor.rs:167,169).
- `seed_xor_split` returns `Vec<Zeroizing<Vec<u8>>>`; the running `last` accumulator and each `mask` are `Zeroizing` (seed_xor.rs:95,98).
- `seed_xor_split_deterministic` (seed_xor.rs:121): the one raw scratch `buf: Vec<u8>` (the SHA pre-image containing `entropy`) is explicitly `zeroize::Zeroize::zeroize(&mut buf)` after hashing (seed_xor.rs:140); `last` and each `mask` are `Zeroizing`.

  - Caller boundary: the CLI layer (`src/cmd/seed_xor.rs`) is where BIP-39 mnemonic→entropy conversion happens; it is in the `zeroize`-using set (grep-confirmed). The library combine never owns per-part entropy.

**VERDICT: GO-ONLY.** The Rust combine borrows rather than copying per-part entropy, and every owned secret buffer (split accumulator, masks, det-split scratch `buf`) is Zeroizing or explicitly zeroized. No Rust action needed.

---

## Detail — M4 (bip85 leaf privkey not zeroized)

**Go gap:** `deriveBip85Child` obtained `pkey *btcec.PrivateKey` via `k.ECPrivKey()`, serialized it, scrubbed the serialized bytes + the ExtendedKey, but never called `pkey.Zero()`, leaving the raw leaf scalar (+ a value-receiver `Serialize()` copy) resident. Fork fix = `defer pkey.Zero()`.

**Rust:** `crates/mnemonic-toolkit/src/bip85.rs::derive_entropy` (bip85.rs:27-56) — this is the Rust analogue of the BIP-85 child-key step.
- It does NOT materialize a standalone secret-key object analogous to the Go `pkey`. It derives the child `Xpriv` (`master.derive_priv` — bip85.rs:45-47) and feeds the private key **inline** into the HMAC engine: `engine.input(&child.private_key.secret_bytes())` (bip85.rs:49-50). `secret_bytes()` returns a `[u8; 32]` by value into the engine call; there is no named `*PrivateKey` left live for the function's remaining lifetime the way the Go `pkey` was.
- The 64-byte HMAC output — the BIP-85 secret — is `Zeroizing::new(vec![0u8; 64])` (bip85.rs:52) and returned as `Zeroizing<Vec<u8>>`, scrubbed by RAII at the caller.
- `format_bip39_phrase` and the other `format_*` apps consume that `Zeroizing` entropy and additionally `mlock`-pin its pages (bip85.rs:83,109,137,169,187,203,240).

**Residual (NOT the M4 gap):** Rust DOES have a documented, separately-tracked residual: the `Xpriv` child and the `secp256k1::SecretKey`/`bitcoin::PrivateKey` materialized in `format_hd_seed_wif`/`format_xprv_child` are `Copy`/stack-bound third-party types with no Drop+Zeroize. This is explicitly called out in SAFETY comments and tracked by FOLLOWUPs `rust-bitcoin-xpriv-zeroize-upstream`, `rust-secp256k1-secretkey-zeroize-upstream`, `rust-bip39-mnemonic-zeroize-upstream` (bip85.rs:42-44,86-87,110-113,139-143). These are upstream-crate limitations (no `.Zero()` equivalent exists to call), categorically different from the Go M4 gap (which was failing to call an available `.Zero()` on an object the code itself materialized).

**VERDICT: GO-ONLY.** The specific M4 gap — a materialized leaf private-key object with an available zeroize method left uncalled — does not exist in Rust's `derive_entropy`: the private key is consumed inline and the derived secret is `Zeroizing`. The pre-existing `Copy`-type residual is a known upstream-blocked item, already tracked, and is NOT introduced/missed by the M4-equivalent path. No new Rust action needed for M4.

---

## Detail — L1 (codex32 DecodeMS1 probe entropy not scrubbed)

**Go gap:** Three sites used `codex32.DecodeMS1` purely as a validity probe and discarded the returned entropy with `_`; `DecodeMS1` → `Seed()` → `parts().data()` allocates a fresh `[]byte` ([prefix][full seed entropy]) per call, left for the GC, never zeroed. Fork fix = capture + `wipeBytes()` the probe's entropy at all three sites.

**Codec primitive (third-party, pinned):** Rust decodes ms1/codex32 via the published `ms-codec = "0.4.4"` crate (wrapping `codex32 = "=0.1.0"`), not in-repo source.
- `ms-codec 0.4.4` is itself zeroize-disciplined INTERNALLY: `ms-codec-0.4.4/src/decode.rs:35-51` wraps the owned decode-path entropy buffer in `Zeroizing<Vec<u8>>` before constructing the public `Payload`.
- BUT the public `Payload` boundary is, BY DOCUMENTED DESIGN, NOT zeroize-wrapped: `ms-codec-0.4.4/src/payload.rs:17-27` — *"the `Vec<u8>` inside `Payload::Entr` is NOT zeroize-wrapped ... Callers MUST wrap the byte buffer at the use site ... so that the secret-material lifetime ends with a scrubbed drop."* The enum is `Payload::Entr(Vec<u8>)` / `Mnem { entropy: Vec<u8> }` (payload.rs:44,55) with **no `Drop`/`ZeroizeOnDrop`/`Zeroize` impl** (grep over the crate src: none). So a bare `Payload` that drops without the caller having moved its bytes into `Zeroizing` leaves the entropy un-scrubbed — exactly the L1 class.

**Toolkit call sites — how each handles the decoded entropy:**
- `slot_ms1.rs:45-77` — moves bytes into `Zeroizing::new(bytes)` / `Zeroizing::new(entropy)` (slot_ms1.rs:50,74). **Scrubbed.** ✓
- `cmd/silent_payment.rs:142-159` — `Zeroizing::new(b)` / `Zeroizing::new(entropy)` (silent_payment.rs:150-151). **Scrubbed.** ✓
- `wallet_import/overlay.rs:131-148` — `Zeroizing::new(bytes)` / `Zeroizing::new(entropy)` (overlay.rs:133,148). **Scrubbed.** ✓
- `cmd/bundle.rs:2009-2048` (import-json ingest) — `Zeroizing::new(bytes)` / `Zeroizing::new(entropy)` (bundle.rs:2029,2038). **Scrubbed.** ✓
- **`cmd/bundle.rs:2473-2484` (`self_check_bundle` ms1 self-check) — the EXACT L1-analogue probe.** `let (_tag, payload) = ms_codec::decode(ms)?;` then the buffer is used ONLY as a comparison oracle: `if payload.as_bytes() != expected_bytes { ... }` (bundle.rs:2477). `payload` is a bare `ms_codec::Payload` (NOT `Zeroizing`-wrapped — verified there is no `Zeroizing` in lines 2470-2486), so at end of each loop iteration it drops with the master-seed entropy bytes (`Payload::Entr(Vec<u8>)` for the device's always-`entr` cards) un-scrubbed on the heap. This is a decode-as-validity/equality probe whose secret return is discarded un-wiped — the same root cause and the same low severity as Go L1. **GAP.** ✗

**VERDICT: RUST-ALSO-AFFECTED (LOW).** One Rust site — `self_check_bundle` at `cmd/bundle.rs:2473` — decodes an ms1 into a bare, non-`Zeroizing` `ms_codec::Payload` used only as an equality oracle and lets it drop un-scrubbed, mirroring the Go L1 probe and violating `ms-codec`'s own documented caller-wrap contract. Severity LOW (defense-in-depth memory-hygiene; CLI host with no air-gap-strengthening, transient heap buffer; the `expected_bytes` oracle it is compared against is itself a `Zeroizing` slot entropy, so the longer-lived copy is already scrubbed). All other toolkit ms1-decode sites are correct.

**Suggested Rust fix (TDD, fix Rust FIRST per the rule):** at `cmd/bundle.rs:2473`, wrap the probe's bytes in `Zeroizing` before the comparison so the decoded entropy scrubs on drop — e.g. `let scrubbed = Zeroizing::new(payload.as_bytes().to_vec()); if scrubbed[..] != expected_bytes[..] { ... }` (or restructure to move the `Vec` out of `payload` into `Zeroizing`). Add a test asserting the self-check path leaves no live un-`Zeroizing` `Payload` (mirror the codebase's existing zeroize-discipline test pattern, e.g. an `ms-codec`-style `lint_zeroize_discipline` assertion or a use-site review test). This is the only Rust-side action this cross-check surfaces.

---

## Coverage of secondary `mnemonic-key` / `descriptor-mnemonic` survey

Surveyed per the task. Neither hosts a scrub-cluster counterpart:
- `mnemonic-key` (`mk-codec`/`mk-cli`) handles **public** key-card material — xpubs, fingerprints (`as_bytes()` on fingerprints/policy-ids at `mk-cli/src/cmd/mod.rs:75-77`, `mk-codec/.../encode.rs:62`), descriptor template IDs. No BIP-39 secret entropy, no slip39/seed_xor/bip85 path, no ms1-entropy decode-probe. `output_advisory.rs:19,29` references `PrivateKeyMaterial` only as an output-classification advisory label, not a resident secret buffer.
- `descriptor-mnemonic` (`md-codec`) handles **public** descriptor/template bytes (md1 phrase/TLV/origin-path). No secret entropy path.

All four scrub-cluster secret paths live exclusively in `mnemonic-toolkit`.

---

## OVERALL CONCLUSION

| Finding | Verdict | Rust action |
|---|---|---|
| M2 (slip39 group-share scrub on error paths) | **GO-ONLY** | none |
| M3 (seedxor per-part entropy intermediates) | **GO-ONLY** | none |
| M4 (bip85 leaf privkey object not zeroized) | **GO-ONLY** | none (pre-existing `Copy`-type residual is separately tracked, upstream-blocked, NOT the M4 gap) |
| L1 (codex32 DecodeMS1 probe entropy not scrubbed) | **RUST-ALSO-AFFECTED (LOW)** | **YES — fix Rust first** |

**Are any Rust-side zeroize fixes needed?** **Yes — exactly one, LOW severity.**

- **Crate / module / line:** `mnemonic-toolkit` → `crates/mnemonic-toolkit/src/cmd/bundle.rs`, function `self_check_bundle`, the ms1 self-check decode probe at **`bundle.rs:2473` (entropy compared/dropped at 2477-2484)**.
- **Why:** the decoded `ms_codec::Payload` carrying master-seed entropy is used only as an equality oracle and dropped without being moved into `Zeroizing`; `ms_codec::Payload` is explicitly NOT zeroize-wrapped and has no Drop/Zeroize impl, so the bytes leak un-scrubbed — the same defect class as the fixed Go L1, and it violates `ms-codec`'s own documented caller-wrap contract.
- **Severity:** LOW (defense-in-depth; transient; the `expected_bytes` it compares against is already `Zeroizing`). Per the project rule (Go-port defect found+fixed ⇒ check + fix Rust first), this warrants a small TDD fix in `mnemonic-toolkit` before the corresponding Go change is considered closed across the constellation.

M2/M3/M4 require **no** Rust action — the primary Rust implementation already scrubs those secrets correctly via idiomatic `Zeroizing`/`ZeroizeOnDrop` RAII on all paths, including error/early returns.

---

*No source code was modified by this cross-check (report file only).*
