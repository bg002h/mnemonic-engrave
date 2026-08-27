# PLAN-P2 — authoring log

**Agent:** P2 plan author. **Date:** 2026-08-27. **Branch:** `plan/p2`, worktree
`/scratch/code/shibboleth/_work/planp2/mnemonic-engrave`, started at `ba1f3ec`.

**Output:** `design/IMPLEMENTATION_PLAN_P2_ms_adopts.md` — 754 lines, **14 rows**,
**18 closure conditions**, **4 crate items adopted / 7 declined**. Committed at
`37fa40b`; the six follow-ups it files landed separately at `08e0516`.

**Gates, each run as its own command, exit code read directly:**

```
scripts/plan-table-check.sh    = 0     53 table rows checked, 0 malformed
scripts/plan-cite-check.sh     = 0     29 / 29 resolved, 0 dangling, 0 ambiguous
scripts/plan-stepref-check.sh  = 0     0 step numbers in prose
```

`plan-cite-check.sh` **does** resolve `mnemonic-secret` — the root was already
present at `scripts/plan-cite-check.sh:78`, added before this cycle, and a probe
proved it rather than the list being read. Two forms of citation are unreachable
and both are recorded in the plan: `design/FOLLOWUPS.md:N` is AMBIGUOUS under
five roots (ambiguity alone exits 1), and a path beginning with a dot loses the
dot, so any workflow-directory anchor is a false DANGLING — filed as F-286 with
a control.

---

## What was measured, and what it contradicted

All measurements ran against `/scratch/code/shibboleth/mnemonic-secret/target/debug/ms`
by absolute path at `7c12f66`, after `cargo build --locked`, exit codes read
directly and never through a pipe. `/home/bcg/.cargo/bin/ms` was **not** used —
it is dated 2026-08-15 against a tree built 2026-08-26.

### Contradictions with the brief's framing

- **`ms` matches neither `mt`'s `0o077` nor `me`'s `0o044`. It has no mode gate
  at all.** `git grep -n` over `crates/` for `fs::write`, `OpenOptions`,
  `set_permissions`, `0o600`, `0o077`, `0o044` and `st_mode` returns **zero
  hits**. `ms` never fstats its stdout. `ms encode > backup.txt` under umask 022
  creates 0644 at exit 0.
- **`ms` has no `std::env::args` site either.** `main` goes from
  `process_hardening::set_non_dumpable()` straight to `Cli::try_parse()`. The
  argv guard is not a port of a weaker version; it is the first one.
- **The binary has eleven subcommands, not eight.** The spec's eight are the
  material-handling ones and the count is right about its own subject —
  `vectors`, `gui-schema` and `gen-man` handle no material. Stated in the plan
  because a reader checking "eight" against `ms --help` finds eleven.

### Contradictions with the spec

- **The argv surface is 14 channels across the eight verbs, not 8.** §6d's table
  gives one channel per verb and understates `derive`, which has four
  (positional, `--hex`, `--phrase`, `--passphrase`). Measured per channel:
  **11 of 12 material-bearing invocations exit 0 with no warning at all**; only
  `derive` warns, at four sites. `ms encode --phrase "<a real seed>"` exits 0 in
  silence — the spec's §1 finding, reproduced.
- **The `-` documentation gap is three channels, not one verb.** §6d names
  `combine`. Measured, `--hex` also carries no stdin sentence in `encode --help`
  or `split --help`, while `--hex -` works on both.
- **§7's "18 argv call sites" is a count of `"$MS"` OCCURRENCES.** The 18
  reproduces exactly, and 5 of them are not invocations of material: 2 are
  `[ -x "$MS" ]` tests, 3 are `--version`, 2 already use `--phrase -`. **The
  number to migrate is 13**, on 11 lines (two lines carry nested invocations).
  A gate reading "the 18 call sites migrated" literally is unsatisfiable.
- **`ms split`'s grouping is NOT §3's defect, measured.** `me sysw pack` refuses
  a codex32 share at exit 4 **grouped and ungrouped alike** — grouping is not
  what blocks it — so the packability argument that decides `encode` has no
  purchase on `split`. This narrowed the plan's scope rather than widening it,
  and is filed as F-284.
- **The conformance-vector pin is NOT endangered by the separator work.** The
  22-row SHA-pinned `display-grouping-vectors.tsv`, whose 5 hyphen and comma
  rows looked like a blocker, drives `render_grouped` directly and never the
  CLI. §6c narrows `parse_separator`, which the vectors never call. Verified:
  `sha256sum -c` exits 0 today and stays untouched.
- **§7's P2 gate IS satisfiable literally, with one precondition the spec does
  not state.** `ms encode --group-size 0` piped into `me sysw pack` **with no
  flags** exits 0, seals and prints a generated passphrase — provided `me`'s own
  stdout is not world-readable. With the same capture at 0644 it exits 2 on
  `me`'s destination gate, which is `me`'s and not `ms`'s.

### Contradictions with the sibling plan

- **`--out` is already taken on this binary and means a DIRECTORY.** P1 measured
  one `--out` in all of `mt`, a refusal string. `ms gen-man --out <DIR>` is
  shipped, exampled twice in `--help`, driven by `crates/ms-cli/tests/gen_man.rs`,
  invoked by the `man-release.yml` workflow at its line 46, **and by
  `scripts/install.sh:305` in `mnemonic-toolkit`**. Filed as F-282, not fixed.
- **`ms`'s tree is fully green before P2 starts.** `mt` entered P1 with two of
  seven CI gates already RED, which cost it a tree-greening row. Measured here:
  fmt 0 diff lines, `clippy -p ms-cli` green, `clippy -p ms-codec` green,
  `cargo nextest run --locked` **414 run / 414 passed / 5 skipped**, and the
  vector checksum pin green. **P2 needs no tree-greening row.**
- **`ms` carries two CROSS-REPO byte-parity gates that `mt` does not** — the
  `g6 invariant` job checks out `mnemonic-toolkit` and diffs `mlock.rs`, and
  `crates/ms-cli/tests/cli_output_class.rs:56` pins `ms`'s advisory wording
  against the toolkit's. Both constrain any change to `advisory.rs`.
- **The test blast radius is an order of magnitude larger.** 31 of 76 test files
  reference `--phrase` or `--hex`, and those 31 hold **147 of the suite's 276
  integration tests** — 53% of the suite lives in files that put seed material
  on argv. The plan forbids greening them by appending `--allow-argv-secret`.

### Things that will bite an implementer, found by running

- **Every journey driver is unrunnable today.** Eight scripts bind
  `MS=$C/mnemonic-secret/target/release/ms`, and **no release build exists**.
  Seven of the eight bind it non-overridably; only
  `design/journeys/derive-pathological-keys.sh:39` uses `${MS:-…}`. The
  precondition is written into the drivers row's gate.
- **`ms` ships two "one stdin per invocation" refusals** — `ms verify - --phrase -`
  and `ms derive - --passphrase-stdin`, both rc 1 — that `--in` makes
  unnecessary. That is the phase's "what adoption FIXES": `--in` is the first
  private way to supply two values at once, not merely a hardening measure.
- **`ms`'s existing argv warning names a threat `ms` partly closed.**
  `set_non_dumpable()` calls `prctl(PR_SET_DUMPABLE, 0)`, so cross-UID
  `/proc/$PID/cmdline` reads are already blocked; the shipped text names that
  vector unqualified. A refusal copied from `me`'s wording would inherit the
  overstatement.
- **`me`'s own remedy carries a comment that P2 falsifies.**
  `crates/me-cli/src/main.rs:2184` states `ms encode --in` DOES NOT EXIST (exit
  64), four lines above the advice it justifies. Both change together, and §6h's
  standing instruction fires exactly here.

---

## Crate items declined, with the measurement behind each

| item | why `ms` declines it |
| --- | --- |
| `exit::write_block` | its `Terminal` arm refuses unconditionally, and §6e's retraction names `ms encode` as the case where that refusal makes the exposure strictly worse |
| `exit::WriteBlock` | `Terminal(PayloadKind)` would be unconstructible, and `WorldReadable(u32)` unreachable because P2 builds no stdout gate |
| `channel::destination` | **a change from `mt`, which adopted it.** `ms`'s `--out` needs `Option::is_some`; the other two arms exist to feed `write_block`, which is declined, so adopting it puts `Terminal` in `ms`'s vocabulary with nothing that may act on it |
| `fd::stdout_mode` | exists to feed a stdout gate; P2 builds none |
| `observation::PayloadKind` | **`ms` already carries a SUPERSET.** `OutputClass` has three variants and `ms` uses two of them, `derive` emitting the watch-only line. `CarriesNoSecret` is documented as a fill image; a watch-only xpub is not nothing. And the three lines are cross-repo byte-parity-locked, so the vocabulary is not `ms`'s to replace |
| `records::split_record_stream` | `read_shares` strips display separators per line; `split_record_stream` does not. Measured: grouped shares recombine at exit 0 today, and would arrive as `ms12g 30dqz …` under the crate's version |
| `records::no_records_guard` | its message advises *"pass them on argv"*, which this phase exists to make unfollowable — the same disqualification P1 found |

Adopted: `remedy::history_purge_block`, `remedy::history_purge_recipes`,
`fd::mode_of`, and `write_private` once P1 moves it in. **Two consecutive
consumers have now declined `exit`, `observation` and `records` entirely**, for
different reasons each time. F-276 records the crate as `me`-shaped in two
places on `mt`'s evidence; §2.3 of the plan adds a third.

---

## Consults, and what they decided

Two ambiguities could not be settled by reading or running. Each went to one
`fable` agent as a single question.

1. **What `--in FILE` means on `encode` and `split`, which have two material
   channels.** **Decided: it means a PHRASE**, joining the existing required
   `ArgGroup` as a third alternative, with hex-from-a-file keeping `--hex -`.
   Content-sniffing was rejected on a specific hazard: today's sniff would be
   safe only because a phrase always contains whitespace, and that restraint is
   invisible, so a later maintainer being liberal with whitespace turns a
   hex-alphabet BIP-39 phrase into valid entropy for a **different wallet** — a
   valid, wrong plate. The plan's gate is the consult's counterexample test: a
   64-hex-character file must make `ms encode --in f` refuse and name `--hex -`,
   while `ms encode --hex - < f` succeeds.
2. **Whether P2 owns a world-readable stdout gate for `ms`.** **Decided: build
   nothing, and file it for an operator ruling.** A refusal is foreclosed by
   §6e's retraction, by F-275's directly analogous ruling on `mt decode`, and by
   the fact that it would reject the ordinary invocation on every default
   machine; a warning is unspecified and adds little over the unconditional
   private-material line `ms` already prints. Filed as F-281 with F-275
   attached, so the eventual ruling is pre-shaped: if anything is built, a
   warning, never a refusal.

Both decisions were adopted, and both had their load-bearing facts re-derived
here rather than transcribed — the `ArgGroup` declarations, the `--help` text,
and `ms`'s existing advisory line were all re-measured before the plan quoted
them.

---

## Follow-ups filed

`F-281` world-readable stdout, in scope with no owning phase (operator ruling) ·
`F-282` `gen-man --out DIR` versus `--out FILE` (later cycle) ·
`F-283` `mnemonic-gui` schema mirror goes stale in P2 (P3) ·
`F-284` `encode` and `split` disagree about their stdout (P3) ·
`F-285` `decode` and `combine` write a recovered seed unprotected (operator
ruling, with F-281) ·
`F-286` `plan-cite-check.sh` strips a leading dot (ownerless residue).

---

## One near-miss worth recording

The plan almost shipped a false path. `ms gen-man --help` says
*"`scripts/install.sh` invokes this post-`cargo install`"* — and
`mnemonic-secret` **has no `scripts/` directory at all**. The claim had already
been written into both the plan and a follow-up from the help text before it was
checked. The real installer is `scripts/install.sh:305` in **`mnemonic-toolkit`**,
which strengthens F-282 rather than weakening it: renaming the flag breaks a
script in a different repository. Both documents were corrected, and the wrong
doc line is noted in F-282 for whoever picks it up.

**This is the "never describe code from its doc comment" rule catching a live
error inside the document that restates it.** It was found only because the
citation was going to be anchored and had to resolve.
