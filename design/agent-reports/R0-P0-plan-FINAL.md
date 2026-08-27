# R0 FINAL — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` at `4a8df9a`
(worktree `review/p0-final`).
**The report folded:** `design/agent-reports/R0-P0-plan-round11.md` (2C/2I/6M/1N).
**The fold:** `20f1663..4a8df9a`.
**Question:** can one implementer execute this plan end to end without a
reviewer present.
**Reviewer:** fresh agent, no prior context on this artifact, per the dispatch.
**Date:** 2026-08-27. Every measurement below was taken this round against the
binaries rebuilt from `4a8df9a`'s source (`cargo build` reported fresh; suite
re-run: **388 run, 388 passed, 1 skipped** — the plan's own figure), by absolute
path, streams separated.

## VERDICT — **NOT GREEN**

| severity | count |
| --- | --- |
| **Critical** | **1** |
| **Important** | **0** |
| Minor | 3 |
| Nit | 1 |

The fold is honest — both halves verified against the diff and the tree (§Q4
below). The plan's measurable claims are overwhelmingly true — 40+ of them
re-measured this round, two false, both Minor (§Q3). Ten of the twelve rows
carry gates that can fail, most of them RED today and re-confirmed RED by
running them (§Q1). The one finding that blocks is row 9's `-` requirement,
which has **no observable at all** — and worse, the gate row 9 does state is
**green on the untouched tree and goes red only if the work is done**, the
exact shape this plan's own §3 narrative names as what let an earlier
condition be discharged with the guard absent.

---

## CRITICAL

### C-1. Row 9's `-` requirement has no gate: the stated differential is GREEN if `-` is never implemented, goes RED if it is implemented correctly, and passes the silently-lossy implementation §3 bolds as the funds-adjacent hazard. No closure condition backstops it.

**Site:** §4 row 9, gate column; §6 (no condition mentions `-`); §4's M5
paragraph.

**The gate as written:**

> **`-` is IMPLEMENTED**; every code `me` produces today reproduced
> **byte-for-byte**, differentially against the pre-change binary — not by
> matching a table

**Measured today** (`/scratch/code/shibboleth/mnemonic-engrave/target/debug/me`):

```
printf 'text:6162\n' | me sysw pack --out b.bin - text:6869
  → rc 4, "record 0 … is not a form this container can place", no file written
me -                 → rc 2 (clap)      me bundle -      → rc 2 (clap)
me sysw show -       → rc 2 (ENOENT)    me sysw wipe -   → rc 2 (clap)
```

`-` reads stdin nowhere (the plan's core assertion here — TRUE). Now walk the
three implementations an implementer could ship:

1. **Do nothing.** Every code reproduces byte-for-byte. The differential is
   **green**. "`-` is IMPLEMENTED" is a sentence with no test behind it, and no
   §6 condition mentions `-`, so P0 closes green with a spec-§7-P0 deliverable
   (`--in`/`--out`/`-`) absent.
2. **Accept-and-ignore** — the "compliant" reading §6b's permissive wording
   allows and §3 bolds as **silently lossy on the artifact that gets cut into
   metal** (`- text:6869` packs 1 record instead of 2 at exit 0). The dash
   cells diff (4→0), the implementer marks the diff intended — row 9 states no
   rule for which diffs are intended, unlike §3's exit-probe precedent
   ("diffing to three hunks, **all intended**") — and the gate passes. Nothing
   asserts the stdin content reached the container.
3. **Implement it correctly.** The dash cells diff (4→0 with 2 records). The
   gate as literally written — *every* code reproduced byte-for-byte — goes
   **RED**, with no carve-out stated.

A gate that is green when the work is absent and red when the work is done is
anti-aligned with its own step. This plan's history graded exactly this shape
Critical before (§3: *"The old observable was green on the untouched tree,
which made the gate for §6d's ordering unfailable and let condition 8 be
discharged with the guard absent"*), and §4's own honesty paragraph (M5) is
falsified by it: *"Everything else is RED-first"* — row 9's `-` work is not
RED-first; it is not gated at all.

**What closes it, three edits to one row plus one clause:**

1. A RED-today observable with the digit and the content pinned:
   `printf 'text:6162\n' | me sysw pack --out b.bin - text:6869` exits **0**
   and the packed container holds **both** records (assert via `me sysw show
   b.bin` / unpack — `pub_len` moves with the record count; today the command
   exits **4** and writes nothing, measured this round). Fails today,
   satisfiable only by a real implementation.
2. Name the surfaces that gain `-` (at minimum `sysw pack`; say what the other
   four do — refuse, or accept-as-default — so the differential's expectations
   are enumerable).
3. Restate the differential as the exit-probe precedent already does: unchanged
   everywhere **except the enumerated dash cells**, each diff justified.
4. Amend M5's paragraph so the regression-gated list is honest (it currently
   names two pieces; row 9's differential half is a third).

---

## MINOR

### M-1. "All four `eprintln!` in the closure live in them [the `refuse_*` functions]" is false — two of the four do.

**Site:** §3, the N-I1 paragraph. Measured: the closure's four `eprintln!` are
at `main.rs:1035` (`refuse_terminal_destination`), `:1058`
(`refuse_world_readable_stdout`), **`:2039` (`read_records` — the stdin-tty
prompt)** and **`:2091` (`emit` — the write-failure report)**.
`refuse_write_block` contains none. No implementer harm — all four functions
stay in `me` under §3's table, and the moving set carries zero stdio (verified:
0 `eprintln!`, 0 bare `println!` across `destination`,
`stdout_world_readable_mode`+stub, `split_record_stream`, `no_records_guard`,
`write_block`) — but the sentence is a wrong count propping a correct ruling.
One clause fixes it: two live in the `refuse_*` pair, and the other two live in
`read_records` and `emit`, which stay for the same reason.

### M-2. "`-` … five surfaces, four different exit codes" reproduces under no rule I can state.

**Site:** §3, the dash paragraph. Positional `-` on the five plausible surfaces
measures **two** distinct codes (2,2,4,2,2 — above). Widening to `--in -` /
`--out -` shapes adds a 0 (`pack --out -` exits 0, creating a file literally
named `-`) — **three** distinct codes, still not four. The claim's substance
(`-` reads stdin nowhere; behaviour is inconsistent across surfaces) is TRUE
and re-verified; the count is not. State the surfaces and the codes, or drop
the number — this plan deleted its line-count gate for exactly this class of
ordering-dependent figure.

### M-3. F-270 is owned by P0 and appears nowhere in the plan.

**Site:** §4, §6; `design/FOLLOWUPS.md:11914`. The fold filed F-270 (the
donor's post-parse arm normalises for its `tx:` prefix only; ` pass:` and
uppercase `MS1…` refused at rc 4 for the wrong reason — re-measured this round,
both true) with **owning phase P0**, on the argument that the fix is one line
in code P0 already rewrites. Every other P0-owned follow-up has a home in the
plan (F-259 → condition 6, F-264 → condition 9, F-265 → condition 10/row 7,
F-266 → row 6's cross-product). F-270 has no row, no condition, no mention —
the plan's text was frozen before the entry was renumbered. The
reconcile-on-entry rule is the safety net, but the plan is the P0 execution
document; one sentence in row 6's step column ("…and the donor's post-parse
arm gets the same normalisation — F-270") closes it.

---

## NIT

**N-1.** Row 2 creates `crates/me-cli/src/io.rs` holding all five moved
functions; rows 3–9 then speak of `fd.rs`, `observation.rs`, `remedy.rs`,
`exit.rs`, `channel.rs`, and §3's mapped table fixes where each function ends
up — but no step says when `io.rs`'s contents are redistributed into those
module files (row 9b says "move the lib-half **modules**", plural). Executable
either way (the mapped table is unambiguous about the end state), but the
moment of the split is implicit. A parenthetical in row 9b — "distributing
`io.rs` per §3's table" — removes the only navigation gap I hit walking it.

---

## Q1 — THE TWELVE ROWS, WALKED. Can each gate fail?

| row | gate can fail? | verified this round |
| --- | --- | --- |
| 1 | **yes** — the `EXIT_*` count inside `no_records_guard` is **1 today** (`main.rs:1915`, re-measured) and must go to 0; leaves 1 if botched | ✔ |
| 2 | **yes** — the greps alone cannot (stated in-row), the pty digit assertion can; terminal arm exists and is untested by all 12 `world_readable_output.rs` tests | ✔ enums `Destination`/`WriteBlock` private (`:928`/`:953`), `use super::` in tests (`:2165`) — the breakage claim is real |
| 3 | **yes** — `Some(0o620)` is unproducible by a masked implementation (`0o620 & 0o044 == 0`) | ✔ mask inside the donor fn at `:912` |
| 4 | **yes — RED today**: `script -qec 'me sysw wipe --fill zeros'` printed **"this payload is BEARER"** at **rc 2** this round | ✔ live |
| 5 | **yes — RED today** (F-264): the shipped recipe `sed -i '/me sysw pack/d' "$HISTFILE"` cannot remove the in-memory entry; the run-under-real-zsh test fails until fixed | ✔ recipe text read at `:2010-2025` |
| 6 | **yes — RED today on four independent axes**, all re-measured: canonical leak (`me <ms1>` rc 2 token in stderr; `bundle`/`sysw wipe`/`sysw show` same; `sysw pack` rc 3 clean), near-miss leak (uppercase `MS1…` on `bundle` rc 2, token in stderr), ordering (`me --nosuchflag <ms1>` rc 2 via clap today, gate demands rc 3 via guard), override scope (`sysw pack --allow-argv-secret <ms1> --out f` **rc 0** today; `bundle --allow-argv-secret` rc 2 clap today, gate demands rc 3 guard). Positive controls pinned to measured rcs (bundle **2**, both helps **0** — all three re-measured) | ✔ live, all |
| 7 | **yes** — mutation-gated; the blindness it fixes is proven (F-265: suite green at 388/388 today with all five sites returning 2) | ✔ suite re-run |
| 8 | **yes — RED today**: `--expect` does not exist (`sysw pack --help` grep: 0); refusal digits pinned; the `Admission` false-refusal case is a named test | ✔ |
| 9 | **PARTIALLY — the `-` half cannot fail. C-1.** The overwrite assertion and the differential can | ✗ |
| 9b | **yes** — the crate boundary is where E0116/cyclic-dep/`no Class` can actually bite (the plan says so: the enumeration is DISCHARGED here, round-8 M-6) | ✔ |
| 10 | regression-gated and honestly labelled so (M5); the enumerated-diff-with-named-finding requirement is a real check | ✔ |
| 11 | operator-gated, correctly not authorised by the plan | — |

## Q2 — CONDITIONS ↔ ROWS

All eleven discharge: 1→rows 1/10; 2 and 3 are marked "assertion, not work"
and **both hold today, re-measured** (16/16 verbs present on `md`/`mk`/`ms`/`mt`;
`mnemonic inspect` bad-HRP → **2**, `md1`-HRP-fails-decode → **1**, and
`mnemonic decode` → 64 exactly as the plan says); 4→row 8; 5→row 5; 6→row 4
(the gate forces the C4 signature changes — a message derived from the carried
kind cannot be built without them); 7 is process, correctly rowless; 8→row 6's
ordering test; 9→row 5; 10→row 7; 11→this round. In the other direction, rows
2/3/9b serve the §5a/§7 crate deliverable, which §6 never restates — row gates
carry it, and the rows are sequential so it cannot be skipped; row 9's `-`
serves §6b and is the one deliverable with **neither** a condition nor a
failable row gate (C-1).

## Q3 — THE CLAIMS AUDIT (where this plan has failed hardest)

Verified TRUE this round, by measurement, not reading: all §1/§3/§4 line
citations (`record.rs:65/73/89/105`; `main.rs:295-298` four constants,
`EXIT_USAGE=2` at `:296`; mask at `:912`; stub at `:921`; `destination :940`,
`write_block :971`, `split_record_stream :1867`, `no_records_guard :1896`,
`read_records :1921` with its three refs at **1928/2026/2048**; `emit :2053`;
`write_block_decides_both_gates_once :2201`; `--allow-argv-secret` declared at
`:252` on `sysw pack` alone; `Show { file }` at `:275`; donor gate
`trimmed :1952` → `by_prefix :1958` while `classify` gets the RAW token
`:1978`; remedy warning naming `history -d` at `:2017`; the
`sysw_cli.rs:2080` comment verbatim); the 8 `EXIT_*` refs distributed
**2/2/3/1** exactly as §4's two tables state; closure counts (0
`env::args`/`clap`/`Cli`/`process::exit`; 4 `eprintln!`, 0 bare `println!`; 9
IO-touching lines in `read_records`); `Class` = ten variants, the named five
argv-forbidden; `chunk_key` `pub(crate)` at `seal/record.rs:247` with the `Ms`
arm `unreachable!()` at `:282`; `mdmk_unconfirmed` (`record.rs:168`) filters
`Class::MdMk` — blind to `mt1` — and `mt_unconfirmed` at `mt.rs:207`; the
address refusal at **rc 4** with the exact quoted wording (assembled at
render — a source grep alone misses it); `pass:` canonical **rc 3** vs
near-miss ` pass:` **rc 4** (F-270's measurement); quoted mnemonic rc 3,
unquoted twelve tokens rc 4, single word rc 4; usage codes `md` 2 / `mk` 64 /
`ms` 64 / `mt` 2 / `me` 2; `mt`'s guard at `validate.rs:627`
(`allow: bool, form: Form`), mask at `:653`, `:585` a different
`Option`-returning function; toolkit `--passphrase` shipping and
`env::args`/`args_os` = **0** there as in `me`; spec §6d lines 824-841 and
§7 P0's crate assignment quoted faithfully in the DEPARTURE entry.

FALSE, both Minor: the eprintln attribution (M-1) and the four-exit-codes
count (M-2). Nothing false changes what gets built.

## Q4 — IS THE FOLD HONEST? Yes, on both halves.

**Claimed closed, verified closed against `git diff 20f1663..4a8df9a`:**
R11 C-1 — the tree line now reads "stream splitting ONLY; the argv guard and
all of `--expect` stay in `me` (both need `Class`)", and `grep -c` for
`the argv gate` / `kind vocabulary` / `by wordlist` / `P0 builds both` /
`string-level recognisers` / `so P0 fixes the recipe` / `really 11 and not
more` / `the digit-pinning work` all return **0** at `4a8df9a`. R11 C-2 — the
malformed sentence replaced by the CLASSIFICATION ruling; bold delimiters
re-counted: **694, even**. R11 I-1 — the `{canonical, near-miss}` fourth axis
is in row 6's cross-product, its RED-today warrant re-verified live (uppercase
`MS1…` leaks on `bundle` at rc 2), and the false donor warrant replaced by
what `:1952/:1958/:1978` actually do — re-verified against source. R11 I-2 —
the §7 DEPARTURE entry exists, states the crate contributes NOTHING to the
argv/`--expect` work, and F-268's owner is corrected to **P3** in
`FOLLOWUPS.md` with the already-satisfied-trigger argument (re-verified:
`mnemonic restore --passphrase` ships; `flag_is_secret` at `secrets.rs:60-68`).
All six Minors and the Nit: fragment deleted, override scoping + `bundle`
companion row present, the five round-9 carries all grep 0, §2.4 recast to
"the *record* half", the F-266 mechanism paragraph in `FOLLOWUPS.md` rewritten
cleanly (its new `:275` citation and `--in`/`--in=` measurements re-verified),
splice recapitalised.

**Claimed declined, and genuinely all it declined:** the donor's post-parse
gap (filed as F-270 — though see M-3), §2.4's line-wrapped donor-code phrase,
and no §6 condition for the departures (R11 I-2's prescribed closure was three
sentences; all three are done). My propagation sweep found nothing else left.
One process note: the fold-filed F-269/F-270 renumbering is recorded in the
merge commit and the entries are ordered correctly.

## WHAT RELEASES IMPLEMENTATION

C-1 is one row's gate column plus one clause in M5's paragraph — no design
question is open, no probe is needed, and the fix is fully specified above
with its RED-today observable already measured. Fold it, re-run the four
plan gates, and a mechanical re-check of that one row (does row 9 now carry a
`-` observable that fails on the untouched tree, and an intended-diff rule)
closes this. The Minors are three sentences. Nothing else in this plan held a
gate this round.
