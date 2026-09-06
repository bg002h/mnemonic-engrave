You are the INDEPENDENT verification reviewer (sonnet tier) for two small device changes in the seedhammer fork, branch `h7-496-497` (tip `a327a217`, base main `5cf93fd7`). Both come from operator rulings of 2026-09-06 recorded in mnemonic-engrave `design/FOLLOWUPS.md` under F-496 and F-497; there is no spec or plan document for them, and none is owed — read the two commit messages, which carry the reasoning in full.

ONE QUESTION: does each change do exactly what its commit message says, and can each new or edited test fail on the defect it names?

Read-only on the branch; commit nothing; no sub-agents; never read any `.jsonl`. Build and RUN in your own detached worktree: `git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/seedhammer-h7-check a327a217`; Go is `/scratch/code/shibboleth/.toolchain/go/bin/go` first on PATH; `TMPDIR=/scratch/code/shibboleth/.tmp`; whole-gui counts only via `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`. Mutate only there; revert after each; remove the worktree when done.

## Already settled — do not re-derive
- The RULINGS are the operator's and are not yours to revisit: F-496, reconcile only when the payload carries no `hash:` record matching the derived digest; F-497, the census owns its scope in copy rather than the device growing durable state.
- Controller-measured at the tip: gui 1290 tests across 24 shards ok; every non-gui package ok; gofmt clean; the emulator walk `cmd/emu/walk_hashlock_phrase.js` ran once in a browser and returned `ok: true`, with the census frame showing the new scope line.
- Secret-handling never gates (operator ruling 2026-08-27).

## Check, by running it
1. **F-496's condition, both directions.** Re-run the two mutations the commit names (drop the `!payloadStatesDigest` guard; invert it) and confirm each reds the subtest the message claims and not the other. Then find a case the tests DO NOT cover and say so: in particular, does `hashlockPreimageRecordRoute` still draw no reconciliation screen (the commit says it deliberately does not get one), and is there any path where a payload phrase assigns a hash and neither branch runs?
2. **The comments that were rewritten.** `hashlockPayloadRoute`'s header claimed §10.2 was "true by construction" with one call site. Confirm the new text is true of the code as it now stands — count the call sites of `composerCopyHashlockReconcile` yourself — and that no OTHER comment or test in the tree still asserts the old single-call-site property (grep for "by construction", "ONE call site", "no-op for a phrase").
3. **F-497's line reaches the screen.** Re-run both mutations (drop the append; append unconditionally). Confirm the copy table alone does not catch the first — that claim is in the commit message and is the reason the two tests exist. Check the line is absent from the stand-alone notice form and from a census with no preimage block.
4. **Fit and paging.** The census draws in `composerReadScreen` with paging. Confirm the added line did not push content off a page or change the page count the walk records (`censusPages`), and that `TestComposerCensusRowsAreDrawnInsideTheBand` still holds.
5. **Counts.** `composer_copy.go` body count 73 → 74 and the copy-table row: confirm both are right and that the gate would still catch a body added without a row.

## Severity
Critical: a payload phrase that assigns a hash with no reconciliation screen where the payload states no matching digest (the defect F-496 exists to remove), or the screen drawn where the payload does state it; a test that cannot fail on what it names; a false claim in either commit message. Important: an uncovered reachable case, a comment that is now false, a paging or fit regression. Minor/Nit: wording.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/h7-496-497-review.md` (create; must not exist): findings `### C-n / I-n / M-n / N-n — title` with the command and verbatim output; the mutation table; the call-site count; closing counts and GREEN / NOT GREEN. Return a two-line summary plus the path.
