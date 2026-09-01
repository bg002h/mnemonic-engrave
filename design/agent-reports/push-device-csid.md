# Push report — device-csid session (2026-09-01)

Two-repo push, seedhammer then mnemonic-engrave, per controller freeze on both.

## 1 — seedhammer (unprotected, direct push)

- Branch: `main`
- SHA: `169073c31a64e20a57a8b8739cc15e35ffc12571`
- Push: `git push origin main` — `2337ed3..169073c main -> main`, no errors.
- CI: run `33531184481` (Build image) and `33531184573` (Test) queued
  immediately on push; not watched to completion per brief (unprotected repo,
  direct push accepted as precedent).
- `git rev-parse origin/main` after push: `169073c31a64e20a57a8b8739cc15e35ffc12571`
  — matches tip.

## 2 — mnemonic-engrave (protected `master`, staging ritual)

- Branch: `master`
- SHA: `006f23118f4f819826f6669bfc2b1203b70c0a4a`
- Staging push: `git push origin master:refs/heads/ci/staging` — new branch
  `ci/staging` created.
- Run found for exact SHA on `--branch ci/staging`: id `33531207679`
  (workflow `release`), `headSha` = `006f23118f4f819826f6669bfc2b1203b70c0a4a`.
- Watched to completion (`gh run watch 33531207679 --exit-status`).
- Final `gh run view --json conclusion,jobs`:
  - Overall conclusion: `success`
  - `build me-preview (all targets)`: success
  - `build me (linux-aarch64)`: success
  - `test (rust + go)`: success (the required context)
  - `build me (macos-x86_64)`: success
  - `build me (windows-x86_64)`: success
  - `build me (macos-aarch64)`: success
  - `build me (linux-x86_64)`: success
  - `assemble + sign + release`: skipped (expected — tag-gated, `ci/staging`
    is not a tag ref)
- Master push: `git push origin master` — `3d570c3..006f231 master -> master`.
  No "Bypassed rule violations" line printed — the required `test (rust + go)`
  check on the exact SHA satisfied branch protection rather than bypassing it.
- Cleanup: `git push origin --delete ci/staging` — deleted.
- `git rev-parse origin/master` after push: `006f23118f4f819826f6669bfc2b1203b70c0a4a`
  — matches tip.

## Outcome

Both repos landed at their target tips with no force-push, no
`enforce_admins` change, and no commits made during the freeze window.
