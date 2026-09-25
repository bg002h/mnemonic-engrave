# F-678 verification review (descriptor-mnemonic `f678-musl`, d2c8488a)

Author: review agent (Sonnet 5), 2026-09-24.
Scope: the one question only — do the two new checks fail when they should,
and can nothing reach a release except on a tag. Diff reviewed: `git diff
origin/main..d2c8488a -- .github/workflows/man-pages.yml Cross.toml` (2 files).
Settled facts from the brief (root cause, CI run 36086241898 green,
`repro-aarch64-musl` skipped by `run_aarch64: false`) taken as given, not
re-derived.

## Check 1 — `prep` step fails closed

Extracted the step body verbatim (byte-diffed against
`.github/workflows/man-pages.yml` lines 249-278 after de-indenting; only
whitespace differed) and ran it locally against three cases:

- Valid `MINISCRIPT_REV` + unmodified `Cargo.lock` → writes the 5-pair
  `--config` args file, `VER=dispatch-<sha12>`, **EXIT=0**.
- `MINISCRIPT_REV=""` → `::error::MINISCRIPT_REV is empty …`, **EXIT=1**.
- Valid rev, but `Cargo.lock` carries an extra uncovered
  `source = "git+https://github.com/example/other-crate?rev=…"` line →
  `::error::Cargo.lock has a git source the [source] config does not
  redirect: …`, **EXIT=1**.

Also checked the upstream `resolve-miniscript-rev` awk extractor (line
96-103), since a Cargo.lock with an empty/missing rev is caught there first:
fed it a `Cargo.lock` where the miniscript `source =` line has no `rev=`
component → `::error::could not resolve a git rev for the 'miniscript'
package …`, **EXIT=1**. Both layers fail closed.

## Check 2 — the dep-info (`vendored-miniscript`) check can fail

Extracted the step body verbatim (byte-identical diff against lines 408-424)
and ran five cases against synthetic `target/<target>/release/deps/*.d` trees:

| case | dep-info content | result |
|---|---|---|
| A: valid | `.../vendor/miniscript/src/lib.rs` | OK, EXIT=0 |
| B: git checkout | `/cargo/git/checkouts/rust-miniscript-…/src/lib.rs` | `does not read vendor/miniscript`, EXIT=1 |
| C: no dep-info at all | empty `deps/` dir | `no miniscript dep-info under target/…`, EXIT=1 (not a vacuous pass) |
| D: neither vendor nor git-checkout path | some other registry path | `does not read vendor/miniscript`, EXIT=1 |
| E: miniscript's own `.d` clean, but a sibling crate's `.d` shows `/git/checkouts/` | two files | caught by the second, broader `grep -l '/git/checkouts/'`, EXIT=1 |

Empty/missing dep-info does not pass vacuously (case C); a checkout path on
either the miniscript file itself or any other crate's file fails the build
(cases B and E).

## Check 3 — release safety: everything that writes to a release is tag-gated

Quoted directly from `d2c8488a`'s `.github/workflows/man-pages.yml`:

- `man-pages` job: `Ensure release exists` (line 60) and `Upload man-page
  bundle` (line 69) — both `if: startsWith(github.ref,
  'refs/tags/descriptor-mnemonic-md-cli-v')`. Unchanged by this diff.
- `musl-binaries` job: **job-level** gate widened to `if:
  startsWith(github.ref, 'refs/tags/descriptor-mnemonic-md-cli-v') ||
  github.event_name == 'workflow_dispatch'` (line 217) — this is what now
  lets the build/package/check steps run on dispatch.
- Compensating for that widening, the two release-writing steps inside the
  same job carry their **own** explicit tag-only guard:
  `ensure-release` (line 427) and `upload` (line 432), both `if:
  startsWith(github.ref, 'refs/tags/descriptor-mnemonic-md-cli-v')`.

**Confirmed this guard is a real addition, not redundant decoration:** on
`origin/main` (pre-fix), these two steps had **no** step-level `if:` at all —
they inherited tag-only behavior solely from the job-level condition. Now
that the job-level condition also admits `workflow_dispatch`, the step-level
guards are load-bearing; without them a manual dispatch would attempt
`gh release create`/`gh release upload`. Grepped the whole file for every
`gh release` invocation (4 total: view/create/upload×2 for man-pages, and
view/create/upload×3 for musl-binaries) — all sit behind one of these two
tag-only conditions. No other command in the diff writes to a release.

## Check 4 — nothing else regressed

- `actionlint .github/workflows/man-pages.yml` → clean (exit 0).
- Asset names on the tag path are byte-identical to before: `VER` on a tag
  ref is still `"${REF_NAME##*-v}"` (only relocated into `prep`, not
  changed), so `md-${VER}-${ARCH}-linux-musl.tar.gz`,
  `SHA256SUMS.${ARCH}`, `PROVENANCE.${ARCH}.txt` still resolve to
  `md-<ver>-{x86_64,aarch64}-linux-musl.tar.gz`, `SHA256SUMS.{x86_64,aarch64}`,
  `PROVENANCE.{x86_64,aarch64}.txt` (lines 330/336/340, 389/394/398,
  upload block 436-439 — untouched by this diff).

## Check 5 — stale comments

One found, **not introduced by this diff but left uncorrected inside a block
the fold claims to have fixed**: `.github/workflows/man-pages.yml` line 14
still reads *"so md's offline **two-block** reproducibility can be
CI-verified"*, describing the `repro` job's config. That job has used the
**three-block** form since commit `0ce18660` (predates F-678; see the file's
own lines 120-126: *"`miniscript_rev` must select the THREE-block `--config`
form"*), so line 14 was already false before this branch. The implementer's
report claims *"I corrected the comments that the change made false: the
man-pages header…"* — but line 14 sits two lines above the sentence they did
edit (lines 17-18, correctly updated to mention the musl-binary legs), inside
the same header block, and was not touched. Severity: Minor — it is a
documentation-only inaccuracy about the `repro` job's own config shape (not
about anything this diff changed), does not affect gating or asset output,
and was already wrong before `d2c8488a`. No other stale "two-block" reference
remains: lines 82 and 123 are correctly past-tense historical narration, and
line 194 (new in this diff) correctly narrates the musl-binaries legs' old
two-block form in the past tense. `Cross.toml`'s comments are internally
consistent with the current three-block code; nothing stale found there.

## Findings

- **Minor**: `man-pages.yml` line 14 still describes the `repro` job as
  "two-block"; it has been three-block since `0ce18660`. Pre-existing, not
  introduced by F-678, but left over inside the header block the implementer
  reported fixing. One-line fix (`two-block` → `three-block`), no functional
  impact.

No Critical or Important findings. Both new checks were independently
reproduced (not just re-run from the implementer's script) and fail closed
in every adversarial case tried, including two cases beyond the brief's
minimum (dep-info present-but-neither-path, and a non-miniscript sibling
crate's dep-info leaking a git checkout). Every release-writing step is
tag-gated, and the newly-added step-level guards are confirmed load-bearing
against the widened job-level condition, not redundant.

**ready to ship: yes**
