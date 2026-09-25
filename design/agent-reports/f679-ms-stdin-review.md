# F-679 ms 0.19.1 (`f679-stdin`) — adversarial review

Reviewer: opus, 2026-09-24. Target: `/scratch/code/shibboleth/ms-worktrees/f679-stdin`,
`git diff origin/master..a77c8e4` (fix `5ebb413`, release `a77c8e4`). Branch not edited.
Binary under test: `ms 0.19.1` built from the worktree (debug profile, `CARGO_TARGET_DIR`
under `/scratch/code/shibboleth/review-ms0191-scratch/`).

**One question:** can any verb read the wrong secret, or silently drop, duplicate or reorder
one, when argv (admitted), a literal `-`, and `--in` are mixed? Are two genuine stdin inputs
still refused?

**Answer: no wrong, dropped, duplicated or reordered secret was found, and stdin+stdin is still
refused on verify and derive.** Result: 0 Critical, 0 Important, 1 Minor, 1 Nit.

## What was run

| check | result |
|---|---|
| `cargo nextest run --locked --workspace` | **624 passed, 11 skipped**, rc 0. Matches the implementer's count |
| clippy 0.1.85 (pinned 1.85.0 bin prepended) `--workspace --all-targets -D warnings` | clean |
| `cargo fmt --check` | clean |
| `ci/repro/vendor-freshness.sh` | OK |

### 1. combine: 3,987-case matrix (`combine_matrix.py`, 3-of-5 split of "abandon ×11 about")

Every case uses `--allow-argv-secret --json`. The oracle is the recovered `phrase` field:

- **Family 1: exactly K valid shares (3,888 cases).** All 60 ordered 3-permutations of the
  5 shares. Each share goes on argv (admitted) or on stdin, in all 2³ splits. A genuine `-` is
  placed at every position, once or twice (0 or 1 times when stdin is empty). **All recovered
  exactly the split phrase.** Exactly K means a dropped share would fail under threshold.
- **Family 2: K valid shares plus one poison share from a different split** (`zoo ×11 wrong`,
  different id), every permutation, channel split and `-` position. **Every case failed.** A
  success would have meant the poison share was silently dropped.
- **Family 3: 3 argv shares plus a duplicate of one of them on stdin, with `-` at each of 4
  positions (12 cases).** Every case failed with `share index '<x>' repeated`. So stdin
  shares are neither deduplicated nor dropped, and argv shares are not consumed twice.
- **Family 4: K-1 argv shares plus `-` with empty stdin.** Every case failed. No share is
  invented.

Order only affects which K shares define the polynomial. The extras are checked by
`combine_shares`'s M6 consistency test, so reordering cannot change the result. My first pass
flagged 12 Family 3 rows as fails. That was my checker's error: `--json` puts errors on
stdout. When I re-read the rows, all 12 were `RepeatedIndex`.

### 2. derive: passphrase reaches the derivation

- Oracle: `derive --in card --passphrase-stdin --json` with `TREZOR` gives `b4e3f5ed`. The
  same card with no passphrase gives a **different** fingerprint, so the comparison can
  detect a dropped passphrase.
- Entropy source `{ms1 positional, --hex, --phrase}` × channel `{argv, --flag=value, -,
  omitted (ms1 only), --in (ms1 only)}` × override on/off. Every admitted-argv or `--in`
  case equals the oracle. Every case with entropy on genuine stdin is refused with
  `cannot read both the entropy source and --passphrase from stdin`. That includes
  `--allow-argv-secret --hex -` and a bare `--allow-argv-secret --passphrase-stdin` with the
  ms1 omitted.
- The admitted `--passphrase TREZOR` (argv) path, with `--in` or an admitted ms1, also
  equals the oracle.

### 3. verify (`vd_matrix.py`)

Card channel `{argv, -, omitted, --in}` × phrase channel `{argv, --phrase=, -}` × card
`{P, Q}` × phrase `{P, Q}` × override on/off. Combinations that put material on argv without
the override are skipped. **0 mismatches:**

- A matching pair exits 0.
- A wrong phrase or wrong card fails.
- Every case with both inputs on stdin is refused with `cannot read both ms1 and --phrase
  from stdin`.

### 4. A user-typed `-` with `--allow-argv-secret`

- `verify --allow-argv-secret - --phrase <P>` with the ms1 on stdin, and `--allow-argv-secret
  <ms1> --phrase -`, both pass. That is covered in the matrix.
- `--allow-argv-secret --phrase <P>` with no positional reads the ms1 from stdin: `OK:
  round-trip valid`.
- **Can an admitted value be mistaken for a `-`, or the reverse?** On single-value channels,
  no. Clap rejects a second value (exit 64) for:
  - `verify <ms1> -`
  - `verify <ms1> <ms1>`
  - two `--phrase` flags
  - `derive <hex> --hex -`
  - `derive --passphrase X --passphrase-stdin`

  On combine (the only multi-value channel), a user's `-` placed before the placeholders
  takes an admitted share, and a later placeholder performs the real stdin read. The share
  set is the same either way, which Family 1 confirms across every position.
- Cross-channel misrouting fails loudly. For example, `verify --allow-argv-secret "<12
  words>" --phrase -` puts a phrase-shaped token on `<positional>` and errors with `string
  length 82 not in v0.1 set`. A spaced `' -'` is not treated as stdin; it is parsed as a
  phrase and errors with `word count 1 invalid`.

### 5. Mutations (scratch copy via `git archive a77c8e4`; each asserted APPLIED by exact-count replace plus a diff)

Run over the four binaries `f679_admitted_is_not_stdin`, `freed_stdin`, `cli_combine` and
`allow_argv_secret` (34 tests; baseline 34/34 green):

| mutation | result |
|---|---|
| M1 (re-run): verify guard `if false && ms1_src.reads_stdin() && phrase_reads_stdin` | **killed**, 3 red: `verify_genuine_stdin_twice_is_still_refused`, `freed_stdin::both_channels_on_stdin_are_still_refused`, `freed_stdin::verify_can_read_a_card_and_a_phrase_in_one_invocation`. Each asserts `cannot read both ms1 and --phrase from stdin` |
| M4 (re-run): combine, admitted `-` also sets `consumed_stdin = true` | **killed**, 1 red: `combine_admitted_share_plus_genuine_stdin` (`not enough shares: have 1, need 2`) |
| **R1 (mine): combine `admitted.next()` → `admitted.clone().next()`**, so every placeholder takes share #1 (the "share used twice" defect) | **killed**, 3 red: `allow_argv_secret::the_override_reaches_every_shape`, `f679::combine_admitted_shares_only`, `f679::combine_admitted_share_plus_genuine_stdin` (`share index 'q' repeated`) |

Proof that the line ran: each mutation changes runtime behaviour on exactly the edited line,
and the red tests fail with the message that line's new behaviour produces. A line that was
never reached cannot turn a test red. After the runs, the scratch `crates/` is byte-identical
to the worktree's (`git diff --no-index --stat` is empty).

### 6. Release hygiene: the 0.19.1 CHANGELOG section

I checked each F-677/F-670 bullet against commit `75f2168` and against the running binary:

- **Help EXAMPLES.** None of the 7 named verbs puts material on argv. Every line uses
  `--in` or `-`.
- **`encode --group-size`.** The help now says it shapes only the stderr engraving card.
- **`split --out` advisory.**
  - `split --out F`: stdout is 0 bytes and there is no advisory.
  - `split --out F --json`: the JSON is on stdout and the advisory fires.
- **`hashlock` `for md compose:` fragment.** It reads `--wrapper wsh --path <your other
  paths> --path keyless,sha256=<hex> --experimental --md-only`.

All four bullets are accurate. For the F-679 bullet's wording, see Nit N1.

## Findings

### Critical
None.

### Important
None.

### Minor

**M1: the derive oracle test does not prove it can tell "passphrase applied" from "passphrase
dropped".** `f679_admitted_is_not_stdin.rs::derive_oracle` computes the expected fingerprint
through the same `--passphrase-stdin` mechanism the rows under test use. It never asserts that
the oracle differs from the no-passphrase fingerprint. If `--passphrase-stdin` ever silently
became a no-op, every row would still equal the oracle and stay green. It is not a defect
today: measured, `TREZOR` gives `b4e3f5ed` and no passphrase gives a different fingerprint.
Other suites may pin passphrase vectors, but I did not check that. Reproduction: read lines
139–179. Fix: one `assert_ne!(derive_oracle(), <no-passphrase fingerprint>)`.

### Nit

**N1: the CHANGELOG F-679 bullet says `f679_admitted_is_not_stdin.rs` "covers argv+argv,
argv+stdin, `--in`, and stdin+stdin for all three verbs".** combine has no stdin+stdin row, and
none is meaningful: extra `-` markers read stdin once, which is intended. The rows present for
combine are argv+argv, admitted+genuine stdin (both orders) and `--in`. Reproduction: `grep -n
'fn combine' crates/ms-cli/tests/f679_admitted_is_not_stdin.rs`. Suggested fix: "…and
stdin+stdin for verify and derive".

## Out of scope, noted only

- A material-shaped value after a *non-secret* flag is admitted onto `<positional>` and
  rewritten to `-`. An example is `combine --allow-argv-secret --in <file-named-like-a-share>`,
  which becomes `--in -`. This existed before this change, and every such case I tried fails
  loudly (file-not-found, clap conflict, or codec error). It is not a silent wrong secret.

## Scratch

The harnesses (`combine_matrix.py`, `vd_matrix.py`, `mutate.py`) and the target dirs were under
`/scratch/code/shibboleth/review-ms0191-scratch/`. They were deleted after the run.

ready to ship: yes
