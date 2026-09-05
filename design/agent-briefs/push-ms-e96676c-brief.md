You are the PUSH agent (sonnet tier) for mnemonic-secret at `/scratch/code/shibboleth/mnemonic-secret`. Local master tip `e96676c` is ONE record commit over `origin/master` `fb98d73` (a push report). Ship it via the ci/staging ritual so the required contexts are SATISFIED, never bypassed.

Follow the newest `design/agent-reports/push-ms-*.md` in that repo as the precedent (same steps, same evidence). The ritual: `git -C <repo> push origin master:refs/heads/ci/staging`; find the run(s) for the exact full SHA (`gh run list --repo <owner/repo> --commit e96676c<full>`, `gh run view <id> --json jobs`; use full SHAs and `--repo`, per-job conclusions -- `gh` fails silently empty otherwise); wait until the FOUR required contexts succeed: `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)`; report the non-required `vendor-freshness` result too; then `git -C <repo> push origin master` and QUOTE its output -- any line containing "Bypassed rule violations" means FAILURE (report it as such; do not retry); then `git -C <repo> push origin --delete ci/staging`. Do not commit or amend anything on master before the push; the controller is frozen off ms master for the window.

Report (your final action): write `/scratch/code/shibboleth/mnemonic-secret/design/agent-reports/push-ms-e96676c.md` (create; must not exist) with every command, run id, per-job conclusion and the verbatim final-push output, then commit ONLY that file on master with a five-line message ending in the trailers
```
Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
```
(use `git commit -F <file>`; the shell is fish and eats backticks). Return two lines: outcome + path. Never read any `.jsonl`.
