# cycle-prep recon — 2026-06-18 — seedhammer-seedxor

**Fork `main` SHA at recon time:** `bc63caa`.
**Design repo HEAD:** `4affb17`.
**Slug:** `seedhammer-seedxor` — teach the SeedHammer fork **Coldcard Seed XOR**, reusing the
constellation's own `mnemonic_toolkit::seed_xor` (the "port our audited Rust" pattern that
shipped SLIP-39 in Cycle D).

Recon = three parallel agents (constellation port-source; authoritative Coldcard protocol +
vectors; fork integration). Protocol facts verified against **Coldcard's `docs/seed-xor.md`**
(the originator) + the in-repo Coldcard byte-pin test, not memory.

---

## Verdict: **GO — combine/recovery only. Small, high-fit. Split deferred (no on-device RNG).**

Seed XOR combine is the **simplest** of the fork's recovery family — pure XOR of BIP-39
entropies, **N-of-N** (no threshold), reusing the entire codex32/SLIP-39 recovery + engrave
machinery. Net-new code ≈ **~100–150 LoC impl + ~150–250 LoC tests** (a thinned clone of
`codex32_polish.go`). The decisive scoping finding: **the firmware has no usable on-device
CSPRNG**, so SPLIT (which mints N−1 random parts) would require standing up an RP2350 TRNG/ROSC
driver + a "device-mints-secrets" threat model — out of proportion. Combine needs **no RNG, no
sha256d, no field math** — just XOR + the existing BIP-39 checksum recompute.

---

## 1. Protocol (verified vs Coldcard `docs/seed-xor.md` — the originator)

- **XOR the BIP-39 entropy bits; exclude + recompute the checksum per part.** Each part is
  itself a valid BIP-39 mnemonic (its last word carries a freshly-computed checksum: 4 bits for
  12-word, 8 for 24-word). The result's last word needs SHA256 (not paper-computable).
- **Strictly N-of-N** — all parts required; *any* subset (incl. N−1) is itself a valid wallet
  (the decoy/plausible-deniability property — and the reason there's no threshold, unlike
  SLIP-39/codex32). **All parts same length.** **Combine is order-independent** (XOR commutes).
- **Open standard** ("no license required… should be fully interoperable") → porting our impl
  is clean. **12 and 24-word both standardized** (Coldcard interop = 16/24/32-byte; the
  toolkit's 20/28-byte (15/21-word) are non-interop extensions — a port should likely restrict
  to 12/18/24-word for Coldcard compatibility, or clearly flag the extensions).
- **No authentication tag** — a wrong-but-valid part silently yields a different wallet. Same
  silent-wrong-seed UX class we already guard in SLIP-39 (→ reuse the fingerprint
  "check-against-records" gate).
- **TDD-ready vectors captured** (Coldcard 24-word 3-part + 12-word 3-part, arithmetic
  reproduced) + the in-repo `tests/lib_seed_xor.rs` G1 Coldcard byte-pin and round-trip suites.

## 2. Port source — `mnemonic_toolkit::seed_xor` (~200 LoC, one file)

- `crates/mnemonic-toolkit/src/seed_xor.rs`: `seed_xor_combine` (pure XOR + equal-length
  guard), `seed_xor_split` (RNG), `seed_xor_split_deterministic` (Coldcard SHA256d). Operates on
  **raw entropy bytes only**; the BIP-39 phrase↔entropy + per-part checksum recompute is
  CLI-layer (the firmware already has BIP-39). **No `math/big`.**
- **Combine is trivially portable: ~50 LoC of Go** (XOR fold + length validation; `bip39.New`
  recomputes the result checksum). Markedly simpler than the SLIP-39 port (no GF(256)/Shamir/
  Feistel/RS1024).
- Deterministic split (only needed IF split is ever built) is byte-exact vs Coldcard's
  `shared/xor_seed.py`: `sha256d(b"Batshitoshi " + master + "%d of %d parts" % (i, n))[:n]`,
  `i` 0-based, hashing the *master* — the subtle bits the in-repo G1 byte-pin guards.

## 3. Fork integration (vs the shipped recovery flows)

- **Template = codex32 recovery** (`gui/codex32_polish.go`), even simpler — N-of-N is flat:
  no threshold parse, no roster map, no `ConsistentShares`/`selectForCombine`/two-level logic.
- **Combine flow:** ask **"How many parts?"** up front (a `ChoiceScreen` like
  `slip39LengthPick` — parts carry no metadata, so N can't be inferred) → collect N BIP-39
  mnemonics via the existing `inputWordsFlow`/`emptyBIP39Mnemonic` → `Entropy()` each →
  equal-length guard → XOR fold → `bip39.New(result)` → the master-fingerprint
  "check-against-records" gate → `backupWalletFlow`. `bip39.Mnemonic.Entropy()` (`bip39.go:158`)
  / `bip39.New` (`:228`) are the access points (both panic on invalid length — gated by the
  equal-length/validated-input checks).
- **Menu hook = Path A:** add a `"SEED XOR"` choice to the input `ChoiceScreen`
  (`gui/gui.go:2012`, currently `{12 WORDS, 24 WORDS, CODEX32, SLIP-39}`); the combine flow
  returns a plain `bip39.Mnemonic`, which the existing `engraveObjectFlow` `case bip39.Mnemonic:`
  (`gui.go:1847`) already routes to `backupWalletFlow` — **no new dispatch case needed**.
- **Reuse verbatim:** BIP-39 entry, entropy conv, `backupWalletFlow`, the recover-flow
  collection-loop pattern, the **Button2-drain idiom** (must replicate on any new confirm
  screen), `showError`/`wipeBytes`/`masterFingerprintFor`/the fingerprint-confirm gate.
- **DECISIVE — no on-device CSPRNG:** `crypto/rand`/`bip39.RandomWord()` are used **only** in
  host tests/tools (`bip39_test.go`, `cmd/biptool`), never in the firmware UI; `driver/otp` is
  secure-boot key storage, NOT entropy; **no TRNG/ROSC driver exists** (grep-confirmed). So
  combine = zero RNG; split = a whole RNG-driver + minting-secrets-on-an-engraver effort.

## 4. Recommended cycle scope

**Combine-only, S-sized, full gated pipeline:** spec R0→R1 → plan R0 → single-implementer TDD
(port `seed_xor_combine` to a tiny `bip39`-adjacent helper + `gui/seedxor_*.go` combine flow +
the "how many parts" picker + menu wiring) with the Coldcard vectors + the toolkit G1 byte-pin
as the cross-check oracle → whole-diff execution review → merge. Reuse the SLIP-39/codex32
recovery idioms; **carry the silent-wrong-part fingerprint gate** (no auth tag). Decide
12/18/24-word (Coldcard-interop) vs also the 15/21-word toolkit extensions.

**Split — DEFER** behind a separate cycle that first stands up an RP2350 TRNG/ROSC CSPRNG driver
(no precedent in `driver/`) and a "device mints secret material" threat-model review. Likely not
worth it for an engraver (whose job is to *reproduce* a seed, not generate one); revisit only if
there's a concrete need to split on-device rather than via the toolkit CLI.

**Priority:** MEDIUM — genuinely useful (completes the recovery suite: BIP-39 / codex32 /
SLIP-39 / Seed XOR all combinable on-device), cheap, Coldcard-ecosystem-compatible, and a clean
reuse of our own audited code. Good next cycle if the user wants one; not urgent.
