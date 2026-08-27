# IMPL-P3-md — the `md` branch of P3, executed

**Agent:** the `md` implementer, one of three parallel branches.
**Worktree:** `/scratch/code/shibboleth/_work/p3md/descriptor-mnemonic`, branch
`impl/p3-md`, from `beb2fb2` (clean).
**Date:** 2026-08-27.
**Plan:** `mnemonic-engrave/design/IMPLEMENTATION_PLAN_P3_md_mk_mnemonic.md`
(R0 GREEN, 0C/0I).

**Nothing outside the worktree was written except this report and
`design/FOLLOWUPS.md` in `mnemonic-engrave`.** No join was touched.

---

## 1. ROWS COMPLETED — 7 of the plan's 19, plus one consequence commit

| # | row | commit | gate result |
| --- | --- | --- | --- |
| 1 | THE PIN (md's manifest) | `e61e0afc` | build 0 · 806 passed |
| 2 | THE md HEADER LEAVES stdout | `7a5b9971` | RED first, then 808 passed |
| 3 | THE md STDOUT UNGROUPS + THE CARD | `5a7a179d` | RED first, then 811 passed |
| 4 | THE md SEPARATOR NARROWS | `9f34a729` | RED first, then 812 passed |
| 5 | THE md READER IS HOISTED | `60761ac3` | RED first, then 815 passed |
| 6 | THE md CHANNELS | `63b0ee83` | RED first (10 of 11), then 828 passed |
| 20 | THE DECLINE, ASSERTED (md's share) | `5e9c574c` | regression-gated, 832 passed |
| — | row 1's consequence: re-vendor + vendor gate | `9914ae41` | gate 1 → 0 |

Rows 7–12 are `mk`'s, 13–15 are `mnemonic`'s, 16–19 are joins. None was touched.

**`md` was invoked by absolute path in every measurement.** The login shell
aliases `md` to `mkdir -p`, and `~/.cargo/bin/md` is a stale install.

---

## 2. FINAL SUITE LINE, VERBATIM

```
     Summary [  18.448s] 832 tests run: 832 passed, 2 skipped
```

Baseline was **805 passed, 2 skipped**. +27 tests, 0 deleted. The 2 skips are
the pre-existing ones the plan records and does not claim are harmless.

## 3. EVERY CI STEP, WITH ITS EXIT CODE

`.github/workflows/ci.yml`, read and executed step by step. `RUSTFLAGS="-D
warnings"` is set repo-wide in that file and was exported for the jobs that
declare it.

| workflow / job | step | exit |
| --- | --- | --- |
| ci.yml `test` | `cargo test --workspace --all-targets` | **0** |
| ci.yml `test` | `cargo test --workspace --doc` | **0** |
| ci.yml `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| ci.yml `fmt` | `cd design && sha256sum -c display-grouping-vectors.tsv.sha256` | **0** |
| ci.yml `fmt` | `cargo fmt --all --check` | **0** |
| ci.yml `doc` | `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps --document-private-items` | **0** |
| ci.yml `freebsd-compile-gate` | `cargo check --target x86_64-unknown-freebsd -p md-cli` | **0** |
| vendor-freshness.yml | `bash ci/repro/vendor-freshness.sh` | **0** |
| vendor-freshness.yml | the same, under an **empty `CARGO_HOME`** | **0** |
| — | negative control: `vendor/mnemonic-io-lib/` moved aside | **1** (correct) |
| man-pages.yml canary | `md gen-man --out man`, then the `*-help*.1` check | **0**, 13 pages, canary clean |

**THREE CI LEGS COULD NOT BE RUN ON THIS BOX, and are reported as unrun rather
than as green:**

| leg | why | what was tried |
| --- | --- | --- |
| ci.yml `musl-check` x86_64 | no musl C toolchain | `cargo check -p md-cli --target x86_64-unknown-linux-musl --all-targets` → **101**, `error occurred in cc-rs: failed to find tool "x86_64-linux-musl-gcc"` |
| ci.yml `musl-check` aarch64 | no `cross`, no `docker`/`podman` | not attempted past that |
| ci.yml `test` windows + macos | no runners | `cargo check -p md-cli --target x86_64-pc-windows-gnu --all-targets` → **101**, `failed to find tool "x86_64-w64-mingw32-gcc"` |

Both failures are the **C** build of `secp256k1-sys`; no Rust of this diff was
reached, so neither is evidence about the code. The FreeBSD whole-crate check
is the one cross-target job that did run, and it passed — a non-Linux,
non-glibc target compiling the whole crate including `main.rs`.

Residual portability risk was read rather than run: every `PermissionsExt` use
and every mode assertion in the new tests is inside `#[cfg(unix)]`, and the one
non-portable test (`md_writes_a_world_readable_stdout_without_refusing`) is
`#[cfg(unix)]` whole. `mnemonic-io-lib` is dependency-free std with the same
`#[cfg(unix)]` shape, and its own header states the non-Unix behaviour.

Not triggered by this diff, checked rather than assumed: `fuzz-smoke.yml`
(paths `fuzz/**`, `crates/md-codec/src/**`), `bitcoind-differential.yml`
(cron / dispatch), `man-pages.yml`'s release jobs (tags only).

## 4. `git diff --stat beb2fb2..HEAD`

```
29 files changed, 2505 insertions(+), 113 deletions(-)
```

Of which `vendor/mnemonic-io-lib/` is 11 files / 1187 lines of verbatim
`cargo vendor` output. **Excluding `vendor/`: 18 files, 1318 insertions, 113
deletions.**

---

## 5. WHAT THE PHASE'S HEADLINE GATE DOES NOW — run, not described

§5 condition 1: *"`md encode` on a CHUNKING policy pipes into `me sysw pack`
with no flags and no `grep`, at exit 0."* A keyed 2-of-2 `wsh(multi)` with two
`--key`, two `--fingerprint` and `--path`:

| state | `me sysw pack --in <md encode stdout> --out payload.bin` |
| --- | --- |
| after row 2 (header on stderr, grouping still on) | **rc 4** — *"record 0 … is not a form this container can place"* |
| after row 3 (also ungrouped) | **rc 0**, payload 396 bytes, mode 0600 |

And §10's opening call, end to end:

```
md encode --in wallet.template --key … --out wallet.md1     rc 0
  wallet.md1 = -rw------- 345 bytes, 4 unbroken md1 lines; stdout 0 bytes
me sysw pack --in wallet.md1 --out payload.bin              rc 0, 396 bytes
```

`me` here is the installed **0.7.0**, which has **no `--expect` flag** — row
19's exact invocation (`--expect descriptor,cosigner,transaction`) exits 2 from
clap on it. That is the join's to run, not this branch's, and it is called out
so the join does not read a clap error as this cycle's defect.

**THE ENGRAVING CARD, as shipped** (the plan's specified shape, `ms`-like,
`label: value`, no prefix char, advisory last):

```
chunk-set-id: 0x95075
md1fj 5r4ps pq2tv yyy4q qxpps g2z7z 883w6 pt24m enw3t sf9m5 …
… one line per chunk …
group size: 5
separator: space
note: stdout is watch-only — public keys only, cannot spend
```

---

## 6. FINDINGS — where the plan and the repo disagreed

### 6.1 CRITICAL-shaped, and it is the reason this branch has an eighth commit: row 1's *"three files, no other edit"* is false in this repo

`descriptor-mnemonic` commits a 108 MB `vendor/` tree and gates every push
touching `Cargo.lock` or a crate manifest with `ci/repro/vendor-freshness.sh`.
Row 1 touches both. The gate failed CLOSED one commit after the pin, named the
uncovered source, and named its own fix.

**Measured three ways under an empty `CARGO_HOME`** (the isolation is
load-bearing — F-324 records a false GREEN from omitting it):

| `--config` form | rc | fails on |
| --- | --- | --- |
| TWO-block — what `man-pages.yml` passes today | **101** | `miniscript` |
| THREE-block — what `vendor-freshness.sh` had | **101** | `mnemonic-io-lib` |
| FOUR-block — what `vendor-freshness.sh` has now | **0** | — |

**The first row is the finding inside the finding: `md`'s tag-time reproducible
build was already broken before P3, by the 2026-08-20 miniscript pin.** P3 adds
a second uncovered git source to a recipe that could not resolve the first.

Fixed on the branch: `cargo vendor vendor/` (which added **exactly one**
directory and moved nothing else in the 125-entry tree) and the mnemonic-engrave
stanza in `vendor-freshness.sh`, rev derived from `Cargo.lock`, failing closed on
an empty match. Verified green under an empty `CARGO_HOME`, with a negative
control that reds, and with the fail-closed guard still tripping on a synthetic
third git source.

**Deliberately NOT fixed here:** `man-pages.yml`'s two `musl-binaries` legs.
This is where the branch diverges from P2's F-324 handling of the same class,
and the reasons are stated so the divergence can be overruled: the workflow
triggers on tags only and needs docker + GHCR (**an edit to a workflow that
cannot be run is a hypothesis**); its `repro:` job calls a **mnemonic-toolkit**
reusable workflow whose only git-source knob is `miniscript_rev`, so `md` cannot
tag until that cross-repo change lands regardless; and P3 is not what made it
red. **Filed as F-333, non-deferrable past the next `md-cli-v*` tag.**

**This is now 2 of 2 repos where the plan's row 1 wording was untrue** (F-324
for `ms`, F-333 for `md`). **The `mk` branch should be asked whether
`mnemonic-key` carries a `vendor/` tree and a vendor gate.**

### 6.2 Plan cell inaccuracies, all minor, none blocking

- **Row 3's gate says the plan's four-way packability measurement gives "0 with
  a 391-byte payload".** Re-measured here: **396 bytes** with `me` 0.7.0 and
  this fixture. The direction is right, the number is fixture- and
  version-dependent; the branch asserts the *property* (no whitespace on any
  stdout line) rather than a byte count, since the byte count is not a fact
  about `md`.
- **Row 4's gate says *"the test does not exist yet"*.** True of the refusal
  test, but `cmd_encode.rs::encode_separator_hyphen` DID exist and pinned
  `--separator hyphen` at exit 0. Row 3's cell names the tests it reddens; row
  4's does not. The test was replaced, not deleted.
- **§1.1 says `encode` has "one emission site on stdout".** There are two: the
  `--policy-id-fingerprint` line is the second, and it defeats `me sysw pack`
  (rc 4 on record 1, reproduced). Filed as **F-331** — closure condition 6 is
  false for that invocation, and the remedy is a §6a scope ruling rather than an
  implementer's edit.

### 6.3 A gate that could not fail, caught and strengthened

`encode_refuses_both_the_positional_and_in` was the **1 of 11** row-6 tests that
passed before the change — because `--in` was an *unknown flag* and clap already
exited 2. Exit code alone was a false PASS. It now also requires the message to
name the conflict, and was re-measured RED:

```
must refuse as a conflict, not as an unknown flag; got "error: unexpected
argument '--in' found\n\n  tip: a similar argument exists: '--fingerprint'"
```

### 6.4 A surface change row 6 implied but did not name

Relaxing four positionals to `required_unless_present = "in_file"` changes `md
gui-schema`'s report of `decode`'s positional from `required: true` to `false`.
Two pre-existing tests pinned `true`. They were folded to assert `false` **plus**
that `--in` is in the schema, and the requirement itself is pinned behaviourally
instead — `every_relaxed_verb_still_refuses_when_no_input_is_supplied`, because
*"required unless present"* is exactly the shape that can make a verb silently
accept nothing. Measured: all six still exit 2 with no input. The GUI's drift
gate compares `choices` and `default_value` only and pins `md` by version tag,
so no join is created.

### 6.5 Text the diff falsified without touching

Row 3 made `--group-size`'s and `--separator`'s clap help false — they still said
they shaped *"the emitted md1 string"*. Caught by sweeping rather than by
reading the diff, and folded in row 4. `md gui-schema` still emits
`choices: null` for both flags, re-measured, so the GUI mirror join's one-sided
guard stays unarmed exactly as the plan records.

---

## 7. DECISIONS THE ROWS LEFT OPEN, AND HOW THEY WERE MADE

- **`--out` is on `encode` only.** §6b's `--out` is *"write the ARTIFACT to a
  file"*, and `encode` is the only `md` verb whose stdout is an artifact —
  `decode` emits a template, `verify` emits `OK`, `inspect`/`repair` emit
  reports, and §6a puts all of those out of scope.
- **`--in` is on six verbs**, not the five the plan's sweep counted: the five
  plus `bytecode`, because row 5 already ruled `bytecode` has the identical
  defect and is the identical verb class.
- **`md`'s `--group-size` default stays 5.** The 5 → 0 flip in the plan is
  `mnemonic`'s row 13; the plan's own card example shows `group size: 5`, which
  is only coherent if md's default is unchanged. After row 3 the flag cannot
  reach stdout at all, so the default is a card-shape choice.
- **`--separator` accepts `space` and the literal `" "` and nothing else.**
  "Whitespace only" was not widened to tab — that would be new surface.
- **No version number in the refusal text.** A draft said *"retired in v0.14"*;
  `md-cli` is 0.13.0 and this branch cuts no release.
- **`--from-policy` conflicts with `--in`** — it supplies the same thing, and
  without the conflict the file would be silently ignored.

## 8. WHAT WAS MUTATION-VERIFIED RATHER THAN READ

1. **Row 5, the fourth verb.** Reverting `bytecode.rs` alone to
   `strip_md1_inputs` reds `dash_reads_stdin_on_all_four_verbs_byte_for_byte`
   naming `bytecode` — the verb the plan says the first draft's gate omitted.
2. **Row 20, the decline scan.** Adding a `mnemonic_io_lib::exit::WriteBlock`
   return type to `cmd/mod.rs` reds the boundary test, quoting the path.
3. **The vendor gate's fail-closed guard.** A synthetic third
   `source = "git+…"` in `Cargo.lock` reds it at exit 1, naming that source.
4. **The vendor gate's positive result.** `vendor/mnemonic-io-lib/` moved aside
   → exit 1, *"no matching package named `mnemonic-io-lib` found"*.

An earlier version of the row-20 scan matched **bare item names** and flagged its
own doc comment explaining the decline. A mention is not an adoption; it now
matches paths rooted at `mnemonic_io_lib::`, and carries two guards against a
false PASS (it asserts it found >10 files, and at least one crate path).

## 9. FOLLOW-UPS FILED

Committed to `mnemonic-engrave/design/FOLLOWUPS.md` as `906ce59`, one commit,
nothing else touched.

- **F-331** — `md encode --policy-id-fingerprint` writes a non-artifact line to
  `encode`'s stdout; `me sysw pack` rc 4 on record 1. Owning phase: whichever
  cycle rules whether §6a binds an opt-in diagnostic flag.
- **F-332** — the terminal-write decline is asserted by a mutation-verified
  source scan and the world-readable arm, but not by a pty. Named rather than
  half-built: a pty test would silently do nothing on five of seven CI legs.
  Owning phase: the test-infra residue.
- **F-333** — the vendor / reproducible-build breakage above. Owning phase:
  before the next `descriptor-mnemonic-md-cli-v*` tag, non-deferrable.

## 10. CONSULTS

**None.** No ambiguity survived reading the plan, the spec and the source, so no
`fable` consult was dispatched.

## 11. THINGS NOT DONE, AND WHY

- **The three CI legs in §3** — no musl C toolchain, no `cross`, no docker, no
  windows/macos runner on this box. Reported as unrun.
- **`man-pages.yml`** — §6.1.
- **`docs/verify-reproducibility.md`'s "fork-free / two `--config` overrides"**
  — wrong since the miniscript pin, more wrong now. Correcting it while the
  workflow still passes two blocks would document a state that does not exist,
  so it moves in the same pass as the workflow. Recorded inside F-333.
- **`md-cli`'s version and CHANGELOG** — no row cuts a release, and the plan
  names a release for `mnemonic-toolkit` only.
- **Rows 7–19** — other branches' and the joins'.

## 12. ONE THING THE CONTROLLER SHOULD ACT ON FIRST

**Ask the `mk` branch whether `mnemonic-key` commits a `vendor/` tree and runs a
`vendor-freshness` gate.** Two of the three repos so far had one, the plan
mentions neither, and in both the pin reddened a gate within one commit of
landing. If `mk` does too, its row-1 commit is red right now and the branch may
not have looked.
