# R0 round 3 — Hashlock kinds Phase 2 (mnemonic-secret)

**Artifact under review (two halves):**

1. **BRANCH** `hashkinds-p2` @ `4d4fb5b` in `/scratch/code/shibboleth/ms-worktrees/hashkinds-p2`
   (worktree of `/scratch/code/shibboleth/mnemonic-secret`), two WIP commits
   (`d831339`, `4d4fb5b`) on top of `7a0e96f`.
2. **PLAN** `design/IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md` @ `32914ce0`
   in `mnemonic-engrave` (1029 lines).

**Rounds 1 and 2:** 2C/9I and 4C/7I, both against hand-written uncompiled Rust.
The method then changed: all six tasks were implemented for real and the plan
rewritten from that build. This round reviews the branch as code, and the plan as
a transcript of it.

**Not in scope:** the spec; phases 1, 3, 4; findings rounds 1 and 2 closed;
operator decisions.

**Toolchain used throughout:** `export PATH=/home/bcg/.cargo/bin:$PATH` →
`cargo 1.85.0 (d73d2caf9 2024-12-31)` / `rustc 1.85.0`, matching
`rust-toolchain.toml`. The system `/usr/bin/cargo` 1.98.0 phantom-lint trap the
plan warns about was avoided.

---

## 0. The green claims — independently re-measured

Every claim in the plan's "Measured outcomes" table was re-run, not taken on trust.

| gate | plan claims | I measured | verdict |
| --- | --- | --- | --- |
| test suites | 100 ok, 0 failed | `cargo test --workspace --locked` → exit 0, **100** `test result:` lines, none non-`ok`. (`cargo nextest run --locked --all-targets` → **568 tests / 99 binaries, 568 passed, 0 failed, 11 skipped**; 99 binaries + 1 doctest suite = the 100.) | **TRUE** |
| `clippy -p ms-codec --all-targets --locked -- -D warnings` | 0 | exit 0, clean | **TRUE** |
| `clippy -p ms-cli --all-targets --locked -- -D warnings` | 0 | exit 0, clean | **TRUE** |
| `cargo +1.95.0 fmt --all -- --check` | clean | exit 0, no output. 1.95.0 is installed. | **TRUE** |
| `cargo vendor vendor/` adds exactly one dir | `vendor/ripemd` | `git diff --name-status HEAD~2..HEAD` shows `vendor/ripemd/*` as the only added vendor path; bare names, no `--versioned-dirs` churn | **TRUE** |
| `ci/repro/vendor-freshness.sh` | OK | `vendor-freshness: OK — vendor/ satisfies Cargo.lock.` | **TRUE** |
| corpus rows | 11 derivation / 10 `qr_text` | 11 / 10, with 7 `kind: "sha256"` + 3 new per-kind | **TRUE** |
| corpus SHA after Task 4 | `0a911f78f3cdc867dcc44483b7f4c0c1ac87b6d9b30b79f52094e8979bc3d8ce` | `sha256sum` → identical | **TRUE** |
| corpus SHA before (Task 2 Step 3) | `4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4` | `git show HEAD~2:… \| sha256sum` → identical | **TRUE** |
| Task 2 diff | 88 insertions / 11 deletions | reproduced from the `HEAD~2` corpus with the plan's own script: `added 66 digest columns across 11 rows`, `git diff --no-index --numstat` → **88 11** | **TRUE** |
| flag total | 68 → 69 | `the_schema_names_every_flag_p2_added_and_the_total_is_67` asserts `total, 69` and passes | **TRUE** |
| QR sizes | 210 / 207 / 148 | asserted and passing; **151** also asserted on the branch but stated nowhere in the plan (see I-8) | mostly true |

**The green claims hold.** No false number was found in the measured-outcomes
table. Every finding below is about behaviour and coverage, not about a
mis-transcribed gate result.

---

## 1. The digest work — verified against `python3 hashlib`

**The four functions are correct.** For `X = 0xab × 32`, an independent
`python3 hashlib` computation (`hashlib.new("ripemd160")` for the bare primitive)
returns exactly the plan's Task 1 Step 5 table:

```
sha256     9a2db2e23f1504cd056606553ac049c5e718e8f9ce9233876df1a7a1821af885
hash256    88b8f02ce56abce1d453e0610318130f4d0a13067549e804af1f5186f81a2691
ripemd160  5786aabcae0e6cd2dfaeca2767dc8996c98f43f4
hash160    e81bfa71da56f187cce1319ee773dabf56988e95
```

**`HashKind::digest`'s four arms, one by one** (`crates/ms-codec/src/hashlock.rs:96-103`):
`Sha256 → B32(digest_sha256)`, `Hash256 → B32(digest_hash256)`,
`Ripemd160 → B20(digest_ripemd160)`, `Hash160 → B20(digest_hash160)`. Each arm
pairs the right width with the right function; no arm is crossed. `token()`
returns the four lowercase fragment names, verified.

**The corpus columns were computed independently and are right.** I re-derived
every new column from `*_x` with `python3 hashlib`: **22 row/stem pairs, 0
mismatches**, across all four suffixes (`_h`, `_h_hash256`, `_h_ripemd160`,
`_h_hash160`). All 11 rows carry `provenance_kinds` naming python as the source.
22 pairs × 3 new columns = the 66 the plan reports.

**Mutation tests — the KAT can fail.**

1. `digest_hash256` rewritten one word short (`Sha256::digest(preimage)` only) →
   `hashlock_kat.rs:57` FAILED: *"correct horse battery staple/hardened: hash256,
   left: 3cf5d421…, right: 98a20fc2…"*. Reverted, re-ran, PASS.
2. `HashKind::Ripemd160`'s arm re-pointed at `digest_hash160` →
   `hashlock_kat.rs:80` FAILED: *"dispatch for ripemd160"*. Reverted, re-ran,
   147/147 other ms-codec tests still passed.

The KAT iterates **both** stems (`hardened` and `sha256`) over all 11 rows, so
`checked` reaches 22 against a `>= 8` floor, and the zero-iteration guard is real.
**This part of the branch is sound.**

**Record grammar (spec §6) — run against the built binary**, phrase
`correct horse battery staple`, stdout captured separately from stderr:

```
(no --kind)   hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
--kind sha256 hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
--kind hash256    hash:hash256:98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
--kind ripemd160  hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b
--kind hash160    hash:hash160:b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd
```

Bare for sha256 (the compat promise) ✓. `hash:<kind>:<hex>` otherwise ✓. Hex
length is `digest_len()*2` in every case (64/64/40/40) ✓. **stdout is exactly one
line in all five invocations** — nothing new reaches it ✓. `--kind RIPEMD160` is
refused by `parse_kind` with a message naming the four lowercase tokens ✓.

---

## 2. Findings

### C-1 — BRANCH — Critical — `for md compose:` hardcodes `sha256=` under every kind

`crates/ms-cli/src/cmd/hashlock.rs:437`:

```rust
writeln!(stderr, "for md compose:  --path ... sha256={}", hex(h)).ok();
```

`h` is now the **chosen kind's** digest, but the operand label is still the
literal `sha256=`. Verified by running the binary for all four kinds:

```
--kind ripemd160 →  for md compose:  --path ... sha256=09e7bb5051d89788fb4e4b374126721dbcc2946b
--kind hash160   →  for md compose:  --path ... sha256=b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd
--kind hash256   →  for md compose:  --path ... sha256=98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
```

The engraving card — the one artifact the operator physically follows — instructs
them to compose a `sha256=` policy operand from a `ripemd160(X)` value. Spec §9
(lines 421-422) is explicit that this is a grammar, not a rename: *"`md compose`'s
option name is a **grammar** decision, not a rename: `sha256=` is kind-specific,
so the three new kinds are siblings."* An operator who follows the card builds a
wallet whose script commits to `sha256(X)` of a digest that is not `sha256(X)`;
the hash branch is unspendable and the preimage that would satisfy it does not
exist.

The `hash256` case is the worse half: the value is 64 hex characters and
**indistinguishable** from a sha256 digest, so nothing on the card, in review, or
at compose time can catch it. That is the §13.2 operator-facing Critical
reproduced inside the tool the reconciliation screen sends the operator to — the
exact failure the plan's Task 5 Step 3 quotes when explaining why `hashlock.rs:325`
had to change.

**The plan requires this fix and the branch did not make it.** Plan Task 5 Step 4:
*"Separately, the line that prints `for md compose: … sha256=<h>` must name the
chosen kind's option (`ripemd160=<h>`), because it is the only thing keeping the
digest function and `md compose`'s option name in agreement (spec §11)."*

**Verified by:** building `ms` from the branch and running all four kinds with
stderr captured; reading `hashlock.rs:437`; reading spec §9 lines 421-422 and the
plan's Task 5 Step 4.

---

### C-2 — BRANCH — Critical — no four-digest fallback; sha256 is silently assumed, and `--help` says otherwise

`crates/ms-cli/src/cmd/hashlock.rs:346`:

```rust
let kind = args.kind.unwrap_or(HashKind::Sha256);
```

There is no notice on any channel. `grep -n "kind.is_none\|no --kind\|each kind"`
over the file returns **nothing** — the four-digest fallback block the plan's
Task 5 Step 3 specifies was never written. Verified by running with no `--kind`:
stdout is the bare sha256 record, stderr is the ordinary engraving card, and no
line anywhere mentions that a kind was assumed or what the other three digests are.

Spec §13.4 (line 630-635): *"This spec **requires the `--kind` flag** and
**permits** the four-digest fallback; **it must not silently assume sha256**."*
The branch does the one thing the spec forbids.

**Compounding it, the program's own help text asserts the missing behaviour.**
The doc comment at `hashlock.rs:52-54` is clap's `--help` output, confirmed by
running `ms hashlock --help`:

```
--kind <KIND>
    Which hash the SCRIPT commits to: sha256, hash256, ripemd160, hash160.
    Omit it and every kind's digest is listed on stderr -- a plate cut before
    this existed carries no kind, and a lookup beats an impossible check.
    Case is rejected, never folded
```

Nothing is listed on stderr. This is a defect in what the tool *claims* to have
done: the operator holding a pre-cycle plate reads the help, omits `--kind`
expecting the lookup that turns an impossible check into a possible one, and
receives a single unlabelled sha256 answer that will silently agree or silently
disagree for reasons they cannot see.

**Verified by:** `grep` over `crates/ms-cli/src/cmd/hashlock.rs` for any
`is_none` / fallback branch (none); running the binary with no `--kind` and
capturing both channels; running `ms hashlock --help`; reading spec §13.4.

---

### I-1 — BRANCH — Important — `--kind` and the new record grammar have zero test coverage

- `grep -rn -- '--kind' crates/ms-cli/tests/` → **no hits**.
- `grep -rn 'hash:hash256\|hash:ripemd160\|hash:hash160' crates/ --include=*.rs`
  → **no hits**.
- `crates/ms-cli/tests/hashlock_kind.rs`, created by the plan's Task 5 Step 1,
  **does not exist** (`ls crates/ms-cli/tests/ | grep -i kind` matches only the
  unrelated `json_error_envelope_per_kind.rs`).
- The stdout-purity pin
  `hashlock_outputs.rs:23 stdout_is_exactly_the_record_under_out_and_under_sha256`
  never passes `--kind`, so the three new stdout record shapes — the funds-relevant
  output of this whole phase — are emitted by code that no assertion ever reaches.

This is why a 568-green branch carries C-1 and C-2. It is the project's
"a gate that cannot fail" class: the phase's headline behaviour ships ungated, and
the green suite is evidence about the sha256 path only.

**Verified by:** the two greps above; `ls` of the test directory; reading
`hashlock_outputs.rs:22-32`.

---

### I-2 — BRANCH — Important — the three new per-kind `qr_text` corpus rows gate nothing (mutation-proven)

`crates/ms-codec/tests/hashlock_qr_text.rs:47`:

```rust
assert!(c.qr_text.len() >= 7, "the corpus lost its qr_text rows");
```

The floor was never raised from 7 to 10 when three rows were added.

**MUTATION-VERIFIED.** I deleted all three non-sha256 rows from the corpus
(`qr_text` 10 → 7) and ran
`cargo nextest run --locked -p ms-codec -E 'binary(hashlock_qr_text)'`:

```
3 tests run: 3 passed, 0 skipped
```

Green. Restored from backup; corpus SHA back to `0a911f78…`, confirmed.

Plan Task 4 Step 5b says those rows are *"phase 4's acceptance target"*, and the
plan's own trap 4 names precisely this failure — *"Without this, Step 5b's rows
exist and gate nothing."* The landed fix covers only the **render** (the `kind`
field plus the match arm); the **presence** of the rows is still unguarded, so
phase 4's cross-repo acceptance target can be deleted by accident and every gate
in this repo stays green.

The three rows themselves are correct: I verified each byte-for-byte against the
`qr_text` construction, and the byte counts (136 / 138 / 136) match
`len(qr_text)` exactly.

**Verified by:** reading line 47; the mutation above; re-reading the plan's trap 4
and Task 4 Step 5b.

---

### I-3 — BRANCH — Important — the flag-enumeration message says "fourteen" and lists thirteen

`crates/ms-cli/tests/gui_schema_emits_spec_v7_json.rs:283-288`. The total moved
`68 → 69` ✓ and the word `thirteen → fourteen` ✓, but `--kind` was **never added
to the parenthetical enumeration**, which still names exactly thirteen flags:

```
--hashlock-phrase, --hashlock-phrase-stdin, --hex, --in, --random, --method,
--out, --json, --no-engraving-card, --emit-record, --group-size, --separator,
--allow-argv-secret
```

The plan's trap 2 is explicit: *"Update the count AND the flag enumeration in its
message."* Half-folded. The message is the obligation a future reader uses to
reconcile the binary against the GUI schema, and it now sends them looking for a
fourteenth flag it declines to name.

(The stale function name `…_the_total_is_67` was left, which the plan explicitly
permits as a deliberate wart. Not a finding.)

**Verified by:** reading lines 276-293 and counting the names in the literal.

---

### I-4 — BRANCH — Important — the release records now assert false things, and the version was not bumped

`crates/ms-codec/Cargo.toml:3` is still `version = "0.9.0"` and **no CHANGELOG or
MIGRATION line was touched** (`git diff --name-only HEAD~2..HEAD | grep -i changelog`
matches only `vendor/ripemd/CHANGELOG.md`). Consequences, each verified:

1. **The corpus SHA moved** `4f1819cd…` → `0a911f78…`. `CHANGELOG.md`'s own
   per-release rule — quoted in the plan's Task 2 Step 3 — says *"its hash moves
   and the pre-1.0 breaking-change axis requires `0.X+1.0`"*. The bump to
   **0.10.0** the plan prescribes did not happen.
2. **`CHANGELOG.md:51` and `MIGRATION.md:80` still pin the corpus at
   `4f1819cd…`** — stale by one commit.
3. **`CHANGELOG.md:40` and `MIGRATION.md:98` both document
   `qr_text(hardened: bool, phrase: &str) -> Zeroizing<String>`** — a signature
   the branch no longer has. `MIGRATION.md` is the document a downstream
   consumer reads to port across the break; it currently describes the
   pre-break API as current.
4. **The public `digest` was removed** (renamed to `digest_sha256`), a hard break
   for any external caller, recorded nowhere.
5. **`--json`'s `sha256_operand` → `hash_operand`** (`hashlock.rs:399`) breaks a
   GUI-facing machine-readable contract. `## ms-cli [Unreleased]` exists in the
   CHANGELOG and does not mention it. The plan's own trap 3 says this rename
   *"belongs in the CHANGELOG with the version bump, not folded in silently"* —
   and then **no step in the plan discharges that obligation**: Task 2 Step 3 only
   says "Record the new value in CHANGELOG.md" (the corpus SHA), and no task
   mentions `MIGRATION.md` or a `## ms-codec [Unreleased]` entry at all.

So this is a defect in both halves: the branch left the records false, and the
plan does not contain the step that would have fixed them.

**Verified by:** `sha256sum` at `HEAD~2` and `HEAD`; `grep -n "^version"` on both
Cargo.tomls; `git diff --name-only`; reading `CHANGELOG.md:36-56` and
`MIGRATION.md:70-105`; `grep -rn "4f1819cd" --include=*.md .`

---

### I-5 — PLAN — Important — Task 3's KAT is the predicted draft and Step 2 contradicts the plan's own trap 6

Plan line 544 / 554: the KAT is written with `use serde::Deserialize;` and
`#[derive(Deserialize)] struct Row { … }`, using `hex::encode(...)`. Plan **line
607** (Task 3 Step 2) then instructs: add `hex = "0.4"` to `[dev-dependencies]`.

Plan **lines 110-111** (trap 6) say the opposite, in bold:

> **`serde_json` is already a dev-dependency; `hex` is not needed.** An earlier
> draft added `hex = "0.4"`. Write a local `hex()` helper instead — and write it
> as a `fold`, because `map(format!).collect()` trips clippy's `format_collect`
> under `-D warnings`.

**The branch does the trap-6 thing.** `crates/ms-codec/tests/hashlock_kat.rs` uses
`serde_json::Value`, a local `fold`-based `hex()` with the `format_collect`
rationale in a comment, and no `Deserialize` struct. `ms-codec`'s
`[dev-dependencies]` is `proptest, bip39, serde, serde_json, zeroize` — **no
`hex`**, verified.

An executor following Task 3 step-by-step writes the other file. The two halves
of the plan cannot both be followed and the step gives no signal that it is the
superseded one.

Two further divergences in the same block, both material:
- The plan's `Row` declares only `hardened_*` fields, so its loop covers **11**
  row/stem pairs; the branch iterates both stems and covers **22**. Same
  `checked >= 8` floor, half the coverage.
- The plan's Step 4 mutation instructions are correct in substance (I ran both and
  both fail as described) but name `hashlock_kat.rs` assertions that the plan's own
  Step 1 code would produce, not the branch's.

**Verified by:** `grep -n` on the plan for lines 110-111, 544, 554, 607; reading
the branch's `hashlock_kat.rs` in full; `sed -n '/\[dev-dependencies\]/,/^\[/p'
crates/ms-codec/Cargo.toml`.

---

### I-6 — PLAN — Important — Task 1's Interfaces contradicts Task 1 Step 4 on the `#[deprecated]` alias

Plan **line 169**, under Task 1 *Interfaces → Produces*:

> `digest` (the existing sha256 function) retained as a `#[deprecated]` alias of
> `digest_sha256`, so phase 3's callers keep compiling until they are migrated.

Plan **line 332**, inside Step 4's code block:

> `// NO `digest` ALIAS.` An earlier draft kept the old name as a `#[deprecated]`
> shim … Measured, that adds **NINE** clippy errors in ms-codec and four in ms-cli
> under `-D warnings`, which is a REQUIRED CI context.

The branch has no alias (`grep -n "deprecated" crates/ms-codec/src/hashlock.rs`
→ nothing). An executor who implements the Interfaces contract — which is the
section other tasks' *Consumes* lines key on — reds a required CI context, and
the plan's own measurement says by how much. The retracted line should be struck,
not left standing beside its own retraction.

**Verified by:** `grep -n "deprecated"` on the plan (two hits, lines 169 and 332)
and on the branch source (zero hits).

---

### I-7 — PLAN — Important — Task 2 Step 1's expected diff is the number the plan itself retracts

Plan **line 470** (Task 2 Step 1): *"Expected: roughly **77 insertions, 0
deletions**. Any large deletion count means the indent is wrong — stop and fix it
rather than committing the reformat."*

Plan **line 91** (Global Constraints): *"**Task 2's diff is 88 insertions / 11
deletions**, not the '77 / 0' an earlier draft predicted … A `0` deletion count is
the wrong thing to check for."*

**I reproduced Task 2 from the `HEAD~2` corpus using the plan's own script:**
`added 66 digest columns across 11 rows`, then
`git diff --no-index --numstat` → **`88  11`**. The Global figure is right; the
step still carries the retracted one, together with a "stop and fix it"
instruction keyed to it. Same shape as I-6: the correction landed in one place and
the wrong text was left in the other — in the half an executor actually reads.

**Verified by:** the reproduction above; `grep -n "77 insertions\|88 insertions"`
on the plan.

---

### I-8 — PLAN — Important — Tasks 1, 4 and 5 prescribe test code and behaviour the branch never had

The plan's structural claim is *"All six tasks were implemented for real … Every
command, error, count and output below is transcribed from that run, not
predicted."* That is false for the following, each verified absent on the branch:

1. **Task 1 Step 1's three unit tests** —
   `the_four_digests_are_four_different_functions`,
   `the_dispatch_selects_the_matching_function`,
   `tokens_are_the_lowercase_fragment_names`. `grep -rn` over `crates/` finds
   **none of the three**, and `mod tests` in `hashlock.rs` contains exactly one
   function, `hardened_output_is_zeroizing_and_32`. Task 1's whole TDD loop
   (Step 1 write → Step 2 see it fail → Step 5 see it pass) describes a loop that
   was never run.
2. **Task 5 Step 1's `crates/ms-cli/tests/hashlock_kind.rs`** and its three
   tests — the file does not exist (this is I-1's cause).
3. **Task 5 Step 3's four-digest fallback block** — not in the branch (C-2). The
   plan goes on to reason about code that does not exist: *"the helper joins
   stdout **and** stderr, because the four-digest fallback is on stderr by the
   purity contract above."*
4. **Task 5 Step 4's `md compose` kind-aware operand** — not in the branch (C-1).
5. **Task 4 Step 4b item 4** prescribes *"Keep the second assertion as a sha256
   row at **148** (was 135)"*. The branch's `the_worst_case_is_210_bytes` has four
   assertions whose second is `qr_text(false, HashKind::Ripemd160, …) == 151`,
   then sha256 at 207 and 148. **`grep -n "151"` on the plan returns nothing** —
   the value is correct (135 + 16 for `\nhash: ripemd160`) but unstated, so an
   executor writes a three-assertion test where the branch has four.

Items 3 and 4 are the load-bearing ones: they are the two behaviours whose absence
is C-1 and C-2. The plan is, in those places, *better* than the branch — which is
itself the finding, because a plan presented as a transcript of a green build is
being used as evidence that the branch does what it describes.

**Verified by:** `grep -rn` for each test name over `crates/`; `ls
crates/ms-cli/tests/`; `sed -n '/#\[cfg(test)\]/,/^}/p' crates/ms-codec/src/hashlock.rs
| grep "fn "`; `grep -n "151\|148\|207\|210"` on the plan; reading the branch's
`hashlock_qr_text.rs:143-157`.

---

### M-1 — BRANCH — Minor — rename collateral inside a doc comment

`crates/ms-cli/tests/hashlock_emit_record.rs:57` now reads:

```
/// carries only the public digest_sha256, and that claim has to hold under the flag
```

The blanket rename landed inside English prose. The plan's trap 1 anticipated
collateral and prescribes a guard —
`grep -rn '"[^"]*digest_sha256' --include=*.rs crates/` **must come back empty**.
I ran it: it does come back empty, and it still misses this, because it inspects
string literals only and this is a `///` comment. The guard should be widened to
`grep -rn "digest_sha256" --include=*.rs crates/ | grep -E ":\s*(///|//)"`, which
finds it immediately.

---

### M-2 — BRANCH — Minor — `ms decode`'s preimage route stays sha256-only and unlabelled

`crates/ms-cli/src/cmd/decode.rs:207` prints `digest: <sha256>` for any preimage
`ms1`, in both text and `--json`. With four kinds in the world that label is
ambiguous, and the plate-verification route through `ms decode` answers only for
sha256 with no indication that it has chosen. Spec §13.4 names `ms hashlock`
specifically, so this is outside the letter of phase 2 — recorded, not gating.
Worth an owning phase (phase 3 or the cycle's follow-up file).

---

## 3. Answers to the brief's specific questions

- **"Is anything left sha256-only that should not be?"** Round 2 found six
  channels behind one line; that line (`hashlock.rs:346-348`) is now correct and
  the stdout record, the `digest:` card line, and all three `--json` keys follow
  the chosen kind. **One channel was missed: `for md compose:` at `:437`** (C-1) —
  it carries the right *value* under the wrong *label*, which is why a
  value-oriented trace misses it. Outside `ms hashlock`, `decode.rs:207` is
  sha256-only (M-2).
- **"Does the `qr_text` corpus test actually exercise the three new rows, or can
  it still pass without them?"** It exercises them while they are present, and it
  **passes without them** — mutation-proven (I-2).
- **"Were the new digest columns computed independently, and are they right?"**
  Yes and yes: `provenance_kinds` on all 11 rows names `python3 hashlib`, and my
  own independent recomputation matched all 22 row/stem pairs with 0 mismatches.
- **"Breaking changes … recorded where this repo records such things, and is the
  version bump handled?"** No, on every count (I-4).
- **"Could an engineer reproduce the branch from the plan alone?"** No. They
  would produce a different KAT (I-5), a `#[deprecated]` alias that reds clippy
  (I-6), a corpus-diff check that stops on the correct diff (I-7), and six tests
  plus two behaviours that would make the result *diverge from* the branch (I-8) —
  two of those divergences being improvements the branch needs.
- **"Does the plan still contain anything predicted rather than observed?"** Yes —
  I-5, I-6, I-7 and I-8 are all that class. The measured-outcomes table and the
  traps sections are genuinely transcribed; the residue is inside Task 1 Step 1,
  Task 2 Step 1, Task 3 Steps 1-2, Task 4 Step 4b item 4, and Task 5 Steps 1, 3
  and 4.

---

## 4. What is sound, so it is not re-reviewed

Stated so a later round spends its budget elsewhere:

- The four digest functions and `HashKind::digest`'s dispatch are correct against
  an independent source, and mutation-verified to be gated.
- The KAT covers both the functions and the dispatch, over both stems, with a
  working zero-iteration guard.
- The corpus's 66 new digest columns are independently computed and correct.
- The record grammar's *shape* is right: bare for sha256, `hash:<kind>:<hex>`
  otherwise, correct hex length, case rejected not folded.
- stdout purity holds under every kind.
- `qr_text`'s kind line is on its own line, the `method:` line is untouched at 73
  characters, the phrase stays last, and the 210/207/151/148 measurements are
  correct.
- Every gate in the plan's measured-outcomes table reproduces.
- The `digest → digest_sha256` call-site renames in `decode.rs`,
  `hashlock_derivation.rs`, `hashlock_repro.rs` and `hashlock_phrase_rule.rs` are
  clean and mechanical.

---

## 5. Counts and verdict

| severity | branch | plan | total |
| --- | --- | --- | --- |
| Critical | 2 | 0 | **2** |
| Important | 4 | 4 | **8** |
| Minor | 2 | 0 | **2** |
| Nit | 0 | 0 | 0 |

- **Critical:** C-1 (`for md compose:` labels every kind's digest `sha256=`),
  C-2 (no four-digest fallback; sha256 silently assumed while `--help` claims
  otherwise).
- **Important:** I-1, I-2, I-3, I-4 (branch); I-5, I-6, I-7, I-8 (plan).
- **Minor:** M-1, M-2.

### Verdict: **NOT GREEN (2C / 8I)**

The digest layer — the part this phase exists to get right — is correct and
properly gated, and every green claim reproduces. The defects are concentrated in
the **CLI surface**, where the phase's operator-facing behaviour shipped with no
test coverage (I-1), which is what let a wrong `md compose` label (C-1) and a
spec-forbidden silent sha256 assumption advertised as its own opposite (C-2) reach
a 568-green branch. The plan is much improved but is not yet the transcript it
claims to be: four blocks still describe the superseded prediction, and two of
those describe the very behaviour the branch is missing.

---

## 6. Worktree state

Left exactly as found. Three mutations were applied and each reverted from a
byte-identical backup:

```
$ git status --porcelain | wc -l
0
$ git branch --show-current
hashkinds-p2
$ sha256sum crates/ms-codec/tests/vectors/hashlock-v0.8.json
0a911f78f3cdc867dcc44483b7f4c0c1ac87b6d9b30b79f52094e8979bc3d8ce
```

No tracked file was modified anywhere but this report. Nothing was committed or
pushed. Scratch files (`/tmp/r3t`, `/tmp/r3-backup-*`) are outside both repos.
