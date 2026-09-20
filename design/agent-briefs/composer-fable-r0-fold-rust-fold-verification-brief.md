# Fold verification — fold A's fold (descriptor-mnemonic), composer fable review r0

Mechanical lens (sonnet). Repo `/scratch/code/shibboleth/descriptor-mnemonic`,
branch `fable-r0-fold`, worktree `/scratch/code/shibboleth/.tmp/fold-rust`. The
FOLD is commit `dd7cdffc` (parent `4de55155`); YOUR RANGE IS
`git diff 4de55155..dd7cdffc`. The review it answers:
`mnemonic-engrave/design/agent-reports/composer-fable-r0-fold-rust-review.md`
(I-1, M-1, M-2, M-3, N-1, N-2, N-3). Read-only; commit nothing; no sub-agents;
never read `.jsonl`. Own worktree:
`rm -rf /scratch/code/shibboleth/.tmp/verify-rust && git -C /scratch/code/shibboleth/descriptor-mnemonic worktree add --detach /scratch/code/shibboleth/.tmp/verify-rust dd7cdffc`;
`CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/verify-rust-target`;
`export TMPDIR=/scratch/code/shibboleth/.tmp`; pinned toolchain first on PATH
(`$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin`). Revert every
mutation; remove the worktree when done.

Already settled (controller ran it): fmt clean, pinned clippy 0 errors, nextest
1356/1356; on the CLI `sh + two key-less` prints the legacy line, `36 slots +
two key-less` the slot cap, `wsh + two key-less` the cap. Do not re-run the
whole suite.

## ONE QUESTION
For each of the seven findings: did the fold change what the finding said was
wrong, exactly as the finding stated it (not a neighbour, not a paraphrase)?
And can each NEW assertion fail — mutate the code it guards (swap the two new
precedence checks back above the cap; make the vector loop ignore `kind`; break
the pairing in the i2 premise) and show the RED output. Did the fold introduce
a new defect (a vector case whose `error` fields the loop cannot read; a
CHANGELOG clause that states a number the review did not measure)?

## Report — FINAL action
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-fable-r0-fold-rust-fold-verification.md`:
verdict line with counts; a table finding → VERIFIED / NOT VERIFIED with the
evidence; each mutation with its RED output; anything new. Return only the
verdict line, the counts, and the path.
