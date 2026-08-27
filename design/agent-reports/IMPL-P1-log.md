# IMPL-P1 — implementation log, `mt` adopts `mnemonic-io-lib`

**Written 2026-08-27 by the P1 implementer.** Worktree
`/scratch/code/shibboleth/_work/p1impl/mnemonic-transaction`, branch `impl/p1`,
based at `cf17591`.

**Rows 1–4 of 12 are built, committed and green. Rows 5–12 are NOT built, and
that is this log's headline finding rather than an apology** — see §3.

---

## 1. STATUS BY ROW

`IMPLEMENTATION_PLAN_P1_mt_adopts.md` §4 is the ordering of record. Prose names
work by NAME, per that table's own instruction.

| row | name | status |
| --- | --- | --- |
| 1 | the tree-greening | **DONE**, commit `566d8e3` |
| 2 | the dash | **DONE**, commit `b3044e8` |
| 3 | the normalisation | **DONE**, commit `2cdc32a` |
| 4 | the override | **DONE**, commit `a4cdefa` |
| 5 | the fish recipe | **NOT STARTED — forbidden by the implementer's brief** |
| 6 | the private write | **NOT STARTED — forbidden by the implementer's brief** |
| 7 | the pin | **NOT STARTED — forbidden by the implementer's brief** |
| 8 | the purge swap | **BLOCKED by the pin** |
| 9 | the fd adoption | **BLOCKED by the pin** |
| 10 | the out channel | **BLOCKED by the pin** |
| 11 | the wording | **BUILDABLE TODAY — not built, because it is out of order** |
| 12 | the decline | **BUILDABLE TODAY — not built, because it is out of order** |

**Rows 11 and 12 are not blocked, and calling them blocked would hide the
controller's cheapest option.** Neither needs the crate. Both are held only by
the table's *"no entry begins until the previous is green"*.

---

## 2. THE FOUR ROWS THAT WERE BUILT

### Row 1 — the tree-greening (`566d8e3`)

Two of `mt`'s seven CI gates were RED at `cf17591`, and the second was invisible
behind the first: CI fails at `cargo fmt --check` and reports every later step
**skipped**, so clippy, build, the 237 tests, both refusal gates and the
journeys had **not executed on that commit at all**.

- `cargo fmt` over the tree. Measured RED first: exit 1, **235 diff lines**,
  7 files. No behaviour change.
- `SPEC_engrave §2.2` deleted from `scripts/check-refusal-coverage.sh`'s
  `REQUIRED` list. Measured RED first: exit 1,
  `the seeded list rules SPEC_engrave §2.2 and no entry claims it`. The rule was
  seeded 2026-08-25 with the `me tx` → `mt encode --record` graft and RETIRED
  when `--record`/`--raw`/`--chunks` collapsed into a single `--qr`; the
  reasoning was written into `tests/refusals.toml:261` at the point of
  retirement and this list was not updated with it. No test is lost —
  `the_retired_record_family_is_unknown_to_the_parser` holds that line.

Regression-gated, and the gate is not two greens but **seven steps that RUN**.
All seven ran and all seven passed.

### Row 2 — the dash (`b3044e8`)

RED first, measured on the built binary by absolute path, exit codes read
directly and never through a pipe:

```
mt decode  - < strings   -> rc 2, error: unexpected argument '-' found
mt verify  - < strings   -> rc 2, same
mt inspect - < strings   -> rc 2, same
```

**One field, three verbs.** `decode`, `verify` and `inspect` all derive from the
shared `ReadArgs`, so this is `EncodeArgs`' hidden positional copied onto it
verbatim: `value_name = "-"`, `value_parser = ["-"]`, `hide = true`.

Asserted as **equality of stdout AND stderr** against the flagless run, which is
the shape F-250 already uses — a bare `success()` would also pass for a `-`
silently taken as a filename, or one that suppressed the report. Plus F-250's
control on all three verbs: a non-dash positional is still an error.

Both tests live in `decode_verify.rs`, including `inspect`'s rows. The unit
under test is the one shared field, and splitting the gate across two suites
would let a later edit green one half while the other regressed.

### Row 3 — the normalisation, F-274 (`2cdc32a`)

`looks_like_a_transaction` lowercases for its `mt1` arm and pins the hex arm to
an even count of hex digits. **It never trimmed.** One stray space made a bearer
artifact unrecognisable to the pre-clap guard; it fell through to clap, and clap
echoed it verbatim — the exact leak §8.2f exists to prevent, at exit 2 instead of
1, with no refusal and no purge advice.

Reproduced before the fix over the plan's generated grid, 4 verbs × 2 carrier
classes × 4 spellings = 32 rows: **16 leaked**, both whitespace spellings, on
every verb, for both classes. The plan's figure reproduces exactly.

The fix is **one `trim()`**, in `command_line_guard`, before the recogniser is
consulted — plus `debug_assert_eq!(a, a.trim())` inside the recogniser so a
second caller that forgets it fails loudly in every test and dev build rather
than silently re-opening the leak. Trimming in two places is how two copies
drift apart; trimming in neither was the defect.

The refusal now reports the **normalised** character count. Quoting the padded
length would give an operator comparing it against what they pasted a number
that matches nothing. No test pinned the old count — checked, not assumed.

The near-miss the charset test exists for is unchanged and green:
`mt verify --in mt1-2026-08-23-cold-storage-transfer.txt` is a filename, and
`-`/`.` are outside the bech32 alphabet, so trimming does not widen the
recogniser onto it.

Listed in `refusals.toml`, so `mutate-refusals.sh` covers it: neuter
`command_line_guard` and all 32 rows leak.

### Row 4 — the override, §6d (`a4cdefa`)

`mt` had no override at all. §6d rules that it gains one, and that the
override's **own parse** runs on raw argv.

**Why the obvious implementation is worse than no override.** `me` can wire this
as an ordinary clap flag because `me sysw pack` HAS a `records` positional to
hand the admitted token to. **No `mt` verb takes material positionally.** The
only positional on any `mt` verb is the hidden `[-]`, whose `value_parser`
admits the literal `-` and nothing else — so an override that admits a
transaction and leaves it in argv turns a clean exit-1 refusal into
`error: invalid value '<the whole transaction>' for '[-]'` at exit 2.

So the pre-parser **strips both the override and every token it admits** from
the argv clap sees, and carries the material in as if it had arrived by `--in`:

```
validate::argv_intake(&argv) -> ArgvIntake { argv, material }
validate::command_line_guard(&intake.argv)     <- the STRIPPED argv
Cli::parse_from(&intake.argv)
```

The override "proceeds" because what it admitted is no longer there to be
refused, and anything it did not admit still is. **One recogniser decides both**
— `looks_like_a_transaction`, on the same normalised candidate — so the strip
set and the refuse set cannot drift; a token the guard refuses but the stripper
misses is a token clap gets to echo.

Declared on all four verbs so `--help` documents it, and always `false` by the
time clap fills it in. `mt` needs no surface check (`me` has one, after a flag
value was found impersonating a subcommand word) because every `mt` verb reads
bearer material and every one declares the flag.

**MUTATION-CHECKED, because the discriminating test is easy to write wrongly.**
Making `argv_intake` also keep the admitted token — the naive implementation —
turns two of the four tests RED:

```
the_argv_override_routes_material_through_the_private_path     FAIL
the_argv_override_strips_the_material_from_the_argv_clap_sees  FAIL
the_argv_override_alone_is_the_bare_invocation                 PASS (control)
the_argv_override_is_documented_on_every_verb                  PASS (control)
```

The second only discriminates because of **token order**: the unknown flag comes
AFTER the material, so the naive build reaches the `[-]` positional first and
echoes the transaction. With the unknown flag first, clap errors on it before
reaching the value and **both worlds pass** — a test that passes in both worlds
is not a test.

**One ambiguity ruled at implementation time, not left silent.** §6d does not
say what happens when `--in` and admitted argv material are both given. `mt`
now prefers the FILE — private, and explicitly named — and warns, naming the
discarded material's length and the path read, never the material. Filed as
**F-277** so the spec rules it once rather than four more implementers each
inventing an answer.

---

## 3. WHY ROWS 5–12 WERE NOT BUILT — the finding

### 3.1 Rows 5, 6 and 7 are forbidden by the implementer's brief

The brief is explicit: *"Write your code only [in the `mt` worktree]"* and
*"Do not touch … `mnemonic-engrave`'s `crates/`."*

- **Row 5, the fish recipe** — edits `crates/mnemonic-io-lib/src/remedy.rs`.
- **Row 6, the private write** — moves `write_private` out of
  `crates/me-cli/src/main.rs` into `crates/mnemonic-io-lib`.
- **Row 7, the pin** — pushes `mnemonic-engrave` `master` to GitHub through the
  `ci/staging` ref. In substance it would publish rows 5 and 6, and it needs a
  `master` FREEZE window the implementer cannot guarantee while the controller
  is working in that repo.

Measured rather than assumed, 2026-08-27, in `/scratch/code/shibboleth/mnemonic-engrave`:

```
git ls-tree -d origin/master crates/     -> crates/me-cli   (that line, alone)
git rev-list --count origin/master..master -> 67
```

So the crate is not on `origin/master`, local `master` is 67 commits ahead, and
**no git-rev pin resolves today.** The plan's §3 said 64; it is 67 now.

### 3.2 Rows 8, 9 and 10 are blocked BY row 7

Each is `mt`-side but each calls into `mnemonic-io-lib`, so each needs the
dependency the pin creates. None can be started.

### 3.3 Rows 11 and 12 are buildable today, and were held only by the ORDER

Neither needs the crate. The plan's own §2.3 says of row 11 in as many words
that *"the wording work below is written in `mt`, and the question of hoisting
it is filed rather than built"*, because `me`'s mask is `0o044` and `mt`'s is
`0o077`, so the sentence differs per tool. Row 12 is regression-gated and writes
no production code at all.

They were **not** built. The brief says an unbuildable row is the finding and
*"do not invent a different plan and build that"*, and resequencing a table
declared *"the only ordering of record"* is the controller's call. This was put
to a `fable` consult, which ruled the same way.

**One interaction the controller should know before resequencing.** Rows 9 and
11 edit the SAME TWO SITES. Row 9 changes the input warning's *keying*
(`is_file()` → the crate's `mode_of`); row 11 changes both sites' *wording* to be
derived from the measured mode. The plan sequenced 9 first, so row 11 as written
assumes a site already re-keyed. Building 11 first lands it on the `is_file()`
baseline — a variant of the row, not the row — and creates a fold hazard for
whoever does 9 afterwards.

---

## 4. MEASUREMENTS TAKEN FOR THE BLOCKED ROWS, so they are not paid for twice

All against the binary at `target/debug/mt` by absolute path, rebuilt
immediately beforehand, exit codes read directly.

**Row 11's defect, both halves, reproduced.** The refusal and the warning each
hard-code the rule's NAME instead of describing what was measured:

| stdout mode | `mt encode --in <psbt>` | what it says |
| --- | --- | --- |
| 0600 | rc **0**, 796 bytes | (no refusal — the control) |
| 0620 | rc 1, 0 bytes | *"its permissions grant read to group or others"* — **false**, `0620 & 0o044 == 0` |
| 0644 | rc 1, 0 bytes | same sentence, and here it is true |
| 0666 | rc 1, 0 bytes | same sentence, true |

| input mode | `mt encode --in <file at that mode>` | what it says |
| --- | --- | --- |
| 0600 | no warning | the control |
| 0620 | warns | *"grant read to group **and** others"* — **false twice over** |
| 0644 | warns | *"and others"* — the group half is false |

**The input warning is worse than the refusal**, exactly as row 11 says: it says
*and* where the refusal says *or*.

**Row 9's missing case, reproduced.** A **named** fifo at mode 0666 passed as
`--in` produces **no mode warning at all** (rc 0, zero lines mentioning a mode) —
`file_mode_warning` still keys on `is_file()`, which `mt`'s own source twelve
lines away already records as measured false.

**A number in the plan that did not reproduce, flagged rather than chased.**
Row 9's cell says *"stdout mode 0600 exits 0 and writes 682 bytes"*. Against
`tests/fixtures/p5_base.json`'s finalized PSBT it is **796** bytes — which is
also the figure `mnemonic-io-lib`'s own `lib.rs` doc table carries for
`mt encode`. Almost certainly a different fixture, not a defect; whoever builds
row 9 should pin the fixture rather than the byte count, or state which one 682
came from.

---

## 5. GATES — the whole validation surface, not the tests alone

Final state of `impl/p1` at `a4cdefa`. Each exit code read directly, never
through a pipe. `cargo build` was run immediately before every behavioural
measurement in this log.

| gate | result |
| --- | --- |
| `cargo fmt --check` | **0** |
| `cargo clippy --all-targets --locked -- -D warnings` | **0** |
| `cargo build --locked` | **0** |
| `cargo nextest run --locked` | **0** — `Summary [0.160s] 245 tests run: 245 passed, 0 skipped` |
| `scripts/check-refusal-coverage.sh` | **0** — `33 refusal tests over 18 ruled refusals` |
| `scripts/mutate-refusals.sh` | **0** — `all 33 refusal tests go red when their check is removed` |
| `scripts/journeys.sh` | **0** — `A, B (both forms) and C all pass on what the operator SEES` |

Baseline was **237**; the four rows added **8** tests and no test was deleted or
weakened. The refusal ledger went 32 → 33.

```
 crates/mt-cli/src/blocks.rs          |   8 +-
 crates/mt-cli/src/main.rs            | 176 +++++++++++++++++----
 crates/mt-cli/src/validate.rs        | 130 +++++++++++++++-
 crates/mt-cli/tests/decode_verify.rs |  76 ++++++++++
 crates/mt-cli/tests/encode.rs        |  18 ++-
 crates/mt-cli/tests/inspect.rs       |   5 +-
 crates/mt-cli/tests/refusals.rs      | 285 ++++++++++++++++++++++++++++++++++-
 crates/mt-cli/tests/refusals.toml    |  12 ++
 crates/mt-cli/tests/tx_record.rs     |  56 +++++--
 scripts/check-refusal-coverage.sh    |  15 +-
 10 files changed, 717 insertions(+), 64 deletions(-)
```

`blocks.rs`, `encode.rs`, `inspect.rs` and `tx_record.rs` are touched by row 1's
`cargo fmt` **only** — no behaviour, no assertion.

### The test diff, enumerated, each edit justified

Row 12 asks for this over the whole phase; here is the part that exists.

| test | file | change | justified by |
| --- | --- | --- | --- |
| `a_bare_dash_means_stdin_on_every_reading_verb` | `decode_verify.rs` | **added** | §6b's `-`, F-250 extended to the reading verbs |
| `a_dash_does_not_open_the_door_to_other_positionals_on_reading_verbs` | `decode_verify.rs` | **added** | F-250's own control, carried across |
| `no_spelling_of_a_bearer_argument_reaches_stderr` | `refusals.rs` | **added** | F-274 |
| `the_argv_override_routes_material_through_the_private_path` | `refusals.rs` | **added** | §6d — *"the same internal path as `--in` content"* |
| `the_argv_override_strips_the_material_from_the_argv_clap_sees` | `refusals.rs` | **added** | §6d — *"never re-presented to clap as a positional"* |
| `the_argv_override_alone_is_the_bare_invocation` | `refusals.rs` | **added** | control for the two above |
| `the_argv_override_is_documented_on_every_verb` | `refusals.rs` | **added** | §6d — *"greppable, so a reviewer can find it"* |
| `material_on_argv_beside_an_in_file_is_warned_about_not_dropped` | `refusals.rs` | **added** | F-277, ruled at implementation time |

**No existing test was edited, weakened or deleted.** Every prior assertion in
the suite still holds unchanged — including
`the_world_readable_refusal_names_the_artifact_this_run_made`, which row 10 is
the one scheduled to change.

---

## 6. FOLLOW-UPS FILED

In `mnemonic-engrave`'s `design/FOLLOWUPS.md`, own commit.

- **F-274 → CLOSED** by row 3, with the closure recorded in the HEADING per that
  file's own convention. Its stated **residue is explicitly NOT closed**: a
  99-character odd-length hex string is still below the recogniser's threshold
  and is still echoed. That remains the F-267 class and row 3 did not touch it.
- **F-277 — NEW.** §6d rules the override's parse and its routing and is
  **silent on the collision with `--in`**. `mt` had to invent an answer to build
  the row. Owning phase: the spec, before P2 gives a second tool the override —
  `ms` alone has eight verbs gaining `--in`, and an implementer who answers it
  by silently preferring argv builds the opposite of `mt`.
- **F-278 — NEW.** F-275 is **ruled but scheduled nowhere.** The operator ruled
  it the same day the plan was written — WARN, do not refuse; `mt decode` prints
  a warning naming the measured mode and proceeds at exit 0 — and **no row in
  the twelve-row table builds it**, while F-275's own heading still carries the
  pre-ruling owning phase *"a ruling the operator owes"*. That is the state a
  burndown sweep is least likely to catch: a reader concludes correctly that it
  is waiting on the operator, which it is not. It needs a row, not a design;
  `file_mode_warning` already has the shape, and lacks only a caller on the
  reading verbs and a stdout-side sibling.

---

## 7. WHAT I WOULD DO NEXT — offered, not decided

1. **Rule the crate-side question first**, because everything after row 4 hangs
   on it. Rows 5, 6 and 7 need someone who may write in `mnemonic-engrave`'s
   `crates/` and may push `master`. Nothing in `mt` unblocks them.
2. **If rows 5–7 stay out of reach**, rows 11 and 12 are a genuine partial
   delivery and cost nothing — but resequence **9 before 11** deliberately, or
   accept that row 11 lands on the `is_file()` baseline and that row 9 will then
   be folding on top of it.
3. **Row 12's decline assertions are worth landing early regardless.** They are
   backstops that protect the adoption rows from importing `me`'s policy, and a
   backstop written after the thing it guards has already been built has guarded
   nothing.
4. **F-278 is small, mt-local, ruled and unscheduled.** It is the cheapest open
   item on the board and it needs no crate.
