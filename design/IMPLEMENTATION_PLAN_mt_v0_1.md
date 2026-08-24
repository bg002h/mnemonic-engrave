# IMPLEMENTATION PLAN — `mt` v0.1

> **Status: GREEN — 0 Critical / 0 Important as of 2026-08-23**, closed by
> `design/agent-reports/R11-pre-implementation-gate.md` after seven review
> lenses. **Implementation may begin at S0.** Risk-set work: funds, addresses,
> a new normative wire format — so the post-implementation adversarial review
> over the whole diff is mandatory and non-deferrable.
>
> **Source of truth is `design/SPEC_mt_v0_1.md`**, GREEN at 0C/0I as of
> 2026-08-23 after R6 (three lenses, 6C + 27I) and R7 (fold verification,
> 30 FIXED / 3 PARTIAL / 0 NOT FIXED). Deferred QR material lives in
> `design/SPEC_mt_qr_DEFERRED.md` and **nothing in this plan reads it**.
>
> This plan does not re-decide anything the spec settled. Where it appears to,
> that is a defect in this plan.

---

## 0. What is being built

A new repository, `mnemonic-transaction`, with two crates matching the
constellation pattern (`md`, `mk`, `ms` all have this shape):

| crate | what it is |
| --- | --- |
| **`mt-codec`** | the `mt1` wire format — chunk header, chunking, BCH, content id. No I/O, no node, no CLI. |
| **`mt-cli`** | the `mt` binary — four verbs, refusals, reports, node queries. |

Four verbs, matching `md` and `mk`: **`encode`**, **`decode`**, **`verify`**,
**`inspect`** (§1.1). `mt qr` is deferred out of v0.1 (§0a) and **no phase below
implements it**.

---

## 1. The decision this plan has to make first — and the constellation already made it

**`mt-codec` FORKS the BCH primitives, exactly as `mk-codec` did. It does not
depend on `md-codec`, and it does not parameterise it.**

An earlier draft of this plan proposed the opposite — adding
`bch_create_checksum_regular_with_const` upstream and depending on it — on the
grounds that `md-codec`'s API is half-parameterised already
(`decode_regular_errors(residue_xor_const, …)` at
`crates/md-codec/src/bch_decode.rs:411` takes the constant, while
`bch_create_checksum_regular` at `crates/md-codec/src/bch.rs:86` and
`bch_verify_regular` at `crates/md-codec/src/bch.rs:99` bake `MD_REGULAR_CONST`
in). **That asymmetry is real, and it is not an invitation.** Checked against
the sibling rather than assumed:

| | `mk-codec` |
| --- | --- |
| depends on `md-codec`? | **no** — the name appears only in doc comments |
| BCH primitives | its own, in `crates/mk-codec/src/string_layer/` (`bch.rs`, `bch_decode.rs`, `chunk.rs`, `header.rs`) |
| its NUMS constant | its own, `MK_REGULAR_CONST` (`crates/mk-codec/src/consts.rs:18`) |
| extraction into a shared crate | **RETIRED, not deferred** — `mnemonic-key/design/FOLLOWUPS.md`, `mc-codex32-extraction-retired-2026-05-03` |

So the established pattern is **fork per codec and carry your own constant** —
and R8 corrected this plan's account of *why*, which is worth having right
because the corrected reason is stronger.

> **An earlier draft said extraction was "deliberately deferred until both crates
> reach v1.0". It was RETIRED on 2026-05-03**, recorded in
> `mnemonic-key/design/FOLLOWUPS.md` as `mc-codex32-extraction-retired-2026-05-03`,
> on a technical finding rather than a scheduling one:
>
> > *"md1 and mk1 use HRP-mixed BCH with per-format target residues that are NOT
> > upstreamable … There is no longer shared code worth extracting — only a
> > shared **pattern** … md1↔mk1 BCH plumbing stays forked **indefinitely**."*
>
> **That is a better argument than the one it replaces.** "Wait until the
> formats settle" implies the fork is temporary and invites building `mt` to be
> absorbed later. **The real reason is that HRP-mixing and per-format residues
> make the code unshareable in principle** — there is nothing to absorb, now or
> at v1.0. `mt` is the third instance of a pattern, not the third tenant of a
> future crate, and it should be built that way.

**This deletes a whole phase of upstream work.** No change to `md-codec`, no
semver question, no publish decision, and the Go port is untouched.

### How the constant is defended

**Operator ruling 2026-08-23: cross-format verification is ABANDONED** —
*"it's unlikely and not worth the effort"* — and the spec's own statement of the
hazard was **false as worded**. §12.22 warned that a copied constant makes
*"`mt1` chunks verify as `md1` chunks"*. It cannot: the HRP is mixed into the
checksum on both sides, so differing HRPs separate the formats **by themselves**,
whatever the constant is. The spec is corrected; the cross-format negative test
this plan proposed is **deleted**, because it returned the same result whether or
not a constant had been copied and therefore measured nothing.

**The real hazard is intra-format, and worse for being silent.** A wrong
constant — copied *or* mistyped — produces chunks that are **self-consistent and
unreadable by every other implementation**, and it surfaces at *recovery*, where
it is indistinguishable from steel damage: checksum failures on a physically
perfect plate, years later, with no second copy of the transaction anywhere.

**So the defence is one artifact and two cheap tripwires:**

1. **A spec-authored, independently derived PINNED BYTE-EXACT VECTOR** — a real
   signed segwit transaction to its exact `mt1` strings, plus a 13-symbol
   checksum micro-vector, with the generator script committed. **It lands in the
   spec before `mt-codec`'s first commit**, so the implementation is checked
   against bytes it did not produce. This is the load-bearing item; everything
   else is a tripwire.

2. **The drift test**, copying the sibling pattern verbatim:
   `SHA-256("shibbolethnumstransaction")` top 65 bits `== MT_REGULAR_CONST`.
   Both siblings have one and `md-codec`'s cites `mk`'s as its model. Cheap,
   and it catches the constant and its domain string drifting apart.

3. **Four `assert_ne!` lines against BOTH siblings' constants and domain
   strings — as HARDCODED LITERALS, never crate imports.** An import would let a
   future refactor move both sides together, which is the one thing these lines
   exist to prevent. Both siblings, not just `md`, because **the mistake that
   actually gets made is against the crate you have open** — and this plan ports
   from `mk`, whose constant and domain string would satisfy the drift test if
   pasted as a pair.

### Two obligations the fork creates, which this plan had not named

R8 gates I11 — **NOT FIXED until now, and correctly reported as never
attempted** rather than folded into something adjacent.

1. **A PROVENANCE PIN.** Every ported package in this constellation records the
   crate and version/SHA it tracks, updated on every sync, so drift is
   auditable. `mt-codec`'s string layer is ported from `mk-codec`, so it carries
   the same pin: the `mk-codec` version and commit the port was taken from, in
   the module header. **Without it the port's ancestry is folklore** — a later
   reader cannot tell which `mk` a given behaviour came from, or whether a `mk`
   fix has been carried across.

2. **A THREE-WAY DEFECT CHECK.** The standing rule is that a defect found in one
   implementation triggers checking the others. With `mt` there are **three**
   BCH implementations, not two — `md`, `mk`, `mt` — and the fork is now
   **permanent** (extraction retired 2026-05-03), so this is not a temporary
   burden that a future shared crate retires. Any BCH, chunking or header defect
   found in one is checked against the other two, and the check is recorded even
   when it finds nothing.

> **These are the price of forking, and the fork is still right** — HRP-mixed
> BCH with per-format residues is unshareable, so there is no version of this
> where one implementation serves three formats. Naming the price is what makes
> it payable; leaving it unnamed is how three copies quietly diverge.

> **Why the vector and not more assertions.** Assertions restate values the
> implementation already reads, so they can be satisfied by the implementation
> agreeing with itself. A vector the implementation did not produce is the only
> kind that can falsify it — the same reasoning that makes R8 C-5 a won't-fix
> (§2a).

## 2. Phases

Each phase is **tests first**, closes on a **command that runs**, and does not
begin until the previous one is green. A phase's gate is written **in the phase
that owns it** and must be executed at least once before that phase closes — a
gate that has never run is a hypothesis, not a gate.

### S0 — the pinned vector, authored HERE, before any phase runs

**R10 Important 1: the load-bearing NUMS defence had no owning phase and does
not exist.** P0 copied *"the spec and its pinned vector"* as though the vector
were already a file; P1 asserted against it *"before any other test is
written"*; **no phase created it**. P0 cannot copy what does not exist, and
`mt-codec` cannot be checked against bytes nothing produced.

**S0 runs in `mnemonic-engrave`, not `mnemonic-transaction`, and completes
before `mt-codec`'s first commit** — that ordering is the whole point of the
ruling it comes from (`design/agent-reports/R8-fable-ruling-nums-defence.md`).
A vector the implementation produced cannot falsify the implementation.

**Deliverable — the vectors in BOTH forms, from ONE generator, plus the
generator:**

- **`design/vectors/mt1_v1_vectors.md`** — human-readable, for a reader
- **`design/vectors/mt1_v1_vectors.json`** — machine-readable, **the file a Rust
  test actually reads and the file the SHA-256 pin covers**

> **`mk`'s pattern cannot be adopted "verbatim" over Markdown — R11 I1.** `mk`
> pins `src/test_vectors/v0.1.json`, resolved from `CARGO_MANIFEST_DIR` and
> consumed by `serde_json`. An implementer handed only a `.md` has three bad
> options unattended: write a Markdown parser inside a test, hand-transcribe the
> values into Rust constants — **which breaks the pin, since the hash would
> cover a file the test does not read** — or invent a sidecar format mid-phase.
> P0 copies the `.json` to `crates/mt-codec/src/test_vectors/mt1_v1.json`,
> matching `mk`'s location shape, and P1 pins **that** file's hash.

1. **One real signed segwit transaction, PRODUCED ON A LOCAL REGTEST NODE — not
   lifted from mainnet.** R11 C3, and the provenance is the whole point:
   `createwallet` → `generatetoaddress` → `walletcreatefundedpsbt` →
   `walletprocesspsbt` → `finalizepsbt`, never broadcast.

   > **A confirmed mainnet transaction makes P2's gate fail on this machine and
   > pass in CI.** Every input of a confirmed transaction is spent, and its
   > parents are confirmed — which is exactly §8.5's refusal condition and makes
   > §6a report `SPENT — ALREADY CONFIRMED` before any input is classified. So
   > `mt encode <vector>` **refuses where a node is reachable and succeeds where
   > one is not.** P2's gate is *"reproduces the vector's strings exactly"*; it
   > would have failed at 3am with the remedy being to redo S0.
   >
   > **A regtest outpoint is unknown to a mainnet node**, so `gettxout` returns
   > null with the parent **not** confirmed → §8.5 does not fire and §6a reports
   > `UNKNOWN`. That is the only provenance under which every later gate passes
   > **both** online and offline.
   >
   > *(A mainnet candidate had already been selected — 162 bytes, past locktime,
   > txid ≠ wtxid — before this gate caught the problem. Its virtue was being
   > independently verifiable; its defect was being confirmed.)*

   Recorded for that transaction: the **raw hex**, its **txid and wtxid both**
   (so §1.1's `TX`-row distinction is pinned by bytes rather than prose), the
   **finalized PSBT in base64**, and **each input's value and outpoint**.

   > **All four forms, because different gates need different ones — R11 C3(b).**
   > The PSBT is the clean-encode input, since its UTXO records satisfy §8.2c
   > and §8.2b; the raw hex exercises §8.2e's loud-warning path; the recorded
   > values back `--input-value` where a test wants the raw form to encode
   > without a node.
2. **Its exact `mt1` strings**, in full form and in `--elide-prefix` form.
3. **A 13-symbol checksum micro-vector** — HRP `"mt"`, a fixed 40-symbol data
   part, the resulting BCH checksum — so a checksum bug is localisable without
   decoding a whole transaction.
4. **`scripts/gen-mt1-vectors.py`, committed**, deriving all of the above from
   the transaction hex **independently of `mt-codec`** — bech32, the 55-bit
   header layout and the BCH polymod implemented directly from §10.13(a2) and
   BIP-93. Slower and dumber than the crate on purpose: an independent
   derivation is the only kind that can disagree.

**Gate.** The generator runs, its output matches the committed vector file
byte-for-byte, and the header fields it emits satisfy the arithmetic of
§10.13(a2): 55 bits, 11 symbols, invariant prefix exactly 8 symbols.

> **`mk` DOES THIS THE OTHER WAY, AND `mt` TAKES BOTH.** Checked against the
> real crate rather than assumed: `mk`'s vectors are generated by
> **`gen_mk_vectors`, a bin inside `mk-codec` itself**
> (`crates/mk-codec/src/bin/gen_mk_vectors.rs`), `include_str!`-baked as
> `v0.1.json` — **produced by the implementation under test**, which is the
> closed loop R8 filed as a Critical against this plan.
>
> But `mk` carries a guard this plan did not: **`V0_1_SHA256`
> (`crates/mk-codec/tests/vectors.rs:41`) pins the file's hash**, enforced by
> `vector_file_sha256_matches_pin()`, so regenerating requires **deliberately
> re-pinning**. Drift cannot happen quietly.
>
> **The two catch different failures and neither covers the other.** A pinned
> hash catches an encoder change silently altering the vectors — and **freezes
> the wrong bytes forever if the constant was wrong when they were first
> generated**. Independent derivation catches exactly that initial wrongness,
> and says nothing about drift afterwards.
>
> **So `mt` does both.** S0 derives the vectors independently of `mt-codec`;
> once they are copied into the crate (P0), they are pinned by SHA-256 with a
> test asserting the match, `mk`'s pattern adopted verbatim. **Regenerating
> them requires re-running S0's independent generator, not the crate** — which
> is the difference that matters, since re-deriving from the implementation is
> how a wrong vector would launder itself into looking correct.
>
> *(Noted in passing: `mk`'s own doc comment names the constant
> `VECTORS_V0_1_SHA256`; it is `V0_1_SHA256`. A grep for the documented name
> finds only the comment, and I nearly filed the pin as missing.)*

**Exit.** The vector exists in this repo and is committed. P0 now has something
to copy.

### P0 — skeleton

**Deliverable.** The **spec and the S0 vector are copied into
`mnemonic-transaction`** — `design/SPEC_mt_v0_1.md` and
`design/vectors/mt1_v1_vectors.md` — because
P5's exhaustiveness gate and P6's journeys read them and **no phase put them
there** (R8 coverage I-12). They are copied with the commit SHA they came from
recorded alongside, so drift against `mnemonic-engrave` is a `git diff` and not
a guess. `mnemonic-transaction` exists with two crates, workspace lints
matching the constellation, CI running `cargo nextest run --locked`, and
`[profile.test] opt-level = 2` / `[profile.dev] opt-level = 2` (keeps
`debug_assertions` — do **not** use `--release` to speed tests).

**No upstream change.** §1 settles this: `mt-codec` forks, so `descriptor-mnemonic`
is not touched by this plan at all.

**Tests first.** Nothing to test yet — which is exactly why this phase does
**not** run the test command.

**Gate.** `cargo build --locked`, `cargo clippy --all-targets -- -D warnings`,
and `cargo fmt --check`. **`cargo nextest run` is deliberately NOT part of this
gate.**

> **From P1 onward the gate is the WHOLE validation surface, not one crate —
> R8 gates I12.** `cargo nextest run --locked` unqualified, plus clippy and fmt,
> across the workspace. A per-phase gate scoped with `-p mt-codec` reports green
> while the CLI crate is broken, and this constellation's rule is that reviews
> run against the whole surface rather than the slice a phase touched.

> **Because it would fail — measured, not reasoned about.** Since nextest
> 0.9.85 the default is `--no-tests=fail`, and on the installed **0.9.140** a
> workspace with no tests prints `error: no tests to run` and exits **4**. The
> earlier version of this gate was *"`cargo build` and an empty
> `cargo nextest run --locked` succeed"* — **a gate that could never pass**, in
> a plan whose own text says a gate that has never executed is a hypothesis
> rather than a gate. Both R8 lenses found it independently, and the fix is not
> `--no-tests=pass`: **an empty test run is a vacuous gate either way**, so the
> honest move is to gate on what P0 actually produces — a tree that builds and
> lints clean — and let P1 be the first phase with tests to run.

**Exit.** The repo exists, builds, and lints clean. No `mt1` behaviour yet.

### P1 — the `mt1` wire format

**Deliverable.** `mt-codec`'s string layer, mirroring the module set `mk-codec`
actually carries in `crates/mk-codec/src/string_layer/` — **`bch.rs`,
`bch_decode.rs`, `chunk.rs`, `header.rs`, `mod.rs`, `pipeline.rs`** — plus
`consts.rs` and `error.rs` at the crate root, where `mk` keeps them.

> **The earlier list was `md-codec`'s, mis-attributed to `mk-codec` — R8
> coverage I-1.** It named `nums.rs` and `error.rs` inside the string layer
> (neither is there) and **omitted `pipeline.rs`, the module that joins the
> pieces**. Listed here from `ls`, not from memory: describing a sibling's
> layout from recollection is the same defect class as describing code from its
> doc comment, and it is the second time in this plan that a claim about `mk`
> was actually a claim about `md`.

**Ported from `mk-codec`, which is the closest sibling** — not from `md-codec`,
and not written from scratch. `mk` already solved chunk + header + BCH for a
*second* format, so it is the one that has already made the
generalise-or-fork decision `mt` faces; `md` has only ever been first.

Normative content, all from the spec — **this plan restates none of it as new
decisions**:

- **HRP = `"mt"`, not `"mt1"`** — the `1` is bech32's separator (§10.13 b). Stated because it is one of the values §10.13(a2) flags as a guess-hazard, and it feeds `hrp_expand` on both the create and verify sides
- header **55 bits = 11 symbols, every field a whole number of symbols** (§10.13 a2):
  `version(5) + chunk_set_id(20) + count−1(15) + index(15)`.
  **There is no `chunked` bit** — it was deleted as dead, and it is what used to break alignment
- `version = 0b00001`; **`count` stores `count − 1`** (a set of 1 stores `0`), `index` is plain and zero-based
- **no bit packer**: every field is a whole number of 5-bit symbols, so the header is built by pushing symbols
- `MT_REGULAR_CONST = 0x1a2fc877f9528d7c1`, from `"shibbolethnumstransaction"` (§12.22)
- `count = ceil(payload_len / 40)`; `bytes_per_chunk = ceil(payload_len / count)`; last chunk takes the remainder (§3b)
- content id = **top 20 bits of the txid in display form** (§10.13 c)
- BCH(93,80,8), `t = 4` per chunk (§3a)

> **This block said "49 bits, `version(4) + chunked(1) + … count−1(12) +
> index(12)`" until 2026-08-23** — the layout two wire-format rulings had
> already replaced. The spec was updated and the plan was not, which is the
> propagation failure this cycle keeps producing: **an implementer following the
> plan would have built a header no conforming decoder could read.**

**Tests first. The pinned vector is the phase's whole point; the rest are
tripwires:**

1. **The pinned byte-exact vector** (§1) — asserted against **before** any other
   test is written, because it is the only artifact here `mt-codec` did not
   produce. Includes at least one **real signed segwit transaction**, so the
   witness-bearing serialisation is exercised rather than a synthetic byte
   string.
2. **The drift test** (§1) — `SHA-256("shibbolethnumstransaction")` top 65 bits
   equals `MT_REGULAR_CONST`.
3. **Four `assert_ne!` lines** (§1) against both siblings' constants and domain
   strings, **hardcoded, no imports**.
4. **A correction test that CORRECTS** — mutate 1, 2, 3 and 4 symbols of a valid
   chunk and assert each is repaired to the original bytes, then mutate 5 and
   assert it is not silently accepted. R8 gates C5: the round-trip gate uses
   *clean* vectors, so the residue is zero and **the correction path is never
   entered** — nothing else in this plan proves the format's whole purpose works.
5. **Chunking property test that distinguishes BALANCED from FILLED** — R8
   gates I2. For random payload lengths, assert **every chunk but the last is
   exactly `ceil(len/count)` bytes** and the last is the remainder, not merely
   that no chunk exceeds 40. A *filling* chunker also round-trips and also stays
   under 40, so the previous test passed on the exact divergence §3b was written
   to close: a filler and a balancer produce different chunk boundaries, and
   §1.1e's mandatory length check then reads the other implementation's strings
   as damaged steel.

**Gate.** `cargo nextest run --locked -p mt-codec` green, **including the pinned
vector and the within-budget correction test**.

**Exit.** A transaction encodes to `mt1` strings and back, byte-exact, **and
matches the spec's pinned vector**.

> **Test vectors are produced here and are the artifact P2–P4 are checked
> against.** Per the spec's own lesson that a corpus can be uniformly wrong,
> each vector records the transaction bytes, the expected strings, and the
> content id — and at least one vector is a **real signed segwit transaction**,
> not a synthetic byte string, so the witness-bearing serialisation is exercised.

### P2 — `mt encode`

**Deliverable.** Input handling and string output.

- the ordered sniffing procedure — binary PSBT before whitespace removal, then base64, then hex (§8.2e)
- **the twelve ruled flags of §10.10**, spellings included — `--in`, `--from`,
  `--to`, `--to-label`, `--input-value <index>:<amount>` (repeatable),
  `--group-size`, `--separator`, `--elide-prefix`, `--quiet`, `--transaction`,
  `--json`, `--bitcoin-cli`
- **node access by SHELLING OUT to `bitcoin-cli -stdin`**, arguments on stdin
  and never on the command line (§10.10 b1) — a txid on the command line lands
  in `ps`, the leak §8.2f refuses for transactions. No JSON-RPC client, no
  cookie handling, no `--rpc`
- **an input path for every operator-supplied value §10.10 requires** — R8
  coverage I-5. Per-input values (§8.2c), the `FROM`/`TO` identities, the
  free-text `TO` label behind its own flag (§10.4). **Not the node location** —
  `bitcoin-cli` holds it (§10.10 b1).
  **A refusal whose threshold cannot be supplied is not a refusal.**
  **Spellings are ruled above**; the input *paths* are P2's to build (R11 I5 —
  this bullet said spellings were still open, three bullets below the list that
  rules them)
- normalisation to **lowercase**; stdout is lowercase, ungrouped (§1.1e, §10.10)
- **1-based chunk numbers and 1-based positions in ALL human-facing output**,
  with `position = codeword_index + 4` (§1.1) — R8 coverage I-11. Wire `index`
  is 0-based and appears nowhere in output. This is the rule whose violation
  sends an operator to re-cut the wrong string
- **exit code 0 means every check passed**, non-zero otherwise (§10.10) — the
  documented `decode` pipeline depends on it
- optional grouping, opt-in, **stdout only**
- **`--elide-prefix`** — first string full, rest carry `index + payload` only (§3b)
- **§10.20's recovery caveat**, carried by `inspect` and `decode`: a **legacy**
  input is txid-malleable, so if a malleated version confirmed first the
  confirmed txid will not match the plate's — **the plate is not wrong, it is
  superseded**, and the original can no longer confirm. R10 Minor 1, and an
  uncarried R8 finding before that: §10.20 says this belongs *"somewhere a
  recoverer will read"*, and no phase had it
- the `stderr` legend suggestion — **five** fields including
  `FORMAT: mt1 codex32` (§5). *(Six until `PLATE n OF m` was deleted on
  2026-08-23; this line said six, another spec change the plan did not inherit.)*
- **the TTY WELCOME LINE — R11 I6, and it is the item the operator's own
  confusion produced.** `mt encode` with nothing piped in blocks on stdin with
  no prompt, so a new user's first action looks like a hang. §10.10 names the
  cost: *"a new user concluding the tool does not work and leaving, which no
  other check catches."* Reported PARTIAL twice and closed by neither fold.
  P2 prints the welcome line and a newline when stdin is a TTY
- **§6a's no-node warning at ENCODE time** — the enumerated skipped checks, and
  the encode-shaped consequence line (*"consider re-running with a node before
  you start"*), distinct from the recovery-time wording §1.1a carries. R11 I6;
  also PARTIAL twice
- **the three mandatory `stderr` blocks, which no phase owned — R8 coverage
  I-3.** All three are ruled and all three are what the operator actually reads:
  the **BEARER warning** carrying both halves (what `mt` checked, that it reads
  witness *shape* rather than script, and that an exotic input can defeat it —
  §5); the **"what correction does and does not cover"** block, printed
  **always, before cutting** (§1.1e); and the **"verify the STEEL, not this
  output"** instruction (§1.1), without which an operator verifies the file
  `mt` just produced and tests nothing that can fail
- the `CUT` and `PREFIX` rows appended to the report (§1.1)

**Tests first.** One test per sniffing branch **including the failure branch**,
each asserting the *message* names what was seen; a test that a hex-encoded PSBT
is refused with the message naming the real problem; a test that grouping never
reaches a non-stdout consumer.

**Deliverable, and P5 and P6 both consume it: the FIXTURE CORPUS.** R8 coverage
I-8 — no phase created it. `tests/fixtures/` holds the malformed and awkward
inputs §8.2e's procedure and §8's refusals are tested against: a binary PSBT, a
base64 PSBT, line-wrapped base64, CRLF and trailing-newline variants, uppercase
hex, a `0x`-prefixed hex, a hex-encoded PSBT, a raw signed transaction, and one
PSBT per refusal that must fire. Built here because P2 is the first phase that
reads input at all.

**Gate.** `mt encode` on each P1 vector reproduces the vector's strings exactly,
**and every fixture is either accepted or refused with the ruled message**.

**`--bitcoin-cli /nonexistent` IS THE OFFLINE MECHANISM**, and P2 asserts it
produces §6a's no-node warning rather than a crash.

> **Two later gates require an offline run and there was no way to force one —
> R11 C3(c).** P4's gate must run *"BOTH with node fixtures and offline"* and
> journey B is *"no node"*, yet none of the twelve flags is `--offline`. The only
> lever is §10.10(b1)'s rule that a `--bitcoin-cli` path which is absent or not
> found yields the no-node warning. Naming it here matters because the
> alternative an implementer reaches for is **editing `PATH`**, which is
> process-global and silently changes the behaviour of neighbouring tests in the
> same run.

> **P2's gate as first written would have started failing once P5 landed — R8
> coverage I-13.** P1's vectors are transactions, and they carry **no input
> values**; once §8.2c's warning and §8.2b's balance refusal exist, encoding a
> bare vector emits warnings the gate did not expect, and a refusal-bearing
> vector fails outright. The vectors therefore come in two kinds from P1
> onward: **clean** ones that must encode silently, and **refusal fixtures**
> that must be rejected by name. A gate that passes in one phase and breaks in a
> later one is a gate nobody will trust by the time it matters.

### P3 — `decode` and `verify`

**Deliverable.** The reading path, which is where the recovery journeys live.

- **prefix restoration before anything else** — a line not beginning `mt1` is elided and is prefixed from the set's full string; full, elided and **mixed** input all accepted (§3b)
- splitting **then** stripping (§1.1e) — including the single-line pasted blob
- length check from the **modal** string length (§1.1e)
- autocorrect: try-as-written first, positional, never touching a string that parses (§1.1e)
- correction reporting: positions **1-based**, `position = codeword_index + 4`, with before-values (§1.1)
- duplicate resolution over **`n`** candidates, post-correction bytes, majority vote forbidden (§1.1)
- `decode` writes **nothing to stdout** unless every check passes, exits non-zero otherwise (§1.1a)

**`mt verify` IS A DELIVERABLE OF THIS PHASE, and had none — R8 gates C4.** The
plan named it in the verb list and in this heading and nowhere else: every bullet
above is a `decode`-path item, and the gate tested `decode` only. A verb with no
deliverable, no test and no gate is a verb nobody has agreed to build.

- **`verify` is STRUCTURAL ONLY and NEVER asks a node** (§1.1) — it must run on
  an air-gapped machine, so a node call here is a defect, not a feature
- it checks: every string parses, every BCH checksum holds, the set is complete
  (chunks 1..`count`), every chunk carries the same `chunk_set_id`, and the
  reassembled transaction **re-derives that id**
- it **reports its margin, not just its verdict** (§1.1) — per corrected chunk,
  the count against `t = 4`, the 1-based positions, and each symbol's
  **before-value**, ordered so the nearest-to-limit chunk is visible

  > **This was assigned to no phase — R8 coverage I-2 — and it is the Critical
  > the journey walk found.** A plate miscut in four places passes `verify` as
  > OK while sitting **one scratch from unrecoverable**, with zero redundancy
  > behind it. A verdict that hides how much of its budget it just spent tells
  > the operator the opposite of what they need. The **before-value** is the
  > load-bearing part: `pos 29 read v corrected to d` is settled against the
  > steel in seconds — if position 29 reads `d` they mistyped, if it reads `v`
  > they miscut — and a report giving only counts and positions leaves them
  > nothing to compare.

- **the re-derivation FAILURE path** (§1.1): when every checksum holds and the
  transaction still does not re-derive its id, report the ranked suspect list in
  **descending correction order**, and state that the check identifies the
  transaction rather than proving every byte
- **`--transaction <psbt|hex>`** compares against the **full 32-byte txid** of the
  supplied transaction's *extracted* form (§1.1), never the 20-bit set id

**Tests first.** A round-trip through `--elide-prefix` (encode elided → decode →
byte-identical transaction), a **mixed** full/elided input test, and a test that
**all-elided input is refused** with the message naming the 8 characters needed —
elision is a display form over a checksum that covers the full data, so a
restoration bug shows up as a checksum failure and must not be mistaken for one.
A test that a >4-error chunk is *not* silently accepted; a test
that the `mt1`→`mtl` autocorrect hazard does not fire on a valid string; a
duplicate-resolution test with **three** candidates asserting refusal rather than
a vote; a test asserting stdout is empty on failure.

**Gate, and it now covers both verbs.**

1. Every P1 vector round-trips through `decode`.
2. **`verify` returns OK on every P1 vector**, and on a vector with **one**
   corrupted symbol returns OK **while reporting `1 of 4`** — the margin report
   is the deliverable, so a `verify` that says OK without it fails this gate.
3. **`verify --transaction` matches the right transaction and REJECTS a wrong
   one whose txid shares the set's 20 bits.** Constructing that input is cheap —
   2^20 double-SHA-256 operations — and it is the only test that distinguishes
   the full-txid comparison from the 20-bit one.
4. **Negative:** a vector corrupted beyond `t = 4` must fail, with the failure
   naming the suspect chunks in descending correction order.

> Gate 3 is the one that would have caught the defect R8 found in the spec: a
> comparison against the set id reports a **match** for any transaction sharing
> 20 bits, and says so in the words *"prove identity"*.

### P4 — `inspect`, the report, and the node

**Deliverable.** The single report layout (§1.1), its three callers, and the
chain queries.

- `SPENT — ALREADY CONFIRMED` checked **first**, before any input is classified
- liveness: LIVE / DEAD / PENDING / UNKNOWN, `DEAD` requires `confirmations ≥ 1`
- `FEE` carries the weakest provenance of any input inline
- offline: every row `UNKNOWN` rather than omitted, with the resolution line naming both a node and a block explorer

**Tests first.** Node responses are **fixtures, not a live node**, so the tests
run air-gapped and deterministically. One fixture per liveness state, including
the mempool-unconfirmed parent that must read PENDING and not DEAD.

**Gate, run BOTH with node fixtures and offline.** For one vector, the three
callers' reports agree on **every row the §1.1 row-presence table says all three
can produce**, and differ **only** where that table predicts — each difference
asserted individually, not waved past.

> **Offline-only would pass vacuously — R8 gates I5.** With no node every
> chain-derived row reads `UNKNOWN` for all three callers, so they agree
> trivially and the gate proves nothing about the rows that matter. The
> node-fixture run is where `FEE`, `STATUS` and the input provenance actually
> differ, and therefore where the row table is actually tested.

> **"Identically" was unsatisfiable, and §1.1's own table says why — R8
> coverage C-4.** `FEE` is present *"when a node is reachable **or** the input
> was a PSBT carrying values"*: `encode` is handed a PSBT with UTXO records and
> can compute it, while `inspect` is handed `mt1` strings and offline cannot.
> The same table makes `mt1 SET` absent for `encode` (it has no strings to
> report on yet) and present for the other two. **A gate demanding identity
> would fail on conformant code**, which is the worst kind: it trains whoever
> hits it to weaken the gate rather than fix the defect.
>
> **Asserting the differences is the stronger test anyway.** "Identical" checks
> that three callers agree; this checks that they differ **exactly where the
> specification says they must** — which is the property the single-owner rule
> exists to protect, and it catches a caller that drops a row it *should* have
> produced. Identity never would.

> **A live-node smoke test is a separate, non-gating check, run ONCE at the end
> of P4** — R10 Minor 2 gave it a moment, since "worth doing" with no scheduled
> point is a thing nobody does. A synced `bitcoind` is available on this
> machine; the run compares `inspect`'s liveness verdict for one confirmed and
> one unconfirmed outpoint against the fixtures P4 was built on. **It must not
> gate CI, which has no node** — a fixture that has drifted from the real RPC is
> exactly what a fixture-only gate cannot see, and the point of running it once
> is to find that.

### P5 — refusals

**Deliverable.** §8 in full, each refusal naming the number that caused it.

**Tests first — one test per refusal in `tests/refusals.toml`, and each must be
shown to FAIL when the refusal is removed.**

> **That discipline was prose with no executable form — R8 gates I6.** "Must be
> shown to fail" is a thing a person does once and nobody re-runs. P5 commits
> `scripts/mutate-refusals.sh`, which for each entry comments out the named
> check, runs **only that refusal's test**, and asserts it goes **red** —
> restoring the source afterwards. A refusal test that passes against code with
> the check deleted is testing nothing, and this constellation has paid for that
> lesson twice.
>
> **The script must fail loudly if a mutation does not apply.** A `sed` that
> matches nothing leaves the code intact, the test passes, and the run reports
> success — a *vacuous* control, which has already happened twice in this cycle
> alone. Each mutation asserts it changed the file before the test runs.

**Not implemented:** §8.7 and §8.7c — moved to the deferred QR spec, unreachable
in v0.1.

**Gate.** Every refusal on the **explicit list below** has a test, and a
committed script asserts the union is exhaustive against that list — so a
refusal cannot be added and silently go untested.

> **The exhaustiveness gate had no well-defined input, which is three findings
> with one cause — R8 coverage I-6, I-7 and gates I7.** "Every numbered refusal
> in §8" is not a set a script can compute:
>
> - **§8's numbering contains non-refusals.** §8.2 (script validity) and §8.8
>   are numbered items that are *not* v0.1 refusals; §8.7 and §8.7c are now
>   pointers to deferred material. A script counting `^\d+[a-z]?\. ` in §8 would
>   demand tests for four things that cannot fire.
> - **A real refusal lives OUTSIDE §8's numbering.** §6a's value-mismatch
>   refusal is normative and is not a numbered §8 item, so a §8-numbering script
>   is **structurally unable to see it** — the one class an exhaustiveness gate
>   exists to prevent.
> - **The spec is in a different repository** (`mnemonic-engrave/design/`) from
>   the crate under test (`mnemonic-transaction`), and no phase put it there —
>   so the script had no file to read either. R8 coverage I-12.
>
> **The fix is a list, not a parser.** P5 commits
> `tests/refusals.toml` in `mnemonic-transaction`, **three fields per entry**,
> and the script asserts a **bijection between that file and the tests that
> exist**:
>
>     [[refusal]]
>     spec  = "§8.1"                      # where it is ruled
>     test  = "refuses_unfinalized_psbt"  # the test that proves it fires
>     check = "src/validate.rs::finalized_guard"   # the function to mutate
>
> **The third field exists because `mutate-refusals.sh` has to LOCATE the check
> — R10 Important 3.** A two-field schema names the rule and the test and gives
> the mutation script nothing to point at, so the implementer would invent the
> locating mechanism: grep for a message string, match a test name to a
> function, or edit by line number. All three break silently on a refactor, and
> the last one mutates whatever has moved into that line. **`check` is a
> `path::symbol` an implementer can resolve exactly**, and it is what makes the
> mutation gate's own "assert the mutation applied" step checkable. A parser over prose
> in another repo is a gate that breaks on a heading edit; a checked-in list
> breaks only when someone adds a refusal and forgets its entry, which is
> exactly the failure to catch.
>
> **Adding a refusal to the spec therefore requires touching this file**, and
> that is the point — the coupling is the mechanism, not an inconvenience.

### P6 — journeys

**Deliverable.** Three journeys as executable acceptance runs. **They are named
here because they were named nowhere — R8 gates I13** ("§ the spec's own" was a
placeholder, and no section enumerates them):

| journey | the moment | what it asserts |
| --- | --- | --- |
| **A — encode** | operator pastes a finalized PSBT and cuts | the three mandatory `stderr` blocks appear, stdout is lowercase and ungrouped, the report matches §1.1 |
| **B — recover** | 2040, `mt1` strings and **no legend**, no node | every row reads `UNKNOWN` rather than being omitted, the read-vs-verified split is visible, and the resolution line names **both** a node and a block explorer |
| **C — miscut** | operator re-cuts one string, drawer holds both | duplicate resolution announces which candidate it discarded; the margin report gives positions **and before-values** |

**Journey B runs with `--elide-prefix` output as well**, since that is the form a
hand engraver most plausibly leaves behind.

**Gate.** Each journey runs end to end as a script and asserts on **what the
operator sees**, not only on exit codes.

---

## 2a. Rulings on review findings — what is NOT being done, and why

**R8 C-5 — bespoke tests for a wrong HRP, a wrong `version`, or a plain
`count`: WON'T FIX.** Operator ruling 2026-08-23.

**The operator's reasoning, on the engraving side.** *"If a user can't get the
first two chars correct for engraving, everything else we could possibly do to
help will fail. An incorrect count or plate id doesn't necessarily preclude
recovering funds if all cards are present — it just means ignoring the headers
and finding the correct order, most if not all of which will be correctly
recorded."*

That holds, and one mechanism makes it hold harder than stated: **the header is
inside the BCH-protected region**, so a miscut `index` is corrected exactly like
a payload symbol, under the same `t = 4` budget. Past that budget a recoverer
holding every string can disregard the headers entirely and search orderings,
with the **content id validating the result** (§10.13 c). Header fields are a
recovery *convenience*; the content id is what makes recovery decidable.

**And the case the finding was actually aimed at — an IMPLEMENTER choosing
`"mt1"` as the HRP, or plain `count` — is already covered**, which is what makes
this a genuine won't-fix rather than an accepted risk. Each wrong choice
produces **different bytes**, so the pinned byte-exact vector below fails on all
three by construction. C-5 asked for three bespoke tests to catch what one
vector catches for free, and three tests that restate the same values the
implementation reads are three more places for the values to drift together.

> **The general rule this is an instance of:** prefer **one artifact the
> implementation did not produce** over N assertions the implementation could
> satisfy by agreeing with itself. That is also why the vector is spec-authored
> and lands first.

## 3. What this plan deliberately does not do

- **No `mt qr`, no `sysw`, no QR anything.** Deferred (§0a).
- **No script evaluation.** There is no script engine in v0.1 (§8.2), so §8.6
  recognises signatures **by shape** and the spec's own warning text says so.
- **No transaction construction.** Out of scope by ruling.
- **No redundancy/fountain coding.** Redundancy is zero by ruling (§12.6); the
  operator's mitigation is cutting duplicate copies, which P3's duplicate
  resolution supports.

---

## 4. Open questions this plan carries

1. ~~The `md-codec` release process.~~ **CLOSED by §1** — `mt-codec` forks, so
   `md-codec` is not modified and there is no version bump, no publish decision
   and no upstream coordination. This question existed only because an earlier
   draft of this plan contradicted the `mk` precedent.
2. **§10.14 says its correction must land BEFORE implementation, and this plan
   said it does not block P0–P2 — R8 gates I14.** Reconciled: §10.14 is the
   legend budget resting on a doc comment rather than the fork's font metrics.
   It binds **§5's plate-area material, which is deferred with `mt qr`**, and
   `mt encode` reserves no area at all — so it does not block the live verbs.
   **Stated rather than assumed**, because "does not block" was an assertion
   with no reasoning attached.
3. **Where the remaining spec open questions land.** §10 holds §10.10, §10.13,
   §10.14 and §10.20.

   > **This entry called §10.10 "flag *spellings* only" and that was false —
   > R8 coverage I-10.** Two more things are open there, and one of them is a
   > dependency of a later phase:
   >
   > - **the refusal-message format** — §8 promises every refusal will *"name
   >   the number that caused it"*, and **P5 tests exactly that**. A test
   >   asserting on a message format nobody has specified is a test written
   >   against a guess.
   > - **exit codes beyond 0** — `0 = every check passed` is now fixed
   >   (§10.10 b), and the rest of the code space is not.
   >
   > **§10.13 is the header, NUMS constant and content id — the entire normative
   > content of P1** (R8 gates I9). It is marked RULED rather than open, so
   > nothing blocks; the entry is kept because a reader seeing "P1's content
   > comes from an open question" deserves the resolution rather than the
   > appearance of one.
   >
   > **Blocking order:** ~~§10.10's spellings~~ **CLOSED 2026-08-23** — all
   > twelve flags are ruled in §10.10, and `--rpc` was **deleted** in favour of
   > shelling out to `bitcoin-cli -stdin`, which already holds the node's
   > location. ~~The refusal-message format~~ **CLOSED 2026-08-23** — §8's
   > preamble rules the three-part format, with a machine-parseable verdict line
   > (`<verb>: REFUSED — §<ref>, <reason with the number>`) that P5's tests
   > assert against without matching prose. **No spec-side item now gates any
   > phase.**

4. ~~Repo creation~~ **CLOSED 2026-08-23** — `bg002h/mnemonic-transaction`
   exists, is **EMPTY** and **PRIVATE**. P0 initialises and pushes to it.

   > **v0.1 publishes nothing, tags nothing, releases nothing, and makes nothing
   > public.** `mt-codec`, `mt-cli` and `mnemonic-transaction` are all free on
   > crates.io (checked 2026-08-23) and stay unclaimed until the operator says
   > otherwise. The repo staying private is deliberate: all five siblings are
   > public, so it will likely be flipped at release — but private→public is one
   > command and the safe direction, and public→private does not un-announce.
