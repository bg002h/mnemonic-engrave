# Bug report: bg002h/mnemonic-engrave @ a757199

## Summary
- **Scope:** 52 Rust source files with about 21,900 lines: all of `me-cli` and `mnemonic-io-lib`, plus the small `rsprobe` script. I left out `design/measurements/*` and `harnesses/*`, which are measurement probes, not product code.
- **Agents:** 18 slices, each read by a reader and a second reviewer. All finished.
- **Result:** 4 suspicions were rated 3 or higher. **3 were demonstrated** with failing tests and 1 was rejected.
- **How they were tested:** the crate doesn't build offline because of a missing git dependency, so each test is a standalone `rustc` copy of the function in question. Tests are under `/tmp/hunt-me/prover-N/` in the sandbox.

Reviewers checked these and found them correct: the descriptor checksum (BIP-380), base58, the secp256k1 field arithmetic, the seal wire bounds, and the varint and transaction parsing in `sysw`.

**Coverage gap:** `main.rs` (3,713 lines) was **not fully read**. The reader read about 400 lines closely and the reviewer about 680; the rest was only searched for risky patterns. It is the main unread area in this repo.

## Confirmed bugs (worst first)

### Medium

**1. `write_private` wipes the existing file before a chmod that can fail** (demonstrated)
- Where: `crates/mnemonic-io-lib/src/write.rs` `write_private`, line 55. The file is opened with `truncate(true)` before `set_permissions(0o600)`.
- Input: `--out` pointing at an existing file the caller can write to but doesn't own, e.g. mode 0666 and owned by root.
- Should: on error, leave the old contents alone.
- Does: chmod fails with EPERM and the function returns Err, but the file is already empty. The old artifact is destroyed and nothing replaces it.
- Test: `prover-4/t.rs`, run as uid 65534.
- Fix: open with `truncate(false)`, and call `set_len(0)` only after the chmod succeeds.

**2. A preview from a previous run is accepted as this run's render** (demonstrated)
- Where: `crates/me-cli/src/preview.rs` `render_plate`, line 192. The output path is never removed before the sidecar starts.
- Input: `me bundle --preview` into a directory that already holds `plate-1.svg`, with a sidecar that exits 0 and writes nothing.
- Should: refuse with `EmptyOutput`. The F9 check exists so that a sidecar that writes nothing can't produce a "valid" preview.
- Does: accepts the old file, and the manifest records it as this run's preview of a different string.
- Test: `prover-1/demo.rs`.
- Fix: remove `out_path` before spawning the sidecar, ignoring NotFound.

### Low

**3. `sysw pack` with a passphrase but no secret records produces a malformed container** (demonstrated)
- Where: `crates/me-cli/src/sysw/mod.rs` `seal_with` / `pack_deterministic_with`, line 639.
- Input: `pack(["text:6869"], Some(PASS), MIN_ITERATIONS)`.
- Should: a plain unsealed container of 61 bytes.
- Does: a 77-byte blob whose header says unsealed and 61 bytes long, but which still has iterations, salt and IV filled in and a 16-byte tag after the declared end. `open` with a passphrase then accepts it without checking the passphrase.
- Note: the CLI avoids this by dropping the passphrase when there are no secrets (`main.rs:2594`), so only the library API is affected.
- Test: `prover-3/t.rs`.
- Fix: use the unsealed path when there are no secret records, or refuse.

## Rejected
- **Interior spaces accepted by `encode_section`** (`seal/container.rs:67`). Rejected because this is a low-level helper: every caller checks records first (`validate_record`, `non_canonical_separator`), and secret sections must allow spaces for BIP-39 mnemonics.

## Not checked
About 50 suspicions were rated 2 or lower and were not sent to provers. The notable ones:
- **Malformed blobs:** `seal/uf2.rs:30` has no size limit, so a blob over 2 MiB would address past `0x11000000`, which the module doc says destroys the firmware. The KDF `iterations` value in `seal/crypto.rs:35` comes from an untrusted header and has no bounds.
- **File writing:** `write.rs` follows symlinks, so a pre-planted symlink makes it write somewhere else.
- **Hashing and counting:** `seal/pubhash.rs` and `sysw/pubhash.rs` store the record count as a u8, which wraps at 256, and join records with newlines, which is ambiguous.
- **Parsing:** path numbers are parsed as i64, which accepts `+` and `-0`. NDEF decoding ignores the SR, MB, ME and IL flags. Records are trimmed for checking but passed on untrimmed.
- **Wrong refusals and labels:** `refusal.rs:1105` turns any 48h script type into P2WSH. `manifest.rs:145` assumes an ms1 plate is always present. `main.rs:2193` underflows when the list of missing plates is empty.
- **History remedy:** the shell commands in `remedy.rs` don't escape regex characters and can break the user's zsh history settings.

## What was read
Every non-test line was read by both a reader and a reviewer, **except `main.rs`**, as noted above.

---

# Coverage follow-up (second reads): `main.rs`

**Scope:** `crates/me-cli/src/main.rs` lines 1-3,552 (all non-test code), split into 3 ranges. Each range was meant to get a reader and a reviewer. The readers of the first two ranges failed (I'm not told why), so those ranges got one full read, by the reviewer. The third range got both. **`main.rs` has now been read in full.**

**Result:** 2 suspicions were rated 3 or higher. 1 was demonstrated with a failing test and 1 was rejected.

## Medium

**4. `me sysw pack --in FILE` silently drops records given on the command line, and they skip the argv-secret check** (demonstrated)
- Where: `crates/me-cli/src/main.rs` `read_records`, line 3044. The `--in` branch returns before argv is looked at, and `pack` has no `conflicts_with` between the two. `seal` does refuse this combination (main.rs:936).
- Input: `me sysw pack --in recs.txt --out p.bin tx:deadbeef`, or the same with a seed on argv.
- Should: refuse with "pass records on argv OR via --in, not both".
- Does: packs only the file's records and exits 0. The argv record is lost, and a secret or `tx:` record on argv is never refused or warned about, even though it is already visible in the process list and shell history.
- Test: `/tmp/hunt-fu/prover-23/t.rs`.

## Rejected
- **Seed check runs on untrimmed records** (`run_seal_cli`, line 980). Rejected because `classify` and `passphrase::is_valid` ignore exactly the same whitespace that `encode_section` trims, so padding can't get a seed past the `--seal-secret` opt-in.

**Not checked:** suspicions rated 2 or lower:
- `main.rs:2983` labels every hash record "sha256 hashlock", even for hash256, ripemd160 or hash160.
- `main.rs:775`: `--echo` labels an mt1 string "validated md1".
- `main.rs:3203`: the `-` splice rebuilds the document from the split records, so blank lines and layout are lost.
- `main.rs:649`: the refusal text says `--allow-argv-secret` only applies to `sysw pack`, but it also works on `seal`.
