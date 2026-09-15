# R0 round 4 — Hashkinds Phase 2 fold verification (mnemonic-secret)

**Scope: two questions only.** Did the fold close each round-3 finding, and did
the fold introduce a new defect? Not a fresh audit. The spec, phases 1/3/4, and
findings closed in rounds 1 and 2 were not re-reviewed. Round 3's independent
confirmation of the digest layer (22 corpus row/stem pairs against `python3
hashlib`, both mutations failing the KAT) was taken as settled and not re-derived.

**Artifacts.**

1. **BRANCH** `hashkinds-p2` @ `1a723eb` in `/scratch/code/shibboleth/ms-worktrees/hashkinds-p2`.
   Fold = `d831339..1a723eb`; round 3 reviewed `4d4fb5b`.
2. **PLAN** `design/IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md` in
   `mnemonic-engrave`. Fold = `280d8429..82c4a4da`.

**Toolchain.** `export PATH=/home/bcg/.cargo/bin:$PATH` → `cargo 1.85.0
(d73d2caf9 2024-12-31)` / `rustc 1.85.0`, matching `rust-toolchain.toml`. The
system 1.98.0 phantom-clippy trap was avoided.

---

## 0. The green claims — independently re-measured

| gate | claimed | I measured | verdict |
| --- | --- | --- | --- |
| test suites | 101 ok, 0 failed | `cargo test --workspace --locked` → exit 0, **101** `test result:` lines, **zero** non-`ok`, zero `FAILED` | **TRUE** |
| (cross-check) | — | `cargo nextest run --locked --all-targets` → **572 tests run, 572 passed, 0 failed, 11 skipped** (568 before the fold + the 4 new `hashlock_kind` tests) | consistent |
| `clippy -p ms-codec --all-targets --locked -- -D warnings` | 0 | exit 0, clean | **TRUE** |
| `clippy -p ms-cli --all-targets --locked -- -D warnings` | 0 | exit 0, clean | **TRUE** |
| `cargo +1.95.0 fmt --all -- --check` | clean | exit 0, no output | **TRUE** |
| `ci/repro/vendor-freshness.sh` | OK | `vendor-freshness: OK — vendor/ satisfies Cargo.lock.` | **TRUE** |
| corpus SHA in CHANGELOG | `0a911f78f3cdc867dcc44483b7f4c0c1ac87b6d9b30b79f52094e8979bc3d8ce` | `sha256sum` on the file → byte-identical | **TRUE** |
| version bump consistency | ms-codec 0.10.0 / ms-cli 0.19.0 | `crates/ms-codec/Cargo.toml:3` = `0.10.0`; `crates/ms-cli/Cargo.toml:3` = `0.19.0`; `ms-codec = { path = "../ms-codec", version = "=0.10.0" }`; `Cargo.lock` carries both | **TRUE** |
| `cargo metadata --locked --offline` | — | exit 0, resolves | **TRUE** |
| corpus rows | 11 derivation / 10 `qr_text` | 11 / 10, `Counter({'sha256': 7, 'hash256': 1, 'ripemd160': 1, 'hash160': 1})` | **TRUE** |

**Every green claim the fold makes reproduces.** No false gate number.

---

## 1. Findings that were closed — stated so a later round does not re-derive them

- **C-1 — CLOSED.** Ran the built binary for all four kinds. The card reads
  `for md compose:  --path ... hash256=98a2…`, `ripemd160=09e7…`,
  `hash160=b5b7…`, `sha256=3cf5…`. **Mutation-proven:** reverting `:437` to the
  literal `sha256={}` reds `the_md_compose_line_names_the_chosen_kind` with
  *"hash256: the card says … — an operator following it composes the wrong
  wallet"*. Restored byte-identical.
- **I-1 — CLOSED.** `crates/ms-cli/tests/hashlock_kind.rs` exists with four
  tests. **All four mutation-proven, one at a time, each reverted from a
  byte-identical backup:**

  | mutation | test that reds |
  | --- | --- |
  | `md compose` line back to literal `sha256=` | `the_md_compose_line_names_the_chosen_kind` |
  | four-digest fallback block deleted (879 bytes) | `without_a_kind_every_digest_is_listed_on_stderr` |
  | `parse_kind` folds case (`s.to_ascii_lowercase()`) | `an_uppercase_kind_is_refused` |
  | `record` always bare (`format!("hash:{}", …)`) | `the_record_follows_the_producer_rule_for_every_kind` |

  In each run the other three tests passed, so no test is vacuous and none is
  merely a duplicate of another.
- **I-2 — CLOSED, and the fold's mutation claim is TRUE.** I deleted all three
  non-sha256 `qr_text` rows (10 → 7) and ran
  `cargo nextest run -p ms-codec -E 'binary(hashlock_qr_text)'`:
  `qr_text_matches_every_corpus_row` FAILED at `hashlock_qr_text.rs:53` —
  *"the corpus carries no qr_text row for kind hash256"*. Under round 3's `>= 7`
  floor the same deletion was green. Corpus restored to `0a911f78…`.
- **I-3 — CLOSED.** Parsed the enumeration out of
  `gui_schema_emits_spec_v7_json.rs` programmatically: **14** flags listed
  (`--kind` now among them), the word is `fourteen`, the asserted total is `69`.
- **I-6 — CLOSED.** Plan line 169's `#[deprecated]` promise is replaced by
  *"No `digest` alias"* with the measured clippy cost. `grep -n "deprecated"
  crates/ms-codec/src/hashlock.rs` → zero hits on the branch. The Interfaces
  contract and Step 4 now agree.
- **I-7 — CLOSED.** Task 2 Step 1 now reads *"Expected: **88 insertions, 11
  deletions** — measured"* with a correct account of the 11 (one previously-final
  line per row gains a comma; 11 rows) and *"a count of 0 is not the goal and
  never was."*
- **M-1 — CLOSED on both halves.** `hashlock_emit_record.rs:57` now reads
  *"carries only the public digest"*. Both prescribed greps run empty on the
  branch: `grep -rn '"[^"]*digest_sha256' --include=*.rs crates/` → exit 1, and
  `grep -rn "digest_sha256" --include=*.rs crates/ | grep -E ":\s*(///|//)"` →
  exit 1.
- **M-2 — CLOSED by deferral, and the deferral is HONEST.** F-534
  (`design/FOLLOWUPS.md:17445`) reproduces the defect at the same file:line,
  states plainly why it is outside phase 2's letter (§13.4 names `ms hashlock`),
  states plainly why it should not be left (*"the same silence §13.4 forbids next
  door"*), names the fix shape, and assigns an owning phase (**phase 3**, or the
  cycle sweep). It does not soften the finding to justify the deferral. Verified
  by sweeping `crates/ms-cli/src/` for `sha256`/`Sha256`: `decode.rs:207` is the
  only remaining sha256-only digest site in the CLI.

**Also verified as not-broken by the fold:** stdout purity holds — stdout is
exactly one line in all five invocations (no `--kind`, and each of the four);
`hashlock_outputs.rs` passes. No test anywhere asserts a stderr line count
(`grep -rn "stderr" crates/ms-cli/tests/*.rs | grep -i "lines()\|count()\|len()"`
→ empty), so the five added stderr lines broke no pinned contract. The version
bump changed the card's `(ms-cli 0.19.0)` string and no test pinned the old
value. `--help` for `--kind` now matches observed behaviour on the default path.

---

## 2. Findings

### C-1 — BRANCH — Critical — C-2 is only closed on the default path: `--no-engraving-card` still silently assumes sha256

The fold's four-digest fallback was placed **inside** the engraving-card guard
(`crates/ms-cli/src/cmd/hashlock.rs:452`, nested in the
`if !args.no_engraving_card {` block that opens at `:427`). Under
`--no-engraving-card` with no `--kind`, stderr is **empty** and stdout is the
bare sha256 record:

```
$ printf 'correct horse battery staple' | ms hashlock --hashlock-phrase-stdin --no-engraving-card
STDOUT: hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
STDERR: []
```

That is the exact behaviour spec §13.4 (lines 630-635) forbids, and the clause is
unconditional: *"This spec **requires the `--kind` flag** and **permits** the
four-digest fallback; it must not silently assume sha256."* The flag's own help
is unconditional too — `--no-engraving-card` is documented as *"Suppress the
stderr card"*, which is not consent to lose a separate §13.4 notice — and
`--kind`'s help still asserts, with no qualifier, *"Omit it and every kind's
digest is listed on stderr."* So the tool once again claims a behaviour it does
not perform on this path.

**This is the journey the clause exists for, and the flag inverts the
incentive.** The operator checking a pre-cycle plate has a good reason to suppress
the card — it carries the *preimage*, the secret — so the safety-conscious
operator is precisely the one who loses the lookup and receives a single
unlabelled sha256 answer that will silently agree or silently disagree.

The fold's own new test cannot see it: `without_a_kind_every_digest_is_listed_on_stderr`
(`hashlock_kind.rs:93`) never passes `--no-engraving-card`. **The plan is again
better than the branch here** — Task 5 Step 3 (plan lines 1001-1010) presents the
`if args.kind.is_none() { … }` block unconditionally, not nested in the card guard.

**Verified by:** reading `hashlock.rs:427-478` for the nesting; running the
binary with `--no-engraving-card` and capturing both channels separately;
re-reading spec §13.4 lines 626-642 and `--no-engraving-card`'s own `--help`
text; confirming the new test's argv.

---

### C-2 — PLAN — Critical — the fold made Task 3 non-compiling: five `E0433`s, machine-proven

The fold corrected Task 3 **Step 2** (plan lines 647-660) to *"No new
dev-dependency — write the helper … `serde_json` is already a dev-dependency and
`hex` is **not needed**"*, and left Task 3 **Step 1** (plan lines 573-644)
untouched. Step 1's prescribed `crates/ms-codec/tests/hashlock_kat.rs` calls
`hex::encode` five times (plan lines 627, 628, 629, 630, 637).

`ms-codec` has no `hex` dependency of any kind — `grep -n "hex"
crates/ms-codec/Cargo.toml` returns nothing; `[dev-dependencies]` is `proptest,
bip39, serde, serde_json, zeroize`.

**Machine-proven.** I extracted the plan's Step 1 block by its anchor (plan line
571, `Create \`crates/ms-codec/tests/hashlock_kat.rs\`:` → the next ```rust
fence, 71 lines), dropped it into the branch's `crates/ms-codec/tests/` and ran
`cargo build -p ms-codec --tests --locked`:

```
error[E0433]: failed to resolve: use of undeclared crate or module `hex`   (×5)
error: could not compile `ms-codec` (test "zz_plan_kat_probe") due to 5 previous errors
```

Probe file removed; `git status --porcelain` empty afterwards.

**Before the fold, Step 1 and Step 2 agreed** (both used the `hex` crate) and the
contradiction was only with trap 6. The fold fixed one half and made the two
halves of the same task mutually exclusive: Step 3 then says *"Run the test to
verify it passes"* on a file that does not build. This is the named failure class
in this repo's CLAUDE.md — *"five compile errors that round 2's fold had
introduced … each one a `cargo build` away"* — recurring in the same position.

Two further divergences in the same block, carried over from round 3's I-5 and
still open: the plan's `Row` declares only `hardened_*` fields so its loop covers
**11** row/stem pairs against the branch's **22** (same `checked >= 8` floor,
half the coverage), and Step 4's mutation expectations name the plan's assertion
strings rather than the branch's.

**Verified by:** the extraction and build above; `grep -n "hex::" <plan>`;
`grep -n "hex" crates/ms-codec/Cargo.toml`; reading plan lines 573-660.

---

### I-1 — BRANCH — Important — no `MIGRATION.md` section for a hard API break

Round 3's I-4 items 3 and 4 named `MIGRATION.md` twice. The fold did not touch
it: `git diff d831339..1a723eb -- MIGRATION.md` is empty.

`design/RELEASE_PROCESS.md` item 5 is explicit: *"**MIGRATION.md update.** If the
release introduces any wire-format or API change relative to the previous minor,
add a new section to `MIGRATION.md` per the v0.1 → v0.2 precedent."* This release
removes the public `digest`, changes `qr_text`'s signature, and renames the
`--json` key `sha256_operand` → `hash_operand`. `MIGRATION.md`'s last section is
still `## v0.8 → v0.9`, whose item 4 (line 98) documents
`qr_text(hardened: bool, phrase: &str) -> Zeroizing<String>` as the shipped API.
There is no `v0.9 → v0.10` section.

Corroborating: every sibling CHANGELOG entry that broke something carries a
**Migration notes** section pointing at the file — `## ms-cli [0.18.0]` ends with
*"See MIGRATION.md v0.7 → v0.8."* The new `0.10.0 / 0.19.0` entry has
Breaking / Added / Corpus and no such section.

The CHANGELOG half of I-4 **is** closed — new corpus SHA correct and matching the
file, the old SHA quoted as "(was …)", the `digest` rename recorded, the
`sha256_operand` → `hash_operand` break recorded, and the fork's phase-4 re-pin
obligation named. What is missing is the one record the release process mandates
for exactly this kind of change.

**Verified by:** `git diff` on `MIGRATION.md` across the fold (empty);
`grep -rn "4f1819cd" --include=*.md .` → `MIGRATION.md:80` still the newest pin;
reading `MIGRATION.md:70-105` and `design/RELEASE_PROCESS.md` items 2 and 5;
reading `CHANGELOG.md:109-131` for the sibling precedent.

---

### I-2 — PLAN — Important — no step prescribes the release work the branch did, and skipping one breaks the build

Round 3's I-4 item 5 said *"no step in the plan discharges that obligation"*. The
plan fold did not add one. `grep -n "MIGRATION" <plan>` → **zero hits**.
`grep -n "0.19.0\|ms-cli/Cargo.toml\|=0.9.0" <plan>` → **zero hits**.

Task 2 Step 3 prescribes only the ms-codec half — the corpus re-pin, the
CHANGELOG record, and the bump to `0.10.0`, staging
`crates/ms-codec/Cargo.toml`. Nothing anywhere prescribes:

1. bumping `ms-cli` 0.18.0 → 0.19.0;
2. updating `crates/ms-cli/Cargo.toml:20`'s `version = "=0.9.0"` to `=0.10.0`
   — **this one is load-bearing.** It is an exact-version requirement against a
   path dependency, so an executor who follows Task 2 Step 3 and bumps ms-codec
   alone gets an unresolvable workspace at the very next `cargo` invocation, with
   no step telling them what to do;
3. `Cargo.lock`;
4. a `MIGRATION.md` section (I-1).

The branch did all four correctly. The plan, which is what a future executor
runs, describes none of them.

**Verified by:** the three greps above; reading plan lines 531-556 (Task 2 Step 3
and its commit) and Task 6 in full; `crates/ms-cli/Cargo.toml:20`.

---

### I-3 — PLAN — Important — the "Measured outcomes" table still says 100 suites; the branch is 101

Plan line 112: `| test suites | **100 ok, 0 failed** |`, under a heading that reads
*"Transcribed from the completed branch, not predicted"* and a Global Constraint
(line 15) that reads *"Every command, error, count and output below is
transcribed from that run, not predicted."*

The fold added a test binary (`crates/ms-cli/tests/hashlock_kind.rs`). I measured
the branch at `1a723eb`: `cargo test --workspace --locked` → **101** `test
result:` lines; nextest → 572 tests (was 568). The fold's own commit message says
*"Branch (hashkinds-p2), now 101 suites green"* — so the correct number was in
hand and the table it labels a transcript was not updated to match.

This is the defect class this whole exercise exists to remove: a number in the
plan that is no longer true of the tree the plan claims to transcribe, in the one
table an executor uses to decide whether their run matches.

**Verified by:** the test run above, counted with `grep -c "^test result:"`;
reading plan lines 107-120 and 13-22; `git log -1 --format=%B 4d6baa72`.

---

### I-4 — PLAN — Important — Task 1 Step 1 still prescribes three unit tests the branch has never had

Round 3's I-8 item 1. Untouched by the fold. Plan lines 223, 239, 248 still
prescribe `the_four_digests_are_four_different_functions`,
`the_dispatch_selects_the_matching_function` and
`tokens_are_the_lowercase_fragment_names`, with a full TDD loop around them
(Step 1 write → Step 2 *"Run the tests to verify they fail"* → Step 5 see them
pass).

On the branch, `grep -rn` for all three names over `crates/` finds **none**, and
`mod tests` in `crates/ms-codec/src/hashlock.rs` still contains exactly one
function, `hardened_output_is_zeroizing_and_32`.

Either half would close this — write the three tests, or strike the block — but
neither was done, so the plan's transcript claim remains false in Task 1 and a
loop the plan says was run was not run.

**Verified by:** `grep -rn` for each name over `crates/`;
`sed -n '/#\[cfg(test)\]/,$p' crates/ms-codec/src/hashlock.rs | grep "fn "` →
one test function; reading plan lines 214-258.

---

### I-5 — PLAN — Important — Task 4 Step 4b item 4 still prescribes a test the branch does not have, and `151` is still stated nowhere

Round 3's I-8 item 5. Untouched. Plan line 808 still reads *"Keep the second
assertion as a sha256 row at **148** (was 135) so both are covered"* — a
two-assertion test.

The branch's `the_worst_case_is_210_bytes`
(`crates/ms-codec/tests/hashlock_qr_text.rs:143-157`) has **four** assertions:
`qr_text(true, Ripemd160, …) == 210`, `qr_text(false, Ripemd160, …) == 151`,
`qr_text(true, Sha256, …) == 207`, `qr_text(false, Sha256, …) == 148`.

`grep -n "151" <plan>` still returns **nothing**. The value is correct
(135 + 16 for `\nhash: ripemd160`) but an executor has no way to derive it from
the plan, so they write a different test from the one that shipped.

**Verified by:** `grep -n "151" <plan>` (empty); reading plan lines 785-812 and
the branch's `hashlock_qr_text.rs:136-157`.

---

### I-6 — PLAN — Important — Task 5 Step 1 now contradicts the section the fold itself added

The fold added a new section (plan lines 76-104, *"The tests are the task, not
the trimming"*) whose table names the four tests that are actually on the branch.
Task 5 Step 1 (plan lines 855-882) was not updated, and still prescribes **three
differently-named tests** — `no_kind_prints_all_four_and_says_so`,
`an_explicit_kind_prints_only_that_one`, `an_unknown_kind_is_refused_not_folded`
— for the same file, `crates/ms-cli/tests/hashlock_kind.rs`. The plan now gives
two mutually exclusive prescriptions for one file, and the newer one is not the
one under the step an executor works from.

Compounding it, Step 1's block is the superseded draft in three further ways,
retracted by prose *below it in the same step*:

1. Every test calls `--phrase`, which the plan's own line 884 then says *"does
   not exist"*. Confirmed against the binary: `ms hashlock --phrase "x"` is
   refused by the **argv guard**, before clap parses, with the BIP-39-on-ARGV
   refusal.
2. Every test calls `run_ms(&[…])` with one argument; the authoritative helper
   given at line 913 takes **two** (`args`, `phrase`). Assembling Step 1 with the
   corrected helper does not compile.
3. `run_ms_expect_fail` is never re-issued in a stdin-taking form, so the third
   test has no working helper at all.

This is the same shape as I-6 and I-7 of round 3 — a correction landing beside
its own retraction rather than replacing it — which the fold fixed in those two
places and not in this one.

**Verified by:** reading plan lines 76-104 and 852-935; running
`ms hashlock --phrase "x"` against the built binary; comparing the prescribed
names against `crates/ms-cli/tests/hashlock_kind.rs` on the branch.

---

### M-1 — BRANCH — Minor — the new CHANGELOG entry sits above the file's own preamble, in a non-conforming header format

The entry was inserted at `CHANGELOG.md:3`, between the `# Changelog` title and
the document's preamble, so *"All notable changes to `ms-codec` and `ms-cli` are
documented in this file…"* and the Keep-a-Changelog/SemVer paragraph now appear
**below** the newest release entry (lines 41-43).

The header is also `## ms-codec 0.10.0 / ms-cli 0.19.0 — hashlock kinds (…)`,
where every other entry in the file and `RELEASE_PROCESS.md` item 2 use the
per-crate bracketed form `## ms-codec [0.10.0] — <date>`. Two crates in one
header also means neither crate's history reads as a clean sequence.

---

### M-2 — BRANCH — Minor — `## ms-cli [Unreleased]` is now stranded below the 0.19.0 entry and is false

`CHANGELOG.md:45` still carries `## ms-cli [Unreleased]` holding F-495's
`--emit-record` notes. `ms-cli` is now **0.19.0**, and that work is in it — the
last release entry for the crate is `## ms-cli [0.18.0] — 2026-09-05`, which does
not mention `--emit-record`. So a reader of 0.19.0's notes is not told
`--emit-record` ships in it, and a section labelled Unreleased describes released
work. Either fold the section into the 0.19.0 entry or re-title it.

---

## 3. Answers to the brief's specific questions

- **"Verify the branch's green claims yourself."** All reproduce — 101 suites /
  0 failures, clippy 0 in both crates, fmt clean, vendor-freshness OK. See §0.
- **"Do the four new tests actually fail when they should?"** Yes. All four
  mutation-proven individually, each reverted from a byte-identical backup; in
  every run the other three stayed green. See §1, I-1.
- **"Is C-2 fully closed?"** **No** — closed on the default path, open under
  `--no-engraving-card` (C-1 above). stdout purity holds under every kind
  (exactly one line in all five invocations), the `--help` text matches behaviour
  on the default path, and `decode.rs:207` is the only other sha256-only site in
  the CLI (F-534, deferred honestly).
- **"Is I-2's mutation claim true?"** **Yes** — deleting the three per-kind rows
  now reds with *"the corpus carries no qr_text row for kind hash256"*.
- **"Are the release records right?"** Partly. Corpus SHA matches the file,
  `cargo metadata` resolves, and the bump is consistent across both manifests,
  the `=0.10.0` pin and `Cargo.lock`. `MIGRATION.md` was not touched at all
  (I-1), and the plan prescribes none of the ms-cli-side release work (I-2).
- **"Does the plan now contain anything predicted rather than observed?"** Yes —
  C-2 (Task 3 Step 1, which no longer compiles), I-3 (the 100-suite row in the
  table labelled "transcribed"), I-4 (Task 1 Step 1's three tests), I-5 (Task 4
  Step 4b item 4), I-6 (Task 5 Step 1). Four blocks were fixed; five remain, and
  one of them is new.
- **"New defects from the fold itself."** Three: the fallback's placement inside
  the card guard (C-1), the Step 1 / Step 2 `hex` contradiction (C-2), and the
  two CHANGELOG structure Minors. The stderr change broke no pinned contract —
  no test asserts a stderr line count, stdout purity still holds, and the version
  string on the card was pinned nowhere.

---

## 4. Counts and verdict

| severity | branch | plan | total |
| --- | --- | --- | --- |
| Critical | 1 | 1 | **2** |
| Important | 1 | 5 | **6** |
| Minor | 2 | 0 | **2** |
| Nit | 0 | 0 | 0 |

- **Critical:** C-1 (`--no-engraving-card` still silently assumes sha256),
  C-2 (Task 3 Step 1 does not compile — five `E0433`s the fold introduced).
- **Important:** I-1 (no MIGRATION.md section); I-2 (no plan step for the ms-cli
  bump, the `=0.9.0` pin, the lockfile or MIGRATION.md); I-3 (measured-outcomes
  table says 100, branch is 101); I-4 (Task 1 Step 1's three absent tests);
  I-5 (Task 4 Step 4b item 4, and `151` unstated); I-6 (Task 5 Step 1
  contradicts the fold's own new table).
- **Minor:** M-1, M-2.

### Verdict: **NOT GREEN (2C / 6I)**

The branch fold is mostly good work: both round-3 Criticals were fixed under
tests written RED first, and I verified every one of those tests can actually
fail. The residue splits cleanly in two. On the **branch**, C-2's fix landed one
brace too deep, so the spec clause it was written to satisfy is still violated on
a documented flag — the flag a careful operator is most likely to use. On the
**plan**, the fold closed four predicted blocks and left five, and its one
substantive edit to Task 3 made that task non-building, which is the exact
failure mode the build gate exists to prevent and which no gate was run against
this fold. Both are small edits; neither is a design problem.

---

## 5. Worktree state

Left exactly as found. Six mutations were applied and each reverted from a
byte-identical backup (`/tmp/r4bk/`, outside both repos); one probe file was
created in `crates/ms-codec/tests/` and deleted.

```
$ git status --porcelain      # (empty)
$ git branch --show-current
hashkinds-p2
$ sha256sum crates/ms-codec/tests/vectors/hashlock-v0.8.json \
            crates/ms-cli/src/cmd/hashlock.rs \
            crates/ms-codec/tests/hashlock_qr_text.rs
0a911f78f3cdc867dcc44483b7f4c0c1ac87b6d9b30b79f52094e8979bc3d8ce  …/hashlock-v0.8.json
2529247806e92ed2f726d09a1447c6f094a4429da5e022b6e6c32cef801847bf  …/cmd/hashlock.rs
6bcb9161cdd85efeac3c7b25ce82aff0c2a2f14aeb4818a7ad892e63ca66378c  …/hashlock_qr_text.rs
$ cargo nextest run --locked --all-targets
572 tests run: 572 passed, 11 skipped
```

All three hashes match the pre-mutation backups. No tracked file was modified
anywhere but this report. Nothing was committed or pushed.
