# composer S0b plan — R0 round 2, TARGETED fold-verification lens

**Artifact under review:** fold commit `b2c72f56f4917ecc678465da5f0b03861db59544`
(pre-fold `6c308b6`) applying round-1's fold-verification findings to
`design/IMPLEMENTATION_PLAN_composer_S0b_presets.md`.
**Question:** did the round-1 fold fix the two round-1 findings (M-4 Important,
M-3 Minor) exactly, and nothing else moved? Not a fresh audit.
**Method:** `cp -r` of the controller's read-only wired scratch
(`/scratch/code/shibboleth/.tmp/plan-build-gate-md`, already carrying the
folded plan's Rust extracted and both fragments hand-wired — confirmed by
inspecting the copy directly rather than re-extracting) to
`/scratch/code/shibboleth/.s0b-r2-lens`, `CARGO_TARGET_DIR` on a copy of the
warm target at `/scratch/code/shibboleth/.s0b-r2-lens-target`, toolchain
1.85.0 confirmed (`rustc --version` inside the copy). The one mutation below
was reverted and diffed byte-identical (`md5sum` match) to the pre-mutation
file before moving on; no two mutations were live at once. Nothing outside
the copy was touched (`git status --short` in the real repo is clean
throughout).

## Item 1 — the M-4 test iterates `PRESET_NAMES` itself — VERIFIED

Read `crates/md-cli/src/cmd/compose.rs`'s new `#[cfg(test)] mod tests`: `for
name in PRESET_NAMES { ... parse_preset(Wrapper::Wsh, &spec) ... }` — this
iterates the real `PRESET_NAMES` constant directly, not a hand-typed parallel
array, and calls `parse_preset` (not a fixture length assertion).

**Mutation, confirmed live.** Added `"seventh-preset"` to `PRESET_NAMES`
(`[&str; 6]` → `[&str; 7]`), no matching `match` arm added.
- `cargo build -p md-cli --all-targets` — compiles clean, no warning.
- `cargo nextest run --locked -p md-cli -E 'binary(/^cli_compose/) +
  test(every_preset_name_parses_with_some_valid_parameters)'` →
  `cmd::compose::tests::every_preset_name_parses_with_some_valid_parameters`
  **FAILED** (`30 tests run: 29 passed, 1 failed` for that filter's baseline
  31 — one test isolated to failure, no other test affected, no harness
  crash). Failure message: `PRESET_NAMES gained \`seventh-preset\` with no
  valid-parameter fixture in this test` — the test fails via its own designed
  `panic!` arm, which nextest reports as a normal FAILED test result for that
  one test (each nextest test runs in its own process; this is "the test
  fails," not "something else panics first" — the other 30 tests in the
  filter still ran and passed in the same invocation).
- Reverted (`md5sum` identical to pre-mutation: `0f5d21ef...`).

## Item 2 — `unreachable!()` gone; tail returns `CliError::Compose`; reachability — VERIFIED

`grep -n unreachable crates/md-cli/src/cmd/compose.rs` returns only a comment
recording the finding — no live `unreachable!()` remains in `parse_preset`'s
dispatch. The tail arm reads:
```
other => Err(CliError::Compose(format!(
    "preset {other}: internal error -- PRESET_NAMES advertises this name but no lowering rule exists for it (this is a bug in md, not a mistake in your command)"
))),
```

**Reachability probed live**, same mutation as item 1 (`seventh-preset` added
to `PRESET_NAMES`, no match arm): built the binary and ran
`md compose --wrapper wsh --preset seventh-preset,2of3` directly →
```
md: preset seventh-preset: internal error -- PRESET_NAMES advertises this name but no lowering rule exists for it (this is a bug in md, not a mistake in your command)
exit code: 1
```
No panic, no exit 101 — matches the plan's own design-note claim (line 523:
"the CLI itself now exits 1 with `preset phantom-preset: internal error --
...` instead of panicking") word-for-word in substance. Reverted, byte-identical.

This arm is unreachable under the *current*, non-drifted `PRESET_NAMES`/`match`
pairing (the name-containment check runs first and only passes for names that
also have a match arm today), so no permanent CI test exercises it directly —
only a hand mutation can reach it, which is exactly what the plan's design
note states ("machine-verified against that exact mutation") rather than
claiming automated regression coverage. This satisfies the brief's "or the
plan states why it is not [tested]" branch.

**Side note, not gating:** the fold *commit message's* own "Probe" line
("`md compose --wrapper wsh --preset phantom-preset,2of3` exits 1 with the
'expected one of ...' line (was exit 101 `unreachable!()`)") does not describe
the drift mutation — reproduced directly: running that exact command against
the *unmutated* `PRESET_NAMES` (where `phantom-preset` is simply not a
member) gives "expected one of ..." both before and after this fold, because
the name-containment check (already in place since round 0) rejects it before
ever reaching the `match`. The commit message's parenthetical "(was exit 101
unreachable!())" does not hold for this scenario — an ordinary unknown name
never reached `unreachable!()` even pre-fold. This is an inaccuracy in the
commit message's own narration of its gate note, not in the plan text (the
plan's design note describes the correct, mutated scenario, which is what I
independently reproduced above). Out of this lens's scope (the plan, not the
commit message) — recorded for completeness, not scored.

## Item 3 — the Minor (M-3 wording) — NOT FULLY FOLDED. Important.

The plan's STATUS line (line 15) states unconditionally: *"The Minor (M-3's
wording paraphrase) is also folded: `need_after_height`'s message now reads
'reads as a block height', matching `--path`'s own wording verbatim rather
than paraphrasing it."*

Grepped every occurrence of both wordings in the plan file:
- `reads as a block height` (5 hits): STATUS line, the test assertion
  (`cli_compose_preset.rs`, line 987), `--path`'s own message (line 1165,
  already correct pre-fold), the design-note comment (line 1349), and the
  actual `parse_preset` source (line 1356) — all correctly updated.
- `is read as a block height` (1 hit, line 540): **the refusal table row
  itself**, tagged `(R0 fidelity M-3)`:
  ```
  | `decaying-multisig`'s `after=` at or above the Unix-time band | names `--path`
  as the only remedy, ... `` preset <name>: after=<v> is read as a block height
  and is above the height band ... `` (R0 fidelity M-3) | 1 |
  ```
  This is the exact same quoted CLI message, still carrying the pre-fold
  wording, in the same document that elsewhere now says "reads as." The fold
  updated the test, the source, and the design-note comment, but missed the
  refusal table row that the fold's own status line claims is "also folded."

**Why this is Important, not cosmetic:** the underlying wording choice
("reads as" vs. "is read as") is itself cosmetic — but the *table is the
document's own record of what Task 2 Step 1 uses to write the failing tests
first*, and it now disagrees with the actual shipped message. A reader
implementing Step 1 directly from this table (as the plan structures the
task) would write a test asserting the STALE wording, which would then fail
against the real `parse_preset` output — reproduced above as
`reads as a block height`, not `is read as`. This is exactly the
incomplete-propagation class of defect (the substance was fixed, one
duplicate quotation site was left behind), and the plan's own status line
overstates it as complete. `scripts/plan-table-check.sh` does not catch this
by design — its own printed scope line states "NOT covered: cell CONTENT."

## Item 4 — nothing else moved

**Hunk-by-hunk walk** of `git diff 6c308b6..b2c72f56f4917ecc678465da5f0b03861db59544 -- design/IMPLEMENTATION_PLAN_composer_S0b_presets.md`
(10 hunks): STATUS line (bookkeeping for both fixes); "What is already
machine-verified" test-count/command bullet (bookkeeping, M-4); the design
note before the refusal table (M-4); the test-assertion wording change in
`cli_compose_preset.rs` (M-3); removal of the old tautological test from
`cli_compose_preset.rs` (M-4, moved); the `need_after_height` source wording
change (M-3); `unreachable!()` → `Err(CliError::Compose(...))` (M-4); the new
`#[cfg(test)] mod tests` block (M-4); Step 5's Run command + Expected count
update (bookkeeping, M-4); Step 1's whole-workspace Expected paragraph update
(bookkeeping, M-4). **Every hunk that moved is attributable to one of the two
findings.** No orphan hunk found. (The line-540 gap above is the converse
defect — a hunk that *should* have moved and did not — not an extra one.)

**Gate, unmutated, post-revert state (`cp -r` of the same wired scratch):**
- `cargo nextest run --locked -p md-cli -E 'binary(/^cli_compose/) +
  test(every_preset_name_parses_with_some_valid_parameters)'` → **31 tests
  run: 31 passed, 0 failed** (was 31 pre-round-1; unchanged count, same as
  the round-1 report's baseline).
- `cargo nextest run --locked -p md-codec -E 'binary(/^compose_/)'` → **52
  tests run: 52 passed, 0 skipped** (unchanged from round 1's 52).
- `cargo clippy --locked -p md-cli -p md-codec --all-targets -- -D warnings`
  → exit 0, clean.
- `cargo fmt --all -- --check` → exit 0, clean.
- `scripts/plan-cite-check.sh` → **25/25** resolved, 0 dangling, 0 ambiguous
  — matches the fold commit's cited 25/25.
- `scripts/plan-glyph-check.sh` → **107 strings, 0 undrawable** — matches the
  fold commit's cited 107.
- `scripts/plan-table-check.sh` → **25 rows, 0 malformed** — matches the fold
  commit's cited 25/0 (explicitly does not cover cell content, so it cannot
  and does not catch the item-3 finding above).
- `scripts/plan-stepref-check.sh` → **13** prose step references (tolerated
  class) — matches the fold commit's cited 13.

All match the fold commit's own cited numbers exactly.

## Closing counts

| severity | count | item |
|---|---|---|
| Critical | 0 | — |
| Important | 1 | Item 3: the M-3 Minor is only partially folded — the refusal table row at line 540 still quotes the pre-fold wording ("is read as a block height") while the STATUS line claims it "is also folded" and every other quotation site in the same document (test, source, comment) was correctly updated to "reads as a block height"; a reader writing Task 2 Step 1's test directly from this table would assert the wrong string |
| Minor | 0 | — |
| Nit | 0 | — |

**Not closed at 0C/0I.** Recommend for the next fold: update line 540's
refusal-table row to `reads as a block height`, matching the other five
sites, and re-run `plan-cite-check.sh`/`plan-table-check.sh` (structurally
unaffected, but re-run as a matter of course since the fold touches the same
file). Items 1, 2, and 4 are fully VERIFIED with no other open finding.
