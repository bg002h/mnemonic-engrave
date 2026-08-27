# IMPLEMENTATION PLAN — P3: `md`, `mk` and `mnemonic` adopt the shared surface

**Status:** DRAFT v1, written 2026-08-27. **NOT reviewed.** No code may be
written until this closes an R0 round at 0C/0I.

**Gates this plan is checked by** — run each **separately** from the commit,
never on the same shell line: `scripts/plan-stepref-check.sh` (prose may not
name a step number), `scripts/plan-table-check.sh`, `scripts/plan-cite-check.sh`.

**CITATIONS INTO ALL THREE SUBJECT REPOS RESOLVE TODAY, AND THAT WAS NOT
ASSUMED.** `plan-cite-check.sh` has carried roots for `descriptor-mnemonic`,
`mnemonic-toolkit` and `mnemonic-key` since 2026-08-18, added for exactly this
reason. Verified with a 16-line probe before a line of this plan was written:
`crates/md-cli/…`, `crates/mk-cli/…` and `crates/mnemonic-toolkit/…` all
resolve **unqualified** and collide with nothing. Three forms do *not* resolve
and are handled rather than hoped over:

- `design/FOLLOWUPS.md` exists under **5** roots and reports AMBIGUOUS. Every
  `design/…` citation here is repo-qualified.
- A leading-dot path loses its dot to the citation regex, so a bare
  `.github/workflows/ci.yml` dangles as `github/…`. CI citations here are
  repo-qualified (`descriptor-mnemonic/.github/…`), which resolves.
- **`mnemonic-gui` is NOT a root**, so a citation to its `md` schema module —
  the `SEPARATORS` constant, at line 24 of that file — reports DANGLING. P3
  touches that repo. Its references are therefore written as prose without
  `path:line` punctuation, the P0 plan's workaround, used here for the one repo
  that still needs it. **The bad form is described rather than reproduced**:
  writing it out as a citation makes this document's own gate red for prose
  doing its job. See the filing below for the one-line change that would fix it,
  and the measurement showing it is collision-free.
- `.tsv` is not in the gate's extension list, so the display-grouping corpus is
  invisible to it. Its facts are measured in prose and by `sha256sum`.

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

## 0. THE SHAPE OF THE PHASE — one plan, three branches, two joins

P3 is the only phase in the cycle whose subject is **more than one repository**,
and the first question is not what to build but whether this is one plan or
three. It is one plan, three parallel implementation branches, one serialised
prerequisite before them and two serialised joins after. The evidence, in the
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

**They share one PREREQUISITE, and it belongs to P1.** `crates/mnemonic-io-lib`
is on neither crates.io nor `origin/master` — `git ls-tree -d origin/master
crates/` in this repo lists `crates/me-cli` alone. All three pins wait on the
same push and the same SHA. That is one gate **before** all three, not an
ordering **among** them.

**The two real collision points are outside the three repos.**

| join | why it cannot be three parallel commits |
| --- | --- |
| the GUI schema mirror | it is one repo, one branch. P2 edits its `ms` schema file; P3 edits its `md`, `mk` and `mnemonic` schema files. The files are disjoint and the branch is not, and two writers on one branch is what the parallel-isolation rule forbids outright |
| the seven journey goldens | the drivers resolve `md`, `mk` **and** `ms` as absolute paths into each repo's own `target/release` (`design/journeys/transcript.sh:9-11`). Regenerating needs all three P3 binaries **and** P2's `ms`, so it is the last thing in the phase and it belongs to one agent |

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

**RECOMMENDATION: one plan, three branches, run in parallel.** Three plans would
triplicate the boundary section, the dependency section and the closure list
over three documents that share one spec row, one crate prerequisite, one GUI
mirror and one golden set — and would leave the joins owned by nobody, which is
the defect §7 of the spec caught twice already.

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

**The channels.** `--in` and `--out` exist on **no** verb: a help-surface sweep
over `encode`, `decode`, `verify`, `inspect` and `repair` counts **0** matches
for either flag on any of them.

**`-` is worse than absent on three verbs, and this is the measurement that
changes the work.** `md repair -` reads stdin and exits 0. On `decode`,
`inspect` and `verify`, `-` is accepted as a **literal positional value** and
fails at **exit 1** with `codex32 decode error: string does not start with HRP
md1`. It is not clap's `unexpected argument` at exit 2, which is what `mt` did
before P1. **A gate written as "the command fails today" would pass before and
after the fix.** The gate has to be equality with the piped-content run.

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
stdout is **5 lines, one of them blank**. §2's cell reads *"non-artifact lines
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

**The channels.** `--in` and `--out` on no verb, same sweep, same **0**.
`--keys FILE` exists on `encode` alone and is read by
`keyfile::read_key_records` (`crates/mk-cli/src/keyfile.rs:99`), which takes a
path.

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

**And the four tests that pin a 2 are not the ones this work touches.** A
histogram of `.code(N)` across `crates/mk-cli/tests` gives **12** sites: four 0,
four **2**, three 5, one 64. All four of the 2s are in
`crates/mk-cli/tests/cli_mk1_repair_reverify.rs` — `:178`, `:194`, `:239`,
`:259` — and every one asserts `SetReassemblyMismatch`
(`crates/mk-cli/src/cmd/repair.rs:380`), the funds fix. **Zero tests pin `mk`'s
invalid-artifact 2 by exit code**, which means the change would ship unnoticed
by the suite in either direction and the gate must construct the assertion.

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

### 1.4 The surfaces outside the three repos

**The GUI mirror, re-measured and matching §2a exactly.** Four
`const SEPARATORS: &[&str] = &["space", "hyphen", "comma"];` declarations, one
each in the GUI's `md`, `mk`, `ms` and `mnemonic` schema modules; eight
`default_value: Some("5")` sites across the same four files, distributed 1 / 1 /
2 / 4.

**And its drift gate cannot see most of what P3 changes — for three independent
reasons, each measured.** The gate's own header scopes it to *"`mnemonic` only"*
and calls extending it to `md`/`ms`/`mk` a deliberate omission. Its choices
comparison is *"SCOPED to flags whose pinned JSON carries NON-NULL `choices`"* —
and the toolkit's `gui-schema` reports `choices = null` for `--separator` at
**all four** of its carrying subcommands, so the hyphen/comma dropdown is
compared against nothing even for `mnemonic`. Its binary comes from a pin far
behind the CLI's current version. **Flipping the default and deleting two
keywords therefore produces zero GUI test failures**, which is what §2a
predicted and what this plan must not let a green run be mistaken for.

**The seven goldens, re-derived from the spec's own command.**
`git ls-files design | xargs grep -l 'chunk-set-id:' | wc -l` prints **34**
today, against the 28 / 29 / 30 the spec records at three earlier moments — the
self-referential count the spec deliberately declines to pin, and this plan
declines to pin it too. The actionable set is the seven tracked files under
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

## 3. THE DEPENDENCY — one pin, three manifests, and it is P1's push that gates it

**`mnemonic-io-lib` is on neither distribution channel today.** Not on
crates.io, and not on `origin/master`: `git ls-tree -d origin/master crates/`
lists `crates/me-cli` alone, while local `master` is far ahead. **None of the
three can pin it until `mnemonic-engrave`'s `master` is pushed at an exact SHA**,
which is P1's push-then-pin entry, and this plan does not plan a workaround
around it. `path =` is ruled out by the spec for the reasons `me-cli`'s own
manifest already records at length: a path does not resolve in a fresh CI
checkout, and this repo currently exists at more than one filesystem location.

**The mechanism is already proven and is not re-proven here.** P1 built a
throwaway crate outside both workspaces, git-depped the library from a local
clone by rev, and called into three of its modules. P3 inherits that result;
what P3 adds is that the **same** rev is pinned three times, in
`descriptor-mnemonic/crates/md-cli/Cargo.toml`,
`mnemonic-key/crates/mk-cli/Cargo.toml` and
`mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml`.

**One consequence for the whole cycle:** P1's crate-side work — the fish recipe
and the private write — must land **before** that push, or P3 pins a crate
without `write_private` in it and every channel entry here is blocked on a
second push. P1's own table is already sequenced for a single push and a single
pin; P3 is downstream of that decision and must not create a second one.

**`lib.rs`'s root re-exports do not cover what P3 needs.** `channel`, `exit` and
`records` items are re-exported at the crate root; `fd`, `observation` and
`remedy` are reachable only as `mnemonic_io_lib::remedy::…`. `mnemonic`'s
adoption is entirely in `remedy`, so an implementer writing
`mnemonic_io_lib::history_purge_block` gets `E0425`. Stated because P1 hit it
with a probe.

---

## 4. TDD ORDER

Each entry is RED first unless its gate column says otherwise. No entry begins
until the previous entry **in its own repo** is green; the three repo columns
run in parallel, and the join entries wait on all of them. **This table is the
only ordering of record**; prose refers to work by NAME — *the pin*, *the md
header*, *the md ungrouping*, *the md separator*, *the md reader*, *the md
channels*, *the mk ungrouping*, *the mk separator*, *the mk blank line*, *the mk
channels*, *the mk exit code*, *the md1 set flag*, *the mnemonic grouping*, *the
mnemonic refusal*, *the mnemonic override*, *the GUI mirror*, *the goldens*,
*the acceptance*, *the decline* — so a renumbering cannot falsify it.

| # | repo | work | the gate that must fail first |
| --- | --- | --- | --- |
| 1 | all three | **THE PIN.** After P1's push lands `crates/mnemonic-io-lib` on `origin/master`, add `mnemonic-io-lib = { git = "https://github.com/bg002h/mnemonic-engrave", rev = "<that SHA>" }` to all three CLI manifests. One SHA, three files, no other edit. **Regression-gated, not RED-first** | `git ls-tree -d origin/master crates/` names `crates/mnemonic-io-lib` — measured today it names `crates/me-cli` alone — and `cargo build --locked` succeeds in all three repos from a **fresh** clone, not from a warm cargo cache. The crate must expose `write_private` at that SHA, asserted by a call, or the channel entries below are blocked. **This entry must not begin before P1's crate-side work is in the pushed SHA**, or the pin is done twice |
| 2 | `md` | **THE md HEADER LEAVES stdout.** `crates/md-cli/src/cmd/encode.rs:172` writes `chunk-set-id: 0x…` to stderr instead. `crates/md-cli/src/cmd/vectors.rs:76` is **not** touched — it writes a fixture file, and §6a's rule is about stdout | **fails today on two named tests**: `a_policy_over_the_single_string_cap_chunks_without_the_flag` and `force_chunked_still_chunks_a_short_policy` (`crates/md-cli/tests/cmd_encode.rs:652`, `:724`) each assert `stdout.contains("chunk-set-id:")`. **Plus the control that must NOT move**: `a_short_policy_still_emits_a_single_string` (`:703`) asserts the header is absent from an unchunked run and stays green, so a change that deleted the header entirely rather than moving it goes RED. **Plus the four helpers that strip it** — `crates/md-cli/tests/cmd_address.rs:72`, `crates/md-cli/tests/cli_output_class.rs:274`, `crates/md-cli/tests/cli_repair.rs:31`, `crates/md-cli/tests/cmd_descriptor.rs:32` — all four filter on `starts_with("md1")` rather than dropping line 0, measured, so none silently eats a chunk. **And stderr must carry it**, asserted, or the chunk-set-id is simply gone |
| 3 | `md` | **THE md STDOUT UNGROUPS, AND THE CARD APPEARS.** stdout emits the unbroken `md1` string; `--group-size` / `--separator` now shape a **stderr engraving card** whose contents this plan fixes below the table | **fails today**: `encode_default_groups_space_5` (`crates/md-cli/tests/cmd_encode.rs:25`) asserts the space/5 default on stdout. **Plus the packability assertion, measured RED four ways today** — `md encode` on a keyed 2-of-2 into `me sysw pack --out` exits **4 on record 0** with the header present and grouped, **4** with the header present and ungrouped, **4** with the header stripped and grouped, and **0 with a 391-byte payload** only when both are fixed. So this entry alone cannot make the pipeline pass and neither can the md header alone, which is the point. **Plus the 36 existing `--group-size` occurrences** across 13 files under `crates/md-cli/tests` — the value that follows is `0` at **all 36**, measured by parsing rather than by a one-line grep that would have seen only the 12 written on a single line — so all 36 keep passing under the new default and none of them is evidence that the default changed |
| 4 | `md` | **THE md SEPARATOR NARROWS** (§6c). `parse_separator` (`crates/md-cli/src/main.rs:54`) accepts whitespace only; `hyphen` and `comma` are rejected with a message naming what replaced them | **RED-first, and the test does not exist yet**: `md encode --separator hyphen` exits **0** today, measured, so the gate is a new test asserting a non-zero exit and a message. **Plus the control**: `--separator space` and a literal `" "` still work. **Plus the corpus, asserted UNCHANGED** — `sha256sum -c descriptor-mnemonic/design/display-grouping-vectors.tsv.sha256` still passes and `display_grouping_conformance` still runs its hyphen and comma rows, because those exercise `md_codec::encode::render_grouped`, which takes a `char`. A fix applied one layer too deep goes RED here |
| 5 | `md` | **THE md READER IS HOISTED AND `-` REACHES FOUR VERBS.** Move `read_md1_strings` from `crates/md-cli/src/cmd/repair.rs:73` to `crates/md-cli/src/cmd/mod.rs` beside `strip_md1_inputs` (`:5`), make it `pub`, and route `decode`, `verify`, `inspect` and `bytecode` through it — the shape `mk` already has at `crates/mk-cli/src/cmd/mod.rs:207` | **the gate is EQUALITY, not failure, and that is load-bearing**: `md decode -` with a card on stdin must produce stdout and stderr byte-equal to `md decode <that card>` at exit 0. **A "fails today" gate would pass in both worlds** — measured, `md decode -` exits **1** with `string does not start with HRP md1`, because `-` is taken as a literal md1 string. Same three assertions for `verify` and `inspect`. **Plus the control**: a non-dash positional on each verb is still parsed as a card |
| 6 | `md` | **THE md CHANNELS.** `--in FILE` on `encode` reads a BIP-388 template (§10's `md encode --in wallet.template`); `--in FILE` on the reading verbs reads md1 strings through the hoisted reader. `--out FILE` writes the artifact through the crate's `write_private` | `md encode --in <a template file> --out f` where `f` already exists at **0644** leaves `f` at **0600** holding the artifact, and `--out` suppresses nothing else. **`0o600` binds on CREATE only**, so the overwrite half is the whole of F-244 and a mode-on-create implementation goes RED. **`--out` OVERWRITES** per §6b's operator ruling, asserted by running the same command twice. **Fails today**: a help-surface sweep counts **0** `--in`/`--out` flags across all five `md` verbs |
| 7 | `mk` | **THE mk STDOUT UNGROUPS, AND THE CARD APPEARS.** Same ruling as `md`'s, at the single emission site `crates/mk-cli/src/cmd/encode.rs:344` | **fails today**: `encode_default_groups_space_5` (`crates/mk-cli/tests/encode_grouping_flags.rs:37`). **Plus the packability assertion, measured RED and GREEN today**: `mk encode` default into `me sysw pack --out` exits **4 on record 0**; the same with `--group-size 0` exits **0 with a 244-byte payload**. `mk` has no header, so grouping alone is the whole defect and the gate is a single-variable measurement |
| 8 | `mk` | **THE mk SEPARATOR NARROWS.** `parse_separator` (`crates/mk-cli/src/format.rs:40`) — note the module differs from `md`'s | **RED-first, new test**: `mk encode --separator hyphen` exits 0 today. **Plus the same corpus assertion**, against `mnemonic-key/design/display-grouping-vectors.tsv.sha256`, which pins a file **byte-identical** to `md`'s — one sha256 across four repos, measured — and whose consumer here is `crates/mk-cli/src/format.rs:115` |
| 9 | `mk` | **THE mk BLANK LINE LEAVES stdout.** `crates/mk-cli/src/cmd/encode.rs:339` prints a blank line between cards on the `--keys` path. §6a's `encode` rule admits the artifact and nothing else, and the card boundary is recoverable from each card's own chunk header | **fails today, and §2 of the spec says it cannot**: `mk encode --keys <a two-record file>` writes **5 lines, 1 of them blank**, measured. The gate is that every stdout line begins `mk1` and the blank count is **0**. **Plus the control**: single-card `mk encode` stdout is byte-identical before and after, which is the invariant the blank line's own comment was protecting |
| 10 | `mk` | **THE mk CHANNELS.** `--in FILE` on the reading verbs through `read_mk1_strings` (`crates/mk-cli/src/cmd/mod.rs:207`); `--in FILE` on `encode` routes to `keyfile::read_key_records` (`crates/mk-cli/src/keyfile.rs:99`), the reader `--keys` already uses. `--keys` is retained and `--in` is mutually exclusive with it. `--out FILE` through the crate's `write_private` | the same 0644 → 0600 create-and-overwrite pair as `md`'s channels, plus: `mk encode --in <keys file>` is byte-equal on stdout to `mk encode --keys <the same file>`, and supplying both exits 64 with a message naming both. **Fails today**: **0** `--in`/`--out` flags across all five `mk` verbs |
| 11 | `mk` | **THE mk EXIT CODE, SPLIT BEFORE IT IS MOVED** (§6f). The arm covering `CliError::Codec` and `CliError::MdCodec` jointly returns **1** at `crates/mk-cli/src/error.rs:108`; `repair` gains `md`'s explicit bypass — return `Ok(2)` on an uncorrectable input rather than propagating the error, the shape `crates/md-cli/src/cmd/repair.rs:124` already has and documents at `:109`. `SetReassemblyMismatch` stays **2** | **BOTH DIRECTIONS ARE RED TODAY AND BOTH MUST BE ASSERTED.** `mk decode <garbage>` exits **2** and must become **1**; `mk repair <a BCH-uncorrectable card>` exits **2** and must **stay** 2 — measured on a real card corrupted past `t`, `error: BCH uncorrectable: long code: more than 4 substitutions`. A one-line edit to the match arm satisfies the first and breaks the second. **Plus the four tests that must not move**: `crates/mk-cli/tests/cli_mk1_repair_reverify.rs:178`, `:194`, `:239`, `:259` all pin `SetReassemblyMismatch` at 2 and are the ONLY `.code(2)` sites in the suite — a histogram of `.code(N)` gives 12 sites, four 0, four 2, three 5, one 64, and **zero of them pin the invalid-artifact 2**. **Plus `verify` and `inspect` and `encode --from-md1 <bad>`**, each measured at 2 today, each asserted at 1 after |
| 12 | `mk` | **THE md1 SET FLAG** (§10). `--from-md1-set FILE` reads md1 strings from a file and binds the stub exactly as repeated `--from-md1` does. It **skips every line that is not an md1 string**, so a file carrying today's `chunk-set-id:` header and one written by tomorrow's `md encode --out` both work | `mk encode --from-md1-set <a 4-chunk file>` produces stdout byte-equal to the same call with four repeated `--from-md1`, measured working today at exit 0 with two `mk1` lines out. **The header-tolerance assertion is what makes this entry independent of the md header**, and it is testable now. **Plus the equivalence that removes the ordering**: fed a GROUPED md1 set and an UNGROUPED one, `mk encode` already produces byte-identical output — measured — so this flag never needs to know which era wrote the file. **Fails today**: `mk encode --help` contains `from-md1-set` **0** times |
| 13 | `mnemonic` | **THE mnemonic GROUPING SURFACE.** `--group-size` default 5 → 0 at all four declaring sites (`crates/mnemonic-toolkit/src/cmd/bundle.rs:82`, `crates/mnemonic-toolkit/src/cmd/convert.rs:350`, `crates/mnemonic-toolkit/src/cmd/ms_shares.rs:76` and `:118`); `parse_separator` (`crates/mnemonic-toolkit/src/display_grouping.rs:45`) narrows to whitespace. The `#` comment headers on `bundle`'s stdout **do not move** | **fails today on all four**: each subcommand's default emits space/5, and `--separator hyphen` exits 0. The gate asserts the ungrouped default and a non-zero exit for both retired keywords, on **each of the four**, generated rather than hand-listed. **Plus the control that pins the decline**: `mnemonic bundle`'s stdout still carries its three `#` comment lines and three blank lines — 12 lines, 6 non-artifact, measured — so an implementer who read §4 as absolute goes RED here. **Plus the corpus**, unchanged, for the same codec-level reason as `md`'s |
| 14 | `mnemonic` | **THE mnemonic ARGV REFUSAL, PRE-PARSER** (§6d). A guard over raw `std::env::args()`, before `Cli::parse()`, that matches a static flag-name table and — for the `<node>=<value>` forms — splits at `=` and tests the token against `secret_taxonomy::SECRET_NODE_TYPES_ARGV` (`crates/mnemonic-toolkit/src/secret_taxonomy.rs:95`). It reports the CLASS and the LENGTH, never the value; names `mnemonic`'s OWN private channels; and prints the crate's `remedy::history_purge_block` | **fails today on all five named channels**, each measured emitting a warning and **proceeding**: `bundle --slot @0.phrase=` and `--passphrase` (2 advisories, exit 0), `convert --passphrase` (exit 0), `derive-child --passphrase`, `restore --passphrase`, `electrum-decrypt --decrypt-password` — the last spelled `password`, not `passphrase`. **Plus the parity assertion that makes the predicate the boundary**: every token in `SECRET_NODE_TYPES_ARGV` is refused when it arrives as `--from <token>=<value>`, generated from the const so a tenth token cannot be added without the test seeing it. **Plus the two controls**: a value of `-` and a value beginning `@env:` are NOT refused, because both are existing private channels and refusing them removes the remedy. **Plus §6h's rule**, asserted: the refusal names no channel that does not exist — `--in` does not exist on any `mnemonic` verb and must not be advised |
| 15 | `mnemonic` | **THE mnemonic OVERRIDE.** `--allow-argv-secret` proceeds. Its own parse is on raw argv, before clap; when present, the override **and** the admitted token are removed from the argv handed to clap and the material is carried in through the same internal path the `-stdin` variants use | `mnemonic convert --allow-argv-secret --from phrase=<the vector> --to xpub --template bip84` exits 0 and emits stdout byte-equal to the `--from phrase=-` run with the phrase on stdin. **The naive implementation fails the SECOND assertion**: if the admitted token is left in argv for clap, an unrelated clap error echoes it — the leak §6d exists to stop, reproduced in `mt`. So the gate also asserts **no material in stderr** for the same invocation plus an unknown flag, where clap must name the flag and never the value. **Plus the control**: `--allow-argv-secret` with no secret material behaves exactly as the flagless run |
| 16 | join | **THE GUI MIRROR.** One agent, one branch, one commit in `mnemonic-gui`: the four `SEPARATORS` constants lose `hyphen` and `comma`, and the `default_value` sites for `md`, `mk` and `mnemonic` follow their CLIs. Its `ms` schema file is **P2's** and is not touched here. **Regression-gated, not RED-first** | **and the gate must be written knowing the drift gate is blind here, which was measured three ways**: the gate scopes itself to `mnemonic` only; its choices comparison is scoped to flags whose pinned JSON carries non-null `choices`, and the toolkit reports `choices = null` for `--separator` at all four carrying subcommands; and its binary comes from a pin far behind the current CLI. So the assertion is a **direct** comparison of each constant against the freshly built `mnemonic gui-schema` output, not a green suite. A green GUI suite is evidence about nothing here |
| 17 | join | **THE GOLDENS.** Regenerate the seven tracked files under `design/journeys/` in this repo, and delete the two header-stripping workarounds the drivers carry. **Regression-gated, not RED-first** | the regeneration is run, not described, and its diff is enumerated. `design/journeys/transcript.sh:37` and `design/journeys/transcript_pathological.sh:29` each explain a regex by *"`md encode` prints `chunk-set-id: 0x…` on stdout"* — both comments and both regexes go. **This entry cannot begin until P2 has landed**, because the same drivers shell out to `ms` with the seed on argv at the 18 call sites P2 owns, and regenerating before that migration means regenerating twice. **The driver binaries are `target/release`, not `target/debug`** (`design/journeys/transcript.sh:9-11`), so all three repos need a release build first or the transcripts pin stale behaviour |
| 18 | join | **THE ACCEPTANCE, RUN** (§10). The two-stage pipeline, with no `grep` and no `--group-size`, on a CHUNKING policy | `md encode --in wallet.template --out wallet.md1`, then a brace group of `cat wallet.md1`, `mk encode --in cosigner1.keys --from-md1-set wallet.md1` and `mt encode --qr --in tx.hex` into `me sysw pack --expect descriptor,cosigner,transaction --out payload.bin`, at exit 0. **And the negative half**: the same pipeline with one producer made to refuse exits non-zero and writes no payload. **A trap measured today, so the gate does not chase it**: `me sysw pack` **without** `--out`, into a shell redirect, exits **2** on a mode-0644 stdout — an unrelated F-252 refusal that would read as this cycle's defect. The pack side takes `--out`. `--no-passphrase` is **not** needed: md1 and mk1 are watch-only, so the container is not sealed, measured at exit 0 both ways |
| 19 | all three | **THE DECLINE, ASSERTED.** No code. Each tool keeps what no §6 ruling changes, and the tests that pin those are named so a later phase cannot delete them as tidying. **Regression-gated, not RED-first** | `md`, `mk` and `mnemonic` each still write to a **terminal** without refusing, so an adoption of `exit::write_block` that imported `me`'s terminal gate goes RED. `md decode`, `mk decode` and `ms decode` stdout shapes are unchanged — §6a puts them out of scope by name. `mnemonic bundle`'s `#` comment headers are still on stdout. `md` and `mk` still accept their material on **argv** with no refusal, per §4. The display-grouping corpus still checksums to `7147b0ec…` in all four repos. **Plus the enumerated diff**: every edit to the three suites — 805, 337 and 3960 tests — listed, each justified by a named §6 ruling or a numbered finding |

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

**FIVE PIECES OF WORK ARE REGRESSION-GATED RATHER THAN RED-FIRST**, and the
column header must not claim otherwise: the pin (a build resolving), the GUI
mirror (a mirror following its source), the goldens (a regeneration), the
decline (a backstop protecting RED-first work), and the *corpus-unchanged* half
of both separator entries. Everything else is RED-first, and every one of those
gates was **run** to establish that it fails — the exit codes are in this
document because they were captured to files and grepped, not inferred.

---

## 5. WHAT MUST BE TRUE TO CLOSE P3

1. **`md encode` on a CHUNKING policy pipes into `me sysw pack` with no flags
   and no `grep`, at exit 0.** §7's P3 gate in as many words. Measured RED four
   ways today — header alone, grouping alone, both, neither — so **both** the md
   header and the md ungrouping are required and neither is sufficient. Those
   two entries build it.
2. **`mk` on an invalid artifact exits 1, `mk repair` on an uncorrectable one
   still exits 2, and `SetReassemblyMismatch` still exits 2.** §6f calls the
   first the only code this cycle changes; the second and third are what stop it
   from being a one-line edit that breaks a parity three other CLIs hold and a
   funds fix four tests pin. The mk exit code builds it.
3. **`mk encode` pipes into `me sysw pack` with no flags**, on the `--keys` path
   §10 actually uses — so with the blank line gone as well as the grouping. The
   mk ungrouping and the mk blank line build it.
4. **`md decode -`, `md verify -` and `md inspect -` each read stdin at exit 0**,
   asserted as byte-equality with the positional run rather than as success,
   because `-` is currently accepted as a literal value and fails at exit 1 —
   the one failure mode a "does it fail" gate cannot distinguish. The md reader
   builds it.
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
11. **The GUI mirror matches its three CLIs, verified by direct comparison
    against a freshly built `gui-schema` and not by a green suite** — measured,
    the drift gate is blind to a `--separator` change and scoped away from
    `md`/`mk` entirely. The GUI mirror builds it.
12. **The seven goldens regenerate, and the two header-stripping workarounds in
    the drivers are gone.** The goldens build it, and it cannot start before P2
    has landed.
13. **§10's acceptance pipeline has been RUN, in both its positive and its
    negative form.** *A gate that has never executed is a hypothesis, not a
    gate.* The acceptance builds it.
14. **All three validation surfaces are green**, not the test counts alone:
    `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
    warnings`, `cargo nextest run --locked`, and each repo's checksum-pinned
    conformance step. Baseline measured 2026-08-27, every exit code read
    directly: **all nine green**, 805 / 337 / 3960 tests. Unlike P1, nothing
    here is red before the phase writes a line.
15. **(assertion, not work — no entry builds it; it is checked, not created)**
    **No exit code moves except `mk`'s invalid artifact.** `md`'s 1/2 split,
    `mnemonic`'s 1-or-2-by-input-shape, every repair code, and the 2-versus-64
    clap split are all unchanged. §6f rules all four in as many words, and a P3
    that renumbered any of them would be acting outside its ruling.
16. **(assertion, not work — checked, not created)** **`FOLLOWUPS.md` in all
    four affected repos grepped for BOTH closure vocabularies before anything is
    scheduled.** This repo closes follow-ups as `CLOSED` **and** as `DONE`, and
    a single-token sweep reports half the truth with total confidence.
17. **An R0 round closing 0C/0I.**

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
- **Publishing `mnemonic-io-lib`.** P3 reaches it by the same rev pin P1 uses.
- **Extending the GUI's drift gate to `md`/`mk`/`ms`.** Its own header calls
  that a natural follow-on deliberately left out; this phase records the
  blindness and files it rather than building a gate mid-cycle.

---

## 7. FILED, NOT BUILT

Six entries are added to `design/FOLLOWUPS.md` in this repo by this plan. Each
carries an owning phase, per the per-phase burndown rule.

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
- **F-293** — the shipped argv advisory prints `(--decrypt-password )` with a
  trailing space inside the parenthesis, because **two** call sites pass the
  flag name with one attached:
  `crates/mnemonic-toolkit/src/cmd/electrum_decrypt.rs:101` and
  `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:2331`. The other 46 sites
  pass a clean name. Cosmetic, in a security message, in shipped code, and
  reproduced by running the binary rather than by reading the source.
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
- **F-296** — `plan-cite-check.sh` has no root for `mnemonic-gui`, so every
  citation into the fourth repo P3 touches reports DANGLING and this plan writes
  them as prose. **The fix is one line** — add
  `"/scratch/code/shibboleth/mnemonic-gui"` to `ROOTS` — and it is
  collision-free, measured rather than asserted: of the GUI's 504 tracked files,
  the only paths it shares with any existing root are top-level `Cargo.toml`,
  `CHANGELOG.md`, `CLAUDE.md` and `README.md`, every one of which is **already**
  ambiguous across the current roots and already reported as such. `src/schema/`
  collides with nothing because no other root has a top-level `src/`, checked
  across all eight. **Owning phase: P3**, before the GUI mirror is written, so
  that entry can cite what it edits.
