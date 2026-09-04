# Push report: engrave master → 917d4e3 via ci/staging

## Tip and commits pushed

Tip SHA: `917d4e37ff0ae733a6fd706a45d4cdfb153262be` (`917d4e3`).

Commits pushed (previous `origin/master` was `2302b9c`), oldest first:

1. `1c0edd2` continuity: H1 plan drafted (ms dbccbe8), gate iterating; three controller defaults still await the operator
2. `d75d56e` continuity: H1 plan R0 round 0 landed (2C/10I fidelity, 0C/4I tests), fold applied, re-gate in flight
3. `917d4e3` continuity: H1 plan fold committed (ms 3592532, gate run 13 green); r1 sonnet fold verification dispatched

(`052f6a5` was already on `origin/master` at the start of this push — it is the prior push's own report commit, not part of this push's payload.)

## Workflow run and per-job conclusions

Run id: **33927403159**, head SHA `917d4e37ff0ae733a6fd706a45d4cdfb153262be`, overall conclusion `success`.

| job | conclusion |
| --- | --- |
| build me (macos-x86_64) | success |
| build me (linux-x86_64) | success |
| **test (rust + go)** (required context) | **success** |
| build me-preview (all targets) | success |
| build me (windows-x86_64) | success |
| build me (macos-aarch64) | success |
| build me (linux-aarch64) | success |
| assemble + sign + release | skipped (tag-only job; expected on a branch push) |

## Bypass check

`grep -i bypass` over the full captured script output found no match. Last lines of the push output, verbatim:

```
To github.com:bg002h/mnemonic-engrave.git
   2302b9c..917d4e3  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
build me (macos-x86_64): success
build me (linux-x86_64): success
test (rust + go): success
build me-preview (all targets): success
build me (windows-x86_64): success
build me (macos-aarch64): success
build me (linux-aarch64): success
assemble + sign + release: skipped
== OK: 917d4e37ff0ae733a6fd706a45d4cdfb153262be is on master with the required check earned
```

Full captured log: `/scratch/code/shibboleth/.tmp/push-engrave-917d4e3.log`.

## Independent post-push verification

- `git fetch origin && git rev-parse origin/master` → `917d4e37ff0ae733a6fd706a45d4cdfb153262be` — matches local tip.
- `gh run view 33927403159 --repo bg002h/mnemonic-engrave --json databaseId,headSha,conclusion,jobs` confirms `headSha` matches and the `test (rust + go)` job's own `conclusion` is `success` (independent of the script's report of the same).
- `git ls-remote origin refs/heads/ci/staging` → empty output; the staging ref is deleted.
- `gh api repos/bg002h/mnemonic-engrave/commits/<sha>/status` returned `total_count: 0` (legacy commit-status API, unused by this repo's Actions-based checks) — not a discrepancy; `gh run view` above is the authoritative source and matches the script's own claim.

## Verdict

**SUCCESS**
