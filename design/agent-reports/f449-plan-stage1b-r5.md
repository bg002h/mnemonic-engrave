# R5 closing gate — `IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md` (r6 fold)

**Verdict: 0 Critical / 1 Important (new) / 5 Minor / 3 Nit — NOT GREEN.**

Reviewer: independent agent, sonnet. Scope: the r6 fold (`b092c18c..4b797fd3`)
against `design/agent-reports/f449-plan-stage1b-r4.md`. Spec read for
authority, not re-reviewed (per brief). Repo built and RUN at
`descriptor-mnemonic` `37367c1f`, `clippy 0.1.85 (4d91de4e48 2025-02-17)`
confirmed. Every probe was reverted; `git status --porcelain` is empty in
both repos as of this report.

Both of r4's Importants are genuinely fixed and confirmed by execution. But
Q2(b)'s own question — "is it actually the shape the render-reparse fixpoint
is about?" — surfaces a new Important the fold's fix introduced: `item_7` no
longer calls a renderer at all, so it no longer tests what Task 8 Step 5b
exists to restore. **This blocks.**

---

## Q1 — status of every r4 finding

| # | status | evidence |
| --- | --- | --- |
| **r4/I-1** (in-crate fixture still RED, blames decode) | **ADDRESSED** | RUN below: 88 bits, `decode_payload` → `Ok`. Same fix r4 measured. |
| **r4/I-2** (`item_7`'s new home can't reach `kind1_from_vector`) | **ADDRESSED, but the fix introduces a new defect** | The call is gone — VERIFIED (see Q2b): `item_7` now compiles clean as an md-cli integration test. The specific unreachable-helper problem r4 cited no longer exists. But dropping the call also dropped the only remaining call to `descriptor_to_template` in the whole plan → **r5/I-1**. |
| **r4/M-1** (§2 walk needs `use miniscript::ForEachKey;`) | **NOT ADDRESSED** | `grep -n "ForEachKey"` over the plan → zero matches. Untouched by this fold (Task 4, lines ~380-440, outside both hunks). |
| **r4/M-2** (`tap_tree.leaves()` vs. `script_tree: Option<…>`) | **NOT ADDRESSED** | `tap_tree.leaves()` still literal at `:431`, unchanged. |
| **r4/M-3** (Task 8 Files line md-codec-only vs. Step 5/5b md-cli) | **NOT ADDRESSED** | `:826` (shifted from r4's `:802` by the fold's earlier insertions) still reads `crates/md-codec/tests/liana_unspendable.rs (extend)` only. Step 5 (`:900`, *"It belongs in **md-cli**'s tests"*) and Step 5b (`:935`, *"LIVES IN crates/md-cli/tests/"*) both still contradict it. |
| **r4/M-4** (5 ALLOW entries the plan creates nothing for) | **NOT ADDRESSED**, and one is now sharper — see **r5/M-1** | `grep -n "fn compressed_33\|fn md_err\|fn all_nums_tr\|fn wsh_wrapping_tr_liana\|fn encode_payload_at_forced_version"` over the plan → zero. |
| **r4/N-1** (`EXEMPTED BY ALLOW` doesn't subtract `defined`) | **NOT ADDRESSED** | `scripts/plan-api-check.sh` untouched by this diff (`git diff --stat` shows only the plan `.md` and the r4 report changed); logic at `plan-api-check.sh:79` unchanged. |
| **r4/N-2** (`case`'s benign collision has nowhere to be recorded) | **NOT ADDRESSED** | Same script, unchanged; re-ran it below — `case [ALSO EXISTS in repo]` still prints with no way to mark it checked. |

---

## Q2 — the two corrected blocks, EXECUTED

### (a) the fixture's origin path — **FIXED. Re-run, confirmed.**

Reproduced r4's exact probe verbatim (same fixture, same module, appended
to `crates/md-codec/src/encode.rs`, reverted after):

```
$ cargo test -p md-codec --lib f449_r5_probe -- --nocapture
PROBE-R5 encode SkipPolicy: OK, 88 bits, 11 bytes
PROBE-R5 decode: Ok("OK")
PROBE-R5 encode Enforce (pre-§6): OK, 88 bits
test encode::f449_r5_probe::a_refused_shape_still_DECODES_so_existing_cards_never_stop_reading ... ok
test result: ok. 1 passed; 0 failed
```

88 bits, `decode_payload` → `Ok`, `Enforce` still `Ok` today (pre-§6) — all
three match r4's "measured fix" prediction exactly. `PathComponent { hardened:
bool, value: u32 }` is the real shape at `origin_path.rs:216-226` (checked by
hand, matching struct literal fields).

**Is `is_ok()` the right assertion, or should it be `assert_eq!` once Task 3
lands?** Verified at source, not reasoned about. `encode.rs`'s write arm
(`InternalKey::NumsPoint | InternalKey::LianaUnspendable => { w.write_bits(1,
1); }`) writes the identical bit for both variants today — Stage 1a's own
comment there says so (*"both non-slot kinds still write exactly the v4 NUMS
encoding. STAGE 1B is what makes them differ, and only at v8"*). `decode.rs`'s
read arm for `Tag::Tr` (`tree.rs:277-282`) has no third case: `is_nums=1`
**always** produces `InternalKey::NumsPoint` — the decoder cannot emit
`LianaUnspendable` at all pre-Task-3. So `assert_eq!(decoded, original)` would
fail today (`LianaUnspendable != NumsPoint`) while `is_ok()` is exactly the
achievable property. The plan's reasoning is correct.

Probe reverted: `git checkout -- crates/md-codec/src/encode.rs`, confirmed
clean.

### (b) `item_7`'s literal template — **compiles and is well-formed as a `md encode` argument, but it no longer exercises the renderer. → r5/I-1**

**Does the template have everything `md encode` needs?** Built it as an
md-cli integration test (`fn md(args: &[&str])`, the calling convention
already used at `cli_bip388_double_wildcard.rs:35`) and ran it:

```
$ cargo test -p md-cli --test f449_r5_item7_probe -- --nocapture
running 1 test
PROBE-R5-ITEM7 exit=1 stdout="" stderr="md: template parse error: miniscript parse failed:
  internal key must have no children, but found 1\n"
test item_7_the_render_reparse_fixpoint_covers_tr_kind_1 ... ok
```

Compiles clean — no unreachable-helper error, confirming r4/I-2's specific
citation is gone. The runtime error is **expected and uninteresting**: Task 6
(the `UNSPENDABLE(liana)` marker grammar) is not implemented at `37367c1f`,
and I confirmed the *unmodified* Task 6 example template
(`tr(UNSPENDABLE(liana),{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})` at `:622`, no
explicit path) fails with the **identical** message — so this is "the marker
doesn't exist yet," not a defect in item_7's specific spelling. Probe file
deleted, repo confirmed clean.

**Design check on `--path`.** The template embeds explicit, *divergent*
per-key origins: `@0` at `48'/0'/0'/3'`, `@1` at `48'/0'/1'/3'` (different
account — a genuine two-signer Liana shape). The test then also passes
`--path bip48`. Read at source: `cmd/encode.rs:119` calls
`apply_path_override(&mut descriptor, args.path)`, and
`parse/path.rs:27-35`'s `apply_path_override` is an **unconditional
whole-descriptor overwrite** — `descriptor.path_decl.paths =
PathDeclPaths::Shared(to_origin_path(Some(&dp)))` — whenever `path.is_some()`,
which it always is here. The doc comment on the sibling
`apply_path_override_per_slot` confirms `encode`/`verify`/`vectors` keep this
"existing whole-descriptor-overwrite behaviour, unchanged." `parse_path_name`
(`path.rs:150`) resolves `"bip48"` to `m/48'/0'/0'/2'` — confirmed by running
the crate's own unit test `parses_name_bip48` in the source (assert already
present, not one I added). So the two embedded divergent origins are dead
text: `--path bip48` silently discards them, and what replaces them is not
even the right BIP-48 script-type suffix for a taproot descriptor (`2'`,
P2WSH, not `3'`) by this plan's own convention (Task 7's fixture uses `3'`).
This doesn't make the command error — it just means the elaborate
per-key-divergent-origin spelling in the template is decorative.

**Is this the shape the render-reparse fixpoint is actually about? No.**
Before the fold, `item_7` called `descriptor_to_template(&kind1_from_vector
(…)).unwrap()`, asserted the *rendered* string contained `UNSPENDABLE(liana)`,
then re-parsed *that rendered string* with `md encode`. That is the actual
fixpoint: render → reparse must succeed. The fold's fix removes both the
`descriptor_to_template` call and the containment assertion, replacing the
whole thing with a hand-typed literal — `md encode` on a string nobody's
renderer produced. **Confirmed nothing else exercises the renderer for this
shape**: `grep -n "descriptor_to_template" design/IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md`
now returns **zero matches**, anywhere in the plan. Task 8 Step 5b exists
specifically because *"item 7 — the render-reparse fixpoint — vanished from
this task silently during [an earlier] fold"* (`:915-916`); this fold restores
a test with the same name and the same task slot, but not the property. Once
Task 6 is implemented, `item_7` will go green without `descriptor_to_template`
ever having been called on a tr-kind-1 descriptor — the exact "test that
reports a false PASS" shape the project's severity rules call out as
blocking, and it happens in the one test §9 assigns to prove this specific
property.

→ **r5/I-1 (Important).** A fix exists inside the crate-boundary constraint
r4 already established (md-codec has no `CARGO_BIN_EXE_md`): render in an
md-codec test (`descriptor_to_template(&kind1_from_vector(…))`, keep the
`contains("UNSPENDABLE(liana)")` assertion) and reach the `md` binary from
md-cli by reading the same underlying vendored **data** file directly —
r4 already confirmed md-cli's tests legitimately cross the boundary for
`tests/vectors` **data** (`vector_corpus.rs:22` et al.), just not test
**code**. Not prescribing the exact mechanism; the finding is that the
current spelling supplies none.

---

## Q3 — staleness across ~1000 lines and six folds

**Nothing new contradicts the SPEC.** Both fold hunks read correctly against
source (verified above).

**Of r4's 9 carried contradictions, 8 are unchanged and 1 is now resolved:**

- **#9 (Task 1's include recipe at `:84` vs. Step 5b's "LIVES IN
  crates/md-cli/tests/" at `:935`) is RESOLVED**, but only because `item_7` no
  longer calls `kind1_from_vector` at all (r5/I-1's root cause). `item_2`,
  which still calls it, has no relocation comment and stays in md-codec where
  the include is valid. Not a fold that reconciled the contradiction on
  purpose — it removed the only obligation that produced it.
- **#1-#8** (the `23`/`22` tr_count line, the `55` vs `TWELVE` call-site
  count, Step 1 heading "as UNIT tests" over a two-home table, Task 7's Files
  line omission, the `encode::` nextest filter, Task 8's Files line — now the
  same defect as r4/M-3 —, Task 4's Files line — r3/M1 —, and the chunk.rs
  version-count line) are all **byte-unchanged**, confirmed by re-grepping
  each cited string; still open, still non-blocking, five-to-six rounds
  deferred.

**Two new observations from this round's own reading:**

- **r5/N-1 (Nit).** `:745`, *"The three fixtures these tests use, defined
  here:"* introduces a code block containing exactly **two** function
  definitions (`tr_liana_with_sortedmulti_a_leaf`, `tr_liana_at_use_site`).
  The third fixture Step 1's tests use, `in_crate_tr_liana_with_sortedmulti_a_leaf`,
  is defined 64 lines earlier in a *different* block, for a stated different
  reason (must be in-crate). Confirmed present verbatim in the pre-fold tree
  too (`git show b092c18c:…|grep "three fixtures"`) — **not introduced by this
  fold**, five rounds have not caught it, purely cosmetic (the fixtures
  themselves are correctly used; only the count in the sentence is off).
- **r5/M-1 (Minor, sharpens still-open r4/M-4).** `:1022`'s own "placeholder
  scan" checklist — written specifically to stop this plan's history of
  falsely claiming `plan-api-check.sh` passes — asserts *"Helpers currently
  defined here: …`wsh_wrapping_tr_liana`/`all_nums_tr`/
  `encode_payload_at_forced_version` (Task 7 Step 2b)…"*. MEASURED:
  `grep -n "fn all_nums_tr\|fn wsh_wrapping_tr_liana\|fn encode_payload_at_forced_version"`
  over the plan → **zero**. The paragraph built to prevent exactly this class
  of false claim (it names r0 and r2's prior instances) contains one itself.
  Same root cause as r4/M-4, viewed from the plan's own text rather than the
  gate's ALLOW list.

**No task references a helper that moved.** Every remaining
`kind1_from_vector` call site (`:221, :306, :312, :540, :753, :763, :841,
:859, :926`, plus its own definition at `:110`) sits under an md-codec Files
line; none is in a relocated (`LIVES IN md-cli`) block. `descriptor_to_template`
has zero remaining call sites (see r5/I-1).

---

## Q4 — the gate

Re-ran `./scripts/plan-api-check.sh design/IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md`:

```
all extracted symbols resolve
EXEMPTED BY ALLOW (22) -- each MUST be a symbol this plan CREATES.
   [... 22 symbols, matching the brief's count ...]
checked 86 candidate symbols from 22 rust blocks
```

(86 vs. r4's 87 candidates — expected, `item_7` lost one `kind1_from_vector`
occurrence; not a defect.)

**Direct answer: no.** Checked all 22 against the plan's own prose for
language claiming pre-existence (`grep -n "existing" design/….md` cross-referenced
against each of the 22 names): the one genuine hit (`wire_version` near *"do
NOT change the existing signatures"* at `:513`) refers to the two
**pre-existing** `to_miniscript_descriptor*` entry points, not to
`wire_version` itself, which the plan does define (`:267`, confirmed by r4 and
unaffected by this fold). No exempted symbol is described as already
existing. The `descriptor_with`/`key_arg` contradiction that produced a green
gate on a broken fixture two rounds ago remains closed.

---

## Ready for implementation: no.

One Important blocks, newly found by this round: **r5/I-1** — `item_7`'s fix
for the crate-boundary problem (correctly identified and correctly fixed by
r4/I-2's standard) is a different test than the one Task 8 Step 5b exists to
restore. It compiles, it will pass once Task 6 lands, and it proves nothing
about the renderer — `descriptor_to_template` has no remaining call site
anywhere in the plan. Everything else this round touched is confirmed sound:
r4/I-1 is fixed and re-verified by execution (88 bits, `Ok`, `Ok`), the
`is_ok()`-not-`assert_eq!` reasoning is confirmed correct at source, the gate
repair still holds (Q4: no false "already exists" claims among the 22
exemptions), and no relocated test now references an unreachable helper. The
carried Minors/Nits (r4/M-1 through N-2, r5/N-1, r5/M-1) do not block; the
five-round-old editing sweep remains cheapest folded alongside r5/I-1's fix
rather than carried into a seventh round.
