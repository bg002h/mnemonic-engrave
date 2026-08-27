# IMPL-P1-crate — rows 5 and 6, the `mnemonic-io-lib` half of P1

**Branch:** `impl/p1-crate`, worktree `_work/p1crate/mnemonic-engrave`.
**Base:** `ba1f3ec`. **Built 2026-08-27.** Both rows GREEN.

| commit | what |
| --- | --- |
| `2efb1b9` | row 5 — the fish purge recipe |
| `93653e3` | persist: fable consult on row 6's module placement |
| `394a39d` | followups: F-280 |
| `54b7943` | row 6 — `write_private` moves into the crate |

`git diff --stat ba1f3ec..HEAD` → 9 files, **660 insertions, 63 deletions**.

---

## Row 5 — the fish purge recipe (F-273)

### The control passed, so the harness is a harness

`crates/mnemonic-io-lib/tests/fish_history_purge.rs`. `script -qc "fish -i"`
fed from a command file, isolated `HOME`/`XDG_DATA_HOME`/`XDG_CONFIG_HOME`,
exactly the shape the plan recorded. Control planted, no purge attempted:
**secret on disk, 1 hit; neighbour on disk, 1 hit.** Run first, and it passes.

This is the row that was deferred once because a control would not pass. It
passes.

### The three measurements the plan asked to reproduce — all three did

fish **4.8.1**, this machine, every figure from this harness:

| attempt | outcome |
| --- | --- |
| *(control — no purge)* | secret on disk, **1 hit** |
| `history clear-session` | **0 hits** |
| `history delete --prefix '<command>'` | **rc 124, killed at 30s, entry still on disk** |
| `history delete --contains '<command>'` | rc 124 at 30s, entry still on disk |
| `history delete --exact '<full line>'` | *"requires --case-sensitive"*, **`$status` 0**, still in memory AND on disk — secret now in history **twice** |
| `history delete --exact --case-sensitive '<full line>'` | purges — by typing the secret a second time |
| the recipe's cost | the unrelated neighbouring command is **also** removed, 0 hits |

**One correction to the harness, none to the finding.** The plan's control
plants `echo mt encode <SECRET>`. A `--prefix 'mt encode'` search against a line
beginning `echo ` matches nothing, so the first run returned **rc 0 in 1s having
found nothing** rather than hanging — my harness's defect, not fish's. Planted
as an operator would actually type it (`<command> <secret>`, command first), the
prefix matches and the hang reproduces at rc 124/30s precisely as recorded. The
committed harness plants it that way. **fish records a command it cannot
resolve**, so the planted line does not need to be a real binary — and the
committed constant deliberately is not one, because `me` and `mt` are both on
`PATH` here and a harness that invokes them is measuring them too.

### One measurement that is NOT in the plan, and it decides the recipe

`--exact` is the spelling fish's manual says does not prompt, so it is what a
reader reaches for on being told `--prefix` hangs. Measured in-session:

```
builtin history delete --exact requires --case-sensitive
RC=0
INMEM:example-cli pack ms1SECRETSECRETPLANTED
```

**It complains, exits 0, and the entry is still in memory and still on disk.**
That is F-264's shape exactly — reports success, purges nothing — in the module
whose header exists to warn about it. Spelled correctly with
`--case-sensitive`, it *does* purge, and the secret is in the history file
anyway because the recipe **is** a line containing it. That is now a committed
test, so the next reader cannot tidy `clear-session` into something "more
precise" without going RED.

### And one more, because the emitted text makes a claim

`history clear-session` does **not** reach an earlier session: planted and saved
in session A, `clear-session` run in a fresh session B, **1 hit before and 1
after**. The emitted text says so, alongside the whole-session cost.

### `TERM` is load-bearing and cost 10s per session

Under `TERM=dumb` the whole session runs in **under 1 second** and the control
still passes — and `history delete --prefix` **stops prompting**, returns rc 0,
and still deletes nothing. A harness tuned that way measures a different
program. The committed harness sets `TERM=xterm` and pays fish's ~10s wait for
a Primary Device Attribute response the pty never sends. `fish_features=no-query-terminal`
was tried and did not remove the wait.

### The standing invariant holds

`history_d_is_named_as_a_warning_and_offered_as_no_recipe`
(`crates/me-cli/tests/history_purge.rs:151`, **citation verified sound**) passes,
and now iterates the fish recipe too: `history -d` is NAMED in the prose and
OFFERED by no recipe.

---

## Row 6 — `write_private` moves into the crate (F-244)

**Located by symbol, and the citation was sound**: `fn write_private` was at
`crates/me-cli/src/main.rs:1079`, exactly where the plan says. 21 lines.

### The RED is the row

Run first against a mode-on-create-only body — `OpenOptions::mode(0o600)` and no
`set_permissions`:

```
FAIL  write::tests::an_existing_world_readable_target_is_tightened_not_inherited
assertion `left == right` failed: F-244: `0o600` binds on CREATE, ...
  left: 420      (0o644)
 right: 384      (0o600)
```

The **fresh-file half of the same test PASSED** against that naive body, which
is why it is not the gate. The pre-existing-0644 half is, and it fails exactly
as the plan's gate column says a mode-on-create implementation must. The test
also pins the contents, so a function that tightened the mode and wrote nothing
could not pass it.

### Placement — a new `write` module, no root re-export

Consulted (`design/agent-reports/CONSULT-P1-row6-module-placement.md`, committed
before this fold). `channel`'s own header says `destination` **never touches a
path** and `fd`'s says it is *"what was measured about stdout"*; admitting an
effectful path-writer to either means rewriting the sentence that makes the
module a boundary. The root re-export set is already partial — `fd`,
`observation` and `remedy` are module-qualified only — so adding to it moves the
inconsistency rather than removing it, and this is the wrong week to mint a
second public path.

### The boundary, checked mechanically

```
grep -rn "EXIT_"  crates/mnemonic-io-lib/src/ , comment lines stripped -> 0 hits
grep -rnw "Class" crates/mnemonic-io-lib/src/ , comment lines stripped -> 0 hits
[dependencies] in the crate's Cargo.toml                              -> still empty
```

`0o600` is a constant and not a parameter. `write.rs` states why that is a
different kind of thing from the `0o044`/`0o077` masks `fd` refuses to publish:
those are a disagreement about **somebody else's** file. What mode a tool
creates **its own** output at is not disputed by either consumer.

### What the diff falsified elsewhere

Four comments in three files claimed the function lived in `me`. All swept:
`channel.rs` (*"`write_private` stays in `me`"*), `main.rs` (*"sat in this same
file"*), `world_readable_output.rs:53` (*"its own comment concedes"* — the doc
no longer concedes, and the stale *"accepted residual"* note went with the
move), and `world_readable_output.rs:230`.

The **reason** stayed with `me` on purpose: naming md1/mk1 material is `me`'s
job. The crate is told to create a file owner-only; it is not told what is in it.

### `me`'s suite unchanged in meaning — measured

The entire diff under `crates/me-cli/tests/` contains **zero non-comment
lines**. `world_readable_output.rs`'s end-to-end F-244 test stays: it runs the
real binary, so it pins that `--out` still routes through the function at all,
which a crate unit test cannot see.

---

## Gates — each run as its own command, exit codes read from a file

```
cargo build --locked                                                rc 0
cargo nextest run --locked
    Summary [  33.615s] 430 tests run: 430 passed, 1 skipped        rc 0
cargo clippy --all-targets --locked -- -D warnings                  rc 0
cargo fmt --check                                                   rc 1   <- F-280, pre-existing
```

Test count: **423** at `ba1f3ec` → **428** after row 5 (+5 fish) → **430** after
row 6 (+2 `write.rs` units).

---

## FINDING — `cargo fmt --check` is RED at the baseline, and CI cannot see it

Filed as **F-280**. Measured at `ba1f3ec` by stashing the tree and using CI's
own pinned toolchain, so this is not a nightly-rustfmt artifact:

```
cargo +1.85.0 fmt --check   ->  exit 1,  77 hunks,  14 files
```

A grep for `fmt` or `clippy` across this repo's `.github/workflows/` returns
**no hits at all**. `release.yml` runs `cargo test --locked` and the Go suites
and nothing else. The sibling `mnemonic-transaction` runs both
(`.github/workflows/ci.yml:16`–`:18`), which is why P1's row 1 could gate on
`fmt` there and cannot here.

**Not fixed, deliberately** — a ~1200-line whitespace diff across 14 untouched
files, in the window immediately before `master` is pushed to earn the SHA `mt`
will pin by rev, buys nothing and risks the pin. F-280 records the order that
makes it stick: **CI steps first, reformat second.**

**These two rows added no fmt debt.** The final per-file hunk distribution is
byte-for-byte identical to the baseline's — 41 `sysw_cli.rs`, 8 `main.rs`, 7
`sysw/mt.rs`, 5 `cli.rs`, … and the one pre-existing hunk in `remedy.rs` is on a
line row 5 did not write. Both new files are clean under `1.85.0` **and**
nightly `rustfmt`.

---

## Citation audit — all 16 engrave-side citations were SOUND, and I moved 7

F-279 measured 14 of 15 **`mt`** citations stale. The `mnemonic-engrave` half is
the opposite: resolved at `ba1f3ec`, **16 of 16 landed on exactly the symbol the
plan names**, `main.rs:1079` for `write_private` included.

**Rows 5 and 6 have now moved 7 of them.** New locations, by symbol:

| plan cites | at `ba1f3ec` | now |
| --- | --- | --- |
| `me-cli/src/main.rs:1079` `write_private` | sound | **`mnemonic-io-lib/src/write.rs:45`** — no longer in `main.rs` at all |
| `me-cli/src/main.rs:1117` the `0o044` mask (row 9's model) | sound | `me-cli/src/main.rs:1093` |
| `me-cli/src/main.rs:1259` `refuse_world_readable_stdout` (§2.3) | sound | `me-cli/src/main.rs:1235` |
| `mnemonic-io-lib/src/channel.rs:33` `destination` | sound | `channel.rs:40` |
| `mnemonic-io-lib/src/lib.rs:74` root re-exports (§3) | sound | `lib.rs:81` |
| `mnemonic-io-lib/src/remedy.rs:66` `history_purge_recipes` | sound | `remedy.rs:79` |
| `mnemonic-io-lib/src/remedy.rs:113` `history_purge_block` | sound | `remedy.rs:144` |

Unchanged and still sound: `fd.rs:46`/`:73`, `exit.rs:41`/`:63`,
`observation.rs:47`, `records.rs:16`/`:52`, `history_purge.rs:108`/`:151`.

**Row 9 and §2.3 are the ones this bites**, because `main.rs:1117` and `:1259`
are cited as models for later work and both now point at doc-comment lines —
the same plausible-and-wrong shape F-279 describes. F-279 owns re-anchoring;
this table is the engrave-side input to it.

---

## Consults

**One**, `model: fable` — *where should `write_private` live in the crate's
public API, and should it be re-exported at the root?* Answer: a new `write`
module, no root re-export. Persisted verbatim to
`design/agent-reports/CONSULT-P1-row6-module-placement.md` and committed in its
own commit (`93653e3`) before the fold. Three of its load-bearing claims were
re-checked against the source by hand before acting on them; all three held.

## Follow-ups filed

**F-280** — the tree is `fmt`-RED at 14 files and CI runs no `fmt` or `clippy`
step. Owning phase: **after the P1 rev-pin push**. It also records a sibling
check owed to any constellation repo whose workflow was copied from this one
rather than from `mt`'s.

## What I could not do, and why

- **`cargo fmt --check` cannot be made GREEN inside these rows.** It is RED at
  the baseline for reasons that predate them; making it green means reformatting
  14 files these rows never touch. Filed rather than done — see above.
- **Nothing else.** Both rows built as written; neither needed a substituted
  design.

## For whoever takes rows 7–13

- The crate-side work is complete, so **row 7's push can proceed**: every change
  P1 makes to `mnemonic-io-lib` is on `impl/p1-crate` and needs a single pin.
- `mnemonic_io_lib::write::write_private` is the path. Not re-exported at the
  root, matching how `mt` already reaches `fd` and `remedy`.
- `history_purge_recipes` now returns **three** pairs. Row 8's purge swap gets
  fish for free, and `mt`'s own harness should carry this file's control.
- The seven moved citations above are input to F-279, not new drift to file.
