**⚠ Proposed Important: #1 (data loss), #3 (malformed blob), #4 (silent record loss), L5 (flash overwrite). Proposals only; an Opus pass decides.**

# Bug-hunt triage (me), branch bughunt/triage, commit 602ddba1 (tests only, not pushed)

## 1. Gate
`cargo +1.85.0 nextest run --locked --workspace -E 'test(/bughunt_me_/)' --no-fail-fast` -> `/scratch/cache/bughunt/me/gate.log`: 7 run, 2 passed, 5 failed, 693 skipped.
- Failing set: {01, 02, 03, 04, 05}. CONFIRMED set: {#1, #2, #3, #4, L5} = {01, 02, 03, 04, 05}. Equal.
- Passing: 06 (L6), 07 (L7), both FALSE POSITIVE.
- fmt --check and clippy -D warnings (1.85.0) clean before the commit. Fix patches: `/scratch/cache/bughunt/me/fix-01..05.patch` (reverted; `git status` showed only the test file after each).

## 2. Table
All tests are in `crates/me-cli/tests/bughunt.rs`. Plants/fixes were each run and reverted.

| # | verdict | test | unmodified-run evidence | fix / plant evidence | CLI reach | grade | why |
|---|---|---|---|---|---|---|---|
| #1 | CONFIRMED | `bughunt_me_01_write_private_keeps_old_file_when_chmod_fails` | `assertion left == right failed: the write failed (chmod EPERM) yet the old file was destroyed  left: []  right: [79, 76, 68, ...]` | fix-01: drop `truncate(true)` in `mnemonic-io-lib/src/write.rs` `write_private`, `set_len(0)` after the chmod: 1 passed | Yes. In a rootless userns, file `old.bin` owned by another uid, mode 666, caller uid 2: `me sysw pack --no-passphrase --out old.bin text:6869` -> `me: old.bin: Operation not permitted (os error 1)`, rc=2, `wc -c old.bin` = 0 (was 14 bytes) | I | Data loss (old file emptied while the command reports failure). Needs a not-owned but writable destination, so rare, but data loss stays I. Fails loudly (rc 2). |
| #2 | CONFIRMED (library) | `bughunt_me_02_stale_preview_is_not_this_runs_render` | `a sidecar that wrote nothing must be refused, got Ok("/tmp/.../plate-1.svg")` | fix-02: remove the stale `plate-N.ext` before spawn in `preview.rs` `render_plate`: 1 passed | Library API only (`preview::render_plate`). CLI: `ME_PREVIEW_BIN=<fake> me bundle --preview <dir>` with a stale `plate-1.svg` present is refused first: `me: preview directory ... already contains plate artifacts (e.g. plate-1.svg); use an empty/clean directory`, rc=2. The stale file can only appear mid-run (race or sidecar that skips a write). | M (proposal) | Fails closed on every CLI path tried; needs a dirty file to appear between the dir check and the render plus a sidecar that exits 0 without writing. The CLI guard already covers the main case. Grader may disagree: a check that silently does not run would be I if the CLI guard can be bypassed. |
| #3 | CONFIRMED (library) | `bughunt_me_03_pack_passphrase_without_secrets_is_well_formed` | `blob has bytes past the header's declared end (len 77, declared 61)  left: 77 right: 61` | fix-03: in `sysw/mod.rs` `pack_deterministic_with`, drop the passphrase when there are no secret records: 1 passed | Library API only: `mnemonic_engrave::sysw::pack(["text:6869"], Some("abandon ability able about above"), MIN_ITERATIONS)`. The CLI never passes a passphrase with no secret records. | I | Wrong data: a blob whose length disagrees with its header. Only the library (and any other caller) reaches it, so the holder-facing risk is low; grade I because the output is malformed, not a refusal. |
| #4 | CONFIRMED | `bughunt_me_04_pack_in_file_plus_argv_does_not_drop_argv` | `exit 0 but the argv record was silently dropped: "text:6162"` | fix-04: `read_records` in `main.rs` refuses argv records plus `--in` (exit 2): 1 passed | Yes. `echo text:6162 > recs.txt; me sysw pack --no-passphrase --in recs.txt --out p4.bin text:6869` -> rc=0; `me sysw show p4.bin` lists one public record (`public record 0: free text`); `text:6869` is gone | I | A silently ignored input with exit 0: data loss. The argv-visibility part is not graded (secret-handling ruling). Whether to refuse or merge is open (section 3). |
| L5 | CONFIRMED (library) | `bughunt_me_05_uf2_oversize_blob_never_addresses_past_flash_end` | `block 8192 targets 0x11000000, past the end of flash` | fix-05: `assert!(blob.len() <= 0x20_0000)` in `seal/uf2.rs` `to_uf2`: 1 passed | Library API only: `mnemonic_engrave::seal::uf2::to_uf2`. Not run through the CLI: reaching a blob over 2 MiB needs oversize records, which I did not try. | I (proposal) | A UF2 that addresses past the flash end destroys the firmware per the module doc; the refusal should be a Result, not a panic (open question). Unknown whether any CLI input can reach it; if none can, M. |
| L6 | FALSE POSITIVE | `bughunt_me_06_open_refuses_absurd_iterations_quickly` | passes (open refuses u32::MAX iterations at once) | Plant: `sysw/wire.rs:174` made `if false && !(MIN_ITERATIONS..=MAX_ITERATIONS).contains(&h.iterations)`: FAIL after 10.017s, `open ran the KDF for u32::MAX iterations instead of refusing`. The real bound is at `sysw/wire.rs:174-175`; the report's `seal/crypto.rs:35` is `derive_key`, called only after `open` has parsed the header with that bound. A first plant in `seal/wire.rs:136` did not fail the test, because `open` uses `sysw/wire.rs`. | n/a | none | The lead named the wrong site. |
| L7 | FALSE POSITIVE | `bughunt_me_07_pack_mt_set_missing_strings_never_panics` | passes (no subset of the six-string mt set panics; exit != 101) | Plant: `sysw/mt.rs:159-160` made `missing` always empty and the guard `present.len() < count` so `Missing{missing: []}` is returned: FAIL, `attempt to subtract with overflow`, exit Some(101). So the underflow at `main.rs` `describe_set_problem` (`names.len() - 1`) is real only if `missing` is empty, and `diagnose` returns `Missing` only when `!missing.is_empty()` (`sysw/mt.rs:160`). | n/a | none (latent N: unreachable) | Guarded by the caller. |

## 3. Open product questions
- #4: refuse argv plus `--in` (fix-04) or merge both sources? The test accepts either, so long as no record is dropped.
- L5: what is the cap, and should `to_uf2` return a Result rather than panic? (Test only requires no block above 0x11000000.)
- #2: should the dirty-directory check be re-run per plate, or is the entry check enough?
- Shared-verb ruling (2026-11-01): none of these verbs (`sysw pack`, `bundle --preview`) is repair/inspect/verify/address; the ruling does not apply.

## 4. Noticed, not tested
- The `me sysw pack` stderr says "NOT SEALED ... cleartext" and `sysw show` prints "no passphrase - BELOW the threshold" for a plain `--no-passphrase` pack; wording only, not tested.
- Tests 01/03/04 write temp files under the system temp dir (`/tmp`), as the tempfile default; harmless.

Counts: 7 items; 5 CONFIRMED (#1, #2, #3, #4, L5), 2 FALSE POSITIVE (L6, L7), 0 UNRESOLVED; proposed grades I x4 (#1, #3, #4, L5), M x1 (#2), none x2.
