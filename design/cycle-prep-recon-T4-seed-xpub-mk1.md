# cycle-prep recon — 2026-06-19 — T4 seed → account xpub → engrave as mk1

**Fork HEAD:** `a4d669d` (T3 merged). **Recon agents (parallel, source-verified):** `a740b4ce0f59be914` (mk1-encoder + xpub-derivation + policy-stub, verified vs `mnemonic-key/crates/mk-codec`), `afdcf148e49695e42` (seed-entry + GUI surface + security spine).

T4 goal: hand-typed BIP-39 seed → derive an account xpub at a chosen standard path → engrave it as an mk1 key card (watch-only steel backup). SECURITY SPINE: seed is SECRET (typed-only, never NFC), deterministic derivation, emits ONLY the public xpub; no CSPRNG.

## USER DECISION (2026-06-19) — the gating scope call
The roadmap's "engrave as mk1" is semantically blocked for a *bare* xpub: mk1's `KeyCard` requires ≥1 `policy_id_stub` (rejected if `stub_count==0` at both encode `encode.rs:24` and decode `decode.rs:26`; fork `errStubCount`), and a stub = top-4-bytes of a **wallet policy's** WalletPolicyId (`key_card.rs:25-30`, `mk-cli` `derive_stub_from_md1`) — a seed-derived xpub has no policy to hash. **User's resolution: go the mk1 route anyway; when the policy id cannot be derived (always, in T4's no-policy scenario), WARN on screen and set the stub ID to 0.** → T4 emits a valid mk1 card with `stub_count=1`, `policy_id_stubs=[[0x00,0x00,0x00,0x00]]`, plus a prominent on-screen warning that the card is NOT bound to a wallet policy. (This is structurally valid and decodes cleanly via the shipped `mk.Decode`; it is a deliberate product choice overriding the recon's "don't mint a sentinel stub" caution — documented as such.)

## Verified facts (cite source)
### Encoder (the central build) — NO mk1 encoder in the fork; buildable, most machinery in-tree
- `mk/mk.go` is decode-only (`ParseHeader`/`Decode`/`reassemble`/`decodeBytecode`/`decodePath`/`reconstructXpub`); no `Encode`/checksum-gen/chunk-split anywhere in `mk/` or `codex32/`. T4 must add an mk1 ENCODER = the reverse of T2b:
  - **bytecode encode** (`encode.rs:55-67`): `header(1)|stub_count(1)|stubs(4×N)|[fp(4) iff bit2]|path(var)|xpub_compact(73)`; header byte version bits7-4=0, fp flag bit2 (valid 0x00/0x04). Reverse of `decodeBytecode`.
  - **compact-73 from xpub** (`xpub_compact.rs:8-15`): `version(4)|parentFP(4)|chaincode(32)|pubkey(33)`. Encoder invariant (`encode.rs:38-48`): xpub depth == path component count AND child_number == terminal path component (a CKD-derived account xpub at e.g. m/84'/0'/0' satisfies this).
  - **path encode** (`path.rs:38-98`): 1-byte std-table indicator (the 14 paths below) else `0xFE+count+LEB128` (cap 10).
  - **chunk split** (`chunk.rs:50-93`, `pipeline.rs:72-105`): `SINGLE_STRING_LONG_BYTES=56`; a 1-stub card is ~84B (80B privacy-preserving) → **ALWAYS chunked → 2 chunks** (53+rest). Stream = `bytecode || SHA-256(bytecode)[0..4]`, split into `CHUNKED_FRAGMENT_LONG_BYTES=53`-byte fragments, each wrapped in the 8-symbol chunked header `{version,type,chunk_set_id(20b),total_chunks-1,chunk_index}`.
  - **BCH gen**: the fork ALREADY has the generators + mk1 targets (`codex32/checksum.go` engine + `codex32/mdmk.go` mk targets/POLYMOD_INIT); **`codex32.NewSeed` (codex32.go:279-383) is a complete working encoder template** (5-bit pack + checksum gen + string assembly, used for ms1) → reuse the pattern, no polynomials re-derived.
- **chunk_set_id / no-CSPRNG:** mk-codec's `fresh_chunk_set_id` uses OsRng, but `encode_with_chunk_set_id(card, csid)` takes a deterministic 20-bit override and the decoder only checks csid CONSISTENCY across chunks (not its value). **T4 pins a DETERMINISTIC csid** (e.g. `SHA-256(bytecode)[0..]` → top 20 bits, or constant 0) — no randomness, no spec change. NOT a blocker.
- **Encoder LOC ≈ 500-750 net-new Go** (bytecode/path/compact-73 encode + chunk split; 5-bit/BCH/string machinery reused).

### Derivation — all primitives exist, fully deterministic; compose them
- `bip39.MnemonicSeed(m, password)` PBKDF2-SHA512 (`bip39.go:217-226`) → `hdkeychain.NewMaster` (`deriveMasterKey`, `gui.go:190-199`) → **`bip32.Derive(mk, path)` walks the path + `.Neuter()` → account xpub** (`bip32/bip32.go:43-53`). `bip32.Fingerprint` for the master FP. No end-to-end helper today (`bip32.Derive` is never called from the GUI — only tests; `fillDescriptor` in `gui_test.go:292-327` is the compose template). Determinism audit: every step deterministic; the only `crypto/rand` in scope is `bip39.RandomWord` (mnemonic GENERATION, off the typed path) — not used.

### Standard-path set (14, verified `path.rs:38-55`)
mainnet 0x01-0x07 / testnet 0x11-0x17: 44'/49'/84'/86'/0'/0' (depth-3 single-sig), 48'/0'/0'/2' & 48'/0'/0'/1' (depth-4 BIP-48 multisig), 87'/0'/0' (BIP-87). **Picker UX concern:** `ChoiceScreen` renders all choices unpaginated/unclipped (`gui.go:1411-1474`); the largest existing list is 5 (`newInputFlow`). 14 entries will overflow the small display → T4 needs **paging (copy `mk1DisplayFlow`'s measure-and-advance) OR a two-stage picker** (script-type → network), OR a curated subset.

### Seed entry + engrave reuse
- Seed entry: `newInputFlow`→`inputWordsFlow` (`gui.go:2068`/`582`) — typed-only word keyboard w/ BIP-39 completion + last-word checksum (`LastWordCandidates`); produces the SECRET `bip39.Mnemonic`. Reachable via the `backupWallet` program. T4 reuses it.
- Engrave: `validateMdmk`/`mdmkFlow` (`gui.go:1891`/`1933`) engrave ONE mk1/md1 string as a single plate (TEXT+QR/TEXT/QR variants of the SAME string). **CRITICAL GAP: multi-chunk ENGRAVE sequencing does NOT exist** — a 2-chunk mk1 needs TWO plates ("plate 1 of 2", "plate 2 of 2"); no loop/sequencing exists today (multi-chunk is read-side only via `mk1GatherFlow`). **T4 must build multi-plate engrave sequencing** (net-new UI).
- Hook: add a NEW top-level `program` (parallel to `backupWallet`) — keep the public-xpub flow distinct from `backupWalletFlow` (whose contract is "engrave the SEED"); do NOT call `engraveSeed`.

### Security spine — holds; one required addition
- Device has NO NFC writer (`Platform` exposes only `NFCReader()`); seed has no emission path. Typed-only entry ✓. `.Neuter()` guarantees no xprv serialized ✓. Output = public xpub-as-mk1 only ✓. **ADD: scrub the 64-byte `MnemonicSeed` buffer after `NewMaster` consumes it** (`wipeBytes` precedent, `slip39_polish.go:330`); `deriveMasterKey` currently doesn't scrub — T4's derive helper should. Master `*hdkeychain.ExtendedKey` private material has no easy scrub hook (defense-in-depth, document).

### Test harness / alloc / placement (unchanged from prior cycles)
`runUI`/`ExtractText`/`uiContains`, `click`/`press`/`runes`, `NFCReader()==nil`, alloc gate = `StartScreen.Flow`+`DescriptorScreen.Confirm` only (new T4 screens not auto-gated). The encoder + derivation are deterministic → golden-vector testable headless (round-trip: `mk.Decode(encode(card)) == card`, plus the `mk/mk_test.go` golden vectors). Go `/home/bcg/.local/go/bin/go`.

## Effort + phasing (one cycle, two phases under one spec)
- **Phase A — headless core:** the mk1 ENCODER (in a new `mkencode` file in the `mk` package, or `mk.Encode`) + a deterministic chunk_set_id + the account-xpub derivation helper (+ seed scrub). Parity-tested by round-trip through the shipped `mk.Decode` + the `mk_test.go` golden vectors. ~600-850 LOC.
- **Phase B — GUI:** new `program` + seed entry (reuse) + standard-path picker (paged/two-stage) + the **stub-0 warning** screen + derive + mk1 encode + **multi-plate engrave sequencing** + (optional) a pre-engrave xpub-confirm display. ~400-600 LOC.

## Biggest risks (for the spec to lock)
1. **Multi-plate engrave sequencing** — net-new; a 2-chunk mk1 = 2 plates; today only single-plate exists. The spec must define the "plate N of M" engrave loop.
2. **The 0-stub warning** — must be prominent + operator-acknowledged before engraving (the card is unbound; user-decided behavior).
3. **Path-picker overflow** at 14 entries — paging or two-stage.
4. **Seed scrub** — scrub the PBKDF2 seed buffer post-derive (the new addition).
5. Encoder fidelity — round-trip through `mk.Decode` is the gate; verify the bytecode/chunk/BCH against mk-codec source (the encode invariant `depth==pathlen && child==terminal`).

## Gate reminder
`SPEC_seedhammer_T4_seed_xpub_mk1.md` MUST pass opus R0 to 0C/0I before code; fold → persist verbatim → re-dispatch until GREEN. Fork-side only; no upstream PR.
