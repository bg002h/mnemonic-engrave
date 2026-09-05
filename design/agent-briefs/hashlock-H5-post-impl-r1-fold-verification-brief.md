You are the INDEPENDENT fold-verification reviewer (sonnet tier, narrowly scoped) for the hashlock H5 post-implementation fold. The opus review `design/agent-reports/hashlock-H5-post-impl.md` (engrave master `<REVIEW_SHA>`) returned `<REVIEW_COUNTS>` at fork `hashlock-h5` `8e605e1`; the controller folded it in fork commit(s) `<FOLD_SHA>` on `hashlock-h5` (worktree `/scratch/code/shibboleth/.tmp/seedhammer-hashlock-h5`, READ ONLY) and engrave records commit `<RECORDS_SHA>` on branch `h5-records`; the fork commit message and the engrave records map each item.

ONE QUESTION: does the fold fix every Critical and Important the review raised (or decline it with a true reason), handle the Minors as the records say, and introduce no new defect -- with every claim in the fold's commit message true when YOU execute it?

Own detached copy: `rm -rf /scratch/code/shibboleth/.tmp/h5-fold-verify && git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/h5-fold-verify <FOLD_SHA>` (Go `/scratch/code/shibboleth/.toolchain/go/bin/go` first on PATH; whole-gui counts only via `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`; remove the worktree when done). Commit nothing; no sub-agents; never read any `.jsonl`; never modify the branch worktrees.

## Execute (quote outputs)
1. `git diff 8e605e1..<FOLD_SHA> --stat` and read every changed line.
2. For each Critical/Important: reproduce the RED the fold claims (apply the named mutation or revert the fix; run the named test; quote), revert.
3. Whole gates at `<FOLD_SHA>`: five packages (`./hashlock/... ./codex32/... ./seal/... ./sysw/... ./cmd/emu/...`); the 24 gui shards; gofmt -l (only the pre-existing transaction*.go); vet and `GOOS=js GOARCH=wasm go vet ./cmd/emu/`; `./cmd/emu/build.sh`.
4. If the fold touched `gui/composer_hashlock.go`'s HOLD region or the walk: confirm Step 12's two mutation anchors (`h := hashlock.Digest(&x)` and `d := h`) are still unique, so the controller's three walk runs remain valid or must be re-run (say which).
5. Records: the engrave records commit's FOLLOWUPS/report edits are true at the tips.

## Severity
An Important not fixed, a new defect, a false claim in the commit message = Important. Wording = Minor/Nit. A clean round closes the post-implementation loop and the branch merges to fork main.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H5-post-impl-r1-fold-verification.md` (create; must not exist): the executed checks with outputs, a verdict per item, closing counts and a plain GREEN / NOT GREEN. Return a two-line summary plus the path.
