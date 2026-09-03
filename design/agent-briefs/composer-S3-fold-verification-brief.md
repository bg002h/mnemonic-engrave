You are the INDEPENDENT fold-verification reviewer (targeted) for the composer S3 implementation on fork branch `composer-s3` (worktree `/scratch/code/shibboleth/wt-composer-s3`). One question: did the controller's fold commit(s) `<S3_FOLD_SHAS>` (diff against `<S3_REVIEWED_TIP>`) fix every Critical and Important in `design/agent-reports/composer-S3-exec-review-r0.md` (mnemonic-engrave master) exactly as filed, can each new or changed test fail, and did nothing else move?

Read-only; every mutation reverted; `git status --porcelain` empty at the end. Go: `/scratch/code/shibboleth/.toolchain/go/bin/go` on PATH, `CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`, `-mod=readonly`, `TMPDIR=/scratch/code/shibboleth/.tmp`; sharded gui runner `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`. Do NOT spawn sub-agents; read no `.jsonl` file.

## Verify
1. Per Critical/Important in the report: the fold's hunk(s) for it; reproduce the report's reproduction on the pre-fold tip (it must fail/misbehave) and on the fold (it must not). Apply the fold's own named mutation to each new guard -> the named test FAILS with pasted output; revert.
2. Minors/Nits: folded, filed (name the F-number in `design/FOLLOWUPS.md`), or declined with a reason -- each accounted for.
3. Nothing else moved: every hunk belongs to a finding; gofmt clean on touched files; `go vet ./gui/` clean (two pre-existing ArtifactDir findings excepted); `go test -timeout 20m ./...` all ok; sharded gui count; `-run '^TestComposer'` ok; `scripts/test-32bit.sh`; oraclelive build; js vet. The firmware size step cannot run here (no /nix) -- say so, do not substitute.

Severity: a finding folded wrongly, a guard that cannot fail, or a hunk outside the findings = Important (Critical if a guarantee breaks). Do not pad.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-S3-exec-review-r1-fold-verification.md` (create; must not exist): per item VERIFIED / NOT VERIFIED with output; closing counts. Return a two-line summary plus the path.
