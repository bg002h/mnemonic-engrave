# IMPL-P3 — the `mnemonic` branch, executed

**Agent:** the `mnemonic` implementer of P3, one of three parallel branches.
**Worktree:** `/scratch/code/shibboleth/_work/p3tk/mnemonic-toolkit`, branch
`impl/p3-mnemonic`, from `8342b2e`.
**Plan:** `design/IMPLEMENTATION_PLAN_P3_md_mk_mnemonic.md` (R0 GREEN, 0C/0I),
rows **1** (the pin, mnemonic's third), **13** (the grouping surface), **14**
(the argv refusal), **15** (the override) and **20** (the decline, mnemonic's
third). Plus the vendor work the coordinator's mid-task warning added.

**No join was touched.** The toolkit release, the GUI pin bump and mirror, and
the journey goldens are all untouched, as is `mnemonic-gui`, the live
`mnemonic-toolkit` checkout, and both sibling repos.

---

## 1. Rows completed, with gate results

| row | work | gate |
| --- | --- | --- |
| **1** | the pin — `mnemonic-io-lib = { git = …, rev = 6c24e62823e6c1ac02aa3862cd6020674bf58544 }` in `crates/mnemonic-toolkit/Cargo.toml` | **regression-gated.** `cargo build --locked` exit 0; `tests/p3_io_lib_pin.rs` reaches both adopted items through `mnemonic_io_lib::remedy::`. The module-path trap is real: mutating the import to the unqualified path gives `error[E0432]: unresolved imports`, exit 101 |
| **13** | the grouping surface — `--group-size` 5 → 0 at all four declaring sites; `parse_separator` narrows to whitespace | **RED-first, and the red was the stated one.** `tests/cli_p3_grouping_surface.rs` was 5 passed / 3 failed before the source edit, then 8/8 |
| **14** | the argv refusal, pre-parser — `src/argv_guard.rs`, exit 2 | **RED-first**, plus two mutations (below). 14 integration + 16 unit tests |
| **15** | the override — `--allow-argv-secret`, parsed on raw argv **and** declared as a global clap flag | byte-equality with the stdin run; no material in stderr on an unrelated clap error; the flagless control |
| **20** | the decline, asserted | `tests/p3_declined_crate_items.rs`, 3 tests, one of them mutation-checked |
| **(added)** | vendor `mnemonic-io-lib` + a fourth source block in `ci/repro/vendor-freshness.sh` | cold-`CARGO_HOME` positive **and** negative control |

**Five commits**, in row order:

```
b262ccfa build(p3): pin mnemonic-io-lib by rev, and assert the module path compiles
9a81ecfb feat(p3)!: mnemonic ungroups by default, and the separator narrows to whitespace
058e85e0 build(p3): vendor mnemonic-io-lib and give the offline gate its fourth source block
8c201fd4 feat(p3)!: mnemonic REFUSES secret material on argv, pre-parser, with --allow-argv-secret
59ade791 test(p3): assert the DECLINE, and repair the two doc surfaces the argv row broke
```

### `git diff --stat 8342b2e..HEAD`

```
202 files changed, 4920 insertions(+), 503 deletions(-)
```

By area: **120** files under `crates/mnemonic-toolkit/tests`, **46** under
`docs/manual`, **11** the newly vendored crate, **9** under
`docs/technical-manual`, **6** under `crates/mnemonic-toolkit/src`, **4** under
`docs/quickstart`, and one each of `.examples-build/gen.sh`,
`.examples-build/Examples.md`, `Cargo.toml`, `Cargo.lock`,
`ci/repro/vendor-freshness.sh`, `CHANGELOG.md`.

### The final suite line, verbatim

```
     Summary [  46.653s] 4007 tests run: 4007 passed, 20 skipped
```

Baseline was **3960 passed, 20 skipped**. The 47 new tests are the four new
files (`p3_io_lib_pin` 2, `cli_p3_grouping_surface` 8, `cli_p3_argv_refusal` 14,
`p3_declined_crate_items` 3), `argv_guard`'s 16 in-crate unit tests, and 4 added
to existing files while rewriting the two grouping tests. The 20 skipped are
unchanged and uninvestigated, exactly as the plan records.

---

## 2. Every CI step, with its exit code

Run locally, each captured to a file and grepped — never read through a pipe.

| workflow | step | exit |
| --- | --- | --- |
| `rust.yml` | `cargo metadata --locked --format-version 1` (lockfile guard) | **0** |
| `rust.yml` | `cargo +1.95.0 fmt --all -- --check` (the pinned formatter) | **0** |
| `rust.yml` | `cargo nextest run --locked` — 4007 passed | **0** |
| `rust.yml` | `cargo clippy --all-targets -- -D warnings` | **0** |
| `rust.yml` | `cargo check --lib -p mnemonic-toolkit` | **0** |
| `rust.yml` | `sh scripts/install-msrv-guard.test.sh` | **0** |
| `rust.yml` | `sh scripts/install-man-step.test.sh` (39 pages, 0 shadow pages) | **0** |
| `rust.yml` | `mlock_g6_invariant --include-ignored` | **0** with `SIBLING_REPO_PATH` set (see §6) |
| `vendor-freshness.yml` | `bash ci/repro/vendor-freshness.sh` under an **empty** `CARGO_HOME` | **0** |
| `sibling-pin-check.yml` | its script, extracted and run | **0** |
| `manual.yml` | `docs/manual` `make verify-examples` — OK (62 transcripts) | **0** |
| `manual.yml` | `docs/manual` `make lint` (6 phases) | **0** |
| `quickstart.yml` | `docs/quickstart` `make verify-examples` — OK (62) | **0** |
| `quickstart.yml` | `docs/quickstart` `make lint` | **0** |
| `technical-manual.yml` | `docs/technical-manual` `make verify-examples` — OK (18) | **0** |
| `technical-manual.yml` | `docs/technical-manual` `make lint` | **0** |
| `examples.yml` | `gen.sh` then `git diff --exit-code -- .examples-build/Examples.md` | **0** |
| — | `docs/manual` `make html` (validates every `include=` range) | **0**, and **0** `PLACEHOLDER`s remain in the rendered HTML |

**Not runnable here, and why**, stated rather than skipped:

- **`bitcoind-differential.yml`'s `--ignored` leg.** `bitcoind` is installed but
  cannot bind an RPC endpoint in this sandbox — `Unable to bind any endpoint for
  RPC server`, and the daemon shuts down. My diff touches
  `tests/bitcoind_differential.rs` (the override migration) and
  `src/cmd/bundle.rs`, so this workflow *will* trigger. Mitigating evidence: the
  file compiles, and its **4 non-`--ignored`** tests were in the 853 that failed
  and now pass.
- **`man-pages.yml` / `repro-drift.yml` / `reproducible-musl-build.yml`.** Tag-
  and cron-triggered, Docker-based. **Already broken before P3** — see F-355,
  proven by measurement, reported and not fixed per the coordinator's
  instruction.
- `install-pin-check.yml` (tag only), `gui-pin-drift-check.yml`
  (`scripts/install.sh` only — untouched), `cross-tool-differential.yml` and
  `fuzz-smoke.yml` (path filters my diff does not hit),
  `miniscript-fork-tripwire.yml` (cron, index query).

---

## 3. The transcript comparison, and 24 rewritten versus 5 regenerated

**The plan's census was scoped to `docs/manual/transcripts` and undercounted.**
Re-measured by *running* all three replays:

| | plan | measured |
| --- | --- | --- |
| workflows replaying against the **local** binary | `quickstart.yml`, `manual-gui.yml`, `technical-manual.yml` | `manual.yml`, `quickstart.yml`, `technical-manual.yml`. **`manual-gui.yml` installs `mnemonic-toolkit-v0.74.0` from a tag** and its transcripts cannot be reddened by a branch |
| goldens invalidated by the **argv refusal** | 19 | **24** — 19 under `docs/manual/transcripts/`, **5** under `docs/technical-manual/transcripts/` |
| goldens invalidated by the **grouping flip** | 4 | **5** — the fifth is `docs/technical-manual/transcripts/mnemonic-bundle-bip84-abandon.out` |
| byte-gated doc surfaces | (3 named) | **4** — the fourth is `.examples-build/Examples.md`, gated by `examples.yml` and shipped as `docs/Examples.pdf` |

**REWRITTEN: 24.** Every `.cmd` invalidated by the refusal had its command moved
to a private channel *before* any golden was regenerated —
`printf '%s' '<material>' | … =-` where one secret is needed, and `@env:`
sentinels where two or more are (a multi-cosigner `bundle` cannot use stdin
twice; measured: *"at most one `--slot @N.<secret>=-` per invocation"*).

**REGENERATED ONLY: 5.** The grouping flip changes output, not commands, so
`22-first-bundle.out`, `41-bundle-inheritance-cards.out`,
`cross-format-recipes/recipe-2-bitcoin-core-to-bundle.out`, `qs-23-bundle.out`
and `mnemonic-bundle-bip84-abandon.out` were regenerated in row 13 with no
`.cmd` change. Four of those five are also in the 24 (their `.cmd` moved in row
14).

**Result:** `git grep -l 'secret material on argv'` over the two live transcript
trees returns **0**. The 11 hits remaining under `docs/manual-gui/transcripts/`
are replayed against the pinned v0.74.0 binary and are correct there.

Prose paired with those transcripts moved with them: every `lines="2-N"` include
range shifted to `1-N-1` (the advisory lines are gone), and one sentence the diff
falsified without touching — `41-mnemonic.md` said *"`verify-bundle` emits the
same three secret-on-argv warnings"* — was rewritten. The manual gained a new
section, **"Secret material on argv is REFUSED"**, with the channel table and the
override; the manual's own flag-coverage lint went RED without it, which is how I
learned it was required.

---

## 4. What I built, and the three places I departed from the plan's letter

### The guard

`crates/mnemonic-toolkit/src/argv_guard.rs`, bin-private, run at the top of
`main()` **before `Cli::try_parse()`**. It does not invent a recogniser: the
`<node>=` half tests against `secret_taxonomy::SECRET_NODE_TYPES_ARGV` and the
`--slot` half against `SECRET_SLOT_SUBKEYS` — the same `pub const`s the toolkit's
own `is_argv_secret_bearing` predicate is kept in lockstep with. **Neither list
is copied.** Both parity tests iterate the const, so a tenth token cannot land
without them seeing it.

**Exit 2**, `mnemonic`'s own refusal family (`ExportWalletSecretInput`,
`ModeViolation`, `ConvertRefusal`). Not 3 — that is `FutureFormat` here. Not 64 —
that is clap's usage code for a parse that never happened. No existing code
moves, per §6f closure condition 17, which `p3_declined_crate_items.rs` pins.

### Departure 1 — rows 14 and 15 are ONE commit

The plan orders them separately. They cannot be: the guard's decision is *"refuse
unless admitted"*, so the override lives inside the mechanism, and **853 existing
tests put secret material on argv**, so row 14 cannot be green until they can opt
in. Row 15's own *gates* are separately written and named. **This is a finding
about the plan's ordering, not a shortcut**, and it is why the plan's per-row
commit rule bent here and nowhere else.

### Departure 2 — three of the eleven argv shapes are NOT refused

The plan says the guard "matches a static flag-name table" and does not enumerate
it. I refuse **nine** shapes and exempt three, each exemption **measured** and
filed:

- **`--share`** (F-351) — `--share -` reads one share; a K-of-N recovery needs
  K ≥ 2. A refusal here prints a remedy that cannot be followed.
- **a positional `ms1`** (F-352) — that is §6d's *second*, value-shape layer;
  this row builds the first.
- **`--ms1` on `verify-bundle` / `import-wallet`** (F-353) — measured, `--ms1 -`
  is not accepted there and `--ms1-stdin` does not exist.

§6h forbids naming a channel that does not exist, and a refusal whose remedy
cannot be followed is worse than the advisory it replaced. The post-clap advisory
still fires on all three paths, so nothing became *quieter*.

The channel is resolved per **(subcommand, flag)** because it genuinely differs —
`--ms1 -` works on `inspect`/`repair`; `--ms1-stdin` exists **only** on the three
`xpub-search` verbs, where `--ms1 -` is taken as a literal one-character string
(exit 1); `--slot @N.x=-` works on `bundle` and `verify-bundle` but is literal on
`import-wallet`, whose channel is `@env:VAR` (verified working).

### Departure 3 — the manual's non-transcript prose is filed, not fixed

~39 further prose blocks still show the argv form (**F-356**, with the sweep that
found them). Fixed: every block paired with a byte-gated transcript. Not fixed:
the rest, because it is a pedagogical rewrite **no gate checks**, and the failure
mode of leaving it is loud and self-correcting — a copied example now produces a
refusal naming the class, the channel and the override.

### The measurement that decided the override's design

Run on this binary **before** the module existed:

```
$ mnemonic convert --from phrase=<12 words> --to xpub --template bip84 --bogus-flag
error: unexpected argument '--bogus-flag' found            <- names the FLAG
$ mnemonic convert --to xpub --template bip84 "<the 12 words>"
error: unexpected argument 'abandon abandon … about' found <- ECHOES the phrase
```

clap does **not** echo a declared flag's value and **does** echo a stray
positional. `mt` strips the admitted token out of argv because its material is
positional; `mnemonic`'s is flag-borne, so it does not — and
`an_unrelated_clap_error_names_the_flag_and_never_the_value` pins that, so a
later change that started echoing reds instead of ships. The positional echo is a
live pre-existing leak, filed as F-352.

### Mutations — the gates were made to fail

| mutation | result |
| --- | --- |
| `argv_guard::inspect` returns `Verdict::Clean` unconditionally | **27 tests run, 8 passed, 19 FAILED** |
| the guard moved **below** `Cli::try_parse()` | **14 run, 11 passed, 3 FAILED** — and the three are exactly the ordering assertions |
| a comment naming `mnemonic_io_lib::exit::write_block` appended to `src/format.rs` | **3 run, 2 passed, 1 FAILED** (the decline gate) |
| the pin's import changed to the unqualified module path | `error[E0432]`, exit **101** |

All restored; the tree is clean.

**And one gate that could not fail was found and fixed before it shipped.** The
plan's row-13 gate is *"a non-zero exit for both retired keywords"*.
`--separator` is a clap `value_parser`, so a retired keyword is exit 64 — and so
is every incomplete invocation. That gate passes in **both** worlds. The tests
assert the **message** (`invalid value 'hyphen' for '--separator'` plus the words
naming what replaced it) on invocations that are otherwise complete and exited 0
before the change.

---

## 5. The vendor work — the coordinator's warning was right, and it was a false green

`ci/repro/vendor-freshness.sh` returned **0** on this box after the pin landed,
and **1** under an empty `CARGO_HOME` on the identical tree: `--offline` stops
the network but not a warm `~/.cargo/git`. **F-359.**

Two things were needed, not one:

1. `cargo vendor vendor/` — **exactly one** new directory,
   `vendor/mnemonic-io-lib/` (11 files). Matches what the `md` branch saw.
2. A **fourth** source-replacement block in `vendor-freshness.sh`. A
   `git+…?rev=…` source has its own key and is not served by
   `source.crates-io`; measured, the 3-block form is still exit 1 *with the
   directory present*. The rev is derived from `Cargo.lock` and fails **closed**,
   mirroring the existing miniscript stanza, so a rev bump auto-tracks.

**Controls, all under a fresh empty `CARGO_HOME`:**

| | exit |
| --- | --- |
| directory vendored, 4-block config | **0** |
| the same, with `vendor/mnemonic-io-lib` moved aside | **1** — *"can't checkout … you are in the offline mode"* |
| restored | **0** |

### `vendor/miniscript` was deliberately NOT committed — and that is F-354

`cargo vendor` also rewrote **16 files** under `vendor/miniscript/`. Investigated
rather than committed:

```
Cargo.toml [patch.crates-io]  rev = ff4732e5f75aa555682343cb180fa72ee3e8e9d5
committed vendor/miniscript/nightly-version  -> nightly-2026-04-24
a fresh clone at ff4732e5, nightly-version   -> nightly-2026-05-08
```

**The committed vendored tree is a different miniscript from the one a normal
build resolves.** Re-vendoring would silently change what the reproducible
release binary compiles, in a funds-relevant dependency, outside P3's row.
Restored with `git checkout -- vendor/miniscript`; `git status -- vendor/` shows
one added directory and nothing else.

**F-355** is the other half, and matches `md`'s **F-333** exactly: `man-pages.yml`
and `repro-drift.yml` both pass `miniscript_rev: "95fdd1c5…"`, and three
`ci/repro/*.sh` default to it, while `Cargo.lock` says `ff4732e5…`. Proven
independent of P3 — with a **correct** `mnemonic-io-lib` stanza present, the
stale rev still fails on **miniscript** at exit 101 under a cold `CARGO_HOME`.
**Reported, not fixed**, per instruction. Recorded here and in F-355 that this
repo is upstream of `descriptor-mnemonic`'s release path.

---

## 6. The 853-test migration, enumerated

Adding the guard reddened **853 of 3987** tests across **116** test binaries.
Every one was a test putting secret material on argv — which is what the row
makes illegal.

They opt in via `--allow-argv-secret`, inserted at the
`Command::cargo_bin("mnemonic")` construction site: **895 insertions across 118
files**. Files with **no** failing test were not touched, so the flag marks the
tests that genuinely carry argv material rather than being sprayed everywhere.
21 sites needed a hand-written shape, because a helper returning `Command` cannot
end in `.arg()`, which returns `&mut Command`.

Two pre-existing tests pinned behaviour row 13 changes and were **rewritten, not
deleted** — `bundle_default_text_is_space_grouped_print_once` and
`ms_shares_split_default_grouped_text_json_unbroken`. Each kept the half
unrelated to grouping (print-once; the `--json`-stays-unbroken invariant) and
gained an explicit `--group-size 5` leg so the flip cannot read as the capability
being removed. `display_grouping.rs`'s unit test lost its hyphen/comma arms and
gained two: one pinning the refusal, one pinning that **intake still strips both**
so a card grouped by an older build still decodes.

---

## 7. Findings about the plan

1. **The doc-transcript census undercounts** — 24 not 19, 5 not 4, three live
   replayers but not the three named, and a **fifth** byte-gated surface
   (`.examples-build/Examples.md`) named nowhere. **F-357.**
2. **Row 1's *"three files, no other edit"* is false in this repo** — the vendor
   tree and the offline gate both needed work. 2 of 2 repos, per the coordinator.
3. **Rows 14 and 15 cannot be separate commits** (§4, Departure 1).
4. **Row 13's stated gate cannot fail as written** (§4, end).
5. **The eleven-shape boundary contains three shapes with no private channel**,
   so the row's *"static flag-name table"* cannot be complete without either
   violating §6h or building a channel first. F-351 / F-352 / F-353.
6. A smaller one: the plan says the unqualified crate path is an `E0425`. For a
   `use` declaration it is **`E0432`**; `E0425` is the expression-path form. The
   trap is real either way.

---

## 8. Consults

**None.** No ambiguity survived measurement — every question that looked like it
needed an operator ruling (`--share`'s missing channel, `--ms1`'s per-verb
channels, the override's argv routing, the miniscript vendor drift) was settled
by running the binary or the gate, and where the answer was *"this needs a
decision outside P3's row"* it became a follow-up rather than a substitution.

## 9. Follow-ups filed

`design/FOLLOWUPS.md` in `mnemonic-engrave`, commit `4ef1ce4`: **F-351** through
**F-359**. F-360..F-363 were taken by P2 while this branch ran and are untouched.
Read **F-354** first — a funds-relevant dependency whose vendored copy and pinned
rev disagree, invisible to every gate in the repo.

## 10. What I could not do

- **`bitcoind-differential.yml`'s `--ignored` leg** — the sandbox blocks the RPC
  bind (§2).
- **The ~39 remaining prose blocks** — filed as F-356 with the reason (§4,
  Departure 3).
- **`vendor/miniscript`, the tag-time repro build, and the three
  `ci/repro/*.sh` defaults** — pre-existing, funds-relevant, and outside this
  row. F-354 / F-355.
- **Every join** — untouched by design.
