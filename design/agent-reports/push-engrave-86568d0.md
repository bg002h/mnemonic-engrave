# Push report: mnemonic-engrave master via ci/staging — 86568d0c

Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)

## Preconditions

```
$ TIP=$(git rev-parse master)
TIP=86568d0c8800ac2ea49d94d63b9e9586855f6e69

$ git ls-remote origin refs/heads/ci/staging
(empty — no stale staging ref)

$ git status --short
(clean)

$ git rev-parse origin/master   # before push
9045b5781c2c025c9dcce2a76d05d925e6525715

$ git log --oneline origin/master..master
86568d0c continuity: H6 plan round 0 folded (7021ea43 / spec 95418b2c); r1 verification dispatched
82364f1f brief: H6 plan R0 r1 fold verification -- SHAs filled (fold 7021ea43 over e6d84d9c; spec 95418b2c)
b0cbf154 report: H6 plan R0 round-0 fold author's report (opus) -- 1C/13I/8M folded, 3 decisions, 1 decline on fit, 3 self-found defects; 97/97; verbatim
7021ea43 fold: H6 plan R0 round 0 -- 1C/13I/8M folded across plan, spec and the three gated trees ...
95418b2c fold: H6 spec, plan R0 round 0 -- the locator test requirement, the vendored-row pin ...
178be3e5 report: engrave push 9045b578 via ci/staging -- test (rust + go) success on run 34013154184, no bypass; verbatim
```

6 commits ahead of origin/master, matching the expected description (push report, H6 spec + plan round-0 folds, an agent report, a brief, continuity — no crate code).

## Command run

```
$ ./scripts/push-via-staging.sh master
```

Run in the foreground, no timeout hit, no backgrounding, no watcher wait needed — the script itself waited out the run.

### Verbatim tail (full captured output)

```
== staging 86568d0c8800ac2ea49d94d63b9e9586855f6e69 (branch master, 6 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34015170572; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   9045b578..86568d0c  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me-preview (all targets): success
build me (linux-aarch64): success
build me (windows-x86_64): success
build me (macos-x86_64): success
build me (macos-aarch64): failure
build me (linux-x86_64): success
assemble + sign + release: skipped
== OK: 86568d0c8800ac2ea49d94d63b9e9586855f6e69 is on master with the required check earned
```

No "Bypassed rule violations" line anywhere in the output.

## Run details (`gh run view 34015170572 --repo bg002h/mnemonic-engrave`)

```
$ gh run view 34015170572 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha
{"conclusion":"failure","status":"completed","headSha":"86568d0c8800ac2ea49d94d63b9e9586855f6e69"}
```

Overall run conclusion is `failure` only because of the non-required `build me (macos-aarch64)` job (informational straggler, matches the script's own report above). The required context for branch protection, `test (rust + go)`, is `success`.

### Per-job conclusions

```
$ gh run view 34015170572 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | "\(.name): \(.conclusion)"'
test (rust + go): success
build me-preview (all targets): success
build me (linux-aarch64): success
build me (windows-x86_64): success
build me (macos-x86_64): success
build me (macos-aarch64): failure
build me (linux-x86_64): success
assemble + sign + release: skipped
```

`assemble + sign + release` is `skipped` as expected — it is gated on `refs/tags/v*`, and this is a plain `ci/**` (then `master`) push, not a tag.

## Post-push verification

```
$ git fetch origin --quiet
$ git rev-parse origin/master
86568d0c8800ac2ea49d94d63b9e9586855f6e69
```

Matches `TIP` exactly — `origin/master` == `86568d0c8800ac2ea49d94d63b9e9586855f6e69`. `ci/staging` ref was deleted by the script (confirmed in the verbatim tail above; `git ls-remote origin refs/heads/ci/staging` returns empty after the run).

## Outcome

**Success, no bypass.** The required `test (rust + go)` check ran and passed against SHA `86568d0c` before it reached `master` via the `ci/staging` staging ritual; no "Bypassed rule violations" message appeared. `origin/master` now equals the recorded `TIP`. The unrelated `build me (macos-aarch64)` job failure is a non-required, informational straggler (per `.github/workflows/release.yml`'s job gating) and does not affect the push's validity.

Nothing was committed by this agent; this report is the only file it wrote.
