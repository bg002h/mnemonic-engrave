# Final whole-branch review — `f449-stage1` (`6cbd49d8..37367c1f`)

**Reviewer:** independent agent (opus), 2026-09-21.
**Artifact:** branch `f449-stage1` in `/scratch/code/shibboleth/dm-worktrees/f449-stage1`, two commits
(`fec6500d` goldens, `37367c1f` refactor).
**Authorities:** `mnemonic-engrave/design/IMPLEMENTATION_PLAN_f449_stage1a_internal_key.md`,
`mnemonic-engrave/design/SPEC_liana_unspendable_internal_key.md` §3f/§4.

**Counts: 0 Critical / 1 Important / 6 Minor / 2 Nit.**
**Verdict: MERGE.** The one Important is a handoff hazard that costs nothing to
record and does not make stage 1a wrong; see the verdict section for the exact
condition I attach to it.

---

## What I ran (all machine-checked, worktree left clean)

Toolchain pinned per the brief: `clippy 0.1.85 (4d91de4e48 2025-02-17)`,
`CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/f449-target`.

| # | check | result |
| --- | --- | --- |
| 1 | `cargo nextest run --locked --workspace --all-features` at tip | **1439 passed, 3 skipped**, incl. `md-codec::internal_key_refactor every_vendored_wire_vector_re_encodes_to_the_same_bytes` (line 1255 of the captured log) |
| 2 | `./scripts/phase-gate.sh` | **all six steps passed** (build, nextest, clippy `-D warnings`, `fmt --check`, `cargo doc` with `RUSTDOCFLAGS=-D warnings`, tsv sha256) |
| 3 | **Identity re-derivation.** `cargo run --example dump_ids` at tip, diffed against the committed `tests/golden/pre_refactor_ids.json` | **IDS IDENTICAL** — all 65 vectors, all four fields (name, `WalletPolicyId`, `WalletDescriptorTemplateId`, 12-word phrase) |
| 4 | **`total_bits` equality base↔tip.** Scratch example emitting `(name, hex, total_bits)` run at `6cbd49d8` and at `37367c1f` | **65/65 identical on BOTH hex and `total_bits`** |
| 5 | **PROBE A (item 0).** Split the four `NumsPoint \| LianaUnspendable` or-patterns so `LianaUnspendable` is `unreachable!()`, re-ran the whole suite | **1439 passed, 3 skipped — unchanged.** Reverted; tree clean. |
| 6 | **PROBE B (bisectability).** `cargo check --locked --workspace --all-targets --all-features` in a detached worktree at `fec6500d` | **exit 0** — commit 1 builds standalone |
| 7 | **PROBE C (non-zero Tr slot).** Built a `wsh(and_v(pk(@1), tr(@0)))` descriptor, canonicalised and encoded it at base and at tip | identical on both sides: `hex=204ef52108006093280a`, 80 bits, `md1yp802gggqpsfx2q26nd0c9nkv89j4`, decodes to Tr slot **1** |
| 8 | **Corpus shape census.** Walked all 65 vendored vectors | 23 carry `Tag::Tr`: `Slot(0)`+tree ×10, `NumsPoint`+tree ×10, `Slot(0)`+no-tree ×3. kiw over those 23 spans **0,1,2,3,4,5**. **Zero vectors carry a non-zero Tr internal-key slot; zero carry `NumsPoint` with no tree.** |
| 9 | `size_of::<InternalKey>()` | **2** |
| 10 | Site counts | `InternalKey::` appears **62** times in `crates/*/src`, **89** times repo-wide incl. tests/fuzz |

All probe worktrees removed; `git status --porcelain` empty in both the branch
worktree and `/scratch/code/shibboleth/descriptor-mnemonic`.

---

## 0. THE CARRIED QUESTION — answered plainly

**No. Not one of the four non-encode `LianaUnspendable` mappings has any gate
behind it, and I measured it rather than inferred it.**

Reproduction (PROBE A above): I split the four or-patterns at
`crates/md-codec/src/render.rs:192`, `crates/md-codec/src/to_miniscript.rs:341`,
`crates/md-codec/src/policy_shape.rs:251` and
`crates/md-cli/src/format/json.rs:356` so that `InternalKey::LianaUnspendable`
hits `unreachable!()` at each, and ran the full `--all-features` suite. It came
back **1439 passed, 3 skipped — byte-identical to the unmutated run.** Four
mutations, each one turning a "load-bearing ruling" into a guaranteed panic, and
not a single test noticed.

The reason is structural and not fixable by widening the corpus: the whole
workspace contains **zero constructions** of `InternalKey::LianaUnspendable` —
all nine occurrences of the identifier are the definition, three comments, and
five match *patterns* (`grep -rn 'LianaUnspendable' --include='*.rs'`). The
variant is statically unconstructible in 1a, so the four arms are unreachable
dead code by construction. That is exactly what the plan intended and it is what
keeps 1a neutral.

**What it costs.** See Important I1.

---

## Findings

### I1 (Important) — the or-pattern shape switches off the one compiler check that would find these four sites in 1b

`crates/md-codec/src/render.rs:192`, `crates/md-codec/src/to_miniscript.rs:341`,
`crates/md-codec/src/policy_shape.rs:251`, `crates/md-cli/src/format/json.rs:356`.

The stated justification for this entire refactor is that a sum type makes the
illegal states unrepresentable and lets the compiler find every site that must
change. At the four sites where stage 1b *must* act, that benefit has been
deliberately switched off: writing

```rust
InternalKey::NumsPoint | InternalKey::LianaUnspendable => …
```

means that when 1b starts constructing `LianaUnspendable`, **no site produces a
non-exhaustive-match error, no site produces a warning, and (PROBE A) no test
fails.** A forgotten site silently does the NUMS thing:

- `to_miniscript.rs:341` → `build_nums_internal_key()` instead of the derived
  Liana xpub → a **different taproot output key** → a **different address**.
  Per SPEC §7a this is the device address-derivation path; funds are on the line.
- `render.rs:192` → the literal NUMS x-only hex instead of the derived xpub
  (SPEC §4 row 1) or `UNSPENDABLE(liana)` (rows 2–3) → a plate that reads as an
  unspendable-key-path wallet when it is not.
- `policy_shape.rs:251` → `KeyPathKind::Nums`, which feeds `SkeletonKey`
  membership ("internal-key kind" is one of its six axes per CHANGELOG 0.45.1)
  → two logically different wallets collapse to one skeleton key, and the
  coordinator-compat evidence lookup answers about the wrong policy.
- `format/json.rs:356` → the published v1 JSON schema reports `is_nums: true`
  for a Liana key. This one is *intentional* in 1a (the schema is frozen) and is
  the least dangerous of the four, but it shares the same invisibility.

I am **not** calling the plan's ruling wrong — I agree `todo!()`/`unreachable!()`
would be worse (a latent panic in rendering and address derivation), and NUMS is
the only mapping that keeps 1a exactly neutral. The defect is narrower: the
ruling is recorded only in prose (the plan's Step-7 table and `37367c1f`'s commit
message), and prose is not a forcing function. Two cheap mechanisms exist that
prose does not have — a characterisation test that constructs the variant and
pins each of the four mappings to today's answer (it would go RED at each site in
1b, turning "did we remember?" into a failing assertion), or simply spelling the
four arms as separate `InternalKey::LianaUnspendable => …` arms with a
`// stage 1b: change me` marker, which restores the exhaustiveness error the type
was introduced to provide. I am not prescribing either; the choice is the
author's, and doing neither is defensible if the 1b plan opens with the list.

**Reproduced:** PROBE A, above. **Severity rationale:** no wrong result in 1a
(the variant cannot be constructed), but the branch's headline benefit is
undelivered at precisely the four sites where a later wrong result would be a
wrong Bitcoin address.

---

### M1 (Minor) — `tests/golden/pre_refactor_ids.json` is committed and consumed by nothing

`crates/md-codec/tests/golden/pre_refactor_ids.json` (added in `fec6500d`).
`grep -rn 'pre_refactor_ids'` over `*.rs`/`*.toml`/`*.sh`/`*.md` in the tree
returns **zero hits**. By contrast `pre_refactor_encodings.json` has exactly one
consumer (`tests/internal_key_refactor.rs:26`).

This is *defensible by design* — `examples/dump_ids.rs`'s own doc comment says
the capture exists because r0 of the combined plan would have generated it after
`identity.rs:90`/`:200` move to `d.wire_version()`, i.e. it is a time capsule for
**stage 1b**, not a 1a gate. So I am not calling it a false gate.

What it costs today: nothing in this branch asserts that the three identity
outputs did not move, and if they had, the discovery would have landed in 1b with
1b's own changes on top of it. I closed that myself (check 3): regenerated at tip
and diffed — **identical for all 65 vectors across policy id, template id and the
12-word phrase.** Recording the measurement here so 1b inherits a verified
baseline rather than an assumption.

---

### M2 (Minor) — the plan's "a non-zero internal-key slot is canonically unreachable" is false, and I can mint the counterexample

`IMPLEMENTATION_PLAN_f449_stage1a_internal_key.md` Step 1b states: *"A non-zero
internal-key slot is therefore **canonically unreachable**, not merely absent
from the corpus."* That is the entire justification for not adding a non-zero
index test.

It holds only when `Tag::Tr` is the **root**. The codec does not enforce that.
PROBE C: a `wsh(and_v(pk(@1), tr))` tree canonicalises the Tr internal key to
`Slot(1)` (`canonicalize_placeholder_indices` registers by pre-order first
appearance, and here `KeyArg{index:1}` comes first), and the result encodes to a
real 80-bit payload and a real card — `md1yp802gggqpsfx2q26nd0c9nkv89j4` — that
`decode_md1_string` reads straight back with the Tr slot at **1**. The repo's own
fixtures already build a nested Tr (`tests/common/mod.rs:538`,
`wrap(Tag::Wsh, tr_node(true, 0, None))`), so this is not a shape nobody
contemplates.

**This does not change the branch's correctness.** I ran the same construction at
`6cbd49d8` and at `37367c1f`: identical `hex`, identical bit count, identical
decode. The write path for `Slot(i)` is bit-for-bit the old `key_index` path.

What it does change is the *strength of the argument*, and therefore the weight
of Task 1's M2 (see triage): the one wire-reachable Tr state the 65-vector gate
does not cover is reachable after all, and the corpus census (check 8) confirms
**0 of 23 Tr vectors carry a non-zero slot**. The layer is covered — by exactly
one unit test, `tree.rs:561 tr_is_nums_false_round_trip` (`Slot(2)`, kiw=2,
`tree: None`) — which is a thinner margin than "canonically unreachable" implies.

---

### M3 (Minor) — `InternalKey` makes `write_node`/`read_node` non-injective

`crates/md-codec/src/tree.rs:150-166` and `:278-296`.

Stage 1a's write path maps **two** variants onto one wire spelling
(`NumsPoint | LianaUnspendable => w.write_bits(1, 1)`), and the read path can
only produce `NumsPoint`. So `read_node(write_node(x)) != x` for
`x = LianaUnspendable`. Unreachable today (I1's measurement), and unavoidable
given the "no wire change" constraint — but it is a genuine loss of a property
the old `(bool, u8)` pair held for every representable value, and it is invisible
to every current test because nothing can construct the variant.

The moment 1b adds an `Arbitrary`/proptest strategy over `InternalKey` — or a
round-trip property test over `Body` — this fires. It should fire: the correct
statement in 1b is that round-trip identity is conditional on the wire version
(injective at v8, lossy at v4). Worth writing down before a future proptest
"discovers" it as a bug.

On the other side of the ledger, and worth saying plainly because it is the
branch's real achievement: the old pair admitted **255 illegal states**
(`is_nums = true` with `key_index != 0`), guarded only by a `debug_assert!` in
`write_node` that fired on *write*, not on construction, and was compiled out in
release. Those are gone by construction. **Net, `InternalKey` is a clear
soundness improvement.**

---

### M4 (Minor) — nine production-source references to fields that no longer exist

These name the **Rust item** `Body::Tr::is_nums` / `Tr.key_index`, not the wire
bit, so they are now false:

| file:line | text |
| --- | --- |
| `crates/md-codec/src/nums.rs:6` | ``Body::Tr { is_nums: true, .. }`` (a pattern that no longer compiles) |
| `crates/md-codec/src/policy_shape.rs:37` | "the wire's only internal-key discriminant is `Body::Tr::is_nums`" |
| `crates/md-codec/src/policy_shape.rs:119` | "(`crate::tree::Body::Tr::is_nums`) says exactly this and nothing" — **rustdoc on a public item** |
| `crates/md-codec/src/policy_shape.rs:122` | "A real extended key sits in the internal-key slot — `is_nums = false`" |
| `crates/md-codec/src/error.rs:483` | "NUMS is now flag-driven via `Body::Tr.is_nums`" — **rustdoc on a public item** |
| `crates/md-codec/src/canonicalize.rs:12` | "the tree's `KeyArg.index` and `Tr.key_index` fields" |
| `crates/md-codec/src/canonicalize.rs:96` | "`Tr.key_index` in `node` (recursive)" |
| `crates/md-codec/src/skeleton.rs:301` | "`is_nums` always changes the internal-key …" |
| `crates/md-cli/src/parse/template.rs:6` | "v0.30 encodes this via the explicit `Body::Tr { is_nums: true, .. }`" |

Also `crates/md-cli/src/parse/reuse.rs:494`: "`Body::Tr` holds the internal key
as a bare index" — now a `Slot`, not a bare index.

**Explicitly NOT stale**, and I checked each before listing the above: every
reference to `is_nums(1)` as the name of the **wire bit** in SPEC v0.30 §7 remains
correct — `tree.rs:59`, `:152`, `:278`, `encode.rs:34`, `decode.rs:85`,
`error.rs:485`, `test_vectors.rs:273`, `parse/template.rs:1598`, and the
bit-length pins in `tree.rs`'s tests. And `format/json.rs:322-327` is correct
because the published JSON schema genuinely still emits those two fields.

Note the two rustdoc entries: `cargo doc -D warnings` cannot catch them because
they are backticked code spans, not intra-doc links — and this branch's own
parent commit (`6cbd49d8`) is *"docs on public items must not LINK to private
ones (CI cargo doc)"*, i.e. the gate that exists next door does not reach these.

---

### M5 (Minor) — `tree.rs:19-20`'s comment states two things the code beside it does not support

```
// Copy is deliberate: the type is one byte plus a discriminant and is passed
// by value at ~98 sites; Hash mirrors what Body already derives.
```

- **"one byte plus a discriminant"** — measured `size_of::<InternalKey>() = 2`.
  Harmless, but it is a size claim that was never run.
- **"~98 sites"** — measured: `InternalKey::` occurs **62** times in
  `crates/*/src` and **89** times repo-wide including tests and fuzz. Neither is
  98. The number appears to be carried over from the plan's *blast-radius*
  estimate for the old `is_nums` identifier ("88 `is_nums` occurrences across 14
  files" plus the md-cli sites), which is a different quantity from "sites where
  `InternalKey` is passed by value".

The `Copy` derive itself is right and the reasoning is right; only the two
numbers are unmeasured. This is Task 1's M1, confirmed and quantified.

---

### M6 (Minor) — the gate pins the golden's self-reported count, never the directory

`crates/md-codec/tests/internal_key_refactor.rs:27-32` asserts `g.count == 65`
and `g.tr_count == 23` — both read out of the golden JSON itself — and then
iterates `g.vectors`. It never lists `tests/vectors/`. `all_vendored_vector_names()`
exists and does exactly that, and `37367c1f` had to add `#[allow(dead_code)]` to
it (`tests/common/vendored.rs:37`) *because this test does not call it*.

Consequence: removing or renaming a vector fails loudly (`load_vendored_phrase`
panics on the missing file), but **adding a 66th vector leaves the gate green and
the new vector entirely outside it**. The corpus is the gate's whole population,
so its growth should be the gate's business. Not a defect in the current tree —
I verified 65 files on disk at both base and tip — but the blind spot is
one `assert_eq!(all_vendored_vector_names().len(), g.count)` wide.

---

### N1 (Nit) — `tests/common/mod.rs:85`'s `tr_node(is_nums, key_index, …)` is now a lossy wrapper

`pub fn tr_node(is_nums: bool, key_index: u8, tree: Option<Node>)` keeps the
retired vocabulary and silently discards `key_index` when `is_nums` is true.
Under the old code that combination tripped `write_node`'s `debug_assert!`; now
it is silently absorbed.

I checked all 27 call sites (`grep -rn 'tr_node(' crates/`): **every one that
passes `is_nums = true` passes `key_index = 0`.** So there is no live defect —
only a footgun for a future test author, who will write `tr_node(true, 3, …)`
expecting to construct something and get `NumsPoint`. This is Task 1's M4,
confirmed harmless today.

---

### N2 (Nit) — `policy_shape.rs:37-39`'s module doc makes a normative F-449 claim that stage 1b will contradict

> "an unspendable-xpub internal key (the Nunchuk shape F-449 records) is, on the
> wire, an ordinary `key_index`"

True at wire version 4, false the moment version 8 ships kind 1. Flagging it
here so 1b's fold has it on the list rather than discovering it as drift; it does
not need to move in 1a.

---

## Cross-cutting answers

### 1. Wire behaviour as a whole — what the 65 do NOT cover, and could this diff have changed it?

The census (check 8) says the 23 Tr vectors cover exactly three shapes:
`Slot(0)` + tree (10), `NumsPoint` + tree (10), `Slot(0)` + no tree (3), across
kiw ∈ {0,1,2,3,4,5}. The wire-reachable Tr states they do **not** cover are:

| uncovered state | could this diff have changed it? | evidence |
| --- | --- | --- |
| `NumsPoint` with `has_tree = false` (bare `tr(<NUMS>)`) | **No.** Old: `write_bits(1,1)` then `write_bits(0,1)`. New: identical. | covered by `tree.rs:546 tr_is_nums_true_round_trip` and `tree.rs:783`; and `md-cli` builds this shape (`parse/template.rs:1992`) |
| `Slot(i)`, `i != 0` | **No — measured.** | PROBE C: identical hex / bit count / decode at base and tip for a real `Slot(1)` card. Also `tree.rs:561` round-trips `Slot(2)` at kiw=2 |
| `Slot(i)` with `i >= n` (out of range) | **No.** `check_placeholder_bounds` and `walk_for_placeholders` both keep the exact `>= n → NUMSSentinelConflict` shape | `validate.rs:805 placeholder_usage_rejects_out_of_range_in_tr_key_index` (`Slot(3)`) still green |
| `Slot(i)` with `i >= 2^kiw` (silently truncated by `write_bits`) | **No.** Pre-existing on both sides, untouched | n/a |
| the old illegal pair `is_nums = true, key_index != 0` | **Yes — and this is the only behaviour change in the diff.** Old code `debug_assert!`-panicked in debug/test builds and silently dropped the index in release. New code cannot represent it. | strictly an improvement; no test asserted the panic (N1's audit of all 27 `tr_node` call sites) |

Beyond `Body::Tr` the diff touches nothing that reaches the wire: `encode.rs` and
`decode.rs` are untouched, and every other edit is a mechanical destructuring
change whose control flow I walked arm by arm against the base
(`canonicalize.rs` ×3 arms, `validate.rs`, `canonical_origin.rs`,
`compose/tr.rs`, `parse/reuse.rs`, `parse/template.rs` ×2, `tests/common/mod.rs`
×3 walkers). The `..`-absorbed sites (`encode.rs:152`, `decode.rs:130`,
`seat/compose.rs:148`, `validate.rs:241`, `canonical_origin.rs:53/:57`) are
correctly left alone, as the plan predicted.

One worth naming because it is the kind of thing a mechanical rewrite gets wrong
and this one did not: `canonicalize.rs:104-106` replaces the in-place
`*key_index = perm[*key_index as usize]` with `*internal_key = InternalKey::Slot(perm[*i as usize])`,
which is a write *through* the same reference the index is borrowed from. It
compiles and is correct because the RHS is evaluated before the place — but it is
the one rewrite in the diff where a reviewer should stop, and it is right.

### 2. Identity hashes — is there a reachable descriptor whose identity this diff changes?

**No.** Two independent arguments, one measured and one structural.

*Measured:* regenerated the identity golden at tip and diffed against the
pre-change capture — **65/65 identical** on `WalletPolicyId`,
`WalletDescriptorTemplateId` and the 12-word phrase (check 3). That is the
strongest form of the answer for the corpus.

*Structural, which extends it past the corpus:* the only Tr-dependent input to
any of the three identities is `crate::tree::write_node`, called at
`identity.rs:90` (template id) and `identity.rs:200` (policy id), after
`canonicalize_placeholder_indices` in both cases — the same canonicalise-then-write
pair `encode_payload_inner` uses at `encode.rs:144-146`. `write_node`'s Tr arm is
bit-identical for every representable input (§1 table). `expand_per_at_n`
(`canonicalize.rs:434-500`) iterates `0..d.n` over path/use-site/fingerprint/xpub
records and **never matches `Body::Tr`** — I read it. `Md1EncodingId` goes through
`encode_payload_for_identity` → `encode_payload_inner`, same path.

So every constructible descriptor hashes the same before and after, and the one
input class the old code could hold and the new cannot (`is_nums = true,
key_index != 0`) hashed identically anyway, because the index had no wire
representation.

### 3. Is `InternalKey` sound?

**Net yes, clearly.** It removes 255 illegal states that the old pair admitted
under nothing stronger than a release-stripped `debug_assert!` (M3, second half).
It introduces exactly one new latent illegal state — `LianaUnspendable` at wire
version 4, which encodes as `NumsPoint` and therefore breaks round-trip
injectivity (M3, first half) — and that state is statically unconstructible in
1a. `Slot(u8)` has the same domain as the old `key_index: u8`, with the same
downstream bounds checks in `validate.rs` and `canonicalize.rs`, so nothing was
loosened there. `Copy`/`Hash` are appropriate and mirror `Body`.

### 4. Structurally invisible to a per-task review

- **Commit 1 builds standalone** (PROBE B, exit 0, `--all-targets --all-features`).
  The branch is bisectable, which matters because the `serde` dev-dependency
  needed by commit 2's test lands in commit 2 with it, not ahead of it.
- **Commit 2 edits a helper commit 1 introduced** —
  `tests/common/vendored.rs` gains `#[allow(dead_code)]` on
  `all_vendored_vector_names` (+6 lines). I checked this is annotation-only: the
  helper's *behaviour* is unchanged, so the golden captured in commit 1 and the
  gate reading it in commit 2 are reading the same corpus the same way. (The
  annotation is itself the fingerprint of M6.)
- **The golden-not-regenerated guard is real, not vacuous.** The golden lands in
  `fec6500d`, a commit containing no `src/` changes, so by the time `37367c1f`
  runs `git diff --quiet HEAD -- crates/md-codec/tests/golden` the golden is
  already an ancestor. Confirmed by the commit ordering and by check 4, which
  re-derives the same bytes from unmodified base code.
- **`fuzz/Cargo.lock` drifted and this commit fixed it** (`md-codec 0.42.0` →
  `0.45.1`). Pre-existing rot, corrected as a side effect; no objection, worth
  knowing it happened.
- The three `parse/template.rs` tests that previously destructured `is_nums: _`
  and asserted only `key_index == 0` now assert
  `internal_key == InternalKey::Slot(0)` — **strictly stronger** than before
  (they could previously pass on a NUMS node). An improvement hiding inside a
  mechanical diff.

### 5. Merge readiness, with 1b landing on top

Ready. The gate is green from an independent run (check 2), the goldens are
independently re-derivable (checks 3–4), each commit builds (check 6), and
`md-cli`'s `md-codec = { path = "../md-codec", version = "=0.45.1" }` pin
(`crates/md-cli/Cargo.toml:28`) still resolves because the version did not move.

**One condition, and it is the whole of my disagreement-or-not on M5 below: no
md-codec release may be cut from `main` between this merge and 1b's bump.**
`md-codec` is a published crate (no `publish = false`; `documentation =
"https://docs.rs/md-codec"`), `CHANGELOG.md:7` records `## md-codec [0.45.1] —
2026-09-20` as a shipped release, and `.github/workflows/release.yml` publishes.
After this merge, `main` carries a crate whose public API differs from the
published 0.45.1 **at the same version number**. That is harmless while it stays
on `main` and 1b bumps it; it becomes a real SemVer break the instant a `0.45.2`
is tagged, because Cargo would resolve that patch into any `md-codec = "0.45"`
dependent and hand it a `Body::Tr` that no longer has the fields it compiles
against. I checked the constellation's external consumers —
`mnemonic-engrave/crates/me-cli/Cargo.toml:26`,
`me-worktrees/policy-harness/crates/me-cli/Cargo.toml:26` (both `"0.42"`) and
`me-impl-scratch/fixture-gen/Cargo.toml:7` (`"=0.36.0"`) — all pin below 0.45, so
**nothing breaks today**.

No CHANGELOG entry is required by observed convention: entries are written at
release commits (`git log -- CHANGELOG.md` shows `md-codec 0.45.1:`,
`md-codec 0.45.0:`, …), nothing in CI checks it, and this branch is not a
release.

---

## Triage of the deferred list

| item | must it be fixed before merge? | reasoning |
| --- | --- | --- |
| **Task 0 M1** — encodings golden discards `total_bits`; a bit-count change confined to already-zero padding would not be caught | **No — and I closed it empirically.** | Check 4: I dumped `(hex, total_bits)` at `6cbd49d8` and at `37367c1f` and diffed. **65/65 identical on both fields.** The hypothetical is real as a *gate* gap and worth a follow-up, but for this diff it is measured shut. |
| **Task 1 M1** — `tree.rs:20` doc states something false | **No.** Filed above as M5 with both numbers measured (`size_of` = 2, not "one byte plus a discriminant"; 62/89 sites, not "~98"). Comment-only; changes no behaviour and re-triggers no gate. |
| **Task 1 M2** — the gate's Tr-slot-index coverage is one test deep | **No, but it deserves more weight than "minor coverage".** | Confirmed and sharpened: 0 of 23 Tr vectors carry a non-zero slot (check 8), and the plan's reason for not adding one — "canonically unreachable" — is **false** (M2, with a mintable counterexample card). What rescues it is that I verified the uncovered shape byte-for-byte across base and tip myself (PROBE C). So the *code* is proven, but the *standing* coverage is one unit test (`tree.rs:561`). Recommend the follow-up carry the counterexample so 1b does not re-inherit the false premise. |
| **Task 1 M3** — stale `is_nums`/`key_index` in production source | **No.** Enumerated as M4: nine genuinely-stale sites (two of them rustdoc on public items), and I separately verified that the ~25 *other* `is_nums` mentions are correct wire-bit references that must NOT be changed. That distinction is the useful output here — a blanket sweep would make the file worse. Comment-only. |
| **Task 1 M4** — `tests/common/mod.rs:85` keeps the retired vocabulary as a lossy wrapper | **No.** Filed as N1. Audited all 27 call sites: every `is_nums = true` call passes `key_index = 0`, so nothing is silently dropped today. Test-helper ergonomics, owning phase 1b at the latest. |
| **Task 1 M5 (RULED)** — public API broke at an unchanged version; 1b bumps to 0.46.0, not 0.45.2 | **No, and I agree with the ruling.** | It is correct on every point I could check: `Body::Tr`'s fields are public API, `md-codec` is published with no `publish = false`, `documentation = docs.rs/md-codec` (`crates/md-codec/Cargo.toml:11`), the repo's stated convention is that `0.X` is the breaking axis (`CHANGELOG.md:5`), and md-cli's `=0.45.1` pin resolves unchanged. My only addition is the condition in §5: the ruling is safe **provided no release is tagged from `main` in the 1a→1b window**. If one is, 0.46.0 must be cut then and there rather than deferred. |

**Net: none of the six deferred items must be fixed before merge.** Four are
comment- or test-helper-only; two (Task 0 M1, Task 1 M2) name real gate gaps that
I closed by direct measurement for this specific diff, leaving the gap itself as
a follow-up rather than a blocker.

---

## Verdict

**MERGE.**

The branch does what it says: **zero wire bytes change**, verified three
independent ways (the committed 65-vector byte gate; my own re-derivation of the
`total_bits` axis the gate discards; and a hand-built counterexample in the one
Tr shape the corpus omits). **Zero identity change**, verified by regenerating
all three identity outputs for all 65 vectors and diffing. The type change is a
net soundness win — 255 illegal states removed. Both commits build; the gate is
green from an independent run.

The single Important (I1) is a **handoff** finding, not a correctness finding:
stage 1a is neutral precisely *because* those four arms are unreachable, and the
cost lands entirely in stage 1b, where a forgotten site is a wrong address with
no compiler error and no failing test. It does not make 1a wrong and I do not
think it should hold the merge — but it should not be carried as a Minor either,
because the mechanism that would normally catch it (exhaustiveness checking) is
the exact mechanism this branch was introduced to provide and has switched off at
those four sites. Whether that is answered by a characterisation test, by
splitting the or-patterns, or by making it the first line of 1b's plan is the
author's call; the finding is that prose alone is currently the only mechanism.

**Conditions I would attach to the merge:**

1. Record I1 as a **gating** stage-1b entry (owning phase: 1b), naming all four
   file:line sites, so that "did 1b change all four?" is a checklist item rather
   than a search.
2. Do not tag an `md-codec` release from `main` between this merge and 1b's
   `0.46.0` bump (§5).

Neither is a change to the branch.

