# F-324 push + re-pin report

Agent run: PUSH + RE-PIN closing `design/FOLLOWUPS.md` F-324 and its
2026-09-02 addendum. Verbatim commands, SHAs, run ids, per-job conclusions,
and push outputs below. Nothing in this report is summarized from memory —
every conclusion was queried from `gh` at the time recorded.

## Step 1 — mnemonic-toolkit

Worktree: `/scratch/code/shibboleth/wt-toolkit-f324`, branch `f324-git-source`,
tip `d39d96269ce352270189c11fabebf9ad070362b4` (two signed commits over local
`master` `d8f06483b82a4c0f0b8da86aeb50d31942c31481`).

**Anomaly found before pushing.** `origin/master` was NOT at `d8f06483` — it
was one commit behind, at `d6277006c285537d3d067e7b9caee8a9bc794c4e`
(`git fetch origin master && git rev-parse origin/master` → `d6277006…`).
`d8f06483` ("report: persist constellation install verification 2026-08-31")
existed only as an unpushed local commit on the shared `master` ref (`git
branch -vv` showed `[origin/master: ahead 1]`). `d6277006` is an ancestor of
`d8f06483` (`git merge-base --is-ancestor d6277006… d8f06483…` → true), so
landing the worktree's `HEAD` on `origin/master` is still a clean fast-forward;
it just also carries that one pre-existing, unrelated report commit along.
Not a violation of the "master moved / non-fast-forward" stop condition (that
condition is about `master` moving *forward past* the base during the push
window, not about `origin` trailing local `master`). Flagged here rather than
silently absorbed.

**Branch protection (measured):**
```
gh api repos/bg002h/mnemonic-toolkit/branches/master/protection
```
→ `required_status_checks.contexts = ["examples", "test (ubuntu-latest)", "clippy"]`,
`strict: false`. No repository rulesets exist (`gh api
repos/bg002h/mnemonic-toolkit/rulesets` → `[]`).

**`ci/staging` push produced NO required-context runs.** The two F-324
commits touch only `.github/workflows/reproducible-musl-build.yml`,
`ci/repro/double-build.sh`, `ci/repro/cc-validate.sh`,
`ci/repro/remap-off-negative.sh`. `rust.yml` (owns `test (ubuntu-latest)` +
`clippy`) and `examples.yml` (owns `examples`) both gate their `push:` trigger
(even for `branches: [..., 'ci/**']`) behind a `paths:` filter
(`crates/**`, `.gitattributes`, `Cargo.toml`, `Cargo.lock`,
`.github/workflows/{rust,examples}.yml`) that this diff does not touch, and
neither workflow has a `workflow_dispatch` trigger. So:

```
$ git push origin HEAD:refs/heads/ci/staging
 * [new branch]        HEAD -> ci/staging
$ gh run list --repo bg002h/mnemonic-toolkit --commit d39d96269ce352270189c11fabebf9ad070362b4 --json databaseId,name,status,conclusion
[{"conclusion":"failure","databaseId":33623284180,"name":"sibling-pin-check","status":"completed"}]
```
Only `sibling-pin-check` ran (it has no `paths:` filter). `rust` and
`examples` never fired on this SHA via the push event — **as anticipated by
the brief's fallback clause** ("If the toolkit's CI on `ci/**` does not run
for a branch push … say so").

**`sibling-pin-check` failure investigated and ruled pre-existing/unrelated.**
`gh run view 33623284180 --repo bg002h/mnemonic-toolkit --log-failed` →
`##[error]sibling-pin-check: .github/workflows/cross-tool-differential.yml:80:
pin 'descriptor-mnemonic-md-cli-v0.11.2' (url
https://github.com/bg002h/descriptor-mnemonic) does not match
scripts/install.sh canonical 'descriptor-mnemonic-md-cli-v0.14.0'`. Confirmed
already broken on `master` before this session touched anything: the same
check failed at `d6277006` (`databaseId 33396338700`, `conclusion: failure`,
already an ancestor of local `master`). `sibling-pin-check` is not a required
context (not in the branch-protection list above). Out of scope for F-324;
not fixed here.

**Resolution: a staging PR, not a staging push, since `pull_request:` has no
`paths:` filter on either `rust.yml` or `examples.yml`.**
```
$ gh pr create --repo bg002h/mnemonic-toolkit --base master --head ci/staging \
    --title "ci: F-324 git_source_url/git_source_rev inputs (staging PR to trigger required checks)" \
    --body "Staging PR only — triggers pull_request-scoped required checks …"
https://github.com/bg002h/mnemonic-toolkit/pull/68
```

Per-job conclusions for `d39d96269ce352270189c11fabebf9ad070362b4`, queried
via `gh api repos/bg002h/mnemonic-toolkit/commits/<sha>/check-runs` after the
PR's checks completed (verbatim `name: status/conclusion`, one line per
check-run; `sibling pins match install.sh` appears 3× — once for each event
that ran it, push/pull_request/pull_request-sync — all the SAME pre-existing
finding):

```
sibling pins match install.sh: completed/failure   (×3, pre-existing, not required)
lib cross-platform check (x86_64-pc-windows-msvc, windows-latest): completed/success
lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest): completed/success
g6 invariant (cross-repo mlock.rs): completed/success
lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest): completed/success
clippy: completed/success                          <- REQUIRED
miri (mlock unsafe): completed/success
test (ubuntu-latest): completed/success             <- REQUIRED
musl build+test (aarch64-unknown-linux-musl): completed/success
test (macos-latest): completed/success
test (release, ubuntu-latest, mlock einval): completed/success
musl build+test (x86_64-unknown-linux-musl): completed/success
install.sh harnesses (man-step + MSRV guard): completed/success
fmt (pinned 1.95.0): completed/success
examples: completed/success                         <- REQUIRED
```

All three required contexts `success` for the exact SHA.

**Master push:**
```
$ git fetch origin master && git rev-parse origin/master
d6277006c285537d3d067e7b9caee8a9bc794c4e
$ git push origin HEAD:master
To github.com:bg002h/mnemonic-toolkit.git
   d6277006..d39d9626  HEAD -> master
```
No "Bypassed rule violations" text in the output — the push was accepted on
the strength of the required-context checks recorded above, not an admin
bypass.

```
$ git push origin --delete ci/staging
 - [deleted]           ci/staging
$ gh pr close 68 --repo bg002h/mnemonic-toolkit --comment "..."
X Pull request bg002h/mnemonic-toolkit#68 … can't be closed because it was already merged
```
GitHub auto-recognized PR #68 as merged once its head commit reached `master`
via the fast-forward push (no merge commit was created; `master` stayed
linear — verified: `git log --oneline -3 origin/master` shows `d39d9626 →
21b6696e → d8f06483`, no merge parent).

```
$ git fetch origin && git rev-parse origin/master
d39d96269ce352270189c11fabebf9ad070362b4
```

**TOOLKIT_SHA = `d39d96269ce352270189c11fabebf9ad070362b4`**, confirmed on
`origin/master`.

## Step 2 — mnemonic-secret

Checkout `/scratch/code/shibboleth/mnemonic-secret`, `master` at
`1068f389116928e4cd22e5b0658749d09b06611d`, `git status --short` empty
(confirmed clean before branching).

**Branch protection (measured):**
```
gh api repos/bg002h/mnemonic-secret/branches/master/protection
```
→ `required_status_checks.contexts = ["test (ubuntu-latest)", "clippy",
"test (ms-codec)", "clippy (ms-codec)"]`, `strict: false`.

Created `f324-repin` from `master`. Edited
`.github/workflows/man-release.yml`:

1. Added job `pins` (runs-on ubuntu-latest, `actions/checkout@v4` — matching
   this file's own unpinned-tag style for `actions/checkout`), one step
   deriving `io_lib_rev` with the exact `ci/repro/vendor-freshness.sh`
   expression (`grep -oE 'mnemonic-engrave\?rev=[0-9a-f]{40}' Cargo.lock |
   head -1 | grep -oE '[0-9a-f]{40}'`), failing closed (`exit 1`) on empty.
2. On `repro:`: added `needs: pins`; changed `uses:
   bg002h/mnemonic-toolkit/.github/workflows/reproducible-musl-build.yml@6e37b18e50f9f857e439db1ebe2748fc91a54612`
   → `@d39d96269ce352270189c11fabebf9ad070362b4`; `toolkit_ref` likewise;
   added `git_source_url: https://github.com/bg002h/mnemonic-engrave` and
   `git_source_rev: ${{ needs.pins.outputs.io_lib_rev }}` under `with:`
   (semantics confirmed by reading the toolkit's own commit `21b6696e`: the
   caller passes the bare URL with no `git+`/`?rev=` decoration — the
   reusable workflow builds that itself).
3. Rewrote the `# ── F-324: …` header comment block to state the fix, cite
   `ms-cli-v0.17.0`'s failed run `33621228397` by id, name TOOLKIT_SHA and its
   two commits, and state F-324 closes once a `workflow_dispatch` run reports
   the `repro` job green (below).
4. Confirmed `musl-binaries` untouched: `needs: repro` and
   `REPRO_IMAGE: ${{ needs.repro.outputs.image }}` both still present,
   unedited (`git diff` shows no hunk touching that job).

**Validation:**
```
$ actionlint .github/workflows/man-release.yml
(exit 0, no output)
$ ruby -ryaml -e "YAML.load_file(ARGV[0]); puts 'YAML OK'" .github/workflows/man-release.yml
YAML OK
```

Diff confined to one file: `1 file changed, 59 insertions(+), 25
deletions(-)`. Committed once, signed:
```
commit a069c77c9ec98f6d0b6972d295c4da548db9d2fc
ci: re-pin man-release.yml's repro job to the toolkit's F-324 fix
```

**Staging push** (this repo's `rust.yml` has NO `paths:` filter on push, by
design, per its own header comment — so the ritual needs only the direct
`ci/staging` push, no PR detour):
```
$ git push origin HEAD:refs/heads/ci/staging
 * [new branch]      HEAD -> ci/staging
```

Per-job conclusions for `a069c77c9ec98f6d0b6972d295c4da548db9d2fc`
(`gh api repos/bg002h/mnemonic-secret/commits/<sha>/check-runs`, de-duplicated
— the SHA accrued two identical passes, one from the `ci/staging` push event
and one re-surfaced by the later `workflow_dispatch` run's shared commit
context; conclusions identical both times):

```
clippy (ms-codec): completed/success                <- REQUIRED
test (ms-codec): completed/success                   <- REQUIRED
test (ubuntu-latest): completed/success               <- REQUIRED
clippy: completed/success                             <- REQUIRED
fmt (pinned 1.95.0): completed/success
freebsd compile-gate (whole-crate): completed/success
test (release, ubuntu-latest, mlock einval): completed/success
miri (mlock unsafe): completed/success
test (macos-latest): completed/success
g6 invariant (cross-repo mlock.rs): completed/success
musl compile/test (x86_64-unknown-linux-musl): completed/success
musl compile/test (aarch64-unknown-linux-musl): completed/success
history purge (recipes RUN under real shells): completed/success
```
(`vendor-freshness` did not run for this SHA — its `paths:` filter does not
include `.github/workflows/man-release.yml`; it is not a required context, so
this is expected and immaterial.)

All four required contexts `success`.

**Master push:**
```
$ git fetch origin master && git rev-parse origin/master
1068f389116928e4cd22e5b0658749d09b06611d
$ git checkout master && git merge --ff-only f324-repin
Updating 1068f38..a069c77
Fast-forward
 .github/workflows/man-release.yml | 84 +++++++++++++++++++++++++++------------
 1 file changed, 59 insertions(+), 25 deletions(-)
$ git push origin master
To github.com:bg002h/mnemonic-secret.git
   1068f38..a069c77  master -> master
```
No "Bypassed rule violations" text.
```
$ git push origin --delete ci/staging
 - [deleted]         ci/staging
$ git fetch origin && git rev-parse origin/master
a069c77c9ec98f6d0b6972d295c4da548db9d2fc
```

**MS_SHA = `a069c77c9ec98f6d0b6972d295c4da548db9d2fc`**, confirmed on
`origin/master`. Local branch `f324-repin` deleted after the ff-merge.

## Step 3 — exercise the gate, then the 0.17.0-binaries question

**Gate run:**
```
$ gh workflow run man-release.yml --repo bg002h/mnemonic-secret --ref master
https://github.com/bg002h/mnemonic-secret/actions/runs/33624724552
```
(`workflow_dispatch` has no declared inputs on this workflow — a bare
dispatch, matching the header comment's "A bare 'Run workflow' click drives
the `repro` caller job".)

Final per-job conclusions (`gh run view 33624724552 --repo
bg002h/mnemonic-secret --json status,conclusion,jobs`):

```
run status: completed / conclusion: success

derive git-source pins:                                    completed/success
ms-man.tar.gz release asset:                                completed/success   (all 5 tag-gated steps SKIPPED — verified individually: Install Rust, Build the ms binary, Generate man pages + tarball, Ensure a release exists, Upload ms-man.tar.gz all "skipped"; only checkout ran)
repro / build-container (resolve BUILT-DIGEST):             completed/success
repro / repro-substrate (x86_64-unknown-linux-musl):        completed/success
repro / repro-x86_64-musl (x86_64-unknown-linux-musl):      completed/success
repro / repro-aarch64-musl (aarch64-unknown-linux-musl):    completed/skipped   (run_aarch64: false, as configured)
musl-binary (${{ matrix.target }}):                         completed/skipped   (if: startsWith(github.ref, 'refs/tags/ms-cli-v') — false on a branch dispatch)
```

`repro-substrate` and `repro-x86_64-musl` — the two jobs that exercise the new
`git_source_url`/`git_source_rev` stanza against ms's actual
`mnemonic-io-lib` pin — both `success`. The derived rev used in this run:
`IO_LIB_REV = 6c24e62823e6c1ac02aa3862cd6020674bf58544` (read directly from
`Cargo.lock` on `master`: `grep -oE 'mnemonic-engrave\?rev=[0-9a-f]{40}'
Cargo.lock` → `mnemonic-engrave?rev=6c24e62823e6c1ac02aa3862cd6020674bf58544`).

**F-324's gate has now executed green**, per the header comment's own closure
condition ("closed once a `workflow_dispatch` run of this workflow reports
the `repro` job green end-to-end against this config").

### The 0.17.0-binaries question

Checked whether `gh run rerun 33621228397 --repo bg002h/mnemonic-secret
--failed` (the original `ms-cli-v0.17.0` tag run) could ever produce the
missing binaries. **It cannot, and it was NOT executed.**

- The tag `ms-cli-v0.17.0` is an annotated tag object
  (`7bbd810b6376e7a14eecf18739b316d17c282908`) dereferencing to commit
  `1068f389116928e4cd22e5b0658749d09b06611d`
  (`gh api repos/bg002h/mnemonic-secret/git/tags/7bbd810b… --jq
  '.object.sha'` → `1068f389…`). Tags are immutable in this repo's ritual
  (stated in the brief and not challenged here).
- GitHub Actions re-runs a workflow using the workflow YAML **as committed at
  the SHA that triggered the run**, not the current content of the ref. A
  re-run of `33621228397` would therefore re-read `man-release.yml` at
  `1068f389…` — the pre-fix version, with `uses:
  …@6e37b18e50f9f857e439db1ebe2748fc91a54612` and no `git_source_url`/
  `git_source_rev` — and fail identically (`repro-substrate` /
  `repro-x86_64-musl` red exactly as `33621228397` first reported, both
  `musl-binary` legs skipped again).
- Confirmed the release's current asset set is still exactly the
  under-published one: `gh release view ms-cli-v0.17.0 --repo
  bg002h/mnemonic-secret --json assets --jq '.assets[].name'` →
  `ms-man.tar.gz` only. For comparison, `ms-cli-v0.16.0`'s assets:
  `ms-0.16.0-aarch64-linux-musl.tar.gz`, `ms-0.16.0-x86_64-linux-musl.tar.gz`,
  `ms-man.tar.gz`, `PROVENANCE.aarch64.txt`, `PROVENANCE.x86_64.txt`,
  `SHA256SUMS.aarch64`, `SHA256SUMS.x86_64` (7 assets, matching the
  FOLLOWUPS.md addendum's count).
- On a `workflow_dispatch` run (the only trigger that reads the CURRENT
  `master` workflow content), `musl-binaries` is unconditionally skipped
  (`if: startsWith(github.ref, 'refs/tags/ms-cli-v')` is false for a branch
  ref) and the man-tarball job's release-upload steps are likewise
  tag-gated-off (confirmed above: all 5 skipped). So **no dispatch, however
  configured, can populate a release** — only a `push: tags: ["ms-cli-v*"]`
  event does, and only if that tag's commit carries the fix.

**Conclusion: `ms-cli-v0.17.0`'s release cannot be completed as-is.** The
binaries for that exact tag/release are unreachable without either (a)
retagging (forbidden — tags immutable here) or (b) a new tag on a commit that
carries the fix, which produces a *new* release rather than completing the
old one.

**Proposed remedy (NOT executed — controller decides):** cut
**`ms-cli-v0.17.1`** as a patch tag on MS_SHA
(`a069c77c9ec98f6d0b6972d295c4da548db9d2fc`), with a CHANGELOG.md entry in
this repo's established format (matching the existing `## ms-cli [0.17.0] —
2026-09-02` style):

```
## ms-cli [0.17.1] — <date>

**Release-infrastructure only: F-324. No code change.**

`man-release.yml`'s `repro` job re-pinned to mnemonic-toolkit's
git_source_url/git_source_rev inputs so the reproducible musl build can
resolve the mnemonic-io-lib git pin --offline. 0.17.0's tag run
(33621228397) failed repro-substrate/repro-x86_64-musl and published only
ms-man.tar.gz; this tag exists so the missing musl-binary/PROVENANCE/
SHA256SUMS assets exist under a resolvable tag.
```

One implementation nuance not itself a decision: this repo keeps
`crates/ms-cli/Cargo.toml`'s `version` field in lockstep with its tag
(confirmed: `version = "0.17.0"` at `1068f389`, matching `ms-cli-v0.17.0`), so
"no code change" would need qualifying to "no functional code change" if the
version field is bumped to `0.17.1` for consistency — a one-line
Cargo.toml/Cargo.lock diff, not a behavior change. Left for whoever executes
the remedy to decide alongside the tag itself.

## CI-red stop condition

Not triggered. No step in this run went red on a required context; the one
observed failure (`sibling-pin-check` on toolkit) was independently confirmed
pre-existing on `master` before this session started and is not a required
context, so it did not block either push.

## Summary of ending state

- `origin/bg002h/mnemonic-toolkit` `master` = `d39d96269ce352270189c11fabebf9ad070362b4` (TOOLKIT_SHA).
- `origin/bg002h/mnemonic-secret` `master` = `a069c77c9ec98f6d0b6972d295c4da548db9d2fc` (MS_SHA).
- Both repos' `ci/staging` branches deleted; no leftover PRs open (toolkit
  PR #68 shows merged, not open).
- mnemonic-secret's F-324 gate has executed green once
  (`workflow_dispatch` run `33624724552`).
- `ms-cli-v0.17.0`'s release remains short its musl binaries/PROVENANCE/
  SHA256SUMS; remedy proposed above, not executed.
- The main mnemonic-toolkit checkout at
  `/scratch/code/shibboleth/mnemonic-toolkit` was not touched (its unrelated
  uncommitted files remain exactly as found).
