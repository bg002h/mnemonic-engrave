# Push record — S6b P5b (design-only push, ci/staging ritual)

## SHA staged

`aa23ca464bcd195d6be7561ed6097e029dde212e` (read via `git rev-parse HEAD` before any push; `git status --short` was clean).

Commits carried by this push (`8ebda45..aa23ca4`):

```
aa23ca4 s6b: record R-N -- the preloaded footer is "POLICY <hex>  DERIVED", option C
227d60f reports: commit the P5b implementation report, verbatim
fdaf01f reports: the P5 push record -- check SATISFIED
8ebda45d14394ff3183516e7fd2820cb33d7902b  (previous origin/master tip)
```

Design documents only — the P5 push record, the P5b implementation report, and ruling R-N. No fork code touched.

## Ritual steps executed

1. `git push origin master:refs/heads/ci/staging` — created branch `ci/staging` at `aa23ca464bcd195d6be7561ed6097e029dde212e`.
2. Located the triggered run via `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging` — matched on `headSha == aa23ca464bcd195d6be7561ed6097e029dde212e`.
3. `gh run watch 32099764466 --repo bg002h/mnemonic-engrave --exit-status` — blocked until completion, **exit code 0**.
4. Re-verified local `master` tip unchanged (`git rev-parse HEAD` → still `aa23ca464bcd195d6be7561ed6097e029dde212e`) immediately before the final push — master stayed frozen for the whole window.
5. `git push origin master` — fast-forward `8ebda45..aa23ca4`.
6. `git push origin --delete ci/staging`.
7. Positive control: `git ls-remote origin refs/heads/master refs/heads/ci/staging`.

## Run id and URL

- Run id: `32099764466`
- URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/32099764466
- Workflow: `release`, triggered by the `ci/staging` push, head SHA `aa23ca464bcd195d6be7561ed6097e029dde212e`.

## Per-job conclusions (verbatim, via `gh api repos/bg002h/mnemonic-engrave/actions/runs/32099764466/jobs`)

```json
{"name":"build me (linux-aarch64)","status":"completed","conclusion":"success","started_at":"2026-08-18T04:36:30Z","completed_at":"2026-08-18T04:38:16Z"}
{"name":"build me (macos-aarch64)","status":"completed","conclusion":"success","started_at":"2026-08-18T04:36:31Z","completed_at":"2026-08-18T04:37:52Z"}
{"name":"test (rust + go)","status":"completed","conclusion":"success","started_at":"2026-08-18T04:36:31Z","completed_at":"2026-08-18T04:38:50Z"}
{"name":"build me (windows-x86_64)","status":"completed","conclusion":"success","started_at":"2026-08-18T04:36:30Z","completed_at":"2026-08-18T04:38:30Z"}
{"name":"build me-preview (all targets)","status":"completed","conclusion":"success","started_at":"2026-08-18T04:36:31Z","completed_at":"2026-08-18T04:37:14Z"}
{"name":"build me (linux-x86_64)","status":"completed","conclusion":"success","started_at":"2026-08-18T04:36:31Z","completed_at":"2026-08-18T04:37:33Z"}
{"name":"build me (macos-x86_64)","status":"completed","conclusion":"success","started_at":"2026-08-18T04:36:31Z","completed_at":"2026-08-18T04:37:30Z"}
{"name":"assemble + sign + release","status":"completed","conclusion":"skipped","started_at":"2026-08-18T04:38:51Z","completed_at":"2026-08-18T04:38:51Z"}
```

`test (rust + go)` (the branch-protection-required context) concluded **success**. `assemble + sign + release` concluded **skipped**, confirming the tag-gate (`refs/tags/v*`) held — the `ci/staging` push could not sign or publish.

## Commit check-runs, filtered to `status == completed` (via `gh api repos/bg002h/mnemonic-engrave/commits/aa23ca464.../check-runs`)

```json
{"name":"assemble + sign + release","status":"completed","conclusion":"skipped"}
{"name":"build me (macos-x86_64)","status":"completed","conclusion":"success"}
{"name":"build me (linux-x86_64)","status":"completed","conclusion":"success"}
{"name":"build me-preview (all targets)","status":"completed","conclusion":"success"}
{"name":"build me (windows-x86_64)","status":"completed","conclusion":"success"}
{"name":"test (rust + go)","status":"completed","conclusion":"success"}
{"name":"build me (macos-aarch64)","status":"completed","conclusion":"success"}
{"name":"build me (linux-aarch64)","status":"completed","conclusion":"success"}
```

All 8 check-runs on the SHA are `completed`; no `in_progress` entries remained (the ambiguous second-run case the brief warned about did not arise before the final push was issued).

## Final push to `master` — exact output

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   8ebda45..aa23ca4  master -> master
```

No "Bypassed rule violations" string appeared. The push reported a clean fast-forward only.

## `ci/staging` deletion

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

## Positive control — `ls-remote`, both refs in the same call

```
$ git ls-remote origin refs/heads/master refs/heads/ci/staging
aa23ca464bcd195d6be7561ed6097e029dde212e	refs/heads/master
```

`master` present at the staged SHA; `ci/staging` absent from the same output.

## Verdict

**SATISFIED**
