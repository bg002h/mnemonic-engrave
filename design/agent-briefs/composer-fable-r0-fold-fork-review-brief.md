# Whole-diff review — fold B (the SeedHammer fork, device), composer fable review r0

Independent reviewer (opus). You did not write the diff, the brief, or the
report. Repo `/scratch/code/shibboleth/seedhammer`, branch `fable-r0-fold`,
base `f5b068faf3049ccf97603dfbaa3709f12893df97`, tip
`0f435048d93f69dfc5b807b9e4addd2dff704e1b`. YOUR RANGE IS
`git diff f5b068fa..0f435048` (13 commits, one per finding plus the two vector
vendorings). Read-only; commit nothing; no sub-agents; never read `.jsonl`.
Own worktree: `rm -rf /scratch/code/shibboleth/.tmp/review-fork && git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/review-fork 0f435048d93f69dfc5b807b9e4addd2dff704e1b`.
Go is `/scratch/code/shibboleth/.toolchain/go/bin/go` FIRST on PATH;
`export TMPDIR=/scratch/code/shibboleth/.tmp`; never build under `/tmp`. One gui
test: `CGO_ENABLED=0 go test ./gui/ -run '^TestName$' -count=1 -v` (a filter
that matches nothing prints ok — confirm with -v); the whole gui package ONLY
via `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`.
Revert every mutation; remove the worktree when done.

What was asked: `design/agent-briefs/composer-fable-r0-fold-fork-brief.md`
(mnemonic-engrave), plus two coordinator messages the report records (the Rust
vector `compose_refusal_keyless_cap.json`; the host-first tpub vectors, 82
rows). What the implementer reports:
`design/agent-reports/composer-fable-r0-fold-fork-implementation.md`. The
findings: the four `composer-fable-r0-{funds-safety,nunchuk,steel-restore,device-flow}.md`.

## Already settled — do not re-derive
- The key-less rule is "more than one key-less path ⇒ refuse", any locks, any
  positions (measured 859 lists in Rust). NOTE a late Rust fold you should
  know about: the Rust primary now orders the cap AFTER `TooManySlots` and
  `LegacyWrapperShape` (its remedy "fold them into one path" cures neither),
  and the vector gained four precedence cases (`precedence_*`, error kinds
  `KeylessUnderTr`, `NoKeyedPath`, `TooManySlots`, `LegacyWrapperShape`). The
  fork tip vendored the vector BEFORE that: check whether the Go
  `ValidatePathList` ordering matches the Rust ordering, and whether the
  fork's vector test can even express the four new kinds — report it as a
  finding if the port would name the cap for `sh + two key-less` or
  `36 slots + two key-less`.
- Gate at the tip as reported: vet baseline-only, gofmt five-file baseline,
  gui 1369/1369 across 24 shards, 24/24 TestFable* PASS, firmware
  1,644,840 → 1,652,268 B. The controller is re-running the gate in parallel;
  do not spend your budget on the whole suite — run what you need.
- Same-seed-different-account is ALLOWED by operator ruling; same KEY at two
  slots is refused as unsupported.

## ONE QUESTION
Over this diff, construct a policy or a tap sequence for which a change here
gives a WRONG result — a screen that states one thing while the strings handed
to the plate planner carry another; a refusal that refuses the wrong thing or
lets the found defect through in a different spelling (the re-serialised same
key; a `tpub` with mainnet-looking depth; a second key-less path in a position
the port misses); a consent line whose k-of-n is not the script's; a Back edge
that now loses something §7b says it must keep; a restore document line that
is false — or a test in this diff that passes while the property it names is
false (the implementer reports catching one such false PASS themselves; look
for the next). Judge each declared deviation, and each new FIXED body against
§8's rules (ASCII, modal-fits, dismissed only by CONTINUE).

## Report — your FINAL action
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-fable-r0-fold-fork-review.md`:
verdict line with counts C/I/M/N; each finding as a counterexample (inputs or
taps, observed, expected, file:line at the tip); deviations each ACCEPTED or a
finding; what you ran with numbers; what you could not verify. Return only the
verdict line, the counts, and the path.
