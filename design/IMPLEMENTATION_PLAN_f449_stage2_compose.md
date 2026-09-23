# F-449 stage 2 — `md compose --unspendable`, and the live Liana gate

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** an operator can ask `md compose` for a Liana-importable `tr` wallet, and the claim is backed by Liana's own importer rather than by our reading of it.

**Architecture:** stage 1b already taught the codec the kind, the CLI the marker, and `md descriptor`/`--json` the rendering. Exactly one decision site is missing: `compose/tr.rs:47`'s `None => InternalKey::NumsPoint`. A new `--unspendable liana|nums` flag (default `nums`) selects between kinds there. Everything else in this stage is evidence: committing the legs stage 1b deferred because it had no CLI, and turning the live harness run into a gate anyone can re-run.

**Tech stack:** Rust (md-codec, md-cli), the committed `harnesses/liana` against a Liana **v15.0** checkout.

**Spec:** `design/SPEC_liana_unspendable_internal_key.md` (GREEN). **Recon:** `design/RECON_f449_stage2.md` — read it first; it measures that three of SPEC §9's four stage-2 items already shipped in 1b.

**Baseline revisions:** `descriptor-mnemonic` main `25acb33c` (md-codec 0.46.0, md-cli 0.18.0, 1500/1500); `mnemonic-engrave` master `2855adea` (R1 M-d — the plan, every script it invokes and the `design/FOLLOWUPS.md` holding F-636/638/640/641 live in THIS repo, so a staleness check needs both revisions or half the citations have nothing to compare against).

## Global Constraints

- **Default is `nums`.** Composing without the flag must produce byte-identical output to today, for every one of the 65 vendored vectors. This is the stage's own regression floor.
- **`liana` is not always achievable.** When `internal_key_path(list)` is `Some`, the composer extracts a real key and there is no unspendable internal key to choose. SPEC §6 row 3: that combination **WARNS**, never silently no-ops.
- **The harness is the oracle, not our opinion.** Any claim that Liana accepts or refuses a shape must come from `harnesses/liana` output committed as evidence, with a known-good control in the same run.
- **A probe that refuses on a parse error proves nothing about policy.** `"Error while parsing xkey"` is a malformed input, not a verdict. Every probe carries a control; a run whose control fails is void.
- **TWO REPOS, and every step must say which** (R0 I-1). `scripts/phase-gate.sh` and all `crates/` work are **descriptor-mnemonic**.
  `scripts/plan-api-check.sh` is **mnemonic-engrave** (measured — it exists in
  no other repo), which matters because the Self-Review makes running it a
  pre-dispatch gate.
  `design/FOLLOWUPS.md` (which holds F-636/638/640/641), `design/evidence/`,
  `harnesses/liana` and `scripts/followups-status.sh` are **mnemonic-engrave**.
  descriptor-mnemonic has its OWN `design/FOLLOWUPS.md` containing none of
  those IDs — writing to the wrong one silently loses the entry.
- **Gate every new refusal on the FLAG, never on the shape** (R0 Appendix B).
  A refusal that fires on the composer's default output turns 65 vendored
  vectors red. `--unspendable liana` is the trigger; the shape is only the
  condition.
- **Reuse the existing refusal text.** §6's messages already ship. A second
  phrasing of one rule splits every assertion that matches on it — this
  project has paid for that before.
- **A control is not sufficient for a harness probe** (R0 C-3). Liana emits
  ONE message for at least three distinct causes, including "the internal key
  does not match the recipe over THESE leaves". A control carries its own
  correct internal key, so it passes while the probe fails for a reason that
  reads as policy. **Every probe must recompute its own internal key for its
  own leaf set, and the gate must assert that before sending.**
- **The stage ships a release, and it GATES (R2 NEW-I-5).** md-codec
  `0.47.0` / md-cli `0.19.0`, with md-cli's exact pin moved in the SAME
  commit: `crates/md-cli/Cargo.toml:28` is
  `md-codec = { path = "../md-codec", version = "=0.46.0" }`, so the moment
  md-codec's version moves a split commit fails to RESOLVE — before rustc
  runs. **No md-codec tag may be cut until Task 8 lands.** Stage 1b made the
  identical rule its G-3.
- Rust-primary: this stage is Rust only. The Go port is stage 3.
- Toolchain: `export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH` — bare `cargo clippy` is 0.1.98 and fails baseline.

---

## File Structure

- **Modify** `crates/md-codec/src/compose/tr.rs:23-50` — `lower_tr` takes the requested kind; the `None =>` arm selects `NumsPoint` or `LianaUnspendable`.
- **Modify** `crates/md-codec/src/compose/mod.rs` — thread the kind through `Composed`/the lowering entry; add the "requested liana but a key was extracted" signal.
- **Modify** `crates/md-cli/src/cmd/compose.rs:590` — `run()` gains the parameter; clap gains `--unspendable`.
- **Modify** `crates/md-cli/src/main.rs:287` — the `Compose` variant, where the flag is declared (measured; `cmd/compose.rs:589-596` is `pub fn run`, R1 M-b).
- **Create** `crates/md-cli/tests/cli_compose_unspendable.rs` — the flag's own tests.
- **Create** `crates/md-cli/tests/liana_evidence_legs.rs` — C3's deferred descriptor- and address-equality legs (md-**cli**: they need the binary, and `all_cases()` is not `pub`; R2 M-j).
- **Create** `scripts/gen-compose-golden.sh` + `crates/md-cli/tests/golden/compose_pre_unspendable.json` — the pre-flag regression floor (Task 1 Step 0).
- **Modify** `crates/md-codec/Cargo.toml`, `crates/md-cli/Cargo.toml`, `CHANGELOG.md` — the release (Task 8).
- **Create** `scripts/liana-live-gate.sh` — the re-runnable §8.8 gate.
- **Create** `design/evidence/f449-stage2/` — harness input/output committed as evidence.
- **Not touched:** the wire format, `tree.rs`, `identity.rs`, `validate.rs`. Stage 1b settled those and this stage adds no wire behaviour.

---

## Task 1: `--unspendable liana|nums` reaches the one decision site

**Files:**
- Modify: `crates/md-codec/src/compose/tr.rs`, `crates/md-codec/src/compose/mod.rs`
- Modify: `crates/md-cli/src/cmd/compose.rs`, the clap declaration
- Test: `crates/md-cli/tests/cli_compose_unspendable.rs` (create)

**Interfaces produced:** `compose::UnspendableKind { Nums, Liana }` (default `Nums`); `lower_tr(list, declared, unspendable)`; `compose::run(wrapper, paths, preset, experimental, json, unspendable)`.

- [ ] **Step 0: Generate and commit the golden BEFORE anything else (R0 C-2, R1 I-g).**
  `scripts/gen-compose-golden.sh` (**descriptor-mnemonic**) runs the CURRENT
  `md` — 0.18.0, no flag — over the preset × wrapper matrix, writing
  `crates/md-cli/tests/golden/compose_pre_unspendable.json`. **Commit it in
  its own commit, before the flag exists.** The reason is provenance and it is
  sufficient on its own: a golden generated after the flag lands is blessed by
  the very code it guards — the defect C-2 was raised against. (**R2 M-g:** an
  earlier draft also argued that Step 1's `include_str!` forced the ordering.
  That include is gone — Step 1 reads the golden at runtime — so do NOT
  reintroduce it on the strength of that sentence; a missing golden must be a
  loud test failure, not a build error that invites stubbing.)

  **The generator's shape, spelled out because no gate can reach it
  (R2 M-h, N-g):**
  - **Record only exit-0 invocations.** MEASURED at `25acb33c` using the six
    presets' own valid-parameter fixtures (`cmd/compose.rs:790-799`, not
    invented): of the 24 preset × wrapper cells, **14 exit 0** — all six
    presets under `tr` and under `wsh`, plus `plain-multisig` under `sh-wsh`
    and under `sh`. The other ten exit 1 (*"legacy wrappers hold one plain
    sorted multisig only"*). Step 1's loop asserts `code == 0` for every row,
    so a generator that records all 24 turns the floor red on the commit that
    creates it.
  - **Capture stdout byte-exact.** MEASURED: `md compose` ends stdout with a
    trailing `\n` and writes `note: stdout is a keyless descriptor template
    (no keys)` to **stderr**. A shell `$(md …)` capture strips that newline
    and then every row mismatches. Redirect to a file, or write the JSON from
    Python — do not interpolate a command substitution.
  - **Refuse to run against a flag-aware binary (R1 M-f, R2 Q4a).** Provenance
    is a git-history argument and git history does not stop a second run:
    re-running this script any time after Task 1 silently re-blesses whatever
    the new binary does, including a live mutation. First line of the script —
    if `md compose --help` mentions `--unspendable`, exit non-zero naming this
    step. That is the one line that turns "asked for" into "enforced".

- [ ] **Step 1: Write the failing test — the default is byte-identical to today**

Create `crates/md-cli/tests/cli_compose_unspendable.rs`:

**R0 C-2 — the first draft of this test could not fail.** It compared
`md compose …` against `md compose … --unspendable nums`: the SAME binary
against itself. A parse bug mapping `"nums" → Liana` makes both sides emit
kind 1, the test passes, and every default `tr` compose silently ships wire
version 8. The floor must compare against **the previous release's committed
output**, which is a fact about 0.46.0 and cannot move when this binary does.

```rust
// crates/md-cli/tests/cli_compose_unspendable.rs
use std::collections::{BTreeMap, BTreeSet};
use std::process::Command as StdCommand;

/// Run `md` with `args`, returning `(stdout, stderr, exit code)`. Every call
/// site must bind and check all three — a bare `md(&[...]);` asserts nothing.
///
/// DEFINED HERE, deliberately: md-cli's tests have NO shared helper, and the
/// two that exist have DIFFERENT signatures — `cli_compose.rs:8` is
/// `fn md() -> Command` (assert_cmd), `liana_input_side.rs:33` is this one.
/// Copied verbatim from the latter rather than invented, so one vocabulary
/// covers both Liana test files. Omitting it is an E0425 the build gate
/// caught in this plan's own first draft.
fn md(args: &[&str]) -> (String, String, i32) {
    let out = StdCommand::new(assert_cmd::cargo::cargo_bin("md"))
        .args(args)
        .output()
        .expect("invoke md");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().expect("md exited normally"),
    )
}

/// The regression floor. Composing WITHOUT the flag must not move a byte,
/// for every vendored preset — this is what lets the flag ship without a
/// re-vector of the whole corpus.
#[test]
fn omitting_the_flag_is_byte_identical_to_the_previous_release() {
    // The golden is generated ONCE from md-cli 0.18.0 (pre-flag) by
    // `scripts/gen-compose-golden.sh` and committed. It is an external fact,
    // so a parse bug in THIS binary cannot move both sides of the comparison.
    // Read at RUNTIME, not `include_str!`. A compile-time include makes a
    // missing golden a BUILD error, which invites stubbing the file to get
    // green -- and a stub blessed by this binary is the exact defect the
    // golden exists to prevent. At runtime a missing golden is a loud,
    // specific test failure instead.
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/compose_pre_unspendable.json");
    let raw = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("golden missing ({e}) -- run scripts/gen-compose-golden.sh with the PRE-flag md and commit it: {path}")
    });
    let golden: BTreeMap<String, String> =
        serde_json::from_str(&raw).expect("golden parses");

    // The value after `--wrapper`, matched as a VALUE rather than by
    // `args.contains("tr")` (R2 N-h) -- a generator that ever emits
    // `--wrapper=tr` would make a contains() guard false for every row and
    // silently retire the explicit-nums comparison below.
    fn value_after<'a>(args: &[&'a str], flag: &str) -> Option<&'a str> {
        if let Some(i) = args.iter().position(|a| *a == flag) {
            return args.get(i + 1).copied();
        }
        let eq = format!("{flag}=");
        args.iter().find_map(|a| a.strip_prefix(eq.as_str()))
    }

    // Coverage by SET, not by length (R1 N-e, R2 M-h). `len() >= 6` cannot
    // tell the correct 14-row golden from a `tr`-only golden of six -- which
    // is exactly the shrunken floor a careless recovery from Task 1b would
    // produce, and it would still report green.
    let mut wrappers: BTreeSet<&str> = BTreeSet::new();
    let mut presets: BTreeSet<&str> = BTreeSet::new();
    for invocation in golden.keys() {
        let args: Vec<&str> = invocation.split(' ').collect();
        wrappers.insert(value_after(&args, "--wrapper").expect("row names a wrapper"));
        let spec = value_after(&args, "--preset").expect("row names a preset");
        presets.insert(spec.split(',').next().unwrap_or(spec));
    }
    assert_eq!(wrappers.len(), 4, "the floor lost a wrapper: {wrappers:?}");
    assert_eq!(presets.len(), 6, "the floor lost a preset: {presets:?}");
    assert_eq!(
        golden.len(),
        14,
        "the floor is the 14 exit-0 cells measured at md-cli 0.18.0, got {}",
        golden.len()
    );

    for (invocation, expected) in &golden {
        let args: Vec<&str> = invocation.split(' ').collect();
        let (out, err, code) = md(&args);
        assert_eq!(code, 0, "{invocation}: {err}");
        assert_eq!(&out, expected, "{invocation} moved against 0.18.0");
        // Explicit `nums` must equal the default -- but ONLY for `tr`.
        // RULING (R2 NEW-I-1): Task 1b refuses `--unspendable` ENTIRELY under
        // wsh/sh/sh-wsh -- the flag, not just the `liana` value -- so
        // appending it to those rows would assert exit 0 on an invocation
        // this stage deliberately makes fail. That refusal is only
        // expressible because Step 7 declares `Option<String>` with NO clap
        // default: OMITTING the flag stays exit 0 everywhere, which is what
        // keeps the eight non-`tr` rows above green.
        if value_after(&args, "--wrapper") == Some("tr") {
            let mut with = args.clone();
            with.extend_from_slice(&["--unspendable", "nums"]);
            let (out2, err2, code2) = md(&with);
            assert_eq!(code2, 0, "{invocation} --unspendable nums: {err2}");
            assert_eq!(out, out2, "{invocation}: explicit nums != default");
        }
    }
}
```

- [ ] **Step 2: Run it — expect FAIL** (`error: unexpected argument '--unspendable'`).

Run: `cargo nextest run --locked -p md-cli --test cli_compose_unspendable`

- [ ] **Step 3: Write the failing test for what the flag actually does**

Add to `crates/md-cli/tests/cli_compose_unspendable.rs`:

```rust
/// `liana` swaps the internal key's KIND and nothing else: the taptree, the
/// slot numbering and the origins are untouched. Asserted by diffing the two
/// outputs rather than by matching a shape, so any collateral change fails.
#[test]
fn liana_changes_the_internal_key_and_nothing_else() {
    let (nums, _, _) = md(&["compose", "--wrapper", "tr", "--preset",
                            "kofn-recovery,2of3,older=26280", "--unspendable", "nums"]);
    let (liana, _, code) = md(&["compose", "--wrapper", "tr", "--preset",
                                "kofn-recovery,2of3,older=26280", "--unspendable", "liana"]);
    assert_eq!(code, 0, "compose --unspendable liana failed");
    assert!(liana.contains("UNSPENDABLE(liana)"), "kind 1 not rendered: {liana}");
    assert!(!liana.contains("50929b74"), "kind 1 must not render NUMS hex: {liana}");
    // Everything after the internal key is identical.
    let tail = |s: &str| s.split_once(',').map(|(_, t)| t.to_string()).unwrap_or_default();
    assert_eq!(tail(&nums), tail(&liana), "the flag changed more than the internal key");
}
```

- [ ] **Step 4: Run it — expect FAIL.**

- [ ] **Step 5: Add the kind to the composer**

Add to `compose/mod.rs` (a FRAGMENT — an addition to an existing file, so
the gate must not assemble over it):

```rust
// crates/md-codec/src/compose/mod.rs
/// Which unspendable taproot internal key a `tr` composition should use when
/// no path supplies a real one. `Nums` is the default and the only kind any
/// released md produced before 0.46.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnspendableKind {
    /// The BIP-341 NUMS H-point. The default, and the only kind any md
    /// released before 0.47.0 could compose.
    #[default]
    Nums,
    /// Liana's own unspendable key, derived over the composed leaf set
    /// (SPEC §2). Required for Liana to import the wallet.
    Liana,
}
```

- [ ] **Step 6: Select at the one site**

In `compose/tr.rs` (a FRAGMENT — a match arm inside a struct literal, so the
build gate deliberately does not assemble it; it needs a reviewer's execution
pass):

```rust
// crates/md-codec/src/compose/tr.rs -- lower_tr gains `unspendable: UnspendableKind`
            internal_key: match ik {
                Some(_) => InternalKey::Slot(0), // numbering assigns @0; see number()
                None => match unspendable {
                    UnspendableKind::Nums => InternalKey::NumsPoint,
                    UnspendableKind::Liana => InternalKey::LianaUnspendable,
                },
            },
```

This is the ONLY behavioural line in the task. Everything else is plumbing.

- [ ] **Step 7: Thread it through `compose::run` and clap**

The declaration goes on the `Compose` variant at **`crates/md-cli/src/main.rs:287`**
(R1 M-b: `cmd/compose.rs:589-596` is `pub fn run`, not the clap struct). A
FRAGMENT — a clap attribute on a field — so the build gate does not assemble it.

**`Option<String>`, NOT `default_value` — R2 NEW-I-1, the stage's one open
decision, RULED here.** Task 1b refuses `--unspendable` under a non-`tr`
wrapper, and with `default_value = "nums"` on a bare `String` that refusal
cannot see the difference between a flag the operator typed and one clap
supplied: MEASURED, `main.rs:858` is `let cli = Cli::parse();` and the binary
holds no `ArgMatches` anywhere (`grep -n "from_arg_matches\|ArgMatches"` →
nothing), so `ValueSource::DefaultValue` is unreachable without restructuring
the whole CLI. `md compose --wrapper wsh` and
`md compose --wrapper wsh --unspendable nums` would deliver byte-identical
state to `cmd::compose::run`. Keeping the default therefore pushes the refusal
onto the DEFAULT invocation and turns **8 of the golden's 14 rows** red (`wsh`
× all six presets, `sh-wsh` × `plain-multisig`, `sh` × `plain-multisig` —
measured) — whose tempting local fix is to narrow the golden to `tr`, i.e. the
shrunken floor R1 named as the dangerous recovery. `Option<String>` with
`None => UnspendableKind::Nums` at the call site keeps the default arm
reachable and the refusal exact. The precedent sits twelve lines up:
`Compile`'s `unspendable_key: Option<String>` (`main.rs:279`).

```rust
/// Which unspendable taproot internal key to use when no spend path supplies
/// a real one: `nums` (the BIP-341 H-point — what every release before
/// md-codec 0.47.0 produced, and what OMITTING this flag still produces) or
/// `liana` (Liana's own derived key, SPEC §2 — required for Liana to import
/// the wallet). Refused under `wsh`/`sh`/`sh-wsh`, where there is no taproot
/// internal key to choose (Task 1b).
#[arg(long, value_name = "KIND")]
unspendable: Option<String>,
```

Parse it with an explicit error naming both values; do not accept a prefix or
a case variant silently. `None` maps to `UnspendableKind::Nums` at the call
site — never by a clap default, for the reason above.

- [ ] **Step 8: Run both tests — expect PASS.**

- [ ] **Step 9: Prove all THREE mutations** — re-pointed for `Option<String>`
  (R2 NEW-I-1): with no clap default, mis-parsing the *string* `"nums"` no
  longer moves the omitted-flag case, so the old two-mutation set would have
  left the outer comparison unproven.
  (a) Swap the arms in Step 6 so `Nums` yields `LianaUnspendable`
      → the **OUTER** golden comparison reddens on every `tr` row.
  (b) Map `None` to `UnspendableKind::Liana` at the call site — the default
      arm → the **OUTER** comparison reddens. This is the mutation that would
      otherwise ship wire version 8 from a flagless compose, and it is the
      reason the golden is an external artifact rather than this binary
      compared against itself.
  (c) Mis-parse `Some("nums")` to `UnspendableKind::Liana`
      → caught ONLY by the **INNER** `explicit nums != default` comparison —
      which is why that comparison must not be deleted when the non-`tr` rows
      skip it.
  **Name the assertion that reddens for each**, and paste all six results
  (applied → red, reverted → green). A mutation that applies and leaves
  everything green is a finding about the floor, not a passing step.

(The golden is Step 0; it must exist before Step 1 compiles.)

- [ ] **Step 10: Gate and commit.**

```bash
cargo nextest run --locked --all-features
./scripts/phase-gate.sh
( cd fuzz && cargo check --all-targets )
```

---

## Task 1b: the flag must not be a silent no-op anywhere it cannot apply

**R0 I-3 and I-4.** Two holes where the flag is accepted and does nothing.

- [ ] **Step 1: non-`tr` wrappers.** `lowering.rs:293-299` routes only
  `Wrapper::Tr` to `lower_tr`, so `--unspendable liana --wrapper wsh` is
  accepted and silently ignored — there is no taproot internal key at all.

  **RULING (R2 NEW-I-1): the refusal is on the FLAG, not on its value.**
  `--unspendable nums --wrapper wsh` is refused too. A flag that cannot affect
  the output is refused, not quietly honoured — and refusing only `liana`
  would leave `nums` as exactly the silent no-op this task exists to close.
  This is the same sentence as Task 1 Step 7's ruling and Task 1 Step 1's test
  comment; all three must say it, because a fold that states a remedy in one
  place and leaves the contradiction standing in another is what R1 and R2
  both caught.

  **Refuse at argument parsing**, naming the wrapper: an unspendable internal
  key is a taproot concept. Test all four wrappers × both values — and, the
  assertion that catches a refusal gated on the wrong thing, that the
  **flagless** invocation still exits 0 under each of the three non-`tr`
  wrappers. That is the same property Task 1's eight non-`tr` golden rows
  assert; asserting it here too is what makes this task's own commit safe.

- [ ] **Step 2: `--json`'s third state (§4a).** R0 I-4: 1b shipped
  `unspendable_kind` on **decode**'s JSON only. `md compose --json` still
  emits `internal_key_path: null` for BOTH kinds, so a consumer cannot tell a
  NUMS composition from a Liana one. The recon and the first draft of this
  plan both wrongly marked §4a's JSON item done. Add the third state to
  compose's JSON and bump the schema doc if the shape changes.

- [ ] **Step 3:** tests for both; prove each can fail.

- [ ] **Step 4: Gate and commit** (descriptor-mnemonic).

---

## Task 2: what compose does when `liana` cannot yield an importable wallet

**R0 C-1 and I-6, split by OWNER — the reviewer's ruling, which dissolves the
refuse-vs-warn trade-off by noticing the two cases have different authorities.**

| case | authority | action |
| --- | --- | --- |
| §6 would refuse the composed shape (e.g. `plain-multisig` → a `sortedmulti_a` leaf under kind 1, §6 row 1) | **md's own rule** | **REFUSE** |
| md-legal but Liana will not import it (`hashlock-gated`, `decaying-multisig`) | **Liana's policy model** | **WARN** |

**Why md may not refuse the second.** Refusing a wallet that is legal under
md's own rules because one coordinator declines it hard-codes Liana into md's
lowering — which SPEC §0a puts out of scope. md owns §6; it does not own
Liana's policy model.

**Why the first is not rule duplication.** `validate_unspendable_shape` is
`pub` at `validate.rs:585` and compose already holds `composed.descriptor`.
That is **one home, two callers**, and the precedent sits twelve lines from
where the call goes: the F-600 fold at `cmd/compose.rs:688-690` says of its
own read-back, *"This is not a second implementation of the rules: it is the
same function, which is what keeps the two verbs from drifting."*

**Why `md encode` refusing later is NOT sufficient** (R0, measured): `md
descriptor` shares compose's parse path and renders the refused shape at exit
0 into a **concrete fundable descriptor**. Encode gates the path to a
*backup*, not the path to *funds* — the asymmetry `cmd/compose.rs:627`
already rules on. A wallet you can fund and never back up is the worst
outcome available here.

- [ ] **Step 1: REFUSE — call the existing validator, gated on the flag.**
  Only when `--unspendable liana` was passed. Never on the default, or 65
  vendored vectors go red. Surface §6's existing message; do not re-word it.

- [ ] **Step 2: Test it** on `plain-multisig`, and assert the DEFAULT still
  composes that preset at exit 0 — the test that catches gating on the shape
  instead of the flag. **R1 M-e: as prescribed, that control cannot fail** —
  the validator is called inside the `--unspendable liana` branch, so no
  defect reachable from this task's own code can redden it. It earns its place
  only through Step 6's mutation, which hoists the call out of the gate; pair
  them in the same step, or delete the control rather than keep a green that
  means nothing.

- [ ] **Step 3: WARN — the Liana-policy cases, keyed on the SHAPE not the name.**
  **R1 I-f:** keying on preset name misses every `--path`-built equivalent.
  Measured: `--path 2of3 --path 1of1,sha256=…,older=100` composes at exit 0
  with a NUMS internal key and no preset at all, so a name-keyed warning is
  silent on exactly the hand-built wallet an operator is most likely to get
  wrong. Detect the composed shape — a hashlock leaf present, or no
  non-timelocked path — and warn on that. `hashlock-gated` and
  `decaying-multisig` then fall out as instances rather than special cases.
  Warn on stderr; exit 0; test BOTH the preset and the `--path` spelling.

  **The predicate is `policy_shape.rs`'s — and the key path is NOT a `Branch`
  (R2 M-i).** `PolicyShape` carries `key_path: KeyPathKind` separately from
  `branches`, which holds tapscript leaves only (`crates/md-codec/src/policy_shape.rs:203-206`, `:245-248` — `src/`, not
  `src/compose/`). So `!branches.iter().any(|b| b.locks.is_empty())` is **true**
  for `simple-timelocked-inheritance`, whose one leaf is timelocked and whose
  unlocked primary path is the key path. MEASURED: `md compose --wrapper tr
  --preset simple-timelocked-inheritance,older=26280 --json` →
  `"internal_key_path": 0`. Uncorrected, `--unspendable liana` on that preset
  prints this warn AND Step 4's — on the canonical Liana shape (unlocked
  primary + one timelocked recovery), which is precisely what Liana accepts.
  **The "no unlocked path" half must count the key path when it carries a real
  key, and Steps 3 and 4 must be mutually exclusive by construction:** Step 4
  fires on `ik.is_some()`, so Step 3's condition requires `ik.is_none()`.
  Test that exactly one of the two fires on `simple-timelocked-inheritance`.

- [ ] **Step 4: SPEC §6 row 3's warn** — `--unspendable liana` where
  `internal_key_path(list)` is `Some`, so a real key was extracted and there
  is no unspendable key to choose. Trigger `ik.is_some() && unspendable == Liana`
  (R0 confirmed correct for `tr`). The codec signals; the CLI prints.

- [ ] **Step 5:** `--json` must still emit valid JSON on stdout while every
  warning goes to stderr. Assert both channels.

- [ ] **Step 6:** prove each refusal and each warning can fail; paste them.
  **Include the gate mutation Step 2 depends on:** hoist Step 1's validator
  call out of the `--unspendable liana` gate so it runs on every compose, and
  show the default-`plain-multisig` control going red. That mutation is what
  the Global Constraint *"gate every new refusal on the FLAG, never on the
  shape"* is written against — and the only thing that makes the control in
  Step 2 non-vacuous.

- [ ] **Step 7: Gate and commit** (descriptor-mnemonic).

---

## Task 2c: SPEC §8.9's stage-2 row — a correction survives an UNSUPPORTED version

**R1 I-c corrected this task; my first draft could not fire.** I wrote it
against a **v8** chunk. Measured at `25acb33c`: `header.rs:36-38`'s
`is_supported_version` returns true for `WF_UNSPENDABLE_VERSION`, and a
corrupted v8 card repairs cleanly at **exit 0** keeping the correction. The
row is about a version this build does **not** support — so the reachable
trigger is a version outside the accepted set (e.g. **12**), not 8.

The fix site was wrong too: `details` are discarded inside
`decode_with_correction` — at **`chunk.rs:658`** (`decode_md1_string(&corrected_strings[0])?`)
and `:666` (`reassemble(&corrected_refs)?`), both re-grepped for this fold;
R2 N-f measured 658, not the 659 the draft carried — not in `repair.rs`.

- [ ] **Step 1: The trigger is reachable and the gap is REAL — MEASURED, so
  the self-close branch is closed (R2 §2).** Construct the chunk rather than
  reasoning about it: `codex32::unwrap_string` a real v4 md1 string
  (`codex32.rs:139`), rewrite the version nibble in the first 5-bit symbol,
  `codex32::wrap_payload` it back (`codex32.rs:82`) — both are `pub`, so this
  is ~15 lines in a test and the BCH checksum comes out valid. R2 did exactly
  that at `25acb33c`:

  ```
  $ md repair md1qzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg   # v12, 1 correctable error
  md: repair: wire-format version mismatch: got 12; accepted versions: 4, 8
  exit=2      # no repair report, no corrected string -- the correction is DISCARDED

  $ md repair md1qzfdsssjjtvyyw2fdssj54qqxppcgsc276kwwfnzntuh   # v4 control, same error
  # Repair report
  #   md1 chunk 0: 1 correction at position 0: 'q' -> '5'
  exit=5      # corrected string on stdout
  ```

  The close bar's first conjunct — *keeps the correction* — is therefore
  measurably FALSE, so **this task cannot close itself**; it is live work. Re-run
  the pair at the head of the task to confirm nothing has moved, then commit
  the two constructed strings as test fixtures so no one re-derives the recipe.

- [ ] **Step 2:** carry `details` through `chunk.rs:658`/`:666` so a
  successful correction is not thrown away when the version check fails
  afterwards. (Step 1 has already shown the gap is real; the "only if" branch
  the draft carried is gone.)

- [ ] **Step 3: the exit code — RULED: reuse `5`, mint nothing (R2 NEW-I-2).**
  MEASURED: "unsupported version after a successful correction" and "BCH
  capacity exceeded" **both exit 2 today**, differing only in the stderr text
  — and `crates/md-cli/src/cmd/repair.rs:14-18` declares the exit set a **D26
  cross-CLI parity contract with `mk repair` / `ms repair` / `mnemonic
  repair`**, three CLIs this plan's "TWO REPOS" constraint does not reach.
  SPEC:724 requires this case to report *"distinctly from the atomic-fail
  exit"*, so a message-only distinction does not satisfy the spec and the
  question cannot be dodged.

  **Resolution: exit `5` (REPAIR_APPLIED).** A correction *was* applied; `5`
  is already distinct from `2`; the code → meaning mapping the other three
  CLIs share is unchanged, so no code is minted and no other repo is touched.
  D28 (*"NO partial corrected chunks are emitted on stdout in the atomic-fail
  case"*, `repair.rs:10-12`) is likewise intact — at exit 5 this is not the
  atomic-fail case, and what reaches stdout is a complete, BCH-valid md1
  string that merely carries a version this build cannot parse, which is
  exactly what the operator needs in order to take the recovered card to a
  newer binary.

  Assert four things: stdout carries the corrected string; the exit is **5**;
  stderr names the version AND the accepted set; and the uncorrectable v4
  control still exits **2 with empty stdout**. Add one line to `repair.rs`'s
  D26 block recording that `5` covers this case, so the next reader of the
  parity contract does not read it as a divergence.
  **If the implementer concludes `5` is wrong, that is a spec escalation, not
  an implementer's choice — stop and say so.**

- [ ] **Step 4:** prove it can fail. **Step 5: Gate and commit** (descriptor-mnemonic).

---

## Task 3: SPEC §6 row 3 — folded into Task 2

R0 showed the three "what happens when `liana` cannot apply" cases must be
decided together or they get three inconsistent answers, so §6 row 3 is now
**Task 2 Step 4**. Nothing is dropped; this heading is kept only so a reader
following the R0 report's task numbers lands somewhere real.

---

## Task 4: C3's deferred evidence legs, as committed tests

Stage 1b deferred these because they needed md-cli. They are now buildable.

**Files:** `crates/md-cli/tests/liana_evidence_legs.rs` (create). **The tree
already answered where this lives (R2 M-j):**
`crates/md-codec/tests/liana_unspendable.rs:430-433` records the C3 ruling
moving the evidence-shape leg to stage 2 *"which needs `md decompose`/`md
descriptor`"* — i.e. the binary, i.e. **md-cli**. The consequence the draft
left out: `all_cases()` (`crates/md-codec/tests/common/liana.rs:108`) is **not
`pub`** and belongs to md-codec's test crate, so this file carries its own
`Case` deserializer pointed at `../md-codec/tests/fixtures/liana/cases.json`.
Write that loader. It is the one place "derive the counts from the corpus"
costs more than a filter, and skipping it is how "four" and "24" get
re-hardcoded.

**R1 I-e — do not hardcode the corpus's shape.** Task 5 adds a fifth ACCEPT
case, which falsifies any assertion naming "four" or "24". **Derive both
counts from the corpus at run time** and assert only that the set is
non-empty and that every ACCEPT case passes. A hardcoded count is precisely
what breaks the next time the corpus grows.

**And the regenerator will erase a hand-added case.**
`scripts/vendor-liana-evidence.sh` rewrites `cases.json` **wholesale** from a
hardcoded 8-name list, so Task 5's vector survives only if that script learns
it — which takes **four edits, not one**. They are enumerated in **Task 5
Step 2**, together with the re-run that proves it worked. Skip them and the
next regeneration deletes the corpus's only nested ACCEPT while these tests
stay green over the loss.

- [ ] **Step 1: Descriptor equality against every ACCEPT shape**

For each vendored case whose recorded Liana verdict is ACCEPT, assert that `md`'s rendered descriptor equals the evidence's descriptor **byte-exact including the checksum**, for every case the corpus marks ACCEPT — count derived, not written down. The whole-branch review already measured this as passing — but that measurement lives in a transcript and dies with it, which is the whole reason it becomes an assertion here. **Do not carry its "8/8" into the test (R1 N-a):** 8 is every vendored case, while this loop runs the ACCEPT subset — 4 today, 5 after Task 5. Derive it.

- [ ] **Step 2: Address equality against every ACCEPT shape**

Each ACCEPT case records Liana's own first three receive and change addresses. Assert md derives the same set — six per ACCEPT case, count derived from the corpus. Use the recorded values as the oracle — do not recompute them from the descriptor with the same code under test, which would assert only that md agrees with itself.

- [ ] **Step 3: Run both — expect PASS** (the data already agrees; these tests pin it).

- [ ] **Step 4: Prove they can fail.** Perturb one recorded address by one character and one descriptor by one character; each must go red naming the case. Restore. Paste.

- [ ] **Step 5: Gate and commit.**

---

## Task 5: the live Liana gate, and F-640 CLOSED

**Files:** `harnesses/liana/` + `design/evidence/f449-stage2/` + `design/FOLLOWUPS.md` (**mnemonic-engrave**); `crates/md-codec/tests/fixtures/liana/cases.json` (**descriptor-mnemonic**).

**F-640 is RESOLVED, not open.** Liana v15.0 **accepts** a nested taptree —
reproduced twice, with a flat control accepted in the same run. §8.1 is
satisfiable and the spec is NOT amended. The accepted shape and full harness
output are in `design/agent-reports/f449-plan-stage2-r0.md` Appendix A and
`design/RECON_f449_stage2.md`.

- [ ] **Step 1: Vector the accepted nested case** into `cases.json`
  (descriptor B: four keys, reusing the corpus's existing key set and internal
  key — the smallest diff, and reachable via `--path`, not any preset). Record
  its verdict, its two inferred recovery paths (`older` 26280 / 52560) and its
  addresses, exactly as the harness returned them.

- [ ] **Step 2: ORDERING and the REGENERATOR (R0 I-5, R1 I-e, R2 NEW-I-3).**
  Task 4's assertions now derive their counts from the corpus, so adding a
  fifth ACCEPT here no longer falsifies them — confirm that before relying on
  it.

  Separately and more dangerously: `scripts/vendor-liana-evidence.sh`
  (**descriptor-mnemonic**) rebuilds `cases.json` **wholesale**, and "add the
  case to its list" is **four edits, not one** — MEASURED against the script,
  where three of the four were unstated and each is a hard `SystemExit`:

  1. **`NAMES`** (`:94-103`) — the ordered 8-name list the rebuild iterates.
  2. **`ACCEPTED_NAMES`** (`:104-109`) — a **second** list. `:186-190` exits if
     a record's `ok` disagrees with membership, and the new case is an ACCEPT,
     so it belongs in both.
  3. A record in
     `<engrave>/design/evidence/composer-fable-r0/fable-liana-parse-in.jsonl`.
  4. A record in `…/fable-liana-parse-out-v15.jsonl` carrying `ok`, `receive`
     and `change`. `:158-163` exits *"missing evidence"* for any `NAMES` entry
     missing from either file — so **adding the name alone breaks the script
     outright**, and the implementer's cheap way out of a broken script is to
     leave `NAMES` untouched, which restores exactly the silent deletion this
     step exists to prevent.

  Those two JSONLs are the script's ONLY inputs (`:65-66`). The evidence
  directory Step 5 creates is provenance for a human, **not** an input the
  script reads — saying so here is what stops the next reader from "fixing"
  the script by pointing it at the wrong path. All four edits land **in the
  same commit as the vector**.

  **Then prove it mechanically, because two rounds of warning-paragraphs have
  now failed to:** re-run `./scripts/vendor-liana-evidence.sh <path-to-mnemonic-engrave>`
  and assert `git diff --exit-code crates/md-codec/tests/fixtures/liana/cases.json`
  comes back clean. A regeneration that reproduces the committed corpus
  byte-for-byte is the only evidence that the vector survives the next one.

- [ ] **Step 3: `scripts/liana-live-gate.sh`** (mnemonic-engrave). Locate the
  checkout, build the harness, run every vendored case plus controls, diff
  against a committed expectation.

  **THE PROBE RULE, which is the whole lesson of R0 C-3:** for every probe the
  gate sends, it must first **recompute the Liana unspendable xpub over that
  probe's own leaf set** and assert the descriptor carries it. Liana emits ONE
  message for at least three causes — a real policy refusal, an internal key
  that does not match the recipe over these leaves, and a raw NUMS point
  (`analysis.rs:596-600` falls through to `or(Key(IK), tree)` on mismatch). A
  control cannot catch this: it carries its own correct key and passes while
  the probe fails for a key reason wearing a policy reason's words. That is
  exactly how three of my probes were misread.

- [ ] **Step 4: §8.8 may not be skipped (R0 I-7).** The spec REQUIRES this run.
  A missing checkout is a loud failure naming the clone command, never a silent
  pass — and **the stage may not close on a skip**. Note the harness takes its
  Liana path as a Cargo path dependency, so an env override is not
  implementable: the gate edits/checks `Cargo.toml` or it states plainly that
  the path is fixed.

- [ ] **Step 5: Commit the evidence** under `design/evidence/f449-stage2/` with
  the Liana tag AND commit SHA in the file — "Liana accepts this" is meaningless
  without the version that accepted it.

- [ ] **Step 6: Close F-640** in `design/FOLLOWUPS.md` (**mnemonic-engrave**)
  citing the vector and the evidence file.

---

## Task 6: the three message-precision follow-ups

**Files:** per follow-up; `design/FOLLOWUPS.md` for the status lines.

These are one class — *the refusal is correct, the explanation names the wrong thing* — and they are cheap together and awkward apart.

- [ ] **Step 1: F-636** — the disjoint-multipath refusal blames md's one-path-per-slot limit when the real problem is an unspendable key at a spending leaf. Detect that the repeated key is the recognised internal key and say so.
- [ ] **Step 2: F-638** — `UnspendableUseSiteNotCanonical` names no `@N`; from the override half it describes a field the operator can see is correct. Carry `idx` and name the placeholder.
- [ ] **Step 3: F-641** — `validate_marker_position` matches the three bytes `tr(`, so `wsh(tr(MARKER,…))` and any identifier ending in `tr` pass it and then fail naming a synthetic key. **R1 M-c: "check the parse position" is the same text-level answer one layer
  down.** The check must ask a STRUCTURAL question — *is the marker the first
  argument of the OUTERMOST `tr` node, per the parse?* — not any question
  about the bytes that precede it. Name in the follow-up's closing note which
  structure is consulted.
- [ ] **Step 4:** each gets a test asserting the NEW wording, and each existing test that pins the old wording is updated deliberately, not blanket-regenerated.
- [ ] **Step 5:** mark all three **Status: CLOSED** with the commit, and run `scripts/followups-status.sh`.
- [ ] **Step 6: Gate and commit.**

---

## Self-Review

**Spec coverage.** §9 stage 2's four items: `--unspendable` → **Task 1**;
`md descriptor` kind 1 and the `UNSPENDABLE(liana)` substitution rule →
**already shipped in 1b** (RECON, measured). **§4a's JSON bump is HALF
shipped** — decode's side landed in 1b; **compose's side is Task 1b Step 2**
(R2 NEW-I-4: the recon, this plan's first draft and this ledger have each
recorded it as fully done, which makes three occurrences of one false fact, so
the ledger now names both halves rather than the convenient one). §6 row 3 →
**Task 2 Step 4**. §8.2's evidence legs → **Task 4**. §8.8's live gate →
**Task 5**. §8.1's nested ACCEPT → **Task 5 Step 1**. §8.9's `md repair` row →
**Task 2c**. The release → **Task 8**.
(R1 N-b: the previous ledger pointed §8.2 at Task 3 — the pointer stub — and
sent both §8.8 and §8.1 to Task 4. Nothing was unscheduled; the pointers were
simply wrong, and a ledger is the artifact a future reader consults to ask
what this stage owed.)

**Deliberately absent, owned elsewhere:** the Go port (stage 3), device work
(stage 4), `me` and F-635 (stage 4a), `/sh2` (stage 5). The **toolkit's** own
md-codec pin bump is a downstream consumer of this stage's tag — SPEC:861-863
puts it in this cycle — and Task 8 Step 5 says which side files it.

**Pre-dispatch gates — all four named, with their expected exit (R2 M-k).**
From **mnemonic-engrave**, against this file:
- `./scripts/plan-build-gate-md.sh` — the ```rust blocks. Expected **0**.
- `./scripts/plan-api-check.sh` — expected **0**. `UnspendableKind`,
  `lower_tr`'s new parameter and `compose::run`'s new parameter are symbols
  this plan CREATES and must appear in the ALLOW list as such, not as
  pre-existing.
- `./scripts/plan-stepref-check.sh` — expected **non-zero**: this plan cites
  step numbers in prose throughout. The count is not the verdict; read the
  hits for stale TARGETS. That is what surfaced the §8.1 → "Task 4 Step 3"
  pointer above, in under a second, after a full review round had missed it.
- `./scripts/plan-cite-check.sh` — **MEASURED on this plan: 1 resolved / 29,
  17 dangling, and every one of the 17 is a false alarm.** The script joins a
  citation to a repo ROOT (`CITE_FORK_ROOT` + this repo), and this plan cites
  bare basenames — `main.rs:858`, `chunk.rs:658`, `cmd/compose.rs:790-799` —
  which live at `crates/md-cli/src/…` in descriptor-mnemonic and so match no
  root-relative path. **Do not read its exit code as a verdict here**, and do
  not "fix" the plan by rewriting citations to satisfy a script that is built
  for a different repo's paths. For descriptor-mnemonic citations the check is
  the resolve-and-print one-liner below, which is what caught R2's own
  `SPEC:873-875` in an 863-line file:

  ```sh
  # from mnemonic-engrave; DM=/scratch/code/shibboleth/descriptor-mnemonic
  grep -ohE '`[A-Za-z0-9_./-]+\.(rs|toml|sh):[0-9]+' PLAN.md | tr -d '`' | sort -u |
  while IFS=: read -r f l; do
    path=$(git -C "$DM" ls-files "*$f" | head -1)
    printf '%-34s %s\n' "$f:$l" "$(sed -n "${l}p" "$DM/$path")"
  done
  ```
  It prints the line so a reader can check it against the claim — which is the
  half no script covers.

**Type consistency.** `InternalKey::{Slot,NumsPoint,LianaUnspendable}` is
stage 1a's shipped shape; `UnspendableKind` is new and distinct from it — the
former is a wire concept, the latter a request. Do not merge them.

**Build-gate coverage — stated, not assumed.** `scripts/plan-build-gate-md.sh`
compiles Task 1's golden test, which reads its golden at RUNTIME rather than
via `include_str!`, so the gate proves the test's logic without the artifact
and a missing golden is a loud test failure instead of a build error that
invites stubbing. It extracts only blocks preceded by an anchor naming a
`.rs` file, and only whole items compile — so **Task 1 Steps 6 and 7 are
FRAGMENTS** (a match arm inside a struct literal; a clap attribute on a field),
deliberately unanchored and NOT gated. They need a reviewer's execution pass.
Three further things the gate cannot reach, named so nothing hides behind it:
- whether `scripts/gen-compose-golden.sh` emits the shape the test
  deserialises — Step 0 now specifies it, but the script itself is still a
  reviewer's execution pass;
- `missing_docs`: the gate's scratch crate carries no `[workspace.lints]`
  table while the real workspace does (`Cargo.toml:11-12`) and
  `scripts/phase-gate.sh:32-33` runs `cargo clippy … -- -D warnings`. That is
  why every public item introduced here — **enum variants included** —
  carries a doc comment (R1 M-a);
- prose contradictions between two places that state the same ruling. Four of
  R2's five Importants were that shape. The only defence is that the rulings
  in Task 1 Step 7, Task 1 Step 1's test comment and Task 1b Step 1 are the
  SAME sentence, so a grep finds all three.

**The one risk worth naming.** The floor is the golden, and the golden is only
as wide as the generator recorded. Step 0 pins it to the 14 exit-0 cells and
Step 1 asserts that set — four wrappers, six presets, fourteen rows — rather
than a length, because a `tr`-only golden of six passes a length check while
silently retiring the eight non-`tr` rows that Task 1b's refusal is most
likely to break. A later stage that adds a preset updates that assertion
deliberately; it must never be relaxed to make a red row green.

---

## Task 7: the spec reconciliation sweep — what did this stage make FALSE?

**Files:** `design/SPEC_liana_unspendable_internal_key.md`, `design/DESIGN_coordinator_compatibility.md`, `design/FOLLOWUPS.md`.

**Why this task exists.** Implementing a stage falsifies spec text the stage
never mentions, and it does so silently. Measured over stage 1b alone, five
times:

- SPEC §9's stage-2 row listed four items; **three had already shipped in 1b**
  — the spec was stale about its own stage boundaries, and a plan written from
  it without measuring would have scheduled work already done.
- SPEC §8.10's mutation list was missing two whole CLASSES that only execution
  surfaced (a symmetric polarity inversion; a pattern-match weakening).
- SPEC §6 row 3 says `--unspendable liana` "is a **no-op today**" — Task 1 of
  THIS plan makes that sentence false.
- SPEC §8.1 asks for a nested taptree Liana ACCEPTS, which may not exist.
- `DESIGN_coordinator_compatibility.md` declared `KeyPathKind` THREE-valued;
  1b added a fourth variant, so the design record actively instructed the next
  reader not to do what the spec required. Stage 1b Task 10 had to retire it.

A spec nobody reconciles becomes a document that describes a plan rather than
the system — and this project treats records as the weak half for exactly this
reason.

- [ ] **Step 1: Grep for claims this stage falsified.** For every behaviour
  this stage changed, search the spec and design docs for the OLD claim, not
  the new one — the superseded phrasing is what survives. Start with
  `no-op`, `today`, `not a variant`, `three-valued`, `cannot`, `does not`,
  `only`, and the names of anything Task 1 or 2 touched.

  **Add `§8.9`, `md repair` and `v8` to that list (R2 §4c), because Task 2c
  reinterprets a row and nothing else in this sweep would catch it.**
  MEASURED for this fold: SPEC:724 reads *"a **v8** chunk with a correctable
  BCH error KEEPS the correction…"* — and v8 IS in `is_supported_version`
  (`crates/md-codec/src/header.rs:36-38`), so as written that row's trigger
  does not exist. The row must say *a version OUTSIDE the accepted set*, and
  it acquires an exit code (`5`) from Task 2c Step 3. Both edits land here.

- [ ] **Step 2: Reconcile §9's stage table against reality.** For each
  remaining stage (3, 4, 4a, 5), confirm every item it lists is still
  outstanding. Anything already shipped moves out with the commit that shipped
  it named. This is the check that would have saved this plan's own recon.

- [ ] **Step 3: §8.1 is CLOSED, not open.** R0 C-3 settled it and I
  reproduced it: Liana v15.0 accepts a nested taptree, so §8.1 is satisfiable
  and the "amend the spec" branch my first draft carried is **deleted** — it
  rested on a premise measured false. Confirm here only that Task 5 vectored
  the accepted case and that F-640 is marked CLOSED in **mnemonic-engrave**'s
  `design/FOLLOWUPS.md`, citing the evidence file.

- [ ] **Step 4: Re-run the measurable claims — with a command, not a promise
  (R2 §4c).** Any spec sentence citing a count, a `file:line` or a command's
  output gets re-run, not re-read: cite the value, do not restate it. Use the
  resolve-and-print one-liner from the Self-Review, **not**
  `plan-cite-check.sh` — measured, that script resolves 1 of this document's
  29 citations because they are bare basenames, so its exit code carries no
  signal here. This fold is the evidence the step is needed: R2's NEW-I-5
  cited `SPEC:873-875` in an 863-line file and the controller propagated it
  into two places before a re-grep caught it. Paste the output into the
  commit. The blind spot that remains: printing proves the line EXISTS, not
  that it says what the citing sentence claims — read every line.

- [ ] **Step 5: Commit the reconciliation separately** from any code, so
  `git log` shows what the stage changed and what it merely made false as two
  reviewable things.

**Standing rule this establishes:** every stage from here ends with this
sweep, before the stage is called done. It is cheap — a grep and a re-run —
and the alternative is a spec that drifts one stage at a time until a reviewer
trusts it and is wrong.

---

## Task 8: the release — the version bump, the exact pin, and two CHANGELOG entries

**R2 NEW-I-5: no task anywhere scheduled this**, and the "owned elsewhere"
list did not name it either, so it was an omission rather than a deferral.
Stage 1b gave the identical work its own Task 10 **and a gating constraint**
(`design/IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md:52`, `:1067`). This
task is that one, transcribed to stage 2's numbers.

**Files (all descriptor-mnemonic):** `crates/md-codec/Cargo.toml`,
`crates/md-cli/Cargo.toml`, `CHANGELOG.md`.

- [ ] **Step 1: The numbers.** md-codec `0.46.0 → 0.47.0`, md-cli
  `0.18.0 → 0.19.0` (measured current values). Stage 2 adds a public
  `enum UnspendableKind`, a parameter to `compose::compose` and
  `compose::compose_with` (both `pub`), and a parameter to `lower_tr` (which
  is `pub(super)`, so that one is not a public break). Under the repo's
  pre-1.0 convention that the second component is the breaking-change axis,
  that is a minor bump on each crate.

- [ ] **Step 2: The pin moves in the SAME commit.** `crates/md-cli/Cargo.toml:28`
  is `md-codec = { path = "../md-codec", version = "=0.46.0" }` — an **exact**
  pin, so the moment md-codec's version moves, a split commit fails to
  RESOLVE before rustc runs and is red on its own. Stage 1b recorded that
  mechanism verbatim; it is why this is one commit and not two.

- [ ] **Step 3: Two CHANGELOG entries.** `CHANGELOG.md` documents both crates
  (*"All notable changes to `md-codec` and `md-cli`"*). md-codec: the kind
  selector at the lowering site. md-cli: `--unspendable`, its refusal under
  non-`tr` wrappers, the §6 refusal and the two warns, compose's `--json`
  third state, and `md repair`'s exit-5 mapping for a corrected-but-unsupported
  version (Task 2c). Match the shape of the `## md-cli [0.18.0] — 2026-09-22`
  entry already in the file.

- [ ] **Step 4: Gate and commit** — `cargo nextest run --locked --all-features`,
  `./scripts/phase-gate.sh`, and a `cargo build --locked` proving the pin
  resolves.

- [ ] **Step 5: The toolkit's pin is NOT this stage's.** **SPEC:861-863**
  (re-grepped for this fold — R2 cited `:873-875`, and the spec is 863 lines
  long) says the toolkit pins md-codec by git tag, so stage 1 did not break
  it, but it is *"in scope for this cycle"* and gets its pin bump and a golden
  refresh **after stage 2**. File it as a follow-up owned by the cycle,
  naming the tag this task creates — `mnemonic-engrave`'s `design/FOLLOWUPS.md`,
  per the two-repos constraint.
