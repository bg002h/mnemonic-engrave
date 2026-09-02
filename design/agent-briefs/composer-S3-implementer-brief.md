You are the SINGLE implementer of a GREEN implementation plan for the SeedHammer fork (Go GUI). Execute it task by task, exactly as written, in a fresh git worktree. No design decisions are yours to make: where the plan is silent or wrong, STOP at that task, write what you found into the report named below, and return.

## The plan
`/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` at mnemonic-engrave master `<S3_GREEN_SHA>` (read it whole first; three parts: Part A = Tasks A1–A11, Part B = B1–B11, Part C = C1–C2). Its spec, for reference only: `design/SPEC_wallet_policy_composer.md` §6b, §6c, §7, §8, §9 items 3–11, §12, §13. The plan's Go was machine-verified: every new `gui/composer_*.go` file extracted into a scratch copy of the fork and, with the plan's fragments of shipped files hand-wired, `go vet ./gui/` clean (two pre-existing ArtifactDir findings aside), `go test -run '^TestComposer' ./gui/` ok (212 PASS lines), the whole gui package 1170/1170 across 24 shards, `./md/ ./mk/ ./sysw/` ok; Part A ALONE (extraction stopped at Task B1) builds and passes its composer tests (89 PASS lines). Expect the code to build as written; if it does not, that is a finding, not a licence to redesign.

## Where
```
cd /scratch/code/shibboleth/seedhammer && git worktree add /scratch/code/shibboleth/wt-composer-s3 -b composer-s3
```
Work only inside the worktree; never touch the main fork checkout, never push, never flash. Go: `/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin/go` (put its `bin` on PATH; `gofmt` beside it); `CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`, default `-mod=readonly` (never `-mod=mod`: it rewrites go.mod). `TMPDIR=/scratch/code/shibboleth/.tmp` for every test run (the tmpfs is small). The sharded gui runner: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`.

## How
- Tasks A1 → A11, then the Part-A milestone check the plan states in Task A11 (Part A builds and passes alone) — record it; then B1 → B11, then C1 → C2. Every step, including running the failing test first and reading its failure. Commit per task, signed-off (`git commit -s -F <file>`), with the plan's message; keep the trailer lines.
- Task A10 is BLOCKED on F-453 (the S0b presets plan) unless the dispatch message says S0b has shipped and names its merge commit: if blocked, do exactly what the task says for the blocked case and continue.
- `gofmt -l` clean on every touched file and `go vet ./gui/` clean (the two pre-existing ArtifactDir findings excepted) before each commit. NO pre-existing test may be edited except the ones the plan names as exact old→new replacements; if another fails, stop and record it.
- Every §8 string verbatim from the plan's copy file; do not reflow a body.
- Task C1 measures (the per-frame capacities and the plate ceiling) and folds numbers into the spec as the plan says; paste the measurements. Task C2 runs CI's gates (`CGO_ENABLED=0 go test -timeout 20m ./...` or the sharded runner, `scripts/test-32bit.sh`, the oraclelive build, the js vet, gofmt) and the firmware size recipe (`nix run .#build-firmware`, then the `tinygo build -size short …` line) against the baseline the plan states.
- Machine-check, never describe: paste real command output for every gate you claim ran.

## The report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-S3-implementation-report.md` (create; must not exist yet): the worktree path and branch; `git log --oneline 321acb56..HEAD`; per task the fail-then-pass evidence (trimmed) and ANY deviation from the plan's Expected lines, verbatim; the Part-A milestone result; Task C1's measurements; Task C2's gate output and firmware sizes; anything you had to decide, could not do, or stopped on. Return a two-line summary plus the path. Do not read any `.jsonl` file.
