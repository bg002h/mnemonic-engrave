You are the PUSH agent (sonnet tier) for mnemonic-toolkit at `/scratch/code/shibboleth/mnemonic-toolkit`. Local master tip `67090e2a` is ONE record commit (a push report) over `origin/master` `7e07088c`. Ship it with the STAGING-PR ritual exactly as recorded in `design/agent-reports/push-toolkit-7e07088c.md` (precedent PR #69): the required contexts are path-filtered, and a docs-only commit cannot earn them from a plain branch push, so the PR form is what makes the checks run on the SHA. Read that report first and mirror its steps, its evidence, and its required-context list (`examples`, `test (ubuntu-latest)`, `clippy`, per that report -- if the report lists otherwise, the report wins).

Rules: full SHAs and `--repo` on every `gh` call, per-job conclusions; the final `git push origin master` output is quoted verbatim and any "Bypassed rule violations" line is FAILURE (report, do not retry); close the PR and delete the staging branch afterwards as the precedent did. Do NOT stage or touch the untracked `cycle-prep-recon-*.md` files in the repo root -- they are not ours. Do not commit anything on master before the push completes.

Report (your final action): write `/scratch/code/shibboleth/mnemonic-toolkit/design/agent-reports/push-toolkit-67090e2a.md` (create; must not exist) with every command, PR number, run ids, per-job conclusions and the verbatim final-push output, then commit ONLY that file on master with a five-line message ending in the trailers
```
Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
```
(use `git commit -F <file>`; the shell is fish). Return two lines: outcome + path. Never read any `.jsonl`.
