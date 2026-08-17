# S6a push 2026-08-16 — R17 GREEN, staging-ref ritual

## What was pushed

Nine commits closing the S6a R0 review gate on `master`: three persisted
review reports (R15, R16, R17), three folds responding to them, a process
change recording a reviewer-tiering directive, and a continuity update.
Documentation and design only — no code in `crates/` or `gui/` changed.

Commit range: `aa2a32e..589ba06`

```
589ba06 continuity: the R0 gate is CLOSED -- GREEN at R17, next action is implementation
b324023 plan: fold R17's Nit -- cite the exit by the same convention 4.7b uses
3172b69 reports: persist the S6a R17 review -- GREEN, 0C/0I, the R0 loop CLOSES
6a2198f plan: fold R16 -- a benign exit reaches the zero cell FROM INSIDE the flow
c3e9705 reports: persist the S6a R16 review -- RED, 0C/1I, MECHANICAL, again against the controller's own text
4c40973 plan: fold R15 -- the mapping is a table of RECORD WRITES, never of statuses
5588e98 reports: persist the S6a R15 review -- RED, 0C/1I, MECHANICAL, against the controller's own fold
d0d397b process: fable is no longer a reviewer tier -- opus is the top of the ladder
4f40f1f plan: step 1's gate prose, its acceptance criteria, and the fall-through exit
```

## Staged SHA

`589ba0694f6ff86aab061470c99fb9d142d6b6df` (full 40 chars) — this was
`master`'s tip for the entire window; verified unmoved immediately before
the final push to `master`.

## Ritual steps and results

1. `git push origin master:refs/heads/ci/staging` — staged the SHA above on
   a `ci/**` ref (builds it, cannot sign/publish).
2. Run found: **run id `31989579092`**,
   `https://github.com/bg002h/mnemonic-engrave/actions/runs/31989579092`,
   `headBranch: ci/staging`, `headSha: 589ba0694f6ff86aab061470c99fb9d142d6b6df`
   (exact match), triggered by the `push` event.
3. `gh run watch 31989579092 --repo bg002h/mnemonic-engrave --exit-status`
   → exit code 0.

### Per-job conclusions (via `gh run view --json jobs`)

| job | status | conclusion |
| --- | --- | --- |
| build me-preview (all targets) | completed | success |
| build me (macos-x86_64) | completed | success |
| **test (rust + go)** | completed | **success** (2m18s) |
| build me (windows-x86_64) | completed | success |
| build me (linux-x86_64) | completed | success |
| build me (linux-aarch64) | completed | success |
| build me (macos-aarch64) | completed | success |
| assemble + sign + release | completed | **skipped** |

The required context `test (rust + go)` passed. `assemble + sign + release`
is `skipped`, as expected on a `ci/**` ref (gated on `refs/tags/v*`) —
confirmed nothing in the run signed or published anything.

4. Verified `master`'s local tip was still `589ba0694f6ff86aab061470c99fb9d142d6b6df`
   immediately before the final push (`git fetch origin master` +
   `git rev-parse HEAD`); `origin/master` was at `aa2a32e...` (the prior tip)
   at that point, confirming no one had advanced it.
5. `git push origin master` → output:
   ```
   To github.com:bg002h/mnemonic-engrave.git
      aa2a32e..589ba06  master -> master
   ```
   **No `remote: Bypassed rule violations` message** — the required check
   was SATISFIED by the SHA's own passing `ci/staging` run, not bypassed.
6. `git push origin --delete ci/staging` → `- [deleted] ci/staging`.
   **Positive control:** `git ls-remote --heads origin | grep -i staging`
   returned nothing (grep exit code 1); full remote heads list afterward
   showed only `refs/heads/master` (at `589ba06...`) and
   `refs/heads/sysw-container` — `ci/staging` is confirmed absent.

## Outcome

GREEN. Required check `test (rust + go)` passed for the exact pushed SHA;
`assemble + sign + release` correctly skipped; final push to `master`
carried no bypass message; `ci/staging` cleanup verified by positive
control, not by trusting the delete command's exit code.
