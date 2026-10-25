r1: 0C/0I/1M/1N

# L3 review r1: mnemonic-engrave PR #21 (B-en), head cd0faf409fdf1a2f2954b2f9a904615ebc0406dc

Reviewer: Opus, independent whole-diff adversarial pass. Diff:
`git diff origin/master...cd0faf4` (release.yml +40/-4, plus the
implementation report). Spec: SPEC-after-b §4, §6, §7. Settled items in the
brief were not re-reviewed.

**The one question:** can the change skip `go-build` or `rust-build`'s build
steps on an event whose build output is used, or on a change that is not
documentation, or break the release (tag) path? **No.**

## Evidence

1. **Release path (tag, dispatch).** The only consumer of these jobs' output
   is `assemble` (`needs: [test, go-build, rust-build, picotool-probe]`,
   `if: startsWith(github.ref, 'refs/tags/v') || github.event_name ==
   'workflow_dispatch'`); `download-artifact` appears nowhere else, and
   release.yml is the repo's only workflow (`scripts/release-workflows/gen.py`
   generates other repos' workflows and is not run by CI). Ran the guard at
   the PR head for the three events where `assemble` runs:
   ```
   push refs/tags/v1.2.3              -> docs-only: full run (push to refs/tags/v1.2.3)   docs_only=false
   workflow_dispatch refs/heads/master -> docs-only: full run (event workflow_dispatch)    docs_only=false
   workflow_dispatch refs/tags/v1.2.3  -> docs-only: full run (event workflow_dispatch)    docs_only=false
   ```
   So every build, stage and upload step runs whenever `assemble` runs. The
   guard step has no `if:` and engrave has no `tag` input, as §4.1 says.
2. **Steps that must still run on a docs-only change.** None in either job.
   The guarded steps are toolchain setup, version extraction, compile, the
   lockstep version check (reads only `crates/me-cli/Cargo.toml` and the built
   binary, neither docs), staging and `upload-artifact`. No attestation,
   signing or required check lives in these jobs; attestation and signing
   are in `assemble`, whose `permissions` (workflow-level) are untouched.
   The only side effect lost on a docs-only run is a `setup-go` cache save,
   which is not a correctness concern.
3. **`CHECK_NAME` on the matrix.** `CHECK_NAME: build me (${{ matrix.name }})`
   expands per leg to exactly the job's rendered name; the five `matrix.name`
   values are distinct, so each leg looks up only its own check run on R
   (exact `check_name` match). L2's logs (run 40226736800) show each leg
   quoting its own name. A leg whose run on R was cancelled or failed gets a
   non-success conclusion and runs in full; `rust-build` has no job-level
   `if:` or `needs:`, so it is never job-skipped on R; a leg docs-skipped on R
   concluded success, which is §4's accepted induction. No other workflow
   emits these names.
4. **Permissions.** `contents: read` covers checkout (the `seedhammer`
   submodule is a public https URL) and the guard's compare call; `checks:
   read` covers the check-runs lookup. `setup-go` caching and
   `upload-artifact@v4` use the runner's runtime token, not `GITHUB_TOKEN`
   scopes. Nothing in either job needs `contents: write`, `id-token` or
   `attestations`. Empirically: L1 on the code head built and uploaded every
   artifact under these permissions, and the final head's checks are all
   pass (run 40226952904, `assemble` skipping as allowed).
5. **`if:` composition.** Checked by eye against §4.2 step 5 (the controller
   also diffed it): `use_cross` steps and both `${{ … }}` forms keep their
   original operand and gain only `&& steps.docs.outputs.docs_only != 'true'`.
   No leg loses its one build step on a code change.

## Critical

None.

## Important

None.

## Minor

- M1 (PR description): it says "This PR changes `.github/workflows/` and
  `ci/`", but the PR touches no `ci/` file (only `.github/workflows/release.yml`
  and `design/agent-reports/ci-after-b-build.md`). It also describes only rule
  A′, while the guard equally skips these jobs on a PR whose whole diff from
  its base is docs and on docs-only pushes to `master` and `ci/**`. Fix:
  "This PR changes `.github/workflows/`", and name all three skip cases.

## Nit

- N1: `design/agent-reports/ci-after-b-build.md` starts with an empty line
  before its heading.
