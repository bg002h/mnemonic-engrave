# R0 round 1 — IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md

**Artifact:** `design/IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md` (commit `97460adc`)
**Spec:** `design/SPEC_hashlock_kinds.md` (commit `ad1820a3`), phase 2 of §9
**Target repo:** `/scratch/code/shibboleth/mnemonic-secret` at `7a0e96f` (clean before and after this review)
**Question:** could a competent engineer with no context execute this plan task by task and end with correct working software, without inventing a decision the plan should have made?

**Method.** Task 1 and Task 3 were applied to the real tree and compiled/run; Task 4's signature
change was applied and the resulting failures observed; Task 2's generator was executed against the
real corpus; Task 5's flag surface and the existing stdout-purity tests were read and the refusal
path executed against the built `ms` binary. Every touched file was restored with `git checkout --`
and the corpus SHA re-verified as `4f1819cd…` (unchanged). Nothing was committed.

**Toolchain caveat for every measurement below.** `rust-toolchain.toml` pins 1.85.0; this box has no
rustup and ran `rustc 1.98.0`. That produces 10 clippy errors on the *unmodified* tree
(`manual_div_ceil` ×2, `mismatched_lifetime_syntaxes` ×2, `repeat().take()` ×9 — lints that do not
exist in 1.85). Every clippy measurement below is stated as a **delta against that measured
baseline**, not as an absolute count.

---

## Verification of the three pre-review machine-checks (all three hold)

1. **`deny(missing_docs)` including variants — CORRECT and correctly located.**
   `crates/ms-codec/src/lib.rs:39` is `#![cfg_attr(not(test), deny(missing_docs))]`. The plan's code
   as written (docs on all six variants, on `as_slice`, on `digest` and on `token`) built clean:
   `cargo build -p ms-codec` → 0 errors.

2. **No `use ripemd::Digest as _;` — CORRECT.** Confirmed by compiling the plan's Task 1 block
   verbatim into `crates/ms-codec/src/hashlock.rs`: 0 errors. The mechanism is that `sha2 0.10` and
   `ripemd 0.1.3` both resolve `digest 0.10.7` (`Cargo.lock:300`), so the `Digest` already imported at
   `hashlock.rs:20` covers `Ripemd160`. See M-6 for the fragility this rests on.

3. **The four digest values for `X = 0xab*32` — CORRECT, all four.** Reproduced independently:
   ```
   sha256    9a2db2e23f1504cd056606553ac049c5e718e8f9ce9233876df1a7a1821af885
   hash256   88b8f02ce56abce1d453e0610318130f4d0a13067549e804af1f5186f81a2691
   ripemd160 5786aabcae0e6cd2dfaeca2767dc8996c98f43f4
   hash160   e81bfa71da56f187cce1319ee773dabf56988e95
   ```
   The `sha256` row is byte-identical to the corpus's `kind[0].digest`, as the plan claims.

**Two further plan claims the brief flagged, both measured TRUE:**

- **Task 2 Step 1's `added 66 digest columns across 11 rows` is exactly right.** All 11 `derivation`
  rows carry *both* `hardened_x` and `sha256_x` (measured by dumping every row's key set), so no row
  is skipped and 11 × 2 × 3 = 66. The `assert row[f"{stem}_h"] == sha256(x)` guard passed on all 22
  stems — the generator's loop does what its prose says.
- **Task 3's `checked >= 8` is satisfiable.** `checked` reaches **11**: every row has
  `hardened_x` + `hardened_h` and gains the three new `hardened_h_*` columns, so the let-else never
  `continue`s. `cargo test -p ms-codec --test hashlock_kat` → `1 passed`, first try, no edits.

Task 1's three unit tests also pass as written (`4 passed` in the lib target, filter `hashlock::`).

---

## Critical

### C-1 — Task 1 Step 4 (+ Task 6 Step 2): the `#[deprecated]` alias turns two required CI clippy jobs red, and the plan's own gate cannot see it

**The concrete failure.** An engineer executes Task 1 Step 4, which adds
`#[deprecated] pub fn digest(...)`. `ms-codec`'s own test targets call it nine times
(`crates/ms-codec/tests/hashlock_derivation.rs` ×7 at `:49,:52,:114,:117,:134,:150` and its import
at `:7`; `crates/ms-codec/tests/hashlock_repro.rs` ×2 at `:19,:112`). `.github/workflows/rust.yml:142`
runs `cargo clippy -p ms-codec --all-targets -- -D warnings`, and the workflow's own header
(`:49-53`) states these contexts are **required status checks**.

Measured, holding the pre-existing baseline lints allowed so the delta is clean:

```
# clean tree
cargo clippy -p ms-codec --all-targets -- -D warnings \
  -A clippy::manual_div_ceil -A mismatched_lifetime_syntaxes
  -> 10 errors; test targets hashlock_derivation and hashlock_repro compile CLEAN

# after Task 1 Step 4
  -> +9 errors: "use of deprecated function `ms_codec::hashlock::digest`"
     could not compile (test "hashlock_derivation") due to 7 previous errors
     could not compile (test "hashlock_repro")      due to 2 previous errors
```

`deprecated` is a stable warn-by-default rustc lint, so this is toolchain-independent — it will fire
identically under CI's 1.85.0, where the baseline is otherwise green.

`ms-cli` is hit too: `crates/ms-cli/src/cmd/hashlock.rs:23,:325` and
`crates/ms-cli/src/cmd/decode.rs:207` in *production* code, plus
`crates/ms-cli/tests/hashlock_phrase_rule.rs:63,:235`. `.github/workflows/rust.yml:264` runs
`cargo clippy --all-targets -p ms-cli -- -D warnings`.

**Why the plan cannot catch it.** Task 6 Step 2 runs
`cargo clippy --all-targets --locked 2>&1 | grep -E "^(warning|error)" | head` — no `-D warnings`,
no `-p`, and truncated by `head`. Its "Expected: no NEW clippy warnings (record any pre-existing
ones)" is a judgment call against a baseline the plan never establishes, and the command does not
reproduce either CI job. So Task 6 Step 5 ("Push") is the step that cannot be completed as written:
the `ci/staging` SHA will never earn its contexts.

**What the plan must decide and does not.** The alias's stated purpose is "so phase 3's callers keep
compiling" — that is about `me-cli`, which pins by git rev and is a different repo. Nothing in
*this* repo needs the alias. The plan has to either (a) migrate all 13 in-repo call sites to
`digest_sha256` in Task 1 and keep the alias purely for the downstream rev-pin, or (b) say
explicitly that the deprecation is suppressed. It says neither, and the engineer must invent one.

---

### C-2 — Task 5 Steps 3-4: under `--kind`, every machine-readable channel still emits the **sha256** digest

**The concrete failure.** Task 5 Step 3's snippet computes `k.digest(&x)` into *new* printed lines
and leaves `crates/ms-cli/src/cmd/hashlock.rs:325` — `let h = digest(&d.x);` — untouched. `h` is what
feeds, measured at those lines:

| line | channel | emits |
| --- | --- | --- |
| `:324` → `:393` | **stdout**, `hash:{hex(h)}` — the record `me sysw pack` reads | sha256, always |
| `:365` | `--json` `"digest"` | sha256, always |
| `:366` | `--json` `"hash_record"` | sha256, always |
| `:367-370` | `--json` `"sha256_operand": "sha256={hex(h)}"` | sha256, always |
| `:405` | stderr card `digest:          {hex(h)}` | sha256, always |
| `:406` | stderr card `for md compose:  --path ... sha256={hex(h)}` | sha256, always |

Step 4 names exactly one of those six (`:406`). The other five are not mentioned anywhere in the plan.

So `ms hashlock --hashlock-phrase-stdin --kind hash256` prints **two different 64-hex values**: a new
`hash256: <correct sha256d>` line, and `digest: <sha256>` / stdout `hash:<sha256>` /
`"sha256_operand": "sha256=<sha256>"`. Neither has a width, a truncation, or any visual distance —
this is the pair spec §3 F4 calls out as having "no structural signal whatever", inside the very tool
§13.2's reconciliation screen instructs the operator to run. Under `--kind ripemd160` the tool
answers a question it was not asked: stdout carries a 64-hex sha256 record while the operator asked
for a 40-hex ripemd160 digest, and §6's grammar (`hash: [<kind>:] <hex>`) is never consulted.

Spec §13.4 is explicit that `ms hashlock` "must not silently assume sha256." The plan satisfies that
for the new human-readable lines and leaves it violated on every channel a machine or a copy-paste
reads.

**What the plan must decide and does not.** What does stdout's `hash:` record say under `--kind`?
`hash:ripemd160:<40hex>` per §6, or refuse, or stay sha256? What do `--json`'s `digest`,
`hash_record` and `sha256_operand` become? Is `sha256_operand` renamed, made kind-aware, or dropped?
`crates/ms-cli/tests/hashlock_outputs.rs:184` pins `sha256_operand` by name, so this is re-pin work
either way. None of it is in the plan.

**How verified.** Read `crates/ms-cli/src/cmd/hashlock.rs:324-326, 360-370, 393, 400-410` against
the plan's Task 5 Steps 3 and 4 line by line; confirmed by `grep -rn "sha256=" crates/ms-cli/src/`
returning exactly `:369` and `:406`, of which the plan names only `:406`.

---

## Important

### I-1 — Task 5 Step 1: the flag hedge points at the right file but gives the wrong remedy

**The actual flag is `--hashlock-phrase TEXT`** (`crates/ms-cli/src/cmd/hashlock.rs:45-47`), and it is
**refused by a pre-parser argv guard** unless `--allow-argv-secret` is also passed. The private
channel is a *separate boolean flag*, `--hashlock-phrase-stdin` (`:48-50`), not a bare stdin default —
the module header at `:9-11` says so explicitly: *"zero must not default to stdin"*, and
`pick_source` exits 64 on zero sources.

The plan's hedge says: *"If the phrase arrives on stdin, drop the `--phrase` pair from `args` and add
`.write_stdin(...)`."* Executed literally that yields `run_ms(&["hashlock"])` + stdin, which is the
**zero-source** case → exit 64 → `.assert().success()` panics. The hedge is therefore not adequate;
it should be replaced with the real thing, which this repo's own suite already uses verbatim
(`crates/ms-cli/tests/hashlock_outputs.rs:28-34, 42, 57`):

```rust
Command::cargo_bin("ms").unwrap()
    .args(["hashlock", "--hashlock-phrase-stdin", "--no-engraving-card"])
    .write_stdin("correct horse battery staple")
```

The third test is worse than a rename: `run_ms_expect_fail(&["hashlock", "--phrase", "x", "--kind", "RIPEMD160"])`
asserts the refusal names `ripemd160`, but the argv guard refuses **before clap parses**, so
`parse_kind` never runs and the message cannot contain the token list. Measured against the built
binary:

```
$ ms hashlock --phrase "correct horse battery staple"
ms: argument 3 on ARGV ... is a BIP-39 mnemonic, 28 characters long.
      Refused BEFORE the command line was parsed; ...
exit=1
```

The plan's own `argv_guard_cross_product.rs` reference is the right instinct; the hedge just stops
one step short of naming the consequence.

**Minor correction to a supporting claim:** the plan says `cli_help_pointer.rs`, `cli_derive_bip48.rs`
and `argv_guard_cross_product.rs` "all **open with** `use assert_cmd::Command;`". Measured, the import
is at `:9`, `:43` and `:63` respectively. All three do use it and do call `Command::cargo_bin("ms")`;
only "open with" is loose. Not a finding on its own.

---

### I-2 — Task 4 Step 4: the call-site enumeration command reports **zero** and the real call sites are invisible to it

**The concrete failure.** Step 4 says: *"Run `cargo build -p ms-codec -p ms-cli 2>&1 | grep -c "^error"`
and fix each call to pass `HashKind::Sha256` where the caller has no kind yet."*

Measured after applying Task 4 Step 3's signature change:

```
cargo build -p ms-codec -p ms-cli   ->  exit 0, "^error" count = 0
```

`qr_text` has **no production call site in this repo** — `grep -rn qr_text` over `crates/` returns
the definition plus `crates/ms-codec/tests/hashlock_qr_text.rs` and nothing else. `cargo build` does
not compile test targets, so the command tells the engineer there is nothing to fix. The five real
call sites only appear under a test build:

```
cargo test -p ms-codec --test hashlock_qr_text
  -> 5 × error[E0061]: this function takes 3 arguments but 2 arguments were supplied
```

(`hashlock_qr_text.rs:54, 88, 115, 124, 126`.) So the answer to the brief's question — *are the call
sites enumerable from the plan?* — is **no**, and the command the plan supplies actively says they
do not exist. `cargo build --all-targets` (or the `cargo test` the step's second half already runs)
is the command that enumerates them.

---

### I-3 — Task 4: three existing assertions the plan never mentions, one of which has no obvious right answer

Applied Task 4 Step 3, patched the five call sites to `HashKind::Sha256`, and ran the suite. Measured
failures, in the order they surface:

1. **`qr_text_matches_every_corpus_row` at `:55`** — `assert_eq!(*got, row.qr_text)`.
   Expected: the plan says to update the corpus rows. Accounted for.
2. **`the_worst_case_is_194_bytes` at `:125`** — `left: 207, right: 194`. The plan does not mention
   this test at all. Its second assertion (`qr_text(false, …) == 135`) becomes 148.
3. **`qr_text_matches_every_corpus_row` at `:62`** — after fixing the corpus strings:
   `"row anchor-hardened: the text is three LF-separated lines", left: 4, right: 3`. The plan does not
   mention `got.lines().count() == 3`.
4. **`assert_eq!(got.len(), row.bytes)` at `:56`** — each row's `bytes` column moves by 13. Step 4
   says update the rows *"only after confirming the difference is exactly the inserted `hash:` line"*
   and says nothing about `bytes`, nor about the three row `note` strings that cite
   `"194 bytes, ECC-L v9, 53 modules"` and `"135 bytes, ECC-L v7"` — both now stale.

Item 2 is the one that needs a **decision the plan should have made**. The test's *name* asserts 194.
After the change sha256 gives 207 and §13.1's measured worst case is **210** (ripemd160, and the plan's
own new test pins ≤ 210). So the engineer must choose: rename to `the_worst_case_is_210_bytes` and
re-key it on ripemd160, or keep it sha256-specific at 207/148, or delete it in favour of the new
budget test. Nothing in the plan chooses.

**Direct answer to the brief's §13.1 question:** the "its own line" placement *does* hold against the
two structural properties the corpus test cares most about — `parameters_come_from_the_constants`
still reads the method line at `lines().nth(1)` and still measures it at exactly 73 characters
(measured: **passes unchanged**), and `last.strip_prefix("phrase: ")` still holds because the phrase
stays last. What it does not hold against is the literal line *count*, the `bytes` column, and the
194/135 pin.

---

### I-4 — Task 1 Step 6: `cargo vendor --versioned-dirs` renames all 130 vendored directories, and the freshness gate cannot see it

The committed tree uses **bare** directory names (`vendor/hex`, `vendor/sha2`, `vendor/serde`;
130 entries). Measured, into a scratch directory so nothing was destroyed:

```
cargo vendor --versioned-dirs /scratch/.../vtest
  -> hex-0.4.3  hex-conservative-0.2.2  hex_lit-0.1.1  ripemd-0.1.3  serde-1.0.228  sha2-0.10.9
  -> 131 directories
```

Run as written (the default output directory *is* `vendor`), Task 1 Step 6 deletes 130 directories
and creates 131 — a ~103 MB rename churn inside the commit that is supposed to show one new
dependency. Worse, the gate the same step runs is blind to it: `ci/repro/vendor-freshness.sh` calls
`cargo metadata --offline --locked`, which reads `.cargo-checksum.json` and does not care about
directory naming, so it reports **OK** either way. The script's own error message names the correct
command: `cargo vendor vendor/`.

---

### I-5 — Task 2 Step 1: `indent=1` reformats the entire corpus

The committed file is `indent=2` (measured on `hashlock-v0.8.json:1-12`). Running the plan's
generator verbatim:

```
added 66 digest columns across 11 rows
git diff --numstat -> 389 insertions, 312 deletions
```

Only 77 of those 389 lines are real (66 digest columns + 11 `provenance_kinds`); the other 312 are a
whole-file re-indent. That destroys the one review affordance this change needs — a diff showing that
*only* digest columns were added to a funds-relevant vendored corpus — and it is a one-character fix
(`indent=2`). The trailing-newline handling (`open(p,"a").write("\n")`) is correct: the file ends
`}\n` today and `json.dump` writes none.

---

### I-6 — No task re-pins the corpus SHA, and two things downstream key on it

Task 2 moves `crates/ms-codec/tests/vectors/hashlock-v0.8.json`, whose SHA-256 is
`4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4` today (measured). That value is
carried in:

- `CHANGELOG.md:50-54`, under an explicit **"Per-release checklist item 1"** which states that a
  corpus-hash move is what forces the version bump — *"its hash moves and the pre-1.0
  breaking-change axis requires `0.X+1.0`"*;
- the fork's `hashlock/testdata/hashlock-v0.8.provenance.json`, which `CHANGELOG.md:54` and
  `design/FOLLOWUPS.md:64` both record as re-pinning to this hash;
- spec §11's gate row *"the cross-language digest KAT | `hashlock/testdata/hashlock-v0.8.json` + its
  ms-codec source"*.

The plan has no step that computes the new SHA, no CHANGELOG entry, and no phase-4 handoff note.
Task 6 Step 4 records `git rev-parse HEAD` for phase 3's rev pin but nothing for phase 4's corpus
pin. Task 4 also changes `qr_text`'s public signature — a breaking change on a `0.9.0` crate that
`crates/ms-cli/Cargo.toml:21` pins as `version = "=0.9.0"` — with no version decision stated either
way. The engineer has to invent the whole release-hygiene half of this phase.

---

### I-7 — Spec-coverage gap: no per-kind `qr_text` corpus row, and no `kind` column for phase 4 to key on

Spec §10 lists, among the vectors this cycle owes: *"**The preimage plate, per kind**: its QR text,
its locator row, and a layout fit assertion at the worst case."* The corpus is the cross-language pin
and it lives in **this** phase — `crates/ms-codec/tests/vectors/hashlock-v0.8.json` is vendored into
the fork as `hashlock/testdata/hashlock-v0.8.json` and asserted by
`seedhammer/hashlock/methodline_h6_test.go:29` (`TestH6MethodLineAndQRTextMatchTheMSCorpus`), whose
own header says it exists so the fork has *"something to assert against that is not a literal it
transcribed itself"* (the same sentence opens `crates/ms-codec/tests/hashlock_qr_text.rs`).

Task 4 hard-codes `HashKind::Sha256` for all seven existing rows and adds two hand-written Rust tests
for `Ripemd160`. The corpus therefore gains **no non-sha256 QR-text row and no `kind` column at all**.
When phase 4 widens Go's `QRText(hardened, phrase)` to take a kind, it has nothing to pass and nothing
to check a non-sha256 plate against — so it will pin against a literal it transcribed itself, which is
precisely the failure both test headers were written to prevent. Adding a `kind` field to the seven
rows plus (at minimum) one `ripemd160` worst-case row is phase-2 work by §9's own assignment
(*"the plate's QR text gains `hash: <kind>` — `qr_text` lives here"*).

---

### I-9 — Task 5 Step 3 prints to stdout, breaking the stdout-purity contract and its test; Step 5's "Expected: PASS" is false

Step 3's snippet uses `println!` for both the kind line and a `for md compose:` line — stdout. But
`crates/ms-cli/tests/hashlock_outputs.rs:22` (`stdout_is_exactly_the_record_under_out_and_under_sha256`,
module header at `:1`: *"stdout purity in the two configurations where a mutation can hide"*) asserts
stdout is **exactly** `hash:<hex>\n`, and `crates/ms-cli/src/cmd/hashlock.rs:5` states the contract:
*"stdout carries the PUBLIC digest record (`me sysw pack` reads it), stderr carries the SECRET
preimage on the card."* With no `--kind`, Step 3's fallback prints five lines to stdout; with one, two.
That test goes red, so Task 5 Step 5's `cargo test -p ms-cli` "Expected: PASS" is false.

Step 3 also contradicts Step 4 on the same line: the existing `for md compose:  --path ... sha256=<h>`
is on **stderr** at `:406`, and Step 4 correctly says to make *that* line kind-aware — while Step 3
adds a second one on stdout.

---

### I-8 — Task 6 Step 2: `cargo fmt --check` is not the gate CI runs, and the workflow says so in writing

CI's job is `cargo +1.95.0 fmt --all -- --check` (`.github/workflows/rust.yml:107`). The comment
directly above it (`:60-63`) states: *"The explicit `+1.95.0` is **REQUIRED** here — this repo's
`rust-toolchain.toml` pins 1.85.0, which a bare `cargo fmt` would otherwise use (and which formats
differently)."* Task 6 Step 2's bare `cargo fmt --check` is exactly the command that comment warns
against, and the same block records a commit (`de593ca`) that left master red by trusting it. Also,
the `&&` chaining means a fmt failure silently skips the clippy half of the step.

---

## Minor / Nit

- **M-1 (Task 5 Step 3).** The snippet reads `let d = k.digest(&x);` — `x` is not a binding in
  `run()` (the preimage is `d.x`), and `d` already names the `Derived` value. Mechanical, but the
  engineer must translate it.
- **M-2 (Task 5 Step 3).** `#[arg(long, value_parser = parse_kind)] kind: Option<HashKind>` requires
  `HashKind` imported into `crates/ms-cli/src/cmd/hashlock.rs`; the plan lists it under "Consumes"
  but never shows the `use` line.
- **M-3 (Task 4 Step 1).** The two appended tests reference `HashKind`, which
  `crates/ms-codec/tests/hashlock_qr_text.rs:9` does not import. One-line fix, compiler-caught.
- **M-4 (Task 4 Step 4).** *"Each failing row prints its expected and actual text"* is false:
  `assert_eq!` panics, so exactly **one** row prints per run (measured — the loop stopped at
  `anchor-hardened`). Seven rows means seven iterations unless the engineer converts the loop to
  collect failures.
- **M-5 (Tasks 2 + 3).** Task 2 generates 33 `sha256_h_{hash256,ripemd160,hash160}` values that no
  test reads — Task 3's `Row` deserializes only the `hardened_*` columns. Dead corpus columns in a
  funds-relevant vendored file are a liability; either assert them or do not emit them.
- **M-6 (Task 1 Step 3).** `ripemd = "0.1"` is the **correct** pin and the plan does not say why:
  `ripemd 0.2.0` moved to `digest 0.11` (measured via `cargo info ripemd@0.2.0`, features
  `alloc = [digest/alloc]`, rust-version 1.85) and its `Digest` would not unify with the `sha2 0.10`
  `Digest` already in scope — so `cargo add ripemd` (which picks 0.2.0) breaks verification item 2
  above with a confusing trait error. One sentence in the plan removes that trap.
- **M-7 (Task 1 Interfaces).** `HashKind` gains `digest` and `token` but not `digest_len()`, which
  spec §5 names as *"the only place a length is written."* `DigestBytes` carries the width instead,
  which is defensible for a crate-local type — but the plan should say it is deliberate, because §5
  reads as a data-model obligation.
- **M-8 (Tasks 1, 4).** Stale prose the plan does not schedule: `hashlock.rs:11-13` ("`digest` is
  SHA-256 of X"), `qr_text`'s doc comment "three labelled lines" (`:177`),
  `hashlock_qr_text.rs:1-7`'s header, and the three corpus `note` strings citing 194/135 bytes.
- **N-1 (Task 3).** There is no step that runs the KAT and sees it **red** — Task 2 has already
  landed the columns, so it is green the moment it is written (measured: green on first run, no
  edits). Step 4's mutation pass is the real gate and does cover it; the "Step 1: Write the failing
  test" heading is just inaccurate.
- **N-2 (Tasks 3, 4).** Step 5 / Step 6 stage `Cargo.lock vendor` when neither task re-vendors.
  Harmless here only because `hex 0.4.3` is already in `vendor/` via `ms-cli` (verified), so
  `vendor-freshness.sh` still passes — but it is staging by habit, not by need.
- **N-3 (Task 1 Step 2).** Expected failure text is loose: rustc says *"failed to resolve: use of
  undeclared type `HashKind`"*, and `DigestBytes` is also undeclared.

---

## Counts

**2 Critical / 9 Important / 8 Minor / 3 Nit.**

- Critical: C-1, C-2
- Important: I-1 … I-9
- Minor: M-1 … M-8
- Nit: N-1 … N-3

## Verdict

**NOT GREEN.**

The plan's *cryptographic core* is sound and was verified end to end — the four functions, the
dispatch, the doc-comment requirement, the ripemd import question, the four digest values, the
generator's 66/11 count and the KAT's `checked = 11` all hold exactly as written, and Tasks 1-3
produce green tests on first execution. What blocks it is everything at the edges: a deprecation that
reds two required CI jobs with no way for the plan's own gate to notice (C-1); a `--kind` flag that
changes what the operator reads and not what the tool emits, inside the one tool §13.2's
reconciliation screen depends on (C-2); a call-site enumeration command that reports zero (I-2);
three existing assertions Task 4 silently breaks, one with no obvious right answer (I-3); a
stdout-purity break behind an "Expected: PASS" (I-9); the wrong vendor flag (I-4), the wrong indent
(I-5), the wrong formatter (I-8); and no corpus SHA re-pin or per-kind QR vector for phase 4 to
build on (I-6, I-7).
