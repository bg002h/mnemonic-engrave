# composer S0b — targeted fold-verification review, round 1

**Question:** did the controller's fold commit `1dc8d409f6e8daa099226937f5f107f56b64dd97`
(descriptor-mnemonic branch `composer-s0b`, worktree `/scratch/code/shibboleth/wt-composer-s0b`,
diff against pre-fold tip `87bc10ff`) fix the whole-diff review's M-1, M-2, M-3 and N-3 exactly as
filed in `design/agent-reports/composer-S0b-exec-review-r0.md`, can each new test fail, and did
nothing else move?

Read-only. `CARGO_TARGET_DIR=/scratch/code/shibboleth/.s0b-verify-target`,
`TMPDIR=/scratch/code/shibboleth/.tmp`. No sub-agents dispatched. `git status --porcelain` empty
at exit; both mutations applied during verification were reverted with `git checkout --`.

## Counts

**0 Critical / 0 Important / 0 Minor / 0 Nit.** All four items VERIFIED, both new tests can fail
on the named mutation, nothing outside the four items plus the CHANGELOG line moved, and the full
gate reproduces the fold commit's figures.

## Per-item verification

### M-2 — VERIFIED

`git diff 87bc10ff 1dc8d409 -- crates/md-cli/src/cmd/compose.rs` shows all six preset arms
(`plain-multisig`, `simple-timelocked-inheritance`, `kofn-recovery`, `tiered-recovery`,
`hashlock-gated`, `decaying-multisig`) swapped to call `named_only(..)?` before `need_ofs(..)?`.

Mutation: swapped the two lines back in `plain-multisig` (`need_ofs(1)?; named_only(&[])?;`).
`cargo nextest run --locked -p md-cli -E 'test(preset_names_an_unknown_parameter_before_counting_kofn)'`
FAILED:

```
Unexpected stderr, failed var.contains(preset plain-multisig admits no 2of3= parameter)
├── var: md: preset plain-multisig needs exactly 1 <k>of<n> parameter, got 0
```

Reverted (`git checkout -- crates/md-cli/src/cmd/compose.rs`); tree clean.

Four M-2 reproductions re-run against the fold (built binary
`/scratch/code/shibboleth/.s0b-verify-target/debug/md`), all now name the token:

```
$ md compose --wrapper wsh --preset "plain-multisig,bogus=1"
md: preset plain-multisig admits no bogus= parameter                (exit 1)
$ md compose --wrapper wsh --preset "plain-multisig,=5"
md: preset plain-multisig admits no = parameter                     (exit 1)
$ md compose --wrapper wsh --preset "plain-multisig,2of3=x"
md: preset plain-multisig admits no 2of3= parameter                 (exit 1)
$ md compose --wrapper wsh --preset "plain-multisig,2of3,bogus=1"   (unchanged row, still correct)
md: preset plain-multisig admits no bogus= parameter                (exit 1)
```

### M-3 — VERIFIED

`need_u32` (`compose.rs:275-297`) detects a trailing `u` on any `older*` key or `t` on `after`,
and only fires the remedy when the stripped remainder parses as `u32`; otherwise falls through to
`parse_u32`'s generic message (confirmed live: `older=xu` → "is not a number in 0..=4294967295").

Mutation: deleted the `if let Some((rest, meaning)) = suffixed { ... }` block.
`cargo nextest run --locked -p md-cli -E 'test(preset_names_the_path_remedy_for_u_and_t_suffixes)'`
FAILED:

```
Unexpected stderr, failed var.contains(preset simple-timelocked-inheritance older: `100u` is
  --path's `u` (older in 512-second units) spelling, ...)
├── var: md: preset simple-timelocked-inheritance older: `100u` is not a number in 0..=4294967295
```

Reverted; tree clean.

`need_after_height`'s band message re-run against the fold, unchanged from the M-3 reproduction
in r0:

```
$ md compose --wrapper wsh --preset "decaying-multisig,2of2,1of1,older1=13140,older2=26280,after=500000000"
md: preset decaying-multisig: after=500000000 reads as a block height and is above the height
band (1..=499999999); presets cannot express a Unix time -- use --path with `after=500000000t`
instead                                                              (exit 1)
```

### M-1 — VERIFIED

`crates/md-codec/tests/compose_support.rs`'s `SINGULAR_TAGS` doc now states two grounds explicitly:
`spine:0` has exactly one legal shape (§12 item 1's exemption); `head:hashed` and the six
`preset:<name>` tags have one vector by deliverable scope (F-453), not one legal shape. The phrase
"by construction" is gone (`grep -n "by construction" compose_support.rs` → no match). The constant
itself is byte-unchanged: `spine:0`, `head:hashed`, and the six `preset:*` tags, same 8 entries, same
order — only the doc comment above it changed.

`git diff 87bc10ff 1dc8d409 -- crates/md-codec/tests/compose_vectors.rs` is empty: the `== 1`
pin for every `SINGULAR_TAGS` member is untouched.

### N-3 — VERIFIED

`crates/md-cli/README.md` gained one `md compose` row:
`` `md compose --wrapper <W> (--path <SPEC>... | --preset <NAME[,...]>)` `` with flags
`--wrapper`, `--path`, `--preset`, `--json`. `md compose --help` (built from the fold) confirms the
same four flags plus `--experimental` (not claimed by the README row, correctly — the row doesn't
list it). Wrapper spellings `tr | wsh | sh-wsh | sh` match `parse_wrapper` (`compose.rs:15-25`)
exactly. Root `README.md`'s subcommand list gained `compose` between `decompose` and `repair`.

### Nothing else moved — VERIFIED

`git diff --stat 87bc10ff 1dc8d409` touches exactly 6 files, each accounted for:

| file | content |
| --- | --- |
| `crates/md-cli/src/cmd/compose.rs` | M-2 (6-arm swap) + M-3 (`need_u32` suffix block) — read in full, no other hunk |
| `crates/md-cli/tests/cli_compose_preset.rs` | the two new regression tests (M-2, M-3) |
| `crates/md-codec/tests/compose_support.rs` | M-1 doc comment only |
| `crates/md-cli/README.md` | N-3 row |
| `README.md` | N-3 subcommand list |
| `CHANGELOG.md` | the changelog line naming M-2/M-3/README row |

No file outside this set changed; no hunk within these files is unaccounted for.

## Build gate (reproduced independently on this tree)

```
cargo fmt --all -- --check                                              exit 0
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings   exit 0
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked   exit 0
cargo nextest run --workspace --all-features --locked
  1342 tests run: 1342 passed, 3 skipped     <- matches fold commit message exactly
cargo nextest run --locked -p md-cli -E 'binary(/^cli_compose/)'
  32 tests run: 32 passed, 0 skipped
```

The brief's parenthetical "(was 31)" for the `cli_compose*` binary count does not match: counting
`#[test]` in the three matching files (`cli_compose_encode_gate.rs`, `cli_compose.rs`,
`cli_compose_preset.rs`) at `87bc10ff` gives 3 + 6 + 21 = **30**, not 31; the fold's two new tests
land on `cli_compose_preset.rs` alone (21 → 23), giving 3 + 6 + 23 = **32**, which is what both the
fold commit's message and this run report. This is a stale figure in the brief, not a fold defect
— the fold's own claimed count (32) is correct and reproduces.

## Closing counts

**0 Critical / 0 Important / 0 Minor / 0 Nit.** All four items (M-1, M-2, M-3, N-3) fixed exactly
as filed; both new regression tests fail on their named mutation and pass on the fold; no hunk
outside the four items and the CHANGELOG line; full gate green with figures matching the fold
commit's message.
