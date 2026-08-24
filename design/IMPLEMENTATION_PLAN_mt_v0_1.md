# IMPLEMENTATION PLAN — `mt` v0.1

> **Status: DRAFT, pre-R0.** No code is written until an architect review closes
> this at 0 Critical / 0 Important. Risk-set work: funds, addresses, a new
> normative wire format.
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
| extraction into a shared crate | **deliberately deferred**, trigger recorded as *"both md-codec and mk-codec at v1.0"* (closure Q-9) |

So the established pattern is **fork per codec, carry your own constant, defer
extraction until the formats have stopped moving.** `mt` is the third format to
face this and has no reason to be the exception — a new codec is precisely when
a shared crate is most likely to be shaped around assumptions that later break.

**This deletes a whole phase of upstream work.** No change to `md-codec`, no
semver question, no publish decision, and the Go port is untouched.

### How the copy-paste hazard is defended

§12.22 names it: *"copy `md-codec`, change the HRP, leave the constant, and
`mt1` chunks verify as `md1` chunks."* **Two tests, and they catch different
failures — one is not a substitute for the other.**

1. **A drift test, copying the sibling pattern verbatim.** Both siblings already
   have one, and `md-codec`'s cites `mk`'s as its model:
   `mk-codec`'s `consts::tests::nums_constants_reproduce_from_domain`, and
   `md-codec`'s at `crates/md-codec/src/bch.rs:122`, asserting
   *"MD_REGULAR_CONST drift from SHA-256(\"shibbolethnums\") top-65-bits"*.

   `mt`'s is the same shape: `SHA-256("shibbolethnumstransaction")`, top 65
   bits, asserted equal to `MT_REGULAR_CONST`. **Independently recomputed before
   this plan was written** — `0x1a2fc877f9528d7c1`, matching the spec; and the
   rule reproduces `md`'s `0x0815c07747a3392e7` from `"shibbolethnums"`, so it
   is the derivation that is confirmed and not one lucky value.

   **What it catches:** the constant and its domain string drifting apart.
   **What it does NOT catch:** a constant copied wholesale from a sibling —
   `MD_REGULAR_CONST` pasted into `mt-codec` with the domain string pasted too
   would satisfy it.

2. **A cross-format negative, which NEITHER sibling has.** An `mt1` chunk must
   **fail** `md1` verification, and an `md1` chunk must fail `mt1` verification.
   This is the §12.22 hazard stated as an executable test, and it is the only
   one of the two that fails when a copied module keeps a copied constant.

> **Both, not either.** The drift test guards the constant against its own
> definition; the cross-format negative guards `mt1` against the neighbouring
> format. A fork defended by only the first is defended against the failure that
> is easy to imagine and not against the one the spec actually warns about.

## 2. Phases

Each phase is **tests first**, closes on a **command that runs**, and does not
begin until the previous one is green. A phase's gate is written **in the phase
that owns it** and must be executed at least once before that phase closes — a
gate that has never run is a hypothesis, not a gate.

### P0 — skeleton

**Deliverable.** `mnemonic-transaction` exists with two crates, workspace lints
matching the constellation, CI running `cargo nextest run --locked`, and
`[profile.test] opt-level = 2` / `[profile.dev] opt-level = 2` (keeps
`debug_assertions` — do **not** use `--release` to speed tests).

**No upstream change.** §1 settles this: `mt-codec` forks, so `descriptor-mnemonic`
is not touched by this plan at all.

**Tests first.** Nothing to test yet beyond the skeleton building.

**Gate.** `cargo build` and an empty `cargo nextest run --locked` succeed.

**Exit.** The repo exists and is green. No `mt1` behaviour yet.

### P1 — the `mt1` wire format

**Deliverable.** `mt-codec`: `header.rs`, `chunk.rs`, `nums.rs`, `bch.rs`,
`bch_decode.rs`, `error.rs` — the module set `mk-codec` carries in
`crates/mk-codec/src/string_layer/`.

**Ported from `mk-codec`, which is the closest sibling** — not from `md-codec`,
and not written from scratch. `mk` already solved chunk + header + BCH for a
*second* format, so it is the one that has already made the
generalise-or-fork decision `mt` faces; `md` has only ever been first.

Normative content, all from the spec — **this plan restates none of it as new
decisions**:

- header 49 bits: `version(4) + chunked(1) + chunk_set_id(20) + count−1(12) + index(12)` (§10.13 a2)
- `MT_REGULAR_CONST = 0x1a2fc877f9528d7c1`, from `"shibbolethnumstransaction"` (§12.22)
- `count = ceil(payload_len / 40)`; `bytes_per_chunk = ceil(payload_len / count)`; last chunk takes the remainder (§3b)
- content id = **top 20 bits of the txid in display form** (§10.13 c)
- BCH(93,80,8), `t = 4` per chunk (§3a)

**Tests first, and two of them are the phase's whole point:**

1. **The drift test** (§1) — `SHA-256("shibbolethnumstransaction")` top 65 bits
   equals `MT_REGULAR_CONST`. Same shape as both siblings'.
2. **The cross-format negative** (§1) — an `mt1` chunk fails `md1` verification
   and vice versa. **Neither sibling has this**, and it is the only one of the
   two that catches a constant copied together with its domain string.
3. Chunking property test: for random payload lengths, `count`, `bytes_per_chunk`
   and the reassembled payload round-trip, and no chunk exceeds 40 bytes.

**Gate.** `cargo nextest run --locked -p mt-codec` green, **including the
cross-format negative**.

**Exit.** A transaction encodes to `mt1` strings and back, byte-exact.

> **Test vectors are produced here and are the artifact P2–P4 are checked
> against.** Per the spec's own lesson that a corpus can be uniformly wrong,
> each vector records the transaction bytes, the expected strings, and the
> content id — and at least one vector is a **real signed segwit transaction**,
> not a synthetic byte string, so the witness-bearing serialisation is exercised.

### P2 — `mt encode`

**Deliverable.** Input handling and string output.

- the ordered sniffing procedure — binary PSBT before whitespace removal, then base64, then hex (§8.2e)
- normalisation to **lowercase**; stdout is lowercase, ungrouped (§1.1e, §12.10)
- optional grouping, opt-in, **stdout only**
- the `stderr` legend suggestion, six fields including `FORMAT: mt1 codex32` (§5)
- the `CUT` and `PREFIX` rows appended to the report (§1.1)

**Tests first.** One test per sniffing branch **including the failure branch**,
each asserting the *message* names what was seen; a test that a hex-encoded PSBT
is refused with the message naming the real problem; a test that grouping never
reaches a non-stdout consumer.

**Gate.** `mt encode` on each P1 vector reproduces the vector's strings exactly.

### P3 — `decode` and `verify`

**Deliverable.** The reading path, which is where the recovery journeys live.

- splitting **then** stripping (§1.1e) — including the single-line pasted blob
- length check from the **modal** string length (§1.1e)
- autocorrect: try-as-written first, positional, never touching a string that parses (§1.1e)
- correction reporting: positions **1-based**, `position = codeword_index + 4`, with before-values (§1.1)
- duplicate resolution over **`n`** candidates, post-correction bytes, majority vote forbidden (§1.1)
- `decode` writes **nothing to stdout** unless every check passes, exits non-zero otherwise (§1.1a)

**Tests first.** A test that a >4-error chunk is *not* silently accepted; a test
that the `mt1`→`mtl` autocorrect hazard does not fire on a valid string; a
duplicate-resolution test with **three** candidates asserting refusal rather than
a vote; a test asserting stdout is empty on failure.

**Gate.** Every P1 vector round-trips through `decode`. **And a negative gate:**
a deliberately corrupted vector must fail, with the failure naming the suspect
chunks in descending correction order.

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

**Gate.** The report renders identically from `encode`, `decode` and `inspect`
for one vector, differing only in stream and the `CUT`/`PREFIX` suffix.

> **A live-node smoke test is a separate, non-gating check.** A synced
> `bitcoind` is available on this machine, and one manual run against it is
> worth doing — but it must not gate CI, which has no node.

### P5 — refusals

**Deliverable.** §8 in full, each refusal naming the number that caused it.

**Tests first — one test per numbered refusal, and each must be shown to FAIL
when the refusal is removed.** A refusal test that passes against code with the
check deleted is testing nothing; this is the mutation discipline this
constellation has already paid for twice.

**Not implemented:** §8.7 and §8.7c — moved to the deferred QR spec, unreachable
in v0.1.

**Gate.** Every numbered refusal in §8 has a test, and a script asserts the
**union is exhaustive** against the spec's numbering — so a refusal cannot be
added to the spec and silently go untested.

### P6 — journeys

**Deliverable.** The three walked journeys (§ the spec's own), as executable
acceptance runs: the operator encoding a finalized PSBT, the 2040 recoverer with
strings and nothing else, and the operator who miscuts and re-cuts.

**Gate.** Each journey runs end to end as a script and asserts on **what the
operator sees**, not only on exit codes.

---

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
2. **Where the four remaining spec open questions land.** §10 holds §10.10
   (CLI surface — flag *spellings* only), §10.13, §10.14, §10.20. None blocks
   P0–P2. **§10.10's spellings must close before P2 ships**, since P2 builds the
   CLI.
3. **Repo creation** — `mnemonic-transaction` does not exist yet. Creating a
   GitHub repo is an outward-facing action and needs the operator's go-ahead,
   including whether it starts private.
