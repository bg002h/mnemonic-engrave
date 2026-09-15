# CONTINUITY — hashlock kinds (F-507), 2026-09-15

Resume point for the cycle that makes all four miniscript hash fragments
authorable and engravable. Phase 2 is shipped; phases 1, 3 and 4 are not.

## The fact the design rests on

All four fragments lower to `OP_SIZE <32> OP_EQUALVERIFY <hashop> <h> OP_EQUAL`.
**The preimage is always 32 bytes**; only `<hashop>` and the digest width (32 or
20) move. `hash160` = RIPEMD-160(SHA-256(x)), `hash256` = sha256d — **four
functions, not two widths**.

**The axis that caused most of this cycle's defects:** the hash KIND (what the
script commits to) is not the preimage METHOD (`preimage_hardened` vs
`preimage_sha256`). They share the token `sha256`. Keep saying both or neither.

## State

| phase | repo | state |
| --- | --- | --- |
| spec | mnemonic-engrave | `design/SPEC_hashlock_kinds.md` — GREEN, 7 rounds incl. a journey walk |
| 1 | descriptor-mnemonic | **not started.** Recon done: `design/RECON_hashkinds_P1_descriptor_mnemonic.md` |
| 2 | mnemonic-secret | **SHIPPED** — `origin/master` `9dcd2e0`, ms-codec 0.10.0 / ms-cli 0.19.0 |
| 3 | mnemonic-engrave | not started. Pins phase 2 **by git rev**, not by publish (spec §9) |
| 4 | seedhammer fork | not started. One phase on purpose; re-pins the corpus SHA |

Phase 2 shipped with 101 suites / 584 tests / clippy 0 / fmt clean / vendor OK,
as six per-task commits, every boundary gated from a clean checkout, tree
byte-identical to the reviewed branch, pushed via `ci/staging` with all four
required contexts success and **no bypass line**.

Corpus SHA phase 4 must re-pin to:
`0a911f78f3cdc867dcc44483b7f4c0c1ac87b6d9b30b79f52094e8979bc3d8ce`.

## Phase 1 is smaller than the spec's §9 table implies

Measured against `descriptor-mnemonic` `40c400de`: **`md-codec` already decodes,
renders and lowers all four kinds**, proptested. An `md1` carrying a `ripemd160`
hashlock reads correctly *today* — it just cannot be composed. Phase 1 is a
**writing-path** change:

1. `descriptor-mnemonic/crates/md-codec/src/compose/mod.rs:151` — `pub hash: Option<[u8; 32]>`. The 32 is the problem.
2. `descriptor-mnemonic/crates/md-codec/src/compose/lowering.rs:78` — `tag: Tag::Sha256`, hardcoded. Only 3 sha256 sites
   in `compose/` at all.
3. `descriptor-mnemonic/crates/md-codec/src/compose/presets.rs:88-90` — `hashlock_gated(_, hash: [u8; 32], _)`, public,
   10 references including the vector corpus.
4. `md-cli` — 37 sha256 sites; `--path`/`--preset` grammar, `PresetParams`,
   `named_only`, the `--json` key.

Phase 1 is **independent of phase 2** (spec §9) and can start immediately.

## Method, earned the hard way

- **Build first, transcribe the plan from the branch.** Two rounds of
  hand-written Rust in markdown took the plan's Criticals from 2 to 4. Every
  ```rust block carries `file=`/`mode=` and is gated by
  `scripts/h2-plan-blocks-vs-tree.sh <plan> <worktree>`.
- **The transcript gate fails toward the CODE.** Three of its first four
  failures were the branch having dropped what the plan prescribed — including
  the method-vs-kind rationale on `HashKind` — not the plan being stale.
- **Reshaping into per-task commits IS a gate**, and
  `scripts/hashkinds-p2-reshape.sh` makes it one command. Run `cargo metadata`
  FIRST at each boundary: an exact version pin breaks resolution before any
  compiler runs, which is how a four-boundary defect hid from nine rounds.
- **A GREEN round is not closure.** Round 6 returned 0C/0I; round 7 then found
  two Importants in the next fold. Rounds 4, 5, 7 and 8 each found a defect, and
  every one arrived in the fold answering the previous round.
- **The journey walk found what six correctness rounds could not** — four
  Importants, all "missing things at moments". The card was internally
  consistent and named the wrong opcode for three of four kinds.
- **Assert a mutation APPLIED.** Two silently no-oped here (one hit the wrong of
  two identical guards; one missed because `fmt` had joined a line) and both
  green results read as passes.
- **Never hand-count.** `F-536`'s enumeration was wrong twice — once by grepping
  the wrong thing, once with a total that did not sum. It is now
  `scripts/hashlock-notice-classify.py`, output pasted verbatim.

## Open follow-ups

- **F-534** — `ms decode`'s preimage route answers only for sha256 (`mnemonic-secret/crates/ms-cli/src/cmd/decode.rs:207`). Phase 3.
- **F-535** — the `phrase:` record carries the method axis, not the kind. Phase 3.
  Note the asymmetry: `qr_text` (the plate) gained `hash: <kind>`; the record it
  is cut from did not.
- **F-536** — six hazard notices still scoped to the engraving card. Sharpest is
  the `--random` data-loss line, suppressed by `mnemonic-secret/design/SPEC_ms_hashlock.md:340`'s own
  documented one-liner. Cycle sweep.

## Housekeeping

Two worktrees are now redundant (their branches are merged and pushed) and are
left in place rather than removed unasked:
`/scratch/code/shibboleth/ms-worktrees/hashkinds-p2` and `…/p2-shaped`.
`mnemonic-secret` master carries one unpushed report commit (`2bf1b3f`), by the
local convention that report commits batch with the next code push.
