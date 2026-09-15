# Hashlock kinds — Phase 2 (mnemonic-secret) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give `ms-codec` a digest function for each of the four miniscript hash kinds, a named dispatch over them, a cross-language known-answer test that catches a wrong one, and a preimage-plate QR that names the kind — so that every later phase has a correct, independently-verified digest to build on.

**Architecture:** Four explicit digest functions (`digest_sha256`, `digest_hash256`, `digest_ripemd160`, `digest_hash160`) plus a crate-local `HashKind` enum whose `digest` method is the single named dispatch. The KAT exercises **both** the functions and the dispatch, with rows computed outside the implementation by `python3 hashlib`. `qr_text` gains the kind so a plate is self-describing. `ms hashlock` gains `--kind`.

**Tech Stack:** Rust, `sha2`, new `ripemd` dependency, `serde_json` (dev), `cargo nextest`.

**Spec:** `design/SPEC_hashlock_kinds.md` (mnemonic-engrave, commit `ad1820a3`). Phase 2 of §9. Read §3 (F1, F4), §5, §10, §13.1, §13.4 before starting.

## Global Constraints

**THIS PLAN WAS DERIVED BY BUILDING IT.** All six tasks were implemented for
real in a worktree of `mnemonic-secret` before this text was written — compiled,
tested, linted under `-D warnings`, and run. Every command, error, count and
output below is transcribed from that run, not predicted. Two earlier drafts
were written the other way round and each shipped Criticals that were compiler
errors (`E0308`, `E0107`, unused imports) and pinned contracts; that is why the
method changed. The working branch is `hashkinds-p2` in
`/scratch/code/shibboleth/ms-worktrees/hashkinds-p2`.

### Environment — read this first or lose an hour

**Put rustup's shim ahead of the system cargo:**

```bash
export PATH=/home/bcg/.cargo/bin:$PATH
cargo --version      # MUST print 1.85.0, the rust-toolchain.toml pin
```

Arch's `/usr/bin/cargo` is **1.98.0** and does not honour `rust-toolchain.toml`,
so `cargo +1.85.0` fails with *"no such command"* and a bare `cargo clippy`
reports **5 phantom errors** in each crate (`div_ceil`, lifetime elision,
`repeat().take()`) — newer lints that 1.85.0 never emits. Measured: with the
shim first, the baseline is **0 errors in both crates**. Without it you will
chase five defects that are not yours.

### Constraints

- **The preimage is always 32 bytes, for every kind** (spec §3 F1). Nothing here
  changes preimage derivation.
- **The four tags are four functions, not two widths** (spec §3 F4).
- **`hash256` is the dangerous one.** `sha256(x)` one word short of `sha256d(x)`
  type-checks, lowers cleanly, and Core agrees with the address. Only the KAT
  catches it.
- **KAT rows are computed OUTSIDE the implementation** — `python3 hashlib`,
  recorded in each row's `provenance`.
- **No type crosses a repo boundary** (spec §5). `HashKind` is local to `ms-codec`.
- **Preimages stay `Zeroizing`.** Digests are public.
- **`-D warnings` is a required CI context.** Both crates must measure 0.
- **`cargo build` is not enough — use `--all-targets`.** Measured: the lib and
  bin compiled clean while eleven test-file errors waited behind
  `--all-targets`, because test files carry their own `use` blocks.

### The transcript gate — run this before committing any fold to this plan

Every ```rust block here carries a `file=`/`mode=` header and is diffed against
the branch:

```bash
scripts/h2-plan-blocks-vs-tree.sh \
  design/IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md \
  /scratch/code/shibboleth/ms-worktrees/hashkinds-p2
```

Expected: **11 blocks checked, 0 FAIL.** `mode=whole` blocks must be byte-identical
to the named file; `mode=fragment` blocks must appear in it as an exact substring.

**This is not decoration.** R0 round 4 found a Critical that was purely a fold
artefact — Task 3 Step 2 was corrected to drop the `hex` dev-dependency while
Step 1 kept calling `hex::encode` five times, so the task no longer compiled and
Step 3 told the executor to run a test that could not build. No gate had been run
on that fold. On its first run this checker failed three more blocks, and in all
three the BRANCH was the side that had drifted — dropped doc comments and two
absent tests that four rounds of re-reading had not caught.

**What it does NOT cover**, and the run prints this itself: bash recipes,
illustrative snippets, the deliberate MUTATION block in Task 3 Step 4 (which must
*not* match the tree), and every prose claim — expected test names, mutation
outcomes, byte counts, spec references. Those still need a reader.

### The verified behaviour this phase produces

Transcribed from the built binary, phrase `correct horse battery staple`:

```
--kind sha256      hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
--kind hash256     hash:hash256:98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
--kind ripemd160   hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b
--kind hash160     hash:hash160:b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd
(no --kind)        hash:3cf5d421…b70a4c12      <- bare, unchanged, backward compatible
```

Bare for sha256 and explicit for the other three is **spec §6's producer rule**,
and getting it wrong is the funds defect §6 names: a bare record MEANS sha256, so
emitting one under `--kind hash256` hands `me sysw pack` a sha256d digest it
reads as sha256. The `sha256` value above matches the corpus's existing
`hardened_h` for that phrase; the `hash256` value matches what the journey-walk
review computed independently.

### The tests are the task, not the trimming

**Read this before Task 5.** The branch this plan transcribes was first built
WITHOUT the tests these tasks prescribe, and it shipped **two Criticals behind a
568-green suite**:

- the engraving card printed `for md compose:  --path ... sha256=<digest>` under
  every `--kind`, telling the operator to compose a `sha256=` operand out of a
  `ripemd160` digest;
- the no-`--kind` path silently assumed sha256 — which §13.4 forbids — while the
  flag's own `--help` text claimed every kind's digest was listed.

Both survived because **`--kind` had zero test coverage**: no test passed the
flag or named `hash:hash256:`. The same shape sat next door in the corpus, and
was mutation-proven — deleting all three new per-kind `qr_text` rows left the
suite green, because the row floor was still `>= 7` for 10 rows.

A green suite is only evidence about what it tests. The tests below are
`crates/ms-cli/tests/hashlock_kind.rs` on the branch, and each one went RED
before its fix:

| test | what it would have caught |
| --- | --- |
| `the_record_follows_the_producer_rule_for_every_kind` | a non-bare record under sha256, or a missing `hash:<kind>:` prefix |
| `the_md_compose_line_names_the_chosen_kind` | **C-1** — the card's `sha256=` operand |
| `without_a_kind_every_digest_is_listed_on_stderr` | **C-2** — the silent sha256 assumption, on BOTH argv shapes |
| `an_uppercase_kind_is_refused` | case folded instead of rejected |
| `under_json_the_object_carries_every_kind_when_none_was_named` | §13.4 silently dropped when it changed channel into `--json` |
| `under_json_with_the_card_the_human_still_gets_the_listing` | the `--json`-with-card cell, where the human reads `sha256=` off the card |

**Six, and the last three were each added by a review round that found the
defect first.** That is the measurement, not a rhetorical point: the file went
4 → 5 → 6 as rounds 4 and 5 each found a path the existing tests could not see.
Every one was a flag combination nobody had passed.

And in `hashlock_qr_text.rs`, a floor is not coverage — assert that a row exists
**for each of the four kinds by name**, not that the array is at least N long.

### Measured outcomes — the numbers an executor should expect

Transcribed from the completed branch, not predicted:

| gate | result |
| --- | --- |
| test suites | **101 ok, 0 failed** |
| `cargo nextest run --locked --all-targets` | **578 run, 578 passed**, 11 skipped |
| `clippy -p ms-codec --all-targets -- -D warnings` | **0** |
| `clippy -p ms-cli --all-targets -- -D warnings` | **0** |
| `cargo fmt --all -- --check` | clean |
| `cargo vendor vendor/` | exactly **one** added directory (`vendor/ripemd`) |
| `ci/repro/vendor-freshness.sh` | OK |
| corpus `derivation` rows / `qr_text` rows | 11 / **10** (7 existing + 3 new per-kind) |
| corpus SHA-256 after Task 4 | `0a911f78f3cdc867dcc44483b7f4c0c1ac87b6d9b30b79f52094e8979bc3d8ce` |

**Task 2's diff is 88 insertions / 11 deletions**, not the "77 / 0" an earlier
draft predicted: adding a key to each row also rewrites that row's previously
final line to gain a comma. Eleven rows, eleven such lines. A `0` deletion count
is the wrong thing to check for.

**The corpus changes TWICE** — digest columns in Task 2, `qr_text` rows in Task 4
— so the SHA above is the one after Task 4. Re-pin from the final state, not
from Task 2.

### Four more traps, all found by running

4. **`qr_text`'s corpus test cannot read the new rows** until it is taught to.
   The loop hardcoded `HashKind::Sha256`, so per-kind rows silently compared
   against a sha256 render. `Row` gains a `kind: String` field and the loop maps
   it. Without this, Step 5b's rows exist and gate nothing.
5. **The four broken `qr_text` assertions surface ONE AT A TIME**, not all at
   once — the corpus-row test aborts at its first failing row, so `bytes`, the
   line count, and the worst case only appear as you fix forward. Expect four
   rounds, not one.
6. **`serde_json` is already a dev-dependency; `hex` is not needed.** An earlier
   draft added `hex = "0.4"`. Write a local `hex()` helper instead — and write it
   as a `fold`, because `map(format!).collect()` trips clippy's `format_collect`
   under `-D warnings`.
7. **The line count is a multi-line assertion.** `got.lines().count(), 3` spans
   two source lines, so a single-line `sed` misses it. It becomes 4.

### Three traps the real build found, which no reviewer would have

1. **Do NOT blanket-rename `digest` with a regex.** It hits **JSON keys, output
   labels and assertion messages** — measured collateral in four files, including
   `k0["digest"]` (a corpus field) and `"digest:"` (an output label). Rename call
   sites only, and re-check every string literal afterwards:
   **two** greps, because one is not enough — the first inspects string
   literals, and the rename also lands in English prose:

   ```bash
   grep -rn '"[^"]*digest_sha256' --include=*.rs crates/          # string literals
   grep -rn "digest_sha256" --include=*.rs crates/ | grep -E ":\s*(///|//)"   # comments
   ```

   Both must come back empty. The literal-only guard was in an earlier draft and
   passed while `crates/ms-cli/tests/hashlock_emit_record.rs:57` read *"carries only the public
   digest_sha256"* in a doc comment.
2. **`ms hashlock`'s flag count is gated.** `--kind` moves the total from 68 to
   **69** and reds
   `gui_schema_emits_spec_v7_json.rs::the_schema_names_every_flag_p2_added_and_the_total_is_67`.
   Its message is the obligation: *"a flag reached the binary and not the schema
   … the GUI's mirror would be describing a different program."* Update the
   count AND the flag enumeration in its message. (The test's own name says 67
   and was already stale at 68 — leave that wart or fix it deliberately.)
3. **Renaming the `--json` key `sha256_operand` → `hash_operand` is a breaking
   change to a machine-readable contract**, pinned by
   `hashlock_outputs.rs::json_both_variants`. It is the right rename — the key is
   wrong for three of four kinds — but it is a GUI-facing contract and belongs in
   the CHANGELOG with the version bump, not folded in silently.

---

## File Structure

| file | responsibility |
| --- | --- |
| `crates/ms-codec/src/hashlock.rs` | the four digest functions, `HashKind`, the dispatch, `qr_text` |
| `crates/ms-codec/Cargo.toml` | the `ripemd` dependency |
| `crates/ms-codec/tests/vectors/hashlock-v0.8.json` | the KAT rows (8 new digest columns) |
| `crates/ms-codec/tests/hashlock_kat.rs` | **new** — the per-kind KAT, covering functions *and* dispatch |
| `crates/ms-codec/tests/hashlock_qr_text.rs` | corpus-driven `qr_text` assertions |
| `crates/ms-cli/src/cmd/hashlock.rs` | `--kind`, the four-digest fallback, `method_line` |

---

### Task 1: The four digest functions and the named dispatch

**Files:**
- Modify: `crates/ms-codec/Cargo.toml` (add `ripemd`)
- Modify: `crates/ms-codec/src/hashlock.rs:59` (`digest`)
- Test: `crates/ms-codec/src/hashlock.rs` (the existing `mod tests`)

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces:
  - `pub enum HashKind { Sha256, Hash256, Ripemd160, Hash160 }`
  - `pub fn digest_sha256(preimage: &[u8; 32]) -> [u8; 32]`
  - `pub fn digest_hash256(preimage: &[u8; 32]) -> [u8; 32]`
  - `pub fn digest_ripemd160(preimage: &[u8; 32]) -> [u8; 20]`
  - `pub fn digest_hash160(preimage: &[u8; 32]) -> [u8; 20]`
  - `pub fn HashKind::digest(self, preimage: &[u8; 32]) -> DigestBytes` where `pub enum DigestBytes { B32([u8; 32]), B20([u8; 20]) }`
  - `pub fn HashKind::token(self) -> &'static str` returning `"sha256"`, `"hash256"`, `"ripemd160"`, `"hash160"`
  - **No `digest` alias.** The old name is RENAMED to `digest_sha256` and this
    repo's call sites move with it (Step 4b). A `#[deprecated]` shim adds nine
    clippy errors here and four in `ms-cli` under `-D warnings`, measured.

- [ ] **Step 1: Write the failing test**

Append to `mod tests` in `crates/ms-codec/src/hashlock.rs`:

```rust file=crates/ms-codec/src/hashlock.rs mode=fragment
    /// The four functions are FOUR FUNCTIONS (spec §3 F4), and the two that
    /// share a width are the pair with no structural signal — so they are
    /// asserted to DIFFER, not merely to compute. The KAT in
    /// `tests/hashlock_kat.rs` pins the VALUES against `python3 hashlib`;
    /// this pins the STRUCTURE, and needs no corpus to do it.
    #[test]
    fn the_four_digests_are_four_different_functions() {
        let x = [0xabu8; 32];
        let s = digest_sha256(&x);
        let d = digest_hash256(&x);
        let r = digest_ripemd160(&x);
        let h = digest_hash160(&x);
        assert_ne!(
            s, d,
            "hash256 is sha256d, not sha256 — one word short is the whole defect"
        );
        assert_ne!(
            r, h,
            "hash160 is ripemd160(sha256(x)); ripemd160 is the bare primitive"
        );
        assert_eq!(s.len(), 32);
        assert_eq!(d.len(), 32);
        assert_eq!(r.len(), 20);
        assert_eq!(h.len(), 20);
    }

    /// The dispatch is the thing every caller uses, so it is tested as such:
    /// four correct functions behind one mis-wired arm is the same lost-funds
    /// outcome with a different cause.
    #[test]
    fn the_dispatch_selects_the_matching_function() {
        let x = [0x11u8; 32];
        assert_eq!(
            HashKind::Sha256.digest(&x),
            DigestBytes::B32(digest_sha256(&x))
        );
        assert_eq!(
            HashKind::Hash256.digest(&x),
            DigestBytes::B32(digest_hash256(&x))
        );
        assert_eq!(
            HashKind::Ripemd160.digest(&x),
            DigestBytes::B20(digest_ripemd160(&x))
        );
        assert_eq!(
            HashKind::Hash160.digest(&x),
            DigestBytes::B20(digest_hash160(&x))
        );
    }

    /// The tokens are the miniscript fragment names, lowercase. They are the
    /// operand name on the engraving card's `for md compose:` line and the
    /// `hash:<kind>:` tag in the record, so a typo here composes a wallet
    /// nobody can spend.
    #[test]
    fn tokens_are_the_lowercase_fragment_names() {
        assert_eq!(HashKind::Sha256.token(), "sha256");
        assert_eq!(HashKind::Hash256.token(), "hash256");
        assert_eq!(HashKind::Ripemd160.token(), "ripemd160");
        assert_eq!(HashKind::Hash160.token(), "hash160");
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p ms-codec hashlock:: 2>&1 | tail -20`
Expected: FAIL to compile — `cannot find function digest_sha256`, `cannot find type HashKind`.

- [ ] **Step 3: Add the dependency**

In `crates/ms-codec/Cargo.toml`, under `[dependencies]`, after the `sha2` line:

```toml file=crates/ms-codec/Cargo.toml mode=fragment
# ripemd160: the bare primitive for the `ripemd160` fragment. `hash160` is
# ripemd160(sha256(x)) and needs the same crate. RustCrypto, to match sha2.
ripemd = "0.1"
```

- [ ] **Step 4: Write the implementation**

In `crates/ms-codec/src/hashlock.rs`, replace the existing `digest` (line 59) with:

```rust file=crates/ms-codec/src/hashlock.rs mode=fragment
/// Which hash the SCRIPT commits to. Crate-local by design: the spec forbids a
/// shared type across repo boundaries (§5), so every consumer defines its own
/// and maps onto these functions.
///
/// NOT the same axis as the preimage METHOD (`preimage_hardened` vs
/// `preimage_sha256`). The two share the token `sha256` and mean different
/// things; four separate reviews of this cycle each found a defect caused by
/// that collision. Where both could be read, name both or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashKind {
    /// `sha256(X)`.
    Sha256,
    /// `sha256d(X)` = `sha256(sha256(X))`.
    Hash256,
    /// `ripemd160(X)`, the bare primitive.
    Ripemd160,
    /// `hash160(X)` = `ripemd160(sha256(X))`.
    Hash160,
}

/// A digest and its width. The width is a CONSEQUENCE of the kind, never a
/// separate thing to keep in sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigestBytes {
    /// A 32-byte digest: `sha256` or `hash256`.
    B32([u8; 32]),
    /// A 20-byte digest: `ripemd160` or `hash160`.
    B20([u8; 20]),
}

impl DigestBytes {
    /// The digest bytes, at their kind's width.
    pub fn as_slice(&self) -> &[u8] {
        match self {
            DigestBytes::B32(b) => &b[..],
            DigestBytes::B20(b) => &b[..],
        }
    }
}

impl HashKind {
    /// THE ONE NAMED DISPATCH in this crate. Spec §10 requires the KAT to
    /// exercise the dispatch and not only the four functions, because four
    /// correct functions plus one mis-wired arm is the same lost-funds outcome
    /// with a different cause.
    pub fn digest(self, preimage: &[u8; 32]) -> DigestBytes {
        match self {
            HashKind::Sha256 => DigestBytes::B32(digest_sha256(preimage)),
            HashKind::Hash256 => DigestBytes::B32(digest_hash256(preimage)),
            HashKind::Ripemd160 => DigestBytes::B20(digest_ripemd160(preimage)),
            HashKind::Hash160 => DigestBytes::B20(digest_hash160(preimage)),
        }
    }

    /// The lowercase miniscript fragment name. Case is rejected, never folded.
    pub fn token(self) -> &'static str {
        match self {
            HashKind::Sha256 => "sha256",
            HashKind::Hash256 => "hash256",
            HashKind::Ripemd160 => "ripemd160",
            HashKind::Hash160 => "hash160",
        }
    }
}

/// H = SHA-256(X): what a sha256 policy carries and the plate shows. Public.
pub fn digest_sha256(preimage: &[u8; 32]) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(&Sha256::digest(preimage));
    h
}

/// H = SHA-256(SHA-256(X)) -- `sha256d`. THE DANGEROUS ONE: written one word
/// short as `sha256(x)` it is still 32 bytes, still type-checks, still lowers,
/// and Core still agrees with the address. Only the KAT catches it.
pub fn digest_hash256(preimage: &[u8; 32]) -> [u8; 32] {
    let once = Sha256::digest(preimage);
    let mut h = [0u8; 32];
    h.copy_from_slice(&Sha256::digest(once));
    h
}

/// H = RIPEMD-160(X) -- the BARE primitive, not hash160.
pub fn digest_ripemd160(preimage: &[u8; 32]) -> [u8; 20] {
    use ripemd::Ripemd160;
    let mut h = [0u8; 20];
    h.copy_from_slice(&Ripemd160::digest(preimage));
    h
}

/// H = RIPEMD-160(SHA-256(X)) -- `hash160`. Same width as ripemd160 and a
/// different preimage relation (spec §3 F4).
pub fn digest_hash160(preimage: &[u8; 32]) -> [u8; 20] {
    use ripemd::Ripemd160;
    let inner = Sha256::digest(preimage);
    let mut h = [0u8; 20];
    h.copy_from_slice(&Ripemd160::digest(inner));
    h
}

// NO `digest` ALIAS. An earlier draft kept the old name as a `#[deprecated]`
// shim "so phase 3 keeps compiling". Every internal call site then becomes a
// deprecation warning, and `-D warnings` is a REQUIRED CI context. Measured by
// building that counterfactual (2026-09-15): `clippy -p ms-codec
// --all-targets` reports 14, `-p ms-cli --all-targets` 4. Without
// `--all-targets` ms-codec reports 1, which is why a gate that drops the flag
// does not see this at all. (An earlier record said "nine"; it did not
// reproduce under any flag combination, and this file's own new unit tests are
// two of the 14.)
//
// `digest` is renamed to `digest_sha256` and its call sites in THIS repo move
// with it. Phase 3 (`me-cli`) is a different repo pinned to a git rev, so it
// does not break until it chooses to bump.
```

**No extra import is needed** — settled by compiling this, not by reasoning:
`sha2::Digest` is already in scope and `Ripemd160` implements the same
`digest::Digest` trait, so `Ripemd160::digest(...)` resolves as written. Do not
add `use ripemd::Digest as _;`.

**THE CRATE DENIES MISSING DOCS, AND IT COUNTS VARIANTS AND METHODS**
(`crates/ms-codec/src/lib.rs:39`). Every public enum, *every variant*, and every
public method needs a `///`. The doc comments above are not decoration; omit one
and the build fails with `error: missing documentation for a variant`. This was
found by compiling the plan's own code before review.

- [ ] **Step 4b: Rename the call sites in this repo**

`digest` is gone, so every caller in `mnemonic-secret` moves to `digest_sha256`.
Find them — do not guess:

```bash
grep -rn "hashlock::digest\b\|[^_]\bdigest(" --include=*.rs crates/ | grep -v "Sha256::digest\|Ripemd160::digest"
```

Expect `crates/ms-cli/src/cmd/hashlock.rs` among them (see Task 5 C-2 for the
one at `:325`, which needs more than a rename).

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p ms-codec hashlock:: 2>&1 | tail -20`
Expected: PASS, including the three new tests.

**The expected values, already verified against `python3 hashlib` before this
plan was written** — for `X = 0xab * 32`:

```
sha256     9a2db2e23f1504cd056606553ac049c5e718e8f9ce9233876df1a7a1821af885
hash256    88b8f02ce56abce1d453e0610318130f4d0a13067549e804af1f5186f81a2691
ripemd160  5786aabcae0e6cd2dfaeca2767dc8996c98f43f4
hash160    e81bfa71da56f187cce1319ee773dabf56988e95
```

The `sha256` row matches the corpus's existing digest for that preimage, so the
new functions are consistent with what is already vendored. If your build
disagrees with any row, stop: the implementation is wrong, not the row.

- [ ] **Step 6: Re-vendor and verify the gate**

Run:
```bash
cargo vendor vendor/ > /dev/null && ci/repro/vendor-freshness.sh
```
Expected: the freshness script reports the vendor tree satisfies `Cargo.lock`,
and `git status` shows **one added directory** (`vendor/ripemd`), not a rename of
everything.

**NOT `--versioned-dirs`.** The committed tree uses bare names
(`vendor/sha2`, 130 entries); `--versioned-dirs` writes `sha2-0.10.9` and so
deletes 130 directories and creates 131 — ~103 MB of rename churn inside a commit
meant to show one new dependency. **And the gate cannot see it**:
`vendor-freshness.sh` shells out to `cargo metadata --offline --locked`, which
reads `.cargo-checksum.json` and does not care about directory naming, so it
reports OK either way. The script's own error message names the correct form.

- [ ] **Step 7: Commit**

```bash
git add crates/ms-codec/Cargo.toml crates/ms-codec/src/hashlock.rs Cargo.lock vendor
git commit -m "hashlock: a digest function per kind, and one named dispatch"
```

---

### Task 2: The KAT rows, computed outside the implementation

**Files:**
- Modify: `crates/ms-codec/tests/vectors/hashlock-v0.8.json` (11 `derivation` rows)

**Interfaces:**
- Consumes: `HashKind` and the four functions from Task 1 (for Task 3, not for generating rows).
- Produces: each `derivation` row gains six columns — `hardened_h_hash256`, `hardened_h_ripemd160`, `hardened_h_hash160`, `sha256_h_hash256`, `sha256_h_ripemd160`, `sha256_h_hash160` — alongside the existing `hardened_h` and `sha256_h`, which stay sha256 and unchanged.

- [ ] **Step 1: Generate the rows with python3, never with the crate**

Run from the repo root:

```bash
python3 - <<'PY'
import json, hashlib
p = "crates/ms-codec/tests/vectors/hashlock-v0.8.json"
d = json.load(open(p))
def ripemd160(b):
    h = hashlib.new("ripemd160"); h.update(b); return h.digest()
added = 0
for row in d["derivation"]:
    for stem in ("hardened", "sha256"):
        xhex = row.get(f"{stem}_x")
        if not xhex:
            continue
        x = bytes.fromhex(xhex)
        assert len(x) == 32, f"{stem}_x is not 32 bytes"
        row[f"{stem}_h_hash256"]   = hashlib.sha256(hashlib.sha256(x).digest()).hexdigest()
        row[f"{stem}_h_ripemd160"] = ripemd160(x).hex()
        row[f"{stem}_h_hash160"]   = ripemd160(hashlib.sha256(x).digest()).hex()
        # the existing *_h stays sha256; assert it rather than rewrite it
        assert row[f"{stem}_h"] == hashlib.sha256(x).hexdigest(), \
            f"{stem}_h is not sha256(X) — the corpus changed meaning"
        added += 3
    row["provenance_kinds"] = (
        "python3 hashlib: h_hash256=sha256(sha256(X)); "
        "h_ripemd160=ripemd160(X); h_hash160=ripemd160(sha256(X)). "
        "Computed OUTSIDE ms-codec: a row generated by the code it tests is not a KAT."
    )
# indent=2 — the committed file's format. indent=1 re-indents the WHOLE file:
# measured, 389 insertions / 312 deletions for 77 real lines, which destroys the
# one review affordance this change needs (a diff showing that ONLY digest
# columns were added to a funds-relevant vendored corpus).
json.dump(d, open(p, "w"), indent=2)
open(p, "a").write("\n")   # json.dump writes none; the file ends "}\n" today
print("added", added, "digest columns across", len(d["derivation"]), "rows")
PY
```

Expected: `added 66 digest columns across 11 rows` — verified exact.

Then check the diff is only additions:

```bash
git diff --numstat crates/ms-codec/tests/vectors/hashlock-v0.8.json
```
Expected: **88 insertions, 11 deletions** — measured. The 11 are not a reformat:
adding a key to each row rewrites that row's previously-final line to gain a
comma, once per row. A deletion count much above 11 means the indent is wrong;
a count of 0 is not the goal and never was. (some rows may carry only one stem; the count is whatever the assertion-guarded loop reports — record it).

- [ ] **Step 2: Verify `openssl` agrees, so the KAT has two independent sources**

Run:
```bash
python3 -c "
import json;d=json.load(open('crates/ms-codec/tests/vectors/hashlock-v0.8.json'))
r=d['derivation'][0];print(r['hardened_x']);print('rmd160',r['hardened_h_ripemd160'])"
echo -n "$(python3 -c "
import json;d=json.load(open('crates/ms-codec/tests/vectors/hashlock-v0.8.json'));print(d['derivation'][0]['hardened_x'])")" \
  | xxd -r -p | openssl dgst -ripemd160
```
Expected: the `openssl` output matches the `rmd160` value printed above.

- [ ] **Step 3: Re-pin the corpus hash, and say so in the CHANGELOG**

The corpus SHA-256 is `4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4`
today, and **three things key on it**: `mnemonic-secret/CHANGELOG.md`'s per-release checklist, the fork's
`hashlock/testdata/hashlock-v0.8.provenance.json`, and spec §11's gate row.

```bash
sha256sum crates/ms-codec/tests/vectors/hashlock-v0.8.json
```

Record the new value in `CHANGELOG.md`. Per-release checklist item 1 in that file
states that a corpus-hash move is what **forces the version bump** — *"its hash
moves and the pre-1.0 breaking-change axis requires `0.X+1.0`"* — and Task 4
independently changes `qr_text`'s public signature, which is breaking on a 0.9.0
crate. So this phase bumps `ms-codec` to **0.10.0**.

Carry the new SHA into the phase-close note: **phase 4 re-pins the fork's copy
against it**, and without that hand-off the fork's provenance check has nothing
to move to.

**This step is the `ms-codec` HALF ONLY.** The `ms-cli` bump, the exact-version
pin that tracks it, the lockfile and `MIGRATION.md` are **Task 5b**, after the
CLI change exists. Do not stop here: bumping `ms-codec` alone leaves
`crates/ms-cli/Cargo.toml`'s `version = "=0.9.0"` pointing at a version that no
longer exists in the workspace, and the very next `cargo` invocation fails to
resolve.

- [ ] **Step 4: Commit**

```bash
git add crates/ms-codec/tests/vectors/hashlock-v0.8.json CHANGELOG.md crates/ms-codec/Cargo.toml
git commit -m "hashlock: KAT rows for the three new kinds, computed in python3"
```

---

### Task 3: The KAT, covering the functions AND the dispatch

**Files:**
- Create: `crates/ms-codec/tests/hashlock_kat.rs`

**Interfaces:**
- Consumes: `HashKind`, `DigestBytes`, the four functions (Task 1); the corpus columns (Task 2).
- Produces: nothing later tasks consume.

- [ ] **Step 1: Write the failing test**

Create `crates/ms-codec/tests/hashlock_kat.rs`. **This is the file, entire** —
including its two local helpers, which is the point of Step 2 below:

```rust file=crates/ms-codec/tests/hashlock_kat.rs mode=whole
//! The per-kind known-answer test (spec §10, §12 item 6).
//!
//! WHY THIS FILE EXISTS. Core's address vectors derive from the descriptor TEXT
//! and are structurally blind to a wrong digest function; `ms hashlock` checked
//! against this crate is self-consistency, not a KAT. Without these rows, a
//! `digest_hash256` written one word short passes every other gate in the cycle
//! and locks funds to a preimage that does not exist.
//!
//! IT COVERS THE DISPATCH TOO. Four correct functions plus one mis-wired arm of
//! `HashKind::digest` is the same failure with a different cause.

use ms_codec::hashlock::{
    digest_hash160, digest_hash256, digest_ripemd160, digest_sha256, HashKind,
};

const CORPUS: &str = include_str!("vectors/hashlock-v0.8.json");

fn hex(b: &[u8]) -> String {
    // `fold`, not `map(format!).collect()` -- clippy's format_collect fires
    // under `-D warnings`, which is a required CI context.
    b.iter()
        .fold(String::with_capacity(b.len() * 2), |mut acc, x| {
            use core::fmt::Write as _;
            let _ = write!(acc, "{x:02x}");
            acc
        })
}

fn hex32(s: &str) -> [u8; 32] {
    let v: Vec<u8> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex"))
        .collect();
    v.try_into().expect("32 bytes")
}

#[test]
fn every_row_pins_all_four_kinds_and_the_dispatch() {
    let v: serde_json::Value = serde_json::from_str(CORPUS).expect("corpus parses");
    let rows = v["derivation"].as_array().expect("derivation rows");
    let mut checked = 0usize;
    for r in rows {
        for stem in ["hardened", "sha256"] {
            let Some(xh) = r[format!("{stem}_x")].as_str() else {
                continue;
            };
            let x = hex32(xh);
            let want = |k: &str| -> String {
                r[format!("{stem}_h{k}")]
                    .as_str()
                    .unwrap_or_else(|| panic!("row missing {stem}_h{k}"))
                    .to_string()
            };
            let phrase = r["phrase"].as_str().unwrap_or("<no phrase>");

            assert_eq!(hex(&digest_sha256(&x)), want(""), "{phrase}/{stem}: sha256");
            assert_eq!(
                hex(&digest_hash256(&x)),
                want("_hash256"),
                "{phrase}/{stem}: hash256"
            );
            assert_eq!(
                hex(&digest_ripemd160(&x)),
                want("_ripemd160"),
                "{phrase}/{stem}: ripemd160"
            );
            assert_eq!(
                hex(&digest_hash160(&x)),
                want("_hash160"),
                "{phrase}/{stem}: hash160"
            );

            // THE DISPATCH, over the same rows.
            for (kind, suffix) in [
                (HashKind::Sha256, ""),
                (HashKind::Hash256, "_hash256"),
                (HashKind::Ripemd160, "_ripemd160"),
                (HashKind::Hash160, "_hash160"),
            ] {
                assert_eq!(
                    hex(kind.digest(&x).as_slice()),
                    want(suffix),
                    "{phrase}/{stem}: dispatch for {}",
                    kind.token()
                );
            }
            checked += 1;
        }
    }
    // A loop that silently iterated zero rows would pass.
    assert!(
        checked >= 8,
        "only {checked} row/stem pairs carried all four kinds"
    );
}
```

- [ ] **Step 2: No new dev-dependency — the `hex` helper is IN the file above**

`ms-codec` has **no `hex` dependency of any kind**, and it is not getting one:
`[dev-dependencies]` is `proptest, bip39, serde, serde_json, zeroize`, and the
whole reason `validate_phrase`'s 64-hex check was rewritten as
`b.is_ascii_hexdigit()` in v0.9 was to keep the `hex` crate out of a codec that
does not otherwise need it (MIGRATION.md v0.8 → v0.9). So the file above defines
its own `hex`, **written as a `fold`** — `map(format!).collect()` trips clippy's
`format_collect` under `-D warnings`, which is a required CI context.

An earlier revision of this plan prescribed `hex::encode` here in Step 1 while
Step 2 told the executor not to add the dependency. Extracted and compiled, that
block produced **five `E0433`s**. It is the failure class the build gate exists
for, and it is why every block in this plan now carries a `file=`/`mode=` header
and is diffed against the branch by
`scripts/h2-plan-blocks-vs-tree.sh <this plan> <ms-worktree>`. Run that before
committing any fold.

- [ ] **Step 3: Run the test to verify it passes**

Run: `cargo test -p ms-codec --test hashlock_kat 2>&1 | tail -10`
Expected: PASS, with `checked` at or above 8.

- [ ] **Step 4: Prove the KAT can fail — mutate, observe, revert**

Break `digest_hash256` in `src/hashlock.rs` by removing the second hash:


**This block is a MUTATION and is deliberately absent from the tree**, so it
carries no `file=` header and the transcript checker lists it as uncovered. That
is correct: it is the one block here that must NOT match the branch.
```rust
pub fn digest_hash256(preimage: &[u8; 32]) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(&Sha256::digest(preimage));   // MUTATION: one word short
    h
}
```

Run: `cargo test -p ms-codec --test hashlock_kat 2>&1 | tail -10`
Expected: FAIL on `hash256`. **Then revert the mutation** and re-run to confirm PASS.

Repeat for the dispatch: swap `HashKind::Ripemd160`'s arm to call `digest_hash160`. Expected: FAIL on `dispatch for ripemd160`. Revert.

- [ ] **Step 5: Commit**

```bash
git add crates/ms-codec/tests/hashlock_kat.rs crates/ms-codec/Cargo.toml Cargo.lock vendor
git commit -m "hashlock: the per-kind KAT, functions and dispatch, mutation-verified"
```

---

### Task 4: `qr_text` names the kind

**Files:**
- Modify: `crates/ms-codec/src/hashlock.rs:208` (`qr_text`)
- Modify: `crates/ms-codec/tests/hashlock_qr_text.rs`
- Modify: `crates/ms-codec/tests/vectors/hashlock-v0.8.json` (`qr_text` rows)

**Interfaces:**
- Consumes: `HashKind` (Task 1).
- Produces: `pub fn qr_text(hardened: bool, kind: HashKind, phrase: &str) -> Zeroizing<String>`.

**Read first:** spec §13.1. The QR change is measured free — the worst case goes 194 → 210 bytes and stays at **53 modules**. The `method:` line must NOT grow (H6 §6.5 pins it at 73 chars); the kind is its own line.

- [ ] **Step 1: Write the failing test**

Append to `crates/ms-codec/tests/hashlock_qr_text.rs`:

```rust file=crates/ms-codec/tests/hashlock_qr_text.rs mode=fragment
/// Spec §13.1: the plate is read years later by someone with neither the tool
/// nor this firmware, so it spells its parameters out. Without the kind it is
/// one step short, and the device tells the operator to store this plate APART
/// from the md1 card that holds the missing step.
///
/// THE TWO AXES, in one assertion each. `hash:` is WHICH HASH THE SCRIPT
/// COMMITS TO; `method:` is HOW THE PREIMAGE WAS DERIVED. They share the token
/// `sha256` and mean different things, and every defect this cycle produced
/// came from reading one as the other -- so this test pins that adding the
/// first did not disturb the second.
#[test]
fn qr_text_names_the_kind_on_its_own_line() {
    let t = qr_text(true, HashKind::Ripemd160, "correct horse battery staple");
    assert!(
        t.contains("\nhash: ripemd160\n"),
        "the kind is not on its own line:\n{}",
        &*t
    );
    assert!(
        t.contains("method: pbkdf2-hmac-sha256"),
        "the METHOD line must survive unchanged -- it is a different axis:\n{}",
        &*t
    );
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p ms-codec --test hashlock_qr_text 2>&1 | tail -15`
Expected: FAIL to compile — `qr_text` takes 2 arguments.

- [ ] **Step 3: Change the signature and add the line**

In `crates/ms-codec/src/hashlock.rs`, change `qr_text` to:

```rust file=crates/ms-codec/src/hashlock.rs mode=fragment
pub fn qr_text(hardened: bool, kind: HashKind, phrase: &str) -> Zeroizing<String> {
    const HEAD: &str = "hashlock v1\n";
    const LABEL: &str = "\nphrase: ";
    let method = if hardened {
        format!(
            "method: pbkdf2-hmac-sha256 iterations={HASHLOCK_ITERATIONS} salt={} dklen={HASHLOCK_DKLEN}",
            core::str::from_utf8(HASHLOCK_SALT).expect("the salt is ASCII"),
        )
    } else {
        "method: sha256".to_string()
    };
    // ITS OWN LINE, never appended to `method:` -- H6 §6.5 pins that line at 73
    // characters and the plate refuses an eleventh row at every font rung.
    let kind_line = format!("\nhash: {}", kind.token());
    let mut out: Zeroizing<String> = Zeroizing::new(String::with_capacity(
        HEAD.len() + method.len() + kind_line.len() + LABEL.len() + phrase.len(),
    ));
    out.push_str(HEAD);
    out.push_str(&method);
    out.push_str(&kind_line);
    out.push_str(LABEL);
    out.push_str(phrase);
    out
}
```

- [ ] **Step 4: Update the call sites — all of which are TESTS**

`cargo build` reports **zero** errors here, and that is not good news: `qr_text`
has **no production caller**. Its five call sites are all test code, so a build
check is silent and only the test suite finds them:

```bash
grep -rn "qr_text(" --include=*.rs crates/ | grep -v "fn qr_text"
```

Pass `HashKind::Sha256` at each, preserving today's behaviour.

M-3: `crates/ms-codec/tests/hashlock_qr_text.rs` does not import `HashKind` —
add it to the `use ms_codec::hashlock::{…}` line or the new tests will not compile.

- [ ] **Step 4b: The four assertions this change breaks**

Run `cargo test -p ms-codec --test hashlock_qr_text` and expect **four** distinct
failures. Three are mechanical; one needs a decision, which this plan makes.

1. **`qr_text_matches_every_corpus_row` (`:55`)** — each row's `qr_text` string.
   Update it. The diff must be exactly one inserted `hash: <kind>` line;
   **the test does not print expected-vs-actual usefully**, so produce the new
   value by inserting that line yourself rather than copying from output.
2. **`assert_eq!(got.len(), row.bytes)` (`:56`)** — every row's `bytes` column
   moves by **13** for `sha256`. Update the column, and the three row `note`
   strings that cite `"194 bytes, ECC-L v9, 53 modules"` and `"135 bytes,
   ECC-L v7"` — both stale after this.
3. **`got.lines().count() == 3` (`:62`)** — now 4. Update the count and the
   message that says "three LF-separated lines".
4. **`the_worst_case_is_194_bytes` (`:125`)** — **the decision.**

   Rename it **`the_worst_case_is_210_bytes`** and re-key it on
   **`HashKind::Ripemd160`**. Reason: `ripemd160` is the longest of the four
   tokens (9 characters against 7, 7, 6), so the sha256 case is no longer the
   worst case and a test named for the worst case must track the real one.
   It ends with **four** assertions, not two, because both the hardened and the
   unhardened row move and the pair is what makes the ranking legible. All four
   measured on the branch:

   | call | bytes |
   | --- | --- |
   | `qr_text(true,  Ripemd160, "0"×100)` | **210** — the true worst case |
   | `qr_text(false, Ripemd160, "0"×100)` | **151** (was 135; `\nhash: ripemd160` is 16) |
   | `qr_text(true,  Sha256,    "0"×100)` | **207** (was 194; `\nhash: sha256` is 13) |
   | `qr_text(false, Sha256,    "0"×100)` | **148** (was 135) |

   The sha256 rows are kept beside the ripemd160 ones because they are the
   common case, not because they bound anything — and 210 − 207 = 3 is the
   whole margin, which is why the test is re-keyed rather than re-numbered.

   This is **not** a spec defect: §13.1's 210 is the true worst case; 207 is the
   sha256 case. Do not "correct" the spec.

**What does NOT break, verified:** `parameters_come_from_the_constants` still
reads the method line at `lines().nth(1)` and still measures it at exactly 73
characters, and `last.strip_prefix("phrase: ")` still holds because the phrase
stays last. The "own line" placement was chosen to preserve both.

- [ ] **Step 5: Run to verify all pass**

Run: `cargo test -p ms-codec 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 5b: Add a per-kind `qr_text` corpus row, so phase 4 has something to key on**

The corpus's `qr_text` rows carry no `kind` column, so after this change the fork
(phase 4) has nothing to assert its Go port against for the three new kinds —
spec §10 requires cross-repo agreement to be pinned by vectors, not by reading.

Add a `kind` field to each existing `qr_text` row (`"sha256"`, preserving
today's meaning) and **one new row per new kind**, with its `bytes` and `note`
computed the same way. That row set is phase 4's acceptance target.

- [ ] **Step 6: Commit**

```bash
git add crates/ms-codec/src/hashlock.rs crates/ms-codec/tests/ Cargo.lock vendor
git commit -m "hashlock: the plate QR names the kind, on its own line"
```

---

### Task 5: `ms hashlock --kind`

**Files:**
- Modify: `crates/ms-cli/src/cmd/hashlock.rs` (the arg struct, `method_line:299`, the output)

**Interfaces:**
- Consumes: `HashKind`, `HashKind::token`, `HashKind::digest` (Task 1).
- Produces: the `--kind` flag; the no-kind fallback.

**Read first:** spec §13.4 — `--kind` is **required** by the spec; the four-digest fallback is **permitted**; silently assuming sha256 is **forbidden**.

- [ ] **Step 1: Write the failing test**

Create `crates/ms-cli/tests/hashlock_kind.rs`. **This is the file, entire** —
its helper included, because the helper is where the two things an executor
gets wrong live:

**1. The phrase goes on STDIN, not in argv.** `--phrase` DOES NOT EXIST, and
`--hashlock-phrase` is refused by the **argv guard before clap parses it** —
this repo does not take secrets on the command line. The flag is
`--hashlock-phrase-stdin`. An earlier revision of this plan prescribed
`--phrase` in every test here and then retracted it in prose *below the block*,
which left two mutually exclusive prescriptions for one file with the wrong one
under the step an executor works from. The retraction is now the only version.

**2. The helper returns stdout AND stderr separately**, because the §13.4
fallback is on stderr under the stdout-purity contract, and the record is on
stdout. A helper that joins them cannot express the purity assertions this file
makes.

```rust file=crates/ms-cli/tests/hashlock_kind.rs mode=whole
//! `ms hashlock --kind`: the operator surface for the three new hash kinds
//! (SPEC_hashlock_kinds §6, §13.4).
//!
//! WHY THIS FILE EXISTS. The flag shipped once with ZERO coverage and two
//! Criticals rode in behind a 568-green suite: the card told the operator to
//! compose a `sha256=` operand out of a `ripemd160` digest, and the no-`--kind`
//! path silently assumed sha256 while its own `--help` claimed otherwise. A
//! green suite is only evidence about what it tests.

use assert_cmd::Command;

const PHRASE: &str = "correct horse battery staple";

fn run(args: &[&str]) -> (String, String) {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// §6's producer rule: BARE for sha256, `hash:<kind>:<hex>` for the rest, hex
/// at `digest_len()*2`. A bare record MEANS sha256, so emitting one under
/// another kind hands `me sysw pack` a digest it reads as sha256 — the scheme
/// §6 names as rejected because it composes an unspendable wallet.
#[test]
fn the_record_follows_the_producer_rule_for_every_kind() {
    let (so, _) = run(&["hashlock", "--hashlock-phrase-stdin"]);
    assert!(
        so.trim().starts_with("hash:") && so.trim().matches(':').count() == 1,
        "no --kind must still emit the BARE sha256 record: {so:?}"
    );
    assert_eq!(so.trim().len(), "hash:".len() + 64, "bare record is 64 hex");

    for (kind, hexlen) in [
        ("sha256", 64),
        ("hash256", 64),
        ("ripemd160", 40),
        ("hash160", 40),
    ] {
        let (so, _) = run(&["hashlock", "--hashlock-phrase-stdin", "--kind", kind]);
        let rec = so.trim();
        if kind == "sha256" {
            assert!(
                !rec.starts_with("hash:sha256:"),
                "--kind sha256 must emit the BARE form, for byte-identical compatibility: {rec:?}"
            );
            assert_eq!(rec.len(), "hash:".len() + hexlen);
        } else {
            let want = format!("hash:{kind}:");
            assert!(rec.starts_with(&want), "{kind}: want {want:?}, got {rec:?}");
            assert_eq!(rec.len(), want.len() + hexlen, "{kind}: hex length");
        }
    }
}

/// §11: this line is the ONLY thing keeping the digest function and `md
/// compose`'s option name in agreement. Under `--kind ripemd160` it must say
/// `ripemd160=`, or it instructs the operator to build a wallet whose hashlock
/// nobody can satisfy.
#[test]
fn the_md_compose_line_names_the_chosen_kind() {
    for kind in ["hash256", "ripemd160", "hash160"] {
        let (_, se) = run(&["hashlock", "--hashlock-phrase-stdin", "--kind", kind]);
        let line = se
            .lines()
            .find(|l| l.contains("for md compose:"))
            .unwrap_or_else(|| panic!("{kind}: no `for md compose:` line on the card"));
        assert!(
            line.contains(&format!("{kind}=")),
            "{kind}: the card says {line:?} — an operator following it composes the wrong wallet"
        );
        assert!(
            !line.contains("sha256="),
            "{kind}: the card still offers a sha256= operand: {line:?}"
        );
    }
}

/// §13.4: never silently assume sha256. A plate cut before this existed carries
/// no kind, so all four are listed and the operator matches.
///
/// BOTH argv shapes, because the first version of this test checked only the
/// default one and the fallback had been nested inside the `--no-engraving-card`
/// guard — so the flag that suppresses the card also suppressed the §13.4
/// notice, and this test passed anyway (R0 round 4, C-1). §13.4's clause is
/// unconditional; the operator who hides the card is the one who most needs the
/// lookup, because the card is what carries the preimage.
#[test]
fn without_a_kind_every_digest_is_listed_on_stderr() {
    for argv in [
        &["hashlock", "--hashlock-phrase-stdin"][..],
        &["hashlock", "--hashlock-phrase-stdin", "--no-engraving-card"][..],
    ] {
        let (so, se) = run(argv);
        for (kind, hexlen) in [
            ("sha256", 64),
            ("hash256", 64),
            ("ripemd160", 40),
            ("hash160", 40),
        ] {
            // The fallback's own line shape: `  <token padded to 10> <hex>`.
            // Matching the DIGEST and its WIDTH, not just the word: stderr
            // already says "sha256" twice without the loop running at all --
            // on the `for md compose:` line and in this block's own header
            // ("stdout carries the sha256 record") -- so a bare `contains`
            // would report coverage of a sha256 row that was never printed.
            // Width is what separates the 20-byte kinds from the 32-byte ones.
            let found = se.lines().any(|l| {
                let Some(rest) = l.strip_prefix("  ") else {
                    return false;
                };
                let Some(rest) = rest.strip_prefix(kind) else {
                    return false;
                };
                let hexpart = rest.trim_start();
                hexpart.len() == hexlen && hexpart.bytes().all(|b| b.is_ascii_hexdigit())
            });
            assert!(
                found,
                "{argv:?}: stderr carries no {kind} digest line; §13.4 forbids \
                 assuming sha256 in silence.\n{se}"
            );
        }
        assert!(
            !so.contains("hash256"),
            "the fallback must not reach stdout — purity contract: {so:?}"
        );
    }
}

/// Case is rejected, never folded (§6).
#[test]
fn an_uppercase_kind_is_refused() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["hashlock", "--hashlock-phrase-stdin", "--kind", "RIPEMD160"])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "RIPEMD160 must be refused, not folded"
    );
}

/// §13.4 on the MACHINE channel. The stderr listing cannot be used under
/// `--json`: `--json --no-engraving-card` pins stderr to exactly the
/// PrivateKeyMaterial advisory (§4.4, §11), a contract `hashlock_outputs.rs`
/// enforces. So the notice travels in the object instead — and this test is
/// what stops that from being a silent drop rather than a change of channel.
#[test]
fn under_json_the_object_carries_every_kind_when_none_was_named() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "hashlock",
            "--hashlock-phrase-stdin",
            "--json",
            "--no-engraving-card",
        ])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();

    assert_eq!(
        v["kind_specified"], false,
        "no --kind was given; the object must say so rather than let \
         `hash_operand`'s sha256 default read as a stated choice"
    );
    let by = v["digests_by_kind"]
        .as_object()
        .expect("digests_by_kind must be an object");
    for (kind, hexlen) in [
        ("sha256", 64),
        ("hash256", 64),
        ("ripemd160", 40),
        ("hash160", 40),
    ] {
        let d = by[kind]
            .as_str()
            .unwrap_or_else(|| panic!("no digests_by_kind.{kind}"));
        assert_eq!(d.len(), hexlen, "{kind}: digest width");
        assert!(d.bytes().all(|b| b.is_ascii_hexdigit()), "{kind}: not hex");
    }
    assert_eq!(by["sha256"], v["digest"], "sha256 row must match `digest`");

    // The purity contract the stderr listing was moved OUT of the way for.
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("no --kind given"),
        "the human listing must not reach stderr under --json"
    );

    // With an explicit kind there is nothing to disambiguate.
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "hashlock",
            "--hashlock-phrase-stdin",
            "--json",
            "--kind",
            "ripemd160",
        ])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["kind"], "ripemd160");
    assert!(v.get("digests_by_kind").is_none(), "{v}");
    assert!(v.get("kind_specified").is_none(), "{v}");
}

/// The cell between the other two: `--json` WITHOUT `--no-engraving-card`.
///
/// This is the normal way to use `--json` — `ms hashlock … --json > out.json` —
/// and it is the one the stderr purity contract does NOT cover, because that
/// contract is on the two flags TOGETHER. The card still prints to the
/// terminal here, so the human is reading stderr, and the `for md compose:`
/// line they are reading says `sha256=`. Without the §13.4 listing beside it
/// they are shown one unlabelled sha256 operand and nothing saying a kind was
/// never chosen — while the machine-readable notice sits in the redirected
/// file they are not looking at.
///
/// The guard was `!args.json` for one round and this cell was untested, which
/// is how it got through (R0 round 5, I-1).
#[test]
fn under_json_with_the_card_the_human_still_gets_the_listing() {
    let (_, se) = run(&["hashlock", "--hashlock-phrase-stdin", "--json"]);
    assert!(
        se.contains("for md compose:"),
        "precondition: the card must still be on stderr here, or this test is \
         asserting nothing:\n{se}"
    );
    for (kind, hexlen) in [
        ("sha256", 64),
        ("hash256", 64),
        ("ripemd160", 40),
        ("hash160", 40),
    ] {
        let found = se.lines().any(|l| {
            let Some(rest) = l.strip_prefix("  ") else {
                return false;
            };
            let Some(rest) = rest.strip_prefix(kind) else {
                return false;
            };
            let hexpart = rest.trim_start();
            hexpart.len() == hexlen && hexpart.bytes().all(|b| b.is_ascii_hexdigit())
        });
        assert!(
            found,
            "--json with the card: stderr carries no {kind} digest line, so the \
             human reading `for md compose: sha256=` has nothing telling them a \
             kind was never chosen.\n{se}"
        );
    }
}
```

**Six tests, not three**, and the last two are a matrix, not decoration.
`hashlock_outputs.rs` pins stderr under `--json --no-engraving-card` — BOTH
flags — to exactly the `PrivateKeyMaterial` advisory (§4.4, §11). That splits
the kind-omitted case into three cells, and §13.4 must be satisfied in every
one:

| argv | where §13.4's notice goes |
| --- | --- |
| no `--json` | the stderr listing (card suppressed or not) |
| `--json --no-engraving-card` | the OBJECT: `kind_specified`, `digests_by_kind` |
| `--json`, card on | the stderr listing — the human is reading the card |

`under_json_the_object_carries_every_kind_when_none_was_named` covers row 2 and
`under_json_with_the_card_the_human_still_gets_the_listing` covers row 3. Row 3
was missed for a round because the guard was written `!args.json`, broader than
the contract it cited; the human was then shown the card's `for md compose:
sha256=` line with no notice anywhere, the notice having gone into the
redirected file. See Step 3.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p ms-cli --test hashlock_kind 2>&1 | tail -15`
Expected: FAIL — `--kind` is not a recognised argument.

- [ ] **Step 3: Implement the flag**

Add to the hashlock args struct:

```rust file=crates/ms-cli/src/cmd/hashlock.rs mode=fragment
    /// Which hash the SCRIPT commits to: sha256, hash256, ripemd160, hash160.
    /// Omit it and every kind's digest is listed on stderr (under --json, in
    /// the object as `digests_by_kind`, with `kind_specified: false`) -- a
    /// plate cut before this existed carries no kind, and a lookup beats an
    /// impossible check. Case is rejected, never folded.
    #[arg(long, value_name = "KIND", value_parser = parse_kind)]
    pub kind: Option<HashKind>,
```

```rust file=crates/ms-cli/src/cmd/hashlock.rs mode=fragment
fn parse_kind(s: &str) -> core::result::Result<HashKind, String> {
    match s {
        "sha256" => Ok(HashKind::Sha256),
        "hash256" => Ok(HashKind::Hash256),
        "ripemd160" => Ok(HashKind::Ripemd160),
        "hash160" => Ok(HashKind::Hash160),
        other => Err(format!(
            "unknown hash kind {other:?}: expected sha256, hash256, ripemd160 or hash160 \
             (lowercase; case is rejected, never folded)"
        )),
    }
}
```

Note the return type is `core::result::Result`, spelled out: this module has its
own `Result` alias in scope and a bare `Result<HashKind, String>` does not
compile against clap's `value_parser`.

**THE ONE LINE THAT MATTERS IS THE DIGEST SOURCE.** An earlier draft added new
output and left `let h = digest(&d.x);` alone — so under `--kind` **six**
channels would still have emitted the sha256 digest: the stdout `hash:` record,
three `--json` keys, and two stderr card lines. Under `--kind hash256` the tool
would print two indistinguishable 64-hex values: the spec's §13.2
operator-facing Critical, reproduced inside the very tool the reconciliation
screen sends the operator to.

So the change is at the source, not at the print sites:

```rust file=crates/ms-cli/src/cmd/hashlock.rs mode=fragment
    let kind = args.kind.unwrap_or(HashKind::Sha256);
    let hb = kind.digest(&d.x);
    let h = hb.as_slice();
```

`hb` is bound separately because `as_slice()` borrows it — `kind.digest(&d.x).as_slice()`
on one line does not outlive the statement. Every existing consumer of `h` is
then correct without further edits. Walk them and confirm — do not assume:

```bash
grep -n 'h_hex\|"digest"\|sha256' crates/ms-cli/src/cmd/hashlock.rs
```

**STDOUT PURITY IS A CONTRACT, AND IT IS TESTED.** `crates/ms-cli/src/cmd/hashlock.rs:5`:
*"stdout carries the PUBLIC digest record (`me sysw pack` reads it), stderr
carries the SECRET preimage on the card."*
`crates/ms-cli/tests/hashlock_outputs.rs:22` asserts stdout is **exactly**
`hash:<hex>\n`. So nothing new goes to stdout — not the kind line, not the
four-digest fallback, not the `for md compose:` line, which already exists on
**stderr** and is made kind-aware there (Step 4).

**THERE ARE TWO CHANNELS FOR §13.4, AND THE SECOND IS NOT OPTIONAL.**
`hashlock_outputs.rs` ALSO pins stderr under `--json --no-engraving-card` to
**exactly** the `PrivateKeyMaterial` advisory (§4.4, §11). So the stderr
fallback must stand down under `--json`, and the notice travels in the object
instead:

```rust file=crates/ms-cli/src/cmd/hashlock.rs mode=fragment
        o.insert("kind".into(), kind.token().into());
        if args.kind.is_none() {
            // SPEC §13.4 for MACHINE consumers. The stderr listing below is
            // suppressed under --json because `--json --no-engraving-card`
            // pins stderr to exactly the advisory (§4.4, §11), so the notice
            // has to travel in the object or it does not travel at all. A
            // consumer reading `hash_operand` alone would otherwise take the
            // sha256 default for a stated choice.
            o.insert("kind_specified".into(), false.into());
            let mut by = serde_json::Map::new();
            for k in [
                HashKind::Sha256,
                HashKind::Hash256,
                HashKind::Ripemd160,
                HashKind::Hash160,
            ] {
                by.insert(k.token().into(), hex(k.digest(&d.x).as_slice()).into());
            }
            o.insert("digests_by_kind".into(), by.into());
        }
```

The human fallback prints to **stderr**, and note both guards — it is
`args.kind.is_none() && !(args.json && args.no_engraving_card)` — both flags,
matching the contract exactly and not one flag wider — and it sits **OUTSIDE** the
`!args.no_engraving_card` block:

```rust file=crates/ms-cli/src/cmd/hashlock.rs mode=fragment
    if args.kind.is_none() && !(args.json && args.no_engraving_card) {
        // SPEC §13.4: a plate cut before --kind existed carries no kind, so
        // listing all four turns an impossible check into a lookup. What is
        // forbidden is assuming sha256 in silence -- which is what this did.
        //
        // OUTSIDE the --no-engraving-card guard ON PURPOSE. §13.4's clause is
        // unconditional, and the operator who suppresses the card is the one
        // who most needs this: the card carries the PREIMAGE, so suppressing
        // it is the safety-conscious choice, and nesting this inside it hands
        // that operator one unlabelled sha256 digest instead. It shipped that
        // way once (R0 round 4, C-1). Do not fold it back in.
        //
        // The guard is `!(json && no_engraving_card)` -- BOTH flags -- because
        // that pair, and only that pair, is what `hashlock_outputs.rs` pins to
        // exactly the PrivateKeyMaterial advisory (§4.4, §11). There the notice
        // changes channel instead, into `kind_specified` and `digests_by_kind`
        // on the object.
        //
        // It was `!args.json` alone for one round (R0 round 5, I-1), which is
        // BROADER than the contract. Under plain `--json > out.json` -- the
        // normal way to use it -- the card still prints to the terminal, its
        // `for md compose:` line still says `sha256=`, and the §13.4 notice had
        // gone into the redirected file. The human was shown one unlabelled
        // sha256 operand with nothing saying a kind was never chosen.
        writeln!(
            stderr,
            "no --kind given; stdout carries the sha256 record. This phrase's digest under each kind:"
        )
        .ok();
        for k in [
            HashKind::Sha256,
            HashKind::Hash256,
            HashKind::Ripemd160,
            HashKind::Hash160,
        ] {
            writeln!(
                stderr,
                "  {:<10} {}",
                k.token(),
                hex(k.digest(&d.x).as_slice())
            )
            .ok();
        }
    }
```

**Do not nest that block inside the engraving-card guard.** It shipped that way
once (R0 round 4, C-1) and `ms hashlock --no-engraving-card` then emitted a bare
sha256 record with empty stderr — the exact silent assumption §13.4 forbids,
while `--help` claimed otherwise. §13.4's clause is unconditional, and the
operator who suppresses the card is the one who most needs the lookup: the card
is what carries the preimage, so suppressing it is the safety-conscious choice.

- [ ] **Step 4: Update `method_line` and the `for md compose:` line**

`method_line` (`:299`) describes the **preimage method** and must keep doing exactly that. Separately, the line that prints `for md compose: … sha256=<h>` must name the chosen kind's option (`ripemd160=<h>`), because it is the only thing keeping the digest function and `md compose`'s option name in agreement (spec §11).

- [ ] **Step 5: Run to verify all pass**

Run: `cargo test -p ms-cli 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/ms-cli/src/cmd/hashlock.rs crates/ms-cli/tests/
git commit -m "ms hashlock: --kind, with a four-digest lookup when it is absent"
```

---

### Task 5b: The release records — the two that break the build, and the two the process mandates

**Files:**
- Modify: `crates/ms-cli/Cargo.toml` (version, and the `ms-codec` pin)
- Modify: `Cargo.lock`, `vendor/`
- Modify: `MIGRATION.md`
- Modify: `CHANGELOG.md`

**Why it is its own task.** Two of these four are load-bearing for the build and
two are mandated by `design/RELEASE_PROCESS.md`; all four were done on the
branch and **none was prescribed by this plan** until R0 round 4 said so. An
executor who follows Task 2 Step 3 and stops has an unresolvable workspace.

- [ ] **Step 1: The pin — this is the one that breaks the build**

`crates/ms-cli/Cargo.toml:20` reads
`ms-codec = { path = "../ms-codec", version = "=0.9.0" }`. The requirement is
**exact**, so Task 2's bump of `ms-codec` to `0.10.0` makes it unsatisfiable.
Move it to `=0.10.0` and bump `ms-cli` itself `0.18.0` → **0.19.0** (its
`--json` key rename is breaking).

```bash
grep -n '^version\|ms-codec = ' crates/ms-cli/Cargo.toml
cargo metadata --locked --offline >/dev/null && echo "workspace resolves"
```

- [ ] **Step 2: Lockfile and vendor**

Run: `cargo metadata --offline >/dev/null` (updates `Cargo.lock`), then
`ci/repro/vendor-freshness.sh`. Task 1 added `ripemd`, so `vendor/` gained
exactly one directory; this step is where the version bumps reach the lockfile.

- [ ] **Step 3: `MIGRATION.md` — RELEASE_PROCESS item 5**

That item is unconditional for a wire-format or API change: *"add a new section
to `MIGRATION.md` per the v0.1 → v0.2 precedent."* This release removes the
public `digest`, changes `qr_text`'s signature, and renames a `--json` key, so
it needs a `## v0.9 → v0.10` section. The file is **oldest-first**; append.

Three things an upgrader cannot get from the CHANGELOG and must get here:

1. `qr_text` is not merely re-signed — the `hash: <kind>` line is
   **unconditional**, so a **sha256** plate cut under v0.10 is NOT byte-identical
   to one cut under v0.9.
2. **A plate with no `hash:` line means the kind is UNKNOWN — NOT sha256.** The
   line records which hash the SCRIPT commits to; its absence records only that
   the tool never asked. Non-sha256 hashlock wallets exist today (spec §13.5:
   `me bundle` already emits byte-identical six-plate output for a sha256 card
   and a `ripemd160` card; §7.4: a decoded `ripemd160` card already shows
   "hashlock" on the device), so an old plate can belong to any of the four.
   The remedy is §13.4's fallback: run with no `--kind` and match the digest
   against the operand the descriptor already names.

   **Do not write the RECORD rule onto a PLATE.** A bare `hash:<hex>` *record*
   does mean sha256 — §6's producer grammar, about a string a producer emits.
   A plate is not a record. Collapsing the two is how this exact sentence
   shipped once saying the opposite (R0 round 5, C-1), inside the fold that was
   closing the silent-sha256 Critical.
3. The record's producer rule keeps **bare** for sha256 precisely so no plate
   already cut becomes unreadable — but a consumer must never emit a bare record
   for a non-sha256 digest, because `me sysw pack` reads bare as sha256 and would
   compose an unspendable wallet.

**Compile the section's snippets before committing it.** They are executable
content in a document nothing else gates: drop them into a scratch test in the
worktree, `cargo test -p ms-codec --test <probe>`, then delete it.

- [ ] **Step 4: `CHANGELOG.md` — structure, not just content**

Three rules the branch got wrong on its first attempt:

1. The entry goes **below** the file's preamble (*"All notable changes…"* and the
   Keep-a-Changelog paragraph), not between the `# Changelog` title and it.
2. **One header per crate**, in the bracketed form every other entry and
   `RELEASE_PROCESS.md` item 2 use: `## ms-codec [0.10.0] — <date>` and
   `## ms-cli [0.19.0] — <date>`. A combined `## ms-codec 0.10.0 / ms-cli 0.19.0`
   header leaves neither crate's history a clean sequence.
3. **Fold any `## <crate> [Unreleased]` section for a crate you are releasing
   into that release's entry.** `ms-cli [Unreleased]` held `--emit-record`
   (F-495); shipping 0.19.0 without folding it leaves a section labelled
   Unreleased describing released work, and 0.19.0's own notes silently omitting
   a feature it contains.

Each entry ends with a **Migration notes** section pointing at `MIGRATION.md`,
per the `## ms-cli [0.18.0]` precedent.

- [ ] **Step 5: Commit**

```bash
git add crates/ms-cli/Cargo.toml Cargo.lock vendor CHANGELOG.md MIGRATION.md
git commit -m "release: ms-codec 0.10.0 / ms-cli 0.19.0, with the migration record"
```

---

### Task 6: Phase gate

**Files:** none modified.

- [ ] **Step 1: Full suite**

Run: `cargo nextest run --locked 2>&1 | tail -5`
Expected: all pass, 0 failed.

- [ ] **Step 2: Format — the command CI runs, not the obvious one**

Run: `cargo +1.95.0 fmt --all -- --check`

**The `+1.95.0` is REQUIRED.** `rust-toolchain.toml` pins 1.85.0, which a bare
`cargo fmt` would use and which **formats differently**
(`mnemonic-secret/.github/workflows/rust.yml:60-63`, whose comment records a commit that left
master red by trusting the bare form). An earlier draft of this plan used the
bare command — exactly the one the workflow warns against.

- [ ] **Step 2b: Lint, with the flag the CI job actually uses**

Run these as SEPARATE commands — chaining them with `&&` means a fmt failure
silently skips the lint:

```bash
cargo clippy -p ms-codec --all-targets --locked -- -D warnings
cargo clippy -p ms-cli   --all-targets --locked -- -D warnings
```

Expected: both clean. `-D warnings` is what the required CI context uses; a gate
without it cannot see the failure C-1 was about.

- [ ] **Step 3: Vendor freshness**

Run: `ci/repro/vendor-freshness.sh`
Expected: the committed `vendor/` tree satisfies `Cargo.lock`.

- [ ] **Step 4: Record the rev for phase 3**

Run:
```bash
git rev-parse HEAD
sha256sum crates/ms-codec/tests/vectors/hashlock-v0.8.json
```

**Both values go in the phase-close note.** Phase 4 re-pins the fork's copy of
the corpus against that SHA; without it the fork's provenance check has nothing
to move to.

Phase 3 (`me-cli`) pins **this exact rev** in its `Cargo.toml`, following the `mt-codec` precedent at `mnemonic-engrave/crates/me-cli/Cargo.toml:54-74` — a git rev pin, not a `cargo publish`, because publishing is irreversible and pinning a rev is not (spec §9).

- [ ] **Step 5: Push**

Use the project's push ritual. Report the rev in the phase-close note so phase 3 can start.
