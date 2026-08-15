# Required-contexts advice — mnemonic-key / mnemonic-secret (2026-08-15)

**This is an agent's advisory recommendation for a human decision, not a decision — no repository setting was changed, and nothing here is authorized until the operator runs it.**

Method: workflow files read in full from both working trees; current protection fetched read-only via
`gh api repos/bg002h/<repo>/branches/<branch>/protection`; exact context names and durations taken from
`commits/<branch>/check-runs` on today's default-branch HEADs; failure history from the workflow-runs API
by path (`actions/workflows/ci.yml/runs`, `actions/workflows/rust.yml/runs`), not `gh run list`.

## Two GitHub semantics that bound every choice below

1. **A skipped job satisfies a required check.** GitHub docs: "A job that is skipped will report its
   status as Success. It will not prevent a pull request from merging, even if it is a required check."
   Successful conclusions for required checks are `success`, `skipped`, and `neutral`.
   (docs.github.com → Status checks reference; also emmer.dev "Skippable GitHub Status Checks Aren't
   Really Required".) Consequence: a required context reachable only through `needs:` can be *silently
   satisfied* when its dependency fails.
2. **A workflow that never fires never creates the check run**, so a required context from a
   path-filtered workflow leaves non-matching PRs wedged at "Expected — waiting for status" forever.
   `rust.yml` documents exactly this ruling in its own PR-trigger comment. Consequence:
   `vendor-freshness` and the fuzz `build` gate are **structurally ineligible** as required contexts
   as long as they carry PR `paths:` filters.

Both repos' current required entries pin `app_id: 15368` (GitHub Actions). Keep that pin on every new
entry — an unpinned context name can be satisfied by any app that posts a status with that string.

Context strings must match **exactly**: the mk formatting context is `fmt (pinned 1.95.0)`, not `fmt`.
A rule naming `fmt` would silently match nothing and wedge every PR.

---

## Repo 1: `bg002h/mnemonic-key` (default branch `main`)

Current protection (fetched 2026-08-15): `strict: false`, contexts = **`build (stable on ubuntu-latest)`**
only. `enforce_admins: false` (settled, out of scope).

### Every PR-time context (workflows: `ci.yml`, `vendor-freshness.yml`, `fuzz-smoke.yml`; `musl-binaries.yml` is tag/dispatch-only)

Durations are from today's green run on HEAD `3462157` (run start 08:15:43Z; whole run ≈ 2m39s).

| Exact context string | What it actually checks (from steps) | Duration | Required now |
| --- | --- | --- | --- |
| `build (stable on ubuntu-latest)` | `cargo build/test/clippy --workspace -D warnings`. The workspace tests **include mk-codec's corpus conformance** (`tests/vectors.rs` with a pinned SHA-256 over `test_vectors/v0.1.json`, `round_trip.rs`, `error_coverage.rs`) | 36s | **YES** |
| `build (beta on ubuntu-latest)` | same, beta toolchain | 42s | no |
| `build (1.85 on ubuntu-latest)` | same, MSRV toolchain | 49s | no |
| `build (stable on macos-latest)` / `(beta …)` / `(1.85 …)` | same, macOS | 34–65s | no |
| `build (stable on windows-latest)` / `(beta …)` / `(1.85 …)` | same, Windows | 105–131s | no |
| `fmt (pinned 1.95.0)` | `sha256sum -c design/display-grouping-vectors.tsv.sha256` (conformance-artifact tripwire) **then** `cargo fmt --check --all` under the pinned canonical formatter | 27s | no |
| `vectors-roundtrip` | release-builds `mk`, runs `mk vectors --out`, jq-normalized diff of the emitted fixtures against the pinned corpus `crates/mk-codec/src/test_vectors/v0.1.json`. `needs: build` (all 9 cells) | 24s (starts after matrix; run-relative ≈ 2m39s) | no |
| `freebsd compile-gate (whole-crate)` | `cargo check --target x86_64-unknown-freebsd -p mk-cli` (BSD process_hardening arm) | 27s | no |
| `musl compile/test (x86_64-unknown-linux-musl)` | native musl `cargo test -p mk-cli` | 47s | no |
| `musl compile/test (aarch64-unknown-linux-musl)` | `cross test --release` under QEMU (output-verifies goldens on aarch64; `cargo install cross` from network) | 75s | no |
| `vendor/ satisfies Cargo.lock (offline)` | offline lockfile resolution against committed `vendor/` | 16s | no — **ineligible** (PR path filter) |
| `cargo fuzz build (compile gate)` | fuzz-target compile gate | ~45s | no — **ineligible** (PR path filter) |

Flake history, measured (last 15 `ci.yml` runs on `main`): 7 failures 2026-06-25 → 2026-07-09, **all**
`freebsd compile-gate (whole-crate)` — the documented `dtolnay/rust-toolchain@master` drift (E0463),
fixed by pinning `@1.85.0`. No other job failed in the sample. The nine build cells were
deterministic-green throughout.

### Is `vectors-roundtrip` the mechanism the Go ports are bound to? **Yes, plainly.**

The fork's Go tests say so verbatim — `/scratch/code/shibboleth/seedhammer/codex32/mdmk_test.go`:
"Golden vectors are RUST-SOURCED (md-codec 0.36 / **mk-codec v0.1.json**), never Go-self-generated".
That file is exactly the corpus `vectors-roundtrip` diffs the built `mk` binary against. Precision on
what the current required set already proves: codec-level corpus conformance *is* inside the required
`build (stable on ubuntu-latest)` cell (via `cargo test --workspace` and the pinned corpus SHA). What
is **outside** every required context today: the release-binary emission path (`mk vectors --out` —
the command a downstream porter actually runs), formatting, the checksum tripwire on
`display-grouping-vectors.tsv`, MSRV, and all non-Linux cells.

### RECOMMENDED required set — `mnemonic-key`

1. `build (stable on ubuntu-latest)` — keep.
2. `fmt (pinned 1.95.0)` — **add**. Buys: the deterministic pinned-formatter gate *and* the
   conformance-vector checksum tripwire (mnemonic-secret already gates its identical checksum step via
   required `clippy`; this closes the asymmetry). Costs: 27s, zero observed flakes, formatter pinned so
   no drift-reds.
3. `vectors-roundtrip` — **add**. Buys: the merge rule now proves the property the Rust-primary rule
   exists for — the shipped binary emits exactly the corpus the Go ports copy. Costs: 24s of work but
   ~2m39s wall (it waits on all nine build cells), and **one workflow-side caveat the operator should
   fix when (or before) requiring it**: because skipped-counts-as-success (semantics §1), a red
   *beta/windows/macos* cell skips `vectors-roundtrip` and the skip **satisfies** the rule — a
   false-green path. Fix: drop `needs: build` from the `vectors-roundtrip` job (it is self-contained;
   it builds `mk-cli` itself and has its own cache key). That is a one-line workflow edit, and it also
   cuts the context's wall time from ~2m39s to ~1min.

Optional fourth, if the operator wants one more: `build (1.85 on ubuntu-latest)` (MSRV; 49s,
deterministic; protects `cargo install` users on the published MSRV, which the freebsd/musl gates also
pin). Not load-bearing for the Rust-primary rule, so it is listed as a choice, not a recommendation.

Deliberately **not** recommended as required: the six non-Linux/beta cells (portability tier — they
stay visible red on the commit status without being able to block on a mac-runner queue),
`freebsd compile-gate` (the only job with a real flake history — 7 consecutive infra failures — and a
required context that reds for infra reasons trains bypass), the musl legs (network `cargo install` +
QEMU; slowest legs), and `vendor-freshness` / fuzz `build` (**ineligible** while PR-path-filtered —
requiring either wedges every non-matching PR at "Expected"). If the operator wants `vendor-freshness`
required — and its own header makes a strong case, it exists because v0.74.0 shipped unreproducible —
the path filter must be removed from its `pull_request` trigger first; at 16s it is cheap enough to run
unconditionally.

**Command to apply (operator's copy-paste; fish-safe):**

```sh
printf '%s' '{"strict":false,"checks":[{"context":"build (stable on ubuntu-latest)","app_id":15368},{"context":"fmt (pinned 1.95.0)","app_id":15368},{"context":"vectors-roundtrip","app_id":15368}]}' \
  | gh api -X PATCH repos/bg002h/mnemonic-key/branches/main/protection/required_status_checks --input -
# verify:
gh api repos/bg002h/mnemonic-key/branches/main/protection/required_status_checks
```

---

## Repo 2: `bg002h/mnemonic-secret` (default branch `master`)

Current protection (fetched 2026-08-15): `strict: false`, contexts = `test (ubuntu-latest)`, `clippy`,
`test (ms-codec)`, `clippy (ms-codec)`. `enforce_admins: false` (settled, out of scope).

### Every PR-time context (workflow `rust.yml` — PR trigger deliberately unfiltered; plus path-filtered `vendor-freshness.yml`, `fuzz-smoke.yml`; `man-release.yml` is tag/dispatch-only)

Durations from today's run on HEAD `de593ca` (start 07:30:35Z; whole run ≈ 2m39s, conclusion **failure**).

| Exact context string | What it actually checks (from steps) | Duration | Required now |
| --- | --- | --- | --- |
| `test (ubuntu-latest)` | build + `cargo test -p ms-cli` (includes `tests/vectors_parity.rs` against the pinned corpus) + mlock G2.1/G2.3-debug/G2.4 fault-injection in fresh subprocesses | 50s | **YES** |
| `test (macos-latest)` | same on macOS (darwin mlock path) | 53s | no |
| `clippy` | `sha256sum -c design/display-grouping-vectors.tsv.sha256` then `cargo clippy -p ms-cli --all-targets -D warnings` | 28s | **YES** |
| `test (ms-codec)` | `cargo test -p ms-codec` — **the codec vector mechanism**: `tests/vectors.rs`, `bch_drift.rs`, `bch_decode.rs`, `parity_smoke.rs`, all against pinned `tests/vectors/v0.1.json` | 28s | **YES** |
| `clippy (ms-codec)` | `cargo clippy -p ms-codec --all-targets -D warnings` | 23s | **YES** |
| `fmt (pinned 1.95.0)` | rustfmt 1.95.0 whole-workspace check, `mlock.rs` exempt (g6 pin) | 15s | no |
| `test (release, ubuntu-latest, mlock einval)` | release-build G2.3: EINVAL must soft-fail (the `debug_assert` is compiled out — a branch no debug test can reach) | 44s | no |
| `miri (mlock unsafe)` | floating-nightly Miri over mlock's 2 unsafe blocks | 51s | no |
| `g6 invariant (cross-repo mlock.rs)` | byte-sync of `ms-cli/src/mlock.rs` vs **mnemonic-toolkit master's** copy | 41s — **FAILING on HEAD now** | no |
| `freebsd compile-gate (whole-crate)` | `cargo check --target x86_64-unknown-freebsd -p ms-cli` | 34s | no |
| `musl compile/test (x86_64-…)` / `(aarch64-…)` | native musl test / `cross test` under QEMU (goldens, mlock skipped) | 54s / 156s | no |
| `vendor/ satisfies Cargo.lock (offline)` | offline lockfile resolution vs `vendor/` | 16s | no — **ineligible** (PR path filter) |
| `cargo fuzz build (compile gate)` | fuzz-target compile gate | ~45s | no — **ineligible** (PR path filter) |

Flake history (last 15 `rust.yml` runs on `master`): 14 green, 1 failure — and the failure is **today's
HEAD**.

### Live finding — the observation's thesis, demonstrated on this repo today

HEAD `de593ca` ("style(ms-cli): rustfmt mlock.rs under the pinned 1.95.0 toolchain") reformatted the
one file the workflow's own fmt-exemption comment says must **not** be reformatted unilaterally, broke
`g6 invariant (cross-repo mlock.rs)` (`g6_mlock_normalized_source_byte_equal` FAILED, own_lines=359 vs
sibling_lines=355 — the toolkit's copy has not landed its half of the lockstep), and **landed on
master anyway** because g6 is not a required context. All four required contexts are green on that
SHA. "The required check passed" is, right now, weaker than it looks on this repo — same shape as the
fork finding that prompted this report. Action item independent of any protection change: land the
toolkit-side reformat (or revert `de593ca`'s mlock.rs hunk) so g6 is green again.

And yet: **do not make g6 required.** It compares against a *different repo's moving master*, so any
lockstep change must land red on one side by construction — a required g6 converts every legitimate
cross-repo sync into an operator bypass, which trains exactly the habit the rule exists to prevent. It
is a real invariant that belongs in the visible suite, watched, not in the merge rule.

### RECOMMENDED required set — `mnemonic-secret`

Keep all four current contexts, add two:

5. `fmt (pinned 1.95.0)` — **add**. Buys: the pinned deterministic formatter gate, symmetric with the
   mk recommendation. Costs: 15s, zero observed flakes. (Note its mlock.rs exemption logic means it
   cannot red on the g6-pinned file — today's g6 red did *not* red fmt.)
6. `test (release, ubuntu-latest, mlock einval)` — **add**. Buys: the only coverage of the
   release-build secret-hygiene soft-fail branch (the debug_assert is compiled out in release, so no
   debug-profile test can reach it); this repo handles seed shares, and mlock *is* the secret-hygiene
   mechanism. Costs: 44s, deterministic in the sample, single-platform.

Deliberately **not** recommended as required: `miri` (real value on the unsafe blocks, but it floats
`nightly` — it can red on a rustc regression with zero defect here; watched, not required),
`g6 invariant` (above), `test (macos-latest)` (portability tier, mac-runner queue exposure — same
ruling as mk's non-Linux cells), `freebsd` / `musl` legs (same tier; the sibling freebsd gate carries
the 7-failure flake history), `vendor-freshness` / fuzz `build` (**ineligible** while PR-path-filtered
— same fix available as on mk if the operator wants them).

**Command to apply (operator's copy-paste; fish-safe):**

```sh
printf '%s' '{"strict":false,"checks":[{"context":"test (ubuntu-latest)","app_id":15368},{"context":"clippy","app_id":15368},{"context":"test (ms-codec)","app_id":15368},{"context":"clippy (ms-codec)","app_id":15368},{"context":"fmt (pinned 1.95.0)","app_id":15368},{"context":"test (release, ubuntu-latest, mlock einval)","app_id":15368}]}' \
  | gh api -X PATCH repos/bg002h/mnemonic-secret/branches/master/protection/required_status_checks --input -
# verify:
gh api repos/bg002h/mnemonic-secret/branches/master/protection/required_status_checks
```

---

## Ruling on the `ci-ok` aggregator restructure: **a weakening here — do not adopt it**

Two independent reasons, both specific to this environment:

1. **Who edits what.** In this project the merge rule's real function is to bind *agent-driven* pushes
   (the operator ruled themselves the bypass). Agents author workflow files routinely, so an aggregator
   moves the definition of "required" into a file the same push can edit — a quiet `needs:` deletion
   inside an otherwise-normal diff, with the protection rule still showing one green `ci-ok`. Repo
   settings are the one channel agents do not touch by norm (this very report is advisory for that
   reason), and a settings change is out-of-band and auditable. Keeping the context list in the
   protected setting keeps the control surface where the threat isn't.
2. **The skipped-counts-as-success footgun.** A naive `ci-ok` with `needs: [...]` is *skipped* when a
   dependency fails, and a skipped required check **passes** branch protection. Making it sound
   requires `if: always()` plus explicit per-dependency result assertions — a discipline that must be
   re-verified every time the job is touched. The same semantics are why `vectors-roundtrip`'s
   `needs: build` should be dropped before it is required (mk section above).

## Should the two repos match? **In shape, yes — and after this change they do. In literal strings, they cannot.**

The current divergence (1 of 14 contexts required on mk vs 4 of 13 on ms) is drift, not policy — mk's
one-context rule predates the growth of its workflow, and nobody chose "vector conformance gated on ms
but not on mk." The repos cannot share literal context strings because the vector mechanism is
structured differently (mk: a dedicated `vectors-roundtrip` job diffing the built binary's emission;
ms: corpus tests inside `test (ms-codec)` + `vectors_parity.rs` inside `test (ubuntu-latest)`). What
should match — and under the recommended sets does — is the **floor**: *default-toolchain
build+test+clippy on Linux, the pinned formatter, the conformance-artifact checksum, and the pinned
vector corpus proven against the built artifact, on every merge.* The one principled asymmetry is
ms's extra `test (release, …, mlock einval)`: ms has secret-handling runtime code with a
release-only branch; mk's equivalent surface (process_hardening) is compile-covered inside its
required build cell. Asymmetric surface, symmetric principle.

## Interaction with the concurrent `ci/**` staging change (assumed, not duplicated)

Every context recommended above lives in `ci.yml` / `rust.yml`, so once those workflows fire on
`ci/**` pushes, a staged SHA earns the full recommended set before touching the default branch —
no new wedge is introduced. One pre-existing gap, unchanged by this recommendation: `rust.yml`'s
*push*-side `paths:` filter means a docs-only staged push still earns no `rust.yml` contexts (the
workflow's own comment documents admin bypass as the accepted answer there). Widening the required
set does not widen that gap; it existed for the current four contexts already.

## Out of scope, per the brief

`enforce_admins: false` on both repos is deliberate and settled ("You are not permitted to bypass,
but I am") — no recommendation is made about it. No repository setting, file, or ref in either
primary repo was modified: `git status --porcelain` is empty in both.
