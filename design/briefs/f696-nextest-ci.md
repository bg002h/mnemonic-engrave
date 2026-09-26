# Brief — F-696: run CI test jobs with cargo nextest

Read F-696 in `/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md`.
Operator (2026-09-26): do it now. The goal is shorter CI wall time with
identical coverage.

## Scope

Every Rust test job in the CI of: mnemonic-toolkit (`rust.yml`; a green master
run took ~21 min), mnemonic-secret, descriptor-mnemonic, mnemonic-key,
mnemonic-engrave and mnemonic-gui. Required checks include `test (macos-latest)`
on toolkit and ms.

## Rules

- **Coverage must not drop.** nextest does NOT run doc-tests: add
  `cargo test --doc` wherever doc-tests exist. Keep every `--include-ignored` /
  env-gated leg (e.g. the toolkit's mlock `g2_*` / `g6` legs) exactly as
  today. For each job, compare the number of tests executed before and after:
  nextest's summary vs `cargo test`'s `test result:` lines. Explain any
  difference.
- **Required check names must not change** (branch protection matches job
  names), unless you list each change so the controller updates protection.
- Install nextest with a pinned, checksum-verified method (e.g.
  `taiki-e/install-action` pinned to a SHA, or the release tarball with a
  sha256 check); keep the toolchain pin.
- Tests that assume serial execution or shared state (see memory "nextest
  isolation hides shared-state bugs"): nextest runs each test in its own
  process, which is usually *safer*, but check for tests that write fixed
  paths or ports. Run the full suite locally with nextest in each repo first.
- Measure: for each job, the wall time of the last green run before and the
  first green run after. Report the table.

## Where

Branch `f696-nextest` per repo, in worktrees under
`/scratch/code/shibboleth/<prefix>-worktrees/f696`. Trigger CI on each branch
in PARALLEL and wait on run ids (never `pgrep -f`). Don't push default branches,
merge or tag. F-695 rule: no `producer | grep -q` under pipefail.

## Deliverable

Commits ending with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
**As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f696-impl.md`**:
per repo the test-count comparison, the timing table, any renamed jobs, and
CI run ids. Return a short summary plus the path.
