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

- [ ] **Step 0: Generate and commit the golden BEFORE anything else (R0 C-2, R1 I-g).**
  `scripts/gen-compose-golden.sh` runs the CURRENT `md` — 0.18.0, no flag —
  over every preset and every wrapper, writing
  `crates/md-cli/tests/golden/compose_pre_unspendable.json`. **Commit it in
  its own commit, before the flag exists.** Two reasons, and the second is the
  point: Step 1's `include_str!` cannot compile without it, and a golden
  generated after the flag lands would be blessed by the very code it guards
  — the defect C-2 was raised against. Do NOT stub it to make the build pass.

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
use std::collections::BTreeMap;
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
    assert!(golden.len() >= 6, "the floor must cover EVERY preset, got {}", golden.len());
    for (invocation, expected) in &golden {
        let args: Vec<&str> = invocation.split(' ').collect();
        let (out, err, code) = md(&args);
        assert_eq!(code, 0, "{invocation}: {err}");
        assert_eq!(&out, expected, "{invocation} moved against 0.18.0");
        // Explicit `nums` must equal the default -- but ONLY for `tr`.
        // Task 1b refuses the flag under wsh/sh/sh-wsh (there is no taproot
        // internal key there), so appending it to those rows would assert
        // exit 0 on an invocation this stage deliberately makes fail.
        if args.contains(&"tr") {
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

- [ ] **Step 9: Prove BOTH mutations.**
  (a) Swap the arms in Step 6 so `Nums` yields `LianaUnspendable`.
  (b) Mis-parse the flag so `"nums"` maps to `UnspendableKind::Liana`.
  **Each must turn the golden test red.** (b) is the one the self-comparing
  draft could not catch and is the reason the golden exists — if (b) stays
  green the floor is still vacuous. Paste all four results.

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
  **Refuse it at argument parsing**, naming the wrapper: an unspendable
  internal key is a taproot concept. Test all four wrappers.

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
  instead of the flag.

- [ ] **Step 3: WARN — the Liana-policy cases, keyed on the SHAPE not the name.**
  **R1 I-f:** keying on preset name misses every `--path`-built equivalent.
  Measured: `--path 2of3 --path 1of1,sha256=…,older=100` composes at exit 0
  with a NUMS internal key and no preset at all, so a name-keyed warning is
  silent on exactly the hand-built wallet an operator is most likely to get
  wrong. Detect the composed shape — a hashlock leaf present, or no
  non-timelocked path — and warn on that. `hashlock-gated` and
  `decaying-multisig` then fall out as instances rather than special cases.
  Warn on stderr; exit 0; test BOTH the preset and the `--path` spelling.

- [ ] **Step 4: SPEC §6 row 3's warn** — `--unspendable liana` where
  `internal_key_path(list)` is `Some`, so a real key was extracted and there
  is no unspendable key to choose. Trigger `ik.is_some() && unspendable == Liana`
  (R0 confirmed correct for `tr`). The codec signals; the CLI prints.

- [ ] **Step 5:** `--json` must still emit valid JSON on stdout while every
  warning goes to stderr. Assert both channels.

- [ ] **Step 6:** prove each refusal and each warning can fail; paste them.

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
`decode_with_correction` (`chunk.rs:659`/`:666`), not in `repair.rs`.

- [ ] **Step 1: Prove the trigger is reachable BEFORE writing the fix.**
  Construct a chunk at an unsupported version with one correctable BCH error
  and show today's behaviour. If it already keeps the correction and reports
  the version distinctly, **this task is already satisfied — say so, record
  the measurement, and close it without a code change.** A task whose gate
  already passes is not work.

- [ ] **Step 2:** only if Step 1 shows a real gap — carry `details` through
  `chunk.rs:659`/`:666` so a successful correction is not thrown away when the
  version check fails afterwards.

- [ ] **Step 3:** test that the corrected payload survives AND that the exit
  distinguishes "unsupported version" from "could not correct".

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

**Files:** `crates/md-codec/tests/liana_evidence_legs.rs` (create) — or md-cli if it needs the binary; choose by what the assertion needs and say why in the file's doc comment.

**R1 I-e — do not hardcode the corpus's shape.** Task 5 adds a fifth ACCEPT
case, which falsifies any assertion naming "four" or "24". **Derive both
counts from the corpus at run time** and assert only that the set is
non-empty and that every ACCEPT case passes. A hardcoded count is precisely
what breaks the next time the corpus grows.

**And the regenerator will erase a hand-added case.**
`scripts/vendor-liana-evidence.sh` rewrites `cases.json` **wholesale** from a
hardcoded 8-name list. Task 5's new vector must be added to that list in the
same commit, or the next regeneration silently deletes it and these tests go
green over a corpus that lost its only nested ACCEPT.

- [ ] **Step 1: Descriptor equality against every ACCEPT shape**

For each vendored case whose recorded Liana verdict is ACCEPT, assert that `md`'s rendered descriptor equals the evidence's descriptor **byte-exact including the checksum**, for every case the corpus marks ACCEPT — count derived, not written down. The whole-branch review measured 8/8; that measurement lives in a transcript and dies with it. This is the same property as an assertion.

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

- [ ] **Step 2: ORDERING and the REGENERATOR (R0 I-5, R1 I-e).** Task 4's
  assertions now derive their counts from the corpus, so adding a fifth ACCEPT
  here no longer falsifies them — confirm that before relying on it.

  Separately and more dangerously: `scripts/vendor-liana-evidence.sh` rebuilds
  `cases.json` **wholesale** from a hardcoded 8-name list. Add this case to
  that list **in the same commit as the vector**, or the next regeneration
  deletes the corpus's only nested ACCEPT and every test stays green over the
  loss.

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
compiles Task 1's golden test, which reads its golden at RUNTIME rather than
via `include_str!` — so the gate proves the test's logic without the artifact,
and a missing golden is a loud test failure rather than a build error that
invites stubbing. What the gate does NOT cover is whether
`scripts/gen-compose-golden.sh` emits the shape the test deserialises; that
needs a reviewer's execution pass.

**Build-gate coverage — stated, not assumed.** `scripts/plan-build-gate-md.sh`
extracts only blocks preceded by an anchor naming a `.rs` file, and only whole
items compile. Task 1 Steps 6 and 7 are FRAGMENTS — a match arm inside a
struct literal, and a clap attribute on a field — so they are deliberately
unanchored and are NOT gated. They still need a reviewer's execution pass. A
gate that hid this would be worse than no gate.

**The one risk worth naming.** Task 1's default-equality test is the stage's regression floor, and it is only as wide as the presets it lists. Extend it to every preset the composer supports before relying on it.

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
