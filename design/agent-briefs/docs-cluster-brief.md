You are the RECORDS agent (sonnet tier) for four documentation follow-ups in the m-format constellation. This is MECHANICAL correction work: every fact below was measured by the controller, and your job is to make the documents say the measured thing, nothing more. Do not redesign anything, do not touch code behaviour, do not audit.

Repos: engrave `/scratch/code/shibboleth/mnemonic-engrave` (branch `docs-cluster` off master: `git -C /scratch/code/shibboleth/mnemonic-engrave worktree add -b docs-cluster /scratch/code/shibboleth/me-worktrees/docs-cluster master`); fork `/scratch/code/shibboleth/seedhammer` (branch `docs-cluster` off main: `git -C /scratch/code/shibboleth/seedhammer worktree add -b docs-cluster /scratch/code/shibboleth/.tmp/seedhammer-docs-cluster main`). Go is `/scratch/code/shibboleth/.toolchain/go/bin/go` first on PATH; `TMPDIR=/scratch/code/shibboleth/.tmp`.

## F-498 — a line citation that has already gone stale once
MEASURED: the production `composerState` literal is at `gui/composer_flow.go:51` on fork main `fcd1546` (`st := &composerState{reg: &seedRegistry{}, bound: composerBoundFrom(ctx.sysw)}`), not `:34`. Three FORK comments cite `:34`: `gui/composer_provenance_test.go:25`, `gui/composer_state_hook.go:17`, `gui/composer_state.go:304`.
FIX, in the fork: do NOT write `:51`. A line number is what went stale; cite the FUNCTION instead — `composerFlow` in `gui/composer_flow.go` — so the next edit above it cannot falsify the comment. Keep each sentence otherwise intact. Verify with `grep -rn "composer_flow.go:34" /scratch/code/shibboleth/.tmp/seedhammer-docs-cluster --include=*.go` returning nothing.
Leave the ENGRAVE design documents alone: `SPEC_hashlock_H5_device_polish.md`, `IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md` and the H6 spec are records of what was true when written, and the H6 spec already flags the citation as stale. Editing a closed plan rewrites history; the fork comment is the copy that ships.

## F-499 — the gofmt baseline is FIVE files
MEASURED on the pristine fork (`gofmt -l .` at main `fcd1546`): `gui/transaction.go`, `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`. Documents say three (they predate the `mt` package's arrival).
FIX: add the measured baseline ONCE where the NEXT cycle will read it rather than editing closed plans — in engrave `CLAUDE.md`, in the "Toolchain on this machine" section, one short paragraph: the five files, the command that lists them, the date, and the point that a gate should compare against this set rather than assert an empty `gofmt -l`. Do not edit `IMPLEMENTATION_PLAN_hashlock_H2_device.md` or `IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`.

## F-502 — two §8h blockquotes show one arm of four
MEASURED: `gui/composer_copy.go` has FOUR `HASH ON EVERY PATH` bodies — `composerCopyHashEveryPath` (:183), `composerCopyHashEveryPathPhrase` (:570), `composerCopyHashEveryPathHeld` (:684) and a fourth at :693 (read it; it is the held+phrase arm) — chosen by `composerCopyHashEveryPathFor` (:582). Two engrave specs quote only the first: `SPEC_wallet_policy_composer.md` §8h (the blockquote at about line 724) and `SPEC_hashlock_H2_device.md` §8h.
FIX: in BOTH specs, replace the single blockquote with all four, each labelled by the condition that selects it (read `composerCopyHashEveryPathFor` for the exact predicates and transcribe the bodies BYTE-EXACT from `composer_copy.go`, joining the concatenated string literals). Add one sentence naming `composerCopyHashEveryPathFor` as the chooser and H6 §10.1 as the section that added the arms. These two specs are LIVE references, not closed plans, which is why they get corrected rather than left.

## F-505 — a byte count that is not a pin
MEASURED: `cmd/emu/emu.wasm` rebuilds to 11,011,082 B where H6 spec §11.6 records 11,011,084 B; the walk file `cmd/emu/walk_hashlock_phrase.js` hashed `dfb9e6d5cf5521349db0c116cf7426039e7ff6c177f86e269f92105ddc9bc581` at fork `e089a539`.
FIX: in engrave `design/SPEC_hashlock_H6_preimage_plates.md` §11.6, keep the figure but mark it as an observation rather than a pin, in one sentence: the artifact is neither the firmware nor shipped, the size is not byte-reproducible across rebuilds (two bytes moved), and a spec needing a reproducible emulator identity should pin the walk file's sha256 instead. Do not delete the original measurement.

## Closing the follow-ups
In engrave `design/FOLLOWUPS.md`, append a short **CLOSED 2026-09-06** paragraph to each of F-498, F-499, F-502 and F-505 stating what was changed and where, and — for F-498 and F-499 — WHY the closed plans were deliberately left alone. Keep each to a few lines; the reasoning belongs in the commit message, not repeated here.

## Gates and commits
Fork: `gofmt -l` on the files you touched must be empty; `go build ./...` and `go test ./gui/ -run TestComposerCopy -count=1` must pass (comment-only changes, so this is a smoke check). Commit ONE fork commit (`git commit -s -F <file>`). Engrave: one commit for the spec/CLAUDE.md/FOLLOWUPS edits (`git commit -F <file>`). Both messages end with the trailers
```
Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
```
Stage paths explicitly; nothing pushed; no commits on `master`/`main`; no sub-agents; never read any `.jsonl`.

FINAL ACTION: write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/docs-cluster-f498-f499-f502-f505.md` (create; must not exist) with, per follow-up, the exact edits and the verification command output; then return a two-line summary plus the path and both branch tips.
