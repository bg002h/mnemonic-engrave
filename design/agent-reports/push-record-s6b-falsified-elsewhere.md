# Push record — S6b falsified-elsewhere sweep + follow-on records

## Staged SHA

`6ec4d3fec336c5e103cde6ec94a227d13378eec8`

Verified as `git rev-parse HEAD` immediately before starting, and re-verified
immediately before the final `git push origin master` (unchanged — `master`
was frozen for the whole window as required).

Commits carried (7, closing the S6b pre-flash cycle's record-keeping):

```
6ec4d3f fold: a fifth site the sweep missed -- F3's own finding heading
d256974 fold: the falsified-elsewhere sweep -- 1 Important, 2 Minor, all annotated
e1e4584 scripts: plan-cite-check resolved unmerged citations against the wrong tree
7175975 reports: persist the falsified-elsewhere sweep -- 1 Important, 2 Minor
688ab44 followups: file F-209 and close it -- and record the lens it exposed
82b55a6 reports: persist the P9 fold verification -- GREEN, 0C/0I, F1/F2/F3 all CLOSED
9c9b6cc reports: the failure-states push record -- check SATISFIED
```

(previous `master` tip before this push: `91b0b24cac501c6fa5c74d73091101f02942e011`)

## Ritual steps executed

1. `git push origin master:refs/heads/ci/staging` — created branch `ci/staging`
   at `6ec4d3f`, triggering the `release` workflow for that exact SHA.
2. `gh run watch 32151778757 --repo bg002h/mnemonic-engrave --exit-status` —
   watched to completion, exit code `0`.
3. Re-verified `git rev-parse HEAD` == `6ec4d3fec336c5e103cde6ec94a227d13378eec8`
   immediately before the final push — unchanged, tip never moved during the
   window.
4. `git push origin master` — fast-forwarded `origin/master` from `91b0b24` to
   `6ec4d3f`. No "Bypassed rule violations" message.
5. `git push origin --delete ci/staging` — deleted the staging branch.
6. `git ls-remote origin 'refs/heads/*'` — confirmed only expected refs remain.

## Run ID and URL

- Run ID: `32151778757`
- URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/32151778757
- Workflow: `release`, triggered by push to `ci/staging`
- Head SHA: `6ec4d3fec336c5e103cde6ec94a227d13378eec8`

## Per-job conclusions (via `gh api .../check-runs`, filtered to `status == completed`)

```
assemble + sign + release: skipped
build me-preview (all targets): success
build me (macos-x86_64): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-aarch64): success
test (rust + go): success
```

`test (rust + go)` — the required branch-protection context — is `success`.
`assemble + sign + release` is `skipped`, as expected: it is gated on
`refs/tags/v*`, and this was a plain branch push (`ci/staging`, not a tag).
This matches the documented expectation in `.github/workflows/release.yml`
and confirms no signing/publishing occurred from a non-tag ref.

## Final push to master — exact output

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   91b0b24..6ec4d3f  master -> master
```

Exit code: `0`. No "Bypassed rule violations" text anywhere in the output —
the required check was satisfied by the SHA earning it on `ci/staging` first,
per `strict: false` semantics.

## Tip-freeze confirmation

`git rev-parse HEAD` was checked twice: once before staging
(`6ec4d3fec336c5e103cde6ec94a227d13378eec8`) and once immediately before the
final push to `master` (same value). The local working tree had no
uncommitted changes (`git status --short` was empty) at the start. The tip
did not move during the window.

## `ls-remote` after cleanup

```
$ git ls-remote origin 'refs/heads/*'
6ec4d3fec336c5e103cde6ec94a227d13378eec8	refs/heads/master
3b4b4ff37a08bb829878de54b83613267f0c273f	refs/heads/sysw-container
```

`ci/staging` is gone. `master` is at the expected SHA. `sysw-container` is a
pre-existing, unrelated branch (not part of this push) and its presence is
expected/unaffected.

## Verdict

**SATISFIED**
