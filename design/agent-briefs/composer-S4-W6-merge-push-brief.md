You are the MERGE + PUSH agent for the SeedHammer fork (`/scratch/code/shibboleth/seedhammer`, branch `main`, remote `bg002h/seedhammer`; `main` is NOT branch-protected, so there is no ci/staging ritual here). You modify no source file; you write ONE report file at the end. Do not read any `.jsonl` file. Do NOT spawn sub-agents. Judge per-JOB conclusions; full 40-char SHAs in every `gh` query; always `--repo bg002h/seedhammer`.

## What to merge and push
- Branch `composer-s4e` (worktree `/scratch/code/shibboleth/wt-composer-s4e`), tip `618f86f1df0db1e97248c83380b1304088f1cd7c`, base `main` = `70008da5f935b36635a442cb2738f8dcc2fce7f1`. Verify in the main checkout: `git rev-parse main` = that SHA, `git status --short` empty, `git merge-base --is-ancestor main composer-s4e` true; STOP if not.
- `git merge --no-ff composer-s4e -F /scratch/code/shibboleth/mnemonic-engrave/design/agent-briefs/composer-S4-W6-merge-message.txt` in the main checkout, then `git push origin main`, then `gh run list --repo bg002h/seedhammer --commit <merge sha> --json databaseId,name,status,conclusion` and `gh run watch <id> --exit-status` in the FOREGROUND on the `test` workflow; judge each job's conclusion.
- Verify: `git fetch origin && git rev-parse origin/main` equals the merge commit.
- Do NOT tag, do NOT flash, do NOT touch the worktree.

## If CI is red
Report the failing job's first error (`gh run view <id> --repo bg002h/seedhammer --log-failed | head -60`); do not revert or force-push; return.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-S4-W6-push-report.md` (create; must not exist): the merge SHA, the run id and each job's conclusion (verbatim), the push output (verbatim), `git rev-parse origin/main` after the fetch, anything you could not do. Return a two-line summary plus the path.
