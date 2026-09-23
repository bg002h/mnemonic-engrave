# CONTINUITY — F-449 stage 2 (written 2026-09-22)

## RESUME POINT

**Stage 1b is SHIPPED.** dm main `25acb33c` (md-codec 0.46.0, md-cli 0.18.0,
1500/1500, CI green no bypass). engrave master `b5576365`+.

**Stage 2 is SHIPPED (2026-09-23).** descriptor-mnemonic main `cf35d61a`
(md-codec 0.47.0, md-cli 0.19.0, CI green, no bypass), tag
`descriptor-mnemonic-md-cli-v0.19.0`. engrave merge `b8ee1ede`.

### Do this first
**Next cycle after F-449 (operator, 2026-09-23):** the coordinator-compat
cycle (`design/DESIGN_coordinator_compatibility.md`, plan 1b onward) starts
once F-449's last stage (5) ships. Nunchuk and Core compatibility work belongs
there, not in F-449. Carry into it: stage 4's "Bitcoin Core imports this form"
for the Liana key was measured on a Core **v30.99 development build**, not a
release (stage 4 plan, F7). Re-measure it on a released Core, or narrow the
wording.

**Stage 3 merge SHA (read by stage 4a Task 5 Step 0): fork `d2350cb`**
(dm vectors `d269c556`).

**Parallel state (2026-09-23):**
- F-642 (toolkit): merged to toolkit master `642300b7` (0.104.0); push, tag
  `mnemonic-toolkit-v0.104.0`, then the demo/sh2 install bump.
- Stage 4a: DONE and reviewed (whole-branch 0C/0I) on branch `f449-stage4a`
  `ed4b7792` (worktree me-worktrees/f449-stage4a). HELD, per below.
- Stage 3: plan GREEN; implementing in dm-worktrees/sh-worktrees/me-worktrees
  `f449-stage3*`; report design/agent-reports/f449-stage3-impl.md.
- Stage 4: plan being authored against the stage-3 scratch end state in
  .tmp/r0-s3/ -> design/IMPLEMENTATION_PLAN_f449_stage4_device.md.
- Stage 5: `demo/sh2/update.sh` as root@quantoshi.xyz works (measured).

**HOLD, updated 2026-09-23:** stage 3 is in fork main (`d2350cb`), and stage
4a's Task 5 Step 0 passed (ancestry OK, `--- PASS` on the v8 probe), so stage
4a is MERGED and `me` v0.11.0 may be tagged. **Still held:** do not
`cargo install` master's `me` as the local binary until the boards run
firmware that contains `d2350cb`. It confirms v8 cards that the currently
flashed images cannot read.

Stage 3: the Go port in the fork's `md/` (three-state kind, version-derived
identity). Without it the DEVICE cannot read kind-1 plates. Rust-primary
rule: port semantics from md-codec 0.47.0 and update the provenance pin.
Also due: F-642 (toolkit pin bump, owned by the cycle), F-643 (stage 3),
F-644 (stage 4), F-645 and F-646 (next dm release).

### How stage 2 went (for the next plan)
Plan R0 3C/7I → R1 0C/7I → R2 0C/5I → R3 0C/2I → R4 GREEN. Every round after
R0 found its Importants in the PREVIOUS FOLD's remedies. Whole-branch review
0C/1I (md repair misread a mixed card set as version 10); its fix leaked a
second time (unrelated chunked cards), closed by ruling 7: the exit-5 branch
is single-string only. Rulings 1-7 and the implementer's rulings are copied at the end of this
file; the SDD ledger was scratch and is deleted.

### The plan
`design/IMPLEMENTATION_PLAN_f449_stage2_compose.md` — 9 task headings
(1, 1b, 2, 2c, 3=pointer, 4, 5, 6, 7). Review history: r0 3C/7I → r1 0C/7I →
r2 0C/5I (folded 387d4cc4) → r3 0C/2I (folded fab1c881) → r4 pending. Reports in `design/agent-reports/f449-plan-stage2-r{0,1,2,3}.md`.
Recon: `design/RECON_f449_stage2.md`.

### What stage 2 actually is
**One flag.** `md compose --unspendable liana|nums` (default `nums`) at the
single site `compose/tr.rs:47`'s `None => InternalKey::NumsPoint`. Three of
SPEC §9's four stage-2 items already shipped in 1b — `md descriptor` kind 1,
the `UNSPENDABLE(liana)` substitution rule, and the JSON `md-cli/2` bump.
Everything else in the stage is evidence.

### Hard-won facts — do NOT re-derive
- **The live Liana gate works.** `harnesses/liana` builds against
  `/scratch/code/shibboleth/.tmp/fable-liana-src-v15/liana` (v15.0). md's
  rendered `preset-kofn-recovery-tr` is ACCEPTED, addresses returned.
- **F-640 is RESOLVED.** Liana ACCEPTS a nested taptree (depth 2, two recovery
  paths at older 26280/52560). Descriptors in r0's Appendix A. §8.1 is
  satisfiable — do NOT amend the spec.
- **Liana emits ONE error for at least three causes** (`analysis.rs:596-600`):
  a real policy refusal, an internal key that doesn't match the recipe over
  THESE leaves, and a raw NUMS point. **A control does not catch this** — it
  carries its own correct key. Every probe must recompute its internal key for
  its own leaf set and assert it before sending.
- `vendor-liana-evidence.sh` rebuilds `cases.json` WHOLESALE from a hardcoded
  8-name list. A hand-added vector is erased unless that list learns it.
- v8 IS in `is_supported_version` (`header.rs:36-38`); a corrupted v8 card
  repairs at exit 0. §8.9's row needs a version OUTSIDE the accepted set.
- Toolchain: `export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH`

### Carried follow-ups
F-636, F-638, F-641 (one class: the refusal is right, the message names the
wrong thing) → stage 2 Task 6. F-635 → stage 4a. F-640 → closes in Task 5.

### Remaining stages
3 = Go port (without it the DEVICE cannot read kind-1 plates) · 4 = device
screens · 4a = `me` · 5 = rebuild demo/sh2 + deploy to quantoshi.xyz/SH2/.
**Nothing an operator touches on the device has changed yet.**

### Rulings made during stage 2 (from the SDD ledger and the implementer's report)
- Ruling 1: T6 F-638 may edit md-codec error.rs + validate.rs to carry `idx` into `UnspendableUseSiteNotCanonical`; the refusal SET must be unchanged (message-only) — "Not touched" meant no wire/validation behaviour, and F-638 is a follow-up the plan schedules — if wrong, one variant's shape changes; the toolkit pins md-codec 0.45.0 (rev b2c5d693) which predates this variant, so its bump already adds the arm and pays nothing extra.
- Ruling 2: F-639 (owned by stage 2, not in plan) is fixed in stage 2 — verify compares serialisations, so both sides skip admission via a new additive pub fn — cost if wrong: verify accepts a card the mint path would now refuse; verify only answers "does it match".
- Ruling 3: fold M-5 (checksum verified over the marker text the operator wrote) — F-641's class — cost if wrong: a checksum over synthetic hex, which no md output emits, stops working.
- Ruling 4: fold M-9 (repair preserves input case) although pre-existing — the new exit-5 branch inherits it and the fix is one line — cost if wrong: none observable.
- Ruling 5: M-2 → F-643 (stage 3), M-7 → F-644 (stage 4), controller-found repair-vs-decode origin gap → F-645 (next dm release); none introduced by this stage.
- Ruling 6: @i-template checksum over synthetic text → F-646 (pre-existing, not this stage) — cost if wrong: the marker/@i checksum inconsistency ships in 0.19.0.
- Ruling 7: exit-5 branch is SINGLE-STRING ONLY; any multi-string version mismatch exits 2 as in 0.18.0 — spec §8.9 row names 'a chunk', and an unsupported version's header layout is unknowable so no multi-string consistency check can be trusted (second occurrence of the class in the remedy → cut at the boundary) — cost if wrong: a damaged multi-chunk future-version set loses its correction here, same as 0.18.0.
- Implementer rulings: see the `Ruling:` lines below, copied verbatim.
  - Ruling: `compose` / `compose_with` take the new parameter (Task 8 Step 1's statement of the public shape) rather than adding a sibling function — 50 md-codec test call sites updated mechanically to pass `UnspendableKind::Nums` — cost if wrong: a reviewer preferring a non-breaking sibling fn would need the call sites reverted; md-codec is minor-bumped anyway.
  - Ruling: compose's `unspendable_kind` uses decode's vocabulary/presence rule ("liana_unspendable" or ABSENT) and the schema string stays `md-cli/2` — md-cli/2 was minted for SPEC §4a's third state (spec states decode+compose together), the field is additive, and no object an earlier compose emitted is reinterpreted — cost if wrong: a reviewer wanting `md-cli/3` bumps one constant + its pinned test + doc row.
  - Ruling: Step 2's control kept (not deleted) — it is made non-vacuous by M1b rather than by the plan's hoist — cost if wrong: none; it is one extra compose per spelling.
  - Ruling: codec signal is a new pub field on `Composed` (single construction site, lowering::finish) rather than a method — cost if wrong: a pub-field addition on a non-#[non_exhaustive] struct is a public break, absorbed by the planned 0.47.0 minor bump; no downstream constructs Composed by literal (grepped).
  - Ruling: my guessed clean-v12 fixture ('5' at position 0) was wrong; the MEASURED corrected card is `md1uzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg` ('q'->'u'), and the fixtures use the measured value — cost if wrong: none; the codec test asserts it is BCH-clean and reads as version 12.
  - Ruling: the plan's assertion-6 fixture `md1qppqqxzxpp29gtcfh4dhmh72l6atuttfxe3cw2xenm` ("one error at data position 7") MEASURES BCH-CLEAN (correct_chunks -> zero corrections), so as written it exits 2, not 5. The test keeps it as a clean-legacy exit-2 control and builds the one-error card by substituting data position 7 ('x'->'q'); the corrected output equals the plan's string — cost if wrong: if the plan intended a different legacy card, assertion 6 still exercises the version-0 advice path, which is what R4 M-1 targets.
  - Ruling: reused md-cli's existing corpus reader `tests/liana_cases.rs` (added `pub fn all_cases()`) instead of writing a third `Case` deserializer as the plan's text says — md-cli already had its own reader over the same cases.json (the plan's premise "this file carries its own Case deserializer" predates/overlooks it); one reader per crate avoids a second copy — cost if wrong: none functional; a reviewer wanting a separate loader copies 10 lines.
  - Ruling: the descriptor-equality leg's input and oracle are the same evidence string, so a perturbed descriptor reddens at decompose, not at the equality assertion; P3 (a render-side code mutation) is what proves the equality assertion itself can fail — cost if wrong: none.
  - Ruling: harness `Cargo.toml`'s relative Liana path resolves only from the MAIN checkout (from a worktree it points at a nonexistent `me-worktrees/.tmp`), so the gate builds a copy with the path rewritten to `$LIANA_CHECKOUT/liana` (default the shared v15.0 clone) and asserts the rewrite — cost if wrong: none to the committed harness; the gate's build dir is under the gitignored `harnesses/liana/target/`.
  - Ruling: expectation + evidence are one file (`liana-live-gate-expected.jsonl`, line 1 = {liana_tag, liana_commit, entry_point, harness, inputs}) rather than a separate evidence file — cost if wrong: splitting is mechanical.
  - Ruling: F-641's check refuses unparseable marker-bearing text with the expression-tree parse error — the downstream Descriptor::from_str runs the same parser over the same bracket/comma structure, so no input that could have succeeded is refused — cost if wrong: a template the raw-text tree parser rejects but the substituted text would accept would be newly refused; I found no such construction (all md template tokens are within the descriptor charset).
  - Ruling: historical records (§1 baseline `compose/tr.rs:45`, §3f's `parse/template.rs:1604, :1629`) were stale since 1a and describe pre-1a code; left as written — cost if wrong: a reader chasing those lines lands on unrelated code, as they already did before this stage.
