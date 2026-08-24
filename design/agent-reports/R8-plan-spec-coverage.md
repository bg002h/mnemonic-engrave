# R8 — PLAN vs SPEC coverage lens

Artifact under review: `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_mt_v0_1.md` (269 lines).
Source of truth: `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_mt_v0_1.md` (3,551 lines).

**The one question.** If someone executed every phase of this plan exactly as written,
would they end up with the tool the spec specifies — and is any normative spec
requirement assigned to no phase?

**Answer: no, and yes.** Coverage is not complete. Six requirements would be built
wrong or not at all, and three phase gates cannot be satisfied as written.

| severity | count |
| --- | --- |
| **Critical** | 5 |
| **Important** | 13 |
| **Minor** | 7 |
| **Nit** | 3 |

Method: read §0a, §1.1, §1.1a, §1.1e, §3, §3a, §3b, §5, §6, §6a, §7, §8, §10.10,
§10.13, §10.14, §10.20 and §12 for normative content, and looked for the phase that
owns each. Then walked the phase chain backwards asking whether each gate's input
exists when that gate runs. Two claims were machine-checked rather than reasoned
about (C-1 and C-2 below); both were false.

I did not re-verify the already-machine-verified set named in the brief (5/5 code
citations, the two NUMS derivations, `mk-codec` having no Cargo dependency on
`md-codec`).

---

## Critical

### C-1 — P0's gate cannot pass. `cargo nextest run` with zero tests exits 4.

**The gap.** P0's gate is *"`cargo build` and an **empty** `cargo nextest run --locked`
succeed"*, and P0's tests-first line is *"Nothing to test yet beyond the skeleton
building."* Since nextest 0.9.85 the default is `--no-tests=fail`. Measured on the
installed toolchain, not inferred:

    cargo-nextest 0.9.140 (a9fef2964 2026-07-05)

    Starting 0 tests across 1 binary
    Summary [0.000s] 0 tests run: 0 passed, 0 skipped
    error: no tests to run
    (hint: use `--no-tests` to customize)
    EXITCODE=4

`mt-cli` is a binary crate (no default test module) and `mt-codec` would be created
empty by the plan's own description, so the workspace has zero tests at P0. **The
first gate in the plan is red on the first run**, and its author would reasonably
conclude the skeleton is broken rather than that the gate is.

**Owning phase.** P0.

**Smallest change.** Gate on `cargo nextest run --locked --no-tests=pass`, or state
that P0 ships one trivial test so the gate has something to run.

---

### C-2 — P1 names the wrong port source. `mk-codec`'s header is 5-bit-symbol-aligned and has no bit packer; §10.13(a2) binds `mt1` to `md-codec`'s `BitWriter`.

**The requirement.** §10.13(a2): *"Fields are written most-significant-bit first in
the order above, **matching `md-codec`'s `BitWriter`**. The 49-bit header is followed
immediately by the chunk payload with **no padding between them**."* 49 bits is not a
multiple of 5, and `count`/`index` are 12 bits each.

**What the plan says.** P1: *"**Ported from `mk-codec`, which is the closest sibling**
— not from `md-codec`, and not written from scratch."*

**Why that is wrong, checked against the sibling rather than assumed:**

| | `mk-codec` | `md-codec` |
| --- | --- | --- |
| header granularity | **5-bit symbols.** `crates/mk-codec/src/string_layer/header.rs:1` — *"5-bit-symbol-aligned string-layer header"*; `:9` — *"All field widths are exactly 5 bits unless otherwise noted"* | bit-level |
| header variants | two — `SINGLE_HEADER_SYMBOLS = 2`, `CHUNKED_HEADER_SYMBOLS = 8` | one |
| a `BitWriter` / `BitReader` | **none.** `grep -rl 'BitWriter\|BitReader' crates/mk-codec/src/` → no matches | `crates/md-codec/src/bitstream.rs`, used by `chunk.rs:37` (`Header::write(&self, w: &mut BitWriter)`) |

`mt1` has no single-string variant, and its 12-bit fields cannot be expressed in
`mk-codec`'s symbol-aligned header at all. An implementer following P1 literally
either invents a bit packer the plan told them not to copy, or reaches for a
symbol-aligned 50-bit header — **which is the silent-divergence failure §10.13(a2)
was written to close** (*"the two implementations disagree silently and produce
nonsense rather than a clean refusal"*).

The plan's own §1 is correct where it matters and P1 overreaches past it: §1 argues
only that `mt-codec` should **fork rather than depend**, citing `mk` as precedent for
the *decision*. P1 turns that into `mk` as the source of the *code*, which the
header layout forbids.

**Owning phase.** P1.

**Smallest change.** P1 names `md-codec` as the port source for `header.rs`,
`chunk.rs` and `bitstream.rs` (the 49-bit MSB-first packing §10.13(a2) binds), and
keeps `mk` only as the precedent for forking. The BCH primitives may come from
either — they are constant-agnostic.

---

### C-3 — P2 prints `PLATE n OF m`, which §0a deletes as "a false completeness claim on permanent steel", and no phase prints the field that replaced it.

**The requirement.** §0a rules explicitly that `mt encode`'s suggested text **is not
§5's field set applied verbatim** (U-5):

| printed | text |
| --- | --- |
| **once** | `BEARER…`, `FROM`, `TO`, `LOCKED TO BLOCK n ~SEASON year`, `FORMAT: mt1 codex32` |
| **per string** | `n/m` — string `n` of `m`, which `mt` knows exactly |

*"**`PLATE n OF m` is dropped, because `mt` cannot compute `m`** … **`PLATE 1 OF 1`
cut onto each of five plates is a false completeness claim on permanent steel**, read
by someone who then stops looking for the other four."*

**What the plan says.** P2: *"the `stderr` legend suggestion, **six fields** including
`FORMAT: mt1 codex32` (**§5**)."*

§5's six fields are `BEARER`, `FROM WALLET`, `LOCKED TO BLOCK`, `TO`, **`PLATE n OF
m`**, `FORMAT: mt1 codex32`. An implementer given "six fields, see §5" prints
`PLATE n OF m`. The string `STRING n` does not appear anywhere in the plan
(`grep -c 'STRING n'` → 0), and neither does `PLATE` (→ 0), so the per-string label
`mt` *can* compute is assigned to no phase.

**Owning phase.** P2.

**Smallest change.** P2 cites **§0a** rather than §5 for the field set, and lists the
five once-fields plus the per-string `STRING n OF m` label explicitly.

---

### C-4 — P4's gate is unsatisfiable. §1.1's own row-presence table makes the three callers' reports differ in more than stream and suffix.

**The gate.** P4: *"The report renders **identically** from `encode`, `decode` and
`inspect` for one vector, differing only in stream and the `CUT`/`PREFIX` suffix."*

**Why it cannot hold as written.** §1.1's row table:

- **`FEE`** — *"present when: a node is reachable, **or** the input was a PSBT carrying
  values."* `encode` is handed a PSBT with `witness_utxo`/`non_witness_utxo` records,
  so `FEE` is computable. `inspect` is handed `mt1` strings, and §1.1 says so in
  terms: *"the decoded transaction carries its inputs' outpoints but not their
  values, so **without a node the fee and provenance rows are simply unavailable**."*
  Offline, `encode` prints a fee and `inspect` prints `UNKNOWN`. Not identical.
- **`mt1 SET`** — *"present when: the caller had strings — `inspect`, `decode`,
  `verify`."* `encode` is not in that list, while the same section ends *"no caller
  reorders, reformats, or **drops a row**."* The gate asserts an identity the spec
  leaves ambiguous for this row.

P4's tests-first line makes it worse rather than better: *"Node responses are
**fixtures, not a live node**, so the tests run air-gapped"* — which pins the gate to
the offline case, the one case where `FEE` provably differs.

**Owning phase.** P4.

**Smallest change.** The gate runs all three callers **against the same node
fixture** (so `FEE` and the provenance column resolve the same way for each), and
states that `encode` emits the `mt1 SET` row. Alternatively, scope the identity to
the rows both callers can produce and say which those are.

---

### C-5 — P1 has no test that can catch a wrong HRP, a wrong `version`, or a plain `count`, and the plan omits all three values from its own normative restatement.

**The requirement.** §10.13(a2) exists because *"R4 found **five** things an
implementer would otherwise guess — and **two of the guesses produce plates another
implementation cannot read**"*:

| field | ruled value | in the plan? |
| --- | --- | --- |
| HRP | **`"mt"`, NOT `"mt1"`** (§10.13 b) | **no** — `HRP` appears once, inside a quotation of §12.22 |
| `version` | **`0b0001`** | **no** — the plan writes `version(4)`, a width, never the value |
| `count` | stores **`count − 1`** | yes |
| `chunked` bit | `1`, always, retained | yes (present in the field list) |
| bit order | MSB-first, `md-codec` `BitWriter` | **no** (see C-2) |

§10.13(b) records what the HRP costs: *"An implementer reading 'its own HRP, `mt1`'
would compute `hrp_expand("mt1")` … **every plate written by one implementation fails
the other's checksum**, and fails it with a *'damaged beyond correction'* diagnostic
that points the recoverer at their steel rather than at their software."* The spec
also records that this was filed as a Minor and skipped once already.

**Why P1's tests do not catch it.** P1 has three tests: the drift test, the
cross-format negative, and a chunking property test.

- The **drift test** pins `MT_REGULAR_CONST` against its domain string. Silent on HRP,
  version and offsets.
- The **cross-format negative** asserts an `mt1` chunk fails `md1` verification. A
  wrong HRP still yields a residue that fails `md1` — the test passes either way.
- The **chunking property test** round-trips against the implementation itself.

And P1's vectors are **produced by the implementation under test** (*"Test vectors are
produced here"*), so they inherit any of these defects and then vouch for them
through P2–P4. This is the plan's own cited hazard — *"a corpus can be uniformly
wrong"* — with no independent oracle anywhere in the phase.

**Owning phase.** P1.

**Smallest change.** Add the three values to P1's normative bullet list, and add a
**header golden-vector test**: one hand-computed 49-bit header (chosen so `count > 1`
and `index > 0`) asserted symbol-by-symbol. That single test pins HRP, version, the
`count − 1` offset, the `chunked` bit and the bit order at once. Additionally, at
least one vector's content id should be produced by an independent tool
(`bitcoin-cli decoderawtransaction` gives the txid) rather than by `mt-codec`.

---

## Important

### I-1 — P1's module set is `md-codec`'s file list mis-attributed to `mk-codec`, and it omits the module that joins the pieces.

P1: *"`header.rs`, `chunk.rs`, `nums.rs`, `bch.rs`, `bch_decode.rs`, `error.rs` — the
module set `mk-codec` carries in `crates/mk-codec/src/string_layer/`."*

Measured — `ls /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/`:

    bch.rs  bch_decode.rs  chunk.rs  header.rs  mod.rs  pipeline.rs

There is no `nums.rs` and no `error.rs` there (those live at `mk-codec` crate root as
`consts.rs` and `error.rs`). The six names the plan lists are `md-codec`'s
(`bch.rs`, `bch_decode.rs`, `chunk.rs`, `header.rs`, `nums.rs`, `error.rs` all exist
in `crates/md-codec/src/`) — which corroborates C-2.

**The omission that matters is `pipeline.rs`**, whose own doc comment reads *"Public
encode/decode entry points: `KeyCard` ↔ `Vec<String>`"* — it is the module that turns
header + chunk + BCH into an actual codec API. P1's exit criterion (*"A transaction
encodes to `mt1` strings and back, byte-exact"*) requires it, and no phase names it.
This is the constellation's recorded failure shape: a plan that lists components and
omits the call that joins them.

**Owning phase.** P1. **Fix:** correct the attribution and add the entry-point module
(`transaction ↔ Vec<String>`) to the list.

---

### I-2 — `verify`'s margin report — the Critical the journey walk found — is assigned to no phase.

§1.1: *"**`verify` REPORTS ITS MARGIN, not just its verdict.** Usability journey walk,
U-2 — the one Critical it found."* The normative output is the `CORRECTION APPLIED. 3
chunks needed repair:` block with `chunk 7  4 of 4 symbols  … <-- NO MARGIN LEFT` and
`Chunk 7 is at its correction limit … Re-cut it.`

P3's only related line is *"correction reporting: positions **1-based**,
`position = codeword_index + 4`, with before-values (§1.1)"*. The word `margin` does
not appear in the plan (`grep -c margin` → 0). The before-values are covered; the
**budget consumed out of `t = 4`, the limit warning and the re-cut instruction are
not** — which is precisely the U-2 defect being re-opened at the implementation layer.

P3's negative gate does cover the descending suspect list on the FAILED path. It is
the **OK path** that is unowned, and the OK path is the whole of U-2.

**Owning phase.** P3. **Fix:** name the margin block in P3's deliverable and add a
test that a 4-of-4-corrected chunk produces the limit warning while still returning
OK.

---

### I-3 — Three mandatory encode-time stderr blocks have no owning phase.

All three are normative, all three are `mt encode` output, and none appears in P2:

1. **The correction-coverage block, "printed ALWAYS, before cutting"** (§1.1, operator
   ruling): *"Before you cut: mt corrects up to 4 wrong CHARACTERS per string … It
   cannot repair a missing STRING or a lost PLATE. There is no redundancy."* §1.1
   states why it exists: *"Nothing in `mt`'s output contradicts the impression that
   'error correction' has the operator covered, and §1.8's zero-redundancy ruling
   lives only in the spec."* The plan's §3 asserts the mitigation is *"cutting
   duplicate copies, which P3's duplicate resolution supports"* — but the part that
   **tells the operator to cut them** is built by nobody.
2. **The verify-the-steel block** (§1.1): *"Now engrave these strings. When you are
   done, verify the ENGRAVING, not this output: `mt verify < typed-from-plates.txt`"*.
   Journey finding U-3.
3. **§6a's encode-shaped no-node warning** — *"The transaction may already be
   unspendable. A plate is ~21 minutes. Consider re-running with a node before
   cutting."* P4 covers only the **recovery-shaped** warning (*"the resolution line
   naming both a node and a block explorer"*). §6a is explicit that these are two
   different texts for two different readers.

**Owning phase.** P2 for (1) and (2); P2 or P4 for (3). **Fix:** list all three in
P2's deliverable.

---

### I-4 — Three ruled CLI behaviours are implemented by no phase.

These are **behaviour**, not spellings, so §10.10's "only spellings remain open" does
not cover them:

| behaviour | ruled at | in the plan |
| --- | --- | --- |
| **`verify --transaction <psbt\|hex>`** — compares the **full 32-byte txid**, not the 20-bit set id (R6 adversarial I-1), and a supplied PSBT is compared against its **extracted** transaction | §1.1 | absent (`grep -c -- '--transaction'` → 0) |
| **`--quiet`** — suppresses the inspection report only; never warnings or refusals, on any verb; does not relax §1.1a's stdout rule | §1.1a, §10.10(b) | absent (`grep -c quiet` → 0) |
| **the TTY welcome line** — `mt encode` on a TTY prints *"reading a transaction from stdin. Paste it and press Ctrl-D"* | §10.10 | absent (`grep -c TTY` → 0, `grep -c stdin` → 0) |

The TTY line is the one the operator's own confusion produced (*"stdin doesn't mean
from the command line?"*), and §10.10 says what its absence costs: *"a new user
concluding the tool does not work and leaving, which no other check catches."*

**Owning phase.** P3 for `--transaction`, P3 for `--quiet`, P2 for the TTY line.

---

### I-5 — §10.10's table lists five operator inputs `mt` needs, and no phase builds an input path for any of them.

§10.10, *"The inputs `mt` needs, and which section needs them"*, minus the QR-deferred
rows:

| input | needed by | absent → |
| --- | --- | --- |
| `FROM` wallet id / fingerprint | §5 | warn, engrave blank |
| `TO` wallet id / fingerprint | §5 | warn, engrave blank |
| `TO` free-text label | §12.4 | **requires an explicit flag** by ruling |
| **input values, per input** | §8.2c, when the PSBT lacks them | **refuse** |
| node location | §6a | the no-node warning |

`FROM` does not appear in the plan at all (`grep -c FROM` → 0). P2 promises the legend
text that consumes `FROM`/`TO` without building any way to supply them; §12.4's
three-state `TO` rule (given / blank+loudly-warned / free-text-behind-a-flag) is
unowned. The per-input value path is the one that carries a **refusal**, and it is
also the input P5's §8.2c test needs.

§10.10 states why this is a plan-level concern rather than a CLI-design one: *"two
implementers given this table will still choose different flag **spellings**, but they
will at least build the same tool. Given different tables they build different
tools."* Naming the flags is deferred correctly; naming the **inputs** is not.

**Owning phase.** P2 (FROM/TO/values/node location), with P5 owning the refusal on
absent values.

---

### I-6 — §6a's value-mismatch refusal sits outside §8's numbering, so P5's exhaustiveness gate structurally cannot see it.

§6a: *"`mt` compares the fetched `value` against the PSBT's UTXO record for that input
and **refuses on mismatch**, naming both numbers."* R3's information lens filed this
(I-2) precisely because the check was being thrown away: *"since §8.2's removal, the
chain's own answer is **the only value check `mt` has** for a segwit input."*

It is a refusal, and it is **not** one of §8's numbered items. P5's gate is *"every
numbered refusal in §8 has a test, and a script asserts the union is exhaustive
against the spec's numbering"* — a script keyed to §8's numbering returns clean while
this refusal is unimplemented and untested. P4 owns the node but lists only report
rows and liveness.

**Owning phase.** P4 (it needs the node) or P5. **Fix:** name it explicitly in one of
them; a numbering-keyed exhaustiveness script cannot be the thing that catches it.

---

### I-7 — P5's exhaustiveness gate cannot close: §8.2 and §8.8 are numbered items in §8 that are not v0.1 refusals, and P5 excludes only §8.7 / §8.7c.

§8's numbered items are 1, 2, 2b, 2c, 2d, 2e, 2f, 2g, 3, 4, 5, 6, 7, 7b, 7c, 8, 9.

- **§8.2** — *"Script validity is **NOT** checked in v0.1"*. Not a refusal; it is the
  removal of one.
- **§8.8** — *"Module size is the operator's choice, defaulting to 0.60 mm — **not a
  refusal**"*, and it is QR/engraving-geometry material whose ruling cites §10.1,
  which is MOVED to `SPEC_mt_qr_DEFERRED.md`. No v0.1 behaviour.

P5's "Not implemented" line names only §8.7 and §8.7c. A script asserting *"every
numbered refusal in §8 has a test"* against the spec's numbering therefore demands
two tests that cannot exist, and the phase cannot go green. This is the plan's own
rule biting it — *"a gate that has never run is a hypothesis"* — because the
exhaustiveness script is the one gate whose failure mode is visible only on first
execution.

**Owning phase.** P5. **Fix:** the exclusion list becomes §8.2, §8.7, §8.7c, §8.8,
each with the one-line reason, and the script asserts against that explicit
allowlist so a *new* §8 item still trips it.

---

### I-8 — No phase creates the PSBT and malformed-input fixture corpus that P2's and P5's tests both consume.

P1's vectors are *"the transaction bytes, the expected strings, and the content id"* —
transactions, not PSBTs and not malformed inputs. But the tests P2 and P5 specify need
a corpus nobody is scheduled to build:

| needed by | fixture |
| --- | --- |
| P2 sniffing, one per branch | binary PSBT with `psbt\xff`; base64 PSBT line-wrapped at 64 and 76 columns; raw hex, uppercase, with and without `0x`; a **hex-encoded PSBT** (`70736274ff…`) that must be refused naming the real problem; something matching nothing |
| P5 §8.1 / §8.3 | a signed-but-**not-finalized** PSBT |
| P5 §8.2b | outputs > inputs; duplicate outpoints; empty `vin`; a fee above 25,000 sat/vB |
| P5 §8.2d | a PSBT whose `non_witness_utxo` does not hash to the input's txid |
| P5 §8.6(a) | a `SIGHASH_NONE` input **in a legacy `scriptSig`** (R2 lens 2, S-1) |
| P5 §8.6(b) | a taproot script-path witness `[preimage(64), script, control_block(65)]` — the case the spec says defeats a length-based recogniser |
| P5 §8.2g | a source file at mode 0644, plus the FIFO and TTY cases |
| P5 §8.7b | a payload above 163,840 bytes |

Several of these are hours of work each (the taproot script-path witness in
particular), and the plan prices none of them. §8.6(b)'s fixture is the one that
decides whether the refusal is real or decorative.

**Owning phase.** P1 should extend its vector artifact to a fixture corpus, or P5
should carry the corpus explicitly as part of its tests-first step.

---

### I-9 — The plan cites `§12.13` three times and `§12.10` once. Neither section exists.

Measured — the numbered items under `## 12. Appendix` are:

    4 5 6 7 8 11 12 15 16 18 19 21 22 23

There is **no §12.10 and no §12.13**; §12 holds only the *settled* questions, and
10/13/14/20 are exactly the four that stayed in §10. The plan's citations:

| plan line | citation | actual home |
| --- | --- | --- |
| 133 | `(§12.13 a2)` — the 49-bit header layout | §10.13(a2) |
| 136 | `(§12.13 c)` — content id, top 20 bits of the txid | §10.13(c) |
| 165 | `(§1.1e, §12.10)` — lowercase, ungrouped stdout | §10.10 |

`§12.22` and `§12.6` do resolve, so the error is not systematic and would not be
caught by pattern.

**Why the gate missed it.** `scripts/plan-cite-check.sh` states its own blind spot:
*"It only resolves citations that LOOK like `path/file.ext:N`."* §-cross-references
are checked by `scripts/spec-structure-check.sh`, which the cycle runs **on the spec,
not on the plan**. So the plan's cross-references are gated by nothing.

**Why it is more than cosmetic here.** §12 is *settled* material and §10 is *open*
material. Citing §12.13 tells a reader the header layout is closed reasoning in the
appendix; it is in fact one of the four questions the plan itself lists as open two
pages later. And §10.13(a2) is the section carrying the three values C-5 shows the
plan omitted — so the one pointer to them is dangling.

**Owning phase.** The plan itself. **Fix:** rewrite the three citations, and run
`spec-structure-check.sh` (or its cross-ref half) against the plan too.

---

### I-10 — "§10.10 (CLI surface — flag *spellings* only)" is false. Exit codes and the refusal-message format are open too, and P5's deliverable is exactly the latter.

§10.10's closing paragraph: *"**Still unspecified, and deliberately:** the flag
spellings, **exit codes**, and **the format of the refusal messages §8 promises will
'name the number that caused it'**."* Only one exit code is pinned (`0 = every check
passed`), plus §1.1a's requirement that `decode` exit non-zero on failure.

P5's deliverable is *"§8 in full, **each refusal naming the number that caused it**"*
and its gate is one test per refusal. If the message format is open, every one of
those tests is asserting on a shape nobody has agreed, and P5 is the phase that will
be forced to invent it — silently, in test assertions, which is the worst place for a
user-facing format to get decided.

**Owning phase.** The plan's §4 item 2, and P5. **Fix:** state the three open pieces
of §10.10 rather than one, and give the refusal-message format an owning phase (P5)
so it is a decision rather than a by-product.

---

### I-11 — The 1-based chunk-numbering rule is assigned to no phase, and it is the rule whose violation costs 21 minutes of steel.

§1.1, in capitals: *"**ALL HUMAN-FACING OUTPUT NUMBERS CHUNKS FROM 1** — `chunk n`
means wire `index n−1`. The zero-based `index` is a wire field (§10.13 a2) and
**appears nowhere in output**."* The spec spells out the consequence: *"An operator
re-cutting the wrong string spends ~21 minutes duplicating a good plate and leaves
the one-scratch-from-unrecoverable string on the shelf."* It also governs the FAILED
report's ranked suspect list.

P3 assigns the **position** convention (`position = codeword_index + 4`) and nothing
assigns the **chunk-number** convention. `chunk n` does not appear in the plan. The
rule spans P3 (correction and duplicate reports) and P4 (the `mt1 SET` row's
`1..14 all present`), so it needs stating once at a level above either.

**Owning phase.** P1 is the natural home (the codec is where 0-based `index` lives and
where the conversion can be made a typed boundary), enforced by tests in P3 and P4.
**Fix:** one bullet, plus one test asserting no report ever prints a 0-based index.

---

### I-12 — P5's exhaustiveness script and P6's journeys read a spec that lives in a different repository, and no phase puts it there.

The code lives in a **new repo, `mnemonic-transaction`** (P0, and §4 item 3 notes it
does not exist yet). The spec lives at
`/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_mt_v0_1.md`.

- P5's gate: *"a script asserts the **union is exhaustive** against **the spec's
  numbering**"* — the script must parse a file in another repo, in CI, in a repo that
  has no reason to check `mnemonic-engrave` out.
- P6's journeys are *"the three walked journeys (§ the spec's own)"* — the acceptance
  criteria are spec prose.

No phase copies, vendors, submodules or pins the spec into the new repo, and P0's
deliverable (workspace, lints, CI, profiles) does not mention it. As written, P5's
gate is a script that cannot find its input on any machine but this one.

**Owning phase.** P0. **Fix:** one line in P0's deliverable — the spec (and the
deferred QR spec, marked unread) is vendored into `mnemonic-transaction/design/` at a
recorded commit, so every later gate has a local, pinned input.

---

### I-13 — P2's gate stops passing once P5 lands, because P1's vectors carry no input values.

**The chain.** P1's vectors record *"the transaction bytes, the expected strings, and
the content id"*. P2's gate is *"`mt encode` on each P1 vector reproduces the vector's
strings exactly"* — so the vector reaches `mt encode` as a **raw transaction**, since
that is all the vector holds.

At P2 time that works, because the refusals do not exist yet. At P5 time it stops:
§10.10's input table says **input values, absent → refuse**, and §8.2c requires the
operator to supply them per input when the PSBT lacks them. A raw transaction is, in
§8.2e's own words, *"simply the **no-UTXO-records** input §8.2c already covers"*. With
no node in CI, every P2 vector is refused.

This is the plan's own rule about gates that were true once — the phase closes green
and the whole-suite gate goes red two phases later, with the fix landing on top of
other work.

**Owning phase.** P1 (the vector format) and P2 (the gate). **Fix:** each P1 vector
carries a **finalized PSBT** alongside the raw bytes and the expected strings, and
P2's gate states which form it feeds. That also gives P5 half of I-8's corpus for
free.

---

## Minor

- **M-1 — §10.20 has no owning phase.** *"Legacy inputs are txid-malleable … **Worth a
  sentence somewhere a recoverer will read.**"* That is an output requirement in
  embryo, landing naturally in P4 (the report) or P3 (`verify`'s FAILED text, where a
  superseded plate looks exactly like miscorrection). The plan's §4 item 2 promises to
  say *"where the four remaining spec open questions land"* and lands only §10.10.

- **M-2 — P4's live-node smoke test is described but never scheduled.** *"A synced
  `bitcoind` is available on this machine, and one manual run against it is worth
  doing."* Correct that it must not gate CI; but as written it is a suggestion with no
  owner and no moment, and everything else about the node is a fixture. Bind it to
  P4's close as a non-CI checklist item, or it will not happen.

- **M-3 — P6 has no "Tests first" line**, breaking the shape every other phase
  follows and the plan's own opening rule (*"Each phase is **tests first**"*).

- **M-4 — `verify` as a fourth report caller is unresolved.** §1.1's `mt1 SET` row
  names *"`inspect`, `decode`, `verify`"*, while the per-caller rule names only
  `inspect`, `decode`, `encode`. P4's gate picks the second reading without saying so.
  One sentence in P4 settles which `verify` prints.

- **M-5 — The content-id basis is not restated where it is used.** §10.13(c) resolves
  the id to the **extracted** transaction's txid, not `unsigned_tx` (*"Two implementers
  picking differently would produce plates neither could reassemble"*), and §1.1's `TX`
  row is the **txid**, not the wtxid (R6 adversarial C-1 — *"segwit is the normal case
  here, not an edge"*). P1 says only *"top 20 bits of the txid in display form"* and P2
  never mentions extraction. `unsigned_tx`, `extracted` and `wtxid` all appear zero
  times in the plan.

- **M-6 — Duplicate-resolution row 1's "ANNOUNCE it" is not named.** §1.1: *"**use the
  good one, and ANNOUNCE it** — printed as a finding, not a log line"*, together with
  the disclosure that a badly-mistyped genuine string looks identical. P3 covers the
  partition rule and the refusal; the announcement is the part that keeps `mt` from
  claiming a proof it does not have.

- **M-7 — §10.13's own closing sentence is not reconciled.** It reads *"It still
  blocks *code* for both verbs, since both fragment with this header"*, against the
  plan's *"None blocks P0–P2."* I read §10.13 as meaning its decisions are all made and
  merely load-bearing, so the plan's claim is right — but a reader has to do that
  reconciliation themselves, on the section P1 is entirely built from. One clause fixes
  it.

---

## Nit

- **N-1** — P1's vectors do not state the case of the expected strings; §1.1e rules
  `mt encode` writes **lowercase**, and P2's gate is a byte-exact comparison against
  them.
- **N-2** — §1.1e's rule that the **final** chunk's expected length is not checked
  until the set is complete is not restated in P3, and it is the branch an implementer
  most easily gets wrong after implementing the modal-length rule.
- **N-3** — §8.7b's ceiling refusal needs a payload above 163,840 bytes to test. P5
  does not note that this fixture is unlike every other one it needs.

---

## What is correct, stated so it is not re-derived

These were checked and are sound; a later round should not spend budget here.

- **The §1 fork decision** — `mk-codec` has no dependency on `md-codec` and carries its
  own constant. The *decision* is right; only the port-source inference from it is
  wrong (C-2).
- **The two-test defence of the copy-paste hazard** — the drift test and the
  cross-format negative do catch different failures, and the plan's statement of what
  each does *not* catch is accurate. It is simply not sufficient (C-5).
- **P5's mutation discipline** — *"each must be shown to FAIL when the refusal is
  removed"* — is the right gate for a refusal set, correctly scoped.
- **The four open questions.** Checked against what §10.10, §10.13, §10.14 and §10.20
  actually say. **"None blocks P0–P2" is correct**: §10.13 is `RULED, ready to build`
  with every decision made; §10.14 is plate-area material for the deferred QR verb and
  §10.21 notes the legend is *"free for `mt encode`, where the legend is `stderr` text
  and `mt` owns no layout"*; §10.20 wants one sentence of output. **"§10.10's spellings
  must close before P2 ships" is correct** — and understated, per I-10.
- **The journeys P6 names ARE the journeys the spec walked.** Verified against
  `design/agent-reports/mt-spec-usability-journeys.md` (Journey A = the operator
  obtaining and engraving a finalized PSBT, 8 steps; Journey B = the recoverer finding
  plates in a drawer, 7 steps) and, for Journey C, against the four spec sites citing
  it (§1.1 duplicate chunks from a re-cut plate at line 221; §1.1 verify localises
  corrections at 365; §1.1 the re-derivation failure report at 410; §2's UR
  contradiction at 1098) plus commits `f843d39` and `31edd79`. P6's three — *"the
  operator encoding a finalized PSBT, the 2040 recoverer with strings and nothing else,
  and the operator who miscuts and re-cuts"* — map one-to-one onto A, B and C. **No
  finding.**
- **`mt qr` is absent from every phase**, as ruled, and nothing in the plan reads
  `SPEC_mt_qr_DEFERRED.md`.
