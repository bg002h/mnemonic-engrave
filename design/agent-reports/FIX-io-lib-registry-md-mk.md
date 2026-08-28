# FIX — `mnemonic-io-lib`: git-rev pin → published registry version, in `descriptor-mnemonic` and `mnemonic-key`

**Agent report. 2026-08-27.** Architect's ruling applied constellation-wide;
`mnemonic-toolkit` went first (`crates/mnemonic-toolkit/Cargo.toml:51`,
`mnemonic-io-lib = "0.1.0"`) and was the reference.

**Outcome in one line: `mnemonic-key` is DONE and committed
(`62271ae`, gate green, script unedited). `descriptor-mnemonic` is COMPLETE BUT
UNCOMMITTED and BLOCKED on a decision — its vendor-freshness gate cannot pass
without an edit, which the brief said to stop and report rather than apply.**

Worktrees written (and nothing else):

| repo | path | branch | base | HEAD now |
| --- | --- | --- | --- | --- |
| `descriptor-mnemonic` | `/scratch/code/shibboleth/_work/mdreg/descriptor-mnemonic` | `fix/io-lib-registry` | `bb2151dc` | `bb2151dc` (**uncommitted working tree**) |
| `mnemonic-key` | `/scratch/code/shibboleth/_work/mkreg/mnemonic-key` | `fix/io-lib-registry` | `ac61e44` | **`62271ae`** (clean) |

---

## 0. The published crate, verified independently

`https://index.crates.io/mn/em/mnemonic-io-lib`, fetched directly:

```
vers  = 0.1.0
cksum = d54d9fc783d32defa2274b346b66549a29756db8514200b12c3c98a2c4579f64   (64 hex)
yanked = False
deps  = [("tempfile", "^3", "dev")]
```

Matches the briefed prefix `d54d9fc783…`. That same 64-hex value appears in
**three** places after the change, in both repos: `Cargo.lock`'s `checksum`, the
`package` digest in `vendor/mnemonic-io-lib/.cargo-checksum.json`, and the index.
Before the change the first two were **absent** (`package` was `null`) — which is
the whole point of the ruling.

---

## 1. `descriptor-mnemonic` (md)

### 1.1 Manifest diff — `crates/md-cli/Cargo.toml`

```diff
-# P3 — the shared IO crate, pinned by exact rev.
+# P3 — the shared constellation IO crate, taken from the REGISTRY.
 #
 # NOT `path =`: a path does not resolve in a fresh CI checkout, and this repo
-# exists at more than one filesystem location on the author's box. NOT
-# crates.io: `mnemonic-io-lib` is not published there.
+# exists at more than one filesystem location on the author's box. And
+# deliberately NOT `git`+`rev` any more: a git source has no published tarball,
+# so neither `Cargo.lock`'s `checksum` nor `.cargo-checksum.json`'s `package`
+# can anchor the bytes that get vendored — the F-354 defect class, which is why
+# `ci/repro/vendor-freshness.sh` has to hand-ground every git source it
+# tolerates. A registry pin is anchored by cargo itself against an immutable
+# tarball on every build, so this crate needs no grounding, and md's only
+# remaining git source is the miniscript fork.
 #
 # `write_private` is the only item `md` adopts (P3 boundary table, 1 of 11) and
 # it is reached as `mnemonic_io_lib::write::write_private` — `write` is a
 # `pub mod` with NO root re-export, so the unqualified path is an E0425.
-mnemonic-io-lib = { git = "https://github.com/bg002h/mnemonic-engrave", rev = "6c24e62823e6c1ac02aa3862cd6020674bf58544", version = "0.1.0" }
+mnemonic-io-lib = "0.1.0"
```

`Cargo.lock`, `cargo metadata` rc 0, "Locking 1 package … Adding mnemonic-io-lib v0.1.0":

```diff
-source = "git+https://github.com/bg002h/mnemonic-engrave?rev=6c24e628…#6c24e628…"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "d54d9fc783d32defa2274b346b66549a29756db8514200b12c3c98a2c4579f64"
```

### 1.2 Vendor delta — **zero net, nothing unrelated rewritten**

`cargo vendor --locked vendor/` → **rc 0**.

* directories **before 126, after 126** — `diff` of the sorted listings is empty:
  **0 added, 0 removed**.
* the *only* paths that moved are inside `vendor/mnemonic-io-lib/`:
  `M .cargo-checksum.json`, `M Cargo.toml`, `M src/remedy.rs`, `?? Cargo.lock` (new).
* cargo's own emitted source config is now the **three-block** form
  (crates-io + `rust-miniscript` + vendored-sources) — no `mnemonic-engrave`
  block. Cargo, not I, decided that.

### 1.3 GATE — **RED, exit 1. This is the stop-and-report.**

CI invocation read, not guessed: `.github/workflows/vendor-freshness.yml:61`
runs `bash ci/repro/vendor-freshness.sh`.

```
$ bash ci/repro/vendor-freshness.sh
EXIT=1
::error::vendor-freshness: could not derive the mnemonic-io-lib rev from Cargo.lock
(expected a 'mnemonic-engrave?rev=<40-hex>' source line). Failing closed.
```

The gate **asserts that `Cargo.lock` must contain a
`bg002h/mnemonic-engrave` git source** (`ci/repro/vendor-freshness.sh:58-63`,
an unconditional `[ -z "$IOLIB_REV" ] && exit 1`). The ruling says it must not.
So the gate does literally assert what the ruling contradicts, and per the brief
**I did not adjust it and did not commit md.**

**Provenance of that assertion, so the decision can be made on facts.** It is not
a long-standing invariant: it was added **two commits ago on this same branch**,
by `9914ae41` — *"P3 row 1, consequence: re-vendor and cover the new git source
in the vendor gate"* — whose only non-vendor file was
`ci/repro/vendor-freshness.sh` (+29/-9). It is scaffolding for the pin now being
removed.

**The minimal edit, MEASURED not proposed.** `git show 9914ae41^:ci/repro/vendor-freshness.sh`
is the pre-pin three-block gate. Restored byte-for-byte into the worktree,
**run against the post-change tree, then deleted again**:

```
EXIT=0
vendor-freshness: resolving Cargo.lock against committed vendor/ (offline, locked;
                  miniscript rev ff4732e5f75aa555682343cb180fa72ee3e8e9d5) ...
vendor-freshness: OK — vendor/ satisfies Cargo.lock.
```

So the edit required is exactly **revert `9914ae41`'s gate hunk** — drop the
`IOLIB_REV` derivation and its fail-closed block, drop the
`| grep -v "mnemonic-engrave?rev=…"` exemption from the `UNCOVERED` filter (which
*tightens* the guard), drop the three `mnemonic-engrave` `--config` stanzas, and
restore the two comment paragraphs. The miniscript fail-closed guard and the
uncovered-git-source guard both remain intact. Nothing is weakened; one dead
exemption is removed. **The worktree does NOT carry this edit.**

### 1.4 Empty `CARGO_HOME`

The gate's early exit prevents running the script itself, so the underlying
resolve was run directly with **cargo's own emitted three-block config**,
`--locked --offline`, `CARGO_HOME` = a fresh empty directory:

```
EXIT=0
```

and afterwards that directory contained only `.global-cache` and `.package-cache`
— **nothing was fetched**, no `git/`, no `registry/`. No warm `~/.cargo/git`
could have supplied anything.

### 1.5 Corruption test — both directions

Built under the vendored source config with `CARGO_TARGET_DIR=target/repro`,
`cargo clean -p mnemonic-io-lib` before each run so the compile really happened.
`mnemonic-io-lib` has **no** normal dependencies, so `cargo build -p mnemonic-io-lib`
compiles exactly that one crate.

| step | rc | evidence |
| --- | --- | --- |
| baseline | **0** | `Compiling mnemonic-io-lib v0.1.0` |
| one byte flipped in `vendor/mnemonic-io-lib/src/write.rs` | **101** | see below |
| restored | **0** | `Compiling mnemonic-io-lib v0.1.0` — **the trap was checked** |

The corruption was `'C'` → `'K'` at byte 6, *inside a doc comment*, same file
length. Deliberately chosen so **rustc alone would not have caught it** — only
the checksum can. Cargo's own error:

```
error: the listed checksum of `…/vendor/mnemonic-io-lib/src/write.rs` has changed:
expected: 860914ee4e789f282c3ead92797cd0bbe43c2a4f95a636c350a309e198016787
actual:   ff43092fd8d88ce01755dd512a8914a375326fcc9f03b4b62eecc321cc7d7217

directory sources are not intended to be edited, …
```

**Second corruption — the anchor the git pin could never have.** Tampering the
`package` digest in `.cargo-checksum.json` (first hex char `d`→`0`) makes plain
`cargo metadata --locked --offline` red:

```
EXIT=101
error: checksum for `mnemonic-io-lib v0.1.0` changed between lock files
…
unable to verify that `mnemonic-io-lib v0.1.0` is the same as when the lockfile was generated
```

Under the git rev **both sides of that comparison were `null`**, so this check
did not exist at all. Restored; back to rc 0.

### 1.6 Suite — matches baseline exactly

`cargo nextest run --locked`, rc 0:

```
    Starting 832 tests across 68 binaries (2 tests skipped)
     Summary [  19.223s] 832 tests run: 832 passed, 2 skipped
```

Briefed baseline was 832 passed / 2 skipped. **Exact match, nothing adjusted.**

### 1.7 `cargo package` — the failure class is gone, and md now packages clean

```
$ cargo package --no-verify --allow-dirty -p md-cli
EXIT=0
   Packaging md-cli v0.13.0
    Updating crates.io index
    Packaged 105 files, 673.1KiB (170.1KiB compressed)
```

(`--allow-dirty` only because the change is uncommitted; the sole complaint
without it was the dirty `crates/md-cli/Cargo.toml`.)

Baseline for comparison, from `bb2151dc`'s own commit message: with the git pin
*and* `version`, `cargo package -p md-cli --no-verify` was **rc 101,
"no matching package named `mnemonic-io-lib` found … crates.io index"**; without
`version` it was rc 101, *"all dependencies must have a version specified when
packaging"* — described there as **PERMANENT** for a crate with 19 published
versions. Both are now gone. **md-cli's publish path is unblocked.**

---

## 2. `mnemonic-key` (mk) — committed, `62271ae`

### 2.1 Manifest diff — `crates/mk-cli/Cargo.toml`

Same substitution; the comment was rewritten to carry the reason and to drop the
now-false *"A GIT SOURCE IN Cargo.lock IS LOAD-BEARING FOR TWO CI SURFACES … see
F-341"* paragraph.

```diff
-mnemonic-io-lib = { git = "https://github.com/bg002h/mnemonic-engrave", rev = "6c24e62823e6c1ac02aa3862cd6020674bf58544", version = "0.1.0" }
+mnemonic-io-lib = "0.1.0"
```

`Cargo.lock` diff is **byte-identical to md's** (same removed `source` line, same
added `source` + `checksum` lines).

### 2.2 Vendor delta — zero net

`cargo vendor --locked vendor/` → rc 0. **134 directories before, 134 after; 0
added, 0 removed.** Same four paths inside `vendor/mnemonic-io-lib/` and nothing
else. Cargo's emitted config is now the **two-block** form — mk has **zero**
`source = "git+…"` entries in `Cargo.lock`.

### 2.3 GATE — **GREEN, exit 0, script BYTE-FOR-BYTE UNCHANGED**

CI invocation: `.github/workflows/vendor-freshness.yml:61`,
`bash ci/repro/vendor-freshness.sh`.

```
EXIT=0
vendor-freshness: resolving Cargo.lock against committed vendor/ (offline, locked; mnemonic-io-lib rev none) ...
vendor-freshness: OK — vendor/ satisfies Cargo.lock.
```

`git diff ac61e44..62271ae -- ci/repro/vendor-freshness.sh` is **empty**. This is
the falsifiable test the brief named, and it passes here: mk's gate derives its
block list from `Cargo.lock` and, uniquely, guards the fail-closed branch with
`[ "$GIT_SOURCES" != "0" ]`, so with no git source it degrades cleanly to the
two-block form.

### 2.4 Empty `CARGO_HOME`, with a negative control

```
CARGO_HOME=<fresh empty dir> bash ci/repro/vendor-freshness.sh   → EXIT=0
```

and afterwards the directory held only `.global-cache` / `.package-cache`.

**Control — proving the test can fail.** Same empty home, `--locked --offline`,
*without* the vendored-sources redirect:

```
EXIT=101
error: no matching package named `bitcoin` found
location searched: crates.io index
```

So the green came from `vendor/`, not from a warm cache.

### 2.5 Corruption test — both directions

| step | rc | evidence |
| --- | --- | --- |
| baseline build | **0** | `Compiling mnemonic-io-lib v0.1.0` |
| one byte flipped in `src/write.rs` (same `'C'`→`'K'`) | **101** | `error: the listed checksum of …/vendor/mnemonic-io-lib/src/write.rs has changed:` `expected: 860914ee…` / `actual: ff43092f…` |
| restored | **0** | `Compiling mnemonic-io-lib v0.1.0` |

`cargo clean -p mnemonic-io-lib` ran before each; the `Compiling` line was
**verified present**, per the brief's trap.

**Package-digest tamper → the GATE itself reds**, rc 1:

```
error: checksum for `mnemonic-io-lib v0.1.0` changed between lock files
::error::vendor/ is out of sync with Cargo.lock — …
```

Restored → gate rc 0.

### 2.6 Suite — matches baseline exactly

```
    Starting 370 tests across 33 binaries
     Summary [   0.241s] 370 tests run: 370 passed, 0 skipped
```

rc 0. Briefed baseline 370 passed / 0 skipped. **Exact match.**

### 2.7 `cargo package`

```
$ cargo package --no-verify -p mk-cli        # on the CLEAN committed tree
EXIT=101
   Packaging mk-cli v0.13.0
    Updating crates.io index
error: failed to prepare local package for uploading
Caused by:
  failed to select a version for the requirement `mk-codec = "^0.5.0"`
  candidate versions found which didn't match: 0.4.2, 0.4.1, 0.4.0, ...
```

It got **past** `mnemonic-io-lib` and failed on the **pre-existing unpublished
sibling** `mk-codec 0.5.0` — exactly the state the old manifest comment recorded
as the baseline. The io-lib class ("all dependencies must have a version
specified when packaging", permanent for a git source) is structurally
impossible now.

---

## 3. The one file that differs between crates.io `0.1.0` and git rev `6c24e628`

**Confirmed exactly as briefed, and nothing beyond it.** Full recursive `diff -r`
of the pre-change vendored git tree against the post-change vendored registry
tree, run in **both** repos (the two pre-change trees were themselves byte-identical,
and so are the two post-change trees):

| path | nature |
| --- | --- |
| `src/remedy.rs` | **the one source difference** — a rustfmt line-wrap of a single `format!`; string and arguments identical |
| `Cargo.toml` | cargo-generated: `[dev-dependencies] tempfile = "3"` → `[dev-dependencies.tempfile] version = "3"`. Manifest normalisation cargo performs when publishing/vendoring, not a source change |
| `.cargo-checksum.json` | cargo-generated: gains the `package` digest (was `null`) |
| `Cargo.lock` | **new file** — the published tarball ships the crate's own lock; a git vendor did not |

The `remedy.rs` hunk in full:

```diff
-        s.push_str(&format!("      \x20   {:<7} {recipe}\n", format!("{shell}:")));
+        s.push_str(&format!(
+            "      \x20   {:<7} {recipe}\n",
+            format!("{shell}:")
+        ));
```

**No divergence beyond the briefed one.** No stop-and-report on this axis.

---

## 4. Differences between the two repos — the interesting part

They did **not** behave the same. Four divergences, all measured:

**(a) The gates are structurally different, and only mk's tolerates the ruling.**
Both were "ported from `mnemonic-toolkit`", but:

* `mnemonic-key/ci/repro/vendor-freshness.sh:58-64` guards its fail-closed branch
  with `[ "$GIT_SOURCES" != "0" ]` and builds `SRC_CONFIG` inside
  `if [ -n "$IO_LIB_REV" ]`. Zero git sources is a supported state. **rc 0, unedited.**
* `descriptor-mnemonic/ci/repro/vendor-freshness.sh:58-63` has no such guard —
  `[ -z "$IOLIB_REV" ] && exit 1`, unconditionally. Zero io-lib git sources is a
  hard failure. **rc 1.**

Neither resembles `mnemonic-toolkit`'s gate, which asserts an *unanchored set*
(`unexpected = [d for d,_ in unanchored if d != fork_dir]`,
`ci/repro/vendor-freshness.sh:204`) rather than the *presence of a named git
source* — which is precisely why the toolkit's gate needed no edit and md's does.
**The set-assertion shape is the portable one; the derive-a-named-rev shape is
not.** Filed as **F-391**.

**(b) `cargo package` outcomes differ.** md-cli now packages at **rc 0** (105
files); mk-cli still fails at **rc 101** on the unpublished sibling
`mk-codec 0.5.0`. Pre-existing and unrelated to this change — mk's own manifest
comment recorded it as the baseline.

**(c) md keeps a git source; mk has none.** md still carries the `rust-miniscript`
`[patch.crates-io]` fork, so its `Cargo.lock` retains one `source = "git+…"`. mk's
now has **zero**.

**(d) md has a stale doc comment mk does not.**
`descriptor-mnemonic/crates/md-cli/tests/crate_pin.rs:4` still says
*"`mnemonic-io-lib` is pinned by exact git rev in `Cargo.toml`"*. The test itself
asserts **behaviour** (`write_private` creates `0600`), not the rev, so it passes
— but the line is now false. **Not edited**, since md is not being committed; it
belongs with the gate fold.

---

## 5. Consequences for open follow-ups — reported, NOT folded

Not touched, per scope. Both are direct consequences of the ruling and both
appear to be **overtaken by it**:

**F-341** (`mk`'s tag-time reproducible musl build cannot be fixed by any input
its shared workflow accepts). Its premise is that
`mnemonic-key/.github/workflows/musl-binaries.yml:80` passes `miniscript_rev: ""`,
selecting a TWO-block `--config` form, while `Cargo.lock` carried a git source
that form cannot redirect (its table: two-block → 101). **mk now has no git source
at all, and the two-block form resolves at rc 0 under an empty `CARGO_HOME`
(§2.4). The premise is gone.**

**F-333** (`md`'s release recipe, same class). Its table read: two-block → 101 on
`miniscript`; three-block → 101 on `mnemonic-io-lib`; four-block → 0. Re-measured
under an empty `CARGO_HOME` after this change:

| `--config` form | rc before | **rc now** | fails on |
| --- | --- | --- | --- |
| TWO-block — what `man-pages.yml:111` passes today (`miniscript_rev: ""`) | 101 | **101** | `miniscript` (pre-existing since 2026-08-20) |
| THREE-block (+ miniscript) — what the shared workflow **can** emit | 101 | **0** | — |
| FOUR-block (+ mnemonic-engrave) | 0 | n/a | — |

**F-333 changes character**: it was *"needs a fourth block the shared workflow has
no slot for — a change in another repo"*. It is now *"set the existing
`miniscript_rev` input to `ff4732e5f75aa555682343cb180fa72ee3e8e9d5`"*, which is
a one-line change in `descriptor-mnemonic`'s own workflow. **F-324**
(`mnemonic-secret`, same class) was not examined — out of scope — but if `ms` has
no miniscript fork it is likely in mk's position, i.e. resolved outright.

---

## 6. What is left open

1. **DECISION REQUIRED — md's gate.** Approve reverting `9914ae41`'s
   `ci/repro/vendor-freshness.sh` hunk (§1.3, measured green), or direct
   otherwise. Nothing in md is committed until this is settled.
2. **md's commit.** Once (1) is settled: stage `Cargo.lock`,
   `crates/md-cli/Cargo.toml`, `vendor/mnemonic-io-lib/` (incl. the new
   `Cargo.lock`), the gate revert, and the `crates/md-cli/tests/crate_pin.rs:4`
   doc-comment line. **The working tree already carries everything except the
   last two**; if it is ever lost, it regenerates exactly with: edit the manifest
   to `mnemonic-io-lib = "0.1.0"`, `cargo metadata --format-version 1`,
   `cargo vendor --locked vendor/`.
3. **F-391** — filed: the two gates disagree on shape, and both are
   resolution-only (measured: a one-byte vendored-file corruption passes mk's
   gate at rc 0 while the build reds at 101). `mnemonic-toolkit`'s checks (2)-(4)
   exist for exactly this.
4. **Nothing pushed, nothing tagged.** Per the brief.
