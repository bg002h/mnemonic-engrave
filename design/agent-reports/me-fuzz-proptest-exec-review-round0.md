# Post-Implementation Adversarial Execution Review — Cycle C / F18 (fuzz + proptest)

- **Artifact under review:** full diff `master..me-fuzz-proptest` (worktree `/scratch/code/shibboleth/me-cycleC`)
- **Commits:** `fac0106` spec (GREEN R0 r1) → `dd3657f` C1 proptest → `5ea7295` C2 cargo-fuzz
- **Reviewer role:** mandatory independent post-impl adversarial execution review (R0 = plan correctness; this = implementation-introduced regressions TDD misses)
- **Round:** 0
- **Environment:** rustc default nightly 1.97.0; toolchains incl. `1.85.0` (pre-installed); Go 1.26.4; cargo-fuzz 0.13.2; submodule `third_party/seedhammer` @ `713aee2` (v1.4.2) populated. All builds/tests run with `CARGO_TARGET_DIR` in `/scratch/code/shibboleth/me-review-scratch-c` (outside the worktree); scratch cleaned up.

## Verdict: **GREEN — 0 Critical / 0 Important** (1 Low, 1 Nit — non-blocking)

Every priority-order risk was verified EMPIRICALLY, not by trusting the impl log. The load-bearing MSRV risk (impl flag #1) is **proven safe**: all test targets incl. `tests/prop.rs` + `proptest 1.11.0` compile clean under stable `1.85.0 --locked`. Fuzz deps are fully detached from the stable CI resolution. No new `pub` API. All three key property teeth reproduced RED by me. Full suite (88), clippy `-D warnings`, `go test`, and a fresh `cargo fuzz build` + smoke all green. The single Low is avoidable-but-harmless production-dep churn in the root lock; it does not gate the PR.

---

## 1. MSRV / stable-1.85.0 CI safety — **PROVEN SAFE (impl flag #1 downgraded)**

The priority risk: CI runs stable `1.85.0` + `cargo test --locked`; if the new proptest dev-dep's resolved lock needs rustc >1.85 for any HOST-built dep, CI reds. Verified empirically:

```
$ CARGO_TARGET_DIR=…/target-1850 cargo +1.85.0 test --locked --no-run   (in the worktree)
   Compiling proptest v1.11.0
   Compiling rand v0.9.4 / rand_core v0.9.5 / getrandom v0.2.17 …
   Compiling base58ck v0.1.101 / secp256k1 / bip39 / miniscript / mk-codec / md-codec …
   Compiling mnemonic-engrave v0.3.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.59s
  Executable tests/prop.rs (…/prop-4ddbeb0f5fe980d6)   ← proptest target compiled
EXIT=0
```

- **`cargo +1.85.0 test --locked --no-run` → EXIT 0**, compiling ALL test targets including `tests/prop.rs` and `proptest 1.11.0`. MSRV is proven safe on the exact CI toolchain.
- **wasip2 1.87 claim spot-checked and confirmed benign:** `wasip2 1.0.4+wasi-0.2.12` IS in the root lock (an entry only, via getrandom's wasi target), but `cargo +1.85.0 tree --locked --target x86_64-unknown-linux-gnu` shows the me-cli subtree as `└── proptest v1.11.0` with **no wasip2/libfuzzer/arbitrary**, and the 1.85.0 build above **never compiled wasip2** and still Finished. It is target-gated to `wasm32-wasi-p2` and never built on the linux CI host. No MSRV impact.

Impl flag #1 (MSRV/1.85.0) is resolved GREEN.

## 2. Isolation — fuzz deps do NOT reach stable CI — **PASS**

- Root `Cargo.lock`: `grep -iE 'libfuzzer|arbitrary'` → **NONE**.
- `cargo metadata --no-deps` workspace packages → **`['mnemonic-engrave']` only**; `mnemonic-engrave-fuzz` is NOT a member.
- Root `Cargo.toml`: `members = ["crates/me-cli"]` (explicit) + `exclude = ["crates/me-cli/fuzz"]`.
- Fuzz crate self-detaches with its OWN empty `[workspace]` table (`crates/me-cli/fuzz/Cargo.toml:21`) and its OWN `fuzz/Cargo.lock` (which DOES contain 4 libfuzzer/arbitrary entries — correctly isolated there).
- `cargo +1.85.0 tree --target x86_64-linux` host subtree = proptest only.
→ Stable `cargo test --locked` at the root can never pull the nightly-only fuzz deps. Both mechanisms (own `[workspace]` + root `exclude`) present and verified.

## 3. No-new-pub-API (R0 I2) — **PASS**

- `git diff master..HEAD -- crates/me-cli/src` for `^\+.*\bpub\b` → **NONE**. The entire `src/` diff is empty (confirmed `git diff --name-only` lists no `src/` file).
- `tests/support/invariants.rs` rides only already-`pub` API and references the crate by external name: 5 `mnemonic_engrave::` uses, **0 `crate::` in code** (the single `crate::` match is doc-comment prose at line 12). Cross-context compilability is not just claimed — it is proven by BOTH the proptest build and the `cargo fuzz build` succeeding on the same shared file.
- The `pub fn assert_*` in `invariants.rs` are test-only (under `tests/`, never in the published library crate) → zero new public surface on the published `mnemonic-engrave`. `proptest` is a dev-dependency (not in the published dependency closure).

## 4. Property teeth are real — **PASS (3 perturbations reproduced RED by me, in an isolated scratch worktree)**

Baseline in a fresh `git worktree`: `cargo test --test prop` → **6 passed**. Then:

| Perturbation (my own edit) | Property | Result |
|---|---|---|
| (a) removed `convert`'s `RefusedSecret` return (`lib.rs`) | P2 | **RED** — `p2_convert_refuses_ms FAILED` on `s="ms1"` (ms routed into validate → `unreachable!` panic caught) |
| (b) dropped first text byte in `decode_text_record` (`ndef.rs`, `1+lang_len` → `2+lang_len`) | P6 | **RED** — `p6_ndef_round_trip FAILED`: `left: None, right: Some("")` |
| (c) fabricated the unchunked-md1 plate string (`Some(format!("{s}X"))`, `bundle.rs`) | P5 | **RED** — `p5_… FAILED`: "emitted plate string is not a verbatim trimmed input line: \"md1…qhX\"" |

Each perturbation reverted; worktree returned pristine (`git status --porcelain` empty). Properties are genuine guards, not theater.

- **P6 boundary re-derived from `ndef.rs` = byte-length 249/250 EXACT.** `text_record` payload = `1+text.len()`; `tlv_wrap` errs when message `= 5+text.len() >= 0xFF` ⇒ `text.len() >= 250`. So encode is Ok iff `text.len() <= 249` bytes; `>= 250` → `Err(TooLong(_))`. The checker's `assert!(t.len() >= 250)` in the `TooLong` arm matches exactly. The `ndef_text` strategy sweeps the boundary in BOTH 1-byte (`"a"×0..=260`) and 3-byte (`"€"×0..=90`, crossing at 84 chars = 252 bytes) units — genuinely exercising the byte-vs-char distinction that R0 I1 corrected (a char-count bound would falsely expect Ok at 84 chars).
- **P4 places `ms1` as a leading line token.** `ms_line()` builds `format!("{hrp}1{tail}")` (hrp ∈ ms/MS/Ms/mS, whitespace-padded) and `multiline_with_ms()` inserts it as its own line at an arbitrary index — mirrors P2's "first token" precision (R0 N-c). The checker independently line-scans via `classify` and asserts `run_bundle` refuses.

## 5. Determinism / no-flake — **PASS**

- `cargo test --locked --test prop` run 3× → `6 passed` every time (0.09–0.13s). proptest's default seed is deterministic; `with_cases(256)` keeps it fast.
- No stray `proptest-regressions/` seed committed (`git ls-files | grep proptest-regressions` → none).
- `proptest-regressions/` is **NOT** gitignored (`git check-ignore` → not ignored) — a real future counterexample can be committed and replayed in CI.

## 6. Scope — **PASS (one Low)**

`git diff --name-only master..HEAD` = `.gitignore`, `Cargo.lock`, `Cargo.toml`, `crates/me-cli/Cargo.toml`, `crates/me-cli/fuzz/{Cargo.toml,Cargo.lock,fuzz_targets/*}`, `crates/me-cli/tests/{prop.rs,support/invariants.rs}`, and the 4 design docs (spec + 2 R0 reviews + impl log). **No `src/*.rs` production change; no CLI/behavior change; no A/B/D bleed.** `src/` diff is empty. `.gitignore` correctly ignores only fuzz `corpus/artifacts/target`. See L1 for the one scope wrinkle in the root lock.

## 7. Full suites — **PASS**

- `ME_REQUIRE_GO=1 cargo test --locked` (root, real Go sidecar) → **88 tests, all pass**: lib 54 + cli 23 + cross_lang 1 + golden 3 + preview_cross_lang 1 + prop 6 (matches the impl-log baseline 82 + 6). Go differential tests ran for real (not skipped).
- `cargo clippy --all-targets --locked -- -D warnings` (workspace; fuzz excluded) → **EXIT 0**. Confirms R0 L1: `tests/support/invariants.rs` in a SUBDIR is not a spurious 0-test target (no `dead_code` under `-D warnings --all-targets`).
- `go test ./...` in `preview/` → **ok**.
- `cargo +nightly fuzz build --fuzz-dir crates/me-cli/fuzz` → **EXIT 0** (both targets, release + libFuzzer/ASan). Independent smoke: `fuzz_convert -runs=1500` and `fuzz_run_bundle -runs=1500` → `Done`, **no crash/panic/leak**. Runtime `corpus/target` confirmed git-ignored; worktree stayed CLEAN.

---

## Findings

### Low

**L1 — root `Cargo.lock` incidentally bumps PRODUCTION transitive deps beyond what adding proptest requires (avoidable churn; harmless).** The branch's root lock bumps `bitcoin 0.32.100 → 0.32.101` and `base58ck 0.1.100 → 0.1.101`, and adds new bitcoin-ecosystem transitive crates (`bitcoin-consensus-encoding`, `bitcoin-internals`, `hex-conservative`) — none of which is a proptest dependency. I proved this is avoidable, not necessary: resetting the lock+manifests to `master`, adding `proptest = "1"` to `crates/me-cli/Cargo.toml`, and running a MINIMAL `cargo build --tests` (not `cargo update`) keeps `bitcoin` at `0.32.100` and `base58ck` at `0.1.100` (identical to master) while still resolving proptest. So the committed lock was fully regenerated (`cargo generate-lockfile`/`cargo update`), incidentally re-resolving production transitive deps in a test-only cycle.
- **Impact:** none functional — the bumps are semver-compatible patch releases, MSRV-clean under 1.85.0 (proven: the 1.85.0 build compiled `base58ck 0.1.101` and all bumped crates and Finished), and all 88 tests pass. The only cost is a slightly wider, harder-to-audit diff (a future reviewer may wonder why the F18 fuzz cycle touched `bitcoin`), which cuts against the project's explicit dep/scope discipline.
- **Fix (optional, non-blocking):** regenerate the lock minimally — restore `master`'s `Cargo.lock`, add proptest via `cargo add proptest --dev` + `cargo build`, and commit only the added proptest subtree, leaving `bitcoin`/`base58ck` pinned. Acceptable to ship as-is given it is provably harmless.

### Nit

**N1 — `#[allow(dead_code)]` on the include in `tests/prop.rs` is redundant** (all 6 checkers are used there; the attribute is only load-bearing at the two fuzz-target sites, where each uses a subset). Harmless — kept for symmetry across the three include sites. No action needed.

---

## Cross-checks against the impl log
Every impl-log claim I re-verified held: 88-test count exact; MSRV audit correct (1.85.0 clean, wasip2 target-gated); isolation proof correct (root lock/tree free of libfuzzer/arbitrary); teeth table reproducible (I reproduced P2/P5/P6 reds independently); fuzz build + smoke reproducible. The impl log's four scrutiny flags are all resolved GREEN, flag #1 (MSRV) definitively downgraded by the empirical 1.85.0 compile.

## Bottom line
**GREEN (0C / 0I).** The load-bearing MSRV risk is empirically proven safe (`cargo +1.85.0 test --locked --no-run` → EXIT 0, proptest 1.11.0 + `tests/prop.rs` compiled; wasip2 target-gated, never host-built). Fuzz deps are fully isolated from stable CI; no new pub API; property teeth are real (P2/P5/P6 reproduced RED); determinism holds; scope is test-only. One non-blocking Low (avoidable root-lock production-dep churn) and one trivial Nit. **Cleared for the controller to open the PR.**
