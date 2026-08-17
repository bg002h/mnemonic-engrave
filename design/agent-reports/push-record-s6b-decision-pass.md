# Push record — S6b decision pass (6 commits)

## SHA staged

`27bd49b15f342e1996d24477ac47882ac64e26bb`

Verified at start: `git rev-parse HEAD` == `27bd49b15f342e1996d24477ac47882ac64e26bb` (matched the frozen tip stated in the dispatch brief).

Six commits pushed, all design/registry documents, no code:

```
27bd49b s6b: record R-F and R-G -- the decision pass is CLOSED
b9998fb s6b: record R-E (fadeClip stays stubbed) and settle Q3 against Rust mk
6d0de2d reports: persist the mk key-identifier research, verbatim
f32edef s6b: correct section 2's false facts, and propose Q1/Q3/Q6
3922fcf reports: persist the S6b plate-mechanism recon, verbatim
36c5667 reports: persist the S6b rulings push record -- check SATISFIED
```

(Full ancestry back to `origin/master` pre-push at `a776358`, which also
includes `a776358 s6b: record the three operator rulings from the decision
pass` as the 7th commit ahead — but the brief describes the top six; all are
design/registry docs, no code.)

## Workflow run

- Trigger: `git push origin master:refs/heads/ci/staging` (new branch `ci/staging` created at `27bd49b`)
- Repo: `bg002h/mnemonic-engrave`
- Workflow: `release`
- Run ID: `32056104070`
- URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/32056104070
- Run-level: `status: completed`, `conclusion: success`
- `headSha` on the run: `27bd49b15f342e1996d24477ac47882ac64e26bb` (matches staged SHA)

## Per-job conclusions (verbatim, from `gh run view 32056104070 --repo bg002h/mnemonic-engrave --json jobs`)

```json
{"conclusion":"success","name":"test (rust + go)","status":"completed"}
{"conclusion":"success","name":"build me-preview (all targets)","status":"completed"}
{"conclusion":"success","name":"build me (linux-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me (macos-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me (macos-aarch64)","status":"completed"}
{"conclusion":"success","name":"build me (windows-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me (linux-aarch64)","status":"completed"}
{"conclusion":"skipped","name":"assemble + sign + release","status":"completed"}
```

The required context, `test (rust + go)`, is `success`.

`assemble + sign + release` is `skipped` — confirms it did not fire on a
`ci/**` push (it is gated on `refs/tags/v*`, and this push carried no tag).

## Final push to `master` — exact output

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   a776358..27bd49b  master -> master
```

No "Bypassed rule violations" string appeared. The required-check gate was
**SATISFIED**, not bypassed.

## `ci/staging` deletion

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

## Positive control — `ci/staging` absent, `master` present, same query

```
$ git ls-remote origin master ci/staging refs/heads/ci/staging
27bd49b15f342e1996d24477ac47882ac64e26bb	refs/heads/master
```

`master` resolves to the pushed SHA; `ci/staging` returns no line in the same
output (not a separate empty query) — confirms deletion, not an unrelated
empty result.

## Verdict

**SATISFIED**
