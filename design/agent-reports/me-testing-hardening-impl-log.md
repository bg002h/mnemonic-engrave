# me funds-safety + testing-hardening — implementation log

Single implementer, worktree `me-testing-hardening` off `master` (base `fdc11aa`).
Executes `design/IMPLEMENTATION_PLAN_me_testing_hardening.md` (GREEN R0). Strict TDD.

Toolchain: cargo 1.97.0-nightly, rustc 1.97.0-nightly; go 1.26.4 at
`/home/bcg/.local/go/bin`. Every go-touching command is prefixed with
`env PATH="/home/bcg/.local/go/bin:$PATH"`; cargo runs with `ME_REQUIRE_GO=1` once Step 2
lands.

---

## Step 0 — worktree + baseline (done)

- Worktree created off `master` (fdc11aa); submodule `third_party/seedhammer` initialized
  at `713aee2e5b5669d7cc02be8c6d09c05cf3727ccf`.
- Toolchains confirmed present in the worktree environment (go 1.26.4, cargo/rustc).
- Baseline `cargo test --locked` (go on PATH): **62 passed, 0 failed, 0 skipped visibly**
  - lib unittests: 41 passed
  - tests/cli.rs: 18 passed
  - tests/cross_lang.rs: 1 passed
  - tests/golden.rs: 1 passed
  - tests/preview_cross_lang.rs: 1 passed
  - doc-tests: 0
- `go test ./...` in `preview/`: ok (`mnemonic-engrave/preview`).
- `go build ./...` in `firmware/ndef-roundtrip/`: BUILD_OK.
- **F7 hazard confirmed:** `firmware/ndef-roundtrip/go.mod` `replace` targets
  `../../../seedhammer-ref-v1.4.2`. That path (`/scratch/code/shibboleth/seedhammer-ref-v1.4.2`)
  happens to exist on this dev host, which is why `cross_lang.rs` and the firmware build pass
  today — but it would NOT resolve in a clean clone (Step 1 fixes this by repointing to the
  pinned submodule).
- Current codec pins: `md-codec 0.36.0`, `mk-codec 0.4.0` (Cargo.lock).
  Available upstream: `md-codec 0.40.0`, `mk-codec 0.4.1` (Step 4 bumps).

Baseline recorded. Proceeding to Step 1.

---

## Step 1 (A4) — hermetic Go oracle (done)

- **Fail-first:** copied the worktree tree (firmware/, third_party/, preview/) into a
  scratch clone at `/scratch/code/shibboleth/me-impl-scratch/hermetic-clone/`, whose parent
  lacks `seedhammer-ref-v1.4.2`. `go build ./...` in `firmware/ndef-roundtrip` failed for the
  RIGHT reason:
  `main.go:10:2: seedhammer.com@v0.0.0: replacement directory ../../../seedhammer-ref-v1.4.2 does not exist`
- **Fix:** `firmware/ndef-roundtrip/go.mod` replace →
  `../../third_party/seedhammer` (mirrors `preview/go.mod`). `go mod tidy` produced no
  go.sum (only dep is the local seedhammer.com replace, pseudo-version v0.0.0, no checksums —
  same as before the change; matches the pre-existing absence of go.sum in firmware/).
- **Verify hermetic:** with the fixed go.mod, `go build ./...` in the scratch clone (ref path
  still absent, submodule present at `third_party/`) → HERMETIC_BUILD_OK; the round-trip
  decode of the md1-short NDEF via `go run .` returned `md1yqpqqxqq8xtwhw4xwn4qh` (submodule
  reader ≡ old ref reader for this input).
- Full `cargo test --locked` with go on PATH: 62 passed, `cross_lang.rs::rust_ndef_parses_in_seedhammer_go_reader`
  now resolves through the submodule and passes.

Touches: `firmware/ndef-roundtrip/go.mod`.

---

## Step 2 (A5 + F3) — CI test gating (done)

- **Fail-first (F3 vacuous pass):** with `go` absent from PATH and `ME_REQUIRE_GO=1`,
  `cargo test --test cross_lang` currently PASSED (`rust_ndef_parses_in_seedhammer_go_reader ... ok`)
  — the differential oracle silently no-oped. That is the bug.
- **Fix (test code):** added a `go_required()` helper (`ME_REQUIRE_GO == "1"`) to BOTH skip
  sites — `tests/cross_lang.rs` and `tests/preview_cross_lang.rs`. When the toolchain is
  missing, `assert!(!go_required(), …)` now hard-fails instead of returning.
  - (a) go absent + `ME_REQUIRE_GO=1` → both tests FAIL (panic, exit 101):
    `ME_REQUIRE_GO=1 but \`go\` is not on PATH: …`.
  - (b) go absent + var unset → skip note + pass (unchanged local behavior).
  - (c) go present + `ME_REQUIRE_GO=1` → run for real + pass.
- **Fix (CI):** added a `test` job to `.github/workflows/release.yml`:
  `actions/checkout@v4` `submodules: true`, Rust (`dtolnay/rust-toolchain@master`) + Go
  (`actions/setup-go@v5`) in the SAME job, `env: ME_REQUIRE_GO: '1'`; steps
  `cargo test --locked`, `go test ./...` in `preview/`, `go build ./... && go test ./...` in
  `firmware/ndef-roundtrip/`. Extended `on.push.branches: [master]` (R0 L2 trigger
  reconciliation). Wired `assemble.needs: [test, go-build, rust-build]` so a red suite blocks
  tag publish (F2).
- Validated with **actionlint** (installed): "no findings".
- **Branch-protection carry-forward (repo setting, NOT YAML — USER ACTION):** blocking a red
  PR from *merging* needs a GitHub branch-protection rule marking `test` as a required status
  check. The implementer cannot set this; must be flagged at handoff / in the PR description.
  Without it, F2's "red suite can merge" is only half-closed (tag publish IS gated by `needs:`
  regardless).
- Full `cargo test --locked` with go on PATH + `ME_REQUIRE_GO=1`: 62 passed, exit 0.

Touches: `crates/me-cli/tests/cross_lang.rs`, `crates/me-cli/tests/preview_cross_lang.rs`,
`.github/workflows/release.yml`.

---

## Step 3 (A1 + B8) — redact bundle error paths (done)

- **External-fact verification (per CLAUDE.md):** confirmed against the extracted codec
  sources (both current 0.36/0.4.0 AND bumped 0.40.0/0.4.1) that `md_codec::Error` and
  `mk_codec::Error` Display are metadata-only. Scanned every `#[error("…")]`: they interpolate
  at most numeric indices/counts, a hex id, an HRP prefix, or a SINGLE offending char
  (`character {c:?} not in codex32 alphabet`, `invalid character {ch} at position {position}`).
  The wrapped `{0}` String variants (`Codex32DecodeError`, `BchUncorrectable`,
  `ChunkedHeaderMalformed`) are constructed from fixed descriptive text
  (e.g. "BCH checksum verification failed", "total_chunks = 0"), NEVER the raw input. So
  showing only the inner `e` (as ConvertError already does, and the audit accepted as
  hardened) is safe.
- **Fail-first (3 tests, all failed for the right reason — real leaks reproduced):**
  - `bundle.rs` unit `no_bundle_error_display_leaks_the_input_body` (B8): Display of
    `Classify`/`Validate`/`Md1HeaderRead` with a `CANARY_SECRET_BODY` input leaked it, e.g.
    `cannot classify 'CANARY_SECRET_BODY': unrecognized HRP 'zz' …`.
  - `cli.rs` `bundle_msx1_mangled_hrp_does_not_leak_secret_body`: `me bundle` on
    `msx10entrs…cj9sxraq34v7f` printed
    `me: cannot classify 'msx10entrs…cj9sxraq34v7f': …` — the intact codex32 secret body.
  - `cli.rs` `bundle_corrupted_mk1_does_not_leak_full_string`: a 1-flip mk1 printed
    `me: invalid string 'mk1qpz…q': mk1 string is not pristine …` — the full string.
- **Fix:** `bundle.rs` Display arms `Classify`/`Validate`/`Md1HeaderRead` now drop the `{s}`
  input interpolation and show only the underlying `e` (bounded metadata), mirroring
  ConvertError. `Mk1SingleString`/`Md1WireVersion` already redacted (`_`); `SetIncomplete*`
  carry a bounded `fmt_chunk_set_id` (not raw input) and are left as-is.
- All 3 tests now pass. Full `cargo test` with go + `ME_REQUIRE_GO=1`: lib 42 (+1 B8),
  cli 20 (+2 leak), cross_lang 1, golden 1, preview_cross_lang 1 → all green, exit 0.

Touches: `crates/me-cli/src/bundle.rs`, `crates/me-cli/tests/cli.rs`.

---

## Step 4 (A2) — codec bumps, fail-closed admission (done)

- **Over-length fixture (captured BEFORE the bump):** scratch crate
  `/scratch/code/shibboleth/me-impl-scratch/fixture-gen` pinned to `md-codec = "=0.36.0"`,
  ran `md_codec::codex32::wrap_payload(&vec![0xA5u8; 51], 405)` (405 bits = 81 data symbols;
  +13 checksum = 94 codex32 symbols, one past the 93-symbol regular-code cap). Result
  (deterministic): `md15kj6tfd9…zfqq6yyhmu3j8` (97 chars, 94 symbols after `md1`). Generator
  confirmed 0.36 `unwrap_string` ADMITS it (bit_count=405) — i.e. the F5 over-length bug.
  Embedded as `OVERLEN_MD1` literal in `lib.rs` tests.
- **Fail-first (on 0.36):** `refuses_overlength_md1` asserted `convert(OVERLEN_MD1).is_err()`
  and FAILED for the right reason — `over-length (94-symbol) md1 must be refused` (0.36
  admitted it).
- **Bump:** `crates/me-cli/Cargo.toml` `md-codec = "0.40"` (caret; SPEC Open-Q4 keeps caret
  ranges, no `=` pin), `mk-codec` left `"0.4"` (caret already permits 0.4.1).
  `cargo update -p md-codec -p mk-codec` → md-codec 0.36.0→0.40.0, mk-codec 0.4.0→0.4.1
  ("10 unchanged dependencies"; only the 2 codecs moved).
- **STOP-condition check (two symptoms — NEITHER triggered):**
  - (a) Goldens byte-identical: `md1-short.ndef` sha256 `b551af76…` and `bundle-md1-mk1.json`
    sha256 `a728117687…` UNCHANGED before/after; `golden.rs::md1_short_matches_golden` and
    `cli.rs::bundle_manifest_golden` both still pass.
  - (b) No previously-passing mk1/md1 admission test newly failed. The mk-codec 0.4.0→0.4.1
    "live risk" (a previously-valid fixture newly rejected) did NOT materialize — all mk1
    fixtures (accepts_valid_mk1, parses_mk1_chunk_with_set_id, happy_path, multi_set,
    reordered, cross_chunk_hash, dropped/duplicate chunk, md1_chunked_set) still green.
- New test now PASSES on 0.40. The fail-closed path is `ConvertError::Validate(ValidateError::Md(
  md_codec::Error::StringSymbolCountOutOfRange { symbols: 94, max: 93 }))` — Display:
  `invalid md1 string: string has 94 symbols; the codex32 regular code caps a string at 93`
  (assertion `contains("symbols") && contains("caps")` passed).
- Full `cargo test` with go + `ME_REQUIRE_GO=1`: lib 43 (+1), cli 20, cross_lang 1, golden 1,
  preview_cross_lang 1 → all green, exit 0.

Touches: `crates/me-cli/Cargo.toml`, `Cargo.lock`, `crates/me-cli/src/lib.rs`.

---

## Step 5 (A3) — refuse non-canonical md1 (done)

- **Fail-first (6 tests failed for the right reason; 1 positive-guard passed pre-impl):**
  - lib.rs convert-level: `refuses_noncanonical_md1_interior_dash`/`_space`/`_newline` and
    `noncanonical_md1_error_names_char_and_byte_position` all FAILED (convert returned Ok —
    md-codec stripped the separator and BCH-passed, then convert emitted the raw bytes).
  - cli.rs exit-code: `convert_refuses_interior_separator_md1_exit_4` and
    `bundle_refuses_interior_separator_md1_exit_4_no_leak` FAILED (exit 0, not 4).
  - `clean_md1_trailing_newline_is_byte_identical` PASSED pre-impl (guard must not regress it).
- **Fix (single shared admission path — `validate::validate`, NOT convert/parse_line):** added
  a `Format::Md`-gated check that runs BEFORE `unwrap_string`: if the (already-trimmed) string
  contains any `char::is_whitespace()` or `-`, return the new `ValidateError::MdNonCanonical
  { ch, pos }` (offending char + byte position; never the input body). Display:
  `non-canonical md1: interior separator '-' at byte 8 — md1 must contain no '-' and no
  interior whitespace …`. mk1 arm untouched (mk-codec already rejects these as InvalidChar).
- **Ordering note:** the canonical check runs before `unwrap_string`, so `OVERLEN_MD1` (no
  separators) still passes the canonical check and hits `StringSymbolCountOutOfRange` — Step 4's
  test unaffected.
- All 8 Step-5 tests pass. `md1-short` golden + every prior test still green. Full suite:
  lib 48 (+5), cli 22 (+2), cross_lang 1, golden 1, preview_cross_lang 1 → exit 0.

Touches: `crates/me-cli/src/validate.rs`, `crates/me-cli/src/lib.rs` (tests),
`crates/me-cli/tests/cli.rs`.
