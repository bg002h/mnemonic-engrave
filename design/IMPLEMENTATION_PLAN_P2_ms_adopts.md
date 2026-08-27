# IMPLEMENTATION PLAN — P2: `ms` adopts `mnemonic-io-lib`

**Status:** DRAFT v2, written 2026-08-27, **R0 round 0 folded the same day**
(2C/8I/8M/0Nit; report at `design/agent-reports/R0-P2-plan-round0.md`, response
at `design/agent-reports/FOLD-P2-round0.md`). No code may be written until this
closes an R0 round at 0C/0I.

**OPERATOR RULING, 2026-08-27, and it re-grades one of round 0's Criticals.**
*"While we prefer secret handling to work well, failure to handle secrets
secretly will no longer be critical or important. Such issues will be logged
when discovered for future optimization."* So a defect whose harm is **material
becoming visible** is recorded and scheduled, never gate-holding. A defect that
is a **gate which cannot fail**, a **refusal that does not refuse**, or a **tool
advising a path that does not run** still blocks, whatever sits nearby — the
ruling re-grades a class of harm, not a class of wording, and this plan does not
relabel work in either direction to move it across that line.

**Gates this plan is checked by** — run each **separately** from the commit,
never on the same shell line: `scripts/plan-stepref-check.sh` (prose may not name
a step number), `scripts/plan-table-check.sh`, `scripts/plan-cite-check.sh`.

**CITATIONS INTO `ms` RESOLVE, AND WERE PROVEN TO** —
`/scratch/code/shibboleth/mnemonic-secret` is already in `plan-cite-check.sh`'s
`ROOTS` list (`scripts/plan-cite-check.sh:101`, inside the array opened at
`scripts/plan-cite-check.sh:95`), added before this cycle. **The line was
re-measured during the R0 round-0 fold and had moved**: the first draft cited
line 78, which the script's own 2026-08-27 repair pushed down into a comment
block — the gate reported it `ok`, because it checks that a line exists and not
what is on it, which is the same F-279 shape this plan warns about twice. A probe
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
PURPOSE — AND THE ORIGINAL REASON GIVEN HERE WAS ALREADY FALSE WHEN IT WAS
WRITTEN.** The first draft justified the rule by saying the crate was still
being edited: the fish recipe not yet prescribed, `write_private` not yet moved.
Both had already landed — re-measured against `origin/master`
(`6c24e62823e6c1ac02aa3862cd6020674bf58544`): `history_purge_recipes` returns
`("fish", "history clear-session")` and the module doc says fish is prescribed;
`git grep -n 'fn write_private' origin/master -- crates/` returns **one** hit,
in `mnemonic-io-lib`'s `write` module, and **none** in `me-cli`. R0 round 0's
I-1 caught it. **The rule survives on a better reason**, which is why it is kept
rather than dropped: F-279, filed 2026-08-27, measured **14 of 15 of the sibling
plan's `mt` line citations gone stale under its own early work, with
`plan-cite-check.sh` reporting all 15 green**, because it checks that a line
exists and not what is on it — and one of those fourteen landed on a real,
unrelated function, which no dangling check can catch. A plan that will be
executed over days cites what cannot drift. Every `ms` citation below therefore
names the SYMBOL beside the number, and the crate gets the symbol alone.

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
codes: clap usage **64**, invalid artifact **1**, repair-uncorrectable **2**,
and **repair-applied 4** — re-measured 2026-08-27 as part of round 0's M-7,
which found the 4 asserted downstream in this document while §6f's row here
listed only three. `ms repair --ms1 <an ms1 with one induced error>` exits
**4**.
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
**twelve** invocations below exit **0** and produce the same artifact as the
argv form (round 0's M-2: the first draft wrote *eleven* over a list of twelve,
recounted here rather than carried):

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

### 1.9 This repo's journey drivers — 18 LINES, 20 invocations, and 13 to migrate

§7 says *"18 argv call sites across 7 scripts"*. **The 18 reproduces exactly and
is a count of `"$MS"` LINES, not of invocations, and not of invocations that
expose material.** Round 0's M-3 found the first draft's column mislabelled —
it was headed *occurrences* while the method quoted, `grep -n`, counts lines —
so both counts are given below and were re-measured independently, `grep -c`
against `grep -o` piped to `wc -l`:

| script | lines (`grep -c`) | invocations (`grep -o` piped to `wc -l`) | of which carry material on argv |
| --- | --- | --- | --- |
| `design/journeys/transcript.sh` | 2 | 2 | 1 |
| `design/journeys/transcript_hashvault.sh` | 4 | 4 | 3 |
| `design/journeys/transcript_pathological.sh` | 3 | 3 | 2 |
| `design/journeys/transcript_tr_pathological.sh` | 2 | 2 | 2 |
| `design/journeys/derive-rcw-keys.sh` | 3 | **4** | 2 |
| `design/journeys/derive-pathological-keys.sh` | 2 | 2 | 0 |
| `design/journeys/derive-hashvault-keys.sh` | 2 | **3** | 3 |
| **total** | **18** | **20** | **13** |

Two lines each carry **two** nested invocations — `design/journeys/derive-rcw-keys.sh:66` and
`design/journeys/derive-hashvault-keys.sh:35`, each an `ms decode` wrapped around an
`ms encode --hex` — which is the whole of the 20-versus-18 gap, and is why one
script's material count exceeds its line count. **The residue is 7, not 5**:
2 `[ -x "$MS" ]` existence tests, 3 `--version` calls, and 2 invocations already
using `--phrase -`, and 20 − 13 = 7. The first draft subtracted from the line
count instead of the invocation count and published 5; the enumeration it
published alongside already summed to 7, so the document disagreed with itself.
**A gate reading *"the 18 argv call sites migrated"* literally is unsatisfiable
— 7 of the 20 are not invocations of material at all.** The number to satisfy
is **13**, and that number is unchanged: all 20 invocations were walked
individually.

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

`channel::destination` is declined, and this is a change from `mt`, which
adopted it. **The verdict is right and the first draft's REASON for it was
falsified by the sibling** — round 0's M-5, which the reviewer was asked to
settle and did. The retracted reason was that `destination`'s other two arms
exist to feed `write_block`, which `ms` declines. That cannot be the
discriminator: `IMPLEMENTATION_PLAN_P1_mt_adopts.md`'s own verdict table
declines **both** `exit::write_block` and `exit::WriteBlock` and adopts
`channel::destination` anyway, so `mt` reaches the opposite verdict from the
same premise.

**The real discriminator is stated elsewhere in this plan and is now carried
here: what a consumer has to MAP the non-`File` arms onto.** `destination`
answers *"file, stream, or terminal"*. `mt` has a world-readable-stdout gate and
a terminal policy of its own, so `Stream` and `Terminal` each land on something
that acts — P1's `--out` work says so in as many words, adopting `destination`
*"with `mt` mapping `Terminal` onto its own permissive policy rather than `me`'s
refusal"*. **`ms` has neither.** P2 builds no stdout mode gate (§6, first
bullet) and the decline work pins its absence, so in `ms` both non-`File` arms
are dead on arrival and `--out` needs only *"is `--out` given"*, which is
`Option::is_some`. **Adopting a three-way classifier to consume one arm of it
puts `Terminal` into `ms`'s vocabulary with nothing that may act on it** — the
same dead-variant shape §2.3 declines `PayloadKind` for.

**F-276 gains nothing from this item.** The crate's boundary is not at fault:
two consumers with different policies reached different verdicts about the same
sound item, which is what a boundary is for. §2.3's `observation` finding is a
separate matter and does add a site.

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
here; it is the first private way to do two things at once** — for those two
shapes — and that is the argument for doing it before the refusal rather than
after.

**AND THERE IS A THIRD SHAPE WHERE THE CLAIM ABOVE DOES NOT HOLD, WHICH THE
FIRST DRAFT DID NOT NAME ANYWHERE.** Round 0's I-3. §6d binds `--in` on `derive`
to the **`ms1` positional**, not to `--phrase` — re-measured, the positional is
ms1-only: `ms derive - < <a file holding a BIP-39 phrase>` exits **1** with
`error: string length 82 not in v0.1 set [50, 56, 62, 69, 75]`. So an operator
who has a **paper seed phrase** and a passphrase — the recovery shape, not the
card shape — loses their one-command form the moment the refusal ships:
`ms derive --phrase <seed> --passphrase <pass>` exits 0 today (with two argv
advisories) and is refused afterwards, while `--phrase -` plus
`--passphrase-stdin` is the contention refusal above and `--in` reads the wrong
kind.

**THE REVIEWER CONCLUDED FROM THIS THAT NO PRIVATE FORM REMAINS. THAT IS TOO
STRONG, AND THE COUNTER WAS MEASURED RATHER THAN ARGUED.** A private route
exists after P2 and it is two commands, because `ms` can convert the phrase to
the kind `derive --in` reads:

```
ms encode --in seed.txt --out card.ms1      # the phrase never touches argv; 0600
ms derive --in card.ms1 --passphrase-stdin < pass.txt
```

Reproduced today in the closest form the binary currently supports — `--in` and
`--out` do not exist yet, so the phrase came in on stdin and the card went to a
variable:

```
ms encode --phrase - --group-size 0 < seed.txt        -> rc 0, ms10entrsq…34v7f
ms derive "$CARD" --passphrase-stdin < pass.txt       -> rc 0, master_fingerprint: ca2c62d2
```

**So the finding is real but its remedy is not a missing capability; it is a
missing SENTENCE.** What P2 owes is that the two-command route be written down
and asserted, because an operator who cannot find it will reach for
`--allow-argv-secret`, which is the exposure this phase exists to close. The
freed-stdin work carries it as a control. **The one-command form is filed, not
built** — F-303, which is where a `--passphrase-file` or a phrase-shaped `--in`
on `derive` belongs; it is a new channel, and §7's P2 row enumerates P2's
content and does not include one.

---

## 3. THE DEPENDENCY — the crate IS reachable, and this section's first draft said the opposite

**THE BLOCKER THIS SECTION WAS BUILT ON IS DISCHARGED, AND THE FIRST DRAFT WAS
ALREADY WRONG WHEN IT WAS MERGED.** Round 0's I-1. The retracted claim was that
`mnemonic-io-lib` was absent from `origin/master`, that fish was still only
described rather than prescribed, and that `write_private` still sat in
`me-cli`. Re-measured 2026-08-27, each command run separately and its exit code
read directly:

| re-measured | result |
| --- | --- |
| `git ls-tree -d origin/master crates/` | names **both** `crates/me-cli` and `crates/mnemonic-io-lib` |
| `git grep -n 'fn write_private' origin/master -- crates/` | **one** hit, in `mnemonic-io-lib`'s `write` module; **zero** in `me-cli` |
| the crate's `history_purge_recipes` | returns `("fish", "history clear-session")`, and the module doc records fish as prescribed |

`origin/master` is `6c24e62823e6c1ac02aa3862cd6020674bf58544`. **So the pin is
available now**, and an implementer must not wait for a push that has already
happened.

**P2 pins that SHA for `ms`**, in `crates/ms-cli/Cargo.toml`, as
`mnemonic-io-lib = { git = "https://github.com/bg002h/mnemonic-engrave", rev = "<the SHA above, or a later one carrying the same items>" }`.
The rev is written out rather than left as a placeholder because there is no
longer a future event to name. If P1 lands further crate work before P2 starts,
the pin moves forward to that SHA; it never moves backward, and the items P2
adopts — the remedy pair, `fd::mode_of`, `write_private` — must all be present
at whatever rev is chosen, which the pin work's gate checks by building.

**This plan does not design a workaround and must not be given one.** A `path =`
dep does not resolve in a fresh CI checkout, and `ms`'s CI is wider than most —
`freebsd-compile-gate` and `musl-check` both build from a clean checkout on
foreign targets, so a path dep out of the workspace fails there first. The
argument is the one `me-cli`'s own Cargo.toml already records for `mt-codec`.
**That paragraph is unaffected by the retraction above** and is the reason a
`path =` dep is still forbidden even now that no push is being waited on.

**WHAT THE RETRACTION COSTS THE ORDERING — AND WHAT IT DOES NOT.** The first
draft derived the whole crate-free-work-first sequence from the pin being
blocked, and called that derivation *"the load-bearing part"*. **It is void:
nothing is blocked, so nothing is being sequenced around a block.** The order in
the table below is unchanged all the same, and now rests on two reasons that
were always the real ones:

- **§7's P2 row rules it** — *"FIRST `--in` on all eight verbs, THEN the argv
  refusal, THEN the 0600 `--out`, THEN `--group-size 0` … and the whitespace-only
  separator"* — and a phase does not re-order its own spec row.
- **The argv refusal genuinely depends on the pin**, because §6d requires the
  refusal to carry the purge commands per §6h and §7's P0 row rules that the
  text comes **from `me` alone**. Writing the refusal before the pin means
  writing a purge paragraph that would then be deleted. So the pin still sits
  ahead of the guard in the table — for a dependency that is real, rather than
  for a blockage that is not.

---

## 4. TDD ORDER

Each entry is RED first unless its gate column says otherwise. No entry begins
until the previous is green. **This table is the only ordering of record**;
prose refers to work by NAME — *the six channels*, *the two-channel ruling*,
*the freed stdin*, *the pin*, *the guard*, *the override*, *the purge text*,
*the private write*, *the ungrouped stdout*, *the whitespace separator*, *the
sibling remedy*, *the drivers*, *the schema*, *the decline* — so a renumbering
cannot falsify it. **The list is in table order and one entry MOVED in the R0
round-0 fold**: the sibling remedy now follows the ungrouped stdout, because
until `ms encode`'s stdout is packable neither the old advice nor the new one
runs. Its own cell carries the reason.

**THE ORDER HONOURS §7's RULING AND IS NOT FREE TO BE REARRANGED.** P2's row
states its contents as *"FIRST `--in` on all eight verbs, THEN the argv refusal,
THEN the 0600 `--out`, THEN `--group-size 0` … and the whitespace-only
separator"*, and marks the phase *"highest safety value; do it before the
cosmetic work"*. §6d separately retracts the *safety* justification for that
sequence — it had rested on a false claim that `ms combine` had no private
channel — and calls the sequencing *"a preference"*. **The ruling stands
regardless**, and the reason first given for it does not: *"it keeps the
crate-free work off the critical path of P1's push"* is retracted with §3's
blocker, since there is no push to be off the critical path of. What holds the
order is §7's ruling itself, plus the one dependency that is real — the guard
needs the crate's purge block, so the pin precedes it.

**THE SIBLING REMEDY IS THE ONE ENTRY §7's RULING DOES NOT PLACE**, which is
what makes moving it legitimate: §7's P2 row enumerates `ms`-side content and
lists the remedy under P2's **gate**, not its contents. So it is sequenced by
its own dependencies, and those put it after the ungrouped stdout.

| # | work | the gate that must fail first |
| --- | --- | --- |
| 1 | **`--in FILE` ON THE SIX VERBS WHERE ITS BINDING IS UNAMBIGUOUS.** `decode`, `verify`, `inspect`, `repair`, `derive`, `combine`. **They are NOT single-channel verbs and the first draft's title said they were** (round 0's M-1, contradicting §1.4's own measurement four pages earlier): `verify` carries 2 argv channels and `derive` carries 4. What is single here is the channel `--in` BINDS to — §6d's per-verb table gives each of the six exactly one, the positional (`--ms1` on `repair`, the share list on `combine`) — which is why these six need no ruling and `encode`/`split` do. §2.5 records what that binding costs `derive`. One new source threaded through `read_input` (`crates/ms-cli/src/parse.rs:21`) and `read_phrase_input` (`crates/ms-cli/src/parse.rs:36`), and through `read_shares` (`crates/ms-cli/src/cmd/combine.rs:54`) for the variadic case, where `--in` reads one share per line with display separators stripped exactly as the stdin path already does | each of the six: stdout **and** stderr byte-equal to the stdin run, at exit 0 — an equality assertion, never a bare success, which a `--in` silently ignored would also satisfy. **Fails today: each exits 64**, measured, `ms` mapping every clap error to 64 at `crates/ms-cli/src/main.rs:176`. **Plus two controls.** `--in <a nonexistent path>` must fail naming the path and must NOT fall back to stdin, or a typo silently reads a terminal. And `--in f` together with the verb's `-` must REFUSE, matching the two contention refusals `ms` already ships — a channel that silently wins over another is how an operator engraves the wrong card |
| 2 | **`--in FILE` ON `encode` AND `split`, MEANING A PHRASE.** Both declare a required clap `ArgGroup` — `"input"` at `crates/ms-cli/src/cmd/encode.rs:27` and `"split_input"` at `crates/ms-cli/src/cmd/split.rs:28`, each currently `.args(["phrase", "hex"])`. `--in` joins the group as a third alternative and resolves to the **phrase** channel through `resolve_secret_payload` (`crates/ms-cli/src/cmd/encode.rs:77`). Hex from a file keeps using `--hex -` with a redirect. **Ruled by consult, 2026-08-27**, over a content-sniffing design; see §7 | **the counterexample test, and it is the whole ruling**: a file holding exactly 64 legal hex characters — a valid entropy length — must make `ms encode --in f` exit non-zero, emit **no** `ms1` on stdout, and name `--hex - <` in its stderr; while `ms encode --hex - < f` on the same file exits 0 and emits the artifact. That single test goes RED if the design drifts to sniffing, RED if a later reader "fixes" `--in` to accept hex, and simultaneously proves the channel it redirects to works. **Fails today at exit 64 on both halves of the first assertion.** **Plus the group's own tests**, `crates/ms-cli/tests/encode_arg_group_violations.rs`, which pin exit 64 on both-supplied and neither-supplied and must be extended to three members rather than rewritten |
| 3 | **`--in` FREES STDIN, AND THE TWO CONTENTION REFUSALS BECOME SATISFIABLE.** No new code beyond what the two preceding entries build; this is the assertion that they composed. §2.5 | `ms verify --in card.txt --phrase -` exits **0** and reports the round trip; `ms derive --in card.txt --passphrase-stdin` exits **0** and derives with the passphrase applied. **Both fail today at rc 1**, with `cannot read both ms1 and --phrase from stdin` and `cannot read both the entropy source and --passphrase from stdin (one stdin per invocation)`, measured. **Plus the control that keeps the refusals alive**: `ms verify - --phrase -` must STILL exit 1, or the fix removed the guard instead of routing around it. **Plus the THIRD shape, which is the one `--in` does NOT free and which the first draft claimed nowhere and denied by implication** (round 0's I-3): a phrase plus a passphrase on `derive`. `--in` on `derive` reads an `ms1`, so there is no one-command private form; the two-command route — `ms encode --in seed.txt --out card.ms1`, then `ms derive --in card.ms1 --passphrase-stdin < pass.txt` — must be asserted end-to-end at exit 0 and must derive the SAME fingerprint as `ms derive --phrase <seed> --passphrase <pass>` does today, so the route is proved equivalent and not merely runnable. Measured in today's closest available form (no `--in`, no `--out` yet): `ms encode --phrase - --group-size 0 < seed.txt` → rc 0, then `ms derive "$CARD" --passphrase-stdin < pass.txt` → rc 0, `master_fingerprint: ca2c62d2`. **The refusal's own text must name this route**, or the operator reaches for `--allow-argv-secret` instead; F-303 carries the one-command form |
| 4 | **PIN THE CRATE.** Add `mnemonic-io-lib` to `crates/ms-cli/Cargo.toml` by GitHub rev, at `6c24e62823e6c1ac02aa3862cd6020674bf58544` or a later `origin/master` carrying the same items. No `path =`, no publish. **NOT BLOCKED — §3's blocker is discharged**, and the first draft both asserted a blocker that no longer exists and wrote a gate that was already green. **Regression-gated, not RED-first** | **The retracted gate was *"`git ls-tree -d origin/master crates/` names `crates/mnemonic-io-lib` — it does not today"*. It does, and it did before this plan was merged**, so that assertion could never have failed (round 0's I-1). The gate is instead, each command run separately: (a) `grep -c mnemonic-io-lib crates/ms-cli/Cargo.toml` is **0 today** and 1 after; (b) a file whose only content is `use mnemonic_io_lib::{fd::mode_of, remedy::history_purge_block, remedy::history_purge_recipes, write::write_private};` **compiles** — which goes RED if the pinned rev predates any adopted item, the failure the retracted gate was reaching for and could not express; (c) `cargo build --locked` in `mnemonic-secret` resolves the rev with `CARGO_HOME` pointed at an empty directory, so a cargo cache cannot supply it; (d) `cargo nextest run --locked` still reports 414 of 414. **No freeze ritual is owed here** — that clause belonged to the push this row no longer waits for; if a later `mnemonic-engrave` `master` push IS needed to move the pin forward, it follows this repo's staging-ref rule and prints no `Bypassed rule violations` |
| 5 | **THE ARGV GUARD, ON RAW ARGV, BEFORE THE PARSER.** A pre-parser layer between `process_hardening::set_non_dumpable()` (`crates/ms-cli/src/main.rs:169`) and `Cli::try_parse()` (`:170`), reading `std::env::args()` — the first `env::args` site `ms` has ever had. Two layers per §6d: **flag-keyed** (a static list of the 14 secret-bearing channels of §1.4, matched as strings, no parse), and **value-shape** (`ms1` by HRP, a BIP-39 phrase by wordlist, hex by charset and length) for material arriving positionally. **EACH TOKEN IS NORMALISED FOUR WAYS BEFORE EITHER LAYER SEES IT, AND THE FOURTH IS THE ONE THE FIRST DRAFT DROPPED:** trim, ASCII-lowercase, the token whole, **and every `=`-split half of it** — because `--phrase=<seed>` is ONE argv token whose left half is not the flag string and whose right half is the secret, so neither layer is even scoped to look at it. This is the donor's own normalisation, not an invention: `argv_candidates` (`crates/me-cli/src/main.rs:350`) does `v.extend(token.split('=').map(norm))` at `crates/me-cli/src/main.rs:354`, and the doc above it at `crates/me-cli/src/main.rs:347` says why. Splitting on **every** `=` rather than the first costs nothing and cannot miss a shape. The refusal names the class and the LENGTH, never the value, names `--in FILE` and `-`, and carries the purge block the purge work adds | **a generated cross-product, not a hand list, and it was GENERATED AND RUN while this plan was folded rather than extrapolated from §1.4's 12 rows** (round 0's M-8 caught the extrapolation). 4 value spellings — canonical, leading-space, trailing-space, UPPERCASE — over 14 channels, times 2 join forms on the **9 flag channels** (space-joined and `=`-joined) and 1 on the **5 positional channels**: 9×4×2 + 5×4 = **92 rows**. Measured 2026-08-27 against the tree's build: **84 of the 92 pass material at exit 0** — **58 in total silence**, **26 carrying `derive`'s advisory only** — and **0 of 92 leak material into stderr today**, so the leak half of the assertion is green everywhere before a line is written and only the exit-code half is a live gate on those 84. **The remaining 8 rows already exit non-zero, and they are NOT gates unless the assertion is sharpened**: UPPERCASE `--phrase` on `encode`, `verify`, `split` and `derive`, in both join forms, exits **1** with `error: unknown BIP-39 word at position 0` and no material in stderr — non-zero AND silent today, so *"exit non-zero and no leak"* can never fail there. **Those 8 rows therefore assert the GUARD's own refusal text, identified by a string only the guard emits, not merely a non-zero exit** — otherwise clap's wordlist error satisfies them forever. The leak assertion is *"no whole material value, and no constituent word of 4+ characters, appears in stderr, case-insensitively"* — **not** *"the material's own characters"*, which round 0's M-6 showed a 12-word English phrase makes unsatisfiable against any English sentence, the canonical refusal included. **WHAT THE 92 DOES NOT COVER, stated because an unstated gap is worse than a narrow gate**: the `--` end-of-options form (`ms decode -- <ms1>`, measured rc **0** today) is a real shape and is absent from the cross-product — the raw-argv scan reaches it because it honours no `--`, but nothing here proves that; and any shape where the material is neither a whole token nor an `=`-delimited half. Abbreviated long flags are NOT a shape on `ms` (`ms encode --phr <seed>` exits **64**, `error: unexpected argument '--phr' found`), and no material channel has a short alias (measured: only `-h`, and `split`'s `-k`/`-n`, which carry no material). **Plus the clap-echo control, which is the one the naive implementation fails**: material that the shape layer does not classify must not reach clap and be printed back. **Plus the near-miss control**: `ms verify --in ms1-2026-08-23-backup.txt` is a FILENAME containing an HRP, not material, and must still be accepted. **Plus the negative control**: `ms vectors`, `ms gui-schema` and `ms gen-man --out <dir>` still exit 0, or the guard is refusing on the binary's name |
| 6 | **`--allow-argv-secret` IS A CHANNEL, NOT A FLAG** (§6d). The override's own parse happens on raw argv. **THE MATERIAL IS SUBSTITUTED OUT, NOT REMOVED — and the first draft said removed, which cannot pass its own gate** (round 0's I-2). Removing the admitted token strands `encode` and `split`, whose required `ArgGroup` then has no member: measured, `ms encode` exits **64** with `error: the following required arguments were not provided:` followed by the group's own usage line naming `--phrase` and `--hex` as alternatives, `ms split -k 2 -n 3` the same, and removing only the value gives `ms encode --phrase` → **64**, `error: a value is required for '--phrase <PHRASE>' but none was supplied`. So the layer **substitutes rather than removes**: the admitted token is replaced by `-`, the stdin sentinel `ms` already parses on every one of the 14 channels — on a flag channel the flag stays and only its value becomes `-`; on a positional channel the positional becomes `-` — the override token itself is dropped, and the material is seeded into the internal path `read_input`/`read_phrase_input`/`read_shares` consult **before** stdin. `-` is not the material, so nothing is re-presented to clap that §6d forbids. §6d rules only that admitted material is *"never re-presented to clap as a positional"*; `--phrase -` satisfies the group, carries nothing, and is inside that ruling — the first draft tightened it to *removed entirely* and boxed itself in. The flag is also declared on all eight material verbs so `--help` documents it | `ms encode --allow-argv-secret --phrase <seed>` exits **0** and emits stdout byte-equal to `ms encode --in <the same phrase in a file>`. **Fails today**: the flag does not exist, so clap rejects it at 64. **Plus the assertion that separates the substitution from a real stdin read, and it is the one a `-`-substituting implementation gets wrong**: the same invocation with **stdin closed** (`0<&-`) must still exit 0 and emit the same bytes — if the material came from stdin rather than the side channel, it cannot. **Plus the assertion the naive implementation fails**: if the admitted token is left in argv for clap, an unrelated later parse error echoes it — so the gate also asserts **no material in stderr for `ms encode --allow-argv-secret --nosuchflag --phrase <seed>`**, where clap must name the flag and never the value. **Plus the two controls**: `ms encode --allow-argv-secret --phrase - < f` behaves exactly as without the override; and `ms encode --allow-argv-secret` **alone**, with no material at all, still exits 64 on the group, so the override cannot be a way to make a required group optional |
| 7 | **THE PURGE TEXT, RUN — NOT PRINTED.** The refusal calls `remedy::history_purge_block`, cited by symbol because the crate is a moving dependency, not because it is unfinished (§3). **The `command` argument is the VERB-QUALIFIED invocation — `ms encode`, not `ms`** — because the recipe is a word-bounded `sed` pattern and a two-character command name is a collision generator: the crate's own doc records that `\bme\b` also removed `cd /home/me` from a six-line sample. **AND THE VERB COMES FROM AN ALLOWLIST — the first draft required the qualification and omitted the mechanism that makes it safe** (round 0's I-6). The guard runs before clap has resolved anything, so the verb is whatever token sits after the binary name, and it is interpolated straight into a shell command the operator is told to run. The donor states the argument at `crates/me-cli/src/main.rs:400`: deriving the words instead *"would admit a TRUNCATED or otherwise unparseable secret into the pattern"*, whereas an allowlist of the tool's own subcommand words cannot carry material at all. `ms`'s allowlist is its **12** command words — `derive`, `encode`, `decode`, `inspect`, `verify`, `vectors`, `gui-schema`, `gen-man`, `repair`, `split`, `combine`, `help` — enumerated from `ms --help`, and `ms` nests no subcommands, so exactly one word is ever appended. **When the token is NOT in the allowlist the pattern falls back to bare `ms` and the emitted text says the match is broad**, because the two failure directions are not symmetric: over-matching costs the operator unrelated history lines, while under-matching leaves a seed in history behind a `sed` that exited 0. No new purge text is written in `ms`; `ms` has none today to supersede | the emitted zsh recipe, **extracted from `ms`'s own stderr and RUN** under a real interactive zsh on a pty, removes the planted entry — with the harness control carried across, so a session that records nothing cannot pass. **Plus the same for bash.** **Plus the MISTYPED-VERB row, which is the one the allowlist exists for**: `ms encoed --phrase <seed>` is still argv carrying a seed, so the guard still refuses; the recipe it emits must still, RUN, remove the planted entry. A recipe built from the typed token gives `sed -i '/\bms encoed\b/d'`, which exits 0 and purges nothing — a remedy reporting success over a seed still in history, the exact shape the crate's `history -d` note calls the trap. **Plus the standing invariant**: `history -d` appears in **no recipe** while still appearing in the warning prose, which is why the recipes are structured data and the naive negative assertion is forbidden. **Plus the collision cost, asserted**: a neighbouring unrelated history line matching the pattern is also removed, measured, so the emitted text must say so — and asserted for the fallback pattern too, where the cost is larger |
| 8 | **THE PRIVATE WRITE — `--out FILE` ON `encode`, `split` AND `repair`.** The three verbs whose stdout CARRIES a canonical artifact — **the first draft said stdout *IS* the artifact on all three, and on `repair` that is false** (round 0's I-5). Re-measured: `ms repair --ms1 <an ms1 with one induced error>` exits **4** and prints `# Repair report`, then `#   ms1 chunk 0: 1 correction at position 1: 'f' -> 'e'`, then the corrected `ms1` — two comment lines and then the artifact. **`--out` on `repair` therefore has to be RULED, and it is: `--out` receives the ARTIFACT LINE ALONE, and the report stays on stdout.** `--out` exists so the next tool can read the file; a payload beginning `# Repair report` is not an `ms1`, and the correction record is what an operator needs to SEE before trusting a repaired card, so it belongs on the stream they are reading, not buried in a file they will feed to an engraver. **`repair` is also the one verb that writes `--out` while exiting non-zero** — 4, VERIFY-ME — and the file is written all the same, because a correction the operator must confirm is still the artifact they asked for; §6a rules `decode`, `verify` and `inspect` report verbs and out of scope, and `combine`/`derive` emit labelled reports rather than artifacts. `--out` writes through the crate's `write_private`, cited by symbol. It **overwrites**, per §6b's explicit ruling | `ms encode --out f` where `f` **already exists at 0644** leaves `f` at **0600** holding the new artifact — the `set_permissions`-on-the-open-file half, which a mode-on-create implementation fails, because `0o600` binds on CREATE only. Asserted with `fd::mode_of` rather than by eyeball. **Fails today**: no `ms` verb has `--out`, and `ms` has never created a file with a mode. **Plus the `repair` assertions, which are what round 0's I-5 showed the `encode`-only gate could not distinguish**: `ms repair --ms1 <the induced-error string> --out f` exits **4**, and `f` holds **exactly the corrected `ms1` and a trailing newline — byte-pinned, no `#` line** — while stdout still carries both report lines. A `--out` that wrote the whole stdout passes a mode-only gate and fails this one. **Plus the collision control**: `ms gen-man --out <dir>` still writes a DIRECTORY of man pages and still exits 0, and `crates/ms-cli/tests/gen_man.rs` is unchanged — the two meanings coexist and F-282 records that they do |
| 9 | **THE UNGROUPED STDOUT — `ms encode` ONLY** (§6a, §6b). stdout carries the canonical `ms1` and nothing else, always ungrouped. `--group-size` and `--separator` move to affecting **the stderr card only**, which §6b states in as many words; the card already exists on `ms encode` and gains the grouped string, which is after this the only place it exists. `--json` is untouched | **the gate is `ms encode --in <a file holding the all-abandon vector>` piped into `me sysw pack` with no flags — exits 0 and writes a payload. The first draft wrote the invocation two different ways and BOTH were unsatisfiable** (round 0's I-4): `ms encode --phrase <the vector>` puts a real BIP-39 phrase on argv, which the guard three entries earlier refuses, and the closure condition's `ms encode` with no flags at all exits **64**, measured — `error: the following required arguments were not provided`, because it names no input channel. **§7's P2 row had it right and this plan drifted from it**: the spec's gate is *"`ms encode --in <file>` piped into `me sysw pack` runs with NO flags and exits 0"*, where *no flags* scopes to `me sysw pack`, not to `ms encode`. **Fails today at exit 4** — *"record 0 … is not a form this container can place"* — measured, with the `--group-size 0` run at exit 0 and 102 bytes as the live control that grouping is the only difference. **Plus the card assertion**: the grouped form appears on **stderr**, and `--no-engraving-card` removes it, which §6c warns is a real change in what a `2>/dev/null` pipeline throws away. **Plus the `--json` control**: `ms encode --json` output is byte-unchanged. `crates/ms-cli/tests/encode_grouping_flags.rs`, `crates/ms-cli/tests/encode_no_engraving_card.rs` and `crates/ms-cli/tests/decode_grouped.rs` are where this lands |
| 10 | **THE WHITESPACE-ONLY SEPARATOR** (§6c). `parse_separator` (`crates/ms-cli/src/format.rs:41`) loses its `hyphen` and `comma` arms and their literal forms, on `encode` and `split` alike — one parser serves both, so it cannot bind to one of them. **INTAKE IS NOT NARROWED**: `is_display_separator` (`crates/ms-cli/src/format.rs:12`) keeps stripping `-` and `,`, because a plate already engraved from a hyphen-grouped card must still decode, and `render_grouped` (`crates/ms-cli/src/format.rs:18`) keeps its `char` parameter | `ms encode --separator hyphen` and `--separator comma` each exit **64**; `--separator space` still exits 0. **All three exit 0 today**, and all three round-trip through `ms decode` at exit 0, measured — which is precisely why the cross-tool argument and not a per-tool one decides it. **Plus the unit test inside `src/`, which the enumerated-diff condition's own scope does not reach** (round 0's M-4): `parse_separator_keyword_and_literal` (`crates/ms-cli/src/format.rs:197`) asserts `parse_separator("hyphen")` yields `-` and `parse_separator("comma")` yields `,`; it is one of the 146 `#[test]`s in `src/`, outside the 276 integration tests, and it must be rewritten to assert the two keywords now REFUSE. **Plus the two controls that keep the blast radius honest**: `ms decode` of a hyphen-grouped and a comma-grouped `ms1` still exits 0, proving intake was not narrowed; and `sha256sum -c display-grouping-vectors.tsv.sha256` still exits 0 with the file untouched, proving the conformance pin was not dragged along — its 5 hyphen and comma rows exercise `render_grouped` directly and never touch the CLI |
| 11 | **THE SIBLING REMEDY BECOMES THE `--in` FORM — AND IT IS A REPAIR, NOT A RETARGET.** In THIS repo, and §6h's standing instruction fires exactly here. **THE ENTRY MOVED TO THIS POSITION, AND THE MOVE IS THE FINDING** (round 0's C-2): it must follow the ungrouped-stdout work, because that is what makes `ms encode`'s stdout packable at all, and the plan's own rule is that no entry begins until the previous is green. At its old position both the old advice and the new advice exit 4. `crates/me-cli/src/main.rs:2164` emits, to a secret-class operator, a line reading `ms encode --phrase - < seed.txt` piped into `me sysw pack --out p.bin`; the comment four lines above at `crates/me-cli/src/main.rs:2160` states `ms encode --in` DOES NOT EXIST (exit 64) and asserts the printed pipeline *is verified to pipe into pack*. **Both statements are false, and the first draft's two anchors into this repo were stale by 24 lines** — it named line numbers 24 further down, which exist, resolve at exit 0 under `plan-cite-check.sh`, and land on unrelated text; F-279's exact shape (round 0's I-7). The stale numbers are described rather than written, because writing them puts two known-wrong citations back into a document the gate will pass; the lines above were re-located by `git grep` on the emitted string. Both change together; the comment is deleted, not amended | **RED-first, and it fails today for the reason the first draft denied.** The retracted claim was *"the current line is correct and must stay correct until `--in` ships"*. **Run verbatim, the emitted line exits 4 and writes nothing**: `ms encode --phrase - < seed.txt` piped into `me sysw pack --out p.bin` → rc **4**, `me: record 0 (records count from 0) is not a form this container can place`, and `p.bin` does not exist. `ms encode`'s default stdout is grouped and `me sysw pack` cannot classify a grouped `ms1` — §1.7 measures exactly this and the first draft contradicted it here. The live control: the same pipeline with `--group-size 0` and `--no-passphrase` exits **0** and writes a **102-byte** payload at mode 0600. **The second retracted claim is the gate's own basis**: *"the assertion `me`'s suite already makes, retargeted"* — there is no such assertion. Re-verified independently: `crates/me-cli/tests/` holds **14** `.rs` files with **33** `Command::new` sites and **0** of them name an `ms` binary; `grep -rn 'ms encode' crates/me-cli/src/` returns the two lines cited above and nothing else; `seed.txt` appears once in the whole crate, inside the emitted literal. **So this entry BUILDS the assertion rather than retargeting one**, and that is its RED: a new test in `me`'s suite that extracts the advised line from `me`'s own stderr, RUNS it, and requires exit 0 and a payload on disk. It is RED against `me` today, RED against a `ms` that has `--in` but still groups its stdout, and GREEN only after the ungrouped-stdout entry. **The cross-repo precondition is part of the gate, not an assumption**: that test needs an `ms` carrying `--in` and ungrouped stdout, which does not exist on any published `mnemonic-secret`, so the test locates its `ms` by an environment variable and **SKIPS explicitly, naming the reason, when it is unset** — never silently, and the phase does not close until it has been run with it set. **The control** is that `me`'s refusal must never advise a channel `ms` lacks — reproduced by running the emitted line, never by reading it, which is the mistake §6h records was made once already and which was made a second time in the text this entry replaces. **`me`'s advice is broken on `master` TODAY and repairing it there is NOT this entry's job** — F-301 carries it with the measurement, because an operator hits it before P2 ships and this entry cannot land until the ungrouped-stdout work does |
| 12 | **THE DRIVERS — 13 MATERIAL INVOCATIONS OUT OF 20, ON 18 LINES.** The material-bearing invocations in the seven scripts of §1.9 move to `--in` or `-`. **The residue is 7, not 5** (round 0's M-3): the first draft subtracted 13 from the LINE count while its own enumeration — 2 `[ -x "$MS" ]` tests, 3 `--version` calls, 2 already using `--phrase -` — summed to 7. Those 7 are left alone and named so a later reader does not "finish" the migration by editing them. `MS` is made overridable in the seven scripts that hard-bind it, following `design/journeys/derive-pathological-keys.sh:39` | **the precondition is part of the gate**: `cargo build --release` in `mnemonic-secret` must produce `target/release/ms`, which **does not exist today**, or nothing here can be run at all. Then: each migrated driver runs to completion at exit 0 against a P2 build, and `design/journeys/derive-rcw-keys.sh` and `design/journeys/derive-hashvault-keys.sh` produce **byte-identical** key output to their committed expectations — the drivers whose output is deterministic and whose only dependency is `ms`. **Fails today in both directions**: unrunnable for want of the binary, and against a P2 build every one of the 13 would exit non-zero on the argv refusal. **Plus the control**: `design/journeys/restore_test_pathological.py` and `design/journeys/restore_test_tr_pathological.py` are NOT edited and still pass, because they already use `--phrase -` |
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
greater force: it is a **pinned external dependency** whose rev this plan may
move forward, so a line number here would be anchored to a rev the executing
implementer is not on. (The first draft gave a different reason — that two of
its modules were being edited that day — and §3 records that both edits had
already landed.)

**AND THE `crates/me-cli/` ANCHORS ARE THE ONES THAT WENT STALE.** The guarantee
above is scoped to `crates/ms-cli/`, and that scope held — all 22 resolved to
what this plan says they hold. The two citations into THIS repo got neither the
symbol treatment nor the anchor, and both were wrong by 24 lines, resolving `ok`
under `plan-cite-check.sh` onto unrelated text (round 0's I-7). They are
re-located by `git grep` on the emitted string, and every `crates/me-cli/`
citation in this document now names what is on the line.

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

**TWO PIECES OF WORK ARE REGRESSION-GATED RATHER THAN RED-FIRST**, and the
column header must not claim otherwise: the pin, whose gate is a build
resolving; and the decline, which is a backstop protecting RED-first work rather
than proving it. **It was three, and the sibling remedy is no longer one of
them** — the first draft called it regression-gated because its current text was
*"correct today"*, and round 0's C-2 ran that text and got exit 4 with no
payload. It is RED-first like the rest. **Everything else is RED-first, and
every one of those gates was RUN and observed to fail**, with the failing exit
code and the failing output quoted in the cell.

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
   1. **And the third shape is reachable privately in two commands and says so
   in the refusal**: a phrase plus a passphrase on `derive`, via
   `ms encode --in seed.txt --out card.ms1` then
   `ms derive --in card.ms1 --passphrase-stdin`, deriving the same fingerprint
   the one-command argv form gives today. `--in` on `derive` reads an `ms1`, so
   this shape is NOT freed by `--in` and the plan claimed otherwise by
   implication until round 0's I-3. The freed stdin builds this; F-303 carries
   the one-command form.
5. **No seed material reaches stderr for any argv carrying it, and every such
   argv is REFUSED** — 4 value spellings on 14 channels, with both the
   space-joined and the `=`-joined form on the 9 flag channels: **92 rows,
   generated**, not the 56 the first draft named, which omitted the `=`-joined
   spelling entirely (round 0's C-1). Measured before any code: **84 of the 92
   pass material at exit 0** and **0 of 92 leak into stderr**, so the exit-code
   half is the live gate; the other **8** already exit 1 silently and therefore
   assert the guard's own refusal text rather than a bare non-zero exit. The
   leak assertion is per whole value and per constituent word of 4+ characters,
   case-insensitively — never per character (round 0's M-6). The guard builds
   this, and the guard's cell names what the 92 does not cover.
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
9. **`me`'s refusal advises only channels that exist AND a pipeline that RUNS,
   verified by running the line it emits — and the test that runs it is BUILT by
   this phase, because none exists.** Re-verified: `crates/me-cli/tests/` holds
   14 `.rs` files with 33 `Command::new` sites, **0** naming an `ms` binary.
   §7's P2 gate requires it, §6h records that the rule was earned by shipping
   the opposite once, and round 0's C-2 found it shipped a second time — the
   advised pipeline exits **4** on `master` today and writes no payload. The
   sibling remedy builds this; F-301 carries the live defect, which lands before
   P2 does.
10. **`ms --out` creates its file 0600 on CREATE and on OVERWRITE**, and
    `ms gen-man --out <dir>` still writes a directory. **And `repair --out`
    holds the corrected `ms1` ALONE, byte-pinned, with the two `#` report lines
    still on stdout** — `ms repair` emits a report and then the artifact, so
    `--out` there had an unspecified meaning and a mode-only assertion could not
    tell the two readings apart (round 0's I-5). The private write builds this.
11. **`ms encode --in <file>`, piped into `me sysw pack` with no flags, exits
    0 and writes a payload.** §7's P2 gate says exactly that, and *no flags*
    scopes to `me sysw pack`. The first draft dropped the channel twice and made
    the condition unsatisfiable: `ms encode` with no flags at all exits **64**,
    measured, because it names no input (round 0's I-4). Today the pipeline
    exits 4 for the grouping reason. The ungrouped stdout builds this.
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
15. **The diff to `ms`'s 276 integration tests AND to the 146 `#[test]`s inside
    `src/` is enumerated, each edit justified by a named §6 ruling or a numbered
    finding, and no test was greened by appending `--allow-argv-secret`.** The
    `src/` half is not decoration: the whitespace-separator work must rewrite
    `parse_separator_keyword_and_literal` (`crates/ms-cli/src/format.rs:197`),
    which the first draft's 276-scoped enumeration did not reach (round 0's
    M-4). The decline builds this.
16. **(assertion, not work — no entry builds it; it is checked, not created)**
    **§6f needs nothing from P2.** `ms`'s measured codes — clap 64, invalid
    artifact 1, repair-uncorrectable 2, repair-applied 4, all four measured in
    §1.1; the first draft asserted the 4 here while §1.1 listed only three, and
    round 0's M-7 found the provenance missing rather than the number wrong —
    are already §6f's
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
- **REWORDING `ms`'s OWN STDOUT ADVISORY.** `ms encode` prints, on every
  invocation, a line telling the operator to redirect to a file — and under the
  default umask 022 that file is **0644**, measured, holding an `ms1` that
  decodes to the seed, while the `--out` this phase adds gives **0600** for the
  same artifact. So after P2 the tool's own standing advice points at the weaker
  of two in-tool channels. This is §6h's rule — remedy text names the channels
  that exist — applied to `ms` instead of the sibling, and P2 does not act on it
  for a stated reason rather than an omission: `fn byte_parity_advisory_lines`
  (`crates/ms-cli/tests/cli_output_class.rs:56`) pins those lines byte-for-byte
  against `mnemonic-toolkit`, so the change is joint cross-repo work under the
  Rust-primary rule. **F-304**, with an owning phase, because round 0's I-8
  found the gap had no row, no condition, no bullet and no follow-up.
- **A ONE-COMMAND PRIVATE FORM FOR `derive` FROM A PHRASE PLUS A PASSPHRASE.**
  §2.5 and the freed-stdin work: the two-command route exists and is asserted,
  the one-command route needs a new channel (a `--passphrase-file`, or a
  phrase-shaped `--in` on `derive`), and §7's P2 row enumerates P2's content and
  includes none. **F-303.**
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

**Ten** entries are added to `design/FOLLOWUPS.md` by this plan — six by the
draft and four by the R0 round-0 fold. Each carries an owning phase, per the
per-phase burndown rule.

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

- **F-301** — **a live defect in `me` on `master` today, not a plan defect.**
  The refusal `me` prints to a secret-class operator advises a pipeline that
  does not run: the emitted line, executed verbatim, exits **4** with
  `me: record 0 (records count from 0) is not a form this container can place`
  and writes no payload, because `ms encode`'s default stdout is grouped and
  `me sysw pack` cannot classify a grouped `ms1`. The same pipeline with
  `--group-size 0` and `--no-passphrase` exits **0** and writes a 102-byte
  payload at 0600. A source comment beside the literal asserts the pipeline is
  verified, and nothing verifies it — 14 test files, 33 `Command::new` sites, 0
  naming an `ms` binary. **This is not the secret-handling class**: the defect
  is that a tool reports a working path it does not have, which is why it still
  gates. **Owning phase: before P2's sibling-remedy entry**, since an operator
  meets it today and that entry cannot land until the ungrouped stdout does.
- **F-302** — `ms`'s argv surface leaks through the `=`-joined spelling, and any
  guard whose gate is built from the space-joined spellings alone will pass its
  own gate while leaking. `ms encode` with the phrase attached to `--phrase` by
  an equals sign, as one argv token, exits **0** and prints the artifact; so do
  the `--hex` and `--passphrase` equivalents. **By the operator ruling of
  2026-08-27 this is the logged class and holds no gate.** It is logged here
  rather than lost, and P2's guard closes it anyway — the normalisation is the
  donor's own, one line, and the cross-product that proves it grew from 56 rows
  to 92. **Owning phase: P2**, with the guard.
- **F-303** — `ms derive` from a phrase plus a passphrase has no one-command
  private form after P2, because `--in` on `derive` reads an `ms1`. The
  two-command route is asserted by the freed-stdin work; a new channel is not
  P2's content. **Owning phase: a later cycle**, alongside the argv work for the
  remaining tools.
- **F-304** — `ms encode`'s unconditional stdout advisory recommends a redirect
  that lands at 0644 under the default umask, while the `--out` P2 adds gives
  0600. Rewording it is cross-repo work because a byte-parity test pins the
  three advisory lines against `mnemonic-toolkit`. **Owning phase: P3**, which
  is where `mnemonic`'s own argv and advisory work already lands, so the two
  sides can be reasoned about together.

Two facts already recorded elsewhere are re-pointed rather than re-filed:
**F-276**, the crate's `me`-shaped boundary, gains a third site in §2.3 — the
`observation` module cannot express `ms`'s existing three-class vocabulary — and
**F-279**, the stale-citation class, is the reason every anchor in this document
names its symbol.
