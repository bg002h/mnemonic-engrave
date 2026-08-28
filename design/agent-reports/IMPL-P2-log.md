# IMPL-P2 — `ms` adopts `mnemonic-io-lib`, executed

**Date:** 2026-08-27. **Plan:** `design/IMPLEMENTATION_PLAN_P2_ms_adopts.md`
(14 rows, 18 closure conditions), R0 GREEN at 0C/0I.
**Branch:** `impl/p2` in `/scratch/code/shibboleth/_work/p2impl/mnemonic-secret`,
from `7c12f66`. **Cross-repo work** landed on `master` in `mnemonic-engrave`.

**All 14 rows are built and green. Two rows deviated from the plan's wording and
both deviations are stated below with their measurements. One row (11) could not
be built exactly as written; the reason is a defect the row uncovered.**

---

## 1. The headline

`ms encode --phrase "<a real seed>"` exited **0 in silence**. It now exits 1,
refused before the command line is parsed, naming the class and the length and
never the value, and carrying a purge recipe that has been RUN under real
interactive shells and observed to purge.

Measured before any code, with the command beside it: `git grep -n 'env::args'`
and `git grep -n 'fs::write\|OpenOptions\|set_permissions\|0o600\|0o077\|0o044\|st_mode'`,
both scoped to `crates/`, both **zero hits**. This was the first installation of
the mechanism on `ms`, not a port.

---

## 2. Rows, with gate results

| # | work | gate result |
| --- | --- | --- |
| 1 | `--in FILE` on the six unambiguous verbs | RED: 6/6 exited **64**. GREEN: `--test in_flag_six_verbs` **10 passed** — stdout AND stderr byte-equal to the stdin run, plus 3 controls |
| 2 | `--in` on `encode`/`split` means a PHRASE | RED: `unexpected argument '--in'` (64). GREEN: **3 passed**, incl. the counterexample (64 legal hex chars refused, naming `--hex - <`) |
| 3 | `--in` frees stdin | Both contention refusals routed around at rc 0; both still fire without `--in`. **4 passed** |
| 4 | pin `mnemonic-io-lib` by git rev | regression-gated. (a) grep 0→1 (b) items compile AND are callable (c) `--locked` build with empty `CARGO_HOME` rc 0 (d) suite green. **Plus a blocking CI consequence the plan does not name — §4** |
| 5 | the argv guard, on RAW argv, before the parser | RED baseline reproduced exactly: **84 of 92 exit 0, 8 non-zero, 0 of 92 leaking**. GREEN: **6 passed**. Mutation: guard disabled → **92 of 92 rows fail** |
| 6 | `--allow-argv-secret` is a CHANNEL | RED: flag did not exist (64). GREEN: **9 passed**, incl. the empty-stdin discriminator with its control |
| 7 | the purge text, RUN | **7 passed** under real interactive zsh and bash on a pty, harness control first |
| 8 | the private write, `--out FILE` | RED: 6 of 7 failed with `unexpected argument '--out'`. GREEN: **7 passed** |
| 9 | the ungrouped stdout | RED: pipeline exit **4**, no payload. GREEN: exit **0**, 118-byte sealed payload; `--out` form 102 bytes at 0600 |
| 10 | the whitespace-only separator | RED: all three separators exited 0. GREEN: **3 passed**; hyphen/comma/`-`/`,` all 64 on both verbs |
| 11 | the sibling remedy | text fixed + **3 passed** with `MS_P2_BIN` set. **Deviation — §5** |
| 12 | the drivers | 13 of 20 invocations migrated; 6 of 7 drivers rc 0; two deterministic key drivers **byte-identical**. One pre-existing failure — §6 |
| 13 | the schema | **55** flags, asserted as arithmetic. **12 passed** |
| 14 | the decline, asserted | **6 passed**, incl. a real pty and a real 0644 fd 1 |

---

## 3. The final numbers

```
$ cargo nextest run --locked
     Summary [   0.402s] 476 tests run: 476 passed, 11 skipped
```

Baseline at `7c12f66` was **414 passed, 5 skipped**. The 11 skipped are the
5 pre-existing `#[ignore]`d mlock/g6 gates plus the 6 new pty-dependent ones,
which the new `history-purge` CI job runs with `--include-ignored`.

```
$ git diff --stat 7c12f66..HEAD
 90 files changed, 5841 insertions(+), 500 deletions(-)
   ...of which vendor/mnemonic-io-lib/ is 11 files
$ git diff --stat 7c12f66..HEAD -- . ':(exclude)vendor'
 79 files changed, 4654 insertions(+), 500 deletions(-)
```

### Every CI step in `ms`'s workflows

| workflow / job | command | rc |
| --- | --- | --- |
| `rust.yml` fmt | `cargo +1.95.0 fmt --all -- --check` | **0** |
| `rust.yml` test-ms-codec | `cargo test -p ms-codec` | **0** |
| `rust.yml` clippy-ms-codec | `cargo clippy -p ms-codec --all-targets -- -D warnings` | **0** |
| `rust.yml` test | `cargo build --tests -p ms-cli` | **0** |
| `rust.yml` test | `cargo test -p ms-cli` | **0** |
| `rust.yml` test G2.1 | `MNEMONIC_TEST_MLOCK_FAIL_MODE=eperm … g2_1 --include-ignored` | **0** |
| `rust.yml` test G2.3 | `…=einval … g2_3 --include-ignored` | **0** |
| `rust.yml` test G2.4 | `…=off … g2_4 --include-ignored` | **0** |
| `rust.yml` test-release-mlock-einval | `cargo test --release … g2_3 --include-ignored` | **0** |
| `rust.yml` miri | `cargo +nightly miri test -p ms-cli --bin ms mlock::` | **0** (7 passed) |
| `rust.yml` clippy — vector pin | `sha256sum -c display-grouping-vectors.tsv.sha256` | **0** |
| `rust.yml` clippy | `cargo clippy --all-targets -p ms-cli -- -D warnings` | **0** |
| `rust.yml` **history-purge (NEW)** | `cargo test -p ms-cli --test history_purge -- --include-ignored` | **0** |
| `rust.yml` **history-purge (NEW)** | `cargo test -p ms-cli --test the_decline -- --include-ignored` | **0** |
| `rust.yml` g6-invariant | `SIBLING_REPO_PATH=… cargo test … mlock_g6_invariant --include-ignored` | **0** |
| `rust.yml` freebsd-compile-gate | `cargo check --target x86_64-unknown-freebsd -p ms-cli` | **0** |
| `vendor-freshness.yml` | `bash ci/repro/vendor-freshness.sh` | **0** |

**Two legs could not be run on this box, and neither is a code gap.**
`musl-check` x86_64 needs `musl-gcc` (`musl-tools`), which is not installed —
`cargo test --target x86_64-unknown-linux-musl` fails in `cc-rs` on
`secp256k1-sys`, before any `ms` code. `musl-check` aarch64 needs `cross` and
Docker, neither present. The `freebsd-compile-gate` DID run and passed, which is
the same cross-target question for the new git dependency (`mnemonic-io-lib` is
pure `std` with no dependencies at all).

`man-release.yml` is tag- and `workflow_dispatch`-triggered only. Its
`musl-binaries` legs are fixed here; its `repro` job is **known-red** — F-324.

---

## 4. Row 4's finding — the git pin breaks the tag-time reproducible build

**F-324**, filed. The plan does not mention vendoring, `vendor-freshness`, or the
reusable repro workflow, and §5's enumeration of *"`ms`'s WHOLE validation
surface"* omits all three. All three turned out to be load-bearing.

`mnemonic-secret` commits a 101 MB `vendor/` tree and builds releases `--locked
--offline` from it. The pin puts the **first** `source = "git+…"` line ever into
`Cargo.lock`, and `source.crates-io` does not serve that key.

**Measured with an EMPTY `CARGO_HOME`** — the isolation is load-bearing, and its
absence produced a **false GREEN on the first attempt**, where the broken
two-block form also exited 0 by resolving from `~/.cargo/git`:

| `--config` form | `cargo build --locked --offline -p ms-cli` |
| --- | --- |
| three-block | **rc 0** |
| two-block (what every release step used) | **rc 101**, `failed to load source for dependency mnemonic-io-lib` |

Two of the three sites are inside `mnemonic-secret` and are **fixed in row 4's
own commit** (per the consult below): `ci/repro/vendor-freshness.sh`, which
failed CLOSED the instant the pin landed exactly as its own comment predicted and
named its own fix; and `man-release.yml`'s two `musl-binaries` legs. Both derive
the rev from `Cargo.lock` rather than hard-coding it. Negative control run: with
`vendor/mnemonic-io-lib/` moved aside, `vendor-freshness.sh` exits 1.

**The third cannot be fixed from this repo.** `man-release.yml`'s `repro` job
calls `bg002h/mnemonic-toolkit/.github/workflows/reproducible-musl-build.yml@6e37b18…`.
Read at that SHA: its only git-source knob is `miniscript_rev`, and the three
`--config` lines it builds hard-code `https://github.com/rust-bitcoin/rust-miniscript`.
There is no input by which a caller can declare a different git source. **`ms`
cannot cut a release tag until a change lands in `mnemonic-toolkit`**, a workflow
shared by `md`, `mk` and `mt`. The job is left CALLED rather than disabled: a
skipped gate prints ok and exit 0.

`cargo vendor vendor/` added exactly ONE directory (11 files) and emitted the
three-block config verbatim; nothing else in the vendor tree churned.

---

## 5. Row 11's finding — the advice branch is unreachable

**F-362**, filed. Row 11 asks for a test that *"extracts the advised line from
`me`'s own stderr, RUNS it, and requires exit 0 and a payload on disk"*.
**The extraction half cannot be done: the branch that prints that line is dead.**

`read_records`'s argv refusal selects the secret-class example only when
`class.is_argv_forbidden()` holds — and that is **exactly** what the pre-parser
`argv_secret_guard` refuses on, at exit 3, before `read_records` runs. Both
layers normalise identically (trim, ASCII-lowercase, `=`-split), so no spelling
gets past one and is caught by the other.

**Measured over eleven argv shapes** (a BIP-39 phrase, an `ms1` in three
spellings, `pass:`, `text:`, three `tx:` forms, `md1`, `mt1`): **0 sightings** of
the `ms encode` advice. The reachable half of that refusal is the `by_prefix`
arm, which always takes the BEARER example.

**What was built instead, and it is stated rather than quietly substituted:** the
two examples move into `mnemonic_engrave::sysw::advice` as `pub const`s — `main.rs`
is a binary target, so an integration test cannot reach a constant declared
there — and `tests/ms_remedy_runs.rs` RUNS the exact bytes the binary formats
into the message, one hop earlier than stderr. A third test **pins the
unreachability**, so the day it changes the suite says so and the extraction can
move to stderr where the plan wanted it.

**A second measurement, and it is why the first mutation did not fail.** After
P2's ungrouped stdout, the OLD advice (`ms encode --phrase - < seed.txt | …`)
**also runs** — swapping the constant back leaves the test green. So F-301's live
defect is closed by P2's `ms`, not by this text change. Two genuinely broken
mutations do go RED (`--nosuchflag`: 2 of 3 tests fail; dropping `--out p.bin`:
1 of 3), so the gate is a gate.

The skip path was exercised: with `MS_P2_BIN` unset both tests print the reason
and the sentence *"P2 does not close until this has been run with it set"*.
**They were run with it set: 3 passed.**

---

## 6. Row 12's residue

The plan's §1.9 table reproduces **exactly**: 18 lines, 20 invocations, 13
carrying material, per-script counts identical. The 7-item residue is exactly as
enumerated: 2 `[ -x "$MS" ]` tests, 3 `--version` calls, 2 already using
`--phrase -`.

Six of seven drivers run to completion at rc 0 against the P2 release build, and
`git status design/journeys/` afterwards shows **only the `.sh` edits** — not one
generated key, xpub, fingerprint, preimage or policy file changed.

`transcript_tr_pathological.sh` exits 1, and it is **not** this migration:
`restore_test_tr_pathological.py` hard-binds `ms` to the live checkout's release
path and raises `FileNotFoundError`. Verified by stashing the migration and
re-running — rc 1 and two `FATAL`s either way. **F-363** files the one-line fix,
deliberately not made here because row 12's own control requires those two python
drivers be left unedited.

`cargo build --release --locked -p ms-cli` (the row's stated precondition) rc 0.
`md` and `mk` release builds were also absent and were built, or four transcripts
would have died at exit 127 on a dependency unrelated to `ms`.

---

## 7. Defects found and closed while building

**A leak the guard missed, found and closed in row 5.** The first draft
classified the RAW argv token, so a **comma-grouped share** on argv was not
recognised as material while the unbroken spelling of the identical secret was —
`ms` strips display separators on intake, so the two are the same input.
`is_ms1_shaped` now strips them first. It surfaced because `cli_combine` has a
test that passed grouped shares positionally; that test is renamed
`grouped_shares_re_ingest_privately_and_are_refused_on_argv` and now asserts both
halves.

**One error kind that disagreed with itself by channel.** Row 2's first draft
returned `BadInput` so the `--hex - <` redirect could live in the message, which
made one file report `Bip39` through `--phrase -` and `BadInput` through `--in`,
with `--json`'s envelope disagreeing by channel. Caught by
`json_error_envelope_per_kind`. The redirect is now a stderr **note** and the
kind stays a property of the input.

**One tautology, written and removed.** A first draft of row 7's mistyped-verb
control asserted `h.contains(S) || !h.contains(S)` — a gate that cannot fail,
which is the class that still blocks under the 2026-08-27 operator ruling.

**One false GREEN, caught by an isolation control.** Row 4's first offline check
passed for the broken two-block form because `~/.cargo/git` supplied the
dependency. Re-run with an empty `CARGO_HOME`, the broken form exits 101.

**One harness that stopped reproducing its own defect.** Row 7's first zsh rc
file set `INC_APPEND_HISTORY`, which writes each line to the file as entered, so
`sed -i` alone succeeded and the trap the message warns about vanished. Stock
zsh writes at exit, which is the whole reason the recipe needs its extra steps.

---

## 8. The test migration — enumerated (condition 15)

148 tests went red on the guard. The plan forbids greening them with
`--allow-argv-secret`; every one migrated to a private channel.

`crates/ms-cli/tests/support/mod.rs` (new) rewrites an invocation onto
`--in FILE` / `-` / `--passphrase-stdin` per §6d's per-verb table, and **PANICS
naming both channels** when an invocation would need two stdins rather than
silently picking one. Four `cli_derive` tests hit that panic and were hand-routed
through the `ms1` card — the two-command route the refusal itself advises.

**59 test files changed. 11 are new.** Of the 48 modified, **34 are pure channel
migration**: not one assertion, not one test name changed — verified
mechanically by diffing for `assert` and `fn …()` lines. The other 14, each
justified:

| file | change | justification |
| --- | --- | --- |
| `cli_derive.rs` | `inline_secret_argv_advisory` → `inline_secret_on_argv_is_now_refused_not_merely_advised` | plan's explicit instruction: rewritten against the refusal, not deleted. Strictly stronger — its subject was a warning that still exited 0 |
| `cli_derive.rs` | 3 tests re-routed through the `ms1` card | two secret channels, one stdin; §2.5's two-command route |
| `cli_combine.rs` | `combine_accepts_comma_grouped_positional_shares` renamed + strengthened | the leak fix above; asserts re-ingest AND argv refusal |
| `encode_grouping_flags.rs` | grouping assertions MOVED to the card; `encode_separator_hyphen` absorbed | §6a/§6b (row 9), §6c (row 10) |
| `encode_canonical_12_word.rs` | stdout assertion → canonical; card assertion added | §6a/§6b |
| `encode_arg_group_violations.rs` | EXTENDED, not rewritten — 2 original assertions untouched | row 2's third group member |
| `verify_phrase_round_trip_ok.rs` | `--in` + `--phrase -` | two materials in one invocation — the shape row 3 makes possible |
| `decode_mnem_japanese.rs`, `inspect_mnem_string.rs`, `cli_output_class.rs`, `cli_repair.rs`, `json_error_envelope_per_kind.rs`, `split_combine_derive_chain.rs`, `encode_output_unchanged_after_split_refactor.rs` | predicate chains → explicit asserts | mechanical, forced by `support::run` returning `Output`; every assertion preserved |
| `lint_zeroize_discipline.rs` | evidence anchor re-pinned | it names `read_phrase_input`'s signature verbatim, which row 1 changed |
| `gui_schema_emits_spec_v7_json.rs` | row 13's arithmetic added | — |

**The `src/` half is not decoration** (R0 round 0's M-4): `format.rs`'s
`parse_separator_keyword_and_literal` asserted that `"hyphen"` yields `'-'`. It is
now `parse_separator_offers_whitespace_only_and_names_why` plus a new
`intake_still_strips_the_retired_separators` — 6 `#[test]`s → 8.

---

## 9. Consults

**One**, to a `fable` agent, on row 4's cross-repo blocker: *proceed with the
in-repo fixes and file the reusable-workflow gap, or stop and hand the question
back before the crate-dependent rows?* **Decided: proceed (A)**, with the two
mechanical in-repo fixes in row 4's own commit, on the grounds that the blocker
binds only at tag time, no row of P2 cuts a tag, and the cross-repo change is
required under every path the frozen plan allows (`path =` forbidden, publish out
of scope) — so it is scheduling, not a decision fork.

---

## 10. Follow-ups filed

| id | one line | owning phase |
| --- | --- | --- |
| **F-324** | the git pin breaks `ms`'s tag-time reproducible musl build; the fix is an input the toolkit's shared reusable workflow does not have | before the next `ms-cli-v*` tag |
| **F-362** | `me`'s secret-class private-channel advice branch is unreachable behind the pre-parser guard | a later cycle |
| **F-363** | the two `restore_test_*.py` journey drivers hard-bind `ms`'s path, so no branch build can run them | ownerless residue |

`F-321`–`F-323` and `F-331`–`F-333`, `F-360`, `F-361` were taken by other agents
while this ran; the numbers above are the ones that were free at filing time.

---

## 11. What was not done

* **`musl-check`, both legs.** `musl-gcc` and `cross`/Docker are absent on this
  box. The failure is in `cc-rs` building `secp256k1-sys`, before any `ms` code.
  `freebsd-compile-gate` — the same cross-target question for the new
  dependency — did run and passed.
* **`man-release.yml`'s `repro` job.** Known-red, F-324, not fixable from this
  repo.
* **Row 11's stderr extraction.** F-362; the substitute is stated in §5 rather
  than made quietly.
* **The two `restore_test_*.py` drivers.** F-363; editing them would have
  falsified row 12's own control in the same commit.
* **Nothing has been pushed.** `impl/p2` is local; `mnemonic-engrave`'s
  `master` carries the cross-repo commits locally and is unpushed.

---

## 12. Commits

**`mnemonic-secret`, branch `impl/p2`** (11, one per row group):

```
185b9b1 P2 row 1: --in FILE on the six verbs with an unambiguous binding
21d7057 P2 row 2: --in on encode and split, and it means a PHRASE
a978b1e P2 row 3: --in frees stdin, and the two contention refusals become satisfiable
9cd0868 P2 row 4: pin mnemonic-io-lib by git rev — and the three-block vendor form it forces
0b795a1 P2 row 5: the argv guard, on RAW argv, before the parser
2997e1a P2 row 6: --allow-argv-secret is a CHANNEL, and the material is SUBSTITUTED
5688d1b P2 row 7: the purge text, RUN under real shells — not printed
2d76f66 P2 row 8: the private write — --out FILE on encode, split and repair
b9f73ee P2 row 9: ms encode's stdout is the canonical ms1, always ungrouped
13665a8 P2 row 10: the whitespace-only separator — emission narrowed, INTAKE untouched
c659b5b P2 rows 13 and 14: the schema still describes the binary, and the DECLINE asserted
```

**`mnemonic-engrave`, `master`** (5):

```
76176cf followups: F-324 — the git pin breaks ms's tag-time reproducible build
5075a4a followups: F-362 — me's secret-class advice branch is unreachable
9f1253e P2 row 11: me's secret-class remedy becomes the --in form, and gains a test that RUNS it
a65106a followups: F-363 — the python restore drivers hard-bind ms's path
c944642 P2 row 12: the drivers — 13 material invocations migrated to a private channel
```
