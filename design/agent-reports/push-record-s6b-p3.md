# Push record — S6b P3 (design docs) — ci/staging ritual

## SHA staged

Full 40-character SHA (read via `git rev-parse HEAD` before any action):

```
10798355af264d203d219c9d9398e73558c842f5
```

Commits included (local `master` was 4 ahead of `origin/master` at the time):

```
1079835 s6b: adjudicate P3's unspecified third change -- ACCEPTED, the spec had fixed one site of a two-site claim
c3cab48 reports: commit the P3 implementation report, verbatim
042d898 s6b: correct the band budget -- 42 was the WRONG BOUND, and R-H's string would have engraved into the screw-hole zone
c5c169f reports: the P2 push record -- check SATISFIED
```

(base: `75f9ca8b6e5adb02d9ac0dc336e0b4bc38e20ad0`, prior `master` tip)

## Staging push

```
$ git push origin master:refs/heads/ci/staging
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      master -> ci/staging
```

## Triggered run

- Run id: `32093352713`
- Workflow: `release`
- URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/32093352713
- Trigger: push to `ci/staging`
- headSha: `10798355af264d203d219c9d9398e73558c842f5` (confirmed matches staged SHA)

`gh run watch 32093352713 --repo bg002h/mnemonic-engrave --exit-status` was run to completion (blocking) and returned without error.

## Per-job conclusions (verbatim, `gh run view --json status,conclusion,jobs`)

```json
{
  "status": "completed",
  "conclusion": "success",
  "headSha": "10798355af264d203d219c9d9398e73558c842f5",
  "url": "https://github.com/bg002h/mnemonic-engrave/actions/runs/32093352713",
  "jobs": [
    {"name": "build me-preview (all targets)", "status": "completed", "conclusion": "success"},
    {"name": "build me (macos-x86_64)",        "status": "completed", "conclusion": "success"},
    {"name": "test (rust + go)",               "status": "completed", "conclusion": "success"},
    {"name": "build me (linux-x86_64)",        "status": "completed", "conclusion": "success"},
    {"name": "build me (linux-aarch64)",       "status": "completed", "conclusion": "success"},
    {"name": "build me (windows-x86_64)",      "status": "completed", "conclusion": "success"},
    {"name": "build me (macos-aarch64)",       "status": "completed", "conclusion": "success"},
    {"name": "assemble + sign + release",      "status": "completed", "conclusion": "skipped"}
  ]
}
```

`assemble + sign + release` reported `skipped` — confirms `assemble + sign + release` is tag-gated (`refs/tags/v*`) and a `ci/**` push cannot sign or publish, as documented in `.github/workflows/release.yml`.

## Check-runs on the SHA, filtered to `status == completed` (per constraint 6)

`gh api repos/bg002h/mnemonic-engrave/commits/10798355af264d203d219c9d9398e73558c842f5/check-runs`:

```json
{"conclusion":"skipped","name":"assemble + sign + release","status":"completed"}
{"conclusion":"success","name":"build me (macos-aarch64)","status":"completed"}
{"conclusion":"success","name":"build me (windows-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me (linux-aarch64)","status":"completed"}
{"conclusion":"success","name":"build me (linux-x86_64)","status":"completed"}
{"conclusion":"success","name":"test (rust + go)","status":"completed"}
{"conclusion":"success","name":"build me (macos-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me-preview (all targets)","status":"completed"}
```

Required context `test (rust + go)` — `completed` / `success`.

## Final push (master)

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   75f9ca8..1079835  master -> master
```

No "Bypassed rule violations" string appeared in the output — the rule was **satisfied**, not bypassed.

## Cleanup

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

## Positive control (per constraint 8)

`git ls-remote origin refs/heads/master refs/heads/ci/staging`:

```
10798355af264d203d219c9d9398e73558c842f5	refs/heads/master
```

`master` present at the pushed SHA; `ci/staging` absent from the same query output.

## Post-push local verification

```
$ git rev-parse HEAD
10798355af264d203d219c9d9398e73558c842f5
$ git rev-parse origin/master
10798355af264d203d219c9d9398e73558c842f5
```

Local `master` and `origin/master` match exactly — no drift during the window (master was frozen for the duration per constraint 2).

## Verdict

**SATISFIED**
