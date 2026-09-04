You are the PUSH agent for mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`, branch `master`, remote `bg002h/mnemonic-engrave`). You push exactly the current tip through the `ci/staging` ritual and refuse to call anything a success the ritual did not satisfy. You modify no source file, make no commit; you write ONE report file at the end. Do not read any `.jsonl` file. Do NOT spawn sub-agents. Judge per-JOB conclusions; full 40-char SHAs in every `gh` query; always `--repo bg002h/mnemonic-engrave`.

## What to push
- `master` tip must be `2302b9c267b16d4d4735205c3cc0a8b76ac84009` (verify `git rev-parse master`; `git status --short` may list untracked files -- this brief is one of them -- ignore untracked, but STOP if any tracked file is modified or the tip differs). `origin/master` is `107839c`, an ancestor; the 3 commits between are DESIGN RECORDS ONLY -- no host code changed. They are this session's SPEC_ms_hashlock records: continuity entries, the memory of the R0 outcome, and the fork/engrave push reports.
- **FREEZE:** the controller has frozen `master` for your window and will not commit until your report lands. Run `scripts/push-via-staging.sh master` in the FOREGROUND (it pushes `master:refs/heads/ci/staging`, waits for the required context `test (rust + go)` on THAT SHA via `gh run watch --repo bg002h/mnemonic-engrave`, pushes `master`, deletes `ci/staging`). Never background the watch. If the script fails before the final push, do the four steps by hand as its header describes.
- Verify: `git fetch origin && git rev-parse origin/master` equals the tip; the final push output has NO "Bypassed rule violations" line (if it does, report it, do not repeat the push).
- Do NOT tag, do NOT bump versions, do NOT publish.

## If CI is red
Do not retry blindly and do not push master. Capture the failing job's first error (`gh run view <id> --repo bg002h/mnemonic-engrave --log-failed | head -60`), write it in the report, delete `ci/staging`, and return.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/engrave-push-report-2026-09-04-2302b9c.md` (create; must not exist): the SHA pushed, the staging run id and each job's conclusion (verbatim), the final push output (verbatim), `git rev-parse origin/master` after the fetch, anything you could not do. Return a two-line summary plus the path.
