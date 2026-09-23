# F-449 stage 2 — `md compose --unspendable`, and the live Liana gate

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** an operator can ask `md compose` for a Liana-importable `tr` wallet, and the claim is backed by Liana's own importer rather than by our reading of it.

**Architecture:** stage 1b already taught the codec the kind, the CLI the marker, and `md descriptor`/`--json` the rendering. Exactly one decision site is missing: `compose/tr.rs:47`'s `None => InternalKey::NumsPoint`. A new `--unspendable liana|nums` flag (default `nums`) selects between kinds there. Everything else in this stage is evidence: committing the legs stage 1b deferred because it had no CLI, and turning the live harness run into a gate anyone can re-run.

**Tech stack:** Rust (md-codec, md-cli), the committed `harnesses/liana` against a Liana **v15.0** checkout.

**Spec:** `design/SPEC_liana_unspendable_internal_key.md` (GREEN). **Recon:** `design/RECON_f449_stage2.md` — read it first; it measures that three of SPEC §9's four stage-2 items already shipped in 1b.

**Baseline revision:** `descriptor-mnemonic` main `25acb33c` (md-codec 0.46.0, md-cli 0.18.0, 1500/1500).

## Global Constraints

- **Default is `nums`.** Composing without the flag must produce byte-identical output to today, for every one of the 65 vendored vectors. This is the stage's own regression floor.
- **`liana` is not always achievable.** When `internal_key_path(list)` is `Some`, the composer extracts a real key and there is no unspendable internal key to choose. SPEC §6 row 3: that combination **WARNS**, never silently no-ops.
- **The harness is the oracle, not our opinion.** Any claim that Liana accepts or refuses a shape must come from `harnesses/liana` output committed as evidence, with a known-good control in the same run.
- **A probe that refuses on a parse error proves nothing about policy.** `"Error while parsing xkey"` is a malformed input, not a verdict. Every probe carries a control; a run whose control fails is void.
- Rust-primary: this stage is Rust only. The Go port is stage 3.
- Toolchain: `export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH` — bare `cargo clippy` is 0.1.98 and fails baseline.

---

## File Structure

- **Modify** `crates/md-codec/src/compose/tr.rs:23-50` — `lower_tr` takes the requested kind; the `None =>` arm selects `NumsPoint` or `LianaUnspendable`.
- **Modify** `crates/md-codec/src/compose/mod.rs` — thread the kind through `Composed`/the lowering entry; add the "requested liana but a key was extracted" signal.
- **Modify** `crates/md-cli/src/cmd/compose.rs:590` — `run()` gains the parameter; clap gains `--unspendable`.
- **Modify** `crates/md-cli/src/main.rs` (or wherever `Compose` is declared) — the flag.
- **Create** `crates/md-cli/tests/cli_compose_unspendable.rs` — the flag's own tests.
- **Create** `crates/md-codec/tests/liana_evidence_legs.rs` — C3's deferred descriptor- and address-equality legs.
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

- [ ] **Step 1: Write the failing test — the default is byte-identical to today**

Create `crates/md-cli/tests/cli_compose_unspendable.rs`:

```rust
// crates/md-cli/tests/cli_compose_unspendable.rs
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
    for preset in ["kofn-recovery,2of3,older=26280", "tiered-recovery,2of3,older=26280"] {
        let (a, _, ca) = md(&["compose", "--wrapper", "tr", "--preset", preset]);
        let (b, _, cb) = md(&["compose", "--wrapper", "tr", "--preset", preset,
                              "--unspendable", "nums"]);
        assert_eq!(ca, 0, "compose failed");
        assert_eq!(cb, 0, "compose --unspendable nums failed");
        assert_eq!(a, b, "explicit `nums` must equal the default for {preset}");
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
    #[default]
    Nums,
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

In `cmd/compose.rs` (also a FRAGMENT — a clap attribute on a field):

```rust
#[arg(long, value_name = "KIND", default_value = "nums")]
/// Which unspendable taproot internal key to use when no spend path supplies
/// a real one: `nums` (the BIP-341 H-point, the default and what every
/// release before md-codec 0.46.0 produced) or `liana` (Liana's own derived
/// key, SPEC §2 — required for Liana to import the wallet).
unspendable: String,
```

Parse it with an explicit error naming both values; do not accept a prefix or a case variant silently.

- [ ] **Step 8: Run both tests — expect PASS.**

- [ ] **Step 9: Prove the mutation.** Swap the two arms in Step 6 so `Nums` yields `LianaUnspendable`. `omitting_the_flag_is_byte_identical_to_the_previous_release` MUST go red. Paste red and green.

- [ ] **Step 10: Gate and commit.**

```bash
cargo nextest run --locked --all-features
./scripts/phase-gate.sh
( cd fuzz && cargo check --all-targets )
```

---

## Task 2: SPEC §6 row 3 — requesting `liana` where a key is extracted WARNS

**Files:** `crates/md-codec/src/compose/{tr.rs,mod.rs}`, `crates/md-cli/src/cmd/compose.rs`, `crates/md-cli/tests/cli_compose_unspendable.rs`.

**Interfaces consumed:** Task 1's `UnspendableKind`.

- [ ] **Step 1: Write the failing test**

Add to `crates/md-cli/tests/cli_compose_unspendable.rs`:

```rust
/// SPEC §6 row 3 (fable M-6). When the first listed path is a bare single
/// key, `internal_key_path` extracts it and the internal key is a REAL key —
/// so `--unspendable liana` has nothing to apply to. Today that is a silent
/// no-op, which hands the operator a NUMS-free wallet they believe is a
/// Liana one. It must say so.
#[test]
fn requesting_liana_where_a_key_is_extracted_warns_and_does_not_pretend() {
    let (out, err, code) = md(&["compose", "--wrapper", "tr",
                                "--path", "1of1",
                                "--path", "2of3,older=26280",
                                "--unspendable", "liana"]);
    assert_eq!(code, 0, "a warning is not a refusal: {err}");
    assert!(!out.contains("UNSPENDABLE(liana)"),
            "no unspendable key exists here — the first path supplies a real one: {out}");
    assert!(err.contains("--unspendable liana"), "must name the flag: {err}");
    assert!(err.contains("@0") || err.contains("first"),
            "must say WHICH path took the internal key: {err}");
}
```

- [ ] **Step 2: Run it — expect FAIL** (no warning on stderr).

- [ ] **Step 3: Signal it from the composer.** `lower_tr` knows both `ik` and `unspendable`; when `ik.is_some() && unspendable == Liana`, record it on `Composed` (a bool or a typed note — match how `experimental` already reports). Do NOT warn from inside md-codec: the codec has no stderr. The CLI prints it.

- [ ] **Step 4: Print it in `cmd/compose.rs`**, next to the existing `--experimental` warning so both use one vocabulary.

Wording must state the fact and the consequence, not just the fact: the first listed path supplies a real internal key, so no unspendable key is used, and **Liana will not import this wallet** — reword to `--unspendable nums` or remove the bare single path.

- [ ] **Step 5: Run it — expect PASS.**

- [ ] **Step 6: Prove the mutation.** Delete the warning; the test must go red. Paste it.

- [ ] **Step 7: Check `--json` too.** If `--json` is passed, the warning must still reach the operator and must NOT corrupt stdout's JSON. Add an assertion that `--json --unspendable liana` on this shape still parses as JSON on stdout AND warns on stderr — the "a contract that changes channel needs a test there" class.

- [ ] **Step 8: Gate and commit.**

---

## Task 3: C3's deferred evidence legs, as committed tests

Stage 1b deferred these because they needed md-cli. They are now buildable.

**Files:** `crates/md-codec/tests/liana_evidence_legs.rs` (create) — or md-cli if it needs the binary; choose by what the assertion needs and say why in the file's doc comment.

- [ ] **Step 1: Descriptor equality against the four ACCEPT shapes**

For each vendored case whose recorded Liana verdict is ACCEPT, assert that `md`'s rendered descriptor equals the evidence's descriptor **byte-exact including the checksum**. The whole-branch review measured 8/8 for this; that measurement lives in a transcript and dies with it. This is the same property as an assertion.

- [ ] **Step 2: Address equality against the four ACCEPT shapes**

Each ACCEPT case records Liana's own first three receive and change addresses. Assert md derives the same 24. Use the recorded values as the oracle — do not recompute them from the descriptor with the same code under test, which would assert only that md agrees with itself.

- [ ] **Step 3: Run both — expect PASS** (the data already agrees; these tests pin it).

- [ ] **Step 4: Prove they can fail.** Perturb one recorded address by one character and one descriptor by one character; each must go red naming the case. Restore. Paste.

- [ ] **Step 5: Gate and commit.**

---

## Task 4: the live Liana gate, and a RULING on §8.1

**Files:** `scripts/liana-live-gate.sh` (create), `design/evidence/f449-stage2/` (create), `design/FOLLOWUPS.md`.

- [ ] **Step 1: Write `scripts/liana-live-gate.sh`**

It must: locate the Liana checkout (env override, default `/scratch/code/shibboleth/.tmp/fable-liana-src-v15/liana`), **fail loudly if absent with the exact clone command**, build the harness, feed it every vendored case plus a known-good control, and diff the result against a committed expectation file. A missing checkout must be a clear skip-with-reason, never a silent pass — see `skipped-gates-are-the-default-failure`.

- [ ] **Step 2: Record the run as evidence.** Commit input and output JSONL under `design/evidence/f449-stage2/`. Include the Liana tag and commit SHA in the file, because "Liana accepts this" is meaningless without the version that accepted it.

- [ ] **Step 3: Settle §8.1 (F-640).** RECON records two nested taptrees refused on policy shape, with a control accepted in the same run. Probe systematically: construct nested shapes across Liana's documented policy forms — nested on the left, on the right, depth 3, and a nested pair of recovery tiers — each with a control. Then rule:

- if ANY nested shape is accepted → vector it, close F-640, and the §8.1 gap is closed for real;
- if none is → record the enumeration as evidence and **amend SPEC §8.1**, because it asks for something that does not exist. A spec requirement no input can satisfy is a defect in the spec, not a permanent open follow-up.

Write the ruling into `design/FOLLOWUPS.md` under F-640 either way, citing the evidence file.

- [ ] **Step 4: Commit.**

---

## Task 5: the three message-precision follow-ups

**Files:** per follow-up; `design/FOLLOWUPS.md` for the status lines.

These are one class — *the refusal is correct, the explanation names the wrong thing* — and they are cheap together and awkward apart.

- [ ] **Step 1: F-636** — the disjoint-multipath refusal blames md's one-path-per-slot limit when the real problem is an unspendable key at a spending leaf. Detect that the repeated key is the recognised internal key and say so.
- [ ] **Step 2: F-638** — `UnspendableUseSiteNotCanonical` names no `@N`; from the override half it describes a field the operator can see is correct. Carry `idx` and name the placeholder.
- [ ] **Step 3: F-641** — `validate_marker_position` matches the three bytes `tr(`, so `wsh(tr(MARKER,…))` and any identifier ending in `tr` pass it and then fail naming a synthetic key. Check the parse position, not the preceding text.
- [ ] **Step 4:** each gets a test asserting the NEW wording, and each existing test that pins the old wording is updated deliberately, not blanket-regenerated.
- [ ] **Step 5:** mark all three **Status: CLOSED** with the commit, and run `scripts/followups-status.sh`.
- [ ] **Step 6: Gate and commit.**

---

## Self-Review

**Spec coverage.** §9 stage 2's four items: `--unspendable` → Task 1; `md descriptor` kind 1, the substitution rule and the JSON bump → **already shipped in 1b** (RECON, measured). §6 row 3 → Task 2. §8.2 → Task 3. §8.8 → Task 4. §8.1 → Task 4 Step 3.

**Deliberately absent, owned elsewhere:** the Go port (stage 3), device work (stage 4), `me` and F-635 (stage 4a), `/sh2` (stage 5).

**Placeholder scan.** Run `./scripts/plan-api-check.sh` against this file before dispatching Task 1; `UnspendableKind`, `lower_tr`'s new parameter and `compose::run`'s new parameter are symbols this plan CREATES and must appear in the ALLOW list as such, not as pre-existing.

**Type consistency.** `InternalKey::{Slot,NumsPoint,LianaUnspendable}` is stage 1a's shipped shape; `UnspendableKind` is new and distinct from it — the former is a wire concept, the latter a request. Do not merge them.

**Build-gate coverage — stated, not assumed.** `scripts/plan-build-gate-md.sh`
extracts only blocks preceded by an anchor naming a `.rs` file, and only whole
items compile. Task 1 Steps 6 and 7 are FRAGMENTS — a match arm inside a
struct literal, and a clap attribute on a field — so they are deliberately
unanchored and are NOT gated. They still need a reviewer's execution pass. A
gate that hid this would be worse than no gate.

**The one risk worth naming.** Task 1's default-equality test is the stage's regression floor, and it is only as wide as the presets it lists. Extend it to every preset the composer supports before relying on it.

---

## Task 6: the spec reconciliation sweep — what did this stage make FALSE?

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

- [ ] **Step 2: Reconcile §9's stage table against reality.** For each
  remaining stage (3, 4, 4a, 5), confirm every item it lists is still
  outstanding. Anything already shipped moves out with the commit that shipped
  it named. This is the check that would have saved this plan's own recon.

- [ ] **Step 3: Rule on §8.1** (carried from Task 4 Step 3). If the evidence
  says no nested shape is acceptable, AMEND the spec. Leaving an unsatisfiable
  requirement in place makes every future stage's acceptance ambiguous.

- [ ] **Step 4: Re-run the measurable claims.** Any spec sentence citing a
  count, a file:line or a command's output gets re-run, not re-read. Cite the
  value, do not restate it.

- [ ] **Step 5: Commit the reconciliation separately** from any code, so
  `git log` shows what the stage changed and what it merely made false as two
  reviewable things.

**Standing rule this establishes:** every stage from here ends with this
sweep, before the stage is called done. It is cheap — a grep and a re-run —
and the alternative is a spec that drifts one stage at a time until a reviewer
trusts it and is wrong.
