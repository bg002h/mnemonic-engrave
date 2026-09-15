# R0 round 7 — fold verification: the phase-2 journey walk + round-6's Minors

**Scope: two questions only.** Did the fold close each finding, and did the fold
introduce a new defect? Not a fresh audit. The spec, phases 1/3/4, and everything
closed in R0 rounds 1–6 were not re-reviewed. Round 6's settled facts — the
23-cell flag matrix, the narrowed `!(args.json && args.no_engraving_card)` guard,
the plate-vs-record prose, the 14/1/4 clippy counterfactual, and every gate
number — were taken as given and not re-derived. The journey walk's verification
that §13.4's fallback survives `--no-engraving-card`, `--json` and the pipe to
`me sysw pack` was likewise taken as given (and incidentally re-observed).

**Artifacts.**

1. **BRANCH** `hashkinds-p2` @ `0f70a8c` in `/scratch/code/shibboleth/ms-worktrees/hashkinds-p2`.
   Fold = `dabd518..0f70a8c` (`3056360`, `0f70a8c`).
2. **PLAN** `design/IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md` in
   `mnemonic-engrave`. Fold = `e83cc637`.

**Toolchain.** `export PATH=/home/bcg/.cargo/bin:$PATH`, then every measurement
taken **from inside the worktree**, where `cargo --version` →
`cargo 1.85.0 (d73d2caf9 2024-12-31)`, matching `rust-toolchain.toml`. From the
`mnemonic-engrave` cwd the same shim reports `1.97.0-nightly` — the pin is
cwd-resolved. The system 1.98.0 phantom-clippy trap was avoided.

**Tree state.** `git status --porcelain` empty in both repos before and after.
Every mutation below was reverted by file restore and re-hashed:
`crates/ms-cli/src/cmd/hashlock.rs` = `3bb18cb7…0ef7`,
`crates/ms-codec/src/hashlock.rs` = `33ee5875…2a45`, byte-identical to `0f70a8c`.

---

## 0. The green claims — independently re-measured

One capture per suite, greped afterwards.

| gate | claimed | measured | verdict |
| --- | --- | --- | --- |
| `cargo test --workspace --locked` | 101 suites / 0 failures | exit 0; `grep -c '^test result:'` → **101**; result lines not ` 0 failed` → **0**; `FAILED` → **0** | **TRUE** |
| `cargo nextest run --locked --all-targets` | 583 tests | `583 tests run: 583 passed, 11 skipped`, exit 0 | **TRUE** |
| `clippy -p ms-codec --all-targets --locked -- -D warnings` | 0 | exit 0, output is `Checking`/`Finished` only | **TRUE** |
| `clippy -p ms-cli --all-targets --locked -- -D warnings` | 0 | exit 0, clean | **TRUE** |
| `cargo +1.95.0 fmt --all -- --check` | clean | exit 0, zero bytes of output | **TRUE** |
| `ci/repro/vendor-freshness.sh` | OK | `vendor-freshness: OK — vendor/ satisfies Cargo.lock.` | **TRUE** |

**The four plan gates, against `e83cc637`'s message.**

| gate | message says | I measured |
| --- | --- | --- |
| `scripts/h2-plan-blocks-vs-tree.sh <plan> <worktree>` | 12 blocks, 0 FAIL | **`12 blocks checked, 0 FAIL`**, exit 0 |
| `scripts/plan-cite-check.sh <plan>` | 11 / 11 ; dangling 0 ; ambiguous 0 | **`citations resolved: 11 / 11 ; dangling: 0 ; ambiguous: 0`**, exit 0 |
| `scripts/plan-glyph-check.sh <plan>` | 55 strings ; 0 undrawable | **`operator strings scanned: 55 ; undrawable: 0`**, exit 0 |
| `scripts/plan-table-check.sh <plan>` | 37 rows ; 0 malformed | **`table rows checked: 37 ; malformed: 0`**, exit 0 |

**rustdoc.** `cargo doc -p ms-codec --no-deps --locked` → 2 warnings, both
**pre-existing and unrelated** to the fold: `bch.rs:33`
(`private_intra_doc_links` on `POLYMOD_INIT`) and `tag.rs:58`
(`invalid_html_tags`, `"<non-utf8>"`). The new `opcode()` doc comment's `<32>`,
`<hashop>` and `<h>` are inside code spans and are **not** flagged.
`lib.rs:39` carries `#![cfg_attr(not(test), deny(missing_docs))]`; `opcode()` is
documented, so the deny still passes — confirmed by the clean clippy above.

**The plan's own test-count measurement, recomputed from history** rather than
read from the plan:

```
$ for c in $(git log --format=%h --reverse -- crates/ms-cli/tests/hashlock_kind.rs); do \
      echo "$c tests=$(git show $c:crates/ms-cli/tests/hashlock_kind.rs | grep -c '^#\[test\]')"; done
72acd1a tests=4   hashlock: --kind gets the tests it shipped without...
8891bbf tests=5   fold: R0 round 4 ...
cfdb738 tests=6   fold: R0 round 5 ...
3056360 tests=10  fold: journey walk -- four things the card was silent or wrong about
0f70a8c tests=11  hashlock: the fallback header names the preimage, not a phrase
```

`4 → 5 → 6 → 10 → 11` — exactly what `plan:139` states. **Eleven, seven added by
review rounds.** The dispatch brief said *"six new tests"*; the fold added
**five** (nextest 578 → 583). The plan is right and the brief is off by one; no
finding against the artifact, recorded so the controller's own tally is corrected.

**The two spec quotes the fold leans on, checked verbatim against the spec and
not against an earlier report:**

| quoted in | spec text | verdict |
| --- | --- | --- |
| the I-4 test's doc comment + `plan:1638` — *"a write-down list that omits a field is worse than no list, because the operator stops writing where the list stops"* | `SPEC_hashlock_kinds.md:622-623`, word for word | **accurate** |
| the I-3 test's doc comment — §13.2 is *"the cycle's operator-facing Critical"*, and *"Both are 64 hex; nothing but a label distinguishes them"* | `SPEC_hashlock_kinds.md:604`, `:609` | **accurate** |

---

## 1. Findings CLOSED — stated so a later round does not re-derive them

### Journey **I-1** (the card named `OP_SHA256` under all four kinds) — **CLOSED**

`crates/ms-codec/src/hashlock.rs:122-138` adds `HashKind::opcode()`;
`crates/ms-cli/src/cmd/hashlock.rs:509` interpolates it.

**All four opcode names verified against an authoritative source, not the doc
comment.** rust-miniscript's own encoder (`vendor/miniscript/src/miniscript/astelem.rs`
in `descriptor-mnemonic`) is the fragment → opcode mapping:

```
:60 Terminal::Sha256    → OP_SIZE … :64 OP_SHA256
:67 Terminal::Hash256   → OP_SIZE … :71 OP_HASH256
:74 Terminal::Ripemd160 → OP_SIZE … :78 OP_RIPEMD160
:81 Terminal::Hash160   → OP_SIZE … :85 OP_HASH160
```

and rust-bitcoin's opcode table (`vendor/bitcoin/src/blockdata/opcodes.rs`)
confirms the four names exist and mean what the card implies:
`OP_RIPEMD160 => 0xa6`, `OP_SHA256 => 0xa8`, `OP_HASH160 => 0xa9`,
`OP_HASH256 => 0xaa`. `OP_SIZE` precedes in all four arms, so the surviving
`OP_SIZE 32` half is correct for every kind. Observed on the real binary:

```
sha256    … the script checks OP_SIZE 32 before OP_SHA256 (composer spec §8i, F-132).
hash256   … the script checks OP_SIZE 32 before OP_HASH256 …
ripemd160 … the script checks OP_SIZE 32 before OP_RIPEMD160 …
hash160   … the script checks OP_SIZE 32 before OP_HASH160 …
```

**Is `opcode()` the right home?** Yes — no new dependency, `&'static str`, and
`ms-codec` already carries exactly this concern: `hashlock.rs:252` documents
`token()` as *"the miniscript fragment names, lowercase"* and `:31` already says
*"a miniscript `sha256(H)` preimage is exactly 32 bytes"*. `opcode()` is the
same fact one column over, not a new layer violation. **Not a finding.**

### Journey **I-2** (an operand `md compose` refuses, with nothing saying why) — **CLOSED for the `md compose` door**

`hashlock.rs:473-486`. **The conditional phrasing achieves what it claims.** The
sentence is *"requires `md compose` support for `<kind>=`. If it answers "unknown
option", that support has not shipped in your `md` yet …"* — the first clause is
timelessly true (the operand does require support), and the only time-bound claim
is guarded by `If it answers`. Phase 1 shipping makes the antecedent false, not
the sentence.

**The error string it predicts is the real one**, re-run today against
`descriptor-mnemonic`'s built `md` for all three kinds:

```
md: path `2of3,hash256=98a2…d488`:   unknown option `hash256`
md: path `2of3,hash160=b5b7…d0bd`:   unknown option `hash160`
md: path `2of3,ripemd160=09e7…946b`: unknown option `ripemd160`
```

Absent for `sha256`, which works today — confirmed on the binary.
**Residual:** I-2 named *two* doors; only one is answered. See **M-1** below.

### Journey **I-3** (a `hash256` record is one prefix-strip from a valid `sha256` record) — **text CLOSED and fully reproduced; placement NOT closed — see I-1 below**

Every factual claim in the new WARNING was **reproduced**, not assessed, with
`me` 0.9.0 (`mnemonic-engrave/target/debug/me`):

```
$ printf 'hash:hash256:98a2…d488\n' | me sysw pack --no-passphrase --out tagged.bin
me: record 0 … hash: must be exactly 64 hex characters
[exit 4]

$ printf 'hash:98a2…d488\n' | me sysw pack --no-passphrase --out stripped.bin
[exit 0]

$ me sysw show stripped.bin
public record 0: sha256 hashlock (hash:) — 98a20fc2..641cd488
```

So *"a tool refuses it asking for \"exactly 64 hex characters\""* — true;
*"the result is accepted"* — true; *"as a SHA256 hashlock"* — true, that is
`me sysw show`'s literal label. The warning is accurate and is `hash256`-only,
correctly: `ripemd160`'s 40-hex body is refused with the **same** message after
the strip, so the 20-byte kinds stay stuck-and-safe. **The prose is sound. Its
guard is not — that is I-1 below.**

### Journey **I-4** (the write-down line named method, omitted kind) — **behaviour CLOSED; its test is half-vacuous — see I-2 below**

`hashlock.rs:501` now names both axes and adds the recovery step. Observed:

> `phrase: 28 characters -- write the method line AND the hash line (ripemd160) next to your phrase unless the phrase is cut on a HASHLOCK PHRASE plate, which carries both; if the method line is lost, try each method that shipped with the version named on this card (ms-cli 0.19.0), and if the hash line is lost, re-run with no --kind and match the digest against your descriptor`

**The new "carries both" claim is NOT a plate-vs-record conflation** — I checked
this specifically, because that class has produced a finding in every round of
this cycle. `SPEC_hashlock_kinds.md:578` — *"13.1 The preimage plate names the
kind — in the QR, and on the locator's `hash` row"* — and `:596` — *"the kind
goes in the QR text and on the locator's `hash` row"*. The QR half is implemented
on this branch: `ms-codec/src/hashlock.rs:404` builds
`let kind_line = format!("\nhash: {}", kind.token());` into `qr_text`. The
pre-existing clause already asserted the plate carries the **method**; the fold
extends the same assertion to the kind, and the spec assigns both to that plate.
The locator-row half is phase 4, exactly as the method half's renderer was — the
sentence's forward-looking status is unchanged, not newly introduced.
**Accurate. Not a finding.**

### Journey **M-1** (the fallback header said "this **phrase's** digest") — **CLOSED**

`hashlock.rs:573-579`. Observed on the `--hex` and plate routes: *"no --kind
given; stdout carries the sha256 record. This preimage's digest under each
kind:"*. True on every route.

### Round-6 **M-1** (the stranded `the GUI` in `MIGRATION.md`, + the third copy at `plan:210`) — **CLOSED**

`MIGRATION.md:191-196` now parses and drops the unsupported attribution:
*"It is a machine-readable contract, so a consumer reading the old key gets
`None` rather than a stale value."* `plan:224-230` likewise replaces
*"GUI-facing contract"* with *"machine-readable contract"*. Remaining `GUI`
matches in `CHANGELOG.md` (`:344`, `:517`, `:703`, `:704`) are unrelated
historical entries about the schema mirror.

The fold's **new** parenthetical claim — *"No code in this constellation reads
either key today — verified by grep across every Go and Rust source in it"* — was
re-run independently rather than inherited:

```
sha256_operand|hash_operand, --include=*.rs --include=*.go, vendor/target/third_party excluded
  ms-worktrees/hashkinds-p2 : 4   (all producer-side: hashlock.rs:400,412; hashlock_outputs.rs:184; hashlock_kind.rs:175)
  mnemonic-engrave          : 0
  descriptor-mnemonic       : 0
  mnemonic-key              : 0
  mnemonic-toolkit          : 0
  seedhammer                : 0
  mnemonic-transaction      : 0
```

**Accurate.**

### Round-6 **M-2** (the stale *"suppressed under `--json`"* comment) — **CLOSED**

`grep -rn "suppressed under --json"` over the branch returns exactly **one** hit,
`hashlock.rs:410`, which is inside the retraction itself (*"This comment said
\"suppressed under --json\" for one round, which was the guard being one flag
wider than its contract"*). The plan's mirror at `plan:1515-1526` carries the
same replacement, which is why the block gate stays at 12/0 FAIL.

### Round-6 **M-3** (`plan:133` said three tests, measured two) — **CLOSED**

Replaced by the `4 → 5 → 6 → 10 → 11` attribution table at `plan:139-149`,
recomputed above and correct.

### Mutation proof of the five new tests — **each reddens exactly one test**

Run as `cargo test -p ms-cli --test hashlock_kind` after each single-line
mutation, reverted and re-hashed between runs. Baseline is 11 passed.

| # | mutation | result | test reddened |
| --- | --- | --- | --- |
| M1 | `opcode()`: `Hash160 => "OP_HASH160"` → `"OP_SHA256"` | 10 passed, 1 failed | `the_card_names_the_kinds_own_opcode` |
| M1b | drop `(64 hex characters)` from the script line | 10 passed, 1 failed | `the_card_names_the_kinds_own_opcode` |
| M2 | caveat guard `is_some_and(|k| k != Sha256)` → `is_some()` | 10 passed, 1 failed | `a_non_sha256_kind_warns_that_md_may_not_accept_the_operand` (the **negative** arm) |
| M2b | caveat guard → `if false` | 10 passed, 1 failed | same test (the **positive** arm) |
| M3 | `kind == Hash256` → `kind == Sha256` on the tag warning | 10 passed, 1 failed | `hash256_warns_against_stripping_its_own_tag` |
| M4′ | remove `({})` + `kind.token()` from the write-down line | 10 passed, 1 failed | `the_write_down_line_names_both_axes` |
| **M4b** | **remove `write the method line AND` from the write-down instruction** | **11 passed, 0 failed** | **NONE — see I-2** |
| M5 | `This preimage's digest` → `This phrase's digest` | 10 passed, 1 failed | `the_fallback_header_names_the_preimage_not_a_phrase` |

Seven of eight mutations redden exactly one test and no other. **M4b does not
redden anything** — that is finding I-2.

### `the_fallback_header_names_the_preimage_not_a_phrase` is **not** over-strict — examined and declined

The brief asked whether asserting `!line.contains("phrase")` on the
`--hashlock-phrase-stdin` route will force a future author to fight the test.
It will not, and here is why: the assertion is scoped to **one line** found by
`l.contains("digest under each kind")`, and that line is a **single static
string** with no per-route branch — the same bytes reach `--random`, `--hex`,
the ms1-plate route and the phrase route. Any future edit that puts "phrase"
back into it would be wrong on three of the four routes, which is the defect the
test exists to catch. The test constrains no other card line. **No finding.**

### Card volume and warning contradictions — examined, no finding

Line counts of the full card (stderr), measured:

| invocation | card lines |
| --- | --- |
| `--kind sha256`, phrase | 10 |
| `--kind ripemd160` / `--kind hash160`, phrase | 11 |
| `--kind hash256`, phrase | 12 |
| `--kind hash256 --method sha256` (adds the brainwallet WARNING) | 13 |
| `--kind hash256 --random` / `--hex` | 12 |
| **no `--kind`, phrase (card + the four-digest table)** | **15** — the maximum |

No combination exceeds 15 lines, so nothing scrolls off a terminal. No pair
contradicts: the `hash256` WARNING and the `md compose` caveat agree (*"the tool
is what needs hash256 support"* / *"`md` is what has to catch up"*), and the
tag WARNING is mutually exclusive with the four-digest table (the table prints
only when no `--kind` was given, in which case `kind` is the `sha256` default
and the WARNING is suppressed — correctly, since the emitted record is bare).

---

## 2. New defects introduced by the fold

### I-1 — BRANCH — **Important** — the `hash256` tag-strip WARNING is nested inside the `--no-engraving-card` guard, so it is absent on the exact route that produces the hazardous record

**Location.** `crates/ms-cli/src/cmd/hashlock.rs:509-523`. The warning sits
inside the card block opened at `:454` (`if !args.no_engraving_card {`) and
closed at `:546`. The §13.4 four-digest listing, by contrast, sits **outside** it
at `:547`.

```
454:    if !args.no_engraving_card {
...
509:        if kind == HashKind::Hash256 {          ← the new WARNING, INSIDE
...
546:    }
547:    if args.kind.is_none() && !(args.json && args.no_engraving_card) {   ← §13.4, OUTSIDE
```

**Constructed failure.** The record that carries the hazard is emitted on
**stdout** and is entirely independent of the card flag. So:

```
$ printf 'correct horse battery staple' \
    | ./target/debug/ms hashlock --hashlock-phrase-stdin --kind hash256 --no-engraving-card
hash:hash256:98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
                                   ← stderr: COMPLETELY EMPTY. Zero bytes.
```

The operator now holds the tagged record and no warning. They pipe it onward,
`me sysw pack` answers *"hash: must be exactly 64 hex characters"* (exit 4,
reproduced above), they obey it literally, and the stripped record packs and is
labelled `sha256 hashlock` — the full I-3 chain, with the fold's mitigation
missing at every step. **This is the walk's own reproduction path**: journey
walk A.5 ran the pipe with `--no-engraving-card`, and C.2 is its continuation.

**Why this is Important and not a design preference.**

1. It is a **regression of a class this cycle already fixed twice.** R0 round 4's
   C-1 was *"`--no-engraving-card` assumed sha256"* — the §13.4 fallback nested
   inside this same guard — and the fix moved it out. R0 round 5's I-1 then
   narrowed the remaining guard to `!(json && no_engraving_card)` because it was
   *"one flag wider than its contract."* The fold added a new funds-safety notice
   and put it back in the place both rounds took things out of.
2. **There is no contract reason to suppress it.** §4.4/§11 pin stderr to exactly
   the `PrivateKeyMaterial` advisory only under `--json --no-engraving-card` —
   round 5 settled that, and the branch honours it. Under plain
   `--no-engraving-card` stderr is *not* pinned: the §13.4 table prints there
   today, observed:
   ```
   $ … ms hashlock --hashlock-phrase-stdin --no-engraving-card
   no --kind given; stdout carries the sha256 record. This preimage's digest under each kind:
     sha256     3cf5d421…4c12
     hash256    98a20fc2…d488
     …
   ```
3. **The warning carries no secret.** The card's reason to exist is the preimage
   (`THIS CARD CARRIES THE PREIMAGE -- the secret`), and `--no-engraving-card` is
   the secret-hygiene flag an operator reaches for precisely *when piping*. The
   fold makes a funds-safety warning a casualty of a secret-hygiene flag.
4. **The test cannot see it.** `hash256_warns_against_stripping_its_own_tag`
   exercises only `--hashlock-phrase-stdin --kind hash256` — the card-on route.
   It passes at 11/11 while the construction above emits nothing.

**Fix shape.** Move the `if kind == HashKind::Hash256` block outside the card
guard, under the same condition the §13.4 listing already uses
(`!(args.json && args.no_engraving_card)`), and add a `--no-engraving-card` arm
to the test so the guard has something that can fail.

**Verified by:** the two commands pasted above, run on `target/debug/ms` built
from `0f70a8c`; `grep -n "no_engraving_card" crates/ms-cli/src/cmd/hashlock.rs`
→ `:81, :454, :547, :559`; the full `me sysw pack` / `me sysw show` chain in §1.

---

### I-2 — BRANCH — **Important** — `the_write_down_line_names_both_axes` has a false-PASS path: its method-axis assertion is satisfied by a different occurrence of `"method line"` later in the same line

**Location.** `crates/ms-cli/tests/hashlock_kind.rs:369-378`.

```rust
    assert!(
        line.contains("method line"),
        "the method axis went missing: {line:?}"
    );
```

The line under test contains `"method line"` **twice** — once in the write-down
instruction (*"write the method line AND the hash line (ripemd160)"*) and once in
the recovery clause (*"if the method line is lost, try each method that
shipped…"*). A substring test on the whole line cannot tell them apart.

**Constructed failure (mutation M4b).** Delete the method axis from the
write-down instruction — that is, regress to the mirror image of the very
finding this test closes:

```
-  write the method line AND the hash line ({}) next to your phrase …
+  write the hash line ({}) next to your phrase …
```

Resulting card line:

```
phrase: 28 characters -- write the hash line (ripemd160) next to your phrase unless the
phrase is cut on a HASHLOCK PHRASE plate, which carries both; if the method line is lost, …
```

```
$ cargo test -p ms-cli --test hashlock_kind
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Green, with the method axis gone from the write-down instruction.** An operator
complying exactly would write down `ripemd160` and nothing about `hardened` —
which is I-4's defect with the two axes swapped, and the suite says nothing. The
test is named `…names_both_axes` and guards one; the kind axis is genuinely
mutation-proven (M4′ above), the method axis is not.

Per this project's severity rule, *"a test that reports a false PASS"* is a
blocking class, and this is literally one: the behaviour is broken and the suite
is green.

**Fix shape.** One line — assert on the instruction, not the bare noun:
`line.contains("write the method line")`. Re-run M4b to confirm it then reds.

**Verified by:** M4b executed as shown, source restored and re-hashed to
`3bb18cb7…0ef7`; M4′ executed as the control, proving the kind half does red.

---

## 3. Minor

### M-1 — BRANCH — Minor — journey I-2 named two downstream doors; the fold answers only `md compose`, and `me sysw pack`'s refusal of a 40-hex record is still unexplained

Journey I-2's body cites both: *"`md compose` answers `unknown option
ripemd160` … and `me sysw pack` answers `hash: must be exactly 64 hex
characters` to the exact pipe `ms hashlock --help`'s EXAMPLES advertises.
Neither error mentions kinds."* The new caveat names `md compose` only.
Reproduced today for `ripemd160`:

```
$ printf 'hash:ripemd160:09e7bb…946b\n' | me sysw pack --no-passphrase --out r.bin
me: record 0 … hash: must be exactly 64 hex characters
[exit 4]
```

Graded **Minor**, not Important, on the walk's own rule: for `ripemd160` and
`hash160` the operator is left **stuck and safe** — the strip yields 40 hex,
which `pack` also refuses — so the cost is confusion, not a wrong outcome, and
the card no longer tells them nothing. The dangerous door (`hash256`) *is*
warned, subject to I-1. The `--help` half of I-2 needs no change: the EXAMPLES
pipe carries no `--kind`, so it emits a bare `sha256` record that packs cleanly.

**Owning phase:** phase 2 if folded now; otherwise phase 3/4, when `me sysw pack`
gains kind support and the caveat can name both doors in one sentence.

---

## 4. Counts and verdict

| severity | round 6 | **round 7** |
| --- | --- | --- |
| Critical | 0 | **0** |
| Important | 0 | **2** (I-1, I-2) |
| Minor | 3 | **1** (M-1) |
| Secret-handling (non-blocking) | 0 | **0** |

**Journey-walk disposition:** I-1 **closed**. I-2 **closed** for the `md compose`
door (residue → M-1). I-3 **text closed and fully reproduced; placement not
closed** → I-1. I-4 **behaviour closed; its test is half-vacuous** → I-2. M-1
**closed**. M-2 was declined by the walk itself and is not a fold obligation.

**Round-6 disposition:** M-1 **closed**, M-2 **closed**, M-3 **closed**.

**New defects introduced by the fold: two** — the tag warning nested in the
card guard (I-1) and the false-PASS path in the write-down test (I-2).

## **NOT GREEN** — 0 Critical / 2 Important.

Both Importants are one-line fixes in the same two files the fold already
touched, and neither is a design problem: the *content* the fold added is
correct everywhere I could check it. All four opcode names hold against
rust-miniscript's encoder and rust-bitcoin's opcode table; the `md compose`
caveat's conditional phrasing genuinely survives phase 1 and predicts the real
error string for all three kinds; the `hash256` warning's every clause
reproduces end to end on `me` 0.9.0, down to `me sysw show`'s literal
`sha256 hashlock` label; the *"carries both"* clause is what §13.1 assigns to
that plate and is not the plate-vs-record conflation it superficially resembles;
and seven of eight mutations red exactly one test.

What the two findings share is that **the fold reasoned about the text and not
about the guard around it.** The warning that prevents a funded-but-unspendable
wallet is switched off by the flag an operator uses when piping — the third time
this cycle a notice has been found nested one guard too deep — and the test that
was written to stop a write-down axis going missing cannot see that axis go
missing. Neither is visible by reading the diff; both took a constructed
execution to find.
