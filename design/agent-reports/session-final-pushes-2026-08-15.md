# Session final pushes — 2026-08-15

Two sequential pushes. Pre-push check on both repos: `git status --porcelain`
(unpiped) showed no modified/staged tracked files. mnemonic-engrave had two
untracked report files (`s5-oracle-fold2-verification-2026-08-15.md`,
`s5-oracle-inline-origins-2026-08-15.md`), which is fine per the rules
(untracked is not blocking).

---

## 1. `/scratch/code/shibboleth/seedhammer` — branch `main`

**Protection check:** `gh api repos/bg002h/seedhammer/branches/main/protection`
→ `404 Branch not protected` (verified before pushing). Plain push used, no
`ci/staging` dance.

**Commit range pushed:** `3c879e73f3f30d80b4499260afe49703c52b4357..84a4f4a5d0e64665d3194766d2b7953b2f476fc0`
(5 commits: 3c879e7, 5edb162, 5ed87c7, 92921ef, 84a4f4a)

**Push output (verbatim):**
```
$ git push origin main
To github.com:bg002h/seedhammer.git
   80d0c5d..84a4f4a  main -> main
```
Bypass message: **no.**

**origin/main confirmed moved:** `git fetch origin main` → `84a4f4a5d0e64665d3194766d2b7953b2f476fc0`,
matches local `HEAD`. MATCH.

**CI run:** `Test` workflow, run
[31921710920](https://github.com/bg002h/seedhammer/actions/runs/31921710920),
triggered on `84a4f4a5d0e64665d3194766d2b7953b2f476fc0`.

Per-job conclusions (via `gh api .../actions/runs/31921710920/jobs`):
- `tinygo-device-build`: **success**
- `tests`: **success**

Run-level: `status: completed`, `conclusion: success`.

(Note: the initial `gh run watch` was auto-backgrounded by the harness at the
120s mark and returned exit 124 from the outer `timeout 185` wrapper with
`tinygo-device-build` still in progress at that snapshot — matches the "Test
workflow is slow" expectation. A follow-up API poll shortly after showed both
jobs completed successfully, so no partial/unfinished status is being
reported here.)

---

## 2. `/scratch/code/shibboleth/mnemonic-engrave` — branch `master`

**Protection check:** `gh api repos/bg002h/mnemonic-engrave/branches/master/protection`
→ required context: `["test (rust + go)"]`. `ci/staging` procedure used.

**Commit range pushed:** `299b35400a9b4e1897c8d72f57638057a9fd493..c97f0aa059b36a72f112a87ef0234e1a507a54e7`
(12 commits: 299b354, 48fe616, 569fd82, 6d1c3b2, cfac0c6, dbed367, b93f6d4,
b4ddb3c, a748051, 30bd402, a960b07, c97f0aa)

**Step 1 — stage the SHA:**
```
$ git push origin master:refs/heads/ci/staging
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      ci/staging
```

**CI run on staged SHA:** `release` workflow, run
[31921809017](https://github.com/bg002h/mnemonic-engrave/actions/runs/31921809017),
`head_sha = c97f0aa059b36a72f112a87ef0234e1a507a54e7` (full 40-char SHA used
for the query).

Per-job conclusions (via `gh api .../actions/runs/31921809017/jobs`):
- `test (rust + go)`: **success**  ← required context, satisfied
- `build me (macos-x86_64)`: success
- `build me (macos-aarch64)`: success
- `build me (windows-x86_64)`: success
- `build me (linux-x86_64)`: success
- `build me (linux-aarch64)`: success
- `build me-preview (all targets)`: success
- `assemble + sign + release`: **skipped** ← confirmed, gated on `refs/tags/v*`,
  did not run from a `ci/**` push

Run-level: `status: completed`, `conclusion: success`.

**Step 2 — push master (verbatim):**
```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   db9e4e8..c97f0aa  master -> master
```
Bypass message: **no.**

**Step 3 — delete staging ref (verbatim):**
```
$ git push origin --delete ci/staging
 - [deleted]         ci/staging
```

**origin/master confirmed moved:** `git fetch origin master` →
`c97f0aa059b36a72f112a87ef0234e1a507a54e7`, matches local `HEAD`. MATCH.

**Stray `ci/*` refs:** `git ls-remote origin 'refs/heads/ci/*'` → empty. None
remain.

---

## Summary

| Repo | Branch | Old SHA | New SHA | Required check | Bypass? |
|---|---|---|---|---|---|
| seedhammer | main | `80d0c5d` | `84a4f4a5` | `Test` → tests: success, tinygo-device-build: success | no |
| mnemonic-engrave | master | `db9e4e8` | `c97f0aa0` | `test (rust + go)`: success; `assemble + sign + release`: skipped | no |
