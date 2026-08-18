# Push record — S6b P2 (ci/staging ritual)

**Date:** 2026-08-17
**Repo:** `bg002h/mnemonic-engrave` (fork)
**What was pushed:** Design documents only, no code — the P2 implementation report
committed verbatim (`d15cadd`), and a fold of P2's two findings into the spec and
spike results (`75f9ca8`).

## SHA staged

Full 40-char SHA, read via `git rev-parse HEAD` (not trusted from the brief):

```
75f9ca8b6e5adb02d9ac0dc336e0b4bc38e20ad0
```

`git log --oneline -5` at staging time:

```
75f9ca8 s6b: fold P2's two findings -- the budget is 28, and the spec named a test that did not need editing
d15cadd reports: commit the P2 implementation report, verbatim
edc44ef reports: the P1 push record -- check SATISFIED
989b6f4 s6b: adjudicate P1's GATE 3.1 deviation -- the implementation is RIGHT and the spec is amended to it
f8a6a25 reports: commit the P1 implementation report, verbatim
```

## Step 1 — stage on ci/staging

```
$ git push origin master:refs/heads/ci/staging
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      master -> ci/staging
```

## Step 2 — run id and watch

Run triggered by the staging push (workflow `release`, event `push`, matching
`headSha` = the staged SHA above):

- **Run id:** `32089527851`
- **URL:** https://github.com/bg002h/mnemonic-engrave/actions/runs/32089527851

```
$ gh run watch 32089527851 --repo bg002h/mnemonic-engrave --exit-status
...
EXIT_CODE=0
```

`gh run watch` blocked until completion and returned exit status 0 (all
non-skipped jobs succeeded).

## Per-job conclusions (verbatim)

From `gh api repos/bg002h/mnemonic-engrave/commits/<sha>/check-runs`, filtered
to `status == completed`:

```
assemble + sign + release: skipped
build me (macos-x86_64): success
test (rust + go): success
build me (windows-x86_64): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (linux-aarch64): success
build me-preview (all targets): success
```

Cross-checked against `gh run view 32089527851 --json jobs`:

```
build me-preview (all targets): completed / success
build me (linux-aarch64): completed / success
build me (macos-aarch64): completed / success
build me (linux-x86_64): completed / success
build me (windows-x86_64): completed / success
test (rust + go): completed / success
build me (macos-x86_64): completed / success
assemble + sign + release: completed / skipped
```

Both sources agree. The required context `test (rust + go)` is `success`.
`assemble + sign + release` is `skipped`, confirming the tag-gate held — a
`ci/**` push cannot sign or publish.

## Step 3 — push master (final)

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   989b6f4..75f9ca8  master -> master
EXIT=0
```

**No "Bypassed rule violations" string appeared.** Plain fast-forward push
output only — the check earned on `ci/staging` satisfied the rule for this
SHA on `master`.

## Step 4 — delete ci/staging

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
EXIT=0
```

## Positive control — ls-remote

```
$ git ls-remote origin | grep -E 'refs/heads/(master|ci/staging)'
75f9ca8b6e5adb02d9ac0dc336e0b4bc38e20ad0	refs/heads/master
```

`ci/staging` is **absent** while `master` is **present**, both in the same
`git ls-remote` invocation, and `master` points at the staged SHA
(`75f9ca8b6e5adb02d9ac0dc336e0b4bc38e20ad0`).

## Verdict

**SATISFIED**
