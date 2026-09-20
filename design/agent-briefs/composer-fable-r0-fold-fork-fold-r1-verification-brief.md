# Fold verification — fold B's r1 fold (the SeedHammer fork), composer fable review r0

Mechanical lens (sonnet). Repo `/scratch/code/shibboleth/seedhammer`, branch
`fable-r0-fold`, worktree `/scratch/code/shibboleth/.tmp/fold-fork`. The r1
FOLD is `0f435048..aabef11` (six commits: `1eb4d4d` M-3 + re-vendor,
`a3d16cd` I-1 + M-2, `f4c484e` I-2, `c891c29` M-1, `f222eed` M-4 + N-4,
`aabef11` M-5 + N-1). The review it answers:
`mnemonic-engrave/design/agent-reports/composer-fable-r0-fold-fork-review.md`
(I-1, I-2, M-1..M-5, N-1..N-4). The fold's own account:
`composer-fable-r0-fold-fork-fold-r1.md` (same directory; N-3 declined there
with a reason, N-2 doc-only). Read-only; commit nothing; no sub-agents; never
read `.jsonl`. Own worktree:
`rm -rf /scratch/code/shibboleth/.tmp/verify-fork && git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/verify-fork aabef11`;
Go `/scratch/code/shibboleth/.toolchain/go/bin/go` first on PATH;
`export TMPDIR=/scratch/code/shibboleth/.tmp`; never build under `/tmp`. One
test: `CGO_ENABLED=0 go test ./gui/ -run '^Name$' -count=1 -v` (confirm with
-v that it ran); the whole gui package only via
`/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`.
Revert every mutation; remove the worktree when done.

Already settled (controller ran it at aabef11): vet baseline-only, gofmt the
five-file baseline, md/sysw/mk ok, gui 1373/1373 across 24 shards, firmware
1,653,724 B flash. Do not re-run the whole suite.

## ONE QUESTION
For each of the eleven findings (I-1, I-2, M-1..M-5, N-1, N-4; N-2 and N-3
as declared): did the fold change what the finding said was wrong, exactly as
stated? Can each NEW assertion fail — mutate and show RED: swap the Go cap
check back above the legacy/slot checks (the four `precedence_*` vector cases
must go RED); widen `soleMulti` back (the mixed-branch fixture must go RED);
re-admit a testnet mk1 card at the card door; count an empty path as key-less
again; drop a `kind` from the vector dispatch; remove one name from
`composerDocOwners`' new entries; dedup on the fingerprint again; delete the
`discardLast` call on the failure leg. Did the fold introduce a new defect —
in particular, is the re-vendored `compose_refusal_keyless_cap.json` byte-equal
to descriptor-mnemonic `b2c5d693`'s, and does its provenance file name that
commit?

## Report — FINAL action
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-fable-r0-fold-fork-fold-r1-verification.md`:
verdict line with counts; a table finding → VERIFIED / NOT VERIFIED with
evidence; each mutation with its RED output; anything new. Return only the
verdict line, the counts, and the path.
