# IMPLEMENTATION PLAN — descriptor input, S1 + S3

**Status:** DRAFT, round 0. No code before this plan's R0 closes 0C/0I.
**Spec:** `SPEC_descriptor_input.md`, FINAL GREEN (20 rounds +
the 15-finding walk; closure verdict and leaves-open list in
`design/agent-reports/R0-descriptor-input-spec-r20-closure.md`), plus the
post-GREEN conjunct-8 amendment under verification by this plan's own
rounds — the plan tracks the spec's CURRENT tip, not a fixed SHA.
**Phase order:** S1 → S3 per F-418; S2 and F-423 parked for the device.
**Plan baseline revs** (for `scripts/plan-staleness-check.sh`):
descriptor-mnemonic `6864f377` · seedhammer fork `d402f18` ·
mnemonic-engrave: **re-pinned at each phase gate to the spec's CURRENT
tip** — the spec carries post-GREEN amendments verified by this plan's own
rounds, and a fixed pre-amendment SHA aims the staleness gate at a tree
without conjunct 8 (PLAN-r2 M2).

**The one structuring decision, from the cycle's banked lesson:** the
vector file is authored FIRST and is the failing test the parser is built
against. Nothing in P1/P2 is specified in prose that a vector row can pin.

---

## P0 — the vector file, both repos' harnesses, red before green

Everything here is data + test scaffolding; no parser code.

- **P0.1** Author `crates/me-cli/testdata/descriptor_seam_vectors.json` per
  spec §7 AS AMENDED (conjunct 8): ≥ **71** physical rows, 9 tags, **88**
  tag-slots, per-tag minima (`formats-happy` 4 · `promotion-near-miss` 15 ·
  `narrowed-4.7` 14 · `accepted-extreme` 1 · `narrowed-4.2` 5 · `neither` 3
  · `whitespace` 3 · `md1-splits` 6 · `gate` **37**, incl. clause 8's
  impossible-wallet trio), schema fields per §7 incl. `covers` and
  `md1_admits` (REQUIRED), `canonical` on every host-admitted row,
  `device_probe` markers, the value fields, per-row `sha256`. All measured
  values re-derived at authoring time (debug `md`, a Go scratch probe) —
  never transcribed from reports. Gate rows deliver via `--in` (r20-M2's
  clause); multi-line gate inputs are the LF-separated record stream
  (r19's note). `wallet_id` computed over the (a′)-materialised policy and **carried by
  MULTISIG rows at the device-default use-site only** — the Go route's
  measured domain (`EncodeMultisig` hard-codes `<0;1>/*`, no single-sig
  arm; PLAN-r2 M4) — so the cross-language gate's scope is stated, not
  authored ad hoc. Clause-8 rows carry NO address fields: the refusal
  assertion is their witness (a colliding-origin wallet derives
  byte-identical addresses to a clean control — measured, PLAN-r2 M5).
- **P0.2** Rust harness `crates/me-cli/tests/descriptor_seam.rs`: pins the
  sha256; asserts the manifest arithmetic — tag minima, **88 slots, 71-row
  floor, 17 overlap slots distributed as 15 second-tags on the §4.5 rows
  plus 2 third-tags on the named pair, `covers` entries distinct within a
  row** (PLAN-r1 I1); **rejects unknown row keys, and asserts per-column
  assertion COUNTS against expected totals** so a mistyped field name reds
  the suite (PLAN-r1 I7); requirement 5's non-vacuity. Host-column
  assertions land `#[ignore]`-tagged, each ignore naming the phase that
  removes it; **the zero-`#[ignore]` grep is P2's gate** (PLAN-r1 M2).
- **P0.3** Fork side (Rust-first, then vendor). **Mechanics (PLAN-r1 I3):
  a NEW worktree on a NEW branch `seam/descriptor-vectors` cut from fork
  `main` (`d402f18`) — never the `ship/tx-engraving` checkout, which is
  another cycle's in-flight branch and lacks the codex32 precedent files.**
  Copy the file byte-identically to
  `seedhammer/nonstandard/testdata/descriptor_seam_vectors.json`; add
  `nonstandard/descriptor_seam_test.go` as `package nonstandard_test`,
  asserting: `device_admits` via `nonstandard.OutputDescriptor` on the
  input; requirement 4's fixed point on `canonical`; `address_0/1` via
  `address.Receive` where present; **`wallet_id` via the fork's own md
  package (`md.EncodeMultisig` → `WalletPolicyIdChunks`, or the
  `WalletPolicyIDStub` route) — the F-212 cross-language gate, both suites
  (PLAN-r1 I2)**; `device_probe` rows never fed to the panicking function;
  the same per-column count assertions as P0.2. `sysw_class` rows: skipped
  with a named reason (S2's arm). **Push: plain
  `git push -u origin seam/descriptor-vectors`** — the fork's `main` has NO
  branch protection (measured: 404) and its `tests` job runs on every
  push, so the staging ritual has nothing to satisfy there (PLAN-r2 M3).
- **P0 gate:** Go seam test green on the fork branch; Rust harness green on
  its non-parser assertions; sha256 pins byte-equal; column-count manifests
  agree across the two suites; baseline suites untouched-green.
  **Staleness re-validation before P1 dispatch** (M4): `scripts/
  plan-staleness-check.sh` against this plan's baselines.
- **P0 review:** proportional — a sonnet fold-vs-manifest pass, counts by
  machine, NOT a fresh audit.

## P1 — the cascade and the admission predicate (host columns green)

- **P1.0** F-413 discharge (PLAN-r1 I4 — owning phase "before S1 closes"):
  a **fable consult substitutes for the operator** per the overnight
  mandate, briefed with the measured facts (the five-version device set,
  the executable-remedy refusal, the normalise-alternative's safety under
  the canonical invariant) and the interim default. Fold the ruling; either
  implement it in P1.1 or re-own the entry explicitly. The FOLLOWUPS entry
  leaves `#ruling-needed` at this task, not at ship. The mandate's two
  tests, applied (PLAN-r2 M6): it GATES (phase-owned, "before S1 closes",
  not deferrable past its phase), and it is NOT an unsettleable funds risk
  — both candidate behaviours are funds-safe; the consult chooses between
  refuse-with-remedy and normalise, either of which the spec's invariants
  license.
- **P1.1** Module placement (PLAN-r1 C2): **`#[doc(hidden)] pub mod
  descriptor` in `crates/me-cli/src/lib.rs`** — lib-public so
  `tests/descriptor_seam.rs` can call the predicate directly (the codex32
  precedent's shape: `mnemonic_engrave::sysw::classify`), doc-hidden to
  keep the published API surface deliberate. Submodules: `cascade.rs`
  (§4.1–§4.5 semantics: the four branches in device order, BlueWallet
  headers + exactly-8-hex fingerprints + the four normative refusals incl.
  F-419's zero-cosigner; the five version bytes; case-insensitive JSON;
  promotion with the key-as-supplied echo), `admit.rs` (the seven shapes +
  the `multi` md1-path twins + conjuncts 2–7 **+ conjunct 8's
  impossible-wallet checks (PLAN-r1 C1), refusing with §6's TWO
  key-identity rows (the tree's text names no next action; §6's rules
  bind) — convergence with the Rust primary**), errors carrying §6's cause
  taxonomy. TDD: un-ignore host-column tests first, watch red, build green.
  No new deps.
- **P1.2** Whitespace normalisation (§4.6); single-document mode + the
  whole-input discriminator with the descriptor-shape gate (§5.1),
  implemented FROM the 37 gate rows. The shipped record refusal (exit 4)
  stays pinned by the existing `sysw_cli.rs` suite.
- **P1 gate:** all P0-ignored host assertions un-ignored and green EXCEPT
  the enumerated `--as md1`-execution set (each remaining ignore names P2);
  full workspace `cargo nextest run --locked` + clippy `-D warnings` + fmt;
  Go seam re-run green; staleness re-check before P2 dispatch.
- **P1 review:** proportional opus pass over the diff (cascade vs measured
  device semantics; conjunct-8 wording vs §6's two key-identity rows).

## P2 — `--as md1` end to end (S3 proper)

Build order within P2 (PLAN-r1 M3; PLAN-r2 N3): **P2.1's flag skeleton →
P2.2 → P2.3 → P2.1's window text → P2.4** — the window refusal's variant selection needs md1-representability
(P2.2) and emits after the identification block (P2.3).

- **P2.1** `--as <descriptor|md1>` flag surface: `md1` implemented;
  `descriptor` present, its full window behaviour (both variants, carriage
  rule, admission-precedes-window, choice-block marking) completed once
  P2.2/P2.3 exist.
- **P2.2** The md1 build path: `md_codec::encode::Descriptor` in-process
  (per-key (a)/(a′)/(a″), the `multi` twins, TLVs, divergent-path mode),
  `encode_md1_string`/`split`, records packed as **`Class::MdMk`**
  (PLAN-r1 N1). Conjunct 8 refuses BEFORE encoding — the published-crate
  gap never reaches the codec.
- **P2.3** The identification block (§5.4): two tiers, full line set —
  wallet-id (materialised base, `none` line), address 0 + compare prompt,
  watch-only, the (a′) annotation — **plus §5.3(b)'s label warning with
  its verbatim text and a test that J1's `Name: sh` fixture fires it
  (PLAN-r1 I5)**.
- **P2.4** §6's refusal texts, one named test per row, **all 36 rows
  (§6 gained the two key-identity rows, PLAN-r2, split per PLAN-r3 I3) — the S2-parked set is EMPTY**
  (PLAN-r1 I6: every §6 trigger is reachable in this build; the two
  `--as descriptor`-mentioning rows fire as conjunct 1's permanent refusal
  and the window refusal, whose two variants get their own tests alongside
  the five-case item-5 matrix). "Verbatim" means WHAT THIS BUILD PRINTS:
  the two window-substituted rows are asserted in their SUBSTITUTED form
  per §5.3's normative substitution (PLAN-r2 NEW-I2). The test file
  asserts its own row-test count == 36. Where `md1_admits`
  is false on an admitted row, the refusal assertion **cites
  §5.3(a)/(a″)** (PLAN-r1 M6).
- **P2.5** F-421's converter referral — one refusal string + one test.
- **P2 gate:** ZERO `#[ignore]` (grep-gated here); all value-field
  assertions live with their counts; full workspace suites; the §11 item 2
  walk on J1's fixture (all four formats, JSON exemplar non-`/0/*`);
  staleness re-check before P3.
- **P2 review:** the MANDATORY post-implementation adversarial execution
  review, whole S1+S3 diff, **opus — fable only per the mandate's
  15-round count trigger, not per-review** (PLAN-r1 M7); report persisted;
  the two walk journeys re-run as integration tests.

## P3 — acceptance, records, ship

- **P3.1** §11's S3-bound items discharged and named (items 2, 3, 4 — all
  36 rows —, 5); items 1/6 recorded as S2-parked (F-418).
- **P3.2** F-416's cross-ref sentence in `SPEC_systemwide_payloads.md` §5.6.
- **P3.3** CHANGELOG; FOLLOWUPS reconciliation: F-419 → P1, F-421 → P2,
  F-413 → discharged at P1.0, F-424 → parked (next publish,
  operator-gated), F-414/F-420 → parked with owners, **F-422 → standing
  decision, no owner (PLAN-r1 M5)**; continuity + memory.
- **P3.4** Merge worktree → master; push via `scripts/push-via-staging.sh`;
  fork branch pushed plain (no protection on the fork — PLAN-r2 M3) and
  left as a branch for the operator's merge decision (fork `main` merges
  are not overnight work — it is another cycle's repo); the unmerged
  branch's integration is owned by **F-425**.

## Out of scope (parked, with owners)

S2 whole (device classifier arm, `--as descriptor` packing, §11 items 1/6,
QR plates) — F-418, awaits the SH2. F-423 plate packing — with S2's
firmware. F-420 (`md`'s referral) — descriptor-mnemonic, with-or-after S1;
stretch if the night allows, own commit + own push. F-413/F-422 — operator
rulings, spec-as-written meanwhile.

## Load-bearing anchors (gives the citation and staleness gates material — PLAN-r3 I5)

`crates/me-cli/src/sysw/mod.rs:205` · `crates/me-cli/src/main.rs:335` ·
`crates/me-cli/tests/codex32_seam.rs:60` · `crates/me-cli/tests/sysw_cli.rs:1928` ·
`descriptor-mnemonic` `crates/md-codec/src/encode.rs:118` and `:120` (the two
validator calls the published crate lacks) · fork `nonstandard/parse.go:36` ·
fork `md/walletpolicyid.go:138` · fork `md/encode_multisig.go:112`. Each
verified this cycle; the per-phase staleness re-check now has citations to
examine.

## What the build gate does not cover here

This plan carries zero fenced code blocks; its executable content is
COMMANDS and FILE PATHS — the ungated class the constellation has measured
at 5-of-22 false. The countermeasure is stated for every review brief:
reviewers EXECUTE the commands and resolve the paths (four of PLAN-r2's
five blockers were found exactly that way), and the citation gate covers
`file:line` references mechanically.

## Stop-rules and boundaries (the overnight mandate)

Per `CONTINUITY_2026-08-28-overnight.md` (operator, 2026-08-28): 5 review
rounds expected and good; at 15 opus reviews the tier switches to fable;
25 is a hard stop — park with a note. No tags/releases/publishes/on-device
actions; pushes in scope.
