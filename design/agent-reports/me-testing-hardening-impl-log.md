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
