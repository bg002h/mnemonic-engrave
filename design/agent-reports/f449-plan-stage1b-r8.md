# R8 closing gate — `IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md` (r9 fold)

**Verdict: 0 Critical / 0 Important / 8 Minor (5 carried + 3 NEW) / 3 Nit (carried) — GREEN on the blocking bar.**

Reviewer: independent agent, sonnet. Scope: the r9 fold, `3cabae02..ee4836c7`, against
`design/agent-reports/f449-plan-stage1b-r7.md` (which reviewed `9fc748dc..3cabae02`).
Repo built and RUN at `descriptor-mnemonic` `37367c1f` (clean); `md` was rebuilt from
that exact SHA into a scratch `CARGO_TARGET_DIR` (per "keep local binaries current" —
the pre-existing `~/.cargo/bin/md` (0.17.0, unpinned) was not trusted for command
output, though its version banner happens to match) and invoked directly, no probe
files left behind. `cargo clippy` was not needed this round (no Rust source changed,
only the plan `.md` and `scripts/plan-api-check.sh`). `git status --porcelain` is
empty in both repos, confirmed before writing this file.

Both of r7's Importants are closed, both RUN-verified below. My own independent
sweep (Q2c) found three new, non-blocking Minors — all documentation/staleness gaps,
none a compile-blocker.

---

## Q1 — status of every r7 finding (2 Important, 5 Minor, 3 Nit)

| # | status | evidence |
| --- | --- | --- |
| **r7/I-1** (`case(...)` called from md-cli tests, defined only reachable from md-codec) | **ADDRESSED** | Task 6 Step 0 (new, `:592-628`) gives md-cli its own `fn case` in `crates/md-cli/tests/liana_cases.rs`, reading the same committed JSON via `include_str!("../../md-codec/tests/fixtures/liana/cases.json")`. RUN-verified path resolution and field coverage, Q2(a). |
| **r7/I-2** (`ORIGINLESS_SPENDABLE_TR` referenced, never defined) | **ADDRESSED, with one new non-blocking caveat** | Same Step 0 adds `const ORIGINLESS_SPENDABLE_TR: &str = concat!(...)`. It compiles as a `const`. Its own illustrative *values* do not parse — see Q2(b), filed as **r8/M-3** (Minor, not Important: self-detecting via the test's own `assert_eq!(code, 0, ...)`, and the plan explicitly disclaims the values as illustrative). |
| r4/M-1 (§2 walk needs `use miniscript::ForEachKey;`) | **NOT ADDRESSED** | `grep -n "ForEachKey"` → zero matches. Outside the diff (Task 4, untouched). |
| r4/M-2 (`tap_tree.leaves()` vs. `script_tree: Option<…>`) | **NOT ADDRESSED** | Still literal at `:429-436`, byte-unchanged. Outside the diff. |
| r4/M-3 (Task 8 Files line md-codec-only vs. Step 5/5b md-cli; item_7's home is a directory, not a file) | **NOT ADDRESSED** | Task 8's Files line (`:868`) still reads `crates/md-codec/tests/liana_unspendable.rs (extend)` only. The reworded comment at `:979-984` ("The test below lives in `crates/md-cli/tests/`, not md-codec's...") still names only a directory. `grep -n "liana_input_side"` → zero matches anywhere in the plan (confirmed absent, same as r7 found). |
| r4/M-4 (5 ALLOW entries the plan creates nothing for) | **NOT ADDRESSED** | `grep -n "fn compressed_33\|fn md_err\|fn all_nums_tr\|fn wsh_wrapping_tr_liana\|fn encode_payload_at_forced_version"` → zero matches. Unaffected by the diff. |
| r5/M-1 (Self-Review's own helper list is false/incomplete) | **NOT ADDRESSED, and now also incomplete in a new way** | `:1093` still claims `wsh_wrapping_tr_liana`/`all_nums_tr`/`encode_payload_at_forced_version` are "currently defined here" (zero `fn` matches, unchanged). It also was **not updated** for this fold's new helpers — filed as **r8/M-2**. |
| r4/N-1 (`EXEMPTED BY ALLOW` doesn't subtract `defined`) | **NOT ADDRESSED** | `git diff --stat 3cabae02..HEAD -- scripts/plan-api-check.sh` shows only additions (30 lines, all appended after the `EXEMPTED BY ALLOW` block); that block's own logic is untouched. |
| r4/N-2 (`case`'s benign collision has nowhere to be recorded) | **NOT ADDRESSED, but now less risky** | The gate still prints `case [ALSO EXISTS in repo -- is it really created here?]` with no way to mark it checked. But the collision is confirmed benign again this round: the two real repo hits are `crates/md-cli/src/seat/directive.rs:282` and `crates/md-cli/src/seat/matching.rs:277`, private `fn case` in unrelated production modules — no scope overlap with the new `crates/md-cli/tests/liana_cases.rs::case`. |
| r5/N-1 (Task 7's "three fixtures" sentence defines two) | **NOT ADDRESSED** | `:787` still reads *"The three fixtures these tests use, defined here:"* over a block with exactly two `fn` definitions (the third, `in_crate_tr_liana_with_sortedmulti_a_leaf`, sits ~64 lines earlier). Outside the diff. |

**8 of 8 carried findings (5M + 3N) are byte-unchanged** outside the fold's two
hunks (Task 6 Step 0's insertion; Task 8's fence-split of item_2/item_7). Both of
r7's Importants are the only things this fold acted on, and both close.

---

## Q2 — the recurring class

### (a) The relative path and the `Case` struct subset — RUN evidence

**Path resolution.** `include_str!`/`include!` resolve relative to the directory
containing the file that invokes them (not the crate root). Simulated the exact
geometry with the real repo tree:

```
$ python3 -c "
import os
base = os.path.abspath('crates/md-cli/tests')
rel = '../../md-codec/tests/fixtures/liana/cases.json'
resolved = os.path.normpath(os.path.join(base, rel))
expected = os.path.abspath('crates/md-codec/tests/fixtures/liana/cases.json')
print('resolved:', resolved)
print('MATCH:', resolved == expected)
"
resolved: /scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/tests/fixtures/liana/cases.json
MATCH: True
```

`crates/md-cli/tests/liana_cases.rs`'s `../../` climbs `tests/` → `md-cli/` → `crates/`,
then descends `md-codec/tests/fixtures/liana/cases.json` — exactly where Task 1 Step 1
writes the file. **Resolves correctly.**

**`Case` struct as a subset.** md-cli's Step 0 struct declares six fields: `name`,
`accepted`, `descriptor_with_checksum`, `expected_xpub`, `liana_receive`,
`liana_change`. Task 1 Step 1's prose states the JSON emits: `name`, `accepted`
(bool), `leaf_tlv_hex`, `leaf_pubkeys_hex`, `expected_xpub`,
`descriptor_with_checksum`, `liana_receive[3]`, `liana_change[3]` — a superset.
Checked the **inverse** direction the dispatch asked for (missing fields serde
needs but the JSON won't supply): all six of md-cli's fields appear verbatim in
Task 1's emitted-field list, and both plans type `liana_receive`/`liana_change`
identically as `Vec<String>`. **No missing field** — deserialization would not
fail for this reason. (Serde ignores JSON fields the struct doesn't declare by
default, so `leaf_tlv_hex`/`leaf_pubkeys_hex` being absent from md-cli's struct
is fine — that's the subset relationship working as intended, not a gap.)

### (b) `ORIGINLESS_SPENDABLE_TR` — is it well-formed and origin-less? RUN evidence

Built `md` fresh at `37367c1f` into a scratch target, then ran the plan's exact
literal (concatenated):

```
$ md decompose "tr(xpub6DXuQW1Q2JpZyweiMewTZuMPvjG8hKhV2qoF6wL9VFxsMBExtbfqAAoR4oMG4GyxFzVdfas1v2eAdfLxyjc4Ceo5B6w6zT/<0;1>/*,and_v(v:pk([73c5da0a/48'/0'/1'/3']xpub6DXuQW1Q2JpZzLV9kZbjmB9NcnQ7UmM8ZMDkM5yCkXqUFEJrJvVXmxfFrnBGJmFWVSYJVPNrVbTQQhZ8ryFHqPzWhBXAFsFCTGgCkeS/<0;1>/*),older(26280)))" --emit template
md: decompose: this is not a descriptor md can parse: base58 encoding error. ...
exit code: 1
```

**Malformed — confirmed, and confirmed why.** Real xpubs are 111 base58 characters.
The plan's two illustrative xpubs measure **98** and **107** characters — both
truncated:

```
$ python3 -c "print(len('xpub6DXuQW1Q2JpZyweiMewTZuMPvjG8hKhV2qoF6wL9VFxsMBExtbfqAAoR4oMG4GyxFzVdfas1v2eAdfLxyjc4Ceo5B6w6zT'))"
98
$ python3 -c "print(len('xpub6DXuQW1Q2JpZzLV9kZbjmB9NcnQ7UmM8ZMDkM5yCkXqUFEJrJvVXmxfFrnBGJmFWVSYJVPNrVbTQQhZ8ryFHqPzWhBXAFsFCTGgCkeS'))"
107
```

**But the SHAPE is right, verified independently.** Substituting two real,
correct-length xpubs into the identical wrapping (`tr(<x1>/<0;1>/*,and_v(v:pk([fp/path]<x2>/<0;1>/*),older(26280)))`)
parses cleanly past base58 and reaches the intended semantic layer:

```
$ md decompose "tr(xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8/<0;1>/*,and_v(v:pk([73c5da0a/48'/0'/1'/3']xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8/<0;1>/*),older(26280)))" --emit template
md: decompose: the same extended key is used at 2 positions — ...
exit code: 1
```

(Deliberately reused the same real key twice, so this second refusal is expected
and itself confirms base58 parsing succeeded this time — it advanced past the
base58 stage to BIP-388's pairwise-distinctness check.) The internal key carries
no `[fp/path]` prefix (origin-less, as G-8 requires) and the leaf key does carry
one — the intended shape is correct.

**Verdict: filed as r8/M-3, Minor, non-blocking.** The specific illustrative bytes
in the plan do not parse, but (1) the plan explicitly says "The exact xpubs above
are illustrative... take them from the vendored cases.json rather than transcribe,"
(2) the shape they demonstrate is verified correct, and (3) an implementer who
skips that instruction gets an immediate, loud RED from
`decompose_keeps_TODAYS_behaviour_for_an_origin_less_key_that_is_NOT_lianas`'s own
`assert_eq!(code, 0, "decompose failed: {err}")` — this is self-detecting, not a
silent wrong-shape teaching risk.

### (c) The sweep — independent of the gate, every helper/constant against its call site's crate root

Traced all ten CLI-shelling call sites (six `md(&[...])`, four `md_err(&[...])`,
per r7's own count, re-confirmed unchanged by `grep -c 'md(&\[\|md_err(&\['
design/IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md`) against what each calls:

- `:483,:487` (Task 4, `a_slot_cannot_appear_twice…`) — uses locally-defined
  `NUMS_HEX`, no cross-file helper. Clean.
- `:634,:646,:655,:663` (Task 6 Step 1, the four new/touched tests) — three call
  `case(...)` or `ORIGINLESS_SPENDABLE_TR`, now satisfied by Step 0 in the same
  file. Clean (this is r7/I-1 and r7/I-2 closing).
- `:911` (Task 8 Step 5, `a_tr_with_no_leaf_keys...`) — literal NUMS hex inline,
  no external helper. Clean.
- `:1005,:1010,:1020` (Task 8 Step 5b, `item_7`) — self-contained `tpl` literal,
  no `case()`/`kind1_from_vector` call. Clean.

No further instance of the unreachable-helper class found among CLI-shelling call
sites. Two **new**, lower-severity gaps found in the same neighbourhood, filed
this round:

- **r8/M-1 (Minor).** Task 6's Files line (`:555`) still names zero test files —
  no `Test:` line at all, unlike every other task (Task 2: `tests/wire_version_8.rs`;
  Task 4: `tests/liana_unspendable.rs`). Step 0's in-block comment
  (`// crates/md-cli/tests/liana_cases.rs`) is the only place the new file is
  named, and nothing states explicitly that Step 1's four tests land in that
  *same* file rather than a different, unnamed one — which would silently
  reintroduce the exact class r7/I-1 just fixed, one file over. **Not blocking**:
  the filename (`liana_cases.rs`, directly under `tests/`, not under a `common/`
  subdirectory the way Task 1's shared-only `tests/common/liana.rs` is) reads
  unambiguously as a standalone test-root file meant to hold both the loader and
  the tests together — but the plan never says so in words, and should.
- **r8/M-2 (Minor).** The Self-Review's "Placeholder scan" helper list (`:1093`)
  was not updated for this fold: it still attributes `case` only to Task 1
  (md-codec's copy) and never mentions Task 6 Step 0's separate `case`/`Case`/
  `ORIGINLESS_SPENDABLE_TR` (md-cli's copy). A reader relying on that sentence
  alone would not learn there are now two distinctly-typed `case()` functions in
  two crates — exactly the ambiguity the cross-crate-reachability gate exists to
  flag, undocumented in the one place meant to be an audit trail of it.

---

## Q3 — the extended gate

Ran it fresh against the r9 plan:

```
$ export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH
$ export CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/f449-r8-target
$ ./scripts/plan-api-check.sh design/IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md
all extracted symbols resolve

EXEMPTED BY ALLOW (22) -- ...
   [unchanged 22-entry list from r7, `case` still flagged ALSO EXISTS]

UNDEFINED CONSTANTS (4) -- referenced in a rust block, defined in none:
   CHUNKED   [NOWHERE]
   DISJOINT   [exists in repo]
   UNSPENDABLE   [NOWHERE]
   WF_REDESIGN_VERSION   [exists in repo]

CALLED FROM md-cli-SHAPED BLOCKS (6) -- ...
   a_tr_with_no_leaf_keys_cannot_be_constructed
   and_v
   case
   decode_md1_string
   item_7_the_render_reparse_fixpoint_covers_tr_kind_1
   older

checked 86 candidate symbols from 24 rust blocks
```

**On the report's own target defect (`ORIGINLESS_SPENDABLE_TR`): no false
negative.** It is now correctly *absent* from UNDEFINED CONSTANTS, because
`const ORIGINLESS_SPENDABLE_TR` is genuinely defined in a rust block this
round — the check works on the case it was built for.

**Both new reports carry real false positives, each verified by hand:**

- **UNDEFINED CONSTANTS.** All 4 entries are noise, not real undefined-symbol
  claims: `CHUNKED` and `UNSPENDABLE` are substrings pulled out of *string
  literal content* (`"…dispatches as CHUNKED"`, `"UNSPENDABLE(liana)"` used as a
  marker spelling, not a Rust identifier) — the extractor strips `//` comments
  but not string bodies. `DISJOINT`'s `[exists in repo]` tag is itself
  misleading: `grep -rqE '(const|static) DISJOINT'` is unanchored and matches
  the unrelated real constant `crates/md-cli/src/bip388.rs:34 pub const
  DISJOINTNESS_RULE`, purely because "DISJOINT" is a string-prefix of
  "DISJOINTNESS_RULE" — confirmed by grepping the constant directly. Only
  `WF_REDESIGN_VERSION` is a legitimate, correctly-tagged pre-existing symbol
  (stage 1a's, used correctly here). **0 of 4 is a genuine plan defect; 3 of 4
  are string-literal noise, one of those three additionally mislabeled
  `[exists in repo]` by an accidental substring match.**
- **CROSS-CRATE REACHABILITY.** Of 6 entries, only `case` is a genuine
  test-helper needing the crate-root check (correctly reported, and correctly
  resolved this round by Task 6 Step 0). `and_v` and `older` are miniscript
  syntax fragments inside a descriptor *string literal*, not Rust calls.
  `a_tr_with_no_leaf_keys_cannot_be_constructed` and
  `item_7_the_render_reparse_fixpoint_covers_tr_kind_1` are the tests' **own**
  names (`fn name(` matches the same "identifier followed by `(`" heuristic as
  a call). `decode_md1_string` is a real, `pub fn` production API in
  `md-codec/src/decode.rs:178` (confirmed by `grep`), reached via its full path
  `md_codec::decode::decode_md1_string` — an ordinary cross-crate library call,
  not a `tests/`-root-scoped helper subject to the `include!`/`CARGO_MANIFEST_DIR`
  constraint this report exists to catch. **1 of 6 is the real thing; 5 of 6 are
  false positives** from the same two causes (string-literal content, and
  definitions vs. calls being indistinguishable to a regex).

**Is "report, don't decide" still the right call here?** Yes, but the false-positive
rate (7 of 10 lines across both new sections, this round) is high enough that an
unbriefed reader would plausibly start skimming past both sections after the
first pass — exactly the risk the dispatch asked about. The header comment's own
stated philosophy ("a gate that hides its blind spot is worse than no gate")
still holds: nothing here is *wrong* to report, and the one true positive
(`case`) is exactly the class the gate was built to catch, sitting inside the
noise rather than being lost. But the two new sections would benefit from
stripping string-literal content before extraction (matches the existing `#
strip line comments` step, just needs to also strip `"..."` spans) and from
excluding `fn `-preceded matches from the cross-crate report — neither is a
correctness bug in this round's plan, both are gate-quality follow-ups, not
filed against the plan itself.

---

## Q4 — staleness, the last sweep

- **Every test's stated home vs. what it calls:** Task 6 Step 1's four tests
  (md-cli, per Step 0's file-comment) now match what they call — `case(...)`/
  `ORIGINLESS_SPENDABLE_TR` are both defined in the same file (Q2a, Q2c). The
  ambiguity is that the *file identity* itself is asserted only by an in-block
  comment, not a `Test:` line — filed as **r8/M-1**, not a call-mismatch.
- **Task 7's fixture-count sentence (`:787`):** unchanged — still "three
  fixtures," still two `fn` definitions in the immediately-following block.
  Outside the diff, carried per r5/N-1.
- **Self-Review's helper list (`:1093`):** stale in the pre-existing way (r5/M-1,
  unchanged) **and** newly incomplete for this fold's own additions — filed as
  **r8/M-2**.
- **`plan-api-check.sh` candidate count:** 86 (same as r7's fold — the Task 6
  Step 0 insertion added one new rust block but it doesn't add any new
  *candidate* identifiers beyond what `case`/`ORIGINLESS_SPENDABLE_TR` already
  contributed as call/const sites; the Task 8 fence-split changed block count
  from 22 to 24 without changing the resolved-candidate total). Not a defect.
- **Rust-fence count:** `grep -c '```rust'` → 24, matches the gate's own "24 rust
  blocks" line exactly.

Nothing else contradicts the spec beyond the two now-closed Importants. The eight
carried r4/r5-vintage staleness items (5 Minor + 3 Nit, tabulated in Q1) remain
outside this fold's two hunks, re-confirmed by `git diff --stat 3cabae02..HEAD`
touching only the plan `.md` and `scripts/plan-api-check.sh`.

---

## Ready for implementation: yes.

**0 Critical / 0 Important.** Both of r7's compile-blockers are closed and
RUN-verified: `case()` is now reachable from md-cli via its own loader over the
shared JSON (path resolves, struct is a valid field subset — Q2a), and
`ORIGINLESS_SPENDABLE_TR` is now a defined `const` whose *shape* is verified
correct even though its specific illustrative bytes are not (self-detecting,
explicitly disclaimed — Q2b, filed as r8/M-3).

**8 Minor / 3 Nit remain, all non-blocking per project severity rules:**
r4/M-1, r4/M-2, r4/M-3, r4/M-4, r5/M-1 (carried, untouched by this fold),
plus three new from this round's independent sweep — r8/M-1 (Task 6 lacks an
explicit test-file line), r8/M-2 (Self-Review helper list not updated for this
fold), r8/M-3 (the illustrative `ORIGINLESS_SPENDABLE_TR` bytes don't parse,
though the shape is right and the gap is self-detecting). Nits: r4/N-1, r4/N-2,
r5/N-1 (carried, untouched). None of these eleven items is a compile-blocker,
a wrong result, or an unmet guarantee — they are documentation/self-audit
completeness gaps in a plan whose own gate (`plan-api-check.sh`) now
independently confirms all extracted symbols resolve.

This closes the loop on the recurring "unreachable helper" class that gated the
last four rounds: r5/I-1, r4/I-2, and r7/I-1 were the same defect at
`kind1_from_vector`/`case`; r7/I-2 was its sibling for a bare constant. All four
are now fixed, and the gate that would have caught the constant-shaped variant
earlier is in place and correctly silent on `ORIGINLESS_SPENDABLE_TR` this round.
