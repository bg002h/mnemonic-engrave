# R2 (closing re-review of the fold) — `IMPLEMENTATION_PLAN_f449_stage1a_internal_key.md`

**1 Critical / 1 Important / 5 Minor / 3 Nit — NOT ready for implementation.**

Scope: did the r2 fold address r1's findings, and are the four new things in it
sound. No fresh audit; stage 1b not reviewed; the spec not re-reviewed.

Plan reviewed at `1712cc0e` (618 lines). Everything below was reproduced in
`/scratch/code/shibboleth/dm-worktrees/f449-stage1` @ `6cbd49d8` under the
PINNED 1.85.0 toolchain. All probe files were deleted and `fuzz/Cargo.lock`
restored; `git status --short` is empty.

**The fold is good work.** Eighteen of the twenty-two items below are cleanly
closed, four of them verified by execution rather than by reading. The one
Critical is in the *new* M-4 test, which is the only part of this fold nobody
had run — and it is unrunnable for two independent, measured reasons.

---

## Q1 — did the fold address each finding?

### Critical

| # | verdict | detail |
| --- | --- | --- |
| **C-1** (`git add` stages deleted artifacts) | **ADDRESSED** | Task 0 Step 4 is now `git add crates/md-codec/tests/golden crates/md-codec/tests/common crates/md-codec/examples` (l. 262). All three paths are created by Task 0 Step 1–3 or already exist (`tests/common/` exists, holding `mod.rs`). Nothing under `tests/vectors/`; `scripts/vendor-liana-evidence.sh` is gone from the plan entirely. |
| **C-2** (the retracted "wire version 2" premise left in Task 1) | **ADDRESSED** | Task 1 Step 1 now calls `decode_vendored(&chunks)` (l. 315) with the reason inline (ll. 312–314), and Step 1a's doc comment retracts the premise verbatim: *"Those 13 are SINGLE-PAYLOAD vectors (`force_chunked: false`), not a different wire version -- nothing is skipped"* (ll. 363–365). `reassemble` no longer appears in the gate. The `use md_codec::chunk::reassemble;` import was dropped with it. |

### Important

| # | verdict | detail |
| --- | --- | --- |
| **I-1** (golden specified three ways) | **ADDRESSED** | The reader is now `struct Golden { count: usize, tr_count: usize, vectors: Vec<(String, String)> }` (l. 300) with `assert_eq!(g.count, 65)`, `assert_eq!(g.tr_count, 23)` and `assert_eq!(g.vectors.len(), g.count)` (ll. 307–309) — it matches the object Task 0 Step 2 emits. Task 1 Step 2's inert `print(len(d),'vectors')` line is deleted. |
| **I-2** (reuse `keyed_phrase_files()`) | **STAYS ADDRESSED** | ll. 154–155 forbid it by name with the measured 46. `grep -n keyed_phrase_files` on the plan returns that one line only — no contradicting instruction survives anywhere. |
| **I-3** (`dump_ids` / helper homes) | **ADDRESSED** | Both examples are created by Task 0 Step 1, both helpers now have one stated home (`tests/common/vendored.rs`) and a stated mechanism (`include!`). Verified to compile and run in all three roots — see Q2(a). |
| **I-4** (Step 8 acceptance count) | **STAYS ADDRESSED** | ll. 546–549 unchanged: "1400 + the new tests … Assert the shape, not a stale number: 3 skipped, 0 failed". |
| **I-5** (2 of 6 gate steps) | **ADDRESSED** | Step 8 now calls `./scripts/phase-gate.sh` (l. 562) instead of a hand-rolled list, and states why. Measured: the script runs six steps and **does** set `RUSTDOCFLAGS="-D warnings"` (`scripts/phase-gate.sh:39`). See **M-B** for one overstatement in the surrounding prose. |
| **I-6** (rule for `LianaUnspendable` at the inverse sites) | **STAYS ADDRESSED** | The four-site ruling table and its rationale are unchanged (ll. 509–534). |
| **I-7** (blast radius omits `crates/*/tests`) | **STAYS ADDRESSED** | The third table row (38) and the `tr_node`/29-call-site note are unchanged; a fourth row for `fuzz/` was added (l. 106). |
| **I-8** (Step 9 guard passes while stale) | **ADDRESSED** | The HEAD-moving path r1 reproduced is gone: Task 1 Step 2 no longer says *"Generate and commit it as its own commit"* — it now states the golden was generated and committed in **Task 0 Step 4**, so Step 9's `git diff --quiet HEAD` (l. 588) compares against a commit the refactor cannot have moved. Residue is non-blocking — see **M-A**. |

### Minor / Nit

| # | verdict | detail |
| --- | --- | --- |
| **M-1** `fuzz/tests/gen_corpus.rs:182` | **ADDRESSED** | Now in the File Structure (l. 93), the blast-radius table (l. 106) and Step 8 (l. 576). Both claims verified: `fuzz/Cargo.toml` carries its own `[workspace]`; `gen_corpus.rs:182` is the `is_nums: false` line of a `Body::Tr` literal; and `( cd fuzz && cargo check --all-targets )` runs clean here, checking `md-codec-fuzz v0.0.0` against the **path** dependency on the local `md-codec`, so it does see the refactor. |
| **M-2** the existing whole-corpus gate | **ADDRESSED** | Heading is now "the task's PRIMARY gate" with the parenthetical naming `vector_corpus.rs` and `phase-gate.sh` (ll. 279–283). |
| **M-3** the `&mut` rewrite | **ADDRESSED** | The fourth shape is given explicitly for `canonicalize.rs:115` (ll. 490–498). |
| **M-4** golden covers `key_index == 0` only | **ADDRESSED IN FORM, BROKEN IN FACT** | A test was added. It cannot pass. See **C-A** below and Q2(b). |
| **M-5** stale `Tr` variant doc | **ADDRESSED** | Step 4 now carries the outer doc comment and says why (ll. 424–439). Three doc comments elsewhere still name `is_nums` and go stale — `nums.rs:6`, `encode.rs:34`, `decode.rs:85` — none of which breaks a build; recorded under Q2(c). |
| **M-6** dangling task references | **ADDRESSED** | `grep -n 'Task [0-9]'` on the plan returns only Task 0 / Task 1 plus the retrospective *"r0 scheduled this inside Task 8, reached after Task 2 changes identity.rs"* at l. 243, which is a statement about r0's numbering. "Task 3" and "Task 6" are gone. |
| **M-7** "the assertion block below" | **STAYS ADDRESSED** | |
| **M-8** document still presents itself as 1a+1b | **PARTIAL** | Title, Goal, Architecture and the File Structure table are rewritten for 1a (though the table's new "not touched" list is wrong — Q2(c)). **Global Constraints is unchanged**: bullets 2–5 are pure stage-1b material. See **M-C**. |
| **N-1** `Copy`/`Hash` on `InternalKey` | **ADDRESSED** | ll. 410–411: *"Copy is deliberate: the type is one byte plus a discriminant and is passed by value at ~98 sites; Hash mirrors what Body already derives."* |
| **N-2** Status line garbled | **ADDRESSED** | ll. 11–13 now read cleanly, one report path, one "Awaiting re-review", both round counts correct against the two reports on disk. |
| **N-3** `split` signature | **ADDRESSED** | l. 72 now `chunk.rs:240 split(&Descriptor) -> Result<Vec<String>, Error>`. |
| **NEW-1** Task 0 Step 4 `git add` | **ADDRESSED** | Same as C-1. |
| **NEW-2** helpers with no home | **ADDRESSED** | And verified by execution, not reasoning — Q2(a). |
| **NEW-3** `cargo doc` cannot fail | **ADDRESSED** | `phase-gate.sh:39` sets `RUSTDOCFLAGS="-D warnings"`; the plan now says so and cites `ci.yml:83`. |
| **NEW-4** Step 9 `git add` dropped `examples` | **OPEN, measured harmless** | See **N-C**. |

---

## The blocking findings

### C-A (Critical, new in this fold). Step 1b's test cannot pass on unmodified code — two independent reasons, both reproduced.

`a_non_zero_slot_index_survives_the_round_trip` (ll. 340–350) is the whole of
M-4's remedy and the plan's only claimed coverage for a non-zero internal-key
index. Task 1 Step 3 says of the gate file: *"Run it to confirm it passes on
unmodified code. Expected: **PASS**."* It cannot.

**Reason 1 — the prescribed fixture does not encode.** The doc comment at
ll. 333–338 specifies `tr(@idx, {pk(@0), pk(@1), pk(@2), pk(@3)})` with
`n >= 8`. `validate_placeholder_usage` (`validate.rs:11-36`) enforces *"Every
`@i` for `0 ≤ i < n` appears at least once in the tree"*, and it runs inside
`encode_payload` at `encode.rs:147`. With n = 8 and four leaves, `@4`, `@5`,
`@6` are unreferenced. Built exactly as specified and run:

```
PLAN-LITERAL idx=1: encode -> Err(PlaceholderNotReferenced { idx: 4, n: 8 })
PLAN-LITERAL idx=2: encode -> Err(PlaceholderNotReferenced { idx: 4, n: 8 })
PLAN-LITERAL idx=7: encode -> Err(PlaceholderNotReferenced { idx: 4, n: 8 })
```

`encode_payload(&d).expect("encode")` panics for every `idx` in the loop.

**Reason 2 — the round trip cannot be equal even with all 8 referenced, because
`encode_payload` CANONICALISES first.** `encode_payload_inner` calls
`canonicalize::canonicalize_placeholder_indices` at **`encode.rs:145`**, one
line *before* the validator at `:147`. `walk_collect_first`
(`canonicalize.rs:55-72`) registers a `Body::Tr`'s `key_index` as a
first-occurrence **before** descending into the tree, and `Tr` is always the
root tag, so the internal key is always the *first* placeholder observed and
`perm[key_index] = 0`. Rebuilt with all eight placeholders referenced
(`tr(@idx, {pk(@0)…pk(@7)})`, n = 8, real BIP-86 origin path) and run:

```
ALL8 idx=1: round_trip_eq=false  decoded_internal_key(is_nums=false key_index=0)
ALL8 idx=2: round_trip_eq=false  decoded_internal_key(is_nums=false key_index=0)
ALL8 idx=7: round_trip_eq=false  decoded_internal_key(is_nums=false key_index=0)
```

`assert_eq!(back, d)` fails for every `idx`, on unmodified code, before any
refactor exists.

**The premise underneath it is also misdiagnosed, and that matters more than
the test.** r1 measured *"0 of the 13 `Slot` vectors carry a non-zero index"*
and the plan reads that as a corpus **gap** ("Widening the corpus from 46 to 65
did not widen this"). It is not a gap. A non-zero internal-key slot is
**canonically unreachable** through `encode_payload`: canonicalisation renumbers
the `Tr` internal key to `0` in every descriptor, so no vector this corpus could
ever contain would carry one. The existing unit test the plan dismisses —
`tree.rs:559`, `tr_is_nums_false_round_trip`, `key_index: 2` at kiw 2 — works
precisely because it calls `write_node`/`read_node` **directly**, bypassing
canonicalisation. That is the only layer at which a non-zero `Slot` exists, and
it is exactly the layer Steps 5 and 6 rewrite.

So the finding is not "fix the fixture". It is that the plan asserts a coverage
gap that does not exist, and closes it with a gate that has never been run. Per
the repo's own rule, *a gate that has never executed is a hypothesis, not a
gate* — and this is the one piece of the fold that had no prior round behind it.

Also worth stating plainly: `tr_descriptor_with_slot`'s body is
`/* build with 8 keys */`. The Self-Review's Placeholder scan (§2, ll. 609–612)
asserts *"One helper is used and not defined by any step: `load_vendored_phrase`"*
— which is now false; this is a second one, and it is the one that does not work.

### I-A (Important, new in this fold). The File Structure table's "not touched by stage 1a" list is wrong on 3 of its 10 files, and contradicts this plan's own Step 7 by name.

ll. 95–97: *"**Not touched by stage 1a**, though the combined plan's table listed
them: `header.rs`, `nums.rs`, `encode.rs`, `decode.rs`, `chunk.rs`,
`identity.rs`, `render.rs`, `validate.rs`, `parse/template.rs`, `decompose/`.
All are stage 1b."* Measured against the worktree:

| file | measured | forced to change in 1a? |
| --- | --- | --- |
| `md-codec/src/render.rs` | `is_nums`/`key_index` destructured at `:187-188`, consumed at `:194`/`:197`; first `#[cfg(test)]` is `:745`, so this is production | **YES** — and **Step 7's ruling table names `md-codec/src/render.rs:194` explicitly** (l. 520) |
| `md-codec/src/validate.rs` | `Body::Tr { is_nums, key_index, tree }` destructured at `:83-87`, `if !*is_nums` at `:94`, index used at `:95-100`; first `#[cfg(test)]` is `:585` | **YES** — Step 7's third pattern applies |
| `md-cli/src/parse/template.rs` | `is_nums: true` at `:1605`, `is_nums: false` at `:1630`; first `#[cfg(test)]` after those is `:1717`, so both are production | **YES** — and **Step 7 names them**: *"`md-cli` breaks at four production sites and they ship in THIS commit (`format/json.rs:348`, `parse/reuse.rs:515`, `parse/template.rs:1604`, `:1629`)"* (l. 536) |
| `header.rs`, `chunk.rs`, `decompose/` | 0 occurrences | no — correct |
| `encode.rs`, `decode.rs`, `identity.rs` | only `key_index_width` and doc-comment prose (`encode.rs:34`, `decode.rs:85`) | no compile change — correct, but the doc comments go stale (M-5 class) |
| `nums.rs` | `:6` doc comment `Body::Tr { is_nums: true, .. }` | no compile change; stale doc |

None of this can produce a wrong *result* — all three files fail to compile
unchanged, so the compiler forces the edit. What it can produce is the wrong
*resolution*: an implementer who trusts "render.rs — not touched by stage 1a,
all are stage 1b" and then meets Step 7's rule for `render.rs:194` has two
authoritative-looking answers, and the cheaper one is to make the compiler happy
and defer the ruling to 1b. The plan calls that ruling *"the plan's one
genuinely load-bearing choice"* (l. 512). A scope table that tells the
implementer the ruling's own site is out of scope is a real defect in the plan,
and it was introduced by this fold. One line to fix.

---

## Q2 — the four new things

### (a) `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/common/vendored.rs"))` — **sound; verified by execution in all three roots.**

I created `tests/common/vendored.rs` carrying the plan's two helpers with their
bodies verbatim, plus `examples/dump_encodings.rs`, `examples/dump_ids.rs` and
`tests/internal_key_refactor.rs`, each opening with the plan's `include!` line —
and the test file **also** doing `mod common;` and calling `common::tr_node`, to
force the collision the brief asks about. Results:

- `cargo clippy --locked -p md-codec --all-targets --all-features -- -D warnings`
  under the pinned 1.85.0 toolchain: **clean, exit 0.** No name clash, no double
  definition, no `dead_code` warning.
- `cargo run -p md-codec --example dump_encodings` printed a real payload
  (`2002001810`), so `CARGO_MANIFEST_DIR` resolves and `hex`/`serde_json`
  dev-dependencies are reachable from `examples/` as claimed.
- `cargo test -p md-codec --test internal_key_refactor`: **1 passed.**

**Why there is no collision, mechanically.** Cargo auto-discovers `tests/*.rs`
and `tests/*/main.rs` as test targets; `tests/common/vendored.rs` is neither, so
it is never compiled as a target of its own. `tests/common/mod.rs` does not
declare `mod vendored;`, so the file is reached only through `include!`. And
`mod common;` puts `tr_node` in the `common` namespace while the included
helpers land at the crate root — disjoint. The claim *"no `pub mod testsupport`
leaking into the shipped crate"* also holds: nothing in `src/` is touched.

Two things the plan should say, neither blocking — **N-A** and **N-B** below.

### (b) The M-4 test and the `n >= 8` precondition — **the kiw reasoning is right; the test is not.** See **C-A**.

The width claim checks out exactly. `decode.rs:88` is
`let key_index_width = (32 - (path_decl.n as u32).saturating_sub(1).leading_zeros()) as u8;`
— the plan's citation is line-exact — and `Descriptor::key_index_width`
(`encode.rs:37-41`) computes the identical expression. So kiw = bit_length(n−1):
n = 8 gives kiw = **3** and `Slot(7)` is representable; n = 4 gives kiw = **2**
and `Slot(7)` would be truncated by the width. The doc comment's reasoning at
ll. 334–338 is correct, and the `n >= 8` precondition is the right one.

**Would the test fail if the refactor zeroed the index?** In principle yes, and
for the right reason: `d` is built directly rather than round-tripped, so
zeroing on the write path, the read path, or both still leaves `back != d`. The
test is well-shaped as a mutation detector. It simply cannot be reached — it
fails identically on unmodified code, for the two reasons in C-A.

`decode_payload(&bytes, bits)` and `encode_payload(&d)` are both real and both
match the test's use: `decode.rs:66 pub fn decode_payload(bytes: &[u8], total_bits: usize) -> Result<Descriptor, Error>`
and `encode.rs:99 pub fn encode_payload(d: &Descriptor) -> Result<(Vec<u8>, usize), Error>`.

### (c) The File Structure table rewrite — **the "not touched" list is wrong. See I-A.**

Three of the ten named files (`render.rs`, `validate.rs`, `md-cli/src/parse/template.rs`)
are touched by this plan's own Step 7, two of them named there explicitly. The
other seven are correct as compile-scope, with three carrying doc comments that
go stale.

### (d) The `phase-gate.sh` substitution — **substantially right; one overstatement.**

`scripts/phase-gate.sh` runs exactly six steps, measured from the file:

1. `cargo nextest run --locked --all-features`
2. `cargo test --workspace --doc --all-features`
3. `cargo clippy --locked --all-targets --all-features -- -D warnings`
4. `cargo fmt --check`
5. `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --all-features` (`:39`)
6. `( cd design && sha256sum -c display-grouping-vectors.tsv.sha256 )`

So the plan's two load-bearing claims are **true**: it is six, and
`RUSTDOCFLAGS="-D warnings"` is set — NEW-3 is genuinely closed. Step 1 has no
`--workspace`, but the repo root is a virtual manifest
(`[workspace] members = ["crates/md-codec", "crates/md-cli"]`), so it covers both
crates. The new examples are compiled by step 3's `--all-targets`, which is what
Step 8's `--all-targets` sentence was for.

`( cd fuzz && cargo check --all-targets )` is **right for that workspace** and
runs here: `fuzz/Cargo.toml` declares its own `[workspace]`, `cargo check` there
reported `Checking md-codec v0.45.1 (…/crates/md-codec)` — the local **path**
dependency, not a registry copy — and `Checking md-codec-fuzz v0.0.0`, then
`Finished`. `--all-targets` is what pulls in `tests/gen_corpus.rs`, whose
`:182` is the `is_nums: false` line inside `Body::Tr { is_nums: false, key_index: 0, tree: None }`.
Citation exact.

The overstatement is **M-B**.

---

## Minor / Nit

- **M-A (Minor).** Task 1 Step 2 is now dead but still executable. Its
  `cargo run --quiet --example dump_encodings > …/pre_refactor_encodings.json`
  (l. 385) survives under a heading that says to generate the golden, while the
  prose four lines down says it was already generated and committed in Task 0
  Step 4; only the python line got the `# (superseded …)` comment. The step also
  opens `cd /scratch/code/shibboleth/descriptor-mnemonic` (l. 383) — the **main
  checkout**, not the worktree the work is done in. r1's recommended fix was to
  delete the step; the fold defanged it instead. Measured mitigation for the
  worst reading: running `cargo nextest run --locked -p md-codec internal_key_refactor`
  in a tree without that test does **not** report a false PASS — nextest 0.9.140
  prints `Starting 0 tests across 38 binaries` / `error: no tests to run` and
  exits **2**. So the stray `cd` cannot green Step 3.
- **M-B (Minor).** *"`scripts/phase-gate.sh` exists and already carries every
  flag"* (ll. 556–557) overstates it. The superseded hand-rolled list ended with
  `cargo check --target x86_64-unknown-freebsd -p md-cli`, which r1 verified runs
  locally under the pinned toolchain; `phase-gate.sh` does **not** run it, and
  names freebsd/musl/windows/macos as CI-only blind spots in a header comment it
  never prints at runtime. Low risk for a refactor with no platform surface, but
  the plan swapped a gate for a smaller one while saying it was swapping up.
- **M-C (Minor, = M-8's residue).** Global Constraints bullets 2–5 (ll. 22–25)
  are pure stage-1b material in a stage-1a plan: *"The wire version is 8, never
  5"*, *"The encoder emits the minimum version that expresses the tree"*,
  *"Identity hashes take the version derived from the tree, never a constant"*
  (naming `identity.rs:90`/`:200`), *"Version bytes come from the render-time
  `--network` flag"*. The plan's own File Structure puts `header.rs` and
  `identity.rs` in 1b, and no task in this plan touches either. Bullet 4 in
  particular reads as an instruction to change `identity.rs` now.
- **M-D (Minor).** `tests/golden/pre_refactor_ids.json` is captured by Task 0
  Step 3 and read by **nothing** in this plan — `grep` finds no reader. Capturing
  it pre-1a is correct and is exactly what Step 3 argues for, but the plan never
  says it is *stage 1b's* input, so it reads like an orphan artifact. One
  sentence.
- **M-E (Minor).** Determinism is still unstated. Neither `dump_encodings` nor
  `dump_ids` is told to emit a stable order, while the stated model
  `examples/dump_skeleton_keys.rs` sorts its file list (`out.sort()`). With Step 2
  still regenerating the file (M-A), an unstable directory order makes Step 9's
  `git diff --quiet HEAD -- …/golden` fire spuriously. r1 raised this as a note
  under Q2(c) and it was not folded.
- **N-A (Nit).** `cargo fmt --check` rewrites the plan's `include!` line **as
  printed**. Reproduced: rustfmt wants
  `include!(concat!(\n    env!("CARGO_MANIFEST_DIR"),\n    "/tests/common/vendored.rs"\n));`
  over four lines. An implementer copying l. 168 verbatim into three files fails
  `phase-gate.sh` step 4.
- **N-B (Nit).** `vendored.rs` must carry **no inner attributes** — and the file
  it sits beside, `tests/common/mod.rs:3`, is `#![allow(dead_code, unused_imports)]`.
  Mirroring that idiom into `vendored.rs` is a hard error; reproduced on a
  minimal crate: *"error: an inner attribute is not permitted in this context"*.
  Related: rustfmt follows `mod` declarations, not `include!`, so
  `vendored.rs` itself is never seen by `cargo fmt --check`.
- **N-C (Nit, = NEW-4).** Step 9's `git add crates/md-codec/src crates/md-cli/src
  crates/md-codec/tests` (l. 593) omits `crates/md-cli/tests` and
  `crates/md-codec/examples`, while the blast-radius table pairs
  `md-codec/tests/ + md-cli/tests/` as one 38-site row. Measured **harmless**:
  `crates/md-cli/tests/` has **zero** compiling `Body::Tr` sites — the only hits
  are comments (`n1_admission_taxonomy.rs:726,729,732`, `smoke.rs:15`) and six
  JSON snapshots that Step 7 deliberately keeps byte-stable by preserving
  `is_nums`/`key_index` in the v1 schema. Task 1 touches no example. So the
  staging set is in fact sufficient; the narrowing was still not deliberate.

---

## Verdict

The fold closed every one of r1's blocking findings — C-1, C-2, I-1, I-3, I-5
and I-8 — and I could not break any of them. The three mechanisms it introduced
in response are each sound where it counts: the `include!` bridge compiles,
runs and passes under pinned clippy with `-D warnings` in all three crate roots
while coexisting with `mod common;`; `phase-gate.sh` really does carry
`RUSTDOCFLAGS`; and `( cd fuzz && cargo check --all-targets )` really does
compile `gen_corpus.rs` against the local path dependency.

What is left is the one thing in the fold that had no round behind it. The M-4
test is unrunnable twice over — the fixture fails `PlaceholderNotReferenced`,
and canonicalisation zeroes the internal key before encoding, so the round trip
cannot be equal on unmodified code — and behind it sits a misread of r1's own
measurement: a non-zero `Slot` index is canonically unreachable, not
under-covered. That makes Task 1 Step 3's "Expected: PASS" false and Step 8's
gate red by construction. Separately, the rewritten File Structure table tells
the implementer three files are out of scope that Step 7 requires them to
change, two of which Step 7 names by path.

Both are edits to single blocks. Neither touches the plan's architecture, which
is sound.

**ready for implementation: no**
