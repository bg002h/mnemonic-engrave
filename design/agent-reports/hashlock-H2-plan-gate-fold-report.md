# Fold — build gate → `IMPLEMENTATION_PLAN_hashlock_H2_device.md`

Brief: `design/agent-briefs/hashlock-H2-plan-gate-fold-brief.md`.
Gate report folded: `design/agent-reports/hashlock-H2-plan-build-gate.md` (GATE GREEN
WITH FIXES, 12 fixes + 2 prose corrections).
Gated tree: `/scratch/code/shibboleth/.tmp/h2-gate` (fork main `c4a64fc` + the plan).
Plan before the fold: engrave master `f94c903`, plan text unchanged since `38509a9`.

Files edited, and only these: `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md`
(+762 / −88) and the new `scripts/h2-plan-blocks-vs-tree.sh`. Nothing committed. The
scratch tree, the spec and the gate report were not touched. No sub-agents. No `.jsonl`
read.

## The one claim this fold makes

Every code block in the plan is now the gated tree's own bytes, and
`scripts/h2-plan-blocks-vs-tree.sh` proves it: **25 blocks checked, 0 FAIL** (14 fragments
verified as exact byte substrings, 5 whole files verified by `diff`, plus 6 more
fragments). Full output below.

## The block-header convention (new)

Every fenced block carrying file content now opens with

    ```go file=<path relative to the fork root> mode=whole|fragment

stated at the top of the plan's File Structure section (a new paragraph after **Gate
coverage**). Markdown uses only the first word of an info string as the language, so
highlighting is unaffected. Blocks with no header are `bash` recipes or captured output,
and the script names them as its own blind spot.

## Per fix — what changed in the plan

| # | Plan section | Before → After |
| --- | --- | --- |
| 1 | Task 3, **new Step 3** ("The copy gate's row and its count") | no step existed → the `composerCopyTable` row for `composerCopyHashlockNoPayloadLead` (section `H2-3`) as a `file=gui/composer_copy_test.go mode=fragment` block, plus the `// 42 SINCE H2 TASK 3 …` comment block and prose bumping `declared` 41 → 42; Task 3's Files list and its `git add` line gained `gui/composer_copy_test.go` |
| 2 | Task 4 Step 1 | prose "Add rows … (section H2-4.2, H2-4.3a, …)" naming six §8 SECTIONS → three blocks: the `hashlock` import for `composer_copy_test.go`, all NINE Task-4 `composerCopyTable` rows verbatim, and the final count block (`// 51 SINCE H2 TASK 4 …` + `if declared != 51`). Prose now names the three functions that had no row at all — `composerCopyHashlockRefusal`, `composerCopyHashlockRelation`, `composerCopyHashEveryPathFor` — and quotes the gate's "under-counted its own new functions by 4", noting that 4 counts all ten H2 rows including Task 3's |
| 3 | Task 4 Step 1 copy block + the §4.5 note under it | the unshortened reuse block ("One phrase per policy. Spending any path of a wsh wallet publishes this digest. Never use this phrase … anyone can then test guesses at the phrase itself.") → the brainstorm's two sentences ("One phrase per policy. Never use this phrase as a passphrase or a password anywhere else."), with the measurement recorded: 484 of 504 drawn and CUT at step 0; **64 characters of headroom against a required margin of 80** after step 1 — still failing |
| 4 | Task 4 Step 1 copy block + the §4.5 note | the reconciliation line ("Before you fund this wallet, run ms hashlock …") lived in `composerCopyHashlockConfirm` → it lives in `composerCopyHashEveryPathPhrase`, whose gated doc comment says why. Measured after both drop-order steps: confirm **290 drawn / headroom 186**, §8h form **254 drawn / headroom 262**. The old Step-1 instruction "if the §4.5 longest body does NOT fit, apply the spec's drop order … record which step was needed" is replaced by the measured record of BOTH steps, and a "do NOT re-lengthen" line. The spec was not edited |
| 5 | Task 4 Step 2 (whole file) + the notes under it | `ctx := NewContext(newPlatform())` → `p := newPlatform(); p.display = sh2DisplaySize; ctx := NewContext(p)`, with the 340 px keyboard rule cited to `gui/passphrase_flow_test.go:28-31` |
| 6 | Task 4 Step 2 | `composerStateForTest` was named only in a parenthetical → written out, returning `&composerState{list: md.PathList{Wrapper: md.ComposeWsh}}`, because `md.ComposeWrapper`'s zero value is `ComposeTr` (`md/compose.go:32`) and a key-less path is refused under `tr` (`gui/composer_shape.go:250`) |
| 7 | Task 4 Step 2 | flows opened at `h.mustReach("EXPERIMENTAL")` → `h.mustReach("What can spend on this path?"); h.choose(1)` first, **7 call sites** (counted in the gated file) |
| 8 | Task 4 Step 2 | `h.mustReach("Hash lock") // §8i rule modal` → `h.mustReach("32-byte value") // the §8i rule modal (composerCopyHashRule)`, **7 call sites**; the §8i modal's title is `Path N hash`, "Hash lock" is the confirm screen's |
| 9 | Task 4 Step 2 | `h.mustReach("Which hash?")` → `h.mustReach("Type a hashlock phrase")` on the zero-payload sessions, **7 of the original 8** (the 1 survivor is `TestHashlockConfirmRelationLine`, which loads 2 payload digests, so the lead genuinely IS "Which hash?" there) |
| 10 | Task 4 Step 2 | the stray `h.tapNav(Button3)` after each method-pick `h.tapRow(…, 2)` is gone, **9 call sites** (counted both in the old block and the new) |
| 11 | Task 4 Step 2 (the `holdConfirm` helper) + Step 4 | "`holdConfirm` is the `ConfirmWarningScreen` hold gesture the existing composer tests use" → the gate's implementation: hold, wait `confirmDelay`, then send an explicit `PointerEvent{Pressed: false}`. The file's own comment records the mechanism — `EventRouter.Events` (`gui/event.go:14-15`) tracks ONE pointer contact globally and reuses the stale `pointer.pressedTag` while `pointer.pressed` is true — and Step 4's mutation table gains a MUTATION row: delete the release → every test with two or more holds hangs at its second |
| 12 | Task 3, **new Step 4** | not in the plan at all → the `gui/composer_gates_test.go` fragment moving `TestComposerLockAndHashEditsAreNotGuardedByTheDiscardConfirm`'s "hash lock" pump target from `"Which hash?"` to `"Path 1 hash"`, filed **in Task 3** because Task 3's no-payload lead swap is what causes it, plus the instruction to run the full shard set (the narrow `-run` selections never touch that file; the gate found it only when shard 11 failed) |

## The two prose corrections (Task 1 Step 5)

The one-paragraph mutation list became a **measured table** headed "the build gate's
MEASURED outcome … which corrected two of this plan's own round-0 predictions".

- **Separator strip.** "exactly the `correct-horse,battery staple` row fails" → four rows
  fail. The gate is quoted verbatim, and I re-measured against the vendored corpus rather
  than trusting it: 4 of the 11 derivation phrases contain a `-` or a `,` —
  `correct-horse,battery staple` (28), `a-b,c` (5), `hashlock phrase row: sixty-four
  printable characters, no hex!!xx` (64) and its `!`-suffixed sibling (65). (The gate calls
  the last two "BOTH 64-char rows"; they measure 64 and 65.)
- **Cap literal 99.** "`TestPhraseMaxCharsIsTheCap` **and the 100-character refusals row**
  fail" → only the first. Re-measured: the corpus's sole `too-long` refusals row is **101**
  characters, refused under either cap, so `TestRefusalRowsMatchTheHost` stays green.

Both mutation rows still function as gates; only the plan's descriptions of SCOPE were
wrong. The table now says so explicitly.

## Beyond the brief — three more things folded, each named as such

1. **A third prose correction, from the gate's Task 4 mutation table.** "`composerHashEdit`
   returns `false` from the phrase route's Back → the same test fails on the path count" is
   measurably imprecise: it fails EARLIER, at `never reached "Type a hashlock phrase"`.
   Task 4 Step 4's mutation table now carries the measured failure for every mutation, and
   flags this one. The gate's verdict counts two prose corrections because it counts only
   the Task 1 table; this is the third.
2. **A thirteenth change the gate applied but did NOT list in its fixes table.**
   `func hashHex(h *[32]byte) string` in the plan's test file is `hashlockHashHex` in the
   gated tree. Reason, verified by me against the fork at `c4a64fc`, not taken from the
   report: `gui` already declares `func hashHex(h [16]byte) string` at
   `gui/seal_fixture_test.go:172` — same package, so the plan as written was a
   redeclaration and could not compile. Folded (the block is the tree's bytes) and named in
   the plan's fold section so the two counts reconcile.
3. **Import changes the plan never mentioned**, each now its own fragment block:
   `"fmt"` into `gui/composer_hash_test.go`; `seedhammer.com/hashlock` into
   `gui/composer_copy.go`, `gui/composer_copy_test.go` and `gui/modal_fits_test.go`;
   `image` into `gui/composer_hashlock.go` (round 0 used `image.Pt` without importing it —
   the old parenthetical told the implementer to "align imports", which is not a
   compilable instruction).

## Other plan changes the fold required

- **STATUS** → `DRAFT -- build gate GREEN WITH FIXES folded; R0 round 0 pending.`
- **File Structure table**: `gui/composer_copy_test.go` and `gui/modal_fits_test.go` split
  into separate rows carrying the row counts and the 41 → 42 → 51 literal; a new
  `gui/composer_gates_test.go` row (fix 12).
- **Task 3 Files/Interfaces**: the `composerHashEdit` citation `:140-172` → `:139-176`
  (measured at `c4a64fc`: doc comment at 139, `func` at 140, closing brace at 176);
  Produces now names the `composerHashRowSet` type and the `composerHashRows` constructor
  instead of a "`composerHashRows` struct" that cannot exist alongside the function; the
  `hashlockPhraseRoute` signature gains the `payload [][32]byte` parameter it actually has.
  Steps renumbered 1-6 (nothing in the plan cross-references Task 3 step numbers — checked).
- **Task 3 Step 2**: the block is the tree's 80 lines, so the "rename the constructor …
  apply that renaming consistently" parenthetical is replaced by a note that the block
  already carries the naming.
- **Task 4 Step 4**: mutations become a table of MEASURED failures, and the two
  pre-existing `go vet` complaints (`gui/freetext_sizeproof_golden_test.go:111`,
  `gui/transaction_golden_test.go:104`, `testing.ArtifactDir requires go1.26`) are named so
  a new one is visible.
- **Task 4 Step 5**: "quote it" → the measured numbers: 1213 top-level tests, partition
  verified exhaustive, 24/24 shards ok, 34 s wall; +8 new top-level tests; `hashlock`'s own
  6 are a separate package and not in the 1213.
- **Task 5 Step 2**: the expected firmware size is now the gate's captured `-size short`
  output — **flash 1,595,236 B / RAM 62,856 B**, **+12,104 B (+0.76%) / +56 B (+0.09%)**
  against `c4a64fc`'s 1,583,132 / 62,800. The rule stated is spec §7.6's own ("expect a
  small delta"): **no numeric flash ceiling is asserted, because neither the spec nor the
  plan sets one** — the acceptance is the delta against the named baseline, and the walk is
  not in the number.
- **Self-review item 2 (Placeholders)**: "the implementer writes them (small)" → none left
  in the Go; the one remaining prose-only executable artifact is Task 5 Step 1's emulator
  walk.
- **New section `## Build gate folded here`** before Self-review: the 12 fixes one line
  each, the 13th unlogged change, the two prose corrections plus the third, the whole-suite
  and size numbers, and the script's output.

## Citations added by this fold, re-grepped (a fold adds citations that were never gated)

Every file:line below was resolved by me against the fork at `c4a64fc` and/or the gated
tree, not carried over from the report:

| Citation | Verified |
| --- | --- |
| `md/compose.go:32` — `ComposeTr ComposeWrapper = iota` | yes, iota 0 |
| `gui/composer_shape.go:250` — the key-less-under-`tr` guard | yes; its message is `composerCopyRefuseKeylessTr()` at `gui/composer_copy.go:218`, "This build will not put a key-less path in taproot." |
| `gui/event.go:14-15` — `pressedTag op.Tag` / `pressed bool` | yes |
| `gui/passphrase_flow_test.go:28-31` — the 340 px / off-canvas rule | yes, verbatim |
| `gui/seal_fixture_test.go:172` — the pre-existing `hashHex([16]byte)` | yes |
| `gui/composer_hash.go:139-176` — `composerHashEdit` at `c4a64fc` | yes (the plan said :140-172) |
| `gui/composer_shape.go:443` — the §8h `showError` line | yes, in the gated tree |
| `gui/freetext_sizeproof_golden_test.go:111`, `gui/transaction_golden_test.go:104` | yes, both are `dir := t.ArtifactDir()` |
| `seal/pbkdf2.go`, `seal/crypto.go` (already linked) | both exist |
| corpus sha256 `a46c197a…11d30` | recomputed on the vendored copy, matches the pin |
| call-site counts 7 / 7 / 7 / 9 (fixes 7-10) | counted in the gated file with `grep -c`, all four confirmed |
| 1,595,236 − 1,583,132 = 12,104 (+0.76%); 62,856 − 62,800 = 56 (+0.09%) | arithmetic re-done |

## Could NOT fold, and why

1. **`cmd/emu/walk_hashlock_phrase.js` (Task 5 Step 1).** The plan carries it as PROSE, not
   as a code block, and the gate did not write or run it (out of scope per its own dispatch
   brief) — there is no file in `/scratch/code/shibboleth/.tmp/h2-gate/cmd/emu/` to make the
   plan match. It stays prose, and the plan now says explicitly (Self-review item 2) that it
   is the plan's one un-gated executable artifact. **This is a real gap: Task 5's walk has
   never been executed, and by the "a plan may not close while any of its own gates has
   never been run" rule that is R0's business, not this fold's.**
2. **The corpus JSON itself** (`hashlock/testdata/hashlock-v0.8.json`). It is vendored by a
   `cp`, not by a code block, so there is nothing to make byte-identical; the sha256 pin is
   the check, and I recomputed it. The script therefore does not cover that file.
3. **The `bash` blocks** (vendor recipe, five `git commit` recipes) and every prose claim.
   The script prints both as its blind spots. The commit recipes' `git add` lines WERE
   updated by hand where the fold added files (Task 3 now stages `composer_copy_test.go`
   and `composer_gates_test.go`), but nothing executes them.
4. **Nothing was declined.** All 12 fixes, both prose corrections, the third correction and
   the 13th change are folded.

## Script output (`scripts/h2-plan-blocks-vs-tree.sh`, full)

```
plan: /scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_hashlock_H2_device.md
tree: /scratch/code/shibboleth/.tmp/h2-gate

PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:127  whole          hashlock/testdata/hashlock-v0.8.provenance.json  (18 lines, identical)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:150  whole          hashlock/hashlock_test.go                     (213 lines, identical)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:373  whole          hashlock/hashlock.go                          (143 lines, identical)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:555  fragment       codex32/mspayload_test.go                     (42 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:604  fragment       codex32/mspayload.go                          (25 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:658  fragment       gui/composer_hash_test.go                     (34 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:697  fragment       gui/composer_hash_test.go                     (6 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:710  fragment       gui/composer_hash.go                          (2 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:717  fragment       gui/composer_hash.go                          (80 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:809  fragment       gui/composer_copy.go                          (6 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:820  fragment       gui/composer_copy_test.go                     (2 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:827  fragment       gui/composer_copy_test.go                     (2 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:836  fragment       gui/composer_gates_test.go                    (8 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:873  fragment       gui/composer_copy.go                          (5 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:883  fragment       gui/composer_copy.go                          (74 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:970  fragment       gui/composer_copy_test.go                     (9 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:982  fragment       gui/composer_copy_test.go                     (23 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1011  fragment       gui/composer_copy_test.go                     (8 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1028  fragment       gui/modal_fits_test.go                        (8 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1039  fragment       gui/modal_fits_test.go                        (21 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1087  whole          gui/composer_hashlock_test.go                 (531 lines, identical)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1678  whole          gui/composer_hashlock.go                      (213 lines, identical)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1901  fragment       gui/composer_state.go                         (4 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1910  fragment       gui/composer_shape.go                         (1 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1916  fragment       gui/composer_copy.go                          (6 lines, verbatim substring)

25 blocks checked, 0 FAIL

NOT COVERED by this script:
  * 7 fenced blocks carry no file= header (bash recipes, illustrative
    snippets); nothing here runs or checks them:
      IMPLEMENTATION_PLAN_hashlock_H2_device.md:119  ```bash
      IMPLEMENTATION_PLAN_hashlock_H2_device.md:540  ```bash
      IMPLEMENTATION_PLAN_hashlock_H2_device.md:636  ```bash
      IMPLEMENTATION_PLAN_hashlock_H2_device.md:853  ```bash
      IMPLEMENTATION_PLAN_hashlock_H2_device.md:1957  ```bash
      IMPLEMENTATION_PLAN_hashlock_H2_device.md:1975  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H2_device.md:1993  ```bash
  * every PROSE claim: expected test names, mutation outcomes, headroom and
    firmware numbers, spec references, file:line citations.
  * whether the tree is GREEN -- this compares TEXT only; `go test` and the
    gate report are what say the text works.
  * files the plan modifies without carrying a block for them.
```

Exit status 0. The same output is pasted (indented) into the plan's
`## Build gate folded here` section, and a fresh run reproduces it byte for byte.
