# IMPLEMENTATION PLAN — P2: `ms` adopts `mnemonic-io-lib`

**Status:** DRAFT v1, written 2026-08-27. **NOT reviewed.** No code may be
written until this closes an R0 round at 0C/0I.

**Gates this plan is checked by** — run each **separately** from the commit,
never on the same shell line: `scripts/plan-stepref-check.sh` (prose may not name
a step number), `scripts/plan-table-check.sh`, `scripts/plan-cite-check.sh`.

**CITATIONS INTO `ms` RESOLVE, AND WERE PROVEN TO** —
`/scratch/code/shibboleth/mnemonic-secret` is already in `plan-cite-check.sh`'s
`ROOTS` list (`scripts/plan-cite-check.sh:78`), added before this cycle. A probe
carrying `crates/ms-cli/src/cmd/encode.rs:27` and
`crates/ms-cli/src/main.rs:169` resolved both at exit 0. **One form must be
avoided and this plan avoids it:** a bare `design/FOLLOWUPS.md:N` citation is
reported **AMBIGUOUS — exists under 5 roots**, and ambiguity alone exits 1.
Follow-ups here are therefore named by ID in prose and never anchored by line.
**A second form is unreachable and it is the gate's limitation, not a bad
citation:** a path beginning with a dot loses the dot, so any anchor into a
repository's workflow directory is looked up without it and reported DANGLING
under every root. The form is described rather than written out, because
writing it out makes this document fail the gate it is describing — reproduced
while drafting. The one workflow this plan cites is therefore named in prose,
and F-286 records the gap.

**CITATIONS INTO `mnemonic-io-lib` ARE BY SYMBOL, WITH NO LINE NUMBER, ON
PURPOSE.** The crate is being edited right now in another worktree by two pieces
of P1: the fish purge recipe is being built into `remedy.rs` (today that module
*describes* fish rather than prescribing it), and `write_private` is moving into
the crate from `me-cli`. Any line number taken from the crate today is wrong by
the time this plan is executed. F-279, filed 2026-08-27, measured this exact
decay on the sibling plan: **14 of 15 of its `mt` line citations had gone stale
under its own early work, and `plan-cite-check.sh` reported all 15 green**,
because it checks that a line exists and not what is on it. Every `ms` citation
below therefore names the SYMBOL beside the number, and the crate gets the
symbol alone.

**Source spec:** `design/SPEC_constellation_cli_uniformity.md:1332` (P2's row and
gate), `:660` (§6b — the channels, and the `--out` overwrite ruling at `:681`),
`:732` (§6c — the separator rule and the card), `:792` (§6d — the argv refusal,
the two-layer detector at `:828`, the pre-parser ordering at `:841`, and the
per-verb `ms` channel table at `:876`), `:923` (§6e — the write gate, and the
terminal-gate retraction that names `ms encode` at `:963`), `:992` (§6f — exit
codes), `:1257` (§6h — remedy text must be executable, and the standing
instruction at `:1292` that `me`'s `--phrase -` advice becomes the `--in` form
when P2 ships one, *"and not before"*).

**Prior art this plan is downstream of:**
`design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` — the crate.
`design/IMPLEMENTATION_PLAN_P1_mt_adopts.md` — the sibling adoption, whose shape
this document copies and whose conclusions it re-measured rather than inherited.

---

## 0. Why this plan exists at all

P1 asked whether a boundary drawn around one consumer survives a second. P2 asks
a harder question, because `ms` differs from both `me` and `mt` in the one
dimension the crate is about.

**`me` and `mt` each already had a write gate, an argv guard and a purge
paragraph; the crate was extracted from what they had.** `ms` has **none of the
three**. Measured, with the command beside it —
`git grep -n 'env::args'`, and `git grep -n 'fs::write\|OpenOptions\|set_permissions\|0o600\|0o077\|0o044\|st_mode'`,
both scoped to `crates/`, both returning **zero hits**. `ms` never reads its own
argv, never fstats its stdout, and never creates a file with a mode. So P2 is
not a port of a mechanism onto a tool that had a worse version of it. It is the
first installation.

And `ms` holds the material the whole cycle is about. `ms decode <ms1>` prints
the BIP-39 mnemonic; a K-subset of `ms split`'s shares recombines to the secret;
`ms encode --phrase` takes the seed itself. **Every ruling about argv, stdout
and process-table exposure binds harder here than anywhere else in the
constellation, and today `ms` enforces none of them.**

The most valuable output of this phase is again the DECLINE list — see §2, where
`ms` takes **4 of the 11** public crate items and declines 7, one of them for a
reason `mt` did not have: `ms` already carries a *richer* vocabulary than the
crate's, cross-repo byte-parity-locked to `mnemonic-toolkit`.

---

## 1. THE INVENTORY — measured, not described

Everything below was run on 2026-08-27 against
`/scratch/code/shibboleth/mnemonic-secret/target/debug/ms` **by absolute path**,
at `7c12f66` (`HEAD`, clean tree), after `cargo build --locked`. Exit codes were
read directly, never through a pipe.

**The installed binary was NOT used and that matters.**
`/home/bcg/.cargo/bin/ms` is dated 2026-08-15; the repo's own build is dated
2026-08-26. A behavioural measurement taken from the installed copy would be
eleven days of commits stale. Every number here comes from the tree's build.

### 1.1 `ms`'s surface — eleven subcommands, of which eight carry material

`ms --help` lists **eleven** subcommands plus `help`: `derive`, `encode`,
`decode`, `inspect`, `verify`, `vectors`, `gui-schema`, `gen-man`, `repair`,
`split`, `combine`. §6d's table names **eight**, and the eight are exactly the
material-handling ones — `vectors`, `gui-schema` and `gen-man` take no artifact
and no secret. **The spec's count is right about its own subject and is not a
count of the binary's verbs**, which is worth saying because a reader checking
"eight" against `ms --help` finds eleven and has no way to tell which reading is
wrong.

`ms` maps every clap error to **64**, deliberately, so that 2 stays reserved for
`ms1` format violations — its own comment says so at
`crates/ms-cli/src/main.rs:176` (inside the `Cli::try_parse()` arm). Measured
codes: clap usage **64**, invalid artifact **1**, repair-uncorrectable **2**.
Those are §6f's `ms` row exactly, and §6f changes only `mk`'s
invalid-artifact 2, which is P3's. **There is no exit-code work in P2.**

### 1.2 `--in` — absent everywhere, and there is one seam for all eight verbs

`--in` exists on **no** `ms` verb; every one of the eight exits 64 when handed
it. What exists instead is three intake helpers in one module, and every verb
reaches material through one of them:

| helper | what it returns | callers |
| --- | --- | --- |
| `read_input` (`crates/ms-cli/src/parse.rs:21`) | whitespace-stripped `String` | `encode --hex`, `decode`, `inspect`, `verify`, `repair`, `derive --hex`, `derive [MS1]` |
| `read_phrase_input` (`crates/ms-cli/src/parse.rs:36`) | `Zeroizing<String>`, runs collapsed | `encode --phrase`, `verify --phrase`, `derive --phrase` |
| `read_stdin_passphrase` (`crates/ms-cli/src/parse.rs:54`) | `Zeroizing<String>`, bytes preserved | `derive --passphrase-stdin` |

`combine` is the exception and reads stdin itself, through `read_shares`
(`crates/ms-cli/src/cmd/combine.rs:54`).

The first two take `Option<&str>` and treat `None` or `Some("-")` as stdin —
the predicate is `is_stdin_arg` (`crates/ms-cli/src/parse.rs:95`). **That
`Option<&str>` is the seam: `--in` is one additional source threaded through
two functions, not eight independent flags**, which is P2's counterpart to P1's
observation that `mt`'s three reading verbs share one clap struct.

### 1.3 `-` — implemented on all eight, and undocumented on THREE channels, not one

Every channel was fed from a file on stdin and its exit code read directly. All
eleven invocations below exit **0** and produce the same artifact as the argv
form:

`encode --phrase -`, `encode --hex -`, `decode -`, `decode` (omitted),
`verify -`, `inspect -`, `repair --ms1 -`, `split --phrase -`, `split --hex -`,
`combine -`, `derive -`, `derive` (omitted).

§6d says `-` is *"documented on 7 of 8 verbs and implemented on all 8"*, with
`combine` the documentation gap. **The implementation half reproduces exactly.
The documentation half is measured wider than the spec states:** `ms encode
--help` and `ms split --help` describe `-` on `--phrase` and say nothing about
it on `--hex`, while `ms combine --help` mentions stdin zero times. So the
`--help` gap is **three channels across three verbs**, not one verb. Counted
with `grep -ci stdin` over each captured `--help`: `combine` 0, and `--hex`'s
own entry in `encode`/`split` carries no stdin sentence.

### 1.4 argv — no guard at all, and a WARNING on one verb of eight

`ms` never reads `std::env::args()`. `main` (`crates/ms-cli/src/main.rs:170`)
goes straight to `Cli::try_parse()`, one line after
`process_hardening::set_non_dumpable()` (`:169`). **There is nothing before the
parser.**

What `ms` does have is a stderr advisory, `secret_in_argv_warning`
(`crates/ms-cli/src/advisory.rs:38`), called from **`derive` alone**, at four
sites: `crates/ms-cli/src/cmd/derive.rs:322` (the `ms1` positional), `:327`
(`--hex`), `:332` (`--phrase`) and `:336` (`--passphrase`).

Measured, one invocation per channel, each with real material on argv and the
count of `secret material on argv` lines in stderr:

| invocation | exit | argv warnings |
| --- | --- | --- |
| `ms encode --phrase <a real 12-word seed>` | 0 | **0** |
| `ms encode --hex <32 hex chars>` | 0 | **0** |
| `ms decode <ms1>` | 0 | **0** |
| `ms verify <ms1>` | 0 | **0** |
| `ms verify <ms1> --phrase <seed>` | 0 | **0** |
| `ms inspect <ms1>` | 0 | **0** |
| `ms repair --ms1 <ms1>` | 0 | **0** |
| `ms split --phrase <seed> -k 2 -n 3` | 0 | **0** |
| `ms derive <ms1>` | 0 | 1 |
| `ms derive --hex <32 hex chars>` | 0 | 1 |
| `ms derive --phrase <seed>` | 0 | 1 |
| `ms derive <ms1> --passphrase <text>` | 0 | 2 |

**`ms encode --phrase "<a real seed>"` exits 0 in silence.** That is the
finding §1 of the spec is about, reproduced, and it is P2's whole reason for
existing.

**The argv surface is 14 channels across the eight verbs, not 8.** §6d's table
gives one channel per verb and understates `derive`, which has four. Enumerated:
`encode` 2 (`--phrase`, `--hex`); `decode` 1 (positional); `verify` 2
(positional, `--phrase`); `inspect` 1; `repair` 1 (`--ms1`); `split` 2; `combine`
1 (a variadic `<SHARES>...`); `derive` 4 (positional, `--hex`, `--phrase`,
`--passphrase`). Four of the fourteen warn; ten do not.

**And `ms` already mitigates half of what the refusal will name.**
`set_non_dumpable` (`crates/ms-cli/src/process_hardening.rs:26`) calls
`prctl(PR_SET_DUMPABLE, 0)`, which makes `/proc/$PID/` unreadable to other
non-root UIDs and disables core dumps. `advisory.rs`'s own module doc concedes
the residual in as many words: same-UID exposure remains. The shipped warning
text nonetheless says *"to avoid /proc/$PID/cmdline exposure"* without
qualification. **A refusal written from `me`'s wording would inherit that
overstatement into a tool that partly closed the hole** — and the live reason to
refuse on `ms` is shell history and the same-UID process table, not a
cross-UID `/proc` read. The wording work below owns this.

### 1.5 The write gate — `ms` has none, and the answer is neither `0o044` nor `0o077`

`mt` refuses a stdout whose mode has any bit in `0o077`; `me` refuses `0o044`.
The question *"which does `ms` use"* has no answer: the grep in §0 returns zero
hits for every mode constant, every `OpenOptions`, and `set_permissions`.
`ms encode > backup.txt` under the default umask 022 creates **0644** and exits
0. **P2 introduces the first mode-aware code `ms` has ever had, and it
introduces it only on `--out`** — see §6 for why no stdout gate is built.

### 1.6 `--out` — THE NAME IS ALREADY TAKEN ON THIS BINARY, AND IT MEANS A DIRECTORY

This is the largest single difference from P1 and the sibling plan does not
predict it. In `mt`, `git grep -- '--out'` found **one** occurrence in the whole
repository and it was a refusal string. In `ms`:

```
ms gen-man --out <DIR>
      --out <DIR>   Directory to write the `*.1` man pages into (created if absent)
```

`ms gen-man --out` is **shipped, documented, exampled twice in `--help`
(`crates/ms-cli/src/main.rs:125`), and invoked by CI** —
`man-release.yml` in `mnemonic-secret`'s workflow directory runs
`./target/release/ms gen-man --out man` at its line 46, to build the
`ms-man.tar.gz` release asset, and `crates/ms-cli/tests/gen_man.rs` drives the
same flag.

So after P2, `--out` means **a directory of man pages** on one verb and **a file
holding the artifact** on three others. The collision is real, it is a genuine
uniformity defect, and **P2 does not fix it** — renaming `gen-man --out` breaks
a release workflow *and a script in a different repository* for a cosmetic gain,
in a phase whose row is funds-safety work. **The second consumer was located
rather than believed**: `ms gen-man --help` says *"`scripts/install.sh` invokes
this"*, and `mnemonic-secret` has no `scripts/` directory at all. The installer
is `scripts/install.sh:305` in **`mnemonic-toolkit`**, which drives
`<bin> gen-man --out` across every sibling that carries the verb. Declined with
the measurement, filed as F-282.

### 1.7 Grouping and separators — the defect, reproduced, and the pin that survives

**§3's decisive measurement, reproduced with both exit codes read directly.**
`ms encode --phrase <the all-abandon vector>` emits, by default,

```
ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f
```

Fed to `me sysw pack --no-passphrase --out <file>` on stdin, that exits **4** —
*"record 0 … is not a form this container can place"* — and writes no payload.
The same command with `--group-size 0` exits **0** and writes a **102-byte**
payload at mode 0600. Under `umask 077`, `ms encode --group-size 0` piped into
`me sysw pack` **with no flags at all** exits 0, seals, and prints a generated
passphrase. So §7's P2 gate is satisfiable literally; the only precondition is
that `me`'s own stdout not be world-readable, which is `me`'s gate and not
`ms`'s.

**All three separators round-trip through `ms`'s own decoder** — `space`,
`hyphen` and `comma` each decode at exit 0, reproducing §6c's `ms` row.

**Two facts about the conformance pin, because the obvious fear is wrong.**
`design/display-grouping-vectors.tsv` in `mnemonic-secret` is SHA-pinned and
checked in CI (`sha256sum -c` inside the `clippy` job). It carries 22 rows, of
which 2 render with `hyphen` and 3 with `comma`. **They do not break.** The test
that consumes them (`crates/ms-cli/src/format.rs:265` names the path) calls
`render_grouped` (`crates/ms-cli/src/format.rs:18`) directly — a pure
`(&str, usize, char)` function — and never goes through the CLI. §6c removes
`hyphen` and `comma` from `parse_separator`
(`crates/ms-cli/src/format.rs:41`), which the vectors never call. Verified
today: `sha256sum -c` exits 0. **The pin is only endangered if an implementer
also narrows `render_grouped` or `is_display_separator`, and the decline below
says not to.**

### 1.8 The validation surface — green today, all of it

`mt` entered P1 with two of its seven CI gates already RED. **`ms` does not**,
measured 2026-08-27 with each exit code read directly:

| gate | today |
| --- | --- |
| `cargo +1.95.0 fmt --all -- --check` | green, 0 diff lines |
| `cargo clippy --locked -p ms-cli --all-targets -- -D warnings` | green |
| `cargo clippy --locked -p ms-codec --all-targets -- -D warnings` | green |
| `cargo nextest run --locked` | green — **414 tests run, 414 passed, 5 skipped** |
| `sha256sum -c display-grouping-vectors.tsv.sha256` | green |

**So P2 needs no tree-greening work, and that is a measured difference from the
sibling plan rather than an assumption carried over from it.**

`ms`'s CI is wider than those five and the extra jobs bear on P2. `rust.yml`
also runs `miri` over the `mlock` module, a `test (release, mlock einval)` job,
three fault-injection steps (G2.1 eperm, G2.3 einval, G2.4 off), a
`freebsd-compile-gate`, a two-target `musl-check`, and **`g6 invariant
(cross-repo mlock.rs)`, which checks out `mnemonic-toolkit` and compares
`mlock.rs` byte-for-byte**. Separately, `crates/ms-cli/tests/cli_output_class.rs:56`
(`fn byte_parity_advisory_lines`) pins `ms`'s advisory wording against
`mnemonic-toolkit`'s. **Two cross-repo byte-parity gates constrain this phase**,
and neither exists in `mt`.

**The test diff is the largest number in this plan.** `git ls-files
'crates/*/tests/*.rs'` gives **76** files holding **276** `#[test]`
functions. Of those, **31 files reference `--phrase` or `--hex`**, and those 31
files hold **147 test functions** — **53% of the suite lives in files that put
seed material on argv.** A further 13 files holding 63 test functions reference
grouping or separator flags. (`cargo nextest` reports 414 rather than 276 because
it also runs the 146 `#[test]`s inside `src/`, of which 5 are skipped.)

### 1.9 This repo's journey drivers — 18 occurrences, but only 13 need migrating

§7 says *"18 argv call sites across 7 scripts"*. **The 18 reproduces exactly and
is a count of `"$MS"` OCCURRENCES, not of invocations that expose material.**
Measured with `grep -n '"\$MS"'` over the seven named scripts:

| script | `"$MS"` occurrences | of which carry material on argv |
| --- | --- | --- |
| `design/journeys/transcript.sh` | 2 | 1 |
| `design/journeys/transcript_hashvault.sh` | 4 | 3 |
| `design/journeys/transcript_pathological.sh` | 3 | 2 |
| `design/journeys/transcript_tr_pathological.sh` | 2 | 2 |
| `design/journeys/derive-rcw-keys.sh` | 3 | 2 |
| `design/journeys/derive-pathological-keys.sh` | 2 | 0 |
| `design/journeys/derive-hashvault-keys.sh` | 2 | 3 |
| **total** | **18** | **13** |

The residue is 2 `[ -x "$MS" ]` existence tests, 3 `--version` calls, and 2
invocations already using `--phrase -`. Two lines each carry **two** nested
invocations, which is why one script's material count exceeds its occurrence
count. **A gate reading *"the 18 argv call sites migrated"* literally is
unsatisfiable — 5 of the 18 are not invocations of material at all.** The
number to satisfy is 13.

Two further scripts reach `ms` and the spec's list of seven does not name them:
`design/journeys/restore_test_pathological.py:56` and
`design/journeys/restore_test_tr_pathological.py:54` both invoke
`ms derive … --phrase -`, already private, needing no change.
`design/journeys/transcript_rcw.sh:13` binds `MS` but uses it only for
`--version` and an existence check.

**AND EVERY ONE OF THEM IS UNRUNNABLE TODAY.** Eight drivers bind
`MS=$C/mnemonic-secret/target/release/ms`, and that file **does not exist** —
no release build is present. Seven of the eight bind it non-overridably; only
`design/journeys/derive-pathological-keys.sh:39` uses `${MS:-…}`. So a P2
implementer cannot point a driver at a branch build without editing it, and the
migration cannot be verified at all without `cargo build --release` in
`mnemonic-secret` first. That precondition is stated in the driver work's gate
rather than discovered during it.

---

## 2. THE BOUNDARY — 4 of 11 items, and 7 declines

### 2.1 The verdicts

| crate item | verdict for `ms` |
| --- | --- |
| `remedy::history_purge_block` | **ADOPT** — `ms` has no purge text at all; §6d makes it a gate item |
| `remedy::history_purge_recipes` | **ADOPT**, as the block's structured half, so a test can run the emitted recipe |
| `fd::mode_of` | **ADOPT**, for `--out`'s post-write assertion only |
| `write_private` (moving into the crate under P1) | **ADOPT** — it is the whole of the 0600 `--out` |
| `channel::destination` | **DECLINE** |
| `fd::stdout_mode` | **DECLINE** |
| `exit::write_block` | **DECLINE** |
| `exit::WriteBlock` | **DECLINE** |
| `observation::PayloadKind` | **DECLINE** |
| `records::split_record_stream` | **DECLINE** |
| `records::no_records_guard` | **DECLINE** |

**The second and third consumers decline the same three modules for different
reasons, and that is the finding.** `mt` took 5 of 11; `ms` takes 4. Neither
took `exit`, `observation` or `records`. F-276 already records the crate as
`me`-shaped in two places on `mt`'s evidence; §2.3 below adds a third place and
a stronger case.

### 2.2 Why `exit::write_block`, `exit::WriteBlock` and `channel::destination` are declined

`write_block`'s `Destination::Terminal` arm returns a refusal unconditionally.
**The spec forbids `ms` from having that refusal, by name.** §6e's retraction
(`design/SPEC_constellation_cli_uniformity.md:963`) reasons the case out on
`ms encode` specifically: refusing a terminal directs the operator to
`--out FILE`, which they must then read in order to hand-engrave, so *"a
screen-only exposure becomes a screen exposure plus a disk artifact"*.
Adopting `write_block` unchanged gives `ms` exactly the refusal that argument
retracts. Calling it with `stdout_is_tty` hard-coded `false` is the alternative
P1 already rejected as a lie to a function about an observable fact.

`WriteBlock` goes with it: its `Terminal(PayloadKind)` variant would be
unconstructible in `ms`, and its `WorldReadable(u32)` variant is unreachable too
because P2 builds no stdout mode gate (§6).

`channel::destination` is declined for a plainer reason and this is a change
from `mt`, which adopted it. `destination` answers *"file, stream, or
terminal"*. `ms`'s `--out` needs only *"is `--out` given"*, which is
`Option::is_some`, and the other two arms exist to feed `write_block`, which is
declined. **Adopting a three-way classifier to consume one arm of it puts
`Terminal` into `ms`'s vocabulary with nothing that may act on it** — the same
dead-variant shape §2.3 declines `PayloadKind` for.

`fd::stdout_mode` is declined for the same reason at one remove: it exists to
feed a stdout gate, and P2 builds none. `fd::mode_of` is adopted, because the
private-write work's gate asserts a mode on a real file and `mode_of`'s contract
— raw `mode & 0o777`, `None` for a character device, `None` on a failed fstat —
is exactly what that assertion needs.

### 2.3 Why `observation::PayloadKind` is declined — `ms` already carries a SUPERSET

`mt` declined `PayloadKind` because every byte it writes is bearer, so
`CarriesNoSecret` is unconstructible. `ms`'s reason is the opposite and
stronger: **`ms` already has a richer axis, and it is byte-parity-locked to
another repository.**

`OutputClass` (`crates/ms-cli/src/advisory.rs:53`) has **three** variants —
`PrivateKeyMaterial`, `WatchOnly`, `Template` — and `ms` uses two of them today:
`encode`, `split`, `decode`, `combine` and `repair` emit the private-material
line, `derive` emits the watch-only one. `PayloadKind` has two variants and its
`CarriesNoSecret` is documented as *"measured to hold nothing: a 65,536-byte
fill image"*. **A watch-only account xpub is not nothing.** There is no mapping
from `ms`'s three classes onto the crate's two that preserves what `ms` already
says.

And the vocabulary is not `ms`'s to change unilaterally.
`crates/ms-cli/tests/cli_output_class.rs:56` (`fn byte_parity_advisory_lines`)
pins all three lines against `mnemonic-toolkit`'s `secret_advisory`. Adopting
`PayloadKind` would give `ms` two overlapping vocabularies for one question,
with a cross-repo gate on the one it already had.

### 2.4 Why the `records` module is declined

`ms` reads **one** artifact per invocation on seven of its eight verbs, so
`split_record_stream` has nothing to split. The eighth is `combine`, and there
it is a near-miss that fails on a measured behaviour.

`read_shares` (`crates/ms-cli/src/cmd/combine.rs:54`) splits stdin into lines,
**strips display separators from each line**, and drops empties.
`split_record_stream` splits into lines and drops empties, and strips nothing.
Measured: `ms split --phrase <vector> -k 2 -n 3` emits shares grouped in fives
by default, and feeding two of them to `ms combine -` recovers the secret at
exit 0 — because `read_shares` removes the intra-line spaces. Under
`split_record_stream` each of those lines would arrive as `ms12g 30dqz …`,
which is not a share. **Swapping it in loses the grouped-card re-ingest, which
is the whole point of a share an operator typed back off metal.**

`no_records_guard` is refused on the same evidence and one more: its message
advises *"pass them on argv, with --in, or on stdin"*, and after P2 **`ms`
refuses argv**. It would print advice that this very phase exists to make
unfollowable — the identical disqualification P1 found.

### 2.5 What adoption FIXES — a capability `ms` cannot express today

`ms` ships **two refusals that exist only because there is one stdin**, both
reproduced today with the exit code read directly:

```
ms verify - --phrase -            -> rc 1, "cannot read both ms1 and --phrase from stdin"
ms derive - --passphrase-stdin    -> rc 1, "cannot read both the entropy source and --passphrase from stdin (one stdin per invocation)"
```

Both are correct today and both become unnecessary the moment a second private
channel exists. `ms verify --in card.txt --phrase -` and
`ms derive --in card.txt --passphrase-stdin` are round-trip checks an operator
plainly wants and **cannot perform privately today at all**: the only way to
supply the second value is on argv. **`--in` is not only a hardening measure
here; it is the first private way to do two things at once**, and that is the
argument for doing it before the refusal rather than after.

---

## 3. THE DEPENDENCY — `ms` cannot reach the crate yet, and the blocker is P1's

**`mnemonic-io-lib` is not on `origin/master`.** `git ls-tree -d origin/master
crates/` lists `crates/me-cli` alone. It is not on crates.io. So today `ms` has
neither a version dep nor a git-rev dep that resolves, and **no work below that
calls into the crate may begin.**

The unblocking work is P1's, not P2's, and it is three things in order: the fish
purge recipe landing in `remedy.rs`, `write_private` moving into the crate from
`me-cli` (where it still sits — `fn write_private` in `crates/me-cli/src/main.rs`,
cited by symbol because that move changes its line), and then the push of
`mnemonic-engrave` `master` through the `ci/staging` ref so the SHA earns its
required check. P1's plan pins the rev at that SHA for `mt`. **P2 pins the same
SHA for `ms`**, in `crates/ms-cli/Cargo.toml`, as
`mnemonic-io-lib = { git = "https://github.com/bg002h/mnemonic-engrave", rev = "<that SHA>" }`.

**This plan does not design a workaround and must not be given one.** A `path =`
dep does not resolve in a fresh CI checkout, and `ms`'s CI is wider than most —
`freebsd-compile-gate` and `musl-check` both build from a clean checkout on
foreign targets, so a path dep out of the workspace fails there first. The
argument is the one `me-cli`'s own Cargo.toml already records for `mt-codec`.

**The consequence for ordering is the load-bearing part.** The `--in` work
depends on nothing outside `ms`, so it proceeds while P1 is still running. The
argv refusal does not: §6d requires the refusal to carry the purge commands per
§6h, and §7's P0 row rules that the text comes **from `me` alone**. So the
refusal cannot be written before the pin without writing a purge paragraph that
would then be deleted. The table below is sequenced so that everything
crate-free happens first and the pin gates exactly the work that needs it.

---

## 4. TDD ORDER

Each entry is RED first unless its gate column says otherwise. No entry begins
until the previous is green. **This table is the only ordering of record**;
prose refers to work by NAME — *the six channels*, *the two-channel ruling*,
*the freed stdin*, *the pin*, *the guard*, *the override*, *the purge text*,
*the sibling remedy*, *the private write*, *the ungrouped stdout*, *the
whitespace separator*, *the drivers*, *the schema*, *the decline* — so a
renumbering cannot falsify it.

**THE ORDER HONOURS §7's RULING AND IS NOT FREE TO BE REARRANGED.** P2's row
states its contents as *"FIRST `--in` on all eight verbs, THEN the argv refusal,
THEN the 0600 `--out`, THEN `--group-size 0` … and the whitespace-only
separator"*, and marks the phase *"highest safety value; do it before the
cosmetic work"*. §6d separately retracts the *safety* justification for that
sequence — it had rested on a false claim that `ms combine` had no private
channel — and calls the sequencing *"a preference"*. **The ruling stands
regardless**, because the ordering is also what keeps the crate-free work off
the critical path of P1's push.

| # | work | the gate that must fail first |
| --- | --- | --- |
| 1 | **`--in FILE` ON THE SIX SINGLE-CHANNEL VERBS.** `decode`, `verify`, `inspect`, `repair`, `derive`, `combine`. One new source threaded through `read_input` (`crates/ms-cli/src/parse.rs:21`) and `read_phrase_input` (`crates/ms-cli/src/parse.rs:36`), and through `read_shares` (`crates/ms-cli/src/cmd/combine.rs:54`) for the variadic case, where `--in` reads one share per line with display separators stripped exactly as the stdin path already does | each of the six: stdout **and** stderr byte-equal to the stdin run, at exit 0 — an equality assertion, never a bare success, which a `--in` silently ignored would also satisfy. **Fails today: each exits 64**, measured, `ms` mapping every clap error to 64 at `crates/ms-cli/src/main.rs:176`. **Plus two controls.** `--in <a nonexistent path>` must fail naming the path and must NOT fall back to stdin, or a typo silently reads a terminal. And `--in f` together with the verb's `-` must REFUSE, matching the two contention refusals `ms` already ships — a channel that silently wins over another is how an operator engraves the wrong card |
| 2 | **`--in FILE` ON `encode` AND `split`, MEANING A PHRASE.** Both declare a required clap `ArgGroup` — `"input"` at `crates/ms-cli/src/cmd/encode.rs:27` and `"split_input"` at `crates/ms-cli/src/cmd/split.rs:28`, each currently `.args(["phrase", "hex"])`. `--in` joins the group as a third alternative and resolves to the **phrase** channel through `resolve_secret_payload` (`crates/ms-cli/src/cmd/encode.rs:77`). Hex from a file keeps using `--hex -` with a redirect. **Ruled by consult, 2026-08-27**, over a content-sniffing design; see §7 | **the counterexample test, and it is the whole ruling**: a file holding exactly 64 legal hex characters — a valid entropy length — must make `ms encode --in f` exit non-zero, emit **no** `ms1` on stdout, and name `--hex - <` in its stderr; while `ms encode --hex - < f` on the same file exits 0 and emits the artifact. That single test goes RED if the design drifts to sniffing, RED if a later reader "fixes" `--in` to accept hex, and simultaneously proves the channel it redirects to works. **Fails today at exit 64 on both halves of the first assertion.** **Plus the group's own tests**, `crates/ms-cli/tests/encode_arg_group_violations.rs`, which pin exit 64 on both-supplied and neither-supplied and must be extended to three members rather than rewritten |
| 3 | **`--in` FREES STDIN, AND THE TWO CONTENTION REFUSALS BECOME SATISFIABLE.** No new code beyond what the two preceding entries build; this is the assertion that they composed. §2.5 | `ms verify --in card.txt --phrase -` exits **0** and reports the round trip; `ms derive --in card.txt --passphrase-stdin` exits **0** and derives with the passphrase applied. **Both fail today at rc 1**, with `cannot read both ms1 and --phrase from stdin` and `cannot read both the entropy source and --passphrase from stdin (one stdin per invocation)`, measured. **Plus the control that keeps the refusals alive**: `ms verify - --phrase -` must STILL exit 1, or the fix removed the guard instead of routing around it |
| 4 | **PIN THE CRATE.** Add `mnemonic-io-lib` to `crates/ms-cli/Cargo.toml` by GitHub rev, at the SHA P1's push produces. No `path =`, no publish. **BLOCKED on P1** — the fish recipe and the private-write move must land before the push, or the pin needs doing twice. **Regression-gated, not RED-first** | `git ls-tree -d origin/master crates/` names `crates/mnemonic-io-lib` — it does **not** today, measured; `cargo build --locked` in `mnemonic-secret` resolves the rev from a **fresh** clone rather than a cargo cache; and `cargo nextest run --locked` still reports 414 of 414. **The freeze binds**: no commits to `mnemonic-engrave` `master` between the staging push and the final push, and the final push must print no `Bypassed rule violations` |
| 5 | **THE ARGV GUARD, ON RAW ARGV, BEFORE THE PARSER.** A pre-parser layer between `process_hardening::set_non_dumpable()` (`crates/ms-cli/src/main.rs:169`) and `Cli::try_parse()` (`:170`), reading `std::env::args()` — the first `env::args` site `ms` has ever had. Two layers per §6d: **flag-keyed** (a static list of the 14 secret-bearing channels of §1.4, matched as strings, no parse), and **value-shape** (`ms1` by HRP, a BIP-39 phrase by wordlist, hex by charset and length) for material arriving positionally. The refusal names the class and the LENGTH, never the value, names `--in FILE` and `-`, and carries the purge block the purge work adds | **a generated cross-product, not a hand list**: 14 channels × 4 spellings (canonical, leading-space, trailing-space, UPPERCASE) = **56 rows**, each asserting exit non-zero AND that the material's own characters never appear in stderr. **All 56 pass material today at exit 0** — 52 of them in total silence, 4 with a warning only — measured per §1.4's table. **Plus the clap-echo control, which is the one the naive implementation fails**: material that the shape layer does not classify must not reach clap and be printed back. **Plus the near-miss control**: `ms verify --in ms1-2026-08-23-backup.txt` is a FILENAME containing an HRP, not material, and must still be accepted. **Plus the negative control**: `ms vectors`, `ms gui-schema` and `ms gen-man --out <dir>` still exit 0, or the guard is refusing on the binary's name |
| 6 | **`--allow-argv-secret` IS A CHANNEL, NOT A FLAG** (§6d). The override's own parse happens on raw argv. When present, the layer removes **both** the override and the admitted token from the argv handed to clap, and carries the material in through the same internal path as `--in` content. The flag is also declared on all eight material verbs so `--help` documents it | `ms encode --allow-argv-secret --phrase <seed>` exits **0** and emits stdout byte-equal to `ms encode --in <the same phrase in a file>`. **Fails today**: the flag does not exist, so clap rejects it at 64. **And the naive implementation fails the second assertion, which is the point**: if the admitted token is left in argv for clap, an unrelated later parse error echoes it — so the gate also asserts **no material in stderr for `ms encode --allow-argv-secret --nosuchflag --phrase <seed>`**, where clap must name the flag and never the value. **Plus the control**: `ms encode --allow-argv-secret --phrase - < f` behaves exactly as without the override |
| 7 | **THE PURGE TEXT, RUN — NOT PRINTED.** The refusal calls `remedy::history_purge_block`, cited by symbol because the crate is in motion. **The `command` argument is the VERB-QUALIFIED invocation — `ms encode`, not `ms`** — because the recipe is a word-bounded `sed` pattern and a two-character command name is a collision generator: the crate's own doc records that `\bme\b` also removed `cd /home/me` from a six-line sample. No new purge text is written in `ms`; `ms` has none today to supersede | the emitted zsh recipe, **extracted from `ms`'s own stderr and RUN** under a real interactive zsh on a pty, removes the planted entry — with the harness control carried across, so a session that records nothing cannot pass. **Plus the same for bash.** **Plus the standing invariant**: `history -d` appears in **no recipe** while still appearing in the warning prose, which is why the recipes are structured data and the naive negative assertion is forbidden. **Plus the collision cost, asserted**: a neighbouring unrelated history line matching the pattern is also removed, measured, so the emitted text must say so |
| 8 | **THE SIBLING REMEDY BECOMES THE `--in` FORM** — in THIS repo, and §6h's standing instruction fires exactly here. `crates/me-cli/src/main.rs:2188` emits, to a secret-class operator, a line reading `ms encode --phrase - < seed.txt` piped into `me sysw pack --out p.bin`; the comment four lines above at `:2184` states `ms encode --in` DOES NOT EXIST (exit 64). Both change together; the comment is deleted, not amended | the advised line, **extracted from `me`'s own stderr and RUN**, exits 0 and produces a payload — the assertion `me`'s suite already makes, retargeted. **Fails today only in the direction that matters**: the current line is correct and must stay correct until `--in` ships, so this is the one entry whose gate is *"the old line stops being the only true one"*. The control is that `me`'s refusal must never advise a channel `ms` lacks — reproduced by running the emitted line, never by reading it, which is the mistake §6h records was made once already |
| 9 | **THE PRIVATE WRITE — `--out FILE` ON `encode`, `split` AND `repair`.** The three verbs whose stdout IS a canonical artifact; §6a rules `decode`, `verify` and `inspect` report verbs and out of scope, and `combine`/`derive` emit labelled reports rather than artifacts. `--out` writes through the crate's `write_private`, cited by symbol. It **overwrites**, per §6b's explicit ruling | `ms encode --out f` where `f` **already exists at 0644** leaves `f` at **0600** holding the new artifact — the `set_permissions`-on-the-open-file half, which a mode-on-create implementation fails, because `0o600` binds on CREATE only. Asserted with `fd::mode_of` rather than by eyeball. **Fails today**: no `ms` verb has `--out`, and `ms` has never created a file with a mode. **Plus the collision control**: `ms gen-man --out <dir>` still writes a DIRECTORY of man pages and still exits 0, and `crates/ms-cli/tests/gen_man.rs` is unchanged — the two meanings coexist and F-282 records that they do |
| 10 | **THE UNGROUPED STDOUT — `ms encode` ONLY** (§6a, §6b). stdout carries the canonical `ms1` and nothing else, always ungrouped. `--group-size` and `--separator` move to affecting **the stderr card only**, which §6b states in as many words; the card already exists on `ms encode` and gains the grouped string, which is after this the only place it exists. `--json` is untouched | `ms encode --phrase <the all-abandon vector>` with **no flags** piped into `me sysw pack` exits **0** and writes a payload. **Fails today at exit 4** — *"record 0 … is not a form this container can place"* — measured, with the `--group-size 0` run at exit 0 and 102 bytes as the live control that grouping is the only difference. **Plus the card assertion**: the grouped form appears on **stderr**, and `--no-engraving-card` removes it, which §6c warns is a real change in what a `2>/dev/null` pipeline throws away. **Plus the `--json` control**: `ms encode --json` output is byte-unchanged. `crates/ms-cli/tests/encode_grouping_flags.rs`, `crates/ms-cli/tests/encode_no_engraving_card.rs` and `crates/ms-cli/tests/decode_grouped.rs` are where this lands |
| 11 | **THE WHITESPACE-ONLY SEPARATOR** (§6c). `parse_separator` (`crates/ms-cli/src/format.rs:41`) loses its `hyphen` and `comma` arms and their literal forms, on `encode` and `split` alike — one parser serves both, so it cannot bind to one of them. **INTAKE IS NOT NARROWED**: `is_display_separator` (`crates/ms-cli/src/format.rs:12`) keeps stripping `-` and `,`, because a plate already engraved from a hyphen-grouped card must still decode, and `render_grouped` (`crates/ms-cli/src/format.rs:18`) keeps its `char` parameter | `ms encode --separator hyphen` and `--separator comma` each exit **64**; `--separator space` still exits 0. **All three exit 0 today**, and all three round-trip through `ms decode` at exit 0, measured — which is precisely why the cross-tool argument and not a per-tool one decides it. **Plus the two controls that keep the blast radius honest**: `ms decode` of a hyphen-grouped and a comma-grouped `ms1` still exits 0, proving intake was not narrowed; and `sha256sum -c display-grouping-vectors.tsv.sha256` still exits 0 with the file untouched, proving the conformance pin was not dragged along — its 5 hyphen and comma rows exercise `render_grouped` directly and never touch the CLI |
| 12 | **THE DRIVERS — 13 INVOCATIONS, NOT 18.** The material-bearing invocations in the seven scripts of §1.9 move to `--in` or `-`. The 5 non-material occurrences are left alone and named so a later reader does not "finish" the migration by editing them. `MS` is made overridable in the seven scripts that hard-bind it, following `design/journeys/derive-pathological-keys.sh:39` | **the precondition is part of the gate**: `cargo build --release` in `mnemonic-secret` must produce `target/release/ms`, which **does not exist today**, or nothing here can be run at all. Then: each migrated driver runs to completion at exit 0 against a P2 build, and `design/journeys/derive-rcw-keys.sh` and `design/journeys/derive-hashvault-keys.sh` produce **byte-identical** key output to their committed expectations — the drivers whose output is deterministic and whose only dependency is `ms`. **Fails today in both directions**: unrunnable for want of the binary, and against a P2 build every one of the 13 would exit non-zero on the argv refusal. **Plus the control**: `design/journeys/restore_test_pathological.py` and `design/journeys/restore_test_tr_pathological.py` are NOT edited and still pass, because they already use `--phrase -` |
| 13 | **THE SCHEMA STILL DESCRIBES THE BINARY.** `ms gui-schema` is clap-derived, so it follows the flag surface automatically; this entry asserts that it did, and files the third-repo mirror drift rather than fixing it | `ms gui-schema` exits 0 and its JSON names `--in` on all eight material verbs, `--allow-argv-secret` on all eight, and `--out` on `encode`, `split` and `repair`. **Measured baseline today: 10 subcommands carrying 36 flags** — derive 9, encode 7, decode 2, inspect 1, verify 3, vectors 1, gen-man 1, repair 2, split 8, combine 2 — so the count is asserted as **55**, and a flag that failed to reach the schema goes RED on the arithmetic rather than on a reader's attention. `crates/ms-cli/tests/gui_schema_emits_spec_v7_json.rs` is where it lands. **The `mnemonic-gui` mirror is NOT regenerated here** — §7 gives that to P3 — and F-283 records that P2 is when it goes stale for `ms` |
| 14 | **THE DECLINE, ASSERTED.** No code. `ms` keeps its own three-class output vocabulary, its own share reader, its own empty-input behaviour, its `gen-man --out DIR`, and its absence of any stdout mode gate — and the tests that pin them are named so a later phase cannot delete them as tidying. **Regression-gated, not RED-first** | `ms encode` to a **real pty** still exits 0 and still prints the `ms1`, so an adoption of `exit::write_block` that imported `me`'s terminal refusal goes RED. `ms encode > <a 0644 file>` still exits **0**, so a stdout mode gate smuggled in goes RED. `crates/ms-cli/tests/cli_output_class.rs:56` (`fn byte_parity_advisory_lines`) still passes, so `PayloadKind` did not displace `OutputClass`. `ms combine -` fed **grouped** shares still recovers the secret at exit 0, so `split_record_stream` did not displace `read_shares`. **Plus the enumerated diff**: every edit to `ms`'s 276 integration tests listed, each justified by a named §6 ruling or a numbered finding |

**EVERY `crates/ms-cli/` LINE NUMBER ABOVE IS ANCHORED AT `7c12f66` AND NAMES ITS
SYMBOL.** F-279 measured what happens otherwise: 14 of 15 of the sibling plan's
citations went stale under its own early work, and the citation gate reported
all 15 green because it checks that a line exists and not what is on it — its
own header says so in the "NOT covered" block. One of that plan's fourteen
landed on a real, unrelated function, which no dangling check can catch. **So:
locate every site by SYMBOL and re-measure the line before quoting it.** The
crate's own items carry no line numbers at all here, for the same reason at
greater force — two of its modules are being edited today.

**THE TESTS MIGRATE TO A PRIVATE CHANNEL, NOT TO THE OVERRIDE.** 147 of `ms`'s
276 integration tests live in the 31 files that pass `--phrase` or `--hex`, and
the cheap way to green them all is to append `--allow-argv-secret` to every
invocation. **That is forbidden here.** A suite that reaches the code only
through the override stops exercising what an operator experiences, and leaves
the refusal itself proven by a handful of cases. Tests migrate to `--in` or `-`;
`--allow-argv-secret` appears only in the tests that exist to prove the override.
`crates/ms-cli/tests/cli_derive.rs:347` (`fn inline_secret_argv_advisory`) is the
one test whose subject is the old advisory, and it is rewritten against the
refusal rather than deleted.

**THREE PIECES OF WORK ARE REGRESSION-GATED RATHER THAN RED-FIRST**, and the
column header must not claim otherwise: the pin, whose gate is a build
resolving; the decline, which is a backstop protecting RED-first work rather
than proving it; and the sibling remedy, whose current text is *correct today*
and whose gate is that a second true form appears — stated in its own cell
rather than left for a reviewer to notice. **Everything else is RED-first, and
every one of those gates was RUN today and observed to fail**, with the failing
exit code and the failing output quoted in the cell.

**`ms`'s TEST COUNT IS 276 INTEGRATION TESTS IN 76 FILES, AND `cargo nextest run
--locked` REPORTS 414 RUN, 414 PASSED, 5 SKIPPED** — the larger figure includes
the 146 `#[test]`s inside `src/`. Both numbers are given because §7's P2 gate
speaks of *"round-trip vectors"* and neither number is the validation surface;
see below.

---

## 5. WHAT MUST BE TRUE TO CLOSE P2

1. **`ms`'s WHOLE validation surface is green, and it is wider than one `cargo
   test`.** `cargo +1.95.0 fmt --all --check`, `cargo clippy -p ms-cli
   --all-targets -- -D warnings`, `cargo clippy -p ms-codec --all-targets -- -D
   warnings`, `cargo nextest run --locked`, the `display-grouping-vectors`
   checksum pin, the `miri` mlock job, the three fault-injection steps, the
   release-mlock job, `freebsd-compile-gate`, both `musl-check` targets, and the
   **two cross-repo byte-parity gates** — `g6 invariant (cross-repo mlock.rs)`
   and `fn byte_parity_advisory_lines`. All five that a developer machine can run
   were measured green today, before P2 wrote a line.
2. **`--in FILE` reads material on all eight verbs, asserted as equality with
   the stdin run rather than as success**, and a missing `--in` file fails
   naming the path instead of silently falling back to stdin. The six channels
   and the two-channel ruling build this.
3. **`ms encode --in f` on a file of legal-length hex REFUSES and names
   `--hex -`, while `ms encode --hex - < f` succeeds.** The two-channel ruling
   builds this, and it is the single assertion that keeps a content-sniffing
   design out.
4. **`ms verify --in card.txt --phrase -` and `ms derive --in card.txt
   --passphrase-stdin` each exit 0**, while `ms verify - --phrase -` still exits
   1. The freed stdin builds this.
5. **No seed material reaches stderr for any argv carrying it**, in any of the
   four spellings, on any of the 14 channels — 56 rows, generated. The guard
   builds this.
6. **`ms encode --phrase "<a real seed>"` REFUSES, and `--allow-argv-secret`
   proceeds.** §7's P2 gate requires exactly this. Today it exits 0 in silence.
   The guard and the override build it.
7. **`--allow-argv-secret` exists on `ms`, and its decision AND the admitted
   material's route are both settled before `Cli::try_parse()`.** A guard that
   reaches its decision by parsing first has reintroduced the leak §6d exists to
   stop; an override that hands the admitted token back to clap has created a new
   one. The override builds this.
8. **Every purge recipe `ms` emits has been RUN, in a harness with a passing
   control, and observed to purge.** Not printed — run. The purge text builds
   this.
9. **`me`'s refusal advises only channels that exist, verified by RUNNING the
   line it emits.** §7's P2 gate requires it and §6h records that the rule was
   earned by shipping the opposite once. The sibling remedy builds this.
10. **`ms --out` creates its file 0600 on CREATE and on OVERWRITE**, and
    `ms gen-man --out <dir>` still writes a directory. The private write builds
    this.
11. **`ms encode` with no flags, piped into `me sysw pack` with no flags, exits
    0 and writes a payload.** §7's P2 gate requires it; today it exits 4. The
    ungrouped stdout builds this.
12. **`ms` offers no hyphen and no comma separator, and still DECODES both.**
    The whitespace separator builds this, and the second half is what keeps an
    already-engraved plate readable.
13. **The 13 material-bearing driver invocations run privately, and the two
    deterministic key-derivation drivers reproduce byte-identical output** —
    which first requires a release build of `ms` to exist, and it does not
    today. The drivers build this. Without it, §7's P4 gate — *"a captured
    journey that regenerates"* — is unsatisfiable when P4 is reached, which is
    the ordering blocker §7 records as I-10.
14. **`ms`'s policy is unchanged wherever no §6 ruling changes it** — its
    three-class output vocabulary, its own share reader, its `gen-man --out DIR`,
    its permissive terminal, and its absence of any stdout mode gate. The
    decline builds this.
15. **The diff to `ms`'s 276 integration tests is enumerated, each edit
    justified by a named §6 ruling or a numbered finding, and no test was
    greened by appending `--allow-argv-secret`.** The decline builds this.
16. **(assertion, not work — no entry builds it; it is checked, not created)**
    **§6f needs nothing from P2.** `ms`'s measured codes — clap 64, invalid
    artifact 1, repair-uncorrectable 2, repair-applied 4 — are already §6f's
    `ms` row, and §6f rules `mk`'s invalid-artifact 2 the only code this cycle
    changes, in P3. A P2 that renumbered anything here would be acting outside
    its ruling.
17. **(assertion, not work — checked, not created)** **Both closure vocabularies
    were grepped before anything was scheduled.** This repo closes follow-ups as
    `CLOSED` **and** as `DONE` — 138 and 45 occurrences respectively in
    `design/FOLLOWUPS.md` today — and §8 records that a single-token sweep
    reported half the truth with total confidence once already. `mnemonic-secret`
    has its own `design/FOLLOWUPS.md` and it must be swept too.
18. **An R0 round closing 0C/0I.**

---

## 6. OUT OF SCOPE

- **A world-readable-stdout gate for `ms`, refusal OR warning.** P2 builds none:
  §7's P2 row and gate list enumerate P2's content and include none; §6e's
  retraction argument names `ms encode` as the case where such a gate makes the
  exposure strictly worse; the directly analogous operator ruling F-275
  (`mt decode`, 2026-08-27) chose warn-and-proceed over refusal for human-read
  output under the default umask 022; and `ms` already prints an unconditional
  private-material line on the same stream. Whether §9a's in-scope line obliges
  any `ms` mode check at all — and if so, whether that existing line already
  discharges it — is F-281, for an operator ruling, not for this plan.
- **Renaming `ms gen-man --out`.** §1.6. It is shipped, exampled, and driven by
  `mnemonic-secret`'s `man-release.yml` workflow. F-282.
- **Regenerating `mnemonic-gui`'s schema mirror.** §7 gives it to P3. P2 asserts
  only that `ms gui-schema` describes the new surface. F-283.
- **Changing `ms split`'s grouping default.** §6a rules the stdout rule binds
  `encode` only, and the packability argument that decides `encode` does not
  reach `split`: measured, `me sysw pack` refuses a codex32 share at exit 4
  **grouped and ungrouped alike**, so grouping is not what blocks it. F-284.
  The separator rule DOES reach `split`, because one `parse_separator` serves
  both verbs and cannot bind to one of them.
- **`--out` on `decode`, `verify`, `inspect`, `combine` or `derive`.** §6a rules
  `decode`, `verify` and `inspect` report verbs whose stdout shapes are
  explicitly out of scope this cycle; `combine` and `derive` likewise emit
  labelled reports rather than a canonical artifact, and §6b's `--out` is
  *"write the artifact to a file"*. That `ms decode` and `ms combine` write a
  recovered seed to an unprotected stdout is real and is F-285, not a silent
  omission.
- **Settling `0o044` against `0o077`.** `ms` has neither and P2 gives it
  neither; the private write creates 0600 and asks no mask question.
- **`ms`'s exit codes.** §6f, and condition 16 above.
- **`--json` schema uniformity.** §6b places it outside the cycle entirely.
- **`mnemonic-toolkit`'s adoption.** It is the sixth consumer, not P2's — and
  the two cross-repo parity gates mean any change to `ms`'s advisory wording
  must be reasoned about jointly with it, which is a further reason not to touch
  that wording here.
- **Publishing `mnemonic-io-lib`.** F-271 records the publish as authorised and
  its pre-flight as unrun; P2 reaches the crate by the same rev pin P1 uses.

---

## 7. CONSULTS, AND WHAT THEY DECIDED

Two ambiguities could not be settled by reading or running, and each was put to
one consult as a single question.

- **What `--in FILE` means on the two verbs that have two material channels.**
  Decided: **it means a PHRASE**, joining the existing required `ArgGroup` as a
  third alternative, with hex-from-a-file keeping `--hex -`. Content-sniffing
  was rejected on a specific hazard rather than on taste: today's sniff would be
  safe only because a phrase always contains whitespace, and that restraint is
  invisible, so a later maintainer being liberal with whitespace turns a
  hex-alphabet BIP-39 phrase into valid entropy for a **different wallet** — a
  valid, wrong plate. The phrase-only rule has no input that both parses as
  BIP-39 and reads as entropy. It also lets `me`'s remedy be one correct command
  with no kind flag, which §6h requires. The counterexample test is the
  two-channel ruling's gate.
- **Whether P2 owns a world-readable-stdout gate for `ms`.** Decided: **build
  nothing, and file it for an operator ruling.** The reasoning is in §6's first
  bullet, and F-281 carries it with the F-275 precedent attached so the eventual
  ruling is cheap and pre-shaped: if anything is ever built, it is a warning,
  never a refusal.

---

## 8. FILED, NOT BUILT

Six entries are added to `design/FOLLOWUPS.md` by this plan. Each carries an
owning phase, per the per-phase burndown rule.

- **F-281** — whether `ms` should gate a world-readable stdout at all, given
  §9a lists the gate in scope while P2's row does not, and given that `ms`
  already prints an unconditional private-material line. **Owning phase:
  operator ruling before the cycle closes.**
- **F-282** — `ms gen-man --out <DIR>` collides with the `--out FILE` this
  cycle introduces; one binary, two meanings. **Owning phase: a later cycle**,
  because renaming it breaks a release workflow.
- **F-283** — `mnemonic-gui`'s schema mirror for `ms` goes stale in P2 while
  §7 gives its regeneration to P3. **Owning phase: P3.**
- **F-284** — `ms split`'s default grouping is unchanged by P2, so `ms encode`
  and `ms split` disagree about their own stdout after this phase. **Owning
  phase: P3**, with the grouping work for `md` and `mk`.
- **F-285** — `ms decode` and `ms combine` write a recovered seed phrase to an
  unprotected stdout and gain no `--out` in P2. **Owning phase: operator ruling**,
  alongside F-281, since both are the same question about the same tool.
- **F-286** — `plan-cite-check.sh` strips a leading dot from a path, so every
  `.github/workflows/…` citation is reported DANGLING no matter which repo owns
  it. Reproduced with a control: a workflow file that exists in **this** repo,
  anchored at its line 1, is reported *"no such file under any root"* — the
  dot is gone from the path the gate looked for. The probe is not quoted here,
  because quoting it makes THIS document fail the gate, which happened once
  while drafting. It is a false DANGLING, which the script's
  own header argues is worse than no gate because it teaches a reader to skim
  the output. **Owning phase: the ownerless residue** — it blocks nothing, and
  the workaround is one sentence of prose.

Two facts already recorded elsewhere are re-pointed rather than re-filed:
**F-276**, the crate's `me`-shaped boundary, gains a third site in §2.3 — the
`observation` module cannot express `ms`'s existing three-class vocabulary — and
**F-279**, the stale-citation class, is the reason every anchor in this document
names its symbol.
