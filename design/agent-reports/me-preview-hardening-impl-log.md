# Implementation log — me preview hardening (Cycle A: F8 + F9 + F10)

Single-implementer TDD execution of `design/SPEC_me_preview_hardening.md` (GREEN at R0
round 1, 0C/0I). Worktree `/scratch/code/shibboleth/me-cycleA`, branch
`me-preview-hardening`, base master `5d68002` (spec commit) / `9fafb6b` (code base).

Order per spec §"Ordering & verification": **A3 first** (mechanical perms/truncate),
then **A1** (dirty-dir refuse), then **A2** (sidecar-output validation, incl. the
mandatory I1 fake/test migration).

Baseline (before any change): Rust `ME_REQUIRE_GO=1 cargo test --locked` = 82 passed
(lib 54 + main 0 + cli 23 + cross_lang 1 + golden 3 + preview_cross_lang 1), 0 skips;
Go `go test ./...` in `preview/` = ok.

Scratch target dir (outside the worktree): `/scratch/code/shibboleth/me-cycleA-scratch/target`.

---

## Step A3 (F10) — owner-only permissions on written artifacts + truncate

**Tests written first (RED):**
- Go `preview/writeout_test.go::TestWriteOutPermIsOwnerOnly` — asserts `writeOut` on a
  real path yields `mode & 0o077 == 0`. Failure line:
  `writeout_test.go:24: writeOut created a group/other-accessible file: mode 0644, want no bits in 0o077`.
- Rust `cli.rs::perms::ndef_out_file_is_owner_only` — failure line:
  `assertion left == right failed: NDEF --out must be owner-only, got 100644`.
- Rust `cli.rs::perms::manifest_file_is_owner_only` — failure line:
  `assertion left == right failed: manifest must be owner-only, got 100644`.
- Rust `cli.rs::perms::manifest_overwrite_shrink_no_trailing_bytes` (I2 regression guard)
  — GREEN today (`fs::write` truncates); it is a guard for the new `OpenOptions` path.
  Teeth proven: with `.truncate(true)` removed from `write_private`, it FAILS at the
  byte-identity assertion (`cli.rs:689`, trailing stale bytes); restored → PASS.

**Change:**
- `preview/main.go` `writeOut`: `os.WriteFile(path, payload, 0o644)` → `0o600`.
- `crates/me-cli/src/main.rs`: new `write_private(path, bytes)` using
  `OpenOptions::new().write(true).create(true).truncate(true)` + `#[cfg(unix)] .mode(0o600)`;
  replaces `std::fs::write` at the NDEF `--out` site and the manifest site. Doc comment
  records the create-only mode residual (R0 L3) and the load-bearing truncate (R0 I2).

**Final counts after A3:** Go `go test ./...` ok. Rust `ME_REQUIRE_GO=1 cargo test --locked`
= 85 passed (lib 54 + main 0 + cli 26 + cross_lang 1 + golden 3 + preview_cross_lang 1),
0 skips. `cargo clippy --all-targets -- -D warnings` clean.

---
