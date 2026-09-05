# Hashlock H1b — implementation report (`me` bumps to ms-codec 0.8)

**Implementer:** one opus agent, no sub-agents.
**Plan:** `design/IMPLEMENTATION_PLAN_hashlock_H1b_me_bump.md`, STATUS R0 GREEN at engrave `eece8a3`.
**Worktree:** `/scratch/code/shibboleth/me-worktrees/hashlock-h1b`, branch `hashlock-h1b`.
**Branch point:** `d723cac` (engrave `master` at dispatch).
**Branch tip:** the commit that adds this report; the code tip is `6f4edf8`.
**Nothing pushed. No commit on `master`. `/scratch/code/shibboleth/me-worktrees/h1b-gate` never touched.**

Staleness: `git diff 0f5ce23..d723cac -- crates/ Cargo.lock Cargo.toml` is **empty**, so
every code citation in the plan was current at the branch point. Every anchor the plan
named was found exactly as quoted; none had moved.

## Commits

| | SHA | Subject |
| --- | --- | --- |
| Tasks 1+2 | `51f25c95163ef7bef9a5daac71c2f8b293c0e416` | `me: ms-codec 0.8 -- a decoded Payload::Preimage is refused as a preimage plate on both host verbs; preimage_plate pin-independent (hashlock H1b, F-473)` |
| Task 3 | `2bb3f3b76b4f0724012ef340ffd8c18d4dd13113` | `me sysw pack: name an id/kind mismatch (TagKindMismatch) instead of 'outside the profile' (hashlock H1b, ruling L24)` |
| Task 4 | `6f4edf84a86f2c94ad510be366f5b5dfd0c7cab9` | `records: me 0.8.1, CHANGELOG H1b entries, F-473 closed / F-454 advanced / F-475 filed (hashlock H1b Task 4)` |
| report | this commit | the file you are reading |

Diff vs the branch point — **exactly the eight files the plan's File Structure names**, nothing else:

```
 Cargo.lock                                         |   8 +-
 crates/me-cli/CHANGELOG.md                         |  29 ++++-
 crates/me-cli/Cargo.toml                           |   4 +-
 crates/me-cli/src/main.rs                          |   9 +-
 crates/me-cli/src/seal/record.rs                   | 121 +++++++++++++++------
 crates/me-cli/src/sysw/mod.rs                      |  66 ++++++++++-
 .../me-cli/tests/preimage_plate_is_not_a_seed.rs   |  19 ++++
 design/FOLLOWUPS.md                                |  65 ++++++++++-
 8 files changed, 276 insertions(+), 45 deletions(-)
```

**`master` moved while this ran** (`d723cac` → `55ee7a4`, eight commits of H2-spec work and
the `d723cac` push report). `comm -12` of the two changed-file lists is **empty** — zero
overlap, so the merge is clean. No action needed; recorded because the controller's
`git diff master..HEAD` will otherwise show H2 files as deletions.

---

## Task 1 — the bump, and the RED

**Step 1 (bump and lock).** `crates/me-cli/Cargo.toml:53` `ms-codec = "0.7"` → `"0.8"`,
then `cargo update -p ms-codec` (never a bare `cargo update`; this repo has no `vendor/`):

```
    Updating crates.io index
     Locking 1 package to latest compatible version
    Updating ms-codec v0.7.0 -> v0.8.0
note: pass `--verbose` to see 35 unchanged dependencies behind latest
```

Lockfile now reads `name = "ms-codec"` / `version = "0.8.0"`. The plan's Expected said no
new crates enter (`pbkdf2`/`hmac`/`sha2` already present). **Machine-checked rather than
believed:** `git diff Cargo.lock | grep -E '^[+-]name = '` returns **nothing** — not one
package line added or removed — and the diff is 4 insertions / 2 deletions. Expected met.

**Step 2 (build).** `cargo build --locked -p mnemonic-engrave` → `Finished` clean. **No
exhaustive match in `me` broke**, so the plan's "the compiler's list is the deviation list"
produced an empty list at this step. (One did break later, in Task 3, and for a good
reason — see O-1.)

**Step 3 — the RED, and it was SEEN.** Run once, whole crate, `--no-fail-fast`
(a superset of the plan's `-E 'test(/preimage/)'` filter, so the six failures and the
totals come from a single capture rather than two runs of the same suite):

```
     Summary [   0.482s] 617 tests run: 608 passed, 9 failed, 2 skipped
```

9 = the 3 box-local `history_purge` + **the SIX F-473 predicts**, each failing by the
mechanism the plan names — ADMISSION, not the profile arm:

| test | failure |
| --- | --- |
| `preimage_plate_is_not_a_seed::a_preimage_plate_is_not_a_seed_record` | `validate_record admitted a 0x03 preimage plate as Ms` |
| `preimage_plate_is_not_a_seed::sysw_pack_names_a_preimage_plate_and_never_echoes_it` | panicked at `preimage_plate_is_not_a_seed.rs:61` — the `!out.status.success()` assert: the verb **accepted** the plate |
| `preimage_plate_is_not_a_seed::seal_names_a_preimage_plate_and_never_echoes_it` | panicked at `preimage_plate_is_not_a_seed.rs:104` — same assert, same reason |
| `sysw::tests::a_preimage_plate_is_named_not_misdiagnosed` | `left: Ok([77, 78, 69, 77, 83, 89, 83, 87, …])` / `right: Err(Unclassifiable(0, PreimagePlate))` — it PACKED |
| `codex32_seam::the_host_never_admits_what_the_device_would_refuse` | `assertion left == right failed: preimage-plate-0x03: host verdict` / `left: true` / `right: false` |
| `record_corpus::every_corpus_record_classifies_as_it_did_before_s2` | `left: "Codex32Secret"` / `right: "Unknown"` |

That is the plan's tests-lens I-4 count exactly (six beyond `history_purge`), and the
`sysw`/`seal` panics land on the *acceptance* asserts rather than the *wording* asserts —
i.e. both verbs really would have engraved a preimage as a seed. Nothing was committed at
this point; the red tree went into the Tasks 1+2 commit together with its fix.

## Task 2 — the refusal on the success path, and the predicate

Committed with Task 1 as `51f25c9`, as the plan requires.

**Step 1.** `validate_record`'s `Format::Ms` arm is now a `match` on the decoded payload:
`Ok((_, Payload::Preimage(_)))` → `RecordError::PreimagePlate`; `Ok((_, Entr | Mnem))` →
`RecordKind::Ms`; a **refusing** wildcard `Ok(_)` for the `#[non_exhaustive]` enum; an
`Err(TagKindMismatch)` arm; and the old `preimage_plate`-then-`Invalid` error path kept, so
the diagnosis survives either pin.

**Step 2.** `preimage_plate` replaced by `id_kind_mismatch` + a shape-keyed `preimage_plate`
(HRP gate, mismatch excluded, `PreimageLengthMismatch` clause, then unshared + 33-byte
payload + first byte `0x03`), plus `RecordError::TagKindMismatch` and its Display arm, plus
the three fidelity-M-5 prose corrections (the `bip93_outside_the_profile` doc line, the
`Bip93OutsideTheProfile` variant doc, and `main.rs`'s operator text — all three now say
`entr` *or* `hash`).

**Step 3.** The witness `the_codec_decodes_the_plate_and_me_still_refuses_it` appended. The
plan's aside held: a bin crate's `[dependencies]` are visible to its integration tests, so
`ms_codec` resolved with no `[dev-dependencies]` entry.

**Step 4 — GREEN.** `-E 'test(/preimage/) | test(/the_host_never_admits/) | test(/the_codec_decodes/)'`:

```
     Summary [   0.018s] 6 tests run: 6 passed, 614 skipped
```

6 PASS, the plan's measured number. The three shape assertions were then added to
`a_preimage_plate_is_named_not_misdiagnosed` and the stale mutation comment replaced;
`cargo fmt` wrapped the long literals; re-run: **6 tests run: 6 passed**.

**Step 5 — whole crate at the Task-2 tree:** `618 tests run: 615 passed, 3 failed, 2 skipped`
(618 = 617 + the witness; the 3 are `history_purge`). `cargo fmt --check` exit 0. Clippy's
only error is the pre-existing `manual_is_multiple_of`.

### Mutations re-run (Task 2)

**(a) delete the `Ok((_, Payload::Preimage(_)))` arm** — replaced with `Ok(_) => Ok(RecordKind::Ms)`:

```
     Summary [   0.065s] 6 tests run: 0 passed, 6 failed, 614 skipped
```

`6 failed, 0 passed` — the tests lens's exact measurement. Failing lines:

- `a_preimage_plate_is_not_a_seed_record` → `validate_record admitted a 0x03 preimage plate as Ms`
- `the_codec_decodes_the_plate_and_me_still_refuses_it` → panicked at `crates/me-cli/tests/preimage_plate_is_not_a_seed.rs:137:5`, which is `assert!(matches!(` — **the second assertion** (see D-1)
- `the_host_never_admits_what_the_device_would_refuse` → `preimage-plate-0x03: host verdict`, `left: true` / `right: false`
- `a_preimage_plate_is_named_not_misdiagnosed` → `left: Ok([77, 78, 69, …])` / `right: Err(Unclassifiable(0, PreimagePlate))`
- `sysw_pack_names_…` → `preimage_plate_is_not_a_seed.rs:61:5`; `seal_names_…` → `:104:5`

Reverted from a byte copy, then `touch`ed.

**(b) drop the shape clause** (`d.len() == 33 && d[0] == 0x03` → `false`):

```
     Summary [   0.009s] 6 tests run: 4 passed, 2 failed, 614 skipped
```

- `sysw_pack_names_a_preimage_plate_and_never_echoes_it` → panicked at
  `crates/me-cli/tests/preimage_plate_is_not_a_seed.rs:65:5`: **`stderr does not name the kind:`**
- `a_preimage_plate_is_named_not_misdiagnosed` → panicked at `crates/me-cli/src/sysw/mod.rs:863:9`:
  `left: Err(Unclassifiable(0, Unrecognised))` / `right: Err(Unclassifiable(0, PreimagePlate))`
- `seal_names_a_preimage_plate_and_never_echoes_it` **stayed green**, exactly as the plan
  predicts, because `validate_record`'s success-path arm already refuses it independently
  of the predicate. The `Unrecognised` (not `Bip93OutsideTheProfile`) left-value also
  confirms the plan's corrected mutation comment: at 0.8 the plate decodes, so
  `bip93_outside_the_profile` is false for it.

Reverted, `touch`ed.

## Task 3 — name an id/kind mismatch

Committed as `2bb3f3b`.

**RED, in the order the compiler forced it — three states, not two:**

1. the test alone →
   `error[E0599]: no variant, associated function, or constant named `TagKindMismatch` found for enum `sysw::UnknownReason` in the current scope` at `sysw/mod.rs:918:61`. This is the plan's stated compile-error RED.
2. the variant added, nothing else →
   `error[E0004]: non-exhaustive patterns: `&UnknownReason::TagKindMismatch` not covered`,
   failing `bin "me"`. See **O-1**.
3. variant + operator words, **without** the `unknown_reason` arm → the value FAIL the plan
   predicts, verbatim:

```
thread 'sysw::tests::an_id_kind_mismatch_is_named_not_misdiagnosed' panicked at crates/me-cli/src/sysw/mod.rs:920:9:
  left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))
 right: Err(Unclassifiable(0, TagKindMismatch))
```

**GREEN** after the `unknown_reason` arm, `-E 'test(/misdiagnosed/)'`:

```
     Summary [   0.007s] 2 tests run: 2 passed, 619 skipped
```

2 PASS — the plan's measured number, with the helper form.

### Mutations re-run (Task 3)

**(i) remove the `id_kind_mismatch` arm** — `2 tests run: 1 passed, 1 failed`:

```
thread 'sysw::tests::an_id_kind_mismatch_is_named_not_misdiagnosed' panicked at crates/me-cli/src/sysw/mod.rs:923:9:
  left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))
 right: Err(Unclassifiable(0, TagKindMismatch))
```

`a_preimage_plate_is_named_not_misdiagnosed` **stayed green** (its arm is separate), as the
plan says.

**(iii) the arm moved AFTER the profile arm** — `2 tests run: 1 passed, 1 failed`:

```
thread 'sysw::tests::an_id_kind_mismatch_is_named_not_misdiagnosed' panicked at crates/me-cli/src/sysw/mod.rs:926:9:
  left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))
 right: Err(Unclassifiable(0, TagKindMismatch))
```

Same test, same left value — so the test pins the **order**, not merely the presence of the
arm. Both reverted from a byte copy and `touch`ed.

Whole crate at the Task-3 tree: `619 tests run: 616 passed, 3 failed, 2 skipped`; fmt clean.
That is the plan's Step 5 Expected (`619 run, 616 passed, 3 failed`) to the test.

## Task 4 — records

- `crates/me-cli/Cargo.toml` → `version = "0.8.1"`. `Cargo.lock` follows; that one line is
  the **entire** lock diff (verified with `git diff Cargo.lock`). Note the refresh needed
  `--offline` rather than `--locked`, since `--locked` exists to forbid exactly this.
- `CHANGELOG.md`: the single `## [Unreleased]` section is KEPT (fidelity N-2 — this file
  dates a heading only at release time; `grep '^## \['` confirms one `[Unreleased]` and the
  next heading is `[0.8.0] - 2026-09-02`). Two H1b `### Added` items beside H0's and the
  `+`-signed `key:` entry. H0's sentence rewritten in the past tense, as fidelity M-2 asks.
- **F-473 CLOSED** by `51f25c9` — the same commit as the bump, which is what the entry
  itself demanded ("Both halves must land in the same commit as the bump"). Both halves and
  the RED/mutation evidence recorded in the entry.
- **F-454 ADVANCED, still OPEN.** The version bump is not a release; it closes when the
  `v0.8.1` tag exists and `release.yml` has published. The tag is explicitly not in scope.
- **F-475 FILED, owning phase H2** — the fidelity lens's M-6.

**M-6 was measured, not transcribed.** The lens claimed the seam row
`bip93-plain-33-byte-payload-0x03`'s `source` prose is wrong about the 0.8 error. Rather
than repeat the claim, a throwaway crate pinned to `ms-codec = "0.8"` called
`ms_codec::decode` on the four strings in play:

| string | id | ms-codec 0.8.0 returns |
| --- | --- | --- |
| `ms10testsqvrsu9…` (the M-6 row) | `test` | `Err` — *unknown tag "test"; not a member of RESERVED_TAG_TABLE* |
| `ms10entrsqv0qqq…` (`preimage-shape-entr-id`) | `entr` | `Err` — *tag "entr" does not name the kind the prefix byte 0x03 carries* |
| `ms10hashsqw46h2…` (`preimage-plate-0x03`, 75 ch) | `hash` | `Ok(Payload::Preimage)` |
| `ms10hashsqw46h2…ssrnvvaudn2k4d` (50 ch) | `hash` | `Err` — *preimage payload is 16 bytes after the prefix; a hashlock preimage is exactly 32 bytes* |

M-6 **confirmed**: the row's error is `UnknownTag`, and `TagKindMismatch` belongs to the
`entr`-id row one over. The row's `host_admits: false` **verdict is unaffected** — the shape
predicate, not the codec error, is what names it, which is precisely fidelity I-1's point
and is now asserted by that exact string inside
`sysw::tests::a_preimage_plate_is_named_not_misdiagnosed`. The same probe independently
confirms all four codec behaviours the plan's design depends on.

`testdata/codex32_seam_vectors.json` was **not** edited (out of scope per the brief;
editing it re-pins `SEAM_VECTORS_SHA256` in both repos and H2 vendors the corpus anyway).

---

## Deviations

**D-1 — the witness's MUTATION doc comment said something the measurement contradicts.**
The plan's Task 2 Step 3 test text reads `delete the arm) -> this fails on `decoded`, and`.
Measured under mutation (a): `decoded` still succeeds — the codec is unchanged by a
`validate_record` mutation — and the failure is at
`preimage_plate_is_not_a_seed.rs:137`, the **second** assertion. The plan's own Step 4 prose
says the same thing ("the witness FAILS on its second assertion"), so the comment
contradicts the plan's other half. Minimal correction applied to the comment only:

```
/// delete the arm) -> this fails on its SECOND assertion (`decoded` still
/// succeeds — the codec is unchanged; measured), and
```

Reason: a MUTATION comment is a claim a future reader will re-run. Left as written it would
send them looking for a failure at the wrong assertion.

**D-2 — Task 2's `git add` line omits two files Task 2's own steps edit.**
The plan stages four paths for the Tasks 1+2 commit, but Step 2 edits
`crates/me-cli/src/main.rs` (the `Bip93OutsideTheProfile` operator text) and Step 4 edits
`crates/me-cli/src/sysw/mod.rs` (three shape assertions, the mutation comment, the variant
doc). Staged both at Task 2. Reason: **the committed tree must be the tree that was gated.**
Step 5 says run the whole crate, then commit; with a four-path `git add` the committed tree
is not the tested one, and no green result would apply to it. Nothing of Task 3 existed in
either file yet, so this is clean — Task 3's commit carries only Task 3's edits, and
`git show --stat 2bb3f3b` confirms it.

No other deviation. Every anchor, Expected count and mutation outcome the plan stated was
reproduced as written.

## Observations (not deviations)

**O-1 — the variant alone breaks the build, which the plan did not predict and which is
good news.** Adding `UnknownReason::TagKindMismatch` without its operator words fails
`bin "me"` with `error[E0004]: non-exhaustive patterns`. `main.rs`'s `U::…` match is
exhaustive, so **the compiler itself forbids a new refusal reason that has no words for the
operator**. That is a structural guard against a silent `Unknown`-shaped refusal, and it is
worth knowing it exists before H2 adds more variants. The plan's stated intermediate
value-FAIL is reachable only once the operator words are in place, which is the order used.

**O-2 — date labels.** The plan's STATUS line is labelled 2026-09-05; git's clock for every
commit on this branch, and the newest dates in `FOLLOWUPS.md`, are **2026-09-04**. New
entries use 2026-09-04 and cite SHAs, per "document dates are session labels; git is the
clock".

---

## Final gate — verbatim, at code tip `6f4edf8` (working tree clean)

The report commit adds only this markdown file under `design/agent-reports/`, so it cannot
move any of these numbers.

`cargo nextest run --locked -p mnemonic-engrave --no-fail-fast`:

```
     Summary [   0.398s] 619 tests run: 616 passed, 3 failed, 2 skipped
        FAIL [   0.004s] (432/619) mnemonic-engrave::history_purge editing_the_file_alone_is_the_trap_the_message_warns_about
        FAIL [   0.006s] (441/619) mnemonic-engrave::history_purge the_emitted_zsh_recipe_actually_purges_the_entry
        FAIL [   0.005s] (443/619) mnemonic-engrave::history_purge the_harness_records_history_at_all
error: test run failed
```

**The only three failures are the box-local `history_purge` trio the brief names as
expected.** No test on this branch's surface fails.

`cargo clippy --locked -p mnemonic-engrave --all-targets -- -D warnings`:

```
error: manual implementation of `.is_multiple_of()`
   --> crates/me-cli/src/sysw/composer_records.rs:114:8
    |
114 |     if s.len() % 2 != 0
    |        ^^^^^^^^^^^^^^^^ help: replace with: `!s.len().is_multiple_of(2)`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#manual_is_multiple_of
    = note: `-D clippy::manual-is-multiple-of` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::manual_is_multiple_of)]`

error: could not compile `mnemonic-engrave` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `mnemonic-engrave` (lib test) due to 1 previous error
```

**Pre-existing and not this branch's** — the local nightly's lint, at
`sysw/composer_records.rs:114`, a file `git diff --name-only d723cac..HEAD` shows this
branch never touches. Not fixed, per the brief. Green in CI. It is the **only** clippy
error.

`cargo fmt -p mnemonic-engrave -- --check` — **exit 0, no output.**

`cargo build --locked -p mnemonic-engrave --bin me`:

```
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.08s
```

### Both host verbs, with the built binary

Plate `ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c`:

```
$ printf '%s\n' "$PLATE" | me sysw pack --out <dir>/h.bin        ; exit 4
me: record 0 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), not a seed record; this container cannot place one yet. A preimage backs a hashlock spend path, not a wallet — keep it with the policy it unlocks, and do not re-encode it as entropy.

$ printf '%s\n' "$PLATE" | me seal --seal-secret --out <dir>/h.uf2 ; exit 4
me: this record is a hashlock PREIMAGE plate (kind 0x03), not a seed record; this container cannot place one yet. A preimage backs a hashlock spend path, not a wallet — keep it with the policy it unlocks, and do not re-encode it as entropy.
```

Both name the kind; both exit 4. Two further pairs were run because they exercise the two
NEW operator messages that no other end-to-end path reaches:

- the 50-character malformed plate `ms10hashsqw46h2at4w46h2at4w46h2at4w4ssrnvvaudn2k4d`
  (the `PreimageLengthMismatch` clause) — **both verbs, exit 4**, same preimage-plate words,
  confirming the plan's "measured on the wired tree" claim.
- the id/kind mismatch `ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9`
  — **both verbs, exit 4**: `me sysw pack` says *"record 0 (records count from 0) is an ms1
  string whose 4-character id and kind byte disagree…"* and `me seal` says *"this ms1
  string's 4-character id and kind byte disagree…"* (ruling L24, fidelity M-3's "same words
  on both verbs").

**No output container was written in any of the six cases** (`ls` of the destination
directory shows no `.bin`/`.uf2`), so the refusal precedes the write.

*Caveat on how these were measured:* a first pass reported `exit=0` for four of them because
a command substitution in the same `echo` clobbered `$?`. That was noticed and every exit
code above was re-measured with the status captured immediately after each pipeline.

---

## Left undone, and why

1. **The `me` 0.8.1 RELEASE (tag, `release.yml` assemble + sign)** — explicitly not this
   plan's and explicitly not this brief's. F-454 stays OPEN and says so.
2. **The seam-corpus prose edit (M-6)** — out of scope by the brief; filed as F-475 with
   owning phase H2, with the measurement, because editing `testdata/` re-pins
   `SEAM_VECTORS_SHA256` in both repos.
3. **The post-implementation opus adversarial review** (plan Task 4, brief
   `design/agent-briefs/hashlock-H1b-post-impl-brief.md` — already drafted on `master` at
   `0e6e03b`) — the controller's, not the implementer's; no sub-agents were spawned.
4. **Nothing pushed**, no `master` commit, `ci/staging` untouched.
5. **`history_purge` ×3** — box-local, expected by the brief, untouched.
