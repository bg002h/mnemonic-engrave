# IMPL-P3-mk — the `mk` branch of P3, executed

**Agent:** the `mk` implementer, one of three parallel branches.
**Worktree:** `/scratch/code/shibboleth/_work/p3mk/mnemonic-key`, branch `impl/p3-mk`.
**Base:** `c5739fc` — 337 passed, 0 skipped.
**Head:** `ac61e44` — **370 passed, 0 skipped**, tree clean.
**Plan:** `mnemonic-engrave/design/IMPLEMENTATION_PLAN_P3_md_mk_mnemonic.md`.

Every number below was produced by running a command and reading a captured
exit code. Nothing is described from a doc comment or from the plan's own text.

---

## 1. ROWS COMPLETED

Table order. Seven commits, one per row, each carrying its gate result.

| # | row | commit | gate |
| --- | --- | --- | --- |
| 1 | the pin (mk side) | `ac61e44` | regression-gated; see §5 — **built LAST, after a reversal** |
| 7 | the mk ungrouping + the card | `fd5e482` | RED-first, 4 tests |
| 8 | the mk separator | `05ba14f` | RED-first, 1 test × 4 spellings |
| 9 | the mk blank line | `9c0e4aa` | RED-first, fixture-independent shape |
| 10 | the mk channels | `2276a7a` | RED-first, 10 tests |
| 11 | the mk exit code | `2e31e9d` | RED-first, 3 of 5 (2 are declared controls) |
| 12 | the md1 set flag | `3f5fc52` | RED-first, 7 tests |
| 20 | the decline, asserted (mk half) | `91eff55` | regression-gated + **mutated to prove it can fail** |

Rows 2–6 (`md`) and 13–15 (`mnemonic`) are other branches'. Rows 16–19 are
joins and were not touched.

### Row 7 — the mk ungrouping, and the card
stdout is the artifact, ungrouped (§6a). `--group-size` / `--separator` now
shape a **stderr engraving card** — `mk` had none, only the one-line
output-class `note:`. Shape follows `ms`'s: grouped string(s) first, then
`group size:` and `separator:`, then the existing advisory, still last.

Packability, run end to end against `me sysw pack --out`:

| | rc | payload |
| --- | --- | --- |
| default output, BEFORE | **4**, "record 0 … is not a form this container can place" | none |
| default output, AFTER | **0** | 244 bytes |

244 is the byte count the plan recorded for `--group-size 0`; the new default
reaches it with no flag and no `grep`.

`--group-size 5` and `--group-size 0` now produce **byte-identical stdout** —
the assertion that says the flag no longer reaches stdout at all. The plan's
named test `encode_default_groups_space_5` is replaced, and it was confirmed it
would RED rather than assumed: char 5 of stdout line 1 is now `s`, where it
asserted `' '`. The 22 existing `--group-size` occurrences across 7 test files
all pass `0` (counted by parsing, not by a one-line grep), so none is evidence
about the default in either direction.

### Row 8 — the mk separator
Whitespace only (§6c). Before: `hyphen`, `comma`, `-`, `,` each **exit 0**.
After: each **exit 64**, with a message naming `space` (§6h — the remedy must be
executable). The corpus is untouched, asserted rather than assumed:
`sha256sum -c design/display-grouping-vectors.tsv.sha256` passes before and
after, and the file keeps its 7 hyphen/comma rows. The GUI drift gate cannot see
this, confirmed by running: `mk gui-schema` reports `"choices": null` for
`--separator`.

### Row 9 — the mk blank line
On a two-record key file, stdout was 6 lines with 1 blank; now 5 lines, 0 blank,
every line `mk1`. Single-card stdout byte-identical before and after.

**This is a §6a violation, not a packability defect, and the two were separated
by measurement:** `me sysw pack` *accepts* the blank line — fed the ungrouped
two-record output with the blank line reinstated it exits 0 with a 498-byte
payload, byte-for-byte the same payload as without it.

The boundary **moved to the card** rather than being dropped, because F-311 is
real: a key file with the same BIP-380 record twice is accepted at exit 0 and
mints two byte-identical cards under one chunk-set id, so the boundary is not
recoverable from the headers.

`mk encode --keys | me sysw pack --out` now exits **0**, 498-byte payload —
P3 closure condition 3 for `mk`, run rather than described.

### Row 10 — the mk channels
`--in` on **all six** callers of `read_mk1_strings` (`decode`, `inspect`,
`verify`, `repair`, `address`, `derive`), plus `encode` where it is the
key-record file. `--out FILE` on `encode`, created 0600, overwriting.

Swept from the binary's own `gui-schema`: `--in` 0 → 7 subcommands; `--out`
2 → 3. The pre-existing two are `vectors` and `gen-man`, both meaning a
**directory** — the collision the plan names, not unified, and the new flag's
help says so.

### Row 11 — the mk exit code
See §2. This is the funds-sensitive row.

### Row 12 — the md1 set flag
`--from-md1-set FILE`, repeatable, byte-equal to the repeated `--from-md1` on a
real four-chunk keyed policy. Skips every non-md1 line and strips display
separators, so a file with today's `chunk-set-id:` header, comments, or grouped
output all bind identically. `mk encode --help | grep -c from-md1-set`: 0 → 2.

### Row 20 — the decline, asserted (mk half)
Four tests, each **mutated to prove it can fail** — a regression gate that
cannot fail is not a gate:

| decline | mutation | result |
| --- | --- | --- |
| no terminal / world-readable write gate (§6e) | unconditional refusal in `encode::run` | RED |
| no argv refusal (§4) | same | RED |
| `mk decode`'s 5 labelled fields (§6a, out of scope by name) | rename the `chunks:` label | RED |
| corpus unchanged at `7147b0ec…` | one appended newline | RED |

The corpus was verified byte-identical across all four repos that carry it.

---

## 2. THE MUTATION NUMBERS — REPRODUCED EXACTLY

Run on a **throwaway copy of the pristine tree at `c5739fc`**, so the plan's
figures are reproduced rather than adapted to a grown suite.

| | result |
| --- | --- |
| naive `=> 1` edit ALONE | **337 tests run: 335 passed, 2 failed** |
| the two failures | `repair_beyond_t4_capacity_exits_2`, `repair_hrp_mismatch_exits_2` — and nothing else |
| the funds-safety four, under that mutation | **4 × PASS** |
| naive edit **+** the `md`-shaped bypass | **337 tests run: 337 passed, 0 failed** |

Both match the plan. Re-run against this branch, where the suite has grown:
naive edit alone → **359 run, 356 passed, 3 failed** — the same two, plus this
row's own discriminating test.

The four-cell table, exit codes captured to files (never read through a pipe):

| invocation | before | after |
| --- | --- | --- |
| `repair <HRP-swapped>` | 2 | **2** |
| `repair <uncorrectable>` | 2 | **2** |
| `decode <HRP-swapped>` | 2 | **1** |
| `decode <uncorrectable>` | 2 | **1** |
| `verify <uncorrectable>` | 2 | **1** |
| `inspect <uncorrectable>` | 2 | **1** |
| `encode --from-md1 <bad>` | 2 | **1** |

That is `md`'s column, cell for cell. `SetReassemblyMismatch` stays **2** and is
pinned both by the four existing tests and by a new exit-code **table test** in
`src/error.rs` that names it as the funds-safety row.

### A defect I introduced and caught before committing
`md repair`'s bypass is a bare `eprintln!` + `return Ok(2)`. Transplanted
verbatim it **deletes `mk repair --json`'s error envelope** — measured: before
the change that command emitted
`{"error":{"details":null,"exit_code":2,"kind":"BchUncorrectable",…},"schema_version":1}`
on stdout; with the naive transplant, stdout was empty. §6b says `--json` is
unchanged this cycle, and no exit-code assertion would have noticed. The bypass
now rebuilds the envelope with the code it actually returns, and the result
diffs **byte-identical** to the pre-change output. Pinned by a test.

### And the sibling check that found more
My first draft of that test asserted *"md repair has no `--json`"*. **False** —
it has one, and it drops its envelope on exactly this path. So copying `md`
verbatim would have made `mk` match `md` by *losing* behaviour. Corrected, and
filed as **F-342** rather than fixed (another branch's repo, pre-existing).

---

## 3. THREE FALSE PASSES CAUGHT WHILE WRITING GATES

Each would have reported success against work not done.

1. **Two `--in` tests passed in the RED run.** clap's own
   `unexpected argument '--in' found` contains `--in` and exits 64, so "the
   error names `--in`" held against a binary with no `--in` at all. Both now
   assert `!contains("unexpected argument")`, and the empty-input one asserts
   the tool's own refusal text.
2. **`!body.contains('x')` as a truncation check fails against a CORRECT
   implementation** — `x` is in the codex32 alphabet and appears inside real
   `mk1` strings. Replaced with byte-equality against what stdout carries.
3. **The reading-verb equality compared two error messages.** `derive` requires
   `--path`/`--index`, and `address` *refuses* a BIP-48 multisig-cosigner origin
   outright — so two rows were two refusals agreeing. Both now assert the
   positional control exits 0 first, and `address` gets a single-sig card.

---

## 4. FINAL GATE SWEEP — every job in this repo's workflows

| job | command | rc |
| --- | --- | --- |
| ci.yml build | `cargo build --workspace` | 0 |
| | `cargo test --workspace` | 0 |
| | `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| ci.yml fmt | `sha256sum -c design/display-grouping-vectors.tsv.sha256` | 0 |
| | `cargo +1.95.0 fmt --check --all` (CI's pinned toolchain) | 0 |
| ci.yml vectors-roundtrip | `cargo build -p mk-cli --release` | 0 |
| | `mk vectors --out` + `jq` diff vs the pinned corpus | 0 |
| ci.yml freebsd-compile-gate | `cargo check --target x86_64-unknown-freebsd -p mk-cli` (whole-crate) | 0 |
| vendor-freshness.yml | `bash ci/repro/vendor-freshness.sh` | 0 |
| fuzz-smoke.yml | **not triggered** — its push paths are `fuzz/**`, `crates/mk-codec/src/**` and its own file; this branch touches none |
| ci.yml musl-check | **NOT RUNNABLE HERE** — `musl-gcc` is absent and the aarch64 leg needs `cross` + QEMU |

Also: `cargo build --locked --all-targets` 0, `cargo nextest run --locked`
**370 passed / 0 skipped**, `cargo fmt --check --all` 0 under 1.85.

**The musl legs are reported, not claimed green.** The diff adds no C
dependency and no new `cfg`; the freebsd whole-crate check is the closest proxy
that did run. Note also that `cargo build --locked` alone does **not** compile
`#[cfg(test)]` items — a `cargo build` that passed while the test cfg had two
type errors was caught by nextest, so `--all-targets` is used above.

---

## 5. THE PIN — A DEVIATION, THEN A REVERSAL

**Row 1 as written ("three files, no other edit") is false for `mnemonic-key`,
and this is the branch's headline finding.**

Measured:

* `ci/repro/vendor-freshness.sh` **fails closed** on any git source in
  `Cargo.lock` — exit 1 with its own error naming the two-block config.
  Fixable in-repo: `cargo vendor vendor/` adds **exactly one** directory
  (`vendor/mnemonic-io-lib/`, 10 files, 80K) with **zero churn** to the 133
  existing crates, and the three-block offline resolve exits 0.
* The half that is **not** fixable in-repo: `mk`'s reproducible-musl build is a
  reusable workflow homed in a fourth repo, which can only ever redirect
  `rust-miniscript`. Filed as **F-341**.

### What I did, and why it changed
I first **declined the pin**, implementing `--out` with a byte-for-byte local
copy of `write_private` (the `output_advisory.rs` precedent), and reported it.
That decision was taken before I knew what the siblings had done.

On the coordinator's cross-branch message I learned both had **pinned** and
filed the same breakage — **F-324** (`ms`) and **F-333** (`md`). That changed
the calculus: a third branch declining alone leaves the boundary table
half-honoured and hands reconciliation to the join, while the release recipe is
broken in the other two anyway and one shared-workflow change closes all three.
So I **reversed and aligned**, in its own commit `ac61e44`, with the local copy
deleted and `mnemonic_io_lib::write::write_private` called instead.

### One thing the siblings may want: `version` alongside `git`/`rev`
`mk-cli` **is published** (17 versions on crates.io) and so is `md-cli` (19).
Machine-checked with `cargo package -p mk-cli --allow-dirty --no-verify`:

| manifest | rc | cause |
| --- | --- | --- |
| no pin (control) | 101 | pre-existing: `mk-codec ^0.5.0` not yet on crates.io |
| bare `git`/`rev` | 101 | **"all dependencies must have a version specified when packaging"** — a permanent blocker |
| `git`/`rev` **+ `version = "0.1.0"`** | 101 | the same pre-existing `mk-codec` cause, byte-identical to the control |

So `version` restores packaging parity for one key. Residue:
`mnemonic-io-lib 0.1.0` must reach crates.io before the next `mk-cli-v*`
publish. **Worth checking whether `md-cli`'s pin carries it.**

### The vendor gate is real, not vacuously green

| check | rc |
| --- | --- |
| `bash ci/repro/vendor-freshness.sh` | 0 |
| the same under an **empty `CARGO_HOME`** | 0 |
| NEGATIVE CONTROL — hide `vendor/mnemonic-io-lib` | 1 |
| NEGATIVE CONTROL — a second unknown git source in the lock | 1 |

The empty-`CARGO_HOME` run is what makes the 0 mean something: with no registry
cache to fall back on, resolution came from committed `vendor/`.

**The fail-closed check earned its keep immediately.** My first rev-derivation
regex anchored on a closing quote after the 40-hex rev, but `Cargo.lock` writes
`git+<url>?rev=<sha>#<sha>` — the `[source."…"]` key cargo matches is the same
string *without* that fragment. The pattern matched nothing, and because the
script fails closed when a git source exists but no block was derived, that
surfaced as a RED with a message instead of a config silently missing its git
block. A comment in the script now records it.

### F-341's shape is worse for `mk` than for `md`
Measured under an empty `CARGO_HOME` against the new lock:

| block list | rc |
| --- | --- |
| two-block — what `musl-binaries.yml` passes today | **101** on `mnemonic-io-lib` |
| three-block with a **miniscript** stanza — the only other list that workflow can emit | **101** |
| three-block with the **mnemonic-engrave** stanza | **0** |

`mk` has **no miniscript git source at all**, so unlike `md` there is no value of
any existing workflow input that helps. **`mk` also did not have `md`'s
pre-existing breakage** — before this pin, the two-block form resolved at rc 0.
Reported and deliberately **not fixed**, matching the siblings.

---

## 6. WHAT I DID NOT DO

* **No join touched** — the toolkit release, the GUI pin bump and mirror, and
  the journey goldens are untouched.
* **No other repo written to.** `descriptor-mnemonic`, `mnemonic-toolkit`,
  `mnemonic-secret`, `mnemonic-gui`, the live `mnemonic-key` checkout and the
  other `_work/` worktrees were read only.
* **The shared reusable workflow in `mnemonic-toolkit`** — F-341, reported.
* **The musl CI legs** — not runnable in this environment; reported, not claimed.
* **`--out` on the reading verbs.** Scoped to `encode`: §6b defines `--out` as
  writing *the artifact*, and §6a scopes the artifact rule to `encode`. A
  `decode` report written 0600 would be a different feature. Widening it is a
  ruling this phase does not make.

---

## 7. THE mk HALF OF §10's ACCEPTANCE, RUN

Not described — executed, with `me` at `mnemonic-engrave` HEAD:

```
mk encode --in cosigner1.keys --from-md1-set wallet.md1        exit 0
  stdout: 2 ungrouped mk1 lines
  stderr: the engraving card + "policy 38bd7cec has 2 cosigner(s); 1 carded"
```

The stub `38bd7cec` matches `EXPECTED_KEYED_POLICY_A_STUB` in
`template_id_stub.rs` — an independent cross-language golden that Rust and the
Go port were each asked for separately.

Piped into `me sysw pack --expect descriptor,cosigner --out payload.bin`, with
no `grep` and no `--group-size`:

| input | rc |
| --- | --- |
| with today's `md` header on `wallet.md1` | **4** on record 0 — the header line |
| with the header removed (i.e. after the **`md` branch's** row 2 lands) | **0**, 589-byte payload |

**So the `mk` half is complete and the only thing between today and a green §10
acceptance is `md`'s header row** — exactly the dependency the plan predicts,
and the reason `--from-md1-set` was built tolerant of both eras.

---

## 8. FOLLOW-UPS FILED

In `mnemonic-engrave/design/FOLLOWUPS.md`, appended in this session:

* **F-341** — `mk`'s tag-time reproducible musl build cannot be fixed by any
  input its shared workflow accepts. Owning phase: **before the next
  `mk-cli-v*` tag**, non-deferrable past it. Third of the F-324 / F-333 family.
* **F-342** — `md repair --json` drops its error envelope on any codec failure
  while `mk repair --json` keeps one. Owning phase: whichever cycle owns
  `--json` uniformity.
* **F-343** — `mk encode` binds stubs in flag order, not argv order, and stub
  order is on the wire. Owning phase: ownerless residue; already documented and
  pinned by a test.

No new secret-handling leak was found, so the 2026-08-27 severity ruling did not
come into play.

---

## 9. DIFFSTAT

`git diff --stat c5739fc..HEAD` — **33 files, 3251 insertions, 172 deletions**,
of which `vendor/mnemonic-io-lib/` is 11 files. Excluding `vendor/`:
**22 files, 2064 insertions, 172 deletions**.

Source: `cmd/{address,decode,derive,encode,inspect,mod,repair,verify}.rs`,
`error.rs`, `format.rs`, `keyfile.rs`, `main.rs`, `Cargo.toml`, `Cargo.lock`,
`ci/repro/vendor-freshness.sh`, `CHANGELOG.md`.
Tests: `channels.rs`, `exit_code_invalid_artifact.rs`, `from_md1_set.rs`,
`p3_declines.rs` (new); `encode_grouping_flags.rs`, `keys_batch.rs` (changed).
`crates/mk-codec` is **untouched** — no normative codec behaviour changed, so
the Rust-primary rule is not engaged.

---

## 10. ONE CONSULT

A `fable` agent settled the pin-versus-local-copy question when I had the
in-repo facts but not the siblings'. It ruled **C** (local copy) on the facts
available, with an explicit *"what would change it"* that the siblings' choice
then satisfied. Recorded because its conclusion was **superseded**, not
followed: the reversal in §5 is mine, on information that arrived afterwards.
