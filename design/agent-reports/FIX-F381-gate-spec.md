# F-381 — vendor-freshness gate SPEC brought up to the shipped gate, re-grounding procedure EXECUTED

**Repo under fix:** `mnemonic-toolkit`, worktree
`/scratch/code/shibboleth/_work/f381/mnemonic-toolkit`, branch
`fix/f381-gate-spec`, base `1f333ad2`, fix commit `788c0198`.
**Agent:** fix-only dispatch (not a review — no reviewer/author split; this
report documents what was done and what was measured, for the persist record).
**Not pushed, not tagged**, per the dispatch brief.

## Where the SPEC actually lives

`design/SPEC_vendor_freshness_ci_guard.md` in `mnemonic-toolkit`. Found by
content grep (`grep -rln "vendor freshness\|vendor-freshness" . --include="*.md"`),
not by guessing a path — F-381 was right that it is not under `ci/repro/*.md`.
Confirmed 122 lines before this fix (`wc -l`), matching F-381's citation
exactly.

## What it claimed vs what the gate does

| | Old SPEC (122 lines) | `ci/repro/vendor-freshness.sh` (247 lines, measured) |
|---|---|---|
| Checks | 1 (`§3.1 The check`, settled to `cargo metadata`) | 4: resolution, integrity, registry provenance, git-fork provenance |
| Occurrences of `INTEGRITY` / `REGISTRY PROVENANCE` / `GIT-FORK` / `grounding` / `(n/4)` | 0 (F-381's own measurement, confirmed) | throughout — these are the check names and progress labels the script prints |
| Re-grounding procedure | absent | present only as a comment in the script header (lines 73-76 in the version read at start of this task) |
| F-354 narrative (why checks 2-4 exist) | absent | present in the script's top-of-file comment block |

F-354: `vendor/miniscript` sat at rev `95fdd1c5` while `Cargo.toml`/`Cargo.lock`
pinned `ff4732e5`, undetected for two months. Check (1) alone was and remains
structurally blind to this — it validates name/version/source-id, never bytes.
Measured on the historical defective tree (from the F-354 fix commit,
`2c4510c0`, and restated in the script's own header): checksum-vs-disk
(a check-2-only design) *also* would not have caught it — the F-354 commit
reports **168 crates, 7479 files, 0 checksum mismatches** on the *defective*
historical tree, because `cargo vendor` wrote the manifest from the same
wrong rev it wrote the files from — self-consistent and wrong. (The
*current*, fixed tree measures **169 crates, 7490 files** — a different
point in history from the 168/7479 figure above, not a contradiction of it;
both are cited correctly, and separately, in the updated SPEC.)

## The re-grounding run, verbatim

All commands run from
`/scratch/code/shibboleth/_work/f381/mnemonic-toolkit` unless noted. A
scratch clone of upstream was used for verification, at
`/tmp/f381-scratch/rust-miniscript` (network egress to github.com confirmed
working before starting: `curl -sS -o /dev/null -w "%{http_code}\n" https://github.com` → `200`).

### Baseline (before any change)
```
$ bash ci/repro/vendor-freshness.sh; echo "EXIT: $?"
vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked; miniscript rev ff4732e5f75aa555682343cb180fa72ee3e8e9d5) ...
vendor-freshness: (1/4) OK — vendor/ satisfies Cargo.lock.
vendor-freshness: (2/4) OK — 7490 files across 169 crates match their recorded sha256.
vendor-freshness: (3/4) OK — 168 crates anchored to Cargo.lock checksums; 1 git-fork source(s) exempt by grounding (miniscript).
vendor-freshness: (4/4) OK — vendor/miniscript matches the tree grounded against upstream ff4732e5f75aa555682343cb180fa72ee3e8e9d5.
EXIT: 0
```

### Attempt 1 — an UNrepresentative rev, caught earlier than intended (kept as a finding)
Picked `aea13ab` (HEAD-ish on `rust-bitcoin/rust-miniscript`'s `master` at
clone time, later determined to be **70 commits ahead** of `ff4732e5`, not
"a handful" as first assumed — verified via
`git log --oneline --reverse ff4732e5..master | nl | grep aea13ab` → line 70).
```
$ sed -i 's/rev = "ff4732e5f75aa555682343cb180fa72ee3e8e9d5"/rev = "aea13aba4e2b0a08e4efa381c30be14830dc0f52"/' Cargo.toml
$ cargo update -p miniscript --precise aea13aba4e2b0a08e4efa381c30be14830dc0f52
    Updating git repository `https://github.com/rust-bitcoin/rust-miniscript`
    Updating git repository `https://github.com/rust-bitcoin/rust-miniscript`
    Updating crates.io index
      Adding miniscript v13.0.0 (https://github.com/rust-bitcoin/rust-miniscript?rev=aea13aba4e2b0a08e4efa381c30be14830dc0f52#aea13aba)
    Removing miniscript v13.0.0 (https://github.com/rust-bitcoin/rust-miniscript?rev=ff4732e5f75aa555682343cb180fa72ee3e8e9d5#ff4732e5)
    note: pass `--verbose` to see 64 unchanged dependencies behind latest
$ bash ci/repro/vendor-freshness.sh; echo "EXIT: $?"
vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked; miniscript rev aea13aba4e2b0a08e4efa381c30be14830dc0f52) ...
error: the lock file /scratch/code/shibboleth/_work/f381/mnemonic-toolkit/Cargo.lock needs to be updated but --locked was passed to prevent this
If you want to try to generate the lock file without accessing the network, remove the --locked flag and use --offline instead.
::error::vendor/ is out of sync with Cargo.lock — the --offline --locked reproducible build cannot resolve a dependency from the committed vendor/ tree. Run 'cargo vendor vendor/' and commit the result (see docs/verify-reproducibility.md). This is the v0.74.0 release-CI failure class, now caught at PR time.
EXIT: 1
```
This was RED, but at **check (1)**, not check (4) — a different failure
signature than the classic v0.74.0 "failed to select a version" message.
Reason: `aea13ab`'s `Cargo.toml` differs from `ff4732e5`'s (a `serde`
optional-dependency addition plus later `[workspace.metadata]` churn between
the two, confirmed via
`git diff ff4732e5f75aa555682343cb180fa72ee3e8e9d5..HEAD -- Cargo.toml` in the
upstream clone), so the resolution `cargo update` computed against the real
upstream manifest at `aea13ab` did not match what a directory-source
replacement using the *stale* `vendor/miniscript/Cargo.toml` could reproduce.
**This is a real, useful negative result**: it is not the check-(4)-only
scenario the brief asked for, and demonstrates that not every pin move
reproduces the F-354 shape — some are already caught earlier, by check (1).
Restored immediately:
```
$ git checkout -- Cargo.toml Cargo.lock
$ bash ci/repro/vendor-freshness.sh; echo "EXIT: $?"
[... same 4/4 OK output as baseline ...]
EXIT: 0
```

### Selecting a representative rev for the true F-354 shape
Required: a later, real commit whose `Cargo.toml` is **byte-identical** to
`ff4732e5`'s (so check (1) cannot see the move — the actual F-354 shape),
while carrying real source changes.
```
$ cd /tmp/f381-scratch/rust-miniscript
$ git log --oneline --reverse ff4732e5f75aa555682343cb180fa72ee3e8e9d5..master -- Cargo.toml
2cc105c ci: enable workspace lint rules
[... 8 more, all later ...]
```
`2cc105c` is the FIRST commit (in chronological order) after `ff4732e5` that
touches `Cargo.toml`; the commit immediately before it, `5dcd5fc` ("policy:
Remove function local wildcard imports"), is the last one with an unchanged
manifest — **18 commits ahead** of `ff4732e5`
(`git log --oneline --reverse ff4732e5..master | nl | grep 5dcd5fc` → line 18).
```
$ git diff ff4732e5f75aa555682343cb180fa72ee3e8e9d5 5dcd5fcbf3b56c83e55864c9fc99386f49074cce -- Cargo.toml
[empty — byte-identical]
$ git diff --stat ff4732e5f75aa555682343cb180fa72ee3e8e9d5 5dcd5fcbf3b56c83e55864c9fc99386f49074cce
 58 files changed, 2746 insertions(+), 1640 deletions(-)
```
Confirmed: identical manifest, substantial content change (a full-tree
reformat plus a new `src/validation.rs`) — the correct rehearsal target.

### The RED (check 4 alone)
```
$ cd /scratch/code/shibboleth/_work/f381/mnemonic-toolkit
$ sed -i 's/rev = "ff4732e5f75aa555682343cb180fa72ee3e8e9d5"/rev = "5dcd5fcbf3b56c83e55864c9fc99386f49074cce"/' Cargo.toml
$ cargo update -p miniscript --precise 5dcd5fcbf3b56c83e55864c9fc99386f49074cce
    Updating git repository `https://github.com/rust-bitcoin/rust-miniscript`
    Updating crates.io index
      Adding miniscript v13.0.0 (https://github.com/rust-bitcoin/rust-miniscript?rev=5dcd5fcbf3b56c83e55864c9fc99386f49074cce#5dcd5fcb)
    Removing miniscript v13.0.0 (https://github.com/rust-bitcoin/rust-miniscript?rev=ff4732e5f75aa555682343cb180fa72ee3e8e9d5#ff4732e5)
    note: pass `--verbose` to see 64 unchanged dependencies behind latest
$ bash ci/repro/vendor-freshness.sh; echo "EXIT: $?"
vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked; miniscript rev 5dcd5fcbf3b56c83e55864c9fc99386f49074cce) ...
vendor-freshness: (1/4) OK — vendor/ satisfies Cargo.lock.
::error::vendor-freshness: 1 content defect(s) in vendor/:
  - the miniscript pin MOVED: Cargo.lock is at 5dcd5fcbf3b56c83e55864c9fc99386f49074cce, but this gate is
      grounded at ff4732e5f75aa555682343cb180fa72ee3e8e9d5. The vendored tree cannot be verified against a
      rev nobody has checked. Re-vendor, verify the tree against upstream at the
      new rev, then update EXPECTED_GIT_FORK_REV / EXPECTED_GIT_FORK_MANIFEST_SHA256
      in ci/repro/vendor-freshness.sh (see GROUNDING in its header).
EXIT: 1
```
**Confirmed live:** check (1) GREEN, check (4) alone RED. This is the shape
the brief specified and the shape check (4) exists for.

### Following the documented recovery procedure verbatim
Header text (`ci/repro/vendor-freshness.sh:73-76`, unmodified):
```
# To RE-GROUND after moving the [patch.crates-io] pin:
#   cargo vendor --locked vendor/
#   sha256sum vendor/miniscript/.cargo-checksum.json
# and paste the rev + digest below. Verify against upstream before you do.
```
```
$ cargo vendor --locked vendor/
   [... full re-vendor output, 169 crates, no errors ...]
To use vendored sources, add this to your .cargo/config.toml for this project:
[source.crates-io]
replace-with = "vendored-sources"
[source."git+https://github.com/rust-bitcoin/rust-miniscript?rev=5dcd5fcbf3b56c83e55864c9fc99386f49074cce"]
git = "https://github.com/rust-bitcoin/rust-miniscript"
rev = "5dcd5fcbf3b56c83e55864c9fc99386f49074cce"
replace-with = "vendored-sources"
[source.vendored-sources]
directory = "vendor/"

$ sha256sum vendor/miniscript/.cargo-checksum.json
9f5d6dccfcd02458c310489ce4e07259afcf06f3db72872501d11b69e9b08f86  vendor/miniscript/.cargo-checksum.json
```
Verification against upstream ("Verify against upstream before you do"),
done as a direct byte-for-byte file comparison against the local clone
checked out at `5dcd5fcbf3b56c83e55864c9fc99386f49074cce` (equivalent to the
GitHub-trees-API method the original grounding used):
```
total files in manifest: 103
compared (excl. Cargo.toml): 102
byte-identical to upstream @ 5dcd5fc: 102
mismatched: 0
missing upstream: 0
```
Constants updated in `ci/repro/vendor-freshness.sh` (temporarily, for the
rehearsal):
```
EXPECTED_GIT_FORK_REV="5dcd5fcbf3b56c83e55864c9fc99386f49074cce"
EXPECTED_GIT_FORK_MANIFEST_SHA256="9f5d6dccfcd02458c310489ce4e07259afcf06f3db72872501d11b69e9b08f86"
```
```
$ bash ci/repro/vendor-freshness.sh; echo "EXIT: $?"
vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked; miniscript rev 5dcd5fcbf3b56c83e55864c9fc99386f49074cce) ...
vendor-freshness: (1/4) OK — vendor/ satisfies Cargo.lock.
vendor-freshness: (2/4) OK — 7495 files across 169 crates match their recorded sha256.
vendor-freshness: (3/4) OK — 168 crates anchored to Cargo.lock checksums; 1 git-fork source(s) exempt by grounding (miniscript).
vendor-freshness: (4/4) OK — vendor/miniscript matches the tree grounded against upstream 5dcd5fcbf3b56c83e55864c9fc99386f49074cce.
EXIT: 0
```
**The documented procedure worked exactly as written, on the first attempt.**
No missing step, no wrong step, no correction needed to the header comment.
(The known "recompilation trap" — `cargo` not noticing a vendored source
change under a fixed package-id — does not apply here: this gate never
compiles anything. It applies to whatever build follows a re-vendor, which
this rehearsal deliberately did not chain into, to keep the rehearsal scoped
to the gate itself.)

### Restore
```
$ git checkout -- Cargo.toml Cargo.lock ci/repro/vendor-freshness.sh vendor/
$ git status --porcelain
?? vendor/miniscript/AGENTS.md
?? vendor/miniscript/CLAUDE.md
?? vendor/miniscript/CONTRIBUTING.md
?? vendor/miniscript/SECURITY.md
?? vendor/miniscript/src/validation.rs
```
`git checkout --` restores tracked-file content but does not remove files a
`cargo vendor` run newly created (the newer rev vendors a few files
`ff4732e5` doesn't have) — a real gotcha for anyone restoring after an
experimental re-vendor, worth stating in the SPEC (done, §5.4).
```
$ git clean -f -- vendor/miniscript/AGENTS.md vendor/miniscript/CLAUDE.md vendor/miniscript/CONTRIBUTING.md vendor/miniscript/SECURITY.md vendor/miniscript/src/validation.rs
Removing vendor/miniscript/AGENTS.md
Removing vendor/miniscript/CLAUDE.md
Removing vendor/miniscript/CONTRIBUTING.md
Removing vendor/miniscript/SECURITY.md
Removing vendor/miniscript/src/validation.rs
$ git status --porcelain
[empty]
$ git diff --stat 1f333ad2
[empty]
```
Tree is byte-identical to the committed baseline `1f333ad2`.

### Gates on the restored tree
```
$ bash ci/repro/vendor-freshness.sh; echo "EXIT: $?"
vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked; miniscript rev ff4732e5f75aa555682343cb180fa72ee3e8e9d5) ...
vendor-freshness: (1/4) OK — vendor/ satisfies Cargo.lock.
vendor-freshness: (2/4) OK — 7490 files across 169 crates match their recorded sha256.
vendor-freshness: (3/4) OK — 168 crates anchored to Cargo.lock checksums; 1 git-fork source(s) exempt by grounding (miniscript).
vendor-freshness: (4/4) OK — vendor/miniscript matches the tree grounded against upstream ff4732e5f75aa555682343cb180fa72ee3e8e9d5.
EXIT: 0

$ cargo nextest run --locked
[... 4007 tests ...]
Summary [  47.889s] 4007 tests run: 4007 passed, 20 skipped
```
4007, not 3960 (the pre-P3 baseline) — matches the brief's corrected number.

## Blind spots stated (now in the SPEC, §4, not only the script)

1. **Trust-on-first-use (check 4).** Proves the vendored tree has not
   *changed* since a human grounded it once against upstream; cannot itself
   prove that grounding was correct.
2. **Checksums prove disk matches the manifest, not that the manifest matches
   the pin** — checks (2)/(3) both terminate at artifacts (`.cargo-checksum.json`,
   `Cargo.lock`) that `cargo vendor` writes self-consistently even from a
   wrong source. Closed for registry crates by the independent tarball
   digest cargo validates; not closed for the git fork except by check (4)'s
   grounding.
3. **No bit-for-bit reproducibility proof** — that stays with
   `repro-drift.yml` and the release `repro` gate; none of the four checks
   here re-derives it.
4. **A brand-new unanchored source fails closed but ungrounded** — check (3)
   catches its *existence* (the set assertion), but it has no check-4-style
   content anchor until a human grounds it individually; the grounding work
   does not generalize automatically to a new dependency.

## Deviations from the brief

None on scope. One correction made mid-task: the first rehearsal attempt
(`aea13ab`) was not representative of the check-(4)-only failure mode (it hit
check (1) instead, for a legitimate reason — its `Cargo.toml` differs from the
pinned rev's). Recognized this, discarded that attempt's result as the primary
evidence, and re-ran with a rev selected specifically to reproduce the F-354
shape (`5dcd5fc`). Both outcomes are recorded in the SPEC (§5.1) and above,
rather than only reporting the second, cleaner run.

## Files touched

- `mnemonic-toolkit` (worktree `/scratch/code/shibboleth/_work/f381/mnemonic-toolkit`,
  branch `fix/f381-gate-spec`): `design/SPEC_vendor_freshness_ci_guard.md`
  rewritten (122 → 396 lines). Commit `788c0198`. **Not pushed.**
- `ci/repro/vendor-freshness.sh`: touched only transiently during the
  rehearsal (constants swapped and swapped back); final state is
  byte-identical to `1f333ad2` (`git diff --stat 1f333ad2` empty after
  restore, reconfirmed before the SPEC commit).
- No changes to F-355 or F-371 (parked, per the brief).

## Gate results (final, on the fix commit `788c0198`)

```
bash ci/repro/vendor-freshness.sh    -> EXIT 0, all four checks OK,
                                          7490 files / 169 crates
cargo nextest run --locked           -> 4007 passed, 20 skipped
```
