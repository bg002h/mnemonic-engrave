# Brief — sign every release's checksums with minisign, and verify in the installer

Operator (2026-09-25): "I want all of it done." Today only mnemonic-engrave's
releases are signed (`SHA256SUMS` + `SHA256SUMS.minisig`). The toolkit, ms, md
and mk release workflows already contain a dormant signing step for
`SHA256SUMS.portable` (same public key as engrave,
`RWQPmgBXsuw5yi8W0SfDr8KF+IqY/Z5U2p724emSODS1UPfJBP3agbKW`), which runs only
when `MINISIGN_SECRET_KEY` is set. **As of 2026-09-26 that secret is now set
in mnemonic-toolkit, mnemonic-secret, descriptor-mnemonic, mnemonic-key and
mnemonic-gui**, and the operator is setting `MINISIGN_SECRET_KEY_PASSWORD` in the
same five. One key for everything, the one engrave already uses.

**Never read, print or copy the secret key or password.** They live only in
GitHub secrets and the operator's `~/.minisign/`.

## Do

1. **Sign the Linux checksums.** `SHA256SUMS.x86_64` and `SHA256SUMS.aarch64`
   are produced by the reproducible musl pipeline: toolkit
   `.github/workflows/man-pages.yml` plus the reusable
   `reproducible-musl-build.yml`, called from ms, md, mk and the toolkit. Add
   signing so each gets a `.minisig`. This is the reproducibility path: signing
   must happen AFTER the digest is final and must not alter any artifact the
   repro gate hashes. Keep secrets out of the containerised build: sign on the
   host step that uploads. If the reusable workflow is the right place, callers
   must pass the secrets explicitly (`secrets:`); say which approach, and why.
2. **GUI:** add the same signing step to mnemonic-gui's release workflow for
   its `SHA256SUMS`.
3. **Every signing step, in every repo:** after signing, **verify** the
   signature against the pinned public key in the same job, and fail the job
   if it doesn't verify. If `MINISIGN_SECRET_KEY` is absent, keep today's
   behaviour (ship unsigned, say so), but only for workflow_dispatch/branch
   runs. On a **release tag**, a missing secret or failed signature must
   **fail** the release, so it can't silently go out unsigned now that the
   secrets exist. Say exactly what each repo does in each case.
4. **Installer** (toolkit `scripts/install.sh`): when `minisign` is available,
   verify the downloaded `SHA256SUMS*` file's `.minisig` against the pinned
   public key before trusting any checksum in it. Refuse on a bad signature,
   and also on a MISSING signature for a release pinned at or after the first
   signed version of each tool. Keep a table of "first signed tag" per
   component, filled in when those releases exist; until then, missing is
   allowed with a note. When `minisign` is not installed, say once that
   signatures weren't checked, and continue. Add `--require-signature` to make
   missing minisign or signature fatal. Tests: a good signature, a bad signature
   (tampered sums file), a missing signature before and after the first-signed
   pin, and minisign absent. Use a THROWAWAY test keypair generated in the
   test, never the real key. Document it in `--help` and the manual.
5. **Prove it before any real release:** in each repo, run the release
   workflow via workflow_dispatch (or on a `ci/` ref) in a mode that signs and
   verifies but does NOT create or upload to a release. If the workflow has no
   such mode, add one. Record run ids showing sign + verify succeeded with the
   real secret. If `MINISIGN_SECRET_KEY_PASSWORD` isn't set yet when you get
   there, report which repos are waiting on it and continue with the rest.

## Where

Branch `sign-releases` in each changed repo, in worktrees under
`/scratch/code/shibboleth/<prefix>-worktrees/sign`. Don't push default branches,
merge or tag. Obey F-695 (no `producer | grep -q` under pipefail). The shell is
zsh; an `rm -rf` guard blocks recursive forced rm (use `find … -delete`).

## Deliverable

Commits ending with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
**As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/sign-all-releases.md`**:
per repo, what signs what and where; the tag-vs-dispatch behaviour matrix; the
installer design and tests; the dispatch run ids proving sign + verify. Return a
short summary plus the path.
