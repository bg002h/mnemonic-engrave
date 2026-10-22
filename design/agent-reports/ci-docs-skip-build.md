
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
