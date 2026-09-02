# F-324 toolkit implementation report — generic `git_source_url`/`git_source_rev` on `reproducible-musl-build.yml`

**Repo:** mnemonic-toolkit. **Worktree:** `/scratch/code/shibboleth/wt-toolkit-f324`
(branch `f324-git-source`, off HEAD `d8f06483`; main checkout untouched, per
brief). **Commit:** `21b6696e09159bc4b52259f32f5b13ca1f037d06` (signed-off,
not pushed).

## What was done

Exactly the scope in the dispatch brief: two files, one commit.

### `git show --stat`

```
 .github/workflows/reproducible-musl-build.yml | 79 ++++++++++++++++++++++++++-
 ci/repro/double-build.sh                      | 19 +++++++
 2 files changed, 97 insertions(+), 1 deletion(-)
```

### Every site changed (line numbers are POST-change, i.e. in the committed file)

`.github/workflows/reproducible-musl-build.yml`:
- L56–68: new header block "F-324: A SECOND, GENERIC git-source knob" beside
  the existing job-scoped-`[source]`-activation comment.
- L124–138: `git_source_url` input in the `workflow_call` `inputs:` block,
  right after `miniscript_rev`. L139–146: `git_source_rev` input, right
  before `toolkit_ref` (both `type: string`, `required: false`,
  `default: ""`).
- L210–217: `git_source_url`, L218–223: `git_source_rev`, added to the
  `workflow_dispatch` `inputs:` block (shorter descriptions, matching that
  block's existing convention).
- L384–385: `GIT_SOURCE_URL`/`GIT_SOURCE_REV` added to `repro-substrate`
  job's `env:`, beside `MINISCRIPT_REV`.
- L410–418: the positive `repro-substrate-positive` step's `SRC_CONFIG`
  assembly — new `if [ -n "$GIT_SOURCE_URL" ] && [ -n "$GIT_SOURCE_REV" ]`
  block (L412–418, comment L410–411) appended after the miniscript stanza's
  closing `fi` (L409), before `source.vendored-sources.directory` (L419).
- L427: comment on the negative step updated to note the git_source blocks
  are also kept (not the isolated block).
- L449–457: the negative `repro-substrate-negative` step's `NEG_CONFIG`
  assembly — identical conditional block (L451–457) added right before the
  `NOTE: source.vendored-sources.directory is DELIBERATELY omitted` comment
  (L458); only that directory line stays isolated as the thing under test.
- L499–500: `GIT_SOURCE_URL`/`GIT_SOURCE_REV` added to `repro-x86_64-musl`
  job's `env:`.
- L662–663: `GIT_SOURCE_URL`/`GIT_SOURCE_REV` added to `repro-aarch64-musl`
  job's `env:`.

`ci/repro/double-build.sh`:
- L48–49: `MINISCRIPT_REV` / `GIT_SOURCE_URL`/`GIT_SOURCE_REV` documented in
  the header's `ENV:` list.
- L117–118: `GIT_SOURCE_URL="${GIT_SOURCE_URL:-}"` /
  `GIT_SOURCE_REV="${GIT_SOURCE_REV:-}"` (double-dash default, per brief —
  unlike `MINISCRIPT_REV`'s single-dash trick at L109, there is no sensible
  non-empty default rev for an arbitrary caller dependency).
- L129–135: `if [ -n "$GIT_SOURCE_URL" ] && [ -n "$GIT_SOURCE_REV" ]` block
  appended after the miniscript stanza's closing `fi` (L128), before
  `source.vendored-sources.directory` (L136–138), mirroring its exact
  quoting (`.git=`, `.rev=`, `.replace-with="vendored-sources"`).

Nothing else touched: `Dockerfile.repro`, `cc-validate.sh`'s own logic beyond
what's flagged below, `vendor-freshness.sh`, and no negative-test semantics
changed (see below).

## Validation run

```
$ bash -n ci/repro/double-build.sh
OK

$ actionlint .github/workflows/reproducible-musl-build.yml
(exit 0, no output — clean)

$ ruby -ryaml -e "YAML.load_file('.github/workflows/reproducible-musl-build.yml'); puts 'OK: YAML parses'"
OK: YAML parses
```

`python3` has no `pyyaml` installed on this box (`ModuleNotFoundError`) —
used Ruby's stdlib `YAML.load_file` instead (Ruby is on PATH); it parsed the
full file cleanly.

`ci/repro/double-build.sh` has no dry/echo mode, so per the brief's fallback
I extracted the SRC_CONFIG-building lines into a scratch script
(`/tmp/.../scratchpad/render-stanza.sh`) and rendered the array with
`printf '%s\n'` under three cases:

**Case 1 — F-324's actual scenario** (`MINISCRIPT_REV=""`,
`GIT_SOURCE_URL=https://github.com/bg002h/mnemonic-engrave`,
`GIT_SOURCE_REV=6c24e62823e6c1ac02aa3862cd6020674bf58544`):
```
--config source.crates-io.replace-with="vendored-sources"
--config source."git+https://github.com/bg002h/mnemonic-engrave?rev=6c24e62823e6c1ac02aa3862cd6020674bf58544".git="https://github.com/bg002h/mnemonic-engrave"
--config source."git+https://github.com/bg002h/mnemonic-engrave?rev=6c24e62823e6c1ac02aa3862cd6020674bf58544".rev="6c24e62823e6c1ac02aa3862cd6020674bf58544"
--config source."git+https://github.com/bg002h/mnemonic-engrave?rev=6c24e62823e6c1ac02aa3862cd6020674bf58544".replace-with="vendored-sources"
--config source.vendored-sources.directory="vendor"
```

**Case 2 — everything unset** (toolkit's own default): unchanged three-block
form (crates-io + miniscript fork + vendored-sources.directory) — byte
identical to pre-F-324.

**Case 3 — codec caller today** (`MINISCRIPT_REV=""`, git_source unset, the
shape `md`/`mk`/`ms` currently pass):
```
--config source.crates-io.replace-with="vendored-sources"
--config source.vendored-sources.directory="vendor"
```
Byte identical to the pre-F-324 two-block form — confirms the change is a
strict addition, not a behavior change, for every existing caller.

## Other callers checked — confirmed unaffected

- `descriptor-mnemonic/.github/workflows/man-pages.yml`,
  `mnemonic-key/.github/workflows/musl-binaries.yml`: both pass
  `miniscript_rev: ""` and nothing for `git_source_url`/`git_source_rev` →
  new inputs default to `""` → Case 3 above → unaffected.
- The toolkit's own in-repo callers, `.github/workflows/man-pages.yml` (L105)
  and `.github/workflows/repro-drift.yml` (L74), both pass
  `miniscript_rev: "95fdd1c5773bd918c574d2225787973f63e16a66"` and nothing
  for the new inputs → Case 2 above → unaffected.
- `mnemonic-secret/.github/workflows/man-release.yml`'s `repro:` job (the
  actual F-324 caller) is unchanged by this commit — it still pins
  `toolkit_ref: 6e37b18e50f9f857e439db1ebe2748fc91a54612` and passes neither
  new input, so it will keep failing exactly as before **until** it is
  re-pinned to a `toolkit_ref` including this commit and passes
  `git_source_url`/`git_source_rev`. That re-pin + a `workflow_dispatch`
  exercise run is the remaining half of F-324 and is out of this task's
  scope (toolkit-side generalization only).
- No negative ("MUST RED") test path asserts an exact `SRC_CONFIG` line
  count or text anywhere searched (`ci/repro/*.sh`,
  `.github/workflows/*.yml`) — the `repro-substrate-negative` step's
  assertion is behavioral (`cargo build` must fail), not textual, so no
  adjustment to test semantics was needed or made.

## Finding the brief did not anticipate — the fix is INCOMPLETE for F-324's actual failure without a follow-up

`ci/repro/cc-validate.sh` (L92–102) and `ci/repro/remap-off-negative.sh`
(L80–92) **each carry their own independent copy** of the identical
`MINISCRIPT_REV`-keyed `SRC_CONFIG` assembly — not sourced from
`double-build.sh` despite this workflow's header comment claiming
*"the single source of truth for the build command + remap is
ci/repro/double-build.sh."* `remap-off-negative.sh`'s own comment says so
explicitly: *"IDENTICAL construction to double-build.sh / cc-validate.sh"*
(L77). Both scripts run genuine fresh `cargo build --locked --offline …
"${SRC_CONFIG[@]}"` invocations (`cc-validate.sh` rebuilds
`secp256k1-sys` twice, under different `SOURCE_DATE_EPOCH` settings, to
probe determinism; `remap-off-negative.sh` rebuilds a full leg with the
remap deliberately disabled) — these are not replays of `double-build.sh`'s
already-resolved artifacts.

Both `repro-x86_64-musl` and `repro-aarch64-musl` run, in order:
`double-build` → `cc-validate` → `remap-off-negative` → `gzip-residue` →
`binary runs` → `assert top-level flag`. I threaded `GIT_SOURCE_URL`/
`GIT_SOURCE_REV` into these two jobs' `env:` (per the brief), so the env
vars ARE present in the process environment when `cc-validate.sh` and
`remap-off-negative.sh` run — but neither script reads them or builds the
fourth `[source]` stanza, because I did not touch those two files (out of
the brief's stated scope: only "the in-shell SRC_CONFIG blocks (around
lines 347-362 and 378-392)" and `double-build.sh` were named).

**Consequence:** once `mnemonic-secret`'s `man-release.yml` is re-pinned to
a `toolkit_ref` carrying this commit and passes `git_source_url`/
`git_source_rev`, the `double-build` step will now succeed (fixed), but the
very next step, `cc-validate`, will fail with the **same** `"can't checkout
… you are in the offline mode"` error for `mnemonic-io-lib` — because its
own `SRC_CONFIG` copy still only knows about the miniscript stanza. The job
will still go red, just one step later than before. `remap-off-negative`
has the identical gap.

**What closing it looks like** (not done here, per scope): in
`cc-validate.sh`, add the identical `if [ -n "$GIT_SOURCE_URL" ] && [ -n
"$GIT_SOURCE_REV" ]` block (same three `--config` lines) right after its
miniscript stanza's closing `fi` (L102), before
`source.vendored-sources.directory` (L103-105); in `remap-off-negative.sh`,
the analogous point is right after its own closing `fi` (L90), before
`source.vendored-sources.directory` (L91-93). Both scripts already read
`MINISCRIPT_REV` the same single-dash way (`cc-validate.sh` L92,
`remap-off-negative.sh` L80) and are invoked with the same job-level env
this commit already wires, so the fix is mechanical and small — but it is
necessary before F-324 is actually closed by exercising `man-release.yml`'s
`repro` job end to end. Recommend filing this as a same-cycle follow-up
(owning phase: before the `workflow_dispatch` exercise run F-324 requires)
rather than letting it surface as a second red run.

## Second commit — cc-validate.sh / remap-off-negative.sh (coordinator-requested follow-up)

The finding above ("the fix is INCOMPLETE...") was addressed in a second
commit on the same branch, per the coordinator's explicit instruction after
reading this report. **Commit:** `d39d96269ce352270189c11fabebf9ad070362b4`
(signed-off, second commit on `f324-git-source`, not amended, not pushed).

### `git show --stat`

```
 ci/repro/cc-validate.sh        | 11 +++++++++++
 ci/repro/remap-off-negative.sh | 11 +++++++++++
 2 files changed, 22 insertions(+)
```

### Sites changed

`ci/repro/cc-validate.sh`:
- After `MINISCRIPT_REV="${MINISCRIPT_REV-...}"` (L92): added
  `GIT_SOURCE_URL="${GIT_SOURCE_URL:-}"` / `GIT_SOURCE_REV="${GIT_SOURCE_REV:-}"`
  (plain `${VAR:-}`, matching `double-build.sh`'s F-324 addition — no
  sensible non-empty default rev for an arbitrary caller dependency).
- After the miniscript stanza's closing `fi` (originally L102, now shifted
  by the 4 inserted lines above it) and before
  `SRC_CONFIG+=( --config 'source.vendored-sources.directory="vendor"' )`:
  added the identical `if [ -n "$GIT_SOURCE_URL" ] && [ -n "$GIT_SOURCE_REV" ]`
  block with the same three `--config` lines (`.git=`, `.rev=`,
  `.replace-with="vendored-sources"`), byte-for-byte the same construction
  as the miniscript stanza and as `double-build.sh`'s F-324 block.

`ci/repro/remap-off-negative.sh`: the identical two edits at the analogous
points (its own `MINISCRIPT_REV` default line and its own miniscript
stanza's closing `fi`, originally L80 and L90 respectively).

Confirmed the two scripts' new blocks are **byte-identical** to each other
(`diff` on the `MINISCRIPT_REV=...` through the closing `)` of
`SRC_CONFIG+=(...vendored-sources.directory...)` span returned no output).

No workflow YAML changes were needed for this commit: `cc-validate.sh` and
`remap-off-negative.sh` are invoked (`.github/workflows/reproducible-musl-build.yml`
lines 556, 573, 740, 754) only as steps inside the `repro-x86_64-musl` and
`repro-aarch64-musl` jobs, and both jobs' `env:` blocks (L499–500, L662–663)
already carry `GIT_SOURCE_URL`/`GIT_SOURCE_REV` from the first commit —
job-level env is inherited by every step in the job, these two scripts
included.

### Validation run

```
$ bash -n ci/repro/double-build.sh && echo OK
OK
$ bash -n ci/repro/cc-validate.sh && echo OK
OK
$ bash -n ci/repro/remap-off-negative.sh && echo OK
OK

$ actionlint .github/workflows/reproducible-musl-build.yml
(exit 0, no output — clean)
```

Dry-rendered the (now-shared, byte-identical) `SRC_CONFIG` logic used by
both scripts via the same scratch-script-plus-`printf` method as the first
commit, under the same three cases:

**ms-shaped** (`MINISCRIPT_REV=""`,
`GIT_SOURCE_URL=https://github.com/bg002h/mnemonic-engrave`,
`GIT_SOURCE_REV=6c24e62823e6c1ac02aa3862cd6020674bf58544`) → four-block form,
identical in shape to `double-build.sh`'s F-324 rendering:
```
--config source.crates-io.replace-with="vendored-sources"
--config source."git+https://github.com/bg002h/mnemonic-engrave?rev=6c24e62823e6c1ac02aa3862cd6020674bf58544".git="https://github.com/bg002h/mnemonic-engrave"
--config source."git+https://github.com/bg002h/mnemonic-engrave?rev=6c24e62823e6c1ac02aa3862cd6020674bf58544".rev="6c24e62823e6c1ac02aa3862cd6020674bf58544"
--config source."git+https://github.com/bg002h/mnemonic-engrave?rev=6c24e62823e6c1ac02aa3862cd6020674bf58544".replace-with="vendored-sources"
--config source.vendored-sources.directory="vendor"
```

**Toolkit default** (nothing set): unchanged three-block form
(crates-io + miniscript fork + vendored-sources.directory).

**Codec-today shape** (`MINISCRIPT_REV=""`, `git_source` unset, what
`md`/`mk`/`ms` currently pass): unchanged two-block form — byte identical
to pre-F-324.

### Residual scope

This closes the gap the first commit's report flagged: `repro-x86_64-musl`
and `repro-aarch64-musl` now resolve a caller's extra git-sourced dependency
consistently across `double-build`, `cc-validate`, and
`remap-off-negative` — all three steps a real gate run exercises. What
remains, unchanged from the first commit's report, is entirely outside this
repo: `mnemonic-secret/.github/workflows/man-release.yml`'s `repro:` job
still needs its `toolkit_ref` re-pinned to (at least)
`d39d96269ce352270189c11fabebf9ad070362b4`, plus `git_source_url`/
`git_source_rev` passed, then a `workflow_dispatch` exercise run per
F-324's "must be EXERCISED, not merely edited."

## Report persisted; nothing folded, nothing pushed

This file is the only thing written to `mnemonic-engrave` by this task, and
it was not `git add`ed there. Two commits were made on the toolkit worktree
branch `f324-git-source` (neither amended, neither pushed):
`21b6696e09159bc4b52259f32f5b13ca1f037d06` (the generalized input +
double-build.sh) and `d39d96269ce352270189c11fabebf9ad070362b4`
(cc-validate.sh + remap-off-negative.sh, closing the gap the first commit's
report flagged).
