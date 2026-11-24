architect bughunt-me: 3 answered; 1 prior rulings changed

# Architect rulings: me bug-hunt FIX doc, STOPs S1, S2, S3

Inputs: `design/FIX_bughunt_me.md` (STOP sections, lines 231-269), grade
`design/agent-reports/grade-bughunt-me.md` (R1-R5), `SPEC_descriptor_input.md:905-911`,
`SPEC_systemwide_payloads.md:908`, `main.rs:934-938, 1511-1533, 3039-3214`,
`as_flag.rs:52-61`, `descriptor_as.rs:133-150, 553-588`. Worktree
`/scratch/code/shibboleth/me-worktrees/bughunt-triage` at c12849b6, cargo 1.85.0,
target `/scratch/cache/bughunt/me/arch-target`. Scratch changes were restored;
`git status --short` is empty.

Prior ruling changed: **R1's placement only** ("at the top of the `--in`
branch"). As placed, R1 replaces the normative §5.1 refusal
(`SPEC_descriptor_input.md:909-911`) and fails
`as_with_argv_and_in_together_is_usage`. That is an unmet guarantee and a red
suite, so it does not bind. R1's text, exit code and "write nothing" stand.

## S1 and S3 (one change): run the argv guard first, and refuse "not both" only without `--as`

**Decision.** If argv and `--in` are both given, the argv bearer/secret guard
runs first (exit 3). After it:
- without `--as`, the invocation is refused with R1's "not both" (exit 2);
- with `--as`, §5.1's existing shape check refuses it (exit 2, unchanged).

§5.1 and its test do not change. S3 takes option (b).

**Why.** The pack arm states this order (main.rs:1519-1524) and
`the_argv_bearer_guard_precedes_the_single_document_check` (descriptor_as.rs:553-588)
pins it, noting the constellation "has already been bitten once" by a gate that
pre-empted this refusal; fix-04 at the top of the `--in` branch repeats that.
Holder steps are equal (one refusal, one re-run); guard-first adds history
advice for a token already in history. Seal's order (main.rs:934-938) has no
`--as` and no stated rule, so the pack arm's own rule wins. Option (b) keeps
normative §5.1 and its test and changes one spec row, not two. The `as_present`
early return also stops `--as md1 --in f -` blocking on stdin before §5.1.

### Code (replaces FIX doc §1 "#4"; file `crates/me-cli/src/main.rs`)

1. `read_records` signature (line 3039-3043). After `allow_argv_secret: bool,`
   add the parameter `as_present: bool,`.
2. Line 3044: `if let Some(p) = in_path {` becomes
   `if let Some(p) = in_path.filter(|_| argv.is_empty()) {`.
3. Inside `if !argv.is_empty() {`, immediately after the closing `}` of the
   `for (i, r) in argv.iter().enumerate()` guard loop (base line 3156) and
   before the comment `// §6b — \`-\` MEANS STDIN, ...`, insert:

```rust
        // R1 (bug hunt #4): argv AND --in. Checked AFTER the guard above, for
        // the pack arm's rule: a flag-shape usage error must not pre-empt a
        // refusal about material already in the shell's history.
        if in_path.is_some() {
            if !as_present {
                return Err((
                    "pass records on argv OR via --in, not both".to_string(),
                    EXIT_USAGE,
                ));
            }
            // With `--as`, the caller's §5.1 shape check refuses this
            // invocation in its own words. Read neither --in nor stdin.
            return Ok(Records {
                records: argv.to_vec(),
                document: argv.join("\n"),
            });
        }
```

4. Caller, line 1511. The call becomes
   `read_records(records, r#in.as_ref(), *allow_argv_secret, r#as.is_some())`.
   Then run `cargo +1.85.0 fmt --all`, which wraps it as
   `let (recs, document) =\n                match read_records(...) {` with the
   arms indented one level.
5. Unit tests in `mod tests`, lines 3618 and 3638. Append `, false` to each
   call, giving `read_records(std::slice::from_ref(&rec), None, false, false)`
   and `read_records(&["text:6869".to_string()], None, false, false)`.

The full diff is saved at `/scratch/cache/bughunt/me/arch-s1s3.patch`. fmt,
clippy (`-D warnings`, all targets, workspace) and the suite are all clean;
the measurements are below.

### Spec

- `SPEC_systemwide_payloads.md:908`. This replaces the FIX doc §2 #4 cell:
  before the row's closing ` |`, append
  ` Without \`--as\`, records on argv together with \`--in\` are refused after the argv bearer/secret guard: \`me: pass records on argv OR via --in, not both\`, exit 2.`
- `SPEC_descriptor_input.md` §5.1 (905-911): no change.

### Tests

- `as_with_argv_and_in_together_is_usage`: unchanged, and it passes. Delete
  FIX doc §4's first bullet. In §6's expectations, gate 1 becomes
  `704 tests run: 704 passed, 2 skipped`, and gate 5 becomes
  `13 tests run: 13 passed` (before #4: `13 tests run: 9 passed, 4 failed`).
- New RED rows. Append them at the end of `crates/me-cli/tests/bughunt.rs`
  with exactly the text in `/scratch/cache/bughunt/me/arch-04cd-tests.patch`.
  The patch is 46 lines and fmt-clean:
  - `bughunt_me_04c_argv_bearer_guard_precedes_not_both`
  - `bughunt_me_04d_as_argv_bearer_guard_precedes_single_document_with_in`

| row | command | RED on base (measured) | GREEN with fix (measured) |
|---|---|---|---|
| 04c | `cargo +1.85.0 nextest run --locked --workspace -E 'test(/(^\|::)bughunt_me_04c_argv_bearer_guard_precedes_not_both$/)'` | bughunt.rs:352 `left: Some(0) right: Some(3)` (argv dropped, exit 0) | PASS |
| 04d | same with `bughunt_me_04d_as_argv_bearer_guard_precedes_single_document_with_in` | bughunt.rs:372 `left: Some(2) right: Some(3)`, stderr `me: --as packs exactly one descriptor per invocation.` | PASS |
| 04, 04b | as in FIX doc §3 | FAIL :135 / :296 (unchanged) | PASS |

Both rows were run on base `main.rs` with `-E 'test(/^bughunt_me_04/)'`:
`4 tests run: 0 passed, 4 failed, 702 skipped` (log
`/scratch/cache/bughunt/me/arch-red.log`).

With the fix, the full run (`/scratch/cache/bughunt/me/arch-full.log`) gave
`704 tests run: 697 passed, 7 failed, 2 skipped`; the 7 are the other items'
unfixed rows (01, 02, 02b, 03, 03b, 05, 05b). `as_with_argv_and_in_together_is_usage`,
`the_argv_bearer_guard_precedes_the_single_document_check` and 04-04d passed.

### Probes on the fixed `me` binary (stdin `/dev/null`, r.txt = `text:6162`)

| `me sysw pack --no-passphrase ...` | rc | stderr first line | written |
|---|---|---|---|
| `--in r.txt --out p.bin text:6869` | 2 | `me: pass records on argv OR via --in, not both` | no |
| `--in r.txt --out p.bin tx:00` | 3 | ``me: record 0, as given (records count from 0), is a `tx:` record on ARGV. Refused; ...`` | no |
| `--as md1 --in r.txt tx:00` | 3 | same bearer refusal | no |
| `--as md1 --in r.txt text:6869` | 2 | `me: --as packs exactly one descriptor per invocation.` | no |
| `--in r.txt --out p.bin -` | 2 | `me: pass records on argv OR via --in, not both` | no |
| `--as md1 --in r.txt -` | 2 | `me: --as packs exactly one descriptor per invocation.` | no |

Fold needed: yes, a FIX doc fold. Replace §1 #4, the §2 #4 row, the §3 rows
(add 04c and 04d) and the §4/§6 counts as above, and lift the "BLOCKED"
banner. No manual change: FIX doc §5's greps show that no `me` manual text uses
argv together with `--in`.

## S2: yes, with one qualifier ("On Unix")

**Decision.** Accept the proposed text, adding "On Unix". The reason is that
`set_permissions` is under `#[cfg(unix)]` (write.rs:58-62), and A3's own
sentence (spec line 102) already scopes the mode to Unix. Without the
qualifier, the sentence would claim behaviour on Windows that the code does
not have.

**Why.** R5 makes the old residual false. After F-244, an overwrite ends at
0o600 (write.rs:55-62). After R5, a refused chmod returns before `set_len(0)`,
so the old bytes survive; RED row `bughunt_me_01` pins this. The "acceptable
residual" clause therefore has nothing left to excuse and goes too.

**Exact text.** In `design/SPEC_me_preview_hardening.md:118-120`, replace:

`**0o600 binds on CREATE; an overwrite of a pre-existing file keeps its old mode** (R0 L3)`
`— acceptable residual: the NDEF/manifest targets are user-named and the preview targets`
`are forced-fresh by A1.`

with:

`` `.mode(0o600)` binds only on create, so on Unix `write_private` also calls `set_permissions(0o600)` on the open file (F-244): an overwrite of a pre-existing file ends at 0o600, and a refused chmod fails the write before any byte of the old file is touched (R0 L3, superseded). ``

The preceding word "Also" stays, so the sentence reads "Also `.mode(0o600)`
binds only on create, ...". The text is spec-only, so there is no code, test
or RED row to run. The claims it makes are the ones already measured by
`bughunt_me_01` (FIX doc §3).

Fold needed: yes. Replace the FIX doc §2 "STOP S2" cell with the text above.

## Open items

None that are Brian's alone. The FIX doc §1 #4 note ("if the architect picks
S3 option (b), the placement changes ...; the text and exit code do not")
holds as written.

Counts: 3 answered (S1, S2, S3); 1 prior ruling changed (R1 placement);
RED rows added 2 (04c, 04d), both run RED on base and GREEN with the fix;
probes 6; Critical 0, Important 0 raised by this report.
