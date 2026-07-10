# Implementation log — me fuzz + proptest targets (Cycle C / F18)

Single-implementer TDD execution of `design/SPEC_me_fuzz_proptest.md`
(GREEN at R0 round 1, 0C/0I). Worktree `me-cycleC`, branch `me-fuzz-proptest`.
One section per step; committed after each green step.

## Environment / preconditions

- Toolchain (local): `cargo/rustc 1.97.0-nightly` (nightly is the default here, so
  `cargo` == `cargo +nightly`). CI targets stable 1.85.0 with `cargo test --locked`.
- `cargo-fuzz 0.13.2` IS installed and nightly IS available → the C2 fuzz crate can be
  built and smoke-run locally (not just written).
- Root `Cargo.lock` verified free of `proptest`/`libfuzzer-sys`/`arbitrary`/`quickcheck`
  before any change.
- `third_party/seedhammer` submodule was UNINITIALIZED in this fresh worktree (worktrees
  need their own submodule checkout). Initialized to the pinned `713aee2` (v1.4.2) so the
  differential `cross_lang`/`preview_cross_lang` tests run for the `ME_REQUIRE_GO=1`
  verification. Not a code change; the submodule pointer is already recorded upstream.
- **Clean baseline** (`env PATH=…go… ME_REQUIRE_GO=1 cargo test --locked`, all green):
  lib **54** + cli **23** + cross_lang **1** + golden **3** + preview_cross_lang **1**
  = **82** real tests. Expect **88** after C1 adds prop.rs (6 properties).

## Step C1 — proptest layer (CI-covered, load-bearing)

Files added / changed:
- `crates/me-cli/tests/support/invariants.rs` — the SHARED checkers (P1–P6), in a
  `tests/` SUBDIR (not a bare `tests/*.rs`, which cargo would compile as a spurious
  0-test target → dead-code warnings under `clippy -D warnings`; R0 L1). Rides ONLY
  already-`pub` API, referenced as `mnemonic_engrave::…` (never `crate::`; R0 N-d) so the
  one file compiles in BOTH the proptest test crate and the separate fuzz crate. Adds
  ZERO new `pub` items to the published lib (R0 I2).
- `crates/me-cli/tests/prop.rs` — the 6 properties, `#[path = "support/invariants.rs"]
  mod invariants;`, `ProptestConfig::with_cases(256)`.
- `crates/me-cli/Cargo.toml` — `proptest = "1"` dev-dep only.
- `Cargo.lock` — regenerated (resolved `proptest 1.11.0`). MSRV audit vs CI's stable
  1.85.0: every HOST-compiled resolved dep declares rust-version ≤ 1.85.0 (proptest
  1.11.0 = exactly 1.85; getrandom 0.4.3 = 1.85). The only >1.85 entry is
  `wasip2 1.0.4` (rust-version 1.87.0), which is gated to
  `cfg(all(target_arch="wasm32", target_os="wasi", target_env="p2"))` and is NEVER
  compiled on the x86_64-linux CI host (`cargo tree --target x86_64-unknown-linux-gnu`
  shows no wasip2). So `cargo test --locked` on stable 1.85.0 is unaffected.

Properties (each maps to one shared checker):
- P1 `p1_convert_never_panics` — `prop_oneof![arb_text (?s).{0,300}, biased_line]`.
- P2 `p2_convert_refuses_ms` — `ms_line` (leading `ms`-token, case/whitespace variants).
- P3 `p3_run_bundle_never_panics` — `prop_oneof![multiline (vec.join("\n")), arb_text]`.
- P4 `p4_run_bundle_refuses_ms_line` — `multiline_with_ms` (ms1 as leading token of its
  own line at an arbitrary position; R0 N-c).
- P5 `p5_manifest_strings_trace_to_input` — `prop_oneof![valid_bundle (Ok-producing,
  whitespace/blank-line padded), multiline]`; `.lines().map(trim).any(==)` (CRLF-aware;
  R0 N3). valid_bundle uses the crate's own proven-valid fixtures (MD1_VALID single;
  MK1_A/MK1_B complete 0x12345 set).
- P6 `p6_ndef_round_trip` — arbitrary UTF-8 + a byte-length sweep in 1-byte (`a`) AND
  3-byte (`€`) units across the 249/250 boundary; Result-aware, keyed on BYTE length
  (R0 I1).

Clean run: `cargo test --locked --test prop` → **6 passed** (properties hold on current
code — insurance, no live bug).

TDD teeth — perturb-then-revert (each: edit src, run the one property, observe RED,
`git checkout -- <src>`; src tree confirmed pristine after; no spurious
`proptest-regressions/` seed persisted):

| Property | Perturbation | Observed RED |
|----------|--------------|--------------|
| P1 | `panic!` at top of `convert` (lib.rs:57) | `p1_… FAILED` — panic caught |
| P3 | `panic!` at top of `run_bundle` (bundle.rs:184) | `p3_… FAILED` — panic caught |
| P2 | remove `convert`'s `RefusedSecret` return | `p2_… FAILED` — ms no longer refused (routes into validate) |
| P4 | remove BOTH bundle ms-refusal sites (defense-in-depth) | `p4_… FAILED` — ms reaches `unreachable!`, panic caught |
| P5 | fabricate the unchunked-md1 plate string (`format!("{s}X")`) | `p5_… FAILED`: "not a verbatim trimmed input line: \"md1…qhX\"" |
| P6 | drop first text byte in `decode_text_record` | `p6_… FAILED` — round-trip `left != right` |

All perturbations reverted; clean `--test prop` re-run → 6 passed.

## Step C2 — cargo-fuzz crate (local/deep, nightly, NOT CI-built)

Files added / changed:
- `crates/me-cli/fuzz/Cargo.toml` — package `mnemonic-engrave-fuzz`, `libfuzzer-sys 0.4`
  + path dep on `mnemonic-engrave`, its OWN empty `[workspace]` table (self-detach; R0
  L2), two `[[bin]]` targets.
- `crates/me-cli/fuzz/fuzz_targets/fuzz_convert.rs` — `assert_convert_no_panic` +
  `assert_convert_ms_refused`.
- `crates/me-cli/fuzz/fuzz_targets/fuzz_run_bundle.rs` — `assert_run_bundle_no_panic` +
  `assert_bundle_ms_line_refused` + `assert_manifest_strings_trace`.
- Both targets `#[path = "../../tests/support/invariants.rs"] mod invariants;` — the SAME
  shared file the proptest layer uses (`#[allow(dead_code)]` since each target uses a
  subset). CI residual (nightly-only, not CI-built) stated in each target's header comment
  and justified by the proptest layer covering every invariant (R0 L4).
- `crates/me-cli/fuzz/Cargo.lock` — the fuzz workspace's OWN lock (committed; contains
  libfuzzer-sys/arbitrary, isolated from root).
- Root `Cargo.toml` — `exclude = ["crates/me-cli/fuzz"]` (belt-and-suspenders).
- `.gitignore` — `/crates/me-cli/fuzz/{corpus,artifacts,target}` (R0 N2).

**Isolation proof (the highest-risk executability check, R0 L2):**
- Regenerated root lock after the fuzz wiring → `git diff --stat Cargo.lock` **empty**
  (root lock unchanged).
- `grep -iE 'libfuzzer|arbitrary' Cargo.lock` (root) → **NONE**.
- `cargo tree` (root) → **NO** libfuzzer-sys / arbitrary.
- `cargo metadata --no-deps` workspace packages = `['mnemonic-engrave']` only —
  `mnemonic-engrave-fuzz` is NOT a workspace member.
- The fuzz crate's OWN `fuzz/Cargo.lock` DOES contain libfuzzer-sys/arbitrary (4 entries)
  → the nightly deps live only in the detached fuzz workspace.
→ stable `cargo test --locked` at the root can never pull the nightly fuzz deps.

**Build + smoke (nightly available locally):**
- `cargo +nightly fuzz build --fuzz-dir crates/me-cli/fuzz` → both targets compiled
  (release, libFuzzer + ASan), `Finished` in ~72s.
- `cargo +nightly fuzz run … fuzz_convert -- -runs=2000` → `Done 2000 runs`, cov 105,
  **no crash/panic/leak**.
- `cargo +nightly fuzz run … fuzz_run_bundle -- -runs=2000` → `Done 2000 runs`, cov 216,
  **no crash/panic/leak**.
- Runtime `corpus/`, `target/` confirmed git-ignored (`!!`).

## Final verification (all GREEN)

- `env PATH=…go… ME_REQUIRE_GO=1 cargo test --locked` (root) → **88** real tests, all pass
  (baseline 82 + prop 6): lib 54 + cli 23 + cross_lang 1 + golden 3 + preview_cross_lang 1
  + prop 6. (Go differential tests ran for real, not skipped.)
- `cargo test --locked` does NOT pull libfuzzer-sys (proven via the tree/lock check above).
- `cargo clippy --all-targets --locked -- -D warnings` (workspace; fuzz excluded) → clean
  (exit 0). Confirms `tests/support/invariants.rs` in a SUBDIR is NOT a spurious 0-test
  target (R0 L1): it compiles only via prop.rs's `#[path]` include, where all 6 checkers
  are used → no dead_code.
- `go test ./...` in `preview/` → ok.
- nightly + cargo-fuzz 0.13.2 available → fuzz build + smoke succeeded (recorded above).
