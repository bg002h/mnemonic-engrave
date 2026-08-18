# Push record — S6b P1 (design docs, no code)

## Staged SHA

Full 40-char SHA (read via `git rev-parse HEAD` before any action):

```
989b6f4e2934b8731955a62e91f0df438ffc7db7
```

Subject: `s6b: adjudicate P1's GATE 3.1 deviation -- the implementation is RIGHT and the spec is amended to it`

Pre-push state: branch `master`, working tree clean, 3 commits ahead of `origin/master` (`2fea99b..989b6f4`).

## Ritual

```sh
git push origin master:refs/heads/ci/staging
```
Output:
```
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      master -> ci/staging
```

## Run

- Workflow: `release` (`.github/workflows/release.yml`, triggers on `ci/**`)
- Run id: `32087456942`
- URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/32087456942
- headSha confirmed via `gh run list --json headSha`: `989b6f4e2934b8731955a62e91f0df438ffc7db7` (matches staged SHA)
- `gh run watch 32087456942 --repo bg002h/mnemonic-engrave --exit-status` was launched; it exceeded the 120s foreground window and was moved to background. Rather than trust an unconfirmed background result, a second, explicit foreground poll (`gh run view --json status,conclusion`, 15s interval, under a 590s timeout) was run to block on completion directly. Both converged: run `status=completed`, `conclusion=success`, and the backgrounded `gh run watch` also completed with exit code 0.

### Per-job conclusions (`gh run view 32087456942 --json status,conclusion,jobs`), all `status: completed`

| job | conclusion |
| --- | --- |
| build me-preview (all targets) | success |
| build me (macos-x86_64) | success |
| **test (rust + go)** | **success** |
| build me (linux-aarch64) | success |
| build me (linux-x86_64) | success |
| build me (windows-x86_64) | success |
| build me (macos-aarch64) | success |
| assemble + sign + release | **skipped** |

`assemble + sign + release` reporting `skipped` confirms the tag-gate (`refs/tags/v*`) held — this `ci/**` push did not sign or publish.

### Check-runs on the staged SHA (filtered to `status == completed`)

```
{"conclusion":"skipped","name":"assemble + sign + release","status":"completed"}
{"conclusion":"success","name":"build me (macos-aarch64)","status":"completed"}
{"conclusion":"success","name":"build me (windows-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me (linux-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me (linux-aarch64)","status":"completed"}
{"conclusion":"success","name":"test (rust + go)","status":"completed"}
{"conclusion":"success","name":"build me (macos-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me-preview (all targets)","status":"completed"}
```

Required context `test (rust + go)` is `success`, `status: completed` — the SHA had earned the check before the final push.

## Final push

```sh
git push origin master
```
Output (verbatim, complete):
```
To github.com:bg002h/mnemonic-engrave.git
   2fea99b..989b6f4  master -> master
```

**"Bypassed rule violations" did NOT appear.** Exit code 0.

```sh
git push origin --delete ci/staging
```
Output:
```
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

## Positive control

`git ls-remote origin` filtered to `refs/heads/(master|ci/staging)$`, single invocation:

```
989b6f4e2934b8731955a62e91f0df438ffc7db7	refs/heads/master
```

`ci/staging` is absent; `master` is present at the staged SHA, in the same `ls-remote` output. No stale `ci/staging` ref remains.

## Post-push confirmation

`git rev-parse HEAD` == `git rev-parse origin/master` == `989b6f4e2934b8731955a62e91f0df438ffc7db7` — `master` was not advanced by the controller during the run; local and remote tips match the staged SHA exactly.

## Verdict

**SATISFIED**
