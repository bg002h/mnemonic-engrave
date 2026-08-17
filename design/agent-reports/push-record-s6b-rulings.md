# Push record — S6b rulings batch (2026-08-17)

## What was pushed

Five commits, all design/registry documents — no code:

- `c48b012` reports: persist the S6b follow-ups push record
- `d55fc4c` followups: correct F-192/F-208 -- the SH2 CAN scroll with arrows
- `ac088f7` followups: F-208 moves into S6b -- operator reaffirmed the arrows
- `19cd62c` s6b: record the three operator rulings from the decision pass
- `a776358` s6b: record R-D -- "all things said must be true" -- and close R-C 1 and 3

## SHA staged

Full 40-character SHA, verified as `git rev-parse HEAD` on entry and again immediately
before the final push (master was frozen for the whole window, per the operator's
hard constraint):

```
a7763583ddf558fc629a0e8af53476d032e7aa0c
```

## Ritual executed

```sh
git push origin master:refs/heads/ci/staging     # builds this exact SHA
gh run watch 32050128559 --repo bg002h/mnemonic-engrave --exit-status
git push origin master                           # no bypass message = SATISFIED
git push origin --delete ci/staging
```

## Workflow run

- Run ID: `32050128559`
- URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/32050128559
- Workflow: `release`
- Trigger: `push` to `ci/staging`
- `headSha` reported by the run (via `gh run view --json headSha`):
  `a7763583ddf558fc629a0e8af53476d032e7aa0c` — matches the staged SHA exactly.
- Run-level `status`: `completed`; run-level `conclusion`: `success`.

## Per-job conclusions (verbatim, from `gh run view 32050128559 --repo bg002h/mnemonic-engrave --json jobs`)

| Job | databaseId | conclusion |
| --- | --- | --- |
| build me-preview (all targets) | 95447248360 | success |
| **test (rust + go)** | 95447248369 | **success** |
| build me (macos-x86_64) | 95447248415 | success |
| build me (linux-aarch64) | 95447248422 | success |
| build me (linux-x86_64) | 95447248447 | success |
| build me (macos-aarch64) | 95447248461 | success |
| build me (windows-x86_64) | 95447248514 | success |
| assemble + sign + release | 95447792205 | skipped |

The required context, `test (rust + go)` (job ID 95447248369), concluded `success`.

`assemble + sign + release` (job ID 95447792205) concluded `skipped`, as expected —
that job is gated on `refs/tags/v*`, and this was a push to `refs/heads/ci/staging`,
so it could not sign or publish anything.

## Final push to master — exact output

```
$ git rev-parse HEAD
a7763583ddf558fc629a0e8af53476d032e7aa0c

$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   add6c1e..a776358  master -> master
```

Exit status: `0`. **No "Bypassed rule violations" string appeared anywhere in the
output.** The push report shows a plain fast-forward update (`add6c1e..a776358`)
with no rule-violation or bypass annotation of any kind.

## ci/staging deletion

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

## Positive-control verification

```
$ git ls-remote origin refs/heads/master refs/heads/ci/staging
a7763583ddf558fc629a0e8af53476d032e7aa0c	refs/heads/master
```

`master` is present at exactly the staged/pushed SHA in the same query where
`ci/staging` returns nothing — i.e. `ci/staging` is confirmed absent while `master`
is confirmed present, not merely "both empty."

## Hard-constraint compliance

- `master` was not committed to, amended, rebased, or otherwise touched by this
  agent; `git rev-parse HEAD` was checked at entry and immediately before the final
  push, both times returning `a7763583ddf558fc629a0e8af53476d032e7aa0c`.
- Every `gh` invocation used `--repo bg002h/mnemonic-engrave` explicitly.
- Every SHA quoted above is the full 40-character form.
- Per-job conclusions were judged individually (table above), not the run-level
  status alone.
- No push to any other remote, no tags created, `seedhammer` fork untouched.
- `enforce_admins` was not inspected, discussed, or proposed for change.

## Verdict

**SATISFIED**
