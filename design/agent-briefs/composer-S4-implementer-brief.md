You are the SINGLE implementer of a GREEN implementation plan that spans TWO repositories: the SeedHammer fork (Go, `cmd/emu` and a scratch tool) and mnemonic-engrave (a shell transcript and a Python capture driver under `design/journeys/`). Execute Tasks 1, 2 and 3 exactly as written, in two fresh git worktrees. No design decisions are yours to make: where the plan is silent or wrong, STOP at that task, write what you found into the report named below, and return.

## The plan
`/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_composer_S4_acceptance.md` at mnemonic-engrave master `5a5f3df977fe78a3c5c485c21b0715e8d07d1567` (read it whole first: §2 is the fixture and every oracle value, measured; §3 Tasks 1-3 are yours; Tasks 4-6 are NOT -- the controller and the operator own them). Its R0 journey lens (`design/agent-reports/composer-S4-plan-R0-r0-journey.md`) pinned every `?` cell in the Task 3 itineraries; the tables you execute are the folded ones at `5a5f3df977fe78a3c5c485c21b0715e8d07d1567`. The oracle values were produced by `md`/`me`/`ms` runs on 2026-09-03; expect your transcript to reproduce them byte for byte -- a different value is a finding, not a licence to re-pin.

## Where
```
cd /scratch/code/shibboleth/seedhammer && git worktree add /scratch/code/shibboleth/wt-composer-s4-emu -b composer-s4-emu
git worktree add /scratch/code/shibboleth/wt-engrave-s4-emu -b composer-s4-emu
```
Work only inside the worktrees; never touch either main checkout, never push, never flash. The engrave-side scripts locate the fork by a RELATIVE path (`design/journeys/capture_*.py`: `../../../seedhammer/cmd/emu`), which from `wt-engrave-s4-emu` resolves to the MAIN fork checkout, not your worktree: give `capture_composer.py` an `EMU` override (env or `--emu`), default unchanged, and run it against `/scratch/code/shibboleth/wt-composer-s4-emu/cmd/emu`; state the override in the report.

Go: `/scratch/code/shibboleth/.toolchain/go/bin/go` (1.26.7; put its `bin` on PATH, `gofmt` beside it); `CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`, default `-mod=readonly` (never `-mod=mod`). `TMPDIR=/scratch/code/shibboleth/.tmp` for every test and every build (the tmpfs is small). Tools BY PATH, never bare: `/scratch/code/shibboleth/descriptor-mnemonic/target/release/md` (built at descriptor-mnemonic `1dc8d409`), `/scratch/code/shibboleth/mnemonic-engrave/target/debug/me` (0.8.0), `~/.cargo/bin/ms` (0.16.0), `~/.cargo/bin/mk` (0.13.0). Python Playwright with chromium is installed; the shipped `capture_tr_pathological.py` shows how it is driven and `walk_s3_nested.js` how an engrave loop is driven (`shPress`/`shRelease`, toolpath stall detection, an unrecognised screen STOPS the walk).

## How
- Task 1 (fork) → Task 2 (engrave) → Task 3 (both), each with its Run/Expected. Commit per task, signed-off (`git commit -s -F <file>`), with a message that names the task; keep the trailer lines the plan's earlier stages used.
- `gofmt -l` clean on every touched Go file; `GOOS=js GOARCH=wasm go vet ./cmd/emu/` clean; `CGO_ENABLED=0 go test -count=1 ./cmd/emu/` ok -- before each fork commit. The confinement test (`cmd/emu/embed_confinement_test.go`) must pass UNCHANGED; if it fails, your blob or identifier leaks past `//go:build js`, and that is the fix, not the test.
- The negative control is a COMMAND (`--prove-it-can-fail`) and it must exit 0 only because the walk threw; paste its output.
- Run the three shipped drivers afterwards (`capture_walletpolicy.py`, `capture_seating.py`, `capture_tr_pathological.py`, each with `--port 8793 --shot-port 8734`, sequentially) and paste the exit codes.
- Machine-check, never describe: paste real command output for every gate you claim ran. Do not read any `.jsonl` file. Do NOT spawn sub-agents.

## The report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-S4-implementation-report.md` (create; must not exist yet): both worktree paths and branches; `git log --oneline main..HEAD` in the fork worktree and `git log --oneline master..HEAD` in the engrave one; per task the Run/Expected evidence (trimmed) and ANY deviation from the plan's Expected lines, verbatim -- including every oracle value the emulator printed (Template-ID, stubs, Policy-ID, the four addresses, the engraved strings) beside the plan's; the negative control's output; the three shipped drivers' exit codes; anything you had to decide, could not do, or stopped on. Return a two-line summary plus the path.
