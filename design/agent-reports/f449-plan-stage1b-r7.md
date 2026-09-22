# R7 closing gate — `IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md` (r8 fold)

**Verdict: 0 Critical / 2 Important (NEW) / 5 Minor (carried) / 3 Nit (carried) — NOT GREEN.**

Reviewer: independent agent, sonnet. Scope: the r8 fold, isolated as
`9fc748dc..3cabae02` (the commit range on top of what `f449-plan-stage1b-r6.md`
already reviewed at `4b797fd3..9fc748dc`) against
`design/agent-reports/f449-plan-stage1b-r6.md`. Repo built and RUN at
`descriptor-mnemonic` `37367c1f` (clean), `cargo clippy 0.1.85 (4d91de4e48
2025-02-17)` confirmed. `md` was rebuilt from that exact SHA into a scratch
`CARGO_TARGET_DIR` (per "keep local binaries current" — the pre-existing
`~/.cargo/bin/md` was not trusted) and invoked directly, no probe files
created. `git status --porcelain` is empty in both repos, confirmed before
writing this file.

r6/I-1 (item_7's assertions don't bind, and item_7a/item_7b never
corresponded) **is fixed** — both parts, RUN-verified below. But the same
sweep that fixed it exposed **two new, previously-uncaught compile-blockers**
in the *neighbouring* Task 6 tests the fold also touched: `case(...)` is
called from two tests that (per the plan's own stated rule) must live in
md-cli, but is defined only reachable from md-codec (`tests/common/liana.rs`)
— the identical unreachable-fixture defect class r5/I-1 found once already,
recurring at a different symbol — and `ORIGINLESS_SPENDABLE_TR` is referenced
and never defined anywhere in the plan. **These block.**

---

## Q1 — status of every r6 finding (9 items: 1 Important, 5 Minor, 3 Nit)

| # | status | evidence |
| --- | --- | --- |
| **r6/I-1** (item_7b's assertion doesn't bind; item_7a/item_7b template mismatch) | **ADDRESSED** | Both parts. See Q2 below — RUN-verified. |
| r4/M-1 (§2 walk needs `use miniscript::ForEachKey;`) | **NOT ADDRESSED** | `grep -n "ForEachKey"` → zero matches. Outside the diff (Task 4, untouched). |
| r4/M-2 (`tap_tree.leaves()` vs. `script_tree: Option<…>`) | **NOT ADDRESSED** | Still literal at `:431-436`, byte-unchanged. Outside the diff. |
| r4/M-3 (Task 8 Files line md-codec-only vs. Step 5/5b md-cli) | **NOT ADDRESSED, and the fold made it less specified, not more** | Task 8's Files line (`:830`) still reads `crates/md-codec/tests/liana_unspendable.rs (extend)` only. r6 found the fold's own comment named the missing file (`crates/md-cli/tests/liana_input_side.rs`, at the old `:953`). **That filename is now gone.** `grep -n "liana_input_side"` → zero matches anywhere in the plan. The current comment at `:939` says only `// LIVES IN crates/md-cli/tests/` — a directory, not a file. So the gap r4/M-3 named is still open, and the one clue r6 had for closing it was deleted without replacement. |
| r4/M-4 (5 ALLOW entries the plan creates nothing for) | **NOT ADDRESSED** | `grep -n "fn compressed_33\|fn md_err\|fn all_nums_tr\|fn wsh_wrapping_tr_liana\|fn encode_payload_at_forced_version"` → zero matches. Unaffected by the diff. |
| r5/M-1 (Self-Review's own helper list is false) | **NOT ADDRESSED** | Line `:1048` still claims `wsh_wrapping_tr_liana`/`all_nums_tr`/`encode_payload_at_forced_version` are "currently defined here." `grep -n "fn wsh_wrapping_tr_liana\|fn all_nums_tr\|fn encode_payload_at_forced_version"` → zero. Outside the diff. |
| r4/N-1 (`EXEMPTED BY ALLOW` doesn't subtract `defined`) | **NOT ADDRESSED** | `git diff --stat 9fc748dc..HEAD -- scripts/plan-api-check.sh` → empty; the script is untouched. |
| r4/N-2 (`case`'s benign collision has nowhere to be recorded) | **NOT ADDRESSED, and no longer just benign** | Re-ran the gate (Q4) — `case [ALSO EXISTS in repo -- is it really created here?]` still prints with no way to mark it checked. Worse: this round's own r7/I-1 (below) shows the collision the gate treats as benign is actually hiding a real unreachable-fixture defect in two tests. |
| r5/N-1 (Task 7's "three fixtures" sentence defines two) | **NOT ADDRESSED** | `:749` still reads *"The three fixtures these tests use, defined here:"* over a block with exactly two `fn` definitions (the third, `in_crate_tr_liana_with_sortedmulti_a_leaf`, sits 64 lines earlier at `:685`). Outside the diff. |

**8 of 9 carried findings are byte-unchanged** outside the fold's three hunks
(Global Constraints, Task 6's three `md(...)` call sites, Task 8 Step 5b's
`item_7`). r6/I-1 is the only one the fold acted on, and it fully closes.

---

## Q2 — the restructured fixpoint, and the sweep

### (a) Can every one of `item_7`'s assertions fail? RUN evidence.

`item_7_the_render_reparse_fixpoint_covers_tr_kind_1` (`:956-978`) is now one
test: encode a literal template → `md decode` (render) → `md encode` again →
assert the two md1 strings are identical, with `assert_eq!(code, 0, …)` bound
at every step.

**Is `md decode`'s default stdout really the BIP-388 template, not JSON?**
RUN, from a freshly built `md` at `37367c1f`:

```
$ md decode --help
...
      --json
          Emit a structured JSON object on stdout instead of the plain BIP-388
          wallet-policy template string
...
EXAMPLES:
  $ md decode md1yqpqqxqq8xtwhw4xwn4qh
  wpkh(@0/<0;1>/*)
```

Confirmed: default stdout (no `--json`) is the plain template. `item_7`'s
Step 2 comment ("`md decode` emits the plain BIP-388 template by default")
is correct.

**Would this test pass today, before Task 6 exists?** RUN, the exact literal
from Step 1:

```
$ md encode "tr(UNSPENDABLE(liana),{pk(@0/48'/0'/0'/3'/<0;1>/*),pk(@1/48'/0'/1'/3'/<0;1>/*)})" --path bip48
md: template parse error: miniscript parse failed: internal key must have no children, but found 1
exit code: 1
```

**No — it fails at Step 1**, `assert_eq!(code, 0, "encode failed: {err}")`
would panic immediately, today. This is **correct**, not a defect: Task 6
(the template grammar's marker substitution) doesn't exist yet at
`37367c1f`, and Task 8 — where `item_7` lives — is sequenced after Tasks 1-7
in the plan, so by the time this test is reached in implementation order,
Tasks 5 and 6 should already have landed. A premature GREEN here would have
been the defect; a RED-today, RED-until-Task-6 test is exactly what TDD
sequencing requires.

**Is the final `assert_eq!` a genuine fixpoint or trivially true?** Genuine —
nothing forces `md1 == md1b` syntactically. It compares the bytes from
encoding the hand-written literal (Step 1) against the bytes from
re-encoding whatever the renderer produced from decoding Step 1's output
(Step 3); it can fail if rendering drops information, reorders paths, or
round-trips to a different-but-equivalent form. All six assertions in the
test (`code==0` ×3, the two `rendered.contains(...)` checks, and the final
`assert_eq!`) are independently falsifiable under distinct failure modes —
none is dead code.

**Net effect on r6/I-1.** Part (a) (discarded return) is fixed: `item_7` now
binds and asserts at every `md(...)` call. Part (b) (item_7a's actual render
target never matched item_7b's literal) is fixed by construction, not by
patching the comparison: the restructured test no longer touches
`kind1_from_vector`/`descriptor_to_template` or the four-key vector at all —
it feeds one literal template through encode→decode→encode and compares the
result to itself, so there is no second, independently-derived string to go
out of sync with. **r6/I-1 is closed, both parts.**

### (b) The sweep — grepped, not counted by eye

```
$ grep -n 'md(&\[' design/IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md
27:  (prose, the Global Constraints bullet itself)
597, 609, 625, 960, 965, 975
```

Exactly **six** real `md(&[...])` call sites. All six destructure into
`(out/md1/rendered/md1b, err, code)` and are immediately followed by
`assert_eq!(code, 0, "…: {err}")`. Confirmed line-by-line: `:597→599`,
`:609→610`, `:625→626`, `:960→961`, `:965→966`, `:975→976`. Matches the
plan's own claim exactly.

The four `md_err(&[...])` sites (`:483, :487, :617, :911`) are all bound to a
variable or used directly inside `assert!`/`assert_eq!` — none discarded.

A broader heuristic scan of every ```rust block for bare statement-calls
(script in scratchpad, pattern: lines ending `;` inside a rust fence that
aren't `let`/`assert`/`panic`/control-flow) found no other discarded
CLI-invocation or `expect()`-less result. The two non-obvious hits
(`w.write_bits(...)` at `:326,330`, `leaf.miniscript().for_each_key(...)` at
`:436`) are legitimate void/closure-driven calls, not the `md()`-discard
class.

**But the same three touched call sites also surfaced two problems the
sweep's mechanical grep can't see, because they aren't about binding a
tuple — they're about whether the surrounding code compiles at all:**

**r7/I-1 (Important) — `case(...)` is called from tests that must live in
md-cli, but is defined only reachable from md-codec.**
`decompose_RECOGNISES_a_real_liana_descriptor_and_gives_it_no_slot` (`:596`,
one of the three tests this fold's sweep just fixed) and
`md_encode_refuses_a_literal_xpub_in_the_internal_key_position_CLEANLY`
(`:616`, untouched by this fold but in the same task/file) both call
`case("preset-kofn-recovery-tr")`. Both shell out via `md`/`md_err`, so per
the plan's own repeated rule (stated three times — Task 4 `:452-456`, Task 7
`:659`, Task 8 `:939-941` — "it shells out to the `md` binary, and md-codec
has no CLI runner in dev-deps and no `CARGO_BIN_EXE_md`") they must live in
**md-cli**'s `tests/`. But `case`/`Case` is produced *only* by Task 1
(`:83-85`): "`fn case(name: &str) -> Case`... in `tests/common/liana.rs`,
pulled in with `include!(concat!(env!("CARGO_MANIFEST_DIR"), …))` —
**examples and tests are separate crate roots** and `use` across them does
not compile," and Task 1's Files line creates that file at
`crates/md-codec/tests/common/liana.rs` — md-codec only. `grep -rn "^fn
case(\|^struct Case"` over both crates in the real repo → zero (confirms
nothing pre-existing fills this). The plan never states a second copy, a
shared path, or a re-derivation for md-cli. This is the **identical defect
class** r5/I-1 found for `kind1_from_vector`/`descriptor_to_template`
(unreachable across the crate boundary), recurring at a different symbol,
in the very task this fold's own sweep touched — and it is why `case`
appears in the gate's ALLOW list (Q4) with the collision note r4/N-2 already
flagged as unresolved; that collision is not benign, it is masking this.

**r7/I-2 (Important) — `ORIGINLESS_SPENDABLE_TR` is referenced and never
defined.** `decompose_keeps_TODAYS_behaviour_for_an_origin_less_key_that_is_NOT_lianas`
(`:609`, the second test this fold's sweep touched) reads
`md(&["decompose", ORIGINLESS_SPENDABLE_TR, "--emit", "template"])`.
`grep -n "ORIGINLESS_SPENDABLE_TR"` over the whole plan → **one** hit, the
use site. No `const`, no `fn`, no prose defining what descriptor string this
should be. `plan-api-check.sh`'s 22-entry ALLOW list (Q4) does not include
it either — the gate's symbol extraction evidently doesn't track bare
SCREAMING_SNAKE identifiers passed as arguments, so this slips past the one
mechanical check the plan leans on. As written, this test cannot compile.

Both are genuine compile-blockers, not weak assertions — the same shape of
defect the build-gate rule exists to catch, in a test suite that has already
paid for this exact class once (r5/I-1) and fixed it there without noticing
it recurs two tests over.

### (c) The `md()` helper contract citation

```
$ sed -n '30,45p' crates/md-cli/tests/cli_bip388_double_wildcard.rs
fn md(args: &[&str]) -> (String, String, i32) {
    let out = StdCommand::new(assert_cmd::cargo::cargo_bin("md"))
        .args(args).output().expect("invoke md");
    (String::from_utf8_lossy(&out.stdout).into_owned(),
     String::from_utf8_lossy(&out.stderr).into_owned(),
     out.status.code().expect("md exited normally"))
}
```

**Right.** Signature `(String, String, i32)` at line 35, body through 44 —
"35-45" is accurate. It does not panic on a non-zero exit code (only if the
binary can't be invoked, or is killed by a signal with no exit code) —
matches the bullet's claim, and matches how `item_7`/Task 6 destructure it
(`(stdout, stderr, code)` order, consistently).

**Minor, new, non-blocking accuracy note:** the bullet states this as *the*
convention for "`md(...)` in md-cli's tests," but `grep -n "fn md(args"
crates/md-cli/tests/*.rs` in the real repo shows **three** distinct
signatures, not the one cited: `cli_bip388_double_wildcard.rs:35` is
`(String, String, i32)`; `acceptance_walks.rs:163` and
`cmd_decompose_roundtrip.rs:89` are `(i32, String, String)` — **code
first**; `cli_compose_hashkinds.rs:20` is `(bool, String, String)`. This
doesn't affect the plan's own new call sites (all consistently
`(out, err, code)`, matching the cited file), so it isn't blocking — but the
bullet's "the established `md` helper" framing overstates a convention that
is actually split three ways, and whichever file a future test's local `md`
helper is copied from will need to match its OWN destructuring order, not
the cited one blindly. Consistent with r6's already-noted "gate blind spot"
observation (`md` isn't in the ALLOW list, resolves by name only).

---

## Q3 — staleness, the last sweep

- **Every test's stated home vs. what it calls:** `item_7`'s home
  (md-cli, per `:939` "LIVES IN crates/md-cli/tests/") matches what it
  calls (`md(...)` only — no codec-local helper — confirmed in Q2(a)). The
  two other Task 6 tests this fold touched do **not** match: see r7/I-1
  above. Everything else is unaffected by this fold and stands as r6 left
  it.
- **Task 7's fixture-count sentence (`:749`):** unchanged — still "three
  fixtures," still two `fn` definitions in that block. Outside the diff,
  carried per r5/N-1.
- **`item_7a`/`item_7b` — do they survive as live references?** No.
  `grep -n "item_7a\|item_7b"` → **one** hit, at `:946`, inside the new
  historical comment ("draft 2: two tests, one per crate -> item_7b
  DISCARDED md()'s return…"). No live code, test name, or Files-line
  reference to either survives. Clean.
- **Self-Review's helper list (`:1048`):** still false for the same three
  names r5/M-1 found — `grep -n "fn wsh_wrapping_tr_liana\|fn all_nums_tr\|fn
  encode_payload_at_forced_version"` → zero. Untouched by this fold.
- **`plan-api-check.sh` candidate count:** 86 (down from r6's 88 — expected
  and correct: merging `item_7a`+`item_7b` back into one test removed the
  two codec-local calls the r7-label split had added, returning the count to
  r5's original 86). Not a defect.

Nothing else contradicts the spec beyond Q2's findings. The eight carried
r4-vintage staleness items r6 already catalogued (the `23`/`22` tr_count
line, the `55`-vs-`TWELVE` count, Step 1's "as UNIT tests" heading, Task 7's
Files-line omission, the `encode::` nextest filter, Task 8's Files line —
r4/M-3 above — , Task 4's Files line, the chunk.rs version-count line)
remain outside this fold's three hunks, re-confirmed by
`git diff --stat 9fc748dc..HEAD` touching only the plan `.md`.

---

## Q4 — the gate

```
$ ./scripts/plan-api-check.sh design/IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md
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

checked 86 candidate symbols from 22 rust blocks
```

Same 22-entry ALLOW list as r6 saw (86 candidates now, vs. r6's 88 — see Q3).
Re-ran `grep -n "existing"` over the whole plan and cross-checked every hit
(`:7, :22, :514, :580, :637, :733, :813, :822, :872`) against the 22 names:
the only near-match is `:514`'s *"do NOT change the existing signatures"*,
which refers to the pre-existing `to_miniscript_descriptor*` entry points,
not to `wire_version` itself (which the plan defines at `:267`, unaffected
by this fold). **Direct answer: no exempted symbol is described by the plan
as already existing** — same conclusion r6 reached, re-verified against the
current text (r8's diff didn't touch any of the 22 symbols' surrounding
prose).

`md` — used in all six sites in Q2(b) — is still not in the ALLOW list at
all; it resolves because *some* `fn md` already exists in the repo (see the
three-signatures note in Q2(c)). This is the gate's already-documented blind
spot (matches by name, not by signature or crate-root reachability), not a
defect this fold introduced.

`ORIGINLESS_SPENDABLE_TR` (r7/I-2) is **not** in the ALLOW list either, and
not flagged as an unresolved symbol — the gate's extraction evidently only
tracks call-shaped identifiers (`name(`), so a bare constant reference passes
through unseen. This is a second, previously undocumented blind spot in the
gate, worth naming for whoever next touches `plan-api-check.sh`, though
fixing the script is out of this review's scope.

---

## Ready for implementation: no.

**Two Important findings block, both new, both compile-blockers in the exact
Task 6 tests this fold's own sweep just touched:**

- **r7/I-1** — `case("preset-kofn-recovery-tr")` is called from
  `decompose_RECOGNISES_a_real_liana_descriptor_and_gives_it_no_slot` and
  `md_encode_refuses_a_literal_xpub_in_the_internal_key_position_CLEANLY`,
  both of which must live in md-cli's `tests/` (they shell out to `md`), but
  `case`/`Case` is defined only in `crates/md-codec/tests/common/liana.rs`
  and — per the plan's own stated Rust fact — unreachable across that crate
  boundary. Same defect class as r5/I-1, recurring at a different symbol,
  uncaught for at least one round.
- **r7/I-2** — `ORIGINLESS_SPENDABLE_TR`, used at `:609`, is never defined
  anywhere in the plan. Neither gap is visible to `plan-api-check.sh`
  (case-collision noise for the first, a call-only extraction heuristic for
  the second).

**r6/I-1 is fully closed** — both parts, RUN-verified: `item_7` now binds
and asserts at every `md(...)` call (Q2 a/b), and the restructuring removes
the render-vs-literal mismatch by construction rather than by patching the
comparison (Q2 a). It correctly fails today (RED, pre-Task-6), which is the
required TDD state, not a defect.

Everything else stands exactly as r6 measured it: 8 of 9 carried findings
(r4/M-1..M-4, r4/N-1..N-2, r5/M-1, r5/N-1) remain open, all Minor/Nit, all
non-blocking per project severity rules, all outside this fold's three
hunks — with one exception worth flagging even though it doesn't change its
severity: r4/M-3's Files-line gap is now *less* specified than r6 found it,
because the fold's restructuring deleted the one comment (`liana_input_side.rs`)
that named the missing file, and replaced it with a directory-only note.

Fix suggestion, not prescriptive: for r7/I-1, either give `case()` a second,
md-cli-side definition (Task 1's own `:186` commit line would need a second
file), or — mirroring how this same fold just resolved `item_7a`/`item_7b` —
drop `case(...)` from these two tests and inline the vendored descriptor
string as a literal, the way `item_7`'s template now is. For r7/I-2, define
`ORIGINLESS_SPENDABLE_TR` as a `const` (a real origin-less spendable tr
descriptor, per G-8's own description) somewhere reachable from the same
file, or replace it with an existing case()-independent literal.
