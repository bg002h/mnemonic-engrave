# R0 round 0 — `IMPLEMENTATION_PLAN_P3_md_mk_mnemonic.md`

**Reviewer:** independent agent (author was a different agent). **Date:** 2026-08-27.
**Artifact:** `design/IMPLEMENTATION_PLAN_P3_md_mk_mnemonic.md` @ `09c37bb`, worktree
`/scratch/code/shibboleth/_work/revp3/mnemonic-engrave`, branch `review/p3`.

## Counts

**0C / 4I / 6M / 3Nit** — NOT GREEN. The four Importants block.

## Verdict on the three-way-parallel recommendation

**SOUND WITH CONDITIONS.**

The structural bet holds where the plan argues it. Verified independently:

- **No build dependency.** `mnemonic-key/crates/mk-cli/Cargo.toml:34` → `md-codec = "0.42.0"`;
  `mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml:32-34` → `ms-codec = "0.7"`,
  `mk-codec = "0.4.1"`, `md-codec = "0.42.0"`. All crates.io **codec** pins. P3 edits CLI
  crates only. Confirmed.
- **Zero crate items added.** `origin/master:crates/mnemonic-io-lib/src/write.rs:45` already
  exposes `write_private`; `remedy.rs:79` / `:144` already expose `history_purge_recipes` /
  `history_purge_block` with the fish recipe and `tests/fish_history_purge.rs`. Nothing in P3
  asks the crate for anything new. Confirmed.
- **The shared corpus is not a collision point.** `design/display-grouping-vectors.tsv` is
  byte-identical (`7147b0ecc8cf…`) across `descriptor-mnemonic`, `mnemonic-key`,
  `mnemonic-secret`, `mnemonic-toolkit`, and none of the three separator narrowings can reach
  it: `mk-cli/src/format.rs`'s conformance test maps the keyword with a **local** `sep()`
  helper (`format.rs:105-109`), and `mnemonic-toolkit/.../display_grouping_conformance.rs:21`
  does the same with `sep_char`. Neither calls `parse_separator`. Confirmed.
- **The GUI's four schema files are disjoint**, and P2 owns only `src/schema/ms.rs`. Confirmed
  (`src/schema/{md,mk,mnemonic,ms}.rs`, one `SEPARATORS` const each, `default_value: Some("5")`
  distributed 1 / 1 / 2 / 4 exactly as claimed).
- **The toolkit's `MD_BIN`-gated cross-tool test is not a hazard**: `cli_cross_tool_differential.rs`
  invokes `md inspect --json` and `md encode … --json` only, and §6b puts `--json` out of scope.
  Its CI installs `descriptor-mnemonic-md-cli-v0.11.2` from a tag, so it is insulated anyway.

**The conditions.** Two of the plan's own joins are not parallel-safe as written, and both are
Importants below:

1. **Entry 16 cannot land green without a `mnemonic-toolkit` release + a `pinned-upstream.toml`
   bump** (I-2). That serialises `GUI mirror ← toolkit release ← the "parallel" mnemonic branch`,
   through a fourth repo, and no release step exists anywhere in the plan.
2. **The `mnemonic` branch's real scope is much larger than entry 13 + entry 14** — 19 committed
   doc transcripts must be *rewritten* and 4 goldens regenerated in a CI surface the plan's
   definition of green does not name (I-3).

One thing makes the recommendation *stronger* than the plan states: entry 1's serialised
prerequisite is already satisfied (M-6). All three branches can start now.

---

## Answer to the headline `mk` exit-code question

**The row separates them correctly, and a naive implementation CANNOT move the funds-safety 2.**
This is a clean result and it is stated first so the rest is read against it.

`crates/mk-cli/src/error.rs`:

```
112:            CliError::Codec(_) | CliError::MdCodec(_) => 2,
...
115:            CliError::SetReassemblyMismatch { .. } => 2,
```

`SetReassemblyMismatch` is its **own match arm**, textually separate. The one-line edit entry 11
describes (`:112` → `1`) cannot reach it. The four miscorrection tests
(`cli_mk1_repair_reverify.rs:178`, `:194`, `:239`, `:259` — all `.code(2)`, verified at those
exact lines) stay green under any edit to the `Codec(_)` arm. Entry 11 names them as controls
and states `SetReassemblyMismatch` stays 2. **No funds-safety regression is reachable from this
row.**

What is wrong with the row is its *evidence*, and that is I-1.

---

## IMPORTANT

### I-1 — Entry 11's exit-code census used a search form that misses two thirds of the suite's exit-code assertions, and two of the missed ones pin the arm being changed

**What the plan asserts** (§1.2 and entry 11): *"a histogram of `.code(N)` gives 12 sites, four
0, four 2, three 5, one 64"*; *"All four of the 2s are in `crates/mk-cli/tests/cli_mk1_repair_reverify.rs`
… and every one asserts `SetReassemblyMismatch`"*; *"**Zero tests pin `mk`'s invalid-artifact 2 by
exit code**, which means the change would ship unnoticed by the suite in either direction and the
gate must construct the assertion"*; entry 11's *"the four tests that must not move … are the ONLY
`.code(2)` sites in the suite"*.

**Measured.** Two assertion families exist in `crates/mk-cli/tests`, not one:

| form | sites | asserting 2 |
| --- | --- | --- |
| `assert_cmd` `.code(N)` | **12** (4×0, 4×2, 3×5, 1×64) | 4 |
| `assert_eq!(code, N)` / `assert_eq!(status.code(), Some(N))` | **24** | **2** |
| total | **36** | **6** |

The two missed 2s are both in `crates/mk-cli/tests/cli_repair.rs`, at **`:152`** and **`:185`**:

- `repair_beyond_t4_capacity_exits_2` (`:140`) — comment: *"5+ substitutions exceed t=4 capacity
  → exit 2 (CliError::Codec)"*.
- `repair_hrp_mismatch_exits_2` (`:171`) — comment: *"HRP mismatch → exit 2
  (CliError::Codec::InvalidHrp)"*.

Both pin **`CliError::Codec(_)`** — the exact arm entry 11 moves to 1.

**The counterexample.** `repair_hrp_mismatch_exits_2` pins an *invalid artifact* (a string that is
not an mk1 at all), not a repair-uncorrectable. Entry 11's prescription is *"`repair` gains `md`'s
explicit bypass — return `Ok(2)` on an **uncorrectable** input"*, and its gate asserts exactly two
things: `mk decode <garbage>` → 1, and `mk repair <a BCH-uncorrectable card>` → 2. An HRP mismatch
on `repair` is asserted by neither. So two implementations both pass the row's gate:

```
implementation A  (bypass scoped to the BCH-uncorrectable variant, as written)
  mk repair <HRP-swapped>  -> 1   ... reds cli_repair.rs:171, a test the plan says does not exist

implementation B  (bypass catches every Codec error inside repair)
  mk repair <HRP-swapped>  -> 2   ... green
```

Measured today, and `md` — the source of the transplanted shape — decides it:

```
md repair <md1 with HRP swapped to ms1>   -> 2
md decode <md1 with HRP swapped to ms1>   -> 1
mk repair <mk1 with HRP swapped to ms1>   -> 2   (pinned by cli_repair.rs:171)
mk decode <mk1 with HRP swapped to ms1>   -> 2   (unpinned; this is the one §6f moves to 1)
```

`md`'s bypass (`cmd/repair.rs:119-127`, `return Ok(2)` at `:124`) matches on **any** error from
`decode_with_correction`, not on "uncorrectable". The plan's own sentence at `:109` —
*"bypassing the `CliError::Codec → 1` default route"* — says so, and the plan quotes it without
noticing that it is wider than "uncorrectable".

**Why it is Important and not Critical.** The wrong implementation reds a test rather than shipping
silently. But the *only* reason it reds is a test the plan states does not exist, while telling the
implementer *"the change would ship unnoticed by the suite in either direction"* and *"the ONLY
`.code(2)` sites"*. An implementer who trusts that census and finds `cli_repair.rs` red has been
handed a contradiction at the exact moment they are deciding a cross-CLI exit-code parity.

**Not prescribing a remedy.** Whether `mk repair` should mirror `md`'s all-codec-errors bypass, or
whether `Codec(InvalidHrp)` on `repair` should follow `decode` to 1, is a §6f ruling this review does
not make. What is reproduced here is that the plan's row does not contain the decision and its
evidence says the decision cannot be observed.

---

### I-2 — Entry 16's "zero GUI test failures" is false, and the entry has no step that can make it true

**What the plan asserts** (§1.4, repeated in entry 16): *"**Flipping the default and deleting two
keywords therefore produces zero GUI test failures**, which is what §2a predicted and what this plan
must not let a green run be mistaken for"*, and entry 16's *"A green GUI suite is evidence about
nothing here"*.

Two of the plan's three reasons check out. The third does not, and it is the one that matters.

**Reason 1 (scope) — TRUE.** `mnemonic-gui/tests/schema_mirror_defaults_drift.rs:29`:
*"`mnemonic` only … Extending to `md`/`ms`/`mk` is a natural follow-on … deliberately out of this
cycle"*. Verified, and the `md`/`ms`/`mk` half is additionally a *one-sided* guard (`:198-206`)
that only arms once those CLIs emit a non-null `default_value`. Measured: `md gui-schema` and
`mk gui-schema` are `version: 1` and emit `default_value: null` and `choices: null` for
`--group-size` and `--separator`. So the md/mk mirror edits are genuinely invisible.

**Reason 2 (choices) — TRUE.** `:23`: the choices comparison is *"SCOPED to flags whose pinned JSON
carries NON-NULL `choices`"*, and `--separator` is the header's own named example. Measured:
`mnemonic gui-schema` reports `choices = null` for `--separator` at all four carrying subcommands.
Deleting `hyphen`/`comma` from `SEPARATORS` is invisible. Confirmed.

**Reason 3 (a stale pin) — FALSE, and the defaults half of the gate is live.**

The plan says *"Its binary comes from a pin far behind the CLI's current version."* That rests on a
stale in-file comment (`schema_mirror_defaults_drift.rs:36`: *"the pinned v0.75.0 binary"*). The
**actual** pin is `pinned-upstream.toml` `[mnemonic] tag = "mnemonic-toolkit-v0.97.0"`, and the
measured toolkit is **`mnemonic 0.97.0`** — the pin is *exactly current*, not far behind.

And the `default_value` comparison is not scoped away for `mnemonic`
(`schema_mirror_defaults_drift.rs:261-273`):

```rust
if !DEFAULT_VALUE_ALLOWLIST.contains(&(sub.name, flag.name)) {
    let hand = hand_default(flag);
    let json = jd.get(flag.name).cloned().flatten();
    if hand != json { default_drift.push(...) }
}
```

`DEFAULT_VALUE_ALLOWLIST` has **one** entry, `("compare-cost", "--feerate")`. `--group-size` is not
in it, and the file says outright: *"if this grows, the mirror or the toolkit has real drift to
reconcile, not to allowlist."*

**Counterexample, both sides measured.**

- GUI hand mirror: `src/schema/mnemonic.rs` carries `--group-size` `default_value: Some("5")` at
  **four** sites (`:332`, `:1281`, `:1960`, `:2050`).
- Pinned binary output: `mnemonic gui-schema` emits `default_value: 5` for `--group-size` at
  `bundle`, `convert`, `ms-shares-combine`, `ms-shares-split`.

Entry 16 flips the GUI side to `0`. The pinned binary still says `5`. `default_drift` gets **four**
entries and `mnemonic_defaults_and_choices_match_pinned_gui_schema` **fails**. Symmetrically, bumping
the pin to a P3 toolkit while the mirror still says `"5"` fails the same way. The gate is a *lockstep*
gate; entry 16 moves one side of it.

CI runs this: `.github/workflows/schema-mirror.yml:144` — `cargo test --workspace` with
`MNEMONIC_BIN: mnemonic` installed from `${{ steps.pins.outputs.mnemonic_tag }}` at `:60`.

**The consequence for the phase shape.** Entry 16 is not a two-line mirror edit. It needs a
`mnemonic-toolkit` **release** carrying the flipped default and a `pinned-upstream.toml` bump before
it can be green — a step that appears in no entry, no closure condition, no out-of-scope item and no
follow-up. That is an ordering dependency running `GUI ← toolkit release ← the mnemonic branch`,
i.e. through a repo the plan treats as a leaf.

---

### I-3 — Entries 13 and 14 break 23 committed, CI-byte-compared goldens in a gate the plan's definition of green does not name

**What the plan asserts.** Closure condition 14: *"All three validation surfaces are green … `cargo
fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run
--locked`, and each repo's checksum-pinned conformance step."* Entry 13's and entry 14's gates name
only in-suite assertions. `mnemonic-toolkit`'s documentation gates are not in that list.

**What exists.** `mnemonic-toolkit/docs/manual/transcripts/` holds **169 tracked files, 62 `.cmd`
transcripts**, replayed against the real installed CLIs and **byte-compared against golden `.out`**
by three workflows — `quickstart.yml` ("Verify worked-example output fidelity (real binaries) …
byte-compares against the golden `.out`"), `manual.yml`, `technical-manual.yml`. None of them is
`cargo nextest`.

**Counterexample A — entry 14 (the argv refusal) invalidates 19 goldens, and they cannot be
regenerated.** Measured: `git grep -l 'secret material on argv' -- docs/manual/transcripts` returns
**19** files, including the manual's and the quickstart's flagship bundle examples:

```
22-first-bundle.out            23-verify.out                24-recover.out
41-bundle-inheritance-cards.err 41-bundle-inheritance-json.err 41-inheritance.out
41-inspect-ms1.err  41-inspect-ms1-json.err  41-repair-ms1.err  41-repair-ms1-json.err
41-seedqr-decode.err  41-seedqr-decode-json.err  41-seedqr-encode.err  41-seedqr-encode-24.err
cross-format-recipes/recipe-2-bitcoin-core-to-bundle.err
qs-23-bundle.out  qs-24-verify.out  qs-26-recover-phrase.out  qs-41-watch-only-xpub.out
```

Every one is a documented worked example whose `.cmd` puts secret material on argv and whose golden
records the tool *warning and proceeding*. Entry 14 makes those commands **refuse**. The `.cmd`
files must be **rewritten** to the stdin channel (or `--allow-argv-secret`) and the surrounding
prose updated — that is not "regenerate the goldens", and no entry, gate or follow-up mentions it.

`22-first-bundle.cmd` verbatim:

```
$MNEMONIC_BIN bundle --network mainnet --template bip84 --slot @0.phrase="abandon abandon … about"
```

and the first line of `22-first-bundle.out`:

```
warning: secret material on argv (--slot @0.phrase=) — pipe via --slot @0.phrase=- to avoid /proc/$PID/cmdline exposure
```

**Counterexample B — entry 13 (the default flip) invalidates 4 more.** Measured: 11 `.cmd`
transcripts invoke a grouping-carrying subcommand with **no** `--group-size`, and **4** of their
goldens pin space-5 grouped output: `22-first-bundle.out`,
`41-bundle-inheritance-cards.out`, `cross-format-recipes/recipe-2-bitcoin-core-to-bundle.out`,
`qs-23-bundle.out`. Reproduced directly — `mnemonic bundle …` stdout today:

```
# ms1 (entropy, BCH-checksummed)
ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f
```

Entry 13 makes that one unbroken string. The golden is byte-compared.

**Why this is the failure mode the plan is otherwise careful about.** The plan is explicit that a
green suite must not be mistaken for evidence (entry 16), and it enumerates the engrave repo's seven
journey goldens precisely because *"the joins owned by nobody … is the defect §7 of the spec caught
twice already"*. The toolkit's own 62 goldens are the same class of artifact, inside a repo the plan
does own, and they appear nowhere.

*(Checked and clear, so the scope is bounded rather than open-ended: the 12 `.cmd` transcripts that
invoke `$MD_BIN`/`$MK_BIN` are all `decode`/`repair`/`inspect`/`address`/`derive` — §6a puts those
out of scope — and the toolkit installs `md`/`mk`/`ms` from version tags, so the md and mk branches
do not red the toolkit's CI. The exposure is entry 13's and entry 14's alone.)*

---

### I-4 — Entry 5's row covers four verbs; its gate covers three, and `md bytecode` is in neither the gate nor any closure condition

**What the plan says.** Entry 5's work column: *"route `decode`, `verify`, `inspect` and
**`bytecode`** through it"*. Its gate column: `md decode -` byte-equality, then *"Same three
assertions for `verify` and `inspect`"*. Closure condition 4: *"`md decode -`, `md verify -` and
`md inspect -` each read stdin at exit 0"*. §1.1: *"`-` is worse than absent on **three** verbs"*.

`bytecode` appears in the work and in nothing that can fail. An implementation that hoists the
reader and wires three verbs passes every stated check while leaving the fourth exactly as it is.

**Measured, with a real card on stdin** (`md encode 'wpkh(@0/<0;1>/*)'` → `md1yq pqqxq q8xtw hw4xw n4qh`):

```
md decode -    -> 1   md: codec error: codex32 decode error: string does not start with HRP md1
md inspect -   -> 1   (same message)
md bytecode -  -> 1   (same message)          <- same defect, same fix, ungated
md verify -    -> 2   error: the following required arguments were not provided: --template <TEMPLATE>
```

Two things follow.

1. `md bytecode` has the identical defect (`-` swallowed as a literal md1 positional,
   `main.rs:196`), so §1.1's "three verbs" undercounts by one and the entry's own "four" is right —
   the gate is what is short.
2. **§1.1's stated baseline for `verify` is wrong as invoked.** The plan says decode, inspect and
   verify all *"fail at **exit 1** with `codex32 decode error: string does not start with HRP md1`.
   It is not clap's `unexpected argument` at exit 2"*. Bare `md verify -` is **2**, from clap's
   missing-`--template`. It reaches 1 only with `--template` supplied:
   `md verify - --template 'wpkh(@0/<0;1>/*)'` → 1. The byte-equality gate is still the right gate
   and the plan's core insight (a "does it fail" gate passes in both worlds) is correct — but a
   `verify` gate written from the stated baseline will be written against the wrong error.

---

## MINOR

### M-1 — §3's E0425 warning omits the one module `md` and `mk` actually adopt

§3 says: *"`channel`, `exit` and `records` items are re-exported at the crate root; `fd`,
`observation` and `remedy` are reachable only as `mnemonic_io_lib::remedy::…`. … Stated because P1
hit it with a probe."*

Measured at `origin/master:crates/mnemonic-io-lib/src/lib.rs`:

```
81:pub use channel::{destination, Destination};
82:pub use exit::{write_block, WriteBlock};
83:pub use records::{no_records_guard, split_record_stream};
```

`write` is a `pub mod` (`:74`) with **no** root re-export. `write_private` therefore needs
`mnemonic_io_lib::write::write_private`; `mnemonic_io_lib::write_private` is an `E0425`. That is the
item **two of the three parallel branches** adopt (entries 6 and 10), and the paragraph written to
prevent exactly this compile error does not name it.

### M-2 — F-293 names 2 of 4 trailing-space sites, and the residue count is wrong

F-293: *"**two** call sites pass the flag name with one attached: `…/cmd/electrum_decrypt.rs:101`
and `…/cmd/import_wallet.rs:2331`. The other 46 sites pass a clean name."*

Measured over `git ls-files crates/mnemonic-toolkit/src`, taking the first string argument of each
`secret_in_argv_warning(` call — **four** sites pass a trailing space:

```
crates/mnemonic-toolkit/src/cmd/electrum_decrypt.rs:101   "--decrypt-password "
crates/mnemonic-toolkit/src/cmd/import_wallet.rs:507      "--decrypt-password "   <- not named
crates/mnemonic-toolkit/src/cmd/import_wallet.rs:2331     "--decrypt-password "
crates/mnemonic-toolkit/src/cmd/seedqr.rs:157             "--digits "             <- not named
```

So "the other 46" is **44**. Reproduced by running the binary, as F-293 requires:

```
$ mnemonic seedqr decode --digits <240 digits>
warning: secret material on argv (--digits ) — pipe via --digits - to avoid /proc/$PID/cmdline exposure
```

The plan says F-293 is *"fixed in passing by the mnemonic refusal"* and names two sites; a literal
implementation leaves half the defect. **F-292's figures, by contrast, are correct and were
re-derived**: 50 `secret_in_argv_warning(` calls under `src/` across 21 files, minus the 2 in
`secret_advisory.rs`'s own unit tests (`:118`, `:129`) = **48 call sites across 20 files**, exactly
as filed.

### M-3 — "`--in` and `--out` exist on no verb" is a claim wider than the search, and the widening matters

§1.1: *"The channels. `--in` and `--out` exist on **no** verb"*, then narrows to a sweep over five
`md` verbs. §1.2 repeats it for `mk`. Measured from each binary's own `gui-schema` (13 `md`
subcommands, 11 `mk` subcommands):

```
md vectors  --out      mk vectors  --out
md gen-man  --out      mk gen-man  --out
```

Four `--out` flags exist today on the two binaries P3 gives an `--out`, and all four mean a
**directory** (`mk --help`: *"Generate roff man pages for the CLI into a directory (`--out <DIR>`)"*).
Entries 6 and 10 add `--out FILE` meaning *a file created 0600 through `write_private`* to the same
binaries. Entry 19's decline list does not mention the collision, and `mnemonic-gui`'s
`pinned-upstream.toml` records that the GUI already mirrors `md gen-man --out`. The per-entry gates
are scoped to `encode`, so nothing is *wrong* — but the headline negative is not as wide as it reads,
and a reader taking it at face value will not know two meanings of `--out` now share a binary.

### M-4 — Entry 9's justification is falsified by an input `mk encode --keys` accepts silently

Entry 9: *"§6a's `encode` rule admits the artifact and nothing else, and **the card boundary is
recoverable from each card's own chunk header**."*

**Counterexample.** A key file with the same BIP-380 record twice:

```
$ mk encode --keys dup.keys --policy-id-stub 5b48af35     -> exit 0
mk1qp d8cwp qqsq4 … mfrjw 2
mk1qp d8cwp p806l … c36tw
<blank>
mk1qp d8cwp qqsq4 … mfrjw 2
mk1qp d8cwp p806l … c36tw
```

Two byte-identical cards sharing one chunk-set-id (`d8cwp`), separated by nothing except the blank
line entry 9 deletes. After entry 9 the boundary is not recoverable from the headers — they are the
same header — and the blank line was the only signal that `mk` had silently accepted a duplicate
cosigner record.

The downstream effect is unchanged by entry 9 (verified: `me sysw pack` skips blanks, so the
duplicate stream behaves identically with and without them), so this is a false justification rather
than a regression. **Worth filing separately, since §10's acceptance runs the `--keys` path**: that
same duplicate stream makes `me sysw pack --no-passphrase --out` classify **all four** structurally
valid mk1 chunks as *"an md1/mk1 this tool could not decode; the device will treat it as a SECRET"*
and still **exit 0**. That is a pre-existing `me`/`mk` interaction, not P3's, and it is recorded here
because P3 is the phase that walks the path.

### M-5 — Entry 2's control does not catch the failure the row credits it with

Entry 2: *"**Plus the control that must NOT move**: `a_short_policy_still_emits_a_single_string`
(`:703`) asserts the header is absent from an unchunked run and stays green, **so a change that
deleted the header entirely rather than moving it goes RED**."*

The test (`crates/md-cli/tests/cmd_encode.rs:703-720`) asserts:

```rust
assert!(!stdout.contains("chunk-set-id:"), "no chunk header for a single string");
```

An implementation that deleted the header **everywhere** leaves this **green** — it asserts absence.
What actually distinguishes "moved to stderr" from "deleted" is the row's separate final clause,
*"**And stderr must carry it**, asserted, or the chunk-set-id is simply gone"*. The gate is correct;
the reasoning attached to the control is not, and a reader who believes the control covers deletion
may treat the stderr assertion as redundant.

### M-6 — Entry 1's RED baseline is already green (measurement decay, not authoring error)

Entry 1's gate: *"`git ls-tree -d origin/master crates/` names `crates/mnemonic-io-lib` — **measured
today it names `crates/me-cli` alone**"*. §0 and §3 build the phase's shape on the same claim
(*"All three pins wait on the same push and the same SHA. That is one gate **before** all three"*).

Measured now, in this worktree:

```
$ git ls-tree -d origin/master crates/
040000 tree 14376ce9…  crates/me-cli
040000 tree 374e54c1…  crates/mnemonic-io-lib
$ git reflog show --date=iso origin/master | head -1
6c24e62 refs/remotes/origin/master@{2026-08-27 15:00:43 -0700}: update by push
```

The plan was committed at **14:56:27**; the push landed at **15:00:43**. The measurement was true
when written and false four minutes later. Two consequences worth writing down before execution:

- The row's stated RED half can no longer fail, and the plan's *"one serialised prerequisite before
  them"* shape claim no longer describes reality — all three branches can start immediately.
- **The hazard the section guarded against did not occur.** `origin/master` at `6c24e62` carries
  `crates/mnemonic-io-lib/src/write.rs:45 pub fn write_private(...)` and
  `remedy.rs:79 history_purge_recipes` / `:144 history_purge_block` with `tests/fish_history_purge.rs`.
  So *"P3 pins a crate without `write_private` in it and every channel entry here is blocked on a
  second push"* is closed, not open.

The row's other two conjuncts (a fresh-clone `cargo build --locked` in all three, and a `write_private`
call asserting the symbol exists) remain real gates.

---

## NIT

- **N-1.** *"`mk encode --keys <a two-record file>` writes **5 lines, 1 of them blank**"* is
  fixture-dependent. With a two-record file built from `mk-cli`'s own `keys_batch.rs` `KEYS` fixture
  I measure **6 lines, 1 blank** (2 chunks + blank + 3 chunks). The load-bearing fact — a blank line
  on stdout on the `--keys` path, `cmd/encode.rs:339` `println!();` — reproduces exactly, and the
  single-card control the row cites is the invariant the site's own comment at `:335-337` protects.
- **N-2.** `git ls-files design | xargs grep -l 'chunk-set-id:' | wc -l` now prints **35**, not the
  34 the plan records — the plan's own file moved it, which is precisely why the plan declines to pin
  the number. The actionable set re-derives **exactly** as listed: the seven tracked files under
  `design/journeys/` are `transcript.sh`, `transcript_pathological.sh`, `transcript_hashvault.txt`,
  `transcript_pathological.txt`, `transcript_rcw.txt`, `transcript_tr_pathological.txt`,
  `transcript_walletpolicy.txt`. `transcript.txt` is correctly *not* among them.
- **N-3.** §7 files six follow-ups but the plan's §5 closure list has 17 items and none of them is
  "the six follow-ups are written". Condition 16 checks *existing* `FOLLOWUPS.md` entries in four
  repos; nothing closes the loop on F-291…F-296 actually landing. F-296 in particular is scheduled
  *"before the GUI mirror is written, so that entry can cite what it edits"* — an ordering constraint
  that lives only in the follow-up's own text.

---

## What I checked and found CLEAN (so a later round does not re-spend the budget)

- **Citations.** Spot-checked ~28 across four repos by resolving the symbol at the cited line, not
  merely that a line exists — the F-279 class. All resolved correctly:
  `md-cli` `encode.rs:172`, `vectors.rs:76`, `main.rs:54/115/146/155/187/196/348/352`,
  `cmd/repair.rs:73/109/124`, `cmd/mod.rs:5`, `tests/cmd_encode.rs:25/652/703/724`;
  `mk-cli` `error.rs:108`, `cmd/encode.rs:78/81/339/344`, `format.rs:40/115`, `cmd/mod.rs:207`,
  `keyfile.rs:99`, `cmd/repair.rs:380`, `tests/cli_mk1_repair_reverify.rs:178/194/239/259`,
  `Cargo.toml:34`; toolkit `secret_taxonomy.rs:95`, `cmd/convert.rs:117/350/1844`,
  `cmd/bundle.rs:82`, `cmd/ms_shares.rs:76/118`, `display_grouping.rs:20/45`, `Cargo.toml:32-34`;
  engrave `design/journeys/transcript.sh:9-11` and `:37`. **Zero stale.**
- **RED-first premises, run.** `md encode --separator hyphen` → 0. `mk encode --separator hyphen`
  → 0. `md repair -` → 0 (reads stdin). `mk encode --keys` puts a blank line on stdout. `mk encode`
  default piped to `me sysw pack --out` → **4 on record 0**; `--group-size 0` → **0**. Every one
  fails today as claimed.
- **The `--from-md1-set` premise.** `mk encode --keys cosigner1.keys --from-md1 ×4 --group-size 0`
  → exit 0, two `mk1` lines, with `note: policy 38bd7cec has 2 cosigner(s); 1 of them carded here`.
  Repeating the flag over a real 4-chunk `md1` set works today, so entry 12 is ergonomics as stated.
- **§10's acceptance, P3's half, RUN.** With the header stripped and both producers ungrouped —
  i.e. simulating the post-P3 output shape — the descriptor+cosigner half passes:
  `{ cat wallet.md1 ; mk encode … } | me sysw pack --expect descriptor,cosigner --out payload.bin`
  → **exit 0**, 589-byte payload. `--expect` **already exists** on `me sysw pack` (P0 shipped it),
  and the plan's claim that `--no-passphrase` is unnecessary reproduces: exit 0 both ways.
- **`mnemonic bundle`'s stdout: 12 lines, 6 non-artifact** — reproduced exactly. The decline is
  correctly grounded.
- **The toolkit's `--group-size` surface**: 4 subcommands, one `parse_separator`
  (`display_grouping.rs:45`) reached by 4 `value_parser =` sites (`bundle.rs:85`, `convert.rs:353`,
  `ms_shares.rs:79`, `:121`). The "narrowing is one function, the default flip is four" claim holds.
  (`display_grouping.rs:113-120`'s own unit test asserts `parse_separator("hyphen") == '-'` and
  `("comma") == ','`; it must be edited, which entry 19's enumerated-diff requirement already covers.)
- **`md`'s 36 `--group-size` occurrences across 13 test files**, all carrying `0` — re-derived by
  parsing, and the 6 that a naive parse misses are prose in comments and assertion messages, all
  saying `--group-size 0`. `encode_default_groups_space_5` (`:25`) really is the only test pinning
  the default.
- **The corpus** is byte-identical `7147b0ecc8cf…` in all four repos, with `.sha256` sidecars in
  three (the toolkit has none — consistent with "three of them pin it in CI").

## Method notes

Ran `cargo build --locked` in all three subject repos before any behavioural measurement (all
Finished, exit 0; HEADs `beb2fb2a` / `c5739fc` / `8342b2ea`, matching the plan's inventory) and
measured against `target/debug/{md,mk,mnemonic}` by absolute path — never a bare `md`. Every exit
code was captured to a file and read from the file, never through a pipe. Wrote to no subject repo.

**Scope of my negatives.** "No stale citations" covers the ~28 listed above, not all 52.
"No collision among the three branches" covers: manifests and lockfiles, the shared `.tsv` and its
consumers, the four GUI schema files, the toolkit's `MD_BIN`/`MK_BIN`-gated tests and workflows, and
the engrave repo's journey drivers. I did not audit `mnemonic-secret` (P2's) or the `mt` repo.
