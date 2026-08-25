# R0 — IMPLEMENTATION_PLAN_P1_me_container.md, round 4

Independent adversarial review of **v6** (`0b3333f`). Author ≠ reviewer. Two
questions: did round 3's remaining eight land, and **can an implementer who was
not present execute §4's twelve steps and arrive at the artifact §1–§3
specifies?**

The second question is where every Critical below came from. Both of them are
*step-execution* defects: nothing in §1–§3 is wrong, and both survive because
reading a step is not running it.

---

## Commands run, and their raw output

```
$ git log --oneline -3
0b3333f fold: P1 plan v6 -- round 3's deferred half, and 3 defects v5 itself introduced
c517286 fold: P1 plan v5 -- R0 round 3's two Criticals were ONE defect (the error path)
5f9f00b persist: R0 round 3 on P1 plan v4 -- 12/13 landed, 2C/7I/3M
```

**Did either fold touch §2.4's wiring table, §4's steps, or §6's W-rows?**

```
$ git diff 5f9f00b..0b3333f -- design/IMPLEMENTATION_PLAN_P1_me_container.md \
    | grep -E '^[-+]' | grep -E 'W6|W7'
+**And W6's prescribed edit is actively wrong.** Adding `TX_PREFIX` to that loop
+container error carries it; the printer names it. Three additions, replacing W6's
-  nine files; W1–W3 all live in `record.rs` and W4–W6 all live in `mod.rs`, so it
+  nine files; **(r3-I6) W1–W3 live in `record.rs`, W4–W7 and W10 live in `mod.rs`,

$ git show c517286 -- design/IMPLEMENTATION_PLAN_P1_me_container.md | grep '^@@'
@@ -796,6 +796,95 @@   <- inserts §2.5 / §2.5a / §2.5b
@@ -1465,6 +1554,29 @@  <- inserts §6.3
```

v5 — the fold that answered round 3's **two Criticals** — was **purely
additive**. §2.4's normative W6/W7 rows, §4's step 6 and §6's W6/W7 closure rows
are byte-identical to v4.

**Where W11/W12/W13 appear in the whole document:**

```
$ grep -n 'W11\|W12\|W13' design/IMPLEMENTATION_PLAN_P1_me_container.md
848:| **W11** | a `TxRecordError` enum whose variants are the rules ...
849:| **W12** | `SyswError` gains a **set-level** variant ...
850:| **W13** | the printer arm for both, at `sysw_error` **and its outer match** ...
858:lowercase hex must **still** report `NonHexBody`. W11 must not swallow the case
1172:becomes no class, and lands in §1.5's refusal path via W11 — so a record's index
```

Three rows in §2.5a plus two passing prose mentions. **Zero hits in §4. Zero hits
in §6.**

**Citation gate (run by me, not transcribed):**

```
$ ./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
─── citations resolved: 81 / 98 ; dangling: 17 ; ambiguous: 0
dangling breakdown, counted from the raw list:
  mnemonic-transaction  pipeline.rs x6 (:17-27,:54,:66,:93,:148,:160)
                        header.rs   x2 (:4,:26)
                        lib.rs      x1 (:9-14)                        = 9
  bitcoin-0.32.9        transaction.rs x7, encode.rs x1               = 8
```

§6.1's 9/8 split is exact.

**The 17 ungated citations, resolved by hand against the sibling repo** (this is
the class where "5 of 22 ungated facts were false" last cycle — here **0 of 9
were false**):

```
pipeline.rs:17-27  content_id_from_txid_display doc comment      -> quoted correctly
pipeline.rs:54     let set_id = content_id_from_txid_display(..) -> exact
pipeline.rs:66     let lower = s.trim().to_ascii_lowercase();    -> exact (E13)
pipeline.rs:93     pub corrected: usize                          -> exact (E19)
pipeline.rs:148    pub struct DecodedSet { pub bytes: Vec<u8> }  -> exact
pipeline.rs:160    pub fn decode_chunk(s:&str, plan:Option<Chunking>) -> Result<DecodedChunk>
                                                                 -> exact
pipeline.rs:234    pub fn decode(strings:&[String]) -> Result<DecodedSet>
                                                                 -> signature in §2.2 is exact
header.rs:4        version(5)|chunk_set_id(20)|count−1(15)|index(15) = 55 bits  -> exact
lib.rs:9-14        "not produced by this crate" rule             -> exact
mt-codec/Cargo.toml  version = "0.1.0"; bitcoin = "0.32"; 0 `bitcoin::` in src/  -> M8 exact
```

**Corpus, read from `mt1_v1.json`:**

```
generator: scripts/gen-mt1-vectors.py (mnemonic-engrave)
even   222 B  txid 2dcf2b97..72ebf630  wtxid d5717c03..ed836f51  txid_is_wtxid False
              bytes_per_chunk 37  6 chunks, all 87 chars  set_id 0x2dcf2
uneven 284 B                                               set_id 0x3b426
```

**§3.1's whole table, recomputed here rather than read:**

```
meta record chars: 153
1/1  rawrec 1857  chunkchars 2001  container 2155  spare 30579
1/2  rawrec 1939  chunkchars 2092  container 2246  spare 30488
2/2  rawrec 3537  chunkchars 3955  container 4109  spare 28625
5/2  rawrec 8313  chunkchars 9383  container 9537  spare 23197
10/2 rawrec 16287 chunkchars 18583 container 18737 spare 13997
body ceiling 16290 ; hex chars avail 32731 -> 16365 whole bytes
raw record spare at 10/2: 16447
```

Every figure in §3.1 and §1.4a reproduces **exactly**. 153, 8,313, 122-over-8191,
18,737, 13,997, 16,290, 16,365, 16,447 — all correct.

**V18/V26, the pair §6's W9 row asserts on:**

```
$ python3 -c "...zip the two 32-byte values..."
differing byte positions: 32 of 32
differing hex chars:      61 of 64
```

**Tree facts:**

```
crates/me-cli/src/sysw/wire.rs:42   MAX_SECTION_LEN: usize = 8191   (step 1's target)
crates/me-cli/src/seal/wire.rs:21   MAX_SECTION_LEN: u32   = 8191   (step 1's guard)
enum Class                          8 variants today -> 10 after W2   ✓
split() dispatches `c if c.is_secret()` else public -> W3 alone suffices, no 11th site
ls crates/me-cli/src/sysw/          9 files ("per-file counts over nine files" ✓)
classify("mt1…") -> UnknownHrp("mt") -> Class::Unknown -> Err(Unclassifiable) -> exit 4
crates/me-cli/testdata/sysw_vectors.json   EXISTS
crates/me-cli/src/sysw/vectors.rs:26       pub const PATH = "testdata/sysw_vectors.json"
crates/me-cli/src/sysw/coverage.rs         COVERAGE, build-failing derivation
```

---

## Part 1 — did round 3's eight remaining findings land?

**I1 — FIXED.** All three citing sites now attribute
`joins_with_lf_and_no_trailing_lf` to `seal`: §1.3's E12 (:347), V19's cell
(:997), §3.1's (M7) paragraph (:1047). `grep -n 'joins_with_lf'` returns exactly
those three plus step 12's *"**Not** `seal`'s …"*. **§4 step 12 is new** and
states a real RED condition — *"It goes RED when `crates/me-cli/src/sysw/mod.rs:260`'s
separator changes"* — which is the one join a transaction-only pack path actually
executes (`split`'s `payload.public.join("\n")`). E12 is no longer a third false
entry in §6's completeness claim.

**I3 — PARTIAL.** The *constructibility* half landed: §3's exception now reads
*"V7 and V27 are the only vectors this exception covers"*, V27 grinds a second
transaction whose display txid begins `2dcf2` (correct — top-20 bits is exactly
the first five hex characters, matching `txid_hex.get(..5)` at pipeline.rs:24),
and R17 gained a row in §6.2. The *isolability* half did not: **E20 has replaced
R10 as the rule that masks R17**, so V27 still stays green with R17's comparison
deleted. See **[I1]** below.

**I4 — WRONGLY FIXED.** The finding's substance is closed — §2.4's W9 now prints
*"form, carried txid AND carried wtxid"* and §6's W9 row asserts the wtxid. But
the fold attached a new assertion, *"V18 and V26 differ in exactly 32 output
positions, and that is the assertion"*, which **cannot be run**: V18 is a vector
the plan requires E17 to REFUSE, so `me sysw show` never emits a line for it. See
**[I2]** below.

**I5 — FIXED.** §6's bullet now reads *"TWO still open (r3-I5)"*, item 3 is
struck through as **DONE** with the artifact verified, and I confirmed the spec
independently: `design/SPEC_engrave_transaction.md:402` carries the **chunks IN
THE CONTAINER** column, the 10/2 row reads **18,737** with **13,997 spare**, and
the spec's amendment block names exactly two still owed (the 16,367 headline and
the 5/2 parenthetical). Plan and spec now agree. *(Nit, not a finding: the bullet
carries a stray unmatched `**` — "…bare records.** All three are…" — and says
"All three are RE-DERIVED" twice in two sentences.)*

**I6 — FIXED, and independently re-derived.** All four v3-era counts corrected,
and I measured each rather than reading it: **9** `mnemonic-transaction`
citations and **8** `bitcoin` (my own `plan-cite-check.sh` run, breakdown above);
*"These **twenty-two**"* over a block I counted at 22 entries; and the struck-grep
bullet now reads *"W1–W3 live in `record.rs`, W4–W7 and W10 live in `mod.rs`, and
W8–W9 live in `main.rs` — which that grep's path cannot see at all, so it sees
**two files, not ten sites**"*. That is **more correct than round 3's own
wording**: round 3 said *"three files and ten sites"*, but the cited grep path is
`crates/me-cli/src/sysw/`, which can see only record.rs and mod.rs — two. `ls`
confirms nine files in that directory.

**M1 — FIXED.** All three manifest-bearing sites now write `mt-codec = "=0.1.0"`
(§2.2's table :695, §4 step 4 :1123, §5 :1224), each with the caret rationale.
The one remaining bare `0.1.0` (:1218, *"publish `mt-codec` 0.1.0 to crates.io"*)
is a version, not a requirement.

**M2 — FIXED.** §4.2 now reads *"Stdin filters EMPTY lines exactly as `--in`
does — `read_records` tests `!l.is_empty()`, so **a line holding a single space
SURVIVES as a record**"*, and routes it to W11's refusal path. `grep -n blank`
returns no live use.

**M3 — PARTIAL.** §2.4's sentence was hedged from *"V1–V27 are record-level"* to
*"**Most** of V1–V27 are"* and five exceptions named. Two halves did not land:
the count is wrong (§3 has **29** vector rows, and at least **8** are not
record-level), and **§4 step 8 — the site round 3 actually named — still files
V19–V24 under "the layout codec"**, which is the mis-siting the finding was
about. See **[I5]** below.

**Score: 5 FIXED, 2 PARTIAL, 1 WRONGLY FIXED.**

---

## Part 2 — defects in v6

## [C1] §2.5a's W11–W13 are built by no step and closed by no condition, while §2.4's superseded W6/W7 remain NORMATIVE — so r3-C1 survives the fold that answered it, and §6's W7 row is a gate that cannot fail

**Severity:** Critical.
**Where:** §2.4's **W6** row (:774) and **W7** row (:775); §4 **step 6** (:1125);
§6's closure table **W6** row (:1280) and **W7** row (:1281); against §2.5 (:835)
and §2.5a's W11/W12/W13 (:848-850).

**The failure, concretely.** An implementer reaches §4 step 6. Its *then* column
is: *"the wiring — **W1–W10 of §2.4**."* They open §2.4, headed **"NORMATIVE —
TEN sites"**, and execute it:

- **W6:** *"`unknown_reason` iterates `[PASS_PREFIX, TEXT_PREFIX]` only … **Add
  `TX_PREFIX`**"* → they add it.
- **W7:** *"**`UnknownReason` gains a rule-carrying variant** — `TxRule(&'static
  str)`"* → they add it.

Then they burn §6's closure table:

| row | outcome |
| --- | --- |
| W6 — *"a `tx:` record with a non-hex body reports `NonHexBody("tx:")`"* | **PASSES** — that is precisely what adding `TX_PREFIX` to the loop does |
| W7 — *"`UnknownReason::TxRule("magic")` exists and is `Copy`"* | **PASSES** — a compile-time existence check. Nothing has to produce it |
| W8 — *"a record with magic `MTX2` names the **magic** rule, and 'not lowercase hex' does NOT appear"* | **FAILS** |

W8 fails because `unknown_reason` is string-only: an `MTX2` record whose body is
valid lowercase hex now matches `TX_PREFIX` and returns `NonHexBody("tx:")`,
whose message (read at `crates/me-cli/src/main.rs:1263-1272`) is *"begins `tx:`,
but its body is not lowercase hex … `printf '%s' 'your text here' | xxd -p -c
256"* — a false statement plus a corrupting instruction. **§2.5 states this
outcome itself** (*"RED against §6's own W8 assertion"*), two pages after the
table that prescribes it.

The implementer's only escape is §2.5/§2.5a — reachable only by reading past the
section §4 pointed them at, because **no step, no closure row and no W-row
cross-references W11, W12 or W13.** `grep` finds them in exactly three table rows
plus two passing prose mentions; §4 has zero hits and §6 has zero hits.

And the deeper half: even an implementer who *does* read §2.5a can close this
plan without building any of it. §6's W-table stops at W10. So P1 goes GREEN with:
`UnknownReason::TxRule` declared and **unproducible** (r3-C1, verbatim); no
`TxRecordError`; no set-level `SyswError` variant, so E20's *"chunk 7 of set
0x2dcf2 is missing"* and R17's *"two sets share top-20 bits"* have no
representation at all; and no arm in `sysw_error`'s outer match at
`crates/me-cli/src/main.rs:1256`. **The two Criticals round 3 raised were folded
into prose and never reached either the execution surface or the closure
surface.**

**Why the plan permits it.** `git show c517286 --stat` on the plan is two pure
insertions — §2.5/§2.5a/§2.5b after line 796, §6.3 after line 1465. The fold
answered the finding by *adding a section that disagrees with §2.4* rather than
by retracting §2.4's rows, and §4 and §6 were never edited to point at the
replacement. §2.5a says *"Three additions, **replacing** W6's edit"*; the word
"replacing" describes an intent the document does not carry out.

**Confidence:** High. Every claim is a grep or a diff, listed in the header. The
W8 failure mode is the plan's own measured statement, not my inference.

---

## [C2] §4 step 4's gate cannot go green: V3 is a whole payload, and `me` refuses a bare `mt1` record until step 6's W5 — which the plan measured itself

**Severity:** Critical.
**Where:** §4 **step 4** (:1123) and **step 5** (:1124); §3's **V3** (:983) and
**V4b** (:987); §2.4's **W5**; §2.2's CHUNKS row and **W10**.

**The failure, concretely.** §4's preamble is binding: *"Each step: failing test
first, watched fail **for the stated reason**, minimal code, **full suite
green**."*

Step 4's test is *"V1–V3 round-trip (**V3 is a whole PAYLOAD now** — a metadata
record plus six bare chunks, §1.4a)"*. Its *then* is *"the layout codec"*.

V3's own row: *"**The whole payload is the vector**, not one record"*, pinning
*"E12's single `\n` between records"* and *"E20's complete set (`count = 6`,
indices 0..5)"*.

Trace what happens at step 4 on the real tree:

```
classify("mt1p9h8jqq…")  -> crate::classify::classify -> Err(UnknownHrp("mt"))
                            (crates/me-cli/src/classify.rs:46-51 matches md/mk/ms only)
seal::record::validate_record -> Err(Unclassifiable)
sysw::classify           -> Class::Unknown
split (sysw/mod.rs:255)  -> Err(SyswError::Unclassifiable(0, Unrecognised))
me sysw pack             -> exit 4, nothing packed
```

**The plan prints this exact refusal itself**, in §1.4a's cost 1, as evidence for
why W5 is needed. So step 4's test does not fail for want of a layout codec — it
fails because `classify` has no `ValidMT` branch, which is **W5, built at step
6**, and because `split` has no set pass, which is **W10, built at step 10**. No
amount of "the layout codec" makes it pass. Step 4, the fourth of twelve, cannot
go green, and the plan's own per-step green rule is unsatisfiable there.

**Step 5 has the same shape.** Its vectors include **V4b**, *"CHUNKS, with txid
AND wtxid AND `chunk_set_id` written out … plus **R15's positive case — which is
now the BINDING**"*. R15 binds a carried txid's top 20 bits to *the chunks'*
`chunk_set_id`, and §2.2 rules that the CHUNKS decode is *"gather the record's
set … reassemble via `mt_codec::decode`, THEN deserialise"* — gathering that
§2.2 says explicitly *"is not `classify`'s … It is `split`'s, which is W10"*,
**step 10**. So step 5's wtxid check on the CHUNKS form also depends on a step
six positions later.

Two implementers diverge here, and not subtly: one blocks and reorders the plan;
the other silently narrows V3 to "round-trip the metadata record alone", which is
a different artifact from the one §3 specifies and leaves E12's and E20's pins in
V3's cell discharged by nothing at step 4.

**Why the plan permits it.** §1.4a's ruling turned V3 from one record into a
payload, and V4b from a record into a record-plus-set. §3's cells were rewritten;
§4's step rows were not. `git show 0b3333f` touches step 4's row only to insert
the `="0.1.0"` pin — the payload clause was carried across unexamined.

**Confidence:** High. The refusal path was traced through the actual source
(`classify.rs:40-51`, `seal/record.rs:118-130`, `sysw/mod.rs:124`, `:255`) and it
is the same refusal the plan itself quotes.

---

## [I1] V27 still has no RED test for R17 — E20 has simply replaced R10 as the rule that masks it, which is r3-I3's failure mode with a different mask

**Severity:** Important.
**Where:** §3's V27 row (:1005) and its exception blockquote (:965-971); §1.3's
**E20** (:365); §6.2's new R17 row (:1587); §4 step 10.

**The failure, concretely.** The fold made V27 *constructible* — a second,
ground transaction whose display txid begins `2dcf2`. It did not make V27
*isolating*, which is the property r3-I3 was actually about (*"V27 stays green
with R17's comparison deleted and R17 has no RED test"*).

Build V27 as v6 specifies: two CHUNKS `tx:` records whose carried txids share
their top 20 bits, each with its complete chunk set. Now read **E20** as written:

> *"**Every `mt1` record in the payload MUST belong to exactly ONE CHUNKS `tx:`
> record's set** … the binding is R15's — `chunk_set_id` == the top 20 bits of
> that record's carried txid"*

By construction both metadata records have the same top-20 bits, so **every chunk
in the payload matches both records' set_id**. No chunk belongs to *exactly one*
set. E20's first clause is violated by every single chunk, and its second clause
fails too (the pooled set carries index 0 twice, index 1 twice, …).

So: delete R17's comparison, and the payload is still refused — by E20. **V27
stays green, and R17 has no RED test.** §6.2's own NORMATIVE sentence,
*"all four get a test that goes RED when its check is removed … verified by
deleting the check by hand and watching the test fail for the stated reason"*,
is now unsatisfiable for the fifth row the fold added to that very table.

This is *not* unfixable — V27 could assert R17's **rule name** on stderr rather
than the bare refusal, which would go RED (E20's message would appear instead).
But the plan never says so, and the contrast with V15 is the tell: V15's row
carries a **NORMATIVE** sentence explaining exactly which perturbation preserves
its RED-ability (*"because it decides whether the vector can go RED (r2-M4)"*).
V27 got construction advice and no such sentence.

**Why the plan permits it.** The fold scoped itself to the question round 3
asked — *can V27 be built?* — and answered it correctly. The reason V27 could not
go RED under v4 was R10; nobody re-asked the question against the rule that the
new construction runs into first.

**Confidence:** High. E20's text read verbatim; the masking is a direct
consequence of R15-as-binding, which §1.4a and §1.3 both state.

---

## [I2] §6's W9 assertion — "V18 and V26 differ in exactly 32 output positions" — cannot be run, because V18 is a vector the plan requires to be REFUSED

**Severity:** Important.
**Where:** §6's **W9** closure row (:1281, added by `0b3333f`); against §1.3's
**E17**, §3's **V18** (:995) and §6's near-miss list.

**The failure, concretely.** The fold answered r3-I4 by making W9 print the
wtxid — correct — and then added: *"**V18 and V26 differ in exactly 32 output
positions**, and that is the assertion."*

V18 is refused. Three places say so:

- §1.3 **E17**: *"What fails it, measured (§1.1a): **V18**, the witness-stripped
  body"*.
- §3's **V18** cell: *"The wtxid is what refuses it."*
- §6's near-miss list: *"**V18/V26** (a stripped body with the real wtxid
  **refused** / the SAME 113 bytes with an honest wtxid **passes**)"*.

`me sysw pack` on V18 aborts at exit 4 with nothing on stdout (§1.5, §2.3). There
is no container, so **`me sysw show` never produces output for V18** and "32
output positions" has nothing to compare against. The assertion cannot be
executed as stated.

And under the only other reading — 32 positions of the *record bytes* — the
number is right (I measured 32 of 32 differing byte positions) but the assertion
tests the **vector definitions**, not W9's behaviour: V18 and V26 are *defined*
as the same 113 bytes differing only in the carried wtxid, so it holds whether or
not W9 is ever implemented. That is a gate that cannot fail.

Worth stating either way: as an assertion on `show`'s **text** output, 32 is the
wrong number. The two values are rendered as 64-character hex, and they differ in
**61 of 64 characters**, not 32.

The finding round 3 raised is genuinely closed by W9's first clause (*"prints the
carried txid **and the carried wtxid**"*), which is a real, RED-able test. The
sentence appended after it is the defect.

**Why the plan permits it.** The pair V18/V26 is the sharpest thing in the plan
and the fold reached for it. It is the right pair for **E17** — where V18 must be
refused, which is exactly what disqualifies it for a **`show`-output** assertion.
The operator-facing pair W9 actually needs is V26 against the honest 222-byte
record, which share a txid and differ in wtxid — the case §2.4's own W9 row
describes one sentence earlier.

**Confidence:** High. The refusal requirement is stated in three sections; both
numbers were computed here.

---

## [I3] §3's twenty-nine vectors have no home — the repo already ships `testdata/sysw_vectors.json`, a build-failing coverage derivation, and a documented regenerate command that would violate §3.2

**Severity:** Important.
**Where:** §3's vector table; §3.2; §6's *"The vectors were not produced by the
code they judge"* bullet (:1319) and *"V4a/V4b write out txid, wtxid and
`chunk_set_id` explicitly **in the vector file**"* (:1253); §3's exception
(:971); §4 steps 4, 5, 8, 9, 10.

**The failure, concretely.** Step 4 begins. The implementer needs V1, V2 and V3
as bytes. There is no path, no format, no schema and no producer: §4 assigns the
construction of V1–V6 and V8–V26 to no step, and the one generator the plan does
commit (`scripts/gen-tx-record-vectors.py`, step 9) is scoped by §3's own
exception to *"V7 and V27"* only.

Three sites refer to *"the vector file"* (:916, :971, :1253) and none names it.
Step 9's generator *"writes it into the vector file beside the corpus pair"* — a
file the plan assumes exists and no step creates.

Meanwhile the tree already has exactly this artifact, and the plan mentions none
of it:

```
crates/me-cli/testdata/sysw_vectors.json        exists
crates/me-cli/src/sysw/vectors.rs:26            pub const PATH = "testdata/sysw_vectors.json"
crates/me-cli/src/sysw/vectors.rs:1-8           "the contract the Go port is checked against"
crates/me-cli/src/sysw/vectors.rs:147           fn every_required_vector_exists()  <- fails the build
crates/me-cli/src/sysw/coverage.rs:7            assert_every_named_test_is_placed  <- fails the build
```

`vectors.rs`'s own module doc is §3's sentence in different words — *"the Go
implementation reads the SAME file … a vector both sides are missing is a defect
neither will ever notice"* — and P1's vectors are `sysw` container vectors. So an
implementer has two plausible homes and the plan rules neither: extend
`sysw_vectors.json` (which drags in `coverage.rs`, whose `COVERAGE` table is
derived from `SPEC_systemwide_payloads` §8.3 and **fails the build** when a
required vector is absent — an eleventh site the ten-site table does not name),
or start a new file (which the existing Go cross-language check never reads). Two
implementers, two artifacts.

**The §3.2 hazard is not hypothetical, it is documented in the repo.**
`vectors.rs:10-14` ships this:

```sh
cargo test -p mnemonic-engrave --lib sysw::vectors::regenerate -- --ignored --nocapture
```

That command rewrites the fixture **from today's code** — precisely what §3.2
forbids (*"never dumped from the encoder under test"*). An implementer who files
P1's vectors in the obvious place and runs the documented refresh has violated
§3.2, and **§6's closure bullet still reports satisfied**, because its stated
evidence is only that *"§3's transaction comes from a corpus generated by
`scripts/gen-mt1-vectors.py`"*. The corpus supplies the *transaction*. Every byte
of the *record framing* — magic, version, form, the two 32-byte identifiers,
`n_fields`, the TLVs, `body_len` — is assembled by something the plan never
names, and the bullet is true regardless of what that something was. It is the
same false-PASS shape as the `grep -c` the plan struck under r2-I4.

**Why the plan permits it.** §3.2 was written as a *rule* (r0-I12) and §6 turned
it into a *closure condition* by pointing at the corpus's provenance. Nobody asked
where the twenty-nine records would physically live, because the vectors are
discussed as pins rather than as a file.

**Confidence:** High for the absence (grep over the whole plan for `vectors.rs`,
`coverage.rs`, `testdata`, `sysw_vectors` — zero hits). High for the existing
infrastructure (paths, constants and the two build-failing assertions read at
source).

---

## [I4] §6.3's five stale spec statements are gated by nothing — §6.3 claims "§6's closure list carries them" and it does not

**Severity:** Important.
**Where:** §6.3 (:1602-1624), closing line *"**P1 owns correcting all five**, and
§6's closure list carries them"*; against §6's bullet list (:1249-1349).

**The failure, concretely.** §6.3 is v5's fold of r3-I7. It ends by delegating
enforcement to §6. §6 does not accept the delegation:

```
$ sed -n '1248,1350p' <plan> | grep -n 'five\|FIVE\|6\.3\|stale\|STALE'
20:  … so it sees **two files, not ten sites** …
68:  1. *"a **16,367-byte** raw transaction"* …
```

Neither hit is the delegation. §6's only spec-correction bullet is *"Spec
corrections this phase owns — **TWO** still open"*, and those two are the
**numeric** corrections to spec §2.3 (the 16,367 headline and the 5/2
parenthetical). They are a different list from §6.3's five, which are **prose**
statements in spec §3.6, §2.1b, §6's P1 row, §6's P3 row and §1's ownership
table.

`grep -n '§6\.3'` over the whole plan returns **one hit — §6.3's own heading**.
Nothing references it. So the five statements the fold identified as falsified by
the ruling — including spec §6's **P1 row**, which is P1's own scope statement,
and spec §3.6's *"R15 validates it only within a single record"*, which a P4
implementer reads for the picker — are owned by P1 and burned down by no closure
condition and no step.

The consequence is concrete: P1 closes GREEN, spec §3.6 still tells the next
phase that R15 is a within-record check, and R15 has meanwhile become the
**binding mechanism** across a metadata record and 202 siblings (§1.4a). That is
the same class as r1-I8's unsatisfiable close condition, inverted: a satisfiable
obligation with no condition to satisfy.

**Why the plan permits it.** §6.3 was appended by v5 as a whole new section
(`@@ -1465,6 +1554,29 @@`) and §6, twenty lines above it, was not touched in
the same commit. The sentence "§6's closure list carries them" describes the
edit the fold intended and did not make.

**Confidence:** High. §6 read in full; the delegation sentence and the grep are
both quoted above.

---

## [I5] "It rests on the other twenty-two" is wrong on both operands, and §4 step 8 still sites V19–V24 at "the layout codec" — the half of r3-M3 that binds an implementer

**Severity:** Important.
**Where:** §2.4 (:798-805); §4 **step 8** (:1127); §3's vector table.

**The failure, concretely.** Two separate errors in the M3 fold.

**(a) The count.** §2.4 now reads *"**Most** of V1–V27 are record-level vectors …
**FIVE are not** … The conclusion stands … but it rests on **the other
twenty-two**."* Both numbers are wrong.

*The denominator.* §3's table has **29 rows**, not 27 — `V4a`, `V4b` and `V17b`
are separate vectors, and §6 requires *"V4a/V4b write out txid, wtxid and
`chunk_set_id`"* individually. Counted mechanically:

```
$ grep -o '^| \*\*\?V[0-9a-b]*\*\*\? ' <plan> | wc -l
29
```

*The exceptions.* At least **eight** are not record-level, not five. The fold
named V20, V23, V24, V25, V27 and missed three that its own cells declare:

| vector | the plan's own words |
| --- | --- |
| **V3** | *"**The whole payload is the vector**, not one record — that is what the ruling changed"* |
| **V4b** | *"CHUNKS … plus **R15's positive case — which is now the BINDING**"* — R15 binds the carried txid to *the chunks'* `set_id`, so the chunks must be present |
| **V15** | *"a chunks record whose carried txid's top 20 bits ≠ **its chunks'** `chunk_set_id` … the perturbation is applied to the **CHUNKS' EMBEDDED** `set_id`"* |

29 − 8 = 21, not 22, and neither operand of the stated arithmetic is right. **This
is the seventh enumeration this cycle to be wrong on first count** — §6.3's own
blockquote, written in the same fold, calls the sixth and says *"an enumeration in
this document is a hypothesis until something re-derives it."*

**(b) The mis-siting, which is what r3-M3 was actually about.** Round 3's M3 ends:
*"it tells an implementer to site three of E13's and E19's tests at a layer that
never sees their input"*, and it names the site — *"**step 8** nonetheless files
V19–V24 under 'the layout codec'"*. Step 8 is unchanged:

```
| 8 | V5–V6, V9–V14, **V16–V17b, V19–V24** | every rule in E1–E19 … | 
```

with the *then* column reading *"the layout codec"*. V20, V23 and V24 are bare
`mt1` records with no `tx:` framing; the layout codec never sees one. An
implementer executing step 8 writes three tests against a codec whose input is a
`tx:`-framed record and passes them vacuously. The fold added a sentence to §2.4
acknowledging the problem and left the instruction that causes it in place.

**Why the plan permits it.** M3 was filed as a Minor about a *claim*, and the
fold treated it as one — it repaired the sentence. The sentence was a symptom;
the step row is the thing an implementer executes.

**Confidence:** High. Row count is mechanical; the three added exceptions are
quoted from their own cells; step 8's text is unchanged in `git diff`.

---

## [M1] Step 8 claims to cover "every rule in E1–E19" while E12's test is step 12's and E17's is step 5's

**Severity:** Minor.
**Where:** §4 step 8 (:1127).

Step 8's *then* column: *"every rule in **E1–E19**, except the two §1.3 names as
having no Rust RED test. **E20 is step 10's**."* It carves out E20 and the two
exceptions and nothing else, but two more rules are not step 8's either: **E12**
is discharged by the new **step 12** (*"E12's RED test, in `sysw`"*), and **E17**
by **step 5** (*"V18 and V26 belong here and not in step 8"* — step 5's own
words). Mapping step 8's vector list to rules leaves E12 and E17 with no vector in
it, which is correct; the sentence above the list is what is false.

Harmless to an implementer following the vector lists (union of all steps = all 29
vectors, checked — no vector is orphaned), but §6 makes E1–E20 completeness a
closure condition, and this is the sentence a reader burns it against.

**Confidence:** High. Rule-to-vector mapping done by hand across all twelve steps.

---

## Verdict

**2C / 5I / 1M — this plan is NOT GREEN.**

Round 3's eight: **5 FIXED, 2 PARTIAL (I3, M3), 1 WRONGLY FIXED (I4)**.

The two Criticals are the same shape and it is the shape Part B was asked to look
for: **§1–§3 were re-designed twice (the `wtxid` field, then the bare-record
ruling) and §4 and §6 were not re-derived against either.** Every finding above
except [I3] and [M1] is a consequence of that single gap — the design surface
moved, the execution surface did not follow, and no round had previously read §4
as a thing to run rather than a thing to check.

Note for the fold that answers this: **[C1] and [C2] both live in §4 and §6, not
in §1–§3.** Nothing in the record layout, the rules, the vectors or the
arithmetic was found wrong this round — I recomputed §3.1's entire table, the
framing constants, the corpus values and both identifier orderings, and all of
them are exact. The predicted failure mode for this fold is therefore the
opposite of last time: not a wrong citation, but editing §2.4/§4/§6 into
agreement and leaving a *fourth* description of the error path behind.
