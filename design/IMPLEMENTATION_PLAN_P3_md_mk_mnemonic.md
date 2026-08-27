# IMPLEMENTATION PLAN — P3: `md`, `mk` and `mnemonic` adopt the shared surface

**Status:** DRAFT v2, written 2026-08-27, **folded against R0 round 0**
(`design/agent-reports/R0-P3-plan-round0.md`, 0C/4I/6M/3Nit). No code may be
written until this closes an R0 round at 0C/0I.

**Gates this plan is checked by** — run each **separately** from the commit,
never on the same shell line: `scripts/plan-stepref-check.sh` (prose may not
name a step number), `scripts/plan-table-check.sh`, `scripts/plan-cite-check.sh`.

**CITATIONS INTO ALL FOUR SUBJECT REPOS RESOLVE TODAY, AND THAT WAS NOT
ASSUMED.** `plan-cite-check.sh` has carried roots for `descriptor-mnemonic`,
`mnemonic-toolkit` and `mnemonic-key` since 2026-08-18, and **gained a
`mnemonic-gui` root on 2026-08-27 — F-296, filed by this plan's first draft and
closed before this revision.** The same change taught the citation regex to keep
a leading dot and added `.tsv` to the gate's extension list.

**Re-probed after that change**, with the exact forms this document uses —
`crates/md-cli/…`, `crates/mk-cli/…`, `crates/mnemonic-toolkit/…`, the GUI's
`src/schema/…` and `tests/…`, its `pinned-upstream.toml`, and the
display-grouping corpus — **8 of 8 resolved, 0 dangling, 0 ambiguous.** So the
three workarounds the first draft carried are **retired rather than merely
recorded**: the GUI's schema modules were written as prose because the root was
missing and are now cited normally; a hidden top-level directory used to lose
its dot and dangle; and the corpus's extension used to be outside the gate's
list and invisible to it. **Those three shapes are described here and NOT
reproduced in citation form** — writing a retracted form out as a `path:line`
citation is what makes a gate flag the prose explaining it, which has happened
four times in this cycle.

**One form still does NOT resolve, and is handled rather than hoped over:**
`design/FOLLOWUPS.md` exists under **five** roots and reports AMBIGUOUS, so
every `design/…` citation here is repo-qualified. CI paths stay repo-qualified
too, because that is what disambiguates them across eight roots.

**What the gate still cannot do is why every citation below names its symbol.**
It checks that a line **exists**, never what is on it.

**Source spec:** `design/SPEC_constellation_cli_uniformity.md:1330` (P3's row
and gate), `:633` (§6a — which verbs the stdout rule binds), `:660` (§6b — the
channels), `:732` (§6c — the separator rule and the engraving card), `:792`
(§6d — the argv refusal and its pre-parser ordering at `:841`), `:992` (§6f —
exit codes, and the `mk` 2 → 1 ruling at `:1140`), `:1257` (§6h — remedy text
must be executable), `:1665` (§10 — acceptance, and `--from-md1-set` at `:1712`).

**Prior art this plan is downstream of:**
`design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` — the crate.
`design/IMPLEMENTATION_PLAN_P1_mt_adopts.md` — the first adoption, whose shape
this document copies and whose crate-side work this document waits on.

---

## 0. THE SHAPE OF THE PHASE — one plan, three branches, four joins

P3 is the only phase in the cycle whose subject is **more than one repository**,
and the first question is not what to build but whether this is one plan or
three. It is one plan, three parallel implementation branches, and **four**
serialised joins after them. Its one-time prerequisite is **already satisfied**
— see below — so all three branches can start immediately. The evidence, in the
order it settles the question:

**There is no build dependency among the three.** Measured in the manifests:
`mk-cli` takes `md-codec = "0.42.0"` from crates.io
(`mnemonic-key/crates/mk-cli/Cargo.toml:34`); `mnemonic-toolkit` takes
`md-codec = "0.42.0"`, `mk-codec = "0.4.1"` and `ms-codec = "0.7"` the same way
(`mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml:32-34`). Every one of
those is a **codec** crate. P3 edits **CLI** crates only. No edit in this plan
changes a published codec, so no rebuild ordering exists among the three.

**The one thing they share is `mnemonic-io-lib`, and P3 asks NOTHING NEW of
it.** That is the serialisation point the shape question exists to find, and the
answer is a measurement rather than a hope: against the same eleven items P1
weighed, `md` and `mk` adopt `write_private` and nothing else, and `mnemonic`
adopts `remedy::history_purge_block` with its structured half and nothing else.
Both are already inside P1's scope — the private write moves the first in, the
fish recipe is the only edit to the second. **P3 adds zero crate items and
changes zero crate signatures.** Two implementers cannot collide inside a
dependency neither of them edits.

**They shared one PREREQUISITE, it belonged to P1, and IT HAS LANDED.** The
first draft recorded `crates/mnemonic-io-lib` as absent from both distribution
channels, so all three pins waited on one push. **Re-measured in this worktree,
that is no longer true:** `git ls-tree -d origin/master crates/` now names
`crates/me-cli` **and** `crates/mnemonic-io-lib`, at `origin/master` =
`6c24e62`, and that SHA already carries `write_private` and the `remedy` pair
the boundary table adopts. The draft was written at 14:56 and the push landed at
15:00 — the claim was true when written and false four minutes later.

**So the phase has no serialised prerequisite at all, and all three branches
start now.** What survives is the *pin itself* as work, and the hazard it
guarded against — a pin taken before P1's crate-side work was in the pushed SHA
— is **closed, not open**.

**The four real collision points are outside the three implementation
branches.** Two were in the first draft; two are added by round 0, and both are
joins the draft's own entries created without naming.

| join | why it cannot be three parallel commits | owner |
| --- | --- | --- |
| the GUI schema mirror | it is one repo, one branch. P2 edits its `ms` schema file; P3 edits its `md`, `mk` and `mnemonic` schema files. The files are disjoint and the branch is not, and two writers on one branch is what the parallel-isolation rule forbids outright | one agent, after all three branches are green |
| **the toolkit release + pin bump** | the GUI's defaults gate is a **lockstep** gate against a pinned `mnemonic` binary, and the mnemonic branch moves one side of it. The mirror cannot go green until a `mnemonic-toolkit` release carries the flipped default and the GUI's two pins move to it. This runs `GUI mirror ← pin bump ← toolkit release ← the mnemonic branch` — through a fourth repo, and strictly **before** the mirror join | the same agent, and it must precede the mirror |
| **the toolkit's 62 doc transcripts** | they live in `mnemonic-toolkit`, so they look like the mnemonic branch's own work, but they are byte-compared by three workflows `cargo nextest` never runs. The mnemonic branch's two entries invalidate 23 of them, and 19 need **rewriting** rather than regenerating | the mnemonic branch, inside it, before that branch calls itself green |
| the seven journey goldens | the drivers resolve `md`, `mk` **and** `ms` as absolute paths into each repo's own `target/release` (`design/journeys/transcript.sh:9-11`). Regenerating needs all three P3 binaries **and** P2's `ms`, so it is the last thing in the phase and it belongs to one agent | one agent, last, and it waits on P2 |

**The serialisation, stated as an order** so no branch has to infer it:

```
[md branch]  ─┐
[mk branch]  ─┼─→ toolkit release ─→ GUI pin bump + mirror ─→ journey goldens
[mnemonic branch, INCLUDING its 23 doc transcripts] ─┘            (waits on P2)
```

Only the three bracketed branches are parallel. Everything right of them is
sequential, and the doc transcripts are **inside** the mnemonic branch rather
than after it — a branch that ships its two entries without them has left a
red CI surface behind that its own definition of green cannot see.

**The one ordering that looked real dissolves under measurement.** §10's
acceptance runs `md encode --out wallet.md1` and then
`mk encode --from-md1-set wallet.md1`, which reads what the first wrote — so
`md`'s channel work looks like a prerequisite for `mk`'s new flag. It is not.
Repeated `--from-md1` over a real four-chunk `md1` set was run both ways today:
fed the **grouped** default output and fed the **ungrouped** output, `mk encode`
produced **byte-identical** `mk1` cards at exit 0, because `mk` strips display
separators on intake (`crates/mk-cli/src/format.rs:33`). So `--from-md1-set`
need only skip lines that are not `md1`, which makes it tolerant of today's
header and of tomorrow's absence of one, and it can be built and gated before
`md` changes at all.

**RECOMMENDATION: one plan, three branches, run in parallel — and R0 round 0
concurred, with conditions that are now folded in.** Three plans would
triplicate the boundary section, the dependency section and the closure list
over three documents that share one spec row, one crate pin, one toolkit
release, one GUI mirror and one golden set — and would leave the joins owned by
nobody, which is the defect §7 of the spec caught twice already.

**The conditions were exactly the two joins the first draft did not name**: the
toolkit release the GUI mirror depends on, and the doc transcripts the mnemonic
branch owns. **Both are now entries with owners and gates**, which is what makes
the parallel recommendation safe rather than merely attractive. The
no-build-dependency argument above was independently re-verified and stands
unchanged.

**WHAT WOULD CHANGE THE RECOMMENDATION**, stated so a reviewer can attack it:

1. **If `mnemonic`'s argv refusal needed a scanner in the crate.** It does not,
   measured: §7 rules that `mnemonic` keys off its existing
   `NodeType::is_argv_secret_bearing`
   (`crates/mnemonic-toolkit/src/cmd/convert.rs:117`), and the token set that
   predicate mirrors is `secret_taxonomy::SECRET_NODE_TYPES_ARGV`
   (`crates/mnemonic-toolkit/src/secret_taxonomy.rs:95`) — a
   `pub const &[&str]` of nine strings, usable in a raw-argv scan with no clap
   involved. If a reviewer rules the scanner belongs in the shared crate
   instead, all three serialise behind one crate change and the answer becomes
   three sequential phases.
2. **If the operator ruled that `md` and `mk` get the argv refusal too.** §4
   currently exempts them in as many words — *"Watch-only material stays on
   argv"* — and that exemption is the only reason `md` and `mk` need none of
   `remedy`. Reverse it and all three tools want the same crate text at the same
   time.
3. **If the world-readable write gate were ruled into P3.** It is not in P3's
   row; if it were, all three would want `exit::write_block` and the crate
   becomes shared mutable ground.

---

## 1. THE INVENTORY — measured, not described

Every figure below was produced on 2026-08-27 by running the binaries at
absolute paths after a `cargo build --locked` in each tree. **A bare `md` in a
measurement is worthless here**: the login shell aliases it to `mkdir -p`, which
exits 0 and silently creates directories. Exit codes were captured to files and
grepped, never read through a pipe.

| repo | HEAD | tree | binary |
| --- | --- | --- | --- |
| `descriptor-mnemonic` | `beb2fb2` | clean, 0 ahead | `md 0.13.0` |
| `mnemonic-key` | `c5739fc` | 0 ahead, 1 untracked design doc | `mk 0.13.0` |
| `mnemonic-toolkit` | `8342b2e` | 0 ahead, 38 untracked files at the root | `mnemonic 0.97.0` |

The toolkit's 38 untracked files are cycle-prep leftovers (`cycle-prep-recon-*`,
unstaged `design/` drafts). They were read, not deleted, and none is in P3's
path. One is named `design/P3_RECON_codec_repos.md` and is **not about this P3**
— it is a 2026-06-24 recon for replicating reproducible musl builds. A future
reader searching for "P3" in that repo will find it first.

**The suites are green and the trees are formatted, in all three.** Measured
once each, captured to a file:

| repo | `cargo nextest run --locked` | `cargo fmt --all --check` | `cargo clippy --workspace --all-targets -- -D warnings` |
| --- | --- | --- | --- |
| `descriptor-mnemonic` | 805 passed, 2 skipped | exit 0 | exit 0 |
| `mnemonic-key` | 337 passed, 0 skipped | exit 0 | exit 0 |
| `mnemonic-toolkit` | 3960 passed, 20 skipped | exit 0 | exit 0 |

**This is a difference from P1 and it removes a whole entry.** P1's first piece
of work is a tree-greening, because `mt`'s `fmt` was red with 235 diff lines and
CI's fail-fast had hidden a second red gate behind it. Nothing here needs that.
The 22 skipped tests are not investigated by this plan and are not claimed to be
harmless; they are recorded so a later reader does not read "green" as "every
test ran".

### 1.1 `md`'s surface

**The header.** One emission site on stdout —
`crates/md-cli/src/cmd/encode.rs:172`, `println!("chunk-set-id: 0x{csid:05x}")`,
inside the chunking arm. Measured: `md encode 'wpkh(@0/<0;1>/*)'` prints no
header; the same with `--force-chunked` prints one. A **second** site writes the
same text into a fixture file rather than to stdout —
`crates/md-cli/src/cmd/vectors.rs:76` — and §6a does not reach it.

**The grouping.** Default `--group-size 5`, separator `space`, applied at both
emission branches through `render_grouped`. `parse_separator`
(`crates/md-cli/src/main.rs:54`) accepts `space|hyphen|comma` or the literal
char; the clap declaration is `crates/md-cli/src/main.rs:115`.

**The card.** There is none. `md encode`'s stderr on success is exactly **one**
line, `note: stdout is a keyless descriptor template (no keys)`, and it is the
same whether the output chunks or not.

**The channels, and a collision the first draft's negative was too wide to
see.** `--in` exists on **no** `md` subcommand at all. `--out` does **not**
exist on the five verbs P3 touches — a sweep over `encode`, `decode`, `verify`,
`inspect` and `repair` counts **0** — but it is **not** absent from the binary:
enumerated from `md gui-schema`, **`md vectors --out` and `md gen-man --out`
already exist**, and both mean a **directory** (`--out <DIR>`, *"Directory to
write `*.1` man pages into"*). The md channels entry adds `--out FILE`, meaning
*a file created 0600*, to the same binary. Nothing here is wrong — the per-entry
gates are scoped to `encode` — but **two meanings of `--out` will share one
binary**, an implementer must not "tidy" them together, and the same is true of
`mk` below. Filed.

**`-` is worse than absent on FOUR verbs, and this is the measurement that
changes the work.** `md repair -` reads stdin and exits 0. On `decode`,
`inspect`, `bytecode` **and** `verify`, `-` is accepted as a **literal
positional value** and fails with `codex32 decode error: string does not start
with HRP md1`. It is not clap's `unexpected argument` at exit 2, which is what
`mt` did before P1. **A gate written as "the command fails today" would pass
before and after the fix.** The gate has to be equality with the piped-content
run.

**The exit codes those four produce are NOT uniform, and a gate written from a
single baseline will be written against the wrong error.** Measured with a real
card on stdin:

| invocation | exit | why |
| --- | --- | --- |
| `md decode -` | 1 | `-` taken as a literal md1 string |
| `md inspect -` | 1 | same |
| `md bytecode -` | 1 | same — **the fourth verb, and the first draft's "three" undercounted it** |
| `md verify -` | **2** | clap, *"the following required arguments were not provided: `--template`"* — the dash defect is never reached |
| `md verify - --template 'wpkh(@0/<0;1>/*)'` | 1 | with the template supplied, the dash defect surfaces |

**Bare `md verify -` is 2, not 1.** The first draft grouped `verify` with
`decode` and `inspect` at exit 1; that is only true once `--template` is
supplied. The equality gate is still the right gate — the plan's core insight
holds — but `verify`'s gate must supply `--template`, or it measures clap
instead of the reader.

**The reader is private and has one caller.** `read_md1_strings`
(`crates/md-cli/src/cmd/repair.rs:73`) handles `-`, strips display separators
per line and drops lines that strip to empty. It is `fn`, not `pub fn`, and its
only caller is `crates/md-cli/src/cmd/repair.rs:112`. The other four verbs each
declare their own `strings: Vec<String>` positional
(`crates/md-cli/src/main.rs:146`, `:155`, `:187`, `:196`) and reach the material
through `strip_md1_inputs` (`crates/md-cli/src/cmd/mod.rs:5`), a pure mapper
with no stdin path at all.

**The exit codes.** `crates/md-cli/src/main.rs:348` returns 2 for
`CliError::BadArg` and `crates/md-cli/src/main.rs:352` returns 1 for everything
else. Measured: `md decode <garbage>` → 1, `md repair <BCH-uncorrectable>` → 2,
`md encode` with no template → 2. `md repair`'s 2 is deliberate and its own
source says why at `crates/md-cli/src/cmd/repair.rs:109`: it returns `Ok(2)` at
`crates/md-cli/src/cmd/repair.rs:124`, *"bypassing the `CliError::Codec → 1`
default route so the repair exit-code contract is honored."* **That sentence is
the mechanism `mk` is missing, and it is why the `mk` exit-code work below is
not a one-line edit.**

### 1.2 `mk`'s surface

**The header.** None, confirmed rather than inherited: `mk encode` with a real
xpub, origin and stub prints two `mk1` lines and nothing else, and there is no
`chunk-set-id` `println!` anywhere in `mk-cli`.

**BUT `mk encode` DOES put a non-artifact line on stdout, and §2 of the spec
says it never does.** `crates/mk-cli/src/cmd/encode.rs:339` prints a **blank
line between cards** on the `--keys` path. Measured with a two-record key file:
stdout carries **exactly one blank line** between the two cards. **The total
line count is fixture-dependent and is not a fact about the defect** — records
chunk to different lengths, so `mk-cli`'s own `KEYS` fixture gives 6 lines with
1 blank while a two-record file whose cards are both 2 chunks gives 5. The
blank-line count is what reproduces invariantly, and it is what the gate below
asserts. §2's cell reads *"non-artifact lines
on stdout: none, ever"* for `mk`; that was measured on a single-card run, where
the branch cannot fire. `me sysw pack` tolerates it — blank lines are skipped by
`records::split_record_stream` — so this is a §4 violation and not a
packability defect, and §10's acceptance runs exactly the `--keys` path that
produces it.

**The grouping.** Default `--group-size 5`, separator `space`
(`crates/mk-cli/src/cmd/encode.rs:78`, `:81`). `parse_separator` lives in
`crates/mk-cli/src/format.rs:40` — a different module from `md`'s, doing the
same job with the same three keywords. One emission site,
`crates/mk-cli/src/cmd/encode.rs:344`.

**The card.** There is none. One stderr line on success:
`note: stdout is watch-only — public keys only, cannot spend`.

**The channels, with the same `--out` collision as `md`.** `--in` on no
subcommand; `--out` on none of the five verbs P3 touches, same sweep, same
**0** — but `mk vectors --out` and `mk gen-man --out` already exist on the
binary, both meaning a **directory**. `--keys FILE` exists on `encode` alone and
is read by `keyfile::read_key_records` (`crates/mk-cli/src/keyfile.rs:99`),
which takes a path.

**`-` already works, and the reader is already shared.** Measured with a real
two-chunk card on stdin: `mk decode -`, `mk inspect -`, `mk verify -` and
`mk repair -` all exit 0 and read it. `mk encode -` exits 64 with
`unexpected argument '-' found`, which is correct — `encode` has no artifact
positional. The reader is `read_mk1_strings`
(`crates/mk-cli/src/cmd/mod.rs:207`), **`pub`, in `cmd/mod.rs`, with five
callers**: `address`, `decode`, `derive`, `inspect` and `repair`. Its `md`
counterpart's doc comment names this parity in as many words. **So `mk`'s
channel work is one function and `md`'s is a hoist plus four call sites** — the
mirror image of P1's finding that `mt`'s shared `ReadArgs` was its largest
available simplification.

**The exit codes, and the thing the spec's ruling does not say.**
`CliError::exit_code` (`crates/mk-cli/src/error.rs:108`) maps
`Codec(UnsupportedVersion)` to 3, `Codec(_) | MdCodec(_)` to **2**,
`FutureFormat` to 3, `ContentMismatch` to 4, `UsageError` to 64, `IoError` to
**1**, and `SetReassemblyMismatch` to **2**.

**Two different §6f table columns come out of the same match arm.** Measured:
`mk decode <garbage>` → 2 and `mk repair <BCH-uncorrectable>` → 2, and both are
`Codec(_)`. §6f's table gives `md`, `ms` and `mnemonic` a **repair-uncorrectable
of 2** and an **invalid artifact of 1**; `md` gets that split by the explicit
`Ok(2)` bypass cited above. **`mk` has no such split, so the ruling as written —
"`mk`'s invalid-artifact 2 becomes 1" — also moves `mk`'s repair-uncorrectable
to 1 and breaks a parity three other CLIs hold.** That is filed and built here.

**HOW WIDE THE BYPASS MUST BE — RULED HERE, BECAUSE THE FIRST DRAFT LEFT THE
DECISION OUT AND ITS GATE COULD NOT OBSERVE IT.** The draft prescribed *"return
`Ok(2)` on an **uncorrectable** input"*. **`md`, the tool this shape is
transplanted from, does not do that.** Read at the source rather than from the
draft's summary of it, `md`'s bypass matches on **any** error out of
`decode_with_correction` — `crates/md-cli/src/cmd/repair.rs:124` sits in a bare
`Err(e) =>` arm, and the function's own doc comment at
`crates/md-cli/src/cmd/repair.rs:107` says *"On atomic-fail (any md_codec"* —
continuing on the next line *"error from `decode_with_correction`)"*.
"Uncorrectable" is narrower than what `md` ships.

**The two readings are distinguishable, and today's four-cell measurement
decides between them:**

| invocation | `md` today | `mk` today |
| --- | --- | --- |
| `repair <HRP swapped to ms1>` | **2** | **2** (pinned by `crates/mk-cli/tests/cli_repair.rs:172`) |
| `repair <BCH-uncorrectable>` | 2 | 2 |
| `decode <HRP swapped to ms1>` | **1** | 2 |
| `decode <garbage>` | 1 | 2 |

`md` already splits by **verb**, not by error kind: `repair` is 2 for every
codec error, everything else is 1. A bypass scoped to the BCH-uncorrectable
variant would send `mk repair <HRP-swapped>` to **1** and red
`repair_hrp_mismatch_exits_2` — a test the first draft asserted does not exist.

**RULING: `mk repair` adopts `md`'s bypass at its true width — any codec error
out of the correcting decode returns `Ok(2)`.** This is the reading that
preserves the cross-CLI parity §6f is protecting, and it is the reading `md`,
`ms` and `mnemonic` already ship.

**Verified by building it, not by arguing it.** On the same scratch copy: the
arm at `crates/mk-cli/src/error.rs:111` moved to `1`, **plus** the
correcting-decode call at `crates/mk-cli/src/cmd/repair.rs:113` wrapped so any
`Err` prints to stderr and returns `Ok(2)`. Result — `cargo nextest run --locked
--no-fail-fast`: **337 tests, 337 passed, 0 failed**, and the four cells become
`repair` 2 / 2 and `decode` 1 / 1, matching `md` column-for-column.

**AND THE CHANGE FALSIFIES A COMMENT NOTHING IN THE DIFF TOUCHES.**
`crates/mk-cli/src/cmd/repair.rs:112` currently reads *"route to exit 2 via
`CliError::Codec(_) => 2` in error.rs"*. After this entry that arm returns 1 and
`repair`'s 2 comes from the new bypass instead, so the comment describes a
mechanism that no longer exists — the same class as the two journey-driver
comments the goldens entry deletes. It is part of the work, not a tidy-up.

**THE EXIT-CODE CENSUS, RE-DERIVED — AND THE FIRST DRAFT'S VERSION WAS WRONG IN
THE ONE DIRECTION THAT MATTERS.** The draft searched for `.code(N)` alone, found
12 sites in `crates/mk-cli/tests`, and concluded *"zero tests pin `mk`'s
invalid-artifact 2 by exit code"* and *"the change would ship unnoticed by the
suite in either direction"*. **Both statements are false.** A `.code(N)` grep
sees one of **three** assertion families this suite uses.

Re-counted by parsing every tracked `.rs` file under each test directory, not by
one regex:

| family | `md` (`crates/md-cli/tests`, 39 files) | `mk` (`crates/mk-cli/tests`, 17 files) |
| --- | --- | --- |
| `assert_cmd` `.code(N)` | 25 — `{0:3, 1:13, 2:9}` | 12 — `{0:4, 2:4, 5:3, 64:1}` |
| `assert_eq!(out.status.code(), Some(N))` | 18 — `{0:6, 2:2, 4:8, 5:2}` | 6 — `{0:3, 4:1, 5:1, 64:1}` |
| `assert_eq!(code, N)` over a bound `let code` | 14 — `{0:8, 1:1, 2:2, 5:3}` | 16 — `{0:6, 2:2, 4:1, 5:4, 64:3}` |
| **total** | **57** — `{0:17, 1:14, 2:13, 4:8, 5:5}` | **34** — `{0:13, 2:6, 4:2, 5:8, 64:5}` |

**So `mk` has SIX sites asserting exit 2, not four.** The four in
`crates/mk-cli/tests/cli_mk1_repair_reverify.rs` (`:178`, `:194`, `:239`,
`:259`) do assert `SetReassemblyMismatch`
(`crates/mk-cli/src/cmd/repair.rs:380`), the funds fix, exactly as the draft
says. The **two the draft missed** are the third family, in
`crates/mk-cli/tests/cli_repair.rs`, and they pin **`CliError::Codec(_)`** — the
arm the exit-code entry moves:

- `repair_beyond_t4_capacity_exits_2` (`crates/mk-cli/tests/cli_repair.rs:140`),
  asserting at `:152` — *"5+ substitutions exceed t=4 capacity → exit 2"*.
- `repair_hrp_mismatch_exits_2` (`crates/mk-cli/tests/cli_repair.rs:172`),
  asserting at `:185` — *"HRP mismatch → exit 2 (`CliError::Codec::InvalidHrp`)"*.

**Verified by mutation rather than by reading.** The one-line edit the draft
describes — `crates/mk-cli/src/error.rs:111`, `=> 2` becoming `=> 1` — was
applied to a scratch copy and the suite run with `--no-fail-fast`:
**337 tests, 335 passed, 2 failed**, and the two failures are exactly the two
tests above. The four funds-safety tests stayed **green**, which independently
confirms that `SetReassemblyMismatch` is unreachable from this arm.

Two consequences, and the second is the reason the exit-code entry is not a
one-line edit:

1. **The change does NOT ship unnoticed.** The suite catches it — but it catches
   it by reddening tests the first draft told the implementer do not exist.
2. **`repair_hrp_mismatch_exits_2` pins an INVALID ARTIFACT at 2 on `repair`**,
   not a repair-uncorrectable. It is the case that decides how wide the bypass
   must be, and the first draft's row contained neither the case nor the
   decision.

### 1.3 `mnemonic`'s surface

**The grouping surface is four subcommands, not one.** Enumerated from the
binary's own `gui-schema` JSON rather than by reading help text — **9 flags
across 4 subcommands**:

| subcommand | flags carried |
| --- | --- |
| `bundle` | `--group-size` (default 5), `--separator` (default `space`), `--no-engraving-card` |
| `convert` | `--group-size` (default 5), `--separator` (default `space`) |
| `ms-shares-split` | `--group-size` (default 5), `--separator` (default `space`) |
| `ms-shares-combine` | `--group-size` (default 5), `--separator` (default `space`) |

§2a of the spec names only `bundle`. The three other carriers are declared at
`crates/mnemonic-toolkit/src/cmd/bundle.rs:82`,
`crates/mnemonic-toolkit/src/cmd/convert.rs:350`, and
`crates/mnemonic-toolkit/src/cmd/ms_shares.rs:76` and `:118`. All four route
through one pair of functions —
`display_grouping::render_grouped`
(`crates/mnemonic-toolkit/src/display_grouping.rs:20`) and
`display_grouping::parse_separator` (`:45`) — so the separator narrowing is a
single-function edit and the default flip is four.

**THE FIVE ARGV CHANNELS ARE ALL REAL, AND ONE IS SPELLED DIFFERENTLY THAN THIS
PLAN'S BRIEF SAID.** Each was run and its stderr grepped for the advisory:

| channel, as invoked | advisory emitted | exit |
| --- | --- | --- |
| `bundle --slot @0.phrase=<phrase> --passphrase <pw>` | **2** — one per channel | 0 |
| `convert --passphrase <pw>` | 1 | 0 |
| `derive-child --passphrase <pw>` | 1 | 2 |
| `restore --passphrase <pw>` | 1 | 0 |
| `electrum-decrypt --decrypt-password <pw>` | 1 | 1 |

The flag is **`--decrypt-password`**, not `--decrypt-passphrase`. The spec has
it right at `design/SPEC_constellation_cli_uniformity.md:1332`; the brief that
commissioned this plan had it wrong. `bundle` also has no `--from`: its secret
channel is `--slot @N.<subkey>=<value>`, which is why it emits two advisories
where the others emit one. Three of the five exit non-zero for unrelated
reasons in the runs above (a missing `--template`, an inapplicable `--length`, a
deliberately invalid ciphertext); the advisory fires regardless, which is the
fact being measured.

**Today every one of them WARNS AND PROCEEDS.** `convert --from
phrase=<the all-abandon vector> --to xpub` prints
`warning: secret material on argv (--from phrase=) — pipe via --from phrase=- to
avoid /proc/$PID/cmdline exposure` and carries on. The warning names the flag
and never the value, so the leak §6d cares about is the process table, not the
message.

**The boundary is the predicate, and the predicate is much wider than five.**
Re-run from the spec's own two commands, scoped by `git ls-files`: **26 files,
86 references** to `secret_in_argv_warning` — reproducing the spec's figures
exactly. Narrowed to source, **21 files** carry it: `secret_advisory.rs` holds
the definition, and the other **20 — nineteen `cmd/` modules plus `repair.rs` —
hold 48 call sites**. The distinct argv-material shapes those call sites name are
**eleven**, not five: `--from <node>=`, `--slot @N.phrase=`, `--share <node>=`,
`--passphrase`, `--bip38-passphrase`, `--decrypt-password`, `--phrase`, `--ms1`,
`--secret`, `--digits`, and a bare **positional `ms1`**. §7 already rules that
*"the five named channels are ASSERTIONS in P3's gate, not the sweep
boundary"*; this is what the boundary measures to.

**The predicate is usable before clap, which is what makes §6d's ordering
achievable.** `NodeType::is_argv_secret_bearing`
(`crates/mnemonic-toolkit/src/cmd/convert.rs:117`) is
`is_secret_bearing() || MiniKey`, and the token set it is kept in lockstep with
is `SECRET_NODE_TYPES_ARGV`
(`crates/mnemonic-toolkit/src/secret_taxonomy.rs:95`) — nine `&'static str`
tokens: `phrase`, `entropy`, `xprv`, `wif`, `ms1`, `bip38`, `electrum-phrase`,
`seedqr`, `minikey`. Matching `--from` in raw argv and splitting at `=` is
string work. **No parse is required to reach the decision**, which is the whole
of C-4.

**Where the advisory is emitted today is not where the refusal must be.**
`emit_secret_in_argv_advisories` (`crates/mnemonic-toolkit/src/cmd/convert.rs:1844`)
takes an already-parsed `&ConvertArgs`. Every one of the 48 sites is
post-`clap`. §6d rules that ordering normative, so the refusal is new code at a
new place, keyed off an old predicate — not a rewrite of the advisory layer.

**`mnemonic bundle`'s stdout carries six non-artifact lines out of twelve** —
three `# ms1 (entropy, …)` / `# mk1 (xpub + origin)` / `# md1 (wallet policy)`
comments and three blank separators, all six measured. **They stay.** See the
boundary section; the question was consulted and ruled, not assumed.

**AND `mnemonic` CARRIES A FOURTH VALIDATION SURFACE THE FIRST DRAFT NEVER
NAMED: 62 COMMITTED DOC TRANSCRIPTS, BYTE-COMPARED IN CI, THAT `cargo nextest`
NEVER RUNS.** `mnemonic-toolkit/docs/manual/transcripts/` holds **169 tracked
files, of which 62 are `.cmd`** — command scripts replayed against the real
installed binaries, with stdout and stderr byte-compared against golden `.out`
and `.err` files. Three workflows do the replaying —
`mnemonic-toolkit/.github/workflows/quickstart.yml`, `manual-gui.yml` and
`technical-manual.yml` — and **none of them is `cargo nextest`**. The first
draft's closure list named only the runner-shaped surfaces, so it could not see
any of these; the closure list below now names this one as a fourth surface in
its own right.
`mnemonic-toolkit/.github/workflows/quickstart.yml:105` describes the mechanism
in its own words: it *"byte-compares against the golden .out"*.

**Two entries below invalidate 23 of these goldens, and the two halves need
DIFFERENT remedies.**

- **The argv refusal invalidates 19, and they cannot be regenerated.** Measured:
  `git grep -l 'secret material on argv' -- docs/manual/transcripts` returns
  **19** files, including the manual's and the QuickStart's flagship bundle
  examples — `22-first-bundle.out`, `23-verify.out`, `24-recover.out`,
  `41-inheritance.out`, `qs-23-bundle.out`, `qs-24-verify.out`,
  `qs-26-recover-phrase.out`, `qs-41-watch-only-xpub.out`, the four
  `41-seedqr-*` files, the three `41-inspect`/`41-repair` pairs, the two
  `41-bundle-inheritance-*` files, and
  `cross-format-recipes/recipe-2-bitcoin-core-to-bundle.err`. Every one is a
  worked example whose `.cmd` puts secret material on argv and whose golden
  records the tool **warning and proceeding**. The refusal entry makes those
  commands **refuse**, so re-running the generator produces a refusal where the
  document explains a result. **The `.cmd` files must be REWRITTEN** — to the
  stdin channel, or to `--allow-argv-secret` — **and the surrounding prose
  updated.** That is authorship, not regeneration, and it is the larger half of
  the mnemonic branch's real scope.
- **The grouping default flip invalidates 4, and those DO just regenerate.**
  Detected by matching grouped artifact lines in every tracked golden:
  `22-first-bundle.out`, `41-bundle-inheritance-cards.out`,
  `cross-format-recipes/recipe-2-bitcoin-core-to-bundle.out` and
  `qs-23-bundle.out` pin space-5 grouped output — e.g.
  `ms10e ntrsq qqqqq qqqqq …` — which the flip makes one unbroken string.

*(Bounded rather than open-ended, so the mnemonic branch knows where this stops:
the `.cmd` transcripts that invoke `$MD_BIN`/`$MK_BIN` are all
`decode`/`repair`/`inspect`/`address`/`derive`, which §6a puts out of scope, and
the toolkit installs `md`/`mk`/`ms` from version tags rather than from a
sibling worktree. **The `md` and `mk` branches cannot red this surface. The
exposure is the mnemonic branch's alone**, which is why the shape section puts
it inside that branch rather than after all three.)*

**This is the failure mode the rest of the plan is careful about, found in the
one repo the plan does own.** The GUI-mirror entry exists precisely because a
green suite can be evidence about nothing; the seven journey goldens are
enumerated precisely because joins owned by nobody are what §7 of the spec
caught twice. These 62 goldens are the same class of artifact and the first
draft did not mention them in any entry, gate, closure condition or follow-up.

### 1.4 The surfaces outside the three repos

**The GUI mirror, re-measured and matching §2a exactly.** Four
`const SEPARATORS: &[&str] = &["space", "hyphen", "comma"];` declarations, one
each in the GUI's `md`, `mk`, `ms` and `mnemonic` schema modules; eight
`default_value: Some("5")` sites across the same four files, distributed 1 / 1 /
2 / 4.

**Its drift gate is blind to the SEPARATOR change and to everything on `md` and
`mk` — but it is NOT blind to the `mnemonic` default flip, and the first draft
said it was.** Three reasons were offered; two hold and the third is false.

**Reason 1 — scope. TRUE.** `tests/schema_mirror_defaults_drift.rs:29` scopes
the gate to *"`mnemonic` only"* and calls extending it to `md`/`ms`/`mk` a
deliberate omission. The `md`/`mk` half is additionally a **one-sided** guard
that arms only once those CLIs emit a non-null `default_value`. Measured:
`md gui-schema` and `mk gui-schema` are `version: 1` and emit
`default_value: null` **and** `choices: null` for both `--group-size` and
`--separator`. The `md` and `mk` mirror edits really are invisible.

**Reason 2 — choices. TRUE.** `tests/schema_mirror_defaults_drift.rs:23` scopes
the choices comparison to *"flags whose pinned JSON carries NON-NULL
`choices`"*, and `mnemonic gui-schema` reports `choices = null` for
`--separator` at all four carrying subcommands. Deleting `hyphen` and `comma`
from a `SEPARATORS` constant is compared against nothing, on any of the four
CLIs.

**Reason 3 — "a pin far behind the CLI's current version". FALSE, and the
defaults half of the gate is LIVE.** That claim rests on a **stale in-file
comment** — `tests/schema_mirror_defaults_drift.rs:36` still says *"the pinned
v0.75.0 binary"*. The actual pins say otherwise, in two places:
`pinned-upstream.toml:22` carries `tag = "mnemonic-toolkit-v0.97.0"`, and the
load-bearing dependency pin at `mnemonic-gui/Cargo.toml:76` is the same tag. The
measured toolkit is `mnemonic 0.97.0`. **The pin is exactly current.**

And the `default_value` comparison is not scoped away for `mnemonic`:
`tests/schema_mirror_defaults_drift.rs:268` compares the hand mirror against the
pinned JSON for every flag not in `DEFAULT_VALUE_ALLOWLIST`, and that allowlist
(`tests/schema_mirror_defaults_drift.rs:48`) has exactly **one** entry,
`("compare-cost", "--feerate")`. `--group-size` is not in it, and the file says
outright that if the list grows *"the mirror or the toolkit has real drift to
reconcile, not to allowlist."*

**Measured by RUNNING it, not by reading it.** On a scratch copy of the GUI with
the four `--group-size` defaults in `src/schema/mnemonic.rs` flipped from `"5"`
to `"0"` — the exact edit the GUI-mirror entry makes — and `MNEMONIC_BIN`
pointing at the pinned-version binary:

```
test md_ms_mk_choices_and_defaults_match_pinned_gui_schema ... ok
test mnemonic_defaults_and_choices_match_pinned_gui_schema ... FAILED
default_value drift between the hand mirror and the pinned gui-schema:
  bundle --group-size :: mirror=Some("0") gui-schema=Some("5")
  convert --group-size :: mirror=Some("0") gui-schema=Some("5")
  ms-shares-split --group-size :: mirror=Some("0") gui-schema=Some("5")
  ms-shares-combine --group-size :: mirror=Some("0") gui-schema=Some("5")
```

**Four violations, and the run reds.** The `md`/`ms`/`mk` test stayed green,
confirming reasons 1 and 2 in the same execution.

**So "flipping the default produces zero GUI test failures" is false, and the
consequence is structural rather than cosmetic.** This is a **lockstep** gate:
it fails if either side moves alone. The GUI mirror therefore cannot be a
two-line edit at the end of the phase — it needs a **`mnemonic-toolkit` release**
carrying the flipped default and a **bump of both toolkit pins** before it can
be green. That is the join the shape section now names, and it appears in no
entry of the first draft.

**The seven goldens, re-derived from the spec's own command.**
`git ls-files design | xargs grep -l 'chunk-set-id:' | wc -l` counts every design
document that so much as *mentions* the string, so it rises every time one is
written — the spec records 28 / 29 / 30 at three earlier moments, the first
draft of this plan measured 34, and re-run in this worktree it is **36**, moved
by that draft and by this fold. **It is self-referential and is deliberately NOT
pinned as a gate by the spec, and this plan does not pin it either**; it is
recorded only to show the actionable set is a stable subset of a drifting
number. That set is the seven tracked files under
`design/journeys/`, and it is still exactly seven: two drivers
(`design/journeys/transcript.sh`, `design/journeys/transcript_pathological.sh`)
and five transcripts (`transcript_hashvault.txt`, `transcript_pathological.txt`,
`transcript_rcw.txt`, `transcript_tr_pathological.txt`,
`transcript_walletpolicy.txt`).

**Two of the drivers carry the workaround this cycle exists to delete**, and
their comments will be falsified by their own regeneration:
`design/journeys/transcript.sh:37` and
`design/journeys/transcript_pathological.sh:29` each explain a regex whose
reason is *"`md encode` prints `chunk-set-id: 0x…` on stdout"*.

**The display-grouping conformance corpus is shared by four repos and is NOT at
risk, which had to be checked rather than assumed.**
`design/display-grouping-vectors.tsv` is **byte-identical** across
`descriptor-mnemonic`, `mnemonic-key`, `mnemonic-secret` and `mnemonic-toolkit`
— one sha256, `7147b0ec…`, for all four — and three of them pin it in CI
(`descriptor-mnemonic/.github/workflows/ci.yml:72`,
`mnemonic-key/.github/workflows/ci.yml:95`, and `mnemonic-secret`'s
`rust.yml`). It contains rows keyed `hyphen` and `comma`, which §6c removes.
**It survives anyway**: its consumers are codec-level
(`descriptor-mnemonic/crates/md-codec/tests/display_grouping_conformance.rs`,
`mnemonic-key/crates/mk-cli/src/format.rs:115`,
`mnemonic-toolkit/crates/mnemonic-toolkit/tests/display_grouping_conformance.rs`),
each maps the keyword to a `char` **inside the test**, and the functions under
test take a `char` and have no keyword vocabulary. §6c's removal is at the CLI's
`parse_separator`. The corpus, its four copies and its three CI pins are
untouched.

---

## 2. THE BOUNDARY — 3 of 11 items adopted, 8 declined by all three

P1 was the first second consumer and took 5 of the same 11 items. **The universe
below is deliberately identical to P1's, item for item**, so the two boundary
tables are comparable line by line rather than being two differently-drawn
lists — including `write_private`, which is not in the crate yet and arrives
through P1's private-write entry. P3 is the third, fourth and fifth
consumers at once. **Each of the three takes less than `mt` did, and the reason
is the phase's row, not the crate's quality.**

**CITE THE CRATE BY SYMBOL ONLY. ITS SURFACE IS IN MOTION RIGHT NOW.** Two P1
entries are in flight as this is written: the fish purge recipe is being built
into `remedy.rs`, and `write_private` is moving into the crate out of
`me-cli`'s `main.rs`. Any line number taken from `crates/mnemonic-io-lib/` today
is wrong by the time this plan is executed. The two moving areas are `remedy`
and whatever module receives `write_private`; nothing else in the crate is
scheduled to move.

### 2.1 The verdicts

| crate item | `md` | `mk` | `mnemonic` |
| --- | --- | --- | --- |
| `write_private` (moving in from `me-cli` by P1's private-write entry) | **ADOPT** | **ADOPT** | DECLINE — no `--out` in P3's row |
| `remedy::history_purge_block` | DECLINE | DECLINE | **ADOPT** |
| `remedy::history_purge_recipes` | DECLINE | DECLINE | **ADOPT**, as the block's structured half |
| `channel::destination` | DECLINE | DECLINE | DECLINE |
| `exit::write_block` | DECLINE | DECLINE | DECLINE |
| `exit::WriteBlock` | DECLINE | DECLINE | DECLINE |
| `observation::PayloadKind` | DECLINE | DECLINE | DECLINE |
| `fd::stdout_mode` | DECLINE | DECLINE | DECLINE |
| `fd::mode_of` | DECLINE | DECLINE | DECLINE |
| `records::split_record_stream` | DECLINE | DECLINE | DECLINE |
| `records::no_records_guard` | DECLINE | DECLINE | DECLINE |

**So three consumers take THREE distinct items between them, and eight of the
eleven are declined by all three.** `md` and `mk` take one apiece and it is the
same one; `mnemonic` takes two and shares neither. That is the phase's headline
finding about the crate and it is not a complaint: P3's row is channels,
presentation and one exit code, and the crate is mostly a **write-gate and
refusal** library. The parts that fit are the parts P3 needs; the parts that do
not are the parts a different phase would need.

### 2.2 Why `remedy` is declined for `md` and `mk` — the spec declines it, not this plan

§4 is explicit and quotes `mt`'s own shipped refusal to make the point: *"md and
mk DO take their strings as arguments; md1/mk1 are watch-only, so a leak there
costs privacy rather than the money."* `md` and `mk` get **no argv refusal in
this cycle**, so they have nothing to print a purge recipe *for*. Adopting
`history_purge_block` into either would be dead code carrying a shell command.

### 2.3 Why `records::split_record_stream` is declined — both tools already have a better one

`md` and `mk` each carry a reader, and the two are a documented parity pair —
in one direction. `read_md1_strings` (`crates/md-cli/src/cmd/repair.rs:73`) says
*"Mirrors mk-cli's `read_mk1_strings` helper (cross-CLI parity)"*;
`read_mk1_strings` (`crates/mk-cli/src/cmd/mod.rs:207`) does not name `md` back,
and its first three doc lines are otherwise word-for-word identical to `md`'s.
Both do
something the crate's function does not: they **strip display separators per
line** before deciding a line is empty, so a grouped card and an unbroken card
both re-ingest — which is the whole of the mstring-grouping contract. The
crate's version filters on `trim().is_empty()` and returns the line unchanged.

Swapping it in would replace a **two-CLI parity contract that both sides
document** with a six-CLI one that has strictly less behaviour, and would leave
each caller to re-apply the strip. **Extending the existing helper to `--in` is
less work and preserves the parity.** This is the decline with the most
measurement behind it and the one a reviewer should attack first.

### 2.4 Why `records::no_records_guard` is declined — its message names another binary's flag

Its refusal text reads *"pass them on argv, with --in, or on stdin"* followed by
*"An EMPTY input is what a FAILED upstream command leaves behind -- `mt encode
--qr > rec.txt` writes nothing when it refuses"*. Printing `mt`'s flag out of
`md`'s mouth is a defect no reader would file against the crate and every reader
would file against `md`.

The tools already refuse an empty input with correct, tool-specific wording —
`md` at `expected at least one md1 string (positional or via stdin with '-')`
and `mk` at the same shape — and they route it to **different codes**: `md`
through `BadArg` to 2, `mk` through `UsageError` to 64. The crate is right to
publish no integer, and the callers are right to keep their own text. **This is
a second instance of F-276's finding and is filed as one.**

### 2.5 Why `exit::write_block`, `WriteBlock` and `PayloadKind` are declined

`write_block`'s `Destination::Terminal` arm returns a refusal unconditionally,
and §6e **retracted** the generalisation of `me`'s terminal gate in as many
words: *"the terminal gate stays scoped to `me`'s binary container"*, justified
by binary-in-a-scrollback and by nothing else. `md1` and `mk1` strings are short
printable ASCII a human must read in order to engrave them. Adopting
`write_block` unchanged gives `md` and `mk` a refusal §6e says they must not
have; calling it with `stdout_is_tty` hard-coded false is a lie to a function
about an observable fact, which P1 rejected for `mt` on the same grounds.

`WriteBlock` goes with it. `PayloadKind` goes for a different reason:
its variants are `Bearer` and `CarriesNoSecret`, and **`md1` and `mk1` are
neither shaped**. They are watch-only — `md`'s own stderr says *"a keyless
descriptor template (no keys)"* and `mk`'s says *"watch-only — public keys only,
cannot spend"*. `Bearer` is plainly false of them, and `CarriesNoSecret` is
worse than merely loose: read at the source rather than from another plan, its
doc comment defines it as *"a 65,536-byte `random`/`zeros`/`ones` fill image,
whose purpose is to DESTROY a payload"*. Filing a cosigner card under that
variant would make `exposure_matters()` a constant `false` **and** would say
something untrue about the artifact — the exact defect `observation.rs` exists
to prevent.

### 2.6 Why `channel::destination` is declined — and the one a reviewer may fairly reverse

`destination(out_given, stdout_is_tty)` returns three variants. For `md` and
`mk` the decision is `if --out { file } else { stdout }`, and `Terminal`
collapses onto `Stream` because §6e leaves them no terminal policy. A variant
nothing can act on, in a shared decision type, is exactly how the policy behind
it gets adopted later by someone tidying up — P1's argument for declining
`WriteBlock`, applied to a smaller type.

**This is the weakest decline in the table and it is marked as such.** Adopting
it costs two lines and buys cross-tool uniformity in how the route is named.
**What would flip it:** a ruling that `--out` on `md`/`mk` should refuse a
terminal, or a later phase giving either tool any terminal-conditional
behaviour. Either makes the third variant live and the decline wrong.

### 2.7 Why `fd::*` is declined

No entry in this plan reads or reasons about a file mode except through
`write_private`, which sets one rather than measuring one. P3's row carries no
world-readable gate for any of the three. `fd::mode_of` and `fd::stdout_mode`
have nothing to be called from.

---

## 3. THE DEPENDENCY — one pin, three manifests, and the push has already landed

**`mnemonic-io-lib` is not on crates.io, but it IS on `origin/master` now.**
Re-measured in this worktree: `git ls-tree -d origin/master crates/` names
`crates/me-cli` **and** `crates/mnemonic-io-lib`, and `origin/master` is
`6c24e62`. The first draft recorded the opposite, correctly, four minutes before
P1's push landed. **All three manifests can pin an exact SHA today**, and this
plan does not plan a workaround around a gate that is already open. `path =` is
ruled out by the spec for the reasons `me-cli`'s own manifest already records at
length: a path does not resolve in a fresh CI checkout, and this repo currently
exists at more than one filesystem location.

**The mechanism is already proven and is not re-proven here.** P1 built a
throwaway crate outside both workspaces, git-depped the library from a local
clone by rev, and called into three of its modules. P3 inherits that result;
what P3 adds is that the **same** rev is pinned three times, in
`descriptor-mnemonic/crates/md-cli/Cargo.toml`,
`mnemonic-key/crates/mk-cli/Cargo.toml` and
`mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml`.

**The consequence the first draft warned about did NOT occur, and is recorded
closed rather than dropped.** The hazard was that P1's crate-side work might not
be inside the pushed SHA, leaving every channel entry here blocked on a second
push. Measured at `6c24e62`: `crates/mnemonic-io-lib/src/write.rs:45` is
`pub fn write_private`, and `crates/mnemonic-io-lib/src/remedy.rs:79` and `:144`
are `history_purge_recipes` and `history_purge_block`. **All three items P3
adopts are in the pushed SHA.** One push, one pin, no second one.

**`lib.rs`'s root re-exports do not cover what P3 needs — and the module TWO of
the three branches adopt is the one that is missing.** Measured at
`crates/mnemonic-io-lib/src/lib.rs`, the root re-exports are exactly three
lines: `:81` `channel::{destination, Destination}`, `:82`
`exit::{write_block, WriteBlock}`, `:83`
`records::{no_records_guard, split_record_stream}`. Everything else is reachable
only through its module path. That leaves **two** traps, not one:

- **`write` is a `pub mod` (`:74`) with NO root re-export.** `write_private` is
  therefore `mnemonic_io_lib::write::write_private`, and
  `mnemonic_io_lib::write_private` is an `E0425`. **This is the item `md` and
  `mk` both adopt** — the two channel entries below — so it is the trap most
  likely to be hit, and the first draft's version of this paragraph did not name
  it.
- `fd`, `observation` and `remedy` are likewise module-qualified only, so
  `mnemonic`'s adoption is written `mnemonic_io_lib::remedy::history_purge_block`
  and `mnemonic_io_lib::history_purge_block` is an `E0425`.

Stated because P1 hit the second of these with a probe, and round 0 found the
first by reading `lib.rs` rather than the paragraph describing it.

---

## 4. TDD ORDER

Each entry is RED first unless its gate column says otherwise. No entry begins
until the previous entry **in its own repo** is green; the three repo columns
run in parallel, and the join entries wait on all of them **in table order** —
the joins are sequential among themselves, not a set. **This table is the
only ordering of record**; prose refers to work by NAME — *the pin*, *the md
header*, *the md ungrouping*, *the md separator*, *the md reader*, *the md
channels*, *the mk ungrouping*, *the mk separator*, *the mk blank line*, *the mk
channels*, *the mk exit code*, *the md1 set flag*, *the mnemonic grouping*, *the
mnemonic refusal*, *the mnemonic override*, *the toolkit release*, *the GUI
mirror*, *the goldens*, *the acceptance*, *the decline* — so a renumbering
cannot falsify it.

**The mnemonic grouping and the mnemonic refusal each own doc-transcript work
INSIDE their own entry**, not as a later join: the branch that ships them
without rewriting or regenerating the goldens named in their gates leaves a red
CI surface behind that this plan's test-runner-shaped definition of green cannot
see.

| # | repo | work | the gate that must fail first |
| --- | --- | --- | --- |
| 1 | all three | **THE PIN.** Add `mnemonic-io-lib = { git = "https://github.com/bg002h/mnemonic-engrave", rev = "<SHA>" }` to all three CLI manifests, at **one** SHA carrying P1's crate-side work. Three files, no other edit. **Regression-gated, not RED-first** | **The prerequisite half of this gate can no longer fail and is recorded as SATISFIED, not asserted**: `git ls-tree -d origin/master crates/` names `crates/mnemonic-io-lib` as of `origin/master` = `6c24e62`, and that SHA carries all three adopted items (`crates/mnemonic-io-lib/src/write.rs:45` `write_private`; `crates/mnemonic-io-lib/src/remedy.rs:79` and `:144`). The first draft recorded it RED four minutes before the push landed. **The two conjuncts that ARE still live gates**: `cargo build --locked` succeeds in all three repos from a **fresh** clone, not a warm cargo cache; and the pinned SHA exposes `write_private`, asserted **by a call that compiles** — written `mnemonic_io_lib::write::write_private`, because `write` has no root re-export and the unqualified path is an `E0425` |
| 2 | `md` | **THE md HEADER LEAVES stdout.** `crates/md-cli/src/cmd/encode.rs:172` writes `chunk-set-id: 0x…` to stderr instead. `crates/md-cli/src/cmd/vectors.rs:76` is **not** touched — it writes a fixture file, and §6a's rule is about stdout | **fails today on two named tests**: `a_policy_over_the_single_string_cap_chunks_without_the_flag` and `force_chunked_still_chunks_a_short_policy` (`crates/md-cli/tests/cmd_encode.rs:652`, `:724`) each assert `stdout.contains("chunk-set-id:")`. **Plus the control that must NOT move**: `a_short_policy_still_emits_a_single_string` (`:703`) asserts the header is absent from an unchunked run and stays green. **Its reasoning in the first draft was wrong and the correction matters**: read at the source, that test asserts `!stdout.contains("chunk-set-id:")` — an **absence**, so an implementation that deleted the header *everywhere* leaves it **green**, not RED. This control catches a header that leaks into an unchunked run; it cannot distinguish "moved to stderr" from "deleted". **The only thing that distinguishes them is the stderr assertion in this same cell**, which is therefore load-bearing rather than belt-and-braces, and must not be dropped as redundant. **Plus the four helpers that strip it** — `crates/md-cli/tests/cmd_address.rs:72`, `crates/md-cli/tests/cli_output_class.rs:274`, `crates/md-cli/tests/cli_repair.rs:31`, `crates/md-cli/tests/cmd_descriptor.rs:32` — all four filter on `starts_with("md1")` rather than dropping line 0, measured, so none silently eats a chunk. **And stderr must carry it**, asserted, or the chunk-set-id is simply gone |
| 3 | `md` | **THE md STDOUT UNGROUPS, AND THE CARD APPEARS.** stdout emits the unbroken `md1` string; `--group-size` / `--separator` now shape a **stderr engraving card** whose contents this plan fixes below the table | **fails today**: `encode_default_groups_space_5` (`crates/md-cli/tests/cmd_encode.rs:25`) asserts the space/5 default on stdout. **Plus the packability assertion, measured RED four ways today** — `md encode` on a keyed 2-of-2 into `me sysw pack --out` exits **4 on record 0** with the header present and grouped, **4** with the header present and ungrouped, **4** with the header stripped and grouped, and **0 with a 391-byte payload** only when both are fixed. So this entry alone cannot make the pipeline pass and neither can the md header alone, which is the point. **Plus the 36 existing `--group-size` occurrences** across 13 files under `crates/md-cli/tests` — the value that follows is `0` at **all 36**, measured by parsing rather than by a one-line grep that would have seen only the 12 written on a single line — so all 36 keep passing under the new default and none of them is evidence that the default changed |
| 4 | `md` | **THE md SEPARATOR NARROWS** (§6c). `parse_separator` (`crates/md-cli/src/main.rs:54`) accepts whitespace only; `hyphen` and `comma` are rejected with a message naming what replaced them | **RED-first, and the test does not exist yet**: `md encode --separator hyphen` exits **0** today, measured, so the gate is a new test asserting a non-zero exit and a message. **Plus the control**: `--separator space` and a literal `" "` still work. **Plus the corpus, asserted UNCHANGED** — `sha256sum -c descriptor-mnemonic/design/display-grouping-vectors.tsv.sha256` still passes and `display_grouping_conformance` still runs its hyphen and comma rows, because those exercise `md_codec::encode::render_grouped`, which takes a `char`. A fix applied one layer too deep goes RED here |
| 5 | `md` | **THE md READER IS HOISTED AND `-` REACHES FOUR VERBS.** Move `read_md1_strings` from `crates/md-cli/src/cmd/repair.rs:73` to `crates/md-cli/src/cmd/mod.rs` beside `strip_md1_inputs` (`:5`), make it `pub`, and route `decode`, `verify`, `inspect` and `bytecode` through it — the shape `mk` already has at `crates/mk-cli/src/cmd/mod.rs:207` | **the gate is EQUALITY, not failure, and that is load-bearing**: `md decode -` with a card on stdin must produce stdout and stderr byte-equal to `md decode <that card>` at exit 0. **A "fails today" gate would pass in both worlds** — measured, `md decode -` exits **1** with `string does not start with HRP md1`, because `-` is taken as a literal md1 string. **THE SAME THREE ASSERTIONS ON ALL FOUR VERBS, `bytecode` INCLUDED.** The first draft's work column named four verbs and its gate named three, so an implementation that hoisted the reader and wired `decode`/`verify`/`inspect` passed every stated check while leaving the fourth untouched — measured, `md bytecode -` exits **1** with the identical message and is the identical defect. **And `verify`'s assertions MUST supply `--template`**: bare `md verify -` exits **2** from clap (*"the following required arguments were not provided: `--template`"*), so a `verify` gate written from the first draft's stated exit-1 baseline measures clap and never reaches the reader. With `--template` supplied it is 1, and the equality gate bites. **Plus the control**: a non-dash positional on each of the four verbs is still parsed as a card |
| 6 | `md` | **THE md CHANNELS.** `--in FILE` on `encode` reads a BIP-388 template (§10's `md encode --in wallet.template`); `--in FILE` on the reading verbs reads md1 strings through the hoisted reader. `--out FILE` writes the artifact through the crate's `write_private` | `md encode --in <a template file> --out f` where `f` already exists at **0644** leaves `f` at **0600** holding the artifact, and `--out` suppresses nothing else. **`0o600` binds on CREATE only**, so the overwrite half is the whole of F-244 and a mode-on-create implementation goes RED. **`--out` OVERWRITES** per §6b's operator ruling, asserted by running the same command twice. **Fails today**: a help-surface sweep counts **0** `--in`/`--out` flags across all five `md` verbs |
| 7 | `mk` | **THE mk STDOUT UNGROUPS, AND THE CARD APPEARS.** Same ruling as `md`'s, at the single emission site `crates/mk-cli/src/cmd/encode.rs:344` | **fails today**: `encode_default_groups_space_5` (`crates/mk-cli/tests/encode_grouping_flags.rs:37`). **Plus the packability assertion, measured RED and GREEN today**: `mk encode` default into `me sysw pack --out` exits **4 on record 0**; the same with `--group-size 0` exits **0 with a 244-byte payload**. `mk` has no header, so grouping alone is the whole defect and the gate is a single-variable measurement |
| 8 | `mk` | **THE mk SEPARATOR NARROWS.** `parse_separator` (`crates/mk-cli/src/format.rs:40`) — note the module differs from `md`'s | **RED-first, new test**: `mk encode --separator hyphen` exits 0 today. **Plus the same corpus assertion**, against `mnemonic-key/design/display-grouping-vectors.tsv.sha256`, which pins a file **byte-identical** to `md`'s — one sha256 across four repos, measured — and whose consumer here is `crates/mk-cli/src/format.rs:115` |
| 9 | `mk` | **THE mk BLANK LINE LEAVES stdout.** `crates/mk-cli/src/cmd/encode.rs:339` prints a blank line between cards on the `--keys` path. §6a's `encode` rule admits the artifact and nothing else. **The first draft justified this with "the card boundary is recoverable from each card's own chunk header"; that justification is FALSE and is withdrawn** — see the gate. The rule stands on §6a alone, which needs no such argument | **fails today, and §2 of the spec says it cannot**: on a two-record key file `mk encode --keys` writes a blank line to stdout, at `crates/mk-cli/src/cmd/encode.rs:339`. **The line COUNT is fixture-dependent and is deliberately not asserted** — measured with `mk-cli`'s own `KEYS` fixture the output is 6 lines with 1 blank (a 2-chunk card and a 3-chunk card), not the 5 the first draft recorded from a different fixture. **The gate is the fixture-independent shape**: every stdout line begins `mk1`, and the blank count is **0**. **Plus the control**: single-card `mk encode` stdout is byte-identical before and after. **AND THE WITHDRAWN JUSTIFICATION, REPRODUCED so nobody restores it**: a key file carrying the same BIP-380 record twice is accepted at **exit 0**, and emits two byte-identical cards sharing one chunk-set-id — measured, both cards begin `mk1qp d8cwp`. Their headers are the same header, so after this entry the boundary is not recoverable from them at all, and the blank line was the only signal that `mk` had silently accepted a duplicate cosigner. Deleting it is still right under §6a; the duplicate-acceptance defect underneath is pre-existing and is filed, not fixed here |
| 10 | `mk` | **THE mk CHANNELS.** `--in FILE` on the reading verbs through `read_mk1_strings` (`crates/mk-cli/src/cmd/mod.rs:207`); `--in FILE` on `encode` routes to `keyfile::read_key_records` (`crates/mk-cli/src/keyfile.rs:99`), the reader `--keys` already uses. `--keys` is retained and `--in` is mutually exclusive with it. `--out FILE` through the crate's `write_private` | the same 0644 → 0600 create-and-overwrite pair as `md`'s channels, plus: `mk encode --in <keys file>` is byte-equal on stdout to `mk encode --keys <the same file>`, and supplying both exits 64 with a message naming both. **Fails today**: **0** `--in`/`--out` flags across all five `mk` verbs |
| 11 | `mk` | **THE mk EXIT CODE, SPLIT BEFORE IT IS MOVED** (§6f). The arm covering `CliError::Codec` and `CliError::MdCodec` jointly returns **1** — it is at `crates/mk-cli/src/error.rs:111`, inside `CliError::exit_code` (`crates/mk-cli/src/error.rs:108`). `repair` gains `md`'s bypass **at md's actual width — ANY codec error out of the correcting decode returns `Ok(2)`, not only an uncorrectable one** (`crates/md-cli/src/cmd/repair.rs:124`, a bare `Err(e) =>` arm documented at `crates/md-cli/src/cmd/repair.rs:107`); the call to wrap is `crates/mk-cli/src/cmd/repair.rs:113`. `SetReassemblyMismatch` stays **2**. **Plus a comment this diff falsifies without touching**: `crates/mk-cli/src/cmd/repair.rs:112` says *"route to exit 2 via `CliError::Codec(_) => 2` in error.rs"*, which stops being true — rewriting it is part of the work | **BOTH DIRECTIONS ARE RED TODAY AND BOTH MUST BE ASSERTED.** `mk decode <garbage>` exits **2** and must become **1**; `mk repair <a BCH-uncorrectable card>` exits **2** and must **stay** 2. **THE THIRD ASSERTION IS WHAT SEPARATES THE TWO CANDIDATE IMPLEMENTATIONS AND THE FIRST DRAFT HAD NO CELL FOR IT**: `mk repair <an mk1 with its HRP swapped to ms1>` — an *invalid artifact* on `repair`, not an uncorrectable one — exits **2** today and must **stay** 2. A bypass scoped to the BCH-uncorrectable variant sends it to 1; `md`'s all-errors bypass keeps it at 2, and `md` is the parity being copied. **Plus the SIX tests that must not move, not four.** The first draft named only `crates/mk-cli/tests/cli_mk1_repair_reverify.rs:178`, `:194`, `:239`, `:259` (the `SetReassemblyMismatch` four) and called them *"the ONLY `.code(2)` sites"* with *"zero tests pin the invalid-artifact 2"*. **Both claims are false**: a `.code(N)` grep sees one of three assertion families, and `crates/mk-cli/tests/cli_repair.rs:152` and `:185` — in `repair_beyond_t4_capacity_exits_2` (`crates/mk-cli/tests/cli_repair.rs:140`) and `repair_hrp_mismatch_exits_2` (`crates/mk-cli/tests/cli_repair.rs:172`) — pin `CliError::Codec(_)` at 2 through `assert_eq!(code, 2)`. **VERIFIED BY MUTATION, both ways, on a scratch copy**: the naive one-line edit alone gives **337 tests, 335 passed, 2 failed**, the two failures being exactly those tests; the edit **plus** the correctly-scoped bypass gives **337 passed, 0 failed**. So the gate is: the suite is green, `repair` is 2 on both an uncorrectable and an HRP mismatch, and `decode` is 1 on both. **Plus `verify` and `inspect` and `encode --from-md1 <bad>`**, each measured at 2 today, each asserted at 1 after |
| 12 | `mk` | **THE md1 SET FLAG** (§10). `--from-md1-set FILE` reads md1 strings from a file and binds the stub exactly as repeated `--from-md1` does. It **skips every line that is not an md1 string**, so a file carrying today's `chunk-set-id:` header and one written by tomorrow's `md encode --out` both work | `mk encode --from-md1-set <a 4-chunk file>` produces stdout byte-equal to the same call with four repeated `--from-md1`, measured working today at exit 0 with two `mk1` lines out. **The header-tolerance assertion is what makes this entry independent of the md header**, and it is testable now. **Plus the equivalence that removes the ordering**: fed a GROUPED md1 set and an UNGROUPED one, `mk encode` already produces byte-identical output — measured — so this flag never needs to know which era wrote the file. **Fails today**: `mk encode --help` contains `from-md1-set` **0** times |
| 13 | `mnemonic` | **THE mnemonic GROUPING SURFACE.** `--group-size` default 5 → 0 at all four declaring sites (`crates/mnemonic-toolkit/src/cmd/bundle.rs:82`, `crates/mnemonic-toolkit/src/cmd/convert.rs:350`, `crates/mnemonic-toolkit/src/cmd/ms_shares.rs:76` and `:118`); `parse_separator` (`crates/mnemonic-toolkit/src/display_grouping.rs:45`) narrows to whitespace. The `#` comment headers on `bundle`'s stdout **do not move** | **fails today on all four**: each subcommand's default emits space/5, and `--separator hyphen` exits 0. The gate asserts the ungrouped default and a non-zero exit for both retired keywords, on **each of the four**, generated rather than hand-listed. **Plus the control that pins the decline**: `mnemonic bundle`'s stdout still carries its three `#` comment lines and three blank lines — 12 lines, 6 non-artifact, measured — so an implementer who read §4 as absolute goes RED here. **Plus the corpus**, unchanged, for the same codec-level reason as `md`'s. **PLUS THE FOUR DOC GOLDENS THIS FLIP INVALIDATES, WHICH `cargo nextest` CANNOT SEE.** `mnemonic-toolkit/docs/manual/transcripts/22-first-bundle.out`, `41-bundle-inheritance-cards.out`, `cross-format-recipes/recipe-2-bitcoin-core-to-bundle.out` and `qs-23-bundle.out` pin space-5 grouped artifact lines and are byte-compared by three workflows outside the test runner. **These four genuinely do just regenerate** — the commands do not change, only their output — so the gate is: regenerate, and the resulting diff touches those four files and **nothing else**. A run that changes a fifth golden means the flip reached a surface this entry did not intend |
| 14 | `mnemonic` | **THE mnemonic ARGV REFUSAL, PRE-PARSER** (§6d). A guard over raw `std::env::args()`, before `Cli::parse()`, that matches a static flag-name table and — for the `<node>=<value>` forms — splits at `=` and tests the token against `secret_taxonomy::SECRET_NODE_TYPES_ARGV` (`crates/mnemonic-toolkit/src/secret_taxonomy.rs:95`). It reports the CLASS and the LENGTH, never the value; names `mnemonic`'s OWN private channels; and prints the crate's `remedy::history_purge_block` | **fails today on all five named channels**, each measured emitting a warning and **proceeding**: `bundle --slot @0.phrase=` and `--passphrase` (2 advisories, exit 0), `convert --passphrase` (exit 0), `derive-child --passphrase`, `restore --passphrase`, `electrum-decrypt --decrypt-password` — the last spelled `password`, not `passphrase`. **Plus the parity assertion that makes the predicate the boundary**: every token in `SECRET_NODE_TYPES_ARGV` is refused when it arrives as `--from <token>=<value>`, generated from the const so a tenth token cannot be added without the test seeing it. **Plus the two controls**: a value of `-` and a value beginning `@env:` are NOT refused, because both are existing private channels and refusing them removes the remedy. **Plus §6h's rule**, asserted: the refusal names no channel that does not exist — `--in` does not exist on any `mnemonic` verb and must not be advised. **PLUS THE 19 DOC TRANSCRIPTS THIS REFUSAL INVALIDATES, WHICH ARE THE LARGER HALF OF THIS ENTRY AND CANNOT BE REGENERATED.** `git grep -l 'secret material on argv' -- docs/manual/transcripts` returns **19** files whose `.cmd` puts secret material on argv and whose golden records the tool *warning and proceeding* — including the manual's and the QuickStart's flagship bundle examples. This entry makes those commands **refuse**, so re-running the generator writes a refusal where the document explains a result. **The `.cmd` files must be REWRITTEN to a private channel (stdin, or `--allow-argv-secret`) and the surrounding prose updated**, then regenerated. The gate is that all three transcript workflows pass with the rewritten commands, and that **no golden still contains the advisory string** except where a `--allow-argv-secret` example deliberately demonstrates the override. **A green `cargo nextest` is not evidence for any of this** — the runner never replays a transcript |
| 15 | `mnemonic` | **THE mnemonic OVERRIDE.** `--allow-argv-secret` proceeds. Its own parse is on raw argv, before clap; when present, the override **and** the admitted token are removed from the argv handed to clap and the material is carried in through the same internal path the `-stdin` variants use | `mnemonic convert --allow-argv-secret --from phrase=<the vector> --to xpub --template bip84` exits 0 and emits stdout byte-equal to the `--from phrase=-` run with the phrase on stdin. **The naive implementation fails the SECOND assertion**: if the admitted token is left in argv for clap, an unrelated clap error echoes it — the leak §6d exists to stop, reproduced in `mt`. So the gate also asserts **no material in stderr** for the same invocation plus an unknown flag, where clap must name the flag and never the value. **Plus the control**: `--allow-argv-secret` with no secret material behaves exactly as the flagless run |
| 16 | join | **THE TOOLKIT RELEASE AND THE PIN BUMP.** Cut a `mnemonic-toolkit` release carrying the flipped `--group-size` default and the narrowed separator, then move **both** GUI pins to that tag: `pinned-upstream.toml:22` and the load-bearing dependency pin at `mnemonic-gui/Cargo.toml:76`, which are required to move in lockstep by `pinned-upstream.toml`'s own text. **This entry did not exist in the first draft and it is a hard prerequisite of the mirror below.** **Regression-gated, not RED-first** | the pinned binary resolved by the GUI's test harness reports the **new** default — `mnemonic gui-schema` emits `default_value: 0` for `--group-size` at `bundle`, `convert`, `ms-shares-split` and `ms-shares-combine` — and both pin sites name the same tag. **Fails today**: both pins read `mnemonic-toolkit-v0.97.0` and that binary emits `5` at all four |
| 17 | join | **THE GUI MIRROR.** One agent, one branch, one commit in `mnemonic-gui`: the four `SEPARATORS` constants lose `hyphen` and `comma`, and the `default_value` sites for `md`, `mk` and `mnemonic` follow their CLIs. Its `ms` schema file is **P2's** and is not touched here. **Regression-gated, not RED-first** | **THE DRIFT GATE IS BLIND TO TWO OF THE THREE EDITS AND LIVE ON THE THIRD — the first draft called it blind to all three, and that is the finding this cell now carries.** Blind, measured: it scopes itself to `mnemonic` only, and `md`/`mk` emit `default_value: null` so the one-sided guard never arms; and its choices comparison is scoped to flags whose pinned JSON carries non-null `choices`, which `--separator` does not, on any of the four CLIs. **LIVE**: `--group-size` for `mnemonic` is compared unconditionally — `DEFAULT_VALUE_ALLOWLIST` (`tests/schema_mirror_defaults_drift.rs:48`) holds exactly one unrelated entry — so this is a **lockstep** gate that reds if either side moves alone. Measured by running it: flipping the four GUI sites with the pin still at the old tag produces **exactly four drift violations** and reds `mnemonic_defaults_and_choices_match_pinned_gui_schema`, while the `md`/`ms`/`mk` test stays green. **So the gate here is BOTH**: the GUI suite must be **green** (which is only achievable after the release-and-pin entry above, and is therefore real evidence rather than vacuous), **AND** each `SEPARATORS` constant is compared **directly** against the freshly built `gui-schema` output, because that half the suite genuinely cannot see |
| 18 | join | **THE GOLDENS.** Regenerate the seven tracked files under `design/journeys/` in this repo, and delete the two header-stripping workarounds the drivers carry. **Regression-gated, not RED-first** | the regeneration is run, not described, and its diff is enumerated. `design/journeys/transcript.sh:37` and `design/journeys/transcript_pathological.sh:29` each explain a regex by *"`md encode` prints `chunk-set-id: 0x…` on stdout"* — both comments and both regexes go. **This entry cannot begin until P2 has landed**, because the same drivers shell out to `ms` with the seed on argv at the 18 call sites P2 owns, and regenerating before that migration means regenerating twice. **The driver binaries are `target/release`, not `target/debug`** (`design/journeys/transcript.sh:9-11`), so all three repos need a release build first or the transcripts pin stale behaviour |
| 19 | join | **THE ACCEPTANCE, RUN** (§10). The two-stage pipeline, with no `grep` and no `--group-size`, on a CHUNKING policy | `md encode --in wallet.template --out wallet.md1`, then a brace group of `cat wallet.md1`, `mk encode --in cosigner1.keys --from-md1-set wallet.md1` and `mt encode --qr --in tx.hex` into `me sysw pack --expect descriptor,cosigner,transaction --out payload.bin`, at exit 0. **And the negative half**: the same pipeline with one producer made to refuse exits non-zero and writes no payload. **A trap measured today, so the gate does not chase it**: `me sysw pack` **without** `--out`, into a shell redirect, exits **2** on a mode-0644 stdout — an unrelated F-252 refusal that would read as this cycle's defect. The pack side takes `--out`. `--no-passphrase` is **not** needed: md1 and mk1 are watch-only, so the container is not sealed, measured at exit 0 both ways |
| 20 | all three | **THE DECLINE, ASSERTED.** No code. Each tool keeps what no §6 ruling changes, and the tests that pin those are named so a later phase cannot delete them as tidying. **Regression-gated, not RED-first** | `md`, `mk` and `mnemonic` each still write to a **terminal** without refusing, so an adoption of `exit::write_block` that imported `me`'s terminal gate goes RED. `md decode`, `mk decode` and `ms decode` stdout shapes are unchanged — §6a puts them out of scope by name. `mnemonic bundle`'s `#` comment headers are still on stdout. `md` and `mk` still accept their material on **argv** with no refusal, per §4. The display-grouping corpus still checksums to `7147b0ec…` in all four repos. **Plus the enumerated diff**: every edit to the three suites — 805, 337 and 3960 tests — listed, each justified by a named §6 ruling or a numbered finding |

**THE ENGRAVING CARD, SPECIFIED — because §6c says the plan must and two
implementers would otherwise render it two ways.** §6c is explicit that `md` and
`mk` have no card to move the grouped form to, that P3 owns its contents, and
that the minimum it must carry is the grouped string itself.

The card follows **`ms`'s** shape, measured today: plain `label: value` lines on
stderr, no prefix character, ending with the existing output-class advisory.
`mnemonic bundle`'s `#`-prefixed stderr card is the other in-constellation
precedent and is **not** followed, because its `#` mirrors the comment headers
on its own stdout — a surface this plan leaves alone. So:

```
md1yq pqqxq q8xtw hw4xw n4qh
group size: 5
separator: space
note: stdout is a keyless descriptor template (no keys)
```

The grouped string comes first because it is the thing a human transcribes; the
existing single advisory line stays last and is unchanged. `mk`'s card is the
same shape with its own advisory. **`--no-engraving-card` is NOT added to `md`
or `mk`** — see the out-of-scope list.

**EVERY LINE NUMBER IN THIS PLAN IS ANCHORED AT `beb2fb2` (`md`), `c5739fc`
(`mk`) AND `8342b2e` (`mnemonic`), AND THE CITATION GATE CANNOT SEE WHEN THEY
GO STALE.** F-279 measured this on P1: 14 of its 15 `mt` citations had drifted
under its own early entries while `plan-cite-check.sh` reported all 15 green,
because the gate checks that a line exists and never what is on it. **P3 cites
three repos, so the exposure is larger, not smaller.** Every citation here names
its symbol beside its number for exactly that reason. **LOCATE EVERY SITE BY
SYMBOL AND RE-MEASURE THE LINE BEFORE QUOTING IT.**

**SIX PIECES OF WORK ARE REGRESSION-GATED RATHER THAN RED-FIRST**, and the
column header must not claim otherwise: the pin (a build resolving), the toolkit
release (a version moving), the GUI mirror (a mirror following its source), the
goldens (a regeneration), the decline (a backstop protecting RED-first work),
and the *corpus-unchanged* half of both separator entries. Everything else is
RED-first, and every one of those gates was **run** to establish that it fails —
the exit codes are in this document because they were captured to files and
grepped, not inferred.

**AND ONE FORMER RED HALF IS NOW SATISFIED RATHER THAN FAILING.** The pin's
`git ls-tree` conjunct passed between the first draft and this revision. It is
kept in the table as a **recorded measurement**, not as a gate expected to fail;
the entry's two live conjuncts are named in its cell. *A gate that can no longer
fail is not a gate, and leaving it labelled RED would make the next reader trust
a check that cannot check anything.*

---

## 5. WHAT MUST BE TRUE TO CLOSE P3

1. **`md encode` on a CHUNKING policy pipes into `me sysw pack` with no flags
   and no `grep`, at exit 0.** §7's P3 gate in as many words. Measured RED four
   ways today — header alone, grouping alone, both, neither — so **both** the md
   header and the md ungrouping are required and neither is sufficient. Those
   two entries build it.
2. **`mk` on an invalid artifact exits 1, `mk repair` exits 2 on an
   uncorrectable card AND on an invalid artifact, and `SetReassemblyMismatch`
   still exits 2.** §6f calls the first the only code this cycle changes. The
   middle pair is the parity `md` already holds — `md repair` is 2 for **any**
   codec error while `md decode` is 1 — and asserting only the uncorrectable
   half is what lets a too-narrow bypass pass a gate while reddening
   `repair_hrp_mismatch_exits_2`. The last is the funds fix. **Six tests pin a 2
   in this suite, not four**, across three assertion families, and a
   `.code(N)`-only census does not find them. The mk exit code builds it.
3. **`mk encode` pipes into `me sysw pack` with no flags**, on the `--keys` path
   §10 actually uses — so with the blank line gone as well as the grouping. The
   mk ungrouping and the mk blank line build it.
4. **`md decode -`, `md inspect -`, `md bytecode -` and `md verify -` — all
   FOUR verbs — each read stdin at exit 0**, asserted as byte-equality with the
   positional run rather than as success, because `-` is currently accepted as a
   literal value and fails, which is the one failure mode a "does it fail" gate
   cannot distinguish. **`bytecode` is in this list because it has the identical
   defect and the first draft's gate omitted it**, and **`verify`'s assertion
   supplies `--template`**, because bare `md verify -` exits 2 in clap and never
   reaches the reader. The md reader builds it.
5. **`--in` and `--out` exist on `md` and `mk`, and `--out` creates 0600 on
   CREATE and on OVERWRITE.** The md channels and the mk channels build it.
6. **No line of `md`'s or `mk`'s stdout on `encode` is anything but the
   artifact** — no header, no blank line, no grouping. The md header, the md
   ungrouping, the mk ungrouping and the mk blank line build it.
7. **`hyphen` and `comma` are gone from `--separator` on `md`, `mk` and
   `mnemonic`, and the shared display-grouping corpus is untouched in all four
   repos.** The two separator entries and the mnemonic grouping build it.
   **This item had no owning phase before this plan.** §7's P2 row assigns
   *"the whitespace-only separator"* to `ms`, whose §3 measurement it names;
   §7's P3 row does not mention the separator at all, while §6c binds it
   *"everywhere"*. P3 claims it because no other phase can.
8. **`mnemonic` REFUSES secret material on argv at all five named channels, and
   the refusal's boundary is the predicate rather than the list.** Asserted as
   the five spot checks §7 requires **plus** a parity test generated from
   `SECRET_NODE_TYPES_ARGV`, so the boundary cannot drift from the list. The
   mnemonic refusal builds it.
9. **`mnemonic`'s refusal decision AND the admitted material's route are both
   settled before `Cli::parse()`.** A guard that reaches its decision by parsing
   first has reintroduced the leak §6d exists to stop, and an override that
   hands the admitted token back to clap has created a new one — reproduced in
   `mt`. The mnemonic refusal and the mnemonic override build it.
10. **`--from-md1-set FILE` exists on `mk encode` and is equivalent to the
    repeated flag.** §10 introduces it and names P3 as its owner. The md1 set
    flag builds it.
11. **A `mnemonic-toolkit` release carries the new grouping default, and BOTH
    GUI pins name it** — `pinned-upstream.toml:22` and
    `mnemonic-gui/Cargo.toml:76`, which that file requires to move in lockstep.
    **This condition did not exist in the first draft**, which believed the pin
    was far behind and the gate therefore blind; it is exactly current, so the
    mirror cannot be green without it. The toolkit release builds it.
12. **The GUI mirror matches its three CLIs BOTH ways: the drift suite is green,
    AND each `SEPARATORS` constant is compared directly against a freshly built
    `gui-schema`.** The suite half is real evidence only after the pin bump, and
    the direct half is required because the drift gate genuinely cannot see a
    `--separator` change on any CLI, nor anything at all on `md`/`mk`. The GUI
    mirror builds it.
13. **The seven journey goldens regenerate, and the two header-stripping
    workarounds in the drivers are gone.** The goldens build it, and it cannot
    start before P2 has landed.
14. **The toolkit's 23 affected doc transcripts are green**: the 19 whose
    goldens pin the argv advisory have had their `.cmd` files **rewritten** to a
    private channel and their prose updated, and the 4 that pin space-5 grouping
    have been regenerated. **This is a fourth validation surface, replayed by
    three workflows that `cargo nextest` never runs**, and it appeared nowhere in
    the first draft. The mnemonic refusal and the mnemonic grouping build it,
    inside their own entries.
15. **§10's acceptance pipeline has been RUN, in both its positive and its
    negative form.** *A gate that has never executed is a hypothesis, not a
    gate.* The acceptance builds it.
16. **FOUR validation surfaces are green, not three, and not the test counts
    alone:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
    -- -D warnings`, `cargo nextest run --locked`, each repo's checksum-pinned
    conformance step, **and `mnemonic-toolkit`'s three doc-transcript
    workflows**. Baseline measured 2026-08-27, every exit code read directly:
    **all nine of the runner-shaped gates green**, 805 / 337 / 3960 tests.
    Unlike P1, nothing here is red before the phase writes a line. **The fourth
    surface was the blind spot: naming only the runner is what let 23 committed
    goldens sit outside every gate in the first draft.**
17. **(assertion, not work — no entry builds it; it is checked, not created)**
    **No exit code moves except `mk`'s invalid artifact.** `md`'s 1/2 split,
    `mnemonic`'s 1-or-2-by-input-shape, every repair code, and the 2-versus-64
    clap split are all unchanged. §6f rules all four in as many words, and a P3
    that renumbered any of them would be acting outside its ruling.
18. **(assertion, not work — checked, not created)** **`FOLLOWUPS.md` in all
    four affected repos grepped for BOTH closure vocabularies before anything is
    scheduled.** This repo closes follow-ups as `CLOSED` **and** as `DONE`, and
    a single-token sweep reports half the truth with total confidence.
19. **(assertion, not work — checked, not created)** **The follow-ups this plan
    files have actually LANDED in `design/FOLLOWUPS.md`**, with owning phases,
    and the ones this phase owns are burned down before it closes. The first
    draft filed six follow-ups and had no condition that any of them reached
    disk — one of them carried an ordering constraint that lived only in its own
    text.
20. **An R0 round closing 0C/0I.**

---

## 6. OUT OF SCOPE

- **A world-readable or terminal write gate on `md`, `mk` or `mnemonic`.** §6e
  retracted the generalisation; the gate stays scoped to `me`'s binary
  container. This is why five of the crate's items are declined.
- **An argv refusal on `md` or `mk`.** §4 exempts watch-only material by name
  and quotes `mt`'s own refusal doing so. Adding one is a ruling, and this phase
  does not make it.
- **Stripping `mnemonic bundle`'s `#` comment headers from stdout.** Consulted
  and ruled during authorship; §6a scopes the stdout rule to `encode` by an
  explicit table `mnemonic` is not in, §2a enumerated `mnemonic`'s involvement
  with no header item, and §9a puts `mnemonic` in a tier that never feeds
  `me sysw pack`. Breaking a shipped machine-readable surface is the exact class
  §6a refused to break for `mk decode` and said *"gets its own phase and its own
  gate"*. Filed.
- **`--no-engraving-card` on `md` or `mk`.** §6c names it for `ms` and
  `mnemonic` only, and warns that it is what makes *"no grouped form anywhere"*
  possible. Giving that hazard to two more tools is new surface nobody asked
  for. Filed.
- **`md vectors`' `chunk-set-id:` line** (`crates/md-cli/src/cmd/vectors.rs:76`).
  It writes a fixture file, not stdout, and §6a is a rule about stdout. An
  implementer who greps for the string finds two sites; only one moves.
- **`decode`, `verify` and `inspect` stdout shapes**, on any of the three. §6a.
- **`--json`, on any of the three.** §6b, unchanged and explicitly out of scope.
- **The 2-versus-64 clap-usage split.** §6f records it and declines to resolve
  it; `md` is 2 and `mk` and `mnemonic` are 64, and this phase leaves that alone.
- **`ms`.** P2's, including its GUI schema file and the 18 argv call sites in
  this repo's journey drivers.
- **Reconciling the two meanings of `--out` on `md` and `mk`.** `md vectors`,
  `md gen-man`, `mk vectors` and `mk gen-man` already carry `--out <DIR>`,
  meaning a **directory**; the two channel entries add `--out FILE`, meaning a
  file created 0600, to the same binaries. Both meanings are correct for their
  verbs and §6b names only the second, so this phase adds the flag and **does
  not** unify, rename or refuse the collision. It is recorded because the first
  draft's headline negative — *"`--in` and `--out` exist on no verb"* — was true
  only of the five verbs it swept, and a reader taking it binary-wide would not
  know the collision exists. `mnemonic-gui` already mirrors `md gen-man --out`,
  so a later unification is a four-repo change, not a two-line one.
- **Publishing `mnemonic-io-lib`.** P3 reaches it by the same rev pin P1 uses.
- **Extending the GUI's drift gate to `md`/`mk`/`ms`.** Its own header calls
  that a natural follow-on deliberately left out; this phase records the
  blindness and files it rather than building a gate mid-cycle.

---

## 7. FILED, NOT BUILT

**Nine** entries are added to `design/FOLLOWUPS.md` in this repo by this plan —
six from the first draft, three from the R0 round 0 fold. Each carries an owning
phase, per the per-phase burndown rule. **Closure condition 19 checks that they
reached disk**, because the first draft had no condition that they did.

- **F-291** — `mk`'s invalid-artifact 2 and its repair-uncorrectable 2 come out
  of the same `exit_code()` arm, so §6f's ruling as written also moves the
  repair code and breaks a parity `md`, `ms` and `mnemonic` all hold at 2.
  **Owning phase: P3**, and the mk exit code builds it. Filed as well as fixed
  because it is a defect in a GREEN spec's normative ruling, not just in code.
- **F-292** — `mnemonic`'s argv-secret surface measures to **48 advisory call
  sites across 20 source files, naming eleven distinct argv-material shapes**,
  against the five channels §7's row names. The spec already says the five are
  assertions rather than the boundary; this records what the boundary measures
  to, so a later reader cannot satisfy the row by refusing five sites out of
  forty-eight and calling it done. **Owning phase: P3**, and the mnemonic
  refusal builds it.
- **F-293** — the shipped argv advisory prints a trailing space inside the
  parenthesis, at **FOUR** call sites, not the two the first draft named. The
  draft's search took the first argument of the call, but the flag name is the
  **second** — the first is the writer — so it under-reported by half.
  Re-derived from the correct argument position, the four are
  `crates/mnemonic-toolkit/src/cmd/electrum_decrypt.rs:101`,
  `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:507`,
  `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:2331` (three passing
  `"--decrypt-password "`) and `crates/mnemonic-toolkit/src/cmd/seedqr.rs:157`
  (passing `"--digits "`). **So the residue is 44, not 46** — 48 call sites
  minus these four. A literal reading of the first draft leaves half the defect
  in place, including the one on a different flag entirely. Cosmetic, in a
  security message, in shipped code, and reproduced by running the binary.
  **Owning phase: P3**, fixed in passing by the mnemonic refusal.
- **F-294** — `records::no_records_guard`'s refusal text names `mt encode --qr`,
  another binary's flag, which makes it unusable by any consumer that is not
  `me` or `mt`. A second instance of F-276's finding that the crate's boundary
  is `me`-shaped, found by the third consumer rather than the second.
  **Owning phase: `mnemonic-io-lib`'s next version, before a sixth consumer.**
- **F-295** — `mnemonic bundle` writes 6 non-artifact lines out of 12 to stdout:
  three `#` kind comments and three blank separators, all redundant with the
  HRPs on the artifact lines. Ruled out of P3 for the reasons in the
  out-of-scope list. **Owning phase: whichever cycle extends §6a's stdout rule
  past `encode` on the four encoders.**
- **F-296 — DONE, and this plan is the beneficiary.** `plan-cite-check.sh` had
  no root for `mnemonic-gui`, so citations into the fourth repo P3 touches
  reported DANGLING and the first draft wrote them as prose. **The one-line fix
  landed on 2026-08-27**, together with leading-dot handling and `.tsv` in the
  extension list. Re-probed for this fold: the GUI's `src/schema/…`, its
  `tests/…`, its `pinned-upstream.toml` and the display-grouping corpus all
  resolve, and this document now cites them normally. **Its owning phase was P3
  and the item is burned down, not carried** — the header records what it
  changed. One residue remains true and unfixed: a bare top-level `Cargo.toml`
  citation is AMBIGUOUS across seven roots, so the GUI's must be written
  repo-qualified, which the GUI entries do.

**Three more are filed by the R0 round 0 fold:**

- **F-311** — `mk encode --keys` silently accepts a key file carrying the same
  BIP-380 record twice, at **exit 0**, emitting two byte-identical cards that
  share one chunk-set-id. Found while checking the first draft's claim that the
  card boundary is recoverable from each card's own chunk header — it is not,
  when the headers are the same header. The blank line the mk blank line entry
  deletes was the only signal that the duplicate had been accepted. **Deleting
  it is still correct under §6a**, and this is the pre-existing defect
  underneath. **Owning phase: NOT P3** — a `mk` admission ruling, out of this
  phase's row.
- **F-312** — `mnemonic-gui`'s drift gate carries a stale in-file comment,
  `tests/schema_mirror_defaults_drift.rs:36`, naming a pinned `v0.75.0` binary
  when both real pins say `v0.97.0`. **This is not cosmetic: the first draft
  read that comment and concluded the gate was blind, which was one of the four
  Importants of R0 round 0.** A comment that outlived its condition propagated a
  false premise into a plan. **Owning phase: P3**, with the toolkit release,
  which moves the pin the comment misreports anyway.
- **F-313** — `mnemonic-toolkit`'s 62 doc transcripts are byte-compared by three
  workflows that no `cargo` invocation runs, so any plan whose definition of
  green is *fmt + clippy + nextest + conformance* is structurally blind to them.
  P3 is the second cycle to define green that way. **The generalisable fix is to
  name the doc-transcript workflows in the standard closure list**, so a future
  plan inherits the surface instead of rediscovering it. **Owning phase:
  ownerless residue** — a process item, burned down with the cross-cutting
  batch.
