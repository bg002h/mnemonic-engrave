# R1 — whole-diff adversarial review, `export-wallet --allow` Phase 1

**Diff:** `5f88071c..05ac190b` in
`/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/export-phase1`
(commits `0c672a4c` refactor + `05ac190b` feature).
**Scope:** implementation-introduced defects R0 could not reach; settled rulings
not re-litigated. The controller's machine-verified items (fmt/clippy/nextest
3928, restore byte-identity, the behaviour matrix) were not re-run.

## Verdict: 0 Critical / 1 Important / 2 Minor

The admission-gate design is correctly implemented and the gate set is closed.
The one Important is a hole in the tests, not in the code: the `sh(<miniscript>)`
sigless-detection arm has no test in its firing direction, and I proved by
mutation that the entire 3928-test suite stays green while that arm silently
re-opens the exact defect class Phase 1 exists to close.

---

## Findings

### I-1 (Important) — the `ShInner::Ms` fired-detection arm is untested in the direction that matters; a regression there ships silent sigless emission with a fully green suite

- **Where:** `crates/mnemonic-toolkit/src/descriptor_builder/allow.rs:192`
  (`ShInner::Ms(ms) => !ms.requires_sig()`), versus the unit vector list in
  `sigless_detection_is_per_wrapper` at `allow.rs:371-386`, which tests
  `sh(pk(K))` (→ false) and `sh(wsh(keyless))` (→ true, a *different* match
  arm, `ShInner::Wsh`) but has **no `sh(<keyless miniscript>)` → true case**.
  No CLI test covers it either.
- **Why it is wrong:** the shape is reachable in production —
  `script_type_from_descriptor` (`wallet_export/mod.rs:226`) classifies
  `sh(<Ms>)` as `P2shMulti` and passes it to the gate — and the acceptance
  bullet reads "fired-detection per enforced wrapper — per-leaf for tr,
  **top-level for wsh/sh** — implemented and **tested**". For `sh` top-level,
  it is implemented but not tested. This arm is also the likeliest to be
  rewritten: the `ShInner` shape already changed once at this pin (miniscript
  PR #915 removed the `SortedMulti` variant; the implementer's own report §8
  notes `WshInner` vanished), so the next miniscript bump forces a human back
  into this match with zero test feedback on this arm.
- **Reproduced, not asserted.** Mutation run in the worktree and then reverted:
  1. Changed line 192 to `ShInner::Ms(_ms) => false,`.
  2. `cargo nextest run --locked --workspace` → **3928 run: 3928 passed, 19
     skipped** — no test notices.
  3. Rebuilt binary; `export-wallet --descriptor
     "sh(and_v(v:after(1383520),sha256(4743d7c47df21d29e3ed3dfec5d0c0a884ccc2708637dddf771c36d214056954)))"
     --format bitcoin-core` (no flag) → **exit 0**, importdescriptors JSON
     emitted, **no warning, no refusal** — a flagless anyone-can-spend export,
     the R3-1/wsh-hole class on the legacy-P2SH wrapper.
  4. Reverted; same invocation on the committed code → **exit 2** with the
     gate's message (current behaviour is correct; the code is right, the net
     under it is missing).
- **Concrete regression path:** the next `rust-miniscript` pin bump reshapes
  `ShInner`; whoever adapts the match returns a default/false for the `Ms`
  case; suite green; a sigless `sh()` descriptor exports flagless from then on.
- **What closes it (shape, untested by me):** a `(sh(<keyless>), true)` vector
  in `sigless_detection_is_per_wrapper` — and, if the CLI level is wanted too,
  one flagless-refusal case on the same string (verified above that the
  committed binary answers exit 2 / exit 0-with-flag for it).

### M-1 (Minor) — `expand_literal_double_star` lost its doc comment; its funds-adjacent contract text is now attached to `parse_descriptor_lenient`

- **Where:** `crates/mnemonic-toolkit/src/parse_descriptor.rs:383-432`. The new
  `parse_descriptor_lenient` (with its own doc block) was inserted *between*
  `expand_literal_double_star`'s doc comment and the function it documented.
  The two doc blocks now fuse into one rustdoc attached to
  `parse_descriptor_lenient` (which therefore *opens* with text about `/**`
  expansion precision and idempotence), and `expand_literal_double_star` —
  whose comment records a funds-adjacent precision contract ("NEVER a naive
  global `str::replace`", the terminator-set anchor) — has no doc at all.
- Structural misattachment, not a wording preference; no behavioural effect.

### M-2 (Minor) — on the `--from-import-json` arm, template-requiring formats reach their own verdict *before* gate 2; the "gate before any format verdict" property is proven only for `--descriptor`

- **Where:** `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:979` — the
  `descriptor_is_general_policy` refusal for template-requiring formats runs
  upstream of gate 2 (`:998-1003`). Probed:
  `export-wallet --from-import-json <sigless-wsh envelope> --format coldcard`
  (flagless) → exit **1**, "cannot represent a general wallet policy", not the
  gate's exit-2 message.
- **Why it matters (and why it is only Minor):** no admission hole — the path
  refuses either way, fail-closed, and nothing is emitted. But the test
  `every_format_meets_the_same_gate_before_its_own_verdict`
  (`tests/cli_export_wallet_allow.rs:857-866`) states the property as "EVERY
  format on EVERY arm" while exercising only the `--descriptor` arm, and the
  property is factually false on the envelope arm. A future reader trusting
  that comment as the invariant would mis-model the surface. (The analogous
  pre-gate Fix-α ordering for tr envelopes is *by design* per the plan; the
  general-policy ordering is merely undocumented.)

---

## What I verified and found sound (the negative space)

- **The admission set is closed.** All `EmitInputs` constructions in the
  workspace: `export_wallet.rs:766` (gate 1 at `:762`), `:1007` (gate 2 at
  `:1002`), `restore.rs:2495`/`:2800` (out of scope, see below), plus two
  `#[cfg(test)]` sites (`export_wallet.rs:1098`, `coldcard.rs:451`). Emitters
  are reached only through `emit_payload`, whose only production callers are
  those four sites. No fifth parse/admission site: `expand_bip388_policy`
  (`wallet_import/pipeline.rs:282`) is pure string substitution — no
  descriptor parse — and its output flows into the lenient intake;
  `CheckedDescriptor::new` (`wallet_export/mod.rs:452`) validates only the
  checksum-suffix *shape*, no parse.
- **The four post-gate lenient re-parses are safe.** Every restore path into
  the shared emitters is strict-by-construction: `WalletRow.descriptor` comes
  from `build_descriptor_string` (site 4, kept strict, `pipeline.rs:39`);
  `build_multisig_import_payload`'s general arm strict-parses at
  `restore.rs:2776` before its `EmitInputs`; `emit_completed_multisig`
  strict-parses + round-trips at `restore.rs:2387`. A lenient parse of a
  strict-parseable string is the identical parse, so the emitter relaxation is
  unreachable-different for restore. The `restore.rs:2073` `from_str` is an
  address-search probe returning bool — never feeds an emitter.
- **The refactor commit is behaviour-preserving, including transitively.**
  Moved code compared hunk-by-hunk: `CliAllow`/`kind`/`kebab`/`allow_set`/
  `emit_allow_notes`/`to_ext_params` are verbatim (visibility widened only);
  build wording strings byte-identical. The one place the *feature* commit
  could have altered `build-descriptor` — it calls the now-lenient
  `descriptor_to_bip388_wallet_policy` at `build_descriptor.rs:324` — cannot
  diverge: `WrapperKind` (`descriptor_builder/ir.rs:95`) has exactly one
  variant, `Wsh`, and strict-vs-lenient differ only on `tr()`. The CHANGELOG's
  "`build-descriptor` behaviour is unchanged" holds.
- **The pipeline.rs:39 keep-strict rationale is sound.** It canonicalizes
  builder output (quorum templates only — cannot be sigless) and is restore's
  canonicalizer; relaxing it would be an unruled restore behaviour change. The
  uniform gate still runs on its output (row-1 column-3 test).
- **`parse_descriptor_lenient` is exactly `from_str` minus the tr-only
  checks.** Read at the pinned rev (`~/.cargo/git/checkouts/rust-miniscript-*/
  ff4732e/src/descriptor/mod.rs:1136-1152`): `from_str` = `Tree::from_str` +
  `from_tree` + (Tr-only `sanity_check()` + per-leaf `ext_check(sane)`).
  `Tree::from_str` → `parse_pre_check` → `verify_checksum`
  (`expression/mod.rs:514`), so the BIP-380 checksum is verified on the
  lenient path — and the unit test's flipped-checksum assertion is sound (the
  positive case proves the `#suffix` is consumed, so the negative can only
  fail via verification, not via stripping).
- **Fix-α untouched and still categorical**: the refusal block is outside the
  diff; the envelope-tr test asserts Fix-α's own message with and without the
  flag; probed orderings confirm tr envelopes exit 1 pre-gate.
- **No wrong-export vector found.** Lenient parse yields the same
  `Descriptor` value as strict for anything strict accepts, so canonical
  strings, multipath splits and the bip388 rewrite are unchanged for all
  previously-accepted inputs (restore byte-identity confirms independently).
  For newly-admitted shapes, the bip388 test pins the keyless leaf surviving
  verbatim with no invented key (`keys_info` count asserted).
- **The author's tests are largely genuine.** The printer tests guard their
  own vacuity (exit-0 + note-printed asserted before any "must not contain");
  row-2 cross-arm identity is byte-equality, not substring; the restore
  invariant is the strong form (flagless restore stdout == flagged export
  stdout); the sane control has a positive-control test. Behavioural probes I
  ran beyond the suite: legacy `sh()` gate both directions (correct),
  repeated-key `tr()` emits flagless (the documented second behaviour change —
  works, no crash), duplicate `--allow sigless-branch` dedups to one note,
  `bare` descriptors refuse pre-gate (fail-closed, making the detector's
  `Bare` arm defensively dead on this surface).
- **Docs diff is a pure insertion** (0 deletion lines, re-counted).

## State of the worktree

Mutation probe fully reverted (`git status` clean at `05ac190b`); binary
rebuilt from committed code and re-probed (sigless `sh()` flagless → exit 2).
