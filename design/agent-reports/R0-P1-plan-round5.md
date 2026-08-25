# R0 — IMPLEMENTATION_PLAN_P1_me_container.md, round 5 (falsification lens)

**Artifact:** `design/IMPLEMENTATION_PLAN_P1_me_container.md` at `0409815` (v7).
**Lens, and only this lens:** *what did the v7 diff make UNTRUE somewhere it never
touched?* Diff surface: `git diff 0b3333f..0409815 -- design/IMPLEMENTATION_PLAN_P1_me_container.md`
(132 insertions, 44 deletions, one file).

Nothing in the FACTS-ALREADY-SETTLED list was re-derived as a finding. Every count
below was produced by a command, not read off the page.

---

## Commands run, and their raw output

### The three document gates (re-run on v7, not transcribed)

```
$ ./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
─── citations resolved: 90 / 107 ; dangling: 17 ; ambiguous: 0
EXIT=0
   (the 17 dangling are exactly 9 mnemonic-transaction + 8 vendored bitcoin — confirmed
    by reading the DANGLING lines; 0 elsewhere. Matches the settled fact.)

$ ./scripts/plan-table-check.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
─── table rows checked: 137 ; malformed: 0
```

### Fold sweep, re-derived exactly (my own, because shell word-splitting corrupts `--terms`)

```
$ python3 - <<'PY'
lines=open('design/IMPLEMENTATION_PLAN_P1_me_container.md').read().split('\n')
block=set(range(1499,1528))
terms=[l.split("'")[1] for i,l in enumerate(lines,1) if i in block and l.startswith("'")]
print("terms:",len(terms))
out=sum(1 for t in terms if [i for i,l in enumerate(lines,1) if t in l and i not in block])
print("terms surviving outside the block:",out)
PY
terms: 27
terms surviving outside the block: 0
```

Confirms the settled fact — **and confirms the sweep's blind spot for this round**:
the terms `'other twenty-two'`, `'TEN sites'` and `'W1–W10'` do not match the
surviving strings `the twenty-two below` (:1480), `ten sites` (:834) or
`FIVE are not` (:813). Every Minor below lives in exactly that gap.

### Re-derived counts (never read)

```
$ awk 'NR>=1001 && NR<=1030' <plan> | grep -oE '^\| \*{0,2}V[0-9]+[ab]?' | sed 's/[|* ]//g'
V1 V2 V3 V4a V4b V5 V6 V7 V8 V9 V10 V11 V12 V13 V14 V15 V16 V17 V17b V18 V19 V20
V21 V22 V23 V24 V25 V26 V27                                        -> 29 rows. v7's "29" is CORRECT.

$ awk 'NR>=1340 && NR<=1354' <plan> | grep -oE '^  \| \*{0,2}W[0-9]+'
W1 W2 W3 W4 W5 W14 W11 W12 W13 W8 W9 W10                           -> 12 rows, not 14. See [I3].

$ grep -n 'fourteen\|FOURTEEN' <plan>
765, 767, 807, 829, 1158, 1334, 1524
```

### The fixture experiment — [C1]'s construction, run against the real tree

Baseline, tree clean:

```
$ cargo test -p mnemonic-engrave --lib sysw::vectors
test sysw::vectors::tests::regenerate ... ignored, regenerates the fixture; run deliberately
test sysw::vectors::tests::every_required_vector_exists ... ok
test sysw::vectors::tests::the_implementation_still_matches_the_recorded_vectors ... ok
test sysw::vectors::tests::every_vector_opens ... ok
test result: ok. 6 passed; 0 failed; 1 ignored
```

Then I did exactly what §3.3 rules NORMATIVE — filed one P1 vector into
`crates/me-cli/testdata/sysw_vectors.json` (a well-formed entry, name `V1`, note
*"P1 V1: RAW (segwit), no optional fields — independently generated per plan
§3.2/§3.3"*) — and did **not** run the `regenerate` command §3.3 refuses:

```
$ cargo test -p mnemonic-engrave --lib sysw::vectors
test sysw::vectors::tests::every_required_vector_exists ... ok            <- W14's assertion #1: PASSES
test sysw::vectors::tests::every_vector_opens ... ok
test sysw::vectors::tests::the_implementation_still_matches_the_recorded_vectors ... FAILED

thread '...' panicked at crates/me-cli/src/sysw/vectors.rs:138:9:
assertion `left == right` failed: the container's output changed — if deliberate, regenerate

test result: FAILED. 5 passed; 1 failed; 1 ignored

$ cargo test -p mnemonic-engrave --lib sysw::coverage::tests::assert_every_named_test_is_placed
test sysw::coverage::tests::assert_every_named_test_is_placed ... ok      <- W14's assertion #2: PASSES
```

Fixture restored from backup; `git status --porcelain` clean; suite re-verified
green (`6 passed; 0 failed; 1 ignored`).

### The three assertions, read at source

```rust
// crates/me-cli/src/sysw/vectors.rs:61-73  -- generate()
pub fn generate() -> Vec<Vector> {
    VECTORS.iter().map(|v| {
        let blob = pack_deterministic(records.clone(), v.passphrase, ...)
            .expect("fixture inputs must pack");        // <- PANICS on a refusal input

// crates/me-cli/src/sysw/vectors.rs:135-143  -- NOT #[ignore]; runs in CI
/// THE golden check: the implementation still produces the recorded bytes.
#[test]
fn the_implementation_still_matches_the_recorded_vectors() {
    assert_eq!(generate(), load(), "the container's output changed — if deliberate, regenerate");
}

// crates/me-cli/src/sysw/vectors.rs:146-155
fn every_required_vector_exists() {
    for want in coverage::required_vectors() { assert!(have.iter().any(|h| h == want), ...) }
}

// crates/me-cli/src/sysw/coverage.rs:101-112  -- what required_vectors() actually returns
COVERAGE.iter().filter_map(|(_, w)| match w { Where::Vector(name) => Some(*name), _ => None })

// crates/me-cli/src/sysw/coverage.rs:228-240
fn assert_every_named_test_is_placed() {
    const HIGHEST: u32 = 23;                            // <- spec §8.3's test ids, not P1's vectors
    for id in 1..=HIGHEST { assert!(COVERAGE.iter().any(|(n, _)| *n == id), ...) }
    assert_eq!(COVERAGE.len() as u32, HIGHEST, "no duplicate or stray ids");
}
```

```
$ grep -oE 'Where::Vector\("[^"]+"\)' crates/me-cli/src/sysw/coverage.rs | sort -u
Where::Vector("S-B")  Where::Vector("S-C")  Where::Vector("S-D")
Where::Vector("S-E")  Where::Vector("S-J")

$ python3 -c "import json;d=json.load(open('crates/me-cli/testdata/sysw_vectors.json'));print(len(d),[v['name'] for v in d]);print(sorted(d[0]))"
8 ['S-A','S-B','S-C','S-D','S-E','S-G','S-I','S-J']
['blob','ct_len','digest','identity','mdmk_unconfirmed','name','note','passphrase','pub_len','records','sealed']
```

### Other source reads

```
$ grep -rn 'TX_PREFIX' crates/
(none)

$ sed -n '30,40p' crates/me-cli/src/sysw/record.rs
pub enum Class { Mnemonic, Codex32Secret, Passphrase, FreeText, Descriptor, MdMk, Address, Unknown }
                                                       -> 8 variants. §6's W2 row ("ten") is CORRECT.

$ sed -n '107,115p' crates/me-cli/src/sysw/mod.rs
fn unknown_reason(record: &str) -> UnknownReason {
    for prefix in [record::PASS_PREFIX, record::TEXT_PREFIX] {
        if record.starts_with(prefix) { return UnknownReason::NonHexBody(prefix); }
    }
    UnknownReason::Unrecognised
}
```

### The v6→v7 pairs the findings turn on

```
$ git show 0b3333f:<plan> | grep -n '^| 8 |'
1127:| 8 | ... | every rule in **E1–E19**, except the two §1.3 names as having no Rust RED test. **E20 is step 10's** |

$ grep -n '^| 8 |' <plan>
1192:| 8 | ... | every rule in **E1–E19 EXCEPT E12, E17 and E20** — **(r4-M1)** ... |

$ git diff 0b3333f..0409815 -- <plan> | grep -c 'not produced by the code they judge'
0                                        <- §6's §3.2 closure bullet: NOT TOUCHED by v7
```

---

## Part 1 — what the v7 diff falsified

## [C1] §3.3's NORMATIVE vector home turns a shipped, non-`#[ignore]` golden test RED — and NEITHER assertion W14 names can fail on anything P1 does. Both measured.

**Severity:** Critical.

**Where.**
*Touched by v7:* §3.3 in full (:1137-1176, new); §2.4's **W14** row (:789, new);
§4 **step 4**'s construction clause (:1188, new); §6's **W14** closure row (:1347,
new).
*Falsified:* `crates/me-cli/src/sysw/vectors.rs:137`
(`the_implementation_still_matches_the_recorded_vectors`) — a test v7 never
mentions; `crates/me-cli/src/sysw/vectors.rs:61-73` (`generate`'s
`.expect("fixture inputs must pack")`); `crates/me-cli/src/sysw/coverage.rs:238`
(`assert_eq!(COVERAGE.len(), HIGHEST)`); §4's own preamble (:1180-1181), *"minimal
code, full suite green"*.

**The failure, concretely.** An implementer reaches step 4. §3.3 is NORMATIVE:
*"P1's vectors extend `crates/me-cli/testdata/sysw_vectors.json`."* They append
V1. They obey §3.3's REFUSAL and do **not** run `regenerate`. Then §4's *"full
suite green"*:

```
test sysw::vectors::tests::the_implementation_still_matches_the_recorded_vectors ... FAILED
assertion `left == right` failed: the container's output changed — if deliberate, regenerate
```

Measured, above. **Step 4 cannot end green, and neither can 5, 6, 7, 8, 9, 10, 11
or 12** — every later step inherits the red fixture.

Three independent reasons, each fatal on its own:

1. **The file is a GOLDEN OF THE CODE UNDER TEST, and the golden check is not
   `--ignored`.** `the_implementation_still_matches_the_recorded_vectors` asserts
   `generate() == load()` on every CI run. `generate()` builds the fixture from
   `coverage::VECTORS` through `pack_deterministic` — `me`'s own encoder. So the
   file's contract is *the exact negation of §3.2*: not "these vectors were not
   produced by the code they judge" but "these vectors are byte-for-byte what the
   code produces." §3.3 refused the *command* (`regenerate`) and missed that the
   equality is enforced continuously by a test. **An independently generated
   vector that differs from `generate()` by one byte turns the suite red, and the
   only documented remedy is the command §3.3 forbids.** The implementer is in a
   trap with no exit the plan names.

2. **The schema cannot represent a refusal at all.** `generate()` calls
   `pack_deterministic(...).expect("fixture inputs must pack")` — it **panics** on
   any record set that does not pack. Counting §3's table, at least twenty of the
   29 vectors are records `me sysw pack` must REFUSE (V6, V8, V9, V10, V11, V12,
   V13, V14, V15, V16, V17, V17b, V18, V19, V20, V23, V24, V25, V27, and V7's
   `16,291 − F` near-miss). A `Vector` in this file is `{blob, pub_len, ct_len,
   sealed, digest, identity, mdmk_unconfirmed}` — the *output of a successful
   pack*. There is no field for "this is refused, with this message." §3.3 ruled
   the home without reading the schema.

3. **The two assertions v7 names as the gate are the wrong two, and both are
   vacuous for P1.** §2.4's W14 row states P1's 29 vectors *"cannot land in
   `testdata/sysw_vectors.json` without extending both."* Measured with a P1
   vector present, **both passed**:
   - `every_required_vector_exists` iterates `required_vectors()`, which is
     `COVERAGE`'s `Where::Vector(...)` entries — today exactly `S-B, S-C, S-D,
     S-E, S-J`. It asserts that vectors a **spec §8.3 test points at** exist.
     Adding 29 new vectors requires no extension and triggers no assertion.
   - `assert_every_named_test_is_placed` iterates spec §8.3 ids `1..=23` and ends
     `assert_eq!(COVERAGE.len() as u32, 23)`. "Extending" it — which is precisely
     what W14's row prescribes — makes it **FAIL**, because P1 adds no spec §8.3
     tests.

   So §6's W14 closure row (*"both pass with P1's 29 vectors present"*) **passes
   today, with zero P1 vectors present**. It is satisfiable without doing any of
   the work — the exact false-PASS shape §6 struck under r2-I4, reintroduced two
   bullets later.

**Why the fold permits it.** Round 4's I3 named `vectors.rs:147` and
`coverage.rs:230` as *"the two build-failing assertions"*. v7 transcribed the
reviewer's remedy instead of reproducing the defect: it cited the two line numbers
the report gave and never opened the file to see what they assert, nor what sits
at `:137` between them, nor what a `Vector` can hold. The plan's own standing rule
— *"never describe code from its doc comment, its name, or an earlier agent's
report"* — is the one this fold broke.

**Confidence:** Very high. The RED and both PASSes were produced by running the
real suite against the real fixture and restoring it (`git status --porcelain`
clean afterwards, suite re-verified green). The schema and the `.expect` are read
at source.

**Suggestion, not authoritative:** the remedy is a decision this fold cannot make
by editing a count — either P1's vectors get a *separate* fixture with a schema
that can hold a refusal (and §3.3's cross-language argument then needs a different
answer), or `sysw_vectors.json` gains a second, non-generated section that
`generate()`/`load()` do not compare. Reproduce the RED above before choosing.

---

## [C2] §2.4 gives W8 to step 6 and W11 to step 7 — W8 is *defined as* the arm for W11's type, and step 6's own test column carries two clauses the plan concedes to step 7. Step 6 cannot go green.

**Severity:** Critical.

**Where.**
*Touched by v7:* §2.4 :807-809, the new split (*"step 4 builds W14, step 6 builds
W1–W5 and W8–W9, step 7 builds W11–W13, and step 10 builds W10"*); §2.4's **W8**
row :783, redefined (*"gains the PER-RECORD arm for W11's `TxRecordError`"*); §4
step 6's *then* column :1190, rewritten from *"W1–W10"* to *"W1–W5 and W8–W9"*.
*Falsified:* §4 step 6's **test** column (:1190, its last two clauses carried
across unexamined) and §4's preamble (:1180-1181).

**The failure, concretely.** Step 6's *test* column ends:

> *"… a `tx:` record with a bad body names its PREFIX, not `Unrecognised`; a `tx:`
> record with a bad MAGIC names the MAGIC RULE, not "not lowercase hex""*

Its *then* column then says, of those same clauses: *"the last two clauses are
W11's and W13's, **built in step 7**."* So step 6 lists two tests, writes the
failing tests first as §4 requires, implements W1–W5 and W8–W9 — and the two tests
still fail, with no code left to write, because the plan assigns their
implementation to the next step. **§4's preamble requires the full suite green at
the end of every step. Step 6's gate cannot go green.** That is round 4's C2,
verbatim in shape, on a different step.

And it does not compile either, independently of the tests. W8 as v7 redefines it
is *"`sysw_error` gains the PER-RECORD arm for **W11's `TxRecordError`**"* — a
match arm over a type W11 introduces. W11 is `crates/me-cli/src/sysw/record.rs`
**(new)**, assigned to step 7. An implementer at step 6 is told to write a match
arm for a type that does not exist for another step. Confirmed absent from the
tree today: `grep -rn 'TX_PREFIX' crates/` returns nothing, so nothing pre-exists.

The dependency is stated in W8's own cell: *"The rule name reaches here from the
parse (W11), which is the only place that knows it."* v7 wrote that sentence and
then put the two sites in different steps.

**Why the fold permits it.** v6's step 6 built *W1–W10*, which contained the whole
error channel (W6, W7, W8), so step 6 was self-contained. v7 correctly retracted
W6/W7 and correctly moved the *replacement* channel (W11–W13) to a new step 7 —
but W8 was already in the middle of that channel, and the split was drawn by
W-number rather than by dependency. The test column below it was never re-read
against the new split; the *then* column even records the problem (*"built in step
7"*) as if noting it were the same as resolving it.

**Confidence:** High. Both halves are read directly off §2.4 :783/:807-809 and §4
:1190; the "does not compile" half needs no execution — a Rust match arm cannot
name a type from a later step.

---

## [I1] §6's §3.2 closure bullet is UNTOUCHED — §3.3 declares it a false PASS in v7's own words and it still stands, verbatim, as the closure condition. Its only compensator is an absence no test can fail on.

**Severity:** Important.

**Where.**
*Touched by v7:* §3.3's REFUSAL blockquote (:1161-1176, new); §6's **W14** row
(:1347, new).
*Falsified / left standing:* §6's bullet at :1368-1373, which
`git diff 0b3333f..0409815 | grep -c 'not produced by the code they judge'`
returns **0** for — the fold never touched it.

**The failure, concretely.** §3.3's new blockquote says, of that bullet:

> *"§6's closure bullet would report satisfied, because its stated evidence is
> only that the *transaction* comes from `gen-mt1-vectors.py`. … the bullet is
> true no matter what that something was. Same false-PASS shape as the `grep -c`
> struck under r2-I4."*

The bullet it is describing reads today, unchanged:

> *"**The vectors were not produced by the code they judge** (§3.2) — satisfied by
> construction for V1–V6 and V8–V27, since §3's transaction comes from a corpus
> generated by `scripts/gen-mt1-vectors.py`…"*

So P1 closes by ticking a bullet the same document proves cannot detect the defect
it exists to detect. **The plan diagnoses a false PASS and then keeps it as the
close condition.** That is the fold's own named lesson — *a replacement that does
not retract is an alternative* — applied to the bullet rather than to a W-row.

Two further things the bullet says that §3.3 now contradicts:

- It scopes the corpus argument to *"V1–V6 and V8–V27"*, i.e. it claims V27 too.
  §3's exception blockquote (:979) says the opposite — *"V7 and V27 are the only
  vectors this exception covers"* — and V27 needs a **second, ground**
  transaction, which no corpus supplies.
- Its only new compensator is §6's W14 row clause *"and the `regenerate` command
  was NOT run (§3.3)"*. That is an **absence**, and §6 itself rules twenty lines
  above: *"An absence is not assertable by any non-zero grep. Each site gets a
  test instead, **and each test can fail**."* No test can fail on "a command was
  not run." An implementer who ran it ticks the box.

**Why the fold permits it.** Round 4's I3 was filed as *"the vectors have no
home"*; v7 answered the headline (it named the home) and treated the report's
second paragraph — the false-PASS in §6 — as background rather than as part of the
finding. Nothing in §6 was opened.

**Confidence:** High. The `git diff | grep -c` is the whole proof for
"untouched"; both quoted texts are verbatim from the current file.

---

## [I2] Step 8's rule range: v7 DELETED the E7/E11 carve-out and does not except E13 or E19, whose only vectors v7 itself moved to step 6.

**Severity:** Important.

**Where.**
*Touched by v7:* §4 step 8's *then* column (:1192) and its vector list.
*Falsified:* §1.3's NORMATIVE exception table (:369-372, untouched) and §6's
closure bullet (:1320-1324, untouched); §1.3's E13 and E19 rows (:350, :356).

**The failure, concretely.** v6 → v7 on the same cell:

```
v6: every rule in **E1–E19**, except the two §1.3 names as having no Rust RED test. **E20 is step 10's**
v7: every rule in **E1–E19 EXCEPT E12, E17 and E20**
```

The carve-out for *"the two §1.3 names"* — **E7 and E11** — is gone. So step 8 now
claims a RED test for E7 and E11, and §1.3's exception table (untouched) says in
NORMATIVE terms that neither can have one:

> E7: *"no decoder check can reach it … nobody [owns it]"*
> E11: *"Delete E11's `==` in Rust and every vector stays green"*

An implementer at step 8 is instructed to make two rules go RED that the plan
measured cannot. §6's closure bullet still reads *"EXCEPT E7 and E11"*, so the
plan now says both things.

Second half, and this one is purely the diff's blast radius: v7 moved **V20, V23**
(E13's only two vectors) and **V24** (E19's only vector) from step 8 to step 6,
and did not add E13 or E19 to step 8's exception list. Mapping step 8's own vector
list — V5, V6, V9–V14, V16, V17, V17b, V19, V21, V22 — to rules gives E1–E10, E14,
E15, E16, E18 and nothing else. **E13 and E19 are named by step 8's rule column and
have no vector in step 8.**

Third, minor but in the same cell: E20 is not in the range `E1–E19`, so
"E1–E19 EXCEPT … E20" excepts a member of a set it is not in.

Rule→step coverage is nonetheless *complete* — I re-derived the union of all
twelve steps and it is all 29 vectors, with E13/E19 discharged at step 6 and E17
at step 5. **The vectors are sited right; the sentence above them is false**, and
§6 makes E1–E20 completeness a closure condition burnt against that sentence.

**Why the fold permits it.** Round 4's M1 asked for E12 and E17 to be added to an
exception list that already carved out E7/E11. The fold rewrote the list from
scratch rather than extending it, dropped what was there, and re-derived it against
v6's vector siting rather than against the siting the same fold had just changed.

**Confidence:** High. v6/v7 cells quoted from `git show`; the rule→vector map
derived by hand across all twelve steps and cross-checked against §1.3's rows.

---

## [I3] "§6 states the per-site assertion for each of the FOURTEEN" — §6 states twelve, and §6 itself says W6 and W7 "are not sites at all".

**Severity:** Important.

**Where.**
*Touched by v7:* §2.4's header (:765, :767 — *"names fourteen sites"* / *"NORMATIVE
— FOURTEEN sites"*), the W6/W7 strike-through rows (:781-782), :807, :829; §6's
grep paragraph (:1330-1337).
*Falsified:* §6's closure table itself (:1340-1353) — **12 W rows**, machine-counted.

**The failure, concretely.** §2.4 closes with a checkable claim: *"§6 states the
per-site assertion for each of the **fourteen**."* Counted:

```
$ awk 'NR>=1340 && NR<=1354' <plan> | grep -oE '^  \| \*{0,2}W[0-9]+'
W1 W2 W3 W4 W5 W14 W11 W12 W13 W8 W9 W10        -> 12
```

Twelve. The two missing are W6 and W7 — and §6, nine lines above its own table,
says: *"**W6 and W7 are retracted (r4-C1) and are not sites at all.**"*

So the same fold asserts both *"fourteen sites"* (four times: :765, :767, :807,
:829) and *"W6 and W7 are not sites at all"* (:1334), which together give twelve.
A reader burning §2.4's claim against §6 finds two site-assertions missing and has
to decide whether that is a gap to fill — which is the failure mode round 4's C1
was: *"the reader executes whichever they reach first."* The `~~W6~~` / `~~W7~~`
rows are still physically in the table, so "fourteen rows" is true and "fourteen
sites" is not, and the document uses the same word for both.

The same miscount reaches §3.3, which calls `coverage.rs` *"a **FOURTEENTH**
wiring site (W14)"* — it is the twelfth actual site.

**Why the fold permits it.** The retraction was done by strike-through, which
preserves the row count while destroying the site count, and the header sentence
was updated from "TEN" to "FOURTEEN" by counting rows. §6's table was rebuilt from
the site list and came out at twelve; nobody diffed the two.

**Confidence:** High — both numbers machine-counted, both texts verbatim.

---

## [I4] W6's retraction left §2.5a's `NonHexBody` near-miss with no producer AND no closure row — while §2.5a still says "and it has a test".

**Severity:** Important.

**Where.**
*Touched by v7:* §2.4's **W6** row, retracted (:781, *"DO NOT add `TX_PREFIX` to
`unknown_reason`'s loop"*); §6's closure table, from which v7 **deleted** the W6
row (*"a `tx:` record with a non-hex body reports `NonHexBody("tx:")`, not
`Unrecognised`"*).
*Falsified:* §2.5a's near-miss paragraph (:881-883, untouched) and §4 step 6's
test clause *"a `tx:` record with a bad body names its PREFIX, not
`Unrecognised`"* (:1190, carried across).

**The failure, concretely.** §2.5a, unchanged by this fold:

> **The near-miss, and it has a test:** a `tx:` record whose body genuinely is not
> lowercase hex must **still** report `NonHexBody`. W11 must not swallow the case
> the existing channel gets right.

Two clauses, both now false.

*"The existing channel gets right"* — it does not, for `tx:`. Read at source:

```rust
fn unknown_reason(record: &str) -> UnknownReason {
    for prefix in [record::PASS_PREFIX, record::TEXT_PREFIX] { ... }   // no TX_PREFIX
    UnknownReason::Unrecognised
}
```

`grep -rn 'TX_PREFIX' crates/` returns nothing. **W6 was the only edit in this
plan that would have made `unknown_reason` produce `NonHexBody("tx:")`, and v7
retracted it.** Worse, W4 (untouched) gives `classify` its own `TX_PREFIX` branch,
so a `tx:` record never reaches `Class::Unknown` and `unknown_reason` is
unreachable for it by construction. `NonHexBody("tx:")` now has **no producer** —
which is exactly, word for word, the reasoning v7 used to retract W7.

*"And it has a test"* — it had one: §6's W6 closure row, which v7 deleted. Nothing
replaced it. An implementer at step 6 writes the clause *"names its PREFIX, not
`Unrecognised`"*, and there is no site in the fourteen-row table that emits
`NonHexBody` for a `tx:` record.

**Why the fold permits it.** The retraction was scoped to W6's *prescribed edit*
being harmful (it makes W8's assertion RED). Nobody asked what else depended on
that edit *being made*. §2.5a's near-miss was written in v5 against a live W6 and
sits two sections away from the row that killed it — the same distance r1-C1 lived
at.

**Confidence:** High. `unknown_reason` read at source; the deleted §6 row is in
the v7 diff as a `-` line; §2.5a confirmed untouched by grepping the diff for its
text.

---

## [I5] No step files V7 or V27 into the fixture that §6's W14 row closes on — and step 4 is told to build its vectors with a generator step 9 commits.

**Severity:** Important.

**Where.**
*Touched by v7:* §3.3's closing sentence (:1174-1176, new); §4 step 4's
construction clause (:1188, new); §6's W14 row (:1347, new).
*Falsified:* §4 **step 9** (:1193, untouched) and §3's V7/V27 exception blockquote
(:965-997, untouched).

**The failure, concretely.** §3.3, new:

> **§4 step 4 constructs V1–V6 and V8–V26 and commits them** … —
> `scripts/gen-tx-record-vectors.py`, **which step 9 already commits**, extended to
> emit the framing rather than only V7 and V27.

Step 9 is five steps after step 4, and step 9's own row is *"this is the step that
**writes and commits** `scripts/gen-tx-record-vectors.py`"*. An implementer
executing §4 in order reaches step 4 and is told to use a script the plan creates
later. "Already" is doing the work of "eventually".

And the arithmetic leaves a hole. Step 4 files **27** vectors (`V1–V6` = 7 rows
including V4a/V4b; `V8–V26` = 20 rows including V17b). V7 and V27 are the two the
exception carves out — and **no step says they are filed into
`crates/me-cli/testdata/sysw_vectors.json`.** Step 9's row names V7 and the
generator but not the fixture; step 10 names V27 but not the fixture. §6's W14 row
nevertheless closes on *"P1's **29** vectors present in `testdata/sysw_vectors.json`"*.

(Under [C1] this row cannot fail on any of it, so the hole is currently invisible
to the gate as well as to the steps — but the ordering defect binds an implementer
regardless of how [C1] is resolved.)

**Why the fold permits it.** §3.3 was written as a *ruling about location* and
attached to the one step that had a vector-construction shaped hole. The step
*order* — which §4 exists to state — was never walked against it.

**Confidence:** High. Step numbers and cell text read verbatim; the 27-vs-29 split
counted off §3's table.

---

## [M1] "These fourteen sites are split across THREE steps, not one" then enumerates FOUR.

**Severity:** Minor.
**Where:** §2.4 :807-809, entirely new in v7.

> *"**These fourteen sites are split across THREE steps, not one (r4-C1, r4-C2):**
> **step 4** builds W14, **step 6** builds W1–W5 and W8–W9, **step 7** builds
> W11–W13, and **step 10** builds W10."*

Steps 4, 6, 7, 10 — four. The enumeration is correct and covers all twelve real
sites (5 + 2 + 3 + 1 + 1 = 12); the count in front of it is not. v6 said *"§4's
step 6 is these ten sites"*, so the sentence was rewritten from one step to a list
and the numeral was written before the list was finished. Harmless to an
implementer who reads the list, which is why it is Minor — but it is the eighth
enumeration in this cycle to be wrong on first count, in the sentence that
announces the fix for the seventh.

## [M2] "FIVE are not" survives six lines above the blockquote that corrects it to eight — in a sentence whose own arithmetic is already the eight-based figure.

**Severity:** Minor.
**Where:** §2.4 :813-818 (untouched prose) against the new r4-I5 blockquote at
:820-827.

The prose still reads *"**(r3-M3) FIVE are not**, and siting their tests at the
codec would site them where their input never arrives"* and lists exactly the five
v6 named — then ends *"but it rests on the other **twenty-one**."* v7 updated the
operand (`twenty-two` → `twenty-one`) and left the count that produces it, so one
sentence now asserts **5 + 21 = 29**. The blockquote immediately below says
*"**eight** are not record-level, not five"* and names V3, V4b and V15.

§4's siting already reflects the eight (V3's payload half, V4b's R15 half and V15
are all step 10), so no implementer acts wrongly; the corrective is adjacent and
unambiguous. Minor for that reason only. Note that the fold added a sweep term for
the *arithmetic* (`'other twenty-two'`) and none for the *count*, so the gate that
exists to catch precisely this could not see it — verified: 27 terms, 0 surviving
outside the block, and `FIVE are not` matches none of them.

## [M3] Two gate rows in §6.1 carry counts the same fold changed underneath them.

**Severity:** Minor.
**Where:** §6.1's table, :1478 and :1480 — both rows edited by v7, both with stale
text left inside the edited cell.

- :1480 — the command is still `./scripts/plan-fold-sweep.sh <doc> --terms <the
  **twenty-two** below>` while the same cell's PASS line now reads *"exactly **27**
  hits … A **twenty-eighth** hit anywhere else is a real finding"* and the block
  below holds 27 (machine-counted). An operator copying the invocation from the
  gate row runs it against the wrong term count.
- :1478 — the cell's leading figure was updated to *"**90** of 107 resolve"* and
  its trailing narrative still says *"it is why the number moved **from 90 to 98**
  rather than staying put."* 90 is now the *resolving* count and 98 the *previous
  total*; the sentence reads as though the current number were the old one. (Both
  the 90/107 and the 137/0 figures themselves re-run correct — see the header.)

## [M4] §6.3's enumeration blockquote is stale in two places v7 moved.

**Severity:** Minor.
**Where:** §6.3 :1708-1711 (untouched) against §2.4 :767 and :820.

> *"This is the **sixth** enumeration this cycle to be wrong on first count — three
> lockstep sites that were four, five wiring sites that were ten, **ten that needed
> thirteen**, three stale statements that were five."*

*"Ten that needed thirteen"* is v7's own count away — the table now names fourteen
rows and twelve sites. And the list carries four examples for a "sixth", while
§2.4's new blockquote calls its own the "SEVENTH". The running tally of this
cycle's miscounts is now itself one of them.

## [M5] §3.3 calls W14 a "FOURTEENTH wiring site" naming only `coverage.rs`; §2.4's W14 row spans two files.

**Severity:** Minor.
**Where:** §3.3 :1157-1159 against §2.4's W14 row :789 and §6's file map :1331.

§3.3: *"This makes **`coverage.rs`** a FOURTEENTH wiring site (W14), because its
`COVERAGE` table fails the build when a required vector is absent."* §2.4's W14
row and §6's map both give W14 as `coverage.rs` **and** `vectors.rs`. The
*"because"* clause is also wrong on the mechanism — it is `vectors.rs`'s
`every_required_vector_exists` that reads `COVERAGE` and checks vector presence;
`coverage.rs`'s own assertion checks spec §8.3 test ids (see [C1] §3). Filed
Minor because [C1] supersedes the substance.

---

## Part 2 — did round 4's eight findings land?

- **C1** (W11–W13 built by no step, W6/W7 still NORMATIVE, §6's W7 row a gate that
  cannot fail) — **PARTIAL.** W6 and W7 are struck with reasons, §6's W6/W7 rows
  are deleted and replaced by W11/W12/W13 rows that can fail, §1.5 and V13 are
  repointed, and step 7 now names the error path. But the new step split put W8 at
  step 6 and W11 at step 7 with W8 defined as the arm for W11's type — **[C2]** —
  and the retraction orphaned §2.5a's `NonHexBody` near-miss — **[I4]**. The
  finding is answered; the answer introduced two defects of the same family.
- **C2** (step 4's gate cannot go green: V3 is a whole payload) — **FIXED.** Step
  4 is now *"V1, V2 and V3's METADATA RECORD, round-tripped AT THE CODEC … no `me
  sysw pack` in the loop"*, V3's whole-payload round-trip is explicitly step 10's,
  and §2.2's `mt-codec` row was repointed to match. Step 4 has, separately, become
  ungreenable for a different reason — **[C1]** — but not this one.
- **I1** (V27 has no RED test for R17; E20 masks it) — **FIXED.** V27's row gained
  the NORMATIVE clause the finding asked for: *"the bare refusal is NOT the
  assertion … V27 asserts R17's RULE NAME on stderr, and goes RED when E20's
  set-completeness message appears in its place."* W12 carries R17's message
  (*"two sets share top-20 bits"*), so the rule name has a producer.
- **I2** (§6's W9 assertion cannot run — V18 is refused, so `show` never prints it)
  — **FIXED.** Replaced with a runnable pair: honest 222-byte record → txid and
  wtxid DIFFER; V26 (required to pass) → EQUAL. Both are records `show` emits.
- **I3** (the 29 vectors have no home; the regenerate command would void §3.2) —
  **WRONGLY FIXED.** §3.3 names a home and refuses the command, but the home is
  `sysw_vectors.json`, whose non-`#[ignore]` golden test goes RED on a filed P1
  vector (measured), whose schema cannot hold a refusal, and whose two named
  assertions are unaffected by P1's vectors (measured). The report's second half —
  §6's closure bullet is a false PASS — was quoted into §3.3 and left unfixed in
  §6. **[C1]**, **[I1]**, **[I5]**.
- **I4** (§6.3's five stale spec statements gated by nothing) — **FIXED.** §6 gains
  a bullet naming all five (spec §3.6, §6's P1 row, §2.1b's R4′ row, §6's P3 row,
  §1's ownership table) and stating the consequence of leaving them; §6.3's claim
  that §6 carried them is retracted and re-attributed to v5.
- **I5** ("the other twenty-two" wrong on both operands; step 8 sites V19–V24 at
  the codec) — **PARTIAL.** The binding half landed: V20/V23/V24 moved to step 6
  with a reason, step 8's list is re-cut, and the 29/8/21 arithmetic is now
  correct in the blockquote. The prose above it still says *"FIVE are not"* in the
  same sentence as *"twenty-one"* — **[M2]** — and the move broke step 8's rule
  range — **[I2]**.
- **M1** (step 8 claims E1–E19 while E12 is step 12's and E17 is step 5's) —
  **WRONGLY FIXED.** E12 and E17 were added to the exception list and the existing
  E7/E11 carve-out was deleted in the same rewrite, so step 8 now claims a RED test
  for the two rules §1.3 measured cannot have one — and E13/E19 were not added
  though the same fold moved their vectors. **[I2]**.

---

## Verdict

**2C / 5I / 5M.**

**v7 is NOT GREEN.** Two Criticals block: **[C1]** — §3.3's NORMATIVE vector home
turns a shipped golden test RED and both assertions W14 names are vacuous for P1,
measured by running the real suite against the real fixture; and **[C2]** — step 6
is assigned W8, whose type W11 arrives at step 7, while two of step 6's own listed
test clauses are conceded to step 7, so step 6 cannot end green.

**On the lens.** It paid. Every one of the twelve findings is a site the diff did
not touch, falsified by a site it did: a shipped test at `vectors.rs:137` that no
version of this plan has ever mentioned; §4's step-order preamble; §1.3's exception
table; §2.5a's near-miss; §6's §3.2 bullet, which the fold quoted and did not open.
Not one of them shares a token with the text that falsified it, and the document's
own `plan-fold-sweep.sh` confirms it: 27 terms, **0** surviving outside the block,
while `the twenty-two below`, `ten sites` and `FIVE are not` sit in the open.

**One observation for the next brief.** Round 4's I3 prescribed a remedy
(`vectors.rs:147` and `coverage.rs:230`) and v7 executed the prescription without
reproducing the defect. Both cited lines are real; both are the wrong assertions;
the one that matters sits between them. **Machine-check the reviewer's remedy, not
only the reviewer's claim** — a four-minute `cargo test` would have caught [C1]
before this round was paid for, and it is the same four minutes §6.1's own r2-C1
blockquote is about.
