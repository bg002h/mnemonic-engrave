# IMPLEMENTATION_PLAN_descriptor_input_S2 — `--as descriptor` end to end

**Status: DRAFT — R0 round 1 (RED, 4C/6I/7M/3N,
`design/agent-reports/R0-S2-plan-r1.md`) folded 2026-08-29.** Single author
per the R0 gate; this plan binds S2 of `SPEC_descriptor_input.md` (GREEN
2026-08-28, amendments through the S1+S3 cycle). S1+S3 shipped 2026-08-29
(engrave `f244442`, records through `4646fa2`); the SH2 is back on the bench
and boots fork `main` @ `a5e29b4` (flashed 2026-08-29), which un-parks
F-418's condition.

**Plan baseline revs** (for `scripts/plan-staleness-check.sh`):
mnemonic-engrave `4646fa2` · seedhammer fork `a5e29b4` ·
descriptor-mnemonic `6c4a56fd` (cited only for context; S2 does not change it).

**Recon ground truth:** `design/agent-reports/RECON-S2-fork-seam.md`
(fork half, all file:line cites verified at `a5e29b4`) plus the host-side
reads recorded in this plan's own citations, plus R0 r1's measured table
("Verified TRUE — do not re-derive these").

## What S2 is, in one paragraph

`me sysw pack --as descriptor` stops answering with §5.1's window refusal and
packs §5.2's **canonical re-encoded descriptor** as one record of class
`Descriptor`; the pack path's §5.1 gate is restructured to key on
**identification, not classification failure**, so the choice block survives
the new arm (r1 C1); the device learns to **classify** that record with the
SAME predicate as the host (`sysw.Classify` descriptor arm — a real §4.7
port, Rust-first per §3/§5.2 and the standing Rust-primary rule) and to
**display** it (`walletPolicyFlow` consumes `ClassDescriptor` and routes to
the existing `DescriptorScreen`); `me sysw show` grows the per-record
confirmation line §11 item 1 assumes (r1 C4); the fork batches F-426 (the
`ypub` classification case) and F-423 (denser `bundlePlatePlan`, measurement
first) into the same firmware build; and §11 items 1 and 4's `--as
descriptor` rows close at the desk while **item 6 (a `ClassDescriptor`
record displayed on the real device) and every flash remain operator-gated**
— S2 is not "shipped" until the operator has seen the screen.

## The two invariants that bound every phase

1. **The vector file changes bytes EXACTLY ONCE in S2** (rewritten at the
   r1 fold; the draft expected zero and r1 I4 proved one is forced). The
   single regeneration is owned by P2.6 (engrave half) and mirrored
   byte-identically by P3.3 (fork half), and carries, together:
   - the NEW uncarried-witness row — `wsh(multi(2,…/0/*))`, §5.4's own
     named witness: `host_admits: false` (conjunct 1's permanent `multi`
     refusal), `md1_admits: false` (fixed `/0/*` use-site path, F-417),
     `format` keeps its cascade branch — restoring §11 item 5's case 3
     after the shipped witness (`md1-split/fixed-index`) flips to CARRIED
     (r1 I4);
   - RETIREMENT of the 4-row `sysw_class` sample column and its header
     block, superseded by P1.2/P3.3's exhaustive derived classification
     assertion (r1 C2/M7 — the column was a sample the draft misread as a
     population, and its input-vs-canonical basis was ambiguous);
   - the `panic:parse` row's `device_probe` retirement: P3.1's convergence
     fix makes the Go parser error cleanly, so the row gains its measured
     `device_admits` boolean and the "must NOT feed" harness rule for it is
     dropped from the header (the `panic:encode` rows are UNCHANGED — S2
     never touches `Encode`, and their scan-door behaviour stands);
   - F-428's citation fixes (the stale `:151` cite → the measured `nonstandard/parse.go:158`, in the
     two `source` annotations and the generator) — F-428's own entry names
     "the next vector-file byte change — realistically the S2/F-426 batch"
     as its owning phase, and this is that change.
   `scripts/descriptor-seam-vectors/rows.py` and the JSON regenerate
   together; ONE sha bump per repo; the count guards
   (`nonstandard/descriptor_seam_test.go:74-77`) update to the regenerated
   populations. Sequencing: the engrave copy updates at P2.6 (its Rust
   assertions land with it); the fork copy updates at P3.3 with the fork's
   test changes; between those commits the two copies transiently differ,
   and P3's gate re-asserts byte-equality of both copies and both pins.
2. **The record surface for non-descriptor records is untouched — verified
   end to end, not just at classify** (strengthened at the r1 fold). An
   input that classified as Mnemonic/Codex32/MdMk/Mt/FreeText before S2
   classifies identically after (P1.2's capture sweep), AND the full
   `me sysw pack` surface over the existing corpus — exit codes, stderr,
   container bytes — is unchanged (P1.0's matrix witness + full suites).
   Arm order alone cannot give this (r1 C1: order protects records an
   earlier arm matched, not records that previously fell through to
   `Unknown`), which is why P1.0 exists.

## P0 — plan gates and the S2 flip inventory

- **P0.1** Machine-count the flip set before any code: every test, §6 row,
  and help/window string that asserts the S3-parked build state. Known
  members (corrected per r1 I2 — the draft listed one real
  `DESCRIPTOR_PATH_SHIPPED` consumer and one non-consumer). The measured
  consumer set (`grep -rn 'DESCRIPTOR_PATH_SHIPPED' crates/`):
  - `crates/me-cli/src/descriptor/gate.rs:42` — the declaration (P2.1
    flips it);
  - `crates/me-cli/src/descriptor/gate.rs:223` — `carriage()`. **Flips an
    EXIT CODE, not a string:** `descriptor_carries` becomes true for every
    admitted, md1-unrepresentable input, so the four `md1-split/*` rows
    move from `EXIT_REFUSED` (3, §5.3's refusal) to `EXIT_USAGE` (2, the
    choice block). §11 item 5's case 3 loses its witness here — replaced by
    invariant 1's new row — and §6's `md1-fixed-index` / `md1-no-wildcard`
    rows survive with changed reachability;
  - `crates/me-cli/src/descriptor/gate.rs:273` — `window_remedy()`, §5.3's
    window SUBSTITUTION: the remedy sentence inside two §6 rows flips from
    "The scannable-plate path is not in this build…" to "Use `--as
    descriptor`, which carries `/0/*` exactly." Those two row tests are
    pinned verbatim (`crates/me-cli/tests/descriptor_refusals.rs:11-16`).
    **This is a DIFFERENT "two variants" from §5.1's window refusal** (the
    md1-representable vs (a)/(a″)-shaped alternatives in
    `identify::window_refusal`) — the inventory carries BOTH collections,
    named separately;
  - `crates/me-cli/src/descriptor/gate.rs:566` — `choice_block()`'s
    `descriptor_head` (the `(not available in this build)` marking, plus
    the M2 padding defect — see P2.2);
  - `crates/me-cli/src/main.rs:365` — the clap help conditional.
  (`crates/me-cli/src/descriptor/as_flag.rs:133` mentions the constant in a
  COMMENT only; the stub at `crates/me-cli/src/descriptor/as_flag.rs:126-138` is P2.1's edit site, not a
  flip-set member.) Also members: `Row::WindowNotInBuild`
  (`crates/me-cli/src/descriptor/refusal.rs:43`), §11 item 5's sibling
  cases (W4/W11), and the M1 build-marked clap help twin. Output: a
  checked-in inventory table in this plan (folded at P0's close) stating,
  per member, its post-S2 behaviour and which P-task flips it. **The §6
  row-test count after S2 is a MEASURED number recorded here** (expected 35
  = 36 − `WindowNotInBuild`, but the measurement governs).
  `WindowNotInBuild` itself: §5.1 keeps the row REACHABLE only if some
  build state still refuses (none does post-S2 — r1 verified §5.1's text
  supports retirement); expected disposition is that the row and its tests
  are retired to the §5.1 choice-block tests, but the spec text governs;
  cite §5.1's exact sentence in the inventory.
- **P0.2** Confirm the byte-change baseline: `sha256sum` of both vector-file
  copies == `542cd492…` (r1 verified both PASS at baseline), and
  `TestDescriptorSeamSyswClass` still counts 4 (`wantSyswClass`,
  `nonstandard/descriptor_seam_test.go:377-391`) — this is the BEFORE
  measurement for invariant 1's single regeneration.
- **P0.3** (new at the r1 fold, M6) Commit `scripts/lint-gate.sh`: clippy on
  BOTH the CI-pinned 1.85.0 and nightly, plus `cargo fmt --check`, as one
  command. F-430's entry says the durable fix is a gate command, not a plan
  clause a human must remember per fold; every later gate in this plan says
  "lint-gate" and means this script. F-430 reconciles in P5.1 as
  resolved-by-script.
- **P0 gate:** the inventory table exists with every member naming its
  flipping task; both shas verified; full baseline suites green on all three
  repos (engrave `cargo nextest run --locked` with `ME_REQUIRE_GO=1` and a Go
  toolchain so `cross_lang` RUNS; fork `go test ./...` with
  `scripts/gui-shard-test.sh ./gui/ 24`; lint-gate).

## P1 — the pack-path gate survives the arm, then the arm (Rust first)

- **P1.0** (new at the r1 fold — C1, and it lands BEFORE the arm) `me sysw
  pack`'s `--as`-omitted path consults the descriptor gate ONLY inside the
  classification-failure branch today: `crates/me-cli/src/main.rs:1504`
  (`admit_check` err) → `crates/me-cli/src/main.rs:1516` (`descriptor::consult`), and
  `admit_check` (`crates/me-cli/src/sysw/mod.rs:403-410`) errs only on
  `Class::Unknown`. The moment classify gains a Descriptor arm, an admitted
  single-line descriptor classifies, the branch is skipped, and §5.4's
  identification block, §5.1's choice block and every omitted-`--as` §6
  refusal silently die — the input packs RAW at exit 0 (r1 C1, measured).
  **The restructure: `consult` runs BEFORE `admit_check` on the
  `--as`-omitted path** — identification, not classification failure, is
  what opens the gate. If `consult` identifies a descriptor, its outcome
  (choice block exit 2 / §6 refusal) applies regardless of classifiability;
  if it answers not-a-descriptor, `admit_check` proceeds as today.
  Pre-arm this is behaviour-preserving (no document both identifies as a
  descriptor and fully classifies — proven by the corpus capture, not
  assumed), so P1.0 lands as its own commit with the full suite green.
  **Witness at EVERY P1/P2 commit boundary:** `item_5_the_five_case_matrix`
  (`crates/me-cli/tests/descriptor_refusals.rs:829-849`) — the exact test
  r1 measured going red under an arm without this restructure.
  The mirrored collision question (can `identify` claim a document whose
  records classify as an existing class?) gets its own negative sweep:
  `consult` answers not-a-descriptor on every non-descriptor corpus
  fixture, asserted against the same committed capture as P1.2.
- **P1.1** `mnemonic_engrave::sysw::classify` (`crates/me-cli/src/sysw/mod.rs:205`)
  gains the Descriptor arm. The predicate, verbatim from §5.2 and implemented
  ONCE by delegating to the shipped S1 code: *a record is `ClassDescriptor`
  iff it parses under §4's cascade and matches §4.7's grammar — the seven
  forms; conjunct 1's md1-path widening does not apply* — i.e.
  `descriptor::host_admits` (`crates/me-cli/src/descriptor/admit.rs:418`),
  which r1 verified IS the predicate verbatim, public, and asserted row-by-row
  against all 71 vectors (`crates/me-cli/tests/descriptor_seam.rs:584`).
  **Where the arm is consulted from is a decision, not an accident** (r1's
  fold question): classify is the shared admission predicate for
  `admit_check` (pack), `--expect` resolution, and `show` — the arm serves
  all of them, and the §5.1 gate is protected by P1.0's ordering, never by
  classify staying ignorant. Ordering within `classify`: the descriptor arm
  runs AFTER every existing arm (invariant 2), and its cost is bounded (the
  cascade on a non-descriptor record fails at the gate cheaply; measure,
  don't assume — record the classify-time delta over the record corpus in
  the report).
- **P1.2** Tests (rewritten at the r1 fold — C2 proved the draft's 4-vs-67
  split unsatisfiable: §5.2's predicate is TRUE on 19 of 71 rows, and 3 of
  the 4 `sysw_class` sample rows' inputs are not even single lines).
  **The classification assertion is DERIVED from the existing columns and
  asserted exhaustively, per row, in both languages:**
  - for every row whose input is a SINGLE LINE (a record cannot contain
    `\n` — `sysw/open.go:67-74` splits on LF):
    `classify(input) == Descriptor` iff `host_admits`, and `== Unknown`
    otherwise (exact equality — this is also the empirical answer to
    "can a descriptor-shaped string collide with another class", per row);
  - for every `host_admits: true` row: `classify(canonical) == Descriptor`
    (the canonical is always a single line, and it is what P2.1 packs).
  The Rust side lands here; the Go side is P3.3's un-skip asserting the
  SAME derived rule — parity is structural, with no hand-stated per-row
  class values to drift (r1 M7's input-vs-canonical ambiguity dissolves:
  both bases are asserted, separately). Plus the negative sweep: every
  pre-existing record-corpus fixture classifies UNCHANGED (enumerate the
  corpus from the existing classify tests, assert equality against a
  pre-S2 capture, and commit the capture as the test's fixture so the diff
  is reviewable).
- **P1.3** (re-ruled at the r1 fold — I1) `crates/me-cli/src/sysw/expect.rs`'s
  module doc (lines 20-32) states `Class::Descriptor` is never produced by
  `classify` — false after P1.1; update it. And the draft's "vocabulary
  unchanged, nothing else" ruling shipped a guaranteed false refusal:
  `Kind::Descriptor` resolves by card HRP alone
  (`crates/me-cli/src/sysw/expect.rs:112`), so `me sysw pack --as descriptor
  --expect descriptor` — the natural belt-and-braces invocation — would
  refuse the very record it just built, 100% reproducibly, on the funds
  path. **The ruling: `Kind::Descriptor` WIDENS in the same commit as the
  arm** — satisfied by an md1 descriptor card (`card_hrp == 'd'`) OR a
  record classifying `Class::Descriptor`; its description
  (`crates/me-cli/src/sysw/expect.rs:96`) updates from "an md1 descriptor
  card" to name both. The VOCABULARY still does not change (no new word).
  Tests: `--as descriptor --expect descriptor` exits 0; `--as md1 --expect
  descriptor` still exits 0; `--expect descriptor` against a
  mnemonic-only container still refuses with the kind's description.
- **P1 gate:** full engrave suite + lint-gate; the matrix witness green at
  every commit boundary; the negative sweeps (classify capture + consult
  not-a-descriptor) green; the classify cost measurement in the report;
  proportional review only if R0 flagged P1 as risky (default: fold into
  P2's review).

## P2 — host packing: `--as descriptor` goes live

- **P2.1** `DESCRIPTOR_PATH_SHIPPED` → `true`, and `descriptor_follower`
  packs §5.2's record at the marked site
  (`crates/me-cli/src/descriptor/as_flag.rs:126-138`): canonical
  re-encode (`Descriptor::encode()` semantics as shipped in S1's cascade
  re-encoder) as ONE record of `Class::Descriptor`, admission (§4.7,
  `Path::Descriptor`) FIRST — conjunct 1's `multi` refusal stays permanent
  and is never dressed as a wait (the sentence already in the code's
  comment). §5.4's identification block prints for this path exactly as
  §5.4 specifies — **verified at r1 (N2), no work required:** `as_flag::run`
  builds the block (`crates/me-cli/src/descriptor/as_flag.rs:79`) BEFORE
  the `match` that selects the follower, so it is already path-independent.
- **P2.2** The P0 inventory executes: window refusal retired per inventory,
  choice block loses `(not available in this build)` on `descriptor`, clap
  help un-marks (M1's conditional), `window_remedy()`'s two §6 row tests
  update to the shipped remedy sentence (P0.1's second "two variants"
  collection), and §11 item 5's matrix updates to the full-build truth
  table: both `--as` values carry → the omitted-`--as` choice block still
  exits 2; explicit `--as descriptor` on an inadmissible input still gets
  the admission refusal, never a window text; case 3's witness becomes
  invariant 1's new row (`wsh(multi(2,…/0/*))` — admitted by NEITHER
  carrier, exit 3). **The shipped choice block also gets M2's fix:** the
  descriptor head at `crates/me-cli/src/descriptor/gate.rs:566` renders
  unpadded with a trailing `\n` while §5.1's NORMATIVE block puts the
  description inline on a padded line — fix the padding, and add the
  verbatim block test that does not exist today (`grep SCANNABLE
  crates/me-cli/tests/` = 0 hits at r1). Every flip lands with its test in
  the same commit.
- **P2.3** §11 item 1, host half: for each of the four formats, `--as
  descriptor` produces a container `me sysw show` reports as ONE `Descriptor`
  record (the surface P2.5 builds); the packed record's **record-classification
  check** (`classify(packed) == Descriptor` — named honestly per r1 N1: it
  is not a fixed point; §7 requirement 4's real fixed point,
  `encode(parse(canonical)) == canonical`, is already asserted by the S1
  seam tests) holds for all four. §5.3(b)'s label warning fires on any
  `Decision::Pack` (`crates/me-cli/src/descriptor/as_flag.rs:88-94`), so
  the JSON exemplar newly prints it on this path — correct per §5.5
  ("carries a label | text only, dropped"), and item 1's test names the
  line so it is expected output, not noise (r1 N3).
- **P2.4** (premise corrected at the r1 fold — I3) The draft told the
  implementer to add "the S2 set that S1 recorded as EMPTY-because-parked";
  the record says the opposite (`crates/me-cli/tests/descriptor_refusals.rs:4-5`:
  *"All 36 rows, and the S2-parked set is EMPTY — every §6 trigger is
  reachable in this build"*). **There are no `--as descriptor`-only §6 rows
  to add; S2 SUBTRACTS one.** The task is: retire `Row::WindowNotInBuild`
  and its tests per P0.1's inventory, update the row-count + set-equality
  assertions (`crates/me-cli/tests/descriptor_refusals.rs:126-141`, against
  `Row::ALL` at `crates/me-cli/src/descriptor/refusal.rs:74-111`) to P0's
  measured number, and invent nothing.
- **P2.5** (new at the r1 fold — C4) `me sysw show` today prints per-record
  confirmation lines ONLY for `Class::MdMk` (`crates/me-cli/src/main.rs:2062-2071`)
  and `Class::Mt` (`crates/me-cli/src/main.rs:2082-2085`); a Descriptor record prints nothing
  (r1 measured), so §11 item 1's "show reports ONE Descriptor record" names
  a surface that does not exist. **Build it, additively:** for each public
  record classifying `Class::Descriptor`, one confirmation block following
  the mdmk/mt pattern, reusing `identify::block` on the re-parsed record so
  the vocabulary is §5.4's (classification proved it parses; if a record
  does not classify, no line — output for every existing container is
  byte-identical). Tests: the four §11 item 1 containers each report
  exactly ONE descriptor record; a no-descriptor container's output is
  unchanged against a committed capture.
- **P2.6** (new at the r1 fold — invariant 1's engrave half) The single
  vector-file regeneration: new witness row, `sysw_class` column
  retirement, `panic:parse` `device_probe` retirement, F-428 citation
  fixes; `scripts/descriptor-seam-vectors/rows.py` + JSON together, engrave
  copy + pin bumped, engrave-side seam assertions updated in the same
  commit. (The fork copy follows byte-identically at P3.3.)
- **P2.7** (new at the r1 fold — I3; spec touches are their own commits,
  marked amendments) The host-truth spec amendments S2 forces: §6's table
  (row retirement, the two remedy-sentence rows, the md1-split rows'
  reachability), §11 item 5 (the sibling "not in this build" clause dies;
  case 3's witness is the new row), §11 item 1 (names P2.5's surface),
  §5.2's Go-arm sentence (r1 C3: "calls `nonstandard.OutputDescriptor`"
  contradicts the predicate sentence in the same section — it becomes
  "parses via `nonstandard.OutputDescriptor` AND enforces §4.7's
  conjuncts", the parity the Rust-primary rule mandates), §5.5's "needs a
  firmware change to be readable | yes, §5.2" row, and §8's "S2 is parked"
  sentence. Enumerated in P0.1's inventory; a diff falsifies text it never
  touches, so the propagation sweep runs over the SPEC too.
- **P2 gate:** full engrave suites (lint-gate, `ME_REQUIRE_GO=1`);
  zero `#[ignore]`; the matrix witness green; propagation sweep whole-repo
  including the spec (the S3-parked phrasings must survive ONLY in
  `design/agent-reports/` and historical review text per P0's inventory);
  staleness re-check; proportional opus review over P1+P2 before the Go
  port starts (Rust is the primary — the port must not begin from
  unreviewed semantics).

## P3 — the Go port: classify, consume, display (fork)

- **P3.1** (rewritten at the r1 fold — C3 + I6) `classifyConstellation`
  (`sysw/classify.go:34-58`) gains the descriptor arm, LAST in the arm
  order (invariant 2 device-side). **The arm is a faithful port of §5.2's
  predicate, not a call to the scan-door parser:** r1 measured
  `nonstandard.OutputDescriptor` alone answering TRUE on 17 rows the host
  REFUSES — anyone-can-spend `sortedmulti(0,…)`, `k > n`, 21 keys,
  mixed-network, hardened use-sites, conjunct-8 key-identity failures —
  every one of which would have reached a program and a screen through the
  already-live admission cells. So the arm = parse via
  `nonstandard.OutputDescriptor` + a port of §4.7's conjuncts over the
  parsed descriptor (the Rust-primary rule makes this mandatory, and P3's
  parity assertion makes it structural). `ClassDescriptor` exists
  (`sysw/record.go:32`); no wire change (class is runtime-derived — recon
  Q1). **Crash containment, because `sysw.Classify` runs on EVERY record of
  every loaded payload** (`gui/sysw_session.go:111`; r1 I6 measured two
  panics reachable from a record string): the short-fingerprint parse panic
  (`nonstandard/parse.go:158`, §4.2 defect 4) is fixed as RUST-CONVERGENCE
  — the Rust check the standing rule requires is already done, r1 measured
  the host refusing the same input cleanly — with a bounds check and a
  clean error, which is what retires the vector row's `device_probe` in
  invariant 1's regeneration; the titled zero-key `Encode` panic
  (`Name: my wallet`) is refused by the ported conjuncts before any screen
  can encode it; `recover` in the arm failing closed to `ClassUnknown` is
  optional hardening at the implementer's discretion (the TinyGo gate must
  stay green either way). Inadmissible descriptor-shaped records therefore
  classify `ClassUnknown` and go INERT — the existing contract for unknown
  records (recon §2); a load-time warning surface for inert records is
  explicitly out of S2's scope. Named unit tests: both measured panic
  inputs classify `ClassUnknown` without crashing.
- **P3.2** (consumer named at the r1 fold — I5) **The S2 consumer is
  `progWalletPolicy` / `walletPolicyFlow` (`gui/wallet_policy.go:33`):** the
  program is "engrave a wallet policy that came from OUTSIDE this device,
  with proof of which wallet it is", and a sysw Descriptor record IS an
  outside wallet policy. Its payload door today offers only `ClassMDMK`
  (`gui/wallet_policy.go:39`); it learns to offer a `ClassDescriptor`
  record as the policy source (exact offer mechanics are the implementer's
  — `syswOffer` is single-class, so this may be a second offer at the same
  door; R0 checks the fit), and an accepted record routes: re-parse
  (classification proved it parses) → `*bip380.Descriptor` →
  `descriptorFlow` (`gui/gui.go:2727-2741`) → the existing
  `DescriptorScreen` (`gui/gui.go:3070-3189`) — display and engrave, the
  md1-card path unchanged. A classified record is §4.7-admitted, so the
  screen's `Encode()` cannot hit the `panic:encode` class (P3.1). The
  new-class checklist is the recon's SIX touch points (r1 M1 corrected the
  draft's five): constant, admission table, `txClassName` arm (all three
  exist), classifier arm (P3.1), this consumer, and **registration in
  `gui/sysw_admit_oracle_test.go`'s `syswConsumers` table** — the oracle
  fails until the call site is named. **The other two admission cells
  (`progBundle`, `progMultisig` — `gui/sysw_admit.go:37,39`) stay declared
  and INERT in S2:** no consumer, records unoffered, filed as a new
  follow-up with an owning future cycle at P5.1; §9 item 2's spec claim is
  amended accordingly (P3.5). The never-executed cell that IS built —
  `admits(progWalletPolicy, ClassDescriptor)` → rendered screen — executes
  for the FIRST TIME in the simulator: a named sim-walk test drives a
  packed S2 container to a rendered `DescriptorScreen` (r1 verified the
  headless harness: `gui/transaction_walk_test.go:28-50` is the template).
  **A gate that has never run is a hypothesis, so running it is a P3 gate,
  not P5 polish.**
- **P3.3** (rewritten at the r1 fold — C2/C3/M7 fork half) The fork's
  vector copy updates to P2.6's exact bytes (byte-equality + pin asserted
  in this commit and re-asserted at the P3 gate).
  `TestDescriptorSeamSyswClass` un-skips and becomes the exhaustive derived
  assertion, SAME rule as P1.2: for every single-line input,
  `sysw.Classify(input) == ClassDescriptor` iff `host_admits`, else
  `ClassUnknown`; for every `host_admits: true` row,
  `sysw.Classify(canonical) == ClassDescriptor`. No sampled column, no
  hand-stated values — a Go/Rust divergence anywhere in the file goes red
  on this test (the r1 C3 divergence, 17 rows, is exactly what it would
  have caught). Count guards update to the regenerated populations.
- **P3.4** F-426: the one `ypubVer` case in `bip380/bip380.go`'s
  classification switch (`bip380/bip380.go:442-455`; declared
  `bip380/bip380.go:433-441`, normalised `bip380/bip380.go:456-462` —
  recon Q5), with a test per direction (bare `ypub` classifies
  and normalises to `xpub`; the host's five-version admission is UNCHANGED
  in S2 — the convergence widening is F-426's later cycle, say so in the
  test's comment).
- **P3.5** (new at the r1 fold — I3 device half; own commit, marked
  amendment) §9 item 2's "untested by construction" claim updates to the
  S2 truth: one cell executed (walletPolicy, by P3.2's walk), two declared
  and inert with a named follow-up.
- **P3 gate:** `go test ./...` + gui shard + vet + gofmt + **the TinyGo
  device build** (r1 M3 — the command CI runs,
  `tinygo build -size full -print-stacks -o /dev/null -target pico-plus2
  -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`,
  runs locally under `nix develop`; an unrun gate is a hypothesis, and
  P5.3's push is one phase too late to learn the port broke the image);
  the sim-walk renders; vector-copy byte-equality + both pins; engrave's
  `ME_REQUIRE_GO=1` suite green against the updated fork worktree;
  proportional opus review of the port against P1/P2's reviewed semantics
  (brief: predicate parity including the conjunct port, arm order, the
  containment fixes, the first-execution walk).

## P4 — F-423: `bundlePlatePlan` packs plates

- **P4.1** (measurement mechanics corrected at the r1 fold — M4) MEASURE
  first, with the fit mechanism the fork actually has: a committed scratch
  program that TRIAL-FITS via `backup.EngraveText(params, plate)` →
  `toPlate(plan, params)` (which errors when a layout does not fit — the
  loop `validateDescriptor` itself uses, `gui/gui.go:722-736`), against
  `engrave.Params` (`engrave/engrave.go:38-44`), the single plate size
  (`backup/backup.go:77`, 85 mm square) and the shipped font metrics
  (`backup/backup.go:58-63` — font size is fixed unless a caller sets it).
  Output pasted into the report, not an estimate. **Ruling: `FontSize`
  reduction is NOT permitted in S2** — the engraving-font standing rules'
  2-stroke-width minimum feature makes the shipped size the floor, and no
  P4 gate could catch a legibility regression (no physical cut). If the
  measurement says one md1 string per plate at the shipped font, F-423
  closes as measured-no-gain and P4.2 is SKIPPED — the measurement decides,
  and that outcome is a valid close.
- **P4.2** Implement the denser plan in `bundlePlatePlan`
  (`gui/bundle_flow.go:384-402`) **packing WITHIN a card only** (r1 M5:
  `bundlePlate`'s card-scoped fields — `cardIdx`, `cardTotal`, `label`,
  `kind`, `gui/bundle_flow.go:371-379` — drive the "Card X of Y" guidance,
  the abort warning and `bundlePlateMark`; packing across cards makes every
  one ill-defined and could co-locate a cardMS1 string with a marked md1
  string. The ms1-marking half is capped by the 2026-08-27 secret-handling
  ruling and does not gate; the operator-guidance half is plain
  correctness). Strings stay visually distinct units. Update
  `TestBundlePlanSingleMD1OnePlate`'s siblings to pin the new arithmetic,
  and update the spec's §5.5 plate cell + the walk-log correction to the
  measured counts (spec-touch = its own commit, marked amendment).
- **P4 gate:** fork suites + TinyGo build; the plan's arithmetic pinned by
  tests; **NO physical cut** — the single-character test-plate protocol and
  any real engraving are the operator's, listed in P5's handover.

## P5 — records, ship, and the operator-gated tail

- **P5.1** FOLLOWUPS reconciliation: F-418 (S2 built — entry updated to
  point at the acceptance handover), F-426 → resolved-in-build, F-423 →
  per P4.1's measured outcome (resolved-in-build-pending-physical-validation,
  or closed measured-no-gain), F-428 → RESOLVED by invariant 1's
  regeneration, F-430 → RESOLVED by P0.3's lint-gate script, and a NEW
  follow-up for the two inert `ClassDescriptor` admission cells
  (`progBundle`, `progMultisig`) with an owning future cycle. CHANGELOG
  Unreleased grows the S2 entry. Continuity + memory.
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
  test plate then a real cut IF P4.2 shipped, and **the ONE built §9 item 2
  cell** — `admits(progWalletPolicy, ClassDescriptor)` → the screen —
  confirmed on hardware (r1 I5: the draft handed the operator all three
  cells, two of which have no consumer and cannot be performed). S2 is
  "shipped" when item 6 is, and not before.

## Review cadence and scale

R0 loop on this plan to 0C/0I (opus; the operator's overnight scale stands:
5 rounds expected-good, fable at 15, hard stop 25). Implementation: ONE
implementer per phase, worktrees (`impl/descriptor-s2` on engrave,
`s2/descriptor-arm` on the fork), controller folds small fixes inline.
Persist-before-fold, agent-persisted reports to `design/agent-reports/`,
propagation sweeps whole-repo + fork + generators + THE SPEC (the S1+S3
lesson is standing), and every fold re-runs lint-gate (P0.3's script).

## What the build gate does not cover here

This plan carries no fenced code blocks; its executable content is commands,
file:line citations, and named tests. The staleness script covers the
citations mechanically against the three baselines above; the R0 reviewer
EXECUTES the commands and resolves the paths (the S1 plan's reviewers found
four of five blockers exactly that way — and r1's Criticals came from
executing the pack path and the Go probe, not from reading). Facts about
spec §5.1/§5.2/§11 sentence content are load-bearing and gated by nothing —
the reviewer reads them against this plan's claims. `.py` and `.json`
citations (rows.py, the vector file) are invisible to the cite-check by its
own stated gap; the reviewer resolves those by hand.
