# F-680 implementation report: aarch64 reproducibility checks on (md, ms, mk, toolkit)

Agent: Opus 5.5 implementer, 2026-09-24/25. Brief: `design/briefs/f680-aarch64-repro.md`.

## Outcome

All four branches are pushed. Every dispatched run concluded `success`, and in each one
`repro-aarch64-musl` **ran** (it was not skipped) and passed. The aarch64 negative check
found real residue in every repo (99 to 485 `/project` matches in the build with the remap
off). The same residue scanner found zero residue in the remapped build, and the A/B
double-build byte-compared real artifacts. Nothing was tagged, uploaded or merged, and no
default branch was pushed.

One gap: **the toolkit's `man-pages.yml` has no `workflow_dispatch` trigger** (it is
tag-only). So its changed `repro` caller cannot run without a tag. Instead I dispatched
`repro-drift.yml` on the same branch. It calls the same reusable workflow at the same commit
with `run_aarch64: true`. See Concerns.

## Toolkit pin

I used `4120af853ec68dc515cb4ac0eb7245a018a50c78` (toolkit `origin/master`). I checked the
ancestry rather than assuming it: `git merge-base --is-ancestor 327f9a16 4120af85` returns
true. Both old pins lack the fix: `6e37b18e` and `d39d9626` each fail
`--is-ancestor 327f9a16`. **The pin had to move in two places.** The fixed scripts
(`ci/repro/remap-off-negative.sh`, `cc-validate.sh`, `residue-lib.sh`) are checked out at
**`toolkit_ref`**, not at the `uses:` ref. Bumping only `uses:` would still have run the
unfixed checks. Each run log confirms the checkout: `HEAD is now at 4120af8 Merge
F-675/F-676/F-677` (md, ms, mk).

## Per repo

| repo | branch | commit | run id | workflow |
|---|---|---|---|---|
| descriptor-mnemonic | f680-aarch64 | `7dd66e5ce74e187f87ad4bf4a01601cc2286769e` | 36093609331 | man-pages.yml |
| mnemonic-secret | f680-aarch64 | `d35eabd80e8127970adbe333e1935d9ec0c10cda` | 36093611840 | man-release.yml |
| mnemonic-key | f680-aarch64 | `a2fcaab140206061ec2fa362f48c8b6aff932c25` | 36093614333 | musl-binaries.yml |
| mnemonic-toolkit | f680-aarch64 | `f9953f7dad4fef48ea2b50c60531e8a4d237e48b` | 36093616833 | repro-drift.yml (see Concerns) |

Worktrees: `/scratch/code/shibboleth/{dm,ms,mk,tk}-worktrees/f680`. I used `dm-`, not
`md-`, because `dm-worktrees` is where descriptor-mnemonic's existing worktrees are
registered.

### Per-leg conclusions

**descriptor-mnemonic 36093609331: run `success`**
- man-pages: success
- resolve-miniscript-rev: success
- repro / build-container: success
- repro / repro-substrate: success
- repro / repro-x86_64-musl: success (04:16:46 to 04:26:46)
- **repro / repro-aarch64-musl: success** (04:15:02 to 04:24:33)
- musl-binary (x86_64): success
- musl-binary (aarch64): success

**mnemonic-secret 36093611840: run `success`**
- derive git-source pins: success
- ms-man.tar.gz release asset: success
- build-container: success
- repro-substrate: success
- repro-x86_64-musl: success
- **repro-aarch64-musl: success** (04:14:50 to 04:18:35)
- musl-binary: skipped. This is by design: on dispatch the whole job is guarded off, and I left that guard alone.

**mnemonic-key 36093614333: run `success`**
- build-container: success
- repro-substrate: success
- repro-x86_64-musl: success
- **repro-aarch64-musl: success** (04:14:47 to 04:19:58)
- musl-binary: skipped. This is by design: the job has a tag-only `if:`, which I left alone.

**mnemonic-toolkit 36093616833 (repro-drift): run `success`**
- read git pins: success
- build-container: success
- repro-substrate: success
- repro-x86_64-musl: success
- **repro-aarch64-musl: success** (04:14:56 to 04:25:14)

### What changed

md, ms and mk each got one commit to their caller workflow:
- `run_aarch64: false` → `true`.
- The `uses:` pin moved to 4120af85.
- `toolkit_ref` moved to 4120af85.
- The "run_aarch64: false … exercised on demand via the toolkit reusable workflow's own
  `workflow_dispatch`" comment block was rewritten.
- "toolkit_ref is a CONCRETE SHA (P3a's / the F-324 enabling commit)" now names the F-675 fix.
- In ms, the historical F-324 sentence that cites `d39d9626` as the commit where
  `git_source_*` arrived is deliberately kept as history.

toolkit, one commit:
- `man-pages.yml`: `run_aarch64: true`, and the rationale for `false` is rewritten.
- `repro-drift.yml`: two comments said "the release path runs `run_aarch64: false`", which
  is now false. I rewrote them. The schedule stays, because it still covers the time
  between releases.
- **Outside caller workflows:** in `docs/verify-reproducibility.md` I edited one sentence
  in §8 that said "the *release* path (`man-pages.yml`) runs `run_aarch64: false`". The
  change made it false, so it counts as a finding under brief step 2. Nothing else outside
  the workflows was touched.

Not touched anywhere: `ensure-release`/`upload` steps and their tag guards, triggers, and
job-level `if:`s. `actionlint` on the four caller files is clean, and a YAML parse confirms
`run_aarch64: True` and the pins.

### Input diff (the reusable workflow's `workflow_call` inputs, old pin → 4120af85)

- **6e37b18e → 4120af85** (md, mk): the only change is two added inputs,
  `git_source_url` and `git_source_rev` (`type: string, required: false, default: ""`).
  Mirror entries were also added under `workflow_dispatch`. Neither caller needs them:
  - md's Cargo.lock has exactly one git source, the miniscript fork `ff4732e5`, which
    `miniscript_rev` already handles.
  - mk's Cargo.lock has no git sources.
  
  I added no new `with:` keys.
- **d39d9626 → 4120af85** (ms): the input blocks are **identical** (empty diff). ms already
  passes `git_source_url`/`git_source_rev`.
- toolkit: it calls `./` at its own commit, so no pin applies.

## Can-fail evidence

These lines are from the md `repro-aarch64-musl` log (job 107941197852). The remap-off
negative found residue:

```
== remap-off NEGATIVE (cross/aarch64): no-remap build MUST leak /project residue ==
  OK: no-remap cross build leaks the expected host-path residue (202 matches):
/project
== remap-off NEGATIVE PASSED: the remap/-ffile-prefix-map IS load-bearing (it strips this /project leak). ==
```

The residue counts in the other repos were: toolkit 485 matches, mk 107, ms 99.

With the remap on, the positive checks ran against the same artifacts:

```
== (i) binaries byte-identical ==
  OK: md identical across /build-a/src and /build-b/src
== (ii) tarballs byte-identical ==
  OK: .tar.gz identical
== (iii) libsecp .o byte-identical ==
  OK: libsecp .o identical (C frontier deterministic)
52bae65bbb9b5d3039e3fb7f25f8bf6e3ca043f007636b8f0cedd9d70936a0b4  md-0.0.0-repro-aarch64-linux-musl.tar.gz
== double-build GATE PASSED: byte-identical across two distinct paths ==
  OK: zero __DATE__/host-path residue in .o and binary.
== cc-validate GATE PASSED: musl-gcc deterministic, epoch honored-or-irrelevant, zero residue ==
```

The other aarch64 canonical hashes:
- mk: `9da88ca5…08dd7`
- ms: `e5e5bbcf…5c754`
- toolkit: `a1818c3b…c9fe0`

In mk, `(d)` also printed `.comment` = `GCC: (GNU) 9.2.0` ("OK: .comment carries the
expected pinned cross-toolchain string").

Why the zero-residue GREEN is not vacuous:
- cc-validate's `PATHS_RE` (`${ROOT}|/project|/build-a|/build-b|/home/|…/registry`, see
  cc-validate.sh:220) contains `/project`.
- It runs through the same `residue_hits` function (residue-lib.sh) as the negative check.
  Measured in the same job, that function returns 202 `/project` hits on the build with
  the remap off and zero on the build with it on.

Locally, `bash ci/repro/residue.test.sh` at 4120af85 reports `21 passed, 0 failed`. That
includes `cross, zero residue -> FAIL (exit 1)` and five repeats of `cross, heavy /project
residue -> PASS`, which is the SIGPIPE-race case.

## Concerns

1. **The toolkit's man-pages.yml caller was not executed** (Important for the "run the
   caller" gate, not a defect in the change). The workflow is tag-only. Adding
   `workflow_dispatch` would have run `man-tarball`'s release create/upload on a branch
   ref, because its upload steps are not dispatch-guarded. That is exactly the kind of
   upload-guard edit the brief forbids. The substitute evidence is repro-drift run
   36093616833 on the same commit. It calls the same `./.github/workflows/reproducible-musl-build.yml`
   with `run_aarch64: true`. The only difference is `toolkit_ref: ${{ github.sha }}` versus
   repro-drift's own pin inputs. The man-pages caller change will first execute at the next
   `mnemonic-toolkit-v*` tag. If you want it proven sooner, the fix is a dispatch trigger
   plus dispatch guards on man-tarball/upload. That is an operator call.
2. **The "~30-60 min QEMU" figure is stale** (Minor). Measured aarch64 durations were:
   - ms: 3m45s
   - mk: 5m11s
   - md: 9m31s
   - toolkit: 10m18s
   
   In every run it finished **before** the x86_64 leg, so enabling it added roughly zero
   wall time to the release path. The new comments still quote the old estimate, which
   came from the original text. The quantitative claim is therefore pessimistic, not
   unsafe. I left it rather than re-edit and re-run, and it is worth a doc touch-up.
3. The md dispatch's `musl-binary (aarch64)` tarball is versioned from Cargo.toml, while the
   repro leg uses `0.0.0-repro`. So their hashes cannot be compared on dispatch, and I did
   not compare them. Published-versus-gated hash equality is still only checked at a tag.
4. The branches are pushed but not merged. Once they merge, the next md/ms/mk/toolkit
   releases wait for the aarch64 gate, which is intended. The F-680 status line in
   FOLLOWUPS.md is left for the controller.
