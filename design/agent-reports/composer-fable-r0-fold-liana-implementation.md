# Fold implementation — lens 5 (Liana) device notice: composer-fable-r0-fold-liana

**Verdict: GREEN.** One commit, `fa070df011b382ae9baa1c54c93bbf353d902c59`, on branch
`fable-r0-fold-liana`, worktree `/scratch/code/shibboleth/.tmp/fold-liana`, off base
`781dc7cd71703412034d8188dbc62c463336bbe5` (fork main). Nothing pushed; no sub-agents
dispatched; no `.jsonl` transcript read; nothing built under `/tmp` (`TMPDIR` pinned to
`/scratch/code/shibboleth/.tmp` throughout, including the shard script and the tinygo
build). Two probe/scratch test files used to empirically determine the predicate
against real `md.Compose` output were deleted before the real test was written and
never committed.

## What was built

`gui/composer_consent.go`: `composerLianaOutsideModelClass(root md.ScriptKind, shape
md.PolicyShape) string` names the FIRST of Liana 8.0's nine refusal classes that
applies, in Liana's own order of refusal, or `""` when the policy fits Liana's model.
Wired into `composerConsentLinesFor` right after the §8g mixed-lock-bases check.

`gui/composer_copy.go`: `composerCopyOutsideLianaModel(class string) string` is the
new §8x body (one FIXED head, one variable class sentence).

## The body and every class phrase, verbatim

```
OUTSIDE LIANA'S MODEL
Liana takes one unlocked path, at least one path locked by older in
blocks, and no hash. This policy: <class>. Bitcoin Core imports it.
```

The nine class phrases, in the checked order:

1. `legacy wrapper`
2. `NUMS key path`
3. `no locked path`
4. `a hash lock`
5. `an absolute lock`
6. `a lock in time units`
7. `no unlocked path`
8. `two paths with one lock`
9. `a second unlocked path`

Class 9 is deliberately unified: Liana refuses a second unlocked MULTI-key path
outright, and silently folds a second unlocked SINGLE-key path into the first as an
extra key without changing its threshold (lens 5 I-2, X24) — both are named
`a second unlocked path` because both leave the operator with a wallet Liana will not
show as built.

A real spendable taproot key path (`md.KeyPathSpendable`) counts as an unlocked path
in the predicate: every shipped `tr` preset puts its primary there rather than in a
leaf, so without this a `tr` wallet with one real key path and one timelocked leaf —
Liana's own accepted shape — would misread as class 7.

## RED before / GREEN after

RED was produced by temporarily removing only the three-line wiring block from
`composerConsentLinesFor` (predicate and body functions left in place, since the test
file references them by name). Running
`TestFableOutsideLianaModelNamesTheFirstClass`:

- All 9 one-per-class positive rows failed (`notice ... not found`).
- The order-priority row (hashlock-gated under `tr`, which is both NUMS and hash —
  must report NUMS, not hash) also failed.
- The 4 negative rows (tr-with-real-key, kofn-recovery-wsh,
  simple-timelocked-inheritance-wsh, tiered-recovery-wsh) passed trivially — so the
  predicate was not trivially satisfiable in either direction.

Restoring the wiring: GREEN, 0 failures, `modalHeadroom` 397 chars on the longest
class line (`two paths with one lock`), margin 80.

## Mutation notes — one per class, RUN and CONFIRMED CAUGHT

Each mutation below was applied to a scratch copy of `composer_consent.go`, the
target test re-run, and the file restored to the committed content before the next
mutation (verified by `diff` against the pre-mutation original after each restore).
This is not a reasoned claim — every one of the ten commands below was executed and
its FAIL confirmed in this session.

| # | mutation | result |
|---|---|---|
| 1 | `root == md.ScriptSh` → `root == md.ScriptWsh` | row 1 (`legacy wrapper`) fails |
| 2 | `shape.KeyPath == md.KeyPathNUMS` disabled | rows 2 and 10 (order-priority) fail |
| 3 | `case !anyLock:` → `case anyLock && false:` | row 3 (the demo payload's own shape) falls through to `""` |
| 4 | `case hash:` → `case hash && false:` | row 4 (hashlock-gated-wsh) falls through to `""` |
| 5 | `case after:` → `case after && false:` | row 5 (decaying-multisig-wsh) falls all the way to `"no unlocked path"` — demonstrates the order dependency the doc comment claims |
| 6 | `case olderUnits:` → `case olderUnits && false:` | row 6 falls through to `""` |
| 7 | `case unlocked == 0:` → `case unlocked == -1:` | row 7 falls through to `""` |
| 8 | `if n >= 2 {` → `if n >= 3 {` | row 8 falls through to `""` |
| 9 | `if unlocked >= 2 {` → `if unlocked >= 3 {` | row 9 falls through to `""` |
| KeyPathSpendable | the `if shape.KeyPath == md.KeyPathSpendable { unlocked++ }` block removed | negative row 11 (tr-with-real-key) MISFIRES as `"no unlocked path"` — the exact regression this fold exists to prevent |

The 10th mutation (KeyPathSpendable) was first attempted with a single-line `sed`
pattern spanning a literal `\n`, which silently no-opped (confirmed with `diff`
showing no change) and reported a false PASS — corrected with a Python
string-replace and re-run to a genuine, caught FAIL.

## Gate — numbers

Worktree `fable-r0-fold-liana`, tip `fa070df011b382ae9baa1c54c93bbf353d902c59`, off
base `781dc7cd71703412034d8188dbc62c463336bbe5`.

- `go vet ./...`: only the 10 pre-existing `testing.ArtifactDir requires go1.26 or
  later (file is go1.25)` notes, in files this fold did not touch (`bspline`,
  `gui/op`, `engrave`, `backup`, `gui/freetext_sizeproof_golden_test.go`,
  `gui/transaction_golden_test.go`).
- `gofmt -l .`: exactly the five-file baseline (`gui/transaction.go`,
  `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`, `mt/mt.go`,
  `mt/mt_test.go`) — unchanged by this fold; every file this fold touched is clean.
- `CGO_ENABLED=0 go test ./gui/ -run '^TestFableOutsideLianaModelNamesTheFirstClass$' -count=1 -v`:
  PASS.
- `scripts/gui-shard-test.sh ./gui/ 24 20m`: 1374 top-level tests enumerated,
  partition verified exhaustive, 24/24 shards `ok`, wall 21s.
- Firmware size (`nix develop -c tinygo build -size short -o /dev/null -target
  pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`):
  - baseline @ `781dc7cd` (main checkout, HEAD verified at that SHA before building):
    flash 1,653,724 B / RAM 63,336 B.
  - this fold (worktree, uncommitted-then-committed state identical): flash
    1,655,084 B / RAM 63,336 B (+1,360 B flash, +0 B RAM).

## Deviations from the brief

None.
