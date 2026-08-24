# R11 — pre-implementation gate, `mt` v0.1

**Artifact:** `design/IMPLEMENTATION_PLAN_mt_v0_1.md` @ `e0bbc27`, against
`design/SPEC_mt_v0_1.md` @ the same tip.
**Question asked:** is this plan safe to execute unattended, and does an
implementer following it exactly produce a correct `mt` v0.1?
**Lens:** last look before the gate opens — fold-hold on `b43d829` / `67e17ca` /
`331cbd0` / `0f1747c`, plus an execution walk S0 → P6.
**Counts: 3 Critical / 6 Important / 10 Minor.**

Everything below was machine-checked where a tool could reach it. Values I
computed rather than read are marked **[computed]**.

---

## Section A — BLOCKERS

### C1 — The plan's own first line tells the implementer not to start. *(Critical)*

Plan line 3:

> **Status: DRAFT, pre-R0.** No code is written until an architect review closes
> this at 0 Critical / 0 Important.

An implementer following the plan **exactly** halts on line 3. Nothing in the
document records that a review closed, so there is no way for them to conclude
the condition is satisfied — and no operator to ask.

**Minimal fix.** Replace the status block with the closing state: `GREEN — 0C/0I
as of <date>, closed by design/agent-reports/R11-pre-implementation-gate.md.
Implementation may begin at S0.`

---

### C2 — Open question 4 requires the operator's go-ahead, and it is stale. *(Critical)*

Plan lines 706–708:

> 4. **Repo creation** — `mnemonic-transaction` does not exist yet. Creating a
>    GitHub repo is an outward-facing action and needs the operator's go-ahead,
>    including whether it starts private.

P0's first deliverable is copying the spec and the S0 vector *into*
`mnemonic-transaction`, so this halts the implementer at the start of P0 — the
second phase. It is also factually stale: `bg002h/mnemonic-transaction` exists,
is empty and is private.

**Minimal fix.** Close it: `4. ~~Repo creation~~ **CLOSED** —
bg002h/mnemonic-transaction exists, is EMPTY and PRIVATE. P0 initialises and
pushes to it. v0.1 publishes nothing, tags nothing, releases nothing and makes
nothing public.`

---

### C3 — S0's vector is under-specified in three ways, and P2's gate cannot pass as a result. *(Critical)*

S0 (plan 169–187) says the deliverable is *"One real signed segwit
transaction … Recorded as raw hex, with its txid and wtxid both stated"*. Three
things are missing, and each one lands on a later gate rather than on S0.

**(a) It never says where the transaction comes from — and the choice decides
whether P2 and P3 pass.** If the implementer lifts a confirmed mainnet
transaction (the obvious route: a synced node with `txindex` is on this box —
**[computed]** `getindexinfo` → `txindex.synced: true`, height 963,812), then
with `bitcoin-cli` reachable:

- §8.5 refuses it — every input's `gettxout` is `null` **and** each parent is
  confirmed, which is exactly the refusal condition;
- §6a reports `SPENT — ALREADY CONFIRMED` before any input is classified.

So `mt encode <the pinned vector>` **refuses on the implementer's machine and
succeeds in CI**, where no node exists. P2's gate is *"`mt encode` on each P1
vector reproduces the vector's strings exactly"*. It fails, in the middle of the
night, with the remedy being to redo phase S0.

**(b) It records no input values, and §8.2c/§8.2b need them.** A raw transaction
carries outpoints, not amounts. With no node and no `--input-value`, the fee is
unknown and §8.2b's balance check cannot run at all. The plan's own P2 note
(lines 406–412) says the vectors split into *"**clean** ones that must encode
silently"* and refusal fixtures — but a raw-hex vector can never encode
*silently*: §8.2e mandates a **loud warning** for every raw signed transaction
(*"WARNING: this is a raw signed transaction, not a PSBT"*). The clean half of
that split is unsatisfiable for the artifact S0 actually produces.

**(c) There is no way to force the offline path, and two gates require it.**
P4's gate must run *"BOTH with node fixtures and offline"*, and journey B is
*"no node"*. Of the twelve ruled flags there is no `--offline`; the only lever is
`--bitcoin-cli <path>` pointed at something that does not exist (§10.10 b1:
absent and not found → §6a's no-node warning). The plan never names it, so the
implementer invents one — most likely by editing `PATH`, which is
process-global and will silently change the behaviour of neighbouring tests.

**Minimal fix — one paragraph in S0 and one clause in P2:**

1. S0 produces its fixtures on a **local regtest `bitcoind`** (`bitcoind` is on
   PATH — **[computed]** `/usr/local/bin/bitcoind`, `Bitcoin Satellite v0.2.4`),
   never from mainnet: `createwallet` → `generatetoaddress` →
   `walletcreatefundedpsbt` → `walletprocesspsbt` → `finalizepsbt`. A regtest
   outpoint is unknown to the mainnet node, so `gettxout` returns null with the
   parent **not** confirmed → §8.5 does not fire and §6a reports `UNKNOWN`. That
   is the only transaction provenance under which every later gate passes both
   online and offline.
2. S0 records, for the same transaction: the **finalized PSBT** (base64) as well
   as the raw hex, and **each input's value and outpoint**. The PSBT is the
   clean-encode input (its UTXO records satisfy §8.2c and §8.2b); the raw hex
   exercises §8.2e's warning path; the recorded values back `--input-value` where
   a test wants the raw form to encode.
3. P2's gate names the offline mechanism: `--bitcoin-cli /nonexistent` is how
   every gate and journey that must run air-gapped does so, and P2 asserts that
   flag produces §6a's no-node warning rather than a crash.

---

### I1 — The vector file is Markdown; the pattern `67e17ca` adopts "verbatim" is a JSON file read from disk. *(Important)*

S0's deliverable is `design/vectors/mt1_v1_vectors.md` (plan 169, 225). Fold
`67e17ca` then rules that once copied into the crate the vectors are *"pinned by
SHA-256 with a test asserting the match, `mk`'s pattern adopted verbatim"*.
`mk`'s pattern, read from the crate: `V0_1_SHA256` at
`crates/mk-codec/tests/vectors.rs:41` pins **`src/test_vectors/v0.1.json`**,
resolved from `CARGO_MANIFEST_DIR`. It cannot be adopted verbatim over a
Markdown file at repo root, and the plan makes no decision about how a Rust test
consumes one.

The implementer's three options are all bad unattended: write a Markdown parser
in a test; hand-transcribe the values into Rust constants (which breaks the pin —
the hash would cover a file the test does not read, and transcription is the
exact failure class this constellation has recorded); or invent a sidecar format
mid-phase.

**Minimal fix.** S0 emits **both** forms from the one generator: the human-readable
`design/vectors/mt1_v1_vectors.md` and a machine-readable
`design/vectors/mt1_v1_vectors.json`. P0 copies the `.md` to `design/vectors/`
and the `.json` to `crates/mt-codec/src/test_vectors/mt1_v1.json` (`mk`'s
location shape). P1 pins the **JSON**'s SHA-256. State that the pin covers the
file the test actually reads.

---

### I2 — §10.13(a2) still says the header is 10 symbols, 60 lines under the table that says 11. *(Important)*

Spec 3373–3374, inside the one section P1 builds its wire format from:

> **Since the header is exactly 10 symbols, the payload begins at symbol 10 of
> the data part** — so a reader can locate it by counting characters, with no bit
> arithmetic.

That is the superseded 50-bit ruling's sentence (50 bits = 10 symbols). The
current layout is 55 bits = **11** symbols, stated in the same subsection's table
and in the plan's S0 gate. This is the propagation class the brief named,
surviving inside the corrected section itself. S0's generator is authored
*from* §10.13(a2), so the coin-flip is live: a generator built on this sentence
produces an 11-symbol-short header, fails S0's own gate (*"55 bits, 11 symbols"*),
and leaves the implementer to adjudicate a spec self-contradiction with no
operator.

**Minimal fix.** *"Since the header is exactly **11** symbols, the payload begins
at symbol index **11** — the 12th character of the data part."*

---

### I3 — Every character-length figure in the spec was computed under the 49-bit header. *(Important)*

**[computed]** I recomputed §1.1e's table under 49-, 50- and 55-bit headers
(`3 + ceil((header + 8·bytes)/5) + 13`). All **6 of 6** full-string values and
**6 of 6** last-string values match the **49-bit** header exactly; none matches
the ruled 55-bit one:

| tx bytes | chunks | bytes/chunk | spec full | **55-bit full** | spec last | **55-bit last** |
| --- | --- | --- | --- | --- | --- | --- |
| 162 | 5 | 33 | 79 | **80** | 74 | **75** |
| 405 | 11 | 37 | 85 | **87** | 82 | **83** |
| 535 | 14 | 39 | 89 | **90** | 71 | **72** |
| 742 | 19 | 40 | 90 | **91** | 61 | **63** |
| 560 | 14 | 40 | 90 | **91** | 90 | **91** |
| 2,498 | 63 | 40 | 90 | **91** | 55 | **56** |

The 535-byte / 14-string worked example propagates the same error everywhere it
appears: the total is **1,242**, not 1,228 — spec lines 372, 482, **723 (the
`CUT` row of the live report example, which P2 implements)**, 958, 1923, 3359.
Also stale: *"a person cutting 90 characters"* (937), *"fourteen 89-character
strings"* (948), and the §1.1e error example *"string 7: 88 characters (expected
89)"* (1006).

Why it matters more than a typo: §1.1e's length check is *"the one damage class
BCH cannot [catch]"*, and the section's own argument is that a wrong expected
length turns the message into *"a false [accusation] … sends someone to re-read a
plate that is correct."* An implementer writing the P1/P3 length test has a
tabulated set of expected values sitting in the spec, one character short of what
their correct implementation produces. The dangerous resolution — drop a header
symbol to match the table — is a wire format no conforming decoder can read.

**Minimal fix.** Replace the table with the recomputed column above, change 1,228
→ 1,242 at the six sites, 90 → 91 and 89 → 90 in the prose, and fix the error
example to *"string 7: 89 characters (expected 90)"*. The historical box at
1262–1266 (*"89 to 90 characters"*, comparing 41-bit to 49-bit) is labelled
history and can be left, but should say so.

Margin check, since the change consumes slack: **[computed]** a 40-byte chunk is
55 + 320 = 375 bits = 75 data symbols against `BCH(93,80,8)`'s 80-symbol data
capacity, 88 symbols total against 93. Five symbols of headroom. Nothing
overflows.

---

### I4 — §10.10's tail was not swept by the two folds that superseded it. *(Important)*

`331cbd0` ruled twelve flags and deleted the node-location row; `0f1747c` ruled
the refusal-message format in §8's preamble. Four passages in §10.10 still assert
the pre-fold state, and §10.10 is *"the one place an implementer building the
binary is certain to read"* (R6's own words):

| spec line | says | actually |
| --- | --- | --- |
| 3224 | *"**Still unspecified, and deliberately:** exit codes, and the format of the refusal messages"* | ruled by `0f1747c`, §8 preamble |
| 3226–3228 | *"Each input the table above requires needs some flag — … and **the node location**"* | (b1) deleted it: *"**NOT AN INPUT**"* |
| 3040 | *"**THE SPEC NAMES THREE FLAGS** while requiring SEVEN operator inputs"* | names twelve |
| 3053–3059 | *"The finding it supports is **unchanged and still open** … **§8.7's plate budget has no input at all**, which makes that numbered refusal unrunnable"* | closed by the flag table; §8.7 moved to the deferred QR spec and P5 explicitly does not implement it |

The live hazard is line 3226: an implementer reading it builds a node-location
flag, which is the `--rpc` the operator deleted. The second is line 3224 — the
one item P5's tests are written against, declared unspecified.

**Minimal fix.** *"Still unspecified, and deliberately: exit codes beyond 0."*
Delete *"and the node location"* from the input sentence, mark the THREE
FLAGS/SEVEN INPUTS block `CLOSED 2026-08-23 — see the flag table below`, and
strike the §8.7 sentence.

---

### I5 — P2's deliverable list contradicts itself three bullets apart. *(Important)*

Same fold, plan side. Plan 348–349 lists *"the twelve ruled flags of §10.10,
**spellings included**"*; plan 350–353 rules out `--rpc`. Then plan 354–359:

> Per-input values (§8.2c), the `FROM`/`TO` identities, the free-text `TO` label
> behind its own flag (§10.4), **and the node location**. … **Flag *spellings*
> remain open**; the *paths* are P2's to build

Both clauses are false as of `331cbd0`, and they sit inside the same phase's
deliverable as the bullets that supersede them.

**Minimal fix.** Delete *"and the node location"*; replace *"Flag spellings
remain open"* with *"Spellings are ruled above; the input **paths** are P2's to
build."*

---

### I6 — Two ruled behaviours still have no owning phase; both were reported PARTIAL and the next fold did not close them. *(Important)*

`R9-fold-verification.md` recorded R8 coverage I-3 and I-4 as **PARTIAL**. The
fold that followed (`38fdcdc`) folded R9's B-1/B-2/B-3 and the provenance pin,
and did not return to them. **[computed]** against the current plan:

- **The TTY welcome line** (§10.10) — `grep -ni "tty\|welcome\|ctrl-d"` → **0
  hits**. `mt encode` with nothing piped in blocks on stdin with no prompt.
  §10.10 states the cost: *"a new user concluding the tool does not work and
  leaving, which no other check catches."* This is the item the operator's own
  confusion produced during the journey walk.
- **§6a's encode-shaped no-node warning** (*"The transaction may already be
  unspendable … Consider re-running with a node before cutting"*) —
  `grep -ni "unspendable\|re-running\|no-node"` → **0 hits** outside an unrelated
  sentence. P2 owns three `stderr` blocks and P4 owns only the *recovery*-shaped
  warning; §6a is explicit that these are two texts for two readers.

**Minimal fix.** One bullet each in P2's deliverable, plus a test per block in
P2's "Tests first" (the journey A gate already asserts *"the three mandatory
`stderr` blocks appear"* — make it four, or name them).

---

## Section B — will slow or confuse, will not stop

1. **P2's fixture gate is a disjunction nothing can fail.** *"every fixture is
   either accepted or refused with the ruled message"* — at P2 the refusals do
   not exist yet, so every refusal fixture is accepted and the gate passes.
   Suggest: at P2 assert only the sniffing fixtures; P5 owns the refusal
   fixtures' assertions (it already commits the bijection script).
2. **`tests/refusals.toml` has no seeded list.** P5's gate cites *"the explicit
   list below"* and what follows is a three-field **schema** plus one example
   row. Exhaustiveness is therefore only ever against the implementer's own
   derivation. Here is that derivation, done once so it is not guessed —
   **refuse:** §8.1, §8.2b, §8.2d, §8.2e step 4 (unrecognised input), §8.2f,
   §8.3, §8.5, §8.6(a), §8.6(b), §8.7b, §8.9, **plus §6a's value-mismatch
   refusal** (spec 2023, outside §8's numbering — the case the gate exists for).
   **Warn, not refuse:** §8.2c's legacy-unbound warning, §8.2g, §8.4 (*"Never
   refuse"*). **Excluded:** §8.2 (script validity — not a refusal), §8.7, §8.7c,
   §8.8 (deferred).
3. **§8.2c and §8.2e can be read as disagreeing about the raw-hex case.** §8.2c's
   body says *"Where a record is absent, `mt` **requires** the operator to supply
   that input's value"* while §8.2e rules *"`mt` never refuses the bytes"* and
   marks §8.2b `✗` for *raw, no node* — and then says a raw transaction *"is
   simply the no-UTXO-records input §8.2c already covers"*. §10.10's table scopes
   the refusal to *"when the **PSBT** lacks them"*, which resolves it, but one
   clause in §8.2c (*"where a **PSBT's** record is absent"*) removes the
   coin-flip. It decides whether the pinned vector encodes or refuses.
4. **Refusal ordering is unspecified.** P5 needs each fixture to trip exactly one
   refusal, and several fixtures naturally trip two (an oversized transaction is
   also value-blind). Either specify precedence or state that each fixture must be
   clean in all other respects — the latter is cheaper and is probably what is
   meant.
5. **§8.7b's fixture is ~1.3 MB.** **[computed]** exceeding 32,768 chunks needs
   >1,310,720 payload bytes, and the fixture must still be finalized and signed to
   reach §8.7b. Worth saying it is synthesised (many outputs, one signed input)
   rather than produced by a wallet, and worth deciding whether it is committed or
   generated at test time.
6. **P3 gate 3 grinds 2²⁰ hashes every run.** *"Constructing that input is
   cheap"* is true (~1M double-SHA-256, sub-second at `opt-level = 2`), but the
   result should be **pinned as a fixture** once found — otherwise every CI run
   re-grinds, and a flaky search is indistinguishable from a real failure.
7. **The S0 generator is not copied into the new repo.** P0 copies the spec and
   the vector file; `scripts/gen-mt1-vectors.py` stays in `mnemonic-engrave`.
   Since the plan's rule is *"regenerating them requires re-running S0's
   independent generator, not the crate"*, record the generator's repo, path and
   commit SHA next to the pin so a future re-pin can find it.
8. **The plan never states the licence or the toolchain.** Constellation standard
   is `MIT OR Unlicense` and the siblings pin Rust 1.85.0. An implementer will
   copy a sibling and probably get it right; one line in P0 makes it certain.
9. **§1.1's summary sentence says one extra row, its own example shows two:**
   *"with the `CUT` row appended"* against a block showing `CUT` **and**
   `PREFIX`. Same defect the note directly above it records having fixed once.
   The plan lists both, so no implementer is misled.
10. **§5 line 1736 says *"not the least important of the six"*** in a table that
    has carried **five** fields since `PLATE n OF m` was deleted. Cosmetic; the
    field count is right everywhere else, and the plan says five.

**Checked and clean** (recording these so a later round does not re-derive them):
header layout agrees in both documents — `version(5) + chunk_set_id(20) +
count−1(15) + index(15)`, 55 bits, 11 symbols, no `chunked` bit, 32,768 chunks,
1,280 KB; **[computed]** `MT_REGULAR_CONST = 0x1a2fc877f9528d7c1` **is** the top
65 bits of `SHA-256("shibbolethnumstransaction")`, and `mk`'s
`0x1062435f91072fa5c` is its own domain string's, so the `assert_ne!` tripwires
have real values to assert against; the twelve flags match one-for-one between
§10.10's table and P2's bullet; no live `--rpc`, `--timelocked` or `--immediate`
survives anywhere; `mk-codec`'s string layer is exactly the six modules P1 names
(`ls`-verified) with `consts.rs`/`error.rs` at the crate root; the legend is five
fields in both documents; `bitcoin-cli -stdin`, `txindex` and `cargo-nextest` are
all present on this machine.

---

## Section C — verdict

**NOT SAFE TO EXECUTE as written** — the plan's own status line and open question
4 halt the implementer before any code is written (C1, C2), and S0's vector is
specified so that P2's gate cannot pass (C3); all three are fixable in well under
an hour, after which the answer is yes.
