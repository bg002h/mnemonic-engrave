# R0 round 2 — IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md (the fold)

**Fold under review:** `git diff cc03773c..ff6d28d4` in `mnemonic-engrave` (1 file, 220 insertions / 44 deletions).
**Round 1 report:** `design/agent-reports/plan-hashkinds-P2-R0-round1.md` (`cc03773c`), 2C / 9I / 8M / 3N.
**Target repo:** `/scratch/code/shibboleth/mnemonic-secret` at `7a0e96f` — clean before and after; corpus SHA re-verified `4f1819cd…` (unchanged). Nothing committed, no tracked file modified but this report.
**Question:** did the fold close each round-1 finding, and did it introduce a new defect?

**Toolchain correction to round 1.** Round 1 measured clippy under `rustc 1.98.0` and stated "this box has no
rustup". That is no longer true: `~/.cargo/bin/rustup` carries **1.85.0** (the `rust-toolchain.toml` pin) with
`clippy 0.1.85`, so every clippy measurement below is the **CI-equivalent absolute**, not a delta against a
proxy baseline. `/usr/bin/cargo` (1.98.0) is first on `PATH`; `PATH=~/.cargo/bin:$PATH` selects the pin.

**Method.** Task 1 (Step 4 code + Step 4b renames + the `ripemd` dep) was applied to the real tree and
clippy-gated with the two exact CI commands; Task 4's `qr_text` change was applied and the worst case measured
for all four kinds; Task 5's Step 3 snippet was applied verbatim and compiled; the stderr fallback was
simulated and `hashlock_outputs` run; the `ms-codec` 0.10.0 bump was applied and `cargo metadata` run;
`cargo vendor` was run into a scratch directory and its directory set diffed against `vendor/`; Task 2's
generator was run verbatim and its `numstat` and resulting SHA measured; the built `ms` binary was driven for
the flag and refusal paths. Every file restored with `git checkout --`; the scratch target dir was removed.

---

## Round-1 findings — status

### C-1 — the `#[deprecated]` alias reds two required CI jobs → **PARTIAL** (mechanism closed, contract text not)

The fix itself is **correct and measured green**. The alias is gone (plan `:218-227`), Step 4b renames this
repo's call sites, Task 6 Step 2 is `cargo +1.95.0 fmt --all -- --check` and Step 2b is the two separate
`-D warnings` clippy commands.

Measured under the pinned 1.85.0, `CARGO_TARGET_DIR` isolated:

```
# committed tree (baseline)
cargo clippy -p ms-codec -p ms-cli --all-targets --locked -- -D warnings   -> exit 0, 0 errors

# after Task 1 Step 4 + Step 4b (alias removed, 11 call sites + 3 imports renamed, ripemd added)
cargo clippy -p ms-codec -p ms-cli --all-targets -- -D warnings            -> exit 0, 0 errors
```

Round 1's +9/+4 deprecation errors are gone and nothing replaced them. C-1's *mechanism* is closed.

**What is not closed** is the plan's own contract for the same task. Task 1's **Interfaces / Produces** list
still reads, at `:55`:

> `digest` (the existing sha256 function) retained as a `#[deprecated]` alias of `digest_sha256`, so phase 3's
> callers keep compiling until they are migrated.

163 lines later the Step 4 code block says, in capitals, `// NO `digest` ALIAS`. An implementer handed Task 1
as a unit has two contradictory instructions about the exact thing round 1 rated Critical, and the one in the
*interface contract* is the one that reproduces it. One-line deletion; see **I-10**.

Two further C-1 residues are filed separately: the rename has no `git add` (**C-3**) and Step 4b's grep misses
the `use` lines (**M-9**).

---

### C-2 — under `--kind`, every machine-readable channel still emits sha256 → **NOT CLOSED** (relocated, and the funds outcome is worse)

The fold does change the right line: `let kind = args.kind.unwrap_or(HashKind::Sha256); let h = kind.digest(&d.x);`
at `crates/ms-cli/src/cmd/hashlock.rs:325`. All six channels now carry the **kind's** digest. That half is right.

But round 1's C-2 asked three questions the fold does not answer:

> *"What does stdout's `hash:` record say under `--kind`? … What do `--json`'s `digest`, `hash_record` and
> `sha256_operand` become? Is `sha256_operand` renamed, made kind-aware, or dropped?"*

The fold's answer is one sentence — *"Then every existing consumer of `h_hex` is correct without further
edits"* — and it is wrong on both counts, one mechanically and one on funds.

**(a) The stdout record emits the bare form for every kind. Spec §6 says the bare form means sha256.**

`crates/ms-cli/src/cmd/hashlock.rs:326` is `let record = format!("hash:{}", hex(&h));` and the fold leaves its
*shape* untouched while changing its *content*. Spec `SPEC_hashlock_kinds.md` §6:

> **Producer rule.** The host emits the **bare** form for sha256 and the explicit form for the other three.

and, in the same section:

> The rejected alternative is any scheme where an old parser reads a non-sha256 digest *as* sha256, which
> composes a wallet nobody can spend, silently.

That is precisely what the fold ships. Concrete failure: an operator runs
`ms hashlock --hashlock-phrase-stdin --kind hash256`; stdout is `hash:<64 hex of sha256d(X)>`; `me sysw pack`
reads the record (the module header at `:5` names it as that channel), sees no kind token, and per §6 treats it
as `sha256`. The composed policy is `sha256(D)` where `D = sha256d(X)` — **no preimage exists**, and the
wallet is unspendable. Round 1's version of this defect printed a sha256 digest for a hash256 request: wrong
kind, but still spendable. The fold's version is not.

§9 assigns "the §6 record grammar, both directions" to **phase 3 (`me-cli`)** — a different repo, which cannot
change what `ms-cli` writes. No later phase fixes this. The plan must decide here: emit the explicit form,
refuse `--kind` on the record path, or keep the record sha256 and say so. It decides none and asserts the
current shape is already correct.

**(b) `--json`'s `sha256_operand` says `sha256=` under every kind.**

`:367-370` is `format!("sha256={}", hex(&h))`. Under `--kind ripemd160` that emits `sha256=<40 hex>`; under
`--kind hash256`, `sha256=<sha256d>`. This is the string §11 keeps in agreement with `md compose`'s option
name — the fold makes the *stderr* twin of it kind-aware in Step 4 and leaves the `--json` one alone.
`crates/ms-cli/tests/hashlock_outputs.rs:184` pins the key by name, so this is re-pin work either way, exactly
as round 1 said. The plan still does not choose.

**(c) The "no further edits" claim is measurably false, and the grep that is supposed to catch it misses the
record line.**

Applying the fold's Step 3 snippet verbatim to `crates/ms-cli/src/cmd/hashlock.rs`:

```
cargo build -p ms-cli   ->  7 errors, 2 warnings
  5 × error[E0308]: expected `&[u8]`, found `&DigestBytes`
      :345 format!("hash:{}", hex(&h))            <- the stdout record
      :384 o.insert("digest".into(), hex(&h)...)
      :388 format!("sha256={}", hex(&h))
      :424 writeln!(stderr, "digest:          {}", hex(&h))
      :425 writeln!(stderr, "for md compose:  --path ... sha256={}", hex(&h))
```

There is no `h_hex` consumer in the file; there are five `hex(&h)` call sites (local `fn hex(b: &[u8]) -> String`
at `:290`), and every one must be rewritten. Compiler-caught, so not itself dangerous — but the plan's
"walk them and confirm — do not assume" grep is

```bash
grep -n "h_hex\|"digest"\|sha256" crates/ms-cli/src/cmd/hashlock.rs
```

which the shell collapses to `h_hex\|digest\|sha256` and which therefore finds `:384`, `:388`, `:424`, `:425`
and **not** `:345` — the stdout `hash:` record, the one channel `me sysw pack` reads and the one where (a) does
its damage. The step designed to catch the C-2 class is blind to the C-2 site.

**C-2 is NOT CLOSED.**

---

### I-1 — the flag hedge gave the wrong remedy → **PARTIAL**

The prose is now right and matches the source: `--hashlock-phrase-stdin` is the real flag
(`crates/ms-cli/src/cmd/hashlock.rs:48-50`), `--hashlock-phrase` is argv-refused, zero sources exit 64, and the
new two-argument `run_ms(args, phrase)` with `.write_stdin(format!("{phrase}\n"))` is the shape this repo's own
suite uses (`hashlock_outputs.rs:28-34` — that suite passes the phrase without a trailing `\n`; both work, the
module header strips one).

**The code was not folded with the prose.** Plan `:694-715` still contains the three test bodies verbatim:

```rust
let out = run_ms(&["hashlock", "--phrase", "correct horse battery staple"]);
…
let out = run_ms_expect_fail(&["hashlock", "--phrase", "x", "--kind", "RIPEMD160"]);
```

and `:723-736` still contains the **old** one-argument `run_ms` / `run_ms_expect_fail` definitions, immediately
above the new one. The plan now defines `run_ms` twice with different arities and calls the dead one three
times.

Round 1's sharpest point on I-1 — that the third test is "worse than a rename" — is untouched. Measured against
the built `ms`:

```
$ ms hashlock --phrase "x" --kind RIPEMD160
ms: argument 3 on ARGV … is a BIP-39 mnemonic, 1 characters long.
      Refused BEFORE the command line was parsed; …
exit=1
```

No occurrence of `ripemd160` anywhere in that text, so `assert!(out.contains("ripemd160"))` fails. (With the
flag corrected to `--hashlock-phrase-stdin`, clap reaches `parse_kind` and the message does name the tokens —
so the remedy exists; the plan simply never applies it to the test body.)

---

### I-2 — the call-site enumeration reported zero → **CLOSED**

Step 4 now states `cargo build` reports zero because `qr_text` has no production caller, and supplies
`grep -rn "qr_text(" --include=*.rs crates/ | grep -v "fn qr_text"`. Measured: 6 lines, of which one (`:31`) is
a doc comment, leaving the **five** real call sites the plan claims — `hashlock_qr_text.rs:54, 88, 115, 124, 126`.

### I-3 — three unmentioned assertions, one needing a decision → **CLOSED**, and the decision is **right**

Step 4b enumerates four failures. The decision (`the_worst_case_is_210_bytes`, re-keyed on `Ripemd160`, second
assertion 148) is confirmed by execution. Applying Task 4 Step 3's `qr_text` and measuring:

```
hardened, phrase = "0"*100:   sha256 207   hash256 208   ripemd160 210   hash160 208
method=sha256, same phrase:   sha256 148   hash256 149   ripemd160 151   hash160 149
lines().count() = 4 ;  lines().nth(1).len() = 73
```

- `ripemd160` (9 chars) really is the longest token — 210 is the true worst case, 207 is the sha256 case.
- `+13` for sha256 (`"\nhash: sha256"`), matching 194→207 and 135→148.
- Spec §13.1 line 591 records `194 → **210 bytes, still 53 modules**`. The fold's *"This is **not** a spec
  defect"* is **confirmed**: do not touch the spec.
- The "What does NOT break, verified" paragraph holds: the method line is still `nth(1)` and still 73 chars.

One miscount carried over from round 1: **two** corpus `note` strings cite byte counts
(`max-phrase-hardened` "194 bytes, ECC-L v9, 53 modules", `max-phrase-sha256` "135 bytes, ECC-L v7"), not three.
Nit, filed below.

### I-4 — `cargo vendor --versioned-dirs` renames 130 directories → **CLOSED**

With `ripemd = "0.1"` added and `cargo vendor` run into a scratch directory:

```
diff <(ls vendor) <(ls /scratch/.../vtest)
84a85
> ripemd
```

130 → 131, bare names, **exactly one added directory**, as the fold's expectation states.

### I-5 — `indent=1` reformats the corpus → **CLOSED** (expectation numbers wrong; see M-10)

Running the fold's generator verbatim on the real corpus:

```
added 66 digest columns across 11 rows
git diff --numstat -> 88   11   crates/ms-codec/tests/vectors/hashlock-v0.8.json
```

312 deletions → 11. The re-indent is gone and the review affordance is restored.

### I-6 — no corpus-SHA re-pin, no CHANGELOG, no version decision → **PARTIAL**, and the new steps are wrong in two ways

Task 2 Step 3 (SHA + CHANGELOG + 0.10.0), Step 4 (commit) and Task 6 Step 4 (both values into the phase-close
note) are all new and address the shape of the gap. Both of the concrete instructions fail:

1. **The recorded SHA is stale before the phase ends.** Step 3 runs `sha256sum` on the corpus at the end of
   Task 2. Measured, that value is `34310fece24bf8bb1fdf6ced39d1f5f624a561c13a780f6b1c2fc0a5b1a36165`. Task 4
   then rewrites the *same file* twice — Step 4b (every row's `qr_text`, `bytes` and two `note` strings) and
   Step 5b (a `kind` column plus one new row per kind). So `CHANGELOG.md` would carry a hash matching no
   committed state, and it is the hash phase 4 re-pins the fork's provenance file against. The re-pin has to
   happen after Task 4, not inside Task 2.
2. **The 0.10.0 bump does not resolve.** Filed as **C-4**.

### I-7 — no per-kind `qr_text` row, no `kind` column → **PARTIAL**

Step 5b exists and is the right instruction. It creates work no step performs:
`crates/ms-codec/tests/hashlock_qr_text.rs:17-24`'s `Row` has fields `name, method, phrase, qr_text, bytes` and
no `kind`; Step 4 tells the engineer to pass `HashKind::Sha256` at every call site, including `:54` inside
`qr_text_matches_every_corpus_row`. Add a `ripemd160` row per Step 5b and that loop calls
`qr_text(hardened, HashKind::Sha256, …)` against a ripemd160 expectation and fails. Nothing in the plan says to
add `kind` to `Row`, map it to a `HashKind`, or change the call. (`Row` is a plain `Deserialize` with no
`deny_unknown_fields`, so the *column* is tolerated — it is simply never read.) Filed as **I-13**.

### I-8 — wrong formatter → **CLOSED**

Task 6 Step 2 is `cargo +1.95.0 fmt --all -- --check` with the `rust.yml:60-63` rationale; Step 2b separates the
clippy commands so a fmt failure cannot skip them. Verified runnable: `~/.cargo/bin/cargo +1.95.0 fmt --all --
--check` exits 0 on the clean tree (see N-3 for the `PATH` caveat).

### I-9 — stdout purity broken behind an "Expected: PASS" → **PARTIAL**

The stdout half is fixed: nothing new goes to stdout, and the `for md compose:` line is correctly identified as
already living on stderr at `:406`. But the fold moved the four-digest fallback onto stderr **without checking
that stderr is pinned too**, and it is — in the same file round 1 cited. Simulated (the fallback's six lines
inserted at `:325`, unconditional, then `cargo test -p ms-cli --test hashlock_outputs`):

```
test json_both_variants ... FAILED
  hashlock_outputs.rs:189: "under --json --no-engraving-card, stderr must be exactly the
  PrivateKeyMaterial advisory (spec §4.4, §11)"
test the_card_names_the_preimage_on_its_first_line_and_carries_the_method_line ... FAILED
  hashlock_outputs.rs:63: the card's FIRST stderr line must contain "PREIMAGE"
test result: FAILED. 9 passed; 2 failed
```

`json_both_variants` asserts stderr **exactly equals** the advisory in two configurations (`:187-193` and
`:198-202`), and neither passes `--kind`, so `args.kind.is_none()` is true and the fallback fires. That failure
is independent of where the block is placed. The second failure is placement-dependent (it goes away if the
fallback prints after the card) and the plan does not say where the block goes.

Task 5 Step 5's "Expected: PASS" is still false. Filed as **I-12**.

### Minors claimed folded

- **M-1 — CLOSED.** The snippet now binds `d` as the `Derived` value and reads `d.x`.
- **M-2 — CLOSED in letter, defective in fact.** The `use` line is shown; it imports `DigestBytes`, which the
  plan's own code never names. See **I-11**.
- **M-3 — CLOSED.** Step 4 states the `HashKind` import for `hashlock_qr_text.rs`.
- **M-4 — CLOSED.** Step 4b says the test "does not print expected-vs-actual usefully" and tells the engineer to
  construct the new value by hand.

M-5…M-8 and N-1…N-3 were not claimed folded and remain open as recorded.

---

## New defects introduced or exposed by the fold

### C-3 (Critical) — nothing stages the Step 4b renames, and `crates/ms-cli/src/cmd/decode.rs` is staged by no task at all

Step 4b is new in this fold and touches five files. The plan's five `git add` lines are:

| line | task | stages |
| --- | --- | --- |
| `:293` | 1 | `crates/ms-codec/Cargo.toml crates/ms-codec/src/hashlock.rs Cargo.lock vendor` |
| `:395` | 2 | `…/hashlock-v0.8.json CHANGELOG.md crates/ms-codec/Cargo.toml` |
| `:521` | 3 | `crates/ms-codec/tests/hashlock_kat.rs crates/ms-codec/Cargo.toml Cargo.lock vendor` |
| `:669` | 4 | `crates/ms-codec/src/hashlock.rs crates/ms-codec/tests/ Cargo.lock vendor` |
| `:856` | 5 | `crates/ms-cli/src/cmd/hashlock.rs crates/ms-cli/tests/` |

Measured call sites needing the rename (`hashlock::digest` → `digest_sha256`), by file:

```
crates/ms-codec/src/hashlock.rs:59                     (definition)          -> staged, Task 1
crates/ms-codec/tests/hashlock_derivation.rs:7,49,52,114,117,134,150         -> staged, Task 4 only
crates/ms-codec/tests/hashlock_repro.rs:19,112                               -> staged, Task 4 only
crates/ms-cli/src/cmd/hashlock.rs:23,325                                     -> staged, Task 5 only
crates/ms-cli/tests/hashlock_phrase_rule.rs:63,235                           -> staged, Task 5 only
crates/ms-cli/src/cmd/decode.rs:207                                          -> STAGED BY NOTHING
```

Two consequences.

1. **Task 1's commit does not compile.** It carries `digest` renamed with eleven callers unrenamed, in both
   crates. Tasks 2 and 3 commit on top of it, so four commits in a row are non-building. The project's
   `git bisect`, and `test (rust + go)` on any of those SHAs, would be red.
2. **`decode.rs` is never committed.** The string `decode` does not appear anywhere in the plan. With the
   repo's explicit-paths rule (no `git add -A`), the engineer finishes Task 6 with `crates/ms-cli/src/cmd/decode.rs`
   still dirty; Task 6 has no clean-tree check; the `ci/staging` SHA and then `master` get a tree in which
   `cmd/decode.rs:207` calls a function that no longer exists. `cargo build -p ms-cli` fails on master.
   Task 6 Step 5 ("Push") therefore cannot be completed as written — the same shape as round 1's C-1.

### C-4 (Critical) — the `ms-codec` 0.10.0 bump makes the workspace unresolvable; no step touches `ms-cli`'s pin

`crates/ms-cli/Cargo.toml:20` is `ms-codec = { path = "../ms-codec", version = "=0.9.0" }` — an **exact** pin,
which round 1 quoted. Task 2 Step 3 says "this phase bumps `ms-codec` to **0.10.0**" and Step 4 stages
`crates/ms-codec/Cargo.toml`. Measured, with only that bump applied:

```
cargo metadata --format-version 1 --offline
error: failed to select a version for the requirement `ms-codec = "=0.9.0"`
candidate versions found which didn't match: 0.10.0
required by package `ms-cli v0.18.0`
exit=101
```

Every cargo invocation from that point fails, starting with Task 3 Step 1. The plan names no remedy and never
mentions `crates/ms-cli/Cargo.toml`.

Two further gaps in the same step, both measured: the bump changes `Cargo.lock` (`ms-codec version 0.9.0 →
0.10.0`, 1 line), and Task 2 Step 4 stages **neither** `Cargo.lock` nor `crates/ms-cli/Cargo.toml` — so even
with the pin fixed by hand, the commit is inconsistent and `--locked` commands (`vendor-freshness.sh` shells out
to `cargo metadata --offline --locked`; Task 6 Step 1 is `cargo nextest run --locked`) fail against it.

### I-10 (Important) — Task 1's Interfaces still promises the alias the fold removed

Plan `:55` vs `:218`. Detailed under **C-1** above. One-line deletion; it must not survive into execution,
because the half that contradicts the fix is the half an implementer treats as the contract.

### I-11 (Important) — the plan's own Step 3 import produces two unused imports, and unused imports are errors in the required CI job

Measured on the tree with Task 1 and the fold's Step 3 snippet applied:

```
warning: unused import: `DigestBytes`      (the plan's `use ms_codec::hashlock::{HashKind, DigestBytes};`)
warning: unused import: `digest_sha256`    (Task 1 Step 4b renamed it in; Task 5 removed its only use)
```

`DigestBytes` is never named in the fold's ms-cli code — `h.as_slice()` is an inherent method. And Task 5
replaces `let h = digest_sha256(&d.x)` with `kind.digest(&d.x)`, orphaning the import Step 4b just created;
nothing tells the engineer to drop it from the `use ms_codec::hashlock::{…}` list at `:22-25`. Both are errors
under `cargo clippy -p ms-cli --all-targets -- -D warnings`, which is the required CI context C-1 was about.
Task 6 Step 2b would catch them; Task 5 Step 5's "Expected: PASS" would not.

### I-12 (Important) — the four-digest fallback on stderr breaks two pinned stderr contracts

Measured, two failures in `hashlock_outputs.rs`. Detailed under **I-9**. The plan needs to decide the same
thing for stderr that it decided for stdout: suppress the fallback under `--json`, or under
`--no-engraving-card`, or emit it only where the card already goes — and say where in the function the block
sits, because the second failure depends on it.

### I-13 (Important) — Step 5b's per-kind corpus rows have no test that can read them

Detailed under **I-7**. `Row` needs a `kind` field and `:54` needs to pass it; neither is a step.

### I-14 (Important) — the plan's `parse_kind` does not compile in the file it is to be pasted into

Not fold-introduced — the block is unchanged context in the diff — but it blocks execution and round 1 did not
find it. `crates/ms-cli/src/cmd/hashlock.rs:31` is `use crate::error::{CliError, Result};`, and that `Result` is
a one-parameter alias. Measured:

```
error[E0107]: type alias takes 1 generic argument but 2 generic arguments were supplied
   --> crates/ms-cli/src/cmd/hashlock.rs:314:27
314 | fn parse_kind(s: &str) -> Result<HashKind, String> {
error[E0308]: mismatched types     (cascading, on the `other => Err(format!(…))` arm)
```

The signature must be `std::result::Result<HashKind, String>`. This is the exact class the repo's own
CLAUDE.md records from the encrypted-payload cycle ("mismatched `Result` types in a match") and the one
`scripts/plan-build-gate.sh` exists to catch — the ms-cli blocks arrive as fragments, so the gate does not see
them, which makes this a reviewer's execution pass by design. It costs one word.

---

## Minor / Nit

- **M-9 (Task 1 Step 4b).** The supplied grep misses every `use` line. Measured, it returns the six
  `hashlock_derivation.rs` call sites but **not** its import at `:7`, not `hashlock_repro.rs:19`, and not
  `crates/ms-cli/src/cmd/hashlock.rs:23` — all three are inside multi-line `use` blocks. Compiler-caught, but
  "Find them — do not guess" under-reports by three. After the change the same pattern also matches the new
  `pub fn digest(self, …)` dispatch method, a false positive.
- **M-10 (Task 2 Step 1).** "Expected: roughly **77 insertions, 0 deletions**" measures **88 insertions, 11
  deletions** — 77 genuinely new lines plus 11 one-per-row comma changes where `provenance_kinds` is appended
  after the previous last key. The check still separates the failure mode it exists for (11 vs 312), but "0
  deletions" is stated flatly and is wrong.
- **M-11 (Task 2 Step 1).** The I-5 fold left a dangling clause mid-sentence: *"…rather than committing the
  reformat. (some rows may carry only one stem; the count is whatever the assertion-guarded loop reports —
  record it)."* That parenthetical belonged to the superseded "Expected" line and now contradicts the
  "verified exact" two lines above it.
- **M-12 (Task 5 Step 3).** `hex::encode` is introduced into a file that already has
  `fn hex(b: &[u8]) -> String` at `:290`. It compiles (the crate lives in the type namespace, the fn in the
  value namespace, and `hex = "0.4"` is a dependency at `ms-cli/Cargo.toml:36`) but leaves two spellings of the
  same operation four lines apart.
- **N-4 (Task 4 Step 4b item 2).** "the three row `note` strings" is **two** — measured over the corpus's seven
  `qr_text` rows, only `max-phrase-hardened` and `max-phrase-sha256` cite byte counts. Inherited from round 1.
- **N-5 (Task 5 Step 3).** `kind: Option<HashKind>` is the only non-`pub` field in a `pub struct HashlockArgs`
  whose every sibling is `pub`. Harmless (`run()` is in the same module) but inconsistent.
- **N-6 (environment, Task 6 Step 2).** `cargo +1.95.0 …` requires the rustup shim. On this box
  `/usr/bin/cargo` is first on `PATH` and answers `error: no such command: +1.95.0`; `~/.cargo/bin/cargo`
  works. Worth one sentence in the step so the engineer does not read the failure as a missing toolchain.

---

## Counts

**4 Critical / 7 Important / 4 Minor / 3 Nit** (new + unclosed, this round).

- Critical: **C-2** (not closed — the record and `sha256_operand` lie about the kind), **C-3**, **C-4**, and
  **C-1 PARTIAL** (mechanism green, contract text contradicts it — counted here as Critical only because the
  surviving half is the one that reproduces round 1's C-1; if the `:55` line is deleted, C-1 is CLOSED and the
  count is 3C).
- Important: I-1 (partial), I-6 (partial), I-7 (partial), I-9 (partial), I-10, I-11, I-12, I-13, I-14 —
  deduplicating the partials that are fully described by a new item, the blocking set is
  **I-1, I-6, I-10, I-11, I-12, I-13, I-14**.
- Minor: M-9…M-12. Nit: N-4…N-6.

**Closed outright, verified by execution: I-2, I-3, I-4, I-5, I-8, M-1, M-3, M-4** — and the C-1 *mechanism*,
which is the round's best result: the two required clippy contexts are measured clean under the pinned 1.85.0
toolchain, against a measured-clean baseline, with the real CI commands.

## Verdict

**NOT GREEN.**

The fold did the hard technical work and got the measurable parts right — the alias removal is green under the
real CI commands, `cargo vendor vendor/` adds exactly one directory, `indent=2` stops the reformat, the flag is
the real one, and the `210 / 207 / 148 / ripemd160-is-longest` decision is correct in every digit and is
correctly judged *not* a spec defect.

What it did not do is finish. Three of the four Criticals are the fold's own new instructions arriving without
the rest of their consequences: a rename with no `git add` (C-3), a version bump that no second file follows
(C-4), and a `--kind` that reaches the digest but not the **grammar** of the record it is printed in — which
per spec §6 turns a `hash256` request into a bare record an old parser reads as `sha256`, the one scheme the
spec names as rejected because it composes an unspendable wallet (C-2). C-2 is not a relocation of round 1's
finding; on funds it is strictly worse than what round 1 measured.

The pattern under all of it is incomplete propagation, and it is visible in the artifact three separate times:
Task 1's Interfaces still promises the alias its Step 4 deletes; Task 5 Step 1 keeps both the old helper and
the old test bodies beside the new helper that replaces them; and the fold's own `use` line adds an import its
own code never uses. Every one is a grep for the superseded phrasing away.
