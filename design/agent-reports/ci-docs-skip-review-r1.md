r1: 0C/0I/2M/3N

# L3 whole-diff review: mnemonic-engrave PR #20, round r1

Reviewer: independent adversarial review (Opus), 2026-10-21.
Subject: PR #20 (`feat/ci-docs-skip`), head 248cb00c, diff
`origin/master...248cb00c` (370fa506 CI change + 3dbd1687, 248cb00c report
commits), the PR description, and `.github/workflows/release.yml` as it will
be after the merge.

## Verdict

Safe and correct as merged. No Critical, no Important. I found no path where
an engrave job reports green on code that no passing full run covers, no
guarded step that reads a docs file, and no change to the master, tag or
dispatch behaviour beyond one extra unguarded step that is deterministic for
a given tree.

## What I checked, and how

### 1. `ci/docs-only-paths.txt` and `ci/code-docs-exempt.txt`

- Ran `bash ci/code-docs-check.sh` in the worktree: `ok (163 files scanned,
  2330 docs files)`. All four new `-` rules match exactly one tracked file
  each, and each basename is unique, so none is a dead or mistyped rule.
- The four protected files are named only in messages, docstrings or
  comments (`seal/mod.rs:732` label `spec: "SPEC_encrypted_payload_delivery.md"`,
  `flake.nix` assertMsg, `refugium-otp.sh:193,317` messages,
  `fake_picotool.py:12` docstring). Protecting rather than rewording is the
  conservative choice and is correct.
- Searched outside the check's own model for reads it cannot see:
  - Path pieces (`"design"`, `/design/`, `os.path.join(..., 'design', ...)`,
    `Path / "design"`), directory walks (`read_dir`, `os.walk`, `listdir`,
    `glob`, `find -name`, `filepath.Walk`, `ls-files`) and
    `include_str!`/`include_bytes!`/`CARGO_MANIFEST_DIR` across every tracked
    non-docs `.rs .go .py .sh .nix .toml .yml` and extensionless script.
    Every assembled-path reader of `design/` is an exempt script
    (acceptance-count.py, followups-status.sh, importability-matrix.py,
    mutation-run.py, shot-server-security-test.py, gen-tx-journey.sh,
    h[256]-plan-blocks-vs-tree.sh, plan-apply-gate-go.sh) or reads
    `design/hardware/` (`run-e2e-otp.sh:561`, `refugium-otp.sh:169`), which
    `- ^design/hardware/` protects. Rust tests read only `testdata/` and
    `tests/vectors/`; the two `read_dir` calls read a temp/user output dir.
  - Go: no `//go:embed` anywhere; `preview/` and `firmware/ndef-roundtrip/`
    are their own modules and cannot embed repo-root `design/`.
  - Cargo: workspace members are `crates/me-cli`, `crates/mnemonic-io-lib`;
    `design/measurements/mt-size-probe` is excluded, so fmt/clippy/nextest
    never touch it. No `#![doc = include_str!(...README.md)]`.
  - Unscanned file types (`.json .txt .js .html .patch .tsv`, extensionless):
    the docs names they carry (`descriptor_seam_vectors.json` "source",
    `seal_vectors.json` "spec", the patch header) are labels no test opens.
- Exempt list: 1 + 18 lines covering 21 scripts, matching SPEC §3's count.
  For each, grep of every tracked non-design file: the only references are
  the script itself, `ci/code-docs-exempt.txt`, root/README prose, or other
  scripts no workflow runs (`descriptor-seam-vectors/gen.py` imports
  `rows.py`; `plan-build-gate-*.sh`/`plan-cite-gate.sh` name
  `plan-build-gate.sh`; `mutation-run-crashtest.py` names `mutation-run.py`).
  None is named by `release.yml` or by any script a guarded step runs
  (run-e2e-otp.sh, refugium-otp.sh, lib/otp-read.sh, fake_picotool.py,
  picotool-argv-probe.sh, seal-clear-test.sh, and the shellcheck list).
  `check-minisign-pin.sh` runs only in the unguarded step before the guard
  (it reads README.md, which is docs, but always runs). The exempt file's
  comments are accurate (`check-minisign-pin.sh` names
  `release-workflows/gen.py`, not the exempt `emit.py`).

### 2. The new step's placement and `if:`

- Inserted immediately after `Self-test the docs-only guard` in the `test`
  job, no `if:`, matching SPEC §2.3's row for engrave. Every step before the
  guard is unconditional, so it runs on every event, before `docs`.
- Tag and dispatch: it needs only the checkout (`git ls-files`, bash 4, GNU
  grep, `sha256sum`, a temp dir). `code-docs-check.test.sh` sets its own
  `GIT_CONFIG_GLOBAL` and identity, so a runner without git identity is
  fine. A tag runs the workflow at the tagged commit, so an older tag has
  no such step and a newer one has the files the step checks. The result is
  a function of the tree; the same SHA's master/PR run already ran it
  (the step is never docs-skipped), so a tag run cannot fail where its
  commit's earlier run passed. Cost in L2: about 2.4 s.

### 3. Master pushes, tag pushes, dispatches after the merge

- `docs-only.sh`'s `push` branch is a refactor into `docs_since` + `passed`
  with the same reference commits and the same full-run reasons; tags
  (`push to refs/tags/...`) and `workflow_dispatch` (`event ...`) still
  force a full run (self-test rows "tag push", "workflow_dispatch" pass in
  the L2 log).
- The merge's own master push diffs against the previous master tip, which
  includes `ci/` and `release.yml`, so it is a full run.
- `assemble` still `needs: [test, go-build, rust-build, picotool-probe]` and
  keeps its `if:`; the required context name `test (rust + go)` and both
  `CHECK_NAME`s are unchanged. Both guarded jobs already have
  `contents: read` (compare API) and `checks: read`.

### 4. L2 evidence (run 39724997419, pull_request, head 3dbd1687)

- Step conclusions from the API: in `test (rust + go)`, all 13 guarded steps
  are `skipped`; in `picotool argv probe`, all 5 are `skipped`. The full
  run log has no lines under any guarded step name and zero
  `cargo|nextest|nix build|Compiling` lines in either job. Job wall times
  19 s and 11 s.
- Both guard lines read "all 1 files changed since the previous PR head
  370fa506... are documentation, the merged base d3f6886... is an ancestor
  of it, and ... passed on it". The new step logged 4x `: OK`,
  `code-docs-check.test.sh: 41 passed, 0 failed`, and
  `code-docs check: ok (163 files scanned, 2330 docs files)`.
- Final head 248cb00c (run 39725480584) is green; 248cb00c touches only
  `design/agent-reports/ci-docs-skip-build.md`.

## Critical / Important

None.

## Minor (one line each)

- M1 (SPEC/reference, not live in engrave): extensionless scripts are not in the scan set; engrave has three (`scripts/sh2-flash`, `scripts/test/fake-picotool`, `scripts/sh-preview-basic`), none run by a guarded step today (`sh2-flash` is only read by `run-e2e-otp.sh` via sed, and is code, not docs).
- M2 (SPEC/reference, not live in engrave): a `//go:embed <docs path>` directive is a whole-line `//` comment, so the check skips it; engrave has none and its Go modules cannot reach `design/`, but §5 does not state this limit for repos whose Go module root can.

## Nits (one line each)

- N1 (engrave `release.yml`, follow-up since SPEC §3 forbids other edits): the `docs-only verdict` echo says "code tree identical to the green parent", which is wrong for an A′ skip (the green reference is the previous PR head).
- N2 (SPEC/reference): `.gitattributes` pins `eol=lf` only for `ci/docs-only*`; the new shared files and `shared-ci.sha256` are unpinned, so a Windows autocrlf checkout fails the sha256 check locally (no CI impact; all sites are Linux).
- N3 (engrave `release.yml`, pre-existing): the `docs-only verdict` step sits textually under the `me-preview` job's header comment, though it belongs to `test`.
