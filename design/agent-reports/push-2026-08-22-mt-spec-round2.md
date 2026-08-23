# Push report — 2026-08-22 — mt spec round 2

## Task

Push `master` for `bg002h/mnemonic-engrave` from local `HEAD ba8fa578de1f863de5f4cac09c7d115a8a1d7bb7`
(4 ahead of `origin/master ed1c52287d61437291fa769d543ced23ee333aa7`, 0 behind) using the
repo's required `ci/staging` gating ritual. Fast-forward only; no force-push. Master was
frozen by the controller for the whole window (tip did not move).

## Pre-flight (read-only, confirming already-established facts)

```
$ git rev-parse HEAD
ba8fa578de1f863de5f4cac09c7d115a8a1d7bb7
$ git status --short
(empty)
$ git ls-remote origin master
ed1c52287d61437291fa769d543ced23ee333aa7  refs/heads/master
$ git log -1 --format='%H %s' ba8fa578de1f863de5f4cac09c7d115a8a1d7bb7
ba8fa578de1f863de5f4cac09c7d115a8a1d7bb7 spec: fix open-question numbering so the §10.6 cross-reference resolves
```

Confirmed clean tree, confirmed HEAD/origin match the brief exactly.

## Commands run, in order

1. `git push origin master:refs/heads/ci/staging`
   → `* [new branch] master -> ci/staging` (also printed the routine "Create a pull
   request for 'ci/staging'" hint, which is not an error).

2. `gh run list --repo bg002h/mnemonic-engrave --limit 8`
   → top row: `queued  ...  spec: fix open-question numbering so the §10.6 cross-reference resolves  release  ci/staging  push  32612159067`
   — commit message matches `ba8fa57` exactly, confirming the correct run was targeted.

3. `gh run watch 32612159067 --repo bg002h/mnemonic-engrave --exit-status`
   → watched to completion, `EXIT: 0`.

4. `gh run view 32612159067 --repo bg002h/mnemonic-engrave --json headSha,status,conclusion,jobs`
   (machine-checked confirmation of per-job conclusions, not just the watch TUI):

   ```json
   {
     "headSha": "ba8fa578de1f863de5f4cac09c7d115a8a1d7bb7",
     "status": "completed",
     "conclusion": "success",
     "jobs": [
       {"name": "build me-preview (all targets)", "status": "completed", "conclusion": "success"},
       {"name": "build me (macos-aarch64)",       "status": "completed", "conclusion": "success"},
       {"name": "build me (linux-aarch64)",       "status": "completed", "conclusion": "success"},
       {"name": "build me (linux-x86_64)",        "status": "completed", "conclusion": "success"},
       {"name": "build me (macos-x86_64)",        "status": "completed", "conclusion": "success"},
       {"name": "test (rust + go)",               "status": "completed", "conclusion": "success"},
       {"name": "build me (windows-x86_64)",      "status": "completed", "conclusion": "success"},
       {"name": "assemble + sign + release",      "status": "completed", "conclusion": "skipped"}
     ]
   }
   ```

   `headSha` matches `ba8fa578de1f863de5f4cac09c7d115a8a1d7bb7` exactly. `test (rust + go)`
   (the required check) = **success**. `assemble + sign + release` = **skipped**, as
   expected for a `ci/**` push (that job is gated on `refs/tags/v*`) — matches the brief's
   stated expectation; nothing anomalous, no stop condition triggered.

5. `git push origin master`
   → `ed1c522..ba8fa57  master -> master`
   → **verbatim full output, nothing else printed**:
   ```
   To github.com:bg002h/mnemonic-engrave.git
      ed1c522..ba8fa57  master -> master
   ```
   The string **"Bypassed rule violations" did NOT appear**. Clean fast-forward, check
   satisfied.

6. `git ls-remote origin master`
   → `ba8fa578de1f863de5f4cac09c7d115a8a1d7bb7  refs/heads/master` — confirms the push landed.

7. `git push origin --delete ci/staging`
   → `- [deleted]  ci/staging` — staging ref removed.

## Outcome

| Check | Result |
| --- | --- |
| Gating run (`ci/staging`, id `32612159067`) headSha | `ba8fa578de1f863de5f4cac09c7d115a8a1d7bb7` (exact match) |
| `test (rust + go)` conclusion | success |
| `assemble + sign + release` conclusion | skipped (expected, gated on tag push) |
| All other jobs (5× `build me *`, `build me-preview`) | success |
| "Bypassed rule violations" in final push output | **NOT present** |
| Final `origin/master` SHA | `ba8fa578de1f863de5f4cac09c7d115a8a1d7bb7` |
| `ci/staging` deleted | yes |
| Tip stayed at `ba8fa57` for the whole window | yes — confirmed by pre-flight check and by the gating run's own `headSha` |
| Force-push used | no |
| Hard-stop conditions triggered | none |

**Result: SUCCESS.** `master` on `bg002h/mnemonic-engrave` now points to
`ba8fa578de1f863de5f4cac09c7d115a8a1d7bb7`, earned via the required-check staging ritual,
fast-forward only, no bypass.
