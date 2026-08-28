# MERGE — F-354's content-aware vendor gate into P3

**Repo:** `mnemonic-toolkit` (worktree `_work/tkmerge/mnemonic-toolkit`, branch
`merge/f354-into-p3`)
**Merge commit:** `1f333ad2`, parents `59ade791` (P3 / master) and `2c4510c0` (F-354)
**Date:** 2026-08-27
**Outcome:** merged, gates green, five mutations plus two extra probes all RED
where they must be. **Not pushed, not tagged.**

---

## 1. The conflict, and why it was not a text merge

`git merge fix/f354-vendor-miniscript` produced exactly **one** conflicted file:
`ci/repro/vendor-freshness.sh`. The 16-file `vendor/miniscript` re-vendor merged
clean.

The real conflict was semantic. F-354's check (3) asserts that **exactly one**
vendored crate lacks an offline provenance anchor and that it is `miniscript`:

```python
unexpected = [d for d, _ in unanchored if d != fork_dir]
```

A git source has no published tarball, so both `.cargo-checksum.json`'s
`package` and `Cargo.lock`'s `checksum` are null. P3 row 1 had added a **second**
such source (`mnemonic-io-lib`, pinned by rev out of `bg002h/mnemonic-engrave`),
so on the merged tree there were two unanchored crates and check (3) went RED on
`mnemonic-io-lib`. That is the gate working — refusing to vouch for a source
nobody grounded.

## 2. The resolution chosen, and why

Two resolutions were live.

**Option A (built first, then discarded):** ground the second source the same way
the first is grounded — table-drive checks (3) and (4) over a list of grounded
git sources, keeping the set assertion exact so a *third* still REDs. This was
implemented and fully mutation-tested; it worked. Its cost is that the gate grows
bespoke provenance machinery, and the exempt set grows from one named crate to
two.

**Option B (the architect's ruling, and what is committed):** eliminate the
second git source instead. `mnemonic-io-lib 0.1.0` had been published to
crates.io, so the dependency moved from `{ git, rev }` to the registry.

The ruling's reasoning, which the evidence below bears out: a registry
dependency is anchored by **cargo itself**, against an immutable public tarball,
on every build, by machinery nobody in this project maintains. A git source gets
only the grounding we hand-build, and each one dilutes the gate's sharpest
property — an exact, *named* unanchored set. Shrink the set rather than
generalise the machinery.

### What that meant concretely

| Change | Detail |
| --- | --- |
| `crates/mnemonic-toolkit/Cargo.toml` | `mnemonic-io-lib = "0.1.0"` replaces the `{ git, rev = "6c24e628…" }` pin |
| `Cargo.lock` | source becomes `registry+…crates.io-index`; gains `checksum = "d54d9fc783d32defa2274b346b66549a29756db8514200b12c3c98a2c4579f64"` |
| `vendor/mnemonic-io-lib` | re-vendored from the published tarball; `.cargo-checksum.json` now carries a `package` digest |
| `ci/repro/vendor-freshness.sh` | **taken from F-354 byte-identical** (`cmp` exit 0) |

**The gate needed no edit.** That was the ruling's own falsifiable test, and it
passed: check (3) reports one unanchored crate again, and it is `miniscript`.

**P3's gate contribution is not dropped on its merits.** Its `IO_LIB_REV`
derivation, its fail-closed guard and its fourth `--config` block existed to
serve a git source that the ruling removed. With the source gone the config is
correctly back to three blocks, and this was verified against an **empty
`CARGO_HOME`** (exit 0) so the result is not an artifact of a warm git checkout.

### Independent verification of the publish

Fetched from the crates.io sparse index directly, not taken from cargo:

```
mnemonic-io-lib 0.1.0  cksum d54d9fc783d32defa2274b346b66549a29756db8514200b12c3c98a2c4579f64  yanked False
```

The vendored `.cargo-checksum.json`'s `package` digest and `Cargo.lock`'s
`checksum` both equal that value.

## 3. Registry 0.1.0 vs git rev `6c24e628` — the difference is formatting only

The coordinator flagged that the two are not byte-identical and asked for any
difference beyond formatting to be reported immediately. Measured by re-vendoring
into a scratch directory and diffing:

* Of the crate's source files, **exactly one differs**: `src/remedy.rs`. The
  other eight are byte-identical.
* The difference is a rustfmt line-wrap of a single `format!` call. The format
  string and its arguments are byte-identical, so the emitted text cannot differ:

```
-        s.push_str(&format!("      \x20   {:<7} {recipe}\n", format!("{shell}:")));
+        s.push_str(&format!(
+            "      \x20   {:<7} {recipe}\n",
+            format!("{shell}:")
+        ));
```

* `Cargo.toml` differs only in cargo's normalisation of the dev-dependency table
  (`[dev-dependencies] tempfile = "3"` → `[dev-dependencies.tempfile]`).
* The published crate adds its own `Cargo.lock` (11 files vs 10) — normal for a
  registry tarball.

**No `#[allow]` attributes separate 0.1.0 from rev `6c24e628`.** The
coordinator's note mentioned two; they are not in this delta. Most likely they
separate 0.1.0 from a *different, earlier* rev pinned by the other repos. Flagged
so the convergence work is not planned against a wrong premise.

**Nothing else in `vendor/` moved.** `diff -rq` between the committed tree and a
fresh `cargo vendor --locked` is empty — so `vendor/miniscript` is byte-identical
to what F-354 vendored, and the local cargo produces the same bytes as F-354's
author's did.

## 4. Content proofs — the F-354 fix survived intact

```
sha256sum vendor/miniscript/src/descriptor/tr/taptree.rs
8f6bc3a95ca74051ee6c883f306ce19f691ad3fc11914558bf04e9bae6783371

cat vendor/miniscript/nightly-version
nightly-2026-05-08
```

Both match the required values (`8f6bc3a9…`, `nightly-2026-05-08`).

## 5. Gates

### Suite

```
Summary [  47.791s] 4007 tests run: 4007 passed, 20 skipped
```

### Gate — all four checks, exit 0

```
vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked; miniscript rev ff4732e5f75aa555682343cb180fa72ee3e8e9d5) ...
vendor-freshness: (1/4) OK — vendor/ satisfies Cargo.lock.
vendor-freshness: (2/4) OK — 7490 files across 169 crates match their recorded sha256.
vendor-freshness: (3/4) OK — 168 crates anchored to Cargo.lock checksums; 1 git-fork source(s) exempt by grounding (miniscript).
vendor-freshness: (4/4) OK — vendor/miniscript matches the tree grounded against upstream ff4732e5f75aa555682343cb180fa72ee3e8e9d5.
```

Also run with an empty `CARGO_HOME`: **exit 0**, same output.

### Other

* `cargo clippy --all-targets --locked -- -D warnings` → exit 0, zero warnings.
* `changelog-check.yml` fires on tag push only — not applicable to this merge.
* `sibling-pin-check.yml` scans `cargo install --git … --tag` lines against
  `scripts/install.sh`; `mnemonic-io-lib` is a Cargo dependency, not a sibling
  CLI, so it is out of that gate's scope.
* `miniscript-fork-tripwire.yml` asserts the `miniscript = { git = …` line still
  exists in the root `Cargo.toml`; unchanged by this merge.

### The suite count is 4007, not the 3960 named in the brief — resolved, not waived

The brief specified `3960 passed, 20 skipped` as "the baseline both sides
preserve". It is measurably not: **3960 is the pre-P3 baseline.** P3 adds exactly
**47** nextest tests, derived rather than estimated:

| Source | Source `#[test]` added | Targets | nextest tests |
| --- | --- | --- | --- |
| `src/argv_guard.rs` (new file) | 16 | bin only | 16 |
| `src/display_grouping.rs` | 2 | **lib and bin** | 4 |
| `tests/cli_p3_argv_refusal.rs` | 14 | 1 | 14 |
| `tests/cli_p3_grouping_surface.rs` | 8 | 1 | 8 |
| `tests/p3_declined_crate_items.rs` | 3 | 1 | 3 |
| `tests/p3_io_lib_pin.rs` | 2 | 1 | 2 |
| | | | **47** |

3960 + 47 = 4007. The two-test discrepancy in a naive `#[test]` count is
`display_grouping.rs` being compiled into **both** the lib and the bin target,
confirmed by nextest listing 13 tests under `mnemonic-toolkit
display_grouping::` and 13 more under `mnemonic-toolkit::bin/mnemonic
display_grouping::`.

This merge changes **no crate code** — `git diff --name-only HEAD` against the P3
parent touched only `ci/` and `vendor/` before the ruling, and after it also
`Cargo.lock`, `crates/mnemonic-toolkit/Cargo.toml` and one doc comment. It
therefore cannot move the count. Matching 3960 would have required dropping P3's
tests, which is the opposite of the requirement.

## 6. Mutation matrix

A standing note first: **the gate compiles nothing.** It is `cargo metadata`
plus file hashing, so the "a vendored-source change does not trigger
recompilation" trap does not apply to gate results. It does apply to build
results, and is addressed in §6.6 where it actually bites.

Every mutation started from a clean, gate-green baseline and was restored to one.

### 6.1 One byte flipped in `vendor/miniscript` → RED, restore → 0

Flipped byte 100 of `vendor/miniscript/src/descriptor/tr/taptree.rs`.

```
vendor-freshness: (1/4) OK — vendor/ satisfies Cargo.lock.
::error::vendor-freshness: 1 content defect(s) in vendor/:
  - vendor/miniscript/src/descriptor/tr/taptree.rs: CONTENT MISMATCH
      recorded 8f6bc3a95ca74051ee6c883f306ce19f691ad3fc11914558bf04e9bae6783371
      on disk  affc9c4f9d29567e2f79f6c067869dac3aa77940c5198cbec0492b2a229ab389
```

Exit 1, names the file. Restored → exit 0, digest back to `8f6bc3a9…`.

### 6.2 One byte flipped in `vendor/mnemonic-io-lib` → RED at check (2)

This is the case the merge exists to create. Flipped byte 200 of
`vendor/mnemonic-io-lib/src/remedy.rs`.

```
::error::vendor-freshness: 1 content defect(s) in vendor/:
  - vendor/mnemonic-io-lib/src/remedy.rs: CONTENT MISMATCH
      recorded 211fd01f71283ff8ab250499f4ab391f05bb1d6404879a69b82fb3bc11c9f502
      on disk  eb15f2acab7af86934e0e9554b002f61df26164f06b796ced9e2bc17c6059416
```

RED at the **integrity** check, naming the file — which is the proof the ruling
asked for: switching to the registry bought cargo's own anchoring rather than
merely removing a check.

**Control, measured on the same corrupted tree:** P3's pre-merge gate (extracted
from `59ade791` and run in place) returned **exit 0** and printed
`vendor-freshness: OK — vendor/ satisfies Cargo.lock.` A corrupted
`mnemonic-io-lib` was invisible before this merge.

Restored → exit 0.

### 6.3 The actual F-354 defect replayed → RED at the grounding check only

Restored the old `vendor/miniscript` tree from master (`taptree.rs` =
`588898e0…`, `nightly-version` = `nightly-2026-04-24`).

```
vendor-freshness: (1/4) OK — vendor/ satisfies Cargo.lock.
::error::vendor-freshness: 1 content defect(s) in vendor/:
  - vendor/miniscript/.cargo-checksum.json: GIT-FORK PROVENANCE MISMATCH
      grounded 30cc80f5ea57305f09790b661805b58cfdcd16aaaddd26c3769078eccd9a1277
      on disk  120099e4d8d706f50e2ccc23b11ca2025bc4bceaee632d2c1edd1bdb660438ab
```

**Exactly one defect, and it comes from check (4).** Check (1) printed OK. Checks
(2) and (3) contributed nothing — they cannot see this. Corroborated by running
check (2)'s logic standalone on the defective tree:

```
check(2) on the DEFECTIVE tree: 169 crates, 7490 files, 0 mismatches
```

Self-consistent and wrong, exactly as F-354 documented. Restored → exit 0 and
both content proofs back.

### 6.4 An additional unanchored source → RED at the set assertion

Created `vendor/fake-git-dep/` — a self-consistent vendored crate with a valid
`.cargo-checksum.json` carrying **no** `package` digest and no entry in
`Cargo.lock`.

```
::error::vendor-freshness: 1 content defect(s) in vendor/:
  - vendored crate(s) with NO offline provenance anchor and no grounding in this gate: fake-git-dep
      A git or path source cannot be checked against Cargo.lock (no published
      tarball digest). Ground it the way miniscript is grounded above, or drop it.
```

Checks (1) and (2) both passed it — cargo ignores the unused directory and the
crate is internally consistent. **The set assertion is the only thing standing
between the tree and a silently exempted new source.** Removed → exit 0.

### 6.5 A pin moved without re-grounding → RED with instructions

Moved the miniscript rev in both `Cargo.toml` and `Cargo.lock` (3 sites) to a
different 40-hex.

```
vendor-freshness: (1/4) OK — vendor/ satisfies Cargo.lock.
::error::vendor-freshness: 1 content defect(s) in vendor/:
  - the miniscript pin MOVED: Cargo.lock is at 95fdd1c5…, but this gate is
      grounded at ff4732e5…. The vendored tree cannot be verified against a
      rev nobody has checked. Re-vendor, verify the tree against upstream at the
      new rev, then update EXPECTED_GIT_FORK_REV / EXPECTED_GIT_FORK_MANIFEST_SHA256
      in ci/repro/vendor-freshness.sh (see GROUNDING in its header).
```

Note again that **check (1) passed**: cargo resolved a moved git pin against the
old vendored tree without complaint. That is precisely the hole check (4) closes.
Restored → exit 0.

### 6.6 The recompilation trap, addressed where it bites — cargo's own anchoring

The gate hashes files, so no compile is involved. The claim that actually needs a
compile is the ruling's central one: that the registry crate is anchored by cargo
rather than by us. Tested directly, with the corruption from §6.2 in place:

```
cargo clean -p mnemonic-io-lib
cargo build --locked --offline -p mnemonic-io-lib   # + vendored source replacement
```

```
error: the listed checksum of `…/vendor/mnemonic-io-lib/src/remedy.rs` has changed:
expected: 211fd01f71283ff8ab250499f4ab391f05bb1d6404879a69b82fb3bc11c9f502
actual:   eb15f2acab7af86934e0e9554b002f61df26164f06b796ced9e2bc17c6059416
```

Exit **101**. Cargo refused, at source-verification time, before compiling.

Restored and re-run: exit 0 with **`Compiling mnemonic-io-lib v0.1.0`** observed
— so the vendored path is genuinely what compiles, not a cached artifact. The
same check was run on the ordinary (networked) build path:
`cargo clean -p mnemonic-io-lib && cargo build --locked -p mnemonic-toolkit`
printed `Compiling mnemonic-io-lib v0.1.0` followed by `Compiling
mnemonic-toolkit v0.97.0`, and `cargo metadata` resolves the package to
`registry+https://github.com/rust-lang/crates.io-index`.

### 6.7 Two extra paths exercised while option A was live

Both were run against the option-A (table-driven) gate before the ruling landed,
and are recorded because they measured the gate's behaviour, not the design:

* **Duplicate entry in the grounded-source table** → exit 1, `duplicate dir in
  the grounded git-source table. Failing closed.`
* **A grounded vendored source deleted entirely** → caught by check (1)
  (`error: no matching package named 'mnemonic-io-lib' found`), so check (4)'s
  MISSING branch is defence-in-depth behind resolution rather than the first
  line.

## 7. Nothing was weakened

No check was loosened to make the merge pass. The gate script is byte-identical
to F-354's (`cmp` exit 0), so there is no room for a weakening to hide. The
unanchored set is *smaller* after this merge than the naive merge would have made
it — one named crate, not two — and §6.4 demonstrates it still fails closed on an
addition.

## 8. Ripples fixed, and one filed

**Fixed** (the change's own falsified prose, not tidying):

* `crates/mnemonic-toolkit/Cargo.toml` — the dependency comment asserted the
  crate "is not on crates.io" and justified the `rev` pin. Rewritten to state the
  registry pin and *why* a git source is avoided here.
* `crates/mnemonic-toolkit/tests/p3_io_lib_pin.rs` — module doc opened with
  "`mnemonic-io-lib` is not on crates.io; it is pinned by `rev`". Rewritten; the
  test's actual gate (a call that compiles) is unchanged and still passes.

A tree-wide grep for `not on crates.io` found no other live site — remaining hits
are about the *toolkit* not being on crates.io (still true) or are historical
design records.

**Filed as F-381** in `mnemonic-engrave`'s `design/FOLLOWUPS.md`:
`design/SPEC_vendor_freshness_ci_guard.md` still describes a one-check gate.
Measured: zero occurrences of `INTEGRITY`, `REGISTRY PROVENANCE`, `GIT-FORK`,
`grounding` or `(n/4)` in the 122-line spec that `vendor-freshness.sh:60` cites
as authoritative. The re-grounding procedure — a *verification* step, not a
transcription step — exists only as a comment inside the script it governs. This
drift predates the merge on both parents; owning phase is the one that closes
F-354/F-355/F-371.

## 9. Scope held

Untouched, as instructed: **F-355**, **F-371**, and the manifests of
`descriptor-mnemonic`, `mnemonic-key` and `mnemonic-engrave`. The live
`mnemonic-toolkit` checkout and the `_work/f354/` and `_work/p3tk/` worktrees
were not written to.

Worth noting for the coordinator's own preconditions: moving `mnemonic-io-lib`
to the registry removes this repo's need for a fourth `--config` stanza
entirely, which shrinks — rather than adds to — the F-355 reproducible-build
stanza surface. That is an observation, not an edit.

**Not pushed. Not tagged.**
