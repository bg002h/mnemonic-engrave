# R0 round 6 — Hashkinds Phase 2 fold verification (mnemonic-secret)

**Scope: two questions only.** Did the fold close each round-5 finding, and did
the fold introduce a new defect? Not a fresh audit. The spec, phases 1/3/4, and
everything closed in rounds 1–4 were not re-reviewed. Round 5's settled facts —
the non-vacuity of the `hashlock_kind.rs` tests, the transcript gate's ability to
fail, the 14-row argv matrix for its branch C-1, and the 14/1/4 deprecation
counterfactual — were taken as given and not re-derived.

**Artifacts.**

1. **BRANCH** `hashkinds-p2` @ `dabd518` in `/scratch/code/shibboleth/ms-worktrees/hashkinds-p2`.
   Fold = `c9a17d4..dabd518` (`cfdb738`, `dabd518`).
2. **PLAN** `design/IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md` in
   `mnemonic-engrave`. Fold = `fc8bb659`.

**Toolchain.** `export PATH=/home/bcg/.cargo/bin:$PATH` inside the worktree →
`cargo 1.85.0 (d73d2caf9 2024-12-31)` / `rustc 1.85.0`, matching
`rust-toolchain.toml`. (Outside the worktree the same shim reports
`1.97.0-nightly` — the pin is cwd-resolved, so every measurement below was taken
from inside the worktree.) The system 1.98.0 phantom-clippy trap was avoided.

---

## 0. The green claims — independently re-measured

One capture per suite, greped twice.

| gate | claimed | measured | verdict |
| --- | --- | --- | --- |
| `cargo test --workspace --locked` | 101 suites / 0 failures | exit 0; `grep -c '^test result:'` → **101**; non-`ok` result lines → **0**; `FAILED` → **0** | **TRUE** |
| `cargo nextest run --locked --all-targets` | 578 tests | `578 tests run: 578 passed, 11 skipped`, exit 0 | **TRUE** |
| `clippy -p ms-codec --all-targets --locked -- -D warnings` | 0 | exit 0, output is two `Checking`/`Finished` lines only | **TRUE** |
| `clippy -p ms-cli --all-targets --locked -- -D warnings` | 0 | exit 0, clean | **TRUE** |
| `cargo +1.95.0 fmt --all -- --check` | clean | exit 0, zero bytes of output | **TRUE** |
| `ci/repro/vendor-freshness.sh` | OK | `vendor-freshness: OK — vendor/ satisfies Cargo.lock.` | **TRUE** |

**The four plan gates, against `fc8bb659`'s message.**

| gate | message says | I measured |
| --- | --- | --- |
| `scripts/h2-plan-blocks-vs-tree.sh <plan> <worktree>` | 12 blocks, 0 FAIL | **`12 blocks checked, 0 FAIL`**, exit 0 |
| `scripts/plan-cite-check.sh <plan>` | 11 / 11 ; dangling 0 ; ambiguous 0 | **`citations resolved: 11 / 11 ; dangling: 0 ; ambiguous: 0`**, exit 0 |
| `scripts/plan-glyph-check.sh <plan>` | 49 strings ; 0 undrawable | **`operator strings scanned: 49 ; undrawable: 0`**, exit 0 |
| `scripts/plan-table-check.sh <plan>` | 28 rows ; 0 malformed | **`table rows checked: 28 ; malformed: 0`**, exit 0 |

The block gate now covers the `toml` fence (`plan:326 → crates/ms-codec/Cargo.toml`,
3 lines, verbatim substring) — round 5's M-3 — and the `mode=whole` block at
`plan:990` reports `crates/ms-cli/tests/hashlock_kind.rs (263 lines, identical)`,
so the plan's copy of the six-test file is byte-for-byte the branch's.

**Worktree state.** `git status --porcelain` is empty in both repos; branch tip
`dabd518`, plan tip `fc8bb659`. Every mutation below was reverted and the
restored `hashlock.rs` re-hashed against `git show HEAD:` — identical
(`4f8e8a9d…86da`).

---

## 1. Findings CLOSED — stated so a later round does not re-derive them

### Round-5 **C-1** (both artifacts — "a plate with no `hash:` line MEANS sha256") — **CLOSED**

Both copies replaced. `MIGRATION.md:150-170` and
`IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md:1495-1508` now say the kind
is **UNKNOWN**, give the §13.5/§7.4 evidence, give the §13.4 remedy, and carry
the "do not write the RECORD rule onto a PLATE" warning.

Every factual claim in the new prose was checked against the source, not the
report:

| new claim | checked how | verdict |
| --- | --- | --- |
| §13.5 — *"`me bundle` already emits byte-identical six-plate output for a sha256 card and a `ripemd160` card"* | `SPEC_hashlock_kinds.md:637-639` reads *"`me bundle` emits byte-identical six-plate output for a sha256 card and a `ripemd160` card, names no preimage plate, and cannot cut one at all"* | **accurate** |
| §7.4 — *"a decoded `ripemd160` card already shows \"hashlock\" on the device, with no digest"* | `SPEC_hashlock_kinds.md:302-304` reads *"…while setting `Hashlock = true` for all four, so a decoded `ripemd160` card already shows \"hashlock\" with no digest"* | **accurate** |
| *"`md-codec` carries all four miniscript hash fragments"* | `descriptor-mnemonic/crates/md-codec/src/{tag,render,tree,to_miniscript}.rs` all match `ripemd160`; string literals `"sha256"`, `"hash256"`, `"ripemd160"`, `"hash160"` all present | **accurate** |
| the remedy — *"Run `ms hashlock --in <plate>` with no `--kind`: it prints … under all four"* | **RUN**, on an ms1-sourced preimage, not a phrase one | **accurate — see below** |

```
$ ./target/debug/ms hashlock --random --out $D/plate.ms1 --no-engraving-card
$ ./target/debug/ms hashlock --in $D/plate.ms1
--- STDOUT:
hash:5d90931a20158d289d601b5441efa34af6b3ae7c790966c9a98db36045a7f7a6
--- STDERR (tail):
source:          preimage supplied (ms1 plate)
no --kind given; stdout carries the sha256 record. This phrase's digest under each kind:
  sha256     5d90931a20158d289d601b5441efa34af6b3ae7c790966c9a98db36045a7f7a6
  hash256    e051a0b806ab58e6c6e76f41188e07b862d319a326c0d7ef335f7f57b0ccdc33
  ripemd160  8f6c73f781cf8b58056ae907d1f218a911bacc31
  hash160    7fa126030b98716346201ca4e100b476db859f23
```

All four, correct widths, from an **ms1 plate** source — the case the MIGRATION
passage is actually about. The remedy works as written.

**Residue sweep — is any sentence anywhere still telling a reader to assume
sha256 for a plate?** No.

```
$ grep -ni "means sha256|absence is the sha256|assume sha256|is the sha256 case" \
      MIGRATION.md CHANGELOG.md README.md <plan> SPEC_hashlock_kinds.md
MIGRATION.md:200  a consumer that previously read `sha256_operand` and assumed sha256 was correct by
MIGRATION.md:205  means sha256; every other kind is `hash:<kind>:<hex>`   <- §6 RECORD rule
<plan>:97         a bare record MEANS sha256, so                          <- §6 RECORD rule
<plan>:1018       A bare record MEANS sha256, so emitting one under       <- §6 RECORD rule
<plan>:1075       §13.4: never silently assume sha256.                    <- the prohibition
```

Every survivor is either §6's producer rule about a **record** (true) or the
§13.4 prohibition itself. `grep -n "hash: line|cut before|pre-v0.10"` finds only
the corrected passages. Nothing remains that extends the record rule to a plate.

### Round-5 **I-1** (branch — `!args.json` broader than the contract it cites) — **CLOSED**

`crates/ms-cli/src/cmd/hashlock.rs:505` is now
`if args.kind.is_none() && !(args.json && args.no_engraving_card) {`.

**The full flag matrix, re-enumerated and run** — `--json` × `--no-engraving-card`
× `--kind` × `--emit-record` × each of the five sources. `se_list` = the §13.4
stderr listing; `se_card` = the engraving card (`for md compose:`); `json_dbk` /
`json_ks` = `digests_by_kind` / `kind_specified` on the object.

```
=== KIND OMITTED ===
phrase                            se_list=yes se_card=yes json_dbk=n/a json_ks=n/a errlines=15
phrase --ncard                    se_list=yes se_card=no  json_dbk=n/a json_ks=n/a errlines=5
phrase --json                     se_list=yes se_card=yes json_dbk=yes json_ks=yes errlines=16
phrase --json --ncard             se_list=no  se_card=no  json_dbk=yes json_ks=yes errlines=1
phrase --emit-record              se_list=yes se_card=yes json_dbk=n/a json_ks=n/a errlines=17
phrase --emit-record --json       se_list=yes se_card=yes json_dbk=yes json_ks=yes errlines=18
phrase --emit-rec --json --ncard  se_list=no  se_card=no  json_dbk=yes json_ks=yes errlines=1
hex -                             se_list=yes se_card=yes json_dbk=n/a json_ks=n/a errlines=15
hex - --ncard                     se_list=yes se_card=no  json_dbk=n/a json_ks=n/a errlines=5
hex - --json                      se_list=yes se_card=yes json_dbk=yes json_ks=yes errlines=16
hex - --json --ncard              se_list=no  se_card=no  json_dbk=yes json_ks=yes errlines=1
in plate                          se_list=yes se_card=yes json_dbk=n/a json_ks=n/a errlines=14
in plate --ncard                  se_list=yes se_card=no  json_dbk=n/a json_ks=n/a errlines=5
in plate --json                   se_list=yes se_card=yes json_dbk=yes json_ks=yes errlines=15
in plate --json --ncard           se_list=no  se_card=no  json_dbk=yes json_ks=yes errlines=1
random                            se_list=yes se_card=yes json_dbk=n/a json_ks=n/a errlines=15
random --ncard                    se_list=yes se_card=no  json_dbk=n/a json_ks=n/a errlines=5
random --json                     se_list=yes se_card=yes json_dbk=yes json_ks=yes errlines=16
random --json --ncard             se_list=no  se_card=no  json_dbk=yes json_ks=yes errlines=1
=== KIND GIVEN (--kind ripemd160) ===
phrase --kind                     se_list=no  se_card=yes json_dbk=n/a json_ks=n/a errlines=10
phrase --kind --json              se_list=no  se_card=yes json_dbk=NO  json_ks=NO  errlines=11
phrase --kind --json --ncard      se_list=no  se_card=no  json_dbk=NO  json_ks=NO  errlines=1
phrase --kind --ncard             se_list=no  se_card=no  json_dbk=n/a json_ks=n/a errlines=0
```

Answering the brief's three sub-questions directly:

- **Is §13.4 satisfied on a channel the relevant audience reads, in every
  kind-omitted cell?** Yes, with no gaps. Nineteen kind-omitted cells. In the
  fifteen where the card prints, the human gets the stderr listing. In the four
  `--json --no-engraving-card` cells the audience is a machine and the notice is
  in the object — and stderr is `errlines=1`, exactly the
  `PrivateKeyMaterial` advisory, so round 3's purity contract still holds
  byte-for-byte. There is no cell where the notice is absent from both channels.
- **Does it ever appear twice?** In the four `--json` card-on cells it appears on
  stderr (human prose) and on stdout (JSON). That is two *different* channels
  with two *different* audiences, which is what the plan's new three-row matrix
  prescribes, not a duplicate on one channel. In the normal shape
  (`--json > out.json`) the operator sees each exactly once.
- **Does it ever appear on a channel the audience does not read?** No. The one
  cell where it rides only on stdout is `--json --no-engraving-card`, whose whole
  point is a machine reader; and even a human running that without redirecting
  sees `kind_specified: false` in the terminal.

Kind-given cells are clean in both directions: no listing, and no
`digests_by_kind` / `kind_specified` keys. The guard is exactly right, not merely
less wrong.

### The new test is real — mutation-proven independently

`crates/ms-cli/tests/hashlock_kind.rs:233`
`under_json_with_the_card_the_human_still_gets_the_listing`. Three mutations, each
applied to a copy, run, then reverted:

| mutation | result |
| --- | --- |
| restore the round-5 guard `!args.json` at `hashlock.rs:505` | **FAIL** `under_json_with_the_card_the_human_still_gets_the_listing` at `hashlock_kind.rs:256`, message *"--json with the card: stderr carries no sha256 digest line, so the human reading `for md compose: sha256=` has nothing telling them a kind was never chosen"* — **the other 5 in the binary all PASS** |
| suppress the card under `--json` (`hashlock.rs:450` → `!args.no_engraving_card && !args.json`) | **FAIL** at `hashlock_kind.rs:235` with the **precondition's own message**: *"precondition: the card must still be on stderr here, or this test is asserting nothing"* — the other 5 PASS |
| every listed row prints the **sha256** digest under its own token (right word, wrong width) | **FAIL** — both `under_json_with_the_card…` and `without_a_kind_every_digest_is_listed_on_stderr`; the other 4 PASS |

So: the test reds for its own cell and nothing else's; its `hexlen` check is
load-bearing (mutation 3, the "right word, wrong width" class); and the
precondition is **meaningful** — it is not decoration, it fires with its own
diagnostic exactly when the test's premise ("the human is reading stderr here")
stops holding. It is the only test in the file that covers the `--json`-with-card
cell: mutation 1 reds it alone.

### Round-5 **M-1** (the "NINE clippy errors" claim) — **CLOSED**

All three copies now agree with each other and with round 5's measurement:

- `CHANGELOG.md:38-41` — *"measured at 14 in `ms-codec` and 4 in `ms-cli` with `--all-targets`"*
- `MIGRATION.md:183-185` — same sentence
- `crates/ms-codec/src/hashlock.rs:158-166` — *"`clippy -p ms-codec --all-targets` reports 14, `-p ms-cli --all-targets` 4. Without `--all-targets` ms-codec reports 1"* plus the retraction of "nine"

**No fourth copy.** `grep -rn "deprecated alias|NINE|nine clippy" --include=*.md
--include=*.rs --include=*.toml .` over the branch (minus `vendor/`, `target/`)
returns exactly those three sites. The word "nine" no longer occurs in any
first-party file.

### Round-5 **M-3** (the `toml` block the gate could not see) — **CLOSED**

`plan:323` now reads ```` ```toml file=crates/ms-codec/Cargo.toml mode=fragment ````,
and `dabd518` added the two prescribed comment lines to
`crates/ms-codec/Cargo.toml:22-23`. The gate went 11 → **12 blocks checked, 0 FAIL**,
with `plan:326 fragment crates/ms-codec/Cargo.toml (3 lines, verbatim substring)`
now listed as PASS. The fold headered the block **first**, let it FAIL, and fixed
toward the tree — which is the right order and is what the commit message says
happened.

### Round-5 **M-4** (the four-test table) — **CLOSED**

`plan:124-131` now has six rows. Measured against the branch:
`grep -c '^#\[test\]' crates/ms-cli/tests/hashlock_kind.rs` → **6**, and the six
`fn` names match the six table rows exactly. The `mode=whole` gate confirms the
plan's embedded copy is byte-identical (263 lines). `plan:148` moved 577 → **578**,
matching the measured nextest total. No "Five tests" / "four tests" / "577"
residue anywhere in the plan.

The row rewritten for test 3 — *"**C-2** — the silent sha256 assumption, on BOTH
argv shapes"* — is accurate: `hashlock_kind.rs:95-97` loops over
`["hashlock","--hashlock-phrase-stdin"]` and
`["hashlock","--hashlock-phrase-stdin","--no-engraving-card"]`.

### Round-5 **M-2** — **NOT closed.** See M-1 below.

---

## 2. Findings

### M-1 — BRANCH + PLAN — Minor — round-5 M-2 is not closed in `MIGRATION.md`; the edit left a sentence that does not parse, the unsupported "GUI" attribution still stands, and a fourth copy was never touched

`MIGRATION.md:191-196`, as shipped at `dabd518`:

```
The value names the chosen kind (`ripemd160=<hex>`). The old key is simply wrong
for three of the four kinds. This is a machine-readable contract the GUI
so a consumer reading the old key gets `None`, not a stale value. (No code in
this constellation reads either key today — the rename is breaking for future
consumers, not for a known one.)
```

The word `consumes,` lived at the start of the next source line and was deleted
with it; **`the GUI` was left behind**. Two consequences, and the second is the
one that matters:

1. The sentence is ungrammatical in a released operator-facing document.
2. **The finding is not fixed.** M-2's complaint was the unsupported claim that
   the GUI consumes this key. `grep -n "machine-readable contract the GUI"
   MIGRATION.md` → `193`. It is still there, now with the parenthetical
   immediately contradicting it.

`cfdb738`'s message says: *"M-2: … Corrected in both copies."* The CHANGELOG copy
(`CHANGELOG.md:84-86`) **was** correctly rewritten to *"It is a machine-readable
contract, though no code in this constellation reads either key today."* The
MIGRATION copy was not. A future reviewer running `git diff <report>..<fold>`
would read the message and conclude M-2 closed.

**A fourth copy exists and was never in scope.** Round 5 named two copies; there
are three:

```
$ grep -n "GUI" <plan>
210:   wrong for three of four kinds — but it is a GUI-facing contract and belongs in
```

`plan:208-211` still calls `hash_operand` *"a GUI-facing contract"*. It is not:
`ms gui-schema` mirrors the **flag** surface (`crates/ms-cli/src/cmd/gui_schema.rs`),
not `hashlock --json`'s output keys, and round 5's constellation-wide grep found
no Go or Rust reader of either key.

**Failure scenario.** An operator migrating reads §3 of MIGRATION.md, hits a
sentence that stops mid-clause, and cannot tell whether a clause was lost or a
requirement was. Nothing funds-bearing turns on it — the next paragraph
(`MIGRATION.md:198-202`) states the actual obligation correctly ("must now check
`kind`") — which is why this is Minor rather than Important, matching round 5's
own rating of M-2.

**Verified by:** `sed -n '191,197p' MIGRATION.md`; `git show cfdb738 -- MIGRATION.md`
(the diff hunk shows `consumes,` removed and `the GUI` retained);
`grep -rn "GUI" MIGRATION.md CHANGELOG.md <plan>`; `git log -1 --format=%B cfdb738`.

**Fix shape:** one line — *"This is a machine-readable contract, so a consumer
reading the old key gets `None`, not a stale value."* — and drop "GUI-facing" at
`plan:210`. Wording only; per the repo's proportional re-review rule this does
not re-trigger a gate.

---

### M-2 — BRANCH + PLAN — Minor — the file still carries the round-5 I-1 claim, 111 lines above the comment that retracts it, and the block gate structurally cannot see it

`crates/ms-cli/src/cmd/hashlock.rs:404-410`, untouched by the fold:

```rust
        if args.kind.is_none() {
            // SPEC §13.4 for MACHINE consumers. The stderr listing below is
            // suppressed under --json because `--json --no-engraving-card`
            // pins stderr to exactly the advisory (§4.4, §11), so the notice
            // has to travel in the object or it does not travel at all.
```

*"The stderr listing below is suppressed under `--json`"* is **now false** —
measured above, `--json` alone prints the listing in all five source shapes. It is
the exact conflation round-5 I-1 identified: a behaviour stated on one flag when
the contract it cites is on two. The fold corrected the *other* copy of this
comment, at `:517-521` (*"The guard is `!(json && no_engraving_card)` — BOTH flags"*),
so the file now contradicts itself across 111 lines.

**Why the gate cannot catch it.** The plan carries the same paragraph at
`plan:1354-1356`, inside the `mode=fragment` block at `plan:1351`. The gate
reports that block **PASS** — because plan and tree agree. `h2-plan-blocks-vs-tree.sh`
proves text *equality*, never text *truth*, and it says so in its own uncovered
list (*"every PROSE claim … whether the tree is GREEN"*). Two copies of a false
statement pass a gate that one copy would also pass.

**Failure scenario.** A maintainer touching the JSON path reads `:406`, takes
"suppressed under --json" as the rule, and re-broadens the guard — the round-5
defect, refolded. The blast radius is contained: the new test reds immediately
(mutation 1 above), which is why this is Minor and not Important. But the
comment is the thing a maintainer reads *before* running the suite, and `:517`
explicitly says *"Do not fold it back in"* — an instruction the earlier comment
undercuts.

**Verified by:** `grep -rn "suppressed under --json" --include=*.rs --include=*.md`
over the branch → one hit, `hashlock.rs:406`; over the plan → `plan:1355`;
`sed -n '404,412p'` and `sed -n '505,528p' crates/ms-cli/src/cmd/hashlock.rs`;
the 19-row matrix above showing `--json` alone prints `se_list=yes`.

**Fix shape:** rewrite `:405-407` to *"…suppressed only under `--json
--no-engraving-card` — BOTH flags — because that pair…"* and mirror it into
`plan:1354-1356` so the block gate stays PASS.

---

### M-3 — PLAN — Minor — `plan:133` says three tests were added by review rounds; measured two, and the next sentence in the same paragraph says two

`IMPLEMENTATION_PLAN_hashkinds_P2_mnemonic_secret.md:133-136`, new in this fold:

> **Six, and the last three were each added by a review round that found the
> defect first.** That is the measurement, not a rhetorical point: the file went
> 4 → 5 → 6 as rounds 4 and 5 each found a path the existing tests could not see.
> Every one was a flag combination nobody had passed.

"4 → 5 → 6" is **two** additions, not three — the sentence refutes itself one
clause later. Measured from the branch history:

```
$ for c in $(git log --format=%h --reverse -- crates/ms-cli/tests/hashlock_kind.rs); do
    echo "$c tests=$(git show $c:crates/ms-cli/tests/hashlock_kind.rs | grep -c '^#\[test\]')"; done
72acd1a  tests=4   hashlock: --kind gets the tests it shipped without, and both Criticals
8891bbf  tests=5   fold: R0 round 4 -- C-1 (--no-engraving-card assumed sha256), I-1, M-1
cfdb738  tests=6   fold: R0 round 5 -- the plate-vs-record conflation, and a guard wider …
```

Round 4 added `under_json_the_object_carries_every_kind_when_none_was_named`;
round 5 added `under_json_with_the_card_the_human_still_gets_the_listing`. The
third-from-last test, `an_uppercase_kind_is_refused`, shipped in the **original**
commit `72acd1a` and was found by no review round. The trailing sentence gives it
away too: *"Every one was a flag combination nobody had passed"* — an uppercase
`--kind` value is not a flag combination.

`fc8bb659`'s own commit message has it right (*"it went 4 -> 5 -> 6 as rounds 4
and 5 each found a flag combination"*), so this is a slip in the plan prose alone.

Minor: the table it introduces is correct, the count "Six" is correct, and an
executor writes the same six tests either way. Logged because the paragraph
explicitly bills itself as *"the measurement, not a rhetorical point"*, and the
repo's own rule is never to hand-count what a tool can count.

**Verified by:** the `git log --reverse` loop above and a per-commit dump of the
`fn` names; `grep -c '^#\[test\]'` → 6 at HEAD.

---

### N-1 — Nit — the new C-1 passage says "the phrase's digest" in the one paragraph that is about a plate with no phrase

`MIGRATION.md:161-162` (new prose): *"Run `ms hashlock --in <plate>` with **no**
`--kind`: it prints the **phrase's** digest under all four…"*. The passage is
about reading an **existing plate**, where the preimage was supplied and there is
no phrase at all — the run above reports `source: preimage supplied (ms1 plate)`
and `method: preimage supplied`.

This is round-5's N-2 (the stderr string *"This phrase's digest under each kind:"*
on phraseless sources), which the fold did not claim to fix — but the fold has now
copied the wording into prose where it is most visibly wrong. Wording only; fix
the string and the sentence together if N-2 is ever picked up.

---

## 3. Answers to the brief's four sceptical questions

1. **Is the replacement C-1 prose true?** Yes, every claim. §13.5 and §7.4 are
   characterised verbatim-accurately (quoted side by side in §1); `md-codec` does
   carry all four fragments; and the prescribed remedy was **run** against an
   ms1-plate source — not a phrase source — and printed all four digests at the
   correct widths. No sentence anywhere in either artifact, the CHANGELOG, the
   README or the spec still tells a reader to assume sha256 for a plate; the
   five surviving "sha256" assertions are all §6's record rule or §13.4's
   prohibition. One wording nit (N-1).
2. **Is the narrowed guard exactly right?** Yes. Nineteen kind-omitted cells and
   four kind-given cells enumerated and run. §13.4 is satisfied on a channel the
   relevant audience reads in **every** kind-omitted cell; the purity contract
   holds at `errlines=1` in all four `--json --no-engraving-card` cells; no cell
   duplicates the notice on one channel; no cell puts it only where its audience
   is not looking. Kind-given cells carry neither the listing nor the two JSON
   keys.
3. **Does the new test test what it names?** Yes, proven by three independent
   mutations, each reverted. It reds alone under the old guard; its width check
   is load-bearing; and its precondition fires with its own message when the
   premise breaks. It is the file's only coverage of the `--json`-with-card cell.
4. **The corrected claims.** 14/4 now agrees across all three copies plus the
   `1`-without-`--all-targets` half in `hashlock.rs`; no fourth copy exists and
   "nine" is gone from every first-party file. The GUI clause is the opposite
   story: correctly dropped in the CHANGELOG, **left standing and garbled** in
   MIGRATION.md, and a third copy at `plan:210` was never in scope (M-1).

---

## 4. Counts and verdict

| severity | branch | plan | total |
| --- | --- | --- | --- |
| Critical | 0 | 0 | **0** |
| Important | 0 | 0 | **0** |
| Minor | 2 (M-1, M-2) | 3 (M-1, M-2, M-3) | **3 findings** |
| Nit | 1 | 0 | **1** |

(M-1 and M-2 each span both artifacts; the row counts where each appears, the
total counts findings.)

Round-5 disposition: **C-1 closed. I-1 closed. M-1 closed. M-3 closed. M-4
closed. M-2 NOT closed** (carried as this round's M-1). Two new defects
introduced by the fold: the MIGRATION.md sentence damage (M-1) and the
plan's "last three" miscount (M-3). One stale statement the fold corrected in
one place and not the other (M-2).

### Verdict: **GREEN (0 Critical / 0 Important)**

The two Criticals-in-new-prose that rounds 4 and 5 each found in this position
did not recur: every factual claim in the replacement C-1 text checks out against
the spec, the sibling repo and the running binary. The guard is now exactly its
contract, proven over the whole matrix rather than the two cells that had tests.
All six green gates and all four plan gates reproduce the fold's stated numbers
exactly.

Three Minors and a Nit remain, all wording-level and all one-line fixes. **Fold
them inline; do not spend a round on them.** Per the repo's proportional
re-review rule a comment/wording fold does not re-trigger this gate. M-1 is the
one worth doing before ship rather than deferring, because the affected sentence
is in released operator documentation and does not parse — and because
`cfdb738`'s message asserts a correction that was not made, which is what a
future `git diff <report>..<fold>` reader would be misled by.

---

## 5. Worktree state

Both repos left clean and byte-identical to their tips.

```
$ git -C /scratch/code/shibboleth/ms-worktrees/hashkinds-p2 status --porcelain
$ git -C /scratch/code/shibboleth/ms-worktrees/hashkinds-p2 log --oneline -1
dabd518 hashlock: say why ripemd serves two of the four kinds

$ git -C /scratch/code/shibboleth/mnemonic-engrave status --porcelain
$ git -C /scratch/code/shibboleth/mnemonic-engrave log --oneline -1
fc8bb659 fold: plan R0 round 5 -- the plate-vs-record conflation, and the matrix behind it

$ sha256sum crates/ms-cli/src/cmd/hashlock.rs ; git show HEAD:crates/ms-cli/src/cmd/hashlock.rs | sha256sum
4f8e8a9da0f8d0282a1ad0110bbcf7b9de7be05650593ae558c90dded62386da  crates/ms-cli/src/cmd/hashlock.rs
4f8e8a9da0f8d0282a1ad0110bbcf7b9de7be05650593ae558c90dded62386da  -
```

Three mutations were applied to `hashlock.rs` during this review (guard restored
to `!args.json`; card suppressed under `--json`; sha256 digest under every
token). Each was reverted from a pre-mutation copy and the hash re-checked, as
above. Scratch fixtures were written under `/tmp/hkmatrix` only.
