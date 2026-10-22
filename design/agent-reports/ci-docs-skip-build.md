
# Re-run after: ci: skip a docs-only push to a PR that already passed (rule A')

PR #20. Applies ci-skip-constellation SPEC.md (GREEN at e8f831d), Part A.

## Change

```
 .github/workflows/release.yml |   6 ++
 ci/code-docs-check.sh         | 165 ++++++++++++++++++++++++++++++++++++++++++
 ci/code-docs-check.test.sh    | 128 ++++++++++++++++++++++++++++++++
 ci/code-docs-exempt.txt       |  28 +++++++
 ci/docs-only-paths.txt        |   8 ++
 ci/docs-only.sh               | 158 +++++++++++++++++++++++++++++-----------
 ci/docs-only.test.sh          | 103 ++++++++++++++++++++++++--
 ci/shared-ci.sha256           |   4 +
 8 files changed, 551 insertions(+), 49 deletions(-)
```

## Gate

`gate-public: 0 failure(s)`

## Local checks

```
ci/docs-only.sh: OK
ci/docs-only.test.sh: OK
ci/code-docs-check.sh: OK
ci/code-docs-check.test.sh: OK
docs-only.test.sh: 73 passed, 0 failed
code-docs-check.test.sh: 41 passed, 0 failed
code-docs check: ok (163 files scanned, 2329 docs files)
```

## L1 (code head 370fa5066020e18b67a6f1115f6f4d7fad48e135)

```
run 39724127812    release                        pull_request   success    ok
  release/build me-preview (all targets)           success    ok
  release/build me (macos-aarch64)                 success    ok
  release/build me (windows-x86_64)                success    ok
  release/build me (linux-aarch64)                 success    ok
  release/build me (macos-x86_64)                  success    ok
  release/picotool argv probe                      success    ok
  release/test (rust + go)                         success    ok
  release/build me (linux-x86_64)                  success    ok
  release/assemble + sign + release                skipped    ok (allowed skip)
pr-checks: PASS (mnemonic-engrave 370fa5066020e18b67a6f1115f6f4d7fad48e135)
```

## L2 (docs head 3dbd16873c81ca5070c5b1e80421f9bc20229f64, previous head 370fa5066020e18b67a6f1115f6f4d7fad48e135)

- run 39724997419 job 119646759705 `release/test (rust + go)`: `docs-only: all 1 files changed since the previous PR head 370fa5066020e18b67a6f1115f6f4d7fad48e135 are documentation, the merged base d3f6886ef0808d53c712ecaded8fb068f6dd6214 is an ancestor of it, and 'test (rust + go)' passed on it; skipping the guarded steps`
- run 39724997419 job 119646759738 `release/picotool argv probe`: `docs-only: all 1 files changed since the previous PR head 370fa5066020e18b67a6f1115f6f4d7fad48e135 are documentation, the merged base d3f6886ef0808d53c712ecaded8fb068f6dd6214 is an ancestor of it, and 'picotool argv probe' passed on it; skipping the guarded steps`
