# R8 — `IMPLEMENTATION_PLAN_mt_v0_1.md`: can the gates run and fail, and is §1's fork ruling sound?

**Lens:** one adversarial question, two halves. Half A — every gate in P0–P6: does the
command exist, can it pass vacuously, can it fail, does it gate what the phase claims.
Half B — the §1 fork-not-parameterise ruling and whether its two-test defense holds.

**Date:** 2026-08-23. **Artifact:** `design/IMPLEMENTATION_PLAN_mt_v0_1.md` (read-only).

**Verdict.** The gates are **not** sound. Six Critical, eleven Important. The single
most important result is machine-proven and inverts §1's central claim: the
cross-format negative — the test the plan calls *"the only one of the two that fails
when a copied module keeps a copied constant"* — **produces the identical result
whether the constant was copied or not**, because the HRP is mixed into the polymod
input and already separates the formats. §12.22's hazard is therefore defended by
**neither** of the two tests, and the hazard the HRP does *not* close (a checksum
computed over `hrp_expand("md")`) has no test at all.

The **fork decision itself is right and should stand.** What is wrong is its defense,
its arithmetic at three copies, and one stale premise.

---

## What was measured (commands run, not reasoned)

| # | check | result |
| --- | --- | --- |
| 1 | `cargo nextest run --locked` on a two-crate workspace with zero tests | **exit 4**, `error: no tests to run` (cargo-nextest 0.9.140) |
| 2 | same, before `Cargo.lock` exists | errors out: *"cannot create the lock file … because --locked was passed"* |
| 3 | Python reimplementation of `md-codec`'s `polymod_run` / `hrp_expand` / `GEN_REGULAR` / `POLYMOD_INIT`, five copy-paste scenarios | table below |
| 4 | does §12.13 exist? | **No.** §12 holds items 4,5,6,7,8,11,12,15,16,18,19,21,22,23 |
| 5 | is the header layout `(a2)` in §10 or §12? | line 3088, and §12 starts at 3290 → **§10.13(a2)**; the spec itself cites `§10.13 a2` (line 657) and `§10.13(c)` (lines 428, 514) |
| 6 | is "Journey A" defined anywhere in the mt docs? | **absent** from `SPEC_mt_v0_1.md`, `SPEC_mt_qr_DEFERRED.md`, `CONTINUITY_mt_2026-08-22.md`, the plan; B and C appear only as *sources* of rulings, never defined; `design/journeys/` has no mt material |
| 7 | how often does the plan mention `verify`? | **twice** — the verb list (line 27) and the P3 heading (line 177). Zero deliverables, zero tests, zero gates |
| 8 | is the Q-9 extraction trigger live? | **retired 2026-05-03** — `mnemonic-key/design/FOLLOWUPS.md:203-207` |
| 9 | does `mk-codec` mix the HRP into its checksum? | yes — `crates/mk-codec/src/string_layer/bch.rs:315,335`; its own doc comment (line 201) says separation comes from the target constants **"+ the HRP"** |
| 10 | §8's actual numbering | `1, 2, 2b, 2c, 2d, 2e, 3, 4, 5, 6, 7 (MOVED), 7b, 7c (MOVED), 8, 9` — §8.2 is a scope ruling and §8.8 says *"not a refusal"* |

Scenario table for check 3 (`data` = 30 random 5-bit symbols, seed 7):

| scenario | build | `verify` as `mt1` | `verify` as `md1` | `verify` as `mk1` |
| --- | --- | --- | --- | --- |
| 1 | **copy-paste**: hrp `"mt"`, constant left = `MD` | True | **False** | — |
| 2 | **correct**: hrp `"mt"`, constant = `MT` | True | **False** | — |
| 3 | genuine `md1` chunk, checked as `mt1` | — | — | False under both a copy-paste and a correct build |
| 4 | **copy-paste from mk**: hrp `"mt"`, constant left = `MK` | True | **False** | False |
| 5 | hrp left `"md"` in the checksum, constant correctly `MT`, rendered `mt1…` | **False** | **False** | — |

Rows 1 and 2 are the finding: **the cross-format negative returns `False` either way.**

---

# CRITICAL

## C1 — P0's gate cannot pass. Measured: it exits 4.

**The gate.** P0: *"`cargo build` and an empty `cargo nextest run --locked` succeed."*

**The scenario.** There is no broken state, because there is no passing state. On a
freshly created two-crate workspace with no tests, `cargo nextest run --locked` prints

```
     Summary [   0.000s] 0 tests run: 0 passed, 0 skipped
error: no tests to run
(hint: use `--no-tests` to customize)
```

and **exits 4**. cargo-nextest 0.9.140 defaults `--no-tests` to `fail`. P0 therefore
can never close, and since *"a phase does not begin until the previous one is green"*,
no phase can start. This is the plan's first gate and it is a hypothesis in the exact
sense §2 warns against — one `cargo nextest run` in an empty directory would have
found it.

**Smallest fix.** Do **not** reach for `--no-tests=pass`: that re-creates the vacuity
the brief asks about, and it stays permanently disarmed once real tests exist. Give P0
**one real test** — the drift test from P1 is the natural candidate, it needs no `mt1`
machinery beyond the constant — and keep the default `fail`. The gate then reads
`cargo build --locked && cargo nextest run --locked`, runs 1 test, can fail (edit the
constant), and the "no tests discovered" condition remains an error forever.

## C2 — The cross-format negative cannot fail on the mistake it exists to catch.

**The claim under attack** (§1, "How the copy-paste hazard is defended", item 2):

> *"A cross-format negative, which NEITHER sibling has. An `mt1` chunk must **fail**
> `md1` verification, and an `md1` chunk must fail `mt1` verification. … it is the only
> one of the two that fails when a copied module keeps a copied constant."*

**The scenario, machine-proven.** The checksum is computed over
`hrp_expand(hrp) ‖ data`, and `hrp_expand("mt") = [3,3,0,13,20]` while
`hrp_expand("md") = [3,3,0,13,4]`. The HRP is therefore *inside* the polymod, and it
separates the two formats **on its own, regardless of the constant**. Build an
`mt-codec` that made the worst §12.22 mistake — HRP correctly changed to `"mt"`,
`MD_REGULAR_CONST` left in place — and:

- an `mt1` chunk fails `md1` verification → **False** (scenario 1)
- a correct `mt1` chunk fails `md1` verification → **False** (scenario 2)
- a genuine `md1` chunk fails `mt1` verification → **False** in both builds (scenario 3)

Both directions of the test return the same value for the broken build and the correct
build. **The test has no discriminating power over the constant at all.** Combined
with the drift test's acknowledged hole (§1 item 1: *"a constant copied wholesale …
with the domain string pasted too would satisfy it"*), the §12.22 hazard is defended
by **neither** test. The plan's own bolded sentence — *"Both, not either"* — is
answered by the arithmetic as *"neither"*.

There is a sensitive form: hold the HRP fixed at `"mt"` and vary only the constant.
Scenario 1's third row measures exactly that and returns **True** — i.e. the copied
constant *is* detectable, but only by a test the plan does not describe.

**Smallest fix.** Replace the cross-format negative with two dependency-free
assertions in `mt-codec`'s `consts.rs`:

1. `assert_ne!(MT_REGULAR_CONST, MD_REGULAR_CONST)` and `assert_ne!(MT_REGULAR_CONST, MK_REGULAR_CONST)` — the constants inlined as literals, so no dependency on either sibling;
2. `assert_ne!(NUMS_DOMAIN, b"shibbolethnums")` and `assert_ne!(NUMS_DOMAIN, b"shibbolethnumskey")` — which closes the drift test's stated hole in two lines.

`mk-codec` already ships (2) against `md` as
`consts::tests::nums_string_differs_from_md1`. Keep a cross-format negative if you
like, but **relabel what it proves**: it demonstrates HRP domain separation, not
constant separation, and the plan must stop calling it the defense against §12.22.

## C3 — The defense is aimed at the wrong neighbour: P1 ports from `mk`, §1 defends against `md`.

**The claim.** §1 defends only the `md1` direction. P1 then says: *"**Ported from
`mk-codec`, which is the closest sibling** — not from `md-codec`."*

**The scenario.** The copy-paste mistake that actually gets made is the one against the
crate you have open. Copy `mk-codec/src/string_layer/`, change the HRP to `"mt"`, leave
`MK_REGULAR_CONST = 0x1062435f91072fa5c` and `NUMS_DOMAIN = b"shibbolethnumskey"`:

- **drift test**: passes — the pasted constant reproduces from the pasted domain string;
- **cross-format negative vs `md1`**: passes in both directions — `md` was never involved (scenario 4, measured);
- result: `mt1` carries `mk1`'s target residue, and **the plan's entire defense is green**.

At two formats there was one pair to defend. At three there are three pairs
(`md↔mk`, `md↔mt`, `mk↔mt`), and the plan covers **one of three** — and not the one
the code is copied from. This is the answer to the brief's question about whether three
copies change the arithmetic: they do, quadratically in pairs and linearly in the
defense that has to be written, and the plan carries the two-format defense unchanged.

**Smallest fix.** C2's fix, stated three-way. Same four lines.

## C4 — `mt verify` is implemented and gated by nothing.

**The gate.** None exists. Measured: the plan contains the word `verify` twice — the
four-verb list (line 27) and the P3 heading `### P3 — decode and verify` (line 177).
Every bullet under P3 is a `decode`-path item (splitting/stripping, modal length,
autocorrect, correction reporting, duplicate resolution, `decode` writes nothing to
stdout). P3's gate is *"Every P1 vector round-trips through `decode`"* plus a `decode`
negative. `verify` appears in no deliverable, no test and no gate.

**The scenario.** `mt verify` ships printing `OK`. Every gate in P0–P6 is green.
What the spec requires and the plan drops:

> **`verify` REPORTS ITS MARGIN, not just its verdict.** Usability journey walk, U-2 —
> the one Critical it found, and five correctness rounds had missed it …
> `chunk 7   4 of 4 symbols   pos 13, 29, 30, 78   <-- NO MARGIN LEFT` …
> *"Chunk 7 is at its correction limit. One more damaged symbol in that string and
> this transaction is unrecoverable. Re-cut it."*

— plus **`verify` LOCALISES every correction** (the `[q>p]` inline rendering, operator
ruling from Journey C), and `verify` is one of the three string-having callers that
render the report (§1.1's row table, line 657).

This is the finding that `/scratch/code/CLAUDE.md` holds up as the constellation's
exemplar for why journey walks exist. The plan drops it silently — which is the same
failure shape: nothing is *wrong* in the plan, a verb is simply *absent*.

**Smallest fix.** Give P3 (or a P3b) a `verify` deliverable with three bullets — margin
report, correction localisation, report rendering — and a gate: *"a vector corrupted
with exactly 4 symbol errors in one chunk verifies OK **and** the output contains
`4 of 4` and `NO MARGIN LEFT` for that chunk; the same vector with 1 error does not."*

## C5 — Nothing gates that BCH correction ever corrects.

**The gate.** P1: nextest green. P3 positive: *"Every P1 vector round-trips through
`decode`"* — clean vectors, so the decoder's residue is zero and the correction path is
**never entered** (`md-codec/src/chunk.rs:568-575` shows the `residue == 0` pass-through
that P1 will port). P3 negative: *"a deliberately corrupted vector must **fail**"*.
P3's four listed tests are: a >4-error chunk is not silently accepted; the `mt1`→`mtl`
hazard does not fire on a valid string; three-candidate duplicate refusal; stdout empty
on failure.

**The scenario.** Implement `decode_regular_errors` as `fn … -> Option<…> { None }`.
Every clean vector still round-trips (residue zero, pass-through). Every >4-error test
still refuses. Every duplicate test is unaffected. The drift test and the cross-format
negative are unaffected. **P0 through P6 are green with zero error-correction
capability** — in a format whose entire ECC rationale is §3a and whose 2040-recoverer
journey exists to exercise it. Every listed test asserts that decoding **fails**; not
one asserts that a repairable plate is **repaired**.

The same hole swallows a subtler fork defect: `md-codec` passes the target constant
into `decode_regular_errors(residue, …)` as a **parameter** while baking it into
`bch_create_checksum_regular` / `bch_verify_regular` — the asymmetry §1 itself
documents. A fork that fixes the two baked call sites and leaves the parameterised call
site passing `MD_REGULAR_CONST` verifies correctly, round-trips correctly, and can
never repair a plate. No gate distinguishes it from correct code.

**Smallest fix.** One test in P1 and one gate line in P3: for each of e = 1,2,3,4,
corrupt e symbols in one chunk of a vector and assert `decode` **succeeds** and returns
the byte-exact original; at e = 5 assert refusal. This also constitutes C4's gate.

## C6 — The test vectors are generated by the implementation under test, so nothing pins `mt1`'s bytes to the spec.

**The claim.** P1: *"**Test vectors are produced here and are the artifact P2–P4 are
checked against.** Per the spec's own lesson that a corpus can be uniformly wrong…"* —
and then the vectors are produced *by `mt-codec`*. Every downstream gate (P2's *"`mt
encode` … reproduces the vector's strings exactly"*, P3's round-trip, P4's report
comparison) is therefore a **self-consistency check against the implementation's own
output**. The plan cites the lesson and then performs the thing the lesson names.

Ordinarily the drift test and the cross-format negative would be the external anchors.
C2 shows the second is insensitive and §1 concedes the first has a hole. So there is
**no anchor at all** between `mt-codec`'s output and the spec's wire format.

**The scenario, machine-proven** (scenario 5): fork the module, change the HRP at the
*rendering* layer to `"mt"`, and leave `hrp_expand("md")` in the two checksum
functions. Result: strings render as `mt1…`, the crate is fully self-consistent
(encode→verify→decode round-trips), the drift test passes (the constant is correct),
and **both directions of the cross-format negative pass**. Every gate is green, and
every plate is unreadable by any conformant `mt1` implementation.

This is precisely §10.13(b), which R5 promoted from MINOR after finding it makes plates
*"MUTUALLY UNVERIFIABLE"* and fails them *"with a 'damaged beyond correction'
diagnostic that points the recoverer at their steel rather than at their software."*
The spec found it and raised its severity; the plan gates it with nothing.

Two more instances of the same hole, both listed separately below because their fixes
differ: the **version nibble** (I1) and **balanced-vs-filled chunking** (I2).

**Smallest fix.** One externally-computed anchor. Pin **one full `mt1` chunk string**
as a literal in the vector file, computed independently of `mt-codec` (a 30-line script
against the published BCH parameters is enough — this review wrote one), and assert
byte equality. Additionally assert the checksum input begins with `hrp_expand("mt") =
[3,3,0,13,20]`, which is two lines and closes scenario 5 directly.

---

# IMPORTANT

## I1 — The plan's header restatement drops the two values the spec flags as guess-hazards.

**The claim.** P1: *"header 49 bits: `version(4) + chunked(1) + chunk_set_id(20) +
count−1(12) + index(12)`"*.

That is the widths only. §10.13(a2) — headed *"because R4 found five things an
implementer would otherwise guess, and **two of the guesses produce plates another
implementation cannot read**"* — pins the **values**:

- `version` = **`0b0001`**, *"Not inherited from `md1`; a shared value would let one format's chunk verify as the other's under a colliding constant"*;
- `chunked` = **`1`, always, and RETAINED** even though `mt1` is always chunked, because dropping the dead bit shifts every later field.

**The scenario.** The implementer ports `header.rs` from `mk-codec`, whose
`VERSION_V0_1: u8 = 0x00` (`crates/mk-codec/src/string_layer/header.rs:30`), and emits
`version = 0`. Self-consistent; vectors self-generated; no test pins the value. Green
through P6, and every plate carries the wrong version — which the spec says is itself
part of the cross-format defense.

**Smallest fix.** Add the two values to the restatement, and one assertion: the first
byte of a vector's decoded header is `0b0001_1…`.

## I2 — P1's chunking property test cannot distinguish balanced from filled — the exact divergence §3b was written to close.

**The claim.** P1 test 3: *"for random payload lengths, `count`, `bytes_per_chunk` and
the reassembled payload round-trip, and **no chunk exceeds 40 bytes**."*

**The scenario.** Implement **filled** chunking: 40-byte chunks, last one short. For a
535-byte payload that is `count = 14`, chunks of `40×13 + 15`. It round-trips, no chunk
exceeds 40 bytes, and `count` matches the rule. The property test is green. The spec's
normative rule gives **`count = 14`, `bytes_per_chunk = 39`, balanced** — and §3b says
in terms that the divergence means *"every plate then fails the other implementation's
§1.1e length check as damaged steel."* §3b's rule was filed as a **Critical** by R6's
implementability lens; the plan restates it and then writes a property test that cannot
see it.

**Smallest fix.** Add to the property: all chunks but the last are exactly
`bytes_per_chunk = ceil(payload_len/count)`, the last is the remainder; plus the spec's
own pinned example, 535 B → `count = 14`, `bytes_per_chunk = 39`.

## I3 — `header.rs` is ported from `mk`, whose field widths are entirely different, with no header-layout test.

`mk-codec`'s chunked header is `version(5) + type(5) + chunk_set_id(20) +
total_chunks(5) + chunk_index(5)` = 40 bits, 8 symbols, and `MAX_CHUNKS = 32`
(`consts.rs:41`). `mt1`'s is 49 bits with **12-bit** count and index and a 4,096-chunk
ceiling. Only `chunk_set_id(20)` transfers.

**The scenario.** The port keeps mk's 5-bit index/count fields. Nothing in P1's three
tests reads the header. The chunking property test trips only if it happens to draw a
payload over 1,280 bytes (> 32 chunks) — and the plan specifies *"random payload
lengths"* with no floor. A 2 kB transaction then produces silently wrong headers, and
§8.7b's 4,096-chunk refusal (P5) is unreachable because the field cannot express it.

**Smallest fix.** A header round-trip test at the boundaries: `count = 4096`,
`index = 4095`, `chunk_set_id = 0xFFFFF`; and give the property test an explicit
payload range that guarantees > 32 chunks.

## I4 — P4's gate fails on conformant code, and omits the fourth caller.

**The gate.** P4: *"The report renders identically from `encode`, `decode` and
`inspect` for one vector, differing only in stream and the `CUT`/`PREFIX` suffix."*

**The scenario.** §1.1's row table (line 657) says `mt1 SET` is present *"when the
caller had strings — `inspect`, `decode`, `verify`"*, which **excludes `encode`**; and
`FEE` is present *"when a node is reachable, **or** the input was a PSBT carrying
values"*, which differs between an `encode` from a PSBT and an offline `decode` from
strings. A conformant implementation therefore produces reports differing by **a whole
row**, and the gate goes red on correct code. The implementer's escapes are to make
`encode` emit `mt1 SET` (contradicting the row table) or to loosen the comparison until
it asserts nothing — and the plan's other gates are too weak to notice either.

The plan inherits this from the spec, whose closing box (*"the only differences are the
stream and the suffix … No caller reorders, reformats, or drops a row"*) contradicts
its own row table. That ambiguity is survivable in a spec and fatal in a byte-equality
gate.

Separately, the gate compares **three** callers and the spec has **four** verbs
rendering the report — `verify` is in the row table's string-having list and in the
gate's blind spot (see C4). The plan's *"its three callers"* reads §1.1's *"three
callers — a pre-engraving operator, a recoverer with a node, a recoverer without"*,
which enumerates **audiences**, not verbs.

**Smallest fix.** State the gate against the row table: *"`decode`, `verify` and
`inspect` render byte-identical reports; `encode`'s is that report minus the `mt1 SET`
row plus the `CUT`/`PREFIX` suffix"* — and name which of the two contradictory spec
sentences governs.

## I5 — P4's gate passes vacuously offline.

**The scenario.** Run the gate with no node fixture wired. All three callers render
every chain-derived row as `UNKNOWN` — which §1.1's rule 1 explicitly requires
(*"A row is never omitted for being unanswerable — it reads `UNKNOWN`"*). Three
identical all-`UNKNOWN` reports. **Gate green with the entire liveness subsystem
unimplemented.** Nothing in the gate exercises P4's actual deliverables:
`SPENT — ALREADY CONFIRMED` checked first, `DEAD` requiring `confirmations ≥ 1`, the
mempool-unconfirmed parent reading `PENDING` not `DEAD`, or `FEE` carrying the weakest
provenance inline.

The fixtures are in the "tests first" list but not in the gate, and P4's gate is the
only command that closes the phase.

**Smallest fix.** Add to the gate: *"the comparison vector's report contains at least
one non-`UNKNOWN` value in every row, and the liveness fixture suite covers all five
states including the mempool parent."*

## I6 — P5's mutation discipline is prose, not a gate, and has no executable form.

**The claim.** P5, tests-first: *"each must be shown to FAIL when the refusal is
removed."* P5's **gate** is: *"Every numbered refusal in §8 has a test, and a script
asserts the union is exhaustive."*

**The scenario.** Write a refusal test that asserts `exit_code != 0` and passes because
the fixture is malformed for an unrelated reason. It satisfies the gate — existence and
exhaustiveness — and passes with the refusal deleted. The mutation requirement is
stated in a paragraph the gate does not read, names no mechanism (`cargo mutants`? a
patch series? a recorded evidence file?), and produces no artifact anyone can check
later. This repo's own note is *"prove the mutated line RAN, not just that it landed"*
— neither half is specified.

**Smallest fix.** Move it into the gate and give it a mechanism: a committed script
that, for each refusal N, applies a recorded one-line mutation, runs only that
refusal's test, and asserts it fails — output committed as the phase's evidence.

## I7 — P5's exhaustiveness script has no well-defined input.

**The scenario.** §8's numbering, enumerated from the spec: `1, 2, 2b, 2c, 2d, 2e, 3,
4, 5, 6, 7, 7b, 7c, 8, 9`. Of these, **§8.2 is a scope ruling** (*"Script validity is
NOT checked in v0.1"*), **§8.8 is explicitly not a refusal** (*"Module size is the
operator's choice … — not a refusal"*), and §8.7/§8.7c are MOVED. The plan's
not-implemented list names **only §8.7 and §8.7c**.

A script enumerating the top-level numbering therefore demands refusal tests for §8.2
and §8.8 — which do not exist and cannot — so the gate **cannot pass**; and a script
enumerating only top-level items **misses §8.2b, §8.2c, §8.2d and §8.2e**, which are
where the real value/provenance refusals and the sniffing procedure live. The moment
the exemption list becomes hand-maintained, the gate's stated purpose (*"a refusal
cannot be added to the spec and silently go untested"*) is gone.

**Smallest fix.** Make the spec side machine-readable: require every refusal in §8 to
carry a marker (e.g. `→ refuse`, which most already do), have the script enumerate
**those**, and record §8.2/§8.7/§8.7c/§8.8 in a committed exemption file with a
one-line reason each — so adding a refusal without a test is red, and exempting one is
a reviewable diff.

## I8 — §12.13 does not exist. The plan's two most load-bearing citations dangle.

**Measured.** §12 contains items 4, 5, 6, 7, 8, 11, 12, 15, 16, 18, 19, 21, 22, 23.
There is **no §12.13**. The header layout `(a2)` is at line 3088, and §12 begins at
line 3290 — so it is **§10.13(a2)**. The spec cites it correctly as `§10.13 a2`
(line 657) and `§10.13(c)` (lines 428, 514).

The plan cites `(§12.13 a2)` for the header layout and `(§12.13 c)` for the content id
— the two most normative facts in P1. §12.22 (the NUMS constant) is correct.

`scripts/plan-cite-check.sh` returned 5/5 because it resolves **`file:line`**
citations, not spec `§` references — the gate-coverage corollary in this repo's
CLAUDE.md, exactly: *16 of 16 gated citations true, 5 of 22 ungated facts false.*

**Smallest fix.** `§12.13` → `§10.13`, twice. And extend `plan-cite-check.sh` to
resolve `§N.M` references against the spec's numbering, which is one grep.

## I9 — P1's entire normative content comes from an open question the plan says does not block it.

**The claim.** §4 item 2: *"§10 holds §10.10 …, §10.13, §10.14, §10.20. **None blocks
P0–P2.**"* (The set of four is correct — verified against §10.)

**The scenario.** §10.13 *is* P1: the NUMS constant, the header layout, and the content
id all live there. The plan lists it as an open question, then builds P1 entirely on it
and declares it non-blocking without argument. §10.13's own heading reads *"RULED,
ready to build"*, so the substance is fine — but the plan's accounting is
self-contradictory, and **I8's mis-citation is what hides it**: a reader following
`§12.13` never lands on the open-questions list and never sees the collision.

**Smallest fix.** With I8 fixed, add one sentence: *"§10.13 is RULED and is P1's
normative source; it remains in §10 only because the section was not re-numbered."*

## I10 — §1's shared-crate premise is stale by three months. The fork is permanent, not deferred.

**The claim.** §1's table: *"extraction into a shared crate | **deliberately deferred**,
trigger recorded as 'both md-codec and mk-codec at v1.0' (closure Q-9)"*, and the
conclusion *"defer extraction until the formats have stopped moving."*

**Measured.** `mnemonic-key/design/FOLLOWUPS.md:203`,
`mc-codex32-extraction-retired-2026-05-03`:

> *"md1 and mk1 use HRP-mixed BCH with per-format target residues that are NOT
> upstreamable … There is no longer shared code worth extracting — only a shared
> *pattern* … **md1↔mk1 BCH plumbing stays forked indefinitely**; the pattern will be
> documented in a future cross-repo `PATTERNS.md`."*

Q-9 was **retired**, not deferred. The plan's conclusion survives — forking is right,
and now more clearly so — but three of its supporting statements are false: extraction
is not pending, the v1.0 trigger no longer exists, and `mt` should **not** be *"built
to be absorbed by it later"* (§10.13's box, repeating the same stale claim). The
retirement note also corroborates C2 directly: separation is **HRP-mixed BCH + a
per-format residue**, and `mk-codec/src/string_layer/bch.rs:201` says the same —
*"target constants … **+ the HRP**"*.

The claim is repeated in `md-codec/src/bch_decode.rs:7-9` and
`mk-codec/design/SPEC_mk_v0_1.md:442` as well; the plan would make it four sites. This
is the "comments outlive their conditions" class.

**Smallest fix.** Replace the table row with *"retired 2026-05-03 (`mnemonic-key`
FOLLOWUPS `mc-codex32-extraction-retired-2026-05-03`) — the forks are permanent"*, and
add `PATTERNS.md` as a P0-owned follow-up, since `mt` is the third instance of the
pattern that note promised to document.

## I11 — Forking creates two obligations the plan does not name: a provenance pin, and a three-way defect check.

**The scenario.** A bug is found in `mk-codec`'s `bch_decode.rs` in six months. Who
knows `mt-codec` has a copy, and a copy of *which revision*?

Both siblings answer this in the file header, and mk's plan **required** it
(`mnemonic-key/design/IMPLEMENTATION_PLAN_mk_v0_1.md:772` — *"File header comment notes
the fork date"*):

- `md-codec/src/bch_decode.rs:3` — *"Forked from `mk-codec` v0.3.1 (`crates/mk-codec/src/string_layer/bch_decode.rs`) at v0.34.0"*
- `mk-codec/src/string_layer/bch.rs:4` — *"Forked from `md-codec` v0.4.x (`crates/md-codec/src/encoding.rs`)"*

The plan requires neither, so the fork lands unpinned. Meanwhile the constellation's
standing rule — a defect found in one implementation triggers a mandatory check of the
others — now spans **three** Rust implementations plus the Go port, and the plan
never says so. That is the honest answer to the brief's arithmetic question: forking is
still right, but the *maintenance* cost is superlinear and the plan books none of it.

**Smallest fix.** Two lines in P1's deliverable: every ported module carries
`//! Forked from mk-codec <version> (<path>) on <date>`, and a sentence in §3 recording
that a BCH defect found in any of md/mk/mt is checked in the other two.

## I12 — No gate runs the full validation surface, and the CI command is narrower than the constellation's.

**The scenario.** P0 runs the full suite while it is empty (and cannot pass, C1). P1
gates `-p mt-codec`. P2–P4 gate behaviours, P5 a script, P6 journeys. **No phase after
P0 runs the whole workspace suite**, so a P2 test broken by P4's work closes P4 green —
against the constellation rule that reviews run against the whole validation surface.

Separately, P0's deliverable is *"CI running `cargo nextest run --locked`"* while
claiming *"workspace lints matching the constellation"*. The sibling standard
(`descriptor-mnemonic/.github/workflows/ci.yml`) is five gates: `cargo test --workspace
--all-targets`, `cargo test --workspace --doc`, `cargo clippy --workspace --all-targets
-- -D warnings`, `cargo fmt --all --check`, `cargo doc`. **nextest does not run
doctests**, and nothing in the plan runs clippy or fmt — so the lints in the deliverable
are gated by nothing, and any `///` example in `mt-codec` (both siblings are dense with
them) is never compiled.

**Smallest fix.** P0's CI gets the five sibling jobs with `cargo test` replaced by
`cargo nextest run --locked` **plus** a separate `cargo test --workspace --doc`; and
every phase gate ends with the full `cargo nextest run --locked`.

## I13 — P6's journeys have no definition anywhere, and its cross-reference is a placeholder.

**The claim.** P6: *"The three walked journeys (**§ the spec's own**), as executable
acceptance runs."*

**Measured.** *"§ the spec's own"* is a literal placeholder naming no section. The spec
defines **no journeys**: "Journey A" appears nowhere in `SPEC_mt_v0_1.md`,
`SPEC_mt_qr_DEFERRED.md`, `CONTINUITY_mt_2026-08-22.md` or the plan; Journeys B and C
appear only as attributions (*"found by walking Journey C"*, six sites) for at least
five rulings. `design/journeys/` holds nothing for `mt`.

**The scenario.** The implementer reaches P6 and must invent the three journeys, then
invent what the operator should see at each step, then assert on it — **after** the
implementation exists. The gate's expectations are authored by the code they are meant
to test. This repo has already recorded that outcome verbatim
(`CONTINUITY_mt_2026-08-22.md:95`): *"The journey was repaired after running it, not
reading it. Five gates went…"* — and separately, *"decisions must outlive the agent"*:
the walks that produced five spec rulings exist only in a finished agent's context.

**Smallest fix.** Before P6 opens, write the three journeys down — steps, inputs, and
the expected operator-visible output at each step — into
`design/journeys/mt/JOURNEY_{A,B,C}.md`, sourced from the spec's six attribution sites.
P6's gate then has an input it did not author.

## I14 — §10.14 says the correction must land "before implementation"; the plan says it doesn't block P2.

**The claim.** *"None blocks P0–P2"*, versus §10.14: *"`legend.rs` hardcodes
`CHARS_PER_LINE = 35.0` / `LINES_FULL_PLATE = 20.0` per a doc comment … the fork's real
ladder has six rungs and those are the 3.8 mm one … **§4's table must be regenerated
before implementation anyway** … and this correction rides along with that
regeneration."*

**The scenario.** P2 ships the legend (*"the `stderr` legend suggestion, six fields"*,
§5, which is **LIVE** for `mt encode`) sized by numbers §10.14 says are wrong. Worse:
§4 has been **MOVED to the deferred QR spec** (§4, line 1566), so the regeneration the
correction *"rides along with"* is now deferred too — a live section's correction
parked on a deferred artifact. The plan carries neither the dependency nor the
contradiction; P2 has no legend-budget test of any kind.

**Smallest fix.** One line in P2: *"§10.14's legend-budget correction is P2-owned;
either regenerate the two constants against the fork's `FontSizes` ladder before P2
closes, or record explicitly that §5's legend ships with a sub-millimetre known error."*

---

# MINOR / NIT

**M1 (Minor).** `--locked` fails outright before any test runs if `Cargo.lock` is not
committed — measured: *"cannot create the lock file … because --locked was passed"*.
P0's deliverable should say the lockfile is generated and committed as part of the
skeleton.

**M2 (Minor).** `mk-codec` already ships
`consts::tests::nums_string_differs_from_md1` — a two-line assertion that closes
exactly the hole §1 says the drift test cannot catch. The plan says *"a cross-format
negative, which NEITHER sibling has"*, which is true, but overlooks the defense a
sibling **does** have and which is strictly more relevant. Folded into C2's fix.

**M3 (Nit).** The plan cites `§1.1e` four times and `§1.1a` once. §1.1 spans lines
161–1053 and carries **no lettered sub-headings** — the spec cross-references them the
same way, so this is inherited, but an implementer cannot locate them. A one-line
pointer (`§1.1e = the splitting/stripping/modal-length rules, ~line 890-950`) costs
nothing.

**M4 (Nit).** P1's Exit (*"A transaction encodes to `mt1` strings and back,
byte-exact"*) is a stronger and better statement than P1's Gate (*"nextest green"*).
The Exit is the gate; say so.

---

## Half B, answered directly

**Is forking right for a third format?** **Yes — and more clearly than the plan
argues.** The premise it leans on (extraction deferred pending a v1.0 trigger) is dead;
the real reason is the one in the retirement note — HRP-mixed BCH with per-format
target residues is not upstreamable, and there is no shared code worth extracting, only
a shared pattern. `mt` should be built as a **permanent** fork, not as something to be
absorbed later. **Do not reverse the ruling.**

**Does three copies change the arithmetic?** Yes, in two places the plan does not
account for. The cross-format defense surface goes from one pair to three, and the plan
covers one of three — the wrong one (C3). The maintenance obligation goes from
"check the sibling" to "check two siblings and the Go port", with no provenance pin to
make that possible (I11).

**Is the two-test pair sufficient?** **No — and the shortfall is worse than "one
mistake gets through".** Measured: the cross-format negative returns the *same value*
for a copy-pasted constant and a correct one, in both directions, because the HRP is
mixed into the polymod and separates the formats by itself (C2). So the pair defends
against **zero** of the constant-copy mistakes: the drift test has its acknowledged
hole, and the negative has no discriminating power at all. Two concrete survivors were
constructed and machine-verified — the `MK`-constant paste (C3) and the
`hrp_expand("md")` residue (C6, scenario 5) — and the second is the one that actually
destroys plates, because HRP mixing does not save you from getting the HRP itself
wrong.

**Does the drift test's stated hole hold up?** Yes. `md-codec`'s drift test hardcodes
`b"shibbolethnums"` inside the test body (`crates/md-codec/src/bch.rs:135`), so copying
the module wholesale copies the test and its domain string together and the assertion
stays green — exactly as §1 describes. `mk-codec`'s uses the `NUMS_DOMAIN` crate
constant, with the same result. The plan's reasoning on this point is **correct**.

## Half A, answered directly

Of the seven gates (P0, P1, P2, P3, P4, P5, P6): one **cannot pass at all** (P0,
measured); one **fails on conformant code** (P4); one **passes vacuously** in its most
likely configuration (P4 offline); one has **no defined input** (P6); one gates
existence where the phase claims mutation-resistance (P5); and the two that do run
(P1, P2/P3) check the implementation against **its own output**. The single largest
class of hole is not any one gate — it is that **no gate anywhere asserts that
something the format is for actually works**: not error correction (C5), not `verify`'s
margin (C4), not liveness classification (I5). Every listed test asserts that a
decode *fails*.

---

*Report written by the R8 gates-and-fork lens as its final action, per the standing
agent-persistence rule. Read-only: the plan was not edited.*
