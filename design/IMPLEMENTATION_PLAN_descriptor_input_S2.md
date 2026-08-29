# IMPLEMENTATION_PLAN_descriptor_input_S2 — `--as descriptor` end to end

**Status: DRAFT — R0 not started.** Single author per the R0 gate; this plan
binds S2 of `SPEC_descriptor_input.md` (GREEN 2026-08-28, amendments through
the S1+S3 cycle). S1+S3 shipped 2026-08-29 (engrave `f244442`, records through
`4646fa2`); the SH2 is back on the bench and boots fork `main` @ `a5e29b4`
(flashed 2026-08-29), which un-parks F-418's condition.

**Plan baseline revs** (for `scripts/plan-staleness-check.sh`):
mnemonic-engrave `4646fa2` · seedhammer fork `a5e29b4` ·
descriptor-mnemonic `6c4a56fd` (cited only for context; S2 does not change it).

**Recon ground truth:** `design/agent-reports/RECON-S2-fork-seam.md`
(fork half, all file:line cites verified at `a5e29b4`) plus the host-side
reads recorded in this plan's own citations.

## What S2 is, in one paragraph

`me sysw pack --as descriptor` stops answering with §5.1's window refusal and
packs §5.2's **canonical re-encoded descriptor** as one record of class
`Descriptor`; the device learns to **classify** that record (`sysw.Classify`
descriptor arm, Rust-first per §3/§5.2, then ported) and to **display** it
(the sysw session routes `ClassDescriptor` to the existing
`DescriptorScreen`); the fork batches F-426 (the `ypub` classification case)
and F-423 (denser `bundlePlatePlan`) into the same firmware build; and §11
items 1 and 4's `--as descriptor` rows close at the desk while **item 6 (a
`ClassDescriptor` record displayed on the real device) and every flash remain
operator-gated** — S2 is not "shipped" until the operator has seen the screen.

## The two invariants that bound every phase

1. **The vector file does not change bytes** unless a phase explicitly says
   so. Expected: NO byte change in S2 (the `sysw_class` column already exists
   on 4 rows; sha `542cd492…` pinned in both repos). If any phase DOES change
   it, that commit also carries F-428's `nonstandard/parse.go:158` citation fix and the
   same fix in `scripts/descriptor-seam-vectors/rows.py`, in both repos, one
   sha bump total.
2. **The record surface for non-descriptor records is untouched.** Every
   classifier change is additive: an input that classified as
   Mnemonic/Codex32/MdMk/Mt/FreeText before S2 classifies identically after.
   Gated per phase by the existing suites plus P1's negative sweep below.

## P0 — plan gates and the S2 flip inventory

- **P0.1** Machine-count the flip set before any code: every test, §6 row,
  and help/window string that asserts the S3-parked build state. Known
  members (verified at `4646fa2`): `Row::WindowNotInBuild`
  (`crates/me-cli/src/descriptor/refusal.rs:43`), the window refusal's two
  substituted variants and their tests, §11 item 5's sibling cases (W4/W11),
  the choice block's `(not available in this build)` marking and its M1
  build-marked clap help twin, and `DESCRIPTOR_PATH_SHIPPED == false`
  consumers (`crates/me-cli/src/main.rs:360-373`,
  `crates/me-cli/src/descriptor/as_flag.rs:126-138`). Output: a checked-in
  inventory table in this plan (folded at P0's close) stating, per member,
  its post-S2 behaviour and which P-task flips it. **The §6 row-test count
  after S2 is a MEASURED number recorded here, not assumed 36.**
  `WindowNotInBuild` itself: §5.1 keeps the row REACHABLE only if some
  build state still refuses (none does post-S2) — expected disposition is
  that the row and its tests are retired to the §5.1 choice-block tests,
  but the spec text governs; cite §5.1's exact sentence in the inventory.
- **P0.2** Confirm the expected-no-byte-change invariant holds at baseline:
  `sha256sum` of both vector-file copies == `542cd492…`, and
  `TestDescriptorSeamSyswClass` still counts 4 (`wantSyswClass`,
  `nonstandard/descriptor_seam_test.go:377-391`).
- **P0 gate:** the inventory table exists with every member naming its
  flipping task; both shas verified; full baseline suites green on all three
  repos (engrave `cargo nextest run --locked` with `ME_REQUIRE_GO=1` and a Go
  toolchain so `cross_lang` RUNS; fork `go test ./...` with
  `scripts/gui-shard-test.sh ./gui/ 24`; clippy on BOTH nightly and the
  CI-pinned **1.85.0** — the F-430 lesson is a plan gate now — plus
  `cargo fmt --check`).

## P1 — the classifier arm, Rust first (§5.2's predicate)

- **P1.1** `mnemonic_engrave::sysw::classify` (`crates/me-cli/src/sysw/mod.rs:205`)
  gains the Descriptor arm. The predicate, verbatim from §5.2 and implemented
  ONCE by delegating to the shipped S1 code: *a record is `ClassDescriptor`
  iff it parses under §4's cascade and matches §4.7's grammar — the seven
  forms; conjunct 1's md1-path widening does not apply* (i.e.
  `admit::admit(d, Path::Descriptor)` semantics, `descriptor/admit.rs`).
  Ordering within `classify`: the descriptor arm runs AFTER every existing
  arm (a record that classifies as anything else today must still win —
  invariant 2), and its cost is bounded (the cascade on a non-descriptor
  record fails at the gate cheaply; measure, don't assume — record the
  classify-time delta over the record corpus in the report).
- **P1.2** Tests from §7's vectors: the 4 `sysw_class: Descriptor` rows
  classify `Descriptor`; every OTHER vector row (67) does NOT; every
  pre-existing record-corpus fixture classifies UNCHANGED (the negative
  sweep — enumerate the corpus from the existing classify tests, assert
  equality against a pre-S2 capture, and commit the capture as the test's
  fixture so the diff is reviewable).
- **P1.3** `crates/me-cli/src/sysw/expect.rs`'s module doc (lines 20-32)
  states `Class::Descriptor` is never produced by `classify` — that sentence
  becomes false in P1.1. Update the doc to the new truth. The `--expect`
  VOCABULARY does not change in S2 (nothing in §11 requires it); the doc
  records that as a decision with this plan as its cite. R0 may challenge.
- **P1 gate:** full engrave suite; the negative sweep green; the classify
  cost measurement in the report; proportional review only if R0 flagged
  P1 as risky (default: fold into P2's review).

## P2 — host packing: `--as descriptor` goes live

- **P2.1** `DESCRIPTOR_PATH_SHIPPED` → `true`, and `descriptor_follower`
  packs §5.2's record at the marked site
  (`crates/me-cli/src/descriptor/as_flag.rs:126-138`): canonical
  re-encode (`Descriptor::encode()` semantics as shipped in S1's cascade
  re-encoder) as ONE record of `Class::Descriptor`, admission (§4.7,
  `Path::Descriptor`) FIRST — conjunct 1's `multi` refusal stays permanent
  and is never dressed as a wait (the sentence already in the code's
  comment). §5.4's identification block prints for this path exactly as §5.4
  specifies (it is path-independent; the S1 implementation already computes
  it before the `--as` fork — verify, don't assume, and cite the call site
  in the report).
- **P2.2** The P0 inventory executes: window refusal retired per inventory,
  choice block loses `(not available in this build)` on `descriptor`, clap
  help un-marks (M1's conditional), §11 item 5's five-case matrix updates to
  the full-build truth table (both `--as` values carry → the omitted-`--as`
  choice block still exits 2; explicit `--as descriptor` on an inadmissible
  input still gets the admission refusal, never a window text). Every flip
  lands with its test in the same commit.
- **P2.3** §11 item 1, host half: for each of the four formats, `--as
  descriptor` produces a container `me sysw show` reports as ONE `Descriptor`
  record; the record string round-trips through P1's classifier
  (`classify(packed) == Descriptor`) — the host-side fixed point.
- **P2.4** §11 item 4's `--as descriptor`-only §6 rows get their named tests
  (the S2 set that S1 recorded as EMPTY-because-parked; enumerate from the
  P0 inventory). Test-file row-count assertion updates to P0's measured
  number.
- **P2 gate:** full engrave suites (both clippy toolchains, fmt,
  `ME_REQUIRE_GO=1`); zero `#[ignore]`; propagation sweep whole-repo (the
  S3-parked phrasings must survive ONLY in `design/agent-reports/` and
  historical spec text per P0's inventory); staleness re-check; proportional
  opus review over P1+P2 before the Go port starts (Rust is the primary —
  the port must not begin from unreviewed semantics).

## P3 — the Go port: classify, consume, display (fork)

- **P3.1** `classifyConstellation` (`sysw/classify.go:34-58`) gains the
  descriptor arm calling `nonstandard.OutputDescriptor` per §5.2, LAST in
  the arm order (invariant 2 device-side). `ClassDescriptor` exists
  (`sysw/record.go:32`); no wire change (class is runtime-derived — recon
  Q1).
- **P3.2** The sysw-session consumer: `gui/sysw_session.go` routes
  `ClassDescriptor` per the `ClassMt` five-touch-point checklist (recon Q3;
  3 of 5 exist — admit table `gui/sysw_admit.go:32-52` and
  `DescriptorScreen` `gui/gui.go:3070-3189` are live code the NFC door
  already exercises). The never-executed admission cells (§9 item 2:
  `admits(progWalletPolicy, ClassDescriptor) == true` → rendered screen)
  execute for the FIRST TIME here in the simulator: a named sim-walk test
  drives a packed S2 container to a rendered `DescriptorScreen` — **a gate
  that has never run is a hypothesis, so running it is a P3 gate, not P5
  polish.**
- **P3.3** Un-skip `TestDescriptorSeamSyswClass`: assert the 4 rows answer
  `Descriptor` AND every `device_admits: false` row does not; keep the
  count-guard. Vector file bytes unchanged (invariant 1).
- **P3.4** F-426: the one `ypubVer` case in `bip380/bip380.go`'s
  classification switch (`bip380/bip380.go:442-455`; declared
  `bip380/bip380.go:433-441`, normalised `bip380/bip380.go:456-462` —
  recon Q5), with a test per direction (bare `ypub` classifies
  and normalises to `xpub`; the host's five-version admission is UNCHANGED
  in S2 — the convergence widening is F-426's later cycle, say so in the
  test's comment).
- **P3 gate:** `go test ./...` + gui shard + vet + gofmt; the sim-walk
  renders; engrave's `ME_REQUIRE_GO=1` suite green against the updated fork
  worktree; proportional opus review of the port against P1/P2's reviewed
  semantics (brief: predicate parity, arm order, the first-execution walk).

## P4 — F-423: `bundlePlatePlan` packs plates

- **P4.1** MEASURE first (the entry's own instruction): from
  `engrave.Params` (`engrave/engrave.go:38-44`) and the shipped font
  metrics, compute how many md1 strings fit one plate side with strings as
  visually distinct units. The measurement is a committed scratch program's
  output pasted into the report, not an estimate.
- **P4.2** Implement the denser plan in `bundlePlatePlan`
  (`gui/bundle_flow.go:384-402`), update
  `TestBundlePlanSingleMD1OnePlate`'s siblings to pin the new arithmetic,
  and update the spec's §5.5 plate cell + the walk-log correction to the
  measured counts (spec-touch = its own commit, marked amendment).
- **P4 gate:** fork suites; the plan's arithmetic pinned by tests; **NO
  physical cut** — the single-character test-plate protocol and any real
  engraving are the operator's, listed in P5's handover.

## P5 — records, ship, and the operator-gated tail

- **P5.1** FOLLOWUPS reconciliation: F-418 (S2 built — entry updated to
  point at the acceptance handover), F-426 → resolved-in-build, F-423 →
  resolved-in-build-pending-physical-validation, F-428 per invariant 1's
  outcome, F-429/F-427 unchanged. CHANGELOG Unreleased grows the S2 entry.
  Continuity + memory.
- **P5.2** Mandatory post-implementation adversarial EXECUTION review over
  the whole S2 diff (both repos), opus; walk journeys re-run; the §5.2
  canonical-record round-trip (pack → classify → decode → same wallet)
  hammered the way fold-1's reviewer hammered `derive.rs`.
- **P5.3** Merges + pushes: engrave via `scripts/push-via-staging.sh`; fork
  via its plain push to `main` ONLY after the review closes green — the
  device boots `main`, so an unreviewed `main` is an unreviewed flash
  candidate.
- **P5.4** **Operator handover, explicitly NOT autonomous:** flash the S2
  firmware (`sh2-flash`), §11 item 6 (a `ClassDescriptor` record loaded and
  DISPLAYED — the operator's eyes are the instrument), F-423's single-char
  test plate then a real cut, and the §9 item 2 cells confirmed on hardware.
  S2 is "shipped" when item 6 is, and not before.

## Review cadence and scale

R0 loop on this plan to 0C/0I (opus; the operator's overnight scale stands:
5 rounds expected-good, fable at 15, hard stop 25). Implementation: ONE
implementer per phase, worktrees (`impl/descriptor-s2` on engrave,
`s2/descriptor-arm` on the fork), controller folds small fixes inline.
Persist-before-fold, agent-persisted reports to `design/agent-reports/`,
propagation sweeps whole-repo + fork + generators (the S1+S3 lesson is
standing), and every fold re-runs the build gate on BOTH clippy toolchains.

## What the build gate does not cover here

This plan carries no fenced code blocks; its executable content is commands,
file:line citations, and named tests. The staleness script covers the
citations mechanically against the three baselines above; the R0 reviewer
EXECUTES the commands and resolves the paths (the S1 plan's reviewers found
four of five blockers exactly that way). Facts about spec §5.1/§5.2/§11
sentence content are load-bearing and gated by nothing — the reviewer reads
them against this plan's claims.
