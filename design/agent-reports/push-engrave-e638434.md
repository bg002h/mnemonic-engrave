# Push mnemonic-engrave master via staging ritual — e638434

Repo: `/scratch/code/shibboleth/mnemonic-engrave`
GitHub: `bg002h/mnemonic-engrave`
Prior `origin/master`: `75f00b56`
Pushed tip (`TIP`): `e638434ab58b23c36864e698356efdd71f199b43`

## Commands run, in order

1. Record tip:
   ```
   TIP=$(git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master)
   ```
   Result: `e638434ab58b23c36864e698356efdd71f199b43`

2. Precondition check:
   ```
   git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
   ```
   Result: empty output, exit 0 — precondition satisfied (no stale `ci/staging` ref).

3. Staging push (foreground, no backgrounding, no separate watcher):
   ```
   bash scripts/push-via-staging.sh master
   ```
   (run from `/scratch/code/shibboleth/mnemonic-engrave`)

   Run id created: `34007761793`

4. Fetch + confirm:
   ```
   git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
   git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
   ```
   Result: `e638434ab58b23c36864e698356efdd71f199b43` — matches `TIP`.

5. Per-job conclusions (full SHA, `--repo`):
   ```
   gh run view 34007761793 --repo bg002h/mnemonic-engrave \
     --json databaseId,headSha,status,conclusion,jobs \
     --jq '{databaseId, headSha, status, conclusion, jobs: [.jobs[] | {name, status, conclusion}]}'
   ```
   Result:
   ```json
   {"conclusion":"success","databaseId":34007761793,"headSha":"e638434ab58b23c36864e698356efdd71f199b43","jobs":[
     {"conclusion":"success","name":"test (rust + go)","status":"completed"},
     {"conclusion":"success","name":"build me (macos-aarch64)","status":"completed"},
     {"conclusion":"success","name":"build me-preview (all targets)","status":"completed"},
     {"conclusion":"success","name":"build me (windows-x86_64)","status":"completed"},
     {"conclusion":"success","name":"build me (linux-aarch64)","status":"completed"},
     {"conclusion":"success","name":"build me (linux-x86_64)","status":"completed"},
     {"conclusion":"success","name":"build me (macos-x86_64)","status":"completed"},
     {"conclusion":"skipped","name":"assemble + sign + release","status":"completed"}
   ],"status":"completed"}
   ```

## Verbatim script output (full tail, `scripts/push-via-staging.sh master`)

```
== staging e638434ab58b23c36864e698356efdd71f199b43 (branch master, 7 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34007761793; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   75f00b56..e638434a  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (macos-aarch64): success
build me-preview (all targets): success
build me (windows-x86_64): success
build me (linux-aarch64): success
build me (macos-x86_64): success
assemble + sign + release: skipped
== OK: e638434ab58b23c36864e698356efdd71f199b43 is on master with the required check earned
```

Script exit code: `0`. No "Bypassed rule violations" line anywhere in the output (checked with `grep -i bypass` over the captured log — no match).

The script ran to completion in the foreground on its own; no separate `gh run view`/`gh run watch` polling loop was needed (the script's own wait covered the required `test (rust + go)` context).

## Post-push verification

- `git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin` — ran clean.
- `git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master` → `e638434ab58b23c36864e698356efdd71f199b43`, equal to `TIP`.
- `ci/staging` ref deleted by the script (confirmed in the script's own output: `- [deleted] ci/staging`).

## Outcome

SUCCESS. `master` is at `e638434ab58b23c36864e698356efdd71f199b43` on `origin`, the required `test (rust + go)` context is satisfied (not bypassed), all build jobs succeeded, `assemble + sign + release` correctly skipped (not a tag push), and `origin/master` matches the pre-recorded `TIP`.
