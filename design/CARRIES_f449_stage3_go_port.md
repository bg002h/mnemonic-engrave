# Carries for F-449 stage 3 — the Go port

**Why this file exists.** Stage 1b is *deciding* the semantics stage 3 must
reproduce, and those decisions are currently scattered across an SDD ledger
(deleted when the branch finishes), nine review reports, and commit messages.
A carry that lives only in a ledger dies with the workspace — that already
happened once this cycle with the version-bump ruling, which had to be moved
into a plan to survive.

Plans decay and are written late, deliberately (see the 2026-08-27 directive:
"completion of a prior phase implementation may make plan stale"). **Constraints
do not decay and are written early.** This is the early half.

The Go port binds **semantics**, not line-for-line code — the fork omits
`rust-miniscript` for TinyGo, so a behaviour-faithful reimplementation is
compliant. Every item below is a semantic the port must match.

---

## C-1 — The kind bit's POLARITY is 1 = Liana, 0 = NUMS, and it is pinned

Not arbitrary and not inferable from a round trip. Measured: inverting the bit
on **both** the write and read sides leaves the entire Rust suite green — a
symmetric inversion is invisible to round-trip testing by construction.

**If Go and Rust disagree on polarity, a plate written by one is silently
misread by the other as the wrong wallet kind, and therefore a wrong address.**

Rust pins it with a test that reads the raw bit off the writer's bytes via
`BitReader`, never through `read_node`, plus a golden 2-byte hex `[0x07, 0x00]`
for `LianaUnspendable` at version 8. **The port needs its own equivalent** —
a round-trip test in Go will not catch this.

Rust: `crates/md-codec/tests/wire_version_8.rs`,
`the_kind_bit_polarity_is_pinned_on_the_wire_not_just_round_tripped`.

## C-2 — Wire version 8, and it must be EVEN

The single-payload auto-dispatch reads **bit 0 of the first symbol** as the
chunked flag, before any header is parsed. For a single payload that bit *is*
`v0`. Usable set is `{4, 8, 12}`. Version 5 routes every single-string plate
into the chunk reassembler, which reports `WireVersionMismatch{got:2}` — the
error reserved for a pre-redesign card. Caught twice, independently.

Go's dispatch: `md/chunk.go:193`, `md/md.go:1235`.

## C-3 — The wire shape at version 8

```
Tag::Tr | is_nums(1) | [kind(1) iff is_nums] | [key_index(kiw) iff !is_nums] | has_tree(1) | [tree]
```
At version 4 the stream is **byte-identical to what shipped** — no kind bit is
written or read. Rust proves this over all 65 vendored vectors. All twelve
combinations of (v4, v8) x (Slot, NumsPoint, Liana) x (tree, no tree) were
verified to write and read identical bit counts.

## C-4 — §2's derivation: wire order, NOT sorted, NOT deduplicated

sha256 over each leaf key's 33-byte compressed pubkey, in descriptor
left-to-right (wire) order, **per key OCCURRENCE**. Sorting or deduplicating
produces the abandoned bitcoin/bips PR #1746 recipe — **a different wallet**.

Two traps for a port specifically:
- the device's address builder sorts a `sortedmulti_a` leaf's keys by *derived*
  key; §2 hashes *account-level* pubkeys in wire order. Feeding the sorted list
  into the hash is the single most likely port error, and §6 refuses
  `sortedmulti_a` at kind 1 partly as a belt against it;
- dedup is unpinnable from the vendored corpus (no case carries a duplicate
  leaf, because md1 refuses both reuse forms). Rust pins it with a **synthetic**
  `[a, a]` vs `[a]` slice. The port needs the same synthetic case.

Three independent parties have reproduced all eight golden xpubs byte-for-byte.

## C-5 — Identity hashes take the version DERIVED FROM THE TREE, never a constant

Go mirrors Rust 3-for-3 with an equally version-less `writeNode`:

| Go site | computes |
| --- | --- |
| `md/encode.go:417` | the wire payload |
| `md/template_id.go:53` | `WalletDescriptorTemplateId` |
| `md/walletpolicyid.go:42` | `WalletPolicyId` |

A constant v4 makes kind 0 and kind 1 hash identically while their addresses
differ — **two different wallets sharing one 12-word identity phrase, one
`Policy id:` line, and one mk1 card stub**. `md/template_id.go:112`
`FormAwareStub` routes to both stub flavours, so the exposure covers the KEY
card's `policy_id_stub` too.

## C-6 — Refusals are ENCODE-SIDE ONLY

`encode.rs:116-131` records a **measured 2026-09-19 regression**: mint-side
refusals leaking into decode made a shipped 2-of-2 **stop reading**, because
`chunk::reassemble` verifies a set by recomputing the encoding id, which ran
admission policy. A refusal on the decode side makes existing plates
unreadable.

Rust asserts the property directly: a refused shape still DECODES.

## C-7 — `EmitTapLeavesChunks` must return a THREE-state internal-key kind

`md/tapleaves.go:188`/`:204` currently return `(keyIndex uint8, isNUMS bool, …)`.
Per §5 the derived xpub takes **no slot**, so `byIndex` can never hold it. A
naive two-state port maps kind 1 onto `isNUMS = true` and the device derives the
**raw H point** — addresses of a different wallet, shown on the consent screen
and at plate verify. See SPEC §7a.

## C-8 — A `debug_assert`, not a runtime error, for the v4+Liana combination

`write_node` asserts that a non-`NumsPoint` unspendable kind is never written at
version 4. It is a programming error, not an input error; a new error variant
would ripple into the refusal work. Go's equivalent should follow the same
reasoning rather than inventing an error.

---

**Maintenance:** add to this file as stage 1b fixes further semantics. Do NOT
turn it into a plan — it is the non-decaying half. Stage 3's plan gets written
when stage 1b has landed, and cites this.
