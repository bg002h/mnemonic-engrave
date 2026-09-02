# Push report — mnemonic-engrave master, 2026-09-02c

## TIP
`67ffa3e165bfb064e576363d24a24e841b55231a`

(8 commits ahead of prior `origin/master` tip `88b4a4aa971274e10aea8eded04d3716426ae371`; `git status --short` was empty at the start of the run.)

## Run
Workflow run id: `33628428758`

Required job conclusion (verbatim, from `gh run view 33628428758 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | select(.name=="test (rust + go)") | .conclusion'`):
```
success
```

## Final push output (verbatim, from `scripts/push-via-staging.sh master`)
```
== staging 67ffa3e165bfb064e576363d24a24e841b55231a (branch master, 8 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33628428758; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   88b4a4a..67ffa3e  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
build me-preview (all targets): success
build me (macos-x86_64): success
test (rust + go): success
build me (linux-aarch64): success
build me (windows-x86_64): success
build me (macos-aarch64): success
build me (linux-x86_64): success
assemble + sign + release: skipped
== OK: 67ffa3e165bfb064e576363d24a24e841b55231a is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output. `assemble + sign + release` reported `skipped` (correctly gated on `refs/tags/v*`; this push published nothing).

## origin/master after fetch
```
git fetch origin && git rev-parse origin/master
```
→ `67ffa3e165bfb064e576363d24a24e841b55231a`

This equals TIP. Push confirmed successful, no bypass, required context satisfied on the pushed SHA.
