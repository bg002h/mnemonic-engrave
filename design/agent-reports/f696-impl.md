# F-696 implementation report: CI test jobs on cargo nextest

Date: 2026-09-26. Brief: `design/briefs/f696-nextest-ci.md`.

## Outcome

All six repos' Rust test jobs now run on `cargo nextest`. The first CI run on
every branch is green, **and every job executed exactly as many tests as its
baseline** (37 of 37 jobs match, table below). No job was renamed, so no
branch-protection context changes.

Wall-time result, stated plainly: **nextest by itself barely moves the
native jobs.** Those jobs are compile-bound. For example, toolkit
`test (ubuntu-latest)` spends 402 s in `cargo build --tests` and 32 s running
tests, and nextest takes the 32 s to 31 s. The real gains come from two things:

1. **The aarch64-musl legs moved off QEMU.** `cross` cannot drive nextest
   (cross-rs#716, open), so the aarch64 row now runs on a **native
   `ubuntu-24.04-arm` runner**. It keeps the same target triple, the same
   `--release` profile, the same skip filters and the same executed count.
   Toolkit's leg went from **1403 s to 290 s**. That leg was the critical
   path of the whole toolkit `rust` workflow (the "21 min").
2. **gui's three-binary real-binary step** went from 182 s to 66 s, because
   the binaries now run concurrently.

Toolkit `rust` critical path (the longest job): **1403 s → 592 s**. That
592 s is `test (macos-latest)`, which on this run built slower on the macOS
runner: 336 s → 465 s in `cargo build --tests`, a step nextest does not
touch.

The runner move is a scope decision I made, not a literal reading of
"switch to nextest". It was the only way to run those legs under nextest
without a fragile archive/remap scheme. It touches four non-required jobs,
and it can be reverted per repo on its own.

## Branches (not merged, no default branch pushed, no tags)

| repo | branch `f696-nextest` | files |
| --- | --- | --- |
| mnemonic-toolkit | `0324cdcb` | `.github/workflows/rust.yml`, `release.yml` |
| mnemonic-secret | `e3b34a30` | `rust.yml`, `release.yml` |
| descriptor-mnemonic | `31c74785` | `ci.yml`, `release.yml` |
| mnemonic-key | `5cd4d283` | `ci.yml`, `release.yml` |
| mnemonic-engrave | `d49941d1` | `.github/workflows/release.yml`, `scripts/release-workflows/gen.py` |
| mnemonic-gui | `e9b65a63` | `.github/workflows/schema-mirror.yml` |

Worktrees: `/scratch/code/shibboleth/<tk|ms|dm|mk|me|gui>-worktrees/f696`.

The four generated `release.yml` files (tk/ms/md/mk) are `emit.py` output
from the modified `gen.py`. Before the change I confirmed that `emit.py`
reproduced all four committed files byte-for-byte. After the change, the diff
in each is the test job only.

**Not regenerated:** mnemonic-transaction's `release.yml`. It is out of
F-696 scope, and its committed copy already predates the generator's signing
changes. The next `emit.py` run will give it the nextest step too.

## What changed, per job

The install step, identical everywhere:
`taiki-e/install-action@9983c65e42da123ff25d1f78505eb6de315aa172 # v2.87.20`
installs `cargo-nextest@0.9.146`. install-action checks each download against
the sha256 in its own manifest; I confirmed that manifest carries 0.9.146
hashes for linux gnu/musl, macOS and windows. `fallback: none` refuses the
unverified cargo-binstall path. The `rust-toolchain.toml` pins are untouched.

- **Doc-tests:** a `cargo test --doc` step was added wherever a library
  target exists: tk, ms-codec, mk, me, gui and the generated release test
  job. dm already had one. Only **ms-codec has a real doc-test (1)**; every
  other library currently has 0. The step keeps any future doc-test
  covered, as `cargo test` would have. ms-cli is bin-only, so it gets no
  step. The musl `--target` legs get no step either: cargo never ran
  doc-tests with an explicit cross `--target`, and the baseline logs contain
  0 "Doc-tests" lines.
- **Unchanged, still `cargo test`, byte-identical:** every
  `--include-ignored` / env-gated leg and every targeted single-binary leg:
  - toolkit and ms `g2_1` / `g2_3` / `g2_4` mlock fault injection
  - `test (release, …, mlock einval)`
  - `g6 invariant`
  - ms `hashlock_repro` by-name (it greps `cargo test` output)
  - ms `history purge`
  - gui `secret_channels_t10`, `schema_mirror`, the snapshot suites
  - the tk/dm differential workflows
  - miri

  These legs are single-binary or single-test, so nextest cannot parallelise
  anything in them.
- **aarch64 skip filters kept:** `--skip mlock --skip g1_` (tk) and
  `--skip mlock` (ms), now passed after `--` to nextest. Locally,
  `cargo test -p mnemonic-toolkit -- --skip mlock --skip g1_` executed 3963
  tests and nextest executed 3963, with 32 skipped = 15 ignored + 17
  filtered. The semantics are identical.

## Test counts: executed per job, baseline vs after (from the CI logs)

"Executed" means the sum of `passed + failed` over every `test result:` line
in the job log, plus N from every nextest `Summary … N tests run`. Both sides
come from the same extractor. Every row is equal.

| repo | job | before | after (nextest + remaining cargo test) |
| --- | --- | ---: | ---: |
| toolkit rust | test (ubuntu-latest) | 4088 | 4085 + 3 = 4088 |
| toolkit rust | test (macos-latest) | 4080 | 4077 + 3 = 4080 |
| toolkit rust | musl x86_64 | 3975 | 3975 |
| toolkit rust | musl aarch64 | 3962 | 3962 |
| toolkit rust | g6 / release-einval | 2 / 1 | 2 / 1 |
| toolkit release | test (ubuntu-latest) | 4085 | 4085 |
| ms rust | test (ms-codec) | 210 | 207 + 3 = 210 |
| ms rust | test (ubuntu-latest) | 442 | 439 + 3 = 442 |
| ms rust | test (macos-latest) | 437 | 434 + 3 = 437 |
| ms rust | musl x86_64 / aarch64 | 439 / 431 | 439 / 431 |
| ms rust | history purge / g6 / release-einval | 13 / 2 / 1 | 13 / 2 / 1 |
| ms release | test (ubuntu-latest) | 647 | 646 + 1 = 647 |
| dm CI | cargo test (ubuntu / macos / windows) | 1573 / 1572 / 1570 | 1573 / 1572 / 1570 |
| dm CI | musl x86_64 / aarch64 | 880 / 880 | 880 / 880 |
| dm release | test (ubuntu / macos) | 1535 / 1534 | 1535 / 1534 |
| mk CI | build × 9 (stable/beta/1.85 × ubuntu/macos/windows) | 391 / 390 / 387 per OS | identical, all 9 |
| mk CI | musl x86_64 / aarch64 | 188 / 188 | 188 / 188 |
| mk release | test (ubuntu / macos) | 391 / 390 | 391 / 390 |
| engrave | test (rust + go) | 690 | 690 |
| gui | schema-mirror gate | 801 | 778 + 23 = 801 |

Where the "+ n" comes from:
- **tk / ms `+3`:** the g2 legs.
- **ms-codec `+3`:** 1 doc-test + 2 from the hashlock by-name step.
- **ms release `+1`:** the ms-codec doc-test.
- **gui `+23`:** the `t10` and `schema_mirror` legs.

The per-OS differences (for example toolkit macOS 4080 vs ubuntu 4088)
exist identically in the baseline, so they are platform-cfg'd tests, not
something nextest changed.

## Timing: job duration (started → completed, so queue time is excluded)

Before is the `ci/staging` run at the **same base SHA** each branch started
from. After is the first run on the `f696-nextest` commit. The after runs
were triggered in parallel across six repos and queued behind each other,
which excluding queue time neutralises. Hosted-runner variance is roughly
±20% per job, so read single-job deltas under 20% as noise.

| repo / workflow | job | before s | after s |
| --- | --- | ---: | ---: |
| toolkit rust | musl aarch64 (was cross+QEMU) | **1403** | **290** |
| toolkit rust | musl x86_64 | 432 | 284 |
| toolkit rust | test (ubuntu-latest) | 456 | 471 |
| toolkit rust | test (macos-latest) | 418 | 592 (build step 336→465) |
| toolkit rust | g6 / release-einval (unchanged) | 141 / 96 | 166 / 95 |
| toolkit release | test (ubuntu-latest) | 419 | 418 |
| ms rust | musl aarch64 (was cross+QEMU) | **229** | **109** |
| ms rust | musl x86_64 | 134 | 138 |
| ms rust | test (ubuntu / macos) | 131 / 102 | 126 / 124 |
| ms rust | test (ms-codec) | 58 | 55 |
| ms release | test (ubuntu-latest) | 119 | 142 |
| dm CI | musl aarch64 (was cross+QEMU) | **375** | **181** |
| dm CI | musl x86_64 | 137 | 130 |
| dm CI | cargo test (ubuntu / macos / windows) | 221 / 251 / 451 | 224 / 282 / 408 |
| dm release | test (ubuntu / macos) | 230 / 364 | 299 / 308 |
| mk CI | musl aarch64 (was cross+QEMU) | 80 | 101 |
| mk CI | musl x86_64 | 60 | 67 |
| mk CI | build stable (ubuntu / macos / windows) | 63 / 106 / 150 | 67 / 60 / 151 |
| mk CI | build beta (ubuntu / macos / windows) | 58 / 72 / 159 | 70 / 99 / 156 |
| mk CI | build 1.85 (ubuntu / macos / windows) | 72 / 101 / 161 | 69 / 103 / 160 |
| mk release | test (ubuntu / macos) | 119 / 160 | 90 / 124 |
| engrave | test (rust + go) | 205 | 205 |
| gui | schema-mirror gate | **841** | **441** |

Step-level attribution for the jobs that moved:
- **tk aarch64:** the test step went from 1345 s (`cross test`) to 255 s
  (native nextest).
- **tk x86_64 musl:** the test step went from 400 s to 256 s.
- **gui:** real-binary tests went from 182 s to 66 s, and the full suite
  from 214 s to 186 s.
- **dm windows:** the test step went from 372 s to 340 s.
- **engrave and tk ubuntu:** the test steps are unchanged within ±1 s
  (compile-bound).

Critical path of each workflow (its longest job):

| workflow | before s | after s |
| --- | ---: | ---: |
| toolkit `rust` | 1403 | 592 |
| dm `CI` | 451 | 408 |
| ms `rust` | 229 | 138 |
| gui `schema-mirror` | 841 | 441 |
| mk `CI` | 161 | 160 |
| engrave | 205 | 205 |

## CI run ids

| repo | workflow | before (last green) | after (first run, all green) |
| --- | --- | --- | --- |
| mnemonic-toolkit | rust | 36222593327 | 36225038036 |
| mnemonic-toolkit | release | 36222593308 | 36225038058 |
| mnemonic-secret | rust | 36222592925 | 36224961768 |
| mnemonic-secret | release | 36222592923 | 36224961787 |
| descriptor-mnemonic | CI | 36222593476 | 36224962074 |
| descriptor-mnemonic | release | 36222593451 | 36224962041 |
| mnemonic-key | CI | 36222593682 | 36224962091 |
| mnemonic-key | release | 36222593717 | 36224962079 |
| mnemonic-engrave | release (`test (rust + go)`) | 36222593282 | 36224962358 |
| mnemonic-gui | schema-mirror | 36222593118 | 36224961644 |

CI was triggered by pushing each commit to `ci/f696-nextest` (the `ci/**`
trigger). Those refs are now deleted. The `f696-nextest` branches remain.

**Toolkit quirk worth recording.** The first push created `f696-nextest` and
`ci/f696-nextest` at the same SHA in one `git push`. `rust.yml` did not fire,
because it has a `paths:` filter and the ci-ref push carried no *new*
commits. Every other workflow fired. I amended the commit (new SHA
`0324cdcb`), pushed the ci ref first, and cancelled the stale runs. **When
you stage a branch whose commit already exists on another ref, a
path-filtered workflow sees zero changed files.**

## Local nextest pre-check (brief: full suite in each repo first)

| repo | nextest result | doc-tests |
| --- | --- | --- |
| tk | 4085/4085 | 0 |
| ms-codec | 207/207 | 1 |
| ms-cli | 439/439 | — |
| dm | 1573/1573 | 0 |
| mk | 391/391 | 0 |
| me (with `ME_REQUIRE_GO=1`) | 690/690 | 0 |
| gui | 766/773 | 0 |

The 7 gui failures reproduce identically under plain `cargo test`. They are
environment, not isolation: `MNEMONIC_BIN` is unset locally, and the local
`mk` does not match the pinned schema. They pass in CI, where the job
provides both.

Fixed-path / port grep over tests found only argv strings and "must not
exist" assertions, with no shared writes. No first-party `harness = false`
targets exist; nextest cannot run those.

## Notes for the controller

- **Trade-off (memory: "nextest isolation hides shared-state bugs").** No CI
  job now runs the main suites under libtest's shared-process threading. A
  test that only fails when it shares a process with another test in the
  same binary would no longer surface in CI. The brief accepted this as
  "usually safer". It is recorded here so it is a decision, not an
  accident.
- **The next wall-time lever is compile caching, not the test runner.**
  toolkit and ms `test (…)` have no `Swatinem/rust-cache` and rebuild from
  scratch every run: about 400 s of each 456 s toolkit job. Also, toolkit's
  `release.yml` `test (ubuntu-latest)` re-runs the same 4085-test suite that
  `rust.yml` already runs, on the same push. Both are outside F-696's scope;
  worth a follow-up.
- `ubuntu-24.04-arm` is a free public-repo runner label. All four repos are
  public; I confirmed toolkit's visibility, and the other three ran there
  green. The four aarch64 jobs it affects are not required checks.
