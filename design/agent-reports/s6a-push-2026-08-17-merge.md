# S6a push record — 2026-08-17 — merge of fork + design-repo cycles

## Repo 1 — fork: `/scratch/code/shibboleth/seedhammer`, branch `main`

- Before: `origin/main` = `b8a23bf3dcf45f0b996bedf8b17f7141f092d282`
- Local tip (unchanged throughout): `b1479a1b38f6b045d27443764c858906e4e6e122`
- Push: `git push origin main` — direct push, unprotected branch, no staging ritual (per instructions for this repo).
  ```
  To github.com:bg002h/seedhammer.git
     b8a23bf..b1479a1  main -> main
  ```
- After: `git fetch origin main && git rev-parse origin/main` = `b1479a1b38f6b045d27443764c858906e4e6e122`
- **Confirmed: `origin/main` == `b1479a1b38f6b045d27443764c858906e4e6e122`.**
- Result: **SUCCESS**

## Repo 2 — design repo: `/scratch/code/shibboleth/mnemonic-engrave`, branch `master`

- Before: `origin/master` = `589ba0694f6ff86aab061470c99fb9d142d6b6df`
- Local tip (unchanged throughout): `434d2f234cdce1a6c5a96b0a4e84d85ec4827d39`

### ci/staging ritual

1. `git rev-parse HEAD` confirmed `434d2f234cdce1a6c5a96b0a4e84d85ec4827d39` before staging.
2. `git push origin master:refs/heads/ci/staging` → `* [new branch] master -> ci/staging` (staged this exact SHA).
3. Matched the run to the SHA via `gh run list --repo bg002h/mnemonic-engrave --limit 5 --json databaseId,headSha,headBranch,status,conclusion,workflowName,createdAt`:
   - Run id **32012316190**, `headSha` = `434d2f234cdce1a6c5a96b0a4e84d85ec4827d39` (full 40-char match), `headBranch` = `ci/staging`, workflow `release`.
   - Run URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/32012316190
4. Watched to completion (`gh run watch 32012316190 --repo bg002h/mnemonic-engrave --exit-status`), then confirmed final state with `gh run view 32012316190 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha,jobs`.

**Run-level:** `status: completed`, `conclusion: success`, `headSha: 434d2f234cdce1a6c5a96b0a4e84d85ec4827d39` (confirmed exact match).

**Per-job conclusions (all `status: completed`):**

| job | conclusion |
| --- | --- |
| build me-preview (all targets) | success |
| build me (macos-aarch64) | success |
| build me (linux-x86_64) | success |
| build me (windows-x86_64) | success |
| **test (rust + go)** | **success** |
| build me (macos-x86_64) | success |
| build me (linux-aarch64) | success |
| assemble + sign + release | **skipped** |

`assemble + sign + release` reporting `skipped` is the expected/correct outcome for a `ci/**` push per `.github/workflows/release.yml` (gated on `refs/tags/v*`). Nothing signed, tagged, or published — confirmed. Rust/Go test suites were unaffected by the documentation-only changes, as expected.

### Final push to master

- Verified immediately before the final push that the local tip (`434d2f234cdce1a6c5a96b0a4e84d85ec4827d39`) still matched what was staged — it had not moved (controller held the freeze).
- `git push origin master`:
  ```
  To github.com:bg002h/mnemonic-engrave.git
     589ba06..434d2f2  master -> master
  ```
- **No `remote: Bypassed rule violations` message or any bypass text appeared.** Output was the plain two-line push summary shown above — the required-status-check was SATISFIED by the SHA earning it on `ci/staging`, not bypassed.

### ci/staging deletion — positive control

- `git push origin --delete ci/staging` → `- [deleted] ci/staging`
- Positive-control check: `git ls-remote --heads origin`
  ```
  434d2f234cdce1a6c5a96b0a4e84d85ec4827d39	refs/heads/master
  3b4b4ff37a08bb829878de54b83613267f0c273f	refs/heads/sysw-container
  ```
  `ci/staging` is absent; `master` is present at the expected SHA; a third, unrelated branch (`sysw-container`) is also listed, proving the query is not returning a silently-empty result.

### After

- `origin/master` = `434d2f234cdce1a6c5a96b0a4e84d85ec4827d39` (confirmed via `git fetch origin master && git rev-parse origin/master`, matches local tip).
- Result: **SUCCESS**

## Summary

| repo | before | after | result |
| --- | --- | --- | --- |
| seedhammer (fork), `main` | `b8a23bf3dcf45f0b996bedf8b17f7141f092d282` | `b1479a1b38f6b045d27443764c858906e4e6e122` | SUCCESS |
| mnemonic-engrave (design), `master` | `589ba0694f6ff86aab061470c99fb9d142d6b6df` | `434d2f234cdce1a6c5a96b0a4e84d85ec4827d39` | SUCCESS |

CI for repo 2 (run 32012316190): GREEN — `test (rust + go)` succeeded; `assemble + sign + release` correctly skipped (tag-gated, not triggered by `ci/**`). No bypass occurred; the required status check was satisfied honestly via the staging ritual.

## Follow-up push — continuity + push record

Two documentation-only commits (the S6a push record from the section above, plus a new continuity file), pushed via the `ci/staging` ritual. The fork `/scratch/code/shibboleth/seedhammer` was **not touched**, per instructions — already in sync from the merge above.

### Before

- `origin/master` = `434d2f234cdce1a6c5a96b0a4e84d85ec4827d39` (the SHA landed by the push recorded above).
- Local tip: `f207cc79392a2100130b1cf4a52912213ca9fb41`, two commits ahead of `origin/master`:
  - `edbd197` — reports: persist the S6a push record — both repos live, check SATISFIED
  - `f207cc7` — continuity: S6a is SHIPPED — next is S6b, then the flash
- Controller froze the tip for the whole window (per the freeze rule); no commits landed on `master` during this push.

### ci/staging ritual

1. `git rev-parse HEAD` confirmed `f207cc79392a2100130b1cf4a52912213ca9fb41` before staging.
2. `git push origin master:refs/heads/ci/staging` → `* [new branch] master -> ci/staging` (staged this exact SHA).
3. Matched the run to the SHA via `gh run list --repo bg002h/mnemonic-engrave --limit 5 --json databaseId,headSha,headBranch,status,conclusion,displayTitle,event`:
   - Run id **32012738104**, `headSha` = `f207cc79392a2100130b1cf4a52912213ca9fb41` (full 40-char match), `headBranch` = `ci/staging`, event `push`.
4. Watched to completion (`gh run watch 32012738104 --repo bg002h/mnemonic-engrave --exit-status`), then confirmed final state with `gh run view 32012738104 --repo bg002h/mnemonic-engrave --json status,conclusion,jobs`.

**Run-level:** `status: completed`, `conclusion: success`.

**Per-job conclusions (all `status: completed`):**

| job | conclusion |
| --- | --- |
| build me-preview (all targets) | success |
| build me (linux-aarch64) | success |
| build me (macos-aarch64) | success |
| build me (windows-x86_64) | success |
| build me (linux-x86_64) | success |
| build me (macos-x86_64) | success |
| **test (rust + go)** | **success** |
| assemble + sign + release | **skipped** |

`assemble + sign + release` reporting `skipped` is the expected/correct outcome for a `ci/**` push (tag-gated, not triggered here). Nothing signed, tagged, or published. Documentation-only commits — no `crates/` or Go source changes — and the full build/test surface stayed green, as expected.

### Final push to master

- Verified immediately before the final push that the local tip (`f207cc79392a2100130b1cf4a52912213ca9fb41`) still matched `master`'s recorded tip and had not moved — controller held the freeze for the whole window.
- `git push origin master`:
  ```
  To github.com:bg002h/mnemonic-engrave.git
     434d2f2..f207cc7  master -> master
  ```
- **No `remote: Bypassed rule violations` message or any bypass text appeared.** Plain two-line push summary — the required-status-check was SATISFIED by the SHA earning it on `ci/staging`, not bypassed.

### ci/staging deletion — positive control

- `git push origin --delete ci/staging` → `- [deleted] ci/staging`
- Positive-control check: `git ls-remote --heads origin`
  ```
  f207cc79392a2100130b1cf4a52912213ca9fb41	refs/heads/master
  3b4b4ff37a08bb829878de54b83613267f0c273f	refs/heads/sysw-container
  ```
  `ci/staging` is absent; `master` is present at the expected new SHA; the unrelated `sysw-container` branch is also listed, proving the query is not returning a silently-empty result.

### After

- `origin/master` = `f207cc79392a2100130b1cf4a52912213ca9fb41` (confirmed via the `ls-remote` positive control above, matches local tip).
- Result: **SUCCESS**

### Summary

| repo | before | after | result |
| --- | --- | --- | --- |
| mnemonic-engrave (design), `master` | `434d2f234cdce1a6c5a96b0a4e84d85ec4827d39` | `f207cc79392a2100130b1cf4a52912213ca9fb41` | SUCCESS |

CI (run 32012738104): GREEN — `test (rust + go)` succeeded; `assemble + sign + release` correctly skipped. No bypass occurred; the required status check was satisfied honestly via the staging ritual. Fork `seedhammer` untouched, as instructed.
