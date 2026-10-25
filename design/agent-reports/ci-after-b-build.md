
# Re-run after: ci: guard the release build jobs with ci/docs-only.sh

PR #21. Copies ci-skip-constellation reference/after-b/B-en (SPEC-after-b.md).

## Change

```
 .github/workflows/release.yml | 44 +++++++++++++++++++++++++++++++++++++++----
 1 file changed, 40 insertions(+), 4 deletions(-)
```

## Gate

`make-after-b-reference --check: PASS (12 files)`

## Local checks

```
ci/docs-only.sh: OK
ci/docs-only.test.sh: OK
ci/code-docs-check.sh: OK
ci/code-docs-check.test.sh: OK
docs-only.test.sh: 73 passed, 0 failed
code-docs-check.test.sh: 41 passed, 0 failed
code-docs check: ok (163 files scanned, 2331 docs files)
PASS B-en .github/workflows/release.yml: exactly the §4.2 edits to go-build, rust-build
after-b-check B-en: PASS
```

## L1 (code head a7556d1423d8cb2272c323969982e2b01a52b55c)

```
run 40225878140    release                        pull_request   success    ok
  release/picotool argv probe                      success    ok
  release/build me (linux-aarch64)                 success    ok
  release/test (rust + go)                         success    ok
  release/build me (windows-x86_64)                success    ok
  release/build me (linux-x86_64)                  success    ok
  release/build me (macos-aarch64)                 success    ok
  release/build me (macos-x86_64)                  success    ok
  release/build me-preview (all targets)           success    ok
  release/assemble + sign + release                skipped    ok (allowed skip)
pr-checks: PASS (mnemonic-engrave a7556d1423d8cb2272c323969982e2b01a52b55c)
```

## L2 (docs head e4dbb65a6fbcffaf36cb2da761291ef29c590454, previous head a7556d1423d8cb2272c323969982e2b01a52b55c)

- run 40226736800 job 121269584178 `release/test (rust + go)`: `docs-only: all 1 files changed since the previous PR head a7556d1423d8cb2272c323969982e2b01a52b55c are documentation, the merged base cc9c030666a5e08228e9ac0763da2e03a07f2993 is an ancestor of it, and 'test (rust + go)' passed on it; skipping the guarded steps`
- run 40226736800 job 121269584346 `release/build me (macos-aarch64)`: `docs-only: all 1 files changed since the previous PR head a7556d1423d8cb2272c323969982e2b01a52b55c are documentation, the merged base cc9c030666a5e08228e9ac0763da2e03a07f2993 is an ancestor of it, and 'build me (macos-aarch64)' passed on it; skipping the guarded steps`
- run 40226736800 job 121269584357 `release/build me (linux-aarch64)`: `docs-only: all 1 files changed since the previous PR head a7556d1423d8cb2272c323969982e2b01a52b55c are documentation, the merged base cc9c030666a5e08228e9ac0763da2e03a07f2993 is an ancestor of it, and 'build me (linux-aarch64)' passed on it; skipping the guarded steps`
- run 40226736800 job 121269584366 `release/build me (linux-x86_64)`: `docs-only: all 1 files changed since the previous PR head a7556d1423d8cb2272c323969982e2b01a52b55c are documentation, the merged base cc9c030666a5e08228e9ac0763da2e03a07f2993 is an ancestor of it, and 'build me (linux-x86_64)' passed on it; skipping the guarded steps`
- run 40226736800 job 121269584371 `release/build me (windows-x86_64)`: `docs-only: all 1 files changed since the previous PR head a7556d1423d8cb2272c323969982e2b01a52b55c are documentation, the merged base cc9c030666a5e08228e9ac0763da2e03a07f2993 is an ancestor of it, and 'build me (windows-x86_64)' passed on it; skipping the guarded steps`
- run 40226736800 job 121269584412 `release/build me (macos-x86_64)`: `docs-only: all 1 files changed since the previous PR head a7556d1423d8cb2272c323969982e2b01a52b55c are documentation, the merged base cc9c030666a5e08228e9ac0763da2e03a07f2993 is an ancestor of it, and 'build me (macos-x86_64)' passed on it; skipping the guarded steps`
- run 40226736800 job 121269584420 `release/picotool argv probe`: `docs-only: all 1 files changed since the previous PR head a7556d1423d8cb2272c323969982e2b01a52b55c are documentation, the merged base cc9c030666a5e08228e9ac0763da2e03a07f2993 is an ancestor of it, and 'picotool argv probe' passed on it; skipping the guarded steps`
- run 40226736800 job 121269584547 `release/build me-preview (all targets)`: `docs-only: all 1 files changed since the previous PR head a7556d1423d8cb2272c323969982e2b01a52b55c are documentation, the merged base cc9c030666a5e08228e9ac0763da2e03a07f2993 is an ancestor of it, and 'build me-preview (all targets)' passed on it; skipping the guarded steps`
