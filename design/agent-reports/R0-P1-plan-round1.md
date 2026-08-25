# R0 — `IMPLEMENTATION_PLAN_P1_me_container.md`, round 1

**Reviewer:** independent R0 agent, round 1. **Artifact:** `design/IMPLEMENTATION_PLAN_P1_me_container.md`
(324 lines, v2). **Prior round:** `design/agent-reports/R0-P1-plan-round0.md` (5C/13I/5M).
**v1 → v2 is `git diff HEAD~2 HEAD~1` (+230 / −112).**

**Machine-checked before writing** (nothing below rests on reading a doc comment):
the plan's `INTERNAL`/`DISPLAY` occurrences (`grep`); the deleted v1 §1.3 body table
(diff); the word "opaque" across `SPEC_engrave_transaction.md` and its git history
(`git log -S`); `sysw::wire::MAX_SECTION_LEN` / `seal::wire::MAX_SECTION_LEN` line
numbers; `seal/mod.rs:171` and `main.rs:1141`; `Class` and `Class::is_secret`;
`sysw::classify` and `unknown_reason`; `encode_section`'s `join("\n")` and its
no-trailing-LF test; `MAX_RECORD_LEN`/`MAX_RECORDS` scope; `--allow-weak`'s
implementation; `me-cli`'s manifest; `mt-codec`'s manifest, `lib.rs:9-14`,
`pipeline.rs:17-27/53`, `to_symbols`; `SPEC_mt_v0_1.md:680` and `:3546-3549`;
`scripts/gen-mt1-vectors.py`'s location; **and I EXECUTED
`scripts/plan-build-gate.sh` against this plan** (see I8). All §3.1 arithmetic
recomputed in Python.

---

## Part 1 — did each of the 18 findings land?

| # | verdict | the sentence that settles it |
| --- | --- | --- |
| **C1** byte order | **NOT FIXED** | The normative layout table at **line 39** still reads `` 6   32    txid        INTERNAL byte order -- see §1.1, this is where bugs live `` — sixteen lines above §1.1's `STANDARD DISPLAY ORDER`. Round 0's C1 was *"two contradictory instructions with no rule saying which wins"*; both halves are still present, with their polarity swapped. See **[C1]** below. |
| **C2** txid ≠ wtxid | **FIXED** | §1.1: *"computed over the transaction with marker, flag and witnesses STRIPPED"*, plus §3's V4 *"Segwit is required: for a legacy transaction txid == wtxid and the vector passes in both worlds."* (The *body's* serialisation form was deleted in the same edit — that is **[C2]** below, a new defect, not this one.) |
| **C3** CHUNKS decode proves nothing | **FIXED** | §2.2: *"CHUNKS | **reassemble via `mt-codec`, THEN deserialise the result** as a Bitcoin transaction"*, plus the txid equality. The BCH-verifier-only channel round 0 executed is closed. (The residual is **[I3]** below.) |
| **C4** TLV order undefined | **FIXED** | E1: *"**TLVs appear in ASCENDING TAG ORDER.**"* (Its *vector* is still an instance — **[I1]** below.) |
| **C5** trailing bytes | **FIXED** | E3 + E4: *"The record ends where the body ends. Trailing bytes are REFUSED"* / *"`39 + Σ(3 + len) + 4 + body_len` MUST equal the decoded length exactly"*, with V10. **E4's formula is arithmetically correct** against §1's offsets (4+1+1+32+1 = 39; TLV = 1+2+len; +4 for `body_len`). |
| **I1** duplicate tags | **FIXED** | E2 + V9: *"A tag appears AT MOST ONCE. A duplicate is REFUSED."* |
| **I2** `body_len` unbounded | **FIXED** | E5 + V11: *"`body_len` is validated against the remaining length BEFORE any allocation."* |
| **I3** fixed-width tags carry a variable `len` | **PARTIAL** | The *zero-length* half landed (E6 + V12). The *wrong-length* half did not: §1.2 says *"exactly 8 bytes"* / *"exactly 4 bytes"* in a **tag table**, and **no rule in E1–E10 and no vector covers `tag=0x02, len=2`** — so §6's *"Every rule E1–E10 has a test that goes RED without its check"* never reaches it. See **[I11]**. |
| **I4** bad magic/version/form | **FIXED** | E9 + V13: *"A bad `magic`, an unknown `version`, or a `form` outside {0x01, 0x02} is REFUSED, each with its own message."* |
| **I5** `me` gains a `bitcoin` dep | **PARTIAL** | §2.2 now says *"**(I5) `me` gains a `bitcoin` dependency.**"* — but it names no version, makes no choice (*"`bitcoin` is a public crate; `mt-codec` already declares it"* — declared **unused**, at 0.32), and **no step in §4 adds it**. The gap round 0 named ("not in the plan at all") is closed; the decision is not made. |
| **I6** publication deadlock | **FIXED** | §5: *"publish `mt-codec` 0.1.0 to crates.io, then depend on the pinned published version. No path dependency, no git dependency, no ordering knot."* Executable as written: GREEN → publish → step 1. |
| **I7** V7 unconstructible + §2.3's falsified numbers | **PARTIAL** | V7 is now *"body at **16,322 B minus the fields present**"* and §3.1 tabulates the third ceiling — **recomputed and correct**: `(32734−3)//2 = 16365`, `−43 = 16322`. But round 0 falsified **two** numbers in spec §2.3 and §6 owns only one: the table row *"5/2 | 4,080 B | … ✅ (**raw-only at 8191, by 31 chars**)"* is still false — `3 + 2×(43+4080) = 8,249`, **58 over 8191** — and appears in no correction list. |
| **I8** refuse vs warn | **FIXED** | §2.3: *"**NORMATIVE: `tx:` follows `seal`'s posture — refuse.**"* Both cited call sites verified: `seal/mod.rs:171` is `record::decode_public_set(&refs)`, `main.rs:1141` is the `mdmk_unconfirmed` loop under the doc comment *"then the container is built anyway"*. (*What* "refuse" means concretely is **[I4]** below.) |
| **I9** passphrase-flag precedence | **PARTIAL** | §4.3 rules *"an explicit flag **always wins**; content decides **only** in their absence"*, which does settle round 0's three named cases. But §4.3's own list of four includes **`--allow-weak`**, which `main.rs:190-192` documents as *"Accepted and ignored"* and `main.rs:893-895` prints *"me: --allow-weak is accepted and ignored"* — so the rule as written makes a documented no-op a sealing determinant. See **[I9]**. |
| **I10** stdin TTY hang | **FIXED** | §4.2: *"**NORMATIVE: if stdin is a TTY and neither `--in` nor argv records are given, refuse with a message naming both real inputs.**"* — and it names why R7's test cannot catch it. |
| **I11** step 1's constant | **FIXED** | §4.1: *"`crates/me-cli/src/seal/wire.rs:21` … `<- FROZEN. Do not touch.` / `crates/me-cli/src/sysw/wire.rs:42` … `<- this one`"*, and *"**Step 1's test asserts BOTH**"*. Both line numbers verified exact. `boundBlob` is gone. |
| **I12** self-generated vectors | **FIXED** | §3.2: *"**hand-constructed or independently generated, never dumped from the encoder under test**"*, plus §6's *"The vectors were not produced by the code they judge"*. `mt-codec/src/lib.rs:9-14` quoted accurately; `scripts/gen-mt1-vectors.py` exists (in **this** repo). |
| **I13** R15 has no negative vector | **FIXED** | V15 added: *"**R15 NEGATIVE: a chunks record whose carried txid's top 20 bits ≠ its chunks' `chunk_set_id`**"*. (Its justification clause is false — **[M1]**.) |

**Tally: 13 FIXED / 4 PARTIAL / 1 NOT FIXED.**

---

## Part 2 — defects in the rewrite

## [C1] §1's normative layout table still says the txid is INTERNAL

**Severity:** Critical.
**Where:** plan line 39, the `off/size/field` block in §1 — against §1.1 (line 55) and §2.2 (line 154).

**The failure, concretely.** Two implementers open §1 to build the record. The layout
block is the only place in the document that states the field's **size, offset and
order together**, and it is the block a Go porter transcribes, because it is the only
part of the plan shaped like a struct. It says:

```
 6   32    txid        INTERNAL byte order -- see §1.1, this is where bugs live
```

The implementer who follows it writes the unreversed `double-SHA256` output. §1.1,
sixteen lines below, says the opposite in bold. The plan states no rule about which
of its own sentences wins. This is round 0's C1 **unchanged in structure** — a
normative claim and its negation, coexisting — with only the polarity of the two
halves swapped, and the consequence round 0 measured is the same one: `mt-codec`
stamps `chunk_set_id = 0x2dcf2` for the corpus's *even* transaction, the
internal-order field yields `0x30f6e`, and R15 refuses a byte-perfect record on the
machine after the container has been flashed.

**Why the plan permits it.** The rewrite edited §1.1's prose and the sections that
cite it (§2.2 line 154, V4 line 188, step 5 line 244 — all four now say *display*)
and **did not touch the table row**. `grep -n "INTERNAL\|DISPLAY"` over the current
file returns exactly one `INTERNAL`, at line 39, and it is the one a struct is built
from. The plan's own §0 supplies the argument against leaving it: *"A format defined
in code first is a format nobody reviewed"* — here the format is defined in two
places and they disagree.

**Confidence:** High. Grepped; the line is in the committed blob
(`8accf364c27dfd410d31fde60440010e`).

---

## [C2] The rewrite DELETED the definition of what the body contains, and §1.1 still cites the deleted text

**Severity:** Critical.
**Where:** plan §1.3 (v1's *"The body"* table, removed by the rewrite), against §1.1's
(C2) paragraph at line 83 and §2.2's decode table.

**The failure, concretely.** v1 §1.3 said:

```
| `0x01` RAW    | the serialized signed transaction, **with witness** |
| `0x02` CHUNKS | the `mt1` strings, **LF-separated**, ASCII           |
```

The rewrite replaced that section wholesale with the E1–E10 encoding-rules table.
**v2 contains no statement of what the body holds in either form.** `grep -n
"witness\|LF\|ASCII"` over the current file: `witness` appears only in §1.1's
*txid-computation* rule and in the narrative sentence at line 83; `LF` appears only
inside V3's "pins" cell; `ASCII` appears nowhere. All §1 has left is the parenthetical
`0x01 = RAW (transaction bytes) | 0x02 = CHUNKS (mt1 strings)`.

**RAW.** Implementer A emits the segwit serialisation (marker, flag, witnesses).
Implementer B emits the legacy serialisation — the same bytes the txid is computed
over, which §1.1 now makes the salient one in the reader's mind. Walk both through
every check the plan has: `magic`/`version`/`form` pass; E1–E10 pass (they constrain
only the TLVs and the lengths); §2.2's *"deserialise the body as a Bitcoin
transaction"* passes for both, because a witness-free serialisation is a valid
transaction; and §2.2's txid equality passes for both, **because the txid strips
witnesses anyway**. Two conforming records, different bytes, and B's plate carries a
transaction with **every signature removed**. Nobody can broadcast it. That is the
one thing this artifact exists to make possible, and the plan has no check that
fires.

**CHUNKS.** The separator, the presence of a trailing separator, and the case are all
unruled. `mt-codec`'s `to_symbols` does `s.trim().to_ascii_lowercase()`
(`pipeline.rs:66-67`), so **it accepts uppercase and whitespace-padded `mt1`
strings** — an encoder that emits uppercase produces a different `body_len`, different
record hex, a different EPD §6.6 public-data hash, and still round-trips through
`mt_codec::decode`. An encoder that appends a trailing `\n` yields an empty final
element on split, which the other implementation's decoder rejects: **one accepts what
the other refuses.**

**Why the plan permits it.** V3's *"LF separation"* is a **vector cell**, and the
plan's own C4 lesson is that *"a vector is an instance, not a rule"* — §1.3 was
rewritten specifically to convert instances into rules and then dropped the two rules
it already had. The dangling citation is the evidence the deletion was unintended:
line 83 still reads *"v1 said … while **§1.3 says the RAW body carries the
witness**"*, and §1.3 no longer says it.

**Confidence:** High. Diff read; `to_symbols` read.

---

## [C3] Nothing in the plan creates `ClassTransaction` — §6's gate closes GREEN while `me sysw pack` still refuses every `tx:` record

**Severity:** Critical.
**Where:** plan §4's TDD table (steps 1–9) and §6's closure list, against spec §6's
P1 row and plan §0.

**The failure, concretely.** P1 is implemented exactly as §4 says. Step 1 raises
`sysw/wire.rs:42`. Steps 4–8 build the record codec and V1–V15 pass. Step 9 adds the
argv guard. `cargo nextest run --locked` is green, clippy is clean, the vectors were
independently generated, `mt-codec` is published. **Every bullet in §6 is satisfied.**
Then:

```
$ mt encode --record --raw < tx.final.psbt | me sysw pack
me: record 0 ... unclassifiable
```

because `sysw::classify` (`crates/me-cli/src/sysw/mod.rs:124`) tests exactly two
prefixes — `PASS_PREFIX`, `TEXT_PREFIX` — then bip39, then `seal::record::
validate_record`, and returns `Class::Unknown` for anything else; `sysw::split`
(`mod.rs:255`) turns `Class::Unknown` into `SyswError::Unclassifiable(i, …)` and packs
nothing. `Class` (`sysw/record.rs:32-41`) has eight variants and no `Transaction`.
`unknown_reason` (`mod.rs:108-115`) iterates `[PASS_PREFIX, TEXT_PREFIX]` only, so
even the error message is wrong — a `tx:` record with a bad body reports
`Unrecognised` instead of naming its prefix.

**Why the plan permits it.** `ClassTransaction` is the **first item** in spec §6's P1
row and in plan §0's statement of scope, and it appears in **no step of §4 and no
bullet of §6**. §4's nine steps cover the constant, stdin, sealing precedence, the
codec, the vectors, the ceiling and the argv guard — every component — and none of
them is the call that joins the codec to `me sysw pack`. V1–V15 are record-level
vectors: they exercise the codec directly and are all green with `classify`
untouched. So the plan's closure gate is satisfiable **on the defect it exists to
prevent** — the shape spec §7 names in its own words: *"a close condition that could
pass on the defect it exists to catch."*

(Arguably Important rather than Critical; I file it Critical because the gate passes,
not because the omission is subtle.)

**Confidence:** High. All four sites read; the enum, the classifier and the reason
function all resolved.

---

## [I1] E1 has no negative vector, so §6's own closure condition is unsatisfiable for it

**Severity:** Important. **Where:** plan §3's V-table (E1 → V2) against §6.

**Concretely.** §6 requires *"Every rule E1–E10 has a test that goes RED without its
check."* E1's only vector is **V2 — `RAW, all three optional fields`**, a *positive*
vector whose bytes are ascending by construction. Delete the decoder's ordering check
entirely and V2 still passes: its input was never out of order. So no test goes RED,
and the closure condition cannot be met as vectored.

The consequence is not cosmetic. A decoder that *accepts* `0x03, 0x02, 0x01` is
conformant to every vector in the list. Rust `me` (built from the rule) refuses it;
the Go port (built from the vectors, which is what §3 says the port is judged
against) accepts it. **The Go port admits a record `me` refuses** — the divergence E1
was written to close, one layer out. Every other rule got a purpose-built negative
(E2→V9, E3/E4→V10, E5→V11, E6→V12, E8→V6, E9→V13); E1 alone did not.

**Confidence:** High.

## [I2] "REFUSED" is used eight times and defined nowhere — no layer, no exit code, no scope

**Severity:** Important. **Where:** E2, E3, E5, E6, E8, E9, E10, §2.3; and §4 steps 6 and 8.

**Concretely.** The plan says REFUSED in eight places and never says what a refusal
*is* here. Three questions have to be answered before step 6's or step 8's test can
be written, and none is:

- **Which layer.** Codec-level `Err`, or a `me sysw pack` exit? Spec §5 is explicit
  that *"Where a refusal RUNS is part of the refusal"* and *"For every refusal above,
  name what runs BEFORE it"* — a rule this repo added after `mt`'s §8.2f was bypassed
  by the invocation it refused. The plan applies it to nothing.
- **What scope.** `me sysw pack` is given five records, one of which is a malformed
  `tx:`. Does it exit non-zero writing nothing, or pack the other four? The container
  it extends does **both** today depending on the failure: `split` **aborts the whole
  pack** on `Class::Unknown` (`sysw/mod.rs:255`), while `report_unconfirmed`
  **warns and packs anyway** (`main.rs:1141`). §2.3 rules "refuse" without saying
  which of those two shapes it means.
- **What the operator sees.** E9 requires *"three distinct messages"* and V13 asserts
  them; there is no statement of what stream they go to or what the exit code is,
  though `me`'s existing pack path uses stderr + exit 4 (`main.rs:975-983`).

Step 6 is *"V5–V6, V9–V14 → every rule in E1–E10"*. Twelve of those fifteen vectors
assert a refusal, and the assertion cannot be written.

**Confidence:** High.

## [I3] §2.2 claims to "close C3", and a valid transaction is itself an arbitrary-byte container

**Severity:** Important. **Where:** plan §2.2, *"Chaining the parse onto the reassembly is what closes C3"*.

**Concretely.** The attacker builds a real, deserialisable transaction with one
output whose `scriptPubKey` is `OP_RETURN <32 bytes of seed>` (or simply a
scriptPubKey that *is* the seed — nothing evaluates it). They compute its txid
honestly and put it in the field. Then:

- the body deserialises as a Bitcoin transaction → §2.2's RAW row passes;
- its txid equals the carried field → §2.2's equality passes;
- E1–E10 pass; the record is well-formed;
- `me` admits it to the **public** section, and `picotool save` reaches those 32
  bytes **with no passphrase** — the precise outcome §2 opens by naming.

Nothing is signed, nothing is verified against a UTXO set, and nothing bounds what a
`scriptPubKey` may contain. The chained parse raises the attacker's cost from *any
32 bytes* to *any 32 bytes wrapped in a syntactically valid transaction*, which is a
few lines of code.

**Why the plan permits it.** §2.2 states the gate as a closure — *"each ends at **the
bytes are a real transaction, and it is the one the record claims**"* — and never
states the residual. That is a load-bearing overstatement: §2 is the plan's entire
anti-smuggling argument, §6.3's requirement is the reason `tx:` gets a decode at all,
and a future reader (or the P3 porter) will treat DECODE as *the* gate. Compare how
the spec handles an equivalent limit and says so out loud: §3.5's *"The txid is shown
for RECOGNITION and MUST NOT be claimed as proof"*, and §2.2a's `ACCEPTED COST`. The
plan needs the same voice and does not have it; §6's closure list bounds nothing here.

**Confidence:** High. (The mechanism is not in dispute; what is arguable is whether
an unstated residual is Important. I file it Important because the plan asserts the
closure rather than the bound.)

## [I4] §6 narrowed the refusal-coverage condition from "every refusal" to "E1–E10", dropping four of this phase's own refusals

**Severity:** Important. **Where:** plan §6, against v1 §6 and spec §5.

**Concretely.** v1's closure list said:

> *"Every refusal added has a test that goes RED without its check — the
> `mutate-refusals.sh` discipline, applied to `me`'s new guards."*

v2 replaced that with:

> *"Every rule E1–E10 has a test that goes RED without its check."*

E1–E10 are the **record-codec** rules. The refusals P1 also adds are **outside** that
set: **R2** (a `tx:` record on argv, step 9), **R7** (empty stdin, step 2), the
**TTY refusal** §4.2 makes NORMATIVE, and **§2.3's decode-failure refusal** — which is
the anti-smuggling gate this whole plan is built around. None of them is covered by
the surviving bullet, and the `mutate-refusals.sh` reference that bound them is gone.

Spec §5 opens with *"Every refusal gets a test, and every refusal test must go RED
when its check is removed"*, and spec §7 restates it as a close condition
(*"Refusal coverage is a bijection"*). The machinery exists in this repo —
`scripts/check-refusal-coverage.sh` and `scripts/mutate-refusals.sh` are both present.
The rewrite dropped the only line that pointed at them.

**Confidence:** High. v1 line read from the diff; both scripts confirmed on disk.

## [I5] Step 3's test lost the content rule, so nothing asserts that a transaction-only payload packs UNSEALED

**Severity:** Important. **Where:** plan §4 step 3 and §4.3, against spec §2.4 and §8.

**Concretely.** v1's step 3 test was:

> *"a payload with no `IsSecret()` record packs **unsealed**; one with any packs
> **sealed**; stderr says which and why, every time"*

v2's is:

> *"explicit passphrase flags **win**; content decides **only** when none is given;
> **stderr says which way and why, every time**"*

The **base rule is gone from the test column**. What remains asserts *precedence* —
what happens when a flag is present — and never asserts the outcome in the case §2.4
actually rules: **no flags, no secret record → unsealed.** A `me` in which
`Class::Transaction` is secret, or in which "content decides" defaults to sealing,
passes step 3 as written.

That is not hypothetical plumbing: `Class::is_secret` is a `matches!` over three
variants (`sysw/record.rs:50-55`), so the new variant's secrecy is decided by whoever
adds it, in a file whose doc comment argues *both* ways about a borderline class
(*"`Class::FreeText` is deliberately NOT secret even though an operator may put
anything in it"*). Get it wrong and every transaction payload seals — costing the
operator a 12-word passphrase to store, those 12 words typed on the device's
on-screen keyboard, and ~31 s of KDF, to protect a payload whose purpose is to become
a plate anyone can read. That is the exact cost §2.4 exists to remove, it is an
operator ruling in §8, and after the rewrite no test in the plan looks at it.

**Confidence:** High.

## [I6] §6's "the build gate has run on it" is a structural false PASS — I ran it

**Severity:** Important. **Where:** plan §6, bullet 1.

**Concretely.** Executed:

```
$ ./scripts/plan-build-gate.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
...
test result: ok. 77 passed; 0 failed
   PASS: the CLI tests compile
   clippy clean
EXIT=0
```

**It extracted nothing from this plan.** The extractor accepts a ```rust block only
when a preceding anchor line names `src/seal/*.rs` or `tests/seal_cli.rs`
(`plan-build-gate.sh:70-74`: `if p.startswith("src/seal/") or p == "tests/seal_cli.rs"`).
This plan contains neither anchor; its one ```rust block is the abridged
`content_id_from_txid_display` signature in §1.1. So the script copied the pristine
crate, applied zero edits, and built, tested and clippy-linted **`me` as it already
is** — reporting GREEN for an artifact it never read a line of.

Worse, the honesty clause misfires the same way: the `== RESULT ==` block names
*"src/main.rs and src/validate.rs arrive as fragments"* — files belonging to the
**encrypted-payload** plan the script was written for. So the gate's own statement of
what it does not cover is about a different document.

This repo's rule is that *"a gate that hides its own blind spot is worse than no
gate"*, and that *"a gate that has never executed is a hypothesis"*. This one is
worse than either: it executes, it is fully green, and it is evidence of nothing.
§6's first bullet is satisfiable today without a single claim in the plan being
machine-checked.

**Confidence:** High. Executed; extractor source read.

## [I7] V4's form is unstated, and no positive vector requires a segwit transaction on the RAW path

**Severity:** Important. **Where:** plan §3's V1, V2, V4 and §4 step 5.

**Concretely.** V4 is *"a SEGWIT transaction, with txid AND wtxid AND `chunk_set_id`
all written out"*. `chunk_set_id` exists only for the **chunks** form, so V4 reads as
a CHUNKS vector — but the row never says, and the two forms have entirely different
bodies. That alone blocks construction: an author cannot build V4 without choosing.

If V4 is CHUNKS, the RAW path has no segwit coverage. V1 (*"RAW, no optional
fields"*) and V2 (*"RAW, all three optional fields"*) place no constraint on the
transaction, and the plan warns about the txid==wtxid trap **only for V4**. So V1 and
V2 get built from a convenient small legacy transaction — the natural choice for a
hand-constructed vector — and an implementation that computes `dSHA256` over the
with-witness RAW body passes both. It then **refuses every honest segwit RAW
record** at §2.2's equality check, and V8 (*"RAW whose carried txid ≠ the body's"* →
assert refusal) passes in that world too, because it also refuses.

This is C2 surviving in the half of the format V4 does not cover, by the same
mechanism the plan already identified: *"for a legacy transaction txid == wtxid and
the vector passes in both worlds."*

**Confidence:** High.

## [I8] The "opaque" sentence §6 requires P1 to correct is not in the spec

**Severity:** Important. **Where:** plan §2 (line 124) and §6's closure list (line 315).

**Concretely.** §2 opens: *"§1 of the spec says **"the record body is opaque to
`me`."** **False**"*, and §6 makes it a closure condition: *"**Spec corrections this
phase owns:** §1's 'opaque' sentence, and §2.3's '16,367-byte raw transaction'."*

`grep -ni opaque design/SPEC_engrave_transaction.md` → **no matches.** The word is
not in the governing spec at all. `git log -S"opaque"` shows it entered at
`f935316` as a §1 table cell — *"`ClassTransaction`. The record body is **opaque** to
it, exactly as `text:` bodies already are"* — and was **removed by `2dce797`, the
journey-walk fold**, before the spec reached R0 GREEN.

So the plan opens its central section by refuting a sentence the R0-GREEN spec does
not contain, and then makes correcting it a **condition of closing the phase**. The
condition cannot be satisfied: there is nothing to edit. Whoever burns it will either
tick it falsely or edit `JOURNEY_WALK_engrave_transaction.md:574`, which is a
different document and a different claim (*"`me` treats the record body as opaque by
design"*, said about where `--chunks` belongs). §2's *argument* still stands and is
still needed; its citation and its closure item do not.

**Confidence:** High. Grepped; history resolved to two commits.

## [I9] §4.3 makes `--allow-weak` — a documented no-op — a sealing determinant

**Severity:** Important. **Where:** plan §4.3.

**Concretely.** §4.3 lists the flags as *"`--passphrase-words`, `--passphrase-ask`,
`--no-passphrase` and `--allow-weak`"* and then rules *"**an explicit flag always
wins**; content decides **only** in their absence."* But `--allow-weak` is not a
passphrase mode. `crates/me-cli/src/main.rs:190-192`:

```rust
/// Accepted and ignored. `me` warns rather than refusing (spec §13 D3);
/// kept so existing invocations keep working.
#[arg(long)]
allow_weak: bool,
```

and `main.rs:893-895` prints *"me: --allow-weak is accepted and ignored"*. So
`me sysw pack --allow-weak < tx.txt` presents a literal reading in which an explicit
flag was given and therefore wins — sealing a transaction-only payload, which is
precisely what §2.4 rules against and what §8 records as an operator decision. Under
the other reading it is ignored and content decides.

§4.3 exists because *"v1 said 'seal by content' and never said what happens when the
operator also passes a flag — so step 3's test could not be written."* For
`--allow-weak` it still cannot. (The three flags that *do* select a mode are already
mutually exclusive in clap — `conflicts_with_all` / `conflicts_with` at
`main.rs:177-189` — so the precedence rule's real work is exactly this fourth case.)

**Confidence:** High for the mechanism; Medium that the intended reading is not
obvious enough to leave alone — but it is the one case §4.3 was added to settle.

## [I10] The `TO` label has no UTF-8 verdict and no length bound, and Rust and Go disagree by default

**Severity:** Important. **Where:** plan §1.2, tag `0x01`.

**Concretely.** §1.2 gives tag `0x01` as *"UTF-8, operator's own words"*, with a
`u16` length and no rule anywhere in E1–E10. Feed both implementations a TLV whose
value is `0x74 0x6f 0xff 0x21` (`to\xffe!` — invalid UTF-8):

- Rust: `String::from_utf8` returns `Err` → the record is refused. (`me` already has
  this exact posture: `sysw/record.rs:93` maps it to `RecordError::NotUtf8`.)
- Go: `string(b)` never fails → the record is accepted and the label is rendered with
  a replacement character, **onto steel**.

One implementation refuses what the other engraves. There is also no maximum: a
`u16` len makes a 65,535-byte label expressible, caught only incidentally by the
section cap, and §3.4 puts this field in the plate's asserted column.

Round 0 filed this as **M5** and v2 did not address it. I raise it because v2 changed
what the omission means: §1.3 now **enumerates** the divergence class and §6 makes
"a test that goes RED without its check" a closure condition for E1–E10 — so a
divergence outside E1–E10 is now a hole in a completeness claim rather than an
unstated detail. The same gap covers I3's other half (**[I11]**).

**Confidence:** High for the divergence; Medium on the severity re-rating.

## [I11] A TLV whose `len` disagrees with its tag's fixed width has no verdict (round 0's I3, other half)

**Severity:** Important. **Where:** plan §1.2 and §1.3.

**Concretely.** §1.2 says tag `0x02` is *"u64 satoshis, big-endian, **exactly 8
bytes**"* and `0x03` is *"**exactly 4 bytes**"*, but the layout gives every TLV its
own `u16` `len`, and **no rule in E1–E10 binds the two**. A record with
`tag=0x02, len=2, value=0x03E8`:

- implementer A refuses (len ≠ 8);
- implementer B reads two bytes as a big-endian u64 → **1,000 sat**;
- implementer C left-pads to 8 → **1,000 sat**, by a different route;
- implementer D right-pads → **281,474,976,710,656,000 sat**.

The fee is engraved on the plate in §3.4's asserted column. E6 refuses only `len = 0`,
so the near-miss `len = 1..7` is exactly the gap. §1.2's "exactly N bytes" is a
sentence in a **tag table**, in the encoder's voice — the same shape round 0's I3
already identified as *"a description of the encoder, not a refusal binding the
decoder"* — and §6's closure sweep covers E1–E10 only, so it never reaches it.

**Confidence:** High.

---

## [M1] V15's justification is false — it passes under both byte orders
V15's cell claims *"this vector alone would have caught C1 on the first run."* It
would not. V15 constructs a **deliberate mismatch** and asserts REFUSE. Under the
wrong (internal) byte order the deliberate mismatch is still a mismatch, so the
refusal still fires and the vector is green. What catches C1 is **V4**, a positive
vector pinning the field's bytes. (Round 0's I13 carried the same claim; the vector
is still required by spec §5, only its stated reason is wrong.)

## [M2] Round 0's M2 was not folded — §1 still cites Go for a rule P1 implements in Rust
§1 line 31: *"(the reserved-prefix rule, `sysw/record.go:41-51`)"*. The site P1 edits
is `crates/me-cli/src/sysw/record.rs:24-29` (the doc comment plus `TEXT_PREFIX` /
`PASS_PREFIX`), where the new `TX_PREFIX` must go.

## [M3] The near-miss count went from six to seven with no seventh named
§6: *"Seven instances this cycle."* Spec §5: *"Six instances in this cycle now, the
most recent two found while fixing F-244."* v1 said six. The increment is unexplained
and the seventh instance is not identified.

## [M4] §3.1 states a floor as an equality and leaves the odd hex character unaccounted
*"32,731 hex chars = 16,365 bytes"*. 16,365 bytes is **32,730** characters; one
character is spare. The final ceiling (16,322) is correct — recomputed — but the
derivation reads as exact when it is a floor, and nothing in the plan says what
happens to a `tx:` record with an odd number of hex characters (R1 covers "not
lowercase hex"; an odd-length run of hex characters is a different failure).

## [M5] §1.1's ```rust fence is a paraphrase presented as source
It is attributed to `pipeline.rs:17-27` but is not byte-exact: the real doc comment
has a blank `///` line after the first and ends *"…rather than raw bytes, **and takes
it as the caller already has it**."* The plan's version compresses both away. The
ruling is right; the quotation is not verbatim, in a document whose whole argument
turns on quoting that function accurately.

## [M6] Round 0's M4 stands — three record sources, no precedence rule
Step 2 adds stdin beside `--in` (which wins over argv today, `main.rs:1215-1226`, and
filters blank lines). Nothing says what `me sysw pack rec1 --in f.txt < g.txt` does,
or whether stdin filters blanks the way `--in` does.

## [M7] The 16,322 ceiling assumes the `tx:` record is ALONE in the section
Records are joined with `\n` and no trailing LF (`sysw/mod.rs:260`; asserted by
`joins_with_lf_and_no_trailing_lf`). Spec §3.6 contemplates several transactions in
one payload, and a payload may also carry `md1`/`mk1` records — each costs its own
length plus one separator. §3.1's table row *"record framing | 16,322 B"* and the
§2.3 correction §6 owns both carry the single-record assumption silently.
(`MAX_RECORD_LEN = 512` and `MAX_RECORDS = 24` are `seal`'s and do **not** apply to
`sysw`, which I checked — `sysw::split` applies neither.)

## [M8] Round 0's M1 stands — publishing `mt-codec` bakes in an unused `bitcoin = "0.32"`
§5 publishes 0.1.0 and does not mention it. `crates/mt-codec/Cargo.toml` still
declares `bitcoin = "0.32"` with 0 uses in `src/`, and §5 correctly notes a published
version *"can be yanked, never replaced"* — so it lands in the immutable artifact and
in every downstream build of `me`.

---

## Verdict

**3 Critical / 11 Important / 8 Minor. NOT GREEN. No code.**

Part 1: **13 FIXED / 4 PARTIAL / 1 NOT FIXED** of round 0's 18.

The three that bind hardest, and all three are products of the rewrite rather than
survivals from v1:

- **C1** — §1's layout table still says `INTERNAL`. Round 0's C1 is not folded; it is
  half-folded, which is the same document with the same contradiction in it.
- **C2** — the rewrite **deleted** v1's body definition while adding the TLV rules,
  so the format now says nothing about whether the RAW body carries the witness.
  Both answers pass every check in the plan and one of them silently discards every
  signature.
- **C3** — no step and no closure condition creates `ClassTransaction`, so §6's gate
  is green while `me sysw pack` still returns `Unclassifiable` for every `tx:` record.

The pattern across Part 2 is worth naming: v2 is markedly stronger on **rules** and
markedly weaker on **gates**. Six of the eleven Importants are closure conditions that
cannot be satisfied, were narrowed, or pass on nothing — I1 (E1's condition
unsatisfiable), I4 (refusal coverage narrowed), I5 (step 3 lost its assertion), I6
(the build gate reports GREEN having read nothing), I7 (V4's form unstated), I8
(a spec correction with no referent).

### What I did NOT examine

- P2–P6, the device, the plate, the fork's Go side, `design/agent-reports/`, style
  and wording.
- Whether `bitcoin 0.32`'s API supports the exact witness-stripped re-serialisation
  §1.1 requires — I established the rule is stated and unversioned, not that any
  particular crate satisfies it.
- Whether `mt-codec` actually publishes cleanly (`cargo publish --dry-run` not run);
  I confirmed its manifest is workspace-inherited and has no path dependencies, which
  is necessary, not sufficient.
- `me`'s existing CLI test suite, and whether step 2's or step 3's changes break it.
- The BCH/bech32 correctness of `mt-codec`, and `mt1_v1.json` as a corpus — treated
  as ground truth, per round 0.
- The Minors round 0 raised that fall outside the 18 were checked opportunistically,
  not systematically.
