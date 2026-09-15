# RECON — hashkinds phase 1 (`descriptor-mnemonic`)

Measured 2026-09-15 against `descriptor-mnemonic` at `40c400de`, working tree
clean. Every line:number and count below came from a command, not from reading a
doc comment. This is recon for the phase-1 plan, not the plan itself.

## Headline: the CODEC is already done. The gap is AUTHORING.

The spec's §9 table reads as though `md-codec` needs the four kinds built. It
does not. **Decode, render and miniscript-lowering already handle all four**, and
the coverage is proptested, not incidental:

| layer | file:line | covers |
| --- | --- | --- |
| tag → wire code | `crates/md-codec/src/tag.rs:129-132` | `Sha256` 0x1D, `Hash160` 0x1E, `Hash256` 0x1F, `Ripemd160` 0x20 |
| wire code → tag | `crates/md-codec/src/tag.rs:193-196` | all four |
| body parse | `crates/md-codec/src/tree.rs:297-322` | 32-byte for `Sha256`/`Hash256`, 20-byte for `Hash160`/`Ripemd160` |
| render | `crates/md-codec/src/render.rs:199-202` | all four, by name |
| → miniscript | `crates/md-codec/src/to_miniscript.rs:665-680` | `Terminal::{Sha256,Hash256,Ripemd160,Hash160}` |

Tests: per-kind round-trips at `tree.rs:590-650`, and
`tests/proptest_to_miniscript.rs` generates `Ripemd160`/`Hash256`/`Hash160`
nodes (10 references; `tests/common/mod.rs:492,800` are the generators).

**Consequence, and it is worth stating to a reviewer:** an `md1` string carrying
a `ripemd160` hashlock decodes and renders correctly **today**. What no tool can
do is *compose* one. Phase 1 is a writing-path change.

## The gap, exactly

**1. The hash slot is sha256-shaped in the type system.**
`crates/md-codec/src/compose/mod.rs:151` — `pub hash: Option<[u8; 32]>`. The
32-byte array is the whole problem: `ripemd160` and `hash160` are 20. This is
where `HashKind` / `HashLock` from spec §9.1 land.

**2. Compose lowering hardcodes the tag.**
`crates/md-codec/src/compose/lowering.rs:78` — `tag: Tag::Sha256` with
`Body::Hash256Body(h)`, unconditionally. The doc comment above it
(`lowering.rs:65`) also spells the shape as `and_v(v:KEYS, and_v(v:sha256(H),
LOCK))` and will need to stop naming one kind. Only **3** sites in
`compose/` mention sha256 at all — this is a small, sharp change.

**3. A public preset signature.**
`crates/md-codec/src/compose/presets.rs:88-90` —
`pub fn hashlock_gated(wrapper: Wrapper, hash: [u8; 32], older_blocks: u32)`.
Spec §9.1 calls this out specifically. **10 references** across the repo,
including `test_vectors.rs:484` and three test files
(`compose_support.rs:337`, `compose_crosscheck.rs:216-217`,
`compose_lowering.rs:721`) — so the signature change is mechanical but touches
the vector corpus.

**4. The CLI surface — the largest piece by volume.** **37** `sha256` sites in
`crates/md-cli/src/`. The named ones from the spec:
- `crates/md-cli/src/main.rs:287-288` — the `--path` grammar help text, which
  spells `[,sha256=HEX]` and `keyless,sha256=HEX`.
- `crates/md-cli/src/cmd/compose.rs:133` — the shared parser, whose own comment
  says *"Shared by `--path ...,sha256=HEX` and `--preset hashlock-gated,sha256=HEX`"*.
- `crates/md-cli/src/cmd/compose.rs:373` — the refusal `"{ctx} needs sha256=<64 hex>"`.
- `crates/md-cli/src/cmd/compose.rs:601` — a help/example string.
Plus the `PresetParams` field, the `named_only` allow-list and the `--json` key
named in spec §9.

`md compose`'s option name is a **grammar** decision, not a rename (spec §9):
`sha256=` is kind-specific, so `hash256=` / `ripemd160=` / `hash160=` are
siblings, on **both** `--path` and `--preset`.

## Method note for the phase-1 plan

Phase 2's plan was hand-written first and its R0 Criticals went **2 → 4** across
two rounds because the Rust lived in markdown and nothing compiled it. The method
that worked was **build the branch first, then transcribe the plan from it**, with
every ```rust block carrying a `file=`/`mode=` header checked by
`scripts/h2-plan-blocks-vs-tree.sh`. Do the same here.

Phase 1 is **independent of phase 2** (spec §9: "phases 1 and 2 are independent
of each other"), so it can start without waiting on the `hashkinds-p2` push.

## Not measured here

Whether `validate` (`compose/mod.rs`) has kind-blind assumptions; the `--json`
key's current name; whether any admission rule keys on the 32-byte width; the
`md-cli` vector corpus's shape. Those belong to the phase-1 build.
