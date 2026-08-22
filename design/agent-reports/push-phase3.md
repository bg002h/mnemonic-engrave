# mnemonic-toolkit — Phase 3 push (Sparrow refusal test)

`ci/staging` push ritual for **mnemonic-toolkit only**, branch `master`.

## Clean-tree check

`git status --porcelain | grep -vE '^\?\?'` → empty (0 tracked changes).
38 untracked pre-existing files present (same count as previous rounds), as
briefed. Treated as clean per standing instruction for this repo.

## Tip

- Branch `master`. Tip SHA: `8342b2eabb9ed96da2fef6aece592842dee1a8e9` (40
  chars confirmed via `wc -c`). One commit since `e95e80a8`: `8342b2ea` —
  "test(export-wallet): Phase 3 — make the Sparrow refusal deliberate".
- **What actually changed, checked before writing anything below**: `git
  diff --stat e95e80a8..8342b2ea` touches exactly one file —
  `crates/mnemonic-toolkit/tests/cli_export_wallet_allow.rs` (+82, test
  file only, no `src/` change). Matches the coordinator's description.

## Staging

`git push origin master:refs/heads/ci/staging --force` → `* [new branch]
master -> ci/staging`. Verified via fresh `git fetch origin` that `git
rev-parse master` == `git rev-parse origin/ci/staging` ==
`8342b2eabb9ed96da2fef6aece592842dee1a8e9`.

## CI on `ci/staging`

Three workflows fired (no `bitcoind-differential` this round — a
test-file-only diff plausibly doesn't touch its trigger paths; not
separately verified since it isn't a required context and wasn't flagged
as a concern):

- `examples` (`databaseId 32587400046`) → **success** (required context)
- `sibling-pin-check` (`databaseId 32587400039`) → **success**
- `rust` (`databaseId 32587400134`) → **success**, 13/13 jobs green,
  including `test (ubuntu-latest)` and `clippy` (both required contexts).
  First `gh run watch` attempt exceeded a single 10-minute blocking call;
  re-issued the same watch in the foreground per the standing correction
  against ending the turn on a background wait, and it completed.

**All three required contexts (`examples`, `test (ubuntu-latest)`,
`clippy`) confirmed green by name.**

## Tip-movement check, before the real push

`git rev-parse master` == `8342b2eabb9ed96da2fef6aece592842dee1a8e9`
(exact match to staged/tested SHA); fresh `git fetch origin` showed
`origin/master` unchanged at `e95e80a8e13b45528daef8ff01f5bed2f41b5522`,
confirmed an ancestor of local `master` (clean fast-forward guaranteed);
re-confirmed 0 tracked changes. No movement observed.

## `git push origin master`

```
To github.com:bg002h/mnemonic-toolkit.git
   e95e80a8..8342b2ea  master -> master
```
No "Bypassed rule violations" text present. Genuine fast-forward.

## The `docs/manual` flag-coverage lint caveat — checked, not just noted

The coordinator flagged in advance that `docs/manual`'s flag-coverage lint
is red **locally** (three undocumented flags), pre-existing at `e95e80a8`
(verified by them via `git stash`), and structurally invisible to CI
because `MD_BIN`/`MK_BIN` default to building the *adjacent sibling
checkout* rather than the toolkit under test — so a green `manual` result
here would not be evidence the docs are fine.

**Checked the actual mechanism for this specific SHA rather than repeating
the caveat verbatim**: `manual.yml`'s own `paths:` filter is
`docs/manual/**`, `docs/tools/render-mermaid-cache.py`,
`.github/workflows/manual.yml`. This commit touches none of those (it's a
single test file, confirmed above) — so **`manual` did not run at all for
this push**, on `ci/staging` or on `master`. There is consequently no
green (or red) `manual` result to potentially misread as evidence either
way for this SHA specifically; the described blind spot is real for any
SHA that *does* touch `docs/manual/**`, but doesn't even come into play
here since nothing relevant changed. Confirmed via `gh run list --commit
8342b2eabb9ed96da2fef6aece592842dee1a8e9` on both waves of runs — `manual`
is absent from both lists, consistent with the path-filter read.

## Post-push verification — the other `[main, master]`-only workflow

`technical-manual` DID fire on the real `master` push (`databaseId
32588481722`) — its own trigger paths are broader than `manual.yml`'s and
this commit apparently intersects them (or it triggers on any push
regardless of path; not independently re-derived since it isn't a required
context and came back green). Job `lint` (which builds/verifies against
real binaries) → **success**.

Full post-master-push wave, all confirmed by name:
- `technical-manual` (`databaseId 32588481722`) → success
- `examples` (`databaseId 32588481703`) → success
- `sibling-pin-check` (`databaseId 32588481723`) → success
- `rust` (`databaseId 32588481724`) → success, 13/13 jobs green again

`git push origin --delete ci/staging` → `- [deleted] ci/staging`. Fresh
fetch confirmed `origin/master` ==
`8342b2eabb9ed96da2fef6aece592842dee1a8e9`.

## VERDICT: PUSHED

All CI signal for this SHA is green across both waves (`ci/staging` + real
`master`): three required contexts (`examples`, `test (ubuntu-latest)`,
`clippy`) plus every other job. `manual` produced no signal at all for
this specific commit (path-filtered out, not a false green) — the
coordinator's caveat about it being structurally blind to the local lint
failure is noted but does not apply to interpreting *this* push, since
`manual` never ran. No tip movement observed. No bypass text.
