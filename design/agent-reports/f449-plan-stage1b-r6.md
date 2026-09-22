# R6 closing gate — `IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md` (r7 fold)

**Verdict: 0 Critical / 1 Important (new, replaces r5/I-1) / 5 Minor (carried) / 3 Nit (carried) — NOT GREEN.**

Reviewer: independent agent, sonnet. Scope: the r7 fold (`4b797fd3..HEAD`) against
`design/agent-reports/f449-plan-stage1b-r5.md`. Spec consulted at
`design/SPEC_liana_unspendable_internal_key.md` for the authoritative
definition of the render-reparse fixpoint (§4/§8 item 7) — not re-reviewed.
Repo built and RUN at `descriptor-mnemonic` `37367c1f`, `cargo clippy 0.1.85
(4d91de4e48 2025-02-17)` confirmed. One probe test was added to prove the
Q2(iii) shape claim and reverted immediately after. `git status --porcelain`
is empty in both repos, confirmed at the end of this review.

The fold's diff is exactly one hunk: `item_7` (which r5/I-1 found called no
renderer) is split into `item_7a` (md-codec, calls `descriptor_to_template`)
and `item_7b` (md-cli, a literal template string passed to `md encode`).
r5/I-1's literal citation — "`descriptor_to_template` has no remaining call
site anywhere in the plan" — is now false: `item_7a` calls it (line 947). But
two things RUN this round show the fixpoint Task 8 Step 5b claims to restore
is still not delivered: (a) `item_7b`'s call to `md(...)` discards its return
value, so the test cannot fail no matter what `md encode` does; (b)
`item_7a`'s actual renderer target and `item_7b`'s literal are not the same
shape and never were — MEASURED against the real vector. **This blocks.**

---

## Q1 — status of every r5 finding (9 items: 1 Important, 5 Minor, 3 Nit)

| # | status | evidence |
| --- | --- | --- |
| **r5/I-1** (`item_7`'s fix doesn't restore the render-reparse fixpoint) | **PARTIAL** | The specific citation ("no remaining call site anywhere in the plan") is fixed — `descriptor_to_template` is called once, at `:947`, inside `item_7a`. But the underlying property is still not delivered; see **Q2 / r6/I-1** below. |
| r4/M-1 (§2 walk needs `use miniscript::ForEachKey;`) | **NOT ADDRESSED** | `grep -n "ForEachKey"` over the plan → zero matches. Outside the diff (Task 4, unaffected). |
| r4/M-2 (`tap_tree.leaves()` vs. `script_tree: Option<…>`) | **NOT ADDRESSED** | Still literal at `:431`, unchanged. Outside the diff. |
| r4/M-3 (Task 8 Files line md-codec-only vs. Step 5/5b md-cli) | **NOT ADDRESSED, and now names a concrete gap** | Task 8's Files line (`:826`) still reads `crates/md-codec/tests/liana_unspendable.rs (extend)` only. The fold's own split now requires a *second*, brand-new file — `crates/md-cli/tests/liana_input_side.rs` (comment at `:953`) — that Task 8's Files line does not mention at all. Same defect, now with a nameable missing entry. |
| r4/M-4 (5 ALLOW entries the plan creates nothing for) | **NOT ADDRESSED** | `grep -n "fn compressed_33\|fn md_err\|fn all_nums_tr\|fn wsh_wrapping_tr_liana\|fn encode_payload_at_forced_version"` over the plan → zero. Unaffected by the diff. |
| r5/M-1 (Self-Review's own helper list is false) | **NOT ADDRESSED** | Line `:1039` still claims `wsh_wrapping_tr_liana`/`all_nums_tr`/`encode_payload_at_forced_version` are "currently defined here." Still zero matches for their `fn` definitions. Outside the diff. |
| r4/N-1 (`EXEMPTED BY ALLOW` doesn't subtract `defined`) | **NOT ADDRESSED** | `scripts/plan-api-check.sh` untouched (`git diff --stat 4b797fd3..HEAD` touches only the plan `.md` and the r5 report). |
| r4/N-2 (`case`'s benign collision has nowhere to be recorded) | **NOT ADDRESSED** | Re-ran the gate below — `case [ALSO EXISTS in repo -- is it really created here?]` still prints with no way to mark it checked. |
| r5/N-1 (Task 7's "three fixtures" sentence defines two) | **NOT ADDRESSED** | `:745` still reads *"The three fixtures these tests use, defined here:"* over a block containing exactly two `fn` definitions. Outside the diff. |

**8 of 9 carried findings are byte-unchanged** (outside the one hunk the fold
touched). r5/I-1 is the only one the fold acted on, and it only partially
closes.

---

## Q2 — the split fixpoint, RUN

### (i) Can `item_7a` and `item_7b` fail independently?

**Structurally yes** (different crates, different test binaries, different
assertions) — **but `item_7b` cannot fail on the property it names**, which
makes the independence moot for that half. `item_7b`'s body is:

```rust
md(&["encode", t, "--path", "bip48"]);        // must not error: it re-parses
```

This is a bare statement — the return value is discarded. Every existing
`fn md(args: &[&str]) -> (…, …, i32)` in this codebase (the convention this
call matches — `cli_bip388_double_wildcard.rs:35`,
`cmd_decompose_roundtrip.rs:89`, `cli_compose_hashkinds.rs:20`,
`acceptance_walks.rs:163`, all confirmed by reading each definition) returns a
plain tuple with **no panic on a non-zero exit code** — only
`.expect("invoke md")`/`.unwrap()` on the *invocation* itself (binary missing
or killed by a signal). Every existing caller of this convention destructures
the tuple and asserts on it (`assert_eq!(code, 0, …)` at
`cli_bip388_double_wildcard.rs:47` is representative). `item_7b` does neither.
As spelled, **`md encode` could return a parse error on this exact template
and the test would still pass** — the identical "test that reports a false
PASS" shape the project's severity rules call out as blocking, on the one
test Task 8 Step 5b exists to restore. (The other convention in this codebase,
`fn md() -> Command` — 19 of the 21 `fn md` definitions — takes zero
arguments; `item_7b`'s call wouldn't even compile against that convention, so
either reading of "the established `md` helper" leaves this test broken.)

→ **r6/I-1 (Important), part (a).**

### (ii) Does `item_7a` live somewhere `descriptor_to_template` and `kind1_from_vector` are both reachable?

**Yes, confirmed by RUNNING it.** Built a probe integration test at
`crates/md-codec/tests/f449_r6_item7_probe.rs`, `include!`-ing
`common/vendored.rs` exactly as Task 1 prescribes for `tests/common/liana.rs`,
and calling `md_codec::render::descriptor_to_template` (confirmed `pub fn` at
`render.rs:146`) on a decoded vendored vector:

```
$ cargo test -p md-codec --test f449_r6_item7_probe -- --nocapture
running 1 test
PROBE-R6 rendered: tr(50929b74…,{and_v(v:pk(@0/<0;1>/*),older(1)),{and_v(v:pk(@1/<0;1>/*),older(2)),and_v(v:multi_a(2,@2/<0;1>/*,@3/<0;1>/*),after(2))}})
test probe_r6_item7_shape ... ok
```

This compiles and runs cleanly from `crates/md-codec/tests/`, the exact home
Task 8's Files line and the fold's own comment (`:944`) both declare for
`item_7a`. Reachability is real. Probe deleted after the run; `git
status --porcelain` empty.

### (iii) Do the two template strings actually correspond?

**No — measured, not inferred. They have never corresponded and could not.**

`item_7a` renders `kind1_from_vector("keyed_compose_tr_nums_three_leaves")`.
The probe above shows what `descriptor_to_template` produces for that vector
**today** (kind 0, pre-stage-1b, so with the raw NUMS hex where Task 5 will
later substitute the marker — everything else about the tree shape is
unaffected by that substitution): a **nested** tap tree, **four** keys
(`@0..@3`), with `older(1)`, `older(2)`, `multi_a(2, …)` and `after(2))`.
Cross-checked against the vendored fixture itself
(`crates/md-codec/tests/vectors/keyed_compose_tr_nums_three_leaves.conformance.json`),
whose `"template"` field is byte-identical in shape to the probe's output.

`item_7b`'s literal is:

```
tr(UNSPENDABLE(liana),{pk(@0/48'/0'/0'/3'/<0;1>/*),pk(@1/48'/0'/1'/3'/<0;1>/*)})
```

— a **flat**, **two**-key tree of plain `pk()` leaves, no `older`, no
`multi_a`, no `after`. There is no substitution or rendering step that turns
the former into the latter; they are two unrelated descriptors that happen to
share the marker token and the first two of the vector's four origin paths.
The fold's own comment at `:964-965` — *"item_7a is what proves the renderer
emits this exact shape; keep the two strings in sync"* — is false on its
face, confirmed by running the renderer against the actual fixture named one
line above it.

→ **r6/I-1 (Important), part (b).**

**Net effect on r5/I-1's original concern.** The crate-boundary problem
(`item_7` calling an unreachable helper) is genuinely fixed. But the actual
render→reparse fixpoint — defined at spec authority by
`format/text.rs:269-274` (RUN, read at source): render `X` → `r1`, reparse
`r1` → `d2`, re-render `d2` → `r2`, assert `r1 == r2` — is still not
implemented for `tr` kind 1 anywhere in this plan. `item_7a` proves the
renderer *can* emit the marker on *some* input; `item_7b`, even if its
assertion were fixed, would only prove that *some* marker-bearing template
parses — never that what the renderer for the vector this plan uses
everywhere else (`keyed_compose_tr_nums_three_leaves`) actually produces is
reparseable. SPEC §8 item 7's charge — "extend that corpus" — is not met by
either half, singly or combined.

---

## Q3 — staleness, the last sweep

- **Every test's stated home vs. what it calls:** checked systematically.
  `item_7a`'s home (md-codec, `tests/liana_unspendable.rs`) matches what it
  calls (`descriptor_to_template`, `kind1_from_vector` — both md-codec-local,
  confirmed reachable in Q2(ii)). `item_7b`'s home (md-cli,
  `tests/liana_input_side.rs`, per the fold's own comment) matches what it
  calls (`md(...)`, a CLI-shelling helper — consistent with md-cli, though
  broken per Q2(i)). All other `kind1_from_vector` call sites
  (`:221,306,312,540,753,763,841,859,926`) remain under md-codec Files lines,
  unmoved. No mismatch beyond the already-carried r4/M-3.
- **Task 7's fixture-count sentence (`:745`, "three fixtures") vs. what it
  defines:** unchanged from r5 — still two `fn` definitions in that block
  (`tr_liana_with_sortedmulti_a_leaf`, `tr_liana_at_use_site`); the third
  (`in_crate_tr_liana_with_sortedmulti_a_leaf`) sits 64 lines earlier. r5/N-1,
  carried, non-blocking.
- **No task references a helper that has moved or been deleted.** Re-grepped
  every `kind1_from_vector(` and the single `descriptor_to_template(` call
  site (list above) — all sit under Files lines that grant them, none in a
  relocated block missing its helper.
- **Self-Review's helper list (`:1039`) vs. what is actually defined:** still
  false for the same three names r5/M-1 found (`wsh_wrapping_tr_liana`,
  `all_nums_tr`, `encode_payload_at_forced_version` — zero `fn` definitions in
  the plan). The fold did not touch this line; carried unchanged.

Nothing new contradicts the spec beyond Q2's finding. The eight carried
r4-vintage staleness items (the `23`/`22` tr_count line, the `55`-vs-`TWELVE`
count, Step 1's "as UNIT tests" heading, Task 7's Files-line omission, the
`encode::` nextest filter, Task 8's Files line — same as r4/M-3 — , Task 4's
Files line, the chunk.rs version-count line) are outside this fold's one
hunk and remain exactly as r5 measured them — re-confirmed by `git diff
--stat 4b797fd3..HEAD` touching only the plan `.md` and the r5 report.

---

## Q4 — the gate

Re-ran `./scripts/plan-api-check.sh design/IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md`:

```
all extracted symbols resolve

EXEMPTED BY ALLOW (22) -- each MUST be a symbol this plan CREATES.
   Error::NetworkRequiredForUnspendable / Error::NonMinimalWireVersion /
   Error::UnspendableNotRootTr / Error::UnspendableUseSiteNotCanonical /
   Error::UnspendableWithSortedMultiA / all_cases / all_kind0_tr_vectors /
   all_nums_tr / case [ALSO EXISTS in repo -- is it really created here?] /
   compressed_33 / encode_payload_at_forced_version /
   in_crate_tr_liana_with_sortedmulti_a_leaf / is_supported_version /
   kind1_from_vector / liana_unspendable_xpub / md_err /
   to_miniscript_descriptor_multipath_with_network / tr_liana_at_use_site /
   tr_liana_with_sortedmulti_a_leaf / validate_unspendable_shape /
   wire_version / wsh_wrapping_tr_liana

checked 88 candidate symbols from 22 rust blocks
```

(88 vs. r5's 86 — expected: the split added `item_7a`'s own
`kind1_from_vector` and `descriptor_to_template` occurrences; not a defect.)

**Direct answer: no.** Checked all 22 against the plan's prose for language
claiming pre-existence (`grep -n "existing"` cross-referenced against each
name): the one hit (`wire_version` near *"do NOT change the existing
signatures"* at `:513`) refers to the pre-existing `to_miniscript_descriptor*`
entry points, not to `wire_version` itself, which the plan defines (`:267`,
unaffected by this fold, confirmed by r4/r5 and re-confirmed here). `md`,
which the fold's diff uses inside `item_7b`, is **not** in the ALLOW list at
all — it resolves because a `fn md` of some shape already exists in the repo
(19+ instances, two incompatible signatures, see Q2(i)) — this is the gate's
already-documented blind spot (it matches symbol names, not signatures or
crate-root reachability), not a new defect the fold introduced. No exempted
symbol is described by the plan as already existing.

---

## Ready for implementation: no.

**One Important blocks: r6/I-1**, replacing r5/I-1 (r5/I-1's specific
citation is fixed; the property it was protecting is not). Two independent,
RUN-verified problems in the same two tests: (a) `item_7b`'s assertion is a
bare, discarded function call — the test cannot fail regardless of whether
`md encode` actually errors on the shown template, matching every existing
`fn md(args: &[&str])` convention in this codebase; (b) `item_7a`'s actual
render target (`keyed_compose_tr_nums_three_leaves`, a nested four-key tree
with `older`/`multi_a`/`after`, confirmed by running the renderer against it
today) and `item_7b`'s literal (a flat two-key plain-`pk` tree) are not the
same descriptor and never could render into one another — the plan's own
"keep the two strings in sync" comment is false. Together, neither the letter
nor the spirit of SPEC §8 item 7 (the render-reparse fixpoint, defined at
`format/text.rs:269-274`) is implemented for `tr` kind 1 by this fold.

Everything else this round touched is unaffected and stands as r5 measured
it: 8 of 9 carried findings (r4/M-1..M-4, r4/N-1..N-2, r5/M-1, r5/N-1) remain
open, all Minor/Nit, all non-blocking per project severity rules, all outside
this fold's single hunk. The gate (Q4) still shows no false "already exists"
claims among the 22 exemptions. Fix suggestion, not prescriptive: give
`item_7a` an exact-string assertion against `item_7b`'s literal (or vice
versa — derive one from the other, e.g. hand-construct the two-key flat
descriptor `item_7a` renders instead of reusing the four-key vector, or build
`item_7b`'s expected string from a value both files can read), and make
`item_7b` actually assert on `md`'s exit code and stderr.
