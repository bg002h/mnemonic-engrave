# Fold verification — the Liana-model consent notice (fork), composer fable review r0 lens 5

Mechanical lens (sonnet). Repo `/scratch/code/shibboleth/seedhammer`, branch
`fable-r0-fold-liana`, ONE commit `fa070df0` on base `781dc7cd`; YOUR RANGE IS
`git diff 781dc7cd..fa070df0`. The brief it implements:
`mnemonic-engrave/design/agent-briefs/composer-fable-r0-fold-liana-brief.md`;
the implementer's report: `.../agent-reports/composer-fable-r0-fold-liana-implementation.md`;
the finding: `composer-fable-r0-liana-core.md` I-1, I-2 and "what the importer
requires". Read-only; commit nothing; no sub-agents; never read `.jsonl`. Own
worktree: `rm -rf /scratch/code/shibboleth/.tmp/verify-liana && git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/verify-liana fa070df0`;
Go `/scratch/code/shibboleth/.toolchain/go/bin/go` first on PATH;
`export TMPDIR=/scratch/code/shibboleth/.tmp`; never build under `/tmp`; one
test with `-run '^Name$' -count=1 -v`; the whole gui only via
`/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`.
Revert every mutation; remove the worktree when done.

Already settled (controller ran at fa070df0): vet baseline-only, gofmt
five-file baseline, gui 1374/1374, firmware 1,655,084 B. Do not re-run the
whole suite.

## ONE QUESTION
Does the predicate name Liana's FIRST applicable class for every path list,
in the brief's order, and stay silent exactly when Liana 8.0 accepts? Check
it against the MEASURED truth, not the brief: the lens-5 evidence
`mnemonic-engrave/design/evidence/composer-fable-r0/fable-liana-parse-out.jsonl`
(variant `md`: `ok` true/false per shape) and `fable-liana-shapes.json` (the
56 shapes with their descriptors). For every shape you can express as a
composer path list (the device-composed 30 at least; the `X*` shapes where
the report gives the `md compose` path list), run the predicate and compare:
the notice must fire iff Liana refused (I-2's X24 is the one exception the
brief names: fires though Liana "accepts"). Report every disagreement as a
finding with the path list, the predicate's answer, and Liana's. Then the
mutations: for each of the nine classes, the report claims one mutation
caught — re-run at least four of them (including class 9's two halves and
class 3) and show RED. Judge the declared deviations. Check the modal-fits
assertion covers the longest class line, and that `composerDocOwners` names
every helper the commit added.

## Report — FINAL action
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-fable-r0-fold-liana-verification.md`:
verdict line with counts; the shape-by-shape agreement table (fires / Liana
refused / agree?); mutations with RED output; deviations judged; anything new.
Return only the verdict line, the counts, and the path.
