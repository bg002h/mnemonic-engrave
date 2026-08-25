# R0 — IMPLEMENTATION_PLAN_P1_me_container.md, round 6 (comprehension lens)

**The one question:** can an implementer who was not here read v8 and build the
right thing? I adopted the persona — a Rust engineer who knows `me` and has read
none of this cycle's reports — and walked §4 step 1 → step 12, then §2.4's table
and §3's vectors. Everything below is measured against the tree at `01ebb1e`.

The facts named settled in the brief were not re-derived. Where I make a numeric
claim, the command that produced it is in the header block.

---

## Commands run, and their raw output

```
$ ./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_P1_me_container.md | tail -3
─── citations resolved: 86 / 103 ; dangling: 17 ; ambiguous: 0
```
(confirms the settled figure, and it is what [M2] turns on.)

```
$ grep -rn 'MAX_SECTION_LEN' crates/ | grep '\.rs:'
crates/me-cli/src/seal/wire.rs:21:pub const MAX_SECTION_LEN: u32 = 8191;
crates/me-cli/src/sysw/wire.rs:42:pub const MAX_SECTION_LEN: usize = 8191;
crates/me-cli/src/sysw/wire.rs:133 :222      (both symbolic)
crates/me-cli/src/main.rs:978                (error text, symbolic)
```
No test pins `sysw`'s value as a literal, so step 1's raise is safe. Its doc
comment is not — see [M3].

```
crates/me-cli/src/sysw/wire.rs:40-42
/// EPD §6's cap, inherited unchanged: 8191 rather than 8192 because the
/// device's scan buffer signals overflow when it is exactly FULL.
pub const MAX_SECTION_LEN: usize = 8191;
```

```
crates/me-cli/src/sysw/record.rs:31-40   enum Class          -- 8 variants
crates/me-cli/src/sysw/record.rs:50-55   is_secret           -- Mnemonic|Codex32Secret|Passphrase
                                          => Class::MdMk is NOT secret
crates/me-cli/src/sysw/mod.rs:124        pub fn classify(record: &str) -> record::Class
                                          -- returns a bare Class, no Result
crates/me-cli/src/main.rs:1255-1300      fn sysw_error(e: &SyswError) -> String
                                          -- matches on SyswError VARIANTS
crates/me-cli/src/sysw/record.rs:195     fn unhex_lower(..)   -- PRIVATE to record.rs
crates/me-cli/src/sysw/record.rs:81-87   decode_body          -- strips TEXT_PREFIX|PASS_PREFIX only
```

```
crates/me-cli/src/main.rs:923-926
// Exactly one passphrase mode. clap enforces mutual exclusion; this
// is the "none given" case, and the DEFAULT is to generate rather
// than to leave a payload unprotected by omission.

crates/me-cli/tests/sysw_cli.rs:121-131
/// The default is to GENERATE, not to leave a payload unprotected by omission.
#[test]
fn omitting_every_passphrase_flag_generates_one() {
    me().args(["sysw", "pack", MD1])
        .assert().success()
        .stderr(predicate::str::contains("write this down"))
        .stderr(predicate::str::contains("12 words"));
}
```
`MD1` is `Class::MdMk`, which `is_secret()` returns false for. This is [I2].

```
$ grep -c 'fn ' crates/me-cli/tests/sysw_cli.rs        -> 31
Every `me sysw pack` invocation in that file passes records on ARGV except
`records_can_come_from_a_file_instead_of_argv` (:183). This is [M6].
```

```
$ grep -n 'reassembl\|mt_codec::decode\b\|DecodedSet' design/IMPLEMENTATION_PLAN_P1_me_container.md
350 356 357   -- E13/E19/E20 (all "without reassembling anything")
582 602 613   -- §2/§2.1 prose
622           -- §2.2's CHUNKS row: "reassemble via `mt_codec::decode`, THEN deserialise"
638 645       -- §2.2 prose
696 706 707   -- §2.2's dependency table
780 1044 1049 -- W5, V20, V25
1649          -- §6.1's command list
```
**Zero hits in the §2.4 wiring table's `change` cells.** This is [C1].

```
Archaeology, measured on the 1,859-line document:
  inline finding markers            169 occurrences, 78 distinct
  lines naming a prior draft (v1..v7)  136
  the word NORMATIVE                 29
  longest table rows, in words:
    §6.1 cite-check row  221 | step 4  201 | step 8  199 | E12  181 | W9  143
  §4 (the build instructions) begins at line 1250 — 67% into the document.
```

```
$ python3 -c "print(3 + 2*(75+222))"
597      <- V1's record length in characters; §3.3's example says pub_len 188. [M4]
```

```
Corpus check (V3's constructibility), $MT/crates/mt-codec/src/test_vectors/mt1_v1.json:
  generator: scripts/gen-mt1-vectors.py (mnemonic-engrave)
  even: size 222, txid 2dcf2b97…, set_id 0x2dcf2, 6 chunk strings present verbatim
V3 is constructible from the corpus. Good.
```

```
$ sed -n '34,44p' scripts/plan-wiring-check.sh
# WHAT IT DOES NOT DO -- a gate that hides its blind spot is worse than no gate
#   * It checks that a W is REFERENCED by a step, never that the step builds the
#     right thing, or in a feasible ORDER.
```
Confirms that [C1] and [I1] are outside all three gates by construction: both are
requirements with **no W token at all**, and the wiring gate only reasons over W
tokens.

---

## Part 1 — the walk

### Step-by-step, as a reader who has never seen this cycle

**Step 1 — raise `sysw::wire::MAX_SECTION_LEN` to 32,734.** Startable, finishable,
unambiguous. §4.1 names both constants and step 1 asserts both. The number itself
is not derived here, but spec §2.3 derives it (`(65536 − 52 − 16)/2`) and the
step states it outright, so nothing is guessed. One residue: [M3].

**Step 2 — stdin.** §4.2 is the best-transferring section in the document. It
rules precedence (`--in` > argv > stdin), names the branch being replaced
(`main.rs:1223-1225`), rules the TTY case, and states that a line holding a single
space survives as a record. I could write this test from the page. No finding.

**Step 3 — content-based sealing.** §4.3 is normative and ordered, and I could
write the four assertions. What I could **not** learn from the page is that
executing it turns a shipped test RED and reverses a documented default for
`md1`/`mk1` payloads. → **[I2]**.

**Step 4 — the codec, the manifest, the generator, the fixture, W14.** This is
the step that does not transfer. I cannot finish it: W14's conformance test is
defined against `me sysw pack` outcomes, and step 4's own row forbids
`me sysw pack` in the loop — and at step 4 `pack` refuses every `tx:` record
anyway. → **[C2]**. Secondary friction: the codec's module is never named
(**[M7]**), the generator is claimed by two steps (**[M1]**), and "27 rows" is not
the number of fixture entries the vectors need (**[M5]**).

**Step 5 — the two identifier fields.** Transfers cleanly. §1.1 and §1.1a each
state the field twice (table + subsection), the `bitcoin` calls are named
(`compute_txid().to_string()`, `compute_wtxid().to_string()`), and the trap is
spelled out. V18/V26's placement here is justified in the row itself. No finding.

**Step 6 — the wiring.** I can build W1, W2, W3, W5 and W9 from their cells. I
cannot build **W4** from its cell for `form = 0x02`: it says *"hex-decode, then
§2.2's DECODE, then `Class::Transaction`"*, and §2.2's DECODE for CHUNKS is
*"gather the record's set … reassemble … THEN deserialise"*, which the same
section says explicitly *"is not `classify`'s"*. → part of **[C1]**. Also: how
the failure leaves `classify`, whose signature is `-> Class`, is not stated
anywhere → **[I1]**.

**Step 7 — the error path.** Same blocker one layer on: W8 is *"`sysw_error`
gains the PER-RECORD arm for W11's `TxRecordError`"*, and `sysw_error` matches on
`SyswError` variants. No row declares a per-record `SyswError` variant — W12 is
explicitly the **set-level** one. → **[I1]**.

**Step 8 — the remaining rules.** Transfers. I checked the exception list against
the vector table by hand: V5→E7-positive, V6→E8, V9→E2, V10→E3/E4, V11→E5,
V12→E6, V13→E9, V14→E10, V16→E1, V17→E16-fee, V17b→E16-fp, V19→E18, V21→E14,
V22→E15. The excepted set (E7, E11, E12, E13, E17, E19, and E20 as out-of-range)
is exactly right. No finding — this is r5-I2 landed properly.

**Step 9 — V7 at the ceiling.** Constructible: §3.1 gives the arithmetic and §3's
exception blockquote gives the `OP_RETURN` free parameter. The only friction is
[M1]: this row still says it writes the generator that step 4 also writes.

**Step 10 — the set pass.** I can build E20 from W10's cell. I cannot tell, from
W10's cell or from any vector, that the CHUNKS form also owes a
reassemble-and-deserialise-and-compare-identifiers chain. → **[C1]**.

**Step 11 — the argv guard.** Buildable for `tx:`. Silent on whether a bare `mt1`
chunk on argv is refused, which §1.4a's ruling made a live question and nothing
in the document answers. → **[I3]**. Also the step that breaks earlier steps'
tests if they were written the local way → **[M6]**.

**Step 12 — E12's RED test in `sysw`.** Exemplary. It names the assertion, the
line whose change makes it go RED (`sysw/mod.rs:260`), and the test it must not
be confused with. No finding.

### §2.4's fourteen-row table — could someone build each site from its cell alone?

W1, W2, W3, W5, W9, W12, W13, W14 (modulo [C2]): yes.
W4: **no**, for `form = 0x02` ([C1]).
W10: **yes, and that is the problem** — the cell is complete and the requirement
is larger than the cell ([C1]).
W8, W11: **no** — the variant W8 matches on is declared by nobody ([I1]).
W6, W7: struck, prefixed **RETRACTED … DO NOT**, and each names its successor. I
do not believe a reader implements them. The retraction rows are fine.

### §3's 29 vectors — could someone construct each one, byte for byte?

Yes for 28 of them. The corpus supplies the transaction, both identifiers, the
set_id and the six chunk strings verbatim; §3.1 supplies V7's arithmetic; V27's
grind is specified with its free knob. **V8 is the exception** — its row has no
construction clause and its natural construction masks its own RED test →
**[I5]**. And the rule-name strings that half the rows assert on are enumerated
nowhere → **[I4]**.

---

## [C1] §2.2's CHUNKS DECODE — reassemble, deserialise, compare both identifiers — is owned by no wiring site and detected by no vector. Built from §2.4 and §4 alone, the anti-smuggling gate exists on the RAW form only, and every gate in §6 still closes green.

**Severity:** Critical.
**Where:** §2.2's CHUNKS row (:622); W4 (:779); W10 (:785); §6's W10 closure row
(:1425); §4 steps 6 and 10 (:1262, :1266); §6.2's decode-failure row.

**The failure, concretely.** §2.2 is NORMATIVE and says the CHUNKS decode is:
gather the set → require it complete and pristine → **reassemble via
`mt_codec::decode`** → **deserialise the result as a Bitcoin transaction** →
**both identifiers must match**. I then go to build it, and:

- **W4's cell** (`classify`) says *"hex-decode, then §2.2's DECODE"*. `classify`
  holds one record. §2.2 itself says the gathering *"is not `classify`'s … It is
  `split`'s, which is W10"*. So for `form = 0x02` the cell instructs something the
  function cannot do, and I must invent the split of labour.
- **W10's cell** says, in full: *"`split` gains a payload-level pass for **E20** —
  set membership, completeness, orphans."* Nothing about reassembly,
  deserialisation or identifier equality.
- **§6's W10 closure row** says: *"V25's three negatives all refuse; V3's complete
  payload packs."* Both are satisfied by an E20-only implementation.
- **No vector fails.** I read all 29 rows. There is no vector whose chunks
  reassemble to bytes that are not a transaction, and none whose reassembled
  transaction's txid differs from the metadata record's carried txid. V15
  perturbs the chunks' embedded `set_id` (R15); V25 is E20's three negatives; V27
  is the collision; V3 and V4b are positives. Delete the whole reassemble →
  deserialise → compare chain and **all 29 vectors stay green**.
- **§6.2's row** *"§2.3's decode-failure refusal | §4 step 7 | 4"* is one test,
  and step 7 does not say which form its record is. A RAW record satisfies it.

So the reader who builds from the wiring table and the TDD order — which is what
§2.4 exists to be — ships `me sysw pack` accepting a set of well-formed `mt1`
chunks carrying arbitrary bytes into the **public, unsealed** section. That is
verbatim the channel §2.1 measured and named C3: *"32 bytes of entropy → one
valid `mt1` string → exact round-trip, with an attacker-chosen `set_id` so R15
passes too"*, and the plan's own words for the fix are *"chaining the parse onto
the reassembly is what closes the CHANNEL C3 measured"*. It is chained in §2.2's
prose and in nothing an implementer builds from.

**Why the plan permits it.** The only place `decode` is bound to a site is a
parenthetical in §2.2's **dependency** table — *"the set gathering (W10) is
`decode`"* (:700) — three sections away from §2.4, inside a table about which
crate version to pin, and phrased as *gathering*, which is E20's word, not
reassembly's. §1.4a's ruling created W10 to answer E20; the CHUNKS decode chain
predates the ruling and was never re-homed when the body stopped being one
record. `plan-wiring-check.sh` cannot see this — its own header says it checks
*"that a W is REFERENCED by a step, never that the step builds the right thing"*,
and this requirement carries no W token at all. `plan-fold-sweep.sh` cannot see
it either: nothing was retracted, so no term was minted.

**A suggestion, not a prescription** (the defect is what matters): either W10's
cell states the whole chain, or a fifteenth row owns it — and either way it wants
a vector whose chunks reassemble to non-transaction bytes, because without one
its absence is undetectable.

**Confidence:** High. The absence was established by grepping the plan for
`reassembl|mt_codec::decode|DecodedSet` (10 hits, none in a §2.4 change cell) and
by reading all 29 vector rows and all 12 closure rows.

---

## [C2] Step 4 must build W14's conformance test, but §3.3's `expect` schema asserts `me sysw pack` outcomes — a packed blob, a rule name on stderr, an exit code — none of which exist until steps 6 and 7. Step 4's own row forbids `me sysw pack` in the loop. The step cannot finish as written, and the two ways out produce materially different fixtures.

**Severity:** Critical.
**Where:** W14 (:789); §3.3's schema example (:1220-1227); §3.3's *"Which step
files which vector"* (:1240); §4 step 4 (:1260); §6's W14 closure row (:1416);
§2.4's *"step 4 builds W14"* (:809).

**The failure, concretely.** Step 4 tells me to file 27 vector rows into
`crates/me-cli/testdata/tx_record_vectors.json` **"with W14's loader"**, and
§2.4 says *"step 4 builds W14"*. W14 is *"the fixture loader **and its conformance
test** … asserts each vector's outcome — a `pass` arm against **the packed blob**,
a `refuse` arm against **the rule name on stderr and the exit code**."* §3.3's
example schema carries the same shape:

```
"expect": { "refuse": { "rule": "magic", "exit": 4 } }
"expect": { "pass": { "blob": "...", "pub_len": 188, "sealed": false } }
```

Three of those five fields — `blob`, `pub_len`, `sealed` — are *container* outputs
(they are `sysw_vectors.json`'s field names verbatim; `crates/me-cli/src/sysw/vectors.rs:29-40`).
`exit` and *"on stderr"* are *process* outputs. So the conformance test as
specified needs `me sysw pack` to (a) accept a `tx:` record and (b) be observed as
a subprocess. At step 4:

- `me sysw pack` **refuses every `tx:` record** — `classify` has no `TX_PREFIX`
  branch until W4, which is step 6's. The plan establishes this itself for bare
  `mt1` in §1.4a's cost 1 and uses it to move V3 out of step 4 (r4-C2).
- step 4's own `then` column says *"**no `me sysw pack` in the loop**"*.
- `tx_vectors.rs` is a `src/` module. `src/` unit tests cannot observe a
  process's stderr or exit status; the repo puts that in
  `crates/me-cli/tests/sysw_cli.rs`.

So there is no reading of step 4 under which W14's conformance test can be
written and go green. **What two engineers do instead, and they diverge
materially:**

- Engineer A takes step 4's *"at the codec"* literally, drops `blob`, `pub_len`,
  `sealed` and `exit` from the schema, and asserts the `refuse` arm against a
  `TxRecordError` variant. Their fixture has no container bytes at all — and
  §6's *"the Go port reads `tx_record_vectors.json`"* now hands the Go port a
  file that pins the record codec and says nothing about packing.
- Engineer B keeps the schema as written, which forces the loader's assertions
  into `tests/sysw_cli.rs` and forces W14's conformance test to step 7 at the
  earliest — leaving step 4 with 27 committed vectors that nothing asserts, i.e.
  a fixture that cannot fail, which is the exact shape §6 struck under r2-I4 and
  r5-C1.

Both are defensible from the page. They are not the same artifact, and the file
is the **cross-language contract** §3.3 says it is.

**A second, independent gap in the same schema.** §3.3 rules that nothing
regenerates the fixture from `me` and that it *"is produced by
`scripts/gen-tx-record-vectors.py`, which **re-implements the framing**
independently"*, and §6 makes the closure check *"the generator reproducing the
committed fixture **byte for byte**"*. If the `pass` arm carries `blob` /
`pub_len` / `sealed`, that Python script must independently re-implement not the
framing but the whole `sysw` **container** — the 52-byte header, `pub_len`,
`ct_len`, the salt/IV/iterations fields and `bound()`. The plan never says so,
and *"the framing"* is the wrong word for it.

**Corroborating detail, measured:** the example's `"pub_len": 188` matches no
vector in this plan. V1 is *"RAW (segwit), no optional fields"*, so its record is
`3 + 2×(75 + 222) = 597` characters and a single-record public section is 597
bytes. The example's numbers were carried over from the fixture §3.3 spends forty
lines rejecting, without being recomputed.

**Why the plan permits it.** r5-C1 correctly retracted `sysw_vectors.json` as the
home. The replacement kept that file's **field names and assertion level** while
moving the vectors to a step where neither is reachable. This is the fold-authors-
the-defect pattern the plan itself flags: the newest text, least read.

**Confidence:** High. The step-4 refusal is the plan's own measured fact; the
field names are read at `sysw/vectors.rs:29-40`; the 597 is arithmetic from §1
and §3.

---

## [I1] W8 is *"the per-record arm for W11's `TxRecordError`"* — but no row declares the `SyswError` variant that arm matches on, and `classify` returns a bare `Class` that cannot carry the error out. The design §2.5a explicitly forbids is the one every closure row accepts.

**Severity:** Important.
**Where:** W8 (:783); W11 (:786); W12 (:787); §2.5a's three-row table (:876-879);
§6's W11 and W8 closure rows (:1420, :1423); `crates/me-cli/src/main.rs:1255`;
`crates/me-cli/src/sysw/mod.rs:124`.

**The failure, concretely.** `sysw_error` is `fn sysw_error(e: &SyswError) -> String`
and matches on `SyswError`'s variants. For W8's arm to exist, `SyswError` must
gain a variant carrying `(index, TxRecordError)`. Which row adds it?

- W11 adds `TxRecordError` **in `record.rs`**.
- W12 adds a `SyswError` variant and says, in bold, that it is the **SET-LEVEL**
  one and *"may NOT carry a bare `usize`"* — i.e. explicitly not this.
- W13 is the printer arm for W12's variant *"plus `sysw_error`'s OUTER match"*.
- §2.5a's summary table lists exactly three additions: W11, W12, W13.

Nothing declares the per-record variant. And the producing side is worse: W4 puts
the `tx:` parse inside `classify`, whose signature is `pub fn classify(record: &str) -> record::Class`.
A `TxRecordError` cannot leave that function. So I must choose:

1. change `classify` to `Result<Class, TxRecordError>` and fix its four call
   sites (`split`, `record::mdmk_unconfirmed`, `main.rs`'s
   `print_mdmk_confirmation`, and `classify`'s own tests) — a signature change no
   row names; or
2. leave `classify` returning `Class::Unknown` for a bad `tx:` record and re-run
   the parse in `split` to recover the reason — which is precisely what §2.5a
   forbids: *"The parse must fail with its reason, not be re-interrogated
   afterwards — that is the shape that produced r3-C1."*

**And option 2 passes every closure row.** §6's W11 row is *"a `tx:` record with
magic `MTX2` and a valid lowercase-hex body is refused naming the **magic** rule,
and the string 'not lowercase hex' does not appear"* — true of a re-parse in
`split`. §6's W8 row is the same assertion. So the architecture the plan spends
§2.5 and §2.5a establishing is not the architecture its gates measure.

**Why the plan permits it.** Rounds 2–4 hammered the *channel* — "the reason must
be carried, not re-derived" — and v5/v6 answered with the type (W11), the
set-level variant (W12) and the printer (W13). The **per-record** container
variant and `classify`'s return type are the two joints between them, and nobody
asked who owns a joint.

**A suggestion, not a prescription:** whatever the answer, it wants to be a row,
because §2.4 is what an implementer builds from.

**Confidence:** High. `sysw_error`'s signature and `classify`'s signature were
read at source; the three-row §2.5a table was read in full.

---

## [I2] Step 3 reverses a shipped, tested default for `md1`/`mk1` payloads — `omitting_every_passphrase_flag_generates_one` goes RED — and the plan never says a shipped test must change or that the rule reaches beyond transaction payloads.

**Severity:** Important.
**Where:** §4 step 3 (:1259); §4.3 rule 1 (:1301-1310);
`crates/me-cli/src/main.rs:923-926`; `crates/me-cli/tests/sysw_cli.rs:121-131`.

**The failure, concretely.** §4.3's rule 1 is unconditional: *"a payload holding
**no** `Class::is_secret()` record packs **UNSEALED**; one holding any packs
**SEALED**."* `Class::MdMk` is not secret (`record.rs:50-55`). So
`me sysw pack <md1 string>` with no flags, which today generates a 12-word
passphrase, must after step 3 pack unsealed. The shipped test that asserts the
current behaviour is:

```rust
/// The default is to GENERATE, not to leave a payload unprotected by omission.
#[test]
fn omitting_every_passphrase_flag_generates_one() {
    me().args(["sysw", "pack", MD1]) … .stderr(contains("write this down"))
```

and `main.rs:923-926` carries the same posture as a comment. §4's preamble
requires *"full suite green"* at every step, so step 3 cannot close without
deleting or rewriting a test whose doc comment states the opposite policy — and
the plan never mentions it, never lists it among the things P1 changes, and never
says the rule applies outside transaction payloads.

**What a reader does instead.** Engineer A reads §4.3 literally, deletes the
test, and ships a `me` in which **every existing `md1`/`mk1`-only invocation stops
generating a passphrase** — a live behaviour change to the shipped tool, made
inside a plan whose title is *"P1: `me`'s transaction container"*. Engineer B
sees a shipped test and a code comment both asserting the opposite, concludes
that content-based sealing is scoped to payloads containing a `tx:` record, and
builds a rule §4.3 does not describe. There is nothing on the page that settles
it, and the two ship different products.

**Why the plan permits it.** §4.3 was written to fix r1-I5 — *"v2 asserted only
precedence and never asserted the outcome in the case spec §2.4 actually rules"* —
and correctly made the base rule normative. Nobody then asked what the base rule
does to the records `me` already packs. It is a plan for a new record class that
silently re-rules the old ones.

**Confidence:** High. The test, the record constant and `is_secret` were all read
at source.

---

## [I3] §1.4a made the transaction ride as bare `mt1` records, and nothing re-ruled R2. Is `me sysw pack mt1p9h8… mt1q…` refused on argv? The plan does not say, and the two answers differ by whether a bearer transaction lands in shell history.

**Severity:** Important.
**Where:** §1.5's *"what runs BEFORE it"* row (:556); §4 step 11 (:1267); §6.2's
R2 row (:1583); §1.4a's cost list (:527-546); §7's consequence table (:1841-1850).
Spec §5's R2 (`design/SPEC_engrave_transaction.md:1533`) reads *"a `tx:` record on
argv"*, singular and prefix-shaped.

**The failure, concretely.** R2's stated rationale is *"argv is world-readable via
`/proc` and lands in shell history; **this material is bearer**"*. After §1.4a,
the material is no longer only in the `tx:` record — for the CHUNKS form the
`tx:` record carries an **empty** body and the transaction is carried entirely by
202 bare `mt1` sibling records. At step 11 I build the argv guard and have to
decide whether `me sysw pack mt1… mt1… mt1…` is refused. The document gives me:

- §1.5: the refusal is for *"a `tx:` record on argv"*;
- step 11: *"a `tx:` record on **argv** refused (R2) at EXIT 3"*;
- §1.4a's cost list: three costs, none of them this;
- §7's *"five things §1.4a's ruling touches outside this plan"*: `gui/scan.go`,
  two `SPEC_systemwide_payloads` tables, a device/host consistency note, and
  `me sysw show` — none of them this.

Engineer A guards `TX_PREFIX` only, and the whole signed transaction goes into
`~/.bash_history` one chunk at a time — the exposure R2 exists to prevent, on the
form §1.4a made the *default* for large spends. Engineer B extends the guard to
`ValidMT` records and hardens a path the spec never asked for. Both can cite the
document.

**Why the plan permits it.** R2 was inherited from a spec written when the chunks
lived inside one hex-encoded `tx:` body, where guarding the prefix guarded
everything. §1.4a moved the payload out from behind the prefix and its cost list
enumerates classification, record count and the payload-level pass — the three
things that broke *inside* `me`. The argv surface broke *outside* it and was not
swept.

**Confidence:** High for the gap (grepped both documents for `argv`; the spec has
one rule and it is prefix-shaped). The right answer is an operator's call, not
mine.

---

## [I4] The rule-name vocabulary is the cross-language contract — the fixture's `refuse.rule` string, `TxRecordError`'s variants, W8's operator line and the Go port must all agree — and the plan names five of roughly eighteen.

**Severity:** Important.
**Where:** W11 (:786); §2.5a's W11 row (:877); §2.5a's `hex` ruling (:903); §1.5's
E9 sentence (:576); V13 (:1036); §3.3's schema (:1222); §6's W14 closure row (:1416).

**The failure, concretely.** §6's W14 row makes the rule name the assertion:
*"the `refuse` arm checks the **RULE NAME**, not merely a non-zero exit — flip any
expected rule name in the fixture and the test goes RED."* The fixture is produced
by an **independent Python generator** (§3.3, NORMATIVE) and read by **both** the
Rust and the Go implementation (§3.3's cross-language paragraph). So the strings
are a wire contract between three artifacts written by three different hands.

The document supplies exactly five of them: `magic`, `version`, `form` (E9, and
V13 requires all three distinct), `body_len` (in W11's *"…"* list), and `hex`
(§2.5a's ruling). Every other refusable rule has none. W11 says only *"an enum
whose variants are the rules — one per E-number that can fail a single record"*,
which is E1, E2, E3, E4, E5, E6, E8, E10, E13, E14, E15, E16, E17, E18, E19 —
fifteen more names, unspecified.

Two engineers pick different spellings for the same refusal — `fee_len` vs
`tlv_width` vs `field_len` for E16, `tag_order` vs `ascending` for E1, `wtxid` vs
`wtxid_mismatch` for E17 — and the Python generator, written from the same page,
picks a third. The fixture then fails against both implementations, or (worse) the
same person writes generator and enum, the strings agree by authorship, and §3.2's
independence guarantee is quietly gone.

**Why the plan permits it.** The rule-name channel was invented in round 3 to fix
r2-C2 (*"the channel that carries a rule identity does not exist"*) and every
round since has argued about **whether** the name can be produced, never about
**what the names are**. E9's three are named only because V13 demanded three
distinct messages.

**A suggestion, not a prescription:** a rule-name column in §1.3's E-table would
put the vocabulary where the rules already are, and would cost one cell each.

**Confidence:** High. Counted by hand off §1.3's twenty rules and cross-checked
against every occurrence of *rule* in §2.5a and §6.

---

## [I5] V8's row has no construction clause, and the natural construction masks its own RED test — the exact defect V15 and V27 each needed a review round to fix.

**Severity:** Important.
**Where:** V8 (:1031); V15's NORMATIVE clause (:1038); V27's NORMATIVE clause
(:1051); §4 step 10 (:1266).

**The failure, concretely.** V8 is, in full: *"RAW whose carried txid ≠ the body's
| the §2.2 consistency refusal"*. §2.2 now requires **two** equalities on the RAW
form — txid **and** wtxid (E17). So how I build V8 decides whether it can go RED:

- Perturb **only the carried txid**, leaving the wtxid honest → deleting the txid
  comparison lets the record through, V8 stops refusing, the test goes RED. The
  vector works.
- Build the record from a **different transaction's identifiers** — the obvious
  construction for *"carried txid ≠ the body's"*, and the one a generator writing
  a "wrong metadata" case reaches for — → both identifiers now mismatch, **E17
  refuses it on its own**, and deleting the txid comparison leaves V8 green. The
  txid equality has no RED test, and §6's *"every rule … has a test that goes RED
  without its check"* is satisfied by a vector that cannot fail.

This document has now been through this twice. V15 carries *"**NORMATIVE, because
it decides whether the vector can go RED (r2-M4)**: the perturbation is applied to
the CHUNKS' EMBEDDED `set_id`, leaving the carried txid HONEST"*. V27 carries
*"**NORMATIVE (r4-I1), because it decides whether the vector can go RED — the same
clause V15 carries**"*. V8 sits between them with no clause at all, and E17 —
added in this very cycle by r2-C1 — is what turned it into a masking case.

**Why the plan permits it.** E17 was added to §1.3 and vectored by V18/V26. V8
predates it (round 0) and was never re-read against the rule that now shadows it.
This is the *"a diff falsifies text it never touches"* shape: adding E17 changed
what V8 proves without editing V8.

**Confidence:** High. Established by reading V8, V15, V27 and E17 together; no
execution needed, the masking is structural.

---

## [M1] `scripts/gen-tx-record-vectors.py` is assigned to step 4 **and** to step 9, and §3's exception blockquote still says it is *"committed in §4 step 9"*.

**Severity:** Minor.
**Where:** §3's V7 exception (:963); §3.3's *"Which step files which vector"*
(:1240); §4 step 4 (:1260); §4 step 9 (:1265); §6's §3.2 closure bullet (:1440).

r5-I5 moved the generator to step 4 in §3.3, in step 4's row and in §6's bullet.
Step 9's `then` column still reads *"**and this is the step that writes and
commits `scripts/gen-tx-record-vectors.py`** (r2-I3), because V7 has no other
input"*, and §3's blockquote still reads *"generated by
`scripts/gen-tx-record-vectors.py` — new in this repo, **committed in §4 step
9**"*. A reader who meets §3 before §4 — the document's own order — learns that
the generator arrives at step 9, and then step 4 asks them to file 27 vectors
with it. Cost is confusion, not a wrong build, because step 4 states its own
answer in bold. Round 5's I5 is **PARTIAL** on this half.

---

## [M2] §6.1's cite-check row states two different citation counts in the same cell: *"86 of 103 resolve"* and *"90 is now the RESOLVING count against a total of 107"*.

**Severity:** Minor.
**Where:** §6.1's gate table, cite-check row (:1566).

Measured on v8: `86 / 103 ; dangling: 17`. The cell's PASS column carries that.
Eight lines later the same cell says *"the citation TOTAL has climbed each round —
90 → 98 → 107 — rather than staying put. **(r5-M3) 90 is now the RESOLVING count
against a total of 107**"*, in the present tense. Both cannot be v8's. r5-M3 asked
for exactly this to stop happening; the fold corrected the PASS column and left
the trend sentence describing v7. Round 5's M3 is **PARTIAL**.

---

## [M3] Step 1 says *"raise **only** `crates/me-cli/src/sysw/wire.rs:42`"*, which leaves the doc comment two lines above asserting the value it just changed.

**Severity:** Minor.
**Where:** §4 step 1 (:1257); `crates/me-cli/src/sysw/wire.rs:40-42`.

```
/// EPD §6's cap, inherited unchanged: 8191 rather than 8192 because the
/// device's scan buffer signals overflow when it is exactly FULL.
pub const MAX_SECTION_LEN: usize = 32_734;   // after step 1, read literally
```

The word *"only"* is there for a good reason (§4.1: do not touch `seal`'s), but a
reader taking it literally ships a comment that states the old value, cites EPD §6
as authority for it, and gives a device-buffer rationale that no longer applies —
while the spec's own note (`SPEC_engrave_transaction.md:1710`) is *"`MaxSectionLen`
→ 32,734 for flash; **NFC keeps 8191**"*, which is the fact that comment should now
carry. This repo has a memory entry for exactly this class (*comments outlive their
conditions*).

---

## [M4] §3.3's schema example is not an instance of any vector in this plan.

**Severity:** Minor.
**Where:** §3.3 (:1220-1227).

`"name": "V1-raw-roundtrip"`, `"note": "the corpus even transaction, RAW form"`,
`"pub_len": 188`. V1 has `n_fields = 0`, so its record is `3 + 2×(75 + 222) = 597`
characters and a single-record public section is 597 bytes, not 188. The three
field names (`blob`, `pub_len`, `sealed`) are `sysw_vectors.json`'s, copied from
the file the surrounding section rejects. A reader building the fixture from the
example inherits both the wrong shape ([C2]) and a number that cannot be checked.

---

## [M5] Step 4's *"27 rows"* is a count of plan table rows, not of fixture entries, and roughly a dozen required entries have no row of their own.

**Severity:** Minor.
**Where:** §4 step 4 (:1260); §3.3 (:1240); §6's near-miss bullet (:1478-1490).

V13 is *"bad magic / unknown version / `form = 0x03`"* — three refusals with three
distinct rule names, so three fixture entries. V17 is `len=2`, `len=7`, `len=9`
refused and `len=8` passing — four. V17b three, V19 two, V23 two, V22 two, V24
two, V25 three, V27 two records. A reader who files *"27 rows"* as 27 JSON entries
loses most of §6's near-miss pairs, which §6 requires by name. The near-miss
bullet is the recovery, but it is 220 lines away and the count in step 4 reads as
authoritative.

---

## [M6] Every `me sysw pack` test in `sysw_cli.rs` passes records on argv; steps 6, 7 and 10 say only *"`me sysw pack` on a real `tx:` record"*, and step 11 then refuses `tx:` records on argv.

**Severity:** Minor.
**Where:** §4 steps 6, 7, 10, 11; `crates/me-cli/tests/sysw_cli.rs` (30 of 31
tests use argv).

Written the way the file's local convention writes them, steps 6, 7 and 10's tests
go RED at step 11, violating *"full suite green"* three steps in a row. The fix is
trivial once seen (`--in` or stdin, both available from step 2), and §1.5 does warn
the attentive reader that R2 exists. But no step that packs a `tx:` record names
its input channel, and the document's own §4.2 establishes three of them.

---

## [M7] The record codec — the thing steps 4, 5 and 8 build — is the one artifact in this plan with no named home.

**Severity:** Minor.
**Where:** §2.4's header (:775, *"NORMATIVE — FOURTEEN sites, and **none of them
is the codec**"*); §4 step 4's `then` column (*"the layout codec"*); W11 (:786).

`TxRecord` appears once in the whole document, in W11's cell, with no field list
of its own (§1's layout table is the de-facto struct, which is fine). `TxRecordError`
is homed at `crates/me-cli/src/sysw/record.rs`. The parse/serialise functions have
no module, no names and no signatures, and `unhex_lower` — the hex decoder W4's
cell calls for — is **private** to `record.rs` (`:195`), so the branch as sited in
`mod.rs` cannot call it. Every resolution converges on *put the parse in
`record.rs` and have `classify` call it*, which is almost certainly the intent, but
it is inferred rather than read. Related: §2.4's header still says *"FOURTEEN
sites"* while §6 and §2.4's own summary sentence say **twelve live** — defensible
as a row count, mildly jarring as a header.

---

## The archaeology question, answered directly

**Does the history drown the instruction? Not quite — but it is close in §4,
which is the section an implementer actually works from.**

Measured: 169 inline finding markers (78 distinct), 136 lines naming a prior
draft, and §4 — the build instructions — begins at line 1250 of 1859, 67% in. The
two longest step rows are step 4 (201 words) and step 8 (199 words). Step 8's
instruction is *"V5–V6, V9–V14, V16–V17b, V19, V21, V22"* and *"every rule in
E1–E19 EXCEPT E7, E11, E12, E13, E17 and E19"* — about 25 words. The other ~174
are which round moved which vector where and why. Step 4's ratio is about 55
instruction to 145 history.

**The instruction survives, and here is why I say so rather than complain about
density:** in every step row and every wiring cell, the instruction is set in
bold and the history is in parentheses attached to a marker, so the two are
visually separable on a first read. Where the history is load-bearing — E11's
*"this is a canonicality rule and nothing more"*, V15's and V27's NORMATIVE
construction clauses, W6/W7's retractions — it is doing work no shorter sentence
could. The retracted rows are struck **and** prefixed **RETRACTED … DO NOT**, and
each names its successor; I do not believe a reader implements one.

The three places the archaeology actually costs something are [M1] (a stale
sentence in the narrative contradicting the corrected step), [M2] (a stale count
inside the cell whose whole subject is counting), and [M5] (a row count that reads
as a fixture count). All three are the same failure mode: **the correction landed
in the instruction and not in the story told about it.**

**On NORMATIVE:** 29 uses, and I found no ambiguous one. It is consistently
attached to a rule the implementer must obey, and twice (V15, V27) to a *vector
construction*, where the document explains why the construction is normative. That
is a good use, not a loose one.

---

## Part 2 — did round 5's twelve findings land?

| # | verdict | evidence |
| --- | --- | --- |
| **C1** — vectors homed in `sysw_vectors.json` | **FIXED** | §3.3 rewritten; `crates/me-cli/testdata/tx_record_vectors.json` + `crates/me-cli/src/sysw/tx_vectors.rs` are NORMATIVE (:1232-1236); W14's row explicitly says *"NOT `coverage.rs`"*. The replacement carries its own defect — see [C2] — but the finding is closed. |
| **C2** — W8 at step 6, W11 at step 7 | **FIXED** | §2.4 (:809-811) now reads *"step 7 builds W8 and W11–W13 … **(r5-C2) W8 is step 7's, not step 6's**"*; step 6's row says *"NOT W8, which is step 7's"*; the two refusal-message clauses now sit in step 7's row. |
| **I1** — §6's §3.2 closure bullet was a false PASS | **FIXED** | Replaced (:1436-1448) by a check that can fail: *"the generator reproducing the committed fixture byte for byte from a clean checkout"*, with v7's absence-based compensator quoted and struck. |
| **I2** — step 8's rule range | **FIXED** | Now *"E1–E19 EXCEPT E7, E11, E12, E13, E17 and E19"* plus E20 named as out-of-range. I checked the residue against the vector table by hand: the fourteen rules left in step 8 each have a vector in step 8's list. |
| **I3** — *"each of the FOURTEEN"* | **FIXED** | :806 now reads *"for each of the **twelve live sites** — W6 and W7 are retracted and are not sites at all (r5-I3)"*, and §6's assertion table has exactly twelve rows. §2.4's header still says *"FOURTEEN sites"*, which is a row count — noted in [M7], not a re-break. |
| **I4** — `NonHexBody` near-miss with no producer | **FIXED** | §2.5's near-miss paragraph rewritten (:884-905) and closed NORMATIVE: *"`NonHexBody(\"tx:\")` is not a value this plan ever produces, and no step asserts it"*; the replacement (`hex` rule via W11/W8) is asserted by step 7. |
| **I5** — no step files V7 or V27; step 4 uses step 9's generator | **PARTIAL** | The filing half is fixed (§3.3 :1240 assigns V1–V6/V8–V26 → step 4, V7 → step 9, V27 → step 10). The generator half is not: step 9 (:1265) and §3's V7 blockquote (:963) both still say step 9 writes and commits it. → [M1]. |
| **M1** — *"THREE steps, not one"* then four | **FIXED** | :808 reads *"split across FOUR steps, not one (r4-C1, r4-C2; count corrected r5-M1)"*. |
| **M2** — *"FIVE are not"* surviving above the blockquote | **FIXED** | :814 reads *"**(r3-M3, corrected r5-M2) EIGHT are not** … and this sentence said FIVE"*, with the residue arithmetic (29 − 8 = 21) consistent in both places. |
| **M3** — two gate rows carrying counts the fold changed | **PARTIAL** | The table-check and wiring rows are current. The cite-check row now carries **both** *"86 of 103 resolve"* and *"90 is now the RESOLVING count against a total of 107"*. Measured on v8: 86/103. → [M2]. |
| **M4** — §6.3's enumeration blockquote stale in two places | **FIXED** | The tally table (:1826-1836) is re-derived to eight entries; rows 5–8 carry this fold's figures (*"29 rows, 8 exceptions, so 21"*, *"four"*, *"twelve live sites"*, *"twenty-seven, then thirty-five"*). |
| **M5** — §3.3 calling W14 a *"FOURTEENTH wiring site"* naming only `coverage.rs` | **FIXED** | `grep -n 'FOURTEENTH\|fourteenth'` returns nothing; §3.3 now reads *"W14 is the loader and its conformance test (§2.4), **not `coverage.rs`**"*, matching W14's two-file row. |

**Score: 10 FIXED, 2 PARTIAL, 0 NOT FIXED, 0 WRONGLY FIXED.** The two PARTIALs are
both the same shape — the correction landed in the instruction and the narrative
around it kept the old value — and both are Minor.

---

## Verdict

**2C / 5I / 7M. v8 is NOT GREEN.**

Both Criticals are transfer failures rather than reasoning failures, which is what
this lens was for: **[C1]** is a normative requirement (§2.2's CHUNKS decode) that
appears in prose and in no buildable unit, is asserted by no vector and no closure
row, and is structurally invisible to all three document gates — so an implementer
building from §2.4 and §4 ships the plan's own measured smuggling channel with
everything green. **[C2]** is round 5's own fix: the vectors got a correct new home
and kept the rejected file's assertion level, so step 4 cannot build what §2.4
assigns it and the two ways out produce different cross-language contracts.

The caution in the brief held again: **both Criticals live in text `git diff
0409815..01ebb1e` created or left standing** — [C2] entirely in the §3.3 rewrite,
[C1] in the W10 row that §1.4a's ruling created and that the r4/r5 folds edited
around without re-reading what §2.2 still demands.

Five rounds of correctness review left this document **factually excellent and
structurally teachable** — §4.2, §4.3, step 5 and step 12 are as good as plan
prose gets, and the retraction machinery does not drown the instruction. What it
has not been asked until now is whether the *set* of buildable units covers the
*set* of normative requirements. It does not, in two places.
