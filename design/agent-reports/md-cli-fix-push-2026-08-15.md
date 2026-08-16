# Push report: descriptor-mnemonic `main` via `ci/staging` — 2026-08-15

**Repo:** `bg002h/descriptor-mnemonic` (local checkout `/scratch/code/shibboleth/descriptor-mnemonic`)
**Task:** push-only, no tag creation/push/deletion.

## Commit range pushed

`5a0a4f41017d71d47f70684c145702d4ca0c3aa9..89ab0f6201459ed062ca79d86c8880306a185262`

Three commits, confirmed via `git log --oneline origin/main..HEAD` before pushing:

- `89ab0f62` ci: build ci/** so a staged SHA can earn its required contexts
- `0c691869` fix(md-cli) fold: pin the origin-to-placeholder MAPPING, and echo the caller's path
- `11b01a9e` fix(md-cli): refuse a descriptor-style origin prefix instead of failing internally

Working tree was clean (`git status --porcelain` produced no output, exit 0) both before staging and immediately before the real push to `main`.

## Pre-push safety check

`git status --porcelain` — empty output, exit code `0`, checked unpiped both before and after the staging push. No tracked-file modifications at any point.

## `ci/**` trigger on its own introducing commit

**Worked on its own introducing commit.** This repo had never had a `ci/**` trigger before commit `89ab0f62` added it. Pushing `main` to `refs/heads/ci/staging` (a ref matching `ci/**`) produced a workflow run immediately — GitHub read the trigger from the content of the pushed ref (which includes `89ab0f62` itself), so the newly-added `ci/**` push trigger activated on the very commit that introduced it. No chicken-and-egg problem observed.

## Workflow run

- Run: **CI**, run ID `31917620755`
- URL: https://github.com/bg002h/descriptor-mnemonic/actions/runs/31917620755
- Triggered by: push to `refs/heads/ci/staging`
- `head_sha`: `89ab0f6201459ed062ca79d86c8880306a185262` (full 40-char SHA; matched exactly)
- Run-level status/conclusion (via `gh api repos/bg002h/descriptor-mnemonic/actions/runs/31917620755`): `completed` / `success`

### All jobs (via `gh api .../actions/runs/31917620755/jobs`)

```
success  cargo doc
success  cargo fmt
success  musl compile/test (x86_64-unknown-linux-musl)
success  cargo clippy
success  musl compile/test (aarch64-unknown-linux-musl)
success  cargo test (macos-latest)
success  cargo test (ubuntu-latest)
success  cargo test (windows-latest)
success  freebsd compile-gate (whole-crate)
```

### Required contexts (both confirmed `success`)

Fetched the actual required-status-check context names for `main` via `gh api repos/bg002h/descriptor-mnemonic/branches/main/protection --jq '.required_status_checks.contexts'`:

```json
["cargo test (ubuntu-latest)","cargo clippy"]
```

Both present in the job list above with conclusion `success`:

- `cargo test (ubuntu-latest)` — **success**
- `cargo clippy` — **success**

## Real push to `main`

Command: `git push origin main`

Verbatim output:

```
To github.com:bg002h/descriptor-mnemonic.git
   5a0a4f41..89ab0f62  main -> main
```

Exit code: `0`. **No "Bypassed rule violations" message appeared** — the push was a clean fast-forward, satisfied by the staged SHA's passing required contexts, not bypassed.

## `origin/main` moved — confirmed by fetch, not push output

```
git fetch origin main
git rev-parse origin/main
```

→ `89ab0f6201459ed062ca79d86c8880306a185262`, matching local `HEAD` and the SHA that was staged and tested. Confirmed independently via `git ls-remote --heads origin`, which also lists `refs/heads/main` at that same SHA.

## `ci/staging` ref cleanup

```
git push origin --delete ci/staging
```

Verbatim output:

```
To github.com:bg002h/descriptor-mnemonic.git
 - [deleted]           ci/staging
```

Confirmed absent from `git ls-remote --heads origin` (not listed among remote branches post-delete).

## Tag check

**No tag was created, pushed, or deleted.** `git tag -l` (local) and `git ls-remote --tags origin` (remote) were compared — identical tag sets, none pointing at `89ab0f6201459ed062ca79d86c8880306a185262`. The pinned oracle tag `md-cli-v0.13.0` remains at `5a0a4f41017d71d47f70684c145702d4ca0c3aa9`, unchanged, as expected (these commits are error-message/test changes only, no encoding-behaviour change, no re-tag implied per task instructions).

## Summary

| Check | Result |
|---|---|
| Tracked files clean before push | Yes (`git status --porcelain` empty, unpiped) |
| `ci/**` trigger fired on introducing commit | Yes, first time this repo has had one |
| Staged run found for full 40-char SHA | Yes, run `31917620755` |
| `cargo test (ubuntu-latest)` | success |
| `cargo clippy` | success |
| All other jobs | success (9/9) |
| Real push to `main` | fast-forward `5a0a4f41..89ab0f62`, no bypass message |
| `origin/main` confirmed moved (via fetch) | Yes, to `89ab0f6201459ed062ca79d86c8880306a185262` |
| `ci/staging` ref deleted | Yes |
| Tag created/pushed/deleted | No — none |
