# mnemonic-toolkit docs pipeline back to green — implementation report

Worktree `/scratch/code/shibboleth/tk-worktrees/docs-green`, branch `docs-green`,
base `642300b7`, **HEAD `6ce7e464`**. Nothing pushed, merged or tagged; branch
protection untouched; no subagents.

| commit | what |
|---|---|
| `86c93f84` | md-cli pin 0.19.0 → 0.20.2 in `scripts/install.sh` + manual/quickstart/technical-manual/cross-tool-differential workflows; `.examples-build/Examples.md` regenerated in the same commit |
| `9b7028f1` | transcripts regenerated against the CI-pinned tier; 4 ms transcripts moved off argv |
| `7dc69c79` | manual flag lint made section-scoped; list gains `md compose`, `md shape-key`, `md/ms/mk gui-schema` (**RED on purpose**, F-647 step 1) |
| `3dade9ec` | manual content: the 3 flags, md compose/shape-key/verdict, ms argv guard, everything the new lint found (green) |
| `6ce7e464` | doc workflows reshaped so they can be required (guard script + no `paths:`) |

## Binaries used (what CI will use)

Built from the tags the workflows install (`cargo install --git … --tag …` into a
scratch root): md-cli `descriptor-mnemonic-md-cli-v0.20.2` (== the installed md's
source HEAD `7b03a408`), ms-cli `ms-cli-v0.19.0` (`4f1e99e`; installed ms is 3
ci/script-only commits past it — identical `--help` on the nine secret-reading and hashlock verbs compared), mk-cli
`mk-cli-v0.13.0` (`0feaaaa`), and `mnemonic` built from the worktree with the
pinned 1.85.0 toolchain (identical top-level `--help` to the installed 0.104.0, same source commit).

**The installed `mk` is NOT mk-cli 0.13.0.** It is built from mnemonic-key `main`,
19 mk-cli commits past the tag, still versioned 0.13.0. It prints a chunk-set-id
warning and has `--in` on six verbs that the release lacks. Goldens were
regenerated with the tag build; with the installed mk, 7 mk transcripts drift.

## Per red job

### manual (run 35862104907) — GREEN locally
- **Cause:** `make audit` stopped at the flag lint. Three flags were undocumented:
  `mnemonic restore --recalibrate-threads`, and `ms hashlock --kind` and
  `--phrase-looks-like-digest-ok`. Behind that, the replay step had 8 more
  drifted transcripts that CI never reached (the same set as quickstart's).
- **Fix:** documented all three, from `--help`, the source, and runs:
  - `--recalibrate-threads`: `cmd/restore.rs::resolve_threads` + `config.rs`. The
    thread count is measured on the first multisig-template search and recorded
    under `[search]` in `~/.mnemonic/mt.conf`. The flag re-measures and
    overwrites the record. Only `run_capped_search` (id-search /
    `--search-address`) reaches it. That same passage still described the
    ~1-hour search ceiling and a forcing `--accept-search-time`; both were
    removed in v0.99.0. The prose and both flag rows (restore, verify-bundle)
    are corrected.
  - `--kind`: selects the stdout record (`hash:<hex>` for sha256,
    `hash:<kind>:<hex>` otherwise). When it is omitted, stdout keeps the sha256
    record and stderr lists all four digests. Every recipe in the chapter now
    carries `--kind`.
  - `--phrase-looks-like-digest-ok`: a phrase of exactly 40 or 64 hex characters
    stops with exit 1 unless this flag confirms it. The old rule ("never
    exactly 64 hex") was stale.
  - Transcripts regenerated (see quickstart).
- **Evidence:** `make audit` with the pinned tier: lint OK, 62 transcripts pass,
  anchor-check matches the baseline. `make pdf` / `make html` build.

### quickstart (run 35862104662) — GREEN locally
- **Cause:** `verify-examples` replays the shared transcript pool. CI failed on 8
  transcripts, not only the `qs-26-recover-md1` the log tail shows:
  - mnemonic 0.104.0's verify-bundle adds a row, `md1_origin_match`
    (23-verify, qs-24-verify, 41-inheritance).
  - The non-canonical-origin info line was reworded (41-bundle-inheritance-*).
  - `md decode` adds a "template does NOT carry the origin" note
    (24-recover-md1, qs-26-recover-md1).
  - The ms argv guard refuses `ms repair --ms1 <ms1>` (43-ms-repair-text). The
    guard exists **since ms-cli 0.17.0**. The `9b7028f1` message says "0.19.0",
    which is the pinned version, not the one that introduced it.
- **Fix:** the ms transcript now writes the card into its tmp cwd and passes
  `--in FILE`. All goldens were regenerated with the pinned tier.
- **Evidence:** `make lint` OK; `make verify-examples` reports 62 transcripts
  pass; pdf and html build.

### technical-manual (run 35862104701) — GREEN locally
- **Cause:** the lint passed and the **build step was never reached**. The
  failing step was `verify-examples`, with 6 drifted transcripts:
  - Three ms argv refusals (`ms decode <ms1>`, and `ms encode --phrase '<words>'`
    twice).
  - md1-decode's origin note.
  - Two verify-bundle `md1_origin_match` rows (including the JSON one).
- **Fix:** the ms transcripts were moved to `--in FILE`. Since 0.17.0, ms encode
  prints unbroken stdout on every path (grouping applies to the stderr card
  only), so that golden is now unbroken too. All goldens were regenerated.
- **Evidence:** `make lint` OK; 18 transcripts pass; `make pdf` and `make html`
  build.

### vendor-freshness (run 35862104734) — NOT fixed (build infrastructure)
- **Cause:** since F-642 (`642300b7`), md-codec is a **git** dependency
  (`descriptor-mnemonic?rev=cf35d61a`). `ci/repro/vendor-freshness.sh`
  source-replaces only the miniscript git source, so:
  - In CI, check (1) makes cargo try to check out descriptor-mnemonic under
    `--offline` and fails.
  - Locally, check (1) passes falsely, because `~/.cargo/git` already caches the
    rev. Check (3) then fails with "vendored crate(s) with NO offline provenance
    anchor … md-codec" (reproduced).
- `vendor/md-codec` itself is present and at 0.47.0, so the tree is not stale. The
  gate simply has no stanza and no grounded anchor for a second git source.
- **Fix needed:** add the `git+…descriptor-mnemonic?rev=` replacement stanza, as
  `man-pages.yml` already does through `needs.pin.outputs.md_url/md_rev`, and
  ground a check-(4) anchor for `vendor/md-codec`. The alternative is to return
  to crates.io once md-codec 0.47.0 is published there (crates.io max is 0.42.0).

### repro-drift (scheduled; red 09-07, 09-14, 09-21) — NOT fixed (build infrastructure)
- **09-21:** the `repro-x86_64-musl` and `repro-substrate` jobs fail with "failed
  to get `md-codec`". The caller `repro-drift.yml` never passes
  `git_source_url` / `git_source_rev` to `reproducible-musl-build.yml`. The F-324
  plumbing exists and `man-pages.yml` uses it; this caller does not.
- **09-07 and 09-14:** both failed earlier, in the same steps, on
  `rust-miniscript?rev=ff4732e5 … offline`. That predates md-codec's git pin, so
  at least one substrate/double-build config path also lacks the miniscript
  stanza. This is a second cause, not diagnosed further.
- **09-21 aarch64:** "remap-off NEGATIVE FAILED — leaked ZERO /project residue"
  (the remap may be a no-op under cross). This is a third, separate finding.

## Also done
- **F-647:** added `md compose` to the list first. Under the **old** lint only 3
  of its 7 flags failed. `--unspendable` passed on the substring
  `--unspendable-key`, and `--path`, `--json` and `--experimental` passed on
  other verbs' text. The section now documents `--wrapper`, `--path`,
  `--preset` (all six presets), `--unspendable nums|liana`, `--experimental`,
  `--md-only` (with the exit-1 refusal, from a real run) and `--json`
  (`unspendable_kind`).
- **Lint made section-scoped** (`docs/manual/tests/lint.sh`, documented in
  `AUTHORING.md`):
  - every listed verb needs a heading;
  - each option the verb *defines* must appear as a whole flag inside that
    verb's section;
  - global options only need to appear somewhere in the chapter;
  - an empty `--help` is a FAIL.

  Against the committed chapters it found **49** gaps plus 5 missing sections.
  Mutation checks M1–M5 all failed as intended (listed in the `7dc69c79`
  message). The first draft had a `printf | grep -q` SIGPIPE false-negative
  under pipefail, found by running it.
- **Newly documented:**
  - `md shape-key`;
  - the coordinator verdict on `md descriptor` and `md compose` (real output);
  - md, ms and mk `gui-schema`;
  - md `--in`/`--out` on every verb that has them, `--experimental` on
    address/descriptor, `--path` on verify;
  - md verify's 0.19.0 no-admission rule, and repair's 0.19.0 exit-5 case at an
    unsupported wire version;
  - an ms chapter section on the argv guard, with the private channel for each
    verb; every ms example moved off argv;
  - ms `--in`/`--out`/`--allow-argv-secret` in each verb's table;
  - mnemonic `--group-size`/`--separator` on convert and ms-shares.
- **mk `--in` rows** are marked "(mk-cli after 0.13.0)", following the chapter's
  existing convention for `--keys`.
- **Verb counts** corrected: md 15, ms 12, mk 10.
- **`scripts/install.sh` tests:** both `*.test.sh` pass.
- **Toolkit suite:** `cargo nextest run --locked --workspace` (1.85.0,
  `CARGO_TARGET_DIR=…/docs-green-target`) ran 4057 tests: 4057 passed, 20
  skipped. No Rust code was touched.
- **Cross-tool differential** against md 0.20.2 (the bumped workflow's pin):
  1 passed.
- **Examples golden:** a second regen shows no drift.

## Could not be run locally
- **The GitHub workflows themselves.** actionlint is clean on all four, but no
  runner simulation was done. Their first run on a PR or on `ci/staging` is the
  born-green check.
- **`Dockerfile.build`:** not needed. pandoc 3.6 and TeX Live 2026 xelatex are
  installed locally, which differs from CI's apt pandoc/texlive. The PDFs build
  locally but byte parity is not claimed. The manual PDF logs contain pre-existing
  "Missing character" warnings (Japanese glyphs and an emoji in DejaVu Sans Mono).
- **manual-gui gates:** not run. That workflow last ran 2026-07-11 (green) and has
  its own GUI-pinned tier.
- **Technical-manual codec-G2 with siblings present:** the worktree layout makes
  symbol-ref-check skip all 726 refs. I ran it in a synthetic workspace (symlinks
  to the sibling repos' HEADs). Toolkit refs pass; 2 fail:
  `mk-codec …pipeline.rs::fresh_chunk_set_id` is gone in mk-codec 0.5.0 / main.
  The toolkit is on mk-codec 0.4.1, where the symbol exists, and CI does not run
  codec-G2. This is pre-existing and not fixed.
- **vendor-freshness and repro-drift fixes:** not applied, as briefed.

## Required-check proposal
Add these four contexts (all from app `github-actions`, the same app as the
existing `examples` / `clippy` / `test (ubuntu-latest)`):

| context | workflow | why it always appears |
|---|---|---|
| `manual` | manual.yml | The job id is `manual` and the job has no job-level `if`. Triggers: every push to master/main/`ci/**` and every PR, with no `paths:`. `ci/doc-gate-guard.sh` gates only steps, so an irrelevant commit still reports a (fast) success. |
| `quickstart` | quickstart.yml | Same shape. The job was renamed from `build`, which manual.yml also used, so the two contexts are now distinct. |
| `technical-manual` | technical-manual.yml | Same shape. The job was renamed from `lint`. |
| `manual-gui` | manual-gui.yml | An aggregate job with `if: always()` and `needs: [changes, lint, verify-examples, verify-examples-gui, build]`. It passes only if `changes` succeeded and every gate job either succeeded or was skipped because nothing gated changed. Failed, cancelled, or skipped while relevant all fail it, so it is fail-closed. |

The guard only ever errs toward running the gate:
- A tag, a dispatch, an unfetchable base, or any git error answers "relevant".
- The diff is two-dot, so it can over-include but never miss a change.
- A new `ci/staging` branch (`before` = 000…) is diffed against master.
- It was tested locally with synthetic events: true and false cases, a new
  branch, an unfetchable before, a tag, a dispatch, and a bogus PR base.

**Why not reuse examples.yml exactly.** examples.yml filters `push:` by paths and
leaves only the PR side unfiltered. `scripts/push-via-staging.sh` earns required
contexts on the `ci/staging` **push**. A push-path-filtered required context
never appears on a code-only (or docs-only) staging push, and the script stops
with "required context(s) NEVER RAN". So I extended the always-run pattern to
`push`. **`examples` itself has this hole today:** a docs-only commit cannot
earn `examples` through the staging script. The same guard would fix it; not
done here.

Path scopes widened where the old filters were wrong:
- The three books replay transcripts against the source-built `mnemonic`, so
  `crates/**` and `Cargo.{toml,lock}` are now gated.
- quickstart also gates `docs/manual/**`, because its `transcripts/` is a
  symlink into the manual.
- Cost: every code push now runs three doc jobs, each installing three CLIs.

## Concerns / findings for other repos
1. **`install.sh` installs stale sibling CLIs by default.** Its default path for
   md/ms/mk is crates.io latest, not the pinned tag. crates.io has md-cli 0.13.0,
   ms-cli 0.14.0 and mk-cli 0.12.1, against pins of 0.20.2, 0.19.0 and 0.13.0. A
   user following the manual's install line gets CLIs several releases older
   than the manual documents.
2. **md: `md shape-key --descriptor` refuses a wallet-exported descriptor.**
   `md descriptor` re-serialises xpubs with parent fingerprint `00000000` (I
   decoded both). A real xpub with its true parent fingerprint then fails
   "does not round-trip chain 0". This is documented as a limitation; it may
   deserve an F-number.
3. **ms help text is stale in three places:**
   - `encode --group-size` says it groups "the emitted ms1 string", but stdout
     has been unbroken since 0.17.0.
   - `repair`'s EXAMPLES teach `--ms1 ms10…` on argv, which is now refused.
   - `split --out` still prints the `share i of N:` labels and the "stdout
     carries private key material" warning while stdout is empty.
4. **toolkit:**
   - `verify-bundle --accept-search-time` help reads garbled ("…for a multisig
     DEPRECATED and IGNORED…").
   - `resolve_threads`' doc comment says "explicit flag > config > measure", but
     no explicit threads flag exists.
5. **me 0.11.0 packs a 40/64-hex phrase** in a `phrase:` record without
   `ms hashlock`'s F-539 stop (measured: exit 0).
6. **Installed binaries are not the releases.** The installed mk is unreleased
   `main`, labelled 0.13.0 (see above). The operator asked for docs to "match
   the installed" binaries; the docs track the tag CI verifies, and mk-main-only
   features are marked "after 0.13.0".
7. **manual-gui is unverified since July.** Requiring `manual-gui` is safe
   structurally, but its gate has not run since 2026-07-11. Trigger it once
   (workflow_dispatch now exists) before adding it as required.
8. **F-647 status not edited.** I did not update
   `mnemonic-engrave/design/FOLLOWUPS.md`. The fix is in `3dade9ec`; closing
   F-647 is for the controller.

---

## Fix round 1: review `toolkit-docs-green-review.md` (0C/4I/2M/2N)

**HEAD is now `333b6745`.** The fold adds three commits on top of `6ce7e464`.
Nothing was pushed, merged or tagged, and branch protection was not touched.

| commit | findings |
|---|---|
| `df2dd32f` | I-1, I-2, I-3, N-2 (guard) + `ci/doc-gate-guard.test.sh` |
| `c4bd589a` | I-4 (reverse lint + `unreleased` marker) + `ci/doc-flag-lint.test.sh` |
| `333b6745` | M-1, N-1 (42-md.md) |

### Per finding

**I-1 (fixed): the watched set is now derived mechanically.**
`ci/doc-gate-paths.py <book>…` walks each book directory to a fixed point. It adds:
- the resolved target of every symlink;
- every `../` path in the book's non-Markdown files (Makefiles, scripts, Lua
  filters, lint configs) that resolves to an existing file or directory in
  the repo, skipping paths that resolve to an ancestor directory.

Markdown is excluded: its prose links pulled LICENSE and all of `docs/manual`
into manual-gui (measured).

Derived sets today:

| book | watched outside its own directory |
|---|---|
| technical-manual | `docs/manual/tests/verify-examples.sh`, `docs/manual/pandoc/filters/include-transcript.lua` (plus `docs/manual/FOLLOWUPS.md`, harmless) |
| manual-gui | `docs/manual/tests/` |
| quickstart | the manual's `transcripts/`, `pandoc/filters/`, `verify-examples.sh`, `.cspell.json`, `.markdownlint-cli2.jsonc`, `.puppeteer.json`, `Dockerfile.build` |

Workflows now call `--book docs/<x> --also '<ERE>'`. The `--also` ERE keeps
what no file spells as a relative path: `crates/`, `Cargo.*`, the mermaid tool,
the workflow file and the guard.

Test (S1): a commit touching only `docs/manual/tests/verify-examples.sh` gives
**true for all four workflows**, both on a `ci/staging` push and on a PR. The
control run with the pre-fix guard gives false for technical-manual and
manual-gui.

**I-2 (fixed): the guard uses `git diff --no-renames --name-only`.**
Test (S2): `git mv docs/manual/transcripts/22-first-bundle.cmd attic/` gives
true for manual and quickstart; the pre-fix control gives false.

**I-3 (fixed): the diff base now depends on where the push goes.**
- A push to master or main diffs against `before`, the previous protected tip.
- A push to any other branch, and a PR, diffs against the merge-base with
  `origin/<default|base>`, never against `before`.
- A shallow checkout is unshallowed only when the merge-base is missing.

Test (S3) reproduces the review's scenario: commit C breaks the docs and is
left on `ci/staging`, then a README-only commit D goes on top with `before=C`.
The guard gives true; the pre-fix control gives false.

**I-4 (fixed): the lint now checks both directions.** Each first-column flag in
a verb section's flag table must be defined by that verb's `--help` on the
pinned binary, or be a global option. The only exemption is a row carrying the
literal marker `(unreleased: <crate> after <X.Y.Z>)`, where X.Y.Z equals the
pinned binary's `--version`. Each exemption is printed and counted (10 today,
all in mk). A marker also fails when stale: wrong version, or the pinned
binary now defines the flag.

Test (`ci/doc-flag-lint.test.sh`, 7/7 pass):
- the baseline passes;
- **R1:** the review's `--no-such-flag` row in md compose fails;
- **R2:** a marker naming the wrong version fails;
- **R3:** a marker on a flag the pinned md defines (`--md-only`) fails as stale;
- **R4:** removing a marker fails;
- **F1, F2:** the forward direction still fails on a removed flag and a removed
  heading.

**M-1 (fixed).** The chapter now says the verdict takes **four** forms, and also
describes the measured-refusal variant. The imports-altered example is a real
run of pinned md 0.20.2 on the vendored evidence wallet
`plain-2of3-wsh-UNSORTED`:

`Nunchuk 2.1.1: imports the multipath form, but reads it as MINISCRIPT -- not the wallet as built (2026-09-19)`.

The Liana 2-of-4 case is named in the prose.

**M-2: not touched here.** It is filed as F-677. Moving `examples.yml` onto the
guard would change a required workflow's shape; that is beyond a trivial edit.

**N-1 (fixed).** The `md descriptor` example regains its stderr line, in the
order the tool prints it.

**N-2: documented, not worked around.** GitHub starts no workflow for a
`[skip ci]` commit, so no guard can run, and `push-via-staging.sh` stops with
"NEVER RAN". The guard header now says this: such commits cannot be pushed to
master through staging.

### Re-run evidence (pinned tier: md 0.20.2 / ms 0.19.0 / mk 0.13.0, plus mnemonic built from this tree)
- **manual:** `make audit` OK (62 transcripts, anchor-check at baseline, "10 row
  exemption(s)").
- **quickstart:** lint OK, 62 transcripts pass.
- **technical-manual:** lint OK, 18 transcripts pass.
- **Builds:** `make pdf` and `make html` succeed for all three books.
- **Examples golden:** no drift.
- **actionlint:** clean on all workflows.
- **`ci/doc-gate-guard.test.sh`:** 44 passed, 0 failed. It covers S0–S3, the
  controls, the N1/N2 negatives and the F1–F5 fail-safes.
- **`ci/doc-flag-lint.test.sh`:** 7 passed, 0 failed.

### New caveats
- **The manual lint now fails against the locally installed mk.** That mk is
  built from mnemonic-key main and defines the flags the markers exempt, so the
  markers read as stale. Run the lint against the pinned binaries, as CI does.
  When mk-cli is released past 0.13.0 and the pin moves, the lint fails until
  the markers are removed. That is intended.
- **No CI job runs the two test scripts.** They are re-runnable by hand.
  `doc-flag-lint.test.sh` needs the pinned binaries.
- **The guard adds a merge-base fetch to technical-manual.** Its checkout is
  depth 1, so the guard may unshallow the repository on a non-default-branch
  push.
- **Fix round 2 (`aafd9d27`):** `ci/doc-gate-guard.test.sh` now runs as an always-run step in the `manual` job, before the guard step. It has no `if:`, so it runs on every push, tag, PR and dispatch; under a CI-like env locally: 44 passed in 3.1 s (`act` is not installed). `ci/doc-flag-lint.test.sh` runs after the audit, behind the guard: it tests the lint, not the guard, and every input it reads (including the script itself, added to `--also`) makes the guard answer relevant.

---

## Fix round 3: re-review `toolkit-docs-green-review-fix1.md` (NEW-1, NEW-2)

**HEAD:** `5978eaa2`. This round adds two commits: `2d71f8d4` (guard) and
`5978eaa2` (lint).

### NEW-1 (fixed in `2d71f8d4`)
- `ci/doc-gate-paths.py` now reads git trees instead of the working directory,
  and resolves symlink targets lexically: no existence check.
- The guard derives the watched set from the base commit and from HEAD, and
  unions the two.
- **Tests:**
  - S4 deletes the shared `verify-examples.sh`. All four books now answer
    relevant. With the round-1 guard (`aafd9d27`), quickstart and
    technical-manual answer false.
  - S5 and S5b retarget a symlink.
  - S6 deletes the symlink itself, once for each of the three books that have
    one.

### NEW-2 (fixed in `5978eaa2`)
- The marker is now `(unreleased: <crate> after <X.Y.Z>: --flag[, --flag]…)`.
  It exempts exactly the flags it lists; every other flag in the cell is
  linted normally.
- The lint fails if the marker:
  - names no flag;
  - names a flag the row does not document;
  - names the wrong version;
  - lists a flag the pinned binary already defines.
- **Tests:**
  - R5: a fake flag added to a marked row fails.
  - R6: a marker naming a flag the row lacks fails.
  - R7: the old marker form, which exempted the whole row, fails.

### Self-check for a third instance of the same shape
I checked every input the derivation reads, and every exemption the lint and
guard grant. I found and fixed four more, one of them in `5978eaa2`'s nested-verb
lookup:

1. **Reference targets could disappear.** A `../` reference only counted if its
   target existed. S7 deletes `docs/manual/.cspell.json`, which quickstart
   imports; the round-1 guard answered false. The base/HEAD union closes it.
2. **Makefile paths through the repo-root variable were never derived.**
   manual-gui's `make lint` runs `$(TOOLKIT_ROOT)/docs/tools/render-mermaid-cache.py`,
   and neither its old `paths:` filter nor its `--also` watched that file. S8
   touches only that tool; the round-1 guard answered false for manual-gui.
   `$(TOOLKIT_ROOT)/…` paths are now resolved from the repo root.
3. **Nested verbs accepted their siblings' flags.** The reverse check matched
   `seed-xor`, `slip39` and `ms-shares` verbs against their parent's section,
   so each verb accepted its sibling's flags. Each verb now uses its own
   subsection. The one section still shared (seedqr encode/decode) is printed
   on every run.
4. **The verb list itself was never checked.** `cli-subcommands.list` defines
   what the lint covers, and nothing checked it; this is how `md compose` went
   undocumented (F-647). The lint now enumerates every leaf subcommand of the
   pinned binaries and fails in both directions. C1 drops `md compose` from the
   list and fails; C2 adds a verb that does not exist and fails.

Checked and clean:
- a deleted book directory is still watched through its own prefix;
- a missing guard or path-derivation script makes the step fail or answer
  relevant;
- sibling repositories outside this repo are outside any path filter by
  construction;
- the global-flag and `--help`/`--version` exemptions are narrow.

Left as is:
- flag tokens are matched lowercase only (the re-review's Nit; pre-existing,
  and the same in both directions);
- flags mentioned in prose rather than in table rows are not reverse-checked.

### Re-run evidence
Pinned tier: md 0.20.2, ms 0.19.0, mk 0.13.0, plus mnemonic built from this
tree.

| check | result |
|---|---|
| `ci/doc-gate-guard.test.sh` | 62 passed, 0 failed |
| `ci/doc-flag-lint.test.sh` | 12 passed, 0 failed |
| manual `make audit` | OK: 10 flag exemptions, 62 transcripts, anchor-check at baseline |
| quickstart | lint OK, 62 transcripts pass |
| technical-manual | lint OK, 18 transcripts pass |
| pdf and html | built for all three books |
| Examples golden | no drift |
| actionlint | clean |
