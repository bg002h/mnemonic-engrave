architect sh2-S3-Q3: 1 answered; 0 prior rulings changed

# Architect ruling sh2-S3-Q3: delete the fork's `bc/*` or keep it as a future UR door

Inputs read: `bughunt-grade-sh2.md` (S3-Q3 at line 120, open row `sh-bc-ur-door` at line 207,
UR rows 10a-d/36/37/38 at lines 41, 46-48 and 180, 184-186), `bughunt-triage-sh2-s3.md` (Q3 at
line 48, "Noticed, not tested" at lines 57-61), the fork (`/scratch/code/shibboleth/seedhammer`,
`origin/main` = `abdcb18`, 2026-10-11; `upstream/main` = `ea4b65b`, which is also the merge base),
the fork's `CLAUDE.md`, and mnemonic-engrave `origin/master:design/FOLLOWUPS.md` (F-234).

## Answer

**Keep `bc/*`. The sh2 fix round deletes nothing under `bc/`.** Items 10a-d, 36, 37 and 38 stay
deferred M under S3-Q2's reach rule. Each one is owed by whichever change first imports
`seedhammer.com/bc/...` from outside `bc/`. That change takes the item's fix first and regrades it
to I. This was not a tie (see the reasons below).

## Reach today (checked myself)

- `git grep -n '"seedhammer.com/bc/' <ref> -- '*.go' | grep -v ':bc/'` prints nothing for
  `origin/main`, `abdcb18` and `upstream/main`. `git grep -n 'bc/' origin/main -- ':!bc'`
  (go.sum excluded) also prints nothing, so no script, flake or doc references it.
- `go list -e -tags <T> -deps -test ./...` (go1.26.8, `GOTOOLCHAIN=local`, on a `git archive` of
  `origin/main` in my scratchpad) shows no `bc` package as a dependency of anything outside `bc/`,
  and the per-package Imports/TestImports/XTestImports show no importer outside `bc/`. I ran this
  for T = "", `refugium`, `tinygo` and `tinygo,refugium`. Inside `bc/`, the five packages
  (bytewords, fountain, ur, urtypes, xoshiro256) only import each other.
- With `bc/` moved out of the scratch copy, `go build ./...` exits 0 with empty output. `go vet ./...`
  gives the same 10 lines with or without `bc/`; the two outputs are identical after sorting.
  (Aside: those 10 lines are a vet failure that already exists on `origin/main`: test files use
  `testing.ArtifactDir`, which needs go1.26, but `go.mod` declares `go 1.25.10`. `go test` still
  passes, for example `ok seedhammer.com/bspline`. This is unrelated to this question and is not
  graded here.)
- Where the door went: upstream removed the only decoder caller itself, in `3c6571c` "gui: delete
  unused functions" (2026-06-12). That commit took the `bc/ur` and `bc/urtypes` imports and the
  `ur.Decoder` fields out of `gui/gui.go` and kept the packages. The fork never closed a door.
  Deleting `bc/*` is therefore safe today, and keeping it costs nothing for the shipped build.

## Why

Correct results do not separate the two options. Nothing the shipped default build reaches, and
nothing under any tag set, imports `bc/` (shown above). Deleting it fixes no reachable defect, and
keeping it ships none. The risk is in wiring it later, and the wiring trigger below handles that.

Fewest holder steps does not separate them either, because no holder-facing surface touches `bc/`.

Least build and review cost decides, together with two standing facts:
1. The fork's `CLAUDE.md` line 7 says `main` "tracks upstream `main` plus our additive features ...
   this tree is kept close to upstream for clean, reviewable PRs". Upstream still carries all of
   `bc/` and still edits it (`0c3b4eb` 2026-02-10 "all: go fix ./...", `3233c94` 2026-05-15 "all:
   upgrade to btcsuite v2"). Deleting it would make a permanent subtractive divergence that
   conflicts with every later upstream edit of `bc/`. It would also break the fork's own rule that
   its changes are additive.
2. mnemonic-engrave F-234 is still **OPEN** (owning phase "Goal 1 — Engrave a Transaction"). Its
   open question 2 reads: "**Raw bytes or UR?** The fork vendors `bc/ur`. UR is self-describing and
   multi-part, which is worth something for recovery". Deleting `bc/*` now would decide an open
   product question as a side effect of a bug-fix round. Keeping it decides nothing. The UR encoder
   side (`ur.Split`, `fountain`, `urtypes.EncodeKey/EncodeDescriptor`) is exactly what F-234 would
   use, and that is where item 37 sits.

Deleting would also cost a PR: five packages plus their tests, including the triage's RED rows in
`bc/fountain` and `bc/urtypes`. Keeping costs a few FOLLOWUPS lines. Brian's existing rulings
stand. S3-Q2 (grade by reach today, mark "becomes I when wired") is exactly the mechanism this
answer relies on.

## What the FIX doc(s) must do (fold needed: yes, the grade/FIX doc, by the coordinator)

1. **Close the open row** `S3-Q3 | open | sh-bc-ur-door` (grade line 207). Record it under Product
   rulings as: "S3-Q3: keep `bc/*`. It is upstream code, and F-234 Q2 may use it. UR items stay
   deferred until wired (architect-sh2-s3q3)." Change the counts line's "open product questions 1
   (S3-Q3, architect)" to 0.
2. **Close none of 10a-d, 36, 37, 38, and move none of them.** They stay deferred M. Their RED rows
   stay in `bc/fountain` and `bc/urtypes` with `t.Skip("deferred-minor: <slug>")`, as the grade's
   Deferred section already says. S3-Q4 (the seqLen cap) stays deferred with fix-10.
3. **Owning trigger on each of the four FOLLOWUPS entries** (`sh-ur-fountain-bounds`,
   `sh-ur-hdkey-index-range`, `sh-ur-range-shape`, `sh-ur-script-type-mismatch`), one clause:
   "owned by the first change that imports `seedhammer.com/bc/` from outside `bc/` (a UR scan door,
   or F-234 choosing UR); that change applies fix-NN, un-skips the row and regrades to I first".
   Also give entry 37 "becomes I when wired" (the grade line 185 text lacks it). If F-234 engraves
   a UR through `EncodeKey`, a range key would be written in a shape that the BCR-2020-007 grammar
   (quoted in triage line 36) and the package's own parser refuse. That is wrong plate data, not a
   refusal.
4. **File the triage's two untested `bc/urtypes` notes** (triage lines 59-60) under the same
   trigger, without a grade, each needing a RED row when wired:
   `sh-ur-hdkey-depth-noop` and `sh-ur-multisig-threshold-unbounded`. I read both at `origin/main`.
   In the first, `bc/urtypes/urtypes.go` `hdkeyFor` (from line 86) does
   `depth := len(k.DerivationPath); if depth == len(k.DerivationPath) { depth = 0 }`, which is
   always true. In the second, the sortedmulti branch (around line 403) assigns
   `desc.Threshold = m.Threshold` with no bound, which is the UR counterpart of 6a.
5. **One cross-link sentence in mnemonic-engrave F-234, open question 2** (same repo as the grade):
   "Choosing UR wires the fork's `bc/*`, which carries deferred defects `sh-ur-*` (sh2 bughunt);
   that phase takes them first." This makes the trigger visible where the wiring decision will be
   made. No spec, printed text, CLI flag or toolkit manual changes.

## Not decided here / Brian's alone

- Nothing in this ruling is Brian's alone. No branch, worktree, tag or persistence change is
  involved.
- Whether to offer fix-10/36/37/38 to `seedhammer/seedhammer` upstream is not decided here. It
  needs a push to an external repository, which no brief authorizes. The FOLLOWUPS entries may say
  "upstream candidate" and nothing more.
- Whether F-234 uses UR at all remains open in F-234. This ruling keeps that option without
  choosing it.

Counts: questions answered 1 · prior rulings changed 0 · Brian's alone 0 · fold needed 1 (grade/FIX doc + one F-234 sentence)
