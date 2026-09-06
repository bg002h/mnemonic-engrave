# docs-cluster: F-498, F-499, F-502, F-505 -- mechanical documentation corrections

Records agent report. Brief: `design/agent-briefs/docs-cluster-brief.md`.
Base commits: engrave master `906a6ecf`, fork main `fcd1546`. Worktrees:
`/scratch/code/shibboleth/me-worktrees/docs-cluster` (branch `docs-cluster`
off `master`), `/scratch/code/shibboleth/.tmp/seedhammer-docs-cluster`
(branch `docs-cluster` off `main`). No `.jsonl` files were read at any
point.

## F-498 -- fork comments citing the stale `gui/composer_flow.go:34`

Verified the production literal's current location:

```
$ grep -n "composerFlow\|composerState{" gui/composer_flow.go
50:func composerFlow(ctx *Context, th *Colors) {
51:	st := &composerState{reg: &seedRegistry{}, bound: composerBoundFrom(ctx.sysw)}
```

Matches the brief's measurement (line 51, not 34, not 51-as-a-new-pin).

Edit, per file (parenthetical citation changed from `(gui/composer_flow.go:34)`
to a function-naming form; sentences otherwise intact):

- `gui/composer_state_hook.go:17` -- `composerState is a LOCAL of composerFlow
  (gui/composer_flow.go)` (function already named in the sentence; the
  parenthetical now cites only the file, no line).
- `gui/composer_provenance_test.go:25` -- `... EXACTLY as composerFlow does
  (gui/composer_flow.go) -- a struct literal ...` (same pattern).
- `gui/composer_state.go:304` -- function was NOT named in this sentence, so
  it now reads `... at its one production site, composerFlow
  (gui/composer_flow.go), and in every test in this package ...`.

Verification command from the brief, run after the edit:

```
$ grep -rn "composer_flow.go:34" /scratch/code/shibboleth/.tmp/seedhammer-docs-cluster --include=*.go
(no output)
```

Left alone, per the brief: `SPEC_hashlock_H5_device_polish.md`,
`IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md`, and the H6 spec (which
already flags the citation as stale) -- these are records of what was true
when written; editing a closed plan rewrites history, and the fork comment is
the copy that ships.

Fork commit: `fb23777b8c3669966dbc65dfe357663f8c8cccd4`.

## F-499 -- gofmt baseline is five files, not three

Reproduced on the pristine fork worktree (`fcd1546`):

```
$ gofmt -l .
gui/transaction.go
gui/transaction_golden_test.go
gui/transaction_txrecord_test.go
mt/mt.go
mt/mt_test.go
```

Matches the brief exactly (five files; the last two are the `mt` package,
absent from documents written before it arrived).

Fix: added one paragraph to engrave `CLAUDE.md`, in the "Toolchain on this
machine (2026-09-02)" section, after the firmware-size bullet -- the five
files, the command, the date (2026-09-06, F-499), and the instruction that a
gate should diff `gofmt -l` against this set rather than assert it empty.
`IMPLEMENTATION_PLAN_hashlock_H2_device.md` and
`IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` were NOT edited, as instructed
(both predate the `mt` package and are closed plans).

## F-502 -- §8h blockquotes showed one arm of four

Read `gui/composer_copy.go` in full around the four bodies and the chooser:

- `composerCopyHashEveryPath` (:182) -- not held, no phrase (the baseline/else arm).
- `composerCopyHashEveryPathPhrase` (:569) -- not held, `composerAnyPathByPhrase(st)` true.
- `composerCopyHashEveryPathHeld` (:683) -- `composerEveryHashedPathHeld(st)` true, not every held path has a phrase.
- `composerCopyHashEveryPathHeldPhrase` (:692, the "fourth at :693" the brief pointed at) -- every hashed path held AND every held path has a phrase.
- Chooser: `composerCopyHashEveryPathFor` (:582), dispatching in that order (held-with-phrase, held, phrase-route, baseline).

Byte-exactness of the transcription was machine-verified, not eyeballed: wrote
a throwaway Go file reproducing the four functions' string concatenations
exactly as they appear in `composer_copy.go`, ran it with `go run`, and diffed
the printed strings against each spec's blockquote (blockquote lines rejoined
with spaces, heading kept separate). All four bodies matched byte-for-byte in
both specs:

```
A: HASH ON EVERY PATH
Every way to spend this wallet needs the preimage of a hash. It is not on this device and not on these plates. Back up every preimage separately.
B: HASH ON EVERY PATH
Every way to spend this wallet needs a hashlock preimage. It is not on this device and not on these plates. Back up every phrase and its method, and every preimage plate, separately.
C: HASH ON EVERY PATH
Every way to spend this wallet needs the preimage of a hash. This composition holds the preimage for each one and can cut a plate for it at Done. Store those plates apart from these, and apart from each other.
D: HASH ON EVERY PATH
Every way to spend this wallet needs a hashlock preimage. This composition holds the phrase and method for each one and can cut a plate at Done. Store those plates apart from these, and apart from each other.
```
(reconstructed from source; matched the reconstruction extracted from both
edited markdown files)

Edited both specs' §8h section:

- `SPEC_wallet_policy_composer.md` §8h (was the single baseline blockquote at
  line ~724, and it was NOT even the current shipped wording -- it read "Back
  the preimage up separately" where the shipped string is "Back up every
  preimage separately"). Replaced with all four bodies, each under a bold
  label naming its condition and function, preceded by one sentence naming
  `composerCopyHashEveryPathFor` (`gui/composer_copy.go:582`) as the chooser
  and H6 §10.1 as the section that added the two held arms.
- `SPEC_hashlock_H2_device.md` §4.7 (was the phrase-route arm only, with a
  parenthetical already flagging that the shipped baseline text had drifted).
  Replaced with all four bodies under the same labelling scheme; the stale
  parenthetical is gone because the section now states the current byte-exact
  text directly rather than noting that it once didn't.

## F-505 -- emu.wasm byte count is not a reproducible pin

Verified the walk file's sha256 independently (not just transcribed):

```
$ sha256sum cmd/emu/walk_hashlock_phrase.js
dfb9e6d5cf5521349db0c116cf7426039e7ff6c177f86e269f92105ddc9bc581  cmd/emu/walk_hashlock_phrase.js
```

Matches the brief's cited hash exactly, and matches at the CURRENT fork tip
(not only at the historical `e089a539` the brief cites) -- confirmed the
commit `e089a539` exists in this fork's history and the file is unchanged
since.

Did not rebuild `cmd/emu/emu.wasm` to re-derive the 11,011,082 B rebuild
figure -- that fact was stated by the controller as measured, and the fix
only needed to add an observation sentence around the existing, unmodified
11,011,084 B figure, not re-derive the drift itself.

Edit: added one paragraph in `SPEC_hashlock_H6_preimage_plates.md` §11.6,
directly after the existing "moved 11,010,867 -> 11,011,084 B" sentence,
marking the figure an observation rather than a pin, keeping the original
measurement verbatim, and naming the walk file's sha256 as the reproducible
alternative. The hash string was kept on one unbroken line (an earlier draft
of this edit accidentally split it across a markdown line break inside the
code span; caught and fixed before committing).

## Closing FOLLOWUPS.md

Appended a `**CLOSED 2026-09-06.**` paragraph directly after each of F-498,
F-499, F-502 and F-505's existing text (each as a single unbroken line,
matching the file's one-paragraph-per-line convention), stating what changed
and where; F-498 and F-499's closures state why the closed plans were left
alone. Reasoning otherwise lives in the commit messages, not repeated here.

## Gates run

Fork (`/scratch/code/shibboleth/.tmp/seedhammer-docs-cluster`, Go
`/scratch/code/shibboleth/.toolchain/go/bin/go` on PATH,
`TMPDIR=/scratch/code/shibboleth/.tmp`):

```
$ gofmt -l gui/composer_state.go gui/composer_state_hook.go gui/composer_provenance_test.go
(empty)

$ go build ./...
(no output, exit 0)

$ go test ./gui/ -run TestComposerCopy -count=1 -v
=== RUN   TestComposerCopyIsVerbatimFromTheSpec
--- PASS: TestComposerCopyIsVerbatimFromTheSpec (0.00s)
=== RUN   TestComposerCopyIsDrawable
--- PASS: TestComposerCopyIsDrawable (0.00s)
=== RUN   TestComposerCopyTableCoversEveryBody
--- PASS: TestComposerCopyTableCoversEveryBody (0.00s)
=== RUN   TestComposerCopyTableCoversTheSameXpubRefusal
--- PASS: TestComposerCopyTableCoversTheSameXpubRefusal (0.04s)
PASS
ok  	seedhammer.com/gui	0.043s
```

(`TestComposerCopyIsVerbatimFromTheSpec` checks Go string constants against
an in-package table, `composerCopyTable()` -- confirmed by reading the test --
not against the markdown specs edited here, so it is unaffected by, and gives
no coverage of, the F-502 markdown edits; the markdown correctness rests on
the manual byte-for-byte reconstruction above.)

## Commits

- Fork, `docs-cluster` off `main` `fcd1546`: `fb23777b8c3669966dbc65dfe357663f8c8cccd4`
  ("records: F-498 -- cite composerFlow by name, not a line, in three
  comments"), signed off, 3 files changed.
- Engrave, `docs-cluster` off `master` `906a6ecf`: `41dc1d5d5cb866331364e59fda62e86e37ba8c50`
  ("records: close F-498, F-499, F-502, F-505 -- docs-cluster documentation
  fixes"), 5 files changed (`CLAUDE.md`, `design/FOLLOWUPS.md`,
  `design/SPEC_hashlock_H2_device.md`,
  `design/SPEC_hashlock_H6_preimage_plates.md`,
  `design/SPEC_wallet_policy_composer.md`).

Nothing pushed. No commits landed on `master` or `main` -- both remain at
`906a6ecf` and `fcd1546` respectively, verified by `git rev-parse` after the
docs-cluster commits. No sub-agents were used.
