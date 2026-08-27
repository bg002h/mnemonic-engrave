# IMPLEMENTATION PLAN — P1: `mt` adopts `mnemonic-io-lib`

**Status:** DRAFT v1, written 2026-08-27. **NOT reviewed.** No code may be
written until this closes an R0 round at 0C/0I.

**Gates this plan is checked by** — run each **separately** from the commit,
never on the same shell line: `scripts/plan-stepref-check.sh` (prose may not
name a step number), `scripts/plan-table-check.sh`, `scripts/plan-cite-check.sh`.

**CITATIONS INTO `mt` RESOLVE NOW, and that is a change from P0.** The P0 plan
wrote sibling-repo references as prose without `path:line` punctuation, because
`plan-cite-check.sh` had no root for `mnemonic-transaction` and would have
reported every one DANGLING forever. P1's subject **is** `mt`, so nearly every
anchor here is cross-repo and that workaround does not scale. The root was added
in its own commit and verified against a two-line probe. `crates/mt-cli/…` and
`crates/mt-codec/…` therefore resolve; so does `scripts/check-refusal-coverage.sh`,
which exists only in the sibling. `design/…` still resolves **here** first, which
matters because `design/SPEC_mt_v0_1.md` exists in both repos.

**Source spec:** `design/SPEC_constellation_cli_uniformity.md:1331` (P1's row and
gate), `:660` (§6b — the channels, and the `--out` ruling at `:712`), `:792`
(§6d — the argv refusal, the pre-parser ordering at `:841`, and *"`mt` gains the
override too"* at `:870`), `:992` (§6f — exit codes), `:1257` (§6h — remedy text
must be executable).

**Prior art this plan is downstream of:**
`design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` — the crate it adopts, and
the conventions this document copies.

---

## 0. Why this plan exists at all

P0 drew a boundary with **one** consumer. `me` is both the donor and the only
caller, so every line of `mnemonic-io-lib` is a line `me` wanted. P1 is where
that stops being a design claim: a second binary, in a second repository, with
different policy, either finds the boundary usable or finds it shaped like `me`.

**The most valuable output of this phase is therefore not the code. It is the
list of crate items `mt` DECLINES, and why** — see §2. Three of the crate's
seven modules fit. Two carry `me`'s policy in their shape while carrying none of
its integers, and adopting them would change what `mt` treats as a dangerous
destination, which §7 of the P0 plan rules a **ruling, never a refactor**.

---

## 1. THE INVENTORY — measured, not described

Everything below was run against
`/scratch/code/shibboleth/mnemonic-transaction/target/debug/mt` by absolute
path, on 2026-08-27, at `cf17591` (`HEAD == origin/main`, clean tree). Exit
codes were read directly, never through a pipe.

### 1.1 `mt`'s surface

Four verbs — `encode`, `decode`, `verify`, `inspect`. `decode`, `verify` and
`inspect` share **one** clap struct, `ReadArgs` (`crates/mt-cli/src/main.rs:61`);
`encode` has its own, `EncodeArgs` (`:97`). **That sharing is the single largest
simplification available to this phase**: the dash work is one field, not three.

`mt` has exactly **two** exit codes. Every refusal returns
`std::process::ExitCode::FAILURE` — **1** — and clap returns 2. That already
matches §6f's `mt` row (`design/SPEC_constellation_cli_uniformity.md:1028`), and
§6f changes only `mk`'s invalid-artifact 2, which is P3's. **There is no exit-code
work in P1.**

### 1.2 `--out` — the count, done properly

An earlier substring count of `--out` in `mt` was worthless. The real answer,
from `git grep -n -- '--out' -- crates/`: **one occurrence in the whole
repository**, and it is a refusal string —
`crates/mt-cli/src/validate.rs:680`, the sentence *"mt has no --out: stdout IS
the …, by design (§3b)"*. No clap argument named `out` exists on any verb.

### 1.3 `-` — re-measured, and the parent's figures reproduce

| verb | `mt <verb> -` today |
| --- | --- |
| `encode` | **rc 1** — accepts `-`, then refuses 0 bytes at §8.2e |
| `decode` | **rc 2** — `error: unexpected argument '-' found` |
| `verify` | **rc 2** — same |
| `inspect` | **rc 2** — same |

`encode` carries the dash as a hidden positional whose `value_parser` admits the
literal `-` and nothing else (`crates/mt-cli/src/main.rs:192`). That is F-250's
fix and the shape the three reading verbs copy.

### 1.4 `std::env::args` — present, and it is the spec's reference

**One** site: `crates/mt-cli/src/main.rs:234`, feeding
`validate::command_line_guard` at `:235`, five lines before `Cli::parse()` at
`:240`. The ordering is the whole refusal and its source says so.

### 1.5 `--allow-argv-secret` — absent

`git grep -n 'allow_argv_secret\|allow-argv-secret' -- crates/` returns
**nothing**. The flag does not exist on any `mt` verb.

### 1.6 The world-readable machinery

| | site |
| --- | --- |
| the stdout REFUSAL | `crates/mt-cli/src/validate.rs:627`, called once, from `encode` only (`crates/mt-cli/src/main.rs:701`) |
| its mask | `crates/mt-cli/src/validate.rs:653` — `mode & 0o077` |
| its wording | `crates/mt-cli/src/validate.rs:659` |
| the input WARNING | `crates/mt-cli/src/validate.rs:561`, called once, from `encode` only (`crates/mt-cli/src/main.rs:301`) |
| its mask | `crates/mt-cli/src/validate.rs:585` — the same `0o077` |
| its wording | `crates/mt-cli/src/validate.rs:589` |

The fd mechanism inside the refusal — `ManuallyDrop` on fd 1, the char-device
exemption, fail-open on a failed `fstat` — is `me`'s
`mnemonic_io_lib::fd::stdout_mode` (`crates/mnemonic-io-lib/src/fd.rs:73`)
comment sentences included.

### 1.7 The purge text

`crates/mt-cli/src/validate.rs:541`, one caller (`:487`, inside the §8.2f
remedy). It advises zsh `history -d $HISTCMD && fc -W` and fish
`history delete --contains <tx>` — the two shapes §6d disqualifies by name.

---

## 2. THE BOUNDARY — what fits, and the four things that do not

### 2.1 The verdicts

| crate item | verdict for `mt` |
| --- | --- |
| `fd::stdout_mode` (`crates/mnemonic-io-lib/src/fd.rs:73`) | **ADOPT** — byte-for-byte the block already inside `mt`'s guard |
| `fd::mode_of` (`crates/mnemonic-io-lib/src/fd.rs:46`) | **ADOPT** — and it corrects the input warning's keying, see below |
| `remedy::history_purge_block` (`crates/mnemonic-io-lib/src/remedy.rs:113`) | **ADOPT** — §6d makes it a gate item, not a courtesy |
| `remedy::history_purge_recipes` (`crates/mnemonic-io-lib/src/remedy.rs:66`) | **ADOPT**, as the block's structured half |
| `channel::destination` (`crates/mnemonic-io-lib/src/channel.rs:33`) | **ADOPT**, with `mt`'s own mapping of `Terminal` |
| `write_private` (`crates/me-cli/src/main.rs:1079`) | **MOVE IT INTO THE CRATE FIRST, then adopt** — P0 deferred this decision to P1 in as many words |
| `exit::write_block` (`crates/mnemonic-io-lib/src/exit.rs:63`) | **DECLINE** |
| `exit::WriteBlock` (`crates/mnemonic-io-lib/src/exit.rs:41`) | **DECLINE** |
| `observation::PayloadKind` (`crates/mnemonic-io-lib/src/observation.rs:47`) | **DECLINE** |
| `records::split_record_stream` (`crates/mnemonic-io-lib/src/records.rs:16`) | **DECLINE** |
| `records::no_records_guard` (`crates/mnemonic-io-lib/src/records.rs:52`) | **DECLINE** |

**So the second consumer takes 5 of the 11 public items, and 3 of the 7
modules.** That is the phase's headline finding and it is not a complaint: the
parts that fit are the parts P0 argued were mechanism, and the parts that do not
are the parts whose *shape* is policy even though their contents are not.

### 2.2 Why `exit::write_block` is declined — measured, not reasoned

`write_block`'s `Destination::Terminal` arm returns a refusal unconditionally.
**`mt` has no terminal refusal.** Run under util-linux `script` against a real
pty, `mt encode` reading the fixture transaction exits **0** and paints 1198
bytes of `mt1` strings across the terminal. `me` refuses the same destination at
exit 2 (F-253). `mt` has a terminal-aware *warning* instead —
`blocks::redirected_output_warning` and `blocks::welcome_if_tty` — and no gate.

Adopting `write_block` unchanged therefore gives `mt` a refusal it does not have.
The P0 plan's own out-of-scope section rules that **"changing what either tool
treats as a dangerous destination is a RULING, never a refactor"** and puts it
outside this cycle. The alternative — calling `write_block` with `stdout_is_tty`
hard-coded `false` — is a lie to a function about an observable fact, and the
next reader repairs it.

`WriteBlock` goes with it: its `Terminal(PayloadKind)` variant would be
unconstructible in `mt`, and a dead variant in a shared decision type is exactly
how the policy behind it gets adopted later by someone tidying up.

### 2.3 Why `observation::PayloadKind` is declined

Its two variants are `Bearer` and `CarriesNoSecret`, and `exposure_matters()`
answers the one question the mode gate may ask. **Every byte `mt` writes is
bearer** — `mt1` strings, a `tx:` record, broadcastable hex. `mt` has no
`wipe`, no fill image, nothing that could construct `CarriesNoSecret`. Adopting
the type puts a variant into `mt`'s vocabulary that nothing can produce and
makes `exposure_matters()` a constant.

`mt` already carries the axis it actually needs: `blocks::Form`, which is
`Strings` or `RawRecord` and selects the refusal's noun. That is a different
question — *what form is this artifact* rather than *does exposing it expose
anything* — and the crate holds no type for it.

**And the vocabulary F-260 actually needs is not in the crate at all.** The P0
plan's §2.3 argues for `observation.rs` from F-259 **and** F-260 jointly, but
what shipped is F-259's half: a payload-kind type. F-260's half — turning an
observed mode into words that are true of it — exists nowhere.
`me`'s own refusal (`crates/me-cli/src/main.rs:1259`) still hard-codes *"grant
read to group or others"*; that is true for `me` because `me`'s mask is `0o044`,
and false for `mt` because `mt`'s is `0o077`. So the wording work below is
written in `mt`, and the question of hoisting it is filed rather than built.

### 2.4 Why the `records` module is declined

`mt`'s reader is `read_strings::read` (`crates/mt-cli/src/read_strings.rs:57`,
698 lines). It strips grouping whitespace **within** a line, splits a single-line
blob at each `mt1` prefix, normalises case, restores an elided prefix from the
first string, and autocorrects by position. `split_record_stream` skips blank
lines and returns one record per line. It is not a simpler version of `mt`'s
reader; it is a different one, and swapping it in loses four behaviours with
tests behind them.

`no_records_guard` is refused on the same evidence and one more. `mt` already
refuses an empty stream, measured on all three reading verbs:
`mt <verb> --in <empty file>` exits **1** with
`REFUSED — §1.1e, no strings found in the input`. The crate's message advises
*"pass them on argv, with --in, or on stdin"* — and **`mt` refuses argv**, so
adopting it would print advice `mt`'s own §8.2f guard exists to prevent
following.

### 2.5 What adoption FIXES, which is the argument for doing it

`fd::mode_of` returns `None` only for a character device. The input warning
(`crates/mt-cli/src/validate.rs:585`) keys on `is_file()` instead — and `mt`'s
own source, twelve lines above at `crates/mt-cli/src/validate.rs:615`, records
that keying as measured false: a **named** fifo carries a mode (`mkfifo` gives
0666) and a third party reading it really does receive the bytes. The correction
was applied to the stdout refusal at R0 round 0 and never to the warning that
shares its mask. Adopting `mode_of` applies `mt`'s own ruling to the site it
was never applied to. **It changes a warning, never a refusal**, and that
distinction is why it is inside this phase rather than filed.

---

## 3. THE DEPENDENCY — how `mt` reaches the crate, and why it is not free

**`mnemonic-io-lib` is not on crates.io.** Checked with the P0 plan's own
protocol, control first, user-agent mandatory:

```
serde            200   <- the CONTROL; without it no 404 beneath means anything
mnemonic-io-lib  404
mnemonic_io_lib  404
```

**And it is not on `origin/master` either.** `git ls-tree -d origin/master
crates/` lists `crates/me-cli` alone; local `master` is **64** commits ahead.
So neither a version dep nor a git-rev dep resolves today.

`me-cli`'s own Cargo.toml already records the answer for the mirror-image case
and explains it at length: it takes `mt-codec` from a **GitHub rev pin** rather
than a path, because a path does not resolve in a fresh CI checkout — the two
repos are not submodules of each other — and because a rev pin keeps
`cargo publish` deferred, publishing being irreversible where pinning is not.
`mt-cli` taking `mnemonic-io-lib` from `bg002h/mnemonic-engrave` by rev is that
arrangement reflected. Both repositories are public, so no deploy key is needed.

**The mechanism was executed, not assumed.** A throwaway crate outside either
workspace, edition 2024, declared
`mnemonic-io-lib = { git = "file:///scratch/code/shibboleth/mnemonic-engrave", rev = "e799d81…" }`,
built clean and ran: `destination(true, true)` returned `File`,
`fd::mode_of` on `/dev/null` returned `None`, and `remedy::history_purge_block`
returned 1007 bytes naming `history -d` in its warning. Cargo resolved the
crate out of the workspace without pulling `me-cli` or its own `mt-codec` git
dep, so there is no fetch cycle.

**One consequence for ordering.** Every crate-side change P1 makes — the fish
recipe, `write_private` — must land **before** the push, or the rev pin needs
doing twice. The table below is sequenced for a single push and a single pin.

**`lib.rs`'s root re-exports do not cover what `mt` needs.**
`crates/mnemonic-io-lib/src/lib.rs:74` re-exports `channel`, `exit` and
`records` items at the root; `fd`, `observation` and `remedy` are reachable only
as `mnemonic_io_lib::fd::…`. That is not a defect — the modules are public — but
an implementer writing `mnemonic_io_lib::history_purge_block` gets `E0425`, as
the probe did.

---

## 4. TDD ORDER

Each entry is RED first unless its gate column says otherwise. No entry begins
until the previous is green. **This table is the only ordering of record**;
prose refers to work by NAME — *the tree-greening*, *the dash*, *the
normalisation*, *the override*, *the fish recipe*, *the private write*, *the
pin*, *the purge swap*, *the fd adoption*, *the out channel*, *the wording*,
*the decline*, *the decode warning* — so a renumbering cannot falsify it.

| # | step | the gate that must fail first |
| --- | --- | --- |
| 1 | **GREEN `mt`'s TREE BEFORE ADDING TO IT.** Run `cargo fmt` on the tree, and delete the retired `SPEC_engrave §2.2` entry from the coverage gate's `REQUIRED` list (`scripts/check-refusal-coverage.sh:130`) — the refusal it names was retired when `--record`/`--raw`/`--chunks` collapsed into `--qr`, with the reasoning already written into `crates/mt-cli/tests/refusals.toml:261`, and the gate was not updated with it. **Regression-gated, not RED-first.** | `cargo fmt --check` exits 0 and `scripts/check-refusal-coverage.sh` exits 0. **Both are RED today**, measured with the exit code read directly and not through a pipe: `fmt` exits 1 with 235 diff lines, the coverage gate exits 1 on the stale entry. **And CI proves the second was invisible**: the last run on the tip (`cf17591`) failed at `fmt` and reported every later step **skipped**, so `clippy`, `build`, the 237 tests, both refusal gates and the journeys have not executed on this commit at all. The gate for this entry is therefore not only two greens but **seven steps that RUN** |
| 2 | **`-` ON `decode`, `verify` AND `inspect`.** One `stdin_dash` field on the shared `ReadArgs` (`crates/mt-cli/src/main.rs:61`), copying `EncodeArgs`' hidden positional verbatim (`crates/mt-cli/src/main.rs:192`) — `value_name = "-"`, `value_parser = ["-"]`, `hide = true`. Three verbs, one field | each of the three: stdout **and** stderr byte-equal to the flagless run, and exit 0 — the equality assertion F-250 already uses (`crates/mt-cli/tests/encode.rs:355`), not a bare `success()`, which a `-` silently treated as a filename would also satisfy. **Fails today: each exits 2** with `error: unexpected argument '-' found`, measured. **Plus the control F-250 also carries** (`crates/mt-cli/tests/encode.rs:406`): a non-dash positional on each of the three is still an error, or the dash has opened a general positional and the pre-clap guard becomes the only thing between a mistyped argument and silent acceptance |
| 3 | **THE ARGV GUARD NORMALISES BEFORE IT CLASSIFIES** — F-274. `looks_like_a_transaction` (`crates/mt-cli/src/validate.rs:503`) lowercases for its `mt1` arm and never trims, so a whitespace-padded bearer artifact is not recognised, falls through to clap, and is echoed | a **generated cross-product**, not a hand list: 4 verbs × 2 carrier classes (an `mt1` set, a raw transaction) × 4 spellings (canonical, leading-space, trailing-space, UPPERCASE) = **32 rows**, each asserting the material never appears in stderr. **16 of the 32 leak today** — both whitespace spellings, on every verb, for both classes — measured by running the grid and counting. Canonical and uppercase are already caught, so those rows are the positive control that the guard is not simply refusing everything. **Plus the near-miss the charset test exists for**: `mt verify --in mt1-2026-08-23-cold-storage-transfer.txt` is a filename, not a carrier, and must still be accepted |
| 4 | **`--allow-argv-secret` IS A CHANNEL, NOT A FLAG** (§6d). The pre-parser layer scans raw argv; when the override is present it **removes both the override and the admitted token from the argv handed to clap** and carries the material into the run through the same internal path as `--in` content. The flag is also declared on all four verbs so `--help` documents it | `mt encode --allow-argv-secret <raw hex>` exits **0** and emits stdout byte-equal to `mt encode --in <the same hex in a file>`. **Fails today**: the flag does not exist, so the guard refuses first at rc 1. **And the naive implementation fails the SECOND assertion, which is the point**: if the admitted token is left in argv for clap, the material is echoed verbatim — reproduced with a token the guard does not classify, `error: invalid value '<99 hex chars>' for '[-]'` at rc 2 with the value in stderr. So the gate also asserts **no material in stderr for `mt encode --allow-argv-secret --nosuchflag <raw hex>`**, where clap must name the flag and never the value. **Plus the control**: `mt encode --allow-argv-secret` with no material behaves exactly as `mt encode` |
| 5 | **A FISH PURGE RECIPE, MEASURED, IN THE CRATE** (F-273). `crates/mnemonic-io-lib/src/remedy.rs` describes fish rather than prescribing it, because P0's implementer could not build a harness whose control passed. **A harness that works was built while writing this plan** and the blocker no longer stands: `script -qc "fish -i" <typescript>` fed from a command file, under an isolated `HOME`/`XDG_DATA_HOME`/`XDG_CONFIG_HOME` | **the control first**: a planted `echo mt encode <SECRET>` followed by `history save` must reach `fish_history` — measured, 1 hit. Then the emitted recipe, run in that harness, leaves **0**. `history clear-session` does: measured 0 hits. `history delete --prefix` does not: measured **rc 124, killed at a 30-second timeout, with the entry still on disk** — F-273's finding reproduced by an independent harness. **The recipe's COST is asserted too**: a neighbouring unrelated command in the same session is also removed, measured, so the emitted text must say so. **Plus the standing invariant** `me`'s suite already carries (`crates/me-cli/tests/history_purge.rs:151`): no recipe OFFERS `history -d`, while the prose still NAMES it to warn against it |
| 6 | **`write_private` MOVES INTO THE CRATE.** `crates/me-cli/src/main.rs:1079`, 21 lines, pure mechanism: `OpenOptions` with `mode(0o600)`, then `set_permissions` on the **open file** rather than the path. P0 deferred this to P1 explicitly. It names no `Class`, publishes no `EXIT_*`, and decides no policy | a unit test in the crate that a naive `std::fs::write` fails: create the target at **0644**, call it, assert the mode afterwards is **0600** and the contents are the new bytes. `0o600` binds on CREATE only, so the `set_permissions` half is the whole of F-244 and a mode-on-create implementation goes RED here. **Plus** `me`'s suite unchanged in meaning, and the crate still holding no `EXIT_*` and no `Class` |
| 7 | **PUSH, THEN PIN.** Push `mnemonic-engrave` `master` through the `ci/staging` ref so the SHA earns its required check, then add `mnemonic-io-lib = { git = "https://github.com/bg002h/mnemonic-engrave", rev = "<that SHA>" }` to `crates/mt-cli/Cargo.toml`. **FREEZE `master` for the whole window** — no commits between the staging push and the final push. **Regression-gated, not RED-first** | `git ls-tree -d origin/master crates/` names `crates/mnemonic-io-lib` — it does **not** today — and `cargo build --locked` in `mnemonic-transaction` resolves the rev from a **fresh** clone, not from a cargo cache. The final push must print no `Bypassed rule violations`. The dependency mechanism itself is already proven: a crate outside both workspaces git-depped the library from a local clone and called into three of its modules |
| 8 | **THE PURGE SWAP.** `mt`'s §8.2f remedy calls the crate's `history_purge_block`; `purge_command` (`crates/mt-cli/src/validate.rs:541`) is **deleted**, not adapted | the emitted zsh recipe, extracted from `mt`'s own stderr and **run** under a real interactive zsh on a pty, removes the planted entry — the shape `crates/me-cli/tests/history_purge.rs:108` already uses, with its harness control (`:83`) carried across so a session that records nothing cannot pass. **Fails today**: `mt` emits `history -d $HISTCMD && fc -W`, and on zsh 5.9.2 `-d` prints timestamps and the builtin rejects the invocation — it reports success and purges nothing. **Plus the fish half from the fish recipe work**, and the assertion that no recipe `mt` emits OFFERS `history -d` |
| 9 | **THE fd ADOPTION.** Both sites call `mnemonic_io_lib::fd`; the `0o077` mask stays at `mt`'s own call sites, exactly as `me` keeps `0o044` at `crates/me-cli/src/main.rs:1117`. The stdout refusal (`crates/mt-cli/src/validate.rs:627`) takes `fd::stdout_mode`; the input warning (`crates/mt-cli/src/validate.rs:561`) takes `fd::mode_of` and **loses its `is_file()` keying**, which `mt`'s own source already records as measured false | the stdout gate is unchanged where no ruling changes it, asserted as a differential with a live control: stdout mode **0600 exits 0 and writes 682 bytes**, **0620 refuses**, **0644 refuses** — all three measured today. **And the warning gains the case its keying exempts**: a **named** fifo at 0666 as the input file must WARN, which it cannot today. It stays a warning and never becomes a refusal — `mt` warns on input and refuses on output, by its own ruling |
| 10 | **THE `--out` CHANNEL on `mt encode`** (§6b). `--out FILE` writes through the crate's `write_private`; `channel::destination` decides the route, with `mt` mapping `Terminal` onto its own permissive policy rather than `me`'s refusal. The §8.2h remedy gains `--out` and loses the sentence saying `mt` has none (`crates/mt-cli/src/validate.rs:680`) | `mt encode --out f` where `f` already exists at **0644** leaves `f` at **0600** holding the artifact; `--out` suppresses the §8.2h stdout gate entirely, since `me` creates the file owner-only; and the refusal's remedy names `--out`. **THE TEST THAT CHANGES IS `the_world_readable_refusal_names_the_artifact_this_run_made`** (`crates/mt-cli/tests/tx_record.rs:304`), which pins the substrings `stdout IS the record` (`:337`) and `stdout IS the strings` (`:346`) — fragments of the sentence being replaced. **A grep of the suite for `--out` does not find it**, so the edit must be located by the fragment, not by the flag. `refuses_a_world_readable_stdout` (`crates/mt-cli/tests/refusals.rs:1498`) asserts only that the override is named and does **not** change |
| 11 | **THE WORDING, DERIVED FROM THE MODE — F-260, at BOTH sites.** The refusal at `crates/mt-cli/src/validate.rs:659` and the warning at `:589` each say the permissions grant read, hard-coded to the rule's name rather than computed from what was measured | at stdout mode **0620** the refusal must not claim read access — measured today it says *"its permissions grant read to group or others"* for a mode where `0620 & 0o044 == 0` and no read bit is set outside owner — while at **0644** it still must, and at **0600** it must not fire at all. **The same pair on the INPUT warning**, which F-260's entry does not name and which is worse there: at input mode 0620 it says *"grant read to group **and** others"*, false twice over. Both were reproduced with a 0600 control that passes, so the divergence is the mode and nothing else |
| 12 | **THE DECLINE, ASSERTED.** No code. `mt` keeps its terminal policy, its `0o077`, its own reader and its own empty-input refusal, and the tests that pin them are named so a later phase cannot delete them as tidying. **Regression-gated, not RED-first** | `mt encode` to a **real pty** still exits **0** and still emits the strings — measured today at 1198 bytes, rc 0 — so an adoption of `exit::write_block` that imported `me`'s terminal refusal goes RED here. `mt <verb> --in <empty>` still exits **1** at §1.1e on all three reading verbs. `mt` still refuses stdout mode 0620, which a `0o044` mask would permit. **Plus the enumerated diff**: every edit to `mt`'s 237 tests listed, each justified by a named §6 ruling or a numbered finding |
| 13 | **THE `decode` WARNING — F-275, the operator's ruling, WARN AND PROCEED.** `world_readable_stdout_guard` and `file_mode_warning` each have exactly one caller, both inside `encode` (`world_readable_stdout_guard` and `file_mode_warning`, cited by SYMBOL and not by line -- see the anchor warning below this table), so the three reading verbs have no gate and no warning at all. The ruling is **not** a refusal: the default umask here is 022, so a plain `mt decode > tx.hex` creates 0644, and an encode-style refusal would reject the ordinary invocation on every default machine. `decode` **warns, naming the measured mode, and still exits 0** — and `--out` is NOT added to close it | **fails today**: `mt decode > <a 0644 file>` is silent, measured at **679 bytes, rc 0, stderr empty**. After: the same run writes the same 679 bytes at **rc 0** — the exit code is asserted UNCHANGED, so a refusal smuggled in as a warning goes RED — with stderr naming the mode it measured. A **0600 control emits nothing**, so a warning that always fires cannot pass. The wording comes from `fd::stdout_mode`'s observation and not from a hard-coded rule name, exactly as the wording builds at both `encode` sites — at 0620 it must not claim read access |

**EVERY `crates/mt-cli/` LINE NUMBER IN THIS PLAN IS ANCHORED AT `cf17591`, AND
THE EARLY ROWS HAVE ALREADY MOVED THEM.** Measured 2026-08-27 against `impl/p1`
at `a4cdefa`, comparing each cited line's *content* across the two revisions:
**14 of the 15 had moved; 1 was unchanged.** The citation gate is **green on all
15**, because it checks that a line exists and not what is on it — its own header
says so in the "NOT covered" block.

One of the fourteen is the dangerous shape rather than a merely stale one. The
`validate.rs` line this plan describes as the `is_file()` keying comment holds,
at the tip, the opening of **`fn looks_like_a_transaction`** — a real function
this plan separately cites elsewhere. A reader following that number lands
somewhere plausible and wrong, which no dangling-citation check can catch. Its
number is deliberately not repeated here: writing it as a citation would make
the gate report it green, which is the very failure being described.

**So: LOCATE EVERY SITE BY SYMBOL, AND RE-MEASURE THE LINE BEFORE QUOTING IT.**
The prose names the symbol beside every line number for exactly this reason.
F-279 owns re-anchoring them when `impl/p1` merges.

**FOUR PIECES OF WORK ARE REGRESSION-GATED RATHER THAN RED-FIRST**, and the
column header must not claim otherwise: the tree-greening, the pin, the decline,
and the *unchanged-elsewhere* half of the fd adoption. The first is a repair
whose gate is *"the gates go green and the later steps run"*; the second is a
build resolving; the third and fourth are backstops that protect RED-first work
rather than proving it. Everything else is RED-first, including the
normalisation and the override, whose gates both fail today for the reasons
their cells state.

**THE `mt` TEST COUNT IS 237, MEASURED** — `cargo nextest run --locked` reports
`237 tests run: 237 passed, 0 skipped`, matching §7's P1 row. It is also **not
the validation surface**, see below.

---

## 5. WHAT MUST BE TRUE TO CLOSE P1

1. **`mt`'s WHOLE validation surface is green — all seven CI steps, not the 237
   tests alone.** §7's P1 gate names the tests and nothing else, which is a third
   of what `mt`'s CI runs: `cargo fmt --check`, `cargo clippy --all-targets
   --locked -- -D warnings`, `cargo build --locked`, `cargo nextest run
   --locked`, `scripts/check-refusal-coverage.sh`,
   `scripts/mutate-refusals.sh`, `scripts/journeys.sh`. Baseline measured
   2026-08-27 at `cf17591`, each exit code read directly:

   | gate | today |
   | --- | --- |
   | `cargo fmt --check` | **RED**, 235 diff lines |
   | `cargo clippy --all-targets --locked -- -D warnings` | green |
   | `cargo build --locked` | green |
   | `cargo nextest run --locked` | green, 237 of 237 |
   | `scripts/check-refusal-coverage.sh` | **RED**, a retired rule still required |
   | `scripts/mutate-refusals.sh` | green, all 32 refusal tests go red without their check |
   | `scripts/journeys.sh` | green, A, B in both forms, and C |

   Two of the seven are red before P1 writes a line, and the second was hidden
   behind the first by CI's fail-fast. The tree-greening builds this.
2. **`mt decode -`, `mt verify -` and `mt inspect -` each read stdin at exit 0**,
   asserted as equality with the flagless run rather than as success. The dash
   builds this.
3. **No bearer material reaches stderr for any argv carrying it as a token**, in
   any of the four spellings, on any of the four verbs — 32 rows, generated. The
   normalisation builds this.
4. **`--allow-argv-secret` exists on `mt`, and its decision AND the admitted
   material's route are both settled before `Cli::parse()`.** A guard that
   reaches its decision by parsing first has reintroduced the leak §6d exists to
   stop, and an override that hands the admitted token back to clap has created a
   new one. The override builds this.
5. **Every purge recipe `mt` emits has been RUN, in a harness with a passing
   control, and observed to purge.** Not printed — run. The fish recipe and the
   purge swap build this.
6. **`mt --out` creates its file 0600 on CREATE and on OVERWRITE.** The private
   write and the out channel build this.
7. **`mt` depends on `mnemonic-io-lib` by a rev that resolves from a fresh
   checkout**, with no path dep and no publish. The pin builds this.
8. **Every refusal or warning `mt` prints about a mode states something true of
   that mode.** The fd adoption and the wording build this.
9. **`mt`'s policy is unchanged wherever no §6 ruling changes it** — the terminal
   destination, the `0o077` mask, its own record reader, its own empty-input
   refusal. The fd adoption and the decline build this.
10. **The diff to `mt`'s 237 tests is enumerated, and each edit is justified by a
    named §6 ruling or a numbered finding.** §7's P1 gate requires this in as
    many words. The decline builds this.
11. **(assertion, not work — no step builds it; it is checked, not created)**
    **§6f needs nothing from P1.** `mt` has exactly two exit codes — 1 for every
    refusal via `ExitCode::FAILURE`, 2 from clap — which is already §6f's `mt`
    row, and the only code this cycle changes is `mk`'s, in P3. A P1 that
    renumbered anything here would be acting outside its ruling.
12. **(assertion, not work — checked, not created)** **`FOLLOWUPS.md` grepped for
    BOTH closure vocabularies before anything is scheduled.** This repo closes
    follow-ups as `CLOSED` **and** as `DONE`, and a single-token sweep reports
    half the truth with total confidence. F-273 was filed the day before this
    plan was written and F-260 the day before that, so the practical risk is low
    today and rises weekly.
14. **`mt decode` writing to a group- or other-readable stdout says so, and still exits 0.** F-275, as the operator ruled it on 2026-08-27 — a warning, never a refusal, and
    `--out` is not what closes it. The decode warning builds this.
15. **An R0 round closing 0C/0I.**

---

## 6. OUT OF SCOPE

- **Giving `mt` a terminal refusal.** §2.2. `me` refuses; `mt` warns. Changing
  that is a ruling and this phase does not make it.
- **Settling `0o044` against `0o077`.** Unchanged from P0's out-of-scope list,
  and it binds harder here: adoption is the moment the weaker rule could be
  adopted silently, on the path where the artifact is cut into metal. The mask
  stays at `mt`'s call sites.
- **`--out` on `decode`, `verify` or `inspect`.** §6b's `--out` ruling
  (`design/SPEC_constellation_cli_uniformity.md:712`) reasons entirely about the
  refusal `mt` prints today, and that refusal fires from `encode` alone. Adding
  the channel to `decode` would half-close a hazard while reading as a whole fix
  — see the filing below.
- **A world-readable gate on `mt decode`.** It is a new refusal, which is a
  ruling. Filed.
- **Changing what `--json` emits, the separator rule, or the grouped default.**
  §6c and §6b place those in P2 and P3.
- **`mnemonic-toolkit`'s adoption.** It is the sixth consumer, not P1's.
- **Publishing `mnemonic-io-lib`.** F-271 records the publish as authorised and
  its pre-flight as unrun; P1 reaches the crate by rev pin and needs no publish,
  which is the reason `me-cli` gives for pinning `mt-codec` the same way.

---

## 7. FILED, NOT BUILT

Three entries are added to `design/FOLLOWUPS.md` by this plan. Each carries an
owning phase, per the per-phase burndown rule.

- **F-274** — `mt`'s argv guard does not trim, so a whitespace-padded bearer
  artifact is unrecognised, reaches clap, and is echoed verbatim to stderr; 16
  of 32 generated rows leak today. **Owning phase: P1**, and the normalisation
  builds it. Filed as well as fixed because it is a live defect in shipped code.
  **The sibling was checked rather than assumed**: `me` refuses all four
  spellings at rc 3 with no leak, so this is `mt`-only and the analogous `me`
  defect, F-270, is already closed.
- **F-275** — `mt decode` writes broadcastable bearer hex into a mode-0644
  stdout at exit 0 with no guard and no warning, while `mt encode` refuses the
  identical destination. Measured: 679 bytes written, rc 0. The inconsistency is
  itself the hazard, because an operator who learned that `mt` refuses a
  world-readable output will believe `decode` is protected too. **RULED
  2026-08-27, and now BUILT here rather than filed**: the operator ruled *warn,
  do not refuse*, because the default umask is 022 and a refusal would reject
  the ordinary `mt decode > tx.hex` on every default machine. **Owning phase:
  P1**, and the decode warning builds it. This bullet is listed under FILED
  because it began there; the ruling moved it, and F-278 records that move.
- **F-276** — the crate's boundary is `me`-shaped in two places the second
  consumer could not use: `exit::write_block` encodes `me`'s terminal policy in
  its control flow, and `observation.rs` shipped F-259's half of its stated
  argument and not F-260's, so no vocabulary exists for describing an observed
  mode. **Owning phase: the crate's next version, before a third consumer** —
  `mnemonic-io-lib` is unpublished, so this is cheap now and expensive later.
