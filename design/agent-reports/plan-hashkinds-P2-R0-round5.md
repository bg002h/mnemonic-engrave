# R0 round 5 — Hashkinds Phase 2 fold verification (mnemonic-secret)

**Scope: two questions only.** Did the fold close each round-4 finding, and did
the fold introduce a new defect? Not a fresh audit. The spec, phases 1/3/4, and
everything closed in rounds 1–3 were not re-reviewed. Round 4's verification of
the digest layer, the corpus, the four original `hashlock_kind.rs` tests, and the
honesty of the F-534 deferral was taken as settled and not re-derived.

**Artifacts.**

1. **BRANCH** `hashkinds-p2` @ `c9a17d4` in `/scratch/code/shibboleth/ms-worktrees/hashkinds-p2`.
   Fold = `1a723eb..c9a17d4` (`8891bbf`, `c9a17d4`).
2. **PLAN** `design/IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md` in
   `mnemonic-engrave`. Fold = `8da7c008`.

**Toolchain.** Inside the worktree, `export PATH=/home/bcg/.cargo/bin:$PATH` →
`cargo 1.85.0 (d73d2caf9 2024-12-31)` / `rustc 1.85.0`, matching
`rust-toolchain.toml`. The system 1.98.0 phantom-clippy trap was avoided.

---

## 0. The green claims — independently re-measured

| gate | claimed | measured | verdict |
| --- | --- | --- | --- |
| `cargo test --workspace --locked` | 101 ok, 0 failed | exit 0; `grep -c '^test result:'` → **101**; non-`ok` result lines → **0**; `FAILED` → **0** | **TRUE** |
| `cargo nextest run --locked --all-targets` | 577 run / 577 passed / 11 skipped | **577 tests run: 577 passed, 11 skipped**, exit 0 | **TRUE** |
| `clippy -p ms-codec --all-targets --locked -- -D warnings` | 0 | exit 0, clean | **TRUE** |
| `clippy -p ms-cli --all-targets --locked -- -D warnings` | 0 | exit 0, clean | **TRUE** |
| `cargo +1.95.0 fmt --all -- --check` | clean | exit 0, no output | **TRUE** |
| `ci/repro/vendor-freshness.sh` | OK | `vendor-freshness: OK — vendor/ satisfies Cargo.lock.` | **TRUE** |
| corpus SHA | `0a911f78…8ce` | `sha256sum` on the file → identical | **TRUE** |
| versions / exact pin | 0.10.0 / 0.19.0 / `=0.10.0` | `crates/ms-codec/Cargo.toml:3` `0.10.0`; `crates/ms-cli/Cargo.toml:3` `0.19.0`; `:20` `version = "=0.10.0"` | **TRUE** |
| `cargo vendor` added dirs | exactly one (`vendor/ripemd`) | `git diff --name-only <base>..HEAD -- vendor` → only `vendor/ripemd` | **TRUE** |

**Plan gates in `8da7c008`'s message, all four re-run by me:**

| gate | message says | I measured |
| --- | --- | --- |
| `scripts/h2-plan-blocks-vs-tree.sh <plan> <worktree>` | 11 blocks checked, 0 FAIL | **11 blocks checked, 0 FAIL**, exit 0 |
| `scripts/plan-cite-check.sh` | 11 / 11 ; dangling 0 ; ambiguous 0 | **11 / 11 ; 0 ; 0**, exit 0 |
| `scripts/plan-glyph-check.sh` | 44 scanned ; 0 undrawable | **44 ; 0**, exit 0 |
| `scripts/plan-table-check.sh` | 23 rows ; 0 malformed | **23 ; 0**, exit 0 |

**Every green claim either fold makes reproduces.** No false gate number.

---

## 1. Findings CLOSED — stated so a later round does not re-derive them

### Round-4 **C-1** (branch — `--no-engraving-card` silently assumed sha256) — **CLOSED**

The §13.4 fallback now sits outside the card guard at
`crates/ms-cli/src/cmd/hashlock.rs:502`, conditioned `args.kind.is_none() && !args.json`.
The two conditions are exhaustive and disjoint over `args.kind.is_none()`, so on
**every** kind-omitted path exactly one channel carries the notice. Verified by
running the built binary over the whole argv matrix and capturing both streams
separately:

| argv (no `--kind`) | stdout `digests_by_kind` | stderr §13.4 line | stderr lines |
| --- | --- | --- | --- |
| `--hashlock-phrase-stdin` | — | yes | 15 |
| `… --no-engraving-card` | — | **yes** (was the defect) | 5 |
| `… --method sha256 --no-engraving-card` | — | yes | 5 |
| `… --emit-record` | — | yes | 17 |
| `--hex - --no-engraving-card` | — | yes | 5 |
| `--in plate.ms1` | — | yes | 14 |
| `--in plate.ms1 --no-engraving-card` | — | yes | 5 |
| `--random --out F --no-engraving-card` | — | yes | 5 |
| `… --json --no-engraving-card` | **yes** | no | **1 (advisory only)** |
| `… --json` | **yes** | no | 11 (card + advisory) |
| `… --emit-record --json --no-engraving-card` | **yes** | no | **1** |
| `--hex - --json --no-engraving-card` | **yes** | no | **1** |
| `--in plate.ms1 --json --no-engraving-card` | **yes** | no | **1** |
| `--random --out F --json --no-engraving-card` | **yes** | no | **1** |

With an explicit `--kind` the notice is correctly absent on both channels
(`--kind sha256 --no-engraving-card` → stderr 0 lines; `--json --kind ripemd160`
→ no `digests_by_kind`, no `kind_specified`).

**Stderr purity is intact in every `--json --no-engraving-card` combination** —
exactly the one-line `PrivateKeyMaterial` advisory, including under
`--emit-record`, `--random`, `--hex` and an ms1 plate. `hashlock_outputs.rs`'s
`json_both_variants` still passes.

**There is no path where §13.4 is violated on both channels at once.** (There is
one where it is violated on the channel the human is looking at — see **I-1**.)

### Round-4 **C-2** (plan — Task 3 Step 1 did not compile) — **CLOSED**

`grep -n "hex::" <plan>` → two hits, both in prose describing the defect
(plan:73, plan:743). No code block calls `hex::encode`. Task 3 Step 1 is now
`mode=whole` against `crates/ms-codec/tests/hashlock_kat.rs` and the gate reports
`(95 lines, identical)`.

Round 4 named two divergences *inside* C-2 that were carried from round 3's I-5;
both are closed by the same replacement:

- the `Row`-struct loop covering 11 pairs against the branch's 22 — gone, the
  block is the shipped file, which loops both stems over all rows.
- Step 4's mutation expectations naming the plan's strings rather than the
  branch's — **I ran both prescribed mutations against the real KAT:**

  | plan Step 4 mutation | plan says | actual failure message |
  | --- | --- | --- |
  | `digest_hash256` one word short | "FAIL on `hash256`" | `… correct horse battery staple/hardened: hash256` |
  | `HashKind::Ripemd160` arm → `digest_hash160` | "FAIL on `dispatch for ripemd160`" | `… correct horse battery staple/hardened: dispatch for ripemd160` |

  Both reverted from a byte-identical backup (verified by SHA-256).

### Round-4 **I-1** (branch — no MIGRATION.md section) — **section added; but see C-1 below**

`MIGRATION.md` gains `## v0.9 → v0.10` at line 119, appended in the file's
oldest-first order. Its factual claims check out **except one**:

- `qr_text(hardened: bool, kind: HashKind, phrase: &str)` — matches
  `crates/ms-codec/src/hashlock.rs:373` exactly.
- "a `hash: <kind>` line is inserted between `method:` and `phrase:`
  unconditionally" — matches the body (`kind_line` is unconditional, pushed
  between `method` and `LABEL`).
- "194 → **210 bytes** … stays at **53 QR modules**" — spec §13.1's own table
  says the same, and the arithmetic is independently right: QR v8-L holds 192
  bytes and v9-L holds 230, so 194 already needed v9 (21 + 4×8 = **53 modules**)
  and 210 still fits v9. The branch's `the_worst_case_is_210_bytes` asserts
  210 / 151 / 207 / 148, and 207 − 194 = 13 = `len("\nhash: sha256")`,
  210 − 194 = 16 = `len("\nhash: ripemd160")`.
- "`ms-cli`'s `ms-codec` requirement moves from `=0.9.0` to `=0.10.0`" — the
  pre-change line is `crates/ms-cli/Cargo.toml:20` = `=0.9.0` (confirmed by
  `plan-cite-check`'s printed line), the branch has `=0.10.0`.
- "no deprecated alias: nine clippy errors in ms-codec and four in ms-cli" —
  **does not reproduce. See M-1.**
- "Reading an existing plate: no `hash:` line MEANS sha256" — **wrong. See C-1.**

### Round-4 **I-2** (plan — no step for the ms-cli release work) — **CLOSED**

`### Task 5b` exists at plan:1375 and prescribes all four items round 4 named:
the `=0.9.0` → `=0.10.0` pin and the `ms-cli` 0.18.0 → 0.19.0 bump (Step 1), the
lockfile and vendor (Step 2), `MIGRATION.md` (Step 3), `CHANGELOG.md` (Step 4).
Task 2 Step 3 now states in as many words that it is the ms-codec half only and
that stopping there leaves an unresolvable workspace. Step 2's claim that
`vendor/` gained *exactly one* directory is true (`vendor/ripemd` only).

### Round-4 **I-3** (plan — 100 suites vs 101) — **CLOSED**

Plan:140 `| test suites | **101 ok, 0 failed** |` and plan:141 a new nextest row
at **577 run, 577 passed**, 11 skipped. Both reproduce exactly. `grep -n "100 ok"`
→ no hits.

### Round-4 **I-4** (plan — three unit tests the branch never had) — **CLOSED, written not struck**

All three now exist in `crates/ms-codec/src/hashlock.rs`'s `mod tests`, and
**all three are mutation-proven** (each mutation applied singly, test binary run,
file restored from a backup and SHA-256-verified):

| mutation | test that reds | others in the binary |
| --- | --- | --- |
| `HashKind::Ripemd160` arm → `digest_hash160` | `the_dispatch_selects_the_matching_function` | all pass |
| `digest_hash256` one word short | `the_four_digests_are_four_different_functions` | all pass |
| `HashKind::Ripemd160.token()` → `"Ripemd160"` | `tokens_are_the_lowercase_fragment_names` | all pass |

### Round-4 **I-5** (plan — Task 4 Step 4b item 4, and `151` unstated) — **CLOSED**

Plan:905–911 now carries a four-row table — 210 / **151** / 207 / 148 — matching
the branch's `the_worst_case_is_210_bytes` assertion for assertion, plus the
`210 − 207 = 3` margin. `grep -n "151" <plan>` → plan:909. The "ripemd160 is the
longest of the four tokens (9 characters against 7, 7, 6)" claim is correct
(`sha256`=6, `hash256`=7, `ripemd160`=9, `hash160`=7).

### Round-4 **I-6** (plan — Task 5 Step 1 was the superseded `--phrase` draft) — **CLOSED**

Task 5 Step 1's block is `mode=whole` against `crates/ms-cli/tests/hashlock_kind.rs`
and the gate reports `(217 lines, identical)`. `grep -n '\-\-phrase' <plan>` → two
hits, both in the retraction prose at plan:966 and plan:970; none in a code block.
The plan also adds "**Five tests, not three**" at plan:1199 naming the fifth.

### Round-4 **M-1 / M-2** (CHANGELOG structure) — **CLOSED**

The entries now sit **below** the preamble, in per-crate bracketed form
(`## ms-codec [0.10.0] — 2026-09-15` at :7, `## ms-cli [0.19.0] — 2026-09-15` at
:45), each ending in a `### Migration notes` section pointing at MIGRATION.md,
per `design/RELEASE_PROCESS.md` item 2 and the `## ms-cli [0.18.0]` precedent.
`grep -n "Unreleased"` → one hit, `CHANGELOG.md:802`, which is the file's legacy
bottom-of-file `## Unreleased` / `(none)` stub and not the `ms-cli [Unreleased]`
section M-2 was about. `--emit-record` is now recorded inside the 0.19.0 entry.

### The new gate is NOT vacuous — proven in both modes

`scripts/h2-plan-blocks-vs-tree.sh` failed on demand:

- **`mode=whole`**: appended ` // PERTURB` to one line inside the block at
  plan:635 →
  `FAIL … :635 whole crates/ms-codec/tests/hashlock_kat.rs -- differs from the tree file`, exit 1.
- **`mode=fragment`**: appended `x` to one line inside the block at plan:246 →
  `FAIL … :246 fragment … -- not a verbatim substring`, naming the exact line,
  `(1 of 61 block lines absent)`, exit 1.

Both reverted; the plan's SHA-256 is byte-identical to its pre-mutation value
(`0cb572db3b967ea9e55378fcbb4cdd153039aa832e98e1289489d91c499804d8`).

**Header coverage.** `grep -n '^```rust' <plan> | grep -v 'file='` → exactly one
hit, plan:764, which is the deliberate mutation block; the plan says so in prose
immediately above it and the script lists it under "NOT COVERED". So no *rust*
block silently lacks a header. One non-rust block does carry unverified file
content — see **M-3**.

### The two new/rewritten integration tests — mutation-proven, not vacuous

| mutation | test that reds | other 4 in the binary |
| --- | --- | --- |
| re-nest the fallback: `… && !args.no_engraving_card` | `without_a_kind_every_digest_is_listed_on_stderr` ("`["hashlock","--hashlock-phrase-stdin","--no-engraving-card"]`: stderr carries no sha256 digest line") | all pass |
| fallback lists `sha256` only | same test | all pass |
| every fallback row prints the **sha256** digest under its own token (right word, wrong width) | same test | all pass |
| JSON `if args.kind.is_none()` → `if false` | `under_json_the_object_carries_every_kind_when_none_was_named` (`left: Null / right: false`) | all pass |
| JSON block made unconditional (`if true`) | same test, at `hashlock_kind.rs:215` — so the *second* half (explicit `--kind` must carry neither key) is load-bearing too | all pass |
| `kind_line` → ` hash: <kind>` (appended to `method:`) | `qr_text_names_the_kind_on_its_own_line` (plus 2 pre-existing) | — |

The third mutation is the important one: it proves the rewritten test's
digest-and-width matching is what does the work, exactly as the fold's comment
claims. All six restored from backups and SHA-256-verified byte-identical.

---

## 2. Findings

### C-1 — BRANCH + PLAN — Critical — MIGRATION.md and Task 5b instruct the reader to do the thing §13.4 forbids: "a plate with no `hash:` line MEANS sha256"

`MIGRATION.md:150-151`, in the section this fold added to close round-4 I-1:

> **Reading an existing plate: no `hash:` line MEANS sha256.** Plates cut before
> v0.10 carry no such line, and that absence is the sha256 case, not an unknown.

and, prescribing that text, `IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md:1419`
(Task 5b Step 3, "Three things an upgrader cannot get from the CHANGELOG and must
get here", item 2):

> 2. **A plate with no `hash:` line MEANS sha256.** Absence is the sha256 case, not
>    an unknown. Every plate cut before this cycle is in that state.

**This is false, and it is the exact claim the whole round-4 Critical was about.**
The `hash:` line on a preimage plate records **which hash the SCRIPT commits to**
— not how the preimage was derived. Its absence records that *the tool never
asked*, which is an unknown, not sha256.

Everything else in the artifact says so:

- **Spec §13.4** (`design/SPEC_hashlock_kinds.md:630-635`): *"For plates already
  cut, printing all four digests when no kind is given turns an **impossible
  check** into a lookup. This spec requires the `--kind` flag and permits the
  four-digest fallback; **it must not silently assume sha256.**"* If absence
  meant sha256, the four-digest fallback would be pointless — you would print
  sha256 and stop. The fallback exists **because** absence is an unknown.
- **The code**, `crates/ms-cli/src/cmd/hashlock.rs:505-507`: *"a plate cut before
  `--kind` existed carries no kind, so listing all four turns an impossible check
  into a lookup."*
- **The test**, `crates/ms-cli/tests/hashlock_kind.rs:85`: *"never silently assume
  sha256. A plate cut before this existed carries no kind, so all four are listed
  and the operator matches."*

**The premise is false in fact, not only in wording.** Non-sha256 hashlock
wallets are buildable today and always were: `md-codec` carries all four
miniscript hash fragments (`crates/md-codec/src/{tree,render,tag,to_miniscript}.rs`
all reference `ripemd160`), and spec §13.5 records that *"`me bundle` emits
byte-identical six-plate output for a **sha256** card and a **`ripemd160`**
card"* — i.e. `ripemd160` cards exist now. So a preimage plate cut by v0.9 `ms`
can perfectly well belong to a `hash256`, `ripemd160` or `hash160` wallet. The
plate is silent about which; that silence is what §13.1 is fixing.

**Concrete failure.** An operator upgrades to v0.10, reads MIGRATION.md as the
release tells them to, and finds their old preimage plate has no `hash:` line.
MIGRATION says that means sha256. They run `ms hashlock --in plate.ms1 --kind
sha256`, get a 64-hex digest, compare it against the `ripemd160=<40 hex>` operand
in their descriptor, see a mismatch, and conclude per §13.2's own instruction
(*"if they differ, do not fund this wallet: build it again"*) that the wallet is
wrong — discarding a correct wallet and re-cutting five plates. In the composing
direction it is worse: they compose `--path ... sha256=<digest>` for a wallet
whose script commits to `ripemd160`, which nobody can spend. Had they instead run
with no `--kind`, the fallback this very fold fixed would have handed them all
four and the match would have been immediate.

This is an unmet guarantee: §13.4's clause is unconditional, and the fold's own
C-1 response commit message repeats the error in the same words (*"the one an
upgrader most needs: a plate with no `hash:` line MEANS sha256"*).

**Not to be confused with the true statement next to it.** MIGRATION §4 and
plan:97 / `hashlock_kind.rs:1007` say *"a **bare record** `hash:<hex>` MEANS
sha256"* — that is §6's producer rule, it is correct, and it is a statement about
the **record** a producer emits, not about a **plate** an operator holds. The
defect is the second sentence's extension of it to plates.

**Verified by:** reading `MIGRATION.md:145-156` and plan:1414-1421; reading spec
§13.4 (lines 630-635) and §13.5; `grep -n "MEANS sha256" MIGRATION.md <plan>`;
`grep -rn "never silently assume\|impossible check into a lookup"` over
`crates/ms-cli/`; `grep -rln "ripemd160" --include=*.rs descriptor-mnemonic/crates`.

**Fix shape:** delete both sentences, or replace with the true one — *"a plate
with no `hash:` line does not say which hash the script commits to; run
`ms hashlock` with no `--kind` and match the digest against your descriptor
(§13.4)."*

---

### I-1 — BRANCH — Important — the `!args.json` guard is broader than the purity contract it cites, and under `--json` with the card the human is shown `sha256=` with no notice at all

`crates/ms-cli/src/cmd/hashlock.rs:502` gates the §13.4 listing on
`args.kind.is_none() && !args.json`. Its own comment gives the reason:

> `--json --no-engraving-card` pins stderr to exactly the advisory (§4.4, §11),
> and that purity contract is load-bearing for machine consumers.

The contract is on **`--json --no-engraving-card`**, both flags. The guard drops
the listing under `--json` **alone** — the default, card-on shape — which the
contract does not cover. `hashlock_outputs.rs` says so at its own test:
*"`--no-engraving-card` on the first invocation is LOAD-BEARING, not tidiness"*,
precisely because the card otherwise sits on stderr.

**The operator's whole terminal for `ms hashlock --hashlock-phrase-stdin --json > out.json`,
with no `--kind` given:**

```
THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries only the public digest.
digest:          3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
for md compose:  --path ... sha256=3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
preimage (ms1):  ms10h ashsq 0p7ja f9gsj jpkjv ll2l2 74w8a 388xg qzlew p73sc ptwxg tjugs pvs8t klufg 89hqj
… 7 more card lines …
warning: stdout carries private key material (can spend) — redirect or encrypt
```

No line says a kind was not chosen. The `for md compose:` line — the line round
3's C-1 established as *"the ONLY line keeping the digest function and `md
compose`'s option name in agreement"* — says `sha256=`, and the §13.4 notice went
into the redirected file. This is the loss pattern the file's own module header
already names: *"`--json` is stdout, which `| jq` filters away — the constructed
loss of R0 r0 adversarial C-1"*. Redirecting `--json` to a file is the normal way
to use it.

**No test can see the difference.** `without_a_kind_every_digest_is_listed_on_stderr`
never passes `--json`; `under_json_the_object_carries_every_kind_when_none_was_named`
always passes `--no-engraving-card`. The `--json`-with-card cell of the matrix is
untested.

**Constructed proof that the narrower guard is correct and non-breaking.** I
changed the condition to `!(args.json && args.no_engraving_card)` — the exact
shape of the contract the comment cites — and ran the whole suite:

```
$ cargo nextest run --locked --all-targets
    Summary [0.252s] 577 tests run: 577 passed, 11 skipped
```

Green. So the current breadth buys nothing a test asserts, and costs the human
channel its §13.4 notice in the default `--json` shape. (Reverted; SHA-256
byte-identical.)

**Verified by:** reading `hashlock.rs:400-425` and `:502-540`; reading
`hashlock_outputs.rs`'s `json_both_variants` and its load-bearing comment;
running the binary with `--json` and `> file` and reading the terminal; the
narrowed-guard suite run above.

---

### M-1 — BRANCH — Minor — "NINE clippy errors in ms-codec" does not reproduce; measured **14**, and this fold added two of them

Three copies of the claim, all written or rewritten by this fold:
`crates/ms-codec/src/hashlock.rs:159` (*"Measured, that adds NINE clippy errors
in ms-codec and four in ms-cli"*), `CHANGELOG.md:38`, `MIGRATION.md:165`.

**Measured.** I built the counterfactual the sentence describes — kept the old
name `digest` as a `#[deprecated]` shim with the call sites still on it (sed
`digest_sha256` → `digest` across the six files that use it, plus the attribute)
— and counted:

```
$ cargo clippy -p ms-codec --all-targets --locked | grep -c '^warning: use of deprecated'
14
$ cargo clippy -p ms-codec              --locked | grep -c '^warning: use of deprecated'
1
$ cargo clippy -p ms-cli   --all-targets --locked | grep -c '^warning: use of deprecated'
4
```

The 14 are `src/hashlock.rs:105,189,215`, `tests/hashlock_derivation.rs:7,50,53,116,123,143,159`,
`tests/hashlock_kat.rs:13,56`, `tests/hashlock_repro.rs:19,113`. `ms-cli`'s
**four** is right. `ms-codec`'s **nine** is not right under any flag combination I
could produce: 14 with `--all-targets` (the flag the claim cites), 1 without.

Note the mechanism: two of those 14 are `src/hashlock.rs:189` and `:215` — the
**new unit tests this same fold added**. Whatever the number once was, the fold
that restated it also invalidated it. (Counting with `-D warnings` is no help:
the first error aborts the crate, so it reports 1, not N.)

Nothing turns on the number — the decision not to ship an alias is right either
way — so this is a Minor. It is logged because the claim now lives in three
documents and one of them is a released CHANGELOG.

---

### M-2 — BRANCH — Minor — "a machine-readable contract the GUI consumes" has no consumer

`MIGRATION.md:174` (new) and `CHANGELOG.md:84` (pre-existing at `1a723eb`, so
propagated rather than invented by this fold) justify the `sha256_operand` →
`hash_operand` rename as breaking *"a machine-readable contract the GUI consumes"*.

```
$ grep -rn "sha256_operand\|hash_operand" --include=*.go --include=*.rs \
      /scratch/code/shibboleth   # excluding mnemonic-secret and its worktrees
(no output)
```

Neither the SeedHammer fork nor `mnemonic-engrave` reads either key; the only
occurrences anywhere are inside `mnemonic-secret` itself. The rename is still
correct and still breaking for any future consumer — only the stated reason is
unsupported. Minor; the fold should either name the real consumer or drop the
clause.

---

### M-3 — PLAN — Minor — the gate's "0 FAIL" excludes a block that carries file content and would fail

Plan:319 is a ```` ```toml ```` block prescribing an addition to
`crates/ms-codec/Cargo.toml`:

```toml
# ripemd160: the bare primitive for the `ripemd160` fragment. `hash160` is
# ripemd160(sha256(x)) and needs the same crate. RustCrypto, to match sha2.
ripemd = "0.1"
```

The shipped file has no such comment — `crates/ms-codec/Cargo.toml:19-22` is a
different comment about pbkdf2/sha2 followed by a bare `ripemd = "0.1"`. The
block has no `file=` header, so the gate lists it as uncovered rather than
checking it. **Demonstrated:** adding
`file=crates/ms-codec/Cargo.toml mode=fragment` to that fence turns the run into
`12 blocks checked, 1 FAIL`, naming *"first line absent from the file: '# ripemd160: the bare primitive …' (2 of 3 block lines absent)"*. Reverted;
plan SHA-256 byte-identical.

The gate is honest about this (it prints every unheadered fence), and the plan's
Global Constraint at line 15 — *"Every command, error, count and output below is
transcribed from that run, not predicted"* — is what the block misses. Two lines
of prescribed file content that never shipped; harmless to an executor, but it is
the one block in the plan that could be gated and is not.

---

### M-4 — PLAN — Minor — the "tests are the task" table still lists four tests; the branch has five

Plan:120 introduces the table as *"The tests below are
`crates/ms-cli/tests/hashlock_kind.rs` on the branch, and each one went RED
before its fix"*, and plan:124-129 lists four rows. The branch's file now has
**five** tests — this fold added `under_json_the_object_carries_every_kind_when_none_was_named`
in the same round and did not add the row.

Round 4's I-6 was this exact shape (two prescriptions for one file) and this is a
weaker instance of it: the authoritative Task 5 Step 1 block is `mode=whole` and
does carry all five, and plan:1199 says "**Five tests, not three**" and names the
fifth. So an executor is not misled about what to write, only about what the
front-matter table claims to enumerate. Minor rather than Important for that
reason. `git diff` falsifying prose it never touches — the class the repo's own
notes name.

---

### N-1 — Nit — `c9a17d4`'s message says "Every ```rust block now carries a `file=`/`mode=` header"

One does not: plan:764, the deliberate mutation block. The **plan itself** is
correct and explicit about the exception (*"This block is a MUTATION and is
deliberately absent from the tree, so it carries no `file=` header"*) and the gate
lists it. Only the commit message overstates. No action needed beyond not
repeating the phrasing.

### N-2 — Nit — the §13.4 stderr block says "This phrase's digest under each kind" on sources that have no phrase

Under `--hex`, `--in <plate.ms1>` and `--random` the listing still reads *"This
**phrase's** digest under each kind:"*. The string is unchanged by this fold (it
was moved verbatim), so this is pre-existing, but the move made it reachable in
more shapes — notably `--random --no-engraving-card`, where nothing else on
stderr mentions a phrase at all. Wording only.

---

## 3. Answers to the brief's four sceptical questions

1. **The new JSON channel.** `digests_by_kind` and `kind_specified` appear under
   exactly the argv where `--kind` is omitted and `--json` is given, and never
   otherwise — verified across 14 argv shapes and four sources. `kind` is always
   present. Stderr purity under `--json --no-engraving-card` is exactly the one
   advisory line in every combination, `--emit-record` and `--random` included.
   **There is no path where §13.4 is silent on both channels.** There is one
   where it is silent on the channel the human reads — **I-1**, Important, not
   Critical, because the notice does travel.
2. **Can the gate fail?** Yes, in both modes, demonstrated with single-character
   perturbations and byte-identical reverts. No *rust* block silently lacks a
   header (the only unheadered one is the intentional mutation, named in prose).
   One `toml` block carries file content and is ungated — **M-3**, with the FAIL
   demonstrated by adding the header.
3. **The five new/changed tests.** All mutation-proven; **none vacuous**. Six
   mutations, each killing exactly the intended test with the other tests in its
   binary still passing. The rewritten stderr test's digest-and-width matching is
   load-bearing (a right-token/wrong-width mutation reds it), and the JSON test's
   *second* half — explicit `--kind` carries neither key — is load-bearing too
   (making the block unconditional reds it at `hashlock_kind.rs:215`).
4. **MIGRATION.md and Task 5b against reality.** The `qr_text` signature, the
   unconditional `hash:` line, 194→210 / 53 modules, the `=0.9.0` → `=0.10.0` pin
   story, the one-added-vendor-directory claim, the oldest-first append, and the
   CHANGELOG's conformance to `RELEASE_PROCESS.md` item 2 all check out.
   **"A plate with no `hash:` line MEANS sha256" does not** — it is false of the
   code, of the spec, and of the world (**C-1**). "Nine clippy errors" does not
   (**M-1**). "The GUI consumes" does not (**M-2**).

---

## 4. Counts and verdict

| severity | branch | plan | total |
| --- | --- | --- | --- |
| Critical | 1 | (same finding) | **1** |
| Important | 1 | 0 | **1** |
| Minor | 2 | 2 | **4** |
| Nit | 1 | 1 | **2** |

- **Critical:** C-1 — MIGRATION.md:150-151 and plan:1419 tell an upgrader that a
  plate with no `hash:` line means sha256, which is the silent assumption §13.4
  forbids and the premise the whole cycle disproves. One finding, present in both
  artifacts because the plan prescribes the branch's text.
- **Important:** I-1 — `hashlock.rs:502`'s `!args.json` is broader than the
  purity contract it cites; under `--json` with the card (the default) the
  operator sees `for md compose: … sha256=` and no §13.4 notice. The narrower
  guard is green at 577/577.
- **Minor:** M-1 (nine-vs-14 clippy errors), M-2 (no GUI consumer), M-3 (the
  ungated `toml` block that would fail), M-4 (four-row table, five tests).
- **Nit:** N-1 (commit-message overstatement), N-2 ("this phrase's digest" with
  no phrase).

### Verdict: **NOT GREEN (1C / 1I)**

Every round-4 finding is closed, and closed well: the gate the fold built is a
real gate — I made it fail twice — the five new or rewritten tests are all
mutation-proven, every measured number in both fold commit messages reproduces,
and the branch is genuinely green at 101 suites / 577 tests / clippy 0 / fmt / vendor.

The residue is one shape, twice. The fold's engineering is sound and its **prose
is where the defects are**. C-1 is a sentence in the document the fold added to
close a finding, asserting the exact proposition the finding existed to refute —
and it reached the plan, the branch, and a commit message without any gate that
could see it, because no gate reads prose. M-1 is a measured number restated into
three documents by the same commit that invalidated it. Both are the class this
repo's own notes name: *records are the weak half*, and *a diff falsifies text it
never touches*. Neither is a design problem, and both are small edits.

---

## 5. Worktree state

Left exactly as found, in both repos.

```
$ git -C /scratch/code/shibboleth/ms-worktrees/hashkinds-p2 status --porcelain   # (empty)
$ git -C /scratch/code/shibboleth/ms-worktrees/hashkinds-p2 branch --show-current
hashkinds-p2                                                                    # @ c9a17d4
$ sha256sum crates/ms-cli/src/cmd/hashlock.rs crates/ms-codec/src/hashlock.rs \
            crates/ms-codec/tests/hashlock_qr_text.rs crates/ms-cli/tests/hashlock_kind.rs \
            crates/ms-codec/tests/vectors/hashlock-v0.8.json
42444391745d799c592d0f3b7e4d3a5f82154d950fec06c187a08f0996fac3f5  …/cmd/hashlock.rs
eb60dbbb5fcc043932fe616914b7772e0795b14d7f5729865400f67dda265558  …/src/hashlock.rs
19723e533d14d8840972f6a27271894e79106b8cad21bd979ce4baa90effc5d1  …/hashlock_qr_text.rs
38312a5ee202a0948bf25530776350e7bd2cc23d61d206baafdc43730aaafaea  …/hashlock_kind.rs
0a911f78f3cdc867dcc44483b7f4c0c1ac87b6d9b30b79f52094e8979bc3d8ce  …/hashlock-v0.8.json

$ git -C /scratch/code/shibboleth/mnemonic-engrave status --porcelain            # (empty)
$ sha256sum design/IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md
0cb572db3b967ea9e55378fcbb4cdd153039aa832e98e1289489d91c499804d8
```

Eleven mutations were applied in total — six test-kill mutations, two plan-gate
perturbations, one deprecated-alias counterfactual spanning six files, one
guard-narrowing experiment, one `file=` header addition — and every one was
reverted from a backup outside both repos (`/tmp/r5bk`, `/tmp/r5bk2`) and
verified with `sha256sum -c`. One temporary file was written to `/tmp` only.
Nothing was committed or pushed. The `mnemonic-engrave` HEAD moved from
`8da7c008` to `ceb4524d` during this review (an unrelated phase-1 recon doc); the
plan under review is untouched by it (`git diff --stat 8da7c008..HEAD -- <plan>`
is empty).
