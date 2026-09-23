# R2 scoped re-review — the fold of `f449-plan-stage2-r1` into `IMPLEMENTATION_PLAN_f449_stage2_compose.md`

**Verdict: NOT ready for implementation.**
**Counts: 0C / 5I / 11M / 8N.**

Scope: *did this fold close r1's seven Importants without introducing an eighth,
and is the plan now implementable?* Not a fresh audit. Reviewed at
`mnemonic-engrave e8d05174` against `descriptor-mnemonic` main `25acb33c`
(`target/debug/md` at that tree reports `md 0.18.0`; `git status --porcelain`
empty before and after every probe below — one scratch test file was created,
run and deleted, and the tree is clean).

Everything marked **MEASURED** was run on this box at `25acb33c`. I did not
re-derive what the brief listed as already machine-checked: the build gate on
the folded plan, I-c's original unreachability, or the runtime-vs-`include_str!`
change.

**Headline.** Five of the seven are materially closed, and I-c is closed *well*
— I constructed the retargeted trigger and it is both reachable and live work
(measurement in §2). What blocks is the same pattern r1 named, one round on:
**this fold's own remedies carry three of the five new Importants.** I-d's fix
asserts, in the plan's own test comment, a behaviour that Task 1 Step 7's clap
declaration cannot produce. I-e's fix prescribes a one-line edit to a script
that measurably rejects it. I-c's retarget lands on a `md repair` exit-code
contract shared with three other CLIs, which the plan's own "TWO REPOS"
constraint does not cover. The remaining two are older: the Self-Review still
certifies as shipped the one fact this cycle has now gotten wrong three times,
and **nothing anywhere in the plan schedules a version bump, a CHANGELOG entry
or the md-cli exact-pin move** that stage 1b treated as gating.

---

## 1. Per-finding verdicts (brief Q1)

| r1 finding | verdict | one line |
| --- | --- | --- |
| **I-a** Task 7 Step 3 still authorises amending §8.1 | **ADDRESSED** | the conditional is deleted; the stale "Task 4 Step 3" pointer is gone and the text now names Task 5's vector. (The Self-Review's copy of the stale pointer is N-b, a different location, still open.) |
| **I-b** `plan-api-check.sh` in the wrong repo | **ADDRESSED** (primary half) | MEASURED: `descriptor-mnemonic/scripts/` holds exactly `matrix-identity-check.sh`, `phase-gate.sh`, `push-via-staging.sh`, `vendor-liana-evidence.sh`; `plan-api-check.sh` is in `mnemonic-engrave/scripts/`, which is what the constraint now says. Second half (the mnemonic-engrave baseline revision) still absent — r1 filed it separately as M-d; still open, non-gating. |
| **I-c** Task 2c's trigger unreachable, fix site wrong | **ADDRESSED** | retargeted at a version outside the accepted set; fix site corrected to `chunk.rs`. I constructed the trigger and measured it — see §2. **But the retarget lands a new Important, NEW-I-2.** |
| **I-d** Task 1b's refusal collides with Task 1's golden | **PARTIAL — the class is reintroduced** | the test no longer appends the flag to non-`tr` rows, but the comment added to justify that asserts a behaviour the prescribed clap declaration cannot produce. **NEW-I-1.** |
| **I-e** counts hardcoded in Task 4; vendoring half dropped | **PARTIAL** | half one ADDRESSED (Task 4 Steps 1–2 now derive from the corpus, and Task 5 Step 2 confirms it). Half two states the hazard and prescribes a remedy the script rejects. **NEW-I-3.** |
| **I-f** WARN keyed on preset NAME | **ADDRESSED** | Step 3 now keys on the composed shape; `policy_shape.rs` supplies exactly the predicate (`Branch.hashlocks`, `Branch.locks`), so it is implementable rather than aspirational. One Minor on how it is computed (M-i). |
| **I-g** Step 9b printed last while saying "FIRST" | **ADDRESSED** | it is Step 0, ahead of everything. Step 2's stated failure (`unexpected argument '--unspendable'`) is now *correct* as written: at Step 2 the golden rows all match and the first failure is the `tr` branch's appended flag. Minor: Step 0's justification still cites `include_str!` (M-g). |

**Brief Q2 — r1's Minors and Nits.** None has become blocking. All six Minors
(M-a … M-f) and all five Nits (N-a … N-e) remain open, exactly as the controller
chose. Carry list, so they are not lost:

- **M-a** `UnspendableKind::{Nums,Liana}` still undocumented. MEASURED: `Cargo.toml:11-12` sets `[workspace.lints.rust] missing_docs = "warn"` and `scripts/phase-gate.sh:32-33` runs `cargo clippy --locked --all-targets --all-features -- -D warnings`. **This turns the gate red on Task 1's first commit.** Two one-line doc comments.
- **M-b** Step 7 still cites `cmd/compose.rs` for the clap attribute. MEASURED: `Compose` is `crates/md-cli/src/main.rs:287`; `cmd/compose.rs:589-596` is `pub fn run`.
- **M-c** F-641's remedy still says "check the parse position".
- **M-d** no mnemonic-engrave baseline revision (now `e8d05174`).
- **M-e** Task 2 Step 2's control assertion cannot fail under the prescribed mechanism.
- **M-f** nothing stops `gen-compose-golden.sh` re-blessing the golden after the flag ships — see brief Q4a below.
- **N-a** Task 4 Step 1 still cites 8/8 for an ACCEPT-only loop.
- **N-b** Self-Review omits Tasks 1b/2c/5/6/7 and carries three stale cross-refs — and note **both of stage 2's own gate items mis-point**: §8.2's evidence legs are Task 4 (the text says Task 3, which is the pointer stub) and §8.8's live gate is Task 5 (the text says Task 4). Nothing is unscheduled; the ledger just points wrong.
- **N-c** two paragraphs share the heading "Build-gate coverage — stated, not assumed."
- **N-d** "The one risk worth naming" still describes the superseded self-comparing test.
- **N-e** `golden.len() >= 6` does not assert what its message claims — now sharper, see M-h.

---

## 2. What I verified as SOUND, so the next round does not re-derive it

### I-c's retargeted trigger: constructed, reachable, and LIVE work — MEASURED

The brief's Q4b asks whether Task 2c can close itself with a measurement. I
built the trigger rather than reasoning about it. `codex32::wrap_payload` /
`codex32::unwrap_string` are both `pub` (`codex32.rs:82`, `:139`), so a
version-12 md1 string with a valid BCH checksum is ~15 lines from a test:
unwrap a real v4 string, rewrite the version nibble in the first 5-bit symbol,
re-wrap. Produced:

```
base                md15zfdsssjjtvyyw2fdssj54qqxppcgsc276kwwfnzntuh   (v4)
V12_CLEAN           md1uzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg
V12_ONE_ERROR       md1qzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg   (pos 3, u -> q)
V4_ONE_ERROR        md1qzfdsssjjtvyyw2fdssj54qqxppcgsc276kwwfnzntuh   (pos 3, 5 -> q)
```

Then, exit codes captured without a pipe:

```
$ md repair md1qzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg      # v12, 1 correctable error
md: repair: wire-format version mismatch: got 12; accepted versions: 4, 8
exit=2                       # NO repair report, NO corrected string

$ md repair md1qzfdsssjjtvyyw2fdssj54qqxppcgsc276kwwfnzntuh      # v4 control, same error
# Repair report
#   md1 chunk 0: 1 correction at position 0: 'q' -> '5'
md15zfdsssjjtvyyw2fdssj54qqxppcgsc276kwwfnzntuh
exit=5
```

So: **the trigger is reachable, the correction IS discarded, and Task 2c is real
work.** The self-close branch is not available on the half that matters, because
Step 1's close bar is conjunctive ("keeps the correction AND reports the version
distinctly") and the first conjunct is measurably false. The bar is tight enough
to answer Q4b: it cannot be used to skip this. The fix site is also confirmed —
`chunk.rs`'s two finishing `?` operators, which I measure at **658** and **666**
(the plan says 659/666; N-f).

### The shape predicate I-f now prescribes is implementable, not aspirational

`policy_shape.rs` is `pub` and `policy_shape(d) -> PolicyShape` gives
`branches: Vec<Branch>` where `Branch.hashlocks` ("empty means no hashlock
anywhere in the branch") and `Branch.locks` ("empty means no timelock anywhere")
are exactly the two predicates Step 3 names. The fold's measured `--path`
counterexample reproduces verbatim:
`md compose --wrapper tr --path 2of3 --path 1of1,sha256=aa…aa,older=100` →
NUMS internal key, hashlocked leaf, exit 0, no preset.

### The refuse/warn split does NOT collide with `reject_nested_unspendable`

I checked the one way Task 2's split could have been quietly wrong.
`decaying-multisig` composes a **nested** taptree
(`{and_v(...older(13140)),{and_v(...older(26280)),and_v(...after(1000000))}}`,
MEASURED), and `validate_unspendable_shape` calls `reject_nested_unspendable`.
That function rejects a nested **`tr()` node** carrying `LianaUnspendable`
(`validate.rs:615-619`, `wsh(tr(UNSPENDABLE(liana),…))`), not a nested taptree
branch. So `decaying-multisig` is not refused by Task 2 Step 1 and correctly
falls to Step 3's WARN. The split stands.

### The `md()` helper the plan copies is verbatim-correct

`crates/md-cli/tests/liana_input_side.rs:33` is exactly the signature and body
the plan reproduces; `cli_compose.rs:8` is `fn md() -> Command` (assert_cmd).
The plan's account of "two helpers, different signatures" is accurate.

### `plan-api-check.sh` on the folded plan is clean

Run against `descriptor-mnemonic`: 23 candidate symbols from 5 rust blocks,
exit 0. The 9 "UNRESOLVED" entries are all std/`assert_cmd` (`args`,
`cargo_bin`, `from_utf8_lossy`, `split_once`, …) and the 4 "undefined
constants" are `CARGO_MANIFEST_DIR`, `EVERY`, `KIND`, `UNSPENDABLE` — prose and
env, not API claims. No plan defect from this gate.

---

## 3. NEW Important findings

## NEW-I-1 — the I-d fold asserts a non-`tr` refusal that Task 1 Step 7's clap declaration cannot implement, and the likely recovery shrinks the stage's regression floor

r1's I-d remedy was one sentence: *"only `--unspendable liana` is refused under
a non-`tr` wrapper; `--unspendable nums` stays accepted everywhere."* The fold
took the **other** horn and wrote it into the test:

`IMPLEMENTATION_PLAN_f449_stage2_compose.md:141-145`:

```rust
        // Explicit `nums` must equal the default -- but ONLY for `tr`.
        // Task 1b refuses the flag under wsh/sh/sh-wsh (there is no taproot
        // internal key there), so appending it to those rows would assert
        // exit 0 on an invocation this stage deliberately makes fail.
```

That says `--unspendable nums --wrapper wsh` **fails**. Task 1 Step 7 prescribes
the declaration:

```rust
#[arg(long, value_name = "KIND", default_value = "nums")]
unspendable: String,
```

MEASURED against the real CLI: `crates/md-cli/src/main.rs:858` is
`let cli = Cli::parse();` — derive only, no `ArgMatches` anywhere in the binary
(`grep -n "from_arg_matches\|ArgMatches" main.rs` → nothing). With
`default_value` and a bare `String`, **`md compose --wrapper wsh` and
`md compose --wrapper wsh --unspendable nums` deliver byte-identical state to
`cmd::compose::run`.** `ValueSource::DefaultValue` is the only thing that
separates them and it is unreachable without restructuring to
`Command::from_arg_matches` or changing the field to `Option<String>` (the
precedent for which is twelve lines up: `Compile`'s `unspendable_key:
Option<String>`, `main.rs:279`).

So Task 1b Step 1 and Task 1 Step 7 cannot both be satisfied. The two ways out:

1. **Refuse only `liana` under non-`tr`.** Then the plan's own comment is false,
   and the `if args.contains(&"tr")` guard silently deletes the explicit-nums
   equality check from every non-`tr` golden row for no reason.
2. **Refuse the whole flag under non-`tr`** as the comment says. Then, with
   `default_value = "nums"`, the refusal fires on the **default** invocation and
   every non-`tr` golden row exits non-zero. MEASURED that those rows exist and
   are numerous: of the 24 preset × wrapper cells, **14 compose at exit 0**, and
   **8 of those 14 are non-`tr`** (`wsh` × all six presets, `sh-wsh` ×
   `plain-multisig`, `sh` × `plain-multisig`). Task 1's
   `assert_eq!(code, 0, "{invocation}: {err}")` turns red on all eight — the
   stage's own regression floor, red on the commit after it was created.

Branch 2's tempting local fix is to narrow the golden to `tr` — which is
precisely r1's "dangerous recovery": a shrunken floor that still reports green,
undetectable because `golden.len() >= 6` passes on a `tr`-only golden of six.
`a-new-gate-makes-old-tests-vacuous`.

**Remedy.** Decide it in one sentence and put it in BOTH places. If the flag
(not just `liana`) is refused under non-`tr`, Step 7 must declare
`unspendable: Option<String>` with no `default_value` and `None => Nums` — and
Step 9's mutation (b) must then be re-pointed, because a mis-parse of the string
`"nums"` no longer moves the omitted-flag case and is caught only by the inner
explicit-vs-default comparison. If only `liana` is refused, delete the comment's
second sentence and say why the guard exists anyway.

## NEW-I-2 — Task 2c Step 3 requires a new `md repair` exit code, and `md repair`'s exit codes are a documented four-CLI parity contract the plan never mentions

Task 2c Step 3: *"test that the corrected payload survives AND that the exit
distinguishes 'unsupported version' from 'could not correct'."*

MEASURED, both arms, exit codes captured without a pipe:

```
$ md repair <v12, one correctable error>
md: repair: wire-format version mismatch: got 12; accepted versions: 4, 8
exit=2

$ md repair <v4, five errors -- beyond t=4>
md: repair: chunk 0 exceeds the BCH correction capacity of t=4 substitution errors; uncorrectable
exit=2
```

**They are the same exit.** They differ only in the stderr message. Satisfying
Step 3 as written therefore means adding an exit code — and
`crates/md-cli/src/cmd/repair.rs:14-18` states the contract:

> Exit codes (**D26 cross-CLI parity with `mk repair` / `ms repair` /
> `mnemonic repair`**): 0 — already valid; 5 — corrections applied
> (REPAIR_APPLIED); 2 — atomic-fail: BCH-uncorrectable / HRP-mismatch /
> parse-reject

`repair.rs:94` is a single `return Ok(2)` for every codec error. So Task 2c Step
3 is either (a) a message-only distinction — in which case it is **already
satisfied today** and the step should say so, or (b) a new exit code, which
changes a parity contract shared with three CLIs in other repos. The plan's
Global Constraints say *"TWO REPOS, and every step must say which"*; under
reading (b) this step reaches four CLIs and the constraint is silently false.

The scope delta between the two readings is roughly a ten-line codec fix versus
a cross-repo contract decision, and the plan does not choose.

**Remedy.** Say which. If message-only: Step 3 asserts the message, and Step 1's
close bar becomes "keeps the correction" alone (which I have already measured
false, so the task stays live). If a new exit code: add a step that records the
D26 parity decision and names the other three CLIs as out of scope for this
stage or in it.

## NEW-I-3 — Task 5 Step 2's regenerator remedy is not executable: the script rejects a name added to `NAMES` alone, and the stage-2 evidence lands in a directory it never reads

Task 5 Step 2 (`:436-439`) and the matching paragraph in Task 4 (`:394-397`)
both say: *"Add this case to that list in the same commit as the vector, or the
next regeneration deletes the corpus's only nested ACCEPT."* r1's remedy had a
second clause the fold dropped — *"extend `vendor-liana-evidence.sh` to read the
stage-2 evidence directory"* — and that clause is the one that makes the first
possible.

MEASURED on `descriptor-mnemonic/scripts/vendor-liana-evidence.sh`:

- It reads **only** `<engrave>/design/evidence/composer-fable-r0/fable-liana-parse-in.jsonl`
  and `…-out-v15.jsonl` (`:65-66`), each holding exactly 8 records with
  `variant == "liana-unspendable-xpub"` (counted).
- `NAMES` (`:94-103`) is one hardcoded list; **`ACCEPTED_NAMES` (`:104-109`) is a
  second one**, and `:186-190` raises `SystemExit` if a record's `ok` disagrees
  with membership.
- `:158-163` raises `SystemExit("missing evidence -- in=[…] out=[…]")` for any
  name in `NAMES` absent from either JSONL.

So "add this case to that list" as written **breaks the vendoring script
outright** on the next run: the new name is in `NAMES`, has no JSONL record, and
the script aborts. Completing the step actually requires four edits, three of
them unstated: `NAMES`, `ACCEPTED_NAMES`, a record in each of the two
`composer-fable-r0` JSONL files — while Task 5 Step 5 commits the stage-2
evidence to `design/evidence/f449-stage2/`, a directory the script never opens.

The implementer's cheap way out of a broken script is to *not* touch `NAMES`,
which restores exactly the silent-deletion defect this paragraph was written to
prevent.

**Remedy.** Either extend the script to read a second evidence directory (r1's
clause), or state the four edits explicitly and say that the stage-2 record is
appended to the `composer-fable-r0` JSONLs, accepting that the evidence
directory in Step 5 is then documentation rather than the script's input.

## NEW-I-4 — the Self-Review still certifies §4a's JSON bump as "already shipped in 1b", the fact this cycle has now recorded wrong three times, while Task 1b Step 2 schedules it as work

`:488`:

> **Spec coverage.** §9 stage 2's four items: `--unspendable` → Task 1; `md
> descriptor` kind 1, the substitution rule and **the JSON bump → already
> shipped in 1b** (RECON, measured).

The plan contradicts itself sixteen screens earlier. Task 1b Step 2 (`:266-273`)
says: *"R0 I-4: 1b shipped `unspendable_kind` on **decode**'s JSON only. `md
compose --json` still emits `internal_key_path: null` for BOTH kinds… **The
recon and the first draft of this plan both wrongly marked §4a's JSON item
done.**"* r1 re-verified the premise and I did not re-derive it.

This is the third recorded occurrence of the same false fact — the recon, the
plan's first draft, and now the plan's own coverage ledger — and the ledger is
the artifact a future reader consults to ask "what did stage 2 owe?".
`three-occurrences-is-a-wrong-shape`. It is distinct from r1's N-b, which is
about cross-reference *targets*; this is a factual claim r0 measured false and
the fold fixed in one place only. That is the r1 headline pattern, unchanged:
*the fold states a remedy in one place and leaves the contradicting text
standing in another.*

**Remedy.** One sentence: `md descriptor` kind 1 and the substitution rule
shipped in 1b; **§4a's JSON bump is HALF shipped — decode's side in 1b,
compose's side in Task 1b Step 2.**

## NEW-I-5 — no task anywhere schedules the version bump, the CHANGELOG entry, or the md-cli exact-pin move, and the plan does not list them as owned elsewhere

MEASURED:

- `crates/md-cli/Cargo.toml:28` is `md-codec = { path = "../md-codec", version = "=0.46.0" }` — an **exact** pin.
- Current versions: md-codec `0.46.0`, md-cli `0.18.0`.
- `CHANGELOG.md` opens with *"All notable changes to `md-codec` and `md-cli` are documented in this file"* and carries `## md-cli [0.18.0] — 2026-09-22` describing stage 1b's CLI surface in detail.
- **Stage 1b's own plan devoted a whole task to this and gated on it.** `design/IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md:52` — *"**G-3** The version bump is **0.46.0, not 0.45.2**… **No md-codec release may be tagged until this lands.**"* — and `:1067` `## Task 10: §4b's doc retirement, the version bump, and the CHANGELOG`, whose Step 2 records the mechanism verbatim: *"`md-cli/Cargo.toml:28` pins `=0.45.1`; the moment md-codec moves, that is unsatisfiable and cargo fails to resolve **before rustc runs**, so a split commit is red on its own."*
- `SPEC_liana_unspendable_internal_key.md:873-875` expects a stage-2 release downstream: *"The toolkit pins md-codec by git tag… it is **in scope for this cycle** and gets its pin bump and a golden refresh **after stage 2**."*
- `grep -ni "changelog\|version bump\|0\.47\|0\.19"` over the plan → **nothing**. The Self-Review's *"Deliberately absent, owned elsewhere"* list names the Go port, device work, `me`/F-635 and `/sh2`. It does not name this, so it is an omission, not a deferral.

Stage 2 adds a user-visible CLI flag, a new `pub enum` to md-codec's `compose`
module, and threads a parameter through `Composed`/the lowering entry
(`compose::compose` and `compose::compose_with` are both `pub`;
`lower_tr` is only `pub(super)`, so that one is not a public break). Under the
repo's stated pre-1.0 convention that *"the second component (`0.X`) is the
breaking-change axis"*, that is at minimum a md-codec minor and a md-cli minor,
with the `=` pin moved **in the same edit** and two CHANGELOG entries.

**Remedy.** Add a Task 8 mirroring stage 1b's Task 10: bump md-codec, move
md-cli's exact pin in the same commit, write both CHANGELOG entries, and say
whether the toolkit's pin bump is stage 2's or the cycle's.

---

# MINOR

## M-a … M-f — all six r1 Minors stand, unfolded by choice

Re-confirmed, not re-argued; see the carry list in §1. **M-a is the one that
turns `phase-gate.sh` red on the first commit** and is two doc comments.

## M-g — Step 0's justification cites the `include_str!` the same fold removed

`:70-77` still reads *"Step 1's `include_str!` cannot compile without it"* and
calls that the first of *"Two reasons"*. The fold replaced the include with a
runtime read in the very next hunk, so reason one is now false: a missing golden
is a test failure, not a compile error. The ordering is still right for reason
two (provenance), so the step survives — but a reader who trusts Step 0's prose
over Step 1's code block writes `include_str!`, which restores the stubbing
invitation C-2 was raised against.

## M-h — Step 0's "every preset and every wrapper" is a 24-cell matrix of which 10 cannot be recorded, and nothing says what the generator does with them

MEASURED with the six presets' own valid-parameter fixtures (taken from
`cmd/compose.rs:790-799`'s `every_preset_name_parses_with_some_valid_parameters`,
not invented): **14 of 24 cells exit 0.** All six presets compose under `tr` and
`wsh`; under `sh-wsh` and `sh` only `plain-multisig` does — the rest fail with
*"legacy wrappers hold one plain sorted multisig only"*, exit 1.

Task 1's loop asserts `code == 0` for **every** golden row, so the generator must
skip the ten. The plan does not say so, and `golden.len() >= 6` (r1's N-e)
cannot tell a correct 14-row golden from a narrowed 6-row one. Two lines fix
both: have the generator record only exit-0 invocations, and assert coverage of
all four wrappers and all six preset names rather than a bare length.

## M-i — Step 3's predicate computed over `branches` alone mislabels a `tr` with a real internal key as Liana-refused, and would fire alongside Step 4's warn

Step 3 says *"a hashlock leaf present, or **no non-timelocked path**"*. In
`PolicyShape`, the taproot key path is **not** a `Branch` — it is
`key_path: KeyPathKind`, and `branches` holds only tapscript leaves
(`policy_shape.rs:203-206`, `:245-248`). So
`!branches.iter().any(|b| b.locks.is_empty())` is **true** for
`simple-timelocked-inheritance`, whose only leaf is timelocked and whose primary
path is the key path. MEASURED: `md compose --wrapper tr --preset
simple-timelocked-inheritance,older=26280 --json` → `"internal_key_path": 0`.

`md compose --wrapper tr --preset simple-timelocked-inheritance,older=26280
--unspendable liana` would then emit Step 4's row-3 warn *and* Step 3's "Liana
will refuse this" — on the canonical Liana shape (unlocked primary + one
timelocked recovery), which lifts to `or(Key(IK), tree)` and is exactly what
Liana accepts. One clause fixes it: the "no unlocked path" half must count the
key path when it carries a real key, and Steps 3 and 4 are mutually exclusive.

## M-j — Task 4's home is already answered by the tree, and `all_cases()` does not reach md-cli

The plan leaves the file's crate open (*"or md-cli if it needs the binary;
choose by what the assertion needs"*). `crates/md-codec/tests/liana_unspendable.rs:430-433`
already answers it: *"The C3 ruling moved the evidence-shape leg (kind-1
descriptor vs Liana's own `descriptor_with_checksum`) to stage 2, **which needs
`md decompose`/`md descriptor`**"* — i.e. md-cli. The consequence the plan does
not mention: `all_cases()` (`crates/md-codec/tests/common/liana.rs:108`) is not
`pub` and belongs to md-codec's test crate, so an md-cli test must carry its own
`Case` deserializer pointed at
`../md-codec/tests/fixtures/liana/cases.json`. That is the one place "derive the
counts from the corpus" costs more than a filter, and it is worth a sentence so
nobody re-hardcodes four and 24 to avoid writing a loader.

## M-k — two mechanical gates for the class that has now survived two rounds are never named

The Self-Review makes `plan-api-check.sh` a pre-dispatch gate. `mnemonic-engrave/scripts/`
also ships `plan-stepref-check.sh` and `plan-cite-check.sh`. I ran the first on
the folded plan: exit 1, **25 step-number-in-prose hits**, including `:488`'s
`§8.1 → Task 4 Step 3` — r1's N-b, mechanically, in under a second. This plan's
style uses step numbers heavily so the gate cannot simply be adopted as-is, but
naming it (even as "expected non-zero; read the hits for stale targets") is
cheaper than a third round finding the same pointers. `machine-checkable claims
get machine-checked, never reviewed`.

---

# NIT

## N-a … N-e — all five r1 Nits stand

See §1's carry list. N-e is subsumed by M-h.

## N-f — `chunk.rs:659` is measured at 658

`grep -n "decode_md1_string(&corrected_strings\[0\])?" crates/md-codec/src/chunk.rs`
→ **658**. `reassemble(&corrected_refs)?` → 666, correct. One character.

## N-g — the golden generator must capture stdout byte-exact, and a shell `$(…)` capture will not

MEASURED: `md compose --wrapper tr --preset kofn-recovery,2of3,older=26280`
writes a trailing `\n` on stdout (`od -c` tail: `) } ) \n`) and writes
`note: stdout is a keyless descriptor template (no keys)` to **stderr**. The
test compares `out` to the golden with `assert_eq!`, so a generator using
`$(md …)` strips the newline and **every row mismatches**. The Self-Review
already flags the generator's shape as needing a reviewer's execution pass;
this is that pass's first finding.

## N-h — `args.contains(&"tr")` is spelling-sensitive

If `gen-compose-golden.sh` ever emits `--wrapper=tr` instead of `--wrapper tr`,
the guard is false for every row and the explicit-nums equality check silently
never runs. The outer golden comparison still catches mutation (b), so this is a
quiet weakening rather than a hole — but matching on the value of the argument
after `--wrapper` costs the same.

---

## 4. The brief's three self-referential risks

**(a) Is "the golden must predate the flag" enforceable, or merely asked for?**
**Merely asked for.** Step 0's provenance argument is a git-history argument, and
git history does not prevent a second run. The script stays in the repo, the
test reads the golden by path at runtime, and
`./scripts/gen-compose-golden.sh` executed any time after Task 1 regenerates it
from the flag-aware binary — silently re-blessing whatever that binary then
does, including a live mutation. r1's M-f named the one-line close (refuse to
run if `md compose --help` mentions `--unspendable`); the fold did not take it
and the plan's text does not acknowledge the gap. Everything *else* about C-2's
remedy is structural and sound.

**(b) Is Task 2c's "already satisfied" bar tight enough?** **Yes — measured, not
argued.** See §2: at an unsupported version the correction is discarded
(exit 2, no repair report), so the first conjunct of Step 1's close bar is false
and the task cannot close itself. The second conjunct is already true (the
message names the version and the accepted set), which is what NEW-I-2 is about.
One residual looseness worth a clause: Step 1 has no instruction for the branch
where the trigger cannot be constructed — it *can* be, as I showed, so the
branch is now moot, but the plan would read better saying "construct it via
`codex32::unwrap_string` → rewrite the version nibble → `codex32::wrap_payload`"
rather than leaving the implementer to discover that route.

**(c) Does anything make Task 7's sweep more than a promise?** **Partly.** Step 1
supplies a concrete grep list (`no-op`, `today`, `not a variant`,
`three-valued`, `cannot`, `does not`, `only`), Step 4 says *"re-run, not
re-read… cite the value, do not restate it"*, and Step 5 forces a separate
commit so `git diff` isolates what the stage made false. That is more mechanism
than most sweeps get. What is absent: no gate runs it, no output is committed,
and no independent reader checks it — the same author who wrote the spec text
audits the spec text. Two cheap upgrades, neither of which needs a review round:
add `§8.9` / `md repair` / `v8` to Step 1's grep list (Task 2c reinterprets that
row and nothing else would catch it), and run `plan-cite-check.sh` over the spec
as Step 4's mechanism instead of a manual re-read.

---

## 5. Task-sequence walk (brief Q3)

Order as printed: 1 → 1b → 2 → 2c → 3 (pointer) → 4 → 5 → 6 → 7.

| boundary | precondition | verdict |
| --- | --- | --- |
| Step 0 → Step 1 | golden exists, committed, pre-flag | **OK on ordering** (I-g closed); generator shape unspecified (M-h, N-g) |
| Step 1 → Step 2 | test compiles without the flag; fails on `unexpected argument` | **OK** — runtime read compiles; the `tr` branch produces exactly that error |
| Step 9's mutations | both must redden the golden test | **OK under Step 7 as written**; re-point (b) if Step 7 changes to `Option<String>` (NEW-I-1) |
| Task 1 → Task 1b | Task 1b's refusal must not falsify Task 1's committed floor | **BLOCKED — NEW-I-1** |
| Task 2 Step 1 → Step 3 | refusal precedes warn so `plain-multisig` does not double-fire | **OK** — document order settles it, and `validate_unspendable_shape` is a no-op at kind 0, so the default cannot trip it |
| Task 2 Step 3 vs Step 4 | mutually exclusive? | **unstated — M-i** |
| Task 2c | trigger constructible | **OK, measured** — but Step 3's exit contract is undecided (NEW-I-2) |
| Task 4 → Task 5 | Task 4's counts must survive a fifth ACCEPT | **OK** — derived now; `all_cases()` + `accepted` filter is a one-line loop. Loader placement unstated (M-j) |
| Task 5 Step 1 → Step 2 | the vector must survive the regenerator | **BLOCKED — NEW-I-3** |
| Task 5 Step 4 | §8.8 may not close on a skip | **OK** — carried intact, including the Cargo-path-dependency correction |
| Task 6, Task 7 | no preconditions on earlier tasks | **OK** (M-c stands on Task 6 Step 3) |
| stage close | release mechanics | **BLOCKED — NEW-I-5**, nothing owns it |

---

## Gate

**0C / 5I — 0C/0I is not met.** `ready for implementation: no`.

Recommended fold order, cheapest-first:

1. **NEW-I-4** and **NEW-I-5** — one sentence and one task; neither needs a
   decision, and NEW-I-5 is a transcription of stage 1b's own Task 10.
2. **NEW-I-3** — four named edits, or one clause extending the script.
3. **NEW-I-1** — one decision, then the same sentence in two places (and Step 9(b)
   re-pointed if the declaration changes).
4. **NEW-I-2** — the only one needing a ruling that reaches outside these two
   repos. If message-only is acceptable, it collapses to a two-line edit.

Take **M-a** in the same fold regardless: it is the one that turns
`phase-gate.sh` red on the stage's first commit, and it is two doc comments.
**M-g** and **M-h** are also worth the same pass, because both sit on Step 0,
the one step whose output every later task consumes.

Re-run `./scripts/plan-build-gate-md.sh` on the fold. It will not catch M-a (its
scratch crate has no `[workspace.lints]` table), it does not reach Steps 6 and 7
(fragments, correctly declared), and it cannot see any of the five Importants
above — four are prose contradictions and one is an absent task.
